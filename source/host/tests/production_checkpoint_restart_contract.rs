//! SWA-ROOT-RECOVERY-v1: checkpoint readiness across an actual OS-process
//! boundary. The parent materializes and commits through shipping owners, then
//! closes them before the child opens the same files. These are storage and
//! Host-composition prerequisites, not provider replay or full resume acceptance.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::session_paths::{
    CONVERSATION_BLOBS_FILENAME, STORE_FILENAME,
};
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::TurnRunOptions;
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CHILD_TEST: &str = "production_checkpoint_restart_child";
const CHILD_REQUEST_ENV: &str = "FABUSHI_CHECKPOINT_RESTART_CONTRACT_REQUEST";
const USER_ID: &str = "checkpoint-restart-user";
const USER_TEXT: &str = "remember this durable user turn";
const ASSISTANT_TEXT: &str = "this reply survives the owner process";
const LEGACY_FIXED_ROOT_ID: &[u8] = b"old";
// self_summary_count (field 17) = 3, followed by unknown field 99 = "keep".
const PRIOR_STATE: &[u8] = &[0x88, 0x01, 0x03, 0x9a, 0x06, 0x04, b'k', b'e', b'e', b'p'];

struct OpenOwners {
    sessions: Arc<ProductionSessionWorkers>,
    closed: bool,
}

impl OpenOwners {
    fn new(root: &Path) -> Self {
        Self {
            sessions: Arc::new(ProductionSessionWorkers::with_agents_root(
                root.join("agents"),
                500,
            )),
            closed: false,
        }
    }

    fn close(mut self) {
        self.sessions.shutdown_with_checkpoint();
        self.closed = true;
        assert_eq!(self.sessions.active_worker_count(), 0, "all owning workers closed");
    }
}

impl Drop for OpenOwners {
    fn drop(&mut self) {
        if !self.closed {
            self.sessions.shutdown();
        }
    }
}

fn composition(root: &Path, sessions: &Arc<ProductionSessionWorkers>) -> HostRunnerComposition {
    HostRunnerComposition::production(
        Arc::new(SandLocalToolPermissionController::production(Arc::new(
            SettingsService::new(root.join("settings.json")),
        ))),
        Arc::clone(sessions),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExpectedCheckpoint {
    root_blob_id: Vec<u8>,
    state_bytes: Option<Vec<u8>>,
    last_user_message_id: Option<String>,
    compaction_epoch: u64,
    critical_blobs: Vec<(Vec<u8>, Vec<u8>)>,
}

enum InitialState {
    NoCheckpoint,
    EmptyCheckpoint,
    UserCheckpoint,
}

struct Fixture {
    root: PathBuf,
    agent_id: String,
    expected: ExpectedCheckpoint,
}

impl Fixture {
    fn new(label: &str, initial: InitialState) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-checkpoint-restart-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).expect("create uniquely owned fixture directory");
        let root = root.canonicalize().expect("absolute fixture root");
        let owners = OpenOwners::new(&root);
        let session = owners.sessions
            .materialize_session_with_active(None, "user", None, None)
            .expect("materialize a real production session");
        let agent_id = session.record.id.clone();
        let mut expected = ExpectedCheckpoint {
            root_blob_id: Vec::new(),
            state_bytes: None,
            last_user_message_id: None,
            compaction_epoch: 0,
            critical_blobs: Vec::new(),
        };
        match initial {
            InitialState::NoCheckpoint => {}
            InitialState::EmptyCheckpoint => {
                expected.root_blob_id = session.agent_store
                    .handle_checkpoint_bytes_blocking(&[])
                    .expect("commit a present zero-length protobuf checkpoint");
                assert_eq!(expected.root_blob_id, Sha256::digest([]).to_vec());
                expected.state_bytes = Some(Vec::new());
                expected.critical_blobs.push((expected.root_blob_id.clone(), Vec::new()));
            }
            InitialState::UserCheckpoint => {
                session.agent_store.handle_checkpoint_bytes_blocking(PRIOR_STATE)
                    .expect("commit prior epoch and unknown wire field");
                session.db.append_transcript_entry(&json!({
                    "id": USER_ID, "kind": "message", "role": "user",
                    "content": USER_TEXT, "confirmed": false,
                })).expect("persist actual unconfirmed user echo");
                let host = composition(&root, &owners.sessions);
                let sink = host.compose_production_checkpoint_sink(
                    &owners.sessions, &root, &agent_id, false,
                    Arc::new(|_| {}), Arc::new(|| Ok(true)),
                ).expect("compose shipping checkpoint sink").expect("main-session sink");
                let options = TurnRunOptions {
                    message_id: Some(USER_ID.into()),
                    inference_request_id: Some("checkpoint-restart-request".into()),
                    recent_message_text: Some(USER_TEXT.into()),
                    ..TurnRunOptions::default()
                };
                let checkpoint = sink.stage_text_turn(
                    &[ProviderMessage { role: "user".into(), content: USER_TEXT.into() }],
                    &options,
                    ASSISTANT_TEXT,
                ).expect("stage through the actual Runner checkpoint helper");
                sink.persist_staged_text_turn(&options, &checkpoint)
                    .expect("commit canonical root, confirmation and transcript mirror");
                assert!(checkpoint.state_bytes.starts_with(PRIOR_STATE));
                expected.root_blob_id = Sha256::digest(&checkpoint.state_bytes).to_vec();
                expected.state_bytes = Some(checkpoint.state_bytes.clone());
                expected.last_user_message_id = Some(USER_ID.into());
                expected.compaction_epoch = 3;
                expected.critical_blobs = vec![
                    (expected.root_blob_id.clone(), checkpoint.state_bytes),
                    (checkpoint.user_message_id, checkpoint.user_message_bytes),
                    (checkpoint.turn_id, checkpoint.turn_bytes),
                ];
                expected.critical_blobs.extend(
                    checkpoint.step_ids.into_iter().zip(checkpoint.step_bytes),
                );
                // Complete existing outline maintenance before taking the durable
                // baseline, so a later readiness read need not backfill entries.
                owners.sessions.prepare_existing_agent(&agent_id)
                    .expect("prepare the committed session").expect("existing session");
                drop((sink, host));
            }
        }
        assert_eq!(session.agent_store.latest_root_blob_id(), expected.root_blob_id);
        assert_eq!(session.agent_store.latest_checkpoint_bytes(), expected.state_bytes);
        drop(session);
        owners.close();
        Self { root, agent_id, expected }
    }

    fn rebind_to_legacy_fixed_root(&mut self) {
        let owners = OpenOwners::new(&self.root);
        let store = owners.sessions.open_agent_store_owner(&self.agent_id)
            .expect("open the committed checkpoint before explicit legacy binding");
        let db = owners.sessions.open_agent_db_owner(&self.agent_id)
            .expect("existing metadata owner");
        let blobs = owners.sessions.create_agent_blob_store(&self.agent_id)
            .expect("existing blob owner");
        let bytes = self.expected.state_bytes.clone().expect("committed user checkpoint");
        // Retained fixed IDs are valid references, even when shorter than a
        // SHA-256 digest. Store the real generated state before its metadata CAS.
        blobs.set_blob_blocking(LEGACY_FIXED_ROOT_ID, &bytes)
            .expect("store valid protobuf under the legacy fixed ID");
        assert!(db.compare_and_set_latest_root_blob_id(
            &self.expected.root_blob_id, LEGACY_FIXED_ROOT_ID,
        ).expect("bind the legacy root through the existing metadata owner"));
        assert!(store.reset_from_db_checked()
            .expect("explicit checked reload establishes the legacy cached root"));
        assert_eq!(store.latest_root_blob_id_checked().expect("checked cached root").as_slice(),
            LEGACY_FIXED_ROOT_ID);
        assert_eq!(store.latest_checkpoint_bytes_checked().expect("checked cached bytes").as_deref(),
            Some(bytes.as_slice()));
        let previous_root = std::mem::replace(
            &mut self.expected.root_blob_id, LEGACY_FIXED_ROOT_ID.to_vec(),
        );
        self.expected.critical_blobs.retain(|(id, _)| id != &previous_root);
        self.expected.critical_blobs.push((LEGACY_FIXED_ROOT_ID.to_vec(), bytes));
        drop((store, db, blobs));
        owners.close();
    }

    fn db_path(&self) -> PathBuf {
        self.root.join("agents").join(&self.agent_id).join(STORE_FILENAME)
    }

    fn blob_path(&self) -> PathBuf {
        self.root.join("agents").join(&self.agent_id).join(CONVERSATION_BLOBS_FILENAME)
    }

    fn reopen(&self, label: &str, expectation: ReopenExpectation) -> ChildRun {
        let token = uuid::Uuid::new_v4().to_string();
        let run_dir = self.root.join(format!("process-{label}-{token}"));
        fs::create_dir(&run_dir).expect("create private child run directory");
        let request_path = run_dir.join("request.json");
        let request = ChildRequest {
            token: token.clone(),
            parent_pid: std::process::id(),
            fixture_root: self.root.clone(),
            agent_id: self.agent_id.clone(),
            expectation,
            expected: self.expected.clone(),
        };
        fs::write(&request_path, serde_json::to_vec(&request).expect("serialize child request"))
            .expect("write private child request");
        let stdout_path = run_dir.join("stdout.log");
        let stderr_path = run_dir.join("stderr.log");
        let mut child = OwnedChild {
            child: Command::new(std::env::current_exe().expect("current contract executable"))
                .args(["--exact", CHILD_TEST, "--ignored", "--nocapture", "--test-threads=1"])
                .env(CHILD_REQUEST_ENV, &request_path)
                .env("SAND_CONVERSATION_GC", "0")
                .env("SAND_STALE_ROOT_GC", "0")
                .env("SAND_RETIRE_LEGACY_STORE_BLOBS", "0")
                .current_dir(&run_dir)
                .stdin(Stdio::null())
                // Files avoid an unread pipe blocking a failing child. The
                // complete output is replayed into the Actions log below.
                .stdout(Stdio::from(File::create(&stdout_path).expect("child stdout")))
                .stderr(Stdio::from(File::create(&stderr_path).expect("child stderr")))
                .spawn().expect("launch exact isolated child test"),
            reaped: false,
        };
        let child_pid = child.child.id();
        let result = child.wait_bounded();
        let logs = format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&fs::read(&stdout_path).expect("read child stdout")),
            String::from_utf8_lossy(&fs::read(&stderr_path).expect("read child stderr")),
        );
        eprintln!("checkpoint reopen {label} pid={child_pid} result={result:?}\n{logs}");
        assert!(child.reaped, "owned child could not be reaped; preserve {}", run_dir.display());
        ChildRun { result, child_pid, token, expectation, run_dir, logs }
    }

    fn inject_fault(&self, fault: Fault) -> Repair {
        // Raw SQL is restricted to fault injection while all production owners
        // are closed. Legitimate initial checkpoints use the real sink above.
        if matches!(fault, Fault::MissingRootBlob | Fault::MalformedRootBlob) {
            let db = open_existing(&self.blob_path(), false);
            let id = hex(&self.expected.root_blob_id);
            let changed = match fault {
                Fault::MissingRootBlob => db.execute("DELETE FROM blobs WHERE id=?1", params![id]),
                Fault::MalformedRootBlob => db.execute(
                    "UPDATE blobs SET data=?2 WHERE id=?1", params![id, vec![0_u8]],
                ),
                _ => unreachable!(),
            }.expect("inject persisted root fault");
            assert_eq!(changed, 1, "fault must affect the actual committed root");
            return Repair::RootBlob;
        }
        let db = open_existing(&self.db_path(), false);
        let original: String = db.query_row(
            "SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0),
        ).expect("original production metadata");
        let changed = match fault {
            Fault::MissingMetadata => db.execute("DELETE FROM kv WHERE key='metadata'", []),
            Fault::MalformedMetadata => db.execute(
                "UPDATE kv SET value=?1 WHERE key='metadata'", params![hex(b"{")],
            ),
            Fault::MissingRootField | Fault::NonStringRootField => {
                let bytes = original.as_bytes().chunks_exact(2).map(|pair| {
                    u8::from_str_radix(std::str::from_utf8(pair).expect("seeded hex"), 16)
                        .expect("seeded metadata byte")
                }).collect::<Vec<_>>();
                let mut metadata: Value = serde_json::from_slice(&bytes).expect("seeded metadata JSON");
                if matches!(fault, Fault::MissingRootField) {
                    assert!(metadata.as_object_mut().expect("metadata object")
                        .remove("latestRootBlobId").is_some());
                } else {
                    metadata["latestRootBlobId"] = json!(42);
                }
                db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![
                    hex(&serde_json::to_vec(&metadata).expect("faulted metadata JSON")),
                ])
            }
            _ => unreachable!(),
        }.expect("inject persisted metadata fault");
        assert_eq!(changed, 1);
        Repair::Metadata(original)
    }

    fn repair(&self, repair: Repair) {
        match repair {
            Repair::RootBlob => {
                let owners = OpenOwners::new(&self.root);
                let blobs = owners.sessions.create_agent_blob_store(&self.agent_id)
                    .expect("explicit repair uses the existing blob worker");
                blobs.set_blob_blocking(
                    &self.expected.root_blob_id,
                    self.expected.state_bytes.as_deref().expect("committed fixture bytes"),
                ).expect("explicitly restore exact original root bytes");
                drop(blobs);
                owners.close();
            }
            Repair::Metadata(original) => {
                // Restore the captured row explicitly, without treating a failed
                // production reopen as permission to seed replacement metadata.
                open_existing(&self.db_path(), false).execute(
                    "INSERT INTO kv (key,value) VALUES ('metadata',?1) \
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    params![original],
                ).expect("explicitly restore the original metadata bytes");
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if thread::panicking() {
            eprintln!("preserved failing checkpoint process fixture: {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove the exact closed fixture directory");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ReopenExpectation {
    Ready,
    Refuse,
}

#[derive(Serialize, Deserialize)]
struct ChildRequest {
    token: String,
    parent_pid: u32,
    fixture_root: PathBuf,
    agent_id: String,
    expectation: ReopenExpectation,
    expected: ExpectedCheckpoint,
}

#[derive(Serialize, Deserialize)]
struct ChildReceipt {
    token: String,
    pid: u32,
    parent_pid: u32,
    expectation: ReopenExpectation,
}

struct OwnedChild {
    child: Child,
    reaped: bool,
}

impl OwnedChild {
    fn poll_until(&mut self, budget: Duration) -> Result<Option<ExitStatus>, String> {
        let deadline = Instant::now() + budget;
        loop {
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                // try_wait reaps an exited child; no blocking pipe read or
                // unbounded wait is needed, including after forced termination.
                self.reaped = true;
                return Ok(Some(status));
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn stop_and_reap(&mut self) -> Result<ExitStatus, String> {
        let wait_error = match self.poll_until(Duration::ZERO) {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => None,
            Err(error) => Some(error),
        };
        let kill_error = self.child.kill().err();
        match self.poll_until(Duration::from_secs(5)) {
            Ok(Some(status)) => Ok(status),
            outcome => Err(format!(
                "owned child {} was not reaped after kill: {outcome:?}; \
                 initial wait error: {wait_error:?}; kill error: {kill_error:?}",
                self.child.id(),
            )),
        }
    }

    fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
        let reason = match self.poll_until(Duration::from_secs(45)) {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => "checkpoint child exceeded its 45-second deadline".to_string(),
            Err(error) => format!("could not observe checkpoint child: {error}"),
        };
        let cleanup = self.stop_and_reap();
        Err(format!("{reason}; exact-child cleanup: {cleanup:?}"))
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.reaped {
            if let Err(error) = self.stop_and_reap() {
                eprintln!("owned checkpoint child cleanup failed: {error}");
            }
        }
    }
}

struct ChildRun {
    result: Result<ExitStatus, String>,
    child_pid: u32,
    token: String,
    expectation: ReopenExpectation,
    run_dir: PathBuf,
    logs: String,
}

impl ChildRun {
    fn assert_completed(self) {
        assert!(self.result.as_ref().is_ok_and(|status| status.success()),
            "checkpoint child failed: {:?}\n{}", self.result, self.logs);
        let receipt: ChildReceipt = serde_json::from_slice(
            &fs::read(self.run_dir.join("completed.json"))
                .expect("successful exit must include a fresh post-assertion child receipt"),
        ).expect("child completion receipt");
        assert_eq!(receipt.token, self.token, "receipt belongs to this exact launch");
        assert_eq!(receipt.pid, self.child_pid, "receipt belongs to the owned child");
        assert_eq!(receipt.parent_pid, std::process::id());
        assert_ne!(receipt.pid, receipt.parent_pid, "this must be a separate OS process");
        assert_eq!(receipt.expectation, self.expectation);
    }
}

fn open_existing(path: &Path, read_only: bool) -> Connection {
    let flags = if read_only { OpenFlags::SQLITE_OPEN_READ_ONLY } else { OpenFlags::SQLITE_OPEN_READ_WRITE };
    let db = Connection::open_with_flags(path, flags).expect("open existing fixture database");
    db.busy_timeout(Duration::from_millis(500)).expect("bounded fixture SQLite wait");
    db
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn mirror_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(base: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !directory.exists() {
            return;
        }
        for entry in fs::read_dir(directory).expect("read transcript mirror directory") {
            let path = entry.expect("mirror entry").path();
            if path.is_dir() {
                collect(base, &path, files);
            } else {
                files.insert(path.strip_prefix(base).expect("fixture mirror path").to_path_buf(),
                    fs::read(&path).expect("exact transcript mirror bytes"));
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

#[derive(Debug, PartialEq, Eq)]
struct DurableState {
    kv: Vec<(String, Vec<u8>)>,
    transcript: Vec<(i64, String, Vec<u8>)>,
    blobs: Vec<(String, String, Vec<u8>)>,
    mirror: BTreeMap<PathBuf, Vec<u8>>,
}

fn durable_state(fixture: &Fixture) -> DurableState {
    // Compare stored row/payload bytes, not SQLite page/WAL layout. Read-only
    // observations cannot repair, normalize or create the subject databases.
    let db = open_existing(&fixture.db_path(), true);
    let kv = db.prepare("SELECT key,CAST(value AS BLOB) FROM kv ORDER BY key")
        .expect("kv query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?))).expect("kv rows")
        .collect::<Result<Vec<_>, _>>().expect("kv bytes");
    let transcript = db.prepare("SELECT seq,id,CAST(entry AS BLOB) FROM transcript_entries ORDER BY seq")
        .expect("transcript query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).expect("transcript rows")
        .collect::<Result<Vec<_>, _>>().expect("transcript bytes");
    let blob_db = open_existing(&fixture.blob_path(), true);
    let blobs = blob_db.prepare("SELECT id,typeof(data),CAST(data AS BLOB) FROM blobs ORDER BY id")
        .expect("blob query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).expect("blob rows")
        .collect::<Result<Vec<_>, _>>().expect("all critical and remaining blob bytes");
    DurableState { kv, transcript, blobs, mirror: mirror_files(&fixture.root.join("transcripts")) }
}

#[derive(Clone, Copy, Debug)]
enum Fault {
    MissingRootBlob,
    MalformedRootBlob,
    MissingMetadata,
    MalformedMetadata,
    MissingRootField,
    NonStringRootField,
}

enum Repair {
    RootBlob,
    Metadata(String),
}

fn assert_ready(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) {
    let prepared = sessions.prepare_existing_agent(&request.agent_id)
        .expect("fresh production preparation must accept intact durable state")
        .expect("already materialized session remains present");
    assert_eq!(prepared.persisted_root_blob_id, request.expected.root_blob_id);
    let store = sessions.open_agent_store_owner(&request.agent_id).expect("fresh AgentStore owner");
    // Legacy getters are byte evidence only. Readiness is decided by the real
    // preparation and Host consumers, without depending on a new snapshot type.
    assert_eq!(store.latest_root_blob_id(), request.expected.root_blob_id);
    assert_eq!(store.latest_checkpoint_bytes(), request.expected.state_bytes);
    let host = composition(&request.fixture_root, sessions);
    let watermark = host.confirmed_user_turn_watermark(sessions, &request.agent_id)
        .expect("fresh canonical watermark");
    assert_eq!(watermark.has_user_turn, request.expected.last_user_message_id.is_some());
    assert_eq!(watermark.last_user_message_id, request.expected.last_user_message_id);
    assert_eq!(host.prompt_compaction_epoch(sessions, &request.agent_id)
        .expect("fresh canonical prompt epoch"), request.expected.compaction_epoch);
    let sink = host.compose_production_checkpoint_sink(
        sessions, &request.fixture_root, &request.agent_id, false,
        Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("fresh shipping checkpoint composition").expect("main-session checkpoint sink");
    assert_eq!(sink.base_state_bytes().expect("sink consumes exact persisted checkpoint"),
        request.expected.state_bytes.clone().unwrap_or_default());
    for (id, bytes) in &request.expected.critical_blobs {
        assert_eq!(sink.read_state_blob(id).expect("fresh worker reads critical blob").as_deref(),
            Some(bytes.as_slice()), "critical blob {}", hex(id));
    }
    if let Some(message_id) = &request.expected.last_user_message_id {
        let db = sessions.open_agent_db_owner(&request.agent_id).expect("fresh transcript owner");
        let entry = db.get_transcript_entry_by_id(message_id)
            .expect("read persisted user confirmation").expect("confirmed user remains present");
        assert_eq!(entry.get("confirmed").and_then(Value::as_bool), Some(true));
        assert_eq!(entry.get("content").and_then(Value::as_str), Some(USER_TEXT));
    }
}

fn assert_refused(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) {
    fn observe<T>(label: &str, result: Result<T, String>, admitted: &mut Vec<String>) {
        match result {
            Ok(_) => admitted.push(format!("{label} admitted damaged persisted authority")),
            Err(error) if error.trim().is_empty() => admitted.push(format!("{label} lost its diagnostic")),
            Err(error) => eprintln!("expected checkpoint refusal at {label}: {error}"),
        }
    }
    let host = composition(&request.fixture_root, sessions);
    let mut admitted = Vec::new();
    observe("checkpoint sink", host.compose_production_checkpoint_sink(
        sessions, &request.fixture_root, &request.agent_id, false,
        Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ), &mut admitted);
    observe("user watermark", host.confirmed_user_turn_watermark(sessions, &request.agent_id), &mut admitted);
    observe("prompt epoch", host.prompt_compaction_epoch(sessions, &request.agent_id), &mut admitted);
    observe("production preparation", sessions.prepare_existing_agent(&request.agent_id), &mut admitted);
    observe("repeat AgentStore open", sessions.open_agent_store_owner(&request.agent_id), &mut admitted);
    assert!(admitted.is_empty(), "{}", admitted.join("\n"));
}

#[test]
#[ignore = "private child entrypoint; launched with --exact and a per-run request by the parent tests"]
fn production_checkpoint_restart_child() {
    let request_path = PathBuf::from(std::env::var_os(CHILD_REQUEST_ENV)
        .expect("private checkpoint child request environment"));
    let request: ChildRequest = serde_json::from_slice(&fs::read(&request_path).expect("child request"))
        .expect("decode child request");
    assert_ne!(request.parent_pid, std::process::id(), "fresh OS process required");
    assert!(request.fixture_root.is_absolute());
    let owners = OpenOwners::new(&request.fixture_root);
    match request.expectation {
        ReopenExpectation::Ready => assert_ready(&request, &owners.sessions),
        ReopenExpectation::Refuse => assert_refused(&request, &owners.sessions),
    }
    owners.close();
    let receipt = ChildReceipt {
        token: request.token,
        pid: std::process::id(),
        parent_pid: request.parent_pid,
        expectation: request.expectation,
    };
    fs::write(request_path.parent().expect("private run directory").join("completed.json"),
        serde_json::to_vec(&receipt).expect("serialize completion receipt"))
        .expect("publish completion only after assertions and worker shutdown");
}

#[test]
fn fresh_process_distinguishes_initialized_session_from_present_empty_checkpoint() {
    for (label, initial) in [
        ("initialized", InitialState::NoCheckpoint),
        ("present-empty", InitialState::EmptyCheckpoint),
    ] {
        let fixture = Fixture::new(label, initial);
        fixture.reopen(label, ReopenExpectation::Ready).assert_completed();
    }
}

#[test]
fn fresh_process_restores_durable_user_checkpoint_watermark_and_epoch() {
    let fixture = Fixture::new("user-turn", InitialState::UserCheckpoint);
    let before = durable_state(&fixture);
    assert!(!before.mirror.is_empty(), "the real sink must have committed a transcript mirror");
    fixture.reopen("user-turn", ReopenExpectation::Ready).assert_completed();
    assert_eq!(durable_state(&fixture), before,
        "readiness must preserve exact durable bytes without replaying a user turn");
}

#[test]
fn fresh_process_preserves_legacy_fixed_root_checkpoint_and_user_identity() {
    let mut fixture = Fixture::new("legacy-fixed-root", InitialState::UserCheckpoint);
    fixture.rebind_to_legacy_fixed_root();
    assert_eq!(fixture.expected.root_blob_id.as_slice(), LEGACY_FIXED_ROOT_ID);
    let before = durable_state(&fixture);
    assert!(!before.mirror.is_empty(), "the real user checkpoint mirror remains present");
    fixture.reopen("legacy-fixed-root", ReopenExpectation::Ready).assert_completed();
    assert_eq!(durable_state(&fixture), before,
        "fresh consumers must retain the fixed root, confirmed user, epoch and exact stored bytes");
}

#[test]
fn fresh_process_refuses_persisted_damage_and_recovers_after_explicit_repair() {
    for fault in [
        Fault::MissingRootBlob,
        Fault::MalformedRootBlob,
        Fault::MissingMetadata,
        Fault::MalformedMetadata,
        Fault::MissingRootField,
        Fault::NonStringRootField,
    ] {
        let fixture = Fixture::new(&format!("{fault:?}"), InitialState::UserCheckpoint);
        let healthy = durable_state(&fixture);
        let repair = fixture.inject_fault(fault);
        let damaged = durable_state(&fixture);
        assert_ne!(damaged, healthy, "{fault:?}: the fault must be durably observable");
        for attempt in 1..=2 {
            let child = fixture.reopen(&format!("refused-{attempt}"), ReopenExpectation::Refuse);
            assert_eq!(durable_state(&fixture), damaged,
                "{fault:?}: refusal {attempt} must preserve metadata, transcript, blobs and mirror bytes");
            child.assert_completed();
        }
        fixture.repair(repair);
        assert_eq!(durable_state(&fixture), healthy, "explicit repair restores exact captured bytes");
        fixture.reopen("repaired", ReopenExpectation::Ready).assert_completed();
        assert_eq!(durable_state(&fixture), healthy, "repaired process must retain the original history");
    }
}
