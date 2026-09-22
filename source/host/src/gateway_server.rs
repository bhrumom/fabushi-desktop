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

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
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
pub const GATEWAY_SLIM_AVATARS_HEADER: &str = "x-sand-slim-avatars";
pub const GATEWAY_TRACEPARENT_HEADER: &str = "traceparent";
pub const SSE_HEARTBEAT_MS: u64 = 15_000;
pub const MAX_REQUEST_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_BODY_BYTES: usize = MAX_REQUEST_PAYLOAD_BYTES * 4 / 3 + 64 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;

const GATEWAY_COMMANDS: &[&str] = &[
    "getTranscript", "getAgentTranscript", "getAgentTranscriptPage", "openAgentWindowed",
    "getAgentTranscriptWindow", "openAgentTail", "getAgentTranscriptTail", "getAgentThread",
    "sendPrompt", "promptAcceptanceStatus", "respondToWidget", "resolveAutoReviewApproval",
    "resolveLocalToolPermission", "dismissWidget", "submitSecret", "reactToMessage",
    "appendConnectorCard", "listAgents", "countAgents", "searchAgents", "searchMedia",
    "createAgent", "kickstartAgent", "requestDiskSaverAudit", "createGroup", "setGroupMembers",
    "updateAgent", "deleteAgent", "deleteAgents", "duplicateAgent", "setAgentUnread",
    "setAgentNotificationsEnabled", "setAgentNotifyOnUpdates", "setAgentHiddenFromSidebar",
    "openAgent", "setWindowFocused", "getAgentMemories", "deleteAgentMemory",
    "clearAgentMemories", "getAgentAutomations", "listAllAutomations", "isAgentNetworkEnabled",
    "isGlobalSearchEnabled", "isEgressTunnelAvailable", "getSharingState", "createRoomFromAgent",
    "createRoomInvite", "joinSharedRoom", "respondToRoomJoinRequest", "createSharedRoom",
    "addOwnAgentToSharedRoom", "removeOwnAgentFromSharedRoom", "setSharedRoomTyping",
    "leaveSharedRoom", "setAgentAutomationEnabled", "createAgentAutomation",
    "updateAgentAutomation", "deleteAgentAutomation", "runAgentAutomationNow",
    "broadcastToAgents", "getAgentWorkflows", "createAgentWorkflow", "updateAgentWorkflow",
    "setAgentWorkflowEnabled", "deleteAgentWorkflow", "runAgentWorkflowNow",
    "importAgentWorkflowText", "importAgentWorkflowUrl", "portAgentLocalSkills",
    "getConversationOutline", "skillsCatalog", "syncPluginSkills", "getPluginSyncStatus",
    "getSkillPublishTargets", "publishSkill", "resyncPublishedSkill", "unpublishSkill",
    "getAgentChannels", "connectChannel", "disconnectChannel", "refreshChannel",
    "getListenerIntegrations", "getListenerConnectUrl", "getSubagents", "getAsyncTasks",
    "setAgentAvatarBytes", "getAgentAvatar", "getForeverBoxStatus", "getCloudAgentInfo",
    "ensureForeverBox", "resetForeverBox", "updateForeverBox", "autoUpdateBoxNow",
    "snapshotBoxStoreNow", "getBoxStoreStatus", "clearBoxStoreNow", "updateHostNow",
    "getHostStatus", "setBoxMigrating", "prepareBoxForRecreate", "resumeBoxAfterRecreate",
    "handBackForeverBox", "startTeachRecording", "stopTeachRecording",
    "getTeachRecordingStatus", "getTrays", "dismissTray", "clearTrays", "uploadAttachment",
    "readAttachmentImage", "readAttachmentText", "readAttachmentChunk", "getHostSettings",
    "setHostSettings", "setBoxSecrets", "getBoxSecretsStatus", "completeMcpOAuth",
    "requestWebAuthnCeremony", "refreshMcp", "listRoutedMcpTools", "executeRoutedMcpTool",
    "listBoxMcpServers",
];

fn is_gateway_command(method: &str) -> bool {
    GATEWAY_COMMANDS.contains(&method)
}

fn slim_command_result(method: &str, mut value: Value) -> Value {
    fn strip_summary(summary: &mut Value) {
        if let Some(object) = summary.as_object_mut() {
            if object.contains_key("avatarDataUrl") {
                object.insert("avatarDataUrl".into(), Value::Null);
            }
        }
    }

    match method {
        "listAgents" => {
            if let Some(rows) = value.as_array_mut() {
                for row in rows {
                    strip_summary(row);
                }
            }
        }
        "updateAgent" | "setGroupMembers" | "setAgentAvatarBytes" => strip_summary(&mut value),
        "createAgent" | "createGroup" | "duplicateAgent" => {
            if let Some(agent) = value.get_mut("agent") {
                strip_summary(agent);
            }
        }
        _ => {}
    }
    value
}

fn slim_event(mut event: Value) -> Value {
    match event.get("channel").and_then(Value::as_str) {
        Some("agents") => {
            if let Some(rows) = event
                .get_mut("payload")
                .and_then(|payload| payload.get_mut("agents"))
                .and_then(Value::as_array_mut)
            {
                for row in rows {
                    if let Some(object) = row.as_object_mut() {
                        if object.contains_key("avatarDataUrl") {
                            object.insert("avatarDataUrl".into(), Value::Null);
                        }
                    }
                }
            }
        }
        Some("agent-upserted") => {
            if let Some(object) = event
                .get_mut("payload")
                .and_then(|payload| payload.get_mut("agent"))
                .and_then(Value::as_object_mut)
            {
                if object.contains_key("avatarDataUrl") {
                    object.insert("avatarDataUrl".into(), Value::Null);
                }
            }
        }
        _ => {}
    }
    event
}


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
pub struct GatewayBridgeHub {
    request_subscribers: Arc<Mutex<Vec<Sender<Value>>>>,
    response_subscribers: Arc<Mutex<Vec<Sender<Value>>>>,
}

impl GatewayBridgeHub {
    pub fn publish_request(&self, frame: Value) {
        if let Ok(mut subscribers) = self.request_subscribers.lock() {
            subscribers.retain(|subscriber| subscriber.send(frame.clone()).is_ok());
        }
    }

    pub fn subscribe_requests(&self) -> Receiver<Value> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut subscribers) = self.request_subscribers.lock() {
            subscribers.push(sender);
        }
        receiver
    }

    pub fn submit_responses(&self, batch: Value) {
        if let Ok(mut subscribers) = self.response_subscribers.lock() {
            subscribers.retain(|subscriber| subscriber.send(batch.clone()).is_ok());
        }
    }

    pub fn subscribe_responses(&self) -> Receiver<Value> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut subscribers) = self.response_subscribers.lock() {
            subscribers.push(sender);
        }
        receiver
    }

    pub fn request_subscriber_count(&self) -> usize {
        self.request_subscribers
            .lock()
            .map(|subscribers| subscribers.len())
            .unwrap_or_default()
    }

    pub fn response_subscriber_count(&self) -> usize {
        self.response_subscribers
            .lock()
            .map(|subscribers| subscribers.len())
            .unwrap_or_default()
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
    pub local_exec: Option<GatewayBridgeHub>,
    pub webauthn: Option<GatewayBridgeHub>,
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
        304 => "Not Modified",
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
    slim_avatars: bool,
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
                let event = if slim_avatars { slim_event(event) } else { event };
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

fn decode_path_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                b'A'..=b'F' => Some(byte - b'A' + 10),
                _ => None,
            };
            let high = digit(bytes[index + 1])?;
            let low = digit(bytes[index + 2])?;
            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

fn query_value<'a>(query: Option<&'a str>, key: &str) -> Option<&'a str> {
    query?.split('&').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn serve_avatar(
    stream: &mut TcpStream,
    deps: &GatewayServerDeps,
    request: &HttpRequest,
    path: &str,
    query: Option<&str>,
) -> io::Result<()> {
    if request
        .headers
        .get("sec-fetch-site")
        .is_some_and(|value| value.eq_ignore_ascii_case("cross-site"))
    {
        return respond_error(stream, 403, "cross-site avatar loads are not allowed");
    }
    let encoded_id = path
        .strip_prefix(&format!("{GATEWAY_AVATARS_PATH}/"))
        .unwrap_or_default();
    let Some(agent_id) = decode_path_component(encoded_id).filter(|value| !value.is_empty()) else {
        return respond_error(stream, 404, "missing agent id");
    };
    let avatar = match deps.api.call("getAgentAvatar", json!({ "id": agent_id })) {
        Ok(value) => value,
        Err(error) => return respond_error(stream, error.status(), error.to_string()),
    };
    let version = avatar.get("version").and_then(Value::as_str);
    let data_url = avatar.get("dataUrl").and_then(Value::as_str);
    let (Some(version), Some(data_url)) = (version, data_url) else {
        return respond_error(stream, 404, "agent has no avatar");
    };
    if query_value(query, "v").is_some_and(|requested| requested != version) {
        return respond_error(stream, 404, "no such avatar version");
    }
    let Some(rest) = data_url.strip_prefix("data:") else {
        return respond_error(stream, 404, "agent has no avatar");
    };
    let Some((mime, encoded)) = rest.split_once(";base64,") else {
        return respond_error(stream, 404, "agent has no avatar");
    };
    if mime.is_empty() {
        return respond_error(stream, 404, "agent has no avatar");
    }
    let bytes = match BASE64_STANDARD.decode(encoded) {
        Ok(bytes) => bytes,
        Err(_) => return respond_error(stream, 404, "agent has no avatar"),
    };
    let etag = format!("\"{version}\"");
    let immutable = query_value(query, "v").is_some();
    if request.headers.get("if-none-match").is_some_and(|value| value == &etag) {
        return write_response(
            stream,
            304,
            mime,
            &[],
            &[
                ("Cache-Control", if immutable { "private, max-age=31536000, immutable" } else { "no-store" }),
                ("ETag", &etag),
            ],
        );
    }
    write_response(
        stream,
        200,
        mime,
        &bytes,
        &[
            ("Cache-Control", if immutable { "private, max-age=31536000, immutable" } else { "no-store" }),
            ("ETag", &etag),
            ("Content-Disposition", "attachment"),
            ("X-Content-Type-Options", "nosniff"),
            ("Content-Security-Policy", "default-src 'none'; sandbox"),
        ],
    )
}

fn serve_bridge_requests(
    stream: &mut TcpStream,
    bridge: &GatewayBridgeHub,
    stop: &AtomicBool,
) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache, no-transform\r\nConnection: keep-alive\r\n\r\nretry: 1000\n\n"
    )?;
    stream.flush()?;
    let receiver = bridge.subscribe_requests();
    let heartbeat = Duration::from_millis(SSE_HEARTBEAT_MS);
    while !stop.load(Ordering::Acquire) {
        match receiver.recv_timeout(heartbeat) {
            Ok(frame) => {
                let encoded = serde_json::to_string(&frame)
                    .map_err(|error| io::Error::other(format!("serialize gateway bridge frame: {error}")))?;
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

fn submit_bridge_responses(
    stream: &mut TcpStream,
    bridge: &GatewayBridgeHub,
    body: &[u8],
) -> io::Result<()> {
    let batch = if body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice::<Value>(body) {
            Ok(value) => value,
            Err(error) => {
                return respond_error(stream, 400, format!("invalid JSON body: {error}"));
            }
        }
    };
    bridge.submit_responses(batch);
    respond_json(stream, 200, json!({ "ok": true }))
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
    let avatar = request.method == "GET" && path.starts_with(&format!("{GATEWAY_AVATARS_PATH}/"));
    let command = request.method == "POST"
        && path
            .strip_prefix(&format!("{GATEWAY_API_PREFIX}/"))
            .is_some_and(|method| !method.is_empty());
    let local_requests = request.method == "GET" && path == GATEWAY_LOCAL_EXEC_REQUESTS_PATH;
    let local_responses = request.method == "POST" && path == GATEWAY_LOCAL_EXEC_RESPONSES_PATH;
    let webauthn_requests = request.method == "GET" && path == GATEWAY_WEBAUTHN_REQUESTS_PATH;
    let webauthn_responses = request.method == "POST" && path == GATEWAY_WEBAUTHN_RESPONSES_PATH;
    if !(events || prepare || avatar || command || local_requests || local_responses || webauthn_requests || webauthn_responses) {
        return respond_error(
            &mut stream,
            404,
            format!("not found: {} {path}", request.method),
        );
    }
    if (local_requests || local_responses) && deps.config.auth_token.is_none() {
        return respond_error(&mut stream, 401, "local-exec requires gateway authentication");
    }
    if (webauthn_requests || webauthn_responses) && deps.config.auth_token.is_none() {
        return respond_error(&mut stream, 401, "webauthn requires gateway authentication");
    }
    if let Some(token) = deps.config.auth_token.as_deref() {
        if !is_authorized(&request, token) {
            return respond_error(&mut stream, 401, "unauthorized");
        }
    }
    if local_requests {
        return match deps.local_exec.as_ref() {
            Some(bridge) => serve_bridge_requests(&mut stream, bridge, stop),
            None => respond_error(&mut stream, 404, "local-exec channel not enabled"),
        };
    }
    if local_responses {
        return match deps.local_exec.as_ref() {
            Some(bridge) => submit_bridge_responses(&mut stream, bridge, &request.body),
            None => respond_error(&mut stream, 404, "local-exec channel not enabled"),
        };
    }
    if webauthn_requests {
        return match deps.webauthn.as_ref() {
            Some(bridge) => serve_bridge_requests(&mut stream, bridge, stop),
            None => respond_error(&mut stream, 404, "webauthn channel not enabled"),
        };
    }
    if webauthn_responses {
        return match deps.webauthn.as_ref() {
            Some(bridge) => submit_bridge_responses(&mut stream, bridge, &request.body),
            None => respond_error(&mut stream, 404, "webauthn channel not enabled"),
        };
    }
    if avatar {
        return serve_avatar(&mut stream, deps, &request, path, query);
    }
    if events {
        let slim_avatars = request
            .headers
            .get(GATEWAY_SLIM_AVATARS_HEADER)
            .is_some_and(|value| value == "1");
        return serve_events(&mut stream, deps, stop, parse_channels(query), slim_avatars);
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
    if !is_gateway_command(method) {
        return respond_error(&mut stream, 404, format!("unknown gateway method: {method}"));
    }
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
        Ok(value) => {
            let slim = request
                .headers
                .get(GATEWAY_SLIM_AVATARS_HEADER)
                .is_some_and(|value| value == "1");
            respond_json(
                &mut stream,
                200,
                if slim { slim_command_result(method, value) } else { value },
            )
        }
        Err(error) => respond_error(&mut stream, error.status(), error.to_string()),
    }
}
