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
fn control_port_preserves_client_event_direction_and_rejects_server_events() {
    let mut client = ControlPortClient::default();
    assert!(matches!(
        client.post_event("renderer-event", json!({"ok": true})),
        Some(ClientAction::Post(CoordinatorFrame::Event { .. }))
    ));

    client.handle_frame(CoordinatorFrame::ready());
    let actions = client.handle_frame(CoordinatorFrame::Event {
        family: "server-event".into(),
        payload: json!({}),
    });
    assert!(matches!(
        actions.first(),
        Some(ClientAction::Post(CoordinatorFrame::Lifecycle {
            phase: LifecyclePhase::Shutdown,
            ..
        }))
    ));
    assert!(matches!(actions.last(), Some(ClientAction::Close)));
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


#[test]
fn client_side_tool_v2_relay_fences_epochs_sequences_and_replays_wire_payloads() {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use mahayana_node_agent_coordinator::client_side_tool_v2_relay::{
        ClientSideToolV2Relay, EncodedToolMessage, ToolMessageKind, ToolTransportEvent,
        CLIENT_SIDE_TOOL_V2_ACCOUNT_SLOT, CLIENT_SIDE_TOOL_V2_WIRE_VERSION,
    };

    fn encoded(kind: ToolMessageKind, call_id: &str) -> EncodedToolMessage {
        let mut wire = Vec::new();
        match kind {
            ToolMessageKind::Call => wire.push((3_u8 << 3) | 2),
            ToolMessageKind::Result => {
                wire.push(0x9a);
                wire.push(0x02);
            }
            ToolMessageKind::Reset => unreachable!(),
        }
        wire.push(call_id.len() as u8);
        wire.extend_from_slice(call_id.as_bytes());
        EncodedToolMessage {
            encoding: "protobuf-base64".into(),
            message_type: match kind {
                ToolMessageKind::Call => "aiserver.v1.ClientSideToolV2Call",
                ToolMessageKind::Result => "aiserver.v1.ClientSideToolV2Result",
                ToolMessageKind::Reset => unreachable!(),
            }.into(),
            bytes: STANDARD.encode(wire),
        }
    }

    fn event(epoch: &str, sequence: u64, kind: ToolMessageKind, call_id: &str) -> ToolTransportEvent {
        ToolTransportEvent {
            version: CLIENT_SIDE_TOOL_V2_WIRE_VERSION,
            kind,
            account_slot: CLIENT_SIDE_TOOL_V2_ACCOUNT_SLOT.into(),
            agent_id: "agent-1".into(),
            epoch: epoch.into(),
            sequence,
            message: (kind != ToolMessageKind::Reset).then(|| encoded(kind, call_id)),
        }
    }

    let mut relay = ClientSideToolV2Relay::default();
    let call = relay.accept(event("epoch-a", 1, ToolMessageKind::Call, "call-7")).unwrap();
    assert_eq!(call.bytes.as_deref(), Some(&[0x1a, 0x06, b'c', b'a', b'l', b'l', b'-', b'7'][..]));
    assert!(relay.accept(event("epoch-a", 1, ToolMessageKind::Call, "duplicate")).is_none());
    assert!(relay.accept(event("epoch-a", 2, ToolMessageKind::Result, "call-7")).is_some());
    assert_eq!(relay.replay().len(), 2);

    assert!(relay.accept(event("epoch-b", 1, ToolMessageKind::Call, "call-8")).is_some());
    assert!(relay.accept(event("epoch-a", 3, ToolMessageKind::Result, "call-7")).is_none());
    assert_eq!(relay.replay().len(), 1);

    assert!(relay.accept(event("epoch-b", 2, ToolMessageKind::Reset, "")).is_some());
    assert!(relay.replay().is_empty());
}


#[test]
fn gateway_dispatch_preserves_command_unreachable_and_transport_failures() {
    use mahayana_node_agent_coordinator::gateway::gateway_errors::SandGatewayCommandError;
    use mahayana_node_agent_coordinator::gateway::gateway_reachability::ReachabilityOutcome;
    use mahayana_node_agent_coordinator::gateway::gateway_request_dispatcher::{
        GatewayDispatchError, GatewayRequestDispatcher, GATEWAY_COMMAND_FAILED,
        GATEWAY_TRANSPORT_FAILED, GATEWAY_UNREACHABLE,
    };

    let mut dispatcher = GatewayRequestDispatcher::default();
    dispatcher.register("command", Box::new(|_| {
        Err(GatewayDispatchError::Command(SandGatewayCommandError::new("bad command")))
    }));
    dispatcher.register("dns", Box::new(|_| {
        Err(GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: "dns failed".into(),
        })
    }));
    dispatcher.register("transport", Box::new(|_| {
        Err(GatewayDispatchError::Transport("socket closed".into()))
    }));

    let command = dispatcher.dispatch("command", json!({})).unwrap_err();
    assert_eq!(command.code, GATEWAY_COMMAND_FAILED);
    assert_eq!(command.transport_kind, None);

    let dns = dispatcher.dispatch("dns", json!({})).unwrap_err();
    assert_eq!(dns.code, GATEWAY_UNREACHABLE);
    assert_eq!(dns.transport_kind.as_deref(), Some("dns"));

    let transport = dispatcher.dispatch("transport", json!({})).unwrap_err();
    assert_eq!(transport.code, GATEWAY_TRANSPORT_FAILED);

    let serialized = serde_json::to_value(dns).unwrap();
    assert_eq!(serialized["transportKind"], "dns");
}


#[test]
fn gateway_dns_diagnostics_match_cursorvm_target_and_failure_matrix() {
    use mahayana_node_agent_coordinator::gateway::gateway_dns_diagnostics::{
        classify_dns_diagnosis, classify_probe_error, dns_target_from_base_url, DnsCluster,
        DnsDiagnosis, DnsProbeResult, GatewayDnsDiagnosticReporter,
        DNS_PROBE_MIN_INTERVAL_MS,
    };

    let target = dns_target_from_base_url(
        Some("https://agent-7.us8.cursorvm.com/api"),
        "probe-1",
    )
    .expect("cursorvm endpoint");
    assert_eq!(target.cluster, DnsCluster::Us8);
    assert_eq!(target.endpoint_hostname, "agent-7.us8.cursorvm.com");
    assert_eq!(target.wildcard_hostname, "probe-1.us8.cursorvm.com");
    assert!(dns_target_from_base_url(Some("http://agent-7.us8.cursorvm.com"), "probe").is_none());
    assert_eq!(classify_probe_error(Some("ENOTFOUND"), false), DnsProbeResult::NotFound);
    assert_eq!(classify_probe_error(Some("EAI_AGAIN"), false), DnsProbeResult::TemporaryFailure);
    assert_eq!(
        classify_dns_diagnosis(
            DnsProbeResult::NotFound,
            DnsProbeResult::Resolved,
            DnsProbeResult::Resolved,
            DnsProbeResult::Resolved,
        ),
        DnsDiagnosis::SystemPathFailure
    );
    assert_eq!(
        classify_dns_diagnosis(
            DnsProbeResult::NotFound,
            DnsProbeResult::NotFound,
            DnsProbeResult::Resolved,
            DnsProbeResult::Resolved,
        ),
        DnsDiagnosis::EndpointFailure
    );

    let mut reporter = GatewayDnsDiagnosticReporter::default();
    assert!(reporter.diagnose(
        1,
        DnsCluster::Us8,
        DnsProbeResult::NotFound,
        DnsProbeResult::NotFound,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
    ).is_some());
    assert!(reporter.diagnose(
        1 + DNS_PROBE_MIN_INTERVAL_MS - 1,
        DnsCluster::Us8,
        DnsProbeResult::NotFound,
        DnsProbeResult::NotFound,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
    ).is_none());
    assert!(reporter.diagnose(
        1 + DNS_PROBE_MIN_INTERVAL_MS,
        DnsCluster::Us8,
        DnsProbeResult::NotFound,
        DnsProbeResult::NotFound,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
        DnsProbeResult::Resolved,
    ).is_some());
}


#[test]
fn gateway_host_supervisor_rejects_stale_connection_resolution_after_invalidation() {
    use mahayana_node_agent_coordinator::gateway::host_supervisor::{
        GatewayConnection, GatewayHealthDecision, GatewayHostSupervisor,
    };
    use std::collections::BTreeMap;

    let mut supervisor = GatewayHostSupervisor::new(5_000);
    let stale = supervisor.begin_connection_attempt();
    supervisor.invalidate();
    let fresh = supervisor.begin_connection_attempt();
    assert_ne!(stale.health_epoch, fresh.health_epoch);

    assert!(supervisor
        .settle_connection_attempt(
            stale,
            GatewayConnection {
                base_url: "https://stale.example".into(),
                headers: BTreeMap::new(),
            },
        )
        .is_err());

    supervisor
        .settle_connection_attempt(
            fresh,
            GatewayConnection {
                base_url: "https://fresh.example".into(),
                headers: BTreeMap::new(),
            },
        )
        .expect("fresh connection wins");
    assert_eq!(supervisor.connection().unwrap().base_url, "https://fresh.example");
    assert_eq!(supervisor.decision(0), GatewayHealthDecision::Probe);
    supervisor.record_health(100, true);
    assert_eq!(supervisor.decision(101), GatewayHealthDecision::UseCached);
    supervisor.mark_transport_live(true);
    assert_eq!(supervisor.decision(10_000), GatewayHealthDecision::UseCached);
}


#[test]
fn gateway_client_tracks_endpoint_stream_stall_retry_and_permanent_refusal() {
    use mahayana_node_agent_coordinator::gateway::gateway_client::{
        extract_gateway_error_message, CoordinatorGatewayClient, GatewayClientTiming,
        SSE_RECONNECT_MAX_MS, SSE_RECONNECT_MIN_MS,
    };
    use mahayana_node_agent_coordinator::gateway::gateway_reachability::ReachabilityOutcome;
    use mahayana_node_agent_coordinator::gateway::host_supervisor::GatewayConnection;
    use std::collections::BTreeMap;
    use std::time::Duration;

    assert_eq!(
        extract_gateway_error_message(r#"{"error":"gateway said no"}"#).as_deref(),
        Some("gateway said no")
    );

    let mut client = CoordinatorGatewayClient::default();
    client
        .install_connection(GatewayConnection {
            base_url: "https://gateway.example".into(),
            headers: BTreeMap::new(),
        })
        .unwrap();
    client.start(100).unwrap();
    assert!(client.state.stream_live);
    assert!(!client.state.stream_stalled(100, GatewayClientTiming::default()));
    client.state.mark_event(200);
    assert!(client
        .state
        .stream_stalled(200 + GatewayClientTiming::default().sse_stall_timeout_ms, GatewayClientTiming::default()));

    let first = client.transport_down(ReachabilityOutcome::Dns).unwrap();
    assert_eq!(first, Duration::from_millis(SSE_RECONNECT_MIN_MS));
    let mut last = first;
    for _ in 0..8 {
        last = client.transport_down(ReachabilityOutcome::Network).unwrap();
    }
    assert_eq!(last, Duration::from_millis(SSE_RECONNECT_MAX_MS));
    assert_eq!(
        client.transport_down(ReachabilityOutcome::AccessDenied),
        None
    );
}

#[test]
fn local_exec_supervisor_matches_grok_spawn_adopt_replace_policy() {
    use mahayana_node_agent_coordinator::local_exec::{
        decide_local_exec_daemon_action, LocalExecDaemonAction, LocalExecDaemonOrigin,
        LocalExecDaemonState, LocalExecSupervisor, LOCAL_EXEC_DAEMON_LIVENESS_INTERVAL_MS,
        LOCAL_EXEC_DAEMON_READINESS_POLL_MS, LOCAL_EXEC_DAEMON_READINESS_TIMEOUT_MS,
        LOCAL_EXEC_DAEMON_REFRESH_INTERVAL_MS, LOCAL_EXEC_DAEMON_RESPAWN_LIMIT,
    };
    use mahayana_node_agent_coordinator::local_exec::daemon_files::LocalExecDaemonDiscovery;

    assert_eq!(LOCAL_EXEC_DAEMON_REFRESH_INTERVAL_MS, 30_000);
    assert_eq!(LOCAL_EXEC_DAEMON_LIVENESS_INTERVAL_MS, 1_000);
    assert_eq!(LOCAL_EXEC_DAEMON_READINESS_TIMEOUT_MS, 5_000);
    assert_eq!(LOCAL_EXEC_DAEMON_READINESS_POLL_MS, 50);
    assert_eq!(LOCAL_EXEC_DAEMON_RESPAWN_LIMIT, 10);
    assert_eq!(decide_local_exec_daemon_action(None), LocalExecDaemonAction::Spawn);

    let idle = LocalExecDaemonDiscovery {
        pid: 41,
        started_at: 1.0,
        entry_realpath: Some("/tmp/daemon".into()),
        generation_token: Some("g-1".into()),
        inflight_count: Some(0),
    };
    assert_eq!(
        decide_local_exec_daemon_action(Some(&idle)),
        LocalExecDaemonAction::Replace { pid: 41 }
    );

    let busy = LocalExecDaemonDiscovery {
        inflight_count: Some(2),
        ..idle.clone()
    };
    assert_eq!(
        decide_local_exec_daemon_action(Some(&busy)),
        LocalExecDaemonAction::Adopt { pid: 41 }
    );

    let mut supervisor = LocalExecSupervisor::default();
    assert_eq!(
        supervisor.reconcile_discovery(Some(&busy)),
        LocalExecDaemonAction::Adopt { pid: 41 }
    );
    assert_eq!(supervisor.state(), &LocalExecDaemonState::Adopting { pid: 41 });
    supervisor
        .mark_active(
            LocalExecDaemonOrigin::Adopted,
            41,
            10,
            "fabushi-local-exec --generation g-1",
            "/tmp/daemon",
            "g-1",
        )
        .unwrap();
    assert!(matches!(
        supervisor.state(),
        LocalExecDaemonState::Active {
            origin: LocalExecDaemonOrigin::Adopted,
            pid: 41,
            ..
        }
    ));
}


#[test]
fn transport_stage_recorder_bounds_echo_correlation_and_settles_once() {
    use mahayana_node_agent_coordinator::telemetry::transport_stage_recorder::{
        TransportIdentity, TransportReportLimiter, TransportStageRecorder,
        MAX_IN_FLIGHT_TRANSPORT_REPORTS, PENDING_SEND_ECHO_MAX,
        PENDING_SEND_ECHO_TTL_MS, SSE_ECHO_STAGE,
    };

    let mut recorder = TransportStageRecorder::default();
    assert!(recorder
        .begin_send(
            TransportIdentity {
                account_slot: "host".into(),
                client_nonce: None,
                traceparent: Some("00-root".into()),
            },
            0,
        )
        .is_none());

    let trace = recorder
        .begin_send(
            TransportIdentity {
                account_slot: "host".into(),
                client_nonce: Some("nonce-1".into()),
                traceparent: Some("00-root".into()),
            },
            10,
        )
        .expect("send trace");
    let mut stage = trace.begin_stage("gateway-post", 2, 1_000, 20);
    let completed = stage.complete(45).expect("first settlement");
    assert_eq!(completed.duration_ms, 25);
    assert!(!completed.is_error);
    assert!(stage.fail(50).is_none(), "stage settlement is idempotent");

    let echo = recorder
        .record_send_echo("host", "nonce-1", 1_100, 50)
        .expect("echo report");
    assert_eq!(echo.stage, SSE_ECHO_STAGE);
    assert_eq!(echo.traceparent.as_deref(), Some("00-root"));
    assert!(recorder
        .record_send_echo("host", "nonce-1", 1_101, 51)
        .is_none());

    for index in 0..=PENDING_SEND_ECHO_MAX {
        recorder
            .begin_send(
                TransportIdentity {
                    account_slot: "host".into(),
                    client_nonce: Some(format!("nonce-{index}")),
                    traceparent: Some(format!("00-{index}")),
                },
                100 + index as u64,
            )
            .expect("bounded trace");
    }
    assert_eq!(recorder.pending_echo_count(), PENDING_SEND_ECHO_MAX);
    assert!(recorder
        .record_send_echo("host", "nonce-0", 2_000, 200)
        .is_none(), "oldest pending echo is evicted");

    recorder
        .begin_send(
            TransportIdentity {
                account_slot: "host".into(),
                client_nonce: Some("expired".into()),
                traceparent: Some("00-expired".into()),
            },
            500,
        )
        .expect("expiring trace");
    assert!(recorder
        .record_send_echo(
            "host",
            "expired",
            2_500,
            500 + PENDING_SEND_ECHO_TTL_MS + 1,
        )
        .is_none());

    let mut limiter = TransportReportLimiter::default();
    for _ in 0..MAX_IN_FLIGHT_TRANSPORT_REPORTS {
        assert!(limiter.try_begin());
    }
    assert!(!limiter.try_begin());
    limiter.settle();
    assert!(limiter.try_begin());
}


#[test]
fn routed_mcp_bridge_serves_private_json_rpc_and_routes_cached_tools() {
    use mahayana_node_agent_coordinator::routed_mcp_bridge::{
        RoutedMcpBackend, RoutedMcpHttpOutcome, RoutedMcpProtocolBridge, RoutedTool,
        RoutedToolCall, ROUTED_MCP_MAX_BODY_BYTES, ROUTED_MCP_PROTOCOL_VERSION,
    };
    use mahayana_node_agent_coordinator::Failure;
    use serde_json::{Value, json};

    #[derive(Default)]
    struct Backend {
        calls: Vec<RoutedToolCall>,
    }

    impl RoutedMcpBackend for Backend {
        fn list_tools(&mut self) -> Result<Vec<RoutedTool>, Failure> {
            Ok(vec![
                RoutedTool {
                    name: "github_search".into(),
                    provider_identifier: "github".into(),
                    tool_name: "search".into(),
                    description: Some("Search repositories".into()),
                    input_schema: Some(json!({ "type": "object" })),
                },
                RoutedTool {
                    name: "github_create_issue".into(),
                    provider_identifier: "github".into(),
                    tool_name: "create".into(),
                    description: Some("Create issue".into()),
                    input_schema: None,
                },
            ])
        }

        fn call_tool(&mut self, call: RoutedToolCall) -> Result<Value, Failure> {
            self.calls.push(call);
            Ok(json!({
                "result": {
                    "case": "success",
                    "value": {
                        "isError": false,
                        "content": [{
                            "content": {
                                "case": "text",
                                "value": { "text": "tool-result" }
                            }
                        }]
                    }
                }
            }))
        }
    }

    fn result(outcome: RoutedMcpHttpOutcome) -> Value {
        match outcome {
            RoutedMcpHttpOutcome::Json(value) => value,
            other => panic!("expected JSON reply, got {other:?}"),
        }
    }

    let mut bridge = RoutedMcpProtocolBridge::with_secret("secret").expect("bridge");
    let mut backend = Backend::default();
    assert_eq!(bridge.path(), "/mcp/secret");
    assert_eq!(
        bridge.local_url(43123).expect("url"),
        "http://127.0.0.1:43123/mcp/secret"
    );
    assert!(matches!(
        bridge.handle_http("GET", "/mcp/secret", b"{}", &mut backend),
        RoutedMcpHttpOutcome::HttpError(404)
    ));
    assert!(matches!(
        bridge.handle_http(
            "POST",
            "/mcp/secret",
            &vec![b'x'; ROUTED_MCP_MAX_BODY_BYTES + 1],
            &mut backend,
        ),
        RoutedMcpHttpOutcome::HttpError(413)
    ));

    let initialized = result(bridge.handle_http(
        "POST",
        "/mcp/secret",
        br#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
        &mut backend,
    ));
    assert_eq!(
        initialized["result"]["protocolVersion"],
        ROUTED_MCP_PROTOCOL_VERSION
    );
    assert!(matches!(
        bridge.handle_http(
            "POST",
            "/mcp/secret",
            br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            &mut backend,
        ),
        RoutedMcpHttpOutcome::Accepted
    ));

    let tools = result(bridge.handle_http(
        "POST",
        "/mcp/secret",
        br#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        &mut backend,
    ));
    assert_eq!(bridge.cached_tool_count(), 2);
    let rows = tools["result"]["tools"].as_array().expect("tools");
    let search = rows
        .iter()
        .find(|tool| tool["name"] == "github_search")
        .expect("search tool");
    assert_eq!(search["annotations"]["readOnlyHint"], true);
    let create = rows
        .iter()
        .find(|tool| tool["name"] == "github_create_issue")
        .expect("create tool");
    assert_eq!(create["annotations"]["readOnlyHint"], false);

    let called = result(bridge.handle_http(
        "POST",
        "/mcp/secret",
        br#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"github_search","arguments":{"q":"rust"}}}"#,
        &mut backend,
    ));
    assert_eq!(called["result"]["isError"], false);
    assert_eq!(called["result"]["content"][0]["text"], "tool-result");
    assert_eq!(backend.calls.len(), 1);
    assert_eq!(backend.calls[0].provider_identifier, "github");
    assert_eq!(backend.calls[0].args["q"], "rust");
    assert!(!backend.calls[0].tool_call_id.is_empty());

    let unknown = result(bridge.handle_http(
        "POST",
        "/mcp/secret",
        br#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"missing"}}"#,
        &mut backend,
    ));
    assert_eq!(unknown["result"]["isError"], true);
}


#[test]
fn oauth_forwarder_scopes_loopback_state_ttl_and_listener_lifecycle() {
    use mahayana_node_agent_coordinator::oauth::mcp_oauth_callback_listener::{
        OAuthCallbackDisposition, classify_callback,
    };
    use mahayana_node_agent_coordinator::oauth::mcp_oauth_forwarder::{
        McpOAuthForwarderState, McpOAuthPendingPayload, OAuthForwarderAction,
        MCP_OAUTH_PENDING_TTL_MS,
    };
    use mahayana_node_agent_coordinator::oauth::mcp_oauth_loopback_registry::{
        McpOAuthLoopbackRegistry, loopback_bind_hosts, parse_loopback_redirect,
    };

    let parsed = parse_loopback_redirect("http://localhost:43124/oauth/callback")
        .expect("loopback redirect");
    assert_eq!(parsed.origin, "http://localhost:43124");
    assert_eq!(parsed.path, "/oauth/callback");
    assert_eq!(loopback_bind_hosts("localhost"), vec!["127.0.0.1", "::1"]);
    assert!(parse_loopback_redirect("https://localhost:43124/oauth/callback").is_err());
    assert!(parse_loopback_redirect("http://example.com:43124/oauth/callback").is_err());
    assert!(parse_loopback_redirect("http://localhost/oauth/callback").is_err());

    let mut pool = McpOAuthLoopbackRegistry::default();
    let first = pool
        .acquire("http://localhost:43124/oauth/callback")
        .expect("first lease");
    let second = pool
        .acquire("http://localhost:43124/other")
        .expect("shared origin lease");
    assert_eq!(pool.active_origin_count(), 1);
    assert_eq!(pool.lease_count(&first.origin), 2);
    assert!(pool.release(&first));
    assert_eq!(pool.lease_count(&second.origin), 1);
    assert!(pool.release(&second));
    assert_eq!(pool.active_origin_count(), 0);

    let mut forwarder = McpOAuthForwarderState::default();
    let actions = forwarder
        .handle_pending(
            McpOAuthPendingPayload {
                redirect_url: "http://localhost:43124/oauth/callback".into(),
                state: "state-1".into(),
                server_name: "github".into(),
            },
            1_000,
        )
        .expect("pending");
    assert!(matches!(
        actions.as_slice(),
        [OAuthForwarderAction::StartListener { origin, .. }]
            if origin == "http://localhost:43124"
    ));
    assert!(forwarder
        .listener_started("http://localhost:43124", 1_001)
        .is_empty());
    assert_eq!(forwarder.listener_count(), 1);
    assert_eq!(
        forwarder.resolve_server("http://localhost:43124", "state-1", 1_002),
        Some("github")
    );

    let disposition = classify_callback(
        "/oauth/callback?state=state-1&code=abc%2B123",
        "/oauth/callback",
        |state| forwarder
            .resolve_server("http://localhost:43124", state, 1_003)
            .map(str::to_string),
    )
    .expect("callback classification");
    let OAuthCallbackDisposition::Complete(callback) = disposition else {
        panic!("expected completed callback");
    };
    assert_eq!(callback.code, "abc+123");
    assert_eq!(callback.server_name, "github");

    let completion = forwarder
        .begin_callback(
            "http://localhost:43124",
            callback.code,
            callback.state.clone(),
            1_004,
        )
        .expect("completion");
    assert!(matches!(
        completion,
        OAuthForwarderAction::Complete { callback }
            if callback.server_name == "github" && callback.state == "state-1"
    ));
    let settled = forwarder.settle_callback(
        "http://localhost:43124",
        &callback.state,
        1_005,
    );
    assert!(matches!(
        settled.as_slice(),
        [OAuthForwarderAction::CloseListener { origin }]
            if origin == "http://localhost:43124"
    ));
    assert_eq!(forwarder.pending_count(), 0);
    assert_eq!(forwarder.listener_count(), 0);

    forwarder
        .handle_pending(
            McpOAuthPendingPayload {
                redirect_url: "http://127.0.0.1:43125/callback".into(),
                state: "expiring".into(),
                server_name: "drive".into(),
            },
            10_000,
        )
        .expect("expiring pending");
    forwarder.listener_started("http://127.0.0.1:43125", 10_001);
    let expired = forwarder.expire(10_000 + MCP_OAUTH_PENDING_TTL_MS);
    assert!(matches!(
        expired.as_slice(),
        [OAuthForwarderAction::CloseListener { origin }]
            if origin == "http://127.0.0.1:43125"
    ));
    assert_eq!(forwarder.pending_count(), 0);
}


#[test]
fn webauthn_provider_models_welcome_consent_sign_and_failure_frames() {
    use std::cell::Cell;
    use std::rc::Rc;

    use mahayana_node_agent_coordinator::webauthn::provider::{
        WebAuthnProvider, WebAuthnRequestFrame, WebAuthnResponseFrame,
    };
    use mahayana_node_agent_coordinator::webauthn::signer::{
        SignerEvent, describe_signer_event_as_status, parse_signer_event_line,
        SIGNER_EVENT_PREFIX,
    };
    use mahayana_node_agent_coordinator::webauthn::{
        ApprovedWebAuthnConsent, WebAuthnCeremony, WebAuthnSigner, WebAuthnSignerResult,
    };
    use serde_json::json;

    struct Signer {
        calls: Rc<Cell<usize>>,
    }

    impl WebAuthnSigner for Signer {
        fn sign(
            &mut self,
            ceremony: &WebAuthnCeremony,
            _approved: Option<&ApprovedWebAuthnConsent>,
        ) -> WebAuthnSignerResult {
            self.calls.set(self.calls.get() + 1);
            WebAuthnSignerResult::Success {
                credential_json: json!({
                    "origin": ceremony.origin,
                    "kind": ceremony.kind,
                }),
            }
        }
    }

    let calls = Rc::new(Cell::new(0));
    let mut provider = WebAuthnProvider::new(
        Signer { calls: calls.clone() },
        Some("computer-1".into()),
        Some("Fabushi".into()),
    );

    let welcome = provider.handle_frame(
        WebAuthnRequestFrame::Welcome {
            provider_id: "provider-1".into(),
        },
        None,
    );
    assert_eq!(provider.provider_id(), Some("provider-1"));
    assert!(matches!(
        welcome.as_slice(),
        [WebAuthnResponseFrame::Hello { computer_id, label }]
            if computer_id.as_deref() == Some("computer-1")
                && label.as_deref() == Some("Fabushi")
    ));
    assert!(matches!(provider.heartbeat(), WebAuthnResponseFrame::Ping));

    let ceremony = WebAuthnCeremony {
        kind: "get".into(),
        origin: "https://example.test".into(),
        payload: json!({ "challenge": "abc" }),
    };
    let approved = provider.handle_frame(
        WebAuthnRequestFrame::Ceremony {
            request_id: "request-1".into(),
            ceremony: ceremony.clone(),
        },
        Some(ApprovedWebAuthnConsent::approved()),
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(provider.in_flight_count(), 0);
    assert!(matches!(
        approved.as_slice(),
        [
            WebAuthnResponseFrame::Stage { stage: "grant", outcome: "ok", .. },
            WebAuthnResponseFrame::Stage { stage: "sign", outcome: "ok", .. },
            WebAuthnResponseFrame::Result { credential_json, .. },
        ] if credential_json["origin"] == "https://example.test"
    ));

    let declined = provider.handle_frame(
        WebAuthnRequestFrame::Ceremony {
            request_id: "request-2".into(),
            ceremony,
        },
        Some(ApprovedWebAuthnConsent::declined()),
    );
    assert_eq!(calls.get(), 1, "declined consent must not invoke the signer");
    assert!(matches!(
        declined.as_slice(),
        [
            WebAuthnResponseFrame::Stage { stage: "grant", outcome: "declined", .. },
            WebAuthnResponseFrame::Error { .. },
        ]
    ));

    assert!(provider
        .handle_frame(
            WebAuthnRequestFrame::Cancel {
                request_id: "missing".into(),
            },
            None,
        )
        .is_empty());
    provider.reset_transport();
    assert_eq!(provider.provider_id(), None);

    let presence = parse_signer_event_line(&format!(
        "{SIGNER_EVENT_PREFIX}{{\"kind\":\"presence-required\"}}"
    ))
    .expect("signer event");
    assert_eq!(presence, SignerEvent::PresenceRequired);
    assert_eq!(
        describe_signer_event_as_status(&presence),
        Some("Touch your security key now")
    );
    let pin = parse_signer_event_line(&format!(
        "{SIGNER_EVENT_PREFIX}{{\"kind\":\"pin-invalid\",\"retries\":2}}"
    ))
    .expect("pin event");
    assert_eq!(pin, SignerEvent::PinInvalid { retries: Some(2) });
    assert_eq!(describe_signer_event_as_status(&pin), None);
}
