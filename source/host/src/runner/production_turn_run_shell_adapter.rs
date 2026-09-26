use std::cell::RefCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedProviderCheckpoint,
};

use super::routed_provider_runtime::RoutedProviderCancellation;
use super::{
    AttemptCheckpoint, AttemptProgress, RetryDecision, StreamAttemptPolicy,
    StreamFailureKind, TransientStreamError,
};

pub const DEFAULT_WATCHDOG_POLL_INTERVAL: Duration =
    Duration::from_millis(10);

pub trait RoutedProviderCheckpointStore: Send + Sync {
    /// Persist the checkpoint before it becomes eligible for resume.
    ///
    /// The returned cursor must identify the durable checkpoint payload.
    fn persist(
        &self,
        checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError>;
}

pub trait RoutedProviderAttemptExecutor {
    fn run_attempt(
        &mut self,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(
            &RoutedProviderCheckpoint,
        ) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRetryEvent {
    pub attempt: u32,
    pub next_attempt: u32,
    pub delay_ms: u64,
    pub resume_from_checkpoint: bool,
    pub watchdog_expired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRetryOutcome {
    Retried,
    Exhausted,
    GaveUpIneligible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRetryReport {
    pub outcome: ProviderRetryOutcome,
    pub attempt: u32,
    pub max_attempts: u32,
    pub delay_ms: Option<u64>,
    pub server_paced: bool,
    pub resume_from_checkpoint: bool,
    pub watchdog_expired: bool,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct ProductionTurnRunShellAdapter {
    pub policy: StreamAttemptPolicy,
    pub watchdog_poll_interval: Duration,
}

impl Default for ProductionTurnRunShellAdapter {
    fn default() -> Self {
        Self {
            policy: StreamAttemptPolicy::default(),
            watchdog_poll_interval: DEFAULT_WATCHDOG_POLL_INTERVAL,
        }
    }
}

impl ProductionTurnRunShellAdapter {
    pub fn run(
        &self,
        cancellation: &RoutedProviderCancellation,
        checkpoint_store: &dyn RoutedProviderCheckpointStore,
        executor: &mut dyn RoutedProviderAttemptExecutor,
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.run_with_retry(
            cancellation,
            checkpoint_store,
            executor,
            on_text_delta,
            &mut |_| {},
        )
    }

    pub fn run_with_retry(
        &self,
        cancellation: &RoutedProviderCancellation,
        checkpoint_store: &dyn RoutedProviderCheckpointStore,
        executor: &mut dyn RoutedProviderAttemptExecutor,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_retry: &mut dyn FnMut(&ProviderRetryEvent),
    ) -> Result<String, ProviderSessionError> {
        self.run_with_retry_reporting(
            cancellation,
            checkpoint_store,
            executor,
            on_text_delta,
            on_retry,
            &mut |_| {},
        )
    }

    pub fn run_with_retry_reporting(
        &self,
        cancellation: &RoutedProviderCancellation,
        checkpoint_store: &dyn RoutedProviderCheckpointStore,
        executor: &mut dyn RoutedProviderAttemptExecutor,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_retry: &mut dyn FnMut(&ProviderRetryEvent),
        on_report: &mut dyn FnMut(&ProviderRetryReport),
    ) -> Result<String, ProviderSessionError> {
        let mut attempt = 1_u32;
        let mut resume_from: Option<RoutedProviderCheckpoint> = None;

        loop {
            if cancellation.is_cancelled() {
                return Err(cancelled_error(cancellation));
            }

            let first_output_seen = Arc::new(AtomicBool::new(false));
            let attempt_done = Arc::new(AtomicBool::new(false));
            let watchdog_expired = Arc::new(AtomicBool::new(false));
            let watchdog_timeout = first_output_timeout_for_attempt(
                self.policy.first_output_timeout,
                attempt,
            );
            let watchdog = spawn_first_output_watchdog(
                watchdog_timeout,
                self.watchdog_poll_interval,
                Arc::clone(&first_output_seen),
                Arc::clone(&attempt_done),
                Arc::clone(&watchdog_expired),
                cancellation.clone(),
            );

            let progress = RefCell::new(AttemptProgress::default());
            let accepted_resume =
                RefCell::new(resume_from.clone());
            let seen_for_delta = Arc::clone(&first_output_seen);
            let mut guarded_delta = |delta: &str, accumulated: &str| {
                seen_for_delta.store(true, Ordering::Release);
                let mut progress = progress.borrow_mut();
                progress.record_output(delta.len());
                // Output after a prior checkpoint invalidates that checkpoint
                // for this attempt until a newer boundary is durably accepted.
                progress.checkpoint = None;
                drop(progress);
                on_text_delta(delta, accumulated);
            };
            let seen_for_checkpoint = Arc::clone(&first_output_seen);
            let mut accept_checkpoint =
                |checkpoint: &RoutedProviderCheckpoint| {
                    let cursor = checkpoint_store.persist(checkpoint)?;
                    // A durable tool-boundary checkpoint is observable provider
                    // progress even when no text delta preceded it. Stop the
                    // first-output watchdog only after persistence succeeds so
                    // an unpersisted boundary can never authorize resume.
                    seen_for_checkpoint.store(true, Ordering::Release);
                    let mut progress = progress.borrow_mut();
                    progress.record_output(0);
                    progress.checkpoint = Some(AttemptCheckpoint::new(
                        cursor,
                        checkpoint.emitted_text_bytes(),
                        checkpoint.tool_calls_completed(),
                    ));
                    drop(progress);
                    *accepted_resume.borrow_mut() =
                        Some(checkpoint.clone());
                    Ok(())
                };

            let timed_out_for_attempt = Arc::clone(&watchdog_expired);
            let should_cancel = || {
                cancellation.is_cancelled()
                    || timed_out_for_attempt.load(Ordering::Acquire)
            };
            let result = executor.run_attempt(
                resume_from.as_ref(),
                &mut guarded_delta,
                &mut accept_checkpoint,
                &should_cancel,
            );

            attempt_done.store(true, Ordering::Release);
            if let Some(watchdog) = watchdog {
                let _ = watchdog.join();
            }

            if cancellation.is_cancelled() {
                return Err(cancelled_error(cancellation));
            }

            let timed_out =
                watchdog_expired.load(Ordering::Acquire);
            let progress = progress.into_inner();
            resume_from = accepted_resume.into_inner();

            match result {
                Ok(value) if !timed_out => return Ok(value),
                Ok(_) => {
                    let error = ProviderSessionError::Transport(
                        "Runner first-output watchdog timed out before provider output."
                            .into(),
                    );
                    if !self.retry(
                        &mut attempt,
                        &progress,
                        &error,
                        true,
                        cancellation,
                        resume_from.as_ref(),
                        on_retry,
                        on_report,
                    )? {
                        return Err(error);
                    }
                }
                Err(error) => {
                    let error = if timed_out {
                        ProviderSessionError::Transport(
                            "Runner first-output watchdog timed out before provider output."
                                .into(),
                        )
                    } else {
                        error
                    };
                    if !self.retry(
                        &mut attempt,
                        &progress,
                        &error,
                        timed_out,
                        cancellation,
                        resume_from.as_ref(),
                        on_retry,
                        on_report,
                    )? {
                        return Err(error);
                    }
                }
            }
        }
    }

    fn retry(
        &self,
        attempt: &mut u32,
        progress: &AttemptProgress,
        error: &ProviderSessionError,
        watchdog_expired: bool,
        cancellation: &RoutedProviderCancellation,
        accepted_resume: Option<&RoutedProviderCheckpoint>,
        on_retry: &mut dyn FnMut(&ProviderRetryEvent),
        on_report: &mut dyn FnMut(&ProviderRetryReport),
    ) -> Result<bool, ProviderSessionError> {
        let transient =
            classify_provider_failure(error, watchdog_expired);
        let decision = self
            .policy
            .retry_decision(*attempt, progress, &transient);
        let (delay, resume_from_checkpoint) = match decision {
            RetryDecision::RetryAfter(delay) => (delay, false),
            RetryDecision::ResumeAfter {
                delay,
                checkpoint: _,
            } => {
                if accepted_resume.is_none() {
                    return Ok(false);
                }
                (delay, true)
            }
            RetryDecision::Fail => {
                let outcome = if *attempt > 1 && *attempt >= self.policy.max_attempts {
                    Some(ProviderRetryOutcome::Exhausted)
                } else if *attempt > 1 || transient.retryable() {
                    Some(ProviderRetryOutcome::GaveUpIneligible)
                } else {
                    None
                };
                if let Some(outcome) = outcome {
                    on_report(&ProviderRetryReport {
                        outcome,
                        attempt: *attempt,
                        max_attempts: self.policy.max_attempts,
                        delay_ms: None,
                        server_paced: false,
                        resume_from_checkpoint: accepted_resume.is_some(),
                        watchdog_expired,
                        error: error.to_string(),
                    });
                }
                return Ok(false);
            }
        };
        let next_attempt = attempt.saturating_add(1);
        let delay_ms = delay.as_millis().min(u64::MAX as u128) as u64;
        let server_paced = transient.retry_after_ms.is_some();
        on_report(&ProviderRetryReport {
            outcome: ProviderRetryOutcome::Retried,
            attempt: *attempt,
            max_attempts: self.policy.max_attempts,
            delay_ms: Some(delay_ms),
            server_paced,
            resume_from_checkpoint,
            watchdog_expired,
            error: error.to_string(),
        });
        on_retry(&ProviderRetryEvent {
            attempt: *attempt,
            next_attempt,
            delay_ms,
            resume_from_checkpoint,
            watchdog_expired,
        });
        sleep_with_cancellation(delay, cancellation)?;
        *attempt = next_attempt;
        Ok(true)
    }
}

pub fn first_output_timeout_for_attempt(
    base: Duration,
    attempt: u32,
) -> Duration {
    base.saturating_mul(1_u32 << attempt.saturating_sub(1).min(8))
}

fn cancelled_error(
    cancellation: &RoutedProviderCancellation,
) -> ProviderSessionError {
    ProviderSessionError::Cancelled(
        cancellation
            .reason()
            .unwrap_or_else(|| "Runner provider request cancelled".into()),
    )
}

fn spawn_first_output_watchdog(
    timeout: Duration,
    poll_interval: Duration,
    first_output_seen: Arc<AtomicBool>,
    attempt_done: Arc<AtomicBool>,
    expired: Arc<AtomicBool>,
    cancellation: RoutedProviderCancellation,
) -> Option<thread::JoinHandle<()>> {
    if timeout.is_zero() {
        expired.store(true, Ordering::Release);
        return None;
    }
    Some(thread::spawn(move || {
        let started = Instant::now();
        while !attempt_done.load(Ordering::Acquire)
            && !first_output_seen.load(Ordering::Acquire)
            && !cancellation.is_cancelled()
        {
            let elapsed = started.elapsed();
            if elapsed >= timeout {
                expired.store(true, Ordering::Release);
                break;
            }
            let remaining = timeout.saturating_sub(elapsed);
            let sleep_for = if poll_interval.is_zero() {
                remaining.min(Duration::from_millis(1))
            } else {
                remaining.min(poll_interval)
            };
            thread::sleep(sleep_for);
        }
    }))
}

fn sleep_with_cancellation(
    delay: Duration,
    cancellation: &RoutedProviderCancellation,
) -> Result<(), ProviderSessionError> {
    let started = Instant::now();
    while started.elapsed() < delay {
        if cancellation.is_cancelled() {
            return Err(cancelled_error(cancellation));
        }
        let remaining = delay.saturating_sub(started.elapsed());
        thread::sleep(remaining.min(Duration::from_millis(10)));
    }
    Ok(())
}

fn classify_provider_failure(
    error: &ProviderSessionError,
    watchdog_expired: bool,
) -> TransientStreamError {
    if watchdog_expired {
        return TransientStreamError {
            kind: StreamFailureKind::Timeout,
            message: error.to_string(),
            retry_after_ms: None,
        };
    }

    let message = error.to_string();
    match error {
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
        ProviderSessionError::Protocol(_) | ProviderSessionError::Tool(_) => {
            TransientStreamError {
                kind: StreamFailureKind::Protocol,
                message,
                retry_after_ms: None,
            }
        }
        ProviderSessionError::Transport(_) => {
            let status = [408_u16, 429, 500, 502, 503, 504]
                .into_iter()
                .find(|status| {
                    message.contains(&format!("({status}"))
                        || message.contains(&format!(" {status} "))
                });
            TransientStreamError::classify(message, status, None)
        }
    }
}
