use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_paths::{
    CONVERSATION_BLOBS_FILENAME, STORE_FILENAME,
};

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-production-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn production_session_workers_bind_worker_blob_store_to_frozen_session_paths() {
    let root = temp_root("paths");
    let runtime = ProductionSessionWorkers::with_agents_root(&root, 500);

    let store = runtime
        .create_agent_blob_store("agent-a")
        .expect("production blob store");
    assert_eq!(
        store.blob_db_path,
        root.join("agent-a").join(CONVERSATION_BLOBS_FILENAME)
    );
    assert_eq!(
        store.legacy_blob_db_path,
        Some(root.join("agent-a").join(STORE_FILENAME))
    );
    assert_eq!(runtime.active_worker_count(), 0);
    assert!(runtime.create_agent_blob_store("../escape").is_err());

    runtime.shutdown();
}

#[test]
fn production_session_workers_use_real_sqlite_backend_and_close_on_host_shutdown() {
    let root = temp_root("sqlite");
    let agent_dir = root.join("agent-live");
    fs::create_dir_all(&agent_dir).expect("agent directory");
    let session_db_path = agent_dir.join(STORE_FILENAME);
    let connection = rusqlite::Connection::open(&session_db_path).expect("session sqlite");
    connection
        .execute_batch(
            "CREATE TABLE kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            ) STRICT;
            CREATE TABLE transcript_entries (
                seq INTEGER PRIMARY KEY,
                id TEXT NOT NULL UNIQUE,
                entry TEXT NOT NULL
            ) STRICT;",
        )
        .expect("session kv");
    let persisted_root = vec![0xaa, 0xbb, 0xcc];
    let metadata = serde_json::to_vec(&serde_json::json!({
        "agentId": "agent-live",
        "latestRootBlobId": to_hex(&persisted_root),
        "name": "Agent Live",
        "mode": "default",
        "isRunEverything": false,
        "createdAt": 1
    }))
    .expect("metadata json");
    connection
        .execute(
            "INSERT INTO kv (key, value) VALUES ('metadata', ?1)",
            rusqlite::params![to_hex(&metadata)],
        )
        .expect("metadata row");
    for (key, value) in [
        (
            "sandProfile",
            r#"{"description":"Shipping profile","avatarPath":"/tmp/avatar.png"}"#,
        ),
        (
            "unreadState",
            r#"{"lastActivityAt":123,"lastViewedAt":"bad","isManuallyUnread":true,"unreadCount":2.9}"#,
        ),
        (
            "automationSpendGuardState",
            r#"{"nudgedAtMs":42,"snoozedUntilMs":-1,"optedOut":true,"cardEntryIds":["card","",2],"pausedAutomationIds":["auto"]}"#,
        ),
        (
            "awaitingUserResponse",
            r#"{"tabId":"tab-1","reason":7,"since":44}"#,
        ),
        (
            "requestIds",
            r#"[{"id":" request-1 ","at":9,"prompt":"hello","source":"turn"},{"id":"   ","at":3}]"#,
        ),
        (
            "episodePending",
            r#"[{"ts":5,"user":"question","agent":"answer"},{"ts":7,"user":"","agent":""}]"#,
        ),
        (
            "memoryPromptSnapshot",
            r#"{"render":"memory","compactionEpoch":3}"#,
        ),
    ] {
        connection
            .execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            )
            .expect("session state row");
    }
    for (id, entry) in [
        (
            "entry-1",
            serde_json::json!({"id":"entry-1","kind":"message","role":"user","content":"first"}),
        ),
        (
            "entry-2",
            serde_json::json!({"id":"entry-2","kind":"notice","text":"second"}),
        ),
    ] {
        connection
            .execute(
                "INSERT INTO transcript_entries (id, entry) VALUES (?1, ?2)",
                rusqlite::params![id, entry.to_string()],
            )
            .expect("transcript row");
    }
    drop(connection);

    let runtime = ProductionSessionWorkers::with_agents_root(&root, 500);
    let store = runtime
        .create_agent_blob_store("agent-live")
        .expect("worker blob store");
    let blob_id = b"production-worker-blob";
    futures::executor::block_on(store.set_blob(&(), blob_id, b"hello"))
        .expect("write through worker pool");
    let read = futures::executor::block_on(store.get_blob(&(), blob_id))
        .expect("read through worker pool");
    assert_eq!(read.as_deref(), Some(b"hello".as_slice()));
    assert_eq!(runtime.active_worker_count(), 1);

    let prepared = runtime
        .prepare_existing_agent("agent-live")
        .expect("prepare existing agent")
        .expect("existing agent session");
    assert_eq!(prepared.session_db_path, session_db_path);
    assert_eq!(
        prepared.blob_db_path,
        agent_dir.join(CONVERSATION_BLOBS_FILENAME)
    );
    assert_eq!(prepared.persisted_root_blob_id, vec![0xaa, 0xbb, 0xcc]);
    assert_eq!(prepared.session_state.profile.description, "Shipping profile");
    assert_eq!(
        prepared.session_state.profile.avatar_path.as_deref(),
        Some("/tmp/avatar.png")
    );
    assert_eq!(prepared.session_state.unread_state.last_activity_at, 123.0);
    assert_eq!(prepared.session_state.unread_state.last_viewed_at, 0.0);
    assert!(prepared.session_state.unread_state.is_manually_unread);
    assert_eq!(prepared.session_state.unread_state.unread_count, 2.0);
    assert_eq!(prepared.session_state.spend_guard_state.nudged_at_ms, Some(42.0));
    assert_eq!(prepared.session_state.spend_guard_state.snoozed_until_ms, None);
    assert!(prepared.session_state.spend_guard_state.opted_out);
    assert_eq!(
        prepared.session_state.spend_guard_state.card_entry_ids,
        vec!["card".to_string()]
    );
    assert_eq!(
        prepared.session_state.awaiting_user_response.as_ref().map(|state| state.tab_id.as_str()),
        Some("tab-1")
    );
    assert_eq!(prepared.session_state.request_ids.len(), 1);
    assert_eq!(prepared.session_state.request_ids[0].id, "request-1");
    assert_eq!(prepared.session_state.request_ids[0].source.as_deref(), Some("turn"));
    assert_eq!(prepared.session_state.pending_episode_turns.len(), 1);
    assert_eq!(prepared.session_state.pending_episode_turns[0].agent, "answer");
    assert_eq!(
        prepared.session_state.memory_prompt_snapshot.as_ref().map(|snapshot| snapshot.render.as_str()),
        Some("memory")
    );
    assert_eq!(
        prepared
            .transcript_tail
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["entry-1", "entry-2"]
    );
    assert_eq!(prepared.transcript_tail.next_before_seq, None);
    assert_eq!(runtime.active_worker_count(), 1);

    runtime.shutdown();
    assert_eq!(runtime.active_worker_count(), 0);
    let _ = fs::remove_dir_all(root);
}
