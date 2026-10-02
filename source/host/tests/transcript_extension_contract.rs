use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::extension::{
    TRANSCRIPT_EXTENSION_DEPENDENCIES, TRANSCRIPT_EXTENSION_ID, TranscriptExtensionEventBridge,
    start_transcript_extension,
};
use mahayana_host_runtime::host_event_bus::SandHostEventBus;

fn temp_root() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-transcript-extension-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn extension_composes_manager_roster_and_profile_watch_lifecycle() {
    let root = temp_root();
    let agents = root.join("agents");
    fs::create_dir_all(&agents).expect("agents root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = Arc::clone(&events);
    let extension = start_transcript_extension(
        &root,
        Arc::clone(&sessions),
        Arc::new(move |event| {
            sink_events.lock().expect("events").push(event);
        }),
    );

    assert_eq!(TRANSCRIPT_EXTENSION_ID, "transcript");
    assert_eq!(
        TRANSCRIPT_EXTENSION_DEPENDENCIES,
        &[
            "attachments",
            "content-search",
            "memory",
            "session",
            "telemetry",
            "trays",
            "turn-execution",
        ]
    );
    let manager = extension.manager();
    assert!(Arc::ptr_eq(&manager.session_workers(), &sessions));
    assert_eq!(extension.roster_emit().outline_stream_coalescing_ms(), 250);
    extension.roster_emit().emit_agents().expect("emit empty roster");
    assert_eq!(
        events
            .lock()
            .expect("events")
            .last()
            .and_then(|event| event.get("channel"))
            .and_then(serde_json::Value::as_str),
        Some("agents")
    );
    // Watch setup is fail-open in production. A platform watcher can be
    // unavailable, but the failure must remain inspectable instead of silently
    // replacing the authoritative roster path.
    assert!(
        extension.profile_watch_active() || extension.profile_watch_error().is_some()
    );
    assert!(!manager.is_disposed());
    drop(extension);
    assert!(manager.is_disposed());

    let _ = fs::remove_dir_all(root);
}


#[test]
fn extension_event_bridge_emits_frozen_host_topics_with_identity_payloads() {
    let bus = SandHostEventBus::default();
    let observed = Arc::new(Mutex::new(Vec::<(String, serde_json::Value)>::new()));
    let mut subscriptions = Vec::new();
    for topic in [
        "transcript.automation-config-changed",
        "transcript.listener-connect-card",
        "transcript.run-started",
        "transcript.run-ended",
    ] {
        let observed = Arc::clone(&observed);
        let topic_owned = topic.to_string();
        subscriptions.push(bus.on(topic, move |payload| {
            observed
                .lock()
                .expect("observed events")
                .push((topic_owned.clone(), payload.clone()));
            Ok(())
        }));
    }
    let bridge = TranscriptExtensionEventBridge::new(bus);

    bridge.automation_config_changed();
    bridge.listener_connect_card("agent-a", "slack");
    bridge.run_started("request-a");
    bridge.run_ended("request-a");

    let observed = observed.lock().expect("observed events");
    assert_eq!(observed.len(), 4);
    assert_eq!(
        observed[0],
        (
            "transcript.automation-config-changed".to_string(),
            serde_json::json!({})
        )
    );
    assert_eq!(
        observed[1],
        (
            "transcript.listener-connect-card".to_string(),
            serde_json::json!({"agentId":"agent-a","platform":"slack"})
        )
    );
    assert_eq!(
        observed[2],
        (
            "transcript.run-started".to_string(),
            serde_json::json!({"requestId":"request-a"})
        )
    );
    assert_eq!(
        observed[3],
        (
            "transcript.run-ended".to_string(),
            serde_json::json!({"requestId":"request-a"})
        )
    );
    drop(subscriptions);
}
