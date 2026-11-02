use std::fs;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::settings_file::{
    get_sand_settings_path, read_sand_settings_file,
};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::inference::model_picker_service::{
    ApplySessionModelSelectionParams, ModelCatalogEntry, SessionModelSelectionRequest,
    SessionModelSelectionRuntime,
};
use mahayana_host_runtime::extensions::inference::typed_replay_serialization::serialize_responses_replay;
use mahayana_host_runtime::extensions::session::session_profile_files::AgentProfileUpdate;
use mahayana_host_runtime::extensions::memory::memory_service::MemoryKind;
use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::workflows::workflow_library::WorkflowSpec;
use mahayana_host_runtime::extensions::session::agent_db_serde::{
    AwaitingUserResponse, SpendGuardState,
};
use mahayana_host_runtime::extensions::session::agent_db_transcript_pages::{
    TranscriptPageQuery, TranscriptWindowQuery,
};
use mahayana_host_runtime::extensions::session::agent_db::{
    AgentDbSubscription, CompactionCommit, CompactionCommitOutcome, SandAgentDb,
    SessionWorkAdmissionRecord,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::typed_replay::{
    ReplayContentPart, ReplayRole, TypedReplayEntry,
};
use mahayana_host_runtime::extensions::transcript::send_message_shaping::{
    UserMessageOptions, create_send_message_entry, create_user_message,
};
use mahayana_host_runtime::storage::store_db::get_sand_agent_db_write_generation;
use mahayana_host_runtime::transcript_mutation_events::{
    TranscriptMutationSubscription, subscribe_transcript_mutations,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-agent-session-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn compaction_session_snapshot(
    owner: &SandAgentDb,
) -> (Vec<(i64, String, String)>, Vec<(String, String)>, u64) {
    let observation = rusqlite::Connection::open_with_flags(
        owner.db_path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).expect("read-only compaction observation");
    let mut transcript = observation.prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq").unwrap();
    let rows = transcript.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    let mut kv = observation.prepare("SELECT key, value FROM kv ORDER BY key").unwrap();
    let values = kv.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    (rows, values, get_sand_agent_db_write_generation(owner.db_path()))
}

fn observe_compaction_session_publications(
    owner: &SandAgentDb,
) -> (Arc<AtomicUsize>, [AgentDbSubscription; 2], TranscriptMutationSubscription) {
    let count = Arc::new(AtomicUsize::new(0));
    let subscriptions = ["lifecycleRevision", "sessionWorkAdmission"].map(|key| {
        let count = Arc::clone(&count);
        owner.subscribe_metadata(key, Arc::new(move || { count.fetch_add(1, Ordering::SeqCst); }))
    });
    let agent_id = owner.read_metadata().unwrap()["agentId"].as_str().unwrap().to_string();
    let transcript = subscribe_transcript_mutations({
        let count = Arc::clone(&count);
        move |mutation| {
            if mutation.get("agentId").and_then(serde_json::Value::as_str) == Some(agent_id.as_str()) {
                count.fetch_add(1, Ordering::SeqCst);
            }
        }
    });
    (count, subscriptions, transcript)
}

fn seed_session_compaction(production: &ProductionSessionWorkers, agent_id: &str) -> CompactionCommit {
    assert_eq!(production.append_agent_transcript_entries(agent_id, &[
        create_user_message("old", "earlier request", UserMessageOptions::default(), 1.0),
        create_user_message("tail", "retained request", UserMessageOptions::default(), 2.0),
    ]).unwrap(), 2);
    CompactionCommit {
        entry_id: "compaction-1".into(),
        expected_transcript_tail_id: "tail".into(),
        first_kept_entry_id: Some("tail".into()),
        summary: "earlier request summarized".into(),
        tokens_before: 123,
        details: None,
        source_turn_id: "turn-1".into(),
        expected_lifecycle_revision: production.read_session_work_admission(agent_id)
            .unwrap().expect("production admission").lifecycle_revision,
    }
}

#[test]
fn production_compaction_preserves_text_send_messages_for_current_and_later_replay() {
    let root = temp_root("compacted-send-message");
    let agents = root.join("agents");
    let production = ProductionSessionWorkers::with_agents_root(&agents, 500);
    let record = production
        .materialize_new_session(None, "user", None)
        .expect("materialized production session");
    let retained_text = "  Original answer\n\twith spacing  ";
    let entries = vec![
        create_user_message("old", "old request", UserMessageOptions::default(), 1.0),
        create_user_message("kept", "retained request", UserMessageOptions::default(), 2.0),
        create_send_message_entry(
            "sent-1",
            serde_json::json!({"type": "text", "content": retained_text}),
            3.0,
        ),
    ];
    assert_eq!(
        production.append_agent_transcript_entries(&record.id, &entries).expect("history"),
        3
    );
    assert!(production
        .read_agent_typed_compacted_transcript_entries(&record.id)
        .expect("no active compaction")
        .is_empty());
    // Capture the same admission generation as the production Runner, then
    // let an ordinary metadata write demonstrate the independent revision.
    let revision = production.read_session_work_admission(&record.id)
        .expect("production admission").expect("materialized admission").lifecycle_revision;
    let owner = production.open_agent_db_owner(&record.id).expect("owner");
    assert!(owner.set_metadata("name", serde_json::json!("renamed before compaction")).unwrap());
    assert_ne!(owner.read_metadata().unwrap()["lifecycleRevision"], revision);
    let commit = CompactionCommit {
        entry_id: "compaction-1".into(),
        expected_transcript_tail_id: "sent-1".into(),
        first_kept_entry_id: Some("kept".into()),
        summary: "earlier request summarized".into(),
        tokens_before: 123,
        details: None,
        source_turn_id: "turn-1".into(),
        expected_lifecycle_revision: revision.clone(),
    };
    let CompactionCommitOutcome::Committed(first_compaction) = production
        .commit_agent_compaction(&record.id, &commit).expect("compaction") else {
        panic!("production admission must commit despite metadata divergence");
    };
    let first_revision = production.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision;
    assert_ne!(first_revision, revision);
    assert_eq!(first_compaction["sourceLifecycleRevision"], revision);
    assert_eq!(first_compaction["lifecycleRevision"], first_revision);
    assert_eq!(owner.read_metadata().unwrap()["lifecycleRevision"], first_revision);
    let durable = production.read_agent_transcript_entries(&record.id).expect("durable history");
    assert_eq!(durable.len(), 4);
    assert_eq!(&durable[..3], entries.as_slice());
    let replay = production
        .read_agent_typed_compacted_transcript_entries(&record.id)
        .expect("typed compacted replay with SendMessage");
    assert_eq!(replay.len(), 3);
    assert!(matches!(
        &replay[0],
        TypedReplayEntry::CompactionSummary(summary)
            if summary.summary == "earlier request summarized" && summary.tokens_before == 123
    ));
    let TypedReplayEntry::Message(user) = &replay[1] else {
        panic!("retained user must remain a message");
    };
    assert_eq!(user.id.as_deref(), Some("kept"));
    assert_eq!(user.role, ReplayRole::User);
    let TypedReplayEntry::Message(assistant) = &replay[2] else {
        panic!("canonical SendMessage must become an assistant message");
    };
    assert_eq!(assistant.id.as_deref(), Some("sent-1"));
    assert_eq!(assistant.role, ReplayRole::Assistant);
    assert_eq!(assistant.content, vec![ReplayContentPart::Text(retained_text.into())]);
    let wire = serialize_responses_replay(&replay).expect("Responses replay");
    assert_eq!(wire.len(), 3);
    assert_eq!(wire[1], serde_json::json!({
        "role": "user", "content": [{"type": "input_text", "text": "retained request"}]
    }));
    assert_eq!(wire[2], serde_json::json!({
        "role": "assistant", "content": [{"type": "input_text", "text": retained_text}]
    }));

    let later_text = "\n  Follow-up answer\t ";
    assert_eq!(
        production.append_agent_transcript_entries(&record.id, &[
            create_user_message("next-user", "continue", UserMessageOptions::default(), 4.0),
            create_send_message_entry(
                "sent-2",
                serde_json::json!({"type": "text", "content": later_text}),
                5.0,
            ),
        ]).expect("subsequent turn history"),
        2
    );
    let later_replay = production
        .read_agent_typed_compacted_transcript_entries(&record.id)
        .expect("subsequent text SendMessage replay");
    assert_eq!(later_replay.len(), 5);
    assert_eq!(&later_replay[..3], replay.as_slice());
    let TypedReplayEntry::Message(later_assistant) = &later_replay[4] else {
        panic!("subsequent SendMessage must remain a message");
    };
    assert_eq!(later_assistant.id.as_deref(), Some("sent-2"));
    assert_eq!(later_assistant.role, ReplayRole::Assistant);
    assert_eq!(later_assistant.content, vec![ReplayContentPart::Text(later_text.into())]);
    let later_wire = serialize_responses_replay(&later_replay).expect("subsequent Responses replay");
    assert_eq!(later_wire.len(), 5);
    assert_eq!(&later_wire[..3], wire.as_slice());
    assert_eq!(later_wire[4], serde_json::json!({
        "role": "assistant", "content": [{"type": "input_text", "text": later_text}]
    }));

    let second_commit = CompactionCommit {
        entry_id: "compaction-2".into(),
        expected_transcript_tail_id: "sent-2".into(),
        first_kept_entry_id: Some("next-user".into()),
        summary: "earlier request and first answer summarized".into(),
        tokens_before: 456,
        details: None,
        source_turn_id: "turn-2".into(),
        expected_lifecycle_revision: production.read_session_work_admission(&record.id)
            .unwrap().unwrap().lifecycle_revision,
    };
    let CompactionCommitOutcome::Committed(second_compaction) = production
        .commit_agent_compaction(&record.id, &second_commit).expect("successive compaction") else {
        panic!("the next captured admission must permit a second compaction");
    };
    let second_revision = production.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision;
    assert_ne!(second_revision, first_revision);
    assert_eq!(second_compaction["sourceLifecycleRevision"], first_revision);
    assert_eq!(second_compaction["lifecycleRevision"], second_revision);
    assert_eq!(owner.read_metadata().unwrap()["lifecycleRevision"], second_revision);
    let second_replay = production.read_agent_typed_compacted_transcript_entries(&record.id).unwrap();
    assert_eq!(second_replay.len(), 3);
    assert!(matches!(&second_replay[0], TypedReplayEntry::CompactionSummary(summary)
        if summary.summary == second_commit.summary && summary.tokens_before == 456));
    assert_eq!(&second_replay[1..], &later_replay[3..]);
    let second_wire = serialize_responses_replay(&second_replay).unwrap();
    assert_eq!(&second_wire[1..], &later_wire[3..]);
    let before_old_retry = compaction_session_snapshot(&owner);
    assert_eq!(production.commit_agent_compaction(&record.id, &commit).unwrap(), CompactionCommitOutcome::RejectedStale);
    assert_eq!(compaction_session_snapshot(&owner), before_old_retry);

    production.shutdown();
    let reopened = ProductionSessionWorkers::with_agents_root(&agents, 500);
    let reopened_replay = reopened
        .read_agent_typed_compacted_transcript_entries(&record.id)
        .expect("reopened production replay");
    assert_eq!(reopened_replay, second_replay);
    assert_eq!(serialize_responses_replay(&reopened_replay).expect("reopened Responses replay"), second_wire);
    assert_eq!(reopened.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision, second_revision);
    assert!(matches!(reopened.commit_agent_compaction(&record.id, &second_commit).unwrap(), CompactionCommitOutcome::AlreadyCommitted(_)));
    reopened.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_compaction_rejects_invalid_send_message_without_publishing_boundary() {
    let root = temp_root("compaction-invalid-send-message");
    let production = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    for message in [
        serde_json::json!({"type": "text", "content": {"text": "malformed text"}}),
        serde_json::json!({"type": "attachment", "url": "https://example.com/file.pdf"}),
        serde_json::json!({"type": "text", "content": "caption", "images": [
            {"url": "https://example.com/image.png"}
        ]}),
    ] {
        let record = production
            .materialize_new_session(None, "user", None)
            .expect("materialized production session");
        assert_eq!(
            production.append_agent_transcript_entries(&record.id, &[
                create_user_message("old", "old request", UserMessageOptions::default(), 1.0),
                create_user_message("kept", "retained request", UserMessageOptions::default(), 2.0),
                create_send_message_entry("unsupported", message, 3.0),
            ]).expect("durable candidate history"),
            3
        );
        let owner = production.open_agent_db_owner(&record.id).expect("owner");
        let metadata_before = owner.read_metadata().expect("metadata before");
        let admission_before = production.read_session_work_admission(&record.id)
            .unwrap().expect("actual production admission");
        let generation_before = get_sand_agent_db_write_generation(owner.db_path());
        // A malformed text row is intentionally invisible to presentation
        // decoding. Observe raw durable rows so a decoder drop cannot satisfy
        // the no-write oracle or hide the unsupported retained tail.
        let observation = rusqlite::Connection::open_with_flags(
            owner.db_path(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ).expect("read-only durable observation");
        let raw_rows = || {
            let mut statement = observation
                .prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq ASC")
                .expect("raw transcript query");
            let rows = statement
                .query_map([], |row| Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                )))
                .expect("raw transcript rows");
            rows
                .collect::<Result<Vec<_>, _>>()
                .expect("raw transcript values")
        };
        let rows_before = raw_rows();
        assert_eq!(rows_before.len(), 3);
        let commit = CompactionCommit {
            entry_id: "rejected-compaction".into(),
            expected_transcript_tail_id: "unsupported".into(),
            first_kept_entry_id: Some("kept".into()),
            summary: "must not be published".into(),
            tokens_before: 123,
            details: None,
            source_turn_id: "turn-1".into(),
            expected_lifecycle_revision: admission_before.lifecycle_revision.clone(),
        };
        let error = production.commit_agent_compaction(&record.id, &commit)
            .expect_err("unsupported SendMessage must reject compaction");
        assert!(error.contains("retained transcript cannot be replayed"), "{error}");
        assert!(error.contains("send-message"), "{error}");
        assert_eq!(raw_rows(), rows_before);
        assert_eq!(owner.read_metadata().expect("metadata after rejection"), metadata_before);
        assert_eq!(owner.read_session_work_admission().unwrap(), Some(admission_before));
        assert_eq!(get_sand_agent_db_write_generation(owner.db_path()), generation_before);
        assert!(production
            .read_agent_typed_compacted_transcript_entries(&record.id)
            .expect("rejected boundary remains inactive")
            .is_empty());
    }
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_compaction_cancellation_while_filesystem_lock_is_held_preserves_state() {
    let root = temp_root("compaction-cancel-filesystem-wait");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let record = production.materialize_new_session(None, "user", None).unwrap();
    let commit = seed_session_compaction(&production, &record.id);
    let owner = production.open_agent_db_owner(&record.id).unwrap();
    let before = compaction_session_snapshot(&owner);
    let admission_before = owner.read_session_work_admission().unwrap().unwrap();
    let (publications, _subscriptions, transcript_subscription) = observe_compaction_session_publications(&owner);
    let cancelled = Arc::new(AtomicBool::new(false));
    let lock_held = Arc::new(AtomicBool::new(false));
    let reads = Arc::new(AtomicUsize::new(0));
    let reads_while_locked = Arc::new(AtomicUsize::new(0));

    let (worker, outcome_receiver) = production.with_agent_filesystem_mutation(&record.id, || {
        lock_held.store(true, Ordering::SeqCst);
        let committing_session = Arc::clone(&production);
        let committing_agent_id = record.id.clone();
        let candidate = commit.clone();
        let cancellation = Arc::clone(&cancelled);
        let held = Arc::clone(&lock_held);
        let cancellation_reads = Arc::clone(&reads);
        let premature_reads = Arc::clone(&reads_while_locked);
        let (started_sender, started_receiver) = mpsc::channel();
        let (outcome_sender, outcome_receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let should_cancel = || {
                cancellation_reads.fetch_add(1, Ordering::SeqCst);
                if held.load(Ordering::SeqCst) {
                    premature_reads.fetch_add(1, Ordering::SeqCst);
                }
                cancellation.load(Ordering::SeqCst)
            };
            let _ = started_sender.send(());
            let outcome = committing_session.commit_agent_compaction_with_cancellation(
                &committing_agent_id, &candidate, &should_cancel,
            );
            let _ = outcome_sender.send(outcome);
        });
        // The real Session mutex is held before launching this attempt. The
        // handshake marks its public call; the holder cancels before releasing
        // ownership, without inferring a parked thread from a sleep or timeout.
        started_receiver.recv_timeout(Duration::from_secs(5))
            .map_err(|error| format!("compaction worker did not start: {error}"))?;
        assert!(matches!(outcome_receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        cancelled.store(true, Ordering::SeqCst);
        assert_eq!(compaction_session_snapshot(&owner), before);
        assert_eq!(publications.load(Ordering::SeqCst), 0);
        lock_held.store(false, Ordering::SeqCst);
        Ok((worker, outcome_receiver))
    }).unwrap();

    // Join only after the bounded result so a lock regression cannot hang the
    // test on an unbounded join. No cancellation callback blocks or reenters.
    let outcome = outcome_receiver.recv_timeout(Duration::from_secs(5))
        .expect("cancelled compaction must return after the filesystem lock releases")
        .expect("clean cancellation outcome");
    worker.join().expect("compaction worker returned");
    assert_eq!(outcome, CompactionCommitOutcome::RejectedCancelled);
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    assert_eq!(reads_while_locked.load(Ordering::SeqCst), 0);
    assert_eq!(compaction_session_snapshot(&owner), before);
    assert_eq!(owner.read_session_work_admission().unwrap(), Some(admission_before.clone()));
    assert_eq!(publications.load(Ordering::SeqCst), 0);
    assert!(production.read_agent_typed_compacted_transcript_entries(&record.id).unwrap().is_empty());

    cancelled.store(false, Ordering::SeqCst);
    let should_cancel = || cancelled.load(Ordering::SeqCst);
    let CompactionCommitOutcome::Committed(entry) = production
        .commit_agent_compaction_with_cancellation(&record.id, &commit, &should_cancel).unwrap() else {
        panic!("the same candidate must commit after cancellation is cleared");
    };
    let next_revision = entry["lifecycleRevision"].as_str().unwrap();
    assert_ne!(next_revision, admission_before.lifecycle_revision);
    let mut expected_admission = admission_before;
    expected_admission.lifecycle_revision = next_revision.to_string();
    assert_eq!(owner.read_session_work_admission().unwrap(), Some(expected_admission));
    assert_eq!(owner.read_metadata().unwrap()["lifecycleRevision"], next_revision);
    let after_commit = compaction_session_snapshot(&owner);
    assert_eq!(&after_commit.0[..before.0.len()], before.0.as_slice());
    assert_eq!(after_commit.0.len(), before.0.len() + 1);
    assert_ne!(after_commit.1, before.1);
    assert_eq!(after_commit.2, before.2 + 1);
    assert_eq!(publications.load(Ordering::SeqCst), 3);
    assert_eq!(
        production.commit_agent_compaction_with_cancellation(&record.id, &commit, &should_cancel).unwrap(),
        CompactionCommitOutcome::AlreadyCommitted(entry),
    );
    assert_eq!(compaction_session_snapshot(&owner), after_commit);
    assert_eq!(publications.load(Ordering::SeqCst), 3);
    transcript_subscription.unsubscribe();
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_compaction_archive_restore_fences_saved_plan_and_committed_retry() {
    let root = temp_root("compaction-archive-restore");
    let production = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let record = production.materialize_new_session(None, "user", None).unwrap();
    let saved_plan = seed_session_compaction(&production, &record.id);
    let owner = production.open_agent_db_owner(&record.id).unwrap();
    let session_key = format!("agent:{}:main", record.id);
    let rows_before = compaction_session_snapshot(&owner).0;
    let metadata_before = owner.read_kv("metadata").unwrap();
    let archived = production.transition_session_work_archive(
        &record.id, &session_key, &record.id, Some(&saved_plan.expected_lifecycle_revision),
        true, Some("operator"), Some("pause"), 3.0,
    ).unwrap();
    let restored = production.transition_session_work_archive(
        &record.id, &session_key, &record.id, Some(&archived.lifecycle_revision),
        false, None, None, 4.0,
    ).unwrap();
    assert_ne!(restored.lifecycle_revision, saved_plan.expected_lifecycle_revision);
    assert_eq!(compaction_session_snapshot(&owner).0, rows_before);
    assert_eq!(owner.read_kv("metadata").unwrap(), metadata_before);
    let (publications, _subscriptions, transcript_subscription) = observe_compaction_session_publications(&owner);
    let before_stale_plan = compaction_session_snapshot(&owner);
    assert_eq!(production.commit_agent_compaction(&record.id, &saved_plan).unwrap(), CompactionCommitOutcome::RejectedStale);
    assert_eq!(compaction_session_snapshot(&owner), before_stale_plan);
    assert_eq!(publications.load(Ordering::SeqCst), 0);

    let mut fresh_plan = saved_plan;
    fresh_plan.expected_lifecycle_revision = restored.lifecycle_revision;
    let CompactionCommitOutcome::Committed(entry) = production.commit_agent_compaction(&record.id, &fresh_plan).unwrap() else {
        panic!("a fresh post-restore admission must permit compaction");
    };
    let resulting_revision = entry["lifecycleRevision"].as_str().unwrap();
    assert_eq!(production.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision, resulting_revision);
    assert_eq!(owner.read_metadata().unwrap()["lifecycleRevision"], resulting_revision);
    let archived = production.transition_session_work_archive(
        &record.id, &session_key, &record.id, Some(resulting_revision),
        true, Some("operator"), Some("pause again"), 5.0,
    ).unwrap();
    production.transition_session_work_archive(
        &record.id, &session_key, &record.id, Some(&archived.lifecycle_revision),
        false, None, None, 6.0,
    ).unwrap();
    let before_old_retry = compaction_session_snapshot(&owner);
    let publications_before_retry = publications.load(Ordering::SeqCst);
    assert_eq!(before_old_retry.0.last().unwrap().1, fresh_plan.entry_id);
    assert_eq!(production.commit_agent_compaction(&record.id, &fresh_plan).unwrap(), CompactionCommitOutcome::RejectedStale);
    assert_eq!(compaction_session_snapshot(&owner), before_old_retry);
    assert_eq!(publications.load(Ordering::SeqCst), publications_before_retry);
    transcript_subscription.unsubscribe();
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_compaction_retains_captured_background_and_media_generations() {
    let root = temp_root("compaction-active-custody");
    let production = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    for media in [false, true] {
        let record = production.materialize_new_session(None, "user", None).unwrap();
        let mut commit = seed_session_compaction(&production, &record.id);
        let owner = production.open_agent_db_owner(&record.id).unwrap();
        let session_key = format!("agent:{}:main", record.id);
        let revision = if media {
            production.capture_session_media_admission(
                &record.id, &session_key, "media-1", Some(&root.join("outgoing.bin")),
            ).unwrap()
        } else {
            production.capture_session_work_admission(&record.id, &session_key, "work-1").unwrap()
        };
        assert_eq!(revision, commit.expected_lifecycle_revision);
        let (publications, _subscriptions, transcript_subscription) = observe_compaction_session_publications(&owner);
        let before_refusal = compaction_session_snapshot(&owner);
        assert_eq!(production.commit_agent_compaction(&record.id, &commit).unwrap(), CompactionCommitOutcome::RejectedStale);
        assert_eq!(compaction_session_snapshot(&owner), before_refusal);
        assert_eq!(publications.load(Ordering::SeqCst), 0);
        if media {
            assert!(production.release_session_media_admission(&record.id, &revision, "media-1").unwrap());
        } else {
            assert!(matches!(production.commit_session_work_result(
                &record.id, &session_key, &revision, "work-1", "result-1", "finished", 0,
            ).unwrap(), mahayana_host_runtime::extensions::session::agent_db::SessionWorkResultCommitOutcome::Committed(_)));
        }
        assert_eq!(production.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision, revision);
        commit.expected_transcript_tail_id = compaction_session_snapshot(&owner).0.last().unwrap().1.clone();
        assert!(matches!(production.commit_agent_compaction(&record.id, &commit).unwrap(), CompactionCommitOutcome::Committed(_)));
        transcript_subscription.unsubscribe();
    }
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_compaction_waits_for_current_generation_cron_settlement() {
    let root = temp_root("compaction-running-cron");
    let production = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let record = production.materialize_new_session(None, "user", None).unwrap();
    let commit = seed_session_compaction(&production, &record.id);
    let owner = production.open_agent_db_owner(&record.id).unwrap();
    // Older-generation running rows remain the existing reconciliation gap;
    // this guard protects custody that the current finish path can settle.
    let historical_key = format!("agent:{}:cron:historical:run-old", record.id);
    assert!(owner.begin_cron_session_run(
        &historical_key, "historical", "run-old", &historical_key, "old-generation", 0.0,
    ).unwrap());
    assert!(production.begin_cron_session_run(&record.id, "automation-1", "run-current", 1.0).unwrap());
    let running = production.list_cron_session_runs(&record.id).unwrap();
    let current = running.iter().find(|row| row.run_id == "run-current").unwrap();
    assert_eq!(current.state, "running");
    assert_eq!(current.lifecycle_revision, commit.expected_lifecycle_revision);
    let (publications, _subscriptions, transcript_subscription) = observe_compaction_session_publications(&owner);
    let before_refusal = compaction_session_snapshot(&owner);
    assert_eq!(production.commit_agent_compaction(&record.id, &commit).unwrap(), CompactionCommitOutcome::RejectedStale);
    assert_eq!(compaction_session_snapshot(&owner), before_refusal);
    assert_eq!(production.list_cron_session_runs(&record.id).unwrap(), running);
    assert_eq!(publications.load(Ordering::SeqCst), 0);
    assert!(production.finish_cron_session_run(
        &record.id, "automation-1", "run-current", "ok", 2.0, Some("completed"),
    ).unwrap());
    let settled = production.list_cron_session_runs(&record.id).unwrap();
    let current = settled.iter().find(|row| row.run_id == "run-current").unwrap();
    assert_eq!(current.state, "terminal");
    assert_eq!(current.terminal_outcome.as_deref(), Some("ok"));
    assert_eq!(current.lifecycle_revision, commit.expected_lifecycle_revision);
    assert_eq!(production.read_session_work_admission(&record.id).unwrap().unwrap().lifecycle_revision, commit.expected_lifecycle_revision);
    assert!(matches!(production.commit_agent_compaction(&record.id, &commit).unwrap(), CompactionCommitOutcome::Committed(_)));
    assert_eq!(publications.load(Ordering::SeqCst), 3);
    assert_eq!(production.list_cron_session_runs(&record.id).unwrap(), settled);
    transcript_subscription.unsubscribe();
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn active_agent_pointer_roster_and_settings_are_owned_by_session_facade() {
    let root = temp_root("active");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let first = store
        .create_session(None, "user", None)
        .expect("first agent");
    let second = store
        .create_session(None, "user", None)
        .expect("second agent");

    assert_eq!(store.read_active_agent_id(), None);
    store.write_active_agent_id(&second.id).expect("active pointer");
    assert_eq!(store.read_active_agent_id().as_deref(), Some(second.id.as_str()));

    assert_eq!(store.count_owned_agents().expect("owned agents"), 2);
    let agents_list = store.list_agents().expect("roster");
    // Frozen Grok roster semantics hide blank non-active agents without a durable
    // footprint; the active blank agent remains visible.
    assert_eq!(agents_list.len(), 1);
    assert_eq!(
        agents_list.iter().filter(|agent| agent.is_active).count(),
        1
    );
    assert!(agents_list.iter().any(|agent| agent.id == second.id && agent.is_active));

    store
        .set_session_notify_on_updates(&first.id, false)
        .expect("notifications");
    store
        .set_session_hidden_from_sidebar(&first.id, true)
        .expect("hidden");
    let settings = read_sand_settings_file(get_sand_settings_path(agents.join(&first.id)));
    assert!(!settings.notify_on_agent_updates);
    assert!(settings.hidden_from_sidebar);

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delete_session_closes_owned_blob_store_clears_directory_and_publishes_removal() {
    let root = temp_root("delete");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let _ = store.open_session(&record.id).expect("open existing");
    assert_eq!(production.active_agent_store_owner_count(), 1);
    let removals = Arc::new(AtomicUsize::new(0));
    let subscription = subscribe_transcript_mutations({
        let removals = Arc::clone(&removals);
        let id = record.id.clone();
        move |mutation| {
            if mutation.get("kind").and_then(serde_json::Value::as_str) == Some("agent-removed")
                && mutation.get("agentId").and_then(serde_json::Value::as_str) == Some(id.as_str())
            {
                removals.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    store.delete_session(&record.id).expect("delete");
    assert!(!store.agent_dir_exists(&record.id));
    assert!(!store.agent_exists(&record.id));
    assert_eq!(production.active_agent_store_owner_count(), 0);
    assert_eq!(removals.load(Ordering::SeqCst), 1);

    subscription.unsubscribe();
    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn deletion_fence_rejects_new_transcript_work_before_durable_write() {
    let root = temp_root("work-admission-delete-fence");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let session_key = production
        .read_session_work_admission(&record.id)
        .expect("read session admission")
        .expect("main session admission")
        .session_key;
    // Retain an observation handle before custody starts. Public owner opens
    // are deliberately rejected while the deletion fast path is active.
    let owner = production
        .open_agent_db_owner(&record.id)
        .expect("owner before deletion");
    let entries_before = owner.get_transcript_entries().expect("initial transcript");
    let awaiting_before = owner
        .get_awaiting_user_response()
        .expect("initial awaiting state");

    production
        .begin_agent_delete(&record.id)
        .expect("publish deletion fence");
    let error = production
        .append_agent_transcript_entries(
            &record.id,
            &[serde_json::json!({
                "id": "blocked-entry",
                "kind": "message",
                "role": "user",
                "content": "must not be admitted",
            })],
        )
        .expect_err("deletion-pending work must be rejected");
    assert!(error.contains("new session work is not admitted"));
    let awaiting = AwaitingUserResponse {
        tab_id: "tab-delete-fence".into(),
        reason: "approval".into(),
        since: 41.0,
    };
    let awaiting_error = production
        .set_agent_awaiting_user_response(&record.id, Some(&awaiting))
        .expect_err("deletion-pending awaiting state must be rejected");
    assert!(awaiting_error.contains("new session work is not admitted"));
    let awaiting_tab_error = production
        .set_agent_awaiting_user_response_for_tab(&record.id, &awaiting.tab_id, None, None)
        .expect_err("deletion-pending tab awaiting state must be rejected");
    assert!(awaiting_tab_error.contains("new session work is not admitted"));
    let session_admitted_error = production
        .append_agent_transcript_entries_if_session_admitted(
            &record.id,
            &session_key,
            &[serde_json::json!({
                "id": "blocked-session-entry",
                "kind": "message",
                "role": "user",
                "content": "session-bound work must not be persisted during deletion",
                "sessionKey": session_key.clone(),
            })],
        )
        .expect_err("deletion-pending session append must be rejected");
    assert!(session_admitted_error.contains("new session work is not admitted"));
    let introduction_error = production
        .set_agent_introduction_pending(&record.id, false)
        .expect_err("deletion-pending introduction state must be rejected");
    assert!(introduction_error.contains("new session work is not admitted"));
    let spend_guard_error = production
        .set_agent_automation_spend_guard_state(&record.id, &SpendGuardState::default())
        .expect_err("deletion-pending spend-guard state must be rejected");
    assert!(spend_guard_error.contains("new session work is not admitted"));
    let memory_snapshot_error = production
        .set_agent_memory_prompt_snapshot(&record.id, &serde_json::json!({"render":"blocked"}))
        .expect_err("deletion-pending memory snapshot must be rejected");
    assert!(memory_snapshot_error.contains("new session work is not admitted"));
    let profile_snapshot_error = production
        .set_agent_profile_prompt_snapshot(&record.id, &serde_json::json!({"name":"blocked"}))
        .expect_err("deletion-pending profile snapshot must be rejected");
    assert!(profile_snapshot_error.contains("new session work is not admitted"));
    let profile_error = production
        .update_agent_profile(
            &record.id,
            &AgentProfileUpdate {
                name: "blocked profile".into(),
                description: "must not write during deletion".into(),
                title: None,
                avatar_shape: None,
                avatar_color: None,
            },
            None,
        )
        .expect_err("deletion-pending profile file mutation must be rejected");
    assert!(profile_error.contains("new session work is not admitted"));
    let avatar_error = production
        .set_agent_avatar_bytes(&record.id, Some(b"blocked-avatar"), None)
        .expect_err("deletion-pending avatar mutation must be rejected");
    assert!(avatar_error.contains("new session work is not admitted"));
    let memory_clear_error = production
        .clear_agent_memories(&record.id)
        .expect_err("deletion-pending external memory clear must be rejected");
    assert!(memory_clear_error.contains("new session work is not admitted"));
    let memory_snapshot_clear_error = production
        .clear_agent_memory_prompt_snapshot(&record.id)
        .expect_err("deletion-pending memory snapshot clear must be rejected");
    assert!(memory_snapshot_clear_error.contains("new session work is not admitted"));
    let memory_remove_error = production
        .remove_agent_memory(&record.id, "blocked-memory")
        .expect_err("deletion-pending external memory removal must be rejected");
    assert!(memory_remove_error.contains("new session work is not admitted"));
    let viewed_error = production
        .mark_agent_viewed(&record.id, 50.0, false)
        .expect_err("deletion-pending viewed state must be rejected");
    assert!(viewed_error.contains("new session work is not admitted"));
    let unread_error = production
        .set_agent_unread(&record.id, true, 51.0)
        .expect_err("deletion-pending unread state must be rejected");
    assert!(unread_error.contains("new session work is not admitted"));
    let partner_error = production
        .add_agent_conversation_partner(&record.id, "partner-delete-fence")
        .expect_err("deletion-pending partner state must be rejected");
    assert!(partner_error.contains("new session work is not admitted"));
    let activity_error = production
        .mark_agent_activity(&record.id, 42.0)
        .expect_err("deletion-pending activity must be rejected");
    assert!(activity_error.contains("new session work is not admitted"));
    assert_eq!(
        owner
            .get_transcript_entries()
            .expect("transcript under deletion fence"),
        entries_before,
        "neither generic nor session-bound rejected work may append a row"
    );
    assert_eq!(
        owner
            .get_awaiting_user_response()
            .expect("awaiting state under deletion fence"),
        awaiting_before,
        "rejected awaiting state mutations must leave the durable value intact"
    );
    production
        .end_agent_delete(&record.id)
        .expect("clear deletion fence");
    assert!(production
        .set_agent_awaiting_user_response(&record.id, Some(&awaiting))
        .expect("awaiting state is admitted after deletion fence clears"));
    assert!(production
        .set_agent_awaiting_user_response_for_tab(&record.id, &awaiting.tab_id, None, None)
        .expect("tab awaiting state is admitted after deletion fence clears"));
    production
        .set_agent_introduction_pending(&record.id, false)
        .expect("introduction state is admitted after deletion fence clears");
    production
        .append_agent_transcript_entries(
            &record.id,
            &[serde_json::json!({
                "id": "admitted-entry",
                "kind": "message",
                "role": "user",
                "content": "admitted after cleanup fence",
            })],
        )
        .expect("work is admitted after deletion fence clears");
    let entries = owner
        .get_transcript_entries()
        .expect("transcript after admission resumes");
    assert_eq!(entries.len(), entries_before.len() + 1);
    assert!(entries.iter().any(|entry| entry["id"] == "admitted-entry"));
    assert!(!entries.iter().any(|entry| {
        entry["id"] == "blocked-entry" || entry["id"] == "blocked-session-entry"
    }));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn model_selection_does_not_report_applied_when_owner_rejects_persistence() {
    let root = temp_root("model-selection-delete-fence");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("session");
    let selection_before = store
        .read_session_model_entry(&record.id)
        .expect("selection before deletion");
    let owner = production
        .open_agent_db_owner(&record.id)
        .expect("owner before deletion");
    let persisted_before = owner
        .read_kv("sessionModelSelection")
        .expect("persisted selection before deletion");

    production
        .begin_agent_delete(&record.id)
        .expect("publish deletion fence");

    let catalog = [ModelCatalogEntry {
        provider: "cursor".into(),
        id: "model-a".into(),
        name: None,
        context_window: None,
        context_tokens: Some(8192),
        api: None,
        base_url: None,
        reasoning: None,
    }];
    let result = store.apply_session_model_selection(
        &record.id,
        ApplySessionModelSelectionParams {
            agent_id: &record.id,
            session_key: &format!("agent:{}:main", record.id),
            default_provider: "cursor",
            default_model: "model-default",
            current_provider: "cursor",
            current_model: "model-default",
            model_catalog: &catalog,
            visibility_policy: None,
            operator_authority: None,
            placement_context: None,
            candidate_execution_mode: None,
            initial_generation: None,
            current_generation: None,
            request: SessionModelSelectionRequest {
                provider: "cursor".into(),
                model: "model-a".into(),
                is_default: false,
                reset_to_default: false,
                alias: None,
                profile_override: None,
                runtime: SessionModelSelectionRuntime::Unchanged,
            },
        },
        100,
    )
    .expect_err("rejected persistence must not report Applied");
    // Deletion now rejects at owner admission, before the model policy or
    // commit edge can report an outcome.
    assert!(result.contains("new session work is not admitted"));
    assert_eq!(
        owner
            .read_kv("sessionModelSelection")
            .expect("selection under deletion fence"),
        persisted_before,
        "a rejected selection must not write the owner-local override"
    );

    production
        .end_agent_delete(&record.id)
        .expect("clear deletion fence");
    assert_eq!(
        store
            .read_session_model_entry(&record.id)
            .expect("selection after deletion fence"),
        selection_before
    );
    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn archived_main_session_rejects_routed_work_admission() {
    let root = temp_root("work-admission-archive");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let current = production
        .read_session_work_admission(&record.id)
        .expect("read admission")
        .expect("main admission");
    let mut archived = SessionWorkAdmissionRecord {
        archived_at_ms: Some(100.0),
        archived_by: Some("human-1".into()),
        archive_reason: Some("review".into()),
        lifecycle_revision: "archived-revision".into(),
        ..current.clone()
    };
    assert!(production
        .compare_and_set_session_work_admission(
            &record.id,
            Some(&current.lifecycle_revision),
            &archived,
        )
        .expect("archive transition"));
    let error = production
        .assert_session_work_admission(&record.id, Some(&current.session_key))
        .expect_err("archived session must reject work");
    assert!(error.contains("is archived"));
    let transcript_error = production
        .append_agent_transcript_entries(
            &record.id,
            &[serde_json::json!({
                "id": "archived-entry",
                "kind": "message",
                "role": "user",
                "content": "must not bypass archive fence",
            })],
        )
        .expect_err("archived session must reject agent-only transcript writes");
    assert!(transcript_error.contains("session is archived"));
    archived.archived_at_ms = None;
    archived.lifecycle_revision = "restored-revision".into();
    assert!(production
        .compare_and_set_session_work_admission(
            &record.id,
            Some("archived-revision"),
            &archived,
        )
        .expect("restore transition"));
    production
        .assert_session_work_admission(&record.id, Some(&current.session_key))
        .expect("restored session admits work");
    let mismatch = production
        .assert_session_work_admission(&record.id, Some("agent:wrong:main"))
        .expect_err("rebound session key must fail closed");
    assert!(mismatch.contains("key mismatch"));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_delete_publishes_and_clears_owner_local_deletion_fence() {
    let root = temp_root("work-admission-deletion-fence");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let owner = production
        .open_agent_db_owner(&record.id)
        .expect("owner before deletion");
    let initial = owner
        .read_session_work_admission()
        .expect("initial admission")
        .expect("admission");

    production
        .begin_agent_delete(&record.id)
        .expect("publish deletion fence");
    let read_error = production
        .read_session_work_admission(&record.id)
        .expect_err("ordinary owner opens must remain fenced");
    assert!(read_error.contains("owner open is not admitted"));
    let fenced = owner
        .read_session_work_admission()
        .expect("observe durable admission through retained handle")
        .expect("admission");
    assert!(fenced.deletion_pending);
    assert_eq!(fenced.session_key, initial.session_key);
    assert_eq!(fenced.session_id, initial.session_id);
    production
        .end_agent_delete(&record.id)
        .expect("clear deletion fence");
    assert!(!production
        .read_session_work_admission(&record.id)
        .expect("read cleared admission")
        .expect("admission")
        .deletion_pending);

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_backend_target_lookup_distinguishes_agent_absence_from_unavailable_state() {
    let root = temp_root("backend-target-agent-state");
    fs::create_dir(&root).expect("exclusive fixture root");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&production));
    let missing_backend = stable_automation_id("missing-agent", "daily");
    assert!(sessions.find_automation_for_backend_delivery("missing-agent", &missing_backend)
        .expect("checked missing agent").is_none());
    assert!(!agents.join("missing-agent").exists());

    for invalid in ["", "..", "../outside", "with/slash", "with\\slash", " spaced "] {
        assert!(sessions.find_automation_for_backend_delivery(invalid, &missing_backend).is_err(),
            "unsafe agent identity must refuse: {invalid:?}");
    }
    production.begin_agent_delete("missing-agent").unwrap();
    assert!(sessions.find_automation_for_backend_delivery("missing-agent", &missing_backend)
        .expect_err("pending deletion precedes missing-agent completion")
        .contains("deletion cleanup"));
    production.end_agent_delete("missing-agent").unwrap();
    assert!(sessions.find_automation_for_backend_delivery("missing-agent", &missing_backend)
        .unwrap().is_none());

    let malformed_agent = agents.join("not-a-directory");
    fs::write(&malformed_agent, b"agent path sentinel").unwrap();
    assert!(production.find_automation_for_backend_delivery("not-a-directory", &missing_backend)
        .is_err());
    assert_eq!(fs::read(&malformed_agent).unwrap(), b"agent path sentinel");
    let no_db = agents.join("missing-db");
    fs::create_dir(&no_db).unwrap();
    assert!(sessions.find_automation_for_backend_delivery("missing-db", &missing_backend).is_err());
    assert!(!no_db.join("store.db").exists(), "lookup must not materialize a missing DB");
    fs::create_dir(no_db.join("store.db")).unwrap();
    fs::write(no_db.join("store.db/sentinel"), b"not a database").unwrap();
    assert!(sessions.find_automation_for_backend_delivery("missing-db", &missing_backend).is_err());
    assert_eq!(fs::read(no_db.join("store.db/sentinel")).unwrap(), b"not a database");

    let record = sessions.create_session(None, "user", None).unwrap();
    assert!(sessions.find_automation_for_backend_delivery(&record.id, &missing_backend)
        .expect("admitted agent with no automations").is_none());
    let created = sessions.create_agent_automation(&record.id, &AutomationSpec {
        name: "Checked target".into(),
        prompt: "Inspect the queue".into(),
        trigger: serde_json::json!({"type":"cron","schedule":"@daily"}),
        is_enabled: Some(true),
    }).unwrap();
    let automation = &created[0];
    let backend_id = stable_automation_id(&record.id, &automation.id);
    let found = sessions.find_automation_for_backend_delivery(&record.id, &backend_id)
        .unwrap().expect("live target");
    assert_eq!(found.agent_id, record.id);
    assert_eq!(found.automation.id, automation.id);
    let definition_before = fs::read(&automation.file_path).unwrap();
    let owner = production.open_agent_db_owner(&record.id).unwrap();
    assert!(owner.write_kv("sessionWorkAdmission", "{unreadable-admission").unwrap());
    for target in [&backend_id, &missing_backend] {
        assert!(sessions.find_automation_for_backend_delivery(&record.id, target).is_err(),
            "malformed durable agent state cannot establish a found or missing target");
    }
    assert_eq!(owner.read_kv("sessionWorkAdmission").unwrap().as_deref(),
        Some("{unreadable-admission"));
    assert_eq!(fs::read(&automation.file_path).unwrap(), definition_before);

    let invalid_root = root.join("not-an-agents-directory");
    fs::write(&invalid_root, b"agents root sentinel").unwrap();
    let invalid_workers = ProductionSessionWorkers::with_agents_root(&invalid_root, 500);
    assert!(invalid_workers.find_automation_for_backend_delivery("missing-agent", &missing_backend)
        .is_err());
    assert_eq!(fs::read(&invalid_root).unwrap(), b"agents root sentinel");
    invalid_workers.shutdown();
    production.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_backend_target_lookup_retains_durable_deletion_refusal_after_reopen() {
    let root = temp_root("backend-target-durable-delete");
    fs::create_dir(&root).expect("exclusive fixture root");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&production));
    let record = sessions.create_session(None, "user", None).unwrap();
    let created = sessions.create_agent_automation(&record.id, &AutomationSpec {
        name: "Retained target".into(),
        prompt: "Inspect retained work".into(),
        trigger: serde_json::json!({"type":"cron","schedule":"@daily"}),
        is_enabled: Some(true),
    }).unwrap();
    let automation_id = created[0].id.clone();
    let backend_id = stable_automation_id(&record.id, &automation_id);
    let missing_backend = stable_automation_id(&record.id, "missing-target");
    let automation_store = sessions.automation_store_for(&record.id).unwrap();
    automation_store.begin_run(&automation_id, "schedule", 100.0, None, Some("retained"),
        Some(&["retained-alias".to_string()])).unwrap().expect("running history");
    assert!(production.begin_cron_session_run(&record.id, &automation_id, "retained", 100.0)
        .unwrap());
    let config_before = fs::read(automation_store.config_path(&automation_id)).unwrap();
    let history_before = fs::read(automation_store.runs_path(&automation_id)).unwrap();
    let cron_before = production.list_cron_session_runs(&record.id).unwrap();
    assert!(sessions.find_automation_for_backend_delivery(&record.id, &backend_id)
        .unwrap().is_some());

    production.begin_agent_delete(&record.id).unwrap();
    assert!(sessions.find_automation_for_backend_delivery(&record.id, &backend_id)
        .expect_err("fast deletion fence").contains("deletion cleanup"));
    production.clear_agent_delete_fast_path(&record.id);
    for target in [&backend_id, &missing_backend] {
        assert!(sessions.find_automation_for_backend_delivery(&record.id, target)
            .expect_err("durable deletion fence").contains("durable deletion cleanup"));
    }
    assert_eq!(production.list_cron_session_runs(&record.id).unwrap(), cron_before);
    production.shutdown();
    drop(sessions);
    drop(production);

    let reopened = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&reopened));
    assert!(!reopened.is_agent_being_deleted(&record.id), "fresh volatile owner");
    for target in [&backend_id, &missing_backend] {
        assert!(sessions.find_automation_for_backend_delivery(&record.id, target)
            .expect_err("reopened durable deletion fence").contains("durable deletion cleanup"));
    }
    assert_eq!(fs::read(automation_store.config_path(&automation_id)).unwrap(), config_before);
    assert_eq!(fs::read(automation_store.runs_path(&automation_id)).unwrap(), history_before);
    assert_eq!(reopened.list_cron_session_runs(&record.id).unwrap(), cron_before);
    reopened.end_agent_delete(&record.id).unwrap();
    assert_eq!(sessions.find_automation_for_backend_delivery(&record.id, &backend_id)
        .unwrap().expect("explicitly cleared admission").automation.id, automation_id);
    assert!(sessions.find_automation_for_backend_delivery(&record.id, &missing_backend)
        .unwrap().is_none());
    assert_eq!(fs::read(automation_store.runs_path(&automation_id)).unwrap(), history_before);
    reopened.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn checked_backend_target_lookup_preserves_live_session_links_and_refuses_dangling_paths() {
    use std::os::unix::fs::symlink;

    let root = temp_root("backend-target-session-links");
    fs::create_dir(&root).expect("exclusive fixture root");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&production));
    let record = sessions.create_session(None, "user", None).unwrap();
    let created = sessions.create_agent_automation(&record.id, &AutomationSpec {
        name: "Linked target".into(),
        prompt: "Inspect linked work".into(),
        trigger: serde_json::json!({"type":"cron","schedule":"@daily"}),
        is_enabled: Some(true),
    }).unwrap();
    let backend_id = stable_automation_id(&record.id, &created[0].id);
    let config_before = fs::read(&created[0].file_path).unwrap();
    production.shutdown();
    drop(sessions);
    drop(production);

    let stored_agent = root.join("stored-agent");
    fs::rename(agents.join(&record.id), &stored_agent).unwrap();
    symlink(&stored_agent, agents.join(&record.id)).unwrap();
    let linked_root = root.join("linked-agents");
    symlink(&agents, &linked_root).unwrap();
    let reopened = Arc::new(ProductionSessionWorkers::with_agents_root(&linked_root, 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&reopened));
    let found = sessions.find_automation_for_backend_delivery(&record.id, &backend_id)
        .unwrap().expect("live root and agent links preserve Session behavior");
    assert_eq!(found.automation.id, created[0].id);
    assert_eq!(fs::read(&found.automation.file_path).unwrap(), config_before);
    assert!(sessions.find_automation_for_backend_delivery("absent-agent", &backend_id)
        .unwrap().is_none());

    let missing_target = root.join("absent-link-target");
    symlink(&missing_target, agents.join("dangling-agent")).unwrap();
    assert!(sessions.find_automation_for_backend_delivery("dangling-agent", &backend_id).is_err());
    assert_eq!(fs::read_link(agents.join("dangling-agent")).unwrap(), missing_target);
    for agents_root in [root.join("dangling-root"), root.join("dangling-parent/agents")] {
        let link = if agents_root.ends_with("agents") {
            agents_root.parent().unwrap().to_path_buf()
        } else {
            agents_root.clone()
        };
        symlink(&missing_target, &link).unwrap();
        let broken = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
        assert!(broken.find_automation_for_backend_delivery("absent-agent", &backend_id).is_err(),
            "dangling path must not become absence: {}", agents_root.display());
        assert_eq!(fs::read_link(&link).unwrap(), missing_target);
        assert!(!missing_target.exists());
        broken.shutdown();
    }
    reopened.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pending_media_admission_blocks_archive_until_same_generation_release() {
    let root = temp_root("work-admission-media");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let session_key = format!("agent:{}:main", record.id);
    let missing_path = production
        .capture_session_media_admission(
            &record.id,
            &session_key,
            "send-media:missing-path",
            None,
        )
        .expect_err("new media custody requires a durable materialized path");
    assert!(missing_path.contains("requires a materialized path"));
    assert!(production
        .read_session_work_admission(&record.id)
        .expect("read admission after missing path")
        .expect("session admission after missing path")
        .admitted_media_work_id
        .is_none());
    let media_path = root.join("agents").join("attachments").join("outgoing.bin");
    let revision = production
        .capture_session_media_admission(
            &record.id,
            &session_key,
            "send-media:tool-1",
            Some(&media_path),
        )
        .expect("capture media generation");
    let admitted = production
        .read_session_work_admission(&record.id)
        .expect("read admission")
        .expect("main admission");
    assert_eq!(
        admitted.admitted_media_work_id.as_deref(),
        Some("send-media:tool-1")
    );
    assert_eq!(
        admitted.admitted_media_paths,
        vec![media_path.to_string_lossy().to_string()]
    );

    let archive_error = production
        .transition_session_work_archive(
            &record.id,
            &session_key,
            &record.id,
            Some(&revision),
            true,
            Some("gateway-operator"),
            Some("manual review"),
            42.0,
        )
        .expect_err("pending media must block archive");
    assert!(archive_error.contains("pending outgoing media"));

    assert!(production
        .release_session_media_admission(&record.id, &revision, "send-media:tool-1")
        .expect("release media admission"));
    assert!(!production
        .release_session_media_admission(&record.id, &revision, "send-media:tool-1")
        .expect("duplicate release"));
    production
        .transition_session_work_archive(
            &record.id,
            &session_key,
            &record.id,
            Some(&revision),
            true,
            Some("gateway-operator"),
            Some("manual review"),
            43.0,
        )
        .expect("archive after media settlement");

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reopened_session_owner_rolls_back_unsettled_materialized_media() {
    let root = temp_root("work-admission-media-recovery");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let session_key = format!("agent:{}:main", record.id);
    let media_path = agents.join(&record.id).join("attachments").join("outgoing.bin");
    fs::create_dir_all(media_path.parent().expect("media parent")).expect("media dir");
    fs::write(&media_path, b"pending media").expect("media file");
    let revision = production
        .capture_session_media_admission(
            &record.id,
            &session_key,
            "send-media:tool-recovery",
            Some(&media_path),
        )
        .expect("capture media generation");
    assert!(!revision.is_empty());
    assert!(production.retire_agent_owners(&record.id, false)
        .expect("retire paired session owners before recovery"));
    assert_eq!(production.active_agent_store_owner_count(), 0);
    assert_eq!(production.active_agent_db_owner_count(), 0);

    production
        .open_agent_db_owner(&record.id)
        .expect("reopen session owner");
    assert!(!media_path.exists());
    let admission = production
        .read_session_work_admission(&record.id)
        .expect("read recovered admission")
        .expect("recovered admission");
    assert!(admission.admitted_media_work_id.is_none());
    assert!(admission.admitted_media_paths.is_empty());

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn background_work_generation_recheck_rejects_stale_terminal_result() {
    let root = temp_root("work-admission-generation");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");

    let expected_revision = production
        .capture_session_work_admission(
            &record.id,
            &format!("agent:{}:main", record.id),
            "background-work-1",
        )
        .expect("capture active generation");
    let current = production
        .read_session_work_admission(&record.id)
        .expect("read admission")
        .expect("main admission");
    let mut rebound = current.clone();
    rebound.lifecycle_revision = "rebound-generation".into();
    assert!(production
        .compare_and_set_session_work_admission(
            &record.id,
            Some(&current.lifecycle_revision),
            &rebound,
        )
        .expect("rebind generation"));

    let error = production
        .assert_session_work_admission_generation(
            &record.id,
            &current.session_key,
            &expected_revision,
        )
        .expect_err("stale terminal result must be rejected");
    assert!(error.contains("generation changed"));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn background_result_receipt_is_hidden_from_production_transcript_projection() {
    let root = temp_root("work-admission-receipt-projection");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    let session_key = format!("agent:{}:main", record.id);
    let lifecycle_revision = production
        .capture_session_work_admission(&record.id, &session_key, "background-work-1")
        .expect("capture generation");
    let result = production
        .commit_session_work_result(
            &record.id,
            &session_key,
            &lifecycle_revision,
            "background-work-1",
            "background-idem-1",
            "background output",
            1,
        )
        .expect("commit receipt");
    assert!(matches!(
        result,
        mahayana_host_runtime::extensions::session::agent_db::SessionWorkResultCommitOutcome::Committed(_)
    ));
    let visible = production
        .read_agent_transcript_entries(&record.id)
        .expect("read visible transcript");
    assert!(visible.iter().all(|entry| {
        !matches!(
            entry.get("kind").and_then(serde_json::Value::as_str),
            Some("background-result-receipt" | "outgoing-media-settlement")
        )
    }));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_facade_exposes_frozen_mint_profile_summary_transcript_and_stat_surfaces() {
    let root = temp_root("frozen-facade");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));

    let minted = store
        .mint_agent(|agent_id| Ok(agent_id.to_string()))
        .expect("mint agent id");
    assert!(!minted.is_empty());
    assert!(!agents.join(&minted).exists());

    let record = store
        .create_session(None, "user", None)
        .expect("materialized agent");
    store
        .write_agent_profile_file(
            &record.id,
            &AgentProfileUpdate {
                name: "  Facade Agent  ".into(),
                description: "  profile description  ".into(),
                title: Some("  Builder  ".into()),
                avatar_shape: Some("  rounded  ".into()),
                avatar_color: Some("  violet  ".into()),
            },
        )
        .expect("profile write");

    production
        .append_agent_transcript_entries(
            &record.id,
            &[serde_json::json!({
                "id":"facade-entry",
                "kind":"message",
                "role":"user",
                "content":"hello facade",
                "timestampMs":10
            })],
        )
        .expect("transcript append");

    let open = store
        .open_session(&record.id)
        .expect("open session")
        .expect("prepared session");
    assert_eq!(
        store
            .get_transcript_entries(&open)
            .expect("session transcript")
            .len(),
        1
    );

    let open_summary = store
        .summarize_open_session(&open)
        .expect("open summary")
        .expect("summary");
    assert_eq!(open_summary.name, "Facade Agent");
    assert!(open_summary.is_active);
    assert_eq!(open_summary.title, "Builder");

    let inactive_summary = store
        .summarize_session(&open, None)
        .expect("session summary")
        .expect("summary");
    assert!(!inactive_summary.is_active);
    assert_eq!(inactive_summary.description, "profile description");

    let stat = store.stat_open_db(&open).expect("db stat");
    assert!(stat.size > 0);
    assert!(stat.mtime_ms >= 0.0);

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_facade_delegates_interaction_state_memory_capacity_and_time_zone() {
    let root = temp_root("interaction-state");
    let agents = root.join("agents");
    let production = Arc::new(
        ProductionSessionWorkers::with_agents_root_and_user_time_zone_resolver(
            &agents,
            500,
            Arc::new(|| Some("America/Los_Angeles".to_string())),
        ),
    );
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store.create_session(None, "user", None).expect("agent");

    assert_eq!(store.list_agent_ids().expect("agent ids"), vec![record.id.clone()]);
    assert!(!store.is_agent_cap_reached().expect("cap"));
    assert_eq!(store.get_user_time_zone().as_deref(), Some("America/Los_Angeles"));

    let memory = store.create_memory_store(agents.join(&record.id));
    assert_eq!(memory.get_location(), agents.join(&record.id).join("memory"));
    assert!(!store.agent_has_content(agents.join(&record.id)));
    memory
        .add_memory("Remember this", 1, MemoryKind::Profile)
        .expect("memory write")
        .expect("memory record");
    assert!(store.agent_has_content(agents.join(&record.id)));

    let prepared = store
        .open_session(&record.id)
        .expect("open")
        .expect("prepared");
    assert!(store.get_session_outline(&prepared).expect("session outline").is_empty());
    assert!(store.get_agent_outline(&record.id).expect("agent outline").is_empty());
    assert_eq!(
        store
            .get_agent_transcript_entries(&record.id)
            .expect("transcript alias"),
        store
            .read_agent_transcript_entries(&record.id)
            .expect("transcript owner")
    );
    assert!(store
        .mark_session_activity(&prepared, 10.0)
        .expect("session activity"));
    assert!(store
        .mark_session_viewed(&prepared, 20.0, false)
        .expect("session viewed"));
    assert!(store
        .mark_session_viewed_now(&prepared, 25.0, false)
        .expect("session viewed now"));

    let awaiting = AwaitingUserResponse {
        tab_id: "tab-a".into(),
        reason: "approval".into(),
        since: 30.0,
    };
    assert!(store
        .set_awaiting_user_response(&record.id, Some(&awaiting))
        .expect("set awaiting"));
    assert!(!store
        .set_awaiting_user_response_for_tab(
            &record.id,
            "tab-b",
            None,
            Some(40.0),
        )
        .expect("wrong tab"));
    assert!(store
        .set_awaiting_user_response_for_tab(
            &record.id,
            "tab-a",
            None,
            Some(40.0),
        )
        .expect("matching tab"));

    production
        .set_agent_memory_prompt_snapshot(
            &record.id,
            &serde_json::json!({"render":"snapshot","compactionEpoch":1}),
        )
        .expect("set memory snapshot");
    assert!(store
        .clear_agent_memory_prompt_snapshot(&record.id)
        .expect("clear memory snapshot"));

    production
        .append_agent_transcript_entries(
            &record.id,
            &[
                serde_json::json!({
                    "id":"approval",
                    "kind":"send-message",
                    "timestampMs":50,
                    "message":{
                        "type":"auto-review-approval",
                        "approval":{"requestId":"req-approval","status":"pending"}
                    }
                }),
                serde_json::json!({
                    "id":"permission",
                    "kind":"send-message",
                    "timestampMs":60,
                    "message":{
                        "type":"local-tool-permission",
                        "ask":{"requestId":"req-permission","status":"pending"}
                    }
                }),
            ],
        )
        .expect("pending transcript");
    assert_eq!(
        store
            .expire_pending_auto_review_approvals(
                &record.id,
                Some("req-approval"),
            )
            .expect("expire approval"),
        vec!["req-approval".to_string()]
    );
    assert_eq!(
        store
            .expire_pending_local_tool_permission_asks(
                &record.id,
                Some("req-permission"),
                None,
            )
            .expect("expire permission"),
        vec!["req-permission".to_string()]
    );

    store
        .ensure_conversation_capacity_for_turn(&prepared)
        .expect("default capacity policy");

    assert!(store
        .store_connector_credential(&record.id, "slack", "token", "secret")
        .expect("credential"));
    let configs = store.list_channel_configs(&record.id).expect("channel configs");
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].platform, "slack");
    assert_eq!(configs[0].token, "secret");

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn archived_workflow_management_remains_available_under_the_deletion_gate() {
    fn files_under(path: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
        let mut files = std::collections::BTreeMap::new();
        if path.is_dir() {
            files.insert(path.to_path_buf(), None);
            for entry in fs::read_dir(path).unwrap() {
                files.extend(files_under(&entry.unwrap().path()));
            }
        } else {
            files.insert(path.to_path_buf(), Some(fs::read(path).unwrap()));
        }
        files
    }

    let root = temp_root("workflow-archive-fence");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let agent = store.create_session(None, "user", None).unwrap();
    let unaffected = store.create_session(None, "user", None).unwrap();
    let original = WorkflowSpec {
        name: "Archived workflow".into(), description: "Original description".into(),
        body: "ORIGINAL_WORKFLOW_BODY".into(), trigger: None, source_ref: None,
    };
    let workflow = store.create_agent_workflow(&agent.id, &original).unwrap()
        .into_iter().find(|row| row.name == original.name).unwrap();
    let disabled_spec = WorkflowSpec { name: "Disabled workflow".into(), ..original.clone() };
    let disabled = store.create_agent_workflow(&agent.id, &disabled_spec).unwrap()
        .into_iter().find(|row| row.name == disabled_spec.name).unwrap();
    store.set_agent_workflow_enabled(&agent.id, &disabled.id, false).unwrap();
    // Unbound management retains its existing migration and shared-definition behavior.
    let legacy = root.join("agents").join(&agent.id).join("workflows/archived-legacy");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("workflow.md"), "---\nname: Archived legacy\n---\nMANAGEMENT_LEGACY_BODY\n").unwrap();
    let home = root.join("home");
    let cwd = root.join("cwd");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&cwd).unwrap();
    fs::write(home.join("CLAUDE.md"), "# Local skill\nLOCAL_MANAGEMENT_BODY\n").unwrap();
    let admission = production.read_session_work_admission(&agent.id).unwrap().unwrap();
    let archived = production.transition_session_work_archive(
        &agent.id, &admission.session_key, &agent.id, Some(&admission.lifecycle_revision),
        true, Some("contract"), Some("workflow-management-control"), 1_000.0,
    ).unwrap();
    let replacement = WorkflowSpec {
        name: "Archived management".into(), body: "ARCHIVED_MANAGEMENT_BODY".into(),
        ..original.clone()
    };
    let created = store.create_agent_workflow(&agent.id, &replacement).unwrap()
        .into_iter().find(|row| row.name == replacement.name).unwrap();
    assert!(fs::read_to_string(&created.file_path).unwrap().contains("ARCHIVED_MANAGEMENT_BODY"));
    assert!(!legacy.exists());
    assert!(fs::read_to_string(root.join("workflows/archived-legacy/SKILL.md")).unwrap()
        .contains("MANAGEMENT_LEGACY_BODY"));
    store.update_agent_workflow(&agent.id, &workflow.id, &replacement).unwrap();
    assert!(fs::read_to_string(&workflow.file_path).unwrap().contains("ARCHIVED_MANAGEMENT_BODY"));
    assert_eq!(store.get_agent_workflow(&unaffected.id, &workflow.id).unwrap().unwrap().body,
        "ARCHIVED_MANAGEMENT_BODY", "the managed definition is shared across sessions");
    let disabled_records = store.set_agent_workflow_enabled(&agent.id, &workflow.id, false).unwrap();
    assert!(!disabled_records.iter().find(|row| row.id == workflow.id).unwrap().is_enabled_for_agent);
    let enabled_records = store.set_agent_workflow_enabled(&agent.id, &disabled.id, true).unwrap();
    assert!(enabled_records.iter().find(|row| row.id == disabled.id).unwrap().is_enabled_for_agent);
    let markdown = store.import_agent_workflow_markdown(&agent.id,
        "---\nname: Archived import\n---\nMARKDOWN_MANAGEMENT_BODY\n", None).unwrap();
    assert_eq!(markdown.result.imported.len(), 1);
    let imported = markdown.workflows.iter()
        .find(|row| row.id == markdown.result.imported[0].id).unwrap();
    assert!(fs::read_to_string(&imported.file_path).unwrap().contains("MARKDOWN_MANAGEMENT_BODY"));
    let source = store.import_agent_workflow_source(&agent.id,
        "https://example.test/SKILL.md", Some("Archived source")).unwrap();
    assert_eq!(source.result.imported.len(), 1);
    assert!(source.workflows.iter()
        .any(|row| row.source_ref.as_deref() == Some("https://example.test/SKILL.md")));
    let ported = store.port_agent_local_skills_from(&agent.id, &home, &cwd).unwrap();
    assert_eq!(ported.result.imported.len(), 1);
    assert!(ported.result.skipped.is_empty());
    store.remove_agent_workflow(&agent.id, &created.id).unwrap();
    assert!(!created.file_path.exists());
    assert_eq!(production.read_session_work_admission(&agent.id).unwrap(), Some(archived.clone()));

    let unaffected_spec = WorkflowSpec { name: "Unaffected workflow".into(), ..original.clone() };
    let unaffected_workflow = store.create_agent_workflow(&unaffected.id, &unaffected_spec).unwrap()
        .into_iter().find(|row| row.name == unaffected_spec.name).unwrap();
    assert!(fs::read_to_string(&unaffected_workflow.file_path).unwrap().contains("ORIGINAL_WORKFLOW_BODY"));
    let restored = production.transition_session_work_archive(
        &agent.id, &admission.session_key, &agent.id, Some(&archived.lifecycle_revision),
        false, None, None, 2_000.0,
    ).unwrap();
    assert_ne!(restored.lifecycle_revision, admission.lifecycle_revision);
    let workflow_store = store.workflow_store_for(&agent.id).unwrap();
    let definitions_before = files_under(workflow_store.get_location());
    let enablement_path = workflow_store.enablement.path();
    let enablement_before = fs::read(&enablement_path).unwrap();
    let records_before = workflow_store.list_all();
    production.begin_agent_delete(&agent.id).unwrap();
    production.clear_agent_delete_fast_path(&agent.id);
    let assert_refused = |result: Result<(), String>| {
        let error = result.expect_err("durable deletion must still refuse workflow management");
        assert!(error.contains("deletion cleanup"), "{error}");
        assert_eq!(files_under(workflow_store.get_location()), definitions_before);
        assert_eq!(fs::read(&enablement_path).unwrap(), enablement_before);
        assert_eq!(workflow_store.list_all(), records_before);
    };
    assert_refused(store.create_agent_workflow(&agent.id, &original).map(|_| ()));
    assert_refused(store.update_agent_workflow(&agent.id, &workflow.id, &original).map(|_| ()));
    assert_refused(store.remove_agent_workflow(&agent.id, &workflow.id).map(|_| ()));
    assert_refused(store.set_agent_workflow_enabled(&agent.id, &workflow.id, true).map(|_| ()));
    assert_refused(store.import_agent_workflow_markdown(&agent.id,
        "---\nname: Deleted import\n---\nDELETION_MUST_NOT_WRITE\n", None).map(|_| ()));
    assert_refused(store.import_agent_workflow_source(&agent.id,
        "https://example.test/deleted.md", Some("Deleted source")).map(|_| ()));
    assert_refused(store.port_agent_local_skills_from(&agent.id, &home, &cwd).map(|_| ()));
    assert_refused(store.port_agent_local_skills(&agent.id).map(|_| ()));
    production.end_agent_delete(&agent.id).unwrap();
    store.remove_agent_workflow(&agent.id, &workflow.id).unwrap();
    assert!(!workflow.file_path.exists());
    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_facade_delegates_automation_workflow_and_channel_stores() {
    let root = temp_root("stores");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store.create_session(None, "user", None).expect("agent");

    let automations = store
        .create_agent_automation(
            &record.id,
            &AutomationSpec {
                name: "Daily review".into(),
                prompt: "Review the inbox".into(),
                trigger: serde_json::json!({"type":"cron","schedule":"0 9 * * *"}),
                is_enabled: Some(true),
            },
        )
        .expect("create automation");
    assert_eq!(automations.len(), 1);
    let automation_id = automations[0].id.clone();
    assert!(automations[0].is_enabled);

    let automations = store
        .set_agent_automation_enabled(&record.id, &automation_id, false)
        .expect("disable automation");
    assert!(!automations[0].is_enabled);
    let automations = store
        .update_agent_automation(
            &record.id,
            &automation_id,
            &AutomationSpec {
                name: "Daily review updated".into(),
                prompt: "Review everything".into(),
                trigger: serde_json::json!({"type":"cron","schedule":"30 9 * * *"}),
                is_enabled: Some(true),
            },
        )
        .expect("update automation");
    assert_eq!(automations[0].name, "Daily review updated");
    assert!(automations[0].is_enabled);
    assert_eq!(
        store.list_agent_automations(&record.id).expect("list automations").len(),
        1
    );
    assert!(store
        .remove_agent_automation(&record.id, &automation_id)
        .expect("remove automation")
        .is_empty());

    let workflows = store
        .create_agent_workflow(
            &record.id,
            &WorkflowSpec {
                name: "Research".into(),
                description: "Research workflow".into(),
                body: "Read sources and summarize.".into(),
                trigger: None,
                source_ref: None,
            },
        )
        .expect("create workflow");
    assert_eq!(workflows.len(), 1);
    let workflow_id = workflows[0].id.clone();
    assert_eq!(
        store
            .get_agent_workflow(&record.id, &workflow_id)
            .expect("get workflow")
            .expect("workflow")
            .name,
        "Research"
    );
    let workflows = store
        .update_agent_workflow(
            &record.id,
            &workflow_id,
            &WorkflowSpec {
                name: "Research updated".into(),
                description: "Updated".into(),
                body: "Read more sources and summarize.".into(),
                trigger: None,
                source_ref: None,
            },
        )
        .expect("update workflow");
    assert_eq!(workflows[0].name, "Research updated");
    let workflows = store
        .set_agent_workflow_enabled(&record.id, &workflow_id, false)
        .expect("disable workflow");
    assert!(!workflows[0].is_enabled_for_agent);
    assert_eq!(
        store.list_agent_workflows(&record.id).expect("list workflows").len(),
        1
    );
    assert!(store
        .remove_agent_workflow(&record.id, &workflow_id)
        .expect("remove workflow")
        .is_empty());

    let channel_store = store.open_channel_store(&record.id).expect("channel store");
    assert!(channel_store.get_location().ends_with("channels"));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_facade_delegates_transcript_and_channel_owners_without_duplication() {
    let root = temp_root("delegation");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");

    production
        .append_agent_transcript_entries(
            &record.id,
            &[
                serde_json::json!({
                    "id":"entry-1",
                    "kind":"message",
                    "role":"user",
                    "content":"hello",
                    "timestampMs":10
                }),
                serde_json::json!({
                    "id":"entry-2",
                    "kind":"tool-call",
                    "name":"search",
                    "timestampMs":20
                }),
                serde_json::json!({
                    "id":"entry-3",
                    "kind":"message",
                    "role":"assistant",
                    "content":"reply",
                    "timestampMs":30
                }),
                serde_json::json!({
                    "id":"entry-4",
                    "kind":"message",
                    "role":"assistant",
                    "content":"branch reply",
                    "replyTo":"entry-1",
                    "branched":true,
                    "timestampMs":40
                })
            ],
        )
        .expect("transcript");
    assert_eq!(
        store
            .read_agent_transcript_entries(&record.id)
            .expect("read transcript")
            .len(),
        4
    );

    let page = store
        .read_agent_transcript_page(
            &record.id,
            TranscriptPageQuery {
                before_seq: None,
                since_ms: Some(0),
                until_ms: 100,
                limit: 10,
            },
        )
        .expect("page");
    assert_eq!(
        page.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1"]
    );

    let window = store
        .read_agent_transcript_window(
            &record.id,
            TranscriptWindowQuery {
                before_seq: None,
                limit: 10,
            },
        )
        .expect("window");
    assert_eq!(
        window
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "entry-3"]
    );

    let tail = store
        .read_agent_transcript_tail(
            &record.id,
            TranscriptWindowQuery {
                before_seq: None,
                limit: 10,
            },
        )
        .expect("tail");
    assert_eq!(
        tail.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "entry-2", "entry-3", "entry-4"]
    );

    let thread = store
        .read_agent_thread(&record.id, "entry-1")
        .expect("thread");
    assert_eq!(
        thread
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "entry-4"]
    );

    assert!(store
        .store_connector_credential(&record.id, "slack", "token", "secret")
        .expect("connector"));
    assert_eq!(
        store.list_agent_channels(&record.id).expect("channels").len(),
        1
    );
    assert!(store.disconnect_channel(&record.id, "slack").expect("disconnect"));

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn session_facade_owns_workflow_import_local_skill_port_and_cross_agent_automation_projection() {
    let root = temp_root("facade-imports");
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let first = store.create_session(None, "user", None).expect("first agent");
    let second = store.create_session(None, "user", None).expect("second agent");

    let markdown = store
        .import_agent_workflow_markdown(
            &first.id,
            "---\nname: Imported markdown\n---\nDo the imported thing.\n",
            None,
        )
        .expect("markdown import");
    assert_eq!(markdown.result.imported.len(), 1);
    assert!(markdown.result.skipped.is_empty());

    let live = store
        .import_agent_workflow_source(
            &first.id,
            "https://example.test/SKILL.md",
            Some("Live skill"),
        )
        .expect("live source import");
    assert_eq!(live.result.imported.len(), 1);
    assert!(live.result.skipped.is_empty());
    assert!(live
        .workflows
        .iter()
        .any(|workflow| workflow.source_ref.as_deref() == Some("https://example.test/SKILL.md")));

    let home = root.join("home");
    let cwd = root.join("cwd");
    fs::create_dir_all(&home).expect("home");
    fs::create_dir_all(cwd.join(".cursor/rules")).expect("rules dir");
    fs::write(home.join("CLAUDE.md"), "# Claude memory\n").expect("home skill");
    fs::write(cwd.join(".cursor/rules/review.md"), "# Review\n").expect("rule");
    let local = store
        .port_agent_local_skills_from(&first.id, &home, &cwd)
        .expect("local skill port");
    assert_eq!(local.result.imported.len(), 2);
    assert!(local.result.skipped.is_empty());

    for (agent_id, name) in [
        (&first.id, "First daily"),
        (&second.id, "Second daily"),
    ] {
        store
            .create_agent_automation(
                agent_id,
                &AutomationSpec {
                    name: name.into(),
                    prompt: "Do work".into(),
                    trigger: serde_json::json!({"type":"cron","schedule":"0 9 * * *"}),
                    is_enabled: Some(true),
                },
            )
            .expect("automation");
    }
    let all = store.list_all_automations().expect("all automations");
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|row| row.agent_id == first.id));
    assert!(all.iter().any(|row| row.agent_id == second.id));
    assert_eq!(
        store
            .list_all_automation_definitions()
            .expect("all definitions")
            .len(),
        2
    );

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}
