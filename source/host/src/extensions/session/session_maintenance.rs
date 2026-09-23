use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::agent_isolation::{AgentWorkerPool, ProductionAgentStoreWorkerBackend};
use crate::storage::store_db::{
    get_sand_agent_db_write_generation, live_db_handle_count,
};

use super::agent_db::{
    append_persisted_transcript_entries, compare_and_set_persisted_latest_root_blob_id,
    has_persisted_legacy_conversation_blobs, legacy_blob_retirement_version,
    read_persisted_latest_root_blob_id, read_persisted_transcript_entries,
    retire_persisted_legacy_conversation_blobs, set_stale_root_cleanup_version,
    stale_root_cleanup_version,
};
use super::conversation_recovery::{
    conversation_structure_fully_resolves, parse_conversation_state_structure,
    transcript_entry_matches_recovered,
};

pub const LEGACY_BLOB_RETIREMENT_VERSION: u64 = 1;
pub const STALE_ROOT_CLEANUP_VERSION: u64 = 1;
pub const HIDDEN_ENTRY_REPAIR_VERSION: u64 = 1;

static LEGACY_RETIREMENT: AtomicBool = AtomicBool::new(false);
static STALE_ROOT_GC: AtomicBool = AtomicBool::new(false);

pub fn pin_legacy_store_blob_retirement(enabled: bool) {
    LEGACY_RETIREMENT.store(enabled, Ordering::Release);
}

pub fn is_legacy_store_blob_retirement_enabled() -> bool {
    match std::env::var("SAND_RETIRE_LEGACY_STORE_BLOBS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("1" | "true" | "on") => true,
        Some("0" | "false" | "off") => false,
        _ => LEGACY_RETIREMENT.load(Ordering::Acquire),
    }
}

pub fn pin_stale_root_gc(enabled: bool) {
    STALE_ROOT_GC.store(enabled, Ordering::Release);
}

pub fn backfill_transcript(
    db_path: &Path,
    busy_timeout_ms: u64,
    rebuilt: &[serde_json::Value],
) -> Result<usize, String> {
    let persisted = read_persisted_transcript_entries(db_path, busy_timeout_ms)
        .map_err(|error| error.to_string())?;
    if persisted.len() >= rebuilt.len()
        || !persisted.iter().enumerate().all(|(index, entry)| {
            rebuilt
                .get(index)
                .is_some_and(|candidate| transcript_entry_matches_recovered(entry, candidate))
        })
    {
        return Ok(0);
    }
    append_persisted_transcript_entries(
        db_path,
        busy_timeout_ms,
        &rebuilt[persisted.len()..],
    )
    .map_err(|error| error.to_string())
}

pub fn find_latest_durable_root_blob_id(
    pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    blob_db_path: &Path,
    legacy_blob_db_path: &Path,
) -> Result<Option<Vec<u8>>, String> {
    futures::executor::block_on(pool.find_latest_root_blob_id(
        agent_id,
        blob_db_path,
        Some(legacy_blob_db_path),
    ))
    .map_err(|error| error.to_string())
}

pub fn recover_conversation_root_if_missing(
    pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, String> {
    let existing = read_persisted_latest_root_blob_id(db_path, busy_timeout_ms)
        .map_err(|error| error.to_string())?;
    if !existing.is_empty() {
        return Ok(false);
    }
    if !blob_db_path.exists()
        && !has_persisted_legacy_conversation_blobs(db_path, busy_timeout_ms)
            .map_err(|error| error.to_string())?
    {
        return Ok(false);
    }

    let recovery_generation = get_sand_agent_db_write_generation(db_path);
    let Some(root) = find_latest_durable_root_blob_id(
        Arc::clone(&pool),
        agent_id,
        blob_db_path,
        db_path,
    )? else {
        return Ok(false);
    };
    let Some(root_blob) = futures::executor::block_on(pool.get_blob(
        agent_id,
        blob_db_path,
        &root,
        Some(db_path),
    ))
    .map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    let Some(structure) = parse_conversation_state_structure(&root_blob) else {
        return Ok(false);
    };
    let refs = structure.refs();
    let resolves = futures::executor::block_on(conversation_structure_fully_resolves(
        &refs,
        |blob_id| {
            let pool = Arc::clone(&pool);
            let agent_id = agent_id.to_string();
            let blob_db_path = blob_db_path.to_path_buf();
            let db_path = db_path.to_path_buf();
            async move {
                pool.get_blob(
                    &agent_id,
                    &blob_db_path,
                    &blob_id,
                    Some(&db_path),
                )
                .await
                .map_err(|error| error.to_string())
            }
        },
    ));
    if !resolves || get_sand_agent_db_write_generation(db_path) != recovery_generation {
        return Ok(false);
    }
    compare_and_set_persisted_latest_root_blob_id(
        db_path,
        busy_timeout_ms,
        &existing,
        &root,
    )
    .map_err(|error| error.to_string())
}

pub fn clear_stale_checkpoint_roots_once(
    pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, String> {
    if !STALE_ROOT_GC.load(Ordering::Acquire)
        || stale_root_cleanup_version(db_path, busy_timeout_ms)
            .map_err(|error| error.to_string())?
            >= STALE_ROOT_CLEANUP_VERSION
    {
        return Ok(false);
    }
    let root = read_persisted_latest_root_blob_id(db_path, busy_timeout_ms)
        .map_err(|error| error.to_string())?;
    if root.is_empty() || live_db_handle_count(db_path) > 1 {
        return Ok(false);
    }
    let Some(root_blob) = futures::executor::block_on(pool.get_blob(
        agent_id,
        blob_db_path,
        &root,
        Some(db_path),
    ))
    .map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    let Some(structure) = parse_conversation_state_structure(&root_blob) else {
        return Ok(false);
    };
    if structure.turns.is_empty() {
        return Ok(false);
    }

    let before = get_sand_agent_db_write_generation(db_path);
    futures::executor::block_on(pool.clear_stale_checkpoint_roots(
        agent_id,
        blob_db_path,
        &hex(&root),
        Some(db_path),
    ))
    .map_err(|error| error.to_string())?;
    if get_sand_agent_db_write_generation(db_path) != before
        || read_persisted_latest_root_blob_id(db_path, busy_timeout_ms)
            .map_err(|error| error.to_string())?
            != root
        || live_db_handle_count(db_path) > 1
    {
        return Ok(false);
    }
    set_stale_root_cleanup_version(db_path, busy_timeout_ms, STALE_ROOT_CLEANUP_VERSION)
        .map_err(|error| error.to_string())
}

pub fn retire_legacy_store_blobs_once(
    pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<bool, String> {
    if !is_legacy_store_blob_retirement_enabled()
        || legacy_blob_retirement_version(db_path, busy_timeout_ms)
            .map_err(|error| error.to_string())?
            >= LEGACY_BLOB_RETIREMENT_VERSION
    {
        return Ok(false);
    }
    if !has_persisted_legacy_conversation_blobs(db_path, busy_timeout_ms)
        .map_err(|error| error.to_string())?
    {
        return retire_persisted_legacy_conversation_blobs(
            db_path,
            busy_timeout_ms,
            LEGACY_BLOB_RETIREMENT_VERSION,
        )
        .map_err(|error| error.to_string());
    }

    let root = read_persisted_latest_root_blob_id(db_path, busy_timeout_ms)
        .map_err(|error| error.to_string())?;
    if root.is_empty() || live_db_handle_count(db_path) > 1 {
        return Ok(false);
    }
    let Some(root_blob) = futures::executor::block_on(pool.get_blob(
        agent_id,
        blob_db_path,
        &root,
        Some(db_path),
    ))
    .map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    let Some(structure) = parse_conversation_state_structure(&root_blob) else {
        return Ok(false);
    };
    if structure.turns.is_empty() {
        return Ok(false);
    }

    let before = get_sand_agent_db_write_generation(db_path);
    let verdict = futures::executor::block_on(pool.verify_legacy_blob_retirement(
        agent_id,
        blob_db_path,
        &hex(&root),
        db_path,
    ))
    .map_err(|error| error.to_string())?;
    if !verdict.is_retirable
        || get_sand_agent_db_write_generation(db_path) != before
        || read_persisted_latest_root_blob_id(db_path, busy_timeout_ms)
            .map_err(|error| error.to_string())?
            != root
        || live_db_handle_count(db_path) > 1
    {
        return Ok(false);
    }
    retire_persisted_legacy_conversation_blobs(
        db_path,
        busy_timeout_ms,
        LEGACY_BLOB_RETIREMENT_VERSION,
    )
    .map_err(|error| error.to_string())
}

pub fn cleanup_legacy_group_member_dirs(
    root_dir: &Path,
    dirname_to_remove: &str,
) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    let Ok(entries) = fs::read_dir(root_dir) else {
        return failures;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let target = entry.path().join(dirname_to_remove);
        if let Err(error) = fs::remove_dir_all(&target) {
            if error.kind() != std::io::ErrorKind::NotFound {
                failures.push((
                    entry.file_name().to_string_lossy().into_owned(),
                    error.to_string(),
                ));
            }
        }
    }
    failures
}

pub fn default_legacy_member_dirname() -> &'static str {
    "members"
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn legacy_member_path(root: &Path, agent_id: &str) -> PathBuf {
    root.join(agent_id).join(default_legacy_member_dirname())
}
