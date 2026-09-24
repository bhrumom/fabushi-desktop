use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::telemetry::extension::{
    TELEMETRY_EXTENSION_ID, start_host_telemetry_extension,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::PersistedHostTelemetryRecord;
use serde_json::json;

fn temp_root() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-host-telemetry-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn telemetry_extension_owns_box_help_structured_log_and_product_analytics_ingress() {
    assert_eq!(TELEMETRY_EXTENSION_ID, "telemetry");
    let root = temp_root();
    let extension = start_host_telemetry_extension(&root).expect("telemetry extension");

    extension
        .logs
        .report_box_help(&json!({
            "conversationId": "agent-a",
            "snapshotCaptured": true,
            "reason": "auth"
        }))
        .expect("box help log");
    extension
        .analytics
        .track_event(
            "sand.box_help",
            &json!({
                "agent_id": "agent-a",
                "snapshot_captured": true,
                "reason": "auth",
                "domain": "example.com",
                "ignored": null,
                "nested": {"not": "allowed"}
            }),
        )
        .expect("product analytics event");

    let text = fs::read_to_string(extension.records_path()).expect("telemetry jsonl");
    let records = text
        .lines()
        .map(|line| serde_json::from_str::<PersistedHostTelemetryRecord>(line).expect("record"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].channel, "structured_log");
    assert_eq!(records[0].event, "sand.box_help");
    assert_eq!(records[0].payload["level"], "info");
    assert_eq!(
        records[0].payload["metadata"]["conversation_id"],
        "agent-a"
    );
    assert_eq!(
        records[0].payload["metadata"]["snapshot_captured"],
        "true"
    );
    assert_eq!(records[0].payload["metadata"]["meta.reason"], "auth");

    assert_eq!(records[1].channel, "product_analytics");
    assert_eq!(records[1].event, "sand.box_help");
    assert_eq!(records[1].payload["agent_id"], "agent-a");
    assert_eq!(records[1].payload["snapshot_captured"], true);
    assert_eq!(records[1].payload["domain"], "example.com");
    assert!(records[1].payload.get("ignored").is_none());
    assert!(records[1].payload.get("nested").is_none());

    let _ = fs::remove_dir_all(root);
}
