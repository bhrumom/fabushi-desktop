use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const DEFAULT_PAGE_LIMIT: usize = 20;
pub const MAX_PAGE_LIMIT: usize = 100;
pub const MAX_CURSOR_LENGTH: usize = 128;
pub const MAX_TRANSCRIPT_ITEM_BYTES: usize = 512 * 1024; // 512 KB
pub const MAX_TRANSCRIPT_PAGE_BYTES: usize = 20 * 1024 * 1024; // 20 MB

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionVisibility {
    Public,
    Direct,
    Incognito,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub thread_id: String,
    pub title: String,
    pub visibility: SessionVisibility,
    pub updated_at_ms: u64,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCatalogQueryParams {
    pub search_term: Option<String>,
    pub limit: usize,
    pub cursor: Option<String>,
}

impl Default for SessionCatalogQueryParams {
    fn default() -> Self {
        Self {
            search_term: None,
            limit: DEFAULT_PAGE_LIMIT,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCatalogPage {
    pub items: Vec<SessionSummary>,
    pub next_cursor: Option<String>,
    pub total_count: usize,
}

pub fn normalize_page_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_PAGE_LIMIT
    } else {
        limit.min(MAX_PAGE_LIMIT)
    }
}

pub fn paginate_sessions(
    all_sessions: &[SessionSummary],
    params: &SessionCatalogQueryParams,
) -> Result<SessionCatalogPage, String> {
    if let Some(cursor) = &params.cursor {
        if cursor.len() > MAX_CURSOR_LENGTH {
            return Err(format!(
                "Cursor length {} exceeds maximum allowed {}",
                cursor.len(),
                MAX_CURSOR_LENGTH
            ));
        }
    }

    let limit = normalize_page_limit(params.limit);

    let filtered: Vec<&SessionSummary> = all_sessions
        .iter()
        .filter(|s| {
            if let Some(ref term) = params.search_term {
                let term_lower = term.to_ascii_lowercase();
                s.title.to_ascii_lowercase().contains(&term_lower)
                    || s.session_id.to_ascii_lowercase().contains(&term_lower)
                    || s.thread_id.to_ascii_lowercase().contains(&term_lower)
            } else {
                true
            }
        })
        .collect();

    let total_count = filtered.len();

    // Cursor is 0-based offset encoded as string
    let start_idx = if let Some(cursor_str) = &params.cursor {
        cursor_str
            .parse::<usize>()
            .map_err(|_| "Invalid cursor offset format".to_string())?
    } else {
        0
    };

    if start_idx >= total_count {
        return Ok(SessionCatalogPage {
            items: Vec::new(),
            next_cursor: None,
            total_count,
        });
    }

    let end_idx = (start_idx + limit).min(total_count);
    let items = filtered[start_idx..end_idx]
        .iter()
        .map(|&s| s.clone())
        .collect();

    let next_cursor = if end_idx < total_count {
        Some(end_idx.to_string())
    } else {
        None
    };

    Ok(SessionCatalogPage {
        items,
        next_cursor,
        total_count,
    })
}

/// Select the visible portion of a transcript after the canonical session
/// owner has resolved a reset-window boundary.
///
/// This is deliberately a pure projection. `reset_tail_start` is supplied by
/// the existing transcript/session owner; this module does not infer, store,
/// or mutate reset generations. Tool results are visible only when the same
/// window contains a corresponding assistant tool call, matching the SDK
/// context-engine contract and avoiding orphan result injection.
pub fn select_visible_transcript_messages(
    entries: &[Value],
    reset_tail_start: Option<usize>,
) -> Vec<Value> {
    let start = reset_tail_start.unwrap_or(0).min(entries.len());
    let window = &entries[start..];
    let paired_tool_call_ids = window
        .iter()
        .flat_map(tool_call_ids)
        .collect::<std::collections::HashSet<_>>();

    window
        .iter()
        .filter(|entry| {
            let Some(message) = message_object(entry) else {
                return true;
            };
            if message.get("role").and_then(Value::as_str) != Some("toolResult") {
                return true;
            }
            message
                .get("toolCallId")
                .and_then(Value::as_str)
                .is_some_and(|tool_call_id| paired_tool_call_ids.contains(tool_call_id))
        })
        .cloned()
        .collect()
}

fn message_object(entry: &Value) -> Option<&Map<String, Value>> {
    entry
        .as_object()
        .and_then(|object| object.get("message").and_then(Value::as_object).or(Some(object)))
}

fn tool_call_ids(entry: &Value) -> Vec<&str> {
    let Some(message) = message_object(entry) else {
        return Vec::new();
    };
    if message.get("role").and_then(Value::as_str) != Some("assistant") {
        return Vec::new();
    }
    message
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|part| part.get("type").and_then(Value::as_str) == Some("toolCall"))
        .filter_map(|part| part.get("id").and_then(Value::as_str))
        .collect()
}

#[derive(Clone, Default)]
pub struct SessionTranscriptLockRegistry {
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl SessionTranscriptLockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire_lock(&self, agent_id: &str, session_id: &str) -> Arc<Mutex<()>> {
        let key = format!("{agent_id}:{session_id}");
        let mut locks = self.locks.lock().unwrap();
        locks.entry(key).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
    }

    pub fn enforce_sync_allowed(&self, visibility: SessionVisibility) -> Result<(), String> {
        if visibility == SessionVisibility::Incognito {
            Err("IncognitoSessionSyncAccessDenied: session is ephemeral and persistence sync is forbidden".to_string())
        } else {
            Ok(())
        }
    }
}
