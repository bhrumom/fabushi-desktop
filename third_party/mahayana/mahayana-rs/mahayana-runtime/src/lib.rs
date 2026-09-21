//! Long-lived local conversation runtime used by all Mahayana frontends.

mod capability_broker;
mod conversation_actor;
mod kernel_conversation;
mod runtime_store;

use crossbeam_channel::Receiver;
use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::Sender;
use fabushi_official_miniapps::OfficialMiniAppEngine;
use fabushi_official_miniapps::app_definition;
use fabushi_official_miniapps::home_html;
use capability_broker::CapabilityBroker;
use conversation_actor::{ConversationActor, ConversationActorRegistry};
use kernel_conversation::KernelConversationProvider;
use runtime_store::{RuntimeStore, RuntimeStoreError};
use mahayana_agent::AgentBackend;
use mahayana_agent::AgentError;
use mahayana_agent_kernel_bridge::LegacyAgentKernelBridge;
use mahayana_conversation::ConversationError;
use mahayana_conversation::ConversationEventSink;
use mahayana_conversation::ConversationProvider;
use mahayana_conversation::ProviderRegistry;
use mahayana_conversation::ResolveApprovalRequest;
use mahayana_conversation::SendMessageRequest;
use mahayana_conversation::SharedConversationEventSink;
use mahayana_core::ApprovalDecision as RuntimeApprovalDecision;
use mahayana_core::ApprovalId;
use mahayana_core::CONVERSATION_SCHEMA_VERSION;
use mahayana_core::Conversation;
use mahayana_core::AskUserRequest;
use mahayana_core::ConversationId;
use mahayana_core::ExecutionRun;
use mahayana_core::HandoffIntent;
use mahayana_core::IntentId;
use mahayana_core::LogicalTurn;
use mahayana_core::MessageId;
use mahayana_core::MODEL_RUNTIME_VERSION;
use mahayana_core::MAHAYANA_AI_CONVERSATION_ID;
use mahayana_core::OperationId;
use mahayana_core::RunId;
use mahayana_core::PluginCommandDescriptor;
use mahayana_core::RUNTIME_ABI_VERSION;
use mahayana_core::RuntimeCommand;
use mahayana_core::RuntimeConfig;
use mahayana_core::RuntimeEvent;
use mahayana_core::RuntimeResponse;
use mahayana_core::RuntimeStatus;
use mahayana_core::TurnId;
use mahayana_core::TurnState;
use mahayana_core::capability::{CapabilityAvailability, CapabilityPolicyDecision, CapabilityRegistry, CapabilityRequest};
use mahayana_kernel::BackendDescriptor;
use mahayana_kernel::Capability;
use mahayana_kernel::CapabilitySet;
use mahayana_kernel::EngineBackend;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_HANDOFF_DEPTH: u8 = 4;
const MAX_HANDOFFS_PER_RUN: u8 = 8;

pub struct RuntimeBuilder {
    config: RuntimeConfig,
    providers: ProviderRegistry,
    agent_backend: Option<Arc<dyn AgentBackend>>,
}

impl RuntimeBuilder {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            providers: ProviderRegistry::default(),
            agent_backend: None,
        }
    }

    pub fn with_provider(
        mut self,
        provider: Arc<dyn ConversationProvider>,
    ) -> Result<Self, RuntimeError> {
        self.providers.register(provider)?;
        Ok(self)
    }

    pub fn with_engine_backend(
        mut self,
        backend: Arc<dyn EngineBackend>,
    ) -> Result<Self, RuntimeError> {
        let workspace_root = self
            .config
            .workspace_roots
            .first()
            .map(|path| path.to_string_lossy().to_string());
        let model = Some(self.config.model.model.clone());
        let data_root = self.config.data_dir.clone();
        let history_path = data_root
            .as_ref()
            .map(|root| root.join("provider-neutral-assistant-transcript.json"));
        self.providers
            .register(Arc::new(KernelConversationProvider::new(
                backend,
                self.config.build_profile,
                workspace_root,
                model,
                history_path,
                data_root,
            )))?;
        Ok(self)
    }

    pub fn with_agent_control_backend(mut self, backend: Arc<dyn AgentBackend>) -> Self {
        self.agent_backend = Some(backend);
        self
    }

    pub fn with_agent_backend(self, backend: Arc<dyn AgentBackend>) -> Result<Self, RuntimeError> {
        let kernel_backend: Arc<dyn EngineBackend> = Arc::new(LegacyAgentKernelBridge::new(
            Arc::clone(&backend),
            legacy_backend_descriptor(backend.as_ref()),
        ));
        Ok(self
            .with_engine_backend(kernel_backend)?
            .with_agent_control_backend(backend))
    }

    pub fn build(self) -> Result<MahayanaRuntime, RuntimeError> {
        MahayanaRuntime::new(self.config, self.providers, self.agent_backend)
    }

    /// Starts an Agent backend on the same Tokio runtime that the long-lived
    /// Mahayana runtime will own. This is required by in-process Codex because
    /// its app-server worker tasks must outlive synchronous FFI construction.
    pub fn build_with_agent_backend<F, Fut>(
        self,
        create_backend: F,
    ) -> Result<MahayanaRuntime, RuntimeError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<dyn AgentBackend>, AgentError>>,
    {
        self.build_with_agent_backend_and(create_backend, |builder, _backend| Ok(builder))
    }

    /// Variant of [`Self::build_with_agent_backend`] that lets callers add
    /// additional conversation providers backed by the same in-process Agent
    /// before the runtime starts (for example, mini-app peers).
    pub fn build_with_agent_backend_and<F, Fut, C>(
        self,
        create_backend: F,
        configure: C,
    ) -> Result<MahayanaRuntime, RuntimeError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<dyn AgentBackend>, AgentError>>,
        C: FnOnce(Self, Arc<dyn AgentBackend>) -> Result<Self, RuntimeError>,
    {
        let async_runtime = create_async_runtime()?;
        let backend = async_runtime
            .block_on(create_backend())
            .map_err(|error| RuntimeError::AgentInitialization(error.to_string()))?;
        let builder = self.with_agent_backend(Arc::clone(&backend))?;
        let builder = configure(builder, backend)?;
        MahayanaRuntime::new_with_async_runtime(
            builder.config,
            builder.providers,
            builder.agent_backend,
            async_runtime,
        )
    }
}

#[derive(Clone)]
struct RunContext {
    turn_id: TurnId,
    run_id: RunId,
    conversation_id: ConversationId,
    actor: Arc<ConversationActor>,
}

#[derive(Clone)]
struct PendingRuntimeApproval {
    provider_key: String,
    request: CapabilityRequest,
}

pub struct MahayanaRuntime {
    config: RuntimeConfig,
    providers: Arc<ProviderRegistry>,
    agent_backend: Option<Arc<dyn AgentBackend>>,
    async_runtime: tokio::runtime::Runtime,
    event_tx: Sender<RuntimeEvent>,
    event_rx: Receiver<RuntimeEvent>,
    operations: Arc<Mutex<HashMap<OperationId, String>>>,
    run_contexts: Arc<Mutex<HashMap<OperationId, RunContext>>>,
    handoff_counts: Mutex<HashMap<RunId, u8>>,
    approvals: Arc<Mutex<HashMap<ApprovalId, PendingRuntimeApproval>>>,
    actors: Arc<ConversationActorRegistry>,
    store: Arc<RuntimeStore>,
    capability_broker: CapabilityBroker,
    official_miniapps: Mutex<OfficialMiniAppEngine>,
    approved_local_plugin_tools: Mutex<HashSet<(String, String)>>,
}

impl MahayanaRuntime {
    fn new(
        config: RuntimeConfig,
        providers: ProviderRegistry,
        agent_backend: Option<Arc<dyn AgentBackend>>,
    ) -> Result<Self, RuntimeError> {
        let async_runtime = create_async_runtime()?;
        Self::new_with_async_runtime(config, providers, agent_backend, async_runtime)
    }

    fn new_with_async_runtime(
        config: RuntimeConfig,
        providers: ProviderRegistry,
        agent_backend: Option<Arc<dyn AgentBackend>>,
        async_runtime: tokio::runtime::Runtime,
    ) -> Result<Self, RuntimeError> {
        if config.remote_agent_enabled {
            return Err(RuntimeError::RemoteAgentForbidden);
        }
        if config.telemetry_enabled && !cfg!(feature = "telemetry") {
            return Err(RuntimeError::TelemetryNotCompiled);
        }
        if matches!(
            config.model.provider,
            mahayana_core::ModelProviderMode::UserConfiguredRemote
        ) && !cfg!(feature = "remote-model-provider")
        {
            return Err(RuntimeError::RemoteModelNotCompiled);
        }

        let (event_tx, event_rx) = crossbeam_channel::bounded(1024);
        let store = Arc::new(RuntimeStore::open(config.data_dir.as_deref())?);
        let actors = Arc::new(ConversationActorRegistry::default());
        let capability_broker = CapabilityBroker::new(Arc::clone(&store));
        let runtime = Self {
            config,
            providers: Arc::new(providers),
            agent_backend,
            async_runtime,
            event_tx,
            event_rx,
            operations: Arc::new(Mutex::new(HashMap::new())),
            run_contexts: Arc::new(Mutex::new(HashMap::new())),
            handoff_counts: Mutex::new(HashMap::new()),
            approvals: Arc::new(Mutex::new(HashMap::new())),
            actors,
            store,
            capability_broker,
            official_miniapps: Mutex::new(OfficialMiniAppEngine::default()),
            approved_local_plugin_tools: Mutex::new(HashSet::new()),
        };
        runtime
            .event_tx
            .send(RuntimeEvent::Ready {
                status: runtime.status(),
            })
            .map_err(|_| RuntimeError::EventConsumerClosed)?;
        runtime.recover_pending_handoffs()?;
        Ok(runtime)
    }

    pub fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            runtime_abi_version: RUNTIME_ABI_VERSION,
            conversation_schema_version: CONVERSATION_SCHEMA_VERSION,
            model_runtime_version: MODEL_RUNTIME_VERSION,
            build_profile: self.config.build_profile,
            model_provider: self.config.model.provider,
            model: self.config.model.model.clone(),
            remote_agent_enabled: self.config.remote_agent_enabled,
            telemetry_enabled: self.config.telemetry_enabled,
            providers: self.providers.keys(),
        }
    }

    /// Prepare the provider/session behind a conversation before the first
    /// user-visible message. This is intentionally side-effect free with
    /// respect to transcript content and is safe to call more than once.
    pub fn warmup_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<(), RuntimeError> {
        let provider = self.providers.for_conversation(&conversation_id)?;
        self.async_runtime
            .block_on(provider.warmup(&conversation_id))
            .map_err(RuntimeError::from)
    }

    /// Reset local conversation/Agent state when the authenticated product
    /// account changes. This also drains queued events so a previous account's
    /// reply cannot appear after the new account is ready.
    pub fn reset_session(&self) -> Result<(), RuntimeError> {
        if let Some(backend) = self.agent_backend.as_ref() {
            backend
                .reset_session()
                .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
        }
        for provider in self.providers.providers() {
            self.async_runtime.block_on(provider.reset_session())?;
        }
        lock(&self.operations)?.clear();
        lock(&self.run_contexts)?.clear();
        lock(&self.handoff_counts)?.clear();
        lock(&self.approvals)?.clear();
        self.actors.clear().map_err(RuntimeError::Synchronization)?;
        while self.event_rx.try_recv().is_ok() {}
        Ok(())
    }

    /// Switch long-lived local providers to an account-scoped transcript path
    /// without deleting the previous account's durable Agent state.
    pub fn switch_conversation_history(
        &self,
        path: Option<std::path::PathBuf>,
    ) -> Result<(), RuntimeError> {
        if let Some(backend) = self.agent_backend.as_ref() {
            backend
                .reset_session()
                .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
        }
        for provider in self.providers.providers() {
            self.async_runtime
                .block_on(provider.set_history_path(path.clone()))?;
        }
        lock(&self.operations)?.clear();
        lock(&self.run_contexts)?.clear();
        lock(&self.handoff_counts)?.clear();
        lock(&self.approvals)?.clear();
        self.actors.clear().map_err(RuntimeError::Synchronization)?;
        while self.event_rx.try_recv().is_ok() {}
        Ok(())
    }

    pub fn execute(&self, command: RuntimeCommand) -> Result<RuntimeResponse, RuntimeError> {
        match command {
            RuntimeCommand::Status => Ok(RuntimeResponse::Status(self.status())),
            RuntimeCommand::WorkspaceStateGet { key } => {
                validate_runtime_state_key(&key)?;
                Ok(RuntimeResponse::RuntimeWorkspaceState {
                    value: self.store.read_workspace_state(&key)?,
                    key,
                })
            }
            RuntimeCommand::WorkspaceStateSet { key, value } => {
                validate_runtime_state_key(&key)?;
                self.store.write_workspace_state(&key, &value, now_millis())?;
                Ok(RuntimeResponse::RuntimeWorkspaceState {
                    key,
                    value: Some(value),
                })
            }
            RuntimeCommand::ListConversations => Ok(RuntimeResponse::Conversations {
                data: self.list_conversations()?,
            }),
            RuntimeCommand::AuthorizeCapability {
                request,
                availability,
                unavailable_reason,
            } => {
                let decision = self
                    .capability_broker
                    .authorize_request(
                        availability,
                        unavailable_reason,
                        request,
                        now_millis(),
                    )
                    .map_err(RuntimeError::CapabilityBroker)?;
                Ok(RuntimeResponse::CapabilityDecision { decision })
            }
            RuntimeCommand::ListCapabilities { query } => {
                let registry = CapabilityRegistry::from_conversations(
                    self.list_conversations()?,
                    self.config.build_profile,
                );
                Ok(RuntimeResponse::Capabilities {
                    data: registry.list(query.as_deref()),
                })
            }
            RuntimeCommand::InvokeCapability {
                capability_id,
                text,
                client_message_id,
            } => {
                let registry = CapabilityRegistry::from_conversations(
                    self.list_conversations()?,
                    self.config.build_profile,
                );
                let capability = registry
                    .resolve(&capability_id)
                    .cloned()
                    .ok_or_else(|| RuntimeError::CapabilityNotFound(capability_id.clone()))?;
                if !capability.is_invokable() {
                    return Err(RuntimeError::CapabilityUnavailable {
                        capability_id: capability.id,
                        reason: capability
                            .unavailable_reason
                            .unwrap_or_else(|| "当前平台不可用".to_string()),
                    });
                }
                let conversation_id = capability.conversation_id.clone();
                let decision = self
                    .capability_broker
                    .authorize(
                        &capability,
                        CapabilityRequest {
                            actor: "human".to_string(),
                            agent_id: None,
                            conversation_id: conversation_id.clone(),
                            run_id: None,
                            capability: capability.id.clone(),
                            target: Value::Null,
                            intent: text.clone(),
                        },
                        now_millis(),
                    )
                    .map_err(RuntimeError::CapabilityBroker)?;
                require_capability_execution_allowed(&capability.id, decision)?;
                let operation_id =
                    self.start_message(
                        conversation_id.clone(),
                        text,
                        client_message_id,
                        None,
                        None,
                        false,
                    )?;
                Ok(RuntimeResponse::CapabilityAccepted {
                    capability_id: capability.id,
                    conversation_id,
                    operation_id,
                })
            }
            RuntimeCommand::ListPluginCommands { plugin_id } => {
                if plugin_id
                    .as_deref()
                    .is_some_and(|plugin_id| app_definition(plugin_id).is_some())
                    && matches!(
                        self.config.build_profile,
                        mahayana_core::BuildProfile::MobileEmbedded
                    )
                {
                    return Ok(RuntimeResponse::PluginCommands {
                        data: local_plugin_commands(plugin_id.as_deref()),
                    });
                }
                let Some(provider) = self.providers.get("miniapp") else {
                    return Ok(RuntimeResponse::PluginCommands {
                        data: local_plugin_commands(plugin_id.as_deref()),
                    });
                };
                let data: Vec<PluginCommandDescriptor> = self
                    .async_runtime
                    .block_on(provider.list_plugin_commands(plugin_id.as_deref()))?;
                Ok(RuntimeResponse::PluginCommands { data })
            }
            RuntimeCommand::PluginUi { plugin_id } => Ok(RuntimeResponse::PluginUi {
                html: home_html(&plugin_id).map_err(RuntimeError::LocalPlugin)?,
                plugin_id,
            }),
            RuntimeCommand::ApproveLocalPluginTool { plugin_id, tool } => {
                let definition = app_definition(&plugin_id).ok_or_else(|| {
                    RuntimeError::LocalPlugin(format!("unknown official plugin: {plugin_id}"))
                })?;
                if !definition.tools.iter().any(|descriptor| {
                    descriptor.get("name").and_then(Value::as_str) == Some(tool.as_str())
                }) {
                    return Err(RuntimeError::LocalPlugin(format!(
                        "{plugin_id} has no MCP Tool {tool}"
                    )));
                }
                lock(&self.approved_local_plugin_tools)?.insert((plugin_id.clone(), tool.clone()));
                Ok(RuntimeResponse::LocalPluginToolApproved { plugin_id, tool })
            }
            RuntimeCommand::CallLocalPluginTool {
                plugin_id,
                tool,
                arguments,
            } => {
                let definition = app_definition(&plugin_id).ok_or_else(|| {
                    RuntimeError::LocalPlugin(format!("unknown official plugin: {plugin_id}"))
                })?;
                let descriptor = definition
                    .tools
                    .iter()
                    .find(|descriptor| {
                        descriptor.get("name").and_then(Value::as_str) == Some(tool.as_str())
                    })
                    .ok_or_else(|| {
                        RuntimeError::LocalPlugin(format!("{plugin_id} has no MCP Tool {tool}"))
                    })?;
                let read_only = descriptor
                    .pointer("/annotations/readOnlyHint")
                    .and_then(Value::as_bool)
                    == Some(true);
                let explicitly_approved = read_only
                    || lock(&self.approved_local_plugin_tools)?
                        .contains(&(plugin_id.clone(), tool.clone()));
                let decision = self
                    .capability_broker
                    .authorize_request(
                        if explicitly_approved {
                            CapabilityAvailability::Ready
                        } else {
                            CapabilityAvailability::PermissionRequired
                        },
                        None,
                        CapabilityRequest {
                            actor: "human".to_string(),
                            agent_id: None,
                            conversation_id: ConversationId(format!("miniapp:{plugin_id}")),
                            run_id: None,
                            capability: format!("miniapp.{plugin_id}.tool.{tool}"),
                            target: serde_json::json!({
                                "pluginId": plugin_id.clone(),
                                "tool": tool.clone(),
                            }),
                            intent: format!("invoke local Mini App tool {plugin_id}/{tool}"),
                        },
                        now_millis(),
                    )
                    .map_err(RuntimeError::CapabilityBroker)?;
                if !matches!(decision, CapabilityPolicyDecision::Allow) {
                    return Err(RuntimeError::LocalPlugin(format!(
                        "host approval is required for {plugin_id}/{tool}"
                    )));
                }
                let outcome = lock(&self.official_miniapps)?
                    .call_tool(&plugin_id, &tool, arguments)
                    .map_err(RuntimeError::LocalPlugin)?;
                let progress = outcome
                    .progress
                    .into_iter()
                    .map(|update| serde_json::to_value(update).unwrap_or(Value::Null))
                    .collect();
                Ok(RuntimeResponse::LocalPluginToolResult {
                    plugin_id,
                    tool,
                    result: outcome.result,
                    progress,
                })
            }
            RuntimeCommand::McpServers => {
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let data = self
                    .async_runtime
                    .block_on(backend.list_mcp_servers())
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpServers { data })
            }
            RuntimeCommand::McpApps => {
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let data = self
                    .async_runtime
                    .block_on(backend.list_connector_apps())
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpApps { data })
            }
            RuntimeCommand::McpOauthLogin { server } => {
                self.authorize_human_capability(
                    "mcp.oauth.manage",
                    serde_json::json!({"server": server.clone()}),
                    "start MCP OAuth login",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let authorization_url = self
                    .async_runtime
                    .block_on(backend.mcp_oauth_login(&server))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpOauth {
                    server,
                    authorization_url: Some(authorization_url),
                    removed: false,
                })
            }
            RuntimeCommand::McpOauthLogout { server } => {
                self.authorize_human_capability(
                    "mcp.oauth.manage",
                    serde_json::json!({"server": server.clone()}),
                    "remove MCP OAuth credentials",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let removed = self
                    .async_runtime
                    .block_on(backend.mcp_oauth_logout(&server))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                self.async_runtime
                    .block_on(backend.refresh_mcp_servers())
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpOauth {
                    server,
                    authorization_url: None,
                    removed,
                })
            }
            RuntimeCommand::McpRemove { server } => {
                self.authorize_human_capability(
                    "mcp.server.manage",
                    serde_json::json!({"server": server.clone()}),
                    "remove an MCP server",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let removed = self
                    .async_runtime
                    .block_on(backend.remove_mcp_server(&server))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpRemoved { server, removed })
            }
            RuntimeCommand::McpCustomInstructions => {
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let instructions = self
                    .async_runtime
                    .block_on(backend.mcp_custom_instructions())
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpCustomInstructions { instructions })
            }
            RuntimeCommand::McpSetCustomInstructions {
                server,
                instructions,
            } => {
                self.authorize_human_capability(
                    "mcp.server.configure",
                    serde_json::json!({"server": server.clone()}),
                    "change MCP server instructions",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                self.async_runtime
                    .block_on(backend.set_mcp_custom_instructions(&server, &instructions))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpCustomInstructionsUpdated { server })
            }
            RuntimeCommand::McpSetToolDisabled {
                server,
                tool,
                disabled,
            } => {
                self.authorize_human_capability(
                    "mcp.tool.policy",
                    serde_json::json!({
                        "server": server.clone(),
                        "tool": tool.clone(),
                        "disabled": disabled,
                    }),
                    "change MCP tool policy",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let disabled_tools = self
                    .async_runtime
                    .block_on(backend.set_mcp_tool_disabled(&server, &tool, disabled))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpToolDisabledUpdated {
                    server,
                    disabled_tools,
                })
            }
            RuntimeCommand::McpRefresh => {
                self.authorize_human_capability(
                    "mcp.refresh",
                    serde_json::json!({}),
                    "refresh MCP server state",
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                self.async_runtime
                    .block_on(backend.refresh_mcp_servers())
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpRefreshed)
            }
            RuntimeCommand::McpToolCall {
                server,
                tool,
                arguments,
            } => {
                self.authorize_human_capability(
                    "mcp.tool.call",
                    serde_json::json!({
                        "server": server.clone(),
                        "tool": tool.clone(),
                    }),
                    &format!("invoke MCP tool {server}/{tool}"),
                )?;
                let backend = self.agent_backend.as_ref().ok_or_else(|| {
                    RuntimeError::AgentBackend("no agent backend is available".into())
                })?;
                let result = self
                    .async_runtime
                    .block_on(backend.call_mcp_tool(&server, &tool, arguments))
                    .map_err(|error| RuntimeError::AgentBackend(error.to_string()))?;
                Ok(RuntimeResponse::McpToolResult {
                    server,
                    tool,
                    result,
                })
            }
            RuntimeCommand::ConversationHistory {
                conversation_id,
                limit,
            } => {
                let provider = self.providers.for_conversation(&conversation_id)?;
                let data = self.async_runtime.block_on(
                    provider.history(&conversation_id, limit.unwrap_or(50).clamp(1, 500)),
                )?;
                Ok(RuntimeResponse::History { data })
            }
            RuntimeCommand::SendMessage {
                conversation_id,
                text,
                client_message_id,
                retry_of_client_message_id,
                inference_provider,
                hidden,
            } => Ok(RuntimeResponse::Accepted {
                operation_id: self.start_message(
                    conversation_id,
                    text,
                    client_message_id,
                    retry_of_client_message_id,
                    inference_provider,
                    hidden,
                )?,
            }),
            RuntimeCommand::AskUser {
                operation_id,
                question,
            } => {
                let question = question.trim().to_string();
                if question.is_empty() {
                    return Err(RuntimeError::Collaboration("ask_user requires a non-empty question".to_string()));
                }
                let context = lock(&self.run_contexts)?
                    .get(&operation_id)
                    .cloned()
                    .ok_or_else(|| ConversationError::OperationNotFound(operation_id.clone()))?;
                let intent_id = IntentId::generated("ask-user");
                let request = AskUserRequest {
                    id: intent_id.clone(),
                    turn_id: context.turn_id.clone(),
                    run_id: context.run_id.clone(),
                    question: question.clone(),
                };
                self.store.enqueue_ask_user(&request, now_millis())?;
                transition_turn_state(
                    &self.event_tx,
                    &self.store,
                    &context,
                    TurnState::WaitingUser,
                )?;
                self.event_tx
                    .send(RuntimeEvent::AgentActivity {
                        operation_id: operation_id.clone(),
                        step_id: intent_id.to_string(),
                        kind: "ask_user".to_string(),
                        title: question,
                        detail: None,
                        status: mahayana_core::RuntimeActivityStatus::Running,
                        metadata: Some(serde_json::json!({ "intentId": intent_id })),
                    })
                    .map_err(|_| RuntimeError::EventConsumerClosed)?;
                Ok(RuntimeResponse::WaitingUser {
                    intent_id,
                    operation_id,
                })
            }
            RuntimeCommand::Handoff {
                operation_id,
                target_agent,
                target_conversation_id,
                inference_provider,
                task,
                constraints,
                expected_output,
                depth,
            } => {
                let context = lock(&self.run_contexts)?
                    .get(&operation_id)
                    .cloned()
                    .ok_or_else(|| ConversationError::OperationNotFound(operation_id.clone()))?;
                self.dispatch_handoff(
                    operation_id,
                    context.run_id.clone(),
                    context.turn_id.clone(),
                    runtime_agent_id_from_conversation(&context.conversation_id),
                    Some(context.conversation_id),
                    target_agent,
                    target_conversation_id,
                    inference_provider,
                    task,
                    constraints,
                    expected_output,
                    depth,
                )
            }
            RuntimeCommand::ExternalHandoff {
                origin_run_id,
                origin_turn_id,
                origin_agent,
                target_agent,
                target_conversation_id,
                inference_provider,
                task,
                constraints,
                expected_output,
                depth,
            } => {
                let synthetic_operation_id = OperationId(origin_run_id.to_string());
                self.dispatch_handoff(
                    synthetic_operation_id,
                    origin_run_id,
                    origin_turn_id,
                    origin_agent,
                    None,
                    target_agent,
                    target_conversation_id,
                    inference_provider,
                    task,
                    constraints,
                    expected_output,
                    depth,
                )
            }
            RuntimeCommand::Interrupt { operation_id } => {
                let provider_key = lock(&self.operations)?
                    .get(&operation_id)
                    .cloned()
                    .ok_or_else(|| ConversationError::OperationNotFound(operation_id.clone()))?;
                let provider = self
                    .providers
                    .get(&provider_key)
                    .ok_or_else(|| ConversationError::ProviderUnavailable(provider_key.clone()))?;
                self.async_runtime
                    .block_on(provider.interrupt(&operation_id))?;
                if let Some(context) = lock(&self.run_contexts)?.get(&operation_id).cloned() {
                    let _ = transition_turn_state(
                        &self.event_tx,
                        &self.store,
                        &context,
                        TurnState::Cancelled,
                    );
                }
                Ok(RuntimeResponse::Interrupted { operation_id })
            }
            RuntimeCommand::ResolveApproval {
                approval_id,
                decision,
                payload,
            } => {
                let pending = lock(&self.approvals)?
                    .remove(&approval_id)
                    .ok_or_else(|| ConversationError::ApprovalNotFound(approval_id.clone()))?;
                let (availability, unavailable_reason, expected_decision) = match decision {
                    RuntimeApprovalDecision::Accept | RuntimeApprovalDecision::AcceptForSession => (
                        CapabilityAvailability::Ready,
                        None,
                        CapabilityPolicyDecision::Allow,
                    ),
                    RuntimeApprovalDecision::Decline | RuntimeApprovalDecision::Cancel => (
                        CapabilityAvailability::Unavailable,
                        Some("user denied the requested capability".to_string()),
                        CapabilityPolicyDecision::Deny,
                    ),
                };
                let broker_decision = self
                    .capability_broker
                    .authorize_request(
                        availability,
                        unavailable_reason,
                        pending.request.clone(),
                        now_millis(),
                    )
                    .map_err(RuntimeError::CapabilityBroker)?;
                if broker_decision != expected_decision {
                    return Err(RuntimeError::CapabilityBroker(
                        "approval resolution disagreed with capability policy".to_string(),
                    ));
                }
                let provider = self
                    .providers
                    .get(&pending.provider_key)
                    .ok_or_else(|| ConversationError::ProviderUnavailable(pending.provider_key.clone()))?;
                self.async_runtime
                    .block_on(provider.resolve_approval(ResolveApprovalRequest {
                        approval_id: approval_id.clone(),
                        decision,
                        payload,
                    }))?;
                Ok(RuntimeResponse::ApprovalResolved { approval_id })
            }
        }
    }

    fn authorize_human_capability(
        &self,
        capability: &str,
        target: Value,
        intent: &str,
    ) -> Result<(), RuntimeError> {
        let decision = self
            .capability_broker
            .authorize_request(
                CapabilityAvailability::Ready,
                None,
                CapabilityRequest {
                    actor: "human".to_string(),
                    agent_id: None,
                    conversation_id: ConversationId(MAHAYANA_AI_CONVERSATION_ID.to_string()),
                    run_id: None,
                    capability: capability.to_string(),
                    target,
                    intent: intent.to_string(),
                },
                now_millis(),
            )
            .map_err(RuntimeError::CapabilityBroker)?;
        require_capability_execution_allowed(capability, decision)
    }

    fn reserve_handoff_slot(&self, origin_run: &RunId) -> Result<(), RuntimeError> {
        let persisted = self.store.count_handoffs_for_run(origin_run)?;
        let mut counts = lock(&self.handoff_counts)?;
        let count = counts.entry(origin_run.clone()).or_default();
        let effective = u64::from(*count).max(persisted);
        if effective >= u64::from(MAX_HANDOFFS_PER_RUN) {
            return Err(RuntimeError::Collaboration(format!(
                "handoff fan-out limit exceeded ({MAX_HANDOFFS_PER_RUN})"
            )));
        }
        *count = (effective + 1).min(u64::from(u8::MAX)) as u8;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_handoff(
        &self,
        origin_operation_id: OperationId,
        origin_run: RunId,
        origin_turn: TurnId,
        origin_agent: Option<String>,
        origin_conversation: Option<ConversationId>,
        target_agent: String,
        target_conversation_id: Option<ConversationId>,
        inference_provider: Option<String>,
        task: String,
        constraints: Value,
        expected_output: Option<String>,
        depth: u8,
    ) -> Result<RuntimeResponse, RuntimeError> {
        let target_agent = target_agent.trim().to_string();
        let task = task.trim().to_string();
        if target_agent.is_empty()
            || target_agent.len() > 160
            || target_agent.chars().any(char::is_control)
        {
            return Err(RuntimeError::Collaboration(
                "handoff targetAgent is invalid".to_string(),
            ));
        }
        if task.is_empty() {
            return Err(RuntimeError::Collaboration(
                "handoff requires a non-empty task".to_string(),
            ));
        }
        if depth >= MAX_HANDOFF_DEPTH {
            return Err(RuntimeError::Collaboration(format!(
                "handoff depth limit exceeded ({MAX_HANDOFF_DEPTH})"
            )));
        }
        if origin_agent.as_deref() == Some(target_agent.as_str()) {
            return Err(RuntimeError::Collaboration(
                "an Agent cannot hand off work to itself".to_string(),
            ));
        }

        let target_conversation = target_conversation_id
            .unwrap_or_else(|| ConversationId(format!("codex:agent:{target_agent}")));
        if origin_conversation.as_ref() == Some(&target_conversation) {
            return Err(RuntimeError::Collaboration(
                "an Agent cannot hand off work to its own conversation".to_string(),
            ));
        }

        let source_conversation = origin_conversation
            .clone()
            .unwrap_or_else(|| ConversationId(MAHAYANA_AI_CONVERSATION_ID.to_string()));
        let policy = self
            .capability_broker
            .authorize_request(
                CapabilityAvailability::Ready,
                None,
                CapabilityRequest {
                    actor: origin_agent
                        .as_ref()
                        .map(|id| format!("agent:{id}"))
                        .unwrap_or_else(|| "human".to_string()),
                    agent_id: origin_agent.clone(),
                    conversation_id: source_conversation,
                    run_id: Some(origin_run.clone()),
                    capability: "agent.handoff".to_string(),
                    target: serde_json::json!({
                        "targetAgent": target_agent,
                        "targetConversationId": target_conversation,
                        "depth": depth.saturating_add(1),
                    }),
                    intent: "dispatch durable work to another Agent".to_string(),
                },
                now_millis(),
            )
            .map_err(RuntimeError::CapabilityBroker)?;
        require_capability_execution_allowed("agent.handoff", policy)?;

        self.reserve_handoff_slot(&origin_run)?;
        let intent_id = IntentId::generated("handoff");
        let intent = HandoffIntent {
            id: intent_id.clone(),
            target_agent: target_agent.clone(),
            target_conversation_id: Some(target_conversation.clone()),
            inference_provider: inference_provider.clone(),
            task: task.clone(),
            constraints,
            expected_output,
            origin_run,
            depth: depth.saturating_add(1),
        };
        self.store.enqueue_handoff(&intent, &origin_turn, now_millis())?;
        let target_operation_id = self.start_message(
            target_conversation,
            handoff_prompt(&intent),
            Some(intent_id.to_string()),
            None,
            inference_provider,
            true,
        )?;
        self.store.mark_handoff_started(
            intent_id.as_str(),
            target_operation_id.as_str(),
            now_millis(),
        )?;
        self.event_tx
            .send(RuntimeEvent::AgentActivity {
                operation_id: origin_operation_id.clone(),
                step_id: intent_id.to_string(),
                kind: "handoff".to_string(),
                title: format!("Handoff → {target_agent}"),
                detail: Some("Durable Agent work queued".to_string()),
                status: mahayana_core::RuntimeActivityStatus::Completed,
                metadata: Some(serde_json::json!({
                    "intentId": intent_id,
                    "targetAgent": target_agent,
                    "targetOperationId": target_operation_id,
                    "depth": intent.depth,
                })),
            })
            .map_err(|_| RuntimeError::EventConsumerClosed)?;
        Ok(RuntimeResponse::HandoffQueued {
            intent_id,
            operation_id: origin_operation_id,
            target_operation_id,
        })
    }

    fn recover_pending_handoffs(&self) -> Result<(), RuntimeError> {
        for pending in self.store.recoverable_handoffs()? {
            let target_conversation = pending
                .intent
                .target_conversation_id
                .clone()
                .unwrap_or_else(|| {
                    ConversationId(format!("codex:agent:{}", pending.intent.target_agent))
                });
            let message_id = MessageId::new(pending.intent.id.to_string())
                .map_err(|error| RuntimeError::Collaboration(error.to_string()))?;
            let existing = self
                .store
                .find_turn_by_client_message(&target_conversation, &message_id)?;

            if let Some(snapshot) = existing.as_ref() {
                if snapshot.state == TurnState::Completed {
                    self.store.mark_handoff_terminal(
                        pending.intent.id.as_str(),
                        true,
                        now_millis(),
                    )?;
                    continue;
                }
                if matches!(snapshot.state, TurnState::Failed | TurnState::Cancelled) {
                    self.store.mark_handoff_terminal(
                        pending.intent.id.as_str(),
                        false,
                        now_millis(),
                    )?;
                    continue;
                }
            }

            let decision = self
                .capability_broker
                .authorize_request(
                    CapabilityAvailability::Ready,
                    None,
                    CapabilityRequest {
                        actor: "runtime-recovery".to_string(),
                        agent_id: None,
                        conversation_id: ConversationId(
                            MAHAYANA_AI_CONVERSATION_ID.to_string(),
                        ),
                        run_id: Some(pending.intent.origin_run.clone()),
                        capability: "agent.handoff".to_string(),
                        target: serde_json::json!({
                            "targetAgent": pending.intent.target_agent.clone(),
                            "targetConversationId": target_conversation.clone(),
                            "depth": pending.intent.depth,
                        }),
                        intent: "recover durable Agent handoff after runtime restart".to_string(),
                    },
                    now_millis(),
                )
                .map_err(RuntimeError::CapabilityBroker)?;
            if let Err(error) = require_capability_execution_allowed("agent.handoff", decision) {
                self.store.mark_handoff_terminal(
                    pending.intent.id.as_str(),
                    false,
                    now_millis(),
                )?;
                let _ = self.event_tx.send(RuntimeEvent::ProviderDegraded {
                    provider: "handoff-recovery".to_string(),
                    message: error.to_string(),
                });
                continue;
            }

            let retry_message_id = if let Some(snapshot) = existing.as_ref() {
                if let Some(run_id) = snapshot.last_run_id.as_ref() {
                    self.store
                        .set_run_state(run_id, TurnState::Failed, Some(now_millis()))?;
                }
                self.store
                    .set_turn_state(&snapshot.turn_id, TurnState::Recovering, None)?;
                Some(pending.intent.id.to_string())
            } else {
                None
            };
            let client_message_id = if retry_message_id.is_some() {
                None
            } else {
                Some(pending.intent.id.to_string())
            };

            match self.start_message(
                target_conversation,
                handoff_prompt(&pending.intent),
                client_message_id,
                retry_message_id,
                pending.intent.inference_provider.clone(),
                true,
            ) {
                Ok(operation_id) => {
                    self.store.mark_handoff_started(
                        pending.intent.id.as_str(),
                        operation_id.as_str(),
                        now_millis(),
                    )?;
                }
                Err(error) => {
                    let _ = self.event_tx.send(RuntimeEvent::ProviderDegraded {
                        provider: "handoff-recovery".to_string(),
                        message: format!(
                            "could not recover handoff {} from turn {}: {error}",
                            pending.intent.id, pending.origin_turn_id
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    fn list_conversations(&self) -> Result<Vec<Conversation>, RuntimeError> {
        let providers = self.providers.providers();
        let (mut conversations, degraded) = self.async_runtime.block_on(async move {
            let mut conversations = Vec::new();
            let mut degraded = Vec::new();
            for provider in providers {
                match provider.list_conversations().await {
                    Ok(mut provider_conversations) => {
                        conversations.append(&mut provider_conversations)
                    }
                    Err(error) => degraded.push((provider.key().to_string(), error.to_string())),
                }
            }
            (conversations, degraded)
        });
        conversations.sort_by(|left, right| {
            right
                .pinned
                .cmp(&left.pinned)
                .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
                .then_with(|| left.id.cmp(&right.id))
        });
        for (provider, message) in degraded {
            let _ = self
                .event_tx
                .send(RuntimeEvent::ProviderDegraded { provider, message });
        }
        Ok(conversations)
    }

    fn start_message(
        &self,
        conversation_id: ConversationId,
        text: String,
        client_message_id: Option<String>,
        retry_of_client_message_id: Option<String>,
        inference_provider: Option<String>,
        hidden: bool,
    ) -> Result<OperationId, RuntimeError> {
        if text.trim().is_empty() {
            return Err(RuntimeError::EmptyMessage);
        }

        let provider = self.providers.for_conversation(&conversation_id)?;
        let provider_key = provider.key().to_string();
        let run_id = RunId::generated("run");
        // operationId remains the backwards-compatible wire key. Canonical
        // runtime ownership now belongs to ExecutionRun.
        let operation_id = OperationId(run_id.0.clone());
        let run_started_at_ms = now_millis();
        let retrying = retry_of_client_message_id.is_some();
        let (
            turn_id,
            user_message_id,
            turn_created_at_ms,
            generation,
            provider_client_message_id,
        ) = if let Some(retry_message_id) = retry_of_client_message_id {
            let (turn_id, created_at_ms, generation) = self
                .store
                .retry_turn_generation(conversation_id.as_str(), &retry_message_id)?
                .ok_or_else(|| RuntimeError::RetryTargetNotFound(retry_message_id.clone()))?;
            let user_message_id = MessageId::new(retry_message_id.clone())
                .map_err(|_| RuntimeError::RetryTargetNotFound(retry_message_id.clone()))?;
            (
                turn_id,
                Some(user_message_id),
                created_at_ms,
                generation,
                Some(retry_message_id),
            )
        } else {
            let turn_id = TurnId::generated("turn");
            let user_message_id = client_message_id
                .as_ref()
                .and_then(|value| MessageId::new(value.clone()).ok())
                .or_else(|| Some(MessageId::generated("message")));
            (
                turn_id,
                user_message_id,
                run_started_at_ms,
                1,
                client_message_id,
            )
        };
        let actor = self
            .actors
            .actor(&conversation_id)
            .map_err(RuntimeError::Synchronization)?;
        let (queued, accepted_sequence) = actor
            .register(turn_id.clone())
            .map_err(RuntimeError::Synchronization)?;
        let initial_state = if retrying {
            TurnState::Recovering
        } else {
            TurnState::Accepted
        };
        let initial_sequence = if retrying {
            actor
                .set_state(&turn_id, TurnState::Recovering)
                .map_err(RuntimeError::Synchronization)?
                .unwrap_or(accepted_sequence)
        } else {
            accepted_sequence
        };

        let turn = LogicalTurn {
            id: turn_id.clone(),
            conversation_id: conversation_id.clone(),
            user_message_id,
            created_at_ms: turn_created_at_ms,
            state: initial_state,
            active_run_id: Some(run_id.clone()),
        };
        let run = ExecutionRun {
            id: run_id.clone(),
            turn_id: turn_id.clone(),
            generation,
            provider: provider_key.clone(),
            started_at_ms: run_started_at_ms,
            finished_at_ms: None,
            state: initial_state,
        };
        self.store.record_turn(&turn)?;
        self.store.record_run(&run)?;

        let context = RunContext {
            turn_id: turn_id.clone(),
            run_id: run_id.clone(),
            conversation_id: conversation_id.clone(),
            actor: Arc::clone(&actor),
        };
        lock(&self.operations)?.insert(operation_id.clone(), provider_key.clone());
        lock(&self.run_contexts)?.insert(operation_id.clone(), context.clone());

        self.event_tx
            .send(RuntimeEvent::TurnStateChanged {
                operation_id: operation_id.clone(),
                turn_id: turn_id.clone(),
                run_id: run_id.clone(),
                conversation_id: conversation_id.clone(),
                state: initial_state,
                sequence: initial_sequence,
            })
            .map_err(|_| RuntimeError::EventConsumerClosed)?;
        if queued {
            transition_turn_state(&self.event_tx, &self.store, &context, TurnState::Queued)?;
        }

        let durable_intent_id = provider_client_message_id.clone();
        let request = SendMessageRequest {
            conversation_id: conversation_id.clone(),
            operation_id: operation_id.clone(),
            text,
            client_message_id: provider_client_message_id,
            inference_provider,
            hidden,
        };
        let sink: SharedConversationEventSink = Arc::new(RuntimeEventSink {
            provider_key,
            event_tx: self.event_tx.clone(),
            approvals: Arc::clone(&self.approvals),
            context: context.clone(),
            store: Arc::clone(&self.store),
        });

        let event_tx = self.event_tx.clone();
        let operations = Arc::clone(&self.operations);
        let run_contexts = Arc::clone(&self.run_contexts);
        let store = Arc::clone(&self.store);
        let task_operation_id = operation_id.clone();
        self.async_runtime.spawn(async move {
            let _gate = actor.gate.lock().await;
            if actor
                .start(&turn_id, run_id.clone())
                .map_err(RuntimeError::Synchronization)
                .is_ok()
            {
                let _ = transition_turn_state(&event_tx, &store, &context, TurnState::Preparing);
                let _ = transition_turn_state(&event_tx, &store, &context, TurnState::Thinking);
            }

            let result = provider.send_message(request, sink).await;
            let terminal_state = if result.is_ok() {
                TurnState::Completed
            } else {
                TurnState::Failed
            };
            let _ = transition_turn_state(&event_tx, &store, &context, terminal_state);
            if let Some(intent_id) = durable_intent_id.as_deref() {
                let _ = store.mark_handoff_terminal(intent_id, result.is_ok(), now_millis());
            }
            let event = match result {
                Ok(()) => RuntimeEvent::OperationCompleted {
                    operation_id: task_operation_id.clone(),
                },
                Err(error) => RuntimeEvent::OperationFailed {
                    operation_id: task_operation_id.clone(),
                    code: if matches!(error, ConversationError::UsageLimitExceeded(_)) {
                        "usage_limit_exceeded"
                    } else {
                        "provider_error"
                    }
                    .to_string(),
                    message: error.to_string(),
                },
            };
            let _ = event_tx.send(event);
            let _ = actor.finish(&run_id);
            if let Ok(mut operations) = operations.lock() {
                operations.remove(&task_operation_id);
            }
            if let Ok(mut contexts) = run_contexts.lock() {
                contexts.remove(&task_operation_id);
            }
        });
        Ok(operation_id)
    }

    pub fn receive(&self, timeout: Duration) -> Result<Option<RuntimeEvent>, RuntimeError> {
        match self.event_rx.recv_timeout(timeout) {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(RuntimeError::EventConsumerClosed),
        }
    }
}

fn legacy_backend_descriptor(backend: &dyn AgentBackend) -> BackendDescriptor {
    BackendDescriptor {
        id: format!("compat:{}", backend.name()),
        display_name: format!("{} compatibility backend", backend.name()),
        native: false,
        capabilities: CapabilitySet::new([
            Capability::Model,
            Capability::FilesystemRead,
            Capability::FilesystemWrite,
            Capability::Process,
            Capability::Git,
            Capability::Network,
            Capability::WebSearch,
            Capability::ComputerUse,
            Capability::ToolProtocol,
            Capability::Mcp,
            Capability::Skills,
            Capability::Plugins,
        ]),
    }
}

fn create_async_runtime() -> Result<tokio::runtime::Runtime, RuntimeError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("mahayana-runtime")
        // Codex app-server thread creation walks a large typed protocol and
        // configuration graph. The platform default (commonly 2 MiB) can
        // overflow on the first embedded thread/turn even though the same
        // code works in the standalone Codex process.
        .thread_stack_size(16 * 1024 * 1024)
        .build()
        .map_err(|error| RuntimeError::Initialization(error.to_string()))
}

fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, RuntimeError> {
    mutex
        .lock()
        .map_err(|_| RuntimeError::Synchronization("mutex poisoned".to_string()))
}

struct RuntimeEventSink {
    provider_key: String,
    event_tx: Sender<RuntimeEvent>,
    approvals: Arc<Mutex<HashMap<ApprovalId, PendingRuntimeApproval>>>,
    context: RunContext,
    store: Arc<RuntimeStore>,
}

impl ConversationEventSink for RuntimeEventSink {
    fn emit(&self, event: RuntimeEvent) -> Result<(), ConversationError> {
        if let RuntimeEvent::AgentActivity {
            kind,
            title,
            status,
            metadata,
            ..
        } = &event
        {
            if kind == "computer"
                && matches!(status, mahayana_core::RuntimeActivityStatus::Running)
            {
                let action = metadata
                    .as_ref()
                    .and_then(|value| value.get("arguments"))
                    .and_then(|value| value.get("action"))
                    .and_then(Value::as_str)
                    .unwrap_or("computer");
                let agent_id = runtime_agent_id_from_conversation(&self.context.conversation_id);
                let decision = CapabilityBroker::new(Arc::clone(&self.store))
                    .authorize_request(
                        CapabilityAvailability::Ready,
                        None,
                        CapabilityRequest {
                            actor: agent_id
                                .as_ref()
                                .map(|id| format!("agent:{id}"))
                                .unwrap_or_else(|| "agent-runtime".to_string()),
                            agent_id,
                            conversation_id: self.context.conversation_id.clone(),
                            run_id: Some(self.context.run_id.clone()),
                            capability: "computer.input.control".to_string(),
                            target: serde_json::json!({
                                "action": action,
                                "deviceId": "local-desktop",
                            }),
                            intent: title.clone(),
                        },
                        now_millis(),
                    )
                    .map_err(ConversationError::Provider)?;
                if decision != CapabilityPolicyDecision::Allow {
                    return Err(ConversationError::Provider(
                        "computer action was not allowed by capability policy".to_string(),
                    ));
                }
            }
        }

        let projected_state = match &event {
            RuntimeEvent::MessageDelta { .. } => Some(TurnState::Streaming),
            RuntimeEvent::ApprovalRequested { .. } => Some(TurnState::WaitingUser),
            RuntimeEvent::AgentActivity { status, .. }
                if matches!(status, mahayana_core::RuntimeActivityStatus::Running) =>
            {
                Some(TurnState::ToolRunning)
            }
            RuntimeEvent::PluginProgress { progress, total, .. }
                if *total == 0 || progress < total =>
            {
                Some(TurnState::ToolRunning)
            }
            _ => None,
        };
        if let Some(state) = projected_state {
            transition_turn_state(&self.event_tx, &self.store, &self.context, state)
                .map_err(|error| ConversationError::Provider(error.to_string()))?;
        }
        if let RuntimeEvent::ApprovalRequested {
            approval_id,
            title,
            details,
            ..
        } = &event
        {
            let request = approval_capability_request(&self.context, title, details);
            let decision = CapabilityBroker::new(Arc::clone(&self.store))
                .authorize_request(
                    CapabilityAvailability::PermissionRequired,
                    None,
                    request.clone(),
                    now_millis(),
                )
                .map_err(ConversationError::Provider)?;
            if decision != CapabilityPolicyDecision::NeedsUser {
                return Err(ConversationError::Provider(
                    "privileged provider approval did not enter needs-user policy".to_string(),
                ));
            }
            self.approvals
                .lock()
                .map_err(|_| ConversationError::Provider("approval map poisoned".to_string()))?
                .insert(
                    approval_id.clone(),
                    PendingRuntimeApproval {
                        provider_key: self.provider_key.clone(),
                        request,
                    },
                );
        }
        self.event_tx
            .send(event)
            .map_err(|_| ConversationError::EventConsumerClosed)
    }
}

fn runtime_agent_id_from_conversation(conversation_id: &ConversationId) -> Option<String> {
    conversation_id
        .as_str()
        .strip_prefix("codex:agent:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn approval_capability_request(
    context: &RunContext,
    title: &str,
    details: &Value,
) -> CapabilityRequest {
    let explicit = details
        .get("capability")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let has_network_context = details.get("networkApprovalContext").is_some()
        || details.get("network_approval_context").is_some();
    let capability = if explicit == Some("computer.control") {
        "computer.input.control"
    } else if has_network_context {
        "browser.network"
    } else if title.contains("执行命令") {
        "shell.execute"
    } else if title.contains("修改文件") || title.contains("应用补丁") {
        "filesystem.write"
    } else if title.contains("扩展权限") {
        "runtime.permissions.expand"
    } else {
        "agent.tool.approval"
    };
    let agent_id = runtime_agent_id_from_conversation(&context.conversation_id);
    CapabilityRequest {
        actor: agent_id
            .as_ref()
            .map(|id| format!("agent:{id}"))
            .unwrap_or_else(|| "agent-runtime".to_string()),
        agent_id,
        conversation_id: context.conversation_id.clone(),
        run_id: Some(context.run_id.clone()),
        capability: capability.to_string(),
        target: serde_json::json!({
            "kind": details.get("kind").cloned().unwrap_or(Value::Null),
            "subject": details.get("subject").cloned().unwrap_or(Value::Null),
            "location": details.get("location").cloned().unwrap_or(Value::Null),
        }),
        intent: title.to_string(),
    }
}

fn transition_turn_state(
    event_tx: &Sender<RuntimeEvent>,
    store: &RuntimeStore,
    context: &RunContext,
    state: TurnState,
) -> Result<(), RuntimeError> {
    let Some(sequence) = context
        .actor
        .set_state(&context.turn_id, state)
        .map_err(RuntimeError::Synchronization)?
    else {
        return Ok(());
    };
    let active_run = (!state.terminal()).then_some(&context.run_id);
    store.set_turn_state(&context.turn_id, state, active_run)?;
    store.set_run_state(
        &context.run_id,
        state,
        state.terminal().then_some(now_millis()),
    )?;
    event_tx
        .send(RuntimeEvent::TurnStateChanged {
            operation_id: OperationId(context.run_id.0.clone()),
            turn_id: context.turn_id.clone(),
            run_id: context.run_id.clone(),
            conversation_id: context.conversation_id.clone(),
            state,
            sequence,
        })
        .map_err(|_| RuntimeError::EventConsumerClosed)
}

fn handoff_prompt(intent: &HandoffIntent) -> String {
    let constraints = if intent.constraints.is_null() {
        String::new()
    } else {
        format!(
            "\nConstraints: {}",
            serde_json::to_string(&intent.constraints).unwrap_or_else(|_| "{}".to_string())
        )
    };
    let expected = intent
        .expected_output
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("\nExpected output: {}", value.trim()))
        .unwrap_or_default();
    format!(
        "Agent handoff (depth {}):\nTask: {}{}{}",
        intent.depth, intent.task, constraints, expected
    )
}

fn validate_runtime_state_key(key: &str) -> Result<(), RuntimeError> {
    let key = key.trim();
    if key.is_empty() || key.len() > 240 || key.chars().any(char::is_control) {
        return Err(RuntimeError::Collaboration(
            "runtime state key must be non-empty, <= 240 bytes, and contain no control characters"
                .to_string(),
        ));
    }
    Ok(())
}

fn require_capability_execution_allowed(
    capability_id: &str,
    decision: CapabilityPolicyDecision,
) -> Result<(), RuntimeError> {
    match decision {
        CapabilityPolicyDecision::Allow => Ok(()),
        CapabilityPolicyDecision::NeedsUser => Err(RuntimeError::CapabilityUnavailable {
            capability_id: capability_id.to_string(),
            reason: "capability requires explicit user permission before execution".to_string(),
        }),
        CapabilityPolicyDecision::Deny => Err(RuntimeError::CapabilityUnavailable {
            capability_id: capability_id.to_string(),
            reason: "capability policy denied this request".to_string(),
        }),
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("runtime initialization failed: {0}")]
    Initialization(String),
    #[error("Agent backend initialization failed: {0}")]
    AgentInitialization(String),
    #[error("Agent backend request failed: {0}")]
    AgentBackend(String),
    #[error("remote Agent gateways are forbidden in embedded runtime builds")]
    RemoteAgentForbidden,
    #[error("remote model provider support was not compiled")]
    RemoteModelNotCompiled,
    #[error("telemetry support was not compiled")]
    TelemetryNotCompiled,
    #[error("message text must not be empty")]
    EmptyMessage,
    #[error("logical retry target was not found or is not terminal: {0}")]
    RetryTargetNotFound(String),
    #[error(transparent)]
    Conversation(#[from] ConversationError),
    #[error("runtime event consumer is closed")]
    EventConsumerClosed,
    #[error("runtime synchronization failed: {0}")]
    Synchronization(String),
    #[error("local Mini App runtime failed: {0}")]
    LocalPlugin(String),
    #[error("runtime store failed: {0}")]
    RuntimeStore(#[from] RuntimeStoreError),
    #[error("capability broker failed: {0}")]
    CapabilityBroker(String),
    #[error("Agent collaboration failed: {0}")]
    Collaboration(String),
    #[error("capability not found: {0}")]
    CapabilityNotFound(String),
    #[error("capability unavailable: {capability_id}: {reason}")]
    CapabilityUnavailable {
        capability_id: String,
        reason: String,
    },
}

fn local_plugin_commands(plugin_id: Option<&str>) -> Vec<PluginCommandDescriptor> {
    let mut commands = fabushi_official_miniapps::OFFICIAL_PLUGIN_IDS
        .iter()
        .filter(|candidate| plugin_id.is_none_or(|plugin_id| plugin_id == **candidate))
        .filter_map(|plugin_id| app_definition(plugin_id))
        .flat_map(|definition| {
            definition
                .commands
                .into_iter()
                .filter_map(move |(command, tool)| {
                    let descriptor = definition.tools.iter().find(|descriptor| {
                        descriptor.get("name").and_then(Value::as_str) == Some(tool.as_str())
                    })?;
                    Some(PluginCommandDescriptor {
                        plugin_id: definition.id.clone(),
                        command,
                        tool,
                        input_schema: descriptor
                            .get("inputSchema")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!({"type":"object"})),
                        annotations: descriptor
                            .get("annotations")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!({})),
                    })
                })
        })
        .collect::<Vec<_>>();
    commands.sort_by(|left, right| {
        left.plugin_id
            .cmp(&right.plugin_id)
            .then_with(|| left.command.cmp(&right.command))
    });
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mahayana_agent::AgentEvent;
    use mahayana_agent::AgentMessageRequest;
    use mahayana_agent::ApprovalResolution;
    use mahayana_agent::SharedAgentEventSink;
    use mahayana_agent::StartThreadRequest;
    use mahayana_core::AgentThreadId;
    use mahayana_core::ApprovalDecision;
    use mahayana_core::CODEX_ASSISTANT_CONVERSATION_ID;
    use mahayana_core::Message;
    use mahayana_core::MessageId;
    use mahayana_core::MessageRole;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct EchoAgent;

    #[async_trait]
    impl AgentBackend for EchoAgent {
        async fn start_thread(
            &self,
            _request: StartThreadRequest,
        ) -> Result<AgentThreadId, AgentError> {
            AgentThreadId::new("thread:test")
                .map_err(|error| AgentError::Backend(error.to_string()))
        }

        async fn send_message(
            &self,
            request: AgentMessageRequest,
            events: SharedAgentEventSink,
        ) -> Result<(), AgentError> {
            events.emit(AgentEvent::MessageDelta {
                delta: "大乘：".to_string(),
            })?;
            events.emit(AgentEvent::MessageCompleted {
                message: Message {
                    id: MessageId::generated("message"),
                    conversation_id: request.conversation_id,
                    role: MessageRole::Assistant,
                    text: format!("大乘：{}", request.text),
                    created_at_ms: 0,
                    metadata: Value::Null,
                },
            })
        }

        async fn interrupt(&self, _operation_id: &OperationId) -> Result<(), AgentError> {
            Ok(())
        }

        async fn resolve_approval(
            &self,
            _resolution: ApprovalResolution,
        ) -> Result<(), AgentError> {
            Ok(())
        }

        fn name(&self) -> &'static str {
            "echo-test"
        }
    }

    #[test]
    fn routes_codex_contact_and_streams_events() {
        let runtime = RuntimeBuilder::new(RuntimeConfig::default())
            .with_agent_backend(Arc::new(EchoAgent))
            .expect("register agent")
            .build()
            .expect("build runtime");
        let ready = runtime
            .receive(Duration::from_millis(10))
            .expect("receive ready")
            .expect("ready event");
        assert!(matches!(ready, RuntimeEvent::Ready { .. }));

        let response = runtime
            .execute(RuntimeCommand::SendMessage {
                conversation_id: ConversationId(CODEX_ASSISTANT_CONVERSATION_ID.to_string()),
                text: "你好".to_string(),
                client_message_id: None,
                retry_of_client_message_id: None,
                inference_provider: None,
                hidden: false,
            })
            .expect("send message");
        let RuntimeResponse::Accepted { operation_id } = response else {
            panic!("expected accepted response");
        };

        let mut saw_delta = false;
        let mut saw_message = false;
        let mut saw_complete = false;
        let mut lifecycle = Vec::new();
        for _ in 0..12 {
            let event = runtime
                .receive(Duration::from_secs(1))
                .expect("receive event")
                .expect("event before timeout");
            match event {
                RuntimeEvent::TurnStateChanged { state, .. } => {
                    lifecycle.push(state);
                }
                RuntimeEvent::MessageDelta {
                    operation_id: event_operation,
                    delta,
                    ..
                } => {
                    assert_eq!(event_operation, operation_id);
                    assert_eq!(delta, "大乘：");
                    saw_delta = true;
                }
                RuntimeEvent::MessageCompleted { message, .. } => {
                    assert_eq!(message.text, "大乘：你好");
                    saw_message = true;
                }
                RuntimeEvent::OperationCompleted {
                    operation_id: event_operation,
                } => {
                    assert_eq!(event_operation, operation_id);
                    saw_complete = true;
                    break;
                }
                _ => {}
            }
        }
        assert!(saw_delta && saw_message && saw_complete);
        assert!(lifecycle.contains(&TurnState::Accepted));
        assert!(lifecycle.contains(&TurnState::Preparing));
        assert!(lifecycle.contains(&TurnState::Thinking));
        assert!(lifecycle.contains(&TurnState::Completed));
    }

    struct CountingAgent {
        starts: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl AgentBackend for CountingAgent {
        async fn start_thread(
            &self,
            _request: StartThreadRequest,
        ) -> Result<AgentThreadId, AgentError> {
            let sequence = self.starts.fetch_add(1, Ordering::SeqCst) + 1;
            AgentThreadId::new(format!("thread:warmup:{sequence}"))
                .map_err(|error| AgentError::Backend(error.to_string()))
        }

        async fn send_message(
            &self,
            request: AgentMessageRequest,
            events: SharedAgentEventSink,
        ) -> Result<(), AgentError> {
            events.emit(AgentEvent::MessageCompleted {
                message: Message {
                    id: MessageId::generated("message"),
                    conversation_id: request.conversation_id,
                    role: MessageRole::Assistant,
                    text: "ready".to_string(),
                    created_at_ms: 0,
                    metadata: Value::Null,
                },
            })
        }

        async fn interrupt(&self, _operation_id: &OperationId) -> Result<(), AgentError> {
            Ok(())
        }

        async fn resolve_approval(
            &self,
            _resolution: ApprovalResolution,
        ) -> Result<(), AgentError> {
            Ok(())
        }

        fn name(&self) -> &'static str {
            "counting-test"
        }
    }

    #[test]
    fn warmup_opens_agent_session_once_before_first_message() {
        let starts = Arc::new(AtomicUsize::new(0));
        let backend: Arc<dyn AgentBackend> = Arc::new(CountingAgent {
            starts: Arc::clone(&starts),
        });
        let runtime = RuntimeBuilder::new(RuntimeConfig::default())
            .with_agent_backend(backend)
            .expect("register agent")
            .build()
            .expect("build runtime");
        let conversation_id =
            ConversationId(CODEX_ASSISTANT_CONVERSATION_ID.to_string());

        runtime
            .warmup_conversation(conversation_id.clone())
            .expect("warm conversation");
        runtime
            .warmup_conversation(conversation_id.clone())
            .expect("repeat warm conversation");
        assert_eq!(starts.load(Ordering::SeqCst), 1);

        let response = runtime
            .execute(RuntimeCommand::SendMessage {
                conversation_id,
                text: "first visible prompt".to_string(),
                client_message_id: Some("first-visible-prompt".to_string()),
                retry_of_client_message_id: None,
                inference_provider: None,
                hidden: false,
            })
            .expect("send first message");
        let RuntimeResponse::Accepted { operation_id } = response else {
            panic!("expected accepted response");
        };
        for _ in 0..4 {
            let Some(event) = runtime
                .receive(Duration::from_secs(1))
                .expect("receive warmup regression event")
            else {
                continue;
            };
            if matches!(
                event,
                RuntimeEvent::OperationCompleted {
                    operation_id: completed
                } if completed == operation_id
            ) {
                break;
            }
        }
        assert_eq!(starts.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn rejects_cloud_agent_configuration_at_runtime_creation() {
        let config = RuntimeConfig {
            remote_agent_enabled: true,
            ..RuntimeConfig::default()
        };
        let result = RuntimeBuilder::new(config)
            .with_agent_backend(Arc::new(EchoAgent))
            .expect("register agent")
            .build();
        assert!(matches!(result, Err(RuntimeError::RemoteAgentForbidden)));
    }

    #[test]
    fn capability_execution_gate_is_fail_closed_for_needs_user_and_deny() {
        assert!(require_capability_execution_allowed(
            "capability:test",
            CapabilityPolicyDecision::Allow,
        ).is_ok());

        for decision in [
            CapabilityPolicyDecision::NeedsUser,
            CapabilityPolicyDecision::Deny,
        ] {
            let error = require_capability_execution_allowed("capability:test", decision)
                .expect_err("non-Allow decisions must never start an execution run");
            assert!(matches!(
                error,
                RuntimeError::CapabilityUnavailable { capability_id, .. }
                    if capability_id == "capability:test"
            ));
        }
    }

    #[test]
    fn provider_approval_audit_is_sanitized_and_capability_typed() {
        let conversation_id = ConversationId("codex:agent:research".to_string());
        let actor = ConversationActorRegistry::default()
            .actor(&conversation_id)
            .expect("create conversation actor");
        let context = RunContext {
            turn_id: TurnId::generated("turn"),
            run_id: RunId::generated("run"),
            conversation_id,
            actor,
        };
        let request = approval_capability_request(
            &context,
            "AI 请求控制这台电脑",
            &serde_json::json!({
                "kind": "local-tool",
                "capability": "computer.control",
                "subject": "Computer · click",
                "location": "local",
                "detail": "password=secret",
                "reason": "OTP 123456"
            }),
        );
        assert_eq!(request.capability, "computer.input.control");
        assert_eq!(request.agent_id.as_deref(), Some("research"));
        let encoded = serde_json::to_string(&request.target).expect("serialize safe target");
        assert!(!encoded.contains("secret"));
        assert!(!encoded.contains("123456"));
        assert!(!encoded.contains("detail"));
        assert!(!encoded.contains("reason"));
    }

    #[test]
    fn approval_decision_wire_values_remain_stable() {
        assert_eq!(
            serde_json::to_value(RuntimeApprovalDecision::AcceptForSession).expect("serialize decision"),
            "acceptForSession"
        );
    }
}