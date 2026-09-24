use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
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
fn production_checkpoint_preserves_journal_and_legacy_views() {
    let bytes = state_bytes(
        &[b"prompt-a", b"prompt-b"],
        &[&[0x01], &[0x02]],
        &[b"archive"],
    );
    let checkpoint =
        ProductionTranscriptCheckpoint::from_state_bytes(&bytes).expect("checkpoint");
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

    let provider =
        ProductionTranscriptMirrorProvider::new(&transcripts_dir, FrozenFixtureCodec);
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
    let next = ProductionTranscriptCheckpoint::from_state_bytes(
        &state_bytes(&[], &[&[0x01]], &[]),
    )
    .expect("next state");
    routed
        .prepare_checkpoint(
            &session.record.id,
            &next,
            &store,
            true,
            true,
        )
        .expect("prepare");
    routed
        .commit_checkpoint(&session.record.id, &[0xaa])
        .expect("commit");

    let jsonl = fs::read_to_string(
        transcripts_dir.join(format!("{}.jsonl", session.record.id)),
    )
    .expect("jsonl");
    assert!(jsonl.contains("hello from production provider"));
    assert!(jsonl.contains("\"role\":\"user\""));

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}
