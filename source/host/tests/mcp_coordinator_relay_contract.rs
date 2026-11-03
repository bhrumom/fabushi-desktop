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

mod desktop_service_contracts {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
        CoordinatorMcpManagerBackend, MCP_UNINSTALL_PLUGIN_METHOD,
    };
    use mahayana_host_runtime::extensions::mcp::mcp_service::{
        McpHostService, McpManagerBackend, PluginSkillsPort, PluginUninstallIntent,
    };
    use mahayana_host_runtime::extensions::mcp::plugin_skills::SandPluginSkillsService;
    use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::{
        get_plugin_skills_cache_path, get_plugin_skills_dir,
    };

    struct TempRoot(PathBuf);
    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct RealService {
        root: TempRoot,
        requests: mpsc::Receiver<Value>,
        relay: Arc<CoordinatorMcpLifecycleRelay>,
        backend: Arc<CoordinatorMcpManagerBackend>,
        skills: Arc<SandPluginSkillsService>,
        service: Arc<McpHostService>,
        payload: Value,
        skill_path: PathBuf,
    }

    #[derive(Debug, PartialEq)]
    struct UninstallCustodySnapshot {
        status: Value,
        image: Vec<u8>,
        marker: Vec<u8>,
    }

    impl RealService {
        fn new() -> Self {
            let root = TempRoot(std::env::temp_dir().join(format!(
                "desktop-mcp-relay-{}", uuid::Uuid::new_v4(),
            )));
            let install = root.0.join("plugins/cache/cursor/calendar/abc123");
            let skill_path = install.join("skills/Schedule/SKILL.md");
            fs::create_dir_all(skill_path.parent().unwrap()).unwrap();
            fs::write(&skill_path,
                "---\nname: Schedule\ndescription: REAL_CALENDAR_DESCRIPTION\n---\nREAL_CALENDAR_BODY\n").unwrap();
            let mut payload = plugin_skills_payload();
            payload["plugins"][0]["installPath"] = json!(install);
            let events = GatewayEventHub::default();
            let requests = events.subscribe();
            let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(events, REQUEST_TIMEOUT));
            let backend = Arc::new(CoordinatorMcpManagerBackend::new(
                Arc::clone(&relay), Arc::new(|_, _| Ok(Vec::new())),
            ));
            let skills = Arc::new(SandPluginSkillsService::new(
                &root.0, Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&relay))),
            ));
            let service = Arc::new(McpHostService::new(backend.clone(), Some(skills.clone())));
            Self { root, requests, relay, backend, skills, service, payload, skill_path }
        }

        fn next(&self, method: &str) -> Value {
            let event = self.requests.recv_timeout(ORACLE_TIMEOUT).expect("actual lifecycle request");
            assert_eq!(event["channel"], HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL);
            assert_eq!(event["payload"]["method"], method);
            event
        }

        fn resolve(&self, event: &Value, value: Value) {
            self.relay.resolve(&json!({
                "requestId":event["payload"]["requestId"], "ok":true, "result":value,
            })).unwrap();
        }

        fn cache_path(&self) -> PathBuf {
            get_plugin_skills_cache_path(get_plugin_skills_dir(&self.root.0))
        }

        fn pending_uninstall(&self, plugin_id: &str) -> PluginUninstallIntent {
            let status = PluginSkillsPort::status(self.skills.as_ref());
            let pending = status["uninstallRecovery"]["pending"].as_array().unwrap();
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0]["pluginId"], plugin_id);
            let operation_id = pending[0]["operationId"].as_str().unwrap().to_string();
            assert_eq!(uuid::Uuid::parse_str(&operation_id).unwrap().to_string(), operation_id);
            PluginUninstallIntent { plugin_id: plugin_id.to_string(), operation_id }
        }

        fn assert_uninstall_custody(&self, confirmed: &[&str], pending: &[&PluginUninstallIntent])
            -> UninstallCustodySnapshot
        {
            self.assert_uninstall_custody_with_refresh(confirmed, pending, None)
        }

        fn refresh_operation_id(&self) -> String {
            let image: Value = serde_json::from_slice(&fs::read(
                get_plugin_skills_dir(&self.root.0).join("uninstall-state.json"),
            ).unwrap()).unwrap();
            let operation_id = image["refreshIntent"]["operationId"].as_str().unwrap().to_string();
            assert_eq!(uuid::Uuid::parse_str(&operation_id).unwrap().to_string(), operation_id);
            operation_id
        }

        fn assert_uninstall_custody_with_refresh(&self, confirmed: &[&str],
            pending: &[&PluginUninstallIntent], refresh: Option<&str>) -> UninstallCustodySnapshot
        {
            let pending_rows = pending.iter().map(|intent| json!({
                "pluginId":intent.plugin_id, "operationId":intent.operation_id,
            })).collect::<Vec<_>>();
            let status = PluginSkillsPort::status(self.skills.as_ref());
            assert_eq!(status, if confirmed.is_empty() && pending.is_empty() {
                json!({"authBlocked":[]})
            } else {
                json!({"authBlocked":[],"uninstallRecovery":{
                    "confirmedPluginIds":confirmed, "pending":pending_rows,
                }})
            });
            let directory = get_plugin_skills_dir(&self.root.0);
            let image = fs::read(directory.join("uninstall-state.json")).unwrap();
            assert_eq!(serde_json::from_slice::<Value>(&image).unwrap(), json!({
                "formatVersion":2, "confirmedPluginIds":confirmed, "pending":pending_rows,
                "authRefusals":[], "refreshIntent":refresh.map(|id| json!({"operationId":id})),
            }));
            let marker = fs::read(directory.join("uninstall-state.version")).unwrap();
            assert_eq!(marker, b"1\n");
            assert!(!self.skills.admission_reader().is_denied("unrelated-plugin-control"));
            UninstallCustodySnapshot { status, image, marker }
        }

        fn seed(&self) {
            let skills = Arc::clone(&self.skills);
            let (worker, result) = spawn_operation(move || skills.sync("seed"));
            let request = self.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
            self.resolve(&request, self.payload.clone());
            assert_eq!(finish(worker, result).unwrap().len(), 1);
        }
    }

    fn spawn_operation<T: Send + 'static>(
        operation: impl FnOnce() -> T + Send + 'static,
    ) -> (thread::JoinHandle<()>, mpsc::Receiver<T>) {
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let _ = tx.send(operation());
        });
        (worker, rx)
    }

    fn finish<T>(worker: thread::JoinHandle<()>, result: mpsc::Receiver<T>) -> T {
        let value = result.recv_timeout(ORACLE_TIMEOUT).expect("bounded actual operation completion");
        worker.join().unwrap();
        value
    }

    #[test]
    fn concrete_backend_preserves_raw_display_and_keeps_confirmation_on_display_failure() {
        let fixture = RealService::new();
        let raw = json!({
            "removed":true, "reason":"  team-server  ",
            "state":{"servers":[],"warnings":["original warning"],"futurePolicy":{"keep":true}},
            "futureReply":{"keep":true},
        });
        for (reply, valid_display) in [
            (raw.clone(), true),
            (json!({"removed":true}), false),
            (json!({"removed":true,"state":null}), false),
            (json!({"removed":true,"state":{"servers":[{"id":"incomplete"}]}}), false),
        ] {
            let backend = Arc::clone(&fixture.backend);
            let (worker, result) = spawn_operation(move || {
                backend.uninstall_plugin_outcome("official:github/review-tools")
            });
            let request = fixture.next(MCP_UNINSTALL_PLUGIN_METHOD);
            assert_eq!(request["payload"]["args"], json!({"pluginId":"official:github/review-tools"}));
            fixture.resolve(&request, reply.clone());
            let outcome = finish(worker, result).unwrap();
            assert!(outcome.removed);
            if valid_display {
                assert_eq!(outcome.reason.as_deref(), Some("team-server"));
                assert_eq!(outcome.desktop_response.unwrap(), reply);
            } else {
                assert!(outcome.desktop_response.is_err());
            }
            assert_eq!(fixture.relay.pending_count(), 0);
            assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }

        // The legacy tuple projection also performs just one mutation and does
        // not depend on the Desktop state shape.
        let backend = Arc::clone(&fixture.backend);
        let (worker, result) = spawn_operation(move || backend.uninstall_plugin("41"));
        let request = fixture.next(MCP_UNINSTALL_PLUGIN_METHOD);
        fixture.resolve(&request, json!({"removed":true,"reason":"  unchanged reason  "}));
        assert_eq!(finish(worker, result).unwrap(), (true, Some("unchanged reason".into())));
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn canonical_desktop_ids_reach_the_concrete_backend_unchanged() {
        let fixture = RealService::new();
        for id in ["184467440737095516160", "official:github/review-tools"] {
            let service = Arc::clone(&fixture.service);
            let (worker, result) = spawn_operation(move || {
                service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":id})).unwrap()
            });
            let catalog = fixture.next(MCP_LIST_CATALOG_METHOD);
            fixture.resolve(&catalog, json!([]));
            let uninstall = fixture.next(MCP_UNINSTALL_PLUGIN_METHOD);
            assert_eq!(uninstall["payload"]["args"], json!({"pluginId":id}));
            let raw = json!({
                "removed":false, "reason":"team-server",
                "state":{"servers":[],"warnings":["original"]},
            });
            fixture.resolve(&uninstall, raw.clone());
            assert_eq!(finish(worker, result).unwrap(), raw);
            assert!(!fixture.skills.admission_reader().is_denied(id));
            assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }

    #[test]
    fn service_close_during_catalog_lookup_prevents_the_uninstall_mutation() {
        let fixture = RealService::new();
        let service = Arc::clone(&fixture.service);
        let (worker, result) = spawn_operation(move || {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"41"})).unwrap()
        });
        let catalog = fixture.next(MCP_LIST_CATALOG_METHOD);
        fixture.service.dispose().unwrap();
        fixture.resolve(&catalog, json!([]));
        assert!(finish(worker, result).unwrap_err().contains("closed"));
        assert!(!fixture.skills.admission_reader().is_denied("41"));
        assert_eq!(fixture.relay.pending_count(), 0);
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn desktop_refresh_and_sync_wait_for_the_actual_loader_and_status_reads_its_cache() {
        let fixture = RealService::new();
        for method in ["refreshMcp", "syncPluginSkills"] {
            let service = Arc::clone(&fixture.service);
            let (worker, result) = spawn_operation(move || {
                service.dispatch_desktop_command(method, &json!({})).unwrap()
            });
            let request = fixture.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
            assert_eq!(request["payload"]["args"], json!({}));
            assert_eq!(fixture.relay.pending_count(), 1);
            assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)),
                "acknowledgment cannot precede the actual sync result");
            fixture.resolve(&request, fixture.payload.clone());
            let response = finish(worker, result).unwrap();
            if method == "refreshMcp" {
                assert_eq!(response, Value::Null);
            } else {
                let records = response.as_array().unwrap();
                assert_eq!(records.len(), 1);
                assert_eq!(records[0]["id"], "plugin-41-schedule");
                assert_eq!(records[0]["pluginId"], "41");
                assert_eq!(records[0]["filePath"], json!(fixture.skill_path));
            }
        }
        let denied = json!({
            "plugins":[], "authBlocked":[{"pluginId":"41","pluginName":"Calendar"}],
            "listedPluginIds":["41"],
            "listedCacheKeys":[{"marketplaceSlug":"cursor","pluginId":"calendar"}],
            "publisherFacts":{}, "currentUserId":11,
        });
        let service = Arc::clone(&fixture.service);
        let (worker, result) = spawn_operation(move || {
            service.dispatch_desktop_command("refreshMcp", &json!({})).unwrap()
        });
        let request = fixture.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
        fixture.resolve(&request, denied);
        assert_eq!(finish(worker, result).unwrap(), Value::Null);
        assert_eq!(fixture.service.dispatch_desktop_command("getPluginSyncStatus", &json!({}))
            .unwrap().unwrap(), json!({
                "authBlocked":[{"pluginId":"41","pluginName":"Calendar"}],
            }));
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
        let before = fs::read(fixture.cache_path()).unwrap();
        for method in ["refreshMcp", "syncPluginSkills"] {
            let service = Arc::clone(&fixture.service);
            let (worker, result) = spawn_operation(move || {
                service.dispatch_desktop_command(method, &json!({})).unwrap()
            });
            let request = fixture.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
            fixture.relay.resolve(&json!({
                "requestId":request["payload"]["requestId"], "ok":false, "error":"real loader failed",
            })).unwrap();
            assert!(finish(worker, result).unwrap_err().contains("real loader failed"));
            assert_eq!(fs::read(fixture.cache_path()).unwrap(), before);
        }
        fixture.service.dispose().unwrap();
        for method in ["refreshMcp", "syncPluginSkills", "getPluginSyncStatus"] {
            assert!(fixture.service.dispatch_desktop_command(method, &json!({})).unwrap()
                .unwrap_err().contains("closed"));
        }
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn desktop_observation_rejects_a_pre_fact_success_from_the_actual_loader() {
        let fixture = RealService::new();
        fixture.seed();
        let before = fs::read(fixture.cache_path()).unwrap();
        let skill_before = fs::read(&fixture.skill_path).unwrap();
        let skills = Arc::clone(&fixture.skills);
        let (old_worker, old_result) = spawn_operation(move || skills.sync("held-before-uninstall"));
        let old_load = fixture.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
        let old_refresh = fixture.refresh_operation_id();

        let service = Arc::clone(&fixture.service);
        let (worker, result) = spawn_operation(move || {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"41"})).unwrap()
        });
        let catalog = fixture.next(MCP_LIST_CATALOG_METHOD);
        fixture.resolve(&catalog, json!([]));
        let uninstall = fixture.next(MCP_UNINSTALL_PLUGIN_METHOD);
        let intent = fixture.pending_uninstall("41");
        fixture.assert_uninstall_custody_with_refresh(&[], &[&intent], Some(&old_refresh));
        fixture.resolve(&uninstall, json!({"removed":true,"state":null}));
        assert!(finish(worker, result).unwrap_err().contains("requires servers"));
        let confirmed = fixture.assert_uninstall_custody_with_refresh(&["41"], &[], Some(&old_refresh));
        assert!(fixture.skills.admission_reader().is_denied("41"));
        assert!(fixture.skills.current().is_empty());
        assert_eq!(fixture.relay.pending_count(), 1);
        assert!(matches!(old_result.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "display failure must not start cleanup refresh");

        fixture.resolve(&old_load, fixture.payload.clone());
        assert_eq!(finish(old_worker, old_result).unwrap_err(),
            "plugin skills sync superseded by an uninstall observation");
        assert_eq!(fs::read(fixture.cache_path()).unwrap(), before);
        assert_eq!(fs::read(&fixture.skill_path).unwrap(), skill_before);
        assert!(fixture.skills.admission_reader().is_denied("41"));
        assert!(fixture.skills.current().is_empty());
        let settled = fixture.assert_uninstall_custody(&["41"], &[]);
        assert_eq!(settled.status, confirmed.status);
        assert_eq!(settled.marker, confirmed.marker);
        assert_eq!(fixture.relay.pending_count(), 0);
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));

        // Only a fresh post-fact materialization can release the actual reader.
        let skills = Arc::clone(&fixture.skills);
        let (worker, result) = spawn_operation(move || skills.sync("fresh-reinstall"));
        let fresh = fixture.next(MCP_LOAD_PLUGIN_SKILLS_METHOD);
        let fresh_refresh = fixture.refresh_operation_id();
        assert_ne!(fresh_refresh, old_refresh);
        let refreshing = fixture.assert_uninstall_custody_with_refresh(&["41"], &[], Some(&fresh_refresh));
        assert_eq!(refreshing.status, settled.status);
        assert_eq!(refreshing.marker, settled.marker);
        fixture.resolve(&fresh, fixture.payload.clone());
        assert_eq!(finish(worker, result).unwrap().len(), 1);
        assert!(!fixture.skills.admission_reader().is_denied("41"));
        assert_eq!(fs::read(&fixture.skill_path).unwrap(), skill_before);
        fixture.assert_uninstall_custody(&[], &[]);
    }

    #[test]
    fn late_desktop_uninstall_reply_cannot_settle_the_successor_service_request() {
        let old = RealService::new();
        let current = RealService::new();
        old.seed();
        current.seed();
        let old_before = fs::read(old.cache_path()).unwrap();
        let current_before = fs::read(current.cache_path()).unwrap();
        let old_skill_before = fs::read(&old.skill_path).unwrap();
        let current_skill_before = fs::read(&current.skill_path).unwrap();

        let service = Arc::clone(&old.service);
        let (old_worker, old_result) = spawn_operation(move || {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"41"})).unwrap()
        });
        let catalog = old.next(MCP_LIST_CATALOG_METHOD);
        old.resolve(&catalog, json!([]));
        let old_request = old.next(MCP_UNINSTALL_PLUGIN_METHOD);
        let old_intent = old.pending_uninstall("41");
        let old_pending = old.assert_uninstall_custody(&[], &[&old_intent]);
        old.relay.cancel_all("Host replaced");
        old.skills.dispose();
        old.service.dispose().unwrap();
        assert!(finish(old_worker, old_result).unwrap_err().contains("closed"));
        assert_eq!(old.assert_uninstall_custody(&[], &[&old_intent]), old_pending);
        assert!(old.skills.admission_reader().is_denied("41"));
        assert!(old.skills.current().is_empty());
        assert_eq!(old.relay.pending_count(), 0);

        let service = Arc::clone(&current.service);
        let (worker, result) = spawn_operation(move || {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"41"})).unwrap()
        });
        let catalog = current.next(MCP_LIST_CATALOG_METHOD);
        current.resolve(&catalog, json!([]));
        let request = current.next(MCP_UNINSTALL_PLUGIN_METHOD);
        let current_intent = current.pending_uninstall("41");
        let current_pending = current.assert_uninstall_custody(&[], &[&current_intent]);
        assert_ne!(current_intent.operation_id, old_intent.operation_id);
        // Both relays have served seed, catalog and uninstall in that order;
        // a sequence-only request identity would collide on this real waiter.
        assert!(old_request["payload"]["requestId"].as_str().unwrap().ends_with("-3"));
        assert!(request["payload"]["requestId"].as_str().unwrap().ends_with("-3"));
        assert_ne!(old_request["payload"]["requestId"], request["payload"]["requestId"]);
        for stale in [
            json!({"requestId":old_request["payload"]["requestId"],"ok":true,
                "result":{"removed":true,"state":{"servers":[]}}}),
            json!({"requestId":old_request["payload"]["requestId"],"ok":false,"error":"old failure"}),
        ] {
            assert!(current.relay.resolve(&stale).is_err());
            assert_eq!(current.relay.pending_count(), 1);
            assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(current.skills.admission_reader().is_denied("41"));
            assert!(current.skills.current().is_empty());
            assert_eq!(current.assert_uninstall_custody(&[], &[&current_intent]), current_pending);
            assert_eq!(old.assert_uninstall_custody(&[], &[&old_intent]), old_pending);
            assert_eq!(fs::read(old.cache_path()).unwrap(), old_before);
            assert_eq!(fs::read(current.cache_path()).unwrap(), current_before);
            assert_eq!(fs::read(&old.skill_path).unwrap(), old_skill_before);
            assert_eq!(fs::read(&current.skill_path).unwrap(), current_skill_before);
            assert!(matches!(current.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(old.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
        current.resolve(&request, json!({"removed":true,"state":null}));
        assert!(finish(worker, result).unwrap_err().contains("requires servers"));
        assert!(current.skills.admission_reader().is_denied("41"));
        current.assert_uninstall_custody(&["41"], &[]);
        assert!(old.skills.admission_reader().is_denied("41"));
        assert_eq!(old.assert_uninstall_custody(&[], &[&old_intent]), old_pending);
        assert_eq!(fs::read(old.cache_path()).unwrap(), old_before);
        assert_eq!(fs::read(current.cache_path()).unwrap(), current_before);
        assert_eq!(fs::read(&old.skill_path).unwrap(), old_skill_before);
        assert_eq!(fs::read(&current.skill_path).unwrap(), current_skill_before);
        assert_eq!(current.relay.pending_count(), 0);
        assert!(matches!(current.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert!(matches!(old.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work() {
        let fixture = RealService::new();
        fixture.seed();
        let before = fs::read(fixture.cache_path()).unwrap();
        let skill_before = fs::read(&fixture.skill_path).unwrap();
        let service = Arc::clone(&fixture.service);
        let (worker, result) = spawn_operation(move || {
            service.dispatch_desktop_command("uninstallMcpPlugin", &json!({"pluginId":"41"})).unwrap()
        });
        let catalog = fixture.next(MCP_LIST_CATALOG_METHOD);
        fixture.resolve(&catalog, json!([]));
        let uninstall = fixture.next(MCP_UNINSTALL_PLUGIN_METHOD);
        assert_eq!(fixture.relay.pending_count(), 1);
        let intent = fixture.pending_uninstall("41");
        let pending = fixture.assert_uninstall_custody(&[], &[&intent]);
        fixture.relay.cancel_all("Host shutdown");
        fixture.skills.dispose();
        fixture.service.dispose().unwrap();
        assert!(finish(worker, result).unwrap_err().contains("closed"));
        assert!(fixture.skills.admission_reader().is_denied("41"));
        assert!(fixture.skills.current().is_empty());
        assert_eq!(fixture.assert_uninstall_custody(&[], &[&intent]), pending,
            "an unknown remote outcome retains its exact pending ticket without a confirmed fact");
        assert_eq!(fs::read(fixture.cache_path()).unwrap(), before);
        assert_eq!(fs::read(&fixture.skill_path).unwrap(), skill_before);
        for stale in [
            json!({"requestId":uninstall["payload"]["requestId"], "ok":true,
                "result":{"removed":true,"state":{"servers":[]}}}),
            json!({"requestId":uninstall["payload"]["requestId"], "ok":false,"error":"late failure"}),
        ] {
            assert!(fixture.relay.resolve(&stale).is_err());
            assert_eq!(fixture.assert_uninstall_custody(&[], &[&intent]), pending);
            assert_eq!(fs::read(fixture.cache_path()).unwrap(), before);
            assert_eq!(fs::read(&fixture.skill_path).unwrap(), skill_before);
        }
        for (method, args) in [
            ("uninstallMcpPlugin", json!({"pluginId":"41"})),
            ("refreshMcp", json!({})),
            ("syncPluginSkills", json!({})),
            ("getPluginSyncStatus", json!({})),
        ] {
            assert!(fixture.service.dispatch_desktop_command(method, &args).unwrap()
                .unwrap_err().contains("closed"));
            assert_eq!(fixture.assert_uninstall_custody(&[], &[&intent]), pending);
            assert_eq!(fs::read(fixture.cache_path()).unwrap(), before);
            assert_eq!(fs::read(&fixture.skill_path).unwrap(), skill_before);
        }
        assert_eq!(fixture.relay.pending_count(), 0);
        assert!(matches!(fixture.requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}
