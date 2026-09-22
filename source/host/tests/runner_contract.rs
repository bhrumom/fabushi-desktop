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


#[test]
fn turn_run_shell_enforces_owner_generation_and_pre_dispatch_supersede_rules() {
    use mahayana_host_runtime::{
        TerminalOutcome, TurnRunOptions, TurnRunShell, TurnRunShellError,
    };

    let mut shell = TurnRunShell::default();
    assert_eq!(
        shell.begin_run("   ", TurnRunOptions::default()),
        Err(TurnRunShellError::EmptyPrompt)
    );

    let first = shell
        .begin_run(
            "recover me",
            TurnRunOptions {
                inference_request_id: Some("request-1".into()),
                message_id: Some("t1u".into()),
                recent_message_text: Some("recover me".into()),
                ..TurnRunOptions::default()
            },
        )
        .expect("first run");
    assert!(first.recovery_shaped);
    assert_eq!(shell.active_request_id(), Some("request-1"));

    assert!(
        !shell.interrupt("superseded without recovery", Some(false)),
        "a pre-dispatch run must not be interrupted by an unsafe supersede"
    );
    assert!(
        shell.interrupt("superseded with recovery", Some(true)),
        "a recovery-shaped run can be safely superseded before dispatch"
    );
    let cancelled = shell.finish_cancelled(&first.owner).expect("cancelled");
    assert_eq!(cancelled.outcome, TerminalOutcome::Cancelled);

    let second = shell
        .begin_run(
            "new turn",
            TurnRunOptions {
                inference_request_id: Some("request-2".into()),
                ..TurnRunOptions::default()
            },
        )
        .expect("second run");
    assert!(second.owner.generation > first.owner.generation);
    assert_eq!(
        shell.mark_dispatched(&first.owner),
        Err(TurnRunShellError::StaleOwner)
    );
    assert_eq!(
        shell.finish_completed(&first.owner),
        Err(TurnRunShellError::StaleOwner)
    );
    shell.mark_dispatched(&second.owner).unwrap();
    let completed = shell.finish_completed(&second.owner).unwrap();
    assert_eq!(completed.outcome, TerminalOutcome::Completed);
    assert!(!shell.has_active_run());
}

#[test]
fn turn_run_shell_converts_checkpoint_boundary_to_waiting_user_or_quiesce() {
    use mahayana_host_runtime::{
        CheckpointBoundary, TerminalOutcome, TurnRunOptions, TurnRunShell,
    };

    let mut shell = TurnRunShell::default();
    let run = shell
        .begin_run(
            "choose",
            TurnRunOptions {
                inference_request_id: Some("request-user".into()),
                ..TurnRunOptions::default()
            },
        )
        .unwrap();
    shell.mark_dispatched(&run.owner).unwrap();
    shell
        .end_turn_awaiting_user(&run.owner, "permission selection")
        .unwrap();
    assert!(matches!(
        shell.checkpoint_boundary(&run.owner).unwrap(),
        CheckpointBoundary::Cancel(ref cancellation)
            if cancellation.intentional && cancellation.reason == "permission selection"
    ));
    let waiting = shell.finish_cancelled(&run.owner).unwrap();
    assert_eq!(waiting.outcome, TerminalOutcome::WaitingUser);

    let upgrade = shell
        .begin_run(
            "upgrade",
            TurnRunOptions {
                inference_request_id: Some("request-upgrade".into()),
                ..TurnRunOptions::default()
            },
        )
        .unwrap();
    shell.request_quiesce_for_upgrade();
    assert!(shell.is_quiescing_for_upgrade());
    assert!(matches!(
        shell.checkpoint_boundary(&upgrade.owner).unwrap(),
        CheckpointBoundary::Cancel(ref cancellation)
            if cancellation.reason.contains("forced host upgrade")
    ));
    let ended = shell.finish_cancelled(&upgrade.owner).unwrap();
    assert!(ended.quiesced_for_upgrade);
    shell.cancel_quiesce_for_upgrade();
    assert!(!shell.is_quiescing_for_upgrade());
}
