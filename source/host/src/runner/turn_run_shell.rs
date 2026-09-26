use uuid::Uuid;

use super::conversation_state::RecentUserMessage;
use super::{TerminalOutcome, TurnSettlement};

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
    pub recent_user_messages: Vec<RecentUserMessage>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointBoundary {
    Continue,
    Cancel(TurnCancellation),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TurnRunShellError {
    #[error("prompt cannot be empty")]
    EmptyPrompt,
    #[error("another turn already owns the runner")]
    AlreadyRunning,
    #[error("stale turn owner")]
    StaleOwner,
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
}

#[derive(Debug, Default)]
pub struct TurnRunShell {
    active: Option<ActiveRun>,
    quiescing_for_upgrade: bool,
    next_generation: u64,
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
        let request_id = options
            .inference_request_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let owner = TurnOwnerToken {
            request_id,
            generation: self.next_generation,
        };
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
        let recovery_shaped = options.message_id.is_some()
            && !options.is_fork
            && options.attachment_count == 0
            && options.image_count == 0
            && options.video_count == 0
            && !options.has_reply_context
            && current_message_text.is_some_and(|text| text.trim() == trimmed);
        self.active = Some(ActiveRun {
            owner: owner.clone(),
            dispatched: false,
            recovery_shaped,
            awaiting_user_selection: false,
            quiesced_for_upgrade: false,
            cancellation: None,
            settlement: TurnSettlement::default(),
        });
        Ok(TurnRunStarted {
            owner,
            recovery_shaped,
        })
    }

    pub fn active_request_id(&self) -> Option<&str> {
        self.active.as_ref().map(|run| run.owner.request_id.as_str())
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
            let cancellation = run.cancellation.clone().unwrap_or_else(|| TurnCancellation {
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
        self.active
            .as_ref()
            .is_some_and(|run| &run.owner == owner)
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
