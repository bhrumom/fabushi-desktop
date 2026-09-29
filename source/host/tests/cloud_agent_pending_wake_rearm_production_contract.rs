const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_host_rearms_durable_cloud_agent_and_lost_subagent_wakes_after_gateway_start() {
    assert!(SHIPPING_HOST.contains("PendingWakeRearm::new("));
    assert!(SHIPPING_HOST.contains("PendingWakeKind::CloudAgent | PendingWakeKind::Subagent"));
    assert!(SHIPPING_HOST.contains("rearm.rearm_pending_wake(pending, now_ms, Some(\"host_startup\"))"));
    assert!(SHIPPING_HOST.contains("is_cloud_watch_armed(agent_id, work_id)"));
    assert!(SHIPPING_HOST.contains("CloudAgentWatchOptions::new(quiet_origin, false)"));
    assert!(SHIPPING_HOST.contains("completion_revivals: Arc<CompletionRevivals>"));
    assert!(SHIPPING_HOST.contains("handle_background_subagent_completion(SubagentCompletion"));
}

#[test]
fn startup_rearm_routes_recreate_interrupted_shell_through_completion_revivals() {
    assert!(SHIPPING_HOST.contains("production shell pending-wake rearm is not wired yet"));
    assert!(!SHIPPING_HOST.contains("production recreate-interrupted shell notice is not wired yet"));
    assert!(SHIPPING_HOST.contains("handle_background_shell_completion(ShellCompletion"));
    assert!(SHIPPING_HOST.contains("shell_id: marker.work_id.clone()"));
    assert!(SHIPPING_HOST.contains("quiet_origin: marker.quiet_origin.clone()"));
    assert!(!SHIPPING_HOST.contains("production lost-subagent pending-wake revival is not wired yet"));
}
