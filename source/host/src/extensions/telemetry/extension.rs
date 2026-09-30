use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::browser_ua::extension::StopSubscription;
use crate::extensions::experiments::{
    HostExperimentsExtension, pin_experiments_diagnostics_reporter,
};
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::inference::production::ProductionInferenceExtension;
use crate::host_paths::get_host_crash_marker_path;

use super::HostTelemetryProjection;
use super::analytics_service::{
    AutomationRunAnalyticsTelemetry, ProductionAnalyticsRuntime,
};
use super::box_log_shipper::{
    BoxLogShipper, BoxLogShipperConfig, BoxTelemetryRecord, DeliverySettlement,
    SUPERVISOR_LOG_PATH, is_box_log_shipping_enabled,
};
use super::desktop_health_forwarder::{
    DesktopHealthForwardResult, forward_desktop_health_with,
};
use super::host_crash_marker::{
    FileHostCrashMarkerStore, ForwardHostCrashMarkerResult, HostCrashMarkerStore,
    forward_host_crash_marker_with, host_crash_marker_metadata,
};
use super::host_telemetry_service::{
    HostProductAnalytics, HostStructuredLogTelemetry, HostTelemetryService,
};
use super::event_loop_telemetry::{
    EventLoopTelemetryRuntime, event_loop_window_telemetry,
};

pub const TELEMETRY_EXTENSION_ID: &str = "telemetry";
pub const TELEMETRY_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Experiments,
    HostExtensionId::Inference,
];
pub const TELEMETRY_FLUSH_TICK: Duration = Duration::from_secs(3);
pub const FATAL_TELEMETRY_FLUSH_TIMEOUT: Duration = Duration::from_millis(2_000);
pub const HOST_CRASH_MARKER_FORWARD_INTERVAL: Duration = Duration::from_secs(5 * 60);
pub const SAND_SUPERVISOR_DESKTOP_HEALTH_PATH: &str =
    "/tmp/sand-supervisor/desktop-health.json";
pub const DESKTOP_HEALTH_FORWARD_INTERVAL: Duration = Duration::from_secs(30);
pub const DESKTOP_HEALTH_HEARTBEAT_MS: u64 = 5 * 60 * 1_000;
pub const DESKTOP_HEALTH_EVENT: &str = "sand.box.desktop_health";

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
    fn start(logs: HostStructuredLogTelemetry) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sand-host-structured-log-flush".into())
            .spawn(move || loop {
                match stop_rx.recv_timeout(TELEMETRY_FLUSH_TICK) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let _ = logs.flush();
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

struct HostTelemetryLifecycle {
    service: Arc<HostTelemetryService>,
    inference_stop: Mutex<Option<StopSubscription>>,
    flush_polling: Mutex<Option<StructuredLogFlushPolling>>,
    crash_marker_forwarder: Mutex<Option<HostCrashMarkerForwarder>>,
    desktop_health_forwarder: Mutex<Option<DesktopHealthForwarder>>,
    event_loop_telemetry: Mutex<Option<EventLoopTelemetryRuntime>>,
    box_log_shipper: Mutex<Option<Arc<BoxLogShipper>>>,
    disposed: AtomicBool,
}

impl HostTelemetryLifecycle {
    fn dispose(&self) {
        if self.disposed.swap(true, Ordering::AcqRel) {
            return;
        }
        pin_experiments_diagnostics_reporter(None);
        if let Ok(mut stop) = self.inference_stop.lock() {
            if let Some(stop) = stop.take() {
                stop();
            }
        }
        if let Ok(mut shipper) = self.box_log_shipper.lock() {
            if let Some(shipper) = shipper.take() {
                shipper.stop_polling();
                let _ = shipper.checkpoint_offsets();
            }
        }
        self.crash_marker_forwarder.lock().ok().and_then(|mut value| value.take());
        self.desktop_health_forwarder.lock().ok().and_then(|mut value| value.take());
        self.event_loop_telemetry.lock().ok().and_then(|mut value| value.take());
        self.flush_polling.lock().ok().and_then(|mut value| value.take());
        let _ = self.service.dispose();
    }
}

impl Drop for HostTelemetryLifecycle {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[derive(Clone)]
pub struct HostTelemetryExtension {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    pub brain: Arc<AutomationRunAnalyticsTelemetry>,
    records_path: PathBuf,
    lifecycle: Arc<HostTelemetryLifecycle>,
}

impl HostTelemetryExtension {
    pub fn records_path(&self) -> &Path {
        &self.records_path
    }

    pub fn dispose(&self) {
        self.lifecycle.dispose();
    }

    pub fn flush_for_fatal_exit(&self) {
        let logs = self.logs.clone();
        let (done_tx, done_rx) = mpsc::channel();
        if thread::Builder::new()
            .name("sand-host-fatal-telemetry-flush".into())
            .spawn(move || {
                let _ = logs.flush();
                let _ = done_tx.send(());
            })
            .is_ok()
        {
            let _ = done_rx.recv_timeout(FATAL_TELEMETRY_FLUSH_TIMEOUT);
        }
    }
}

pub fn start_host_telemetry_extension(
    app_data_dir: &Path,
    backend_url: String,
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    inference: Arc<ProductionInferenceExtension>,
) -> io::Result<HostTelemetryExtension> {
    let service = Arc::new(HostTelemetryService::open(
        app_data_dir.join("telemetry").join("host-events.jsonl"),
    )?);
    service.start()?;

    let telemetry_enabled =
        std::env::var("SAND_DISABLE_TELEMETRY").as_deref() != Ok("1");
    let flush_polling = telemetry_enabled
        .then(|| StructuredLogFlushPolling::start(service.logs.clone()));
    let crash_marker_forwarder =
        Some(HostCrashMarkerForwarder::start(service.logs.clone()));
    let desktop_health_forwarder = telemetry_enabled
        .then(|| DesktopHealthForwarder::start(service.logs.clone()));
    let event_loop_telemetry = telemetry_enabled.then(|| {
        let logs = service.logs.clone();
        EventLoopTelemetryRuntime::start(Arc::new(move |report| {
            let _ = logs.report_projection(&event_loop_window_telemetry(report));
        }))
    });
    let box_log_shipper = if telemetry_enabled
        && is_box_log_shipping_enabled(
            std::env::var("SAND_HOST_IN_BOX").ok().as_deref(),
            std::env::var("SAND_BOX_LOG_SHIP_DISABLED").ok().as_deref(),
        )
    {
        let logs_for_batch = service.logs.clone();
        let logs_for_ship = service.logs.clone();
        let mut config = BoxLogShipperConfig::default();
        let host_log_file = std::env::var("SAND_HOST_LOG_FILE")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        config.skip_paths = vec![PathBuf::from(
            host_log_file.as_deref().unwrap_or(SUPERVISOR_LOG_PATH),
        )];
        let shipper = Arc::new(BoxLogShipper::new(
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
        ));
        if let Err(error) = shipper.start() {
            let _ = service.dispose();
            return Err(error);
        }
        Some(shipper)
    } else {
        None
    };

    let analytics_runtime = ProductionAnalyticsRuntime::start(
        backend_url,
        Arc::clone(&auth),
        Arc::clone(&experiments),
    );
    service.analytics.attach_runtime(analytics_runtime);
    service.analytics.mark_active("host_startup");

    let model_exposure_analytics = service.analytics.clone();
    let inference_stop = inference.on_model_experiment_applied(Arc::new(move || {
        model_exposure_analytics.mark_active("model_experiment_applied");
    }));

    let diagnostic_logs = service.logs.clone();
    pin_experiments_diagnostics_reporter(Some(Arc::new(move |diagnostic| {
        let _ = diagnostic_logs.report_experiments_diagnostic(&diagnostic);
    })));

    let brain = Arc::new(AutomationRunAnalyticsTelemetry::new(
        Arc::new(service.logs.clone()),
        Arc::new(service.analytics.clone()),
    ));
    let lifecycle = Arc::new(HostTelemetryLifecycle {
        service: Arc::clone(&service),
        inference_stop: Mutex::new(Some(inference_stop)),
        flush_polling: Mutex::new(flush_polling),
        crash_marker_forwarder: Mutex::new(crash_marker_forwarder),
        desktop_health_forwarder: Mutex::new(desktop_health_forwarder),
        event_loop_telemetry: Mutex::new(event_loop_telemetry),
        box_log_shipper: Mutex::new(box_log_shipper),
        disposed: AtomicBool::new(false),
    });
    Ok(HostTelemetryExtension {
        logs: service.logs.clone(),
        analytics: service.analytics.clone(),
        brain,
        records_path: service.records_path().to_path_buf(),
        lifecycle,
    })
}
