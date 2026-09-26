use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::host_paths::get_host_crash_marker_path;

use super::HostTelemetryProjection;
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

pub const TELEMETRY_EXTENSION_ID: &str = "telemetry";
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

#[derive(Clone)]
pub struct HostTelemetryExtension {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    records_path: PathBuf,
    crash_marker_forwarder: Arc<HostCrashMarkerForwarder>,
    _desktop_health_forwarder: Option<Arc<DesktopHealthForwarder>>,
}

impl HostTelemetryExtension {
    pub fn records_path(&self) -> &Path {
        &self.records_path
    }
}

pub fn start_host_telemetry_extension(
    app_data_dir: &Path,
) -> io::Result<HostTelemetryExtension> {
    let service = HostTelemetryService::open(
        app_data_dir.join("telemetry").join("host-events.jsonl"),
    )?;
    let crash_marker_forwarder =
        Arc::new(HostCrashMarkerForwarder::start(service.logs.clone()));
    let desktop_health_forwarder =
        (std::env::var("SAND_DISABLE_TELEMETRY").as_deref() != Ok("1"))
            .then(|| Arc::new(DesktopHealthForwarder::start(service.logs.clone())));
    Ok(HostTelemetryExtension {
        logs: service.logs.clone(),
        analytics: service.analytics.clone(),
        records_path: service.records_path().to_path_buf(),
        crash_marker_forwarder,
        _desktop_health_forwarder: desktop_health_forwarder,
    })
}
