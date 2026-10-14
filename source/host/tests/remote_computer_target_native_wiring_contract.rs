const TARGET: &str = include_str!("../src/remote_computer_target.rs");
const MAIN: &str = include_str!("../app/src/main.rs");
const CARGO: &str = include_str!("../Cargo.toml");

#[test]
fn target_reuses_product_native_computer_owner_and_remote_policy() {
    for required in [
        "pub struct NativeDesktopRemoteToolCatalog",
        "pub struct NativeDesktopRemoteToolAuthorizer",
        "pub struct NativeDesktopRemoteExecutor",
        "mahayana_computer::control_policy().remote_control_enabled",
        "mahayana_computer::ComputerControlLeaseRequest::new(",
        "ComputerControlOrigin::RemoteMobile",
        "mahayana_computer::execute_with_lease(",
        "REMOTE_NATIVE_TOOL_GRANT_MAX_SECONDS",
        "sensitive_result_channel_unavailable",
        "\"screenshotTransferred\": false",
    ] {
        assert!(TARGET.contains(required), "missing native target invariant: {required}");
    }
    assert!(TARGET.contains("tool_name != \"Computer\""));
    assert!(!TARGET.contains("AllowAllRemoteComputerToolAuthorizer"));
    assert!(CARGO.contains("mahayana-computer = { path = \"../mahayana/mahayana-rs/mahayana-computer\" }"));
}

#[test]
fn shipping_host_attaches_target_service_to_same_network_owner() {
    for required in [
        "production_remote_computer_target_service(",
        "NetworkEnrollmentSupervisor::start_with_service(owner, target_service)",
        "FABUSHI_API_BASE_URL_ENV",
        "DEFAULT_FABUSHI_API_BASE_URL",
    ] {
        assert!(MAIN.contains(required), "missing shipping target wiring: {required}");
    }
}
