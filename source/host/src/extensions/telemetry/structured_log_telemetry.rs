use std::collections::BTreeMap;

use serde_json::Value;

use super::HostTelemetryProjection;

pub const BOX_HELP_EVENT: &str = "sand.box_help";

pub fn box_help_telemetry(report: &Value) -> HostTelemetryProjection {
    let conversation_id = report
        .get("conversationId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let snapshot_captured = report
        .get("snapshotCaptured")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut metadata = BTreeMap::from([
        ("conversation_id".to_string(), conversation_id),
        (
            "snapshot_captured".to_string(),
            snapshot_captured.to_string(),
        ),
    ]);
    if let Some(reason) = report.get("reason").and_then(Value::as_str) {
        metadata.insert("meta.reason".into(), reason.to_string());
    }

    HostTelemetryProjection {
        level: Some("info"),
        event: Some(BOX_HELP_EVENT),
        metadata,
    }
}
