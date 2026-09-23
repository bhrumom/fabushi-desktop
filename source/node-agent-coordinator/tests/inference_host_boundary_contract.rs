use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_node_agent_coordinator::inference_router::{
    InferenceProvider, RunnerInferenceEvent, configured_inference_provider,
    host_transcript_method, parse_runner_inference_event,
};
use serde_json::json;

#[test]
fn coordinator_provider_selection_is_pure_and_host_runtime_independent() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-coordinator-provider-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp settings root");
    let settings = root.join("settings.json");
    fs::write(&settings, r#"{"router":{"provider":"claude-code"}}"#)
        .expect("write settings");
    assert_eq!(
        configured_inference_provider(&settings),
        Some(InferenceProvider::ClaudeCode)
    );
    let _ = fs::remove_dir_all(root);

    let cargo = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("coordinator Cargo.toml");
    assert!(
        !cargo.contains("mahayana-host-runtime"),
        "Coordinator must not link the Host/Runner implementation crate in-process"
    );
}

#[test]
fn renderer_tail_alias_is_normalized_before_host_dispatch() {
    assert_eq!(host_transcript_method("openAgentTail"), "getAgentTranscriptTail");
    assert_eq!(host_transcript_method("getAgentTranscriptTail"), "getAgentTranscriptTail");
    assert_eq!(host_transcript_method("getAgentTranscriptWindow"), "getAgentTranscriptWindow");
}

#[test]
fn runner_inference_events_are_correlated_by_stream_id() {
    let (stream, event) = parse_runner_inference_event(&json!({
        "streamId":"stream-42",
        "type":"delta",
        "content":"hello"
    }))
    .expect("delta event");
    assert_eq!(stream, "stream-42");
    assert_eq!(
        event,
        RunnerInferenceEvent::Delta {
            content: "hello".into()
        }
    );

    let (_, completed) = parse_runner_inference_event(&json!({
        "streamId":"stream-42",
        "type":"completed",
        "content":"hello world"
    }))
    .expect("completed event");
    assert_eq!(
        completed,
        RunnerInferenceEvent::Completed {
            content: "hello world".into()
        }
    );

    let (_, cancelled) = parse_runner_inference_event(&json!({
        "streamId":"stream-42",
        "type":"cancelled",
        "message":"user cancelled"
    }))
    .expect("cancelled event");
    assert_eq!(
        cancelled,
        RunnerInferenceEvent::Cancelled {
            message: "user cancelled".into()
        }
    );

    assert!(parse_runner_inference_event(&json!({
        "streamId":"stream-42",
        "type":"unknown"
    }))
    .is_err());
}
