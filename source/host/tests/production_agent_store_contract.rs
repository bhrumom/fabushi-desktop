use std::fs;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

fn encode_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

fn push_bytes(field: u64, value: &[u8], output: &mut Vec<u8>) {
    encode_varint((field << 3) | 2, output);
    encode_varint(value.len() as u64, output);
    output.extend_from_slice(value);
}

fn push_string(field: u64, value: &str, output: &mut Vec<u8>) {
    push_bytes(field, value.as_bytes(), output);
}

fn push_varint(field: u64, value: u64, output: &mut Vec<u8>) {
    encode_varint(field << 3, output);
    encode_varint(value, output);
}

fn subagent_state_bytes(created: u64, last_used: u64) -> Vec<u8> {
    let mut state = Vec::new();
    push_bytes(1, &[], &mut state);
    push_varint(2, created, &mut state);
    push_varint(3, last_used, &mut state);
    push_bytes(4, b"default", &mut state);
    state
}

fn map_entry(key: &str, value: &[u8]) -> Vec<u8> {
    let mut entry = Vec::new();
    push_string(1, key, &mut entry);
    push_bytes(2, value, &mut entry);
    entry
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


#[test]
fn production_agent_store_reads_latest_request_and_full_conversation_from_same_root() {
    let root = temp_root("conversation-read");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    let blob_store = workers
        .create_agent_blob_store(&session.record.id)
        .expect("blob store");

    let mut user = Vec::new();
    push_string(1, "hello from durable state", &mut user);
    push_string(2, "message-1", &mut user);
    let user_id = Sha256::digest(&user).to_vec();
    futures::executor::block_on(blob_store.set_blob(&(), &user_id, &user))
        .expect("user blob");

    let mut agent_turn = Vec::new();
    push_bytes(1, &user_id, &mut agent_turn);
    push_string(3, "request-42", &mut agent_turn);
    let mut turn = Vec::new();
    push_bytes(1, &agent_turn, &mut turn);
    let turn_id = Sha256::digest(&turn).to_vec();
    futures::executor::block_on(blob_store.set_blob(&(), &turn_id, &turn))
        .expect("turn blob");

    let mut checkpoint = Vec::new();
    push_bytes(8, &turn_id, &mut checkpoint);
    session
        .agent_store
        .handle_checkpoint_bytes(&checkpoint)
        .expect("checkpoint");

    assert_eq!(
        session
            .agent_store
            .get_last_request_id_from_conversation()
            .expect("last request id"),
        Some("request-42".into())
    );
    let conversation = session
        .agent_store
        .get_full_conversation()
        .expect("full conversation");
    assert_eq!(conversation.turns.len(), 1);
    assert!(conversation.todos.is_empty());
    assert_eq!(conversation.summary, None);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_agent_store_metadata_key_surface_matches_frozen_agent_store() {
    let keys = [
        ProductionAgentMetadataKey::AgentId,
        ProductionAgentMetadataKey::LatestRootBlobId,
        ProductionAgentMetadataKey::Name,
        ProductionAgentMetadataKey::Mode,
        ProductionAgentMetadataKey::IsRunEverything,
        ProductionAgentMetadataKey::ApprovalMode,
        ProductionAgentMetadataKey::CreatedAt,
        ProductionAgentMetadataKey::LastUsedModel,
        ProductionAgentMetadataKey::LastDebugServerPort,
        ProductionAgentMetadataKey::CurrentPlanUri,
        ProductionAgentMetadataKey::SubagentInfo,
        ProductionAgentMetadataKey::BlobEncryptionKey,
    ];
    assert_eq!(
        keys.map(ProductionAgentMetadataKey::as_str),
        [
            "agentId",
            "latestRootBlobId",
            "name",
            "mode",
            "isRunEverything",
            "approvalMode",
            "createdAt",
            "lastUsedModel",
            "lastDebugServerPort",
            "currentPlanUri",
            "subagentInfo",
            "blobEncryptionKey",
        ]
    );
}


#[test]
fn production_agent_store_subagent_refs_override_inline_and_missing_refs_fall_back_inline() {
    let root = temp_root("subagents");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    let blob_store = workers
        .create_agent_blob_store(&session.record.id)
        .expect("blob store");

    let referenced_state = subagent_state_bytes(20, 30);
    let referenced_id = Sha256::digest(&referenced_state).to_vec();
    futures::executor::block_on(blob_store.set_blob(
        &(),
        &referenced_id,
        &referenced_state,
    ))
    .expect("referenced subagent state");

    let missing_id = vec![0xAB; 32];
    let mut checkpoint = Vec::new();
    push_bytes(
        16,
        &map_entry("subagent-ref", &subagent_state_bytes(1, 2)),
        &mut checkpoint,
    );
    push_bytes(
        31,
        &map_entry("subagent-ref", &referenced_id),
        &mut checkpoint,
    );
    push_bytes(
        16,
        &map_entry("subagent-fallback", &subagent_state_bytes(7, 8)),
        &mut checkpoint,
    );
    push_bytes(
        31,
        &map_entry("subagent-fallback", &missing_id),
        &mut checkpoint,
    );
    session
        .agent_store
        .handle_checkpoint_bytes(&checkpoint)
        .expect("checkpoint");

    let (_, subagents) = session
        .agent_store
        .get_full_conversation_with_subagents()
        .expect("subagent hydration");
    assert_eq!(subagents.len(), 2);
    assert_eq!(subagents["subagent-ref"].created_timestamp_ms, 20);
    assert_eq!(subagents["subagent-ref"].last_used_timestamp_ms, 30);
    assert_eq!(subagents["subagent-fallback"].created_timestamp_ms, 7);
    assert_eq!(subagents["subagent-fallback"].last_used_timestamp_ms, 8);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_agent_store_missing_subagent_ref_without_inline_fails_closed() {
    let root = temp_root("missing-subagent");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");

    let mut checkpoint = Vec::new();
    push_bytes(
        31,
        &map_entry("subagent-missing", &[0xCD; 32]),
        &mut checkpoint,
    );
    session
        .agent_store
        .handle_checkpoint_bytes(&checkpoint)
        .expect("checkpoint");

    let error = session
        .agent_store
        .get_full_conversation_with_subagents()
        .expect_err("missing ref without inline fallback must fail");
    assert!(error.contains("subagent state ref blob not found"));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn constructor_and_legacy_reset_do_not_grant_canonical_write_authority() {
    let root = temp_root("canonical-authority");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers.materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    session.db.append_transcript_entry(&serde_json::json!({
        "id":"authority-user", "kind":"message", "role":"user", "content":"pending", "confirmed":false
    })).expect("pending user");
    let owner = ProductionAgentStore::new(Arc::clone(&session.db),
        workers.create_agent_blob_store(&session.record.id).expect("actual blob worker"));
    let mut checkpoint = Vec::new();
    push_string(99, "requires checked initialization", &mut checkpoint);
    let root_id = Sha256::digest(&checkpoint).to_vec();
    let before = session.db.read_kv("metadata").expect("metadata");
    assert!(owner.checkpoint_snapshot_checked().is_err());
    assert!(owner.handle_checkpoint_bytes(&checkpoint).is_err());
    assert!(owner.handle_checkpoint_bytes_and_confirm_user_message_blocking(&checkpoint, "authority-user").is_err());
    assert_eq!(owner.get_blob(&root_id).expect("no preauthorization blob write"), None);
    assert_eq!(session.db.read_kv("metadata").expect("metadata unchanged"), before);
    assert_eq!(session.db.get_transcript_entry_by_id("authority-user").expect("pending echo")
        .and_then(|entry| entry.get("confirmed").and_then(serde_json::Value::as_bool)), Some(false));

    assert!(!owner.reset_from_db_checked().expect("checked explicit no-root"));
    let empty_root = owner.handle_checkpoint_bytes(&[]).expect("initialized writer");
    assert!(owner.try_reset_from_db(), "legacy reset retains its successful boolean");
    assert_eq!(owner.latest_root_blob_id(), empty_root);
    assert_eq!(owner.latest_checkpoint_bytes(), Some(Vec::new()));
    assert!(owner.checkpoint_snapshot_checked().is_err(), "legacy reset cannot grant checked authority");
    assert!(owner.handle_checkpoint_bytes(&checkpoint).is_err());
    assert_eq!(owner.get_blob(&root_id).expect("still no unauthorized blob write"), None);
    assert!(owner.reset_from_db_checked().expect("checked reset restores authority"));
    assert_eq!(owner.handle_checkpoint_bytes(&checkpoint).expect("checked writer"), root_id);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn compatibility_missing_child_write_succeeds_but_failed_checked_reset_fences_future_writes() {
    let root = temp_root("missing-child-authority");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers.materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    let child = b"bounded direct child bytes";
    let child_id = Sha256::digest(child).to_vec();
    let mut checkpoint = Vec::new();
    push_bytes(8, &child_id, &mut checkpoint);
    let committed_root = session.agent_store.handle_checkpoint_bytes(&checkpoint)
        .expect("compatibility handler retains protobuf-only payload acceptance");
    assert_eq!(session.db.get_latest_root_blob_id().expect("committed root"), committed_root);
    assert_eq!(session.agent_store.latest_checkpoint_bytes(), Some(checkpoint.clone()));
    assert!(session.agent_store.checkpoint_snapshot_checked().is_err(), "canonical readers require the child");
    let compatible_replacement = vec![0x9a, 0x06, 0x03, b'n', b'e', b'w'];
    assert_eq!(session.agent_store.handle_checkpoint_bytes(&compatible_replacement)
        .expect("known durable base permits compatibility replacement despite missing children"),
        Sha256::digest(&compatible_replacement).to_vec());
    assert_eq!(session.agent_store.handle_checkpoint_bytes(&checkpoint)
        .expect("restore protobuf-only compatibility checkpoint"), committed_root);
    assert!(session.agent_store.reset_from_db_checked().is_err(), "failed strict reset is not empty readiness");
    let mut next = Vec::new();
    push_string(99, "must not be written after failed reset", &mut next);
    let next_id = Sha256::digest(&next).to_vec();
    assert!(session.agent_store.handle_checkpoint_bytes(&next).is_err());
    assert!(session.agent_store.handle_checkpoint_bytes_and_confirm_user_message_blocking(&next, "unused-user").is_err());
    assert_eq!(session.agent_store.get_blob(&next_id).expect("no failed-authority blob write"), None);
    assert_eq!(session.db.get_latest_root_blob_id().expect("root retained"), committed_root);

    workers.create_agent_blob_store(&session.record.id).expect("repair worker")
        .set_blob_blocking(&child_id, child).expect("repair required child");
    assert!(session.agent_store.reset_from_db_checked().expect("checked repair"));
    assert_eq!(session.agent_store.checkpoint_snapshot_checked().expect("checked snapshot").expect("root").checkpoint_bytes, checkpoint);
    assert_eq!(session.agent_store.handle_checkpoint_bytes(&next).expect("write after checked repair"), next_id);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn typed_root_metadata_rebinding_requires_checked_reset_before_new_checkpoint() {
    let root = temp_root("typed-root-rebind");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers.materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    let mut replacement = Vec::new();
    push_string(99, "typed replacement", &mut replacement);
    let replacement_id = Sha256::digest(&replacement).to_vec();
    workers.create_agent_blob_store(&session.record.id).expect("blob owner")
        .set_blob_blocking(&replacement_id, &replacement).expect("replacement bytes");
    let replacement_hex = replacement_id.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert!(session.agent_store.set_metadata(ProductionAgentMetadataKey::LatestRootBlobId,
        serde_json::json!(replacement_hex)).expect("typed root binding"));
    assert!(session.agent_store.checkpoint_snapshot_checked().is_err());
    assert!(session.agent_store.handle_checkpoint_bytes(&[]).is_err());
    assert_eq!(session.db.get_latest_root_blob_id().expect("typed persisted root"), replacement_id);
    assert!(session.agent_store.reset_from_db_checked().expect("checked rebind"));
    assert_eq!(session.agent_store.checkpoint_snapshot_checked().expect("checked snapshot").expect("root").checkpoint_bytes, replacement);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn synchronous_metadata_reentry_refuses_mutation_without_blocking_or_clearing_active_cache() {
    let root = temp_root("metadata-reentry");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let session = workers.materialize_session_with_active(None, "user", None, None)
        .expect("materialized session");
    let initial_root = session.agent_store.handle_checkpoint_bytes(&[]).expect("initial checkpoint");
    let weak = Arc::downgrade(&session.agent_store);
    let (callback_tx, callback_rx) = mpsc::channel();
    let subscription = session.agent_store.subscribe_to_metadata(
        ProductionAgentMetadataKey::LatestRootBlobId,
        Arc::new(move |_| {
            let owner = weak.upgrade().expect("outer owner remains alive");
            let observed = (
                owner.checkpoint_snapshot_checked().is_err(),
                owner.reset_from_db_checked().is_err(),
                owner.handle_checkpoint_bytes(&[]).is_err(),
                owner.handle_checkpoint_bytes_and_confirm_user_message_blocking(&[], "reentrant-user").is_err(),
                owner.try_reset_from_db(),
                owner.latest_root_blob_id(),
                owner.latest_checkpoint_bytes(),
            );
            let _ = callback_tx.send(observed);
        }),
    );
    let mut checkpoint = Vec::new();
    push_string(99, "outer checkpoint must finish", &mut checkpoint);
    let expected = Sha256::digest(&checkpoint).to_vec();
    let outer = Arc::clone(&session.agent_store);
    let (done_tx, done_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = outer.handle_checkpoint_bytes(&checkpoint);
        let _ = done_tx.send(result);
    });
    let completion = done_rx.recv_timeout(Duration::from_secs(5));
    if completion.is_err() {
        // Never join a wedged callback or drop/close its live DB on timeout.
        // The contract fails promptly while the test process retains custody.
        std::mem::forget(subscription);
        std::mem::forget(session);
        std::mem::forget(workers);
        panic!("real metadata callback reentry blocked the checkpoint owner");
    }
    assert_eq!(completion.expect("bounded completion").expect("outer write succeeds"), expected);
    worker.join().expect("finished checkpoint thread");
    let (read_refused, reset_refused, write_refused, atomic_refused, legacy_reset, visible_root, visible_bytes) =
        callback_rx.recv_timeout(Duration::from_secs(1)).expect("real root metadata listener ran");
    assert!(read_refused && reset_refused && write_refused && atomic_refused);
    assert!(!legacy_reset);
    assert_eq!(visible_root, initial_root);
    assert_eq!(visible_bytes, Some(Vec::new()));
    assert_eq!(session.agent_store.checkpoint_snapshot_checked().expect("published cache remains ready").expect("root").root_blob_id, expected);
    assert_eq!(session.db.get_latest_root_blob_id().expect("durable outer root"), expected);
    drop(subscription);
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
