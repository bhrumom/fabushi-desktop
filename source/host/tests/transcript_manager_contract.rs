use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;

fn temp_root() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-transcript-manager-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn manager_is_the_single_production_composition_owner() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));

    assert!(Arc::ptr_eq(&manager.session_workers(), &sessions));
    let runtime_a = manager.transcript_runtime();
    let runtime_b = manager.transcript_runtime();
    assert!(Arc::ptr_eq(&runtime_a, &runtime_b));

    let runners_a = manager.runner_registry();
    let runners_b = manager.runner_registry();
    assert!(Arc::ptr_eq(&runners_a, &runners_b));
    assert_eq!(runners_a.active_count(), 0);

    let ack_a = manager.ack_obligations();
    let ack_b = manager.ack_obligations();
    assert!(Arc::ptr_eq(&ack_a, &ack_b));
    ack_a.record_send("agent-a", 1.0).expect("durable ack");
    assert_eq!(ack_b.pending_obligations().len(), 1);

    assert!(!manager.is_disposed());
    manager.dispose();
    assert!(manager.is_disposed());
    manager.dispose();
    assert_eq!(runners_a.active_count(), 0);

    let _ = fs::remove_dir_all(root);
}
