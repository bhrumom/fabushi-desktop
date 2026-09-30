use std::collections::BTreeMap;

use crate::gateway_server::GatewayCommandReport;

use super::HostTelemetryProjection;

pub const GATEWAY_COMMAND_ERROR_EVENT: &str = "sand.gateway_command_error";
pub const GATEWAY_COMMAND_TIMING_EVENT: &str = "sand.gateway_command_timing";

fn metadata(report: &GatewayCommandReport) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::from([
        ("method".into(), report.method.clone()),
        ("duration_ms".into(), report.duration_ms.to_string()),
        ("status".into(), report.status.to_string()),
    ]);
    if let Some(value) = report.request_id.as_deref() {
        metadata.insert("request_id".into(), value.into());
    }
    if let Some(value) = report.traceparent.as_deref() {
        metadata.insert("traceparent".into(), value.into());
    }
    if let Some(value) = report.error.as_deref() {
        metadata.insert("error".into(), value.into());
    }
    if let Some(value) = report.reason.as_deref() {
        metadata.insert("reason".into(), value.into());
    }
    if let Some(value) = report.error_class.as_deref() {
        metadata.insert("error_class".into(), value.into());
    }
    if let Some(value) = report.errno.as_deref() {
        metadata.insert("errno".into(), value.into());
    }
    metadata
}

pub fn gateway_command_timing_telemetry(report: &GatewayCommandReport) -> HostTelemetryProjection {
    HostTelemetryProjection {
        level: Some("info"),
        event: Some(GATEWAY_COMMAND_TIMING_EVENT),
        metadata: metadata(report),
    }
}

pub fn gateway_command_error_telemetry(report: &GatewayCommandReport) -> HostTelemetryProjection {
    HostTelemetryProjection {
        level: Some(if report.status >= 500 {
            "error"
        } else {
            "warn"
        }),
        event: Some(GATEWAY_COMMAND_ERROR_EVENT),
        metadata: metadata(report),
    }
}
