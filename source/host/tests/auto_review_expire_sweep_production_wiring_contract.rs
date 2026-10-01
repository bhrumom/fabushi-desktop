#[test]
fn shipping_startup_sweep_reports_failures_through_unique_host_structured_log_owner() {
    const HOST: &str = include_str!("../app/src/main.rs");
    const EXTENSION: &str = include_str!("../src/extensions/auto_review/extension.rs");
    const SERVICE: &str = include_str!("../src/extensions/auto_review/auto_review_service.rs");
    const OWNER: &str = include_str!("../src/extensions/telemetry/host_telemetry_service.rs");
    const COORDINATOR: &str = include_str!("../../node-agent-coordinator/src/main.rs");
    const ELECTRON: &str = include_str!("../../electron-main/telemetry/desktop-structured-log-telemetry.ts");

    let install = EXTENSION
        .find("service.set_expire_sweep_failed_sink(expire_sweep_failed)")
        .expect("expire-sweep sink install");
    let sweep = EXTENSION
        .find("service.sweep_stale_boot_state(now_ms())")
        .expect("startup sweep");
    assert!(install < sweep);

    for stage in [
        "list_agent_record_ids",
        "expire_pending_auto_review_approvals",
        "clear_awaiting_user_response",
    ] {
        assert!(SERVICE.contains(stage), "{stage}");
    }
    assert!(SERVICE.contains("self.report_expire_sweep_failed("));
    assert!(HOST.contains("start_auto_review_extension_with_expire_sweep_telemetry("));
    assert!(HOST.contains(".report_auto_review_expire_sweep_failed(stage, error_class)"));
    assert!(OWNER.contains("pub fn report_auto_review_expire_sweep_failed("));
    assert!(OWNER.contains("event: Some(\"sand.auto_review.expire_sweep_failed\")"));
    assert!(!COORDINATOR.contains("sand.auto_review.expire_sweep_failed"));
    assert!(!ELECTRON.contains("sand.auto_review.expire_sweep_failed"));
}
