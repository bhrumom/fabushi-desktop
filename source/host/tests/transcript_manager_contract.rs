use std::fs;
use std::future::{ready, Future};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::automations::automation::AutomationSpec;
use mahayana_host_runtime::extensions::session::agent_db_serde::AwaitingUserResponse;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::AgentDeletionRuntimeDeps;
use mahayana_host_runtime::extensions::session::agent_db::ChannelInboundAppendOutcome;
use mahayana_host_runtime::extensions::transcript::background_wakes::{
    BackgroundWakes, InboundAppendAction,
};
use mahayana_host_runtime::extensions::session::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffDecision, HandoffRequest, HandoffStartResult,
    HandoffTelemetry, HandoffTrigger,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::client_side_tool_v2_producer::{
    ClientSideToolV2ProducedValue, ClientSideToolV2TransportKind,
};
use mahayana_host_runtime::extensions::transcript::pending_wake_rearm::{
    LostSubagentWake, PendingWakeReport, PendingWakeRuntimePort,
};
use mahayana_host_runtime::extensions::transcript::roster_emit::ProductionRosterEmit;
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    DurablePendingWakeMarker, PendingWakeKind, QuietWakeOrigin, SandPendingWakeStore,
};
use mahayana_host_runtime::extensions::transcript::transcript_manager::{
    TranscriptManager, TranscriptTurnExecutionPort,
};
use mahayana_host_runtime::extensions::turn_execution::extension::turn_execution_extension;
use mahayana_host_runtime::extensions::turn_execution::turn_execution_service::TurnExecutor;
use serde_json::{json, Value};

fn temp_root() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fabushi-transcript-manager-{}-{suffix}-{sequence}",
        std::process::id()
    ))
}

#[test]
fn manager_is_the_single_production_composition_owner() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));

    assert!(Arc::ptr_eq(&manager.session_workers(), &sessions));
    let runtime_a = manager.transcript_runtime();
    let runtime_b = manager.transcript_runtime();
    assert!(Arc::ptr_eq(&runtime_a, &runtime_b));

    let runners_a = manager.runner_registry();
    let runners_b = manager.runner_registry();
    assert!(Arc::ptr_eq(&runners_a, &runners_b));
    assert_eq!(runners_a.active_count(), 0);

    let group_a = manager.group_chat();
    let group_b = manager.group_chat();
    assert!(Arc::ptr_eq(&group_a, &group_b));

    let widgets_a = manager.widget_responses();
    let widgets_b = manager.widget_responses();
    assert!(Arc::ptr_eq(&widgets_a, &widgets_b));

    let roster = Arc::new(ProductionRosterEmit::new(
        Arc::clone(&sessions),
        manager.transcript_runtime(),
        Arc::new(|_| {}),
    ));
    manager
        .bind_roster_emit(Arc::clone(&roster))
        .expect("bind manager roster");
    assert!(Arc::ptr_eq(
        &manager.roster_emit().expect("manager roster"),
        &roster,
    ));

    let workflows_a = manager.workflow_commands();
    let workflows_b = manager.workflow_commands();
    assert!(Arc::ptr_eq(&workflows_a, &workflows_b));

    let wakes_a = manager.background_wakes();
    let wakes_b = manager.background_wakes();
    assert!(Arc::ptr_eq(&wakes_a, &wakes_b));
    {
        let mut wakes = wakes_a.lock().expect("background wakes");
        mahayana_host_runtime::extensions::transcript::background_wakes::BackgroundWakes::enqueue(
            &mut wakes.pending_inbound,
            "agent-a",
            json!({"kind":"inbound"}),
        );
        wakes.dm_preempted_wake_agent_ids.insert("agent-a".into());
    }

    let rooms_a = manager.shared_rooms();
    let rooms_b = manager.shared_rooms();
    assert!(Arc::ptr_eq(&rooms_a, &rooms_b));

    let tool_call = manager
        .publish_client_side_tool_v2(
            "agent-tools",
            ClientSideToolV2ProducedValue::call("call-1", b"call".to_vec()),
        )
        .expect("manager-owned client tool call");
    let tool_result = manager
        .publish_client_side_tool_v2(
            "agent-tools",
            ClientSideToolV2ProducedValue::result("call-1", b"result".to_vec()),
        )
        .expect("manager-owned client tool result");
    assert_eq!(tool_call.kind, ClientSideToolV2TransportKind::Call);
    assert_eq!(tool_result.kind, ClientSideToolV2TransportKind::Result);
    assert_eq!(tool_call.epoch, tool_result.epoch);
    assert_eq!(tool_call.sequence, 1);
    assert_eq!(tool_result.sequence, 2);
    let tool_reset = manager
        .reset_client_side_tool_v2("agent-tools")
        .expect("manager-owned client tool reset");
    assert_eq!(tool_reset.kind, ClientSideToolV2TransportKind::Reset);
    assert_eq!(tool_reset.epoch, tool_call.epoch);
    assert_eq!(tool_reset.sequence, 3);

    let automation_config_changed = Arc::new(Mutex::new(0usize));
    let automation_config_changed_capture = Arc::clone(&automation_config_changed);
    manager.set_automation_config_changed_observer(Some(Arc::new(move || {
        *automation_config_changed_capture
            .lock()
            .expect("automation config observer") += 1;
    })));
    let listener_cards = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let listener_cards_capture = Arc::clone(&listener_cards);
    manager.set_listener_connect_observer(Some(Arc::new(move |agent_id, platform| {
        listener_cards_capture
            .lock()
            .expect("listener observer")
            .push((agent_id.to_string(), platform.to_string()));
    })));
    let channel_config_changed = Arc::new(Mutex::new(0usize));
    let channel_config_changed_capture = Arc::clone(&channel_config_changed);
    manager.set_channel_config_changed_observer(Some(Arc::new(move || {
        *channel_config_changed_capture
            .lock()
            .expect("channel config observer") += 1;
    })));
    assert!(
        manager
            .widget_responses()
            .bind_channel_config_changed(Arc::new(|| {}))
            .is_err(),
        "WidgetResponses channel-config signal must remain owned by TranscriptManager"
    );
    manager.emit_automation_config_changed();
    manager.emit_listener_connect_card("agent-observer", "slack");
    assert_eq!(*automation_config_changed.lock().expect("automation count"), 1);
    assert_eq!(
        listener_cards.lock().expect("listener cards").as_slice(),
        &[("agent-observer".to_string(), "slack".to_string())],
    );

    let ack_a = manager.ack_obligations();
    let ack_b = manager.ack_obligations();
    assert!(Arc::ptr_eq(&ack_a, &ack_b));
    ack_a.record_send("agent-a", 1.0).expect("durable ack");
    assert_eq!(ack_b.pending_obligations().len(), 1);

    let session = SandAgentSessionStore::new(Arc::clone(&sessions));
    let member_profile = |name: &str| SandAgentProfile {
        name: name.to_string(),
        description: String::new(),
        title: String::new(),
        avatar_shape: String::new(),
        avatar_color: String::new(),
    };
    let first_profile = member_profile("First member");
    let second_profile = member_profile("Second member");
    let first_group_member = session
        .create_session(Some(&first_profile), "user", None)
        .expect("first group member");
    let second_group_member = session
        .create_session(Some(&second_profile), "user", None)
        .expect("second group member");
    let created_group = manager
        .dispatch_session_gateway_call(
            "createGroup",
            &json!({
                "name": "Manager-owned room",
                "memberIds": [first_group_member.id, second_group_member.id],
            }),
        )
        .expect("group method handled")
        .expect("manager-owned group created");
    assert_eq!(created_group["agent"]["isGroup"], true);
    assert_eq!(
        created_group["agent"]["memberIds"].as_array().map(Vec::len),
        Some(2),
    );

    let agent = session.create_session(None, "user", None).expect("agent");
    let avatar = manager
        .dispatch_agent_lifecycle_gateway_call(
            &AgentDeletionRuntimeDeps::default(),
            None,
            "getAgentAvatar",
            &json!({"id": agent.id.clone()}),
        )
        .expect("agent lifecycle method handled")
        .expect("manager-owned lifecycle avatar");
    assert!(avatar.get("version").is_some());
    manager
        .switch_agent(&agent.id, 10.0)
        .expect("switch active agent");
    assert_eq!(
        sessions.memory_service().active_agent_id().as_deref(),
        Some(agent.id.as_str()),
    );
    sessions
        .append_agent_transcript_entries(
            &agent.id,
            &[json!({
                "id":"manager-secret-signal",
                "kind":"send-message",
                "message":{
                    "type":"secret-request",
                    "secretRequest":{
                        "label":"Manager-owned token",
                        "target":{"kind":"channel-credential","platform":"slack","field":"token"}
                    }
                }
            })],
        )
        .expect("append manager-owned secret request");
    assert!(manager
        .submit_secret_with(
            "manager-secret-signal",
            "manager-secret",
            &agent.id,
            11.0,
            |_| Ok(()),
        )
        .expect("manager-owned secret submission"));
    assert_eq!(*channel_config_changed.lock().expect("channel count"), 1);
    manager
        .set_window_focused(true, 123.0)
        .expect("manager window focus");
    assert_eq!(manager.window_focused_at_ms(), Some(123.0));
    assert_eq!(
        workflows_a.watched_agent_id().as_deref(),
        Some(agent.id.as_str())
    );

    let runtime = manager.transcript_runtime();
    runtime
        .session_runtime()
        .schedule_deferred_activation(&agent.id, None);
    assert_eq!(
        runtime.session_runtime().pending_activation_agent_id().as_deref(),
        Some(agent.id.as_str())
    );

    ack_a
        .record_send(&agent.id, 10.0)
        .expect("durable ack for redrive");
    assert!(ack_a.arm_redrive_timer(
        &agent.id,
        mahayana_host_runtime::extensions::transcript::ack_obligations::AckRedriveTrigger::Idle,
        10,
    ));
    assert!(ack_a.redrive_schedule(&agent.id).is_some());

    let handoff = BoxHandoffService::new(BoxHandoffDeps::default());
    let _ = handoff.start(HandoffRequest {
        agent_id: agent.id.clone(),
        instruction: "finish browser step".into(),
        telemetry: HandoffTelemetry::default(),
    });
    assert_eq!(handoff.pending_count(), 1);
    manager
        .set_handoff_service(handoff.clone())
        .expect("bind handoff");

    roster.set_outline_stream_coalescing_ms(60_000);
    roster.queue_outline_stream_update(&agent.id, "stream-item", json!({"text":"pending"}));
    assert!(roster.has_pending_outline_stream_update());

    assert!(!manager.is_disposed());
    manager.dispose();
    assert!(manager.is_disposed());
    assert!(manager.roster_emit().is_none());
    assert!(!roster.has_pending_outline_stream_update());
    manager.emit_automation_config_changed();
    manager.emit_channel_config_changed();
    manager.emit_listener_connect_card("agent-observer", "github");
    assert_eq!(*automation_config_changed.lock().expect("automation count"), 1);
    assert_eq!(*channel_config_changed.lock().expect("channel count"), 1);
    assert_eq!(listener_cards.lock().expect("listener cards").len(), 1);
    manager.dispose();
    assert_eq!(runners_a.active_count(), 0);
    assert!(workflows_a.watched_agent_id().is_none());
    assert!(runtime.session_runtime().pending_activation_agent_id().is_none());
    assert!(ack_a.redrive_schedule(&agent.id).is_none());
    {
        let wakes = wakes_a.lock().expect("background wakes after dispose");
        assert!(wakes.is_closed());
        assert!(wakes.pending_inbound.is_empty());
        assert!(wakes.pending_channel_failures.is_empty());
        assert!(wakes.pending_event_wakes.is_empty());
        assert!(wakes.reviving_inbound_agent_ids.is_empty());
        assert!(wakes.reviving_channel_failure_agent_ids.is_empty());
        assert!(wakes.reviving_event_agent_ids.is_empty());
        assert!(wakes.dm_preempted_wake_agent_ids.is_empty());
    }
    assert_eq!(handoff.pending_count(), 0);
    assert_eq!(sessions.memory_service().active_agent_id(), None);
    assert_eq!(sessions.active_agent_store_owner_count(), 0);
    assert_eq!(sessions.active_agent_db_owner_count(), 0);
    assert_eq!(sessions.active_worker_count(), 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn manager_dispose_closes_original_queue_before_cleanup_and_rejects_snapshotted_callback_and_ticket() {
    // Dropping the real registered observer during dispose can run captured
    // cleanup code. Observe that exact teardown boundary, not just its return.
    struct CleanupReadinessProbe {
        wakes: Arc<Mutex<BackgroundWakes<Value>>>,
        observed: Arc<Mutex<Vec<(bool, usize, bool)>>>,
    }
    impl Drop for CleanupReadinessProbe {
        fn drop(&mut self) {
            let mut wakes = self.wakes.lock().unwrap();
            let closed = wakes.is_closed();
            let tickets = wakes.request_inbound_readiness().len();
            let rejected = wakes.enqueue_inbound_and_claim("cleanup-agent", json!({"late":"cleanup"})).is_err();
            self.observed.lock().unwrap().push((closed, tickets, rejected));
        }
    }

    let root = temp_root();
    fs::create_dir_all(&root).unwrap();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let manager = Arc::new(TranscriptManager::new(&root, sessions));
    let queue = manager.background_wakes();
    let reserved = queue.lock().unwrap().enqueue_inbound_and_claim("agent-a", json!({"text":"reserved before spawn"}))
        .unwrap().unwrap();
    let cleanup_observed = Arc::new(Mutex::new(Vec::new()));
    let cleanup_probe = CleanupReadinessProbe { wakes: Arc::clone(&queue), observed: Arc::clone(&cleanup_observed) };
    manager.set_agent_run_lifecycle_observer(Some(Arc::new(move |_, _| {
        let _retained_for_observer_lifetime = &cleanup_probe;
    })));

    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let (observed_tx, observed_rx) = mpsc::channel();
    let callback_queue = Arc::clone(&queue);
    manager.set_channel_config_changed_observer(Some(Arc::new(move || {
        entered_tx.send(()).unwrap();
        release_rx.lock().unwrap().recv_timeout(Duration::from_secs(10)).expect("dispose before snapshotted callback resumes");
        let mut wakes = callback_queue.lock().unwrap();
        let closed = wakes.is_closed();
        let tickets = wakes.request_inbound_readiness().len();
        let rejected = wakes.enqueue_inbound_and_claim("agent-a", json!({"late":"snapshotted callback"})).is_err();
        observed_tx.send((closed, tickets, rejected)).unwrap();
    })));
    let emitting_manager = Arc::clone(&manager);
    let emitter = thread::spawn(move || emitting_manager.emit_channel_config_changed());
    entered_rx.recv_timeout(Duration::from_secs(5)).expect("real manager emitter has snapshotted its callback");
    manager.dispose();
    assert!(manager.is_disposed());
    assert!(Arc::ptr_eq(&queue, &manager.background_wakes()), "disposal retains the permanently closed queue object");
    assert_eq!(*cleanup_observed.lock().unwrap(), vec![(true, 0, true)], "closure precedes downstream observer teardown");
    release_tx.send(()).unwrap();
    assert_eq!(observed_rx.recv_timeout(Duration::from_secs(5)).unwrap(), (true, 0, true));
    emitter.join().expect("snapshotted real-manager callback returns");

    // A successful resume call and repeated disposal cannot replace the queue
    // with Default or grant the previously reserved ticket append permission.
    manager.resume_after_recreate().unwrap();
    manager.dispose();
    {
        let mut wakes = queue.lock().unwrap();
        assert!(wakes.is_closed());
        assert!(!wakes.owns_inbound_append("agent-a", &reserved));
        assert!(!wakes.release_inbound_launch("agent-a", &reserved));
        assert_eq!(wakes.pause_inbound_append("agent-a", &reserved), InboundAppendAction::Stale);
        assert_eq!(wakes.record_inbound_append_outcome("agent-a", &reserved, None,
            &ChannelInboundAppendOutcome::Committed { inserted_count: 1 }), InboundAppendAction::Stale);
        assert!(!wakes.finish_inbound_append("agent-a", &reserved));
        assert!(wakes.request_inbound_readiness().is_empty());
        assert_eq!(wakes.take_inbound_append_or_end_revival("agent-a"), None);
        assert!(wakes.pending_inbound.is_empty());
        assert!(wakes.reviving_inbound_agent_ids.is_empty());
    }

    let next_root = root.join("replacement");
    fs::create_dir_all(&next_root).unwrap();
    let next_sessions = Arc::new(ProductionSessionWorkers::with_agents_root(next_root.join("agents"), 500));
    let next_manager = TranscriptManager::new(&next_root, next_sessions);
    let next_queue = next_manager.background_wakes();
    assert!(!Arc::ptr_eq(&queue, &next_queue));
    {
        let mut wakes = next_queue.lock().unwrap();
        assert!(!wakes.is_closed(), "only a newly constructed manager owns a fresh open queue");
        let next = wakes.enqueue_inbound_and_claim("agent-a", reserved.envelopes[0].clone()).unwrap().unwrap();
        assert_ne!(next.claim_id, reserved.claim_id);
        assert_ne!(next.worker_token, reserved.worker_token);
        assert!(wakes.owns_inbound_append("agent-a", &next));
        assert!(!wakes.owns_inbound_append("agent-a", &reserved));
    }
    next_manager.dispose();
    assert!(queue.lock().unwrap().is_closed());
    let _ = fs::remove_dir_all(root);
}


struct ManagerFakeExecutor;

impl TurnExecutor for ManagerFakeExecutor {
    fn is_inference_ready(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(ready(true))
    }

    fn create_runner(&self, session: Value, hooks: Value) -> Value {
        json!({"kind":"runner","session":session,"hooks":hooks})
    }

    fn create_group_member_runner(
        &self,
        session: Value,
        hooks: Value,
        overrides: Value,
    ) -> Value {
        json!({
            "kind":"group",
            "session":session,
            "hooks":hooks,
            "overrides":overrides
        })
    }
}

#[test]
fn manager_owns_memory_gateway_and_invalidates_prompt_snapshot_after_mutation() {
    use mahayana_host_runtime::extensions::memory::memory_service::MemoryKind;

    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let session = SandAgentSessionStore::new(Arc::clone(&sessions));
    let agent = session.create_session(None, "user", None).expect("agent");
    let stored = sessions
        .memory_service()
        .store_for_agent(&agent.id)
        .add_memory("Remember this", 42, MemoryKind::Profile)
        .expect("memory write")
        .expect("memory record");
    sessions
        .set_agent_memory_prompt_snapshot(&agent.id, &json!({"fingerprint":"stale"}))
        .expect("snapshot");

    let listed = manager
        .dispatch_memory_gateway_call("getAgentMemories", &json!({"id":agent.id}))
        .expect("handled")
        .expect("list");
    assert_eq!(listed[0]["id"], stored.id);
    assert_eq!(listed[0]["content"], "Remember this");
    assert_eq!(listed[0]["createdAt"], 0);
    assert_eq!(listed[0]["kind"], "profile");

    let removed = manager
        .dispatch_memory_gateway_call(
            "deleteAgentMemory",
            &json!({"id":agent.id,"memoryId":stored.id}),
        )
        .expect("handled")
        .expect("delete");
    assert_eq!(removed, json!(true));
    assert!(sessions
        .get_agent_memory_prompt_snapshot(&agent.id)
        .expect("snapshot read")
        .is_none());

    sessions
        .memory_service()
        .store_for_agent(&agent.id)
        .add_memory("Clear this", 43, MemoryKind::Log)
        .expect("memory write")
        .expect("memory record");
    sessions
        .set_agent_memory_prompt_snapshot(&agent.id, &json!({"fingerprint":"stale-again"}))
        .expect("snapshot");
    assert_eq!(
        manager
            .dispatch_memory_gateway_call("clearAgentMemories", &json!({"id":agent.id}))
            .expect("handled")
            .expect("clear"),
        Value::Null,
    );
    assert!(sessions.memory_service().list(&agent.id).is_empty());
    assert!(sessions
        .get_agent_memory_prompt_snapshot(&agent.id)
        .expect("snapshot read")
        .is_none());

    sessions
        .begin_agent_delete(&agent.id)
        .expect("publish memory deletion fence");
    let blocked = manager
        .dispatch_memory_gateway_call("clearAgentMemories", &json!({"id":agent.id}))
        .expect("handled")
        .expect_err("memory gateway must honor deletion fence");
    assert!(blocked.contains("new session work is not admitted"));
    sessions
        .end_agent_delete(&agent.id)
        .expect("clear memory deletion fence");

    manager.dispose();
    let _ = fs::remove_dir_all(root);
}

// MGS-v1 uses real stores on both sides of the agents-root boundary. The
// outside store must be readable independently, so malformed fixture data
// cannot masquerade as refusal by the production Gateway entry.
struct MemoryGatewaySelectorFixture {
    root: std::path::PathBuf,
    manager: Option<TranscriptManager>,
    session_owner: Weak<ProductionSessionWorkers>,
    expected: Value,
    before: Vec<(std::path::PathBuf, Option<Vec<u8>>)>,
    notifications: Arc<AtomicU64>,
    unsubscribe: Option<mahayana_host_runtime::extensions::memory::memory_service::MemoryUnsubscribe>,
}

fn memory_gateway_selector_files(
    root: &std::path::Path,
) -> Vec<(std::path::PathBuf, Option<Vec<u8>>)> {
    fn visit(
        root: &std::path::Path,
        directory: &std::path::Path,
        entries: &mut Vec<(std::path::PathBuf, Option<Vec<u8>>)>,
    ) {
        for entry in fs::read_dir(directory).expect("owned memory fixture directory") {
            let entry = entry.expect("owned memory fixture entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("fixture-owned path").to_path_buf();
            let kind = entry.file_type().expect("owned memory fixture type");
            if kind.is_dir() {
                entries.push((relative, None));
                visit(root, &path, entries);
            } else {
                assert!(kind.is_file(), "fixture must not introduce symlink semantics");
                entries.push((relative, Some(fs::read(&path).expect("owned memory fixture bytes"))));
            }
        }
    }
    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

impl MemoryGatewaySelectorFixture {
    const AGENT_ID: &'static str = "memory-selector-agent";
    const DAY_MS: i64 = 86_400_000;

    fn new() -> Self {
        use mahayana_host_runtime::extensions::memory::memory_service::{FileMemoryStore, MemoryKind};

        let root = temp_root();
        fs::create_dir(&root).expect("create exclusively owned memory selector fixture");
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let mut fixture = Self {
            manager: Some(TranscriptManager::new(&root, Arc::clone(&sessions))),
            session_owner: Arc::downgrade(&sessions),
            root,
            expected: Value::Null,
            before: Vec::new(),
            notifications: Arc::new(AtomicU64::new(0)),
            unsubscribe: None,
        };
        let store = sessions.memory_service().store_for_agent(Self::AGENT_ID);
        let profile = store.add_memory("Existing profile ☃", Self::DAY_MS, MemoryKind::Profile)
            .expect("profile write").expect("new profile fact");
        let log = store.add_memory("Existing log ☃", 2 * Self::DAY_MS, MemoryKind::Log)
            .expect("log write").expect("new log fact");
        fixture.expected = json!([
            {"id": profile.id, "content": "Existing profile ☃", "createdAt": Self::DAY_MS, "kind": "profile"},
            {"id": log.id, "content": "Existing log ☃", "createdAt": 2 * Self::DAY_MS, "kind": "log"},
        ]);

        let outside = FileMemoryStore::new(fixture.root.join("outside/memory"));
        let sentinel = outside.add_memory(
            "External sentinel must never enter an agent reply", 3 * Self::DAY_MS, MemoryKind::Profile,
        ).expect("outside sentinel write").expect("new outside fact");
        assert_eq!(outside.list_memories(100), vec![sentinel], "outside oracle must be readable");

        let notifications = Arc::clone(&fixture.notifications);
        fixture.unsubscribe = Some(sessions.memory_service().subscribe(Arc::new(move || {
            notifications.fetch_add(1, Ordering::Relaxed);
        })));
        fixture.before = memory_gateway_selector_files(&fixture.root);
        fixture.assert_valid_read();
        fixture
    }

    fn manager(&self) -> &TranscriptManager {
        self.manager.as_ref().expect("live memory selector manager")
    }

    fn read(&self, args: Value) -> Result<Value, String> {
        self.manager().dispatch_memory_gateway_call("getAgentMemories", &args)
            .expect("existing memory read command must remain registered")
    }

    fn assert_unchanged(&self) {
        assert_eq!(memory_gateway_selector_files(&self.root), self.before,
            "a memory read must preserve valid/outside files and create no owner or directory");
        assert_eq!(self.notifications.load(Ordering::Relaxed), 0,
            "a memory read must not publish a mutation notification");
    }

    fn assert_valid_read(&self) {
        assert_eq!(self.read(json!({"id": Self::AGENT_ID})).expect("valid memory read"), self.expected);
        self.assert_unchanged();
    }

    fn assert_checked_list_error(&self, expected: &str) {
        let error = self.read(json!({"id": format!(" \t{}\n", Self::AGENT_ID)}))
            .expect_err("a storage error must not become a partial Gateway array");
        assert_eq!(error, expected);
        for private in [
            "Existing profile ☃", "Existing log ☃",
            "External sentinel must never enter an agent reply",
        ] {
            assert!(!error.contains(private), "read errors must not include memory content");
        }
        assert_eq!(self.read(json!({"id": " ../outside "})).expect_err("canonical selector"),
            "Invalid Sand agent id: ../outside");
        assert_eq!(self.read(json!({})).expect_err("required selector"),
            "getAgentMemories requires id");
        self.assert_unchanged();
    }
}

impl Drop for MemoryGatewaySelectorFixture {
    fn drop(&mut self) {
        if let Some(unsubscribe) = self.unsubscribe.take() {
            unsubscribe();
        }
        if let Some(manager) = self.manager.take() {
            manager.dispose();
            drop(manager);
        }
        // dispose() stops work, but the Session-owned CallSession SQLite handle
        // closes only when all manager fields and their Session Arcs are dropped.
        // Preserve a failing fixture without a second panic or premature deletion.
        if thread::panicking() {
            return;
        }
        assert!(self.session_owner.upgrade().is_none(),
            "memory selector Session owner must be destroyed before fixture cleanup");
        fs::remove_dir_all(&self.root).expect("remove owned memory selector fixture");
    }
}

#[test]
fn memory_gateway_rejects_path_selectors_before_external_memory_read() {
    let fixture = MemoryGatewaySelectorFixture::new();
    let absolute = fixture.root.join("outside").to_str().expect("UTF-8 fixture path").to_string();
    for selector in [
        ".", "..", "../outside", " \t../outside\n", "../outside/.",
        "memory-selector-agent/../../outside", "memory-selector-agent/memory",
        r"..\outside", r"memory-selector-agent\memory", r"C:\outside",
        "memory-selector-agent\0outside", absolute.as_str(),
    ] {
        let error = fixture.read(json!({"id": selector}))
            .expect_err("a path selector must fail before its external memory can become a reply");
        assert_eq!(error, format!("Invalid Sand agent id: {}", selector.trim()));
        fixture.assert_valid_read();
    }
}

#[cfg(windows)]
#[test]
fn memory_gateway_manager_rejects_native_drive_relative_selector() {
    use mahayana_host_runtime::extensions::memory::memory_service::{FileMemoryStore, MemoryKind};
    use std::path::{Component, PathBuf, Prefix};

    const CHILD_ROOT: &str = "FABUSHI_MEMORY_MANAGER_NATIVE_PATH_CHILD_ROOT";
    const TEST_NAME: &str = "memory_gateway_manager_rejects_native_drive_relative_selector";
    const MARKER: &str = "MEMORY_MANAGER_NATIVE_PATH_CHILD_COMPLETED";
    const SENTINEL: &str = "Windows drive-relative outside memory sentinel";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        let cwd = std::env::current_dir().expect("child CWD");
        assert_eq!(fs::canonicalize(&cwd).expect("actual child CWD"),
            fs::canonicalize(&root).expect("owned child root"));
        let drive = match cwd.components().next().expect("native Windows prefix") {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive,
                other => panic!("fixture requires a local drive, got {other:?}"),
            },
            other => panic!("fixture requires a native drive prefix, got {other:?}"),
        };
        let fixture = MemoryGatewaySelectorFixture::new();
        let outside = FileMemoryStore::new(root.join("drive-relative-outside/memory"));
        let sentinel = outside.add_memory(SENTINEL, MemoryGatewaySelectorFixture::DAY_MS, MemoryKind::Profile)
            .expect("outside seed").expect("outside record");
        let before = memory_gateway_selector_files(&outside.get_location());
        for drive in [char::from(drive).to_ascii_uppercase(), char::from(drive).to_ascii_lowercase()] {
            let selector = format!("{drive}:drive-relative-outside");
            let unsafe_join = fixture.root.join("agents").join(&selector).join("memory");
            assert!(!unsafe_join.starts_with(fixture.root.join("agents")));
            assert_eq!(FileMemoryStore::new(&unsafe_join).list_memories_checked(100)
                .expect("actual unsafe joined path must read the outside sentinel"), vec![sentinel.clone()]);
            let error = fixture.read(json!({"id": selector})).expect_err("native prefixed ID must be refused");
            assert_eq!(error, format!("Invalid Sand agent id: {selector}"));
            assert!(!error.contains(SENTINEL), "outside memory must not enter the error");
            assert_eq!(memory_gateway_selector_files(&outside.get_location()), before);
            fixture.assert_valid_read();
        }
        drop(fixture);
        println!("\n{MARKER}");
        return;
    }

    let cwd = std::env::current_dir().expect("parent CWD");
    let root = temp_root();
    fs::create_dir(&root).expect("parent owns child CWD");
    let result = std::process::Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
        .current_dir(&root)
        .env(CHILD_ROOT, &root)
        .output();
    let cleanup = fs::remove_dir_all(&root);
    let output = result.expect("execute the exact native-path child test");
    assert!(output.status.success(), "native-path child failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines()
        .filter(|line| *line == MARKER).count(), 1, "child must reach its terminal assertion marker");
    cleanup.expect("parent removes child CWD only after child exit");
    assert_eq!(std::env::current_dir().expect("unchanged parent CWD"), cwd);
}

#[cfg(unix)]
#[test]
fn memory_gateway_manager_preserves_literal_colon_agent_selector() {
    use mahayana_host_runtime::extensions::memory::memory_service::MemoryKind;

    let mut fixture = MemoryGatewaySelectorFixture::new();
    let record = fixture.manager().session_workers().memory_service().store_for_agent("C:outside")
        .add_memory("Unix literal colon profile", MemoryGatewaySelectorFixture::DAY_MS, MemoryKind::Profile)
        .expect("literal colon store").expect("literal colon record");
    fixture.before = memory_gateway_selector_files(&fixture.root);
    assert_eq!(fixture.read(json!({"id": " \tC:outside\n"})).expect("one normal Unix component"), json!([
        {"id": record.id, "content": record.content,
            "createdAt": MemoryGatewaySelectorFixture::DAY_MS, "kind": "profile"},
    ]));
    fixture.assert_valid_read();
}

#[test]
fn memory_gateway_keeps_required_trimmed_missing_and_unknown_read_contracts() {
    let fixture = MemoryGatewaySelectorFixture::new();
    for args in [
        Value::Null, json!({}), json!({"id": ""}), json!({"id": " \t\n"}),
        json!({"id": 7}), json!({"id": false}), json!({"id": null}),
        json!({"id": [MemoryGatewaySelectorFixture::AGENT_ID]}), json!({"id": {"id": "nested"}}),
    ] {
        assert_eq!(fixture.read(args).expect_err("required ID contract"), "getAgentMemories requires id");
        fixture.assert_valid_read();
    }
    assert_eq!(fixture.read(json!({
        "id": format!(" \t{}\n", MemoryGatewaySelectorFixture::AGENT_ID),
    })).expect("existing trimmed-ID compatibility"), fixture.expected);
    for missing in ["missing-safe-agent", " missing-safe-agent "] {
        assert_eq!(fixture.read(json!({"id": missing})).expect("safe missing ID"), json!([]));
        fixture.assert_valid_read();
    }
    assert!(fixture.manager().dispatch_memory_gateway_call(
        "getAgentMemories.extra", &json!({"id": "../outside"}),
    ).is_none(), "unknown methods keep the existing unhandled outcome");
    fixture.assert_valid_read();
}

#[test]
fn memory_gateway_preserves_projection_order_and_hundred_record_limit() {
    use mahayana_host_runtime::extensions::memory::memory_service::{FileMemoryStore, memory_id_for};

    let mut fixture = MemoryGatewaySelectorFixture::new();
    let memory_dir = fixture.root.join("agents")
        .join(MemoryGatewaySelectorFixture::AGENT_ID).join("memory");
    fs::write(memory_dir.join("profile.md"), concat!(
        "# Profiles\n\n- (1970-01-02) Earlier profile\n- (1970-01-03) Later profile\n",
    )).expect("ordered profile fixture");
    let log = (0..=100).map(|index| format!("- (1970-01-04) Log {index:03}\n"))
        .collect::<String>();
    fs::write(memory_dir.join("log/1970-01.md"), log).expect("bounded log fixture");
    assert_eq!(FileMemoryStore::new(&memory_dir).list_memories(usize::MAX).len(), 103,
        "the fixture must exceed the Gateway's existing bound");
    let mut expected = vec![
        json!({"id": memory_id_for("Later profile"), "content": "Later profile",
            "createdAt": 2 * MemoryGatewaySelectorFixture::DAY_MS, "kind": "profile"}),
        json!({"id": memory_id_for("Earlier profile"), "content": "Earlier profile",
            "createdAt": MemoryGatewaySelectorFixture::DAY_MS, "kind": "profile"}),
    ];
    for index in (3..=100).rev() {
        let content = format!("Log {index:03}");
        expected.push(json!({"id": memory_id_for(&content), "content": content,
            "createdAt": 3 * MemoryGatewaySelectorFixture::DAY_MS, "kind": "log"}));
    }
    assert_eq!(expected.len(), 100, "independent bounded projection oracle");
    fixture.expected = Value::Array(expected);
    fixture.before = memory_gateway_selector_files(&fixture.root);
    fixture.assert_valid_read();
}

#[test]
fn memory_gateway_checked_list_manager_preserves_corrupt_bytes_and_repair() {
    for relative in ["profile.md", "log/1970-01.md"] {
        let mut fixture = MemoryGatewaySelectorFixture::new();
        let memory = fixture.root.join("agents")
            .join(MemoryGatewaySelectorFixture::AGENT_ID).join("memory");
        let destination = memory.join(relative);
        let repaired = fs::read(&destination).expect("original readable bytes");
        let healthy = fixture.before.clone();
        let mut corrupt = repaired.clone();
        corrupt.extend_from_slice(b"<!-- invalid UTF-8: \xff\xfe -->\n");
        fs::write(&destination, &corrupt).expect("fixture-owned corruption");
        assert_eq!(fs::read(&destination).expect("raw corrupt bytes"), corrupt);
        let original_error = fs::read_to_string(&destination).expect_err("invalid UTF-8 fixture");
        assert_eq!(original_error.kind(), std::io::ErrorKind::InvalidData);
        fixture.before = memory_gateway_selector_files(&fixture.root);
        assert_eq!(fixture.manager().session_workers().memory_service()
            .list(MemoryGatewaySelectorFixture::AGENT_ID).len(), 1,
            "the old best-effort result would silently omit a real fact");

        fixture.assert_checked_list_error(&original_error.to_string());
        assert_eq!(fs::read(&destination).expect("raw bytes after refused read"), corrupt);

        fs::write(&destination, &repaired).expect("explicit fixture repair");
        fixture.before = healthy;
        fixture.assert_valid_read();
    }
}

#[test]
fn memory_gateway_checked_list_manager_preserves_path_type_failures() {
    for profile_obstacle in [true, false] {
        let mut fixture = MemoryGatewaySelectorFixture::new();
        let memory = fixture.root.join("agents")
            .join(MemoryGatewaySelectorFixture::AGENT_ID).join("memory");
        let destination = memory.join(if profile_obstacle { "profile.md" } else { "log" });
        let original = fs::read(if profile_obstacle {
            destination.clone()
        } else {
            destination.join("1970-01.md")
        }).expect("original readable bytes");
        let healthy = fixture.before.clone();
        if profile_obstacle {
            fs::remove_file(&destination).expect("remove fixture profile");
            fs::create_dir(&destination).expect("profile directory obstacle");
            fs::write(destination.join("sentinel.bin"), b"owned obstacle\x00\xff").expect("sentinel");
        } else {
            fs::remove_file(destination.join("1970-01.md")).expect("remove fixture log");
            fs::remove_dir(&destination).expect("remove fixture log directory");
            fs::write(&destination, b"owned log obstacle\x00\xff").expect("file at log directory");
        }
        let obstacle = fs::symlink_metadata(&destination).expect("actual obstacle type");
        assert_eq!(obstacle.is_dir(), profile_obstacle);
        assert_eq!(obstacle.is_file(), !profile_obstacle);
        let native_error = if profile_obstacle {
            Some(fs::read_to_string(&destination).expect_err("real profile read failure"))
        } else {
            fs::read_dir(&destination)
                .and_then(|entries| entries.collect::<std::io::Result<Vec<_>>>()).err()
        };
        eprintln!("MGL manager obstacle profile={profile_obstacle}: native error {:?}",
            native_error.as_ref().map(|error| (error.kind(), error.raw_os_error())));
        let error = fixture.manager().session_workers().memory_service()
            .list_checked(MemoryGatewaySelectorFixture::AGENT_ID).expect_err("checked store refuses type obstacle");
        if profile_obstacle {
            let native_error = native_error.expect("profile read failed");
            assert_eq!(error.kind(), native_error.kind());
            assert_eq!(error.raw_os_error(), native_error.raw_os_error());
        } else {
            assert_eq!(error.kind(), std::io::ErrorKind::NotADirectory);
        }
        fixture.before = memory_gateway_selector_files(&fixture.root);
        fixture.assert_checked_list_error(&error.to_string());

        if profile_obstacle {
            fs::remove_file(destination.join("sentinel.bin")).expect("remove owned sentinel");
            fs::remove_dir(&destination).expect("remove owned empty directory");
            fs::write(&destination, &original).expect("restore profile");
        } else {
            fs::remove_file(&destination).expect("remove owned log obstacle");
            fs::create_dir(&destination).expect("restore log directory");
            fs::write(destination.join("1970-01.md"), &original).expect("restore log");
        }
        fixture.before = healthy;
        fixture.assert_valid_read();
    }
}

#[test]
fn runner_registry_owns_turn_execution_and_manager_delegates_to_the_same_owner() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let runner_owner = manager.runner_registry();
    let (_, registry) = turn_execution_extension();
    let registry = Arc::new(Mutex::new(registry));
    manager.set_turn_execution(TranscriptTurnExecutionPort::new(Arc::clone(&registry)));

    assert!(runner_owner.turn_execution().is_some());
    assert!(!runner_owner.can_execute());
    assert!(!runner_owner.can_execute_group_member());
    assert!(!futures::executor::block_on(runner_owner.is_run_ready()));
    assert!(!manager.can_execute());
    assert!(!manager.can_execute_group_member());
    assert!(!futures::executor::block_on(manager.is_run_ready()));

    registry
        .lock()
        .expect("turn execution registry")
        .bind_executor(Box::new(ManagerFakeExecutor))
        .expect("bind executor");

    assert!(runner_owner.can_execute());
    assert!(runner_owner.can_execute_group_member());
    assert!(futures::executor::block_on(runner_owner.is_run_ready()));
    assert_eq!(
        runner_owner
            .create_runner(json!({"id":"owned-session"}), json!({"hook":"owned"}))
            .expect("owned runner")["kind"],
        "runner"
    );
    assert_eq!(
        runner_owner
            .create_group_member_runner(
                json!({"id":"owned-member"}),
                json!({"hook":"owned-group"}),
                json!({"model":"owned-override"}),
            )
            .expect("owned group runner")["kind"],
        "group"
    );

    assert!(manager.can_execute());
    assert!(manager.can_execute_group_member());
    assert!(futures::executor::block_on(manager.is_run_ready()));
    assert_eq!(
        manager
            .create_runner(json!({"id":"session"}), json!({"hook":"main"}))
            .expect("runner")["kind"],
        "runner"
    );
    assert_eq!(
        manager
            .create_group_member_runner(
                json!({"id":"member"}),
                json!({"hook":"group"}),
                json!({"model":"override"}),
            )
            .expect("group runner")["kind"],
        "group"
    );

    manager.dispose();
    let _ = fs::remove_dir_all(root);
}


#[derive(Default)]
struct ManagerPendingWakeRuntime {
    shell_watches: Mutex<Vec<(String, String)>>,
    reports: Mutex<Vec<PendingWakeReport>>,
}

impl PendingWakeRuntimePort for ManagerPendingWakeRuntime {
    fn can_execute(&self) -> bool {
        true
    }

    fn is_agent_gone(&self, _agent_id: &str) -> bool {
        false
    }

    fn is_group_session(&self, _agent_id: &str) -> Result<bool, String> {
        Ok(false)
    }

    fn cloud_watch_is_armed(&self, _agent_id: &str, _work_id: &str) -> bool {
        false
    }

    fn watch_cloud_agent(
        &self,
        _marker: &DurablePendingWakeMarker,
    ) -> Result<(), String> {
        Ok(())
    }

    fn watch_background_shell(
        &self,
        agent_id: &str,
        work_id: &str,
        _title: Option<&str>,
        _quiet_origin: Option<&QuietWakeOrigin>,
    ) -> Result<(), String> {
        self.shell_watches
            .lock()
            .expect("shell watches")
            .push((agent_id.to_string(), work_id.to_string()));
        Ok(())
    }

    fn deliver_recreate_interrupted_shell_notice(
        &self,
        _marker: &DurablePendingWakeMarker,
    ) -> Result<(), String> {
        Ok(())
    }

    fn revive_lost_subagent(&self, _wake: LostSubagentWake) -> Result<(), String> {
        Ok(())
    }

    fn revive_channel_delivery_failure(
        &self,
        _marker: &DurablePendingWakeMarker,
    ) -> Result<(), String> {
        Ok(())
    }

    fn emit_async_tasks_for_agent(&self, _agent_id: &str) {}

    fn report_pending_wake(&self, report: PendingWakeReport) {
        self.reports.lock().expect("pending-wake reports").push(report);
    }
}

#[test]
fn manager_owns_pending_wake_rearm_and_replays_durable_wakes() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let store = manager
        .transcript_runtime()
        .pending_wake_store()
        .cloned()
        .expect("production pending-wake store");
    store.mark_pending(DurablePendingWakeMarker {
        agent_id: "agent-pending".into(),
        kind: PendingWakeKind::Shell,
        work_id: "shell-pending".into(),
        marked_at_ms: chrono::Utc::now().timestamp_millis() as f64,
        quiet_origin: None,
        title: Some("Pending shell".into()),
        subagent_type: None,
        interrupted_by_recreate: false,
        delivery_operation_id: None,
        address_token: None,
        failure_reason: None,
        session_key: None,
    });

    let runtime = Arc::new(ManagerPendingWakeRuntime::default());
    manager
        .bind_pending_wake_runtime(runtime.clone())
        .expect("bind manager pending-wake owner");
    assert!(
        manager
            .bind_pending_wake_runtime(Arc::new(ManagerPendingWakeRuntime::default()))
            .is_err(),
        "TranscriptManager must reject a second pending-wake owner"
    );
    manager
        .rearm_pending_wakes()
        .expect("manager-owned pending-wake rearm");

    assert_eq!(
        runtime.shell_watches.lock().expect("shell watches").as_slice(),
        &[("agent-pending".to_string(), "shell-pending".to_string())]
    );
    assert!(
        runtime
            .reports
            .lock()
            .expect("pending-wake reports")
            .iter()
            .any(|report| report.outcome == "rearmed" && report.work_id == "shell-pending")
    );

    manager.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manager_refuses_partial_pending_wake_recovery_and_keeps_unknown_custody_busy() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, sessions);
    let transcript = manager.transcript_runtime();
    let store = transcript.pending_wake_store().expect("pending wake owner");
    let runtime = Arc::new(ManagerPendingWakeRuntime::default());
    manager.bind_pending_wake_runtime(runtime.clone()).expect("bind owner");
    assert!(!transcript.has_carryable_pending_wake());

    let marker = DurablePendingWakeMarker {
        agent_id: "agent-pending".into(),
        kind: PendingWakeKind::Shell,
        work_id: "shell-pending".into(),
        marked_at_ms: chrono::Utc::now().timestamp_millis() as f64,
        quiet_origin: None,
        title: Some("Pending shell".into()),
        subagent_type: None,
        interrupted_by_recreate: false,
        delivery_operation_id: None,
        address_token: None,
        failure_reason: None,
        session_key: None,
    };
    store.mark_pending_checked(marker.clone()).expect("persist original");
    let original = fs::read(store.file_path()).expect("original image");
    let corrupt = serde_json::to_vec(&json!({
        "version": 1,
        "pending": [marker, {"agentId": null}]
    })).expect("malformed row image");
    fs::write(store.file_path(), &corrupt).expect("persist malformed suffix");

    let error = manager.rearm_pending_wakes().expect_err("whole image must refuse");
    assert!(error.contains("pending wake"), "{error}");
    assert!(runtime.shell_watches.lock().expect("watches").is_empty());
    assert!(runtime.reports.lock().expect("reports").is_empty());
    assert!(transcript.has_carryable_pending_wake(), "unknown custody cannot authorize idle teardown");
    assert_eq!(fs::read(store.file_path()).expect("retained image"), corrupt);

    fs::write(store.file_path(), original).expect("restore original image");
    manager.rearm_pending_wakes().expect("repaired owner resumes original work");
    assert_eq!(runtime.shell_watches.lock().expect("watches").as_slice(),
        &[("agent-pending".to_string(), "shell-pending".to_string())]);
    // Ordinary Shell rearm hands off to its watcher and clears this marker;
    // the retained-custody policy is specific to the durable wake kinds.
    assert!(SandPendingWakeStore::new(&root).list_pending_checked()
        .expect("reopened pending state").is_empty());
    assert!(!transcript.has_carryable_pending_wake());
    manager.dispose();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn manager_is_the_upgrade_resume_facade_for_shipping_host_lifecycle() {
    use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker;

    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));

    assert!(!manager.is_quiescing_for_upgrade());
    manager.mark_upgrade_resume_pending(UpgradeResumeMarker {
        agent_id: "agent-upgrade".into(),
        marked_at_ms: 42.0,
        source: Some("background-revival".into()),
        automation_id: None,
        automation_run_id: None,
        original_work: None,
    }).expect("durable marker");
    assert_eq!(
        manager.transcript_runtime().upgrade_resume_agent_ids(),
        vec!["agent-upgrade".to_string()]
    );

    let registry = manager.runner_registry();
    assert!(!registry.is_quiescing_for_upgrade());
    let summary = manager.quiesce_for_upgrade().expect("durable handoff");
    assert!(summary.quiescing);
    assert_eq!(summary.running_turns, 0);
    assert!(manager.is_quiescing_for_upgrade());
    assert!(registry.is_quiescing_for_upgrade());
    manager.resume_after_recreate().expect("explicit resume");
    assert!(!manager.is_quiescing_for_upgrade());
    assert!(!registry.is_quiescing_for_upgrade());

    manager.dispose();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn manager_refuses_failed_marker_handoff_and_preserves_existing_quiesce() {
    use mahayana_host_runtime::extensions::transcript::sand_upgrade_resume_store::UpgradeResumeMarker;

    let root = temp_root();
    fs::create_dir_all(&root).unwrap();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let manager = TranscriptManager::new(&root, sessions);
    let runtime = manager.transcript_runtime();
    let registry = manager.runner_registry();
    runtime.begin_provider_run("running");
    let cancellation = registry.register_routed_provider("running", "marker-refusal-running").unwrap();
    let store = runtime.upgrade_resume_store().unwrap();
    fs::write(store.file_path(), b"bad recovery evidence").unwrap();
    assert!(manager.quiesce_for_upgrade().unwrap_err().contains("malformed"));
    assert!(!manager.is_quiescing_for_upgrade());
    assert!(!registry.is_quiescing_for_upgrade());
    assert!(!cancellation.is_cancelled(), "failed handoff cannot irreversibly cancel the task");
    assert!(runtime.is_agent_running("running"));
    assert!(manager.mark_upgrade_resume_pending(UpgradeResumeMarker {
        agent_id: "other".into(), marked_at_ms: 1.0, source: None,
        automation_id: None, automation_run_id: None,
        original_work: None,
    }).is_err());
    assert_eq!(fs::read(store.file_path()).unwrap(), b"bad recovery evidence");

    fs::remove_file(store.file_path()).unwrap();
    assert!(manager.quiesce_for_upgrade().unwrap().quiescing);
    assert!(manager.is_quiescing_for_upgrade());
    assert!(registry.is_quiescing_for_upgrade());
    assert!(cancellation.is_cancelled(), "cancellation follows durable handoff");
    fs::write(store.file_path(), b"later corruption").unwrap();
    assert!(manager.quiesce_for_upgrade().is_err());
    assert!(registry.is_quiescing_for_upgrade(), "a retry refusal cannot undo existing quiesce");
    assert!(manager.is_quiescing_for_upgrade());
    assert_eq!(fs::read(store.file_path()).unwrap(), b"later corruption");
    manager.resume_after_recreate().unwrap();
    assert!(!registry.is_quiescing_for_upgrade());
    assert!(!manager.is_quiescing_for_upgrade());
    manager.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manager_emits_active_session_automations_through_the_roster_surface() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = Arc::clone(&events);
    let roster = Arc::new(ProductionRosterEmit::new(
        Arc::clone(&sessions),
        manager.transcript_runtime(),
        Arc::new(move |event| captured.lock().expect("events").push(event)),
    ));
    manager.bind_roster_emit(roster).expect("bind roster");

    let active = sessions
        .materialize_new_session(None, "user", None)
        .expect("active agent");
    let inactive = sessions
        .materialize_new_session(None, "user", None)
        .expect("inactive agent");
    manager.switch_agent(&active.id, 1_000.0).expect("activate agent");

    let store = SandAgentSessionStore::new(Arc::clone(&sessions));
    let automations = store
        .create_agent_automation(
            &active.id,
            &AutomationSpec {
                name: "Morning Check".into(),
                prompt: "Check the inbox.".into(),
                trigger: json!({"type":"cron","schedule":"0 9 * * 1-5"}),
                is_enabled: Some(true),
            },
        )
        .expect("create automation");
    assert_eq!(automations.len(), 1);

    assert!(manager.emit_automations(&active.id).expect("emit active"));
    let emitted = events.lock().expect("events").clone();
    let automation_event = emitted
        .iter()
        .rev()
        .find(|event| event.get("channel").and_then(Value::as_str) == Some("automations"))
        .expect("automation event");
    assert_eq!(
        automation_event
            .pointer("/payload/agentId")
            .and_then(Value::as_str),
        Some(active.id.as_str())
    );
    assert_eq!(
        automation_event
            .pointer("/payload/automations/0/name")
            .and_then(Value::as_str),
        Some("Morning Check")
    );

    let before = emitted.len();
    assert!(!manager.emit_automations(&inactive.id).expect("skip inactive"));
    assert_eq!(events.lock().expect("events").len(), before);

    manager.dispose();
    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manager_owns_box_handoff_state_and_handback_settlement() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
        root.join("agents"),
        500,
    ));
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let roster = Arc::new(ProductionRosterEmit::new(
        Arc::clone(&sessions),
        manager.transcript_runtime(),
        Arc::new(|_| {}),
    ));
    manager
        .bind_roster_emit(roster)
        .expect("bind manager roster");

    let handoff = BoxHandoffService::new(BoxHandoffDeps::default());
    manager
        .set_handoff_service(handoff)
        .expect("bind manager handoff owner");

    let record = sessions
        .materialize_new_session(None, "user", None)
        .expect("materialize agent");
    sessions
        .set_agent_awaiting_user_response(
            &record.id,
            Some(&AwaitingUserResponse {
                tab_id: "box-tab".into(),
                reason: "box-help".into(),
                since: 1.0,
            }),
        )
        .expect("awaiting state");

    let started = manager
        .start_box_handoff(HandoffRequest {
            agent_id: record.id.clone(),
            instruction: "Complete the browser step".into(),
            telemetry: HandoffTelemetry::default(),
        })
        .expect("start manager-owned handoff");
    let request_id = match started {
        HandoffStartResult::Started { request_id } => request_id,
        HandoffStartResult::AlreadyPending { .. } => panic!("unexpected existing handoff"),
    };
    assert_eq!(
        manager
            .box_handoff(&record.id)
            .expect("manager pending handoff")
            .request_id,
        request_id
    );

    let entry = json!({
        "id": "box-manager-request",
        "kind": "send-message",
        "message": {"type":"text","content":"Complete the browser step"},
        "timestampMs": 2,
        "boxRequestId": request_id,
        "boxInstruction": "Complete the browser step"
    });
    manager
        .transcript_runtime()
        .track_box_request_entry(&record.id, &entry);
    sessions
        .append_agent_transcript_entries(&record.id, &[entry])
        .expect("persist box request");

    let decision = manager
        .hand_back_forever_box(
            &record.id,
            HandoffTrigger::Name("button".into()),
        )
        .expect("manager hand back");
    let HandoffDecision::End(decision) = decision else {
        panic!("expected handoff settlement");
    };
    assert_eq!(decision.resolution, "completed");
    assert!(manager.box_handoff(&record.id).is_none());
    assert!(
        sessions
            .get_agent_awaiting_user_response(&record.id)
            .expect("awaiting state")
            .is_none()
    );
    let entries = sessions
        .read_agent_transcript_entries(&record.id)
        .expect("read transcript");
    let resolved = entries
        .iter()
        .find(|entry| {
            entry.get("id").and_then(Value::as_str) == Some("box-manager-request")
        })
        .expect("resolved box entry");
    assert_eq!(resolved["boxResolution"], "completed");

    manager.dispose();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manager_secret_write_refusal_is_redacted_and_does_not_resume_before_repair() {
    use std::collections::{BTreeMap, BTreeSet};
    use mahayana_host_runtime::extensions::attachments::attachments_service::{
        AttachmentsBox, AttachmentsService,
    };
    use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
    use mahayana_host_runtime::extensions::content_search::extension::{
        GLOBAL_SEARCH_GATE, start_production_content_search_extension_with_paths,
    };
    use mahayana_host_runtime::extensions::experiments::{HostExperimentsExtension, HostExperimentsOptions};
    use mahayana_host_runtime::extensions::memory::extension::start_memory_extension;
    use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
    use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManagerServices;
    use mahayana_host_runtime::extensions::trays::extension::HostTraysExtension;
    use mahayana_host_runtime::extensions::trays::trays_service::TrayEvent;

    // These unrelated attachment ports must never be consumed by this path.
    // Credential storage, Session, Memory, manager and trays are real owners.
    struct UnusedAttachmentPorts;
    impl GenerateImageAuth for UnusedAttachmentPorts {
        fn get_access_token(&self) -> Result<String, String> {
            panic!("secret submission must not request attachment authentication")
        }
        fn get_machine_id(&self) -> Result<String, String> {
            panic!("secret submission must not request attachment machine identity")
        }
    }
    impl AttachmentsBox for UnusedAttachmentPorts {
        fn run_state(&self, _: &str) -> Result<String, String> {
            panic!("secret submission must not inspect an attachment box")
        }
        fn upload_file(&self, _: &str, _: &str, _: &[u8]) -> Result<(), String> {
            panic!("secret submission must not upload an attachment")
        }
    }

    let root = temp_root();
    fs::create_dir_all(&root).expect("credential manager root");
    let memory = start_memory_extension(root.join("agents"));
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root_and_dependencies(
        root.join("agents"), 500, Arc::new(|| None), memory.service(),
    ));
    assert!(Arc::ptr_eq(&sessions.memory_service(), &memory.service()), "fixture must share the actual Memory owner");
    let manager = TranscriptManager::new(&root, Arc::clone(&sessions));
    let telemetry = Arc::new(HostTelemetryService::open_with_identity_tags(
        root.join("credential-events.jsonl"), BTreeMap::new(),
    ).expect("real file-backed telemetry"));
    let experiments = Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: false, env_gate_overrides: None,
    }));
    experiments.replace_feature_flag_overrides(BTreeMap::from([(GLOBAL_SEARCH_GATE.to_string(), false)]));
    let content_search = Arc::new(start_production_content_search_extension_with_paths(
        &root, &root.join("agents"), experiments, Arc::new(|_| {}),
    ));
    let trays = Arc::new(HostTraysExtension::default());
    manager.set_production_services(TranscriptManagerServices {
        telemetry: Arc::clone(&telemetry.brain),
        product_analytics: telemetry.analytics.clone(),
        trace_flusher: telemetry.api(),
        memory: memory.clone(),
        content_search,
        attachments: Arc::new(AttachmentsService::new(
            &root, Arc::new(UnusedAttachmentPorts), Arc::new(UnusedAttachmentPorts), None,
        )),
        trays: Arc::clone(&trays),
    }).expect("bind actual manager services");
    assert!(Arc::ptr_eq(&manager.production_services().expect("bound services").trays, &trays),
        "manager must publish to the observed real tray owner");
    let tray_events = Arc::new(Mutex::new(Vec::<TrayEvent>::new()));
    let tray_events_capture = Arc::clone(&tray_events);
    let tray_subscription = trays.subscribe(Arc::new(move |event| {
        tray_events_capture.lock().expect("tray delivery events").push(event);
    }));
    let changed = Arc::new(AtomicU64::new(0));
    let changed_capture = Arc::clone(&changed);
    manager.set_channel_config_changed_observer(Some(Arc::new(move || {
        changed_capture.fetch_add(1, Ordering::SeqCst);
    })));
    let resumed = Arc::new(Mutex::new(Vec::<String>::new()));
    let agent = sessions.materialize_new_session(None, "user", None).expect("credential agent");
    let other = sessions.materialize_new_session(None, "user", None).expect("other credential agent");
    for (owner, platform) in [
        (agent.id.as_str(), "slack"),
        (agent.id.as_str(), "discord"),
        (other.id.as_str(), "slack"),
    ] {
        assert!(sessions.store_connector_credential(owner, platform, "token", "CCWA-EXISTING")
            .expect("seed real credential owner"), "valid credential fixture must be admitted");
    }
    sessions.append_agent_transcript_entries(&agent.id, &[json!({
        "id":"manager-checked-secret",
        "kind":"send-message",
        "message":{
            "type":"secret-request",
            "secretRequest":{
                "label":"Manager checked token",
                "target":{"kind":"channel-credential","platform":"slack","field":"token"}
            }
        }
    })]).expect("persist manager credential request");
    manager.switch_agent(&agent.id, 10.0).expect("stabilize active Agent before refusal baseline");
    assert!(sessions.memory_service().active_agent_id().as_deref() == Some(agent.id.as_str()),
        "manager must activate the shared Memory owner");
    let transcript_before = sessions.read_agent_transcript_entries(&agent.id).expect("request baseline");
    let obligations_before = manager.ack_obligations().pending_obligations();
    let store = sessions.connector_secret_store();
    let path = store.file_path(&agent.id, "slack");
    let untouched_paths = [
        sessions.open_channel_store(&agent.id).expect("target channel owner").config_path("slack"),
        store.file_path(&agent.id, "discord"),
        sessions.open_channel_store(&agent.id).expect("other platform owner").config_path("discord"),
        store.file_path(&other.id, "slack"),
        sessions.open_channel_store(&other.id).expect("other Agent channel owner").config_path("slack"),
    ];
    let untouched = untouched_paths.iter().map(|path| {
        (path.clone(), fs::read(path).expect("snapshot metadata and unrelated credential bytes"))
    }).collect::<Vec<_>>();
    let submitted = "  CCWA-MANAGER-SUBMITTED-SECRET  ";
    let faults: [Option<&[u8]>; 4] = [
        Some(br#""CCWA-MANAGER-STORED-SENTINEL""#),
        Some(br#"{"token":"CCWA-MANAGER-STORED-SENTINEL",broken}"#),
        Some(b"\xffCCWA-MANAGER-STORED-SENTINEL"),
        None,
    ];
    for (index, damaged) in faults.into_iter().enumerate() {
        if let Some(bytes) = damaged {
            fs::write(&path, bytes).expect("install real unreadable credential");
        } else {
            fs::remove_file(&path).expect("replace fixture file for real read error");
            fs::create_dir(&path).expect("credential read-error directory");
            fs::write(path.join("sentinel"), b"CCWA-MANAGER-DIRECTORY-SENTINEL")
                .expect("read-error directory contents");
        }
        let names_before = fs::read_dir(path.parent().expect("credential parent"))
            .expect("credential neighbors before refusal")
            .map(|entry| entry.expect("credential neighbor").file_name()).collect::<BTreeSet<_>>();
        let resumed_capture = Arc::clone(&resumed);
        let result = manager.submit_secret_with(
            "manager-checked-secret", submitted, &agent.id, 11.0,
            move |prompt| {
                resumed_capture.lock().expect("actual resume calls").push(prompt);
                Ok(())
            },
        );
        assert!(matches!(result, Ok(false)), "store refusal must preserve the manager accepted=false result");
        assert!(resumed.lock().expect("actual resume calls").is_empty(), "refusal must not resume or deliver an acknowledgement");
        assert_eq!(changed.load(Ordering::SeqCst), 0);
        assert!(sessions.read_agent_transcript_entries(&agent.id).expect("request after refusal") == transcript_before,
            "refusal must leave the real persisted request incomplete");
        assert!(manager.ack_obligations().pending_obligations() == obligations_before,
            "refusal must not create an acknowledgement obligation");
        if let Some(bytes) = damaged {
            assert!(fs::read(&path).expect("credential after refusal").as_slice() == bytes,
                "manager refusal must preserve exact credential bytes");
        } else {
            assert!(path.is_dir(), "manager refusal must preserve the read-error directory");
            assert!(fs::read(path.join("sentinel")).expect("read-error sentinel") == b"CCWA-MANAGER-DIRECTORY-SENTINEL",
                "manager refusal must preserve directory contents");
        }
        let names_after = fs::read_dir(path.parent().expect("credential parent"))
            .expect("credential neighbors after refusal")
            .map(|entry| entry.expect("credential neighbor").file_name()).collect::<BTreeSet<_>>();
        assert!(names_after == names_before, "manager refusal must leave no temporary credential entry");
        for (unchanged_path, bytes) in &untouched {
            assert!(fs::read(unchanged_path).expect("unchanged owner bytes") == *bytes,
                "manager refusal must preserve metadata and other Agent/platform state");
        }
        let current_trays = trays.list();
        assert_eq!(current_trays.len(), 1);
        let tray = &current_trays[0];
        assert!(tray.kind == "error" && tray.title == "Could not store the secret",
            "manager must use its fixed secret-write error tray");
        assert!(tray.detail == "The secure input could not write the credential to its store.",
            "manager must use the fixed redacted error detail");
        assert!(tray.agent_id.as_deref() == Some(agent.id.as_str()), "tray must identify the request's Agent");
        let dedupe_key = format!("transcript-secret-store:{}:manager-checked-secret", agent.id);
        assert!(tray.dedupe_key.as_deref() == Some(dedupe_key.as_str()), "tray must use existing request deduplication");
        assert_eq!(tray.count, Some((index + 1) as u64));
        assert!(tray.raw_detail.is_none() && tray.error_kind.is_none() && tray.request_id.is_none() && tray.actions.is_none(),
            "fixed refusal tray must not carry raw error payloads");
        let serialized = serde_json::to_string(tray).expect("serialize actual error tray");
        let root_text = root.to_string_lossy();
        for forbidden in [submitted.trim(), "CCWA-MANAGER-STORED-SENTINEL", "CCWA-MANAGER-DIRECTORY-SENTINEL", root_text.as_ref()] {
            assert!(!serialized.contains(forbidden), "error tray must not disclose stored/submitted secrets or paths");
        }
        let delivered = tray_events.lock().expect("tray delivery events");
        assert_eq!(delivered.len(), index + 1);
        assert!(matches!(delivered.last(), Some(TrayEvent::Pushed(delivered_tray)) if delivered_tray == tray),
            "the real tray owner must publish the same redacted error");
    }

    fs::remove_dir_all(&path).expect("explicitly repair final directory fault");
    fs::write(&path, br#"{"token":"CCWA-OLD","nested":{"keep":true},"number":7,"nullable":null}"#)
        .expect("repair the same credential path without altering the request");
    let trays_before_success = trays.list();
    let resumed_capture = Arc::clone(&resumed);
    let result = manager.submit_secret_with(
        "manager-checked-secret", submitted, &agent.id, 12.0,
        move |prompt| {
            resumed_capture.lock().expect("actual resume calls").push(prompt);
            Ok(())
        },
    );
    assert!(matches!(result, Ok(true)), "same manager request must succeed after explicit repair");
    assert_eq!(changed.load(Ordering::SeqCst), 1);
    {
        let prompts = resumed.lock().expect("actual resume calls");
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].contains("Manager checked token") && prompts[0].contains("channel-credential"),
            "resume must receive the actual successful request acknowledgement");
        assert!(!prompts[0].contains(submitted.trim()) && !prompts[0].contains(root.to_string_lossy().as_ref()),
            "successful acknowledgement must not expose secret value or path");
    }
    let committed_bytes = fs::read(&path).expect("committed manager credential");
    let committed: Value = serde_json::from_slice(&committed_bytes).expect("valid committed credential object");
    assert!(committed == json!({"token":"CCWA-MANAGER-SUBMITTED-SECRET","nested":{"keep":true},"number":7,"nullable":null}),
        "manager repair must merge the trimmed credential and preserve unrelated values");
    let committed_entries = sessions.read_agent_transcript_entries(&agent.id).expect("committed manager request");
    assert!(committed_entries.iter().any(|entry| entry["id"] == "manager-checked-secret" && entry["secretProvided"] == true),
        "successful manager request must durably record its provided flag");
    let repeated = manager.submit_secret_with(
        "manager-checked-secret", "CCWA-REPEATED-SECRET", &agent.id, 13.0,
        |_| panic!("completed secret request must not resume again"),
    );
    assert!(matches!(repeated, Ok(false)), "completed request must retain its existing ignored result");
    assert_eq!(changed.load(Ordering::SeqCst), 1);
    assert_eq!(resumed.lock().expect("actual resume calls").len(), 1);
    assert!(trays.list() == trays_before_success, "success and duplicate suppression must not emit a failure tray");
    assert_eq!(tray_events.lock().expect("tray delivery events").len(), 4);
    assert!(fs::read(&path).expect("credential after duplicate suppression") == committed_bytes,
        "completed request must not overwrite the credential again");
    for (unchanged_path, bytes) in &untouched {
        assert!(fs::read(unchanged_path).expect("unaffected owner bytes after repair") == *bytes,
            "successful repair must preserve metadata and unrelated owner state");
    }

    manager.dispose();
    drop(manager);
    drop(sessions);
    drop(memory);
    telemetry.dispose().expect("close file-backed telemetry");
    drop(telemetry);
    drop(tray_subscription);
    drop(trays);
    let reopened = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
    assert!(reopened.get_connector_secret(&agent.id, "slack", "token").expect("reopened manager credential").as_deref()
        == Some("CCWA-MANAGER-SUBMITTED-SECRET"), "fresh owner must observe the repaired credential");
    assert!(reopened.read_agent_transcript_entries(&agent.id).expect("reopened manager request") == committed_entries,
        "fresh owner must observe the committed request flag");
    assert!(fs::read(&path).expect("reopened raw credential") == committed_bytes,
        "reopening must preserve committed credential bytes");
    reopened.shutdown();
    drop(reopened);
    fs::remove_dir_all(root).expect("remove closed manager credential fixture");
}
