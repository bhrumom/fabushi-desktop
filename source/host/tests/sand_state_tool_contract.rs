use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::memory::agent_state::SandAgentState;
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, get_agent_memory_dir,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_state_tool::{
    SAND_UPDATE_STATE_TOOL_NAME, SandStateToolBridge, SandStateWriter,
};
use serde_json::{Value, json};

struct DelegateBridge;

impl RoutedToolBridge for DelegateBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "delegate_tool".into(),
            provider_identifier: "delegate".into(),
            tool_name: "delegate_tool".into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(Value::String("delegated".into()))
    }
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-sand-state-tool-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn update_state_bridge_advertises_tool_and_delegates_unknown_tools() {
    let root = temp_root("advertise");
    let state = Arc::new(SandAgentState::new(&root, "agent-a").expect("state"));
    let writer: Arc<dyn SandStateWriter> = state;
    let bridge = SandStateToolBridge::new(Arc::new(DelegateBridge), writer);

    let tools = bridge.list_tools().expect("tools");
    assert_eq!(tools[0].name, SAND_UPDATE_STATE_TOOL_NAME);
    assert!(tools.iter().any(|tool| tool.name == "delegate_tool"));

    let delegate = tools.iter().find(|tool| tool.name == "delegate_tool").expect("delegate");
    assert_eq!(
        bridge.call_tool(delegate, json!({}), "tool-delegate").expect("delegate result"),
        Value::String("delegated".into())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn update_state_bridge_routes_memory_routines_and_workflows_to_host_owner() {
    let root = temp_root("routing");
    let state = Arc::new(SandAgentState::new(&root, "agent-b").expect("state"));
    let writer: Arc<dyn SandStateWriter> = state.clone();
    let bridge = SandStateToolBridge::new(Arc::new(DelegateBridge), writer);
    let tools = bridge.list_tools().expect("tools");
    let tool = tools
        .iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME)
        .expect("update_state");

    let memory = bridge.call_tool(
        tool,
        json!({
            "target":"memory",
            "action":"write",
            "fact":"User prefers concise status reports",
            "tier":"profile"
        }),
        "tool-memory",
    ).expect("memory");
    assert!(memory.as_str().is_some_and(|value| value.contains("Remembered")));
    let store = FileMemoryStore::new(get_agent_memory_dir(
        root.join("agents").join("agent-b"),
    ));
    assert_eq!(store.count_memories(), 1);

    let created = bridge.call_tool(
        tool,
        json!({
            "target":"routine",
            "action":"create",
            "name":"Morning brief",
            "prompt":"Summarize important changes",
            "schedule":"0 9 * * *"
        }),
        "tool-routine-create",
    ).expect("routine create");
    assert!(created.as_str().is_some_and(|value| value.contains("Saved routine")));

    let updated = bridge.call_tool(
        tool,
        json!({
            "target":"routine",
            "action":"update",
            "id":"morning-brief",
            "prompt":"Summarize only actionable changes"
        }),
        "tool-routine-update",
    ).expect("routine update");
    assert!(updated.as_str().is_some_and(|value| value.contains("Updated routine")));
    let routine = state.automation_record("morning-brief").expect("routine");
    assert_eq!(routine.name, "Morning brief");
    assert_eq!(routine.prompt, "Summarize only actionable changes");
    assert_eq!(routine.schedule, "0 9 * * *");

    let workflow = bridge.call_tool(
        tool,
        json!({
            "target":"workflow",
            "action":"write",
            "name":"Deploy checklist",
            "description":"Use this when preparing a release",
            "body":"# Deploy\nVerify the package."
        }),
        "tool-workflow",
    ).expect("workflow");
    assert!(workflow.as_str().is_some_and(|value| value.contains("Saved workflow")));
    assert!(root.join("workflows").join("deploy-checklist").join("SKILL.md").is_file());

    let _ = fs::remove_dir_all(root);
}


#[test]
fn routine_writes_run_auto_review_before_state_mutation() {
    let root = temp_root("routine-review");
    let state = Arc::new(SandAgentState::new(&root, "agent-review").expect("state"));
    let writer: Arc<dyn SandStateWriter> = state.clone();
    let bridge = SandStateToolBridge::new(Arc::new(DelegateBridge), writer)
        .with_routine_auto_review(Arc::new(|target, tool_call_id| {
            assert_eq!(target.operation, "create");
            assert_eq!(target.name, "Reviewed routine");
            assert_eq!(tool_call_id, "tool-routine-reviewed");
            Err(ProviderSessionError::Tool("review denied".into()))
        }));
    let tool = bridge
        .list_tools()
        .expect("tools")
        .into_iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME)
        .expect("update_state");
    let error = bridge.call_tool(
        &tool,
        json!({
            "target":"routine",
            "action":"create",
            "name":"Reviewed routine",
            "prompt":"Do something",
            "schedule":"0 8 * * *"
        }),
        "tool-routine-reviewed",
    ).expect_err("review must block write");
    assert!(error.to_string().contains("review denied"));
    assert!(state.automation_record("reviewed-routine").is_none());
    let _ = fs::remove_dir_all(root);
}


#[test]
fn pause_resume_and_delete_do_not_enter_routine_auto_review() {
    let root = temp_root("routine-nonreview-actions");
    let state = Arc::new(SandAgentState::new(&root, "agent-nonreview").expect("state"));
    let created = state.create_automation(&mahayana_host_runtime::automations::automation::AutomationSpec {
        name: "Daily".into(),
        prompt: "Summarize".into(),
        trigger: json!({"type":"cron","schedule":"0 8 * * *"}),
        is_enabled: Some(true),
    });
    assert!(created.ok);
    let writer: Arc<dyn SandStateWriter> = state.clone();
    let bridge = SandStateToolBridge::new(Arc::new(DelegateBridge), writer)
        .with_routine_auto_review(Arc::new(|_, _| {
            Err(ProviderSessionError::Tool("review should not run".into()))
        }));
    let tool = bridge
        .list_tools()
        .expect("tools")
        .into_iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME)
        .expect("update_state");
    bridge.call_tool(
        &tool,
        json!({"target":"routine","action":"pause","id":"daily"}),
        "tool-pause",
    ).expect("pause bypasses auto-review");
    bridge.call_tool(
        &tool,
        json!({"target":"routine","action":"resume","id":"daily"}),
        "tool-resume",
    ).expect("resume bypasses auto-review");
    bridge.call_tool(
        &tool,
        json!({"target":"routine","action":"delete","id":"daily"}),
        "tool-delete",
    ).expect("delete bypasses auto-review");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workflow_body_write_is_reviewed_only_when_a_routine_references_it() {
    let root = temp_root("workflow-routine-review");
    let state = Arc::new(SandAgentState::new(&root, "agent-workflow-review").expect("state"));
    assert!(state.write_workflow(
        None,
        "Release Playbook",
        Some("Release steps"),
        "old body",
    ).ok);
    assert!(state.create_automation(&mahayana_host_runtime::automations::automation::AutomationSpec {
        name: "Release watcher".into(),
        prompt: "Follow the Release Playbook when CI passes".into(),
        trigger: json!({"type":"cron","schedule":"0 8 * * *"}),
        is_enabled: Some(true),
    }).ok);
    let writer: Arc<dyn SandStateWriter> = state.clone();
    let bridge = SandStateToolBridge::new(Arc::new(DelegateBridge), writer)
        .with_routine_auto_review(Arc::new(|target, tool_call_id| {
            assert_eq!(tool_call_id, "tool-workflow");
            assert_eq!(target.operation, "workflow_body");
            assert_eq!(target.spec.name, "Release Playbook");
            assert_eq!(target.referenced_workflows.len(), 1);
            assert_eq!(target.referencing_routines.len(), 1);
            assert_eq!(
                target.referencing_routines[0].prompt.as_deref(),
                Some("Follow the Release Playbook when CI passes")
            );
            Err(ProviderSessionError::Tool("review denied".into()))
        }));
    let tool = bridge
        .list_tools()
        .expect("tools")
        .into_iter()
        .find(|tool| tool.name == SAND_UPDATE_STATE_TOOL_NAME)
        .expect("update_state");
    let error = bridge.call_tool(
        &tool,
        json!({
            "target":"workflow",
            "action":"write",
            "id":"release-playbook",
            "name":"Release Playbook",
            "description":"Release steps",
            "body":"new body"
        }),
        "tool-workflow",
    ).expect_err("referenced workflow write must be reviewed");
    assert!(error.to_string().contains("review denied"));
    let _ = fs::remove_dir_all(root);
}
