use mahayana_host_runtime::host_gateway_api::{
    CREATE_AGENT_NONCE_LEDGER_CAP, CreateAgentNonceLedger, sanitize_create_agent_args,
};
use mahayana_host_runtime::sand_host::{
    BOX_READY_REPORT_ATTEMPTS, BOX_READY_REPORT_RETRY_MS, BOX_READY_STAGE_MARKER_PATH,
    box_ready_duration_ms, compute_host_health, should_report_box_ready,
};
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
    assert!(SHIPPING_HOST.contains("sanitize_create_agent_args"));
    assert!(SHIPPING_HOST.contains("create_agent_nonces"));

    let sanitized = sanitize_create_agent_args(&json!({
        "name": "Agent",
        "purpose": "not-sand",
        "templateId": "../escape"
    }));
    assert!(sanitized.get("purpose").is_none());
    assert!(sanitized.get("templateId").is_none());

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

#[test]
fn shipping_host_reports_box_ready_only_after_gateway_discovery() {
    let discovery = SHIPPING_HOST.find("write_gateway_discovery(").unwrap();
    let ready = SHIPPING_HOST
        .find("should_report_box_ready(")
        .expect("shipping Host must wire frozen box-ready report");
    assert!(discovery < ready);
    assert!(SHIPPING_HOST.contains("report_box_boot_stage_confirmed"));
    assert!(SHIPPING_HOST.contains("BOX_READY_REPORT_ATTEMPTS"));
    assert!(SHIPPING_HOST.contains("BOX_READY_REPORT_RETRY_MS"));
    assert!(SHIPPING_HOST.contains("BOX_READY_STAGE_MARKER_PATH"));

    assert_eq!(BOX_READY_REPORT_ATTEMPTS, 3);
    assert_eq!(BOX_READY_REPORT_RETRY_MS, 30_000);
    assert_eq!(BOX_READY_STAGE_MARKER_PATH, "/tmp/sand-box-ready-stage");
    assert!(should_report_box_ready(Some("boot-a"), Some(100), None));
    assert!(!should_report_box_ready(
        Some("boot-a"),
        Some(100),
        Some("boot-a"),
    ));
    assert_eq!(box_ready_duration_ms(250, 100), 150);
}

#[test]
fn shipping_host_copy_in_mode_precedes_long_lived_runtime_bootstrap() {
    let copy_in = SHIPPING_HOST
        .find("std::env::args().any(|arg| arg == BOX_COPY_IN_ARG)")
        .expect("shipping Host must recognize frozen --box-copy-in mode");
    let crash_guard = SHIPPING_HOST
        .find("install_process_crash_guard")
        .expect("shipping Host must install process crash guards");
    let host_lock = SHIPPING_HOST
        .find("let host_lock = match acquire_host_lock(")
        .expect("shipping Host must acquire the single-host lock");
    assert!(copy_in < crash_guard);
    assert!(copy_in < host_lock);
    assert!(SHIPPING_HOST.contains("execute_production_box_copy_in_from_env"));
    assert!(SHIPPING_HOST.contains("std::process::exit(exit_code)"));
}
