const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

fn compact_source(source: &str) -> String {
    source.chars().filter(|value| !value.is_whitespace()).collect()
}

#[test]
fn cloud_agent_watch_preserves_explicit_and_automation_quiet_origin() {
    let compact = compact_source(SHIPPING_HOST);
    assert!(compact.contains("args.get(\"quietOrigin\")"));
    assert!(compact.contains("args.get(\"automationWake\")"));
    assert!(compact.contains("\"automation\":{"));
    assert!(compact.contains("CloudAgentWatchOptions::new("));
}

#[test]
fn silence_allowed_turns_carry_the_generic_quiet_origin_marker() {
    let compact = compact_source(SHIPPING_HOST);
    assert!(compact.contains("args.get(\"isSilenceAllowed\")"));
    assert!(compact.contains(".filter(|allowed|*allowed)"));
    assert!(compact.contains(".map(|_|serde_json::json!({}))"));
}
