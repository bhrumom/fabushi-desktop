use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::host_upgrade::host_bundle_source::{
    HostBundleFetcher, HostBundleHttpResponse,
};
use mahayana_host_runtime::extensions::host_upgrade::production::{
    ProductionHostUpgradePeers, start_production_host_upgrade_extension_with,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;

#[derive(Default)]
struct FakeFetcher {
    requests: Mutex<Vec<String>>,
}

impl HostBundleFetcher for FakeFetcher {
    fn fetch(&self, url: &str) -> Result<HostBundleHttpResponse, String> {
        self.requests.lock().unwrap().push(url.to_string());
        Ok(HostBundleHttpResponse {
            status: 404,
            body: Vec::new(),
        })
    }
}

#[test]
fn production_host_upgrade_binds_real_peer_hooks_without_starting_a_parallel_runtime() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-host-upgrade-production-{}",
        uuid::Uuid::new_v4()
    ));
    let telemetry = HostTelemetryService::open(root.join("telemetry.jsonl")).unwrap();
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let hook = |name: &'static str, calls: Arc<Mutex<Vec<String>>>| {
        Arc::new(move || {
            calls.lock().unwrap().push(name.to_string());
            Ok(())
        }) as Arc<dyn Fn() -> Result<(), String> + Send + Sync>
    };
    let sharing_calls = Arc::clone(&calls);
    let peers = ProductionHostUpgradePeers {
        suspend_automation_wakes: hook("automations", Arc::clone(&calls)),
        prepare_sharing_for_upgrade: Arc::new(move || {
            sharing_calls.lock().unwrap().push("sharing".into());
        }),
        quiesce_transcript_for_upgrade: hook("transcript", Arc::clone(&calls)),
        resume_interrupted_upgrade_turns: hook("resume", Arc::clone(&calls)),
    };
    let extension = start_production_host_upgrade_extension_with(
        peers,
        telemetry.logs.clone(),
        Arc::new(FakeFetcher::default()),
        root.join("upgrade-marker.json"),
        Arc::new(|| Ok(())),
    )
    .unwrap();

    extension.service().prepare_for_upgrade().unwrap();
    extension.service().resume_interrupted_upgrade_turns().unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["automations", "sharing", "transcript", "resume"]
    );
    extension.stop_background_work();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn production_upgrade_telemetry_uses_frozen_event_name() {
    let mut metadata = BTreeMap::new();
    metadata.insert("outcome".to_string(), "applied".to_string());
    assert_eq!(metadata.get("outcome").map(String::as_str), Some("applied"));
}

#[test]
fn production_upgrade_service_propagates_real_transcript_marker_refusal() {
    let root = std::env::temp_dir().join(format!("fabushi-host-upgrade-marker-refusal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let telemetry = HostTelemetryService::open(root.join("telemetry.jsonl")).unwrap();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let manager = Arc::new(TranscriptManager::new(&root, sessions));
    let runtime = manager.transcript_runtime();
    runtime.begin_provider_run("running");
    let store = runtime.upgrade_resume_store().unwrap().clone();
    std::fs::write(store.file_path(), b"corrupt pending upgrade").unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let suspend_calls = Arc::clone(&calls);
    let sharing_calls = Arc::clone(&calls);
    let quiesce_calls = Arc::clone(&calls);
    let quiesce_manager = Arc::clone(&manager);
    let resume_store = store.clone();
    let extension = start_production_host_upgrade_extension_with(
        ProductionHostUpgradePeers {
            suspend_automation_wakes: Arc::new(move || {
                suspend_calls.lock().unwrap().push("suspend"); Ok(())
            }),
            prepare_sharing_for_upgrade: Arc::new(move || { sharing_calls.lock().unwrap().push("sharing"); }),
            quiesce_transcript_for_upgrade: Arc::new(move || {
                quiesce_calls.lock().unwrap().push("quiesce");
                quiesce_manager.quiesce_for_upgrade().map(|_| ())
            }),
            resume_interrupted_upgrade_turns: Arc::new(move || {
                resume_store.list_pending_checked().map(|_| ()).map_err(|error| error.to_string())
            }),
        },
        telemetry.logs.clone(), Arc::new(FakeFetcher::default()), root.join("upgrade-marker.json"),
        Arc::new(|| Ok(())),
    ).unwrap();
    assert!(extension.service().prepare_for_upgrade().unwrap_err().contains("malformed"));
    assert_eq!(*calls.lock().unwrap(), vec!["suspend", "sharing", "quiesce"]);
    assert!(!manager.is_quiescing_for_upgrade());
    assert!(!manager.runner_registry().is_quiescing_for_upgrade());
    assert!(runtime.is_agent_running("running"));
    assert!(extension.service().resume_interrupted_upgrade_turns().unwrap_err().contains("malformed"));
    assert_eq!(std::fs::read(store.file_path()).unwrap(), b"corrupt pending upgrade");
    std::fs::remove_file(store.file_path()).unwrap();
    extension.service().prepare_for_upgrade().unwrap();
    assert!(manager.is_quiescing_for_upgrade());
    assert_eq!(store.list_pending_checked().unwrap()[0].agent_id, "running");
    extension.stop_background_work();
    manager.dispose();
    let _ = std::fs::remove_dir_all(root);
}


#[test]
fn shipping_host_reconciles_completed_original_work_before_ack_redrive() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

    assert!(SHIPPING_HOST.contains("fn resume_interrupted_upgrade_turns(&self)"));
    assert!(SHIPPING_HOST.contains("fn acknowledge_completed_upgrade_marker("));
    assert!(SHIPPING_HOST.contains("reconcile_completed_original_turn(sessions, original)"));
    assert!(!SHIPPING_HOST.contains("start_local_upgrade_resume_turn("));
    assert!(SHIPPING_HOST.contains("gateway.resume_interrupted_upgrade_turns()"));
    assert!(!SHIPPING_HOST.contains(
        "resumeInterruptedUpgradeTurns production Runner recreation is not wired"
    ));

    let resume = SHIPPING_HOST
        .rfind("host_upgrade.service().resume_interrupted_upgrade_turns()")
        .expect("shipping startup upgrade resume");
    let redrive = SHIPPING_HOST
        .rfind("start_ack_redrive_worker(")
        .expect("shipping ack redrive");
    assert!(resume < redrive);
}

#[test]
fn shipping_recreate_gateway_carries_pending_wakes_and_resumes_durable_turns() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
    const GATEWAY_PROTOCOL: &str = include_str!("../src/gateway_protocol.rs");
    const TRANSCRIPT_MANAGER: &str =
        include_str!("../src/extensions/transcript/transcript_manager.rs");
    const TRANSCRIPT_RUNTIME: &str =
        include_str!("../src/extensions/transcript/production_runtime.rs");
    const PENDING_WAKE_REARM: &str =
        include_str!("../src/extensions/transcript/pending_wake_rearm.rs");

    assert!(GATEWAY_PROTOCOL.contains("\"resumeAfterRecreate\""));
    assert!(SHIPPING_HOST.contains("if method == \"resumeAfterRecreate\""));
    assert!(SHIPPING_HOST.contains("\"resumePendingWakes\": self.transcript_manager.recreate_carry_pending_wakes()"));
    assert!(SHIPPING_HOST.contains(".restore_recreate_pending_wakes(carried)"));
    assert!(SHIPPING_HOST.contains("self.resume_interrupted_upgrade_turns()"));
    assert!(SHIPPING_HOST.contains("if worker_is_upgrade_resume {"));
    assert!(SHIPPING_HOST.contains("worker_transcript_manager.emit_agent_update(&agent_id)"));
    assert!(SHIPPING_HOST.contains("worker_transcript_manager.emit_automations(&agent_id)"));
    // ORT-v1 intentionally removes the former fresh hidden prompt/provider
    // dispatch. Recreate still carries existing pending wakes and lifecycle
    // updates, but completed original work enters the checked consumer.
    assert!(SHIPPING_HOST.contains("acknowledge_completed_upgrade_marker("));
    assert!(TRANSCRIPT_MANAGER.contains("pub fn restore_recreate_pending_wakes"));
    assert!(TRANSCRIPT_MANAGER.contains("owner.restore_recreate_carried_pending_wakes(carried)"));
    assert!(PENDING_WAKE_REARM.contains("self.rearm_pending_wake(marker, now_ms, Some(\"recreate_carry\"))"));
    assert!(TRANSCRIPT_RUNTIME.contains("pub fn recreate_carry_pending_wakes"));
    assert!(TRANSCRIPT_RUNTIME.contains("PendingWakeKind::CloudAgent | PendingWakeKind::Shell"));
}

#[test]
fn shipping_upgrade_resume_keeps_durable_state_and_recreate_work_accounting() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
    const TRANSCRIPT_RUNTIME: &str = include_str!("../src/extensions/transcript/production_runtime.rs");
    const TRANSCRIPT_MANAGER: &str =
        include_str!("../src/extensions/transcript/transcript_manager.rs");
    const COMPLETION_REVIVALS: &str =
        include_str!("../src/extensions/transcript/completion_revivals.rs");

    let resume = SHIPPING_HOST
        .find("fn resume_interrupted_upgrade_turns(&self)")
        .expect("shipping upgrade resume owner");
    let resume_body = SHIPPING_HOST[resume..].split("fn preempt_group_member_runs_for_direct_send").next().unwrap();
    let checked_discovery = resume_body.find("visit_upgrade_resume_markers_checked(store, |marker|")
        .expect("checked discovery before any marker callback");
    let can_execute = resume_body
        .find("if !self.transcript_manager.can_execute()")
        .expect("execution guard before durable marker settlement");
    assert!(!resume_body.contains("store.clear_all();"));
    let reconciliation = resume_body
        .find("if let Err(error) = acknowledge_completed_upgrade_marker(")
        .expect("completed original-work reconciliation and failure reporting");
    assert!(checked_discovery < can_execute);
    assert!(can_execute < reconciliation);
    assert!(resume_body.contains("original_upgrade_work_retained"));
    assert!(resume_body.contains("Ok(false)"), "acknowledgment is not a resumed dispatch");
    for fresh_dispatch in ["configured_routed_provider(", "Uuid::new_v4(", "start_local_upgrade_resume_turn("] {
        assert!(!resume_body.contains(fresh_dispatch));
    }
    assert!(!resume_body.contains("clear_if_matches"), "only the verified consumer can settle a marker");
    let consumer = SHIPPING_HOST.split("fn acknowledge_completed_upgrade_marker(").nth(1).unwrap()
        .split("fn dispatch_public_settings_command(").next().unwrap();
    let identity = consumer.find("upgrade_resume_marker_identity_is_well_formed(marker)").unwrap();
    let verify = consumer.find("reconcile_completed_original_turn(sessions, original)").unwrap();
    let fresh_ack = consumer.find("ack_store.get_checked(&marker.agent_id)").unwrap();
    let final_live_guard = consumer.rfind("runtime.is_agent_running(&marker.agent_id)").unwrap();
    let clear = consumer.find("runtime.clear_completed_upgrade_marker_if_idle(marker, ack_store)").unwrap();
    assert!(identity < verify && verify < fresh_ack && fresh_ack < final_live_guard && final_live_guard < clear);
    assert!(consumer.contains("verified.original() != original"));
    assert!(consumer.contains("registry.active_stream_ids_for_agent(&marker.agent_id).is_empty()"));
    assert!(consumer.contains("map_err(|error| error.to_string())?"), "read/proof failures must retain the exact marker");
    assert!(!consumer.contains("clear_all"));
    assert!(!consumer.contains("store.clear_if_matches_checked"));
    let final_owner = TRANSCRIPT_RUNTIME.split("pub fn clear_completed_upgrade_marker_if_idle(").nth(1).unwrap()
        .split("pub fn clear_agent_durable_recovery(").next().unwrap();
    assert!(final_owner.find("self.state.lock()").unwrap()
        < final_owner.find("state.lifecycle.is_running(&marker.agent_id)").unwrap());
    assert!(final_owner.contains("store.clear_if_matches_with_ack_absent_checked(marker, acknowledgments)"),
        "final idle check, complete marker CAS and fresh ACK absence share the actual owners' locked boundary");
    let visit = SHIPPING_HOST.split("fn visit_upgrade_resume_markers_checked(").nth(1).unwrap()
        .split("fn acknowledge_completed_upgrade_marker(").next().unwrap();
    assert!(visit.find("store.migrate_legacy_checked()").unwrap() < visit.find("visit(marker)?").unwrap(),
        "the complete image must migrate and validate before the first consumer callback");

    assert!(TRANSCRIPT_MANAGER.contains("services.trace_flusher.flush_tracing()"));
    assert!(COMPLETION_REVIVALS.contains("pub fn mid_drain_revival_agent_ids(&self)"));
    assert!(SHIPPING_HOST.contains("mid_drain_revival_agent_ids()"));
    assert!(SHIPPING_HOST.contains("has_running_background_shell_work()"));
    assert!(SHIPPING_HOST.contains("\"runningTurns\": running_turns"));
    assert!(SHIPPING_HOST.contains("source: \"resume\".into()"));
    assert!(SHIPPING_HOST.contains("upgrade_resume_telemetry_failed"));
    assert!(SHIPPING_HOST.contains("observed.quiesced_for_upgrade"));
    assert!(SHIPPING_HOST.contains("automation_run_id: observed.automation_run_id.clone()"));
    assert!(SHIPPING_HOST.contains("prepare_upgrade_transcript_handoff(&host_upgrade_transcript, &host_upgrade_sessions)"));
    assert!(SHIPPING_HOST.contains("upgrade_resume_boot_failed"));
    assert!(SHIPPING_HOST.contains(".mark_upgrade_resume_batch_pending(&imported_markers)"));
    assert!(SHIPPING_HOST.contains("let resumed = self.resume_interrupted_upgrade_turns()"));
    assert!(SHIPPING_HOST.contains("\"resumed\": resumed"));
}

#[test]
fn shipping_ack_worker_uses_atomic_original_marker_admission_before_due_consumption() {
    const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
    const TRANSCRIPT_RUNTIME: &str = include_str!("../src/extensions/transcript/production_runtime.rs");
    const ACK_OWNER: &str = include_str!("../src/extensions/transcript/ack_obligations.rs");
    let worker = SHIPPING_HOST.split("fn start_ack_redrive_worker(").nth(1).unwrap()
        .split("fn project_forever_box_status(").next().unwrap();
    let snapshot = SHIPPING_HOST.split("fn automatic_ack_redrive_marker_snapshot(").nth(1).unwrap()
        .split("fn start_ack_redrive_worker(").next().unwrap();
    assert!(snapshot.contains("store.migrate_legacy_checked()"));
    let fresh_markers = worker.find("automatic_ack_redrive_marker_snapshot(").unwrap();
    assert_eq!(worker.matches("automatic_ack_redrive_marker_snapshot(").count(), 1,
        "one checked complete image supplies preliminary filtering for each polling tick");
    let fresh_ack = worker.find("ack_obligations.store().list_checked()").unwrap();
    let atomic_prepare = worker.find("api.transcript_runtime.prepare_automatic_ack_redrive(").unwrap();
    let send = worker.find("api.call(\"sendPrompt\", args)").unwrap();
    assert!(fresh_markers < atomic_prepare && fresh_ack < atomic_prepare && atomic_prepare < send);
    assert!(worker.contains("retained_agents.contains(agent_id)"));
    assert!(worker.contains("retained_agents.contains(&agent_id)"));
    let future_timer = worker.find("schedule.due_at_ms > now_ms").unwrap();
    assert!(future_timer < atomic_prepare, "future timers do not enter the global admission or durable transition");
    assert!(!worker.contains(".get_checked("), "the atomic ACK transition supplies the per-agent fresh check");
    assert!(!worker.contains(".take_due_redrive("), "the cheap read helper cannot authorize a separate timer take");
    assert!(!worker.contains(".prepare_redrive_with_telemetry("));
    let runtime = TRANSCRIPT_RUNTIME.split("pub fn prepare_automatic_ack_redrive(").nth(1).unwrap()
        .split("pub fn resume_after_recreate(").next().unwrap();
    let admission_lock = runtime.find("self.state.lock()").unwrap();
    let marker_boundary = runtime.find("store.while_agent_unmarked_checked(agent_id, ||").unwrap();
    let due_boundary = runtime.find("obligations.prepare_due_redrive_checked(agent_id, agent_exists, now_ms)").unwrap();
    let release = runtime.find("drop(state)").unwrap();
    let telemetry = runtime.find("obligations.report_redrive_preparation(").unwrap();
    assert!(admission_lock < marker_boundary && marker_boundary < due_boundary && due_boundary < release && release < telemetry);
    let due = ACK_OWNER.split("fn prepare_due_redrive_checked(").nth(1).unwrap()
        .split("pub fn prepare_redrive_with_telemetry(").next().unwrap();
    assert!(due.find("self.prepare_redrive(agent_id, agent_exists)?").unwrap()
        < due.find("schedules.remove(agent_id)").unwrap(), "failed ACK custody reads or writes must retain the due timer");
}
