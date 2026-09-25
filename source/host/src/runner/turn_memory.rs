use std::env;

use crate::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind,
};
use crate::extensions::session::agent_db::SandAgentDb;

use super::sand_memory::{
    MEMORY_EPISODE_PREFIX, MEMORY_EXTRACTION_ARCHIVE_SCAN_LIMIT,
    MEMORY_EXTRACTION_NONE_SENTINEL, MEMORY_RECENT_PROMPT_LIMIT,
    EpisodeTurn as MemoryEpisodeTurn, apply_extracted_memories,
    build_episode_system_prompt, build_episode_user_prompt,
    build_extraction_system_prompt, build_extraction_user_prompt,
    episode_interval, gather_extraction_memories, parse_extracted_memories,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnExchange {
    pub user: String,
    pub agent: String,
}

pub enum TurnMemoryMode<'a> {
    Extract,
    RecordEvidence(&'a mut dyn FnMut(&TurnExchange)),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnMemoryReport {
    pub evidence_recorded: bool,
    pub extraction_attempted: bool,
    pub added_memories: usize,
    pub removed_memories: usize,
    pub episode_recorded: bool,
    pub episode_summary_attempted: bool,
    pub episode_memory_added: bool,
}

pub fn run_turn_memory_with<Execute>(
    memory_store: &FileMemoryStore,
    episode_progress: Option<&SandAgentDb>,
    turn_timestamp: i64,
    exchange: TurnExchange,
    mode: TurnMemoryMode<'_>,
    mut execute: Execute,
) -> TurnMemoryReport
where
    Execute: FnMut(&str, &str) -> Result<String, String>,
{
    let mut report = TurnMemoryReport::default();

    if let TurnMemoryMode::RecordEvidence(record) = mode {
        if let Some(progress) = episode_progress {
            let _ = progress.clear_pending_episode_turns();
        }
        record(&exchange);
        report.evidence_recorded = true;
        return report;
    }

    report.extraction_attempted = true;
    run_memory_extraction(
        memory_store,
        &exchange,
        &mut execute,
        &mut report,
    );

    let Some(progress) = episode_progress else {
        return report;
    };
    if progress
        .record_episode_turn(
            &crate::extensions::session::agent_db_serde::EpisodeTurn {
                ts: turn_timestamp,
                user: exchange.user.clone(),
                agent: exchange.agent.clone(),
            },
        )
        .is_err()
    {
        return report;
    }
    report.episode_recorded = true;

    let Ok(pending) = progress.get_pending_episode_turns() else {
        return report;
    };
    let interval_raw = env::var("SAND_MEMORY_EPISODE_INTERVAL").ok();
    if pending.len() < episode_interval(interval_raw.as_deref()) {
        return report;
    }

    report.episode_summary_attempted = true;
    let turns = pending
        .iter()
        .map(|turn| MemoryEpisodeTurn {
            ts: turn.ts,
            user: turn.user.clone(),
            agent: turn.agent.clone(),
        })
        .collect::<Vec<_>>();
    let latest_timestamp = pending
        .last()
        .map(|turn| turn.ts)
        .unwrap_or(turn_timestamp);

    let summary = execute(
        &build_episode_system_prompt(),
        &build_episode_user_prompt(&turns),
    )
    .ok()
    .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
    .filter(|value| {
        !value.is_empty()
            && !value.eq_ignore_ascii_case(MEMORY_EXTRACTION_NONE_SENTINEL)
    });

    if let Some(summary) = summary {
        if memory_store
            .add_memory(
                &format!("{MEMORY_EPISODE_PREFIX}{summary}"),
                latest_timestamp,
                MemoryKind::Log,
            )
            .ok()
            .flatten()
            .is_some()
        {
            report.episode_memory_added = true;
        }
    }

    let _ = progress.clear_pending_episode_turns();
    report
}

fn run_memory_extraction<Execute>(
    memory_store: &FileMemoryStore,
    exchange: &TurnExchange,
    execute: &mut Execute,
    report: &mut TurnMemoryReport,
) where
    Execute: FnMut(&str, &str) -> Result<String, String>,
{
    let recall = memory_store.recall(MEMORY_RECENT_PROMPT_LIMIT);
    let archive = memory_store.list_memories(MEMORY_EXTRACTION_ARCHIVE_SCAN_LIMIT);
    let existing = gather_extraction_memories(
        &recall,
        &archive,
        &format!("{}\n{}", exchange.user, exchange.agent),
    );
    let Ok(raw) = execute(
        &build_extraction_system_prompt(),
        &build_extraction_user_prompt(
            &exchange.user,
            &exchange.agent,
            &existing,
        ),
    ) else {
        return;
    };
    let extraction = parse_extracted_memories(&raw, &existing);
    let Ok(applied) = apply_extracted_memories(
        memory_store,
        &extraction,
        chrono::Utc::now().timestamp_millis(),
        &existing,
    ) else {
        return;
    };
    report.added_memories = applied.added.len();
    report.removed_memories = applied.removed.len();
}
