use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::Value;

use super::gateway_reachability::ReachabilityOutcome;
use super::gateway_request_dispatcher::GatewayDispatchError;
use super::host_supervisor::GatewayConnection;
use super::sse_block_decoder::SseBlockDecoder;

pub const SSE_RECONNECT_MIN_MS: u64 = 1_000;
pub const SSE_RECONNECT_MAX_MS: u64 = 10_000;
pub const SSE_STALL_TIMEOUT_MS: u64 = 35_000;
pub const SSE_CONNECT_TIMEOUT_MS: u64 = 15_000;
pub const SEND_POST_TIMEOUT_MS: u64 = 15_000;
pub const ROSTER_READ_TIMEOUT_MS: u64 = 15_000;
pub const TRACE_WINDOW_ROOT_CACHE_MS: u64 = 5_000;
pub const HOST_ACCOUNT_SLOT: &str = "host";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GatewayClientTiming {
    pub sse_reconnect_min_ms: u64,
    pub sse_reconnect_max_ms: u64,
    pub sse_stall_timeout_ms: u64,
    pub sse_connect_timeout_ms: u64,
    pub send_post_timeout_ms: u64,
    pub roster_read_timeout_ms: u64,
}

impl Default for GatewayClientTiming {
    fn default() -> Self {
        Self {
            sse_reconnect_min_ms: SSE_RECONNECT_MIN_MS,
            sse_reconnect_max_ms: SSE_RECONNECT_MAX_MS,
            sse_stall_timeout_ms: SSE_STALL_TIMEOUT_MS,
            sse_connect_timeout_ms: SSE_CONNECT_TIMEOUT_MS,
            send_post_timeout_ms: SEND_POST_TIMEOUT_MS,
            roster_read_timeout_ms: ROSTER_READ_TIMEOUT_MS,
        }
    }
}

pub fn is_permanent_refusal(outcome: ReachabilityOutcome) -> bool {
    matches!(
        outcome,
        ReachabilityOutcome::NoStorage
            | ReachabilityOutcome::BoxBlocked
            | ReachabilityOutcome::AccessDenied
    )
}

pub fn is_pre_dispatch(outcome: ReachabilityOutcome) -> bool {
    matches!(outcome, ReachabilityOutcome::Refused | ReachabilityOutcome::Dns)
}

pub fn extract_gateway_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()?
        .get("error")
        .and_then(Value::as_str)
        .filter(|message| !message.is_empty())
        .map(str::to_string)
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("gateway endpoint changed from {previous} to {next}")]
pub struct GatewayEndpointChangedError {
    pub previous: String,
    pub next: String,
}

pub fn with_auth(
    headers: &BTreeMap<String, String>,
    token: Option<&str>,
) -> BTreeMap<String, String> {
    let mut output = headers.clone();
    if let Some(token) = token.filter(|token| !token.is_empty()) {
        output.insert("authorization".into(), format!("Bearer {token}"));
    }
    output
}

#[derive(Debug, Clone, PartialEq)]
pub struct GatewayEvent {
    pub channel: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayClientState {
    pub connected: bool,
    pub reconnect_attempt: u32,
    pub endpoint_generation: u64,
    pub stream_live: bool,
    pub last_event_ms: Option<u64>,
    pub last_outcome: Option<ReachabilityOutcome>,
    pub connection: Option<GatewayConnection>,
}

impl Default for GatewayClientState {
    fn default() -> Self {
        Self {
            connected: false,
            reconnect_attempt: 0,
            endpoint_generation: 0,
            stream_live: false,
            last_event_ms: None,
            last_outcome: None,
            connection: None,
        }
    }
}

impl GatewayClientState {
    pub fn install_connection(
        &mut self,
        connection: GatewayConnection,
    ) -> Result<u64, GatewayEndpointChangedError> {
        if let Some(previous) = &self.connection {
            if previous.base_url != connection.base_url && self.connected {
                return Err(GatewayEndpointChangedError {
                    previous: previous.base_url.clone(),
                    next: connection.base_url,
                });
            }
        }
        let changed = self
            .connection
            .as_ref()
            .is_none_or(|previous| previous.base_url != connection.base_url);
        if changed {
            self.endpoint_generation = self.endpoint_generation.saturating_add(1);
        }
        self.connection = Some(connection);
        Ok(self.endpoint_generation)
    }

    pub fn connected(&mut self, now_ms: u64) {
        self.connected = true;
        self.stream_live = true;
        self.reconnect_attempt = 0;
        self.last_outcome = None;
        self.last_event_ms = Some(now_ms);
    }

    pub fn mark_event(&mut self, now_ms: u64) {
        self.connected = true;
        self.stream_live = true;
        self.last_event_ms = Some(now_ms);
    }

    pub fn stream_stalled(&self, now_ms: u64, timing: GatewayClientTiming) -> bool {
        self.stream_live
            && self
                .last_event_ms
                .is_some_and(|last| now_ms.saturating_sub(last) >= timing.sse_stall_timeout_ms)
    }

    pub fn disconnected(&mut self, outcome: ReachabilityOutcome) -> Option<Duration> {
        self.connected = false;
        self.stream_live = false;
        self.last_outcome = Some(outcome);
        if is_permanent_refusal(outcome) || !outcome.retryable() {
            return None;
        }
        self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
        let exponent = self.reconnect_attempt.saturating_sub(1).min(4);
        let delay_ms = SSE_RECONNECT_MIN_MS
            .saturating_mul(1_u64 << exponent)
            .min(SSE_RECONNECT_MAX_MS);
        Some(Duration::from_millis(delay_ms))
    }

    pub fn force_reconnect(&mut self) {
        self.connected = false;
        self.stream_live = false;
        self.last_event_ms = None;
    }
}

#[derive(Debug)]
pub struct CoordinatorGatewayClient {
    pub state: GatewayClientState,
    pub timing: GatewayClientTiming,
    closed: bool,
}

impl Default for CoordinatorGatewayClient {
    fn default() -> Self {
        Self {
            state: GatewayClientState::default(),
            timing: GatewayClientTiming::default(),
            closed: false,
        }
    }
}

impl CoordinatorGatewayClient {
    pub fn install_connection(
        &mut self,
        connection: GatewayConnection,
    ) -> Result<u64, GatewayEndpointChangedError> {
        if self.closed {
            return Err(GatewayEndpointChangedError {
                previous: "closed".into(),
                next: connection.base_url,
            });
        }
        self.state.install_connection(connection)
    }

    pub fn start(&mut self, now_ms: u64) -> Result<(), &'static str> {
        if self.closed {
            return Err("gateway client is closed");
        }
        if self.state.connection.is_none() {
            return Err("gateway connection is unresolved");
        }
        self.state.connected(now_ms);
        Ok(())
    }

    pub fn connection_for_dispatch(&self) -> Result<&GatewayConnection, GatewayDispatchError> {
        if self.closed {
            return Err(GatewayDispatchError::Transport(
                "gateway client is closed".into(),
            ));
        }
        if !self.state.connected {
            return Err(GatewayDispatchError::Unreachable {
                outcome: self.state.last_outcome.unwrap_or(ReachabilityOutcome::Network),
                message: "gateway transport is not connected".into(),
            });
        }
        self.state.connection.as_ref().ok_or_else(|| {
            GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "gateway connection is unresolved".into(),
            }
        })
    }

    pub fn dispatch_command<F>(
        &mut self,
        method: &str,
        args: Value,
        dispatch: F,
    ) -> Result<Value, GatewayDispatchError>
    where
        F: FnOnce(&GatewayConnection, &str, Value) -> Result<Value, GatewayDispatchError>,
    {
        if self.closed {
            return Err(GatewayDispatchError::Transport(
                "gateway client is closed".into(),
            ));
        }
        let connection = self.state.connection.as_ref().ok_or_else(|| {
            GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "gateway connection is unresolved".into(),
            }
        })?;
        let result = dispatch(connection, method, args);
        if let Err(GatewayDispatchError::Unreachable { outcome, .. }) = &result {
            self.state.disconnected(*outcome);
        }
        result
    }

    pub fn accept_event(
        &mut self,
        now_ms: u64,
        channel: impl Into<String>,
        payload: Value,
    ) -> Option<GatewayEvent> {
        if self.closed {
            return None;
        }
        self.state.mark_event(now_ms);
        Some(GatewayEvent {
            channel: channel.into(),
            payload,
        })
    }

    pub fn transport_down(
        &mut self,
        outcome: ReachabilityOutcome,
    ) -> Option<Duration> {
        self.state.disconnected(outcome)
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.state.force_reconnect();
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}


fn parse_sse_http_base(base_url: &str) -> Result<(String, u16, String), GatewayDispatchError> {
    let Some(rest) = base_url.strip_prefix("http://") else {
        return Err(GatewayDispatchError::Transport(format!(
            "unsupported Host gateway SSE scheme in {base_url}"
        )));
    };
    let (authority, base_path) = rest
        .split_once('/')
        .map_or((rest, String::new()), |(authority, path)| {
            (authority, format!("/{}", path.trim_end_matches('/')))
        });
    let (host, port) = if let Some(ipv6) = authority.strip_prefix('[') {
        let Some((host, suffix)) = ipv6.split_once(']') else {
            return Err(GatewayDispatchError::Transport(
                "invalid IPv6 Host gateway authority".into(),
            ));
        };
        let port = suffix
            .strip_prefix(':')
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(|| GatewayDispatchError::Transport(
                "Host gateway URL must include a port".into(),
            ))?;
        (host.to_string(), port)
    } else {
        let (host, port) = authority.rsplit_once(':').ok_or_else(|| {
            GatewayDispatchError::Transport("Host gateway URL must include a port".into())
        })?;
        let port = port.parse::<u16>().map_err(|_| {
            GatewayDispatchError::Transport("invalid Host gateway port".into())
        })?;
        (host.to_string(), port)
    };
    Ok((host, port, base_path))
}

fn sse_outcome_for_io(error: &std::io::Error) -> ReachabilityOutcome {
    match error.kind() {
        std::io::ErrorKind::ConnectionRefused => ReachabilityOutcome::Refused,
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ReachabilityOutcome::Timeout,
        std::io::ErrorKind::NotFound | std::io::ErrorKind::AddrNotAvailable => ReachabilityOutcome::Dns,
        _ => ReachabilityOutcome::Network,
    }
}

fn sse_data(block: &str) -> Option<String> {
    let data = block
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>();
    (!data.is_empty()).then(|| data.join("\n"))
}

pub fn stream_http_events<Connected, Event, Continue>(
    connection: &GatewayConnection,
    on_connected: Connected,
    mut on_event: Event,
    should_continue: Continue,
) -> Result<(), GatewayDispatchError>
where
    Connected: FnOnce(),
    Event: FnMut(Value),
    Continue: Fn() -> bool,
{
    let (host, port, base_path) = parse_sse_http_base(&connection.base_url)?;
    let mut addresses = format!("{host}:{port}").to_socket_addrs().map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: format!("gateway events unreachable (dns): {error}"),
        }
    })?;
    let socket = addresses.next().ok_or_else(|| GatewayDispatchError::Unreachable {
        outcome: ReachabilityOutcome::Dns,
        message: "gateway events unreachable (dns)".into(),
    })?;
    let connect_timeout = Duration::from_millis(SSE_CONNECT_TIMEOUT_MS);
    let mut stream = TcpStream::connect_timeout(&socket, connect_timeout).map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: sse_outcome_for_io(&error),
            message: format!("gateway events connect failed: {error}"),
        }
    })?;
    stream
        .set_write_timeout(Some(connect_timeout))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(SSE_STALL_TIMEOUT_MS)))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    let path = format!("{base_path}/events");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nAccept: text/event-stream\r\nConnection: keep-alive\r\n"
    )
    .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    for (name, value) in &connection.headers {
        write!(stream, "{name}: {value}\r\n")
            .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    }
    write!(stream, "\r\n")
        .and_then(|_| stream.flush())
        .map_err(|error| GatewayDispatchError::Unreachable {
            outcome: sse_outcome_for_io(&error),
            message: format!("gateway events request failed: {error}"),
        })?;

    let mut buffered = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| GatewayDispatchError::Unreachable {
            outcome: sse_outcome_for_io(&error),
            message: format!("gateway events handshake failed: {error}"),
        })?;
        if count == 0 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "gateway events ended during handshake".into(),
            });
        }
        buffered.extend_from_slice(&chunk[..count]);
        if let Some(index) = buffered.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        if buffered.len() > 64 * 1024 {
            return Err(GatewayDispatchError::Transport(
                "gateway events response headers are too large".into(),
            ));
        }
    };
    let headers = std::str::from_utf8(&buffered[..header_end]).map_err(|_| {
        GatewayDispatchError::Transport("gateway events headers are not UTF-8".into())
    })?;
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| GatewayDispatchError::Transport(
            "gateway events response has no HTTP status".into(),
        ))?;
    if !(200..300).contains(&status) {
        if matches!(status, 401 | 403) {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::AccessDenied,
                message: format!("gateway events failed with HTTP {status}"),
            });
        }
        if status >= 500 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Http(status),
                message: format!("gateway events failed with HTTP {status}"),
            });
        }
        return Err(GatewayDispatchError::Command(
            super::gateway_errors::SandGatewayCommandError::new(format!(
                "gateway events failed with HTTP {status}"
            )),
        ));
    }

    on_connected();
    let mut decoder = SseBlockDecoder::default();
    let initial = &buffered[header_end..];
    if !initial.is_empty() {
        for block in decoder.push_bytes(initial) {
            if let Some(data) = sse_data(&block) {
                let value = serde_json::from_str::<Value>(&data).map_err(|error| {
                    GatewayDispatchError::Transport(format!(
                        "gateway events contained invalid JSON: {error}"
                    ))
                })?;
                on_event(value);
            }
        }
    }
    while should_continue() {
        let count = stream.read(&mut chunk).map_err(|error| GatewayDispatchError::Unreachable {
            outcome: sse_outcome_for_io(&error),
            message: format!("gateway events stream failed: {error}"),
        })?;
        if count == 0 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "gateway events stream ended".into(),
            });
        }
        for block in decoder.push_bytes(&chunk[..count]) {
            if let Some(data) = sse_data(&block) {
                let value = serde_json::from_str::<Value>(&data).map_err(|error| {
                    GatewayDispatchError::Transport(format!(
                        "gateway events contained invalid JSON: {error}"
                    ))
                })?;
                on_event(value);
            }
        }
    }
    Ok(())
}
