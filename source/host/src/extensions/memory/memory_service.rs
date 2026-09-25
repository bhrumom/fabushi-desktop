use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use sha1::{Digest, Sha1};

use crate::watched_directory::{ChangeListener, WatchedDirectory};

pub const MEMORY_DIRNAME: &str = "memory";
pub const PROFILE_FILENAME: &str = "profile.md";
pub const LOG_DIRNAME: &str = "log";
pub const MEMORY_CHANGE_DEBOUNCE_MS: u64 = 50;
pub const MEMORY_PROFILE_PROMPT_LIMIT: usize = 100;
pub const MEMORY_MAX_CONTENT_LENGTH: usize = 500;

const FACT_PREFIX: &str = "- (";
const PROFILE_HEADER: &str =
    "# About the user\n\n<!-- Enduring facts, one per line as \"- (YYYY-MM-DD) <fact>\". -->\n\n";
const LOG_HEADER: &str =
    "# Memory log\n\n<!-- Dated facts, one per line as \"- (YYYY-MM-DD) <fact>\". -->\n\n";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRecord {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub kind: MemoryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryRecall {
    pub profile: Vec<MemoryRecord>,
    pub recent: Vec<MemoryRecord>,
}

#[derive(Debug, Clone)]
struct StoredMemoryFact {
    record: MemoryRecord,
    path: PathBuf,
    line: usize,
    order: usize,
}

#[derive(Debug, Clone)]
pub struct FileMemoryStore {
    dir: WatchedDirectory,
    profile_file: PathBuf,
    log_dir: PathBuf,
}

impl FileMemoryStore {
    pub fn new(memory_dir: impl Into<PathBuf>) -> Self {
        let memory_dir = memory_dir.into();
        Self {
            profile_file: memory_dir.join(PROFILE_FILENAME),
            log_dir: memory_dir.join(LOG_DIRNAME),
            dir: WatchedDirectory::new(memory_dir, MEMORY_CHANGE_DEBOUNCE_MS),
        }
    }

    pub fn get_location(&self) -> PathBuf {
        self.dir.get_location().to_path_buf()
    }

    pub fn set_on_change(&self, listener: Option<ChangeListener>) -> Result<(), String> {
        self.dir.set_on_change(listener)
    }

    pub fn recall(&self, recent_limit: usize) -> MemoryRecall {
        let mut facts = self.facts();
        facts.sort_by(|a, b| {
            b.record
                .created_at
                .cmp(&a.record.created_at)
                .then_with(|| b.order.cmp(&a.order))
        });
        MemoryRecall {
            profile: facts
                .iter()
                .filter(|fact| fact.record.kind == MemoryKind::Profile)
                .take(MEMORY_PROFILE_PROMPT_LIMIT)
                .map(|fact| fact.record.clone())
                .collect(),
            recent: facts
                .iter()
                .filter(|fact| fact.record.kind == MemoryKind::Log)
                .take(recent_limit)
                .map(|fact| fact.record.clone())
                .collect(),
        }
    }

    pub fn list_memories(&self, limit: usize) -> Vec<MemoryRecord> {
        let mut facts = self.facts();
        facts.sort_by(|a, b| {
            let a_profile = usize::from(a.record.kind == MemoryKind::Profile);
            let b_profile = usize::from(b.record.kind == MemoryKind::Profile);
            b_profile
                .cmp(&a_profile)
                .then_with(|| b.record.created_at.cmp(&a.record.created_at))
                .then_with(|| b.order.cmp(&a.order))
        });
        facts
            .into_iter()
            .take(limit)
            .map(|fact| fact.record)
            .collect()
    }

    pub fn count_memories(&self) -> usize {
        self.facts().len()
    }

    pub fn has_memories(&self) -> bool {
        self.count_memories() > 0
    }

    pub fn add_memory(
        &self,
        content: &str,
        created_at: i64,
        kind: MemoryKind,
    ) -> io::Result<Option<MemoryRecord>> {
        let normalized = normalize_memory_content(content);
        if normalized.is_empty() {
            return Ok(None);
        }
        let key = memory_dedupe_key(&normalized);
        if self
            .facts()
            .iter()
            .any(|fact| memory_dedupe_key(&fact.record.content) == key)
        {
            return Ok(None);
        }

        let path = match kind {
            MemoryKind::Profile => self.profile_file.clone(),
            MemoryKind::Log => self
                .log_dir
                .join(format!("{}.md", format_memory_date(created_at).chars().take(7).collect::<String>())),
        };
        let raw = fs::read_to_string(&path).unwrap_or_default();
        let header = match kind {
            MemoryKind::Profile => PROFILE_HEADER,
            MemoryKind::Log => LOG_HEADER,
        };
        let mut next = if raw.is_empty() { header.to_string() } else { raw };
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(&serialize_fact_line(&normalized, created_at));
        next.push('\n');
        self.dir.write_file_atomic(&path, next.as_bytes())?;

        Ok(Some(MemoryRecord {
            id: memory_id_for(&normalized),
            content: normalized,
            created_at,
            kind,
        }))
    }

    pub fn remove_memory_by_content(&self, content: &str) -> io::Result<bool> {
        let normalized = normalize_memory_content(content);
        if normalized.is_empty() {
            return Ok(false);
        }
        self.remove_memory(&memory_id_for(&normalized))
    }

    pub fn remove_memory(&self, id: &str) -> io::Result<bool> {
        let Some(fact) = self.facts().into_iter().find(|fact| fact.record.id == id) else {
            return Ok(false);
        };
        let raw = fs::read_to_string(&fact.path).unwrap_or_default();
        let mut lines = raw.split('\n').map(ToOwned::to_owned).collect::<Vec<_>>();
        if fact.line >= lines.len() {
            return Ok(false);
        }
        lines.remove(fact.line);
        self.dir.write_file_atomic(&fact.path, lines.join("\n").as_bytes())?;
        Ok(true)
    }

    pub fn clear_memories(&self) -> io::Result<()> {
        if !self.has_memories() {
            return Ok(());
        }
        match fs::remove_dir_all(&self.log_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        self.dir.write_file_atomic(&self.profile_file, PROFILE_HEADER.as_bytes())
    }

    fn facts(&self) -> Vec<StoredMemoryFact> {
        let mut facts = Vec::new();
        append_facts_from_file(
            &mut facts,
            &self.profile_file,
            MemoryKind::Profile,
        );

        let mut log_paths = match fs::read_dir(&self.log_dir) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().map(|kind| kind.is_file()).unwrap_or(false))
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
                .collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };
        log_paths.sort();
        for path in log_paths {
            append_facts_from_file(&mut facts, &path, MemoryKind::Log);
        }
        facts
    }
}

#[derive(Debug, Clone)]
pub struct MemoryService {
    agents_root_dir: PathBuf,
}

impl MemoryService {
    pub fn new(agents_root_dir: impl Into<PathBuf>) -> Self {
        Self {
            agents_root_dir: agents_root_dir.into(),
        }
    }

    pub fn create_agent_store(&self, agent_dir: impl AsRef<Path>) -> FileMemoryStore {
        FileMemoryStore::new(get_agent_memory_dir(agent_dir))
    }

    pub fn store_for_agent(&self, agent_id: &str) -> FileMemoryStore {
        self.create_agent_store(self.agents_root_dir.join(agent_id))
    }

    pub fn agent_has_content(&self, agent_dir: impl AsRef<Path>) -> bool {
        agent_memory_has_content(agent_dir)
    }

    pub fn agents_root_dir(&self) -> &Path {
        &self.agents_root_dir
    }
}

pub fn get_agent_memory_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(MEMORY_DIRNAME)
}

pub fn normalize_memory_content(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MEMORY_MAX_CONTENT_LENGTH)
        .collect()
}

pub fn memory_dedupe_key(content: &str) -> String {
    normalize_memory_content(content).to_lowercase()
}

pub fn memory_id_for(content: &str) -> String {
    let digest = Sha1::digest(memory_dedupe_key(content).as_bytes());
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        .chars()
        .take(16)
        .collect()
}

pub fn format_memory_date(created_at: i64) -> String {
    if created_at <= 0 {
        return "unknown date".into();
    }
    DateTime::<Utc>::from_timestamp_millis(created_at)
        .map(|value| value.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown date".into())
}

pub fn serialize_fact_line(content: &str, created_at: i64) -> String {
    format!("- ({}) {}", format_memory_date(created_at), content)
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

fn append_facts_from_file(
    output: &mut Vec<StoredMemoryFact>,
    path: &Path,
    kind: MemoryKind,
) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    for (line, text) in raw.split('\n').enumerate() {
        let Some(fact) = parse_fact_line(text, kind) else {
            continue;
        };
        let Some(created_at) = parse_memory_date_ms(&fact.date) else {
            continue;
        };
        let order = output.len();
        output.push(StoredMemoryFact {
            record: MemoryRecord {
                id: memory_id_for(&fact.content),
                content: fact.content,
                created_at,
                kind,
            },
            path: path.to_path_buf(),
            line,
            order,
        });
    }
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

fn parse_memory_date_ms(value: &str) -> Option<i64> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
    date.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis().into()
}

fn valid_memory_date(value: &str) -> bool {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}
