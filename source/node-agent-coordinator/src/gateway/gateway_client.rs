use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::Value;

use super::gateway_reachability::ReachabilityOutcome;
use super::gateway_request_dispatcher::GatewayDispatchError;
use super::host_supervisor::GatewayConnection;

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
