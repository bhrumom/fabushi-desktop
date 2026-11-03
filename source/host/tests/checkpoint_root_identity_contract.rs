//! SWA-ROOT-RECOVERY-v1: accepted legacy root spellings remain writable while
//! malformed roots cannot authorize either checkpoint CAS edge.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::ProductionAgentStore;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::TurnRunOptions;
use mahayana_host_runtime::storage::store_db::get_sand_agent_db_write_generation;
use rusqlite::params;
use serde_json::json;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct Fixture {
    root: PathBuf,
    workers: ProductionSessionWorkers,
    agent_id: String,
    db_path: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-checkpoint-root-identity-{label}-{}", uuid::Uuid::new_v4(),
        ));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None)
            .expect("materialize actual Session");
        let agent_id = session.record.id.clone();
        let db_path = session.record.db_path.clone();
        Self { root, workers, agent_id, db_path }
    }

    fn use_fixed_root(&self, store: &ProductionAgentStore, root: &[u8], state: &[u8]) {
        self.workers.create_agent_blob_store(&self.agent_id).expect("existing blob owner")
            .set_blob_blocking(root, state).expect("retained fixed-ID blob");
        let db = self.workers.open_agent_db_owner(&self.agent_id).expect("existing DB owner");
        assert!(db.compare_and_set_latest_root_blob_id(
            &store.latest_root_blob_id_checked().expect("captured root"), root,
        ).expect("explicit fixed-root binding"));
        assert!(store.reset_from_db_checked().expect("checked fixed-root reload"));
    }

    fn normalize_root_spelling(&self, root: &[u8]) {
        let owner = self.workers.open_agent_db_owner(&self.agent_id).expect("actual DB owner");
        let mut metadata = owner.read_metadata().expect("existing metadata");
        metadata["latestRootBlobId"] = json!(format!(" \n{}\t", hex(root).to_ascii_uppercase()));
        let raw = hex(&serde_json::to_vec(&metadata).expect("metadata encoding"));
        // Raw SQL preserves an imported legacy spelling. The actual writes
        // under test still use the Session/AgentStore and shipping sink owners.
        let db = rusqlite::Connection::open(&self.db_path).expect("existing metadata database");
        assert_eq!(db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![raw])
            .expect("persist legacy root spelling"), 1);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.workers.shutdown();
        if !std::thread::panicking() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[test]
fn normalized_ordinary_and_fixed_roots_continue_through_both_checkpoint_owners() {
    for fixed in [false, true] {
        let fixture = Fixture::new(if fixed { "fixed" } else { "content-addressed" });
        let initial = vec![0x98, 0x06, 0x01]; // unknown protobuf field 99 = 1
        let store = fixture.workers.open_agent_store_owner(&fixture.agent_id).expect("AgentStore");
        store.handle_checkpoint_bytes(&initial).expect("initial checkpoint");
        if fixed {
            fixture.use_fixed_root(&store, &[0xaa, 0xbb, 0xcc], &initial);
        }
        let original_root = store.latest_root_blob_id_checked().expect("original root");
        fixture.normalize_root_spelling(&original_root);
        drop(store);
        assert!(fixture.workers.retire_agent_owners(&fixture.agent_id, true)
            .expect("close exact original owners"));

        let store = fixture.workers.open_agent_store_owner(&fixture.agent_id)
            .expect("normalized nonempty root is readable after reopen");
        assert_eq!(store.latest_root_blob_id_checked().expect("reopened root"), original_root);
        assert_eq!(store.latest_checkpoint_bytes_checked().expect("reopened bytes"), Some(initial.clone()));
        let mut next = initial;
        next.extend_from_slice(&[0x98, 0x06, 0x02]); // retain prior opaque bytes
        let generic_root = store.handle_checkpoint_bytes(&next)
            .expect("generic checkpoint CAS compares decoded root identity");
        assert_ne!(generic_root, original_root);

        if fixed {
            fixture.use_fixed_root(&store, &[0xde, 0xad, 0xba, 0xbe], &next);
        }
        let atomic_base = store.latest_root_blob_id_checked().expect("atomic base root");
        fixture.normalize_root_spelling(&atomic_base);
        let db = fixture.workers.open_agent_db_owner(&fixture.agent_id).expect("live transcript owner");
        db.append_transcript_entry(&json!({
            "id":"normalized-user", "kind":"message", "role":"user",
            "content":"continue from the retained checkpoint", "confirmed":false,
        })).expect("durable pending user echo");
        let host = HostRunnerComposition::with_sink(
            Arc::new(SandLocalToolPermissionController::production(Arc::new(
                SettingsService::new(fixture.root.join("settings.json")),
            ))), Arc::new(|_| {}),
        );
        let sink = host.compose_production_checkpoint_sink(
            &fixture.workers, &fixture.root, &fixture.agent_id, false,
            Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).expect("actual shipping checkpoint composition").expect("main-session sink");
        sink.checkpoint_text_turn(
            &[ProviderMessage { role: "user".into(), content: "continue from the retained checkpoint".into() }],
            &TurnRunOptions {
                message_id: Some("normalized-user".into()),
                inference_request_id: Some("normalized-request".into()),
                recent_message_text: Some("continue from the retained checkpoint".into()),
                ..TurnRunOptions::default()
            },
            "continued and committed",
        ).expect("atomic root and user-confirmation CAS compares decoded root identity");
        assert_eq!(db.get_transcript_entry_by_id("normalized-user").expect("user echo")
            .expect("retained user echo")["confirmed"], json!(true));
        assert_ne!(store.latest_root_blob_id_checked().expect("successor root"), atomic_base);
        assert!(store.latest_checkpoint_bytes_checked().expect("successor bytes")
            .expect("present successor").starts_with(&next));
        host.dispose();
    }
}

#[test]
fn malformed_root_strings_refuse_both_cas_edges_without_mutation_or_notification() {
    let cases: &[(&str, &[u8])] = &[
        (" ", &[]), ("\t\n", &[]), ("xx", &[]), ("a", &[]),
        ("éé", &[]), ("🙂", &[]),
        ("+a", &[0x0a]), ("+A", &[0x0a]), ("+0", &[0x00]),
        ("ab+c", &[0xab, 0x0c]),
    ];
    for &(malformed, expected_root) in cases {
        let fixture = Fixture::new("malformed");
        if !expected_root.is_empty() {
            let store = fixture.workers.open_agent_store_owner(&fixture.agent_id)
                .expect("valid owner before malformed spelling");
            fixture.use_fixed_root(&store, expected_root, &[]);
        }
        let owner = fixture.workers.open_agent_db_owner(&fixture.agent_id).expect("DB owner");
        assert_eq!(owner.get_latest_root_blob_id().expect("admitted original root"), expected_root);
        owner.append_transcript_entry(&json!({
            "id":"pending-user", "kind":"message", "role":"user",
            "content":"must remain pending", "confirmed":false,
        })).expect("pending user echo");
        let valid_raw = owner.read_kv("metadata").expect("metadata read").expect("metadata");
        let mut metadata = owner.read_metadata().expect("metadata object");
        metadata["latestRootBlobId"] = json!(malformed);
        let malformed_raw = hex(&serde_json::to_vec(&metadata).expect("malformed metadata encoding"));
        let writer = rusqlite::Connection::open(&fixture.db_path).expect("existing metadata DB");
        writer.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![malformed_raw])
            .expect("inject exact raw root string");
        let before = owner.get_transcript_entries().expect("transcript before refusal");
        let generation = get_sand_agent_db_write_generation(&fixture.db_path);
        let hits = Arc::new(AtomicUsize::new(0));
        let _subscription = owner.subscribe_metadata("latestRootBlobId", {
            let hits = Arc::clone(&hits);
            Arc::new(move || { hits.fetch_add(1, Ordering::SeqCst); })
        });
        assert!(!matches!(owner.compare_and_set_latest_root_blob_id(expected_root, &[0xaa, 0xbb]), Ok(true)),
            "malformed spelling cannot match an admitted root: {malformed:?}");
        assert!(!matches!(owner.commit_checkpoint_root_and_confirm_user_message(
            expected_root, &[0xcc, 0xdd], "pending-user",
        ), Ok(Some(_))), "malformed root cannot confirm a user: {malformed:?}");
        assert_eq!(owner.read_kv("metadata").expect("metadata after refusal"), Some(malformed_raw));
        assert_eq!(owner.get_transcript_entries().expect("transcript after refusal"), before);
        assert_eq!(get_sand_agent_db_write_generation(&fixture.db_path), generation);
        assert_eq!(hits.load(Ordering::SeqCst), 0);

        writer.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![valid_raw])
            .expect("explicit exact metadata repair");
        assert!(owner.compare_and_set_latest_root_blob_id(expected_root, &[0xaa, 0xbb])
            .expect("the original valid root remains writable after repair"));
        assert!(owner.commit_checkpoint_root_and_confirm_user_message(
            &[0xaa, 0xbb], &[0xcc, 0xdd], "pending-user",
        ).expect("fixed-root CAS after explicit repair").is_some());
        assert_eq!(get_sand_agent_db_write_generation(&fixture.db_path), generation + 2);
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }
}
