use mahayana_host_runtime::extensions::node_host::gateway_connection::{
    can_reuse_node_host_device_token, gateway_origin_scope, NodeHostGatewayConfig,
};
use mahayana_host_runtime::extensions::node_host::command_dispatch::{
    dispatch_node_worker_computer_command, ComputerCommandInput, ComputerOperation,
    COMPUTER_CONTRACT_MISMATCH, COMPUTER_DRIVER_UNAVAILABLE,
};
use mahayana_host_runtime::extensions::node_host::runtime_manifest::{
    NodeHostManifest, NODE_AGENT_CLI_CLAUDE_RUN_COMMAND, NODE_DESKTOP_STREAM_COMMAND,
    NODE_SYSTEM_RUN_COMMANDS,
};
use std::collections::HashSet;

#[test]
fn test_ocrm_node_host_gateway_auth_and_scope() {
    let cfg = NodeHostGatewayConfig::new("127.0.0.1", 8080, false);
    assert_eq!(cfg.to_url(), "http://127.0.0.1:8080");

    assert_eq!(
        gateway_origin_scope("http://localhost:8080/api/v1"),
        "http://localhost:8080"
    );
    assert_eq!(
        gateway_origin_scope("https://gateway.example.com:443/ws"),
        "https://gateway.example.com:443"
    );

    // Matching scope with token
    let saved = "http://127.0.0.1:8080";
    let candidates = ["http://127.0.0.1:8080/ws", "http://127.0.0.1:8080/"];
    assert!(can_reuse_node_host_device_token(
        Some(saved),
        &candidates,
        Some("node-tok-1234")
    ));

    // Mismatched scope
    let mismatched = ["http://127.0.0.1:8080", "http://192.168.1.1:8080"];
    assert!(!can_reuse_node_host_device_token(
        Some(saved),
        &mismatched,
        Some("node-tok-1234")
    ));

    // Missing token
    assert!(!can_reuse_node_host_device_token(
        Some(saved),
        &candidates,
        None
    ));
}

#[test]
fn test_ocrm_node_host_computer_command() {
    // Close operation succeeds even if driver unavailable
    let close_input = ComputerCommandInput {
        operation: ComputerOperation::Close,
        provider_generation: None,
        action_params_json: None,
    };
    assert!(dispatch_node_worker_computer_command(&close_input, None).is_ok());

    // Capabilities operation requires driver
    let caps_input = ComputerCommandInput {
        operation: ComputerOperation::Capabilities,
        provider_generation: None,
        action_params_json: None,
    };
    assert_eq!(
        dispatch_node_worker_computer_command(&caps_input, None),
        Err(COMPUTER_DRIVER_UNAVAILABLE)
    );
    assert!(dispatch_node_worker_computer_command(&caps_input, Some(42)).is_ok());

    // Act operation with matching generation
    let act_input = ComputerCommandInput {
        operation: ComputerOperation::Act,
        provider_generation: Some(42),
        action_params_json: Some(r#"{"action":"mouse_click"}"#.to_string()),
    };
    assert!(dispatch_node_worker_computer_command(&act_input, Some(42)).is_ok());

    // Act operation with mismatched generation
    let act_mismatch = ComputerCommandInput {
        operation: ComputerOperation::Act,
        provider_generation: Some(99),
        action_params_json: None,
    };
    assert_eq!(
        dispatch_node_worker_computer_command(&act_mismatch, Some(42)),
        Err(COMPUTER_CONTRACT_MISMATCH)
    );
}

#[test]
fn test_ocrm_node_host_runtime_manifest() {
    let manifest = NodeHostManifest::build(true, true, None);
    assert!(manifest.advertised_commands.iter().any(|c| c == NODE_AGENT_CLI_CLAUDE_RUN_COMMAND));
    assert!(manifest.advertised_commands.iter().any(|c| c == NODE_DESKTOP_STREAM_COMMAND));
    assert!(manifest.advertised_commands.iter().any(|c| c == NODE_SYSTEM_RUN_COMMANDS));

    // With allowlist filter
    let mut allowlist = HashSet::new();
    allowlist.insert(NODE_SYSTEM_RUN_COMMANDS.to_string());
    let filtered_manifest = NodeHostManifest::build(true, true, Some(&allowlist));
    assert_eq!(filtered_manifest.advertised_commands, vec![NODE_SYSTEM_RUN_COMMANDS.to_string()]);
}
