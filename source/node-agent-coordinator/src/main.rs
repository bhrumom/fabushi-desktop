use chrono::{SecondsFormat, Utc};
use mahayana_node_agent_coordinator::carrier::{
    parse_bootstrap_argument, CarrierChannel, CarrierEnvelope, CoordinatorBootstrap,
};
use mahayana_node_agent_coordinator::control_port_client::{ClientAction, ControlPortClient};
use mahayana_node_agent_coordinator::gateway::gateway_client::CoordinatorGatewayClient;
use mahayana_node_agent_coordinator::gateway::gateway_reachability::ReachabilityOutcome;
use mahayana_node_agent_coordinator::gateway::gateway_request_dispatcher::{
    dispatch_http_json, failure_for,
};
use mahayana_node_agent_coordinator::gateway::host_supervisor::read_gateway_discovery;
use mahayana_node_agent_coordinator::protocol::{
    CoordinatorFrame, Failure, ReplyOutcome, COORDINATOR_DISCONNECTED,
};
use mahayana_node_agent_coordinator::renderer_port_server::{
    RendererPortServer, ServerAction,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::{env, fs};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
use std::time::Duration;

const HOST_CRASH_CIRCUIT_LIMIT: u64 = 6;

#[derive(Debug)]
struct ActiveHostStdin {
    generation: u64,
    stdin: ChildStdin,
}

#[derive(Debug)]
struct PendingHostRequest {
    generation: u64,
    request_id: String,
    channel: CarrierChannel,
}

#[derive(Debug)]
struct CoordinatorState {
    bootstrap: CoordinatorBootstrap,
    host_bin: PathBuf,
    gateway_discovery_path: PathBuf,
    host_stdin: Mutex<Option<ActiveHostStdin>>,
    gateway_client: Mutex<CoordinatorGatewayClient>,
    pending: Mutex<HashMap<String, PendingHostRequest>>,
    control_port: Mutex<ControlPortClient>,
    renderer_port: Mutex<RendererPortServer>,
    main_data_port: Mutex<RendererPortServer>,
    stdout_lock: Mutex<()>,
    spawn_lock: Mutex<()>,
    host_generation: AtomicU64,
    lifecycle_sequence: AtomicU64,
    closed: AtomicBool,
    consecutive_crashes: AtomicU64,
}

impl CoordinatorState {
    fn write_frame(&self, channel: CarrierChannel, frame: &CoordinatorFrame) -> io::Result<()> {
        let _guard = self
            .stdout_lock
            .lock()
            .map_err(|_| io::Error::other("stdout lock poisoned"))?;
        let envelope = CarrierEnvelope::new(
            channel,
            serde_json::to_value(frame)
                .map_err(|error| io::Error::other(format!("coordinator frame serialization failed: {error}")))?,
        );
        let mut stdout = io::stdout().lock();
        serde_json::to_writer(&mut stdout, &envelope)
            .map_err(|error| io::Error::other(format!("coordinator envelope serialization failed: {error}")))?;
        writeln!(stdout)?;
        stdout.flush()
    }

    fn post_event(&self, family: &str, payload: Value) {
        let action = self
            .renderer_port
            .lock()
            .ok()
            .and_then(|server| server.post_event(family.to_string(), payload));
        if let Some(ServerAction::Post(frame)) = action {
            let _ = self.write_frame(CarrierChannel::Data, &frame);
        }
    }

    fn lifecycle(&self, lifecycle: &str, generation: u64, recoverable: bool, detail: Option<&str>) {
        let sequence = self.lifecycle_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let mut event = json!({
            "type": "host.lifecycle",
            "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            "lifecycle": lifecycle,
            "state": if lifecycle == "running" { "running" } else { "stopped" },
            "generation": generation,
            "sequence": sequence,
            "recoverable": recoverable,
        });
        if let Some(detail) = detail {
            event["reason"] = Value::String(detail.to_string());
        }
        self.post_event("runtime", event);
    }

    fn complete_request(
        &self,
        channel: CarrierChannel,
        request_id: &str,
        outcome: ReplyOutcome,
    ) {
        let actions = match channel {
            CarrierChannel::Data => self
                .renderer_port
                .lock()
                .map(|mut server| server.complete_request(request_id, outcome))
                .unwrap_or_default(),
            CarrierChannel::MainData => self
                .main_data_port
                .lock()
                .map(|mut server| server.complete_request(request_id, outcome))
                .unwrap_or_default(),
            CarrierChannel::Control => Vec::new(),
        };
        for action in actions {
            if let ServerAction::Post(frame) = action {
                let _ = self.write_frame(channel, &frame);
            }
        }
    }

    fn reject_generation(&self, generation: u64, message: &str) {
        let rejected = {
            let mut pending = match self.pending.lock() {
                Ok(pending) => pending,
                Err(_) => return,
            };
            let keys = pending
                .iter()
                .filter_map(|(key, request)| (request.generation == generation).then_some(key.clone()))
                .collect::<Vec<_>>();
            keys.into_iter()
                .filter_map(|key| pending.remove(&key))
                .collect::<Vec<_>>()
        };
        for request in rejected {
            self.complete_request(
                request.channel,
                &request.request_id,
                ReplyOutcome::Failed {
                    failure: Failure::new(COORDINATOR_DISCONNECTED, message),
                },
            );
        }
    }

    fn abort_request(&self, channel: CarrierChannel, request_id: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            let host_request_id = pending
                .iter()
                .find_map(|(host_request_id, request)| {
                    (request.channel == channel && request.request_id == request_id)
                        .then(|| host_request_id.clone())
                });
            if let Some(host_request_id) = host_request_id {
                pending.remove(&host_request_id);
            }
        }
    }
}

fn spawn_host(state: Arc<CoordinatorState>) -> io::Result<u64> {
    let spawn_guard = state
        .spawn_lock
        .lock()
        .map_err(|_| io::Error::other("spawn lock poisoned"))?;
    if state.closed.load(Ordering::SeqCst) {
        return Err(io::Error::other("coordinator is closed"));
    }
    if let Ok(active) = state.host_stdin.lock() {
        if active.is_some() {
            return Ok(state.host_generation.load(Ordering::SeqCst));
        }
    }

    let generation = state.host_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = fs::remove_file(&state.gateway_discovery_path);
    let mut child = Command::new(&state.host_bin)
        .env(
            "SAND_DATA_ROOT",
            state.bootstrap.process_config.data_dir.trim(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            io::Error::other(format!(
                "failed to spawn Mahayana Host {}: {error}",
                state.host_bin.display()
            ))
        })?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Host stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Host stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Host stderr unavailable"))?;
    *state
        .host_stdin
        .lock()
        .map_err(|_| io::Error::other("host stdin lock poisoned"))? =
        Some(ActiveHostStdin { generation, stdin });
    state.lifecycle("running", generation, true, None);
    drop(spawn_guard);

    {
        let discovery_state = Arc::clone(&state);
        thread::spawn(move || {
            for _ in 0..500 {
                if discovery_state.closed.load(Ordering::SeqCst)
                    || discovery_state.host_generation.load(Ordering::SeqCst) != generation
                {
                    return;
                }
                if let Ok(connection) =
                    read_gateway_discovery(&discovery_state.gateway_discovery_path)
                {
                    if let Ok(mut gateway) = discovery_state.gateway_client.lock() {
                        let now_ms =
                            u64::try_from(Utc::now().timestamp_millis()).unwrap_or_default();
                        if gateway.install_connection(connection).is_ok()
                            && gateway.start(now_ms).is_ok()
                        {
                            discovery_state.lifecycle(
                                "gateway-connected",
                                generation,
                                true,
                                None,
                            );
                            return;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            discovery_state.lifecycle(
                "gateway-discovery-timeout",
                generation,
                true,
                Some("Host gateway discovery did not appear"),
            );
        });
    }

    {
        let output_state = Arc::clone(&state);
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if output_state.host_generation.load(Ordering::SeqCst) != generation {
                    continue;
                }
                let value = match serde_json::from_str::<Value>(&line) {
                    Ok(value) => {
                        output_state.consecutive_crashes.store(0, Ordering::SeqCst);
                        value
                    }
                    Err(error) => {
                        output_state.lifecycle(
                            "protocol-error",
                            generation,
                            true,
                            Some(&format!("invalid Host JSON: {error}")),
                        );
                        continue;
                    }
                };

                if let Some(event) = value.get("event").filter(|event| event.is_object()) {
                    output_state.post_event("runtime", event.clone());
                    continue;
                }

                let Some(id) = value.get("id") else {
                    output_state.lifecycle(
                        "protocol-error",
                        generation,
                        true,
                        Some("Host response omitted id"),
                    );
                    continue;
                };
                let host_request_id = id.as_str().map(str::to_owned).unwrap_or_else(|| id.to_string());
                let pending = output_state
                    .pending
                    .lock()
                    .ok()
                    .and_then(|mut pending| pending.remove(&host_request_id));
                let Some(pending) = pending else {
                    continue;
                };

                let outcome = if value.get("ok").and_then(Value::as_bool) == Some(true) {
                    ReplyOutcome::Ok {
                        value: value.get("result").cloned().unwrap_or(Value::Null),
                    }
                } else {
                    ReplyOutcome::Failed {
                        failure: Failure::new(
                            "HOST_REQUEST_FAILED",
                            value
                                .get("error")
                                .and_then(Value::as_str)
                                .unwrap_or("Mahayana Host request failed"),
                        ),
                    }
                };
                output_state.complete_request(pending.channel, &pending.request_id, outcome);
            }
        });
    }

    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else { break };
            eprintln!("[mahayana-host:generation-{generation}] {line}");
        }
    });

    thread::spawn(move || {
        let status = child.wait();
        if let Ok(mut active) = state.host_stdin.lock() {
            if active
                .as_ref()
                .is_some_and(|entry| entry.generation == generation)
            {
                *active = None;
            }
        }
        let detail = match status {
            Ok(status) => format!("Host exited with {status}"),
            Err(error) => format!("Host wait failed: {error}"),
        };
        if let Ok(mut gateway) = state.gateway_client.lock() {
            let _ = gateway.transport_down(ReachabilityOutcome::Network);
        }
        state.reject_generation(generation, &detail);
        state.lifecycle("stopped", generation, true, Some(&detail));

        if state.closed.load(Ordering::SeqCst) {
            return;
        }
        let crashes = state.consecutive_crashes.fetch_add(1, Ordering::SeqCst) + 1;
        if crashes >= HOST_CRASH_CIRCUIT_LIMIT {
            state.lifecycle(
                "circuit-open",
                generation,
                true,
                Some(&format!(
                    "Host crash circuit opened after {crashes} consecutive startup failures"
                )),
            );
            return;
        }
        let delay_ms =
            (250_u64.saturating_mul(1_u64 << crashes.saturating_sub(1).min(4))).min(4_000);
        thread::sleep(Duration::from_millis(delay_ms));
        if !state.closed.load(Ordering::SeqCst) {
            if let Err(error) = spawn_host(Arc::clone(&state)) {
                state.lifecycle(
                    "spawn-failed",
                    state.host_generation.load(Ordering::SeqCst),
                    true,
                    Some(&error.to_string()),
                );
            }
        }
    });

    Ok(generation)
}

fn dispatch_to_host(
    state: &Arc<CoordinatorState>,
    channel: CarrierChannel,
    request_id: String,
    method: String,
    args: Value,
) -> io::Result<()> {
    if method == "coordinator.health" {
        let generation = state.host_generation.load(Ordering::SeqCst);
        let pending = state.pending.lock().map(|pending| pending.len()).unwrap_or_default();
        let (gateway_connected, gateway_generation, gateway_base_url) = state
            .gateway_client
            .lock()
            .map(|gateway| {
                (
                    gateway.state.connected,
                    gateway.state.endpoint_generation,
                    gateway
                        .state
                        .connection
                        .as_ref()
                        .map(|connection| connection.base_url.clone()),
                )
            })
            .unwrap_or((false, 0, None));
        state.complete_request(
            channel,
            &request_id,
            ReplyOutcome::Ok {
                value: json!({
                    "protocolVersion": 1,
                    "hostGeneration": generation,
                    "pending": pending,
                    "hostRunning": state.host_stdin.lock().map(|host| host.is_some()).unwrap_or(false),
                    "gateway": {
                        "connected": gateway_connected,
                        "endpointGeneration": gateway_generation,
                        "baseUrl": gateway_base_url
                    },
                    "processConfig": {
                        "appVersion": state.bootstrap.process_config.app_version.clone(),
                        "isPackaged": state.bootstrap.process_config.is_packaged,
                        "dataDir": state.bootstrap.process_config.data_dir.clone()
                    }
                }),
            },
        );
        return Ok(());
    }

    let host_running = state
        .host_stdin
        .lock()
        .map_err(|_| io::Error::other("host stdin lock poisoned"))?
        .is_some();
    if !host_running {
        spawn_host(Arc::clone(state))?;
    }

    let gateway_connection = state
        .gateway_client
        .lock()
        .ok()
        .and_then(|gateway| gateway.connection_for_dispatch().ok().cloned());
    if let Some(connection) = gateway_connection {
        let generation = state.host_generation.load(Ordering::SeqCst);
        let host_request_id = format!("{}:{request_id}", channel.wire_name());
        state
            .pending
            .lock()
            .map_err(|_| io::Error::other("pending lock poisoned"))?
            .insert(
                host_request_id.clone(),
                PendingHostRequest {
                    generation,
                    request_id: request_id.clone(),
                    channel,
                },
            );

        let dispatch_state = Arc::clone(state);
        thread::spawn(move || {
            let result = dispatch_http_json(&connection, &method, args);
            let pending_request = dispatch_state
                .pending
                .lock()
                .ok()
                .and_then(|mut pending| pending.remove(&host_request_id));
            let Some(pending_request) = pending_request else {
                return;
            };
            match result {
                Ok(value) => dispatch_state.complete_request(
                    pending_request.channel,
                    &pending_request.request_id,
                    ReplyOutcome::Ok { value },
                ),
                Err(error) => {
                    if matches!(
                        error,
                        mahayana_node_agent_coordinator::gateway::gateway_request_dispatcher::GatewayDispatchError::Unreachable { .. }
                    ) {
                        if let mahayana_node_agent_coordinator::gateway::gateway_request_dispatcher::GatewayDispatchError::Unreachable { outcome, .. } = &error {
                            if let Ok(mut gateway) = dispatch_state.gateway_client.lock() {
                                let _ = gateway.transport_down(*outcome);
                            }
                        }
                    }
                    dispatch_state.complete_request(
                        pending_request.channel,
                        &pending_request.request_id,
                        ReplyOutcome::Failed {
                            failure: failure_for(&error),
                        },
                    );
                }
            }
        });
        return Ok(());
    }

    // Compatibility lane while the Host gateway is still starting, and for
    // migration/test Hosts that do not publish Grok gateway discovery yet.
    let mut active = state
        .host_stdin
        .lock()
        .map_err(|_| io::Error::other("host stdin lock poisoned"))?;
    let active = active
        .as_mut()
        .ok_or_else(|| io::Error::other("Host did not provide stdin"))?;

    let host_request_id = format!("{}:{request_id}", channel.wire_name());
    state
        .pending
        .lock()
        .map_err(|_| io::Error::other("pending lock poisoned"))?
        .insert(
            host_request_id.clone(),
            PendingHostRequest {
                generation: active.generation,
                request_id: request_id.clone(),
                channel,
            },
        );

    let host_request = json!({
        "id": host_request_id,
        "method": method,
        "params": args,
    });
    serde_json::to_writer(&mut active.stdin, &host_request)
        .map_err(|error| io::Error::other(format!("Host request serialization failed: {error}")))?;
    writeln!(active.stdin)?;
    active.stdin.flush()
}

fn execute_actions(
    state: &Arc<CoordinatorState>,
    channel: CarrierChannel,
    actions: Vec<ServerAction>,
) -> bool {
    let mut close = false;
    for action in actions {
        match action {
            ServerAction::Post(frame) => {
                let _ = state.write_frame(channel, &frame);
            }
            ServerAction::Dispatch {
                request_id,
                method,
                args,
            } => {
                if let Err(error) =
                    dispatch_to_host(state, channel, request_id.clone(), method, args)
                {
                    state.complete_request(
                        channel,
                        &request_id,
                        ReplyOutcome::Failed {
                            failure: Failure::new(
                                "COORDINATOR_HOST_DISPATCH_FAILED",
                                error.to_string(),
                            ),
                        },
                    );
                }
            }
            ServerAction::Abort { request_id } => {
                state.abort_request(channel, &request_id);
            }
            ServerAction::Close => close = true,
        }
    }
    close
}

fn execute_control_actions(state: &Arc<CoordinatorState>, actions: Vec<ClientAction>) -> bool {
    let mut close = false;
    for action in actions {
        match action {
            ClientAction::Post(frame) => {
                let _ = state.write_frame(CarrierChannel::Control, &frame);
            }
            ClientAction::Close => close = true,
            ClientAction::Resolve { .. }
            | ClientAction::Reject { .. }
            | ClientAction::Event { .. } => {}
        }
    }
    close
}

fn main() {
    let arguments = env::args().collect::<Vec<_>>();
    let bootstrap = match parse_bootstrap_argument(arguments.iter().map(String::as_str)) {
        Ok(bootstrap) => bootstrap,
        Err(detail) => {
            eprintln!("node-agent-coordinator: {detail}");
            std::process::exit(2);
        }
    };

    let host_bin = match env::var_os("MAHAYANA_APP_HOST_BIN").filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => {
            eprintln!("MAHAYANA_APP_HOST_BIN is required");
            std::process::exit(2);
        }
    };

    let gateway_discovery_path =
        PathBuf::from(bootstrap.process_config.data_dir.trim()).join("gateway.json");
    let gateway_client = CoordinatorGatewayClient::default();

    let state = Arc::new(CoordinatorState {
        bootstrap,
        host_bin,
        gateway_discovery_path,
        host_stdin: Mutex::new(None),
        gateway_client: Mutex::new(gateway_client),
        pending: Mutex::new(HashMap::new()),
        control_port: Mutex::new(ControlPortClient::default()),
        renderer_port: Mutex::new(RendererPortServer::default()),
        main_data_port: Mutex::new(RendererPortServer::default()),
        stdout_lock: Mutex::new(()),
        spawn_lock: Mutex::new(()),
        host_generation: AtomicU64::new(0),
        lifecycle_sequence: AtomicU64::new(0),
        closed: AtomicBool::new(false),
        consecutive_crashes: AtomicU64::new(0),
    });

    if let Ok(control) = state.control_port.lock() {
        if let ClientAction::Post(frame) = control.start() {
            if state.write_frame(CarrierChannel::Control, &frame).is_err() {
                std::process::exit(1);
            }
        }
    }

    for line in io::stdin().lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("coordinator stdin read failed: {error}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let envelope = match serde_json::from_str::<CarrierEnvelope>(&line) {
            Ok(envelope) => envelope,
            Err(error) => {
                eprintln!("node-agent-coordinator: invalid carrier envelope: {error}");
                break;
            }
        };
        let Some(channel) = envelope.classify() else {
            eprintln!("node-agent-coordinator: unknown carrier channel {}", envelope.channel);
            break;
        };

        let close = match channel {
            CarrierChannel::Control => {
                let actions = state
                    .control_port
                    .lock()
                    .map(|mut client| client.handle_value(envelope.frame))
                    .unwrap_or_default();
                execute_control_actions(&state, actions)
            }
            CarrierChannel::Data => {
                let actions = state
                    .renderer_port
                    .lock()
                    .map(|mut server| server.handle_value(envelope.frame))
                    .unwrap_or_default();
                execute_actions(&state, CarrierChannel::Data, actions)
            }
            CarrierChannel::MainData => {
                let actions = state
                    .main_data_port
                    .lock()
                    .map(|mut server| server.handle_value(envelope.frame))
                    .unwrap_or_default();
                execute_actions(&state, CarrierChannel::MainData, actions)
            }
        };
        if close {
            break;
        }
    }

    state.closed.store(true, Ordering::SeqCst);
    if let Ok(mut control) = state.control_port.lock() {
        let _ = control.handle_port_closed();
    }
    if let Ok(mut server) = state.renderer_port.lock() {
        let _ = server.handle_port_closed();
    }
    if let Ok(mut server) = state.main_data_port.lock() {
        let _ = server.handle_port_closed();
    }
    if let Ok(mut active) = state.host_stdin.lock() {
        active.take();
    }
    let generation = state.host_generation.load(Ordering::SeqCst);
    state.reject_generation(generation, "Coordinator input closed");
}
