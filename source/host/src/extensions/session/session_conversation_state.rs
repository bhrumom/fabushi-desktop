use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::storage::store_db::live_db_handle_count;

use super::agent_db_recovery::{AgentDbRecoveryError, DbRecoveryOptions, open_configured_db};
use super::agent_db_schema::{
    GET_TRANSCRIPT_ENTRY_SQL, LIST_BRANCHED_ENTRIES_SQL, LIST_TRANSCRIPT_ENTRIES_SQL,
};
use super::agent_db_serde::parse_transcript_entry;
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
    read_transcript_page, read_transcript_tail, read_transcript_window,
};

#[derive(Debug, thiserror::Error)]
pub enum SessionConversationStateError {
    #[error("session conversation-state database error: {0}")]
    Database(#[from] AgentDbRecoveryError),
    #[error("session conversation-state sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptThread {
    pub entries: Vec<Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionConversationState {
    busy_timeout_ms: u64,
}

impl SessionConversationState {
    pub fn new(busy_timeout_ms: u64) -> Self {
        Self { busy_timeout_ms }
    }

    fn open_read_db(&self, db_path: &Path) -> Result<Connection, SessionConversationStateError> {
        let agent_id = db_path
            .parent()
            .and_then(Path::file_name)
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let options = DbRecoveryOptions {
            recover_on_corruption: false,
            busy_timeout_ms: self.busy_timeout_ms,
            ..DbRecoveryOptions::default()
        };
        Ok(open_configured_db(
            db_path,
            &agent_id,
            &options,
            live_db_handle_count(db_path) > 0,
        )?)
    }

    pub fn read_agent_transcript_entries(
        &self,
        db_path: &Path,
    ) -> Result<Vec<Value>, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        read_entries(&db)
    }

    pub fn read_agent_transcript_page(
        &self,
        db_path: &Path,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        Ok(read_transcript_page(&db, query)?)
    }

    pub fn read_agent_transcript_window(
        &self,
        db_path: &Path,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<BTreeMap<String, usize>>, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        let branched = read_branched_entries(&db)?;
        let counts = branch_reply_counts(&branched);
        Ok(read_transcript_window(&db, query, |entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let id = entry.get("id").and_then(Value::as_str)?;
                    counts.get(id).copied().map(|count| (id.to_string(), count))
                })
                .collect()
        })?)
    }

    pub fn read_agent_transcript_tail(
        &self,
        db_path: &Path,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        Ok(read_transcript_tail(&db, query)?)
    }

    pub fn read_agent_thread(
        &self,
        db_path: &Path,
        root_id: &str,
    ) -> Result<TranscriptThread, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        let root_raw = db
            .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![root_id], |row| row.get::<_, String>(0))
            .optional()?;
        let root = root_raw.as_deref().and_then(parse_transcript_entry);
        let branched = read_branched_entries(&db)?;
        let mut entries = Vec::new();
        if let Some(root) = root {
            entries.push(root);
        }
        entries.extend(thread_descendants(root_id, &branched));
        Ok(TranscriptThread { entries })
    }
}

fn read_entries(db: &Connection) -> Result<Vec<Value>, SessionConversationStateError> {
    let mut statement = db.prepare(LIST_TRANSCRIPT_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        if let Some(entry) = parse_transcript_entry(&row?) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn read_branched_entries(db: &Connection) -> Result<Vec<Value>, SessionConversationStateError> {
    let mut statement = db.prepare(LIST_BRANCHED_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        if let Some(entry) = parse_transcript_entry(&row?) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn branch_reply_counts(branched: &[Value]) -> HashMap<String, usize> {
    let by_id = branched
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("id")?.as_str()?.to_string(),
                entry,
            ))
        })
        .collect::<HashMap<_, _>>();
    let mut counts = HashMap::new();
    for entry in branched {
        if let Some(root) = resolve_branch_root(entry, &by_id) {
            *counts.entry(root).or_insert(0) += 1;
        }
    }
    counts
}

fn thread_descendants(root_id: &str, branched: &[Value]) -> Vec<Value> {
    let by_id = branched
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("id")?.as_str()?.to_string(),
                entry,
            ))
        })
        .collect::<HashMap<_, _>>();
    branched
        .iter()
        .filter(|entry| resolve_branch_root(entry, &by_id).as_deref() == Some(root_id))
        .cloned()
        .collect()
}

fn resolve_branch_root(
    entry: &Value,
    branched_by_id: &HashMap<String, &Value>,
) -> Option<String> {
    let mut current = entry;
    let mut seen = std::collections::HashSet::new();
    if let Some(id) = current.get("id").and_then(Value::as_str) {
        seen.insert(id.to_string());
    }
    loop {
        let parent_id = current.get("replyTo").and_then(Value::as_str)?;
        let Some(parent) = branched_by_id.get(parent_id) else {
            return Some(parent_id.to_string());
        };
        if !seen.insert(parent_id.to_string()) {
            return None;
        }
        current = parent;
    }
}
