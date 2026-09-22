//! Mahayana Host/Runner ownership boundary matching Grok Bot 0.18.
//!
//! Coordinator supervision lives in `source/node-agent-coordinator`; this
//! crate owns accepted turn execution, stream attempts, retry/checkpoint policy,
//! cancellation and terminal settlement.

pub mod extensions;
pub mod host_paths;
pub mod host_discovery;
pub mod host_lock;
pub mod gateway_config;
pub mod runner;

pub use runner::{
    AttemptCheckpoint, AttemptProgress, CheckpointBoundary, HIDDEN_PROMPT_MARKER,
    MAX_SAFE_TOKEN_COUNT, ModelResolutionTicket, RecentUserMessage, ResolvedModelTracker,
    RetryDecision, SUMMARIZATION_MAX_PROMPT_CHARS, SanitizedUsage, StreamAttemptPolicy,
    StreamFailureKind, StreamWatchdog, TOOL_CALL_IDENTITY_CAP, TerminalOutcome, ToolCallIdentity,
    ToolSurfaceUpdate, TransientStreamError, TurnCancellation, TurnEndedUsage, TurnOwnerToken,
    TurnRunFinished, TurnRunOptions, TurnRunShell, TurnRunShellError, TurnRunStarted,
    TurnSettlement, TurnUsage, add_token_counts, build_unanswered_questions_note,
    merge_turn_usage, sanitize_usage, select_unconfirmed_user_messages, should_use_self_summary,
    to_safe_token_count, to_safe_usage_count, total_input_tokens, turn_usage_from_turn_ended,
};
