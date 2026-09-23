use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-production-transcript-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn production_send_runtime_persists_acceptance_and_replays_nonce_without_redispatch() {
    let root = temp_root("accepted");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let dispatches = AtomicUsize::new(0);
    let persisted = AtomicUsize::new(0);
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "hello",
        "clientNonce": "nonce-a"
    });

    let first = runtime
        .execute_send(
            &args,
            || {
                dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({
                    "accepted": true,
                    "operationId": "op-a"
                }))
            },
            |_| {
                persisted.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect("first send");
    assert_eq!(first["operationId"], "op-a");
    assert_eq!(dispatches.load(Ordering::SeqCst), 1);
    assert_eq!(persisted.load(Ordering::SeqCst), 1);
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);

    let replay = runtime
        .execute_send(
            &args,
            || {
                dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({"accepted": true, "operationId": "wrong"}))
            },
            |_| Ok(()),
        )
        .expect("replayed send");
    assert_eq!(replay["operationId"], "op-a");
    assert_eq!(dispatches.load(Ordering::SeqCst), 1);

    let status = runtime
        .prompt_acceptance_status(&serde_json::json!({
            "accountSlot": "host",
            "clientNonce": "nonce-a"
        }))
        .expect("acceptance status");
    assert_eq!(status["outcome"], "found");
    assert_eq!(status["record"]["status"], "accepted");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_send_runtime_clears_failed_unaccepted_nonce_for_retry() {
    let root = temp_root("retry");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "retry",
        "clientNonce": "nonce-retry"
    });

    let first = runtime.execute_send(
        &args,
        || Err(ProductionSendError::Internal("transport failed".into())),
        |_| Ok(()),
    );
    assert!(first.is_err());

    let second = runtime
        .execute_send(
            &args,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-retry"})),
            |_| Ok(()),
        )
        .expect("retry dispatch");
    assert_eq!(second["operationId"], "op-retry");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_send_runtime_rejects_nonce_digest_reuse() {
    let root = temp_root("digest");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let first = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "one",
        "clientNonce": "nonce-shared"
    });
    runtime
        .execute_send(
            &first,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-one"})),
            |_| Ok(()),
        )
        .expect("first");

    let mismatch = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "two",
        "clientNonce": "nonce-shared"
    });
    assert!(matches!(
        runtime.execute_send(
            &mismatch,
            || Ok(serde_json::json!({"accepted": true, "operationId": "op-two"})),
            |_| Ok(()),
        ),
        Err(ProductionSendError::Conflict(_))
    ));
    let _ = fs::remove_dir_all(root);
}
