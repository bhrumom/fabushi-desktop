use std::fs;
use std::path::PathBuf;

fn shipping_main_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app/src/main.rs");
    fs::read_to_string(path).expect("read shipping Host main.rs")
}

#[test]
fn shipping_provider_turn_consumes_dynamic_sections_in_frozen_order() {
    let source = shipping_main_source();
    let provider_start = source.find("let mut provider_messages =").expect("shipping provider projection");
    let tail = &source[provider_start..];

    let workflows = tail.find("append_workflows_system_prompt(").expect("shipping workflows section");
    let channels = tail.find("append_channels_system_prompt(").expect("shipping channels section");
    let directory = tail.find("append_agent_directory_system_prompt(").expect("shipping agent directory section");
    let mcp = tail.find("append_mcp_system_prompt_sections(").expect("shipping MCP sections");

    assert!(workflows < channels && channels < directory && directory < mcp);
    assert!(tail[..channels].contains("list_agent_channels(&agent_id)"));
    assert!(tail[..directory].contains("summary.member_ids") && tail[..directory].contains("summary.is_group"));
    assert!(tail[..mcp].contains("mcp_service.list_installed()"));
    assert!(tail[..mcp].contains("mcp_discovery_unavailable"));
    assert_eq!(source.matches("append_channels_system_prompt(").count(), 1);
    assert_eq!(source.matches("append_agent_directory_system_prompt(").count(), 1);
    assert_eq!(source.matches("append_mcp_system_prompt_sections(").count(), 1);
}
