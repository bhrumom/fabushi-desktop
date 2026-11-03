use std::env;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::extensions::inference::provider_session::ProviderSessionError;

use super::box_tool_access::{RunnerBoxResourcePort, RunnerBoxWriteRequest};

pub const MCP_TEXT_FILE_THRESHOLD_BYTES: usize = 40_000;
pub const SAND_SHELL_FILE_OUTPUT_THRESHOLD_BYTES: usize = MCP_TEXT_FILE_THRESHOLD_BYTES;
pub const MAX_OUTPUT_FILE_SIZE: usize = 1_000_000;
pub const AGENT_TOOLS_DIR: &str = ".sand/tools";

pub fn is_large_output_spill_enabled_from(value: Option<&str>) -> bool {
    value != Some("1")
}

pub fn is_large_output_spill_enabled() -> bool {
    is_large_output_spill_enabled_from(
        env::var("SAND_DISABLE_LARGE_OUTPUT_SPILL").ok().as_deref(),
    )
}

fn is_inline_text(item: &Value) -> bool {
    item.get("type").and_then(Value::as_str) == Some("text")
        && item.get("text").and_then(Value::as_str).is_some()
        && item.get("outputLocation").is_none()
}

fn cap_text(text: &str) -> String {
    if text.chars().count() <= MAX_OUTPUT_FILE_SIZE {
        return text.to_string();
    }
    text.chars().take(MAX_OUTPUT_FILE_SIZE).collect()
}

/// Materializes oversized MCP text output into the Agent box, matching Grok's
/// Runner-owned spill boundary. Known write refusals keep the original inline
/// result; an uncertain dispatched write must remain unknown to effect custody.
pub fn maybe_spill_mcp_text_result(
    box_resources: &dyn RunnerBoxResourcePort,
    result: Value,
    tool_call_id: &str,
    threshold_bytes: usize,
) -> Result<Value, ProviderSessionError> {
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return Ok(result);
    }
    let Some(content) = result.get("content").and_then(Value::as_array) else {
        return Ok(result);
    };
    let inline = content
        .iter()
        .filter(|item| is_inline_text(item))
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if inline.is_empty() {
        return Ok(result);
    }
    let aggregate = inline.join("\n\n");
    if threshold_bytes == 0 || aggregate.len() <= threshold_bytes {
        return Ok(result);
    }

    let capped = cap_text(&aggregate);
    let relative_path = format!("{AGENT_TOOLS_DIR}/{}.txt", Uuid::new_v4());
    let data = capped.as_bytes().to_vec();
    match box_resources.execute_write(RunnerBoxWriteRequest {
        path: relative_path.clone(),
        data: data.clone(),
        tool_call_id: tool_call_id.to_string(),
    }) {
        Ok(()) => {}
        Err(error @ ProviderSessionError::ToolOutcomeUnknown(_)) => return Err(error),
        Err(_) => return Ok(result),
    }

    let output_location = json!({
        "filePath": relative_path,
        "sizeBytes": data.len(),
        "lineCount": capped.split('\n').count(),
    });
    let mut materialized = Vec::with_capacity(content.len());
    let mut emitted = false;
    for item in content {
        if is_inline_text(item) {
            if !emitted {
                materialized.push(json!({
                    "type": "text",
                    "text": "",
                    "outputLocation": output_location.clone(),
                }));
                emitted = true;
            }
        } else {
            materialized.push(item.clone());
        }
    }

    let mut spilled = result;
    if let Some(object) = spilled.as_object_mut() {
        object.insert("content".into(), Value::Array(materialized));
    }
    Ok(spilled)
}
