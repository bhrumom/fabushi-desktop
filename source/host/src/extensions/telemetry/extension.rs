use std::io;
use std::path::{Path, PathBuf};

use super::host_telemetry_service::{
    HostProductAnalytics, HostStructuredLogTelemetry, HostTelemetryService,
};

pub const TELEMETRY_EXTENSION_ID: &str = "telemetry";

#[derive(Clone)]
pub struct HostTelemetryExtension {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    records_path: PathBuf,
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
    Ok(HostTelemetryExtension {
        logs: service.logs.clone(),
        analytics: service.analytics.clone(),
        records_path: service.records_path().to_path_buf(),
    })
}
