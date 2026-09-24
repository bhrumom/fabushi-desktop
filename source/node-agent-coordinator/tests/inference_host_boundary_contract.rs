use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_node_agent_coordinator::inference_router::{
    ActiveInferenceStreamRegistry, InferenceProvider, InferenceStreamSupersede,
    RunnerInferenceEvent, configured_inference_provider, host_transcript_method,
    is_direct_user_send, parse_host_routed_prompt_acceptance,
    parse_runner_inference_event, project_runner_turn_context,
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
fn coordinator_projects_durable_turn_context_without_host_linkage() {
    let projected = project_runner_turn_context(
        &json!({
            "attachmentPaths":["/tmp/a"],
            "selectedImages":[{"id":"image"}],
            "selectedVideos":[{"id":"video"}],
            "replyContext":{"id":"reply"},
            "isFork":true,
            "richText":"**hello**",
            "composedAtMs":10,
            "enterEpochMs":20,
            "requestSource":"handoff-resume",
            "ackRedrive":true,
            "ackRedriveTrigger":"idle",
            "redriveAttempts":2,
            "ignored":"not-forwarded"
        }),
        "t7u",
        vec![
            json!({"id":"t6u","text":"older"}),
            json!({"id":"t7u","text":"current","richText":"**current**"}),
        ],
    );
    assert_eq!(projected["messageId"], "t7u");
    assert_eq!(projected["recentUserMessages"].as_array().unwrap().len(), 2);
    assert_eq!(projected["attachmentPaths"], json!(["/tmp/a"]));
    assert_eq!(projected["selectedImages"][0]["id"], "image");
    assert_eq!(projected["selectedVideos"][0]["id"], "video");
    assert_eq!(projected["replyContext"]["id"], "reply");
    assert_eq!(projected["isFork"], true);
    assert_eq!(projected["richText"], "**hello**");
    assert_eq!(projected["composedAtMs"], 10);
    assert_eq!(projected["enterEpochMs"], 20);
    assert_eq!(projected["requestSource"], "handoff-resume");
    assert_eq!(projected["ackRedrive"], true);
    assert_eq!(projected["ackRedriveTrigger"], "idle");
    assert_eq!(projected["redriveAttempts"], 2);
    assert!(projected.get("ignored").is_none());
}

#[test]
fn host_routed_prompt_acceptance_supplies_authoritative_recovery_identity() {
    let acceptance = parse_host_routed_prompt_acceptance(&json!({
        "accepted": true,
        "duplicate": false,
        "echoEntryId": "user-message:9",
        "userMessageId": "user-message:9",
        "recentUserMessages": [
            {"id":"user-message:8","text":"older","confirmed":true},
            {"id":"user-message:9","text":"current"}
        ]
    }))
    .expect("Host acceptance");
    assert!(!acceptance.duplicate);
    assert_eq!(acceptance.echo_entry_id.as_deref(), Some("user-message:9"));
    assert_eq!(acceptance.user_message_id.as_deref(), Some("user-message:9"));
    assert_eq!(acceptance.recent_user_messages.len(), 2);
    assert!(parse_host_routed_prompt_acceptance(&json!({"accepted":false})).is_err());
}

#[test]
fn active_inference_stream_supersede_fences_start_accept_and_stale_finish() {
    let registry = ActiveInferenceStreamRegistry::default();
    registry.begin("agent-a", "stream-1").expect("begin stream");
    assert_eq!(
        registry.request_supersede("agent-a"),
        InferenceStreamSupersede::DeferredUntilAccepted {
            stream_id: "stream-1".into(),
        }
    );
    assert!(registry
        .mark_accepted("agent-a", "stream-1")
        .expect("accept stream"));
    assert_eq!(
        registry.request_supersede("agent-a"),
        InferenceStreamSupersede::CancelNow {
            stream_id: "stream-1".into(),
        }
    );
    assert!(registry.finish("agent-a", "stream-1"));
    assert_eq!(registry.current_stream_id("agent-a"), None);

    registry.begin("agent-a", "stream-2").expect("next stream");
    assert!(!registry.finish("agent-a", "stale-stream"));
    assert_eq!(
        registry.current_stream_id("agent-a").as_deref(),
        Some("stream-2")
    );
    assert!(registry.finish("agent-a", "stream-2"));
}

#[test]
fn only_direct_user_sends_supersede_the_active_provider_turn() {
    assert!(is_direct_user_send(&json!({"prompt":"hello"})));
    assert!(is_direct_user_send(&json!({
        "prompt":"fork still direct",
        "isFork":true
    })));
    assert!(!is_direct_user_send(&json!({
        "prompt":"automation",
        "automationWake":{"id":"auto-1"}
    })));
    assert!(!is_direct_user_send(&json!({
        "prompt":"group",
        "groupContext":{"groupId":"g-1"}
    })));
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
