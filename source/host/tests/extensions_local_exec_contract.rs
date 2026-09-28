use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::local_exec::{
    local_exec_error::SandLocalExecError,
    local_exec_failure_classifier::{
        classify_local_exec_failure, LocalExecFailureClass, LocalExecFailureClassification,
    },
};
use mahayana_host_runtime::extensions::local_exec::extension::start_local_exec_extension;
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
    let telemetry = HostTelemetryService::open_with_identity_tags(
        &path,
        Default::default(),
    )
    .expect("open telemetry");
    let extension = start_local_exec_extension(telemetry.logs.clone());

    assert!(!extension.check_live_computer_for_ask(Some("agent-local-1")));

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
