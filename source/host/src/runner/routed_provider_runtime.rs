use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::extensions::inference::cursor_inference_transport::CursorInferenceAuth;
use crate::extensions::session::typed_replay::TypedReplayEntry;
use crate::extensions::inference::provider_session::{
    FabushiResponsesRequestBinding, McpCatalogRefusal, ProviderMessage, ProviderPartialToolCall,
    ProviderSessionError, ProviderTokenUsage,
    ProviderToolStepReminderHook, RoutedMcpMetaToolDefinition, RoutedProvider,
    RoutedProviderCheckpoint, RoutedProviderOptions, RoutedToolDefinition,
    resolve_codex_request_model,
    run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps,
};
use crate::host_request_context::HostRequestContext;
use crate::runner::coordinator_tool_relay::CoordinatorToolExecutionError;
use crate::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryEvent, ProviderRetryReport,
    RoutedProviderAttemptControl, RoutedProviderAttemptExecutor, RoutedProviderCheckpointStore,
    ToolEffectUnknownResult,
};
use crate::runner::system_prompt_assembly::render_request_context_system_prompt_with_capabilities;
use crate::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
use crate::runner::tools::mcp_meta_tools::{
    execute_routed_tool_with_delivery_tracker, execute_routed_tool_with_timeout,
};

pub const ROUTED_MCP_PROTOCOL_VERSION: &str = "2025-03-26";
pub const ROUTED_MCP_MAX_BODY_BYTES: usize = 1_048_576;

#[derive(Clone, Default)]
pub struct RoutedProviderCancellation {
    cancelled: Arc<AtomicBool>,
    reason: Arc<Mutex<Option<String>>>,
}

impl RoutedProviderCancellation {
    pub fn cancel(&self, reason: impl Into<String>) -> bool {
        let first = !self.cancelled.swap(true, Ordering::AcqRel);
        if first {
            if let Ok(mut slot) = self.reason.lock() {
                *slot = Some(reason.into());
            }
        }
        first
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(crate) fn flag(&self) -> &AtomicBool {
        &self.cancelled
    }

    pub(crate) fn shares_cancellation_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cancelled, &other.cancelled)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason.lock().ok().and_then(|reason| reason.clone())
    }

    pub fn check(&self) -> Result<(), ProviderSessionError> {
        if self.is_cancelled() {
            Err(ProviderSessionError::Transport(
                self.reason().unwrap_or_else(|| "Runner provider request cancelled".into()),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct ActiveRoutedProviderTask {
    agent_id: Option<String>,
    cancellation: RoutedProviderCancellation,
}

#[derive(Default)]
pub struct RoutedProviderTaskRegistry {
    active: Mutex<HashMap<String, ActiveRoutedProviderTask>>,
}

impl RoutedProviderTaskRegistry {
    pub fn register(
        &self,
        stream_id: &str,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        self.register_internal(None, stream_id, RoutedProviderCancellation::default())
    }

    pub fn register_for_agent(
        &self,
        agent_id: &str,
        stream_id: &str,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        self.register_for_agent_with_cancellation(
            agent_id,
            stream_id,
            RoutedProviderCancellation::default(),
        )
    }

    /// Index the admitted turn's existing token without resetting cancellation.
    pub fn register_for_agent_with_cancellation(
        &self,
        agent_id: &str,
        stream_id: &str,
        cancellation: RoutedProviderCancellation,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err(ProviderSessionError::Protocol(
                "Runner provider agent id is empty".into(),
            ));
        }
        self.register_internal(Some(agent_id.to_string()), stream_id, cancellation)
    }

    fn register_internal(
        &self,
        agent_id: Option<String>,
        stream_id: &str,
        cancellation: RoutedProviderCancellation,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        let stream_id = stream_id.trim();
        if stream_id.is_empty() {
            return Err(ProviderSessionError::Protocol(
                "Runner provider stream id is empty".into(),
            ));
        }
        let mut active = self
            .active
            .lock()
            .map_err(|_| ProviderSessionError::Protocol(
                "Runner provider registry lock poisoned".into(),
            ))?;
        if active.contains_key(stream_id) {
            return Err(ProviderSessionError::Protocol(format!(
                "Runner provider stream already exists: {stream_id}"
            )));
        }
        active.insert(
            stream_id.to_string(),
            ActiveRoutedProviderTask {
                agent_id,
                cancellation: cancellation.clone(),
            },
        );
        Ok(cancellation)
    }

    pub fn cancel(&self, stream_id: &str, reason: impl Into<String>) -> bool {
        let reason = reason.into();
        self.active
            .lock()
            .ok()
            .and_then(|active| active.get(stream_id).cloned())
            .is_some_and(|task| task.cancellation.cancel(reason))
    }

    pub fn cancel_for_agent(
        &self,
        agent_id: &str,
        stream_id: &str,
        reason: impl Into<String>,
    ) -> bool {
        let reason = reason.into();
        let cancellation = self.active.lock().ok().and_then(|active| {
            active.get(stream_id)
                .filter(|task| task.agent_id.as_deref() == Some(agent_id))
                .map(|task| task.cancellation.clone())
        });
        // The captured token remains the target if the stream is replaced.
        cancellation.is_some_and(|cancellation| cancellation.cancel(reason))
    }

    pub fn agent_id_for_stream(&self, stream_id: &str) -> Option<String> {
        self.active
            .lock()
            .ok()
            .and_then(|active| active.get(stream_id).and_then(|task| task.agent_id.clone()))
    }

    pub fn cancel_agent(&self, agent_id: &str, reason: impl Into<String>) -> usize {
        let reason = reason.into();
        let cancellations = self
            .active
            .lock()
            .map(|active| {
                active
                    .values()
                    .filter(|task| task.agent_id.as_deref() == Some(agent_id))
                    .map(|task| task.cancellation.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        cancellations
            .into_iter()
            .filter(|cancellation| cancellation.cancel(reason.clone()))
            .count()
    }

    pub fn active_stream_ids_for_agent(&self, agent_id: &str) -> Vec<String> {
        let mut streams = self
            .active
            .lock()
            .map(|active| {
                active
                    .iter()
                    .filter_map(|(stream_id, task)| {
                        (task.agent_id.as_deref() == Some(agent_id))
                            .then(|| stream_id.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        streams.sort();
        streams
    }

    pub fn finish(&self, stream_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            active.remove(stream_id);
        }
    }

    pub fn cancel_all(&self, reason: &str) {
        let cancellations = self
            .active
            .lock()
            .map(|active| {
                active
                    .values()
                    .map(|task| task.cancellation.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for cancellation in cancellations {
            cancellation.cancel(reason.to_string());
        }
    }

    pub fn active_count(&self) -> usize {
        self.active.lock().map(|active| active.len()).unwrap_or_default()
    }
}

/// Preserve the relay owner's execution disposition across the provider edge.
pub fn coordinator_tool_execution_error(error: CoordinatorToolExecutionError) -> ProviderSessionError {
    let unknown = error.outcome_unknown();
    let message = error.into_error().to_string();
    if unknown {
        ProviderSessionError::ToolOutcomeUnknown(message)
    } else {
        ProviderSessionError::Tool(message)
    }
}

pub trait RoutedToolBridge: Send + Sync {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError>;

    fn list_mcp_meta_tools(
        &self,
    ) -> Result<Vec<RoutedMcpMetaToolDefinition>, ProviderSessionError> {
        self.list_tools()
            .map(|tools| tools.into_iter().map(RoutedMcpMetaToolDefinition::from).collect())
    }
    fn observe_partial_tool_call(
        &self,
        _partial: &ProviderPartialToolCall,
    ) -> Result<(), ProviderSessionError> {
        Ok(())
    }
    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunnerRequestContextSnapshot {
    pub context: HostRequestContext,
    pub rules: Option<Vec<Value>>,
}

pub trait RunnerRequestContextSource: Send + Sync {
    fn resolve(&self) -> RunnerRequestContextSnapshot;
}

#[derive(Debug, Clone)]
pub struct ProductionRoutedProviderCheckpointStore {
    directory: PathBuf,
}

impl ProductionRoutedProviderCheckpointStore {
    pub fn new(data_dir: &Path, agent_id: &str, stream_id: &str) -> Self {
        let agent_key = format!("{:x}", Sha256::digest(agent_id.as_bytes()));
        let stream_key = format!("{:x}", Sha256::digest(stream_id.as_bytes()));
        Self {
            directory: data_dir
                .join("runner-provider-checkpoints")
                .join(agent_key)
                .join(stream_key),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn persist_with_publication(
        &self,
        checkpoint: &RoutedProviderCheckpoint,
        checkpoint_id: Uuid,
        mut sync_directory: impl FnMut(&Path) -> io::Result<()>,
    ) -> Result<String, ProviderSessionError> {
        ensure_checkpoint_directory(&self.directory, &mut sync_directory).map_err(|error| {
            ProviderSessionError::Transport(format!(
                "could not prepare Runner checkpoint directory: {error}"
            ))
        })?;
        let temporary_path = self.directory.join(format!("{checkpoint_id}.tmp"));
        let final_path = self.directory.join(format!("{checkpoint_id}.json"));
        let payload = serde_json::to_vec(checkpoint).map_err(|error| {
            ProviderSessionError::Protocol(format!(
                "could not encode Runner provider checkpoint: {error}"
            ))
        })?;
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&temporary_path).map_err(|error| {
            ProviderSessionError::Transport(format!(
                "could not persist Runner provider checkpoint: {error}"
            ))
        })?;
        // Ownership starts only after create_new succeeds. A collision must
        // never give this operation permission to clean somebody else's entry.
        let mut staged = StagedProviderCheckpoint {
            path: temporary_path,
            file: Some(file),
        };
        let write_result = {
            let file = staged.file.as_mut().expect("staged checkpoint owns its file");
            file.write_all(&payload).and_then(|()| file.sync_all())
        };
        drop(staged.file.take());
        write_result.map_err(|error| {
            ProviderSessionError::Transport(format!(
                "could not persist Runner provider checkpoint: {error}"
            ))
        })?;
        // Both names are in the same directory. Unlike rename, hard_link
        // publishes the complete synced payload without replacing a collision.
        fs::hard_link(&staged.path, &final_path).map_err(|error| {
            ProviderSessionError::Transport(format!(
                "could not commit Runner provider checkpoint: {error}"
            ))
        })?;
        // A failed barrier leaves the visible final file intact but cannot
        // grant checkpoint acceptance. The guard cleans only our staging name.
        sync_directory(&self.directory).map_err(|error| {
            ProviderSessionError::Transport(format!(
                "could not synchronize Runner provider checkpoint directory: {error}"
            ))
        })?;
        Ok(final_path.to_string_lossy().into_owned())
    }
}

impl RoutedProviderCheckpointStore for ProductionRoutedProviderCheckpointStore {
    fn persist(
        &self,
        checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        self.persist_with_publication(checkpoint, Uuid::new_v4(), sync_checkpoint_directory)
    }
}

struct StagedProviderCheckpoint {
    path: PathBuf,
    file: Option<fs::File>,
}

impl Drop for StagedProviderCheckpoint {
    fn drop(&mut self) {
        drop(self.file.take());
        // Final publication, if any, is a separate link and is never removed.
        let _ = fs::remove_file(&self.path);
    }
}

fn ensure_checkpoint_directory(
    directory: &Path,
    sync_directory: &mut impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let mut missing = Vec::new();
    let mut existing = directory;
    loop {
        if existing.as_os_str().is_empty() {
            existing = Path::new(".");
        }
        match fs::metadata(existing) {
            Ok(metadata) if metadata.is_dir() => break,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Runner checkpoint path is not a directory",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(existing);
                existing = existing.parent().ok_or(error)?;
            }
            Err(error) => return Err(error),
        }
    }
    // Creation may have succeeded during an earlier refused write whose parent
    // barrier failed. Reestablish that entry before creating any descendants.
    if let Some(parent) = existing.parent() {
        let parent = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
        sync_directory(parent)?;
    }
    for directory in missing.into_iter().rev() {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(directory) {
            Ok(()) => {}
            // Another creator may have won. Keep its mode and synchronize the
            // same parent entry instead of assuming the winner's barrier ran.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists && directory.is_dir() => {}
            Err(error) => return Err(error),
        }
        let parent = directory.parent().filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        sync_directory(parent)?;
    }
    Ok(())
}

fn sync_checkpoint_directory(directory: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::File::open(directory)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        // File sync and no-clobber hard-link publication still apply. std does
        // not provide a portable non-Unix directory durability barrier.
        let _ = directory;
        Ok(())
    }
}

struct ProductionRoutedProviderAttemptExecutor<'a> {
    provider: RoutedProvider,
    fabushi_model: Option<String>,
    codex_model: Option<String>,
    fabushi_request_binding: Option<FabushiResponsesRequestBinding>,
    data_dir: &'a Path,
    messages: &'a [ProviderMessage],
    typed_replay: Option<&'a [TypedReplayEntry]>,
    tools: &'a [RoutedToolDefinition],
    cursor_auth: Option<Arc<dyn CursorInferenceAuth>>,
    bridge: Arc<dyn RoutedToolBridge>,
    is_computer_use_subagent: bool,
    usage_sink: Option<Arc<dyn Fn(ProviderTokenUsage) + Send + Sync>>,
    max_steps: usize,
    tool_step_reminder: Option<Arc<ProviderToolStepReminderHook>>,
}

impl RoutedProviderAttemptExecutor for ProductionRoutedProviderAttemptExecutor<'_> {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(
            &RoutedProviderCheckpoint,
        ) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        attempt.check()?;
        let mut mcp_server = if self.provider == RoutedProvider::ClaudeCode {
            Some(start_routed_mcp_server_for_attempt(
                Arc::clone(&self.bridge), attempt.clone(), self.is_computer_use_subagent,
            )?)
        } else {
            None
        };
        let bridge = Arc::clone(&self.bridge);
        let tool_attempt = attempt.clone();
        let is_computer_use_subagent = self.is_computer_use_subagent;
        let mut execute_tool = move |tool: &RoutedToolDefinition, args: Value, tool_call_id: &str| {
            let operation_bridge = Arc::clone(&bridge);
            let owned_tool = tool.clone();
            let owned_tool_call_id = tool_call_id.to_string();
            execute_routed_tool_with_timeout(
                tool, &args, is_computer_use_subagent, &tool_attempt,
                {
                    let args = args.clone();
                    move || operation_bridge.call_tool(&owned_tool, args, &owned_tool_call_id)
                },
            )
        };
        let partial_bridge = Arc::clone(&self.bridge);
        let partial_attempt = attempt.clone();
        let mut on_partial_tool_call = move |partial: ProviderPartialToolCall| {
            partial_attempt.check()?;
            partial_bridge.observe_partial_tool_call(&partial)
        };
        let usage_sink = self.usage_sink.clone();
        let mut on_usage = move |usage: ProviderTokenUsage| {
            if let Some(sink) = usage_sink.as_ref() {
                sink(usage);
            }
        };
        let result = {
            let mut options = RoutedProviderOptions {
                data_dir: self.data_dir,
                typed_replay: self.typed_replay,
                fabushi_model: self.fabushi_model.as_deref(),
                codex_model: self.codex_model.as_deref(),
                fabushi_request_binding: self.fabushi_request_binding.as_ref(),
                cursor_auth: self.cursor_auth.clone(),
                tools: self.tools,
                mcp_server_url: mcp_server.as_ref().map(RoutedMcpServer::url),
                execute_tool: &mut execute_tool,
                on_text_delta,
                on_partial_tool_call: Some(&mut on_partial_tool_call),
                should_cancel,
            };
            run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps(
                self.provider,
                self.messages,
                &mut options,
                resume_from,
                on_checkpoint,
                &mut on_usage,
                self.max_steps,
                self.tool_step_reminder.as_deref(),
            )
        };
        if let Some(server) = mcp_server.as_mut() {
            // Join before observing: an entered list refresh can finish during
            // close. Admission closure itself is not user cancellation, and
            // it does not settle detached tool custody.
            let catalog_result = server.close_and_observe_catalog();
            if !attempt.is_cancelled()
                && attempt.effect_custody().unsettled_reason().is_none()
                && !matches!(
                    &result,
                    Err(ProviderSessionError::ToolOutcomeUnknown(_)
                        | ProviderSessionError::Authentication(_)
                        | ProviderSessionError::Configuration(_))
                )
            {
                catalog_result?;
            }
        }
        result
    }
}

pub struct RoutedProviderRun<'a> {
    pub provider: RoutedProvider,
    pub fabushi_model: Option<String>,
    pub codex_model: Option<String>,
    pub fabushi_request_binding: Option<FabushiResponsesRequestBinding>,
    pub data_dir: &'a Path,
    pub messages: &'a [ProviderMessage],
    pub typed_replay: Option<&'a [TypedReplayEntry]>,
    pub bridge: Arc<dyn RoutedToolBridge>,
    pub cursor_auth: Option<Arc<dyn CursorInferenceAuth>>,
    pub request_context: RunnerRequestContextSnapshot,
    pub cancellation: RoutedProviderCancellation,
    pub checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    pub retry_sink: Option<Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>>,
    pub retry_report_sink: Option<Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>>,
    pub stream_attempt_runtime: Option<Arc<StreamAttemptRuntime>>,
    pub usage_sink: Option<Arc<dyn Fn(ProviderTokenUsage) + Send + Sync>>,
    pub cloud_agents_enabled: bool,
    pub multitask_enabled: bool,
    pub is_computer_use_subagent: bool,
    pub max_steps: usize,
    pub tool_step_reminder: Option<Arc<ProviderToolStepReminderHook>>,
}

pub fn run_routed_provider_in_runner(
    run: RoutedProviderRun<'_>,
    on_text_delta: &mut dyn FnMut(&str, &str),
) -> Result<String, ProviderSessionError> {
    run.cancellation.check()?;

    // A direct invocation may have no Host-admitted model. Resolve its
    // compatibility fallback once before tools and share it across attempts.
    let codex_model = if run.provider == RoutedProvider::Codex {
        Some(resolve_codex_request_model(run.codex_model.as_deref(), None)?)
    } else {
        None
    };

    let system_prompt = render_request_context_system_prompt_with_capabilities(
        &run.request_context.context,
        run.request_context.rules.as_deref(),
        run.cloud_agents_enabled,
        run.multitask_enabled,
    );
    if run.typed_replay.is_some()
        && !matches!(
            run.provider,
            RoutedProvider::Cursor
                | RoutedProvider::Codex
                | RoutedProvider::Fabushi
                | RoutedProvider::OpenRouter
                | RoutedProvider::ClaudeCode
        )
    {
        return Err(ProviderSessionError::Protocol(
            "typed compacted replay reached the provider runtime before provider-specific message merge was enabled".into(),
        ));
    }
    let mut provider_messages = Vec::with_capacity(run.messages.len() + usize::from(!system_prompt.is_empty()));
    if !system_prompt.is_empty() {
        provider_messages.push(ProviderMessage {
            role: "system".into(),
            content: system_prompt,
        });
    }
    provider_messages.extend_from_slice(run.messages);

    let direct_tools = if run.provider == RoutedProvider::ClaudeCode {
        Vec::new()
    } else {
        run.cancellation.check()?;
        run.bridge.list_tools()?
    };
    let delta_cancellation = run.cancellation.clone();
    let mut guarded_delta = |delta: &str, accumulated: &str| {
        if !delta_cancellation.is_cancelled() {
            on_text_delta(delta, accumulated);
        }
    };

    let mut executor = ProductionRoutedProviderAttemptExecutor {
        provider: run.provider,
        fabushi_model: run.fabushi_model.clone(),
        codex_model,
        fabushi_request_binding: if run.provider == RoutedProvider::Fabushi {
            Some(run.fabushi_request_binding.clone().unwrap_or_else(
                FabushiResponsesRequestBinding::for_runner_invocation,
            ))
        } else {
            None
        },
        data_dir: run.data_dir,
        messages: &provider_messages,
        typed_replay: run.typed_replay,
        tools: &direct_tools,
        cursor_auth: run.cursor_auth.clone(),
        bridge: Arc::clone(&run.bridge),
        is_computer_use_subagent: run.is_computer_use_subagent,
        usage_sink: run.usage_sink.clone(),
        max_steps: run.max_steps.max(1),
        tool_step_reminder: run.tool_step_reminder.clone(),
    };
    let retry_sink = run.retry_sink.clone();
    let mut on_retry = move |event: &ProviderRetryEvent| {
        if let Some(sink) = retry_sink.as_ref() {
            sink(event);
        }
    };
    let retry_report_sink = run.retry_report_sink.clone();
    let mut on_retry_report = move |report: &ProviderRetryReport| {
        if let Some(sink) = retry_report_sink.as_ref() {
            sink(report);
        }
    };
    let adapter = ProductionTurnRunShellAdapter::default();
    let result = match run.stream_attempt_runtime.clone() {
        Some(runtime) => adapter.run_with_attempt_runtime_reporting(
            runtime,
            &run.cancellation,
            run.checkpoint_store.as_ref(),
            &mut executor,
            &mut guarded_delta,
            &mut on_retry,
            &mut on_retry_report,
        ),
        None => adapter.run_with_retry_reporting(
            &run.cancellation,
            run.checkpoint_store.as_ref(),
            &mut executor,
            &mut guarded_delta,
            &mut on_retry,
            &mut on_retry_report,
        ),
    };

    run.cancellation.check()?;
    result
}

pub struct RoutedMcpServer {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    worker_failed: bool,
    catalog_refusal: Arc<Mutex<Option<McpCatalogRefusal>>>,
    attempt: RoutedProviderAttemptControl,
}

impl RoutedMcpServer {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn close(&mut self) {
        self.attempt.close_admission();
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = TcpStream::connect(
                self.url
                    .strip_prefix("http://")
                    .and_then(|value| value.split('/').next())
                    .unwrap_or("127.0.0.1:0"),
            );
            self.worker_failed |= worker.join().is_err();
        }
    }

    /// Retire the endpoint and observe its final catalog state. The shared
    /// refusal survives worker failure; a failed join is never a clean catalog.
    pub fn close_and_observe_catalog(&mut self) -> Result<(), ProviderSessionError> {
        self.close();
        let refusal = *self.catalog_refusal.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(refusal) = refusal {
            Err(ProviderSessionError::McpCatalog(refusal))
        } else if self.worker_failed {
            Err(ProviderSessionError::Protocol(
                "routed MCP catalog observation failed: worker did not complete".into(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for RoutedMcpServer {
    fn drop(&mut self) {
        self.close();
    }
}

pub fn start_routed_mcp_server(
    bridge: Arc<dyn RoutedToolBridge>,
) -> Result<RoutedMcpServer, ProviderSessionError> {
    start_routed_mcp_server_with_cancellation(
        bridge,
        RoutedProviderCancellation::default(),
    )
}

pub fn start_routed_mcp_server_with_cancellation(
    bridge: Arc<dyn RoutedToolBridge>,
    cancellation: RoutedProviderCancellation,
) -> Result<RoutedMcpServer, ProviderSessionError> {
    let runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy::default()));
    let generation = runtime.begin_attempt();
    start_routed_mcp_server_for_attempt(
        bridge, RoutedProviderAttemptControl::new(runtime, generation, cancellation), false,
    )
}

pub fn start_routed_mcp_server_for_attempt(
    bridge: Arc<dyn RoutedToolBridge>,
    attempt: RoutedProviderAttemptControl,
    is_computer_use_subagent: bool,
) -> Result<RoutedMcpServer, ProviderSessionError> {
    attempt.check()?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| {
        ProviderSessionError::Transport(format!("could not bind routed MCP server: {error}"))
    })?;
    listener.set_nonblocking(true).map_err(|error| {
        ProviderSessionError::Transport(format!("could not configure routed MCP server: {error}"))
    })?;
    let address = listener
        .local_addr()
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let secret = Uuid::new_v4().to_string();
    let path = format!("/mcp/{secret}");
    let url = format!("http://{}{}", address, path);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let catalog_refusal = Arc::new(Mutex::new(None));
    let worker_catalog_refusal = Arc::clone(&catalog_refusal);
    let worker_attempt = attempt.clone();
    let worker = thread::Builder::new()
        .name("mahayana-runner-routed-mcp".into())
        .spawn(move || {
            let mut tools = HashMap::<String, RoutedToolDefinition>::new();
            while !worker_stop.load(Ordering::Acquire) && !worker_attempt.is_cancelled() {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // A nonblocking listener can yield accepted sockets that are
                        // still nonblocking on some platforms. The routed MCP HTTP
                        // parser is deliberately synchronous, so normalize each
                        // accepted stream before the first header read instead of
                        // treating an early WouldBlock as a closed request.
                        if stream.set_nonblocking(false).is_err() {
                            continue;
                        }
                        let _ = serve_request(
                            stream,
                            &path,
                            &mut tools,
                            &worker_catalog_refusal,
                            &bridge,
                            &worker_attempt,
                            is_computer_use_subagent,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
        })
        .map_err(|error| ProviderSessionError::Transport(format!(
            "could not start routed MCP server: {error}"
        )))?;

    Ok(RoutedMcpServer {
        url,
        stop,
        worker: Some(worker),
        worker_failed: false,
        catalog_refusal,
        attempt,
    })
}

fn serve_request(
    mut stream: TcpStream,
    expected_path: &str,
    tools: &mut HashMap<String, RoutedToolDefinition>,
    catalog_refusal: &Mutex<Option<McpCatalogRefusal>>,
    bridge: &Arc<dyn RoutedToolBridge>,
    attempt: &RoutedProviderAttemptControl,
    is_computer_use_subagent: bool,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut request = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(());
        }
        request.extend_from_slice(&chunk[..count]);
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        if request.len() > 64 * 1024 {
            return write_response(&mut stream, 431, None);
        }
    };
    let headers = std::str::from_utf8(&request[..header_end])
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "headers are not UTF-8"))?;
    let mut lines = headers.lines();
    let mut request_line = lines.next().unwrap_or("").split_whitespace();
    let method = request_line.next().unwrap_or("");
    let path = request_line.next().unwrap_or("");
    if method != "POST" || path != expected_path {
        return write_response(&mut stream, 404, None);
    }
    let content_length = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    if content_length > ROUTED_MCP_MAX_BODY_BYTES {
        return write_response(&mut stream, 413, None);
    }
    let expected = header_end.saturating_add(content_length);
    while request.len() < expected {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..count]);
    }
    if request.len() < expected {
        return write_response(&mut stream, 400, None);
    }
    let message = match serde_json::from_slice::<Value>(&request[header_end..expected]) {
        Ok(value) if value.is_object() => value,
        _ => return write_response(&mut stream, 400, None),
    };

    let rpc_method = message.get("method").and_then(Value::as_str).unwrap_or("");
    if let Err(error) = attempt.check() {
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        return write_json(
            &mut stream,
            json!({
                "jsonrpc":"2.0",
                "id":id,
                "result":tool_failure(
                    error.to_string()
                )
            }),
        );
    }
    if rpc_method == "notifications/initialized" {
        return write_response(&mut stream, 202, None);
    }
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let mut tool_result_tracker = None;
    let result = match rpc_method {
        "initialize" => json!({
            "protocolVersion": ROUTED_MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "fabushi-runner-plugins", "version": "1" }
        }),
        "tools/list" => match bridge.list_tools() {
            Ok(discovered) => {
                // The sequential worker publishes the whole accepted refresh.
                // Keep existing public-name aliases and last-wins selection.
                *tools = discovered.into_iter().map(|tool| (tool.name.clone(), tool)).collect();
                *catalog_refusal.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
                json!({
                    "tools": tools.values().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description.clone().unwrap_or_else(|| format!(
                            "{} via {}", tool.tool_name, tool.provider_identifier
                        )),
                        "inputSchema": tool.input_schema
                    })).collect::<Vec<_>>()
                })
            }
            Err(ProviderSessionError::McpCatalog(refusal)) => {
                // Received corruption invalidates old descriptors. Availability
                // failures below retain the current map and unresolved fault.
                tools.clear();
                *catalog_refusal.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(refusal);
                tool_failure(refusal.to_string())
            }
            Err(error) => tool_failure(error.to_string()),
        },
        "tools/call" => {
            let params = message.get("params").and_then(Value::as_object);
            let name = params.and_then(|value| value.get("name")).and_then(Value::as_str).unwrap_or("");
            let args = params.and_then(|value| value.get("arguments")).cloned().unwrap_or_else(|| json!({}));
            match tools.get(name) {
                Some(tool) => {
                    let operation_bridge = Arc::clone(bridge);
                    let owned_tool = tool.clone();
                    let guard_tool = owned_tool.clone();
                    let guard_args = args.clone();
                    let tool_call_id = Uuid::new_v4().to_string();
                    let (outcome, tracker) = execute_routed_tool_with_delivery_tracker(
                        &guard_tool,
                        &guard_args,
                        is_computer_use_subagent,
                        attempt,
                        move || operation_bridge.call_tool(
                            &owned_tool,
                            args,
                            &tool_call_id,
                        ),
                    );
                    tool_result_tracker = tracker;
                    match outcome {
                        Ok(value) => normalize_tool_result(value),
                        Err(error) => tool_failure(error.to_string()),
                    }
                },
                None => tool_failure(format!("Unknown Fabushi plugin tool: {name}")),
            }
        }
        _ => json!({}),
    };
    let delivery = write_json(&mut stream, json!({"jsonrpc":"2.0","id":id,"result":result}));
    if delivery.is_err() {
        if let Some(tracker) = tool_result_tracker {
            tracker.mark_unknown(ToolEffectUnknownResult::DiscardedResult);
        }
    }
    delivery
}

fn normalize_tool_result(value: Value) -> Value {
    if let Some(root) = value.as_object() {
        if root.contains_key("content") || root.contains_key("structuredContent") {
            let mut normalized = root.clone();
            normalized.entry("isError").or_insert(Value::Bool(false));
            return Value::Object(normalized);
        }
    }
    json!({
        "isError": false,
        "content": [{"type":"text","text":value.to_string()}]
    })
}

fn tool_failure(message: impl Into<String>) -> Value {
    json!({
        "isError": true,
        "content": [{"type":"text","text":message.into()}]
    })
}

fn write_json(stream: &mut TcpStream, value: Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(&value)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    write_response(stream, 200, Some(body))
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    body: Option<Vec<u8>>,
) -> std::io::Result<()> {
    let body = body.unwrap_or_default();
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Response",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )?;
    if !body.is_empty() {
        write!(stream, "Content-Type: application/json\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(&body)?;
    stream.flush()
}

#[cfg(test)]
mod checkpoint_publication_tests {
    use super::*;
    use crate::extensions::inference::codex_direct_responses::{
        CodexDirectCheckpoint, CodexDirectUsage,
    };

    struct PrivateRoot(PathBuf);

    impl PrivateRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "fabushi-checkpoint-publication-{}", Uuid::new_v4().simple()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path).expect("create owned checkpoint fixture");
            Self(path)
        }
    }

    impl Drop for PrivateRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn checkpoint() -> RoutedProviderCheckpoint {
        RoutedProviderCheckpoint::Fabushi(CodexDirectCheckpoint {
            model: Some("Provider/Case-Sensitive@Model".into()),
            input: vec![
                json!({"role": "user", "content": "private prompt λ"}),
                json!({"type": "function_call_output", "call_id": "call-1", "output": "private tool result"}),
            ],
            text: "Checking ".into(),
            response_id: "response-1".into(),
            usage: CodexDirectUsage {
                input_tokens: 17,
                output_tokens: 9,
                cache_read_tokens: 4,
                cache_write_tokens: 2,
            },
            completed_steps: 1,
            tool_calls_completed: 1,
        })
    }

    fn assert_checkpoint(path: &Path, expected: &RoutedProviderCheckpoint) {
        let bytes = fs::read(path).expect("read complete checkpoint payload");
        assert_eq!(bytes, serde_json::to_vec(expected).expect("encode expected checkpoint"));
        let decoded: RoutedProviderCheckpoint =
            serde_json::from_slice(&bytes).expect("decode published checkpoint");
        assert_eq!(&decoded, expected);
        let document: Value = serde_json::from_slice(&bytes).expect("read checkpoint wire shape");
        assert_eq!(document["provider"], "fabushi");
        assert_eq!(document["checkpoint"]["model"], "Provider/Case-Sensitive@Model");
        assert_eq!(document["checkpoint"]["input"][1]["output"], "private tool result");
    }

    fn entries(directory: &Path) -> Vec<PathBuf> {
        let mut paths = fs::read_dir(directory).expect("list checkpoint entries")
            .map(|entry| entry.expect("read checkpoint entry").path())
            .collect::<Vec<_>>();
        paths.sort();
        paths
    }

    #[test]
    fn production_checkpoint_round_trip_preserves_payload_and_hashed_isolation() {
        let root = PrivateRoot::new();
        let stores = [
            ProductionRoutedProviderCheckpointStore::new(&root.0, "agent/../private-a", "stream/../private-a"),
            ProductionRoutedProviderCheckpointStore::new(&root.0, "agent/../private-b", "stream/../private-a"),
            ProductionRoutedProviderCheckpointStore::new(&root.0, "agent/../private-a", "stream/../private-b"),
        ];
        assert_eq!(stores[0].directory().parent(), stores[2].directory().parent());
        assert_ne!(stores[0].directory().parent(), stores[1].directory().parent());
        assert_ne!(stores[0].directory(), stores[2].directory());
        let checkpoint = checkpoint();
        let mut published = Vec::new();
        for store in &stores {
            let relative = store.directory().strip_prefix(&root.0).expect("agent cannot escape data root");
            let names = relative.iter().map(|name| name.to_str().expect("ASCII path")).collect::<Vec<_>>();
            assert_eq!(names.len(), 3);
            assert_eq!(names[0], "runner-provider-checkpoints");
            for name in &names[1..] {
                assert_eq!(name.len(), 64);
                assert!(name.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
            }
            let path = PathBuf::from(store.persist(&checkpoint).expect("publish actual checkpoint"));
            assert_eq!(path.parent(), Some(store.directory()));
            assert_eq!(path.extension().and_then(|extension| extension.to_str()), Some("json"));
            assert!(Uuid::parse_str(path.file_stem().and_then(|name| name.to_str()).expect("UUID name")).is_ok());
            assert_checkpoint(&path, &checkpoint);
            assert_eq!(entries(store.directory()), vec![path.clone()]);
            published.push(path);
        }
        let reopened = ProductionRoutedProviderCheckpointStore::new(
            &root.0, "agent/../private-a", "stream/../private-a",
        );
        assert_eq!(reopened.directory(), stores[0].directory());
        let later = PathBuf::from(reopened.persist(&checkpoint).expect("publish distinct checkpoint"));
        assert_ne!(later, published[0]);
        assert_checkpoint(&published[0], &checkpoint);
        assert_checkpoint(&later, &checkpoint);
        let mut expected = vec![published[0].clone(), later];
        expected.sort();
        assert_eq!(entries(reopened.directory()), expected);
    }

    #[cfg(unix)]
    #[test]
    fn private_checkpoint_staging_and_published_file_have_owner_only_permissions() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let root = PrivateRoot::new();
        let store = ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "stream");
        let checkpoint = checkpoint();
        let id = Uuid::from_u128(1);
        let temporary = store.directory().join(format!("{id}.tmp"));
        let final_path = store.directory().join(format!("{id}.json"));
        let mut observed_publication = false;
        let cursor = store.persist_with_publication(&checkpoint, id, |directory| {
            if directory == store.directory() {
                assert!(!observed_publication);
                observed_publication = true;
                let staged = fs::metadata(&temporary).expect("owned staging file before cleanup");
                let published = fs::metadata(&final_path).expect("visible complete checkpoint");
                assert_eq!(staged.permissions().mode() & 0o777, 0o600);
                assert_eq!(published.permissions().mode() & 0o777, 0o600);
                assert_eq!((staged.dev(), staged.ino()), (published.dev(), published.ino()));
                assert_eq!(published.nlink(), 2);
                assert_checkpoint(&temporary, &checkpoint);
                assert_checkpoint(&final_path, &checkpoint);
            }
            sync_checkpoint_directory(directory)
        }).expect("publish private checkpoint");
        assert!(observed_publication, "final directory barrier must run before success");
        assert_eq!(PathBuf::from(cursor), final_path);
        assert!(!temporary.exists());
        assert_eq!(fs::metadata(&final_path).expect("retained final file").nlink(), 1);
        assert_eq!(entries(store.directory()), vec![final_path]);
    }

    #[test]
    fn occupied_checkpoint_temporary_preserves_original_and_unrelated_entries() {
        let root = PrivateRoot::new();
        let store = ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "stream");
        fs::create_dir_all(store.directory()).expect("prepare occupied fixture directory");
        let id = Uuid::from_u128(2);
        let temporary = store.directory().join(format!("{id}.tmp"));
        let final_path = store.directory().join(format!("{id}.json"));
        let unrelated = store.directory().join("other-operation.tmp");
        let unrelated_directory = store.directory().join("other-directory");
        fs::write(&temporary, b"original staging owner").expect("seed occupied staging file");
        fs::write(&unrelated, b"unrelated staging owner").expect("seed unrelated staging file");
        fs::create_dir(&unrelated_directory).expect("seed unrelated directory");
        let marker = unrelated_directory.join("marker");
        fs::write(&marker, b"unrelated child").expect("seed unrelated child");
        let before = entries(store.directory());

        let result = store.persist_with_publication(&checkpoint(), id, sync_checkpoint_directory);
        assert!(matches!(result, Err(ProviderSessionError::Transport(message))
            if message.starts_with("could not persist Runner provider checkpoint:")));
        assert_eq!(fs::read(&temporary).expect("original staging entry retained"), b"original staging owner");
        assert_eq!(fs::read(&unrelated).expect("unrelated staging entry retained"), b"unrelated staging owner");
        assert_eq!(fs::read(&marker).expect("unrelated child retained"), b"unrelated child");
        assert!(!final_path.exists(), "temporary collision cannot publish a checkpoint");
        assert_eq!(entries(store.directory()), before);
    }

    #[test]
    fn occupied_checkpoint_final_preserves_original_and_cleans_only_owned_temporary() {
        let root = PrivateRoot::new();
        let store = ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "stream");
        fs::create_dir_all(store.directory()).expect("prepare occupied fixture directory");
        let id = Uuid::from_u128(3);
        let temporary = store.directory().join(format!("{id}.tmp"));
        let final_path = store.directory().join(format!("{id}.json"));
        let unrelated = store.directory().join("other-operation.tmp");
        fs::write(&final_path, b"original final owner, even if not valid JSON").expect("seed occupied final file");
        fs::write(&unrelated, b"other operation").expect("seed unrelated entry");
        let mut expected = entries(store.directory());
        let checkpoint = checkpoint();

        let result = store.persist_with_publication(&checkpoint, id, sync_checkpoint_directory);
        assert!(matches!(result, Err(ProviderSessionError::Transport(message))
            if message.starts_with("could not commit Runner provider checkpoint:")));
        assert_eq!(fs::read(&final_path).expect("original final entry retained"), b"original final owner, even if not valid JSON");
        assert_eq!(fs::read(&unrelated).expect("other operation retained"), b"other operation");
        assert!(!temporary.exists(), "failed no-clobber publication must clean its own staging file");
        assert_eq!(entries(store.directory()), expected);

        let later = PathBuf::from(store.persist(&checkpoint).expect("ordinary write after collision refusal"));
        assert_ne!(later, final_path);
        assert_checkpoint(&later, &checkpoint);
        assert_eq!(fs::read(&final_path).expect("collision still preserved"), b"original final owner, even if not valid JSON");
        expected.push(later);
        expected.sort();
        assert_eq!(entries(store.directory()), expected);
    }

    #[test]
    fn checkpoint_nested_directories_publish_in_order_and_preserve_existing_modes() {
        let root = PrivateRoot::new();
        let existing = root.0.join("existing");
        fs::create_dir(&existing).expect("seed existing ancestor");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&existing, fs::Permissions::from_mode(0o750)).expect("set existing ancestor mode");
        }
        let first_new = existing.join("new");
        let data_dir = first_new.join("nested");
        let store = ProductionRoutedProviderCheckpointStore::new(&data_dir, "agent", "stream");
        let agent_directory = store.directory().parent().expect("agent directory").to_path_buf();
        let checkpoint_root = agent_directory.parent().expect("checkpoint root").to_path_buf();
        let id = Uuid::from_u128(4);
        let final_path = store.directory().join(format!("{id}.json"));
        let mut barriers = Vec::new();
        let checkpoint = checkpoint();
        let cursor = store.persist_with_publication(&checkpoint, id, |directory| {
            barriers.push(directory.to_path_buf());
            assert_eq!(final_path.exists(), directory == store.directory());
            sync_checkpoint_directory(directory)
        }).expect("publish complete nested directory chain");
        assert_eq!(PathBuf::from(cursor), final_path);
        assert_eq!(barriers, vec![
            root.0.clone(), existing.clone(), first_new.clone(), data_dir.clone(),
            checkpoint_root.clone(), agent_directory.clone(), store.directory().to_path_buf(),
        ]);
        assert_checkpoint(&final_path, &checkpoint);
        assert_eq!(entries(store.directory()), vec![final_path]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&existing).expect("existing ancestor").permissions().mode() & 0o777, 0o750);
            for directory in [
                first_new.as_path(), data_dir.as_path(), checkpoint_root.as_path(),
                agent_directory.as_path(), store.directory(),
            ] {
                assert_eq!(fs::metadata(directory).expect("new private directory").permissions().mode() & 0o777, 0o700);
            }
            fs::set_permissions(store.directory(), fs::Permissions::from_mode(0o750)).expect("set existing stream mode");
            store.persist(&checkpoint).expect("write through existing checkpoint directory");
            assert_eq!(fs::metadata(store.directory()).expect("existing stream directory").permissions().mode() & 0o777, 0o750);
        }
    }

    #[cfg(unix)]
    #[test]
    fn checkpoint_directory_barrier_failure_is_retried_before_descendants() {
        let root = PrivateRoot::new();
        let retained = root.0.join("new-data");
        let data_dir = retained.join("nested-data");
        let store = ProductionRoutedProviderCheckpointStore::new(&data_dir, "agent", "stream");
        let unrelated = root.0.join("unrelated");
        fs::write(&unrelated, b"preserved ancestor entry").expect("seed unrelated entry");
        let checkpoint = checkpoint();
        for attempt in 0..2 {
            let mut barriers = Vec::new();
            let result = store.persist_with_publication(&checkpoint, Uuid::from_u128(5), |directory| {
                barriers.push(directory.to_path_buf());
                if directory == root.0.as_path() {
                    assert!(retained.is_dir(), "new entry precedes its parent barrier");
                    assert!(!data_dir.exists(), "failed barrier must stop descendant creation");
                    return Err(io::Error::other("injected parent directory barrier failure"));
                }
                sync_checkpoint_directory(directory)
            });
            assert!(matches!(result, Err(ProviderSessionError::Transport(_))));
            let expected = if attempt == 0 {
                vec![root.0.parent().expect("fixture parent").to_path_buf(), root.0.clone()]
            } else {
                vec![root.0.clone()]
            };
            assert_eq!(barriers, expected, "retained directory must retry its failed entry barrier");
            assert!(retained.is_dir());
            assert!(!data_dir.exists());
            assert!(!store.directory().exists());
            assert_eq!(fs::read(&unrelated).expect("unrelated entry retained"), b"preserved ancestor entry");
        }

        let later = PathBuf::from(store.persist(&checkpoint).expect("ordinary publication after barrier repair"));
        assert_checkpoint(&later, &checkpoint);
        assert_eq!(entries(store.directory()), vec![later]);
        assert_eq!(fs::read(&unrelated).expect("unrelated entry still retained"), b"preserved ancestor entry");
    }

    #[cfg(unix)]
    #[test]
    fn checkpoint_final_barrier_failure_retains_visible_payload_without_success() {
        let root = PrivateRoot::new();
        let store = ProductionRoutedProviderCheckpointStore::new(&root.0, "agent", "stream");
        fs::create_dir_all(store.directory()).expect("prepare existing checkpoint directory");
        let checkpoint = checkpoint();
        let id = Uuid::from_u128(6);
        let temporary = store.directory().join(format!("{id}.tmp"));
        let final_path = store.directory().join(format!("{id}.json"));
        let unrelated = store.directory().join("other-operation.tmp");
        fs::write(&unrelated, b"other operation").expect("seed unrelated staging entry");
        let mut barriers = Vec::new();
        let result = store.persist_with_publication(&checkpoint, id, |directory| {
            barriers.push(directory.to_path_buf());
            if directory == store.directory() {
                assert_checkpoint(&temporary, &checkpoint);
                assert_checkpoint(&final_path, &checkpoint);
                return Err(io::Error::other("injected final directory barrier failure"));
            }
            assert!(!final_path.exists());
            sync_checkpoint_directory(directory)
        });
        assert!(matches!(result, Err(ProviderSessionError::Transport(_))));
        assert_eq!(barriers, vec![
            store.directory().parent().expect("agent directory").to_path_buf(),
            store.directory().to_path_buf(),
        ]);
        assert_checkpoint(&final_path, &checkpoint);
        assert!(!temporary.exists(), "uncertain publication cleans only its owned staging name");
        assert_eq!(fs::read(&unrelated).expect("other staging owner retained"), b"other operation");
        let mut expected = vec![final_path.clone(), unrelated.clone()];
        expected.sort();
        assert_eq!(entries(store.directory()), expected);

        let later = PathBuf::from(store.persist(&checkpoint).expect("ordinary write after uncertain publication"));
        assert_ne!(later, final_path);
        assert_checkpoint(&later, &checkpoint);
        assert_checkpoint(&final_path, &checkpoint);
        assert_eq!(fs::read(&unrelated).expect("other operation still retained"), b"other operation");
        expected.push(later);
        expected.sort();
        assert_eq!(entries(store.directory()), expected);
    }
}
