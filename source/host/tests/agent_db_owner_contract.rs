use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db::{
    CompactionCommit, CompactionCommitOutcome, ProviderReviewContinuationOutcome, SandAgentDb,
    SandAgentDbOptions, SessionProviderReview, SessionWorkAdmissionRecord,
    SessionWorkResultCommitOutcome,
};
use mahayana_host_runtime::extensions::session::agent_db_recovery::DbRecoveryOptions;
use mahayana_host_runtime::extensions::session::agent_db_transcript_pages::{
    TranscriptPageQuery, TranscriptWindowQuery,
};
use mahayana_host_runtime::extensions::session::agent_db_serde::{
    AwaitingUserResponse, SandProfile,
};
use mahayana_host_runtime::storage::store_db::live_db_handle_count;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-agent-db-owner-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn owner_registers_live_handle_notifies_shipping_mutations_and_releases_on_close() {
    let root = temp_root("lifecycle");
    let agent_dir = root.join("agent-owner");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");

    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    assert_eq!(live_db_handle_count(&db_path), 1);

    let metadata_hits = Arc::new(AtomicUsize::new(0));
    let profile_hits = Arc::new(AtomicUsize::new(0));
    let awaiting_hits = Arc::new(AtomicUsize::new(0));

    let _metadata_sub = owner.subscribe_metadata(
        "latestRootBlobId",
        {
            let hits = Arc::clone(&metadata_hits);
            Arc::new(move || {
                hits.fetch_add(1, Ordering::SeqCst);
            })
        },
    );
    let _profile_sub = owner.subscribe_sand_profile({
        let hits = Arc::clone(&profile_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });
    let _awaiting_sub = owner.subscribe_awaiting_user_response({
        let hits = Arc::clone(&awaiting_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });

    let initial_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("initial revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("initial revision value");

    assert!(owner
        .compare_and_set_latest_root_blob_id(&[], &[0xaa, 0xbb])
        .expect("root cas"));
    let root_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("root revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("root revision value");
    assert_ne!(root_revision, initial_revision);
    assert!(owner
        .set_sand_profile(&SandProfile {
            description: "profile".into(),
            avatar_path: Some("/tmp/avatar.png".into()),
        })
        .expect("profile"));
    assert!(owner
        .set_awaiting_user_response(Some(&AwaitingUserResponse {
            tab_id: "tab-a".into(),
            reason: "approval".into(),
            since: 1.0,
        }))
        .expect("awaiting"));

    assert_eq!(metadata_hits.load(Ordering::SeqCst), 1);
    assert_eq!(profile_hits.load(Ordering::SeqCst), 1);
    assert_eq!(awaiting_hits.load(Ordering::SeqCst), 1);

    assert!(owner.clear_conversation().expect("clear conversation"));
    let cleared_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("cleared revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("cleared revision value");
    assert_ne!(cleared_revision, root_revision);
    assert!(owner
        .get_awaiting_user_response()
        .expect("awaiting after clear")
        .is_none());
    assert_eq!(metadata_hits.load(Ordering::SeqCst), 2);
    assert_eq!(profile_hits.load(Ordering::SeqCst), 1);
    assert_eq!(awaiting_hits.load(Ordering::SeqCst), 2);

    owner.close(true);
    assert!(owner.is_closed());
    assert_eq!(live_db_handle_count(&db_path), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owner_drops_busy_writes_and_reports_the_operation() {
    let root = temp_root("busy");
    let agent_dir = root.join("agent-busy");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let callback_observed = Arc::clone(&observed);
    let owner = SandAgentDb::open_with_options(
        &db_path,
        SandAgentDbOptions {
            recovery: DbRecoveryOptions {
                busy_timeout_ms: 20,
                ..DbRecoveryOptions::default()
            },
            on_busy_error: Some(Arc::new(move |operation, error| {
                callback_observed
                    .lock()
                    .expect("busy events")
                    .push(format!("{operation}:{error}"));
            })),
        },
    )
    .expect("owner");

    let locker = rusqlite::Connection::open(&db_path).expect("locker");
    locker.execute_batch("BEGIN IMMEDIATE").expect("write lock");
    assert!(!owner.write_kv("busy-test", "value").expect("busy write"));
    locker.execute_batch("ROLLBACK").expect("unlock");
    drop(locker);

    let observed = observed.lock().expect("busy observations");
    assert_eq!(observed.len(), 1);
    assert!(observed[0].starts_with("writeKv:busy-test:"));
    drop(observed);

    owner.close(false);
    assert_eq!(live_db_handle_count(&db_path), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_work_admission_record_migrates_and_compare_and_sets_generation() {
    let root = temp_root("work-admission-record");
    let agent_dir = root.join("agent-work");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let first = SessionWorkAdmissionRecord {
        session_key: "agent:agent-work:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: Some("work-1".into()),
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &first)
        .expect("create admission record"));
    assert!(!owner
        .compare_and_set_session_work_admission(None, &first)
        .expect("create-only conflict"));
    owner
        .append_transcript_entries(&[serde_json::json!({
            "id": "permission-1",
            "kind": "send-message",
            "message": {"type": "local-tool-permission", "ask": {"status": "pending"}},
        })])
        .expect("seed permission entry");
    let updated = owner
        .update_transcript_entry_if_session_admitted(
            &first.session_key,
            "permission-1",
            &serde_json::json!({
                "id": "permission-1",
                "kind": "send-message",
                "message": {"type": "local-tool-permission", "ask": {"status": "allowed"}},
            }),
        )
        .expect("session-admitted update")
        .expect("updated permission entry");
    assert_eq!(updated["message"]["ask"]["status"], "allowed");

    let rebound = SessionWorkAdmissionRecord {
        session_key: "agent:other-agent:main".into(),
        ..first.clone()
    };
    assert!(!owner
        .compare_and_set_session_work_admission(Some("revision-1"), &rebound)
        .expect("identity-rebinding CAS is rejected"));
    assert_eq!(
        owner
            .read_session_work_admission()
            .expect("read admission after rejected rebind")
            .expect("admission remains present")
            .session_key,
        first.session_key
    );

    let mut archived = first.clone();
    archived.lifecycle_revision = "revision-2".into();
    archived.archived_at_ms = Some(42.0);
    archived.archived_by = Some("human-1".into());
    archived.archive_reason = Some("manual review".into());
    assert!(owner
        .compare_and_set_session_work_admission(Some("revision-1"), &archived)
        .expect("guarded archive transition"));
    assert!(!owner
        .compare_and_set_session_work_admission(Some("revision-1"), &first)
        .expect("stale transition"));
    let archived_update = owner
        .update_transcript_entry_if_session_admitted(
            &first.session_key,
            "permission-1",
            &serde_json::json!({
                "id": "permission-1",
                "kind": "send-message",
                "message": {"type": "local-tool-permission", "ask": {"status": "denied"}},
            }),
        )
        .expect_err("archived session update must reject");
    assert!(archived_update
        .to_string()
        .contains("session is not admitted for transcript update"));
    assert_eq!(
        owner
            .read_session_work_admission()
            .expect("read admission record")
            .expect("record")
            .lifecycle_revision,
        "revision-2"
    );
    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn provider_review_owner_is_generation_bound_idempotent_and_conflict_safe() {
    let root = temp_root("provider-review-owner");
    let agent_dir = root.join("agent-review");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let admission = SessionWorkAdmissionRecord {
        session_key: "agent:agent-review:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &admission)
        .expect("create admission"));
    let review = SessionProviderReview {
        id: "review-1".into(),
        session_id: "session-1".into(),
        run_id: "run-1".into(),
        provider: "openai".into(),
        model: "model-1".into(),
        runtime_id: "openclaw".into(),
        api: Some("openai-chatgpt-responses".into()),
        native_thread_id: None,
        native_turn_id: None,
        explanation: "provider-authored refusal".into(),
        continuation_message: Some("Review and continue".into()),
        error_type: Some("policy".into()),
    };
    assert_eq!(owner
        .record_session_provider_review("revision-1", &review)
        .expect("record review"), review);
    let repeated = SessionProviderReview { id: "review-2".into(), ..review.clone() };
    assert_eq!(owner
        .record_session_provider_review("revision-1", &repeated)
        .expect("idempotent review"), review);
    let conflicting = SessionProviderReview { explanation: "different".into(), ..review.clone() };
    assert!(owner
        .record_session_provider_review("revision-1", &conflicting)
        .is_err());
    let mut accepted = admission.clone();
    accepted.provider_review_id = Some(review.id.clone());
    accepted.provider_review_phase = Some("accepted".into());
    accepted.provider_review = Some(review.clone());
    assert!(owner
        .compare_and_set_session_work_admission(Some("revision-1"), &accepted)
        .expect("mark review accepted"));
    let continuation_refusal = SessionProviderReview {
        id: "review-3".into(),
        run_id: "run-2".into(),
        explanation: "provider-authored follow-up refusal".into(),
        ..review.clone()
    };
    assert_eq!(
        owner
            .record_session_provider_review("revision-1", &continuation_refusal)
            .expect("replace after accepted continuation"),
        continuation_refusal
    );
    assert!(owner
        .record_session_provider_review("stale", &review)
        .is_err());
    assert_eq!(owner
        .read_session_provider_review()
        .expect("read review")
        .expect("review exists")
        .id, "review-3");
    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn provider_review_continuation_settlement_is_run_and_generation_bound() {
    let root = temp_root("provider-review-settlement");
    let agent_dir = root.join("agent-review-settlement");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let owner = SandAgentDb::open(agent_dir.join("store.db"), 50).expect("owner");
    let review = SessionProviderReview {
        id: "review-settle".into(),
        session_id: "session-settle".into(),
        run_id: "refusal-run".into(),
        provider: "openai".into(),
        model: "model-1".into(),
        runtime_id: "openclaw".into(),
        api: Some("openai-chatgpt-responses".into()),
        native_thread_id: None,
        native_turn_id: None,
        explanation: "provider-authored refusal".into(),
        continuation_message: Some("continue".into()),
        error_type: None,
    };
    let admission = SessionWorkAdmissionRecord {
        session_key: "agent:agent-review-settlement:main".into(),
        session_id: "session-settle".into(),
        lifecycle_revision: "revision-settle".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: Some(review.id.clone()),
        provider_review_phase: Some("accepted".into()),
        provider_review_continuation_run_id: Some("continuation-run".into()),
        provider_review: Some(review),
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &admission)
        .expect("create accepted review"));
    assert!(!owner
        .settle_provider_review_continuation(
            "stale-revision",
            "continuation-run",
            ProviderReviewContinuationOutcome::Completed,
        )
        .expect("stale settlement is explicit"));
    assert!(owner
        .settle_provider_review_continuation(
            "revision-settle",
            "continuation-run",
            ProviderReviewContinuationOutcome::Completed,
        )
        .expect("complete continuation"));
    let settled = owner
        .read_session_work_admission()
        .expect("read settled admission")
        .expect("settled admission exists");
    assert_eq!(settled.provider_review_phase.as_deref(), Some("completed"));
    assert!(settled.provider_review.is_none());
    assert!(!owner
        .settle_provider_review_continuation(
            "revision-settle",
            "continuation-run",
            ProviderReviewContinuationOutcome::Completed,
        )
        .expect("duplicate settlement is ignored"));
    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_archive_transition_is_guarded_idempotent_and_blocks_active_work() {
    let root = temp_root("work-admission-archive-transition");
    let agent_dir = root.join("agent-work");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let record = SessionWorkAdmissionRecord {
        session_key: "agent:agent-work:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &record)
        .expect("create admission record"));

    let archived = owner
        .transition_session_work_archive(
            "session-1",
            Some("revision-1"),
            true,
            Some("gateway-operator"),
            Some("manual review"),
            42.0,
        )
        .expect("archive transition");
    assert_eq!(archived.archived_at_ms, Some(42.0));
    assert_eq!(archived.archive_reason.as_deref(), Some("manual review"));
    assert_eq!(
        owner
            .transition_session_work_archive(
                "session-1",
                Some("revision-1"),
                true,
                Some("other-operator"),
                Some("different reason"),
                43.0,
            )
            .expect("idempotent archive")
            .archived_at_ms,
        Some(42.0)
    );
    let restored = owner
        .transition_session_work_archive(
            "session-1",
            Some("revision-1"),
            false,
            None,
            None,
            44.0,
        )
        .expect("restore transition");
    assert!(restored.archived_at_ms.is_none());

    let mut active = restored;
    active.admitted_work_id = Some("work-1".into());
    assert!(owner
        .compare_and_set_session_work_admission(Some("revision-1"), &active)
        .expect("admit active work"));
    let error = owner
        .transition_session_work_archive(
            "session-1",
            Some("revision-1"),
            true,
            Some("gateway-operator"),
            None,
            45.0,
        )
        .expect_err("active work must block archive");
    assert!(error.to_string().contains("admitted work"));

    active.admitted_work_id = None;
    active.admitted_media_work_id = Some("send-media:tool-1".into());
    assert!(owner
        .compare_and_set_session_work_admission(Some("revision-1"), &active)
        .expect("admit pending media"));
    let media_error = owner
        .transition_session_work_archive(
            "session-1",
            Some("revision-1"),
            true,
            Some("gateway-operator"),
            None,
            46.0,
        )
        .expect_err("pending media must block archive");
    assert!(media_error.to_string().contains("pending outgoing media"));
    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn admitted_transcript_append_is_rejected_after_archive_without_partial_write() {
    let root = temp_root("work-admission-transcript-append");
    let agent_dir = root.join("agent-work");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let session_key = "agent:agent-work:main";
    let record = SessionWorkAdmissionRecord {
        session_key: session_key.into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &record)
        .expect("create admission record"));

    let first_entry = serde_json::json!({
        "id": "entry-before-archive",
        "kind": "message",
        "role": "user",
        "content": "accepted"
    });
    assert_eq!(
        owner
            .append_transcript_entries_if_session_admitted(session_key, &[first_entry])
            .expect("append while admitted"),
        1
    );

    owner
        .transition_session_work_archive(
            "session-1",
            Some("revision-1"),
            true,
            Some("test"),
            Some("archive"),
            42.0,
        )
        .expect("archive transition");

    let rejected_entry = serde_json::json!({
        "id": "entry-after-archive",
        "kind": "message",
        "role": "user",
        "content": "must not be written"
    });
    let error = owner
        .append_transcript_entries_if_session_admitted(session_key, &[rejected_entry])
        .expect_err("archived session must reject transcript append");
    assert!(error
        .to_string()
        .contains("session is not admitted for transcript append"));
    let ids = owner
        .get_transcript_entries()
        .expect("read transcript")
        .into_iter()
        .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str).map(str::to_owned))
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["entry-before-archive"]);

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn media_settlement_appends_and_releases_in_one_owner_transaction() {
    let root = temp_root("media-settlement");
    let agent_dir = root.join("agent-media-settlement");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let admission = SessionWorkAdmissionRecord {
        session_key: "agent:agent-media-settlement:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: Some("send-media:tool-1".into()),
        admitted_media_paths: vec!["/tmp/send-media-tool-1".into()],
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &admission)
        .expect("create admission"));

    let stale_identity = owner
        .append_transcript_entries_and_release_media(
            &[serde_json::json!({
                "id": "send-stale-identity",
                "kind": "send-message",
                "content": "must not settle",
            })],
            "agent:agent-media-settlement:rebound",
            "revision-1",
            "send-media:tool-1",
        )
        .expect_err("rebound session identity must reject media settlement");
    assert!(stale_identity
        .to_string()
        .contains("media settlement generation or identity is stale"));

    assert_eq!(
        owner
            .append_transcript_entries_and_release_media(
                &[serde_json::json!({
                    "id": "send-1",
                    "kind": "send-message",
                    "content": "owned media"
                })],
                "agent:agent-media-settlement:main",
                "revision-1",
                "send-media:tool-1",
            )
            .expect("atomic media settlement"),
        1
    );
    assert_eq!(
        owner
            .read_session_work_admission()
            .expect("read admission")
            .expect("admission")
            .admitted_media_work_id,
        None
    );
    assert!(owner
        .read_session_work_admission()
        .expect("read admission paths")
        .expect("admission paths")
        .admitted_media_paths
        .is_empty());
    assert!(owner
        .get_transcript_entry_by_id("send-1")
        .expect("read transcript")
        .is_some());
    let receipt = owner
        .get_transcript_entry_by_id("outgoing-media-settlement:send-media:tool-1")
        .expect("read media settlement receipt")
        .expect("media settlement receipt");
    assert_eq!(
        receipt.get("kind").and_then(serde_json::Value::as_str),
        Some("outgoing-media-settlement")
    );

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn background_result_receipt_is_idempotent_and_conflicts_fail_closed() {
    let root = temp_root("work-admission-receipt");
    let agent_dir = root.join("agent-work");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let admission = SessionWorkAdmissionRecord {
        session_key: "agent:agent-work:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: Some("work-1".into()),
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &admission)
        .expect("create admission"));

    let committed = owner
        .commit_session_work_result(
            &admission.session_key,
            &admission.lifecycle_revision,
            "work-1",
            "idem-1",
            "background output",
            2,
        )
        .expect("commit receipt");
    assert!(matches!(committed, SessionWorkResultCommitOutcome::Committed(_)));

    let duplicate = owner
        .commit_session_work_result(
            &admission.session_key,
            &admission.lifecycle_revision,
            "work-1",
            "idem-1",
            "background output",
            2,
        )
        .expect("duplicate receipt");
    assert!(matches!(
        duplicate,
        SessionWorkResultCommitOutcome::AlreadyCommitted(_)
    ));

    let conflict = owner
        .commit_session_work_result(
            &admission.session_key,
            &admission.lifecycle_revision,
            "work-1",
            "idem-1",
            "different output",
            2,
        )
        .expect("conflicting receipt");
    assert_eq!(conflict, SessionWorkResultCommitOutcome::ConflictingResult);

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stale_background_result_cas_rolls_back_receipt_insert() {
    let root = temp_root("work-admission-stale-receipt");
    let agent_dir = root.join("agent-work");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");
    let admission = SessionWorkAdmissionRecord {
        session_key: "agent:agent-work:main".into(),
        session_id: "session-1".into(),
        lifecycle_revision: "revision-1".into(),
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: Some("work-stale".into()),
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    assert!(owner
        .compare_and_set_session_work_admission(None, &admission)
        .expect("create admission"));

    // Force the owner CAS to lose its lifecycle race after the receipt INSERT
    // but before the compare-and-set. The surrounding transaction must roll
    // back both the trigger update and the receipt insert.
    let trigger = rusqlite::Connection::open(&db_path).expect("trigger connection");
    trigger
        .execute_batch(
            "CREATE TRIGGER stale_work_admission_after_receipt
             AFTER INSERT ON transcript_entries
             WHEN NEW.id = 'background-result:idem-stale'
             BEGIN
               UPDATE kv
               SET value = replace(value, 'revision-1', 'revision-2')
               WHERE key = 'sessionWorkAdmission';
             END;",
        )
        .expect("install stale-race trigger");

    let outcome = owner
        .commit_session_work_result(
            &admission.session_key,
            &admission.lifecycle_revision,
            "work-stale",
            "idem-stale",
            "stale output",
            0,
        )
        .expect("stale result outcome");
    assert_eq!(outcome, SessionWorkResultCommitOutcome::RejectedStale);

    assert!(owner
        .get_transcript_entries()
        .expect("transcript remains readable")
        .iter()
        .all(|entry| entry.get("id").and_then(serde_json::Value::as_str)
            != Some("background-result:idem-stale")));
    let retained = owner
        .read_session_work_admission()
        .expect("read admission")
        .expect("admission remains");
    assert_eq!(retained.lifecycle_revision, "revision-1");
    assert_eq!(retained.admitted_work_id.as_deref(), Some("work-stale"));

    trigger
        .execute_batch("DROP TRIGGER stale_work_admission_after_receipt")
        .expect("remove stale-race trigger");
    owner.close(false);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn owner_routes_unread_request_episode_snapshot_and_spend_guard_mutations() {
    let root = temp_root("state");
    let agent_dir = root.join("agent-state");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    assert!(owner.mark_activity(10.0).expect("activity"));
    let unread = owner.get_unread_state().expect("unread after activity");
    assert_eq!(unread.last_activity_at, 10.0);
    assert_eq!(unread.unread_count, 1.0);

    assert!(owner.mark_viewed(12.0, false).expect("viewed"));
    assert!(owner.mark_unread(20.0).expect("manual unread"));
    let unread = owner.get_unread_state().expect("manual unread state");
    assert!(unread.is_manually_unread);
    assert_eq!(unread.unread_count, 1.0);
    assert!(owner.mark_read(21.0).expect("read"));
    assert!(!owner.get_unread_state().expect("read state").is_manually_unread);

    assert!(owner.set_introduction_pending(true).expect("intro"));
    assert!(owner.get_introduction_pending().expect("intro state"));

    let spend = mahayana_host_runtime::extensions::session::agent_db_serde::SpendGuardState {
        nudged_at_ms: Some(42.0),
        snoozed_until_ms: None,
        opted_out: true,
        card_entry_ids: vec!["card-a".into()],
        paused_automation_ids: vec!["auto-a".into()],
    };
    assert!(owner
        .set_automation_spend_guard_state(&spend)
        .expect("spend guard"));
    assert_eq!(
        owner.get_automation_spend_guard_state().expect("spend state"),
        spend
    );

    assert!(owner
        .record_request_id(" request-1 ", 30.0, Some(" prompt "), Some("turn"))
        .expect("request"));
    assert!(!owner
        .record_request_id("request-1", 31.0, None, None)
        .expect("duplicate request"));
    let requests = owner.get_request_ids().expect("requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].id, "request-1");
    assert_eq!(requests[0].prompt.as_deref(), Some("prompt"));

    assert!(owner
        .record_episode_turn(&mahayana_host_runtime::extensions::session::agent_db_serde::EpisodeTurn {
            ts: 40.0,
            user: "question".into(),
            agent: "answer".into(),
        })
        .expect("episode"));
    assert_eq!(owner.get_pending_episode_turns().expect("episodes").len(), 1);

    assert!(owner
        .set_memory_prompt_snapshot(&serde_json::json!({
            "render": "memory",
            "compactionEpoch": 2
        }))
        .expect("memory snapshot"));
    assert_eq!(
        owner
            .get_memory_prompt_snapshot()
            .expect("memory snapshot read")
            .map(|snapshot| snapshot.render),
        Some("memory".into())
    );

    assert!(owner
        .set_agent_profile_prompt_snapshot(&serde_json::json!({"name":"Agent"}))
        .expect("profile snapshot"));
    assert_eq!(
        owner
            .get_agent_profile_prompt_snapshot()
            .expect("profile snapshot read"),
        Some(serde_json::json!({"name":"Agent"}))
    );

    assert!(owner.clear_transient_state().expect("clear transient"));
    assert_eq!(owner.get_unread_state().expect("cleared unread").unread_count, 0.0);
    assert!(owner.get_request_ids().expect("cleared requests").is_empty());
    assert!(owner.get_pending_episode_turns().expect("cleared episodes").is_empty());
    assert!(owner
        .get_memory_prompt_snapshot()
        .expect("cleared memory snapshot")
        .is_none());
    assert!(owner
        .get_agent_profile_prompt_snapshot()
        .expect("cleared profile snapshot")
        .is_none());

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn owner_routes_partner_origin_purpose_and_transcript_mutations() {
    let root = temp_root("transcript-owner");
    let agent_dir = root.join("agent-transcript");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    assert_eq!(owner.get_agent_origin().expect("default origin"), "user");
    assert!(owner.set_agent_origin("dev").expect("set origin"));
    assert_eq!(owner.get_agent_origin().expect("dev origin"), "dev");

    assert!(owner.set_agent_purpose("research").expect("purpose"));
    assert_eq!(
        owner.get_agent_purpose().expect("purpose read").as_deref(),
        Some("research")
    );
    assert!(owner.clear_agent_purpose().expect("clear purpose"));
    assert!(owner.get_agent_purpose().expect("purpose cleared").is_none());

    assert!(!owner
        .add_conversation_partner("agent-transcript")
        .expect("reject self"));
    assert!(owner
        .add_conversation_partner(" partner-b ")
        .expect("partner"));
    assert!(!owner
        .add_conversation_partner("partner-b")
        .expect("partner dedupe"));
    assert_eq!(
        owner.get_conversation_partner_ids().expect("partners"),
        vec!["partner-b".to_string()]
    );

    assert_eq!(
        owner
            .append_transcript_entries(&[
                serde_json::json!({
                    "id":"entry-1",
                    "kind":"message",
                    "role":"user",
                    "content":"hello"
                }),
                serde_json::json!({
                    "id":"entry-2",
                    "kind":"notice",
                    "text":"notice"
                }),
            ])
            .expect("append"),
        2
    );
    assert_eq!(
        owner
            .append_transcript_entries(&[
                serde_json::json!({
                    "id":"entry-1",
                    "kind":"message",
                    "role":"user",
                    "content":"duplicate"
                }),
            ])
            .expect("duplicate append"),
        0
    );
    assert_eq!(
        owner
            .update_transcript_entry(
                "entry-1",
                &serde_json::json!({
                    "id":"entry-1",
                    "kind":"message",
                    "role":"user",
                    "content":"edited"
                }),
            )
            .expect("update")
            .and_then(|entry| entry.get("content").cloned()),
        Some(serde_json::Value::String("edited".into()))
    );
    assert!(owner
        .update_transcript_entry(
            "missing",
            &serde_json::json!({
                "id":"missing",
                "kind":"message",
                "role":"user",
                "content":"missing"
            }),
        )
        .expect("missing update")
        .is_none());
    assert!(owner.delete_transcript_entry("entry-2").expect("delete"));
    assert!(owner.delete_transcript_entry("entry-2").expect("idempotent committed delete"));
    assert_eq!(
        owner
            .append_transcript_entries(&[
                serde_json::json!({
                    "id":"branch-1",
                    "kind":"message",
                    "role":"assistant",
                    "content":"branch",
                    "replyTo":"entry-1",
                    "branched":true
                }),
            ])
            .expect("append branch"),
        1
    );

    assert_eq!(
        owner
            .get_transcript_entries()
            .expect("owner transcript entries")
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "branch-1"]
    );
    let page = owner
        .get_transcript_page(TranscriptPageQuery {
            before_seq: None,
            since_ms: None,
            until_ms: i64::MAX,
            limit: 10,
        })
        .expect("owner transcript page");
    assert_eq!(
        page.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1"]
    );
    let window = owner
        .get_transcript_window(TranscriptWindowQuery {
            before_seq: None,
            limit: 10,
        })
        .expect("owner transcript window");
    assert_eq!(
        window
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1"]
    );
    assert_eq!(window.thread_counts.get("entry-1"), Some(&1));
    let tail = owner
        .get_transcript_tail(TranscriptWindowQuery {
            before_seq: None,
            limit: 10,
        })
        .expect("owner transcript tail");
    assert_eq!(
        tail.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "branch-1"]
    );
    assert_eq!(
        owner
            .get_transcript_entry_by_id("entry-1")
            .expect("entry by id")
            .and_then(|entry| entry.get("content").cloned()),
        Some(serde_json::Value::String("edited".into()))
    );
    assert_eq!(
        owner
            .get_branched_entries()
            .expect("branched entries")
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["branch-1"]
    );
    assert_eq!(
        owner
            .get_thread_entries("entry-1")
            .expect("thread entries")
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "branch-1"]
    );

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owner_commits_guarded_compaction_and_replays_from_cut_point() {
    let root = temp_root("compaction");
    let agent_dir = root.join("agent-compaction");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    owner
        .append_transcript_entries(&[
            serde_json::json!({"id":"old", "kind":"message", "role":"user", "content":"old"}),
            serde_json::json!({"id":"kept", "kind":"message", "role":"user", "content":"kept"}),
        ])
        .expect("history");
    let expected_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("revision value");
    let commit = CompactionCommit {
        entry_id: "compaction-1".into(),
        expected_transcript_tail_id: "kept".into(),
        first_kept_entry_id: Some("kept".into()),
        summary: "summary".into(),
        tokens_before: 123,
        details: Some(serde_json::json!({"files": ["a.txt"]})),
        source_turn_id: "turn-1".into(),
        expected_lifecycle_revision: expected_revision.clone(),
    };

    let outcome = owner.commit_compaction(&commit).expect("commit");
    let committed = match outcome {
        CompactionCommitOutcome::Committed(entry) => entry,
        other => panic!("expected committed outcome, got {other:?}"),
    };
    assert_eq!(committed.get("kind").and_then(serde_json::Value::as_str), Some("compaction"));
    assert_ne!(
        owner
            .get_metadata("lifecycleRevision")
            .expect("next revision")
            .and_then(|value| value.as_str().map(ToOwned::to_owned)),
        Some(expected_revision.clone())
    );
    let replay = owner
        .get_compacted_transcript_entries()
        .expect("replay projection");
    assert_eq!(
        replay
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["compaction-1", "kept"]
    );

    assert!(matches!(
        owner.commit_compaction(&commit).expect("duplicate commit"),
        CompactionCommitOutcome::AlreadyCommitted(_)
    ));
    let mut normalized_duplicate = commit.clone();
    normalized_duplicate.summary = "  summary  ".into();
    assert!(matches!(
        owner
            .commit_compaction(&normalized_duplicate)
            .expect("normalized duplicate commit"),
        CompactionCommitOutcome::AlreadyCommitted(_)
    ));
    let mut conflict = commit.clone();
    conflict.summary = "different".into();
    assert!(matches!(
        owner.commit_compaction(&conflict),
        Err(mahayana_host_runtime::extensions::session::agent_db::AgentDbProjectionError::CompactionConflict(_))
    ));

    let mut stale = commit.clone();
    stale.entry_id = "compaction-stale".into();
    assert!(matches!(
        owner.commit_compaction(&stale).expect("stale commit"),
        CompactionCommitOutcome::RejectedStale
    ));
    let mut missing_cut = commit;
    missing_cut.entry_id = "compaction-missing".into();
    missing_cut.first_kept_entry_id = Some("missing".into());
    missing_cut.expected_lifecycle_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("current revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("current revision value");
    assert!(matches!(
        owner.commit_compaction(&missing_cut),
        Err(mahayana_host_runtime::extensions::session::agent_db::AgentDbProjectionError::CompactionInvalid(_))
    ));

    let mut empty_summary = commit.clone();
    empty_summary.entry_id = "compaction-empty-summary".into();
    empty_summary.expected_lifecycle_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("current revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("current revision value");
    empty_summary.summary = "  ".into();
    assert!(matches!(
        owner.commit_compaction(&empty_summary),
        Err(mahayana_host_runtime::extensions::session::agent_db::AgentDbProjectionError::CompactionInvalid(_))
    ));

    let mut oversized_summary = empty_summary;
    oversized_summary.entry_id = "compaction-oversized-summary".into();
    oversized_summary.summary = "x".repeat(16_001);
    assert!(matches!(
        owner.commit_compaction(&oversized_summary),
        Err(mahayana_host_runtime::extensions::session::agent_db::AgentDbProjectionError::CompactionInvalid(_))
    ));

    owner.close(true);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owner_rejects_compaction_when_transcript_tail_changes_after_planning() {
    let root = temp_root("compaction-tail-fence");
    let agent_dir = root.join("agent-compaction-tail-fence");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    owner
        .append_transcript_entries(&[
            serde_json::json!({"id":"old", "kind":"message", "role":"user", "content":"old"}),
            serde_json::json!({"id":"tail-before", "kind":"message", "role":"user", "content":"planned"}),
        ])
        .expect("history");
    let expected_revision = owner
        .get_metadata("lifecycleRevision")
        .expect("revision")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .expect("revision value");
    owner
        .append_transcript_entries(&[
            serde_json::json!({"id":"tail-after", "kind":"message", "role":"assistant", "content":"concurrent"}),
        ])
        .expect("concurrent append");

    let commit = CompactionCommit {
        entry_id: "compaction-tail-stale".into(),
        expected_transcript_tail_id: "tail-before".into(),
        first_kept_entry_id: Some("tail-before".into()),
        summary: "summary".into(),
        tokens_before: 123,
        details: None,
        source_turn_id: "turn-stale".into(),
        expected_lifecycle_revision: expected_revision,
    };
    assert!(matches!(
        owner.commit_compaction(&commit).expect("stale tail outcome"),
        CompactionCommitOutcome::RejectedStale
    ));
    assert!(owner
        .get_transcript_entries()
        .expect("transcript remains readable")
        .iter()
        .all(|entry| entry.get("kind").and_then(serde_json::Value::as_str) != Some("compaction")));

    owner.close(true);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn owner_closes_generic_metadata_versions_and_legacy_blob_surface() {
    let root = temp_root("metadata-surface");
    let agent_dir = root.join("agent-metadata");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    assert_eq!(
        owner
            .get_metadata("agentId")
            .expect("agent id metadata")
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .as_deref(),
        Some("agent-metadata")
    );
    let name_hits = Arc::new(AtomicUsize::new(0));
    let _name_sub = owner.subscribe_metadata("name", {
        let hits = Arc::clone(&name_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });
    assert!(owner
        .set_metadata("name", serde_json::Value::String("Renamed".into()))
        .expect("set name"));
    assert_eq!(name_hits.load(Ordering::SeqCst), 1);
    assert!(owner
        .set_metadata("name", serde_json::Value::String("Renamed".into()))
        .expect("idempotent set name"));
    assert_eq!(name_hits.load(Ordering::SeqCst), 1);

    let root_hits = Arc::new(AtomicUsize::new(0));
    let _root_sub = owner.subscribe_metadata("latestRootBlobId", {
        let hits = Arc::clone(&root_hits);
        Arc::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        })
    });
    assert!(owner
        .compare_and_set_latest_root_blob_id(&[], &[0xaa, 0xbb])
        .expect("root cas"));
    assert!(!owner
        .compare_and_set_latest_root_blob_id(&[], &[0xcc])
        .expect("stale root cas"));
    assert_eq!(owner.get_latest_root_blob_id().expect("latest root"), vec![0xaa, 0xbb]);
    assert_eq!(root_hits.load(Ordering::SeqCst), 1);

    assert!(owner.set_hidden_entry_repair_version(2).expect("hidden version"));
    assert_eq!(owner.get_hidden_entry_repair_version().expect("hidden version read"), 2);
    assert!(owner.set_stale_root_cleanup_version(3).expect("stale version"));
    assert_eq!(owner.get_stale_root_cleanup_version().expect("stale version read"), 3);

    let legacy = rusqlite::Connection::open(&db_path).expect("legacy writer");
    legacy
        .execute(
            "INSERT INTO blobs (id, data) VALUES (?1, ?2)",
            rusqlite::params!["aa", vec![1_u8, 2, 3]],
        )
        .expect("legacy blob");
    drop(legacy);
    assert!(owner.has_legacy_conversation_blobs().expect("legacy present"));
    assert!(owner
        .retire_legacy_conversation_blobs(4)
        .expect("retire legacy blobs"));
    assert!(!owner.has_legacy_conversation_blobs().expect("legacy retired"));
    assert_eq!(
        owner
            .get_legacy_blob_retirement_version()
            .expect("legacy version"),
        4
    );

    let snapshot = owner.serde_snapshot().expect("serde snapshot");
    assert!(snapshot.request_ids.is_empty());

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owner_matches_frozen_main_transcript_and_missing_thread_semantics() {
    let root = temp_root("main-transcript");
    let agent_dir = root.join("agent-main");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    let owner = SandAgentDb::open(&db_path, 50).expect("owner");

    for entry in [
        serde_json::json!({
            "id":"root",
            "kind":"message",
            "role":"user",
            "content":"root"
        }),
        serde_json::json!({
            "id":"branch",
            "kind":"message",
            "role":"assistant",
            "content":"branch",
            "replyTo":"root",
            "branched":true
        }),
        serde_json::json!({
            "id":"nested",
            "kind":"message",
            "role":"assistant",
            "content":"nested",
            "replyTo":"branch",
            "branched":true
        }),
        serde_json::json!({
            "id":"orphan",
            "kind":"message",
            "role":"assistant",
            "content":"orphan",
            "replyTo":"missing-root",
            "branched":true
        }),
    ] {
        assert!(owner.append_transcript_entry(&entry).expect("append entry"));
    }
    assert!(!owner
        .append_transcript_entry(&serde_json::json!({
            "id":"root",
            "kind":"message",
            "role":"user",
            "content":"duplicate"
        }))
        .expect("duplicate append"));

    assert_eq!(
        owner
            .get_main_transcript_entries()
            .expect("main transcript")
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root", "orphan"]
    );
    assert_eq!(
        owner
            .get_thread_entries("root")
            .expect("root thread")
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root", "branch", "nested"]
    );
    assert!(owner
        .get_thread_entries("missing-root")
        .expect("missing root thread")
        .is_empty());

    owner.close(false);
    let _ = fs::remove_dir_all(root);
}
