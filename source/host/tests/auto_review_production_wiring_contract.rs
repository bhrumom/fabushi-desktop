const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_host_binds_one_auto_review_controller_per_routed_runner_turn() {
    assert!(SHIPPING_HOST.contains("auto_review.bind_runner(&agent_id, approvals_resolvable)"));
    assert!(SHIPPING_HOST.contains("auto_review_service.unbind_runner("));
    assert!(SHIPPING_HOST.contains("spawn_error_auto_review.unbind_runner("));
}

#[test]
fn shipping_cloud_agent_tool_uses_live_auto_review_hook() {
    assert!(SHIPPING_HOST.contains("build_cloud_agent_auto_review_hook("));
    assert!(SHIPPING_HOST.contains("review: Some(cloud_agent_review)"));
    assert!(SHIPPING_HOST.contains("create_sand_backend_smart_mode_classifier_executor("));
    assert!(SHIPPING_HOST.contains("review_sand_cloud_agent_action("));
    assert!(SHIPPING_HOST.contains("review_sand_cloud_agent_lifecycle_action("));
}

#[test]
fn background_turns_do_not_expose_resolvable_approval_cards() {
    assert!(SHIPPING_HOST.contains("Some(\"group-member\" | \"automation\")"));
    assert!(SHIPPING_HOST.contains("let approvals_resolvable = !matches!("));
}
