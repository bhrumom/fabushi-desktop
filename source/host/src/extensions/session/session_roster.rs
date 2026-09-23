use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use crate::storage::store_db::live_db_handle_count;
use crate::automations::automation_store::agent_has_automations;
use crate::workflows::workflow_store::agent_has_workflows;

use super::agent_db::reseed_minimal_persisted_agent_db_if_missing;
use super::session_mutations::recover_agent_with_missing_db;
use super::session_paths::get_agent_db_path;
use super::session_summaries::{
    AgentSummary, DurableFootprint, build_summary, file_mtime_ms,
    load_agent_db_extras,
};

fn durable_footprint(agent_dir: &Path) -> DurableFootprint {
    DurableFootprint {
        has_automations: agent_has_automations(agent_dir),
        has_workflows: agent_has_workflows(agent_dir),
        ..DurableFootprint::default()
    }
}

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
        let footprint = durable_footprint(entry.path().as_path());
        if !db_path.is_file() {
            if let Some(summary) = recover_agent_with_missing_db(
                &db_path,
                &dir_name,
                active_agent_id,
                footprint,
                live_db_handle_count(&db_path) > 0,
                |path| {
                    reseed_minimal_persisted_agent_db_if_missing(path, busy_timeout_ms)
                        .map_err(|error| error.to_string())
                },
            )? {
                summaries.push(summary);
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
                footprint,
            )?,
            Err(_) => build_summary(
                None,
                &db_path,
                &dir_name,
                mtime_ms,
                active_agent_id,
                true,
                footprint,
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
    let footprint = durable_footprint(db_path.parent().unwrap_or_else(|| Path::new(".")));
    if !db_path.is_file() {
        return recover_agent_with_missing_db(
            &db_path,
            agent_id,
            active_agent_id,
            footprint,
            live_db_handle_count(&db_path) > 0,
            |path| {
                reseed_minimal_persisted_agent_db_if_missing(path, busy_timeout_ms)
                    .map_err(|error| error.to_string())
            },
        );
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
        footprint,
    )
}
