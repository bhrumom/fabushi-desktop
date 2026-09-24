use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::{
    SandAgentProfile, get_sand_profile_path, write_sand_profile_file,
};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionTranscriptRuntime;
use mahayana_host_runtime::extensions::transcript::roster_emit::ProductionRosterEmit;
use serde_json::Value;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-roster-emit-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn profile(name: &str) -> SandAgentProfile {
    SandAgentProfile {
        name: name.into(),
        description: "description".into(),
        title: String::new(),
        avatar_shape: String::new(),
        avatar_color: String::new(),
    }
}

#[test]
fn production_roster_emit_publishes_full_and_incremental_ordered_events() {
    let root = temp_root("events");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let record = sessions
        .materialize_new_session(Some(&profile("First")), "user", None)
        .expect("materialize agent");
    SandAgentSessionStore::new(Arc::clone(&sessions))
        .write_active_agent_id(&record.id)
        .expect("active pointer");

    let transcript = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let sink_events = Arc::clone(&events);
    let emitter = ProductionRosterEmit::new(
        Arc::clone(&sessions),
        transcript,
        Arc::new(move |event| sink_events.lock().expect("events").push(event)),
    );

    emitter.emit_agents().expect("full roster");
    assert!(emitter.cache_seeded());
    let first = events.lock().expect("events")[0].clone();
    assert_eq!(first["channel"], "agents");
    assert_eq!(first["payload"]["activeAgentId"], record.id);
    assert_eq!(first["payload"]["coverage"]["kind"], "complete-roster");
    assert_eq!(first["payload"]["ordered"]["replicaKey"], "roster");
    assert_eq!(first["payload"]["ordered"]["sequence"], 1);
    assert_eq!(first["payload"]["agents"][0]["name"], "First");
    assert!(first["payload"]["agents"][0]["snapshotEpoch"].is_string());

    write_sand_profile_file(
        get_sand_profile_path(root.join(&record.id)),
        &profile("Renamed"),
    )
    .expect("rename profile");
    emitter
        .emit_agent_update(&record.id)
        .expect("incremental update");
    let events = events.lock().expect("events");
    let second = events.last().expect("second event");
    assert_eq!(second["channel"], "agent-upserted");
    assert_eq!(second["payload"]["agent"]["id"], record.id);
    assert_eq!(second["payload"]["agent"]["name"], "Renamed");
    assert_eq!(second["payload"]["ordered"]["sequence"], 2);

    drop(events);
    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn incremental_emit_falls_back_to_full_roster_before_cache_seed() {
    let root = temp_root("fallback");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let record = sessions
        .materialize_new_session(Some(&profile("Agent")), "user", None)
        .expect("materialize agent");
    let transcript = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
    let events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let sink_events = Arc::clone(&events);
    let emitter = ProductionRosterEmit::new(
        Arc::clone(&sessions),
        transcript,
        Arc::new(move |event| sink_events.lock().expect("events").push(event)),
    );

    emitter
        .emit_agent_update(&record.id)
        .expect("fallback full");
    assert_eq!(events.lock().expect("events")[0]["channel"], "agents");

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
