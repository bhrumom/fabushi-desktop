use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use crate::host_paths::get_host_crash_marker_path;

use super::HostTelemetryProjection;
use super::host_crash_marker::{
    FileHostCrashMarkerStore, ForwardHostCrashMarkerResult, HostCrashMarkerStore,
    forward_host_crash_marker_with, host_crash_marker_metadata,
};
use super::host_telemetry_service::{
    HostProductAnalytics, HostStructuredLogTelemetry, HostTelemetryService,
};

pub const TELEMETRY_EXTENSION_ID: &str = "telemetry";
pub const HOST_CRASH_MARKER_FORWARD_INTERVAL: Duration = Duration::from_secs(5 * 60);

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

#[derive(Clone)]
pub struct HostTelemetryExtension {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    records_path: PathBuf,
    crash_marker_forwarder: Arc<HostCrashMarkerForwarder>,
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
    Ok(HostTelemetryExtension {
        logs: service.logs.clone(),
        analytics: service.analytics.clone(),
        records_path: service.records_path().to_path_buf(),
        crash_marker_forwarder,
    })
}
