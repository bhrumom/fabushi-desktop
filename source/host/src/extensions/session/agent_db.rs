use std::path::Path;

use rusqlite::{OptionalExtension, params};

use crate::storage::store_db::{bump_db_write_generation, live_db_handle_count};
use crate::transcript_mutation_events::publish_transcript_mutation;

use super::agent_db_recovery::{
    AgentDbRecoveryError, DbRecoveryOptions, open_configured_db,
};
use super::agent_db_schema::{
    CLEAR_BLOBS_SQL, CLEAR_TRANSCRIPT_ENTRIES_SQL, COMPARE_AND_SET_KV_SQL, DELETE_KV_SQL,
    DELETE_TRANSCRIPT_ENTRY_SQL, GET_KV_SQL, GET_TRANSCRIPT_ENTRY_SQL, HAS_LEGACY_BLOB_SQL,
    INSERT_TRANSCRIPT_ENTRY_SQL, LIST_TRANSCRIPT_ENTRIES_SQL, NEWEST_DIVIDER_ANCHOR_TIMESTAMP_SQL,
    SET_KV_SQL, UPDATE_TRANSCRIPT_ENTRY_SQL,
};
use super::agent_db_serde::{
    AwaitingUserResponse, EPISODE_PENDING_MAX, EPISODE_TURN_TEXT_CAP, EpisodeTurn,
    MemoryPromptSnapshot, REQUEST_ID_HISTORY_MAX, REQUEST_ID_PROMPT_MAX, RequestRecord,
    SandProfile, SpendGuardState, UnreadState, parse_awaiting_state,
    parse_memory_prompt_snapshot, parse_pending_episode_turns, parse_profile,
    parse_request_records, parse_transcript_entry, parse_unread_state,
    resolve_spend_guard_state,
};
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptWindowQuery, read_transcript_tail,
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
    #[error("latestRootBlobId hex is invalid: {0}")]
    LatestRootHex(String),
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

#[derive(Debug, Clone, PartialEq)]
pub struct AgentMetadataProjection {
    pub agent_id: String,
    pub created_at: f64,
    pub latest_root_blob_id: Vec<u8>,
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
    let db = open_projection_db(db_path, busy_timeout_ms)?;
    let Some(metadata) = read_metadata_json(&db)? else {
        return Ok(None);
    };
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
