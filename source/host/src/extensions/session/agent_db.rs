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
use crate::extensions::memory::memory_service::MemoryRecoveryRecord;
use crate::transcript_mutation_events::publish_transcript_mutation;

use super::agent_db_recovery::{
    AgentDbRecoveryError, DbRecoveryOptions, acquire_automation_custody_recovery_guard,
    open_configured_db, recover_corrupt_store_db,
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
    is_valid_transcript_entry, parse_memory_prompt_snapshot, parse_owner_transcript_entry,
    parse_pending_episode_turns, parse_profile,
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
    #[error("fresh transcript append is invalid: {0}")]
    TranscriptAppendInvalid(String),
    #[error("fresh transcript commit outcome is unknown; do not replay: {0}")]
    TranscriptCommitUnknown(String),
    #[error("latestRootBlobId hex is invalid: {0}")]
    LatestRootHex(String),
    #[error("canonical Agent checkpoint root is invalid: {0}")]
    CheckpointRootInvalid(String),
    #[error("original turn terminal data is invalid: {0}")]
    OriginalTurnInvalid(String),
    #[error("original turn terminal authority is unavailable: {0}")]
    OriginalTurnUnavailable(String),
    #[error("original turn terminal fence changed: {0}")]
    OriginalTurnStale(String),
    #[error("original turn terminal identity conflicts: {0}")]
    OriginalTurnConflict(String),
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
const KV_PENDING_MEMORY_MUTATION: &str = "pendingExternalMemoryMutation";
const KV_PENDING_PROFILE_MUTATION: &str = "pendingExternalProfileMutation";
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
const MAX_COMPACTION_SUMMARY_CHARS: usize = 16_000;

pub(crate) const ORIGINAL_TURN_TERMINAL_PREFIX: &str = "original-routed-terminal:";
pub(crate) const ORIGINAL_TURN_TERMINAL_KEY: &str = "original-routed-terminal:current";
pub(crate) const ORIGINAL_TURN_TERMINAL_MAX_BYTES: usize = 64 * 1024;
pub(crate) const ORIGINAL_TURN_MAX_COMMITS: usize = 5;

/// Identity data captured by the admitted Transcript lease. Possessing or
/// deserializing this reference is not proof that a turn completed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OriginalTurnReference {
    pub version: u16,
    pub scope: OriginalTurnScope,
    pub agent_id: String,
    pub session_key: String,
    pub session_id: String,
    pub original_lifecycle_revision: String,
    pub stream_id: String,
    pub input_digest: String,
    pub client_nonce: Option<String>,
    pub user_message_id: String,
    pub echo_entry_id: String,
    pub request_source: Option<String>,
    pub provider: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OriginalTurnScope {
    #[serde(rename = "nativeOrdinaryUserV1")]
    NativeOrdinaryUserV1,
}

impl OriginalTurnReference {
    /// Structural validation only; Session and Runner independently check the
    /// actual owner rows, canonical blobs and sealed completed observations.
    pub fn validate(&self) -> Result<(), String> {
        let bounded_identity = |value: &str| {
            !value.trim().is_empty()
                && value.len() <= 1024
                && !value.chars().any(char::is_control)
        };
        if self.version != 1
            || self.provider != "fabushi"
            || self.session_id != self.agent_id
            || self.session_key != format!("agent:{}:main", self.agent_id)
            || self.echo_entry_id != self.user_message_id
            || self.request_source.as_deref().is_some_and(|source| source != "turn")
            || self.input_digest.len() != 64
            || !self.input_digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || [
                self.agent_id.as_str(), self.session_key.as_str(), self.session_id.as_str(),
                self.original_lifecycle_revision.as_str(), self.stream_id.as_str(),
                self.user_message_id.as_str(), self.echo_entry_id.as_str(), self.model_id.as_str(),
            ].into_iter().any(|value| !bounded_identity(value))
            || self.client_nonce.as_deref().is_some_and(|value| !bounded_identity(value))
        {
            return Err("unsupported or inconsistent original turn reference".into());
        }
        Ok(())
    }
}

/// Session persists neutral records; completion and tool-custody policy stays
/// in Runner. These records cannot be submitted through a public raw writer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanonicalTurnCommitRecord {
    pub(crate) ordinal: u32,
    pub(crate) request_id: String,
    pub(crate) message_id: String,
    pub(crate) prior_root_blob_id: Option<Vec<u8>>,
    pub(crate) root_blob_id: Vec<u8>,
    pub(crate) user_blob_id: Vec<u8>,
    pub(crate) turn_blob_id: Vec<u8>,
    pub(crate) step_blob_ids: Vec<Vec<u8>>,
    pub(crate) assistant_result_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SameTurnCompactionBinding {
    pub(crate) entry_id: String,
    pub(crate) source_turn_id: String,
    pub(crate) source_lifecycle_revision: String,
    pub(crate) lifecycle_revision: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StoredCompletedTerminal {
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OriginalTurnTerminalRecord {
    pub(crate) version: u16,
    pub(crate) original: OriginalTurnReference,
    pub(crate) terminal: StoredCompletedTerminal,
    pub(crate) final_result_sha256: [u8; 32],
    pub(crate) sent_message_count: u64,
    pub(crate) reacted: bool,
    pub(crate) custody_json: String,
    pub(crate) canonical: Vec<CanonicalTurnCommitRecord>,
    pub(crate) settlement_lifecycle_revision: String,
    pub(crate) compaction: Option<SameTurnCompactionBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalTurnReceiptCommit {
    Inserted,
    AlreadyCommitted,
    Replaced,
}

/// One inspected SQLite image, not a completed-turn capability. Blob checks
/// happen outside the connection mutex; Session brings this exact image back
/// to the same owner before publishing or returning a verified candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OriginalTurnTerminalFence {
    original: OriginalTurnReference,
    pub(super) final_root_blob_id: Vec<u8>,
    pub(super) original_user_content: String,
    pub(super) settlement_lifecycle_revision: String,
    pub(super) compaction: Option<SameTurnCompactionBinding>,
    raw_metadata: String,
    raw_admission: String,
    raw_user: String,
    raw_compaction: Option<String>,
    raw_slot: Option<String>,
}

impl OriginalTurnTerminalFence {
    pub(super) fn contains_record(&self, record: &OriginalTurnTerminalRecord) -> bool {
        self.raw_slot.as_deref()
            .and_then(|raw| serde_json::from_str::<OriginalTurnTerminalRecord>(raw).ok())
            .as_ref() == Some(record)
    }
}

struct OriginalTurnTerminalSlot {
    raw: String,
    record: OriginalTurnTerminalRecord,
}

fn reject_original_turn_terminal_key(key: &str) -> Result<(), AgentDbProjectionError> {
    if key.starts_with(ORIGINAL_TURN_TERMINAL_PREFIX) {
        return Err(AgentDbProjectionError::OriginalTurnInvalid(
            "terminal slots are reserved for the Session terminal transaction".into(),
        ));
    }
    Ok(())
}

fn clear_original_turn_terminal_slots(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute(
        "DELETE FROM kv WHERE substr(key, 1, ?1) = ?2",
        params![ORIGINAL_TURN_TERMINAL_PREFIX.len() as i64, ORIGINAL_TURN_TERMINAL_PREFIX],
    )
}

fn validate_original_turn_terminal_record(
    record: &OriginalTurnTerminalRecord,
) -> Result<(), AgentDbProjectionError> {
    record.original.validate().map_err(AgentDbProjectionError::OriginalTurnInvalid)?;
    if record.version != 1
        || record.canonical.is_empty()
        || record.canonical.len() > ORIGINAL_TURN_MAX_COMMITS
        || record.settlement_lifecycle_revision.trim().is_empty()
        || record.settlement_lifecycle_revision.len() > 1024
        || record.settlement_lifecycle_revision.chars().any(char::is_control)
        || record.custody_json.trim().is_empty()
    {
        return Err(AgentDbProjectionError::OriginalTurnInvalid(
            "unsupported terminal version, commit count or settlement identity".into(),
        ));
    }
    for (ordinal, commit) in record.canonical.iter().enumerate() {
        let message_id = if ordinal == 0 {
            record.original.user_message_id.as_str()
        } else {
            record.original.stream_id.as_str()
        };
        if commit.ordinal as usize != ordinal
            || commit.request_id != record.original.stream_id
            || commit.message_id != message_id
            || commit.root_blob_id.len() != 32
            || commit.user_blob_id.len() != 32
            || commit.turn_blob_id.len() != 32
            || commit.step_blob_ids.iter().any(|id| id.len() != 32)
            || commit.prior_root_blob_id.as_ref().is_some_and(Vec::is_empty)
            || (ordinal > 0 && commit.prior_root_blob_id.as_ref()
                != Some(&record.canonical[ordinal - 1].root_blob_id))
        {
            return Err(AgentDbProjectionError::OriginalTurnInvalid(
                "terminal canonical references have an invalid identity or order".into(),
            ));
        }
    }
    match record.compaction.as_ref() {
        None if record.settlement_lifecycle_revision == record.original.original_lifecycle_revision => {}
        Some(binding) if binding.entry_id == format!(
                "compaction:{}:{}", record.original.agent_id, record.original.stream_id,
            )
            && binding.source_turn_id == record.original.stream_id
            && binding.source_lifecycle_revision == record.original.original_lifecycle_revision
            && binding.lifecycle_revision == record.settlement_lifecycle_revision
            && binding.lifecycle_revision != binding.source_lifecycle_revision => {}
        _ => return Err(AgentDbProjectionError::OriginalTurnInvalid(
            "terminal settlement is not the original or its bound compaction successor".into(),
        )),
    }
    // Custody is deliberately opaque here. Runner alone parses and qualifies
    // its existing typed report and the actual canonical blob contents.
    Ok(())
}

fn read_original_turn_terminal_slot(
    db: &rusqlite::Connection,
) -> Result<Option<OriginalTurnTerminalSlot>, AgentDbProjectionError> {
    let stored = db.query_row(
        "SELECT length(CAST(value AS BLOB)), \
         CASE WHEN length(CAST(value AS BLOB)) <= ?1 THEN value ELSE NULL END \
         FROM kv WHERE key = ?2",
        params![ORIGINAL_TURN_TERMINAL_MAX_BYTES as i64, ORIGINAL_TURN_TERMINAL_KEY],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
    ).optional()?;
    let Some((byte_length, raw)) = stored else { return Ok(None); };
    let raw = raw.filter(|raw| byte_length >= 0
        && byte_length as usize == raw.len()
        && raw.len() <= ORIGINAL_TURN_TERMINAL_MAX_BYTES)
        .ok_or_else(|| AgentDbProjectionError::OriginalTurnInvalid(
            "stored terminal slot exceeds its encoded byte limit".into(),
        ))?;
    let record = serde_json::from_str::<OriginalTurnTerminalRecord>(&raw)
        .map_err(|error| AgentDbProjectionError::OriginalTurnInvalid(format!(
            "stored terminal slot is malformed: {error}",
        )))?;
    validate_original_turn_terminal_record(&record)?;
    Ok(Some(OriginalTurnTerminalSlot { raw, record }))
}

fn capture_original_turn_terminal_fence(
    db: &rusqlite::Connection,
    original: &OriginalTurnReference,
    expected_final_root: &[u8],
    slot: Option<&OriginalTurnTerminalSlot>,
) -> Result<OriginalTurnTerminalFence, AgentDbProjectionError> {
    original.validate().map_err(AgentDbProjectionError::OriginalTurnInvalid)?;
    if expected_final_root.is_empty() {
        return Err(AgentDbProjectionError::OriginalTurnInvalid(
            "terminal completion requires a present canonical root".into(),
        ));
    }
    let raw_metadata = read_kv(db, "metadata")?.ok_or_else(|| {
        AgentDbProjectionError::OriginalTurnUnavailable("persisted metadata is missing".into())
    })?;
    let metadata: serde_json::Value = serde_json::from_slice(
        &decode_hex(&raw_metadata).map_err(AgentDbProjectionError::MetadataHex)?,
    )?;
    if !metadata.get("lifecycleRevision").and_then(serde_json::Value::as_str)
        .is_some_and(|revision| !revision.trim().is_empty()
            && revision.len() <= 1024 && !revision.chars().any(char::is_control))
    {
        return Err(AgentDbProjectionError::OriginalTurnInvalid(
            "persisted metadata has no valid independent lifecycle token".into(),
        ));
    }
    let root = metadata.get("latestRootBlobId").and_then(serde_json::Value::as_str)
        .ok_or_else(|| AgentDbProjectionError::OriginalTurnInvalid(
            "persisted metadata requires a canonical root string".into(),
        ))?;
    let root = decode_hex(root).map_err(AgentDbProjectionError::LatestRootHex)?;
    if metadata.get("agentId").and_then(serde_json::Value::as_str)
            != Some(original.agent_id.as_str())
        || root.as_slice() != expected_final_root
    {
        return Err(AgentDbProjectionError::OriginalTurnStale(
            "canonical agent or root differs from the completed original".into(),
        ));
    }
    let raw_admission = read_kv(db, KV_SESSION_WORK_ADMISSION)?.ok_or_else(|| {
        AgentDbProjectionError::OriginalTurnUnavailable("terminal admission is missing".into())
    })?;
    let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
    if !session_admission_identity_is_well_formed(&admission)
        || admission.session_key != original.session_key
        || admission.session_id != original.session_id
        || admission.deletion_pending
        || admission.archived_at_ms.is_some()
        || admission.admitted_work_id.is_some()
        || admission.admitted_media_work_id.is_some()
        || !admission.admitted_media_paths.is_empty()
        || admission.provider_review_id.is_some()
        || admission.provider_review.is_some()
        || admission.provider_review_continuation_run_id.is_some()
        || !matches!(admission.provider_review_phase.as_deref(), None | Some("completed"))
        || read_kv(db, KV_PENDING_MEMORY_MUTATION)?.is_some()
        || read_kv(db, KV_PENDING_PROFILE_MUTATION)?.is_some()
    {
        return Err(AgentDbProjectionError::OriginalTurnStale(
            "session identity, lifecycle or independent custody refuses terminal evidence".into(),
        ));
    }
    if db.query_row(
        "SELECT EXISTS(SELECT 1 FROM cron_session_runs \
         WHERE lifecycle_revision = ?1 AND state = 'running')",
        params![admission.lifecycle_revision], |row| row.get::<_, bool>(0),
    )? {
        return Err(AgentDbProjectionError::OriginalTurnStale(
            "session still has running scheduled work".into(),
        ));
    }
    let raw_user = db.query_row(GET_TRANSCRIPT_ENTRY_SQL,
        params![original.user_message_id], |row| row.get::<_, String>(0),
    ).optional()?.ok_or_else(|| AgentDbProjectionError::OriginalTurnUnavailable(
        "original durable user row is missing".into(),
    ))?;
    let user: serde_json::Value = serde_json::from_str(&raw_user)?;
    let user_content = user.get("content").and_then(serde_json::Value::as_str)
        .filter(|content| !content.trim().is_empty());
    let optional_binding_matches = |key: &str, expected: Option<&str>| {
        match user.get(key) {
            None | Some(serde_json::Value::Null) => expected.is_none(),
            Some(value) => value.as_str().is_some_and(|value| Some(value) == expected),
        }
    };
    if user.get("id").and_then(serde_json::Value::as_str) != Some(original.user_message_id.as_str())
        || user.get("kind").and_then(serde_json::Value::as_str) != Some("message")
        || user.get("role").and_then(serde_json::Value::as_str) != Some("user")
        || user.get("confirmed").and_then(serde_json::Value::as_bool) != Some(true)
        || user_content.is_none()
        || ["branched", "hidden", "isHidden", "isStreaming"].into_iter().any(|key| {
            user.get(key).is_some_and(|value| !matches!(value, serde_json::Value::Bool(false)))
        })
        || ["fromAgent", "toAgent", "channel", "sender"].into_iter().any(|key| {
            user.get(key).is_some_and(|value| !value.is_null())
        })
        || !optional_binding_matches("clientNonce", original.client_nonce.as_deref())
        || user.get("sessionKey").is_some_and(|value| {
            value.as_str() != Some(original.session_key.as_str())
        })
        || user.get("sessionId").is_some_and(|value| {
            value.as_str() != Some(original.session_id.as_str())
        })
    {
        return Err(AgentDbProjectionError::OriginalTurnStale(
            "original user row is not the confirmed local admitted input".into(),
        ));
    }
    let compaction_id = format!("compaction:{}:{}", original.agent_id, original.stream_id);
    let raw_compaction = db.query_row(GET_TRANSCRIPT_ENTRY_SQL,
        params![compaction_id], |row| row.get::<_, String>(0),
    ).optional()?;
    let compaction = if admission.lifecycle_revision == original.original_lifecycle_revision {
        None
    } else {
        let entry: serde_json::Value = serde_json::from_str(raw_compaction.as_deref()
            .ok_or_else(|| AgentDbProjectionError::OriginalTurnStale(
                "admission changed without the original stream's compaction".into(),
            ))?)?;
        compaction_first_kept_entry_id(&entry)?;
        if entry.get("id").and_then(serde_json::Value::as_str) != Some(compaction_id.as_str())
            || entry.get("kind").and_then(serde_json::Value::as_str) != Some("compaction")
            || entry.get("sourceTurnId").and_then(serde_json::Value::as_str)
                != Some(original.stream_id.as_str())
            || entry.get("sourceLifecycleRevision").and_then(serde_json::Value::as_str)
                != Some(original.original_lifecycle_revision.as_str())
            || entry.get("lifecycleRevision").and_then(serde_json::Value::as_str)
                != Some(admission.lifecycle_revision.as_str())
        {
            return Err(AgentDbProjectionError::OriginalTurnStale(
                "compaction does not prove the exact original-to-settlement generation".into(),
            ));
        }
        Some(SameTurnCompactionBinding {
            entry_id: compaction_id,
            source_turn_id: original.stream_id.clone(),
            source_lifecycle_revision: original.original_lifecycle_revision.clone(),
            lifecycle_revision: admission.lifecycle_revision.clone(),
        })
    };
    // Metadata's lifecycleRevision is intentionally not compared with the
    // independently owned admission revision: ordinary checkpoints rotate it.
    Ok(OriginalTurnTerminalFence {
        original: original.clone(),
        final_root_blob_id: root,
        original_user_content: user_content.expect("checked user content").to_string(),
        settlement_lifecycle_revision: admission.lifecycle_revision,
        compaction,
        raw_metadata,
        raw_admission,
        raw_user,
        raw_compaction,
        raw_slot: slot.map(|slot| slot.raw.clone()),
    })
}

/// A committed value and its existing transcript notification. Session carries
/// this out of its filesystem/deletion lock before publishing to subscribers.
pub(crate) struct CommittedTranscriptAppend<T> {
    value: T,
    entries: Vec<serde_json::Value>,
    db_path: PathBuf,
}

impl<T> CommittedTranscriptAppend<T> {
    pub(crate) fn publish(self) -> T {
        if !self.entries.is_empty() {
            publish_entries_upserted(&self.db_path, self.entries);
        }
        self.value
    }
}

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
    pub expected_transcript_tail_id: String,
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
    pub deletion_pending: bool,
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
    pub provider_review_continuation_run_id: Option<String>,
    #[serde(default)]
    pub provider_review: Option<SessionProviderReview>,
}

/// Immutable preparation for one live-owner channel input claim. A captured
/// binding remains authoritative even when reading/preparing its entries fails.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelInboundAppendAttempt {
    pub session_key: String,
    pub session_id: String,
    pub lifecycle_revision: String,
    pub entries: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelInboundAppendFailureKind {
    Admission,
    Closed,
    Storage,
    InvalidBatch,
    IdentityConflict,
    RollbackUnconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelInboundAppendFailure {
    pub kind: ChannelInboundAppendFailureKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelInboundAppendOutcome {
    Committed { inserted_count: usize },
    NotWritten(ChannelInboundAppendFailure),
    Unknown(ChannelInboundAppendFailure),
}

/// Durable identity/state projection for one scheduled or event-triggered
/// automation run. Cleanup and transcript archival are deliberately not
/// implemented by this slice; the row is the owner-local handoff required by
/// the later cron reaper.
#[derive(Debug, Clone, PartialEq)]
pub struct CronSessionRunRecord {
    pub session_key: String,
    pub automation_id: String,
    pub run_id: String,
    pub session_id: String,
    pub lifecycle_revision: String,
    pub updated_at_ms: f64,
    pub base_session: bool,
    pub state: String,
    pub terminal_outcome: Option<String>,
    pub terminal_detail: Option<String>,
    pub active_controller: bool,
    pub pending_outgoing_media: bool,
    pub unsettled_subagent: bool,
    pub fence_state_known: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingMemoryMutation {
    pub mutation_id: String,
    pub operation: String,
    pub session_key: String,
    pub session_id: String,
    pub lifecycle_revision: String,
    pub records: Vec<MemoryRecoveryRecord>,
    pub prior_snapshot: Option<MemoryPromptSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileFileSnapshot {
    pub filename: String,
    pub contents: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingProfileMutation {
    pub mutation_id: String,
    pub operation: String,
    pub session_key: String,
    pub session_id: String,
    pub lifecycle_revision: String,
    pub before_files: Vec<ProfileFileSnapshot>,
    pub after_files: Vec<ProfileFileSnapshot>,
    pub previous_profile: SandProfile,
    pub next_profile: Option<SandProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderReviewContinuationOutcome {
    Completed,
    Cancelled,
    Failed,
    OutcomeUnknown,
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

/// A committed result and its existing synchronous notifications. The Session
/// facade carries this out of its filesystem/deletion boundary before publishing.
/// Only the result transaction constructs it; it is neither durable state nor
/// an independently writable notification queue.
#[must_use = "publish the result after releasing the caller's owner locks"]
pub(crate) struct SessionWorkResultPublication {
    outcome: SessionWorkResultCommitOutcome,
    db_path: PathBuf,
    committed_entry: Option<serde_json::Value>,
}

impl SessionWorkResultPublication {
    pub(crate) fn publish(self) -> SessionWorkResultCommitOutcome {
        if matches!(&self.outcome, SessionWorkResultCommitOutcome::Committed(_)) {
            if let Some(entry) = self.committed_entry {
                publish_entries_upserted(&self.db_path, vec![entry]);
            }
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
            );
        }
        self.outcome
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompactionCommitOutcome {
    Committed(serde_json::Value),
    AlreadyCommitted(serde_json::Value),
    RejectedStale,
    RejectedCancelled,
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

    /// Open an already materialized Agent without inventing missing metadata.
    /// Canonical checkpoint consumers validate the persisted root explicitly.
    pub fn open_existing_agent(
        db_path: impl AsRef<Path>,
        busy_timeout_ms: u64,
    ) -> Result<Self, AgentDbProjectionError> {
        let mut options = SandAgentDbOptions::default();
        options.recovery.busy_timeout_ms = busy_timeout_ms;
        Self::open_with_options_and_seed(db_path, options, false)
    }

    /// A terminal inspection must preserve unavailable source state. Ordinary
    /// foreground opens keep their recovery policy; this path neither rebuilds
    /// corruption nor invents absent metadata.
    pub(super) fn open_existing_agent_without_recovery(
        db_path: impl AsRef<Path>,
        busy_timeout_ms: u64,
    ) -> Result<Self, AgentDbProjectionError> {
        let mut options = SandAgentDbOptions::default();
        options.recovery.busy_timeout_ms = busy_timeout_ms;
        let recover_on_corruption = options.recovery.recover_on_corruption;
        let reject_if_corrupt = options.recovery.reject_if_corrupt;
        options.recovery.recover_on_corruption = false;
        options.recovery.reject_if_corrupt = true;
        let mut owner = Self::open_with_options_and_seed(db_path, options, false)?;
        // This instance enters the existing shared owner map. Terminal SQL
        // never calls recovery helpers, while later ordinary foreground APIs
        // retain their original recovery policy on this same cached owner.
        owner.options.recovery.recover_on_corruption = recover_on_corruption;
        owner.options.recovery.reject_if_corrupt = reject_if_corrupt;
        Ok(owner)
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

    /// All terminal operations are strict, including candidate absence. Never
    /// recover/retry a transaction or collapse a closed/busy owner into None.
    /// The operation is internal SQL only; it cannot call Runner or listeners.
    fn original_turn_terminal_transaction<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Connection)
            -> Result<(T, bool), AgentDbProjectionError>,
    ) -> Result<T, AgentDbProjectionError> {
        let mut guard = self.connection.lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        if self.is_closed() {
            return Err(AgentDbProjectionError::OriginalTurnUnavailable(
                "terminal database owner is closed".into(),
            ));
        }
        let db = guard.as_mut().ok_or_else(|| AgentDbProjectionError::OriginalTurnUnavailable(
            "terminal database connection is unavailable".into(),
        ))?;
        if !db.is_autocommit() {
            self.retire_channel_inbound_connection(&mut guard);
            return Err(AgentDbProjectionError::OriginalTurnUnavailable(
                "terminal database has an unsettled prior transaction".into(),
            ));
        }
        let recovery = acquire_automation_custody_recovery_guard(&self.db_path, db)?;
        let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let result = (|| {
            recovery.require_available(&transaction)?;
            let result = operation(&transaction)?;
            recovery.require_available(&transaction)?;
            Ok::<_, AgentDbProjectionError>(result)
        })();
        let (value, changed) = match result {
            Ok(result) => result,
            Err(error) => {
                let rollback = transaction.rollback();
                let clean = db.is_autocommit();
                if rollback.is_err() || !clean {
                    if !clean {
                        self.retire_channel_inbound_connection(&mut guard);
                    }
                    return Err(AgentDbProjectionError::OriginalTurnUnavailable(format!(
                        "terminal rollback was not confirmed after {error}",
                    )));
                }
                return Err(error);
            }
        };
        if let Err(error) = transaction.commit() {
            if !db.is_autocommit() {
                let _ = db.execute_batch("ROLLBACK");
            }
            if !db.is_autocommit() {
                self.retire_channel_inbound_connection(&mut guard);
            }
            return Err(AgentDbProjectionError::OriginalTurnUnavailable(format!(
                "terminal COMMIT outcome is unknown: {error}",
            )));
        }
        if changed {
            bump_db_write_generation(&self.db_path);
        }
        // The slot is internal evidence, so there is no Transcript/UI callback.
        drop(recovery);
        drop(guard);
        Ok(value)
    }

    pub(super) fn prepare_original_turn_terminal_fence(
        &self,
        original: &OriginalTurnReference,
        expected_final_root: &[u8],
    ) -> Result<OriginalTurnTerminalFence, AgentDbProjectionError> {
        self.original_turn_terminal_transaction(|db| {
            let slot = read_original_turn_terminal_slot(db)?;
            let fence = capture_original_turn_terminal_fence(
                db, original, expected_final_root, slot.as_ref(),
            )?;
            Ok((fence, false))
        })
    }

    pub(super) fn read_original_turn_terminal_candidate_checked(
        &self,
        original: &OriginalTurnReference,
    ) -> Result<Option<(OriginalTurnTerminalRecord, OriginalTurnTerminalFence)>, AgentDbProjectionError> {
        original.validate().map_err(AgentDbProjectionError::OriginalTurnInvalid)?;
        self.original_turn_terminal_transaction(|db| {
            // The transaction's recovery guard has already refused a
            // reconstructed store before either absence branch is reachable.
            let Some(slot) = read_original_turn_terminal_slot(db)? else {
                return Ok((None, false));
            };
            if slot.record.original != *original {
                return Ok((None, false));
            }
            let final_root = &slot.record.canonical.last()
                .expect("strict terminal record has a canonical commit").root_blob_id;
            let fence = capture_original_turn_terminal_fence(db, original, final_root, Some(&slot))?;
            if slot.record.settlement_lifecycle_revision != fence.settlement_lifecycle_revision
                || slot.record.compaction != fence.compaction
            {
                return Err(AgentDbProjectionError::OriginalTurnStale(
                    "stored terminal settlement no longer matches Session".into(),
                ));
            }
            Ok((Some((slot.record, fence)), false))
        })
    }

    pub(super) fn recheck_original_turn_terminal_fence(
        &self,
        expected: &OriginalTurnTerminalFence,
    ) -> Result<(), AgentDbProjectionError> {
        self.original_turn_terminal_transaction(|db| {
            let slot = read_original_turn_terminal_slot(db)?;
            let current = capture_original_turn_terminal_fence(
                db, &expected.original, &expected.final_root_blob_id, slot.as_ref(),
            )?;
            if current != *expected {
                return Err(AgentDbProjectionError::OriginalTurnStale(
                    "inspected terminal owner rows changed".into(),
                ));
            }
            Ok(((), false))
        })
    }

    pub(super) fn commit_original_turn_terminal_record(
        &self,
        expected: &OriginalTurnTerminalFence,
        record: &OriginalTurnTerminalRecord,
    ) -> Result<OriginalTurnReceiptCommit, AgentDbProjectionError> {
        validate_original_turn_terminal_record(record)?;
        let encoded = serde_json::to_string(record)?;
        if encoded.len() > ORIGINAL_TURN_TERMINAL_MAX_BYTES {
            return Err(AgentDbProjectionError::OriginalTurnInvalid(
                "terminal record exceeds its encoded byte limit".into(),
            ));
        }
        if record.original != expected.original
            || record.canonical.last().map(|commit| &commit.root_blob_id)
                != Some(&expected.final_root_blob_id)
            || record.settlement_lifecycle_revision != expected.settlement_lifecycle_revision
            || record.compaction != expected.compaction
        {
            return Err(AgentDbProjectionError::OriginalTurnInvalid(
                "terminal record differs from its prepared Session binding".into(),
            ));
        }
        self.original_turn_terminal_transaction(|db| {
            let slot = read_original_turn_terminal_slot(db)?;
            if slot.as_ref().is_some_and(|slot| {
                slot.record.original.stream_id == record.original.stream_id && slot.raw != encoded
            }) {
                // Keep a same-stream conflict explicit even when another
                // captured row has also become stale.
                return Err(AgentDbProjectionError::OriginalTurnConflict(
                    "the current slot already binds this stream to different terminal data".into(),
                ));
            }
            let current = capture_original_turn_terminal_fence(
                db, &expected.original, &expected.final_root_blob_id, slot.as_ref(),
            )?;
            if current != *expected {
                return Err(AgentDbProjectionError::OriginalTurnStale(
                    "terminal owner rows changed after canonical qualification".into(),
                ));
            }
            if slot.as_ref().is_some_and(|slot| slot.raw == encoded) {
                return Ok((OriginalTurnReceiptCommit::AlreadyCommitted, false));
            }
            let outcome = if let Some(slot) = slot.as_ref() {
                if slot.record.canonical.last().map(|commit| commit.root_blob_id.as_slice())
                    == Some(expected.final_root_blob_id.as_slice())
                {
                    return Err(AgentDbProjectionError::OriginalTurnStale(
                        "a different stream cannot replace the slot at the same canonical root".into(),
                    ));
                }
                if db.execute(COMPARE_AND_SET_KV_SQL,
                    params![encoded, ORIGINAL_TURN_TERMINAL_KEY, slot.raw],
                )? != 1 {
                    return Err(AgentDbProjectionError::OriginalTurnStale(
                        "terminal slot replacement lost its raw compare".into(),
                    ));
                }
                OriginalTurnReceiptCommit::Replaced
            } else {
                if db.execute("INSERT INTO kv (key, value) VALUES (?1, ?2)",
                    params![ORIGINAL_TURN_TERMINAL_KEY, encoded],
                )? != 1 {
                    return Err(AgentDbProjectionError::OriginalTurnStale(
                        "terminal slot insertion did not commit one row".into(),
                    ));
                }
                OriginalTurnReceiptCommit::Inserted
            };
            // A SQL trigger can change another row in this same transaction.
            // Re-read every captured image after the slot write and roll all
            // effects back if the final owner image is no longer exact.
            let written_slot = read_original_turn_terminal_slot(db)?;
            let after = capture_original_turn_terminal_fence(
                db, &expected.original, &expected.final_root_blob_id, written_slot.as_ref(),
            )?;
            let mut committed = expected.clone();
            committed.raw_slot = Some(encoded.clone());
            if after != committed {
                return Err(AgentDbProjectionError::OriginalTurnStale(
                    "terminal owner rows changed during slot publication".into(),
                ));
            }
            Ok((outcome, true))
        })
    }

    pub fn write_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        reject_original_turn_terminal_key(key)?;
        self.run_write(&format!("writeKv:{key}"), |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(SET_KV_SQL, params![key, value])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    /// Write one session-scoped projection only when the caller still owns
    /// the exact admitted session generation. The admission check and KV
    /// write share one SQLite transaction so an older Gateway request cannot
    /// publish policy into a newer lifecycle generation.
    pub fn write_session_kv_if_generation(
        &self,
        key: &str,
        value: &str,
        expected_session_id: &str,
        expected_lifecycle_revision: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        reject_original_turn_terminal_key(key)?;
        let expected_session_id = expected_session_id.trim();
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        if key.trim().is_empty()
            || expected_session_id.is_empty()
            || expected_lifecycle_revision.is_empty()
        {
            return Ok(false);
        }
        let changed = self.run_write_value("writeSessionKvIfGeneration", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(bool, bool), rusqlite::Error> {
                let Some(raw_admission) = read_kv(db, KV_SESSION_WORK_ADMISSION)? else {
                    return Ok((false, false));
                };
                let admission = serde_json::from_str::<SessionWorkAdmissionRecord>(&raw_admission)
                    .map_err(|error| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(error))
                    })?;
                if admission.session_id != expected_session_id
                    || admission.lifecycle_revision != expected_lifecycle_revision
                    || !session_work_write_is_admitted(db)?
                {
                    return Ok((false, false));
                }
                let changed = db.execute(SET_KV_SQL, params![key, value])? > 0;
                Ok((changed, changed))
            })();
            match result {
                Ok(result) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(result)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(key.to_string()),
            );
        }
        Ok(changed)
    }

    pub fn delete_kv(&self, key: &str) -> Result<bool, AgentDbProjectionError> {
        reject_original_turn_terminal_key(key)?;
        self.run_write(&format!("deleteKv:{key}"), |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(DELETE_KV_SQL, params![key])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
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

    /// Re-seed the owner-local admission record after a cloned database has
    /// received its new directory identity. This is intentionally the one
    /// pre-admission reset that may replace the copied source record.
    pub fn reset_session_work_admission_for_clone(
        &self,
        agent_id: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "cloned session admission requires an agent identity".into(),
            ));
        }
        let record = SessionWorkAdmissionRecord {
            session_key: format!("agent:{agent_id}:main"),
            session_id: agent_id.to_string(),
            lifecycle_revision: Uuid::new_v4().to_string(),
            deletion_pending: false,
            archived_at_ms: None,
            archived_by: None,
            archive_reason: None,
            admitted_work_id: None,
            admitted_media_work_id: None,
            admitted_media_paths: Vec::new(),
            terminal_result_idempotency_key: None,
            provider_review_id: None,
            provider_review_phase: None,
            provider_review_continuation_run_id: None,
            provider_review: None,
        };
        let encoded = serde_json::to_string(&record)?;
        let changed = self.run_write("resetSessionWorkAdmissionForClone", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<usize, rusqlite::Error> {
                let changes = db.execute(
                    SET_KV_SQL,
                    params![KV_SESSION_WORK_ADMISSION, encoded],
                )?;
                clear_original_turn_terminal_slots(db)?;
                Ok(changes)
            })();
            match result {
                Ok(changes) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changes > 0)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
            );
        }
        Ok(changed)
    }

    /// Compare-and-set the agent-local work-admission record. A missing
    /// expected revision means create-only; a supplied revision means update
    /// only when the currently stored generation still matches.
    pub fn compare_and_set_session_work_admission(
        &self,
        expected_lifecycle_revision: Option<&str>,
        next: &SessionWorkAdmissionRecord,
    ) -> Result<bool, AgentDbProjectionError> {
        self.compare_and_set_session_work_admission_with_deletion_transition(
            expected_lifecycle_revision,
            next,
            Some(next.deletion_pending),
        )
    }

    /// Clear deletion custody only when the durable record still carries the
    /// deletion fence observed by the deletion owner. Ordinary admission CAS
    /// operations preserve the current fence and cannot clear it implicitly.
    pub fn clear_session_deletion_pending(
        &self,
        expected_lifecycle_revision: &str,
        next: &SessionWorkAdmissionRecord,
    ) -> Result<bool, AgentDbProjectionError> {
        self.transition_session_deletion_pending(
            Some(expected_lifecycle_revision),
            true,
            next,
        )
    }

    /// Transition deletion custody only when the durable record still carries
    /// the expected prior fence value. This is the sole owner operation that
    /// may change `deletion_pending` across a CAS.
    pub fn transition_session_deletion_pending(
        &self,
        expected_lifecycle_revision: Option<&str>,
        expected_current_deletion_pending: bool,
        next: &SessionWorkAdmissionRecord,
    ) -> Result<bool, AgentDbProjectionError> {
        if next.deletion_pending == expected_current_deletion_pending {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "deletion custody transition must change deletion_pending".into(),
            ));
        }
        self.compare_and_set_session_work_admission_with_deletion_transition(
            expected_lifecycle_revision,
            next,
            Some(expected_current_deletion_pending),
        )
    }

    fn compare_and_set_session_work_admission_with_deletion_transition(
        &self,
        expected_lifecycle_revision: Option<&str>,
        next: &SessionWorkAdmissionRecord,
        expected_current_deletion_pending: Option<bool>,
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
        db.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<bool, AgentDbProjectionError> {
            let current = db
                .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let matches = match (expected_lifecycle_revision, current.as_deref()) {
                (None, None) => true,
                (Some(expected), Some(raw)) => {
                    serde_json::from_str::<SessionWorkAdmissionRecord>(raw)
                        .ok()
                        .is_some_and(|record| {
                            record.lifecycle_revision == expected
                                && record.session_key == next.session_key
                                && record.session_id == next.session_id
                                && expected_current_deletion_pending
                                    .map_or(true, |expected| {
                                        record.deletion_pending == expected
                                    })
                        })
                }
                _ => false,
            };
            if !matches {
                return Ok(false);
            }
            if let Some(raw_pending) = db
                .query_row(GET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
            {
                let current_record = current.as_deref().and_then(|raw| {
                    serde_json::from_str::<SessionWorkAdmissionRecord>(raw).ok()
                });
                if current_record
                    .as_ref()
                    .is_some_and(|record| record.lifecycle_revision != next.lifecycle_revision)
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "pending memory mutation blocks lifecycle revision transition".into(),
                    ));
                }
                if serde_json::from_str::<PendingMemoryMutation>(&raw_pending)
                    .ok()
                    .is_none_or(|mutation| {
                        mutation.mutation_id.trim().is_empty()
                            || mutation.operation != "remove" && mutation.operation != "clear"
                            || mutation.session_key != next.session_key
                            || mutation.session_id != next.session_id
                            || mutation.lifecycle_revision != next.lifecycle_revision
                    })
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "pending memory mutation journal is malformed or rebound".into(),
                    ));
                }
            }
            if let Some(raw_pending) = db
                .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
            {
                let current_record = current.as_deref().and_then(|raw| {
                    serde_json::from_str::<SessionWorkAdmissionRecord>(raw).ok()
                });
                if current_record
                    .as_ref()
                    .is_some_and(|record| record.lifecycle_revision != next.lifecycle_revision)
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "pending profile mutation blocks lifecycle revision transition".into(),
                    ));
                }
                if serde_json::from_str::<PendingProfileMutation>(&raw_pending)
                    .ok()
                    .is_none_or(|mutation| {
                        mutation.mutation_id.trim().is_empty()
                            || !matches!(
                                mutation.operation.as_str(),
                                "profile-update" | "avatar-replace" | "avatar-remove"
                            )
                            || mutation.session_key != next.session_key
                            || mutation.session_id != next.session_id
                            || mutation.lifecycle_revision != next.lifecycle_revision
                    })
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "pending profile mutation journal is malformed or rebound".into(),
                    ));
                }
            }
            let changed = match current {
                Some(previous) => db.execute(
                    COMPARE_AND_SET_KV_SQL,
                    params![encoded, KV_SESSION_WORK_ADMISSION, previous],
                )?,
                None => db.execute(SET_KV_SQL, params![KV_SESSION_WORK_ADMISSION, encoded])?,
            };
            Ok(changed == 1)
        })();
        let changed = match result {
            Ok(changed) => {
                match db.execute_batch("COMMIT") {
                    Ok(()) => changed,
                    Err(error) => {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error.into());
                    }
                }
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                return Err(error);
            }
        };
        drop(guard);
        if changed {
            notify_agent_db_listeners(
                &self.db_path,
                AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
            );
        }
        Ok(changed)
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
            || record.deletion_pending
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
            // replaces the consumed review under the same session generation,
            // and the terminal refusal releases that continuation's work
            // marker in the same owner CAS. Pending reviews remain strict
            // idempotent/conflict-safe CAS.
            if record.provider_review_continuation_run_id.as_deref()
                != Some(review.run_id.as_str())
                || record.admitted_work_id.as_deref() != Some(review.run_id.as_str())
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "provider review continuation work identity changed".into(),
                ));
            }
            record.admitted_work_id = None;
        }
        record.provider_review_id = Some(review.id.clone());
        record.provider_review_phase = Some("pending".into());
        record.provider_review_continuation_run_id = None;
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

    /// Settle one admitted provider-review continuation against the exact
    /// lifecycle generation and continuation run that claimed it. A completed
    /// continuation clears the review; known non-success and unknown outcomes
    /// retain the facts in a non-actionable terminal phase.
    pub fn settle_provider_review_continuation(
        &self,
        expected_lifecycle_revision: &str,
        continuation_run_id: &str,
        outcome: ProviderReviewContinuationOutcome,
    ) -> Result<bool, AgentDbProjectionError> {
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        let continuation_run_id = continuation_run_id.trim();
        if expected_lifecycle_revision.is_empty() || continuation_run_id.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "provider review settlement requires lifecycle and continuation identity".into(),
            ));
        }
        let Some(mut record) = self.read_session_work_admission()? else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session work admission record is unavailable".into(),
            ));
        };
        if record.lifecycle_revision != expected_lifecycle_revision
            || record.provider_review_phase.as_deref() != Some("accepted")
            || record.provider_review_continuation_run_id.as_deref() != Some(continuation_run_id)
            || record.admitted_work_id.as_deref() != Some(continuation_run_id)
            || record.deletion_pending
        {
            return Ok(false);
        }
        if record.admitted_work_id.as_deref() == Some(continuation_run_id) {
            record.admitted_work_id = None;
        }
        match outcome {
            ProviderReviewContinuationOutcome::Completed => {
                record.provider_review_id = None;
                record.provider_review_phase = Some("completed".into());
                record.provider_review_continuation_run_id = None;
                record.provider_review = None;
            }
            ProviderReviewContinuationOutcome::Cancelled => {
                record.provider_review_phase = Some("cancelled".into());
                record.provider_review_continuation_run_id = None;
            }
            ProviderReviewContinuationOutcome::Failed => {
                record.provider_review_phase = Some("failed".into());
                record.provider_review_continuation_run_id = None;
            }
            ProviderReviewContinuationOutcome::OutcomeUnknown => {
                record.provider_review_phase = Some("outcome-unknown".into());
                record.provider_review_continuation_run_id = None;
            }
        }
        self.compare_and_set_session_work_admission(
            Some(expected_lifecycle_revision),
            &record,
        )
    }

    /// Roll back a claimed review when the routed Runner never accepted the
    /// dispatch. This is distinct from a Runner terminal failure, which has a
    /// durable in-flight owner and therefore remains non-actionable.
    pub fn release_provider_review_continuation(
        &self,
        expected_lifecycle_revision: &str,
        continuation_run_id: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        let continuation_run_id = continuation_run_id.trim();
        if expected_lifecycle_revision.is_empty() || continuation_run_id.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "provider review release requires lifecycle and continuation identity".into(),
            ));
        }
        let Some(mut record) = self.read_session_work_admission()? else {
            return Ok(false);
        };
        if record.lifecycle_revision != expected_lifecycle_revision
            || record.provider_review_phase.as_deref() != Some("accepted")
            || record.provider_review_continuation_run_id.as_deref() != Some(continuation_run_id)
            || record.admitted_work_id.as_deref() != Some(continuation_run_id)
            || record.deletion_pending
        {
            return Ok(false);
        }
        record.provider_review_phase = Some("pending".into());
        record.provider_review_continuation_run_id = None;
        self.compare_and_set_session_work_admission(
            Some(expected_lifecycle_revision),
            &record,
        )
    }

    /// Atomically transition archive state for one canonical session
    /// generation. Active admitted work is preserved and blocks archiving;
    /// callers must settle it before retrying.
    pub fn transition_session_work_archive(
        &self,
        expected_session_key: &str,
        expected_session_id: &str,
        expected_lifecycle_revision: Option<&str>,
        archived: bool,
        archived_by: Option<&str>,
        archive_reason: Option<&str>,
        archived_at_ms: f64,
    ) -> Result<SessionWorkAdmissionRecord, AgentDbProjectionError> {
        let expected_session_key = expected_session_key.trim();
        let expected_session_id = expected_session_id.trim();
        if expected_session_key.is_empty()
            || expected_session_id.is_empty()
            || !archived_at_ms.is_finite()
        {
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
        db.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<
            (SessionWorkAdmissionRecord, bool),
            AgentDbProjectionError,
        > {
            let raw = db
                .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
                .ok_or_else(|| AgentDbProjectionError::WorkAdmissionInvalid(
                    "session work admission record is unavailable".into(),
                ))?;
            let mut record: SessionWorkAdmissionRecord = serde_json::from_str(&raw)?;
            if !session_admission_identity_is_bound(db, &record)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session archive transition owner identity is malformed or rebound".into(),
                ));
            }
            if db
                .query_row(GET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
                .is_some()
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "pending memory mutation blocks archive/restore transition".into(),
                ));
            }
            if db
                .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
                .is_some()
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "pending profile mutation blocks archive/restore transition".into(),
                ));
            }
            if record.session_key != expected_session_key
                || record.session_id != expected_session_id
            {
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
                return Ok((record, false));
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
            record.lifecycle_revision = Uuid::new_v4().to_string();
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
            Ok((record, true))
        })();
        match result {
            Ok((record, changed)) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                drop(guard);
                if changed {
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
                    );
                }
                Ok(record)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
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
        self.commit_session_work_result_deferred(
            session_key,
            expected_lifecycle_revision,
            work_id,
            idempotency_key,
            content,
            sent_message_count,
        )
        .map(SessionWorkResultPublication::publish)
    }

    pub(crate) fn commit_session_work_result_deferred(
        &self,
        session_key: &str,
        expected_lifecycle_revision: &str,
        work_id: &str,
        idempotency_key: &str,
        content: &str,
        sent_message_count: usize,
    ) -> Result<SessionWorkResultPublication, AgentDbProjectionError> {
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
            if !session_work_write_is_admitted(db)? {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            }
            let raw_admission = db
                .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(raw_admission) = raw_admission else {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            };
            let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key
                || admission.lifecycle_revision != expected_lifecycle_revision
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Ok(SessionWorkResultCommitOutcome::RejectedStale);
            }
            let receipt_id = format!("background-result:{idempotency_key}");
            if let Some(raw) = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![receipt_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
            {
                let Some(entry) = parse_owner_transcript_entry(&raw) else {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "background result receipt contains malformed JSON".into(),
                    ));
                };
                let receipt = parse_session_work_result_receipt(&entry).ok_or_else(|| {
                    AgentDbProjectionError::WorkAdmissionInvalid(
                        "background result receipt is missing required identity".into(),
                    )
                })?;
                if entry.get("id").and_then(serde_json::Value::as_str) == Some(receipt_id.as_str())
                    && receipt.session_key == session_key
                    && receipt.lifecycle_revision == expected_lifecycle_revision
                    && receipt.work_id == work_id
                    && receipt.idempotency_key == idempotency_key
                    && receipt.content == content
                    && receipt.sent_message_count == sent_message_count
                {
                    return Ok(SessionWorkResultCommitOutcome::AlreadyCommitted(receipt));
                }
                return Ok(SessionWorkResultCommitOutcome::ConflictingResult);
            }

            let mut admission = admission;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key
                || admission.lifecycle_revision != expected_lifecycle_revision
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
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

        let (outcome, committed_entry) = match result {
            // A stale CAS after the receipt insert must roll back the whole
            // transaction. Committing here would leave an orphan receipt
            // while the admission marker still owns the work, causing a
            // later retry to be misreported as AlreadyCommitted.
            Ok(SessionWorkResultCommitOutcome::RejectedStale) => {
                let _ = db.execute_batch("ROLLBACK");
                Ok((SessionWorkResultCommitOutcome::RejectedStale, None))
            }
            Ok(outcome) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                let committed_entry =
                    if let SessionWorkResultCommitOutcome::Committed(receipt) = &outcome {
                        let entry_id = format!("background-result:{}", receipt.idempotency_key);
                        let entry = db
                            .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                                row.get::<_, String>(0)
                            })?;
                        // Preserve the existing post-COMMIT read/error and hidden
                        // receipt projection; only notification timing changes.
                        parse_transcript_entry(&entry)
                    } else {
                        None
                    };
                Ok((outcome, committed_entry))
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }?;
        drop(guard);
        Ok(SessionWorkResultPublication {
            outcome,
            db_path: self.db_path.clone(),
            committed_entry,
        })
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
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let existing = db
                    .query_row(GET_KV_SQL, params!["metadata"], |row| row.get::<_, String>(0))
                    .optional()?;
                if existing.is_some() {
                    return Ok(false);
                }
                Ok(db.execute(SET_KV_SQL, params!["metadata", encoded])? == 1)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
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
        if key == "agentId"
            && value.as_str().map(str::trim) != Some(self.agent_dir_name.as_str())
        {
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
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let Some(current_raw) = read_kv(db, "metadata")? else {
                    return Ok(false);
                };
                if current_raw != raw {
                    return Ok(false);
                }
                Ok(db.execute(
                    COMPARE_AND_SET_KV_SQL,
                    params![next_raw, "metadata", raw],
                )? == 1)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
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

    /// Register one background automation run in the existing session owner.
    /// The identity and lifecycle revision are immutable for the row; a
    /// duplicate registration can only refresh an already-running row.
    pub fn begin_cron_session_run(
        &self,
        session_key: &str,
        automation_id: &str,
        run_id: &str,
        session_id: &str,
        lifecycle_revision: &str,
        started_at_ms: f64,
    ) -> Result<bool, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let automation_id = automation_id.trim();
        let run_id = run_id.trim();
        let session_id = session_id.trim();
        let lifecycle_revision = lifecycle_revision.trim();
        if session_key.is_empty()
            || automation_id.is_empty()
            || run_id.is_empty()
            || session_id.is_empty()
            || lifecycle_revision.is_empty()
            || !started_at_ms.is_finite()
        {
            return Ok(false);
        }
        self.run_write("beginCronSessionRun", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(
                    "INSERT INTO cron_session_runs (session_key, automation_id, run_id, session_id, lifecycle_revision, updated_at_ms, base_session, state) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 'running') \
                     ON CONFLICT(session_key) DO UPDATE SET updated_at_ms = excluded.updated_at_ms \
                     WHERE cron_session_runs.automation_id = excluded.automation_id \
                       AND cron_session_runs.run_id = excluded.run_id \
                       AND cron_session_runs.session_id = excluded.session_id \
                       AND cron_session_runs.lifecycle_revision = excluded.lifecycle_revision \
                       AND cron_session_runs.state = 'running'",
                    params![
                        session_key,
                        automation_id,
                        run_id,
                        session_id,
                        lifecycle_revision,
                        started_at_ms
                    ],
                )? > 0)
            })();
            match result {
                Ok(changed) => {
                    match db.execute_batch("COMMIT") {
                        Ok(()) => Ok(changed),
                        Err(error) => {
                            let _ = db.execute_batch("ROLLBACK");
                            Err(error)
                        }
                    }
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    /// Settle a registered background automation run only when its durable
    /// identity and lifecycle revision still match. Cleanup/archive remains a
    /// separate later reaper operation.
    pub fn finish_cron_session_run(
        &self,
        session_key: &str,
        session_id: &str,
        lifecycle_revision: &str,
        outcome: &str,
        finished_at_ms: f64,
        detail: Option<&str>,
    ) -> Result<bool, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let session_id = session_id.trim();
        let lifecycle_revision = lifecycle_revision.trim();
        let outcome = outcome.trim();
        if session_key.is_empty()
            || session_id.is_empty()
            || lifecycle_revision.is_empty()
            || outcome.is_empty()
            || !matches!(outcome, "ok" | "error" | "interrupted")
            || !finished_at_ms.is_finite()
        {
            return Ok(false);
        }
        self.run_write("finishCronSessionRun", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(
                    "UPDATE cron_session_runs \
                     SET updated_at_ms = ?1, state = 'terminal', terminal_outcome = ?2, terminal_detail = ?3 \
                     WHERE session_key = ?4 AND session_id = ?5 AND lifecycle_revision = ?6 AND state = 'running'",
                    params![
                        finished_at_ms,
                        outcome,
                        detail,
                        session_key,
                        session_id,
                        lifecycle_revision
                    ],
                )? > 0)
            })();
            match result {
                Ok(changed) => {
                    match db.execute_batch("COMMIT") {
                        Ok(()) => Ok(changed),
                        Err(error) => {
                            let _ = db.execute_batch("ROLLBACK");
                            Err(error)
                        }
                    }
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    /// Atomically publish a complete terminal fence snapshot with the row's
    /// terminal outcome. A future candidate projection must not observe the
    /// fence marker before the terminal state is durable.
    pub fn finish_cron_session_run_with_fences(
        &self,
        session_key: &str,
        session_id: &str,
        lifecycle_revision: &str,
        outcome: &str,
        finished_at_ms: f64,
        detail: Option<&str>,
        active_controller: bool,
        pending_outgoing_media: bool,
        unsettled_subagent: bool,
    ) -> Result<bool, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let session_id = session_id.trim();
        let lifecycle_revision = lifecycle_revision.trim();
        let outcome = outcome.trim();
        if session_key.is_empty()
            || session_id.is_empty()
            || lifecycle_revision.is_empty()
            || outcome.is_empty()
            || !matches!(outcome, "ok" | "error" | "interrupted")
            || !finished_at_ms.is_finite()
        {
            return Ok(false);
        }
        self.run_write("finishCronSessionRunWithFences", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(
                    "UPDATE cron_session_runs \
                     SET updated_at_ms = ?1, state = 'terminal', terminal_outcome = ?2, \
                         terminal_detail = ?3, active_controller = ?4, \
                         pending_outgoing_media = ?5, unsettled_subagent = ?6, \
                         fence_state_known = 1 \
                     WHERE session_key = ?7 AND session_id = ?8 \
                       AND lifecycle_revision = ?9 AND state = 'running'",
                    params![
                        finished_at_ms,
                        outcome,
                        detail,
                        active_controller,
                        pending_outgoing_media,
                        unsettled_subagent,
                        session_key,
                        session_id,
                        lifecycle_revision,
                    ],
                )? > 0)
            })();
            match result {
                Ok(changed) => {
                    match db.execute_batch("COMMIT") {
                        Ok(()) => Ok(changed),
                        Err(error) => {
                            let _ = db.execute_batch("ROLLBACK");
                            Err(error)
                        }
                    }
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    /// Publish the complete row-specific activity fence before terminal
    /// settlement. Unknown or stale identities never become known by this
    /// operation.
    pub fn publish_cron_session_run_fences(
        &self,
        session_key: &str,
        session_id: &str,
        lifecycle_revision: &str,
        active_controller: bool,
        pending_outgoing_media: bool,
        unsettled_subagent: bool,
    ) -> Result<bool, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let session_id = session_id.trim();
        let lifecycle_revision = lifecycle_revision.trim();
        if session_key.is_empty() || session_id.is_empty() || lifecycle_revision.is_empty() {
            return Ok(false);
        }
        self.run_write("publishCronSessionRunFences", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(
                    "UPDATE cron_session_runs \
                     SET active_controller = ?1, pending_outgoing_media = ?2, \
                         unsettled_subagent = ?3, fence_state_known = 1 \
                     WHERE session_key = ?4 AND session_id = ?5 \
                       AND lifecycle_revision = ?6 AND state = 'running'",
                    params![
                        active_controller,
                        pending_outgoing_media,
                        unsettled_subagent,
                        session_key,
                        session_id,
                        lifecycle_revision
                    ],
                )? > 0)
            })();
            match result {
                Ok(changed) => {
                    match db.execute_batch("COMMIT") {
                        Ok(()) => Ok(changed),
                        Err(error) => {
                            let _ = db.execute_batch("ROLLBACK");
                            Err(error)
                        }
                    }
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    /// Read the per-run projections owned by this agent database. This is
    /// intentionally a projection only; retention policy and deletion remain
    /// outside this method and must use a later guarded lifecycle operation.
    pub fn list_cron_session_runs(
        &self,
    ) -> Result<Vec<CronSessionRunRecord>, AgentDbProjectionError> {
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
        let mut statement = db.prepare(
            "SELECT session_key, automation_id, run_id, session_id, lifecycle_revision, \
                    updated_at_ms, base_session, state, terminal_outcome, terminal_detail, \
                    active_controller, pending_outgoing_media, unsettled_subagent, fence_state_known \
             FROM cron_session_runs ORDER BY updated_at_ms ASC, session_key ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(CronSessionRunRecord {
                session_key: row.get(0)?,
                automation_id: row.get(1)?,
                run_id: row.get(2)?,
                session_id: row.get(3)?,
                lifecycle_revision: row.get(4)?,
                updated_at_ms: row.get(5)?,
                base_session: row.get::<_, i64>(6)? != 0,
                state: row.get(7)?,
                terminal_outcome: row.get(8)?,
                terminal_detail: row.get(9)?,
                active_controller: row.get::<_, i64>(10)? != 0,
                pending_outgoing_media: row.get::<_, i64>(11)? != 0,
                unsettled_subagent: row.get::<_, i64>(12)? != 0,
                fence_state_known: row.get::<_, i64>(13)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
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
            if !session_work_write_is_admitted(db).map_err(AgentDbProjectionError::Sqlite)? {
                return Ok(None);
            }
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
            let Some(current_root) = metadata
                .get("latestRootBlobId")
                .and_then(serde_json::Value::as_str)
            else {
                return Ok(None);
            };
            let current_root_bytes = decode_hex(current_root)
                .map_err(AgentDbProjectionError::LatestRootHex)?;
            if (current_root_bytes.is_empty() && !current_root.is_empty())
                || current_root_bytes.as_slice() != expected_root
            {
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
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
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
        let Some(current) = metadata
            .get("latestRootBlobId")
            .and_then(serde_json::Value::as_str)
        else {
            return Ok(false);
        };
        let current_bytes = decode_hex(current)
            .map_err(AgentDbProjectionError::LatestRootHex)?;
        if (current_bytes.is_empty() && !current.is_empty())
            || current_bytes.as_slice() != expected_root
        {
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
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let Some(current_raw) = read_kv(db, "metadata")? else {
                    return Ok(false);
                };
                if current_raw != raw {
                    return Ok(false);
                }
                Ok(db.execute(
                    COMPARE_AND_SET_KV_SQL,
                    params![next_raw, "metadata", raw],
                )? == 1)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
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

    /// Canonical readiness distinguishes a persisted empty root from absent,
    /// malformed or unavailable state. This read never seeds or repairs it.
    pub fn read_checkpoint_root_checked(
        &self,
    ) -> Result<Option<Vec<u8>>, AgentDbProjectionError> {
        self.read_checkpoint_root_with_identity_checked(None)
    }

    pub(crate) fn read_checkpoint_root_for_agent_checked(
        &self,
        agent_id: &str,
    ) -> Result<Option<Vec<u8>>, AgentDbProjectionError> {
        self.read_checkpoint_root_with_identity_checked(Some(agent_id))
    }

    fn read_checkpoint_root_with_identity_checked(
        &self,
        expected_agent_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, AgentDbProjectionError> {
        if self.is_closed() {
            return Err(AgentDbProjectionError::CheckpointRootInvalid(
                "database owner is closed".into(),
            ));
        }
        let guard = self.connection.lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let db = guard.as_ref().ok_or_else(|| {
            AgentDbProjectionError::CheckpointRootInvalid(
                "database connection is unavailable".into(),
            )
        })?;
        let raw = read_kv(db, "metadata")?.ok_or_else(|| {
            AgentDbProjectionError::CheckpointRootInvalid(
                "persisted metadata is missing".into(),
            )
        })?;
        let bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
        let metadata: serde_json::Value = serde_json::from_slice(&bytes)?;
        if let Some(expected_agent_id) = expected_agent_id {
            if metadata.get("agentId").and_then(serde_json::Value::as_str).map(str::trim)
                != Some(expected_agent_id)
            {
                return Err(AgentDbProjectionError::CheckpointRootInvalid(
                    "persisted metadata agent identity is missing or rebound".into(),
                ));
            }
        }
        let root = metadata.as_object()
            .and_then(|object| object.get("latestRootBlobId"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| AgentDbProjectionError::CheckpointRootInvalid(
                "persisted metadata requires a string latestRootBlobId".into(),
            ))?;
        let root = decode_hex(root).map_err(AgentDbProjectionError::LatestRootHex)?;
        if root.is_empty() {
            return Ok(None);
        }
        Ok(Some(root))
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

    pub fn set_unread_state(
        &self,
        state: &UnreadState,
    ) -> Result<bool, AgentDbProjectionError> {
        self.write_unread_state_owner(state)
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
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                db.execute(CLEAR_BLOBS_SQL, [])?;
                db.execute(
                    SET_KV_SQL,
                    params![KV_LEGACY_BLOB_RETIREMENT, version.to_string()],
                )?;
                Ok(true)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
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
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
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
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
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
        self.run_write("recordRequestId", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let mut records = parse_request_records(
                    read_kv(db, KV_REQUEST_IDS)?.as_deref(),
                );
                if records.is_empty() {
                    if let Some(legacy) = read_kv(db, KV_LATEST_REQUEST_ID)?
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
                Ok(db.execute(SET_KV_SQL, params![KV_REQUEST_IDS, raw])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
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
        self.run_write("recordEpisodeTurn", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let mut turns = parse_pending_episode_turns(
                    read_kv(db, KV_EPISODE)?.as_deref(),
                );
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
                Ok(db.execute(SET_KV_SQL, params![KV_EPISODE, raw])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn clear_pending_episode_turns(&self) -> Result<bool, AgentDbProjectionError> {
        self.run_write("clearPendingEpisodeTurns", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(DELETE_KV_SQL, params![KV_EPISODE])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
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

    pub fn read_pending_profile_mutation(
        &self,
    ) -> Result<Option<PendingProfileMutation>, AgentDbProjectionError> {
        self.read_kv(KV_PENDING_PROFILE_MUTATION)?
            .map(|raw| serde_json::from_str(&raw).map_err(AgentDbProjectionError::from))
            .transpose()
    }

    pub fn prepare_pending_profile_mutation(
        &self,
        mutation: &PendingProfileMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.mutation_id.trim().is_empty()
            || mutation.session_key.trim().is_empty()
            || mutation.session_id.trim().is_empty()
            || mutation.lifecycle_revision.trim().is_empty()
            || mutation.before_files.is_empty()
            || !matches!(
                mutation.operation.as_str(),
                "profile-update" | "avatar-replace" | "avatar-remove"
            )
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending profile mutation identity or file state is incomplete".into(),
            ));
        }
        let encoded = serde_json::to_string(mutation)?;
        self.run_write("preparePendingProfileMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let raw_admission = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?;
                let Some(raw_admission) = raw_admission else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                if db
                    .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                    .is_some()
                {
                    return Ok(false);
                }
                Ok(db
                    .execute(SET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION, &encoded])?
                    > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn seal_pending_profile_mutation(
        &self,
        mutation: &PendingProfileMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.after_files.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending profile mutation cannot be sealed without after-state".into(),
            ));
        }
        let expected = serde_json::to_string(mutation)?;
        self.run_write("sealPendingProfileMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                let Some(raw_pending) = db
                    .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                let mut unsealed = mutation.clone();
                unsealed.after_files.clear();
                let unsealed = serde_json::to_string(&unsealed)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if raw_pending != unsealed || !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                Ok(db.execute(
                    COMPARE_AND_SET_KV_SQL,
                    params![expected, KV_PENDING_PROFILE_MUTATION, raw_pending],
                )? == 1)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn commit_pending_profile_mutation(
        &self,
        mutation: &PendingProfileMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.after_files.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending profile mutation cannot commit without after-state".into(),
            ));
        }
        let expected = serde_json::to_string(mutation)?;
        self.run_write("commitPendingProfileMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let raw_admission = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?;
                let Some(raw_admission) = raw_admission else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                let Some(raw_pending) = db
                    .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                if raw_pending != expected {
                    return Ok(false);
                }
                let current = db
                    .query_row(GET_KV_SQL, params![KV_PROFILE], |row| row.get::<_, String>(0))
                    .optional()?;
                let current_profile = parse_profile(current.as_deref());
                if current_profile != mutation.previous_profile {
                    return Ok(false);
                }
                if let Some(next) = mutation.next_profile.as_ref() {
                    let encoded = serde_json::json!({
                        "description": next.description.clone(),
                        "avatarPath": next.avatar_path.clone(),
                    })
                    .to_string();
                    db.execute(SET_KV_SQL, params![KV_PROFILE, encoded])?;
                }
                Ok(db.execute(DELETE_KV_SQL, params![KV_PENDING_PROFILE_MUTATION])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn clear_pending_profile_mutation(
        &self,
        mutation: &PendingProfileMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        let expected = serde_json::to_string(mutation)?;
        self.run_write("clearPendingProfileMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let Some(raw_admission) = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                let Some(raw_pending) = db
                    .query_row(GET_KV_SQL, params![KV_PENDING_PROFILE_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                if raw_pending != expected {
                    return Ok(false);
                }
                Ok(db.execute(DELETE_KV_SQL, params![KV_PENDING_PROFILE_MUTATION])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn read_pending_memory_mutation(
        &self,
    ) -> Result<Option<PendingMemoryMutation>, AgentDbProjectionError> {
        self.read_kv(KV_PENDING_MEMORY_MUTATION)?
            .map(|raw| serde_json::from_str(&raw).map_err(AgentDbProjectionError::from))
            .transpose()
    }

    pub fn prepare_pending_memory_mutation(
        &self,
        mutation: &PendingMemoryMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.mutation_id.trim().is_empty()
            || mutation.session_key.trim().is_empty()
            || mutation.session_id.trim().is_empty()
            || mutation.lifecycle_revision.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending memory mutation identity is incomplete".into(),
            ));
        }
        let encoded = serde_json::to_string(mutation)?;
        self.run_write("preparePendingMemoryMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let raw_admission = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?;
                let Some(raw_admission) = raw_admission else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                if db
                    .query_row(GET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                    .is_some()
                {
                    return Ok(false);
                }
                Ok(db
                    .execute(SET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION, &encoded])?
                    > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn commit_pending_memory_mutation(
        &self,
        mutation: &PendingMemoryMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.mutation_id.trim().is_empty()
            || mutation.session_key.trim().is_empty()
            || mutation.session_id.trim().is_empty()
            || mutation.lifecycle_revision.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending memory mutation identity is incomplete".into(),
            ));
        }
        let expected = serde_json::to_string(mutation)?;
        self.run_write("commitPendingMemoryMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let raw_admission = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?;
                let Some(raw_admission) = raw_admission else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                let Some(raw_pending) = db
                    .query_row(GET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                if raw_pending != expected {
                    return Ok(false);
                }
                let changed_snapshot = db.execute(DELETE_KV_SQL, params![KV_MEMORY_SNAPSHOT])? > 0;
                let changed_journal =
                    db.execute(DELETE_KV_SQL, params![KV_PENDING_MEMORY_MUTATION])? > 0;
                Ok(changed_snapshot || changed_journal)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn clear_pending_memory_mutation(
        &self,
        mutation: &PendingMemoryMutation,
    ) -> Result<bool, AgentDbProjectionError> {
        if mutation.mutation_id.trim().is_empty()
            || mutation.session_key.trim().is_empty()
            || mutation.session_id.trim().is_empty()
            || mutation.lifecycle_revision.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "pending memory mutation identity is incomplete".into(),
            ));
        }
        let expected = serde_json::to_string(mutation)?;
        self.run_write("clearPendingMemoryMutation", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let Some(raw_admission) = db
                    .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                if admission.session_key != mutation.session_key
                    || admission.session_id != mutation.session_id
                    || admission.lifecycle_revision != mutation.lifecycle_revision
                {
                    return Ok(false);
                }
                let Some(raw_pending) = db
                    .query_row(GET_KV_SQL, params![KV_PENDING_MEMORY_MUTATION], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()?
                else {
                    return Ok(false);
                };
                if raw_pending != expected {
                    return Ok(false);
                }
                Ok(db.execute(DELETE_KV_SQL, params![KV_PENDING_MEMORY_MUTATION])? > 0)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
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
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
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
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(changed)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })
    }

    pub fn clear_clone_mutation_journals(&self) -> Result<bool, AgentDbProjectionError> {
        self.run_write("clearCloneMutationJournals", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                let mut changed = false;
                for key in [KV_PENDING_MEMORY_MUTATION, KV_PENDING_PROFILE_MUTATION] {
                    changed |= db.execute(DELETE_KV_SQL, params![key])? > 0;
                }
                Ok(changed)
            })();
            match result {
                Ok(changed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
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
                if !session_work_write_is_admitted(db)? {
                    return Ok(Vec::new());
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
                Ok(inserted)
            })();
            match result {
                Ok(inserted) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
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

    /// Plan and commit a fresh append against one physical-row snapshot. The
    /// crate-internal planner must be pure, nonblocking and nonreentrant: no
    /// Session/DB calls, filesystem I/O, listeners or other owner mutations.
    /// Unlike compatibility writes this never retries the planner, recovers the
    /// database after an error, or interprets closed/busy as an empty success.
    pub(crate) fn commit_transcript_entries<T>(
        &self,
        session_key: Option<&str>,
        expected_lifecycle_revision: Option<&str>,
        planner: impl FnOnce(&[serde_json::Value])
            -> Result<(T, Vec<serde_json::Value>), String>,
    ) -> Result<CommittedTranscriptAppend<T>, AgentDbProjectionError> {
        let session_key = session_key.map(str::trim);
        if session_key == Some("")
            || expected_lifecycle_revision.is_some_and(|revision| revision.trim().is_empty())
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "fresh transcript append requires nonempty supplied identity".into(),
            ));
        }
        let mut guard = self.connection.lock()
            .map_err(|_| AgentDbProjectionError::OwnerPoisoned)?;
        let db = guard.as_mut().ok_or_else(|| {
            AgentDbProjectionError::WorkAdmissionInvalid("agent database is closed".into())
        })?;
        // RAII remains an unwind backstop. Normal failure returns must
        // establish rollback explicitly before the connection is reusable.
        let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let prepared = (|| -> Result<(T, Vec<serde_json::Value>), AgentDbProjectionError> {
            match (read_kv(&transaction, KV_SESSION_WORK_ADMISSION)?, session_key) {
                (Some(raw), Some(session_key)) => {
                    let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw)?;
                    if !session_admission_identity_is_bound(&transaction, &admission)?
                        || admission.session_key != session_key
                        || admission.archived_at_ms.is_some()
                        || admission.deletion_pending
                        || expected_lifecycle_revision
                            .is_some_and(|revision| revision != admission.lifecycle_revision)
                    {
                        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                            "session identity or generation is not admitted for fresh append".into(),
                        ));
                    }
                }
                (None, None) if expected_lifecycle_revision.is_none() => {}
                _ => return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "fresh transcript append requires the current session admission identity".into(),
                )),
            }

            let existing = {
                let mut statement = transaction.prepare(
                    "SELECT id, entry FROM transcript_entries ORDER BY seq",
                )?;
                let rows = statement.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                let mut entries = Vec::new();
                for row in rows {
                    let (id, raw) = row?;
                    let entry: serde_json::Value = serde_json::from_str(&raw).map_err(|_| {
                        AgentDbProjectionError::TranscriptAppendInvalid(
                            "stored transcript row is not valid JSON".into(),
                        )
                    })?;
                    if id.trim().is_empty() || !entry.is_object()
                        || entry.get("id").and_then(serde_json::Value::as_str) != Some(id.as_str())
                    {
                        return Err(AgentDbProjectionError::TranscriptAppendInvalid(
                            "stored transcript identity is missing or differs from its SQL identity".into(),
                        ));
                    }
                    entries.push(entry);
                }
                entries
            };
            let (value, entries) = planner(&existing)
                .map_err(AgentDbProjectionError::TranscriptAppendInvalid)?;
            for entry in &entries {
                let id = entry.get("id").and_then(serde_json::Value::as_str)
                    .filter(|id| !id.trim().is_empty())
                    .ok_or_else(|| AgentDbProjectionError::TranscriptAppendInvalid(
                        "every fresh transcript row requires an identity".into(),
                    ))?;
                if entry.get("kind").and_then(serde_json::Value::as_str)
                    .is_none_or(|kind| kind.trim().is_empty())
                    || session_key.is_some_and(|key| {
                        entry.get("sessionKey").and_then(serde_json::Value::as_str)
                            .map(str::trim) != Some(key)
                    })
                {
                    return Err(AgentDbProjectionError::TranscriptAppendInvalid(
                        "every fresh transcript row requires its kind and admitted provenance".into(),
                    ));
                }
                if transaction.execute(INSERT_TRANSCRIPT_ENTRY_SQL, params![id, entry.to_string()])? != 1 {
                    return Err(AgentDbProjectionError::TranscriptAppendInvalid(
                        "fresh transcript insertion was ignored or its identity is already occupied".into(),
                    ));
                }
            }
            Ok((value, entries))
        })();
        let (value, entries) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                let rollback = transaction.rollback();
                let clean = db.is_autocommit();
                if rollback.is_err() || !clean {
                    if !clean {
                        self.retire_channel_inbound_connection(&mut guard);
                    }
                    return Err(AgentDbProjectionError::TranscriptCommitUnknown(format!(
                        "fresh append rollback was not confirmed after {error}",
                    )));
                }
                return Err(error);
            }
        };
        if let Err(error) = transaction.commit() {
            // COMMIT's destructor cleanup is best-effort. Its result remains
            // unknown even when cleanup succeeds; an unsettled connection must
            // be retired before another owner operation can observe its rows.
            if !db.is_autocommit() {
                let _ = db.execute_batch("ROLLBACK");
            }
            if !db.is_autocommit() {
                self.retire_channel_inbound_connection(&mut guard);
            }
            return Err(AgentDbProjectionError::TranscriptCommitUnknown(error.to_string()));
        }
        if !entries.is_empty() {
            bump_db_write_generation(&self.db_path);
        }
        drop(guard);
        Ok(CommittedTranscriptAppend { value, entries, db_path: self.db_path.clone() })
    }

    /// Append entries only while the canonical session is currently admitted.
    /// The admission check and transcript insert share one SQLite transaction
    /// so archive/restore cannot race a channel or other session-scoped
    /// producer at the durable write edge.
    pub fn append_transcript_entries_if_session_admitted(
        &self,
        session_key: &str,
        entries: &[serde_json::Value],
    ) -> Result<usize, AgentDbProjectionError> {
        let session_key = session_key.trim();
        if session_key.is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session-scoped transcript append requires a session key".into(),
            ));
        }
        if entries.is_empty() {
            return Ok(0);
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
            if !session_work_write_is_admitted(db)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session owner identity is not admitted for transcript append".into(),
                ));
            }
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
            let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session is not admitted for transcript append".into(),
                ));
            }
            let mut inserted = Vec::new();
            let mut statement = db.prepare(INSERT_TRANSCRIPT_ENTRY_SQL)?;
            for entry in entries {
                let Some(id) = entry.get("id").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                if entry.get("kind").and_then(serde_json::Value::as_str).is_none() {
                    continue;
                }
                if entry
                    .get("sessionKey")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    != Some(session_key)
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "session-scoped transcript append requires matching entry provenance"
                            .into(),
                    ));
                }
                if statement.execute(params![id, entry.to_string()])? > 0 {
                    inserted.push(entry.clone());
                }
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
                drop(guard);
                if !inserted.is_empty() {
                    publish_entries_upserted(&self.db_path, inserted);
                }
                Ok(count)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    /// Prepare and append one channel claim against its original Session
    /// binding. This deliberately does not use the generic write helper:
    /// neither an ignored insert nor an automatic storage retry is an
    /// acknowledgement of the complete channel batch.
    pub fn append_channel_inbound_entries(
        &self,
        attempt: &mut Option<ChannelInboundAppendAttempt>,
        prepare: impl FnOnce(
            &SessionWorkAdmissionRecord,
            &[serde_json::Value],
        ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure>,
    ) -> ChannelInboundAppendOutcome {
        let (outcome, publication) = self.append_channel_inbound_entries_deferred(attempt, prepare);
        if let Some(publication) = publication {
            publication.publish();
        }
        outcome
    }

    /// The Session facade also owns a non-reentrant filesystem mutex. Carry
    /// only confirmed publication out to that caller so both locks are gone
    /// before synchronous transcript observers run.
    pub(crate) fn append_channel_inbound_entries_deferred(
        &self,
        attempt: &mut Option<ChannelInboundAppendAttempt>,
        prepare: impl FnOnce(
            &SessionWorkAdmissionRecord,
            &[serde_json::Value],
        ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure>,
    ) -> (ChannelInboundAppendOutcome, Option<ChannelInboundAppendPublication>) {
        self.append_channel_inbound_entries_deferred_with_control(
            attempt,
            prepare,
            &ChannelInboundTransactionControl::default(),
        )
    }

    fn append_channel_inbound_entries_deferred_with_control(
        &self,
        attempt: &mut Option<ChannelInboundAppendAttempt>,
        prepare: impl FnOnce(
            &SessionWorkAdmissionRecord,
            &[serde_json::Value],
        ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure>,
        control: &ChannelInboundTransactionControl,
    ) -> (ChannelInboundAppendOutcome, Option<ChannelInboundAppendPublication>) {
        let mut guard = match self.connection.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // An unwound owner may still have a live transaction. Do not
                // allow any later operation to settle it implicitly.
                let mut guard = poisoned.into_inner();
                self.retire_channel_inbound_connection(&mut guard);
                return (ChannelInboundAppendOutcome::Unknown(channel_inbound_failure(
                    ChannelInboundAppendFailureKind::RollbackUnconfirmed,
                    "channel append owner lock is poisoned",
                )), None);
            }
        };
        let Some(db) = guard.as_ref().filter(|_| !self.is_closed()) else {
            return (ChannelInboundAppendOutcome::NotWritten(channel_inbound_failure(
                ChannelInboundAppendFailureKind::Closed,
                "channel append owner is closed",
            )), None);
        };
        let (outcome, retire) = append_channel_inbound_transaction(
            db,
            &self.agent_dir_name,
            attempt,
            prepare,
            control,
        );
        if retire {
            self.retire_channel_inbound_connection(&mut guard);
        }
        let publication = if let ChannelInboundAppendOutcome::Committed { inserted_count } = &outcome {
            if *inserted_count > 0 {
                bump_db_write_generation(&self.db_path);
                Some(ChannelInboundAppendPublication {
                    db_path: self.db_path.clone(),
                    entries: attempt.as_ref().and_then(|attempt| attempt.entries.clone())
                        .expect("confirmed channel append retains its prepared entries"),
                })
            } else {
                None
            }
        } else {
            None
        };
        drop(guard);
        (outcome, publication)
    }

    fn retire_channel_inbound_connection(
        &self,
        connection: &mut Option<rusqlite::Connection>,
    ) {
        self.closed.store(true, Ordering::Release);
        drop(connection.take());
        if self.handle_registered.swap(false, Ordering::AcqRel) {
            release_live_db_handle(&self.db_path);
        }
    }

    /// Atomically publish one session-scoped spend-guard card together with
    /// its owner-local state and activity projection. The deletion/archive
    /// fence is checked in the same SQLite transaction as every mutation.
    pub fn append_spend_guard_card_if_session_admitted(
        &self,
        session_key: &str,
        entry: &serde_json::Value,
        state: &SpendGuardState,
        activity_at: f64,
    ) -> Result<bool, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let Some(entry_id) = entry.get("id").and_then(serde_json::Value::as_str) else {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "spend-guard card requires an entry identity".into(),
            ));
        };
        if session_key.is_empty()
            || entry.get("kind").and_then(serde_json::Value::as_str).is_none()
            || entry
                .get("sessionKey")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                != Some(session_key)
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "spend-guard card requires matching session provenance".into(),
            ));
        }
        let spend_guard_raw = serialize_spend_guard_state(state);
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
        let result = (|| -> Result<(bool, bool), AgentDbProjectionError> {
            if !session_work_write_is_admitted(db)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session owner identity is not admitted for spend-guard card".into(),
                ));
            }
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
            let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session is not admitted for spend-guard card".into(),
                ));
            }
            let inserted = db.execute(
                INSERT_TRANSCRIPT_ENTRY_SQL,
                params![entry_id, entry.to_string()],
            )? > 0;
            if !inserted {
                return Ok((false, false));
            }
            let spend_changed = if let Some(raw) = spend_guard_raw.as_deref() {
                db.execute(SET_KV_SQL, params![KV_SPEND_GUARD, raw])? > 0
            } else {
                db.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD])? > 0
            };
            let legacy_changed =
                db.execute(DELETE_KV_SQL, params![KV_SPEND_GUARD_LEGACY])? > 0;
            let current = parse_unread_state(
                read_kv(db, KV_UNREAD)?.as_deref(),
            );
            let activity_changed = current.last_activity_at < activity_at;
            if activity_changed {
                let raw = serde_json::json!({
                    "lastActivityAt": activity_at,
                    "lastViewedAt": current.last_viewed_at,
                    "isManuallyUnread": current.is_manually_unread,
                    "unreadCount": current.unread_count + 1.0,
                })
                .to_string();
                db.execute(SET_KV_SQL, params![KV_UNREAD, raw])?;
            }
            Ok((inserted, spend_changed || legacy_changed || activity_changed))
        })();
        match result {
            Ok((false, _)) => {
                db.execute_batch("ROLLBACK")?;
                Ok(false)
            }
            Ok((inserted, metadata_changed)) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                drop(guard);
                if inserted || metadata_changed {
                    bump_db_write_generation(&self.db_path);
                }
                if inserted {
                    publish_entries_upserted(&self.db_path, vec![entry.clone()]);
                }
                if metadata_changed {
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata(KV_SPEND_GUARD.into()),
                    );
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata(KV_UNREAD.into()),
                    );
                }
                Ok(inserted)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    /// Atomically append transcript entries and clear one captured outgoing
    /// media fence. A stale or rebound fence rolls back the transcript write.
    pub fn append_transcript_entries_and_release_media(
        &self,
        entries: &[serde_json::Value],
        expected_session_key: &str,
        expected_lifecycle_revision: &str,
        media_work_id: &str,
    ) -> Result<usize, AgentDbProjectionError> {
        if entries.is_empty()
            || expected_session_key.trim().is_empty()
            || expected_lifecycle_revision.trim().is_empty()
            || media_work_id.trim().is_empty()
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "media settlement requires transcript entries, session identity, generation, and work identity"
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
            if !session_work_write_is_admitted(db)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session owner identity is not admitted for media settlement".into(),
                ));
            }
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
            if !session_admission_identity_is_well_formed(&admission) {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "outgoing media settlement admission identity is malformed".into(),
                ));
            }
            let receipt_id = format!("outgoing-media-settlement:{}", media_work_id.trim());
            if let Some(raw_receipt) = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![&receipt_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?
            {
                if !session_admission_identity_is_well_formed(&admission)
                    || admission.session_key != expected_session_key.trim()
                    || admission.lifecycle_revision != expected_lifecycle_revision.trim()
                    || admission.archived_at_ms.is_some()
                    || admission.deletion_pending
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "media settlement replay is stale or session is not admitted".into(),
                    ));
                }
                let receipt = parse_owner_transcript_entry(&raw_receipt).ok_or_else(|| {
                    AgentDbProjectionError::WorkAdmissionInvalid(
                        "outgoing media settlement receipt is malformed".into(),
                    )
                })?;
                let receipt_matches = receipt
                    .get("kind")
                    .and_then(serde_json::Value::as_str)
                    == Some("outgoing-media-settlement")
                    && receipt.get("id").and_then(serde_json::Value::as_str)
                        == Some(receipt_id.as_str())
                    && receipt
                        .get("sessionKey")
                        .and_then(serde_json::Value::as_str)
                        == Some(expected_session_key.trim())
                    && receipt
                        .get("mediaWorkId")
                        .and_then(serde_json::Value::as_str)
                        == Some(media_work_id.trim())
                    && receipt
                        .get("lifecycleRevision")
                        .and_then(serde_json::Value::as_str)
                        == Some(expected_lifecycle_revision.trim());
                if !receipt_matches {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "outgoing media settlement receipt is rebound".into(),
                    ));
                }
                if admission.admitted_media_work_id.is_some()
                    || !admission.admitted_media_paths.is_empty()
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "outgoing media settlement receipt conflicts with a pending media fence"
                            .into(),
                    ));
                }
                let receipt_entry_ids = receipt
                    .get("transcriptEntryIds")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| {
                        AgentDbProjectionError::WorkAdmissionInvalid(
                            "outgoing media settlement receipt is missing transcript identities"
                                .into(),
                        )
                    })?;
                let incoming_entry_ids = entries
                    .iter()
                    .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
                    .collect::<Vec<_>>();
                if receipt_entry_ids.len() != incoming_entry_ids.len()
                    || !receipt_entry_ids.iter().zip(incoming_entry_ids.iter()).all(
                        |(expected, actual)| expected.as_str() == Some(*actual),
                    )
                {
                    return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                        "outgoing media settlement replay conflicts with the committed transcript"
                            .into(),
                    ));
                }
                for entry in entries {
                    let entry_id = entry
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            AgentDbProjectionError::WorkAdmissionInvalid(
                                "outgoing media settlement replay has no entry identity".into(),
                            )
                        })?;
                    let raw_existing = db
                        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                            row.get::<_, String>(0)
                        })
                        .optional()?
                        .ok_or_else(|| {
                            AgentDbProjectionError::WorkAdmissionInvalid(
                                "outgoing media settlement replay lost its transcript entry"
                                    .into(),
                            )
                        })?;
                    if parse_transcript_entry(&raw_existing).as_ref() != Some(entry) {
                        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                            "outgoing media settlement replay conflicts with transcript content"
                                .into(),
                        ));
                    }
                }
                return Ok(Vec::new());
            }
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != expected_session_key.trim()
                || admission.lifecycle_revision != expected_lifecycle_revision.trim()
                || admission.admitted_media_work_id.as_deref() != Some(media_work_id.trim())
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "media settlement generation or identity is stale".into(),
                ));
            }
            let mut inserted = Vec::new();
            {
                let mut statement = db.prepare(INSERT_TRANSCRIPT_ENTRY_SQL)?;
                for entry in entries {
                    let id = entry
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .filter(|id| !id.trim().is_empty())
                        .ok_or_else(|| {
                            AgentDbProjectionError::WorkAdmissionInvalid(
                                "media settlement requires transcript entry identities".into(),
                            )
                        })?;
                    if entry
                        .get("kind")
                        .and_then(serde_json::Value::as_str)
                        .filter(|kind| !kind.trim().is_empty())
                        .is_none()
                    {
                        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                            "media settlement requires transcript entry kinds".into(),
                        ));
                    }
                    if entry
                        .get("sessionKey")
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        != Some(expected_session_key.trim())
                    {
                        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                            "media settlement requires matching entry provenance".into(),
                        ));
                    }
                    if !is_valid_transcript_entry(entry) {
                        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                            "media settlement requires valid transcript entry payloads".into(),
                        ));
                    }
                    if statement.execute(params![id, entry.to_string()])? > 0 {
                        inserted.push(entry.clone());
                    }
                }
            }
            if inserted.len() != entries.len() {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "media settlement did not persist every transcript entry".into(),
                ));
            }
            let receipt = media_settlement_receipt_entry(
                &receipt_id,
                expected_session_key.trim(),
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
        self.commit_compaction_with_cancellation(commit, &|| false)
    }

    /// The cancellation reader must not block, panic, or reenter a DB owner.
    /// Its final read linearizes cancellation against this transaction commit.
    pub fn commit_compaction_with_cancellation(
        &self,
        commit: &CompactionCommit,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<CompactionCommitOutcome, AgentDbProjectionError> {
        let entry_id = commit.entry_id.trim();
        let source_turn_id = commit.source_turn_id.trim();
        let expected_revision = commit.expected_lifecycle_revision.trim();
        let expected_tail_id = commit.expected_transcript_tail_id.trim();
        if entry_id.is_empty()
            || source_turn_id.is_empty()
            || expected_revision.is_empty()
            || expected_tail_id.is_empty()
        {
            return Err(AgentDbProjectionError::CompactionInvalid(
                "entry_id, source_turn_id, expected_transcript_tail_id, and expected_lifecycle_revision are required".into(),
            ));
        }
        let summary = commit.summary.trim();
        if summary.is_empty() {
            return Err(AgentDbProjectionError::CompactionInvalid(
                "summary must be non-empty".into(),
            ));
        }
        if summary.chars().count() > MAX_COMPACTION_SUMMARY_CHARS {
            return Err(AgentDbProjectionError::CompactionInvalid(format!(
                "summary exceeds {MAX_COMPACTION_SUMMARY_CHARS} characters"
            )));
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

        if should_cancel() {
            return Ok(CompactionCommitOutcome::RejectedCancelled);
        }
        db.execute_batch("BEGIN IMMEDIATE")?;
        let mut admission_updated = false;
        let result = (|| -> Result<CompactionCommitOutcome, AgentDbProjectionError> {
            if !session_work_write_is_admitted(db)? {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }
            let raw_admission = read_kv(db, KV_SESSION_WORK_ADMISSION)?;
            let admission = raw_admission
                .as_deref()
                .map(serde_json::from_str::<SessionWorkAdmissionRecord>)
                .transpose()?;
            let existing = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            if let Some(raw) = existing {
                let Some(existing) = parse_owner_transcript_entry(&raw) else {
                    return Err(AgentDbProjectionError::CompactionInvalid(
                        "existing compaction identity contains malformed JSON".into(),
                    ));
                };
                if existing.get("kind").and_then(serde_json::Value::as_str) == Some("compaction") {
                    compaction_first_kept_entry_id(&existing)?;
                }
                if compaction_payload_matches(&existing, commit) {
                    if let Some(admission) = admission.as_ref() {
                        let current_revision = admission.lifecycle_revision.as_str();
                        let result_revision = existing
                            .get("lifecycleRevision")
                            .and_then(serde_json::Value::as_str);
                        let source_revision = existing
                            .get("sourceLifecycleRevision")
                            .and_then(serde_json::Value::as_str);
                        // Historical compactions advanced metadata only. Their
                        // unchanged source admission remains a compatible retry
                        // only while no successor compaction has been written.
                        let latest_compaction_id = db
                            .query_row(
                                "SELECT id FROM transcript_entries \
                                 WHERE json_extract(entry, '$.kind') = 'compaction' \
                                 ORDER BY seq DESC LIMIT 1",
                                [],
                                |row| row.get::<_, String>(0),
                            )
                            .optional()?;
                        if latest_compaction_id.as_deref() != Some(entry_id)
                            || (Some(current_revision) != result_revision
                                && Some(current_revision) != source_revision)
                        {
                            return Ok(CompactionCommitOutcome::RejectedStale);
                        }
                    }
                    return Ok(CompactionCommitOutcome::AlreadyCommitted(existing));
                }
                return Err(AgentDbProjectionError::CompactionConflict(entry_id.to_string()));
            }

            if admission.as_ref().is_some_and(|record| {
                record.admitted_work_id.is_some()
                    || record.admitted_media_work_id.is_some()
                    || !record.admitted_media_paths.is_empty()
                    || record.provider_review_id.is_some()
                    || record.provider_review.is_some()
                    || record.provider_review_continuation_run_id.is_some()
                    || !matches!(record.provider_review_phase.as_deref(), None | Some("completed"))
            }) || read_kv(db, KV_PENDING_MEMORY_MUTATION)?.is_some()
                || read_kv(db, KV_PENDING_PROFILE_MUTATION)?.is_some()
            {
                return Ok(CompactionCommitOutcome::RejectedStale);
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
            let metadata_revision = metadata
                .get("lifecycleRevision")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AgentDbProjectionError::LifecycleRevisionUnavailable(
                        "metadata has no lifecycleRevision".into(),
                    )
                })?;
            let current_revision = admission
                .as_ref()
                .map(|record| record.lifecycle_revision.as_str())
                .unwrap_or(metadata_revision);
            if !session_work_write_is_admitted(db)? {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }
            if current_revision != expected_revision {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }
            let running_cron = db.query_row(
                "SELECT EXISTS(SELECT 1 FROM cron_session_runs \
                 WHERE lifecycle_revision = ?1 AND state = 'running')",
                params![current_revision],
                |row| row.get::<_, bool>(0),
            )?;
            if running_cron {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }

            let current_tail_id = db
                .query_row(
                    "SELECT id FROM transcript_entries ORDER BY seq DESC LIMIT 1",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            if current_tail_id.as_deref() != Some(expected_tail_id) {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }

            if let Some(first_kept_entry_id) = commit.first_kept_entry_id.as_deref() {
                let present = db
                    .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![first_kept_entry_id.trim()], |row| {
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
            entry.insert("summary".into(), serde_json::Value::String(summary.to_string()));
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
            validate_compaction_retained_replay(db, &entry)?;
            db.execute(
                INSERT_TRANSCRIPT_ENTRY_SQL,
                params![entry_id, entry.to_string()],
            )?;

            let metadata_object = metadata.as_object_mut().ok_or_else(|| {
                AgentDbProjectionError::CompactionInvalid("metadata is not an object".into())
            })?;
            metadata_object.insert(
                "lifecycleRevision".into(),
                serde_json::Value::String(next_revision.clone()),
            );
            let next_metadata = encode_hex(&serde_json::to_vec(&metadata)?);
            if db.execute(
                COMPARE_AND_SET_KV_SQL,
                params![next_metadata, "metadata", raw_metadata],
            )? != 1
            {
                return Ok(CompactionCommitOutcome::RejectedStale);
            }
            if let Some(raw_admission) = raw_admission {
                // Preserve fields written by newer owners instead of
                // serializing only the currently known admission struct.
                let mut admission_json: serde_json::Value = serde_json::from_str(&raw_admission)?;
                let admission_object = admission_json.as_object_mut().ok_or_else(|| {
                    AgentDbProjectionError::CompactionInvalid("admission is not an object".into())
                })?;
                admission_object.insert(
                    "lifecycle_revision".into(),
                    serde_json::Value::String(next_revision),
                );
                if db.execute(
                    COMPARE_AND_SET_KV_SQL,
                    params![admission_json.to_string(), KV_SESSION_WORK_ADMISSION, raw_admission],
                )? != 1
                {
                    return Ok(CompactionCommitOutcome::RejectedStale);
                }
                admission_updated = true;
            }
            Ok(CompactionCommitOutcome::Committed(entry))
        })();

        match result {
            Ok(outcome @ CompactionCommitOutcome::Committed(_)) => {
                if should_cancel() {
                    db.execute_batch("ROLLBACK")?;
                    return Ok(CompactionCommitOutcome::RejectedCancelled);
                }
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                // Listeners may synchronously read this same owner.
                drop(guard);
                bump_db_write_generation(&self.db_path);
                if let CompactionCommitOutcome::Committed(entry) = &outcome {
                    publish_entries_upserted(&self.db_path, vec![entry.clone()]);
                    notify_agent_db_listeners(
                        &self.db_path,
                        AgentDbListenerChannel::Metadata("lifecycleRevision".into()),
                    );
                    if admission_updated {
                        notify_agent_db_listeners(
                            &self.db_path,
                            AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
                        );
                    }
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
        let first_kept = compaction_first_kept_entry_id(&compaction)?;
        let start = match first_kept {
            Some(first_kept) => entries[..compaction_index]
                .iter()
                .position(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(first_kept))
                .ok_or_else(|| {
                    AgentDbProjectionError::CompactionInvalid(format!(
                        "compaction boundary {first_kept} is missing"
                    ))
                })?,
            None => compaction_index + 1,
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
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(Option<serde_json::Value>, bool), rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok((None, false));
                }
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
            })();
            match result {
                Ok(outcome) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(outcome)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if let Some(entry) = updated.as_ref() {
            publish_entries_upserted(&self.db_path, vec![entry.clone()]);
        }
        Ok(updated)
    }

    /// Update one transcript entry only while the captured session remains
    /// admitted. The admission check and update share one SQLite transaction
    /// so an archived/rebound session cannot settle a permission card after a
    /// preflight check has passed.
    pub fn update_transcript_entry_if_session_admitted(
        &self,
        session_key: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        if session_key.trim().is_empty() || entry_id.trim().is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session-scoped transcript update requires session key and entry identity".into(),
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
        let result = (|| -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
            if !session_work_write_is_admitted(db)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session owner identity is not admitted for transcript update".into(),
                ));
            }
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
            let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key.trim()
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session is not admitted for transcript update".into(),
                ));
            }
            let raw_entry = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(raw_entry) = raw_entry else {
                return Ok(None);
            };
            let entry = parse_transcript_entry(&raw_entry).ok_or_else(|| {
                AgentDbProjectionError::WorkAdmissionInvalid(
                    "target transcript entry is not valid JSON".into(),
                )
            })?;
            if entry
                .get("sessionKey")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                != Some(session_key.trim())
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "target transcript entry provenance does not match admitted session".into(),
                ));
            }
            if db.execute(
                UPDATE_TRANSCRIPT_ENTRY_SQL,
                params![next.to_string(), entry_id],
            )? != 1
            {
                return Ok(None);
            }
            Ok(Some(next.clone()))
        })();
        match result {
            Ok(updated) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                drop(guard);
                if let Some(entry) = updated.as_ref() {
                    publish_entries_upserted(&self.db_path, vec![entry.clone()]);
                }
                Ok(updated)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
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
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(Option<serde_json::Value>, bool), rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok((None, false));
                }
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
            })();
            match result {
                Ok(outcome) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(outcome)
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
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

    /// Settle one transcript identity only while the captured session remains
    /// admitted. The existing row and replacement row must both retain the
    /// same session provenance inside the owner transaction.
    pub fn settle_transcript_entry_identity_if_session_admitted(
        &self,
        session_key: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
        let session_key = session_key.trim();
        let entry_id = entry_id.trim();
        let Some(next_id) = next.get("id").and_then(serde_json::Value::as_str) else {
            return Ok(None);
        };
        if session_key.is_empty() || entry_id.is_empty() || next_id.trim().is_empty() {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session-scoped transcript identity settlement requires session key and entry identities"
                    .into(),
            ));
        }
        if next
            .get("sessionKey")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            != Some(session_key)
        {
            return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                "session-scoped transcript identity settlement requires matching entry provenance"
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
        let result = (|| -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
            if !session_work_write_is_admitted(db)? {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session owner identity is not admitted for transcript identity settlement".into(),
                ));
            }
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
            let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
            if !session_admission_identity_is_well_formed(&admission)
                || admission.session_key != session_key
                || admission.archived_at_ms.is_some()
                || admission.deletion_pending
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "session is not admitted for transcript identity settlement".into(),
                ));
            }
            let raw_entry = db
                .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
                    row.get::<_, String>(0)
                })
                .optional()?;
            let Some(raw_entry) = raw_entry else {
                return Ok(None);
            };
            let entry = parse_transcript_entry(&raw_entry).ok_or_else(|| {
                AgentDbProjectionError::WorkAdmissionInvalid(
                    "target transcript entry is not valid JSON".into(),
                )
            })?;
            if entry
                .get("sessionKey")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                != Some(session_key)
            {
                return Err(AgentDbProjectionError::WorkAdmissionInvalid(
                    "target transcript entry provenance does not match admitted session".into(),
                ));
            }
            if db.execute(
                "UPDATE transcript_entries SET id = ?1, entry = ?2 WHERE id = ?3",
                params![next_id, next.to_string(), entry_id],
            )? != 1
            {
                return Ok(None);
            }
            Ok(Some(next.clone()))
        })();
        match result {
            Ok(updated) => {
                if let Err(error) = db.execute_batch("COMMIT") {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(error.into());
                }
                drop(guard);
                if let Some(entry) = updated.as_ref() {
                    if entry_id != next_id {
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
                    publish_entries_upserted(&self.db_path, vec![entry.clone()]);
                }
                Ok(updated)
            }
            Err(error) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    /// Return committed success for an admitted delete, including an already
    /// absent row. Only an actual row removal advances write generation or
    /// publishes a transcript mutation.
    pub fn delete_transcript_entry(
        &self,
        entry_id: &str,
    ) -> Result<bool, AgentDbProjectionError> {
        let (committed, changed) = self.run_write_value("deleteTranscriptEntry", |db| {
            db.execute_batch("BEGIN IMMEDIATE")?;
            let result = (|| -> Result<(bool, bool), rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok((false, false));
                }
                let changed = db.execute(DELETE_TRANSCRIPT_ENTRY_SQL, params![entry_id])? > 0;
                Ok((true, changed))
            })();
            match result {
                Ok(outcome) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok((outcome, outcome.1))
                }
                Err(error) => {
                    let _ = db.execute_batch("ROLLBACK");
                    Err(error)
                }
            }
        })?;
        if changed {
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
            let result = (|| -> Result<bool, rusqlite::Error> {
                if !session_work_write_is_admitted(db)? {
                    return Ok(false);
                }
                db.execute(CLEAR_BLOBS_SQL, [])?;
                db.execute(CLEAR_TRANSCRIPT_ENTRIES_SQL, [])?;
                clear_original_turn_terminal_slots(db)?;
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
                Ok(true)
            })();
            match result {
                Ok(committed) => {
                    if let Err(error) = db.execute_batch("COMMIT") {
                        let _ = db.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                    Ok(committed)
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

fn channel_inbound_failure(
    kind: ChannelInboundAppendFailureKind,
    detail: impl Into<String>,
) -> ChannelInboundAppendFailure {
    ChannelInboundAppendFailure { kind, detail: detail.into() }
}

fn channel_inbound_storage_failure(error: rusqlite::Error) -> ChannelInboundAppendFailure {
    channel_inbound_failure(ChannelInboundAppendFailureKind::Storage, error.to_string())
}

/// An ephemeral notification for an already confirmed commit. This is not a
/// durable receipt and cannot authorize replay or resolve an unknown result.
pub(crate) struct ChannelInboundAppendPublication {
    db_path: PathBuf,
    entries: Vec<serde_json::Value>,
}

impl ChannelInboundAppendPublication {
    pub(crate) fn publish(self) {
        publish_entries_upserted(&self.db_path, self.entries);
    }
}

/// Production always executes the real transaction commands. Unit fault
/// controls alter only a command/acknowledgement, never the outcome decision.
#[derive(Default)]
struct ChannelInboundTransactionControl {
    #[cfg(test)]
    lose_commit_acknowledgement: bool,
    #[cfg(test)]
    fail_rollback: bool,
}

impl ChannelInboundTransactionControl {
    fn commit(&self, db: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        db.execute_batch("COMMIT")?;
        #[cfg(test)]
        if self.lose_commit_acknowledgement {
            return Err(rusqlite::Error::InvalidQuery);
        }
        Ok(())
    }

    fn rollback(&self, db: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        #[cfg(test)]
        if self.fail_rollback {
            return Err(rusqlite::Error::InvalidQuery);
        }
        db.execute_batch("ROLLBACK")
    }
}

fn read_channel_inbound_admission(
    db: &rusqlite::Connection,
    agent_id: &str,
    attempt: Option<&ChannelInboundAppendAttempt>,
) -> Result<SessionWorkAdmissionRecord, ChannelInboundAppendFailure> {
    let raw = read_kv(db, KV_SESSION_WORK_ADMISSION)
        .map_err(channel_inbound_storage_failure)?
        .ok_or_else(|| channel_inbound_failure(
            ChannelInboundAppendFailureKind::Admission,
            "channel append requires an existing session admission record",
        ))?;
    let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw).map_err(|_| {
        channel_inbound_failure(
            ChannelInboundAppendFailureKind::Admission,
            "channel append session admission record is malformed",
        )
    })?;
    if admission.session_id != agent_id
        || !session_admission_identity_is_bound(db, &admission)
            .map_err(channel_inbound_storage_failure)?
        || admission.archived_at_ms.is_some()
        || admission.deletion_pending
    {
        return Err(channel_inbound_failure(
            ChannelInboundAppendFailureKind::Admission,
            "channel append session owner is malformed, archived, or deleting",
        ));
    }
    if attempt.is_some_and(|attempt| {
        attempt.session_key != admission.session_key
            || attempt.session_id != admission.session_id
            || attempt.lifecycle_revision != admission.lifecycle_revision
    }) {
        return Err(channel_inbound_failure(
            ChannelInboundAppendFailureKind::IdentityConflict,
            "channel append captured session identity or lifecycle changed",
        ));
    }
    Ok(admission)
}

fn read_channel_inbound_history(
    db: &rusqlite::Connection,
) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure> {
    let mut statement = db.prepare("SELECT id, entry FROM transcript_entries ORDER BY seq")
        .map_err(channel_inbound_storage_failure)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(channel_inbound_storage_failure)?;
    let mut history = Vec::new();
    for row in rows {
        let (id, raw) = row.map_err(channel_inbound_storage_failure)?;
        let entry = parse_owner_transcript_entry(&raw).ok_or_else(|| {
            channel_inbound_failure(
                ChannelInboundAppendFailureKind::Storage,
                "channel append allocation history contains an undecodable transcript row",
            )
        })?;
        if id.trim().is_empty()
            || entry.get("id").and_then(serde_json::Value::as_str) != Some(id.as_str())
        {
            return Err(channel_inbound_failure(
                ChannelInboundAppendFailureKind::Storage,
                "channel append allocation history contains a mismatched row identity",
            ));
        }
        history.push(entry);
    }
    Ok(history)
}

fn validate_channel_inbound_batch(
    attempt: &ChannelInboundAppendAttempt,
    entries: &[serde_json::Value],
) -> Result<(), ChannelInboundAppendFailure> {
    let mut ids = std::collections::HashSet::with_capacity(entries.len());
    for entry in entries {
        let id = entry.get("id").and_then(serde_json::Value::as_str);
        if id.is_none_or(|id| id.trim().is_empty() || !ids.insert(id))
            || entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || entry.get("role").and_then(serde_json::Value::as_str) != Some("user")
            || !is_valid_transcript_entry(entry)
        {
            return Err(channel_inbound_failure(
                ChannelInboundAppendFailureKind::InvalidBatch,
                "channel append requires unique nonempty message IDs and valid user messages",
            ));
        }
        if entry.get("sessionKey").and_then(serde_json::Value::as_str)
            != Some(attempt.session_key.as_str())
            || entry.get("sessionId").is_some_and(|value| {
                value.as_str() != Some(attempt.session_id.as_str())
            })
            || entry.get("lifecycleRevision").is_some_and(|value| {
                value.as_str() != Some(attempt.lifecycle_revision.as_str())
            })
        {
            return Err(channel_inbound_failure(
                ChannelInboundAppendFailureKind::IdentityConflict,
                "channel append entry provenance does not match its captured session",
            ));
        }
    }
    Ok(())
}

fn channel_inbound_transaction_failure(
    db: &rusqlite::Connection,
    failure: ChannelInboundAppendFailure,
    commit_attempted: bool,
    control: &ChannelInboundTransactionControl,
) -> (ChannelInboundAppendOutcome, bool) {
    // A failed COMMIT acknowledgement is uncertain even if SQLite already
    // left the transaction or a subsequent ROLLBACK succeeds.
    if commit_attempted && db.is_autocommit() {
        return (ChannelInboundAppendOutcome::Unknown(failure), false);
    }
    let rolled_back = control.rollback(db).is_ok();
    let clean = db.is_autocommit();
    if !rolled_back || !clean {
        return (ChannelInboundAppendOutcome::Unknown(channel_inbound_failure(
            ChannelInboundAppendFailureKind::RollbackUnconfirmed,
            format!("channel append rollback was not confirmed after {:?}", failure.kind),
        )), !clean);
    }
    if commit_attempted {
        (ChannelInboundAppendOutcome::Unknown(failure), false)
    } else {
        (ChannelInboundAppendOutcome::NotWritten(failure), false)
    }
}

fn append_channel_inbound_transaction(
    db: &rusqlite::Connection,
    agent_id: &str,
    attempt: &mut Option<ChannelInboundAppendAttempt>,
    prepare: impl FnOnce(
        &SessionWorkAdmissionRecord,
        &[serde_json::Value],
    ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure>,
    control: &ChannelInboundTransactionControl,
) -> (ChannelInboundAppendOutcome, bool) {
    if !db.is_autocommit() {
        return (ChannelInboundAppendOutcome::Unknown(channel_inbound_failure(
            ChannelInboundAppendFailureKind::RollbackUnconfirmed,
            "channel append owner already has an unsettled transaction",
        )), true);
    }
    if let Err(error) = db.execute_batch("BEGIN IMMEDIATE") {
        let failure = channel_inbound_storage_failure(error);
        return if db.is_autocommit() {
            (ChannelInboundAppendOutcome::NotWritten(failure), false)
        } else {
            channel_inbound_transaction_failure(db, failure, false, control)
        };
    }
    let inserted = (|| -> Result<usize, ChannelInboundAppendFailure> {
        let admission = read_channel_inbound_admission(db, agent_id, attempt.as_ref())?;
        let retained = attempt.get_or_insert_with(|| ChannelInboundAppendAttempt {
            session_key: admission.session_key.clone(),
            session_id: admission.session_id.clone(),
            lifecycle_revision: admission.lifecycle_revision.clone(),
            entries: None,
        });
        if retained.entries.is_none() {
            let history = read_channel_inbound_history(db)?;
            retained.entries = Some(prepare(&admission, &history)?);
        }
        let entries = retained.entries.as_ref().expect("channel batch prepared");
        validate_channel_inbound_batch(retained, entries)?;
        let mut statement = db.prepare(
            "INSERT INTO transcript_entries (id, entry) VALUES (?1, ?2)",
        ).map_err(channel_inbound_storage_failure)?;
        for entry in entries {
            let id = entry["id"].as_str().expect("validated channel entry ID");
            if channel_inbound_entry_id_exists(db, id)? {
                return Err(channel_inbound_failure(
                    ChannelInboundAppendFailureKind::IdentityConflict,
                    "channel append entry ID already exists",
                ));
            }
            let inserted = statement.execute(params![id, entry.to_string()])
                .map_err(|error| {
                    if error.sqlite_error().is_some_and(|error| {
                        matches!(
                            error.extended_code,
                            rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
                                | rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                        )
                    }) {
                        channel_inbound_failure(
                            ChannelInboundAppendFailureKind::IdentityConflict,
                            "channel append entry ID conflicts with an existing row",
                        )
                    } else {
                        channel_inbound_storage_failure(error)
                    }
                })?;
            if inserted != 1 {
                return Err(channel_inbound_failure(
                    ChannelInboundAppendFailureKind::IdentityConflict,
                    "channel append did not insert every intended row",
                ));
            }
        }
        // A trigger must not change the generation or remove/replace a member
        // while still letting its INSERT report success.
        read_channel_inbound_admission(db, agent_id, Some(retained))?;
        for entry in entries {
            let id = entry["id"].as_str().expect("validated channel entry ID");
            let stored = db.query_row(GET_TRANSCRIPT_ENTRY_SQL, params![id], |row| {
                row.get::<_, String>(0)
            }).optional().map_err(channel_inbound_storage_failure)?;
            if stored.as_deref().and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .as_ref() != Some(entry)
            {
                return Err(channel_inbound_failure(
                    ChannelInboundAppendFailureKind::IdentityConflict,
                    "channel append stored row differs from its prepared entry",
                ));
            }
        }
        Ok(entries.len())
    })();
    match inserted {
        Err(failure) => channel_inbound_transaction_failure(db, failure, false, control),
        Ok(inserted_count) => match control.commit(db) {
            Ok(()) if db.is_autocommit() => (
                ChannelInboundAppendOutcome::Committed { inserted_count }, false,
            ),
            Ok(()) => channel_inbound_transaction_failure(
                db,
                channel_inbound_failure(
                    ChannelInboundAppendFailureKind::Storage,
                    "channel append commit left a live transaction",
                ),
                true,
                control,
            ),
            Err(error) => channel_inbound_transaction_failure(
                db, channel_inbound_storage_failure(error), true, control,
            ),
        },
    }
}

fn channel_inbound_entry_id_exists(
    db: &rusqlite::Connection,
    id: &str,
) -> Result<bool, ChannelInboundAppendFailure> {
    db.query_row("SELECT 1 FROM transcript_entries WHERE id = ?1", params![id], |_| Ok(()))
        .optional()
        .map(|row| row.is_some())
        .map_err(channel_inbound_storage_failure)
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
    session_key: &str,
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
        "sessionKey": session_key,
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

/// Re-seed the owner-local session admission when an agent database is cloned.
/// Cloning is a pre-admission setup operation: the copied source record belongs
/// to the source agent and must never be reused by the new agent identity.
pub fn reset_persisted_session_work_admission(
    db_path: &Path,
    busy_timeout_ms: u64,
    agent_id: &str,
) -> Result<bool, AgentDbProjectionError> {
    let agent_id = agent_id.trim();
    if agent_id.is_empty() {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "cloned session admission requires an agent identity".into(),
        ));
    }
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let record = SessionWorkAdmissionRecord {
        session_key: format!("agent:{agent_id}:main"),
        session_id: agent_id.to_string(),
        lifecycle_revision: Uuid::new_v4().to_string(),
        deletion_pending: false,
        archived_at_ms: None,
        archived_by: None,
        archive_reason: None,
        admitted_work_id: None,
        admitted_media_work_id: None,
        admitted_media_paths: Vec::new(),
        terminal_result_idempotency_key: None,
        provider_review_id: None,
        provider_review_phase: None,
        provider_review_continuation_run_id: None,
        provider_review: None,
    };
    let encoded = serde_json::to_string(&record)?;
    let changed = transaction.execute(
        SET_KV_SQL,
        params![KV_SESSION_WORK_ADMISSION, encoded],
    )? > 0;
    clear_original_turn_terminal_slots(&transaction)?;
    transaction.commit()?;
    if changed {
        bump_db_write_generation(db_path);
        notify_agent_db_listeners(
            db_path,
            AgentDbListenerChannel::Metadata(KV_SESSION_WORK_ADMISSION.into()),
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    let metadata = owner.read_metadata()?;
    if metadata
        .get("name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|current| current == name)
    {
        return Ok(false);
    }
    owner.set_metadata("name", serde_json::Value::String(name.to_string()))
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.compare_and_set_latest_root_blob_id(expected_root, next_root)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.write_kv(key, &version.to_string())
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.retire_legacy_conversation_blobs(version)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.append_transcript_entries(entries)
}

pub fn delete_persisted_transcript_entry(
    db_path: &Path,
    busy_timeout_ms: u64,
    entry_id: &str,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.delete_transcript_entry(entry_id)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.set_automation_spend_guard_state(state)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.clear_transient_state()
}

/// Remove source-agent cross-store recovery custody from a cloned database.
/// These journals belong to the source identity and must not be replayed or
/// rebound after clone admission is reseeded. The source database is untouched.
pub fn clear_persisted_clone_mutation_journals(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.clear_clone_mutation_journals()
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.set_sand_profile(profile)
}

fn write_unread_state(
    db_path: &Path,
    busy_timeout_ms: u64,
    state: &UnreadState,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.set_unread_state(state)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.set_awaiting_user_response(state)
}

pub fn set_persisted_awaiting_user_response_for_tab(
    db_path: &Path,
    busy_timeout_ms: u64,
    tab_id: &str,
    state: Option<&AwaitingUserResponse>,
    if_since_before: Option<f64>,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.set_awaiting_user_response_for_tab(tab_id, state, if_since_before)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.record_request_id(id, at, prompt, source)
}

pub fn record_persisted_episode_turn(
    db_path: &Path,
    busy_timeout_ms: u64,
    turn: &EpisodeTurn,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.record_episode_turn(turn)
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
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.update_transcript_entry(entry_id, next)
}

pub fn update_persisted_transcript_entry_if_session_admitted(
    db_path: &Path,
    busy_timeout_ms: u64,
    session_key: &str,
    entry_id: &str,
    next: &serde_json::Value,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    let session_key = session_key.trim();
    if session_key.is_empty() || entry_id.trim().is_empty() {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "session-scoped persisted transcript update requires session key and entry identity"
                .into(),
        ));
    }
    let mut db = open_projection_db(db_path, busy_timeout_ms)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    if !session_work_write_is_admitted(&transaction)? {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "session owner identity is not admitted for persisted transcript update".into(),
        ));
    }
    let raw_admission = transaction
        .query_row(GET_KV_SQL, params![KV_SESSION_WORK_ADMISSION], |row| {
            row.get::<_, String>(0)
        })
        .optional()?
        .ok_or_else(|| {
            AgentDbProjectionError::WorkAdmissionInvalid(
                "session work admission record is unavailable".into(),
            )
        })?;
    let admission: SessionWorkAdmissionRecord = serde_json::from_str(&raw_admission)?;
    if !session_admission_identity_is_well_formed(&admission)
        || admission.session_key != session_key
        || admission.archived_at_ms.is_some()
        || admission.deletion_pending
    {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "session is not admitted for persisted transcript update".into(),
        ));
    }
    let raw_entry = transaction
        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    let Some(raw_entry) = raw_entry else {
        transaction.commit()?;
        return Ok(None);
    };
    let entry = parse_transcript_entry(&raw_entry).ok_or_else(|| {
        AgentDbProjectionError::WorkAdmissionInvalid(
            "target transcript entry is not valid JSON".into(),
        )
    })?;
    if entry
        .get("sessionKey")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        != Some(session_key)
    {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "target transcript entry provenance does not match admitted session".into(),
        ));
    }
    if transaction.execute(
        UPDATE_TRANSCRIPT_ENTRY_SQL,
        params![next.to_string(), entry_id],
    )? != 1
    {
        transaction.commit()?;
        return Ok(None);
    }
    transaction.commit()?;
    bump_db_write_generation(db_path);
    publish_entries_upserted(db_path, vec![next.clone()]);
    Ok(Some(next.clone()))
}

pub fn clear_persisted_conversation(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.clear_conversation()
}

fn write_persisted_kv(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
    value: &str,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.write_kv(key, value)
}

fn delete_persisted_kv(
    db_path: &Path,
    busy_timeout_ms: u64,
    key: &str,
) -> Result<bool, AgentDbProjectionError> {
    let mut options = SandAgentDbOptions::default();
    options.recovery.busy_timeout_ms = busy_timeout_ms;
    let owner = SandAgentDb::open_with_options_and_seed(db_path, options, false)?;
    owner.delete_kv(key)
}


fn read_transcript_entries_from_db(
    db: &rusqlite::Connection,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut statement = db.prepare(LIST_TRANSCRIPT_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        if let Some(entry) = parse_owner_transcript_entry(&row?) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn validate_compaction_retained_replay(
    db: &rusqlite::Connection,
    compaction: &serde_json::Value,
) -> Result<(), AgentDbProjectionError> {
    let mut replay = vec![compaction.clone()];
    if let Some(first_kept) = compaction_first_kept_entry_id(compaction)? {
        let mut statement = db.prepare(
            "SELECT entry FROM transcript_entries WHERE seq >= \
             (SELECT seq FROM transcript_entries WHERE id = ?1) ORDER BY seq ASC",
        )?;
        let rows = statement.query_map(params![first_kept], |row| row.get::<_, String>(0))?;
        for row in rows {
            // Read the retained rows directly: a presentation decoder must not
            // hide an unsupported or malformed candidate before this guard.
            let entry: serde_json::Value = serde_json::from_str(&row?).map_err(|error| {
                AgentDbProjectionError::CompactionInvalid(format!(
                    "retained transcript is not valid JSON: {error}"
                ))
            })?;
            if matches!(
                entry.get("kind").and_then(serde_json::Value::as_str),
                Some("compaction" | "background-result-receipt" | "outgoing-media-settlement")
            ) {
                continue;
            }
            replay.push(entry);
        }
    }
    super::typed_replay::project_typed_replay_entries(&replay)
        .map(|_| ())
        .map_err(|error| AgentDbProjectionError::CompactionInvalid(format!(
            "retained transcript cannot be replayed: {error}"
        )))
}

fn compaction_first_kept_entry_id(
    entry: &serde_json::Value,
) -> Result<Option<&str>, AgentDbProjectionError> {
    let has_identity = ["id", "sourceTurnId", "sourceLifecycleRevision", "lifecycleRevision"]
        .iter()
        .all(|key| {
            entry
                .get(*key)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        });
    let valid_summary = entry
        .get("summary")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|summary| {
            !summary.trim().is_empty() && summary.chars().count() <= MAX_COMPACTION_SUMMARY_CHARS
        });
    if !has_identity
        || !valid_summary
        || entry.get("tokensBefore").and_then(serde_json::Value::as_u64).is_none()
    {
        return Err(AgentDbProjectionError::CompactionInvalid(
            "compaction entry is missing or has invalid replay fields".into(),
        ));
    }
    match entry.get("firstKeptEntryId") {
        Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(id)) if !id.trim().is_empty() => Ok(Some(id)),
        _ => Err(AgentDbProjectionError::CompactionInvalid(
            "compaction entry has an invalid retained-entry boundary".into(),
        )),
    }
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
            == Some(commit.summary.trim())
        && existing
            .get("tokensBefore")
            .and_then(serde_json::Value::as_u64)
            == Some(commit.tokens_before)
        && existing.get("details") == commit.details.as_ref()
        && existing.get("sourceTurnId").and_then(serde_json::Value::as_str)
            == Some(commit.source_turn_id.trim())
        && existing.get("sourceLifecycleRevision").and_then(serde_json::Value::as_str)
            == Some(commit.expected_lifecycle_revision.trim())
}

fn read_transcript_entry_from_db(
    db: &rusqlite::Connection,
    entry_id: &str,
) -> Result<Option<serde_json::Value>, AgentDbProjectionError> {
    let raw = db
        .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![entry_id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let entry = parse_owner_transcript_entry(&raw);
    // A malformed receipt is an unknown recovery outcome, not evidence that
    // the associated transcript/media work never committed.
    if (entry_id.starts_with("background-result:")
        || entry_id.starts_with("outgoing-media-settlement:"))
        && entry
            .as_ref()
            .and_then(|entry| entry.get("id"))
            .and_then(serde_json::Value::as_str)
            != Some(entry_id)
    {
        return Err(AgentDbProjectionError::WorkAdmissionInvalid(
            "stored session receipt is malformed or has a mismatched identity".into(),
        ));
    }
    Ok(entry)
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

/// Check owner-local metadata writes at the same SQLite transaction edge as
/// the write. Legacy stores without an admission record retain compatibility;
/// malformed, archived, or deleting records fail closed.
fn session_work_write_is_admitted(
    db: &rusqlite::Connection,
) -> Result<bool, rusqlite::Error> {
    let Some(raw) = read_kv(db, KV_SESSION_WORK_ADMISSION)? else {
        return Ok(true);
    };
    let Ok(admission) = serde_json::from_str::<SessionWorkAdmissionRecord>(&raw) else {
        return Ok(false);
    };
    if !session_admission_identity_is_bound(db, &admission)? {
        return Ok(false);
    }
    Ok(!admission.deletion_pending && admission.archived_at_ms.is_none())
}

fn session_admission_identity_is_bound(
    db: &rusqlite::Connection,
    admission: &SessionWorkAdmissionRecord,
) -> Result<bool, rusqlite::Error> {
    if !session_admission_identity_is_well_formed(admission) {
        return Ok(false);
    }
    let Some(raw_metadata) = read_kv(db, "metadata")? else {
        return Ok(false);
    };
    let Ok(metadata_bytes) = decode_hex(&raw_metadata) else {
        return Ok(false);
    };
    let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&metadata_bytes) else {
        return Ok(false);
    };
    let Some(agent_id) = metadata
        .get("agentId")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(false);
    };
    let expected_session_key = format!("agent:{agent_id}:main");
    Ok(admission.session_id == agent_id && admission.session_key == expected_session_key)
}

fn session_admission_identity_is_well_formed(
    admission: &SessionWorkAdmissionRecord,
) -> bool {
    !admission.session_key.trim().is_empty()
        && !admission.session_id.trim().is_empty()
        && !admission.lifecycle_revision.trim().is_empty()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(raw: &str) -> Result<Vec<u8>, String> {
    let clean = raw.trim();
    if !clean.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("hex string contains non-hexadecimal characters".into());
    }
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

#[cfg(test)]
mod original_turn_terminal_tests {
    use super::*;
    use serde_json::{Value, json};
    use crate::extensions::session::agent_db_recovery::{
        AUTOMATION_CUSTODY_UNAVAILABLE_KV, automation_custody_recovery_marker_path,
    };
    use crate::storage::store_db::get_sand_agent_db_write_generation;

    // These are neutral storage oracles. The fixture deliberately does not
    // manufacture Runner's opaque proof or claim canonical/custody acceptance.
    // Runner's real checkpoint-sink tests own that positive qualification.
    struct Fixture {
        root: PathBuf,
        owner: SandAgentDb,
        original: OriginalTurnReference,
        root_id: Vec<u8>,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fabushi-original-terminal-{label}-{}", Uuid::new_v4(),
            ));
            let agent_id = format!("agent-{}", Uuid::new_v4());
            let directory = root.join(&agent_id);
            std::fs::create_dir_all(&directory).unwrap();
            let owner = SandAgentDb::open(directory.join("store.db"), 0).unwrap();
            let admission: SessionWorkAdmissionRecord = serde_json::from_value(json!({
                "session_key": format!("agent:{agent_id}:main"),
                "session_id": agent_id,
                "lifecycle_revision": owner.read_metadata().unwrap()["lifecycleRevision"],
            })).unwrap();
            assert!(owner.compare_and_set_session_work_admission(None, &admission).unwrap());
            let original = OriginalTurnReference {
                version: 1,
                scope: OriginalTurnScope::NativeOrdinaryUserV1,
                agent_id: admission.session_id.clone(),
                session_key: admission.session_key.clone(),
                session_id: admission.session_id.clone(),
                original_lifecycle_revision: admission.lifecycle_revision,
                stream_id: "original-stream".into(),
                input_digest: "a".repeat(64),
                client_nonce: Some("original-nonce".into()),
                user_message_id: "original-user".into(),
                echo_entry_id: "original-user".into(),
                request_source: None,
                provider: "fabushi".into(),
                model_id: "resolved-native-model".into(),
            };
            assert!(owner.append_transcript_entry(&json!({
                "id": original.user_message_id, "kind": "message", "role": "user",
                "content": "original durable user text", "isStreaming": false,
                "sessionKey": original.session_key, "clientNonce": original.client_nonce,
            })).unwrap());
            let root_id = vec![1; 32];
            assert!(owner.commit_checkpoint_root_and_confirm_user_message(
                &[], &root_id, &original.user_message_id,
            ).unwrap().is_some());
            Self { root, owner, original, root_id }
        }

        fn raw_db(&self) -> rusqlite::Connection {
            rusqlite::Connection::open(self.owner.db_path()).unwrap()
        }

        fn prepare(&self) -> OriginalTurnTerminalFence {
            self.owner.prepare_original_turn_terminal_fence(&self.original, &self.root_id).unwrap()
        }

        fn persist(&self) -> OriginalTurnTerminalRecord {
            let fence = self.prepare();
            let record = record_for(&fence);
            assert_eq!(self.owner.commit_original_turn_terminal_record(&fence, &record).unwrap(),
                OriginalTurnReceiptCommit::Inserted);
            record
        }

        fn slot(&self) -> Option<String> {
            read_kv(&self.raw_db(), ORIGINAL_TURN_TERMINAL_KEY).unwrap()
        }

        fn rows(&self) -> (Vec<(String, String)>, Vec<(i64, String, String)>) {
            let db = self.raw_db();
            let mut statement = db.prepare("SELECT key, value FROM kv ORDER BY key").unwrap();
            let kv = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            let mut statement = db.prepare(
                "SELECT seq, id, entry FROM transcript_entries ORDER BY seq",
            ).unwrap();
            let entries = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            (kv, entries)
        }

        fn generation(&self) -> u64 {
            get_sand_agent_db_write_generation(self.owner.db_path())
        }

        fn next_stream(&mut self, ordinal: u8) -> OriginalTurnTerminalFence {
            let next_root = vec![ordinal; 32];
            assert!(self.owner.commit_checkpoint_root_and_confirm_user_message(
                &self.root_id, &next_root, &self.original.user_message_id,
            ).unwrap().is_some());
            self.root_id = next_root;
            self.original.stream_id = format!("stream-{ordinal}");
            self.prepare()
        }

        fn compact_original(&self) -> Value {
            let commit = CompactionCommit {
                entry_id: format!("compaction:{}:{}", self.original.agent_id, self.original.stream_id),
                expected_transcript_tail_id: self.original.user_message_id.clone(),
                first_kept_entry_id: Some(self.original.user_message_id.clone()),
                summary: "Earlier context was summarized for this original turn.".into(),
                tokens_before: 100,
                details: None,
                source_turn_id: self.original.stream_id.clone(),
                expected_lifecycle_revision: self.original.original_lifecycle_revision.clone(),
            };
            match self.owner.commit_compaction(&commit).unwrap() {
                CompactionCommitOutcome::Committed(entry) => entry,
                outcome => panic!("actual compaction refused: {outcome:?}"),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            self.owner.close(false);
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn record_for(fence: &OriginalTurnTerminalFence) -> OriginalTurnTerminalRecord {
        OriginalTurnTerminalRecord {
            version: 1,
            original: fence.original.clone(),
            terminal: StoredCompletedTerminal::Completed,
            final_result_sha256: [7; 32],
            sent_message_count: 0,
            reacted: false,
            custody_json: "{}".into(),
            canonical: vec![CanonicalTurnCommitRecord {
                ordinal: 0,
                request_id: fence.original.stream_id.clone(),
                message_id: fence.original.user_message_id.clone(),
                prior_root_blob_id: None,
                root_blob_id: fence.final_root_blob_id.clone(),
                user_blob_id: vec![2; 32],
                turn_blob_id: vec![3; 32],
                step_blob_ids: vec![vec![4; 32]],
                assistant_result_sha256: [7; 32],
            }],
            settlement_lifecycle_revision: fence.settlement_lifecycle_revision.clone(),
            compaction: fence.compaction.clone(),
        }
    }

    #[test]
    fn real_checkpoint_revision_difference_insert_duplicate_and_reopen() {
        let fixture = Fixture::new("reopen");
        let metadata = fixture.owner.read_metadata().unwrap();
        assert_ne!(metadata["lifecycleRevision"].as_str().unwrap(),
            fixture.original.original_lifecycle_revision);
        let before_insert = fixture.prepare();
        assert_eq!(before_insert.original_user_content, "original durable user text");
        let record = fixture.persist();
        let raw = fixture.slot().unwrap();
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&before_insert, &record),
            Err(AgentDbProjectionError::OriginalTurnStale(_))));
        let current = fixture.prepare();
        let generation = fixture.generation();
        assert_eq!(fixture.owner.commit_original_turn_terminal_record(&current, &record).unwrap(),
            OriginalTurnReceiptCommit::AlreadyCommitted);
        assert_eq!(fixture.generation(), generation);
        assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
        fixture.owner.close(true);
        let reopened = SandAgentDb::open_existing_agent_without_recovery(fixture.owner.db_path(), 0).unwrap();
        // Strict opening changes only that open, not later ordinary APIs on
        // the shared cached owner.
        assert!(reopened.options.recovery.recover_on_corruption);
        assert!(!reopened.options.recovery.reject_if_corrupt);
        let (stored, fence) = reopened.read_original_turn_terminal_candidate_checked(&fixture.original)
            .unwrap().unwrap();
        assert_eq!(stored, record);
        reopened.recheck_original_turn_terminal_fence(&fence).unwrap();
        assert_eq!(read_kv(&fixture.raw_db(), ORIGINAL_TURN_TERMINAL_KEY).unwrap(), Some(raw));
        reopened.close(false);
    }

    #[test]
    fn same_stream_conflict_precedes_a_second_changed_raw_fence() {
        let fixture = Fixture::new("conflict");
        let stored = fixture.persist();
        let raw = fixture.slot().unwrap();
        for field in ["digest", "model", "result", "custody", "delivery"] {
            let mut original = fixture.original.clone();
            if field == "digest" { original.input_digest = "b".repeat(64); }
            if field == "model" { original.model_id = "other-resolved-model".into(); }
            let fence = fixture.owner.prepare_original_turn_terminal_fence(&original, &fixture.root_id).unwrap();
            let mut changed = record_for(&fence);
            match field {
                "result" => changed.final_result_sha256 = [8; 32],
                "custody" => changed.custody_json = "{\"different\":true}".into(),
                "delivery" => changed.sent_message_count = 1,
                _ => {}
            }
            assert_ne!(changed, stored);
            assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&fence, &changed),
                Err(AgentDbProjectionError::OriginalTurnConflict(_))), "{field}");
            assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
        }
        let fence = fixture.prepare();
        let mut changed = stored;
        changed.reacted = true;
        fixture.raw_db().execute("UPDATE kv SET value = value || '20' WHERE key = 'metadata'", []).unwrap();
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&fence, &changed),
            Err(AgentDbProjectionError::OriginalTurnConflict(_))));
        assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
    }

    #[test]
    fn different_stream_requires_a_new_current_root_and_keeps_one_slot() {
        let mut fixture = Fixture::new("bounded-replacement");
        let first = fixture.original.clone();
        fixture.persist();
        let old_raw = fixture.slot();
        fixture.original.stream_id = "same-root-foreign-stream".into();
        let same_root = fixture.prepare();
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&same_root, &record_for(&same_root)),
            Err(AgentDbProjectionError::OriginalTurnStale(_))));
        assert_eq!(fixture.slot(), old_raw);
        for ordinal in 2..=40 {
            let fence = fixture.next_stream(ordinal);
            assert_eq!(fixture.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)).unwrap(),
                OriginalTurnReceiptCommit::Replaced);
            let count: i64 = fixture.raw_db().query_row(
                "SELECT count(*) FROM kv WHERE substr(key, 1, ?1) = ?2",
                params![ORIGINAL_TURN_TERMINAL_PREFIX.len() as i64, ORIGINAL_TURN_TERMINAL_PREFIX],
                |row| row.get(0),
            ).unwrap();
            assert_eq!(count, 1);
            assert!(fixture.slot().unwrap().len() <= ORIGINAL_TURN_TERMINAL_MAX_BYTES);
        }
        fixture.owner.close(true);
        let reopened = SandAgentDb::open_existing_agent_without_recovery(fixture.owner.db_path(), 0).unwrap();
        assert!(reopened.read_original_turn_terminal_candidate_checked(&first).unwrap().is_none());
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).unwrap().is_some());
        reopened.close(false);
    }

    #[test]
    fn generic_kv_mutators_refuse_the_whole_reserved_prefix() {
        let fixture = Fixture::new("reserved-prefix");
        fixture.persist();
        let before = fixture.rows();
        for key in [ORIGINAL_TURN_TERMINAL_KEY.to_string(),
            format!("{ORIGINAL_TURN_TERMINAL_PREFIX}another-stream"), ORIGINAL_TURN_TERMINAL_PREFIX.into()]
        {
            assert!(matches!(fixture.owner.write_kv(&key, "replacement"),
                Err(AgentDbProjectionError::OriginalTurnInvalid(_))));
            assert!(matches!(fixture.owner.write_session_kv_if_generation(
                &key, "replacement", &fixture.original.session_id, &fixture.original.original_lifecycle_revision,
            ), Err(AgentDbProjectionError::OriginalTurnInvalid(_))));
            assert!(matches!(fixture.owner.delete_kv(&key),
                Err(AgentDbProjectionError::OriginalTurnInvalid(_))));
        }
        assert_eq!(fixture.rows(), before);
        assert!(fixture.owner.write_kv("original-routed-terminal-unrelated", "allowed").unwrap());
        assert!(fixture.owner.delete_kv("original-routed-terminal-unrelated").unwrap());
    }

    #[test]
    fn encoded_utf8_capacity_and_commit_count_are_strict_without_truncation() {
        let fixture = Fixture::new("capacity");
        let fence = fixture.prepare();
        let mut record = record_for(&fence);
        record.custody_json.clear();
        let overhead = serde_json::to_string(&record).unwrap().len();
        record.custody_json = "x".repeat(ORIGINAL_TURN_TERMINAL_MAX_BYTES - overhead);
        assert_eq!(serde_json::to_string(&record).unwrap().len(), ORIGINAL_TURN_TERMINAL_MAX_BYTES);
        assert_eq!(fixture.owner.commit_original_turn_terminal_record(&fence, &record).unwrap(),
            OriginalTurnReceiptCommit::Inserted);
        let raw = fixture.slot().unwrap();
        let fence = fixture.prepare();
        record.custody_json.push('é');
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&fence, &record),
            Err(AgentDbProjectionError::OriginalTurnInvalid(_))));
        assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
        // SQLite character length is below the cap; the persisted byte length
        // is above it. Refuse before deserializing this input.
        fixture.raw_db().execute(SET_KV_SQL,
            params![ORIGINAL_TURN_TERMINAL_KEY, "é".repeat(40_000)],
        ).unwrap();
        assert!(matches!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original),
            Err(AgentDbProjectionError::OriginalTurnInvalid(_))));

        let counts = Fixture::new("commit-count");
        let fence = counts.prepare();
        let mut record = record_for(&fence);
        let template = record.canonical[0].clone();
        record.canonical.clear();
        for ordinal in 0..ORIGINAL_TURN_MAX_COMMITS {
            let mut commit = template.clone();
            commit.ordinal = ordinal as u32;
            commit.message_id = if ordinal == 0 { counts.original.user_message_id.clone() }
                else { counts.original.stream_id.clone() };
            commit.prior_root_blob_id = record.canonical.last().map(|prior| prior.root_blob_id.clone());
            commit.root_blob_id = if ordinal + 1 == ORIGINAL_TURN_MAX_COMMITS { counts.root_id.clone() }
                else { vec![20 + ordinal as u8; 32] };
            record.canonical.push(commit);
        }
        assert_eq!(counts.owner.commit_original_turn_terminal_record(&fence, &record).unwrap(),
            OriginalTurnReceiptCommit::Inserted);
        let raw = counts.slot();
        let fence = counts.prepare();
        let mut excess = record.canonical.last().unwrap().clone();
        excess.ordinal += 1;
        excess.prior_root_blob_id = Some(excess.root_blob_id.clone());
        record.canonical.push(excess);
        assert!(matches!(counts.owner.commit_original_turn_terminal_record(&fence, &record),
            Err(AgentDbProjectionError::OriginalTurnInvalid(_))));
        assert_eq!(counts.slot(), raw);
    }

    #[test]
    fn only_the_actual_same_stream_compaction_successor_is_eligible() {
        let fixture = Fixture::new("compaction");
        let entry = fixture.compact_original();
        let fence = fixture.prepare();
        assert_ne!(fence.settlement_lifecycle_revision, fixture.original.original_lifecycle_revision);
        assert_eq!(fence.compaction.as_ref().unwrap().entry_id, entry["id"].as_str().unwrap());
        let record = fixture.persist();
        fixture.owner.close(true);
        let reopened = SandAgentDb::open_existing_agent_without_recovery(fixture.owner.db_path(), 0).unwrap();
        assert_eq!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original)
            .unwrap().unwrap().0, record);
        for field in ["sourceTurnId", "sourceLifecycleRevision", "lifecycleRevision", "summary"] {
            let mut changed = entry.clone();
            if field == "summary" { changed.as_object_mut().unwrap().remove(field); }
            else { changed[field] = json!("unrelated"); }
            fixture.raw_db().execute(UPDATE_TRANSCRIPT_ENTRY_SQL,
                params![changed.to_string(), entry["id"].as_str().unwrap()],
            ).unwrap();
            assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err(), "{field}");
        }
        fixture.raw_db().execute(UPDATE_TRANSCRIPT_ENTRY_SQL,
            params![entry.to_string(), entry["id"].as_str().unwrap()],
        ).unwrap();
        let mut admission = reopened.read_session_work_admission().unwrap().unwrap();
        let prior = admission.lifecycle_revision.clone();
        admission.lifecycle_revision = "unrelated-successor".into();
        assert!(reopened.compare_and_set_session_work_admission(Some(&prior), &admission).unwrap());
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        reopened.close(false);
    }

    #[test]
    fn every_raw_image_is_rechecked_even_when_decoded_values_are_unchanged() {
        for field in ["metadata", "admission", "user", "compaction", "slot"] {
            let fixture = Fixture::new(field);
            let record = fixture.persist();
            let fence = fixture.prepare();
            let db = fixture.raw_db();
            match field {
                "metadata" => { db.execute("UPDATE kv SET value = value || '20' WHERE key = 'metadata'", []).unwrap(); }
                "admission" => { db.execute("UPDATE kv SET value = value || ' ' WHERE key = ?1",
                    params![KV_SESSION_WORK_ADMISSION]).unwrap(); }
                "user" => { db.execute("UPDATE transcript_entries SET entry = entry || ' ' WHERE id = ?1",
                    params![fixture.original.user_message_id]).unwrap(); }
                "compaction" => { db.execute(INSERT_TRANSCRIPT_ENTRY_SQL,
                    params![format!("compaction:{}:{}", fixture.original.agent_id, fixture.original.stream_id), "{}"],
                ).unwrap(); }
                "slot" => { db.execute("UPDATE kv SET value = value || ' ' WHERE key = ?1",
                    params![ORIGINAL_TURN_TERMINAL_KEY]).unwrap(); }
                _ => unreachable!(),
            }
            let changed = fixture.rows();
            let generation = fixture.generation();
            assert!(fixture.owner.recheck_original_turn_terminal_fence(&fence).is_err(), "{field}");
            assert!(fixture.owner.commit_original_turn_terminal_record(&fence, &record).is_err(), "{field}");
            assert_eq!(fixture.rows(), changed, "{field}");
            assert_eq!(fixture.generation(), generation, "{field}");
        }
    }

    #[test]
    fn post_insert_triggers_roll_back_slot_and_changed_owner_rows() {
        for field in ["metadata", "admission", "user", "compaction", "slot", "delete-slot", "memory", "profile", "cron", "recovery"] {
            let fixture = Fixture::new(field);
            let fence = fixture.prepare();
            let body = match field {
                "metadata" => "UPDATE kv SET value = value || '20' WHERE key = 'metadata';".into(),
                "admission" => "UPDATE kv SET value = value || ' ' WHERE key = 'sessionWorkAdmission';".into(),
                "user" => "UPDATE transcript_entries SET entry = entry || ' ' WHERE id = 'original-user';".into(),
                "compaction" => format!(
                    "INSERT INTO transcript_entries (id, entry) VALUES ('compaction:{}:{}', '{{}}');",
                    fixture.original.agent_id, fixture.original.stream_id,
                ),
                "slot" => "UPDATE kv SET value = value || ' ' WHERE key = 'original-routed-terminal:current';".into(),
                "delete-slot" => "DELETE FROM kv WHERE key = 'original-routed-terminal:current';".into(),
                "memory" => "INSERT INTO kv (key, value) VALUES ('pendingExternalMemoryMutation', 'retained');".into(),
                "profile" => "INSERT INTO kv (key, value) VALUES ('pendingExternalProfileMutation', 'retained');".into(),
                "cron" => format!(
                    "INSERT INTO cron_session_runs (session_key, automation_id, run_id, session_id, \
                     lifecycle_revision, updated_at_ms, state) \
                     VALUES ('cron', 'automation', 'run', '{}', '{}', 0, 'running');",
                    fixture.original.session_id, fixture.original.original_lifecycle_revision,
                ),
                "recovery" => format!("INSERT INTO kv (key, value) VALUES ('{AUTOMATION_CUSTODY_UNAVAILABLE_KV}', '');"),
                _ => unreachable!(),
            };
            fixture.raw_db().execute_batch(&format!(
                "CREATE TRIGGER terminal_after_insert AFTER INSERT ON kv \
                 WHEN NEW.key = 'original-routed-terminal:current' BEGIN {body} END;",
            )).unwrap();
            let before = fixture.rows();
            let generation = fixture.generation();
            assert!(fixture.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)).is_err(), "{field}");
            assert_eq!(fixture.rows(), before, "all trigger effects roll back: {field}");
            assert!(fixture.slot().is_none());
            assert_eq!(fixture.generation(), generation);
            assert_eq!(fixture.raw_db().query_row("SELECT count(*) FROM cron_session_runs", [],
                |row| row.get::<_, i64>(0)).unwrap(), 0);
            assert!(fixture.owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
        }
    }

    #[test]
    fn post_replace_trigger_preserves_the_previous_slot_atomically() {
        let mut fixture = Fixture::new("replace-trigger");
        fixture.persist();
        let previous = fixture.slot();
        let fence = fixture.next_stream(2);
        fixture.raw_db().execute_batch(
            "CREATE TRIGGER terminal_after_update AFTER UPDATE ON kv
             WHEN NEW.key = 'original-routed-terminal:current'
             BEGIN UPDATE kv SET value = value || ' ' WHERE key = 'sessionWorkAdmission'; END;",
        ).unwrap();
        let before = fixture.rows();
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)),
            Err(AgentDbProjectionError::OriginalTurnStale(_))));
        assert_eq!(fixture.rows(), before);
        assert_eq!(fixture.slot(), previous);
    }

    #[test]
    fn real_deferred_commit_failure_is_unavailable_even_after_clean_rollback() {
        let fixture = Fixture::new("commit-failure");
        let fence = fixture.prepare();
        fixture.owner.connection.lock().unwrap().as_ref().unwrap().execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE terminal_parent (id INTEGER PRIMARY KEY);
             CREATE TABLE terminal_child (
                 parent_id INTEGER REFERENCES terminal_parent(id) DEFERRABLE INITIALLY DEFERRED
             );
             CREATE TRIGGER terminal_deferred_commit AFTER INSERT ON kv
             WHEN NEW.key = 'original-routed-terminal:current'
             BEGIN INSERT INTO terminal_child (parent_id) VALUES (7); END;",
        ).unwrap();
        let before = fixture.rows();
        let generation = fixture.generation();
        assert!(matches!(fixture.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)),
            Err(AgentDbProjectionError::OriginalTurnUnavailable(message)) if message.contains("COMMIT outcome is unknown")));
        assert_eq!(fixture.rows(), before);
        assert_eq!(fixture.generation(), generation);
        assert!(fixture.owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
        assert_eq!(fixture.raw_db().query_row("SELECT count(*) FROM terminal_child", [],
            |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert!(!fixture.owner.is_closed());
    }

    #[test]
    fn closed_busy_poisoned_and_unsettled_owners_never_report_absence() {
        let closed = Fixture::new("closed");
        let fence = closed.prepare();
        closed.owner.close(false);
        assert!(closed.owner.read_original_turn_terminal_candidate_checked(&closed.original).is_err());
        assert!(closed.owner.prepare_original_turn_terminal_fence(&closed.original, &closed.root_id).is_err());
        assert!(closed.owner.recheck_original_turn_terminal_fence(&fence).is_err());
        assert!(closed.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)).is_err());

        let busy = Fixture::new("busy");
        let fence = busy.prepare();
        let writer = busy.raw_db();
        writer.execute_batch("BEGIN IMMEDIATE").unwrap();
        assert!(busy.owner.read_original_turn_terminal_candidate_checked(&busy.original).is_err());
        assert!(busy.owner.prepare_original_turn_terminal_fence(&busy.original, &busy.root_id).is_err());
        assert!(busy.owner.recheck_original_turn_terminal_fence(&fence).is_err());
        assert!(busy.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)).is_err());
        writer.execute_batch("ROLLBACK").unwrap();
        assert!(busy.owner.read_original_turn_terminal_candidate_checked(&busy.original).unwrap().is_none());

        let poisoned = Fixture::new("poisoned");
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = poisoned.owner.connection.lock().unwrap();
            panic!("poison only this fixture owner");
        })).is_err());
        assert!(matches!(poisoned.owner.read_original_turn_terminal_candidate_checked(&poisoned.original),
            Err(AgentDbProjectionError::OwnerPoisoned)));
        poisoned.owner.connection.clear_poison();

        let unsettled = Fixture::new("unsettled");
        unsettled.owner.connection.lock().unwrap().as_ref().unwrap().execute_batch("BEGIN IMMEDIATE").unwrap();
        assert!(unsettled.owner.read_original_turn_terminal_candidate_checked(&unsettled.original).is_err());
        assert!(unsettled.owner.is_closed());
        assert!(unsettled.owner.connection.lock().unwrap().is_none());
        assert_eq!(live_db_handle_count(unsettled.owner.db_path()), 0);
        assert!(unsettled.slot().is_none());
    }

    #[test]
    fn malformed_unsupported_and_duplicate_field_slots_are_never_absence() {
        let fixture = Fixture::new("bad-slot");
        let record = fixture.persist();
        let raw = fixture.slot().unwrap();
        let mut unsupported = serde_json::to_value(&record).unwrap();
        unsupported["version"] = json!(2);
        let mut unknown = serde_json::to_value(&record).unwrap();
        unknown["unrecognizedAuthority"] = json!(true);
        let duplicate = format!("{{\"version\":1,{}", raw.strip_prefix('{').unwrap());
        let mut foreign = fixture.original.clone();
        foreign.stream_id = "foreign-stream".into();
        for damaged in ["{".to_string(), unsupported.to_string(), unknown.to_string(), duplicate] {
            fixture.raw_db().execute(SET_KV_SQL, params![ORIGINAL_TURN_TERMINAL_KEY, damaged]).unwrap();
            assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
            assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&foreign).is_err());
            assert!(fixture.owner.prepare_original_turn_terminal_fence(&fixture.original, &fixture.root_id).is_err());
            assert_eq!(fixture.slot().as_deref(), Some(damaged.as_str()));
        }
    }

    #[test]
    fn missing_invalid_or_independently_owned_source_rows_are_refused_unchanged() {
        let mutations = [
            ("metadata", "lifecycleRevision", Value::Null),
            ("metadata", "lifecycleRevision", json!("")),
            ("metadata", "lifecycleRevision", json!("bad\ntoken")),
            ("metadata", "agentId", json!("foreign-agent")),
            ("metadata", "latestRootBlobId", json!("0202")),
            ("admission", "session_key", json!("agent:foreign:main")),
            ("admission", "session_id", json!("foreign-agent")),
            ("admission", "lifecycle_revision", json!("restored-generation")),
            ("admission", "deletion_pending", json!(true)),
            ("admission", "archived_at_ms", json!(100)),
            ("admission", "admitted_work_id", json!("independent-work")),
            ("admission", "admitted_media_work_id", json!("independent-media")),
            ("admission", "admitted_media_paths", json!(["/retained/media"])),
            ("admission", "provider_review_id", json!("review")),
            ("admission", "provider_review_phase", json!("pending")),
            ("admission", "provider_review_continuation_run_id", json!("continuation")),
            ("admission", "provider_review", json!({})),
            ("user", "id", json!("foreign-user")),
            ("user", "kind", json!("divider")),
            ("user", "role", json!("assistant")),
            ("user", "confirmed", json!(false)),
            ("user", "content", json!("")),
            ("user", "clientNonce", json!("foreign-nonce")),
            ("user", "sessionKey", json!("agent:foreign:main")),
            ("user", "hidden", json!(true)),
            ("user", "branched", json!(true)),
            ("user", "isStreaming", json!(true)),
            ("user", "fromAgent", json!("child")),
            ("user", "toAgent", json!("foreign")),
            ("user", "channel", json!("external")),
        ];
        for (source, field, replacement) in mutations {
            let fixture = Fixture::new("invalid-source");
            fixture.persist();
            let db = fixture.raw_db();
            match source {
                "metadata" => {
                    let mut metadata = fixture.owner.read_metadata().unwrap();
                    metadata[field] = replacement;
                    db.execute(SET_KV_SQL, params!["metadata", encode_hex(&serde_json::to_vec(&metadata).unwrap())]).unwrap();
                }
                "admission" => {
                    let mut admission: Value = serde_json::from_str(&read_kv(&db, KV_SESSION_WORK_ADMISSION).unwrap().unwrap()).unwrap();
                    admission[field] = replacement;
                    db.execute(SET_KV_SQL, params![KV_SESSION_WORK_ADMISSION, admission.to_string()]).unwrap();
                }
                "user" => {
                    let raw: String = db.query_row(GET_TRANSCRIPT_ENTRY_SQL,
                        params![fixture.original.user_message_id], |row| row.get(0)).unwrap();
                    let mut user: Value = serde_json::from_str(&raw).unwrap();
                    user[field] = replacement;
                    db.execute(UPDATE_TRANSCRIPT_ENTRY_SQL, params![user.to_string(), fixture.original.user_message_id]).unwrap();
                }
                _ => unreachable!(),
            }
            let before = fixture.rows();
            assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original).is_err(), "{source}.{field}");
            assert_eq!(fixture.rows(), before);
        }
        for source in ["metadata", KV_SESSION_WORK_ADMISSION, "user", KV_PENDING_MEMORY_MUTATION, KV_PENDING_PROFILE_MUTATION] {
            let fixture = Fixture::new("missing-source");
            fixture.persist();
            let db = fixture.raw_db();
            match source {
                "user" => { db.execute(DELETE_TRANSCRIPT_ENTRY_SQL, params![fixture.original.user_message_id]).unwrap(); }
                KV_PENDING_MEMORY_MUTATION | KV_PENDING_PROFILE_MUTATION => { db.execute(SET_KV_SQL, params![source, "retained malformed journal"]).unwrap(); }
                _ => { db.execute(DELETE_KV_SQL, params![source]).unwrap(); }
            }
            let before = fixture.rows();
            assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original).is_err(), "{source}");
            assert_eq!(fixture.rows(), before);
        }
    }

    #[test]
    fn recovery_guard_contention_refuses_all_terminal_phases_before_absence() {
        let fixture = Fixture::new("recovery-contention");
        let fence = fixture.prepare();
        let mut guard_path = fixture.owner.db_path().as_os_str().to_os_string();
        guard_path.push(".automation-custody.guard");
        let guard = std::fs::OpenOptions::new().read(true).write(true).open(PathBuf::from(guard_path)).unwrap();
        fs2::FileExt::try_lock_exclusive(&guard).unwrap();
        assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        assert!(fixture.owner.prepare_original_turn_terminal_fence(&fixture.original, &fixture.root_id).is_err());
        assert!(fixture.owner.recheck_original_turn_terminal_fence(&fence).is_err());
        assert!(fixture.owner.commit_original_turn_terminal_record(&fence, &record_for(&fence)).is_err());
        fs2::FileExt::unlock(&guard).unwrap();
        assert!(fixture.owner.read_original_turn_terminal_candidate_checked(&fixture.original).unwrap().is_none());
        assert!(fixture.slot().is_none());
    }

    #[test]
    fn actual_reconstruction_salvages_slot_but_never_restores_terminal_authority() {
        let fixture = Fixture::new("salvage");
        let record = fixture.persist();
        let fence = fixture.prepare();
        let raw = fixture.slot().unwrap();
        fixture.owner.close(true);
        let recovered = recover_corrupt_store_db(
            fixture.owner.db_path(), &fixture.original.agent_id,
            &DbRecoveryOptions { busy_timeout_ms: 0, ..DbRecoveryOptions::default() },
            "terminal salvage oracle",
        ).unwrap();
        assert_eq!(read_kv(&recovered, ORIGINAL_TURN_TERMINAL_KEY).unwrap().as_deref(), Some(raw.as_str()));
        assert!(read_kv(&recovered, AUTOMATION_CUSTODY_UNAVAILABLE_KV).unwrap().is_some());
        drop(recovered);
        let marker = automation_custody_recovery_marker_path(fixture.owner.db_path());
        assert!(marker.is_file());
        let reopened = SandAgentDb::open_existing_agent_without_recovery(fixture.owner.db_path(), 0).unwrap();
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        assert!(reopened.prepare_original_turn_terminal_fence(&fixture.original, &fixture.root_id).is_err());
        assert!(reopened.recheck_original_turn_terminal_fence(&fence).is_err());
        assert!(reopened.commit_original_turn_terminal_record(&fence, &record).is_err());
        assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
        // Exercise each marker independently using only this test-owned DB.
        // Neither malformed marker contents nor an absent slot proves absence.
        fixture.raw_db().execute(DELETE_KV_SQL, params![ORIGINAL_TURN_TERMINAL_KEY]).unwrap();
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        std::fs::remove_file(&marker).unwrap();
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        std::fs::write(&marker, []).unwrap();
        fixture.raw_db().execute(DELETE_KV_SQL, params![AUTOMATION_CUSTODY_UNAVAILABLE_KV]).unwrap();
        assert!(reopened.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        assert!(marker.is_file());
        reopened.close(false);
    }

    #[test]
    fn clone_resets_and_clear_remove_prefix_in_the_same_transaction() {
        for operation in ["owner-clone", "persisted-clone", "clear"] {
            let fixture = Fixture::new(operation);
            fixture.persist();
            let db = fixture.raw_db();
            db.execute(SET_KV_SQL, params![format!("{ORIGINAL_TURN_TERMINAL_PREFIX}legacy-row"), "retained"]).unwrap();
            db.execute(SET_KV_SQL, params!["original-routed-terminal-unrelated", "unrelated"]).unwrap();
            db.execute(SET_KV_SQL, params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, "retained recovery marker"]).unwrap();
            let marker = automation_custody_recovery_marker_path(fixture.owner.db_path());
            std::fs::write(&marker, b"retained recovery marker").unwrap();
            db.execute_batch(
                "CREATE TRIGGER terminal_refuse_cleanup BEFORE DELETE ON kv
                 WHEN OLD.key = 'original-routed-terminal:current'
                 BEGIN SELECT RAISE(ABORT, 'terminal cleanup refused'); END;",
            ).unwrap();
            let apply = || match operation {
                "owner-clone" => fixture.owner.reset_session_work_admission_for_clone("clone-agent"),
                "persisted-clone" => reset_persisted_session_work_admission(fixture.owner.db_path(), 0, "clone-agent"),
                "clear" => fixture.owner.clear_conversation(),
                _ => unreachable!(),
            };
            let before = fixture.rows();
            assert!(apply().is_err(), "{operation}");
            assert_eq!(fixture.rows(), before, "{operation}");
            db.execute_batch("DROP TRIGGER terminal_refuse_cleanup").unwrap();
            assert!(apply().unwrap(), "{operation}");
            assert_eq!(db.query_row(
                "SELECT count(*) FROM kv WHERE substr(key, 1, ?1) = ?2",
                params![ORIGINAL_TURN_TERMINAL_PREFIX.len() as i64, ORIGINAL_TURN_TERMINAL_PREFIX],
                |row| row.get::<_, i64>(0),
            ).unwrap(), 0);
            assert_eq!(read_kv(&db, "original-routed-terminal-unrelated").unwrap().as_deref(), Some("unrelated"));
            assert_eq!(read_kv(&db, AUTOMATION_CUSTODY_UNAVAILABLE_KV).unwrap().as_deref(), Some("retained recovery marker"));
            assert_eq!(std::fs::read(&marker).unwrap(), b"retained recovery marker");
            if operation == "clear" {
                assert!(fixture.owner.get_transcript_entries().unwrap().is_empty());
                assert!(fixture.owner.get_latest_root_blob_id().unwrap().is_empty());
            } else {
                assert_eq!(fixture.owner.read_session_work_admission().unwrap().unwrap().session_id, "clone-agent");
            }
        }
    }

    #[test]
    fn actual_agent_clone_cannot_copy_a_source_terminal_slot() {
        let fixture = Fixture::new("actual-clone");
        fixture.persist();
        let raw = fixture.slot().unwrap();
        fixture.raw_db().execute(SET_KV_SQL,
            params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, "source recovery marker"],
        ).unwrap();
        fixture.owner.close(true);
        let target = fixture.root.join("clone-agent");
        crate::agents::agent_clone::clone_agent_dir(
            fixture.owner.db_path().parent().unwrap(), &target, "clone-agent", "Clone", 0,
        ).unwrap();
        assert_eq!(fixture.slot().as_deref(), Some(raw.as_str()));
        let cloned = SandAgentDb::open_existing_agent_without_recovery(target.join("store.db"), 0).unwrap();
        assert!(cloned.read_kv(ORIGINAL_TURN_TERMINAL_KEY).unwrap().is_none());
        assert!(cloned.read_kv(AUTOMATION_CUSTODY_UNAVAILABLE_KV).unwrap().is_some());
        assert!(cloned.read_original_turn_terminal_candidate_checked(&fixture.original).is_err());
        cloned.close(false);
    }
}

#[cfg(test)]
mod channel_inbound_append_tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use crate::transcript_mutation_events::{
        subscribe_transcript_mutations, TranscriptMutationSubscription,
    };

    fn fixture(label: &str) -> (PathBuf, SandAgentDb) {
        let root = std::env::temp_dir().join(format!("channel-append-{label}-{}", Uuid::new_v4()));
        let agent_id = format!("agent-{label}-{}", Uuid::new_v4());
        let directory = root.join(&agent_id);
        std::fs::create_dir_all(&directory).unwrap();
        let owner = SandAgentDb::open(directory.join("store.db"), 50).unwrap();
        let admission: SessionWorkAdmissionRecord = serde_json::from_value(serde_json::json!({
            "session_key": format!("agent:{agent_id}:main"),
            "session_id": agent_id,
            "lifecycle_revision": "generation-1",
        })).unwrap();
        assert!(owner.compare_and_set_session_work_admission(None, &admission).unwrap());
        (root, owner)
    }

    fn entries(
        admission: &SessionWorkAdmissionRecord,
        _: &[serde_json::Value],
    ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure> {
        Ok(["channel-first", "channel-second"].into_iter().map(|id| serde_json::json!({
            "id": id, "kind": "message", "role": "user", "content": id,
            "sessionKey": admission.session_key,
        })).collect())
    }

    fn observe(owner: &SandAgentDb) -> (Arc<AtomicUsize>, TranscriptMutationSubscription) {
        let count = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&count);
        let agent_id = owner.agent_dir_name.clone();
        let subscription = subscribe_transcript_mutations(move |mutation| {
            if mutation.get("agentId").and_then(serde_json::Value::as_str) == Some(agent_id.as_str()) {
                observed.fetch_add(1, Ordering::SeqCst);
            }
        });
        (count, subscription)
    }

    #[test]
    fn real_commit_failure_stays_unknown_after_successful_rollback() {
        let (root, owner) = fixture("commit-refused");
        owner.connection.lock().unwrap().as_ref().unwrap().execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE channel_parent (id INTEGER PRIMARY KEY);
             CREATE TABLE channel_child (
                 parent_id INTEGER REFERENCES channel_parent(id) DEFERRABLE INITIALLY DEFERRED
             );
             CREATE TRIGGER channel_deferred_commit AFTER INSERT ON transcript_entries
             BEGIN INSERT INTO channel_child (parent_id) VALUES (7); END;",
        ).unwrap();
        let (publications, subscription) = observe(&owner);
        let mut attempt = None;
        let outcome = owner.append_channel_inbound_entries(&mut attempt, entries);
        assert!(matches!(outcome, ChannelInboundAppendOutcome::Unknown(failure)
            if failure.kind == ChannelInboundAppendFailureKind::Storage));
        assert_eq!(attempt.as_ref().unwrap().entries.as_ref().unwrap().len(), 2);
        assert!(owner.get_transcript_entries().unwrap().is_empty());
        assert!(owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
        assert!(!owner.is_closed());
        assert_eq!(publications.load(Ordering::SeqCst), 0);
        // The cleanup is proven clean, but it does not downgrade the failed
        // COMMIT acknowledgement or authorize replay of this retained claim.
        subscription.unsubscribe();
        owner.close(false);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unconfirmed_rollback_retires_the_exact_connection_before_reuse() {
        let (root, owner) = fixture("rollback-unconfirmed");
        owner.connection.lock().unwrap().as_ref().unwrap().execute_batch(
            "CREATE TRIGGER channel_abort_later BEFORE INSERT ON transcript_entries
             WHEN NEW.id = 'channel-second'
             BEGIN SELECT RAISE(ABORT, 'later insert failure'); END;",
        ).unwrap();
        let (publications, subscription) = observe(&owner);
        let mut attempt = None;
        let (outcome, publication) = owner.append_channel_inbound_entries_deferred_with_control(
            &mut attempt, entries,
            &ChannelInboundTransactionControl { fail_rollback: true, ..Default::default() },
        );
        assert!(matches!(outcome, ChannelInboundAppendOutcome::Unknown(failure)
            if failure.kind == ChannelInboundAppendFailureKind::RollbackUnconfirmed));
        assert!(publication.is_none());
        assert_eq!(attempt.as_ref().unwrap().entries.as_ref().unwrap().len(), 2);
        assert!(owner.is_closed());
        assert!(owner.connection.lock().unwrap().is_none());
        assert_eq!(live_db_handle_count(owner.db_path()), 0);
        let inspection = rusqlite::Connection::open(owner.db_path()).unwrap();
        assert_eq!(inspection.query_row("SELECT count(*) FROM transcript_entries", [],
            |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert!(matches!(owner.append_channel_inbound_entries(
            &mut attempt, |_, _| panic!("retired connection cannot be reused"),
        ), ChannelInboundAppendOutcome::NotWritten(failure)
            if failure.kind == ChannelInboundAppendFailureKind::Closed));
        assert_eq!(publications.load(Ordering::SeqCst), 0);
        // Reopen observes storage only; the shipping queue still holds Unknown
        // and cannot infer replay or restart recovery from this absence.
        subscription.unsubscribe();
        drop(inspection);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn real_commit_with_lost_acknowledgement_is_never_not_written() {
        let (root, owner) = fixture("commit-ack-lost");
        let (publications, subscription) = observe(&owner);
        let mut attempt = None;
        let (outcome, publication) = owner.append_channel_inbound_entries_deferred_with_control(
            &mut attempt, entries,
            &ChannelInboundTransactionControl {
                lose_commit_acknowledgement: true,
                ..Default::default()
            },
        );
        assert!(matches!(outcome, ChannelInboundAppendOutcome::Unknown(failure)
            if failure.kind == ChannelInboundAppendFailureKind::Storage));
        assert!(publication.is_none());
        let retained = attempt.as_ref().unwrap().entries.as_ref().unwrap();
        assert_eq!(&owner.get_transcript_entries().unwrap(), retained);
        assert_eq!(publications.load(Ordering::SeqCst), 0);
        assert!(owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
        assert!(!owner.is_closed());
        subscription.unsubscribe();
        owner.close(false);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn inherited_unsettled_transaction_is_retired_without_preparation() {
        let (root, owner) = fixture("unsettled-owner");
        owner.connection.lock().unwrap().as_ref().unwrap()
            .execute_batch("BEGIN IMMEDIATE").unwrap();
        let mut attempt = None;
        assert!(matches!(owner.append_channel_inbound_entries(
            &mut attempt, |_, _| panic!("an unsettled owner cannot admit another append"),
        ), ChannelInboundAppendOutcome::Unknown(failure)
            if failure.kind == ChannelInboundAppendFailureKind::RollbackUnconfirmed));
        assert!(attempt.is_none());
        assert!(owner.is_closed());
        assert!(owner.connection.lock().unwrap().is_none());
        assert_eq!(live_db_handle_count(owner.db_path()), 0);
        let _ = std::fs::remove_dir_all(root);
    }
}

#[cfg(test)]
mod fresh_append_tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use crate::extensions::transcript::transcript_entry_ids::{
        TranscriptEntryIdKind, next_entry_id,
    };
    use crate::storage::store_db::get_sand_agent_db_write_generation;
    use crate::transcript_mutation_events::{
        TranscriptMutationSubscription, subscribe_transcript_mutations,
    };
    use serde_json::{Value, json};

    struct Fixture {
        root: PathBuf,
        owner: SandAgentDb,
        recoveries: Arc<AtomicUsize>,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let root = std::env::temp_dir().join(format!(
                "fabushi-fresh-db-{label}-{}-{suffix}", std::process::id(),
            ));
            let db_path = root.join(format!("agent-{suffix}")).join("store.db");
            let recoveries = Arc::new(AtomicUsize::new(0));
            let captured = Arc::clone(&recoveries);
            let owner = SandAgentDb::open_with_options(&db_path, SandAgentDbOptions {
                recovery: DbRecoveryOptions {
                    busy_timeout_ms: 0,
                    on_corruption_recovered: Some(Arc::new(move |_| {
                        captured.fetch_add(1, Ordering::SeqCst);
                    })),
                    ..DbRecoveryOptions::default()
                },
                ..SandAgentDbOptions::default()
            }).unwrap();
            Self { root, owner, recoveries }
        }

        fn raw_db(&self) -> rusqlite::Connection {
            rusqlite::Connection::open(self.owner.db_path()).unwrap()
        }

        fn rows(&self) -> Vec<(i64, String, String)> {
            let db = self.raw_db();
            let mut statement = db.prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq").unwrap();
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
            rows
        }

        fn generation(&self) -> u64 {
            get_sand_agent_db_write_generation(self.owner.db_path())
        }

        fn bind_admission(&self) -> SessionWorkAdmissionRecord {
            let metadata = self.owner.read_metadata().unwrap();
            let agent_id = metadata["agentId"].as_str().unwrap();
            let admission: SessionWorkAdmissionRecord = serde_json::from_value(json!({
                "session_key": format!("agent:{agent_id}:main"),
                "session_id": agent_id,
                "lifecycle_revision": metadata["lifecycleRevision"],
            })).unwrap();
            assert!(self.owner.compare_and_set_session_work_admission(None, &admission).unwrap());
            admission
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            self.owner.close(false);
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    struct Publications {
        values: Arc<Mutex<Vec<Value>>>,
        subscription: Option<TranscriptMutationSubscription>,
    }

    impl Publications {
        fn new(owner: &SandAgentDb) -> Self {
            let values = Arc::new(Mutex::new(Vec::new()));
            let captured = Arc::clone(&values);
            let agent = agent_id_for_db_path(owner.db_path());
            let subscription = subscribe_transcript_mutations(move |mutation| {
                if mutation.get("agentId").and_then(Value::as_str) == Some(agent.as_str()) {
                    captured.lock().unwrap().push(Value::Object(mutation.clone()));
                }
            });
            Self { values, subscription: Some(subscription) }
        }

        fn values(&self) -> Vec<Value> {
            self.values.lock().unwrap().clone()
        }
    }

    impl Drop for Publications {
        fn drop(&mut self) {
            self.subscription.take().unwrap().unsubscribe();
        }
    }

    fn message(id: &str, content: &str) -> Value {
        json!({"id": id, "kind": "message", "role": "user", "content": content})
    }

    fn refuse_rollback_commands(owner: &SandAgentDb) {
        unsafe extern "C" fn authorizer(
            _user_data: *mut std::ffi::c_void,
            action: std::ffi::c_int,
            operation: *const std::ffi::c_char,
            _second: *const std::ffi::c_char,
            _database: *const std::ffi::c_char,
            _trigger: *const std::ffi::c_char,
        ) -> std::ffi::c_int {
            if action == rusqlite::ffi::SQLITE_TRANSACTION && !operation.is_null() {
                // SQLite owns this nullable, NUL-terminated callback argument.
                // The callback neither retains it nor reenters the connection.
                let operation = unsafe { std::ffi::CStr::from_ptr(operation) };
                if operation.to_bytes() == b"ROLLBACK" {
                    return rusqlite::ffi::SQLITE_DENY;
                }
            }
            rusqlite::ffi::SQLITE_OK
        }

        let guard = owner.connection.lock().unwrap();
        let db = guard.as_ref().unwrap();
        // The static callback needs no borrowed user data. Install it while
        // holding the actual owner's connection lock; retirement closes that
        // connection and no test retains or uses its raw handle afterward.
        let result = unsafe {
            rusqlite::ffi::sqlite3_set_authorizer(
                db.handle(), Some(authorizer), std::ptr::null_mut(),
            )
        };
        assert_eq!(result, rusqlite::ffi::SQLITE_OK);
    }

    #[test]
    fn fresh_commit_returns_value_and_defers_exact_publication_until_requested() {
        let fixture = Fixture::new("publication");
        let events = Publications::new(&fixture.owner);
        let before = fixture.generation();
        let calls = AtomicUsize::new(0);
        let context = String::from("owned producer context");
        let expected = vec![message("t0u", "one"), message("t1u", "two")];
        let committed = fixture.owner.commit_transcript_entries(None, None, |snapshot| {
            calls.fetch_add(1, Ordering::SeqCst);
            assert!(snapshot.is_empty());
            // Moving context makes this a real FnOnce, not a replayable FnMut.
            Ok((context, expected.clone()))
        }).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.owner.get_transcript_entries().unwrap(), expected);
        assert_eq!(fixture.generation(), before + 1);
        assert!(events.values().is_empty(), "commit itself must not call listeners");
        assert_eq!(committed.publish(), "owned producer context");
        assert_eq!(events.values(), vec![json!({
            "kind": "entries-upserted", "agentId": fixture.owner.agent_dir_name,
            "entries": expected,
        })]);

        let unchanged = (fixture.rows(), fixture.generation(), events.values());
        let existing = fixture.owner.commit_transcript_entries(None, None, |snapshot| {
            assert_eq!(snapshot, expected.as_slice());
            Ok(("explicit existing result", Vec::new()))
        }).unwrap().publish();
        assert_eq!(existing, "explicit existing result");
        assert_eq!((fixture.rows(), fixture.generation(), events.values()), unchanged);
    }

    #[test]
    fn closed_busy_and_failed_snapshot_reads_never_invoke_the_fresh_planner() {
        for case in ["closed", "busy", "read"] {
            let fixture = Fixture::new(case);
            let events = Publications::new(&fixture.owner);
            let before = (fixture.rows(), fixture.generation());
            let control = fixture.raw_db();
            match case {
                "closed" => fixture.owner.close(false),
                "busy" => control.execute_batch("BEGIN IMMEDIATE").unwrap(),
                "read" => control.execute_batch("ALTER TABLE transcript_entries RENAME TO unavailable_transcript").unwrap(),
                _ => unreachable!(),
            }
            let calls = AtomicUsize::new(0);
            let error = fixture.owner.commit_transcript_entries(None, None, |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(((), vec![message("t0u", "not admitted")]))
            }).err().expect("strict failure must not become empty success");
            match case {
                "closed" => assert!(matches!(error, AgentDbProjectionError::WorkAdmissionInvalid(_))),
                "busy" => assert!(matches!(error, AgentDbProjectionError::Sqlite(ref sqlite)
                    if sqlite.sqlite_error_code() == Some(rusqlite::ErrorCode::DatabaseBusy))),
                "read" => assert!(matches!(error, AgentDbProjectionError::Sqlite(_))),
                _ => unreachable!(),
            }
            match case {
                "busy" => control.execute_batch("ROLLBACK").unwrap(),
                "read" => control.execute_batch("ALTER TABLE unavailable_transcript RENAME TO transcript_entries").unwrap(),
                _ => {},
            }
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!((fixture.rows(), fixture.generation()), before);
            assert!(events.values().is_empty());
            assert_eq!(fixture.recoveries.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn invalid_or_occupied_second_plan_row_rolls_back_without_replaying_planner() {
        let fixture = Fixture::new("invalid-plan");
        fixture.owner.append_transcript_entries(&[message("occupied", "keep")]).unwrap();
        let events = Publications::new(&fixture.owner);
        for invalid in [
            message("occupied", "collision"),
            message("t0u", "duplicate within batch"),
            message(" ", "blank id"),
            json!({"kind": "message", "role": "user", "content": "no id"}),
            json!({"id": "second", "content": "no kind"}),
            json!({"id": "second", "kind": " "}),
        ] {
            let before = (fixture.rows(), fixture.generation());
            let calls = AtomicUsize::new(0);
            let error = fixture.owner.commit_transcript_entries(None, None, |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(((), vec![message("t0u", "must roll back"), invalid]))
            }).err().expect("invalid fresh plan must fail");
            assert!(matches!(error, AgentDbProjectionError::TranscriptAppendInvalid(_)));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!((fixture.rows(), fixture.generation()), before);
            assert!(events.values().is_empty());
        }
        let before = (fixture.rows(), fixture.generation());
        let calls = AtomicUsize::new(0);
        let error = fixture.owner.commit_transcript_entries(None, None, |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Err::<((), Vec<Value>), _>("producer plan refused".into())
        }).err().unwrap();
        assert!(matches!(error, AgentDbProjectionError::TranscriptAppendInvalid(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!((fixture.rows(), fixture.generation()), before);
        assert!(events.values().is_empty());
        assert_eq!(fixture.recoveries.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn fresh_db_admission_is_rechecked_before_planning_even_without_session_facade() {
        for case in ["legacy-selector", "wrong-selector", "stale", "empty", "archive", "delete", "foreign-id", "metadata", "malformed"] {
            let fixture = Fixture::new(case);
            let mut admission = fixture.bind_admission();
            let original_revision = admission.lifecycle_revision.clone();
            let events = Publications::new(&fixture.owner);
            let control = fixture.raw_db();
            match case {
                "archive" => admission.archived_at_ms = Some(100.0),
                "delete" => admission.deletion_pending = true,
                "foreign-id" => admission.session_id = "foreign-agent".into(),
                "empty" => admission.lifecycle_revision.clear(),
                _ => {},
            }
            control.execute("UPDATE kv SET value = ?1 WHERE key = 'sessionWorkAdmission'",
                [serde_json::to_string(&admission).unwrap()]).unwrap();
            if case == "metadata" {
                control.execute("UPDATE kv SET value = 'bad-hex' WHERE key = 'metadata'", []).unwrap();
            } else if case == "malformed" {
                control.execute("UPDATE kv SET value = 'not-json' WHERE key = 'sessionWorkAdmission'", []).unwrap();
            }
            let key = match case {
                "legacy-selector" => None,
                "wrong-selector" => Some("agent:other:main"),
                _ => Some(admission.session_key.as_str()),
            };
            let revision = if case == "stale" { Some("stale-generation") }
                else if case == "legacy-selector" { None }
                else { Some(original_revision.as_str()) };
            let calls = AtomicUsize::new(0);
            let before = (fixture.rows(), fixture.generation());
            assert!(fixture.owner.commit_transcript_entries(key, revision, |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(((), Vec::<Value>::new()))
            }).is_err());
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!((fixture.rows(), fixture.generation()), before);
            assert!(events.values().is_empty());
        }
    }

    #[test]
    fn fresh_commit_failure_is_typed_unknown_and_does_not_replay_or_publish() {
        let fixture = Fixture::new("commit-failure");
        let events = Publications::new(&fixture.owner);
        // The INSERT succeeds; SQLite checks this deferred constraint only at
        // COMMIT. This is a real commit failure, not an injected planner error.
        {
            let guard = fixture.owner.connection.lock().unwrap();
            guard.as_ref().unwrap().execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE fresh_parent(id TEXT PRIMARY KEY);
                 CREATE TABLE fresh_child(parent_id TEXT REFERENCES fresh_parent(id)
                     DEFERRABLE INITIALLY DEFERRED);
                 CREATE TRIGGER fail_fresh_commit AFTER INSERT ON transcript_entries
                 BEGIN INSERT INTO fresh_child(parent_id) VALUES ('missing'); END;",
            ).unwrap();
        }
        let before = (fixture.rows(), fixture.generation());
        let calls = AtomicUsize::new(0);
        let error = fixture.owner.commit_transcript_entries(None, None, |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(("must never escape as accepted", vec![message("t0u", "commit will fail")]))
        }).err().expect("deferred foreign key must fail COMMIT");
        assert!(matches!(error, AgentDbProjectionError::TranscriptCommitUnknown(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!((fixture.rows(), fixture.generation()), before);
        assert!(events.values().is_empty());
        assert_eq!(fixture.recoveries.load(Ordering::SeqCst), 0);
        assert!(fixture.owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
        let control = fixture.raw_db();
        assert_eq!(control.query_row("SELECT count(*) FROM fresh_child", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        control.execute_batch("DROP TRIGGER fail_fresh_commit").unwrap();
        fixture.owner.commit_transcript_entries(None, None, |_| {
            Ok(((), vec![message("t0u", "new explicit request")]))
        }).unwrap().publish();
        assert_eq!(fixture.generation(), before.1 + 1);
        assert_eq!(events.values().len(), 1);
        // This proves refusal/no implicit replay for a failing COMMIT. It does
        // not simulate a crash or an acknowledgment lost after a successful one.
    }

    #[test]
    fn fresh_insert_and_commit_failures_retire_an_unclean_connection() {
        for failure in ["insert", "commit"] {
            let fixture = Fixture::new(&format!("rollback-refused-{failure}"));
            let events = Publications::new(&fixture.owner);
            {
                let guard = fixture.owner.connection.lock().unwrap();
                let db = guard.as_ref().unwrap();
                if failure == "insert" {
                    db.execute_batch(
                        "CREATE TRIGGER refuse_second_fresh_row BEFORE INSERT ON transcript_entries
                         WHEN NEW.id = 't1u'
                         BEGIN SELECT RAISE(ABORT, 'fresh second insert refused'); END;",
                    ).unwrap();
                } else {
                    db.execute_batch(
                        "PRAGMA foreign_keys = ON;
                         CREATE TABLE fresh_rollback_parent(id TEXT PRIMARY KEY);
                         CREATE TABLE fresh_rollback_child(parent_id TEXT REFERENCES fresh_rollback_parent(id)
                             DEFERRABLE INITIALLY DEFERRED);
                         CREATE TRIGGER fail_fresh_rollback_commit AFTER INSERT ON transcript_entries
                         BEGIN INSERT INTO fresh_rollback_child(parent_id) VALUES ('missing'); END;",
                    ).unwrap();
                }
            }
            refuse_rollback_commands(&fixture.owner);
            let before = (fixture.rows(), fixture.generation());
            let calls = AtomicUsize::new(0);
            let error = fixture.owner.commit_transcript_entries(None, None, |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(((), vec![message("t0u", "actual first insert"), message("t1u", "actual second insert")]))
            }).err().expect("uncertain cleanup must refuse the fresh append");
            assert!(matches!(&error, AgentDbProjectionError::TranscriptCommitUnknown(_)));
            let expected_cause = if failure == "insert" { "fresh second insert refused" } else { "FOREIGN KEY" };
            assert!(error.to_string().contains(expected_cause), "{failure}: {error}");
            assert_eq!(calls.load(Ordering::SeqCst), 1, "cleanup must not replay the planner");
            assert!(fixture.owner.is_closed());
            assert!(fixture.owner.connection.lock().unwrap().is_none());
            assert_eq!(live_db_handle_count(fixture.owner.db_path()), 0);
            assert_eq!((fixture.rows(), fixture.generation()), before);
            assert!(fixture.owner.get_transcript_entries().unwrap().is_empty(),
                "the retired owner cannot expose an uncommitted prefix");
            assert!(fixture.owner.commit_transcript_entries::<()>(None, None, |_| {
                panic!("a retired owner cannot accept another planner")
            }).is_err());
            assert!(events.values().is_empty());
            assert_eq!(fixture.recoveries.load(Ordering::SeqCst), 0);

            let reopened = SandAgentDb::open_existing_agent(fixture.owner.db_path(), 0).unwrap();
            assert!(reopened.get_transcript_entries().unwrap().is_empty());
            assert_eq!(fixture.generation(), before.1);
            reopened.close(false);
            // Reopening inspects storage; it does not resolve the refused
            // caller's unknown outcome or authorize replay of its work.
        }
    }

    #[test]
    fn actual_agent_inbound_commit_unknown_retains_reply_claim_without_side_effects() {
        assert_actual_agent_inbound_unknown_retains_reply_claim("commit", false);
    }

    #[test]
    fn actual_agent_inbound_rollback_unknown_retires_owner_and_retains_reply_claim() {
        for failure in ["insert", "commit"] {
            assert_actual_agent_inbound_unknown_retains_reply_claim(failure, true);
        }
    }

    fn assert_actual_agent_inbound_unknown_retains_reply_claim(failure: &str, refuse_rollback: bool) {
        use crate::extensions::session::production::ProductionSessionWorkers;
        use crate::extensions::session::session_conversation_state::PendingConversationTurnClaimResult;
        use crate::extensions::transcript::agent_to_agent_messaging::persist_agent_inbound_message;

        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!(
            "fabushi-fresh-a2a-unknown-{failure}-{refuse_rollback}-{}-{suffix}", std::process::id(),
        ));
        let sessions = ProductionSessionWorkers::with_agents_root(&root, 50);
        let source = sessions.materialize_new_session(None, "user", None).unwrap();
        let target = sessions.materialize_new_session(None, "user", None).unwrap();
        let admission = sessions.read_session_work_admission(&target.id).unwrap().unwrap();
        let owner = sessions.open_agent_db_owner(&target.id).unwrap();
        // Settle ordinary first-read roster activity seeding before measuring
        // side effects of the refused inbound transaction.
        assert!(sessions.summarize_agent_by_id(&source.id, None).unwrap().is_some());
        assert!(sessions.summarize_agent_by_id(&target.id, None).unwrap().is_some());
        let turns = sessions.pending_conversation_turns();
        let turn_id = turns.register(
            &target.id, Some("pending-unknown-reply"), &source.id, &admission.session_id,
            None, Duration::from_secs(60),
        ).unwrap();
        assert!(turns.set_outbound_message_id(&target.id, &turn_id, "request-unknown"));
        assert!(turns.mark_ready(&target.id, &turn_id));
        let mut expected_claim = turns.snapshot(&target.id, &turn_id).unwrap();
        assert!(!expected_claim.claimed);
        expected_claim.claimed = true;

        {
            // Foreign-key enforcement belongs to the actual connection used
            // by Session, not the separate observation connection below.
            let guard = owner.connection.lock().unwrap();
            let db = guard.as_ref().unwrap();
            if failure == "commit" {
                db.execute_batch(
                    "PRAGMA foreign_keys = ON;
                     CREATE TABLE fresh_a2a_parent(id TEXT PRIMARY KEY);
                     CREATE TABLE fresh_a2a_child(parent_id TEXT REFERENCES fresh_a2a_parent(id)
                         DEFERRABLE INITIALLY DEFERRED);
                     CREATE TRIGGER fail_fresh_a2a_commit AFTER INSERT ON transcript_entries
                     BEGIN INSERT INTO fresh_a2a_child(parent_id) VALUES ('missing'); END;",
                ).unwrap();
                assert_eq!(db.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
            } else {
                db.execute_batch(
                    "CREATE TRIGGER fail_fresh_a2a_insert AFTER INSERT ON transcript_entries
                     BEGIN SELECT RAISE(FAIL, 'fresh reply insert refused after mutation'); END;",
                ).unwrap();
            }
        }
        if refuse_rollback {
            refuse_rollback_commands(&owner);
        }
        let control = rusqlite::Connection::open(owner.db_path()).unwrap();
        let durable_snapshot = || {
            let mut transcript = control.prepare(
                "SELECT seq, id, entry FROM transcript_entries ORDER BY seq",
            ).unwrap();
            let rows = transcript.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
            let mut kv = control.prepare("SELECT key, value FROM kv ORDER BY key").unwrap();
            let values = kv.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
            (rows, values, get_sand_agent_db_write_generation(owner.db_path()))
        };
        let before = durable_snapshot();
        assert!(before.0.is_empty());
        let partners = sessions.get_agent_conversation_partner_ids(&target.id).unwrap();
        assert!(partners.is_empty());
        let events = Publications::new(&owner);
        let inbound = json!({
            "from": {"id": source.id}, "messageId": "reply-unknown",
            "replyToId": "request-unknown", "text": "reply awaiting durable commit",
            "timestampMs": 100,
        });

        let error = persist_agent_inbound_message(&sessions, &target.id, &inbound)
            .expect_err("the actual inbound producer must refuse uncertain settlement");
        assert!(error.contains("commit outcome is unknown"), "{error}");
        assert!(error.contains("do not replay"), "{error}");
        let expected_cause = if failure == "commit" { "FOREIGN KEY" } else { "fresh reply insert refused after mutation" };
        assert!(error.contains(expected_cause), "the actual SQL fault must cause failure: {error}");
        assert_eq!(turns.snapshot(&target.id, &turn_id), Some(expected_claim.clone()),
            "unknown settlement keeps the pending claim held, neither released nor completed");
        assert_eq!(turns.claim_reply(
            &target.id, &source.id, None, &admission.session_id, "competing-reply",
            Some("request-unknown"), None, "must not acquire held custody", 101,
        ), PendingConversationTurnClaimResult::NotFound);
        assert_eq!(turns.snapshot(&target.id, &turn_id), Some(expected_claim));
        assert_eq!(durable_snapshot(), before);
        assert!(events.values().is_empty());
        if failure == "commit" {
            assert_eq!(control.query_row("SELECT count(*) FROM fresh_a2a_child", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        }
        if refuse_rollback {
            assert!(owner.is_closed());
            assert!(owner.connection.lock().unwrap().is_none());
            assert_eq!(live_db_handle_count(owner.db_path()), 0);
            assert!(owner.get_transcript_entries().unwrap().is_empty());
            assert!(sessions.get_agent_conversation_partner_ids(&target.id).is_err(),
                "Session cannot admit reads through its retired cached owner");
        } else {
            assert!(!owner.is_closed());
            assert!(owner.connection.lock().unwrap().as_ref().unwrap().is_autocommit());
            assert_eq!(sessions.get_agent_conversation_partner_ids(&target.id).unwrap(), partners);
        }

        // This exercises the producer's unknown-result custody branch after
        // actual write/COMMIT errors and, when selected, refused cleanup. It
        // does not prove crash reconciliation or replay the held inbound.
        drop(events);
        drop(control);
        sessions.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn immediate_transaction_excludes_an_independent_allocator_until_commit() {
        let fixture = Fixture::new("independent-owner");
        let first = SandAgentDb::open_existing_agent(fixture.owner.db_path(), 0).unwrap();
        let second = SandAgentDb::open_existing_agent(fixture.owner.db_path(), 0).unwrap();
        let before = fixture.generation();
        let events = Publications::new(&fixture.owner);
        let (snapshot_tx, snapshot_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = first.commit_transcript_entries(None, None, |snapshot| {
                let id = next_entry_id(snapshot, TranscriptEntryIdKind::UserMessage);
                snapshot_tx.send((snapshot.len(), id.clone())).unwrap();
                // Test-only rendezvous at the production planner proves the
                // first real BEGIN IMMEDIATE and snapshot are held right now.
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                Ok((id.clone(), vec![message(&id, "first producer")]))
            }).map(|committed| committed.publish());
            let _ = done_tx.send(result);
        });
        assert_eq!(snapshot_rx.recv_timeout(Duration::from_secs(5)).unwrap(), (0, "t0u".into()));
        let second_calls = AtomicUsize::new(0);
        let competing = second.commit_transcript_entries(None, None, |snapshot| {
            second_calls.fetch_add(1, Ordering::SeqCst);
            let id = next_entry_id(snapshot, TranscriptEntryIdKind::UserMessage);
            Ok((id.clone(), vec![message(&id, "second producer")]))
        });
        // Release before assertions so a failing oracle cannot strand the
        // first worker. No timing guess or sleep establishes the ordering.
        release_tx.send(()).unwrap();
        let first_result = done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        worker.join().unwrap();
        assert!(matches!(competing, Err(AgentDbProjectionError::Sqlite(ref error))
            if error.sqlite_error_code() == Some(rusqlite::ErrorCode::DatabaseBusy)));
        assert_eq!(second_calls.load(Ordering::SeqCst), 0);
        assert_eq!(first_result.unwrap(), "t0u");
        assert_eq!(fixture.generation(), before + 1);
        assert_eq!(events.values().len(), 1);

        // This is an explicit new call after observed commit, not owner retry.
        let second_id = second.commit_transcript_entries(None, None, |snapshot| {
            second_calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(snapshot, &[message("t0u", "first producer")]);
            let id = next_entry_id(snapshot, TranscriptEntryIdKind::UserMessage);
            Ok((id.clone(), vec![message(&id, "second producer")]))
        }).unwrap().publish();
        assert_eq!(second_id, "t1u");
        assert_eq!(second_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.owner.get_transcript_entries().unwrap(), vec![
            message("t0u", "first producer"), message("t1u", "second producer"),
        ]);
        assert_eq!(fixture.generation(), before + 2);
        assert_eq!(events.values().len(), 2);
    }
}
