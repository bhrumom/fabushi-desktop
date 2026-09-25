use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::workflow_commands::dispatch_workflow_command;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-workflow-commands-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn call(
    workers: Arc<ProductionSessionWorkers>,
    method: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    dispatch_workflow_command(workers, method, &args)
        .expect("workflow command handled")
        .expect("workflow command result")
}

#[test]
fn shipping_workflow_commands_delegate_crud_and_imports_to_session_store() {
    let root = temp_root("crud");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");

    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Research",
                "description": "Research workflow",
                "body": "Read sources and summarize.",
                "sourceRef": "local"
            }
        }),
    );
    assert_eq!(created.as_array().map(Vec::len), Some(1));
    let workflow_id = created[0]["id"].as_str().expect("workflow id").to_string();
    assert_eq!(created[0]["name"], "Research");
    assert_eq!(created[0]["sourceRef"], "local");
    assert_eq!(created[0]["isEnabledForAgent"], true);

    let listed = call(
        Arc::clone(&workers),
        "getAgentWorkflows",
        serde_json::json!({"id": agent.id}),
    );
    assert_eq!(listed[0]["id"], workflow_id);

    let updated = call(
        Arc::clone(&workers),
        "updateAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id,
            "spec": {
                "name": "Research updated",
                "description": "Updated",
                "body": "Read more sources and summarize."
            }
        }),
    );
    assert_eq!(updated[0]["name"], "Research updated");

    let disabled = call(
        Arc::clone(&workers),
        "setAgentWorkflowEnabled",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id,
            "isEnabled": false
        }),
    );
    assert_eq!(disabled[0]["isEnabledForAgent"], false);

    let imported = call(
        Arc::clone(&workers),
        "importAgentWorkflowText",
        serde_json::json!({
            "id": agent.id,
            "name": "Imported text",
            "markdown": "---\nname: Imported\n---\nDo the imported thing."
        }),
    );
    assert_eq!(imported["result"]["imported"].as_array().map(Vec::len), Some(1));

    let linked = call(
        Arc::clone(&workers),
        "importAgentWorkflowUrl",
        serde_json::json!({
            "id": agent.id,
            "url": "https://example.test/SKILL.md",
            "name": "Live skill"
        }),
    );
    assert_eq!(linked["result"]["imported"].as_array().map(Vec::len), Some(1));
    assert!(linked["workflows"]
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["sourceRef"] == "https://example.test/SKILL.md"));

    let after_delete = call(
        Arc::clone(&workers),
        "deleteAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "workflowId": workflow_id
        }),
    );
    assert!(!after_delete
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["id"] == workflow_id));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn automation_backed_workflow_creation_is_owned_by_rust_session_store() {
    let root = temp_root("automation-create");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let session = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = session.create_session(None, "user", None).expect("agent");

    let created = call(
        Arc::clone(&workers),
        "createAgentWorkflow",
        serde_json::json!({
            "id": agent.id,
            "spec": {
                "name": "Scheduled",
                "body": "Run later",
                "trigger": {"schedule":"0 9 * * *","isEnabled":true}
            }
        }),
    );
    let created = created.as_array().expect("workflow list");
    assert_eq!(created.len(), 1);
    assert_eq!(created[0]["name"], "Scheduled");
    assert_eq!(created[0]["source"], "automation");
    assert_eq!(created[0]["trigger"]["schedule"], "0 9 * * *");
    assert_eq!(created[0]["trigger"]["isEnabled"], true);

    let listed = call(
        Arc::clone(&workers),
        "getAgentWorkflows",
        serde_json::json!({"id": agent.id}),
    );
    assert!(listed
        .as_array()
        .expect("workflow list")
        .iter()
        .any(|workflow| workflow["source"] == "automation"));

    assert!(dispatch_workflow_command(
        Arc::clone(&workers),
        "runAgentWorkflowNow",
        &serde_json::json!({"id":agent.id,"workflowId":"missing"}),
    )
    .is_none());

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
