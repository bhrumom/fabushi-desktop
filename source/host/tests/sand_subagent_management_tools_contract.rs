use mahayana_host_runtime::runner::subagent_runtime::{
    SubagentRuntime, SubagentSessionSnapshot,
};
use mahayana_host_runtime::runner::tools::sand_subagent_management_tools::{
    SteerReview, check_subagent, describe_running_subagent, elapsed_label,
    message_subagent, not_running_message, stop_subagent,
};

#[test]
fn elapsed_labels_match_frozen_rounding_and_minute_format() {
    assert_eq!(elapsed_label(0), "0s");
    assert_eq!(elapsed_label(1_499), "1s");
    assert_eq!(elapsed_label(1_500), "2s");
    assert_eq!(elapsed_label(89_400), "89s");
    assert_eq!(elapsed_label(89_500), "1m 30s");
    assert_eq!(elapsed_label(120_000), "2m");
}

fn running_runtime() -> SubagentRuntime {
    let mut runtime = SubagentRuntime::default();
    runtime.register_session(
        "worker-a",
        SubagentSessionSnapshot {
            observed_tool_call_count: 3,
            recent_activity: vec!["Read file".into(), "Clicked button".into()],
            transcript_path: Some("/tmp/worker-a.jsonl".into()),
            ..SubagentSessionSnapshot::default()
        },
    );
    runtime.dispatch_background_subagent(
        "parent",
        "box",
        "worker-a",
        "computerUse",
        "call-a",
        "Inspect the UI",
        None,
        None,
        1_000,
    );
    runtime
}

#[test]
fn check_projection_lists_and_expands_live_runtime_without_second_registry() {
    let runtime = running_runtime();
    let list = check_subagent(&runtime, None, 62_000);
    assert!(list.starts_with("1 subagent(s) running:"));
    assert!(list.contains("worker-a [computerUse]"));
    assert!(list.contains("running for 1m 1s"));
    assert!(list.contains("3 tool call(s)"));
    assert!(list.contains("Pass a subagent_id"));

    let detailed = check_subagent(&runtime, Some(" worker-a "), 62_000);
    assert!(detailed.contains("Recent activity (oldest → newest):"));
    assert!(detailed.contains("Read file"));
    assert!(detailed.contains("/tmp/worker-a.jsonl"));
}

#[test]
fn not_running_message_reports_current_ids_or_empty_registry() {
    let runtime = running_runtime();
    let running = runtime.list_running_subagents(2_000);
    assert!(
        not_running_message("missing", &running)
            .ends_with("Currently running: worker-a.")
    );
    assert!(
        not_running_message("missing", &[])
            .ends_with("No subagents are running right now.")
    );
}

#[test]
fn steer_review_denial_does_not_interrupt_and_allowed_message_uses_runtime() {
    let mut runtime = running_runtime();
    let denied = message_subagent(
        &mut runtime,
        "worker-a",
        "Try another button",
        Some(&SteerReview {
            allowed: false,
            reason: "review blocked".into(),
        }),
        2_000,
    );
    assert_eq!(denied, "review blocked");
    assert!(runtime.is_running("worker-a"));

    let delivered = message_subagent(
        &mut runtime,
        "worker-a",
        "Try another button",
        Some(&SteerReview {
            allowed: true,
            reason: String::new(),
        }),
        2_000,
    );
    assert!(delivered.starts_with("Message delivered to subagent worker-a."));
}

#[test]
fn stop_marks_runtime_aborting_and_missing_targets_get_frozen_guidance() {
    let mut runtime = running_runtime();
    let stopped = stop_subagent(&mut runtime, "worker-a", 2_000);
    assert_eq!(
        stopped,
        "Stopping subagent worker-a. It will be torn down and won't report back."
    );

    let missing = stop_subagent(&mut runtime, "missing", 2_000);
    assert!(missing.starts_with("No subagent \"missing\" is currently running."));
}

#[test]
fn detailed_description_without_activity_has_explicit_empty_state() {
    let info = mahayana_host_runtime::runner::subagent_runtime::RunningSubagentInfo {
        subagent_id: "id".into(),
        subagent_type: "generalPurpose".into(),
        title: "Title".into(),
        elapsed_ms: 4_000,
        tool_call_count: 0,
        recent_activity: vec![],
        transcript_path: None,
    };
    let text = describe_running_subagent(&info, true);
    assert!(text.contains("No tool activity recorded yet."));
}
