use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::transcript::client_side_tool_v2_producer::{
    ClientSideToolV2Producer, ClientSideToolV2TransportKind,
};
use mahayana_host_runtime::extensions::transcript::client_side_tool_v2_projection::{
    ProjectedClientSideToolV2, ToolProjectionPhase, project_generated_tool_case,
    project_routed_tool_call,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::turn_observation::{
    ObservedRoutedToolBridge, TurnObservation,
};
use serde_json::{Value, json};

fn tool(name: &str, provider: &str) -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: name.into(),
        provider_identifier: provider.into(),
        tool_name: name.into(),
        description: None,
        input_schema: json!({"type":"object"}),
    }
}

#[test]
fn frozen_shell_projection_preserves_typed_params_partial_and_terminal_semantics() {
    let args = json!({
        "command": "pwd",
        "workingDirectory": "/workspace",
        "is_background": false
    });
    let partial = project_routed_tool_call(
        ToolProjectionPhase::Partial,
        &tool("Shell", "mahayana-box"),
        &args,
        "call-shell",
        "model-1",
        None,
    )
    .expect("partial shell projection");
    match partial {
        ProjectedClientSideToolV2::Call {
            is_streaming,
            message,
            protobuf_bytes,
            ..
        } => {
            assert!(is_streaming);
            assert_eq!(message["tool"], 15);
            assert_eq!(message["modelCallId"], "model-1");
            assert_eq!(message["runTerminalCommandV2Params"]["command"], "pwd");
            assert_eq!(message["runTerminalCommandV2Params"]["cwd"], "/workspace");
            assert!(!protobuf_bytes.is_empty());
        }
        other => panic!("unexpected shell projection {other:?}"),
    }

    let completed = project_routed_tool_call(
        ToolProjectionPhase::Completed,
        &tool("Shell", "mahayana-box"),
        &args,
        "call-shell",
        "",
        Some(Ok(json!({
            "kind": "success",
            "stdout": "/workspace\n",
            "stderr": "",
            "exitCode": 0,
            "workingDirectory": "/workspace"
        }))),
    )
    .expect("completed shell projection");
    match completed {
        ProjectedClientSideToolV2::Result {
            message,
            protobuf_bytes,
            ..
        } => {
            assert_eq!(message["tool"], 15);
            assert_eq!(message["runTerminalCommandV2Result"]["endedReason"], 1);
            assert_eq!(message["runTerminalCommandV2Result"]["exitCodeV2"], 0);
            assert!(!protobuf_bytes.is_empty());
        }
        other => panic!("unexpected shell result {other:?}"),
    }
}

#[test]
fn read_and_mcp_projection_keep_frozen_fail_closed_and_struct_semantics() {
    let read = tool("Read", "mahayana-box");
    let non_text = project_routed_tool_call(
        ToolProjectionPhase::Completed,
        &read,
        &json!({"path":"/workspace/image.png"}),
        "read-1",
        "",
        Some(Ok(json!({
            "kind":"success",
            "output":{"kind":"data","data":[1,2,3]},
            "totalLines":0
        }))),
    )
    .expect("read projection");
    assert_eq!(
        non_text.message()["error"]["clientVisibleErrorMessage"],
        "Read completed with non-text content; bytes remain available in ordinary transcript transport"
    );

    let mcp = RoutedToolDefinition {
        name: "github.get_issue".into(),
        provider_identifier: "github".into(),
        tool_name: "get_issue".into(),
        description: None,
        input_schema: json!({"type":"object"}),
    };
    let call = project_routed_tool_call(
        ToolProjectionPhase::Started,
        &mcp,
        &json!({"issue_number":20}),
        "mcp-1",
        "",
        None,
    )
    .expect("mcp call projection");
    assert_eq!(call.message()["tool"], 49);
    assert_eq!(call.message()["callMcpToolParams"]["server"], "github");
    assert_eq!(call.message()["callMcpToolParams"]["toolName"], "get_issue");
    assert!(
        call.message()["callMcpToolParams"]["toolArgs"]["fields"]["issue_number"].is_object()
    );

    let result = project_routed_tool_call(
        ToolProjectionPhase::Completed,
        &mcp,
        &json!({"issue_number":20}),
        "mcp-1",
        "",
        Some(Ok(json!({"title":"parity","open":true}))),
    )
    .expect("mcp result projection");
    assert_eq!(result.message()["tool"], 49);
    assert!(
        result.message()["callMcpToolResult"]["result"]["fields"]["title"].is_object()
    );
}

#[test]
fn frozen_generated_cases_are_schema_backed_and_unknown_cases_fail_closed() {
    let web = project_generated_tool_case(
        ToolProjectionPhase::Started,
        "webSearchToolCall",
        "web-1",
        &json!({"searchTerm":"Grok 0.18"}),
        None,
        "",
        "",
    )
    .expect("web projection");
    assert_eq!(web.message()["tool"], 18);
    assert_eq!(web.message()["webSearchParams"]["searchTerm"], "Grok 0.18");
    assert!(!web.protobuf_bytes().is_empty());

    assert!(
        project_generated_tool_case(
            ToolProjectionPhase::Started,
            "sendMessageToolCall",
            "send-1",
            &json!({"message":"hello"}),
            None,
            "",
            "",
        )
        .is_none()
    );
}

struct FakeShell;

impl RoutedToolBridge for FakeShell {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![tool("Shell", "mahayana-box")])
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Ok(json!({
            "kind":"success",
            "stdout":"ok\n",
            "stderr":"",
            "exitCode":0,
            "workingDirectory":"/workspace"
        }))
    }
}

#[test]
fn shipping_observed_bridge_emits_call_then_result_into_host_producer_contract() {
    let seen = Arc::new(Mutex::new(Vec::<ProjectedClientSideToolV2>::new()));
    let observation = TurnObservation::shared("agent-1", None);
    {
        let seen = Arc::clone(&seen);
        observation
            .lock()
            .expect("observation")
            .set_client_side_tool_v2_handler(Arc::new(move |event| {
                seen.lock().expect("seen").push(event);
            }));
    }
    let bridge = ObservedRoutedToolBridge::new(Arc::new(FakeShell), observation);
    bridge
        .call_tool(
            &tool("Shell", "mahayana-box"),
            json!({"command":"printf ok","workingDirectory":"/workspace"}),
            "shipping-call-1",
        )
        .expect("shipping tool call");

    let events = seen.lock().expect("seen");
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], ProjectedClientSideToolV2::Call { .. }));
    assert!(matches!(events[1], ProjectedClientSideToolV2::Result { .. }));

    let mut producer = ClientSideToolV2Producer::with_epoch("contract-epoch");
    let first = producer
        .publish("agent-1", events[0].clone().into_produced_value())
        .expect("producer call");
    let second = producer
        .publish("agent-1", events[1].clone().into_produced_value())
        .expect("producer result");
    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(first.kind, ClientSideToolV2TransportKind::Call);
    assert_eq!(second.kind, ClientSideToolV2TransportKind::Result);
    assert_eq!(
        first.message.expect("call bytes").message_type,
        "aiserver.v1.ClientSideToolV2Call"
    );
    assert_eq!(
        second.message.expect("result bytes").message_type,
        "aiserver.v1.ClientSideToolV2Result"
    );
}
