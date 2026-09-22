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
