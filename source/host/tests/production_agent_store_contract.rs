use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::{
    ProductionAgentMetadataKey, ProductionAgentStore,
};
use sha2::{Digest, Sha256};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-production-agent-store-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn production_agent_store_persists_content_addressed_checkpoint_and_resets_from_db() {
    let root = temp_root("checkpoint");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");

    assert!(session.agent_store.latest_root_blob_id().is_empty());
    assert_eq!(session.agent_store.latest_checkpoint_bytes(), None);

    // Empty bytes are the canonical protobuf encoding of an empty
    // ConversationStateStructure and generated fromBinary accepts them.
    let checkpoint = Vec::<u8>::new();
    let expected_root = Sha256::digest(&checkpoint).to_vec();
    let root_blob_id = session
        .agent_store
        .handle_checkpoint_bytes(&checkpoint)
        .expect("checkpoint");
    assert_eq!(root_blob_id, expected_root);
    assert_eq!(
        session.db.get_latest_root_blob_id().expect("metadata root"),
        expected_root
    );
    assert_eq!(
        session.agent_store.get_blob(&expected_root).expect("root blob"),
        Some(checkpoint.clone())
    );
    assert_eq!(
        session.agent_store.latest_checkpoint_bytes(),
        Some(checkpoint.clone())
    );

    let reopened = ProductionAgentStore::new(
        Arc::clone(&session.db),
        workers
            .create_agent_blob_store(&session.record.id)
            .expect("second blob store"),
    );
    assert!(reopened.try_reset_from_db());
    assert_eq!(reopened.latest_root_blob_id(), expected_root);
    assert_eq!(reopened.latest_checkpoint_bytes(), Some(checkpoint));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_agent_store_rejects_malformed_checkpoint_without_advancing_metadata() {
    let root = temp_root("malformed");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");

    let error = session
        .agent_store
        .handle_checkpoint_bytes(&[0x80])
        .expect_err("malformed protobuf must fail");
    assert!(error.contains("invalid ConversationStateStructure checkpoint"));
    assert!(
        session
            .db
            .get_latest_root_blob_id()
            .expect("metadata root")
            .is_empty()
    );

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_agent_store_metadata_bridge_gets_sets_and_subscribes() {
    let root = temp_root("metadata-bridge");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");

    assert_eq!(session.agent_store.get_id(), session.record.id);
    assert_eq!(
        session
            .agent_store
            .get_metadata(ProductionAgentMetadataKey::Mode)
            .expect("read mode"),
        Some(serde_json::Value::String("default".into()))
    );

    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let _subscription = session.agent_store.subscribe_to_metadata(
        ProductionAgentMetadataKey::Name,
        Arc::new(move |value| {
            capture.lock().expect("capture metadata").push(value);
        }),
    );

    assert!(
        session
            .agent_store
            .set_metadata(
                ProductionAgentMetadataKey::Name,
                serde_json::Value::String("Renamed Agent".into()),
            )
            .expect("set name")
    );
    assert_eq!(
        session
            .agent_store
            .get_metadata(ProductionAgentMetadataKey::Name)
            .expect("read name"),
        Some(serde_json::Value::String("Renamed Agent".into()))
    );
    assert_eq!(
        *seen.lock().expect("read captured metadata"),
        vec![Some(serde_json::Value::String("Renamed Agent".into()))]
    );

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
