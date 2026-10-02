use std::fs;
use std::future::{ready, Future};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::transcript::agent_lifecycle::AgentDeletionRuntimeDeps;
use mahayana_host_runtime::extensions::session::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffRequest, HandoffTelemetry,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::client_side_tool_v2_producer::{
    ClientSideToolV2ProducedValue, ClientSideToolV2TransportKind,
};
use mahayana_host_runtime::extensions::transcript::roster_emit::ProductionRosterEmit;
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
    std::env::temp_dir().join(format!(
        "fabushi-transcript-manager-{}-{suffix}",
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
    let first_group_member = session
        .create_session(None, "user", None)
        .expect("first group member");
    let second_group_member = session
        .create_session(None, "user", None)
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
    assert_eq!(created_group["isGroup"], true);
    assert_eq!(created_group["memberIds"].as_array().map(Vec::len), Some(2));

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
