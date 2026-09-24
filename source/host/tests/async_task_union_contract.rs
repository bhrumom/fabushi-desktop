use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::transcript::async_task_union::{
    AsyncTask, LEDGER_ONLY_DETAIL, marker_label, merge_async_tasks,
    pending_wake_marker_to_async_task,
};
use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionTranscriptRuntime;
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    DurablePendingWakeMarker, PendingWakeKind,
};

fn marker(
    agent_id: &str,
    kind: PendingWakeKind,
    work_id: &str,
    marked_at_ms: f64,
    title: Option<&str>,
    subagent_type: Option<&str>,
) -> DurablePendingWakeMarker {
    DurablePendingWakeMarker {
        agent_id: agent_id.into(),
        kind,
        work_id: work_id.into(),
        marked_at_ms,
        quiet_origin: None,
        title: title.map(str::to_string),
        subagent_type: subagent_type.map(str::to_string),
        interrupted_by_recreate: false,
    }
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-async-task-union-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn frozen_marker_projection_preserves_labels_detail_and_frontend_shape() {
    let cloud = marker(
        "agent-a",
        PendingWakeKind::CloudAgent,
        "bc-1",
        20.0,
        None,
        None,
    );
    assert_eq!(marker_label(&cloud), "Cloud agent bc-1");
    let task = pending_wake_marker_to_async_task(&cloud);
    assert_eq!(task.kind, "cloud-agent");
    assert_eq!(task.id, "bc-1");
    assert_eq!(task.status, "running");
    assert_eq!(task.detail.as_deref(), Some(LEDGER_ONLY_DETAIL));

    let subagent = marker(
        "agent-a",
        PendingWakeKind::Subagent,
        "task-1",
        10.0,
        Some("Research"),
        Some("cursor-agent"),
    );
    let task = pending_wake_marker_to_async_task(&subagent);
    assert_eq!(task.label, "Research");
    assert_eq!(
        task.detail.as_deref(),
        Some("cursor-agent · from the durable pending-wake ledger")
    );
    assert_eq!(
        serde_json::to_value(&task).expect("serialize task"),
        serde_json::json!({
            "kind": "subagent",
            "id": "task-1",
            "label": "Research",
            "status": "running",
            "startedAtMs": 10.0,
            "detail": "cursor-agent · from the durable pending-wake ledger"
        })
    );
}

#[test]
fn merge_deduplicates_live_ownership_and_sorts_by_start_then_id() {
    let live = vec![AsyncTask {
        kind: "cloud-agent".into(),
        id: "same".into(),
        label: "live owner".into(),
        status: "running".into(),
        started_at_ms: 30.0,
        detail: None,
    }];
    let markers = vec![
        marker(
            "agent-a",
            PendingWakeKind::CloudAgent,
            "same",
            5.0,
            Some("ledger duplicate"),
            None,
        ),
        marker(
            "agent-a",
            PendingWakeKind::Shell,
            "shell-b",
            10.0,
            None,
            None,
        ),
        marker(
            "agent-a",
            PendingWakeKind::Subagent,
            "task-a",
            10.0,
            None,
            None,
        ),
    ];
    let merged = merge_async_tasks(&live, &markers);
    assert_eq!(
        merged
            .iter()
            .map(|task| format!("{}:{}", task.kind, task.id))
            .collect::<Vec<_>>(),
        vec![
            "shell:shell-b",
            "subagent:task-a",
            "cloud-agent:same",
        ]
    );
    assert_eq!(merged[2].label, "live owner");
}

#[test]
fn shipping_runtime_projects_durable_pending_wakes_for_get_async_tasks() {
    let root = temp_root("runtime");
    fs::create_dir_all(&root).expect("root");
    let runtime = ProductionTranscriptRuntime::new(Some(&root));
    let store = runtime.pending_wake_store().expect("pending wake store");

    assert!(store.mark_pending(marker(
        "agent-a",
        PendingWakeKind::Shell,
        "shell-7",
        40.0,
        Some("Install dependencies"),
        None,
    )));
    assert!(store.mark_pending(marker(
        "agent-b",
        PendingWakeKind::Subagent,
        "task-other",
        5.0,
        None,
        None,
    )));

    let tasks = runtime.get_async_tasks("agent-a", &[]);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].kind, "shell");
    assert_eq!(tasks[0].id, "shell-7");
    assert_eq!(tasks[0].label, "Install dependencies");

    let _ = fs::remove_dir_all(root);
}
