use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use super::session_paths::get_agent_db_path;
use super::session_summaries::{
    AgentSummary, DurableFootprint, build_summary, file_mtime_ms,
    load_agent_db_extras, minimal_agent_summary,
};

pub fn list_agents(
    root_dir: &Path,
    busy_timeout_ms: u64,
    active_agent_id: Option<&str>,
) -> Result<Vec<AgentSummary>, String> {
    let entries = match fs::read_dir(root_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut summaries = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let db_path = get_agent_db_path(root_dir, &dir_name)
            .map_err(|error| error.to_string())?;
        let mtime_ms = file_mtime_ms(&db_path);
        if !db_path.is_file() {
            let profile_path = entry.path().join("profile.json");
            if profile_path.is_file() {
                summaries.push(minimal_agent_summary(
                    &dir_name,
                    &db_path,
                    file_mtime_ms(&profile_path),
                    active_agent_id,
                ));
            }
            continue;
        }
        let summary = match load_agent_db_extras(
            &db_path,
            busy_timeout_ms,
            &dir_name,
            mtime_ms,
        ) {
            Ok(extras) => build_summary(
                Some(&extras),
                &db_path,
                &dir_name,
                mtime_ms,
                active_agent_id,
                false,
                DurableFootprint::default(),
            )?,
            Err(_) => build_summary(
                None,
                &db_path,
                &dir_name,
                mtime_ms,
                active_agent_id,
                true,
                DurableFootprint::default(),
            )?,
        };
        if let Some(summary) = summary {
            summaries.push(summary);
        }
    }
    summaries.sort_by(|a, b| {
        b.updated_at
            .partial_cmp(&a.updated_at)
            .unwrap_or(Ordering::Equal)
    });
    Ok(summaries)
}

pub fn summarize_agent_by_id(
    root_dir: &Path,
    busy_timeout_ms: u64,
    agent_id: &str,
    active_agent_id: Option<&str>,
) -> Result<Option<AgentSummary>, String> {
    let db_path = get_agent_db_path(root_dir, agent_id)
        .map_err(|error| error.to_string())?;
    let mtime_ms = file_mtime_ms(&db_path);
    if !db_path.is_file() {
        let profile_path = root_dir.join(agent_id).join("profile.json");
        if !profile_path.is_file() {
            return Ok(None);
        }
        return Ok(Some(minimal_agent_summary(
            agent_id,
            &db_path,
            file_mtime_ms(&profile_path),
            active_agent_id,
        )));
    }
    let extras = load_agent_db_extras(
        &db_path,
        busy_timeout_ms,
        agent_id,
        mtime_ms,
    )?;
    build_summary(
        Some(&extras),
        &db_path,
        agent_id,
        mtime_ms,
        active_agent_id,
        true,
        DurableFootprint::default(),
    )
}
