use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, ProductionAgentStateCheckpointSink,
    build_text_turn_checkpoint,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::ProductionTurnAgentOwner;
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::turn_agent_composition::TurnAgentComposition;
use mahayana_host_runtime::runner::{TerminalOutcome, TurnRunOptions};
use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
    decode_transcript_mirror_conversation_state,
};
use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
};
use mahayana_host_runtime::transcript_mirror::production_provider::{
    ProductionTranscriptMirrorProvider,
};
use serde_json::Value;

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
    let prior = session.agent_store.latest_checkpoint_bytes().unwrap_or_default();
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
        prior,
        true,
    )
    .expect("sink");
    sink.checkpoint_text_turn(
        &user_messages(),
        &TurnRunOptions {
            inference_request_id: Some("request-1".into()),
            message_id: Some("message-1".into()),
            recent_message_text: Some("hello checkpoint".into()),
            ..TurnRunOptions::default()
        },
        "assistant checkpoint",
    )
    .expect("checkpoint");

    let state = session
        .agent_store
        .latest_checkpoint_bytes()
        .expect("latest checkpoint");
    let decoded =
        decode_transcript_mirror_conversation_state(&state).expect("decoded state");
    assert_eq!(decoded.turns.len(), 1);
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
    let prior = session.agent_store.latest_checkpoint_bytes().unwrap_or_default();
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
        prior,
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
