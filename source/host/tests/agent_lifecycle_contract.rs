use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::{
    AgentLifecycleGatewayError, ProductionAgentLifecycle,
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
