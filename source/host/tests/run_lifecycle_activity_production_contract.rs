use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionTranscriptRuntime;
use mahayana_host_runtime::sand_activity::{ActivityUpdate, AgentActivity};

#[test]
fn shipping_runtime_projects_runner_activity_into_agent_roster_rows() {
    let runtime = ProductionTranscriptRuntime::new(None);
    runtime.begin_provider_run("agent-a");
    runtime.track_runner_activity_update(
        "agent-a",
        &ActivityUpdate::ToolCall {
            id: "call-1".into(),
            name: "mcpToolCall".into(),
            status: "pending".into(),
            args: Some(r#"{"providerIdentifier":"calendar"}"#.into()),
            summary: None,
        },
        1_000,
    );

    let mut rows = serde_json::json!([{"id":"agent-a"},{"id":"agent-b"}]);
    runtime.decorate_agent_summaries(&mut rows);
    assert_eq!(rows[0]["isRunning"], true);
    assert_eq!(rows[0]["isRunningTurn"], true);
    assert_eq!(rows[0]["currentActivity"]["kind"], "tool");
    assert_eq!(rows[0]["currentActivity"]["tool"], "CallMcpTool");
    assert_eq!(rows[0]["currentActivity"]["detail"], "calendar");
    assert_eq!(rows[1]["isRunning"], false);

    runtime.track_runner_activity_update(
        "agent-a",
        &ActivityUpdate::TextDelta { text: "hello".into() },
        1_100,
    );
    let mut held = serde_json::json!([{"id":"agent-a"}]);
    runtime.decorate_agent_summaries(&mut held);
    assert_eq!(held[0]["currentActivity"]["tool"], "CallMcpTool");

    runtime.track_runner_activity_update(
        "agent-a",
        &ActivityUpdate::TurnEnded,
        4_000,
    );
    runtime.end_provider_run("agent-a");
    let mut ended = serde_json::json!([{"id":"agent-a"}]);
    runtime.decorate_agent_summaries(&mut ended);
    assert_eq!(ended[0]["isRunning"], false);
    assert!(ended[0]["currentActivity"].is_null());
}

#[test]
fn run_lifecycle_composing_and_retrying_follow_frozen_updates() {
    let runtime = ProductionTranscriptRuntime::new(None);
    runtime.begin_provider_run("agent-a");
    runtime.track_runner_activity_update(
        "agent-a",
        &ActivityUpdate::ToolCall {
            id: "send".into(),
            name: "SendMessage".into(),
            status: "pending".into(),
            args: None,
            summary: None,
        },
        10,
    );
    let mut composing = serde_json::json!([{"id":"agent-a"}]);
    runtime.decorate_agent_summaries(&mut composing);
    assert_eq!(composing[0]["isComposingMessage"], true);

    runtime.track_runner_activity_update("agent-a", &ActivityUpdate::Retrying, 20);
    let mut retrying = serde_json::json!([{"id":"agent-a"}]);
    runtime.decorate_agent_summaries(&mut retrying);
    assert_eq!(retrying[0]["isRetrying"], true);

    runtime.track_runner_activity_update(
        "agent-a",
        &ActivityUpdate::SendMessage,
        30,
    );
    let mut sent = serde_json::json!([{"id":"agent-a"}]);
    runtime.decorate_agent_summaries(&mut sent);
    assert_eq!(sent[0]["isComposingMessage"], false);
    assert_eq!(sent[0]["isRetrying"], false);
    runtime.end_provider_run("agent-a");
}
