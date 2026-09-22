use chrono::{SecondsFormat, Utc};
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

#[derive(Debug)]
struct ActiveHostStdin {
    generation: u64,
    stdin: ChildStdin,
}

#[derive(Debug)]
struct PendingRequest {
    generation: u64,
    id: Value,
}

#[derive(Debug)]
struct CoordinatorState {
    host_bin: PathBuf,
    host_stdin: Mutex<Option<ActiveHostStdin>>,
    pending: Mutex<HashMap<String, PendingRequest>>,
    stdout_lock: Mutex<()>,
    spawn_lock: Mutex<()>,
    host_generation: AtomicU64,
    lifecycle_sequence: AtomicU64,
    closed: AtomicBool,
    consecutive_crashes: AtomicU64,
}

impl CoordinatorState {
    fn write_json(&self, value: &Value) -> io::Result<()> {
        let _guard = self.stdout_lock.lock().map_err(|_| io::Error::other("stdout lock poisoned"))?;
        let mut stdout = io::stdout().lock();
        serde_json::to_writer(&mut stdout, value)
            .map_err(|error| io::Error::other(format!("coordinator output serialization failed: {error}")))?;
        writeln!(stdout)?;
        stdout.flush()
    }

    fn lifecycle(&self, lifecycle: &str, generation: u64, recoverable: bool, detail: Option<&str>) {
        let sequence = self.lifecycle_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let mut event = json!({
            "event": {
                "type": "host.lifecycle",
                "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                "lifecycle": lifecycle,
                "state": if lifecycle == "running" { "running" } else { "stopped" },
                "generation": generation,
                "sequence": sequence,
                "recoverable": recoverable,
            }
        });
        if let Some(detail) = detail {
            event["event"]["reason"] = Value::String(detail.to_string());
        }
        let _ = self.write_json(&event);
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
            let _ = self.write_json(&json!({
                "id": request.id,
                "ok": false,
                "error": message,
                "coordinator": {
                    "hostGeneration": generation,
                    "settledBy": "host-supervisor"
                }
            }));
        }
    }
}

fn pending_key(id: &Value) -> String {
    serde_json::to_string(id).unwrap_or_else(|_| "null".into())
}

fn spawn_host(state: Arc<CoordinatorState>) -> io::Result<u64> {
    let _spawn_guard = state.spawn_lock.lock().map_err(|_| io::Error::other("spawn lock poisoned"))?;
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
        .map_err(|error| io::Error::other(format!("failed to spawn Mahayana Host {}: {error}", state.host_bin.display())))?;

    let stdin = child.stdin.take().ok_or_else(|| io::Error::other("Host stdin unavailable"))?;
    let stdout = child.stdout.take().ok_or_else(|| io::Error::other("Host stdout unavailable"))?;
    let stderr = child.stderr.take().ok_or_else(|| io::Error::other("Host stderr unavailable"))?;
    *state.host_stdin.lock().map_err(|_| io::Error::other("host stdin lock poisoned"))? =
        Some(ActiveHostStdin { generation, stdin });
    state.consecutive_crashes.store(0, Ordering::SeqCst);
    state.lifecycle("running", generation, true, None);
    // Host spawn serialization is only needed through publication of the active generation.
    // Release the guard before the wait thread takes ownership of the shared state.
    drop(_spawn_guard);

    {
        let output_state = Arc::clone(&state);
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if output_state.host_generation.load(Ordering::SeqCst) != generation {
                    continue;
                }
                let value = match serde_json::from_str::<Value>(&line) {
                    Ok(value) => value,
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
                if let Some(id) = value.get("id") {
                    if let Ok(mut pending) = output_state.pending.lock() {
                        pending.remove(&pending_key(id));
                    }
                }
                let _ = output_state.write_json(&value);
            }
        });
    }

    {
        let error_state = Arc::clone(&state);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                let Ok(line) = line else { break };
                eprintln!("[mahayana-host:generation-{generation}] {line}");
            }
            let _ = error_state;
        });
    }

    thread::spawn(move || {
        let status = child.wait();
        if let Ok(mut active) = state.host_stdin.lock() {
            if active.as_ref().is_some_and(|entry| entry.generation == generation) {
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
        let delay_ms = (250_u64.saturating_mul(1_u64 << crashes.saturating_sub(1).min(4))).min(4_000);
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

fn write_to_host(state: &Arc<CoordinatorState>, line: &str, id: Option<Value>) -> io::Result<()> {
    let mut active = state.host_stdin.lock().map_err(|_| io::Error::other("host stdin lock poisoned"))?;
    if active.is_none() {
        drop(active);
        spawn_host(Arc::clone(state))?;
        active = state.host_stdin.lock().map_err(|_| io::Error::other("host stdin lock poisoned"))?;
    }
    let active = active.as_mut().ok_or_else(|| io::Error::other("Host did not provide stdin"))?;
    if let Some(id) = id {
        state.pending.lock().map_err(|_| io::Error::other("pending lock poisoned"))?.insert(
            pending_key(&id),
            PendingRequest { generation: active.generation, id },
        );
    }
    writeln!(active.stdin, "{line}")?;
    active.stdin.flush()
}

fn main() {
    let host_bin = match env::var_os("MAHAYANA_APP_HOST_BIN").filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => {
            eprintln!("MAHAYANA_APP_HOST_BIN is required");
            std::process::exit(2);
        }
    };

    let state = Arc::new(CoordinatorState {
        host_bin,
        host_stdin: Mutex::new(None),
        pending: Mutex::new(HashMap::new()),
        stdout_lock: Mutex::new(()),
        spawn_lock: Mutex::new(()),
        host_generation: AtomicU64::new(0),
        lifecycle_sequence: AtomicU64::new(0),
        closed: AtomicBool::new(false),
        consecutive_crashes: AtomicU64::new(0),
    });

    if let Err(error) = spawn_host(Arc::clone(&state)) {
        state.lifecycle("spawn-failed", 0, true, Some(&error.to_string()));
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
        let parsed = match serde_json::from_str::<Value>(&line) {
            Ok(value) => value,
            Err(error) => {
                let _ = state.write_json(&json!({
                    "ok": false,
                    "error": format!("invalid coordinator request JSON: {error}")
                }));
                continue;
            }
        };

        if parsed.get("method").and_then(Value::as_str) == Some("coordinator.health") {
            let generation = state.host_generation.load(Ordering::SeqCst);
            let pending = state.pending.lock().map(|pending| pending.len()).unwrap_or_default();
            let _ = state.write_json(&json!({
                "id": parsed.get("id").cloned().unwrap_or(Value::Null),
                "ok": true,
                "result": {
                    "protocolVersion": 1,
                    "hostGeneration": generation,
                    "pending": pending,
                    "hostRunning": state.host_stdin.lock().map(|host| host.is_some()).unwrap_or(false)
                }
            }));
            continue;
        }

        let id = parsed.get("id").cloned();
        if let Err(error) = write_to_host(&state, &line, id.clone()) {
            if let Some(id) = id {
                if let Ok(mut pending) = state.pending.lock() {
                    pending.remove(&pending_key(&id));
                }
                let _ = state.write_json(&json!({
                    "id": id,
                    "ok": false,
                    "error": format!("Coordinator could not dispatch to Host: {error}")
                }));
            }
        }
    }

    state.closed.store(true, Ordering::SeqCst);
    if let Ok(mut active) = state.host_stdin.lock() {
        active.take();
    }
    state.reject_generation(
        state.host_generation.load(Ordering::SeqCst),
        "Coordinator input closed",
    );
}
