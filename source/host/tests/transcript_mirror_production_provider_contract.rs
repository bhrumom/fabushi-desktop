const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::runner::persist_checkpoint_with_mirror;
use mahayana_host_runtime::transcript_mirror::production_provider::{
    ProductionTranscriptCheckpoint, ProductionTranscriptMirrorProvider,
};
use mahayana_host_runtime::transcript_mirror::transcript_occurrence_deriver::{
    DecodedTranscriptStep, DecodedTranscriptTurn, DecodedUserMessage,
    TranscriptOccurrenceCodec,
};

#[derive(Clone)]
struct FrozenFixtureCodec;

impl TranscriptOccurrenceCodec for FrozenFixtureCodec {
    fn decode_turn(&self, bytes: &[u8]) -> Result<DecodedTranscriptTurn, String> {
        match bytes {
            [0x01] => Ok(DecodedTranscriptTurn::Agent {
                user_message: vec![0x09],
                steps: Vec::new(),
            }),
            [0x02] => Ok(DecodedTranscriptTurn::Shell),
            _ => Ok(DecodedTranscriptTurn::Undefined),
        }
    }

    fn decode_user_message(&self, bytes: &[u8]) -> Result<DecodedUserMessage, String> {
        if bytes == [0x09] {
            Ok(DecodedUserMessage {
                text: "hello from production provider".into(),
                text_blob_id: None,
            })
        } else {
            Err("unknown user fixture".into())
        }
    }

    fn decode_step(&self, _bytes: &[u8]) -> Result<DecodedTranscriptStep, String> {
        Ok(DecodedTranscriptStep::Undefined)
    }
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-production-transcript-provider-{label}-{}-{suffix}",
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

fn push_length_delimited(field: u64, value: &[u8], output: &mut Vec<u8>) {
    encode_varint((field << 3) | 2, output);
    encode_varint(value.len() as u64, output);
    output.extend_from_slice(value);
}

fn state_bytes(
    root_prompts: &[&[u8]],
    turns: &[&[u8]],
    summary_archives: &[&[u8]],
) -> Vec<u8> {
    let mut output = Vec::new();
    for value in root_prompts {
        push_length_delimited(1, value, &mut output);
    }
    for value in turns {
        push_length_delimited(8, value, &mut output);
    }
    for value in summary_archives {
        push_length_delimited(13, value, &mut output);
    }
    output
}

#[test]
fn shipping_runner_binds_generated_checkpoint_codec_into_file_transcript_mirror() {
    assert!(SHIPPING_HOST.contains("ProductionTranscriptMirrorProvider::with_reporter("));
    assert!(SHIPPING_HOST.contains("report_journal_outcome(&report)"));
    assert!(SHIPPING_HOST.contains("GeneratedTranscriptOccurrenceCodec::new("));
    assert!(SHIPPING_HOST.contains("RejectGeneratedToolJsonProjection"));
    assert!(SHIPPING_HOST.contains(".route_for_session("));
    assert!(SHIPPING_HOST.contains("ProductionAgentStateCheckpointSink::new("));
    assert!(SHIPPING_HOST.contains("transcript_mirror,"));
}

#[test]
fn production_checkpoint_preserves_journal_and_legacy_views() {
    let bytes = state_bytes(
        &[b"prompt-a", b"prompt-b"],
        &[&[0x01], &[0x02]],
        &[b"archive"],
    );
    let checkpoint =
        ProductionTranscriptCheckpoint::from_state_bytes(&bytes).expect("checkpoint");
    assert_eq!(checkpoint.state_bytes, bytes);
    assert_eq!(checkpoint.journal.turns, vec![vec![0x01], vec![0x02]]);
    assert_eq!(checkpoint.legacy.root_prompt_messages_json.len(), 2);
    assert_eq!(checkpoint.legacy.summary_archives, vec![b"archive".to_vec()]);
    assert_eq!(checkpoint.root_prompt_count(), 2);
}

#[test]
fn provider_routes_real_worker_blob_store_through_shared_journal() {
    let root = temp_root("journal");
    let agents_root = root.join("agents");
    let transcripts_dir = root.join("transcripts");
    let workers = ProductionSessionWorkers::with_agents_root(&agents_root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    let store = Arc::new(
        workers
            .create_agent_blob_store(&session.record.id)
            .expect("blob store"),
    );
    futures::executor::block_on(store.set_blob(&(), &[0x01], &[0x01]))
        .expect("turn blob");
    futures::executor::block_on(store.set_blob(&(), &[0x09], &[0x09]))
        .expect("user blob");

    let outcomes = Arc::new(Mutex::new(Vec::new()));
    let captured_outcomes = Arc::clone(&outcomes);
    let provider = ProductionTranscriptMirrorProvider::with_reporter(
        &transcripts_dir,
        FrozenFixtureCodec,
        Arc::new(move |outcome| {
            captured_outcomes.lock().unwrap().push(outcome.clone());
        }),
    );
    let routed = provider
        .route_for_session(
            Arc::clone(&store),
            &[],
            Arc::new(|| Ok(true)),
        )
        .expect("route");

    let base = ProductionTranscriptCheckpoint::from_state_bytes(&[])
        .expect("base state");
    routed
        .recover(&session.record.id, &base, &store)
        .expect("recover");
    let next_bytes = state_bytes(&[], &[&[0x01]], &[]);
    let next = ProductionTranscriptCheckpoint::from_state_bytes(&next_bytes)
        .expect("next state");
    futures::executor::block_on(persist_checkpoint_with_mirror(
        Some(&routed),
        Some(session.agent_store.as_ref()),
        &session.record.id,
        &next,
        &store,
        true,
        false,
        true,
        |_| Err("local checkpoint fallback must not run for a production AgentStore".into()),
    ))
    .expect("production settle checkpoint transaction");
    assert_eq!(
        session.agent_store.latest_checkpoint_bytes().as_deref(),
        Some(next_bytes.as_slice())
    );
    assert!(!session.agent_store.latest_root_blob_id().is_empty());

    let jsonl = fs::read_to_string(
        transcripts_dir
            .join(&session.record.id)
            .join(format!("{}.jsonl", session.record.id)),
    )
    .expect("jsonl");
    assert!(jsonl.contains("hello from production provider"));
    assert!(jsonl.contains("\"role\":\"user\""));

    let outcomes = outcomes.lock().unwrap();
    assert!(outcomes.iter().any(|outcome| {
        outcome.op == "replay"
            && outcome.outcome == "ok"
            && outcome.conversation_id == session.record.id
    }));
    assert!(outcomes.iter().any(|outcome| {
        outcome.op == "checkpoint"
            && outcome.outcome == "ok"
            && outcome.entry_count == Some(1)
    }));
    assert!(outcomes.iter().any(|outcome| {
        outcome.op == "append"
            && outcome.outcome == "ok"
            && outcome.entry_count == Some(1)
    }));
    drop(outcomes);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
