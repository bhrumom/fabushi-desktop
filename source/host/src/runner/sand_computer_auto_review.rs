use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use super::sand_auto_review::{
    SandAutoReviewController, SandAutoReviewDecision, SandAutoReviewMode,
    SandAutoReviewRequest, SandAutoReviewRequestOutcome, SandAutoReviewSurface,
    fingerprint_sand_auto_review_target, sand_auto_review_approval_expiry_policy,
};
use super::sand_auto_review_classifier_run::{
    AutoReviewClassifierDecision, AutoReviewClassifierError,
};
use super::sand_auto_review_summaries::summarize_sand_computer_typed_text;

pub const SAND_COMPUTER_CLASSIFIER_TARGET_ACTION: &str = "sand_computer";
pub const SAND_COMPUTER_AUTO_REVIEW_MAX_DESCRIPTION_CHARS: usize = 500;
pub const SAND_COMPUTER_AUTO_REVIEW_MAX_TEXT_CHARS: usize = 2_000;
pub const SAND_COMPUTER_AUTO_REVIEW_MAX_KEY_CHARS: usize = 256;
pub const SAND_COMPUTER_AUTO_REVIEW_MAX_PATH_POINTS: usize = 64;
pub const SAND_COMPUTER_AUTO_REVIEW_CLASSIFIER_ERROR_REASON: &str =
    "An error occurred while classifying this action. Please review manually.";
pub const SAND_COMPUTER_PAGE_STATE_CHROME_UNREACHABLE: &str = "chrome-unreachable";
pub const SAND_COMPUTER_AUTO_REVIEW_BYPASS_ACTIONS: &[&str] =
    &["screenshot", "move", "wait", "scroll"];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct SandComputerAutoReviewBlockedError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxIdentity {
    pub box_id: String,
    pub window_generation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstructionPermissions {
    pub allow_instructions: Vec<String>,
    pub block_instructions: Vec<String>,
}

pub fn compute_sand_computer_page_state_identity(stdout: &str) -> String {
    let mut lines = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    lines.sort_unstable();
    let mut hash = Sha256::new();
    hash.update(lines.join("\n").as_bytes());
    format!("{:x}", hash.finalize())
}

pub fn is_sand_computer_auto_review_bypass_action(action: &str) -> bool {
    SAND_COMPUTER_AUTO_REVIEW_BYPASS_ACTIONS.contains(&action)
}

pub fn is_sand_computer_auto_review_mutating_action(action: &str) -> bool {
    !is_sand_computer_auto_review_bypass_action(action)
}

pub fn requires_sand_computer_declared_description(action: &str) -> bool {
    matches!(action, "click" | "drag")
}

pub fn normalize_sand_computer_exact_action_args(raw: &Value) -> Result<Value, SandComputerAutoReviewBlockedError> {
    let object = raw
        .as_object()
        .ok_or_else(|| SandComputerAutoReviewBlockedError("Computer action must be an object.".into()))?;
    if object.get("text").and_then(Value::as_str).is_some_and(|value| value.chars().count() > SAND_COMPUTER_AUTO_REVIEW_MAX_TEXT_CHARS) {
        return Err(reject("text", SAND_COMPUTER_AUTO_REVIEW_MAX_TEXT_CHARS));
    }
    if object.get("key").and_then(Value::as_str).is_some_and(|value| value.chars().count() > SAND_COMPUTER_AUTO_REVIEW_MAX_KEY_CHARS) {
        return Err(reject("key", SAND_COMPUTER_AUTO_REVIEW_MAX_KEY_CHARS));
    }
    if object.get("path").and_then(Value::as_array).is_some_and(|value| value.len() > SAND_COMPUTER_AUTO_REVIEW_MAX_PATH_POINTS) {
        return Err(reject("path", SAND_COMPUTER_AUTO_REVIEW_MAX_PATH_POINTS));
    }
    Ok(raw.clone())
}

pub fn normalize_sand_computer_description(
    description: Option<&str>,
) -> Result<Option<String>, SandComputerAutoReviewBlockedError> {
    let Some(description) = description.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if description.chars().count() > SAND_COMPUTER_AUTO_REVIEW_MAX_DESCRIPTION_CHARS {
        return Err(reject(
            "description",
            SAND_COMPUTER_AUTO_REVIEW_MAX_DESCRIPTION_CHARS,
        ));
    }
    Ok(Some(description.to_string()))
}

pub fn build_sand_computer_auto_review_canonical_target(
    exact_action: &Value,
    description: Option<&str>,
    box_identity: &BoxIdentity,
    display_state_identity: &str,
) -> Result<Value, SandComputerAutoReviewBlockedError> {
    let action = normalize_sand_computer_exact_action_args(exact_action)?;
    let description = normalize_sand_computer_description(description)?;
    Ok(json!({
        "exact_action": action,
        "description": description,
        "box_identity": {
            "box_id": box_identity.box_id,
            "window_generation": box_identity.window_generation,
        },
        "display_state_identity": display_state_identity,
    }))
}

pub fn fingerprint_sand_computer_auto_review_target(target: &Value) -> String {
    fingerprint_sand_auto_review_target(target)
}

pub fn build_project_permissions_context(
    personal: Option<&InstructionPermissions>,
    user: Option<&InstructionPermissions>,
    project: Option<&InstructionPermissions>,
) -> Option<Value> {
    let mut allow = Vec::new();
    let mut block = Vec::new();
    for instructions in [personal, user, project].into_iter().flatten() {
        allow.extend(instructions.allow_instructions.iter().cloned());
        block.extend(instructions.block_instructions.iter().cloned());
    }
    if allow.is_empty() && block.is_empty() {
        None
    } else {
        Some(json!({
            "auto_run": {
                "allow_instructions": allow,
                "block_instructions": block,
            }
        }))
    }
}

pub fn build_sand_computer_classifier_risk_target(
    canonical_target: &Value,
    personal: Option<&InstructionPermissions>,
    user: Option<&InstructionPermissions>,
    project: Option<&InstructionPermissions>,
) -> Value {
    let action = canonical_target
        .get("exact_action")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut arguments = Map::new();
    arguments.insert("exact_action".into(), action);
    if let Some(description) = canonical_target.get("description").cloned().filter(|value| !value.is_null()) {
        arguments.insert("declared_purpose".into(), description);
    }
    if let Some(value) = canonical_target.pointer("/box_identity/box_id").cloned() {
        arguments.insert("box_id".into(), value);
    }
    if let Some(value) = canonical_target.pointer("/box_identity/window_generation").cloned() {
        arguments.insert("window_generation".into(), value);
    }
    if let Some(value) = canonical_target.get("display_state_identity").cloned() {
        arguments.insert("display_state_identity".into(), value);
    }
    if let Some(context) = build_project_permissions_context(personal, user, project) {
        arguments.insert("permissions".into(), context);
    }
    json!({
        "action": SAND_COMPUTER_CLASSIFIER_TARGET_ACTION,
        "arguments": Value::Object(arguments),
    })
}

pub fn summarize_blocked_action(
    canonical_target: &Value,
    fingerprint: &str,
    reason: &str,
) -> Value {
    let action = canonical_target
        .pointer("/exact_action/action")
        .and_then(Value::as_str)
        .unwrap_or("action");
    let purpose = canonical_target
        .get("description")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|value| format!(" to {}", value.chars().take(160).collect::<String>()))
        .unwrap_or_default();
    let exact = canonical_target.get("exact_action").unwrap_or(&Value::Null);
    let summary = match action {
        "click" => format!(
            "Click at ({}, {}) on Grok Bot's computer{purpose}",
            display_number(exact.get("x")),
            display_number(exact.get("y"))
        ),
        "drag" => format!(
            "Drag from ({}, {}) to ({}, {}) on Grok Bot's computer{purpose}",
            display_number(exact.get("x")),
            display_number(exact.get("y")),
            display_number(exact.get("x2")),
            display_number(exact.get("y2"))
        ),
        "type" => summarize_sand_computer_typed_text(
            exact.get("text").and_then(Value::as_str).unwrap_or_default(),
        ),
        "key" => format!(
            "Press {} on Grok Bot's computer{purpose}",
            exact.get("key").and_then(Value::as_str).unwrap_or("a key")
        ),
        other => format!("{other} on Grok Bot's computer{purpose}"),
    };
    json!({
        "surface": "computer",
        "fingerprint": fingerprint,
        "reason": reason,
        "summary": summary,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn run_sand_computer_auto_review_preflight<F, C>(
    mode: SandAutoReviewMode,
    exact_action: &Value,
    description: Option<&str>,
    box_identity: &BoxIdentity,
    agent_id: &str,
    request_source: &str,
    controller: Option<&SandAutoReviewController>,
    mut capture_display_state_identity: C,
    mut classify: F,
) -> Result<(), SandComputerAutoReviewBlockedError>
where
    F: FnMut(&Value, &str) -> Result<AutoReviewClassifierDecision, AutoReviewClassifierError>,
    C: FnMut() -> Result<String, String>,
{
    let action = exact_action
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| SandComputerAutoReviewBlockedError("Computer action is missing action.".into()))?;
    if mode == SandAutoReviewMode::Off || !is_sand_computer_auto_review_mutating_action(action) {
        return Ok(());
    }
    if mode == SandAutoReviewMode::Enforce
        && requires_sand_computer_declared_description(action)
        && normalize_sand_computer_description(description)?.is_none()
    {
        return Err(SandComputerAutoReviewBlockedError(
            "Computer click and drag actions require a declared purpose in Auto-review enforce mode.".into(),
        ));
    }
    let display_state_identity = capture_display_state_identity()
        .map_err(SandComputerAutoReviewBlockedError)?;
    let canonical = build_sand_computer_auto_review_canonical_target(
        exact_action,
        description,
        box_identity,
        &display_state_identity,
    )?;
    let risk_target = build_sand_computer_classifier_risk_target(&canonical, None, None, None);
    if mode == SandAutoReviewMode::Shadow {
        let _ = classify(&risk_target, "shadow");
        return Ok(());
    }
    let decision = classify(&risk_target, "enforce")
        .map_err(|error| SandComputerAutoReviewBlockedError(format!("{error:?}")))?;
    let (reason, proposed_rule) = match decision {
        AutoReviewClassifierDecision::Allow => return Ok(()),
        AutoReviewClassifierDecision::Reject { reason } => {
            return Err(SandComputerAutoReviewBlockedError(reason));
        }
        AutoReviewClassifierDecision::Block {
            reason,
            proposed_rule,
        } => (reason, proposed_rule),
    };
    let Some(controller) = controller else {
        return Err(SandComputerAutoReviewBlockedError(reason));
    };
    let fingerprint = fingerprint_sand_computer_auto_review_target(&canonical);
    let blocked = summarize_blocked_action(&canonical, &fingerprint, &reason);
    let request = SandAutoReviewRequest {
        agent_id: Some(agent_id.to_string()),
        surface: SandAutoReviewSurface::Computer,
        fingerprint,
        reason: reason.clone(),
        summary: blocked["summary"].as_str().unwrap_or("Computer action").to_string(),
        command: None,
        proposed_rule,
        expiry_policy: Some(sand_auto_review_approval_expiry_policy(request_source)),
    };
    let decision = match controller.request_approval(request) {
        SandAutoReviewRequestOutcome::Immediate(decision) => decision,
        SandAutoReviewRequestOutcome::Pending(pending) => pending
            .wait()
            .map_err(|error| SandComputerAutoReviewBlockedError(error.to_string()))?,
    };
    match decision {
        SandAutoReviewDecision::Denied { reason } => Err(SandComputerAutoReviewBlockedError(reason)),
        SandAutoReviewDecision::Approved => {
            let next = capture_display_state_identity()
                .map_err(SandComputerAutoReviewBlockedError)?;
            if next != display_state_identity {
                controller.report_display_recheck_failed(Some(agent_id));
                return Err(SandComputerAutoReviewBlockedError(
                    "Computer display changed while approval was pending; retry the action.".into(),
                ));
            }
            Ok(())
        }
    }
}

fn reject(field: &str, max: usize) -> SandComputerAutoReviewBlockedError {
    SandComputerAutoReviewBlockedError(format!(
        "Computer Auto-review rejected oversized {field} (max {max} characters)."
    ))
}

fn display_number(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_f64)
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{}", value as i64)
            } else {
                value.to_string()
            }
        })
        .unwrap_or_else(|| "?".into())
}
