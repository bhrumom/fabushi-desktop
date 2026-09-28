const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_host_rearms_durable_cloud_agent_wakes_after_gateway_start() {
    assert!(SHIPPING_HOST.contains("PendingWakeRearm::new("));
    assert!(SHIPPING_HOST.contains("marker.kind == PendingWakeKind::CloudAgent"));
    assert!(SHIPPING_HOST.contains("rearm.rearm_pending_wake(pending, now_ms, Some(\"host_startup\"))"));
    assert!(SHIPPING_HOST.contains("is_cloud_watch_armed(agent_id, work_id)"));
    assert!(SHIPPING_HOST.contains("CloudAgentWatchOptions::new(quiet_origin, false)"));
}

#[test]
fn startup_rearm_does_not_claim_shell_or_subagent_shipping_parity() {
    assert!(SHIPPING_HOST.contains("production shell pending-wake rearm is not wired yet"));
    assert!(SHIPPING_HOST.contains("production lost-subagent pending-wake revival is not wired yet"));
}
