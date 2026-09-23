use std::fs;
use std::path::{Path, PathBuf};

pub const MEMORY_DIRNAME: &str = "memory";
pub const PROFILE_FILENAME: &str = "profile.md";
pub const LOG_DIRNAME: &str = "log";
pub const MEMORY_CHANGE_DEBOUNCE_MS: u64 = 50;

const FACT_PREFIX: &str = "- (";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Profile,
    Log,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryFact {
    pub date: String,
    pub content: String,
    pub kind: MemoryKind,
}

pub fn get_agent_memory_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(MEMORY_DIRNAME)
}

pub fn normalize_memory_content(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(500)
        .collect()
}

pub fn parse_facts(raw: &str, kind: MemoryKind) -> Vec<MemoryFact> {
    raw.lines()
        .filter_map(|line| parse_fact_line(line, kind))
        .collect()
}

pub fn agent_memory_has_content(agent_dir: impl AsRef<Path>) -> bool {
    let memory_dir = get_agent_memory_dir(agent_dir);
    let profile_path = memory_dir.join(PROFILE_FILENAME);
    if read_valid_facts(&profile_path, MemoryKind::Profile)
        .is_some_and(|facts| !facts.is_empty())
    {
        return true;
    }

    let log_dir = memory_dir.join(LOG_DIRNAME);
    let Ok(entries) = fs::read_dir(log_dir) else {
        return false;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                == Some("md")
        })
        .any(|entry| {
            read_valid_facts(&entry.path(), MemoryKind::Log)
                .is_some_and(|facts| !facts.is_empty())
        })
}

fn read_valid_facts(path: &Path, kind: MemoryKind) -> Option<Vec<MemoryFact>> {
    let raw = fs::read_to_string(path).ok()?;
    Some(parse_facts(&raw, kind))
}

fn parse_fact_line(line: &str, kind: MemoryKind) -> Option<MemoryFact> {
    let line = line.trim();
    let rest = line.strip_prefix(FACT_PREFIX)?;
    let (date, content) = rest.split_once(") ")?;
    if !valid_memory_date(date) {
        return None;
    }
    let content = normalize_memory_content(content);
    if content.is_empty() {
        return None;
    }
    Some(MemoryFact {
        date: date.to_string(),
        content,
        kind,
    })
}

fn valid_memory_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        && value[5..7].parse::<u8>().is_ok_and(|month| (1..=12).contains(&month))
        && value[8..10].parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
}
