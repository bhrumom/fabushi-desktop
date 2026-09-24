use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::session_runtime::SessionRuntime;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-transcript-session-runtime-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn focus_and_switch_follow_frozen_active_session_viewed_semantics() {
    let root = temp_root("switch");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&sessions));
    let first = store.create_session(None, "user", None).expect("first");
    let second = store.create_session(None, "user", None).expect("second");
    store
        .write_active_agent_id(&first.id)
        .expect("seed active pointer");
    sessions
        .append_agent_transcript_entries(
            &second.id,
            &[serde_json::json!({"id":"second-1","kind":"message","role":"user","content":"hello"})],
        )
        .expect("second transcript");

    sessions
        .mark_agent_activity(&first.id, 100.0)
        .expect("first activity");
    let runtime = SessionRuntime::new();
    assert_eq!(runtime.window_focused_at_ms(), None);
    assert!(runtime
        .set_window_focused(&sessions, true, 200.0)
        .expect("focus"));
    assert_eq!(runtime.window_focused_at_ms(), Some(200.0));

    let focused_first = sessions
        .open_agent_db_owner(&first.id)
        .expect("first owner")
        .get_unread_state()
        .expect("first unread");
    assert_eq!(focused_first.last_viewed_at, 200.0);

    sessions
        .mark_agent_activity(&first.id, 300.0)
        .expect("new first activity");
    sessions
        .set_agent_unread(&second.id, true, 250.0)
        .expect("manual second unread");

    let entries = runtime
        .switch_agent(&sessions, &second.id, 400.0)
        .expect("switch");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], "second-1");
    assert_eq!(runtime.get_entries(), entries);
    runtime.append_entry(serde_json::json!({"id":"local","kind":"notice","text":"local"}));
    assert_eq!(runtime.get_entries().len(), 2);
    assert!(runtime.remove_entry("local"));
    assert_eq!(runtime.get_entries().len(), 1);
    assert_eq!(
        store.read_active_agent_id().as_deref(),
        Some(second.id.as_str())
    );

    let left_behind = sessions
        .open_agent_db_owner(&first.id)
        .expect("first owner")
        .get_unread_state()
        .expect("first unread");
    assert_eq!(left_behind.last_viewed_at, 400.0);

    let activated = sessions
        .open_agent_db_owner(&second.id)
        .expect("second owner")
        .get_unread_state()
        .expect("second unread");
    assert_eq!(activated.last_viewed_at, 400.0);
    assert!(!activated.is_manually_unread);

    assert!(!runtime
        .set_window_focused(&sessions, false, 500.0)
        .expect("blur"));
    assert_eq!(runtime.window_focused_at_ms(), None);

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn opening_the_already_active_agent_does_not_rewrite_viewed_state() {
    let root = temp_root("same-agent");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&sessions));
    let agent = store.create_session(None, "user", None).expect("agent");
    store
        .write_active_agent_id(&agent.id)
        .expect("active pointer");
    sessions
        .mark_agent_activity(&agent.id, 100.0)
        .expect("activity");
    sessions
        .append_agent_transcript_entries(
            &agent.id,
            &[serde_json::json!({"id":"same-1","kind":"message","role":"user","content":"seed"})],
        )
        .expect("same transcript");

    let runtime = SessionRuntime::new();
    let first_snapshot = runtime
        .switch_agent(&sessions, &agent.id, 900.0)
        .expect("same active");
    assert_eq!(first_snapshot.len(), 1);
    assert_eq!(runtime.get_entries(), first_snapshot);
    runtime.append_entry(serde_json::json!({"id":"memory-only","kind":"notice","text":"cached"}));
    let second_snapshot = runtime
        .switch_agent(&sessions, &agent.id, 901.0)
        .expect("same active cached");
    assert_eq!(second_snapshot.len(), 2);
    assert_eq!(second_snapshot[1]["id"], "memory-only");
    let unread = sessions
        .open_agent_db_owner(&agent.id)
        .expect("owner")
        .get_unread_state()
        .expect("unread");
    assert_eq!(unread.last_viewed_at, 0.0);

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
