use std::time::{Duration, Instant};

use super::{AttemptCheckpoint, TransientStreamError};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttemptProgress {
    pub output_events: u64,
    pub output_bytes: usize,
    pub checkpoint: Option<AttemptCheckpoint>,
}

impl AttemptProgress {
    pub fn record_output(&mut self, bytes: usize) {
        self.output_events = self.output_events.saturating_add(1);
        self.output_bytes = self.output_bytes.saturating_add(bytes);
    }

    pub fn saw_output(&self) -> bool {
        self.output_events > 0 || self.output_bytes > 0
    }

    pub fn resumable(&self) -> bool {
        self.checkpoint.as_ref().is_some_and(AttemptCheckpoint::is_resumable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamAttemptPolicy {
    pub first_output_timeout: Duration,
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub max_retry_after: Duration,
}

impl Default for StreamAttemptPolicy {
    fn default() -> Self {
        Self {
            first_output_timeout: Duration::from_secs(150),
            max_attempts: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(4),
            max_retry_after: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryDecision {
    RetryAfter(Duration),
    ResumeAfter { delay: Duration, checkpoint: AttemptCheckpoint },
    Fail,
}

impl StreamAttemptPolicy {
    pub fn retry_decision(
        &self,
        attempt: u32,
        progress: &AttemptProgress,
        error: &TransientStreamError,
    ) -> RetryDecision {
        if attempt >= self.max_attempts || !error.retryable() {
            return RetryDecision::Fail;
        }

        let delay = error
            .retry_after_ms
            .map(Duration::from_millis)
            .map(|value| value.min(self.max_retry_after))
            .unwrap_or_else(|| {
                let exponent = attempt.saturating_sub(1).min(8);
                self.initial_backoff
                    .saturating_mul(1_u32 << exponent)
                    .min(self.max_backoff)
            });

        if !progress.saw_output() {
            return RetryDecision::RetryAfter(delay);
        }
        if let Some(checkpoint) = progress.checkpoint.clone().filter(AttemptCheckpoint::is_resumable) {
            return RetryDecision::ResumeAfter { delay, checkpoint };
        }
        RetryDecision::Fail
    }
}

#[derive(Debug)]
pub struct StreamWatchdog {
    started: Instant,
    first_output_timeout: Duration,
    first_output_seen: bool,
}

impl StreamWatchdog {
    pub fn new(policy: &StreamAttemptPolicy) -> Self {
        Self {
            started: Instant::now(),
            first_output_timeout: policy.first_output_timeout,
            first_output_seen: false,
        }
    }

    pub fn mark_output(&mut self) {
        self.first_output_seen = true;
    }

    pub fn expired(&self) -> bool {
        !self.first_output_seen && self.started.elapsed() >= self.first_output_timeout
    }

    pub fn expired_at(&self, elapsed: Duration) -> bool {
        !self.first_output_seen && elapsed >= self.first_output_timeout
    }
}


use std::future::Future;
use std::pin::Pin;

pub type OuterStreamFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCancelReason {
    pub intentional: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OuterCheckpointDisposition {
    Persisted,
    IgnoredStaleGeneration,
}

/// Frozen outer persistence contract around a generated Agent stream attempt.
///
/// This is deliberately independent from the provider-retry checkpoint store:
/// State is the real Agent ConversationStateStructure projection, while the
/// routed-provider store remains only a retry/resume transport checkpoint.
pub trait OuterStreamPersistence<Context, State>: Send + Sync {
    fn generation(&self) -> u64;
    fn run_generation(&self) -> u64;

    fn prepare_checkpoint_for_persistence(&self, checkpoint: &mut State);

    fn persist_step_checkpoint<'a>(
        &'a self,
        context: &'a Context,
        checkpoint: &'a State,
    ) -> OuterStreamFuture<'a, Result<(), String>>;

    fn note_checkpoint(&self, _checkpoint: &State) {}

    fn persist_final_state<'a>(
        &'a self,
        context: &'a Context,
        checkpoint: &'a State,
    ) -> OuterStreamFuture<'a, Result<(), String>>;

    fn commit_disk_pressure_reminder(&self);
    fn release_disk_pressure_reminder(&self);

    fn note_automation_status_reminder(&self) {}

    fn is_awaiting_user_selection(&self) -> bool;
    fn is_quiescing_for_upgrade(&self) -> bool;
    fn mark_quiesced_for_upgrade(&self);
    fn cancel_run(&self, cancellation: StreamCancelReason);
}

pub async fn persist_outer_stream_checkpoint<P, Context, State>(
    persistence: &P,
    context: &Context,
    checkpoint: &mut State,
) -> Result<OuterCheckpointDisposition, String>
where
    P: OuterStreamPersistence<Context, State>,
{
    if persistence.run_generation() != persistence.generation() {
        return Ok(OuterCheckpointDisposition::IgnoredStaleGeneration);
    }

    persistence.prepare_checkpoint_for_persistence(checkpoint);
    persistence
        .persist_step_checkpoint(context, checkpoint)
        .await?;
    persistence.note_checkpoint(checkpoint);
    persistence.commit_disk_pressure_reminder();
    persistence.note_automation_status_reminder();

    if persistence.is_awaiting_user_selection() {
        persistence.cancel_run(StreamCancelReason {
            intentional: true,
            reason: "awaiting user selection".into(),
        });
    } else if persistence.is_quiescing_for_upgrade() {
        persistence.mark_quiesced_for_upgrade();
        persistence.cancel_run(StreamCancelReason {
            intentional: true,
            reason: "quiescing for forced host upgrade".into(),
        });
    }

    Ok(OuterCheckpointDisposition::Persisted)
}

pub async fn persist_outer_stream_final_state<P, Context, State>(
    persistence: &P,
    context: &Context,
    final_state: &State,
) -> Result<OuterCheckpointDisposition, String>
where
    P: OuterStreamPersistence<Context, State>,
{
    if persistence.run_generation() != persistence.generation() {
        return Ok(OuterCheckpointDisposition::IgnoredStaleGeneration);
    }
    persistence.persist_final_state(context, final_state).await?;
    persistence.commit_disk_pressure_reminder();
    Ok(OuterCheckpointDisposition::Persisted)
}

/// Mirrors the frozen outer finally-owner: callers must invoke this exactly
/// once after attempt cleanup/final persistence, including failure paths.
pub fn release_outer_stream_persistence<P, Context, State>(persistence: &P)
where
    P: OuterStreamPersistence<Context, State>,
{
    persistence.release_disk_pressure_reminder();
}
