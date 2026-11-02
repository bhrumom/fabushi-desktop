use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
    CoordinatorMcpLifecycleRelay, CoordinatorMcpLifecycleRelayError,
    CoordinatorPluginSkillsLoader, HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL,
    MCP_LIST_CATALOG_METHOD, MCP_LOAD_PLUGIN_SKILLS_METHOD,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD, decode_effective_plugins,
    decode_mcp_catalog, decode_mcp_server_state,
};
use mahayana_host_runtime::extensions::mcp::plugin_skills::{LoadedPlugins, PluginSkillsLoader};
use mahayana_host_runtime::gateway_server::GatewayEventHub;
use serde_json::{Value, json};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

const ORACLE_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

fn start_plugin_skills_load(
    relay: &Arc<CoordinatorMcpLifecycleRelay>,
) -> (
    thread::JoinHandle<()>,
    mpsc::Receiver<Result<LoadedPlugins, String>>,
) {
    let loader = CoordinatorPluginSkillsLoader::new(Arc::clone(relay));
    let (result_tx, result_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let _ = result_tx.send(loader.load());
    });
    (worker, result_rx)
}

fn first_request_incarnation(request_id: &str) -> uuid::Uuid {
    assert!(request_id.len() < 256);
    let incarnation = request_id
        .strip_prefix("host-mcp-")
        .and_then(|value| value.strip_suffix("-1"))
        .expect("first request must carry a relay incarnation and sequence one");
    let incarnation = uuid::Uuid::parse_str(incarnation).expect("relay UUID");
    assert_eq!(incarnation.get_version_num(), 4);
    incarnation
}

fn plugin_skills_payload() -> Value {
    json!({
        "plugins": [{
            "identifier": {
                "source": "cursor-first-party",
                "name": "calendar",
                "pluginDbId": "41",
                "version": "abc123"
            },
            "displayName": "Calendar",
            "installPath": "/tmp/plugins/calendar",
            "skills": [{
                "name": "Schedule",
                "description": "schedule events",
                "path": "skills/Schedule/SKILL.md"
            }]
        }],
        "authBlocked": [],
        "listedPluginIds": ["41"],
        "listedCacheKeys": [{"marketplaceSlug":"cursor","pluginId":"calendar"}],
        "publisherFacts": {"41":{"publisherUserId":7,"marketplaceTeamId":9}},
        "currentUserId": 11
    })
}

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
fn host_mcp_lifecycle_close_settles_an_admitted_waiter_and_rejects_late_resolution() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events,
        Duration::from_secs(60),
    ));
    let (result_tx, result_rx) = mpsc::channel();
    let worker_relay = Arc::clone(&relay);
    let worker = thread::spawn(move || {
        result_tx
            .send(worker_relay.request(MCP_LIST_CATALOG_METHOD, json!({})))
            .unwrap();
    });
    let event = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("admitted lifecycle request");
    let request_id = event["payload"]["requestId"].as_str().unwrap();
    assert_eq!(relay.pending_count(), 1);

    relay.cancel_all("Host teardown");
    relay.cancel_all("duplicate teardown");
    assert_eq!(
        result_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("close must settle the admitted waiter before its timeout"),
        Err(CoordinatorMcpLifecycleRelayError::Closed(
            "Host teardown".into()
        ))
    );
    worker.join().unwrap();
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(
        relay.resolve(&json!({"requestId": request_id, "ok": true, "result": []})),
        Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
    ));
    assert!(matches!(
        relay.request(MCP_LIST_CATALOG_METHOD, json!({})),
        Err(CoordinatorMcpLifecycleRelayError::Closed(_))
    ));
    assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
}

#[test]
fn replaced_relay_rejects_stale_results_without_settling_the_new_skills_load() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let old_relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events.clone(),
        REQUEST_TIMEOUT,
    ));
    let (old_worker, old_result_rx) = start_plugin_skills_load(&old_relay);
    let old_event = receiver
        .recv_timeout(ORACLE_TIMEOUT)
        .expect("old Skills request");
    assert_eq!(old_event["payload"]["method"], MCP_LOAD_PLUGIN_SKILLS_METHOD);
    let old_id = old_event["payload"]["requestId"].as_str().unwrap().to_string();
    let held_success = json!({
        "requestId": old_id,
        "ok": true,
        "result": {
            "plugins": [],
            "authBlocked": [],
            "listedPluginIds": [],
            "listedCacheKeys": [],
            "publisherFacts": {},
            "currentUserId": 1,
        },
    });
    let held_failure = json!({
        "requestId": old_id,
        "ok": false,
        "error": "old desktop owner failed",
    });

    old_relay.cancel_all("Host replaced");
    assert_eq!(
        old_result_rx
            .recv_timeout(ORACLE_TIMEOUT)
            .unwrap()
            .expect_err("closed old load"),
        CoordinatorMcpLifecycleRelayError::Closed("Host replaced".into()).to_string()
    );
    old_worker.join().unwrap();
    assert_eq!(old_relay.pending_count(), 0);
    drop(old_relay);

    let new_relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events,
        REQUEST_TIMEOUT,
    ));
    let (new_worker, new_result_rx) = start_plugin_skills_load(&new_relay);
    let new_event = receiver
        .recv_timeout(ORACLE_TIMEOUT)
        .expect("new Skills request");
    assert_eq!(new_event["payload"]["method"], MCP_LOAD_PLUGIN_SKILLS_METHOD);
    let new_id = new_event["payload"]["requestId"].as_str().unwrap().to_string();
    assert_eq!(new_relay.pending_count(), 1);

    // Deliver held replies to the current owner, as the Coordinator does.
    // The behavioral oracle precedes format/inequality assertions so a
    // sequence-only regression fails at actual waiter custody.
    for stale_reply in [held_success, held_failure] {
        assert!(matches!(
            new_relay.resolve(&stale_reply),
            Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
        ));
        assert_eq!(new_relay.pending_count(), 1);
        assert!(matches!(
            new_result_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }
    assert_ne!(
        first_request_incarnation(&old_id),
        first_request_incarnation(&new_id)
    );
    assert_ne!(old_id, new_id);

    assert!(matches!(
        new_relay.resolve(&json!({"requestId": new_id, "ok": "true"})),
        Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
    ));
    assert_eq!(new_relay.pending_count(), 1);
    assert!(matches!(
        new_result_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));

    let current_reply = json!({
        "requestId": new_id,
        "ok": true,
        "result": plugin_skills_payload(),
    });
    assert_eq!(
        new_relay
            .resolve(&current_reply)
            .expect("current reply settles its waiter"),
        json!({"resolved": true, "requestId": new_id})
    );
    let loaded = new_result_rx
        .recv_timeout(ORACLE_TIMEOUT)
        .expect("current Skills load completes")
        .expect("current Skills payload decodes");
    new_worker.join().unwrap();
    assert_eq!(loaded.plugins.len(), 1);
    assert_eq!(
        loaded.plugins[0].identifier.plugin_db_id.as_deref(),
        Some("41")
    );
    assert_eq!(
        loaded.plugins[0].skills[0].name.as_deref(),
        Some("Schedule")
    );
    assert_eq!(loaded.listed_plugin_ids, vec!["41".to_string()]);
    assert_eq!(loaded.current_user_id, Some(11));
    assert_eq!(loaded.publisher_facts["41"].marketplace_team_id, Some(9));
    assert_eq!(new_relay.pending_count(), 0);
    for duplicate in [
        current_reply,
        json!({"requestId": new_id, "ok": false, "error": "duplicate failure"}),
    ] {
        assert!(matches!(
            new_relay.resolve(&duplicate),
            Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
        ));
    }
    assert!(matches!(
        new_result_rx.try_recv(),
        Err(mpsc::TryRecvError::Disconnected)
    ));
    assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
}

#[test]
fn two_live_relays_cannot_consume_each_others_first_request() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let left = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events.clone(),
        REQUEST_TIMEOUT,
    ));
    let right = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events,
        REQUEST_TIMEOUT,
    ));
    let (left_worker, left_result_rx) = start_plugin_skills_load(&left);
    let left_event = receiver
        .recv_timeout(ORACLE_TIMEOUT)
        .expect("left first request");
    let left_id = left_event["payload"]["requestId"].as_str().unwrap().to_string();
    let (right_worker, right_result_rx) = start_plugin_skills_load(&right);
    let right_event = receiver
        .recv_timeout(ORACLE_TIMEOUT)
        .expect("right first request");
    let right_id = right_event["payload"]["requestId"].as_str().unwrap().to_string();
    assert_eq!(left.pending_count(), 1);
    assert_eq!(right.pending_count(), 1);

    for (target, other_id) in [(&left, &right_id), (&right, &left_id)] {
        for ok in [true, false] {
            assert!(matches!(
                target.resolve(&json!({
                    "requestId": other_id,
                    "ok": ok,
                    "result": plugin_skills_payload(),
                    "error": "other live owner failed",
                })),
                Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
            ));
            assert_eq!(left.pending_count(), 1);
            assert_eq!(right.pending_count(), 1);
            assert!(matches!(
                left_result_rx.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ));
            assert!(matches!(
                right_result_rx.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ));
        }
    }
    assert_ne!(
        first_request_incarnation(&left_id),
        first_request_incarnation(&right_id)
    );

    let left_reply = json!({"requestId": left_id, "ok": true, "result": plugin_skills_payload()});
    left.resolve(&left_reply).expect("left current reply");
    let loaded_left = left_result_rx.recv_timeout(ORACLE_TIMEOUT).unwrap().unwrap();
    left_worker.join().unwrap();
    assert_eq!(loaded_left.current_user_id, Some(11));
    assert_eq!(left.pending_count(), 0);
    assert_eq!(right.pending_count(), 1);
    assert!(matches!(
        right_result_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    assert!(matches!(
        left.resolve(&left_reply),
        Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
    ));

    let mut right_payload = plugin_skills_payload();
    right_payload["currentUserId"] = json!(22);
    let right_reply = json!({"requestId": right_id, "ok": true, "result": right_payload});
    right.resolve(&right_reply).expect("right current reply");
    let loaded_right = right_result_rx
        .recv_timeout(ORACLE_TIMEOUT)
        .unwrap()
        .unwrap();
    right_worker.join().unwrap();
    assert_eq!(loaded_right.current_user_id, Some(22));
    assert_eq!(right.pending_count(), 0);
    assert!(matches!(
        right.resolve(&right_reply),
        Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
    ));
    assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
}

#[test]
fn host_mcp_lifecycle_publication_allows_synchronous_resolution() {
    let events = GatewayEventHub::default();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events.clone(),
        Duration::from_secs(60),
    ));
    let weak_relay = Arc::downgrade(&relay);
    let (callback_tx, callback_rx) = mpsc::channel();
    let _subscription = events.subscribe_listener(move |event| {
        if event["channel"] != HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL {
            return;
        }
        let relay = weak_relay.upgrade().expect("live relay during publication");
        let before = relay.pending_count();
        let resolved = relay.resolve(&json!({
            "requestId": event["payload"]["requestId"],
            "ok": true,
            "result": [{"id": "inline-plugin"}],
        }));
        callback_tx
            .send((before, resolved, relay.pending_count()))
            .unwrap();
    });
    let (result_tx, result_rx) = mpsc::channel();
    let worker_relay = Arc::clone(&relay);
    let worker = thread::spawn(move || {
        result_tx
            .send(worker_relay.request(MCP_LIST_CATALOG_METHOD, json!({})))
            .unwrap();
    });

    assert_eq!(
        result_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("synchronous resolve must not deadlock publication"),
        Ok(json!([{"id": "inline-plugin"}]))
    );
    worker.join().unwrap();
    let (before, resolved, after) = callback_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("synchronous resolution report");
    assert_eq!(before, 1);
    assert_eq!(resolved.unwrap()["resolved"], true);
    assert_eq!(after, 0);
    assert_eq!(relay.pending_count(), 0);
}

#[test]
fn host_mcp_lifecycle_publication_allows_synchronous_close() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events.clone(),
        Duration::from_secs(60),
    ));
    let weak_relay = Arc::downgrade(&relay);
    let (callback_tx, callback_rx) = mpsc::channel();
    let _subscription = events.subscribe_listener(move |event| {
        if event["channel"] != HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL {
            return;
        }
        let relay = weak_relay.upgrade().expect("live relay during publication");
        let before = relay.pending_count();
        relay.cancel_all("listener closes relay");
        callback_tx
            .send((before, relay.pending_count()))
            .unwrap();
    });
    let (result_tx, result_rx) = mpsc::channel();
    let worker_relay = Arc::clone(&relay);
    let worker = thread::spawn(move || {
        result_tx
            .send(worker_relay.request(MCP_LIST_CATALOG_METHOD, json!({})))
            .unwrap();
    });

    assert_eq!(
        result_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("synchronous close must not deadlock publication"),
        Err(CoordinatorMcpLifecycleRelayError::Closed(
            "listener closes relay".into()
        ))
    );
    worker.join().unwrap();
    assert_eq!(
        callback_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        (1, 0)
    );
    let event = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("the already-admitted request was published");
    assert!(matches!(
        relay.resolve(&json!({
            "requestId": event["payload"]["requestId"],
            "ok": true,
            "result": [],
        })),
        Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
    ));
    assert!(matches!(
        relay.request(MCP_LIST_CATALOG_METHOD, json!({})),
        Err(CoordinatorMcpLifecycleRelayError::Closed(_))
    ));
    assert_eq!(relay.pending_count(), 0);
    assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
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


#[test]
fn plugin_skills_loader_uses_the_same_correlated_coordinator_relay() {
    let events = GatewayEventHub::default();
    let receiver = events.subscribe();
    let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
        events,
        Duration::from_secs(2),
    ));
    let loader = CoordinatorPluginSkillsLoader::new(Arc::clone(&relay));
    let handle = thread::spawn(move || loader.load());

    let event = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("plugin skills request event");
    assert_eq!(event["payload"]["method"], MCP_LOAD_PLUGIN_SKILLS_METHOD);
    let request_id = event["payload"]["requestId"]
        .as_str()
        .expect("request id")
        .to_string();
    relay
        .resolve(&json!({
            "requestId": request_id,
            "ok": true,
            "result": plugin_skills_payload(),
        }))
        .expect("settle plugin skills load");

    let loaded = handle
        .join()
        .expect("loader worker")
        .expect("plugin skills payload");
    assert_eq!(loaded.plugins.len(), 1);
    assert_eq!(loaded.plugins[0].identifier.plugin_db_id.as_deref(), Some("41"));
    assert_eq!(loaded.plugins[0].skills[0].name.as_deref(), Some("Schedule"));
    assert_eq!(loaded.current_user_id, Some(11));
    assert_eq!(loaded.publisher_facts["41"].marketplace_team_id, Some(9));
}
