use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::{
    AgentDeletionRuntimeDeps, AgentLifecycleGatewayError, ProductionAgentLifecycle,
    dispatch_production_agent_lifecycle_gateway_call,
};
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-agent-lifecycle-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn deletion_runtime_runs_owner_hooks_around_durable_session_delete() {
    let root = temp_root("runtime-cleanup");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store.create_session(None, "user", None).expect("agent");

    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let hook = |label: &'static str, calls: Arc<Mutex<Vec<String>>>| {
        Arc::new(move |agent_id: &str| {
            calls.lock().expect("calls").push(format!("{label}:{agent_id}"));
            Ok(())
        }) as mahayana_host_runtime::extensions::transcript::agent_lifecycle::AgentDeletionHook
    };
    let lifecycle = ProductionAgentLifecycle::with_deletion_runtime(
        Arc::clone(&production),
        AgentDeletionRuntimeDeps {
            cancel_runner: Some(hook("runner", Arc::clone(&calls))),
            forget_ack: Some(hook("ack", Arc::clone(&calls))),
            release_box: Some(hook("box", Arc::clone(&calls))),
            forget_handoff: Some(hook("handoff", Arc::clone(&calls))),
        },
    );

    lifecycle.delete_agent(&record.id).expect("delete");
    assert!(!store.agent_dir_exists(&record.id));
    let expected = vec![
        format!("runner:{}", record.id),
        format!("ack:{}", record.id),
        format!("box:{}", record.id),
        format!("handoff:{}", record.id),
    ];
    assert_eq!(*calls.lock().expect("calls"), expected);

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn deleting_active_agent_selects_existing_successor_and_returns_its_transcript() {
    let root = temp_root("active-successor");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let first = store
        .create_session(None, "user", None)
        .expect("first agent");
    let second = store
        .create_session(None, "user", None)
        .expect("second agent");
    store
        .write_active_agent_id(&first.id)
        .expect("active pointer");
    production
        .append_agent_transcript_entries(
            &second.id,
            &[json!({
                "id":"successor-entry",
                "kind":"message",
                "role":"user",
                "content":"survives delete"
            })],
        )
        .expect("successor transcript");

    let lifecycle = ProductionAgentLifecycle::new(Arc::clone(&production));
    let result = lifecycle.delete_agent(&first.id).expect("delete active");

    assert!(!store.agent_dir_exists(&first.id));
    assert!(store.agent_exists(&second.id));
    assert_eq!(store.read_active_agent_id().as_deref(), Some(second.id.as_str()));
    assert_eq!(result["transcript"].as_array().map(Vec::len), Some(1));
    assert_eq!(result["transcript"][0]["id"], "successor-entry");

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn deleting_non_active_agent_keeps_active_transcript_and_delete_all_clears_pointer() {
    let root = temp_root("batch");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let active = store
        .create_session(None, "user", None)
        .expect("active");
    let other = store
        .create_session(None, "user", None)
        .expect("other");
    store
        .write_active_agent_id(&active.id)
        .expect("active pointer");
    production
        .append_agent_transcript_entries(
            &active.id,
            &[json!({
                "id":"active-entry",
                "kind":"message",
                "role":"user",
                "content":"keep me"
            })],
        )
        .expect("active transcript");

    let lifecycle = ProductionAgentLifecycle::new(Arc::clone(&production));
    let kept = lifecycle
        .delete_agents(&[other.id.clone(), other.id.clone()])
        .expect("delete duplicate non-active");
    assert_eq!(store.read_active_agent_id().as_deref(), Some(active.id.as_str()));
    assert_eq!(kept["transcript"][0]["id"], "active-entry");

    let cleared = lifecycle
        .delete_agents(std::slice::from_ref(&active.id))
        .expect("delete last active");
    assert_eq!(store.read_active_agent_id(), None);
    assert_eq!(cleared["transcript"], json!([]));
    assert!(!store.active_agent_pointer_path().exists());

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn duplicate_agent_gateway_uses_clone_owner_and_activates_empty_copy() {
    use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
    use mahayana_host_runtime::extensions::session::agent_db::{
        read_persisted_agent_metadata_projection, read_persisted_agent_origin,
        read_persisted_agent_purpose,
    };

    let root = temp_root("duplicate");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let source = store
        .create_session(
            Some(&SandAgentProfile {
                name: "Alpha".into(),
                description: "source".into(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "dev",
            Some("disk-saver"),
        )
        .expect("source");
    production
        .append_agent_transcript_entries(
            &source.id,
            &[json!({
                "id":"source-entry",
                "kind":"message",
                "role":"user",
                "content":"do not copy chat"
            })],
        )
        .expect("source transcript");
    store
        .write_active_agent_id(&source.id)
        .expect("active source");

    let duplicated = dispatch_production_agent_lifecycle_gateway_call(
        &production,
        "duplicateAgent",
        &json!({"id":source.id}),
    )
    .expect("handled")
    .expect("duplicated");

    let clone_id = duplicated["agent"]["id"]
        .as_str()
        .expect("clone id")
        .to_string();
    assert_ne!(clone_id, source.id);
    assert_eq!(duplicated["agent"]["name"], "Alpha copy");
    assert_eq!(duplicated["transcript"], json!([]));
    assert_eq!(store.read_active_agent_id().as_deref(), Some(clone_id.as_str()));
    assert_eq!(
        read_persisted_agent_metadata_projection(
            &production.session_db_path(&clone_id).expect("clone db"),
            500
        )
        .expect("metadata")
        .expect("metadata present")
        .agent_id,
        clone_id
    );
    assert_eq!(
        read_persisted_agent_origin(
            &production.session_db_path(&clone_id).expect("clone db"),
            500
        )
        .expect("origin"),
        "user"
    );
    assert_eq!(
        read_persisted_agent_purpose(
            &production.session_db_path(&clone_id).expect("clone db"),
            500
        )
        .expect("purpose"),
        None
    );
    assert_eq!(
        store.read_agent_transcript_entries(&source.id)
            .expect("source transcript")
            .len(),
        1
    );

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_gateway_contract_validates_delete_arguments_and_uses_lifecycle_owner() {
    let root = temp_root("gateway");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store
        .create_session(None, "user", None)
        .expect("agent");
    store
        .write_active_agent_id(&record.id)
        .expect("active pointer");

    assert_eq!(
        dispatch_production_agent_lifecycle_gateway_call(
            &production,
            "deleteAgent",
            &json!({"id":""}),
        ),
        Some(Err(AgentLifecycleGatewayError::BadRequest(
            "missing or invalid id".into()
        )))
    );
    let deleted = dispatch_production_agent_lifecycle_gateway_call(
        &production,
        "deleteAgents",
        &json!({"ids":[record.id]}),
    )
    .expect("handled")
    .expect("deleted");
    assert_eq!(deleted["transcript"], json!([]));
    assert_eq!(store.read_active_agent_id(), None);

    assert!(dispatch_production_agent_lifecycle_gateway_call(
        &production,
        "not-a-lifecycle-method",
        &json!({}),
    )
    .is_none());

    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}
