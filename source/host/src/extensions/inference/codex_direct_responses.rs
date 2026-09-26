use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CodexDirectUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

impl CodexDirectUsage {
    fn add(self, next: Self) -> Self {
        Self {
            input_tokens: self.input_tokens.saturating_add(next.input_tokens),
            output_tokens: self.output_tokens.saturating_add(next.output_tokens),
            cache_read_tokens: self.cache_read_tokens.saturating_add(next.cache_read_tokens),
            cache_write_tokens: self.cache_write_tokens.saturating_add(next.cache_write_tokens),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodexDirectTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
    pub source: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodexDirectOptions {
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub instructions: String,
    pub input: Vec<Value>,
    pub tools: Vec<CodexDirectTool>,
    pub max_steps: usize,
}

impl CodexDirectOptions {
    pub fn new(
        model: impl Into<String>,
        instructions: impl Into<String>,
        input: Vec<Value>,
    ) -> Self {
        Self {
            model: model.into(),
            reasoning_effort: None,
            instructions: instructions.into(),
            input,
            tools: Vec::new(),
            max_steps: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodexDirectResult {
    pub text: String,
    pub response_id: String,
    pub usage: CodexDirectUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexDirectCheckpoint {
    pub input: Vec<Value>,
    pub text: String,
    pub response_id: String,
    pub usage: CodexDirectUsage,
    pub completed_steps: usize,
    pub tool_calls_completed: usize,
}

#[derive(Debug, Error)]
pub enum CodexDirectError {
    #[error("Codex direct transport failed: {0}")]
    Transport(String),
    #[error("Codex direct protocol failed: {0}")]
    Protocol(String),
    #[error("Codex direct tool failed: {0}")]
    Tool(String),
    #[error("Codex direct request cancelled: {0}")]
    Cancelled(String),
}

pub trait CodexDirectTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), CodexDirectError>;
}

fn usage_of(response: &Value) -> CodexDirectUsage {
    let usage = response.get("usage").and_then(Value::as_object);
    let details = usage
        .and_then(|usage| usage.get("input_tokens_details"))
        .and_then(Value::as_object);
    CodexDirectUsage {
        input_tokens: usage
            .and_then(|usage| usage.get("input_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output_tokens: usage
            .and_then(|usage| usage.get("output_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cache_read_tokens: details
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cache_write_tokens: 0,
    }
}

fn request_tools(tools: &[CodexDirectTool]) -> Option<Vec<Value>> {
    if tools.is_empty() {
        return None;
    }
    Some(
        tools
            .iter()
            .map(|tool| {
                let mut value = json!({
                    "type": "function",
                    "name": tool.name,
                    "parameters": tool.parameters,
                    "strict": false,
                });
                if let Some(description) = tool.description.as_deref() {
                    value["description"] = Value::String(description.to_string());
                }
                value
            })
            .collect(),
    )
}

fn safe_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        json!({
            "isError": true,
            "error": format!("failed to serialize tool output: {error}")
        })
        .to_string()
    })
}

fn output_items(completed: &Value, observed: &[Value]) -> Vec<Value> {
    completed
        .get("output")
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty())
        .cloned()
        .unwrap_or_else(|| observed.to_vec())
}

pub fn run_codex_direct_responses(
    transport: &mut dyn CodexDirectTransport,
    options: &CodexDirectOptions,
    execute_tool: &mut dyn FnMut(
        &CodexDirectTool,
        Value,
        &str,
    ) -> Result<Value, CodexDirectError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
) -> Result<CodexDirectResult, CodexDirectError> {
    let mut ignore_checkpoint =
        |_checkpoint: &CodexDirectCheckpoint| Ok(());
    run_codex_direct_responses_with_lifecycle(
        transport,
        options,
        None,
        execute_tool,
        on_text_delta,
        &mut ignore_checkpoint,
        &|| false,
    )
}

pub fn run_codex_direct_responses_with_cancel(
    transport: &mut dyn CodexDirectTransport,
    options: &CodexDirectOptions,
    execute_tool: &mut dyn FnMut(
        &CodexDirectTool,
        Value,
        &str,
    ) -> Result<Value, CodexDirectError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
) -> Result<CodexDirectResult, CodexDirectError> {
    let mut ignore_checkpoint =
        |_checkpoint: &CodexDirectCheckpoint| Ok(());
    run_codex_direct_responses_with_lifecycle(
        transport,
        options,
        None,
        execute_tool,
        on_text_delta,
        &mut ignore_checkpoint,
        should_cancel,
    )
}

pub fn run_codex_direct_responses_with_lifecycle(
    transport: &mut dyn CodexDirectTransport,
    options: &CodexDirectOptions,
    resume_from: Option<&CodexDirectCheckpoint>,
    execute_tool: &mut dyn FnMut(
        &CodexDirectTool,
        Value,
        &str,
    ) -> Result<Value, CodexDirectError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), CodexDirectError>,
    should_cancel: &dyn Fn() -> bool,
) -> Result<CodexDirectResult, CodexDirectError> {
    let max_steps = options.max_steps.max(1);
    let tools_by_name = options
        .tools
        .iter()
        .map(|tool| (tool.name.as_str(), tool))
        .collect::<HashMap<_, _>>();
    let declared_tools = request_tools(&options.tools);
    let (
        mut input,
        mut text,
        mut response_id,
        mut total_usage,
        mut completed_steps,
        mut tool_calls_completed,
    ) = match resume_from {
        Some(checkpoint) => (
            checkpoint.input.clone(),
            checkpoint.text.clone(),
            checkpoint.response_id.clone(),
            checkpoint.usage,
            checkpoint.completed_steps,
            checkpoint.tool_calls_completed,
        ),
        None => (
            options.input.clone(),
            String::new(),
            String::new(),
            CodexDirectUsage::default(),
            0,
            0,
        ),
    };

    if completed_steps >= max_steps {
        return Err(CodexDirectError::Protocol(format!(
            "provider resume checkpoint already exhausted Fabushi's {max_steps}-step tool limit"
        )));
    }

    for _step in completed_steps..max_steps {
        if should_cancel() {
            return Err(CodexDirectError::Cancelled(
                "Runner cancelled before provider dispatch".into(),
            ));
        }
        let mut request = json!({
            "model": options.model,
            "instructions": options.instructions,
            "input": input,
            "include": ["reasoning.encrypted_content"],
            "stream": true,
            "store": false,
        });
        if let Some(tools) = declared_tools.as_ref() {
            request["tools"] = Value::Array(tools.clone());
            request["tool_choice"] = Value::String("auto".into());
            request["parallel_tool_calls"] = Value::Bool(true);
        }
        if let Some(effort) = options.reasoning_effort.as_deref() {
            request["reasoning"] = json!({ "effort": effort, "summary": "auto" });
        }

        let mut completed: Option<Value> = None;
        let mut observed_output = Vec::new();
        transport.stream_response(&request, &mut |event| {
            if should_cancel() {
                return Err(CodexDirectError::Cancelled(
                    "Runner cancelled the provider stream".into(),
                ));
            }
            match event.get("type").and_then(Value::as_str) {
                Some("response.output_text.delta") => {
                    if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                        text.push_str(delta);
                        on_text_delta(delta, &text);
                    }
                }
                Some("response.output_item.done") => {
                    if let Some(item) = event.get("item").filter(|value| value.is_object()) {
                        observed_output.push(item.clone());
                    }
                }
                Some("response.completed") => {
                    completed = event.get("response").filter(|value| value.is_object()).cloned();
                }
                Some("response.failed") | Some("error") => {
                    let failure = event
                        .get("response")
                        .and_then(|value| value.get("error"))
                        .or_else(|| event.get("error"))
                        .cloned()
                        .unwrap_or(event);
                    return Err(CodexDirectError::Protocol(format!(
                        "provider returned failure: {}",
                        safe_json(&failure).chars().take(4096).collect::<String>()
                    )));
                }
                _ => {}
            }
            Ok(())
        }, should_cancel)?;

        let completed = completed.ok_or_else(|| {
            CodexDirectError::Protocol(
                "response stream ended without response.completed".into(),
            )
        })?;
        if let Some(id) = completed.get("id").and_then(Value::as_str) {
            response_id = id.to_string();
        }
        total_usage = total_usage.add(usage_of(&completed));

        let output = output_items(&completed, &observed_output);
        let calls = output
            .iter()
            .filter(|item| {
                item.get("type").and_then(Value::as_str) == Some("function_call")
                    && item.get("name").and_then(Value::as_str).is_some()
                    && item.get("call_id").and_then(Value::as_str).is_some()
            })
            .cloned()
            .collect::<Vec<_>>();
        if calls.is_empty() {
            return Ok(CodexDirectResult {
                text,
                response_id,
                usage: total_usage,
            });
        }

        let calls_in_step = calls.len();
        let mut results = Vec::new();
        for call in calls {
            if should_cancel() {
                return Err(CodexDirectError::Cancelled(
                    "Runner cancelled before tool execution".into(),
                ));
            }
            let name = call.get("name").and_then(Value::as_str).unwrap_or("");
            let call_id = call.get("call_id").and_then(Value::as_str).unwrap_or("");
            let output = match tools_by_name.get(name).copied() {
                None => json!({
                    "isError": true,
                    "error": format!("Unknown Fabushi tool: {name}")
                }),
                Some(tool) => {
                    let arguments = call
                        .get("arguments")
                        .and_then(Value::as_str)
                        .unwrap_or("{}");
                    match serde_json::from_str::<Value>(arguments) {
                        Ok(args) => match execute_tool(tool, args, call_id) {
                            Ok(value) => value,
                            Err(error) => json!({
                                "isError": true,
                                "error": error.to_string()
                            }),
                        },
                        Err(_) => json!({
                            "isError": true,
                            "error": "Tool arguments were not valid JSON."
                        }),
                    }
                }
            };
            results.push(json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": safe_json(&output),
            }));
        }

        input.extend(output);
        input.extend(results);
        completed_steps = completed_steps.saturating_add(1);
        tool_calls_completed =
            tool_calls_completed.saturating_add(calls_in_step);
        on_checkpoint(&CodexDirectCheckpoint {
            input: input.clone(),
            text: text.clone(),
            response_id: response_id.clone(),
            usage: total_usage,
            completed_steps,
            tool_calls_completed,
        })?;
    }

    Err(CodexDirectError::Protocol(format!(
        "provider exceeded Fabushi's {max_steps}-step tool limit"
    )))
}
