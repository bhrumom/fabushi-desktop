use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_task_subagent_tool::{
    SAND_TASK_TOOL_NAME, SubagentLaunchRecord, SubagentTaskSink, SubagentTaskToolBridge,
};
use serde_json::{Value, json};

struct EmptyBridge;

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected delegate call".into()))
    }
}

#[derive(Default)]
struct RecordingTaskSink {
    calls: Mutex<Vec<(String, String, String)>>,
}

impl SubagentTaskSink for RecordingTaskSink {
    fn launch_subagent(
        &self,
        prompt: &str,
        subagent_type: &str,
        tool_call_id: &str,
    ) -> Result<SubagentLaunchRecord, ProviderSessionError> {
        self.calls.lock().unwrap().push((
            prompt.to_string(),
            subagent_type.to_string(),
            tool_call_id.to_string(),
        ));
        Ok(SubagentLaunchRecord {
            id: "child-agent".into(),
            subagent_type: subagent_type.to_string(),
        })
    }
}

#[test]
fn task_tool_dispatches_real_subagent_sink_contract() {
    let sink = Arc::new(RecordingTaskSink::default());
    let bridge = SubagentTaskToolBridge::new(Arc::new(EmptyBridge), sink.clone());
    let tools = bridge.list_tools().expect("task tool list");
    let task = tools
        .iter()
        .find(|tool| tool.name == SAND_TASK_TOOL_NAME)
        .expect("Task tool installed");
    let result = bridge
        .call_tool(
            task,
            json!({"prompt":"research this","subagent_type":"general-purpose"}),
            "tool-1",
        )
        .expect("Task launch");
    assert_eq!(result["subagent_id"], "child-agent");
    assert_eq!(result["status"], "running");
    assert_eq!(
        sink.calls.lock().unwrap().as_slice(),
        &[(
            "research this".to_string(),
            "general-purpose".to_string(),
            "tool-1".to_string(),
        )]
    );
}

#[test]
fn shipping_host_owns_task_child_runner_and_live_parent_projection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(root.join("app/src/main.rs")).expect("shipping host main");
    for required in [
        "struct ProductionSubagentTaskSink",
        "impl SubagentTaskSink for ProductionSubagentTaskSink",
        "materialize_new_session(Some(&profile), \"subagent\"",
        "start_routed_provider_task(",
        "\"parentAgentId\": self.parent_agent_id",
        ".with_subagent_task_sink(subagent_task_sink)",
        ".with_generated_agent_runtime(Arc::clone(&worker_generated_agent_runtime))",
        "runner.begin_generated_subagent(",
        "worker_transcript_runtime.begin_live_subagent(parent_agent_id)",
        "worker_completion_revivals.handle_background_subagent_completion",
        "worker_transcript_runtime.end_live_subagent(parent_agent_id)",
        "publish_generated_subagents(",
    ] {
        assert!(main.contains(required), "missing shipping wiring: {required}");
    }
    assert!(
        main.contains("if generated_parent_agent_id.is_none()"),
        "child sessions must not recursively install Task"
    );
}
