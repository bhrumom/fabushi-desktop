use mahayana_host_runtime::gateway_server::GatewayEventHub;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolExecutionError, CoordinatorToolRelay, CoordinatorToolRelayError, ROUTED_TOOL_EXECUTE_METHOD,
    ROUTED_TOOL_LIST_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};
use serde_json::{Value, json};
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

const EXECUTION_BOUND: Duration = Duration::from_secs(5);

fn published_request_id(events: &mpsc::Receiver<Value>, method: &str) -> String {
    let event = events.recv_timeout(EXECUTION_BOUND).expect("actual relay publication");
    assert_eq!(event["channel"], RUNNER_TOOL_REQUEST_EVENT_CHANNEL);
    assert_eq!(event["payload"]["method"], method);
    event["payload"]["requestId"].as_str().expect("opaque request ID").to_string()
}

fn start_execution_request(relay: &Arc<CoordinatorToolRelay>) -> (
    mpsc::Receiver<Result<Value, CoordinatorToolExecutionError>>, thread::JoinHandle<()>,
) {
    let relay = Arc::clone(relay);
    let (returned, result) = mpsc::channel();
    let worker = thread::spawn(move || {
        returned.send(relay.request_tool_execution(json!({
            "toolName":"effect", "args":{"nonIdempotent":true},
        }))).expect("return actual execution request");
    });
    (result, worker)
}

fn relay_result<T: Send + 'static>(
    method: &'static str,
    value: Value,
    request: impl FnOnce(Arc<CoordinatorToolRelay>) -> T + Send + 'static,
) -> T {
    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = Arc::new(CoordinatorToolRelay::with_timeout(events, EXECUTION_BOUND));
    let worker_relay = Arc::clone(&relay);
    let (returned, result) = mpsc::channel();
    let worker = thread::spawn(move || {
        assert!(returned.send(request(worker_relay)).is_ok());
    });
    let request_id = published_request_id(&event_rx, method);
    assert_eq!(relay.resolve(&json!({
        "requestId":request_id, "ok":true, "result":value,
    })).expect("resolve the published request")["resolved"], true);
    let result = result.recv_timeout(EXECUTION_BOUND).expect("relay result within its bound");
    worker.join().expect("owned relay request worker");
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    result
}

#[test]
fn execution_aware_relay_preserves_definitive_mcp_result_values() {
    let nested = json!({
        "error":"timeout after connection reset is user data",
        "result":{"case":"error","value":{"error":"not envelope authority"}},
    });
    for value in [
        json!({"result":{"case":"success","value":{"content":[],"isError":false}}}),
        json!({"result":{"case":"success","value":{
            "content":[{"content":{"case":"text","value":{"text":"done"}}}],
            "isError":true, "structuredContent":nested.clone(),
        }}}),
        json!({"success":{}}),
        json!({"success":{"content":[{"text":{"text":"server error result"}}],"isError":true}}),
        json!({"content":[],"isError":false}),
        json!({"content":[{"type":"text","text":"deadline exceeded is returned data"}],"isError":true}),
        json!({"content":[],"structuredContent":nested.clone()}),
        json!({"structuredContent":nested}),
        json!({"result":{"case":"rejected","value":{"reason":"declined","isReadonly":false}}}),
        json!({"rejected":{}}),
        json!({"result":{"case":"permissionDenied","value":{"error":"permission denied","isReadonly":true}}}),
        json!({"permissionDenied":{"error":"explicit preflight refusal"}}),
        json!({"result":{"case":"toolNotFound","value":{"name":"missing","availableTools":["other"]}}}),
        json!({"toolNotFound":{}}),
        json!({"result":{"case":"serverNotFound","value":{"name":"missing","availableServers":[]}}}),
        json!({"serverNotFound":{"availableServers":["other"]}}),
        json!({"result":{"case":"approved","value":{}}}),
        json!({"approved":{}}),
    ] {
        let returned = relay_result(ROUTED_TOOL_EXECUTE_METHOD, value.clone(), |relay| {
            relay.request_tool_execution(json!({"toolName":"effect","args":{}}))
        }).expect("explicit known MCP result");
        assert_eq!(returned, value, "known result must retain its entire original value");
    }
}

#[test]
fn execution_aware_relay_refuses_ambiguous_and_malformed_mcp_results() {
    for value in [
        json!({"result":{"case":"error","value":{"error":"connection reset"}}}),
        json!({"error":{"error":"a harmless-looking arbitrary message"}}),
        Value::Null,
        json!([]),
        json!("completed"),
        json!({}),
        json!({"receipt":"completed"}),
        json!({"isError":false}),
        json!({"result":null}),
        json!({"result":{}}),
        json!({"result":{"case":true,"value":{}}}),
        json!({"result":{"case":"unrecognized","value":{}}}),
        json!({"result":{"case":"success"}}),
        json!({"result":{"case":"success","value":null}}),
        json!({"success":[]}),
        json!({"success":{"content":{}}}),
        json!({"result":{"case":"success","value":{"content":[null]}}}),
        json!({"success":{"isError":"false"}}),
        json!({"result":{"case":"success","value":{"structuredContent":[]}}}),
        json!({"content":null}),
        json!({"content":[],"isError":0}),
        json!({"structuredContent":"not an object"}),
        json!({"rejected":{"reason":false}}),
        json!({"permissionDenied":{"isReadonly":"true"}}),
        json!({"toolNotFound":{"availableTools":[1]}}),
        json!({"serverNotFound":{"name":null}}),
        json!({"success":{},"error":{"error":"competing oneof arm"}}),
        json!({"result":{"case":"success","value":{}},"success":{}}),
        json!({"content":[],"result":{"case":"success","value":{}}}),
        json!({"structuredContent":{},"error":{"error":"competing raw envelope"}}),
    ] {
        let error = relay_result(ROUTED_TOOL_EXECUTE_METHOD, value.clone(), |relay| {
            relay.request_tool_execution(json!({"toolName":"effect","args":{}}))
        }).expect_err("an ambiguous reply cannot prove external completion");
        assert!(error.outcome_unknown(), "published execution value: {value}");
        assert!(matches!(error.into_error(), CoordinatorToolRelayError::Protocol(_)));
    }
}

#[test]
fn legacy_relay_keeps_list_and_execute_opaque_result_contract() {
    for method in [ROUTED_TOOL_LIST_METHOD, ROUTED_TOOL_EXECUTE_METHOD] {
        for value in [
            Value::Null,
            json!({"receipt":"legacy opaque value"}),
            json!({"result":{"case":"error","value":{"error":"transport uncertain"}}}),
        ] {
            let returned = relay_result(method, value.clone(), move |relay| {
                relay.request(method, json!({}))
            }).expect("legacy result projection remains compatible");
            assert_eq!(returned, value);
        }
    }
}

#[test]
fn execution_aware_relay_retains_original_errors_and_publication_disposition() {
    for failure in ["timeout", "close", "remote", "opaque-remote"] {
        let events = GatewayEventHub::default();
        let event_rx = events.subscribe();
        let timeout = if failure == "timeout" { Duration::from_millis(250) } else { EXECUTION_BOUND };
        let relay = Arc::new(CoordinatorToolRelay::with_timeout(events, timeout));
        let (result, worker) = start_execution_request(&relay);
        let request_id = published_request_id(&event_rx, ROUTED_TOOL_EXECUTE_METHOD);
        let message = if failure == "remote" {
            "COORDINATOR_DISCONNECTED: control command timed out"
        } else {
            "la demande a échoué; no execution disposition is supplied"
        };
        match failure {
            "timeout" => {}
            "close" => relay.cancel_all("closed after dispatch"),
            _ => {
                relay.resolve(&json!({"requestId":request_id,"ok":false,"error":message}))
                    .expect("opaque remote failure resolution");
            }
        }
        let error = result.recv_timeout(EXECUTION_BOUND).expect("local waiter ended")
            .expect_err("controlled relay failure");
        worker.join().expect("owned execution worker");
        assert!(error.outcome_unknown(), "post-publication {failure}");
        match error.into_error() {
            CoordinatorToolRelayError::Timeout(id) => {
                assert_eq!(failure, "timeout");
                assert_eq!(id, request_id);
            }
            CoordinatorToolRelayError::Closed(reason) => {
                assert_eq!(failure, "close");
                assert_eq!(reason, "closed after dispatch");
            }
            CoordinatorToolRelayError::Remote { request_id: id, message: original } => {
                assert!(matches!(failure, "remote" | "opaque-remote"));
                assert_eq!(id, request_id);
                assert_eq!(original, message);
            }
            other => panic!("original relay error changed: {other:?}"),
        }
        assert_eq!(relay.pending_count(), 0);
        assert!(matches!(relay.resolve(&json!({
            "requestId":request_id,"ok":true,"result":{"content":[]},
        })), Err(CoordinatorToolRelayError::Protocol(_))));
    }

    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let relay = CoordinatorToolRelay::with_timeout(events, EXECUTION_BOUND);
    relay.cancel_all("closed before request");
    let error = relay.request_tool_execution(json!({"toolName":"effect"}))
        .expect_err("known pre-publication refusal");
    assert!(!error.outcome_unknown());
    assert!(matches!(error.into_error(), CoordinatorToolRelayError::Closed(_)));
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(event_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
}

#[test]
fn replacement_relay_rejects_late_old_replies_without_settling_its_first_request() {
    for close_old in [false, true] {
        let events = GatewayEventHub::default();
        let event_rx = events.subscribe();
        let old = Arc::new(CoordinatorToolRelay::with_timeout(
            events.clone(), if close_old { EXECUTION_BOUND } else { Duration::from_millis(250) },
        ));
        let (old_result, old_worker) = start_execution_request(&old);
        let old_id = published_request_id(&event_rx, ROUTED_TOOL_EXECUTE_METHOD);
        if close_old { old.cancel_all("replace relay while downstream remains unconfirmed"); }
        assert!(old_result.recv_timeout(EXECUTION_BOUND).expect("old local waiter ended")
            .expect_err("old request is unsettled").outcome_unknown());
        old_worker.join().expect("old request worker");

        let replacement = Arc::new(CoordinatorToolRelay::with_timeout(events, EXECUTION_BOUND));
        let (result, worker) = start_execution_request(&replacement);
        let current_id = published_request_id(&event_rx, ROUTED_TOOL_EXECUTE_METHOD);
        assert_ne!(old_id, current_id, "the first sequence of a new owner cannot reuse an old ID");
        for stale in [
            json!({"requestId":old_id,"ok":true,"result":{"content":[]}}),
            json!({"requestId":old_id,"ok":false,"error":"late old error"}),
        ] {
            assert!(matches!(replacement.resolve(&stale), Err(CoordinatorToolRelayError::Protocol(_))));
            assert_eq!(replacement.pending_count(), 1);
            assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
        let current_value = json!({"content":[],"structuredContent":{"owner":"replacement"}});
        let ack = replacement.resolve(&json!({
            "requestId":current_id,"ok":true,"result":current_value,
        })).expect("matching new result");
        assert_eq!(ack["requestId"], current_id);
        assert_eq!(result.recv_timeout(EXECUTION_BOUND).expect("new waiter")
            .expect("new result"), current_value);
        worker.join().expect("new request worker");
        assert_eq!((old.pending_count(), replacement.pending_count()), (0, 0));
    }
}

#[test]
fn simultaneous_relays_cannot_resolve_each_others_first_request() {
    let events = GatewayEventHub::default();
    let event_rx = events.subscribe();
    let first = Arc::new(CoordinatorToolRelay::with_timeout(events.clone(), EXECUTION_BOUND));
    let second = Arc::new(CoordinatorToolRelay::with_timeout(events, EXECUTION_BOUND));
    let (first_result, first_worker) = start_execution_request(&first);
    let first_id = published_request_id(&event_rx, ROUTED_TOOL_EXECUTE_METHOD);
    let (second_result, second_worker) = start_execution_request(&second);
    let second_id = published_request_id(&event_rx, ROUTED_TOOL_EXECUTE_METHOD);
    assert_ne!(first_id, second_id);
    for (relay, result, foreign_id) in [
        (&first, &first_result, &second_id), (&second, &second_result, &first_id),
    ] {
        for stale in [
            json!({"requestId":foreign_id,"ok":true,"result":{"content":[]}}),
            json!({"requestId":foreign_id,"ok":false,"error":"another owner's failure"}),
            json!({"requestId":"runner-tool-1","ok":true,"result":{"content":[]}}),
        ] {
            assert!(matches!(relay.resolve(&stale), Err(CoordinatorToolRelayError::Protocol(_))));
            assert_eq!(relay.pending_count(), 1);
            assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }
    for (relay, result, id) in [
        (&first, &first_result, &first_id), (&second, &second_result, &second_id),
    ] {
        let value = json!({"content":[],"structuredContent":{"request":id}});
        relay.resolve(&json!({"requestId":id,"ok":true,"result":value}))
            .expect("only the matching owner resolves its request");
        assert_eq!(result.recv_timeout(EXECUTION_BOUND).expect("matching result")
            .expect("known execution result"), value);
        assert_eq!(relay.pending_count(), 0);
    }
    first_worker.join().expect("first owned worker");
    second_worker.join().expect("second owned worker");
}
