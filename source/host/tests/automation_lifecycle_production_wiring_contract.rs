use mahayana_host_runtime::automations::automation_schedule::summarize_schedule_next_7_days;

#[test]
fn frozen_schedule_summary_keeps_subminute_and_timezone_semantics() {
    let dense = summarize_schedule_next_7_days(
        "@every 30s",
        Some("America/Los_Angeles"),
        1_900_000_000_000.0,
    );
    assert_eq!(dense.scheduled_fires_next_7_days, 20_160);
    assert!(dense.fires_on_weekend);
    assert!(dense.fires_overnight);

    let daily = summarize_schedule_next_7_days(
        "0 9 * * *",
        Some("America/Los_Angeles"),
        1_900_000_000_000.0,
    );
    assert_eq!(daily.scheduled_fires_next_7_days, 7);
    assert!(daily.fires_on_weekend);
    assert!(!daily.fires_overnight);
}

#[test]
fn shipping_runtime_owns_all_frozen_lifecycle_sources_and_stable_fields() {
    let runtime = include_str!("../src/extensions/transcript/automation_runtime.rs");
    assert!(runtime.contains("stable_automation_id(agent_id, &snapshot.id)"));
    assert!(runtime.contains("summarize_schedule_next_7_days("));
    assert!(runtime.contains("AutomationLifecycleSource::Agent"));
    assert!(runtime.contains("AutomationLifecycleSource::AutomationsUi"));
    assert!(runtime.contains("AutomationLifecycleSource::WorkflowUi"));
    assert!(runtime.contains("AutomationLifecycleSource::SpendGuard"));
    assert!(runtime.contains("let _ = self.record_changes(agent_id, &before, &before, AutomationLifecycleSource::Agent);"));
    assert!(runtime.contains("store.set_on_change(Some(Arc::new(move ||"));
    assert!(runtime.contains("runtime.record_agent_store_change(&watched_agent_id)"));

    let snapshot = include_str!("../src/extensions/transcript/automation_snapshot.rs");
    assert!(snapshot.contains("None => Some(AutomationLifecycleAction::Created)") == false);
    assert!(snapshot.contains("AutomationAction::Updated"));
    assert!(snapshot.contains("AutomationAction::Enabled"));
    assert!(snapshot.contains("AutomationAction::Disabled"));

    let workflow = include_str!("../src/extensions/transcript/workflow_commands.rs");
    assert!(workflow.contains(".with_workflow_ui_mutation(agent_id, mutation)"));

    let spend_guard = include_str!("../src/extensions/transcript/automation_spend_guard_runtime.rs");
    assert!(spend_guard.contains("disable_every_enabled_routine"));
    assert!(spend_guard.contains("re_enable_guard_paused_routines"));
}

#[test]
fn production_composition_routes_to_unique_host_structured_log_owner() {
    let manager = include_str!("../src/extensions/transcript/transcript_manager.rs");
    assert!(manager.contains("watch_agent_automations(agent_id)"));
    assert!(manager.contains("previous.set_on_change(None)"));

    let app = include_str!("../../app/src/main.rs");
    assert!(app.contains(".set_lifecycle_reporter(Some(Arc::new(move |event|"));
    assert!(app.contains("lifecycle_logs.report_automation_lifecycle(event)"));

    let owner = include_str!("../src/extensions/telemetry/host_telemetry_service.rs");
    assert!(owner.contains("pub fn report_automation_lifecycle("));
    assert!(owner.contains("self.report_projection(&automation_lifecycle_telemetry(report))"));

    let mapper = include_str!("../src/extensions/telemetry/automation_lifecycle_telemetry.rs");
    assert_eq!(mapper.matches("sand.automation.lifecycle").count(), 1);
    assert!(mapper.contains(r#"level: Some("info")"#));

    let coordinator = include_str!("../../node-agent-coordinator/src/main.rs");
    assert!(!coordinator.contains("reportAutomationLifecycle"));
    assert!(!coordinator.contains("report_automation_lifecycle"));
    assert!(!coordinator.contains("sand.automation.lifecycle"));

    let electron = include_str!("../../electron-main/telemetry/desktop-structured-log-telemetry.ts");
    assert!(!electron.contains("reportAutomationLifecycle"));
    assert!(!electron.contains("sand.automation.lifecycle"));
}
