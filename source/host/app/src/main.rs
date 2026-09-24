//! Grok-aligned Mahayana Host process entrypoint.
//!
//! The shipping desktop Host process is owned by `source/host/app`. During the
//! migration, `mahayana-unified-app-host` remains an internal compatibility
//! backend so existing product commands keep working while Grok Host/Runner
//! modules are moved behind this process boundary. Electron must never launch
//! the legacy third_party desktop Host binary directly.

use mahayana_host_runtime::extensions::auth::auth_service::HostAuthServiceOptions;
use mahayana_host_runtime::extensions::auth::extension::{
    HostAuthExtension, start_host_auth_extension_with_options,
};
use mahayana_host_runtime::extensions::auth::user_full_name_service::production_user_full_name_fetch;
use mahayana_host_runtime::extensions::experiments::{
    HostExperimentsExtension, start_host_experiments_extension,
};
use mahayana_host_runtime::extensions::box_lifecycle::box_lifecycle_service::BoxLifecycleService;
use mahayana_host_runtime::extensions::box_lifecycle::extension::start_box_lifecycle_extension;
use mahayana_host_runtime::extensions::box_lifecycle::production::{
    ProductionBoxLifecycleClient, ProductionBoxLifecycleClientFactory,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::memory::extension::HostMemoryExtension;
use mahayana_host_runtime::extensions::memory::production::start_production_memory_extension;
use mahayana_host_runtime::extensions::session::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffDecision, HandoffTrigger, PendingHandoff,
    ScreenshotPayload, decide_box_hand_back,
};
use mahayana_host_runtime::extensions::session::extension::start_session_extension;
use mahayana_host_runtime::extensions::settings::extension::start_settings_extension;
use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
    persist_accepted_send_prompt_context,
};
use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;
use mahayana_host_runtime::extensions::transcript::extension::start_transcript_extension;
use mahayana_host_runtime::extensions::transcript::send_message_shaping::shape_send_prompt_media_args;
use mahayana_host_runtime::extensions::transcript::box_handoff_resume::{
    build_box_handoff_resume_send_args, settle_box_handoff_state,
};
use mahayana_host_runtime::extensions::transcript::ack_obligations::{
    AckObligations, AckRedrivePreparation, build_ack_redrive_send_args,
};
use mahayana_host_runtime::extensions::transcript::runner_registry::TranscriptRunnerRegistry;
use mahayana_host_runtime::extensions::transcript::run_scheduler::WatchdogStage;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::{
    AgentDeletionRuntimeDeps, AgentLifecycleGatewayError,
    dispatch_production_agent_lifecycle_gateway_call_with_runtime,
};
use mahayana_host_runtime::extensions::source_map::extension::start_source_map_extension;
use mahayana_host_runtime::extensions::source_map::source_map_service::SandSourceMap;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::managed_setup::team_rules::ProductionTeamRulesResolver;
use mahayana_host_runtime::extensions::webauthn_proxy::extension::{
    HostWebAuthnProxyExtension, start_webauthn_proxy_extension,
};
use mahayana_host_runtime::extensions::telemetry::webauthn_proxy_telemetry::{
    WebAuthnProxyReport, webauthn_proxy_telemetry,
};
use mahayana_host_runtime::extensions::telemetry::queue_telemetry_mappers::{
    QueueAcceptedReport, QueueDequeuedReport, QueueWatchdogReport,
    queue_accepted_telemetry, queue_dequeued_telemetry, queue_watchdog_telemetry,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostStructuredLogTelemetry;
use mahayana_host_runtime::extensions::telemetry::extension::start_host_telemetry_extension;
use mahayana_host_runtime::extensions::trays::extension::{
    HostTraysExtension, start_trays_extension,
};
use mahayana_host_runtime::extensions::transcript::agent_run_error::provider_failure_tray;
use mahayana_host_runtime::extensions::forever_box::{
    ForeverBoxExtensionOptions, ForeverBoxLifecycle, ForeverBoxRunnerResourcePort,
    BoxStatus, ForeverBoxService, start_forever_box_extension,
};
use mahayana_host_runtime::extensions::auth::credential_renewer::RenewalOutcome;
use mahayana_host_runtime::extensions::browser_ua::{
    BrowserUaExtensionRuntime, BrowserUaHostLog, start_browser_ua_extension,
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
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::ProviderRetryEvent;
use mahayana_host_runtime::runner::production_turn_input_projection::create_production_turn_input_projection;
use mahayana_host_runtime::runner_production_bridge::{
    ProductionActionAuditInput, ProductionRunnerCompositionInput,
    create_production_runner_composition,
};
use mahayana_host_runtime::runner::sand_action_audit::{
    ActionAuditRecord, ActionAuditSink,
};
use mahayana_host_runtime::runner::turn_observation::{
    TurnObservation, TurnObservationHandle,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedToolBridge, RunnerRequestContextSource,
};
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, ROUTED_TOOL_EXECUTE_METHOD, ROUTED_TOOL_LIST_METHOD,
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD,
};
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::attachment_paths::{
    AgentMediaKind, file_url_for_path, persist_agent_media_bytes,
};
use mahayana_host_runtime::runner::tools::send_message_encoding::{
    image_mime_from_path, resolve_box_media_attachment,
};
use mahayana_host_runtime::runner::tools::send_message_tool::{
    ResolvedAttachmentSource, SendMessageSink, file_path_from_file_url,
};
use mahayana_host_runtime::runner::tools::sand_reaction_tool::ReactionSink;
use mahayana_host_runtime::gateway_config::{gateway_scheme, resolve_gateway_server_config};
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayCommandReport, GatewayEventHub, GatewayServerDeps, start_gateway_server,
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
    Arc, Mutex, mpsc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ensure_managed_runtime_layout(app_data_dir: &Path) -> io::Result<()> {
    // The desktop product owns this fallback workspace.  It must exist before
    // the native engine canonicalizes the path while opening the first Agent
    // session.  User-selected workspace paths are validated elsewhere and are
    // never created implicitly.
    fs::create_dir_all(app_data_dir.join("feature-host/runtime/workspace"))
}


struct ProductionBrowserUaLog;

impl BrowserUaHostLog for ProductionBrowserUaLog {
    fn log(&self, message: &str) {
        eprintln!("mahayana-host-browser-ua {message}");
    }
}

fn start_production_browser_ua(
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
) -> BrowserUaExtensionRuntime {
    start_browser_ua_extension(
        auth,
        experiments,
        Arc::new(ProductionBrowserUaLog),
        None,
        None,
    )
}

struct ProductionHostExtensions {
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    memory: HostMemoryExtension,
    team_rules: Arc<ProductionTeamRulesResolver>,
    team_rules_renewal_subscription: Option<u64>,
    source_map: Arc<SandSourceMap>,
    trays: Arc<HostTraysExtension>,
    box_lifecycle: Arc<BoxLifecycleService<ProductionBoxLifecycleClient<HostAuthExtension>>>,
    webauthn_proxy: Arc<HostWebAuthnProxyExtension>,
}

impl Drop for ProductionHostExtensions {
    fn drop(&mut self) {
        if let Some(subscription) = self.team_rules_renewal_subscription.take() {
            self.auth.service().unsubscribe_from_renewal(subscription);
        }
    }
}

fn start_production_host_extensions() -> Result<ProductionHostExtensions, String> {
    let auth_options = HostAuthServiceOptions::production(|message| {
        eprintln!("mahayana-host-auth {message}");
    })
    .map_err(|error| error.to_string())?;

    let backend_url = auth_options
        .backend_url
        .clone()
        .ok_or_else(|| "production Auth requires a configured backend URL".to_string())?;
    let auth = Arc::new(
        start_host_auth_extension_with_options(
            auth_options,
            production_user_full_name_fetch(backend_url.clone()),
        )
        .map_err(|error| error.to_string())?,
    );
    let experiments = Arc::new(start_host_experiments_extension());
    let memory = start_production_memory_extension();
    let team_rules = Arc::new(ProductionTeamRulesResolver::new(
        backend_url,
        Arc::clone(&auth),
    ));
    team_rules.preload();
    let weak_team_rules = Arc::downgrade(&team_rules);
    let team_rules_renewal_subscription =
        auth.service().subscribe_to_renewal(Arc::new(move |event| {
            if event.result.outcome != RenewalOutcome::Renewed {
                return;
            }
            let Some(resolver) = weak_team_rules.upgrade() else {
                return;
            };
            let _ = thread::Builder::new()
                .name("host-managed-team-rules-renewal".into())
                .spawn(move || {
                    let _ = resolver.refresh();
                });
        }));

    let source_map = Arc::new(start_source_map_extension());

    let trays = Arc::new(start_trays_extension());

    let factory =
        ProductionBoxLifecycleClientFactory::from_process_env().map_err(|error| error.to_string())?;
    let box_lifecycle = Arc::new(start_box_lifecycle_extension(Arc::clone(&auth), &factory));
    let webauthn_proxy = Arc::new(start_webauthn_proxy_extension(Arc::new(
        |report: WebAuthnProxyReport| {
            let projection = webauthn_proxy_telemetry(&report);
            eprintln!(
                "mahayana-host-webauthn level={} event={} metadata={}",
                projection.level.unwrap_or("info"),
                projection.event.unwrap_or("sand.webauthn_proxy"),
                serde_json::to_string(&projection.metadata).unwrap_or_else(|_| "{}".into()),
            );
        },
    )));

    Ok(ProductionHostExtensions {
        auth,
        experiments,
        memory,
        team_rules,
        team_rules_renewal_subscription: Some(team_rules_renewal_subscription),
        source_map,
        trays,
        box_lifecycle,
        webauthn_proxy,
    })
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

struct ProductionSendMessageSink {
    sessions: Arc<ProductionSessionWorkers>,
    forever_box: Arc<ForeverBoxService>,
    ack_obligations: Arc<AckObligations>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    ack_token: Option<String>,
    agent_id: String,
}

impl ProductionSendMessageSink {
    fn persist_media_bytes(
        &self,
        source_name: &str,
        bytes: &[u8],
        kind: AgentMediaKind,
    ) -> Option<String> {
        let db_path = self.sessions.session_db_path(&self.agent_id).ok()?;
        let agent_dir = db_path.parent()?;
        let path = persist_agent_media_bytes(agent_dir, source_name, bytes, kind).ok()?;
        file_url_for_path(path)
    }
}

impl SendMessageSink for ProductionSendMessageSink {
    fn resolve_attachment_source(
        &self,
        source_url: &str,
        _tool_call_id: &str,
    ) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        let Some(source_path) = file_path_from_file_url(source_url) else {
            return Ok(ResolvedAttachmentSource {
                url: source_url.to_string(),
                file_name: None,
            });
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
            if let Some(url) = self.persist_media_bytes(&source_path_text, &bytes, kind) {
                return Ok(ResolvedAttachmentSource { url, file_name });
            }
        }

        let status = self.forever_box.get_status(&self.agent_id);
        let remote_box_has_desktop = status.vnc_url.is_some()
            || status.windows.as_ref().is_some_and(|windows| !windows.is_empty());
        let resolved = resolve_box_media_attachment(
            &source_path_text,
            remote_box_has_desktop,
            |path| self.forever_box.box_().download_file(&self.agent_id, path).ok(),
            |bytes, _mime| self.persist_media_bytes(
                &source_path_text,
                bytes,
                AgentMediaKind::Image,
            ),
            |name, bytes| self.persist_media_bytes(
                name,
                bytes,
                AgentMediaKind::Attachment,
            ),
        )
        .unwrap_or_else(|| source_url.to_string());
        Ok(ResolvedAttachmentSource { url: resolved, file_name })
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
    experiments: Arc<HostExperimentsExtension>,
    events: GatewayEventHub,
    routed_tool_relay: Arc<CoordinatorToolRelay>,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    agent_deletion_runtime: AgentDeletionRuntimeDeps,
    forever_box: Arc<ForeverBoxService>,
    session_handoff: BoxHandoffService,
    webauthn_proxy: Arc<HostWebAuthnProxyExtension>,
    trays: Arc<HostTraysExtension>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    transcript_manager: Arc<TranscriptManager>,
    telemetry_logs: HostStructuredLogTelemetry,
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

#[derive(Clone)]
struct CoordinatorRoutedToolBridge {
    relay: Arc<CoordinatorToolRelay>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    agent_id: String,
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

fn start_routed_provider_task(
    routed_tool_relay: Arc<CoordinatorToolRelay>,
    events: GatewayEventHub,
    host_tx: mpsc::Sender<HostLaneRequest>,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    experiments: Arc<HostExperimentsExtension>,
    session_workers: Arc<ProductionSessionWorkers>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    forever_box: Arc<ForeverBoxService>,
    trays: Arc<HostTraysExtension>,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    let provider_name = args.get("provider").and_then(serde_json::Value::as_str).unwrap_or("");
    let provider = RoutedProvider::parse(provider_name)
        .filter(|provider| *provider != RoutedProvider::Cursor)
        .ok_or_else(|| GatewayCommandError::Internal(format!(
            "unsupported routed provider: {provider_name}"
        )))?;
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
    transcript_runtime
        .require_routed_turn_lease(&agent_id, &stream_id)
        .map_err(map_production_send_error)?;
    let routed_turn_lease_guard = RoutedTurnLeaseGuard::new(
        Arc::clone(&transcript_runtime),
        agent_id.clone(),
        stream_id.clone(),
        events.clone(),
    );
    let messages = decode_provider_messages(&args)?;
    let turn_input = create_production_turn_input_projection(
        &args,
        &stream_id,
        &messages,
    )
    .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
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

    transcript_runtime.begin_provider_run(&agent_id);
    let ack_token = ack_obligations
        .mint_ack_run_token(&agent_id)
        .map_err(|error| {
            transcript_runtime.end_provider_run(&agent_id);
            let _ = transcript_runtime.retire_idle_live_session(
                &session_workers,
                &agent_id,
            );
            GatewayCommandError::Internal(format!(
                "could not mint ack run token for {agent_id}: {error}"
            ))
        })?;
    let cancellation = runner_registry
        .register_routed_provider(&agent_id, &stream_id)
        .map_err(|error| {
            ack_obligations.retire_ack_run_token(&agent_id, ack_token.as_deref());
            transcript_runtime.end_provider_run(&agent_id);
            let _ = transcript_runtime.retire_idle_live_session(
                &session_workers,
                &agent_id,
            );
            GatewayCommandError::Internal(error.to_string())
        })?;
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
    let worker_cancellation = cancellation.clone();
    let worker_sessions = Arc::clone(&session_workers);
    let worker_retire_sessions = Arc::clone(&session_workers);
    let worker_ack_obligations = Arc::clone(&ack_obligations);
    let worker_transcript_runtime = Arc::clone(&transcript_runtime);
    let worker_ack_token = ack_token.clone();
    let worker_trays = Arc::clone(&trays);
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
            if let Ok(mut observation) = observation.lock() {
                observation.turn_started(started_at_ms());
            }
            let bridge: Arc<dyn RoutedToolBridge> = Arc::new(CoordinatorRoutedToolBridge {
                relay: routed_tool_relay,
                transcript_runtime: Arc::clone(&worker_transcript_runtime),
                agent_id: agent_id.clone(),
            });
            let box_resources = Arc::new(ForeverBoxRunnerResourcePort::new(
                Arc::clone(&forever_box),
                agent_id.clone(),
            ));
            let delta_events = worker_events.clone();
            let delta_stream_id = stream_id.clone();
            let delta_runtime = Arc::clone(&worker_transcript_runtime);
            let delta_agent_id = agent_id.clone();
            let delta_observation = Arc::clone(&observation);
            let mut on_text_delta = move |delta: &str, accumulated: &str| {
                if !delta.is_empty() {
                    if let Ok(mut observation) = delta_observation.lock() {
                        let _ = observation.observe_first_token(
                            "text",
                            None,
                            None,
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
            let send_message_sink: Arc<dyn SendMessageSink> = Arc::new(
                ProductionSendMessageSink {
                    sessions: worker_sessions,
                    forever_box: Arc::clone(&forever_box),
                    ack_obligations: Arc::clone(&worker_ack_obligations),
                    transcript_runtime: Arc::clone(&worker_transcript_runtime),
                    ack_token: worker_ack_token.clone(),
                    agent_id: agent_id.clone(),
                },
            );
            let reaction_sink: Arc<dyn ReactionSink> = Arc::new(
                ProductionReactionSink {
                    host_tx,
                    agent_id: agent_id.clone(),
                },
            );
            let retry_runtime = Arc::clone(&worker_transcript_runtime);
            let retry_agent_id = agent_id.clone();
            let retry_observation = Arc::clone(&observation);
            let retry_sink: Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync> =
                Arc::new(move |event: &ProviderRetryEvent| {
                    retry_runtime.track_runner_activity_update(
                        &retry_agent_id,
                        &ActivityUpdate::Retrying,
                        started_at_ms(),
                    );
                    if let Ok(observation) = retry_observation.lock() {
                        observation.report_turn_retry(serde_json::json!({
                            "attempt": event.attempt,
                            "nextAttempt": event.next_attempt,
                            "delayMs": event.delay_ms,
                            "resumeFromCheckpoint": event.resume_from_checkpoint,
                            "watchdogExpired": event.watchdog_expired,
                        }));
                    }
                });
            let audit_events = worker_events.clone();
            let action_audit_sink: Arc<dyn ActionAuditSink> = Arc::new(
                move |record: ActionAuditRecord| {
                    audit_events.publish(serde_json::json!({
                        "channel": "runner-action-audit",
                        "payload": record,
                    }));
                },
            );
            let composition = create_production_runner_composition(
                ProductionRunnerCompositionInput {
                    provider,
                    bridge,
                    request_context: resolved_request_context,
                    cancellation,
                    checkpoint_store,
                    retry_sink: Some(retry_sink),
                    box_resources: Some(box_resources),
                    send_message_sink: Some(send_message_sink),
                    reaction_sink: Some(reaction_sink),
                    action_audit: Some(ProductionActionAuditInput {
                        agent_id: agent_id.clone(),
                        turn_id: Some(stream_id.clone()),
                        sink: action_audit_sink,
                    }),
                    observation: Some(Arc::clone(&observation)),
                },
            );
            let owner = ProductionTurnAgentOwner::new(composition)
                .with_agent_state_checkpoint_sink(agent_state_checkpoint_sink);
            let mut runner = SandAgentRunner::new(owner);
            let result = runner.run_routed_provider_with_options(
                &data_dir,
                &messages,
                turn_input.options,
                &mut on_text_delta,
            );

            // A terminal inference event is the renderer-visible completion
            // boundary. Do not publish it until the Host has actually settled
            // the run: otherwise the UI can render the final assistant turn
            // while the registry/live session still owns the provider, and an
            // immediate app quit races that cleanup path.
            worker_transcript_runtime.track_runner_activity_update(
                &agent_id,
                &ActivityUpdate::TurnEnded,
                started_at_ms(),
            );
            worker_transcript_runtime.end_provider_run(&agent_id);
            worker_registry.finish_routed_provider(&worker_stream_id);
            worker_ack_obligations.retire_ack_run_token(
                &agent_id,
                worker_ack_token.as_deref(),
            );
            worker_routed_turn_lease.settle();
            let _ = worker_transcript_runtime
                .retire_idle_live_session(&worker_retire_sessions, &agent_id);

            if !worker_cancellation.is_cancelled() {
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
                        worker_trays.push_error(provider_failure_tray(
                            &agent_id,
                            &message,
                            started_at_ms() as i64,
                        ));
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
        transcript_runtime.end_provider_run(&spawn_error_agent_id);
        runner_registry.finish_routed_provider(&accepted_stream_id);
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
        if method == "isAgentNetworkEnabled" {
            return Ok(serde_json::Value::Bool(
                self.experiments.is_agent_network_enabled(),
            ));
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
                        }
                    }
                    "deleteAgents" => {
                        if let Some(ids) = args.get("ids").and_then(serde_json::Value::as_array) {
                            for agent_id in ids.iter().filter_map(serde_json::Value::as_str) {
                                self.transcript_runtime
                                    .session_runtime()
                                    .mark_agent_deleted(agent_id);
                                self.transcript_manager.clear_agent_durable_recovery(agent_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
            return result;
        }
        if let Some(result) =
            dispatch_production_session_gateway_call(&self.session_workers, method, &args)
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
                        self.ack_obligations
                            .record_send(agent_id, started_at_ms() as f64)
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
        if method == RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD {
            return start_routed_provider_task(
                Arc::clone(&self.routed_tool_relay),
                self.events.clone(),
                self.host_tx.clone(),
                self.data_dir.clone(),
                Arc::clone(&self.request_context),
                Arc::clone(&self.experiments),
                Arc::clone(&self.session_workers),
                Arc::clone(&self.runner_registry),
                Arc::clone(&self.ack_obligations),
                Arc::clone(&self.transcript_runtime),
                Arc::clone(&self.forever_box),
                Arc::clone(&self.trays),
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
            let cancelled = self.runner_registry.cancel_stream(stream_id, reason);
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
            let durable_args = args.clone();
            let runner_args = shape_send_prompt_media_args(&args);
            let watchdog_registry = Arc::clone(&self.runner_registry);
            let watchdog_ack_obligations = Arc::clone(&self.ack_obligations);
            let watchdog_events = self.events.clone();
            let watchdog_logs = self.telemetry_logs.clone();
            let accepted_logs = self.telemetry_logs.clone();
            let dequeued_logs = self.telemetry_logs.clone();
            return self
                .transcript_runtime
                .execute_send_with_queue_observers(
                    &durable_args,
                    || {
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
                        if accepted.get("accepted").and_then(serde_json::Value::as_bool)
                            == Some(true)
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
                                    .map_err(ProductionSendError::Internal)?
                                    .is_some_and(|summary| !summary.is_group);
                                if direct_local {
                                    self.ack_obligations
                                        .record_send(agent_id, started_at_ms() as f64)
                                        .map_err(|error| ProductionSendError::Internal(
                                            format!(
                                                "could not record durable ack obligation for {agent_id}: {error}"
                                            )
                                        ))?;
                                }
                            }
                        }
                        Ok(persisted)
                    },
                    move |event| {
                        let interrupted = if event.stage == WatchdogStage::Trip {
                            let interrupted = watchdog_registry
                                .interrupt_wedged_run_for_watchdog(&event.agent_id);
                            if interrupted {
                                let _ = watchdog_ack_obligations.record_interrupt(
                                    &event.agent_id,
                                    started_at_ms() as f64,
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
                )
                .map_err(map_production_send_error);
        }
        call_host_lane(&self.host_tx, method, args)
    }

    fn prepare_for_upgrade(&self) -> Result<serde_json::Value, GatewayCommandError> {
        let summary = self.transcript_runtime.quiesce_for_upgrade();
        Ok(serde_json::json!({
            "quiescing": summary.quiescing,
            "runningTurns": summary.running_turns,
        }))
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        log_gateway_command_report("complete", &report);
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        log_gateway_command_report("error", &report);
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
    method: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
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

    let production_extensions = match start_production_host_extensions() {
        Ok(extensions) => extensions,
        Err(error) => {
            eprintln!("failed to start production Host extensions: {error}");
            return;
        }
    };
    let browser_ua_runtime = start_production_browser_ua(
        Arc::clone(&production_extensions.auth),
        Arc::clone(&production_extensions.experiments),
    );
    let lifecycle: Arc<dyn ForeverBoxLifecycle> =
        production_extensions.box_lifecycle.clone();
    let forever_box = start_forever_box_extension(
        production_box,
        lifecycle,
        ForeverBoxExtensionOptions::from_process_env(),
    );
    let runner_request_context: Arc<dyn RunnerRequestContextSource> =
        Arc::new(ProductionRunnerRequestContextSource::new(
            Arc::clone(&production_extensions.auth),
            Arc::clone(&production_extensions.team_rules),
            app_data_dir.join("transcripts"),
        ));
    let gateway_events = GatewayEventHub::default();
    let settings_extension = start_settings_extension();
    let settings_for_session = Arc::clone(&settings_extension);
    let host_telemetry = match start_host_telemetry_extension(&app_data_dir) {
        Ok(telemetry) => telemetry,
        Err(error) => {
            eprintln!("failed to start Mahayana Host telemetry extension: {error}");
            return;
        }
    };
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
    let runner_registry = transcript_manager.runner_registry();
    let ack_obligations = transcript_manager.ack_obligations();
    let agent_deletion_runtime = AgentDeletionRuntimeDeps {
        cancel_runner: Some({
            let runner_registry = Arc::clone(&runner_registry);
            Arc::new(move |agent_id| {
                let _ = runner_registry.cancel_agent(agent_id, "agent deleted");
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
    let routed_tool_relay = Arc::new(CoordinatorToolRelay::new(gateway_events.clone()));
    let transcript_runtime = transcript_manager.transcript_runtime();
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
    let gateway_api = Arc::new(UnifiedGatewayApi {
            host_tx: host_tx.clone(),
            experiments: Arc::clone(&production_extensions.experiments),
            events: gateway_events.clone(),
            routed_tool_relay: Arc::clone(&routed_tool_relay),
            data_dir: app_data_dir.clone(),
            request_context: runner_request_context,
            session_workers: Arc::clone(&session_workers),
            runner_registry: Arc::clone(&runner_registry),
            ack_obligations: Arc::clone(&ack_obligations),
            agent_deletion_runtime,
            forever_box: Arc::clone(&forever_box),
            session_handoff: session_handoff.clone(),
            webauthn_proxy: Arc::clone(&production_extensions.webauthn_proxy),
            trays: Arc::clone(&production_extensions.trays),
            transcript_runtime: Arc::clone(&transcript_runtime),
            transcript_manager: Arc::clone(&transcript_manager),
            telemetry_logs: host_telemetry.logs.clone(),
        });
    let gateway_server = match start_gateway_server(GatewayServerDeps {
        api: gateway_api.clone(),
        events: gateway_events.clone(),
        local_exec: None,
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

    let ack_redrive_stop = Arc::new(AtomicBool::new(false));
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
                let _ = reply.send(dispatch_gateway_call(&host, &forever_box, &method, args));
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
    session_extension.shutdown();
    browser_ua_runtime.stop();
    forever_box.dispose();
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
}

#[cfg(test)]
mod tests {
    use super::{
        BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, ProductionBrowserUaLog,
        ProductionHostExtensions, ProductionRunnerRequestContextSource, UnifiedGatewayApi,
        decode_provider_messages,
        dispatch_box_environment_call, ensure_managed_runtime_layout, is_platform_request_json,
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
