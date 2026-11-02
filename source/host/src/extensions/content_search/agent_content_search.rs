pub const AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT: usize = 5;
pub const AGENT_CONTENT_SEARCH_MAX_RESULTS: usize = 50;

const SNIPPET_LEAD: usize = 30;
const SNIPPET_TRAIL: usize = 60;
const ELLIPSIS: &str = "…";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentContentRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchableTranscriptEntryKind {
    Message {
        content: String,
        role: AgentContentRole,
    },
    SendMessage {
        message_type: String,
        content: Option<String>,
    },
    Notice {
        text: String,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchableTranscriptEntry {
    pub id: String,
    pub kind: SearchableTranscriptEntryKind,
    pub timestamp_ms: Option<f64>,
    pub hidden_outbound_agent_peer_message: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentContentMatch {
    pub entry_id: String,
    pub role: AgentContentRole,
    pub timestamp_ms: f64,
    pub snippet: String,
}

pub fn entry_search_text(entry: &SearchableTranscriptEntry) -> &str {
    match &entry.kind {
        SearchableTranscriptEntryKind::Message { content, .. } => content,
        SearchableTranscriptEntryKind::SendMessage {
            message_type,
            content,
        } if message_type == "text" => content.as_deref().unwrap_or(""),
        SearchableTranscriptEntryKind::Notice { text } => text,
        SearchableTranscriptEntryKind::SendMessage { .. }
        | SearchableTranscriptEntryKind::Other => "",
    }
}

fn flatten_whitespace(text: &str) -> String {
    let mut flat = String::new();
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !flat.is_empty() {
                pending_space = true;
            }
            continue;
        }
        if pending_space {
            flat.push(' ');
            pending_space = false;
        }
        flat.push(ch);
    }
    flat
}

fn find_normalized_query(
    flat: &str,
    normalized_query: &str,
) -> Option<(usize, usize)> {
    if normalized_query.is_empty() {
        return None;
    }

    let lowered = flat.to_lowercase();
    let match_start = lowered.find(normalized_query)?;
    let match_end = match_start + normalized_query.len();

    // Lookup must use the whole string for contextual Greek sigma. For origin
    // lengths, str and char lowercase use the same mapping except Σ -> σ/ς,
    // whose alternatives both occupy two UTF-8 bytes. See the standard-library
    // sources: https://doc.rust-lang.org/src/alloc/str.rs.html (to_lowercase)
    // and https://doc.rust-lang.org/src/core/char/methods.rs.html (to_lowercase).
    // One forward scan maps even a partial lowercase expansion to its complete
    // original character; no growing prefixes are lowercased or rescanned.
    let mut lowered_offset = 0;
    let mut source_start = None;
    for (source_offset, ch) in flat.char_indices() {
        lowered_offset += ch.to_lowercase().map(char::len_utf8).sum::<usize>();
        if source_start.is_none() && lowered_offset > match_start {
            source_start = Some(source_offset);
        }
        if lowered_offset >= match_end {
            return Some((source_start?, source_offset + ch.len_utf8()));
        }
    }
    None
}

enum SnippetContext {
    Characters,
    Bytes,
}

fn build_content_snippet_with_context(
    text: &str,
    normalized_query: &str,
    context: SnippetContext,
) -> Option<String> {
    if normalized_query.is_empty() {
        return None;
    }

    let flat = flatten_whitespace(text);
    let (match_start, match_end) = find_normalized_query(&flat, normalized_query)?;
    let (start, end) = match context {
        SnippetContext::Characters => (
            flat[..match_start]
                .char_indices()
                .rev()
                .nth(SNIPPET_LEAD - 1)
                .map(|(index, _)| index)
                .unwrap_or(0),
            flat[match_end..]
                .char_indices()
                .nth(SNIPPET_TRAIL)
                .map(|(index, _)| match_end + index)
                .unwrap_or(flat.len()),
        ),
        SnippetContext::Bytes => {
            let mut start = match_start.saturating_sub(SNIPPET_LEAD);
            let mut end = match_end.saturating_add(SNIPPET_TRAIL).min(flat.len());
            while !flat.is_char_boundary(start) {
                start += 1;
            }
            while !flat.is_char_boundary(end) {
                end -= 1;
            }
            (start, end)
        }
    };

    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str(ELLIPSIS);
    }
    snippet.push_str(&flat[start..end]);
    if end < flat.len() {
        snippet.push_str(ELLIPSIS);
    }
    Some(snippet)
}

pub fn build_content_snippet(text: &str, normalized_query: &str) -> Option<String> {
    build_content_snippet_with_context(text, normalized_query, SnippetContext::Characters)
}

pub(crate) fn build_content_snippet_with_byte_context(
    text: &str,
    normalized_query: &str,
) -> Option<String> {
    build_content_snippet_with_context(text, normalized_query, SnippetContext::Bytes)
}

pub fn find_agent_content_matches(
    entries: &[SearchableTranscriptEntry],
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
        if entry.hidden_outbound_agent_peer_message {
            continue;
        }
        let Some(snippet) = build_content_snippet(entry_search_text(entry), normalized_query) else {
            continue;
        };
        let role = match entry.kind {
            SearchableTranscriptEntryKind::Message { role, .. } => role,
            _ => AgentContentRole::Assistant,
        };
        matches.push(AgentContentMatch {
            entry_id: entry.id.clone(),
            role,
            timestamp_ms: entry.timestamp_ms.unwrap_or(0.0),
            snippet,
        });
    }
    matches
}

pub fn find_agent_content_matches_default(
    entries: &[SearchableTranscriptEntry],
    normalized_query: &str,
) -> Vec<AgentContentMatch> {
    find_agent_content_matches(
        entries,
        normalized_query,
        AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT,
    )
}
