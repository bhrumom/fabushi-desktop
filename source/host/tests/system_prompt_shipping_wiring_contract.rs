const SHIPPING_BOX_RUNNER: &str = include_str!("../src/extensions/forever_box/runner_tools.rs");
const HOST_RUNNER_COMPOSITION: &str = include_str!("../src/host_runner_composition.rs");
const SHIPPING_HOST_MAIN: &str = include_str!("../app/src/main.rs");

#[test]
fn shipping_provider_consumes_live_box_and_computer_state_in_frozen_prompt_order() {
    let mcp = SHIPPING_HOST_MAIN
        .find("append_mcp_system_prompt_sections(")
        .expect("shipping MCP prompt binding");
    let remote = SHIPPING_HOST_MAIN
        .find("append_remote_box_system_prompt(")
        .expect("shipping remote-box prompt binding");
    let computer = SHIPPING_HOST_MAIN
        .find("append_computer_system_prompt(")
        .expect("shipping Computer prompt binding");
    assert!(mcp < remote && remote < computer);

    for production_binding in [
        "forever_box.get_status(&agent_id)",
        "forever_box.box_().is_available()",
        "forever_box.box_().inner().shared_desktop().is_some()",
        "forever_box.box_().get_agent_window_index(&agent_id)",
        "session_handoff.get(&agent_id).is_some()",
        ".with_availability_check(Arc::new(move |args|",
        "ComputerToolExposure::ScreenshotOnly",
        "ComputerToolExposure::Full",
        "ComputerToolExposure::Disabled",
    ] {
        assert!(
            SHIPPING_HOST_MAIN.contains(production_binding),
            "missing shipping production binding: {production_binding}"
        );
    }
}


#[test]
fn shipping_remote_box_and_computer_owners_are_consumed_not_test_only_helpers() {
    for binding in [
        "RemoteBoxResourceCoordinator",
        ".connect(box_preparing",
        ".computer_use_plan(&connection",
        ".clear_connection()",
    ] {
        assert!(
            SHIPPING_BOX_RUNNER.contains(binding),
            "shipping ForeverBox Runner port bypasses remote-box owner: {binding}"
        );
    }

    assert!(HOST_RUNNER_COMPOSITION.contains("computer_use: Arc<Mutex<ComputerUseCoordination>>"));
    assert!(HOST_RUNNER_COMPOSITION.contains("pub fn computer_use_coordination(&self)"));

    for binding in [
        "let shipping_box_resources = Arc::new(ForeverBoxRunnerResourcePort::new(",
        "let computer_use_owner = host_runner_composition.computer_use_coordination();",
        "owner.allocate_window(&agent_id)",
        "owner.begin_preparation(&agent_id)",
        "box-chrome --sand-prepare",
        "owner.record_turn_ended(usage)",
        "owner.free_window(&agent_id)",
        "shipping_box_resources.clone()",
    ] {
        assert!(
            SHIPPING_HOST_MAIN.contains(binding),
            "shipping Host does not consume remote-box/computer owner: {binding}"
        );
    }
}
