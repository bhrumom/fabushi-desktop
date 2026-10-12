const SHIPPING_HOST_MAIN: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_host_consumes_external_machine_through_runner_bridge() {
    for needle in [
        "ProductionExternalMachineExecutor::for_turn(",
        "external_machine_executor: Some(external_machine_executor)",
        "external_shell_review: Some(external_shell_review)",
        "ShellApprovalSurface::HostShell",
        "current_modes().host_shell",
    ] {
        assert!(
            SHIPPING_HOST_MAIN.contains(needle),
            "shipping Host is missing external-machine production binding: {needle}"
        );
    }
}
