use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::settings_file::{
    get_sand_settings_path, read_sand_settings_file,
};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::agent_db_transcript_pages::{
    TranscriptPageQuery, TranscriptWindowQuery,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::transcript_mutation_events::subscribe_transcript_mutations;

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
    assert_eq!(removals.load(Ordering::SeqCst), 1);

    subscription.unsubscribe();
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
