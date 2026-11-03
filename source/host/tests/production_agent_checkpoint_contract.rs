use std::fs;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::session::production_agent_store::{
    ProductionAgentStore, ProductionWorkerBlobStore,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, ProductionAgentStateCheckpointSink,
    build_generated_turn_checkpoint_with_rich_text, build_text_turn_checkpoint,
    build_text_turn_checkpoint_with_rich_text, prepare_settled_checkpoint_state,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::ProductionTurnAgentOwner;
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::turn_agent_composition::TurnAgentComposition;
use mahayana_host_runtime::runner::{TerminalOutcome, TokenDetailsPersistenceTracker, TurnRunOptions};
use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
    decode_conversation_state_recovery_fields, decode_transcript_mirror_conversation_state,
};
use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
};
use mahayana_host_runtime::transcript_mirror::production_provider::{
    ProductionTranscriptMirrorProvider,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-production-agent-checkpoint-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn user_messages() -> Vec<ProviderMessage> {
    vec![ProviderMessage {
        role: "user".into(),
        content: "hello checkpoint".into(),
    }]
}

#[test]
fn canonical_text_turn_checkpoint_preserves_prior_wire_and_appends_turn_reference() {
    let prior = vec![0x9a, 0x06, 0x03, b'o', b'l', b'd'];
    let checkpoint = build_text_turn_checkpoint(
        &prior,
        "hello",
        "message-1",
        Some("request-1"),
        "world",
    );
    assert_eq!(&checkpoint.state_bytes[..prior.len()], prior.as_slice());
    let decoded = decode_transcript_mirror_conversation_state(&checkpoint.state_bytes)
        .expect("decode state");
    assert_eq!(decoded.turns, vec![checkpoint.turn_id.clone()]);
    assert!(!checkpoint.user_message_id.is_empty());
    assert_eq!(checkpoint.step_ids.len(), 1);

    let rich = build_text_turn_checkpoint_with_rich_text(
        &prior,
        "hello",
        Some("{\"type\":\"doc\",\"content\":[]}"),
        "message-rich",
        Some("request-rich"),
        "world",
    );
    let rich_wire = b"{\"type\":\"doc\",\"content\":[]}";
    assert!(
        rich.user_message_bytes
            .windows(rich_wire.len())
            .any(|window| window == rich_wire)
    );
    assert_ne!(rich.user_message_id, checkpoint.user_message_id);
}

#[test]
fn generated_turn_checkpoint_hashes_and_references_tool_steps_before_assistant_text() {
    let tool_step = vec![0x12, 0x02, 0x0a, 0x00];
    let checkpoint = build_generated_turn_checkpoint_with_rich_text(
        &[],
        "use a tool",
        None,
        "message-tool",
        Some("request-tool"),
        std::slice::from_ref(&tool_step),
        "done",
    );
    assert_eq!(checkpoint.step_bytes.len(), 2);
    assert_eq!(checkpoint.step_bytes[0], tool_step);
    assert_eq!(
        checkpoint.step_ids[0],
        Sha256::digest(&checkpoint.step_bytes[0]).to_vec()
    );
    assert_eq!(
        checkpoint.step_ids[1],
        Sha256::digest(&checkpoint.step_bytes[1]).to_vec()
    );

    // AgentConversationTurnStructure field 2 contains the ordered blob ids.
    let first = checkpoint
        .turn_bytes
        .windows(checkpoint.step_ids[0].len())
        .position(|window| window == checkpoint.step_ids[0].as_slice())
        .expect("tool step id is referenced");
    let second = checkpoint
        .turn_bytes
        .windows(checkpoint.step_ids[1].len())
        .position(|window| window == checkpoint.step_ids[1].as_slice())
        .expect("assistant step id is referenced");
    assert!(first < second, "tool step must precede terminal assistant text");
}

#[test]
fn production_sink_commits_real_agent_wire_through_mirror_and_agent_store() {
    let root = temp_root("production");
    let agents_root = root.join("agents");
    let transcripts_dir = root.join("transcripts");
    let sessions = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let session = sessions
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    let blob_store = Arc::new(
        sessions
            .create_agent_blob_store(&session.record.id)
            .expect("blob store"),
    );
    session
        .db
        .append_transcript_entry(&serde_json::json!({
            "id": "message-1",
            "kind": "message",
            "role": "user",
            "content": "hello checkpoint",
            "confirmed": false
        }))
        .expect("durable user echo");
    let prior_snapshot = session.agent_store.checkpoint_snapshot_checked().expect("checked prior snapshot");
    let prior = prior_snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone()).unwrap_or_default();
    let provider = ProductionTranscriptMirrorProvider::new(
        &transcripts_dir,
        GeneratedTranscriptOccurrenceCodec::new(
            RejectGeneratedToolJsonProjection,
        ),
    );
    let mirror = Arc::new(
        provider
            .route_for_session(
                Arc::clone(&blob_store),
                &prior,
                Arc::new(|| Ok(true)),
            )
            .expect("route"),
    );
    let sink = ProductionAgentStateCheckpointSink::new(
        session.record.id.clone(),
        Arc::clone(&session.agent_store),
        Arc::clone(&blob_store),
        mirror,
        prior_snapshot,
        true,
    )
    .expect("sink");
    assert!(sink.committed_original_turn_observations().expect("new production Journal log").is_empty());
    let fixture_blob = b"canonical generated Agent state blob";
    let fixture_blob_id = Sha256::digest(fixture_blob).to_vec();
    blob_store
        .set_blob_blocking(&fixture_blob_id, fixture_blob)
        .expect("fixture blob");
    assert_eq!(
        sink.read_state_blob(&fixture_blob_id)
            .expect("canonical blob read")
            .as_deref(),
        Some(fixture_blob.as_slice())
    );
    sink.checkpoint_text_turn(
        &user_messages(),
        &TurnRunOptions {
            inference_request_id: Some("request-1".into()),
            message_id: Some("message-1".into()),
            recent_message_text: Some("hello checkpoint".into()),
            turn_started_at_ms: Some(1),
            ..TurnRunOptions::default()
        },
        "assistant checkpoint",
    )
    .expect("checkpoint");
    assert_eq!(sink.committed_original_turn_observations().expect("actual canonical and Journal commit").len(), 1);

    let state = session
        .agent_store
        .latest_checkpoint_bytes()
        .expect("latest checkpoint");
    let decoded =
        decode_transcript_mirror_conversation_state(&state).expect("decoded state");
    assert_eq!(decoded.turns.len(), 1);
    let recovery =
        decode_conversation_state_recovery_fields(&state).expect("decoded recovery state");
    assert_eq!(recovery.turn_timings.len(), 1);
    assert!(recovery.turn_timings[0].timestamp_ms > 1);
    assert_eq!(
        recovery.turn_timings[0].duration_ms,
        recovery.turn_timings[0].timestamp_ms.saturating_sub(1),
    );
    assert!(!session.agent_store.latest_root_blob_id().is_empty());
    assert_eq!(
        session
            .db
            .get_transcript_entry_by_id("message-1")
            .expect("read user echo")
            .and_then(|entry| entry.get("confirmed").and_then(serde_json::Value::as_bool)),
        Some(true)
    );

    let jsonl = fs::read_to_string(
        transcripts_dir
            .join(&session.record.id)
            .join(format!("{}.jsonl", session.record.id)),
    )
    .expect("journal transcript");
    assert!(jsonl.contains("hello checkpoint"));
    assert!(jsonl.contains("assistant checkpoint"));

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn missing_confirmed_message_aborts_root_advance() {
    let root = temp_root("missing-confirm");
    let agents_root = root.join("agents");
    let transcripts_dir = root.join("transcripts");
    let sessions = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let session = sessions
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    let blob_store = Arc::new(
        sessions
            .create_agent_blob_store(&session.record.id)
            .expect("blob store"),
    );
    let prior_snapshot = session.agent_store.checkpoint_snapshot_checked().expect("checked prior snapshot");
    let prior = prior_snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone()).unwrap_or_default();
    let prior_root = session.agent_store.latest_root_blob_id();
    let provider = ProductionTranscriptMirrorProvider::new(
        &transcripts_dir,
        GeneratedTranscriptOccurrenceCodec::new(
            RejectGeneratedToolJsonProjection,
        ),
    );
    let mirror = Arc::new(
        provider
            .route_for_session(
                Arc::clone(&blob_store),
                &prior,
                Arc::new(|| Ok(true)),
            )
            .expect("route"),
    );
    let sink = ProductionAgentStateCheckpointSink::new(
        session.record.id.clone(),
        Arc::clone(&session.agent_store),
        blob_store,
        mirror,
        prior_snapshot,
        true,
    )
    .expect("sink");
    let error = sink
        .checkpoint_text_turn(
            &user_messages(),
            &TurnRunOptions {
                inference_request_id: Some("request-missing".into()),
                message_id: Some("missing-message".into()),
                recent_message_text: Some("hello checkpoint".into()),
                ..TurnRunOptions::default()
            },
            "assistant checkpoint",
        )
        .expect_err("missing user echo must reject checkpoint");
    assert!(error.to_string().contains("were not committed atomically"));
    assert_eq!(session.agent_store.latest_root_blob_id(), prior_root);
    assert!(sink.committed_original_turn_observations().expect("failed confirmation cannot observe a commit").is_empty());

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

struct EmptyBridge;

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool("no tools".into()))
    }
}

struct MemoryProviderCheckpointStore;

impl RoutedProviderCheckpointStore for MemoryProviderCheckpointStore {
    fn persist(
        &self,
        _checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        Ok("provider-checkpoint".into())
    }
}

struct FailingAgentCheckpointSink {
    calls: Mutex<usize>,
}

impl AgentStateCheckpointSink for FailingAgentCheckpointSink {
    fn checkpoint_text_turn(
        &self,
        _messages: &[ProviderMessage],
        _options: &TurnRunOptions,
        _assistant_content: &str,
    ) -> Result<(), ProviderSessionError> {
        *self.calls.lock().expect("calls") += 1;
        Err(ProviderSessionError::Protocol(
            "durable Agent checkpoint failed".into(),
        ))
    }
}

#[test]
fn owner_does_not_settle_completed_before_agent_checkpoint_succeeds() {
    let composition = TurnAgentComposition::new(
        RoutedProvider::OpenRouter,
        Arc::new(EmptyBridge),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "test".into(),
                shell: None,
                time_zone: Some("UTC".into()),
                transcripts_folder: "/tmp/transcripts".into(),
                user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(MemoryProviderCheckpointStore),
    );
    let sink = Arc::new(FailingAgentCheckpointSink {
        calls: Mutex::new(0),
    });
    let owner = ProductionTurnAgentOwner::new(composition)
        .with_agent_state_checkpoint_sink(sink.clone());
    let mut runner = SandAgentRunner::new(owner);
    let error = runner
        .run_with_options(
            &user_messages(),
            TurnRunOptions {
                inference_request_id: Some("request-1".into()),
                message_id: Some("message-1".into()),
                ..TurnRunOptions::default()
            },
            || Ok("provider completed".into()),
        )
        .expect_err("checkpoint failure must fail the turn");
    assert!(error.to_string().contains("durable Agent checkpoint failed"));
    assert_eq!(*sink.calls.lock().expect("calls"), 1);
    assert!(matches!(
        runner.last_finished().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Failed {
            retryable: false,
            message,
        }) if message.contains("durable Agent checkpoint failed")
    ));
}


#[test]
fn production_sink_can_stage_generated_state_before_outer_stream_final_persist() {
    let root = temp_root("staged-generated-state");
    let agents_root = root.join("agents");
    let transcripts_dir = root.join("transcripts");
    let sessions = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let session = sessions
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    session
        .db
        .append_transcript_entry(&serde_json::json!({
            "id": "message-staged",
            "kind": "message",
            "role": "user",
            "content": "stage me",
            "confirmed": false
        }))
        .expect("user echo");
    let blob_store = Arc::new(
        sessions
            .create_agent_blob_store(&session.record.id)
            .expect("blob store"),
    );
    let prior_snapshot = session.agent_store.checkpoint_snapshot_checked().expect("checked prior snapshot");
    let prior = prior_snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone()).unwrap_or_default();
    let provider = ProductionTranscriptMirrorProvider::new(
        &transcripts_dir,
        GeneratedTranscriptOccurrenceCodec::new(
            RejectGeneratedToolJsonProjection,
        ),
    );
    let mirror = Arc::new(
        provider
            .route_for_session(
                Arc::clone(&blob_store),
                &prior,
                Arc::new(|| Ok(true)),
            )
            .expect("route"),
    );
    let sink = ProductionAgentStateCheckpointSink::new(
        session.record.id.clone(),
        Arc::clone(&session.agent_store),
        blob_store,
        mirror,
        prior_snapshot,
        true,
    )
    .expect("sink");
    let options = TurnRunOptions {
        inference_request_id: Some("request-staged".into()),
        message_id: Some("message-staged".into()),
        recent_message_text: Some("stage me".into()),
        ..TurnRunOptions::default()
    };
    let expected_base = sink.checked_base_snapshot().expect("captured staging base");
    let staged = sink
        .stage_text_turn(&expected_base, &[ProviderMessage {
            role: "user".into(),
            content: "stage me".into(),
        }], &options, "assistant staged")
        .expect("stage generated state");
    assert_eq!(session.agent_store.latest_checkpoint_bytes().unwrap_or_default(), prior);
    assert_ne!(staged.state_bytes, prior);

    sink.persist_staged_text_turn(&expected_base, &options, &staged)
        .expect("persist staged generated state");
    assert_eq!(
        session.agent_store.latest_checkpoint_bytes().expect("final state"),
        staged.state_bytes
    );

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_sink_rejects_stale_snapshot_before_mirror_recovery() {
    let root = temp_root("constructor-admission");
    let sessions = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    let session = sessions.materialize_session_with_active(None, "user", None, None)
        .expect("actual initialized session");
    let expected_base = session.agent_store.checkpoint_snapshot_checked().expect("captured absence");
    assert!(expected_base.is_none());
    let blobs = Arc::new(sessions.create_agent_blob_store(&session.record.id).expect("actual blob owner"));
    let transcripts = root.join("transcripts");
    let provider = ProductionTranscriptMirrorProvider::new(
        &transcripts, GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection),
    );
    let route_read = Arc::new(AtomicBool::new(false));
    let observed_route_read = Arc::clone(&route_read);
    let mirror = Arc::new(provider.route_for_session(
        blobs.clone(), &[], Arc::new(move || {
            observed_route_read.store(true, Ordering::Release);
            Ok(true)
        }),
    ).expect("prepare the existing route without recovery"));
    session.agent_store.handle_checkpoint_bytes_blocking(&[]).expect("publish a real present-empty root");
    let current = session.agent_store.checkpoint_snapshot_checked().expect("checked competing root");
    assert!(current.as_ref().is_some_and(|snapshot| snapshot.checkpoint_bytes.is_empty()));
    assert!(!transcripts.join(&session.record.id).exists());

    let result = ProductionAgentStateCheckpointSink::new(
        session.record.id.clone(), Arc::clone(&session.agent_store), blobs,
        mirror, expected_base, true,
    );
    assert!(matches!(result, Err(error) if error.contains("root changed before sink composition")));
    assert!(!route_read.load(Ordering::Acquire), "stale construction must refuse before mirror route selection or recovery");
    assert!(!transcripts.join(&session.record.id).exists(), "refusal cannot claim or recover a Journal");
    assert_eq!(session.agent_store.checkpoint_snapshot_checked().expect("unchanged competing root"), current);
    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_sink_can_checkpoint_inside_existing_localpool_executor() {
    let root = temp_root("nested-localpool");
    let agents_root = root.join("agents");
    let transcripts_dir = root.join("transcripts");
    let sessions = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let session = sessions
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    session
        .db
        .append_transcript_entry(&serde_json::json!({
            "id": "message-nested",
            "kind": "message",
            "role": "user",
            "content": "nested checkpoint",
            "confirmed": false
        }))
        .expect("user echo");
    let blob_store = Arc::new(
        sessions
            .create_agent_blob_store(&session.record.id)
            .expect("blob store"),
    );
    let prior_snapshot = session.agent_store.checkpoint_snapshot_checked().expect("checked prior snapshot");
    let prior = prior_snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone()).unwrap_or_default();
    let provider = ProductionTranscriptMirrorProvider::new(
        &transcripts_dir,
        GeneratedTranscriptOccurrenceCodec::new(
            RejectGeneratedToolJsonProjection,
        ),
    );
    let mirror = Arc::new(
        provider
            .route_for_session(
                Arc::clone(&blob_store),
                &prior,
                Arc::new(|| Ok(true)),
            )
            .expect("route"),
    );
    let sink = ProductionAgentStateCheckpointSink::new(
        session.record.id.clone(),
        Arc::clone(&session.agent_store),
        blob_store,
        mirror,
        prior_snapshot,
        true,
    )
    .expect("sink");

    futures::executor::block_on(async {
        sink.checkpoint_text_turn(
            &[ProviderMessage {
                role: "user".into(),
                content: "nested checkpoint".into(),
            }],
            &TurnRunOptions {
                inference_request_id: Some("request-nested".into()),
                message_id: Some("message-nested".into()),
                recent_message_text: Some("nested checkpoint".into()),
                ..TurnRunOptions::default()
            },
            "assistant nested",
        )
        .expect("checkpoint inside existing LocalPool");
    });

    assert!(!session.agent_store.latest_root_blob_id().is_empty());
    assert_eq!(
        session
            .db
            .get_transcript_entry_by_id("message-nested")
            .expect("read user echo")
            .and_then(|entry| entry.get("confirmed").and_then(serde_json::Value::as_bool)),
        Some(true)
    );

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn settled_checkpoint_suppresses_compaction_stale_tokens_and_appends_timing() {
    // ConversationStateStructure:
    //   summary_archives (13) = [0xaa]
    //   token_details (5) = { used_tokens: 120, max_tokens: 1000 }
    let state = [
        0x6a, 0x01, 0xaa,
        0x2a, 0x05, 0x08, 0x78, 0x10, 0xe8, 0x07,
    ];
    let mut tracker = TokenDetailsPersistenceTracker::new(0);
    let settled = prepare_settled_checkpoint_state(
        &state,
        &mut tracker,
        Some(100),
        175,
    )
    .expect("settled state");
    let decoded =
        decode_conversation_state_recovery_fields(&settled).expect("decode settled state");
    assert!(decoded.token_details.is_none());
    assert_eq!(decoded.summary_archives, vec![vec![0xaa]]);
    assert_eq!(decoded.turn_timings.len(), 1);
    assert_eq!(decoded.turn_timings[0].duration_ms, 75);
    assert_eq!(decoded.turn_timings[0].timestamp_ms, 175);
}

#[derive(Clone, Copy)]
enum OriginalJournalFault {
    None,
    RemovePreparedWal,
    PublishCompetingRoot,
}

const COMPETING_CHECKPOINT: &[u8] = &[0x9a, 0x06, 0x04, b'r', b'a', b'c', b'e'];

struct OriginalObservationFixture {
    root: std::path::PathBuf,
    agent_id: String,
    sessions: Arc<ProductionSessionWorkers>,
    store: Arc<ProductionAgentStore>,
    blobs: Arc<ProductionWorkerBlobStore>,
    sink: ProductionAgentStateCheckpointSink,
    fault_injected: Arc<AtomicBool>,
}

impl OriginalObservationFixture {
    fn new(label: &str, persistence_enabled: bool, journal_enabled: bool, fault: OriginalJournalFault) -> Self {
        let root = temp_root(label);
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let session = sessions.materialize_session_with_active(None, "user", None, None).expect("actual Session owner");
        let agent_id = session.record.id.clone();
        session.db.append_transcript_entry(&serde_json::json!({
            "id":"observation-user", "kind":"message", "role":"user",
            "content":"hello checkpoint", "confirmed":false
        })).expect("actual durable user row");
        let blobs = Arc::new(sessions.create_agent_blob_store(&agent_id).expect("actual canonical blob owner"));
        let prior_snapshot = session.agent_store.checkpoint_snapshot_checked().expect("checked prior snapshot");
        let prior = prior_snapshot.as_ref().map(|snapshot| snapshot.checkpoint_bytes.clone()).unwrap_or_default();
        let transcripts = root.join("transcripts");
        let pending_root = transcripts.clone();
        let fault_injected = Arc::new(AtomicBool::new(false));
        let observed_fault = Arc::clone(&fault_injected);
        let competing_store = Arc::clone(&session.agent_store);
        let provider = ProductionTranscriptMirrorProvider::with_reporter(
            &transcripts,
            GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection),
            Arc::new(move |outcome| {
                if outcome.op == "checkpoint" && outcome.outcome == "ok" {
                    // Both faults use the actual Journal's post-prepare report
                    // before the sink's canonical publication attempt.
                    match fault {
                        OriginalJournalFault::None => return,
                        OriginalJournalFault::RemovePreparedWal => {
                            fs::remove_file(pending_root.join(&outcome.conversation_id)
                                .join(format!("{}.journal-pending.json", outcome.conversation_id)))
                                .expect("remove the exact fixture's prepared Journal WAL");
                        }
                        OriginalJournalFault::PublishCompetingRoot => {
                            competing_store.handle_checkpoint_bytes_blocking(COMPETING_CHECKPOINT)
                                .expect("a real concurrent owner publishes its different root");
                        }
                    }
                    observed_fault.store(true, Ordering::Release);
                }
            }),
        );
        let mirror = Arc::new(provider.route_for_session(
            Arc::clone(&blobs), &prior, Arc::new(move || Ok(journal_enabled)),
        ).expect("actual selected mirror route"));
        let sink = ProductionAgentStateCheckpointSink::new(
            agent_id.clone(), Arc::clone(&session.agent_store), Arc::clone(&blobs),
            mirror, prior_snapshot, persistence_enabled,
        ).expect("actual production checkpoint sink");
        Self { root, agent_id, sessions, store: Arc::clone(&session.agent_store), blobs, sink, fault_injected }
    }

    fn options(first: bool) -> TurnRunOptions {
        TurnRunOptions {
            inference_request_id: Some("observation-stream".into()),
            message_id: first.then(|| "observation-user".into()),
            recent_message_text: Some("hello checkpoint".into()),
            ..TurnRunOptions::default()
        }
    }
}

impl Drop for OriginalObservationFixture {
    fn drop(&mut self) {
        self.sessions.shutdown_with_checkpoint();
        assert_eq!(self.sessions.active_worker_count(), 0);
        if std::thread::panicking() {
            eprintln!("preserved failing original observation fixture: {}", self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[test]
fn original_turn_observations_are_object_safe_unavailable_by_default_and_absent_until_real_commit() {
    let unsupported = FailingAgentCheckpointSink { calls: Mutex::new(0) };
    let sink: &dyn AgentStateCheckpointSink = &unsupported;
    assert!(sink.committed_original_turn_observations().is_err(), "a legacy sink cannot mint sealed evidence");
    assert!(matches!(sink.checked_base_snapshot(), Err(ProviderSessionError::Protocol(_))),
        "a legacy sink cannot establish an absent canonical root");
    assert!(sink.base_state_bytes().is_err(), "byte convenience cannot bypass checked authority");
    let fixture = OriginalObservationFixture::new("original-stage-boundary", true, true, OriginalJournalFault::None);
    let before = fixture.store.latest_root_blob_id();
    let expected_base = fixture.sink.checked_base_snapshot().expect("captured staging base");
    let checkpoint = fixture.sink.stage_text_turn(
        &expected_base, &user_messages(), &OriginalObservationFixture::options(true), "actual committed response",
    ).expect("stage actual generated checkpoint");
    assert!(fixture.sink.committed_original_turn_observations().unwrap().is_empty());
    assert_eq!(fixture.store.latest_root_blob_id(), before, "staging is not durable evidence");
    fixture.sink.persist_staged_text_turn(&expected_base, &OriginalObservationFixture::options(true), &checkpoint)
        .expect("canonical, user confirmation and Journal COMMIT");
    let first = fixture.sink.committed_original_turn_observations().unwrap();
    assert_eq!(first.len(), 1);
    fixture.sink.checkpoint_text_turn(&user_messages(), &OriginalObservationFixture::options(false), "actual nudge")
        .expect("second actual commit");
    assert_eq!(first.len(), 1, "a previously returned immutable observation snapshot cannot be rewritten");
    assert_eq!(fixture.sink.committed_original_turn_observations().unwrap().len(), 2);
}

#[test]
fn production_sink_keeps_invocation_snapshot_binding_after_own_success() {
    let fixture = OriginalObservationFixture::new("captured-invocation", true, true, OriginalJournalFault::None);
    let expected_base = fixture.sink.checked_base_snapshot().expect("first invocation captured base");
    let options = OriginalObservationFixture::options(true);
    let staged = fixture.sink.stage_text_turn(
        &expected_base, &user_messages(), &options, "older staged invocation",
    ).expect("stage against the captured root");
    fixture.sink.checkpoint_text_turn(&user_messages(), &options, "another successful invocation")
        .expect("the same sink publishes a real successor");
    let successor = fixture.store.checkpoint_snapshot_checked().expect("checked successor");
    assert_ne!(successor, expected_base);
    assert_eq!(fixture.sink.checked_base_snapshot().expect("sink advances from its own commit"), successor);
    let journal_path = fixture.root.join("transcripts").join(&fixture.agent_id)
        .join(format!("{}.jsonl", fixture.agent_id));
    let journal_before = fs::read(&journal_path).expect("successful Journal bytes");
    let db = fixture.sessions.open_agent_db_owner(&fixture.agent_id).expect("actual DB owner");
    let user_before = db.get_transcript_entry_by_id("observation-user").expect("confirmed user before refusal");

    assert!(fixture.sink.stage_text_turn(&expected_base, &user_messages(), &options, "must not rebase").is_err());
    assert!(fixture.sink.persist_staged_text_turn(&expected_base, &options, &staged).is_err());
    assert_eq!(fixture.store.checkpoint_snapshot_checked().expect("successor remains current"), successor);
    assert_eq!(fixture.blobs.get_blob_blocking(&staged.turn_id).expect("rejected turn is unpublished"), None);
    assert_eq!(db.get_transcript_entry_by_id("observation-user").expect("confirmed user after refusal"), user_before);
    assert_eq!(fs::read(&journal_path).expect("Journal after refusal"), journal_before);

    fixture.sink.checkpoint_text_turn(
        &user_messages(), &OriginalObservationFixture::options(false), "explicit subsequent invocation",
    ).expect("later work captures the sink's own current successor");
    assert_ne!(fixture.sink.checked_base_snapshot().expect("second successful publication"), successor);
}

#[test]
fn original_turn_observation_overflow_is_permanent_without_failing_successful_checkpoints() {
    let fixture = OriginalObservationFixture::new("original-observation-bound", true, true, OriginalJournalFault::None);
    let limit = mahayana_host_runtime::extensions::transcript::turn_runtime::MAX_REPLY_NUDGES + 2;
    assert_eq!(limit, 5, "ORT-v1's durable bound must be deliberately migrated with the invocation bound");
    for invocation in 0..limit + 2 {
        fixture.sink.checkpoint_text_turn(
            &user_messages(), &OriginalObservationFixture::options(invocation == 0),
            &format!("real checkpoint {invocation}"),
        ).expect("proof ineligibility must never retry or fail an otherwise successful checkpoint");
        if invocation < limit {
            assert_eq!(fixture.sink.committed_original_turn_observations().unwrap().len(), invocation + 1);
        } else {
            assert!(fixture.sink.committed_original_turn_observations().is_err(), "overflow cannot truncate into a shorter proof");
        }
    }
    let state = fixture.store.latest_checkpoint_bytes().unwrap();
    assert_eq!(decode_conversation_state_recovery_fields(&state).unwrap().turns.len(), limit + 2);
}

#[test]
fn original_turn_skip_and_legacy_success_or_swallowed_write_failure_never_expose_observations() {
    let skipped = OriginalObservationFixture::new("original-skip", false, true, OriginalJournalFault::None);
    assert!(skipped.sink.committed_original_turn_observations().is_err());
    skipped.sink.checkpoint_text_turn(&user_messages(), &OriginalObservationFixture::options(true), "skip mode response")
        .expect("existing skip checkpoint behavior");
    assert!(skipped.sink.committed_original_turn_observations().is_err());
    assert!(!skipped.store.latest_root_blob_id().is_empty());

    for blocked in [false, true] {
        let fixture = OriginalObservationFixture::new(
            if blocked { "original-legacy-failure" } else { "original-legacy-success" }, true, false, OriginalJournalFault::None,
        );
        assert!(fixture.sink.committed_original_turn_observations().is_err(), "Legacy is unavailable immediately after route selection");
        let expected_base = fixture.sink.checked_base_snapshot().expect("captured legacy staging base");
        let mut checkpoint = fixture.sink.stage_text_turn(
            &expected_base, &user_messages(), &OriginalObservationFixture::options(true), "generated response omitted by Legacy",
        ).unwrap();
        // Exercise the actual Legacy full-write branch with a real root-prompt
        // JSON blob. This is negative route evidence, never a generated positive
        // proof fixture: Legacy does not attest the generated turn appended here.
        let prompt = br#"{"role":"user","content":"actual legacy root prompt"}"#;
        let prompt_id = Sha256::digest(prompt).to_vec();
        fixture.blobs.set_blob_blocking(&prompt_id, prompt).expect("persist actual Legacy prompt blob");
        checkpoint.state_bytes.extend_from_slice(&[0x0a, 32]);
        checkpoint.state_bytes.extend_from_slice(&prompt_id);
        let directory = fixture.root.join("transcripts").join(&fixture.agent_id);
        if blocked {
            fs::create_dir_all(directory.parent().unwrap()).unwrap();
            fs::write(&directory, b"owned Legacy filesystem obstruction").unwrap();
        }
        fixture.sink.persist_staged_text_turn(&expected_base, &OriginalObservationFixture::options(true), &checkpoint)
            .expect("Legacy's observational write result preserves existing canonical checkpoint behavior");
        assert!(!fixture.store.latest_root_blob_id().is_empty());
        assert!(fixture.sink.committed_original_turn_observations().is_err());
        if blocked {
            assert_eq!(fs::read(&directory).unwrap(), b"owned Legacy filesystem obstruction");
        } else {
            let journal = fs::read_to_string(directory.join(format!("{}.jsonl", fixture.agent_id)))
                .expect("actual Legacy offload successfully wrote its root-prompt transcript");
            assert!(journal.contains("actual legacy root prompt"));
            assert!(!journal.contains("generated response omitted by Legacy"));
        }
    }
}

#[test]
fn original_turn_journal_commit_failure_after_durable_root_and_confirmation_cannot_mint_observation() {
    let fixture = OriginalObservationFixture::new("original-journal-commit-failure", true, true, OriginalJournalFault::RemovePreparedWal);
    let before = fixture.store.latest_root_blob_id();
    let error = fixture.sink.checkpoint_text_turn(
        &user_messages(), &OriginalObservationFixture::options(true), "durable but unmirrored result",
    ).expect_err("actual Journal COMMIT must refuse its removed prepared WAL");
    assert!(error.to_string().contains("transcript mirror commit failed after durable checkpoint"));
    assert!(fixture.fault_injected.load(Ordering::Acquire), "fault reached the actual post-prepare boundary");
    assert_ne!(fixture.store.latest_root_blob_id(), before, "canonical root really committed before mirror refusal");
    assert_eq!(fixture.sessions.open_agent_db_owner(&fixture.agent_id).unwrap()
        .get_transcript_entry_by_id("observation-user").unwrap().unwrap()["confirmed"], true);
    assert!(fixture.sink.committed_original_turn_observations().is_err(), "root-only success is not a committed original-turn observation");
    let committed = fixture.store.checkpoint_snapshot_checked().expect("durable root remains readable");
    assert!(fixture.sink.checked_base_snapshot().is_err(), "the failed sink retains its original expectation");
    assert!(fixture.sink.checkpoint_text_turn(
        &user_messages(), &OriginalObservationFixture::options(false), "must not adopt the unmirrored root",
    ).is_err());
    assert_eq!(fixture.store.checkpoint_snapshot_checked().expect("no silent rebase after Journal refusal"), committed);
    assert!(fixture.sink.committed_original_turn_observations().is_err(), "a later refusal cannot revive completion authority");
}

#[test]
fn original_turn_prior_snapshot_is_checked_under_the_actual_store_mutation_guard() {
    let fixture = OriginalObservationFixture::new(
        "original-prior-root-race", true, true, OriginalJournalFault::PublishCompetingRoot,
    );
    let error = fixture.sink.checkpoint_text_turn(
        &user_messages(), &OriginalObservationFixture::options(true), "stale original result",
    ).expect_err("a competing committed root after prepare cannot become this sink's unobserved prior");
    assert!(!error.to_string().is_empty());
    assert!(fixture.fault_injected.load(Ordering::Acquire));
    assert_eq!(fixture.store.latest_checkpoint_bytes().as_deref(), Some(COMPETING_CHECKPOINT));
    assert_eq!(fixture.store.latest_root_blob_id(), Sha256::digest(COMPETING_CHECKPOINT).to_vec(),
        "the original-turn writer must not overwrite the intervening real checkpoint");
    assert_eq!(fixture.sessions.open_agent_db_owner(&fixture.agent_id).unwrap()
        .get_transcript_entry_by_id("observation-user").unwrap().unwrap()["confirmed"], false);
    assert!(fixture.sink.committed_original_turn_observations().unwrap().is_empty());
}
