const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_host_rearms_durable_cloud_agent_and_lost_subagent_wakes_after_gateway_start() {
    assert!(SHIPPING_HOST.contains("PendingWakeRearm::new("));
    assert!(SHIPPING_HOST.contains("for pending in store.list_pending()"));
    assert!(SHIPPING_HOST.contains("background_shell_watches: Arc<RunnerBackgroundShellWatches>"));
    assert!(SHIPPING_HOST.contains("rearm.rearm_pending_wake(pending, now_ms, Some(\"host_startup\"))"));
    assert!(SHIPPING_HOST.contains("is_cloud_watch_armed(agent_id, work_id)"));
    assert!(SHIPPING_HOST.contains("CloudAgentWatchOptions::new(quiet_origin, false)"));
    assert!(SHIPPING_HOST.contains("completion_revivals: Arc<CompletionRevivals>"));
    assert!(SHIPPING_HOST.contains("handle_background_subagent_completion(SubagentCompletion"));
}

#[test]
fn startup_rearm_routes_recreate_interrupted_shell_through_completion_revivals() {
    assert!(!SHIPPING_HOST.contains("production shell pending-wake rearm is not wired yet"));
    assert!(SHIPPING_HOST.contains("watch_background_shell("));
    assert!(SHIPPING_HOST.contains("BackgroundShellWatchOptions::new("));
    assert!(!SHIPPING_HOST.contains("production recreate-interrupted shell notice is not wired yet"));
    assert!(SHIPPING_HOST.contains("handle_background_shell_completion(ShellCompletion"));
    assert!(SHIPPING_HOST.contains("shell_id: marker.work_id.clone()"));
    assert!(SHIPPING_HOST.contains("quiet_origin: marker.quiet_origin.clone()"));
    assert!(!SHIPPING_HOST.contains("production lost-subagent pending-wake revival is not wired yet"));
}


#[test]
fn shipping_generated_subagent_dispatch_persists_durable_pending_wake_before_provider_run() {
    let dispatch = SHIPPING_HOST
        .find("if let Ok(Some(pending)) = runner.begin_generated_subagent(")
        .expect("shipping generated subagent dispatch must expose its pending wake");
    let provider_run = SHIPPING_HOST
        .find("runner.run_routed_provider_with_projected_messages(")
        .expect("shipping generated subagent must execute through the Runner");
    assert!(dispatch < provider_run);
    assert!(SHIPPING_HOST.contains("kind: PendingWakeKind::Subagent"));
    assert!(SHIPPING_HOST.contains("work_id: pending.work_id.clone()"));
    assert!(SHIPPING_HOST.contains("subagent_type: Some(pending.subagent_type.clone())"));
    assert!(SHIPPING_HOST.contains("pending_subagent_wake_persist_failed"));
    assert!(SHIPPING_HOST.contains("worker_transcript_runtime.get_async_tasks(parent_agent_id, &[])"));
    assert!(SHIPPING_HOST.contains("handle_background_subagent_completion(SubagentCompletion"));
}
