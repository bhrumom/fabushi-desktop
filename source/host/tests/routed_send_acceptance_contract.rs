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
        "clientNonce": "routed-nonce",
        "streamId": "stream-a"
    });
    let persists = AtomicUsize::new(0);

    let first = runtime
        .accept_routed_send(&args, |_: &serde_json::Value| {
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
        })
        .expect("first routed admission");
    assert!(!first.duplicate);
    assert_eq!(first.context.user_message_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);
    assert!(!runtime.is_turn_dispatch_idle("agent-a"));
    assert!(runtime.has_routed_turn_lease("agent-a", "stream-a"));

    let duplicate = runtime
        .accept_routed_send(&args, |_: &serde_json::Value| {
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
        })
        .expect("duplicate routed admission");
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.context.echo_entry_id.as_deref(), Some("user-message:1"));
    assert_eq!(runtime.current_turn_epoch("agent-a"), 1);
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);
    assert_eq!(persists.load(Ordering::SeqCst), 2);

    runtime
        .require_routed_turn_lease("agent-a", "stream-a")
        .expect("provider start lease");
    runtime
        .settle_routed_turn("agent-a", "stream-a", 10_000)
        .expect("terminal settlement")
        .expect("lease settlement");
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert!(runtime.is_turn_dispatch_idle("agent-a"));
    assert!(!runtime.has_routed_turn_lease("agent-a", "stream-a"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn routed_prompt_queue_lease_blocks_the_next_turn_until_runner_terminal() {
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-admission-queue-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let first_args = serde_json::json!({
        "agentId": "agent-a",
        "prompt": "first",
        "clientNonce": "nonce-a",
        "streamId": "stream-a"
    });
    runtime
        .accept_routed_send(&first_args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:1".into()),
                user_message_id: Some("user-message:1".into()),
                recent_user_messages: vec![],
            })
        })
        .expect("first admission");

    let second_runtime = Arc::clone(&runtime);
    let (tx, rx) = mpsc::channel();
    let join = std::thread::spawn(move || {
        let args = serde_json::json!({
            "agentId": "agent-a",
            "prompt": "second",
            "clientNonce": "nonce-b",
            "streamId": "stream-b"
        });
        let result = second_runtime.accept_routed_send(&args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext {
                echo_entry_id: Some("user-message:2".into()),
                user_message_id: Some("user-message:2".into()),
                recent_user_messages: vec![],
            })
        });
        tx.send(result).expect("send second admission");
    });

    std::thread::sleep(Duration::from_millis(30));
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(runtime.in_flight_run_count("agent-a"), 2);
    assert_eq!(runtime.queued_turn_count("agent-a"), 1);

    runtime
        .settle_routed_turn("agent-a", "stream-a", 20_000)
        .expect("first terminal")
        .expect("first lease");
    let second = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("second admission released")
        .expect("second admission");
    assert!(!second.duplicate);
    assert!(runtime.has_routed_turn_lease("agent-a", "stream-b"));
    assert_eq!(runtime.in_flight_run_count("agent-a"), 1);

    runtime
        .settle_routed_turn("agent-a", "stream-b", 30_000)
        .expect("second terminal")
        .expect("second lease");
    assert_eq!(runtime.in_flight_run_count("agent-a"), 0);
    assert!(runtime.is_turn_dispatch_idle("agent-a"));
    join.join().expect("second admission thread");

    let _ = std::fs::remove_dir_all(root);
}


#[test]
fn group_member_routed_admission_projects_running_without_user_turn_badge() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-routed-group-member-{}",
        uuid::Uuid::new_v4()
    ));
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let args = serde_json::json!({
        "agentId": "member-a",
        "prompt": "hidden group prompt",
        "clientNonce": "group-member-nonce",
        "streamId": "group-member-stream",
        "requestSource": "group-member",
        "groupMemberTurn": true,
    });
    runtime
        .accept_routed_send(&args, |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        })
        .expect("group member admission");

    let mut roster = serde_json::json!([{"id":"member-a"}]);
    runtime.decorate_agent_summaries(&mut roster);
    assert_eq!(roster[0]["isRunning"], true);
    assert_eq!(roster[0]["isRunningTurn"], false);
    assert_eq!(runtime.active_turn_source("member-a").as_deref(), Some("group-member"));

    runtime
        .settle_routed_turn("member-a", "group-member-stream", 50_000)
        .expect("settlement")
        .expect("lease");
    assert_eq!(runtime.in_flight_run_count("member-a"), 0);
    let _ = std::fs::remove_dir_all(root);
}
