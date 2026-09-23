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
    assert_eq!(runtime.active_worker_count(), 1);

    runtime.shutdown();
    assert_eq!(runtime.active_worker_count(), 0);
    let _ = fs::remove_dir_all(root);
}
