use std::env;

use serde_json::{Value, json};
use uuid::Uuid;

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

fn cap_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let notice = format!("\n[OUTPUT TRUNCATED: original {} UTF-8 bytes; middle omitted]\n", text.len());
    if limit < notice.len() { return notice[..limit].to_string(); }
    let available = limit.saturating_sub(notice.len());
    let mut head = available * 2 / 5;
    let mut tail = text.len().saturating_sub(available - head);
    while !text.is_char_boundary(head) { head -= 1; }
    while !text.is_char_boundary(tail) { tail += 1; }
    format!("{}{}{}", &text[..head], notice, &text[tail..])
}

/// Bounded fallback when storage is disabled/unavailable. Preserve error flags
/// and non-text blocks; report partial data rather than pretending it is complete.
pub fn bound_mcp_inline_text(mut result: Value, limit: usize) -> Value {
    let Some(content) = result.get("content").and_then(Value::as_array) else { return result; };
    let aggregate = content.iter().filter(|item| is_inline_text(item))
        .filter_map(|item| item.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join("\n\n");
    if aggregate.len() <= limit { return result; }
    let text = cap_text(&aggregate, limit);
    let retained = text.len();
    let mut emitted = false;
    let bounded = content.iter().filter_map(|item| {
        if !is_inline_text(item) { return Some(item.clone()); }
        if emitted { return None; }
        emitted = true;
        Some(json!({"type":"text","text":text}))
    }).collect::<Vec<_>>();
    result["content"] = Value::Array(bounded);
    result["outputTruncation"] = json!({"originalBytes":aggregate.len(),"retainedBytes":retained,
        "truncated":true,"materialized":false,
        "notice":"Only partial output is available. Query smaller read-only pages; reconcile any uncertain mutation before retrying it."});
    result
}

/// Materializes oversized MCP text output into the Agent box, matching Grok's
/// Runner-owned spill boundary. Failure to write is deliberately non-fatal:
/// a bounded, explicitly partial inline result is returned.
pub fn maybe_spill_mcp_text_result(
    box_resources: &dyn RunnerBoxResourcePort,
    result: Value,
    tool_call_id: &str,
    threshold_bytes: usize,
) -> Value {
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return bound_mcp_inline_text(result, MCP_TEXT_FILE_THRESHOLD_BYTES);
    }
    let Some(content) = result.get("content").and_then(Value::as_array) else {
        return result;
    };
    let inline = content
        .iter()
        .filter(|item| is_inline_text(item))
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if inline.is_empty() {
        return result;
    }
    let aggregate = inline.join("\n\n");
    if threshold_bytes == 0 || aggregate.len() <= threshold_bytes {
        return result;
    }

    let capped = cap_text(&aggregate, MAX_OUTPUT_FILE_SIZE);
    let relative_path = format!("{AGENT_TOOLS_DIR}/{}.txt", Uuid::new_v4());
    let data = capped.as_bytes().to_vec();
    if box_resources
        .execute_write(RunnerBoxWriteRequest {
            path: relative_path.clone(),
            data: data.clone(),
            tool_call_id: tool_call_id.to_string(),
        })
        .is_err()
    {
        return bound_mcp_inline_text(result, MCP_TEXT_FILE_THRESHOLD_BYTES);
    }

    let output_location = json!({
        "filePath": relative_path,
        "sizeBytes": data.len(),
        "lineCount": capped.split('\n').count(),
        "originalSizeBytes": aggregate.len(),
        "truncated": aggregate.len() > data.len(),
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
    spilled
}
