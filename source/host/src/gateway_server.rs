use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{Value, json};
use thiserror::Error;

use crate::gateway_config::{GatewayServerConfig, is_loopback_host};

pub const GATEWAY_API_PREFIX: &str = "/api";
pub const GATEWAY_EVENTS_PATH: &str = "/events";
pub const GATEWAY_HEALTH_PATH: &str = "/health";
pub const GATEWAY_AUTH_SCHEME: &str = "Bearer";
pub const GATEWAY_PREPARE_UPGRADE_PATH: &str = "/prepare-upgrade";
pub const GATEWAY_LOCAL_EXEC_REQUESTS_PATH: &str = "/local-exec/requests";
pub const GATEWAY_LOCAL_EXEC_RESPONSES_PATH: &str = "/local-exec/responses";
pub const GATEWAY_WEBAUTHN_REQUESTS_PATH: &str = "/webauthn/requests";
pub const GATEWAY_WEBAUTHN_RESPONSES_PATH: &str = "/webauthn/responses";
pub const GATEWAY_AVATARS_PATH: &str = "/avatars";
pub const GATEWAY_REQUEST_ID_HEADER: &str = "x-sand-request-id";
pub const GATEWAY_MINT_DEDUPE_HEADER: &str = "x-sand-mint-dedupe";
pub const SSE_HEARTBEAT_MS: u64 = 15_000;
pub const MAX_REQUEST_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_BODY_BYTES: usize = MAX_REQUEST_PAYLOAD_BYTES * 4 / 3 + 64 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GatewayHealth {
    pub is_busy: bool,
    pub busy_only_awaiting_approval: Option<bool>,
    pub active_agent_id: Option<String>,
    pub last_busy_at_ms: Option<u64>,
}

#[derive(Debug, Error)]
pub enum GatewayCommandError {
    #[error("unknown gateway method: {0}")]
    UnknownMethod(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Internal(String),
}

impl GatewayCommandError {
    fn status(&self) -> u16 {
        match self {
            Self::UnknownMethod(_) => 404,
            Self::BadRequest(_) => 400,
            Self::Conflict(_) => 409,
            Self::Internal(_) => 500,
        }
    }
}

pub trait GatewayApi: Send + Sync + 'static {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError>;
    fn health(&self) -> GatewayHealth {
        GatewayHealth::default()
    }
    fn prepare_for_upgrade(&self) -> Result<Value, GatewayCommandError> {
        Ok(json!({ "quiescing": false, "runningTurns": 0 }))
    }
}

#[derive(Clone, Default)]
pub struct GatewayEventHub {
    subscribers: Arc<Mutex<Vec<Sender<Value>>>>,
}

impl GatewayEventHub {
    pub fn publish(&self, event: Value) {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.retain(|subscriber| subscriber.send(event.clone()).is_ok());
        }
    }

    pub fn subscribe(&self) -> Receiver<Value> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push(sender);
        }
        receiver
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers
            .lock()
            .map(|subscribers| subscribers.len())
            .unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct GatewayServerDeps {
    pub api: Arc<dyn GatewayApi>,
    pub events: GatewayEventHub,
    pub config: GatewayServerConfig,
    pub started_at: u64,
}

pub struct GatewayServer {
    port: u16,
    stop: Arc<AtomicBool>,
    wake_addr: SocketAddr,
    thread: Option<JoinHandle<()>>,
}

impl GatewayServer {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn close(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect_timeout(&self.wake_addr, Duration::from_millis(100));
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for GatewayServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start_gateway_server(deps: GatewayServerDeps) -> io::Result<GatewayServer> {
    if deps.config.tls.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "TLS gateway transport is not wired into the Rust Host yet",
        ));
    }
    let port = deps.config.port.unwrap_or(0);
    let listener = TcpListener::bind((deps.config.host.as_str(), port))?;
    listener.set_nonblocking(true)?;
    let local = listener.local_addr()?;
    let wake_addr = match local.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => SocketAddr::from(([127, 0, 0, 1], local.port())),
        IpAddr::V6(ip) if ip.is_unspecified() => SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], local.port())),
        _ => local,
    };
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let deps = Arc::new(deps);
    let thread_deps = Arc::clone(&deps);
    let thread = thread::spawn(move || {
        while !thread_stop.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let connection_deps = Arc::clone(&thread_deps);
                    let connection_stop = Arc::clone(&thread_stop);
                    thread::spawn(move || {
                        let _ = handle_connection(stream, &connection_deps, &connection_stop);
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) if thread_stop.load(Ordering::Acquire) => break,
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        }
    });
    Ok(GatewayServer {
        port: local.port(),
        stop,
        wake_addr,
        thread: Some(thread),
    })
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    let mut bytes = Vec::with_capacity(2048);
    let mut chunk = [0_u8; 4096];
    let header_end;
    loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "request closed"));
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = index + 4;
            break;
        }
        if bytes.len() > MAX_HEADER_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request headers are too large",
            ));
        }
    }

    let header_text = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "request headers are not utf-8"))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request line"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or("/").to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > MAX_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "request body is too large",
        ));
    }
    let already = bytes.len().saturating_sub(header_end);
    while bytes.len().saturating_sub(header_end) < content_length {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len().saturating_sub(header_end) > MAX_BODY_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request body is too large",
            ));
        }
    }
    let available = bytes.len().saturating_sub(header_end);
    if available < content_length && content_length > already {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "request body ended early",
        ));
    }
    Ok(HttpRequest {
        method,
        target,
        headers,
        body: bytes[header_end..header_end + content_length].to_vec(),
    })
}

fn split_target(target: &str) -> (&str, Option<&str>) {
    target
        .split_once('?')
        .map_or((target, None), |(path, query)| (path, Some(query)))
}

fn host_header_hostname(value: Option<&String>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(rest) = value.strip_prefix('[') {
        let end = rest.find(']')?;
        return Some(rest[..end].to_string());
    }
    Some(value.split(':').next().unwrap_or(value).to_string())
}

fn is_authorized(request: &HttpRequest, expected: &str) -> bool {
    request
        .headers
        .get("authorization")
        .and_then(|value| value.strip_prefix(&format!("{GATEWAY_AUTH_SCHEME} ")))
        .is_some_and(|provided| provided.as_bytes() == expected.as_bytes())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, &str)],
) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "Response",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )?;
    for (name, value) in extra_headers {
        write!(stream, "{name}: {value}\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(body)?;
    stream.flush()
}

fn respond_json(stream: &mut TcpStream, status: u16, value: Value) -> io::Result<()> {
    let encoded = serde_json::to_vec(&value)
        .map_err(|error| io::Error::other(format!("gateway response serialization failed: {error}")))?;
    write_response(
        stream,
        status,
        "application/json",
        &encoded,
        &[(GATEWAY_MINT_DEDUPE_HEADER, "1")],
    )
}

fn respond_error(stream: &mut TcpStream, status: u16, message: impl Into<String>) -> io::Result<()> {
    respond_json(stream, status, json!({ "error": message.into() }))
}

fn parse_channels(query: Option<&str>) -> Option<Vec<String>> {
    let raw = query?
        .split('&')
        .find_map(|part| part.split_once('=').filter(|(key, _)| *key == "channels").map(|(_, value)| value))?;
    let channels = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!channels.is_empty()).then_some(channels)
}

fn event_channel(event: &Value) -> Option<&str> {
    event.get("channel").and_then(Value::as_str)
}

fn serve_events(
    stream: &mut TcpStream,
    deps: &GatewayServerDeps,
    stop: &AtomicBool,
    channels: Option<Vec<String>>,
) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache, no-transform\r\nConnection: keep-alive\r\n\r\nretry: 1000\n\n"
    )?;
    stream.flush()?;
    let receiver = deps.events.subscribe();
    let heartbeat = Duration::from_millis(SSE_HEARTBEAT_MS);
    while !stop.load(Ordering::Acquire) {
        match receiver.recv_timeout(heartbeat) {
            Ok(event) => {
                if channels
                    .as_ref()
                    .is_some_and(|allowed| event_channel(&event).is_some_and(|channel| !allowed.iter().any(|value| value == channel)))
                {
                    continue;
                }
                let encoded = serde_json::to_string(&event)
                    .map_err(|error| io::Error::other(format!("serialize gateway event: {error}")))?;
                write!(stream, "data: {encoded}\n\n")?;
                stream.flush()?;
            }
            Err(RecvTimeoutError::Timeout) => {
                write!(stream, ":ping\n\n")?;
                stream.flush()?;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    deps: &GatewayServerDeps,
    stop: &AtomicBool,
) -> io::Result<()> {
    let request = match read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            let status = if error.to_string().contains("too large") { 413 } else { 400 };
            let _ = respond_error(&mut stream, status, error.to_string());
            return Ok(());
        }
    };
    if request.headers.contains_key("origin") {
        return respond_error(
            &mut stream,
            403,
            "browser-origin gateway requests are not allowed",
        );
    }
    if deps.config.auth_token.is_none() {
        let host = host_header_hostname(request.headers.get("host"));
        if host.as_deref().is_none_or(|host| !is_loopback_host(host)) {
            return respond_error(&mut stream, 403, "untrusted gateway host");
        }
    }

    let (path, query) = split_target(&request.target);
    if request.method == "GET" && path == GATEWAY_HEALTH_PATH {
        let health = deps.api.health();
        let mut value = json!({
            "ok": true,
            "pid": std::process::id(),
            "isBusy": health.is_busy,
            "activeAgentId": health.active_agent_id,
            "startedAt": deps.started_at,
            "lastBusyAtMs": health.last_busy_at_ms,
        });
        if let Some(busy_only) = health.busy_only_awaiting_approval {
            value["busyOnlyAwaitingApproval"] = Value::Bool(busy_only);
        }
        return respond_json(&mut stream, 200, value);
    }

    let events = request.method == "GET" && path == GATEWAY_EVENTS_PATH;
    let prepare = request.method == "POST" && path == GATEWAY_PREPARE_UPGRADE_PATH;
    let command = request.method == "POST"
        && path
            .strip_prefix(&format!("{GATEWAY_API_PREFIX}/"))
            .is_some_and(|method| !method.is_empty());
    let known_but_unwired = path.starts_with(&format!("{GATEWAY_AVATARS_PATH}/"))
        || path == GATEWAY_LOCAL_EXEC_REQUESTS_PATH
        || path == GATEWAY_LOCAL_EXEC_RESPONSES_PATH
        || path == GATEWAY_WEBAUTHN_REQUESTS_PATH
        || path == GATEWAY_WEBAUTHN_RESPONSES_PATH;
    if !(events || prepare || command || known_but_unwired) {
        return respond_error(
            &mut stream,
            404,
            format!("not found: {} {path}", request.method),
        );
    }
    if let Some(token) = deps.config.auth_token.as_deref() {
        if !is_authorized(&request, token) {
            return respond_error(&mut stream, 401, "unauthorized");
        }
    }
    if known_but_unwired {
        return respond_error(&mut stream, 404, "gateway bridge channel is not enabled");
    }
    if events {
        return serve_events(&mut stream, deps, stop, parse_channels(query));
    }
    if prepare {
        return match deps.api.prepare_for_upgrade() {
            Ok(value) => respond_json(&mut stream, 200, value),
            Err(error) => respond_error(&mut stream, error.status(), error.to_string()),
        };
    }

    let method = path
        .strip_prefix(&format!("{GATEWAY_API_PREFIX}/"))
        .unwrap_or_default();
    let args = if request.body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice::<Value>(&request.body) {
            Ok(value) => value,
            Err(error) => {
                return respond_error(&mut stream, 400, format!("invalid JSON body: {error}"));
            }
        }
    };
    match deps.api.call(method, args) {
        Ok(value) => respond_json(&mut stream, 200, value),
        Err(error) => respond_error(&mut stream, error.status(), error.to_string()),
    }
}
