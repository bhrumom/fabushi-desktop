//! ORT-v1 shipping consumer tests. Accepted user input, canonical checkpoint,
//! Journal mirror, terminal capture, effect custody and durable owners are real.
//! The Runner contract owns the generated producer and OS-child harness; its
//! private App child below reopens those files through the shipping consumer.

use super::completed_guard_custody_report;
use super::super::{
    RoutedProviderTerminal, acknowledge_completed_upgrade_marker,
    automatic_ack_redrive_marker_snapshot, visit_upgrade_resume_markers_checked,
};
use mahayana_host_runtime::extensions::inference::provider_session::{ProviderMessage, ProviderSessionError};
use mahayana_host_runtime::extensions::session::agent_db::{OriginalTurnReceiptCommit, OriginalTurnReference, SandAgentDb};
use mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::ProductionAgentStore;
use mahayana_host_runtime::extensions::transcript::ack_obligations::{AckObligations, AckRedrivePreparation, AckRedriveTrigger, MAX_ACK_REDRIVES};
use mahayana_host_runtime::extensions::transcript::production_runtime::{ProductionSendError, ProductionTranscriptRuntime};
use mahayana_host_runtime::extensions::transcript::runner_registry::TranscriptRunnerRegistry;
use mahayana_host_runtime::extensions::transcript::sand_ack_obligation_store::SandAckObligationStore;
use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker;
use mahayana_host_runtime::runner::production_agent_checkpoint::{AgentStateCheckpointSink, ProductionAgentStateCheckpointSink};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedProviderCancellation;
use mahayana_host_runtime::runner::{ToolEffectCustodyReport, TurnRunOptions};
use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
    ConversationTurnStructureFields, decode_conversation_state_recovery_fields,
    decode_conversation_turn_structure_fields,
};
use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection};
use mahayana_host_runtime::transcript_mirror::production_provider::ProductionTranscriptMirrorProvider;
use mahayana_host_runtime::transcript_mirror::transcript_mirror_router::TranscriptMirrorRoute;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const ORIGINAL_STREAM: &str = "original-app-stream";
const ORIGINAL_PROMPT: &str = "Finish the originally accepted user work.";
const ORIGINAL_RESULT: &str = "The originally accepted work is complete.";
const TERMINAL_SLOT: &str = "original-routed-terminal:current";

struct OriginalTurnAppFixture {
    root: PathBuf,
    sessions: Arc<ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    registry: TranscriptRunnerRegistry,
    owner: Arc<SandAgentDb>,
    agent_store: Arc<ProductionAgentStore>,
    sink: Arc<ProductionAgentStateCheckpointSink>,
    original: OriginalTurnReference,
    cancellation: RoutedProviderCancellation,
    lifecycle: Arc<Mutex<Vec<(String, String)>>>,
}

#[derive(Debug, PartialEq)]
struct OriginalTurnBytes {
    root: Vec<u8>,
    checkpoint: Vec<u8>,
    canonical_blobs: BTreeMap<Vec<u8>, Vec<u8>>,
    transcript: Vec<Value>,
    mirrors: BTreeMap<PathBuf, Vec<u8>>,
    acceptance: Vec<u8>,
}

fn read_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !directory.exists() { return; }
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read(path).unwrap());
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

impl OriginalTurnBytes {
    fn read(root_directory: &Path, owner: &SandAgentDb, agent_store: &ProductionAgentStore) -> Self {
        let checkpoint = agent_store.checkpoint_snapshot_checked().unwrap().unwrap();
        let root = agent_store.latest_root_blob_id_checked().unwrap();
        let mut canonical_blobs = BTreeMap::from([(root.clone(), checkpoint.checkpoint_bytes.clone())]);
        for turn_id in decode_conversation_state_recovery_fields(&checkpoint.checkpoint_bytes).unwrap().turns {
            let turn = agent_store.get_blob(&turn_id).unwrap().unwrap();
            let Some(ConversationTurnStructureFields::Agent { user_message, steps, .. }) =
                decode_conversation_turn_structure_fields(&turn).unwrap() else {
                    panic!("fixture must contain the actual generated Agent turn");
                };
            for id in std::iter::once(user_message).chain(steps) {
                canonical_blobs.insert(id.clone(), agent_store.get_blob(&id).unwrap().unwrap());
            }
            canonical_blobs.insert(turn_id, turn);
        }
        Self {
            root,
            checkpoint: checkpoint.checkpoint_bytes,
            canonical_blobs,
            transcript: owner.get_transcript_entries().unwrap(),
            mirrors: read_files(&root_directory.join("transcripts")),
            acceptance: fs::read(root_directory.join("send-acceptance.json")).unwrap(),
        }
    }
}

impl OriginalTurnAppFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-original-turn-app-{label}-{}", uuid::Uuid::new_v4(),
        ));
        fs::create_dir_all(&root).unwrap();
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = sessions.materialize_session_with_active(None, "user", None, None).unwrap();
        let agent_id = session.record.id.clone();
        let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
        let lifecycle = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&lifecycle);
        runtime.set_agent_run_lifecycle_observer(Some(Arc::new(move |kind, request| {
            observed.lock().unwrap().push((kind.into(), request.into()));
        })));
        let args = json!({
            "agentId":agent_id, "sessionKey":format!("agent:{agent_id}:main"),
            "prompt":ORIGINAL_PROMPT, "streamId":ORIGINAL_STREAM, "clientNonce":"original-app-nonce",
        });
        let accepted = runtime.accept_routed_send(&args, |accepted| {
            persist_accepted_send_prompt_context(&sessions, &args, accepted)
                .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
        }).unwrap();
        assert!(accepted.context.accepted_durably);
        let admission = sessions.read_session_work_admission(&agent_id).unwrap().unwrap();
        let original = runtime.bind_original_turn_model(
            &agent_id, ORIGINAL_STREAM, &admission, "original-native-model",
        ).unwrap().unwrap();
        assert_eq!(original.user_message_id, accepted.context.user_message_id.unwrap());
        let registry = TranscriptRunnerRegistry::default();
        let cancellation = registry.register_routed_provider(&agent_id, ORIGINAL_STREAM).unwrap();
        let blobs = Arc::new(sessions.create_agent_blob_store(&agent_id).unwrap());
        let prior = session.agent_store.checkpoint_snapshot_checked().unwrap()
            .map(|snapshot| snapshot.checkpoint_bytes).unwrap_or_default();
        let provider = ProductionTranscriptMirrorProvider::new(
            root.join("transcripts"), GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection),
        );
        let mirror = Arc::new(provider.route_for_session(Arc::clone(&blobs), &prior, Arc::new(|| Ok(true))).unwrap());
        assert_eq!(mirror.route(&agent_id).unwrap(), TranscriptMirrorRoute::Journal);
        let sink = Arc::new(ProductionAgentStateCheckpointSink::new(
            agent_id, Arc::clone(&session.agent_store), blobs, mirror, prior, true,
        ).unwrap());
        sink.checkpoint_text_turn(
            &[ProviderMessage { role: "user".into(), content: ORIGINAL_PROMPT.into() }],
            &TurnRunOptions {
                inference_request_id: Some(ORIGINAL_STREAM.into()),
                message_id: Some(original.user_message_id.clone()),
                recent_message_text: Some(ORIGINAL_PROMPT.into()),
                ..TurnRunOptions::default()
            }, ORIGINAL_RESULT,
        ).unwrap();
        assert_eq!(sink.committed_original_turn_observations().unwrap().len(), 1);
        assert_eq!(session.db.get_transcript_entry_by_id(&original.user_message_id).unwrap().unwrap()["confirmed"], true);
        let journal = root.join("transcripts").join(&original.agent_id).join(format!("{}.jsonl", original.agent_id));
        let mirrored = fs::read_to_string(journal).unwrap();
        assert!(mirrored.contains(ORIGINAL_PROMPT));
        assert!(mirrored.contains(ORIGINAL_RESULT));
        Self {
            root, sessions, runtime, registry, owner: Arc::clone(&session.db),
            agent_store: Arc::clone(&session.agent_store), sink, original, cancellation, lifecycle,
        }
    }

    fn terminal(&self) -> RoutedProviderTerminal {
        RoutedProviderTerminal::capture(
            ORIGINAL_STREAM, false, &self.cancellation, Ok(ORIGINAL_RESULT.into()), 1, false,
            completed_guard_custody_report(ORIGINAL_STREAM),
        )
    }

    fn mark_and_retire(&self) -> UpgradeResumeMarker {
        self.runtime.quiesce_for_upgrade().unwrap();
        let marker = self.runtime.upgrade_resume_store().unwrap().list_pending_checked().unwrap().pop().unwrap();
        assert_eq!(marker.original_work.as_ref(), Some(&self.original));
        self.registry.request_quiesce_for_upgrade();
        assert!(self.cancellation.is_cancelled());
        self.registry.finish_routed_provider(ORIGINAL_STREAM);
        self.runtime.settle_routed_turn(&self.original.agent_id, ORIGINAL_STREAM, 10_000).unwrap().unwrap();
        self.runtime.resume_after_recreate();
        self.registry.cancel_quiesce_for_upgrade();
        marker
    }

    fn acknowledge(&self, ack: &SandAckObligationStore, marker: &UpgradeResumeMarker) -> Result<bool, String> {
        acknowledge_completed_upgrade_marker(
            &self.sessions, &self.runtime, &self.registry, ack, marker,
        )
    }

    fn bytes(&self) -> OriginalTurnBytes {
        OriginalTurnBytes::read(&self.root, &self.owner, &self.agent_store)
    }

    fn receipt(&self) -> Option<String> {
        self.owner.read_kv(TERMINAL_SLOT).unwrap()
    }

    fn close(self) {
        self.sessions.shutdown();
        let root = self.root.clone();
        drop(self);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn original_turn_completed_marker_acknowledges_frozen_terminal_without_new_work() {
    let fixture = OriginalTurnAppFixture::new("completed");
    let metadata = fixture.owner.read_metadata().unwrap();
    let admission = fixture.sessions.read_session_work_admission(&fixture.original.agent_id).unwrap().unwrap();
    assert!(metadata["lifecycleRevision"].as_str().is_some());
    assert_ne!(metadata["lifecycleRevision"].as_str(), Some(admission.lifecycle_revision.as_str()),
        "a real canonical checkpoint rotates metadata independently of admission");
    let terminal = fixture.terminal();
    assert_eq!(terminal.event["payload"]["type"], "completed");
    assert!(!fixture.runtime.upgrade_resume_store().unwrap().file_path().exists());
    assert_eq!(terminal.persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap(), OriginalTurnReceiptCommit::Inserted);
    let receipt = fixture.receipt().unwrap();
    let before = fixture.bytes();
    let marker = fixture.mark_and_retire();
    // Cancellation after the last result-changing boundary cannot rewrite the
    // captured Completed facts or make this exact receipt repeat effects.
    assert_eq!(terminal.event["payload"]["type"], "completed");
    assert_eq!(terminal.persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap(), OriginalTurnReceiptCommit::AlreadyCommitted);
    assert_eq!(fixture.receipt().as_deref(), Some(receipt.as_str()));
    assert_eq!(fixture.bytes(), before, "repeat finalization cannot rewrite the committed canonical or Journal bytes");
    let lifecycle = fixture.lifecycle.lock().unwrap().clone();
    let epoch = fixture.runtime.current_turn_epoch(&fixture.original.agent_id);
    let ack = SandAckObligationStore::new(&fixture.root);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(true));
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false));
    assert!(fixture.runtime.upgrade_resume_store().unwrap().list_pending_checked().unwrap().is_empty());
    assert_eq!(fixture.bytes(), before, "acknowledgment cannot append user/result/mirror data or mint a new nonce");
    assert_eq!(fixture.receipt().as_deref(), Some(receipt.as_str()));
    assert_eq!(*fixture.lifecycle.lock().unwrap(), lifecycle);
    assert_eq!(fixture.runtime.current_turn_epoch(&fixture.original.agent_id), epoch);
    assert!(!fixture.runtime.is_agent_running(&fixture.original.agent_id));
    assert!(fixture.registry.active_stream_ids_for_agent(&fixture.original.agent_id).is_empty());
    assert!(!ack.file_path().exists(), "delivery counters cannot fulfill or fabricate independent ACK custody");
    fixture.close();
}

#[test]
fn original_turn_completed_marker_retains_fresh_pending_corrupt_ack_and_live_work() {
    let fixture = OriginalTurnAppFixture::new("ack-and-live");
    fixture.terminal().persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap();
    let marker = fixture.mark_and_retire();
    let marker_bytes = fs::read(fixture.runtime.upgrade_resume_store().unwrap().file_path()).unwrap();
    let before = fixture.bytes();
    let ack = SandAckObligationStore::new(&fixture.root);
    assert_eq!(ack.get(&fixture.original.agent_id), None, "prime lenient absence before another writer publishes");
    let writer = SandAckObligationStore::new(&fixture.root);
    writer.record_send(&fixture.original.agent_id, 1.0).unwrap();
    let ack_bytes = fs::read(ack.file_path()).unwrap();
    assert_eq!(ack.get(&fixture.original.agent_id), None);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false));
    assert_eq!(fs::read(ack.file_path()).unwrap(), ack_bytes);
    fs::write(ack.file_path(), b"unreadable ACK custody").unwrap();
    assert!(fixture.acknowledge(&ack, &marker).is_err());
    assert_eq!(fs::read(ack.file_path()).unwrap(), b"unreadable ACK custody");
    fs::remove_file(ack.file_path()).unwrap();
    fixture.runtime.begin_provider_run(&fixture.original.agent_id);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false));
    fixture.runtime.end_provider_run(&fixture.original.agent_id);
    fixture.registry.register_routed_provider(&fixture.original.agent_id, "new-foreground").unwrap();
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false));
    fixture.registry.finish_routed_provider("new-foreground");
    assert_eq!(fs::read(fixture.runtime.upgrade_resume_store().unwrap().file_path()).unwrap(), marker_bytes);
    assert_eq!(fixture.bytes(), before);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(true), "fresh absence and retired live work restore the qualified path");
    fixture.close();
}

#[test]
fn original_turn_marker_retains_unbound_missing_and_foreign_original_references() {
    let fixture = OriginalTurnAppFixture::new("reference-retention");
    let terminal = fixture.terminal();
    let marker = fixture.mark_and_retire();
    let ack = SandAckObligationStore::new(&fixture.root);
    let store = fixture.runtime.upgrade_resume_store().unwrap();
    let marker_bytes = fs::read(store.file_path()).unwrap();
    let before = fixture.bytes();
    assert_eq!(fixture.receipt(), None);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false), "canonical output alone is not a completed receipt");
    assert_eq!(fs::read(store.file_path()).unwrap(), marker_bytes);
    terminal.persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap();
    let foreign_agent = fixture.sessions.materialize_new_session(None, "user", None).unwrap().id;
    let mut foreign_original = fixture.original.clone();
    foreign_original.agent_id = foreign_agent.clone();
    foreign_original.session_id = foreign_agent.clone();
    foreign_original.session_key = format!("agent:{foreign_agent}:main");
    foreign_original.original_lifecycle_revision = fixture.sessions.read_session_work_admission(&foreign_agent)
        .unwrap().unwrap().lifecycle_revision;
    let foreign_marker = UpgradeResumeMarker {
        agent_id: foreign_agent, original_work: Some(foreign_original), ..marker.clone()
    };
    store.mark_pending_checked(foreign_marker.clone()).unwrap();
    let foreign_bytes = fs::read(store.file_path()).unwrap();
    assert_ne!(fixture.acknowledge(&ack, &foreign_marker), Ok(true), "another Session cannot borrow the original receipt");
    assert_eq!(fs::read(store.file_path()).unwrap(), foreign_bytes);
    assert!(store.clear_if_matches_checked(&foreign_marker).unwrap());
    let mut latest = marker.clone();
    for field in ["unbound", "modelId", "inputDigest", "streamId", "clientNonce", "userMessageId", "originalLifecycleRevision"] {
        let mut value = serde_json::to_value(&marker).unwrap();
        value["markedAtMs"] = (latest.marked_at_ms + 1.0).into();
        if field == "unbound" {
            value.as_object_mut().unwrap().remove("originalWork");
        } else if field == "inputDigest" {
            value["originalWork"][field] = "f".repeat(64).into();
        } else {
            value["originalWork"][field] = format!("foreign-{field}").into();
            if field == "userMessageId" {
                value["originalWork"]["echoEntryId"] = "foreign-userMessageId".into();
            }
        }
        latest = serde_json::from_value(value).unwrap();
        store.mark_pending_checked(latest.clone()).unwrap();
        let frozen = fs::read(store.file_path()).unwrap();
        assert_ne!(fixture.acknowledge(&ack, &latest), Ok(true), "{field}");
        assert_eq!(fs::read(store.file_path()).unwrap(), frozen, "{field}");
        assert_eq!(fixture.bytes(), before, "{field}");
    }
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false), "an old exact-clear cannot consume the replacement marker");
    latest.original_work = Some(fixture.original.clone());
    latest.marked_at_ms += 1.0;
    store.mark_pending_checked(latest.clone()).unwrap();
    assert_eq!(fixture.acknowledge(&ack, &latest), Ok(true));
    fixture.close();
}

fn unknown_guard_custody(stream_id: &str) -> ToolEffectCustodyReport {
    use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderAttemptControl;
    use mahayana_host_runtime::runner::tools::mcp_meta_tools::execute_routed_tool_with_timeout_ms;
    use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime, ToolEffectCustodyObservation};
    let runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy::default()));
    let generation = runtime.begin_attempt();
    let attempt = RoutedProviderAttemptControl::new(Arc::clone(&runtime), generation, RoutedProviderCancellation::default());
    let result = execute_routed_tool_with_timeout_ms(&attempt, "unknown-original-effect".into(), 5_000, || {
        Err(ProviderSessionError::ToolOutcomeUnknown("controlled downstream uncertainty".into()))
    });
    assert!(matches!(result, Err(ProviderSessionError::ToolOutcomeUnknown(_))));
    let observation = ToolEffectCustodyObservation::Known {
        run_generation: generation.value(), snapshot: runtime.effect_custody(),
    };
    assert!(observation.requires_retention());
    let mut report = ToolEffectCustodyReport::new(stream_id);
    report.record(Some(&observation));
    report
}

#[test]
fn original_turn_finalizer_refuses_noncompleted_unknown_and_result_drift_without_receipt() {
    let fixture = OriginalTurnAppFixture::new("terminal-refusal");
    let before = fixture.bytes();
    let unknown = unknown_guard_custody(ORIGINAL_STREAM);
    for (label, waiting, result, report) in [
        ("waiting-user", true, Ok(ORIGINAL_RESULT.into()), completed_guard_custody_report(ORIGINAL_STREAM)),
        ("failed", false, Err(ProviderSessionError::Tool("known failure".into())), completed_guard_custody_report(ORIGINAL_STREAM)),
        ("unavailable", false, Ok(ORIGINAL_RESULT.into()), ToolEffectCustodyReport::new(ORIGINAL_STREAM)),
        ("foreign-custody", false, Ok(ORIGINAL_RESULT.into()), completed_guard_custody_report("foreign-stream")),
        ("unknown-custody", false, Ok(ORIGINAL_RESULT.into()), unknown),
        ("different-final-result", false, Ok("not the committed result".into()), completed_guard_custody_report(ORIGINAL_STREAM)),
    ] {
        let terminal = RoutedProviderTerminal::capture(ORIGINAL_STREAM, waiting, &fixture.cancellation, result, 1, false, report);
        assert!(terminal.persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).is_err(), "{label}");
        assert_eq!(fixture.receipt(), None, "{label}");
        assert_eq!(fixture.bytes(), before, "{label}");
    }
    assert!(fixture.registry.cancel_stream(ORIGINAL_STREAM, "actual local cancellation"));
    let cancelled = fixture.terminal();
    assert_eq!(cancelled.event["payload"]["type"], "cancelled");
    assert!(cancelled.persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).is_err());
    let marker = fixture.mark_and_retire();
    let ack = SandAckObligationStore::new(&fixture.root);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(false));
    assert_eq!(fixture.receipt(), None);
    assert_eq!(fixture.bytes(), before);
    fixture.close();
}

#[test]
fn original_turn_marker_requalifies_actual_saved_custody_before_acknowledgment() {
    let fixture = OriginalTurnAppFixture::new("replay-custody");
    fixture.terminal().persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap();
    let original_receipt = fixture.receipt().unwrap();
    let marker = fixture.mark_and_retire();
    let before = fixture.bytes();
    let marker_bytes = fs::read(fixture.runtime.upgrade_resume_store().unwrap().file_path()).unwrap();
    // Fault injection changes an actual production-written receipt; it never
    // constructs a settled JSON fixture that could stand in for a real commit.
    let mut damaged: Value = serde_json::from_str(&original_receipt).unwrap();
    damaged["custodyJson"] = serde_json::to_string(&unknown_guard_custody(ORIGINAL_STREAM)).unwrap().into();
    let connection = rusqlite::Connection::open(fixture.sessions.session_db_path(&fixture.original.agent_id).unwrap()).unwrap();
    connection.execute("UPDATE kv SET value = ?1 WHERE key = ?2", rusqlite::params![damaged.to_string(), TERMINAL_SLOT]).unwrap();
    let ack = SandAckObligationStore::new(&fixture.root);
    assert!(fixture.acknowledge(&ack, &marker).is_err());
    assert_eq!(fs::read(fixture.runtime.upgrade_resume_store().unwrap().file_path()).unwrap(), marker_bytes);
    assert_eq!(fixture.bytes(), before);
    connection.execute("UPDATE kv SET value = ?1 WHERE key = ?2", rusqlite::params![original_receipt, TERMINAL_SLOT]).unwrap();
    drop(connection);
    assert_eq!(fixture.acknowledge(&ack, &marker), Ok(true));
    fixture.close();
}

#[test]
fn original_turn_ack_gate_retains_due_timer_attempt_and_bytes_until_marker_releases() {
    let fixture = OriginalTurnAppFixture::new("ack-timer");
    let marker = fixture.mark_and_retire();
    let obligations = AckObligations::new(&fixture.root);
    obligations.store().record_send(&fixture.original.agent_id, 1.0).unwrap();
    assert!(obligations.arm_redrive_timer(&fixture.original.agent_id, AckRedriveTrigger::Boot, 0));
    let schedule = obligations.redrive_schedule(&fixture.original.agent_id).unwrap();
    let bytes = fs::read(obligations.store().file_path()).unwrap();
    let marker_store = fixture.runtime.upgrade_resume_store().unwrap();
    let marker_bytes = fs::read(marker_store.file_path()).unwrap();
    assert!(marker_store.clear_if_matches_checked(&marker).unwrap());
    assert_eq!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains(&fixture.original.agent_id)), Ok(false));
    // A retained marker can be published after the worker's cheap preliminary
    // check. Only the real atomic owner may take the due timer or mutate ACK.
    marker_store.mark_pending_checked(marker.clone()).unwrap();
    assert_eq!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ), Ok(None));
    assert_eq!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains(&fixture.original.agent_id)), Ok(true));
    assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), Some(schedule));
    assert_eq!(obligations.store().get_checked(&fixture.original.agent_id).unwrap().unwrap().redrive_attempts, 0.0);
    assert_eq!(fs::read(obligations.store().file_path()).unwrap(), bytes);
    fs::write(marker_store.file_path(), b"unknown complete marker image").unwrap();
    assert!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains(&fixture.original.agent_id)).is_err());
    assert!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains("unrelated-agent")).is_err());
    assert!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ).is_err());
    assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), Some(schedule));
    assert_eq!(fs::read(obligations.store().file_path()).unwrap(), bytes);
    fs::write(marker_store.file_path(), &marker_bytes).unwrap();
    assert_eq!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains("unrelated-agent")), Ok(false));
    let mut maxed = obligations.store().get_checked(&fixture.original.agent_id).unwrap().unwrap();
    maxed.redrive_attempts = MAX_ACK_REDRIVES as f64;
    obligations.store().upsert(maxed).unwrap();
    let max_bytes = fs::read(obligations.store().file_path()).unwrap();
    for agent_exists in [true, false] {
        assert_eq!(fixture.runtime.prepare_automatic_ack_redrive(
            &obligations, &fixture.original.agent_id, agent_exists, schedule.due_at_ms,
        ), Ok(None), "retention precedes both max-attempt and deleted-agent ACK clearing");
        assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), Some(schedule));
        assert_eq!(fs::read(obligations.store().file_path()).unwrap(), max_bytes);
    }
    assert!(marker_store.clear_if_matches_checked(&marker).unwrap());
    assert_eq!(automatic_ack_redrive_marker_snapshot(&fixture.runtime)
        .map(|ids| ids.contains(&fixture.original.agent_id)), Ok(false));
    fs::write(obligations.store().file_path(), b"unknown current ACK image").unwrap();
    assert!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ).is_err());
    assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), Some(schedule));
    assert_eq!(fs::read(obligations.store().file_path()).unwrap(), b"unknown current ACK image");
    fs::write(obligations.store().file_path(), &bytes).unwrap();
    fixture.runtime.quiesce_for_upgrade().unwrap();
    assert_eq!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ), Ok(None));
    assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), Some(schedule));
    assert_eq!(fs::read(obligations.store().file_path()).unwrap(), bytes);
    fixture.runtime.resume_after_recreate();
    assert!(matches!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ).unwrap(), Some((AckRedriveTrigger::Boot, AckRedrivePreparation::Ready(_)))));
    assert_eq!(obligations.store().get_checked(&fixture.original.agent_id).unwrap().unwrap().redrive_attempts, 1.0);
    assert_eq!(obligations.redrive_schedule(&fixture.original.agent_id), None);
    let committed = fs::read(obligations.store().file_path()).unwrap();
    assert_eq!(fixture.runtime.prepare_automatic_ack_redrive(
        &obligations, &fixture.original.agent_id, true, schedule.due_at_ms,
    ), Ok(None));
    assert_eq!(fs::read(obligations.store().file_path()).unwrap(), committed);
    obligations.dispose();
    fixture.close();
}

#[test]
fn original_turn_checked_marker_migration_calls_no_consumer_for_invalid_whole_image() {
    let fixture = OriginalTurnAppFixture::new("migration-consumer");
    fixture.terminal().persist_original_turn(&fixture.sessions, &fixture.original, fixture.sink.as_ref()).unwrap();
    let mut legacy_marker = fixture.mark_and_retire();
    legacy_marker.original_work = None;
    let before = fixture.bytes();
    let store = fixture.runtime.upgrade_resume_store().unwrap();
    for bytes in [
        br#"{"version":1,"pending":[{"agentId":"valid","markedAtMs":1,"source":"turn"},{"agentId":"invalid","markedAtMs":"bad"}]}"#.as_slice(),
        br#"{"version":99,"pending":[]}"#.as_slice(),
    ] {
        fs::write(store.file_path(), bytes).unwrap();
        let mut visited = 0;
        assert!(visit_upgrade_resume_markers_checked(store, |_| { visited += 1; Ok(true) }).is_err());
        assert_eq!(visited, 0);
        assert_eq!(fs::read(store.file_path()).unwrap(), bytes);
    }
    fs::write(store.file_path(), serde_json::to_vec(&json!({"version":1,"pending":[legacy_marker]})).unwrap()).unwrap();
    let ack = SandAckObligationStore::new(&fixture.root);
    let mut visited = 0;
    assert_eq!(visit_upgrade_resume_markers_checked(store, |marker| {
        visited += 1;
        assert_eq!(marker.agent_id, fixture.original.agent_id);
        assert!(marker.original_work.is_none());
        let image: Value = serde_json::from_slice(&fs::read(store.file_path()).unwrap()).unwrap();
        assert_eq!(image["version"], 2, "complete migration precedes the first callback");
        fixture.acknowledge(&ack, &marker)
    }), Ok(0));
    assert_eq!(visited, 1);
    assert_eq!(store.list_pending_checked().unwrap(), vec![legacy_marker]);
    assert_eq!(fixture.bytes(), before, "legacy migration cannot claim an otherwise real completed receipt");
    fixture.close();
}

#[test]
#[ignore = "private shipping App child; launched by the generated Runner contract's owned-process harness"]
fn original_turn_shipping_ack_process_child() {
    use std::io::Write;

    // This is the existing Runner harness's test DTO. Extra producer/checkpoint
    // fields stay with that harness; this child never creates completion proof.
    struct ChildRequest {
        token: String,
        parent_pid: u32,
        fixture_root: PathBuf,
        agent_id: String,
        expectation: String,
        original: Option<OriginalTurnReference>,
        original_marker: Option<UpgradeResumeMarker>,
    }

    fn optional_bytes(path: &Path) -> Option<Vec<u8>> {
        match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => panic!("read existing custody {}: {error}", path.display()),
        }
    }

    let request_path = PathBuf::from(std::env::var_os("FABUSHI_CHECKPOINT_RESTART_CONTRACT_REQUEST")
        .expect("private checkpoint child request environment"));
    let wire: Value = serde_json::from_slice(&fs::read(&request_path).expect("child request"))
        .expect("decode the existing Runner child request");
    let request = ChildRequest {
        token: wire["token"].as_str().expect("launch token").into(),
        parent_pid: u32::try_from(wire["parent_pid"].as_u64().expect("parent PID")).expect("u32 parent PID"),
        fixture_root: PathBuf::from(wire["fixture_root"].as_str().expect("fixture root")),
        agent_id: wire["agent_id"].as_str().expect("original agent").into(),
        expectation: wire["expectation"].as_str().expect("child expectation").into(),
        original: serde_json::from_value(wire["original"].clone()).expect("producer's original identity"),
        original_marker: serde_json::from_value(wire["original_marker"].clone()).expect("saved producer marker"),
    };
    assert!(!request.token.trim().is_empty(), "receipt must identify this launch");
    assert_ne!(request.parent_pid, 0);
    assert_ne!(request.parent_pid, std::process::id(), "fresh OS process required");
    assert!(request.fixture_root.is_absolute() && request.fixture_root.is_dir());
    assert!(request_path.is_absolute() && request_path.starts_with(&request.fixture_root));
    let receipt_path = request_path.parent().expect("private run directory").join("completed.json");
    assert!(!receipt_path.exists(), "completion must be fresh for this child");
    assert!(matches!(request.expectation.as_str(),
        "OriginalMarkerAck" | "OriginalMarkerNoop" | "OriginalMarkerPendingAck" | "OriginalMarkerCorruptAck"));
    let original = request.original.as_ref().expect("actual generated producer's original reference");
    original.validate().expect("full original identity is well formed");
    assert_eq!(original.agent_id, request.agent_id);
    let saved_marker = request.original_marker.as_ref().expect("parent's actual saved v2 marker");
    assert_eq!(saved_marker.agent_id, request.agent_id);
    assert_eq!(saved_marker.original_work.as_ref(), Some(original));
    assert_ne!(std::env::var("FABUSHI_RESPONSES_MODEL").expect("changed current model"), original.model_id);
    assert_eq!(std::env::var("FABUSHI_RESPONSES_URL").expect("unavailable provider"), "http://127.0.0.1:9/unused");
    assert!(!PathBuf::from(std::env::var_os("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE")
        .expect("unavailable credential path")).exists());

    let sessions = ProductionSessionWorkers::with_agents_root(request.fixture_root.join("agents"), 500);
    let runtime = ProductionTranscriptRuntime::new(Some(&request.fixture_root));
    let lifecycle = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let observed = Arc::clone(&lifecycle);
    runtime.set_agent_run_lifecycle_observer(Some(Arc::new(move |kind, stream| {
        observed.lock().unwrap().push((kind.into(), stream.into()));
    })));
    let registry = TranscriptRunnerRegistry::default();
    let ack = SandAckObligationStore::new(&request.fixture_root);
    let store = runtime.upgrade_resume_store().expect("fresh production marker owner");
    let marker_before = optional_bytes(store.file_path());
    let mut pending = store.list_pending_checked().expect("strict current marker image");
    let marker = if request.expectation == "OriginalMarkerNoop" {
        assert!(pending.is_empty() && marker_before.is_none(), "prior App child consumed the exact marker");
        saved_marker.clone()
    } else {
        let image: Value = serde_json::from_slice(marker_before.as_deref().expect("producer's stored marker"))
            .expect("stored marker JSON");
        assert_eq!(image["version"], 2);
        assert_eq!(pending, vec![saved_marker.clone()], "use exactly the producer's whole marker");
        pending.pop().unwrap()
    };
    let ack_before = optional_bytes(ack.file_path());
    match request.expectation.as_str() {
        "OriginalMarkerPendingAck" => assert!(ack.get_checked(&request.agent_id).unwrap().is_some()),
        "OriginalMarkerCorruptAck" => assert!(ack.get_checked(&request.agent_id).is_err()),
        _ => assert!(ack.get_checked(&request.agent_id).unwrap().is_none()),
    }
    let owner = sessions.open_agent_db_owner(&request.agent_id).expect("fresh original Session owner");
    let agent_store = sessions.open_agent_store_owner(&request.agent_id).expect("fresh original canonical owner");
    let terminal_before = owner.read_kv(TERMINAL_SLOT).unwrap().expect("actual producer-persisted terminal slot");
    let terminal: Value = serde_json::from_str(&terminal_before).expect("actual stored terminal JSON");
    assert_eq!(terminal["original"], serde_json::to_value(original).unwrap());
    let before = OriginalTurnBytes::read(&request.fixture_root, &owner, &agent_store);
    assert!(!before.mirrors.is_empty(), "producer committed the actual Journal mirror");
    let effect_path = request.fixture_root.join("original-effect.txt");
    let effect_before = fs::read(&effect_path).expect("actual generated tool-effect receipt");
    assert!(!effect_before.is_empty());
    let epoch = runtime.current_turn_epoch(&request.agent_id);
    assert_eq!(registry.active_count(), 0);
    assert!(!runtime.is_agent_running(&request.agent_id));

    for attempt in 0..2 {
        let result = acknowledge_completed_upgrade_marker(&sessions, &runtime, &registry, &ack, &marker);
        match request.expectation.as_str() {
            "OriginalMarkerAck" if attempt == 0 => assert_eq!(result, Ok(true)),
            "OriginalMarkerCorruptAck" => assert!(result.is_err(), "corrupt independent ACK must retain custody"),
            _ => assert_eq!(result, Ok(false)),
        }
        if matches!(request.expectation.as_str(), "OriginalMarkerAck" | "OriginalMarkerNoop") {
            assert!(store.list_pending_checked().unwrap().is_empty());
            assert!(optional_bytes(store.file_path()).is_none(), "exact removal and repeated no-op");
        } else {
            assert_eq!(store.list_pending_checked().unwrap(), vec![marker.clone()]);
            assert_eq!(optional_bytes(store.file_path()), marker_before, "refusal preserves the exact marker bytes");
        }
        assert_eq!(optional_bytes(ack.file_path()), ack_before, "independent ACK custody is unchanged");
        assert_eq!(OriginalTurnBytes::read(&request.fixture_root, &owner, &agent_store), before,
            "acknowledgment cannot rewrite canonical, user/result, mirror or accepted-input bytes");
        assert_eq!(owner.read_kv(TERMINAL_SLOT).unwrap().as_deref(), Some(terminal_before.as_str()));
        assert_eq!(fs::read(&effect_path).unwrap(), effect_before, "no tool-effect replay");
        assert!(lifecycle.lock().unwrap().is_empty(), "acknowledgment cannot dispatch a turn");
        assert_eq!(runtime.current_turn_epoch(&request.agent_id), epoch);
        assert!(!runtime.is_agent_running(&request.agent_id));
        assert_eq!(registry.active_count(), 0);
        assert!(registry.active_stream_ids_for_agent(&request.agent_id).is_empty());
    }

    runtime.dispose();
    drop((agent_store, owner));
    sessions.shutdown_with_checkpoint();
    assert_eq!(sessions.active_worker_count(), 0, "all fresh Session workers closed");
    drop((ack, registry, runtime, sessions));
    // The existing parent also compares all raw KV, transcript and blob rows
    // across this closed-process boundary; no second App launch harness exists.
    let receipt = json!({
        "token":request.token, "pid":std::process::id(), "parent_pid":request.parent_pid,
        "expectation":request.expectation, "produced":null,
    });
    let mut completed = fs::OpenOptions::new().write(true).create_new(true).open(&receipt_path)
        .expect("create fresh post-shutdown completion receipt");
    completed.write_all(&serde_json::to_vec(&receipt).unwrap()).expect("write child completion");
    completed.sync_all().expect("persist child completion after all assertions and owner closure");
}
