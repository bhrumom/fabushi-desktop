//! Mahayana Host/Runner ownership boundary matching Grok Bot 0.18.
//!
//! Coordinator supervision lives in `source/node-agent-coordinator`; this
//! crate owns accepted turn execution, stream attempts, retry/checkpoint policy,
//! cancellation and terminal settlement.

pub mod extensions;
pub mod runner;

pub use runner::{
    AttemptCheckpoint, AttemptProgress, RetryDecision, StreamAttemptPolicy,
    TerminalOutcome, TransientStreamError,
};
