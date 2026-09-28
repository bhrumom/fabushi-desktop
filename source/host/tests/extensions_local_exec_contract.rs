use std::fs;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;

use mahayana_host_runtime::extensions::local_exec::extension::start_local_exec_extension;
use mahayana_host_runtime::extensions::local_exec::{
    local_exec_error::SandLocalExecError,
    local_exec_failure_classifier::{
        LocalExecFailureClass, LocalExecFailureClassification, classify_local_exec_failure,
    },
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;

#[test]
fn local_exec_ask_gate_reports_refusal_with_agent_identity() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-local-exec-telemetry-{}-{unique}.jsonl",
        std::process::id()
    ));
    let telemetry = HostTelemetryService::open_with_identity_tags(&path, Default::default())
        .expect("open telemetry");
    let before_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_millis();
    let extension = start_local_exec_extension(telemetry.logs.clone());

    assert!(!extension.check_live_computer_for_ask(Some("agent-local-1")));
    let after_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_millis();

    let raw = fs::read_to_string(&path).expect("read telemetry record");
    let record: serde_json::Value =
        serde_json::from_str(raw.lines().next().expect("telemetry line"))
            .expect("parse telemetry record");
    assert_eq!(record["event"], "sand.local_exec.refused");
    assert_eq!(record["payload"]["metadata"]["cause"], "no_providers");
    assert_eq!(record["payload"]["metadata"]["site"], "ask_gate");
    assert_eq!(
        record["payload"]["metadata"]["conversation_id"],
        "agent-local-1"
    );
    assert_eq!(record["payload"]["metadata"]["provider_count"], "0");
    assert_eq!(record["payload"]["metadata"]["live_provider_count"], "0");
    assert_eq!(record["payload"]["metadata"]["ever_registered"], "false");
    // Construction and refusal use a real clock; crossing a millisecond is valid.
    let empty_for_ms = record["payload"]["metadata"]["empty_for_ms"]
        .as_str()
        .expect("elapsed time string")
        .parse::<u64>()
        .expect("unsigned elapsed milliseconds");
    assert!(
        u128::from(empty_for_ms) <= after_ms.saturating_sub(before_ms),
        "refusal duration must fit within the production call interval"
    );
    let _ = fs::remove_file(path);
}

#[test]
fn local_exec_error_preserves_provider_message() {
    let error = SandLocalExecError::new("computer disconnected");
    assert_eq!(error.to_string(), "computer disconnected");
}

#[test]
fn failure_classifier_matches_grok_spawn_errno_contract() {
    assert_eq!(
        classify_local_exec_failure("Error: spawn rg ENOENT"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnEnoent,
            errno: Some("ENOENT".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("spawnSync tool EACCES"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnPermissions,
            errno: Some("EACCES".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("spawn helper ETIMEDOUT"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnOther,
            errno: Some("ETIMEDOUT".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("request failed EPERM"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::Other,
            errno: Some("EPERM".into()),
        }
    );
}

#[test]
fn local_exec_provider_lifecycle_is_persisted_through_host_telemetry() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-local-exec-provider-telemetry-{}-{unique}.jsonl",
        std::process::id()
    ));
    let telemetry = HostTelemetryService::open_with_identity_tags(&path, Default::default())
        .expect("open telemetry");
    let extension = start_local_exec_extension(telemetry.logs.clone());
    let (send, receive) = mpsc::channel();
    let registration = extension.register_provider(send);
    let welcome = receive
        .recv_timeout(Duration::from_millis(100))
        .expect("welcome");
    let provider_id = welcome["providerId"]
        .as_str()
        .expect("provider id")
        .to_string();
    extension.submit_responses(json!({
        "providerId": provider_id,
        "frames": [{
            "kind": "hello",
            "localRoot": "/Users/test",
            "terminalsFolder": "terminals",
            "computerId": "mac-telemetry",
            "supervised": true,
            "variant": "sand"
        }]
    }));
    drop(registration);

    let records = fs::read_to_string(&path)
        .expect("read telemetry")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("telemetry record"))
        .filter(|record| record["event"] == "sand.local_exec.provider")
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["payload"]["metadata"]["phase"], "registered");
    assert_eq!(records[1]["payload"]["metadata"]["phase"], "hello");
    assert_eq!(
        records[1]["payload"]["metadata"]["computer_id_present"],
        "true"
    );
    assert_eq!(records[1]["payload"]["metadata"]["supervised"], "true");
    assert_eq!(records[1]["payload"]["metadata"]["variant"], "sand");
    assert_eq!(records[2]["payload"]["metadata"]["phase"], "detached");
    assert_eq!(records[2]["payload"]["metadata"]["had_hello"], "true");
    assert_eq!(records[2]["payload"]["metadata"]["emptied"], "true");
    let _ = fs::remove_file(path);
}
