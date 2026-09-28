const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn cloud_agent_watch_preserves_explicit_and_automation_quiet_origin() {
    assert!(SHIPPING_HOST.contains("args.get(\"quietOrigin\")"));
    assert!(SHIPPING_HOST.contains("args.get(\"automationWake\")"));
    assert!(SHIPPING_HOST.contains("\"automation\": {"));
    assert!(SHIPPING_HOST.contains("CloudAgentWatchOptions::new("));
}

#[test]
fn silence_allowed_turns_carry_the_generic_quiet_origin_marker() {
    assert!(SHIPPING_HOST.contains("args.get(\"isSilenceAllowed\")"));
    assert!(SHIPPING_HOST.contains(".filter(|allowed| *allowed)"));
    assert!(SHIPPING_HOST.contains(".map(|_| serde_json::json!({}))"));
}
