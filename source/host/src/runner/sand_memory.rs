use std::collections::HashSet;
use std::io;

use crate::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryRecall, MemoryRecord, format_memory_date,
    memory_dedupe_key, normalize_memory_content,
};

pub const MEMORY_RECENT_PROMPT_LIMIT: usize = 30;
pub const MEMORY_RECENT_PROMPT_CHAR_BUDGET: usize = 4_000;
pub const MEMORY_MAX_CONTENT_LENGTH: usize = 500;
pub const MEMORY_EXTRACTION_PROMPT_MARKER: &str = "<<SAND_MEMORY_EXTRACTION>>";
pub const MEMORY_EPISODE_PROMPT_MARKER: &str = "<<SAND_MEMORY_EPISODE>>";
pub const MEMORY_EPISODE_PREFIX: &str = "[episode] ";
pub const MEMORY_NOTE_PREFIX: &str = "[note] ";
pub const MEMORY_EXTRACTION_NONE_SENTINEL: &str = "NONE";
pub const MEMORY_EXTRACTION_ARCHIVE_SCAN_LIMIT: usize = 500;
pub const DEFAULT_EPISODE_INTERVAL: usize = 6;
pub const MEMORY_DECAY_HALF_LIFE_DAYS: f64 = 30.0;
pub const DAY_MS: f64 = 86_400_000.0;
pub const MEMORY_SYSTEM_PROMPT_HEADER: &str =
    "Memory: durable facts you have learned about the user and their world.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenMemorySnapshot {
    pub render: String,
    pub compaction_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenMemoryPrompt {
    pub render: String,
    pub snapshot_to_persist: Option<FrozenMemorySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAddition {
    pub content: String,
    pub kind: MemoryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryExtraction {
    pub additions: Vec<MemoryAddition>,
    pub removals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppliedMemoryExtraction {
    pub added: Vec<MemoryRecord>,
    pub removed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeTurn {
    pub ts: i64,
    pub user: String,
    pub agent: String,
}

pub fn is_memory_freeze_enabled(disable_memory_freeze: Option<&str>) -> bool {
    disable_memory_freeze != Some("1")
}

pub fn resolve_frozen_memory_prompt(
    snapshot: Option<&FrozenMemorySnapshot>,
    compaction_epoch: u64,
    live_render: impl FnOnce() -> (String, bool),
) -> FrozenMemoryPrompt {
    if let Some(snapshot) = snapshot.filter(|value| value.compaction_epoch == compaction_epoch) {
        return FrozenMemoryPrompt {
            render: snapshot.render.clone(),
            snapshot_to_persist: None,
        };
    }
    let (render, has_facts) = live_render();
    FrozenMemoryPrompt {
        snapshot_to_persist: has_facts.then(|| FrozenMemorySnapshot {
            render: render.clone(),
            compaction_epoch,
        }),
        render,
    }
}

pub fn episode_interval(raw: Option<&str>) -> usize {
    raw.and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_EPISODE_INTERVAL)
}

pub fn is_memorable_exchange(user_message: &str) -> bool {
    let user = user_message.trim();
    if user.is_empty() {
        return false;
    }
    if user.chars().count() > 40 || user.contains('?') {
        return true;
    }
    let normalized = user
        .trim_end_matches(|ch: char| ch.is_whitespace() || matches!(ch, '!' | '.' | '…' | ',' | '~' | ')' | ']'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    !matches!(
        normalized.as_str(),
        "hi" | "hey" | "hello" | "yo" | "sup" | "thanks" | "thank you" | "ty" | "thx"
            | "ok" | "okay" | "k" | "kk" | "cool" | "nice" | "great" | "awesome"
            | "perfect" | "yes" | "yep" | "yeah" | "no" | "nope" | "sure" | "got it"
            | "gotcha" | "lol" | "haha" | "np" | "done" | "good" | "bye"
    )
}

pub fn memory_importance(content: &str) -> f64 {
    if content.starts_with(MEMORY_EPISODE_PREFIX) {
        1.5
    } else if content.starts_with(MEMORY_NOTE_PREFIX) {
        0.5
    } else {
        1.0
    }
}

pub fn memory_recall_rank(memory: &MemoryRecord) -> f64 {
    memory_importance(&memory.content).log2()
        + memory.created_at as f64 / (MEMORY_DECAY_HALF_LIFE_DAYS * DAY_MS)
}

pub fn fact_line(memory: &MemoryRecord) -> String {
    format!(
        "- (learned {}) {}",
        format_memory_date(memory.created_at),
        memory.content
    )
}

pub fn render_memory_system_prompt(recall: &MemoryRecall, location: Option<&str>) -> String {
    if recall.profile.is_empty() && recall.recent.is_empty() && location.is_none() {
        return String::new();
    }

    let mut lines = vec![
        MEMORY_SYSTEM_PROMPT_HEADER.to_string(),
        "These persist across every conversation with this agent, even after the chat is cleared. Rely on them so you stay consistent and avoid re-asking what you already know.".to_string(),
    ];
    if let Some(location) = location {
        lines.push(format!(
            "Your memory lives in a folder at {location}: profile.md holds who the user is (kept in mind every turn) and log/ holds dated history."
        ));
        lines.push(
            "Read or grep those files with Read and Shell on your own computer when you need older facts that are not listed here. To CHANGE memory, prefer the update_state tool (target \"memory\"): action \"write\" with a fact and a tier (profile | log | note), or action \"forget\" with the exact text of a recorded fact."
                .to_string(),
        );
    }
    if !recall.profile.is_empty() {
        lines.push("About the user:".into());
        lines.extend(recall.profile.iter().map(fact_line));
    }
    if !recall.recent.is_empty() {
        lines.push("Recently:".into());
        let mut budget = MEMORY_RECENT_PROMPT_CHAR_BUDGET;
        let mut shown = 0usize;
        for memory in &recall.recent {
            let line = fact_line(memory);
            if shown > 0 && line.len() > budget {
                break;
            }
            budget = budget.saturating_sub(line.len());
            shown += 1;
            lines.push(line);
        }
        let omitted = recall.recent.len().saturating_sub(shown);
        if omitted > 0 {
            lines.push(match location {
                Some(_) => format!(
                    "({omitted} more log facts on disk — grep the log/ folder for them.)"
                ),
                None => format!("({omitted} more log facts not shown.)"),
            });
        }
    }
    if recall.profile.is_empty() && recall.recent.is_empty() {
        lines.push("No facts recorded yet.".into());
    }
    lines.join("\n")
}

pub fn build_extraction_system_prompt() -> String {
    [
        MEMORY_EXTRACTION_PROMPT_MARKER,
        "You maintain the long-term memory of a personal assistant. Read the latest exchange and decide what — if anything — is worth remembering for future, unrelated conversations.",
        "",
        "Tag each fact you keep with a category:",
        "- \"profile\": enduring facts about who the user is and how to work with them — their name and how to address them, role, location, languages, lasting preferences and constraints, and important people or relationships. These are remembered indefinitely.",
        "- \"log\": substantive history worth keeping — ongoing projects and tasks, decisions, commitments, and time-bound details.",
        "- \"note\": minor, low-stakes details that might help someday but are not worth keeping in mind every turn (small one-off preferences, incidental context). Notes fade from the always-visible list fastest but stay on disk.",
        "",
        "Do NOT record one-off request mechanics, what the assistant did this turn, general knowledge, or anything already present in the existing memory list.",
        "",
        "If the new exchange updates or contradicts a fact in the existing memory list (e.g. the user moved, changed jobs, or renamed something), drop anything clearly superseded: output a line \"remove: <the exact existing fact text>\" and then add the corrected fact. Only remove facts that appear verbatim in the existing list — never invent removals.",
        "",
        "Write each fact as a self-contained statement, one per line: \"profile: <fact>\", \"log: <fact>\", or \"note: <fact>\" to add, or \"remove: <existing fact>\" to drop a superseded one.",
        "Output exactly NONE (and nothing else) when there is nothing to add or remove.",
    ]
    .join("\n")
}

pub fn build_extraction_user_prompt(
    user_message: &str,
    agent_message: &str,
    existing_memories: &[String],
) -> String {
    let existing = if existing_memories.is_empty() {
        "(empty)".to_string()
    } else {
        existing_memories
            .iter()
            .map(|memory| format!("- {memory}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "Existing memory:\n{existing}\n\nLatest exchange:\nUser: {}\nAssistant: {}",
        non_empty_or(user_message.trim(), "(no message)"),
        non_empty_or(agent_message.trim(), "(no message)")
    )
}

pub fn parse_extracted_memories(raw: &str, existing_memories: &[String]) -> MemoryExtraction {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(MEMORY_EXTRACTION_NONE_SENTINEL) {
        return MemoryExtraction::default();
    }
    let mut seen = existing_memories
        .iter()
        .map(|memory| memory_dedupe_key(memory))
        .collect::<HashSet<_>>();
    let mut extraction = MemoryExtraction::default();

    for raw_line in trimmed.lines() {
        let stripped = strip_list_prefix(raw_line.trim());
        let (tag, value) = split_category(stripped);
        let bare = normalize_memory_content(value);
        if bare.is_empty() || bare.eq_ignore_ascii_case(MEMORY_EXTRACTION_NONE_SENTINEL) {
            continue;
        }
        if tag == Some("remove") {
            extraction.removals.push(bare);
            continue;
        }
        let content = if tag == Some("note") {
            normalize_memory_content(&format!("{MEMORY_NOTE_PREFIX}{bare}"))
        } else {
            bare
        };
        let key = memory_dedupe_key(&content);
        if !seen.insert(key) {
            continue;
        }
        extraction.additions.push(MemoryAddition {
            content,
            kind: if tag == Some("profile") {
                MemoryKind::Profile
            } else {
                MemoryKind::Log
            },
        });
    }
    extraction
}

pub fn select_relevant_memories(
    query: &str,
    memories: &[MemoryRecord],
    max: usize,
) -> Vec<MemoryRecord> {
    if max == 0 || memories.is_empty() {
        return Vec::new();
    }
    let query_tokens = relevance_tokens(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }
    let mut scored = memories
        .iter()
        .filter_map(|memory| {
            let overlap = relevance_tokens(&memory.content)
                .iter()
                .filter(|token| query_tokens.contains(*token))
                .count();
            (overlap > 0).then_some((memory.clone(), overlap))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|(left, left_overlap), (right, right_overlap)| {
        right_overlap
            .cmp(left_overlap)
            .then_with(|| right.created_at.cmp(&left.created_at))
    });
    scored
        .into_iter()
        .take(max)
        .map(|(memory, _)| memory)
        .collect()
}

pub fn gather_extraction_memories(
    recall: &MemoryRecall,
    archive: &[MemoryRecord],
    exchange_text: &str,
) -> Vec<String> {
    let mut in_prompt = recall.profile.clone();
    in_prompt.extend(recall.recent.clone());
    let seen = in_prompt
        .iter()
        .map(|memory| memory_dedupe_key(&memory.content))
        .collect::<HashSet<_>>();
    let candidates = archive
        .iter()
        .filter(|memory| !seen.contains(&memory_dedupe_key(&memory.content)))
        .cloned()
        .collect::<Vec<_>>();
    in_prompt
        .into_iter()
        .chain(select_relevant_memories(exchange_text, &candidates, 10))
        .map(|memory| memory.content)
        .collect()
}

pub fn apply_extracted_memories(
    store: &FileMemoryStore,
    extraction: &MemoryExtraction,
    now_ms: i64,
    known_memories: &[String],
) -> io::Result<AppliedMemoryExtraction> {
    let known = known_memories
        .iter()
        .map(|memory| memory_dedupe_key(memory))
        .collect::<HashSet<_>>();
    let mut result = AppliedMemoryExtraction::default();
    for removal in &extraction.removals {
        if known.contains(&memory_dedupe_key(removal))
            && store.remove_memory_by_content(removal)?
        {
            result.removed.push(removal.clone());
        }
    }
    for addition in &extraction.additions {
        if let Some(record) = store.add_memory(&addition.content, now_ms, addition.kind)? {
            result.added.push(record);
        }
    }
    Ok(result)
}

pub fn build_episode_system_prompt() -> String {
    [
        MEMORY_EPISODE_PROMPT_MARKER,
        "You maintain the long-term memory of a personal desktop assistant named Grok Bot.",
        "You are given the most recent turns of a conversation between the user and Grok Bot, in order, each tagged with its date.",
        "Write ONE short journal-style sentence (two at most) capturing what the user and Grok Bot were actually working on across these turns — the throughline, key decisions, and outcomes — so it stays useful months from now.",
        "Anchor any time references with the absolute dates shown, never relative words like \"yesterday\". Drop greetings, acknowledgements, and anything ephemeral. Never invent details.",
        "Output just the sentence(s), no preamble or bullets. Output exactly NONE if nothing in this stretch is worth remembering.",
    ]
    .join("\n")
}

pub fn build_episode_user_prompt(turns: &[EpisodeTurn]) -> String {
    let rendered = turns
        .iter()
        .map(|turn| {
            let mut lines = vec![format!("({})", format_memory_date(turn.ts))];
            if !turn.user.trim().is_empty() {
                lines.push(format!("User: {}", turn.user.trim()));
            }
            if !turn.agent.trim().is_empty() {
                lines.push(format!("Grok Bot: {}", turn.agent.trim()));
            }
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("Recent turns, oldest first:\n\n{rendered}")
}

fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

fn split_category(line: &str) -> (Option<&'static str>, &str) {
    for tag in ["profile", "log", "note", "remove"] {
        if line.len() > tag.len()
            && line[..tag.len()].eq_ignore_ascii_case(tag)
            && line.as_bytes().get(tag.len()) == Some(&b':')
        {
            return (Some(tag), line[tag.len() + 1..].trim());
        }
    }
    (None, line)
}

fn strip_list_prefix(line: &str) -> &str {
    let line = line.trim_start();
    if let Some(rest) = line.strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("• "))
    {
        return rest.trim_start();
    }
    let bytes = line.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index > 0
        && index + 1 < bytes.len()
        && matches!(bytes[index], b'.' | b')')
        && bytes[index + 1].is_ascii_whitespace()
    {
        return line[index + 2..].trim_start();
    }
    line
}

fn relevance_tokens(text: &str) -> HashSet<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|token| token.chars().count() >= 4 && !is_relevance_stopword(token))
        .collect()
}

fn is_relevance_stopword(token: &str) -> bool {
    matches!(
        token,
        "that" | "this" | "with" | "from" | "they" | "them" | "then" | "than"
            | "what" | "when" | "where" | "which" | "will" | "would" | "could"
            | "should" | "have" | "been" | "being" | "about" | "just" | "like"
            | "your" | "does" | "were" | "also" | "into" | "over" | "only"
            | "some" | "more" | "most" | "very" | "much" | "here" | "there"
            | "their" | "these" | "those" | "because" | "while" | "after"
            | "before" | "user"
    )
}
