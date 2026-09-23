use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db_schema::AGENT_DB_SCHEMA;
use mahayana_host_runtime::extensions::session::agent_db_transcript_pages::{
    TranscriptPageQuery, TranscriptWindowQuery,
};
use mahayana_host_runtime::extensions::session::session_conversation_state::SessionConversationState;
use rusqlite::params;
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-conversation-state-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn seed(db_path: &std::path::Path) {
    let db = rusqlite::Connection::open(db_path).expect("db");
    db.execute_batch(AGENT_DB_SCHEMA).expect("schema");
    for (id, entry) in [
        ("root", json!({"id":"root","kind":"message","role":"user","content":"root","timestampMs":10})),
        ("branch-1", json!({"id":"branch-1","kind":"message","role":"assistant","content":"one","branched":true,"replyTo":"root","timestampMs":11})),
        ("branch-2", json!({"id":"branch-2","kind":"message","role":"assistant","content":"two","branched":true,"replyTo":"branch-1","timestampMs":12})),
        ("other-branch", json!({"id":"other-branch","kind":"message","role":"assistant","content":"other","branched":true,"replyTo":"other-root","timestampMs":13})),
        ("tool", json!({"id":"tool","kind":"tool-call","name":"search","timestampMs":14})),
        ("notice", json!({"id":"notice","kind":"notice","text":"notice","timestampMs":15})),
    ] {
        db.execute(
            "INSERT INTO transcript_entries (id, entry) VALUES (?1, ?2)",
            params![id, entry.to_string()],
        )
        .expect("entry");
    }
}

#[test]
fn transcript_read_owner_matches_frozen_page_window_tail_and_thread_semantics() {
    let root = temp_root("reads");
    let agent_dir = root.join("agent-a");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    let db_path = agent_dir.join("store.db");
    seed(&db_path);
    let state = SessionConversationState::new(500);

    let all = state
        .read_agent_transcript_entries(&db_path)
        .expect("all entries");
    assert_eq!(all.len(), 6);

    let page = state
        .read_agent_transcript_page(
            &db_path,
            TranscriptPageQuery {
                before_seq: None,
                since_ms: None,
                until_ms: 100,
                limit: 5,
            },
        )
        .expect("page");
    assert_eq!(
        page.entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root"]
    );

    let window = state
        .read_agent_transcript_window(
            &db_path,
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
        vec!["root", "notice"]
    );
    assert_eq!(
        window.thread_counts,
        BTreeMap::from([("root".to_string(), 2usize)])
    );

    let tail = state
        .read_agent_transcript_tail(
            &db_path,
            TranscriptWindowQuery {
                before_seq: None,
                limit: 10,
            },
        )
        .expect("tail");
    assert_eq!(tail.entries.len(), 6);

    let thread = state
        .read_agent_thread(&db_path, "root")
        .expect("thread");
    assert_eq!(
        thread
            .entries
            .iter()
            .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["root", "branch-1", "branch-2"]
    );
    assert!(
        state
            .read_agent_thread(&db_path, "missing")
            .expect("missing thread")
            .entries
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
