use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};

use super::routed_provider_runtime::RoutedToolBridge;

pub const RUNNER_BOX_TOOL_PROVIDER: &str = "mahayana-box";
pub const RUNNER_BOX_SHELL_TOOL_NAME: &str = "Shell";
pub const RUNNER_BOX_READ_TOOL_NAME: &str = "Read";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerBoxShellRequest {
    pub command: String,
    pub working_directory: String,
    pub tool_call_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerBoxReadRequest {
    pub path: String,
    pub tool_call_id: String,
    pub offset: Option<i32>,
    pub limit: Option<u32>,
    pub encoding_hint: Option<String>,
}

/// Host-owned Box resource port consumed by the independent Runner.
///
/// Runner owns the model-facing tool names and schemas. Host supplies only
/// concrete resource execution; this prevents ForeverBox lifecycle concerns
/// from moving into the Runner process boundary.
pub trait RunnerBoxResourcePort: Send + Sync {
    fn execute_shell(
        &self,
        request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError>;

    fn execute_read(
        &self,
        request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError>;
}

pub fn runner_box_tool_definitions() -> Vec<RoutedToolDefinition> {
    vec![
        RoutedToolDefinition {
            name: RUNNER_BOX_SHELL_TOOL_NAME.into(),
            provider_identifier: RUNNER_BOX_TOOL_PROVIDER.into(),
            tool_name: RUNNER_BOX_SHELL_TOOL_NAME.into(),
            description: Some(
                "Run a shell command on your own box. This is the default shell surface for files and commands inside the isolated box."
                    .into(),
            ),
            input_schema: json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["command"],
                "properties": {
                    "command": {"type": "string"},
                    "workingDirectory": {
                        "type": "string",
                        "description": "Working directory inside the box; defaults to /workspace."
                    }
                }
            }),
        },
        RoutedToolDefinition {
            name: RUNNER_BOX_READ_TOOL_NAME.into(),
            provider_identifier: RUNNER_BOX_TOOL_PROVIDER.into(),
            tool_name: RUNNER_BOX_READ_TOOL_NAME.into(),
            description: Some(
                "Reads a file on your own computer (the box), the same filesystem Shell acts on. Text reads support offset/limit paging."
                    .into(),
            ),
            input_schema: json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": {"type": "string"},
                    "offset": {"type": "integer"},
                    "limit": {"type": "integer", "minimum": 0},
                    "encodingHint": {"type": "string"}
                }
            }),
        },
    ]
}

#[derive(Clone)]
pub struct RunnerBoxToolBridge {
    upstream: Arc<dyn RoutedToolBridge>,
    box_resources: Arc<dyn RunnerBoxResourcePort>,
}

impl RunnerBoxToolBridge {
    pub fn new(
        upstream: Arc<dyn RoutedToolBridge>,
        box_resources: Arc<dyn RunnerBoxResourcePort>,
    ) -> Self {
        Self {
            upstream,
            box_resources,
        }
    }

    fn is_box_tool(tool: &RoutedToolDefinition) -> bool {
        tool.provider_identifier == RUNNER_BOX_TOOL_PROVIDER
            && matches!(
                tool.name.as_str(),
                RUNNER_BOX_SHELL_TOOL_NAME | RUNNER_BOX_READ_TOOL_NAME
            )
    }

    fn parse_shell_request(
        args: &Value,
        tool_call_id: &str,
    ) -> Result<RunnerBoxShellRequest, ProviderSessionError> {
        let command = args
            .get("command")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProviderSessionError::Tool(
                    "Shell requires a non-empty command".into(),
                )
            })?;
        let working_directory = args
            .get("workingDirectory")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("/workspace");
        Ok(RunnerBoxShellRequest {
            command: command.into(),
            working_directory: working_directory.into(),
            tool_call_id: tool_call_id.into(),
        })
    }

    fn parse_read_request(
        args: &Value,
        tool_call_id: &str,
    ) -> Result<RunnerBoxReadRequest, ProviderSessionError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ProviderSessionError::Tool("Read requires a path".into()))?;
        let offset = match args.get("offset") {
            None | Some(Value::Null) => None,
            Some(value) => {
                let raw = value.as_i64().ok_or_else(|| {
                    ProviderSessionError::Tool("Read offset must be an integer".into())
                })?;
                Some(i32::try_from(raw).map_err(|_| {
                    ProviderSessionError::Tool("Read offset is out of range".into())
                })?)
            }
        };
        let limit = match args.get("limit") {
            None | Some(Value::Null) => None,
            Some(value) => {
                let raw = value.as_u64().ok_or_else(|| {
                    ProviderSessionError::Tool(
                        "Read limit must be a non-negative integer".into(),
                    )
                })?;
                Some(u32::try_from(raw).map_err(|_| {
                    ProviderSessionError::Tool("Read limit is out of range".into())
                })?)
            }
        };
        let encoding_hint = args
            .get("encodingHint")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        Ok(RunnerBoxReadRequest {
            path: path.into(),
            tool_call_id: tool_call_id.into(),
            offset,
            limit,
            encoding_hint,
        })
    }
}

impl RoutedToolBridge for RunnerBoxToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let mut tools = self.upstream.list_tools()?;
        let mut names = tools
            .iter()
            .map(|tool| tool.name.clone())
            .collect::<BTreeSet<_>>();
        for tool in runner_box_tool_definitions() {
            if !names.insert(tool.name.clone()) {
                return Err(ProviderSessionError::Protocol(format!(
                    "Host tool name collides with Runner box tool {}",
                    tool.name
                )));
            }
            tools.push(tool);
        }
        Ok(tools)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        if !Self::is_box_tool(tool) {
            return self.upstream.call_tool(tool, args, tool_call_id);
        }
        match tool.name.as_str() {
            RUNNER_BOX_SHELL_TOOL_NAME => self
                .box_resources
                .execute_shell(Self::parse_shell_request(&args, tool_call_id)?),
            RUNNER_BOX_READ_TOOL_NAME => self
                .box_resources
                .execute_read(Self::parse_read_request(&args, tool_call_id)?),
            _ => Err(ProviderSessionError::Tool(format!(
                "Unsupported Runner box tool {}",
                tool.name
            ))),
        }
    }
}
