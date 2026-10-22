use mahayana_node_agent_coordinator::protocol::Failure;
use mahayana_node_agent_coordinator::runner_tool_relay::{
    LOCAL_BROWSER_ACTION_METHOD, ROUTED_TOOL_EXECUTE_METHOD, ROUTED_TOOL_LIST_METHOD,
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
    execute_runner_tool_request, parse_runner_tool_request,
};
use serde_json::json;

#[test]
fn runner_tool_relay_accepts_only_the_frozen_electron_main_commands() {
    assert_eq!(RUNNER_TOOL_REQUEST_EVENT_CHANNEL, "runner-tool-request");
    assert_eq!(
        RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD,
        "runner.resolveRoutedToolRequest"
    );

    let list = parse_runner_tool_request(&json!({
        "requestId": "runner-tool-1",
        "method": ROUTED_TOOL_LIST_METHOD,
        "args": {},
    }))
    .expect("list request");
    assert_eq!(list.request_id, "runner-tool-1");
    assert_eq!(list.method, ROUTED_TOOL_LIST_METHOD);

    let execute = parse_runner_tool_request(&json!({
        "requestId": "runner-tool-2",
        "method": ROUTED_TOOL_EXECUTE_METHOD,
        "args": {"providerIdentifier":"github","toolName":"search","args":{"q":"x"}},
    }))
    .expect("execute request");
    assert_eq!(execute.method, ROUTED_TOOL_EXECUTE_METHOD);

    let local_browser = parse_runner_tool_request(&json!({
        "requestId": "runner-tool-local",
        "method": LOCAL_BROWSER_ACTION_METHOD,
        "args": {
            "type":"snapshot",
            "conversationId":"agent-a",
            "workspaceId":"workspace-a",
            "tabId":"tab-a",
            "expectedGeneration":2,
            "operationId":"tool-call-a"
        },
    }))
    .expect("local browser action request");
    assert_eq!(local_browser.method, LOCAL_BROWSER_ACTION_METHOD);
    assert_eq!(local_browser.args["type"], "snapshot");

    let denied = parse_runner_tool_request(&json!({
        "requestId": "runner-tool-3",
        "method": "spawnLocalExecDaemon",
        "args": {},
    }))
    .expect_err("unapproved command must be denied");
    assert_eq!(denied.code, "RUNNER_TOOL_RELAY_METHOD_DENIED");

    let malformed = parse_runner_tool_request(&json!({
        "requestId": "runner-tool-4",
        "method": ROUTED_TOOL_LIST_METHOD,
        "args": [],
    }))
    .expect_err("non-object args must fail");
    assert_eq!(malformed.code, "RUNNER_TOOL_RELAY_PROTOCOL_ERROR");
}

#[test]
fn runner_tool_relay_executes_via_control_boundary_and_builds_host_resolution() {
    let success = execute_runner_tool_request(
        &json!({
            "requestId": "runner-tool-7",
            "method": ROUTED_TOOL_EXECUTE_METHOD,
            "args": {"toolName":"search","args":{"q":"dharma"}},
        }),
        |method, args| {
            assert_eq!(method, ROUTED_TOOL_EXECUTE_METHOD);
            assert_eq!(args["toolName"], "search");
            Ok(json!({"content":[{"type":"text","text":"ok"}]}))
        },
    )
    .expect("valid request");
    assert_eq!(success["requestId"], "runner-tool-7");
    assert_eq!(success["ok"], true);
    assert_eq!(success["result"]["content"][0]["text"], "ok");

    let failed = execute_runner_tool_request(
        &json!({
            "requestId": "runner-tool-8",
            "method": ROUTED_TOOL_LIST_METHOD,
            "args": {},
        }),
        |_method, _args| Err(Failure::new("MCP_OFFLINE", "connector transport is down")),
    )
    .expect("valid request with failed backend");
    assert_eq!(failed["requestId"], "runner-tool-8");
    assert_eq!(failed["ok"], false);
    assert_eq!(
        failed["error"],
        "MCP_OFFLINE: connector transport is down"
    );
}
