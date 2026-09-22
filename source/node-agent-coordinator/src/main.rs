use chrono::{SecondsFormat, Utc};
use mahayana_node_agent_coordinator::carrier::{
    parse_bootstrap_argument, CarrierChannel, CarrierEnvelope, CoordinatorBootstrap,
};
use mahayana_node_agent_coordinator::control_port_client::{
    ClientAction, ControlPortClient, ControlPortPhase,
};
use mahayana_node_agent_coordinator::client_side_tool_v2_relay::{
    ClientSideToolV2Relay, CLIENT_SIDE_TOOL_V2_FAMILY,
};
use mahayana_node_agent_coordinator::gateway::gateway_client::{
    CoordinatorGatewayClient, stream_http_events,
};
use mahayana_node_agent_coordinator::gateway::gateway_event_families::coordinator_event_family_for_sse_channel;
use mahayana_node_agent_coordinator::gateway::gateway_reachability::ReachabilityOutcome;
use mahayana_node_agent_coordinator::gateway::gateway_request_dispatcher::{
    GatewayDispatchError, dispatch_http_json, failure_for,
};
use mahayana_node_agent_coordinator::gateway::host_supervisor::{
    GatewayConnection, GatewayHostSupervisor, HEALTH_PROBE_TTL_MS, read_gateway_discovery,
};
use mahayana_node_agent_coordinator::oauth::mcp_oauth_callback_listener::{
    McpOAuthCallbackListener, OAuthCallback, start_mcp_oauth_callback_listener,
};
use mahayana_node_agent_coordinator::oauth::mcp_oauth_forwarder::{
    McpOAuthForwarderState, McpOAuthPendingPayload, OAuthForwarderAction,
};
use mahayana_node_agent_coordinator::oauth::mcp_oauth_loopback_registry::McpOAuthLoopbackRegistry;
use mahayana_node_agent_coordinator::local_exec::supervisor::{
    ExpectedLocalExecProcessIdentity, LOCAL_EXEC_DAEMON_LIVENESS_INTERVAL_MS,
    LOCAL_EXEC_DAEMON_REFRESH_INTERVAL_MS, LocalExecControl, LocalExecDaemonRuntime,
    LocalExecProcessIdentity, LocalExecSupervisorError, SpawnLocalExecDaemonRequest,
};
use mahayana_node_agent_coordinator::protocol::{
    CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, COORDINATOR_DISCONNECTED,
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
    mpsc::{self, RecvTimeoutError, Sender},
};
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalExecRuntimeCommand {
    Refresh,
    SetPaused(bool),
    Dispose,
}

struct CoordinatorState {
    bootstrap: CoordinatorBootstrap,
    host_bin: PathBuf,
    gateway_discovery_path: PathBuf,
    host_stdin: Mutex<Option<ActiveHostStdin>>,
    gateway_client: Mutex<CoordinatorGatewayClient>,
    host_supervisor: Mutex<GatewayHostSupervisor>,
    tool_relay: Mutex<ClientSideToolV2Relay>,
    tool_replay_done: AtomicBool,
    oauth_forwarder: Mutex<McpOAuthForwarderState>,
    oauth_loopback: Mutex<McpOAuthLoopbackRegistry>,
    oauth_listeners: Mutex<HashMap<String, McpOAuthCallbackListener>>,
    pending: Mutex<HashMap<String, PendingHostRequest>>,
    control_port: Mutex<ControlPortClient>,
    local_exec_commands: Mutex<Option<Sender<LocalExecRuntimeCommand>>>,
    renderer_port: Mutex<RendererPortServer>,
    main_data_port: Mutex<RendererPortServer>,
    stdout_lock: Mutex<()>,
    spawn_lock: Mutex<()>,
    host_generation: AtomicU64,
    lifecycle_sequence: AtomicU64,
    closed: AtomicBool,
    consecutive_crashes: AtomicU64,
    gateway_events_live: AtomicBool,
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


    fn gateway_lifecycle(&self, lifecycle: &str, generation: u64, detail: Option<&str>) {
        let sequence = self.lifecycle_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let mut event = json!({
            "type": "gateway.lifecycle",
            "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            "lifecycle": lifecycle,
            "generation": generation,
            "sequence": sequence,
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


fn control_command(
    state: &Arc<CoordinatorState>,
    method: &str,
    args: Value,
) -> Result<Value, Failure> {
    let waiter = state
        .control_port
        .lock()
        .map_err(|_| Failure::new(COORDINATOR_DISCONNECTED, "control port lock poisoned"))?
        .call_waiting(method.to_string(), args)?;
    let frame = match waiter.action {
        ClientAction::Post(frame) => frame,
        _ => {
            return Err(Failure::new(
                "COORDINATOR_CONTROL_PROTOCOL_ERROR",
                "control command did not produce a request frame",
            ));
        }
    };
    state
        .write_frame(CarrierChannel::Control, &frame)
        .map_err(|error| {
            Failure::new(
                COORDINATOR_DISCONNECTED,
                format!("control request write failed: {error}"),
            )
        })?;
    waiter
        .reply
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| {
            Failure::new(
                COORDINATOR_DISCONNECTED,
                format!("control request {method} did not settle: {error}"),
            )
        })?
}

fn local_exec_error(error: Failure) -> LocalExecSupervisorError {
    LocalExecSupervisorError::new(format!("{}: {}", error.code, error.message))
}

fn parse_u64_field(value: &Value, field: &str) -> Result<u64, LocalExecSupervisorError> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| LocalExecSupervisorError::new(format!("missing {field}")))
}

fn parse_string_field(value: &Value, field: &str) -> Result<String, LocalExecSupervisorError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| LocalExecSupervisorError::new(format!("missing {field}")))
}

fn parse_local_exec_identity(value: Value) -> Result<LocalExecProcessIdentity, LocalExecSupervisorError> {
    let pid = u32::try_from(parse_u64_field(&value, "pid")?)
        .map_err(|_| LocalExecSupervisorError::new("local-exec pid is out of range"))?;
    Ok(LocalExecProcessIdentity {
        pid,
        start_epoch_ms: parse_u64_field(&value, "startEpochMs")?,
        command: parse_string_field(&value, "command")?,
        entry_realpath: parse_string_field(&value, "entryRealpath")?,
        generation_token: parse_string_field(&value, "generationToken")?,
    })
}

fn local_exec_identity_json(identity: &LocalExecProcessIdentity) -> Value {
    json!({
        "pid": identity.pid,
        "startEpochMs": identity.start_epoch_ms,
        "command": identity.command,
        "entryRealpath": identity.entry_realpath,
        "generationToken": identity.generation_token,
    })
}

struct CoordinatorLocalExecControl {
    state: Arc<CoordinatorState>,
}

impl LocalExecControl for CoordinatorLocalExecControl {
    fn now_ms(&self) -> u64 {
        coordinator_now_ms()
    }

    fn sleep_ms(&mut self, duration_ms: u64) {
        thread::sleep(Duration::from_millis(duration_ms));
    }

    fn resolve_gateway_connection(&mut self) -> Result<Value, LocalExecSupervisorError> {
        control_command(&self.state, "resolveGatewayConnection", json!({}))
            .map_err(local_exec_error)
    }

    fn mint_local_exec_daemon_credential(
        &mut self,
    ) -> Result<Option<Value>, LocalExecSupervisorError> {
        let value = control_command(&self.state, "mintLocalExecDaemonCredential", json!({}))
            .map_err(local_exec_error)?;
        Ok((!value.is_null()).then_some(value))
    }

    fn spawn_local_exec_daemon(
        &mut self,
        request: SpawnLocalExecDaemonRequest,
    ) -> Result<LocalExecProcessIdentity, LocalExecSupervisorError> {
        let value = control_command(
            &self.state,
            "spawnLocalExecDaemon",
            json!({
                "env": request.env,
                "logPath": request.log_path.to_string_lossy(),
            }),
        )
        .map_err(local_exec_error)?;
        parse_local_exec_identity(value)
    }

    fn is_process_alive(&mut self, pid: u32) -> Result<bool, LocalExecSupervisorError> {
        control_command(&self.state, "isProcessAlive", json!({ "pid": pid }))
            .map_err(local_exec_error)?
            .as_bool()
            .ok_or_else(|| LocalExecSupervisorError::new("isProcessAlive returned a non-boolean"))
    }

    fn get_process_identity(
        &mut self,
        expected: &ExpectedLocalExecProcessIdentity,
    ) -> Result<Option<LocalExecProcessIdentity>, LocalExecSupervisorError> {
        let value = control_command(
            &self.state,
            "getProcessIdentity",
            json!({
                "pid": expected.pid,
                "entryRealpath": expected.entry_realpath,
                "generationToken": expected.generation_token,
                "startEpochMs": expected.start_epoch_ms,
                "command": expected.command,
                "discoveryStartedAt": expected.discovery_started_at,
            }),
        )
        .map_err(local_exec_error)?;
        if value.is_null() {
            return Ok(None);
        }
        parse_local_exec_identity(value).map(Some)
    }

    fn terminate_process(
        &mut self,
        identity: &LocalExecProcessIdentity,
    ) -> Result<bool, LocalExecSupervisorError> {
        let value = control_command(
            &self.state,
            "terminateProcess",
            json!({ "identity": local_exec_identity_json(identity) }),
        )
        .map_err(local_exec_error)?;
        value
            .get("terminated")
            .and_then(Value::as_bool)
            .ok_or_else(|| LocalExecSupervisorError::new("terminateProcess returned no terminated flag"))
    }
}

fn signal_local_exec(state: &Arc<CoordinatorState>, command: LocalExecRuntimeCommand) {
    let sender = state
        .local_exec_commands
        .lock()
        .ok()
        .and_then(|sender| sender.clone());
    if let Some(sender) = sender {
        let _ = sender.send(command);
    }
}

fn ensure_local_exec_runtime_started(state: &Arc<CoordinatorState>) {
    let mut slot = match state.local_exec_commands.lock() {
        Ok(slot) => slot,
        Err(_) => return,
    };
    if slot.is_some() {
        return;
    }
    let (sender, receiver) = mpsc::channel();
    *slot = Some(sender);
    drop(slot);

    let runtime_state = Arc::clone(state);
    let data_dir = runtime_state.bootstrap.process_config.data_dir.clone();
    let is_packaged = runtime_state.bootstrap.process_config.is_packaged;
    thread::spawn(move || {
        let control = CoordinatorLocalExecControl {
            state: Arc::clone(&runtime_state),
        };
        let mut runtime = LocalExecDaemonRuntime::new(data_dir, is_packaged, control);
        runtime.start();
        let mut last_refresh = Instant::now();
        loop {
            if runtime_state.closed.load(Ordering::SeqCst) {
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(
                LOCAL_EXEC_DAEMON_LIVENESS_INTERVAL_MS,
            )) {
                Ok(LocalExecRuntimeCommand::Refresh) => runtime.refresh_tick(),
                Ok(LocalExecRuntimeCommand::SetPaused(paused)) => runtime.set_paused(paused),
                Ok(LocalExecRuntimeCommand::Dispose) => break,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            runtime.liveness_tick();
            if last_refresh.elapsed()
                >= Duration::from_millis(LOCAL_EXEC_DAEMON_REFRESH_INTERVAL_MS)
            {
                runtime.refresh_tick();
                last_refresh = Instant::now();
            }
        }
        runtime.dispose();
    });
}

fn coordinator_now_ms() -> u64 {
    u64::try_from(Utc::now().timestamp_millis()).unwrap_or_default()
}

fn close_oauth_listener(state: &Arc<CoordinatorState>, origin: &str) {
    let listener = state
        .oauth_listeners
        .lock()
        .ok()
        .and_then(|mut listeners| listeners.remove(origin));
    if let Some(listener) = listener {
        if let Ok(mut registry) = state.oauth_loopback.lock() {
            let _ = listener.close(&mut registry);
        }
    }
}

fn complete_oauth_callback(
    state: &Arc<CoordinatorState>,
    callback: OAuthCallback,
) -> Result<(), Failure> {
    let generation = state.host_generation.load(Ordering::SeqCst);
    let connection = wait_for_gateway_connection(state, generation).map_err(|error| {
        Failure::new(
            "MCP_OAUTH_COMPLETION_UNAVAILABLE",
            format!("Host gateway unavailable during OAuth completion: {error}"),
        )
    })?;
    dispatch_http_json(
        &connection,
        "completeMcpOAuth",
        json!({
            "code": callback.code,
            "state": callback.state,
        }),
    )
    .map(|_| ())
    .map_err(|error| failure_for(&error))
}

fn apply_oauth_actions(state: &Arc<CoordinatorState>, actions: Vec<OAuthForwarderAction>) {
    for action in actions {
        match action {
            OAuthForwarderAction::CloseListener { origin } => {
                close_oauth_listener(state, &origin);
            }
            OAuthForwarderAction::Complete { callback } => {
                if let Err(error) = complete_oauth_callback(state, callback) {
                    eprintln!(
                        "node-agent-coordinator: mcp-oauth completion failed: {}",
                        error.message
                    );
                }
            }
            OAuthForwarderAction::StartListener {
                origin,
                redirect_url,
                state: pending_state,
            } => {
                let resolve_state = Arc::clone(state);
                let resolve_origin = origin.clone();
                let callback_state = Arc::clone(state);
                let settled_state = Arc::clone(state);
                let settled_origin = origin.clone();

                let listener = {
                    let mut registry = match state.oauth_loopback.lock() {
                        Ok(registry) => registry,
                        Err(_) => {
                            let actions = state
                                .oauth_forwarder
                                .lock()
                                .map(|mut forwarder| {
                                    forwarder.listener_failed(
                                        &origin,
                                        &pending_state,
                                        coordinator_now_ms(),
                                    )
                                })
                                .unwrap_or_default();
                            apply_oauth_actions(state, actions);
                            continue;
                        }
                    };
                    start_mcp_oauth_callback_listener(
                        &redirect_url,
                        &mut registry,
                        move |callback_state_value| {
                            resolve_state
                                .oauth_forwarder
                                .lock()
                                .ok()
                                .and_then(|forwarder| {
                                    forwarder
                                        .resolve_server(
                                            &resolve_origin,
                                            callback_state_value,
                                            coordinator_now_ms(),
                                        )
                                        .map(str::to_string)
                                })
                        },
                        move |callback| complete_oauth_callback(&callback_state, callback),
                        move |callback_state_value| {
                            let state_value = callback_state_value.to_string();
                            let settle_state = Arc::clone(&settled_state);
                            let settle_origin = settled_origin.clone();
                            thread::spawn(move || {
                                let actions = settle_state
                                    .oauth_forwarder
                                    .lock()
                                    .map(|mut forwarder| {
                                        forwarder.settle_callback(
                                            &settle_origin,
                                            &state_value,
                                            coordinator_now_ms(),
                                        )
                                    })
                                    .unwrap_or_default();
                                apply_oauth_actions(&settle_state, actions);
                            });
                        },
                    )
                };

                match listener {
                    Ok(listener) => {
                        if let Ok(mut listeners) = state.oauth_listeners.lock() {
                            if let Some(previous) = listeners.insert(origin.clone(), listener) {
                                if let Ok(mut registry) = state.oauth_loopback.lock() {
                                    let _ = previous.close(&mut registry);
                                }
                            }
                        }
                        let actions = state
                            .oauth_forwarder
                            .lock()
                            .map(|mut forwarder| {
                                forwarder.listener_started(&origin, coordinator_now_ms())
                            })
                            .unwrap_or_default();
                        apply_oauth_actions(state, actions);
                    }
                    Err(error) => {
                        eprintln!(
                            "node-agent-coordinator: mcp-oauth listener start failed for {}: {}",
                            origin,
                            error.message
                        );
                        let actions = state
                            .oauth_forwarder
                            .lock()
                            .map(|mut forwarder| {
                                forwarder.listener_failed(
                                    &origin,
                                    &pending_state,
                                    coordinator_now_ms(),
                                )
                            })
                            .unwrap_or_default();
                        apply_oauth_actions(state, actions);
                    }
                }
            }
        }
    }
}

fn handle_oauth_pending(state: &Arc<CoordinatorState>, payload: &Value) {
    let Some(redirect_url) = payload.get("redirectUrl").and_then(Value::as_str) else {
        eprintln!("node-agent-coordinator: mcp-oauth pending event missing redirectUrl");
        return;
    };
    let Some(oauth_state) = payload.get("state").and_then(Value::as_str) else {
        eprintln!("node-agent-coordinator: mcp-oauth pending event missing state");
        return;
    };
    let Some(server_name) = payload.get("serverName").and_then(Value::as_str) else {
        eprintln!("node-agent-coordinator: mcp-oauth pending event missing serverName");
        return;
    };
    let actions = state
        .oauth_forwarder
        .lock()
        .map_err(|_| ())
        .and_then(|mut forwarder| {
            forwarder
                .handle_pending(
                    McpOAuthPendingPayload {
                        redirect_url: redirect_url.to_string(),
                        state: oauth_state.to_string(),
                        server_name: server_name.to_string(),
                    },
                    coordinator_now_ms(),
                )
                .map_err(|error| {
                    eprintln!(
                        "node-agent-coordinator: invalid mcp-oauth pending event: {}",
                        error.message
                    );
                })
        })
        .unwrap_or_default();
    apply_oauth_actions(state, actions);
}

fn start_oauth_expiry_loop(state: Arc<CoordinatorState>) {
    thread::spawn(move || {
        while !state.closed.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            let actions = state
                .oauth_forwarder
                .lock()
                .map(|mut forwarder| forwarder.expire(coordinator_now_ms()))
                .unwrap_or_default();
            apply_oauth_actions(&state, actions);
        }
    });
}

fn post_tool_event(state: &Arc<CoordinatorState>, event: mahayana_node_agent_coordinator::client_side_tool_v2_relay::RendererToolEvent) {
    if let Ok(payload) = serde_json::to_value(event) {
        state.post_event(CLIENT_SIDE_TOOL_V2_FAMILY, payload);
    }
}

fn replay_tool_events(state: &Arc<CoordinatorState>) {
    let events = state
        .tool_relay
        .lock()
        .map(|relay| relay.replay())
        .unwrap_or_default();
    for event in events {
        post_tool_event(state, event);
    }
}

fn dispatch_gateway_event(state: &Arc<CoordinatorState>, value: Value) {
    let (channel, family, payload) = match (
        value.get("channel").and_then(Value::as_str),
        value.get("payload"),
    ) {
        (Some(channel), Some(payload)) => (
            channel.to_string(),
            coordinator_event_family_for_sse_channel(channel)
                .unwrap_or("runtime")
                .to_string(),
            payload.clone(),
        ),
        _ => ("runtime".into(), "runtime".into(), value),
    };
    let now_ms = coordinator_now_ms();
    if let Ok(mut gateway) = state.gateway_client.lock() {
        let _ = gateway.accept_event(now_ms, channel.clone(), payload.clone());
    }

    if channel == "mcp-oauth-pending" {
        handle_oauth_pending(state, &payload);
        return;
    }

    if channel == CLIENT_SIDE_TOOL_V2_FAMILY {
        let projected = state
            .tool_relay
            .lock()
            .ok()
            .and_then(|mut relay| relay.accept_value(payload));
        if let Some(event) = projected {
            post_tool_event(state, event);
        }
        return;
    }

    state.post_event(&family, payload);
}

fn run_gateway_event_stream(
    state: Arc<CoordinatorState>,
    generation: u64,
    mut connection: GatewayConnection,
) {
    loop {
        if state.closed.load(Ordering::SeqCst)
            || state.host_generation.load(Ordering::SeqCst) != generation
        {
            state.gateway_events_live.store(false, Ordering::SeqCst);
            return;
        }

        let on_connected_state = Arc::clone(&state);
        let on_event_state = Arc::clone(&state);
        let continue_state = Arc::clone(&state);
        let result = stream_http_events(
            &connection,
            move || {
                on_connected_state
                    .gateway_events_live
                    .store(true, Ordering::SeqCst);
                let now_ms =
                    u64::try_from(Utc::now().timestamp_millis()).unwrap_or_default();
                if let Ok(mut gateway) = on_connected_state.gateway_client.lock() {
                    let _ = gateway.start(now_ms);
                }
                if let Ok(mut supervisor) = on_connected_state.host_supervisor.lock() {
                    supervisor.mark_transport_live(true);
                    supervisor.record_health(now_ms, true);
                }
                signal_local_exec(&on_connected_state, LocalExecRuntimeCommand::Refresh);
                on_connected_state.gateway_lifecycle(
                    "transport-connected",
                    generation,
                    None,
                );
            },
            move |event| dispatch_gateway_event(&on_event_state, event),
            move || {
                !continue_state.closed.load(Ordering::SeqCst)
                    && continue_state.host_generation.load(Ordering::SeqCst) == generation
            },
        );

        state.gateway_events_live.store(false, Ordering::SeqCst);
        if state.closed.load(Ordering::SeqCst)
            || state.host_generation.load(Ordering::SeqCst) != generation
        {
            return;
        }

        let outcome = match &result {
            Err(GatewayDispatchError::Unreachable { outcome, .. }) => *outcome,
            Err(GatewayDispatchError::Command(_)) => ReachabilityOutcome::AccessDenied,
            Err(GatewayDispatchError::Transport(_)) | Ok(()) => ReachabilityOutcome::Network,
        };
        let detail = result
            .err()
            .map(|error| error.to_string())
            .unwrap_or_else(|| "gateway event stream ended".into());
        let delay = state
            .gateway_client
            .lock()
            .ok()
            .and_then(|mut gateway| gateway.transport_down(outcome));
        if let Ok(mut supervisor) = state.host_supervisor.lock() {
            supervisor.mark_transport_live(false);
            supervisor.invalidate();
        }
        state.gateway_lifecycle("transport-down", generation, Some(&detail));
        let Some(delay) = delay else {
            return;
        };
        thread::sleep(delay);

        if state.closed.load(Ordering::SeqCst)
            || state.host_generation.load(Ordering::SeqCst) != generation
        {
            return;
        }
        match read_gateway_discovery(&state.gateway_discovery_path) {
            Ok(next_connection) => {
                if let Ok(mut supervisor) = state.host_supervisor.lock() {
                    let attempt = supervisor.begin_connection_attempt();
                    let _ = supervisor.settle_connection_attempt(attempt, next_connection.clone());
                }
                if let Ok(mut gateway) = state.gateway_client.lock() {
                    let _ = gateway.install_connection(next_connection.clone());
                }
                connection = next_connection;
            }
            Err(error) => {
                state.gateway_lifecycle(
                    "rediscovery-failed",
                    generation,
                    Some(&error.to_string()),
                );
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
    if let Ok(mut supervisor) = state.host_supervisor.lock() {
        supervisor.invalidate();
    }
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
                    let supervised = discovery_state
                        .host_supervisor
                        .lock()
                        .ok()
                        .is_some_and(|mut supervisor| {
                            let attempt = supervisor.begin_connection_attempt();
                            supervisor
                                .settle_connection_attempt(attempt, connection.clone())
                                .is_ok()
                        });
                    let installed = supervised
                        && discovery_state
                            .gateway_client
                            .lock()
                            .ok()
                            .is_some_and(|mut gateway| {
                                gateway.install_connection(connection.clone()).is_ok()
                            });
                    if installed {
                        discovery_state.gateway_lifecycle(
                            "discovered",
                            generation,
                            None,
                        );
                        run_gateway_event_stream(
                            Arc::clone(&discovery_state),
                            generation,
                            connection,
                        );
                        return;
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            discovery_state.gateway_lifecycle(
                "discovery-timeout",
                generation,
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
                    if !output_state.gateway_events_live.load(Ordering::SeqCst) {
                        output_state.post_event("runtime", event.clone());
                    }
                    continue;
                }

                output_state.lifecycle(
                    "protocol-error",
                    generation,
                    true,
                    Some(
                        "unexpected non-event Host stdout frame; Coordinator business requests must use the Host gateway",
                    ),
                );
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
        if let Ok(mut supervisor) = state.host_supervisor.lock() {
            supervisor.mark_transport_live(false);
            supervisor.invalidate();
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

fn wait_for_gateway_connection(
    state: &Arc<CoordinatorState>,
    generation: u64,
) -> io::Result<GatewayConnection> {
    for _ in 0..500 {
        if state.closed.load(Ordering::SeqCst) {
            return Err(io::Error::other("coordinator closed while waiting for Host gateway"));
        }
        if state.host_generation.load(Ordering::SeqCst) != generation {
            return Err(io::Error::other("Host generation changed while waiting for gateway"));
        }
        if let Some(connection) = state
            .host_supervisor
            .lock()
            .ok()
            .and_then(|supervisor| supervisor.connection().cloned())
        {
            return Ok(connection);
        }
        if let Some(connection) = state
            .gateway_client
            .lock()
            .ok()
            .and_then(|gateway| gateway.state.connection.clone())
        {
            return Ok(connection);
        }
        // The discovery worker normally installs this asynchronously. Reading
        // it here as well removes the historical stdin business-request
        // fallback without introducing a startup race for the first command.
        if let Ok(connection) = read_gateway_discovery(&state.gateway_discovery_path) {
            if let Ok(mut supervisor) = state.host_supervisor.lock() {
                let attempt = supervisor.begin_connection_attempt();
                let _ = supervisor.settle_connection_attempt(attempt, connection.clone());
            }
            if let Ok(mut gateway) = state.gateway_client.lock() {
                let _ = gateway.install_connection(connection.clone());
            }
            return Ok(connection);
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Host gateway discovery did not become ready before command dispatch",
    ))
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

    let generation = state.host_generation.load(Ordering::SeqCst);
    let connection = wait_for_gateway_connection(state, generation)?;
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
        if matches!(result, Err(GatewayDispatchError::Unreachable { .. })) {
            // A Host crash closes the in-flight HTTP socket before the process
            // waiter can reject its generation. Give that owner a short window
            // to settle the request as COORDINATOR_DISCONNECTED. If the Host is
            // still alive, this is a genuine gateway/network failure and the
            // dispatch worker below owns the reply.
            for _ in 0..20 {
                let host_alive = dispatch_state
                    .host_stdin
                    .lock()
                    .ok()
                    .and_then(|active| active.as_ref().map(|entry| entry.generation))
                    == Some(generation);
                if !host_alive {
                    return;
                }
                thread::sleep(Duration::from_millis(5));
            }
        }
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
                if let GatewayDispatchError::Unreachable { outcome, .. } = &error {
                    if let Ok(mut gateway) = dispatch_state.gateway_client.lock() {
                        let _ = gateway.transport_down(*outcome);
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
    Ok(())
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
                let serving_ready = channel == CarrierChannel::Data
                    && matches!(
                        &frame,
                        CoordinatorFrame::Lifecycle {
                            phase: LifecyclePhase::Ready,
                            ..
                        }
                    );
                let _ = state.write_frame(channel, &frame);
                if serving_ready
                    && !state.tool_replay_done.swap(true, Ordering::SeqCst)
                {
                    replay_tool_events(state);
                }
            }
            ServerAction::Dispatch {
                request_id,
                method,
                args,
            } => {
                if channel == CarrierChannel::MainData && method == "setGatewayPaused" {
                    let paused = args
                        .get("paused")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    signal_local_exec(state, LocalExecRuntimeCommand::SetPaused(paused));
                }
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
        host_supervisor: Mutex::new(GatewayHostSupervisor::new(HEALTH_PROBE_TTL_MS)),
        tool_relay: Mutex::new(ClientSideToolV2Relay::default()),
        tool_replay_done: AtomicBool::new(false),
        oauth_forwarder: Mutex::new(McpOAuthForwarderState::default()),
        oauth_loopback: Mutex::new(McpOAuthLoopbackRegistry::default()),
        oauth_listeners: Mutex::new(HashMap::new()),
        pending: Mutex::new(HashMap::new()),
        control_port: Mutex::new(ControlPortClient::default()),
        local_exec_commands: Mutex::new(None),
        renderer_port: Mutex::new(RendererPortServer::default()),
        main_data_port: Mutex::new(RendererPortServer::default()),
        stdout_lock: Mutex::new(()),
        spawn_lock: Mutex::new(()),
        host_generation: AtomicU64::new(0),
        lifecycle_sequence: AtomicU64::new(0),
        closed: AtomicBool::new(false),
        consecutive_crashes: AtomicU64::new(0),
        gateway_events_live: AtomicBool::new(false),
    });

    start_oauth_expiry_loop(Arc::clone(&state));

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
                let (actions, serving) = state
                    .control_port
                    .lock()
                    .map(|mut client| {
                        let actions = client.handle_value(envelope.frame);
                        let serving = client.phase() == ControlPortPhase::Serving;
                        (actions, serving)
                    })
                    .unwrap_or_default();
                let close = execute_control_actions(&state, actions);
                if serving {
                    ensure_local_exec_runtime_started(&state);
                }
                close
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
    signal_local_exec(&state, LocalExecRuntimeCommand::Dispose);
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
    if let Ok(mut relay) = state.tool_relay.lock() {
        relay.clear();
    }
    let oauth_actions = state
        .oauth_forwarder
        .lock()
        .map(|mut forwarder| forwarder.dispose())
        .unwrap_or_default();
    apply_oauth_actions(&state, oauth_actions);
    if let Ok(mut registry) = state.oauth_loopback.lock() {
        registry.dispose();
    }
    let generation = state.host_generation.load(Ordering::SeqCst);
    state.reject_generation(generation, "Coordinator input closed");
}
