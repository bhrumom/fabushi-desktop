use std::cmp::Ordering;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::extensions::session::production::ProductionSessionWorkers;

pub const AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT: usize = 5;
pub const AGENT_CONTENT_SEARCH_MAX_RESULTS: usize = 50;
const SNIPPET_LEAD: usize = 30;
const SNIPPET_TRAIL: usize = 60;
const ELLIPSIS: &str = "…";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSearchResult {
    pub agent_id: String,
    pub entry_id: String,
    pub role: String,
    pub timestamp_ms: f64,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentContentMatch {
    pub entry_id: String,
    pub role: String,
    pub timestamp_ms: f64,
    pub snippet: String,
}

pub fn entry_search_text(entry: &Value) -> &str {
    match entry.get("kind").and_then(Value::as_str) {
        Some("message") => entry.get("content").and_then(Value::as_str).unwrap_or_default(),
        Some("send-message")
            if entry
                .get("message")
                .and_then(Value::as_object)
                .and_then(|message| message.get("type"))
                .and_then(Value::as_str)
                == Some("text") =>
        {
            entry
                .get("message")
                .and_then(Value::as_object)
                .and_then(|message| message.get("content"))
                .and_then(Value::as_str)
                .unwrap_or_default()
        }
        Some("notice") => entry.get("text").and_then(Value::as_str).unwrap_or_default(),
        _ => "",
    }
}

pub fn build_content_snippet(text: &str, normalized_query: &str) -> Option<String> {
    if normalized_query.is_empty() {
        return None;
    }
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return None;
    }
    let lowered = flat.to_lowercase();
    let index = lowered.find(normalized_query)?;
    let query_end = index.saturating_add(normalized_query.len());
    let start = char_boundary_at_or_after(&flat, index.saturating_sub(SNIPPET_LEAD));
    let end = char_boundary_at_or_before(
        &flat,
        query_end.saturating_add(SNIPPET_TRAIL).min(flat.len()),
    );
    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str(ELLIPSIS);
    }
    snippet.push_str(flat.get(start..end).unwrap_or(&flat));
    if end < flat.len() {
        snippet.push_str(ELLIPSIS);
    }
    Some(snippet)
}

pub fn find_agent_content_matches(
    entries: &[Value],
    normalized_query: &str,
    limit: usize,
) -> Vec<AgentContentMatch> {
    if normalized_query.is_empty() || limit == 0 {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for entry in entries.iter().rev() {
        if matches.len() >= limit {
            break;
        }
        if entry
            .get("hiddenOutboundAgentPeerMessage")
            .and_then(Value::as_bool)
            == Some(true)
        {
            continue;
        }
        let Some(snippet) = build_content_snippet(entry_search_text(entry), normalized_query) else {
            continue;
        };
        let Some(entry_id) = entry.get("id").and_then(Value::as_str) else {
            continue;
        };
        let role = if entry.get("kind").and_then(Value::as_str) == Some("message") {
            entry
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("assistant")
        } else {
            "assistant"
        };
        matches.push(AgentContentMatch {
            entry_id: entry_id.to_string(),
            role: role.to_string(),
            timestamp_ms: entry
                .get("timestampMs")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite())
                .unwrap_or_default(),
            snippet,
        });
    }
    matches
}

pub fn search_agents_linear(
    session: &Arc<ProductionSessionWorkers>,
    query: &str,
    limit: usize,
) -> Result<Vec<TranscriptSearchResult>, String> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let agents = session.list_agent_summaries(None)?;
    let mut results = Vec::new();
    for agent in agents {
        let entries = session.read_agent_transcript_entries(&agent.id)?;
        for matched in find_agent_content_matches(
            &entries,
            &normalized,
            AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT,
        ) {
            results.push(TranscriptSearchResult {
                agent_id: agent.id.clone(),
                entry_id: matched.entry_id,
                role: matched.role,
                timestamp_ms: if matched.timestamp_ms > 0.0 {
                    matched.timestamp_ms
                } else {
                    agent.updated_at
                },
                snippet: matched.snippet,
            });
        }
    }
    results.sort_by(|left, right| {
        right
            .timestamp_ms
            .partial_cmp(&left.timestamp_ms)
            .unwrap_or(Ordering::Equal)
    });
    results.truncate(limit);
    Ok(results)
}

fn char_boundary_at_or_after(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index < text.len() && !text.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn char_boundary_at_or_before(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
