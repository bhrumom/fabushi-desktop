//! SWA-ROOT-RECOVERY-v1: checkpoint readiness across an actual OS-process
//! boundary. The parent materializes and commits through shipping owners, then
//! closes them before the child opens the same files. These are storage and
//! Host-composition prerequisites. ORT-v1 cases additionally run the actual
//! generated Native owner against bounded loopback Responses, then reconcile
//! and acknowledge its actual upgrade marker in the shipping App after that
//! process and all of its Session owners have closed.
//! GTH-v1 cases retain known completed tool history after an actual generated
//! failure, then open it in fresh Host processes without a completion grant.
//! CRA-v1 cases bind actual checked root identity from admission through commit,
//! including controlled competing writes and recovery in fresh Host processes.

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
    McpCatalogKind, McpCatalogReason, McpCatalogRefusal, ProviderMessage, ProviderSessionError,
    RoutedProvider, RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::{
    ProductionAgentCheckpointSnapshot, ProductionAgentMetadataKey, ProductionAgentStore,
};
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
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, ROUTED_TOOL_EXECUTE_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, CommittedOriginalTurnObservation, TextTurnCheckpointArtifacts,
    original_turn_result_digest, persist_completed_original_turn,
    prove_completed_original_turn, reconcile_completed_original_turn,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentOwner, ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    RunnerRequestContextSnapshot, coordinator_tool_execution_error,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::ProviderRetryOutcome;
use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
use mahayana_host_runtime::runner::turn_agent_composition::{SAND_AGENT_TOKEN_LIMIT, TurnAgentComposition};
use mahayana_host_runtime::runner::{
    TerminalOutcome, ToolEffectCustodyObservation, ToolEffectCustodyReport,
    ToolEffectCustodySnapshot, ToolEffectUnknownResult, TurnRunOptions,
};
use mahayana_host_runtime::runner_production_bridge::create_production_runner;
use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
    ConversationTurnStructureFields, decode_conversation_state_recovery_fields,
    decode_conversation_turn_structure_fields, decode_user_message_identity_fields,
};
use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    CanonicalGeneratedToolJsonProjection, GeneratedTranscriptOccurrenceCodec,
};
use mahayana_host_runtime::transcript_mirror::transcript_occurrence_deriver::{
    DecodedTranscriptStep, TranscriptOccurrenceCodec,
};
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
const GENERATED_PROVIDER_FAILURE: &str = "generated history permanent provider failure";
const GENERATED_PARTIAL_TEXT: &str = "partial failed continuation must not become a final assistant";
const GENERATED_CANCEL_REASON: &str = "cancel the admitted generated turn after its real effect";
const GENERATED_PRIOR_TEXT: &str = "completed earlier generated invocation";
const GENERATED_STORE_OBSTRUCTION: &[u8] = b"unrelated occupied provider checkpoint entry";
const GENERATED_REFUSED_WRITE_TEXT: &str = "completed generation whose final write must be refused";
const GENERATED_REPAIRED_WRITE_TEXT: &str = "completed generated invocation after explicit storage repair";
const GENERATED_FINAL_WRITE_REFUSAL: &str = "generated history fixture final-write refusal";
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    generated_scenario: Option<GeneratedHistoryScenario>,
    canonical_scenario: Option<CanonicalRootScenario>,
    canonical_turns: Option<Vec<CanonicalExpectedTurn>>,
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
                let expected_base = sink.checked_base_snapshot().expect("capture actual fixture base once");
                let checkpoint = sink.stage_text_turn(
                    &expected_base,
                    &[ProviderMessage { role: "user".into(), content: USER_TEXT.into() }],
                    &options,
                    ASSISTANT_TEXT,
                ).expect("stage through the actual Runner checkpoint helper");
                sink.persist_staged_text_turn(&expected_base, &options, &checkpoint)
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
            generated_scenario: None,
            canonical_scenario: None, canonical_turns: None,
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
            generated_scenario: self.generated_scenario,
            canonical_scenario: self.canonical_scenario,
            canonical_turns: self.canonical_turns.clone(),
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
    ProduceGeneratedFailure,
    GeneratedHistoryReady,
    ProduceCanonicalRoot,
    CanonicalRootReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum GeneratedHistoryScenario {
    ProviderFailure,
    OrderedProviderFailure,
    Cancellation,
    ProviderStoreRefusal,
    UnknownDownstream,
    ZeroEffectFailure,
    RootDrift,
    LaterFailure,
    FinalPersistenceRefusal,
}

impl GeneratedHistoryScenario {
    fn tool_count(self) -> usize {
        match self {
            Self::ZeroEffectFailure => 0,
            Self::OrderedProviderFailure => 2,
            _ => 1,
        }
    }

    fn request_count(self) -> usize {
        match self {
            Self::ProviderStoreRefusal | Self::UnknownDownstream | Self::ZeroEffectFailure => 1,
            Self::LaterFailure => 3,
            Self::FinalPersistenceRefusal => 4,
            _ => 2,
        }
    }

    fn retains_history(self) -> bool {
        !matches!(self, Self::UnknownDownstream | Self::ZeroEffectFailure | Self::RootDrift)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum CanonicalBase { Absent, Empty, Legacy }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum CanonicalDrift { AbsentToEmpty, EmptyToAbsent, SameBytesOtherRoot, SameRootOtherBytes }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum CanonicalEarlyError { Build, Prompt, Catalog, Projection, Preflight }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum CanonicalRootScenario {
    Valid(CanonicalBase),
    PreProjection(CanonicalDrift),
    PostProjection,
    StrictRead(Fault),
    MissingDirectTurn,
    RetiredOwners,
    SameSinkBeforeStage,
    SameSinkAfterStage,
    Early(CanonicalEarlyError),
    JournalAfterCommit,
}

impl CanonicalRootScenario {
    fn request_count(self) -> usize {
        match self {
            Self::Valid(_) | Self::SameSinkBeforeStage | Self::SameSinkAfterStage
                | Self::Early(_) | Self::JournalAfterCommit => 2,
            _ => 0,
        }
    }

    fn initial_state(self) -> InitialState {
        match self {
            Self::Valid(CanonicalBase::Absent) | Self::PreProjection(CanonicalDrift::AbsentToEmpty) => InitialState::NoCheckpoint,
            Self::StrictRead(_) | Self::MissingDirectTurn => InitialState::UserCheckpoint,
            _ => InitialState::EmptyCheckpoint,
        }
    }

    fn uses_legacy_root(self) -> bool {
        matches!(self, Self::Valid(CanonicalBase::Legacy)
            | Self::PreProjection(CanonicalDrift::SameBytesOtherRoot | CanonicalDrift::SameRootOtherBytes))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CanonicalExpectedTurn {
    message_id: String,
    user_text: String,
    request_id: String,
    assistant_text: String,
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
    #[serde(default)]
    generated_scenario: Option<GeneratedHistoryScenario>,
    #[serde(default)]
    canonical_scenario: Option<CanonicalRootScenario>,
    #[serde(default)]
    canonical_turns: Option<Vec<CanonicalExpectedTurn>>,
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
    #[serde(default)]
    canonical_turns: Option<Vec<CanonicalExpectedTurn>>,
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
    durable_state_for(&fixture.root, &fixture.agent_id)
}

fn durable_state_for(root: &Path, agent_id: &str) -> DurableState {
    // Compare stored row/payload bytes, not SQLite page/WAL layout. Read-only
    // observations cannot repair, normalize or create the subject databases.
    let directory = root.join("agents").join(agent_id);
    let db = open_existing(&directory.join(STORE_FILENAME), true);
    let kv = db.prepare("SELECT key,CAST(value AS BLOB) FROM kv ORDER BY key")
        .expect("kv query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?))).expect("kv rows")
        .collect::<Result<Vec<_>, _>>().expect("kv bytes");
    let transcript = db.prepare("SELECT seq,id,CAST(entry AS BLOB) FROM transcript_entries ORDER BY seq")
        .expect("transcript query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).expect("transcript rows")
        .collect::<Result<Vec<_>, _>>().expect("transcript bytes");
    let blob_db = open_existing(&directory.join(CONVERSATION_BLOBS_FILENAME), true);
    let blobs = blob_db.prepare("SELECT id,typeof(data),CAST(data AS BLOB) FROM blobs ORDER BY id")
        .expect("blob query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).expect("blob rows")
        .collect::<Result<Vec<_>, _>>().expect("all critical and remaining blob bytes");
    DurableState { kv, transcript, blobs, mirror: mirror_files(&root.join("transcripts")) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    stages: Arc<Mutex<Vec<(usize, &'static str)>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<(), String>>>,
    outcome: Option<Result<(), String>>,
}

fn original_process_stage(token: &str, stage: &str) {
    eprintln!("checkpoint child token={token} pid={} stage={stage}", std::process::id());
}

fn original_wire_stage(
    stages: &Mutex<Vec<(usize, &'static str)>>,
    ordinal: usize,
    stage: &'static str,
) {
    let mut stages = stages.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if stages.len() < 128 {
        stages.push((ordinal, stage));
    } else if let Some(last) = stages.last_mut() {
        *last = (ordinal, "stage-capture-limit");
    }
}

fn original_wire_join_result(
    joined: thread::Result<Result<(), String>>,
) -> Result<(), String> {
    match joined {
        Ok(result) => result,
        Err(payload) => {
            let message = payload.downcast_ref::<&str>().copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("non-string panic payload");
            Err(format!("original Responses worker panicked: {}",
                message.chars().take(1024).collect::<String>()))
        }
    }
}

fn original_wire_request(
    stream: &mut TcpStream,
    ordinal: usize,
    stages: &Mutex<Vec<(usize, &'static str)>>,
) -> Result<Value, String> {
    // Match the existing model/provider fixtures when accepted sockets inherit
    // the listener's nonblocking mode. Both directions remain timeout-bounded.
    stream.set_nonblocking(false)
        .map_err(|error| format!("original turn request blocking mode: {error}"))?;
    original_wire_stage(stages, ordinal, "blocking-mode");
    stream.set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("original turn request read timeout: {error}"))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("original turn request write timeout: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    original_wire_stage(stages, ordinal, "reading-headers");
    let header_end = loop {
        if Instant::now() >= deadline { return Err("original turn request deadline".into()); }
        let count = stream.read(&mut chunk)
            .map_err(|error| format!("original turn request headers: {error}"))?;
        if count == 0 { return Err("closed original turn request headers".into()); }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") { break index + 4; }
        if bytes.len() > 64 * 1024 { return Err("oversized original turn headers".into()); }
    };
    original_wire_stage(stages, ordinal, "headers-complete");
    let headers = std::str::from_utf8(&bytes[..header_end])
        .map_err(|error| format!("original turn request header encoding: {error}"))?;
    let length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok()).flatten()
    }).ok_or("original turn request headers: missing or invalid content length")?;
    if length > 1024 * 1024 { return Err("oversized original turn request".into()); }
    original_wire_stage(stages, ordinal, "reading-body");
    while bytes.len() < header_end + length {
        if Instant::now() >= deadline { return Err("original turn body deadline".into()); }
        let count = stream.read(&mut chunk)
            .map_err(|error| format!("original turn request body: {error}"))?;
        if count == 0 { return Err("closed original turn request body".into()); }
        bytes.extend_from_slice(&chunk[..count]);
    }
    original_wire_stage(stages, ordinal, "body-complete");
    serde_json::from_slice(&bytes[header_end..header_end + length])
        .map_err(|error| format!("original turn request JSON: {error}"))
}

impl OriginalTurnWireServer {
    fn new(root: &Path, invocations: usize, empty_final: bool) -> Self {
        Self::with_generated_scenario(root, invocations, empty_final, None, None)
    }

    fn generated(root: &Path, scenario: GeneratedHistoryScenario) -> Self {
        Self::with_generated_scenario(root, 1, false, Some(scenario), None)
    }

    fn canonical(root: &Path, scenario: CanonicalRootScenario) -> Self {
        Self::with_generated_scenario(root, 1, false, None, Some(scenario))
    }

    fn with_generated_scenario(
        root: &Path,
        invocations: usize,
        empty_final: bool,
        generated_scenario: Option<GeneratedHistoryScenario>,
        canonical_scenario: Option<CanonicalRootScenario>,
    ) -> Self {
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
        let stages = Arc::new(Mutex::new(Vec::new()));
        original_wire_stage(&stages, 0, "listener-ready");
        let worker_stages = Arc::clone(&stages);
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
                    Err(error) => return Err(format!("original turn Responses accept: {error}")),
                };
                let ordinal = captured.lock().map_err(|_| "request capture poisoned")?.len();
                original_wire_stage(&worker_stages, ordinal, "accepted");
                let request = original_wire_request(&mut stream, ordinal, &worker_stages)?;
                captured.lock().map_err(|_| "request capture poisoned")?.push(request);
                original_wire_stage(&worker_stages, ordinal, "request-parsed");
                let events = if let Some(scenario) = canonical_scenario {
                    if ordinal >= scenario.request_count() {
                        return Err(format!("unexpected canonical-root provider request {ordinal}"));
                    }
                    vec![
                        json!({"type":"response.output_text.delta", "delta":canonical_reply(ordinal)}),
                        json!({"type":"response.completed", "response":{"id":format!("canonical-root-{ordinal}"), "output":[]}}),
                    ]
                } else if let Some(scenario) = generated_scenario {
                    generated_history_wire_events(scenario, ordinal)?
                } else if ordinal == 0 {
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
                original_wire_stage(&worker_stages, ordinal, "writing-response");
                write!(stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len(),
                ).map_err(|error| format!("original turn response write: {error}"))?;
                original_wire_stage(&worker_stages, ordinal, "response-written");
            }
            Ok(())
        });
        Self { origin, credential, requests, stages, stop, worker: Some(worker), outcome: None }
    }

    fn diagnostics(&self) -> String {
        let requests = self.requests.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).len();
        let stages = self.stages.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        format!("parsed_requests={requests} stages={stages:?}")
    }

    fn finish(&mut self) -> Result<(), String> {
        self.stop.store(true, Ordering::Release);
        if let Some(outcome) = self.outcome.as_ref() {
            let outcome = outcome.clone();
            if self.worker.as_ref().is_some_and(|worker| worker.is_finished()) {
                if let Some(worker) = self.worker.take() {
                    let late = original_wire_join_result(worker.join());
                    eprintln!("late original Responses outcome={late:?}; retained outcome={outcome:?}");
                }
            }
            return outcome;
        }
        let outcome = if let Some(worker) = self.worker.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(2));
            }
            if worker.is_finished() {
                original_wire_join_result(worker.join())
            } else {
                // Keep the unresolved handle and the failure, without an
                // unbounded join or replacing a timeout with success in Drop.
                self.worker = Some(worker);
                Err("owned Responses worker did not stop within its five-second bound".into())
            }
        } else {
            Ok(())
        };
        self.outcome = Some(outcome.clone());
        outcome
    }
}

impl Drop for OriginalTurnWireServer {
    fn drop(&mut self) {
        let outcome = self.finish();
        if outcome.is_err() || thread::panicking() {
            eprintln!("original Responses cleanup outcome={outcome:?} {}", self.diagnostics());
        }
    }
}

#[cfg(unix)]
#[test]
fn original_turn_wire_reader_clears_forced_nonblocking_and_preserves_json() {
    use std::os::fd::AsRawFd;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    client.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    let before = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
    assert!(before >= 0);
    assert_ne!(before & libc::O_NONBLOCK, 0);
    let expected = json!({"model":ORIGINAL_MODEL,"input":[{"content":"original request"}]});
    let body = serde_json::to_vec(&expected).unwrap();
    client.write_all(format!(
        "POST /v1/responses HTTP/1.1\r\nContent-Length: {}\r\n\r\n", body.len(),
    ).as_bytes()).unwrap();
    client.write_all(&body).unwrap();

    let stages = Mutex::new(Vec::new());
    assert_eq!(original_wire_request(&mut stream, 0, &stages).expect("bounded original turn reader"), expected);
    let after = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
    assert!(after >= 0);
    assert_eq!(after & libc::O_NONBLOCK, 0, "the actual reader restores blocking I/O");
    assert_eq!(stream.read_timeout().unwrap(), Some(Duration::from_secs(2)));
    assert_eq!(stream.write_timeout().unwrap(), Some(Duration::from_secs(2)));
}

#[test]
fn original_turn_wire_worker_retains_request_error_without_header_contents() {
    let fixture = Fixture::new("original-wire-worker-error", InitialState::NoCheckpoint);
    let mut server = OriginalTurnWireServer::new(&fixture.root, 1, false);
    let mut client = TcpStream::connect(server.origin.strip_prefix("http://").unwrap()).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    client.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    client.write_all(b"POST /v1/responses HTTP/1.1\r\nAuthorization: private-fixture-marker\r\n\r\n").unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(client.read(&mut byte).expect("worker closes the refused request"), 0);
    let error = server.finish().expect_err("a joined failing worker cannot become success");
    assert_eq!(error, "original turn request headers: missing or invalid content length");
    assert!(!error.contains("private-fixture-marker"));
    assert!(server.requests.lock().unwrap().is_empty());
    assert!(server.worker.is_none(), "the exact failed worker was joined");
    assert_eq!(server.finish().expect_err("later cleanup must retain the joined failure"), error);
    assert!(!server.diagnostics().contains("private-fixture-marker"));
    assert!(server.worker.is_none(), "repeated finish cannot invent another worker");
}

fn generated_history_call_id(ordinal: usize) -> String {
    format!("generated-history-effect-{ordinal}")
}

fn generated_history_wire_events(
    scenario: GeneratedHistoryScenario,
    ordinal: usize,
) -> Result<Vec<Value>, String> {
    if ordinal >= scenario.request_count() {
        return Err(format!("unexpected generated history provider request {ordinal}"));
    }
    if ordinal == 0 && scenario.tool_count() != 0 {
        let calls = (0..scenario.tool_count()).map(|index| json!({
            "type":"function_call", "name":"b", "call_id":generated_history_call_id(index),
            "arguments":json!({"ordinal":index.to_string()}).to_string(),
        })).collect::<Vec<_>>();
        return Ok(vec![json!({"type":"response.completed", "response":{
            "id":"generated-history-tools", "output":calls,
        }})]);
    }
    if matches!(scenario, GeneratedHistoryScenario::LaterFailure | GeneratedHistoryScenario::FinalPersistenceRefusal)
        && ordinal == 1
    {
        return Ok(vec![
            json!({"type":"response.output_text.delta", "delta":GENERATED_PRIOR_TEXT}),
            json!({"type":"response.completed", "response":{"id":"prior-generated-success", "output":[]}}),
        ]);
    }
    if scenario == GeneratedHistoryScenario::FinalPersistenceRefusal {
        let text = if ordinal == 2 { GENERATED_REFUSED_WRITE_TEXT } else { GENERATED_REPAIRED_WRITE_TEXT };
        return Ok(vec![
            json!({"type":"response.output_text.delta", "delta":text}),
            json!({"type":"response.completed", "response":{"id":format!("generated-write-{ordinal}"), "output":[]}}),
        ]);
    }
    let failure = if scenario == GeneratedHistoryScenario::RootDrift {
        json!({"code":"misalignment_policy_violation", "misalignment":{
            "detailed_explanation":GENERATED_PROVIDER_FAILURE,
            "steer":{"message":"preserve the original typed refusal"},
        }})
    } else {
        json!({"message":GENERATED_PROVIDER_FAILURE})
    };
    let mut events = Vec::new();
    if !matches!(scenario, GeneratedHistoryScenario::ZeroEffectFailure | GeneratedHistoryScenario::LaterFailure) {
        events.push(json!({"type":"response.output_text.delta", "delta":GENERATED_PARTIAL_TEXT}));
    }
    events.push(json!({"type":"response.failed", "response":{"error":failure}}));
    Ok(events)
}

fn write_generated_history_effect(root: &Path, ordinal: usize) -> Result<Value, ProviderSessionError> {
    let call_id = generated_history_call_id(ordinal);
    let path = root.join("generated-effects").join(format!("{ordinal}.txt"));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
    file.write_all(call_id.as_bytes()).and_then(|()| file.sync_all())
        .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
    Ok(json!({"receipt":"committed", "ordinal":ordinal.to_string()}))
}

struct GeneratedHistoryBridge {
    root: PathBuf,
    scenario: GeneratedHistoryScenario,
    calls: Arc<AtomicUsize>,
    relay: Option<Arc<CoordinatorToolRelay>>,
}

impl RoutedToolBridge for GeneratedHistoryBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(vec![RoutedToolDefinition {
            name: "b".into(), provider_identifier: "generated-history-contract".into(),
            tool_name: "b".into(), description: None, input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        let ordinal = self.calls.fetch_add(1, Ordering::AcqRel);
        assert!(ordinal < self.scenario.tool_count(), "a failed turn must never replay an effect");
        assert_eq!(tool.name, "b");
        assert_eq!(call_id, generated_history_call_id(ordinal));
        assert_eq!(args, json!({"ordinal":ordinal.to_string()}));
        match &self.relay {
            Some(relay) => relay.request_tool_execution(json!({
                "providerIdentifier":tool.provider_identifier, "toolName":tool.tool_name,
                "name":tool.name, "args":args, "toolCallId":call_id,
            })).map_err(coordinator_tool_execution_error),
            None => write_generated_history_effect(&self.root, ordinal),
        }
    }
}

fn inspect_generated_history_checkpoint(
    root: &Path,
    agent_id: &str,
    original: &OriginalTurnReference,
    scenario: GeneratedHistoryScenario,
    sessions: &Arc<ProductionSessionWorkers>,
) -> ExpectedCheckpoint {
    let store = sessions.open_agent_store_owner(agent_id).expect("actual generated canonical owner");
    let snapshot = store.checkpoint_snapshot_checked().expect("checked generated history root")
        .expect("present-empty baseline or actual committed generated checkpoint");
    assert_eq!(snapshot.root_blob_id, Sha256::digest(&snapshot.checkpoint_bytes).to_vec());
    let fields = decode_conversation_state_recovery_fields(&snapshot.checkpoint_bytes)
        .expect("actual generated canonical protobuf");
    let repaired_write = scenario == GeneratedHistoryScenario::FinalPersistenceRefusal;
    assert_eq!(fields.turns.len(), usize::from(scenario.retains_history()) + usize::from(repaired_write));
    let mut critical_blobs = vec![(snapshot.root_blob_id.clone(), snapshot.checkpoint_bytes.clone())];
    let mut journal = Vec::new();
    if scenario.retains_history() {
        let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
        let turn_id = &fields.turns[0];
        let turn_bytes = store.get_blob(turn_id).expect("actual generated turn blob read").expect("turn exists");
        assert_eq!(*turn_id, Sha256::digest(&turn_bytes).to_vec());
        let Some(ConversationTurnStructureFields::Agent { user_message, steps, request_id }) =
            decode_conversation_turn_structure_fields(&turn_bytes).expect("canonical Agent turn")
        else { panic!("generated history must retain an Agent turn"); };
        assert_eq!(request_id.as_deref(), Some(ORIGINAL_STREAM));
        let user_bytes = store.get_blob(&user_message).expect("actual user blob read").expect("user exists");
        assert_eq!(user_message, Sha256::digest(&user_bytes).to_vec());
        let user = decode_user_message_identity_fields(&user_bytes).expect("actual user identity");
        assert_eq!(user.message_id, original.user_message_id);
        assert_eq!(user.text, USER_TEXT);
        critical_blobs.push((turn_id.clone(), turn_bytes));
        critical_blobs.push((user_message, user_bytes));
        journal.push(json!({"role":"user", "message":{"content":[{"type":"text", "text":USER_TEXT}]}}));
        let prior_success = matches!(scenario,
            GeneratedHistoryScenario::LaterFailure | GeneratedHistoryScenario::FinalPersistenceRefusal);
        assert_eq!(steps.len(), scenario.tool_count() + usize::from(prior_success));
        for (ordinal, step_id) in steps.iter().enumerate() {
            let bytes = store.get_blob(step_id).expect("actual generated step read").expect("step exists");
            assert_eq!(*step_id, Sha256::digest(&bytes).to_vec());
            let expected_step = if ordinal < scenario.tool_count() {
                let input = json!({
                    "name":"b", "args":{"ordinal":ordinal.to_string()},
                    "toolCallId":generated_history_call_id(ordinal),
                    "providerIdentifier":"generated-history-contract", "toolName":"b",
                    "serverIdentifier":"generated-history-contract",
                });
                let result = json!({"success":{"isError":false, "structuredContent":{
                    "receipt":"committed", "ordinal":ordinal.to_string(),
                }}});
                journal.push(json!({"role":"assistant", "message":{"content":[{
                    "type":"tool_use", "name":"b", "input":input,
                }]}}));
                journal.push(json!({"role":"tool", "message":{"content":[{
                    "type":"tool_result", "name":"b", "result":result,
                }]}}));
                DecodedTranscriptStep::Tool { name: "b".into(), input, result: Some(result) }
            } else {
                assert!(prior_success, "a failed invocation cannot invent a final Assistant step");
                journal.push(json!({"role":"assistant", "message":{"content":[{
                    "type":"text", "text":GENERATED_PRIOR_TEXT,
                }]}}));
                DecodedTranscriptStep::Assistant { text: GENERATED_PRIOR_TEXT.into() }
            };
            assert_eq!(codec.decode_step(&bytes).expect("actual generated step projection"), expected_step);
            critical_blobs.push((step_id.clone(), bytes));
        }
        if repaired_write {
            let turn_id = &fields.turns[1];
            let turn_bytes = store.get_blob(turn_id).expect("repaired turn blob read").expect("repaired turn exists");
            assert_eq!(*turn_id, Sha256::digest(&turn_bytes).to_vec());
            let Some(ConversationTurnStructureFields::Agent { user_message, steps, request_id }) =
                decode_conversation_turn_structure_fields(&turn_bytes).expect("repaired canonical Agent turn")
            else { panic!("the repaired generated write must retain an Agent turn"); };
            assert_eq!(request_id.as_deref(), Some(ORIGINAL_STREAM));
            assert_eq!(steps.len(), 1, "repair cannot restore refused text or replay the earlier tool");
            let user_bytes = store.get_blob(&user_message).expect("repaired user blob read").expect("repaired user exists");
            assert_eq!(user_message, Sha256::digest(&user_bytes).to_vec());
            let user = decode_user_message_identity_fields(&user_bytes).expect("repaired user identity");
            let messages = vec![ProviderMessage { role: "user".into(), content: USER_TEXT.into() }];
            let nudge = shape_reply_nudge_turn_input(&messages, &messages, &TurnRunOptions::default());
            assert_eq!(user.message_id, ORIGINAL_STREAM, "the actual hidden nudge keeps its request identity");
            assert_eq!(user.text, nudge.options.recent_message_text.expect("actual shaped nudge text"));
            journal.push(json!({"role":"user", "message":{"content":[{"type":"text", "text":user.text}]}}));
            let step_bytes = store.get_blob(&steps[0]).expect("repaired Assistant step read").expect("repaired step exists");
            assert_eq!(steps[0], Sha256::digest(&step_bytes).to_vec());
            assert_eq!(codec.decode_step(&step_bytes).expect("repaired generated step projection"),
                DecodedTranscriptStep::Assistant { text: GENERATED_REPAIRED_WRITE_TEXT.into() });
            journal.push(json!({"role":"assistant", "message":{"content":[{
                "type":"text", "text":GENERATED_REPAIRED_WRITE_TEXT,
            }]}}));
            critical_blobs.push((turn_id.clone(), turn_bytes));
            critical_blobs.push((user_message, user_bytes));
            critical_blobs.push((steps[0].clone(), step_bytes));
        }
    } else if scenario == GeneratedHistoryScenario::RootDrift {
        assert_eq!(snapshot.checkpoint_bytes, PRIOR_STATE, "preserve the competing owner's exact state");
    } else {
        assert!(snapshot.checkpoint_bytes.is_empty(), "unknown or absent observations cannot create canonical history");
    }
    let journal_path = root.join("transcripts").join(agent_id).join(format!("{agent_id}.jsonl"));
    let actual_journal = match fs::read_to_string(&journal_path) {
        Ok(text) => text.lines().map(|line| serde_json::from_str::<Value>(line)
            .expect("actual Journal occurrence JSON")).collect::<Vec<_>>(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => panic!("read generated Journal {}: {error}", journal_path.display()),
    };
    assert_eq!(actual_journal, journal, "exact canonical user/tool/result order, with no fabricated final text");
    let db = sessions.open_agent_db_owner(agent_id).expect("actual accepted user owner");
    let user = db.get_transcript_entry_by_id(&original.user_message_id)
        .expect("accepted original user read").expect("accepted original user remains present");
    assert_eq!(user["content"], USER_TEXT);
    assert_eq!(user.get("confirmed").and_then(Value::as_bool), scenario.retains_history().then_some(true),
        "the existing accepted-user writer leaves confirmation absent until a canonical commit");
    assert!(reconcile_completed_original_turn(sessions, original)
        .expect("strict failed original-turn lookup").is_none());
    ExpectedCheckpoint {
        root_blob_id: snapshot.root_blob_id,
        state_bytes: Some(snapshot.checkpoint_bytes),
        last_user_message_id: if repaired_write { Some(ORIGINAL_STREAM.into()) }
            else { scenario.retains_history().then(|| original.user_message_id.clone()) },
        compaction_epoch: if scenario == GeneratedHistoryScenario::RootDrift { 3 } else { 0 },
        critical_blobs,
    }
}

struct OriginalEffectBridge {
    receipt: PathBuf,
    calls: Arc<AtomicUsize>,
    token: String,
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
        original_process_stage(&self.token, "effect-write-start");
        fs::OpenOptions::new().write(true).create_new(true).open(&self.receipt)
            .and_then(|mut file| file.write_all(call_id.as_bytes()))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        original_process_stage(&self.token, "effect-committed");
        Ok(json!({"receipt":"committed"}))
    }
}

fn produce_generated_failure(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) -> ProducedOriginal {
    let scenario = request.generated_scenario.expect("typed generated failure scenario");
    let prior_success = matches!(scenario,
        GeneratedHistoryScenario::LaterFailure | GeneratedHistoryScenario::FinalPersistenceRefusal);
    let root = &request.fixture_root;
    original_process_stage(&request.token, &format!("generated-{scenario:?}-admission"));
    let runtime = ProductionTranscriptRuntime::new(Some(root));
    let args = json!({
        "agentId":request.agent_id, "streamId":ORIGINAL_STREAM, "prompt":USER_TEXT,
        "sessionKey":format!("agent:{}:main", request.agent_id), "clientNonce":"generated-history-user-nonce",
    });
    let accepted = runtime.accept_routed_send(&args, |accepted| {
        persist_accepted_send_prompt_context(sessions, &args, accepted)
            .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
    }).expect("actual generated routed admission");
    assert!(!accepted.duplicate);
    let control = runtime.capture_routed_turn_start(&request.agent_id, ORIGINAL_STREAM)
        .expect("capture the actual accepted lease cancellation and retirement identity");
    let cancellation = control.cancellation();
    let admission = sessions.read_session_work_admission(&request.agent_id)
        .expect("generated Session admission").expect("main admission");
    let original = runtime.bind_original_turn_model(&request.agent_id, ORIGINAL_STREAM, &admission, ORIGINAL_MODEL)
        .expect("bind the admitted generated model").expect("ordinary Native original user");
    assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
    let host = composition(root, sessions);
    let sink = host.compose_production_checkpoint_sink(
        sessions, root, &request.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("real generated checkpoint composition").expect("canonical main sink");
    assert!(sink.committed_original_turn_observations().expect("initial Journal observations").is_empty());
    fs::create_dir(root.join("generated-effects")).expect("private counted effect directory");
    let events = GatewayEventHub::default();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(events.clone(), Duration::from_secs(3)));
    let downstream_calls = Arc::new(AtomicUsize::new(0));
    let downstream_count = Arc::clone(&downstream_calls);
    let downstream_root = root.clone();
    let weak_relay = Arc::downgrade(&relay);
    let subscription = events.subscribe_listener(move |event| {
        if event["channel"] != RUNNER_TOOL_REQUEST_EVENT_CHANNEL { return; }
        assert_eq!(event["payload"]["method"], ROUTED_TOOL_EXECUTE_METHOD);
        assert_eq!(event["payload"]["args"]["toolName"], "b");
        assert_eq!(event["payload"]["args"]["toolCallId"], generated_history_call_id(0));
        assert_eq!(downstream_count.fetch_add(1, Ordering::AcqRel), 0);
        write_generated_history_effect(&downstream_root, 0).expect("actual published downstream file effect");
        weak_relay.upgrade().expect("live downstream relay").resolve(&json!({
            "requestId":event["payload"]["requestId"], "ok":false,
            "error":"downstream control returned without a definitive execution result",
        })).expect("actual relay remote failure after publication");
    });
    let calls = Arc::new(AtomicUsize::new(0));
    let retries = Arc::new(AtomicUsize::new(0));
    let retry_count = Arc::clone(&retries);
    let reports = Arc::new(Mutex::new(Vec::new()));
    let retry_reports = Arc::clone(&reports);
    let provider_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, &request.agent_id, ORIGINAL_STREAM));
    if scenario == GeneratedHistoryScenario::ProviderStoreRefusal {
        fs::create_dir_all(provider_store.directory().parent().expect("provider directory parent"))
            .expect("prepare the actual store's parent");
        fs::write(provider_store.directory(), GENERATED_STORE_OBSTRUCTION).expect("occupy the real provider directory");
    }
    let turn_composition = TurnAgentComposition::new(
        RoutedProvider::Fabushi,
        Arc::new(GeneratedHistoryBridge {
            root: root.clone(), scenario, calls: Arc::clone(&calls),
            relay: (scenario == GeneratedHistoryScenario::UnknownDownstream).then(|| Arc::clone(&relay)),
        }),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "generated-history-contract".into(), shell: None, time_zone: Some("UTC".into()),
                transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(), user_full_name: None,
            }, rules: None,
        },
        cancellation.clone(), provider_store.clone(),
    ).with_fabushi_model(ORIGINAL_MODEL)
        .with_retry_sink(Arc::new(move |_| { retry_count.fetch_add(1, Ordering::AcqRel); }))
        .with_retry_report_sink(Arc::new(move |report| { retry_reports.lock().expect("retry report capture").push(report.clone()); }));
    let successes = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::clone(&successes);
    let mut runner = create_production_runner(
        turn_composition,
        ProductionTurnAgentBuildBindings::new(
            ProductionTurnAgentStaticConfig {
                model_id: ORIGINAL_MODEL.into(), agent_token_limit: SAND_AGENT_TOKEN_LIMIT,
                conversation_id: request.agent_id.clone(), is_box_scoped_subagent: false,
                is_subagent_runner: false, is_shared_room_runner: false,
                sand_send_message_delivery_owed: false, transcripts_folder_available: true,
            },
            Arc::new(|| None), Arc::new(|_, _, _| panic!("bounded generated history must not summarize")),
        ),
        ProductionTurnAgentLifecycleBindings::new(&request.agent_id, ORIGINAL_STREAM)
            .with_profile_announcement_commit(Some(Arc::new(move || { success_count.fetch_add(1, Ordering::AcqRel); }))),
        Some(sink.clone()), Arc::new(AtomicBool::new(false)),
        Arc::new(Mutex::new(SubagentRuntime::default())), None, None, None, None,
    );
    let messages = vec![ProviderMessage { role: "user".into(), content: USER_TEXT.into() }];
    let options = TurnRunOptions {
        inference_request_id: Some(ORIGINAL_STREAM.into()), message_id: Some(original.user_message_id.clone()),
        recent_message_text: Some(USER_TEXT.into()), turn_started_at_ms: Some(1), ..TurnRunOptions::default()
    };
    let mut custody = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
    let (failed_messages, failed_options) = if prior_success {
        assert_eq!(runner.run_routed_provider_with_options(root, &messages, options.clone(), &mut |_, _| {})
            .expect("actual earlier generated invocation succeeds"), GENERATED_PRIOR_TEXT);
        assert_eq!(runner.last_finished().expect("earlier shell terminal").outcome, TerminalOutcome::Completed);
        custody.record(runner.last_tool_effect_custody());
        let observations = sink.committed_original_turn_observations().expect("earlier successful Journal authority");
        assert_eq!(observations.len(), 1);
        prove_completed_original_turn(&original, project_turn_terminal(false, false, true, 0, false),
            original_turn_result_digest(GENERATED_PRIOR_TEXT), 0, false, &custody, &observations)
            .expect("the earlier actual success is eligible before the failed invocation");
        let shaped = shape_reply_nudge_turn_input(&messages, &messages, &options);
        (shaped.provider_messages, shaped.options)
    } else { (messages.clone(), options.clone()) };
    let before = durable_state_for(root, &request.agent_id);
    let provider_before = (scenario == GeneratedHistoryScenario::FinalPersistenceRefusal)
        .then(|| mirror_files(&root.join("runner-provider-checkpoints")));
    let store = sessions.open_agent_store_owner(&request.agent_id).expect("actual competing-root owner");
    let blob_path = root.join("agents").join(&request.agent_id).join(CONVERSATION_BLOBS_FILENAME);
    let mut competing_state = None;
    let mut final_write_refusal_injected = false;
    let expected_delta = if scenario == GeneratedHistoryScenario::FinalPersistenceRefusal {
        GENERATED_REFUSED_WRITE_TEXT
    } else { GENERATED_PARTIAL_TEXT };
    let mut deltas = Vec::new();
    let error = runner.run_routed_provider_with_options(root, &failed_messages, failed_options, &mut |delta, _| {
        deltas.push(delta.to_string());
        assert_eq!(delta, expected_delta);
        assert_eq!(calls.load(Ordering::Acquire), scenario.tool_count(), "cancellation/drift follows actual returned effects");
        if scenario == GeneratedHistoryScenario::Cancellation {
            assert!(runtime.cancel_routed_turn_lease(ORIGINAL_STREAM, Some(&request.agent_id), GENERATED_CANCEL_REASON));
        } else if scenario == GeneratedHistoryScenario::RootDrift {
            assert!(competing_state.is_none());
            store.handle_checkpoint_bytes_blocking(PRIOR_STATE).expect("real competing canonical commit");
            competing_state = Some(durable_state_for(root, &request.agent_id));
        } else if scenario == GeneratedHistoryScenario::FinalPersistenceRefusal {
            assert!(!final_write_refusal_injected);
            // The real stream can finish and stage from readable blobs. Its
            // first final WorkerBlobStore INSERT then refuses before any
            // canonical, confirmation or Journal mutation.
            open_existing(&blob_path, false).execute_batch(&format!(
                "CREATE TRIGGER generated_history_final_write_refusal BEFORE INSERT ON blobs \
                 BEGIN SELECT RAISE(ABORT, '{GENERATED_FINAL_WRITE_REFUSAL}'); END;",
            )).expect("install the uniquely owned final-write fault");
            final_write_refusal_injected = true;
        }
    }).expect_err("the actual generated invocation failure must remain a failure");
    original_process_stage(&request.token, &format!("generated-{scenario:?}-returned"));
    let has_partial_output = matches!(scenario, GeneratedHistoryScenario::ProviderFailure
        | GeneratedHistoryScenario::OrderedProviderFailure | GeneratedHistoryScenario::Cancellation
        | GeneratedHistoryScenario::RootDrift | GeneratedHistoryScenario::FinalPersistenceRefusal);
    assert_eq!(deltas, if has_partial_output { vec![expected_delta.to_string()] } else { Vec::new() });
    assert_eq!(calls.load(Ordering::Acquire), scenario.tool_count());
    assert_eq!(retries.load(Ordering::Acquire), 0, "history persistence grants neither retry nor replay");
    assert_eq!(successes.load(Ordering::Acquire), usize::from(prior_success),
        "failed history must not invoke successful lifecycle hooks");
    let finished = runner.last_finished().expect("actual failed generated shell terminal");
    assert_eq!(finished.owner.request_id, ORIGINAL_STREAM);
    assert!(finished.owner.generation > 0);
    let failed_generation = finished.owner.generation;
    // RoutedProviderCancellation::check currently preserves its reason as
    // Transport. Pin that real shell outcome separately from Host's cancelled
    // terminal projection; do not invent a Cancelled provider-error variant.
    assert_eq!(finished.outcome, TerminalOutcome::Failed {
        retryable: matches!(&error, ProviderSessionError::Transport(_)), message: error.to_string(),
    });
    assert_eq!(cancellation.is_cancelled(), scenario == GeneratedHistoryScenario::Cancellation);
    let terminal = project_turn_terminal(false, cancellation.is_cancelled(), false, 0, false);
    assert_eq!(terminal.kind, if scenario == GeneratedHistoryScenario::Cancellation {
        TurnTerminalKind::Cancelled
    } else { TurnTerminalKind::Failed });
    match scenario {
        GeneratedHistoryScenario::Cancellation => {
            assert_eq!(cancellation.reason().as_deref(), Some(GENERATED_CANCEL_REASON));
            assert!(matches!(&error, ProviderSessionError::Transport(reason) if reason == GENERATED_CANCEL_REASON));
        }
        GeneratedHistoryScenario::RootDrift => {
            let ProviderSessionError::ProviderRefusal(refusal) = &error else {
                panic!("history staging cannot replace the original typed provider refusal: {error:?}");
            };
            assert_eq!(refusal.category, "misalignment");
            assert_eq!(refusal.explanation.as_deref(), Some(GENERATED_PROVIDER_FAILURE));
            assert_eq!(durable_state_for(root, &request.agent_id), competing_state.expect("actual competing commit was observed"));
        }
        GeneratedHistoryScenario::ProviderStoreRefusal => {
            assert!(matches!(&error, ProviderSessionError::Tool(message)
                if message.contains("could not prepare Runner checkpoint directory")));
            assert_eq!(fs::read(provider_store.directory()).expect("original refusing entry"), GENERATED_STORE_OBSTRUCTION);
        }
        GeneratedHistoryScenario::UnknownDownstream => {
            assert!(matches!(&error, ProviderSessionError::Tool(_)));
            assert_eq!(downstream_calls.load(Ordering::Acquire), 1);
            assert_eq!(relay.pending_count(), 0);
        }
        GeneratedHistoryScenario::FinalPersistenceRefusal => {
            assert!(final_write_refusal_injected);
            let expected = format!(
                "generated Agent stream persistence failed: Runner Agent checkpoint blob write failed: \
                 conversation blob sqlite operation failed: {GENERATED_FINAL_WRITE_REFUSAL}",
            );
            assert!(matches!(&error, ProviderSessionError::Protocol(message) if message == &expected),
                "the actual final-persistence error must retain its complete provenance: {error:?}");
        }
        _ => assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains(GENERATED_PROVIDER_FAILURE))),
    }
    let (started_revision, checkpoint_revision) = match scenario {
        GeneratedHistoryScenario::ZeroEffectFailure | GeneratedHistoryScenario::LaterFailure
            | GeneratedHistoryScenario::FinalPersistenceRefusal => (0, 0),
        GeneratedHistoryScenario::ProviderStoreRefusal | GeneratedHistoryScenario::UnknownDownstream => (1, 0),
        _ => (scenario.tool_count() as u64, scenario.tool_count() as u64),
    };
    assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::Known {
        run_generation: finished.owner.generation,
        snapshot: ToolEffectCustodySnapshot {
            started_revision, checkpoint_revision, pending_effects: 0,
            unknown_result: (scenario == GeneratedHistoryScenario::UnknownDownstream)
                .then_some(ToolEffectUnknownResult::DownstreamOutcomeUnknown),
        },
    }), "canonical history persistence must not advance accepted provider-checkpoint custody");
    custody.record(runner.last_tool_effect_custody());
    assert!(sink.committed_original_turn_observations().is_err(), "any failure permanently invalidates this sink's completion authority");
    assert!(prove_completed_original_turn(&original, terminal, original_turn_result_digest(""), 0, false, &custody, &[]).is_err());
    if matches!(scenario, GeneratedHistoryScenario::UnknownDownstream | GeneratedHistoryScenario::ZeroEffectFailure
        | GeneratedHistoryScenario::LaterFailure | GeneratedHistoryScenario::FinalPersistenceRefusal)
    {
        assert_eq!(durable_state_for(root, &request.agent_id), before, "ineligible or empty history leaves every canonical and Journal byte unchanged");
    }
    if matches!(scenario, GeneratedHistoryScenario::UnknownDownstream | GeneratedHistoryScenario::ZeroEffectFailure) {
        assert!(!provider_store.directory().exists(), "no accepted provider checkpoint may be fabricated");
    }
    if matches!(scenario, GeneratedHistoryScenario::ProviderStoreRefusal | GeneratedHistoryScenario::UnknownDownstream) {
        let reports = reports.lock().expect("actual retry reports");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
        assert_eq!(reports[0].error_code, "SAND-E0407");
        assert!(!reports[0].resume_from_checkpoint);
    }
    if scenario == GeneratedHistoryScenario::FinalPersistenceRefusal {
        let prior_checkpoint = inspect_generated_history_checkpoint(
            root, &request.agent_id, &original, GeneratedHistoryScenario::LaterFailure, sessions,
        );
        assert_eq!(sink.base_state_bytes().expect("failed write retains the readable sink base"),
            prior_checkpoint.state_bytes.clone().unwrap());
        let provider_before = provider_before.expect("actual prior successful provider checkpoint");
        assert_eq!(mirror_files(&root.join("runner-provider-checkpoints")), provider_before);
        let fault = open_existing(&blob_path, false);
        let trigger_count: i64 = fault.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name='generated_history_final_write_refusal'",
            [], |row| row.get(0),
        ).expect("the actual refusing trigger remains installed");
        assert_eq!(trigger_count, 1);
        fault.execute_batch("DROP TRIGGER generated_history_final_write_refusal;")
            .expect("explicitly repair only the owned final-write obstacle");
        drop(fault);
        assert_eq!(durable_state_for(root, &request.agent_id), before,
            "removing the fault cannot mutate canonical rows or Journal history");
        assert!(sink.committed_original_turn_observations().is_err(),
            "repair alone cannot restore earlier completion observations");

        let repaired_input = shape_reply_nudge_turn_input(&messages, &messages, &options);
        let mut repaired_deltas = Vec::new();
        let repaired_result = runner.run_routed_provider_with_options(
            root, &repaired_input.provider_messages, repaired_input.options, &mut |delta, _| {
                assert_eq!(delta, GENERATED_REPAIRED_WRITE_TEXT);
                assert_eq!(calls.load(Ordering::Acquire), 1);
                repaired_deltas.push(delta.to_string());
            },
        ).expect("an explicitly requested later generated invocation can commit after storage repair");
        assert_eq!(repaired_result, GENERATED_REPAIRED_WRITE_TEXT);
        assert_eq!(repaired_deltas, vec![GENERATED_REPAIRED_WRITE_TEXT.to_string()]);
        let repaired_finished = runner.last_finished().expect("actual repaired invocation shell terminal");
        assert_eq!(repaired_finished.owner.request_id, ORIGINAL_STREAM);
        assert!(repaired_finished.owner.generation > failed_generation);
        assert_eq!(repaired_finished.outcome, TerminalOutcome::Completed);
        assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::Known {
            run_generation: repaired_finished.owner.generation,
            snapshot: ToolEffectCustodySnapshot {
                started_revision: 0, checkpoint_revision: 0, pending_effects: 0, unknown_result: None,
            },
        }), "the later write has no new or replayed tool effects");
        custody.record(runner.last_tool_effect_custody());
        assert_eq!(calls.load(Ordering::Acquire), 1);
        assert_eq!(retries.load(Ordering::Acquire), 0, "the explicit later invocation is not an automatic retry");
        assert!(reports.lock().expect("actual final-write retry reports").is_empty());
        assert_eq!(successes.load(Ordering::Acquire), 1, "the already committed profile hook stays idempotent");
        assert_eq!(mirror_files(&root.join("runner-provider-checkpoints")), provider_before);
        assert_ne!(store.checkpoint_snapshot_checked().expect("actual repaired canonical root")
            .expect("the repaired write committed").root_blob_id, prior_checkpoint.root_blob_id);
        assert!(sink.committed_original_turn_observations().is_err(),
            "a successful later commit through this same sink cannot revive failed original-turn authority");
        assert!(prove_completed_original_turn(&original, project_turn_terminal(false, false, true, 0, false),
            original_turn_result_digest(&repaired_result), 0, false, &custody, &[]).is_err());
    }
    sessions.prepare_existing_agent(&request.agent_id).expect("finish existing outline maintenance before closure")
        .expect("actual generated Session remains present");
    let checkpoint = inspect_generated_history_checkpoint(root, &request.agent_id, &original, scenario, sessions);
    assert!(runtime.quiesce_for_upgrade().expect("actual failed original upgrade marker").quiescing);
    let markers = runtime.upgrade_resume_store().expect("actual generated marker owner")
        .list_pending_checked().expect("strict failed original marker image");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].original_work.as_ref(), Some(&original));
    assert!(runtime.settle_routed_turn_control(&control, 10).expect("retire only the captured failed lease").is_some());
    drop((runner, sink, host, store, subscription, relay, runtime));
    ProducedOriginal { original, checkpoint }
}

fn produce_original_turn(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) -> ProducedOriginal {
    original_process_stage(&request.token, "admission-start");
    let root = &request.fixture_root;
    let runtime = ProductionTranscriptRuntime::new(Some(root));
    let args = json!({
        "agentId":request.agent_id, "streamId":ORIGINAL_STREAM, "prompt":USER_TEXT,
        "sessionKey":format!("agent:{}:main", request.agent_id), "clientNonce":"original-user-nonce",
    });
    let accepted = runtime.accept_routed_send(&args, |accepted| {
        persist_accepted_send_prompt_context(sessions, &args, accepted)
            .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
    }).expect("actual routed admission and gateway persistence");
    assert!(!accepted.duplicate);
    let admission = sessions.read_session_work_admission(&request.agent_id)
        .expect("actual Session admission").expect("main admission");
    let original = runtime.bind_original_turn_model(&request.agent_id, ORIGINAL_STREAM, &admission, ORIGINAL_MODEL)
        .expect("same admitted lease model binding").expect("ordinary Native user is eligible");
    assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
    original_process_stage(&request.token, "admission-and-model-bound");
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
    original_process_stage(&request.token, "checkpoint-sink-composed");
    let calls = Arc::new(AtomicUsize::new(0));
    let turn_composition = TurnAgentComposition::new(
        RoutedProvider::Fabushi,
        Arc::new(OriginalEffectBridge {
            receipt: root.join("original-effect.txt"), calls: Arc::clone(&calls),
            token: request.token.clone(),
        }),
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
    original_process_stage(&request.token, "runner-composed");
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
        original_process_stage(&request.token, &format!("invocation-{invocation}-start"));
        let mut first_delta = true;
        let result = runner.run_routed_provider_with_options(
            root, &input_messages, input_options, &mut |_, _| {
                if first_delta {
                    original_process_stage(&request.token, &format!("invocation-{invocation}-first-delta"));
                    first_delta = false;
                }
            },
        );
        original_process_stage(&request.token, &format!("invocation-{invocation}-returned-{}",
            if result.is_ok() { "ok" } else { "error" }));
        let result = result.expect("actual generated Native invocation completed");
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
    original_process_stage(&request.token, "invocations-and-custody-complete");
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
    original_process_stage(&request.token, "proof-qualified");
    assert_eq!(persist_completed_original_turn(sessions, &proof).expect("strict Session terminal commit"), OriginalTurnReceiptCommit::Inserted);
    original_process_stage(&request.token, "terminal-inserted");
    assert_eq!(persist_completed_original_turn(sessions, &proof).expect("same proof is idempotent"), OriginalTurnReceiptCommit::AlreadyCommitted);
    assert_eq!(reconcile_completed_original_turn(sessions, &original).expect("same-process strict recheck").expect("verified completion").original(), &original);
    assert!(runtime.quiesce_for_upgrade().expect("publish the actual bound lease upgrade marker").quiescing);
    let markers = runtime.upgrade_resume_store().expect("actual upgrade marker owner")
        .list_pending_checked().expect("strict producer marker image");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].agent_id, original.agent_id);
    assert_eq!(markers[0].source.as_deref(), Some("turn"));
    assert_eq!(markers[0].original_work.as_ref(), Some(&original));
    original_process_stage(&request.token, "original-marker-written");
    runtime.settle_routed_turn(&request.agent_id, ORIGINAL_STREAM, 10).expect("retire actual routed lease");
    let store = sessions.open_agent_store_owner(&request.agent_id).expect("actual final canonical owner");
    let snapshot = store.checkpoint_snapshot_checked().expect("checked final root").expect("committed final state");
    let checkpoint = ExpectedCheckpoint {
        root_blob_id: snapshot.root_blob_id.clone(), state_bytes: Some(snapshot.checkpoint_bytes.clone()),
        last_user_message_id: Some(original.user_message_id.clone()), compaction_epoch: request.expected.compaction_epoch,
        critical_blobs: vec![(snapshot.root_blob_id, snapshot.checkpoint_bytes)],
    };
    drop((runner, sink, host, store, runtime));
    original_process_stage(&request.token, "producer-return");
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

fn assert_generated_history_reopen(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) {
    let original = request.original.as_ref().expect("actual failed producer original identity");
    let scenario = request.generated_scenario.expect("typed failed producer scenario");
    let prepared = sessions.prepare_existing_agent(&request.agent_id)
        .expect("fresh consumer accepts intact generated history").expect("existing Session");
    assert_eq!(prepared.persisted_root_blob_id, request.expected.root_blob_id);
    let checkpoint = inspect_generated_history_checkpoint(
        &request.fixture_root, &request.agent_id, original, scenario, sessions,
    );
    assert_eq!(checkpoint, request.expected, "fresh process reads the exact root/user/tool/result blobs");
    let host = composition(&request.fixture_root, sessions);
    let watermark = host.confirmed_user_turn_watermark(sessions, &request.agent_id)
        .expect("fresh failed-history watermark");
    assert_eq!(watermark.has_user_turn, scenario.retains_history());
    assert_eq!(watermark.last_user_message_id, request.expected.last_user_message_id);
    assert_eq!(host.prompt_compaction_epoch(sessions, &request.agent_id)
        .expect("fresh failed-history epoch"), request.expected.compaction_epoch);
    let sink = host.compose_production_checkpoint_sink(
        sessions, &request.fixture_root, &request.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("fresh actual history consumer").expect("main checkpoint sink");
    assert_eq!(sink.base_state_bytes().expect("fresh sink exact history"), request.expected.state_bytes.clone().unwrap());
    assert!(sink.committed_original_turn_observations().expect("fresh empty completion log").is_empty(),
        "reading persisted failed history must not manufacture completion observations");
    let markers = SandUpgradeResumeStore::new(&request.fixture_root).list_pending_checked()
        .expect("fresh strict original marker read");
    assert_eq!(markers, vec![request.original_marker.clone().expect("actual failed producer marker")]);
    assert_eq!(markers[0].original_work.as_ref(), Some(original));
    assert!(!request.fixture_root.join("original-turn-credential.json").exists());
    let effects = mirror_files(&request.fixture_root.join("generated-effects"));
    assert_eq!(effects.len(), scenario.tool_count());
    for ordinal in 0..scenario.tool_count() {
        assert_eq!(effects.get(&PathBuf::from(format!("{ordinal}.txt"))).map(Vec::as_slice),
            Some(generated_history_call_id(ordinal).as_bytes()));
    }
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
    original_process_stage(&request.token, "owners-opening");
    let owners = OpenOwners::new(&request.fixture_root);
    let mut canonical_turns = None;
    let produced = match request.expectation {
        ReopenExpectation::Ready => { assert_ready(&request, &owners.sessions); None }
        ReopenExpectation::Refuse => { assert_refused(&request, &owners.sessions); None }
        ReopenExpectation::ProduceOriginal => Some(produce_original_turn(&request, &owners.sessions)),
        ReopenExpectation::ProduceGeneratedFailure => Some(produce_generated_failure(&request, &owners.sessions)),
        ReopenExpectation::GeneratedHistoryReady => {
            assert_generated_history_reopen(&request, &owners.sessions); None
        }
        ReopenExpectation::ProduceCanonicalRoot => {
            let (produced, turns) = produce_canonical_root(&request, &owners.sessions);
            canonical_turns = Some(turns);
            Some(produced)
        }
        ReopenExpectation::CanonicalRootReady => {
            assert_canonical_root_reopen(&request, &owners.sessions); None
        }
        ReopenExpectation::OriginalReady | ReopenExpectation::OriginalAbsent | ReopenExpectation::OriginalRefuse => {
            assert_original_reopen(&request, &owners.sessions); None
        }
        ReopenExpectation::OriginalMarkerAck | ReopenExpectation::OriginalMarkerNoop
            | ReopenExpectation::OriginalMarkerPendingAck | ReopenExpectation::OriginalMarkerCorruptAck => {
            panic!("marker acknowledgment must use the bound shipping App executable")
        }
    };
    original_process_stage(&request.token, "owners-closing");
    owners.close();
    original_process_stage(&request.token, "owners-closed");
    let receipt = ChildReceipt {
        token: request.token,
        pid: std::process::id(),
        parent_pid: request.parent_pid,
        expectation: request.expectation,
        produced,
        canonical_turns,
    };
    fs::write(request_path.parent().expect("private run directory").join("completed.json"),
        serde_json::to_vec(&receipt).expect("serialize completion receipt"))
        .expect("publish completion only after assertions and worker shutdown");
    original_process_stage(&receipt.token, "completion-receipt-written");
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
    let child = fixture.run_child("produce-original", ReopenExpectation::ProduceOriginal, Some(&server));
    // Capture the server result even when the producer fails, before any panic
    // can transfer cleanup to Drop and hide the request/response failure phase.
    let server_result = server.finish();
    eprintln!("original Responses producer_pid={} outcome={server_result:?} {}",
        child.child_pid, server.diagnostics());
    assert!(server_result.is_ok() && child.result.as_ref().is_ok_and(|status| status.success()),
        "original turn producer failed: child={:?}; Responses worker={server_result:?}\n{}",
        child.result, child.logs);
    let receipt = child.assert_completed();
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

fn produced_generated_failure_fixture(label: &str, scenario: GeneratedHistoryScenario) -> Fixture {
    let mut fixture = Fixture::new(label, InitialState::EmptyCheckpoint);
    fixture.generated_scenario = Some(scenario);
    let mut server = OriginalTurnWireServer::generated(&fixture.root, scenario);
    let child = fixture.run_child("produce-generated-failure", ReopenExpectation::ProduceGeneratedFailure, Some(&server));
    let server_result = server.finish();
    eprintln!("generated history scenario={scenario:?} producer_pid={} outcome={server_result:?} {}",
        child.child_pid, server.diagnostics());
    assert!(server_result.is_ok() && child.result.as_ref().is_ok_and(|status| status.success()),
        "generated history producer failed: child={:?}; Responses worker={server_result:?}\n{}", child.result, child.logs);
    if scenario == GeneratedHistoryScenario::RootDrift {
        assert!(child.logs.contains("interrupted_generated_history_not_committed phase=observation-or-staging"),
            "the original typed refusal must retain a separate fixed history-failure diagnostic");
    }
    let receipt = child.assert_completed();
    let requests = server.requests.lock().expect("actual generated provider capture");
    assert_eq!(requests.len(), scenario.request_count(), "no new provider attempt or effect replay");
    assert!(requests.iter().all(|request| request["model"] == ORIGINAL_MODEL));
    if scenario.request_count() > 1 {
        let input = requests[1]["input"].as_array().expect("actual post-tool continuation input");
        let tool_items = input.iter().filter(|item| matches!(item["type"].as_str(),
            Some("function_call" | "function_call_output"))).collect::<Vec<_>>();
        assert_eq!(tool_items.len(), scenario.tool_count() * 2);
        for ordinal in 0..scenario.tool_count() {
            assert_eq!(tool_items[ordinal * 2]["type"], "function_call");
            assert_eq!(tool_items[ordinal * 2]["call_id"], generated_history_call_id(ordinal));
            assert_eq!(tool_items[ordinal * 2 + 1]["type"], "function_call_output");
            assert_eq!(tool_items[ordinal * 2 + 1]["call_id"], generated_history_call_id(ordinal));
            let result: Value = serde_json::from_str(tool_items[ordinal * 2 + 1]["output"].as_str()
                .expect("actual serialized tool result")).expect("actual returned tool result JSON");
            assert_eq!(result, json!({"receipt":"committed", "ordinal":ordinal.to_string()}));
        }
    }
    drop(requests);
    fs::remove_file(&server.credential).expect("remove actual producer credential before all fresh consumers");
    let produced = receipt.produced.expect("failed producer's actual reference and persisted-state observation");
    fixture.original = Some(produced.original);
    fixture.expected = produced.checkpoint;
    let markers = SandUpgradeResumeStore::new(&fixture.root).list_pending_checked()
        .expect("actual failed producer's durable marker");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].agent_id, fixture.agent_id);
    assert_eq!(markers[0].source.as_deref(), Some("turn"));
    assert_eq!(markers[0].original_work, fixture.original);
    fixture.original_marker = Some(markers[0].clone());
    let terminal_count: i64 = open_existing(&fixture.db_path(), true).query_row(
        "SELECT COUNT(*) FROM kv WHERE key=?1", params![ORIGINAL_TERMINAL_KEY], |row| row.get(0),
    ).expect("read actual completion-authority absence");
    assert_eq!(terminal_count, 0, "neither failed history nor an earlier invocation can seal the original turn");
    let provider_files = mirror_files(&fixture.root.join("runner-provider-checkpoints"));
    if matches!(scenario, GeneratedHistoryScenario::UnknownDownstream | GeneratedHistoryScenario::ZeroEffectFailure) {
        assert!(provider_files.is_empty());
    } else if scenario == GeneratedHistoryScenario::ProviderStoreRefusal {
        assert_eq!(provider_files.len(), 1);
        assert_eq!(provider_files.values().next().unwrap(), GENERATED_STORE_OBSTRUCTION);
    } else {
        assert_eq!(provider_files.len(), 1, "one real provider boundary covers the returned tool batch");
        let (path, bytes) = provider_files.iter().next().unwrap();
        assert_eq!(path.extension().and_then(|value| value.to_str()), Some("json"));
        let checkpoint: RoutedProviderCheckpoint = serde_json::from_slice(bytes).expect("actual durable provider checkpoint JSON");
        let RoutedProviderCheckpoint::Fabushi(checkpoint) = checkpoint else {
            panic!("the actual generated Native boundary must retain its provider identity");
        };
        assert_eq!(checkpoint.model.as_deref(), Some(ORIGINAL_MODEL));
        assert_eq!(checkpoint.tool_calls_completed, scenario.tool_count());
    }
    fixture
}

fn assert_generated_failure_reopens(label: &str, scenario: GeneratedHistoryScenario) {
    let fixture = produced_generated_failure_fixture(label, scenario);
    let before = durable_state(&fixture);
    let provider_before = mirror_files(&fixture.root.join("runner-provider-checkpoints"));
    let effect_before = mirror_files(&fixture.root.join("generated-effects"));
    assert_eq!(effect_before.len(), scenario.tool_count());
    let marker_path = SandUpgradeResumeStore::new(&fixture.root).file_path().to_path_buf();
    let marker_before = fs::read(&marker_path).expect("actual failed original marker bytes");
    for attempt in 1..=2 {
        let receipt = fixture.reopen(&format!("generated-fresh-consumer-{attempt}"), ReopenExpectation::GeneratedHistoryReady)
            .assert_completed();
        assert!(receipt.produced.is_none(), "a fresh consumer may not produce another invocation");
        assert_eq!(durable_state(&fixture), before, "fresh read cannot append, normalize or duplicate generated history");
        assert_eq!(mirror_files(&fixture.root.join("runner-provider-checkpoints")), provider_before,
            "fresh read cannot dispatch or accept another provider checkpoint");
        assert_eq!(mirror_files(&fixture.root.join("generated-effects")), effect_before, "fresh read cannot replay a tool");
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before, "failed original marker must remain byte-exact and pending");
    }
}

#[test]
fn generated_failure_history_provider_failure_survives_repeated_fresh_consumers() {
    for scenario in [GeneratedHistoryScenario::ProviderFailure, GeneratedHistoryScenario::OrderedProviderFailure] {
        assert_generated_failure_reopens("generated-provider-failure", scenario);
    }
}

#[test]
fn generated_failure_history_cancellation_survives_repeated_fresh_consumers() {
    assert_generated_failure_reopens("generated-cancellation", GeneratedHistoryScenario::Cancellation);
}

#[test]
fn generated_failure_history_provider_store_refusal_preserves_unaccepted_effect() {
    assert_generated_failure_reopens("generated-provider-store-refusal", GeneratedHistoryScenario::ProviderStoreRefusal);
}

#[test]
fn generated_failure_history_unknown_downstream_outcome_preserves_canonical_bytes() {
    assert_generated_failure_reopens("generated-downstream-unknown", GeneratedHistoryScenario::UnknownDownstream);
}

#[test]
fn generated_failure_history_zero_effect_refusal_preserves_canonical_bytes() {
    assert_generated_failure_reopens("generated-zero-effect", GeneratedHistoryScenario::ZeroEffectFailure);
}

#[test]
fn generated_failure_history_root_drift_preserves_competing_checkpoint() {
    assert_generated_failure_reopens("generated-root-drift", GeneratedHistoryScenario::RootDrift);
}

#[test]
fn generated_failure_history_later_failure_invalidates_prior_completion() {
    assert_generated_failure_reopens("generated-later-failure", GeneratedHistoryScenario::LaterFailure);
}

#[test]
fn generated_failure_history_final_persistence_refusal_stays_ineligible_after_repair() {
    assert_generated_failure_reopens("generated-final-persistence-refusal", GeneratedHistoryScenario::FinalPersistenceRefusal);
}

#[test]
fn generated_failure_history_successful_control_preserves_original_completion() {
    let fixture = produced_original_fixture("generated-success-control", InitialState::EmptyCheckpoint, false, 1, false, false);
    let before = durable_state(&fixture);
    let provider_before = mirror_files(&fixture.root.join("runner-provider-checkpoints"));
    let effect_before = fs::read(fixture.root.join("original-effect.txt")).expect("successful actual effect");
    let marker_path = SandUpgradeResumeStore::new(&fixture.root).file_path().to_path_buf();
    let marker_before = fs::read(&marker_path).expect("successful original marker");
    for attempt in 1..=2 {
        fixture.reopen(&format!("generated-success-fresh-{attempt}"), ReopenExpectation::OriginalReady).assert_completed();
        assert_eq!(durable_state(&fixture), before);
        assert_eq!(mirror_files(&fixture.root.join("runner-provider-checkpoints")), provider_before);
        assert_eq!(fs::read(fixture.root.join("original-effect.txt")).unwrap(), effect_before);
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before);
    }
}

fn canonical_reply(ordinal: usize) -> String {
    format!("canonical admission completed response {ordinal}")
}

#[derive(Clone, Default)]
struct CanonicalRunCounters {
    effects: Arc<AtomicUsize>,
    retries: Arc<AtomicUsize>,
    retry_reports: Arc<AtomicUsize>,
    successes: Arc<AtomicUsize>,
    discoveries: Arc<AtomicUsize>,
    catalog_refused: Arc<AtomicBool>,
}

struct CanonicalRootBridge {
    root: PathBuf,
    counters: CanonicalRunCounters,
}

impl RoutedToolBridge for CanonicalRootBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.counters.discoveries.fetch_add(1, Ordering::AcqRel);
        if self.counters.catalog_refused.load(Ordering::Acquire) {
            // The App's JSON parser has separate controls. This typed discovery
            // boundary tests the actual Runner's precedence and error custody.
            return Err(ProviderSessionError::McpCatalog(McpCatalogRefusal {
                kind: McpCatalogKind::MetaTools, row: Some(0), reason: McpCatalogReason::NameMustBeString,
            }));
        }
        Ok(vec![RoutedToolDefinition {
            name: "b".into(), provider_identifier: "canonical-root-contract".into(),
            tool_name: "b".into(), description: None, input_schema: json!({"type":"object"}),
        }])
    }

    fn call_tool(&self, _tool: &RoutedToolDefinition, _args: Value, call_id: &str) -> Result<Value, ProviderSessionError> {
        let ordinal = self.counters.effects.fetch_add(1, Ordering::AcqRel);
        let mut file = fs::OpenOptions::new().write(true).create_new(true)
            .open(self.root.join("canonical-effects").join(format!("{ordinal}.txt")))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        file.write_all(call_id.as_bytes()).and_then(|()| file.sync_all())
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        Ok(json!({"receipt":"committed", "callId":call_id}))
    }
}

fn canonical_build_bindings(agent_id: &str, invalid_model: bool) -> ProductionTurnAgentBuildBindings {
    ProductionTurnAgentBuildBindings::new(
        ProductionTurnAgentStaticConfig {
            model_id: if invalid_model { String::new() } else { ORIGINAL_MODEL.into() },
            agent_token_limit: SAND_AGENT_TOKEN_LIMIT, conversation_id: agent_id.into(),
            is_box_scoped_subagent: false, is_subagent_runner: false, is_shared_room_runner: false,
            sand_send_message_delivery_owed: false, transcripts_folder_available: true,
        },
        Arc::new(|| None), Arc::new(|_, _, _| panic!("bounded canonical admission must not summarize")),
    )
}

fn canonical_runner(
    root: &Path,
    agent_id: &str,
    sink: Arc<dyn AgentStateCheckpointSink>,
    cancellation: RoutedProviderCancellation,
    counters: &CanonicalRunCounters,
) -> ProductionTurnAgentOwner {
    let retries = Arc::clone(&counters.retries);
    let retry_reports = Arc::clone(&counters.retry_reports);
    let successes = Arc::clone(&counters.successes);
    let turn = TurnAgentComposition::new(
        RoutedProvider::Fabushi,
        Arc::new(CanonicalRootBridge { root: root.to_path_buf(), counters: counters.clone() }),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "canonical-root-contract".into(), shell: None, time_zone: Some("UTC".into()),
                transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(), user_full_name: None,
            }, rules: None,
        },
        cancellation,
        Arc::new(ProductionRoutedProviderCheckpointStore::new(root, agent_id, ORIGINAL_STREAM)),
    ).with_fabushi_model(ORIGINAL_MODEL)
        .with_retry_sink(Arc::new(move |_| { retries.fetch_add(1, Ordering::AcqRel); }))
        .with_retry_report_sink(Arc::new(move |_| { retry_reports.fetch_add(1, Ordering::AcqRel); }));
    // Use the same production owner assembly as create_production_runner.
    // These cases deliberately retain that owner across explicit binding
    // changes; the facade/factory path remains covered by the GTH/ORT cases.
    ProductionTurnAgentOwner::new(turn)
        .with_build_bindings(canonical_build_bindings(agent_id, false))
        .with_lifecycle_bindings(
            ProductionTurnAgentLifecycleBindings::new(agent_id, ORIGINAL_STREAM)
                .with_profile_announcement_commit(Some(Arc::new(move || { successes.fetch_add(1, Ordering::AcqRel); }))),
        )
        .with_upgrade_quiesce_signal(Arc::new(AtomicBool::new(false)))
        .with_agent_state_checkpoint_sink(sink)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalObserverBoundary { Checked(usize), Staged }

struct CanonicalRootObserver {
    delegate: Arc<dyn AgentStateCheckpointSink>,
    generation: usize,
    epoch: AtomicUsize,
    boundary: CanonicalObserverBoundary,
    checked: AtomicUsize,
    staged: AtomicUsize,
    persisted: AtomicUsize,
    fired: AtomicUsize,
    captured: Mutex<Option<Option<ProductionAgentCheckpointSnapshot>>>,
    action: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}

impl CanonicalRootObserver {
    fn new(
        delegate: Arc<dyn AgentStateCheckpointSink>,
        generation: usize,
        boundary: CanonicalObserverBoundary,
        action: impl FnOnce() + Send + 'static,
    ) -> Self {
        Self {
            delegate, generation, epoch: AtomicUsize::new(generation), boundary,
            checked: AtomicUsize::new(0), staged: AtomicUsize::new(0), persisted: AtomicUsize::new(0),
            fired: AtomicUsize::new(0), captured: Mutex::new(None), action: Mutex::new(Some(Box::new(action))),
        }
    }

    fn fire(&self) {
        assert_eq!(self.epoch.load(Ordering::Acquire), self.generation);
        assert_eq!(self.fired.fetch_add(1, Ordering::AcqRel), 0, "one owned boundary injection only");
        let action = self.action.lock().expect("owned boundary action").take().expect("unused boundary action");
        // The real getter/stager has returned and released its locks. Preserve
        // that genuine result while the actual competing owner runs.
        action();
    }

    fn assert_captured(&self, expected: &Option<ProductionAgentCheckpointSnapshot>) {
        let captured = self.captured.lock().expect("captured real checked result");
        assert_eq!(captured.as_ref(), Some(expected), "the invocation must pass its original complete checked snapshot");
    }

    fn after_stage(&self, result: &Result<TextTurnCheckpointArtifacts, ProviderSessionError>) {
        self.staged.fetch_add(1, Ordering::AcqRel);
        if result.is_ok() && self.boundary == CanonicalObserverBoundary::Staged { self.fire(); }
    }

    fn finish(&self, actual_generation: u64, checked: usize, staged: usize, persisted: usize) {
        assert_eq!(actual_generation, self.generation as u64);
        assert_eq!(self.checked.load(Ordering::Acquire), checked);
        assert_eq!(self.staged.load(Ordering::Acquire), staged);
        assert_eq!(self.persisted.load(Ordering::Acquire), persisted);
        assert_eq!(self.fired.load(Ordering::Acquire), 1);
        assert!(self.action.lock().expect("consumed owned boundary action").is_none());
        assert_eq!(self.epoch.compare_exchange(self.generation, self.generation + 1, Ordering::AcqRel, Ordering::Acquire),
            Ok(self.generation), "retire this exact observer generation");
    }
}

impl AgentStateCheckpointSink for CanonicalRootObserver {
    fn invalidate_original_turn_completion(&self) { self.delegate.invalidate_original_turn_completion(); }

    fn committed_original_turn_observations(&self) -> Result<Vec<CommittedOriginalTurnObservation>, ProviderSessionError> {
        self.delegate.committed_original_turn_observations()
    }

    fn checked_base_snapshot(&self) -> Result<Option<ProductionAgentCheckpointSnapshot>, ProviderSessionError> {
        let ordinal = self.checked.fetch_add(1, Ordering::AcqRel) + 1;
        let genuine = self.delegate.checked_base_snapshot()?;
        if ordinal == 1 { *self.captured.lock().expect("capture actual projection read") = Some(genuine.clone()); }
        if self.boundary == CanonicalObserverBoundary::Checked(ordinal) { self.fire(); }
        Ok(genuine)
    }

    fn read_state_blob(&self, id: &[u8]) -> Result<Option<Vec<u8>>, ProviderSessionError> {
        self.delegate.read_state_blob(id)
    }

    fn stage_text_turn(&self, expected: &Option<ProductionAgentCheckpointSnapshot>, messages: &[ProviderMessage],
        options: &TurnRunOptions, text: &str) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError>
    {
        self.assert_captured(expected);
        let result = self.delegate.stage_text_turn(expected, messages, options, text);
        self.after_stage(&result);
        result
    }

    fn stage_generated_turn(&self, expected: &Option<ProductionAgentCheckpointSnapshot>, messages: &[ProviderMessage],
        options: &TurnRunOptions, steps: &[Vec<u8>], text: &str) -> Result<TextTurnCheckpointArtifacts, ProviderSessionError>
    {
        self.assert_captured(expected);
        let result = self.delegate.stage_generated_turn(expected, messages, options, steps, text);
        self.after_stage(&result);
        result
    }

    fn persist_staged_text_turn(&self, expected: &Option<ProductionAgentCheckpointSnapshot>, options: &TurnRunOptions,
        checkpoint: &TextTurnCheckpointArtifacts) -> Result<(), ProviderSessionError>
    {
        self.assert_captured(expected);
        self.persisted.fetch_add(1, Ordering::AcqRel);
        self.delegate.persist_staged_text_turn(expected, options, checkpoint)
    }
}

fn canonical_expected_turn(options: &TurnRunOptions, text: &str) -> CanonicalExpectedTurn {
    CanonicalExpectedTurn {
        message_id: options.message_id.as_ref().or(options.inference_request_id.as_ref())
            .expect("actual invocation user identity").clone(),
        user_text: options.recent_message_text.clone().expect("actual invocation user text"),
        request_id: options.inference_request_id.clone().expect("actual invocation request identity"),
        assistant_text: text.into(),
    }
}

fn canonical_checkpoint_view(
    root: &Path,
    agent_id: &str,
    sessions: &Arc<ProductionSessionWorkers>,
    expected_turns: &[CanonicalExpectedTurn],
    check_journal: bool,
) -> ExpectedCheckpoint {
    let store = sessions.open_agent_store_owner(agent_id).expect("actual canonical checkpoint reader");
    let snapshot = store.checkpoint_snapshot_checked().expect("actual strict canonical snapshot");
    let state_bytes = snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone());
    let fields = decode_conversation_state_recovery_fields(state_bytes.as_deref().unwrap_or_default())
        .expect("checked canonical state protobuf");
    assert_eq!(fields.turns.len(), expected_turns.len(), "only explicitly committed invocations survive");
    let mut critical_blobs = snapshot.as_ref().map(|snapshot|
        vec![(snapshot.root_blob_id.clone(), snapshot.checkpoint_bytes.clone())]).unwrap_or_default();
    let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
    let mut journal = Vec::new();
    for (id, expected) in fields.turns.iter().zip(expected_turns) {
        let turn = store.get_blob(id).expect("canonical turn read").expect("canonical turn exists");
        let Some(ConversationTurnStructureFields::Agent { user_message, steps, request_id }) =
            decode_conversation_turn_structure_fields(&turn).expect("canonical Agent turn")
        else { panic!("these real generated controls must retain Agent turns"); };
        assert_eq!(request_id.as_deref(), Some(expected.request_id.as_str()));
        assert_eq!(steps.len(), 1, "no failed invocation or tool step may be fabricated");
        let user_bytes = store.get_blob(&user_message).expect("canonical user read").expect("canonical user exists");
        let user = decode_user_message_identity_fields(&user_bytes).expect("canonical accepted user identity");
        assert_eq!(user.message_id, expected.message_id);
        assert_eq!(user.text, expected.user_text);
        let step = store.get_blob(&steps[0]).expect("canonical Assistant read").expect("canonical Assistant exists");
        assert_eq!(codec.decode_step(&step).expect("canonical generated Assistant projection"),
            DecodedTranscriptStep::Assistant { text: expected.assistant_text.clone() });
        critical_blobs.push((id.clone(), turn));
        critical_blobs.push((user_message, user_bytes));
        critical_blobs.push((steps[0].clone(), step));
        journal.push(json!({"role":"user", "message":{"content":[{"type":"text", "text":expected.user_text}]}}));
        journal.push(json!({"role":"assistant", "message":{"content":[{"type":"text", "text":expected.assistant_text}]}}));
    }
    if check_journal {
        let path = root.join("transcripts").join(agent_id).join(format!("{agent_id}.jsonl"));
        let text = fs::read_to_string(path).expect("actual canonical Journal");
        let actual = text.lines().map(|line| serde_json::from_str::<Value>(line)
            .expect("actual canonical Journal occurrence")).collect::<Vec<_>>();
        assert_eq!(actual, journal, "exact user/Assistant order without refused or replayed work");
    }
    ExpectedCheckpoint {
        root_blob_id: snapshot.map(|snapshot| snapshot.root_blob_id).unwrap_or_default(),
        state_bytes, last_user_message_id: expected_turns.last().map(|turn| turn.message_id.clone()),
        compaction_epoch: fields.self_summary_count, critical_blobs,
    }
}

fn mutate_canonical_root(
    root: &Path,
    agent_id: &str,
    sessions: &Arc<ProductionSessionWorkers>,
    store: &ProductionAgentStore,
    drift: CanonicalDrift,
) {
    let previous = store.checkpoint_snapshot_checked().expect("capture actual competing write base");
    match drift {
        CanonicalDrift::AbsentToEmpty => {
            assert!(previous.is_none());
            store.handle_checkpoint_bytes_blocking(&[]).expect("real present-empty competitor");
        }
        CanonicalDrift::EmptyToAbsent => {
            let previous = previous.as_ref().expect("real present-empty base");
            assert!(previous.checkpoint_bytes.is_empty());
            assert!(sessions.open_agent_db_owner(agent_id).expect("real root metadata owner")
                .compare_and_set_latest_root_blob_id(&previous.root_blob_id, &[])
                .expect("explicit competing absent binding"));
        }
        CanonicalDrift::SameBytesOtherRoot => {
            let previous = previous.as_ref().expect("real fixed root base");
            let next_root = b"other-legacy-root";
            sessions.create_agent_blob_store(agent_id).expect("real competing blob owner")
                .set_blob_blocking(next_root, &previous.checkpoint_bytes).expect("same bytes under another valid ID");
            assert!(sessions.open_agent_db_owner(agent_id).expect("real metadata owner")
                .compare_and_set_latest_root_blob_id(&previous.root_blob_id, next_root)
                .expect("real competing root binding"));
        }
        CanonicalDrift::SameRootOtherBytes => {
            let previous = previous.as_ref().expect("real fixed root base");
            assert_eq!(previous.root_blob_id, LEGACY_FIXED_ROOT_ID);
            assert_ne!(previous.checkpoint_bytes, PRIOR_STATE);
            sessions.create_agent_blob_store(agent_id).expect("real competing blob owner")
                .set_blob_blocking(&previous.root_blob_id, PRIOR_STATE).expect("different valid bytes under the same fixed ID");
        }
    }
    store.reset_from_db_checked().expect("explicit checked store reload, never an implicit sink rebind");
    let next = store.checkpoint_snapshot_checked().expect("real competing snapshot is admissible");
    assert_ne!(next, previous);
    assert!(root.join("agents").join(agent_id).is_dir());
}

fn restore_canonical_snapshot(
    sessions: &Arc<ProductionSessionWorkers>,
    agent_id: &str,
    store: &ProductionAgentStore,
    original: &Option<ProductionAgentCheckpointSnapshot>,
) {
    let current = store.checkpoint_snapshot_checked().expect("read the explicit competing root before repair");
    let next_id = original.as_ref().map(|snapshot| snapshot.root_blob_id.as_slice()).unwrap_or_default();
    if let Some(snapshot) = original {
        sessions.create_agent_blob_store(agent_id).expect("existing repair blob owner")
            .set_blob_blocking(&snapshot.root_blob_id, &snapshot.checkpoint_bytes)
            .expect("restore only captured original checkpoint bytes");
    }
    assert!(sessions.open_agent_db_owner(agent_id).expect("existing repair metadata owner")
        .compare_and_set_latest_root_blob_id(
            current.as_ref().map(|snapshot| snapshot.root_blob_id.as_slice()).unwrap_or_default(), next_id,
        ).expect("explicitly restore the captured root binding"));
    store.reset_from_db_checked().expect("explicit checked repair reload");
    assert_eq!(&store.checkpoint_snapshot_checked().expect("strict repaired snapshot"), original);
}

enum CanonicalFaultRepair { Metadata(String), Blob(Vec<u8>, Vec<u8>) }

fn inject_live_canonical_read_fault(
    root: &Path,
    agent_id: &str,
    store: &ProductionAgentStore,
    scenario: CanonicalRootScenario,
) -> CanonicalFaultRepair {
    let snapshot = store.checkpoint_snapshot_checked().expect("healthy strict fixture snapshot")
        .expect("strict fault fixture has a real prior checkpoint");
    let directory = root.join("agents").join(agent_id);
    if matches!(scenario, CanonicalRootScenario::StrictRead(Fault::MissingRootBlob | Fault::MalformedRootBlob)
        | CanonicalRootScenario::MissingDirectTurn)
    {
        let id = if scenario == CanonicalRootScenario::MissingDirectTurn {
            decode_conversation_state_recovery_fields(&snapshot.checkpoint_bytes).expect("actual direct references")
                .turns.into_iter().next().expect("actual existing direct turn reference")
        } else { snapshot.root_blob_id };
        let bytes = store.get_blob(&id).expect("capture owned fault target").expect("owned target exists");
        let db = open_existing(&directory.join(CONVERSATION_BLOBS_FILENAME), false);
        let changed = if scenario == CanonicalRootScenario::StrictRead(Fault::MalformedRootBlob) {
            db.execute("UPDATE blobs SET data=?2 WHERE id=?1", params![hex(&id), vec![0_u8]])
        } else { db.execute("DELETE FROM blobs WHERE id=?1", params![hex(&id)]) }
            .expect("inject the exact owned live blob fault");
        assert_eq!(changed, 1);
        return CanonicalFaultRepair::Blob(id, bytes);
    }
    let CanonicalRootScenario::StrictRead(fault) = scenario else { panic!("expected a typed strict read fault"); };
    let db = open_existing(&directory.join(STORE_FILENAME), false);
    let original: String = db.query_row("SELECT value FROM kv WHERE key='metadata'", [], |row| row.get(0))
        .expect("capture exact owned metadata image");
    let changed = match fault {
        Fault::MissingMetadata => db.execute("DELETE FROM kv WHERE key='metadata'", []),
        Fault::MalformedMetadata => db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![hex(b"{")]),
        Fault::MissingRootField | Fault::NonStringRootField => {
            let bytes = original.as_bytes().chunks_exact(2).map(|pair|
                u8::from_str_radix(std::str::from_utf8(pair).expect("actual metadata hex"), 16).expect("actual metadata byte"))
                .collect::<Vec<_>>();
            let mut value: Value = serde_json::from_slice(&bytes).expect("actual metadata JSON");
            if fault == Fault::MissingRootField { value.as_object_mut().unwrap().remove("latestRootBlobId"); }
            else { value["latestRootBlobId"] = json!(42); }
            db.execute("UPDATE kv SET value=?1 WHERE key='metadata'", params![hex(&serde_json::to_vec(&value).unwrap())])
        }
        _ => unreachable!(),
    }.expect("inject only the owned live metadata fault");
    assert_eq!(changed, 1);
    CanonicalFaultRepair::Metadata(original)
}

fn repair_live_canonical_read_fault(root: &Path, agent_id: &str, sessions: &Arc<ProductionSessionWorkers>, repair: CanonicalFaultRepair) {
    match repair {
        CanonicalFaultRepair::Metadata(raw) => {
            assert_eq!(open_existing(&root.join("agents").join(agent_id).join(STORE_FILENAME), false).execute(
                "INSERT INTO kv(key,value) VALUES('metadata',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![raw],
            ).expect("restore the exact captured metadata image"), 1);
        }
        CanonicalFaultRepair::Blob(id, bytes) => {
            sessions.create_agent_blob_store(agent_id).expect("existing explicit repair blob owner")
                .set_blob_blocking(&id, &bytes).expect("restore the exact captured blob image");
        }
    }
}

fn assert_canonical_custody(runner: &ProductionTurnAgentOwner) {
    let finished = runner.last_finished().expect("actual generated shell outcome");
    assert_eq!(finished.owner.request_id, ORIGINAL_STREAM);
    assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::Known {
        run_generation: finished.owner.generation,
        snapshot: ToolEffectCustodySnapshot {
            started_revision: 0, checkpoint_revision: 0, pending_effects: 0, unknown_result: None,
        },
    }));
}

fn assert_canonical_success(
    runner: &mut ProductionTurnAgentOwner,
    root: &Path,
    messages: &[ProviderMessage],
    options: TurnRunOptions,
    ordinal: usize,
) {
    let mut deltas = Vec::new();
    let result = runner.run_routed_provider_with_options(root, messages, options, &mut |delta, _| deltas.push(delta.to_string()))
        .expect("actual explicit generated invocation succeeds");
    assert_eq!(result, canonical_reply(ordinal));
    assert_eq!(deltas, vec![canonical_reply(ordinal)]);
    assert_eq!(runner.last_finished().unwrap().outcome, TerminalOutcome::Completed);
    assert_canonical_custody(runner);
}

fn canonical_run_error(
    runner: &mut ProductionTurnAgentOwner,
    root: &Path,
    messages: &[ProviderMessage],
    options: TurnRunOptions,
    expected_delta: Option<String>,
) -> ProviderSessionError {
    let mut deltas = Vec::new();
    let error = runner.run_routed_provider_with_options(root, messages, options, &mut |delta, _| deltas.push(delta.to_string()))
        .expect_err("the actual canonical admission/publication refusal must remain an error");
    assert_eq!(deltas, expected_delta.into_iter().collect::<Vec<_>>());
    error
}

fn assert_canonical_failed_shell(runner: &ProductionTurnAgentOwner, error: &ProviderSessionError) {
    let finished = runner.last_finished().expect("actual refused shell terminal");
    assert_eq!(finished.owner.request_id, ORIGINAL_STREAM);
    assert_eq!(finished.outcome, TerminalOutcome::Failed {
        retryable: matches!(error, ProviderSessionError::Transport(_)), message: error.to_string(),
    });
}

fn run_canonical_competitor(
    cancellation: RoutedProviderCancellation,
    run: impl FnOnce() -> DurableState + Send + 'static,
) -> DurableState {
    // This helper runs only inside the private, PID-bound harness child. A
    // separate thread avoids nesting the generated stream's LocalPool executor.
    let worker = thread::spawn(run);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !worker.is_finished() && Instant::now() < deadline { thread::sleep(Duration::from_millis(2)); }
    if !worker.is_finished() {
        cancellation.cancel("owned canonical competitor exceeded its deadline");
        let cleanup_deadline = Instant::now() + Duration::from_secs(5);
        while !worker.is_finished() && Instant::now() < cleanup_deadline { thread::sleep(Duration::from_millis(2)); }
        if worker.is_finished() {
            let outcome = worker.join();
            panic!("owned canonical competitor exceeded its deadline; joined outcome={outcome:?}");
        }
        // Do not detach an unresolved writer or enter an unbounded join. The
        // existing parent captures the failing child and reaps this exact PID.
        eprintln!("owned canonical competitor did not stop after cancellation; aborting only the private harness child");
        std::process::abort();
    }
    worker.join().expect("the exact owned competing invocation thread completed")
}

fn produce_canonical_root(
    request: &ChildRequest,
    sessions: &Arc<ProductionSessionWorkers>,
) -> (ProducedOriginal, Vec<CanonicalExpectedTurn>) {
    let scenario = request.canonical_scenario.expect("typed canonical-root process scenario");
    let root = &request.fixture_root;
    original_process_stage(&request.token, &format!("canonical-{scenario:?}-admission"));
    let runtime = ProductionTranscriptRuntime::new(Some(root));
    let args = json!({
        "agentId":request.agent_id, "streamId":ORIGINAL_STREAM, "prompt":USER_TEXT,
        "sessionKey":format!("agent:{}:main", request.agent_id), "clientNonce":"canonical-root-user-nonce",
    });
    let accepted = runtime.accept_routed_send(&args, |accepted| {
        persist_accepted_send_prompt_context(sessions, &args, accepted)
            .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
    }).expect("actual canonical-root routed admission");
    assert!(!accepted.duplicate);
    let control = runtime.capture_routed_turn_start(&request.agent_id, ORIGINAL_STREAM).expect("actual admitted lease control");
    let cancellation = control.cancellation();
    let admission = sessions.read_session_work_admission(&request.agent_id).expect("actual Session admission").expect("main Session");
    let original = runtime.bind_original_turn_model(&request.agent_id, ORIGINAL_STREAM, &admission, ORIGINAL_MODEL)
        .expect("bind admitted original model").expect("ordinary original user");
    assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
    let host = composition(root, sessions);
    let sink: Arc<dyn AgentStateCheckpointSink> = host.compose_production_checkpoint_sink(
        sessions, root, &request.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("actual canonical-root sink composition").expect("main canonical sink");
    let store = sessions.open_agent_store_owner(&request.agent_id).expect("actual canonical root owner");
    let initial = sink.checked_base_snapshot().expect("actual initially admissible complete snapshot");
    assert_eq!(initial, store.checkpoint_snapshot_checked().expect("independent actual checked root read"));
    assert!(sink.committed_original_turn_observations().expect("initial completion log").is_empty());
    fs::create_dir(root.join("canonical-effects")).expect("owned canonical effect directory");
    let counters = CanonicalRunCounters::default();
    let mut runner = canonical_runner(root, &request.agent_id, Arc::clone(&sink), cancellation.clone(), &counters);
    let messages = vec![ProviderMessage { role: "user".into(), content: USER_TEXT.into() }];
    let options = TurnRunOptions {
        inference_request_id: Some(ORIGINAL_STREAM.into()), message_id: Some(original.user_message_id.clone()),
        recent_message_text: Some(USER_TEXT.into()), turn_started_at_ms: Some(1), ..TurnRunOptions::default()
    };
    let nudge = shape_reply_nudge_turn_input(&messages, &messages, &options);
    let mut expected_turns = if matches!(scenario, CanonicalRootScenario::StrictRead(_) | CanonicalRootScenario::MissingDirectTurn) {
        vec![CanonicalExpectedTurn {
            message_id: USER_ID.into(), user_text: USER_TEXT.into(), request_id: "checkpoint-restart-request".into(),
            assistant_text: ASSISTANT_TEXT.into(),
        }]
    } else { Vec::new() };

    match scenario {
        CanonicalRootScenario::Valid(_) => {
            let mut custody = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
            assert_canonical_success(&mut runner, root, &messages, options.clone(), 0);
            custody.record(runner.last_tool_effect_custody());
            let first = sink.checked_base_snapshot().expect("own commit advances retained complete snapshot");
            assert_ne!(first, initial);
            assert_eq!(first, store.checkpoint_snapshot_checked().unwrap());
            assert_eq!(sink.committed_original_turn_observations().unwrap().len(), 1);
            expected_turns.push(canonical_expected_turn(&options, &canonical_reply(0)));
            assert_canonical_success(&mut runner, root, &nudge.provider_messages, nudge.options.clone(), 1);
            custody.record(runner.last_tool_effect_custody());
            let second = sink.checked_base_snapshot().expect("second explicit own commit advances retained snapshot");
            assert_ne!(second, first);
            assert_eq!(second, store.checkpoint_snapshot_checked().unwrap());
            let observations = sink.committed_original_turn_observations().expect("valid repeated completion observations");
            assert_eq!(observations.len(), 2);
            prove_completed_original_turn(&original, project_turn_terminal(false, false, true, 0, false),
                original_turn_result_digest(&canonical_reply(1)), 0, false, &custody, &observations)
                .expect("real repeated successful invocations remain eligible");
            expected_turns.push(canonical_expected_turn(&nudge.options, &canonical_reply(1)));
        }
        CanonicalRootScenario::PreProjection(drift) => {
            mutate_canonical_root(root, &request.agent_id, sessions, &store, drift);
            let competing = durable_state_for(root, &request.agent_id);
            let error = canonical_run_error(&mut runner, root, &messages, options.clone(), None);
            assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains("checkpoint root changed")));
            assert_canonical_failed_shell(&runner, &error);
            assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
            assert!(sink.checked_base_snapshot().is_err(), "the old sink never adopts the explicitly reloaded competitor");
            assert_eq!(durable_state_for(root, &request.agent_id), competing);
        }
        CanonicalRootScenario::PostProjection => {
            let competing = Arc::new(Mutex::new(None));
            let observed = Arc::clone(&competing);
            let competitor = Arc::clone(&store);
            let effect_root = root.clone();
            let effect_agent = request.agent_id.clone();
            let observer = Arc::new(CanonicalRootObserver::new(Arc::clone(&sink), 1, CanonicalObserverBoundary::Checked(1), move || {
                competitor.handle_checkpoint_bytes_blocking(PRIOR_STATE).expect("real post-projection competing root");
                *observed.lock().unwrap() = Some(durable_state_for(&effect_root, &effect_agent));
            }));
            runner = runner.with_agent_state_checkpoint_sink(observer.clone());
            let error = canonical_run_error(&mut runner, root, &messages, options.clone(), None);
            assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains("checkpoint root changed")));
            assert_canonical_failed_shell(&runner, &error);
            assert_canonical_custody(&runner);
            observer.finish(runner.last_finished().unwrap().owner.generation, 2, 0, 0);
            assert_eq!(durable_state_for(root, &request.agent_id), competing.lock().unwrap().take().unwrap());
        }
        CanonicalRootScenario::StrictRead(_) | CanonicalRootScenario::MissingDirectTurn => {
            let healthy = durable_state_for(root, &request.agent_id);
            let repair = inject_live_canonical_read_fault(root, &request.agent_id, &store, scenario);
            let damaged = durable_state_for(root, &request.agent_id);
            assert_ne!(damaged, healthy);
            let error = canonical_run_error(&mut runner, root, &messages, options.clone(), None);
            assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains("checkpoint prior state is unavailable")));
            assert_canonical_failed_shell(&runner, &error);
            assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
            assert!(sink.checked_base_snapshot().is_err());
            assert_eq!(durable_state_for(root, &request.agent_id), damaged, "strict refusal cannot repair damaged authority");
            repair_live_canonical_read_fault(root, &request.agent_id, sessions, repair);
            assert_eq!(durable_state_for(root, &request.agent_id), healthy, "only explicit exact repair restores fixture bytes");
            assert_eq!(sink.checked_base_snapshot().expect("original still-current root after exact repair"), initial);
        }
        CanonicalRootScenario::RetiredOwners => {
            sessions.prepare_existing_agent(&request.agent_id).expect("finish ordinary outline maintenance before retirement")
                .expect("existing admitted Session before retirement");
            let before = durable_state_for(root, &request.agent_id);
            assert!(sessions.retire_agent_owners(&request.agent_id, true).expect("complete actual per-agent retirement"));
            assert_eq!(sessions.active_agent_store_owner_count(), 0);
            assert_eq!(sessions.active_agent_db_owner_count(), 0);
            assert_eq!(sessions.active_worker_count(), 0);
            let error = canonical_run_error(&mut runner, root, &messages, options.clone(), None);
            assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains("checkpoint prior state is unavailable")));
            assert_canonical_failed_shell(&runner, &error);
            assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
            assert!(sink.checked_base_snapshot().is_err());
            assert_eq!(sessions.active_agent_store_owner_count(), 0, "refusal cannot reopen retired owners");
            assert_eq!(sessions.active_agent_db_owner_count(), 0);
            assert_eq!(sessions.active_worker_count(), 0);
            assert_eq!(durable_state_for(root, &request.agent_id), before);
        }
        CanonicalRootScenario::SameSinkBeforeStage | CanonicalRootScenario::SameSinkAfterStage => {
            let after_stage = scenario == CanonicalRootScenario::SameSinkAfterStage;
            let competing = Arc::new(Mutex::new(None));
            let observed = Arc::clone(&competing);
            let shared = Arc::clone(&sink);
            let effect_root = root.clone();
            let effect_agent = request.agent_id.clone();
            let effect_messages = messages.clone();
            let effect_options = options.clone();
            let effect_cancellation = cancellation.clone();
            let effect_counters = counters.clone();
            let competitor_ordinal = usize::from(after_stage);
            let observer = Arc::new(CanonicalRootObserver::new(Arc::clone(&sink), 1,
                if after_stage { CanonicalObserverBoundary::Staged } else { CanonicalObserverBoundary::Checked(2) }, move || {
                    let state = run_canonical_competitor(effect_cancellation.clone(), move || {
                        let mut competitor = canonical_runner(&effect_root, &effect_agent, Arc::clone(&shared), effect_cancellation, &effect_counters);
                        assert_canonical_success(&mut competitor, &effect_root, &effect_messages, effect_options, competitor_ordinal);
                        assert_eq!(shared.committed_original_turn_observations().expect("B's genuine successful commit").len(), 1);
                        durable_state_for(&effect_root, &effect_agent)
                    });
                    *observed.lock().unwrap() = Some(state);
                }));
            runner = runner.with_agent_state_checkpoint_sink(observer.clone());
            let error = canonical_run_error(&mut runner, root, &messages, options.clone(), Some(canonical_reply(1 - competitor_ordinal)));
            assert!(matches!(&error, ProviderSessionError::Protocol(message)
                if message.contains("captured") || message.contains("root changed")));
            assert_eq!(error.to_string().starts_with("generated Agent stream persistence failed:"), after_stage);
            assert_canonical_failed_shell(&runner, &error);
            assert_canonical_custody(&runner);
            observer.finish(runner.last_finished().unwrap().owner.generation, 2, 1, usize::from(after_stage));
            assert_eq!(durable_state_for(root, &request.agent_id), competing.lock().unwrap().take().unwrap(),
                "A cannot append on B's root or mutate B's confirmation/Journal/blob bytes");
            expected_turns.push(canonical_expected_turn(&options, &canonical_reply(competitor_ordinal)));
        }
        CanonicalRootScenario::Early(kind) => {
            assert_canonical_success(&mut runner, root, &messages, options.clone(), 0);
            let mut custody = ToolEffectCustodyReport::new(ORIGINAL_STREAM);
            custody.record(runner.last_tool_effect_custody());
            let observations = sink.committed_original_turn_observations().expect("actual prior successful authority");
            assert_eq!(observations.len(), 1);
            prove_completed_original_turn(&original, project_turn_terminal(false, false, true, 0, false),
                original_turn_result_digest(&canonical_reply(0)), 0, false, &custody, &observations)
                .expect("real prior success qualifies before early refusal");
            expected_turns.push(canonical_expected_turn(&options, &canonical_reply(0)));
            let prior = sink.checked_base_snapshot().expect("capture prior successful expectation");
            let prior_generation = runner.last_finished().unwrap().owner.generation;
            let competing = Arc::new(Mutex::new(None));
            let mut observer = None;
            match kind {
                CanonicalEarlyError::Build => { runner = runner.with_build_bindings(canonical_build_bindings(&request.agent_id, true)); }
                CanonicalEarlyError::Prompt => {}
                CanonicalEarlyError::Catalog | CanonicalEarlyError::Projection => {
                    store.handle_checkpoint_bytes_blocking(PRIOR_STATE).expect("actual competing root before early projection");
                    counters.catalog_refused.store(kind == CanonicalEarlyError::Catalog, Ordering::Release);
                }
                CanonicalEarlyError::Preflight => {
                    let observed = Arc::clone(&competing);
                    let competitor = Arc::clone(&store);
                    let effect_root = root.clone();
                    let effect_agent = request.agent_id.clone();
                    let boundary = Arc::new(CanonicalRootObserver::new(Arc::clone(&sink), (prior_generation + 1) as usize,
                        CanonicalObserverBoundary::Checked(1), move || {
                            competitor.handle_checkpoint_bytes_blocking(PRIOR_STATE).expect("actual early preflight competitor");
                            *observed.lock().unwrap() = Some(durable_state_for(&effect_root, &effect_agent));
                        }));
                    runner = runner.with_agent_state_checkpoint_sink(boundary.clone());
                    observer = Some(boundary);
                }
            }
            let before = durable_state_for(root, &request.agent_id);
            let error = canonical_run_error(&mut runner, root,
                if kind == CanonicalEarlyError::Prompt { &[] } else { &messages }, nudge.options.clone(), None);
            match kind {
                CanonicalEarlyError::Build => assert!(matches!(&error, ProviderSessionError::Configuration(message)
                    if message == "production turn model id is empty")),
                CanonicalEarlyError::Prompt => assert!(matches!(&error, ProviderSessionError::Configuration(message)
                    if message == "Runner production turn requires a non-empty user prompt.")),
                CanonicalEarlyError::Catalog => assert!(matches!(&error, ProviderSessionError::McpCatalog(refusal)
                    if *refusal == (McpCatalogRefusal { kind: McpCatalogKind::MetaTools, row: Some(0), reason: McpCatalogReason::NameMustBeString }))),
                _ => assert!(matches!(&error, ProviderSessionError::Protocol(message) if message.contains("checkpoint root changed"))),
            }
            if matches!(kind, CanonicalEarlyError::Build | CanonicalEarlyError::Prompt) {
                assert_eq!(runner.last_finished().unwrap().owner.generation, prior_generation,
                    "pre-shell refusal preserves the historical finished invocation; it does not invent a new terminal");
                assert_eq!(runner.last_finished().unwrap().outcome, TerminalOutcome::Completed);
            } else { assert_canonical_failed_shell(&runner, &error); }
            if let Some(observer) = observer.as_ref() {
                assert_canonical_custody(&runner);
                observer.finish(runner.last_finished().unwrap().owner.generation, 2, 0, 0);
                assert_eq!(durable_state_for(root, &request.agent_id), competing.lock().unwrap().take().unwrap());
            } else {
                assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
                assert_eq!(durable_state_for(root, &request.agent_id), before);
            }
            assert!(sink.committed_original_turn_observations().is_err(), "early errors must revoke genuine prior observations");
            counters.catalog_refused.store(false, Ordering::Release);
            if kind == CanonicalEarlyError::Build { runner = runner.with_build_bindings(canonical_build_bindings(&request.agent_id, false)); }
            if matches!(kind, CanonicalEarlyError::Catalog | CanonicalEarlyError::Projection | CanonicalEarlyError::Preflight) {
                restore_canonical_snapshot(sessions, &request.agent_id, &store, &prior);
            }
            assert_eq!(sink.checked_base_snapshot().expect("only exact explicit repair restores currentness"), prior);
            assert!(sink.committed_original_turn_observations().is_err());
            assert_canonical_success(&mut runner, root, &nudge.provider_messages, nudge.options.clone(), 1);
            assert!(sink.committed_original_turn_observations().is_err(), "later actual success cannot revive completion authority");
            expected_turns.push(canonical_expected_turn(&nudge.options, &canonical_reply(1)));
        }
        CanonicalRootScenario::JournalAfterCommit => {
            assert_canonical_success(&mut runner, root, &messages, options.clone(), 0);
            assert_eq!(sink.committed_original_turn_observations().expect("actual prior Journal observation").len(), 1);
            expected_turns.push(canonical_expected_turn(&options, &canonical_reply(0)));
            let prior = sink.checked_base_snapshot().expect("complete pre-fault expectation");
            let directory = root.join("transcripts").join(&request.agent_id);
            let journal_path = directory.join(format!("{}.jsonl", request.agent_id));
            let journal_before = fs::read(&journal_path).expect("exact pre-fault Journal bytes");
            let wal_path = directory.join(format!("{}.journal-pending.json", request.agent_id));
            assert!(!wal_path.exists());
            let captured = Arc::new(Mutex::new(None));
            let saved = Arc::clone(&captured);
            let calls = Arc::new(AtomicUsize::new(0));
            let notifications = Arc::clone(&calls);
            let fault_path = wal_path.clone();
            let subscription = store.subscribe_to_metadata(ProductionAgentMetadataKey::LatestRootBlobId, Arc::new(move |_| {
                assert_eq!(notifications.fetch_add(1, Ordering::AcqRel), 0, "one exact after-COMMIT WAL fault");
                let bytes = fs::read(&fault_path).expect("actual WAL was prepared before canonical COMMIT");
                assert!(!bytes.is_empty());
                *saved.lock().unwrap() = Some(bytes);
                fs::remove_file(&fault_path).expect("remove only the owned prepared WAL after canonical COMMIT");
            }));
            let error = canonical_run_error(&mut runner, root, &nudge.provider_messages, nudge.options.clone(), Some(canonical_reply(1)));
            drop(subscription);
            assert_eq!(calls.load(Ordering::Acquire), 1);
            assert!(matches!(&error, ProviderSessionError::Protocol(message)
                if message.starts_with("generated Agent stream persistence failed:")
                    && message.contains("transcript mirror commit failed after durable checkpoint")
                    && message.contains("prepared transcript WAL is missing at commit")));
            assert_canonical_failed_shell(&runner, &error);
            assert_canonical_custody(&runner);
            let durable = store.checkpoint_snapshot_checked().expect("actual post-COMMIT canonical root");
            assert_ne!(durable, prior, "Journal refusal is not canonical rollback");
            assert_eq!(fs::read(&journal_path).unwrap(), journal_before);
            assert!(!wal_path.exists());
            assert!(sink.checked_base_snapshot().is_err(), "the failed sink retains its old expectation");
            assert!(sink.committed_original_turn_observations().is_err());
            let wal = captured.lock().unwrap().take().expect("captured exact prepared WAL bytes");
            let mut restored = fs::OpenOptions::new().write(true).create_new(true).open(&wal_path)
                .expect("restore only the explicitly removed owned WAL");
            restored.write_all(&wal).and_then(|()| restored.sync_all()).expect("durably restore captured WAL bytes");
            drop(restored);
            assert_eq!(fs::read(&wal_path).unwrap(), wal);
            let before = durable_state_for(root, &request.agent_id);
            let refusal = canonical_run_error(&mut runner, root, &nudge.provider_messages, nudge.options.clone(), None);
            assert!(matches!(&refusal, ProviderSessionError::Protocol(message) if message.contains("checkpoint root changed")));
            assert_canonical_failed_shell(&runner, &refusal);
            assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
            assert_eq!(durable_state_for(root, &request.agent_id), before,
                "the stale sink cannot recover the WAL, rebind, or dispatch another model request");
            expected_turns.push(canonical_expected_turn(&nudge.options, &canonical_reply(1)));
        }
    }
    assert!(!cancellation.is_cancelled());
    assert_eq!(counters.effects.load(Ordering::Acquire), 0);
    assert_eq!(counters.retries.load(Ordering::Acquire), 0);
    assert_eq!(counters.retry_reports.load(Ordering::Acquire), 0);
    assert_eq!(counters.successes.load(Ordering::Acquire), usize::from(scenario.request_count() != 0));
    assert!(mirror_files(&root.join("canonical-effects")).is_empty());
    assert!(mirror_files(&root.join("runner-provider-checkpoints")).is_empty(), "no fabricated provider tool checkpoint");
    if !matches!(scenario, CanonicalRootScenario::Valid(_)) {
        assert!(sink.committed_original_turn_observations().is_err(), "every actual failed owner must remain ineligible");
    }
    let db = open_existing(&root.join("agents").join(&request.agent_id).join(STORE_FILENAME), true);
    let terminal_count: i64 = db.query_row("SELECT COUNT(*) FROM kv WHERE key=?1", params![ORIGINAL_TERMINAL_KEY], |row| row.get(0)).unwrap();
    assert_eq!(terminal_count, 0);
    let user_raw: String = db.query_row("SELECT entry FROM transcript_entries WHERE id=?1", params![original.user_message_id], |row| row.get(0)).unwrap();
    let user: Value = serde_json::from_str(&user_raw).unwrap();
    assert_eq!(user.get("confirmed").and_then(Value::as_bool),
        expected_turns.iter().any(|turn| turn.message_id == original.user_message_id).then_some(true));
    drop(db);
    let checkpoint = if scenario == CanonicalRootScenario::RetiredOwners {
        request.expected.clone()
    } else {
        sessions.prepare_existing_agent(&request.agent_id).expect("complete existing outline maintenance before closure").expect("existing Session");
        canonical_checkpoint_view(root, &request.agent_id, sessions, &expected_turns, scenario != CanonicalRootScenario::JournalAfterCommit)
    };
    assert!(runtime.quiesce_for_upgrade().expect("actual canonical-root original marker").quiescing);
    let markers = runtime.upgrade_resume_store().unwrap().list_pending_checked().expect("actual pending original marker");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].original_work.as_ref(), Some(&original));
    assert!(runtime.settle_routed_turn_control(&control, 10).expect("retire the exact admitted lease").is_some());
    drop((runner, sink, host, store, runtime));
    (ProducedOriginal { original, checkpoint }, expected_turns)
}

fn assert_canonical_root_reopen(request: &ChildRequest, sessions: &Arc<ProductionSessionWorkers>) {
    let original = request.original.as_ref().expect("actual canonical producer original reference");
    let expected_turns = request.canonical_turns.as_ref().expect("actual producer's explicitly committed invocation expectations");
    let prepared = sessions.prepare_existing_agent(&request.agent_id)
        .expect("fresh real Session opens intact canonical root").expect("existing Session");
    assert_eq!(prepared.persisted_root_blob_id, request.expected.root_blob_id);
    let host = composition(&request.fixture_root, sessions);
    // In the post-durable Journal case this new owner performs the existing
    // pending-WAL recovery. Parent checks that first recovery separately from
    // every subsequent read-only consumer.
    let sink = host.compose_production_checkpoint_sink(
        sessions, &request.fixture_root, &request.agent_id, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
    ).expect("fresh actual canonical sink and Journal recovery").expect("main canonical sink");
    let snapshot = sink.checked_base_snapshot().expect("fresh checked complete root identity");
    assert_eq!(snapshot.as_ref().map(|snapshot| &snapshot.root_blob_id),
        request.expected.state_bytes.as_ref().map(|_| &request.expected.root_blob_id));
    assert_eq!(snapshot.as_ref().map(|snapshot| &snapshot.checkpoint_bytes), request.expected.state_bytes.as_ref());
    assert_eq!(canonical_checkpoint_view(&request.fixture_root, &request.agent_id, sessions, expected_turns, true), request.expected);
    let watermark = host.confirmed_user_turn_watermark(sessions, &request.agent_id).expect("fresh canonical watermark");
    assert_eq!(watermark.has_user_turn, !expected_turns.is_empty());
    assert_eq!(watermark.last_user_message_id, request.expected.last_user_message_id);
    assert_eq!(host.prompt_compaction_epoch(sessions, &request.agent_id).expect("fresh canonical compaction epoch"),
        request.expected.compaction_epoch);
    assert!(sink.committed_original_turn_observations().expect("new sink has no invocation receipts").is_empty());
    assert!(reconcile_completed_original_turn(sessions, original).expect("strict fresh original completion lookup").is_none());
    let markers = SandUpgradeResumeStore::new(&request.fixture_root).list_pending_checked().expect("fresh checked original marker");
    assert_eq!(markers, vec![request.original_marker.clone().expect("actual canonical producer marker")]);
    assert_eq!(markers[0].original_work.as_ref(), Some(original));
    assert!(!request.fixture_root.join("original-turn-credential.json").exists());
    assert!(mirror_files(&request.fixture_root.join("canonical-effects")).is_empty());
    assert!(mirror_files(&request.fixture_root.join("runner-provider-checkpoints")).is_empty());
}

fn assert_canonical_root_processes(label: &str, scenario: CanonicalRootScenario) {
    let mut fixture = Fixture::new(label, scenario.initial_state());
    if scenario.uses_legacy_root() { fixture.rebind_to_legacy_fixed_root(); }
    fixture.canonical_scenario = Some(scenario);
    let mut server = OriginalTurnWireServer::canonical(&fixture.root, scenario);
    let child = fixture.run_child("produce-canonical-root", ReopenExpectation::ProduceCanonicalRoot, Some(&server));
    let server_result = server.finish();
    eprintln!("canonical root scenario={scenario:?} producer_pid={} outcome={server_result:?} {}",
        child.child_pid, server.diagnostics());
    assert!(server_result.is_ok() && child.result.as_ref().is_ok_and(|status| status.success()),
        "canonical root producer failed: child={:?}; Responses worker={server_result:?}\n{}", child.result, child.logs);
    let receipt = child.assert_completed();
    let requests = server.requests.lock().expect("actual canonical provider request capture");
    assert_eq!(requests.len(), scenario.request_count(), "only explicitly admitted invocations may contact the model");
    assert!(requests.iter().all(|request| request["model"] == ORIGINAL_MODEL));
    drop(requests);
    fs::remove_file(&server.credential).expect("remove producer credential before all fresh canonical consumers");
    let produced = receipt.produced.expect("actual canonical producer checkpoint and identity");
    fixture.original = Some(produced.original);
    fixture.expected = produced.checkpoint;
    fixture.canonical_turns = Some(receipt.canonical_turns.expect("producer committed-turn expectations"));
    let markers = SandUpgradeResumeStore::new(&fixture.root).list_pending_checked().expect("actual pending canonical marker");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].original_work, fixture.original);
    fixture.original_marker = Some(markers[0].clone());
    let marker_path = SandUpgradeResumeStore::new(&fixture.root).file_path().to_path_buf();
    let marker_before = fs::read(&marker_path).unwrap();
    let terminal_count: i64 = open_existing(&fixture.db_path(), true).query_row(
        "SELECT COUNT(*) FROM kv WHERE key=?1", params![ORIGINAL_TERMINAL_KEY], |row| row.get(0),
    ).expect("no original terminal receipt was fabricated");
    assert_eq!(terminal_count, 0);
    let wal_path = fixture.root.join("transcripts").join(&fixture.agent_id)
        .join(format!("{}.journal-pending.json", fixture.agent_id));
    assert_eq!(wal_path.exists(), scenario == CanonicalRootScenario::JournalAfterCommit);
    let mut before = durable_state(&fixture);
    for attempt in 1..=2 {
        let receipt = fixture.reopen(&format!("canonical-fresh-consumer-{attempt}"), ReopenExpectation::CanonicalRootReady)
            .assert_completed();
        assert!(receipt.produced.is_none());
        assert!(receipt.canonical_turns.is_none());
        let after = durable_state(&fixture);
        if scenario == CanonicalRootScenario::JournalAfterCommit && attempt == 1 {
            assert_eq!(after.kv, before.kv, "Journal recovery cannot rewrite canonical metadata or completion authority");
            assert_eq!(after.transcript, before.transcript);
            assert_eq!(after.blobs, before.blobs);
            assert_ne!(after.mirror, before.mirror, "the first fresh owner actually consumes the restored prepared WAL");
        } else {
            assert_eq!(after, before, "subsequent consumers cannot normalize, duplicate or replay canonical history");
        }
        assert!(!wal_path.exists());
        assert_eq!(fs::read(&marker_path).unwrap(), marker_before, "no canonical read acknowledges the original marker");
        assert!(mirror_files(&fixture.root.join("canonical-effects")).is_empty());
        assert!(mirror_files(&fixture.root.join("runner-provider-checkpoints")).is_empty());
        before = after;
    }
}

#[test]
fn canonical_root_admission_valid_roots_advance_across_explicit_invocations() {
    for base in [CanonicalBase::Absent, CanonicalBase::Empty, CanonicalBase::Legacy] {
        assert_canonical_root_processes("canonical-valid", CanonicalRootScenario::Valid(base));
    }
}

#[test]
fn canonical_root_admission_pre_projection_drift_prevents_provider_execution() {
    for drift in [CanonicalDrift::AbsentToEmpty, CanonicalDrift::EmptyToAbsent,
        CanonicalDrift::SameBytesOtherRoot, CanonicalDrift::SameRootOtherBytes]
    {
        assert_canonical_root_processes("canonical-pre-projection", CanonicalRootScenario::PreProjection(drift));
    }
}

#[test]
fn canonical_root_admission_post_projection_drift_prevents_provider_execution() {
    assert_canonical_root_processes("canonical-post-projection", CanonicalRootScenario::PostProjection);
}

#[test]
fn canonical_root_admission_strict_read_failures_preserve_durable_state() {
    for fault in [Fault::MissingRootBlob, Fault::MalformedRootBlob, Fault::MissingMetadata,
        Fault::MalformedMetadata, Fault::MissingRootField, Fault::NonStringRootField]
    {
        assert_canonical_root_processes("canonical-strict-read", CanonicalRootScenario::StrictRead(fault));
    }
    assert_canonical_root_processes("canonical-direct-reference", CanonicalRootScenario::MissingDirectTurn);
}

#[test]
fn canonical_root_admission_completed_owner_retirement_prevents_provider_execution() {
    assert_canonical_root_processes("canonical-retired", CanonicalRootScenario::RetiredOwners);
}

#[test]
fn canonical_root_admission_same_sink_staging_and_persistence_keep_captured_base() {
    for scenario in [CanonicalRootScenario::SameSinkBeforeStage, CanonicalRootScenario::SameSinkAfterStage] {
        assert_canonical_root_processes("canonical-same-sink", scenario);
    }
}

#[test]
fn canonical_root_admission_early_errors_permanently_revoke_prior_completion() {
    for kind in [CanonicalEarlyError::Build, CanonicalEarlyError::Prompt, CanonicalEarlyError::Catalog,
        CanonicalEarlyError::Projection, CanonicalEarlyError::Preflight]
    {
        assert_canonical_root_processes("canonical-early-error", CanonicalRootScenario::Early(kind));
    }
}

#[test]
fn canonical_root_admission_post_durable_journal_failure_refuses_stale_sink() {
    assert_canonical_root_processes("canonical-journal-after-commit", CanonicalRootScenario::JournalAfterCommit);
}
