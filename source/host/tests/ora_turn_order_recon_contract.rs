//! Executable oracles for the canonical Conversation turn, transcript ordering and
//! transcript reconciliation contracts.
//!
//! Oracle IDs: `ORA-TURN-001`, `ORA-ORDER-001`, `ORA-RECON-001`.
//!
//! Normative definitions:
//! - `projects/fabushi-communication-platform/quality/oracles/conversation-turn-oracle.md`
//! - `projects/fabushi-communication-platform/quality/oracles/transcript-ordering-oracle.md`
//! - `projects/fabushi-communication-platform/quality/oracles/reconciliation-oracle.md`
//!
//! Every case carries machine-readable `// oracle:` and `// regression:` annotations.
//! `scripts/validate-quality-oracles.mjs` fails closed when a normative invariant has
//! neither an executable case nor an explicit `pending` disposition in
//! `projects/fabushi-communication-platform/quality/oracles/oracle-execution-status.json`.
//!
//! These cases exercise the existing canonical owners only:
//! - `extensions::transcript::run_scheduler`
//! - `extensions::transcript::run_lifecycle`
//! - `extensions::transcript::send_pipeline`
//! - `extensions::transcript::prompt_acceptance_ledger`
//! - `extensions::transcript::transcript_entry_ids`
//!
//! No parallel turn/transcript owner is introduced. Lower-layer coverage here is not a
//! substitute for the packaged temporal acceptance required by AC-43/AC-44.

use std::fs;

use mahayana_host_runtime::extensions::transcript::prompt_acceptance_ledger::{
    AcceptanceLookup, AcceptanceStatus, PromptAcceptanceError, PromptAcceptanceLedger,
    SendAdmission, SendInput, send_input_digest,
};
use mahayana_host_runtime::extensions::transcript::run_lifecycle::RunLifecycleState;
use mahayana_host_runtime::extensions::transcript::run_scheduler::{
    ActivePhase, QueuedRun, RunLane, RunScheduler, RunSettlement, WatchdogStage,
};
use mahayana_host_runtime::extensions::transcript::send_pipeline::{
    SendBegin, SendEchoIdentity, SendPipelineState,
};
use mahayana_host_runtime::extensions::transcript::transcript_entry_ids::{
    TranscriptEntryIdKind, next_entry_id,
};
use serde_json::{Value, json};
use uuid::Uuid;

fn temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("fabushi-ora-{name}-{}", Uuid::new_v4()))
}

fn send_input(prompt: &str) -> SendInput {
    SendInput {
        agent_id: Some("agent-1".into()),
        prompt: prompt.into(),
        rich_text: None,
        reply_to_id: None,
        is_fork: false,
        attachment_paths: Vec::new(),
        attachment_names: Vec::new(),
    }
}

fn queued(id: &str, lane: RunLane, at: u64) -> QueuedRun {
    QueuedRun {
        task_id: id.into(),
        lane,
        source: "oracle".into(),
        enqueued_at_ms: at,
        accepted_at_ms: Some(at.saturating_sub(2)),
        ack_token: Some(format!("ack-{id}")),
    }
}

#[test]
// oracle: TURN-I01
// regression: REG-TURN-FINAL-001
fn ora_turn_settles_exactly_one_terminal_outcome_per_generation() {
    let mut scheduler = RunScheduler::new(100, 20);
    scheduler
        .enqueue("agent", queued("turn-1", RunLane::User, 0))
        .unwrap();
    let active = scheduler.start_next("agent", 10).expect("active turn");
    assert_eq!(active.task_id, "turn-1");

    let first_terminal = matches!(
        scheduler.settle("agent", active.generation, 20),
        RunSettlement::ActiveSettled { .. }
    );
    assert!(
        first_terminal,
        "the addressed turn must produce exactly one terminal outcome"
    );
    assert!(scheduler.active("agent").is_none());

    let replayed_terminal = matches!(
        scheduler.settle("agent", active.generation, 21),
        RunSettlement::ActiveSettled { .. }
    );
    assert!(
        !replayed_terminal,
        "replaying the same settlement must not produce a second terminal outcome"
    );
}

#[test]
// oracle: TURN-I02
// regression: REG-TURN-INTERMEDIATE-001
fn ora_turn_nested_segment_cannot_become_the_user_terminal() {
    let mut lifecycle = RunLifecycleState::default();
    lifecycle.begin_session_run("agent", 100, false);
    lifecycle.begin_session_run("agent", 105, true);
    lifecycle.set_activity("agent", Some("thinking"));
    assert_eq!(lifecycle.in_flight_count("agent"), 2);

    assert!(
        lifecycle.end_session_run("agent", 130).is_none(),
        "an inner intermediate segment must not settle the user-visible turn"
    );
    assert_eq!(lifecycle.in_flight_count("agent"), 1);

    let terminal = lifecycle
        .end_session_run("agent", 160)
        .expect("outer terminal");
    assert_eq!(terminal.agent_id, "agent");
    assert_eq!(terminal.turn_worthy_begin_count, 1);
    assert!(
        lifecycle.end_session_run("agent", 170).is_none(),
        "a settled turn must not emit a second terminal"
    );
}

#[test]
// oracle: TURN-I04
fn ora_turn_completed_entry_cannot_return_to_running() {
    let mut lifecycle = RunLifecycleState::default();
    lifecycle.begin_session_run("agent", 10, false);
    lifecycle.set_composing("agent", true);
    lifecycle.set_retrying("agent", true);
    lifecycle.set_activity("agent", Some("thinking"));
    assert!(lifecycle.end_session_run("agent", 40).is_some());

    assert_eq!(lifecycle.in_flight_count("agent"), 0);
    assert!(!lifecycle.is_composing("agent"));
    assert!(!lifecycle.is_retrying("agent"));
    assert_eq!(lifecycle.activity("agent"), None);
    assert!(lifecycle.running_agent_ids().is_empty());
    assert_eq!(lifecycle.active_run_session(), None);
    assert!(lifecycle.end_session_run("agent", 50).is_none());
}

#[test]
// oracle: TURN-I05
// regression: REG-CROSS-CONVERSATION-001
fn ora_turn_addressed_owner_is_not_moved_by_visible_selection() {
    let mut lifecycle = RunLifecycleState::default();
    lifecycle.begin_session_run("agent-a", 10, false);
    lifecycle.begin_session_run("agent-b", 20, false);
    assert_eq!(lifecycle.in_flight_count("agent-a"), 1);
    assert_eq!(lifecycle.in_flight_count("agent-b"), 1);

    // Visible selection moved to agent-b. Settling agent-a must stay on agent-a.
    let settled = lifecycle
        .end_session_run("agent-a", 30)
        .expect("agent-a terminal");
    assert_eq!(settled.agent_id, "agent-a");
    assert_eq!(lifecycle.in_flight_count("agent-b"), 1);
    assert_eq!(lifecycle.running_agent_ids().len(), 1);
}

#[test]
// oracle: TURN-I07
fn ora_turn_cancelled_run_emits_no_later_terminal() {
    let mut scheduler = RunScheduler::new(100, 20);
    scheduler
        .enqueue("agent", queued("cancelled", RunLane::User, 0))
        .unwrap();
    let cancelled = scheduler.start_next("agent", 10).expect("active turn");

    let tripped = scheduler.watchdog_tick("agent", 120).expect("watchdog trip");
    assert_eq!(tripped.stage, WatchdogStage::Trip);
    assert_eq!(
        scheduler.active("agent").unwrap().phase,
        ActivePhase::Interrupted
    );

    let escaped = scheduler.watchdog_tick("agent", 140).expect("watchdog escape");
    assert_eq!(escaped.stage, WatchdogStage::Escape);
    assert_eq!(escaped.ack_token.as_deref(), Some("ack-cancelled"));
    assert!(scheduler.active("agent").is_none());

    scheduler
        .enqueue("agent", queued("successor", RunLane::User, 141))
        .unwrap();
    let successor = scheduler.start_next("agent", 141).expect("successor turn");
    assert_eq!(successor.task_id, "successor");

    assert!(
        matches!(
            scheduler.settle("agent", cancelled.generation, 150),
            RunSettlement::ZombieSettled { .. }
        ),
        "a cancelled turn must not emit a terminal outcome after cancellation"
    );
    assert_eq!(
        scheduler.active("agent").unwrap().generation,
        successor.generation
    );
    assert!(matches!(
        scheduler.settle("agent", successor.generation, 160),
        RunSettlement::ActiveSettled { task_id, .. } if task_id == "successor"
    ));
}

#[test]
// oracle: ORD-I01
// oracle: ORD-I02
// regression: REG-TRANSCRIPT-ORDER-001
fn ora_order_entry_ids_never_collide_with_committed_ids() {
    let mut entries: Vec<Value> = vec![
        json!({"id": "t1u", "kind": "message", "role": "user"}),
        json!({"id": "t0a1", "kind": "message", "role": "assistant"}),
    ];

    let next = next_entry_id(&entries, TranscriptEntryIdKind::AssistantMessage);
    assert_eq!(next, "t0a2", "a replayed append must not reuse a committed id");
    entries.push(json!({"id": next, "kind": "message", "role": "assistant"}));

    let replayed = next_entry_id(&entries, TranscriptEntryIdKind::AssistantMessage);
    assert_eq!(replayed, "t0a3");
    assert_ne!(replayed, next);

    let user = next_entry_id(&entries, TranscriptEntryIdKind::UserMessage);
    assert_eq!(user, "t2u");

    let ids = entries
        .iter()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let unique = ids.iter().copied().collect::<std::collections::HashSet<_>>();
    assert_eq!(
        ids.len(),
        unique.len(),
        "committed entry ids must stay unique across appends"
    );
}

#[test]
// oracle: ORD-I03
// oracle: REC-I04
// regression: REG-TRANSCRIPT-DUP-001
fn ora_order_optimistic_entry_merges_with_authoritative_echo() {
    let root = temp_dir("order-merge");
    let mut ledger = PromptAcceptanceLedger::new(Some(&root));
    let mut pipeline = SendPipelineState::default();
    let input = send_input("hello");

    assert!(matches!(
        pipeline
            .begin_send(&mut ledger, &input, Some("nonce-order"))
            .unwrap(),
        SendBegin::Dispatch { .. }
    ));
    pipeline
        .record_pending_acceptance(
            &mut ledger,
            "nonce-order",
            SendEchoIdentity {
                agent_id: "agent-1".into(),
                echo_entry_id: Some("t1u".into()),
            },
        )
        .unwrap();
    pipeline.mark_send_accepted(&mut ledger, Some("nonce-order"));
    pipeline.finish_send(&mut ledger, Some("nonce-order"), false);

    match ledger.lookup("host", "nonce-order") {
        AcceptanceLookup::Found(record) => {
            assert_eq!(record.status, AcceptanceStatus::Accepted);
            assert_eq!(record.echo_entry_id.as_deref(), Some("t1u"));
        }
        other => panic!("expected one accepted record, got {other:?}"),
    }
    assert_eq!(
        ledger.record_count(),
        1,
        "optimistic plus authoritative must not duplicate the accepted entry"
    );

    assert!(matches!(
        pipeline
            .begin_send(&mut ledger, &input, Some("nonce-order"))
            .unwrap(),
        SendBegin::DuplicateNoop { .. }
    ));
    assert_eq!(ledger.record_count(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
// oracle: REC-I01
fn ora_recon_duplicate_application_is_idempotent() {
    let root = temp_dir("recon-idempotent");
    let mut ledger = PromptAcceptanceLedger::new(Some(&root));
    let digest = send_input_digest(&send_input("hello"));

    assert!(matches!(
        ledger.admit_send("host", "nonce-1", &digest).unwrap(),
        SendAdmission::Dispatch
    ));
    ledger
        .record_pending("host", "nonce-1", digest.clone(), "agent-1", None)
        .unwrap();
    assert_eq!(ledger.record_count(), 1);

    for _ in 0..3 {
        assert!(matches!(
            ledger.admit_send("host", "nonce-1", &digest).unwrap(),
            SendAdmission::Duplicate(_)
        ));
    }
    assert_eq!(
        ledger.record_count(),
        1,
        "repeated duplicate application must stay a single record"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
// oracle: REC-I05
// regression: REG-TRANSCRIPT-REPLACE-001
fn ora_recon_divergent_payload_cannot_overwrite_committed_text() {
    let root = temp_dir("recon-divergent");
    let mut ledger = PromptAcceptanceLedger::new(Some(&root));
    let committed = send_input_digest(&send_input("the committed final answer"));
    ledger.admit_send("host", "nonce-x", &committed).unwrap();
    ledger
        .record_pending(
            "host",
            "nonce-x",
            committed.clone(),
            "agent-1",
            Some("t1u".into()),
        )
        .unwrap();
    ledger.mark_accepted("host", "nonce-x");

    let partial = send_input_digest(&send_input(""));
    assert!(
        matches!(
            ledger.admit_send("host", "nonce-x", &partial),
            Err(PromptAcceptanceError::DigestMismatch { .. })
        ),
        "a divergent or empty payload must fail closed instead of replacing committed text"
    );
    match ledger.lookup("host", "nonce-x") {
        AcceptanceLookup::Found(record) => {
            assert_eq!(record.status, AcceptanceStatus::Accepted);
            assert_eq!(record.echo_entry_id.as_deref(), Some("t1u"));
        }
        other => panic!("committed record must survive, got {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
// oracle: REC-I07
fn ora_recon_reload_converges_to_the_same_settled_state() {
    let root = temp_dir("recon-reload");
    let digest = send_input_digest(&send_input("final"));
    {
        let mut ledger = PromptAcceptanceLedger::new(Some(&root));
        ledger.admit_send("host", "accepted", &digest).unwrap();
        ledger
            .record_pending(
                "host",
                "accepted",
                digest.clone(),
                "agent-1",
                Some("t1u".into()),
            )
            .unwrap();
        ledger.mark_accepted("host", "accepted");
        ledger.admit_send("host", "rejected", &digest).unwrap();
        ledger
            .record_pending("host", "rejected", digest.clone(), "agent-1", None)
            .unwrap();
        ledger.mark_rejected("host", "rejected", "OFFLINE");
        ledger.dispose();
    }

    let mut reopened = PromptAcceptanceLedger::new(Some(&root));
    assert!(matches!(
        reopened.lookup("host", "accepted"),
        AcceptanceLookup::Found(record)
            if record.status == AcceptanceStatus::Accepted
                && record.echo_entry_id.as_deref() == Some("t1u")
    ));
    assert!(matches!(
        reopened.lookup("host", "rejected"),
        AcceptanceLookup::Found(record) if record.status == AcceptanceStatus::Rejected
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
// oracle: REC-I08
// regression: REG-CROSS-CONVERSATION-001
fn ora_recon_scope_prevents_cross_conversation_correlation() {
    let root = temp_dir("recon-scope");
    let mut ledger = PromptAcceptanceLedger::new(Some(&root));
    let digest = send_input_digest(&send_input("scoped"));

    ledger.admit_send("host", "nonce-a", &digest).unwrap();
    ledger
        .record_pending(
            "host",
            "nonce-a",
            digest.clone(),
            "agent-a",
            Some("t1u".into()),
        )
        .unwrap();
    ledger.admit_send("host", "nonce-b", &digest).unwrap();
    ledger
        .record_pending(
            "host",
            "nonce-b",
            digest.clone(),
            "agent-b",
            Some("t1u".into()),
        )
        .unwrap();

    assert_eq!(
        ledger.record_count(),
        2,
        "two addressed owners must not be collapsed into one record"
    );
    assert!(matches!(
        ledger.lookup("host", "nonce-a"),
        AcceptanceLookup::Found(_)
    ));
    assert!(matches!(
        ledger.lookup("host", "nonce-b"),
        AcceptanceLookup::Found(_)
    ));
    assert_eq!(
        ledger.lookup("other-host", "nonce-a"),
        AcceptanceLookup::NotFound,
        "an acceptance record must not be visible outside its account scope"
    );
    let _ = fs::remove_dir_all(root);
}
