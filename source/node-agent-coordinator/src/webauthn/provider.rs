use std::collections::HashSet;
use std::io::{Read, Write};
use std::time::Duration;

use serde_json::{Value, json};

use crate::gateway::gateway_client::{SSE_CONNECT_TIMEOUT_MS, SSE_STALL_TIMEOUT_MS};
use crate::gateway::gateway_reachability::ReachabilityOutcome;
use crate::gateway::gateway_request_dispatcher::GatewayDispatchError;
use crate::gateway::host_supervisor::GatewayConnection;
use crate::gateway::http_transport::{parse_gateway_http_base, read_http_response};
use crate::gateway::sse_block_decoder::SseBlockDecoder;

use crate::webauthn::{
    ApprovedWebAuthnConsent, WebAuthnCeremony, WebAuthnSigner, WebAuthnSignerError,
    WebAuthnSignerResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WebAuthnRequestFrame {
    Welcome { provider_id: String },
    Ceremony {
        request_id: String,
        ceremony: WebAuthnCeremony,
    },
    Cancel { request_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebAuthnResponseFrame {
    Hello {
        computer_id: Option<String>,
        label: Option<String>,
    },
    Stage {
        request_id: String,
        stage: &'static str,
        outcome: &'static str,
    },
    Result {
        request_id: String,
        credential_json: Value,
    },
    Error {
        request_id: String,
        error: WebAuthnSignerError,
    },
    Ping,
}

#[derive(Debug)]
pub struct WebAuthnProvider<S: WebAuthnSigner> {
    signer: S,
    computer_id: Option<String>,
    label: Option<String>,
    provider_id: Option<String>,
    in_flight: HashSet<String>,
}

impl<S: WebAuthnSigner> WebAuthnProvider<S> {
    pub fn new(
        signer: S,
        computer_id: Option<String>,
        label: Option<String>,
    ) -> Self {
        Self {
            signer,
            computer_id,
            label,
            provider_id: None,
            in_flight: HashSet::new(),
        }
    }

    pub fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    pub fn reset_transport(&mut self) {
        self.provider_id = None;
        self.in_flight.clear();
    }

    pub fn heartbeat(&self) -> WebAuthnResponseFrame {
        WebAuthnResponseFrame::Ping
    }

    pub fn handle_frame(
        &mut self,
        frame: WebAuthnRequestFrame,
        consent: Option<ApprovedWebAuthnConsent>,
    ) -> Vec<WebAuthnResponseFrame> {
        match frame {
            WebAuthnRequestFrame::Welcome { provider_id } => {
                self.provider_id = Some(provider_id);
                vec![WebAuthnResponseFrame::Hello {
                    computer_id: self.computer_id.clone(),
                    label: self.label.clone(),
                }]
            }
            WebAuthnRequestFrame::Cancel { request_id } => {
                self.in_flight.remove(&request_id);
                Vec::new()
            }
            WebAuthnRequestFrame::Ceremony {
                request_id,
                ceremony,
            } => self.run_ceremony(request_id, ceremony, consent),
        }
    }

    fn run_ceremony(
        &mut self,
        request_id: String,
        ceremony: WebAuthnCeremony,
        consent: Option<ApprovedWebAuthnConsent>,
    ) -> Vec<WebAuthnResponseFrame> {
        if request_id.trim().is_empty() || !self.in_flight.insert(request_id.clone()) {
            return vec![WebAuthnResponseFrame::Error {
                request_id,
                error: WebAuthnSignerError {
                    name: "InvalidStateError".into(),
                    code: Some("duplicate_request".into()),
                    message: "WebAuthn request id is empty or already active".into(),
                },
            }];
        }

        if consent.as_ref().is_some_and(|value| !value.approved) {
            self.in_flight.remove(&request_id);
            return vec![
                WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "grant",
                    outcome: "declined",
                },
                WebAuthnResponseFrame::Error {
                    request_id,
                    error: WebAuthnSignerError {
                        name: "NotAllowedError".into(),
                        code: Some("consent_declined".into()),
                        message: "The security key request was declined on this computer".into(),
                    },
                },
            ];
        }

        let mut frames = Vec::new();
        if consent.is_some() {
            frames.push(WebAuthnResponseFrame::Stage {
                request_id: request_id.clone(),
                stage: "grant",
                outcome: "ok",
            });
        }

        match self.signer.sign(&ceremony, consent.as_ref()) {
            WebAuthnSignerResult::Success { credential_json } => {
                frames.push(WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "sign",
                    outcome: "ok",
                });
                frames.push(WebAuthnResponseFrame::Result {
                    request_id: request_id.clone(),
                    credential_json,
                });
            }
            WebAuthnSignerResult::Failed { error } => {
                frames.push(WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "sign",
                    outcome: "failed",
                });
                frames.push(WebAuthnResponseFrame::Error {
                    request_id: request_id.clone(),
                    error,
                });
            }
        }
        self.in_flight.remove(&request_id);
        frames
    }
}


pub const GATEWAY_WEBAUTHN_REQUESTS_PATH: &str = "/webauthn/requests";
pub const GATEWAY_WEBAUTHN_RESPONSES_PATH: &str = "/webauthn/responses";
pub const WEBAUTHN_HEARTBEAT_INTERVAL_MS: u64 = 10_000;

fn webauthn_io_outcome(error: &std::io::Error) -> ReachabilityOutcome {
    match error.kind() {
        std::io::ErrorKind::ConnectionRefused => ReachabilityOutcome::Refused,
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ReachabilityOutcome::Timeout,
        std::io::ErrorKind::NotFound | std::io::ErrorKind::AddrNotAvailable => ReachabilityOutcome::Dns,
        _ => ReachabilityOutcome::Network,
    }
}

pub fn parse_webauthn_request_frame(value: Value) -> Result<WebAuthnRequestFrame, String> {
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "WebAuthn request frame has no kind".to_string())?;
    match kind {
        "welcome" => Ok(WebAuthnRequestFrame::Welcome {
            provider_id: value
                .get("providerId")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "WebAuthn welcome frame has no providerId".to_string())?
                .to_string(),
        }),
        "ceremony" => Ok(WebAuthnRequestFrame::Ceremony {
            request_id: value
                .get("requestId")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "WebAuthn ceremony frame has no requestId".to_string())?
                .to_string(),
            ceremony: serde_json::from_value(
                value
                    .get("ceremony")
                    .cloned()
                    .ok_or_else(|| "WebAuthn ceremony frame has no ceremony".to_string())?,
            )
            .map_err(|error| format!("invalid WebAuthn ceremony: {error}"))?,
        }),
        "cancel" => Ok(WebAuthnRequestFrame::Cancel {
            request_id: value
                .get("requestId")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "WebAuthn cancel frame has no requestId".to_string())?
                .to_string(),
        }),
        other => Err(format!("unknown WebAuthn request frame kind {other}")),
    }
}

pub fn webauthn_response_frame_value(frame: &WebAuthnResponseFrame) -> Value {
    match frame {
        WebAuthnResponseFrame::Hello {
            computer_id,
            label,
        } => {
            let mut value = json!({ "kind": "hello" });
            if let Some(computer_id) = computer_id {
                value["computerId"] = Value::String(computer_id.clone());
            }
            if let Some(label) = label {
                value["label"] = Value::String(label.clone());
            }
            value
        }
        WebAuthnResponseFrame::Stage {
            request_id,
            stage,
            outcome,
        } => json!({
            "kind": "stage",
            "requestId": request_id,
            "stage": stage,
            "outcome": outcome,
        }),
        WebAuthnResponseFrame::Result {
            request_id,
            credential_json,
        } => json!({
            "kind": "result",
            "requestId": request_id,
            "credentialJson": credential_json,
        }),
        WebAuthnResponseFrame::Error { request_id, error } => {
            let mut value = json!({
                "kind": "error",
                "requestId": request_id,
                "name": error.name,
                "message": error.message,
            });
            if let Some(code) = &error.code {
                value["code"] = Value::String(code.clone());
            }
            value
        }
        WebAuthnResponseFrame::Ping => json!({ "kind": "ping" }),
    }
}

pub fn post_webauthn_frames(
    connection: &GatewayConnection,
    provider_id: Option<&str>,
    frames: &[WebAuthnResponseFrame],
) -> Result<(), GatewayDispatchError> {
    let endpoint = parse_gateway_http_base(&connection.base_url)
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    let addresses = endpoint.socket_addrs().map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: format!("WebAuthn response endpoint unreachable (dns): {error}"),
        }
    })?;
    if addresses.is_empty() {
        return Err(GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: "WebAuthn response endpoint unreachable (dns)".into(),
        });
    }

    let timeout = Duration::from_millis(SSE_CONNECT_TIMEOUT_MS);
    let mut stream = endpoint.connect_any(&addresses, timeout).map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: webauthn_io_outcome(&error),
            message: format!("WebAuthn response endpoint connect failed: {error}"),
        }
    })?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;

    let mut batch = json!({
        "frames": frames
            .iter()
            .map(webauthn_response_frame_value)
            .collect::<Vec<_>>(),
    });
    if let Some(provider_id) = provider_id.filter(|provider_id| !provider_id.is_empty()) {
        batch["providerId"] = Value::String(provider_id.to_string());
    }
    let body = serde_json::to_vec(&batch)
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    let path = format!("{}{}", endpoint.base_path, GATEWAY_WEBAUTHN_RESPONSES_PATH);
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        endpoint.host,
        body.len()
    )
    .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    for (name, value) in &connection.headers {
        write!(stream, "{name}: {value}\r\n")
            .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    }
    write!(stream, "\r\n")
        .and_then(|_| stream.write_all(&body))
        .and_then(|_| stream.flush())
        .map_err(|error| GatewayDispatchError::Unreachable {
            outcome: webauthn_io_outcome(&error),
            message: format!("WebAuthn response delivery failed: {error}"),
        })?;

    let response = read_http_response(&mut stream, 64 * 1024, 1024 * 1024).map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: webauthn_io_outcome(&error),
            message: format!("WebAuthn response acknowledgement failed: {error}"),
        }
    })?;
    if (200..300).contains(&response.status) {
        return Ok(());
    }
    if matches!(response.status, 401 | 403) {
        return Err(GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::AccessDenied,
            message: format!(
                "WebAuthn response endpoint rejected authorization with HTTP {}",
                response.status
            ),
        });
    }
    if response.status >= 500 {
        return Err(GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Http(response.status),
            message: format!(
                "WebAuthn response endpoint failed with HTTP {}",
                response.status
            ),
        });
    }
    Err(GatewayDispatchError::Transport(format!(
        "WebAuthn response endpoint rejected the batch with HTTP {}",
        response.status
    )))
}

fn webauthn_sse_data(block: &str) -> Option<String> {
    let data = block
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>();
    (!data.is_empty()).then(|| data.join("\n"))
}

pub fn stream_webauthn_requests<Connected, Frame, Continue>(
    connection: &GatewayConnection,
    on_connected: Connected,
    mut on_frame: Frame,
    should_continue: Continue,
) -> Result<(), GatewayDispatchError>
where
    Connected: FnOnce(),
    Frame: FnMut(WebAuthnRequestFrame),
    Continue: Fn() -> bool,
{
    let endpoint = parse_gateway_http_base(&connection.base_url)
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    let addresses = endpoint.socket_addrs().map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: format!("WebAuthn request stream unreachable (dns): {error}"),
        }
    })?;
    if addresses.is_empty() {
        return Err(GatewayDispatchError::Unreachable {
            outcome: ReachabilityOutcome::Dns,
            message: "WebAuthn request stream unreachable (dns)".into(),
        });
    }

    let connect_timeout = Duration::from_millis(SSE_CONNECT_TIMEOUT_MS);
    let mut stream = endpoint.connect_any(&addresses, connect_timeout).map_err(|error| {
        GatewayDispatchError::Unreachable {
            outcome: webauthn_io_outcome(&error),
            message: format!("WebAuthn request stream connect failed: {error}"),
        }
    })?;
    stream
        .set_write_timeout(Some(connect_timeout))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(SSE_STALL_TIMEOUT_MS)))
        .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;

    let path = format!("{}{}", endpoint.base_path, GATEWAY_WEBAUTHN_REQUESTS_PATH);
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: text/event-stream\r\nConnection: keep-alive\r\n",
        endpoint.host
    )
    .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    for (name, value) in &connection.headers {
        write!(stream, "{name}: {value}\r\n")
            .map_err(|error| GatewayDispatchError::Transport(error.to_string()))?;
    }
    write!(stream, "\r\n")
        .and_then(|_| stream.flush())
        .map_err(|error| GatewayDispatchError::Unreachable {
            outcome: webauthn_io_outcome(&error),
            message: format!("WebAuthn request stream handshake failed: {error}"),
        })?;

    let mut buffered = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| {
            GatewayDispatchError::Unreachable {
                outcome: webauthn_io_outcome(&error),
                message: format!("WebAuthn request stream handshake failed: {error}"),
            }
        })?;
        if count == 0 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "WebAuthn request stream ended during handshake".into(),
            });
        }
        buffered.extend_from_slice(&chunk[..count]);
        if let Some(index) = buffered.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        if buffered.len() > 64 * 1024 {
            return Err(GatewayDispatchError::Transport(
                "WebAuthn request stream response headers are too large".into(),
            ));
        }
    };

    let headers = std::str::from_utf8(&buffered[..header_end]).map_err(|_| {
        GatewayDispatchError::Transport(
            "WebAuthn request stream response headers are not UTF-8".into(),
        )
    })?;
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| {
            GatewayDispatchError::Transport(
                "WebAuthn request stream response has no HTTP status".into(),
            )
        })?;
    if !(200..300).contains(&status) {
        if matches!(status, 401 | 403) {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::AccessDenied,
                message: format!("WebAuthn request stream failed with HTTP {status}"),
            });
        }
        if status >= 500 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Http(status),
                message: format!("WebAuthn request stream failed with HTTP {status}"),
            });
        }
        return Err(GatewayDispatchError::Transport(format!(
            "WebAuthn request stream failed with HTTP {status}"
        )));
    }

    on_connected();
    let mut decoder = SseBlockDecoder::default();
    let deliver = |decoder: &mut SseBlockDecoder,
                   bytes: &[u8],
                   on_frame: &mut Frame| {
        for block in decoder.push_bytes(bytes) {
            let Some(data) = webauthn_sse_data(&block) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(&data) else {
                continue;
            };
            if let Ok(frame) = parse_webauthn_request_frame(value) {
                on_frame(frame);
            }
        }
    };
    deliver(&mut decoder, &buffered[header_end..], &mut on_frame);
    if !should_continue() {
        return Ok(());
    }

    loop {
        if !should_continue() {
            return Ok(());
        }
        let count = stream.read(&mut chunk).map_err(|error| {
            GatewayDispatchError::Unreachable {
                outcome: webauthn_io_outcome(&error),
                message: format!("WebAuthn request stream failed: {error}"),
            }
        })?;
        if count == 0 {
            return Err(GatewayDispatchError::Unreachable {
                outcome: ReachabilityOutcome::Network,
                message: "WebAuthn request stream closed".into(),
            });
        }
        deliver(&mut decoder, &chunk[..count], &mut on_frame);
        if !should_continue() {
            return Ok(());
        }
    }
}
