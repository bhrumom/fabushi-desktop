//! SWA-ROOT-RECOVERY-v1: checkpoint readiness across an actual OS-process
//! boundary. The parent materializes and commits through shipping owners, then
//! closes them before the child opens the same files. These are storage and
//! Host-composition prerequisites. ORT-v1 cases additionally run the actual
//! generated Native owner against bounded loopback Responses, then reconcile
//! and acknowledge its actual upgrade marker in the shipping App after that
//! process and all of its Session owners have closed.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::agent_db::{
    CompactionCommit, CompactionCommitOutcome, OriginalTurnReceiptCommit, OriginalTurnReference,
};
use mahayana_host_runtime::extensions::session::agent_db_recovery::{
    AUTOMATION_CUSTODY_UNAVAILABLE_KV, automation_custody_recovery_marker_path,
};
use mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context;
use mahayana_host_runtime::extensions::session::session_paths::{
    CONVERSATION_BLOBS_FILENAME, STORE_FILENAME,
};
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime,
};
use mahayana_host_runtime::extensions::transcript::sand_ack_obligation_store::SandAckObligationStore;
use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::{
    SandUpgradeResumeStore, UpgradeResumeMarker,
};
use mahayana_host_runtime::extensions::transcript::turn_runtime::{
    MAX_REPLY_NUDGES, TurnTerminalKind, TurnTerminalProjection, project_turn_terminal,
    shape_closing_send_nudge_turn_input, shape_reply_nudge_turn_input,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    original_turn_result_digest, persist_completed_original_turn,
    prove_completed_original_turn, reconcile_completed_original_turn,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
use mahayana_host_runtime::runner::turn_agent_composition::{SAND_AGENT_TOKEN_LIMIT, TurnAgentComposition};
use mahayana_host_runtime::runner::{
    ToolEffectCustodyObservation, ToolEffectCustodyReport, ToolEffectUnknownResult, TurnRunOptions,
};
use mahayana_host_runtime::runner_production_bridge::create_production_runner;
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CHILD_TEST: &str = "production_checkpoint_restart_child";
const CHILD_REQUEST_ENV: &str = "FABUSHI_CHECKPOINT_RESTART_CONTRACT_REQUEST";
const APP_CHILD_TEST: &str = "tests::original_turn_recovery_tests::original_turn_shipping_ack_process_child";
const APP_CHILD_EXECUTABLE_ENV: &str = "FABUSHI_ORIGINAL_TURN_APP_TEST_EXECUTABLE";
const APP_CHILD_SHA256_ENV: &str = "FABUSHI_ORIGINAL_TURN_APP_TEST_SHA256";
const USER_ID: &str = "checkpoint-restart-user";
const USER_TEXT: &str = "remember this durable user turn";
const ASSISTANT_TEXT: &str = "this reply survives the owner process";
const LEGACY_FIXED_ROOT_ID: &[u8] = b"old";
const ORIGINAL_STREAM: &str = "original-native-stream";
const ORIGINAL_MODEL: &str = "Vendor/Nested/CaseSensitive@OriginalTurn";
const ORIGINAL_TERMINAL_KEY: &str = "original-routed-terminal:current";
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
    original: Option<OriginalTurnReference>,
    original_invocations: usize,
    original_marker: Option<UpgradeResumeMarker>,
    original_compaction: bool,
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
        Self {
            root, agent_id, expected, original: None, original_invocations: 1,
            original_marker: None, original_compaction: false,
        }
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
        self.run_child(label, expectation, None)
    }

    fn run_child(
        &self,
        label: &str,
        expectation: ReopenExpectation,
        server: Option<&OriginalTurnWireServer>,
    ) -> ChildRun {
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
            original: self.original.clone(),
            original_invocations: self.original_invocations,
            original_marker: self.original_marker.clone(),
            original_compaction: self.original_compaction,
        };
        fs::write(&request_path, serde_json::to_vec(&request).expect("serialize child request"))
            .expect("write private child request");
        let stdout_path = run_dir.join("stdout.log");
        let stderr_path = run_dir.join("stderr.log");
        let (executable, child_test) = if matches!(expectation,
            ReopenExpectation::OriginalMarkerAck | ReopenExpectation::OriginalMarkerNoop
                | ReopenExpectation::OriginalMarkerPendingAck | ReopenExpectation::OriginalMarkerCorruptAck
        ) {
            assert!(server.is_none(), "App reconciliation has no provider transport");
            let executable = PathBuf::from(std::env::var_os(APP_CHILD_EXECUTABLE_ENV)
                .expect("Actions must bind the actual shipping App compiler artifact"));
            assert!(executable.is_absolute() && executable.is_file(),
                "bound App test artifact must be an existing absolute file: {}", executable.display());
            let expected_digest = std::env::var(APP_CHILD_SHA256_ENV)
                .expect("Actions must bind the exact App compiler artifact digest");
            let mut file = File::open(&executable).expect("open bound App compiler artifact");
            let mut hasher = Sha256::new();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let length = file.read(&mut buffer).expect("hash bound App compiler artifact");
                if length == 0 { break; }
                hasher.update(&buffer[..length]);
            }
            let actual_digest = format!("{:x}", hasher.finalize());
            assert_eq!(actual_digest, expected_digest, "App child must be the exact bound build artifact");
            eprintln!("checkpoint App child executable={} sha256={actual_digest}", executable.display());
            (executable, APP_CHILD_TEST)
        } else {
            (std::env::current_exe().expect("current contract executable"), CHILD_TEST)
        };
        let mut command = Command::new(executable);
        command.args(["--exact", child_test, "--ignored", "--nocapture", "--test-threads=1"])
            .env(CHILD_REQUEST_ENV, &request_path)
            .env("SAND_CONVERSATION_GC", "0")
            .env("SAND_STALE_ROOT_GC", "0")
            .env("SAND_RETIRE_LEGACY_STORE_BLOBS", "0")
            .env("FABUSHI_RESPONSES_MODEL", "changed-current-model-must-not-replace-original")
            .env_remove("FABUSHI_PRODUCT_MODE").env_remove("SAND_PACKAGED")
            .env_remove("SAND_AGENT_MODEL").env_remove("SAND_AGENT_MOCK_RESPONSE")
            .env("NO_PROXY", "*").env("no_proxy", "*");
        if let Some(server) = server {
            command.env("FABUSHI_RESPONSES_URL", format!("{}/v1/responses", server.origin))
                .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &server.credential);
        } else {
            // Reconciliation has no provider, credential or current-model work.
            command.env("FABUSHI_RESPONSES_URL", "http://127.0.0.1:9/unused")
                .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", run_dir.join("unavailable-credential.json"));
        }
        let mut child = OwnedChild {
            child: command
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
    ProduceOriginal,
    OriginalReady,
    OriginalAbsent,
    OriginalRefuse,
    OriginalMarkerAck,
    OriginalMarkerNoop,
    OriginalMarkerPendingAck,
    OriginalMarkerCorruptAck,
}

#[derive(Serialize, Deserialize)]
struct ChildRequest {
    token: String,
    parent_pid: u32,
    fixture_root: PathBuf,
    agent_id: String,
    expectation: ReopenExpectation,
    expected: ExpectedCheckpoint,
    original: Option<OriginalTurnReference>,
    original_invocations: usize,
    original_marker: Option<UpgradeResumeMarker>,
    original_compaction: bool,
}

#[derive(Serialize, Deserialize)]
struct ProducedOriginal {
    original: OriginalTurnReference,
    checkpoint: ExpectedCheckpoint,
}

#[derive(Serialize, Deserialize)]
struct ChildReceipt {
    token: String,
    pid: u32,
    parent_pid: u32,
    expectation: ReopenExpectation,
    produced: Option<ProducedOriginal>,
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
    fn assert_completed(self) -> ChildReceipt {
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
        receipt
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

struct OriginalTurnWireServer {
    origin: String,
    credential: PathBuf,
    requests: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<(), String>>>,
}

fn original_wire_request(stream: &mut TcpStream) -> Result<Value, String> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        if Instant::now() >= deadline { return Err("original turn request deadline".into()); }
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 { return Err("closed original turn request headers".into()); }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") { break index + 4; }
        if bytes.len() > 64 * 1024 { return Err("oversized original turn headers".into()); }
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|error| error.to_string())?;
    let length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok()).flatten()
    }).ok_or("original turn request has no content length")?;
    if length > 1024 * 1024 { return Err("oversized original turn request".into()); }
    while bytes.len() < header_end + length {
        if Instant::now() >= deadline { return Err("original turn body deadline".into()); }
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 { return Err("closed original turn request body".into()); }
        bytes.extend_from_slice(&chunk[..count]);
    }
    serde_json::from_slice(&bytes[header_end..header_end + length]).map_err(|error| error.to_string())
}

impl OriginalTurnWireServer {
    fn new(root: &Path, invocations: usize, empty_final: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("private original turn Responses listener");
        listener.set_nonblocking(true).expect("bounded original turn accept");
        let origin = format!("http://{}", listener.local_addr().expect("fixture listener address"));
        let credential = root.join("original-turn-credential.json");
        let mut options = fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&credential).expect("private fixture credential").write_all(json!({
            "version":1, "issuer":origin, "accessToken":"original-turn-fixture-token",
            "expiresAtMs":4_102_444_800_000_u64, "sessionId":"original-turn-fixture-session",
            "deviceId":"original-turn-fixture-device", "userId":"original-turn-fixture-account"
        }).to_string().as_bytes()).expect("write private fixture credential");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(40);
            while !worker_stop.load(Ordering::Acquire) {
                if Instant::now() >= deadline { return Err("original turn Responses deadline".into()); }
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => return Err(error.to_string()),
                };
                let request = original_wire_request(&mut stream)?;
                let ordinal = {
                    let mut requests = captured.lock().map_err(|_| "request capture poisoned")?;
                    let ordinal = requests.len();
                    requests.push(request);
                    ordinal
                };
                let events = if ordinal == 0 {
                    vec![json!({"type":"response.completed", "response":{
                        "id":"original-tool-response", "output":[{
                            "type":"function_call", "name":"b", "call_id":"original-terminal-effect", "arguments":"{}"
                        }]
                    }})]
                } else if empty_final && ordinal == invocations {
                    vec![json!({"type":"response.completed", "response":{"id":"empty-original-result","output":[]}})]
                } else {
                    vec![
                        json!({"type":"response.output_text.delta", "delta":format!("completed invocation {ordinal}")}),
                        json!({"type":"response.completed", "response":{"id":format!("original-response-{ordinal}"),"output":[]}}),
                    ]
                };
                let body = events.iter().map(|event| format!("data: {event}\n\n")).collect::<String>();
                write!(stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len(),
                ).map_err(|error| error.to_string())?;
            }
            Ok(())
        });
        Self { origin, credential, requests, stop, worker: Some(worker) }
    }

    fn finish(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            assert!(worker.is_finished(), "owned Responses worker did not stop within its bound");
            worker.join().expect("original Responses worker panic").expect("original Responses worker result");
        }
    }
}

impl Drop for OriginalTurnWireServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            if worker.is_finished() { let _ = worker.join(); }
        }
    }
}

struct OriginalEffectBridge {
    receipt: PathBuf,
    calls: Arc<AtomicUsize>,
}

impl RoutedToolBridge for OriginalEffectBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "b".into(), provider_identifier: "original-turn-contract".into(),
            tool_name: "b".into(), description: None, input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, _args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        assert_eq!(tool.name, "b");
        assert_eq!(call_id, "original-terminal-effect");
        assert_eq!(self.calls.fetch_add(1, Ordering::AcqRel), 0, "the actual effect must run once");
        fs::OpenOptions::new().write(true).create_new(true).open(&self.receipt)
            .and_then(|mut file| file.write_all(call_id.as_bytes()))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        Ok(json!({"receipt":"committed"}))
    }
}

fn produce_original_turn(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) -> ProducedOriginal {
    let root = &request.fixture_root;
    let runtime = ProductionTranscriptRuntime::new(Some(root));
    let args = json!({
        "agentId":request.agent_id, "streamId":ORIGINAL_STREAM, "prompt":USER_TEXT,
        "sessionKey":format!("agent:{}:main", request.agent_id), "clientNonce":"original-user-nonce",
    });
    let accepted = runtime.accept_routed_send(&args, |accepted| {
        persist_accepted_send_prompt_context(sessions, &args, accepted)
            .map_err(|error| ProductionSendError::Internal(error.to_string()))
    }).expect("actual routed admission and gateway persistence");
    assert!(!accepted.duplicate);
    let admission = sessions.read_session_work_admission(&request.agent_id)
        .expect("actual Session admission").expect("main admission");
    let original = runtime.bind_original_turn_model(&request.agent_id, ORIGINAL_STREAM, &admission, ORIGINAL_MODEL)
        .expect("same admitted lease model binding").expect("ordinary Native user is eligible");
    assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
    if request.original_compaction {
        let entries = sessions.read_agent_transcript_entries(&request.agent_id)
            .expect("actual admitted transcript before same-stream compaction");
        let tail = entries.last().and_then(|entry| entry.get("id")).and_then(Value::as_str)
            .expect("actual transcript tail identity");
        assert_eq!(tail, original.user_message_id);
        let entry = match sessions.commit_agent_compaction(&request.agent_id, &CompactionCommit {
            entry_id: format!("compaction:{}:{}", original.agent_id, original.stream_id),
            expected_transcript_tail_id: tail.to_string(),
            first_kept_entry_id: Some(original.user_message_id.clone()),
            summary: "Retain the accepted original user turn for this same-stream compaction.".into(),
            tokens_before: 0,
            details: None,
            source_turn_id: original.stream_id.clone(),
            expected_lifecycle_revision: original.original_lifecycle_revision.clone(),
        }).expect("real Session compaction commit") {
            CompactionCommitOutcome::Committed(entry) => entry,
            outcome => panic!("same-stream compaction must actually commit: {outcome:?}"),
        };
        assert_eq!(entry["sourceTurnId"].as_str(), Some(original.stream_id.as_str()));
        assert_eq!(entry["sourceLifecycleRevision"].as_str(), Some(original.original_lifecycle_revision.as_str()));
        let successor = sessions.read_session_work_admission(&request.agent_id)
            .expect("actual committed successor admission").expect("same Session remains admitted");
        assert_ne!(successor.lifecycle_revision, original.original_lifecycle_revision);
        assert_eq!(entry["lifecycleRevision"].as_str(), Some(successor.lifecycle_revision.as_str()));
        // Keep the original reference and canonical epoch. The existing Session
        // transaction owns this successor; no metadata or receipt is fabricated.
    }
    let host = composition(root, sessions);
    let sink = host.compose_production_checkpoint_sink(
        sessions, root, &request.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("actual production checkpoint composition").expect("canonical main sink");
    assert!(sink.committed_original_turn_observations().expect("new Journal sink log").is_empty());
    let calls = Arc::new(AtomicUsize::new(0));
    let turn_composition = TurnAgentComposition::new(
        RoutedProvider::Fabushi,
        Arc::new(OriginalEffectBridge { receipt: root.join("original-effect.txt"), calls: Arc::clone(&calls) }),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "original-turn-contract".into(), shell: None, time_zone: Some("UTC".into()),
                transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(), user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(ProductionRoutedProviderCheckpointStore::new(root, &request.agent_id, ORIGINAL_STREAM)),
    ).with_fabushi_model(ORIGINAL_MODEL);
    let mut runner = create_production_runner(
        turn_composition,
        ProductionTurnAgentBuildBindings::new(
            ProductionTurnAgentStaticConfig {
                model_id: ORIGINAL_MODEL.into(), agent_token_limit: SAND_AGENT_TOKEN_LIMIT,
                conversation_id: request.agent_id.clone(), is_box_scoped_subagent: false,
                is_subagent_runner: false, is_shared_room_runner: false,
                sand_send_message_delivery_owed: false, transcripts_folder_available: true,
            },
            Arc::new(|| None), Arc::new(|_, _, _| panic!("bounded original turn must not summarize")),
        ),
        ProductionTurnAgentLifecycleBindings::new(&request.agent_id, ORIGINAL_STREAM),
        Some(sink.clone()), Arc::new(AtomicBool::new(false)),
        Arc::new(Mutex::new(SubagentRuntime::default())), None, None, None, None,
    );
    let messages = vec![ProviderMessage { role: "user".into(), content: USER_TEXT.into() }];
    let options = TurnRunOptions {
        inference_request_id: Some(ORIGINAL_STREAM.into()), message_id: Some(original.user_message_id.clone()),
        recent_message_text: Some(USER_TEXT.into()), turn_started_at_ms: Some(1), ..TurnRunOptions::default()
    };
    let mut custody = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
    let mut results = Vec::new();
    assert!((1..=MAX_REPLY_NUDGES + 2).contains(&request.original_invocations));
    for invocation in 0..request.original_invocations {
        let (input_messages, input_options) = if invocation == 0 {
            (messages.clone(), options.clone())
        } else {
            let shaped = if invocation == MAX_REPLY_NUDGES + 1 {
                shape_closing_send_nudge_turn_input(&messages, &messages, &options)
            } else {
                shape_reply_nudge_turn_input(&messages, &messages, &options)
            };
            (shaped.provider_messages, shaped.options)
        };
        let result = runner.run_routed_provider_with_options(root, &input_messages, input_options, &mut |_, _| {})
            .expect("actual generated Native invocation completed");
        let finished = runner.last_finished().expect("actual shell terminal");
        assert_eq!(finished.owner.request_id, ORIGINAL_STREAM);
        match runner.last_tool_effect_custody().expect("actual generated custody observation") {
            ToolEffectCustodyObservation::Known { run_generation, snapshot } => {
                assert!(*run_generation > 0);
                assert_eq!(*run_generation, finished.owner.generation);
                assert!(snapshot.unsettled_reason().is_none());
                if invocation == 0 { assert!(snapshot.started_revision > 0, "the real tool effect was instrumented"); }
            }
            observation => panic!("generated owner did not produce Known custody: {observation:?}"),
        }
        custody.record(runner.last_tool_effect_custody());
        results.push(result);
    }
    assert_eq!(calls.load(Ordering::Acquire), 1);
    let observations = sink.committed_original_turn_observations().expect("sealed actual Journal commits");
    assert_eq!(observations.len(), request.original_invocations);
    let digest = original_turn_result_digest(results.last().expect("actual final result"));
    let terminal = project_turn_terminal(false, false, true, 0, false);
    // Every negative starts from the same actual owner evidence. No serialized
    // settled custody is used to manufacture a positive completion.
    for kind in [TurnTerminalKind::WaitingUser, TurnTerminalKind::Cancelled, TurnTerminalKind::Failed] {
        assert!(prove_completed_original_turn(&original, TurnTerminalProjection { kind, ..terminal }, digest, 0, false, &custody, &observations).is_err());
    }
    assert!(prove_completed_original_turn(&original, TurnTerminalProjection { delivered: true, ..terminal }, digest, 0, false, &custody, &observations).is_err());
    assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &ToolEffectCustodyReport::new(ORIGINAL_STREAM), &observations).is_err());
    assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &custody, &[]).is_err());
    for replacement in [ToolEffectCustodyObservation::NotStarted, ToolEffectCustodyObservation::Known {
        run_generation: 0,
        snapshot: match &custody.invocations()[0] {
            ToolEffectCustodyObservation::Known { snapshot, .. } => snapshot.clone(), _ => unreachable!(),
        },
    }] {
        let mut refused = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
        refused.record(Some(&replacement));
        for observation in &custody.invocations()[1..] { refused.record(Some(observation)); }
        assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &refused, &observations).is_err());
    }
    for unknown in [false, true] {
        let mut first = custody.invocations()[0].clone();
        if let ToolEffectCustodyObservation::Known { snapshot, .. } = &mut first {
            if unknown { snapshot.unknown_result = Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown); }
            else { snapshot.pending_effects = 1; }
        }
        let mut refused = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
        refused.record(Some(&first));
        for observation in &custody.invocations()[1..] { refused.record(Some(observation)); }
        assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &refused, &observations).is_err());
    }
    let mut foreign = ToolEffectCustodyReport::new("foreign-stream");
    for observation in custody.invocations() { foreign.record(Some(observation)); }
    assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &foreign, &observations).is_err());
    let mut unavailable = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
    unavailable.record(None);
    for observation in &custody.invocations()[1..] { unavailable.record(Some(observation)); }
    assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &unavailable, &observations).is_err());
    let mut foreign_agent = original.clone();
    foreign_agent.agent_id = format!("other-{}", original.agent_id);
    foreign_agent.session_id = foreign_agent.agent_id.clone();
    foreign_agent.session_key = format!("agent:{}:main", foreign_agent.agent_id);
    assert!(prove_completed_original_turn(&foreign_agent, terminal, digest, 0, false, &custody, &observations).is_err());
    let mut overflow_custody = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
    for _ in 0..MAX_REPLY_NUDGES + 3 { overflow_custody.record(Some(&custody.invocations()[0])); }
    assert!(prove_completed_original_turn(
        &original, terminal, digest, 0, false, &overflow_custody,
        &vec![observations[0].clone(); MAX_REPLY_NUDGES + 3],
    ).is_err());
    if request.original_invocations > 1 {
        assert_ne!(results.first(), results.last());
        assert!(prove_completed_original_turn(&original, terminal, original_turn_result_digest(&results[0]), 0, false, &custody, &observations).is_err());
        let mut reversed = observations.clone();
        reversed.reverse();
        assert!(prove_completed_original_turn(&original, terminal, digest, 0, false, &custody, &reversed).is_err());
    }
    let proof = prove_completed_original_turn(&original, terminal, digest, 0, false, &custody, &observations)
        .expect("actual completed owner plus sealed matching commits qualify");
    assert_eq!(persist_completed_original_turn(sessions, &proof).expect("strict Session terminal commit"), OriginalTurnReceiptCommit::Inserted);
    assert_eq!(persist_completed_original_turn(sessions, &proof).expect("same proof is idempotent"), OriginalTurnReceiptCommit::AlreadyCommitted);
    assert_eq!(reconcile_completed_original_turn(sessions, &original).expect("same-process strict recheck").expect("verified completion").original(), &original);
    assert!(runtime.quiesce_for_upgrade().expect("publish the actual bound lease upgrade marker").quiescing);
    let markers = runtime.upgrade_resume_store().expect("actual upgrade marker owner")
        .list_pending_checked().expect("strict producer marker image");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].agent_id, original.agent_id);
    assert_eq!(markers[0].source.as_deref(), Some("turn"));
    assert_eq!(markers[0].original_work.as_ref(), Some(&original));
    runtime.settle_routed_turn(&request.agent_id, ORIGINAL_STREAM, 10).expect("retire actual routed lease");
    let store = sessions.open_agent_store_owner(&request.agent_id).expect("actual final canonical owner");
    let snapshot = store.checkpoint_snapshot_checked().expect("checked final root").expect("committed final state");
    let checkpoint = ExpectedCheckpoint {
        root_blob_id: snapshot.root_blob_id.clone(), state_bytes: Some(snapshot.checkpoint_bytes.clone()),
        last_user_message_id: Some(original.user_message_id.clone()), compaction_epoch: request.expected.compaction_epoch,
        critical_blobs: vec![(snapshot.root_blob_id, snapshot.checkpoint_bytes)],
    };
    drop((runner, sink, host, store, runtime));
    ProducedOriginal { original, checkpoint }
}

fn assert_original_reopen(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) {
    let original = request.original.as_ref().expect("actual producer's reference");
    let result = reconcile_completed_original_turn(sessions, original);
    match request.expectation {
        ReopenExpectation::OriginalReady => {
            let verified = result.expect("fresh strict completion read").expect("durable completion remains available");
            assert_eq!(verified.original(), original);
        }
        ReopenExpectation::OriginalAbsent => assert!(result.expect("strict missing slot read").is_none()),
        ReopenExpectation::OriginalRefuse => assert!(result.is_err(), "unavailable or invalid evidence cannot verify: {result:?}"),
        _ => unreachable!(),
    }
    assert_eq!(fs::read(request.fixture_root.join("original-effect.txt")).expect("original effect receipt"), b"original-terminal-effect");
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
    let produced = match request.expectation {
        ReopenExpectation::Ready => { assert_ready(&request, &owners.sessions); None }
        ReopenExpectation::Refuse => { assert_refused(&request, &owners.sessions); None }
        ReopenExpectation::ProduceOriginal => Some(produce_original_turn(&request, &owners.sessions)),
        ReopenExpectation::OriginalReady | ReopenExpectation::OriginalAbsent | ReopenExpectation::OriginalRefuse => {
            assert_original_reopen(&request, &owners.sessions); None
        }
        ReopenExpectation::OriginalMarkerAck | ReopenExpectation::OriginalMarkerNoop
            | ReopenExpectation::OriginalMarkerPendingAck | ReopenExpectation::OriginalMarkerCorruptAck => {
            panic!("marker acknowledgment must use the bound shipping App executable")
        }
    };
    owners.close();
    let receipt = ChildReceipt {
        token: request.token,
        pid: std::process::id(),
        parent_pid: request.parent_pid,
        expectation: request.expectation,
        produced,
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

fn read_stored_kv(fixture: &Fixture, key: &str) -> String {
    open_existing(&fixture.db_path(), true).query_row(
        "SELECT value FROM kv WHERE key=?1", params![key], |row| row.get(0),
    ).expect("actual persisted owner row")
}

fn replace_stored_kv(fixture: &Fixture, key: &str, raw: &str) {
    assert_eq!(open_existing(&fixture.db_path(), false).execute(
        "UPDATE kv SET value=?2 WHERE key=?1", params![key, raw],
    ).expect("closed-owner fault or exact repair"), 1);
}

fn original_terminal(fixture: &Fixture) -> Value {
    serde_json::from_str(&read_stored_kv(fixture, ORIGINAL_TERMINAL_KEY))
        .expect("actual Runner-persisted original terminal record")
}

fn produced_original_fixture(
    label: &str,
    initial: InitialState,
    legacy_prior: bool,
    invocations: usize,
    empty_final: bool,
    original_compaction: bool,
) -> Fixture {
    let mut fixture = Fixture::new(label, initial);
    if legacy_prior { fixture.rebind_to_legacy_fixed_root(); }
    let prior_identity = fixture.expected.state_bytes.as_ref()
        .map(|_| fixture.expected.root_blob_id.clone());
    fixture.original_invocations = invocations;
    fixture.original_compaction = original_compaction;
    let mut server = OriginalTurnWireServer::new(&fixture.root, invocations, empty_final);
    let receipt = fixture.run_child("produce-original", ReopenExpectation::ProduceOriginal, Some(&server))
        .assert_completed();
    server.finish();
    let requests = server.requests.lock().expect("actual provider request capture");
    assert_eq!(requests.len(), invocations + 1, "one real tool continuation plus each generated invocation");
    assert!(requests.iter().all(|request| request["model"] == ORIGINAL_MODEL));
    drop(requests);
    fs::remove_file(&server.credential).expect("remove producer credential before every recovery consumer");
    let produced = receipt.produced.expect("completed producer reports its actual reference and checkpoint");
    fixture.original = Some(produced.original);
    fixture.expected = produced.checkpoint;
    let marker_store = SandUpgradeResumeStore::new(&fixture.root);
    let mut markers = marker_store.list_pending_checked().expect("producer's actual bound marker after closure");
    assert_eq!(markers.len(), 1);
    let marker = markers.pop().unwrap();
    assert_eq!(marker.agent_id, fixture.agent_id);
    assert_eq!(marker.original_work, fixture.original);
    let marker_image: Value = serde_json::from_slice(&fs::read(marker_store.file_path())
        .expect("actual quiesce marker bytes")).expect("actual quiesce marker envelope");
    assert_eq!(marker_image["version"], 2);
    fixture.original_marker = Some(marker);
    let record = original_terminal(&fixture);
    assert_eq!(record["original"], serde_json::to_value(fixture.original.as_ref().unwrap()).unwrap());
    assert_eq!(record["canonical"].as_array().unwrap().len(), invocations);
    assert_eq!(record["canonical"][0]["priorRootBlobId"], serde_json::to_value(prior_identity).unwrap(),
        "the actual previous root Option and fixed identity survive the commit boundary");
    assert_eq!(record["original"]["modelId"], ORIGINAL_MODEL);
    if empty_final {
        assert_eq!(record["finalResultSha256"], serde_json::to_value(original_turn_result_digest("")).unwrap());
        assert_eq!(record["canonical"][0]["stepBlobIds"].as_array().unwrap().len(), 1,
            "the actual generated tool-only turn has no invented Assistant step");
    }
    assert_eq!(fs::read(fixture.root.join("original-effect.txt")).expect("one actual effect"), b"original-terminal-effect");
    // Checkpoint metadata and admission revisions belong to different mutation
    // domains. A normal generated completion must not require their equality.
    let raw_metadata = read_stored_kv(&fixture, "metadata");
    let metadata_bytes = raw_metadata.as_bytes().chunks_exact(2).map(|pair| {
        u8::from_str_radix(std::str::from_utf8(pair).expect("metadata hex"), 16).expect("metadata byte")
    }).collect::<Vec<_>>();
    let metadata: Value = serde_json::from_slice(&metadata_bytes).expect("actual metadata image");
    assert_ne!(metadata["lifecycleRevision"], record["original"]["originalLifecycleRevision"]);
    let admission: Value = serde_json::from_str(&read_stored_kv(&fixture, "sessionWorkAdmission"))
        .expect("actual final Session admission image");
    assert_eq!(record["settlementLifecycleRevision"], admission["lifecycle_revision"]);
    assert_ne!(metadata["lifecycleRevision"], record["settlementLifecycleRevision"]);
    if original_compaction {
        let original = fixture.original.as_ref().unwrap();
        let compaction_id = format!("compaction:{}:{}", original.agent_id, original.stream_id);
        let raw_entry: String = open_existing(&fixture.db_path(), true).query_row(
            "SELECT entry FROM transcript_entries WHERE id=?1", params![compaction_id], |row| row.get(0),
        ).expect("real committed same-stream compaction row");
        let entry: Value = serde_json::from_str(&raw_entry).expect("actual compaction image");
        assert_eq!(entry["kind"], "compaction");
        assert_eq!(entry["firstKeptEntryId"].as_str(), Some(original.user_message_id.as_str()));
        assert_eq!(entry["sourceTurnId"].as_str(), Some(original.stream_id.as_str()));
        assert_eq!(entry["sourceLifecycleRevision"].as_str(), Some(original.original_lifecycle_revision.as_str()));
        assert_eq!(entry["lifecycleRevision"], record["settlementLifecycleRevision"]);
        assert_ne!(entry["lifecycleRevision"], entry["sourceLifecycleRevision"]);
        assert_eq!(record["compaction"], json!({
            "entryId": entry["id"], "sourceTurnId": entry["sourceTurnId"],
            "sourceLifecycleRevision": entry["sourceLifecycleRevision"],
            "lifecycleRevision": entry["lifecycleRevision"],
        }), "terminal binding must exactly name the actual Session transaction");
    } else {
        assert!(record["compaction"].is_null());
        assert_eq!(record["settlementLifecycleRevision"], record["original"]["originalLifecycleRevision"]);
    }
    fixture
}

#[test]
fn original_turn_real_generated_owner_reconciles_across_closed_processes_and_legacy_prior_roots() {
    for (label, initial, legacy, invocations, empty_final) in [
        ("original-no-prior", InitialState::NoCheckpoint, false, 1, false),
        ("original-present-empty", InitialState::EmptyCheckpoint, false, 1, false),
        ("original-fixed-prior", InitialState::UserCheckpoint, true, 2, false),
        ("original-all-nudges", InitialState::NoCheckpoint, false, MAX_REPLY_NUDGES + 2, false),
        ("original-tool-only", InitialState::NoCheckpoint, false, 1, true),
    ] {
        let fixture = produced_original_fixture(label, initial, legacy, invocations, empty_final, false);
        let before = durable_state(&fixture);
        assert!(!before.mirror.is_empty(), "the producer committed an actual Journal mirror");
        for attempt in 1..=2 {
            fixture.reopen(&format!("original-verified-{attempt}"), ReopenExpectation::OriginalReady).assert_completed();
            assert_eq!(durable_state(&fixture), before,
                "reconciliation must not dispatch, replay an effect, rewrite a checkpoint or normalize owner rows");
        }
    }
}

#[test]
fn original_turn_real_generated_completion_acknowledges_exact_marker_in_fresh_app_after_ack_repair() {
    for (label, compaction) in [
        ("original-app-ack", false),
        ("original-app-ack-compaction", true),
    ] {
        let fixture = produced_original_fixture(label, InitialState::NoCheckpoint, false, 2, false, compaction);
        let before = durable_state(&fixture);
        assert!(!before.mirror.is_empty(), "actual generated Journal history is present");
        let effect_path = fixture.root.join("original-effect.txt");
        let effect_before = fs::read(&effect_path).expect("actual generated effect receipt");
        let provider_root = fixture.root.join("runner-provider-checkpoints");
        let provider_before = mirror_files(&provider_root);
        assert!(!provider_before.is_empty(), "the producer created real provider checkpoints");
        // The loopback provider has been joined and its credential removed.
        // App consumers also receive an unavailable URL and credential, and
        // observe their actual lifecycle owner to reject any new turn dispatch.
        assert!(!fixture.root.join("original-turn-credential.json").exists());
        let marker_path = SandUpgradeResumeStore::new(&fixture.root).file_path().to_path_buf();
        let marker_before = fs::read(&marker_path).expect("actual bound v2 quiesce marker");
        let (ack_path, pending_obligation) = {
            let ack = SandAckObligationStore::new(&fixture.root);
            let recorded = ack.record_send(&fixture.agent_id, 11.0)
                .expect("real independent ACK owner records pending work");
            assert!(recorded.created);
            assert_eq!(ack.get_checked(&fixture.agent_id).unwrap(), Some(recorded.obligation.clone()));
            (ack.file_path().to_path_buf(), recorded.obligation)
        };
        let pending_ack = fs::read(&ack_path).expect("real pending ACK owner image");
        let assert_original_unchanged = || {
            assert_eq!(durable_state(&fixture), before,
                "fresh App acknowledgment cannot rewrite canonical, user, result, admission, receipt or Journal bytes");
            assert_eq!(fs::read(&effect_path).unwrap(), effect_before, "no duplicate original tool effect");
            assert_eq!(mirror_files(&provider_root), provider_before,
                "recovery cannot create another provider request or alter prior provider checkpoints");
        };

        let pending = fixture.reopen("app-pending-ack", ReopenExpectation::OriginalMarkerPendingAck)
            .assert_completed();
        assert!(pending.produced.is_none());
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before);
        assert_eq!(fs::read(&ack_path).unwrap(), pending_ack);
        assert_original_unchanged();

        // Inject corruption only after all owning processes close. A failed
        // strict ACK read must preserve both the damaged bytes and the marker.
        fs::write(&ack_path, b"{").expect("closed-owner independent ACK corruption");
        let corrupt = fixture.reopen("app-corrupt-ack", ReopenExpectation::OriginalMarkerCorruptAck)
            .assert_completed();
        assert!(corrupt.produced.is_none());
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before);
        assert_eq!(fs::read(&ack_path).unwrap(), b"{");
        assert_original_unchanged();

        fs::write(&ack_path, &pending_ack).expect("explicitly repair the exact captured ACK image");
        assert_eq!(fs::read(&ack_path).unwrap(), pending_ack);
        {
            let ack = SandAckObligationStore::new(&fixture.root);
            assert_eq!(ack.get_checked(&fixture.agent_id).unwrap(), Some(pending_obligation));
            assert!(ack.clear(&fixture.agent_id).expect("explicit owner completion of independent ACK work"));
            assert!(ack.get_checked(&fixture.agent_id).unwrap().is_none());
        }
        let cleared_ack = fs::read(&ack_path).expect("actual ACK owner wrote its empty image");
        assert_ne!(cleared_ack, pending_ack);
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before);
        assert_original_unchanged();

        let acknowledged = fixture.reopen("app-exact-marker-ack", ReopenExpectation::OriginalMarkerAck)
            .assert_completed();
        assert!(acknowledged.produced.is_none());
        assert!(!marker_path.exists(), "the shipping App consumed exactly the actual original marker");
        assert_eq!(fs::read(&ack_path).unwrap(), cleared_ack);
        assert_original_unchanged();

        let repeated = fixture.reopen("app-fresh-marker-noop", ReopenExpectation::OriginalMarkerNoop)
            .assert_completed();
        assert!(repeated.produced.is_none());
        assert!(!marker_path.exists(), "a second fresh App process leaves the consumed marker absent");
        assert_eq!(fs::read(&ack_path).unwrap(), cleared_ack);
        assert_original_unchanged();
    }
}

#[test]
fn original_turn_real_generated_receipt_refuses_malformed_custody_and_canonical_identity_in_new_process() {
    let fixture = produced_original_fixture("original-record-faults", InitialState::NoCheckpoint, false, 2, false, false);
    let healthy = durable_state(&fixture);
    let raw = read_stored_kv(&fixture, ORIGINAL_TERMINAL_KEY);
    let record: Value = serde_json::from_str(&raw).expect("actual terminal JSON");
    for fault in [
        "unsupported-version", "wrong-terminal", "wrong-request", "wrong-user", "wrong-order",
        "wrong-final-result", "missing-commit", "foreign-stream", "not-started", "zero-generation",
        "pending", "unknown-result", "unknown-nested-field", "duplicate-custody-field", "malformed", "oversized",
    ] {
        let mut damaged = record.clone();
        let mut custody: Value = serde_json::from_str(damaged["custodyJson"].as_str().unwrap()).unwrap();
        match fault {
            "unsupported-version" => damaged["version"] = json!(2),
            "wrong-terminal" => damaged["terminal"] = json!("failed"),
            "wrong-request" => damaged["canonical"][0]["requestId"] = json!("different-stream"),
            "wrong-user" => damaged["canonical"][0]["messageId"] = json!("different-user"),
            "wrong-order" => damaged["canonical"].as_array_mut().unwrap().reverse(),
            "wrong-final-result" => damaged["finalResultSha256"] = json!(vec![0_u8; 32]),
            "missing-commit" => { damaged["canonical"].as_array_mut().unwrap().remove(0); }
            "foreign-stream" => custody["streamId"] = json!("different-stream"),
            "not-started" => custody["invocations"][0] = json!({"state":"notStarted"}),
            "zero-generation" => custody["invocations"][0]["runGeneration"] = json!(0),
            "pending" => custody["invocations"][0]["snapshot"]["pendingEffects"] = json!(1),
            "unknown-result" => custody["invocations"][0]["snapshot"]["unknownResult"] = json!("downstreamOutcomeUnknown"),
            "unknown-nested-field" => custody["invocations"][0]["snapshot"]["unsupportedAuthority"] = json!(true),
            "duplicate-custody-field" | "malformed" | "oversized" => {}
            _ => unreachable!(),
        }
        damaged["custodyJson"] = if fault == "duplicate-custody-field" {
            json!(format!("{{\"streamId\":{},\"streamId\":{},\"invocations\":{}}}",
                custody["streamId"], custody["streamId"], custody["invocations"]))
        } else {
            json!(serde_json::to_string(&custody).unwrap())
        };
        let damaged_raw = match fault {
            "malformed" => "{".to_string(),
            "oversized" => " ".repeat(64 * 1024 + 1),
            _ => serde_json::to_string(&damaged).unwrap(),
        };
        replace_stored_kv(&fixture, ORIGINAL_TERMINAL_KEY, &damaged_raw);
        let damaged_state = durable_state(&fixture);
        assert_ne!(damaged_state, healthy, "{fault}: fault must affect the actual stored proof data");
        fixture.reopen(fault, ReopenExpectation::OriginalRefuse).assert_completed();
        assert_eq!(durable_state(&fixture), damaged_state, "{fault}: refusal preserves exact damaged owner bytes");
        replace_stored_kv(&fixture, ORIGINAL_TERMINAL_KEY, &raw);
        assert_eq!(durable_state(&fixture), healthy, "{fault}: explicit repair restores exact original bytes");
    }
    fixture.reopen("original-record-repaired", ReopenExpectation::OriginalReady).assert_completed();
    assert_eq!(durable_state(&fixture), healthy);
}

#[test]
fn original_turn_real_generated_receipt_rechecks_actual_blobs_user_and_admission_after_restart() {
    let fixture = produced_original_fixture("original-source-faults", InitialState::NoCheckpoint, false, 2, false, false);
    let healthy = durable_state(&fixture);
    let record = original_terminal(&fixture);
    let first = &record["canonical"][0];
    for (label, id) in [
        ("root", serde_json::from_value::<Vec<u8>>(record["canonical"][1]["rootBlobId"].clone()).unwrap()),
        ("turn", serde_json::from_value(first["turnBlobId"].clone()).unwrap()),
        ("user", serde_json::from_value(first["userBlobId"].clone()).unwrap()),
        ("tool", serde_json::from_value(first["stepBlobIds"][0].clone()).unwrap()),
        ("assistant", serde_json::from_value(first["stepBlobIds"].as_array().unwrap().last().unwrap().clone()).unwrap()),
    ] {
        let db = open_existing(&fixture.blob_path(), false);
        let original: Vec<u8> = db.query_row("SELECT data FROM blobs WHERE id=?1", params![hex(&id)], |row| row.get(0))
            .expect("actual generated blob bytes");
        let mut changed = original.clone();
        *changed.last_mut().expect("generated root/turn/user/step has bytes") ^= 1;
        // Keep the wire framing intact while changing its content under the
        // original ID, so this exercises content addressing as well as decoding.
        assert_eq!(db.execute("UPDATE blobs SET data=?2 WHERE id=?1", params![hex(&id), changed]).unwrap(), 1);
        drop(db);
        let damaged = durable_state(&fixture);
        fixture.reopen(&format!("damaged-{label}"), ReopenExpectation::OriginalRefuse).assert_completed();
        assert_eq!(durable_state(&fixture), damaged, "{label}: checked read cannot repair canonical bytes");
        assert_eq!(open_existing(&fixture.blob_path(), false).execute(
            "UPDATE blobs SET data=?2 WHERE id=?1", params![hex(&id), original],
        ).unwrap(), 1);
        assert_eq!(durable_state(&fixture), healthy);
    }
    let missing_tool: Vec<u8> = serde_json::from_value(first["stepBlobIds"][0].clone()).unwrap();
    let db = open_existing(&fixture.blob_path(), false);
    let tool_bytes: Vec<u8> = db.query_row("SELECT data FROM blobs WHERE id=?1", params![hex(&missing_tool)], |row| row.get(0)).unwrap();
    assert_eq!(db.execute("DELETE FROM blobs WHERE id=?1", params![hex(&missing_tool)]).unwrap(), 1);
    drop(db);
    let missing = durable_state(&fixture);
    fixture.reopen("missing-recorded-tool", ReopenExpectation::OriginalRefuse).assert_completed();
    assert_eq!(durable_state(&fixture), missing);
    open_existing(&fixture.blob_path(), false).execute("INSERT INTO blobs(id,data) VALUES(?1,?2)",
        params![hex(&missing_tool), tool_bytes]).unwrap();
    assert_eq!(durable_state(&fixture), healthy);
    let original = fixture.original.as_ref().unwrap();
    let db = open_existing(&fixture.db_path(), false);
    let raw_user: String = db.query_row("SELECT entry FROM transcript_entries WHERE id=?1",
        params![original.user_message_id], |row| row.get(0)).expect("actual confirmed user row");
    let mut changed_user: Value = serde_json::from_str(&raw_user).unwrap();
    changed_user["content"] = json!(format!("{USER_TEXT} "));
    assert_eq!(db.execute("UPDATE transcript_entries SET entry=?2 WHERE id=?1",
        params![original.user_message_id, serde_json::to_string(&changed_user).unwrap()]).unwrap(), 1);
    drop(db);
    let damaged = durable_state(&fixture);
    fixture.reopen("exact-user-content-changed", ReopenExpectation::OriginalRefuse).assert_completed();
    assert_eq!(durable_state(&fixture), damaged, "even one trailing byte cannot be normalized away");
    open_existing(&fixture.db_path(), false).execute("UPDATE transcript_entries SET entry=?2 WHERE id=?1",
        params![original.user_message_id, raw_user]).unwrap();
    let raw_admission = read_stored_kv(&fixture, "sessionWorkAdmission");
    for (label, field, value) in [
        ("independent-revision", "lifecycle_revision", json!("foreign-revision")),
        ("archived", "archived_at_ms", json!(1.0)),
        ("background-custody", "admitted_work_id", json!("independent-background-work")),
        ("media-custody", "admitted_media_work_id", json!("independent-media-work")),
        ("provider-review", "provider_review_id", json!("independent-review")),
    ] {
        let mut admission: Value = serde_json::from_str(&raw_admission).unwrap();
        admission[field] = value;
        replace_stored_kv(&fixture, "sessionWorkAdmission", &serde_json::to_string(&admission).unwrap());
        let damaged = durable_state(&fixture);
        fixture.reopen(label, ReopenExpectation::OriginalRefuse).assert_completed();
        assert_eq!(durable_state(&fixture), damaged, "{label}: independent custody must be retained");
        replace_stored_kv(&fixture, "sessionWorkAdmission", &raw_admission);
        assert_eq!(durable_state(&fixture), healthy);
    }
    fixture.reopen("original-source-repaired", ReopenExpectation::OriginalReady).assert_completed();
    assert_eq!(durable_state(&fixture), healthy);
}

#[test]
fn original_turn_restart_refuses_recovery_unavailability_before_complete_or_absent_terminal_slot() {
    let fixture = produced_original_fixture("original-recovery-unavailable", InitialState::NoCheckpoint, false, 1, false, false);
    let healthy = durable_state(&fixture);
    let raw = read_stored_kv(&fixture, ORIGINAL_TERMINAL_KEY);
    for has_slot in [true, false] {
        if !has_slot {
            assert_eq!(open_existing(&fixture.db_path(), false).execute(
                "DELETE FROM kv WHERE key=?1", params![ORIGINAL_TERMINAL_KEY],
            ).unwrap(), 1);
            fixture.reopen("healthy-absent-slot", ReopenExpectation::OriginalAbsent).assert_completed();
        }
        for marker_kind in ["filesystem", "kv"] {
            let marker = automation_custody_recovery_marker_path(&fixture.db_path());
            let marker_bytes = br#"{"version":1,"reason":"cron_authority_not_recovered"}"#;
            if marker_kind == "filesystem" {
                fs::write(&marker, marker_bytes).expect("closed-owner recovery fault injection");
            } else {
                open_existing(&fixture.db_path(), false).execute(
                    "INSERT INTO kv(key,value) VALUES(?1,?2)",
                    params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, std::str::from_utf8(marker_bytes).unwrap()],
                ).unwrap();
            }
            let damaged = durable_state(&fixture);
            fixture.reopen(&format!("recovery-{marker_kind}-{has_slot}"), ReopenExpectation::OriginalRefuse).assert_completed();
            assert_eq!(durable_state(&fixture), damaged, "recovery refusal must not salvage, clear or reissue terminal authority");
            if marker_kind == "filesystem" {
                assert_eq!(fs::read(&marker).unwrap(), marker_bytes);
                fs::remove_file(&marker).expect("explicit removal of only the injected fixture marker");
            } else {
                assert_eq!(open_existing(&fixture.db_path(), false).execute(
                    "DELETE FROM kv WHERE key=?1", params![AUTOMATION_CUSTODY_UNAVAILABLE_KV],
                ).unwrap(), 1);
            }
        }
        if !has_slot {
            open_existing(&fixture.db_path(), false).execute(
                "INSERT INTO kv(key,value) VALUES(?1,?2)", params![ORIGINAL_TERMINAL_KEY, raw],
            ).unwrap();
        }
        assert_eq!(durable_state(&fixture), healthy);
    }
    fixture.reopen("recovery-explicitly-restored", ReopenExpectation::OriginalReady).assert_completed();
}
