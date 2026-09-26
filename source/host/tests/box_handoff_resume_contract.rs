use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_db::read_persisted_agent_serde_snapshot;
use mahayana_host_runtime::extensions::session::agent_db_serde::AwaitingUserResponse;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::box_handoff_resume::{
    BOX_HANDOFF_DISMISSED_PROMPT, BOX_HANDOFF_RESUME_PROMPT, BOX_HANDOFF_VIEWER_CLOSED_PROMPT,
    box_handoff_resume_prompt, build_box_handoff_resume_send_args, settle_box_handoff_state,
};
use mahayana_host_runtime::extensions::transcript::production_runtime::classify_send_dispatch;
use mahayana_host_runtime::extensions::transcript::run_scheduler::RunLane;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-box-handoff-resume-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn frozen_handoff_prompts_and_hidden_send_contract_are_preserved() {
    assert_eq!(box_handoff_resume_prompt("button"), BOX_HANDOFF_RESUME_PROMPT);
    assert_eq!(
        box_handoff_resume_prompt("dismissed"),
        BOX_HANDOFF_DISMISSED_PROMPT
    );
    assert_eq!(
        box_handoff_resume_prompt("viewer-closed"),
        BOX_HANDOFF_VIEWER_CLOSED_PROMPT
    );

    let args = build_box_handoff_resume_send_args("agent-a", "dismissed", 123);
    assert_eq!(args["agentId"], "agent-a");
    assert_eq!(args["prompt"], BOX_HANDOFF_DISMISSED_PROMPT);
    assert_eq!(args["appendUserMessage"], false);
    assert_eq!(args["awaitTurn"], false);
    assert_eq!(args["directAddressedAcceptance"], true);
    assert_eq!(args["requestSource"], "handoff-resume");
    assert_eq!(args["hidden"], true);
    assert_eq!(args["skipAckObligation"], true);
    assert_eq!(args["clientNonce"], "box-handoff-resume:agent-a:123");

    let (lane, source) = classify_send_dispatch(&args).expect("handoff dispatch");
    assert_eq!(lane, RunLane::Background);
    assert_eq!(source, "handoff-resume");

    let ack_redrive = serde_json::json!({
        "requestSource": "handoff-resume",
        "ackRedrive": true
    });
    let (lane, source) = classify_send_dispatch(&ack_redrive).expect("ack dispatch");
    assert_eq!(lane, RunLane::Background);
    assert_eq!(source, "ack-redrive");
}

#[test]
fn production_handoff_settlement_clears_awaiting_and_resolves_matching_box_entry() {
    let root = temp_root("production");
    let sessions = ProductionSessionWorkers::with_agents_root(&root, 500);
    let record = sessions
        .materialize_new_session(None, "user", None)
        .expect("materialize agent");

    sessions
        .set_agent_awaiting_user_response(
            &record.id,
            Some(&AwaitingUserResponse {
                tab_id: "tab-1".into(),
                reason: "box-help".into(),
                since: 9.0,
            }),
        )
        .expect("awaiting state");
    sessions
        .append_agent_transcript_entries(
            &record.id,
            &[
                serde_json::json!({
                    "id": "box-request-1",
                    "kind": "send-message",
                    "message": {"type":"text","content":"Please take over"},
                    "timestampMs": 10,
                    "boxRequestId": "request-1",
                    "boxInstruction": "Solve the captcha"
                }),
                serde_json::json!({
                    "id": "box-request-other",
                    "kind": "send-message",
                    "message": {"type":"text","content":"Other request"},
                    "timestampMs": 11,
                    "boxRequestId": "request-other"
                }),
            ],
        )
        .expect("box request entries");

    let settled = settle_box_handoff_state(
        &sessions,
        &record.id,
        "request-1",
        "completed",
    )
    .expect("settle handoff");
    assert!(settled.awaiting_cleared);
    assert_eq!(
        settled
            .resolved_entry
            .as_ref()
            .and_then(|entry| entry.get("boxResolution"))
            .and_then(serde_json::Value::as_str),
        Some("completed")
    );

    let snapshot = read_persisted_agent_serde_snapshot(&record.db_path, 500)
        .expect("snapshot");
    assert!(snapshot.awaiting_user_response.is_none());

    let entries = sessions
        .read_agent_transcript_entries(&record.id)
        .expect("transcript");
    let resolved = entries
        .iter()
        .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some("box-request-1"))
        .expect("resolved entry");
    assert_eq!(resolved["boxResolution"], "completed");
    let untouched = entries
        .iter()
        .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some("box-request-other"))
        .expect("other entry");
    assert!(untouched.get("boxResolution").is_none());

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
