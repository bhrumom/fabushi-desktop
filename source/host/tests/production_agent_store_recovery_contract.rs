use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::agent_db::{PendingProfileMutation, ProfileFileSnapshot};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::ProductionWorkerBlobStore;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    TextTurnCheckpointArtifacts, build_text_turn_checkpoint,
};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn temp_root(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-root-recovery-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn encoded_metadata(value: Value) -> String {
    hex(&serde_json::to_vec(&value).expect("encode metadata"))
}

fn rewrite_raw_metadata(path: &Path, raw: Option<&str>) {
    let db = Connection::open(path).expect("fixture database");
    match raw {
        Some(raw) => {
            db.execute(
                "INSERT INTO kv (key, value) VALUES ('metadata', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![raw],
            )
            .expect("fixture metadata overwrite");
        }
        None => {
            db.execute("DELETE FROM kv WHERE key = 'metadata'", [])
                .expect("fixture metadata removal");
        }
    }
}

fn durable_rows(path: &Path) -> (Vec<(String, String)>, Vec<(i64, String, String)>) {
    let db = Connection::open(path).expect("inspect durable database");
    let mut kv_query = db.prepare("SELECT key, value FROM kv ORDER BY key").expect("kv query");
    let kv = kv_query
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("kv rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect kv");
    let mut transcript_query = db
        .prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq")
        .expect("transcript query");
    let transcript = transcript_query
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("transcript rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect transcript");
    (kv, transcript)
}

fn composition(root: &Path) -> HostRunnerComposition {
    HostRunnerComposition::with_sink(
        Arc::new(SandLocalToolPermissionController::production(Arc::new(
            SettingsService::new(root.join("host-settings.json")),
        ))),
        Arc::new(|_| {}),
    )
}

fn write_turn_blobs(store: &ProductionWorkerBlobStore, turn: &TextTurnCheckpointArtifacts) {
    store
        .set_blob_blocking(&turn.user_message_id, &turn.user_message_bytes)
        .expect("user blob");
    for (id, bytes) in turn.step_ids.iter().zip(&turn.step_bytes) {
        store.set_blob_blocking(id, bytes).expect("step blob");
    }
    store
        .set_blob_blocking(&turn.turn_id, &turn.turn_bytes)
        .expect("turn blob");
}

#[test]
fn composed_sink_refuses_durable_root_corruption_before_child_blobs_or_mirror_preparation() {
    use std::collections::BTreeMap;
    use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
    use mahayana_host_runtime::runner::TurnRunOptions;

    fn files_and_directories(path: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
        let mut snapshot = BTreeMap::new();
        if !path.exists() {
            return snapshot;
        }
        snapshot.insert(path.to_path_buf(), None);
        for entry in fs::read_dir(path).expect("transcript directory") {
            let entry = entry.expect("transcript entry");
            let path = entry.path();
            if entry.file_type().expect("transcript entry type").is_dir() {
                snapshot.extend(files_and_directories(&path));
            } else {
                snapshot.insert(path.clone(), Some(fs::read(path).expect("transcript bytes")));
            }
        }
        snapshot
    }

    for case in ["missing", "null", "missing-blob", "corrupt-blob", "valid-drift"] {
        let root = temp_root(&format!("sink-before-write-{case}"));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None)
            .expect("initialized session");
        session.db.append_transcript_entry(&json!({
            "id": "staged-user", "kind": "message", "role": "user",
            "content": "preserve this pending request", "confirmed": false,
        })).expect("pending user");
        let host = composition(&root);
        let sink = host.compose_production_checkpoint_sink(
            &workers, &root, &session.record.id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).expect("compose valid initial state").expect("main checkpoint sink");
        let messages = [ProviderMessage {
            role: "user".into(), content: "preserve this pending request".into(),
        }];
        let options = TurnRunOptions {
            message_id: Some("staged-user".into()),
            inference_request_id: Some("staged-request".into()),
            recent_message_text: Some("preserve this pending request".into()),
            ..TurnRunOptions::default()
        };
        let expected_base = sink.checked_base_snapshot().expect("capture valid canonical base");
        let staged = sink.stage_text_turn(&expected_base, &messages, &options, "recovered assistant answer")
            .expect("stage while base is valid");
        let valid_metadata = session.db.read_kv("metadata").expect("metadata read").expect("metadata");
        let mut damaged = session.db.read_metadata().expect("metadata object");
        let blob_store = workers.create_agent_blob_store(&session.record.id).expect("blob owner");
        match case {
            "missing" => { damaged.as_object_mut().expect("metadata object").remove("latestRootBlobId"); }
            "null" => { damaged["latestRootBlobId"] = Value::Null; }
            "missing-blob" => { damaged["latestRootBlobId"] = json!("eeee"); }
            "corrupt-blob" => {
                blob_store.set_blob_blocking(&[0xee, 0xee], &[0x80]).expect("corrupt durable base");
                damaged["latestRootBlobId"] = json!("eeee");
            }
            "valid-drift" => {
                blob_store.set_blob_blocking(&[0xee, 0xee], &[]).expect("valid other durable base");
                damaged["latestRootBlobId"] = json!("eeee");
            }
            _ => unreachable!(),
        }
        rewrite_raw_metadata(&session.record.db_path, Some(&encoded_metadata(damaged)));
        let durable_before = durable_rows(&session.record.db_path);
        let transcript_path = root.join("transcripts");
        let mirror_before = files_and_directories(&transcript_path);
        let mut staged_ids = vec![
            staged.user_message_id.clone(), staged.turn_id.clone(),
            Sha256::digest(&staged.state_bytes).to_vec(),
        ];
        staged_ids.extend(staged.step_ids.clone());
        for id in &staged_ids {
            assert_eq!(blob_store.get_blob_blocking(id).expect("unpublished staged blob"), None);
        }

        assert!(sink.stage_text_turn(&expected_base, &messages, &options, "must refuse new staging").is_err(), "{case}");
        let error = sink.persist_staged_text_turn(&expected_base, &options, &staged)
            .err().expect("damaged or drifted durable root refuses publication");
        assert!(error.to_string().contains("production AgentStore"), "{case}: {error}");
        for id in &staged_ids {
            assert_eq!(blob_store.get_blob_blocking(id).expect("no rejected child or state blob"), None, "{case}");
        }
        assert_eq!(durable_rows(&session.record.db_path), durable_before, "{case}: no root or confirmation write");
        assert_eq!(files_and_directories(&transcript_path), mirror_before, "{case}: no mirror preparation");
        assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("retained empty snapshot"), None);

        rewrite_raw_metadata(&session.record.db_path, Some(&valid_metadata));
        sink.persist_staged_text_turn(&expected_base, &options, &staged).expect("explicit repair permits original staged commit");
        assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("committed state"), Some(staged.state_bytes.clone()));
        assert_eq!(session.db.get_transcript_entry_by_id("staged-user").expect("confirmed entry")
            .and_then(|entry| entry.get("confirmed").and_then(Value::as_bool)), Some(true));
        let mirror = fs::read_to_string(transcript_path.join(&session.record.id)
            .join(format!("{}.jsonl", session.record.id))).expect("committed transcript mirror");
        assert!(mirror.contains("preserve this pending request"));
        assert!(mirror.contains("recovered assistant answer"));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn checked_production_open_distinguishes_initialized_empty_from_present_empty_checkpoint() {
    let root = temp_root("explicit-empty");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("new session");
    assert!(!session.agent_store.reset_from_db_checked().expect("explicit empty root"));
    assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("empty state"), None);

    let checkpoint_root = session.agent_store.handle_checkpoint_bytes(&[]).expect("empty protobuf");
    assert_eq!(checkpoint_root, Sha256::digest(b"").to_vec());
    workers.retire_agent_owners(&session.record.id, false).expect("retire owners");
    let reopened = workers.open_agent_store_owner(&session.record.id).expect("reopen checkpoint");
    assert_eq!(reopened.latest_root_blob_id_checked().expect("root"), checkpoint_root);
    assert_eq!(reopened.latest_checkpoint_bytes_checked().expect("present state"), Some(Vec::new()));
    assert!(reopened.reset_from_db_checked().expect("present checkpoint"));
    assert!(reopened.try_reset_from_db(), "legacy bool still accepts empty protobuf");
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn malformed_metadata_is_rejected_before_seeding_repair_or_owner_cache_and_can_be_repaired() {
    for case in [
        "absent", "raw-hex", "raw-unicode", "json", "nonobject", "missing-agent",
        "wrong-agent", "missing-root", "null-root", "number-root", "odd-root",
        "nonhex-root", "unicode-root",
    ] {
        let root = temp_root(case);
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let record = workers
            .materialize_new_session_with_active(None, "user", None, None)
            .expect("initialized metadata");
        let db = workers.open_agent_db_owner(&record.id).expect("db owner");
        db.append_transcript_entry(&json!({
            "id": "pending-user", "kind": "message", "role": "user",
            "content": "preserve pending request", "confirmed": false,
        }))
        .expect("pending transcript");
        let valid_raw = db.read_kv("metadata").expect("metadata read").expect("metadata");
        let mut metadata = db.read_metadata().expect("valid metadata");
        let malformed = match case {
            "absent" => None,
            "raw-hex" => Some("zz".to_string()),
            "raw-unicode" => Some("ø".to_string()),
            "json" => Some(hex(b"{")),
            "nonobject" => Some(encoded_metadata(json!([]))),
            other => {
                match other {
                    "missing-agent" => { metadata.as_object_mut().expect("object").remove("agentId"); }
                    "wrong-agent" => { metadata["agentId"] = json!("another-agent"); }
                    "missing-root" => { metadata.as_object_mut().expect("object").remove("latestRootBlobId"); }
                    "null-root" => { metadata["latestRootBlobId"] = Value::Null; }
                    "number-root" => { metadata["latestRootBlobId"] = json!(17); }
                    "odd-root" => { metadata["latestRootBlobId"] = json!("f"); }
                    "nonhex-root" => { metadata["latestRootBlobId"] = json!("zz"); }
                    "unicode-root" => { metadata["latestRootBlobId"] = json!("ø"); }
                    _ => unreachable!(),
                }
                Some(encoded_metadata(metadata))
            }
        };
        workers.retire_agent_owners(&record.id, false).expect("retire before restart");
        let agent_dir = record.db_path.parent().expect("agent directory");
        fs::remove_file(agent_dir.join("profile.json")).expect("remove recoverable profile");
        fs::remove_file(agent_dir.join("settings.json")).expect("remove recoverable settings");
        rewrite_raw_metadata(&record.db_path, malformed.as_deref());
        let before = durable_rows(&record.db_path);

        let error = workers.open_agent_store_owner(&record.id).err().expect("bad root must refuse");
        assert!(error.contains("production AgentStore"), "{case}: {error}");
        assert!(workers.prepare_existing_agent(&record.id).is_err(), "{case}");
        assert_eq!(durable_rows(&record.db_path), before, "{case}: no metadata/transcript repair");
        assert_eq!(workers.active_agent_store_owner_count(), 0, "{case}: no empty cache");
        assert_eq!(workers.active_agent_db_owner_count(), 0, "{case}: no invalid DB owner cache");
        assert_eq!(workers.active_worker_count(), 0, "{case}: no recovery scan");
        assert!(!agent_dir.join("profile.json").exists(), "{case}: profile repair must wait");
        assert!(!agent_dir.join("settings.json").exists(), "{case}: settings repair must wait");

        rewrite_raw_metadata(&record.db_path, Some(&valid_raw));
        let repaired = workers.open_agent_store_owner(&record.id).expect("explicitly repaired metadata");
        assert_eq!(repaired.latest_checkpoint_bytes_checked().expect("repaired empty root"), None);
        assert!(workers.prepare_existing_agent(&record.id).expect("valid repair").is_some());
        assert!(agent_dir.join("profile.json").is_file());
        assert!(agent_dir.join("settings.json").is_file());
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn production_consumers_refuse_missing_or_corrupt_checkpoint_and_recover_after_repair() {
    for corrupt_blob in [false, true] {
        let root = temp_root(if corrupt_blob { "bad-blob" } else { "missing-blob" });
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers
            .materialize_session_with_active(None, "user", None, None)
            .expect("session");
        let store = workers.create_agent_blob_store(&session.record.id).expect("blob owner");
        let turn = build_text_turn_checkpoint(None, "retained question", "message-original", Some("request-original"), "retained answer");
        write_turn_blobs(&store, &turn);
        let root_id = Sha256::digest(&turn.state_bytes).to_vec();
        if corrupt_blob {
            store.set_blob_blocking(&root_id, &[0x80]).expect("damaged root fixture");
        }
        assert!(session.db.compare_and_set_latest_root_blob_id(&[], &root_id).expect("published root fixture"));
        workers.retire_agent_owners(&session.record.id, false).expect("restart owners");
        let before = durable_rows(&session.record.db_path);
        let composition = composition(&root);
        let expected_error = if corrupt_blob { "invalid production AgentStore checkpoint" } else { "checkpoint root blob is missing" };
        let error = composition
            .confirmed_user_turn_watermark(&workers, &session.record.id)
            .err()
            .expect("unknown history is not a no-user watermark");
        assert!(error.contains(expected_error), "{error}");
        assert!(composition.prompt_compaction_epoch(&workers, &session.record.id).is_err());
        assert!(composition.compose_production_checkpoint_sink(
            &workers, &root, &session.record.id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).is_err());
        assert!(workers.prepare_existing_agent(&session.record.id).is_err());
        assert_eq!(workers.active_agent_store_owner_count(), 0, "failure cannot install empty state");
        assert_eq!(durable_rows(&session.record.db_path), before);
        assert!(!root.join("transcripts").exists(), "rejected recovery must not construct a journal");
        assert_eq!(store.get_blob_blocking(&root_id).expect("failed root unchanged"), if corrupt_blob { Some(vec![0x80]) } else { None });

        store.set_blob_blocking(&root_id, &turn.state_bytes).expect("repair original root bytes");
        let watermark = composition.confirmed_user_turn_watermark(&workers, &session.record.id).expect("repaired watermark");
        assert!(watermark.has_user_turn);
        assert_eq!(watermark.last_user_message_id.as_deref(), Some("message-original"));
        assert_eq!(composition.prompt_compaction_epoch(&workers, &session.record.id).expect("repaired epoch"), 0);
        let sink = composition.compose_production_checkpoint_sink(
            &workers, &root, &session.record.id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).expect("repaired checkpoint consumer").expect("main sink");
        assert_eq!(sink.base_state_bytes().expect("recovered prior bytes"), turn.state_bytes);
        assert_eq!(durable_rows(&session.record.db_path), before, "reads do not rewrite settlement state");
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn checked_reset_preserves_last_good_snapshot_while_legacy_bool_reset_keeps_its_contract() {
    let root = temp_root("retain-snapshot");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
    let checkpoint = vec![0x9a, 0x06, 0x03, b'o', b'l', b'd'];
    let root_id = session.agent_store.handle_checkpoint_bytes(&checkpoint).expect("valid checkpoint");
    let metadata = session.db.read_kv("metadata").expect("read metadata").expect("metadata");
    rewrite_raw_metadata(&session.record.db_path, None);
    let before = durable_rows(&session.record.db_path);
    assert!(session.agent_store.reset_from_db_checked().is_err());
    assert_eq!(session.agent_store.latest_root_blob_id_checked().expect("prior root"), root_id);
    assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("prior snapshot"), Some(checkpoint.clone()));
    assert!(workers.open_agent_store_owner(&session.record.id).is_err(), "cached snapshot does not authorize corrupt durable re-entry");
    assert_eq!(durable_rows(&session.record.db_path), before);
    assert!(!session.agent_store.try_reset_from_db(), "compatibility API keeps its boolean failure");
    assert_eq!(session.agent_store.latest_checkpoint_bytes(), None, "compatibility API still clears its cache");

    rewrite_raw_metadata(&session.record.db_path, Some(&metadata));
    assert!(session.agent_store.reset_from_db_checked().expect("explicit repair and reload"));
    let recovered = workers.open_agent_store_owner(&session.record.id).expect("re-entry after repair");
    assert!(Arc::ptr_eq(&recovered, &session.agent_store));
    assert_eq!(recovered.latest_checkpoint_bytes_checked().expect("restored checkpoint"), Some(checkpoint));
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn root_refusal_preserves_pending_profile_recovery_until_original_bytes_are_repaired() {
    for corrupt_blob in [false, true] {
        let root = temp_root("pending-profile");
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
        let admission = session.db.read_session_work_admission().expect("admission read").expect("admission");
        let profile_path = session.record.db_path.parent().expect("agent directory").join("profile.json");
        let before_file = fs::read(&profile_path).expect("original profile bytes");
        let mut after_file = before_file.clone();
        after_file.push(b'\n');
        let mut mutation = PendingProfileMutation {
            mutation_id: "profile-recovery-original".into(),
            operation: "profile-update".into(),
            session_key: admission.session_key,
            session_id: admission.session_id,
            lifecycle_revision: admission.lifecycle_revision,
            before_files: vec![ProfileFileSnapshot {
                filename: "profile.json".into(), contents: Some(before_file.clone()),
            }],
            after_files: Vec::new(),
            previous_profile: session.db.get_sand_profile().expect("original profile state"),
            next_profile: None,
        };
        assert!(session.db.prepare_pending_profile_mutation(&mutation).expect("prepare real profile journal"));
        fs::write(&profile_path, &after_file).expect("profile write before interrupted settlement");
        mutation.after_files = vec![ProfileFileSnapshot {
            filename: "profile.json".into(), contents: Some(after_file.clone()),
        }];
        assert!(session.db.seal_pending_profile_mutation(&mutation).expect("seal real profile journal"));
        let root_id = vec![0xde, 0xad];
        let store = workers.create_agent_blob_store(&session.record.id).expect("blob owner");
        if corrupt_blob {
            store.set_blob_blocking(&root_id, &[0x80]).expect("damaged root fixture");
        }
        assert!(session.db.compare_and_set_latest_root_blob_id(&[], &root_id).expect("root fixture"));
        workers.retire_agent_owners(&session.record.id, false).expect("restart owners");
        let before = durable_rows(&session.record.db_path);

        assert!(workers.open_agent_store_owner(&session.record.id).is_err());
        assert!(workers.open_agent_db_owner(&session.record.id).is_err());
        assert!(workers.prepare_existing_agent(&session.record.id).is_err());
        assert_eq!(durable_rows(&session.record.db_path), before, "journal must remain unsettled");
        assert_eq!(fs::read(&profile_path).expect("unrecovered file"), after_file);
        assert_eq!(workers.active_agent_db_owner_count(), 0);
        assert_eq!(workers.active_agent_store_owner_count(), 0);

        store.set_blob_blocking(&root_id, &[]).expect("repair checkpoint bytes");
        let recovered = workers.open_agent_store_owner(&session.record.id).expect("valid-root journal recovery");
        let db = workers.open_agent_db_owner(&session.record.id).expect("recovered DB owner");
        assert_eq!(fs::read(&profile_path).expect("restored profile file"), before_file);
        assert_eq!(db.read_pending_profile_mutation().expect("settled journal"), None);
        assert_eq!(recovered.latest_checkpoint_bytes_checked().expect("restored checkpoint"), Some(Vec::new()));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn production_checkpoint_writes_refuse_durable_root_corruption_without_writing_or_confirming() {
    for bad_root in [Value::Null, json!(42), json!([])] {
        let root = temp_root("write-refusal");
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
        session.db.append_transcript_entry(&json!({
            "id": "pending-user", "kind": "message", "role": "user",
            "content": "must remain pending", "confirmed": false,
        })).expect("pending user message");
        let valid_raw = session.db.read_kv("metadata").expect("metadata read").expect("metadata");
        let mut metadata = session.db.read_metadata().expect("valid metadata");
        metadata["latestRootBlobId"] = bad_root;
        rewrite_raw_metadata(&session.record.db_path, Some(&encoded_metadata(metadata)));
        let before = durable_rows(&session.record.db_path);
        let checkpoint = vec![0x9a, 0x06, 0x03, b'n', b'e', b'w'];
        let next_root = Sha256::digest(&checkpoint).to_vec();

        assert!(session.agent_store.handle_checkpoint_bytes(&checkpoint).is_err());
        assert!(session.agent_store.handle_checkpoint_bytes_and_confirm_user_message_blocking(
            &checkpoint, "pending-user",
        ).is_err());
        assert_eq!(durable_rows(&session.record.db_path), before);
        assert_eq!(session.agent_store.get_blob(&next_root).expect("no unadmitted blob write"), None);
        assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("unchanged cache"), None);

        rewrite_raw_metadata(&session.record.db_path, Some(&valid_raw));
        assert_eq!(session.agent_store.handle_checkpoint_bytes_and_confirm_user_message_blocking(
            &checkpoint, "pending-user",
        ).expect("explicit empty root permits repaired publication"), next_root);
        assert_eq!(session.db.get_transcript_entry_by_id("pending-user").expect("confirmed row")
            .expect("user row").get("confirmed"), Some(&Value::Bool(true)));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn storage_open_failure_does_not_become_an_empty_cached_checkpoint() {
    let root = temp_root("storage-error");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
    let root_id = session.agent_store.handle_checkpoint_bytes(&[]).expect("valid checkpoint");
    let blob_path = session.agent_store.blob_db_path.clone();
    workers.retire_agent_owners(&session.record.id, false).expect("close blob handles");
    let saved_blob_path = blob_path.with_extension("saved");
    fs::rename(&blob_path, &saved_blob_path).expect("retain original blob database");
    fs::create_dir(&blob_path).expect("unopenable database path fixture");
    let before = durable_rows(&session.record.db_path);

    let error = workers.open_agent_store_owner(&session.record.id).err().expect("I/O must refuse recovery");
    assert!(error.contains("could not read production AgentStore checkpoint"), "{error}");
    assert_eq!(workers.active_agent_store_owner_count(), 0);
    assert_eq!(durable_rows(&session.record.db_path), before);
    assert!(blob_path.is_dir(), "failure must not replace the blocked database path");
    fs::remove_dir(&blob_path).expect("remove injected path failure");
    fs::rename(&saved_blob_path, &blob_path).expect("restore original blob database");
    let restored = workers.open_agent_store_owner(&session.record.id).expect("repair is retryable");
    assert_eq!(restored.latest_root_blob_id_checked().expect("restored root"), root_id);
    assert_eq!(restored.latest_checkpoint_bytes_checked().expect("restored bytes"), Some(Vec::new()));
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cached_empty_or_stale_state_refuses_changed_durable_root_until_checked_reload() {
    let root = temp_root("cached-root");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
    let store = workers.create_agent_blob_store(&session.record.id).expect("blob owner");
    // Retained AgentStore2 supports fixedRootBlobId: a readable checkpoint is
    // valid even when its root ID is not a SHA-256 digest of the payload.
    let fixed_root = vec![0xaa, 0xbb, 0xcc];
    let checkpoint = vec![0x9a, 0x06, 0x03, b'o', b'l', b'd'];
    store.set_blob_blocking(&fixed_root, &checkpoint).expect("fixed root blob");
    assert!(session.db.compare_and_set_latest_root_blob_id(&[], &fixed_root).expect("legacy root commit"));
    let before = durable_rows(&session.record.db_path);
    let error = workers.open_agent_store_owner(&session.record.id).err().expect("cached empty must refuse changed root");
    assert!(error.contains("checked reload is required"), "{error}");
    assert!(composition(&root).prompt_compaction_epoch(&workers, &session.record.id).is_err());
    assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("unchanged cached state"), None);
    assert_eq!(durable_rows(&session.record.db_path), before);

    assert!(session.agent_store.reset_from_db_checked().expect("explicit legacy reload"));
    let cached = workers.open_agent_store_owner(&session.record.id).expect("matching cached owner");
    assert!(Arc::ptr_eq(&cached, &session.agent_store));
    assert_eq!(cached.latest_root_blob_id_checked().expect("fixed root"), fixed_root);
    store.set_blob_blocking(&fixed_root, &[0x80]).expect("later corruption");
    assert!(workers.open_agent_store_owner(&session.record.id).is_err());
    assert_eq!(cached.latest_checkpoint_bytes_checked().expect("last good snapshot"), Some(checkpoint.clone()));
    store.set_blob_blocking(&fixed_root, &checkpoint).expect("explicit repair");
    assert!(workers.open_agent_store_owner(&session.record.id).is_ok());
    workers.retire_agent_owners(&session.record.id, false).expect("close original owners");
    let fresh_workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let fresh = fresh_workers.open_agent_store_owner(&session.record.id).expect("fresh fixed-root reader");
    assert_eq!(fresh.latest_checkpoint_bytes_checked().expect("legacy checkpoint"), Some(checkpoint));
    fresh_workers.shutdown();
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn legacy_empty_root_maintenance_refreshes_the_existing_cached_agent_store() {
    let root = temp_root("legacy-salvage");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
    let store = workers.create_agent_blob_store(&session.record.id).expect("blob owner");
    let turn = build_text_turn_checkpoint(None, "legacy question", "message-legacy", Some("request-legacy"), "legacy answer");
    write_turn_blobs(&store, &turn);
    let root_id = Sha256::digest(&turn.state_bytes).to_vec();
    store.set_blob_blocking(&root_id, &turn.state_bytes).expect("unclaimed legacy root");
    let composition = composition(&root);
    assert_eq!(composition.prompt_compaction_epoch(&workers, &session.record.id).expect("preparation cached empty root"), 0);
    assert_eq!(session.agent_store.latest_checkpoint_bytes_checked().expect("initial empty cache"), None);

    let prepared = workers.prepare_existing_agent(&session.record.id).expect("legacy recovery").expect("existing session");
    assert_eq!(prepared.persisted_root_blob_id, root_id);
    let recovered = workers.open_agent_store_owner(&session.record.id).expect("reuse refreshed owner");
    assert!(Arc::ptr_eq(&recovered, &session.agent_store));
    assert_eq!(recovered.latest_checkpoint_bytes_checked().expect("recovered cache"), Some(turn.state_bytes));
    let watermark = composition.confirmed_user_turn_watermark(&workers, &session.record.id).expect("legacy recovery consumer");
    assert!(watermark.has_user_turn);
    assert_eq!(watermark.last_user_message_id.as_deref(), Some("message-legacy"));
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn normalized_empty_root_remains_readable_without_widening_raw_string_cas() {
    let root = temp_root("normalized-empty");
    let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let record = workers.materialize_new_session_with_active(None, "user", None, None).expect("session");
    let db = workers.open_agent_db_owner(&record.id).expect("DB owner");
    let mut metadata = db.read_metadata().expect("metadata");
    metadata["latestRootBlobId"] = json!(" ");
    rewrite_raw_metadata(&record.db_path, Some(&encoded_metadata(metadata)));
    let before = durable_rows(&record.db_path);
    let owner = workers.open_agent_store_owner(&record.id).expect("legacy normalized empty read");
    assert_eq!(owner.checkpoint_snapshot_checked().expect("checked normalized root"), None);
    assert!(!db.compare_and_set_latest_root_blob_id(&[], &[0x55; 32]).expect("known raw-string CAS refusal"));
    assert_eq!(durable_rows(&record.db_path), before, "read normalization must not rewrite or widen the CAS");
    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cached_same_id_drift_refuses_before_pending_recovery_and_materialization_repairs() {
    for case in ["pending-profile", "missing-files"] {
        let root = temp_root(&format!("cached-drift-before-recovery-{case}"));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let session = workers.materialize_session_with_active(None, "user", None, None).expect("session");
        let original = vec![0x9a, 0x06, 0x03, b'o', b'l', b'd'];
        let root_id = session.agent_store.handle_checkpoint_bytes(&original).expect("cached original root");
        let agent_dir = session.record.db_path.parent().expect("agent directory");
        let profile_path = agent_dir.join("profile.json");
        let settings_path = agent_dir.join("settings.json");
        let before_file = fs::read(&profile_path).expect("profile bytes");
        let mut after_file = before_file.clone();
        after_file.push(b'\n');
        if case == "pending-profile" {
            let admission = session.db.read_session_work_admission().expect("admission").expect("initialized admission");
            let mut mutation = PendingProfileMutation {
                mutation_id: "cached-drift-profile".into(), operation: "profile-update".into(),
                session_key: admission.session_key, session_id: admission.session_id,
                lifecycle_revision: admission.lifecycle_revision,
                before_files: vec![ProfileFileSnapshot { filename: "profile.json".into(), contents: Some(before_file.clone()) }],
                after_files: Vec::new(), previous_profile: session.db.get_sand_profile().expect("previous profile"),
                next_profile: None,
            };
            assert!(session.db.prepare_pending_profile_mutation(&mutation).expect("prepare actual journal"));
            fs::write(&profile_path, &after_file).expect("interrupted profile write");
            mutation.after_files = vec![ProfileFileSnapshot { filename: "profile.json".into(), contents: Some(after_file.clone()) }];
            assert!(session.db.seal_pending_profile_mutation(&mutation).expect("seal actual journal"));
        } else {
            fs::remove_file(&profile_path).expect("remove repairable profile");
            fs::remove_file(&settings_path).expect("remove repairable settings");
        }
        let blobs = workers.create_agent_blob_store(&session.record.id).expect("actual blob owner");
        blobs.set_blob_blocking(&root_id, &[0x9a, 0x06, 0x03, b'n', b'e', b'w'])
            .expect("same-ID replacement remains valid protobuf");
        let before = durable_rows(&session.record.db_path);
        assert!(workers.prepare_existing_agent(&session.record.id).is_err());
        assert!(workers.open_agent_db_owner(&session.record.id).is_err());
        assert!(workers.open_agent_store_owner(&session.record.id).is_err());
        assert_eq!(durable_rows(&session.record.db_path), before, "{case}: refusal must not settle any journal");
        assert_eq!(session.agent_store.latest_checkpoint_bytes(), Some(original.clone()));
        if case == "pending-profile" {
            assert_eq!(fs::read(&profile_path).expect("unrecovered file"), after_file);
            assert!(session.db.read_pending_profile_mutation().expect("pending journal").is_some());
        } else {
            assert!(!profile_path.exists(), "profile repair must wait for a valid cached binding");
            assert!(!settings_path.exists(), "settings repair must wait for a valid cached binding");
        }
        blobs.set_blob_blocking(&root_id, &original).expect("explicit original-byte repair");
        assert!(workers.prepare_existing_agent(&session.record.id).expect("retry after durable repair").is_some());
        assert!(profile_path.is_file());
        assert!(settings_path.is_file());
        if case == "pending-profile" {
            assert_eq!(fs::read(&profile_path).expect("recovered file"), before_file);
            assert_eq!(session.db.read_pending_profile_mutation().expect("settled journal"), None);
        }
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}
