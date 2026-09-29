use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_task_subagent_tool::{
    SAND_TASK_TOOL_NAME, SubagentLaunchRecord, SubagentTaskReviewCallback,
    SubagentTaskSink, SubagentTaskToolBridge,
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


#[test]
fn task_launch_review_receives_real_tool_call_id_and_can_fence_dispatch() {
    let sink = Arc::new(RecordingTaskSink::default());
    let review_calls = Arc::new(Mutex::new(Vec::<(String, String, String)>::new()));
    let calls = Arc::clone(&review_calls);
    let review: SubagentTaskReviewCallback = Arc::new(move |prompt, subagent_type, tool_call_id| {
        calls.lock().unwrap().push((
            prompt.to_string(),
            subagent_type.to_string(),
            tool_call_id.to_string(),
        ));
        Ok(Some("production review blocked launch".into()))
    });
    let bridge = SubagentTaskToolBridge::new(Arc::new(EmptyBridge), sink.clone())
        .with_review(review);
    let tools = bridge.list_tools().expect("task tool list");
    let task = tools
        .iter()
        .find(|tool| tool.name == SAND_TASK_TOOL_NAME)
        .expect("Task tool installed");
    let result = bridge
        .call_tool(
            task,
            json!({"prompt":"review me","subagent_type":"general-purpose"}),
            "task-review-call",
        )
        .expect("review denial is a tool result");
    assert_eq!(result, Value::String("production review blocked launch".into()));
    assert!(sink.calls.lock().unwrap().is_empty());
    assert_eq!(
        review_calls.lock().unwrap().as_slice(),
        &[(
            "review me".to_string(),
            "general-purpose".to_string(),
            "task-review-call".to_string(),
        )]
    );
}

#[test]
fn shipping_task_launch_is_bound_to_subagent_auto_review_before_dispatch() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(root.join("app/src/main.rs")).expect("shipping host main");
    let composition = fs::read_to_string(root.join("src/runner/turn_agent_composition.rs"))
        .expect("turn composition");
    let toolset = fs::read_to_string(root.join("src/runner/tools/turn_toolset.rs"))
        .expect("turn toolset");
    for required in [
        "build_sand_subagent_launch_review_target",
        "subagent_task_review",
        "review_sand_subagent_action(",
        ".with_subagent_task_review(subagent_task_review)",
    ] {
        assert!(main.contains(required), "missing launch review wiring: {required}");
    }
    assert!(composition.contains("SubagentTaskReviewCallback"));
    assert!(toolset.contains("task_bridge.with_review(review)"));
}
