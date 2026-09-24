use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::settings_file::{
    get_sand_settings_path, read_sand_settings_file,
};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::memory::memory_service::MemoryKind;
use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::workflows::workflow_library::WorkflowSpec;
use mahayana_host_runtime::extensions::session::agent_db_serde::AwaitingUserResponse;
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
