use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
    CoordinatorMcpLifecycleRelay, CoordinatorMcpLifecycleRelayError,
    HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_LIST_CATALOG_METHOD,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
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
