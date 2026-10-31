use std::collections::VecDeque;

use serde_json::Value;
use uuid::Uuid;

use super::conversation_state::RecentUserMessage;
use super::{TerminalOutcome, TurnSettlement};

pub const TOOL_LOOP_HISTORY_CAPACITY: usize = 64;
pub const TOOL_LOOP_REPEAT_THRESHOLD: u8 = 3;
const TOOL_LOOP_MAX_IDENTITY_BYTES: usize = 64 * 1024;
const TOOL_LOOP_CANCEL_REASON: &str = "repeated identical tool call detected";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnOwnerToken {
    pub request_id: String,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCancellation {
    pub intentional: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TurnRunOptions {
    pub inference_request_id: Option<String>,
    pub message_id: Option<String>,
    pub recent_message_text: Option<String>,
    pub recent_message_rich_text: Option<String>,
    pub recent_user_messages: Vec<RecentUserMessage>,
    pub turn_started_at_ms: Option<u64>,
    pub is_fork: bool,
    pub attachment_count: usize,
    pub image_count: usize,
    pub video_count: usize,
    pub has_reply_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRunStarted {
    pub owner: TurnOwnerToken,
    pub recovery_shaped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRunFinished {
    pub owner: TurnOwnerToken,
    pub outcome: TerminalOutcome,
    pub quiesced_for_upgrade: bool,
    pub ended_on_silent_tool_calls: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointBoundary {
    Continue,
    Cancel(TurnCancellation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolLoopDecision {
    Continue { consecutive_repeats: u8 },
    Cancel { reason: &'static str },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TurnRunShellError {
    #[error("prompt cannot be empty")]
    EmptyPrompt,
    #[error("another turn already owns the runner")]
    AlreadyRunning,
    #[error("stale turn owner")]
    StaleOwner,
    #[error("tool name cannot be empty")]
    EmptyToolName,
    #[error("turn settlement failed: {0}")]
    Settlement(&'static str),
}

#[derive(Debug)]
struct ActiveRun {
    owner: TurnOwnerToken,
    dispatched: bool,
    recovery_shaped: bool,
    awaiting_user_selection: bool,
    quiesced_for_upgrade: bool,
    cancellation: Option<TurnCancellation>,
    settlement: TurnSettlement,
    last_tool_call_identity: Option<String>,
    consecutive_tool_calls: u8,
    tool_call_history: VecDeque<String>,
}

pub fn is_recovery_shaped_turn(prompt: &str, options: &TurnRunOptions) -> bool {
    let trimmed = prompt.trim();
    let current_message_text = options
        .message_id
        .as_deref()
        .and_then(|message_id| {
            options
                .recent_user_messages
                .iter()
                .find(|message| message.id == message_id)
                .map(|message| message.text.as_str())
        })
        .or(options.recent_message_text.as_deref());
    options.message_id.is_some()
        && !options.is_fork
        && options.attachment_count == 0
        && options.image_count == 0
        && options.video_count == 0
        && !options.has_reply_context
        && current_message_text.is_some_and(|text| text.trim() == trimmed)
}

#[derive(Debug, Default)]
pub struct TurnRunShell {
    active: Option<ActiveRun>,
    quiescing_for_upgrade: bool,
    next_generation: u64,
}

fn tool_call_identity(tool_name: &str, arguments: &Value) -> String {
    let normalized = canonical_json(arguments);
    let identity = format!("{}:{normalized}", tool_name.trim());
    if identity.len() <= TOOL_LOOP_MAX_IDENTITY_BYTES {
        identity
    } else {
        format!("{}:{}", tool_name.trim(), crate::sha256::sha256_hex(identity))
    }
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            let fields = entries
                .into_iter()
                .map(|(key, value)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("JSON object keys are serializable"),
                        canonical_json(value)
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => serde_json::to_string(value).expect("JSON values are serializable"),
    }
}

impl TurnRunShell {
    pub fn begin_run(
        &mut self,
        prompt: &str,
        options: TurnRunOptions,
    ) -> Result<TurnRunStarted, TurnRunShellError> {
        if self.active.is_some() {
            return Err(TurnRunShellError::AlreadyRunning);
        }
        let trimmed = prompt.trim();
        if trimmed.is_empty()
            && options.attachment_count == 0
            && options.image_count == 0
            && options.video_count == 0
        {
            return Err(TurnRunShellError::EmptyPrompt);
        }
        self.next_generation = self.next_generation.saturating_add(1);
        let recovery_shaped = is_recovery_shaped_turn(trimmed, &options);
        let request_id = options
            .inference_request_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let owner = TurnOwnerToken {
            request_id,
            generation: self.next_generation,
        };
        self.active = Some(ActiveRun {
            owner: owner.clone(),
            dispatched: false,
            recovery_shaped,
            awaiting_user_selection: false,
            quiesced_for_upgrade: false,
            cancellation: None,
            settlement: TurnSettlement::default(),
            last_tool_call_identity: None,
            consecutive_tool_calls: 0,
            tool_call_history: VecDeque::with_capacity(TOOL_LOOP_HISTORY_CAPACITY),
        });
        Ok(TurnRunStarted {
            owner,
            recovery_shaped,
        })
    }

    pub fn active_request_id(&self) -> Option<&str> {
        self.active
            .as_ref()
            .map(|run| run.owner.request_id.as_str())
    }

    pub fn active_owner(&self) -> Option<&TurnOwnerToken> {
        self.active.as_ref().map(|run| &run.owner)
    }

    pub fn has_active_run(&self) -> bool {
        self.active.is_some()
    }

    pub fn mark_dispatched(&mut self, owner: &TurnOwnerToken) -> Result<(), TurnRunShellError> {
        self.require_owner_mut(owner)?.dispatched = true;
        Ok(())
    }

    /// Observe one tool call at the canonical turn owner boundary.
    ///
    /// A real tool producer must call this before the detector can be
    /// considered wired.
    pub fn observe_tool_call(
        &mut self,
        owner: &TurnOwnerToken,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<ToolLoopDecision, TurnRunShellError> {
        if tool_name.trim().is_empty() {
            return Err(TurnRunShellError::EmptyToolName);
        }
        let identity = tool_call_identity(tool_name, arguments);
        let run = self.require_owner_mut(owner)?;
        if run.last_tool_call_identity.as_deref() == Some(identity.as_str()) {
            run.consecutive_tool_calls = run
                .consecutive_tool_calls
                .saturating_add(1)
                .min(TOOL_LOOP_REPEAT_THRESHOLD);
        } else {
            run.last_tool_call_identity = Some(identity.clone());
            run.consecutive_tool_calls = 1;
        }
        if run.tool_call_history.len() == TOOL_LOOP_HISTORY_CAPACITY {
            run.tool_call_history.pop_front();
        }
        run.tool_call_history.push_back(identity);
        if run.consecutive_tool_calls >= TOOL_LOOP_REPEAT_THRESHOLD {
            let cancellation = TurnCancellation {
                intentional: true,
                reason: TOOL_LOOP_CANCEL_REASON.to_string(),
            };
            if run.cancellation.is_none() {
                run.cancellation = Some(cancellation);
            }
            return Ok(ToolLoopDecision::Cancel {
                reason: TOOL_LOOP_CANCEL_REASON,
            });
        }
        Ok(ToolLoopDecision::Continue {
            consecutive_repeats: run.consecutive_tool_calls,
        })
    }

    pub fn tool_call_history_len(
        &self,
        owner: &TurnOwnerToken,
    ) -> Result<usize, TurnRunShellError> {
        Ok(self.require_owner(owner)?.tool_call_history.len())
    }

    pub fn interrupt(
        &mut self,
        reason: impl Into<String>,
        supersede_carries_recovery: Option<bool>,
    ) -> bool {
        let Some(run) = self.active.as_mut() else {
            return false;
        };
        if !run.dispatched {
            if let Some(carries_recovery) = supersede_carries_recovery {
                if !carries_recovery || !run.recovery_shaped {
                    return false;
                }
            }
        }
        if run.cancellation.is_none() {
            run.cancellation = Some(TurnCancellation {
                intentional: true,
                reason: reason.into(),
            });
        }
        true
    }

    pub fn request_quiesce_for_upgrade(&mut self) {
        self.quiescing_for_upgrade = true;
    }

    pub fn cancel_quiesce_for_upgrade(&mut self) {
        self.quiescing_for_upgrade = false;
    }

    pub fn is_quiescing_for_upgrade(&self) -> bool {
        self.quiescing_for_upgrade
    }

    pub fn mark_quiesced_for_upgrade(
        &mut self,
        owner: &TurnOwnerToken,
    ) -> Result<(), TurnRunShellError> {
        self.require_owner_mut(owner)?.quiesced_for_upgrade = true;
        Ok(())
    }

    pub fn end_turn_awaiting_user(
        &mut self,
        owner: &TurnOwnerToken,
        reason: impl Into<String>,
    ) -> Result<(), TurnRunShellError> {
        let run = self.require_owner_mut(owner)?;
        run.awaiting_user_selection = true;
        run.cancellation = Some(TurnCancellation {
            intentional: true,
            reason: reason.into(),
        });
        Ok(())
    }

    pub fn checkpoint_boundary(
        &mut self,
        owner: &TurnOwnerToken,
    ) -> Result<CheckpointBoundary, TurnRunShellError> {
        let quiescing = self.quiescing_for_upgrade;
        let run = self.require_owner_mut(owner)?;
        if run.awaiting_user_selection {
            let cancellation = run
                .cancellation
                .clone()
                .unwrap_or_else(|| TurnCancellation {
                    intentional: true,
                    reason: "awaiting user selection".into(),
                });
            return Ok(CheckpointBoundary::Cancel(cancellation));
        }
        if quiescing {
            run.quiesced_for_upgrade = true;
            let cancellation = TurnCancellation {
                intentional: true,
                reason: "quiescing for forced host upgrade".into(),
            };
            run.cancellation = Some(cancellation.clone());
            return Ok(CheckpointBoundary::Cancel(cancellation));
        }
        if let Some(cancellation) = run.cancellation.clone() {
            return Ok(CheckpointBoundary::Cancel(cancellation));
        }
        Ok(CheckpointBoundary::Continue)
    }

    pub fn cancellation(
        &self,
        owner: &TurnOwnerToken,
    ) -> Result<Option<&TurnCancellation>, TurnRunShellError> {
        Ok(self.require_owner(owner)?.cancellation.as_ref())
    }

    pub fn owns_final_state(&self, owner: &TurnOwnerToken) -> bool {
        self.active.as_ref().is_some_and(|run| &run.owner == owner)
    }

    pub fn finish_completed(
        &mut self,
        owner: &TurnOwnerToken,
    ) -> Result<TurnRunFinished, TurnRunShellError> {
        self.finish(owner, TerminalOutcome::Completed)
    }

    pub fn finish_failed(
        &mut self,
        owner: &TurnOwnerToken,
        retryable: bool,
        message: impl Into<String>,
    ) -> Result<TurnRunFinished, TurnRunShellError> {
        self.finish(
            owner,
            TerminalOutcome::Failed {
                retryable,
                message: message.into(),
            },
        )
    }

    pub fn finish_cancelled(
        &mut self,
        owner: &TurnOwnerToken,
    ) -> Result<TurnRunFinished, TurnRunShellError> {
        let outcome = if self.require_owner(owner)?.awaiting_user_selection {
            TerminalOutcome::WaitingUser
        } else {
            TerminalOutcome::Cancelled
        };
        self.finish(owner, outcome)
    }

    fn finish(
        &mut self,
        owner: &TurnOwnerToken,
        outcome: TerminalOutcome,
    ) -> Result<TurnRunFinished, TurnRunShellError> {
        let run = self.require_owner_mut(owner)?;
        run.settlement
            .settle(outcome.clone())
            .map_err(TurnRunShellError::Settlement)?;
        let quiesced_for_upgrade = run.quiesced_for_upgrade;
        let finished_owner = run.owner.clone();
        self.active = None;
        Ok(TurnRunFinished {
            owner: finished_owner,
            outcome,
            quiesced_for_upgrade,
            ended_on_silent_tool_calls: false,
        })
    }

    fn require_owner(&self, owner: &TurnOwnerToken) -> Result<&ActiveRun, TurnRunShellError> {
        self.active
            .as_ref()
            .filter(|run| &run.owner == owner)
            .ok_or(TurnRunShellError::StaleOwner)
    }

    fn require_owner_mut(
        &mut self,
        owner: &TurnOwnerToken,
    ) -> Result<&mut ActiveRun, TurnRunShellError> {
        self.active
            .as_mut()
            .filter(|run| &run.owner == owner)
            .ok_or(TurnRunShellError::StaleOwner)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TOOL_LOOP_HISTORY_CAPACITY, ToolLoopDecision, TurnRunOptions, TurnRunShell,
        TurnRunShellError,
    };
    use serde_json::json;

    #[test]
    fn tool_loop_identity_sorts_object_keys() {
        let mut shell = TurnRunShell::default();
        let owner = shell
            .begin_run("loop", TurnRunOptions::default())
            .expect("start")
            .owner;
        let first = shell
            .observe_tool_call(&owner, "search", &json!({"b": 2, "a": 1}))
            .expect("first");
        let second = shell
            .observe_tool_call(&owner, "search", &json!({"a": 1, "b": 2}))
            .expect("second");
        assert_eq!(first, ToolLoopDecision::Continue { consecutive_repeats: 1 });
        assert_eq!(second, ToolLoopDecision::Continue { consecutive_repeats: 2 });
    }

    #[test]
    fn third_consecutive_identical_call_requests_cancellation() {
        let mut shell = TurnRunShell::default();
        let owner = shell
            .begin_run("loop", TurnRunOptions::default())
            .expect("start")
            .owner;
        for _ in 0..2 {
            assert!(matches!(
                shell.observe_tool_call(&owner, "search", &json!({"q": "same"})),
                Ok(ToolLoopDecision::Continue { .. })
            ));
        }
        assert_eq!(
            shell
                .observe_tool_call(&owner, "search", &json!({"q": "same"}))
                .expect("third"),
            ToolLoopDecision::Cancel {
                reason: "repeated identical tool call detected"
            }
        );
        assert!(shell
            .cancellation(&owner)
            .expect("cancellation read")
            .is_some());
    }

    #[test]
    fn distinct_call_resets_consecutive_count_and_history_is_bounded() {
        let mut shell = TurnRunShell::default();
        let owner = shell
            .begin_run("loop", TurnRunOptions::default())
            .expect("start")
            .owner;
        for index in 0..(TOOL_LOOP_HISTORY_CAPACITY + 4) {
            shell
                .observe_tool_call(&owner, "search", &json!({"q": index}))
                .expect("observe");
        }
        assert_eq!(
            shell.tool_call_history_len(&owner).expect("history"),
            TOOL_LOOP_HISTORY_CAPACITY
        );
        assert_eq!(
            shell
                .observe_tool_call(&owner, "search", &json!({"q": "new"}))
                .expect("reset"),
            ToolLoopDecision::Continue {
                consecutive_repeats: 1
            }
        );
    }

    #[test]
    fn empty_and_stale_tool_calls_do_not_change_the_owner() {
        let mut shell = TurnRunShell::default();
        let owner = shell
            .begin_run("loop", TurnRunOptions::default())
            .expect("start")
            .owner;
        assert_eq!(
            shell.observe_tool_call(&owner, " ", &json!({})),
            Err(TurnRunShellError::EmptyToolName)
        );
        let stale = super::super::TurnOwnerToken {
            request_id: "stale".into(),
            generation: owner.generation.saturating_sub(1),
        };
        assert_eq!(
            shell.observe_tool_call(&stale, "search", &json!({})),
            Err(TurnRunShellError::StaleOwner)
        );
        assert_eq!(shell.tool_call_history_len(&owner).expect("history"), 0);
    }
}
