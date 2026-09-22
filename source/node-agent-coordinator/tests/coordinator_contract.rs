use mahayana_node_agent_coordinator::control_port_client::{ClientAction, ControlPortClient};
use mahayana_node_agent_coordinator::gateway::{GatewayHealthDecision, GatewayHostSupervisor};
use mahayana_node_agent_coordinator::mcp::{McpRoute, RoutedMcpBridge};
use mahayana_node_agent_coordinator::protocol::{
    CoordinatorFrame, LifecyclePhase, ReplyOutcome, COORDINATOR_CANCELLED,
    COORDINATOR_PROTOCOL_VERSION,
};
use mahayana_node_agent_coordinator::renderer_port_server::{
    RendererPortPhase, RendererPortServer, RendererPortSettlement, ServerAction,
};
use mahayana_node_agent_coordinator::supervisor::{
    CoordinatorSupervisor, GatewayState, HostGeneration,
};
use serde_json::json;

#[test]
fn protocol_hello_negotiates_exact_version() {
    let mut server = RendererPortServer::default();
    assert_eq!(server.handle_frame(CoordinatorFrame::hello()), vec![
        ServerAction::Post(CoordinatorFrame::ready())
    ]);
    assert_eq!(server.phase(), RendererPortPhase::Serving);

    let mut wrong = RendererPortServer::default();
    let actions = wrong.handle_frame(CoordinatorFrame::Lifecycle {
        phase: LifecyclePhase::Hello,
        protocol_version: Some(COORDINATOR_PROTOCOL_VERSION + 1),
        reason: None,
        detail: None,
    });
    assert!(matches!(actions.last(), Some(ServerAction::Close)));
    assert!(matches!(wrong.settlement(), Some(RendererPortSettlement::ProtocolBreach(_))));
}

#[test]
fn request_ids_are_unique_for_the_session() {
    let mut server = RendererPortServer::default();
    server.handle_frame(CoordinatorFrame::hello());
    server.handle_frame(CoordinatorFrame::Request {
        request_id: "r-1".into(),
        method: "sendPrompt".into(),
        args: json!({}),
    });
    server.complete_request("r-1", ReplyOutcome::Ok { value: json!(true) });
    let actions = server.handle_frame(CoordinatorFrame::Request {
        request_id: "r-1".into(),
        method: "sendPrompt".into(),
        args: json!({}),
    });
    assert!(matches!(actions.last(), Some(ServerAction::Close)));
}

#[test]
fn cancel_aborts_and_settles_the_request() {
    let mut server = RendererPortServer::default();
    server.handle_frame(CoordinatorFrame::hello());
    server.handle_frame(CoordinatorFrame::Request {
        request_id: "r-2".into(),
        method: "sendPrompt".into(),
        args: json!({}),
    });
    let actions = server.handle_frame(CoordinatorFrame::Cancel { request_id: "r-2".into() });
    assert!(actions.iter().any(|a| matches!(a, ServerAction::Abort { request_id } if request_id == "r-2")));
    assert!(actions.iter().any(|a| matches!(
        a,
        ServerAction::Post(CoordinatorFrame::Reply {
            outcome: ReplyOutcome::Failed { failure },
            ..
        }) if failure.code == COORDINATOR_CANCELLED
    )));
}

#[test]
fn disconnect_rejects_every_pending_control_call() {
    let mut client = ControlPortClient::default();
    client.handle_frame(CoordinatorFrame::ready());
    client.call("one", json!({})).unwrap();
    client.call("two", json!({})).unwrap();
    let actions = client.handle_port_closed();
    let rejects = actions.iter().filter(|a| matches!(a, ClientAction::Reject { .. })).count();
    assert_eq!(rejects, 2);
}

#[test]
fn reconnect_resync_reports_generation_and_pending_state() {
    let mut supervisor = CoordinatorSupervisor::default();
    let first = supervisor.connect();
    assert_eq!(first.coordinator_generation, 1);
    supervisor.register_request("r", "sendPrompt", json!({})).unwrap();
    let snapshot = supervisor.reconnect(HostGeneration(0));
    assert_eq!(snapshot.coordinator_generation, 2);
    assert_eq!(snapshot.pending_request_ids, vec!["r"]);
}

#[test]
fn host_restart_settles_old_generation_requests() {
    let mut supervisor = CoordinatorSupervisor::default();
    supervisor.connect();
    supervisor.register_request("r", "sendPrompt", json!({})).unwrap();
    let settlements = supervisor.host_restarted(HostGeneration(2));
    assert_eq!(settlements.len(), 1);
    assert!(supervisor.snapshot().pending_request_ids.is_empty());
    assert_eq!(supervisor.snapshot().host_generation, HostGeneration(2));
}

#[test]
fn gateway_down_up_is_resynchronizable() {
    let mut supervisor = CoordinatorSupervisor::default();
    supervisor.connect();
    supervisor.set_gateway_state(GatewayState::Down);
    assert_eq!(supervisor.snapshot().gateway, GatewayState::Down);
    supervisor.set_gateway_state(GatewayState::Up);
    assert_eq!(supervisor.snapshot().gateway, GatewayState::Up);

    let mut gateway = GatewayHostSupervisor::new(5_000);
    assert_eq!(gateway.decision(0), GatewayHealthDecision::Probe);
    gateway.record_health(100, false);
    assert_eq!(gateway.decision(101), GatewayHealthDecision::Reconnect);
    gateway.invalidate();
    assert_eq!(gateway.decision(102), GatewayHealthDecision::Probe);
}

#[test]
fn mcp_routing_is_explicit_and_generation_bound() {
    let mut bridge = RoutedMcpBridge::default();
    bridge.upsert(McpRoute {
        server_name: "filesystem".into(),
        transport: "stdio".into(),
        generation: 7,
    });
    let (route, tool, args) = bridge.route_tool_call("filesystem", "read_file", json!({"path":"a"})).unwrap();
    assert_eq!(route.generation, 7);
    assert_eq!(tool, "read_file");
    assert_eq!(args["path"], "a");
    assert!(bridge.route_tool_call("missing", "x", json!({})).is_err());
}

#[test]
fn server_direction_frames_from_renderer_are_protocol_breaches() {
    let mut server = RendererPortServer::default();
    server.handle_frame(CoordinatorFrame::hello());
    let actions = server.handle_frame(CoordinatorFrame::Event {
        family: "illegal".into(),
        payload: json!({}),
    });
    assert!(matches!(actions.last(), Some(ServerAction::Close)));
    assert!(matches!(server.settlement(), Some(RendererPortSettlement::ProtocolBreach(_))));
}

#[test]
fn crash_is_recorded_in_resync_snapshot() {
    let mut supervisor = CoordinatorSupervisor::default();
    supervisor.connect();
    let crash = supervisor.record_crash("host", "process exited 137");
    let snapshot = supervisor.snapshot();
    assert_eq!(snapshot.last_crash.as_ref(), Some(&crash));
}


#[test]
fn gateway_event_families_match_the_frozen_coordinator_contract() {
    use mahayana_node_agent_coordinator::gateway::gateway_event_families::{
        coordinator_event_family_for_sse_channel, sse_channel_for_family,
    };

    assert_eq!(sse_channel_for_family("agents-workflow"), Some("workflows"));
    assert_eq!(sse_channel_for_family("mcp-servers-updated"), Some("mcp-servers"));
    assert_eq!(sse_channel_for_family("host-settings"), Some("host-settings"));
    assert_eq!(coordinator_event_family_for_sse_channel("automations"), Some("agents-automation"));
    assert_eq!(coordinator_event_family_for_sse_channel("unknown"), None);
}

#[test]
fn sse_decoder_preserves_chunk_boundaries_and_utf8_payloads() {
    use mahayana_node_agent_coordinator::gateway::sse_block_decoder::SseBlockDecoder;

    let mut decoder = SseBlockDecoder::default();
    assert!(decoder.push_bytes(b"event: transcript\ndata: ").is_empty());
    let unicode = "法布施".as_bytes();
    assert!(decoder.push_bytes(&unicode[..2]).is_empty());
    assert!(decoder.push_bytes(&unicode[2..]).is_empty());
    let blocks = decoder.push_bytes(b"\n\nevent: agents\ndata: ok\n\n");
    assert_eq!(blocks, vec![
        "event: transcript\ndata: 法布施".to_string(),
        "event: agents\ndata: ok".to_string(),
    ]);

    let mut boundary = SseBlockDecoder::default();
    assert!(boundary.push("event: one\n").is_empty());
    assert_eq!(boundary.push("\nevent: two\n\n"), vec![
        "event: one".to_string(),
        "event: two".to_string(),
    ]);
}

#[test]
fn gateway_reachability_matches_http_network_and_base_url_semantics() {
    use mahayana_node_agent_coordinator::gateway::gateway_reachability::{
        classify_base_url_kind, classify_system_error, outcome_for_http_status, BaseUrlKind,
        ReachabilityOutcome,
    };

    assert_eq!(outcome_for_http_status(503), Some(ReachabilityOutcome::Http(503)));
    assert_eq!(outcome_for_http_status(403), Some(ReachabilityOutcome::AccessDenied));
    assert_eq!(outcome_for_http_status(404), None);
    assert_eq!(
        classify_system_error(Some("ECONNREFUSED"), false, ""),
        ReachabilityOutcome::Refused
    );
    assert_eq!(
        classify_system_error(Some("EAI_AGAIN"), false, ""),
        ReachabilityOutcome::Dns
    );
    assert_eq!(
        classify_system_error(None, true, ""),
        ReachabilityOutcome::Timeout
    );
    assert!(!ReachabilityOutcome::AccessDenied.retryable());
    assert!(ReachabilityOutcome::Http(503).retryable());
    assert_eq!(
        classify_base_url_kind(Some("http://127.0.0.1:7777")),
        BaseUrlKind::Loopback
    );
    assert_eq!(
        classify_base_url_kind(Some("https://agent.us8.cursorvm.com")),
        BaseUrlKind::PodProxy
    );
    assert_eq!(classify_base_url_kind(Some("not a url")), BaseUrlKind::Unknown);
}


#[test]
fn box_vnc_proxy_maps_primary_and_fork_viewers_without_touching_foreign_urls() {
    use mahayana_node_agent_coordinator::gateway::box_vnc_proxy::{
        proxify_box_vnc_url, proxify_forever_box_status, VncProxyDescriptor,
    };

    let descriptor = VncProxyDescriptor {
        primary_url: "https://proxy.example/sand-special-treatment-v1/vnc.html?primary=1".into(),
        fork_base_url: "https://fork.example/base".into(),
        network_token: "network-123".into(),
    };
    assert_eq!(
        proxify_box_vnc_url("http://127.0.0.1:6080/vnc.html", &descriptor),
        descriptor.primary_url
    );
    let fork = proxify_box_vnc_url(
        "http://localhost:6081/vnc.html?path=websockify%3Ftoken%3Ddisplay-7",
        &descriptor,
    );
    assert!(fork.starts_with("https://fork.example/base/sand-special-treatment-v1/vnc.html?"));
    assert!(fork.contains("network_token=network-123"));
    assert!(fork.contains("path=websockify%3Ftoken%3Ddisplay-7%26network_token%3Dnetwork-123"));
    assert_eq!(
        proxify_box_vnc_url("https://remote.example:6080/vnc.html", &descriptor),
        "https://remote.example:6080/vnc.html"
    );

    let status = json!({
        "vncUrl": "http://127.0.0.1:6080/vnc.html",
        "windows": [{"vncUrl": "http://127.0.0.1:6081/vnc.html"}],
        "state": "ready"
    });
    let projected = proxify_forever_box_status(&status, Some(&descriptor));
    assert_eq!(projected["vncUrl"], descriptor.primary_url);
    assert!(projected["windows"][0]["vncUrl"].as_str().unwrap().contains("network_token=network-123"));
    assert_eq!(projected["state"], "ready");
}


#[test]
fn local_exec_daemon_files_parse_and_retire_only_the_expected_generation() {
    use mahayana_node_agent_coordinator::local_exec::daemon_files::{
        parse_discovery, read_local_exec_daemon_discovery,
        remove_local_exec_daemon_discovery_if_matches, resolve_local_exec_daemon_paths,
        write_secret_json_file,
    };
    use std::fs;
    use uuid::Uuid;

    let root = std::env::temp_dir().join(format!("fabushi-local-exec-{}", Uuid::new_v4()));
    let paths = resolve_local_exec_daemon_paths(&root);
    assert!(paths.discovery_path.ends_with("local-exec-daemon.json"));
    let value = json!({
        "pid": 4242,
        "startedAt": 1234.5,
        "entryRealpath": "/tmp/daemon",
        "generationToken": "generation-a",
        "inflightCount": 2
    });
    let expected = parse_discovery(value.clone()).expect("valid discovery");
    write_secret_json_file(&paths.discovery_path, &value).expect("write discovery");
    assert_eq!(
        read_local_exec_daemon_discovery(&paths.discovery_path)
            .expect("read discovery")
            .as_ref(),
        Some(&expected)
    );

    let wrong = parse_discovery(json!({
        "pid": 4242,
        "startedAt": 1234.5,
        "entryRealpath": "/tmp/daemon",
        "generationToken": "generation-b"
    })).unwrap();
    assert!(!remove_local_exec_daemon_discovery_if_matches(&paths.discovery_path, &wrong)
        .expect("mismatched generation is restored"));
    assert!(paths.discovery_path.exists());

    assert!(remove_local_exec_daemon_discovery_if_matches(&paths.discovery_path, &expected)
        .expect("matching generation retires"));
    assert!(!paths.discovery_path.exists());
    let _ = fs::remove_dir_all(root);
}


#[test]
fn carrier_bootstrap_and_channels_preserve_grok_process_boundaries() {
    use mahayana_node_agent_coordinator::carrier::{
        parse_bootstrap_argument, Carrier, CarrierChannel, CarrierEnvelope,
    };

    let argument = r#"--bootstrap={"processConfig":{"appVersion":"1.2.75","isPackaged":true,"dataDir":"/tmp/fabushi"}}"#;
    let bootstrap = parse_bootstrap_argument([argument]).expect("valid bootstrap");
    assert_eq!(bootstrap.process_config.app_version, "1.2.75");
    assert!(bootstrap.process_config.is_packaged);

    let mut carrier = Carrier::new(bootstrap);
    carrier
        .accept_envelope(CarrierEnvelope::new(
            CarrierChannel::Control,
            json!({"kind":"control"}),
        ))
        .unwrap();
    carrier
        .accept_envelope(CarrierEnvelope::new(
            CarrierChannel::Data,
            json!({"kind":"renderer"}),
        ))
        .unwrap();
    carrier
        .accept_envelope(CarrierEnvelope::new(
            CarrierChannel::MainData,
            json!({"kind":"main"}),
        ))
        .unwrap();
    let messages = carrier.drain();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].channel, CarrierChannel::Control);
    assert_eq!(messages[1].channel, CarrierChannel::Data);
    assert_eq!(messages[2].channel, CarrierChannel::MainData);
    carrier.close();
    assert!(carrier.post(CarrierChannel::Data, json!({})).is_err());
}
