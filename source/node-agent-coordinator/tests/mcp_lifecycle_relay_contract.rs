use mahayana_node_agent_coordinator::carrier::CarrierChannel;
use mahayana_node_agent_coordinator::mcp_lifecycle_relay::{
    HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_INSTALL_PLUGIN_METHOD,
    MCP_LIST_CATALOG_METHOD, MCP_LIST_EFFECTIVE_PLUGINS_METHOD, MCP_LOAD_PLUGIN_SKILLS_METHOD,
    MCP_LIST_SERVERS_METHOD,
    MCP_ADD_SERVER_METHOD, MCP_REMOVE_SERVER_METHOD, MCP_RESTART_SERVERS_METHOD,
    MCP_SET_INSTRUCTIONS_METHOD, MCP_AUTHENTICATE_SERVER_METHOD, MCP_LOGOUT_ACCOUNT_METHOD,
    MCP_REMOVE_ACCOUNT_METHOD, MCP_RENAME_ACCOUNT_METHOD, MCP_NOTE_AUTH_COMPLETED_METHOD,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD, MCP_UNINSTALL_PLUGIN_METHOD,
    box_mcp_carrier_allowed, execute_host_mcp_lifecycle_request,
    parse_host_mcp_lifecycle_request,
};
use mahayana_node_agent_coordinator::protocol::Failure;
use serde_json::json;

#[test]
fn box_mcp_raw_commands_require_the_native_main_data_carrier() {
    for method in [
        "loadBoxMcpServers",
        "listBoxMcpToolsRaw",
        "executeBoxMcpToolRaw",
        MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
    ] {
        assert!(!box_mcp_carrier_allowed(method, CarrierChannel::Data), "{method}");
        assert!(!box_mcp_carrier_allowed(method, CarrierChannel::Control), "{method}");
        assert!(box_mcp_carrier_allowed(method, CarrierChannel::MainData), "{method}");
    }
}

#[test]
fn box_mcp_carrier_gate_keeps_exact_names_and_existing_command_boundaries() {
    for method in [
        "listBoxMcpServers",
        "listAgents",
        "echo",
        MCP_LIST_SERVERS_METHOD,
        MCP_INSTALL_PLUGIN_METHOD,
        "loadBoxMcpServer",
        "loadBoxMcpServers.extra",
        "LoadBoxMcpServers",
        "loadBoxMcpServers ",
        "listBoxMcpTools",
        "listBoxMcpToolsRawExtra",
        "executeBoxMcpTool",
        "executeBoxMcpToolRawExtra",
        "mcp.executeBoxMcpToolRaw",
        "",
    ] {
        for channel in [CarrierChannel::Control, CarrierChannel::Data, CarrierChannel::MainData] {
            assert!(box_mcp_carrier_allowed(method, channel), "{method} on {channel:?}");
        }
    }

    for method in [
        "loadBoxMcpServers",
        "listBoxMcpToolsRaw",
        "executeBoxMcpToolRaw",
        "loadBoxMcpServers.extra",
        "listBoxMcpToolsRawExtra",
        "executeBoxMcpToolRawExtra",
    ] {
        let denied = parse_host_mcp_lifecycle_request(&json!({
            "requestId": "box-is-not-a-desktop-lifecycle-command",
            "method": method,
            "args": {},
        }))
        .expect_err("the native Box gate must not expand the desktop lifecycle relay");
        assert_eq!(denied.code, "MCP_LIFECYCLE_RELAY_METHOD_DENIED");
    }
}

#[test]
fn host_mcp_lifecycle_relay_accepts_only_the_frozen_owner_commands() {
    assert_eq!(HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, "host-mcp-lifecycle-request");
    assert_eq!(MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD, "mcp.resolveLifecycleRequest");
    for method in [
        MCP_LIST_SERVERS_METHOD,
        MCP_LIST_CATALOG_METHOD,
        MCP_LIST_EFFECTIVE_PLUGINS_METHOD,
        MCP_LOAD_PLUGIN_SKILLS_METHOD,
        MCP_INSTALL_PLUGIN_METHOD,
        MCP_UNINSTALL_PLUGIN_METHOD,
        MCP_ADD_SERVER_METHOD,
        MCP_REMOVE_SERVER_METHOD,
        MCP_RESTART_SERVERS_METHOD,
        MCP_SET_INSTRUCTIONS_METHOD,
        MCP_AUTHENTICATE_SERVER_METHOD,
        MCP_LOGOUT_ACCOUNT_METHOD,
        MCP_REMOVE_ACCOUNT_METHOD,
        MCP_RENAME_ACCOUNT_METHOD,
        MCP_NOTE_AUTH_COMPLETED_METHOD,
    ] {
        let parsed = parse_host_mcp_lifecycle_request(&json!({
            "requestId": format!("mcp-{method}"),
            "method": method,
            "args": {},
        }))
        .expect("approved MCP lifecycle method");
        assert_eq!(parsed.method, method);
    }
    let denied = parse_host_mcp_lifecycle_request(&json!({
        "requestId": "mcp-denied",
        "method": "spawnLocalExecDaemon",
        "args": {},
    }))
    .expect_err("unrelated Coordinator command must be denied");
    assert_eq!(denied.code, "MCP_LIFECYCLE_RELAY_METHOD_DENIED");
}

#[test]
fn host_mcp_lifecycle_relay_preserves_request_id_and_remote_failure() {
    let success = execute_host_mcp_lifecycle_request(
        &json!({
            "requestId": "host-mcp-7",
            "method": MCP_LIST_CATALOG_METHOD,
            "args": {"forceRefresh": true},
        }),
        |method, args| {
            assert_eq!(method, MCP_LIST_CATALOG_METHOD);
            assert_eq!(args["forceRefresh"], true);
            Ok(json!([{"id":"7"}]))
        },
    )
    .expect("valid lifecycle request");
    assert_eq!(success["requestId"], "host-mcp-7");
    assert_eq!(success["ok"], true);
    assert_eq!(success["result"][0]["id"], "7");

    let failed = execute_host_mcp_lifecycle_request(
        &json!({
            "requestId": "host-mcp-8",
            "method": MCP_UNINSTALL_PLUGIN_METHOD,
            "args": {"pluginId":"8"},
        }),
        |_method, _args| Err(Failure::new("MCP_OWNER_OFFLINE", "desktop owner unavailable")),
    )
    .expect("valid lifecycle request with failed owner");
    assert_eq!(failed["requestId"], "host-mcp-8");
    assert_eq!(failed["ok"], false);
    assert_eq!(failed["error"], "MCP_OWNER_OFFLINE: desktop owner unavailable");
}

#[test]
fn host_mcp_lifecycle_relay_echoes_instance_qualified_ids_on_success_and_failure() {
    for request_id in [
        "host-mcp-5f94563b-4e9d-44cf-8cb7-b6b8b789e441-1",
        "host-mcp-5f94563b-4e9d-44cf-8cb7-b6b8b789e441-18446744073709551615",
    ] {
        let payload = json!({
            "requestId": request_id,
            "method": MCP_LOAD_PLUGIN_SKILLS_METHOD,
            "args": {},
        });
        let parsed = parse_host_mcp_lifecycle_request(&payload).expect("opaque instance ID parses");
        assert_eq!(parsed.request_id, request_id);
        assert_eq!(parsed.method, MCP_LOAD_PLUGIN_SKILLS_METHOD);

        let success = execute_host_mcp_lifecycle_request(&payload, |method, args| {
            assert_eq!(method, MCP_LOAD_PLUGIN_SKILLS_METHOD);
            assert_eq!(args, json!({}));
            Ok(json!({"plugins": [], "currentUserId": 11}))
        })
        .expect("instance-qualified request executes");
        assert_eq!(
            success,
            json!({
                "requestId": request_id,
                "ok": true,
                "result": {"plugins": [], "currentUserId": 11},
            })
        );

        let failure = execute_host_mcp_lifecycle_request(&payload, |method, args| {
            assert_eq!(method, MCP_LOAD_PLUGIN_SKILLS_METHOD);
            assert_eq!(args, json!({}));
            Err(Failure::new("MCP_OWNER_OFFLINE", "desktop owner unavailable"))
        })
        .expect("failed execution still returns the opaque request ID");
        assert_eq!(
            failure,
            json!({
                "requestId": request_id,
                "ok": false,
                "error": "MCP_OWNER_OFFLINE: desktop owner unavailable",
            })
        );
    }
}

#[test]
fn host_mcp_lifecycle_request_id_limit_remains_256_bytes() {
    for request_id in ["x".repeat(256), "é".repeat(128)] {
        let payload = json!({
            "requestId": request_id,
            "method": MCP_LIST_CATALOG_METHOD,
            "args": {},
        });
        let response = execute_host_mcp_lifecycle_request(&payload, |_, _| Ok(json!([])))
            .expect("opaque ID at the byte limit remains accepted");
        assert_eq!(response["requestId"], request_id);
        assert_eq!(response["ok"], true);
    }

    for request_id in ["x".repeat(257), format!("{}x", "é".repeat(128))] {
        let mut executed = false;
        let denied = execute_host_mcp_lifecycle_request(
            &json!({
                "requestId": request_id,
                "method": MCP_LIST_CATALOG_METHOD,
                "args": {},
            }),
            |_, _| {
                executed = true;
                Ok(json!([]))
            },
        )
        .expect_err("oversized ID must fail before reaching the executor");
        assert_eq!(denied.code, "MCP_LIFECYCLE_RELAY_PROTOCOL_ERROR");
        assert_eq!(
            denied.message,
            "Host MCP lifecycle requestId exceeds 256 bytes"
        );
        assert!(!executed);
    }
}
