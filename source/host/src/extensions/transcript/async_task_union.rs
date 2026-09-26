use std::collections::HashSet;

use serde::Serialize;

use super::sand_pending_wake_store::{DurablePendingWakeMarker, PendingWakeKind};

pub const LEDGER_ONLY_DETAIL: &str = "from the durable pending-wake ledger";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncTask {
    pub kind: String,
    pub id: String,
    pub label: String,
    pub status: String,
    pub started_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

fn kind_name(kind: PendingWakeKind) -> &'static str {
    match kind {
        PendingWakeKind::CloudAgent => "cloud-agent",
        PendingWakeKind::Shell => "shell",
        PendingWakeKind::Subagent => "subagent",
    }
}

pub fn marker_label(marker: &DurablePendingWakeMarker) -> String {
    if let Some(title) = marker.title.as_deref().filter(|title| !title.is_empty()) {
        return title.to_string();
    }
    match marker.kind {
        PendingWakeKind::CloudAgent => format!("Cloud agent {}", marker.work_id),
        PendingWakeKind::Shell => format!("Background command {}", marker.work_id),
        PendingWakeKind::Subagent => format!("Background task {}", marker.work_id),
    }
}

pub fn pending_wake_marker_to_async_task(marker: &DurablePendingWakeMarker) -> AsyncTask {
    let detail = marker
        .subagent_type
        .as_deref()
        .filter(|value| !value.is_empty())
        .map(|value| format!("{value} · {LEDGER_ONLY_DETAIL}"))
        .or_else(|| Some(LEDGER_ONLY_DETAIL.to_string()));
    AsyncTask {
        kind: kind_name(marker.kind).to_string(),
        id: marker.work_id.clone(),
        label: marker_label(marker),
        status: "running".to_string(),
        started_at_ms: marker.marked_at_ms,
        detail,
    }
}

pub fn merge_async_tasks(
    live_tasks: &[AsyncTask],
    markers: &[DurablePendingWakeMarker],
) -> Vec<AsyncTask> {
    let mut seen = live_tasks
        .iter()
        .map(|task| (task.kind.clone(), task.id.clone()))
        .collect::<HashSet<_>>();
    let mut merged = live_tasks.to_vec();
    for marker in markers {
        let key = (kind_name(marker.kind).to_string(), marker.work_id.clone());
        if seen.insert(key) {
            merged.push(pending_wake_marker_to_async_task(marker));
        }
    }
    merged.sort_by(|a, b| {
        a.started_at_ms
            .partial_cmp(&b.started_at_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    merged
}
