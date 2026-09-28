use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
    CoordinatorMcpLifecycleRelay, CoordinatorMcpLifecycleRelayError,
    HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_LIST_CATALOG_METHOD,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD, decode_effective_plugins,
    decode_mcp_catalog, decode_mcp_server_state,
};
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use serde_json::json;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn host_mcp_lifecycle_relay_round_trips_through_gateway_event_correlation() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events.clone(),
        Duration::from_secs(2),
    ));
    let worker = Arc::clone(&relay);
    let handle = thread::spawn(move || {
        worker.request(MCP_LIST_CATALOG_METHOD, json!({"forceRefresh":true}))
    });

    let event = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("lifecycle request event");
    assert_eq!(event["channel"], HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL);
    assert_eq!(event["payload"]["method"], MCP_LIST_CATALOG_METHOD);
    assert_eq!(event["payload"]["args"]["forceRefresh"], true);
    let request_id = event["payload"]["requestId"]
        .as_str()
        .expect("request id")
        .to_string();

    let settlement = relay
        .resolve(&json!({
            "requestId": request_id,
            "ok": true,
            "result": [{"id":"plugin-1"}],
        }))
        .expect("settle request");
    assert_eq!(settlement["resolved"], true);
    assert_eq!(
        handle
            .join()
            .expect("relay worker")
            .expect("relay result")[0]["id"],
        "plugin-1"
    );
}

#[test]
fn host_mcp_lifecycle_relay_rejects_unapproved_methods_and_remote_failures() {
    let denied = CoordinatorMcpLifecycleRelay::with_timeout(
        GatewayEventHub::default(),
        Duration::from_millis(25),
    );
    assert!(matches!(
        denied.request("spawnLocalExecDaemon", json!({})),
        Err(CoordinatorMcpLifecycleRelayError::UnsupportedMethod(_))
    ));

    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events,
        Duration::from_secs(2),
    ));
    let worker = Arc::clone(&relay);
    let handle = thread::spawn(move || worker.request(MCP_LIST_CATALOG_METHOD, json!({})));
    let event = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("request event");
    let request_id = event["payload"]["requestId"]
        .as_str()
        .expect("request id")
        .to_string();
    relay
        .resolve(&json!({
            "requestId": request_id,
            "ok": false,
            "error": "owner offline",
        }))
        .expect("settle failed request");
    assert!(matches!(
        handle.join().expect("relay worker"),
        Err(CoordinatorMcpLifecycleRelayError::Remote { message, .. }) if message == "owner offline"
    ));
    assert_eq!(
        MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
        "mcp.resolveLifecycleRequest"
    );
}


#[test]
fn coordinator_owner_snapshots_decode_into_the_rust_mcp_service_contract() {
    let servers = decode_mcp_server_state(json!({
        "servers": [{
            "id":"server-1",
            "name":"Calendar",
            "serverIdentifier":"calendar",
            "accountKey":"default",
            "pluginId":"plugin-1",
            "isTeamServer":false,
            "status":"connected",
            "transport":"stdio",
            "toolCount":2,
            "disabledToolCount":1,
            "customInstructions":"be concise"
        }]
    }))
    .expect("server snapshot");
    assert_eq!(servers[0].server_identifier, "calendar");
    assert_eq!(servers[0].disabled_tool_count, Some(1));

    let catalog = decode_mcp_catalog(json!([{
        "id":"plugin-1",
        "name":"calendar",
        "displayName":"Calendar",
        "description":"calendar tools",
        "category":"productivity",
        "connectors":[{"name":"calendar"}],
        "fields":[{"key":"token","label":"Token","hint":"secret","isRequired":true,"isSecret":true}],
        "skills":[{"name":"Schedule","description":"schedule events","sourceUrl":"https://github.com/o/r/blob/main/skills/Schedule/SKILL.md"}]
    }]))
    .expect("catalog snapshot");
    assert_eq!(catalog[0].connector_count, 1);
    assert_eq!(catalog[0].fields[0].key, "token");
    assert_eq!(catalog[0].skills[0].name, "Schedule");

    let effective = decode_effective_plugins(json!([{
        "pluginId":"plugin-1",
        "installMode":"team-required",
        "isEnabled":true,
        "hasTeamConfiguredVariables":true
    }]))
    .expect("effective plugins snapshot");
    assert!(effective[0].is_enabled);
    assert!(effective[0].has_team_configured_variables);
}
