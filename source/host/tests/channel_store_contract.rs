use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::channel_store::{
    FileChannelStore, get_agent_channels_dir, label_for,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-channel-store-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn labels_match_frozen_grok_clamping_and_connector_defaults() {
    assert_eq!(label_for("slack", None), "Slack");
    assert_eq!(label_for("discord", Some("  ")), "Discord");
    assert_eq!(label_for("matrix", None), "matrix");
    assert_eq!(
        label_for("slack", Some("  Team   Operations  ")),
        "Team Operations"
    );
    assert_eq!(
        label_for("slack", Some(&"x".repeat(100))).chars().count(),
        80
    );
}

#[test]
fn file_channel_store_writes_lists_and_removes_safe_platforms() {
    let root = temp_root("file");
    let store = FileChannelStore::new(root.join("channels"));
    let changes = Arc::new(AtomicUsize::new(0));
    store.set_on_change(Some({
        let changes = Arc::clone(&changes);
        Arc::new(move || {
            changes.fetch_add(1, Ordering::SeqCst);
        })
    }));

    assert!(!store.write_metadata("../escape", "bad").expect("invalid platform"));
    assert!(store.write_metadata("slack", "").expect("write slack"));
    assert!(store.write_metadata("discord", "  Community   Bot ").expect("write discord"));
    assert_eq!(store.read_label("slack").as_deref(), Some("Slack"));
    assert_eq!(store.read_label("discord").as_deref(), Some("Community Bot"));
    assert_eq!(store.list_platforms(), vec!["discord".to_string(), "slack".to_string()]);
    let connections = store.list_connections();
    assert_eq!(connections.len(), 2);
    assert_eq!(connections[0].status, "configured");
    assert_eq!(changes.load(Ordering::SeqCst), 2);

    assert!(store.remove("slack").expect("remove slack"));
    assert!(!store.remove("slack").expect("remove absent slack"));
    assert_eq!(store.list_platforms(), vec!["discord".to_string()]);
    assert_eq!(changes.load(Ordering::SeqCst), 3);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_connector_credentials_materialize_channel_metadata_and_filter_on_token() {
    let root = temp_root("production");
    let agents = root.join("agents");
    let runtime = ProductionSessionWorkers::with_agents_root(&agents, 500);
    let record = runtime
        .materialize_new_session(None, "user", None)
        .expect("materialize session");

    assert!(runtime
        .store_connector_credential(&record.id, "slack", "token", "secret-token")
        .expect("store token"));
    assert_eq!(
        fs::read_to_string(
            get_agent_channels_dir(&agents.join(&record.id))
                .join("slack")
                .join("connection.json")
        )
        .expect("channel metadata"),
        "{\n  \"label\": \"Slack\"\n}\n"
    );
    let channels = runtime.list_agent_channels(&record.id).expect("list channels");
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].platform, "slack");
    assert_eq!(channels[0].label, "Slack");

    assert!(runtime
        .store_connector_credential(&record.id, "discord", "clientId", "id-only")
        .expect("store non-token secret"));
    assert_eq!(
        runtime.list_agent_channels(&record.id).expect("filtered channels").len(),
        1
    );

    assert!(runtime.disconnect_channel(&record.id, "slack").expect("disconnect"));
    assert!(runtime.list_agent_channels(&record.id).expect("after disconnect").is_empty());
    assert_eq!(
        runtime.get_connector_secret(&record.id, "slack", "token").expect("secret read"),
        None
    );

    runtime.shutdown();
    let _ = fs::remove_dir_all(root);
}
