use std::env;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

pub const NETWORK_WORKER_PROTOCOL_VERSION: u32 = 1;
pub const NETWORK_WORKER_START_TIMEOUT_MS: u64 = 30_000;
pub const NETWORK_WORKER_STOP_TIMEOUT_MS: u64 = 5_000;
pub const NETWORK_WORKER_STATUS_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkWorkerStatus {
    pub backend_state: String,
    pub tailscale_ips: Vec<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NetworkWorkerProcessOptions {
    pub executable_path: PathBuf,
    pub hostname: String,
    pub control_url: String,
    pub auth_key: String,
    pub state_dir: PathBuf,
    pub state_key_b64: String,
    pub start_timeout_ms: u64,
    pub stop_timeout_ms: u64,
}

impl fmt::Debug for NetworkWorkerProcessOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NetworkWorkerProcessOptions")
            .field("executable_path", &self.executable_path)
            .field("hostname", &self.hostname)
            .field("control_url", &self.control_url)
            .field("auth_key", &"<redacted>")
            .field("state_dir", &self.state_dir)
            .field("state_key_b64", &"<redacted>")
            .field("start_timeout_ms", &self.start_timeout_ms)
            .field("stop_timeout_ms", &self.stop_timeout_ms)
            .finish()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkWorkerInitialize<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    protocol_version: u32,
    hostname: &'a str,
    control_url: &'a str,
    auth_key: &'a str,
    state_dir: String,
    state_key_b64: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkWorkerControl<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    request_id: &'a str,
}

#[derive(Debug)]
pub struct OwnedNetworkWorker {
    child: Option<Child>,
    parent_pipe: Option<ChildStdin>,
    response_rx: Option<mpsc::Receiver<Result<String, String>>>,
    pub pid: u32,
    pub executable_path: PathBuf,
    stop_timeout_ms: u64,
}

impl OwnedNetworkWorker {
    pub fn status(&mut self) -> Result<NetworkWorkerStatus, String> {
        let request_id = format!("status-{}", Uuid::new_v4().simple());
        let request = serde_json::to_string(&NetworkWorkerControl {
            kind: "status",
            request_id: &request_id,
        })
        .map_err(|error| format!("could not serialize network worker status request: {error}"))?;
        let pipe = self
            .parent_pipe
            .as_mut()
            .ok_or_else(|| "network worker control pipe is closed".to_string())?;
        writeln!(pipe, "{request}")
            .and_then(|_| pipe.flush())
            .map_err(|error| format!("could not request network worker status: {error}"))?;

        let receiver = self
            .response_rx
            .as_ref()
            .ok_or_else(|| "network worker status pipe is closed".to_string())?;
        let deadline = Instant::now() + Duration::from_millis(NETWORK_WORKER_STATUS_TIMEOUT_MS);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err("network worker status timed out".into());
            }
            let line = match receiver.recv_timeout(remaining.min(Duration::from_millis(100))) {
                Ok(Ok(line)) => line,
                Ok(Err(error)) => return Err(format!("could not read network worker status: {error}")),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("network worker status pipe closed".into());
                }
            };
            let event: Value = serde_json::from_str(line.trim())
                .map_err(|_| "network worker emitted invalid status protocol".to_string())?;
            if event.get("requestId").and_then(Value::as_str) != Some(request_id.as_str()) {
                continue;
            }
            match event.get("type").and_then(Value::as_str) {
                Some("status") => {
                    let backend_state = event
                        .get("backendState")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let tailscale_ips = event
                        .get("tailscaleIps")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                    return Ok(NetworkWorkerStatus {
                        backend_state,
                        tailscale_ips,
                    });
                }
                Some("error") => {
                    let code = event
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("network_worker_error");
                    return Err(format!("network worker status failed: {code}"));
                }
                _ => continue,
            }
        }
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.close_inner(true)
    }

    fn close_inner(&mut self, strict: bool) -> Result<(), String> {
        if let Some(pipe) = self.parent_pipe.as_mut() {
            let request_id = format!("shutdown-{}", Uuid::new_v4().simple());
            let shutdown = serde_json::to_string(&NetworkWorkerControl {
                kind: "shutdown",
                request_id: &request_id,
            })
            .map_err(|error| format!("could not serialize network worker shutdown: {error}"))?;
            let _ = writeln!(pipe, "{shutdown}");
            let _ = pipe.flush();
        }
        self.parent_pipe.take();

        let Some(mut child) = self.child.take() else {
            return Ok(());
        };
        let grace = Duration::from_millis(self.stop_timeout_ms.min(1_000));
        if wait_for_child_exit(&mut child, grace)? {
            return Ok(());
        }
        let forced = terminate_child(
            &mut child,
            self.pid,
            Duration::from_millis(self.stop_timeout_ms),
        )?;
        if forced && strict {
            return Err(format!(
                "fabushi-network-worker pid {} required forced shutdown",
                self.pid
            ));
        }
        Ok(())
    }
}

impl Drop for OwnedNetworkWorker {
    fn drop(&mut self) {
        let _ = self.close_inner(false);
    }
}

pub fn network_worker_binary_name() -> &'static str {
    if cfg!(windows) {
        "fabushi-network-worker.exe"
    } else {
        "fabushi-network-worker"
    }
}

pub fn resolve_network_worker_executable(host_executable: &Path) -> Result<PathBuf, String> {
    let directory = host_executable.parent().ok_or_else(|| {
        format!(
            "cannot resolve network worker sibling for Host executable {}",
            host_executable.display()
        )
    })?;
    Ok(directory.join(network_worker_binary_name()))
}

pub fn default_network_worker_executable() -> Result<PathBuf, String> {
    let current_exe = env::current_exe()
        .map_err(|error| format!("could not resolve Host executable: {error}"))?;
    resolve_network_worker_executable(&current_exe)
}

pub fn start_network_worker(
    options: NetworkWorkerProcessOptions,
) -> Result<OwnedNetworkWorker, String> {
    validate_options(&options)?;

    let metadata = fs::metadata(&options.executable_path).map_err(|error| {
        format!(
            "network worker executable is unavailable at {}: {error}",
            options.executable_path.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "network worker executable is not a file: {}",
            options.executable_path.display()
        ));
    }
    fs::create_dir_all(&options.state_dir).map_err(|error| {
        format!(
            "could not create network worker state directory {}: {error}",
            options.state_dir.display()
        )
    })?;

    let mut command = Command::new(&options.executable_path);
    command
        .current_dir(
            options
                .executable_path
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        )
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    copy_required_os_environment(&mut command);

    let mut child = command.spawn().map_err(|error| {
        format!(
            "could not spawn network worker {}: {error}",
            options.executable_path.display()
        )
    })?;
    let pid = child.id();

    let mut parent_pipe = match child.stdin.take() {
        Some(pipe) => pipe,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("network worker did not expose its parent-lifetime pipe".into());
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("network worker did not expose its private status pipe".into());
        }
    };
    if let Some(stderr) = child.stderr.take() {
        spawn_redacted_log_reader(stderr);
    }

    let initialize = serde_json::to_string(&NetworkWorkerInitialize {
        kind: "initialize",
        protocol_version: NETWORK_WORKER_PROTOCOL_VERSION,
        hostname: &options.hostname,
        control_url: &options.control_url,
        auth_key: &options.auth_key,
        state_dir: options.state_dir.to_string_lossy().into_owned(),
        state_key_b64: &options.state_key_b64,
    })
    .map_err(|error| format!("could not serialize network worker initialization: {error}"))?;
    if let Err(error) = writeln!(parent_pipe, "{initialize}").and_then(|_| parent_pipe.flush()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("could not initialize network worker: {error}"));
    }

    let readiness_rx = spawn_protocol_reader(stdout);
    let deadline = Instant::now() + Duration::from_millis(options.start_timeout_ms);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Err(format!("network worker exited before readiness: {status}"));
        }
        let now = Instant::now();
        if now >= deadline {
            drop(parent_pipe);
            let _ = terminate_child(
                &mut child,
                pid,
                Duration::from_millis(options.stop_timeout_ms),
            );
            return Err("network worker readiness timed out".into());
        }
        let remaining = deadline.saturating_duration_since(now);
        let wait = remaining.min(Duration::from_millis(100));
        match readiness_rx.recv_timeout(wait) {
            Ok(Ok(line)) => {
                let event: Value = serde_json::from_str(line.trim())
                    .map_err(|_| "network worker emitted invalid readiness protocol".to_string())?;
                match event.get("type").and_then(Value::as_str) {
                    Some("ready") => break,
                    Some("error") => {
                        drop(parent_pipe);
                        let _ = terminate_child(
                            &mut child,
                            pid,
                            Duration::from_millis(options.stop_timeout_ms),
                        );
                        let code = event
                            .get("code")
                            .and_then(Value::as_str)
                            .unwrap_or("network_worker_error");
                        return Err(format!("network worker rejected initialization: {code}"));
                    }
                    _ => {}
                }
            }
            Ok(Err(error)) => {
                drop(parent_pipe);
                let _ = terminate_child(
                    &mut child,
                    pid,
                    Duration::from_millis(options.stop_timeout_ms),
                );
                return Err(format!("could not read network worker readiness: {error}"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                drop(parent_pipe);
                let _ = terminate_child(
                    &mut child,
                    pid,
                    Duration::from_millis(options.stop_timeout_ms),
                );
                return Err("network worker status pipe closed before readiness".into());
            }
        }
    }
    Ok(OwnedNetworkWorker {
        child: Some(child),
        parent_pipe: Some(parent_pipe),
        response_rx: Some(readiness_rx),
        pid,
        executable_path: options.executable_path,
        stop_timeout_ms: options.stop_timeout_ms,
    })
}

fn validate_options(options: &NetworkWorkerProcessOptions) -> Result<(), String> {
    if options.hostname.len() < 8 || options.hostname.len() > 128 {
        return Err("network worker hostname must be 8..128 characters".into());
    }
    if !options.control_url.starts_with("https://") {
        return Err("network worker control URL must use HTTPS".into());
    }
    if options.state_key_b64.len() < 40 || options.state_key_b64.len() > 64 {
        return Err("network worker state key has invalid encoded length".into());
    }
    if options.start_timeout_ms == 0 || options.stop_timeout_ms == 0 {
        return Err("network worker timeouts must be non-zero".into());
    }
    Ok(())
}

fn copy_required_os_environment(command: &mut Command) {
    // Keep the worker isolated from Host/Fabushi credentials. Only OS/runtime
    // variables required to locate system resources or temporary directories
    // are inherited. Proxy variables are intentionally excluded because they
    // may themselves contain credentials.
    for name in [
        "PATH",
        "HOME",
        "USERPROFILE",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "TMPDIR",
    ] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
}

fn spawn_redacted_log_reader<R>(reader: R)
where
    R: std::io::Read + Send + 'static,
{
    let _ = thread::Builder::new()
        .name("fabushi-network-worker-log".into())
        .spawn(move || {
            for _line in BufReader::new(reader).lines().map_while(Result::ok) {
                eprintln!("[fabushi-network-worker] diagnostic event");
            }
        });
}

fn spawn_protocol_reader<R>(reader: R) -> mpsc::Receiver<Result<String, String>>
where
    R: std::io::Read + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let _ = thread::Builder::new()
        .name("fabushi-network-worker-protocol".into())
        .spawn(move || {
            for line in BufReader::new(reader).lines() {
                match line {
                    Ok(line) => {
                        let _ = sender.send(Ok(line));
                    }
                    Err(error) => {
                        let _ = sender.send(Err(error.to_string()));
                        break;
                    }
                }
            }
        });
    receiver
}

fn wait_for_child_exit(child: &mut Child, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn terminate_child(child: &mut Child, pid: u32, timeout: Duration) -> Result<bool, String> {
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Ok(false);
    }
    if let Err(term_error) = request_graceful_termination(child) {
        child.kill().map_err(|kill_error| {
            format!(
                "could not terminate network worker pid {pid}: {term_error}; force kill failed: {kill_error}"
            )
        })?;
        let _ = child.wait();
        return Ok(true);
    }
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(25));
    }
    child
        .kill()
        .map_err(|error| format!("could not force-stop network worker pid {pid}: {error}"))?;
    let _ = child.wait();
    Ok(true)
}

#[cfg(unix)]
fn request_graceful_termination(child: &mut Child) -> Result<(), String> {
    let result = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "could not send SIGTERM to network worker pid {}: {}",
            child.id(),
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(not(unix))]
fn request_graceful_termination(child: &mut Child) -> Result<(), String> {
    child.kill().map_err(|error| {
        format!(
            "could not terminate network worker pid {}: {error}",
            child.id()
        )
    })
}
