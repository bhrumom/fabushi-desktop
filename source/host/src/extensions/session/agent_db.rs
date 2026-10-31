use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::sqlite_busy::{is_sqlite_busy_error, is_sqlite_corrupt_error};
use crate::storage::store_db::{
    bump_db_write_generation, live_db_handle_count, register_live_db_handle,
    release_live_db_handle,
};
use crate::transcript_mutation_events::publish_transcript_mutation;

use super::agent_db_recovery::{
    AgentDbRecoveryError, DbRecoveryOptions, open_configured_db, recover_corrupt_store_db,
};
use super::agent_db_schema::{
    CLEAR_BLOBS_SQL, CLEAR_TRANSCRIPT_ENTRIES_SQL, COMPARE_AND_SET_KV_SQL, DELETE_KV_SQL,
    DELETE_TRANSCRIPT_ENTRY_SQL, GET_KV_SQL, GET_TRANSCRIPT_ENTRY_SQL, HAS_LEGACY_BLOB_SQL,
    INSERT_TRANSCRIPT_ENTRY_SQL, LIST_BRANCHED_ENTRIES_SQL, LIST_TRANSCRIPT_ENTRIES_SQL,
    NEWEST_DIVIDER_ANCHOR_TIMESTAMP_SQL, SET_KV_SQL, UPDATE_TRANSCRIPT_ENTRY_SQL,
};
use super::agent_db_serde::{
    AwaitingUserResponse, EPISODE_PENDING_MAX, EPISODE_TURN_TEXT_CAP, EpisodeTurn,
    MemoryPromptSnapshot, REQUEST_ID_HISTORY_MAX, REQUEST_ID_PROMPT_MAX, RequestRecord,
    SandProfile, SpendGuardState, UnreadState, parse_awaiting_state,
    parse_memory_prompt_snapshot, parse_pending_episode_turns, parse_profile,
    parse_request_records, parse_transcript_entry, parse_unread_state,
    resolve_spend_guard_state, serialize_spend_guard_state,
};
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
    read_transcript_page, read_transcript_tail, read_transcript_window,
};

#[derive(Debug, thiserror::Error)]
pub enum AgentDbProjectionError {
    #[error("agent session database open/recovery error: {0}")]
    Recovery(#[from] AgentDbRecoveryError),
    #[error("agent session sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("agent metadata hex is invalid: {0}")]
    MetadataHex(String),
    #[error("agent metadata json is invalid utf-8: {0}")]
    MetadataUtf8(#[from] std::string::FromUtf8Error),
    #[error("agent metadata json is invalid: {0}")]
    MetadataJson(#[from] serde_json::Error),
    #[error("agent lifecycle revision unavailable: {0}")]
    LifecycleRevisionUnavailable(String),
    #[error("agent compaction entry is invalid: {0}")]
    CompactionInvalid(String),
    #[error("agent compaction entry conflicts with an existing identity: {0}")]
    CompactionConflict(String),
    #[error("agent session work-admission record is invalid: {0}")]
    WorkAdmissionInvalid(String),
    #[error("latestRootBlobId hex is invalid: {0}")]
    LatestRootHex(String),
    #[error("agent db owner mutex poisoned")]
    OwnerPoisoned,
}

const KV_PROFILE: &str = "sandProfile";
const KV_UNREAD: &str = "unreadState";
const KV_AWAITING: &str = "awaitingUserResponse";
const KV_REQUEST_IDS: &str = "requestIds";
const KV_LATEST_REQUEST_ID: &str = "latestRequestId";
const KV_EPISODE: &str = "episodePending";
const KV_MEMORY_SNAPSHOT: &str = "memoryPromptSnapshot";
const KV_SPEND_GUARD: &str = "automationSpendGuardState";
const KV_SPEND_GUARD_LEGACY: &str = "automationSpendGuardNudgedAt";
const KV_HIDDEN_REPAIR: &str = "hiddenEntryRepairVersion";
const KV_STALE_ROOT_CLEANUP: &str = "staleRootCleanupVersion";
const KV_LEGACY_BLOB_RETIREMENT: &str = "legacyStoreBlobRetirementVersion";
const KV_PROFILE_SNAPSHOT: &str = "agentProfilePromptSnapshot";
const KV_ORIGIN: &str = "origin";
const KV_INTRODUCTION: &str = "introductionPending";
const KV_PURPOSE: &str = "purpose";
const KV_PARTNERS: &str = "conversationPartners";
const KV_SESSION_WORK_ADMISSION: &str = "sessionWorkAdmission";

#[derive(Debug, Clone, PartialEq)]
pub struct AgentMetadataProjection {
    pub agent_id: String,
    pub created_at: f64,
    pub latest_root_blob_id: Vec<u8>,
    pub lifecycle_revision: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentDbSerdeSnapshot {
    pub profile: SandProfile,
    pub unread_state: UnreadState,
    pub spend_guard_state: SpendGuardState,
    pub awaiting_user_response: Option<AwaitingUserResponse>,
    pub request_ids: Vec<RequestRecord>,
    pub pending_episode_turns: Vec<EpisodeTurn>,
    pub memory_prompt_snapshot: Option<MemoryPromptSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompactionCommit {
    pub entry_id: String,
    pub first_kept_entry_id: Option<String>,
    pub summary: String,
    pub tokens_before: u64,
    pub details: Option<serde_json::Value>,
    pub source_turn_id: String,
    pub expected_lifecycle_revision: String,
}

/// Durable generation and lifecycle facts used by the session work-admission
/// owner. This is intentionally agent-DB local until the full session catalog
/// migration exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionWorkAdmissionRecord {
    pub session_key: String,
    pub session_id: String,
    pub lifecycle_revision: String,
    #[serde(default)]
    pub archived_at_ms: Option<f64>,
    #[serde(default)]
    pub archived_by: Option<String>,
    #[serde(default)]
    pub archive_reason: Option<String>,
    #[serde(default)]
    pub admitted_work_id: Option<String>,
    /// Durable fence for materialized outgoing media that has not yet reached
    /// the transcript owner. This is separate from background work because a
    /// foreground turn may own both identities at once.
    #[serde(default)]
    pub admitted_media_work_id: Option<String>,
    /// Absolute paths materialized for the fenced media work. This is durable
    /// recovery input; it is not itself permission to publish or delete media.
    #[serde(default)]
    pub admitted_media_paths: Vec<String>,
    #[serde(default)]
    pub terminal_result_idempotency_key: Option<String>,
    #[serde(default)]
    pub provider_review_id: Option<String>,
    #[serde(default)]
    pub provider_review_phase: Option<String>,
    #[serde(default)]
    pub provider_review: Option<SessionProviderReview>,
}

/// Provider-authored refusal facts retained by the session admission owner.
/// This record contains no continuation capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionProviderReview {
    pub id: String,
    pub session_id: String,
    pub run_id: String,
    pub provider: String,
    pub model: String,
    pub runtime_id: String,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub native_thread_id: Option<String>,
    #[serde(default)]
    pub native_turn_id: Option<String>,
    pub explanation: String,
    #[serde(default)]
    pub continuation_message: Option<String>,
    #[serde(default)]
    pub error_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionWorkResultReceipt {
    pub session_key: String,
    pub lifecycle_revision: String,
    pub work_id: String,
    pub idempotency_key: String,
    pub content: String,
    pub sent_message_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionWorkResultCommitOutcome {
    Committed(SessionWorkResultReceipt),
    AlreadyCommitted(SessionWorkResultReceipt),
    RejectedStale,
    ConflictingResult,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompactionCommitOutcome {
    Committed(serde_json::Value),
    AlreadyCommitted(serde_json::Value),
    RejectedStale,
}

pub type AgentDbListener = Arc<dyn Fn() + Send + Sync + 'static>;
pub type AgentDbBusyCallback = Arc<dyn Fn(&str, &str) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SandAgentDbOptions {
    pub recovery: DbRecoveryOptions,
    pub on_busy_error: Option<AgentDbBusyCallback>,
}

impl Default for SandAgentDbOptions {
    fn default() -> Self {
        Self {
            recovery: DbRecoveryOptions::default(),
            on_busy_error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AgentDbListenerChannel {
    Metadata(String),
    Profile,
    Awaiting,
}

type AgentDbListenerKey = (PathBuf, AgentDbListenerChannel);
type AgentDbListenerMap = HashMap<AgentDbListenerKey, BTreeMap<u64, AgentDbListener>>;

fn agent_db_listener_registry() -> &'static Mutex<AgentDbListenerMap> {
    static REGISTRY: OnceLock<Mutex<AgentDbListenerMap>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_agent_db_listener_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn resolve_agent_db_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn notify_agent_db_listeners(
    db_path: &Path,
    channel: AgentDbListenerChannel,
) {
    let key = (resolve_agent_db_path(db_path), channel);
    let listeners = agent_db_listener_registry()
        .lock()
        .ok()
        .and_then(|registry| {
            registry
                .get(&key)
                .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
        })
        .unwrap_or_default();
    for listener in listeners {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| listener()));
    }
}

pub struct AgentDbSubscription {
    key: AgentDbListenerKey,
    id: u64,
    active: bool,
}

impl AgentDbSubscription {
    pub fn unsubscribe(mut self) {
        self.remove();
    }

    fn remove(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut registry) = agent_db_listener_registry().lock() {
            if let Some(listeners) = registry.get_mut(&self.key) {
                listeners.remove(&self.id);
                if listeners.is_empty() {
                    registry.remove(&self.key);
                }
            }
        }
        self.active = false;
    }
}

impl Drop for AgentDbSubscription {
    fn drop(&mut self) {
        self.remove();
    }
}

fn subscribe_agent_db_listener(
    db_path: &Path,
    channel: AgentDbListenerChannel,
    listener: AgentDbListener,
) -> AgentDbSubscription {
    let key = (resolve_agent_db_path(db_path), channel);
    let id = next_agent_db_listener_id();
    agent_db_listener_registry()
        .lock()
        .expect("agent db listener registry poisoned")
        .entry(key.clone())
        .or_default()
        .insert(id, listener);
    AgentDbSubscription {
        key,
        id,
        active: true,
    }
}

/// Long-lived Rust owner for the frozen Grok SandAgentDb lifecycle boundary.
///
/// Production keeps one owner per live agent. The held SQLite connection
/// registers the live-handle fence used by corruption recovery and maintenance,
/// while existing projection helpers remain the single persistence implementation.
pub struct SandAgentDb {
    db_path: PathBuf,
    agent_dir_name: String,
    options: SandAgentDbOptions,
    connection: Mutex<Option<rusqlite::Connection>>,
    recovered: AtomicBool,
    handle_registered: AtomicBool,
    closed: AtomicBool,
}

impl SandAgentDb {
    pub fn open(
        db_path: impl AsRef<Path>,
        busy_timeout_ms: u64,
    ) -> Result<Self, AgentDbProjectionError> {
        let mut options = SandAgentDbOptions::default();
        options.recovery.busy_timeout_ms = busy_timeout_ms;
        Self::open_with_options(db_path, options)
    }

    pub fn open_conversation_store(
        db_path: impl AsRef<Path>,
        busy_timeout_ms: u64,
    ) -> Result<Self, AgentDbProjectionError> {
        let mut options = SandAgentDbOptions::default();
        options.recovery.busy_timeout_ms = busy_timeout_ms;
        Self::open_with_options_and_seed(db_path, options, false)
    }

    pub fn open_with_options(
        db_path: impl AsRef<Path>,
        options: SandAgentDbOptions,
    ) -> Result<Self, AgentDbProjectionError> {
        Self::open_with_options_and_seed(db_path, options, true)
    }

    fn open_with_options_and_seed(
        db_path: impl AsRef<Path>,
        options: SandAgentDbOptions,
        seed_default_agent_metadata: bool,
    ) -> Result<Self, AgentDbProjectionError> {
        let db_path = resolve_agent_db_path(db_path.as_ref());
        let agent_dir_name = db_path
            .parent()
            .and_then(Path::file_name)
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let connection = open_configured_db(
            &db_path,
            &agent_dir_name,
            &options.recovery,
            live_db_handle_count(&db_path) > 0,
        )?;
        register_live_db_handle(&db_path);
        let owner = Self {
            db_path,
            agent_dir_name,
            options,
            connection: Mutex::new(Some(connection)),
            recovered: AtomicBool::new(false),
            handle_registered: AtomicBool::new(true),
            closed: AtomicBool::new(false),
        };
        if seed_default_agent_metadata {
            if let Err(error) = owner.seed_default_metadata_if_missing() {
                owner.close(false);
                return Err(error);
            }
        }
        Ok(owner)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn busy_timeout_ms(&self) -> u64 {
        self.options.recovery.busy_timeout_ms
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub fn read_kv(&self, key: &str) -> Result<Option<String>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(None);
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(None);
        };
        Ok(read_kv(db, key)?)
    }

    pub fn write_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        self.run_write(&format!("writeKv:{key}"), |db| {
            db.execute(SET_KV_SQL, params![key, value]).map(|changes| changes > 0)
        })
    }

    pub fn delete_kv(&self, key: &str) -> Result<bool, AgentDbProjectionError> {
        self.run_write(&format!("deleteKv:{key}"), |db| {
            db.execute(DELETE_KV_SQL, params![key]).map(|changes| changes > 0)
        })
    }

    pub fn read_session_work_admission(
        &self,
    ) -> Result<Option<SessionWorkAdmissionRecord>, AgentDbProjectionError> {
        let Some(raw) = self.read_kv(KV_SESSION_WORK_ADMISSION)? else {
            return Ok(None);
        };
        serde_json::from_str(&raw)
            .map(Some)
            .map_err(AgentDbProjectionError::MetadataJson)
    }

    /// Compare-and-set the agent-local work-admission record. A missing
    /// expected revision means create-only; a supplied revision means update
    /// only when the currently stored generation still matches.
    pub fn compare_and_set_session_work_admission(
        &self,
        expected_lifecycle_revision: Option<&str>,
        next: &SessionWorkAdmissionRecord,
    ) -> Result<bool, AgentDbProjectionError> {
        if next.session_key.trim().is_empty()
            || next.session_id.trim().is_empty()
            || next.lifecycle_revision.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session work admission requires session key, session id, and lifecycle revision"
                    .into(),
            ));
        }
        let encoded = serde_json::to_string(next)?;
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_mut() else {
            return Ok(false);
        };
        let current = db
            .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                row.get::<_, String>(0)
            })
            .optional()?;
        let matches = match (expected_lifecycle_revision, current.as_deref()) {
            (None, None) => true,
            (Some(expected), Some(raw)) => serde_json::from_str::<SessionWorkAdmissionRecord>(raw)
                .ok()
                .is_some_and(|record| record.lifecycle_revision == expected),
            _ => false,
        };
        if !matches {
            return Ok(false);
        }
        let changed = match current {
            Some(previous) => db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![encoded, KV_SESSION_WORK_ADMISSION, previous],
            )?,
            None => db.execute(
                SET_KV_SQL,
                params![KV_SESSION_WORK_ADMISSION, encoded],
            )?,
        };
        drop(guard);
        if changed == 1 {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
            );
        }
        Ok(changed == 1)
    }

    pub fn read_session_provider_review(
        &self,
    ) -> Result<Option<SessionProviderReview>, AgentDbProjectionError> {
        Ok(self
            .read_session_work_admission()?
            .and_then(|record| record.provider_review))
    }

    /// Record one provider-authored refusal against the captured admission
    /// generation. Identical terminal notifications retain the first review
    /// identity; conflicting facts fail closed.
    pub fn record_session_provider_review(
        &self,
        expected_lifecycle_revision: &str,
        review: &SessionProviderReview,
    ) -> Result<SessionProviderReview, AgentDbProjectionError> {
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        if expected_lifecycle_revision.is_empty()
            || review.id.trim().is_empty()
            || review.session_id.trim().is_empty()
            || review.run_id.trim().is_empty()
            || review.provider.trim().is_empty()
            || review.model.trim().is_empty()
            || review.runtime_id.trim().is_empty()
            || review.explanation.trim().is_empty()
            || review.explanation.len() > 64 * 1024
            || review
                .continuation_message
                .as_ref()
                .is_some_and(|message| message.trim().is_empty() || message.len() > 1024)
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "provider review is missing bounded identity or explanation".into(),
            ));
        }
        let Some(mut record) = self.read_session_work_admission()? else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session work admission record is unavailable".into(),
            ));
        };
        if record.lifecycle_revision != expected_lifecycle_revision
            || record.session_id != review.session_id
            || record.archived_at_ms.is_some()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "provider review admission generation changed".into(),
            ));
        }
        if let Some(previous) = record.provider_review.as_ref() {
            let mut previous_identity_free = previous.clone();
            previous_identity_free.id.clear();
            let mut next_identity_free = review.clone();
            next_identity_free.id.clear();
            if previous_identity_free == next_identity_free {
                return Ok(previous.clone());
            }
            if record.provider_review_phase.as_deref() != Some("accepted") {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "provider review conflicts with the existing terminal facts".into(),
                ));
            }
            // An accepted continuation may produce a new terminal refusal. It
            // replaces the consumed review under the same session generation;
            // pending reviews remain strict idempotent/conflict-safe CAS.
        }
        record.provider_review_id = Some(review.id.clone());
        record.provider_review_phase = Some("pending".into());
        record.provider_review = Some(review.clone());
        if !self.compare_and_set_session_work_admission(
            Some(expected_lifecycle_revision),
            &record,
        )? {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "provider review lost its lifecycle race".into(),
            ));
        }
        Ok(review.clone())
    }

    /// Atomically transition archive state for one canonical session
    /// generation. Active admitted work is preserved and blocks archiving;
    /// callers must settle it before retrying.
    pub fn transition_session_work_archive(
        &self,
        expected_session_id: &str,
        expected_lifecycle_revision: Option<&str>,
        archived: bool,
        archived_by: Option<&str>,
        archive_reason: Option<&str>,
        archived_at_ms: f64,
    ) -> Result<SessionWorkAdmissionRecord, AgentDbProjectionError> {
        let expected_session_id = expected_session_id.trim();
        if expected_session_id.is_empty() || !archived_at_ms.is_finite() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session archive transition requires a valid session identity and timestamp".into(),
            ));
        }
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_mut() else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "agent database is closed".into(),
            ));
        };
        let raw = db
            .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .ok_or_else(|| AgentDbProjectionError::WorkAdmissionInvalid(
                "session work admission record is unavailable".into(),
            ))?;
        let mut record: SessionWorkAdmissionRecord = serde_json::from_str(&raw)?;
        if record.session_id != expected_session_id {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session archive transition identity changed".into(),
            ));
        }
        if expected_lifecycle_revision.is_some_and(|expected| {
            record.lifecycle_revision != expected.trim()
        }) {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session archive transition lifecycle changed".into(),
            ));
        }
        if record.archived_at_ms.is_some() == archived {
            return Ok(record);
        }
        if archived && record.admitted_work_id.is_some() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session has admitted work; archive requires settlement before retry".into(),
            ));
        }
        if archived && record.admitted_media_work_id.is_some() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session has pending outgoing media; archive requires settlement before retry"
                    .into(),
            ));
        }
        if archived {
            record.archived_at_ms = Some(archived_at_ms);
            record.archived_by = archived_by
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            record.archive_reason = archive_reason
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .or_else(|| Some("manual".into()));
        } else {
            record.archived_at_ms = None;
            record.archived_by = None;
            record.archive_reason = None;
        }
        let encoded = serde_json::to_string(&record)?;
        if db.execute(
            COMPARE_AND_SET_KV_SQL,
            params![encoded, KV_SESSION_WORK_ADMISSION, raw],
        )? != 1
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session archive transition lost its lifecycle race".into(),
            ));
        }
        drop(guard);
        notify_agent_db_listeners(
            &self.db_path,
            AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
        );
        Ok(record)
    }

    /// Commit one background terminal receipt and the admission transition in
    /// the same SQLite transaction. The receipt is a hidden transcript entry
    /// so retries can recover the original payload rather than re-stage it.
    pub fn commit_session_work_result(
        &self,
        session_key: &str,
        expected_lifecycle_revision: &str,
        work_id: &str,
        idempotency_key: &str,
        content: &str,
        sent_message_count: usize,
    ) -> Result<SessionWorkResultCommitOutcome, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        let work_id = work_id.trim();
        let idempotency_key = idempotency_key.trim();
        if session_key.is_empty()
            || expected_lifecycle_revision.is_empty()
            || work_id.is_empty()
            || idempotency_key.is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "background result requires session identity, generation, work, and idempotency"
                    .into(),
            ));
        }

        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "agent database is closed".into(),
            ));
        };
        db.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<SessionWorkResultCommitOutcome, AgentDbProjectionError> {
            let receipt_id = format!("background-result:{idempotency_key}");
            if let Some(raw) = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![receipt_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
            {
                let Some(entry) = parse_transcript_entry(&raw) else {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "background result receipt contains malformed JSON".into(),
                    ));
                };
                let receipt = parse_session_work_result_receipt(&entry).ok_or_else(|| {
                    AgentDbProjectionError::WorkAdmissionInvalid(
                        "background result receipt is missing required identity".into(),
                    )
                })?;
                if receipt.session_key == session_key
                    && receipt.work_id == work_id
                    && receipt.content == content
                    && receipt.sent_message_count == sent_message_count
                {
                    return Ok(SessionWorkResultCommitOutcome::AlreadyCommitted(receipt));
                }
                return Ok(SessionWorkResultCommitOutcome::ConflictingResult);
            }

            let raw_admission = db
                .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(raw_admission) = raw_admission else {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            };
            let mut admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if admission.session_key != session_key
                || admission.lifecycle_revision != expected_lifecycle_revision
                || admission.archived_at_ms.is_some()
                || admission.admitted_work_id.as_deref() != Some(work_id)
            {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            }

            let receipt = SessionWorkResultReceipt {
                session_key: session_key.to_string(),
                lifecycle_revision: expected_lifecycle_revision.to_string(),
                work_id: work_id.to_string(),
                idempotency_key: idempotency_key.to_string(),
                content: content.to_string(),
                sent_message_count,
            };
            let entry = session_work_result_receipt_entry(&receipt, &receipt_id);
            db.execute(
                INSERT_TRANSCRIPT_ENTRY_SQL,
                params![receipt_id, entry.to_string()],
            )?;
            admission.admitted_work_id = None;
            admission.terminal_result_idempotency_key = Some(idempotency_key.to_string());
            let encoded = serde_json::to_string(&admission)?;
            if db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![encoded, KV_SESSION_WORK_ADMISSION, raw_admission],
            )? != 1
            {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            }
            Ok(SessionWorkResultCommitOutcome::Committed(receipt))
        })();

        match result {
            Ok(outcome) => {
                db.execute_batch("COMMIT")?;
                if let SessionWorkResultCommitOutcome::Committed(receipt) = &outcome {
                    let entry_id = format!("background-result:{}", receipt.idempotency_key);
                    let entry = db
                        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                            row.get::<_, String>(0)
                        })?;
                    if let Some(entry) = parse_transcript_entry(&entry) {
                        publish_entries_upserted(&self.db_path, vec![entry]);
                    }
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
                    );
                }
                Ok(outcome)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    pub fn read_metadata(&self) -> Result<serde_json::Value, AgentDbProjectionError> {
        let Some(raw) = self.read_kv("metadata")? else {
            return Ok(default_agent_metadata(&self.agent_dir_name));
        };
        let bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn get_metadata(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        Ok(self
            .read_metadata()?
            .as_object()
            .and_then(|metadata| metadata.get(key).cloned()))
    }

    pub fn initialize_metadata_if_missing(
        &self,
        value: serde_json::Value,
    ) -> Result<bool, AgentDbProjectionError> {
        if self.is_closed() || !value.is_object() {
            return Ok(false);
        }
        let encoded = encode_hex(&serde_json::to_vec(&value)?);
        let changed = self.run_write("initializeMetadata", |db| {
            let existing = db
                .query_row(GET_KV_SQL, params!["metadata"], |row| row.get::<_, String>(0))
                .optional()?;
            if existing.is_some() {
                return Ok(false);
            }
            db.execute(SET_KV_SQL, params!["metadata", encoded])
                .map(|changes| changes == 1)
        })?;
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata("metadata".to_string()),
            );
        }
        Ok(changed)
    }

    pub fn set_metadata(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<bool, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(false);
        }
        let Some(raw) = self.read_kv("metadata")? else {
            return Ok(false);
        };
        let bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
        let mut metadata: serde_json::Value = serde_json::from_slice(&bytes)?;
        let Some(object) = metadata.as_object_mut() else {
            return Ok(false);
        };
        let has_lifecycle_revision = object
            .get("lifecycleRevision")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|revision| !revision.trim().is_empty());
        if object.get(key) == Some(&value) && (key == "lifecycleRevision" || has_lifecycle_revision)
        {
            return Ok(true);
        }
        object.insert(key.to_string(), value);
        if key != "lifecycleRevision" {
            object.insert(
                "lifecycleRevision".to_string(),
                serde_json::json!(Uuid::new_v4().to_string()),
            );
        }
        let next_raw = encode_hex(&serde_json::to_vec(&metadata)?);
        let changed = self.run_write(&format!("setMetadata:{key}"), |db| {
            db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![next_raw, "metadata", raw],
            )
            .map(|changes| changes == 1)
        })?;
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(key.to_string()),
            );
        }
        Ok(changed)
    }

    /// Returns the durable session lifecycle revision, migrating legacy stores
    /// exactly once through the existing metadata CAS owner.
    pub fn ensure_lifecycle_revision(&self) -> Result<String, AgentDbProjectionError> {
        if let Some(existing) = self
            .get_metadata("lifecycleRevision")?
            .and_then(|value| value.as_str().map(str::trim).map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
        {
            return Ok(existing);
        }

        let candidate = Uuid::new_v4().to_string();
        if self.set_metadata("lifecycleRevision", serde_json::json!(candidate))? {
            return Ok(candidate);
        }

        self.get_metadata("lifecycleRevision")?
            .and_then(|value| value.as_str().map(str::trim).map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AgentDbProjectionError::LifecycleRevisionUnavailable(
                    "lifecycle revision migration did not publish a value".to_string(),
                )
            })
    }

    /// Atomically advances the durable Agent root and marks the addressed
    /// user transcript entry as confirmed. Conversation blobs are written
    /// before this call; metadata + recovery watermark must commit together.
    pub fn commit_checkpoint_root_and_confirm_user_message(
        &self,
        expected_root: &[u8],
        next_root: &[u8],
        entry_id: &str,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        let entry_id = entry_id.trim();
        if self.is_closed() || entry_id.is_empty() {
            return Ok(None);
        }
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(None);
        };

        db.execute_batch("BEGIN IMMEDIATE")?;
        let transaction = (|| -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
            let raw_metadata = db
                .query_row(GET_KV_SQL, params!["metadata"], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(raw_metadata) = raw_metadata else {
                return Ok(None);
            };
            let bytes =
                decode_hex(&raw_metadata).map_err(AgentDbProjectionError::MetadataHex)?;
            let mut metadata: serde_json::Value = serde_json::from_slice(&bytes)?;
            let current_root = metadata
                .get("latestRootBlobId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if current_root != encode_hex(expected_root) {
                return Ok(None);
            }

            let raw_entry = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(mut entry) = raw_entry
                .as_deref()
                .and_then(parse_transcript_entry)
            else {
                return Ok(None);
            };
            if entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
                || entry.get("role").and_then(serde_json::Value::as_str) != Some("user")
            {
                return Ok(None);
            }
            let Some(entry_object) = entry.as_object_mut() else {
                return Ok(None);
            };
            entry_object.insert("confirmed".into(), serde_json::Value::Bool(true));

            let Some(metadata_object) = metadata.as_object_mut() else {
                return Ok(None);
            };
            metadata_object.insert(
                "latestRootBlobId".into(),
                serde_json::Value::String(encode_hex(next_root)),
            );
            metadata_object.insert(
                "lifecycleRevision".into(),
                serde_json::json!(Uuid::new_v4().to_string()),
            );
            let next_metadata = encode_hex(&serde_json::to_vec(&metadata)?);
            if db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![next_metadata, "metadata", raw_metadata],
            )? != 1
            {
                return Ok(None);
            }
            if db.execute(
                UPDATE_TRANSCRIPT_ENTRY_SQL,
                params![entry.to_string(), entry_id],
            )? != 1
            {
                return Ok(None);
            }
            Ok(Some(entry))
        })();

        let confirmed = match transaction {
            Ok(Some(entry)) => {
                db.execute_batch("COMMIT")?;
                entry
            }
            Ok(None) => {
                let _ = db.execute_batch("ROLLBACK");
                return Ok(None);
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                if let AgentDbProjectionError::Sqlite(sqlite) = &error {
                    if is_sqlite_busy_error(sqlite) {
                        if let Some(callback) = self.options.on_busy_error.as_ref() {
                            callback(
                                "commitCheckpointRootAndConfirmUserMessage",
                                &sqlite.to_string(),
                            );
                        }
                        return Ok(None);
                    }
                }
                return Err(error);
            }
        };
        drop(guard);
        bump_db_write_generation(&self.db_path);
        notify_agent_db_listeners(
            &self.db_path,
            AgentDbListenerChannel::Metadata("latestRootBlobId".into()),
        );
        publish_entries_upserted(&self.db_path, vec![confirmed.clone()]);
        Ok(Some(confirmed))
    }

    pub fn compare_and_set_latest_root_blob_id(
        &self,
        expected_root: &[u8],
        next_root: &[u8],
    ) -> Result<bool, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(false);
        }
        let Some(raw) = self.read_kv("metadata")? else {
            return Ok(false);
        };
        let bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
        let mut metadata: serde_json::Value = serde_json::from_slice(&bytes)?;
        let current = metadata
            .get("latestRootBlobId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if current != encode_hex(expected_root) {
            return Ok(false);
        }
        let Some(object) = metadata.as_object_mut() else {
            return Ok(false);
        };
        object.insert(
            "latestRootBlobId".into(),
            serde_json::Value::String(encode_hex(next_root)),
        );
        object.insert(
            "lifecycleRevision".into(),
            serde_json::json!(Uuid::new_v4().to_string()),
        );
        let next_raw = encode_hex(&serde_json::to_vec(&metadata)?);
        let changed = self.run_write("compareAndSetLatestRootBlobId", |db| {
            db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![next_raw, "metadata", raw],
            )
            .map(|changes| changes == 1)
        })?;
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata("latestRootBlobId".into()),
            );
        }
        Ok(changed)
    }

    pub fn get_latest_root_blob_id(&self) -> Result<Vec<u8>, AgentDbProjectionError> {
        let root = self
            .get_metadata("latestRootBlobId")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_default();
        decode_hex(&root).map_err(AgentDbProjectionError::LatestRootHex)
    }

    pub fn serde_snapshot(&self) -> Result<AgentDbSerdeSnapshot, AgentDbProjectionError> {
        Ok(AgentDbSerdeSnapshot {
            profile: self.get_sand_profile()?,
            unread_state: self.get_unread_state()?,
            spend_guard_state: self.get_automation_spend_guard_state()?,
            awaiting_user_response: self.get_awaiting_user_response()?,
            request_ids: self.get_request_ids()?,
            pending_episode_turns: self.get_pending_episode_turns()?,
            memory_prompt_snapshot: self.get_memory_prompt_snapshot()?,
        })
    }

    pub fn subscribe_metadata(
        &self,
        key: impl Into<String>,
        listener: AgentDbListener,
    ) -> AgentDbSubscription {
        subscribe_agent_db_listener(
            &self.db_path,
            AgentDbListenerChannel::Metadata(key.into()),
            listener,
        )
    }

    pub fn subscribe_sand_profile(
        &self,
        listener: AgentDbListener,
    ) -> AgentDbSubscription {
        subscribe_agent_db_listener(
            &self.db_path,
            AgentDbListenerChannel::Profile,
            listener,
        )
    }

    pub fn subscribe_awaiting_user_response(
        &self,
        listener: AgentDbListener,
    ) -> AgentDbSubscription {
        subscribe_agent_db_listener(
            &self.db_path,
            AgentDbListenerChannel::Awaiting,
            listener,
        )
    }

    pub fn get_sand_profile(&self) -> Result<SandProfile, AgentDbProjectionError> {
        Ok(parse_profile(self.read_kv(KV_PROFILE)?.as_deref()))
    }

    pub fn set_sand_profile(
        &self,
        profile: &SandProfile,
    ) -> Result<bool, AgentDbProjectionError> {
        if self.get_sand_profile()? == *profile {
            return Ok(true);
        }
        let raw = serde_json::json!({
            "description": profile.description.clone(),
            "avatarPath": profile.avatar_path.clone(),
        })
        .to_string();
        let wrote = self.write_kv(KV_PROFILE, &raw)?;
        if wrote {
            notify_agent_db_listeners(&self.db_path, AgentDbListenerChannel::Profile);
        }
        Ok(wrote)
    }

    pub fn get_awaiting_user_response(
        &self,
    ) -> Result<Option<AwaitingUserResponse>, AgentDbProjectionError> {
        Ok(parse_awaiting_state(self.read_kv(KV_AWAITING)?.as_deref()))
    }

    pub fn set_awaiting_user_response(
        &self,
        state: Option<&AwaitingUserResponse>,
    ) -> Result<bool, AgentDbProjectionError> {
        if self.get_awaiting_user_response()?.as_ref() == state {
            return Ok(false);
        }
        let wrote = if let Some(state) = state {
            let raw = serde_json::json!({
                "tabId": state.tab_id.clone(),
                "reason": state.reason.clone(),
                "since": state.since,
            })
            .to_string();
            self.write_kv(KV_AWAITING, &raw)?
        } else {
            self.delete_kv(KV_AWAITING)?
        };
        if wrote {
            notify_agent_db_listeners(&self.db_path, AgentDbListenerChannel::Awaiting);
        }
        Ok(wrote)
    }

    pub fn set_awaiting_user_response_for_tab(
        &self,
        tab_id: &str,
        state: Option<&AwaitingUserResponse>,
        if_since_before: Option<f64>,
    ) -> Result<bool, AgentDbProjectionError> {
        let current = self.get_awaiting_user_response()?;
        if current.as_ref().is_some_and(|current| current.tab_id != tab_id) {
            return Ok(false);
        }
        if state.is_none()
            && (current.is_none()
                || if_since_before.is_some_and(|before| {
                    current.as_ref().is_some_and(|current| current.since >= before)
                }))
        {
            return Ok(false);
        }
        if state.is_some_and(|state| state.tab_id != tab_id) {
            return Ok(false);
        }
        self.set_awaiting_user_response(state)
    }

    pub fn get_unread_state(&self) -> Result<UnreadState, AgentDbProjectionError> {
        Ok(parse_unread_state(self.read_kv(KV_UNREAD)?.as_deref()))
    }

    fn write_unread_state_owner(
        &self,
        state: &UnreadState,
    ) -> Result<bool, AgentDbProjectionError> {
        let raw = serde_json::json!({
            "lastActivityAt": state.last_activity_at,
            "lastViewedAt": state.last_viewed_at,
            "isManuallyUnread": state.is_manually_unread,
            "unreadCount": state.unread_count,
        })
        .to_string();
        self.write_kv(KV_UNREAD, &raw)
    }

    pub fn mark_activity(&self, at: f64) -> Result<bool, AgentDbProjectionError> {
        let current = self.get_unread_state()?;
        if current.last_activity_at >= at {
            return Ok(false);
        }
        self.write_unread_state_owner(&UnreadState {
            last_activity_at: at,
            unread_count: current.unread_count + 1.0,
            ..current
        })
    }

    pub fn mark_viewed(
        &self,
        at: f64,
        preserve_manual_unread: bool,
    ) -> Result<bool, AgentDbProjectionError> {
        let current = self.get_unread_state()?;
        if preserve_manual_unread && current.is_manually_unread {
            return Ok(false);
        }
        if !current.is_manually_unread && current.last_viewed_at >= at {
            return Ok(false);
        }
        self.write_unread_state_owner(&UnreadState {
            last_viewed_at: at,
            is_manually_unread: false,
            unread_count: 0.0,
            ..current
        })
    }

    pub fn get_newest_divider_anchor_timestamp_ms(
        &self,
    ) -> Result<f64, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(0.0);
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(0.0);
        };
        Ok(newest_divider_anchor_timestamp_ms(db)?)
    }

    pub fn mark_unread(&self, at: f64) -> Result<bool, AgentDbProjectionError> {
        let current = self.get_unread_state()?;
        let newest = self.get_newest_divider_anchor_timestamp_ms()?;
        let last_activity_at = if current.last_activity_at == 0.0 {
            at
        } else {
            current.last_activity_at
        };
        let newest_bound = if newest > 0.0 {
            newest - 1.0
        } else {
            f64::INFINITY
        };
        let last_viewed_at = current
            .last_viewed_at
            .min(last_activity_at - 1.0)
            .min(at - 1.0)
            .min(newest_bound);
        self.write_unread_state_owner(&UnreadState {
            last_activity_at,
            last_viewed_at,
            is_manually_unread: true,
            unread_count: current.unread_count.max(1.0),
        })
    }

    pub fn mark_read(&self, at: f64) -> Result<bool, AgentDbProjectionError> {
        let current = self.get_unread_state()?;
        self.write_unread_state_owner(&UnreadState {
            last_viewed_at: current.last_activity_at.max(at),
            is_manually_unread: false,
            unread_count: 0.0,
            ..current
        })
    }

    pub fn get_introduction_pending(&self) -> Result<bool, AgentDbProjectionError> {
        Ok(self.read_kv(KV_INTRODUCTION)?.as_deref() == Some("1"))
    }

    pub fn set_introduction_pending(
        &self,
        pending: bool,
    ) -> Result<bool, AgentDbProjectionError> {
        if pending {
            self.write_kv(KV_INTRODUCTION, "1")
        } else {
            self.delete_kv(KV_INTRODUCTION)
        }
    }

    fn read_version(&self, key: &str) -> Result<u64, AgentDbProjectionError> {
        Ok(self
            .read_kv(key)?
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or_default())
    }

    pub fn get_hidden_entry_repair_version(&self) -> Result<u64, AgentDbProjectionError> {
        self.read_version(KV_HIDDEN_REPAIR)
    }

    pub fn set_hidden_entry_repair_version(
        &self,
        version: u64,
    ) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_HIDDEN_REPAIR, &version.to_string())
    }

    pub fn get_stale_root_cleanup_version(&self) -> Result<u64, AgentDbProjectionError> {
        self.read_version(KV_STALE_ROOT_CLEANUP)
    }

    pub fn set_stale_root_cleanup_version(
        &self,
        version: u64,
    ) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_STALE_ROOT_CLEANUP, &version.to_string())
    }

    pub fn get_legacy_blob_retirement_version(&self) -> Result<u64, AgentDbProjectionError> {
        self.read_version(KV_LEGACY_BLOB_RETIREMENT)
    }

    pub fn has_legacy_conversation_blobs(&self) -> Result<bool, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(false);
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(false);
        };
        Ok(db
            .query_row(HAS_LEGACY_BLOB_SQL, [], |_| Ok(()))
            .optional()?
            .is_some())
    }

    pub fn retire_legacy_conversation_blobs(
        &self,
        version: u64,
    ) -> Result<bool, AgentDbProjectionError> {
        self.run_write("retireLegacyConversationBlobs", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(), rusqlite::Error> {
                db.execute(CLEAR_BLOBS_SQL, [])?;
                db.execute(
                    SET_KV_SQL,
                    params![KV_LEGACY_BLOB_RETIREMENT, version.to_string()],
                )?;
                Ok(())
            })();
            match result {
                Ok(()) => {
                    db.execute_batch("COMMIT")?;
                    Ok(true)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn get_automation_spend_guard_state(
        &self,
    ) -> Result<SpendGuardState, AgentDbProjectionError> {
        let state = self.read_kv(KV_SPEND_GUARD)?;
        let legacy = self.read_kv(KV_SPEND_GUARD_LEGACY)?;
        Ok(resolve_spend_guard_state(state.as_deref(), legacy.as_deref()))
    }

    pub fn set_automation_spend_guard_state(
        &self,
        state: &SpendGuardState,
    ) -> Result<bool, AgentDbProjectionError> {
        let raw = serialize_spend_guard_state(state);
        self.run_write("setAutomationSpendGuardState", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                let changed = if let Some(raw) = raw.as_deref() {
                    db.execute(SET_KV_SQL, params![KV_SPEND_GUARD, raw])? > 0
                } else {
                    db.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD])? > 0
                };
                let legacy_changed =
                    db.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD_LEGACY])? > 0;
                Ok(changed || legacy_changed)
            })();
            match result {
                Ok(changed) => {
                    db.execute_batch("COMMIT")?;
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn get_request_ids(&self) -> Result<Vec<RequestRecord>, AgentDbProjectionError> {
        let mut records = parse_request_records(self.read_kv(KV_REQUEST_IDS)?.as_deref());
        if records.is_empty() {
            if let Some(legacy) = self
                .read_kv(KV_LATEST_REQUEST_ID)?
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            {
                records.push(RequestRecord {
                    id: legacy,
                    at: 0.0,
                    prompt: None,
                    source: None,
                });
            }
        }
        Ok(records)
    }

    pub fn record_request_id(
        &self,
        id: &str,
        at: f64,
        prompt: Option<&str>,
        source: Option<&str>,
    ) -> Result<bool, AgentDbProjectionError> {
        let id = id.trim();
        if id.is_empty() {
            return Ok(false);
        }
        let mut records = self.get_request_ids()?;
        if records.last().is_some_and(|record| record.id == id) {
            return Ok(false);
        }
        let label = prompt
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.chars().take(REQUEST_ID_PROMPT_MAX).collect::<String>());
        records.push(RequestRecord {
            id: id.to_string(),
            at,
            prompt: label,
            source: source.filter(|value| !value.is_empty()).map(ToOwned::to_owned),
        });
        if records.len() > REQUEST_ID_HISTORY_MAX {
            records.drain(0..records.len() - REQUEST_ID_HISTORY_MAX);
        }
        let raw = serde_json::Value::Array(
            records
                .iter()
                .map(|record| {
                    let mut value = serde_json::Map::new();
                    value.insert("id".into(), serde_json::Value::String(record.id.clone()));
                    value.insert("at".into(), json_number(record.at));
                    if let Some(prompt) = &record.prompt {
                        value.insert("prompt".into(), serde_json::Value::String(prompt.clone()));
                    }
                    if let Some(source) = &record.source {
                        value.insert("source".into(), serde_json::Value::String(source.clone()));
                    }
                    serde_json::Value::Object(value)
                })
                .collect(),
        )
        .to_string();
        self.write_kv(KV_REQUEST_IDS, &raw)
    }

    pub fn get_pending_episode_turns(
        &self,
    ) -> Result<Vec<EpisodeTurn>, AgentDbProjectionError> {
        Ok(parse_pending_episode_turns(self.read_kv(KV_EPISODE)?.as_deref()))
    }

    pub fn record_episode_turn(
        &self,
        turn: &EpisodeTurn,
    ) -> Result<bool, AgentDbProjectionError> {
        let mut turns = self.get_pending_episode_turns()?;
        turns.push(EpisodeTurn {
            ts: turn.ts,
            user: turn.user.chars().take(EPISODE_TURN_TEXT_CAP).collect(),
            agent: turn.agent.chars().take(EPISODE_TURN_TEXT_CAP).collect(),
        });
        if turns.len() > EPISODE_PENDING_MAX {
            turns.drain(0..turns.len() - EPISODE_PENDING_MAX);
        }
        let raw = serde_json::Value::Array(
            turns
                .iter()
                .map(|turn| {
                    serde_json::json!({
                        "ts": turn.ts,
                        "user": turn.user.clone(),
                        "agent": turn.agent.clone(),
                    })
                })
                .collect(),
        )
        .to_string();
        self.write_kv(KV_EPISODE, &raw)
    }

    pub fn clear_pending_episode_turns(&self) -> Result<bool, AgentDbProjectionError> {
        self.delete_kv(KV_EPISODE)
    }

    pub fn get_memory_prompt_snapshot(
        &self,
    ) -> Result<Option<MemoryPromptSnapshot>, AgentDbProjectionError> {
        Ok(parse_memory_prompt_snapshot(
            self.read_kv(KV_MEMORY_SNAPSHOT)?.as_deref(),
        ))
    }

    pub fn set_memory_prompt_snapshot(
        &self,
        snapshot: &serde_json::Value,
    ) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_MEMORY_SNAPSHOT, &snapshot.to_string())
    }

    pub fn clear_memory_prompt_snapshot(&self) -> Result<bool, AgentDbProjectionError> {
        self.delete_kv(KV_MEMORY_SNAPSHOT)
    }

    pub fn get_agent_profile_prompt_snapshot(
        &self,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        let Some(raw) = self.read_kv(KV_PROFILE_SNAPSHOT)? else {
            return Ok(None);
        };
        Ok(serde_json::from_str::<serde_json::Value>(&raw).ok())
    }

    pub fn set_agent_profile_prompt_snapshot(
        &self,
        snapshot: &serde_json::Value,
    ) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_PROFILE_SNAPSHOT, &snapshot.to_string())
    }

    pub fn clear_agent_profile_prompt_snapshot(&self) -> Result<bool, AgentDbProjectionError> {
        self.delete_kv(KV_PROFILE_SNAPSHOT)
    }

    pub fn clear_transient_state(&self) -> Result<bool, AgentDbProjectionError> {
        self.run_write("clearTransientState", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                let mut changed = false;
                for key in [
                    KV_UNREAD,
                    KV_SPEND_GUARD_LEGACY,
                    KV_SPEND_GUARD,
                    KV_AWAITING,
                    KV_LATEST_REQUEST_ID,
                    KV_REQUEST_IDS,
                    KV_EPISODE,
                    KV_MEMORY_SNAPSHOT,
                    KV_PROFILE_SNAPSHOT,
                ] {
                    changed |= db.execute(DELETE_KV_SQL, params![key])? > 0;
                }
                Ok(changed)
            })();
            match result {
                Ok(changed) => {
                    db.execute_batch("COMMIT")?;
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }


    pub fn get_agent_origin(&self) -> Result<String, AgentDbProjectionError> {
        Ok(if self.read_kv(KV_ORIGIN)?.as_deref() == Some("dev") {
            "dev".to_string()
        } else {
            "user".to_string()
        })
    }

    pub fn set_agent_origin(&self, origin: &str) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_ORIGIN, if origin == "dev" { "dev" } else { "user" })
    }

    pub fn get_agent_purpose(&self) -> Result<Option<String>, AgentDbProjectionError> {
        Ok(self
            .read_kv(KV_PURPOSE)?
            .filter(|value| valid_agent_purpose(value)))
    }

    pub fn set_agent_purpose(&self, purpose: &str) -> Result<bool, AgentDbProjectionError> {
        self.write_kv(KV_PURPOSE, purpose)
    }

    pub fn clear_agent_purpose(&self) -> Result<bool, AgentDbProjectionError> {
        self.delete_kv(KV_PURPOSE)
    }

    pub fn get_conversation_partner_ids(
        &self,
    ) -> Result<Vec<String>, AgentDbProjectionError> {
        let Some(raw) = self.read_kv(KV_PARTNERS)? else {
            return Ok(Vec::new());
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return Ok(Vec::new());
        };
        Ok(value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default())
    }

    pub fn add_conversation_partner(
        &self,
        partner_id: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        let id = partner_id.trim();
        if id.is_empty() {
            return Ok(false);
        }
        let own_id = self
            .read_kv("metadata")?
            .and_then(|raw| decode_hex(&raw).ok())
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .and_then(|metadata| {
                metadata
                    .get("agentId")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_default();
        if id == own_id {
            return Ok(false);
        }
        let mut values = self
            .get_conversation_partner_ids()?
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        if !values.insert(id.to_string()) {
            return Ok(false);
        }
        let raw = serde_json::Value::Array(
            values
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        )
        .to_string();
        self.write_kv(KV_PARTNERS, &raw)
    }

    pub fn append_transcript_entry(
        &self,
        entry: &serde_json::Value,
    ) -> Result<bool, AgentDbProjectionError> {
        Ok(self.append_transcript_entries(std::slice::from_ref(entry))? > 0)
    }

    pub fn append_transcript_entries(
        &self,
        entries: &[serde_json::Value],
    ) -> Result<usize, AgentDbProjectionError> {
        if entries.is_empty() {
            return Ok(0);
        }
        let inserted = self.run_write_value("appendTranscriptEntries", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<Vec<serde_json::Value>, rusqlite::Error> {
                let mut inserted = Vec::new();
                {
                    let mut statement = db.prepare(INSERT_TRANSCRIPT_ENTRY_SQL)?;
                    for entry in entries {
                        let Some(id) = entry.get("id").and_then(serde_json::Value::as_str) else {
                            continue;
                        };
                        if entry.get("kind").and_then(serde_json::Value::as_str).is_none() {
                            continue;
                        }
                        if statement.execute(params![id, entry.to_string()])? > 0 {
                            inserted.push(entry.clone());
                        }
                    }
                }
                Ok(inserted)
            })();
            match result {
                Ok(inserted) => {
                    db.execute_batch("COMMIT")?;
                    let changed = !inserted.is_empty();
                    Ok((inserted, changed))
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if !inserted.is_empty() {
            publish_entries_upserted(&self.db_path, inserted.clone());
        }
        Ok(inserted.len())
    }

    /// Atomically append transcript entries and clear one captured outgoing
    /// media fence. A stale or rebound fence rolls back the transcript write.
    pub fn append_transcript_entries_and_release_media(
        &self,
        entries: &[serde_json::Value],
        expected_lifecycle_revision: &str,
        media_work_id: &str,
    ) -> Result<usize, AgentDbProjectionError> {
        if entries.is_empty()
            || expected_lifecycle_revision.trim().is_empty()
            || media_work_id.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "media settlement requires transcript entries, generation, and work identity"
                    .into(),
            ));
        }
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_mut() else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "agent database is closed".into(),
            ));
        };
        db.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
            let raw_admission = db
                .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
                .ok_or_else(|| {
                    AgentDbProjectionError::WorkAdmissionInvalid(
                        "session work admission record is unavailable".into(),
                    )
                })?;
            let mut admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if admission.lifecycle_revision != expected_lifecycle_revision.trim()
                || admission.admitted_media_work_id.as_deref() != Some(media_work_id.trim())
                || admission.archived_at_ms.is_some()
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "media settlement generation or identity is stale".into(),
                ));
            }
            let mut inserted = Vec::new();
            {
                let mut statement = db.prepare(INSERT_TRANSCRIPT_ENTRY_SQL)?;
                for entry in entries {
                    let Some(id) = entry.get("id").and_then(serde_json::Value::as_str) else {
                        continue;
                    };
                    if entry.get("kind").and_then(serde_json::Value::as_str).is_none() {
                        continue;
                    }
                    if statement.execute(params![id, entry.to_string()])? > 0 {
                        inserted.push(entry.clone());
                    }
                }
            }
            let receipt_id = format!("outgoing-media-settlement:{}", media_work_id.trim());
            let receipt = media_settlement_receipt_entry(
                &receipt_id,
                media_work_id.trim(),
                expected_lifecycle_revision.trim(),
                &admission.admitted_media_paths,
                &inserted,
            );
            db.execute(
                INSERT_TRANSCRIPT_ENTRY_SQL,
                params![receipt_id, receipt.to_string()],
            )?;
            admission.admitted_media_work_id = None;
            admission.admitted_media_paths.clear();
            let encoded = serde_json::to_string(&admission)?;
            if db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![encoded, KV_SESSION_WORK_ADMISSION, raw_admission],
            )? != 1
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "media settlement lost its lifecycle race".into(),
                ));
            }
            Ok(inserted)
        })();
        match result {
            Ok(inserted) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                let count = inserted.len();
                if !inserted.is_empty() {
                    publish_entries_upserted(&self.db_path, inserted);
                }
                notify_agent_db_listeners(
                    &self.db_path,
                    AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
                );
                Ok(count)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    /// Atomically commits one guarded compaction entry and advances the
    /// session lifecycle revision. Older rows remain durable; replay uses the
    /// latest compaction entry as the boundary for the active context.
    pub fn commit_compaction(
        &self,
        commit: &CompactionCommit,
    ) -> Result<CompactionCommitOutcome, AgentDbProjectionError> {
        let entry_id = commit.entry_id.trim();
        let source_turn_id = commit.source_turn_id.trim();
        let expected_revision = commit.expected_lifecycle_revision.trim();
        if entry_id.is_empty() || source_turn_id.is_empty() || expected_revision.is_empty() {
            return Err(AgentDbProjectionError::CompactionInvalid(
                "entry_id, source_turn_id, and expected_lifecycle_revision are required".into(),
            ));
        }
        if let Some(first_kept_entry_id) = commit.first_kept_entry_id.as_deref() {
            if first_kept_entry_id.trim().is_empty() {
                return Err(AgentDbProjectionError::CompactionInvalid(
                    "first_kept_entry_id must be non-empty when present".into(),
                ));
            }
        }

        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Err(AgentDbProjectionError::CompactionInvalid(
                "agent database is closed".into(),
            ));
        };

        db.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<CompactionCommitOutcome, AgentDbProjectionError> {
            let existing = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            if let Some(raw) = existing {
                let Some(existing) = parse_transcript_entry(&raw) else {
                    return Err(AgentDbProjectionError::CompactionInvalid(
                        "existing compaction identity contains malformed JSON".into(),
                    ));
                };
                if compaction_payload_matches(&existing, commit) {
                    return Ok(CompactionCommitOutcome::AlreadyCommitted(existing));
                }
                return Err(AgentDbProjectionError::CompactionConflict(entry_id.to_string()));
            }

            let raw_metadata = db
                .query_row(GET_KV_SQL, params!["metadata"], |row| row.get::<_, String>(0))
                .optional()?;
            let Some(raw_metadata) = raw_metadata else {
                return Err(AgentDbProjectionError::CompactionInvalid(
                    "metadata is missing".into(),
                ));
            };
            let metadata_bytes = decode_hex(&raw_metadata).map_err(AgentDbProjectionError::MetadataHex)?;
            let mut metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes)?;
            let current_revision = metadata
                .get("lifecycleRevision")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AgentDbProjectionError::LifecycleRevisionUnavailable(
                        "metadata has no lifecycleRevision".into(),
                    )
                })?;
            if current_revision != expected_revision {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }

            if let Some(first_kept_entry_id) = commit.first_kept_entry_id.as_deref() {
                let present = db
                    .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![first_kept_entry_id], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                    .is_some();
                if !present {
                    return Err(AgentDbProjectionError::CompactionInvalid(format!(
                        "first kept entry {first_kept_entry_id} does not exist"
                    )));
                }
            }

            let next_revision = Uuid::new_v4().to_string();
            let mut entry = serde_json::Map::new();
            entry.insert("id".into(), serde_json::Value::String(entry_id.to_string()));
            entry.insert("kind".into(), serde_json::Value::String("compaction".into()));
            entry.insert(
                "firstKeptEntryId".into(),
                commit
                    .first_kept_entry_id
                    .as_ref()
                    .map(|value| serde_json::Value::String(value.trim().to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
            entry.insert("summary".into(), serde_json::Value::String(commit.summary.clone()));
            entry.insert(
                "tokensBefore".into(),
                serde_json::Value::Number(serde_json::Number::from(commit.tokens_before)),
            );
            if let Some(details) = commit.details.clone() {
                entry.insert("details".into(), details);
            }
            entry.insert(
                "sourceTurnId".into(),
                serde_json::Value::String(source_turn_id.to_string()),
            );
            entry.insert(
                "sourceLifecycleRevision".into(),
                serde_json::Value::String(expected_revision.to_string()),
            );
            entry.insert(
                "lifecycleRevision".into(),
                serde_json::Value::String(next_revision.clone()),
            );
            let entry = serde_json::Value::Object(entry);
            db.execute(
                INSERT_TRANSCRIPT_ENTRY_SQL,
                params![entry_id, entry.to_string()],
            )?;

            let metadata_object = metadata.as_object_mut().ok_or_else(|| {
                AgentDbProjectionError::CompactionInvalid("metadata is not an object".into())
            })?;
            metadata_object.insert(
                "lifecycleRevision".into(),
                serde_json::Value::String(next_revision),
            );
            let next_metadata = encode_hex(&serde_json::to_vec(&metadata)?);
            if db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![next_metadata, "metadata", raw_metadata],
            )? != 1
            {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }
            Ok(CompactionCommitOutcome::Committed(entry))
        })();

        match result {
            Ok(outcome @ CompactionCommitOutcome::Committed(_)) => {
                db.execute_batch("COMMIT")?;
                bump_db_write_generation(&self.db_path);
                if let CompactionCommitOutcome::Committed(entry) = &outcome {
                    publish_entries_upserted(&self.db_path, vec![entry.clone()]);
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata("lifecycleRevision".into()),
                    );
                }
                Ok(outcome)
            }
            Ok(outcome) => {
                let _ = db.execute_batch("ROLLBACK");
                Ok(outcome)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    /// Returns the active replay projection after the newest compaction.
    /// Malformed compaction boundaries fail closed instead of silently
    /// replaying an incomplete conversation.
    pub fn get_compacted_transcript_entries(
        &self,
    ) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
        let entries = self.get_transcript_entries()?;
        let Some(compaction_index) = entries
            .iter()
            .rposition(|entry| entry.get("kind").and_then(serde_json::Value::as_str) == Some("compaction"))
        else {
            return Ok(entries);
        };
        let compaction = entries[compaction_index].clone();
        if compaction.get("firstKeptEntryId").is_none()
            || compaction.get("summary").and_then(serde_json::Value::as_str).is_none()
            || compaction.get("tokensBefore").and_then(serde_json::Value::as_u64).is_none()
        {
            return Err(AgentDbProjectionError::CompactionInvalid(
                "compaction entry is missing replay fields".into(),
            ));
        }
        let first_kept = compaction
            .get("firstKeptEntryId")
            .and_then(serde_json::Value::as_str);
        let start = match first_kept {
            Some(first_kept) if !first_kept.is_empty() => entries
                .iter()
                .position(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(first_kept))
                .ok_or_else(|| {
                    AgentDbProjectionError::CompactionInvalid(format!(
                        "compaction boundary {first_kept} is missing"
                    ))
                })?,
            Some(_) | None => entries.len(),
        };
        let mut replay = vec![compaction];
        replay.extend(
            entries
                .into_iter()
                .skip(start)
                .filter(|entry| entry.get("kind").and_then(serde_json::Value::as_str) != Some("compaction")),
        );
        Ok(replay)
    }

    pub fn update_transcript_entry(
        &self,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        let updated = self.run_write_value("updateTranscriptEntry", |db| {
            let present = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
                .is_some();
            if !present {
                return Ok((None, false));
            }
            let changed = db.execute(
                UPDATE_TRANSCRIPT_ENTRY_SQL,
                params![next.to_string(), entry_id],
            )? > 0;
            Ok((changed.then(|| next.clone()), changed))
        })?;
        if let Some(entry) = updated.as_ref() {
            publish_entries_upserted(&self.db_path, vec![entry.clone()]);
        }
        Ok(updated)
    }

    pub fn settle_transcript_entry_identity(
        &self,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        let Some(next_id) = next.get("id").and_then(serde_json::Value::as_str) else {
            return Ok(None);
        };
        let updated = self.run_write_value("settleTranscriptEntryIdentity", |db| {
            let present = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| row.get::<_, String>(0))
                .optional()?
                .is_some();
            if !present {
                return Ok((None, false));
            }
            let changed = db.execute(
                "UPDATE transcript_entries SET id = ?1, entry = ?2 WHERE id = ?3",
                params![next_id, next.to_string(), entry_id],
            )? > 0;
            Ok((changed.then(|| next.clone()), changed))
        })?;
        if let Some(entry) = updated.as_ref() {
            if entry_id != next_id {
                let mut mutation = serde_json::Map::new();
                mutation.insert("kind".into(), serde_json::Value::String("entry-deleted".into()));
                mutation.insert("agentId".into(), serde_json::Value::String(self.agent_dir_name.clone()));
                mutation.insert("entryId".into(), serde_json::Value::String(entry_id.to_string()));
                publish_transcript_mutation(&mutation);
            }
            publish_entries_upserted(&self.db_path, vec![entry.clone()]);
        }
        Ok(updated)
    }

    pub fn delete_transcript_entry(
        &self,
        entry_id: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        let committed = self.run_write("deleteTranscriptEntry", |db| {
            db.execute(DELETE_TRANSCRIPT_ENTRY_SQL, params![entry_id])?;
            Ok(true)
        })?;
        if committed {
            let mut mutation = serde_json::Map::new();
            mutation.insert(
                "kind".into(),
                serde_json::Value::String("entry-deleted".into()),
            );
            mutation.insert(
                "agentId".into(),
                serde_json::Value::String(self.agent_dir_name.clone()),
            );
            mutation.insert(
                "entryId".into(),
                serde_json::Value::String(entry_id.to_string()),
            );
            publish_transcript_mutation(&mutation);
        }
        Ok(committed)
    }


    pub fn get_transcript_entries(
        &self,
    ) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(Vec::new());
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        Ok(read_transcript_entries_from_db(db)?)
    }

    pub fn get_transcript_page(
        &self,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(TranscriptPage {
                entries: Vec::new(),
                next_before_seq: None,
            });
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(TranscriptPage {
                entries: Vec::new(),
                next_before_seq: None,
            });
        };
        Ok(read_transcript_page(db, query)?)
    }

    pub fn get_transcript_window(
        &self,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<BTreeMap<String, usize>>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(TranscriptWindow {
                entries: Vec::new(),
                next_before_seq: None,
                thread_counts: BTreeMap::new(),
            });
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(TranscriptWindow {
                entries: Vec::new(),
                next_before_seq: None,
                thread_counts: BTreeMap::new(),
            });
        };
        let branched = read_branched_entries_from_db(db)?;
        let counts = branch_reply_counts_from_entries(&branched);
        Ok(read_transcript_window(db, query, |entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let id = entry.get("id").and_then(serde_json::Value::as_str)?;
                    counts.get(id).copied().map(|count| (id.to_string(), count))
                })
                .collect()
        })?)
    }

    pub fn get_transcript_tail(
        &self,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(TranscriptPage {
                entries: Vec::new(),
                next_before_seq: None,
            });
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(TranscriptPage {
                entries: Vec::new(),
                next_before_seq: None,
            });
        };
        Ok(read_transcript_tail(db, query)?)
    }

    pub fn get_transcript_entry_by_id(
        &self,
        entry_id: &str,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(None);
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(None);
        };
        Ok(read_transcript_entry_from_db(db, entry_id)?)
    }

    pub fn get_branched_entries(
        &self,
    ) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(Vec::new());
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        Ok(read_branched_entries_from_db(db)?)
    }

    pub fn get_thread_entries(
        &self,
        root_id: &str,
    ) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(Vec::new());
        }
        let guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        let Some(root) = read_transcript_entry_from_db(db, root_id)? else {
            return Ok(Vec::new());
        };
        let branched = read_branched_entries_from_db(db)?;
        let mut entries = vec![root];
        entries.extend(thread_descendants_from_entries(root_id, &branched));
        Ok(entries)
    }

    pub fn get_main_transcript_entries(
        &self,
    ) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
        let entries = self.get_transcript_entries()?;
        Ok(main_transcript_entries_from_entries(&entries))
    }

    pub fn clear_conversation(&self) -> Result<bool, AgentDbProjectionError> {
        if self.is_closed() {
            return Ok(false);
        }
        let raw_metadata = self.read_kv("metadata")?;
        let next_metadata = if let Some(raw) = raw_metadata.as_deref() {
            let bytes = decode_hex(raw).map_err(AgentDbProjectionError::MetadataHex)?;
            let mut metadata: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(AgentDbProjectionError::MetadataJson)?;
            if let Some(object) = metadata.as_object_mut() {
                object.insert(
                    "latestRootBlobId".into(),
                    serde_json::Value::String(String::new()),
                );
                object.insert(
                    "currentPlanUri".into(),
                    serde_json::Value::String(String::new()),
                );
                object.insert(
                    "lifecycleRevision".into(),
                    serde_json::json!(Uuid::new_v4().to_string()),
                );
            }
            Some(encode_hex(&serde_json::to_vec(&metadata)?))
        } else {
            None
        };

        let committed = self.run_write("clearConversation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(), rusqlite::Error> {
                db.execute(CLEAR_BLOBS_SQL, [])?;
                db.execute(CLEAR_TRANSCRIPT_ENTRIES_SQL, [])?;
                for key in [
                    KV_AWAITING,
                    KV_LATEST_REQUEST_ID,
                    KV_REQUEST_IDS,
                    KV_EPISODE,
                    KV_MEMORY_SNAPSHOT,
                    KV_PROFILE_SNAPSHOT,
                ] {
                    db.execute(DELETE_KV_SQL, params![key])?;
                }
                if let Some(next_metadata) = next_metadata.as_deref() {
                    db.execute(SET_KV_SQL, params!["metadata", next_metadata])?;
                }
                Ok(())
            })();
            match result {
                Ok(()) => {
                    db.execute_batch("COMMIT")?;
                    Ok(true)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if !committed {
            return Ok(false);
        }

        let mut mutation = serde_json::Map::new();
        mutation.insert(
            "kind".into(),
            serde_json::Value::String("conversation-cleared".into()),
        );
        mutation.insert(
            "agentId".into(),
            serde_json::Value::String(self.agent_dir_name.clone()),
        );
        publish_transcript_mutation(&mutation);
        notify_agent_db_listeners(&self.db_path, AgentDbListenerChannel::Awaiting);
        notify_agent_db_listeners(
            &self.db_path,
            AgentDbListenerChannel::Metadata("latestRootBlobId".into()),
        );
        Ok(true)
    }

    pub fn close(&self, checkpoint: bool) {
        if self
            .handle_registered
            .swap(false, Ordering::AcqRel)
        {
            release_live_db_handle(&self.db_path);
        }
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let Ok(mut guard) = self.connection.lock() else {
            return;
        };
        let Some(db) = guard.take() else {
            return;
        };
        if checkpoint {
            let _ = db.query_row(
                "PRAGMA wal_checkpoint(TRUNCATE)",
                [],
                |row| Ok((row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
            );
        }
        drop(db);
    }

    fn seed_default_metadata_if_missing(&self) -> Result<(), AgentDbProjectionError> {
        if self.read_kv("metadata")?.is_some() {
            return Ok(());
        }
        let metadata = default_agent_metadata(&self.agent_dir_name);
        let raw = encode_hex(&serde_json::to_vec(&metadata)?);
        let _ = self.write_kv("metadata", &raw)?;
        Ok(())
    }

    fn run_write<F>(
        &self,
        operation: &str,
        mut write: F,
    ) -> Result<bool, AgentDbProjectionError>
    where
        F: FnMut(&rusqlite::Connection) -> Result<bool, rusqlite::Error>,
    {
        self.run_write_value(operation, |db| {
            write(db).map(|changed| (changed, changed))
        })
    }

    fn run_write_value<T, F>(
        &self,
        operation: &str,
        mut write: F,
    ) -> Result<T, AgentDbProjectionError>
    where
        T: Default,
        F: FnMut(&rusqlite::Connection) -> Result<(T, bool), rusqlite::Error>,
    {
        if self.is_closed() {
            return Ok(T::default());
        }
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let Some(db) = guard.as_ref() else {
            return Ok(T::default());
        };
        match write(db) {
            Ok((value, changed)) => {
                if changed {
                    bump_db_write_generation(&self.db_path);
                }
                return Ok(value);
            }
            Err(error) if is_sqlite_busy_error(&error) => {
                let message = error.to_string();
                drop(guard);
                if let Some(callback) = self.options.on_busy_error.as_ref() {
                    callback(operation, &message);
                }
                return Ok(T::default());
            }
            Err(error)
                if self.options.recovery.recover_on_corruption
                    && !self.recovered.load(Ordering::Acquire)
                    && is_sqlite_corrupt_error(&error) =>
            {
                let own_handle = usize::from(
                    self.handle_registered.load(Ordering::Acquire),
                );
                if live_db_handle_count(&self.db_path).saturating_sub(own_handle) > 0 {
                    return Err(error.into());
                }
                let cause = error.to_string();
                let old = guard.take();
                drop(old);
                match recover_corrupt_store_db(
                    &self.db_path,
                    &self.agent_dir_name,
                    &self.options.recovery,
                    &cause,
                ) {
                    Ok(recovered) => {
                        *guard = Some(recovered);
                        self.recovered.store(true, Ordering::Release);
                    }
                    Err(recovery_error) => {
                        self.closed.store(true, Ordering::Release);
                        if self
                            .handle_registered
                            .swap(false, Ordering::AcqRel)
                        {
                            release_live_db_handle(&self.db_path);
                        }
                        return Err(recovery_error.into());
                    }
                }
            }
            Err(error) => return Err(error.into()),
        }

        let Some(db) = guard.as_ref() else {
            return Ok(T::default());
        };
        match write(db) {
            Ok((value, changed)) => {
                if changed {
                    bump_db_write_generation(&self.db_path);
                }
                Ok(value)
            }
            Err(error) if is_sqlite_busy_error(&error) => {
                let message = error.to_string();
                drop(guard);
                if let Some(callback) = self.options.on_busy_error.as_ref() {
                    callback(operation, &message);
                }
                Ok(T::default())
            }
            Err(error) => Err(error.into()),
        }
    }

}

impl Drop for SandAgentDb {
    fn drop(&mut self) {
        self.close(false);
    }
}

fn session_work_result_receipt_entry(
    receipt: &SessionWorkResultReceipt,
    entry_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": entry_id,
        "kind": "background-result-receipt",
        "sessionKey": receipt.session_key,
        "lifecycleRevision": receipt.lifecycle_revision,
        "workId": receipt.work_id,
        "idempotencyKey": receipt.idempotency_key,
        "content": receipt.content,
        "sentMessageCount": receipt.sent_message_count,
    })
}

fn media_settlement_receipt_entry(
    entry_id: &str,
    media_work_id: &str,
    lifecycle_revision: &str,
    media_paths: &[String],
    transcript_entries: &[serde_json::Value],
) -> serde_json::Value {
    let transcript_entry_ids = transcript_entries
        .iter()
        .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    serde_json::json!({
        "id": entry_id,
        "kind": "outgoing-media-settlement",
        "mediaWorkId": media_work_id,
        "lifecycleRevision": lifecycle_revision,
        "mediaPaths": media_paths,
        "transcriptEntryIds": transcript_entry_ids,
    })
}

fn parse_session_work_result_receipt(
    entry: &serde_json::Value,
) -> Option<SessionWorkResultReceipt> {
    if entry.get("kind").and_then(serde_json::Value::as_str)
        != Some("background-result-receipt")
    {
        return None;
    }
    Some(SessionWorkResultReceipt {
        session_key: entry.get("sessionKey")?.as_str()?.to_string(),
        lifecycle_revision: entry.get("lifecycleRevision")?.as_str()?.to_string(),
        work_id: entry.get("workId")?.as_str()?.to_string(),
        idempotency_key: entry.get("idempotencyKey")?.as_str()?.to_string(),
        content: entry.get("content")?.as_str()?.to_string(),
        sent_message_count: usize::try_from(
            entry.get("sentMessageCount")?.as_u64()?,
        )
        .ok()?,
    })
}


fn default_agent_metadata(agent_dir_name: &str) -> serde_json::Value {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64;
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let blob_encryption_key = first
        .as_bytes()
        .iter()
        .chain(second.as_bytes().iter())
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    serde_json::json!({
        "agentId": agent_dir_name,
        "latestRootBlobId": "",
        "name": "New Agent",
        "mode": "default",
        "isRunEverything": false,
        "createdAt": created_at,
        "lifecycleRevision": Uuid::new_v4().to_string(),
        "blobEncryptionKey": blob_encryption_key,
    })
}

fn open_projection_db(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<rusqlite::Connection, AgentDbProjectionError> {
    let agent_dir_name = db_path
        .parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let options = DbRecoveryOptions {
        busy_timeout_ms,
        ..DbRecoveryOptions::default()
    };
    Ok(open_configured_db(
        db_path,
        &agent_dir_name,
        &options,
        live_db_handle_count(db_path) > 0,
    )?)
}

fn read_metadata_json(
    db: &rusqlite::Connection,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    let Some(raw) = read_kv(db, "metadata")? else {
        return Ok(None);
    };
    let metadata_bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
    let metadata_json = String::from_utf8(metadata_bytes)?;
    Ok(Some(serde_json::from_str(&metadata_json)?))
}

pub fn read_persisted_latest_root_blob_id(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Vec<u8>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(metadata) = read_metadata_json(&db)? else {
        return Ok(Vec::new());
    };
    let Some(root_hex) = metadata
        .get("latestRootBlobId")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(Vec::new());
    };
    decode_hex(root_hex).map_err(AgentDbProjectionError::LatestRootHex)
}

pub fn read_persisted_agent_metadata_projection(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Option<AgentMetadataProjection>, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    if owner.read_kv("metadata")?.is_none() {
        return Ok(None);
    }
    let metadata = owner.read_metadata()?;
    let lifecycle_revision = owner.ensure_lifecycle_revision()?;
    let agent_id = metadata
        .get("agentId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let created_at = metadata
        .get("createdAt")
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite())
        .unwrap_or_default();
    let latest_root_blob_id = metadata
        .get("latestRootBlobId")
        .and_then(serde_json::Value::as_str)
        .map(decode_hex)
        .transpose()
        .map_err(AgentDbProjectionError::LatestRootHex)?
        .unwrap_or_default();
    Ok(Some(AgentMetadataProjection {
        agent_id,
        created_at,
        latest_root_blob_id,
        lifecycle_revision,
    }))
}

pub fn read_persisted_agent_origin(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<String, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(if read_kv(&db, KV_ORIGIN)?.as_deref() == Some("dev") {
        "dev".to_string()
    } else {
        "user".to_string()
    })
}

pub fn read_persisted_introduction_pending(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(read_kv(&db, KV_INTRODUCTION)?.as_deref() == Some("1"))
}

pub fn read_persisted_agent_purpose(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Option<String>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(read_kv(&db, KV_PURPOSE)?
        .filter(|value| valid_agent_purpose(value)))
}

pub fn read_persisted_conversation_partner_ids(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Vec<String>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(raw) = read_kv(&db, KV_PARTNERS)? else {
        return Ok(Vec::new());
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Ok(Vec::new());
    };
    Ok(value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default())
}

pub fn add_persisted_conversation_partner(
    db_path: &Path,
    busy_timeout_ms: u64,
    partner_id: &str,
) -> Result<bool, AgentDbProjectionError> {
    let id = partner_id.trim();
    if id.is_empty() {
        return Ok(false);
    }
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let own_id = read_metadata_json(&transaction)?
        .and_then(|metadata| metadata.get("agentId").and_then(serde_json::Value::as_str).map(ToOwned::to_owned))
        .unwrap_or_default();
    if id == own_id {
        transaction.rollback()?;
        return Ok(false);
    }
    let current = transaction
        .query_row(GET_KV_SQL, params![KV_PARTNERS], |row| row.get::<_, String>(0))
        .optional()?;
    let mut values = current
        .as_deref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<std::collections::BTreeSet<_>>();
    if !values.insert(id.to_string()) {
        transaction.rollback()?;
        return Ok(false);
    }
    transaction.execute(
        SET_KV_SQL,
        params![KV_PARTNERS, serde_json::to_string(&values.into_iter().collect::<Vec<_>>())?],
    )?;
    transaction.commit()?;
    bump_db_write_generation(db_path);
    Ok(true)
}

fn valid_agent_purpose(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-'
        })
}

pub fn read_persisted_agent_name(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Option<String>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(read_metadata_json(&db)?
        .and_then(|metadata| {
            metadata
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        }))
}

pub fn set_persisted_agent_id(
    db_path: &Path,
    busy_timeout_ms: u64,
    agent_id: &str,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(raw) = read_kv(&db, "metadata")? else {
        return Ok(false);
    };
    let metadata_bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes)?;
    if metadata
        .get("agentId")
        .and_then(serde_json::Value::as_str)
        == Some(agent_id)
    {
        return Ok(true);
    }
    let Some(object) = metadata.as_object_mut() else {
        return Ok(false);
    };
    object.insert(
        "agentId".into(),
        serde_json::Value::String(agent_id.to_string()),
    );
    let next_raw = encode_hex(&serde_json::to_vec(&metadata)?);
    let changed = db.execute(SET_KV_SQL, params!["metadata", next_raw])? > 0;
    if changed {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(
            db_path,
            AgentDbListenerChannel::Metadata("agentId".into()),
        );
    }
    Ok(changed)
}

pub fn set_persisted_agent_name(
    db_path: &Path,
    busy_timeout_ms: u64,
    name: &str,
) -> Result<bool, AgentDbProjectionError> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(false);
    }
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(raw) = read_kv(&db, "metadata")? else {
        return Ok(false);
    };
    let metadata_bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes)?;
    if metadata
        .get("name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|current| current == name)
    {
        return Ok(false);
    }
    let Some(object) = metadata.as_object_mut() else {
        return Ok(false);
    };
    object.insert("name".into(), serde_json::Value::String(name.to_string()));
    let next_raw = encode_hex(&serde_json::to_vec(&metadata)?);
    let changed = db.execute(SET_KV_SQL, params!["metadata", next_raw])? > 0;
    if changed {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(
            db_path,
            AgentDbListenerChannel::Metadata("name".into()),
        );
    }
    Ok(changed)
}

pub fn reseed_minimal_persisted_agent_db_if_missing(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<(), AgentDbProjectionError> {
    if db_path.is_file() {
        return Ok(());
    }
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    drop(db);
    bump_db_write_generation(db_path);
    Ok(())
}

pub fn initialize_persisted_agent_record(
    db_path: &Path,
    busy_timeout_ms: u64,
    agent_id: &str,
    origin: &str,
    purpose: Option<&str>,
    created_at_ms: u64,
    blob_encryption_key_hex: &str,
) -> Result<(), AgentDbProjectionError> {
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let existing = transaction
        .query_row(GET_KV_SQL, params!["metadata"], |row| row.get::<_, String>(0))
        .optional()?;
    if existing.is_none() {
        let metadata = serde_json::json!({
            "agentId": agent_id,
            "latestRootBlobId": "",
            "name": "New Agent",
            "mode": "default",
            "isRunEverything": false,
            "createdAt": created_at_ms,
            "blobEncryptionKey": blob_encryption_key_hex,
        });
        transaction.execute(
            SET_KV_SQL,
            params!["metadata", encode_hex(&serde_json::to_vec(&metadata)?)],
        )?;
    }
    transaction.execute(
        SET_KV_SQL,
        params!["origin", if origin == "dev" { "dev" } else { "user" }],
    )?;
    transaction.execute(
        SET_KV_SQL,
        params!["introductionPending", "1"],
    )?;
    if let Some(purpose) = purpose.map(str::trim).filter(|value| !value.is_empty()) {
        transaction.execute(SET_KV_SQL, params!["purpose", purpose])?;
    }
    transaction.commit()?;
    bump_db_write_generation(db_path);
    Ok(())
}

pub fn compare_and_set_persisted_latest_root_blob_id(
    db_path: &Path,
    busy_timeout_ms: u64,
    expected_root: &[u8],
    next_root: &[u8],
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(raw) = read_kv(&db, "metadata")? else {
        return Ok(false);
    };
    let metadata_bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
    let metadata_json = String::from_utf8(metadata_bytes)?;
    let mut metadata: serde_json::Value = serde_json::from_str(&metadata_json)?;
    let current_root = metadata
        .get("latestRootBlobId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if current_root != encode_hex(expected_root) {
        return Ok(false);
    }
    let Some(object) = metadata.as_object_mut() else {
        return Ok(false);
    };
    object.insert(
        "latestRootBlobId".into(),
        serde_json::Value::String(encode_hex(next_root)),
    );
    let next_raw = encode_hex(&serde_json::to_vec(&metadata)?);
    let changed = db.execute(
        COMPARE_AND_SET_KV_SQL,
        params![next_raw, "metadata", raw],
    )? == 1;
    if changed {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(
            db_path,
            AgentDbListenerChannel::Metadata("latestRootBlobId".into()),
        );
    }
    Ok(changed)
}

pub fn read_persisted_version(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
) -> Result<u64, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(read_kv(&db, key)?
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_default())
}

pub fn write_persisted_version(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
    version: u64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let changed = db.execute(SET_KV_SQL, params![key, version.to_string()])? > 0;
    if changed {
        bump_db_write_generation(db_path);
    }
    Ok(changed)
}

pub fn stale_root_cleanup_version(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<u64, AgentDbProjectionError> {
    read_persisted_version(db_path, busy_timeout_ms, KV_STALE_ROOT_CLEANUP)
}

pub fn set_stale_root_cleanup_version(
    db_path: &Path,
    busy_timeout_ms: u64,
    version: u64,
) -> Result<bool, AgentDbProjectionError> {
    write_persisted_version(db_path, busy_timeout_ms, KV_STALE_ROOT_CLEANUP, version)
}

pub fn hidden_entry_repair_version(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<u64, AgentDbProjectionError> {
    read_persisted_version(db_path, busy_timeout_ms, KV_HIDDEN_REPAIR)
}

pub fn set_hidden_entry_repair_version(
    db_path: &Path,
    busy_timeout_ms: u64,
    version: u64,
) -> Result<bool, AgentDbProjectionError> {
    write_persisted_version(db_path, busy_timeout_ms, KV_HIDDEN_REPAIR, version)
}

pub fn legacy_blob_retirement_version(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<u64, AgentDbProjectionError> {
    read_persisted_version(db_path, busy_timeout_ms, KV_LEGACY_BLOB_RETIREMENT)
}

pub fn has_persisted_legacy_conversation_blobs(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(db
        .query_row(HAS_LEGACY_BLOB_SQL, [], |_| Ok(()))
        .optional()?
        .is_some())
}

pub fn retire_persisted_legacy_conversation_blobs(
    db_path: &Path,
    busy_timeout_ms: u64,
    version: u64,
) -> Result<bool, AgentDbProjectionError> {
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    transaction.execute(CLEAR_BLOBS_SQL, [])?;
    transaction.execute(
        SET_KV_SQL,
        params![KV_LEGACY_BLOB_RETIREMENT, version.to_string()],
    )?;
    transaction.commit()?;
    bump_db_write_generation(db_path);
    Ok(true)
}

pub fn read_persisted_transcript_entries(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Vec<serde_json::Value>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let mut statement = db.prepare(LIST_TRANSCRIPT_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        let raw = row?;
        if let Some(entry) = parse_transcript_entry(&raw) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub fn append_persisted_transcript_entries(
    db_path: &Path,
    busy_timeout_ms: u64,
    entries: &[serde_json::Value],
) -> Result<usize, AgentDbProjectionError> {
    if entries.is_empty() {
        return Ok(0);
    }
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let mut inserted = Vec::new();
    {
        let mut statement = transaction.prepare(INSERT_TRANSCRIPT_ENTRY_SQL)?;
        for entry in entries {
            let Some(id) = entry.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if entry.get("kind").and_then(serde_json::Value::as_str).is_none() {
                continue;
            }
            if statement.execute(params![id, entry.to_string()])? > 0 {
                inserted.push(entry.clone());
            }
        }
    }
    transaction.commit()?;
    if !inserted.is_empty() {
        bump_db_write_generation(db_path);
        publish_entries_upserted(db_path, inserted.clone());
    }
    Ok(inserted.len())
}

pub fn delete_persisted_transcript_entry(
    db_path: &Path,
    busy_timeout_ms: u64,
    entry_id: &str,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let changed = db.execute(DELETE_TRANSCRIPT_ENTRY_SQL, params![entry_id])? > 0;
    if changed {
        bump_db_write_generation(db_path);
        let mut mutation = serde_json::Map::new();
        mutation.insert("kind".into(), serde_json::Value::String("entry-deleted".into()));
        mutation.insert(
            "agentId".into(),
            serde_json::Value::String(agent_id_for_db_path(db_path)),
        );
        mutation.insert("entryId".into(), serde_json::Value::String(entry_id.to_string()));
        publish_transcript_mutation(&mutation);
    }
    Ok(changed)
}

pub fn read_persisted_automation_spend_guard_state(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<SpendGuardState, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let state = read_kv(&db, KV_SPEND_GUARD)?;
    let legacy = read_kv(&db, KV_SPEND_GUARD_LEGACY)?;
    Ok(resolve_spend_guard_state(state.as_deref(), legacy.as_deref()))
}

pub fn set_persisted_automation_spend_guard_state(
    db_path: &Path,
    busy_timeout_ms: u64,
    state: &SpendGuardState,
) -> Result<bool, AgentDbProjectionError> {
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let changed = if let Some(raw) = serialize_spend_guard_state(state) {
        transaction.execute(SET_KV_SQL, params![KV_SPEND_GUARD, raw])? > 0
    } else {
        transaction.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD])? > 0
    };
    let legacy_changed =
        transaction.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD_LEGACY])? > 0;
    transaction.commit()?;
    if changed || legacy_changed {
        bump_db_write_generation(db_path);
    }
    Ok(changed || legacy_changed)
}

pub fn read_persisted_agent_profile_prompt_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(raw) = read_kv(&db, KV_PROFILE_SNAPSHOT)? else {
        return Ok(None);
    };
    Ok(serde_json::from_str::<serde_json::Value>(&raw).ok())
}

pub fn clear_persisted_transient_state(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let mut changed = false;
    for key in [
        KV_UNREAD,
        KV_SPEND_GUARD_LEGACY,
        KV_SPEND_GUARD,
        KV_AWAITING,
        KV_LATEST_REQUEST_ID,
        KV_REQUEST_IDS,
        KV_EPISODE,
        KV_MEMORY_SNAPSHOT,
        KV_PROFILE_SNAPSHOT,
    ] {
        changed |= transaction.execute(DELETE_KV_SQL, params![key])? > 0;
    }
    transaction.commit()?;
    if changed {
        bump_db_write_generation(db_path);
    }
    Ok(changed)
}

pub fn read_persisted_agent_serde_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<AgentDbSerdeSnapshot, AgentDbProjectionError> {
    let agent_dir_name = db_path
        .parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let options = DbRecoveryOptions {
        busy_timeout_ms,
        ..DbRecoveryOptions::default()
    };
    let db = open_configured_db(
        db_path,
        &agent_dir_name,
        &options,
        live_db_handle_count(db_path) > 0,
    )?;

    let profile = read_kv(&db, KV_PROFILE)?;
    let unread = read_kv(&db, KV_UNREAD)?;
    let spend_guard = read_kv(&db, KV_SPEND_GUARD)?;
    let spend_guard_legacy = read_kv(&db, KV_SPEND_GUARD_LEGACY)?;
    let awaiting = read_kv(&db, KV_AWAITING)?;
    let request_ids_raw = read_kv(&db, KV_REQUEST_IDS)?;
    let legacy_request_id = read_kv(&db, KV_LATEST_REQUEST_ID)?;
    let episode = read_kv(&db, KV_EPISODE)?;
    let memory_snapshot = read_kv(&db, KV_MEMORY_SNAPSHOT)?;

    let mut request_ids = parse_request_records(request_ids_raw.as_deref());
    if request_ids.is_empty() {
        if let Some(legacy) = legacy_request_id.as_deref() {
            let id = legacy.trim();
            if !id.is_empty() {
                request_ids.push(RequestRecord {
                    id: id.to_string(),
                    at: 0.0,
                    prompt: None,
                    source: None,
                });
            }
        }
    }

    Ok(AgentDbSerdeSnapshot {
        profile: parse_profile(profile.as_deref()),
        unread_state: parse_unread_state(unread.as_deref()),
        spend_guard_state: resolve_spend_guard_state(
            spend_guard.as_deref(),
            spend_guard_legacy.as_deref(),
        ),
        awaiting_user_response: parse_awaiting_state(awaiting.as_deref()),
        request_ids,
        pending_episode_turns: parse_pending_episode_turns(episode.as_deref()),
        memory_prompt_snapshot: parse_memory_prompt_snapshot(memory_snapshot.as_deref()),
    })
}

pub fn set_persisted_sand_profile(
    db_path: &Path,
    busy_timeout_ms: u64,
    profile: &SandProfile,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_profile(read_kv(&db, KV_PROFILE)?.as_deref());
    if current == *profile {
        return Ok(true);
    }
    let raw = serde_json::json!({
        "description": profile.description.clone(),
        "avatarPath": profile.avatar_path.clone(),
    })
    .to_string();
    let wrote = db.execute(SET_KV_SQL, params![KV_PROFILE, raw])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(db_path, AgentDbListenerChannel::Profile);
    }
    Ok(wrote)
}

fn write_unread_state(
    db_path: &Path,
    busy_timeout_ms: u64,
    state: &UnreadState,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let raw = serde_json::json!({
        "lastActivityAt": state.last_activity_at,
        "lastViewedAt": state.last_viewed_at,
        "isManuallyUnread": state.is_manually_unread,
        "unreadCount": state.unread_count,
    })
    .to_string();
    let wrote = db.execute(SET_KV_SQL, params![KV_UNREAD, raw])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
    }
    Ok(wrote)
}

pub fn mark_persisted_activity(
    db_path: &Path,
    busy_timeout_ms: u64,
    at: f64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_unread_state(read_kv(&db, KV_UNREAD)?.as_deref());
    drop(db);
    if current.last_activity_at >= at {
        return Ok(false);
    }
    write_unread_state(
        db_path,
        busy_timeout_ms,
        &UnreadState {
            last_activity_at: at,
            unread_count: current.unread_count + 1.0,
            ..current
        },
    )
}

pub fn mark_persisted_viewed(
    db_path: &Path,
    busy_timeout_ms: u64,
    at: f64,
    preserve_manual_unread: bool,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_unread_state(read_kv(&db, KV_UNREAD)?.as_deref());
    drop(db);
    if preserve_manual_unread && current.is_manually_unread {
        return Ok(false);
    }
    if !current.is_manually_unread && current.last_viewed_at >= at {
        return Ok(false);
    }
    write_unread_state(
        db_path,
        busy_timeout_ms,
        &UnreadState {
            last_viewed_at: at,
            is_manually_unread: false,
            unread_count: 0.0,
            ..current
        },
    )
}

fn newest_divider_anchor_timestamp_ms(
    db: &rusqlite::Connection,
) -> Result<f64, rusqlite::Error> {
    Ok(db
        .query_row(NEWEST_DIVIDER_ANCHOR_TIMESTAMP_SQL, [], |row| {
            row.get::<_, Option<f64>>(0)
        })
        .optional()?
        .flatten()
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or_default())
}

pub fn read_persisted_newest_divider_anchor_timestamp_ms(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<f64, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    Ok(newest_divider_anchor_timestamp_ms(&db)?)
}

pub fn mark_persisted_unread(
    db_path: &Path,
    busy_timeout_ms: u64,
    at: f64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_unread_state(read_kv(&db, KV_UNREAD)?.as_deref());
    let newest = newest_divider_anchor_timestamp_ms(&db)?;
    drop(db);
    let last_activity_at = if current.last_activity_at == 0.0 {
        at
    } else {
        current.last_activity_at
    };
    let newest_bound = if newest > 0.0 {
        newest - 1.0
    } else {
        f64::INFINITY
    };
    let last_viewed_at = current
        .last_viewed_at
        .min(last_activity_at - 1.0)
        .min(at - 1.0)
        .min(newest_bound);
    write_unread_state(
        db_path,
        busy_timeout_ms,
        &UnreadState {
            last_activity_at,
            last_viewed_at,
            is_manually_unread: true,
            unread_count: current.unread_count.max(1.0),
        },
    )
}

pub fn mark_persisted_read(
    db_path: &Path,
    busy_timeout_ms: u64,
    at: f64,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_unread_state(read_kv(&db, KV_UNREAD)?.as_deref());
    drop(db);
    write_unread_state(
        db_path,
        busy_timeout_ms,
        &UnreadState {
            last_viewed_at: current.last_activity_at.max(at),
            is_manually_unread: false,
            unread_count: 0.0,
            ..current
        },
    )
}

pub fn set_persisted_awaiting_user_response(
    db_path: &Path,
    busy_timeout_ms: u64,
    state: Option<&AwaitingUserResponse>,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_awaiting_state(read_kv(&db, KV_AWAITING)?.as_deref());
    if current.as_ref() == state {
        return Ok(false);
    }
    let wrote = if let Some(state) = state {
        let raw = serde_json::json!({
            "tabId": state.tab_id.clone(),
            "reason": state.reason.clone(),
            "since": state.since,
        })
        .to_string();
        db.execute(SET_KV_SQL, params![KV_AWAITING, raw])? > 0
    } else {
        db.execute(DELETE_KV_SQL, params![KV_AWAITING])? > 0
    };
    if wrote {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(db_path, AgentDbListenerChannel::Awaiting);
    }
    Ok(wrote)
}

pub fn set_persisted_awaiting_user_response_for_tab(
    db_path: &Path,
    busy_timeout_ms: u64,
    tab_id: &str,
    state: Option<&AwaitingUserResponse>,
    if_since_before: Option<f64>,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let current = parse_awaiting_state(read_kv(&db, KV_AWAITING)?.as_deref());
    drop(db);
    if current.as_ref().is_some_and(|current| current.tab_id != tab_id) {
        return Ok(false);
    }
    if state.is_none()
        && (current.is_none()
            || if_since_before.is_some_and(|before| {
                current.as_ref().is_some_and(|current| current.since >= before)
            }))
    {
        return Ok(false);
    }
    if state.is_some_and(|state| state.tab_id != tab_id) {
        return Ok(false);
    }
    set_persisted_awaiting_user_response(db_path, busy_timeout_ms, state)
}

pub fn record_persisted_request_id(
    db_path: &Path,
    busy_timeout_ms: u64,
    id: &str,
    at: f64,
    prompt: Option<&str>,
    source: Option<&str>,
) -> Result<bool, AgentDbProjectionError> {
    let id = id.trim();
    if id.is_empty() {
        return Ok(false);
    }
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let mut records = parse_request_records(read_kv(&db, KV_REQUEST_IDS)?.as_deref());
    if records.is_empty() {
        if let Some(legacy) = read_kv(&db, KV_LATEST_REQUEST_ID)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            records.push(RequestRecord {
                id: legacy,
                at: 0.0,
                prompt: None,
                source: None,
            });
        }
    }
    if records.last().is_some_and(|record| record.id == id) {
        return Ok(false);
    }
    let label = prompt
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(REQUEST_ID_PROMPT_MAX).collect::<String>());
    records.push(RequestRecord {
        id: id.to_string(),
        at,
        prompt: label,
        source: source.filter(|value| !value.is_empty()).map(ToOwned::to_owned),
    });
    if records.len() > REQUEST_ID_HISTORY_MAX {
        records.drain(0..records.len() - REQUEST_ID_HISTORY_MAX);
    }
    let raw = serde_json::Value::Array(
        records
            .iter()
            .map(|record| {
                let mut value = serde_json::Map::new();
                value.insert("id".into(), serde_json::Value::String(record.id.clone()));
                value.insert("at".into(), json_number(record.at));
                if let Some(prompt) = &record.prompt {
                    value.insert("prompt".into(), serde_json::Value::String(prompt.clone()));
                }
                if let Some(source) = &record.source {
                    value.insert("source".into(), serde_json::Value::String(source.clone()));
                }
                serde_json::Value::Object(value)
            })
            .collect(),
    )
    .to_string();
    let wrote = db.execute(SET_KV_SQL, params![KV_REQUEST_IDS, raw])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
    }
    Ok(wrote)
}

pub fn record_persisted_episode_turn(
    db_path: &Path,
    busy_timeout_ms: u64,
    turn: &EpisodeTurn,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let mut turns = parse_pending_episode_turns(read_kv(&db, KV_EPISODE)?.as_deref());
    turns.push(EpisodeTurn {
        ts: turn.ts,
        user: turn.user.chars().take(EPISODE_TURN_TEXT_CAP).collect(),
        agent: turn.agent.chars().take(EPISODE_TURN_TEXT_CAP).collect(),
    });
    if turns.len() > EPISODE_PENDING_MAX {
        turns.drain(0..turns.len() - EPISODE_PENDING_MAX);
    }
    let raw = serde_json::Value::Array(
        turns
            .iter()
            .map(|turn| {
                serde_json::json!({
                    "ts": turn.ts,
                    "user": turn.user.clone(),
                    "agent": turn.agent.clone(),
                })
            })
            .collect(),
    )
    .to_string();
    let wrote = db.execute(SET_KV_SQL, params![KV_EPISODE, raw])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
    }
    Ok(wrote)
}

pub fn clear_persisted_pending_episode_turns(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    delete_persisted_kv(db_path, busy_timeout_ms, KV_EPISODE)
}

pub fn set_persisted_memory_prompt_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
    snapshot: &serde_json::Value,
) -> Result<bool, AgentDbProjectionError> {
    write_persisted_kv(db_path, busy_timeout_ms, KV_MEMORY_SNAPSHOT, &snapshot.to_string())
}

pub fn clear_persisted_memory_prompt_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    delete_persisted_kv(db_path, busy_timeout_ms, KV_MEMORY_SNAPSHOT)
}

pub fn set_persisted_agent_profile_prompt_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
    snapshot: &serde_json::Value,
) -> Result<bool, AgentDbProjectionError> {
    write_persisted_kv(db_path, busy_timeout_ms, KV_PROFILE_SNAPSHOT, &snapshot.to_string())
}

pub fn clear_persisted_agent_profile_prompt_snapshot(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    delete_persisted_kv(db_path, busy_timeout_ms, KV_PROFILE_SNAPSHOT)
}

pub fn set_persisted_agent_origin(
    db_path: &Path,
    busy_timeout_ms: u64,
    origin: &str,
) -> Result<bool, AgentDbProjectionError> {
    write_persisted_kv(
        db_path,
        busy_timeout_ms,
        KV_ORIGIN,
        if origin == "dev" { "dev" } else { "user" },
    )
}

pub fn set_persisted_introduction_pending(
    db_path: &Path,
    busy_timeout_ms: u64,
    pending: bool,
) -> Result<bool, AgentDbProjectionError> {
    if pending {
        write_persisted_kv(db_path, busy_timeout_ms, KV_INTRODUCTION, "1")
    } else {
        delete_persisted_kv(db_path, busy_timeout_ms, KV_INTRODUCTION)
    }
}

pub fn set_persisted_agent_purpose(
    db_path: &Path,
    busy_timeout_ms: u64,
    purpose: Option<&str>,
) -> Result<bool, AgentDbProjectionError> {
    if let Some(purpose) = purpose {
        write_persisted_kv(db_path, busy_timeout_ms, KV_PURPOSE, purpose)
    } else {
        delete_persisted_kv(db_path, busy_timeout_ms, KV_PURPOSE)
    }
}

pub fn read_persisted_transcript_entry(
    db_path: &Path,
    busy_timeout_ms: u64,
    entry_id: &str,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let raw = db
        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    Ok(raw.as_deref().and_then(parse_transcript_entry))
}

pub fn update_persisted_transcript_entry(
    db_path: &Path,
    busy_timeout_ms: u64,
    entry_id: &str,
    next: &serde_json::Value,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    if read_persisted_transcript_entry(db_path, busy_timeout_ms, entry_id)?.is_none() {
        return Ok(None);
    }
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let changed = db.execute(
        UPDATE_TRANSCRIPT_ENTRY_SQL,
        params![next.to_string(), entry_id],
    )? > 0;
    if !changed {
        return Ok(None);
    }
    bump_db_write_generation(db_path);
    publish_entries_upserted(db_path, vec![next.clone()]);
    Ok(Some(next.clone()))
}

pub fn clear_persisted_conversation(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let raw_metadata = read_kv(&db, "metadata")?;
    let next_metadata = if let Some(raw) = raw_metadata.as_deref() {
        let bytes = decode_hex(raw).map_err(AgentDbProjectionError::MetadataHex)?;
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(AgentDbProjectionError::MetadataJson)?;
        if let Some(object) = metadata.as_object_mut() {
            object.insert(
                "latestRootBlobId".into(),
                serde_json::Value::String(String::new()),
            );
            object.insert(
                "currentPlanUri".into(),
                serde_json::Value::String(String::new()),
            );
        }
        Some(encode_hex(&serde_json::to_vec(&metadata)?))
    } else {
        None
    };

    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    transaction.execute(CLEAR_BLOBS_SQL, [])?;
    transaction.execute(CLEAR_TRANSCRIPT_ENTRIES_SQL, [])?;
    for key in [
        KV_AWAITING,
        KV_LATEST_REQUEST_ID,
        KV_REQUEST_IDS,
        KV_EPISODE,
        KV_MEMORY_SNAPSHOT,
        KV_PROFILE_SNAPSHOT,
    ] {
        transaction.execute(DELETE_KV_SQL, params![key])?;
    }
    if let Some(next_metadata) = next_metadata {
        transaction.execute(SET_KV_SQL, params!["metadata", next_metadata])?;
    }
    transaction.commit()?;
    bump_db_write_generation(db_path);
    let mut mutation = serde_json::Map::new();
    mutation.insert(
        "kind".into(),
        serde_json::Value::String("conversation-cleared".into()),
    );
    mutation.insert(
        "agentId".into(),
        serde_json::Value::String(agent_id_for_db_path(db_path)),
    );
    publish_transcript_mutation(&mutation);
    notify_agent_db_listeners(db_path, AgentDbListenerChannel::Awaiting);
    notify_agent_db_listeners(
        db_path,
        AgentDbListenerChannel::Metadata("latestRootBlobId".into()),
    );
    Ok(true)
}

fn write_persisted_kv(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
    value: &str,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let wrote = db.execute(SET_KV_SQL, params![key, value])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
    }
    Ok(wrote)
}

fn delete_persisted_kv(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
) -> Result<bool, AgentDbProjectionError> {
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let wrote = db.execute(DELETE_KV_SQL, params![key])? > 0;
    if wrote {
        bump_db_write_generation(db_path);
    }
    Ok(wrote)
}


fn read_transcript_entries_from_db(
    db: &rusqlite::Connection,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
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

fn compaction_payload_matches(
    existing: &serde_json::Value,
    commit: &CompactionCommit,
) -> bool {
    existing.get("kind").and_then(serde_json::Value::as_str) == Some("compaction")
        && existing.get("id").and_then(serde_json::Value::as_str) == Some(commit.entry_id.trim())
        && existing
            .get("firstKeptEntryId")
            .and_then(serde_json::Value::as_str)
            == commit.first_kept_entry_id.as_deref().map(str::trim)
        && existing.get("summary").and_then(serde_json::Value::as_str)
            == Some(commit.summary.as_str())
        && existing
            .get("tokensBefore")
            .and_then(serde_json::Value::as_u64)
            == Some(commit.tokens_before)
        && existing.get("details") == commit.details.as_ref()
        && existing.get("sourceTurnId").and_then(serde_json::Value::as_str)
            == Some(commit.source_turn_id.trim())
}

fn read_transcript_entry_from_db(
    db: &rusqlite::Connection,
    entry_id: &str,
) -> Result<Option<serde_json::Value>, rusqlite::Error> {
    let raw = db
        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    Ok(raw.as_deref().and_then(parse_transcript_entry))
}

fn read_branched_entries_from_db(
    db: &rusqlite::Connection,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
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

fn branch_reply_counts_from_entries(
    branched: &[serde_json::Value],
) -> HashMap<String, usize> {
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
        if let Some(root) = resolve_branch_root_from_entries(entry, &by_id) {
            *counts.entry(root).or_insert(0) += 1;
        }
    }
    counts
}

fn thread_descendants_from_entries(
    root_id: &str,
    branched: &[serde_json::Value],
) -> Vec<serde_json::Value> {
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
        .filter(|entry| {
            resolve_branch_root_from_entries(entry, &by_id).as_deref() == Some(root_id)
        })
        .cloned()
        .collect()
}

fn resolve_branch_root_from_entries(
    entry: &serde_json::Value,
    branched_by_id: &HashMap<String, &serde_json::Value>,
) -> Option<String> {
    let mut current = entry;
    let mut seen = std::collections::HashSet::new();
    if let Some(id) = current.get("id").and_then(serde_json::Value::as_str) {
        seen.insert(id.to_string());
    }
    loop {
        let parent_id = current.get("replyTo").and_then(serde_json::Value::as_str)?;
        let Some(parent) = branched_by_id.get(parent_id) else {
            return Some(parent_id.to_string());
        };
        if !seen.insert(parent_id.to_string()) {
            return None;
        }
        current = parent;
    }
}

fn main_transcript_entries_from_entries(
    entries: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    let by_id = entries
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("id")?.as_str()?.to_string(),
                entry,
            ))
        })
        .collect::<HashMap<_, _>>();
    let is_thread_message = |entry: &serde_json::Value| {
        is_main_branched_entry(entry)
            && resolve_main_branch_root(entry, &by_id).is_some()
    };
    if !entries.iter().any(is_thread_message) {
        return entries.to_vec();
    }
    entries
        .iter()
        .filter(|entry| !is_thread_message(entry))
        .cloned()
        .collect()
}

fn is_main_branched_entry(entry: &serde_json::Value) -> bool {
    matches!(
        entry.get("kind").and_then(serde_json::Value::as_str),
        Some("message" | "send-message" | "user-attachment" | "notice")
    ) && entry.get("branched").and_then(serde_json::Value::as_bool) == Some(true)
}

fn main_entry_reply_to(entry: &serde_json::Value) -> Option<&str> {
    if !matches!(
        entry.get("kind").and_then(serde_json::Value::as_str),
        Some("message" | "send-message" | "user-attachment" | "notice")
    ) {
        return None;
    }
    entry.get("replyTo").and_then(serde_json::Value::as_str)
}

fn resolve_main_branch_root(
    entry: &serde_json::Value,
    by_id: &HashMap<String, &serde_json::Value>,
) -> Option<String> {
    let mut current = entry;
    let mut seen = std::collections::HashSet::new();
    if let Some(id) = current.get("id").and_then(serde_json::Value::as_str) {
        seen.insert(id.to_string());
    }
    loop {
        let parent_id = main_entry_reply_to(current)?;
        let parent = by_id.get(parent_id)?;
        if !is_main_branched_entry(parent) {
            return Some(parent_id.to_string());
        }
        if !seen.insert(parent_id.to_string()) {
            return None;
        }
        current = parent;
    }
}

fn publish_entries_upserted(db_path: &Path, entries: Vec<serde_json::Value>) {
    let mut mutation = serde_json::Map::new();
    mutation.insert(
        "kind".into(),
        serde_json::Value::String("entries-upserted".into()),
    );
    mutation.insert(
        "agentId".into(),
        serde_json::Value::String(agent_id_for_db_path(db_path)),
    );
    mutation.insert("entries".into(), serde_json::Value::Array(entries));
    publish_transcript_mutation(&mutation);
}

fn agent_id_for_db_path(db_path: &Path) -> String {
    db_path
        .parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn json_number(value: f64) -> serde_json::Value {
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

pub fn read_persisted_transcript_tail(
    db_path: &Path,
    busy_timeout_ms: u64,
    limit: i64,
) -> Result<TranscriptPage, AgentDbProjectionError> {
    let agent_dir_name = db_path
        .parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let options = DbRecoveryOptions {
        busy_timeout_ms,
        ..DbRecoveryOptions::default()
    };
    let db = open_configured_db(
        db_path,
        &agent_dir_name,
        &options,
        live_db_handle_count(db_path) > 0,
    )?;
    Ok(read_transcript_tail(
        &db,
        TranscriptWindowQuery {
            before_seq: None,
            limit,
        },
    )?)
}

fn read_kv(
    db: &rusqlite::Connection,
    key: &str,
) -> Result<Option<String>, rusqlite::Error> {
    db.query_row(GET_KV_SQL, params![key], |row| row.get::<_, String>(0))
        .optional()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(raw: &str) -> Result<Vec<u8>, String> {
    let clean = raw.trim();
    if clean.len() % 2 != 0 {
        return Err("odd-length hex string".into());
    }
    let mut output = Vec::with_capacity(clean.len() / 2);
    for index in (0..clean.len()).step_by(2) {
        let pair = &clean[index..index + 2];
        let byte = u8::from_str_radix(pair, 16)
            .map_err(|_| format!("invalid hex pair at offset {index}: {pair}"))?;
        output.push(byte);
    }
    Ok(output)
}
