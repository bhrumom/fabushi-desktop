use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::box_tool_access::{
    RUNNER_BOX_READ_TOOL_NAME, RUNNER_BOX_SHELL_TOOL_NAME, RUNNER_BOX_TOOL_PROVIDER,
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxToolBridge,
    RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::large_output_spill::{
    AGENT_TOOLS_DIR, MCP_TEXT_FILE_THRESHOLD_BYTES, is_large_output_spill_enabled,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use serde_json::{Value, json};

struct Upstream {
    collide: bool,
}

impl RoutedToolBridge for Upstream {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            origin: None,
            name: if self.collide { "Shell" } else { "SearchPlugins" }.into(),
            provider_identifier: "host".into(),
            tool_name: "search".into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(json!({"upstream":tool.name,"args":args}))
    }
}

#[derive(Default)]
struct BoxPort {
    shells: Mutex<Vec<RunnerBoxShellRequest>>,
    reads: Mutex<Vec<RunnerBoxReadRequest>>,
    writes: Mutex<Vec<RunnerBoxWriteRequest>>,
    failure: Option<ProviderSessionError>,
}

impl RunnerBoxResourcePort for BoxPort {
    fn execute_shell(
        &self,
        request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        self.shells.lock().expect("shell calls").push(request.clone());
        if let Some(error) = self.failure.as_ref() {
            return Err(error.clone());
        }
        Ok(json!({"kind":"success","command":request.command}))
    }

    fn execute_read(
        &self,
        request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        self.reads.lock().expect("read calls").push(request.clone());
        if let Some(error) = self.failure.as_ref() {
            return Err(error.clone());
        }
        Ok(json!({"kind":"success","path":request.path}))
    }

    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().expect("write calls").push(request);
        if let Some(error) = self.failure.as_ref() {
            return Err(error.clone());
        }
        Ok(())
    }
}

#[test]
fn runner_box_bridge_merges_frozen_shell_and_read_with_host_tools() {
    let box_port = Arc::new(BoxPort::default());
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    );

    let tools = bridge.list_tools().expect("tool list");
    assert!(tools.iter().any(|tool| tool.name == "SearchPlugins"));
    let shell = tools
        .iter()
        .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME)
        .expect("Shell definition");
    let read = tools
        .iter()
        .find(|tool| tool.name == RUNNER_BOX_READ_TOOL_NAME)
        .expect("Read definition");
    assert_eq!(shell.provider_identifier, RUNNER_BOX_TOOL_PROVIDER);
    assert_eq!(read.provider_identifier, RUNNER_BOX_TOOL_PROVIDER);
    assert_eq!(shell.input_schema["required"][0], "command");
    assert_eq!(read.input_schema["required"][0], "path");

    let shell_result = bridge
        .call_tool(
            shell,
            json!({"command":"pwd","workingDirectory":"/workspace/project"}),
            "shell-call",
        )
        .expect("Shell call");
    assert_eq!(shell_result["command"], "pwd");
    assert_eq!(
        box_port.shells.lock().expect("shell calls").as_slice(),
        &[RunnerBoxShellRequest {
            command: "pwd".into(),
            working_directory: "/workspace/project".into(),
            tool_call_id: "shell-call".into(),
            is_background: false,
            block_until_ms: None,
        }]
    );

    let read_result = bridge
        .call_tool(
            read,
            json!({"path":"/workspace/a.txt","offset":2,"limit":8,"encodingHint":"utf-8"}),
            "read-call",
        )
        .expect("Read call");
    assert_eq!(read_result["path"], "/workspace/a.txt");
    assert_eq!(
        box_port.reads.lock().expect("read calls").as_slice(),
        &[RunnerBoxReadRequest {
            path: "/workspace/a.txt".into(),
            tool_call_id: "read-call".into(),
            offset: Some(2),
            limit: Some(8),
            encoding_hint: Some("utf-8".into()),
        }]
    );
}

#[test]
fn runner_box_bridge_preserves_explicit_background_and_positive_block_until_semantics() {
    let box_port = Arc::new(BoxPort::default());
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    );
    let tools = bridge.list_tools().expect("tool list");
    let shell = tools
        .iter()
        .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME)
        .expect("Shell definition");

    assert_eq!(shell.input_schema["properties"]["is_background"]["type"], "boolean");
    assert_eq!(shell.input_schema["properties"]["block_until_ms"]["type"], "integer");
    assert_eq!(shell.input_schema["properties"]["block_until_ms"]["minimum"], 0);

    bridge
        .call_tool(
            shell,
            json!({
                "command":"sleep 10",
                "is_background":true,
                "block_until_ms":750
            }),
            "background-shell-call",
        )
        .expect("background Shell call");
    bridge
        .call_tool(
            shell,
            json!({
                "command":"sleep 10",
                "is_background":false,
                "block_until_ms":75
            }),
            "timed-shell-call",
        )
        .expect("timed Shell call");

    assert_eq!(
        box_port.shells.lock().expect("shell calls").as_slice(),
        &[
            RunnerBoxShellRequest {
                command: "sleep 10".into(),
                working_directory: "/workspace".into(),
                tool_call_id: "background-shell-call".into(),
                is_background: true,
                block_until_ms: Some(750),
            },
            RunnerBoxShellRequest {
                command: "sleep 10".into(),
                working_directory: "/workspace".into(),
                tool_call_id: "timed-shell-call".into(),
                is_background: false,
                block_until_ms: Some(75),
            },
        ]
    );
}

#[test]
fn runner_box_bridge_preserves_upstream_dispatch_and_fails_closed_on_name_collision() {
    let box_port = Arc::new(BoxPort::default());
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    );
    let upstream_tool = bridge
        .list_tools()
        .expect("tool list")
        .into_iter()
        .find(|tool| tool.name == "SearchPlugins")
        .expect("upstream tool");
    let upstream = bridge
        .call_tool(&upstream_tool, json!({"q":"calendar"}), "host-call")
        .expect("upstream dispatch");
    assert_eq!(upstream["upstream"], "SearchPlugins");
    assert!(box_port.shells.lock().expect("shell calls").is_empty());
    assert!(box_port.reads.lock().expect("read calls").is_empty());

    let colliding = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: true }),
        box_port,
    );
    let error = colliding
        .list_tools()
        .expect_err("Host tools must not shadow Runner box tools");
    assert!(error.to_string().contains("collides"));
}

#[test]
fn runner_box_bridge_validates_model_arguments_before_host_resource_execution() {
    let box_port = Arc::new(BoxPort::default());
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    );
    let tools = bridge.list_tools().expect("tool list");
    let shell = tools.iter().find(|tool| tool.name == "Shell").expect("Shell");
    let read = tools.iter().find(|tool| tool.name == "Read").expect("Read");

    assert!(bridge.call_tool(shell, json!({"command":"   "}), "bad").is_err());
    assert!(bridge.call_tool(read, json!({"path":""}), "bad").is_err());
    assert!(bridge.call_tool(read, json!({"path":"/x","limit":-1}), "bad").is_err());
    assert!(box_port.shells.lock().expect("shell calls").is_empty());
    assert!(box_port.reads.lock().expect("read calls").is_empty());
}

#[test]
fn runner_box_shell_review_fences_side_effect_before_resource_execution() {
    let box_port = Arc::new(BoxPort::default());
    let seen = Arc::new(Mutex::new(Vec::<RunnerBoxShellRequest>::new()));
    let seen_review = Arc::clone(&seen);
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    )
    .with_shell_review(Arc::new(move |request| {
        seen_review.lock().expect("review calls").push(request.clone());
        Ok(Some("Auto-review blocked shell".into()))
    }));
    let tools = bridge.list_tools().expect("tool list");
    let shell = tools
        .iter()
        .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME)
        .expect("Shell definition");
    let result = bridge
        .call_tool(
            shell,
            json!({"command":"curl example.com","workingDirectory":"/workspace"}),
            "shell-review-call",
        )
        .expect("review denial is a tool result");
    assert_eq!(result, Value::String("Auto-review blocked shell".into()));
    assert_eq!(seen.lock().expect("review calls").len(), 1);
    assert!(box_port.shells.lock().expect("shell calls").is_empty());
}

fn assert_box_failure_class_preserved(failure: ProviderSessionError) {
    let box_port = Arc::new(BoxPort {
        failure: Some(failure.clone()),
        ..BoxPort::default()
    });
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(Upstream { collide: false }),
        box_port.clone(),
    );
    let tools = bridge.list_tools().expect("tool list");
    for (tool_name, args, tool_call_id) in [
        (
            RUNNER_BOX_SHELL_TOOL_NAME,
            json!({"command":"effect-once"}),
            "foreground-failure",
        ),
        (
            RUNNER_BOX_SHELL_TOOL_NAME,
            json!({"command":"effect-once","block_until_ms":75}),
            "timed-failure",
        ),
        (
            RUNNER_BOX_SHELL_TOOL_NAME,
            json!({"command":"effect-once","is_background":true}),
            "background-failure",
        ),
        (
            RUNNER_BOX_READ_TOOL_NAME,
            json!({"path":"/workspace/result.txt"}),
            "read-failure",
        ),
    ] {
        let tool = tools
            .iter()
            .find(|tool| tool.name == tool_name)
            .expect("Box tool");
        let error = bridge
            .call_tool(tool, args, tool_call_id)
            .expect_err("Box failure must remain an error through the real Runner bridge");
        match (&failure, &error) {
            (
                ProviderSessionError::ToolOutcomeUnknown(expected),
                ProviderSessionError::ToolOutcomeUnknown(actual),
            )
            | (
                ProviderSessionError::Tool(expected),
                ProviderSessionError::Tool(actual),
            ) => assert_eq!(actual, expected),
            _ => panic!("Box error class changed: expected {failure:?}, got {error:?}"),
        }
    }
    let shells = box_port.shells.lock().expect("shell calls");
    assert_eq!(shells.len(), 3, "bridge must not replay a failed Shell call");
    assert_eq!(shells[0].tool_call_id, "foreground-failure");
    assert!(!shells[0].is_background);
    assert_eq!(shells[0].block_until_ms, None);
    assert_eq!(shells[1].tool_call_id, "timed-failure");
    assert_eq!(shells[1].block_until_ms, Some(75));
    assert_eq!(shells[2].tool_call_id, "background-failure");
    assert!(shells[2].is_background);
    let reads = box_port.reads.lock().expect("read calls");
    assert_eq!(reads.len(), 1, "bridge must not replay a failed Read call");
    assert_eq!(reads[0].tool_call_id, "read-failure");
    assert!(box_port.writes.lock().expect("write calls").is_empty());
}

#[test]
fn runner_box_bridge_preserves_unknown_box_execution_outcomes() {
    assert_box_failure_class_preserved(ProviderSessionError::ToolOutcomeUnknown(
        "same Box failure detail".into(),
    ));
}

#[test]
fn runner_box_bridge_preserves_known_box_refusals() {
    assert_box_failure_class_preserved(ProviderSessionError::Tool(
        "same Box failure detail".into(),
    ));
}

struct SpillUpstream {
    calls: Mutex<Vec<String>>,
    result: Value,
}

impl RoutedToolBridge for SpillUpstream {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            origin: None,
            name: "McpSpillSource".into(),
            provider_identifier: "mcp-spill-contract".into(),
            tool_name: "read_large_text".into(),
            description: None,
            input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        assert_eq!(tool.tool_name, "read_large_text");
        assert_eq!(args, json!({"input":"once"}));
        self.calls.lock().expect("upstream calls").push(tool_call_id.into());
        Ok(self.result.clone())
    }
}

fn assert_mcp_spill_failure_preserved(failure: ProviderSessionError) {
    assert!(
        is_large_output_spill_enabled(),
        "this contract requires the shipping MCP spill path to be enabled",
    );
    let text = "x".repeat(MCP_TEXT_FILE_THRESHOLD_BYTES + 1);
    let inline = json!({
        "isError":false,
        "content":[{"type":"text","text":text.clone()}],
        "structuredContent":{"upstream":"completed"},
    });
    let upstream = Arc::new(SpillUpstream {
        calls: Mutex::new(Vec::new()),
        result: inline.clone(),
    });
    let box_port = Arc::new(BoxPort {
        failure: Some(failure.clone()),
        ..BoxPort::default()
    });
    let bridge = RunnerBoxToolBridge::new(upstream.clone(), box_port.clone());
    let tool = bridge
        .list_tools()
        .expect("tool list")
        .into_iter()
        .find(|tool| tool.name == "McpSpillSource")
        .expect("upstream MCP tool");
    let result = bridge.call_tool(&tool, json!({"input":"once"}), "mcp-spill-call");
    match (&failure, result) {
        (
            ProviderSessionError::ToolOutcomeUnknown(expected),
            Err(ProviderSessionError::ToolOutcomeUnknown(actual)),
        ) => assert_eq!(&actual, expected),
        (ProviderSessionError::Tool(_), Ok(actual)) => {
            assert_eq!(actual, inline, "known refusal must retain the completed inline result");
            assert!(actual["content"][0].get("outputLocation").is_none());
        }
        (expected, actual) => panic!("spill changed execution disposition: {expected:?} -> {actual:?}"),
    }
    assert_eq!(
        upstream.calls.lock().expect("upstream calls").as_slice(),
        &["mcp-spill-call".to_string()],
        "spill failure must not replay the completed upstream operation",
    );
    let writes = box_port.writes.lock().expect("write calls");
    assert_eq!(writes.len(), 1, "spill failure must not issue another write");
    assert_eq!(writes[0].tool_call_id, "mcp-spill-call");
    assert_eq!(writes[0].data, text.as_bytes());
    assert!(writes[0].path.starts_with(&format!("{AGENT_TOOLS_DIR}/")));
    assert!(writes[0].path.ends_with(".txt"));
    assert!(
        box_port.shells.lock().expect("shell calls").is_empty(),
        "an unconfirmed spill must not trigger Shell cleanup",
    );
    assert!(
        box_port.reads.lock().expect("read calls").is_empty(),
        "an unconfirmed spill must not issue a recovery Read",
    );
}

#[test]
fn runner_box_bridge_preserves_unknown_mcp_spill_write_without_replay() {
    assert_mcp_spill_failure_preserved(ProviderSessionError::ToolOutcomeUnknown(
        "same spill write failure detail".into(),
    ));
}

#[test]
fn runner_box_bridge_keeps_known_mcp_spill_refusal_inline_without_replay() {
    assert_mcp_spill_failure_preserved(ProviderSessionError::Tool(
        "same spill write failure detail".into(),
    ));
}
