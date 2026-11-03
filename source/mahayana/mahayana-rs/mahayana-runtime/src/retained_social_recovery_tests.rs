//! NMR-SOCIAL-OWNER-03 / NMR-SOCIAL-AC-02: preserve retained social records while
//! continuing ordinary Agent recovery. These fixtures use the real SQLite
//! RuntimeStore; registered providers are in-process recorders, never services.

use super::*;
use async_trait::async_trait;
use mahayana_agent::{
    AgentEvent, AgentMessageRequest, ApprovalResolution, SharedAgentEventSink,
    StartThreadRequest,
};
use mahayana_core::{AgentThreadId, Message};
use rusqlite::{Connection, types::Value as SqlValue};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const SOCIAL: &str = "mahayana:contact:42";
const PROMPT: &str = "Retain this exact request: 中文\n  line two\u{0000}tail";

struct OwnedStore(PathBuf);

impl OwnedStore {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "mahayana-retained-conversation-{}-{}-{}",
            std::process::id(),
            now_millis(),
            NEXT.fetch_add(1, Ordering::SeqCst),
        ));
        std::fs::create_dir(&path).expect("create exclusively owned fixture directory");
        Self(path)
    }

    fn config(&self) -> RuntimeConfig {
        RuntimeConfig {
            data_dir: Some(self.0.clone()),
            ..RuntimeConfig::default()
        }
    }

    fn seed(&self) -> RuntimeStore {
        RuntimeStore::open(Some(&self.0)).expect("open actual SQLite RuntimeStore")
    }
}

impl Drop for OwnedStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn seed_turn(
    store: &RuntimeStore,
    suffix: &str,
    conversation: &str,
    state: TurnState,
    message_id: Option<&str>,
) -> (TurnId, RunId, MessageId) {
    let turn_id = TurnId(format!("turn:{suffix}"));
    let run_id = RunId(format!("run:{suffix}"));
    let message_id = MessageId(
        message_id.map(str::to_owned).unwrap_or_else(|| format!("message:{suffix}")),
    );
    store.record_turn(&LogicalTurn {
        id: turn_id.clone(),
        conversation_id: ConversationId(conversation.to_owned()),
        user_message_id: Some(message_id.clone()),
        created_at_ms: 10,
        state,
        active_run_id: Some(run_id.clone()),
    }).expect("persist interrupted turn");
    store.record_run(&ExecutionRun {
        id: run_id.clone(),
        turn_id: turn_id.clone(),
        generation: 1,
        provider: if conversation.starts_with("mahayana:contact:") {
            "mahayana-social"
        } else if conversation.starts_with("telegram:") {
            "telegram"
        } else {
            "mahayana-ai"
        }.to_owned(),
        started_at_ms: 11,
        finished_at_ms: None,
        state,
    }).expect("persist interrupted execution");
    store.record_turn_request(&turn_id, &message_id, PROMPT, None, false, 12)
        .expect("persist byte-sensitive recovery input");
    (turn_id, run_id, message_id)
}

fn connection(root: &Path) -> Connection {
    Connection::open(root.join("runtime.sqlite3")).expect("read fixture SQLite rows")
}

fn rows(connection: &Connection, sql: &str, arguments: &[&dyn rusqlite::ToSql]) -> Vec<Vec<SqlValue>> {
    let mut statement = connection.prepare(sql).expect("prepare snapshot query");
    let columns = statement.column_count();
    statement.query_map(arguments, |row| {
        (0..columns).map(|column| row.get::<_, SqlValue>(column)).collect()
    }).expect("query snapshot").map(|row| row.expect("read snapshot row")).collect()
}

fn snapshot_all(root: &Path) -> BTreeMap<String, Vec<Vec<SqlValue>>> {
    let connection = connection(root);
    let tables = rows(
        &connection,
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        &[],
    );
    tables.into_iter().map(|row| {
        let SqlValue::Text(table) = &row[0] else { panic!("table name must be text"); };
        let sql = format!("SELECT * FROM \"{}\" ORDER BY rowid", table.replace('"', "\"\""));
        (table.clone(), rows(&connection, &sql, &[]))
    }).collect()
}

fn snapshot_social(root: &Path) -> Vec<Vec<Vec<SqlValue>>> {
    let connection = connection(root);
    [
        "SELECT * FROM conversations WHERE conversation_id = ?1 ORDER BY rowid",
        "SELECT * FROM turns WHERE conversation_id = ?1 ORDER BY rowid",
        "SELECT r.* FROM runs r JOIN turns t ON t.turn_id = r.turn_id WHERE t.conversation_id = ?1 ORDER BY r.rowid",
        "SELECT q.* FROM turn_requests q JOIN turns t ON t.turn_id = q.turn_id WHERE t.conversation_id = ?1 ORDER BY q.rowid",
    ].into_iter().map(|sql| rows(&connection, sql, &[&SOCIAL])).collect()
}

#[derive(Default)]
struct RecordingSocialProvider {
    calls: AtomicUsize,
    sends: Mutex<Vec<SendMessageRequest>>,
}

#[async_trait]
impl ConversationProvider for RecordingSocialProvider {
    fn key(&self) -> &'static str { "mahayana-social" }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, ConversationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }

    async fn history(&self, _: &ConversationId, _: u32) -> Result<Vec<Message>, ConversationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
        _: SharedConversationEventSink,
    ) -> Result<(), ConversationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.sends.lock().expect("record provider send").push(request);
        Ok(())
    }

    async fn interrupt(&self, _: &OperationId) -> Result<(), ConversationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn resolve_approval(&self, _: ResolveApprovalRequest) -> Result<(), ConversationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Default)]
struct RecordingAgent {
    sends: Mutex<Vec<(ConversationId, String)>>,
}

#[async_trait]
impl AgentBackend for RecordingAgent {
    async fn start_thread(&self, _: StartThreadRequest) -> Result<AgentThreadId, AgentError> {
        Ok(AgentThreadId("thread:retention-control".to_owned()))
    }

    async fn send_message(
        &self,
        request: AgentMessageRequest,
        events: SharedAgentEventSink,
    ) -> Result<(), AgentError> {
        self.sends.lock().expect("record Agent send")
            .push((request.conversation_id.clone(), request.text.clone()));
        events.emit(AgentEvent::MessageCompleted {
            message: Message {
                id: MessageId::generated("message"),
                conversation_id: request.conversation_id,
                role: MessageRole::Assistant,
                text: "Recovered Agent work completed".to_owned(),
                created_at_ms: 20,
                metadata: Value::Null,
            },
        })
    }

    async fn interrupt(&self, _: &OperationId) -> Result<(), AgentError> { Ok(()) }

    async fn resolve_approval(&self, _: ApprovalResolution) -> Result<(), AgentError> { Ok(()) }

    fn name(&self) -> &'static str { "retention-control" }
}

fn assert_ready_only(runtime: &MahayanaRuntime) {
    assert!(matches!(runtime.receive(Duration::from_secs(1)).expect("receive ready"),
        Some(RuntimeEvent::Ready { .. })));
    assert!(runtime.receive(Duration::from_millis(20)).expect("inspect recovery events").is_none(),
        "retained work must emit no state transition, degradation, or completion");
    assert!(runtime.operations.lock().expect("inspect operations").is_empty());
}

fn await_completed(runtime: &MahayanaRuntime) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match runtime.receive(Duration::from_millis(100)).expect("receive recovery event") {
            Some(RuntimeEvent::OperationCompleted { .. }) => return,
            Some(RuntimeEvent::OperationFailed { message, .. }) => panic!("recovery failed: {message}"),
            Some(RuntimeEvent::ProviderDegraded { message, .. }) => panic!("recovery degraded: {message}"),
            _ => {}
        }
    }
    panic!("recovered work did not reach durable completion before deadline");
}

#[test]
fn retained_social_pending_turns_survive_repeated_sqlite_restart_without_dispatch() {
    let root = OwnedStore::new();
    let store = root.seed();
    for (suffix, state) in [
        ("accepted", TurnState::Accepted),
        ("queued", TurnState::Queued),
        ("preparing", TurnState::Preparing),
        ("thinking", TurnState::Thinking),
        ("tool-running", TurnState::ToolRunning),
        ("streaming", TurnState::Streaming),
        ("recovering", TurnState::Recovering),
    ] {
        seed_turn(&store, suffix, SOCIAL, state, None);
    }
    drop(store);
    // A historical row without durable input must also be retained without
    // consulting provider history or marking its turn failed.
    connection(&root.0).execute("DELETE FROM turn_requests WHERE turn_id = 'turn:queued'", [])
        .expect("seed pre-turn-request historical row");
    let before = snapshot_all(&root.0);
    let provider = Arc::new(RecordingSocialProvider::default());
    for _ in 0..2 {
        let runtime = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
            .with_provider(provider.clone()).expect("register no-network dispatch recorder")
            .build().expect("retired social data must not prevent startup");
        assert_ready_only(&runtime);
        drop(runtime);
        assert_eq!(snapshot_all(&root.0), before, "all durable column values must survive restart");
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    assert!(provider.sends.lock().expect("read recorded calls").is_empty());
}

#[test]
fn retained_social_waiting_user_preserves_pending_question_and_payload_bytes() {
    let root = OwnedStore::new();
    let store = root.seed();
    let (turn_id, run_id, _) = seed_turn(&store, "waiting", SOCIAL, TurnState::WaitingUser, None);
    store.enqueue_ask_user(&AskUserRequest {
        id: IntentId("ask-user:retained".to_owned()),
        turn_id,
        run_id,
        question: "Keep pending question exactly: 中文\n  answer?".to_owned(),
    }, 13).expect("persist pending user question");
    drop(store);
    let before = snapshot_all(&root.0);
    for _ in 0..2 {
        let runtime = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
            .build().expect("waiting social turn remains retained with no provider");
        assert_ready_only(&runtime);
        drop(runtime);
        assert_eq!(snapshot_all(&root.0), before,
            "WaitingUser must not become failed; raw text/JSON payloads must remain equal");
    }
}

#[test]
fn retained_social_handoffs_preserve_queued_running_and_terminal_target_records() {
    let root = OwnedStore::new();
    let store = root.seed();
    for (suffix, state) in [
        ("unstarted", None),
        ("running", Some(TurnState::Thinking)),
        ("waiting", Some(TurnState::WaitingUser)),
        ("completed", Some(TurnState::Completed)),
        ("failed", Some(TurnState::Failed)),
        ("cancelled", Some(TurnState::Cancelled)),
    ] {
        let intent = HandoffIntent {
            id: IntentId(format!("handoff:{suffix}")),
            target_agent: "historical-social-target".to_owned(),
            target_conversation_id: Some(ConversationId(SOCIAL.to_owned())),
            inference_provider: None,
            task: format!("{PROMPT} {suffix}"),
            constraints: serde_json::json!({"retain": ["中文", "\n", 42]}),
            expected_output: Some("existing payload".to_owned()),
            origin_run: RunId("run:historical-origin".to_owned()),
            depth: 1,
        };
        store.enqueue_handoff(&intent, &TurnId("turn:historical-origin".to_owned()), 14)
            .expect("persist historical handoff");
        if let Some(state) = state {
            seed_turn(&store, suffix, SOCIAL, state, Some(intent.id.as_str()));
            store.mark_handoff_started(intent.id.as_str(), &format!("run:{suffix}"), 15)
                .expect("persist running handoff dispatch");
        }
    }
    drop(store);
    // Preserve representation as well as parsed meaning. Recovery may read
    // these valid JSON payloads but must not normalize or rewrite them.
    connection(&root.0).execute(
        "UPDATE pending_intents SET payload_json = '  ' || payload_json || char(10)", [],
    ).expect("retain historical JSON whitespace");
    let before = snapshot_all(&root.0);
    let provider = Arc::new(RecordingSocialProvider::default());
    for _ in 0..2 {
        let runtime = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
            .with_provider(provider.clone()).expect("register dispatch recorder")
            .build().expect("retain social handoffs");
        assert_ready_only(&runtime);
        drop(runtime);
        assert_eq!(snapshot_all(&root.0), before,
            "handoff, target turn, audit and exact JSON payload rows must be unchanged");
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn retained_social_policy_keeps_agent_recovery_on_shared_async_builder() {
    let root = OwnedStore::new();
    let store = root.seed();
    seed_turn(&store, "social", SOCIAL, TurnState::Thinking, None);
    let (turn_id, _, message_id) = seed_turn(
        &store, "agent", "codex:agent:recovery-control", TurnState::Thinking, None,
    );
    drop(store);
    let social_before = snapshot_social(&root.0);
    let agent = Arc::new(RecordingAgent::default());
    let backend: Arc<dyn AgentBackend> = agent.clone();
    let runtime = RuntimeBuilder::new(root.config())
        .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
        .build_with_agent_backend(|| async { Ok(backend) })
        .expect("shared Agent runtime must start despite retained social work");
    await_completed(&runtime);
    let recovered = runtime.store.find_turn_by_client_message(
        &ConversationId("codex:agent:recovery-control".to_owned()), &message_id,
    ).expect("read recovered logical turn").expect("same logical turn exists");
    assert_eq!(recovered.turn_id, turn_id);
    assert_eq!(recovered.generation, 2);
    assert_eq!(recovered.state, TurnState::Completed);
    assert_eq!(*agent.sends.lock().expect("inspect Agent recovery"), vec![(
        ConversationId("codex:agent:recovery-control".to_owned()), PROMPT.to_owned(),
    )]);
    drop(runtime);
    assert_eq!(snapshot_social(&root.0), social_before);
}

#[test]
fn retained_social_policy_keeps_missing_handoff_target_on_agent_fallback() {
    let root = OwnedStore::new();
    let store = root.seed();
    let intent = HandoffIntent {
        id: IntentId("handoff:agent-fallback".to_owned()),
        target_agent: "fallback-control".to_owned(),
        target_conversation_id: None,
        inference_provider: None,
        task: "Resume the ordinary Agent handoff".to_owned(),
        constraints: Value::Null,
        expected_output: None,
        origin_run: RunId("run:historical-origin".to_owned()),
        depth: 1,
    };
    store.enqueue_handoff(&intent, &TurnId("turn:historical-origin".to_owned()), 14)
        .expect("persist Agent handoff without explicit target conversation");
    drop(store);
    let agent = Arc::new(RecordingAgent::default());
    let runtime = RuntimeBuilder::new(root.config())
        .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
        .with_agent_backend(agent.clone()).expect("register normal Agent")
        .build().expect("recover default Agent handoff target");
    await_completed(&runtime);
    let sends = agent.sends.lock().expect("inspect handoff send");
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0.as_str(), "codex:agent:fallback-control");
    assert!(sends[0].1.contains("Resume the ordinary Agent handoff"));
    assert_eq!(rows(&connection(&root.0),
        "SELECT state FROM handoff_dispatch WHERE intent_id = 'handoff:agent-fallback'", &[]),
        vec![vec![SqlValue::Text("completed".to_owned())]]);
}

#[test]
fn default_policy_recovers_social_work_with_registered_provider() {
    let root = OwnedStore::new();
    let store = root.seed();
    let (turn_id, _, message_id) = seed_turn(&store, "default", SOCIAL, TurnState::Thinking, None);
    drop(store);
    let provider = Arc::new(RecordingSocialProvider::default());
    let runtime = RuntimeBuilder::new(root.config())
        .with_provider(provider.clone()).expect("register non-desktop social provider")
        .build().expect("ordinary social recovery remains supported");
    await_completed(&runtime);
    let sends = provider.sends.lock().expect("read ordinary social sends");
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].conversation_id.as_str(), SOCIAL);
    assert_eq!(sends[0].text, PROMPT);
    assert_eq!(sends[0].client_message_id.as_deref(), Some(message_id.as_str()));
    let recovered = runtime.store.find_turn_by_client_message(&ConversationId(SOCIAL.to_owned()), &message_id)
        .expect("read default recovered turn").expect("default turn exists");
    assert_eq!(recovered.turn_id, turn_id);
    assert_eq!(recovered.generation, 2);
    assert_eq!(recovered.state, TurnState::Completed);
}

#[test]
fn default_policy_does_not_hide_missing_social_provider() {
    let root = OwnedStore::new();
    let store = root.seed();
    seed_turn(&store, "default-missing", SOCIAL, TurnState::Thinking, None);
    drop(store);
    let before = snapshot_all(&root.0);
    let result = RuntimeBuilder::new(root.config()).build();
    assert!(matches!(result, Err(RuntimeError::Conversation(
        ConversationError::ProviderUnavailable(provider)
    )) if provider == "mahayana-social"));
    assert_eq!(snapshot_all(&root.0), before);
}

#[test]
fn retained_social_policy_does_not_hide_unrelated_or_nearby_namespace_errors() {
    for (conversation, missing_provider) in [
        ("miniapp:unavailable", Some("miniapp")),
        ("mahayana:contact", Some("mahayana-social")),
        ("mahayana:contacts:42", Some("mahayana-social")),
        ("unknown:42", None),
    ] {
        let root = OwnedStore::new();
        let store = root.seed();
        seed_turn(&store, "unrelated", conversation, TurnState::Thinking, None);
        drop(store);
        let before = snapshot_all(&root.0);
        let result = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveSocialContacts)
            .build();
        match (result, missing_provider) {
            (Err(RuntimeError::Conversation(ConversationError::ProviderUnavailable(actual))), Some(expected)) => {
                assert_eq!(actual, expected);
            }
            (Err(RuntimeError::Conversation(ConversationError::UnsupportedConversation(actual))), None) => {
                assert_eq!(actual, conversation);
            }
            _ => panic!("retention must preserve the original error for {conversation}"),
        }
        assert_eq!(snapshot_all(&root.0), before);
    }
}

#[test]
fn default_policy_keeps_explicit_retry_requirement_for_waiting_user() {
    let root = OwnedStore::new();
    let store = root.seed();
    let (_, run_id, message_id) = seed_turn(&store, "default-waiting", SOCIAL, TurnState::WaitingUser, None);
    drop(store);
    let runtime = RuntimeBuilder::new(root.config()).build()
        .expect("default waiting-user recovery keeps its prior behavior");
    let turn = runtime.store.find_turn_by_client_message(&ConversationId(SOCIAL.to_owned()), &message_id)
        .expect("read waiting-user turn").expect("turn preserved");
    assert_eq!(turn.state, TurnState::Failed);
    assert_eq!(rows(&connection(&root.0), "SELECT state FROM runs WHERE run_id = ?1", &[&run_id.as_str()]),
        vec![vec![SqlValue::Text("failed".to_owned())]]);
}


const TELEGRAM: &str = "telegram:chat:-1000042";

#[derive(Default)]
struct RecordingTelegramProvider(RecordingSocialProvider);

#[async_trait]
impl ConversationProvider for RecordingTelegramProvider {
    fn key(&self) -> &'static str { "telegram" }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, ConversationError> {
        self.0.list_conversations().await
    }

    async fn history(&self, conversation: &ConversationId, limit: u32) -> Result<Vec<Message>, ConversationError> {
        self.0.history(conversation, limit).await
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
        events: SharedConversationEventSink,
    ) -> Result<(), ConversationError> {
        self.0.send_message(request, events).await
    }

    async fn interrupt(&self, operation: &OperationId) -> Result<(), ConversationError> {
        self.0.interrupt(operation).await
    }

    async fn resolve_approval(&self, request: ResolveApprovalRequest) -> Result<(), ConversationError> {
        self.0.resolve_approval(request).await
    }
}

fn snapshot_telegram(root: &Path) -> Vec<Vec<Vec<SqlValue>>> {
    let connection = connection(root);
    [
        "SELECT * FROM conversations WHERE conversation_id = ?1 ORDER BY rowid",
        "SELECT * FROM turns WHERE conversation_id = ?1 ORDER BY rowid",
        "SELECT r.* FROM runs r JOIN turns t ON t.turn_id = r.turn_id WHERE t.conversation_id = ?1 ORDER BY r.rowid",
        "SELECT q.* FROM turn_requests q JOIN turns t ON t.turn_id = q.turn_id WHERE t.conversation_id = ?1 ORDER BY q.rowid",
    ].into_iter().map(|sql| rows(&connection, sql, &[&TELEGRAM])).collect()
}

#[test]
fn retained_telegram_turns_and_handoffs_survive_desktop_sqlite_restart_without_dispatch() {
    let root = OwnedStore::new();
    let store = root.seed();
    for (suffix, state) in [
        ("accepted", TurnState::Accepted),
        ("queued", TurnState::Queued),
        ("preparing", TurnState::Preparing),
        ("thinking", TurnState::Thinking),
        ("tool-running", TurnState::ToolRunning),
        ("streaming", TurnState::Streaming),
        ("waiting-user", TurnState::WaitingUser),
        ("recovering", TurnState::Recovering),
    ] {
        let (turn_id, run_id, _) = seed_turn(
            &store, &format!("telegram-{suffix}"), TELEGRAM, state, None,
        );
        if state == TurnState::WaitingUser {
            store.enqueue_ask_user(&AskUserRequest {
                id: IntentId("ask-user:retained-telegram".to_owned()),
                turn_id,
                run_id,
                question: "Retain Telegram question exactly: 中文\n  answer?\u{0000}".to_owned(),
            }, 13).expect("persist historical Telegram question");
        }
    }
    // Unknown data under the retired namespace is preserved, not parsed as a
    // valid chat or normalized. Stored IDs are never trimmed.
    for (suffix, conversation) in [
        ("unknown", "telegram:chat:not-a-numeric-chat"),
        ("suffix-whitespace", "telegram:chat:42 \n"),
    ] {
        seed_turn(&store, suffix, conversation, TurnState::Thinking, None);
    }
    seed_turn(&store, "desktop-social", SOCIAL, TurnState::WaitingUser, None);
    for (suffix, state) in [
        ("unstarted", None),
        ("queued", Some(TurnState::Queued)),
        ("running", Some(TurnState::Thinking)),
        ("waiting", Some(TurnState::WaitingUser)),
        ("completed", Some(TurnState::Completed)),
        ("failed", Some(TurnState::Failed)),
        ("cancelled", Some(TurnState::Cancelled)),
    ] {
        let intent = HandoffIntent {
            id: IntentId(format!("handoff:telegram-{suffix}")),
            target_agent: "historical-telegram-target".to_owned(),
            target_conversation_id: Some(ConversationId(TELEGRAM.to_owned())),
            inference_provider: None,
            task: format!("{PROMPT} {suffix}"),
            constraints: serde_json::json!({"retain": ["中文", "\n", 42]}),
            expected_output: Some("original Telegram handoff payload".to_owned()),
            origin_run: RunId("run:historical-telegram-origin".to_owned()),
            depth: 1,
        };
        store.enqueue_handoff(&intent, &TurnId("turn:historical-telegram-origin".to_owned()), 14)
            .expect("persist Telegram handoff");
        if let Some(state) = state {
            seed_turn(&store, &format!("telegram-handoff-{suffix}"), TELEGRAM, state, Some(intent.id.as_str()));
            store.mark_handoff_started(intent.id.as_str(), &format!("run:telegram-handoff-{suffix}"), 15)
                .expect("persist target handoff dispatch");
        }
    }
    drop(store);
    let connection = connection(&root.0);
    connection.execute("DELETE FROM turn_requests WHERE turn_id = 'turn:telegram-queued'", [])
        .expect("preserve legacy Telegram turn without durable input");
    connection.execute("UPDATE handoff_dispatch SET state = 'queued' WHERE intent_id = 'handoff:telegram-queued'", [])
        .expect("seed queued historical handoff dispatch");
    let payloads = {
        let mut statement = connection.prepare("SELECT intent_id, payload_json FROM pending_intents ORDER BY intent_id")
            .expect("read original fixture payloads");
        let payloads = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .expect("query payload representations")
            .collect::<Result<Vec<_>, _>>().expect("collect payload representations");
        payloads
    };
    for (id, payload) in payloads {
        let mut value: Value = serde_json::from_str(&payload).expect("valid original payload");
        value.as_object_mut().expect("payload object").insert(
            "retainedUnknownField".to_owned(), serde_json::json!([true, "人", "\u{0000}", 42]),
        );
        let original = format!("  {}\n", serde_json::to_string(&value).expect("serialize original fixture"));
        connection.execute("UPDATE pending_intents SET payload_json = ?1 WHERE intent_id = ?2", [&original, &id])
            .expect("persist byte-sensitive historical representation");
    }
    drop(connection);
    let before = snapshot_all(&root.0);
    let telegram = Arc::new(RecordingTelegramProvider::default());
    let social = Arc::new(RecordingSocialProvider::default());
    for _ in 0..2 {
        let runtime = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveDesktopLegacyConversations)
            .with_provider(telegram.clone()).expect("register Telegram call recorder")
            .with_provider(social.clone()).expect("register Social call recorder")
            .build().expect("retained Telegram work must not prevent desktop startup");
        assert_ready_only(&runtime);
        drop(runtime);
        assert_eq!(snapshot_all(&root.0), before,
            "every retained SQLite field and raw question/handoff payload must survive restart");
    }
    assert_eq!(telegram.0.calls.load(Ordering::SeqCst), 0);
    assert!(telegram.0.sends.lock().expect("inspect Telegram sends").is_empty());
    assert_eq!(social.calls.load(Ordering::SeqCst), 0);
    assert!(social.sends.lock().expect("inspect Social sends").is_empty());

    // The same new policy is not a blanket recovery disable. Exercise the
    // shared async constructor with both retained namespaces and a live Agent.
    let mixed = OwnedStore::new();
    let store = mixed.seed();
    seed_turn(&store, "mixed-telegram", TELEGRAM, TurnState::Thinking, None);
    seed_turn(&store, "mixed-social", SOCIAL, TurnState::Thinking, None);
    let (turn_id, _, message_id) = seed_turn(
        &store, "mixed-agent", "codex:agent:desktop-retention-control", TurnState::Thinking, None,
    );
    drop(store);
    let telegram_before = snapshot_telegram(&mixed.0);
    let social_before = snapshot_social(&mixed.0);
    let agent = Arc::new(RecordingAgent::default());
    let backend: Arc<dyn AgentBackend> = agent.clone();
    let runtime = RuntimeBuilder::new(mixed.config())
        .with_retained_conversation_policy(RetainedConversationPolicy::PreserveDesktopLegacyConversations)
        .with_provider(telegram.clone()).expect("register mixed Telegram recorder")
        .with_provider(social.clone()).expect("register mixed Social recorder")
        .build_with_agent_backend(|| async { Ok(backend) })
        .expect("desktop policy keeps ordinary Agent recovery");
    await_completed(&runtime);
    let recovered = runtime.store.find_turn_by_client_message(
        &ConversationId("codex:agent:desktop-retention-control".to_owned()), &message_id,
    ).expect("read recovered Agent").expect("original Agent logical turn");
    assert_eq!(recovered.turn_id, turn_id);
    assert_eq!(recovered.generation, 2);
    assert_eq!(recovered.state, TurnState::Completed);
    assert_eq!(*agent.sends.lock().expect("read ordinary Agent send"), vec![(
        ConversationId("codex:agent:desktop-retention-control".to_owned()), PROMPT.to_owned(),
    )]);
    drop(runtime);
    assert_eq!(snapshot_telegram(&mixed.0), telegram_before);
    assert_eq!(snapshot_social(&mixed.0), social_before);
    assert_eq!(telegram.0.calls.load(Ordering::SeqCst), 0);
    assert_eq!(social.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn legacy_telegram_recovery_keeps_default_and_social_only_policies() {
    for policy in [
        RetainedConversationPolicy::None,
        RetainedConversationPolicy::PreserveSocialContacts,
    ] {
        let root = OwnedStore::new();
        let store = root.seed();
        let (turn_id, _, message_id) = seed_turn(&store, "ordinary-telegram", TELEGRAM, TurnState::Thinking, None);
        drop(store);
        let provider = Arc::new(RecordingTelegramProvider::default());
        let runtime = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(policy)
            .with_provider(provider.clone()).expect("register ordinary Telegram provider")
            .build().expect("old policy still recovers registered Telegram work");
        await_completed(&runtime);
        {
            let sends = provider.0.sends.lock().expect("read ordinary Telegram send");
            assert_eq!(sends.len(), 1);
            assert_eq!(sends[0].conversation_id.as_str(), TELEGRAM);
            assert_eq!(sends[0].text, PROMPT);
            assert_eq!(sends[0].client_message_id.as_deref(), Some(message_id.as_str()));
        }
        assert_eq!(provider.0.calls.load(Ordering::SeqCst), 1,
            "durable input recovery should not fetch provider history");
        let recovered = runtime.store.find_turn_by_client_message(&ConversationId(TELEGRAM.to_owned()), &message_id)
            .expect("read original Telegram turn").expect("original logical turn remains");
        assert_eq!(recovered.turn_id, turn_id);
        assert_eq!(recovered.generation, 2);
        assert_eq!(recovered.state, TurnState::Completed);
        drop(runtime);

        let waiting = OwnedStore::new();
        let store = waiting.seed();
        let (_, run_id, message_id) = seed_turn(&store, "ordinary-waiting", TELEGRAM, TurnState::WaitingUser, None);
        drop(store);
        let provider = Arc::new(RecordingTelegramProvider::default());
        let runtime = RuntimeBuilder::new(waiting.config())
            .with_retained_conversation_policy(policy)
            .with_provider(provider.clone()).expect("register waiting-user control")
            .build().expect("old policy keeps explicit-retry waiting-user behavior");
        let turn = runtime.store.find_turn_by_client_message(&ConversationId(TELEGRAM.to_owned()), &message_id)
            .expect("read waiting-user Telegram turn").expect("waiting-user turn remains");
        assert_eq!(turn.state, TurnState::Failed);
        assert_eq!(turn.generation, 1);
        assert_eq!(rows(&connection(&waiting.0), "SELECT state FROM runs WHERE run_id = ?1", &[&run_id.as_str()]),
            vec![vec![SqlValue::Text("failed".to_owned())]]);
        assert!(matches!(runtime.receive(Duration::from_secs(1)).expect("read ready"), Some(RuntimeEvent::Ready { .. })));
        assert!(matches!(runtime.receive(Duration::from_secs(1)).expect("read explicit-retry notice"),
            Some(RuntimeEvent::ProviderDegraded { provider, message })
                if provider == "turn-recovery" && message.contains("explicit retry is required")));
        assert!(runtime.receive(Duration::from_millis(20)).expect("read final waiting event").is_none());
        assert!(runtime.operations.lock().expect("inspect waiting operations").is_empty());
        assert_eq!(provider.0.calls.load(Ordering::SeqCst), 0);
        drop(runtime);

        let missing = OwnedStore::new();
        let store = missing.seed();
        seed_turn(&store, "missing-telegram", TELEGRAM, TurnState::Thinking, None);
        drop(store);
        let before = snapshot_all(&missing.0);
        let result = RuntimeBuilder::new(missing.config())
            .with_retained_conversation_policy(policy).build();
        assert!(matches!(result, Err(RuntimeError::Conversation(
            ConversationError::ProviderUnavailable(provider)
        )) if provider == "telegram"), "old policy must not hide an absent Telegram provider");
        assert_eq!(snapshot_all(&missing.0), before);
    }

    for (conversation, missing_provider) in [
        ("telegram:chat:", Some("telegram")),
        ("telegram:chat", Some("telegram")),
        ("telegram:chats:42", Some("telegram")),
        ("telegram:other:42", Some("telegram")),
        (" \ttelegram:chat:42\n", None),
        ("mahayana:contact:", Some("mahayana-social")),
        ("mahayana:contacts:42", Some("mahayana-social")),
        ("miniapp:unavailable", Some("miniapp")),
        ("unknown:42", None),
    ] {
        let root = OwnedStore::new();
        let store = root.seed();
        seed_turn(&store, "desktop-unrelated", conversation, TurnState::Thinking, None);
        drop(store);
        let before = snapshot_all(&root.0);
        let result = RuntimeBuilder::new(root.config())
            .with_retained_conversation_policy(RetainedConversationPolicy::PreserveDesktopLegacyConversations)
            .build();
        match (result, missing_provider) {
            (Err(RuntimeError::Conversation(ConversationError::ProviderUnavailable(actual))), Some(expected)) => {
                assert_eq!(actual, expected);
            }
            (Err(RuntimeError::Conversation(ConversationError::UnsupportedConversation(actual))), None) => {
                assert_eq!(actual, conversation);
            }
            _ => panic!("desktop retention must preserve the existing error for {conversation}"),
        }
        assert_eq!(snapshot_all(&root.0), before);
    }

    let fallback = OwnedStore::new();
    let store = fallback.seed();
    seed_turn(&store, "fallback-retained", TELEGRAM, TurnState::WaitingUser, None);
    let intent = HandoffIntent {
        id: IntentId("handoff:desktop-agent-fallback".to_owned()),
        target_agent: "desktop-fallback-control".to_owned(),
        target_conversation_id: None,
        inference_provider: None,
        task: "Resume ordinary Agent handoff while Telegram remains retained".to_owned(),
        constraints: Value::Null,
        expected_output: None,
        origin_run: RunId("run:historical-origin".to_owned()),
        depth: 1,
    };
    store.enqueue_handoff(&intent, &TurnId("turn:historical-origin".to_owned()), 14)
        .expect("persist missing-target Agent handoff");
    drop(store);
    let telegram_before = snapshot_telegram(&fallback.0);
    let agent = Arc::new(RecordingAgent::default());
    let runtime = RuntimeBuilder::new(fallback.config())
        .with_retained_conversation_policy(RetainedConversationPolicy::PreserveDesktopLegacyConversations)
        .with_agent_backend(agent.clone()).expect("register fallback Agent")
        .build().expect("desktop policy preserves missing-target fallback");
    await_completed(&runtime);
    {
        let sends = agent.sends.lock().expect("inspect fallback send");
        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].0.as_str(), "codex:agent:desktop-fallback-control");
        assert!(sends[0].1.contains("Resume ordinary Agent handoff while Telegram remains retained"));
    }
    assert_eq!(rows(&connection(&fallback.0),
        "SELECT state FROM handoff_dispatch WHERE intent_id = 'handoff:desktop-agent-fallback'", &[]),
        vec![vec![SqlValue::Text("completed".to_owned())]]);
    drop(runtime);
    assert_eq!(snapshot_telegram(&fallback.0), telegram_before,
        "Agent fallback must not fail or rewrite retained waiting-user work");
}
