use serde_json::Value;

use crate::extensions::inference::provider_session::RoutedToolDefinition;

#[derive(Debug, Clone, PartialEq)]
pub struct McpStateToolDefinition {
    pub name: String,
    pub provider_identifier: String,
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct McpStateServer {
    pub server_identifier: String,
    pub server_name: String,
    pub status: String,
    pub tools: Vec<McpStateToolDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct McpStateSuccess {
    pub servers: Vec<McpStateServer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpStateExecResult {
    Success(McpStateSuccess),
}

pub trait SandMcpToolProvider {
    fn get_tools(&self) -> Result<Vec<RoutedToolDefinition>, String>;
}

pub fn execute_mcp_state(
    provider: &dyn SandMcpToolProvider,
) -> Result<McpStateExecResult, String> {
    let mut servers = Vec::<McpStateServer>::new();

    for tool in provider.get_tools()? {
        let index = servers
            .iter()
            .position(|server| server.server_identifier == tool.provider_identifier);

        let mapped = McpStateToolDefinition {
            name: tool.name,
            provider_identifier: tool.provider_identifier.clone(),
            tool_name: tool.tool_name,
            description: tool.description,
            input_schema: tool.input_schema,
        };

        if let Some(index) = index {
            servers[index].tools.push(mapped);
        } else {
            servers.push(McpStateServer {
                server_identifier: tool.provider_identifier.clone(),
                server_name: tool.provider_identifier,
                status: "connected".into(),
                tools: vec![mapped],
            });
        }
    }

    Ok(McpStateExecResult::Success(McpStateSuccess { servers }))
}
