use super::*;
use async_trait::async_trait;
use mahayana_core::Message;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const WAIT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy)]
enum ProviderOutcome {
    Complete,
    Fail,
    UsageLimit,
}

struct CountedProvider {
    requests: Mutex<Vec<SendMessageRequest>>,
    outcome: ProviderOutcome,
}

impl CountedProvider {
    fn new(outcome: ProviderOutcome) -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
            outcome,
        }
    }
}

#[async_trait]
impl ConversationProvider for CountedProvider {
    fn key(&self) -> &'static str {
        "mahayana-ai"
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, ConversationError> {
        Ok(Vec::new())
    }

    async fn history(
        &self,
        _conversation_id: &ConversationId,
        _limit: u32,
    ) -> Result<Vec<Message>, ConversationError> {
        Ok(Vec::new())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
        _events: SharedConversationEventSink,
    ) -> Result<(), ConversationError> {
        // Count the actual provider entry even when that provider returns an error.
        self.requests.lock().expect("provider requests").push(request);
        match self.outcome {
            ProviderOutcome::Complete => Ok(()),
            ProviderOutcome::Fail => Err(ConversationError::Provider("entered provider".into())),
            ProviderOutcome::UsageLimit => {
                Err(ConversationError::UsageLimitExceeded("entered provider".into()))
            }
        }
    }

    async fn interrupt(&self, _operation_id: &OperationId) -> Result<(), ConversationError> {
        Ok(())
    }

    async fn resolve_approval(
        &self,
        _request: ResolveApprovalRequest,
    ) -> Result<(), ConversationError> {
        Ok(())
    }
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mahayana-dispatch-admission-{}-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir(&path).expect("create isolated store directory");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if !std::thread::panicking() {
                panic!("remove isolated dispatch-admission directory: {error}");
            }
        }
    }
}

struct Fixture {
    // Drop the runtime and both database connections before removing this directory.
    runtime: MahayanaRuntime,
    observer: Connection,
    provider: Arc<CountedProvider>,
    _directory: TestDirectory,
}

impl Fixture {
    fn new(outcome: ProviderOutcome) -> Self {
        let directory = TestDirectory::new();
        let provider = Arc::new(CountedProvider::new(outcome));
        let runtime = RuntimeBuilder::new(RuntimeConfig {
            data_dir: Some(directory.0.clone()),
            ..RuntimeConfig::default()
        })
        .with_provider(provider.clone())
        .expect("register counted provider")
        .build()
        .expect("open actual runtime with SQLite");
        let observer = Connection::open(directory.0.join("runtime.sqlite3"))
            .expect("open independent SQLite observer");
        observer
            .execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .expect("configure SQLite observer");
        Self { runtime, observer, provider, _directory: directory }
    }

    fn actor(&self, conversation: &ConversationId) -> Arc<ConversationActor> {
        self.runtime.actors.actor(conversation).expect("actual conversation actor")
    }

    fn assert_accepted(&self, operation: &OperationId, conversation: &ConversationId) -> String {
        let (turn_id, turn_state, run_state, stored_conversation): (String, String, String, String) =
            self.observer
                .query_row(
                    "SELECT t.turn_id, t.state, r.state, t.conversation_id
                     FROM turns t JOIN runs r ON r.turn_id = t.turn_id WHERE r.run_id = ?1",
                    params![operation.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .expect("accepted turn and run persisted before injection");
        assert_eq!(turn_state, "accepted");
        assert_eq!(run_state, "accepted");
        assert_eq!(stored_conversation, conversation.as_str());
        assert!(self.runtime.operations.lock().unwrap().contains_key(operation));
        let contexts = self.runtime.run_contexts.lock().unwrap();
        let context = contexts.get(operation).expect("accepted run context");
        assert_eq!(context.turn_id.as_str(), turn_id);
        assert_eq!(context.run_id.as_str(), operation.as_str());
        assert_eq!(&context.conversation_id, conversation);
        assert!(self.provider.requests.lock().unwrap().is_empty());
        turn_id
    }

    fn reject_state(&self, table: &str, id: &str, state: TurnState, cause: &str) {
        let id_column = match table {
            "turns" => "turn_id",
            "runs" => "run_id",
            _ => panic!("only the actual turn/run state tables are injection targets"),
        };
        self.observer
            .execute_batch(&format!(
                "CREATE TRIGGER reject_dispatch BEFORE UPDATE OF state ON {table}
                 WHEN OLD.{id_column} = '{}' AND NEW.state = '{}'
                 BEGIN SELECT RAISE(ABORT, '{}'); END;",
                id.replace('\'', "''"),
                state.as_str(),
                cause.replace('\'', "''"),
            ))
            .expect("install operation-scoped SQLite refusal");
    }

    fn assert_failed_rows(&self, operation: &OperationId, turn_id: &str) {
        let (state, active): (String, Option<String>) = self.observer.query_row(
            "SELECT state, active_run_id FROM turns WHERE turn_id = ?1",
            params![turn_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(state, "failed");
        assert_eq!(active, None);
        let (state, finished): (String, Option<i64>) = self.observer.query_row(
            "SELECT state, finished_at_ms FROM runs WHERE run_id = ?1",
            params![operation.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(state, "failed");
        assert!(finished.is_some());
    }

    fn assert_later_send(&self, conversation: &ConversationId, rejected: &OperationId) {
        let actor = self.actor(conversation);
        let later = accept(&self.runtime, conversation, "client:later", "independent later message");
        assert_ne!(&later, rejected);
        let events = settle(&self.runtime, &actor, &later);
        assert_completed(&events, &later);
        assert_eq!(states(&events, &later), vec![
            TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
        ]);
        assert_one_request(
            &self.provider, &later, conversation, "client:later", "independent later message",
        );
    }
}

fn conversation(name: &str) -> ConversationId {
    ConversationId::new(format!("mahayana-ai:agent:admission-{name}")).unwrap()
}

fn accept(
    runtime: &MahayanaRuntime,
    conversation: &ConversationId,
    client_message_id: &str,
    text: &str,
) -> OperationId {
    let response = runtime.execute(RuntimeCommand::SendMessage {
        conversation_id: conversation.clone(),
        text: text.to_string(),
        client_message_id: Some(client_message_id.to_string()),
        retry_of_client_message_id: None,
        inference_provider: Some("test-inference".to_string()),
        hidden: false,
    }).expect("send accepted before dispatch");
    let RuntimeResponse::Accepted { operation_id } = response else {
        panic!("expected accepted response");
    };
    operation_id
}

fn settle(
    runtime: &MahayanaRuntime,
    actor: &ConversationActor,
    operation: &OperationId,
) -> Vec<RuntimeEvent> {
    let deadline = Instant::now() + WAIT;
    let mut events = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "terminal event timed out for {operation}");
        let event = runtime.receive(remaining).expect("receive runtime event")
            .expect("terminal event before deadline");
        let terminal = matches!(&event,
            RuntimeEvent::OperationCompleted { operation_id }
            | RuntimeEvent::OperationFailed { operation_id, .. } if operation_id == operation
        );
        events.push(event);
        if terminal {
            break;
        }
    }
    // The dispatch task holds this real gate through event emission and cleanup.
    // Acquiring it proves retirement has finished, without a timing sleep.
    let _settled_gate = runtime.async_runtime.block_on(async {
        tokio::time::timeout(WAIT, actor.gate.lock()).await.expect("dispatch cleanup before deadline")
    });
    assert!(!runtime.operations.lock().unwrap().contains_key(operation));
    assert!(!runtime.run_contexts.lock().unwrap().contains_key(operation));
    events.extend(runtime.event_rx.try_iter());
    events
}

fn states(events: &[RuntimeEvent], operation: &OperationId) -> Vec<TurnState> {
    events.iter().filter_map(|event| match event {
        RuntimeEvent::TurnStateChanged { operation_id, state, .. } if operation_id == operation => {
            Some(*state)
        }
        _ => None,
    }).collect()
}

fn assert_failed(events: &[RuntimeEvent], operation: &OperationId, code: &str, message: &str) {
    let failures: Vec<_> = events.iter().filter_map(|event| match event {
        RuntimeEvent::OperationFailed { operation_id, code, message } if operation_id == operation => {
            Some((code.as_str(), message.as_str()))
        }
        _ => None,
    }).collect();
    assert_eq!(failures, vec![(code, message)]);
    assert!(!events.iter().any(|event| matches!(event,
        RuntimeEvent::OperationCompleted { operation_id } if operation_id == operation
    )));
}

fn assert_completed(events: &[RuntimeEvent], operation: &OperationId) {
    assert_eq!(events.iter().filter(|event| matches!(event,
        RuntimeEvent::OperationCompleted { operation_id } if operation_id == operation
    )).count(), 1);
    assert!(!events.iter().any(|event| matches!(event,
        RuntimeEvent::OperationFailed { operation_id, .. } if operation_id == operation
    )));
}

fn sqlite_failure(cause: &str) -> String {
    RuntimeError::RuntimeStore(RuntimeStoreError::Sqlite(cause.to_string())).to_string()
}

fn assert_one_request(
    provider: &CountedProvider,
    operation: &OperationId,
    conversation: &ConversationId,
    client_message_id: &str,
    text: &str,
) {
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "actual provider entry count");
    let request = &requests[0];
    assert_eq!(&request.operation_id, operation);
    assert_eq!(&request.conversation_id, conversation);
    assert_eq!(request.client_message_id.as_deref(), Some(client_message_id));
    assert_eq!(request.text, text);
    assert_eq!(request.inference_provider.as_deref(), Some("test-inference"));
    assert!(!request.hidden);
}

fn refuses_state_write(table: &str, phase: TurnState) {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("state-refusal");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:rejected", "must not enter provider");
    let turn_id = fixture.assert_accepted(&operation, &conversation);
    let target = if table == "turns" { turn_id.as_str() } else { operation.as_str() };
    let cause = format!("refused {} {table} write", phase.as_str());
    fixture.reject_state(table, target, phase, &cause);
    drop(gate);

    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, "runtime_admission_failed", &sqlite_failure(&cause));
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    let mut expected = vec![TurnState::Accepted];
    if phase == TurnState::Thinking {
        expected.push(TurnState::Preparing);
    }
    expected.push(TurnState::Failed);
    assert_eq!(states(&events, &operation), expected);
    fixture.assert_failed_rows(&operation, &turn_id);

    fixture.observer.execute_batch("DROP TRIGGER reject_dispatch;").unwrap();
    fixture.assert_later_send(&conversation, &operation);
    fixture.assert_failed_rows(&operation, &turn_id);
    assert_failed(&events, &operation, "runtime_admission_failed", &sqlite_failure(&cause));
}

#[test]
fn preparing_turn_write_refusal_never_enters_provider_and_later_send_succeeds() {
    refuses_state_write("turns", TurnState::Preparing);
}

#[test]
fn preparing_run_write_refusal_never_enters_provider_and_later_send_succeeds() {
    refuses_state_write("runs", TurnState::Preparing);
}

#[test]
fn thinking_turn_write_refusal_never_enters_provider_and_later_send_succeeds() {
    refuses_state_write("turns", TurnState::Thinking);
}

#[test]
fn thinking_run_write_refusal_never_enters_provider_and_later_send_succeeds() {
    refuses_state_write("runs", TurnState::Thinking);
}

#[test]
fn missing_run_after_accepted_turn_write_refuses_provider_and_retires_ownership() {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("missing-run");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:rejected", "missing run");
    let turn_id = fixture.assert_accepted(&operation, &conversation);
    assert_eq!(fixture.observer.execute(
        "DELETE FROM runs WHERE run_id = ?1", params![operation.as_str()],
    ).unwrap(), 1);
    drop(gate);

    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, "runtime_admission_failed", &sqlite_failure(&format!(
        "run state update for {operation} affected 0 rows; expected 1",
    )));
    assert_eq!(states(&events, &operation), vec![TurnState::Accepted]);
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    let state: String = fixture.observer.query_row(
        "SELECT state FROM turns WHERE turn_id = ?1", params![turn_id], |row| row.get(0),
    ).unwrap();
    assert_eq!(state, "failed", "failed bookkeeping still attempts the existing turn");
    let count: i64 = fixture.observer.query_row(
        "SELECT COUNT(*) FROM runs WHERE run_id = ?1", params![operation.as_str()], |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 0, "admission must not reconstruct a missing run");
    fixture.assert_later_send(&conversation, &operation);
}

#[test]
fn missing_turn_after_acceptance_refuses_provider_without_reconstruction() {
    let mut fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("missing-turn");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:rejected", "missing turn");
    let turn_id = fixture.assert_accepted(&operation, &conversation);
    let transaction = fixture.observer.transaction().unwrap();
    assert_eq!(transaction.execute("DELETE FROM turn_requests WHERE turn_id = ?1", params![&turn_id]).unwrap(), 1);
    assert_eq!(transaction.execute("DELETE FROM runs WHERE run_id = ?1", params![operation.as_str()]).unwrap(), 1);
    assert_eq!(transaction.execute("DELETE FROM turns WHERE turn_id = ?1", params![&turn_id]).unwrap(), 1);
    transaction.commit().unwrap();
    drop(gate);

    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, "runtime_admission_failed", &sqlite_failure(&format!(
        "turn state update for {turn_id} affected 0 rows; expected 1",
    )));
    assert_eq!(states(&events, &operation), vec![TurnState::Accepted]);
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    let count: i64 = fixture.observer.query_row(
        "SELECT COUNT(*) FROM turns WHERE turn_id = ?1", params![turn_id], |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 0, "admission must not reconstruct a missing turn");
    fixture.assert_later_send(&conversation, &operation);
}

#[test]
fn failed_bookkeeping_preserves_original_admission_error_and_cleanup() {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("bookkeeping-refusal");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:rejected", "two distinct refusals");
    let turn_id = fixture.assert_accepted(&operation, &conversation);
    fixture.reject_state("turns", &turn_id, TurnState::Preparing, "original admission refusal");
    fixture.observer.execute_batch(&format!(
        "CREATE TRIGGER reject_failed_bookkeeping BEFORE UPDATE OF state ON turns
         WHEN OLD.turn_id = '{}' AND NEW.state = 'failed'
         BEGIN SELECT RAISE(ABORT, 'later bookkeeping refusal'); END;",
        turn_id.replace('\'', "''"),
    )).unwrap();
    drop(gate);

    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, "runtime_admission_failed", &sqlite_failure("original admission refusal"));
    assert_eq!(states(&events, &operation), vec![TurnState::Accepted]);
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    fixture.observer.execute_batch("DROP TRIGGER reject_dispatch; DROP TRIGGER reject_failed_bookkeeping;").unwrap();
    fixture.assert_later_send(&conversation, &operation);
}

#[test]
fn actor_start_poison_refuses_provider_and_retires_accepted_operation() {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("poisoned-actor");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:rejected", "poisoned owner");
    fixture.assert_accepted(&operation, &conversation);
    actor.poison_state_for_admission_test();
    drop(gate);

    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, "runtime_admission_failed",
        "runtime synchronization failed: conversation actor mutex poisoned");
    assert_eq!(states(&events, &operation), vec![TurnState::Accepted]);
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    // A poisoned actor remains refused; this does not reset or replace that owner.
    fixture.assert_later_send(&ConversationId::new("mahayana-ai:agent:healthy-owner").unwrap(), &operation);
}

#[test]
fn admission_cleanup_preserves_another_conversations_accepted_operation() {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let rejected_conversation = conversation("isolated-refusal");
    let other_conversation = conversation("isolated-healthy");
    let rejected_actor = fixture.actor(&rejected_conversation);
    let other_actor = fixture.actor(&other_conversation);
    let rejected_gate = fixture.runtime.async_runtime.block_on(rejected_actor.gate.lock());
    let other_gate = fixture.runtime.async_runtime.block_on(other_actor.gate.lock());
    let rejected = accept(&fixture.runtime, &rejected_conversation, "client:rejected", "refused lane");
    let turn_id = fixture.assert_accepted(&rejected, &rejected_conversation);
    let other = accept(&fixture.runtime, &other_conversation, "client:other", "other lane");
    fixture.assert_accepted(&other, &other_conversation);
    fixture.reject_state("turns", &turn_id, TurnState::Preparing, "one lane refused");
    drop(rejected_gate);

    let mut events = settle(&fixture.runtime, &rejected_actor, &rejected);
    assert_failed(&events, &rejected, "runtime_admission_failed", &sqlite_failure("one lane refused"));
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    fixture.assert_accepted(&other, &other_conversation);
    drop(other_gate);
    events.extend(settle(&fixture.runtime, &other_actor, &other));
    assert_completed(&events, &other);
    assert_eq!(states(&events, &other), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_one_request(&fixture.provider, &other, &other_conversation, "client:other", "other lane");
    assert_failed(&events, &rejected, "runtime_admission_failed", &sqlite_failure("one lane refused"));
}

#[test]
fn healthy_sqlite_admission_enters_provider_once_after_gate_release() {
    let fixture = Fixture::new(ProviderOutcome::Complete);
    let conversation = conversation("healthy");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:healthy", "healthy message");
    fixture.assert_accepted(&operation, &conversation);
    drop(gate);
    let events = settle(&fixture.runtime, &actor, &operation);
    assert_completed(&events, &operation);
    assert_eq!(states(&events, &operation), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_one_request(&fixture.provider, &operation, &conversation, "client:healthy", "healthy message");
}

fn provider_failure_keeps_classification(outcome: ProviderOutcome, code: &str, message: &str) {
    let fixture = Fixture::new(outcome);
    let conversation = conversation("entered-provider");
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    let operation = accept(&fixture.runtime, &conversation, "client:entered", "provider returns failure");
    let turn_id = fixture.assert_accepted(&operation, &conversation);
    drop(gate);
    let events = settle(&fixture.runtime, &actor, &operation);
    assert_failed(&events, &operation, code, message);
    assert_eq!(states(&events, &operation), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Failed,
    ]);
    assert_one_request(&fixture.provider, &operation, &conversation, "client:entered", "provider returns failure");
    fixture.assert_failed_rows(&operation, &turn_id);
}

#[test]
fn entered_provider_error_keeps_provider_error_code_and_one_entry() {
    provider_failure_keeps_classification(ProviderOutcome::Fail, "provider_error", "provider failed: entered provider");
}

#[test]
fn entered_usage_limit_keeps_usage_limit_code_and_one_entry() {
    provider_failure_keeps_classification(ProviderOutcome::UsageLimit, "usage_limit_exceeded", "model usage limit exceeded: entered provider");
}

#[test]
fn explicit_in_memory_mode_keeps_no_storage_setters_and_normal_admission() {
    let provider = Arc::new(CountedProvider::new(ProviderOutcome::Complete));
    let runtime = RuntimeBuilder::new(RuntimeConfig::default())
        .with_provider(provider.clone()).unwrap().build().unwrap();
    runtime.store.set_turn_state(&TurnId::new("turn:not-persisted").unwrap(), TurnState::Thinking, None).unwrap();
    runtime.store.set_run_state(&RunId::new("run:not-persisted").unwrap(), TurnState::Thinking, None).unwrap();
    let conversation = conversation("in-memory");
    let actor = runtime.actors.actor(&conversation).unwrap();
    let operation = accept(&runtime, &conversation, "client:in-memory", "explicit non-durable mode");
    let events = settle(&runtime, &actor, &operation);
    assert_completed(&events, &operation);
    assert_eq!(states(&events, &operation), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_one_request(&provider, &operation, &conversation, "client:in-memory", "explicit non-durable mode");
}
