//! Grok-aligned Mahayana Host process entrypoint.
//!
//! The shipping desktop Host process is owned by `source/host/app`. During the
//! migration, `mahayana-unified-app-host` remains an internal compatibility
//! backend so existing product commands keep working while Grok Host/Runner
//! modules are moved behind this process boundary. Electron must never launch
//! the legacy third_party desktop Host binary directly.

use mahayana_host_runtime::extensions::action_audit::action_audit_service::{AuditAction, AuditRecord};
use mahayana_host_runtime::extensions::box_store_sync::box_copy_in::{
    BOX_COPY_IN_ARG, execute_production_box_copy_in_from_env,
};
use mahayana_host_runtime::extensions::box_store_sync::production::ProductionBoxStoreSyncApi;
use mahayana_host_runtime::extensions::action_audit::extension::ActionAuditExtension;
use mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentsService;
use mahayana_host_runtime::extensions::auto_review::extension::{
    HostAutoReviewExtension, start_auto_review_extension,
};
use mahayana_host_runtime::extensions::auto_review::sand_backend_smart_mode_classifier_exec::{
    create_sand_backend_smart_mode_classifier_executor_with_cancellation,
};
use mahayana_host_runtime::extensions::auth::extension::HostAuthExtension;
use mahayana_host_runtime::extensions::telemetry::auto_review_approval_telemetry::{
    AutoReviewApprovalReport, auto_review_approval_telemetry,
};
use mahayana_host_runtime::runner::auto_review_gate::{
    AutoReviewGate, AutoReviewGateDependencies, AutoReviewInstructions, ShellApprovalSurface,
};
use mahayana_host_runtime::runner::sand_auto_review::{
    SandAutoReviewApprovalStatus, SandAutoReviewController, SandAutoReviewDecision,
    SandAutoReviewEvent, SandAutoReviewExpiryCause, SandAutoReviewMode, SandAutoReviewModes,
    SandAutoReviewResolution, fingerprint_sand_auto_review_target,
    sand_auto_review_approval_expiry_policy,
};
use mahayana_host_runtime::runner::sand_auto_review_classifier_run::{
    AutoReviewClassifierDecision, run_sand_auto_review_classifier,
};
use mahayana_host_runtime::runner::sand_auto_review_tool_escalations::{
    McpApprovalRequest, ShellApprovalRequest, ShellApprovalTarget,
    request_sand_mcp_approval, request_sand_shell_approval,
};
use mahayana_host_runtime::runner::sand_automation_auto_review::{
    AutomationReviewOutcome, SAND_AUTOMATION_WRITE_CLASSIFIER_ERROR_REASON,
    review_sand_automation_write,
};
use mahayana_host_runtime::runner::sand_auto_review_summaries::CloudLifecycleAction;
use mahayana_host_runtime::runner::sand_browser_auto_review::run_sand_browser_auto_review_preflight;
use mahayana_host_runtime::runner::sand_computer_auto_review::{
    BoxIdentity, SAND_COMPUTER_AUTO_REVIEW_CLASSIFIER_ERROR_REASON,
    run_sand_computer_auto_review_preflight,
};
use mahayana_host_runtime::runner::sand_cloud_agent_auto_review::{
    CloudAgentReviewImage, CloudAgentReviewOutcome,
    SAND_CLOUD_AGENT_CLASSIFIER_ERROR_REASON,
    build_sand_cloud_agent_lifecycle_review_target, build_sand_cloud_agent_review_target,
    review_sand_cloud_agent_action, review_sand_cloud_agent_lifecycle_action,
};
use mahayana_host_runtime::runner::sand_subagent_auto_review::{
    SubagentReviewOutcome, SAND_SUBAGENT_CLASSIFIER_ERROR_REASON,
    build_sand_subagent_launch_review_target, build_sand_subagent_steer_review_target,
    review_sand_subagent_action,
};
use mahayana_host_runtime::extensions::cloud_agents::cloud_agents_service::SandCloudAgentManager;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_profile_files::AgentProfileUpdate;
use mahayana_host_runtime::agents::agent_messaging::AgentMessageImage;
use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::transcript::agent_to_agent_messaging::{
    AgentWakeRequest, ProductionAgentToAgentMessaging, should_interrupt_priority_peer,
};
use mahayana_host_runtime::extensions::memory::agent_state::SandAgentState;
use mahayana_host_runtime::extensions::telemetry::HostTelemetryProjection;
use mahayana_host_runtime::extensions::local_exec::extension::HostLocalExecExtension;
use mahayana_host_runtime::extensions::local_tool_permission::extension::{
    HostLocalToolPermissionExtension,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_resolution::{
    LocalToolPermissionResolutionArgs, SandLocalToolPermissionResolutionError,
};
use mahayana_host_runtime::extensions::transcript::widget_responses::WidgetResponses;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::extensions::session::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffDecision, HandoffRequest, HandoffStartResult,
    HandoffTelemetry, HandoffTrigger, PendingHandoff, ScreenshotPayload, decide_box_hand_back,
};
use mahayana_host_runtime::extensions::session::extension::start_session_extension;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::extensions::secrets::extension::{
    HostSecretsExtension, SecretsGatewayError, dispatch_secrets_gateway_call,
};
use mahayana_host_runtime::extensions::notifications::extension::{
    notification_agent_from_value, start_notifications_extension,
};
use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
    dispatch_production_session_gateway_call_with_content_search,
    persist_accepted_send_prompt_context,
};
use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::completion_revivals::{
    CompletionRevivalRuntimePort, CompletionRevivals, RevivalExecution, RevivalReport,
    ShellCompletion, SubagentCompletion,
};
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    DurablePendingWakeMarker, PendingWakeKind, QuietWakeOrigin, coerce_quiet_origin,
};
use mahayana_host_runtime::extensions::transcript::pending_wake_rearm::{
    LostSubagentWake, PendingWakeRearm, PendingWakeReport, PendingWakeRuntimePort,
};
use mahayana_host_runtime::extensions::transcript::group_chat_glue::GroupChatGlue;
use mahayana_host_runtime::extensions::transcript::group_chat_orchestrator::GroupMemberTurnRequest;
use mahayana_host_runtime::extensions::cross_user_sharing::production::{
    CrossUserGatewayError, ProductionCrossUserRuntime, RemoteRequestedTurnRunner,
    SharedRoomTurnRunner,
};
use mahayana_host_runtime::groups::group_chat::{GroupDescription, GroupMember};
use mahayana_host_runtime::extensions::transcript::send_group_fanout::{
    GroupMemberTurnExecutor, LocalGroupFanoutDisposition, collect_new_member_send_messages,
    dispatch_local_group_send,
};
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;
use mahayana_host_runtime::extensions::transcript::extension::start_transcript_extension;
use mahayana_host_runtime::extensions::transcript::send_message_shaping::shape_send_prompt_media_args;
use mahayana_host_runtime::extensions::transcript::box_handoff_resume::{
    build_box_handoff_resume_send_args, settle_box_handoff_state,
};
use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker;
use mahayana_host_runtime::extensions::transcript::upgrade_recreate_resume::build_upgrade_resume_prompt;
use mahayana_host_runtime::extensions::transcript::box_request_entries::resolve_box_request_entry;
use mahayana_host_runtime::extensions::transcript::workflow_commands::{
    WorkflowCommandError, WorkflowRunNowPlan, dispatch_workflow_command_with_runtime,
    prepare_workflow_run_now,
};
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationExecutionResult, FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::{
    AutomationCommandError, dispatch_automation_command,
};
use mahayana_host_runtime::automations::automation_status_reminder::create_automation_status_reminder;
use mahayana_host_runtime::automations::automation_trigger::{trigger_matches_event, trigger_members};
use mahayana_host_runtime::extensions::automations::listener_integrations::count_listener_platforms;
use mahayana_host_runtime::extensions::automations::fire_delivery::{
    PreparedBackendFire, prepare_backend_fire,
};
use mahayana_host_runtime::extensions::automations::production_lifecycle::ProductionAutomationsLifecycle;
use mahayana_host_runtime::extensions::automations::sand_automation_cloud_sync::{
    ScheduledCloudAutomation, is_server_schedulable, sand_cloud_definition,
};
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    BackendAutomationFire, FireCompletion,
};
use mahayana_host_runtime::extensions::transcript::ack_obligations::{
    AckObligations, AckRedrivePreparation, build_ack_redrive_empty_delivery_report,
    build_ack_redrive_send_args,
};
use mahayana_host_runtime::extensions::transcript::runner_registry::TranscriptRunnerRegistry;
use mahayana_host_runtime::extensions::transcript::run_scheduler::WatchdogStage;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::{
    AgentDeletionRuntimeDeps, AgentLifecycleGatewayError,
    dispatch_production_agent_lifecycle_gateway_call_with_runtime,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderOptions,
    RoutedToolDefinition, configured_routed_provider, run_routed_provider_text,
};
use mahayana_host_runtime::extensions::inference::inference_service::{
    InferenceUsage, authorize_routed_provider_request,
};
use mahayana_host_runtime::extensions::inference::production::ProductionInferenceExtension;
use mahayana_host_runtime::extensions::inference::cursor_session::{
    RequestLineage, SandSessionOptions,
};
use mahayana_host_runtime::extensions::inference::generated_inference_codec::InferenceReason;
use mahayana_host_runtime::extensions::webauthn_proxy::extension::HostWebAuthnProxyExtension;
use mahayana_host_runtime::extensions::telemetry::automation_fire_telemetry::{
    AutomationFireDroppedReport, automation_fire_dropped_telemetry,
};
use mahayana_host_runtime::extensions::telemetry::queue_telemetry_mappers::{
    QueueAcceptedReport, QueueDequeuedReport, QueueWatchdogReport,
    queue_accepted_telemetry, queue_dequeued_telemetry, queue_watchdog_telemetry,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostStructuredLogTelemetry;
use mahayana_host_runtime::extensions::telemetry::revival_telemetry_mappers::{
    SubagentRevivalReport, subagent_revival_telemetry,
};
use mahayana_host_runtime::extensions::experiments::HostExperimentsExtension;
use mahayana_host_runtime::extensions::content_search::extension::ProductionContentSearchExtension;
use mahayana_host_runtime::extensions::trays::extension::HostTraysExtension;
use mahayana_host_runtime::extensions::host_upgrade::production::{
    ProductionHostUpgradeExtension, ProductionHostUpgradePeers,
    start_production_host_upgrade_extension,
};
use mahayana_host_runtime::host_production_extensions::{
    start_production_host_box_extensions, start_production_host_extensions,
};
#[cfg(test)]
use mahayana_host_runtime::host_production_extensions::{
    ProductionBrowserUaLog, ProductionHostExtensions,
};
use mahayana_host_runtime::extensions::telemetry::turn_empty_delivery_telemetry::turn_empty_delivery_telemetry;
use mahayana_host_runtime::extensions::telemetry::turn_telemetry_mappers::{
    TokenUsage as TelemetryTokenUsage, TtftFields, TurnAwaitFields, TurnInterruptFields,
    TurnRetryFields, TurnUsageFields, UserMessageReceivedFields, ttft_telemetry,
    turn_await_telemetry, turn_interrupt_telemetry, turn_retry_telemetry,
    turn_usage_telemetry, user_message_received_telemetry,
};
use mahayana_host_runtime::extensions::telemetry::agent_error_telemetry::{
    AgentErrorReport, agent_error_detail_telemetry, agent_error_telemetry,
};
use mahayana_host_runtime::extensions::transcript::agent_run_error::provider_failure_tray;
use mahayana_host_runtime::extensions::transcript::turn_runtime::classify_agent_error;
use mahayana_host_runtime::ports::telemetry::sand_error_detail;
use mahayana_host_runtime::extensions::forever_box::{
    ForeverBoxRunnerResourcePort, BoxStatus, ForeverBoxService,
};
use mahayana_host_runtime::extensions::teach_recording::extension::{
    CAP_SLACK_MS, TeachRecordingServiceDeps, create_teach_recording_extension,
    load_teach_queue_key,
};
use mahayana_host_runtime::extensions::teach_recording::teach_recording_service::{
    LEARN_SKILL_NAME, ProductionTeachRecordingRuntime, SAND_TEACH_MAX_DURATION_MS,
    SandTeachRecordingServiceFactory, TeachRecordingApi, TeachRecordingRuntimePort, TeachStatus,
};
use mahayana_host_runtime::runner_context_production_provider::ProductionRunnerRequestContextSource;
use mahayana_host_runtime::sand_activity::ActivityUpdate;
use mahayana_host_runtime::runner::production_turn_agent_owner::ProductionTurnAgentOwner;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, ProductionAgentStateCheckpointSink,
};
use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
};
use mahayana_host_runtime::transcript_mirror::production_provider::{
    ProductionTranscriptMirrorProvider,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryOutcome, ProviderRetryReport,
};
use mahayana_host_runtime::runner::production_turn_input_projection::create_production_turn_input_projection;
use mahayana_host_runtime::runner::prompt_collector_glue::project_provider_messages_for_turn;
use mahayana_host_runtime::runner::sand_memory::{
    MEMORY_RECENT_PROMPT_LIMIT, is_memorable_exchange,
};
use mahayana_host_runtime::runner::turn_memory::{
    TurnMemoryMode, build_turn_memory_exchange, run_turn_memory_with,
};
use mahayana_host_runtime::runner::tools::sand_spotlight_tools::spotlight_prompt_section;
use mahayana_host_runtime::runner::tools::sand_state_tool::{
    RoutineAutoReviewCallback, RoutinePostWriteCallback, SandStateWriter,
};
use mahayana_host_runtime::runner::tools::listener_connect_cards::{
    surface_listener_connect_cards,
};
use mahayana_host_runtime::runner::tools::sand_multitask_todo_tool::MultitaskTodoState;
use mahayana_host_runtime::runner::tools::sand_browser_tools::{
    BrowserAutoReviewCallback, BrowserPersistImageCallback, BrowserPossibleNavigationCallback,
    BrowserToolExecutor, ProductionBrowserToolExecutor, capture_browser_review_state,
    to_browser_review_action,
};
use mahayana_host_runtime::runner::host_computer_tool_dependencies::ProductionComputerToolExecutor;
use mahayana_host_runtime::runner::host_file_transfer_dependencies::ProductionFileTransferExecutor;
use mahayana_host_runtime::runner::host_external_machine_dependencies::ProductionExternalMachineExecutor;
use mahayana_host_runtime::runner::tools::sand_file_transfer_tools::FileTransferExecutor;
use mahayana_host_runtime::runner::tools::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalMachineShellArgs, ExternalShellAutoReviewCallback,
};
use mahayana_host_runtime::runner::tools::sand_computer_tool::{
    ComputerAutoReviewCallback, ComputerPersistImageCallback, ComputerToolExecutor,
    to_exact_action_value, validate_computer_action,
};
use mahayana_host_runtime::runner::system_prompt_assembly::{append_automations_system_prompt, append_memory_system_prompt};
use mahayana_host_runtime::runner_production_bridge::{
    ProductionActionAuditInput, ProductionRunnerCompositionInput,
    create_production_runner_composition,
};
use mahayana_host_runtime::runner::sand_action_audit::{
    ActionAuditRecord, ActionAuditSink, normalize_navigation_url,
};
use mahayana_host_runtime::runner::turn_observation::{
    TurnObservation, TurnObservationHandle,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSource,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderTokenUsage, merge_provider_token_usage,
};
use mahayana_host_runtime::runner::box_tool_access::{
    BoxShellAutoReviewCallback, RunnerBoxResourcePort, RunnerBoxShellRequest,
};
use mahayana_host_runtime::cloud_agents::cloud_agent_tool::{CloudAgentReviewHook, CloudAgentToolDependencies};
use mahayana_host_runtime::runner::background_work::{
    CloudAgentWatchOptions, RunnerCloudAgentWatches,
};
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, ROUTED_TOOL_EXECUTE_METHOD, ROUTED_TOOL_LIST_METHOD,
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD,
};
use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
    BoxServerStatusLoader, CoordinatorMcpLifecycleRelay, CoordinatorMcpManagerBackend,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
};
use mahayana_host_runtime::extensions::mcp::mcp_service::McpHostService;
use mahayana_host_runtime::extensions::mcp::production_box_state::{
    MCP_STATE_EXEC_FIELD_NUMBER, MCP_TOOL_EXEC_FIELD_NUMBER, ProductionBoxMcpStateLoader,
    execute_box_mcp_raw,
};
use mahayana_host_runtime::runner::tools::mcp_host_service_management_sink::McpHostServiceManagementSink;
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::subagent_runtime::{
    RunOutcome as GeneratedSubagentRunOutcome, SubagentLineage as GeneratedSubagentLineage,
    SubagentRuntime,
};
use mahayana_host_runtime::attachment_paths::{
    AgentMediaKind, file_url_for_path, persist_agent_media_bytes,
};
use mahayana_host_runtime::runner::tools::send_message_encoding::{
    image_mime_from_path, resolve_box_media_attachment,
};
use mahayana_host_runtime::runner::tools::box_help_tool::{BoxHelpOutcome, BoxHelpRequest};
use mahayana_host_runtime::runner::tools::send_message_tool::{
    ResolvedAttachmentSource, SendMessageSink, file_path_from_file_url,
};
use mahayana_host_runtime::selected_image_inputs::read_image_file_dimensions;
use mahayana_host_runtime::runner::tools::sand_reaction_tool::ReactionSink;
use mahayana_host_runtime::runner::tools::sand_agent_management_tools::{
    AgentManagementRecord, AgentManagementSink,
};
use mahayana_host_runtime::runner::tools::sand_task_subagent_tool::{
    SubagentLaunchRecord, SubagentTaskReviewCallback, SubagentTaskSink,
};
use mahayana_host_runtime::runner::tools::sand_subagent_management_tools::{
    SteerReview, SubagentSteerReviewCallback,
};
use mahayana_host_runtime::gateway_config::{gateway_scheme, resolve_gateway_server_config};
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandContext, GatewayCommandError, GatewayCommandReport,
    GatewayEventHub, GatewayHealth, GatewayServerDeps, start_gateway_server,
};
use mahayana_host_runtime::host_gateway_api::{
    CreateAgentNonceLedger, sanitize_create_agent_args,
};
use mahayana_host_runtime::sand_host::{
    BOX_READY_REPORT_ATTEMPTS, BOX_READY_REPORT_RETRY_MS, BOX_READY_STAGE_MARKER_PATH,
    box_ready_duration_ms, compute_host_health, should_report_box_ready,
};
use mahayana_host_runtime::host_discovery::{
    GatewayDiscoveryInfo, clear_gateway_discovery, write_gateway_discovery,
};
use mahayana_host_runtime::host_lock::acquire_host_lock;
use mahayana_host_runtime::host_initial_transcript_load::{
    InitialTranscriptDegradedReason, ensure_initial_transcript_loaded,
    load_initial_transcript_resiliently,
};
use mahayana_host_runtime::host_paths::{get_gateway_discovery_path, get_host_lock_path};
use mahayana_host_runtime::r#box::box_env::BoxEnvironmentUpdate;
use mahayana_host_runtime::r#box::exec_daemon_process::start_managed_box_exec_daemon_from_process_env;
use mahayana_host_runtime::r#box::production::{
    BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, ProductionBoxEnvironment,
};
use mahayana_unified_app_host::{
    PlatformRequestHost, UnifiedAppHost, default_unified_app_data_dir, dispatch_json,
    is_platform_request_json,
};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex, Weak, mpsc,
    atomic::{AtomicBool, Ordering},
};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SHUTDOWN_WATCHDOG_MS: u64 = 5_000;

fn teach_recording_status_value(status: TeachStatus) -> serde_json::Value {
    serde_json::json!({
        "state": status.state,
        "agentId": status.agent_id,
        "startedAtMs": status.started_at_ms,
        "maxDurationMs": status.max_duration_ms,
    })
}


fn ensure_managed_runtime_layout(app_data_dir: &Path) -> io::Result<()> {
    // The desktop product owns this fallback workspace.  It must exist before
    // the native engine canonicalizes the path while opening the first Agent
    // session.  User-selected workspace paths are validated elsewhere and are
    // never created implicitly.
    fs::create_dir_all(app_data_dir.join("feature-host/runtime/workspace"))
}

fn install_shutdown_signal_worker(
    host_tx: mpsc::Sender<HostLaneRequest>,
    shutdown_complete: Arc<AtomicBool>,
) -> io::Result<thread::JoinHandle<()>> {
    let mut signals = Signals::new([SIGTERM, SIGINT])?;
    Ok(thread::spawn(move || {
        let Some(signal) = signals.forever().next() else {
            return;
        };
        let label = if signal == SIGTERM { "SIGTERM" } else { "SIGINT" };
        eprintln!("[sand-host] received {label}, shutting down");
        let _ = host_tx.send(HostLaneRequest::StdinClosed);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(SHUTDOWN_WATCHDOG_MS));
            if !shutdown_complete.load(Ordering::Acquire) {
                eprintln!("[sand-host] shutdown watchdog expired");
                std::process::exit(1);
            }
        });
    }))
}


enum HostLaneRequest {
    Stdin(String),
    Gateway {
        method: String,
        args: serde_json::Value,
        reply: mpsc::SyncSender<Result<serde_json::Value, GatewayCommandError>>,
    },
    StdinClosed,
}

const RUNNER_ACCEPT_ROUTED_PROMPT_GATEWAY_METHOD: &str = "runner.acceptRoutedPrompt";
const RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD: &str = "runner.startRoutedProvider";
const RUNNER_CANCEL_ROUTED_PROVIDER_GATEWAY_METHOD: &str = "runner.cancelRoutedProvider";
const RUNNER_INFERENCE_EVENT_CHANNEL: &str = "runner-inference";

fn persist_runner_media_bytes(
    sessions: &ProductionSessionWorkers,
    agent_id: &str,
    source_name: &str,
    bytes: &[u8],
    kind: AgentMediaKind,
) -> Option<String> {
    let db_path = sessions.session_db_path(agent_id).ok()?;
    let agent_dir = db_path.parent()?;
    let path = persist_agent_media_bytes(agent_dir, source_name, bytes, kind).ok()?;
    file_url_for_path(path)
}

fn resolve_production_attachment_source(
    sessions: &ProductionSessionWorkers,
    forever_box: &ForeverBoxService,
    agent_id: &str,
    source_url: &str,
) -> ResolvedAttachmentSource {
    let Some(source_path) = file_path_from_file_url(source_url) else {
        return ResolvedAttachmentSource {
            url: source_url.to_string(),
            file_name: None,
        };
    };
    let file_name = source_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    let source_path_text = source_path.to_string_lossy().into_owned();
    if let Ok(bytes) = fs::read(&source_path) {
        let kind = if image_mime_from_path(&source_path_text).is_some() {
            AgentMediaKind::Image
        } else {
            AgentMediaKind::Attachment
        };
        if let Some(url) = persist_runner_media_bytes(
            sessions,
            agent_id,
            &source_path_text,
            &bytes,
            kind,
        ) {
            return ResolvedAttachmentSource { url, file_name };
        }
    }
    let status = forever_box.get_status(agent_id);
    let remote_box_has_desktop = status.vnc_url.is_some()
        || status.windows.as_ref().is_some_and(|windows| !windows.is_empty());
    let resolved = resolve_box_media_attachment(
        &source_path_text,
        remote_box_has_desktop,
        |path| forever_box.box_().download_file(agent_id, path).ok(),
        |bytes, _mime| persist_runner_media_bytes(
            sessions,
            agent_id,
            &source_path_text,
            bytes,
            AgentMediaKind::Image,
        ),
        |name, bytes| persist_runner_media_bytes(
            sessions,
            agent_id,
            name,
            bytes,
            AgentMediaKind::Attachment,
        ),
    )
    .unwrap_or_else(|| source_url.to_string());
    ResolvedAttachmentSource { url: resolved, file_name }
}

struct ProductionSendMessageSink {
    sessions: Arc<ProductionSessionWorkers>,
    forever_box: Arc<ForeverBoxService>,
    session_handoff: BoxHandoffService,
    ack_obligations: Arc<AckObligations>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    ack_token: Option<String>,
    agent_id: String,
}

impl ProductionSendMessageSink {

    fn fulfill_ack_obligation(&self) {
        if let Some(ack_token) = self.ack_token.as_deref() {
            if let Err(error) = self
                .ack_obligations
                .fulfill_ack_obligation(&self.agent_id, ack_token)
            {
                eprintln!(
                    "mahayana-host-ack fulfill_failed agent={} error={error}",
                    self.agent_id
                );
            }
        }
    }
}

impl SendMessageSink for ProductionSendMessageSink {
    fn is_awaiting_user_selection(&self) -> bool {
        self.sessions
            .get_agent_awaiting_user_response(&self.agent_id)
            .ok()
            .flatten()
            .is_some()
    }

    fn request_box_help(
        &self,
        request: BoxHelpRequest,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<BoxHelpOutcome, ProviderSessionError> {
        let outcome = self.session_handoff.start(HandoffRequest {
            agent_id: self.agent_id.clone(),
            instruction: request.instruction.clone(),
            telemetry: HandoffTelemetry {
                reason: request.reason,
                domain: request.domain,
                idp_domain: request.idp_domain,
            },
        });
        match outcome {
            HandoffStartResult::AlreadyPending { request_id, instruction } => {
                Ok(BoxHelpOutcome::AlreadyPending { request_id, instruction })
            }
            HandoffStartResult::Started { request_id } => {
                let entry = serde_json::json!({
                    "id": format!("runner-box-help:{tool_call_id}"),
                    "kind": "send-message",
                    "message": {
                        "type": "text",
                        "content": request.instruction.clone(),
                    },
                    "timestampMs": timestamp_ms,
                    "boxRequestId": request_id.clone(),
                    "boxInstruction": request.instruction,
                });
                let tracking = self
                    .transcript_runtime
                    .track_box_request_entry(&self.agent_id, &entry);
                if let Some(superseded_request_id) = tracking.superseded_request_id.as_deref() {
                    if let Err(error) = resolve_box_request_entry(
                        &self.sessions,
                        &self.agent_id,
                        superseded_request_id,
                        "dismissed",
                    ) {
                        eprintln!(
                            "mahayana-host box_request_supersede_failed agent={} request={} error={error}",
                            self.agent_id,
                            superseded_request_id,
                        );
                    }
                }
                if let Err(error) = self.sessions.append_agent_transcript_entries(
                    &self.agent_id,
                    &[entry],
                ) {
                    let _ = self
                        .transcript_runtime
                        .resolve_box_request_tracking(&request_id);
                    self.session_handoff.forget(&self.agent_id);
                    return Err(ProviderSessionError::Tool(format!(
                        "could not persist request_box_help for {}: {error}",
                        self.agent_id
                    )));
                }
                self.fulfill_ack_obligation();
                self.transcript_runtime.track_runner_activity_update(
                    &self.agent_id,
                    &ActivityUpdate::SendMessage,
                    started_at_ms(),
                );
                Ok(BoxHelpOutcome::Started { request_id })
            }
        }
    }

    fn resolve_attachment_source(
        &self,
        source_url: &str,
        _tool_call_id: &str,
    ) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        Ok(resolve_production_attachment_source(
            self.sessions.as_ref(),
            self.forever_box.as_ref(),
            &self.agent_id,
            source_url,
        ))
    }

    fn read_media_dimensions(&self, resolved_url: &str) -> Option<(u32, u32)> {
        let path = file_path_from_file_url(resolved_url)?;
        let path = path.to_str()?;
        let dimensions = read_image_file_dimensions(path)?;
        Some((dimensions.width, dimensions.height))
    }

    fn send_message(
        &self,
        message: serde_json::Value,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<Option<String>, ProviderSessionError> {
        let entry_id = format!("runner-send:{tool_call_id}");
        let entry = serde_json::json!({
            "id": entry_id.clone(),
            "kind": "send-message",
            "message": message,
            "timestampMs": timestamp_ms,
        });
        self.sessions
            .append_agent_transcript_entries(&self.agent_id, &[entry])
            .map_err(|error| ProviderSessionError::Tool(format!(
                "could not persist SendMessage for {}: {error}",
                self.agent_id
            )))?;
        self.fulfill_ack_obligation();
        self.transcript_runtime.track_runner_activity_update(
            &self.agent_id,
            &ActivityUpdate::SendMessage,
            started_at_ms(),
        );
        Ok(Some(entry_id))
    }
}

fn reaction_gateway_args(
    agent_id: &str,
    message_address: &str,
    emoji: &str,
) -> serde_json::Value {
    serde_json::json!({
        "agentId": agent_id,
        "entryId": message_address,
        "emoji": emoji,
    })
}

struct ProductionAgentManagementSink {
    sessions: Arc<ProductionSessionWorkers>,
    forever_box: Arc<ForeverBoxService>,
    messaging: Arc<ProductionAgentToAgentMessaging>,
    agent_messages: Arc<Mutex<Vec<String>>>,
    agent_id: String,
}

impl AgentManagementSink for ProductionAgentManagementSink {
    fn self_agent_id(&self) -> &str {
        &self.agent_id
    }

    fn resolve_image_source(
        &self,
        image: &AgentMessageImage,
        _tool_call_id: &str,
    ) -> Result<AgentMessageImage, ProviderSessionError> {
        let resolved = resolve_production_attachment_source(
            self.sessions.as_ref(),
            self.forever_box.as_ref(),
            &self.agent_id,
            &image.url,
        );
        Ok(AgentMessageImage {
            url: resolved.url,
            alt: image.alt.clone(),
        })
    }

    fn send_to_agent(
        &self,
        target_id: &str,
        message: &str,
        images: &[AgentMessageImage],
        priority: bool,
    ) -> Result<String, ProviderSessionError> {
        let entry_id = self
            .messaging
            .send_to_agent(&self.agent_id, target_id, message, images, priority)
            .map_err(ProviderSessionError::Tool)?;
        if !message.trim().is_empty() {
            if let Ok(mut agent_messages) = self.agent_messages.lock() {
                agent_messages.push(message.to_string());
            }
        }
        Ok(entry_id)
    }

    fn create_agent(
        &self,
        name: &str,
        description: &str,
    ) -> Result<AgentManagementRecord, ProviderSessionError> {
        let profile = SandAgentProfile {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        };
        let record = self
            .sessions
            .materialize_new_session(Some(&profile), "user", None)
            .map_err(ProviderSessionError::Tool)?;
        Ok(AgentManagementRecord {
            id: record.id,
            name: record.profile.name,
        })
    }

    fn update_agent(
        &self,
        agent_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<AgentManagementRecord>, ProviderSessionError> {
        let Some(current) = self
            .sessions
            .get_agent_profile_text(agent_id)
            .map_err(ProviderSessionError::Tool)?
        else {
            return Ok(None);
        };
        let update = AgentProfileUpdate {
            name: name.unwrap_or(&current.name).to_string(),
            description: description.unwrap_or(&current.description).to_string(),
            title: None,
            avatar_shape: None,
            avatar_color: None,
        };
        let updated = self
            .sessions
            .update_agent_profile(agent_id, &update, None)
            .map_err(ProviderSessionError::Tool)?;
        Ok(updated.map(|summary| AgentManagementRecord {
            id: summary.id,
            name: summary.name,
        }))
    }
}

struct ProductionReactionSink {
    host_tx: mpsc::Sender<HostLaneRequest>,
    agent_id: String,
}

impl ReactionSink for ProductionReactionSink {
    fn react(
        &self,
        message_address: &str,
        emoji: &str,
    ) -> Result<(), ProviderSessionError> {
        call_host_lane(
            &self.host_tx,
            "reactToMessage",
            reaction_gateway_args(&self.agent_id, message_address, emoji),
        )
        .map_err(|error| {
            ProviderSessionError::Tool(format!(
                "could not react to {message_address}: {error}"
            ))
        })?;
        Ok(())
    }
}

struct RoutedTurnLeaseGuard {
    runtime: Arc<ProductionTranscriptRuntime>,
    agent_id: String,
    stream_id: String,
    events: GatewayEventHub,
    settled: bool,
}

impl RoutedTurnLeaseGuard {
    fn new(
        runtime: Arc<ProductionTranscriptRuntime>,
        agent_id: String,
        stream_id: String,
        events: GatewayEventHub,
    ) -> Self {
        Self {
            runtime,
            agent_id,
            stream_id,
            events,
            settled: false,
        }
    }

    fn settle(&mut self) {
        if self.settled {
            return;
        }
        self.settled = true;
        if let Ok(Some(settlement)) = self.runtime.settle_routed_turn(
            &self.agent_id,
            &self.stream_id,
            started_at_ms(),
        ) {
            if let Some(watchdog) = settlement.watchdog {
                self.events.publish(serde_json::json!({
                    "channel": "run-queue-watchdog",
                    "payload": {
                        "agentId": watchdog.agent_id,
                        "stage": watchdog.stage.as_str(),
                        "activeLane": watchdog.active_lane.as_str(),
                        "activeSource": watchdog.active_source,
                        "activeRuntimeMs": watchdog.active_runtime_ms,
                        "waitingUserAgeMs": watchdog.waiting_user_age_ms,
                        "ackToken": watchdog.ack_token,
                        "interrupted": false
                    }
                }));
            }
            if let Some(next) = settlement.next {
                self.events.publish(serde_json::json!({
                    "channel": "run-queue-dequeued",
                    "payload": {
                        "agentId": next.agent_id,
                        "lane": next.lane.as_str(),
                        "source": next.source,
                        "queueWaitMs": next.queue_wait_ms,
                        "acceptedToRunMs": next.accepted_to_run_ms,
                        "jumpedBackground": next.jumped_background,
                        "depthUser": next.depth_user,
                        "depthAgent": next.depth_agent,
                        "depthBackground": next.depth_background
                    }
                }));
            }
        }
    }
}

impl Drop for RoutedTurnLeaseGuard {
    fn drop(&mut self) {
        self.settle();
    }
}

struct UnifiedGatewayApi {
    host_tx: mpsc::Sender<HostLaneRequest>,
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
    inference: Arc<ProductionInferenceExtension>,
    content_search: Arc<ProductionContentSearchExtension>,
    events: GatewayEventHub,
    routed_tool_relay: Arc<CoordinatorToolRelay>,
    mcp_lifecycle_relay: Arc<CoordinatorMcpLifecycleRelay>,
    mcp_service: Arc<McpHostService>,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    agent_deletion_runtime: AgentDeletionRuntimeDeps,
    forever_box: Arc<ForeverBoxService>,
    teach_recording: Arc<Mutex<Option<TeachRecordingApi>>>,
    local_exec: Arc<HostLocalExecExtension>,
    session_handoff: BoxHandoffService,
    webauthn_proxy: Arc<HostWebAuthnProxyExtension>,
    trays: Arc<HostTraysExtension>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    generated_agent_runtime: Arc<Mutex<SubagentRuntime>>,
    completion_revivals: Arc<CompletionRevivals>,
    transcript_manager: Arc<TranscriptManager>,
    telemetry_logs: HostStructuredLogTelemetry,
    production_action_auditor: ActionAuditExtension,
    cloud_agents: Arc<SandCloudAgentManager>,
    cloud_agent_watches: Arc<RunnerCloudAgentWatches>,
    secrets: Arc<HostSecretsExtension>,
    local_tool_permission: Arc<HostLocalToolPermissionExtension>,
    auto_review: Arc<HostAutoReviewExtension>,
    host_runner_composition: Arc<HostRunnerComposition>,
    box_store_sync: ProductionBoxStoreSyncApi,
    cross_user: Arc<ProductionCrossUserRuntime>,
    host_upgrade: Arc<ProductionHostUpgradeExtension>,
    automations_lifecycle: Arc<Mutex<Weak<ProductionAutomationsLifecycle>>>,
    create_agent_nonces: Mutex<CreateAgentNonceLedger<serde_json::Value>>,
    last_busy_at_ms: Mutex<u64>,
}

#[derive(Clone)]
struct LocalRoutedRunnerDeps {
    routed_tool_relay: Arc<CoordinatorToolRelay>,
    mcp_service: Arc<McpHostService>,
    auth: Arc<HostAuthExtension>,
    auto_review: Arc<HostAutoReviewExtension>,
    events: GatewayEventHub,
    host_tx: mpsc::Sender<HostLaneRequest>,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
    inference: Arc<ProductionInferenceExtension>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    generated_agent_runtime: Arc<Mutex<SubagentRuntime>>,
    completion_revivals: Arc<CompletionRevivals>,
    forever_box: Arc<ForeverBoxService>,
    local_exec: Arc<HostLocalExecExtension>,
    local_tool_permission: Arc<HostLocalToolPermissionExtension>,
    session_handoff: BoxHandoffService,
    trays: Arc<HostTraysExtension>,
    telemetry_logs: HostStructuredLogTelemetry,
    production_action_auditor: ActionAuditExtension,
    cloud_agents: Arc<SandCloudAgentManager>,
    cloud_agent_watches: Arc<RunnerCloudAgentWatches>,
    host_runner_composition: Arc<HostRunnerComposition>,
    box_store_sync: ProductionBoxStoreSyncApi,
    automations_lifecycle: Arc<Mutex<Weak<ProductionAutomationsLifecycle>>>,
}

#[derive(Clone)]
struct ProductionSubagentTaskSink {
    deps: LocalRoutedRunnerDeps,
    parent_agent_id: String,
    provider: RoutedProvider,
    parent_stream_id: String,
    root_parent_request_id: String,
}

impl SubagentTaskSink for ProductionSubagentTaskSink {
    fn launch_subagent(
        &self,
        prompt: &str,
        subagent_type: &str,
        tool_call_id: &str,
    ) -> Result<SubagentLaunchRecord, ProviderSessionError> {
        let profile = SandAgentProfile {
            name: format!("{} subagent", subagent_type.trim()),
            description: prompt.trim().to_string(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        };
        let child = self
            .deps
            .session_workers
            .materialize_new_session(Some(&profile), "subagent", Some(subagent_type))
            .map_err(ProviderSessionError::Tool)?;
        let child_id = child.id.clone();
        let child_stream_id = uuid::Uuid::new_v4().to_string();
        let normalized_type = subagent_type
            .chars()
            .filter(|ch| !matches!(ch, '-' | '_' | ' '))
            .collect::<String>();
        let args = serde_json::json!({
            "provider": self.provider.as_str(),
            "agentId": child_id,
            "streamId": child_stream_id,
            "messages": [{"role":"user","content":prompt}],
            "requestSource": "subagent",
            "parentAgentId": self.parent_agent_id,
            "subagentType": subagent_type,
            "parentAgentToolCallId": tool_call_id,
            "lineage": {
                "parentRequestId": self.parent_stream_id,
                "rootParentRequestId": self.root_parent_request_id,
                "parentAgentToolCallId": tool_call_id,
            },
            "skipLabeling": true,
            "isComputerUseSubagent": normalized_type.eq_ignore_ascii_case("computeruse"),
            "isBrowserUseSubagent": normalized_type.eq_ignore_ascii_case("browseruse"),
        });
        start_routed_provider_task(
            Arc::clone(&self.deps.routed_tool_relay),
            Arc::clone(&self.deps.mcp_service),
            self.deps.events.clone(),
            self.deps.host_tx.clone(),
            self.deps.data_dir.clone(),
            Arc::clone(&self.deps.request_context),
            Arc::clone(&self.deps.auth),
            Arc::clone(&self.deps.auto_review),
            Arc::clone(&self.deps.experiments),
            Arc::clone(&self.deps.settings),
            Arc::clone(&self.deps.inference),
            Arc::clone(&self.deps.session_workers),
            Arc::clone(&self.deps.runner_registry),
            Arc::clone(&self.deps.ack_obligations),
            Arc::clone(&self.deps.transcript_runtime),
            Arc::clone(&self.deps.generated_agent_runtime),
            Arc::clone(&self.deps.completion_revivals),
            Arc::clone(&self.deps.forever_box),
            Arc::clone(&self.deps.local_exec),
            Arc::clone(&self.deps.local_tool_permission),
            self.deps.session_handoff.clone(),
            Arc::clone(&self.deps.trays),
            self.deps.telemetry_logs.clone(),
            self.deps.production_action_auditor.clone(),
            Arc::clone(&self.deps.cloud_agents),
            Arc::clone(&self.deps.cloud_agent_watches),
            Arc::clone(&self.deps.host_runner_composition),
            self.deps.box_store_sync.clone(),
            Arc::clone(&self.deps.automations_lifecycle),
            None,
            args,
        )
        .map_err(|error| ProviderSessionError::Tool(format!("subagent launch failed: {error:?}")))?;
        Ok(SubagentLaunchRecord {
            id: child.id,
            subagent_type: subagent_type.to_string(),
        })
    }
}

fn publish_generated_subagents(
    events: &GatewayEventHub,
    runtime: &Arc<Mutex<SubagentRuntime>>,
    parent_agent_id: &str,
) {
    let subagents = runtime
        .lock()
        .map(|runtime| runtime.list_subagents())
        .unwrap_or_default()
        .into_iter()
        .map(|(id, record)| {
            let status = match record.status {
                mahayana_host_runtime::runner::subagent_runtime::SubagentStatus::Running => "running",
                mahayana_host_runtime::runner::subagent_runtime::SubagentStatus::Done => "done",
                mahayana_host_runtime::runner::subagent_runtime::SubagentStatus::Error => "error",
                mahayana_host_runtime::runner::subagent_runtime::SubagentStatus::Aborted => "aborted",
            };
            serde_json::json!({
                "subagentId": id,
                "subagentType": record.subagent_type,
                "title": record.title,
                "status": status,
            })
        })
        .collect::<Vec<_>>();
    events.publish(serde_json::json!({
        "channel": "subagents",
        "payload": {
            "parentAgentId": parent_agent_id,
            "subagents": subagents,
        }
    }));
}

#[derive(Clone)]
struct ProductionPendingWakeRuntime {
    gateway: Arc<Mutex<Weak<UnifiedGatewayApi>>>,
    cloud_agent_watches: Arc<RunnerCloudAgentWatches>,
    completion_revivals: Arc<CompletionRevivals>,
}

impl ProductionPendingWakeRuntime {
    fn gateway(&self) -> Option<Arc<UnifiedGatewayApi>> {
        self.gateway
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade()
    }
}

impl PendingWakeRuntimePort for ProductionPendingWakeRuntime {
    fn can_execute(&self) -> bool {
        self.gateway().is_some()
    }

    fn is_agent_gone(&self, agent_id: &str) -> bool {
        self.gateway().is_none_or(|gateway| {
            gateway
                .transcript_runtime
                .session_runtime()
                .is_agent_gone(&gateway.session_workers, agent_id)
        })
    }

    fn is_group_session(&self, agent_id: &str) -> Result<bool, String> {
        let gateway = self
            .gateway()
            .ok_or_else(|| "Host gateway is not ready for pending-wake rearm".to_string())?;
        Ok(gateway
            .session_workers
            .summarize_agent_by_id(agent_id, None)?
            .is_some_and(|summary| summary.is_group))
    }

    fn cloud_watch_is_armed(&self, agent_id: &str, work_id: &str) -> bool {
        self.cloud_agent_watches
            .is_cloud_watch_armed(agent_id, work_id)
    }

    fn watch_cloud_agent(
        &self,
        agent_id: &str,
        work_id: &str,
        quiet_origin: Option<&QuietWakeOrigin>,
    ) -> Result<(), String> {
        let quiet_origin = quiet_origin
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| format!("could not encode quiet wake origin: {error}"))?;
        let _ = self.cloud_agent_watches.watch_cloud_agent(
            agent_id,
            work_id,
            CloudAgentWatchOptions::new(quiet_origin, false),
        );
        Ok(())
    }

    fn watch_background_shell(
        &self,
        _agent_id: &str,
        _work_id: &str,
        _title: Option<&str>,
        _quiet_origin: Option<&QuietWakeOrigin>,
    ) -> Result<(), String> {
        Err("production shell pending-wake rearm is not wired yet".into())
    }

    fn deliver_recreate_interrupted_shell_notice(
        &self,
        marker: &DurablePendingWakeMarker,
    ) -> Result<(), String> {
        self.completion_revivals
            .handle_background_shell_completion(ShellCompletion {
                agent_id: marker.agent_id.clone(),
                shell_id: marker.work_id.clone(),
                title: marker
                    .title
                    .clone()
                    .unwrap_or_else(|| "Background command".into()),
                status: "error".into(),
                detail: Some(
                    "A host restart interrupted observation of this background command before its terminal result could be delivered; the process state did not survive the recreate boundary, so its final state is unknown."
                        .into(),
                ),
                output_path: None,
                quiet_origin: marker.quiet_origin.clone(),
            });
        Ok(())
    }

    fn revive_lost_subagent(&self, wake: LostSubagentWake) -> Result<(), String> {
        self.completion_revivals
            .handle_background_subagent_completion(SubagentCompletion {
                parent_agent_id: wake.parent_agent_id,
                subagent_agent_id: wake.subagent_agent_id,
                title: wake.title,
                subagent_type: wake.subagent_type,
                status: "error".into(),
                result: wake.result,
                quiet_origin: wake.quiet_origin,
            });
        Ok(())
    }

    fn emit_async_tasks_for_agent(&self, agent_id: &str) {
        if let Some(gateway) = self.gateway() {
            gateway.emit_async_tasks_for_agent(agent_id);
        }
    }

    fn report_pending_wake(&self, report: PendingWakeReport) {
        let Some(gateway) = self.gateway() else {
            return;
        };
        gateway.events.publish(serde_json::json!({
            "channel": "pending-wake",
            "payload": {
                "agentId": report.conversation_id,
                "kind": report.kind,
                "workId": report.work_id,
                "outcome": report.outcome,
                "ageMs": report.age_ms,
                "reason": report.reason,
                "isQuietOrigin": report.is_quiet_origin,
            }
        }));
    }
}

#[derive(Clone)]
struct ProductionCompletionRevivalRuntime {
    gateway: Arc<Mutex<Weak<UnifiedGatewayApi>>>,
}

impl ProductionCompletionRevivalRuntime {
    fn gateway(&self) -> Option<Arc<UnifiedGatewayApi>> {
        self.gateway
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade()
    }
}

impl CompletionRevivalRuntimePort for ProductionCompletionRevivalRuntime {
    fn can_execute(&self) -> bool {
        let Some(gateway) = self.gateway() else {
            return false;
        };
        configured_routed_provider(&gateway.data_dir.join("settings.json")).is_some()
    }

    fn is_agent_deleted(&self, agent_id: &str) -> bool {
        self.gateway().is_none_or(|gateway| {
            gateway
                .transcript_runtime
                .session_runtime()
                .is_agent_gone(&gateway.session_workers, agent_id)
        })
    }

    fn is_agent_gone(&self, agent_id: &str) -> bool {
        self.is_agent_deleted(agent_id)
    }

    fn clear_pending_wake(&self, agent_id: &str, kind: PendingWakeKind, work_id: &str) {
        let Some(gateway) = self.gateway() else {
            return;
        };
        if let Some(store) = gateway.transcript_runtime.pending_wake_store() {
            let _ = store.clear_one(agent_id, kind, work_id);
        }
        gateway.emit_async_tasks_for_agent(agent_id);
    }

    fn run_background_revival(
        &self,
        agent_id: &str,
        source: &str,
        prompt: &str,
        is_silence_allowed: bool,
        auto_review_epoch: &str,
    ) -> Result<RevivalExecution, String> {
        let gateway = self
            .gateway()
            .ok_or_else(|| "Host gateway is not ready for background revival".to_string())?;
        let provider = configured_routed_provider(&gateway.data_dir.join("settings.json"))
            .ok_or_else(|| "no routed provider configured for background revival".to_string())?;
        run_local_background_revival_turn(
            gateway.local_routed_runner_deps(),
            provider,
            agent_id,
            source,
            prompt,
            is_silence_allowed,
            auto_review_epoch,
        )
    }

    fn mark_resume_pending_for_quiesced_revival(&self, agent_id: &str) {
        let Some(gateway) = self.gateway() else {
            return;
        };
        if let Some(store) = gateway.transcript_runtime.upgrade_resume_store() {
            let _ = store.mark_pending(UpgradeResumeMarker {
                agent_id: agent_id.to_string(),
                marked_at_ms: started_at_ms() as f64,
                source: Some("background-revival".into()),
                automation_id: None,
                automation_run_id: None,
            });
        }
    }

    fn report_revival(&self, report: RevivalReport) {
        let Some(gateway) = self.gateway() else {
            return;
        };
        let projection = subagent_revival_telemetry(&SubagentRevivalReport {
            parent_agent_id: report.agent_id,
            outcome: report.outcome,
            completion_count: i64::try_from(report.completion_count).unwrap_or(i64::MAX),
            subagent_type: report.subagent_type,
            subagent_agent_id: report.subagent_agent_id,
            reason: report.reason,
            sent_message_count: report
                .sent_message_count
                .map(|count| i64::try_from(count).unwrap_or(i64::MAX)),
            is_quiet_origin: Some(report.is_quiet_origin),
        });
        if let Err(error) = gateway.telemetry_logs.report_projection(&projection) {
            eprintln!("mahayana-host background_revival_telemetry_failed error={error}");
        }
    }

    fn report_revival_error(&self, agent_id: &str, title: &str, error: &str) {
        eprintln!(
            "mahayana-host background_revival_failed agent={agent_id} title={title} error={error}"
        );
    }
}

impl UnifiedGatewayApi {
    fn local_routed_runner_deps(&self) -> LocalRoutedRunnerDeps {
        LocalRoutedRunnerDeps {
            routed_tool_relay: Arc::clone(&self.routed_tool_relay),
            mcp_service: Arc::clone(&self.mcp_service),
            auth: Arc::clone(&self.auth),
            auto_review: Arc::clone(&self.auto_review),
            events: self.events.clone(),
            host_tx: self.host_tx.clone(),
            data_dir: self.data_dir.clone(),
            request_context: Arc::clone(&self.request_context),
            experiments: Arc::clone(&self.experiments),
            settings: Arc::clone(&self.settings),
            inference: Arc::clone(&self.inference),
            session_workers: Arc::clone(&self.session_workers),
            runner_registry: Arc::clone(&self.runner_registry),
            ack_obligations: Arc::clone(&self.ack_obligations),
            transcript_runtime: Arc::clone(&self.transcript_runtime),
            generated_agent_runtime: Arc::clone(&self.generated_agent_runtime),
            completion_revivals: Arc::clone(&self.completion_revivals),
            forever_box: Arc::clone(&self.forever_box),
            local_exec: Arc::clone(&self.local_exec),
            local_tool_permission: Arc::clone(&self.local_tool_permission),
            session_handoff: self.session_handoff.clone(),
            trays: Arc::clone(&self.trays),
            telemetry_logs: self.telemetry_logs.clone(),
            production_action_auditor: self.production_action_auditor.clone(),
            cloud_agents: Arc::clone(&self.cloud_agents),
            cloud_agent_watches: Arc::clone(&self.cloud_agent_watches),
            host_runner_composition: Arc::clone(&self.host_runner_composition),
            box_store_sync: self.box_store_sync.clone(),
            automations_lifecycle: Arc::clone(&self.automations_lifecycle),
        }
    }

    fn emit_async_tasks_for_agent(&self, agent_id: &str) {
        let tasks = self.transcript_runtime.get_async_tasks(agent_id, &[]);
        self.events.publish(serde_json::json!({
            "channel": "async-tasks",
            "payload": {
                "parentAgentId": agent_id,
                "tasks": tasks,
            }
        }));
    }

    fn refresh_production_automations(&self) {
        let lifecycle = self
            .automations_lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade();
        if let Some(lifecycle) = lifecycle {
            // Reconcile both backend relay state and the cloud scheduling
            // shadow. The listener reconnect watcher remains an interactive
            // handoff owner and is never armed merely because a routine exists.
            lifecycle.request_reconcile();
        }
    }

    fn delete_production_automation_schedules(&self, agent_id: &str) {
        let lifecycle = self
            .automations_lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade();
        if let Some(lifecycle) = lifecycle {
            lifecycle.delete_agent_schedules(agent_id);
        }
    }

    fn resume_after_listener_connect(&self, agent_id: &str, platform: &str) -> Result<(), String> {
        let nonce = format!(
            "listener-connect-resume:{agent_id}:{platform}:{}",
            uuid::Uuid::new_v4()
        );
        self.call(
            "sendPrompt",
            listener_connect_resume_args(agent_id, platform, &nonce),
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
    }

    fn dispatch_production_listener_event(&self, event: serde_json::Value) -> bool {
        let Some(provider) = configured_routed_provider(&self.data_dir.join("settings.json")) else {
            return false;
        };
        if provider == RoutedProvider::Cursor {
            // Fail closed: do not ACK backend relay delivery until the Cursor
            // background Runner path is owned by the Rust Host.
            return false;
        }

        let runtime = self.transcript_manager.automation_runtime();
        let Ok(entries) = runtime.list_all_automation_definitions() else {
            return false;
        };
        let lifecycle = self
            .automations_lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade();
        let matches = entries
            .into_iter()
            .filter(|entry| entry.automation.is_enabled)
            .filter(|entry| {
                let cloud = ScheduledCloudAutomation {
                    id: entry.automation.id.clone(),
                    name: entry.automation.name.clone(),
                    prompt: entry.automation.prompt.clone(),
                    is_enabled: entry.automation.is_enabled,
                    trigger: entry.automation.trigger.clone(),
                };
                lifecycle
                    .as_ref()
                    .map(|owner| owner.should_schedule_locally(&entry.agent_id, &cloud))
                    .unwrap_or_else(|| !is_server_schedulable(&entry.automation.trigger))
            })
            .filter(|entry| trigger_matches_event(&entry.automation.trigger, &event, true, false))
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return false;
        }

        let mut accepted = 0usize;
        for entry in matches {
            let runtime = Arc::clone(&runtime);
            let deps = self.local_routed_runner_deps();
            let event = event.clone();
            let agent_id = entry.agent_id;
            let automation_id = entry.automation.id;
            let automation_name = entry.automation.name;
            let spawn = thread::Builder::new()
                .name("mahayana-automation-listener-fire".into())
                .spawn(move || {
                    let execute_deps = deps.clone();
                    let execute_agent_id = agent_id.clone();
                    let execute_automation_id = automation_id.clone();
                    let execute_automation_name = automation_name.clone();
                    let executor = Arc::new(move |_agent_id: &str, _automation_id: &str, _automation_name: &str, prompt: &str| {
                        run_local_automation_turn(
                            execute_deps.clone(),
                            provider,
                            &execute_agent_id,
                            &execute_automation_id,
                            &execute_automation_name,
                            prompt,
                        )
                    });
                    if let Err(error) = runtime.enqueue_event_automation_fire_with(
                        &agent_id,
                        &automation_id,
                        event,
                        None,
                        executor,
                    ) {
                        eprintln!(
                            "[sand:automations] listener fire failed agent={agent_id} automation={automation_id}: {error}"
                        );
                    }
                });
            if spawn.is_ok() {
                accepted += 1;
            }
        }
        accepted > 0
    }

    fn report_production_backend_fire_dropped(
        &self,
        fire: &BackendAutomationFire,
        reason: &str,
        error_type: Option<&str>,
        error_code: Option<&str>,
    ) {
        let now_ms = started_at_ms();
        let scheduled_for_ms = fire.scheduled_for_ms.map(|value| value as f64);
        let lateness_ms = fire
            .scheduled_for_ms
            .map(|value| now_ms.saturating_sub(value) as f64);
        let projection = automation_fire_dropped_telemetry(&AutomationFireDroppedReport {
            conversation_id: fire.sand_agent_id.clone(),
            trigger: if fire.event.is_some() { "event".into() } else { "schedule".into() },
            reason: reason.to_owned(),
            scheduled_for_ms,
            lateness_ms,
            error_type: error_type.map(str::to_owned),
            error_code: error_code.map(str::to_owned),
            run_uuid: Some(fire.id.clone()),
            fire_age_ms: Some(now_ms.saturating_sub(fire.timestamp_ms) as f64),
            has_definition_revision: Some(fire.definition_revision.is_some()),
            box_uptime_ms: None,
        });
        let _ = self.telemetry_logs.report_projection(&projection);
    }

    fn dispatch_production_backend_fire(
        &self,
        fire: BackendAutomationFire,
        completion: Arc<dyn Fn(Option<FireCompletion>) + Send + Sync>,
    ) -> bool {
        let runtime = self.transcript_manager.automation_runtime();
        let Ok(entries) = runtime.list_all_automations() else {
            return false;
        };
        let definition_time_zone = self.session_workers.resolve_user_time_zone();
        match prepare_backend_fire(&entries, &fire, |entry| {
            let cloud = ScheduledCloudAutomation {
                id: entry.automation.id.clone(),
                name: entry.automation.name.clone(),
                prompt: entry.automation.prompt.clone(),
                is_enabled: entry.automation.is_enabled,
                trigger: entry.automation.trigger.clone(),
            };
            sand_cloud_definition(
                &entry.agent_id,
                &cloud,
                definition_time_zone.as_deref(),
            )
            .map(|definition| definition.hash)
        }) {
            PreparedBackendFire::Complete {
                completion: terminal,
                reason,
            } => {
                if reason != "existing_run" {
                    self.report_production_backend_fire_dropped(&fire, reason, None, None);
                }
                completion(Some(terminal));
                true
            }
            PreparedBackendFire::Abandon { .. } => false,
            PreparedBackendFire::Schedule {
                agent_id,
                automation_id,
                run_uuid,
                ..
            } => {
                let Some(provider) =
                    configured_routed_provider(&self.data_dir.join("settings.json"))
                else {
                    return false;
                };
                if provider == RoutedProvider::Cursor {
                    return false;
                }
                let Some(automation) = entries
                    .iter()
                    .find(|entry| {
                        entry.agent_id == agent_id && entry.automation.id == automation_id
                    })
                    .map(|entry| entry.automation.clone())
                else {
                    return false;
                };
                let deps = self.local_routed_runner_deps();
                thread::Builder::new()
                    .name("mahayana-automation-cloud-fire".into())
                    .spawn(move || {
                        let result = runtime.run_server_scheduled_automation_with(
                            &agent_id,
                            &automation_id,
                            run_uuid,
                            |prompt| {
                                run_local_automation_turn(
                                    deps,
                                    provider,
                                    &agent_id,
                                    &automation_id,
                                    &automation.name,
                                    prompt,
                                )
                            },
                        );
                        completion(automation_fire_completion(result));
                    })
                    .is_ok()
            }
            PreparedBackendFire::Event {
                agent_id,
                automation_id,
                run_uuid,
                event,
            } => {
                let Some(provider) =
                    configured_routed_provider(&self.data_dir.join("settings.json"))
                else {
                    return false;
                };
                if provider == RoutedProvider::Cursor {
                    return false;
                }
                let Some(automation) = entries
                    .iter()
                    .find(|entry| {
                        entry.agent_id == agent_id && entry.automation.id == automation_id
                    })
                    .map(|entry| entry.automation.clone())
                else {
                    return false;
                };
                let deps = self.local_routed_runner_deps();
                thread::Builder::new()
                    .name("mahayana-automation-cloud-event".into())
                    .spawn(move || {
                        let execute_deps = deps.clone();
                        let execute_agent_id = agent_id.clone();
                        let execute_automation_id = automation_id.clone();
                        let execute_automation_name = automation.name.clone();
                        let executor = Arc::new(move |_agent_id: &str, _automation_id: &str, _automation_name: &str, prompt: &str| {
                            run_local_automation_turn(
                                execute_deps.clone(),
                                provider,
                                &execute_agent_id,
                                &execute_automation_id,
                                &execute_automation_name,
                                prompt,
                            )
                        });
                        let result = runtime.enqueue_event_automation_fire_with(
                            &agent_id,
                            &automation_id,
                            event,
                            Some(run_uuid),
                            executor,
                        );
                        completion(automation_fire_completion(result));
                    })
                    .is_ok()
            }
        }
    }

    fn resume_interrupted_upgrade_turns(&self) -> Result<(), String> {
        let Some(store) = self.transcript_runtime.upgrade_resume_store() else {
            return Ok(());
        };
        let pending = store.list_pending();
        if pending.is_empty() {
            return Ok(());
        }

        store.clear_all();
        for marker in pending {
            let Some(summary) = self
                .session_workers
                .summarize_agent_by_id(&marker.agent_id, None)?
            else {
                continue;
            };
            if summary.is_group {
                continue;
            }

            let resumed_source = match marker.source.as_deref() {
                Some("notification") => "background-revival",
                Some(source) if !source.trim().is_empty() => source,
                _ => "handoff-resume",
            };
            let automation_wake = if resumed_source == "automation" {
                marker.automation_id.as_deref().and_then(|automation_id| {
                    self.transcript_manager
                        .automation_runtime()
                        .get_agent_automations(&marker.agent_id)
                        .ok()?
                        .into_iter()
                        .find(|automation| automation.id == automation_id)
                        .map(|automation| serde_json::json!({
                            "id": automation.id,
                            "name": automation.name,
                        }))
                })
            } else {
                None
            };

            let provider = configured_routed_provider(&self.data_dir.join("settings.json"));
            let result = match provider {
                Some(provider) if provider != RoutedProvider::Cursor => {
                    start_local_upgrade_resume_turn(
                        self.local_routed_runner_deps(),
                        provider,
                        &marker,
                        automation_wake,
                    )
                }
                _ => {
                    let mut args = serde_json::json!({
                        "agentId": marker.agent_id,
                        "prompt": build_upgrade_resume_prompt(resumed_source),
                        "clientNonce": format!(
                            "upgrade-resume:{}:{}",
                            marker.agent_id,
                            uuid::Uuid::new_v4()
                        ),
                        "appendUserMessage": false,
                        "hidden": true,
                        "requestSource": resumed_source,
                        "upgradeResume": true,
                        "skipAckObligation": true,
                        "awaitTurn": false,
                    });
                    if let Some(automation_wake) = automation_wake {
                        args["automationWake"] = automation_wake;
                    }
                    self.call("sendPrompt", args)
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                }
            };

            if let Err(error) = result {
                let mut tray =
                    provider_failure_tray(&marker.agent_id, &error, started_at_ms() as i64);
                tray.title = "Agent failed to resume after host update".into();
                self.trays.push_error(tray);
                eprintln!(
                    "mahayana-host upgrade_resume_failed agent={} error={error}",
                    marker.agent_id
                );
            }
        }
        Ok(())
    }

    fn preempt_group_member_runs_for_direct_send(&self, args: &serde_json::Value) {
        let request_source = args
            .get("requestSource")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let is_direct = matches!(request_source, None | Some("turn"))
            && args.get("automationWake").is_none_or(serde_json::Value::is_null)
            && args.get("groupContext").is_none_or(serde_json::Value::is_null);
        if !is_direct {
            return;
        }
        let Some(agent_id) = args
            .get("agentId")
            .or_else(|| args.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return;
        };
        for stream_id in self
            .runner_registry
            .active_stream_ids_for_agent(agent_id)
            .into_iter()
            .filter(|stream_id| stream_id.starts_with("group-member-"))
        {
            let _ = self.runner_registry.cancel_stream(
                &stream_id,
                "direct user message preempted group member turn",
            );
        }
    }

    fn persisted_user_entry(
        &self,
        agent_id: &str,
        context: &PersistedSendContext,
    ) -> Result<Option<serde_json::Value>, ProductionSendError> {
        let Some(user_message_id) = context.user_message_id.as_deref() else {
            return Ok(None);
        };
        Ok(self
            .session_workers
            .read_agent_transcript_entries(agent_id)
            .map_err(ProductionSendError::Internal)?
            .into_iter()
            .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(user_message_id)))
    }

    fn dispatch_mirror_or_group_send_if_supported(
        &self,
        args: &serde_json::Value,
        shaped_args: &serde_json::Value,
        context: &PersistedSendContext,
    ) -> Result<Option<serde_json::Value>, ProductionSendError> {
        let Some(agent_id) = args.get("agentId").or_else(|| args.get("id"))
            .and_then(serde_json::Value::as_str).map(str::trim).filter(|value| !value.is_empty())
        else { return Ok(None); };
        let agent_dir = self.session_workers.agents_root().join(agent_id);

        if let Some(remote_room) =
            mahayana_host_runtime::groups::remote_room_store::read_sand_remote_room_config(&agent_dir)
        {
            if remote_room.is_revoked == Some(true) {
                return Err(ProductionSendError::Rejected("This shared room is no longer active.".into()));
            }
            let has_files = shaped_args.get("attachmentPaths").and_then(serde_json::Value::as_array)
                .is_some_and(|values| !values.is_empty());
            let has_videos = shaped_args.get("selectedVideos").and_then(serde_json::Value::as_array)
                .is_some_and(|values| !values.is_empty());
            if has_files || has_videos {
                return Err(ProductionSendError::BadRequest(
                    "Shared mirror rooms only support image attachments.".into(),
                ));
            }
            let entry = self.persisted_user_entry(agent_id, context)?.ok_or_else(|| {
                ProductionSendError::Internal("mirror-room send is missing its durable user entry".into())
            })?;
            self.cross_user.publish_room_entry_and_wait(&remote_room.room_id, &entry)
                .map_err(ProductionSendError::Internal)?;
            return Ok(Some(serde_json::json!({"accepted": true, "mirrorRoom": true})));
        }

        let Some(group_config) =
            mahayana_host_runtime::groups::group_store::read_sand_group_config(&agent_dir)
        else { return Ok(None); };
        if let Some(shared_room_id) = group_config.shared_room_id.as_deref() {
            if let Some(entry) = self.persisted_user_entry(agent_id, context)? {
                self.cross_user.publish_room_entry_and_wait(shared_room_id, &entry)
                    .map_err(ProductionSendError::Internal)?;
            }
        }
        let Some(provider) = configured_routed_provider(&self.data_dir.join("settings.json")) else {
            return Ok(None);
        };
        let deps = self.local_routed_runner_deps();
        let executor: GroupMemberTurnExecutor = Arc::new(move |request| {
            run_local_group_member_turn(deps.clone(), provider, request)
        });
        let epoch = self.transcript_runtime.current_turn_epoch(agent_id);
        match dispatch_local_group_send(
            Arc::clone(&self.session_workers),
            Arc::clone(&self.transcript_runtime),
            agent_id,
            epoch,
            executor,
            self.cross_user.remote_executor(),
        ).map_err(ProductionSendError::Internal)? {
            LocalGroupFanoutDisposition::NotGroup | LocalGroupFanoutDisposition::DeferredRemote { .. } => Ok(None),
            LocalGroupFanoutDisposition::Completed { posted_messages, member_failures } => Ok(Some(serde_json::json!({
                "accepted": true,
                "groupFanout": true,
                "postedMessages": posted_messages,
                "memberFailureCount": member_failures.len(),
            }))),
        }
    }
}

fn run_local_group_member_turn(
    deps: LocalRoutedRunnerDeps,
    provider: RoutedProvider,
    request: GroupMemberTurnRequest,
) -> Result<Vec<String>, String> {
    let member_id = request.member.id.clone();
    let before = deps
        .session_workers
        .read_agent_transcript_entries(&member_id)?;
    let stream_id = format!("group-member-{}", uuid::Uuid::new_v4());
    let client_nonce = format!("group-member:{}:{}", member_id, uuid::Uuid::new_v4());
    let admission_args = serde_json::json!({
        "agentId": member_id,
        "prompt": request.prompt,
        "clientNonce": client_nonce,
        "streamId": stream_id,
        "appendUserMessage": false,
        "requestSource": "group-member",
        "groupMemberTurn": true,
        "skipAckObligation": true,
    });
    deps.transcript_runtime
        .accept_routed_send(&admission_args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .map_err(|error| error.to_string())?;

    let receiver = deps.events.subscribe();
    let runner_args = serde_json::json!({
        "provider": provider.as_str(),
        "agentId": member_id,
        "streamId": stream_id,
        "requestSource": "group-member",
        "groupMemberTurn": true,
        "messages": [
            {
                "role": "system",
                "content": request.system_prompt,
            },
            {
                "role": "user",
                "content": request.prompt,
            }
        ],
    });
    start_routed_provider_task(
        deps.routed_tool_relay,
        deps.mcp_service,
        deps.events.clone(),
        deps.host_tx,
        deps.data_dir,
        deps.request_context,
        deps.auth,
        deps.auto_review,
        deps.experiments,
        deps.settings,
        deps.inference,
        Arc::clone(&deps.session_workers),
        Arc::clone(&deps.runner_registry),
        deps.ack_obligations,
        deps.transcript_runtime,
        deps.generated_agent_runtime,
        deps.completion_revivals,
        deps.forever_box,
        deps.local_exec,
        deps.local_tool_permission,
        deps.session_handoff,
        deps.trays,
        deps.telemetry_logs,
        deps.production_action_auditor,
        deps.cloud_agents,
        deps.cloud_agent_watches,
        deps.host_runner_composition,
        deps.box_store_sync,
        Arc::clone(&deps.automations_lifecycle),
        None,
        runner_args,
    )
    .map_err(|error| error.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(30 * 60);
    loop {
        let now = Instant::now();
        if now >= deadline {
            let _ = deps.runner_registry.cancel_stream(
                &stream_id,
                "group member turn timed out",
            );
            return Err(format!("group member turn timed out: {member_id}"));
        }
        let wait = deadline
            .saturating_duration_since(now)
            .min(Duration::from_millis(250));
        match receiver.recv_timeout(wait) {
            Ok(event) => {
                if event.get("channel").and_then(serde_json::Value::as_str)
                    != Some(RUNNER_INFERENCE_EVENT_CHANNEL)
                {
                    continue;
                }
                let Some(payload) = event.get("payload") else {
                    continue;
                };
                if payload
                    .get("streamId")
                    .and_then(serde_json::Value::as_str)
                    != Some(stream_id.as_str())
                {
                    continue;
                }
                match payload.get("type").and_then(serde_json::Value::as_str) {
                    Some("completed") => break,
                    Some("failed" | "cancelled") => {
                        return Err(payload
                            .get("message")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("group member Runner failed")
                            .to_string());
                    }
                    _ => {}
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Host event bus disconnected during group member turn".into());
            }
        }
    }

    let after = deps
        .session_workers
        .read_agent_transcript_entries(&member_id)?;
    Ok(collect_new_member_send_messages(&before, &after))
}

fn start_local_upgrade_resume_turn(
    deps: LocalRoutedRunnerDeps,
    provider: RoutedProvider,
    marker: &UpgradeResumeMarker,
    automation_wake: Option<serde_json::Value>,
) -> Result<(), String> {
    if provider == RoutedProvider::Cursor {
        return Err("Cursor upgrade resume stays on the compatibility production path".into());
    }

    let resumed_source = match marker.source.as_deref() {
        Some("notification") => "background-revival".to_string(),
        Some(source) if !source.trim().is_empty() => source.to_string(),
        _ => "handoff-resume".to_string(),
    };
    let prompt = build_upgrade_resume_prompt(&resumed_source);
    let stream_id = format!("upgrade-resume-{}", uuid::Uuid::new_v4());
    let client_nonce = format!(
        "upgrade-resume:{}:{}",
        marker.agent_id,
        uuid::Uuid::new_v4()
    );
    let ack_token = if matches!(resumed_source.as_str(), "turn" | "handoff-resume") {
        deps.ack_obligations
            .mint_ack_run_token(&marker.agent_id)
            .map_err(|error| error.to_string())?
    } else {
        None
    };

    let mut admission_args = serde_json::json!({
        "agentId": marker.agent_id,
        "prompt": prompt,
        "clientNonce": client_nonce,
        "streamId": stream_id,
        "appendUserMessage": false,
        "hidden": true,
        "requestSource": "upgrade-resume",
        "ackToken": ack_token,
        "skipAckObligation": true,
    });
    if let Some(automation_wake) = automation_wake.clone() {
        admission_args["automationWake"] = automation_wake;
    }

    if let Err(error) = deps.transcript_runtime.accept_routed_send(&admission_args, |_| {
        Ok::<_, ProductionSendError>(PersistedSendContext::default())
    }) {
        deps.ack_obligations
            .retire_ack_run_token(&marker.agent_id, ack_token.as_deref());
        return Err(error.to_string());
    }

    let mut runner_args = serde_json::json!({
        "provider": provider.as_str(),
        "agentId": marker.agent_id,
        "streamId": stream_id,
        "requestSource": resumed_source,
        "hidden": true,
        "upgradeResume": true,
        "messages": [{
            "role": "user",
            "content": prompt,
        }],
    });
    if let Some(ack_token) = ack_token {
        runner_args["ackToken"] = serde_json::Value::String(ack_token);
    }
    if let Some(automation_wake) = automation_wake {
        runner_args["automationWake"] = automation_wake;
    }

    start_routed_provider_task(
        deps.routed_tool_relay,
        deps.mcp_service,
        deps.events.clone(),
        deps.host_tx,
        deps.data_dir,
        deps.request_context,
        deps.auth,
        deps.auto_review,
        deps.experiments,
        deps.settings,
        deps.inference,
        Arc::clone(&deps.session_workers),
        Arc::clone(&deps.runner_registry),
        deps.ack_obligations,
        deps.transcript_runtime,
        deps.generated_agent_runtime,
        deps.completion_revivals,
        deps.forever_box,
        deps.local_exec,
        deps.local_tool_permission,
        deps.session_handoff,
        deps.trays,
        deps.telemetry_logs,
        deps.production_action_auditor,
        deps.cloud_agents,
        deps.cloud_agent_watches,
        deps.host_runner_composition,
        deps.box_store_sync,
        Arc::clone(&deps.automations_lifecycle),
        None,
        runner_args,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn listener_connection_state_or_disconnected(
    platform: &str,
    result: Result<bool, String>,
) -> bool {
    match result {
        Ok(connected) => connected,
        Err(_) => {
            eprintln!(
                "[sand:listener-integrations] {platform} connection read degraded to disconnected"
            );
            false
        }
    }
}

fn listener_connect_resume_args(
    agent_id: &str,
    platform: &str,
    client_nonce: &str,
) -> serde_json::Value {
    let prompt = if platform == "slack" {
        "Slack is connected. Continue configuring the listener from where you left off. If this routine listens in a Slack channel, make sure the bot is invited to that channel."
    } else {
        "The integration is connected. Continue configuring the listener from where you left off."
    };
    serde_json::json!({
        "agentId": agent_id,
        "prompt": prompt,
        "clientNonce": client_nonce,
        "appendUserMessage": false,
        "hidden": true,
        "requestSource": "handoff-resume",
        "awaitTurn": false,
    })
}

fn automation_fire_completion(
    result: Result<Option<FireAutomationOutcome>, String>,
) -> Option<FireCompletion> {
    match result {
        Ok(Some(FireAutomationOutcome::Ok)) => Some(FireCompletion::succeeded()),
        Ok(Some(FireAutomationOutcome::Error)) => Some(FireCompletion::failed(
            "Automation run failed on the Sand box",
        )),
        Ok(Some(FireAutomationOutcome::Interrupted)) => Some(FireCompletion::failed(
            "Automation run was interrupted on the Sand box",
        )),
        Ok(None) => None,
        Err(error) => Some(FireCompletion::failed(error)),
    }
}

fn automation_terminal_from_event(
    event: &serde_json::Value,
    stream_id: &str,
) -> Option<Result<AutomationExecutionResult, String>> {
    if event.get("channel").and_then(serde_json::Value::as_str)
        != Some(RUNNER_INFERENCE_EVENT_CHANNEL)
    {
        return None;
    }
    let payload = event.get("payload")?;
    if payload.get("streamId").and_then(serde_json::Value::as_str) != Some(stream_id) {
        return None;
    }
    match payload.get("type").and_then(serde_json::Value::as_str) {
        Some("completed") => Some(Ok(AutomationExecutionResult::Completed)),
        Some("cancelled") => Some(Ok(AutomationExecutionResult::Interrupted {
            detail: "Interrupted before it finished.".into(),
        })),
        Some("failed") => Some(Err(
            payload
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("automation Runner failed")
                .to_string(),
        )),
        _ => None,
    }
}

fn run_local_automation_turn(
    deps: LocalRoutedRunnerDeps,
    provider: RoutedProvider,
    agent_id: &str,
    automation_id: &str,
    automation_name: &str,
    prompt: &str,
) -> Result<AutomationExecutionResult, String> {
    if provider == RoutedProvider::Cursor {
        return Err("Cursor automation turns remain on the compatibility path".into());
    }
    let stream_id = format!("automation-{}", uuid::Uuid::new_v4());
    let client_nonce = format!(
        "automation:{}:{}:{}",
        agent_id,
        automation_id,
        uuid::Uuid::new_v4()
    );
    let automation_wake = serde_json::json!({
        "id": automation_id,
        "name": automation_name,
    });
    let admission_args = serde_json::json!({
        "agentId": agent_id,
        "prompt": prompt,
        "clientNonce": client_nonce,
        "streamId": stream_id,
        "appendUserMessage": false,
        "hidden": true,
        "requestSource": "automation",
        "automationWake": automation_wake,
        "skipAckObligation": true,
    });
    deps.transcript_runtime
        .accept_routed_send(&admission_args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .map_err(|error| error.to_string())?;

    let receiver = deps.events.subscribe();
    let runner_args = serde_json::json!({
        "provider": provider.as_str(),
        "agentId": agent_id,
        "streamId": stream_id,
        "requestSource": "automation",
        "automationWake": {
            "id": automation_id,
            "name": automation_name,
        },
        "messages": [{
            "role": "user",
            "content": prompt,
        }],
    });
    start_routed_provider_task(
        deps.routed_tool_relay,
        deps.mcp_service,
        deps.events.clone(),
        deps.host_tx,
        deps.data_dir,
        deps.request_context,
        deps.auth,
        deps.auto_review,
        deps.experiments,
        deps.settings,
        deps.inference,
        Arc::clone(&deps.session_workers),
        Arc::clone(&deps.runner_registry),
        deps.ack_obligations,
        deps.transcript_runtime,
        deps.generated_agent_runtime,
        deps.completion_revivals,
        deps.forever_box,
        deps.local_exec,
        deps.local_tool_permission,
        deps.session_handoff,
        deps.trays,
        deps.telemetry_logs,
        deps.production_action_auditor,
        deps.cloud_agents,
        deps.cloud_agent_watches,
        deps.host_runner_composition,
        deps.box_store_sync,
        Arc::clone(&deps.automations_lifecycle),
        None,
        runner_args,
    )
    .map_err(|error| error.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(30 * 60);
    loop {
        let now = Instant::now();
        if now >= deadline {
            let _ = deps
                .runner_registry
                .cancel_stream(&stream_id, "automation turn timed out");
            return Ok(AutomationExecutionResult::Interrupted {
                detail: "Interrupted before it finished.".into(),
            });
        }
        let wait = deadline
            .saturating_duration_since(now)
            .min(Duration::from_millis(250));
        match receiver.recv_timeout(wait) {
            Ok(event) => {
                if let Some(result) = automation_terminal_from_event(&event, &stream_id) {
                    return result;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Host event bus disconnected during automation turn".into());
            }
        }
    }
}

fn run_local_background_revival_turn(
    deps: LocalRoutedRunnerDeps,
    provider: RoutedProvider,
    agent_id: &str,
    source: &str,
    prompt: &str,
    is_silence_allowed: bool,
    auto_review_epoch: &str,
) -> Result<RevivalExecution, String> {
    let stream_id = format!("background-revival-{}", uuid::Uuid::new_v4());
    let client_nonce = format!(
        "background-revival:{}:{}",
        agent_id,
        uuid::Uuid::new_v4()
    );
    let before = deps
        .session_workers
        .read_agent_transcript_entries(agent_id)
        .unwrap_or_default();
    let admission_args = serde_json::json!({
        "agentId": agent_id,
        "prompt": prompt,
        "clientNonce": client_nonce,
        "streamId": stream_id,
        "appendUserMessage": false,
        "hidden": true,
        "requestSource": source,
        "skipAckObligation": true,
    });
    deps.transcript_runtime
        .accept_routed_send(&admission_args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .map_err(|error| error.to_string())?;

    let receiver = deps.events.subscribe();
    let runner_args = serde_json::json!({
        "provider": provider.as_str(),
        "agentId": agent_id,
        "streamId": stream_id,
        "requestSource": source,
        "hidden": true,
        "isSilenceAllowed": is_silence_allowed,
        "autoReviewEpoch": auto_review_epoch,
        "messages": [{
            "role": "user",
            "content": prompt,
        }],
    });
    start_routed_provider_task(
        deps.routed_tool_relay,
        deps.mcp_service,
        deps.events.clone(),
        deps.host_tx,
        deps.data_dir,
        deps.request_context,
        deps.auth,
        deps.auto_review,
        deps.experiments,
        deps.settings,
        deps.inference,
        Arc::clone(&deps.session_workers),
        Arc::clone(&deps.runner_registry),
        deps.ack_obligations,
        deps.transcript_runtime,
        deps.generated_agent_runtime,
        deps.completion_revivals,
        deps.forever_box,
        deps.local_exec,
        deps.local_tool_permission,
        deps.session_handoff,
        deps.trays,
        deps.telemetry_logs,
        deps.production_action_auditor,
        deps.cloud_agents,
        deps.cloud_agent_watches,
        deps.host_runner_composition,
        deps.box_store_sync,
        Arc::clone(&deps.automations_lifecycle),
        None,
        runner_args,
    )
    .map_err(|error| error.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(30 * 60);
    loop {
        let now = Instant::now();
        if now >= deadline {
            let _ = deps
                .runner_registry
                .cancel_stream(&stream_id, "background revival timed out");
            return Ok(RevivalExecution {
                aborted: true,
                quiesced_for_upgrade: false,
                sent_message_count: 0,
            });
        }
        let wait = deadline
            .saturating_duration_since(now)
            .min(Duration::from_millis(250));
        match receiver.recv_timeout(wait) {
            Ok(event) => {
                let Some(result) = automation_terminal_from_event(&event, &stream_id) else {
                    continue;
                };
                match result {
                    Ok(AutomationExecutionResult::Completed) => {
                        let after = deps
                            .session_workers
                            .read_agent_transcript_entries(agent_id)
                            .unwrap_or_default();
                        return Ok(RevivalExecution {
                            aborted: false,
                            quiesced_for_upgrade: false,
                            sent_message_count: collect_new_member_send_messages(&before, &after)
                                .len(),
                        });
                    }
                    Ok(AutomationExecutionResult::Interrupted { .. }) => {
                        return Ok(RevivalExecution {
                            aborted: true,
                            quiesced_for_upgrade: false,
                            sent_message_count: 0,
                        });
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Host event bus disconnected during background revival".into());
            }
        }
    }
}

fn start_ack_redrive_worker(
    api: Arc<UnifiedGatewayApi>,
    ack_obligations: Arc<AckObligations>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    events: GatewayEventHub,
    stop: Arc<AtomicBool>,
) -> io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name("mahayana-ack-redrive".into())
        .spawn(move || {
            let _ = ack_obligations.arm_boot_redrives(started_at_ms());
            while !stop.load(Ordering::Acquire) {
                if api.transcript_runtime.is_quiescing_for_upgrade() {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                let now_ms = started_at_ms();
                let obligations = ack_obligations.pending_obligations();
                for obligation in &obligations {
                    let agent_id = obligation.agent_id.as_str();
                    let active = !runner_registry.active_stream_ids_for_agent(agent_id).is_empty()
                        || api.transcript_runtime.is_agent_running(agent_id);
                    if !active && ack_obligations.redrive_schedule(agent_id).is_none() {
                        let _ = ack_obligations
                            .schedule_ack_redrive_after_idle(agent_id, now_ms);
                    }
                }

                for obligation in obligations {
                    if stop.load(Ordering::Acquire)
                        || api.transcript_runtime.is_quiescing_for_upgrade()
                    {
                        break;
                    }
                    let agent_id = obligation.agent_id.clone();
                    if !runner_registry.active_stream_ids_for_agent(&agent_id).is_empty()
                        || api.transcript_runtime.is_agent_running(&agent_id)
                    {
                        continue;
                    }
                    let Some(trigger) =
                        ack_obligations.take_due_redrive(&agent_id, now_ms)
                    else {
                        continue;
                    };
                    let agent_exists = session_workers
                        .session_db_path(&agent_id)
                        .is_ok_and(|path| path.is_file());
                    match ack_obligations.prepare_redrive(&agent_id, agent_exists) {
                        Ok(AckRedrivePreparation::Missing) => {}
                        Ok(AckRedrivePreparation::LostAgentDeleted(lost)) => {
                            events.publish(serde_json::json!({
                                "channel": "ack-obligation",
                                "payload": {
                                    "agentId": agent_id,
                                    "outcome": "lost",
                                    "reason": "agent_deleted",
                                    "ageMs": started_at_ms() as f64 - lost.created_at_ms,
                                    "coalescedCount": lost.coalesced_count,
                                    "redriveAttempts": lost.redrive_attempts,
                                }
                            }));
                        }
                        Ok(AckRedrivePreparation::LostMaxRedrives(lost)) => {
                            events.publish(serde_json::json!({
                                "channel": "ack-obligation",
                                "payload": {
                                    "agentId": agent_id,
                                    "outcome": "lost",
                                    "reason": "max_redrives",
                                    "ageMs": started_at_ms() as f64 - lost.created_at_ms,
                                    "coalescedCount": lost.coalesced_count,
                                    "redriveAttempts": lost.redrive_attempts,
                                }
                            }));
                        }
                        Ok(AckRedrivePreparation::Ready(bumped)) => {
                            let send_now_ms = started_at_ms();
                            events.publish(serde_json::json!({
                                "channel": "ack-obligation",
                                "payload": {
                                    "agentId": agent_id,
                                    "outcome": "redrive",
                                    "reason": trigger.as_str(),
                                    "ageMs": send_now_ms as f64 - bumped.created_at_ms,
                                    "coalescedCount": bumped.coalesced_count,
                                    "redriveAttempts": bumped.redrive_attempts,
                                }
                            }));
                            let args = build_ack_redrive_send_args(
                                &agent_id,
                                &bumped,
                                trigger,
                                send_now_ms,
                            );
                            if let Err(error) = api.call("sendPrompt", args) {
                                eprintln!(
                                    "mahayana-host-ack redrive_failed agent={agent_id} error={error}"
                                );
                                events.publish(serde_json::json!({
                                    "channel": "ack-obligation",
                                    "payload": {
                                        "agentId": agent_id,
                                        "outcome": "redrive_error",
                                        "reason": trigger.as_str(),
                                        "redriveAttempts": bumped.redrive_attempts,
                                        "message": error.to_string(),
                                    }
                                }));
                            }
                        }
                        Err(error) => {
                            eprintln!(
                                "mahayana-host-ack redrive_prepare_failed agent={agent_id} error={error}"
                            );
                        }
                    }
                }

                let mut slept = 0u64;
                while slept < 100 && !stop.load(Ordering::Acquire) {
                    let slice = (100 - slept).min(25);
                    thread::sleep(Duration::from_millis(slice));
                    slept = slept.saturating_add(slice);
                }
            }
        })
}

fn project_forever_box_status(
    status: &BoxStatus,
    handoff: Option<&PendingHandoff>,
) -> serde_json::Value {
    let windows = status.windows.as_ref().map(|windows| {
        windows
            .iter()
            .map(|window| {
                serde_json::json!({
                    "windowIndex": window.window_index,
                    "vncUrl": window.vnc_url,
                })
            })
            .collect::<Vec<_>>()
    });
    let handoff = handoff.map(|pending| {
        serde_json::json!({
            "requestId": pending.request_id,
            "instruction": pending.instruction,
            "snapshotDataUrl": pending.snapshot_data_url,
        })
    });
    serde_json::json!({
        "agentId": status.agent_id,
        "state": status.state,
        "vncUrl": status.vnc_url,
        "windows": windows,
        "imageUpdateAvailable": status.image_update_available,
        "pull": status.pull_percent.map(|percent| serde_json::json!({ "percent": percent })),
        "handoff": handoff,
    })
}

fn required_box_agent_id<'a>(
    method: &str,
    args: &'a serde_json::Value,
) -> Result<&'a str, GatewayCommandError> {
    args.get("id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GatewayCommandError::BadRequest(format!("{method} requires id")))
}

fn call_host_lane(
    host_tx: &mpsc::Sender<HostLaneRequest>,
    method: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    let (reply, result) = mpsc::sync_channel(1);
    host_tx
        .send(HostLaneRequest::Gateway {
            method: method.to_string(),
            args,
            reply,
        })
        .map_err(|_| GatewayCommandError::Internal("Mahayana Host lane is closed".into()))?;
    result
        .recv_timeout(Duration::from_secs(120))
        .map_err(|_| GatewayCommandError::Internal("Mahayana Host gateway request timed out".into()))?
}

struct ProductionAutoReviewGateDeps {
    auto_review: Arc<HostAutoReviewExtension>,
    controller: Arc<SandAutoReviewController>,
    box_id: String,
}

impl AutoReviewGateDependencies for ProductionAutoReviewGateDeps {
    fn base_modes(&self) -> SandAutoReviewModes {
        self.auto_review.current_modes()
    }

    fn current_modes(&self) -> Option<SandAutoReviewModes> {
        Some(self.auto_review.current_modes())
    }

    fn controller(&self) -> Option<Arc<SandAutoReviewController>> {
        Some(Arc::clone(&self.controller))
    }

    fn resolve_box_id(&self) -> String {
        self.box_id.clone()
    }

    fn instructions(&self) -> Option<AutoReviewInstructions> {
        Some(self.auto_review.instructions())
    }
}

type RoutedMcpAutoReviewCallback = Arc<
    dyn Fn(&RoutedToolDefinition, &serde_json::Value, &str)
            -> Result<Option<String>, ProviderSessionError>
        + Send
        + Sync
        + 'static,
>;

#[derive(Clone)]
struct CoordinatorRoutedToolBridge {
    relay: Arc<CoordinatorToolRelay>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    agent_id: String,
    mcp_review: Option<RoutedMcpAutoReviewCallback>,
}

impl RoutedToolBridge for CoordinatorRoutedToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let value = self
            .relay
            .request(ROUTED_TOOL_LIST_METHOD, serde_json::json!({}))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        decode_routed_tools(value)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: serde_json::Value,
        tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        if let Some(review) = self.mcp_review.as_ref() {
            if let Some(reason) = review(tool, &args, tool_call_id)? {
                return Ok(serde_json::Value::String(reason));
            }
        }
        let activity_args = serde_json::json!({
            "providerIdentifier": tool.provider_identifier,
            "serverIdentifier": tool.provider_identifier,
        })
        .to_string();
        self.transcript_runtime.track_runner_activity_update(
            &self.agent_id,
            &ActivityUpdate::ToolCall {
                id: tool_call_id.to_string(),
                name: "mcpToolCall".into(),
                status: "pending".into(),
                args: Some(activity_args.clone()),
                summary: tool.description.clone(),
            },
            started_at_ms(),
        );
        let result = self.relay
            .request(
                ROUTED_TOOL_EXECUTE_METHOD,
                serde_json::json!({
                    "providerIdentifier": tool.provider_identifier,
                    "name": tool.name,
                    "toolName": tool.tool_name,
                    "args": args,
                    "toolCallId": tool_call_id,
                    "agentId": self.agent_id,
                }),
            )
            .map_err(|error| ProviderSessionError::Tool(error.to_string()));
        self.transcript_runtime.track_runner_activity_update(
            &self.agent_id,
            &ActivityUpdate::ToolCall {
                id: tool_call_id.to_string(),
                name: "mcpToolCall".into(),
                status: if result.is_ok() { "completed" } else { "failed" }.into(),
                args: Some(activity_args),
                summary: tool.description.clone(),
            },
            started_at_ms(),
        );
        result
    }
}

fn decode_routed_tools(
    value: serde_json::Value,
) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
    let rows = value.as_array().ok_or_else(|| {
        ProviderSessionError::Protocol("listRoutedMcpTools did not return an array".into())
    })?;
    Ok(rows
        .iter()
        .filter_map(|row| {
            Some(RoutedToolDefinition {
                name: row.get("name")?.as_str()?.to_string(),
                provider_identifier: row.get("providerIdentifier")?.as_str()?.to_string(),
                tool_name: row.get("toolName")?.as_str()?.to_string(),
                description: row.get("description").and_then(serde_json::Value::as_str).map(str::to_string),
                input_schema: row.get("inputSchema").cloned().unwrap_or_else(|| {
                    serde_json::json!({"type":"object","additionalProperties":true})
                }),
            })
        })
        .collect())
}

fn decode_provider_messages(
    args: &serde_json::Value,
) -> Result<Vec<ProviderMessage>, GatewayCommandError> {
    let rows = args
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires messages".into()
        ))?;
    let messages = rows
        .iter()
        .filter_map(|row| {
            Some(ProviderMessage {
                role: row.get("role")?.as_str()?.to_string(),
                content: row.get("content")?.as_str()?.to_string(),
            })
        })
        .collect::<Vec<_>>();
    if messages.is_empty() {
        return Err(GatewayCommandError::Internal(
            "runner.startRoutedProvider requires at least one message".into()
        ));
    }
    Ok(messages)
}

fn build_cloud_agent_auto_review_hook(
    auth: Arc<HostAuthExtension>,
    auto_review: Arc<HostAutoReviewExtension>,
    controller: Arc<mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewController>,
    agent_id: String,
    request_source: String,
    cancellation: RoutedProviderCancellation,
    conversation_context: Vec<ProviderMessage>,
) -> CloudAgentReviewHook {
    Arc::new(move |args, images, tool_call_id| {
        let Some(object) = args.as_object() else {
            return Ok(None);
        };
        let action = object
            .get("action")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .unwrap_or_default();
        if action.is_empty() {
            return Ok(None);
        }

        let mode = auto_review.current_modes().cloud_agent;
        let cancelled = || cancellation.is_cancelled();
        if matches!(action, "launch" | "reply") {
            let prompt = object
                .get("prompt")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let review_images = images
                .iter()
                .map(|image| CloudAgentReviewImage {
                    name: Path::new(&image.path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(&image.path)
                        .to_string(),
                    data: image.data.clone(),
                })
                .collect::<Vec<_>>();
            let Some(target) = build_sand_cloud_agent_review_target(
                action,
                prompt,
                object.get("agent_id").and_then(serde_json::Value::as_str),
                &review_images,
                object
                    .get("interrupt")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                object.get("repo_url").and_then(serde_json::Value::as_str),
                object.get("title").and_then(serde_json::Value::as_str),
            ) else {
                return Ok(None);
            };
            let classifier_cancellation = cancellation.clone();
            let mut classifier =
                create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                    Arc::clone(&auth),
                    Arc::new(move || classifier_cancellation.is_cancelled()),
                )
                .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
            let classifier_context = conversation_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let outcome = review_sand_cloud_agent_action(
                mode,
                &target,
                Some(controller.as_ref()),
                &request_source,
                cancelled,
                |risk_target, classifier_mode| {
                    run_sand_auto_review_classifier(
                        &mut classifier,
                        tool_call_id,
                        &agent_id,
                        classifier_mode,
                        || risk_target.clone(),
                        || Ok(classifier_context.clone()),
                        &[],
                        SAND_CLOUD_AGENT_CLASSIFIER_ERROR_REASON,
                    )
                },
            )
            .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
            return Ok(match outcome {
                CloudAgentReviewOutcome::Allowed => None,
                CloudAgentReviewOutcome::Blocked(reason) => Some(reason),
                CloudAgentReviewOutcome::Cancelled => {
                    Some("The cloud agent action was cancelled.".into())
                }
            });
        }

        let lifecycle = match action {
            "rename" => Some(CloudLifecycleAction::Rename),
            "cancel" => Some(CloudLifecycleAction::Cancel),
            "archive" => Some(CloudLifecycleAction::Archive),
            "unarchive" => Some(CloudLifecycleAction::Unarchive),
            "delete" => Some(CloudLifecycleAction::Delete),
            _ => None,
        };
        let Some(lifecycle) = lifecycle else {
            return Ok(None);
        };
        let agent = object
            .get("agent_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let Some(target) = build_sand_cloud_agent_lifecycle_review_target(
            action,
            agent,
            object.get("title").and_then(serde_json::Value::as_str),
        ) else {
            return Ok(None);
        };
        let outcome = review_sand_cloud_agent_lifecycle_action(
            mode,
            lifecycle,
            &target,
            Some(controller.as_ref()),
            &request_source,
            cancelled,
        )
        .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
        Ok(match outcome {
            CloudAgentReviewOutcome::Allowed => None,
            CloudAgentReviewOutcome::Blocked(reason) => Some(reason),
            CloudAgentReviewOutcome::Cancelled => {
                Some("The cloud agent action was cancelled.".into())
            }
        })
    })
}

fn start_routed_provider_task(
    routed_tool_relay: Arc<CoordinatorToolRelay>,
    mcp_service: Arc<McpHostService>,
    events: GatewayEventHub,
    host_tx: mpsc::Sender<HostLaneRequest>,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    auth: Arc<HostAuthExtension>,
    auto_review: Arc<HostAutoReviewExtension>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
    inference: Arc<ProductionInferenceExtension>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    generated_agent_runtime: Arc<Mutex<SubagentRuntime>>,
    completion_revivals: Arc<CompletionRevivals>,
    forever_box: Arc<ForeverBoxService>,
    local_exec: Arc<HostLocalExecExtension>,
    local_tool_permission: Arc<HostLocalToolPermissionExtension>,
    session_handoff: BoxHandoffService,
    trays: Arc<HostTraysExtension>,
    telemetry_logs: HostStructuredLogTelemetry,
    production_action_auditor: ActionAuditExtension,
    cloud_agents: Arc<SandCloudAgentManager>,
    cloud_agent_watches: Arc<RunnerCloudAgentWatches>,
    host_runner_composition: Arc<HostRunnerComposition>,
    box_store_sync: ProductionBoxStoreSyncApi,
    automations_lifecycle: Arc<Mutex<Weak<ProductionAutomationsLifecycle>>>,
    gateway_context: Option<GatewayCommandContext>,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    let provider_name = args.get("provider").and_then(serde_json::Value::as_str).unwrap_or("");
    let requested_provider = RoutedProvider::parse(provider_name)
        .ok_or_else(|| GatewayCommandError::Internal(format!(
            "unsupported routed provider: {provider_name}"
        )))?;
    let provider = authorize_routed_provider_request(
        inference.route(),
        requested_provider,
    )
    .map_err(GatewayCommandError::Internal)?;
    let agent_id = args.get("agentId").and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires agentId".into()
        ))?
        .to_string();
    let stream_id = args.get("streamId").and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires streamId".into()
        ))?
        .to_string();
    let request_source = args
        .get("requestSource")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let generated_parent_agent_id = args
        .get("parentAgentId")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let generated_subagent_type = args
        .get("subagentType")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("general-purpose")
        .to_string();
    let generated_tool_call_id = args
        .get("parentAgentToolCallId")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&stream_id)
        .to_string();
    let is_ack_redrive = args
        .get("ackRedrive")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let is_upgrade_resume = args
        .get("upgradeResume")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let turn_hidden = args
        .get("hidden")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    transcript_runtime
        .require_routed_turn_lease(&agent_id, &stream_id)
        .map_err(map_production_send_error)?;
    let routed_turn_lease_guard = RoutedTurnLeaseGuard::new(
        Arc::clone(&transcript_runtime),
        agent_id.clone(),
        stream_id.clone(),
        events.clone(),
    );
    let lifecycle_messages = decode_provider_messages(&args)?;
    let turn_input = create_production_turn_input_projection(
        &args,
        &stream_id,
        &lifecycle_messages,
    )
    .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
    let mut provider_messages =
        project_provider_messages_for_turn(&args, &lifecycle_messages).messages;
    if let Some(prepared_session) = session_workers
        .prepare_existing_agent(&agent_id)
        .map_err(|error| {
            GatewayCommandError::Internal(format!(
                "could not prepare production session worker state for {agent_id}: {error}"
            ))
        })?
    {
        session_workers
            .ensure_capacity_for_turn(&prepared_session)
            .map_err(|error| {
                GatewayCommandError::Internal(format!(
                    "conversation capacity gate rejected {agent_id}: {error}"
                ))
            })?;
    }

    let memory_store = session_workers.memory_service().store_for_agent(&agent_id);
    let memory_recall = memory_store.recall(MEMORY_RECENT_PROMPT_LIMIT);
    let memory_location = memory_store.get_location().to_string_lossy().into_owned();
    append_memory_system_prompt(
        &mut provider_messages,
        &memory_recall,
        Some(&memory_location),
    );
    let automation_store = session_workers
        .open_automation_store(&agent_id)
        .map_err(|error| GatewayCommandError::Internal(format!(
            "could not open production automation store for {agent_id}: {error}"
        )))?;
    let automation_location = automation_store.get_location().to_string_lossy().into_owned();
    let automation_time_zone = automation_store.resolved_user_time_zone();
    let automation_definitions = automation_store.list_definitions();
    append_automations_system_prompt(
        &mut provider_messages,
        &automation_definitions,
        Some(&automation_location),
        automation_time_zone.as_deref(),
    );
    let firing_automation_id = args
        .get("automationWake")
        .and_then(serde_json::Value::as_object)
        .and_then(|wake| wake.get("id"))
        .and_then(serde_json::Value::as_str);
    if let Some(reminder) =
        create_automation_status_reminder(&automation_store, firing_automation_id)
    {
        provider_messages.push(ProviderMessage {
            role: "system".into(),
            content: reminder,
        });
    }
    let spotlight_enabled = experiments.check_feature_gate("sand_spotlight");
    let multitask_enabled = !args
        .get("groupMemberTurn")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        && experiments.check_feature_gate("sand_multitask");
    if spotlight_enabled {
        provider_messages.push(ProviderMessage {
            role: "system".into(),
            content: spotlight_prompt_section(true),
        });
    }

    let agent_store = session_workers
        .open_agent_store_owner(&agent_id)
        .map_err(|error| GatewayCommandError::Internal(format!(
            "could not open production AgentStore for {agent_id}: {error}"
        )))?;
    let blob_store = Arc::new(
        session_workers
            .create_agent_blob_store(&agent_id)
            .map_err(|error| GatewayCommandError::Internal(format!(
                "could not open production Agent blob store for {agent_id}: {error}"
            )))?,
    );
    let prior_state_bytes = agent_store.latest_checkpoint_bytes().unwrap_or_default();
    let transcript_provider = ProductionTranscriptMirrorProvider::new(
        data_dir.join("transcripts"),
        GeneratedTranscriptOccurrenceCodec::new(
            RejectGeneratedToolJsonProjection,
        ),
    );
    let journal_experiments = Arc::clone(&experiments);
    let transcript_mirror = Arc::new(
        transcript_provider
            .route_for_session(
                Arc::clone(&blob_store),
                &prior_state_bytes,
                Arc::new(move || Ok(
                    journal_experiments
                        .check_feature_gate("sand_new_transcript_journal")
                )),
            )
            .map_err(|error| GatewayCommandError::Internal(format!(
                "could not create production transcript mirror for {agent_id}: {error}"
            )))?,
    );
    let agent_state_checkpoint_sink: Arc<dyn AgentStateCheckpointSink> = Arc::new(
        ProductionAgentStateCheckpointSink::new(
            agent_id.clone(),
            agent_store,
            blob_store,
            transcript_mirror,
            prior_state_bytes,
            true,
        )
        .map_err(|error| GatewayCommandError::Internal(format!(
            "could not initialize production Agent checkpoint sink for {agent_id}: {error}"
        )))?,
    );

    let is_group_member_turn = args
        .get("groupMemberTurn")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    transcript_runtime.begin_provider_run_with_kind(&agent_id, is_group_member_turn);
    let supplied_ack_token = args
        .get("ackToken")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let ack_token = if let Some(ack_token) = supplied_ack_token {
        if !ack_obligations
            .is_ack_run_token_for_agent(&agent_id, ack_token)
            .map_err(|error| GatewayCommandError::Internal(error.to_string()))?
        {
            transcript_runtime.end_provider_run_with_kind(&agent_id, is_group_member_turn);
            let _ = transcript_runtime.retire_idle_live_session(
                &session_workers,
                &agent_id,
            );
            return Err(GatewayCommandError::BadRequest(
                "runner ackToken is not owned by this agent".into(),
            ));
        }
        Some(ack_token.to_string())
    } else {
        ack_obligations
            .mint_ack_run_token(&agent_id)
            .map_err(|error| {
                transcript_runtime.end_provider_run_with_kind(&agent_id, is_group_member_turn);
                let _ = transcript_runtime.retire_idle_live_session(
                    &session_workers,
                    &agent_id,
                );
                GatewayCommandError::Internal(format!(
                    "could not mint ack run token for {agent_id}: {error}"
                ))
            })?
    };
    let cancellation = runner_registry
        .register_routed_provider(&agent_id, &stream_id)
        .map_err(|error| {
            ack_obligations.retire_ack_run_token(&agent_id, ack_token.as_deref());
            transcript_runtime.end_provider_run_with_kind(&agent_id, is_group_member_turn);
            let _ = transcript_runtime.retire_idle_live_session(
                &session_workers,
                &agent_id,
            );
            GatewayCommandError::Internal(error.to_string())
        })?;
    if !is_group_member_turn {
        host_runner_composition.bind_local_permission_surface(&agent_id);
    }
    let approvals_resolvable = !matches!(
        request_source.as_deref(),
        Some("group-member" | "automation")
    );
    let auto_review_controller = auto_review.bind_runner(&agent_id, approvals_resolvable);
    let auto_review_service = auto_review.service();
    let auto_review_request_source = request_source
        .clone()
        .unwrap_or_else(|| "turn".to_string());
    let auto_review_context = lifecycle_messages.clone();
    let checkpoint_store = Arc::new(
        ProductionRoutedProviderCheckpointStore::new(
            &data_dir,
            &agent_id,
            &stream_id,
        ),
    );
    let resolved_request_context = request_context.resolve();
    let worker_events = events.clone();
    let accepted_stream_id = stream_id.clone();
    let worker_stream_id = stream_id.clone();
    let worker_registry = Arc::clone(&runner_registry);
    let worker_mcp_service = Arc::clone(&mcp_service);
    let worker_host_runner_composition = Arc::clone(&host_runner_composition);
    let worker_cancellation = cancellation.clone();
    let worker_sessions = Arc::clone(&session_workers);
    let worker_retire_sessions = Arc::clone(&session_workers);
    let worker_ack_obligations = Arc::clone(&ack_obligations);
    let worker_transcript_runtime = Arc::clone(&transcript_runtime);
    let worker_ack_token = ack_token.clone();
    let worker_trays = Arc::clone(&trays);
    let worker_telemetry_logs = telemetry_logs.clone();
    let worker_request_source = request_source.clone();
    let worker_session_options = SandSessionOptions {
        model_id: args
            .get("modelId")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        is_summarization_session: args
            .get("isSummarizationSession")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        is_computer_use_subagent: args
            .get("isComputerUseSubagent")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        is_browser_use_subagent: args
            .get("isBrowserUseSubagent")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        request_source: request_source.clone(),
        conversation_id: Some(agent_id.clone()),
        inference_reason: match args
            .get("inferenceReason")
            .and_then(serde_json::Value::as_i64)
        {
            Some(1) => Some(InferenceReason::GeminiVideoSubagent),
            _ => None,
        },
        lineage: args
            .get("lineage")
            .and_then(serde_json::Value::as_object)
            .and_then(|lineage| {
                let parent_request_id = lineage
                    .get("parentRequestId")?
                    .as_str()?
                    .trim()
                    .to_string();
                let root_parent_request_id = lineage
                    .get("rootParentRequestId")?
                    .as_str()?
                    .trim()
                    .to_string();
                if parent_request_id.is_empty() || root_parent_request_id.is_empty() {
                    return None;
                }
                Some(RequestLineage {
                    parent_request_id,
                    root_parent_request_id,
                    parent_agent_tool_call_id: lineage
                        .get("parentAgentToolCallId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned),
                })
            }),
        skip_labeling: args
            .get("skipLabeling")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    };
    let worker_generated_parent_agent_id = generated_parent_agent_id.clone();
    let worker_generated_subagent_type = generated_subagent_type.clone();
    let worker_generated_tool_call_id = generated_tool_call_id.clone();
    let worker_generated_lineage = worker_session_options.lineage.as_ref().map(|lineage| {
        GeneratedSubagentLineage {
            parent_request_id: Some(lineage.parent_request_id.clone()),
            root_parent_request_id: Some(lineage.root_parent_request_id.clone()),
            parent_agent_tool_call_id: lineage.parent_agent_tool_call_id.clone(),
        }
    });
    let worker_subagent_task_sink: Option<Arc<dyn SubagentTaskSink>> =
        if generated_parent_agent_id.is_none() {
            let root_parent_request_id = worker_session_options
                .lineage
                .as_ref()
                .map(|lineage| lineage.root_parent_request_id.clone())
                .unwrap_or_else(|| stream_id.clone());
            Some(Arc::new(ProductionSubagentTaskSink {
                deps: LocalRoutedRunnerDeps {
                    routed_tool_relay: Arc::clone(&routed_tool_relay),
                    mcp_service: Arc::clone(&mcp_service),
                    auth: Arc::clone(&auth),
                    auto_review: Arc::clone(&auto_review),
                    events: events.clone(),
                    host_tx: host_tx.clone(),
                    data_dir: data_dir.clone(),
                    request_context: Arc::clone(&request_context),
                    experiments: Arc::clone(&experiments),
                    settings: Arc::clone(&settings),
                    inference: Arc::clone(&inference),
                    session_workers: Arc::clone(&session_workers),
                    runner_registry: Arc::clone(&runner_registry),
                    ack_obligations: Arc::clone(&ack_obligations),
                    transcript_runtime: Arc::clone(&transcript_runtime),
                    generated_agent_runtime: Arc::clone(&generated_agent_runtime),
                    completion_revivals: Arc::clone(&completion_revivals),
                    forever_box: Arc::clone(&forever_box),
                    local_exec: Arc::clone(&local_exec),
                    local_tool_permission: Arc::clone(&local_tool_permission),
                    session_handoff: session_handoff.clone(),
                    trays: Arc::clone(&trays),
                    telemetry_logs: telemetry_logs.clone(),
                    production_action_auditor: production_action_auditor.clone(),
                    cloud_agents: Arc::clone(&cloud_agents),
                    cloud_agent_watches: Arc::clone(&cloud_agent_watches),
                    host_runner_composition: Arc::clone(&host_runner_composition),
                    box_store_sync: box_store_sync.clone(),
                    automations_lifecycle: Arc::clone(&automations_lifecycle),
                },
                parent_agent_id: agent_id.clone(),
                provider,
                parent_stream_id: stream_id.clone(),
                root_parent_request_id,
            }))
        } else {
            None
        };
    let worker_subagent_management_runtime = generated_parent_agent_id
        .is_none()
        .then(|| Arc::clone(&generated_agent_runtime));
    let worker_is_ack_redrive = is_ack_redrive;
    let worker_is_upgrade_resume = is_upgrade_resume;
    let worker_memory_store = memory_store.clone();
    let worker_turn_hidden = turn_hidden;
    let worker_gateway_context = gateway_context;
    let state_sand_root = session_workers
        .memory_service()
        .agents_root_dir()
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| GatewayCommandError::Internal(
            "production memory agents root has no sand root parent".into()
        ))?;
    let state_writer: Arc<dyn SandStateWriter> = Arc::new(
        SandAgentState::new(state_sand_root, agent_id.clone())
            .map_err(|error| GatewayCommandError::Internal(format!(
                "could not create production agent-state owner for {agent_id}: {error}"
            )))?,
    );
    let multitask_todo_state: Option<Arc<dyn MultitaskTodoState>> = if multitask_enabled {
        Some(
            session_workers
                .open_agent_db_owner(&agent_id)
                .map_err(|error| GatewayCommandError::Internal(format!(
                    "could not open production multitask todo state for {agent_id}: {error}"
                )))?,
        )
    } else {
        None
    };
    let cloud_agent_dir = session_workers
        .session_db_path(&agent_id)
        .map_err(GatewayCommandError::Internal)?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| GatewayCommandError::Internal(
            format!("agent database path has no parent for {agent_id}")
        ))?;
    let worker_cloud_agents = Arc::clone(&cloud_agents);
    let worker_cloud_agent_watches = Arc::clone(&cloud_agent_watches);
    let worker_box_store_sync = box_store_sync.clone();
    let worker_generated_agent_runtime = Arc::clone(&generated_agent_runtime);
    let worker_completion_revivals = Arc::clone(&completion_revivals);
    let cloud_agent_quiet_origin = args
        .get("quietOrigin")
        .filter(|origin| origin.is_object())
        .cloned()
        .or_else(|| {
            args.get("automationWake")
                .and_then(serde_json::Value::as_object)
                .and_then(|wake| {
                    let id = wake.get("id").and_then(serde_json::Value::as_str)?;
                    let name = wake.get("name").and_then(serde_json::Value::as_str)?;
                    Some(serde_json::json!({
                        "automation": {
                            "id": id,
                            "name": name,
                        }
                    }))
                })
        })
        .or_else(|| {
            args.get("isSilenceAllowed")
                .and_then(serde_json::Value::as_bool)
                .filter(|allowed| *allowed)
                .map(|_| serde_json::json!({}))
        });
    let worker_auth = Arc::clone(&auth);
    let worker_auto_review = Arc::clone(&auto_review);
    let worker_auto_review_controller = Arc::clone(&auto_review_controller);
    let spawn_error_auto_review = Arc::clone(&auto_review_service);
    let spawn_error_agent_id = agent_id.clone();
    let spawn = thread::Builder::new()
        .name(format!("mahayana-runner-provider-{agent_id}"))
        .spawn(move || {
            let mut worker_routed_turn_lease = routed_turn_lease_guard;
            let observation_events = worker_events.clone();
            let observation: TurnObservationHandle = TurnObservation::shared(
                agent_id.clone(),
                Some(Arc::new(move |event| {
                    observation_events.publish(serde_json::json!({
                        "channel": "runner-turn-observation",
                        "payload": event,
                    }));
                })),
            );
            let await_telemetry_logs = worker_telemetry_logs.clone();
            let await_conversation_id = agent_id.clone();
            let ttft_telemetry_logs = worker_telemetry_logs.clone();
            let ttft_conversation_id = agent_id.clone();
            let ttft_trace_id = worker_gateway_context
                .as_ref()
                .and_then(|context| context.trace_id.clone())
                .unwrap_or_default();
            let ttft_span_id = worker_gateway_context
                .as_ref()
                .and_then(|context| context.span_id.clone())
                .unwrap_or_default();
            let ttft_dispatch_started = worker_gateway_context
                .as_ref()
                .map(|context| context.dispatch_started);
            let runner_started_at_ms = started_at_ms();
            if let Ok(mut observation) = observation.lock() {
                observation.set_first_token_handler(Arc::new(move |event| {
                    let Some(chunk_type) = event
                        .get("chunkType")
                        .and_then(serde_json::Value::as_str)
                    else {
                        return;
                    };
                    let projection = ttft_telemetry(&TtftFields {
                        conversation_id: ttft_conversation_id.clone(),
                        ttft_ms: event.get("ttftMs").and_then(serde_json::Value::as_f64),
                        skew: event
                            .get("skew")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                        skew_reason: event
                            .get("skewReason")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        chunk_type: chunk_type.to_string(),
                        is_fork: event
                            .get("isFork")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                        model_id: event
                            .get("modelId")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        trace_id: ttft_trace_id.clone(),
                        span_id: ttft_span_id.clone(),
                    });
                    if let Err(error) = ttft_telemetry_logs.report_projection(&projection) {
                        eprintln!(
                            "mahayana-host ttft_telemetry_failed agent={} error={error}",
                            ttft_conversation_id
                        );
                    }
                }));
                observation.set_turn_await_handler(Arc::new(move |event| {
                    let Some(await_index) = event
                        .get("awaitIndex")
                        .and_then(serde_json::Value::as_u64)
                    else {
                        return;
                    };
                    let Some(block_until_ms) = event
                        .get("blockUntilMs")
                        .and_then(serde_json::Value::as_u64)
                    else {
                        return;
                    };
                    let Some(outcome) = event
                        .get("outcome")
                        .and_then(serde_json::Value::as_str)
                    else {
                        return;
                    };
                    let projection = turn_await_telemetry(&TurnAwaitFields {
                        conversation_id: await_conversation_id.clone(),
                        block_until_ms,
                        outcome: outcome.to_string(),
                        await_index,
                    });
                    if let Err(error) = await_telemetry_logs.report_projection(&projection) {
                        eprintln!(
                            "mahayana-host turn_await_telemetry_failed agent={} error={error}",
                            await_conversation_id
                        );
                    }
                }));
                observation.turn_started(runner_started_at_ms);
            }
            let auto_review_gate = Arc::new(AutoReviewGate::new(Arc::new(
                ProductionAutoReviewGateDeps {
                    auto_review: Arc::clone(&worker_auto_review),
                    controller: Arc::clone(&worker_auto_review_controller),
                    box_id: agent_id.clone(),
                },
            )));
            let _ = auto_review_gate.current_modes();

            let mcp_review_gate = Arc::clone(&auto_review_gate);
            let mcp_review_auth = Arc::clone(&worker_auth);
            let mcp_review_controller = Arc::clone(&worker_auto_review_controller);
            let mcp_review_cancellation = worker_cancellation.clone();
            let mcp_review_agent_id = agent_id.clone();
            let mcp_review_request_source = auto_review_request_source.clone();
            let mcp_review_context = auto_review_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let mcp_review: RoutedMcpAutoReviewCallback = Arc::new(
                move |tool: &RoutedToolDefinition,
                      args: &serde_json::Value,
                      tool_call_id: &str| {
                    mcp_review_gate.assert_no_pending_approval().map_err(|error| {
                        ProviderSessionError::Tool(error.to_string())
                    })?;
                    let mode = mcp_review_gate.current_modes().mcp;
                    if mode == SandAutoReviewMode::Off {
                        return Ok(None);
                    }
                    let instructions = mcp_review_gate.user_instructions();
                    let risk_target = serde_json::json!({
                        "action": "mcp",
                        "arguments": {
                            "server_display_name": tool.provider_identifier,
                            "tool_name": if tool.tool_name.trim().is_empty() { &tool.name } else { &tool.tool_name },
                            "mcp_arguments": args,
                            "allow_instructions": instructions.as_ref().map(|value| &value.allow_instructions),
                            "block_instructions": instructions.as_ref().map(|value| &value.block_instructions),
                        }
                    });
                    let classifier_cancellation = mcp_review_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&mcp_review_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let classifier_mode = if mode == SandAutoReviewMode::Shadow {
                        "shadow"
                    } else {
                        "enforce"
                    };
                    let decision = run_sand_auto_review_classifier(
                        &mut classifier,
                        tool_call_id,
                        &mcp_review_agent_id,
                        classifier_mode,
                        || risk_target.clone(),
                        || Ok(mcp_review_context.clone()),
                        &[],
                        "Auto-review could not classify this MCP action.",
                    )
                    .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
                    if mode == SandAutoReviewMode::Shadow {
                        return Ok(None);
                    }
                    let (reason, proposed_rule) = match decision {
                        AutoReviewClassifierDecision::Allow => return Ok(None),
                        AutoReviewClassifierDecision::Reject { reason } => {
                            return Ok(Some(reason));
                        }
                        AutoReviewClassifierDecision::Block {
                            reason,
                            proposed_rule,
                        } => (reason, proposed_rule),
                    };
                    let decision = request_sand_mcp_approval(
                        mcp_review_controller.as_ref(),
                        &McpApprovalRequest {
                            fingerprint: fingerprint_sand_auto_review_target(&risk_target),
                            reason: reason.clone(),
                            server_display_name: tool.provider_identifier.clone(),
                            tool_name: if tool.tool_name.trim().is_empty() {
                                tool.name.clone()
                            } else {
                                tool.tool_name.clone()
                            },
                            mcp_arguments: Some(args.clone()),
                            description: tool.description.clone(),
                            proposed_rule,
                            expiry_policy: sand_auto_review_approval_expiry_policy(
                                &mcp_review_request_source,
                            ),
                        },
                    )
                    .map_err(ProviderSessionError::Tool)?;
                    Ok(match decision {
                        SandAutoReviewDecision::Approved => None,
                        SandAutoReviewDecision::Denied { reason: denied } => Some(
                            if denied.trim().is_empty() { reason } else { denied },
                        ),
                    })
                },
            );
            let bridge: Arc<dyn RoutedToolBridge> = Arc::new(CoordinatorRoutedToolBridge {
                relay: routed_tool_relay,
                transcript_runtime: Arc::clone(&worker_transcript_runtime),
                agent_id: agent_id.clone(),
                mcp_review: Some(mcp_review),
            });
            let box_resources: Arc<dyn RunnerBoxResourcePort> =
                Arc::new(ForeverBoxRunnerResourcePort::new(
                    Arc::clone(&forever_box),
                    agent_id.clone(),
                ));
            let box_shell_gate = Arc::clone(&auto_review_gate);
            let box_shell_auth = Arc::clone(&worker_auth);
            let box_shell_controller = Arc::clone(&worker_auto_review_controller);
            let box_shell_cancellation = worker_cancellation.clone();
            let box_shell_agent_id = agent_id.clone();
            let box_shell_request_source = auto_review_request_source.clone();
            let box_shell_context = auto_review_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let box_shell_review: BoxShellAutoReviewCallback = Arc::new(
                move |request: &RunnerBoxShellRequest| {
                    box_shell_gate.assert_no_pending_approval().map_err(|error| {
                        ProviderSessionError::Tool(error.to_string())
                    })?;
                    let mode = box_shell_gate.current_modes().box_shell;
                    let approval_identity =
                        box_shell_gate.shell_approval_identity(ShellApprovalSurface::BoxShell);
                    if mode == SandAutoReviewMode::Off {
                        box_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::BoxShell);
                        return Ok(None);
                    }
                    let instructions = box_shell_gate.user_instructions();
                    let risk_target = serde_json::json!({
                        "action": "shell",
                        "arguments": {
                            "command": request.command,
                            "working_directory": request.working_directory,
                            "surface": "isolated_box",
                            "approval_identity": approval_identity,
                            "allow_instructions": instructions.as_ref().map(|value| &value.allow_instructions),
                            "block_instructions": instructions.as_ref().map(|value| &value.block_instructions),
                        }
                    });
                    let classifier_cancellation = box_shell_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&box_shell_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let classifier_mode = if mode == SandAutoReviewMode::Shadow {
                        "shadow"
                    } else {
                        "enforce"
                    };
                    let decision = run_sand_auto_review_classifier(
                        &mut classifier,
                        &request.tool_call_id,
                        &box_shell_agent_id,
                        classifier_mode,
                        || risk_target.clone(),
                        || Ok(box_shell_context.clone()),
                        &[],
                        "Auto-review could not classify this shell command.",
                    )
                    .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
                    if mode == SandAutoReviewMode::Shadow {
                        box_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::BoxShell);
                        return Ok(None);
                    }
                    let (reason, proposed_rule) = match decision {
                        AutoReviewClassifierDecision::Allow => {
                            box_shell_gate.mark_shell_side_effect_start(
                                ShellApprovalSurface::BoxShell,
                            );
                            return Ok(None);
                        }
                        AutoReviewClassifierDecision::Reject { reason } => {
                            return Ok(Some(reason));
                        }
                        AutoReviewClassifierDecision::Block {
                            reason,
                            proposed_rule,
                        } => (reason, proposed_rule),
                    };
                    let identity_before_approval = approval_identity.clone();
                    let recheck_gate = Arc::clone(&box_shell_gate);
                    let decision = request_sand_shell_approval(
                        box_shell_controller.as_ref(),
                        &ShellApprovalRequest {
                            target: ShellApprovalTarget {
                                surface: mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewSurface::BoxShell,
                                description: Some("Run a shell command in the isolated box".into()),
                                working_directory: Some(request.working_directory.clone()),
                            },
                            fingerprint: fingerprint_sand_auto_review_target(&risk_target),
                            reason: reason.clone(),
                            command: request.command.clone(),
                            proposed_rule,
                            expiry_policy: sand_auto_review_approval_expiry_policy(
                                &box_shell_request_source,
                            ),
                        },
                        |_| Ok(()),
                        move |_| {
                            let current = recheck_gate
                                .shell_approval_identity(ShellApprovalSurface::BoxShell);
                            if current == identity_before_approval {
                                Ok(())
                            } else {
                                Err("Shell approval context changed while awaiting review".into())
                            }
                        },
                    )
                    .map_err(ProviderSessionError::Tool)?;
                    Ok(match decision {
                        SandAutoReviewDecision::Approved => {
                            box_shell_gate.mark_shell_side_effect_start(
                                ShellApprovalSurface::BoxShell,
                            );
                            None
                        }
                        SandAutoReviewDecision::Denied { reason: denied } => Some(
                            if denied.trim().is_empty() { reason } else { denied },
                        ),
                    })
                },
            );

            let external_shell_gate = Arc::clone(&auto_review_gate);
            let external_shell_auth = Arc::clone(&worker_auth);
            let external_shell_controller = Arc::clone(&worker_auto_review_controller);
            let external_shell_cancellation = worker_cancellation.clone();
            let external_shell_agent_id = agent_id.clone();
            let external_shell_request_source = auto_review_request_source.clone();
            let external_shell_context = auto_review_context
                .iter()
                .map(|message| serde_json::json!({"role": message.role, "content": message.content}))
                .collect::<Vec<_>>();
            let external_shell_review: ExternalShellAutoReviewCallback = Arc::new(
                move |request: &ExternalMachineShellArgs| {
                    external_shell_gate.assert_no_pending_approval()
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let mode = external_shell_gate.current_modes().host_shell;
                    let approval_identity =
                        external_shell_gate.shell_approval_identity(ShellApprovalSurface::HostShell);
                    if mode == SandAutoReviewMode::Off {
                        external_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::HostShell);
                        return Ok(None);
                    }
                    let instructions = external_shell_gate.user_instructions();
                    let risk_target = serde_json::json!({
                        "action": "shell",
                        "arguments": {
                            "command": request.command,
                            "working_directory": request.working_directory,
                            "surface": "host_machine",
                            "approval_identity": approval_identity,
                            "allow_instructions": instructions.as_ref().map(|value| &value.allow_instructions),
                            "block_instructions": instructions.as_ref().map(|value| &value.block_instructions),
                        }
                    });
                    let classifier_cancellation = external_shell_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&external_shell_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let classifier_mode = if mode == SandAutoReviewMode::Shadow { "shadow" } else { "enforce" };
                    let decision = run_sand_auto_review_classifier(
                        &mut classifier,
                        &request.tool_call_id,
                        &external_shell_agent_id,
                        classifier_mode,
                        || risk_target.clone(),
                        || Ok(external_shell_context.clone()),
                        &[],
                        "Auto-review could not classify this shell command.",
                    )
                    .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
                    if mode == SandAutoReviewMode::Shadow {
                        external_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::HostShell);
                        return Ok(None);
                    }
                    let (reason, proposed_rule) = match decision {
                        AutoReviewClassifierDecision::Allow => {
                            external_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::HostShell);
                            return Ok(None);
                        }
                        AutoReviewClassifierDecision::Reject { reason } => return Ok(Some(reason)),
                        AutoReviewClassifierDecision::Block { reason, proposed_rule } => (reason, proposed_rule),
                    };
                    let identity_before_approval = approval_identity.clone();
                    let recheck_gate = Arc::clone(&external_shell_gate);
                    let decision = request_sand_shell_approval(
                        external_shell_controller.as_ref(),
                        &ShellApprovalRequest {
                            target: ShellApprovalTarget {
                                surface: mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewSurface::HostShell,
                                description: Some("Run a shell command on the user's computer".into()),
                                working_directory: Some(request.working_directory.clone()),
                            },
                            fingerprint: fingerprint_sand_auto_review_target(&risk_target),
                            reason: reason.clone(),
                            command: request.command.clone(),
                            proposed_rule,
                            expiry_policy: sand_auto_review_approval_expiry_policy(&external_shell_request_source),
                        },
                        |_| Ok(()),
                        move |_| {
                            let current = recheck_gate.shell_approval_identity(ShellApprovalSurface::HostShell);
                            if current == identity_before_approval {
                                Ok(())
                            } else {
                                Err("Shell approval context changed while awaiting review".into())
                            }
                        },
                    ).map_err(ProviderSessionError::Tool)?;
                    Ok(match decision {
                        SandAutoReviewDecision::Approved => {
                            external_shell_gate.mark_shell_side_effect_start(ShellApprovalSurface::HostShell);
                            None
                        }
                        SandAutoReviewDecision::Denied { reason: denied } => {
                            Some(if denied.trim().is_empty() { reason } else { denied })
                        }
                    })
                },
            );
            let browser_media_sessions = Arc::clone(&worker_sessions);
            let computer_media_sessions = Arc::clone(&worker_sessions);
            let computer_review_box = Arc::clone(&box_resources);
            let computer_review_auth = Arc::clone(&worker_auth);
            let computer_review_auto_review = Arc::clone(&worker_auto_review);
            let computer_review_controller = Arc::clone(&worker_auto_review_controller);
            let computer_review_cancellation = worker_cancellation.clone();
            let computer_review_agent_id = agent_id.clone();
            let computer_review_request_source = auto_review_request_source.clone();
            let computer_review_context = auto_review_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let computer_review_workspace_paths = vec!["/workspace".to_string()];
            let computer_auto_review: ComputerAutoReviewCallback =
                Arc::new(move |args, tool_call_id| {
                    let mode = computer_review_auto_review.current_modes().computer;
                    validate_computer_action(args, Some(mode))
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let initial_display = computer_review_box.browser_window_index()?;
                    let box_identity = BoxIdentity {
                        box_id: std::env::var("SAND_BOX_HOST")
                            .ok()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "127.0.0.1".into()),
                        window_generation: format!(
                            "{}:{}",
                            computer_review_agent_id,
                            initial_display
                        ),
                    };
                    let exact_action = to_exact_action_value(args);
                    let classifier_cancellation = computer_review_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&computer_review_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    run_sand_computer_auto_review_preflight(
                        mode,
                        &exact_action,
                        args.description.as_deref(),
                        &box_identity,
                        &computer_review_agent_id,
                        &computer_review_request_source,
                        Some(computer_review_controller.as_ref()),
                        || {
                            capture_browser_review_state(
                                computer_review_box.as_ref(),
                                initial_display,
                                None,
                                tool_call_id,
                            )
                            .map(|state| state.display_state_identity)
                            .map_err(|error| error.to_string())
                        },
                        |risk_target, classifier_mode| {
                            run_sand_auto_review_classifier(
                                &mut classifier,
                                tool_call_id,
                                &computer_review_agent_id,
                                classifier_mode,
                                || risk_target.clone(),
                                || Ok(computer_review_context.clone()),
                                &computer_review_workspace_paths,
                                SAND_COMPUTER_AUTO_REVIEW_CLASSIFIER_ERROR_REASON,
                            )
                        },
                    )
                    .map_err(|error| ProviderSessionError::Tool(error.to_string()))
                });
            let browser_review_box = Arc::clone(&box_resources);
            let browser_review_auth = Arc::clone(&worker_auth);
            let browser_review_auto_review = Arc::clone(&worker_auto_review);
            let browser_review_controller = Arc::clone(&worker_auto_review_controller);
            let browser_review_cancellation = worker_cancellation.clone();
            let browser_review_agent_id = agent_id.clone();
            let browser_review_request_source = auto_review_request_source.clone();
            let browser_review_context = auto_review_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let browser_auto_review: BrowserAutoReviewCallback =
                Arc::new(move |spec, args, tool_call_id| {
                    let mode = browser_review_auto_review.current_modes().computer;
                    let initial_display = browser_review_box.browser_window_index()?;
                    let exact_action = serde_json::to_value(to_browser_review_action(
                        spec.op,
                        args,
                        &browser_review_agent_id,
                    ))
                    .map_err(|error| {
                        ProviderSessionError::Tool(format!(
                            "Browser Auto-review action projection failed: {error}"
                        ))
                    })?;
                    let view_id = exact_action
                        .get("viewId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string);
                    let box_identity = BoxIdentity {
                        box_id: std::env::var("SAND_BOX_HOST")
                            .ok()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "127.0.0.1".into()),
                        window_generation: format!(
                            "{}:{}",
                            browser_review_agent_id,
                            initial_display
                        ),
                    };
                    let classifier_cancellation = browser_review_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&browser_review_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    run_sand_browser_auto_review_preflight(
                        mode,
                        &exact_action,
                        &box_identity,
                        &browser_review_agent_id,
                        &browser_review_request_source,
                        Some(browser_review_controller.as_ref()),
                        || {
                            let display = browser_review_box
                                .browser_window_index()
                                .map_err(|error| error.to_string())?;
                            capture_browser_review_state(
                                browser_review_box.as_ref(),
                                display,
                                view_id.as_deref(),
                                tool_call_id,
                            )
                            .map_err(|error| error.to_string())
                        },
                        |risk_target, classifier_mode| {
                            run_sand_auto_review_classifier(
                                &mut classifier,
                                tool_call_id,
                                &browser_review_agent_id,
                                classifier_mode,
                                || risk_target.clone(),
                                || Ok(browser_review_context.clone()),
                                &[],
                                SAND_COMPUTER_AUTO_REVIEW_CLASSIFIER_ERROR_REASON,
                            )
                        },
                    )
                    .map_err(|error| ProviderSessionError::Tool(error.to_string()))
                });
            let routine_review_auth = Arc::clone(&worker_auth);
            let routine_review_auto_review = Arc::clone(&worker_auto_review);
            let routine_review_controller = Arc::clone(&worker_auto_review_controller);
            let routine_review_cancellation = worker_cancellation.clone();
            let routine_review_agent_id = agent_id.clone();
            let routine_review_request_source = auto_review_request_source.clone();
            let routine_review_context = auto_review_context
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "role": message.role,
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>();
            let routine_auto_review: RoutineAutoReviewCallback =
                Arc::new(move |target, tool_call_id| {
                    let mode = routine_review_auto_review.current_modes().automation_write;
                    let classifier_cancellation = routine_review_cancellation.clone();
                    let mut classifier =
                        create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                            Arc::clone(&routine_review_auth),
                            Arc::new(move || classifier_cancellation.is_cancelled()),
                        )
                        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                    let outcome = review_sand_automation_write(
                        mode,
                        target,
                        &routine_review_agent_id,
                        Some(routine_review_controller.as_ref()),
                        &routine_review_request_source,
                        |risk_target, classifier_mode| {
                            run_sand_auto_review_classifier(
                                &mut classifier,
                                tool_call_id,
                                &routine_review_agent_id,
                                classifier_mode,
                                || risk_target.clone(),
                                || Ok(routine_review_context.clone()),
                                &[],
                                SAND_AUTOMATION_WRITE_CLASSIFIER_ERROR_REASON,
                            )
                        },
                    )
                    .map_err(|error| {
                        ProviderSessionError::Tool(match error {
                            mahayana_host_runtime::runner::sand_auto_review_classifier_run::AutoReviewClassifierError::Aborted(reason)
                            | mahayana_host_runtime::runner::sand_auto_review_classifier_run::AutoReviewClassifierError::Failed(reason) => reason,
                        })
                    })?;
                    match outcome {
                        AutomationReviewOutcome::Allowed => Ok(()),
                        AutomationReviewOutcome::Blocked(reason) => {
                            Err(ProviderSessionError::Tool(reason))
                        }
                    }
                });
            let cloud_agent_review = build_cloud_agent_auto_review_hook(
                Arc::clone(&worker_auth),
                Arc::clone(&worker_auto_review),
                Arc::clone(&worker_auto_review_controller),
                agent_id.clone(),
                auto_review_request_source.clone(),
                worker_cancellation.clone(),
                auto_review_context.clone(),
            );
            let cloud_watch_parent_agent_id = agent_id.clone();
            let cloud_watch_owner = Arc::clone(&worker_cloud_agent_watches);
            let cloud_watch_quiet_origin = cloud_agent_quiet_origin.clone();
            let cloud_agent_watch = Arc::new(move |bc_id: &str, after_followup: bool| {
                let _ = cloud_watch_owner.watch_cloud_agent(
                    &cloud_watch_parent_agent_id,
                    bc_id,
                    CloudAgentWatchOptions::new(
                        cloud_watch_quiet_origin.clone(),
                        after_followup,
                    ),
                );
            });
            let cloud_agent_tool = CloudAgentToolDependencies {
                manager: Arc::clone(&worker_cloud_agents),
                agent_dir: cloud_agent_dir,
                box_resources: Arc::clone(&box_resources),
                cancellation: worker_cancellation.clone(),
                review: Some(cloud_agent_review),
                watch: Some(cloud_agent_watch),
            };
            let subagent_task_review: Option<SubagentTaskReviewCallback> =
                worker_subagent_management_runtime.as_ref().map(|_| {
                    let review_auth = Arc::clone(&worker_auth);
                    let review_auto_review = Arc::clone(&worker_auto_review);
                    let review_controller = Arc::clone(&worker_auto_review_controller);
                    let review_cancellation = worker_cancellation.clone();
                    let review_agent_id = agent_id.clone();
                    let review_request_source = auto_review_request_source.clone();
                    let review_context = auto_review_context
                        .iter()
                        .map(|message| {
                            serde_json::json!({
                                "role": message.role,
                                "content": message.content,
                            })
                        })
                        .collect::<Vec<_>>();
                    Arc::new(move |prompt: &str, subagent_type: &str, tool_call_id: &str| {
                        let Some(target) = build_sand_subagent_launch_review_target(
                            prompt,
                            Some(subagent_type),
                        ) else {
                            return Ok(Some("Task requires a non-empty prompt.".into()));
                        };
                        let mode = review_auto_review.current_modes().subagent_launch;
                        let classifier_cancellation = review_cancellation.clone();
                        let mut classifier =
                            create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                                Arc::clone(&review_auth),
                                Arc::new(move || classifier_cancellation.is_cancelled()),
                            )
                            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                        let outcome = review_sand_subagent_action(
                            mode,
                            &target,
                            Some(review_controller.as_ref()),
                            &review_request_source,
                            || review_cancellation.is_cancelled(),
                            |risk_target, classifier_mode| {
                                run_sand_auto_review_classifier(
                                    &mut classifier,
                                    tool_call_id,
                                    &review_agent_id,
                                    classifier_mode,
                                    || risk_target.clone(),
                                    || Ok(review_context.clone()),
                                    &[],
                                    SAND_SUBAGENT_CLASSIFIER_ERROR_REASON,
                                )
                            },
                        )
                        .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
                        Ok(match outcome {
                            SubagentReviewOutcome::Allowed => None,
                            SubagentReviewOutcome::Blocked(reason) => Some(reason),
                            SubagentReviewOutcome::Cancelled => {
                                Some("The subagent launch was cancelled.".into())
                            }
                        })
                    }) as SubagentTaskReviewCallback
                });
            let subagent_steer_review: Option<SubagentSteerReviewCallback> =
                worker_subagent_management_runtime.as_ref().map(|_| {
                    let review_auth = Arc::clone(&worker_auth);
                    let review_auto_review = Arc::clone(&worker_auto_review);
                    let review_controller = Arc::clone(&worker_auto_review_controller);
                    let review_cancellation = worker_cancellation.clone();
                    let review_agent_id = agent_id.clone();
                    let review_request_source = auto_review_request_source.clone();
                    let review_context = auto_review_context
                        .iter()
                        .map(|message| {
                            serde_json::json!({
                                "role": message.role,
                                "content": message.content,
                            })
                        })
                        .collect::<Vec<_>>();
                    Arc::new(move |subagent_id: &str, message: &str, tool_call_id: &str| {
                        let Some(target) = build_sand_subagent_steer_review_target(
                            subagent_id,
                            message,
                        ) else {
                            return Ok(SteerReview {
                                allowed: false,
                                reason: "MessageSubagent requires a subagent id and message.".into(),
                            });
                        };
                        let mode = review_auto_review.current_modes().subagent_launch;
                        let classifier_cancellation = review_cancellation.clone();
                        let mut classifier =
                            create_sand_backend_smart_mode_classifier_executor_with_cancellation(
                                Arc::clone(&review_auth),
                                Arc::new(move || classifier_cancellation.is_cancelled()),
                            )
                            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
                        let outcome = review_sand_subagent_action(
                            mode,
                            &target,
                            Some(review_controller.as_ref()),
                            &review_request_source,
                            || review_cancellation.is_cancelled(),
                            |risk_target, classifier_mode| {
                                run_sand_auto_review_classifier(
                                    &mut classifier,
                                    tool_call_id,
                                    &review_agent_id,
                                    classifier_mode,
                                    || risk_target.clone(),
                                    || Ok(review_context.clone()),
                                    &[],
                                    SAND_SUBAGENT_CLASSIFIER_ERROR_REASON,
                                )
                            },
                        )
                        .map_err(|error| ProviderSessionError::Tool(format!("{error:?}")))?;
                        Ok(match outcome {
                            SubagentReviewOutcome::Allowed => SteerReview {
                                allowed: true,
                                reason: String::new(),
                            },
                            SubagentReviewOutcome::Blocked(reason) => SteerReview {
                                allowed: false,
                                reason,
                            },
                            SubagentReviewOutcome::Cancelled => SteerReview {
                                allowed: false,
                                reason: "The subagent steering action was cancelled.".into(),
                            },
                        })
                    }) as SubagentSteerReviewCallback
                });
            let delta_events = worker_events.clone();
            let delta_stream_id = stream_id.clone();
            let delta_runtime = Arc::clone(&worker_transcript_runtime);
            let delta_agent_id = agent_id.clone();
            let delta_observation = Arc::clone(&observation);
            let mut on_text_delta = move |delta: &str, accumulated: &str| {
                if !delta.is_empty() {
                    if let Ok(mut observation) = delta_observation.lock() {
                        let observed_perf_ms = ttft_dispatch_started
                            .map(|dispatch_started| dispatch_started.elapsed().as_secs_f64() * 1_000.0);
                        let _ = observation.observe_first_token(
                            "text",
                            observed_perf_ms.map(|_| 0.0),
                            observed_perf_ms,
                            Some(provider.as_str()),
                            turn_input.options.is_fork,
                        );
                    }
                }
                delta_runtime.track_runner_activity_update(
                    &delta_agent_id,
                    &ActivityUpdate::TextDelta {
                        text: delta.to_string(),
                    },
                    started_at_ms(),
                );
                delta_events.publish(serde_json::json!({
                    "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                    "payload": {
                        "streamId": delta_stream_id,
                        "type": "delta",
                        "content": accumulated
                    }
                }));
            };
            let agent_wake_events = worker_events.clone();
            let priority_registry = Arc::clone(&worker_registry);
            let priority_runtime = Arc::clone(&worker_transcript_runtime);
            let agent_messaging = Arc::new(ProductionAgentToAgentMessaging::new(
                Arc::clone(&worker_sessions),
                Arc::new(move |request: &AgentWakeRequest| {
                    agent_wake_events.publish(serde_json::json!({
                        "channel": "agent-inbound-wake-request",
                        "payload": request,
                    }));
                }),
                Some(Arc::new(move |target_agent_id: &str, reason: &str| {
                    if !should_interrupt_priority_peer(
                        priority_runtime.active_turn_lane(target_agent_id),
                    ) {
                        return 0;
                    }
                    priority_registry.cancel_agent(target_agent_id, reason)
                })),
            ));
            let turn_agent_messages = Arc::new(Mutex::new(Vec::<String>::new()));
            let agent_management_sink: Arc<dyn AgentManagementSink> = Arc::new(
                ProductionAgentManagementSink {
                    sessions: Arc::clone(&worker_sessions),
                    forever_box: Arc::clone(&forever_box),
                    messaging: agent_messaging,
                    agent_messages: Arc::clone(&turn_agent_messages),
                    agent_id: agent_id.clone(),
                },
            );
            let send_message_sink: Arc<dyn SendMessageSink> = Arc::new(
                ProductionSendMessageSink {
                    sessions: worker_sessions,
                    forever_box: Arc::clone(&forever_box),
                    session_handoff: session_handoff.clone(),
                    ack_obligations: Arc::clone(&worker_ack_obligations),
                    transcript_runtime: Arc::clone(&worker_transcript_runtime),
                    ack_token: worker_ack_token.clone(),
                    agent_id: agent_id.clone(),
                },
            );
            let routine_post_write: Option<RoutinePostWriteCallback> =
                automations_lifecycle
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .upgrade()
                    .map(|lifecycle| {
                    let sink = Arc::clone(&send_message_sink);
                    let callback_agent_id = agent_id.clone();
                    Arc::new(move |target: &mahayana_host_runtime::runner::sand_automation_auto_review::AutomationWriteTarget, tool_call_id: &str| {
                        if !matches!(target.operation.as_str(), "create" | "update")
                            || !target.spec.is_enabled
                        {
                            return Ok(());
                        }
                        let mut platforms = trigger_members(&target.spec.trigger)
                            .into_iter()
                            .filter_map(|listener| {
                                listener
                                    .get("type")
                                    .and_then(serde_json::Value::as_str)
                                    .filter(|platform| matches!(*platform, "slack" | "github"))
                                    .map(ToOwned::to_owned)
                            })
                            .collect::<Vec<_>>();
                        platforms.sort();
                        platforms.dedup();
                        let surfaced = surface_listener_connect_cards(
                            &platforms,
                            Some(|platform: &str| lifecycle.is_platform_connected(platform)),
                            |platform| match platform {
                                "slack" => Some("Slack".to_string()),
                                "github" => Some("GitHub".to_string()),
                                _ => None,
                            },
                        );
                        for card in surfaced.cards {
                            let platform = card.platform.clone();
                            let card_tool_call_id =
                                format!("{tool_call_id}:listener:{platform}");
                            sink.send_message(
                                serde_json::json!({
                                    "type": card.message_type,
                                    "platform": platform,
                                    "reason": card.reason,
                                }),
                                started_at_ms(),
                                &card_tool_call_id,
                            )?;
                            lifecycle.watch_listener_connection(
                                callback_agent_id.clone(),
                                platform,
                            );
                        }
                        Ok(())
                    }) as RoutinePostWriteCallback
                });
            let reaction_sink: Arc<dyn ReactionSink> = Arc::new(
                ProductionReactionSink {
                    host_tx,
                    agent_id: agent_id.clone(),
                },
            );
            let retry_runtime = Arc::clone(&worker_transcript_runtime);
            let retry_agent_id = agent_id.clone();
            let retry_sink: Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync> =
                Arc::new(move |_event: &ProviderRetryEvent| {
                    retry_runtime.track_runner_activity_update(
                        &retry_agent_id,
                        &ActivityUpdate::Retrying,
                        started_at_ms(),
                    );
                });
            let retry_observation = Arc::clone(&observation);
            let retry_telemetry_logs = worker_telemetry_logs.clone();
            let retry_conversation_id = agent_id.clone();
            let retry_report_sink: Arc<dyn Fn(&ProviderRetryReport) + Send + Sync> =
                Arc::new(move |report: &ProviderRetryReport| {
                    let outcome = match report.outcome {
                        ProviderRetryOutcome::Retried => "retried",
                        ProviderRetryOutcome::Exhausted => "exhausted",
                        ProviderRetryOutcome::GaveUpIneligible => "gave_up_ineligible",
                    };
                    if let Ok(observation) = retry_observation.lock() {
                        observation.report_turn_retry(serde_json::json!({
                            "outcome": outcome,
                            "attempt": report.attempt,
                            "maxAttempts": report.max_attempts,
                            "delayMs": report.delay_ms,
                            "serverPaced": report.server_paced,
                            "resumeFromCheckpoint": report.resume_from_checkpoint,
                            "watchdogExpired": report.watchdog_expired,
                            "error": report.error,
                        }));
                    }
                    let projection = turn_retry_telemetry(&TurnRetryFields {
                        conversation_id: retry_conversation_id.clone(),
                        outcome: outcome.into(),
                        attempt: u64::from(report.attempt),
                        max_attempts: u64::from(report.max_attempts),
                        error_type: report.error_type.clone(),
                        error_code: report.error_code.clone(),
                        cause: report.cause.clone(),
                        delay_ms: report.delay_ms.map(|value| value as f64),
                        server_paced: Some(report.server_paced),
                    });
                    if let Err(error) = retry_telemetry_logs.report_projection(&projection) {
                        eprintln!(
                            "mahayana-host turn_retry_telemetry_failed agent={} error={error}",
                            retry_conversation_id
                        );
                    }
                });
            let audit_events = worker_events.clone();
            let host_action_auditor = Arc::clone(production_action_auditor.service());
            let action_audit_sink: Arc<dyn ActionAuditSink> = Arc::new(
                move |record: ActionAuditRecord| {
                    audit_events.publish(serde_json::json!({
                        "channel": "runner-action-audit",
                        "payload": record,
                    }));
                    match serde_json::from_value::<AuditAction>(record.action.clone()) {
                        Ok(action) => host_action_auditor.record(AuditRecord {
                            occurred_at_ms: record.occurred_at_ms,
                            agent_id: record.agent_id,
                            turn_id: record.turn_id,
                            box_id: None,
                            action,
                        }),
                        Err(error) => eprintln!(
                            "mahayana-host-action-audit invalid runner action: {error}"
                        ),
                    }
                },
            );
            let browser_media_agent_id = agent_id.clone();
            let browser_persist_image: BrowserPersistImageCallback = Arc::new(
                move |bytes, mime| {
                    if mime != "image/png" {
                        return;
                    }
                    let Ok(db_path) =
                        browser_media_sessions.session_db_path(&browser_media_agent_id)
                    else {
                        eprintln!(
                            "mahayana-host browser_screenshot_persist_failed agent={} error=session-db-unavailable",
                            browser_media_agent_id
                        );
                        return;
                    };
                    let Some(agent_dir) = db_path.parent() else {
                        eprintln!(
                            "mahayana-host browser_screenshot_persist_failed agent={} error=agent-dir-unavailable",
                            browser_media_agent_id
                        );
                        return;
                    };
                    if let Err(error) = persist_agent_media_bytes(
                        agent_dir,
                        "browser-screenshot.png",
                        bytes,
                        AgentMediaKind::Image,
                    ) {
                        eprintln!(
                            "mahayana-host browser_screenshot_persist_failed agent={} error={error}",
                            browser_media_agent_id
                        );
                    }
                },
            );
            let browser_navigation_urls =
                Arc::new(Mutex::new(BTreeMap::<String, String>::new()));
            let browser_navigation_urls_sink = Arc::clone(&browser_navigation_urls);
            let browser_navigation_audit = Arc::clone(&action_audit_sink);
            let browser_navigation_agent_id = agent_id.clone();
            let browser_navigation_turn_id = stream_id.clone();
            let browser_possible_navigation: BrowserPossibleNavigationCallback =
                Arc::new(move |response| {
                    let Some(url) = response
                        .url
                        .as_deref()
                        .and_then(normalize_navigation_url)
                    else {
                        return;
                    };
                    let view_id = response
                        .view_id
                        .as_deref()
                        .unwrap_or("default")
                        .to_string();
                    let should_record =
                        if let Ok(mut urls) = browser_navigation_urls_sink.lock() {
                            if urls.get(&view_id).is_some_and(|prior| prior == &url) {
                                false
                            } else {
                                urls.insert(view_id, url.clone());
                                true
                            }
                        } else {
                            false
                        };
                    if !should_record {
                        return;
                    }
                    browser_navigation_audit.record(ActionAuditRecord {
                        occurred_at_ms: started_at_ms(),
                        agent_id: browser_navigation_agent_id.clone(),
                        turn_id: Some(browser_navigation_turn_id.clone()),
                        action: serde_json::json!({
                            "kind": "browserNavigation",
                            "url": url,
                            "pageTitle": response.title.as_deref().unwrap_or_default(),
                        }),
                    });
                });
            let browser_executor: Arc<dyn BrowserToolExecutor> = Arc::new(
                ProductionBrowserToolExecutor::new(
                    Arc::clone(&box_resources),
                    agent_id.clone(),
                )
                .with_auto_review_callback(browser_auto_review)
                .with_persist_image_callback(browser_persist_image)
                .with_possible_navigation_callback(browser_possible_navigation),
            );
            let computer_media_agent_id = agent_id.clone();
            let computer_persist_image: ComputerPersistImageCallback = Arc::new(
                move |bytes, mime| {
                    if mime != "image/webp" {
                        return None;
                    }
                    let db_path = computer_media_sessions
                        .session_db_path(&computer_media_agent_id)
                        .ok()?;
                    let agent_dir = db_path.parent()?;
                    let path = persist_agent_media_bytes(
                        agent_dir,
                        "computer-screenshot.webp",
                        bytes,
                        AgentMediaKind::Image,
                    )
                    .ok()?;
                    file_url_for_path(path)
                },
            );
            let computer_executor: Arc<dyn ComputerToolExecutor> = Arc::new(
                ProductionComputerToolExecutor::new(Arc::clone(&box_resources))
                    .with_auto_review_callback(computer_auto_review)
                    .with_persist_image_callback(computer_persist_image),
            );
            let file_transfer_executor: Arc<dyn FileTransferExecutor> = Arc::new(
                ProductionFileTransferExecutor::new(
                    Arc::clone(&forever_box),
                    Arc::clone(&local_exec),
                    Arc::clone(&local_tool_permission),
                    agent_id.clone(),
                ),
            );
            let external_machine_executor: Arc<dyn ExternalMachineExecutor> = Arc::new(
                ProductionExternalMachineExecutor::new(
                    Arc::clone(&local_exec),
                    Arc::clone(&local_tool_permission),
                    agent_id.clone(),
                ),
            );
            let worker_provider_usage = Arc::new(Mutex::new(None::<ProviderTokenUsage>));
            let usage_store = Arc::clone(&worker_provider_usage);
            let usage_inference = Arc::clone(&inference);
            let usage_provider = provider;
            let usage_sink: Arc<dyn Fn(ProviderTokenUsage) + Send + Sync> =
                Arc::new(move |usage| {
                    usage_inference.record_usage(
                        usage_provider,
                        InferenceUsage {
                            input_tokens: Some(usage.input_tokens),
                            output_tokens: Some(usage.output_tokens),
                            cache_read_tokens: Some(usage.cache_read_tokens),
                            cache_write_tokens: Some(usage.cache_write_tokens),
                        },
                    );
                    if let Ok(mut stored) = usage_store.lock() {
                        *stored = Some(merge_provider_token_usage(stored.take(), usage));
                    }
                });
            let cursor_auth = inference.cursor_auth_for_session(Some(&worker_session_options));
            let mut composition = create_production_runner_composition(
                ProductionRunnerCompositionInput {
                    provider,
                    bridge,
                    cursor_auth: Some(cursor_auth),
                    request_context: resolved_request_context,
                    cancellation,
                    checkpoint_store,
                    retry_sink: Some(retry_sink),
                    retry_report_sink: Some(retry_report_sink),
                    usage_sink: Some(usage_sink),
                    spotlight_enabled,
                    box_resources: Some(box_resources),
                    browser_executor: Some(browser_executor),
                    computer_executor: Some(computer_executor),
                    file_transfer_executor: Some(file_transfer_executor),
                    external_machine_executor: Some(external_machine_executor),
                    external_shell_review: Some(external_shell_review),
                    mcp_management_sink: Some(Arc::new(McpHostServiceManagementSink::new(
                        Arc::clone(&worker_mcp_service),
                        Some(agent_id.clone()),
                        true,
                    ))),
                    send_message_sink: Some(send_message_sink),
                    reaction_sink: Some(reaction_sink),
                    cloud_agent_tool: Some(cloud_agent_tool),
                    multitask_enabled,
                    action_audit: Some(ProductionActionAuditInput {
                        agent_id: agent_id.clone(),
                        turn_id: Some(stream_id.clone()),
                        sink: action_audit_sink,
                    }),
                    observation: Some(Arc::clone(&observation)),
                },
            )
            .with_agent_management_sink(agent_management_sink)
            .with_state_writer(state_writer)
            .with_routine_auto_review(routine_auto_review)
            .with_box_shell_review(box_shell_review);
            if let Some(subagent_task_sink) = worker_subagent_task_sink {
                composition = composition.with_subagent_task_sink(subagent_task_sink);
                if let Some(subagent_task_review) = subagent_task_review {
                    composition = composition.with_subagent_task_review(subagent_task_review);
                }
            }
            if let Some(subagent_runtime) = worker_subagent_management_runtime {
                composition = composition.with_subagent_management(
                    subagent_runtime,
                    subagent_steer_review,
                );
            }
            if let Some(routine_post_write) = routine_post_write {
                composition = composition.with_routine_post_write(routine_post_write);
            }
            if let Some(todo_state) = multitask_todo_state {
                composition = composition.with_multitask_todo_state(todo_state);
            }
            let owner = ProductionTurnAgentOwner::new(composition)
                .with_agent_state_checkpoint_sink(agent_state_checkpoint_sink);
            let mut runner = SandAgentRunner::new(owner)
                .with_generated_agent_runtime(Arc::clone(&worker_generated_agent_runtime));
            let generated_prompt = lifecycle_messages
                .iter()
                .rev()
                .find(|message| message.role == "user" && !message.content.trim().is_empty())
                .map(|message| message.content.trim().to_string())
                .unwrap_or_default();
            let generated_parent = worker_generated_parent_agent_id.clone();
            if let Some(parent_agent_id) = generated_parent.as_deref() {
                let _ = runner.begin_generated_subagent(
                    parent_agent_id,
                    "shipping-runner",
                    &agent_id,
                    &worker_generated_subagent_type,
                    &worker_generated_tool_call_id,
                    &generated_prompt,
                    worker_generated_lineage.clone(),
                    started_at_ms(),
                );
                worker_transcript_runtime.begin_live_subagent(parent_agent_id);
                publish_generated_subagents(
                    &worker_events,
                    &worker_generated_agent_runtime,
                    parent_agent_id,
                );
            }
            let result = runner.run_routed_provider_with_projected_messages(
                &data_dir,
                &lifecycle_messages,
                &provider_messages,
                turn_input.options,
                &mut on_text_delta,
            );
            let waiting_user = matches!(
                runner.last_finished().map(|finished| &finished.outcome),
                Some(mahayana_host_runtime::runner::TerminalOutcome::WaitingUser)
            );

            if !waiting_user && !worker_turn_hidden {
                if let Ok(content) = result.as_ref() {
                    if let Some(user_prompt) = lifecycle_messages
                        .iter()
                        .rev()
                        .find(|message| {
                            message.role == "user" && !message.content.trim().is_empty()
                        })
                        .map(|message| message.content.trim().to_string())
                        .filter(|prompt| is_memorable_exchange(prompt))
                    {
                        if let Ok(episode_db) =
                            worker_retire_sessions.open_agent_db_owner(&agent_id)
                        {
                            let memory_cancellation = worker_cancellation.clone();
                            let mut execute_memory_prompt =
                                |system_prompt: &str, user_prompt: &str| -> Result<String, String> {
                                    let messages = vec![
                                        ProviderMessage {
                                            role: "system".into(),
                                            content: system_prompt.to_string(),
                                        },
                                        ProviderMessage {
                                            role: "user".into(),
                                            content: user_prompt.to_string(),
                                        },
                                    ];
                                    let mut reject_tool = |
                                        _tool: &RoutedToolDefinition,
                                        _args: serde_json::Value,
                                        _tool_call_id: &str,
                                    | -> Result<serde_json::Value, ProviderSessionError> {
                                        Err(ProviderSessionError::Tool(
                                            "turn-memory inference exposes no tools".into(),
                                        ))
                                    };
                                    let mut ignore_delta = |_delta: &str, _accumulated: &str| {};
                                    let should_cancel = || memory_cancellation.is_cancelled();
                                    let mut options = RoutedProviderOptions {
                                        data_dir: &data_dir,
                                        cursor_auth: Some(auth.clone()),
                                        tools: &[],
                                        mcp_server_url: None,
                                        execute_tool: &mut reject_tool,
                                        on_text_delta: &mut ignore_delta,
                                        should_cancel: &should_cancel,
                                    };
                                    run_routed_provider_text(provider, &messages, &mut options)
                                        .map_err(|error| error.to_string())
                                };
                            let remembered_agent_messages = turn_agent_messages
                                .lock()
                                .map(|messages| messages.clone())
                                .unwrap_or_default();
                            let exchange = build_turn_memory_exchange(
                                user_prompt,
                                remembered_agent_messages,
                                content.clone(),
                            );
                            let _ = run_turn_memory_with(
                                &worker_memory_store,
                                Some(episode_db.as_ref()),
                                runner_started_at_ms as i64,
                                exchange,
                                TurnMemoryMode::Extract,
                                &mut execute_memory_prompt,
                            );
                        }
                    }
                }
            }

            if worker_is_ack_redrive
                && !waiting_user
                && !worker_cancellation.is_cancelled()
                && result.is_ok()
            {
                let observed_tool_call_count = observation
                    .lock()
                    .map(|observation| observation.observed_tool_call_count())
                    .unwrap_or_default();
                let stream_output_produced = result
                    .as_ref()
                    .ok()
                    .is_some_and(|content| !content.is_empty());
                let obligation = worker_ack_obligations.store().get(&agent_id);
                if let Some(report) = build_ack_redrive_empty_delivery_report(
                    obligation.as_ref(),
                    &agent_id,
                    Some(&worker_stream_id),
                    worker_request_source.as_deref(),
                    observed_tool_call_count,
                    stream_output_produced,
                    started_at_ms().saturating_sub(runner_started_at_ms),
                ) {
                    let projection = turn_empty_delivery_telemetry(&report);
                    if let Err(error) = worker_telemetry_logs.report_projection(&projection) {
                        eprintln!(
                            "mahayana-host-ack empty_delivery_telemetry_failed agent={agent_id} error={error}"
                        );
                    }
                }
            }

            if let Some(parent_agent_id) = generated_parent.as_deref() {
                let generated_outcome = match result.as_ref() {
                    Ok(content) => GeneratedSubagentRunOutcome::Completed(content.clone()),
                    Err(_) if worker_cancellation.is_cancelled() => GeneratedSubagentRunOutcome::Aborted,
                    Err(error) => GeneratedSubagentRunOutcome::Error(error.to_string()),
                };
                if let Ok(settled) = runner.settle_generated_subagent(
                    &agent_id,
                    generated_outcome,
                    started_at_ms(),
                ) {
                    if let Some(completion) = settled.completion {
                        worker_completion_revivals.handle_background_subagent_completion(
                            SubagentCompletion {
                                parent_agent_id: completion.parent_agent_id,
                                subagent_agent_id: completion.subagent_agent_id,
                                title: completion.title,
                                subagent_type: completion.subagent_type,
                                status: completion.status,
                                result: completion.result,
                                quiet_origin: None,
                            },
                        );
                    }
                }
                worker_transcript_runtime.end_live_subagent(parent_agent_id);
                publish_generated_subagents(
                    &worker_events,
                    &worker_generated_agent_runtime,
                    parent_agent_id,
                );
            }

            // A terminal inference event is the renderer-visible completion
            // boundary. Do not publish it until the Host has actually settled
            // the run: otherwise the UI can render the final assistant turn
            // while the registry/live session still owns the provider, and an
            // immediate app quit races that cleanup path.
            let usage_report = worker_transcript_runtime.settle_turn_usage(
                &agent_id,
                worker_request_source.as_deref().unwrap_or("turn"),
            );
            let usage_projection = turn_usage_telemetry(&TurnUsageFields {
                conversation_id: usage_report.agent_id.clone(),
                source: usage_report.source,
                request_id: usage_report.request_id,
                request_id_count: u64::try_from(usage_report.request_id_count)
                    .unwrap_or(u64::MAX),
                turn_ended_seq: usage_report.turn_ended_seq,
                usage: worker_provider_usage
                    .lock()
                    .ok()
                    .and_then(|usage| *usage)
                    .map(|usage| TelemetryTokenUsage {
                        input_tokens: usage.input_tokens,
                        output_tokens: usage.output_tokens,
                        cache_read_tokens: usage.cache_read_tokens,
                        cache_write_tokens: usage.cache_write_tokens,
                        reasoning_tokens: usage.reasoning_tokens,
                    }),
            });
            if let Err(error) = worker_telemetry_logs.report_projection(&usage_projection) {
                eprintln!(
                    "mahayana-host turn_usage_telemetry_failed agent={} error={error}",
                    agent_id
                );
            }
            worker_transcript_runtime.track_runner_activity_update(
                &agent_id,
                &ActivityUpdate::TurnEnded,
                started_at_ms(),
            );
            worker_transcript_runtime.end_provider_run_with_kind(&agent_id, is_group_member_turn);
            worker_registry.finish_routed_provider(&worker_stream_id);
            auto_review_service.unbind_runner(
                &agent_id,
                SandAutoReviewExpiryCause::SessionEnd,
            );
            if !is_group_member_turn {
                worker_host_runner_composition.unbind_local_permission_surface(&agent_id);
            }
            worker_ack_obligations.retire_ack_run_token(
                &agent_id,
                worker_ack_token.as_deref(),
            );
            worker_routed_turn_lease.settle();
            let _ = worker_transcript_runtime
                .retire_idle_live_session(&worker_retire_sessions, &agent_id);
            let _ = worker_box_store_sync.schedule_store_db_snapshot(&agent_id);

            if waiting_user {
                worker_events.publish(serde_json::json!({
                    "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                    "payload": {
                        "streamId": stream_id,
                        "type": "completed",
                        "content": "",
                        "waitingUser": true
                    }
                }));
            } else if !worker_cancellation.is_cancelled() {
                match result {
                    Ok(content) => worker_events.publish(serde_json::json!({
                        "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                        "payload": {
                            "streamId": stream_id,
                            "type": "completed",
                            "content": content
                        }
                    })),
                    Err(error) => {
                        let message = error.to_string();
                        if worker_is_ack_redrive || worker_is_upgrade_resume {
                            let report = AgentErrorReport {
                                source: if worker_is_upgrade_resume {
                                    "resume".into()
                                } else {
                                    "ack_redrive".into()
                                },
                                conversation_id: agent_id.clone(),
                                request_id: Some(worker_stream_id.clone()),
                                error: classify_agent_error(&error),
                                detail: Some(sand_error_detail(&error)),
                            };
                            let summary = agent_error_telemetry(&report);
                            if let Err(telemetry_error) =
                                worker_telemetry_logs.report_projection(&summary)
                            {
                                eprintln!(
                                    "mahayana-host-ack agent_error_telemetry_failed agent={agent_id} error={telemetry_error}"
                                );
                            }
                            if let Some(detail) = agent_error_detail_telemetry(&report) {
                                if let Err(telemetry_error) =
                                    worker_telemetry_logs.report_projection(&detail)
                                {
                                    eprintln!(
                                        "mahayana-host-ack agent_error_detail_telemetry_failed agent={agent_id} error={telemetry_error}"
                                    );
                                }
                            }
                        }
                        let mut tray = provider_failure_tray(
                            &agent_id,
                            &message,
                            started_at_ms() as i64,
                        );
                        if worker_is_upgrade_resume {
                            tray.title = "Agent failed to resume after host update".into();
                        }
                        worker_trays.push_error(tray);
                        worker_events.publish(serde_json::json!({
                            "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                            "payload": {
                                "streamId": stream_id,
                                "type": "failed",
                                "message": message
                            }
                        }))
                    },
                }
            }
        });
    if let Err(error) = spawn {
        transcript_runtime.end_provider_run_with_kind(
            &spawn_error_agent_id,
            is_group_member_turn,
        );
        runner_registry.finish_routed_provider(&accepted_stream_id);
        spawn_error_auto_review.unbind_runner(
            &spawn_error_agent_id,
            SandAutoReviewExpiryCause::SessionEnd,
        );
        if !is_group_member_turn {
            host_runner_composition.unbind_local_permission_surface(&spawn_error_agent_id);
        }
        ack_obligations.retire_ack_run_token(
            &spawn_error_agent_id,
            ack_token.as_deref(),
        );
        let _ = transcript_runtime
            .retire_idle_live_session(&session_workers, &spawn_error_agent_id);
        return Err(GatewayCommandError::Internal(format!(
            "could not start routed provider Runner: {error}"
        )));
    }
    Ok(serde_json::json!({
        "accepted": true,
        "streamId": accepted_stream_id,
        "provider": provider.as_str(),
    }))
}

impl GatewayApi for UnifiedGatewayApi {
    fn call(
        &self,
        method: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, GatewayCommandError> {
        if method == "loadBoxMcpServers" {
            let config_json = args
                .get("configJson")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "loadBoxMcpServers requires configJson".into(),
                ))?;
            let server_identifiers = self
                .forever_box
                .load_mcp_servers(config_json)
                .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
            return Ok(serde_json::json!({ "serverIdentifiers": server_identifiers }));
        }
        if matches!(method, "listBoxMcpToolsRaw" | "executeBoxMcpToolRaw") {
            let payload_hex = args
                .get("payloadHex")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    format!("{method} requires payloadHex"),
                ))?;
            let field_number = if method == "listBoxMcpToolsRaw" {
                MCP_STATE_EXEC_FIELD_NUMBER
            } else {
                MCP_TOOL_EXEC_FIELD_NUMBER
            };
            let mut accessor = self
                .forever_box
                .mcp_resource_accessor()
                .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
            let response_hex = execute_box_mcp_raw(&mut accessor, field_number, payload_hex)
                .map_err(GatewayCommandError::Internal)?;
            return Ok(serde_json::json!({ "payloadHex": response_hex }));
        }

        if matches!(
            method,
            "getTeachRecordingStatus" | "startTeachRecording" | "stopTeachRecording"
        ) {
            let api = self
                .teach_recording
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
                .ok_or_else(|| GatewayCommandError::Internal(
                    "teach-recording production extension is not initialized".into(),
                ))?;
            let status = match method {
                "getTeachRecordingStatus" => api.get_status(),
                "startTeachRecording" => {
                    let agent_id = args.get("agentId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| GatewayCommandError::BadRequest(
                            "startTeachRecording requires agentId".into(),
                        ))?;
                    let entry_point = args.get("entryPoint")
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    api.start(agent_id, entry_point)
                        .map_err(|error| GatewayCommandError::BadRequest(error.to_string()))?
                }
                "stopTeachRecording" => {
                    let agent_id = args.get("agentId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| GatewayCommandError::BadRequest(
                            "stopTeachRecording requires agentId".into(),
                        ))?;
                    let save = args.get("save")
                        .and_then(serde_json::Value::as_bool)
                        .ok_or_else(|| GatewayCommandError::BadRequest(
                            "stopTeachRecording requires boolean save".into(),
                        ))?;
                    api.stop(agent_id, save)
                        .map_err(|error| GatewayCommandError::BadRequest(error.to_string()))?
                }
                _ => unreachable!(),
            };
            return Ok(teach_recording_status_value(status));
        }

        if method == "getListenerIntegrations" {
            let lifecycle = self
                .automations_lifecycle
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .upgrade()
                .ok_or_else(|| GatewayCommandError::Internal(
                    "production Automations lifecycle is not initialized".into(),
                ))?;
            let definitions = self
                .transcript_manager
                .automation_runtime()
                .list_all_automation_definitions()
                .map_err(GatewayCommandError::Internal)?;
            let counts = count_listener_platforms(
                definitions
                    .iter()
                    .map(|entry| (entry.automation.is_enabled, &entry.automation.trigger)),
            );
            let mut integrations = Vec::with_capacity(2);
            for platform in ["slack", "github"] {
                let is_connected = listener_connection_state_or_disconnected(
                    platform,
                    lifecycle.is_platform_connected(platform),
                );
                let status = lifecycle.listener_source_status(platform);
                let mut integration = serde_json::Map::new();
                integration.insert("platform".into(), serde_json::Value::String(platform.into()));
                integration.insert("isConnected".into(), serde_json::Value::Bool(is_connected));
                integration.insert(
                    "state".into(),
                    serde_json::Value::String(
                        status
                            .as_ref()
                            .map(|status| status.state.as_str())
                            .unwrap_or("idle")
                            .to_string(),
                    ),
                );
                integration.insert(
                    "neededByCount".into(),
                    serde_json::Value::Number(
                        counts.get(platform).copied().unwrap_or_default().into(),
                    ),
                );
                if let Some(detail) = status.as_ref().and_then(|status| status.detail.as_ref()) {
                    integration.insert("detail".into(), serde_json::Value::String(detail.clone()));
                }
                if let Some(status) = status.as_ref().filter(|status| !status.scope_issues.is_empty()) {
                    integration.insert(
                        "scopeIssues".into(),
                        serde_json::to_value(&status.scope_issues)
                            .map_err(|error| GatewayCommandError::Internal(error.to_string()))?,
                    );
                }
                integrations.push(serde_json::Value::Object(integration));
            }
            return Ok(serde_json::json!({ "integrations": integrations }));
        }
        if method == "getListenerConnectUrl" {
            let platform = args
                .get("platform")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|platform| matches!(*platform, "github" | "slack"))
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "getListenerConnectUrl requires platform github or slack".into(),
                ))?;
            let lifecycle = self
                .automations_lifecycle
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .upgrade()
                .ok_or_else(|| GatewayCommandError::Internal(
                    "production Automations lifecycle is not initialized".into(),
                ))?;
            return Ok(serde_json::json!({
                "url": lifecycle.get_connect_url(platform),
            }));
        }

        if method == "createAgent" {
            let projected = sanitize_create_agent_args(&args);

            let nonce = projected
                .get("clientNonce")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if let Some(nonce) = nonce {
                let mut ledger = self
                    .create_agent_nonces
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(cached) = ledger.get(&nonce).cloned() {
                    return Ok(cached);
                }
                let minted = dispatch_production_session_gateway_call(
                    &self.session_workers,
                    method,
                    &projected,
                )
                .ok_or_else(|| GatewayCommandError::UnknownMethod(method.to_string()))?
                .map_err(|error| match error {
                    SessionGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
                    SessionGatewayError::Internal(message) => GatewayCommandError::Internal(message),
                })?;
                ledger.insert(nonce, minted.clone());
                return Ok(minted);
            }
            return dispatch_production_session_gateway_call(
                &self.session_workers,
                method,
                &projected,
            )
            .ok_or_else(|| GatewayCommandError::UnknownMethod(method.to_string()))?
            .map_err(|error| match error {
                SessionGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
                SessionGatewayError::Internal(message) => GatewayCommandError::Internal(message),
            });
        }

        if let Some(result) = dispatch_secrets_gateway_call(&self.secrets, method, &args) {
            return result.map_err(map_secrets_gateway_error);
        }
        if let Some(result) = self.cross_user.call_gateway(method, &args) {
            return result.map_err(|error| match error {
                CrossUserGatewayError::BadRequest(message) => {
                    GatewayCommandError::BadRequest(message)
                }
                CrossUserGatewayError::Internal(message) => {
                    GatewayCommandError::Internal(message)
                }
            });
        }
        if method == "isAgentNetworkEnabled" {
            return Ok(serde_json::Value::Bool(
                self.experiments.is_agent_network_enabled(),
            ));
        }
        if method == "runAgentAutomationNow" {
            let agent_id = args
                .get("id")
                .or_else(|| args.get("agentId"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "runAgentAutomationNow requires id".into(),
                ))?
                .to_string();
            let automation_id = args
                .get("automationId")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "runAgentAutomationNow requires automationId".into(),
                ))?
                .to_string();
            let runtime = self.transcript_manager.automation_runtime();
            let automation = runtime
                .get_agent_automations(&agent_id)
                .map_err(GatewayCommandError::Internal)?
                .into_iter()
                .find(|automation| automation.id == automation_id);
            let Some(automation) = automation else {
                return Ok(serde_json::Value::Null);
            };
            let Some(provider) =
                configured_routed_provider(&self.data_dir.join("settings.json"))
            else {
                return call_host_lane(&self.host_tx, method, args);
            };
            if provider == RoutedProvider::Cursor {
                return call_host_lane(&self.host_tx, method, args);
            }
            let deps = self.local_routed_runner_deps();
            runtime
                .run_agent_automation_now_with(
                    &agent_id,
                    &automation_id,
                    |prompt| {
                        run_local_automation_turn(
                            deps,
                            provider,
                            &agent_id,
                            &automation_id,
                            &automation.name,
                            prompt,
                        )
                    },
                )
                .map_err(GatewayCommandError::Internal)?;
            return Ok(serde_json::Value::Null);
        }
        if method == "runAgentWorkflowNow" {
            let plan = prepare_workflow_run_now(Arc::clone(&self.session_workers), &args)
                .map_err(map_workflow_command_error)?;
            match plan {
                None => return Ok(serde_json::Value::Null),
                Some(WorkflowRunNowPlan::Automation {
                    agent_id,
                    automation_id,
                    automation_name,
                }) => {
                    let Some(provider) =
                        configured_routed_provider(&self.data_dir.join("settings.json"))
                    else {
                        return call_host_lane(&self.host_tx, method, args);
                    };
                    if provider == RoutedProvider::Cursor {
                        return call_host_lane(&self.host_tx, method, args);
                    }
                    let runtime = self.transcript_manager.automation_runtime();
                    let deps = self.local_routed_runner_deps();
                    runtime
                        .run_agent_automation_now_with(
                            &agent_id,
                            &automation_id,
                            |prompt| {
                                run_local_automation_turn(
                                    deps,
                                    provider,
                                    &agent_id,
                                    &automation_id,
                                    &automation_name,
                                    prompt,
                                )
                            },
                        )
                        .map_err(GatewayCommandError::Internal)?;
                    return Ok(serde_json::Value::Null);
                }
                Some(WorkflowRunNowPlan::Reference { .. }) => {
                    // A normal workflow reference is a visible user turn. Keep
                    // it on the compatibility path until the Rust user-turn
                    // history/context projection is complete; sending only the
                    // expanded recipe here would silently drop conversation
                    // history.
                    return call_host_lane(&self.host_tx, method, args);
                }
            }
        }
        let automation_runtime = self.transcript_manager.automation_runtime();
        if let Some(result) = dispatch_automation_command(
            automation_runtime.as_ref(),
            method,
            &args,
        ) {
            let result = result.map_err(map_automation_command_error);
            if result.is_ok() && method != "getAgentAutomations" {
                self.refresh_production_automations();
            }
            return result;
        }
        let workflow_automation_runtime = self.transcript_manager.automation_runtime();
        if let Some(result) = dispatch_workflow_command_with_runtime(
            Arc::clone(&self.session_workers),
            Some(workflow_automation_runtime.as_ref()),
            method,
            &args,
        ) {
            return result.map_err(map_workflow_command_error);
        }
        if method == "promptAcceptanceStatus" {
            return self
                .transcript_manager
                .prompt_acceptance_status(&args)
                .map_err(map_production_send_error);
        }
        if method == "setWindowFocused" {
            let is_focused = args
                .get("isFocused")
                .and_then(serde_json::Value::as_bool)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "setWindowFocused requires isFocused".into()
                ))?;
            self.transcript_manager
                .set_window_focused(is_focused, started_at_ms() as f64)
                .map_err(GatewayCommandError::Internal)?;
            return Ok(serde_json::Value::Null);
        }
        if method == "openAgent" {
            let agent_id = args
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "openAgent requires id".into()
                ))?;
            return self.transcript_manager
                .switch_agent(agent_id, started_at_ms() as f64)
                .map(serde_json::Value::Array)
                .map_err(GatewayCommandError::Internal);
        }
        if method == "getAsyncTasks" {
            let agent_id = args
                .get("id")
                .or_else(|| args.get("agentId"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "getAsyncTasks requires id".into()
                ))?;
            return serde_json::to_value(
                self.transcript_manager.get_async_tasks(agent_id, &[])
            )
            .map_err(|error| GatewayCommandError::Internal(error.to_string()));
        }
        if let Some(result) =
            dispatch_production_agent_lifecycle_gateway_call_with_runtime(
                &self.session_workers,
                &self.agent_deletion_runtime,
                method,
                &args,
            )
        {
            let result = result.map_err(|error| match error {
                AgentLifecycleGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
                AgentLifecycleGatewayError::Internal(message) => GatewayCommandError::Internal(message),
            });
            if result.is_ok() {
                match method {
                    "deleteAgent" => {
                        if let Some(agent_id) = args.get("id").and_then(serde_json::Value::as_str) {
                            self.transcript_runtime
                                .session_runtime()
                                .mark_agent_deleted(agent_id);
                            self.transcript_manager.clear_agent_durable_recovery(agent_id);
                            self.host_runner_composition.forget_local_tool_permission(agent_id);
                            self.delete_production_automation_schedules(agent_id);
                        }
                    }
                    "deleteAgents" => {
                        if let Some(ids) = args.get("ids").and_then(serde_json::Value::as_array) {
                            for agent_id in ids.iter().filter_map(serde_json::Value::as_str) {
                                self.transcript_runtime
                                    .session_runtime()
                                    .mark_agent_deleted(agent_id);
                                self.transcript_manager.clear_agent_durable_recovery(agent_id);
                                self.host_runner_composition.forget_local_tool_permission(agent_id);
                                self.delete_production_automation_schedules(agent_id);
                            }
                        }
                    }
                    _ => {}
                }
                if matches!(method, "deleteAgent" | "deleteAgents") {
                    self.refresh_production_automations();
                }
            }
            return result;
        }
        if let Some(result) =
            dispatch_production_session_gateway_call_with_content_search(
                &self.session_workers,
                Some(self.content_search.as_ref()),
                method,
                &args,
            )
        {
            let mut value = result.map_err(|error| match error {
                SessionGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
                SessionGatewayError::Internal(message) => GatewayCommandError::Internal(message),
            })?;
            if method == "listAgents" {
                self.transcript_manager.decorate_agent_summaries(&mut value);
            }
            return Ok(value);
        }
        if method == "getForeverBoxStatus" {
            let agent_id = required_box_agent_id(method, &args)?;
            let status = self.forever_box.get_status(agent_id);
            let handoff = self.session_handoff.get(agent_id);
            return Ok(project_forever_box_status(&status, handoff.as_ref()));
        }
        if method == "ensureForeverBox" {
            let agent_id = required_box_agent_id(method, &args)?;
            let status = self
                .forever_box
                .ensure(agent_id)
                .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
            let handoff = self.session_handoff.get(agent_id);
            return Ok(project_forever_box_status(&status, handoff.as_ref()));
        }
        if method == "handBackForeverBox" {
            let agent_id = required_box_agent_id(method, &args)?;
            let trigger = args
                .get("trigger")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("button");
            let handoff_trigger = HandoffTrigger::Name(trigger.to_string());
            let decision = decide_box_hand_back(
                self.session_handoff.get(agent_id).as_ref(),
                &handoff_trigger,
            );
            self.session_handoff
                .end(agent_id, handoff_trigger)
                .map_err(GatewayCommandError::Internal)?;
            if let HandoffDecision::End(decision) = decision {
                let _ = self
                    .transcript_runtime
                    .resolve_box_request_tracking(&decision.request_id);
                if let Err(error) = settle_box_handoff_state(
                    &self.session_workers,
                    agent_id,
                    &decision.request_id,
                    &decision.resolution,
                ) {
                    eprintln!(
                        "mahayana-host box_handoff_settlement_failed agent={agent_id} error={error}"
                    );
                }
                let resume_args = build_box_handoff_resume_send_args(
                    agent_id,
                    &decision.trigger,
                    started_at_ms(),
                );
                if let Err(error) = self.call("sendPrompt", resume_args) {
                    eprintln!(
                        "mahayana-host box_handoff_resume_failed agent={agent_id} error={error}"
                    );
                }
            }
            let status = self.forever_box.get_status(agent_id);
            let handoff = self.session_handoff.get(agent_id);
            return Ok(project_forever_box_status(&status, handoff.as_ref()));
        }
        if method == RUNNER_ACCEPT_ROUTED_PROMPT_GATEWAY_METHOD {
            self.preempt_group_member_runs_for_direct_send(&args);
            let durable_args = args.clone();
            let acceptance = self
                .transcript_runtime
                .accept_routed_send(&durable_args, |accepted| {
                    persist_accepted_send_prompt_context(
                        &self.session_workers,
                        &durable_args,
                        accepted,
                    )
                    .map_err(map_session_send_error)
                })
                .map_err(map_production_send_error)?;

            if !acceptance.duplicate
                && durable_args
                    .get("skipAckObligation")
                    .and_then(serde_json::Value::as_bool)
                    != Some(true)
            {
                let agent_id = durable_args
                    .get("agentId")
                    .or_else(|| durable_args.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                if let Some(agent_id) = agent_id {
                    let direct_local = self
                        .session_workers
                        .summarize_agent_by_id(agent_id, None)
                        .map_err(GatewayCommandError::Internal)?
                        .is_some_and(|summary| !summary.is_group);
                    if direct_local {
                        let accepted_at_ms = started_at_ms() as f64;
                        let _ack_guard = self.ack_obligations.arm_send_guard(
                            agent_id,
                            accepted_at_ms,
                            true,
                        );
                        self.ack_obligations
                            .record_send(agent_id, accepted_at_ms)
                            .map_err(|error| GatewayCommandError::Internal(format!(
                                "could not record durable ack obligation for {agent_id}: {error}"
                            )))?;
                    }
                }
            }

            let context = acceptance.context;
            let recent_user_messages = context
                .recent_user_messages
                .into_iter()
                .map(|message| {
                    let mut value = serde_json::json!({
                        "id": message.id,
                        "text": message.text,
                    });
                    if let Some(confirmed) = message.confirmed {
                        value["confirmed"] = serde_json::Value::Bool(confirmed);
                    }
                    value
                })
                .collect::<Vec<_>>();
            return Ok(serde_json::json!({
                "accepted": true,
                "duplicate": acceptance.duplicate,
                "echoEntryId": context.echo_entry_id,
                "userMessageId": context.user_message_id,
                "recentUserMessages": recent_user_messages,
            }));
        }
        if method == RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD {
            return self
                .routed_tool_relay
                .resolve(&args)
                .map_err(|error| GatewayCommandError::BadRequest(error.to_string()));
        }
        if method == MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD {
            return self
                .mcp_lifecycle_relay
                .resolve(&args)
                .map_err(|error| GatewayCommandError::BadRequest(error.to_string()));
        }
        if method == RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD {
            return start_routed_provider_task(
                Arc::clone(&self.routed_tool_relay),
                Arc::clone(&self.mcp_service),
                self.events.clone(),
                self.host_tx.clone(),
                self.data_dir.clone(),
                Arc::clone(&self.request_context),
                Arc::clone(&self.auth),
                Arc::clone(&self.auto_review),
                Arc::clone(&self.experiments),
                Arc::clone(&self.settings),
                Arc::clone(&self.inference),
                Arc::clone(&self.session_workers),
                Arc::clone(&self.runner_registry),
                Arc::clone(&self.ack_obligations),
                Arc::clone(&self.transcript_runtime),
                Arc::clone(&self.generated_agent_runtime),
                Arc::clone(&self.completion_revivals),
                Arc::clone(&self.forever_box),
                Arc::clone(&self.local_exec),
                Arc::clone(&self.local_tool_permission),
                self.session_handoff.clone(),
                Arc::clone(&self.trays),
                self.telemetry_logs.clone(),
                self.production_action_auditor.clone(),
                Arc::clone(&self.cloud_agents),
                Arc::clone(&self.cloud_agent_watches),
                Arc::clone(&self.host_runner_composition),
                self.box_store_sync.clone(),
                Arc::clone(&self.automations_lifecycle),
                None,
                args,
            );
        }
        if method == "getTrays" {
            return serde_json::to_value(self.trays.list())
                .map_err(|error| GatewayCommandError::Internal(error.to_string()));
        }
        if method == "dismissTray" {
            let id = args
                .get("id")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "dismissTray requires id".into()
                ))?;
            return Ok(serde_json::Value::Bool(self.trays.dismiss(id)));
        }
        if method == "clearTrays" {
            self.trays.clear_all();
            return Ok(serde_json::Value::Null);
        }
        if method == "requestWebAuthnCeremony" {
            return self
                .webauthn_proxy
                .request_ceremony(args)
                .map_err(|error| GatewayCommandError::Internal(error.to_string()));
        }
        if method == RUNNER_CANCEL_ROUTED_PROVIDER_GATEWAY_METHOD {
            let stream_id = args
                .get("streamId")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| GatewayCommandError::Internal(
                    "runner.cancelRoutedProvider requires streamId".into()
                ))?;
            let reason = args
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Runner provider request cancelled");
            let cancelled_agent_id = self.runner_registry.agent_id_for_stream(stream_id);
            let was_in_flight = cancelled_agent_id
                .as_deref()
                .is_some_and(|agent_id| self.transcript_runtime.is_agent_running(agent_id));
            let cancelled = self.runner_registry.cancel_stream(stream_id, reason);
            if reason.to_ascii_lowercase().contains("superseded") {
                if let Some(agent_id) = cancelled_agent_id.as_deref() {
                    let projection = turn_interrupt_telemetry(&TurnInterruptFields {
                        conversation_id: agent_id.to_string(),
                        reason: "superseded".into(),
                        had_active_run: cancelled,
                        was_in_flight,
                    });
                    if let Err(error) = self.telemetry_logs.report_projection(&projection) {
                        eprintln!(
                            "mahayana-host superseded_turn_interrupt_telemetry_failed agent={} error={error}",
                            agent_id
                        );
                    }
                }
            }
            if cancelled {
                self.events.publish(serde_json::json!({
                    "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                    "payload": {
                        "streamId": stream_id,
                        "type": "cancelled",
                        "message": reason
                    }
                }));
            }
            return Ok(serde_json::json!({
                "streamId": stream_id,
                "cancelled": cancelled,
            }));
        }
        // UnifiedAppHost owns a QuickJS runtime and is intentionally !Send.
        // Product calls remain on its owner lane while the Runner provider
        // worker above streams through the Host event hub.
        if method == "sendPrompt" {
            self.preempt_group_member_runs_for_direct_send(&args);
            let durable_args = args.clone();
            let runner_args = shape_send_prompt_media_args(&args);
            let watchdog_registry = Arc::clone(&self.runner_registry);
            let watchdog_ack_obligations = Arc::clone(&self.ack_obligations);
            let watchdog_transcript_runtime = Arc::clone(&self.transcript_runtime);
            let watchdog_events = self.events.clone();
            let watchdog_logs = self.telemetry_logs.clone();
            let user_message_logs = self.telemetry_logs.clone();
            let accepted_logs = self.telemetry_logs.clone();
            let dequeued_logs = self.telemetry_logs.clone();
            let send_agent_id = durable_args
                .get("agentId")
                .or_else(|| durable_args.get("id"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            let send_was_in_flight = send_agent_id
                .as_deref()
                .is_some_and(|agent_id| self.transcript_runtime.is_agent_running(agent_id));
            let send_ack_guard = Mutex::new(None);
            let persisted_send_context = Mutex::new(None::<PersistedSendContext>);
            let send_result = self
                .transcript_runtime
                .execute_send_with_queue_observers(
                    &durable_args,
                    || {
                        let persisted = persisted_send_context
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone()
                            .ok_or_else(|| ProductionSendError::Internal(
                                "send dispatch started before durable persistence".into(),
                            ))?;
                        if let Some(group_result) =
                            self.dispatch_mirror_or_group_send_if_supported(
                                &durable_args,
                                &runner_args,
                                &persisted,
                            )?
                        {
                            return Ok(group_result);
                        }
                        call_host_lane(&self.host_tx, method, runner_args)
                            .map_err(map_gateway_send_error)
                    },
                    |accepted| {
                        let persisted = persist_accepted_send_prompt_context(
                            &self.session_workers,
                            &durable_args,
                            accepted,
                        )
                        .map_err(map_session_send_error)?;
                        *persisted_send_context
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                            Some(persisted.clone());
                        if accepted.get("accepted").and_then(serde_json::Value::as_bool)
                            == Some(true)
                        {
                            if let Some(agent_id) = send_agent_id.as_deref() {
                                let direct_local = self
                                    .session_workers
                                    .summarize_agent_by_id(agent_id, None)
                                    .map_err(ProductionSendError::Internal)?
                                    .is_some_and(|summary| !summary.is_group);
                                if direct_local {
                                    let projection = user_message_received_telemetry(
                                        &UserMessageReceivedFields {
                                            conversation_id: agent_id.to_string(),
                                            was_in_flight: send_was_in_flight,
                                        },
                                    );
                                    if let Err(error) = user_message_logs.report_projection(&projection) {
                                        eprintln!(
                                            "mahayana-host user_message_received_telemetry_failed agent={agent_id} error={error}"
                                        );
                                    }
                                }
                                if direct_local
                                    && durable_args
                                        .get("skipAckObligation")
                                        .and_then(serde_json::Value::as_bool)
                                        != Some(true)
                                {
                                    let accepted_at_ms = started_at_ms() as f64;
                                    let guard = self.ack_obligations.arm_send_guard(
                                        agent_id,
                                        accepted_at_ms,
                                        true,
                                    );
                                    self.ack_obligations
                                        .record_send(agent_id, accepted_at_ms)
                                        .map_err(|error| ProductionSendError::Internal(
                                            format!(
                                                "could not record durable ack obligation for {agent_id}: {error}"
                                            )
                                        ))?;
                                    *send_ack_guard
                                        .lock()
                                        .map_err(|_| ProductionSendError::Internal(
                                            "send ack guard slot poisoned".into()
                                        ))? = Some(guard);
                                }
                            }
                        }
                        Ok(persisted)
                    },
                    move |event| {
                        let was_in_flight = event.stage == WatchdogStage::Trip
                            && watchdog_transcript_runtime.is_agent_running(&event.agent_id);
                        let interrupted = if event.stage == WatchdogStage::Trip {
                            let interrupted = watchdog_registry
                                .interrupt_wedged_run_for_watchdog(&event.agent_id);
                            if interrupted {
                                let _ = watchdog_ack_obligations.record_interrupt(
                                    &event.agent_id,
                                    started_at_ms() as f64,
                                );
                            }
                            let projection = turn_interrupt_telemetry(&TurnInterruptFields {
                                conversation_id: event.agent_id.clone(),
                                reason: "watchdog".into(),
                                had_active_run: interrupted,
                                was_in_flight,
                            });
                            if let Err(error) = watchdog_logs.report_projection(&projection) {
                                eprintln!(
                                    "mahayana-host turn_interrupt_telemetry_failed agent={} error={error}",
                                    event.agent_id
                                );
                            }
                            interrupted
                        } else {
                            false
                        };
                        if event.stage == WatchdogStage::Escape {
                            let _ = watchdog_ack_obligations.retire_ack_run_token(
                                &event.agent_id,
                                event.ack_token.as_deref(),
                            );
                        }
                        let projection = queue_watchdog_telemetry(&QueueWatchdogReport {
                            conversation_id: event.agent_id.clone(),
                            stage: event.stage.as_str().to_string(),
                            active_lane: Some(event.active_lane.as_str().to_string()),
                            active_source: Some(event.active_source.clone()),
                            active_runtime_ms: event.active_runtime_ms as f64,
                            waiting_user_age_ms: event.waiting_user_age_ms.map(|value| value as f64),
                            interrupted: (event.stage == WatchdogStage::Trip).then_some(interrupted),
                        });
                        let _ = watchdog_logs.report_projection(&projection);
                        watchdog_events.publish(serde_json::json!({
                            "channel": "run-queue-watchdog",
                            "payload": {
                                "agentId": event.agent_id,
                                "stage": event.stage.as_str(),
                                "activeLane": event.active_lane.as_str(),
                                "activeSource": event.active_source,
                                "activeRuntimeMs": event.active_runtime_ms,
                                "waitingUserAgeMs": event.waiting_user_age_ms,
                                "ackToken": event.ack_token,
                                "interrupted": interrupted
                            }
                        }));
                        interrupted
                    },
                    move |event| {
                        let projection = queue_accepted_telemetry(&QueueAcceptedReport {
                            conversation_id: event.agent_id.clone(),
                            lane: event.lane.as_str().to_string(),
                            source: event.source.clone(),
                            position: i64::try_from(event.position).unwrap_or(i64::MAX),
                            depth_user: i64::try_from(event.depth_user).unwrap_or(i64::MAX),
                            depth_agent: i64::try_from(event.depth_agent).unwrap_or(i64::MAX),
                            depth_background: i64::try_from(event.depth_background).unwrap_or(i64::MAX),
                            has_active: event.has_active,
                        });
                        let _ = accepted_logs.report_projection(&projection);
                    },
                    move |event| {
                        let projection = queue_dequeued_telemetry(&QueueDequeuedReport {
                            conversation_id: event.agent_id.clone(),
                            lane: event.lane.as_str().to_string(),
                            source: event.source.clone(),
                            queue_wait_ms: event.queue_wait_ms as f64,
                            accepted_to_run_ms: event.accepted_to_run_ms.map(|value| value as f64),
                            jumped_background: i64::try_from(event.jumped_background).unwrap_or(i64::MAX),
                            depth_user: i64::try_from(event.depth_user).unwrap_or(i64::MAX),
                            depth_agent: i64::try_from(event.depth_agent).unwrap_or(i64::MAX),
                            depth_background: i64::try_from(event.depth_background).unwrap_or(i64::MAX),
                        });
                        let _ = dequeued_logs.report_projection(&projection);
                    },
                );
            // Dropping the guard after the complete dispatch scope mirrors
            // Grok's Symbol.dispose send guard: it only recreates a missing
            // direct-local obligation and never double-coalesces an existing one.
            drop(send_ack_guard);
            return send_result.map_err(map_production_send_error);
        }
        if method == "resolveAutoReviewApproval" {
            let agent_id = args
                .get("agentId")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveAutoReviewApproval requires agentId".into(),
                ))?;
            let entry_id = args
                .get("entryId")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveAutoReviewApproval requires entryId".into(),
                ))?;
            let request_id = args
                .get("requestId")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveAutoReviewApproval requires requestId".into(),
                ))?;
            let resolution = match args
                .get("resolution")
                .and_then(serde_json::Value::as_str)
            {
                Some("approved") => SandAutoReviewResolution::Approved,
                Some("denied") => SandAutoReviewResolution::Denied,
                _ => {
                    return Err(GatewayCommandError::BadRequest(
                        "resolveAutoReviewApproval requires approved or denied resolution".into(),
                    ));
                }
            };
            self.auto_review
                .service()
                .resolve_approval_for_entry(
                    request_id,
                    resolution,
                    agent_id,
                    entry_id,
                )
                .map_err(|error| {
                    if error.contains("SAND_AUTO_REVIEW_STALE") {
                        GatewayCommandError::BadRequest(error)
                    } else {
                        GatewayCommandError::Internal(error)
                    }
                })?;
            return Ok(serde_json::Value::Null);
        }
        if method == "resolveLocalToolPermission" {
            let agent_id = args
                .get("agentId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveLocalToolPermission requires agentId".into(),
                ))?;
            let entry_id = args
                .get("entryId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveLocalToolPermission requires entryId".into(),
                ))?;
            let request_id = args
                .get("requestId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveLocalToolPermission requires requestId".into(),
                ))?;
            let resolution = args
                .get("resolution")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| GatewayCommandError::BadRequest(
                    "resolveLocalToolPermission requires resolution".into(),
                ))?;
            self.local_tool_permission
                .resolve_ask(&LocalToolPermissionResolutionArgs {
                    agent_id: agent_id.to_string(),
                    entry_id: entry_id.to_string(),
                    request_id: request_id.to_string(),
                    resolution: resolution.to_string(),
                })
                .map_err(map_local_tool_permission_resolution_error)?;
            return Ok(serde_json::Value::Null);
        }
        if method == "setHostSettings" {
            let permission_changed = args.get("localToolPermission").is_some();
            let result = call_host_lane(&self.host_tx, method, args)?;
            if permission_changed {
                self.local_tool_permission.note_permission_changed();
            }
            return Ok(result);
        }
        call_host_lane(&self.host_tx, method, args)
    }

    fn health(&self) -> GatewayHealth {
        let running_agent_ids = self.transcript_runtime.live_running_agent_ids();
        let awaiting_approval_agent_ids =
            self.auto_review.service().agent_ids_with_pending_approvals();
        let active_agent_id = self
            .transcript_runtime
            .active_agent_id(&self.session_workers);
        let has_other_background_work =
            self.transcript_runtime.has_carryable_pending_wake();
        let mut last_busy_at_ms = self
            .last_busy_at_ms
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let health = compute_host_health(
            running_agent_ids,
            awaiting_approval_agent_ids,
            has_other_background_work,
            active_agent_id,
            started_at_ms(),
            *last_busy_at_ms,
        );
        *last_busy_at_ms = health.last_busy_at_ms;
        GatewayHealth {
            is_busy: health.is_busy,
            busy_only_awaiting_approval: Some(health.busy_only_awaiting_approval),
            active_agent_id: health.active_agent_id,
            last_busy_at_ms: Some(health.last_busy_at_ms),
        }
    }

    fn prepare_for_upgrade(&self) -> Result<serde_json::Value, GatewayCommandError> {
        self.host_upgrade
            .service()
            .prepare_for_upgrade()
            .map_err(GatewayCommandError::Internal)?;
        Ok(serde_json::json!({
            "quiescing": self.transcript_runtime.is_quiescing_for_upgrade(),
            "runningTurns": self.transcript_runtime.live_running_agent_ids().len(),
            "resumeAgentIds": self.transcript_runtime.upgrade_resume_agent_ids(),
        }))
    }

    fn call_with_context(
        &self,
        method: &str,
        args: serde_json::Value,
        context: &GatewayCommandContext,
    ) -> Result<serde_json::Value, GatewayCommandError> {
        if method == RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD {
            return start_routed_provider_task(
                Arc::clone(&self.routed_tool_relay),
                Arc::clone(&self.mcp_service),
                self.events.clone(),
                self.host_tx.clone(),
                self.data_dir.clone(),
                Arc::clone(&self.request_context),
                Arc::clone(&self.auth),
                Arc::clone(&self.auto_review),
                Arc::clone(&self.experiments),
                Arc::clone(&self.settings),
                Arc::clone(&self.inference),
                Arc::clone(&self.session_workers),
                Arc::clone(&self.runner_registry),
                Arc::clone(&self.ack_obligations),
                Arc::clone(&self.transcript_runtime),
                Arc::clone(&self.generated_agent_runtime),
                Arc::clone(&self.completion_revivals),
                Arc::clone(&self.forever_box),
                Arc::clone(&self.local_exec),
                Arc::clone(&self.local_tool_permission),
                self.session_handoff.clone(),
                Arc::clone(&self.trays),
                self.telemetry_logs.clone(),
                self.production_action_auditor.clone(),
                Arc::clone(&self.cloud_agents),
                Arc::clone(&self.cloud_agent_watches),
                Arc::clone(&self.host_runner_composition),
                self.box_store_sync.clone(),
                Arc::clone(&self.automations_lifecycle),
                Some(context.clone()),
                args,
            );
        }
        self.call(method, args)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        log_gateway_command_report("complete", &report);
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        log_gateway_command_report("error", &report);
    }
}

fn map_secrets_gateway_error(error: SecretsGatewayError) -> GatewayCommandError {
    match error {
        SecretsGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
        SecretsGatewayError::Internal(message) => GatewayCommandError::Internal(message),
    }
}

fn map_automation_command_error(error: AutomationCommandError) -> GatewayCommandError {
    match error {
        AutomationCommandError::BadRequest(message) => GatewayCommandError::BadRequest(message),
        AutomationCommandError::Internal(message) => GatewayCommandError::Internal(message),
    }
}

fn map_local_tool_permission_resolution_error(
    error: SandLocalToolPermissionResolutionError,
) -> GatewayCommandError {
    match error {
        SandLocalToolPermissionResolutionError::UnknownResolution
        | SandLocalToolPermissionResolutionError::Stale => {
            GatewayCommandError::BadRequest(error.to_string())
        }
        SandLocalToolPermissionResolutionError::Transcript(message) => {
            GatewayCommandError::Internal(message)
        }
    }
}

fn map_workflow_command_error(error: WorkflowCommandError) -> GatewayCommandError {
    match error {
        WorkflowCommandError::BadRequest(message) => GatewayCommandError::BadRequest(message),
        WorkflowCommandError::Internal(message) => GatewayCommandError::Internal(message),
    }
}

fn map_gateway_send_error(error: GatewayCommandError) -> ProductionSendError {
    match error {
        GatewayCommandError::BadRequest(message) => ProductionSendError::BadRequest(message),
        GatewayCommandError::Conflict(message) => ProductionSendError::Conflict(message),
        GatewayCommandError::UnknownMethod(message) | GatewayCommandError::Internal(message) => {
            ProductionSendError::Internal(message)
        }
    }
}

fn map_session_send_error(error: SessionGatewayError) -> ProductionSendError {
    match error {
        SessionGatewayError::BadRequest(message) => ProductionSendError::BadRequest(message),
        SessionGatewayError::Internal(message) => ProductionSendError::Internal(message),
    }
}

fn map_production_send_error(error: ProductionSendError) -> GatewayCommandError {
    match error {
        ProductionSendError::BadRequest(message) => GatewayCommandError::BadRequest(message),
        ProductionSendError::Conflict(message) | ProductionSendError::Rejected(message) => {
            GatewayCommandError::Conflict(message)
        }
        ProductionSendError::Internal(message) => GatewayCommandError::Internal(message),
    }
}

fn log_gateway_command_report(kind: &str, report: &GatewayCommandReport) {
    let mut value = serde_json::json!({
        "kind": kind,
        "method": report.method,
        "durationMs": report.duration_ms,
        "status": report.status,
    });
    if let Some(request_id) = report.request_id.as_deref() {
        value["requestId"] = serde_json::Value::String(request_id.to_string());
    }
    if let Some(traceparent) = report.traceparent.as_deref() {
        value["traceparent"] = serde_json::Value::String(traceparent.to_string());
    }
    if let Some(error) = report.error.as_deref() {
        value["error"] = serde_json::Value::String(error.to_string());
    }
    if let Some(reason) = report.reason.as_deref() {
        value["reason"] = serde_json::Value::String(reason.to_string());
    }
    if let Some(error_class) = report.error_class.as_deref() {
        value["errorClass"] = serde_json::Value::String(error_class.to_string());
    }
    if let Some(errno) = report.errno.as_deref() {
        value["errno"] = serde_json::Value::String(errno.to_string());
    }
    eprintln!("mahayana-host-gateway-command {value}");
}

fn decode_box_environment_update(
    args: &serde_json::Value,
) -> Result<BoxEnvironmentUpdate, GatewayCommandError> {
    let object = args.as_object().ok_or_else(|| {
        GatewayCommandError::Internal("box.applyEnvironment params must be an object".into())
    })?;
    let raw_env = object.get("env").and_then(serde_json::Value::as_object).ok_or_else(|| {
        GatewayCommandError::Internal("box.applyEnvironment params.env must be an object".into())
    })?;
    let replace = object
        .get("replace")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            GatewayCommandError::Internal("box.applyEnvironment params.replace must be a boolean".into())
        })?;
    let mut env = BTreeMap::new();
    for (name, value) in raw_env {
        let value = value.as_str().ok_or_else(|| {
            GatewayCommandError::Internal(format!(
                "box.applyEnvironment env value for {name} must be a string"
            ))
        })?;
        env.insert(name.clone(), value.to_string());
    }
    Ok(BoxEnvironmentUpdate { env, replace })
}

fn dispatch_box_environment_call<Apply>(
    method: &str,
    args: &serde_json::Value,
    mut apply: Apply,
) -> Option<Result<serde_json::Value, GatewayCommandError>>
where
    Apply: FnMut(&BoxEnvironmentUpdate) -> Result<(), String>,
{
    if method != BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD {
        return None;
    }
    Some(
        decode_box_environment_update(args).and_then(|update| {
            apply(&update)
                .map_err(GatewayCommandError::Internal)
                .map(|()| serde_json::json!({ "applied": true }))
        }),
    )
}

fn dispatch_gateway_call(
    host: &UnifiedAppHost,
    forever_box: &ForeverBoxService,
    attachments: &AttachmentsService,
    method: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    if let Some(result) = attachments.dispatch_gateway(method, &args) {
        return result.map_err(GatewayCommandError::Internal);
    }

    match host.grok_gateway_call(method, args.clone()) {
        Ok(Some(result)) => return Ok(result),
        Ok(None) => {}
        Err(error) => {
            return Err(GatewayCommandError::Internal(format!(
                "Mahayana Host Grok compatibility dispatch failed for {method}: {error}"
            )));
        }
    }

    if let Some(result) = dispatch_box_environment_call(method, &args, |update| {
        forever_box
            .apply_environment(update)
            .map_err(|error| error.to_string())
    }) {
        return result;
    }

    let request = serde_json::json!({
        "id": "gateway",
        "method": method,
        "params": args,
    });
    let encoded = serde_json::to_string(&request)
        .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
    let response = dispatch_json(host, &encoded);
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
    if parsed.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(parsed.get("result").cloned().unwrap_or(serde_json::Value::Null));
    }
    let message = parsed
        .get("error")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Mahayana Host gateway request failed")
        .to_string();
    if message.contains("unknown") {
        Err(GatewayCommandError::UnknownMethod(method.to_string()))
    } else {
        Err(GatewayCommandError::Internal(message))
    }
}

fn started_at_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}

fn write_response(stdout: &Mutex<io::Stdout>, response: &str) -> io::Result<()> {
    let mut stdout = stdout
        .lock()
        .map_err(|_| io::Error::other("desktop host stdout lock poisoned"))?;
    writeln!(stdout, "{response}")?;
    stdout.flush()
}

fn write_runtime_event(
    stdout: &Mutex<io::Stdout>,
    gateway_events: &GatewayEventHub,
    event: serde_json::Value,
) -> io::Result<()> {
    gateway_events.publish(event.clone());
    let frame = serde_json::json!({ "event": event });
    let encoded = serde_json::to_string(&frame)
        .map_err(|error| io::Error::other(format!("event serialization failed: {error}")))?;
    write_response(stdout, &encoded)
}

fn drain_ready_runtime_events(
    host: &UnifiedAppHost,
    stdout: &Mutex<io::Stdout>,
    gateway_events: &GatewayEventHub,
) -> io::Result<()> {
    // Product-local events enqueued by a just-completed command must cross the
    // exact same Grok projection boundary as events delivered by the background
    // PUSH worker. Writing this synchronous drain raw creates a second event
    // protocol and drops compatibility metadata such as attachment contexts.
    let event_source = host.feature_event_source();
    loop {
        match event_source.receive(Duration::ZERO) {
            Ok(Some(event)) => {
                let event = event_source.project_grok_gateway_event(&event);
                write_runtime_event(stdout, gateway_events, event)?;
            }
            Ok(None) => return Ok(()),
            Err(error) => {
                eprintln!("failed to drain Mahayana runtime event: {error}");
                return Ok(());
            }
        }
    }
}

fn main() {
    if std::env::args().any(|arg| arg == BOX_COPY_IN_ARG) {
        let environment = std::env::vars().collect::<BTreeMap<_, _>>();
        let exit_code = execute_production_box_copy_in_from_env(&environment, Path::new("/"));
        std::process::exit(exit_code);
    }

    let _process_crash_guard =
        mahayana_host_runtime::process_crash_guard::install_process_crash_guards(
            "sand-host",
            None,
        );
    let app_data_dir = default_unified_app_data_dir();
    if let Err(error) = ensure_managed_runtime_layout(&app_data_dir) {
        eprintln!(
            "failed to initialize managed Mahayana runtime layout at {}: {error}",
            app_data_dir.display()
        );
        std::process::exit(1);
    }

    let host = match UnifiedAppHost::new(app_data_dir.clone()) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("failed to initialize unified Mahayana app host: {error}");
            std::process::exit(1);
        }
    };
    let production_box = ProductionBoxEnvironment::from_process_env();
    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    // Platform/account HTTP may legitimately take tens of seconds. Keep it on
    // a dedicated Rust product lane so feature.receive and Agent commands keep
    // flowing through the primary serial Host lane without starvation. The
    // child protocol already correlates responses by id, so out-of-order
    // platform replies are safe.
    let platform_host = match PlatformRequestHost::new(app_data_dir.clone()) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("failed to initialize Mahayana platform request lane: {error}");
            std::process::exit(1);
        }
    };
    let stdout = Arc::new(Mutex::new(io::stdout()));

    let host_lock_path = get_host_lock_path();
    let host_lock = match acquire_host_lock(&host_lock_path, std::process::id()) {
        Ok(acquisition) => acquisition.lock,
        Err(error) => {
            eprintln!(
                "failed to acquire Mahayana Host lock at {}: {error}",
                host_lock_path.display()
            );
            return;
        }
    };

    let mut box_exec_daemon =
        match start_managed_box_exec_daemon_from_process_env(&app_data_dir) {
            Ok(daemon) => daemon,
            Err(error) => {
                eprintln!("failed to start managed Grok box exec-daemon: {error}");
                return;
            }
        };

    let gateway_events = GatewayEventHub::default();
    let production_extensions = match start_production_host_extensions(
        &app_data_dir,
        gateway_events.clone(),
    ) {
        Ok(extensions) => extensions,
        Err(error) => {
            eprintln!("failed to start production Host extensions: {error}");
            return;
        }
    };
    let host_telemetry = production_extensions.telemetry.clone();
    let box_extensions =
        start_production_host_box_extensions(&production_extensions, production_box);
    let forever_box = Arc::clone(&box_extensions.forever_box);
    let attachments_service = box_extensions.attachments.service();
    let secrets_extension = Arc::clone(&box_extensions.secrets);
    let runner_request_context: Arc<dyn RunnerRequestContextSource> =
        Arc::new(ProductionRunnerRequestContextSource::new(
            Arc::clone(&production_extensions.auth),
            Arc::clone(production_extensions.managed_setup.team_rules()),
            app_data_dir.join("transcripts"),
        ));
    let settings_extension = Arc::clone(&production_extensions.settings);
    let local_tool_permission_extension =
        Arc::clone(&production_extensions.local_tool_permission);
    let settings_for_session = Arc::clone(&settings_extension);

    let session_workers = Arc::new(
        ProductionSessionWorkers::production_with_dependencies(
            Arc::new(move || settings_for_session.get_user_time_zone()),
            production_extensions.memory.service(),
        ),
    );
    let handoff_prepare_box = Arc::clone(&forever_box);
    let handoff_screenshot_box = Arc::clone(&forever_box);
    let handoff_status_box = Arc::clone(&forever_box);
    let handoff_status_events = gateway_events.clone();
    let handoff_started_events = gateway_events.clone();
    let handoff_ended_events = gateway_events.clone();
    let handoff_logs = host_telemetry.logs.clone();
    let handoff_analytics = host_telemetry.analytics.clone();
    let handoff_deps = BoxHandoffDeps {
        prepare: Some(Arc::new(move |request| {
            handoff_prepare_box
                .ensure(&request.agent_id)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })),
        grab_screenshot: Some(Arc::new(move |agent_id| {
            handoff_screenshot_box
                .capture_screenshot(agent_id)
                .map(|bytes| bytes.map(ScreenshotPayload::Bytes))
                .map_err(|error| error.to_string())
        })),
        on_status_changed: Some(Arc::new(move |agent_id, pending| {
            let status = handoff_status_box.get_status(agent_id);
            handoff_status_events.publish(serde_json::json!({
                "channel": "forever-box",
                "payload": project_forever_box_status(&status, pending.as_ref()),
            }));
        })),
        on_started: Some(Arc::new(move |event| {
            handoff_started_events.publish(serde_json::json!({
                "channel": "session.box-handoff-started",
                "payload": {
                    "agentId": event.agent_id,
                    "instruction": event.instruction,
                },
            }));
        })),
        on_ended: Some(Arc::new(move |event| {
            handoff_ended_events.publish(serde_json::json!({
                "channel": "session.box-handoff-ended",
                "payload": {
                    "agentId": event.agent_id,
                    "requestId": event.request_id,
                    "resolution": event.resolution,
                    "trigger": event.trigger,
                },
            }));
            Ok(())
        })),
        report_box_help: Some(Arc::new(move |event| {
            let _ = handoff_logs.report_box_help(&event);
        })),
        track_event: Some(Arc::new(move |name, properties| {
            let _ = handoff_analytics.track_event(name, &properties);
        })),
        ..BoxHandoffDeps::default()
    };
    let session_extension = start_session_extension(
        Arc::clone(&production_extensions.experiments),
        Arc::clone(&session_workers),
        handoff_deps,
    );
    let session_workers = session_extension.store();
    let session_handoff = session_extension.handoff_service();
    let auto_review_update_events = gateway_events.clone();
    let auto_review_update_sessions = Arc::clone(&session_workers);
    let auto_review_update_sink = Arc::new(move |agent_id: &str, update: serde_json::Value| {
        let request_id = update
            .get("requestId")
            .or_else(|| update.pointer("/message/approval/requestId"))
            .and_then(serde_json::Value::as_str);
        let Some(request_id) = request_id else {
            return;
        };
        let Ok(entries) = auto_review_update_sessions.read_agent_transcript_entries(agent_id) else {
            return;
        };
        let Some(entry) = entries.into_iter().find(|entry| {
            entry.get("id").and_then(serde_json::Value::as_str) == Some(request_id)
        }) else {
            return;
        };
        let event_type = if update.get("type").and_then(serde_json::Value::as_str)
            == Some("send-message")
        {
            "appended"
        } else {
            "updated"
        };
        auto_review_update_events.publish(serde_json::json!({
            "channel": "transcript",
            "payload": {
                "type": event_type,
                "agentId": agent_id,
                "entry": entry,
            }
        }));
    });
    let auto_review_logs = host_telemetry.logs.clone();
    let auto_review_telemetry_sink = Arc::new(move |event: &SandAutoReviewEvent| {
        let (event_type, approval, cause) = match event {
            SandAutoReviewEvent::Created(approval) => ("created", approval, None),
            SandAutoReviewEvent::Resolved(approval) => ("resolved", approval, None),
            SandAutoReviewEvent::Expired { approval, cause } => (
                "expired",
                approval,
                Some(match cause {
                    SandAutoReviewExpiryCause::Ttl => "ttl",
                    SandAutoReviewExpiryCause::Cancelled => "cancelled",
                    SandAutoReviewExpiryCause::UserRedirect => "user_redirect",
                    SandAutoReviewExpiryCause::SettingsChange => "settings_change",
                    SandAutoReviewExpiryCause::SessionEnd => "session_end",
                    SandAutoReviewExpiryCause::Quiesce => "quiesce",
                    SandAutoReviewExpiryCause::Other(value) => value.as_str(),
                }.to_string()),
            ),
        };
        let status = match approval.status {
            SandAutoReviewApprovalStatus::Pending => "pending",
            SandAutoReviewApprovalStatus::Approved => "approved",
            SandAutoReviewApprovalStatus::Denied => "denied",
            SandAutoReviewApprovalStatus::Expired => "expired",
        };
        let now_ms = started_at_ms();
        let report = AutoReviewApprovalReport {
            event_type: event_type.to_string(),
            conversation_id: approval.agent_id.clone(),
            approval_id: approval.id.clone(),
            surface: approval.surface.key().to_string(),
            status: status.to_string(),
            age_ms: now_ms.saturating_sub(approval.created_at_ms) as f64,
            ttl_ms: approval
                .expires_at_ms
                .map(|expires_at_ms| expires_at_ms.saturating_sub(approval.created_at_ms) as f64),
            cause,
        };
        let _ = auto_review_logs.report_projection(&auto_review_approval_telemetry(&report));
    });
    let auto_review_extension = Arc::new(start_auto_review_extension(
        Arc::clone(&session_workers),
        Arc::clone(&production_extensions.experiments),
        Arc::clone(&settings_extension),
        format!("host-{}", uuid::Uuid::new_v4()),
        auto_review_update_sink,
        auto_review_telemetry_sink,
    ));
    let transcript_event_hub = gateway_events.clone();
    let transcript_extension = start_transcript_extension(
        &app_data_dir,
        Arc::clone(&session_workers),
        Arc::new(move |event| transcript_event_hub.publish(event)),
    );
    if let Some(error) = transcript_extension.profile_watch_error() {
        eprintln!(
            "[sand-host] profile watcher unavailable; roster RPC remains authoritative: {error}"
        );
    }
    let transcript_manager = transcript_extension.manager();
    let content_search_extension = Arc::clone(&production_extensions.content_search);
    let permission_widget_responses =
        Arc::new(WidgetResponses::new(Arc::clone(&session_workers)));
    let stranded_permission_logs = host_telemetry.logs.clone();
    local_tool_permission_extension.bind_transcript(
        permission_widget_responses,
        started_at_ms(),
        Some(Arc::new(move || {
            let projection = HostTelemetryProjection {
                level: Some("warn"),
                event: Some("sand.local_tool_permission.stranded_retirement"),
                metadata: std::collections::BTreeMap::new(),
            };
            let _ = stranded_permission_logs.report_projection(&projection);
        })),
        Some(Arc::new(|message| eprintln!("{message}"))),
    );
    let retired_permission_events = gateway_events.clone();
    let local_exec_retirement_owner = Arc::clone(&production_extensions.local_exec);
    local_tool_permission_extension.bind_approval_retired_sink(Some(Arc::new(move |approval_id| {
        local_exec_retirement_owner.retire_approval(approval_id);
        retired_permission_events.publish(serde_json::json!({
            "channel": "local-tool-permission.approval-retired",
            "payload": { "approvalId": approval_id },
        }));
    })));

    let notification_baseline = session_workers
        .list_agent_summaries(None)
        .ok()
        .map(|summaries| {
            summaries
                .into_iter()
                .filter_map(|summary| serde_json::to_value(summary).ok())
                .filter_map(|value| notification_agent_from_value(&value))
                .collect::<Vec<_>>()
        });
    let notification_transcript = transcript_manager.transcript_runtime();
    let _notifications_extension = match start_notifications_extension(
        Arc::clone(&production_extensions.auth),
        gateway_events.clone(),
        notification_baseline,
        Arc::new(move || {
            notification_transcript
                .session_runtime()
                .window_focused_at_ms()
                .filter(|value| value.is_finite() && *value >= 0.0)
                .map(|value| value as u64)
        }),
    ) {
        Ok(extension) => extension,
        Err(error) => {
            eprintln!("failed to start production Notifications extension: {error}");
            return;
        }
    };
    {
        let dropped_logs = host_telemetry.logs.clone();
        transcript_manager
            .automation_runtime()
            .set_dropped_fire_reporter(Some(Arc::new(move |dropped| {
                let lateness_ms = dropped.scheduled_for_ms.map(|scheduled_for_ms| {
                    (started_at_ms() as f64 - scheduled_for_ms).max(0.0)
                });
                let projection = automation_fire_dropped_telemetry(
                    &AutomationFireDroppedReport {
                        conversation_id: dropped.agent_id,
                        trigger: dropped.trigger.as_str().to_string(),
                        reason: dropped.reason,
                        scheduled_for_ms: dropped.scheduled_for_ms,
                        lateness_ms,
                        error_type: None,
                        error_code: None,
                        run_uuid: dropped.run_uuid,
                        fire_age_ms: None,
                        has_definition_revision: None,
                        box_uptime_ms: None,
                    },
                );
                let _ = dropped_logs.report_projection(&projection);
            })));
    }
    let runner_registry = transcript_manager.runner_registry();
    let host_runner_composition = Arc::new(HostRunnerComposition::production(
        local_tool_permission_extension.controller(),
        Arc::clone(&session_workers),
    ));
    let permission_surface_owner = Arc::clone(&host_runner_composition);
    local_tool_permission_extension.bind_ask_surfaces(Arc::new(move |agent_id| {
        permission_surface_owner.can_ask_local_tool_permission(agent_id)
    }));
    let ack_obligations = transcript_manager.ack_obligations();
    let cross_user_deletion_slot =
        Arc::new(Mutex::new(Weak::<ProductionCrossUserRuntime>::new()));
    let agent_deletion_runtime = AgentDeletionRuntimeDeps {
        cancel_runner: Some({
            let runner_registry = Arc::clone(&runner_registry);
            let transcript_runtime = transcript_manager.transcript_runtime();
            let telemetry_logs = host_telemetry.logs.clone();
            Arc::new(move |agent_id| {
                let was_in_flight = transcript_runtime.is_agent_running(agent_id);
                let had_active_run =
                    runner_registry.cancel_agent(agent_id, "agent deleted") > 0;
                let projection = turn_interrupt_telemetry(&TurnInterruptFields {
                    conversation_id: agent_id.to_string(),
                    reason: "agent_deleted".into(),
                    had_active_run,
                    was_in_flight,
                });
                telemetry_logs
                    .report_projection(&projection)
                    .map_err(|error| error.to_string())?;
                Ok(())
            })
        }),
        forget_ack: Some({
            let ack_obligations = Arc::clone(&ack_obligations);
            Arc::new(move |agent_id| {
                let _ = ack_obligations
                    .forget_agent(agent_id)
                    .map_err(|error| error.to_string())?;
                Ok(())
            })
        }),
        sharing_departure: Some({
            let slot = Arc::clone(&cross_user_deletion_slot);
            Arc::new(move |agent_id| {
                let runtime = slot
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .upgrade();
                if let Some(runtime) = runtime {
                    runtime.note_agent_deleted(agent_id)?;
                }
                Ok(())
            })
        }),
        release_box: Some({
            let forever_box = Arc::clone(&forever_box);
            Arc::new(move |agent_id| {
                forever_box.release_agent(agent_id);
                Ok(())
            })
        }),
        forget_handoff: Some({
            let handoff = session_handoff.clone();
            Arc::new(move |agent_id| {
                handoff.forget(agent_id);
                Ok(())
            })
        }),
    };

    let gateway_config = match resolve_gateway_server_config() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to resolve Mahayana gateway configuration: {error}");
            return;
        }
    };
    let gateway_started_at = started_at_ms();
    let local_exec_extension = Arc::clone(&production_extensions.local_exec);
    let local_exec_ask_owner = Arc::clone(&local_exec_extension);
    local_tool_permission_extension.bind_live_computer_check(Arc::new(move |agent_id| {
        local_exec_ask_owner.check_live_computer_for_ask(Some(agent_id))
    }));
    let routed_tool_relay = Arc::new(CoordinatorToolRelay::new(gateway_events.clone()));
    let mcp_lifecycle_relay = Arc::new(CoordinatorMcpLifecycleRelay::new(gateway_events.clone()));
    let box_mcp_owner = Arc::clone(&forever_box);
    let box_status_loader: BoxServerStatusLoader = Arc::new(move |ids, kick_only| {
        let accessor = box_mcp_owner
            .mcp_resource_accessor()
            .map_err(|error| error.to_string())?;
        ProductionBoxMcpStateLoader::new(accessor).list_servers(ids, kick_only)
    });
    let mcp_manager_backend = Arc::new(CoordinatorMcpManagerBackend::new(
        Arc::clone(&mcp_lifecycle_relay),
        box_status_loader,
    ));
    let transcript_runtime = transcript_manager.transcript_runtime();
    let generated_agent_runtime = Arc::new(Mutex::new(SubagentRuntime::default()));
    let box_store_idle_runtime = Arc::clone(&transcript_runtime);
    let mcp_service = match production_extensions.start_mcp(
        &app_data_dir,
        Arc::clone(&mcp_lifecycle_relay),
        mcp_manager_backend,
    ) {
        Ok(service) => service,
        Err(error) => {
            eprintln!("failed to start production MCP extension: {error}");
            return;
        }
    };
    if let Err(error) = production_extensions.start_box_store_sync(Arc::new(move || {
        box_store_idle_runtime.live_running_agent_ids().is_empty()
            && !box_store_idle_runtime.has_carryable_pending_wake()
    })) {
        eprintln!("failed to start production BoxStoreSync extension: {error}");
        return;
    }
    let box_store_sync_api = match production_extensions.box_store_sync_api() {
        Ok(Some(api)) => api,
        Ok(None) => {
            eprintln!("production BoxStoreSync started without an API");
            return;
        }
        Err(error) => {
            eprintln!("failed to access production BoxStoreSync API: {error}");
            return;
        }
    };
    match load_initial_transcript_resiliently(|| {
        ensure_initial_transcript_loaded(&session_workers, transcript_runtime.session_runtime())
    }) {
        Ok(outcome) => match outcome.degraded {
            Some(InitialTranscriptDegradedReason::SqliteBusy(error)) => {
                eprintln!(
                    "[sand-host] initial transcript load still locked after retries (kept alive): {error}"
                );
            }
            Some(InitialTranscriptDegradedReason::AgentLimit) => {
                eprintln!(
                    "[sand-host] no session at the agent cap; starting without one so the roster and delete stay reachable"
                );
            }
            None => {
                eprintln!(
                    "[sand-host] initial transcript loaded entries={}",
                    outcome.entry_count
                );
            }
        },
        Err(error) => {
            eprintln!("[sand-host] initial transcript load failed: {error}");
            return;
        }
    }
    let automations_lifecycle_slot =
        Arc::new(Mutex::new(Weak::<ProductionAutomationsLifecycle>::new()));
    let cloud_agent_completion_gateway_slot =
        Arc::new(Mutex::new(Weak::<UnifiedGatewayApi>::new()));
    let completion_revivals = Arc::new(CompletionRevivals::new(Arc::new(
        ProductionCompletionRevivalRuntime {
            gateway: Arc::clone(&cloud_agent_completion_gateway_slot),
        },
    )));
    let cloud_watch_manager = production_extensions.cloud_agents.service();
    let cloud_watch_pending_store = transcript_runtime.pending_wake_store().cloned();
    let cloud_watch_pending_events = gateway_events.clone();
    let cloud_watch_settled = Arc::clone(&completion_revivals);
    let cloud_watch_async_runtime = Arc::clone(&transcript_runtime);
    let cloud_watch_async_events = gateway_events.clone();
    let cloud_agent_watches = Arc::new(RunnerCloudAgentWatches::new(
        Arc::new(move |bc_id, wait_for_restart| {
            let result = cloud_watch_manager.await_completion(bc_id, wait_for_restart);
            mahayana_host_runtime::runner::background_work::CloudAgentWatchOutcome {
                status: result.status.to_string(),
                text: result.text,
            }
        }),
        cloud_watch_pending_store.map(|store| {
            Arc::new(move |pending: &mahayana_host_runtime::runner::background_work::CloudAgentPendingWatch| {
                let quiet_origin = pending
                    .quiet_origin
                    .as_ref()
                    .and_then(coerce_quiet_origin);
                let written = store.mark_pending(DurablePendingWakeMarker {
                    agent_id: pending.parent_agent_id.clone(),
                    kind: PendingWakeKind::CloudAgent,
                    work_id: pending.work_id.clone(),
                    marked_at_ms: started_at_ms() as f64,
                    quiet_origin,
                    title: Some(pending.title.clone()),
                    subagent_type: Some("cursor-agent".into()),
                    interrupted_by_recreate: false,
                });
                if !written {
                    eprintln!(
                        "mahayana-host pending_cloud_agent_wake_persist_failed agent={} work={}",
                        pending.parent_agent_id, pending.work_id
                    );
                }
                cloud_watch_pending_events.publish(serde_json::json!({
                    "channel": "pending-wake",
                    "payload": {
                        "agentId": pending.parent_agent_id,
                        "kind": "cloud-agent",
                        "workId": pending.work_id,
                        "outcome": if written { "persisted" } else { "persist_failed" },
                    }
                }));
            }) as mahayana_host_runtime::runner::background_work::CloudAgentPendingCallback
        }),
        Some(Arc::new(move |completion| {
            cloud_watch_settled.handle_background_subagent_completion(SubagentCompletion {
                parent_agent_id: completion.parent_agent_id,
                subagent_agent_id: completion.work_id,
                title: completion.title,
                subagent_type: "cursor-agent".into(),
                status: completion.status,
                result: completion.result,
                quiet_origin: completion
                    .quiet_origin
                    .as_ref()
                    .and_then(coerce_quiet_origin),
            });
        })),
        Some(Arc::new(move |agent_id| {
            let tasks = cloud_watch_async_runtime.get_async_tasks(agent_id, &[]);
            cloud_watch_async_events.publish(serde_json::json!({
                "channel": "async-tasks",
                "payload": {
                    "parentAgentId": agent_id,
                    "tasks": tasks,
                }
            }));
        })),
    ));
    let cross_user_runner_deps = LocalRoutedRunnerDeps {
        routed_tool_relay: Arc::clone(&routed_tool_relay),
        mcp_service: Arc::clone(&mcp_service),
        auth: Arc::clone(&production_extensions.auth),
        auto_review: Arc::clone(&auto_review_extension),
        events: gateway_events.clone(),
        host_tx: host_tx.clone(),
        data_dir: app_data_dir.clone(),
        request_context: Arc::clone(&runner_request_context),
        experiments: Arc::clone(&production_extensions.experiments),
        settings: Arc::clone(&settings_extension),
        inference: Arc::clone(&production_extensions.inference),
        session_workers: Arc::clone(&session_workers),
        runner_registry: Arc::clone(&runner_registry),
        ack_obligations: Arc::clone(&ack_obligations),
        transcript_runtime: Arc::clone(&transcript_runtime),
        generated_agent_runtime: Arc::clone(&generated_agent_runtime),
        completion_revivals: Arc::clone(&completion_revivals),
        forever_box: Arc::clone(&forever_box),
        local_exec: Arc::clone(&local_exec_extension),
        local_tool_permission: Arc::clone(&local_tool_permission_extension),
        session_handoff: session_handoff.clone(),
        trays: Arc::clone(&production_extensions.trays),
        telemetry_logs: host_telemetry.logs.clone(),
        production_action_auditor: production_extensions.action_audit.clone(),
        cloud_agents: production_extensions.cloud_agents.service(),
        cloud_agent_watches: Arc::clone(&cloud_agent_watches),
        host_runner_composition: Arc::clone(&host_runner_composition),
        box_store_sync: box_store_sync_api.clone(),
        automations_lifecycle: Arc::clone(&automations_lifecycle_slot),
    };
    let cross_user_settings_path = app_data_dir.join("settings.json");
    let remote_requested_runner_deps = cross_user_runner_deps.clone();
    let remote_requested_settings_path = cross_user_settings_path.clone();
    let run_remote_requested_turn: RemoteRequestedTurnRunner = Arc::new(
        move |agent_id, system_prompt, prompt| {
            let provider = configured_routed_provider(&remote_requested_settings_path)
                .ok_or_else(|| "no routed provider configured for shared-room turn".to_string())?;
            run_local_group_member_turn(
                remote_requested_runner_deps.clone(),
                provider,
                GroupMemberTurnRequest {
                    member: GroupMember {
                        id: agent_id.to_string(),
                        name: agent_id.to_string(),
                        description: String::new(),
                    },
                    system_prompt: system_prompt.to_string(),
                    prompt: prompt.to_string(),
                    group: GroupDescription {
                        name: "Shared room".into(),
                        description: String::new(),
                    },
                    peers: Vec::new(),
                    new_messages: Vec::new(),
                    shared_room_id: None,
                },
            )
        },
    );
    let shared_room_runner_deps = cross_user_runner_deps;
    let shared_room_settings_path = cross_user_settings_path;
    let shared_room_sessions = Arc::clone(&session_workers);
    let shared_room_runtime = Arc::clone(&transcript_runtime);
    let run_shared_room_turn: SharedRoomTurnRunner = Arc::new(
        move |room_agent_id, remote_executor| {
            let provider = configured_routed_provider(&shared_room_settings_path)
                .ok_or_else(|| "no routed provider configured for shared-room fanout".to_string())?;
            let deps = shared_room_runner_deps.clone();
            let executor: GroupMemberTurnExecutor = Arc::new(move |request| {
                run_local_group_member_turn(deps.clone(), provider, request)
            });
            let epoch = shared_room_runtime.next_turn_epoch(room_agent_id);
            match dispatch_local_group_send(
                Arc::clone(&shared_room_sessions),
                Arc::clone(&shared_room_runtime),
                room_agent_id,
                epoch,
                executor,
                remote_executor,
            )? {
                LocalGroupFanoutDisposition::NotGroup => Err(format!(
                    "shared-room relay target is not a group: {room_agent_id}"
                )),
                LocalGroupFanoutDisposition::DeferredRemote { .. } => Err(
                    "shared-room relay fanout is still missing a remote executor".to_string(),
                ),
                LocalGroupFanoutDisposition::Completed { .. } => Ok(()),
            }
        },
    );
    let cross_user = match ProductionCrossUserRuntime::new(
        Arc::clone(&production_extensions.auth),
        production_extensions.notify_bus.clone(),
        Arc::clone(&attachments_service),
        transcript_manager.shared_rooms(),
        run_remote_requested_turn,
        run_shared_room_turn,
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to initialize production CrossUserSharing: {error}");
            return;
        }
    };

    *cross_user_deletion_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
        Arc::downgrade(&cross_user);

    let host_upgrade_gateway_slot =
        Arc::new(Mutex::new(Weak::<UnifiedGatewayApi>::new()));
    let host_upgrade_automation = transcript_manager.automation_runtime();
    let host_upgrade_production_automations = Arc::clone(&automations_lifecycle_slot);
    let host_upgrade_sharing = Arc::clone(&cross_user);
    let host_upgrade_transcript = Arc::clone(&transcript_runtime);
    let host_upgrade_resume_gateway = Arc::clone(&host_upgrade_gateway_slot);
    let host_upgrade = match start_production_host_upgrade_extension(
        ProductionHostUpgradePeers {
            suspend_automation_wakes: Arc::new(move || {
                host_upgrade_automation.suspend_wakes();
                if let Some(runtime) = host_upgrade_production_automations
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .upgrade()
                {
                    runtime.suspend_wakes();
                }
                Ok(())
            }),
            prepare_sharing_for_upgrade: Arc::new(move || {
                host_upgrade_sharing.prepare_for_upgrade();
            }),
            quiesce_transcript_for_upgrade: Arc::new(move || {
                host_upgrade_transcript.quiesce_for_upgrade();
                Ok(())
            }),
            resume_interrupted_upgrade_turns: Arc::new(move || {
                let gateway = host_upgrade_resume_gateway
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .upgrade()
                    .ok_or_else(|| "Host gateway is not ready for upgrade resume".to_string())?;
                gateway.resume_interrupted_upgrade_turns()
            }),
        },
        host_telemetry.logs.clone(),
        Arc::new(|| Ok(())),
    ) {
        Ok(extension) => extension,
        Err(error) => {
            eprintln!("failed to start production HostUpgrade extension: {error}");
            return;
        }
    };

    let teach_recording_slot = Arc::new(Mutex::new(None::<TeachRecordingApi>));

    let gateway_api = Arc::new(UnifiedGatewayApi {
            host_tx: host_tx.clone(),
            auth: Arc::clone(&production_extensions.auth),
            experiments: Arc::clone(&production_extensions.experiments),
            settings: Arc::clone(&settings_extension),
            inference: Arc::clone(&production_extensions.inference),
            content_search: Arc::clone(&content_search_extension),
            events: gateway_events.clone(),
            routed_tool_relay: Arc::clone(&routed_tool_relay),
            mcp_lifecycle_relay: Arc::clone(&mcp_lifecycle_relay),
            mcp_service: Arc::clone(&mcp_service),
            data_dir: app_data_dir.clone(),
            request_context: Arc::clone(&runner_request_context),
            session_workers: Arc::clone(&session_workers),
            runner_registry: Arc::clone(&runner_registry),
            ack_obligations: Arc::clone(&ack_obligations),
            agent_deletion_runtime,
            forever_box: Arc::clone(&forever_box),
            teach_recording: Arc::clone(&teach_recording_slot),
            local_exec: Arc::clone(&local_exec_extension),
            session_handoff: session_handoff.clone(),
            webauthn_proxy: Arc::clone(&production_extensions.webauthn_proxy),
            trays: Arc::clone(&production_extensions.trays),
            transcript_runtime: Arc::clone(&transcript_runtime),
            generated_agent_runtime: Arc::clone(&generated_agent_runtime),
            completion_revivals: Arc::clone(&completion_revivals),
            transcript_manager: Arc::clone(&transcript_manager),
            telemetry_logs: host_telemetry.logs.clone(),
            production_action_auditor: production_extensions.action_audit.clone(),
            cloud_agents: production_extensions.cloud_agents.service(),
            cloud_agent_watches: Arc::clone(&cloud_agent_watches),
            secrets: Arc::clone(&secrets_extension),
            local_tool_permission: Arc::clone(&local_tool_permission_extension),
            auto_review: Arc::clone(&auto_review_extension),
            host_runner_composition: Arc::clone(&host_runner_composition),
            box_store_sync: box_store_sync_api.clone(),
            cross_user: Arc::clone(&cross_user),
            host_upgrade: Arc::clone(&host_upgrade),
            automations_lifecycle: Arc::clone(&automations_lifecycle_slot),
            create_agent_nonces: Mutex::new(CreateAgentNonceLedger::default()),
            last_busy_at_ms: Mutex::new(gateway_started_at),
        });
    *host_upgrade_gateway_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::downgrade(&gateway_api);
    *cloud_agent_completion_gateway_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::downgrade(&gateway_api);

    // Re-arm the durable CloudAgent and Subagent subsets through the single
    // frozen PendingWakeRearm state machine. Shell watcher ownership remains
    // explicit manifest work until its shipping owner is composed.
    if let Some(store) = transcript_runtime.pending_wake_store().cloned() {
        let rearm = PendingWakeRearm::new(
            Some(store.clone()),
            Arc::new(ProductionPendingWakeRuntime {
                gateway: Arc::clone(&cloud_agent_completion_gateway_slot),
                cloud_agent_watches: Arc::clone(&cloud_agent_watches),
                completion_revivals: Arc::clone(&completion_revivals),
            }),
        );
        let now_ms = started_at_ms() as f64;
        for pending in store
            .list_pending()
            .into_iter()
            .filter(|marker| {
                matches!(
                    marker.kind,
                    PendingWakeKind::CloudAgent | PendingWakeKind::Subagent
                )
            })
        {
            rearm.rearm_pending_wake(pending, now_ms, Some("host_startup"));
        }
    }

    let listener_runtime = transcript_manager.automation_runtime();
    let listener_lifecycle_slot = Arc::clone(&automations_lifecycle_slot);
    let production_listeners = Arc::new(move || {
        let Ok(entries) = listener_runtime.list_all_automation_definitions() else {
            return (Vec::new(), Vec::new());
        };
        let lifecycle = listener_lifecycle_slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade();
        let mut slack = Vec::new();
        let mut github = Vec::new();
        for entry in entries.into_iter().filter(|entry| entry.automation.is_enabled) {
            let cloud = ScheduledCloudAutomation {
                id: entry.automation.id.clone(),
                name: entry.automation.name.clone(),
                prompt: entry.automation.prompt.clone(),
                is_enabled: entry.automation.is_enabled,
                trigger: entry.automation.trigger.clone(),
            };
            let should_schedule_locally = lifecycle
                .as_ref()
                .map(|owner| owner.should_schedule_locally(&entry.agent_id, &cloud))
                .unwrap_or_else(|| !is_server_schedulable(&entry.automation.trigger));
            if !should_schedule_locally {
                continue;
            }
            for listener in trigger_members(&entry.automation.trigger) {
                match listener.get("type").and_then(serde_json::Value::as_str) {
                    Some("slack") => slack.push(listener),
                    Some("github") => github.push(listener),
                    _ => {}
                }
            }
        }
        (slack, github)
    });
    let listener_gateway = Arc::clone(&gateway_api);
    let relay_sink = Arc::new(move |event: serde_json::Value| {
        listener_gateway.dispatch_production_listener_event(event)
    });
    let fire_gateway = Arc::clone(&gateway_api);
    let fire_dispatch = Arc::new(move |fire, completion| {
        fire_gateway.dispatch_production_backend_fire(fire, completion)
    });
    let resume_gateway = Arc::clone(&gateway_api);
    let on_listener_connected = Arc::new(move |agent_id: &str, platform: &str| {
        if let Err(error) = resume_gateway.resume_after_listener_connect(agent_id, platform) {
            eprintln!(
                "[sand:automations] listener reconnect resume failed agent={agent_id} platform={platform}: {error}"
            );
        }
    });
    let listener_session_workers = Arc::clone(&session_workers);
    let listener_agent_channels = Arc::new(move |agent_id: &str| {
        listener_session_workers
            .list_agent_channels(agent_id)?
            .into_iter()
            .map(|connection| serde_json::to_value(connection).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()
    });
    let cloud_runtime = transcript_manager.automation_runtime();
    let cloud_definitions = Arc::new(move || {
        cloud_runtime
            .list_all_automation_definitions()
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|entry| {
                        (
                            entry.agent_id,
                            ScheduledCloudAutomation {
                                id: entry.automation.id,
                                name: entry.automation.name,
                                prompt: entry.automation.prompt,
                                is_enabled: entry.automation.is_enabled,
                                trigger: entry.automation.trigger,
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
    });
    let cloud_agent_sessions = Arc::clone(&session_workers);
    let cloud_agent_ids = Arc::new(move || cloud_agent_sessions.list_agent_record_ids());
    let cloud_time_zone_sessions = Arc::clone(&session_workers);
    let cloud_time_zone = Arc::new(move || cloud_time_zone_sessions.resolve_user_time_zone());

    let automations_lifecycle = match production_extensions.start_automations(
        production_listeners,
        relay_sink,
        fire_dispatch,
        on_listener_connected,
        listener_agent_channels,
        cloud_definitions,
        cloud_agent_ids,
        cloud_time_zone,
        Arc::new(|message| eprintln!("{message}")),
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to start production Automations extension: {error}");
            return;
        }
    };
    *gateway_api
        .automations_lifecycle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
        Arc::downgrade(&automations_lifecycle);
    gateway_api.refresh_production_automations();

    let teach_runtime: Arc<dyn TeachRecordingRuntimePort> =
        Arc::new(ProductionTeachRecordingRuntime::new(Arc::clone(&forever_box)));
    let teach_factory = SandTeachRecordingServiceFactory::new(teach_runtime);
    let teach_experiments = Arc::clone(&production_extensions.experiments);
    let teach_sessions = Arc::clone(&session_workers);
    let teach_managed_setup = Arc::clone(&production_extensions.managed_setup);
    let teach_analytics_started = host_telemetry.analytics.clone();
    let teach_analytics_stopped = host_telemetry.analytics.clone();
    let teach_logs_cap = host_telemetry.logs.clone();
    let teach_logs_start = host_telemetry.logs.clone();
    let teach_gateway = Arc::downgrade(&gateway_api);
    let teach_deps = TeachRecordingServiceDeps {
        is_enabled: Arc::new(move || teach_experiments.check_feature_gate("sand_teach_by_demonstration")),
        cap_delay_ms: SAND_TEACH_MAX_DURATION_MS + CAP_SLACK_MS,
        send_learning_prompt: Arc::new(move |agent_id, content, client_nonce, rich_text| {
            let gateway = teach_gateway.upgrade()
                .ok_or_else(|| "teach-recording gateway is unavailable".to_string())?;
            let mut args = serde_json::json!({
                "agentId": agent_id,
                "prompt": content,
                "clientNonce": client_nonce,
                "directAddressedAcceptance": true,
                "awaitTurn": false,
            });
            if let Some(rich_text) = rich_text {
                args["richText"] = serde_json::Value::String(rich_text.to_string());
            }
            gateway.call("sendPrompt", args).map(|_| ()).map_err(|error| error.to_string())
        }),
        list_agent_ids: Arc::new(move || teach_sessions.list_agent_record_ids()),
        queue_signature_key: Arc::new(|| load_teach_queue_key(None)),
        ensure_learning_workflow: Arc::new(move || Ok(teach_managed_setup.ensure_managed_skill(LEARN_SKILL_NAME))),
        track_recording_started: Arc::new(move |event| {
            let _ = teach_analytics_started.track_event("sand.teach.recording_started", &event);
        }),
        track_recording_stopped: Arc::new(move |event| {
            let _ = teach_analytics_stopped.track_event("sand.teach.recording_stopped", &event);
        }),
        report_cap_stop_failed: Arc::new(move |event| {
            let _ = teach_logs_cap.report_teach_recording_cap_stop_failed(&event);
        }),
        report_start_failed: Arc::new(move |event| {
            let _ = teach_logs_start.report_teach_recording_start_failed(&event);
        }),
    };
    let teach_recording_extension = create_teach_recording_extension(&teach_factory, teach_deps);
    *teach_recording_slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
        Some(teach_recording_extension.api());

    let gateway_server = match start_gateway_server(GatewayServerDeps {
        api: gateway_api.clone(),
        events: gateway_events.clone(),
        local_exec: Some(local_exec_extension.gateway_bridge()),
        webauthn: Some(production_extensions.webauthn_proxy.gateway_bridge()),
        config: gateway_config.clone(),
        started_at: gateway_started_at,
    }) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("failed to start Mahayana Host gateway: {error}");
            return;
        }
    };

    let teach_recovery_service = Arc::clone(teach_recording_extension.service());
    let _teach_recovery_worker = thread::Builder::new()
        .name("mahayana-teach-recording-recovery".into())
        .spawn(move || {
            if let Err(error) = teach_recovery_service.recover_pending() {
                eprintln!("teach-recording: pending delivery recovery failed: {error}");
            }
        });

    let ack_redrive_stop = Arc::new(AtomicBool::new(false));

    let gateway_discovery_path = get_gateway_discovery_path();
    let gateway_discovery = GatewayDiscoveryInfo {
        port: gateway_server.port(),
        pid: std::process::id(),
        started_at: gateway_started_at,
        scheme: Some(gateway_scheme(&gateway_config).to_string()),
        host: Some(gateway_config.host.clone()),
        token: gateway_config.auth_token.clone(),
    };
    if let Err(error) = write_gateway_discovery(&gateway_discovery, &gateway_discovery_path) {
        eprintln!(
            "failed to publish Mahayana Host gateway discovery at {}: {error}",
            gateway_discovery_path.display()
        );
        return;
    }

    let ready_boot_id = std::env::var("SAND_BOX_BOOT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let ready_boot_started_at_ms = std::env::var("SAND_BOX_BOOT_STARTED_AT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());
    let ready_marker = fs::read_to_string(BOX_READY_STAGE_MARKER_PATH).ok();
    if should_report_box_ready(
        ready_boot_id.as_deref(),
        ready_boot_started_at_ms,
        ready_marker.as_deref(),
    ) {
        if let (Some(boot_id), Some(boot_started_at_ms)) =
            (ready_boot_id, ready_boot_started_at_ms)
        {
            let ready_logs = host_telemetry.logs.clone();
            let _box_ready_worker = thread::spawn(move || {
                let duration_ms = box_ready_duration_ms(started_at_ms(), boot_started_at_ms);
                for attempt in 1..=BOX_READY_REPORT_ATTEMPTS {
                    if ready_logs.report_box_boot_stage_confirmed("ready", duration_ms) {
                        let _ = fs::write(BOX_READY_STAGE_MARKER_PATH, &boot_id);
                        return;
                    }
                    if attempt < BOX_READY_REPORT_ATTEMPTS {
                        thread::sleep(Duration::from_millis(BOX_READY_REPORT_RETRY_MS));
                    }
                }
            });
        }
    }

    let shutdown_complete = Arc::new(AtomicBool::new(false));
    let _shutdown_signal_worker = match install_shutdown_signal_worker(
        host_tx.clone(),
        Arc::clone(&shutdown_complete),
    ) {
        Ok(worker) => worker,
        Err(error) => {
            eprintln!("failed to install Mahayana Host shutdown signal handlers: {error}");
            return;
        }
    };

    production_extensions.notify_bus.mark_background_work_ready();
    if let Err(error) =
        cross_user.start_background_work(Arc::clone(&production_extensions.experiments))
    {
        eprintln!("[sand-host] CrossUserSharing background work failed: {error}");
    }
    let _ = local_tool_permission_extension.background_work_ready();

    if let Err(error) = host_upgrade.service().resume_interrupted_upgrade_turns() {
        eprintln!("mahayana-host upgrade_resume_boot_failed error={error}");
    }
    let _ack_redrive_worker = match start_ack_redrive_worker(
        Arc::clone(&gateway_api),
        Arc::clone(&ack_obligations),
        Arc::clone(&session_workers),
        Arc::clone(&runner_registry),
        gateway_events.clone(),
        Arc::clone(&ack_redrive_stop),
    ) {
        Ok(worker) => Some(worker),
        Err(error) => {
            eprintln!("failed to start Mahayana ack-redrive worker: {error}");
            None
        }
    };

    // Runtime events travel as unsolicited JSON frames. The event worker blocks
    // in Rust instead of issuing 500 ms JSON-RPC receive requests from Electron.
    // Test mode has a non-blocking deterministic backend, so a small sleep keeps
    // that lane from spinning while CI is idle.
    let event_source = host.feature_event_source();
    let event_stdout = Arc::clone(&stdout);
    let event_gateway = gateway_events.clone();
    let _event_worker = thread::spawn(move || loop {
        match event_source.receive(Duration::from_secs(30)) {
            Ok(Some(event)) => {
                let event = event_source.project_grok_gateway_event(&event);
                if write_runtime_event(&event_stdout, &event_gateway, event).is_err() {
                    break;
                }
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                eprintln!("Mahayana runtime event stream failed: {error}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    let platform_stdout = Arc::clone(&stdout);
    let (platform_tx, platform_rx) = mpsc::channel::<String>();
    let _platform_worker = thread::spawn(move || {
        while let Ok(line) = platform_rx.recv() {
            let response = platform_host.dispatch_json(&line);
            if write_response(&platform_stdout, &response).is_err() {
                break;
            }
        }
    });

    // Read stdin on a lightweight transport thread. The UnifiedAppHost itself
    // stays on this owner thread so QuickJS and the rest of the Host runtime
    // never cross a Send/Sync boundary. Gateway requests join the same serial
    // lane through HostLaneRequest.
    let stdin_tx = host_tx.clone();
    let _stdin_worker = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    eprintln!("failed to read host request: {error}");
                    break;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            if stdin_tx.send(HostLaneRequest::Stdin(line)).is_err() {
                return;
            }
        }
        let _ = stdin_tx.send(HostLaneRequest::StdinClosed);
    });
    drop(host_tx);

    while let Ok(request) = host_rx.recv() {
        match request {
            HostLaneRequest::Gateway { method, args, reply } => {
                let _ = reply.send(dispatch_gateway_call(&host, &forever_box, &attachments_service, &method, args));
            }
            HostLaneRequest::StdinClosed => break,
            HostLaneRequest::Stdin(line) => {
                if is_platform_request_json(&line) {
                    if platform_tx.send(line).is_err() {
                        break;
                    }
                    continue;
                }
                let response = dispatch_json(&host, &line);
                if write_response(&stdout, &response).is_err() {
                    break;
                }
                // Commands may enqueue product-local events that are not backed
                // by the model runtime receiver. Drain those immediately so
                // they are pushed in the same turn.
                if drain_ready_runtime_events(&host, &stdout, &gateway_events).is_err() {
                    break;
                }
            }
        }
    }
    drop(platform_tx);
    ack_redrive_stop.store(true, Ordering::Release);
    drop(gateway_server);
    runner_registry.cancel_all("Mahayana Host shutting down");
    routed_tool_relay.cancel_all("Mahayana Host shutting down");
    mcp_lifecycle_relay.cancel_all("Mahayana Host shutting down");
    if let Err(error) = production_extensions.stop_box_store_sync() {
        eprintln!("failed to stop production BoxStoreSync extension cleanly: {error}");
    }
    if let Err(error) = production_extensions.stop_mcp() {
        eprintln!("failed to stop production MCP extension cleanly: {error}");
    }
    if let Err(error) = production_extensions.stop_automations() {
        eprintln!("failed to stop production Automations extension cleanly: {error}");
    }
    cross_user.stop();
    production_extensions.notify_bus.stop();
    session_extension.shutdown();
    drop(teach_recording_extension);
    box_extensions.stop();
    if let Some(daemon) = box_exec_daemon.as_mut() {
        if let Err(error) = daemon.close() {
            eprintln!("failed to stop managed Grok box exec-daemon cleanly: {error}");
        }
    }
    if let Err(error) = clear_gateway_discovery(&gateway_discovery_path) {
        eprintln!(
            "failed to clear Mahayana Host gateway discovery at {}: {error}",
            gateway_discovery_path.display()
        );
    }
    if let Err(error) = host_lock.release() {
        eprintln!(
            "failed to release Mahayana Host lock at {}: {error}",
            host_lock_path.display()
        );
    }
    shutdown_complete.store(true, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::{
        BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, ProductionBrowserUaLog,
        ProductionHostExtensions, ProductionRunnerRequestContextSource, UnifiedGatewayApi,
        decode_provider_messages,
        dispatch_box_environment_call, ensure_managed_runtime_layout, is_platform_request_json,
        AutomationExecutionResult, automation_fire_completion, automation_terminal_from_event,
        listener_connect_resume_args,
        project_forever_box_status, reaction_gateway_args,
    };
    use mahayana_host_runtime::extensions::forever_box::BoxStatus;
    use mahayana_host_runtime::extensions::session::box_handoff_service::PendingHandoff;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn production_browser_ua_log_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ProductionBrowserUaLog>();
    }

    #[test]
    fn shipping_production_extension_graph_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ProductionHostExtensions>();
        assert_send_sync::<ProductionSessionWorkers>();
        assert_send_sync::<TranscriptRunnerRegistry>();
    }

    #[test]
    fn gateway_proxy_is_send_sync_without_moving_the_unified_host() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UnifiedGatewayApi>();
        assert_send_sync::<ProductionRunnerRequestContextSource>();
    }

    #[test]
    fn routes_platform_http_away_from_the_feature_event_lane() {
        assert!(is_platform_request_json(
            r#"{"id":1,"method":"platform.request","params":{}}"#,
        ));
        assert!(!is_platform_request_json(
            r#"{"id":2,"method":"feature.receive","params":{"timeoutMs":500}}"#,
        ));
        assert!(!is_platform_request_json(
            r#"{"id":3,"method":"feature.execute","params":{}}"#,
        ));
    }



    #[test]
    fn listener_integration_status_errors_degrade_to_disconnected() {
        assert!(listener_connection_state_or_disconnected(
            "github",
            Ok(true)
        ));
        assert!(!listener_connection_state_or_disconnected(
            "slack",
            Err("temporary auth failure".into())
        ));
    }

    #[test]
    fn listener_gateway_methods_are_owned_by_production_automations_lifecycle() {
        let source = include_str!("main.rs");
        assert!(source.contains("if method == \"getListenerIntegrations\""));
        assert!(source.contains("if method == \"getListenerConnectUrl\""));
        assert!(source.contains("lifecycle.is_platform_connected(platform)"));
        assert!(source.contains("lifecycle.get_connect_url(platform)"));
        assert!(
            !source.contains("watch_listener_connection(entry.agent_id.clone(), platform)"),
            "existing routines must not implicitly arm the interactive connect watcher"
        );
    }

    #[test]
    fn listener_connect_resume_is_hidden_handoff_with_stable_nonce() {
        let args = listener_connect_resume_args("agent-1", "slack", "nonce-123");
        assert_eq!(args["agentId"], "agent-1");
        assert_eq!(args["clientNonce"], "nonce-123");
        assert_eq!(args["requestSource"], "handoff-resume");
        assert_eq!(args["appendUserMessage"], false);
        assert_eq!(args["hidden"], true);
        assert!(
            args["prompt"]
                .as_str()
                .expect("resume prompt")
                .contains("bot is invited")
        );
    }

    #[test]
    fn backend_fire_completion_preserves_retryable_none_and_terminal_outcomes() {
        assert_eq!(automation_fire_completion(Ok(None)), None);
        assert_eq!(
            automation_fire_completion(Ok(Some(
                mahayana_host_runtime::extensions::transcript::automation_run_path::FireAutomationOutcome::Ok
            )))
            .expect("success completion")
            .status,
            "succeeded"
        );
        assert_eq!(
            automation_fire_completion(Err("runner failed".into()))
                .expect("failed completion")
                .status,
            "failed"
        );
    }

    #[test]
    fn runner_provider_gateway_rejects_empty_message_batches() {
        assert!(decode_provider_messages(&serde_json::json!({"messages": []})).is_err());
        let messages = decode_provider_messages(&serde_json::json!({
            "messages": [{"role":"user","content":"hello"}]
        }))
        .expect("provider messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "hello");
    }

    #[test]
    fn automation_terminal_event_maps_runner_settlement_without_guessing() {
        let completed = serde_json::json!({
            "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
            "payload": {"streamId":"automation-1","type":"completed","content":""}
        });
        assert_eq!(
            automation_terminal_from_event(&completed, "automation-1"),
            Some(Ok(AutomationExecutionResult::Completed))
        );

        let cancelled = serde_json::json!({
            "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
            "payload": {"streamId":"automation-1","type":"cancelled","message":"stop"}
        });
        assert_eq!(
            automation_terminal_from_event(&cancelled, "automation-1"),
            Some(Ok(AutomationExecutionResult::Interrupted {
                detail: "Interrupted before it finished.".into()
            }))
        );

        let failed = serde_json::json!({
            "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
            "payload": {"streamId":"automation-1","type":"failed","message":"provider failed"}
        });
        assert_eq!(
            automation_terminal_from_event(&failed, "automation-1"),
            Some(Err("provider failed".into()))
        );

        let other = serde_json::json!({
            "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
            "payload": {"streamId":"automation-2","type":"completed"}
        });
        assert_eq!(automation_terminal_from_event(&other, "automation-1"), None);
    }

    #[test]
    fn reaction_sink_uses_authoritative_host_gateway_payload() {
        assert_eq!(
            reaction_gateway_args("agent-a", "t3u", "👍"),
            serde_json::json!({
                "agentId": "agent-a",
                "entryId": "t3u",
                "emoji": "👍"
            })
        );
    }

    #[test]
    fn forever_box_status_projection_carries_pending_handoff_for_renderer() {
        let status = BoxStatus {
            agent_id: "agent-a".into(),
            state: "running".into(),
            vnc_url: Some("http://127.0.0.1/vnc.html".into()),
            windows: None,
            image_update_available: Some(true),
            pull_percent: None,
        };
        let handoff = PendingHandoff {
            request_id: "request-a".into(),
            instruction: "Sign in".into(),
            snapshot_data_url: Some("data:image/webp;base64,YWJj".into()),
        };
        let projected = project_forever_box_status(&status, Some(&handoff));
        assert_eq!(projected["agentId"], "agent-a");
        assert_eq!(projected["handoff"]["requestId"], "request-a");
        assert_eq!(
            projected["handoff"]["snapshotDataUrl"],
            "data:image/webp;base64,YWJj"
        );
    }

    #[test]
    fn shipping_gateway_routes_box_environment_to_production_box_owner() {
        let args = serde_json::json!({
            "env": { "FABUSHI_AGENT": "enabled", "SHELL": "/bin/zsh" },
            "replace": true
        });
        let mut observed = None;
        let result = dispatch_box_environment_call(
            BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD,
            &args,
            |update| {
                observed = Some(update.clone());
                Ok(())
            },
        )
        .expect("box route should be owned by production Host")
        .expect("box route should succeed");

        assert_eq!(result, serde_json::json!({ "applied": true }));
        let observed = observed.expect("production box update");
        assert_eq!(observed.env["FABUSHI_AGENT"], "enabled");
        assert_eq!(observed.env["SHELL"], "/bin/zsh");
        assert!(observed.replace);
    }

    #[test]
    fn creates_product_owned_fallback_workspace_before_host_startup() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "fabushi-mahayana-desktop-layout-{}-{suffix}",
            std::process::id()
        ));

        ensure_managed_runtime_layout(&root).expect("initialize managed runtime layout");
        assert!(root.join("feature-host/runtime/workspace").is_dir());

        fs::remove_dir_all(root).expect("remove test layout");
    }
}
