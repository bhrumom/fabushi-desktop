use serde_json::{Value, json};

use super::sand_auto_review::{
    SandAutoReviewController, SandAutoReviewDecision, SandAutoReviewExpiryPolicy,
    SandAutoReviewMode, SandAutoReviewRequest, SandAutoReviewRequestOutcome,
    SandAutoReviewSurface, fingerprint_sand_auto_review_target,
};
use super::sand_auto_review_classifier_run::{
    AutoReviewClassifierDecision, AutoReviewClassifierError,
};
use super::sand_auto_review_summaries::summarize_sand_automation_write_action;

pub const SAND_AUTOMATION_WRITE_CLASSIFIER_TARGET_ACTION: &str = "sand_automation_write";
pub const SAND_AUTOMATION_WRITE_CLASSIFIER_ERROR_REASON: &str =
    "An error occurred while classifying this automation change. Please review manually.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationReference {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationWriteTarget {
    pub operation: String,
    pub name: String,
    pub trigger_description: String,
    pub prompt: String,
    pub is_enabled: Option<bool>,
    pub referencing_routines: Vec<AutomationReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationReviewOutcome {
    Allowed,
    Blocked(String),
}

pub fn build_sand_automation_write_risk_target(target: &AutomationWriteTarget) -> Value {
    json!({
        "action": SAND_AUTOMATION_WRITE_CLASSIFIER_TARGET_ACTION,
        "arguments": {
            "operation": target.operation,
            "name": target.name,
            "trigger_description": target.trigger_description,
            "prompt": target.prompt,
            "is_enabled": target.is_enabled,
            "referencing_routines": target.referencing_routines.iter().map(|routine| json!({
                "id": routine.id,
                "name": routine.name,
            })).collect::<Vec<_>>(),
        }
    })
}

pub fn review_sand_automation_write<F>(
    mode: SandAutoReviewMode,
    target: &AutomationWriteTarget,
    controller: Option<&SandAutoReviewController>,
    request_source: &str,
    mut classify: F,
) -> Result<AutomationReviewOutcome, AutoReviewClassifierError>
where
    F: FnMut(&Value, &str) -> Result<AutoReviewClassifierDecision, AutoReviewClassifierError>,
{
    if mode == SandAutoReviewMode::Off {
        return Ok(AutomationReviewOutcome::Allowed);
    }
    let risk_target = build_sand_automation_write_risk_target(target);
    if mode == SandAutoReviewMode::Shadow {
        let _ = classify(&risk_target, "shadow");
        return Ok(AutomationReviewOutcome::Allowed);
    }
    let decision = classify(&risk_target, "enforce")?;
    match decision {
        AutoReviewClassifierDecision::Allow => Ok(AutomationReviewOutcome::Allowed),
        AutoReviewClassifierDecision::Reject { reason } => {
            Ok(AutomationReviewOutcome::Blocked(reason))
        }
        AutoReviewClassifierDecision::Block {
            reason,
            proposed_rule,
        } => {
            let Some(controller) = controller else {
                return Ok(AutomationReviewOutcome::Blocked(reason));
            };
            let summary = summarize_sand_automation_write_action(
                &target.operation,
                &target.name,
                &target.trigger_description,
                &target.prompt,
                target.is_enabled,
                &target
                    .referencing_routines
                    .iter()
                    .map(|routine| routine.name.clone())
                    .collect::<Vec<_>>(),
            );
            let request = SandAutoReviewRequest {
                agent_id: None,
                surface: SandAutoReviewSurface::AutomationWrite,
                fingerprint: fingerprint_sand_auto_review_target(&risk_target),
                reason: reason.clone(),
                summary,
                command: None,
                proposed_rule,
                expiry_policy: Some(super::sand_auto_review::sand_auto_review_approval_expiry_policy(
                    request_source,
                )),
            };
            settle(controller.request_approval(request), reason)
        }
    }
}

fn settle(
    outcome: SandAutoReviewRequestOutcome,
    classifier_reason: String,
) -> Result<AutomationReviewOutcome, AutoReviewClassifierError> {
    let decision = match outcome {
        SandAutoReviewRequestOutcome::Immediate(decision) => decision,
        SandAutoReviewRequestOutcome::Pending(pending) => pending.wait().map_err(|error| {
            AutoReviewClassifierError::Failed(format!(
                "Auto-review approval channel closed: {error}"
            ))
        })?,
    };
    Ok(match decision {
        SandAutoReviewDecision::Approved => AutomationReviewOutcome::Allowed,
        SandAutoReviewDecision::Denied { reason } => {
            AutomationReviewOutcome::Blocked(if reason.is_empty() {
                classifier_reason
            } else {
                reason
            })
        }
    })
}

pub fn default_automation_approval_expiry_policy(source: &str) -> SandAutoReviewExpiryPolicy {
    super::sand_auto_review::sand_auto_review_approval_expiry_policy(source)
}
