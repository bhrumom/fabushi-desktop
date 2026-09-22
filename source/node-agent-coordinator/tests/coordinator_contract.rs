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
