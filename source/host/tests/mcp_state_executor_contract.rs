use mahayana_host_runtime::extensions::inference::provider_session::RoutedToolDefinition;
use mahayana_host_runtime::ports::mcp_state_executor::{
    McpStateExecResult, SandMcpToolProvider, execute_mcp_state,
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
