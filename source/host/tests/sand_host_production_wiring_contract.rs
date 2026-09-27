use mahayana_host_runtime::host_gateway_api::{
    CREATE_AGENT_NONCE_LEDGER_CAP, CreateAgentNonceLedger,
};
use mahayana_host_runtime::sand_host::compute_host_health;
use serde_json::json;

const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const TRANSCRIPT_RUNTIME: &str =
    include_str!("../src/extensions/transcript/production_runtime.rs");

#[test]
fn shipping_gateway_uses_the_grok_host_health_owner() {
    assert!(SHIPPING_HOST.contains("fn health(&self) -> GatewayHealth"));
    assert!(SHIPPING_HOST.contains("compute_host_health("));
    assert!(SHIPPING_HOST.contains("live_running_agent_ids()"));
    assert!(SHIPPING_HOST.contains("has_carryable_pending_wake()"));
    assert!(SHIPPING_HOST.contains("agent_ids_with_pending_approvals()"));
    assert!(SHIPPING_HOST.contains("active_agent_id(&self.session_workers)"));
    assert!(TRANSCRIPT_RUNTIME.contains("pub fn live_running_agent_ids"));
    assert!(TRANSCRIPT_RUNTIME.contains("pub fn has_carryable_pending_wake"));
}

#[test]
fn health_keeps_approval_only_waits_from_refreshing_busy_clock() {
    let health = compute_host_health(
        ["agent-a".to_string()],
        ["agent-a".to_string()],
        false,
        Some("agent-a".into()),
        900,
        500,
    );
    assert!(health.is_busy);
    assert!(health.busy_only_awaiting_approval);
    assert_eq!(health.last_busy_at_ms, 500);

    let active = compute_host_health(
        ["agent-a".to_string()],
        Vec::<String>::new(),
        false,
        Some("agent-a".into()),
        900,
        500,
    );
    assert!(active.is_busy);
    assert!(!active.busy_only_awaiting_approval);
    assert_eq!(active.last_busy_at_ms, 900);
}

#[test]
fn shipping_create_agent_path_consumes_nonce_and_input_policy() {
    assert!(SHIPPING_HOST.contains(r#"method == "createAgent""#));
    assert!(SHIPPING_HOST.contains("is_sand_agent_purpose"));
    assert!(SHIPPING_HOST.contains("sanitize_template_id"));
    assert!(SHIPPING_HOST.contains("create_agent_nonces"));

    let mut ledger = CreateAgentNonceLedger::default();
    for index in 0..(CREATE_AGENT_NONCE_LEDGER_CAP + 3) {
        ledger.insert(format!("nonce-{index}"), json!({ "id": index }));
    }
    assert_eq!(ledger.len(), CREATE_AGENT_NONCE_LEDGER_CAP);
    assert!(ledger.get("nonce-0").is_none());
    assert_eq!(
        ledger
            .get(&format!("nonce-{}", CREATE_AGENT_NONCE_LEDGER_CAP + 2))
            .and_then(|value| value.get("id"))
            .and_then(serde_json::Value::as_u64),
        Some((CREATE_AGENT_NONCE_LEDGER_CAP + 2) as u64),
    );
}
