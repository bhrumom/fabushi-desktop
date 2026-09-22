use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::protocol::Failure;

pub const INFERENCE_TRANSCRIPT_SCHEMA_VERSION: u32 = 2;
pub const INFERENCE_TRANSCRIPT_LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceRoute {
    pub provider: String,
    pub host_slot: String,
}

#[derive(Debug, Default)]
pub struct InferenceRouter {
    by_agent: HashMap<String, InferenceRoute>,
    default: Option<InferenceRoute>,
}

impl InferenceRouter {
    pub fn set_default(&mut self, route: InferenceRoute) {
        self.default = Some(route);
    }

    pub fn clear_default(&mut self) {
        self.default = None;
    }

    pub fn bind_agent(&mut self, agent_id: impl Into<String>, route: InferenceRoute) {
        self.by_agent.insert(agent_id.into(), route);
    }

    pub fn unbind_agent(&mut self, agent_id: &str) {
        self.by_agent.remove(agent_id);
    }

    pub fn resolve(&self, agent_id: &str) -> Option<&InferenceRoute> {
        self.by_agent.get(agent_id).or(self.default.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoredRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredReaction {
    pub emoji: String,
    pub by: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredEntry {
    pub provider: String,
    pub role: StoredRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rich_text: Option<String>,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<StoredReaction>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptStore {
    pub schema_version: u32,
    pub agents: HashMap<String, Vec<StoredEntry>>,
}

impl Default for TranscriptStore {
    fn default() -> Self {
        Self {
            schema_version: INFERENCE_TRANSCRIPT_SCHEMA_VERSION,
            agents: HashMap::new(),
        }
    }
}

impl TranscriptStore {
    pub fn parse(value: Value) -> Self {
        let Some(root) = value.as_object() else {
            return Self::default();
        };
        if root.get("schemaVersion").and_then(Value::as_u64)
            != Some(INFERENCE_TRANSCRIPT_SCHEMA_VERSION as u64)
        {
            return Self::default();
        }
        let Some(agents) = root.get("agents").and_then(Value::as_object) else {
            return Self::default();
        };

        let mut parsed = HashMap::new();
        for (agent_id, raw_entries) in agents {
            let Some(rows) = raw_entries.as_array() else {
                continue;
            };
            let mut entries = rows
                .iter()
                .filter_map(|row| serde_json::from_value::<StoredEntry>(row.clone()).ok())
                .filter(valid_entry)
                .collect::<Vec<_>>();
            if entries.len() > INFERENCE_TRANSCRIPT_LIMIT {
                entries.drain(..entries.len() - INFERENCE_TRANSCRIPT_LIMIT);
            }
            parsed.insert(agent_id.clone(), entries);
        }
        Self {
            schema_version: INFERENCE_TRANSCRIPT_SCHEMA_VERSION,
            agents: parsed,
        }
    }

    pub fn append(&mut self, agent_id: &str, new_entries: impl IntoIterator<Item = StoredEntry>) {
        let entries = self.agents.entry(agent_id.to_string()).or_default();
        entries.extend(new_entries.into_iter().filter(valid_entry));
        if entries.len() > INFERENCE_TRANSCRIPT_LIMIT {
            entries.drain(..entries.len() - INFERENCE_TRANSCRIPT_LIMIT);
        }
    }

    pub fn entries(&self, agent_id: &str) -> &[StoredEntry] {
        self.agents.get(agent_id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn toggle_local_reaction(
        &mut self,
        agent_id: &str,
        entry_id: &str,
        emoji: &str,
    ) -> Option<&StoredEntry> {
        let emoji = emoji.trim();
        if emoji.is_empty() {
            return None;
        }
        let entry = self
            .agents
            .get_mut(agent_id)?
            .iter_mut()
            .find(|entry| entry.id == entry_id)?;
        if let Some(index) = entry
            .reactions
            .iter()
            .position(|reaction| reaction.emoji == emoji && reaction.by == "me")
        {
            entry.reactions.remove(index);
        } else {
            entry.reactions.push(StoredReaction {
                emoji: emoji.to_string(),
                by: "me".into(),
            });
        }
        Some(entry)
    }

    pub fn next_turn_number(
        &self,
        agent_id: &str,
        remote_entry_ids: impl IntoIterator<Item = String>,
    ) -> u64 {
        let remote_entry_ids = remote_entry_ids.into_iter().collect::<Vec<_>>();
        self.entries(agent_id)
            .iter()
            .map(|entry| entry.id.as_str())
            .chain(remote_entry_ids.iter().map(String::as_str))
            .filter_map(turn_number_from_id)
            .max()
            .map_or(0, |turn| turn.saturating_add(1))
    }
}

fn valid_entry(entry: &StoredEntry) -> bool {
    matches!(
        entry.provider.as_str(),
        "codex" | "claude-code" | "openrouter" | "fabushi"
    ) && !entry.id.trim().is_empty()
}

pub fn project_transcript_entry(entry: &StoredEntry) -> Value {
    match entry.role {
        StoredRole::User => {
            let mut value = json!({
                "kind": "message",
                "id": entry.id,
                "role": "user",
                "content": entry.content,
                "isStreaming": false,
                "timestampMs": entry.timestamp_ms,
            });
            if let Some(rich_text) = &entry.rich_text {
                value["richText"] = Value::String(rich_text.clone());
            }
            if let Some(client_nonce) = &entry.client_nonce {
                value["clientNonce"] = Value::String(client_nonce.clone());
            }
            if !entry.reactions.is_empty() {
                value["reactions"] = serde_json::to_value(&entry.reactions).unwrap_or(Value::Null);
            }
            value
        }
        StoredRole::Assistant => {
            let mut value = json!({
                "kind": "send-message",
                "id": entry.id,
                "message": {
                    "type": "text",
                    "content": entry.content,
                },
                "timestampMs": entry.timestamp_ms,
            });
            if !entry.reactions.is_empty() {
                value["reactions"] = serde_json::to_value(&entry.reactions).unwrap_or(Value::Null);
            }
            value
        }
    }
}

pub fn turn_number_from_id(id: &str) -> Option<u64> {
    let rest = id.strip_prefix('t')?;
    let digit_count = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count == 0 {
        return None;
    }
    let (digits, suffix) = rest.split_at(digit_count);
    let valid_suffix = suffix == "u"
        || suffix
            .strip_prefix('s')
            .is_some_and(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()));
    valid_suffix.then(|| digits.parse::<u64>().ok()).flatten()
}

#[derive(Debug, Clone)]
pub struct InferenceTranscriptFile {
    path: PathBuf,
}

impl InferenceTranscriptFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> TranscriptStore {
        let value = fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok());
        value.map(TranscriptStore::parse).unwrap_or_default()
    }

    pub fn persist(&self, store: &TranscriptStore) -> Result<(), Failure> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Failure::new(
                    "INFERENCE_STORE_WRITE_FAILED",
                    format!("could not create transcript directory: {error}"),
                )
            })?;
        }
        let temporary = self.path.with_extension(format!(
            "{}.tmp",
            Uuid::new_v4()
        ));
        let bytes = serde_json::to_vec_pretty(store).map_err(|error| {
            Failure::new(
                "INFERENCE_STORE_SERIALIZE_FAILED",
                format!("could not serialize transcript store: {error}"),
            )
        })?;
        fs::write(&temporary, bytes).map_err(|error| {
            Failure::new(
                "INFERENCE_STORE_WRITE_FAILED",
                format!("could not write temporary transcript: {error}"),
            )
        })?;
        fs::rename(&temporary, &self.path).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            Failure::new(
                "INFERENCE_STORE_RENAME_FAILED",
                format!("could not atomically replace transcript store: {error}"),
            )
        })
    }

    pub fn append(
        &self,
        agent_id: &str,
        entries: impl IntoIterator<Item = StoredEntry>,
    ) -> Result<TranscriptStore, Failure> {
        let mut store = self.load();
        store.append(agent_id, entries);
        self.persist(&store)?;
        Ok(store)
    }

    pub fn toggle_local_reaction(
        &self,
        agent_id: &str,
        entry_id: &str,
        emoji: &str,
    ) -> Result<Option<StoredEntry>, Failure> {
        let mut store = self.load();
        let updated = store
            .toggle_local_reaction(agent_id, entry_id, emoji)
            .cloned();
        if updated.is_some() {
            self.persist(&store)?;
        }
        Ok(updated)
    }
}
