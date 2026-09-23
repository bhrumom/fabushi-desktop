use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, mpsc};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::protocol::Failure;

pub const INFERENCE_TRANSCRIPT_SCHEMA_VERSION: u32 = 2;
pub const INFERENCE_TRANSCRIPT_LIMIT: usize = 200;

/// Normalize renderer-facing transcript aliases before crossing the Host boundary.
/// The Rust Session gateway owns `getAgentTranscriptTail`; `openAgentTail`
/// remains a renderer/Coordinator convenience alias and must never leak to Host.
pub fn host_transcript_method(method: &str) -> &str {
    match method {
        "openAgentTail" => "getAgentTranscriptTail",
        other => other,
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceProvider {
    Cursor,
    Codex,
    ClaudeCode,
    OpenRouter,
}

impl InferenceProvider {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cursor" => Some(Self::Cursor),
            "codex" => Some(Self::Codex),
            "claude-code" => Some(Self::ClaudeCode),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::OpenRouter => "openrouter",
        }
    }
}

pub fn configured_inference_provider(settings_path: &Path) -> Option<InferenceProvider> {
    let value: Value =
        serde_json::from_str(&fs::read_to_string(settings_path).ok()?).ok()?;
    [
        value.get("inferenceProvider"),
        value.get("inference_provider"),
        value.get("router").and_then(|router| router.get("provider")),
    ]
    .into_iter()
    .flatten()
    .find_map(|value| value.as_str().and_then(InferenceProvider::parse))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerInferenceEvent {
    Delta { content: String },
    Completed { content: String },
    Failed { message: String },
    Cancelled { message: String },
}

pub fn parse_runner_inference_event(
    value: &Value,
) -> Result<(String, RunnerInferenceEvent), Failure> {
    let stream_id = value
        .get("streamId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Failure::new(
            "INFERENCE_RUNNER_EVENT_INVALID",
            "Runner inference event is missing streamId",
        ))?
        .to_string();
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("");
    let event = match event_type {
        "delta" => RunnerInferenceEvent::Delta {
            content: value
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        },
        "completed" => RunnerInferenceEvent::Completed {
            content: value
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        },
        "failed" => RunnerInferenceEvent::Failed {
            message: value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Runner inference failed")
                .to_string(),
        },
        "cancelled" => RunnerInferenceEvent::Cancelled {
            message: value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Runner inference cancelled")
                .to_string(),
        },
        other => {
            return Err(Failure::new(
                "INFERENCE_RUNNER_EVENT_INVALID",
                format!("unknown Runner inference event type: {other}"),
            ));
        }
    };
    Ok((stream_id, event))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceRoute {
    pub provider: String,
    pub host_slot: String,
}

type InferenceTask = Box<dyn FnOnce() + Send + 'static>;

#[derive(Default)]
pub struct InferenceTaskQueue {
    workers: Mutex<HashMap<String, mpsc::Sender<InferenceTask>>>,
}

impl InferenceTaskQueue {
    fn spawn_worker(agent_id: &str) -> Result<mpsc::Sender<InferenceTask>, Failure> {
        let (sender, receiver) = mpsc::channel::<InferenceTask>();
        thread::Builder::new()
            .name(format!("inference-router-{agent_id}"))
            .spawn(move || {
                while let Ok(task) = receiver.recv() {
                    let _ = catch_unwind(AssertUnwindSafe(task));
                }
            })
            .map_err(|error| {
                Failure::new(
                    "INFERENCE_QUEUE_SPAWN_FAILED",
                    format!("could not start inference queue worker: {error}"),
                )
            })?;
        Ok(sender)
    }

    pub fn enqueue<F>(&self, agent_id: &str, task: F) -> Result<(), Failure>
    where
        F: FnOnce() + Send + 'static,
    {
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err(Failure::new(
                "INFERENCE_QUEUE_AGENT_REQUIRED",
                "local inference routing requires a non-empty agentId",
            ));
        }
        let mut task: InferenceTask = Box::new(task);
        let mut workers = self.workers.lock().map_err(|_| {
            Failure::new(
                "INFERENCE_QUEUE_LOCK_FAILED",
                "inference queue lock poisoned",
            )
        })?;

        if let Some(sender) = workers.get(agent_id) {
            match sender.send(task) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    task = error.0;
                    workers.remove(agent_id);
                }
            }
        }

        let sender = Self::spawn_worker(agent_id)?;
        sender.send(task).map_err(|error| {
            Failure::new(
                "INFERENCE_QUEUE_DISCONNECTED",
                format!("inference queue worker stopped before enqueue: {error}"),
            )
        })?;
        workers.insert(agent_id.to_string(), sender);
        Ok(())
    }

    pub fn worker_count(&self) -> usize {
        self.workers
            .lock()
            .map(|workers| workers.len())
            .unwrap_or_default()
    }

    pub fn dispose(&self) {
        if let Ok(mut workers) = self.workers.lock() {
            workers.clear();
        }
    }
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

#[derive(Debug)]
pub struct CoordinatorInferenceRouter {
    settings_path: PathBuf,
    routes: Mutex<InferenceRouter>,
}

impl CoordinatorInferenceRouter {
    pub fn new(settings_path: impl Into<PathBuf>) -> Self {
        Self {
            settings_path: settings_path.into(),
            routes: Mutex::new(InferenceRouter::default()),
        }
    }

    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    pub fn resolve(&self, agent_id: &str) -> InferenceRoute {
        let configured = configured_inference_provider(&self.settings_path)
            .unwrap_or(InferenceProvider::Cursor);
        let fallback = InferenceRoute {
            provider: configured.as_str().to_string(),
            host_slot: "host".into(),
        };
        let Ok(mut routes) = self.routes.lock() else {
            return fallback;
        };
        routes.set_default(fallback.clone());
        routes.resolve(agent_id).cloned().unwrap_or(fallback)
    }

    pub fn bind_agent(
        &self,
        agent_id: impl Into<String>,
        route: InferenceRoute,
    ) -> Result<(), Failure> {
        let mut routes = self.routes.lock().map_err(|_| {
            Failure::new(
                "INFERENCE_ROUTER_LOCK_FAILED",
                "Coordinator inference router lock poisoned",
            )
        })?;
        routes.bind_agent(agent_id, route);
        Ok(())
    }

    pub fn unbind_agent(&self, agent_id: &str) -> Result<(), Failure> {
        let mut routes = self.routes.lock().map_err(|_| {
            Failure::new(
                "INFERENCE_ROUTER_LOCK_FAILED",
                "Coordinator inference router lock poisoned",
            )
        })?;
        routes.unbind_agent(agent_id);
        Ok(())
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAttachment {
    pub path: String,
    pub name: String,
}

pub fn parse_send_prompt_attachments(args: &Value) -> Result<Vec<StoredAttachment>, Failure> {
    let paths = match args.get("attachmentPaths") {
        None => &[][..],
        Some(Value::Array(paths)) => paths.as_slice(),
        Some(_) => {
            return Err(Failure::new(
                "INFERENCE_ROUTER_INVALID_ATTACHMENTS",
                "attachmentPaths must be an array",
            ));
        }
    };
    let names = match args.get("attachmentNames") {
        None => &[][..],
        Some(Value::Array(names)) => names.as_slice(),
        Some(_) => {
            return Err(Failure::new(
                "INFERENCE_ROUTER_INVALID_ATTACHMENTS",
                "attachmentNames must be an array",
            ));
        }
    };
    if paths.is_empty() && names.is_empty() {
        return Ok(Vec::new());
    }
    if paths.len() != names.len() {
        return Err(Failure::new(
            "INFERENCE_ROUTER_INVALID_ATTACHMENTS",
            "attachmentPaths and attachmentNames must have the same length",
        ));
    }
    paths
        .iter()
        .zip(names)
        .map(|(path, name)| {
            let path = path.as_str().unwrap_or("").trim();
            let name = name.as_str().unwrap_or("").trim();
            if path.is_empty() || name.is_empty() {
                return Err(Failure::new(
                    "INFERENCE_ROUTER_INVALID_ATTACHMENTS",
                    "attachment paths and names must be non-empty strings",
                ));
            }
            Ok(StoredAttachment {
                path: path.to_string(),
                name: name.to_string(),
            })
        })
        .collect()
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
    pub attachments: Vec<StoredAttachment>,
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
            if !entry.attachments.is_empty() {
                value["attachments"] =
                    serde_json::to_value(&entry.attachments).unwrap_or(Value::Null);
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)).map_err(
                |error| {
                    Failure::new(
                        "INFERENCE_STORE_WRITE_FAILED",
                        format!("could not secure temporary transcript: {error}"),
                    )
                },
            )?;
        }
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
