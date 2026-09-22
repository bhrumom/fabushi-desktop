use chrono::{SecondsFormat, Utc};
use mahayana_node_agent_coordinator::carrier::{parse_bootstrap_argument, CoordinatorBootstrap};
use mahayana_node_agent_coordinator::protocol::{
    CoordinatorFrame, Failure, ReplyOutcome, COORDINATOR_DISCONNECTED,
};
use mahayana_node_agent_coordinator::renderer_port_server::{
    RendererPortServer, ServerAction,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
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
}

#[derive(Debug)]
struct CoordinatorState {
    bootstrap: CoordinatorBootstrap,
    host_bin: PathBuf,
    host_stdin: Mutex<Option<ActiveHostStdin>>,
    pending: Mutex<HashMap<String, PendingHostRequest>>,
    renderer_port: Mutex<RendererPortServer>,
    stdout_lock: Mutex<()>,
    spawn_lock: Mutex<()>,
    host_generation: AtomicU64,
    lifecycle_sequence: AtomicU64,
    closed: AtomicBool,
    consecutive_crashes: AtomicU64,
}

impl CoordinatorState {
    fn write_frame(&self, frame: &CoordinatorFrame) -> io::Result<()> {
        let _guard = self.stdout_lock.lock().map_err(|_| io::Error::other("stdout lock poisoned"))?;
        let mut stdout = io::stdout().lock();
        serde_json::to_writer(&mut stdout, frame)
            .map_err(|error| io::Error::other(format!("coordinator frame serialization failed: {error}")))?;
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
            let _ = self.write_frame(&frame);
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

    fn complete_request(&self, request_id: &str, outcome: ReplyOutcome) {
        let actions = self
            .renderer_port
            .lock()
            .map(|mut server| server.complete_request(request_id, outcome))
            .unwrap_or_default();
        for action in actions {
            if let ServerAction::Post(frame) = action {
                let _ = self.write_frame(&frame);
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
                &request.request_id,
                ReplyOutcome::Failed {
                    failure: Failure::new(COORDINATOR_DISCONNECTED, message),
                },
            );
        }
    }

    fn abort_request(&self, request_id: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(request_id);
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
    let mut child = Command::new(&state.host_bin)
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
                let request_id = id.as_str().map(str::to_owned).unwrap_or_else(|| id.to_string());
                let pending = output_state
                    .pending
                    .lock()
                    .ok()
                    .and_then(|mut pending| pending.remove(&request_id));
                if pending.is_none() {
                    continue;
                }

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
                output_state.complete_request(&request_id, outcome);
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
    request_id: String,
    method: String,
    args: Value,
) -> io::Result<()> {
    if method == "coordinator.health" {
        let generation = state.host_generation.load(Ordering::SeqCst);
        let pending = state.pending.lock().map(|pending| pending.len()).unwrap_or_default();
        state.complete_request(
            &request_id,
            ReplyOutcome::Ok {
                value: json!({
                    "protocolVersion": 1,
                    "hostGeneration": generation,
                    "pending": pending,
                    "hostRunning": state.host_stdin.lock().map(|host| host.is_some()).unwrap_or(false),
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

    let mut active = state
        .host_stdin
        .lock()
        .map_err(|_| io::Error::other("host stdin lock poisoned"))?;
    if active.is_none() {
        drop(active);
        spawn_host(Arc::clone(state))?;
        active = state
            .host_stdin
            .lock()
            .map_err(|_| io::Error::other("host stdin lock poisoned"))?;
    }
    let active = active
        .as_mut()
        .ok_or_else(|| io::Error::other("Host did not provide stdin"))?;

    state
        .pending
        .lock()
        .map_err(|_| io::Error::other("pending lock poisoned"))?
        .insert(
            request_id.clone(),
            PendingHostRequest {
                generation: active.generation,
                request_id: request_id.clone(),
            },
        );

    let host_request = json!({
        "id": request_id,
        "method": method,
        "params": args,
    });
    serde_json::to_writer(&mut active.stdin, &host_request)
        .map_err(|error| io::Error::other(format!("Host request serialization failed: {error}")))?;
    writeln!(active.stdin)?;
    active.stdin.flush()
}

fn execute_actions(state: &Arc<CoordinatorState>, actions: Vec<ServerAction>) -> bool {
    let mut close = false;
    for action in actions {
        match action {
            ServerAction::Post(frame) => {
                let _ = state.write_frame(&frame);
            }
            ServerAction::Dispatch {
                request_id,
                method,
                args,
            } => {
                if let Err(error) =
                    dispatch_to_host(state, request_id.clone(), method, args)
                {
                    state.complete_request(
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
                state.abort_request(&request_id);
            }
            ServerAction::Close => close = true,
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

    let state = Arc::new(CoordinatorState {
        bootstrap,
        host_bin,
        host_stdin: Mutex::new(None),
        pending: Mutex::new(HashMap::new()),
        renderer_port: Mutex::new(RendererPortServer::default()),
        stdout_lock: Mutex::new(()),
        spawn_lock: Mutex::new(()),
        host_generation: AtomicU64::new(0),
        lifecycle_sequence: AtomicU64::new(0),
        closed: AtomicBool::new(false),
        consecutive_crashes: AtomicU64::new(0),
    });

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

        let value = match serde_json::from_str::<Value>(&line) {
            Ok(value) => value,
            Err(error) => {
                let actions = state
                    .renderer_port
                    .lock()
                    .map(|mut server| {
                        server.handle_value(json!({
                            "kind": "invalid",
                            "detail": error.to_string()
                        }))
                    })
                    .unwrap_or_default();
                if execute_actions(&state, actions) {
                    break;
                }
                continue;
            }
        };

        let actions = state
            .renderer_port
            .lock()
            .map(|mut server| server.handle_value(value))
            .unwrap_or_default();
        if execute_actions(&state, actions) {
            break;
        }
    }

    state.closed.store(true, Ordering::SeqCst);
    if let Ok(mut active) = state.host_stdin.lock() {
        active.take();
    }
    let generation = state.host_generation.load(Ordering::SeqCst);
    state.reject_generation(generation, "Coordinator input closed");
}
