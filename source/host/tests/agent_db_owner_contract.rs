use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db::{
    SandAgentDb, SandAgentDbOptions,
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

    assert!(owner
        .compare_and_set_latest_root_blob_id(&[], &[0xaa, 0xbb])
        .expect("root cas"));
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
