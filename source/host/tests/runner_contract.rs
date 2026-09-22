use std::time::Duration;

use mahayana_host_runtime::{
    AttemptCheckpoint, AttemptProgress, RetryDecision, StreamAttemptPolicy,
    TerminalOutcome, TransientStreamError,
};
use mahayana_host_runtime::runner::{StreamFailureKind, StreamWatchdog, TurnSettlement};

#[test]
fn first_output_watchdog_expires_only_before_output() {
    let policy = StreamAttemptPolicy {
        first_output_timeout: Duration::from_millis(100),
        ..StreamAttemptPolicy::default()
    };
    let mut watchdog = StreamWatchdog::new(&policy);
    assert!(!watchdog.expired_at(Duration::from_millis(99)));
    assert!(watchdog.expired_at(Duration::from_millis(100)));
    watchdog.mark_output();
    assert!(!watchdog.expired_at(Duration::from_secs(10)));
}

#[test]
fn transient_failure_retries_before_any_visible_output() {
    let policy = StreamAttemptPolicy::default();
    let error = TransientStreamError::classify("connection reset", None, None);
    assert_eq!(error.kind, StreamFailureKind::Transport);
    assert!(matches!(
        policy.retry_decision(1, &AttemptProgress::default(), &error),
        RetryDecision::RetryAfter(_)
    ));
}

#[test]
fn partial_output_is_not_retried_without_checkpoint() {
    let policy = StreamAttemptPolicy::default();
    let error = TransientStreamError::classify("upstream timeout", None, None);
    let mut progress = AttemptProgress::default();
    progress.record_output(12);
    assert_eq!(policy.retry_decision(1, &progress, &error), RetryDecision::Fail);
}

#[test]
fn partial_output_can_resume_from_checkpoint() {
    let policy = StreamAttemptPolicy::default();
    let error = TransientStreamError::classify("upstream timeout", None, None);
    let mut progress = AttemptProgress::default();
    progress.record_output(12);
    progress.checkpoint = Some(AttemptCheckpoint::new("cursor-7", 12, 0));
    assert!(matches!(
        policy.retry_decision(1, &progress, &error),
        RetryDecision::ResumeAfter { .. }
    ));
}

#[test]
fn retry_after_is_bounded() {
    let policy = StreamAttemptPolicy::default();
    let error = TransientStreamError::classify("rate limit", Some(429), Some(120_000));
    match policy.retry_decision(1, &AttemptProgress::default(), &error) {
        RetryDecision::RetryAfter(delay) => assert_eq!(delay, Duration::from_secs(30)),
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn max_attempt_count_is_terminal() {
    let policy = StreamAttemptPolicy::default();
    let error = TransientStreamError::classify("503", Some(503), None);
    assert_eq!(
        policy.retry_decision(policy.max_attempts, &AttemptProgress::default(), &error),
        RetryDecision::Fail
    );
}

#[test]
fn authentication_and_protocol_errors_do_not_retry() {
    let policy = StreamAttemptPolicy::default();
    for error in [
        TransientStreamError::classify("unauthorized", Some(401), None),
        TransientStreamError::classify("malformed stream", None, None),
    ] {
        assert_eq!(
            policy.retry_decision(1, &AttemptProgress::default(), &error),
            RetryDecision::Fail
        );
    }
}

#[test]
fn terminal_settlement_is_exactly_once() {
    let mut settlement = TurnSettlement::default();
    settlement.settle(TerminalOutcome::Completed).unwrap();
    assert!(settlement.settle(TerminalOutcome::Cancelled).is_err());
    assert_eq!(settlement.outcome(), Some(&TerminalOutcome::Completed));
}
