use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::extensions::auth::auth_service::HostAuthRenewalEvent;
use crate::extensions::auth::credential_renewer::RenewalOutcome;
use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::browser_ua::extension::StopSubscription;
use crate::extensions::experiments::{
    HostExperimentsExtension, pin_experiments_diagnostics_reporter,
};
use crate::extensions::inference::production::ProductionInferenceExtension;
use crate::extensions::inference::sand_model_experiment::SandModelExperimentArm;
use crate::host_paths::get_host_crash_marker_path;
use crate::ports::sand_analytics_types::sand_message_length_bucket;
use crate::ports::telemetry::resolve_sand_box_identity_tags;

use super::HostTelemetryProjection;
use super::analytics_service::{
    AnalyticsClient, AutomationRunAnalyticsTelemetry, ProductionAnalyticsRuntime,
    TelemetryService, product_analytics_event,
};
use super::box_log_ship_telemetry::{BoxLogShipReport, box_log_ship_telemetry};
use super::box_log_shipper::{
    BoxLogShipper, BoxLogShipperConfig, BoxTelemetryRecord, DeliverySettlement,
    SUPERVISOR_LOG_PATH, is_box_log_shipping_enabled,
};
use super::desktop_health_forwarder::{
    DesktopHealthForwardResult, forward_desktop_health_with,
};
use super::event_loop_telemetry::{
    EventLoopTelemetryRuntime, EventLoopTrigger, event_loop_window_telemetry,
};
use super::host_crash_marker::{
    FileHostCrashMarkerStore, ForwardHostCrashMarkerResult, HostCrashMarkerStore,
    forward_host_crash_marker_with, host_crash_marker_metadata,
};
use super::host_lifecycle_progress::{
    HostLifecycleProgress, HostLifecycleReport, WatchdogArm,
};
use super::host_tracing::HostTracing;
use super::model_experiment_exposure::{
    ModelExperimentExposureAnalytics, ModelExperimentExposureExperiments,
    ModelExperimentExposureLatch, SandModelExperimentState as ExposureModelExperimentState,
};
use super::pressure_cpu_profiler::PressureCpuProfiler;
use super::experiments_diagnostic_telemetry::{
    ExperimentsDiagnostic, experiments_diagnostic_telemetry,
};
use super::lifecycle_telemetry::box_infrastructure_telemetry;
use super::structured_log_telemetry::{BOX_HELP_EVENT, box_help_telemetry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedHostTelemetryRecord {
    pub channel: String,
    pub event: String,
    pub payload: Value,
}

struct JsonlHostTelemetrySink {
    path: PathBuf,
    file: Mutex<File>,
}

impl JsonlHostTelemetrySink {
    fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self {
            path,
            file: Mutex::new(file),
        })
    }

    fn emit(&self, record: &PersistedHostTelemetryRecord) -> io::Result<()> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("Host telemetry sink mutex poisoned"))?;
        serde_json::to_writer(&mut *file, record).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.flush()
    }

    fn flush(&self) -> io::Result<()> {
        self.file
            .lock()
            .map_err(|_| io::Error::other("Host telemetry sink mutex poisoned"))?
            .flush()
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone)]
pub struct HostStructuredLogTelemetry {
    sink: Arc<JsonlHostTelemetrySink>,
    identity_tags: Arc<Mutex<BTreeMap<String, String>>>,
}

impl HostStructuredLogTelemetry {
    pub fn flush(&self) -> io::Result<()> {
        self.sink.flush()
    }

    pub fn report_projection(&self, projection: &HostTelemetryProjection) -> io::Result<()> {
        let mut metadata = self
            .identity_tags
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        metadata.extend(projection.metadata.clone());
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: projection.event.unwrap_or("sand.host.telemetry").to_string(),
            payload: json!({
                "level": projection.level.unwrap_or("info"),
                "metadata": metadata,
            }),
        })
    }

    pub fn report_box_log_record(&self, record: &BoxTelemetryRecord) -> io::Result<()> {
        match record {
            BoxTelemetryRecord::Log { source, line } => {
                let text = line.chars().take(2_048).collect::<String>();
                self.report_projection(&HostTelemetryProjection {
                    level: Some("info"),
                    event: Some("sand.box.log"),
                    metadata: BTreeMap::from([
                        ("source".into(), source.clone()),
                        ("text".into(), text),
                    ]),
                })
            }
            BoxTelemetryRecord::Infrastructure { event } => {
                self.report_projection(&box_infrastructure_telemetry(event))
            }
        }
    }

    pub fn report_box_log_ship(&self, report: &BoxLogShipReport) -> io::Result<()> {
        self.report_projection(&box_log_ship_telemetry(report))
    }

    pub fn report_box_help(&self, report: &Value) -> io::Result<()> {
        let projection = box_help_telemetry(report);
        debug_assert_eq!(projection.event, Some(BOX_HELP_EVENT));
        self.report_projection(&projection)
    }

    pub fn report_teach_recording_cap_stop_failed(&self, report: &Value) -> io::Result<()> {
        let mut metadata = BTreeMap::new();
        if let Some(error_class) = report.get("errorClass").and_then(Value::as_str) {
            metadata.insert("error_class".into(), error_class.to_string());
        }
        self.report_projection(&HostTelemetryProjection {
            level: Some("warn"),
            event: Some("sand.teach.cap_stop_failed"),
            metadata,
        })
    }

    pub fn report_teach_recording_start_failed(&self, report: &Value) -> io::Result<()> {
        let mut metadata = BTreeMap::new();
        if let Some(kind) = report.get("kind").and_then(Value::as_str) {
            metadata.insert("kind".into(), kind.to_string());
        }
        if let Some(error_class) = report.get("errorClass").and_then(Value::as_str) {
            metadata.insert("error_class".into(), error_class.to_string());
        }
        if let Some(window_index) = report.get("windowIndex").and_then(Value::as_u64) {
            metadata.insert("window_index".into(), window_index.to_string());
        }
        if let Some(entry_point) = report.get("entryPoint").and_then(Value::as_str) {
            metadata.insert("entry_point".into(), entry_point.to_string());
        }
        self.report_projection(&HostTelemetryProjection {
            level: Some("warn"),
            event: Some("sand.teach.recording_start_failed"),
            metadata,
        })
    }

    pub fn report_search_index_health(&self, report: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.search_index_health".into(),
            payload: report.clone(),
        })
    }

    pub fn report_host_extension_diagnostic(&self, diagnostic: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.host_extension_diagnostic".into(),
            payload: diagnostic.clone(),
        })
    }

    pub fn report_experiments_diagnostic(
        &self,
        diagnostic: &ExperimentsDiagnostic,
    ) -> io::Result<()> {
        self.report_projection(&experiments_diagnostic_telemetry(diagnostic))
    }

    pub fn report_automation_shadow_prune(&self, report: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.automation_shadow_prune".into(),
            payload: report.clone(),
        })
    }

    pub fn report_box_boot_stage_confirmed(&self, stage: &str, duration_ms: u64) -> bool {
        let mut metadata = BTreeMap::new();
        metadata.insert("stage".into(), stage.to_string());
        metadata.insert("duration_ms".into(), duration_ms.to_string());
        self.report_projection(&HostTelemetryProjection {
            level: Some("info"),
            event: Some("sand.box.boot_stage_confirmed"),
            metadata,
        })
        .is_ok()
    }
}

#[derive(Clone)]
pub struct HostProductAnalytics {
    sink: Arc<JsonlHostTelemetrySink>,
    runtime: Arc<Mutex<Option<Arc<ProductionAnalyticsRuntime>>>>,
}

impl HostProductAnalytics {
    pub fn attach_runtime(&self, runtime: Arc<ProductionAnalyticsRuntime>) {
        *self.runtime.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(runtime);
    }

    pub fn detach_runtime(&self) {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    pub fn mark_active(&self, reason: &str) {
        if let Some(runtime) = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            runtime.mark_active(reason);
        }
    }

    pub fn can_record_events(&self) -> bool {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .is_some_and(|runtime| runtime.can_record_events())
    }

    pub fn track_event(&self, name: &str, properties: &Value) -> io::Result<()> {
        let event = product_analytics_event(name, properties);
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "product_analytics".into(),
            event: event.name.clone(),
            payload: serde_json::to_value(&event.properties).map_err(io::Error::other)?,
        })?;
        if let Some(runtime) = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            runtime.track(event);
        }
        Ok(())
    }
}

impl AnalyticsClient for HostProductAnalytics {
    fn track_event(&self, name: &str, properties: &Value) {
        let _ = HostProductAnalytics::track_event(self, name, properties);
    }
}

impl TelemetryService for HostStructuredLogTelemetry {
    fn report(&self, method: &'static str, report: &Value) {
        let event = match method {
            "startTurn" => "sand.turn.start",
            "reportToolCallError" => "sand.tool_call.error",
            "reportToolCallStalled" => "sand.tool_call.stalled",
            "reportToolCallStarted" => "sand.tool_call.started",
            "reportAgentError" => "sand.agent.error",
            "reportBotBlock" => "sand.bot.block",
            "reportDaemonPing" => "sand.daemon.ping",
            "reportBoxBootStage" => "sand.box.boot_stage",
            "reportExecDaemonRestart" => "sand.exec_daemon.restart",
            "reportSupervisorRestart" => "sand.supervisor.restart",
            "reportTurnInterrupt" => "sand.turn.interrupt",
            "reportTurnAwait" => "sand.turn.await",
            "reportTurnRetry" => "sand.turn.retry",
            "reportUserMessageReceived" => "sand.user_message.received",
            "reportClosingSendNudge" => "sand.closing_send_nudge",
            "reportSubagentRevival" => "sand.subagent.revival",
            "reportShellRevival" => "sand.shell.revival",
            "reportComputerUseUsage" => "sand.computer_use.usage",
            "reportTtft" => "sand.turn.ttft",
            "reportSendDispatch" => "sand.send.dispatch",
            "reportQueueAccepted" => "sand.queue.accepted",
            "reportQueueDequeued" => "sand.queue.dequeued",
            "reportQueueWatchdog" => "sand.queue.watchdog",
            "reportAckObligation" => "sand.ack_obligation",
            "reportPendingWake" => "sand.pending_wake",
            "reportTurnUsage" => "sand.turn.usage",
            "reportTurnEmptyDelivery" => "sand.turn.empty_delivery",
            "reportJournalOutcome" => "sand.journal.outcome",
            "reportAutoReviewExpireSweepFailed" => "sand.auto_review.expire_sweep_failed",
            "reportAutomationLifecycle" => "sand.automation.lifecycle",
            "reportAutomationFireDropped" => "sand.automation.fire_dropped",
            "reportAutomationRun" => "sand.automation.run",
            _ => "sand.host.telemetry",
        };
        let _ = self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: event.into(),
            payload: report.clone(),
        });
    }
}


pub const TELEMETRY_FLUSH_TICK: Duration = Duration::from_secs(3);
pub const FATAL_TELEMETRY_FLUSH_TIMEOUT: Duration = Duration::from_millis(2_000);
pub const HOST_CRASH_MARKER_FORWARD_INTERVAL: Duration = Duration::from_secs(5 * 60);
pub const SAND_SUPERVISOR_DESKTOP_HEALTH_PATH: &str =
    "/tmp/sand-supervisor/desktop-health.json";
pub const DESKTOP_HEALTH_FORWARD_INTERVAL: Duration = Duration::from_secs(30);
pub const DESKTOP_HEALTH_HEARTBEAT_MS: u64 = 5 * 60 * 1_000;
pub const DESKTOP_HEALTH_EVENT: &str = "sand.box.desktop_health";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostBundleIdentity {
    pub host_bundle_version: Option<String>,
    pub box_store_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageSentReport {
    pub agent_id: String,
    pub prompt: Option<String>,
    pub attachment_paths: Vec<String>,
    pub rich_text: Option<String>,
    pub is_fork: bool,
    pub source: Option<String>,
    pub is_group_room: bool,
}

impl HostStructuredLogTelemetry {
    pub fn set_host_bundle_identity(&self, identity: &HostBundleIdentity) {
        let mut tags = self
            .identity_tags
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(version) = identity
            .host_bundle_version
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            tags.insert("host_bundle_version".into(), version.to_string());
        }
        if let Some(box_store_id) = identity
            .box_store_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            tags.insert("box_store_id".into(), box_store_id.to_string());
        }
    }

    pub fn report_inference_credential_renewal(
        &self,
        event: &HostAuthRenewalEvent,
    ) -> io::Result<()> {
        let mut metadata = BTreeMap::from([
            (
                "outcome".into(),
                match event.result.outcome {
                    RenewalOutcome::Renewed => "renewed",
                    RenewalOutcome::Failed => "failed",
                }
                .to_string(),
            ),
            (
                "consecutive_failures".into(),
                event.result.consecutive_failures.to_string(),
            ),
            ("duration_ms".into(), event.result.duration_ms.to_string()),
        ]);
        if let Some(error_summary) = event.result.error_summary.as_deref() {
            metadata.insert("error_summary".into(), error_summary.to_string());
        }
        self.report_projection(&HostTelemetryProjection {
            level: Some(if event.result.outcome == RenewalOutcome::Failed {
                "error"
            } else {
                "info"
            }),
            event: Some("sand.inference.credential_renewal"),
            metadata,
        })
    }

    pub fn report_host_log(&self, level: &str, message: &str) -> io::Result<()> {
        let level = match level {
            "error" => "error",
            "warn" => "warn",
            _ => "info",
        };
        self.report_projection(&HostTelemetryProjection {
            level: Some(level),
            event: Some("sand.host.log"),
            metadata: BTreeMap::from([("message".into(), message.to_string())]),
        })
    }

    pub fn report_host_lifecycle(&self, report: HostLifecycleReport) -> io::Result<()> {
        let mut metadata = BTreeMap::new();
        let (level, outcome, phase, duration_ms) = match report {
            HostLifecycleReport::Completed {
                phase,
                plugin_count,
                entry_count,
                duration_ms,
            } => {
                if let Some(value) = plugin_count {
                    metadata.insert("plugin_count".into(), value.to_string());
                }
                if let Some(value) = entry_count {
                    metadata.insert("entry_count".into(), value.to_string());
                }
                ("info", "completed", phase, duration_ms)
            }
            HostLifecycleReport::Failed {
                phase,
                duration_ms,
                ..
            } => ("error", "failed", phase, duration_ms),
            HostLifecycleReport::Stuck {
                phase,
                duration_ms,
                ..
            } => ("warn", "stuck", phase, duration_ms),
        };
        metadata.insert("outcome".into(), outcome.into());
        metadata.insert("phase".into(), phase);
        metadata.insert("duration_ms".into(), duration_ms.to_string());
        self.report_projection(&HostTelemetryProjection {
            level: Some(level),
            event: Some("sand.host.lifecycle"),
            metadata,
        })
    }
}

impl ModelExperimentExposureExperiments for HostExperimentsExtension {
    fn has_hydrated_statsig_user_id(&self) -> bool {
        HostExperimentsExtension::has_hydrated_statsig_user_id(self)
    }

    fn wait_for_hydrated_statsig_user_id(&self, timeout_ms: u64) -> bool {
        HostExperimentsExtension::wait_for_hydrated_statsig_user_id(
            self,
            Duration::from_millis(timeout_ms),
        )
    }

    fn get_sand_model_experiment_state(&self) -> Option<ExposureModelExperimentState> {
        HostExperimentsExtension::get_sand_model_experiment_state(self).map(|state| {
            ExposureModelExperimentState {
                active: state.active,
                arm: match state.arm {
                    SandModelExperimentArm::Control => "control",
                    SandModelExperimentArm::Treatment => "treatment",
                }
                .into(),
            }
        })
    }

    fn log_sand_model_experiment_exposure(&self) -> bool {
        HostExperimentsExtension::log_sand_model_experiment_exposure(self)
    }
}

impl ModelExperimentExposureAnalytics for HostProductAnalytics {
    fn can_record_events(&self) -> bool {
        HostProductAnalytics::can_record_events(self)
    }

    fn track_event(&self, name: &str, arm: &str) {
        let _ = HostProductAnalytics::track_event(self, name, &json!({ "arm": arm }));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopHealthForwardState {
    last_forwarded_revision: Option<u64>,
    last_forwarded_at_ms: Option<u64>,
}

pub fn forward_desktop_health_file_to_logs(
    path: &Path,
    logs: &HostStructuredLogTelemetry,
    state: &Mutex<DesktopHealthForwardState>,
    now_ms: u64,
) -> DesktopHealthForwardResult {
    let raw = fs::read_to_string(path).ok();
    let (last_revision, last_at_ms) = state
        .lock()
        .map(|state| {
            (
                state.last_forwarded_revision,
                state.last_forwarded_at_ms,
            )
        })
        .unwrap_or((None, None));
    let mut next_state = None;
    let result = forward_desktop_health_with(
        raw.as_deref(),
        last_revision,
        last_at_ms,
        now_ms,
        DESKTOP_HEALTH_HEARTBEAT_MS,
        |level, metadata| {
            let _ = logs.report_projection(&HostTelemetryProjection {
                level: Some(level),
                event: Some(DESKTOP_HEALTH_EVENT),
                metadata: metadata.clone(),
            });
        },
        |revision, at_ms| {
            next_state = Some((revision, at_ms));
        },
    );
    if let Some((revision, at_ms)) = next_state {
        if let Ok(mut state) = state.lock() {
            state.last_forwarded_revision = Some(revision);
            state.last_forwarded_at_ms = Some(at_ms);
        }
    }
    result
}

fn wall_clock_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn terminal_crash_marker_result(result: ForwardHostCrashMarkerResult) -> bool {
    matches!(
        result,
        ForwardHostCrashMarkerResult::Absent
            | ForwardHostCrashMarkerResult::Delivered
            | ForwardHostCrashMarkerResult::ParseError
    )
}

pub fn forward_host_crash_marker_to_logs(
    store: &impl HostCrashMarkerStore,
    logs: &HostStructuredLogTelemetry,
    last_handled: &Mutex<Option<String>>,
) -> ForwardHostCrashMarkerResult {
    forward_host_crash_marker_with(
        store,
        |raw| {
            last_handled
                .lock()
                .ok()
                .and_then(|value| value.clone())
                .as_deref()
                == Some(raw)
        },
        |raw| {
            if let Ok(mut value) = last_handled.lock() {
                *value = Some(raw.to_string());
            }
        },
        |marker| {
            let mut metadata = host_crash_marker_metadata(marker);
            metadata.insert("error_code".into(), "SAND-E0001".into());
            metadata.insert("error_domain".into(), "registry".into());
            metadata.insert("error_retryable".into(), "false".into());
            logs.report_projection(&HostTelemetryProjection {
                level: Some("error"),
                event: Some("sand.host.crash"),
                metadata,
            })
            .is_ok()
        },
    )
}

struct HostCrashMarkerForwarder {
    stop: mpsc::Sender<()>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl HostCrashMarkerForwarder {
    fn start(logs: HostStructuredLogTelemetry) -> Self {
        let store = FileHostCrashMarkerStore::new(get_host_crash_marker_path());
        let last_handled = Arc::new(Mutex::new(None));
        let initial = forward_host_crash_marker_to_logs(&store, &logs, &last_handled);
        let (stop_tx, stop_rx) = mpsc::channel();

        let worker = if terminal_crash_marker_result(initial) {
            None
        } else {
            Some(thread::spawn(move || {
                loop {
                    match stop_rx.recv_timeout(HOST_CRASH_MARKER_FORWARD_INTERVAL) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let result =
                                forward_host_crash_marker_to_logs(&store, &logs, &last_handled);
                            if terminal_crash_marker_result(result) {
                                break;
                            }
                        }
                    }
                }
            }))
        };

        Self {
            stop: stop_tx,
            worker: Mutex::new(worker),
        }
    }
}

impl Drop for HostCrashMarkerForwarder {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}

struct DesktopHealthForwarder {
    stop: mpsc::Sender<()>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl DesktopHealthForwarder {
    fn start(logs: HostStructuredLogTelemetry) -> Self {
        let path = PathBuf::from(SAND_SUPERVISOR_DESKTOP_HEALTH_PATH);
        let state = Arc::new(Mutex::new(DesktopHealthForwardState::default()));
        let _ = forward_desktop_health_file_to_logs(
            &path,
            &logs,
            &state,
            wall_clock_now_ms(),
        );

        let (stop_tx, stop_rx) = mpsc::channel();
        let worker_state = Arc::clone(&state);
        let worker = thread::spawn(move || {
            loop {
                match stop_rx.recv_timeout(DESKTOP_HEALTH_FORWARD_INTERVAL) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let _ = forward_desktop_health_file_to_logs(
                            &path,
                            &logs,
                            &worker_state,
                            wall_clock_now_ms(),
                        );
                    }
                }
            }
        });

        Self {
            stop: stop_tx,
            worker: Mutex::new(Some(worker)),
        }
    }
}

impl Drop for DesktopHealthForwarder {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}

struct StructuredLogFlushPolling {
    stop: mpsc::Sender<()>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl StructuredLogFlushPolling {
    fn start(
        logs: HostStructuredLogTelemetry,
        on_tick: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sand-host-structured-log-flush".into())
            .spawn(move || loop {
                match stop_rx.recv_timeout(TELEMETRY_FLUSH_TICK) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let _ = logs.flush();
                        if let Some(on_tick) = on_tick.as_ref() {
                            on_tick();
                        }
                    }
                }
            })
            .ok();
        Self {
            stop: stop_tx,
            worker: Mutex::new(worker),
        }
    }
}

impl Drop for StructuredLogFlushPolling {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}



#[derive(Clone, Default)]
pub struct HostTelemetryRuntimeHooks {
    pub tracing: Option<Arc<HostTracing>>,
    pub pressure_profiler: Option<Arc<Mutex<PressureCpuProfiler>>>,
}

#[derive(Clone)]
struct ProductionHostTelemetryDependencies {
    backend_url: String,
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    inference: Arc<ProductionInferenceExtension>,
}

#[derive(Default)]
struct HostTelemetryRuntimeState {
    auth_renewal_subscription: Option<u64>,
    inference_stop: Option<StopSubscription>,
    flush_polling: Option<StructuredLogFlushPolling>,
    crash_marker_forwarder: Option<HostCrashMarkerForwarder>,
    desktop_health_forwarder: Option<DesktopHealthForwarder>,
    event_loop_telemetry: Option<EventLoopTelemetryRuntime>,
    box_log_shipper: Option<Arc<BoxLogShipper>>,
    model_experiment_exposure: Option<Arc<ModelExperimentExposureLatch>>,
    host_bundle_identity_set: bool,
}

pub struct HostTelemetryService {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    pub brain: Arc<AutomationRunAnalyticsTelemetry>,
    sink: Arc<JsonlHostTelemetrySink>,
    started: AtomicBool,
    disposed: AtomicBool,
    production: Option<ProductionHostTelemetryDependencies>,
    hooks: HostTelemetryRuntimeHooks,
    runtime: Mutex<HostTelemetryRuntimeState>,
    clock_origin: Instant,
}

impl HostTelemetryService {
    pub fn open(records_path: impl Into<PathBuf>) -> io::Result<Self> {
        Self::open_with_identity_tags(records_path, resolve_sand_box_identity_tags())
    }

    pub fn open_with_identity_tags(
        records_path: impl Into<PathBuf>,
        mut identity_tags: BTreeMap<String, String>,
    ) -> io::Result<Self> {
        Self::open_internal(records_path, &mut identity_tags, None, HostTelemetryRuntimeHooks::default())
    }

    pub fn open_production(
        records_path: impl Into<PathBuf>,
        mut identity_tags: BTreeMap<String, String>,
        backend_url: String,
        auth: Arc<HostAuthExtension>,
        experiments: Arc<HostExperimentsExtension>,
        inference: Arc<ProductionInferenceExtension>,
        hooks: HostTelemetryRuntimeHooks,
    ) -> io::Result<Self> {
        Self::open_internal(
            records_path,
            &mut identity_tags,
            Some(ProductionHostTelemetryDependencies {
                backend_url,
                auth,
                experiments,
                inference,
            }),
            hooks,
        )
    }

    fn open_internal(
        records_path: impl Into<PathBuf>,
        identity_tags: &mut BTreeMap<String, String>,
        production: Option<ProductionHostTelemetryDependencies>,
        hooks: HostTelemetryRuntimeHooks,
    ) -> io::Result<Self> {
        identity_tags.retain(|_, value| !value.is_empty());
        let sink = Arc::new(JsonlHostTelemetrySink::open(records_path)?);
        let identity_tags = Arc::new(Mutex::new(identity_tags.clone()));
        let logs = HostStructuredLogTelemetry {
            sink: Arc::clone(&sink),
            identity_tags,
        };
        let analytics = HostProductAnalytics {
            sink: Arc::clone(&sink),
            runtime: Arc::new(Mutex::new(None)),
        };
        let brain = Arc::new(AutomationRunAnalyticsTelemetry::new(
            Arc::new(logs.clone()),
            Arc::new(analytics.clone()),
        ));
        Ok(Self {
            logs,
            analytics,
            brain,
            sink,
            started: AtomicBool::new(false),
            disposed: AtomicBool::new(false),
            production,
            hooks,
            runtime: Mutex::new(HostTelemetryRuntimeState::default()),
            clock_origin: Instant::now(),
        })
    }

    pub fn start(&self) -> io::Result<()> {
        if self.disposed.load(Ordering::Acquire) {
            return Err(io::Error::other("Host telemetry service is already disposed"));
        }
        if self.started.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        if let Err(error) = self.sink.flush() {
            self.started.store(false, Ordering::Release);
            return Err(error);
        }

        let Some(production) = self.production.as_ref().cloned() else {
            return Ok(());
        };
        let telemetry_enabled = std::env::var("SAND_DISABLE_TELEMETRY").as_deref() != Ok("1");
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| io::Error::other("Host telemetry runtime mutex poisoned"))?;

        let renewal_logs = self.logs.clone();
        let renewal_id = production.auth.service().subscribe_to_renewal(Arc::new(move |event| {
            let _ = renewal_logs.report_inference_credential_renewal(&event);
        }));
        if let Some(missed) = production.auth.service().get_last_renewal_event() {
            let _ = self.logs.report_inference_credential_renewal(&missed);
        }
        runtime.auth_renewal_subscription = Some(renewal_id);

        let analytics_runtime = ProductionAnalyticsRuntime::start(
            production.backend_url.clone(),
            Arc::clone(&production.auth),
            Arc::clone(&production.experiments),
        );
        self.analytics.attach_runtime(analytics_runtime);
        self.analytics.mark_active("host_startup");

        let exposure_experiments: Arc<dyn ModelExperimentExposureExperiments> =
            production.experiments.clone();
        let exposure_analytics: Arc<dyn ModelExperimentExposureAnalytics> =
            Arc::new(self.analytics.clone());
        let exposure = Arc::new(ModelExperimentExposureLatch::new(
            exposure_experiments,
            exposure_analytics,
            std::env::vars().collect(),
        ));
        let inference_exposure = Arc::clone(&exposure);
        runtime.inference_stop = Some(
            production
                .inference
                .on_model_experiment_applied(Arc::new(move || inference_exposure.note())),
        );
        runtime.model_experiment_exposure = Some(exposure);

        let diagnostic_logs = self.logs.clone();
        pin_experiments_diagnostics_reporter(Some(Arc::new(move |diagnostic| {
            let _ = diagnostic_logs.report_experiments_diagnostic(&diagnostic);
        })));

        if telemetry_enabled {
            let pressure_tick = self.hooks.pressure_profiler.as_ref().map(|profiler| {
                let profiler = Arc::clone(profiler);
                Arc::new(move || {
                    if let Ok(mut profiler) = profiler.lock() {
                        let _ = profiler.on_tick(wall_clock_now_ms().min(i64::MAX as u64) as i64);
                    }
                }) as Arc<dyn Fn() + Send + Sync>
            });
            runtime.flush_polling = Some(StructuredLogFlushPolling::start(
                self.logs.clone(),
                pressure_tick,
            ));
            runtime.desktop_health_forwarder =
                Some(DesktopHealthForwarder::start(self.logs.clone()));

            let event_logs = self.logs.clone();
            let pressure = self.hooks.pressure_profiler.clone();
            let experiments = Arc::clone(&production.experiments);
            runtime.event_loop_telemetry = Some(EventLoopTelemetryRuntime::start(Arc::new(
                move |report| {
                    let _ = event_logs.report_projection(&event_loop_window_telemetry(report));
                    if report.trigger == EventLoopTrigger::Pressure
                        && experiments.check_feature_gate("sand_enable_pressure_cpu_profiler")
                    {
                        if let Some(profiler) = pressure.as_ref() {
                            if let Ok(mut profiler) = profiler.lock() {
                                profiler.on_pressure(
                                    wall_clock_now_ms().min(i64::MAX as u64) as i64,
                                );
                            }
                        }
                    }
                },
            )));
        }

        if telemetry_enabled
            && is_box_log_shipping_enabled(
                std::env::var("SAND_HOST_IN_BOX").ok().as_deref(),
                std::env::var("SAND_BOX_LOG_SHIP_DISABLED").ok().as_deref(),
            )
        {
            let logs_for_batch = self.logs.clone();
            let logs_for_ship = self.logs.clone();
            let mut config = BoxLogShipperConfig::default();
            let host_log_file = std::env::var("SAND_HOST_LOG_FILE")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            config.skip_paths = vec![PathBuf::from(
                host_log_file.as_deref().unwrap_or(SUPERVISOR_LOG_PATH),
            )];
            runtime.box_log_shipper = Some(Arc::new(BoxLogShipper::new(
                config,
                Arc::new(move |records: &[BoxTelemetryRecord]| {
                    records
                        .iter()
                        .map(|record| {
                            if logs_for_batch.report_box_log_record(record).is_ok() {
                                DeliverySettlement::Delivered
                            } else {
                                DeliverySettlement::Dropped
                            }
                        })
                        .collect()
                }),
                Arc::new(move |report| {
                    if logs_for_ship.report_box_log_ship(report).is_ok() {
                        DeliverySettlement::Delivered
                    } else {
                        DeliverySettlement::Dropped
                    }
                }),
            )));
        }

        Ok(())
    }

    pub fn api(self: &Arc<Self>) -> HostTelemetryApi {
        HostTelemetryApi {
            service: Arc::clone(self),
        }
    }

    pub fn set_host_bundle_identity(&self, identity: HostBundleIdentity) -> io::Result<()> {
        if self.disposed.load(Ordering::Acquire) {
            return Err(io::Error::other("Host telemetry service is already disposed"));
        }
        self.logs.set_host_bundle_identity(&identity);
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| io::Error::other("Host telemetry runtime mutex poisoned"))?;
        if runtime.host_bundle_identity_set {
            return Ok(());
        }

        let started_crash_forwarder = runtime.crash_marker_forwarder.is_none();
        if started_crash_forwarder {
            runtime.crash_marker_forwarder =
                Some(HostCrashMarkerForwarder::start(self.logs.clone()));
        }
        if let Some(shipper) = runtime.box_log_shipper.as_ref() {
            if let Err(error) = shipper.start() {
                if started_crash_forwarder {
                    runtime.crash_marker_forwarder.take();
                }
                return Err(error);
            }
        }
        runtime.host_bundle_identity_set = true;
        Ok(())
    }

    pub fn report_message_sent(&self, report: MessageSentReport) {
        let text = report.prompt.as_deref().unwrap_or("").trim();
        let char_count = text.chars().count();
        self.analytics.mark_active("user_action");
        let _ = self.analytics.track_event(
            "sand.message.sent",
            &json!({
                "agent_id": report.agent_id,
                "char_count": char_count,
                "length_bucket": sand_message_length_bucket(char_count as f64).as_str(),
                "attachment_count": report.attachment_paths.len(),
                "has_rich_text": report.rich_text.as_deref().is_some_and(|value| !value.is_empty()),
                "is_fork": report.is_fork,
                "source": report.source.as_deref().unwrap_or("desktop"),
                "is_group_room": report.is_group_room,
            }),
        );
    }

    pub fn note_sand_model_experiment_active(&self) {
        let exposure = self
            .runtime
            .lock()
            .ok()
            .and_then(|runtime| runtime.model_experiment_exposure.clone());
        if let Some(exposure) = exposure {
            exposure.note();
        }
    }

    pub fn flush_tracing(&self) {
        if let Some(tracing) = self.hooks.tracing.as_ref() {
            tracing.flush();
        }
    }

    fn flush_for_fatal_exit_inner(&self) {
        if let Ok(runtime) = self.runtime.lock() {
            if let Some(shipper) = runtime.box_log_shipper.as_ref() {
                shipper.stop_polling();
                let _ = shipper.checkpoint_offsets();
            }
        }
        let _ = self.logs.flush();
        self.flush_tracing();
    }

    pub fn dispose(&self) -> io::Result<()> {
        if self.disposed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }

        pin_experiments_diagnostics_reporter(None);
        if let Ok(mut runtime) = self.runtime.lock() {
            if let Some(production) = self.production.as_ref() {
                if let Some(subscription) = runtime.auth_renewal_subscription.take() {
                    production.auth.service().unsubscribe_from_renewal(subscription);
                }
            }
            if let Some(stop) = runtime.inference_stop.take() {
                stop();
            }
            if let Some(shipper) = runtime.box_log_shipper.take() {
                shipper.stop_polling();
                let _ = shipper.checkpoint_offsets();
            }
            runtime.crash_marker_forwarder.take();
            runtime.desktop_health_forwarder.take();
            runtime.event_loop_telemetry.take();
            runtime.flush_polling.take();
            runtime.model_experiment_exposure.take();
            runtime.host_bundle_identity_set = false;
        }

        if let Some(profiler) = self.hooks.pressure_profiler.as_ref() {
            if let Ok(mut profiler) = profiler.lock() {
                profiler.dispose();
            }
        }
        self.analytics.detach_runtime();
        self.sink.flush()?;
        if let Some(tracing) = self.hooks.tracing.as_ref() {
            tracing.dispose();
        }
        Ok(())
    }

    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::Acquire)
    }

    pub fn records_path(&self) -> &Path {
        self.sink.path()
    }

    fn monotonic_now_ms(&self) -> f64 {
        self.clock_origin.elapsed().as_secs_f64() * 1_000.0
    }
}

impl Drop for HostTelemetryService {
    fn drop(&mut self) {
        let _ = self.dispose();
    }
}

#[derive(Clone)]
pub struct HostTelemetryApi {
    service: Arc<HostTelemetryService>,
}

impl HostTelemetryApi {
    pub fn service_dispose(&self) -> io::Result<()> {
        self.service.dispose()
    }

    pub fn set_host_bundle_identity(&self, identity: HostBundleIdentity) -> io::Result<()> {
        self.service.set_host_bundle_identity(identity)
    }

    pub fn flush_tracing(&self) {
        self.service.flush_tracing();
    }

    pub fn flush_for_fatal_exit(&self) {
        let service = Arc::clone(&self.service);
        let (done_tx, done_rx) = mpsc::channel();
        if thread::Builder::new()
            .name("sand-host-fatal-telemetry-flush".into())
            .spawn(move || {
                service.flush_for_fatal_exit_inner();
                let _ = done_tx.send(());
            })
            .is_ok()
        {
            let _ = done_rx.recv_timeout(FATAL_TELEMETRY_FLUSH_TIMEOUT);
        }
    }

    pub fn report_message_sent(&self, report: MessageSentReport) {
        self.service.report_message_sent(report);
    }

    pub fn note_sand_model_experiment_active(&self) {
        self.service.note_sand_model_experiment_active();
    }

    pub fn forward_console(&self, level: &str, message: &str) {
        let _ = self.service.logs.report_host_log(level, message);
    }

    pub fn monotonic_now_ms(&self) -> f64 {
        self.service.monotonic_now_ms()
    }

    pub fn create_host_lifecycle_progress(
        &self,
        started_at: f64,
        watchdog: WatchdogArm,
    ) -> HostLifecycleProgress {
        let service = Arc::clone(&self.service);
        let now = Arc::new(move || service.monotonic_now_ms());
        let logs = self.service.logs.clone();
        let report = Arc::new(move |report| {
            let _ = logs.report_host_lifecycle(report);
        });
        HostLifecycleProgress::new(started_at, now, report, watchdog)
    }
}
