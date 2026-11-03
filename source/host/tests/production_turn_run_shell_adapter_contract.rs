use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::inference::provider_session::{
    OpenRouterCheckpoint, ProviderMessage, ProviderSessionError,
    RoutedProviderCheckpoint,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryEvent, ProviderRetryOutcome,
    ProviderRetryReport, RoutedProviderAttemptControl, RoutedProviderAttemptExecutor,
    RoutedProviderCheckpointStore, ToolEffectUnknownResult,
    first_output_timeout_for_attempt, run_production_turn_shell_lifecycle,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation,
};
use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
use serde_json::json;

use mahayana_host_runtime::runner::{
    TerminalOutcome, TurnRunOptions, TurnRunShell,
};

#[derive(Clone)]
enum Behavior {
    FailBeforeOutput,
    OutputCheckpointThenFail,
    OutputThenFail,
    ToolCheckpointThenStallUntilCancelled,
    Success {
        delta: &'static str,
        accumulated: &'static str,
        result: &'static str,
    },
    SilentUntilCancelled,
}

struct FakeExecutor {
    behaviors: VecDeque<Behavior>,
    resumes: Vec<Option<RoutedProviderCheckpoint>>,
    attempts: usize,
}

impl FakeExecutor {
    fn new(behaviors: impl Into<VecDeque<Behavior>>) -> Self {
        Self {
            behaviors: behaviors.into(),
            resumes: Vec::new(),
            attempts: 0,
        }
    }
}

fn checkpoint() -> RoutedProviderCheckpoint {
    RoutedProviderCheckpoint::OpenRouter(OpenRouterCheckpoint {
        conversation: vec![
            json!({"role":"user","content":"calendar"}),
            json!({"role":"tool","tool_call_id":"call-1","content":"daily"}),
        ],
        text: "Checking ".into(),
        completed_steps: 1,
        tool_calls_completed: 1,
    })
}

fn tool_only_checkpoint() -> RoutedProviderCheckpoint {
    RoutedProviderCheckpoint::OpenRouter(OpenRouterCheckpoint {
        conversation: vec![
            json!({"role":"user","content":"calendar"}),
            json!({"role":"tool","tool_call_id":"call-tool-only","content":"daily"}),
        ],
        text: String::new(),
        completed_steps: 1,
        tool_calls_completed: 1,
    })
}

impl RoutedProviderAttemptExecutor for FakeExecutor {
    fn run_attempt(
        &mut self,
        _attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(
            &RoutedProviderCheckpoint,
        ) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        self.resumes.push(resume_from.cloned());
        match self.behaviors.pop_front().expect("behavior") {
            Behavior::FailBeforeOutput => Err(
                ProviderSessionError::Transport(
                    "connection reset before output".into(),
                ),
            ),
            Behavior::OutputCheckpointThenFail => {
                on_text_delta("Checking ", "Checking ");
                on_checkpoint(&checkpoint())?;
                Err(ProviderSessionError::Transport(
                    "connection reset after accepted checkpoint".into(),
                ))
            }
            Behavior::OutputThenFail => {
                on_text_delta("partial", "partial");
                Err(ProviderSessionError::Transport(
                    "connection reset after partial output".into(),
                ))
            }
            Behavior::ToolCheckpointThenStallUntilCancelled => {
                on_checkpoint(&tool_only_checkpoint())?;
                while !should_cancel() {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(ProviderSessionError::Cancelled(
                    "post-tool continuation watchdog".into(),
                ))
            }
            Behavior::Success {
                delta,
                accumulated,
                result,
            } => {
                if !delta.is_empty() {
                    on_text_delta(delta, accumulated);
                }
                Ok(result.into())
            }
            Behavior::SilentUntilCancelled => {
                while !should_cancel() {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(ProviderSessionError::Cancelled(
                    "attempt cancellation observed".into(),
                ))
            }
        }
    }
}

#[derive(Default)]
struct FakeStore {
    saved: Mutex<Vec<RoutedProviderCheckpoint>>,
    fail: bool,
}

impl RoutedProviderCheckpointStore for FakeStore {
    fn persist(
        &self,
        checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        if self.fail {
            return Err(ProviderSessionError::Transport(
                "checkpoint persistence failed".into(),
            ));
        }
        self.saved
            .lock()
            .expect("saved checkpoints")
            .push(checkpoint.clone());
        Ok(format!(
            "checkpoint-{}",
            self.saved.lock().expect("saved checkpoints").len()
        ))
    }
}

fn policy(max_attempts: u32) -> StreamAttemptPolicy {
    StreamAttemptPolicy {
        first_output_timeout: Duration::from_millis(25),
        max_attempts,
        initial_backoff: Duration::ZERO,
        max_backoff: Duration::ZERO,
        max_retry_after: Duration::ZERO,
    }
}

#[derive(Clone, Copy)]
enum PermanentFailure {
    Authentication,
    Configuration,
}

impl PermanentFailure {
    fn error(self) -> ProviderSessionError {
        // Deliberately transient-looking text: only the explicit kind is authority.
        let message = "connection timeout (503) in a permanent credential/configuration failure".into();
        match self {
            Self::Authentication => ProviderSessionError::Authentication(message),
            Self::Configuration => ProviderSessionError::Configuration(message),
        }
    }

    fn report_type(self) -> &'static str {
        match self {
            Self::Authentication => "AuthenticationError",
            Self::Configuration => "ConfigurationError",
        }
    }
}

struct PermanentAfterWatchdog {
    failure: PermanentFailure,
    attempts: usize,
    unknown_effect: bool,
    user_cancellation: Option<RoutedProviderCancellation>,
}

impl RoutedProviderAttemptExecutor for PermanentAfterWatchdog {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        assert!(resume_from.is_none());
        if self.unknown_effect {
            let receipt = attempt.admit_tool().expect("admit before watchdog");
            receipt.result_tracker().mark_unknown(ToolEffectUnknownResult::DownstreamOutcomeUnknown);
            receipt.complete();
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        // Unknown custody can itself set should_cancel. Observe the actual
        // attempt retirement as well so this case still proves watchdog expiry.
        while attempt.is_current() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(!attempt.is_current(), "the real watchdog must retire the attempt");
        assert!(should_cancel());
        on_text_delta("stale", "stale");
        on_checkpoint(&tool_only_checkpoint())?;
        if let Some(cancellation) = &self.user_cancellation {
            cancellation.cancel("user cancelled after watchdog");
        }
        Err(self.failure.error())
    }
}

#[test]
fn production_permanent_failures_survive_watchdog_and_report_without_retry() {
    for failure in [PermanentFailure::Authentication, PermanentFailure::Configuration] {
        let adapter = ProductionTurnRunShellAdapter {
            policy: policy(3), watchdog_poll_interval: Duration::from_millis(1),
        };
        let store = FakeStore::default();
        let mut executor = PermanentAfterWatchdog {
            failure, attempts: 0, unknown_effect: false, user_cancellation: None,
        };
        let mut retries = Vec::new();
        let mut reports = Vec::new();
        let mut deltas = Vec::new();
        let error = adapter.run_with_retry_reporting(
            &RoutedProviderCancellation::default(), &store, &mut executor,
            &mut |delta, _| deltas.push(delta.to_string()),
            &mut |event| retries.push(event.clone()),
            &mut |report| reports.push(report.clone()),
        ).expect_err("permanent failure must not become a watchdog retry");
        assert_eq!(std::mem::discriminant(&error), std::mem::discriminant(&failure.error()));
        assert_eq!(executor.attempts, 1);
        assert!(retries.is_empty());
        assert!(deltas.is_empty());
        assert!(store.saved.lock().unwrap().is_empty());
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
        assert_eq!(reports[0].error_type, failure.report_type());
        assert_eq!(reports[0].error_code, "SAND-E0405");
        assert!(reports[0].watchdog_expired);
        assert!(!reports[0].resume_from_checkpoint);
        assert_eq!(reports[0].delay_ms, None);
        assert_eq!(reports[0].error, error.to_string());
    }
}

#[test]
fn production_user_cancel_and_unknown_custody_keep_precedence_over_permanent_failures() {
    for failure in [PermanentFailure::Authentication, PermanentFailure::Configuration] {
        for user_cancel in [false, true] {
            let adapter = ProductionTurnRunShellAdapter {
                policy: policy(3), watchdog_poll_interval: Duration::from_millis(1),
            };
            let cancellation = RoutedProviderCancellation::default();
            let runtime = Arc::new(StreamAttemptRuntime::new(adapter.policy.clone()));
            let store = FakeStore::default();
            let mut executor = PermanentAfterWatchdog {
                failure, attempts: 0, unknown_effect: !user_cancel,
                user_cancellation: user_cancel.then(|| cancellation.clone()),
            };
            let mut reports = Vec::new();
            let error = adapter.run_with_attempt_runtime_reporting(
                Arc::clone(&runtime), &cancellation, &store, &mut executor,
                &mut |_, _| panic!("stale output"),
                &mut |_| panic!("permanent/custody/cancel failure must not retry"),
                &mut |report| reports.push(report.clone()),
            ).expect_err("terminal precedence");
            assert_eq!(executor.attempts, 1);
            assert!(store.saved.lock().unwrap().is_empty());
            if user_cancel {
                assert!(matches!(error, ProviderSessionError::Cancelled(_)));
                assert!(reports.is_empty());
            } else {
                assert!(matches!(error, ProviderSessionError::Tool(_)));
                assert_eq!(runtime.effect_custody().unknown_result,
                    Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown));
                assert_eq!(reports.len(), 1);
                assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
                assert_eq!(reports[0].error_type, "ToolEffectCustodyError");
                assert_eq!(reports[0].error_code, "SAND-E0407");
                assert!(reports[0].watchdog_expired);
            }
        }
    }
}

#[test]
fn production_turn_adapter_retries_before_output() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(2),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([
        Behavior::FailBeforeOutput,
        Behavior::Success {
            delta: "done",
            accumulated: "done",
            result: "done",
        },
    ]));
    let mut deltas = Vec::new();

    let result = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |delta, _| deltas.push(delta.to_string()),
        )
        .expect("retry before output");

    assert_eq!(result, "done");
    assert_eq!(executor.attempts, 2);
    assert_eq!(executor.resumes, vec![None, None]);
    assert_eq!(deltas, vec!["done".to_string()]);
}

#[test]
fn production_turn_adapter_emits_retry_event_before_next_attempt() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(2),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([
        Behavior::FailBeforeOutput,
        Behavior::Success {
            delta: "done",
            accumulated: "done",
            result: "done",
        },
    ]));
    let mut retries = Vec::<ProviderRetryEvent>::new();

    let result = adapter
        .run_with_retry(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
            &mut |event| retries.push(event.clone()),
        )
        .expect("retry event path");

    assert_eq!(result, "done");
    assert_eq!(
        retries,
        vec![ProviderRetryEvent {
            attempt: 1,
            next_attempt: 2,
            delay_ms: 0,
            resume_from_checkpoint: false,
            watchdog_expired: false,
        }]
    );
}

#[test]
fn production_turn_adapter_reports_retried_then_exhausted_without_false_retry_activity() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(2),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([
        Behavior::FailBeforeOutput,
        Behavior::FailBeforeOutput,
    ]));
    let mut retries = Vec::<ProviderRetryEvent>::new();
    let mut reports = Vec::<ProviderRetryReport>::new();

    let error = adapter
        .run_with_retry_reporting(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
            &mut |event| retries.push(event.clone()),
            &mut |report| reports.push(report.clone()),
        )
        .expect_err("second retryable failure exhausts the bounded policy");

    assert!(error.to_string().contains("connection reset"));
    assert_eq!(retries.len(), 1, "terminal exhaustion is not another retry activity");
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].outcome, ProviderRetryOutcome::Retried);
    assert_eq!(reports[0].attempt, 1);
    assert_eq!(reports[0].max_attempts, 2);
    assert_eq!(reports[0].error_type, "TransportError");
    assert_eq!(reports[0].error_code, "SAND-E0406");
    assert!(reports[0].cause.contains("connection reset"));
    assert_eq!(reports[1].outcome, ProviderRetryOutcome::Exhausted);
    assert_eq!(reports[1].attempt, 2);
    assert_eq!(reports[1].max_attempts, 2);
    assert_eq!(reports[1].delay_ms, None);
}

#[test]
fn production_turn_adapter_reports_gave_up_when_partial_output_has_no_checkpoint() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(3),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([Behavior::OutputThenFail]));
    let mut reports = Vec::<ProviderRetryReport>::new();

    let _ = adapter
        .run_with_retry_reporting(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
            &mut |_| {},
            &mut |report| reports.push(report.clone()),
        )
        .expect_err("unsafe partial output must not retry");

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
    assert_eq!(reports[0].attempt, 1);
    assert_eq!(reports[0].max_attempts, 3);
}

#[test]
fn production_turn_adapter_doubles_first_output_deadline_after_retry() {
    let base = Duration::from_millis(25);
    assert_eq!(
        first_output_timeout_for_attempt(base, 1),
        Duration::from_millis(25)
    );
    assert_eq!(
        first_output_timeout_for_attempt(base, 2),
        Duration::from_millis(50)
    );
    assert_eq!(
        first_output_timeout_for_attempt(base, 3),
        Duration::from_millis(100)
    );
    assert_eq!(
        first_output_timeout_for_attempt(base, 20),
        Duration::from_millis(6_400)
    );
}

#[test]
fn production_turn_adapter_persists_checkpoint_before_resuming() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(2),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([
        Behavior::OutputCheckpointThenFail,
        Behavior::Success {
            delta: "done",
            accumulated: "Checking done",
            result: "Checking done",
        },
    ]));
    let mut deltas = Vec::new();

    let result = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |delta, _| deltas.push(delta.to_string()),
        )
        .expect("resume accepted checkpoint");

    assert_eq!(result, "Checking done");
    assert_eq!(executor.attempts, 2);
    assert!(executor.resumes[0].is_none());
    assert_eq!(executor.resumes[1], Some(checkpoint()));
    assert_eq!(
        store.saved.lock().expect("saved checkpoints").as_slice(),
        &[checkpoint()]
    );
    assert_eq!(deltas, vec!["Checking ".to_string(), "done".to_string()]);
}

#[test]
fn production_turn_adapter_never_retries_partial_output_without_new_checkpoint() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(3),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor =
        FakeExecutor::new(VecDeque::from([Behavior::OutputThenFail]));

    let error = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
        )
        .expect_err("partial output without checkpoint must fail closed");

    assert!(error.to_string().contains("connection reset"));
    assert_eq!(executor.attempts, 1);
}

#[test]
fn production_turn_adapter_does_not_accept_an_unpersisted_checkpoint() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(3),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore {
        saved: Mutex::new(Vec::new()),
        fail: true,
    };
    let mut executor =
        FakeExecutor::new(VecDeque::from([Behavior::OutputCheckpointThenFail]));

    let error = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
        )
        .expect_err("failed persistence must prevent resume");

    assert!(error.to_string().contains("checkpoint persistence failed"));
    assert_eq!(executor.attempts, 1);
}

#[test]
fn production_turn_adapter_real_store_refusal_retains_uncheckpointed_effect_and_never_retries() {
    struct CompletedEffectExecutor {
        effect_path: std::path::PathBuf,
        attempts: usize,
        resumes: Vec<Option<RoutedProviderCheckpoint>>,
    }

    impl RoutedProviderAttemptExecutor for CompletedEffectExecutor {
        fn run_attempt(
            &mut self,
            attempt: &RoutedProviderAttemptControl,
            resume_from: Option<&RoutedProviderCheckpoint>,
            on_text_delta: &mut dyn FnMut(&str, &str),
            on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
            should_cancel: &dyn Fn() -> bool,
        ) -> Result<String, ProviderSessionError> {
            use std::io::Write;
            self.attempts += 1;
            self.resumes.push(resume_from.cloned());
            assert!(!should_cancel());
            let receipt = attempt.admit_tool().expect("admit one real file effect");
            let mut effect = std::fs::OpenOptions::new()
                .create(true).append(true).open(&self.effect_path).expect("effect file");
            effect.write_all(b"x").expect("count the actual effect");
            effect.sync_all().expect("persist effect receipt");
            drop(effect);
            receipt.complete();
            on_text_delta("Checking ", "Checking ");
            on_checkpoint(&checkpoint())?;
            Err(ProviderSessionError::Transport("connection reset after checkpoint callback".into()))
        }
    }

    let root = std::env::temp_dir().join(format!(
        "fabushi-checkpoint-store-adapter-{}-{}", std::process::id(), uuid::Uuid::new_v4(),
    ));
    std::fs::create_dir(&root).expect("exclusive fixture root");
    let store = ProductionRoutedProviderCheckpointStore::new(&root, "agent-a", "original-stream");
    std::fs::create_dir_all(store.directory().parent().expect("checkpoint parent"))
        .expect("fixture parent");
    let original_bytes = b"existing non-directory checkpoint entry";
    std::fs::write(store.directory(), original_bytes).expect("refusing real store entry");
    let effect_path = root.join("effect-count");
    let mut executor = CompletedEffectExecutor { effect_path: effect_path.clone(), attempts: 0, resumes: Vec::new() };
    let adapter = ProductionTurnRunShellAdapter {
        policy: StreamAttemptPolicy { first_output_timeout: Duration::from_secs(5), ..policy(3) },
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let runtime = Arc::new(StreamAttemptRuntime::new(adapter.policy.clone()));
    let cancellation = RoutedProviderCancellation::default();
    let mut retries = Vec::new();
    let mut reports = Vec::new();
    let mut deltas = Vec::new();
    adapter.run_with_attempt_runtime_reporting(
        Arc::clone(&runtime), &cancellation, &store, &mut executor,
        &mut |delta, _| deltas.push(delta.to_string()),
        &mut |event| retries.push(event.clone()),
        &mut |report| reports.push(report.clone()),
    ).expect_err("a real publication refusal cannot cover the completed effect");

    assert_eq!(executor.attempts, 1);
    assert_eq!(executor.resumes, vec![None]);
    assert_eq!(std::fs::read(&effect_path).expect("actual effect count"), b"x");
    assert_eq!(std::fs::read(store.directory()).expect("preserved refusing entry"), original_bytes);
    assert!(retries.is_empty());
    assert_eq!(deltas, vec!["Checking "]);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
    assert!(!reports[0].resume_from_checkpoint);
    let custody = runtime.effect_custody();
    assert_eq!(custody.started_revision, 1);
    assert_eq!(custody.checkpoint_revision, 0);
    assert_eq!(custody.pending_effects, 0);
    assert_eq!(custody.unknown_result, None);
    assert!(custody.retry_block_reason().is_some());

    // Explicitly repair only this fixture's path. Repairing storage must not
    // erase the original invocation's uncheckpointed effect or restart it.
    std::fs::remove_file(store.directory()).expect("explicit fixture repair");
    std::fs::create_dir(store.directory()).expect("repaired checkpoint directory");
    adapter.run_with_attempt_runtime_reporting(
        Arc::clone(&runtime), &cancellation, &store, &mut executor,
        &mut |_, _| panic!("repaired storage cannot reopen original output"),
        &mut |_| panic!("repaired storage cannot authorize replay"),
        &mut |_| panic!("repaired storage cannot start another attempt"),
    ).expect_err("the same custody remains ineligible after repair");
    assert_eq!(runtime.effect_custody(), custody);
    assert_eq!(executor.attempts, 1);
    assert_eq!(executor.resumes, vec![None]);
    assert_eq!(std::fs::read(effect_path).expect("no repeated effect"), b"x");
    assert_eq!(std::fs::read_dir(store.directory()).expect("no accepted payload").count(), 0);
    std::fs::remove_dir_all(root).expect("remove only successful owned fixture");
}

#[test]
fn production_turn_adapter_first_output_watchdog_interrupts_silent_attempt() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(1),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor =
        FakeExecutor::new(VecDeque::from([Behavior::SilentUntilCancelled]));

    let error = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
        )
        .expect_err("silent attempt must hit first-output watchdog");

    assert!(error.to_string().contains("first-output watchdog"));
    assert_eq!(executor.attempts, 1);
}

#[test]
fn production_turn_adapter_tool_only_checkpoint_rearms_watchdog_and_resumes() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(2),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let store = FakeStore::default();
    let mut executor = FakeExecutor::new(VecDeque::from([
        Behavior::ToolCheckpointThenStallUntilCancelled,
        Behavior::Success {
            delta: "done",
            accumulated: "done",
            result: "done",
        },
    ]));

    let result = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
        )
        .expect("post-tool watchdog must resume from the durable checkpoint");

    assert_eq!(result, "done");
    assert_eq!(executor.attempts, 2);
    assert_eq!(executor.resumes[0], None);
    assert_eq!(executor.resumes[1], Some(tool_only_checkpoint()));
    assert_eq!(
        store.saved.lock().expect("saved checkpoints").as_slice(),
        &[tool_only_checkpoint()]
    );
}

#[test]
fn production_turn_adapter_user_cancellation_wins_without_retry() {
    let adapter = ProductionTurnRunShellAdapter {
        policy: policy(3),
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    cancellation.cancel("user cancelled");
    let store = FakeStore::default();
    let mut executor =
        FakeExecutor::new(VecDeque::from([Behavior::FailBeforeOutput]));

    let error = adapter
        .run(
            &cancellation,
            &store,
            &mut executor,
            &mut |_delta, _| {},
        )
        .expect_err("user cancellation must settle immediately");

    assert!(matches!(error, ProviderSessionError::Cancelled(_)));
    assert_eq!(executor.attempts, 0);
}

#[test]
fn production_turn_lifecycle_freezes_prepared_input_before_execution_and_unwinds() {
    let mut shell = TurnRunShell::default();
    let mut last_finished = None;
    let upgrade_quiescing = Arc::new(AtomicBool::new(false));
    let turn_quiesced = Arc::new(AtomicBool::new(false));
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "  build app  ".into(),
    }];
    let phases = Arc::new(Mutex::new(Vec::<String>::new()));

    let result = run_production_turn_shell_lifecycle(
        &mut shell,
        &mut last_finished,
        &messages,
        TurnRunOptions {
            inference_request_id: Some("request-1".into()),
            ..TurnRunOptions::default()
        },
        upgrade_quiescing,
        turn_quiesced,
        {
            let phases = Arc::clone(&phases);
            move |prompt, started| {
                phases
                    .lock()
                    .expect("phases")
                    .push(format!("prepare:{prompt}:{}", started.owner.request_id));
                Ok(vec!["frozen".to_string()])
            }
        },
        {
            let phases = Arc::clone(&phases);
            move |prepared| {
                assert_eq!(prepared.prompt, "build app");
                assert_eq!(prepared.started.owner.request_id, "request-1");
                assert_eq!(prepared.prepared, vec!["frozen".to_string()]);
                phases.lock().expect("phases").push("execute".into());
                Ok("done".into())
            }
        },
        {
            let phases = Arc::clone(&phases);
            move |started| {
                phases
                    .lock()
                    .expect("phases")
                    .push(format!("unwind:{}", started.owner.request_id));
            }
        },
    )
    .expect("production lifecycle");

    assert_eq!(result, "done");
    assert!(!shell.has_active_run());
    assert!(matches!(
        last_finished.as_ref().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Completed)
    ));
    assert_eq!(
        phases.lock().expect("phases").as_slice(),
        &[
            "prepare:build app:request-1".to_string(),
            "execute".to_string(),
            "unwind:request-1".to_string(),
        ]
    );
}

#[test]
fn production_turn_lifecycle_unwinds_and_settles_failed_prepare() {
    let mut shell = TurnRunShell::default();
    let mut last_finished = None;
    let unwound = Arc::new(AtomicBool::new(false));
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "fail safely".into(),
    }];

    let error = run_production_turn_shell_lifecycle(
        &mut shell,
        &mut last_finished,
        &messages,
        TurnRunOptions::default(),
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        |_prompt, _started| {
            Err::<(), _>(ProviderSessionError::Protocol(
                "prepare failed".into(),
            ))
        },
        |_prepared| -> Result<String, ProviderSessionError> {
            panic!("execute must not run after prepare failure")
        },
        {
            let unwound = Arc::clone(&unwound);
            move |_started| {
                unwound.store(true, Ordering::Release);
            }
        },
    )
    .expect_err("prepare failure must settle the turn");

    assert!(error.to_string().contains("prepare failed"));
    assert!(unwound.load(Ordering::Acquire));
    assert!(!shell.has_active_run());
    assert!(matches!(
        last_finished.as_ref().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Failed {
            retryable: false,
            ..
        })
    ));
}
