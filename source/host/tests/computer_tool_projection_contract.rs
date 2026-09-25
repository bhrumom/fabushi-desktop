use std::cell::RefCell;

use mahayana_host_runtime::r#box::box_shell_command::HostShellArgsInput;
use mahayana_host_runtime::runner::host_computer_tool_dependencies::{
    ComputerProjectionError, GeneratedComputerAction, GeneratedComputerUseResult,
    execute_host_shell, from_generated_computer_use_result,
    resolve_browser_window_index, to_generated_computer_use_args,
};
use mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewMode;
use mahayana_host_runtime::runner::tools::sand_computer_tool::{
    ComputerActionArgs, ComputerActionName, ComputerCoordinate, ComputerUseResult,
    MouseButtonInput, ReportedComputerAction, ScrollDirectionInput,
    build_computer_action_sequence, describe_outcome, drag_path,
    persist_computer_screenshot, reported_batch_position, to_action,
    validate_computer_action,
};

#[test]
fn computer_schema_core_validates_drag_wait_followups_and_enforce_description() {
    let mut drag = ComputerActionArgs::simple(ComputerActionName::Drag);
    assert!(validate_computer_action(&drag, None).is_err());
    drag.x = Some(1);
    drag.y = Some(2);
    drag.x2 = Some(3);
    drag.y2 = Some(4);
    assert_eq!(
        drag_path(&drag),
        Some(vec![
            ComputerCoordinate { x: 1, y: 2 },
            ComputerCoordinate { x: 3, y: 4 }
        ])
    );
    assert!(validate_computer_action(&drag, Some(SandAutoReviewMode::Enforce)).is_err());
    drag.description = Some("Drag the selected card".into());
    assert!(validate_computer_action(&drag, Some(SandAutoReviewMode::Enforce)).is_ok());

    let mut wait = ComputerActionArgs::simple(ComputerActionName::Wait);
    wait.duration_ms = Some(30_001);
    assert!(validate_computer_action(&wait, None).is_err());

    let mut primary = ComputerActionArgs::simple(ComputerActionName::Move);
    primary.then_actions.push(ComputerActionArgs::simple(ComputerActionName::Click));
    assert!(validate_computer_action(&primary, Some(SandAutoReviewMode::Enforce)).is_err());
}

#[test]
fn actions_and_batch_reporting_match_frozen_defaults() {
    let mut click = ComputerActionArgs::simple(ComputerActionName::Click);
    click.x = Some(10);
    click.y = Some(20);
    let projected = to_action(&click).expect("click");
    assert_eq!(projected.action_case, "click");
    assert_eq!(projected.value["button"], "LEFT");
    assert_eq!(projected.value["count"], 1);

    let mut scroll = ComputerActionArgs::simple(ComputerActionName::Scroll);
    scroll.x = Some(5);
    scroll.y = Some(6);
    scroll.direction = Some(ScrollDirectionInput::Up);
    scroll.amount = Some(4);
    let position = reported_batch_position(&[
        {
            let mut drag = ComputerActionArgs::simple(ComputerActionName::Drag);
            drag.path = vec![
                ComputerCoordinate { x: 1, y: 2 },
                ComputerCoordinate { x: 3, y: 4 },
            ];
            drag
        },
        scroll.clone(),
    ])
    .expect("reported");
    assert_eq!(position, ReportedComputerAction::Scroll { x: 5, y: 6 });

    click.button = Some(MouseButtonInput::Right);
    click.count = Some(2);
    let action = to_action(&click).expect("right click");
    assert_eq!(action.value["button"], "RIGHT");
    assert_eq!(action.value["count"], 2);
}

#[test]
fn computer_sequence_adds_exactly_one_final_screenshot() {
    let mut primary = ComputerActionArgs::simple(ComputerActionName::Type);
    primary.text = Some("hello".into());
    primary.then_actions.push(ComputerActionArgs::simple(ComputerActionName::Wait));
    let sequence = build_computer_action_sequence(&primary, None).expect("sequence");
    assert_eq!(sequence.len(), 3);
    assert_eq!(sequence[0].action_case, "type");
    assert_eq!(sequence[1].action_case, "wait");
    assert_eq!(sequence[2].action_case, "screenshot");

    let screenshot = build_computer_action_sequence(
        &ComputerActionArgs::simple(ComputerActionName::Screenshot),
        None,
    )
    .expect("screenshot");
    assert_eq!(screenshot.len(), 1);
}

#[test]
fn outcome_render_and_screenshot_persistence_match_frozen_contract() {
    let mut result = ComputerUseResult::Success(
        mahayana_host_runtime::runner::tools::sand_computer_tool::ComputerUseSuccess {
            screenshot: Some("AQID".into()),
            screenshot_path: None,
            cursor_position: Some(ComputerCoordinate { x: 9, y: 8 }),
        },
    );
    persist_computer_screenshot(
        &mut result,
        Some(|bytes: &[u8], mime: &str| {
            assert_eq!(bytes, &[1, 2, 3]);
            assert_eq!(mime, "image/webp");
            Some("file:///saved.webp".to_string())
        }),
    )
    .expect("persist");
    let rendered = describe_outcome(&result, false);
    assert!(rendered.contains("Screenshot saved to file:///saved.webp."));
    assert!(rendered.contains("Cursor is at (9, 8)."));

    assert_eq!(
        describe_outcome(&ComputerUseResult::Error("boom".into()), true),
        "Screenshot failed: boom"
    );
}

#[test]
fn host_projection_validates_protocol_and_normalizes_generated_results() {
    let mut click = ComputerActionArgs::simple(ComputerActionName::Click);
    click.x = Some(11);
    click.y = Some(12);
    click.button = Some(MouseButtonInput::Middle);
    let protocol = to_action(&click).expect("protocol");
    let generated = to_generated_computer_use_args(
        "tool-1",
        &[protocol],
        Some(true),
        Some("target"),
    )
    .expect("generated");
    assert_eq!(generated.tool_call_id, "tool-1");
    assert_eq!(generated.bind_unmapped_characters, Some(true));
    assert!(matches!(
        &generated.actions[0],
        GeneratedComputerAction::Click { button, count, .. }
            if button == "MIDDLE" && *count == 1.0
    ));

    let invalid = mahayana_host_runtime::runner::tools::sand_computer_tool::ComputerProtocolAction {
        action_case: "click".into(),
        value: serde_json::json!({"button":"BOGUS","count":1}),
    };
    assert!(matches!(
        to_generated_computer_use_args("", &[invalid], None, None),
        Err(ComputerProjectionError(_))
    ));

    let normalized = from_generated_computer_use_result(
        GeneratedComputerUseResult::Success {
            screenshot: Some("abc".into()),
            cursor_position: Some(ComputerCoordinate { x: 1, y: 2 }),
        },
    );
    assert!(matches!(normalized, ComputerUseResult::Success(_)));
}

#[test]
fn host_shell_projection_runs_barrier_then_audit_then_generated_executor() {
    let order = RefCell::new(Vec::<String>::new());
    let result = execute_host_shell(
        HostShellArgsInput {
            command: "pwd".into(),
            name: "pwd".into(),
            working_directory: "/workspace".into(),
            tool_call_id: "tool".into(),
        },
        || order.borrow_mut().push("barrier".into()),
        |command| order.borrow_mut().push(format!("audit:{command}")),
        |args| {
            order.borrow_mut().push(format!("execute:{}", args.command));
            assert!(args.skip_approval);
            42
        },
    );
    assert_eq!(result, 42);
    assert_eq!(
        order.into_inner(),
        vec!["barrier", "audit:pwd", "execute:pwd"]
    );
}

#[test]
fn browser_identity_projection_waits_for_box_before_window_lookup() {
    let order = RefCell::new(Vec::<String>::new());
    let index = resolve_browser_window_index(
        &(),
        "box-7",
        |_, box_id| -> Result<(), &'static str> {
            order.borrow_mut().push(format!("ready:{box_id}"));
            Ok(())
        },
        |box_id| {
            order.borrow_mut().push(format!("window:{box_id}"));
            Some(3)
        },
    )
    .expect("window");
    assert_eq!(index, Some(3));
    assert_eq!(
        order.into_inner(),
        vec!["ready:box-7", "window:box-7"]
    );
}
