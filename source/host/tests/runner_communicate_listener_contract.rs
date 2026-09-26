use mahayana_host_runtime::runner::tools::communicate_tool::{
    CommunicateActivity, CommunicateResult, SAND_TOOL_MARKER, build_error_result,
    build_success_result, completed_tool_call, empty_tool_call, encode_error, encode_sand_step,
    executing_tool_call, render_result, run_communicate_execution, serialize_error,
    tool_call_wrapper,
};
use mahayana_host_runtime::runner::tools::listener_connect_cards::{
    surface_listener_connect_cards,
};
use serde_json::{Value, json};

#[test]
fn listener_cards_are_fail_soft_and_preserve_frozen_instruction() {
    let platforms = vec!["slack".to_string(), "github".to_string(), "linear".to_string()];
    let surfaced = surface_listener_connect_cards(
        &platforms,
        Some(|platform: &str| -> Result<bool, String> {
            match platform {
                "slack" => Ok(false),
                "github" => Ok(true),
                _ => Err("lookup failed".into()),
            }
        }),
        |platform| (platform == "slack").then(|| "Slack".to_string()),
    );
    assert_eq!(surfaced.cards.len(), 1);
    assert_eq!(surfaced.cards[0].platform, "slack");
    assert_eq!(surfaced.cards[0].reason, "so this routine can fire");
    let reminder = surfaced.reminder.expect("reminder");
    assert!(reminder.starts_with("Slack isn't connected"));
    assert!(reminder.contains("don't paste a link"));
    assert!(reminder.contains("resumed automatically"));

    let absent = surface_listener_connect_cards(
        &platforms,
        None::<fn(&str) -> Result<bool, String>>,
        |_| None,
    );
    assert!(absent.cards.is_empty());
    assert!(absent.reminder.is_none());
}

#[test]
fn communicate_encoding_wraps_marker_activity_success_and_error() {
    let mut payload = serde_json::Map::new();
    payload.insert("phase".into(), Value::String("executing".into()));
    payload.insert("tool".into(), Value::String("demo".into()));
    let encoded = encode_sand_step(&payload);
    let parsed: Value = serde_json::from_str(&encoded).expect("sand json");
    assert_eq!(parsed[SAND_TOOL_MARKER], true);
    assert_eq!(parsed["tool"], "demo");

    let wrapped = tool_call_wrapper(&payload);
    assert_eq!(
        wrapped["tool"]["case"],
        Value::String("communicateUpdateToolCall".into())
    );
    assert!(empty_tool_call()["tool"]["value"]["args"].is_null());

    let activity = executing_tool_call(
        "UploadFile",
        Some(&CommunicateActivity {
            detail: Some("report.pdf".into()),
            target: Some("Mac".into()),
        }),
    );
    let current_step = activity["tool"]["value"]["args"]["currentStep"]
        .as_str()
        .expect("currentStep");
    let decoded: Value = serde_json::from_str(current_step).expect("decoded");
    assert_eq!(decoded["detail"], "report.pdf");
    assert_eq!(decoded["target"], "Mac");

    let success = build_success_result("done");
    assert_eq!(render_result(&success), "done");
    let completed = completed_tool_call(&success);
    assert_eq!(
        completed["tool"]["value"]["result"]["result"]["case"],
        "success"
    );

    let empty = build_success_result("");
    assert_eq!(render_result(&empty), "Tool completed.");

    let error = build_error_result("boom");
    assert_eq!(render_result(&error), "Error: boom");
    assert!(encode_error("boom").contains("boom"));
    assert_eq!(
        run_communicate_execution(|| -> Result<String, &str> { Err("failed") }),
        CommunicateResult::Error {
            error: "failed".into()
        }
    );
    assert_eq!(
        serialize_error("serialized")["tool"]["value"]["result"]["result"]["case"],
        json!("error")
    );
}
