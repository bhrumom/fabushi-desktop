use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::runner::sand_auto_review::{
    SandAutoReviewMode,
};
use crate::runner::sand_computer_auto_review::is_sand_computer_auto_review_bypass_action;

pub const SAND_COMPUTER_MAX_WAIT_MS: u64 = 30_000;
pub const SAND_COMPUTER_MAX_FOLLOW_UP_ACTIONS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputerActionName {
    Screenshot,
    Click,
    Move,
    Drag,
    Type,
    Key,
    Scroll,
    Wait,
}

impl ComputerActionName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Screenshot => "screenshot",
            Self::Click => "click",
            Self::Move => "move",
            Self::Drag => "drag",
            Self::Type => "type",
            Self::Key => "key",
            Self::Scroll => "scroll",
            Self::Wait => "wait",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonInput {
    Left,
    Right,
    Middle,
}

impl MouseButtonInput {
    fn generated_name(self) -> &'static str {
        match self {
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Middle => "MIDDLE",
        }
    }

    fn user_name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirectionInput {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirectionInput {
    fn generated_name(self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputerCoordinate {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerActionArgs {
    pub action: ComputerActionName,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub x2: Option<i64>,
    pub y2: Option<i64>,
    pub path: Vec<ComputerCoordinate>,
    pub text: Option<String>,
    pub key: Option<String>,
    pub button: Option<MouseButtonInput>,
    pub count: Option<u8>,
    pub direction: Option<ScrollDirectionInput>,
    pub amount: Option<i64>,
    pub duration_ms: Option<u64>,
    pub description: Option<String>,
    pub then_actions: Vec<ComputerActionArgs>,
}

impl ComputerActionArgs {
    pub fn simple(action: ComputerActionName) -> Self {
        Self {
            action,
            x: None,
            y: None,
            x2: None,
            y2: None,
            path: Vec::new(),
            text: None,
            key: None,
            button: None,
            count: None,
            direction: None,
            amount: None,
            duration_ms: None,
            description: None,
            then_actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct SandComputerToolInputError(pub String);

pub fn drag_path(args: &ComputerActionArgs) -> Option<Vec<ComputerCoordinate>> {
    if args.path.len() >= 2 {
        return Some(args.path.clone());
    }
    Some(vec![
        ComputerCoordinate {
            x: args.x?,
            y: args.y?,
        },
        ComputerCoordinate {
            x: args.x2?,
            y: args.y2?,
        },
    ])
}

pub fn validate_computer_action(
    args: &ComputerActionArgs,
    auto_review_mode: Option<SandAutoReviewMode>,
) -> Result<(), SandComputerToolInputError> {
    if args.then_actions.len() > SAND_COMPUTER_MAX_FOLLOW_UP_ACTIONS {
        return Err(SandComputerToolInputError(format!(
            "Computer then supports at most {SAND_COMPUTER_MAX_FOLLOW_UP_ACTIONS} follow-up actions."
        )));
    }
    validate_single_action(args, auto_review_mode, false)?;
    for follow_up in &args.then_actions {
        if follow_up.action == ComputerActionName::Screenshot {
            return Err(SandComputerToolInputError(
                "Screenshot is not allowed inside then; the tool captures one automatically at the end."
                    .to_string(),
            ));
        }
        if auto_review_mode == Some(SandAutoReviewMode::Enforce)
            && !is_sand_computer_auto_review_bypass_action(follow_up.action.as_str())
        {
            return Err(SandComputerToolInputError(format!(
                "{} is not allowed inside then while Computer Auto-review is enforcing.",
                follow_up.action.as_str()
            )));
        }
        validate_single_action(follow_up, auto_review_mode, true)?;
    }
    Ok(())
}

fn validate_single_action(
    args: &ComputerActionArgs,
    auto_review_mode: Option<SandAutoReviewMode>,
    _follow_up: bool,
) -> Result<(), SandComputerToolInputError> {
    if args.action == ComputerActionName::Drag && drag_path(args).is_none() {
        return Err(SandComputerToolInputError(
            "Drag requires x, y, x2, and y2 or a path with at least 2 points.".to_string(),
        ));
    }
    if args.action == ComputerActionName::Wait
        && args.duration_ms.unwrap_or(1_000) > SAND_COMPUTER_MAX_WAIT_MS
    {
        return Err(SandComputerToolInputError(format!(
            "Wait duration must be at most {SAND_COMPUTER_MAX_WAIT_MS}ms."
        )));
    }
    if args.action == ComputerActionName::Click
        && !matches!(args.count.unwrap_or(1), 1..=3)
    {
        return Err(SandComputerToolInputError(
            "Click count must be between 1 and 3.".to_string(),
        ));
    }
    if auto_review_mode == Some(SandAutoReviewMode::Enforce)
        && matches!(args.action, ComputerActionName::Click | ComputerActionName::Drag)
        && args
            .description
            .as_deref()
            .map(str::trim)
            .is_none_or(str::is_empty)
    {
        return Err(SandComputerToolInputError(
            "Click and drag require description: a concise statement of the intended UI target and purpose."
                .to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputerProtocolAction {
    pub action_case: String,
    pub value: Value,
}

fn coordinate(args: &ComputerActionArgs) -> Option<Value> {
    Some(json!({
        "x": args.x?,
        "y": args.y?,
    }))
}

pub fn to_action(args: &ComputerActionArgs) -> Result<ComputerProtocolAction, SandComputerToolInputError> {
    let (action_case, value) = match args.action {
        ComputerActionName::Screenshot => ("screenshot", json!({})),
        ComputerActionName::Click => (
            "click",
            json!({
                "coordinate": coordinate(args),
                "button": args.button.unwrap_or(MouseButtonInput::Left).generated_name(),
                "count": args.count.unwrap_or(1),
            }),
        ),
        ComputerActionName::Move => (
            "mouseMove",
            json!({"coordinate": coordinate(args)}),
        ),
        ComputerActionName::Drag => {
            let path = drag_path(args).ok_or_else(|| {
                SandComputerToolInputError(
                    "Drag requires x, y, x2, and y2 or a path with at least 2 points.".into(),
                )
            })?;
            (
                "drag",
                json!({
                    "path": path.iter().map(|point| json!({"x":point.x,"y":point.y})).collect::<Vec<_>>(),
                    "button": args.button.unwrap_or(MouseButtonInput::Left).generated_name(),
                }),
            )
        }
        ComputerActionName::Type => ("type", json!({"text":args.text.clone().unwrap_or_default()})),
        ComputerActionName::Key => ("key", json!({"key":args.key.clone().unwrap_or_default()})),
        ComputerActionName::Scroll => (
            "scroll",
            json!({
                "coordinate": coordinate(args),
                "direction": args.direction.unwrap_or(ScrollDirectionInput::Down).generated_name(),
                "amount": args.amount.unwrap_or(3),
            }),
        ),
        ComputerActionName::Wait => (
            "wait",
            json!({"durationMs":args.duration_ms.unwrap_or(1_000)}),
        ),
    };
    Ok(ComputerProtocolAction {
        action_case: action_case.to_string(),
        value,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportedComputerAction {
    Drag { x: i64, y: i64 },
    Move { x: i64, y: i64 },
    Scroll { x: i64, y: i64 },
    Click {
        x: i64,
        y: i64,
        button: String,
        count: u8,
    },
}

pub fn to_reported_action(args: &ComputerActionArgs) -> Option<ReportedComputerAction> {
    match args.action {
        ComputerActionName::Drag => {
            let first = drag_path(args)?.first().copied()?;
            Some(ReportedComputerAction::Drag {
                x: first.x,
                y: first.y,
            })
        }
        ComputerActionName::Move => Some(ReportedComputerAction::Move {
            x: args.x?,
            y: args.y?,
        }),
        ComputerActionName::Scroll => Some(ReportedComputerAction::Scroll {
            x: args.x?,
            y: args.y?,
        }),
        ComputerActionName::Click => Some(ReportedComputerAction::Click {
            x: args.x?,
            y: args.y?,
            button: args.button.unwrap_or(MouseButtonInput::Left).user_name().to_string(),
            count: args.count.unwrap_or(1),
        }),
        _ => None,
    }
}

pub fn reported_batch_position(sequence: &[ComputerActionArgs]) -> Option<ReportedComputerAction> {
    let positions = sequence
        .iter()
        .filter_map(to_reported_action)
        .collect::<Vec<_>>();
    positions
        .iter()
        .find(|position| !matches!(position, ReportedComputerAction::Drag { .. }))
        .cloned()
        .or_else(|| positions.first().cloned())
}

pub fn build_computer_action_sequence(
    args: &ComputerActionArgs,
    auto_review_mode: Option<SandAutoReviewMode>,
) -> Result<Vec<ComputerProtocolAction>, SandComputerToolInputError> {
    validate_computer_action(args, auto_review_mode)?;
    let mut sequence = vec![args.clone()];
    sequence.extend(args.then_actions.clone());
    let mut actions = sequence
        .iter()
        .map(to_action)
        .collect::<Result<Vec<_>, _>>()?;
    if sequence
        .last()
        .is_none_or(|action| action.action != ComputerActionName::Screenshot)
    {
        actions.push(to_action(&ComputerActionArgs::simple(
            ComputerActionName::Screenshot,
        ))?);
    }
    Ok(actions)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputerUseSuccess {
    pub screenshot: Option<String>,
    pub screenshot_path: Option<String>,
    pub cursor_position: Option<ComputerCoordinate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputerUseResult {
    Success(ComputerUseSuccess),
    Error(String),
    Other(String),
}

pub fn describe_outcome(result: &ComputerUseResult, screenshot: bool) -> String {
    if let ComputerUseResult::Error(error) = result {
        return format!(
            "{} failed: {error}",
            if screenshot { "Screenshot" } else { "Computer action" }
        );
    }
    let heading = if screenshot {
        "Screenshot captured from the box desktop."
    } else {
        "Computer action ran on the box desktop."
    };
    let ComputerUseResult::Success(success) = result else {
        return heading.to_string();
    };
    let mut lines = vec![heading.to_string()];
    if let Some(path) = success
        .screenshot_path
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("Screenshot saved to {path}."));
    }
    if let Some(position) = success.cursor_position {
        lines.push(format!("Cursor is at ({}, {}).", position.x, position.y));
    }
    lines.join("\n")
}

pub fn persist_computer_screenshot<Persist>(
    result: &mut ComputerUseResult,
    mut persist: Option<Persist>,
) -> Result<(), SandComputerToolInputError>
where
    Persist: FnMut(&[u8], &str) -> Option<String>,
{
    let ComputerUseResult::Success(success) = result else {
        return Ok(());
    };
    let Some(encoded) = success.screenshot.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let Some(persist) = persist.as_mut() else {
        return Ok(());
    };
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        SandComputerToolInputError(format!("invalid screenshot base64: {error}"))
    })?;
    if let Some(path) = persist(&bytes, "image/webp") {
        success.screenshot_path = Some(path);
    }
    Ok(())
}
