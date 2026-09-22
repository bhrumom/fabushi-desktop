use std::path::Path;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::{
    agents::settings_file::{get_sand_settings_path, read_sand_settings_file, write_sand_settings_file, DEFAULT_HIDDEN_FROM_SIDEBAR, DEFAULT_NOTIFY_ON_AGENT_UPDATES},
    extensions::{
        box_store_sync::{box_store_diagnostics::{pin_box_store_diagnostics_reporter, report_box_store_diagnostic}, box_store_sync_error::SandBoxStoreSyncError},
        cloud_agents::cloud_agent_launch_error::SandCloudAgentLaunchError,
        session::conversation_blobs_path::conversation_blobs_path,
        transcript::{channel_delivery_unregistered_error::SandChannelDeliveryUnregisteredError, send_not_persisted_error::SandSendNotPersistedError},
    },
    notify_drain_gate::NotifyDrainGate,
    storage::agent_paths::{assert_valid_sand_agent_id, resolve_sand_agent_dir},
};
use serde_json::{Map, Value};
use uuid::Uuid;

#[test]
fn notify_drain_gate_matches_notify_floor_and_safety_poll_semantics() {
    use std::cell::Cell;
    let now = Cell::new(1_000_u64);
    let mut gate = NotifyDrainGate::new(|| now.get(), || true, || true);
    assert!(gate.should_drain(false));
    gate.record_poll();
    assert!(!gate.should_drain(false));
    gate.record_notify();
    now.set(4_999);
    assert!(!gate.should_drain(false));
    now.set(5_000);
    assert!(gate.should_drain(false));
}

#[test]
fn agent_paths_reject_traversal_and_whitespace() {
    assert!(assert_valid_sand_agent_id("agent-1").is_ok());
    for bad in ["../x", " x", "x ", "a/b", "."] { assert!(assert_valid_sand_agent_id(bad).is_err()); }
    let dir = resolve_sand_agent_dir("agent-1", Some(Path::new("/tmp/home"))).unwrap();
    assert_eq!(dir.file_name().unwrap(), "agent-1");
}

#[test]
fn settings_file_uses_defaults_preserves_unknown_fields_and_atomic_replace() {
    let root = std::env::temp_dir().join(format!("fabushi-settings-{}", Uuid::new_v4()));
    let path = get_sand_settings_path(&root);
    let defaults = read_sand_settings_file(&path);
    assert_eq!(defaults.notify_on_agent_updates, DEFAULT_NOTIFY_ON_AGENT_UPDATES);
    assert_eq!(defaults.hidden_from_sidebar, DEFAULT_HIDDEN_FROM_SIDEBAR);
    let mut first = Map::new(); first.insert("unknown".into(), Value::String("keep".into())); first.insert("hiddenFromSidebar".into(), Value::Bool(true));
    write_sand_settings_file(&path, &first).unwrap();
    let mut update = Map::new(); update.insert("notifyOnAgentUpdates".into(), Value::Bool(false));
    write_sand_settings_file(&path, &update).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"unknown\": \"keep\""));
    let settings = read_sand_settings_file(&path);
    assert!(settings.hidden_from_sidebar);
    assert!(!settings.notify_on_agent_updates);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn session_and_error_contracts_match_reference_messages() {
    assert_eq!(conversation_blobs_path("/tmp/agent/store.db"), Path::new("/tmp/agent/conversation-blobs.db"));
    assert_eq!(SandChannelDeliveryUnregisteredError.to_string(), "No channel delivery mechanism is registered.");
    assert!(SandSendNotPersistedError.to_string().contains("client retry is not swallowed"));
    assert_eq!(SandBoxStoreSyncError::new("sync failed").to_string(), "sync failed");
    assert_eq!(SandCloudAgentLaunchError::new("launch failed").to_string(), "launch failed");
}

#[test]
fn box_store_diagnostics_reporter_can_be_pinned_and_cleared() {
    let seen = Arc::new(Mutex::new(0_u32));
    let sink = Arc::clone(&seen);
    pin_box_store_diagnostics_reporter(Some(Arc::new(move |_| *sink.lock().unwrap() += 1)));
    report_box_store_diagnostic(&Map::new());
    pin_box_store_diagnostics_reporter(None);
    assert_eq!(*seen.lock().unwrap(), 1);
}
