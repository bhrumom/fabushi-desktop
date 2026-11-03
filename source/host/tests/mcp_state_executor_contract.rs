use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderPartialToolCall, ProviderSessionError, RoutedMcpOrigin, RoutedToolDefinition,
};
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, CoordinatorToolRelayError,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::turn_toolset::project_turn_mcp_toolset;
use mahayana_host_runtime::ports::mcp_state_executor::{
    McpStateExecResult, SandMcpToolProvider, decode_canonical_mcp_state_result,
    encode_canonical_mcp_state_result, execute_mcp_state,
};
use serde_json::json;

struct Provider {
    tools: Vec<RoutedToolDefinition>,
    fail: bool,
}

impl SandMcpToolProvider for Provider {
    fn get_tools(&self) -> Result<Vec<RoutedToolDefinition>, String> {
        if self.fail {
            Err("unavailable".into())
        } else {
            Ok(self.tools.clone())
        }
    }
}

fn tool(provider: &str, name: &str, tool_name: &str) -> RoutedToolDefinition {
    RoutedToolDefinition {
        origin: None,
        name: name.into(),
        provider_identifier: provider.into(),
        tool_name: tool_name.into(),
        description: Some(format!("{name} description")),
        input_schema: json!({"type":"object","properties":{"q":{"type":"string"}}}),
    }
}

#[test]
fn groups_tools_by_provider_in_first_seen_order_and_marks_connected() {
    let provider = Provider {
        tools: vec![
            tool("github", "github_search", "search"),
            tool("linear", "linear_get", "get"),
            tool("github", "github_issue", "issue"),
        ],
        fail: false,
    };

    let McpStateExecResult::Success(result) = execute_mcp_state(&provider).unwrap();
    assert_eq!(result.servers.len(), 2);

    assert_eq!(result.servers[0].server_identifier, "github");
    assert_eq!(result.servers[0].server_name, "github");
    assert_eq!(result.servers[0].status, "connected");
    assert_eq!(result.servers[0].tools.len(), 2);
    assert_eq!(result.servers[0].tools[0].name, "github_search");
    assert_eq!(result.servers[0].tools[0].tool_name, "search");
    assert_eq!(
        result.servers[0].tools[0].input_schema["properties"]["q"]["type"],
        "string"
    );

    assert_eq!(result.servers[1].server_identifier, "linear");
    assert_eq!(result.servers[1].tools.len(), 1);
}

#[test]
fn empty_provider_is_success_and_provider_failure_is_not_hidden() {
    let empty = Provider { tools: Vec::new(), fail: false };
    let McpStateExecResult::Success(result) = execute_mcp_state(&empty).unwrap();
    assert!(result.servers.is_empty());

    let failed = Provider { tools: Vec::new(), fail: true };
    assert_eq!(execute_mcp_state(&failed).unwrap_err(), "unavailable");
}


#[test]
fn canonical_agent_v1_wire_round_trip_preserves_grouping_and_tool_schema() {
    let provider = Provider {
        tools: vec![
            tool("github", "github_search", "search"),
            tool("linear", "linear_get", "get"),
            tool("github", "github_issue", "issue"),
        ],
        fail: false,
    };
    let projected = execute_mcp_state(&provider).expect("project state");
    let bytes = encode_canonical_mcp_state_result(&projected).expect("encode canonical protobuf");
    let decoded = decode_canonical_mcp_state_result(&bytes).expect("decode canonical protobuf");
    assert_eq!(decoded, projected);
}

#[test]
fn canonical_mcp_state_preserves_origin_in_process_and_never_serializes_relay_authority() {
    let events = GatewayEventHub::default();
    let published = events.subscribe();
    let relay = CoordinatorToolRelay::new(events.clone());
    let foreign_relay = CoordinatorToolRelay::new(events);

    let mut repeated = tool("z-provider", "  repeated-public  ", "  raw-name  ");
    repeated.description = Some("  literal description  ".into());
    // Single-entry maps keep byte comparison independent of protobuf map order.
    repeated.input_schema = json!({"allOf": [
        {"type": "object"},
        {"properties": {"q": {"type": "string"}}},
        {"additionalProperties": false},
        {"default": {"nested": [null, true, 1.25, {"mode": "literal"}]}},
    ]});
    let mut other = tool("a-provider", "public-other", "");
    other.description = None;
    other.input_schema = json!([{"type": "string"}, false, null, 2.5]);
    let metadata_free_tools = vec![repeated.clone(), other, repeated];
    let bindings: Vec<_> = (0..metadata_free_tools.len())
        .map(|index| {
            let origin = RoutedMcpOrigin::from_wire(&json!({
                "version": 1,
                "coordinatorSourceId": "1b42cc41-58cd-46a0-aa38-20dfcd52b5d7",
                "mcpAttemptId": format!("cb62c476-c3f1-4d22-a6e4-{:012x}", index + 1),
            }))
            .expect("parse canonical native origin");
            relay.bind_tool_origin(origin).expect("bind to current relay")
        })
        .collect();
    assert_ne!(bindings[0], bindings[2], "duplicate public rows retain different origins");
    let bound_tools = metadata_free_tools
        .iter()
        .cloned()
        .zip(&bindings)
        .map(|(mut tool, binding)| {
            tool.origin = Some(binding.clone());
            tool
        })
        .collect();
    let projected = execute_mcp_state(&Provider { tools: bound_tools, fail: false })
        .expect("project real relay bindings");
    let McpStateExecResult::Success(state) = &projected;
    assert_eq!(state.servers.len(), 2);
    assert_eq!(state.servers[0].server_identifier, "z-provider");
    assert_eq!(state.servers[1].server_identifier, "a-provider");
    assert_eq!(state.servers[0].tools.len(), 2);
    assert_eq!(state.servers[1].tools.len(), 1);
    for server in &state.servers {
        assert_eq!(server.server_name, server.server_identifier);
        assert_eq!(server.status, "connected");
        assert_eq!(server.error_message, None);
    }
    let grouped_rows: Vec<_> = state.servers.iter().flat_map(|server| &server.tools).collect();
    for (row, original_index) in grouped_rows.iter().zip([0, 2, 1]) {
        let expected = &metadata_free_tools[original_index];
        assert_eq!(row.name, expected.name);
        assert_eq!(row.provider_identifier, expected.provider_identifier);
        assert_eq!(row.tool_name, expected.tool_name);
        assert_eq!(row.description, expected.description);
        assert_eq!(row.input_schema, expected.input_schema);
        assert_eq!(row.origin.as_ref(), Some(&bindings[original_index]));
        relay.validate_tool_origin(row.origin.as_ref()).expect("current row remains executable");
        let foreign = foreign_relay.validate_tool_origin(row.origin.as_ref())
            .expect_err("another relay cannot borrow this row's binding");
        assert!(!foreign.outcome_unknown(), "foreign binding refuses before publication");
        assert!(matches!(foreign.into_error(), CoordinatorToolRelayError::Protocol(_)));
    }

    let metadata_free = execute_mcp_state(&Provider { tools: metadata_free_tools, fail: false })
        .expect("project corresponding public fields without metadata");
    let bytes = encode_canonical_mcp_state_result(&projected).expect("encode bound projection");
    let public_bytes = encode_canonical_mcp_state_result(&metadata_free)
        .expect("encode metadata-free projection");
    assert_eq!(bytes, public_bytes, "canonical bytes must not expose relay authority or native origin");
    let decoded = decode_canonical_mcp_state_result(&bytes).expect("decode external protobuf");
    assert_eq!(decoded, metadata_free, "wire preserves every public field, schema and row order");
    assert_ne!(decoded, projected, "wire decode must not recreate the in-process bindings");
    let McpStateExecResult::Success(decoded_state) = &decoded;
    for row in decoded_state.servers.iter().flat_map(|server| &server.tools) {
        assert!(row.origin.is_none());
        let unbound = relay.validate_tool_origin(row.origin.as_ref())
            .expect_err("decoded wire rows have no local execution authority");
        assert!(!unbound.outcome_unknown(), "unbound wire row refuses before publication");
        assert!(matches!(unbound.into_error(), CoordinatorToolRelayError::Protocol(_)));
    }

    let foreign_binding = foreign_relay.bind_tool_origin(bindings[0].origin().clone())
        .expect("foreign relay can bind the same wire identity for itself");
    assert_eq!(foreign_binding.origin(), bindings[0].origin());
    assert_ne!(foreign_binding, bindings[0], "wire identity is not the private relay binding");
    foreign_relay.validate_tool_origin(Some(&foreign_binding))
        .expect("foreign relay positive control is live");
    assert_eq!(relay.pending_count(), 0);
    assert_eq!(foreign_relay.pending_count(), 0);
    assert!(matches!(published.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
}

struct RecordingBridge {
    tools: Vec<RoutedToolDefinition>,
    observed: Arc<Mutex<Vec<String>>>,
    called: Arc<Mutex<Vec<String>>>,
}

impl RoutedToolBridge for RecordingBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(self.tools.clone())
    }

    fn observe_partial_tool_call(
        &self,
        partial: &ProviderPartialToolCall,
    ) -> Result<(), ProviderSessionError> {
        self.observed
            .lock()
            .expect("record partial")
            .push(partial.tool_call_id.clone());
        Ok(())
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        _args: serde_json::Value,
        tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        self.called
            .lock()
            .expect("record call")
            .push(format!("{}:{tool_call_id}", tool.name));
        Ok(json!({"ok": true}))
    }
}

#[test]
fn shipping_projection_bridge_routes_discovery_through_mcp_state_and_delegates_execution() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let called = Arc::new(Mutex::new(Vec::new()));
    let first = tool("github", "github_search", "search");
    let second = tool("linear", "linear_get", "get");
    let bridge: Arc<dyn RoutedToolBridge> = Arc::new(RecordingBridge {
        tools: vec![first.clone(), second.clone()],
        observed: Arc::clone(&observed),
        called: Arc::clone(&called),
    });
    let projected = project_turn_mcp_toolset(bridge, None, None, false)
        .expect("project turn MCP toolset");

    let tools = projected.list_tools().expect("projected tools");
    assert_eq!(tools, vec![first.clone(), second.clone()]);

    projected
        .observe_partial_tool_call(&ProviderPartialToolCall {
            tool: first.clone(),
            tool_call_id: "partial-1".into(),
            raw_arguments: "{".into(),
            model_call_id: Some("model-1".into()),
        })
        .expect("partial observation");
    assert_eq!(
        observed.lock().expect("observed lock").as_slice(),
        ["partial-1"]
    );

    let result = projected
        .call_tool(&second, json!({"q":"status"}), "call-2")
        .expect("delegated tool call");
    assert_eq!(result, json!({"ok": true}));
    assert_eq!(
        called.lock().expect("called lock").as_slice(),
        ["linear_get:call-2"]
    );
}

