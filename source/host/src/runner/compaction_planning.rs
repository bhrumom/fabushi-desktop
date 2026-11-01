//! Fixed-source context-compaction planning at the Runner boundary.
//!
//! This module only decides whether a canonical transcript projection needs
//! compaction and where a tool-safe retained tail begins. It owns no session
//! writes, summarization transport, or provider serialization.

use serde_json::Value;

const CHARS_PER_TOKEN_ESTIMATE: u64 = 4;
const IMAGE_BLOCK_TOKENS: u64 = 2_000;
pub const MAX_COMPACTION_SUMMARY_CHARS: usize = 16_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionSettings {
    pub enabled: bool,
    pub reserve_tokens: u64,
    pub keep_recent_tokens: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompactionPlan {
    pub first_kept_entry_id: String,
    pub entries_to_summarize: Vec<Value>,
    pub tokens_before: u64,
    pub summary_token_budget: u64,
    pub latest_user_request: Option<String>,
    pub is_split_turn: bool,
}

/// Mirrors the fixed source threshold contract. Invalid windows never trigger
/// a destructive history operation.
pub fn should_compact(
    context_tokens: u64,
    context_window: u64,
    settings: CompactionSettings,
) -> bool {
    settings.enabled
        && context_window > 0
        && context_tokens > context_window.saturating_sub(settings.reserve_tokens)
}

/// Estimate one canonical transcript row using the fixed source's conservative
/// character heuristic. Unknown metadata rows contribute no context tokens;
/// malformed replay rows are rejected later by the typed projection owner.
pub fn estimate_entry_tokens(entry: &Value) -> u64 {
    let object = match entry.as_object() {
        Some(object) => object,
        None => return 0,
    };
    let mut chars = 0u64;
    match object.get("kind").and_then(Value::as_str) {
        Some("message") => {
            chars = value_chars(object.get("content"));
            if object.get("role").and_then(Value::as_str) == Some("assistant") {
                if let Some(calls) = object.get("toolCalls").and_then(Value::as_array) {
                    chars = chars.saturating_add(value_chars(Some(&Value::Array(calls.clone()))));
                }
            }
        }
        Some("send-message") => {
            chars = value_chars(object.get("message").and_then(|value| value.get("content")));
        }
        Some("user-attachment") => return IMAGE_BLOCK_TOKENS,
        Some("tool-call") => {
            chars = value_chars(object.get("name"));
            chars = chars.saturating_add(value_chars(object.get("arguments").or_else(|| object.get("args"))));
        }
        Some("notice") => chars = value_chars(object.get("text")),
        Some("event") => chars = value_chars(object.get("event")),
        Some("compaction") => chars = value_chars(object.get("summary")),
        _ => {}
    }
    if chars == 0 {
        0
    } else {
        chars.saturating_add(CHARS_PER_TOKEN_ESTIMATE - 1) / CHARS_PER_TOKEN_ESTIMATE
    }
}

/// Prepare a bounded retained tail. The plan is pure and does not imply that
/// summary generation or durable commit succeeded.
pub fn prepare_compaction_plan(
    entries: &[Value],
    context_tokens: u64,
    context_window: u64,
    settings: CompactionSettings,
    preserve_from_entry_id: Option<&str>,
) -> Option<CompactionPlan> {
    if !should_compact(context_tokens, context_window, settings) || entries.len() < 2 {
        return None;
    }
    let last = entries.last()?;
    if last.get("kind").and_then(Value::as_str) == Some("compaction") {
        return None;
    }

    let mut cut = None;
    let mut last_allowed = entries.len() - 1;
    for (index, entry) in entries.iter().enumerate() {
        if preserve_from_entry_id.is_some_and(|id| entry.get("id").and_then(Value::as_str) == Some(id)) {
            last_allowed = index;
        }
        if is_cut_point(entry) {
            cut = Some(index);
        }
    }
    let mut cut = cut?;
    cut = resolve_tool_pair_boundary(entries, cut);
    last_allowed = resolve_tool_pair_boundary(entries, last_allowed);

    let mut retained_tokens = 0u64;
    for index in (0..entries.len()).rev() {
        retained_tokens = retained_tokens.saturating_add(estimate_entry_tokens(&entries[index]));
        if retained_tokens >= settings.keep_recent_tokens {
            break;
        }
        if is_cut_point(&entries[index]) {
            cut = resolve_tool_pair_boundary(entries, index);
        }
    }
    cut = cut.min(last_allowed);
    if cut == 0 || cut >= entries.len() {
        return None;
    }
    let first_kept_entry_id = entries[cut]
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())?
        .to_string();
    let tokens_before = entries[..cut]
        .iter()
        .map(estimate_entry_tokens)
        .fold(0u64, u64::saturating_add);
    let latest_user_request = entries[cut..]
        .iter()
        .rev()
        .find(|entry| entry_role(entry) == Some("user"))
        .and_then(|entry| entry_text(entry))
        .filter(|text| !text.trim().is_empty());

    Some(CompactionPlan {
        first_kept_entry_id,
        entries_to_summarize: entries[..cut].to_vec(),
        tokens_before,
        summary_token_budget: settings.reserve_tokens.saturating_mul(4) / 5,
        latest_user_request,
        is_split_turn: entry_role(&entries[cut]) == Some("assistant"),
    })
}

fn value_chars(value: Option<&Value>) -> u64 {
    value.map(|value| value.to_string().chars().count() as u64).unwrap_or_default()
}

fn entry_role(entry: &Value) -> Option<&str> {
    entry.get("role").and_then(Value::as_str).or_else(|| {
        entry
            .get("message")
            .and_then(|message| message.get("role"))
            .and_then(Value::as_str)
    })
}

fn entry_text(entry: &Value) -> Option<String> {
    entry
        .get("content")
        .and_then(Value::as_str)
        .or_else(|| entry.get("text").and_then(Value::as_str))
        .map(ToOwned::to_owned)
}

fn is_cut_point(entry: &Value) -> bool {
    matches!(entry_role(entry), Some("user" | "assistant"))
}

fn resolve_tool_pair_boundary(entries: &[Value], index: usize) -> usize {
    let mut pair_start = index;
    for (candidate_index, entry) in entries.iter().enumerate() {
        let Some(calls) = entry.get("toolCalls").and_then(Value::as_array) else {
            continue;
        };
        let ids = calls
            .iter()
            .filter_map(|call| call.get("id").and_then(Value::as_str))
            .collect::<Vec<_>>();
        if ids.is_empty() {
            continue;
        }
        let has_later_result = entries[candidate_index + 1..].iter().any(|result| {
            result
                .get("toolCallId")
                .and_then(Value::as_str)
                .is_some_and(|id| ids.iter().any(|candidate| *candidate == id))
        });
        if has_later_result && candidate_index <= index {
            pair_start = pair_start.min(candidate_index);
        }
    }
    pair_start
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invalid_window_does_not_trigger_compaction() {
        let settings = CompactionSettings { enabled: true, reserve_tokens: 10, keep_recent_tokens: 1 };
        assert!(!should_compact(100, 0, settings));
    }

    #[test]
    fn plan_keeps_tool_call_boundary_and_latest_user() {
        let entries = vec![
            json!({"id":"u1", "kind":"message", "role":"user", "content":"old"}),
            json!({"id":"a1", "kind":"message", "role":"assistant", "content":"call", "toolCalls":[{"id":"c1","name":"read","arguments":{}}]}),
            json!({"id":"t1", "kind":"message", "role":"tool", "toolCallId":"c1", "toolName":"read", "result":{"ok":true}}),
            json!({"id":"u2", "kind":"message", "role":"user", "content":"latest"}),
        ];
        let plan = prepare_compaction_plan(
            &entries,
            100,
            100,
            CompactionSettings { enabled: true, reserve_tokens: 1, keep_recent_tokens: 1 },
            None,
        ).expect("plan");
        assert_eq!(plan.first_kept_entry_id, "u2");
        assert_eq!(plan.latest_user_request.as_deref(), Some("latest"));
    }
}
