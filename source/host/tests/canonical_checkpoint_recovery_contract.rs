//! CKR-v1 exercises persisted corruption through real Session, DB, blob-worker
//! and shipping Host owners. These are bounded direct-reference contracts,
//! not evidence of provider disk-resume or real-service compatibility.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::agent_db::SandAgentDb;
use mahayana_host_runtime::extensions::session::conversation_recovery::MAX_ROOT_BLOB_BYTES;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_paths::{
    CONVERSATION_BLOBS_FILENAME, STORE_FILENAME,
};
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::TurnRunOptions;
use mahayana_host_runtime::runner::production_agent_checkpoint::build_text_turn_checkpoint;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_field(field: u64, value: &[u8]) -> Vec<u8> {
    fn varint(mut value: u64, output: &mut Vec<u8>) {
        while value >= 128 {
            output.push(value as u8 | 0x80);
            value >>= 7;
        }
        output.push(value as u8);
    }
    let mut output = Vec::new();
    varint(field << 3 | 2, &mut output);
    varint(value.len() as u64, &mut output);
    output.extend_from_slice(value);
    output
}

struct Fixture {
    root: PathBuf,
    agent_id: String,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("clock").as_nanos();
        let root = std::env::temp_dir().join(format!(
            "fabushi-canonical-checkpoint-{label}-{}-{suffix}", std::process::id(),
        ));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None)
            .expect("actual Session materialization");
        let agent_id = session.record.id.clone();
        workers.shutdown();
        drop(session);
        drop(workers);
        Self { root, agent_id }
    }

    fn workers(&self) -> Arc<ProductionSessionWorkers> {
        Arc::new(ProductionSessionWorkers::with_agents_root(self.root.join("agents"), 500))
    }

    fn db_path(&self) -> PathBuf {
        self.root.join("agents").join(&self.agent_id).join(STORE_FILENAME)
    }

    fn blob_path(&self) -> PathBuf {
        self.root.join("agents").join(&self.agent_id).join(CONVERSATION_BLOBS_FILENAME)
    }

    fn metadata(&self) -> Value {
        let db = Connection::open(self.db_path()).expect("read fixture metadata");
        let raw: String = db.query_row("SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0))
            .expect("materialized metadata");
        let bytes = raw.as_bytes().chunks_exact(2).map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("fixture hex"), 16)
                .expect("fixture hex byte")
        }).collect::<Vec<_>>();
        serde_json::from_slice(&bytes).expect("materialized metadata JSON")
    }

    fn replace_metadata_raw(&self, value: Option<&str>) {
        let db = Connection::open(self.db_path()).expect("fault injection DB");
        match value {
            Some(value) => { db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![value]).expect("metadata fault"); }
            None => { db.execute("DELETE FROM kv WHERE key='metadata'", []).expect("absent metadata fault"); }
        }
    }

    fn replace_metadata(&self, value: &Value) {
        self.replace_metadata_raw(Some(&hex(&serde_json::to_vec(value).expect("metadata JSON"))));
    }

    fn bind_root(&self, root: &[u8]) {
        let mut metadata = self.metadata();
        metadata["latestRootBlobId"] = json!(hex(root));
        self.replace_metadata(&metadata);
    }

    fn write_blob(&self, id: &[u8], bytes: &[u8]) {
        let workers = self.workers();
        workers.create_agent_blob_store(&self.agent_id).expect("actual blob worker")
            .set_blob_blocking(id, bytes).expect("persist blob through actual worker");
        workers.shutdown();
    }

    fn make_blob_read_fail(&self, id: &[u8]) {
        // A TEXT value in an old/non-STRICT table is a deterministic SQLite
        // column decoding failure, unlike filesystem permissions under root.
        let db = Connection::open(self.blob_path()).expect("fault injection blob DB");
        db.execute_batch("ALTER TABLE blobs RENAME TO blobs_before_fault;
            CREATE TABLE blobs (id TEXT PRIMARY KEY, data);
            INSERT INTO blobs SELECT id, data FROM blobs_before_fault;
            DROP TABLE blobs_before_fault;")
            .expect("inject legacy blob storage shape");
        db.execute("UPDATE blobs SET data='unreadable-as-bytes' WHERE id=?1", params![hex(id)])
            .expect("inject blob read failure");
    }

    fn composition(&self, workers: &Arc<ProductionSessionWorkers>) -> HostRunnerComposition {
        HostRunnerComposition::production(
            Arc::new(SandLocalToolPermissionController::production(Arc::new(
                SettingsService::new(self.root.join("settings.json")),
            ))),
            Arc::clone(workers),
        )
    }

    fn finish(self, workers: Arc<ProductionSessionWorkers>) {
        workers.shutdown();
        drop(workers);
        fs::remove_dir_all(self.root).expect("remove closed fixture");
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DurableState {
    kv: Vec<(String, String)>,
    transcript: Vec<(i64, String, String)>,
    blobs: Vec<(String, String, String)>,
    mirror: Vec<(PathBuf, Vec<u8>)>,
}

fn files_under(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    if !path.exists() { return Vec::new(); }
    let mut files = Vec::new();
    for entry in fs::read_dir(path).expect("read mirror directory") {
        let path = entry.expect("mirror entry").path();
        if path.is_dir() { files.extend(files_under(&path)); }
        else { files.push((path.clone(), fs::read(path).expect("read mirror file"))); }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn durable_state(fixture: &Fixture) -> DurableState {
    let db = Connection::open(fixture.db_path()).expect("inspect durable DB");
    let kv = db.prepare("SELECT key,value FROM kv ORDER BY key").expect("kv query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?))).expect("kv rows")
        .collect::<Result<Vec<_>, _>>().expect("kv values");
    let transcript = db.prepare("SELECT seq,id,entry FROM transcript_entries ORDER BY seq").expect("transcript query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).expect("transcript rows")
        .collect::<Result<Vec<_>, _>>().expect("transcript values");
    let blobs = if fixture.blob_path().exists() {
        let blobs = Connection::open(fixture.blob_path()).expect("inspect durable blobs");
        let rows = blobs.prepare("SELECT id,typeof(data),hex(data) FROM blobs ORDER BY id").expect("blob query")
            .query_map([], |row| {
                let encoded: String = row.get(2)?;
                Ok((row.get(0)?, row.get(1)?, hex(&Sha256::digest(encoded.as_bytes()))))
            }).expect("blob rows")
            .collect::<Result<Vec<_>, _>>().expect("blob values");
        rows
    } else { Vec::new() };
    DurableState { kv, transcript, blobs, mirror: files_under(&fixture.root.join("transcripts")) }
}

fn assert_host_refuses(fixture: &Fixture, workers: &Arc<ProductionSessionWorkers>) {
    let before = durable_state(fixture);
    let host = fixture.composition(workers);
    assert!(host.confirmed_user_turn_watermark(workers, &fixture.agent_id).is_err(), "watermark must propagate canonical failure");
    assert!(host.prompt_compaction_epoch(workers, &fixture.agent_id).is_err(), "prompt epoch must propagate canonical failure");
    assert!(host.compose_production_checkpoint_sink(workers, &fixture.root, &fixture.agent_id,
        false, Arc::new(|_| {}), Arc::new(|| Ok(true))).is_err(), "checkpoint composition must propagate canonical failure");
    assert_eq!(durable_state(fixture), before, "refusal must not replace metadata, commit a root, or append a transcript mirror checkpoint");
}

#[test]
fn fresh_actual_owners_distinguish_no_root_from_persisted_empty_protobuf() {
    let fixture = Fixture::new("empty");
    let workers = fixture.workers();
    let store = workers.open_agent_store_owner(&fixture.agent_id).expect("explicit no-root open");
    assert!(store.checkpoint_snapshot_checked().expect("no-root snapshot").is_none());
    let host = fixture.composition(&workers);
    assert!(!host.confirmed_user_turn_watermark(&workers, &fixture.agent_id).expect("empty watermark").has_user_turn);
    assert_eq!(host.prompt_compaction_epoch(&workers, &fixture.agent_id).expect("empty epoch"), 0);
    let empty_root = store.handle_checkpoint_bytes(&[]).expect("persist empty protobuf");
    assert_eq!(empty_root, Sha256::digest([]).to_vec());
    workers.shutdown();
    drop(store);
    drop(host);
    drop(workers);

    let mut metadata = fixture.metadata();
    metadata["latestRootBlobId"] = json!(format!("  {}  ", hex(&empty_root).to_uppercase()));
    fixture.replace_metadata(&metadata);
    let reopened = fixture.workers();
    let store = reopened.open_agent_store_owner(&fixture.agent_id).expect("fresh owner reopen");
    let snapshot = store.checkpoint_snapshot_checked().expect("checked empty protobuf").expect("persisted empty blob remains Some");
    assert_eq!(snapshot.root_blob_id, empty_root);
    assert!(snapshot.checkpoint_bytes.is_empty());
    drop(store);
    fixture.finish(reopened);
}

#[test]
fn shipping_checkpoint_round_trip_preserves_generated_turn_and_unknown_wire_fields() {
    let fixture = Fixture::new("generated");
    let workers = fixture.workers();
    let store = workers.open_agent_store_owner(&fixture.agent_id).expect("owner");
    let unknown = bytes_field(99, b"future-field-must-survive");
    store.handle_checkpoint_bytes(&unknown).expect("opaque prior field checkpoint");
    let db = workers.open_agent_db_owner(&fixture.agent_id).expect("DB owner");
    db.append_transcript_entry(&json!({"id":"ckr-user", "kind":"message", "role":"user", "content":"preserve me", "confirmed":false}))
        .expect("stage actual user transcript");
    let host = fixture.composition(&workers);
    let sink = host.compose_production_checkpoint_sink(&workers, &fixture.root, &fixture.agent_id,
        false, Arc::new(|_| {}), Arc::new(|| Ok(true))).expect("shipping composition").expect("sink");
    sink.checkpoint_text_turn(&[ProviderMessage { role: "user".into(), content: "preserve me".into() }],
        &TurnRunOptions { message_id: Some("ckr-user".into()), inference_request_id: Some("ckr-request".into()),
            recent_message_text: Some("preserve me".into()), ..TurnRunOptions::default() }, "durable reply")
        .expect("shipping checkpoint commit");
    let committed = store.checkpoint_snapshot_checked().expect("committed snapshot").expect("root");
    assert!(committed.checkpoint_bytes.windows(unknown.len()).any(|window| window == unknown.as_slice()));
    assert_eq!(committed.root_blob_id, Sha256::digest(&committed.checkpoint_bytes).to_vec());
    let mirror = files_under(&fixture.root.join("transcripts"));
    assert!(!mirror.is_empty(), "actual mirror must be populated by shipping checkpoint");
    workers.shutdown();
    drop((sink, host, store, db, workers));

    let reopened = fixture.workers();
    let store = reopened.open_agent_store_owner(&fixture.agent_id).expect("fresh generated checkpoint owner");
    let recovered = store.checkpoint_snapshot_checked().expect("recovered snapshot").expect("recovered root");
    assert_eq!(recovered.root_blob_id, committed.root_blob_id);
    assert_eq!(recovered.checkpoint_bytes, committed.checkpoint_bytes);
    let host = fixture.composition(&reopened);
    let watermark = host.confirmed_user_turn_watermark(&reopened, &fixture.agent_id).expect("recovered watermark");
    assert!(watermark.has_user_turn);
    assert_eq!(watermark.last_user_message_id.as_deref(), Some("ckr-user"));
    assert_eq!(files_under(&fixture.root.join("transcripts")), mirror);
    drop((host, store));
    fixture.finish(reopened);
}

#[test]
fn corrupt_metadata_never_becomes_an_empty_session_or_gets_seeded_on_reopen() {
    for case in ["absent", "hex", "non-ascii-hex", "json", "non-object", "missing-root", "non-string-root", "root-hex", "root-non-ascii", "root-odd"] {
        let fixture = Fixture::new(case);
        let mut metadata = fixture.metadata();
        match case {
            "absent" => fixture.replace_metadata_raw(None),
            "hex" => fixture.replace_metadata_raw(Some("zz")),
            "non-ascii-hex" => fixture.replace_metadata_raw(Some("aéx")),
            "json" => fixture.replace_metadata_raw(Some(&hex(b"{"))),
            "non-object" => fixture.replace_metadata(&json!([])),
            "missing-root" => { metadata.as_object_mut().expect("object").remove("latestRootBlobId"); fixture.replace_metadata(&metadata); }
            "non-string-root" => { metadata["latestRootBlobId"] = json!(7); fixture.replace_metadata(&metadata); }
            "root-hex" => { metadata["latestRootBlobId"] = json!("zz"); fixture.replace_metadata(&metadata); }
            "root-non-ascii" => { metadata["latestRootBlobId"] = json!("aéx"); fixture.replace_metadata(&metadata); }
            "root-odd" => { metadata["latestRootBlobId"] = json!("abc"); fixture.replace_metadata(&metadata); }
            _ => unreachable!(),
        }
        let before = durable_state(&fixture);
        let db = SandAgentDb::open_existing_agent(fixture.db_path(), 500).expect("open existing DB without seeding");
        assert!(db.read_checkpoint_root_checked().is_err(), "{case}");
        db.close(false);
        drop(db);
        let workers = fixture.workers();
        for _ in 0..2 {
            assert!(workers.open_agent_store_owner(&fixture.agent_id).is_err(), "{case}: failed open must not cache a valid empty owner");
            assert!(workers.prepare_existing_agent(&fixture.agent_id).is_err(), "{case}: preparation must not repair malformed metadata");
        }
        assert_host_refuses(&fixture, &workers);
        assert_eq!(durable_state(&fixture), before, "{case}: persisted corruption is retained");
        fixture.finish(workers);
    }
}

#[test]
fn both_root_cas_operations_refuse_absent_or_non_string_binding_without_confirming_user() {
    for case in ["missing-metadata", "missing-root", "non-string-root"] {
        let fixture = Fixture::new(case);
        let workers = fixture.workers();
        let owner = workers.open_agent_db_owner(&fixture.agent_id).expect("DB owner");
        owner.append_transcript_entry(&json!({"id":"cas-user", "kind":"message", "role":"user", "content":"pending", "confirmed":false})).expect("pending user");
        let mut metadata = fixture.metadata();
        match case {
            "missing-metadata" => fixture.replace_metadata_raw(None),
            "missing-root" => { metadata.as_object_mut().expect("object").remove("latestRootBlobId"); fixture.replace_metadata(&metadata); }
            _ => { metadata["latestRootBlobId"] = json!(false); fixture.replace_metadata(&metadata); }
        }
        let before = durable_state(&fixture);
        assert!(!owner.compare_and_set_latest_root_blob_id(&[], &[0x55; 32]).expect("known CAS noncommit"));
        assert!(owner.commit_checkpoint_root_and_confirm_user_message(&[], &[0x66; 32], "cas-user").expect("known atomic noncommit").is_none());
        assert_eq!(durable_state(&fixture), before, "{case}: no metadata rewrite or confirmation");
        drop(owner);
        fixture.finish(workers);
    }
}

#[test]
fn bad_root_blobs_refuse_shipping_readiness_and_repeated_open_without_publication() {
    for case in ["missing-root-blob", "corrupt-fixed-root", "invalid-protobuf", "root-read-error"] {
        let fixture = Fixture::new(case);
        let bytes = if case == "invalid-protobuf" { vec![0x80] } else { bytes_field(99, b"persisted root") };
        let root = if case == "corrupt-fixed-root" { vec![0x88; 32] } else { Sha256::digest(&bytes).to_vec() };
        if case != "missing-root-blob" {
            fixture.write_blob(&root, if case == "corrupt-fixed-root" { &[0x80] } else { &bytes });
        }
        if case == "root-read-error" { fixture.make_blob_read_fail(&root); }
        fixture.bind_root(&root);
        let workers = fixture.workers();
        for _ in 0..2 { assert!(workers.open_agent_store_owner(&fixture.agent_id).is_err(), "{case}"); }
        assert_host_refuses(&fixture, &workers);
        fixture.finish(workers);
    }
}

#[test]
fn required_direct_refs_are_checked_by_actual_owner_not_only_root_protobuf() {
    // Each required group has a missing-reference negative. The additional
    // cases distinguish empty identity and actual read failures.
    for (field, fault) in [(1, "missing"), (3, "missing"), (6, "missing"), (8, "missing"), (13, "missing"),
        (1, "empty-id"), (3, "read-error"), (13, "read-error")] {
        let fixture = Fixture::new(&format!("ref-{field}-{fault}"));
        let child = b"actual bounded direct reference bytes";
        let child_id = if fault == "empty-id" { Vec::new() } else { Sha256::digest(child).to_vec() };
        if fault == "read-error" { fixture.write_blob(&child_id, child); }
        let checkpoint = bytes_field(field, &child_id);
        let root_id = Sha256::digest(&checkpoint).to_vec();
        fixture.write_blob(&root_id, &checkpoint);
        if fault == "read-error" { fixture.make_blob_read_fail(&child_id); }
        fixture.bind_root(&root_id);
        let workers = fixture.workers();
        assert!(workers.open_agent_store_owner(&fixture.agent_id).is_err(), "field {field}: {fault}");
        assert_host_refuses(&fixture, &workers);
        fixture.finish(workers);
    }
}

#[test]
fn failed_open_can_be_retried_after_real_blob_repair_and_after_fresh_workers_reopen() {
    let fixture = Fixture::new("repair-retry");
    let checkpoint = bytes_field(99, b"original retained bytes");
    let root = Sha256::digest(&checkpoint).to_vec();
    fixture.bind_root(&root);
    let workers = fixture.workers();
    for _ in 0..2 { assert!(workers.open_agent_store_owner(&fixture.agent_id).is_err()); }
    assert_host_refuses(&fixture, &workers);
    workers.create_agent_blob_store(&fixture.agent_id).expect("repair blob owner")
        .set_blob_blocking(&root, &checkpoint).expect("repair exact missing root");
    let repaired = workers.open_agent_store_owner(&fixture.agent_id).expect("retry must build valid owner, not reuse an empty failure");
    let snapshot = repaired.checkpoint_snapshot_checked().expect("repaired snapshot").expect("root");
    assert_eq!(snapshot.root_blob_id, root);
    assert_eq!(snapshot.checkpoint_bytes, checkpoint);
    workers.shutdown();
    drop((workers, repaired));
    let reopened = fixture.workers();
    let owner = reopened.open_agent_store_owner(&fixture.agent_id).expect("fresh repaired owner");
    let snapshot = owner.checkpoint_snapshot_checked().expect("fresh snapshot").expect("root");
    assert_eq!(snapshot.checkpoint_bytes, checkpoint);
    drop(owner);
    fixture.finish(reopened);
}

#[test]
fn closed_db_and_cached_binding_drift_refuse_canonical_readiness() {
    let fixture = Fixture::new("binding-drift");
    let workers = fixture.workers();
    let store = workers.open_agent_store_owner(&fixture.agent_id).expect("initialized owner");
    let db = workers.open_agent_db_owner(&fixture.agent_id).expect("DB owner");
    store.handle_checkpoint_bytes(&[]).expect("initial checkpoint");
    let changed = bytes_field(99, b"external replacement");
    let changed_root = Sha256::digest(&changed).to_vec();
    workers.create_agent_blob_store(&fixture.agent_id).expect("blob worker")
        .set_blob_blocking(&changed_root, &changed).expect("replacement bytes");
    fixture.bind_root(&changed_root);
    assert!(store.checkpoint_snapshot_checked().is_err(), "captured cache cannot silently follow changed metadata");
    assert_host_refuses(&fixture, &workers);
    assert!(store.reset_from_db_checked().expect("explicit checked reload"));
    assert_eq!(store.checkpoint_snapshot_checked().expect("checked snapshot").expect("root").checkpoint_bytes, changed);
    db.close(false);
    assert!(db.read_checkpoint_root_checked().is_err(), "closed DB is not an empty root");
    assert!(store.checkpoint_snapshot_checked().is_err());
    drop((db, store));
    fixture.finish(workers);
}

#[test]
fn already_composed_shipping_sink_rechecks_before_staging_and_before_persisting() {
    let fixture = Fixture::new("sink-rechecks");
    let workers = fixture.workers();
    let owner = workers.open_agent_store_owner(&fixture.agent_id).expect("owner");
    owner.handle_checkpoint_bytes(&[]).expect("valid prior");
    let db = workers.open_agent_db_owner(&fixture.agent_id).expect("DB owner");
    db.append_transcript_entry(&json!({"id":"sink-user", "kind":"message", "role":"user", "content":"pending", "confirmed":false}))
        .expect("unconfirmed user");
    let host = fixture.composition(&workers);
    let sink = host.compose_production_checkpoint_sink(&workers, &fixture.root, &fixture.agent_id,
        false, Arc::new(|_| {}), Arc::new(|| Ok(true))).expect("valid composition").expect("sink");
    let messages = [ProviderMessage { role: "user".into(), content: "pending".into() }];
    let options = TurnRunOptions { message_id: Some("sink-user".into()),
        inference_request_id: Some("sink-request".into()), ..TurnRunOptions::default() };
    let expected_base = sink.checked_base_snapshot().expect("capture valid canonical base");
    let staged = sink.stage_text_turn(&expected_base, &messages, &options, "must not commit").expect("stage while canonical root is valid");
    let mut metadata = fixture.metadata();
    metadata.as_object_mut().expect("object").remove("latestRootBlobId");
    fixture.replace_metadata(&metadata);
    let before = durable_state(&fixture);
    assert!(sink.stage_text_turn(&expected_base, &messages, &options, "must not stage").is_err());
    assert!(sink.persist_staged_text_turn(&expected_base, &options, &staged).is_err());
    assert_eq!(durable_state(&fixture), before, "neither comparison may publish a blob, confirm the user, advance a root or journal a checkpoint");
    drop((sink, host, owner, db));
    fixture.finish(workers);
}

#[test]
fn preparation_validates_selected_recovery_candidate_before_cas_and_does_not_skip_it() {
    for fault in ["missing-summary", "missing-todo", "shared-oversize-todo"] {
        let fixture = Fixture::new(fault);
        let generated = build_text_turn_checkpoint(&[], "recover user", "recover-id", Some("recover-request"), "recover reply");
        fixture.write_blob(&generated.user_message_id, &generated.user_message_bytes);
        for (id, bytes) in generated.step_ids.iter().zip(&generated.step_bytes) { fixture.write_blob(id, bytes); }
        fixture.write_blob(&generated.turn_id, &generated.turn_bytes);
        // The lower-scoring valid candidate must not be silently selected
        // after the scanner chooses the corrupt root with an extra prompt.
        let fallback_id = Sha256::digest(&generated.state_bytes).to_vec();
        fixture.write_blob(&fallback_id, &generated.state_bytes);
        let shared = if fault == "shared-oversize-todo" { vec![b'x'; MAX_ROOT_BLOB_BYTES + 1] } else { b"prompt bytes".to_vec() };
        let prompt_id = Sha256::digest(&shared).to_vec();
        fixture.write_blob(&prompt_id, &shared);
        let mut candidate = generated.state_bytes;
        candidate.extend(bytes_field(1, &prompt_id));
        if fault == "missing-summary" { candidate.extend(bytes_field(6, &[0x55; 32])); }
        if fault == "missing-todo" { candidate.extend(bytes_field(3, &[0x44; 32])); }
        if fault == "shared-oversize-todo" { candidate.extend(bytes_field(3, &prompt_id)); }
        let candidate_id = Sha256::digest(&candidate).to_vec();
        fixture.write_blob(&candidate_id, &candidate);
        let before = durable_state(&fixture);
        let workers = fixture.workers();
        assert!(workers.prepare_existing_agent(&fixture.agent_id).is_err(), "selected {fault} candidate must fail before CAS");
        assert_eq!(durable_state(&fixture), before, "{fault}: no root repair or transcript backfill after selected corruption");
        assert_eq!(fixture.metadata()["latestRootBlobId"], json!(""));
        fixture.finish(workers);
    }
}

#[test]
fn preparation_recovers_real_generated_orphan_then_fresh_owner_opens_the_same_root() {
    let fixture = Fixture::new("valid-orphan-repair");
    let generated = build_text_turn_checkpoint(&[], "recover user", "recover-id", Some("recover-request"), "recover reply");
    fixture.write_blob(&generated.user_message_id, &generated.user_message_bytes);
    for (id, bytes) in generated.step_ids.iter().zip(&generated.step_bytes) { fixture.write_blob(id, bytes); }
    fixture.write_blob(&generated.turn_id, &generated.turn_bytes);
    let root = Sha256::digest(&generated.state_bytes).to_vec();
    fixture.write_blob(&root, &generated.state_bytes);
    let workers = fixture.workers();
    let prepared = workers.prepare_existing_agent(&fixture.agent_id).expect("valid orphan recovery").expect("existing session");
    assert_eq!(prepared.persisted_root_blob_id, root);
    assert_eq!(workers.open_agent_store_owner(&fixture.agent_id).expect("recovered owner")
        .checkpoint_snapshot_checked().expect("checked repaired checkpoint").expect("root").checkpoint_bytes, generated.state_bytes);
    workers.shutdown();
    drop(workers);
    let reopened = fixture.workers();
    let owner = reopened.open_agent_store_owner(&fixture.agent_id).expect("fresh repaired Session owner");
    let snapshot = owner.checkpoint_snapshot_checked().expect("fresh repaired snapshot").expect("root");
    assert_eq!(snapshot.root_blob_id, root);
    assert_eq!(snapshot.checkpoint_bytes, generated.state_bytes);
    drop(owner);
    fixture.finish(reopened);
}

#[test]
fn legacy_fixed_roots_and_direct_references_remain_readable_and_same_id_drift_refuses_reentry() {
    for root_length in [3, 32] {
        let fixture = Fixture::new(&format!("fixed-ids-{root_length}"));
        let root = vec![0x88; root_length];
        let mut checkpoint = bytes_field(99, b"unknown fields remain intact");
        for (field, length) in [(1, 1), (3, 2), (6, 17), (8, 32), (13, 33)] {
            let id = vec![field as u8; length];
            fixture.write_blob(&id, b"retained fixed reference bytes");
            checkpoint.extend(bytes_field(field, &id));
        }
        fixture.write_blob(&root, &checkpoint);
        fixture.bind_root(&root);
        let workers = fixture.workers();
        let owner = workers.open_agent_store_owner(&fixture.agent_id).expect("fixed IDs remain legal");
        let snapshot = owner.checkpoint_snapshot_checked().expect("available fixed references").expect("root");
        assert_eq!(snapshot.root_blob_id, root);
        assert_eq!(snapshot.checkpoint_bytes, checkpoint);
        let replacement = bytes_field(99, b"different valid bytes under the same fixed ID");
        let blobs = workers.create_agent_blob_store(&fixture.agent_id).expect("actual blob owner");
        blobs.set_blob_blocking(&root, &replacement).expect("out-of-band fixed-root drift");
        assert!(owner.checkpoint_snapshot_checked().is_err(), "matching ID does not prove cached bytes remain durable");
        assert_host_refuses(&fixture, &workers);
        assert_eq!(owner.latest_checkpoint_bytes(), Some(checkpoint.clone()));
        blobs.set_blob_blocking(&root, &checkpoint).expect("explicit original-byte repair");
        assert!(workers.open_agent_store_owner(&fixture.agent_id).is_ok());
        let db = Connection::open(fixture.blob_path()).expect("inject exact root deletion");
        db.execute("DELETE FROM blobs WHERE id=?1", params![hex(&root)]).expect("delete only captured root");
        drop(db);
        assert!(owner.checkpoint_snapshot_checked().is_err(), "a cached root cannot conceal a deleted durable blob");
        assert_host_refuses(&fixture, &workers);
        blobs.set_blob_blocking(&root, &checkpoint).expect("restore exact root");
        assert_eq!(owner.checkpoint_snapshot_checked().expect("repaired availability"), Some(snapshot));
        drop((owner, blobs));
        fixture.finish(workers);
    }
}
