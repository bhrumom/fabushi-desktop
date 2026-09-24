use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::extensions::telemetry::sand_error_tags::SandErrorValue;
use crate::runner::transient_stream_error::{StreamFailureKind, TransientStreamError};
use crate::runner::would_recover_via_prepend;

use super::send_pipeline::{PersistedSendContext, RecoverySend};

pub fn classify_agent_error(error: &ProviderSessionError) -> SandErrorValue {
    let message = error.to_string();
    let lower = message.to_ascii_lowercase();

    if lower.contains("first-output watchdog")
        || lower.contains("first token stall")
        || lower.contains("first-token stall")
    {
        return SandErrorValue::new("SAND-E0402");
    }
    if lower.contains("context window overflow")
        || lower.contains("context-window overflow")
        || lower.contains("context length")
    {
        return SandErrorValue::new("SAND-E0404");
    }
    if lower.contains("conversation too large")
        || lower.contains("conversation-size hard cap")
        || lower.contains("conversation size hard cap")
    {
        return SandErrorValue::new("SAND-E0414");
    }

    let transient = match error {
        ProviderSessionError::Cancelled(_) => TransientStreamError {
            kind: StreamFailureKind::Cancelled,
            message,
            retry_after_ms: None,
        },
        ProviderSessionError::Authentication(_) => TransientStreamError {
            kind: StreamFailureKind::Authentication,
            message,
            retry_after_ms: None,
        },
        ProviderSessionError::Configuration(_) => TransientStreamError {
            kind: StreamFailureKind::InvalidRequest,
            message,
            retry_after_ms: None,
        },
        ProviderSessionError::Protocol(_) => TransientStreamError {
            kind: StreamFailureKind::Protocol,
            message,
            retry_after_ms: None,
        },
        ProviderSessionError::Tool(_) => TransientStreamError {
            kind: StreamFailureKind::Unknown,
            message,
            retry_after_ms: None,
        },
        ProviderSessionError::Transport(_) => {
            TransientStreamError::classify(message, None, None)
        }
    };

    match transient.kind {
        StreamFailureKind::Capacity | StreamFailureKind::RateLimit => {
            SandErrorValue::new("SAND-E0401")
        }
        StreamFailureKind::Timeout
        | StreamFailureKind::Transport
        | StreamFailureKind::Server => SandErrorValue::new("SAND-E0406"),
        StreamFailureKind::Authentication
        | StreamFailureKind::InvalidRequest
        | StreamFailureKind::Protocol => SandErrorValue::new("SAND-E0405"),
        StreamFailureKind::Cancelled | StreamFailureKind::Unknown => {
            SandErrorValue::new("SAND-E0407")
        }
    }
}

pub struct QueuedTurnRecoveryCheck<'a> {
    pub epoch: u64,
    pub current_epoch: u64,
    pub recovery_break_epoch: u64,
    pub prompt: &'a str,
    pub context: &'a PersistedSendContext,
    pub latest_recovery: Option<&'a RecoverySend>,
    pub is_fork: bool,
    pub has_reply_context: bool,
    pub attachment_count: usize,
    pub image_count: usize,
    pub video_count: usize,
}

pub fn should_supersede_stale_turn(args: QueuedTurnRecoveryCheck<'_>) -> bool {
    if args.epoch == args.current_epoch {
        return false;
    }
    let Some(message_id) = args
        .context
        .user_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    if args.is_fork
        || args.has_reply_context
        || args.attachment_count > 0
        || args.image_count > 0
        || args.video_count > 0
    {
        return false;
    }
    let Some(raw_text) = args
        .context
        .recent_user_messages
        .iter()
        .find(|message| message.id == message_id)
        .map(|message| message.text.as_str())
    else {
        return false;
    };
    if raw_text != args.prompt.trim() || args.epoch <= args.recovery_break_epoch {
        return false;
    }
    let Some(latest) = args.latest_recovery else {
        return false;
    };
    latest.epoch == args.current_epoch
        && would_recover_via_prepend(
            &latest.recent_user_messages,
            &latest.message_id,
            message_id,
        )
}
