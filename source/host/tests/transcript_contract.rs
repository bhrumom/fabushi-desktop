use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mahayana_host_runtime::extensions::transcript::prompt_acceptance_ledger::{
    AcceptanceLookup, AcceptanceStatus, MAX_RECORDS, PromptAcceptanceError,
    PromptAcceptanceLedger, SendAdmission, SendInput, canonical_send_input,
    send_input_digest,
};
use uuid::Uuid;

fn temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("fabushi-host-{name}-{}", Uuid::new_v4()))
}

fn sample_input(prompt: &str) -> SendInput {
    SendInput {
        agent_id: Some("agent-1".into()),
        prompt: prompt.into(),
        rich_text: None,
        reply_to_id: None,
        is_fork: false,
        attachment_paths: vec!["/tmp/a.txt".into()],
        attachment_names: vec!["a.txt".into()],
    }
}

#[test]
fn acceptance_digest_matches_canonical_wire_shape() {
    let input = sample_input("hello");
    assert_eq!(
        canonical_send_input(&input),
        r#"["agent-1","hello",null,null,false,["/tmp/a.txt"],["a.txt"]]"#
    );
    assert_eq!(
        send_input_digest(&input),
        "8609d62307b7aa0b2fef3270029a3b7c414928fc5b03b23eae366dd5a94aaf14"
    );
}

#[test]
fn acceptance_ledger_replays_nonce_outcome_and_rejects_digest_mismatch() {
    let root = temp_dir("acceptance");
    let clock = Arc::new(AtomicU64::new(1_000));
    let now = {
        let clock = clock.clone();
        Arc::new(move || clock.load(Ordering::SeqCst))
    };
    let mut ledger = PromptAcceptanceLedger::with_clock(Some(&root), now);
    let digest = send_input_digest(&sample_input("hello"));

    assert_eq!(
        ledger.admit_send("host", "nonce-1", &digest).unwrap(),
        SendAdmission::Dispatch
    );
    let pending = ledger
        .record_pending(
            "host",
            "nonce-1",
            digest.clone(),
            "agent-1",
            Some("t1u".into()),
        )
        .unwrap();
    assert_eq!(pending.status, AcceptanceStatus::Pending);
    assert!(matches!(
        ledger.admit_send("host", "nonce-1", &digest).unwrap(),
        SendAdmission::Duplicate(record) if record.status == AcceptanceStatus::Pending
    ));

    let mismatch = send_input_digest(&sample_input("different"));
    assert!(matches!(
        ledger.admit_send("host", "nonce-1", &mismatch),
        Err(PromptAcceptanceError::DigestMismatch { .. })
    ));

    ledger.mark_accepted("host", "nonce-1");
    assert!(matches!(
        ledger.lookup("host", "nonce-1"),
        AcceptanceLookup::Found(record) if record.status == AcceptanceStatus::Accepted
    ));
    ledger.clear_unless_accepted("host", "nonce-1");
    assert!(matches!(
        ledger.lookup("host", "nonce-1"),
        AcceptanceLookup::Found(record) if record.status == AcceptanceStatus::Accepted
    ));

    clock.store(1_100, Ordering::SeqCst);
    ledger
        .record_pending("host", "nonce-2", digest.clone(), "agent-1", None)
        .unwrap();
    ledger.mark_rejected("host", "nonce-2", "OFFLINE");
    assert!(matches!(
        ledger.admit_send("host", "nonce-2", &digest),
        Err(PromptAcceptanceError::Rejected { code }) if code == "OFFLINE"
    ));

    ledger.dispose();
    drop(ledger);

    let mut reopened = PromptAcceptanceLedger::new(Some(&root));
    assert!(matches!(
        reopened.lookup("host", "nonce-1"),
        AcceptanceLookup::Found(record)
            if record.status == AcceptanceStatus::Accepted && record.echo_entry_id.as_deref() == Some("t1u")
    ));
    assert!(matches!(
        reopened.lookup("host", "nonce-2"),
        AcceptanceLookup::Found(record)
            if record.status == AcceptanceStatus::Rejected
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn acceptance_ledger_tracks_eviction_and_corrupt_history_as_unknown_durability() {
    let root = temp_dir("gaps");
    let clock = Arc::new(AtomicU64::new(10));
    let now = {
        let clock = clock.clone();
        Arc::new(move || clock.load(Ordering::SeqCst))
    };
    let mut ledger = PromptAcceptanceLedger::with_clock(Some(&root), now);
    let digest = send_input_digest(&sample_input("hello"));

    for index in 0..=MAX_RECORDS {
        clock.store(index as u64 + 10, Ordering::SeqCst);
        ledger
            .record_pending(
                "host",
                format!("nonce-{index}"),
                digest.clone(),
                "agent-1",
                None,
            )
            .unwrap();
        ledger.mark_accepted("host", &format!("nonce-{index}"));
    }
    assert_eq!(ledger.record_count(), MAX_RECORDS);
    assert!(ledger.gaps().eviction_horizon_ms.is_some());
    assert_eq!(
        ledger.lookup("host", "never-seen"),
        AcceptanceLookup::UnknownDurability
    );
    ledger.dispose();
    drop(ledger);

    let file = root.join("send-acceptance.json");
    fs::write(&file, "{not-json").unwrap();
    clock.store(9_999, Ordering::SeqCst);
    let now = {
        let clock = clock.clone();
        Arc::new(move || clock.load(Ordering::SeqCst))
    };
    let mut damaged = PromptAcceptanceLedger::with_clock(Some(&root), now);
    assert_eq!(
        damaged.lookup("host", "unknown"),
        AcceptanceLookup::UnknownDurability
    );
    assert_eq!(damaged.gaps().corrupt_reset_at_ms, Some(9_999));
    let has_backup = fs::read_dir(&root)
        .unwrap()
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().contains("corrupt-9999"));
    assert!(has_backup);
    let _ = fs::remove_dir_all(root);
}
