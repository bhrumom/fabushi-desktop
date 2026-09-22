use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::extensions::inference::provider_session::{
    OpenRouterCheckpoint, ProviderSessionError,
    RoutedProviderCheckpoint,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, RoutedProviderAttemptExecutor,
    RoutedProviderCheckpointStore,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation,
};
use mahayana_host_runtime::runner::StreamAttemptPolicy;
use serde_json::json;

#[derive(Clone)]
enum Behavior {
    FailBeforeOutput,
    OutputCheckpointThenFail,
    OutputThenFail,
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

impl RoutedProviderAttemptExecutor for FakeExecutor {
    fn run_attempt(
        &mut self,
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
