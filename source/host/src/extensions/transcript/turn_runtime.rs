use crate::runner::conversation_state::would_recover_via_prepend;

use super::send_pipeline::{PersistedSendContext, RecoverySend};

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
