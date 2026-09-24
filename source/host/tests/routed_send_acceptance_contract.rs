use std::sync::atomic::{AtomicUsize, Ordering};

use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use mahayana_host_runtime::runner::RecoveryUserMessage;

#[test]
fn routed_prompt_admission_owns_nonce_durability_and_recovery_identity() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-admission-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "hello",
        "clientNonce": "routed-nonce"
    });
    let persists = AtomicUsize::new(0);

    let persist = |_| {
        persists.fetch_add(1, Ordering::SeqCst);
        Ok::<_, ProductionSendError>(PersistedSendContext {
            echo_entry_id: Some("user-message:1".into()),
            user_message_id: Some("user-message:1".into()),
            recent_user_messages: vec![RecoveryUserMessage {
                id: "user-message:1".into(),
                text: "hello".into(),
                confirmed: None,
            }],
        })
    };

    let first = runtime
        .accept_routed_send(&args, persist)
        .expect("first routed admission");
    assert!(!first.duplicate);
    assert_eq!(first.context.user_message_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert!(runtime.is_turn_dispatch_idle("agent-a"));

    let duplicate = runtime
        .accept_routed_send(&args, persist)
        .expect("duplicate routed admission");
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.context.echo_entry_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(persists.load(Ordering::SeqCst), 2);

    let _ = std::fs::remove_dir_all(root);
}
