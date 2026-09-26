use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::telemetry::desktop_health_forwarder::DesktopHealthForwardResult;
use mahayana_host_runtime::extensions::telemetry::extension::{
    DESKTOP_HEALTH_EVENT, DESKTOP_HEALTH_HEARTBEAT_MS, DesktopHealthForwardState,
    TELEMETRY_EXTENSION_ID, forward_desktop_health_file_to_logs,
    start_host_telemetry_extension,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::{
    HostTelemetryService, PersistedHostTelemetryRecord,
};
use mahayana_host_runtime::extensions::telemetry::queue_telemetry_mappers::{
    QueueAcceptedReport, queue_accepted_telemetry,
};
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
    extension.logs.report_projection(&queue_accepted_telemetry(&QueueAcceptedReport {
        conversation_id: "agent-a".into(), lane: "user".into(), source: "turn".into(),
        position: 0, depth_user: 1, depth_agent: 0, depth_background: 0, has_active: false,
    })).expect("queue telemetry");

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
    assert_eq!(records.len(), 3);

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

    assert_eq!(records[1].channel, "structured_log");
    assert_eq!(records[1].event, "sand.queue.accepted");
    assert_eq!(records[1].payload["level"], "info");
    assert_eq!(records[1].payload["metadata"]["conversation_id"], "agent-a");
    assert_eq!(records[1].payload["metadata"]["lane"], "user");
    assert_eq!(records[2].channel, "product_analytics");
    assert_eq!(records[2].event, "sand.box_help");
    assert_eq!(records[2].payload["agent_id"], "agent-a");
    assert_eq!(records[2].payload["snapshot_captured"], true);
    assert_eq!(records[2].payload["domain"], "example.com");
    assert!(records[2].payload.get("ignored").is_none());
    assert!(records[2].payload.get("nested").is_none());

    let _ = fs::remove_dir_all(root);
}


#[test]
fn desktop_health_file_forwards_through_shipping_structured_log_with_frozen_heartbeat() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("telemetry root");
    let health_path = root.join("desktop-health.json");
    fs::write(
        &health_path,
        r#"{
          "updatedAtMs": 100,
          "revision": 7,
          "supervisionEnabled": true,
          "components": [
            {
              "name": "d1/xvfb",
              "up": false,
              "crashloop": false,
              "restartsInWindow": 2,
              "downReason": "oom"
            }
          ]
        }"#,
    )
    .expect("desktop health fixture");

    let service =
        HostTelemetryService::open(root.join("desktop-health-events.jsonl"))
            .expect("telemetry service");
    let state = Mutex::new(DesktopHealthForwardState::default());

    assert_eq!(
        forward_desktop_health_file_to_logs(
            &health_path,
            &service.logs,
            &state,
            1_000,
        ),
        DesktopHealthForwardResult::Emitted
    );
    assert_eq!(
        forward_desktop_health_file_to_logs(
            &health_path,
            &service.logs,
            &state,
            1_000 + DESKTOP_HEALTH_HEARTBEAT_MS - 1,
        ),
        DesktopHealthForwardResult::Skipped
    );
    assert_eq!(
        forward_desktop_health_file_to_logs(
            &health_path,
            &service.logs,
            &state,
            1_000 + DESKTOP_HEALTH_HEARTBEAT_MS,
        ),
        DesktopHealthForwardResult::Emitted
    );

    let text = fs::read_to_string(service.records_path()).expect("desktop health jsonl");
    let records = text
        .lines()
        .map(|line| {
            serde_json::from_str::<PersistedHostTelemetryRecord>(line)
                .expect("desktop health record")
        })
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    for record in &records {
        assert_eq!(record.channel, "structured_log");
        assert_eq!(record.event, DESKTOP_HEALTH_EVENT);
        assert_eq!(record.payload["level"], "warn");
        assert_eq!(record.payload["metadata"]["overall"], "degraded");
        assert_eq!(record.payload["metadata"]["down"], "1");
        assert_eq!(
            record.payload["metadata"]["down_reason"],
            "primary/xvfb=oom"
        );
    }

    let _ = fs::remove_dir_all(root);
}
