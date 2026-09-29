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
