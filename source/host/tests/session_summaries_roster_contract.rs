use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-session-roster-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn production_roster_projects_profile_transcript_unread_and_settings() {
    let root = temp_root("summary");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let record = workers
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: " Agent ".into(),
                description: "desc".into(),
                title: "Title".into(),
                avatar_shape: "round".into(),
                avatar_color: "blue".into(),
            }),
            "dev",
            Some("research"),
        )
        .expect("materialize");
    workers
        .append_agent_transcript_entries(
            &record.id,
            &[json!({
                "id":"m1",
                "kind":"send-message",
                "message":{"type":"text","content":"hello **there**"}
            })],
        )
        .expect("append");
    workers.mark_agent_activity(&record.id, 100.0).expect("activity");
    workers.set_agent_unread(&record.id, true, 101.0).expect("unread");

    let summary = workers
        .summarize_agent_by_id(&record.id, Some(&record.id))
        .expect("summary")
        .expect("present");
    assert_eq!(summary.id, record.id);
    assert_eq!(summary.name, "Agent");
    assert_eq!(summary.title, "Title");
    assert_eq!(summary.last_message_id.as_deref(), Some("m1"));
    assert_eq!(summary.last_message_preview.as_deref(), Some("hello there"));
    assert!(summary.has_unread);
    assert_eq!(summary.origin, "dev");
    assert_eq!(summary.purpose.as_deref(), Some("research"));
    assert!(summary.is_active);

    let listed = workers
        .list_agent_summaries(Some(&record.id))
        .expect("list summaries");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, record.id);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
