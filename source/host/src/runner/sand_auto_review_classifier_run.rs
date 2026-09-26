pub const SAND_AUTO_REVIEW_BLOCK_REASON: &str = "Blocked by Auto-review";
pub const SAND_AUTO_REVIEW_CLASSIFIER_MAX_ATTEMPTS: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoReviewClassifierDecision {
    Allow,
    Block {
        reason: String,
        proposed_rule: Option<String>,
    },
    Reject {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartModeClassifierDecision {
    Allow,
    Block,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartModeClassifierSuccess {
    pub decision: SmartModeClassifierDecision,
    pub block_reason: Option<String>,
    pub proposed_allow_rule: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartModeClassifierResult {
    Success(SmartModeClassifierSuccess),
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoReviewClassifierError {
    Aborted(String),
    Failed(String),
}

pub struct AutoReviewClassifierRequest<'a, Target, Message> {
    pub tool_call_id: &'a str,
    pub parent_conversation_id: &'a str,
    pub mode: &'a str,
    pub target: Target,
    pub conversation_context: Vec<Message>,
    pub workspace_paths: &'a [String],
    pub suppress_tool_call_id_logging: bool,
    pub max_attempts: u32,
}

pub trait SandAutoReviewClassifierExecutor<Target, Message> {
    fn execute(
        &mut self,
        request: AutoReviewClassifierRequest<'_, Target, Message>,
    ) -> Result<SmartModeClassifierResult, AutoReviewClassifierError>;
}

pub fn run_sand_auto_review_classifier<Target, Message>(
    executor: &mut dyn SandAutoReviewClassifierExecutor<Target, Message>,
    tool_call_id: &str,
    parent_conversation_id: &str,
    mode: &str,
    build_target: impl FnOnce() -> Target,
    load_conversation_context: impl FnOnce() -> Result<Vec<Message>, AutoReviewClassifierError>,
    workspace_paths: &[String],
    error_reason: &str,
) -> Result<AutoReviewClassifierDecision, AutoReviewClassifierError> {
    let conversation_context = match load_conversation_context() {
        Ok(context) => context,
        Err(AutoReviewClassifierError::Aborted(reason)) => {
            return Err(AutoReviewClassifierError::Aborted(reason));
        }
        Err(AutoReviewClassifierError::Failed(_)) => {
            return Ok(AutoReviewClassifierDecision::Reject {
                reason: error_reason.to_string(),
            });
        }
    };

    let result = match executor.execute(AutoReviewClassifierRequest {
        tool_call_id,
        parent_conversation_id,
        mode,
        target: build_target(),
        conversation_context,
        workspace_paths,
        suppress_tool_call_id_logging: true,
        max_attempts: SAND_AUTO_REVIEW_CLASSIFIER_MAX_ATTEMPTS,
    }) {
        Ok(result) => result,
        Err(AutoReviewClassifierError::Aborted(reason)) => {
            return Err(AutoReviewClassifierError::Aborted(reason));
        }
        Err(AutoReviewClassifierError::Failed(_)) => {
            return Ok(AutoReviewClassifierDecision::Reject {
                reason: error_reason.to_string(),
            });
        }
    };

    let SmartModeClassifierResult::Success(success) = result else {
        return Ok(AutoReviewClassifierDecision::Reject {
            reason: error_reason.to_string(),
        });
    };

    match success.decision {
        SmartModeClassifierDecision::Allow => Ok(AutoReviewClassifierDecision::Allow),
        SmartModeClassifierDecision::Block => {
            let reason = success
                .block_reason
                .as_deref()
                .map(str::trim)
                .filter(|reason| !reason.is_empty())
                .unwrap_or(SAND_AUTO_REVIEW_BLOCK_REASON)
                .to_string();
            let proposed_rule = success
                .proposed_allow_rule
                .as_deref()
                .map(str::trim)
                .filter(|rule| !rule.is_empty())
                .map(ToOwned::to_owned);
            Ok(AutoReviewClassifierDecision::Block {
                reason,
                proposed_rule,
            })
        }
        SmartModeClassifierDecision::Unknown => Ok(AutoReviewClassifierDecision::Reject {
            reason: error_reason.to_string(),
        }),
    }
}
