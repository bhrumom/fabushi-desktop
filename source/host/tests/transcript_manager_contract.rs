use std::fs;
use std::future::{ready, Future};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffRequest, HandoffTelemetry,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
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

    let workflows_a = manager.workflow_commands();
    let workflows_b = manager.workflow_commands();
    assert!(Arc::ptr_eq(&workflows_a, &workflows_b));

    let rooms_a = manager.shared_rooms();
    let rooms_b = manager.shared_rooms();
    assert!(Arc::ptr_eq(&rooms_a, &rooms_b));

    let ack_a = manager.ack_obligations();
    let ack_b = manager.ack_obligations();
    assert!(Arc::ptr_eq(&ack_a, &ack_b));
    ack_a.record_send("agent-a", 1.0).expect("durable ack");
    assert_eq!(ack_b.pending_obligations().len(), 1);

    let session = SandAgentSessionStore::new(Arc::clone(&sessions));
    let agent = session.create_session(None, "user", None).expect("agent");
    manager
        .switch_agent(&agent.id, 10.0)
        .expect("switch active agent");
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

    assert!(!manager.is_disposed());
    manager.dispose();
    assert!(manager.is_disposed());
    manager.dispose();
    assert_eq!(runners_a.active_count(), 0);
    assert!(workflows_a.watched_agent_id().is_none());
    assert!(runtime.session_runtime().pending_activation_agent_id().is_none());
    assert!(ack_a.redrive_schedule(&agent.id).is_none());
    assert_eq!(handoff.pending_count(), 0);
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
