use serde_json::Value;

use crate::r#box::box_shell_command::{
    HostShellArgs, HostShellArgsInput, build_host_shell_args,
};

use super::tools::sand_computer_tool::{
    ComputerCoordinate, ComputerProtocolAction, ComputerUseResult, ComputerUseSuccess,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedComputerUseArgs {
    pub tool_call_id: String,
    pub actions: Vec<GeneratedComputerAction>,
    pub bind_unmapped_characters: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedComputerAction {
    MouseMove { coordinate: Option<ComputerCoordinate> },
    Click {
        coordinate: Option<ComputerCoordinate>,
        button: String,
        count: f64,
        modifier_keys: Option<String>,
    },
    MouseDown { button: String },
    MouseUp { button: String },
    Drag {
        path: Vec<ComputerCoordinate>,
        button: String,
        modifier_keys: Option<String>,
    },
    Scroll {
        coordinate: Option<ComputerCoordinate>,
        direction: String,
        amount: f64,
        modifier_keys: Option<String>,
    },
    Type { text: String },
    Key { key: String, hold_duration_ms: Option<f64> },
    Wait { duration_ms: f64 },
    Screenshot,
    CursorPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ComputerProjectionError(pub String);

fn record<'a>(value: &'a Value, action_case: &str) -> Result<&'a serde_json::Map<String, Value>, ComputerProjectionError> {
    value.as_object().ok_or_else(|| {
        ComputerProjectionError(format!("computer action {action_case} has no value"))
    })
}

fn required_number(
    value: Option<&Value>,
    field: &str,
) -> Result<f64, ComputerProjectionError> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| ComputerProjectionError(format!("computer action field {field} is not numeric")))
}

fn required_string(
    value: Option<&Value>,
    field: &str,
) -> Result<String, ComputerProjectionError> {
    value
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| ComputerProjectionError(format!("computer action field {field} is not textual")))
}

fn optional_coordinate(value: Option<&Value>) -> Result<Option<ComputerCoordinate>, ComputerProjectionError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let object = record(value, "coordinate")?;
    Ok(Some(ComputerCoordinate {
        x: required_number(object.get("x"), "coordinate.x")? as i64,
        y: required_number(object.get("y"), "coordinate.y")? as i64,
    }))
}

fn button(value: Option<&Value>) -> Result<String, ComputerProjectionError> {
    let value = required_string(value, "button")?;
    if matches!(value.as_str(), "LEFT" | "RIGHT" | "MIDDLE" | "BACK" | "FORWARD") {
        Ok(value)
    } else {
        Err(ComputerProjectionError(format!(
            "unknown computer action button: {value}"
        )))
    }
}

fn direction(value: Option<&Value>) -> Result<String, ComputerProjectionError> {
    let value = required_string(value, "direction")?;
    if matches!(value.as_str(), "UP" | "DOWN" | "LEFT" | "RIGHT") {
        Ok(value)
    } else {
        Err(ComputerProjectionError(format!(
            "unknown computer action direction: {value}"
        )))
    }
}

pub fn generated_action(
    action: &ComputerProtocolAction,
) -> Result<GeneratedComputerAction, ComputerProjectionError> {
    let value = record(&action.value, &action.action_case)?;
    match action.action_case.as_str() {
        "mouseMove" => Ok(GeneratedComputerAction::MouseMove {
            coordinate: optional_coordinate(value.get("coordinate"))?,
        }),
        "click" => Ok(GeneratedComputerAction::Click {
            coordinate: optional_coordinate(value.get("coordinate"))?,
            button: button(value.get("button"))?,
            count: required_number(value.get("count"), "count")?,
            modifier_keys: value
                .get("modifierKeys")
                .map(|value| required_string(Some(value), "modifierKeys"))
                .transpose()?,
        }),
        "mouseDown" => Ok(GeneratedComputerAction::MouseDown {
            button: button(value.get("button"))?,
        }),
        "mouseUp" => Ok(GeneratedComputerAction::MouseUp {
            button: button(value.get("button"))?,
        }),
        "drag" => {
            let raw = value
                .get("path")
                .and_then(Value::as_array)
                .ok_or_else(|| ComputerProjectionError("computer action drag path is not an array".into()))?;
            let mut path = Vec::with_capacity(raw.len());
            for point in raw {
                path.push(optional_coordinate(Some(point))?.ok_or_else(|| {
                    ComputerProjectionError("computer action drag path contains no coordinate".into())
                })?);
            }
            Ok(GeneratedComputerAction::Drag {
                path,
                button: button(value.get("button"))?,
                modifier_keys: value
                    .get("modifierKeys")
                    .map(|value| required_string(Some(value), "modifierKeys"))
                    .transpose()?,
            })
        }
        "scroll" => Ok(GeneratedComputerAction::Scroll {
            coordinate: optional_coordinate(value.get("coordinate"))?,
            direction: direction(value.get("direction"))?,
            amount: required_number(value.get("amount"), "amount")?,
            modifier_keys: value
                .get("modifierKeys")
                .map(|value| required_string(Some(value), "modifierKeys"))
                .transpose()?,
        }),
        "type" => Ok(GeneratedComputerAction::Type {
            text: required_string(value.get("text"), "text")?,
        }),
        "key" => Ok(GeneratedComputerAction::Key {
            key: required_string(value.get("key"), "key")?,
            hold_duration_ms: value
                .get("holdDurationMs")
                .map(|value| required_number(Some(value), "holdDurationMs"))
                .transpose()?,
        }),
        "wait" => Ok(GeneratedComputerAction::Wait {
            duration_ms: required_number(value.get("durationMs"), "durationMs")?,
        }),
        "screenshot" => Ok(GeneratedComputerAction::Screenshot),
        "cursorPosition" => Ok(GeneratedComputerAction::CursorPosition),
        other => Err(ComputerProjectionError(format!(
            "unsupported computer action: {other}"
        ))),
    }
}

pub fn to_generated_computer_use_args(
    tool_call_id: &str,
    actions: &[ComputerProtocolAction],
    bind_unmapped_characters: Option<bool>,
    description: Option<&str>,
) -> Result<GeneratedComputerUseArgs, ComputerProjectionError> {
    Ok(GeneratedComputerUseArgs {
        tool_call_id: tool_call_id.to_string(),
        actions: actions
            .iter()
            .map(generated_action)
            .collect::<Result<Vec<_>, _>>()?,
        bind_unmapped_characters,
        description: description.map(ToOwned::to_owned),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedComputerUseResult {
    Success {
        screenshot: Option<String>,
        cursor_position: Option<ComputerCoordinate>,
    },
    Error(String),
    Other(String),
}

pub fn from_generated_computer_use_result(
    result: GeneratedComputerUseResult,
) -> ComputerUseResult {
    match result {
        GeneratedComputerUseResult::Success {
            screenshot,
            cursor_position,
        } => ComputerUseResult::Success(ComputerUseSuccess {
            screenshot,
            screenshot_path: None,
            cursor_position,
        }),
        GeneratedComputerUseResult::Error(error) => ComputerUseResult::Error(error),
        GeneratedComputerUseResult::Other(case) => ComputerUseResult::Other(case),
    }
}

pub fn execute_host_shell<ResultType, Assert, Audit, Execute>(
    args: HostShellArgsInput,
    mut assert_no_pending_approval: Assert,
    mut audit_shell_command: Audit,
    execute: Execute,
) -> ResultType
where
    Assert: FnMut(),
    Audit: FnMut(&str),
    Execute: FnOnce(HostShellArgs) -> ResultType,
{
    assert_no_pending_approval();
    audit_shell_command(&args.command);
    execute(build_host_shell_args(args))
}

pub fn resolve_browser_window_index<Context, ErrorType, Ensure, Lookup>(
    context: &Context,
    box_id: &str,
    ensure_ready: Ensure,
    lookup: Lookup,
) -> Result<Option<u32>, ErrorType>
where
    Ensure: FnOnce(&Context, &str) -> Result<(), ErrorType>,
    Lookup: FnOnce(&str) -> Option<u32>,
{
    ensure_ready(context, box_id)?;
    Ok(lookup(box_id))
}
