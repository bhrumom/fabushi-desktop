use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, CoordinatorToolRelayError, ROUTED_TOOL_EXECUTE_METHOD,
    ROUTED_TOOL_LIST_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};
use serde_json::json;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

#[test]
fn runner_tool_request_round_trips_through_gateway_event_and_resolution() {
    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(
        events,
        Duration::from_secs(1),
    ));
    let worker = {
        let relay = Arc::clone(&relay);
        thread::spawn(move || relay.request(ROUTED_TOOL_LIST_METHOD, json!({})))
    };

    let event = event_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("runner tool request event");
    assert_eq!(event["channel"], RUNNER_TOOL_REQUEST_EVENT_CHANNEL);
    assert_eq!(event["payload"]["method"], ROUTED_TOOL_LIST_METHOD);
    let request_id = event["payload"]["requestId"]
        .as_str()
        .expect("request id")
        .to_string();

    let ack = relay
        .resolve(&json!({
            "requestId": request_id,
            "ok": true,
            "result": [{"name":"github_search"}],
        }))
        .expect("resolve request");
    assert_eq!(ack["resolved"], true);
    assert_eq!(
        worker.join().expect("worker").expect("tool list"),
        json!([{"name":"github_search"}])
    );
    assert_eq!(relay.pending_count(), 0);
}

#[test]
fn runner_tool_request_propagates_remote_failure_and_rejects_unknown_methods() {
    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(
        events,
        Duration::from_secs(1),
    ));
    let worker = {
        let relay = Arc::clone(&relay);
        thread::spawn(move || {
            relay.request(
                ROUTED_TOOL_EXECUTE_METHOD,
                json!({"toolName":"create_issue","args":{"title":"x"}}),
            )
        })
    };

    let event = event_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("runner tool request event");
    let request_id = event["payload"]["requestId"]
        .as_str()
        .expect("request id")
        .to_string();
    relay
        .resolve(&json!({
            "requestId": request_id,
            "ok": false,
            "error": "connector unavailable",
        }))
        .expect("resolve failure");
    assert!(matches!(
        worker.join().expect("worker"),
        Err(CoordinatorToolRelayError::Remote { message, .. })
            if message == "connector unavailable"
    ));

    assert!(matches!(
        relay.request("deleteEverything", json!({})),
        Err(CoordinatorToolRelayError::UnsupportedMethod(method))
            if method == "deleteEverything"
    ));
}

#[test]
fn runner_tool_request_timeout_and_shutdown_clear_waiters() {
    let relay = CoordinatorToolRelay::with_timeout(
        GatewayEventHub::default(),
        Duration::from_millis(20),
    );
    assert!(matches!(
        relay.request(ROUTED_TOOL_LIST_METHOD, json!({})),
        Err(CoordinatorToolRelayError::Timeout(_))
    ));
    assert_eq!(relay.pending_count(), 0);

    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(
        events,
        Duration::from_secs(1),
    ));
    let worker = {
        let relay = Arc::clone(&relay);
        thread::spawn(move || relay.request(ROUTED_TOOL_LIST_METHOD, json!({})))
    };
    let event = event_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("pending request");
    relay.cancel_all("host shutdown");
    relay.cancel_all("duplicate shutdown");
    assert!(matches!(
        worker.join().expect("worker"),
        Err(CoordinatorToolRelayError::Closed(reason)) if reason == "host shutdown"
    ));
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(
        relay.resolve(&json!({"requestId":event["payload"]["requestId"],"ok":true,"result":[]})),
        Err(CoordinatorToolRelayError::Protocol(_))
    ));
    assert!(matches!(
        relay.request(ROUTED_TOOL_EXECUTE_METHOD, json!({})),
        Err(CoordinatorToolRelayError::Closed(_))
    ));
    assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
}

#[test]
fn runner_tool_publication_allows_synchronous_resolve_and_close() {
    for close in [false, true] {
        let events = GatewayEventHub::default();
        let relay = Arc::new(CoordinatorToolRelay::with_timeout(events.clone(), Duration::from_secs(30)));
        let weak_relay = Arc::downgrade(&relay);
        let (callback_tx, callback_rx) = mpsc::channel();
        let _subscription = events.subscribe_listener(move |event| {
            if event["channel"] != RUNNER_TOOL_REQUEST_EVENT_CHANNEL {
                return;
            }
            let relay = weak_relay.upgrade().expect("live relay during publication");
            let before = relay.pending_count();
            let resolution = if close {
                relay.cancel_all("inline Host shutdown");
                None
            } else {
                Some(relay.resolve(&json!({
                    "requestId":event["payload"]["requestId"], "ok":true,
                    "result":{"receipt":"completed"},
                })))
            };
            callback_tx.send((before, relay.pending_count(), resolution)).unwrap();
        });
        let (result_tx, result_rx) = mpsc::channel();
        let worker_relay = Arc::clone(&relay);
        let worker = thread::spawn(move || {
            result_tx.send(worker_relay.request(ROUTED_TOOL_EXECUTE_METHOD, json!({}))).unwrap();
        });
        let result = result_rx.recv_timeout(Duration::from_secs(5))
            .expect("synchronous callback must not deadlock the actual request");
        worker.join().expect("owned request worker");
        let (before, after, resolution) = callback_rx.recv_timeout(Duration::from_secs(5))
            .expect("actual synchronous callback result");
        assert_eq!((before, after, relay.pending_count()), (1, 0, 0));
        if close {
            assert_eq!(result, Err(CoordinatorToolRelayError::Closed("inline Host shutdown".into())));
            assert!(resolution.is_none());
        } else {
            assert_eq!(result, Ok(json!({"receipt":"completed"})));
            assert_eq!(resolution.expect("resolution attempted").expect("inline response accepted")["resolved"], true);
        }
    }
}
