use std::fs;
use std::path::Path;

use crate::agents::agent_avatar::{
    CANONICAL_AVATAR_FILENAME, invalidate_avatar_data_url_cache,
    list_conventional_avatar_filenames,
};

use super::agent_db::read_persisted_agent_serde_snapshot;
use super::agent_db_serde::SandProfile;
use super::session_summaries::{
    AgentSummary, DurableFootprint, agent_has_durable_footprint, build_summary,
    file_mtime_ms,
};

pub fn set_agent_avatar_bytes<F>(
    db_path: &Path,
    busy_timeout_ms: u64,
    png_bytes: Option<&[u8]>,
    set_profile: F,
) -> Result<(), String>
where
    F: FnOnce(&SandProfile) -> Result<(), String>,
{
    let agent_dir = db_path
        .parent()
        .ok_or_else(|| "agent database has no parent directory".to_string())?;
    let prior_state = if png_bytes.is_none() {
        Some(
            read_persisted_agent_serde_snapshot(db_path, busy_timeout_ms)
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    let previous_files = list_conventional_avatar_filenames(agent_dir)
        .into_iter()
        .filter_map(|name| {
            let path = agent_dir.join(name);
            fs::read(&path).ok().map(|bytes| (path, bytes))
        })
        .collect::<Vec<_>>();
    let restore_previous = || -> Result<(), String> {
        for (path, bytes) in &previous_files {
            fs::write(path, bytes).map_err(|error| error.to_string())?;
        }
        Ok(())
    };
    for name in list_conventional_avatar_filenames(agent_dir) {
        let _ = fs::remove_file(agent_dir.join(name));
    }
    match png_bytes {
        None => {
            let state = prior_state
                .as_ref()
                .expect("prior avatar state is captured for avatar removal");
            if state.profile.avatar_path.is_some() {
                if let Err(error) = set_profile(&SandProfile {
                    description: state.profile.description.clone(),
                    avatar_path: None,
                }) {
                    let restore_error = restore_previous().err();
                    return Err(match restore_error {
                        Some(restore_error) => {
                            format!("{error}; avatar rollback failed: {restore_error}")
                        }
                        None => error,
                    });
                }
            }
        }
        Some(bytes) => {
            if let Err(error) = fs::create_dir_all(agent_dir) {
                let restore_error = restore_previous().err();
                return Err(match restore_error {
                    Some(restore_error) => {
                        format!("{error}; avatar rollback failed: {restore_error}")
                    }
                    None => error.to_string(),
                });
            }
            if let Err(error) = fs::write(agent_dir.join(CANONICAL_AVATAR_FILENAME), bytes) {
                let restore_error = restore_previous().err();
                return Err(match restore_error {
                    Some(restore_error) => {
                        format!("{error}; avatar rollback failed: {restore_error}")
                    }
                    None => error.to_string(),
                });
            }
        }
    }
    invalidate_avatar_data_url_cache(agent_dir);
    Ok(())
}

pub fn recover_agent_with_missing_db<F, D>(
    db_path: &Path,
    dir_name: &str,
    active_agent_id: Option<&str>,
    footprint: DurableFootprint,
    has_live_db_handle: bool,
    mut is_agent_being_deleted: D,
    mut reseed_minimal_store_db_if_missing: F,
) -> Result<Option<AgentSummary>, String>
where
    F: FnMut(&Path) -> Result<(), String>,
    D: FnMut() -> bool,
{
    if is_agent_being_deleted() {
        return Ok(None);
    }
    let agent_dir = db_path
        .parent()
        .ok_or_else(|| "agent database has no parent directory".to_string())?;
    let intact = agent_dir.join("profile.json").is_file()
        || agent_has_durable_footprint(agent_dir, footprint);
    if !intact || is_agent_being_deleted() {
        return Ok(None);
    }
    let mtime_ms = if has_live_db_handle {
        None
    } else {
        reseed_minimal_store_db_if_missing(db_path)?;
        file_mtime_ms(db_path)
    };
    build_summary(
        None,
        db_path,
        dir_name,
        mtime_ms,
        active_agent_id,
        true,
        footprint,
    )
}
