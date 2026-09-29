use std::sync::Arc;

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{ProviderSessionError, RoutedToolDefinition};
use crate::runner::routed_provider_runtime::RoutedToolBridge;

pub const SAND_TASK_TOOL_NAME: &str = "Task";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentLaunchRecord {
    pub id: String,
    pub subagent_type: String,
}

pub trait SubagentTaskSink: Send + Sync {
    fn launch_subagent(
        &self,
        prompt: &str,
        subagent_type: &str,
        tool_call_id: &str,
    ) -> Result<SubagentLaunchRecord, ProviderSessionError>;
}

pub struct SubagentTaskToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    sink: Arc<dyn SubagentTaskSink>,
}

impl SubagentTaskToolBridge {
    pub fn new(delegate: Arc<dyn RoutedToolBridge>, sink: Arc<dyn SubagentTaskSink>) -> Self {
        Self { delegate, sink }
    }
}

fn task_definition() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: SAND_TASK_TOOL_NAME.into(),
        provider_identifier: "fabushi-runner".into(),
        tool_name: SAND_TASK_TOOL_NAME.into(),
        description: Some(
            "Delegate a self-contained task to a background subagent. The child runs asynchronously through the same shipping Runner boundary and the parent is revived separately when it settles."
                .into(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["prompt"],
            "additionalProperties": false,
            "properties": {
                "prompt": {"type":"string","minLength":1},
                "subagent_type": {"type":"string","minLength":1}
            }
        }),
    }
}

impl RoutedToolBridge for SubagentTaskToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let mut tools = self.delegate.list_tools()?;
        tools.retain(|tool| {
            !tool.name.eq_ignore_ascii_case(SAND_TASK_TOOL_NAME)
                && !tool.tool_name.eq_ignore_ascii_case(SAND_TASK_TOOL_NAME)
        });
        tools.insert(0, task_definition());
        Ok(tools)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        if !tool.name.eq_ignore_ascii_case(SAND_TASK_TOOL_NAME)
            && !tool.tool_name.eq_ignore_ascii_case(SAND_TASK_TOOL_NAME)
        {
            return self.delegate.call_tool(tool, args, tool_call_id);
        }
        let prompt = args
            .get("prompt")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ProviderSessionError::Tool("Task requires prompt".into()))?;
        let subagent_type = args
            .get("subagent_type")
            .or_else(|| args.get("subagentType"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("general-purpose");
        let launched = self
            .sink
            .launch_subagent(prompt, subagent_type, tool_call_id)?;
        Ok(json!({
            "subagent_id": launched.id,
            "subagent_type": launched.subagent_type,
            "status": "running"
        }))
    }
}
