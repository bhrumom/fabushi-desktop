use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::transcript::ack_obligations::{
    ACK_REDRIVE_IDLE_DELAY_MS, MAX_ACK_REDRIVES, AckObligations, build_ack_redrive_prompt,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-ack-manager-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn ack_tokens_are_agent_scoped_and_visible_send_fulfills_coalesced_obligation() {
    let root = temp_root("tokens");
    let ack = AckObligations::new(&root);
    let first = ack
        .record_send_and_mint_token("agent-a", 10.0)
        .expect("first reservation");
    let second = ack
        .record_send_and_mint_token("agent-a", 20.0)
        .expect("second reservation");
    assert!(first.created);
    assert!(!second.created);
    assert_eq!(second.obligation.coalesced_count, 2.0);

    assert!(!ack
        .fulfill_ack_obligation("agent-b", &first.ack_token)
        .expect("wrong agent"));
    assert!(ack.store().get("agent-a").is_some());

    assert!(ack
        .fulfill_ack_obligation("agent-a", &first.ack_token)
        .expect("fulfill"));
    assert!(ack.store().get("agent-a").is_none());
    assert!(ack.retire_ack_run_token("agent-a", Some(&first.ack_token)));
    assert!(ack.retire_ack_run_token("agent-a", Some(&second.ack_token)));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_enqueue_rollback_restores_previous_obligation_only_if_reservation_is_current() {
    let root = temp_root("rollback");
    let ack = AckObligations::new(&root);
    let first = ack
        .record_send_and_mint_token("agent-a", 10.0)
        .expect("first reservation");
    let second = ack
        .record_send_and_mint_token("agent-a", 20.0)
        .expect("second reservation");

    assert!(ack
        .rollback_ack_reservation("agent-a", &second.ack_token)
        .expect("rollback second"));
    let restored = ack.store().get("agent-a").expect("restored first");
    assert_eq!(restored.last_send_at_ms, 10.0);
    assert_eq!(restored.coalesced_count, 1.0);

    assert!(ack
        .fulfill_ack_obligation("agent-a", &first.ack_token)
        .expect("fulfill first"));
    assert!(ack.store().get("agent-a").is_none());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn ack_redrive_constants_and_prompt_keep_the_frozen_recovery_contract() {
    assert_eq!(MAX_ACK_REDRIVES, 3);
    assert_eq!(ACK_REDRIVE_IDLE_DELAY_MS, 5_000);
    let prompt = build_ack_redrive_prompt();
    assert!(prompt.contains("SendMessage"));
    assert!(prompt.contains("NEVER guess"));
    assert!(prompt.contains("ask them to resend"));
}
