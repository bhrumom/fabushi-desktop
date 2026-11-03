use super::*;
use async_trait::async_trait;
use mahayana_core::Message;
use rusqlite::{params, types::Value as SqlValue, Connection};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::oneshot;

const WAIT: Duration = Duration::from_secs(10);

struct AdmissionProvider {
    requests: Mutex<Vec<SendMessageRequest>>,
    holds: Mutex<HashMap<String, oneshot::Receiver<()>>>,
    entered_tx: Sender<OperationId>,
    entered_rx: Receiver<OperationId>,
}

impl AdmissionProvider {
    fn new() -> Self {
        let (entered_tx, entered_rx) = crossbeam_channel::unbounded();
        Self {
            requests: Mutex::new(Vec::new()),
            holds: Mutex::new(HashMap::new()),
            entered_tx,
            entered_rx,
        }
    }

    fn hold(&self, text: &str) -> oneshot::Sender<()> {
        let (release, wait) = oneshot::channel();
        assert!(self.holds.lock().unwrap().insert(text.to_string(), wait).is_none());
        release
    }
}

#[async_trait]
impl ConversationProvider for AdmissionProvider {
    fn key(&self) -> &'static str { "mahayana-ai" }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, ConversationError> {
        Ok(Vec::new())
    }

    async fn history(
        &self, _conversation_id: &ConversationId, _limit: u32,
    ) -> Result<Vec<Message>, ConversationError> {
        Ok(Vec::new())
    }

    async fn send_message(
        &self, request: SendMessageRequest, _events: SharedConversationEventSink,
    ) -> Result<(), ConversationError> {
        let wait = self.holds.lock().unwrap().remove(&request.text);
        let operation = request.operation_id.clone();
        self.requests.lock().unwrap().push(request);
        self.entered_tx.send(operation).expect("record actual provider entry");
        if let Some(wait) = wait {
            wait.await.map_err(|_| ConversationError::Provider("test hold abandoned".into()))?;
        }
        Ok(())
    }

    async fn interrupt(&self, _operation_id: &OperationId) -> Result<(), ConversationError> {
        Ok(())
    }

    async fn resolve_approval(&self, _request: ResolveApprovalRequest) -> Result<(), ConversationError> {
        Ok(())
    }
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mahayana-command-acceptance-{}-{}-{}", std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir(&path).expect("isolated acceptance store directory");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if !std::thread::panicking() {
                panic!("remove isolated acceptance directory: {error}");
            }
        }
    }
}

struct Fixture {
    // Drop both SQLite owners and the runtime before removing their directory.
    runtime: MahayanaRuntime,
    observer: Connection,
    provider: Arc<AdmissionProvider>,
    directory: TestDirectory,
}

impl Fixture {
    fn new() -> Self { Self::open(TestDirectory::new()) }

    fn open(directory: TestDirectory) -> Self {
        let provider = Arc::new(AdmissionProvider::new());
        let runtime = RuntimeBuilder::new(RuntimeConfig {
            data_dir: Some(directory.0.clone()),
            ..RuntimeConfig::default()
        }).with_provider(provider.clone()).expect("register acceptance provider")
            .build().expect("open actual Runtime and recovery entrypoint");
        // Consume only the constructor's mandatory Ready event. Any recovery
        // admission/event remains observable by the reopen negative oracle.
        assert!(matches!(runtime.event_rx.try_recv().expect("constructor Ready event"),
            RuntimeEvent::Ready { .. }));
        let observer = Connection::open(directory.0.join("runtime.sqlite3"))
            .expect("independent SQLite observer");
        observer.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .expect("configure observer");
        Self { runtime, observer, provider, directory }
    }

    fn reopen(self) -> Self {
        let Self { runtime, observer, provider, directory } = self;
        drop(runtime);
        drop(observer);
        drop(provider);
        Self::open(directory)
    }

    fn actor(&self, conversation: &ConversationId) -> Arc<ConversationActor> {
        self.runtime.actors.actor(conversation).expect("actual actor")
    }

    fn rows(&self) -> Vec<(&'static str, Vec<Vec<SqlValue>>)> {
        ["conversations", "turns", "turn_requests", "runs"].into_iter().map(|table| {
            let mut statement = self.observer.prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
                .expect("read all admission fields");
            let columns = statement.column_count();
            let rows = statement.query_map([], |row| {
                (0..columns).map(|column| row.get::<_, SqlValue>(column)).collect::<rusqlite::Result<Vec<_>>>()
            }).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            (table, rows)
        }).collect()
    }

    fn turn_for(&self, operation: &OperationId) -> TurnId {
        self.observer.query_row("SELECT turn_id FROM runs WHERE run_id = ?1",
            params![operation.as_str()], |row| row.get::<_, String>(0))
            .map(TurnId).expect("persisted run owns a turn")
    }

    fn assert_no_owners_or_events(&self) {
        assert!(self.runtime.operations.lock().unwrap().is_empty());
        assert!(self.runtime.run_contexts.lock().unwrap().is_empty());
        let events: Vec<_> = self.runtime.event_rx.try_iter().collect();
        assert!(events.is_empty(), "refused command published events: {events:?}");
    }

    fn reject_insert(&self, table: &str, message: &str, queued_only: bool, cause: &str) {
        let message = message.replace('\'', "''");
        let target = match table {
            "turns" => format!("NEW.user_message_id = '{message}'"),
            "turn_requests" => format!("NEW.message_id = '{message}'"),
            "runs" => format!("NEW.turn_id IN (SELECT turn_id FROM turns WHERE user_message_id = '{message}')"),
            _ => panic!("only actual admission tables are fault targets"),
        };
        let queued = if queued_only { " AND NEW.state = 'queued'" } else { "" };
        self.observer.execute_batch(&format!(
            "CREATE TRIGGER refuse_admission BEFORE INSERT ON {table}
             WHEN {target}{queued} BEGIN SELECT RAISE(ABORT, '{}'); END;",
            cause.replace('\'', "''"),
        )).expect("install conditional SQLite refusal");
    }

    fn remove_refusal(&self) {
        self.observer.execute_batch("DROP TRIGGER refuse_admission;").unwrap();
    }

    fn settle(&self, conversation: &ConversationId, operations: &[OperationId]) -> Vec<RuntimeEvent> {
        let deadline = Instant::now() + WAIT;
        let mut terminal = HashSet::new();
        let mut events = Vec::new();
        while terminal.len() < operations.len() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "accepted operations did not settle");
            let event = self.runtime.receive(remaining).unwrap().expect("terminal event before deadline");
            match &event {
                RuntimeEvent::OperationCompleted { operation_id }
                | RuntimeEvent::OperationFailed { operation_id, .. } if operations.contains(operation_id) => {
                    terminal.insert(operation_id.clone());
                }
                _ => {}
            }
            events.push(event);
        }
        let actor = self.actor(conversation);
        let _retired = self.runtime.async_runtime.block_on(async {
            tokio::time::timeout(WAIT, actor.gate.lock()).await.expect("cleanup releases actual actor gate")
        });
        for operation in operations {
            assert!(!self.runtime.operations.lock().unwrap().contains_key(operation));
            assert!(!self.runtime.run_contexts.lock().unwrap().contains_key(operation));
            assert!(events.iter().any(|event| matches!(event,
                RuntimeEvent::OperationCompleted { operation_id } if operation_id == operation
            )), "operation failed: {events:?}");
        }
        events.extend(self.runtime.event_rx.try_iter());
        events
    }

    fn complete_original(&self, conversation: &ConversationId) -> OperationId {
        let operation = accept(&self.runtime, conversation, "original", "original input", None);
        let events = self.settle(conversation, &[operation.clone()]);
        assert_eq!(states(&events, &operation), vec![
            TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
        ]);
        assert_request(&self.provider, &operation, conversation, "original", "original input", false);
        operation
    }
}

fn conversation(name: &str) -> ConversationId {
    ConversationId::new(format!("mahayana-ai:agent:acceptance-{name}")).unwrap()
}

fn command(conversation: &ConversationId, message: &str, text: &str, retry: Option<&str>) -> RuntimeCommand {
    RuntimeCommand::SendMessage {
        conversation_id: conversation.clone(), text: text.to_string(),
        client_message_id: Some(message.to_string()),
        retry_of_client_message_id: retry.map(str::to_string),
        inference_provider: Some(if retry.is_some() { "retry-inference" } else { "original-inference" }.to_string()),
        hidden: retry.is_some(),
    }
}

fn accept(runtime: &MahayanaRuntime, conversation: &ConversationId, message: &str, text: &str, retry: Option<&str>) -> OperationId {
    match runtime.execute(command(conversation, message, text, retry)).expect("accepted command") {
        RuntimeResponse::Accepted { operation_id } => operation_id,
        other => panic!("expected Accepted, got {other:?}"),
    }
}

fn assert_sql_refusal(result: Result<RuntimeResponse, RuntimeError>, cause: &str) {
    match result {
        Err(RuntimeError::RuntimeStore(RuntimeStoreError::Sqlite(error))) => {
            assert!(error.contains(cause), "original SQLite cause lost: {error}");
        }
        other => panic!("expected original SQLite refusal {cause}, got {other:?}"),
    }
}

fn states(events: &[RuntimeEvent], operation: &OperationId) -> Vec<TurnState> {
    events.iter().filter_map(|event| match event {
        RuntimeEvent::TurnStateChanged { operation_id, state, .. } if operation_id == operation => Some(*state),
        _ => None,
    }).collect()
}

fn sequence(events: &[RuntimeEvent], operation: &OperationId, expected: TurnState) -> u64 {
    events.iter().find_map(|event| match event {
        RuntimeEvent::TurnStateChanged { operation_id, state, sequence, .. }
            if operation_id == operation && *state == expected => Some(*sequence),
        _ => None,
    }).expect("state has an actor-owned sequence")
}

fn assert_request(provider: &AdmissionProvider, operation: &OperationId, conversation: &ConversationId, message: &str, text: &str, retry: bool) {
    let requests = provider.requests.lock().unwrap();
    let matches: Vec<_> = requests.iter().filter(|request| &request.operation_id == operation).collect();
    assert_eq!(matches.len(), 1, "one actual provider entry per accepted operation");
    let request = matches[0];
    assert_eq!(&request.conversation_id, conversation);
    assert_eq!(request.client_message_id.as_deref(), Some(message));
    assert_eq!(request.text, text);
    assert_eq!(request.inference_provider.as_deref(), Some(if retry { "retry-inference" } else { "original-inference" }));
    assert_eq!(request.hidden, retry);
}

#[test]
fn command_acceptance_statement_refusal_rolls_back_bundle_and_actor() {
    for table in ["turns", "turn_requests", "runs"] {
        let fixture = Fixture::new();
        let conversation = conversation(table);
        let before = fixture.rows();
        fixture.reject_insert(table, "refused", false, "admission statement refused");
        assert_sql_refusal(fixture.runtime.execute(command(&conversation, "refused", "refused input", None)),
            "admission statement refused");
        assert_eq!(fixture.rows(), before, "partial bundle escaped {table} refusal");
        fixture.assert_no_owners_or_events();
        assert!(fixture.provider.requests.lock().unwrap().is_empty());
        fixture.remove_refusal();
        let later = accept(&fixture.runtime, &conversation, "later", "later input", None);
        let events = fixture.settle(&conversation, &[later.clone()]);
        assert_eq!(states(&events, &later), vec![
            TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
        ], "refused candidate left a ghost queue registration");
        assert_eq!(sequence(&events, &later, TurnState::Accepted), 1);
        assert_request(&fixture.provider, &later, &conversation, "later", "later input", false);
        assert_eq!(fixture.provider.requests.lock().unwrap().len(), 1);
    }
}

#[test]
fn command_acceptance_retry_refusal_preserves_completed_predecessor() {
    for table in ["turn_requests", "runs"] {
        let fixture = Fixture::new();
        let conversation = conversation(table);
        let original = fixture.complete_original(&conversation);
        let turn = fixture.turn_for(&original);
        let before = fixture.rows();
        fixture.reject_insert(table, "original", false, "retry admission refused");
        assert_sql_refusal(fixture.runtime.execute(command(&conversation, "attempt", "refused retry input", Some("original"))),
            "retry admission refused");
        assert_eq!(fixture.rows(), before, "retry overwrote completed predecessor or request");
        fixture.assert_no_owners_or_events();
        assert_eq!(fixture.provider.requests.lock().unwrap().len(), 1);
        fixture.remove_refusal();
        let retry = accept(&fixture.runtime, &conversation, "later-attempt", "valid retry input", Some("original"));
        let events = fixture.settle(&conversation, &[retry.clone()]);
        assert_eq!(states(&events, &retry), vec![
            TurnState::Recovering, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
        ]);
        assert_eq!(fixture.turn_for(&retry), turn);
        let generation: i64 = fixture.observer.query_row("SELECT generation FROM runs WHERE run_id = ?1",
            params![retry.as_str()], |row| row.get(0)).unwrap();
        assert_eq!(generation, 2);
        assert_request(&fixture.provider, &retry, &conversation, "original", "valid retry input", true);
        assert_eq!(fixture.provider.requests.lock().unwrap().len(), 2);
    }
}

#[test]
fn command_acceptance_queued_refusal_preserves_active_and_later_queued_work() {
    for table in ["turns", "runs"] {
        let fixture = Fixture::new();
        let conversation = conversation(table);
        let release = fixture.provider.hold("active input");
        let active = accept(&fixture.runtime, &conversation, "active", "active input", None);
        assert_eq!(fixture.provider.entered_rx.recv_timeout(WAIT).unwrap(), active);
        let before = fixture.rows();
        fixture.reject_insert(table, "refused", true, "queued admission refused");
        assert_sql_refusal(fixture.runtime.execute(command(&conversation, "refused", "refused queued input", None)),
            "queued admission refused");
        assert_eq!(fixture.rows(), before);
        assert_eq!(fixture.runtime.operations.lock().unwrap().len(), 1);
        assert!(fixture.runtime.operations.lock().unwrap().contains_key(&active));
        let contexts = fixture.runtime.run_contexts.lock().unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts.get(&active).unwrap().run_id.as_str(), active.as_str());
        drop(contexts);
        assert_eq!(fixture.provider.requests.lock().unwrap().len(), 1);
        let mut events: Vec<_> = fixture.runtime.event_rx.try_iter().collect();
        assert!(events.iter().all(|event| matches!(event,
            RuntimeEvent::TurnStateChanged { operation_id, .. } if operation_id == &active
        )), "refused queued command published an event: {events:?}");
        fixture.remove_refusal();
        let later = accept(&fixture.runtime, &conversation, "later", "later queued input", None);
        let (turn_state, run_state): (String, String) = fixture.observer.query_row(
            "SELECT t.state, r.state FROM turns t JOIN runs r ON r.turn_id = t.turn_id WHERE r.run_id = ?1",
            params![later.as_str()], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!((turn_state.as_str(), run_state.as_str()), ("queued", "queued"));
        release.send(()).expect("release only the active provider");
        events.extend(fixture.settle(&conversation, &[active.clone(), later.clone()]));
        assert_eq!(states(&events, &later), vec![
            TurnState::Accepted, TurnState::Queued, TurnState::Preparing,
            TurnState::Thinking, TurnState::Completed,
        ]);
        assert_eq!(sequence(&events, &later, TurnState::Queued),
            sequence(&events, &later, TurnState::Accepted).saturating_add(1));
        assert_request(&fixture.provider, &active, &conversation, "active", "active input", false);
        assert_request(&fixture.provider, &later, &conversation, "later", "later queued input", false);
        assert_eq!(fixture.provider.requests.lock().unwrap().len(), 2);
    }
}

#[test]
fn command_acceptance_same_predecessor_refusal_keeps_winning_retry_registration() {
    let fixture = Fixture::new();
    let conversation = conversation("same-predecessor");
    let original = fixture.complete_original(&conversation);
    let expected = fixture.runtime.store.retry_turn_predecessor(conversation.as_str(), "original")
        .unwrap().unwrap();
    assert_eq!(expected.next_generation, 2);
    let actor = fixture.actor(&conversation);
    let gate = fixture.runtime.async_runtime.block_on(actor.gate.lock());
    // No state can advance while this gate is held: the real Runtime winner
    // selects this same predecessor, commits generation 2, and remains queued
    // at its async dispatch gate. The loser deliberately retains the old token.
    let winner = accept(&fixture.runtime, &conversation, "winner", "winning retry", Some("original"));
    let winner_rows = fixture.rows();
    let loser = RunId::generated("run");
    assert_ne!(loser.as_str(), winner.as_str());
    let refused = actor.admit_turn(expected.turn_id.clone(), true, |state| {
        fixture.runtime.store.record_admission(
            &LogicalTurn {
                id: expected.turn_id.clone(), conversation_id: conversation.clone(),
                user_message_id: Some(MessageId::new("original").unwrap()),
                created_at_ms: expected.created_at_ms, state, active_run_id: Some(loser.clone()),
            },
            &ExecutionRun {
                id: loser.clone(), turn_id: expected.turn_id.clone(),
                generation: expected.next_generation, provider: "mahayana-ai".to_string(),
                started_at_ms: now_millis(), finished_at_ms: None, state,
            },
            "losing retry", Some("retry-inference"), true, Some(&expected),
        )
    });
    assert!(matches!(&refused,
        Err(ActorAdmissionError::Persistence(RuntimeStoreError::Sqlite(cause)))
            if cause.contains("retry predecessor changed")
    ), "same predecessor was admitted twice: {refused:?}");
    assert_eq!(fixture.rows(), winner_rows);
    assert_eq!(fixture.runtime.operations.lock().unwrap().len(), 1);
    assert!(fixture.runtime.operations.lock().unwrap().contains_key(&winner));
    assert_eq!(fixture.runtime.run_contexts.lock().unwrap().len(), 1);
    assert_eq!(fixture.runtime.run_contexts.lock().unwrap().get(&winner).unwrap().turn_id, expected.turn_id);
    assert_eq!(fixture.provider.requests.lock().unwrap().len(), 1);
    drop(gate);
    let events = fixture.settle(&conversation, &[winner.clone()]);
    assert_eq!(states(&events, &winner), vec![
        TurnState::Recovering, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_eq!(sequence(&events, &winner, TurnState::Preparing),
        sequence(&events, &winner, TurnState::Recovering).saturating_add(2),
        "refused candidate consumed actor sequence or altered winner state");
    assert_eq!(fixture.turn_for(&winner), fixture.turn_for(&original));
    assert_request(&fixture.provider, &winner, &conversation, "original", "winning retry", true);
    let generations: Vec<i64> = fixture.observer.prepare("SELECT generation FROM runs ORDER BY generation")
        .unwrap().query_map([], |row| row.get(0)).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(generations, vec![1, 2]);
    let later = accept(&fixture.runtime, &conversation, "later", "after stale retry", None);
    let events = fixture.settle(&conversation, &[later.clone()]);
    assert_eq!(states(&events, &later), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ], "same-TurnId refusal damaged or left another registration");
}

#[test]
fn command_acceptance_reopen_does_not_dispatch_refused_retry() {
    let fixture = Fixture::new();
    let conversation = conversation("reopen");
    let original = fixture.complete_original(&conversation);
    let turn = fixture.turn_for(&original);
    let before = fixture.rows();
    fixture.reject_insert("runs", "original", false, "refused before restart");
    assert_sql_refusal(fixture.runtime.execute(command(&conversation, "refused", "must not recover this input", Some("original"))),
        "refused before restart");
    fixture.remove_refusal();
    let fixture = fixture.reopen();
    assert_eq!(fixture.rows(), before, "restart changed the completed predecessor");
    assert!(fixture.runtime.store.recoverable_turns().unwrap().is_empty());
    fixture.assert_no_owners_or_events();
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    let retry = accept(&fixture.runtime, &conversation, "explicit", "explicit retry after restart", Some("original"));
    let events = fixture.settle(&conversation, &[retry.clone()]);
    assert_eq!(states(&events, &retry), vec![
        TurnState::Recovering, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_eq!(fixture.turn_for(&retry), turn);
    assert_request(&fixture.provider, &retry, &conversation, "original", "explicit retry after restart", true);
    assert_eq!(fixture.provider.requests.lock().unwrap().len(), 1);
}

#[test]
fn command_acceptance_deferred_foreign_key_commit_refusal_rolls_back_every_owner() {
    let fixture = Fixture::new();
    let conversation = conversation("commit-refusal");
    let enabled: i64 = fixture.runtime.store.mini_app_port().unwrap().with_connection(|connection| {
        connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))
    }).unwrap();
    assert_eq!(enabled, 1, "actual Runtime connection must enforce the deferred constraint");
    fixture.observer.execute_batch(
        "CREATE TABLE acceptance_commit_parent(id INTEGER PRIMARY KEY);
         CREATE TABLE acceptance_commit_child(
             run_id TEXT PRIMARY KEY, parent_id INTEGER NOT NULL,
             FOREIGN KEY(parent_id) REFERENCES acceptance_commit_parent(id) DEFERRABLE INITIALLY DEFERRED
         );
         CREATE TRIGGER refuse_admission AFTER INSERT ON runs
         BEGIN INSERT INTO acceptance_commit_child(run_id, parent_id) VALUES (NEW.run_id, 42); END;"
    ).unwrap();
    let before = fixture.rows();
    // The trigger itself succeeds; only COMMIT checks this deferred FK. The
    // production transaction must roll back both its bundle and this fixture row.
    assert_sql_refusal(fixture.runtime.execute(command(&conversation, "refused", "commit must fail", None)),
        "FOREIGN KEY constraint failed");
    assert_eq!(fixture.rows(), before);
    let children: i64 = fixture.observer.query_row("SELECT COUNT(*) FROM acceptance_commit_child", [], |row| row.get(0)).unwrap();
    assert_eq!(children, 0);
    fixture.assert_no_owners_or_events();
    assert!(fixture.provider.requests.lock().unwrap().is_empty());
    fixture.remove_refusal();
    let later = accept(&fixture.runtime, &conversation, "later", "after refused commit", None);
    let events = fixture.settle(&conversation, &[later.clone()]);
    assert_eq!(states(&events, &later), vec![
        TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
    ]);
    assert_request(&fixture.provider, &later, &conversation, "later", "after refused commit", false);
    assert_eq!(sequence(&events, &later, TurnState::Accepted), 1);
}

#[test]
fn command_acceptance_map_poison_is_rejected_before_durable_or_actor_publication() {
    fn poison<T>(mutex: &Mutex<T>) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = mutex.lock().unwrap();
            panic!("poison actual publication mutex");
        }));
        assert!(result.is_err());
        assert!(mutex.is_poisoned());
    }
    for contexts in [false, true] {
        let fixture = Fixture::new();
        let conversation = conversation(if contexts { "contexts-poison" } else { "operations-poison" });
        let before = fixture.rows();
        if contexts { poison(&fixture.runtime.run_contexts); } else { poison(&fixture.runtime.operations); }
        assert!(matches!(fixture.runtime.execute(command(&conversation, "refused", "poisoned publication", None)),
            Err(RuntimeError::Synchronization(_))));
        assert_eq!(fixture.rows(), before);
        assert!(fixture.provider.requests.lock().unwrap().is_empty());
        // Repair only the deliberate test poison marker, without deleting any
        // stored entry; then inspect owners and prove the actor was untouched.
        fixture.runtime.operations.clear_poison();
        fixture.runtime.run_contexts.clear_poison();
        fixture.assert_no_owners_or_events();
        let later = accept(&fixture.runtime, &conversation, "later", "after lock repair", None);
        let events = fixture.settle(&conversation, &[later.clone()]);
        assert_eq!(states(&events, &later), vec![
            TurnState::Accepted, TurnState::Preparing, TurnState::Thinking, TurnState::Completed,
        ]);
        assert_eq!(sequence(&events, &later, TurnState::Accepted), 1);
    }
}
