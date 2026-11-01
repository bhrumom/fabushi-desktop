//! Fixed-source context-compaction planning at the Runner boundary.
//!
//! This module only decides whether a canonical transcript projection needs
//! compaction and where a tool-safe retained tail begins. It owns no session
//! writes, summarization transport, or provider serialization.

use std::collections::BTreeSet;

use serde_json::{Value, json};

const CHARS_PER_TOKEN_ESTIMATE: u64 = 4;
const IMAGE_BLOCK_TOKENS: u64 = 2_000;
pub const MAX_COMPACTION_SUMMARY_CHARS: usize = 16_000;
pub const MAX_LATEST_USER_REQUEST_CHARS: usize = 800;

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
    pub turn_prefix_entries: Vec<Value>,
    pub previous_summary: Option<String>,
    pub previous_details: Option<Value>,
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
    let previous_summary = entries[..cut]
        .iter()
        .rev()
        .find(|entry| entry.get("kind").and_then(Value::as_str) == Some("compaction"))
        .and_then(|entry| entry.get("summary").and_then(Value::as_str))
        .map(ToOwned::to_owned);
    let is_split_turn = entry_role(&entries[cut]) == Some("assistant");
    let turn_prefix_start = if is_split_turn {
        entries[..cut]
            .iter()
            .rposition(|entry| entry_role(entry) == Some("user"))
            .unwrap_or(cut)
    } else {
        cut
    };
    let turn_prefix_entries = entries[turn_prefix_start..cut]
        .iter()
        .filter(|entry| entry.get("kind").and_then(Value::as_str) != Some("compaction"))
        .cloned()
        .collect();
    let entries_to_summarize = entries[..turn_prefix_start]
        .iter()
        .filter(|entry| entry.get("kind").and_then(Value::as_str) != Some("compaction"))
        .cloned()
        .collect();
    let latest_user_request = entries[..cut]
        .iter()
        .rev()
        .find(|entry| entry_role(entry) == Some("user"))
        .and_then(|entry| entry_text(entry))
        .filter(|text| !text.trim().is_empty())
        .map(bound_latest_user_request);

    Some(CompactionPlan {
        first_kept_entry_id,
        entries_to_summarize,
        turn_prefix_entries,
        previous_summary,
        previous_details: entries[..cut]
            .iter()
            .rev()
            .find(|entry| entry.get("kind").and_then(Value::as_str) == Some("compaction"))
            .and_then(|entry| entry.get("details").cloned()),
        tokens_before,
        summary_token_budget: settings.reserve_tokens.saturating_mul(4) / 5,
        latest_user_request,
        is_split_turn,
    })
}

/// Build the bounded user prompt consumed by the existing summarization
/// inference owner. The prompt is deliberately a projection of the plan;
/// this module does not decide transport, persistence, or provider format.
pub fn build_compaction_summary_prompt(plan: &CompactionPlan) -> Result<String, String> {
    let transcript = serde_json::to_string(&plan.entries_to_summarize)
        .map_err(|error| format!("could not encode compaction transcript: {error}"))?;
    let previous = plan.previous_summary.as_deref().unwrap_or("(none)");
    let latest_request = plan.latest_user_request.as_deref().unwrap_or("(none)");
    Ok(format!(
        "Create a concise, durable conversation summary for the next agent turn. Preserve user intent, decisions, constraints, unresolved work, tool outcomes, and important identifiers. Do not invent facts. The summary must stand alone and fit within {budget} tokens.\n\nPrevious compaction summary (already captured; correct it only when the transcript provides newer facts):\n{previous}\n\nLatest retained user request (must remain actionable even when this is a split turn):\n{latest_request}\n\nTranscript rows to summarize (JSON, in canonical order):\n{transcript}",
        budget = plan.summary_token_budget,
    ))
}

/// Prompt for the prefix of a turn whose recent suffix remains verbatim.
pub fn build_turn_prefix_summary_prompt(plan: &CompactionPlan) -> Result<String, String> {
    let transcript = serde_json::to_string(&plan.turn_prefix_entries)
        .map_err(|error| format!("could not encode split-turn prefix: {error}"))?;
    Ok(format!(
        "This is the PREFIX of a turn that was too large to keep. The SUFFIX (recent work) is retained verbatim. Summarize only the prefix so the next agent can understand the retained suffix. Use exactly these sections:\n\n## Original Request\n[What did the user ask for?]\n\n## Early Progress\n- [Key decisions and work done in the prefix]\n\n## Context for Suffix\n- [Information needed to understand the kept suffix]\n\nBe concise and preserve exact paths, identifiers, and errors.\n\nPrefix transcript rows (JSON):\n{transcript}"
    ))
}

/// Fit model output to the provider-independent persisted-summary bound.
/// The marker is part of the durable summary so a later turn knows that the
/// tail was intentionally elided rather than silently missing.
pub fn fit_compaction_summary(summary: &str, summary_token_budget: u64) -> Result<String, String> {
    let normalized = normalize_summary_text(summary)?;
    let token_bound = usize::try_from(summary_token_budget)
        .unwrap_or(usize::MAX)
        .saturating_mul(CHARS_PER_TOKEN_ESTIMATE as usize);
    let max_chars = MAX_COMPACTION_SUMMARY_CHARS.min(token_bound.max(1));
    if normalized.chars().count() <= max_chars {
        return Ok(normalized);
    }
    const MARKER: &str = "\n\n[Compaction summary truncated to fit budget]";
    if max_chars <= MARKER.chars().count() {
        return Ok(normalized.chars().take(max_chars).collect());
    }
    let prefix_chars = max_chars - MARKER.chars().count();
    Ok(format!(
        "{}{}",
        normalized.chars().take(prefix_chars).collect::<String>(),
        MARKER
    ))
}

/// Fixed-source no-summary fallback. It deliberately names the dropped span
/// and carries forward the previous summary/request instead of inventing facts.
pub fn compaction_without_summary(plan: &CompactionPlan) -> String {
    let mut fallback = format!(
        "[{} earlier message(s) were removed without a summary because summarization failed. The messages after this summary are retained verbatim; ask the user if older details matter.]",
        plan.entries_to_summarize.len() + plan.turn_prefix_entries.len()
    );
    if let Some(request) = plan.latest_user_request.as_deref().filter(|value| !value.is_empty()) {
        fallback.push_str("\n\n## Original request of the current turn\n");
        fallback.push_str(&serde_json::to_string(request).unwrap_or_else(|_| "\"(unavailable)\"".into()));
    }
    if let Some(previous) = plan.previous_summary.as_deref().filter(|value| !value.is_empty()) {
        fallback.push_str("\n\n## Previous summary\n");
        fallback.push_str(previous);
    }
    fallback
}

/// Reconstruct the fixed source's bounded file-operation details from the
/// canonical transcript rows. The returned value is the only details shape
/// accepted by the compaction entry owner; unknown tool rows are ignored just
/// as the fixed source ignores tools outside its read/write/edit contract.
pub fn build_compaction_details(plan: &CompactionPlan) -> Option<Value> {
    let mut read = BTreeSet::new();
    let mut written = BTreeSet::new();
    let mut edited = BTreeSet::new();
    let mut quality_degraded = false;
    let mut previous_latest_request = None::<String>;

    if let Some(previous) = plan.previous_details.as_ref().and_then(Value::as_object) {
        add_string_array(&mut read, previous.get("readFiles"));
        add_string_array(&mut edited, previous.get("modifiedFiles"));
        quality_degraded = previous
            .get("qualityDegraded")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        previous_latest_request = previous
            .get("latestUnresolvedUserRequest")
            .and_then(Value::as_str)
            .filter(|request| !request.trim().is_empty())
            .map(ToOwned::to_owned);
    }

    for entry in plan
        .entries_to_summarize
        .iter()
        .chain(plan.turn_prefix_entries.iter())
    {
        let Some(object) = entry.as_object() else {
            continue;
        };
        match object.get("role").and_then(Value::as_str) {
            Some("assistant") => {
                if let Some(calls) = object.get("toolCalls").and_then(Value::as_array) {
                    for call in calls {
                        let Some(call) = call.as_object() else {
                            continue;
                        };
                        let Some(name) = call.get("name").and_then(Value::as_str) else {
                            continue;
                        };
                        let path = call
                            .get("arguments")
                            .or_else(|| call.get("args"))
                            .and_then(Value::as_object)
                            .and_then(|arguments| {
                                ["path", "file_path", "filePath"]
                                    .into_iter()
                                    .find_map(|key| arguments.get(key).and_then(Value::as_str))
                            });
                        let Some(path) = path.filter(|path| !path.trim().is_empty()) else {
                            continue;
                        };
                        match normalize_file_tool_name(name).as_str() {
                            "read" => {
                                read.insert(path.to_string());
                            }
                            "write" => {
                                written.insert(path.to_string());
                            }
                            "edit" => {
                                edited.insert(path.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some("tool") => {
                if normalize_file_tool_name(
                    object
                        .get("toolName")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                ) == "apply_patch"
                {
                    for value in [object.get("result"), object.get("content")]
                        .into_iter()
                        .flatten()
                    {
                        collect_apply_patch_summary(value, &mut written, &mut edited);
                    }
                }
            }
            _ => {}
        }
    }

    let modified = written.into_iter().chain(edited).collect::<BTreeSet<_>>();
    let read_files = read
        .difference(&modified)
        .cloned()
        .collect::<Vec<_>>();
    let modified_files = modified.into_iter().collect::<Vec<_>>();
    let latest_request = plan
        .latest_user_request
        .as_deref()
        .filter(|request| !request.trim().is_empty())
        .or(previous_latest_request.as_deref());
    if read_files.is_empty()
        && modified_files.is_empty()
        && latest_request.is_none()
        && !quality_degraded
    {
        return None;
    }

    let mut details = json!({
        "readFiles": read_files,
        "modifiedFiles": modified_files,
    });
    let object = details.as_object_mut().expect("details object");
    if let Some(request) = latest_request {
        object.insert(
            "latestUnresolvedUserRequest".into(),
            Value::String(request.to_string()),
        );
    }
    if quality_degraded {
        object.insert("qualityDegraded".into(), Value::Bool(true));
    }
    Some(details)
}

fn add_string_array(target: &mut BTreeSet<String>, value: Option<&Value>) {
    let Some(values) = value.and_then(Value::as_array) else {
        return;
    };
    target.extend(
        values
            .iter()
            .filter_map(Value::as_str)
            .filter(|path| !path.trim().is_empty())
            .map(ToOwned::to_owned),
    );
}

fn collect_apply_patch_summary(
    value: &Value,
    written: &mut BTreeSet<String>,
    edited: &mut BTreeSet<String>,
) {
    let Some(object) = value.as_object() else {
        if let Some(values) = value.as_array() {
            for value in values {
                collect_apply_patch_summary(value, written, edited);
            }
        }
        return;
    };
    let Some(summary) = object
        .get("details")
        .and_then(Value::as_object)
        .and_then(|details| details.get("summary"))
        .and_then(Value::as_object)
    else {
        return;
    };
    add_string_array(written, summary.get("added"));
    add_string_array(edited, summary.get("modified"));
}

fn normalize_file_tool_name(value: &str) -> String {
    let name = value.to_ascii_lowercase();
    let start = if name.starts_with("mcp__") { 5 } else { 0 };
    name[start..]
        .find("__")
        .map(|offset| name[start + offset + 2..].to_string())
        .unwrap_or(name)
}

/// Validate summarizer output before the session owner is allowed to mutate
/// durable transcript state. Empty, oversized, or control-only output is a
/// no-summary failure and leaves the old transcript authoritative.
pub fn validate_compaction_summary(summary: &str) -> Result<String, String> {
    let normalized = normalize_summary_text(summary)?;
    if normalized.chars().count() > MAX_COMPACTION_SUMMARY_CHARS {
        return Err(format!(
            "compaction summary exceeds {} characters",
            MAX_COMPACTION_SUMMARY_CHARS
        ));
    }
    Ok(normalized)
}

fn normalize_summary_text(summary: &str) -> Result<String, String> {
    let normalized = summary.trim();
    if normalized.is_empty() {
        return Err("compaction summarizer returned an empty summary".into());
    }
    if normalized
        .chars()
        .any(|character| character.is_control() && character != '\n' && character != '\t')
    {
        return Err("compaction summary contains unsupported control characters".into());
    }
    Ok(normalized.to_string())
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

fn bound_latest_user_request(text: String) -> String {
    if text.chars().count() <= MAX_LATEST_USER_REQUEST_CHARS {
        return text;
    }
    const MARKER: &str = "\n[... latest user request truncated ...]\n";
    let budget = MAX_LATEST_USER_REQUEST_CHARS.saturating_sub(MARKER.chars().count());
    let head = budget / 2;
    let tail = budget.saturating_sub(head);
    let chars = text.chars().collect::<Vec<_>>();
    format!(
        "{}{}{}",
        chars.iter().take(head).copied().collect::<String>(),
        MARKER,
        chars
            .iter()
            .rev()
            .take(tail)
            .rev()
            .copied()
            .collect::<String>()
    )
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
        let Some(pair_end) = entries[candidate_index + 1..]
            .iter()
            .enumerate()
            .filter_map(|(offset, result)| {
                result
                    .get("toolCallId")
                    .and_then(Value::as_str)
                    .is_some_and(|id| ids.iter().any(|candidate| *candidate == id))
                    .then_some(candidate_index + 1 + offset)
            })
            .max()
        else {
            continue;
        };
        if candidate_index <= index && index <= pair_end {
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
        assert_eq!(plan.entries_to_summarize.len(), 3);
    }

    #[test]
    fn split_turn_separates_prefix_from_older_history() {
        let entries = vec![
            json!({"id":"old", "kind":"message", "role":"user", "content":"old"}),
            json!({"id":"old-a", "kind":"message", "role":"assistant", "content":"done"}),
            json!({"id":"current", "kind":"message", "role":"user", "content":"current request"}),
            json!({"id":"current-a", "kind":"message", "role":"assistant", "content":"partial"}),
            json!({"id":"kept", "kind":"message", "role":"user", "content":"follow up"}),
        ];
        let plan = prepare_compaction_plan(
            &entries,
            100,
            100,
            CompactionSettings { enabled: true, reserve_tokens: 1, keep_recent_tokens: 1 },
            None,
        ).expect("plan");
        assert!(plan.is_split_turn);
        assert_eq!(plan.turn_prefix_entries.len(), 2);
        assert_eq!(plan.turn_prefix_entries[0]["id"], "current");
        assert_eq!(plan.entries_to_summarize.len(), 2);
    }

    #[test]
    fn prior_compaction_is_summary_context_not_new_history() {
        let entries = vec![
            json!({"id":"c0", "kind":"compaction", "summary":"prior", "tokensBefore":10}),
            json!({"id":"u1", "kind":"message", "role":"user", "content":"new"}),
            json!({"id":"a1", "kind":"message", "role":"assistant", "content":"answer"}),
        ];
        let plan = prepare_compaction_plan(
            &entries,
            100,
            100,
            CompactionSettings { enabled: true, reserve_tokens: 1, keep_recent_tokens: 1 },
            None,
        ).expect("plan");
        assert_eq!(plan.previous_summary.as_deref(), Some("prior"));
        assert!(plan.entries_to_summarize.iter().all(|entry| {
            entry.get("kind").and_then(Value::as_str) != Some("compaction")
        }));
    }

    #[test]
    fn file_operation_details_merge_previous_and_current_rows() {
        let plan = CompactionPlan {
            first_kept_entry_id: "keep".into(),
            entries_to_summarize: vec![
                json!({
                    "kind":"message",
                    "role":"assistant",
                    "toolCalls":[
                        {"name":"read", "arguments":{"path":"src/lib.rs"}},
                        {"name":"write", "arguments":{"path":"src/new.rs"}}
                    ]
                }),
                json!({
                    "kind":"message",
                    "role":"tool",
                    "toolName":"apply_patch",
                    "result":{"details":{"summary":{"modified":["src/lib.rs"],"added":["src/a.rs"]}}}
                }),
            ],
            turn_prefix_entries: Vec::new(),
            previous_summary: Some("prior".into()),
            previous_details: Some(json!({"readFiles":["src/old.rs"],"modifiedFiles":["src/kept.rs"]})),
            tokens_before: 10,
            summary_token_budget: 10,
            latest_user_request: Some("continue".into()),
            is_split_turn: false,
        };
        assert_eq!(
            build_compaction_details(&plan),
            Some(json!({
                "readFiles":["src/old.rs"],
                "modifiedFiles":["src/a.rs","src/kept.rs","src/lib.rs","src/new.rs"],
                "latestUnresolvedUserRequest":"continue"
            }))
        );
    }

    #[test]
    fn summary_failure_fallback_preserves_previous_context_and_request() {
        let plan = CompactionPlan {
            first_kept_entry_id: "keep".into(),
            entries_to_summarize: vec![json!({"kind":"message","role":"user","content":"old"})],
            turn_prefix_entries: Vec::new(),
            previous_summary: Some("prior decisions".into()),
            previous_details: None,
            tokens_before: 10,
            summary_token_budget: 10,
            latest_user_request: Some("finish this".into()),
            is_split_turn: true,
        };
        let fallback = compaction_without_summary(&plan);
        assert!(fallback.contains("without a summary"));
        assert!(fallback.contains("finish this"));
        assert!(fallback.contains("prior decisions"));
    }
}
