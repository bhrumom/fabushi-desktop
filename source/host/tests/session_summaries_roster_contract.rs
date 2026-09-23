use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::{
    SandAgentProfile, write_sand_profile_file,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::storage::store_db::{
    register_live_db_handle, release_live_db_handle,
};
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


#[test]
fn production_roster_recovers_profile_only_and_quarantined_missing_databases() {
    let root = temp_root("missing-db");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);

    let profile_agent = root.join("profile-only");
    fs::create_dir_all(&profile_agent).expect("profile dir");
    write_sand_profile_file(
        profile_agent.join("profile.json"),
        &SandAgentProfile {
            name: "Recovered Profile".into(),
            description: "profile survives db loss".into(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        },
    )
    .expect("profile");
    assert!(!profile_agent.join("store.db").exists());

    let quarantined_agent = root.join("quarantined-only");
    fs::create_dir_all(&quarantined_agent).expect("quarantine dir");
    fs::write(
        quarantined_agent.join("store.db.corrupt-20260924"),
        b"quarantined",
    )
    .expect("quarantined marker");
    assert!(!quarantined_agent.join("store.db").exists());

    let listed = workers.list_agent_summaries(None).expect("recovered roster");
    let profile_summary = listed
        .iter()
        .find(|summary| summary.id == "profile-only")
        .expect("profile agent summary");
    assert_eq!(profile_summary.name, "Recovered Profile");
    assert!(profile_agent.join("store.db").is_file());

    assert!(listed.iter().any(|summary| summary.id == "quarantined-only"));
    assert!(quarantined_agent.join("store.db").is_file());

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_roster_does_not_reseed_missing_db_while_live_handle_is_registered() {
    let root = temp_root("live-handle");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let agent_dir = root.join("live-agent");
    fs::create_dir_all(&agent_dir).expect("agent dir");
    write_sand_profile_file(
        agent_dir.join("profile.json"),
        &SandAgentProfile {
            name: "Live Agent".into(),
            description: String::new(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        },
    )
    .expect("profile");
    let db_path = agent_dir.join("store.db");
    register_live_db_handle(&db_path);

    let summary = workers
        .summarize_agent_by_id("live-agent", None)
        .expect("summary")
        .expect("live summary");
    assert_eq!(summary.name, "Live Agent");
    assert!(!db_path.exists());

    release_live_db_handle(&db_path);
    let summary = workers
        .summarize_agent_by_id("live-agent", None)
        .expect("summary after release")
        .expect("recovered summary");
    assert_eq!(summary.name, "Live Agent");
    assert!(db_path.is_file());

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
