use mahayana_host_runtime::gateway_protocol::{
    FABUSHI_GATEWAY_COMPAT_COMMANDS, GATEWAY_PREPARE_UPGRADE_PATH, GROK_GATEWAY_COMMANDS,
    OPENCLAW_GATEWAY_METHODS, is_grok_gateway_command, is_openclaw_gateway_command,
    parse_command_args, slim_command_result, slim_event,
};
use serde_json::json;
use mahayana_host_runtime::host_gateway_api::{
    FABUSHI_NATIVE_ATTACHMENTS_GATEWAY_METHODS, FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS,
    FABUSHI_NATIVE_CLOUD_WORKERS_GATEWAY_METHODS,
    FABUSHI_NATIVE_MINIAPP_GATEWAY_METHODS, HostGatewayOwner, host_gateway_owner,
};

#[test]
fn native_link_preview_registration_keeps_inventory_and_owner() {
    assert_eq!(FABUSHI_NATIVE_ATTACHMENTS_GATEWAY_METHODS, &["fetchLinkMetadata"]);
    assert_eq!(host_gateway_owner("fetchLinkMetadata"), Some(HostGatewayOwner::Attachments));
    assert!(is_grok_gateway_command("fetchLinkMetadata"));
    assert!(!GROK_GATEWAY_COMMANDS.contains(&"fetchLinkMetadata"));
    assert!(!is_openclaw_gateway_command("fetchLinkMetadata"));
    assert!(!FABUSHI_GATEWAY_COMPAT_COMMANDS.contains(&"fetchLinkMetadata"));
    for unknown in [
        "fetchLinkMetadata.extra", "FetchLinkMetadata", "attachments.fetchLinkMetadata",
        "readRemoteMediaBuffer", "saveRemoteMedia",
    ] {
        assert_eq!(host_gateway_owner(unknown), None, "{unknown}");
        assert!(!is_grok_gateway_command(unknown), "{unknown}");
    }
    for existing in ["uploadAttachment", "readAttachmentImage", "readAttachmentText", "readAttachmentChunk"] {
        assert_eq!(host_gateway_owner(existing), Some(HostGatewayOwner::Attachments));
        assert!(GROK_GATEWAY_COMMANDS.contains(&existing));
    }
}

#[test]
fn native_box_mcp_extensions_are_exact_and_resolve_only_to_mcp() {
    assert_eq!(
        FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS,
        &["loadBoxMcpServers", "listBoxMcpToolsRaw", "executeBoxMcpToolRaw"]
    );
    for method in FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS {
        assert!(!GROK_GATEWAY_COMMANDS.contains(method));
        assert!(!FABUSHI_NATIVE_MINIAPP_GATEWAY_METHODS.contains(method));
        assert!(!FABUSHI_NATIVE_CLOUD_WORKERS_GATEWAY_METHODS.contains(method));
        assert!(!FABUSHI_GATEWAY_COMPAT_COMMANDS.contains(method));
        assert!(!is_openclaw_gateway_command(method));
        assert!(is_grok_gateway_command(method));
        assert_eq!(host_gateway_owner(method), Some(HostGatewayOwner::Mcp));
    }

    // This existing frozen method is not one of the three repaired routes.
    assert!(GROK_GATEWAY_COMMANDS.contains(&"listBoxMcpServers"));
    assert_eq!(host_gateway_owner("listBoxMcpServers"), Some(HostGatewayOwner::Mcp));
    for unknown in [
        "LoadBoxMcpServers",
        "loadBoxMcpServers.extra",
        "listBoxMcpTools",
        "listBoxMcpToolsRawExtra",
        "executeBoxMcpTool",
        "executeBoxMcpToolRawExtra",
        "mcp.executeBoxMcpToolRaw",
    ] {
        assert_eq!(host_gateway_owner(unknown), None, "{unknown}");
        assert!(!is_grok_gateway_command(unknown), "{unknown}");
        assert!(!is_openclaw_gateway_command(unknown), "{unknown}");
    }
}

#[test]
fn native_cloud_worker_projections_preserve_exact_routes_outside_frozen_inventory() {
    assert_eq!(
        FABUSHI_NATIVE_CLOUD_WORKERS_GATEWAY_METHODS,
        &["cloudWorkers.status", "cloudWorkers.capabilities", "cloudWorkers.doctor"]
    );
    for method in FABUSHI_NATIVE_CLOUD_WORKERS_GATEWAY_METHODS {
        assert!(!GROK_GATEWAY_COMMANDS.contains(method));
        assert!(!FABUSHI_NATIVE_MINIAPP_GATEWAY_METHODS.contains(method));
        assert!(!FABUSHI_GATEWAY_COMPAT_COMMANDS.contains(method));
        assert!(!is_openclaw_gateway_command(method));
        assert!(is_grok_gateway_command(method));
        assert_eq!(host_gateway_owner(method), Some(HostGatewayOwner::CloudWorkers));
    }
}

#[test]
fn native_mini_app_extensions_preserve_frozen_inventory_and_mcp_owner() {
    assert_eq!(
        FABUSHI_NATIVE_MINIAPP_GATEWAY_METHODS,
        &["listApprovedMiniApps", "listInstalledMiniApps", "installApprovedMiniApp",
            "getInstalledMiniAppResources", "uninstallMiniApp"]
    );
    for method in FABUSHI_NATIVE_MINIAPP_GATEWAY_METHODS {
        assert!(!GROK_GATEWAY_COMMANDS.contains(method));
        assert!(is_grok_gateway_command(method));
        assert_eq!(host_gateway_owner(method), Some(HostGatewayOwner::Mcp));
    }
}

#[test]
fn openclaw_gateway_extensions_preserve_frozen_inventory_and_owners() {
    assert_eq!(OPENCLAW_GATEWAY_METHODS.len(), 26);
    assert_eq!(
        OPENCLAW_GATEWAY_METHODS.iter().filter(|method| **method == "device.token.revoke").count(),
        1,
    );
    assert_eq!(host_gateway_owner("device.token.revoke"), Some(HostGatewayOwner::DevicePairing));
    for unknown in ["device.Token.revoke", "device.token.revoke.extra", "device.token.Revoke"] {
        assert!(!is_openclaw_gateway_command(unknown));
        assert!(!is_grok_gateway_command(unknown));
        assert_eq!(host_gateway_owner(unknown), None);
    }
    assert!(OPENCLAW_GATEWAY_METHODS.contains(&"sessions.patch"));
    assert!(OPENCLAW_GATEWAY_METHODS.contains(&"sessions.providerReview.continue"));
    assert_eq!(
        host_gateway_owner("sessions.patch"),
        Some(HostGatewayOwner::TranscriptManager)
    );
    assert_eq!(
        host_gateway_owner("sessions.providerReview.continue"),
        Some(HostGatewayOwner::TranscriptManager)
    );
    for method in OPENCLAW_GATEWAY_METHODS {
        assert!(!GROK_GATEWAY_COMMANDS.contains(method));
        assert!(is_openclaw_gateway_command(method));
        assert!(is_grok_gateway_command(method));
        assert!(host_gateway_owner(method).is_some());
    }
}

#[test]
fn gateway_protocol_parses_empty_and_json_command_args() {
    assert_eq!(parse_command_args(b"").expect("empty body"), json!({}));
    assert_eq!(
        parse_command_args(br#"{"agentId":"agent-1","prompt":"hello"}"#)
            .expect("json body"),
        json!({"agentId":"agent-1","prompt":"hello"})
    );
    assert!(parse_command_args(b"{").is_err());
}

#[test]
fn gateway_protocol_keeps_frozen_command_inventory_and_upgrade_path() {
    assert_eq!(GROK_GATEWAY_COMMANDS.len(), 122);
    assert!(GROK_GATEWAY_COMMANDS.contains(&"listRoutedMcpTools"));
    assert!(GROK_GATEWAY_COMMANDS.contains(&"executeRoutedMcpTool"));
    assert_eq!(FABUSHI_GATEWAY_COMPAT_COMMANDS, &["resumeAfterRecreate"]);
    assert!(is_grok_gateway_command("sendPrompt"));
    assert!(is_grok_gateway_command("requestWebAuthnCeremony"));
    assert!(is_grok_gateway_command("resumeAfterRecreate"));
    assert!(is_grok_gateway_command("listRoutedMcpTools"));
    assert!(is_grok_gateway_command("executeRoutedMcpTool"));
    assert!(!is_grok_gateway_command("feature.auth.status"));
    assert!(!is_grok_gateway_command("not-a-grok-command"));
    assert_eq!(GATEWAY_PREPARE_UPGRADE_PATH, "/prepare-upgrade");
}

#[test]
fn gateway_protocol_strips_inline_avatars_from_slim_commands() {
    let listed = slim_command_result(
        "listAgents",
        json!([
            {"id":"one","avatarDataUrl":"data:image/png;base64,one"},
            {"id":"two","avatarDataUrl":null}
        ]),
    );
    assert_eq!(listed[0]["avatarDataUrl"], json!(null));
    assert_eq!(listed[1]["avatarDataUrl"], json!(null));

    let created = slim_command_result(
        "createAgent",
        json!({
            "agent":{"id":"one","avatarDataUrl":"data:image/png;base64,one"},
            "other":"preserved"
        }),
    );
    assert_eq!(created["agent"]["avatarDataUrl"], json!(null));
    assert_eq!(created["other"], "preserved");

    let untouched = slim_command_result(
        "getTranscript",
        json!({"avatarDataUrl":"keep-on-non-summary-result"}),
    );
    assert_eq!(untouched["avatarDataUrl"], "keep-on-non-summary-result");
}

#[test]
fn gateway_protocol_strips_inline_avatars_from_agent_events_only() {
    let agents = slim_event(json!({
        "channel":"agents",
        "payload":{
            "agents":[
                {"id":"one","avatarDataUrl":"data:image/png;base64,one"}
            ],
            "activeAgentId":"one"
        }
    }));
    assert_eq!(agents["payload"]["agents"][0]["avatarDataUrl"], json!(null));
    assert_eq!(agents["payload"]["activeAgentId"], "one");

    let upserted = slim_event(json!({
        "channel":"agent-upserted",
        "payload":{
            "agent":{"id":"one","avatarDataUrl":"data:image/png;base64,one"}
        }
    }));
    assert_eq!(upserted["payload"]["agent"]["avatarDataUrl"], json!(null));

    let other = slim_event(json!({
        "channel":"transcript",
        "payload":{"avatarDataUrl":"preserve"}
    }));
    assert_eq!(other["payload"]["avatarDataUrl"], "preserve");
}
