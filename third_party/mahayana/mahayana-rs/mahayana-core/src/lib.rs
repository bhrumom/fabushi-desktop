//! Stable product contracts shared by every Mahayana surface.

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

pub mod capability;

pub const RUNTIME_ABI_VERSION: u32 = 1;
pub const CONVERSATION_SCHEMA_VERSION: u32 = 1;
pub const MODEL_RUNTIME_VERSION: u32 = 1;
pub const MAHAYANA_AI_CONVERSATION_ID: &str = "mahayana-ai:agent:assistant";
/// Legacy source-compatible alias. New product surfaces must use `MAHAYANA_AI_CONVERSATION_ID`.
pub const CODEX_ASSISTANT_CONVERSATION_ID: &str = MAHAYANA_AI_CONVERSATION_ID;
pub const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-chat";
pub const DEFAULT_DACHENG_RESPONSES_BASE_URL: &str = "https://api.ombhrum.com/codex-deepseek/v1";

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ContractError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            pub fn generated(prefix: &str) -> Self {
                Self(format!("{prefix}:{}", uuid::Uuid::new_v4()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

string_id!(ConversationId);
string_id!(MessageId);
string_id!(OperationId);
string_id!(ApprovalId);
string_id!(AgentThreadId);
string_id!(TurnId);
string_id!(RunId);
string_id!(IntentId);

/// Canonical lifecycle for one user-visible logical turn.
///
/// UI surfaces project this state; they must not infer thinking/tool/streaming
/// from DOM shape, arbitrary strings, or a generic busy flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TurnState {
    Accepted,
    Queued,
    Preparing,
    Thinking,
    ToolRunning,
    Streaming,
    WaitingUser,
    Completed,
    Failed,
    Cancelled,
    Recovering,
}

impl TurnState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Queued => "queued",
            Self::Preparing => "preparing",
            Self::Thinking => "thinking",
            Self::ToolRunning => "tool-running",
            Self::Streaming => "streaming",
            Self::WaitingUser => "waiting-user",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Recovering => "recovering",
        }
    }

    pub const fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalTurn {
    pub id: TurnId,
    pub conversation_id: ConversationId,
    pub user_message_id: Option<MessageId>,
    pub created_at_ms: i64,
    pub state: TurnState,
    pub active_run_id: Option<RunId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRun {
    pub id: RunId,
    pub turn_id: TurnId,
    pub generation: u32,
    pub provider: String,
    pub started_at_ms: i64,
    pub finished_at_ms: Option<i64>,
    pub state: TurnState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffIntent {
    pub id: IntentId,
    pub target_agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_conversation_id: Option<ConversationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inference_provider: Option<String>,
    pub task: String,
    #[serde(default)]
    pub constraints: Value,
    #[serde(default)]
    pub expected_output: Option<String>,
    pub origin_run: RunId,
    pub depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskUserRequest {
    pub id: IntentId,
    pub turn_id: TurnId,
    pub run_id: RunId,
    pub question: String,
}


#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildProfile {
    #[default]
    DesktopFull,
    MobileEmbedded,
    WebWasm,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelProviderMode {
    LocalModel,
    LocalLoopback,
    #[default]
    FirstPartyDacheng,
    UserConfiguredRemote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ModelConfig {
    pub provider: ModelProviderMode,
    pub model: String,
    pub base_url: Option<String>,
    /// The key used to locate a secret in platform secure storage. Secret
    /// values are never part of this serializable runtime configuration.
    pub credential_key: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: ModelProviderMode::FirstPartyDacheng,
            model: DEFAULT_DEEPSEEK_MODEL.to_string(),
            base_url: Some(DEFAULT_DACHENG_RESPONSES_BASE_URL.to_string()),
            credential_key: Some("mahayana.account.session".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RuntimeConfig {
    #[serde(default)]
    pub build_profile: BuildProfile,
    #[serde(default)]
    pub model: ModelConfig,
    pub data_dir: Option<PathBuf>,
    pub workspace_roots: Vec<PathBuf>,
    #[serde(default)]
    pub remote_agent_enabled: bool,
    #[serde(default)]
    pub telemetry_enabled: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            build_profile: BuildProfile::DesktopFull,
            model: ModelConfig::default(),
            data_dir: None,
            workspace_roots: Vec::new(),
            remote_agent_enabled: false,
            telemetry_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PeerKind {
    MahayanaAi,
    /// Read compatibility for persisted pre-sovereign conversation payloads.
    CodexAi,
    TelegramContact {
        user_id: i64,
    },
    MahayanaContact {
        contact_id: String,
    },
    MiniApp {
        app_id: String,
    },
}

impl PeerKind {
    pub fn provider_key(&self) -> &'static str {
        match self {
            Self::MahayanaAi | Self::CodexAi => "mahayana-ai",
            Self::TelegramContact { .. } => "telegram",
            Self::MahayanaContact { .. } => "mahayana-social",
            Self::MiniApp { .. } => "miniapp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    pub peer: PeerKind,
    pub pinned: bool,
    pub unread_count: u32,
    pub updated_at_ms: i64,
}

impl Conversation {
    pub fn mahayana_assistant() -> Self {
        Self {
            id: ConversationId(MAHAYANA_AI_CONVERSATION_ID.to_string()),
            title: "Mahayana（大乘 AI）".to_string(),
            peer: PeerKind::MahayanaAi,
            pinned: true,
            unread_count: 0,
            updated_at_ms: 0,
        }
    }

    /// Source-compatible helper for callers not yet migrated. It intentionally
    /// returns the sovereign Mahayana conversation, never a new Codex identity.
    pub fn codex_assistant() -> Self {
        Self::mahayana_assistant()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageRole {
    User,
    Assistant,
    Contact,
    MiniApp,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    pub role: MessageRole,
    pub text: String,
    pub created_at_ms: i64,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCommandDescriptor {
    pub plugin_id: String,
    pub command: String,
    pub tool: String,
    pub input_schema: Value,
    #[serde(default)]
    pub annotations: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum RuntimeCommand {
    #[serde(rename = "mahayana.runtime.status")]
    Status,
    #[serde(rename = "mahayana.runtime.workspaceState.get")]
    WorkspaceStateGet { key: String },
    #[serde(rename = "mahayana.runtime.workspaceState.set")]
    WorkspaceStateSet {
        key: String,
        value: Value,
    },
    #[serde(rename = "mahayana.conversation.list")]
    ListConversations,
    #[serde(rename = "mahayana.capability.list")]
    ListCapabilities { query: Option<String> },
    #[serde(rename = "mahayana.capability.authorize")]
    AuthorizeCapability {
        request: capability::CapabilityRequest,
        availability: capability::CapabilityAvailability,
        #[serde(rename = "unavailableReason", default, skip_serializing_if = "Option::is_none")]
        unavailable_reason: Option<String>,
    },
    #[serde(rename = "mahayana.capability.invoke")]
    InvokeCapability {
        #[serde(rename = "capabilityId")]
        capability_id: String,
        text: String,
        #[serde(rename = "clientMessageId")]
        client_message_id: Option<String>,
    },
    #[serde(rename = "mahayana.plugin.commands")]
    ListPluginCommands {
        #[serde(rename = "pluginId")]
        plugin_id: Option<String>,
    },
    #[serde(rename = "mahayana.plugin.ui")]
    PluginUi {
        #[serde(rename = "pluginId")]
        plugin_id: String,
    },
    #[serde(rename = "mahayana.plugin.approveLocal")]
    ApproveLocalPluginTool {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        tool: String,
    },
    #[serde(rename = "mahayana.plugin.callLocal")]
    CallLocalPluginTool {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        tool: String,
        #[serde(default)]
        arguments: Value,
    },
    #[serde(rename = "mahayana.mcp.servers")]
    McpServers,
    #[serde(rename = "mahayana.mcp.apps")]
    McpApps,
    #[serde(rename = "mahayana.mcp.oauth.login")]
    McpOauthLogin { server: String },
    #[serde(rename = "mahayana.mcp.oauth.logout")]
    McpOauthLogout { server: String },
    #[serde(rename = "mahayana.mcp.remove")]
    McpRemove { server: String },
    #[serde(rename = "mahayana.mcp.customInstructions")]
    McpCustomInstructions,
    #[serde(rename = "mahayana.mcp.setCustomInstructions")]
    McpSetCustomInstructions {
        server: String,
        instructions: String,
    },
    #[serde(rename = "mahayana.mcp.setToolDisabled")]
    McpSetToolDisabled {
        server: String,
        tool: String,
        disabled: bool,
    },
    #[serde(rename = "mahayana.mcp.refresh")]
    McpRefresh,
    #[serde(rename = "mahayana.mcp.tool.call")]
    McpToolCall {
        server: String,
        tool: String,
        #[serde(default)]
        arguments: Value,
    },
    #[serde(rename = "mahayana.conversation.history")]
    ConversationHistory {
        #[serde(rename = "conversationId")]
        conversation_id: ConversationId,
        limit: Option<u32>,
    },
    #[serde(rename = "mahayana.conversation.send")]
    SendMessage {
        #[serde(rename = "conversationId")]
        conversation_id: ConversationId,
        text: String,
        #[serde(rename = "clientMessageId")]
        client_message_id: Option<String>,
        #[serde(
            rename = "retryOfClientMessageId",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        retry_of_client_message_id: Option<String>,
        #[serde(rename = "inferenceProvider", default, skip_serializing_if = "Option::is_none")]
        inference_provider: Option<String>,
        #[serde(default)]
        hidden: bool,
    },
    #[serde(rename = "mahayana.agent.askUser")]
    AskUser {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        question: String,
    },
    #[serde(rename = "mahayana.agent.handoff")]
    Handoff {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "targetAgent")]
        target_agent: String,
        #[serde(rename = "targetConversationId", default, skip_serializing_if = "Option::is_none")]
        target_conversation_id: Option<ConversationId>,
        #[serde(rename = "inferenceProvider", default, skip_serializing_if = "Option::is_none")]
        inference_provider: Option<String>,
        task: String,
        #[serde(default)]
        constraints: Value,
        #[serde(rename = "expectedOutput", default, skip_serializing_if = "Option::is_none")]
        expected_output: Option<String>,
        #[serde(default)]
        depth: u8,
    },
    #[serde(rename = "mahayana.agent.externalHandoff")]
    ExternalHandoff {
        #[serde(rename = "originRunId")]
        origin_run_id: RunId,
        #[serde(rename = "originTurnId")]
        origin_turn_id: TurnId,
        #[serde(rename = "originAgent", default, skip_serializing_if = "Option::is_none")]
        origin_agent: Option<String>,
        #[serde(rename = "targetAgent")]
        target_agent: String,
        #[serde(rename = "targetConversationId", default, skip_serializing_if = "Option::is_none")]
        target_conversation_id: Option<ConversationId>,
        #[serde(rename = "inferenceProvider", default, skip_serializing_if = "Option::is_none")]
        inference_provider: Option<String>,
        task: String,
        #[serde(default)]
        constraints: Value,
        #[serde(rename = "expectedOutput", default, skip_serializing_if = "Option::is_none")]
        expected_output: Option<String>,
        #[serde(default)]
        depth: u8,
    },
    #[serde(rename = "mahayana.operation.interrupt")]
    Interrupt {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.approval.resolve")]
    ResolveApproval {
        #[serde(rename = "approvalId")]
        approval_id: ApprovalId,
        decision: ApprovalDecision,
        #[serde(default)]
        payload: Value,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApprovalDecision {
    Accept,
    AcceptForSession,
    Decline,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum RuntimeResponse {
    #[serde(rename = "mahayana.runtime.status")]
    Status(RuntimeStatus),
    #[serde(rename = "mahayana.runtime.workspaceState")]
    RuntimeWorkspaceState {
        key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
    #[serde(rename = "mahayana.conversation.list")]
    Conversations { data: Vec<Conversation> },
    #[serde(rename = "mahayana.capability.list")]
    Capabilities {
        data: Vec<capability::CapabilityDescriptor>,
    },
    #[serde(rename = "mahayana.capability.decision")]
    CapabilityDecision {
        decision: capability::CapabilityPolicyDecision,
    },
    #[serde(rename = "mahayana.capability.accepted")]
    CapabilityAccepted {
        #[serde(rename = "capabilityId")]
        capability_id: String,
        #[serde(rename = "conversationId")]
        conversation_id: ConversationId,
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.plugin.commands")]
    PluginCommands { data: Vec<PluginCommandDescriptor> },
    #[serde(rename = "mahayana.plugin.ui")]
    PluginUi {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        html: String,
    },
    #[serde(rename = "mahayana.plugin.approvedLocal")]
    LocalPluginToolApproved {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        tool: String,
    },
    #[serde(rename = "mahayana.plugin.localResult")]
    LocalPluginToolResult {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        tool: String,
        result: Value,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        progress: Vec<Value>,
    },
    #[serde(rename = "mahayana.mcp.servers")]
    McpServers { data: Vec<Value> },
    #[serde(rename = "mahayana.mcp.apps")]
    McpApps { data: Vec<Value> },
    #[serde(rename = "mahayana.mcp.oauth")]
    McpOauth {
        server: String,
        #[serde(rename = "authorizationUrl")]
        authorization_url: Option<String>,
        #[serde(default)]
        removed: bool,
    },
    #[serde(rename = "mahayana.mcp.removed")]
    McpRemoved { server: String, removed: bool },
    #[serde(rename = "mahayana.mcp.customInstructions")]
    McpCustomInstructions {
        instructions: HashMap<String, String>,
    },
    #[serde(rename = "mahayana.mcp.customInstructionsUpdated")]
    McpCustomInstructionsUpdated { server: String },
    #[serde(rename = "mahayana.mcp.toolDisabledUpdated")]
    McpToolDisabledUpdated {
        server: String,
        disabled_tools: Vec<String>,
    },
    #[serde(rename = "mahayana.mcp.refreshed")]
    McpRefreshed,
    #[serde(rename = "mahayana.mcp.tool.result")]
    McpToolResult {
        server: String,
        tool: String,
        result: Value,
    },
    #[serde(rename = "mahayana.conversation.history")]
    History { data: Vec<Message> },
    #[serde(rename = "mahayana.agent.waitingUser")]
    WaitingUser {
        #[serde(rename = "intentId")]
        intent_id: IntentId,
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.agent.handoffQueued")]
    HandoffQueued {
        #[serde(rename = "intentId")]
        intent_id: IntentId,
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "targetOperationId")]
        target_operation_id: OperationId,
    },
    #[serde(rename = "mahayana.operation.accepted")]
    Accepted {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.operation.interrupted")]
    Interrupted {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.approval.resolved")]
    ApprovalResolved {
        #[serde(rename = "approvalId")]
        approval_id: ApprovalId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub runtime_abi_version: u32,
    pub conversation_schema_version: u32,
    pub model_runtime_version: u32,
    pub build_profile: BuildProfile,
    pub model_provider: ModelProviderMode,
    pub model: String,
    pub remote_agent_enabled: bool,
    pub telemetry_enabled: bool,
    pub providers: Vec<String>,
}

/// Provider-reported model token counts. These values come from the selected
/// Mahayana model backend and are never estimated by the product host.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTokenUsage {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
}

/// Latest model usage checkpoint for a running operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTokenUsageSnapshot {
    /// Codex supplies the cumulative thread total. Lightweight Responses-only
    /// runtimes omit it instead of pretending a client-side estimate is authoritative.
    pub total: Option<ModelTokenUsage>,
    pub last: ModelTokenUsage,
    pub model_context_window: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeActivityStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum RuntimeEvent {
    #[serde(rename = "mahayana.runtime.ready")]
    Ready { status: RuntimeStatus },
    #[serde(rename = "mahayana.turn.state")]
    TurnStateChanged {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "turnId")]
        turn_id: TurnId,
        #[serde(rename = "runId")]
        run_id: RunId,
        #[serde(rename = "conversationId")]
        conversation_id: ConversationId,
        state: TurnState,
        sequence: u64,
    },
    #[serde(rename = "mahayana.message.delta")]
    MessageDelta {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "conversationId")]
        conversation_id: ConversationId,
        delta: String,
    },
    #[serde(rename = "mahayana.message.completed")]
    MessageCompleted {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        message: Message,
    },
    #[serde(rename = "mahayana.model.usage.updated")]
    ModelUsageUpdated {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        usage: ModelTokenUsageSnapshot,
    },
    #[serde(rename = "mahayana.approval.requested")]
    ApprovalRequested {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "approvalId")]
        approval_id: ApprovalId,
        title: String,
        details: Value,
    },
    #[serde(rename = "mahayana.plugin.progress")]
    PluginProgress {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "pluginId")]
        plugin_id: String,
        tool: String,
        progress: u64,
        total: u64,
        message: String,
    },
    #[serde(rename = "mahayana.agent.activity")]
    AgentActivity {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        #[serde(rename = "stepId")]
        step_id: String,
        kind: String,
        title: String,
        detail: Option<String>,
        status: RuntimeActivityStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
    },
    #[serde(rename = "mahayana.operation.completed")]
    OperationCompleted {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
    },
    #[serde(rename = "mahayana.operation.failed")]
    OperationFailed {
        #[serde(rename = "operationId")]
        operation_id: OperationId,
        code: String,
        message: String,
    },
    #[serde(rename = "mahayana.runtime.lagged")]
    Lagged { skipped: usize },
    #[serde(rename = "mahayana.provider.degraded")]
    ProviderDegraded { provider: String, message: String },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("{0} must not be empty")]
    EmptyIdentifier(&'static str),
    #[error("message text must not be empty")]
    EmptyMessage,
    #[error("unsupported conversation id: {0}")]
    UnsupportedConversation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_state_wire_contract_is_explicit_and_stable() {
        assert_eq!(
            serde_json::to_value(TurnState::WaitingUser).expect("serialize"),
            serde_json::Value::String("waiting-user".into())
        );
        assert_eq!(TurnState::ToolRunning.as_str(), "tool-running");
        assert!(TurnState::Completed.terminal());
        assert!(!TurnState::Streaming.terminal());
    }

    #[test]
    fn runtime_workspace_state_wire_contract_is_explicit() {
        let command = RuntimeCommand::WorkspaceStateSet {
            key: "agent-workspace:drafts:v2".into(),
            value: serde_json::json!({"agent:a": {"text": "hello"}}),
        };
        let json = serde_json::to_value(command).expect("serialize workspace state command");
        assert_eq!(json["@type"], "mahayana.runtime.workspaceState.set");
        assert_eq!(json["key"], "agent-workspace:drafts:v2");
    }

    #[test]
    fn default_config_is_first_party_deepseek_without_remote_agent() {
        let config = RuntimeConfig::default();
        assert_eq!(config.model.provider, ModelProviderMode::FirstPartyDacheng);
        assert_eq!(config.model.model, "deepseek-chat");
        assert!(!config.remote_agent_enabled);
        assert!(!config.telemetry_enabled);
    }

    #[test]
    fn command_wire_contract_uses_stable_type_and_camel_case_ids() {
        let command = RuntimeCommand::SendMessage {
            conversation_id: ConversationId(CODEX_ASSISTANT_CONVERSATION_ID.to_string()),
            text: "你好".to_string(),
            client_message_id: Some("client-1".to_string()),
                            inference_provider: None,
            hidden: false,
        };
        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["@type"], "mahayana.conversation.send");
        assert_eq!(json["conversationId"], CODEX_ASSISTANT_CONVERSATION_ID);
        assert_eq!(json["clientMessageId"], "client-1");
    }

    #[test]
    fn legacy_codex_helper_returns_the_pinned_sovereign_conversation() {
        let conversation = Conversation::codex_assistant();
        assert_eq!(conversation.id.as_str(), CODEX_ASSISTANT_CONVERSATION_ID);
        assert_eq!(conversation.title, "Mahayana（大乘 AI）");
        assert_eq!(conversation.peer, PeerKind::MahayanaAi);
        assert!(conversation.pinned);
    }

    #[test]
    fn capability_command_wire_contract_uses_stable_selector() {
        let command = RuntimeCommand::InvokeCapability {
            capability_id: "miniapp.bot-father".to_string(),
            text: "创建一个机器人".to_string(),
            client_message_id: Some("client-2".to_string()),
        };
        let json = serde_json::to_value(command).expect("serialize capability command");
        assert_eq!(json["@type"], "mahayana.capability.invoke");
        assert_eq!(json["capabilityId"], "miniapp.bot-father");
        assert_eq!(json["clientMessageId"], "client-2");
    }
}