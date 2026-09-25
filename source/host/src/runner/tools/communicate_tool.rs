use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const SAND_TOOL_MARKER: &str = "__sand_tool__";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "case", content = "value", rename_all = "camelCase")]
pub enum CommunicateResult {
    Success { current_step: String },
    Error { error: String },
}

pub fn encode_sand_step(payload: &serde_json::Map<String, Value>) -> String {
    let mut row = serde_json::Map::new();
    row.insert(SAND_TOOL_MARKER.to_string(), Value::Bool(true));
    for (key, value) in payload {
        row.insert(key.clone(), value.clone());
    }
    Value::Object(row).to_string()
}

pub fn tool_call_wrapper(payload: &serde_json::Map<String, Value>) -> Value {
    json!({
        "tool": {
            "case": "communicateUpdateToolCall",
            "value": {
                "args": {
                    "currentStep": encode_sand_step(payload)
                }
            }
        }
    })
}

pub fn empty_tool_call() -> Value {
    json!({
        "tool": {
            "case": "communicateUpdateToolCall",
            "value": {}
        }
    })
}

pub fn encode_error(message: &str) -> String {
    let mut payload = serde_json::Map::new();
    payload.insert("error".into(), Value::String(message.to_string()));
    encode_sand_step(&payload)
}

pub fn encode_success(result: &str) -> String {
    let mut payload = serde_json::Map::new();
    payload.insert("result".into(), Value::String(result.to_string()));
    encode_sand_step(&payload)
}

pub fn build_success_result(text: impl Into<String>) -> CommunicateResult {
    CommunicateResult::Success {
        current_step: text.into(),
    }
}

pub fn build_error_result(message: impl Into<String>) -> CommunicateResult {
    CommunicateResult::Error {
        error: message.into(),
    }
}

pub fn completed_tool_call(result: &CommunicateResult) -> Value {
    let (args, encoded_result) = match result {
        CommunicateResult::Error { error } => (
            encode_error(if error.is_empty() { "Tool failed." } else { error }),
            json!({"case":"error","value":{"error":error}}),
        ),
        CommunicateResult::Success { current_step } => (
            encode_success(current_step),
            json!({"case":"success","value":{"currentStep":current_step}}),
        ),
    };
    json!({
        "tool": {
            "case": "communicateUpdateToolCall",
            "value": {
                "args": {"currentStep": args},
                "result": {"result": encoded_result}
            }
        }
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommunicateActivity {
    pub detail: Option<String>,
    pub target: Option<String>,
}

pub fn executing_tool_call(
    tool_name: &str,
    activity: Option<&CommunicateActivity>,
) -> Value {
    let mut payload = serde_json::Map::new();
    payload.insert("phase".into(), Value::String("executing".into()));
    payload.insert("tool".into(), Value::String(tool_name.to_string()));
    if let Some(detail) = activity
        .and_then(|activity| activity.detail.as_deref())
        .filter(|value| !value.is_empty())
    {
        payload.insert("detail".into(), Value::String(detail.to_string()));
    }
    if let Some(target) = activity
        .and_then(|activity| activity.target.as_deref())
        .filter(|value| !value.is_empty())
    {
        payload.insert("target".into(), Value::String(target.to_string()));
    }
    tool_call_wrapper(&payload)
}

pub fn render_result(result: &CommunicateResult) -> String {
    match result {
        CommunicateResult::Error { error } => format!("Error: {error}"),
        CommunicateResult::Success { current_step } if current_step.is_empty() => {
            "Tool completed.".to_string()
        }
        CommunicateResult::Success { current_step } => current_step.clone(),
    }
}

pub fn run_communicate_execution<F, E>(execute: F) -> CommunicateResult
where
    F: FnOnce() -> Result<String, E>,
    E: std::fmt::Display,
{
    match execute() {
        Ok(text) => build_success_result(text),
        Err(error) => build_error_result(error.to_string()),
    }
}

pub fn serialize_error(error: impl std::fmt::Display) -> Value {
    let message = error.to_string();
    completed_tool_call(&build_error_result(message))
}
