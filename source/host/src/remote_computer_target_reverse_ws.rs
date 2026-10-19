use super::{
    validate_reverse_ws_payload_binding, validate_reverse_ws_response_binding, FabushiCoreCredential,
    FabushiCoreTargetClient, RemoteComputerNativeExecutor, RemoteComputerTargetError,
    RemoteComputerToolAuthorizer, RemoteComputerToolCatalog, RemoteTargetCancellation,
    ReverseWsIdentity, REMOTE_TARGET_FRAME_MAX_BYTES, REVERSE_WS_CAPABILITY,
    REVERSE_WS_FALLBACK_ENV,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{interval_at, sleep, timeout, Instant, MissedTickBehavior};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};

const REMOTE_WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REMOTE_WS_REGISTER_TIMEOUT: Duration = Duration::from_secs(10);
const REMOTE_WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
const REMOTE_WS_BINDING_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const REMOTE_WS_STOP_POLL_INTERVAL: Duration = Duration::from_millis(250);
const REMOTE_WS_MAX_PENDING: usize = 16;
const REMOTE_WS_MAX_BACKOFF: Duration = Duration::from_secs(30);
const REMOTE_WS_LEASE_SECONDS: u64 = 120;
const REMOTE_WS_MAX_PAYLOAD_BYTES: usize = REMOTE_TARGET_FRAME_MAX_BYTES;

type ReverseWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(super) struct ReverseWsManager {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl ReverseWsManager {
    pub(super) fn start(
        client: Arc<FabushiCoreTargetClient>,
        catalog: Arc<dyn RemoteComputerToolCatalog>,
        authorizer: Arc<dyn RemoteComputerToolAuthorizer>,
        executor: Arc<dyn RemoteComputerNativeExecutor>,
    ) -> Result<Self, RemoteComputerTargetError> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let join = thread::Builder::new()
            .name("fabushi-fdn-reverse-ws".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
                if let Ok(runtime) = runtime {
                    runtime.block_on(run_manager(client, catalog, authorizer, executor, stop_thread));
                }
            })
            .map_err(|_| RemoteComputerTargetError::TransportFailed)?;
        Ok(Self { stop, join: Some(join) })
    }
}

impl Drop for ReverseWsManager {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct PendingPrivateRequest {
    cancellation: Arc<RemoteTargetCancellation>,
    device_id: String,
    network_generation: i64,
    session_id: Option<String>,
    payload_request_id: Option<String>,
}

struct PrivateRequestResponse {
    outer_request_id: String,
    result: Result<Value, RemoteComputerTargetError>,
}

enum SessionEvent {
    Stop,
    Heartbeat,
    RefreshBinding,
    Response(Option<PrivateRequestResponse>),
    Message(Option<Result<Message, tokio_tungstenite::tungstenite::Error>>),
}

async fn run_manager(
    client: Arc<FabushiCoreTargetClient>,
    catalog: Arc<dyn RemoteComputerToolCatalog>,
    authorizer: Arc<dyn RemoteComputerToolAuthorizer>,
    executor: Arc<dyn RemoteComputerNativeExecutor>,
    stop: Arc<AtomicBool>,
) {
    let mut backoff = Duration::from_secs(1);
    while !stop.load(Ordering::Acquire) {
        if !fallback_enabled() {
            sleep(REMOTE_WS_STOP_POLL_INTERVAL).await;
            continue;
        }
        let (credential, identity) = match client.reverse_ws_context() {
            Ok(context) => context,
            Err(_) => {
                sleep(backoff).await;
                backoff = backoff.saturating_mul(2).min(REMOTE_WS_MAX_BACKOFF);
                continue;
            }
        };
        let endpoint = match client.reverse_websocket_endpoint() {
            Ok(endpoint) => endpoint,
            Err(_) => {
                sleep(backoff).await;
                backoff = backoff.saturating_mul(2).min(REMOTE_WS_MAX_BACKOFF);
                continue;
            }
        };
        match connect_and_register(&endpoint, &credential, &identity).await {
            Ok(mut socket) => {
                backoff = Duration::from_secs(1);
                let _ = run_session(
                    &mut socket,
                    Arc::clone(&client),
                    Arc::clone(&catalog),
                    Arc::clone(&authorizer),
                    Arc::clone(&executor),
                    identity,
                    Arc::clone(&stop),
                )
                .await;
                let _ = socket.close(None).await;
            }
            Err(_) => {
                sleep(backoff).await;
                backoff = backoff.saturating_mul(2).min(REMOTE_WS_MAX_BACKOFF);
            }
        }
    }
}

fn fallback_enabled() -> bool {
    std::env::var(REVERSE_WS_FALLBACK_ENV).ok().as_deref() == Some("1")
}

async fn connect_and_register(
    endpoint: &url::Url,
    credential: &FabushiCoreCredential,
    identity: &ReverseWsIdentity,
) -> Result<ReverseWebSocket, ()> {
    if credential.device_id != identity.device_id || credential.user_id != identity.account_id {
        return Err(());
    }
    let mut request = endpoint.as_str().into_client_request().map_err(|_| ())?;
    let authorization = format!("Bearer {}", credential.access_token());
    let header = HeaderValue::from_str(&authorization).map_err(|_| ())?;
    request.headers_mut().insert(AUTHORIZATION, header);
    let config = WebSocketConfig {
        max_message_size: Some(REMOTE_WS_MAX_PAYLOAD_BYTES),
        max_frame_size: Some(REMOTE_WS_MAX_PAYLOAD_BYTES),
        ..WebSocketConfig::default()
    };
    let (mut socket, _) = timeout(
        REMOTE_WS_CONNECT_TIMEOUT,
        connect_async_with_config(request, Some(config), false),
    )
    .await
    .map_err(|_| ())?
    .map_err(|_| ())?;

    send_json(&mut socket, &json!({
        "type": "register",
        "deviceId": identity.device_id,
        "name": "Fabushi Desktop",
        "platform": std::env::consts::OS,
        "capabilities": [REVERSE_WS_CAPABILITY],
        "tools": [],
        "leaseSeconds": REMOTE_WS_LEASE_SECONDS,
        "metadata": { "kind": "fabushi-fdn-target" }
    }))
    .await?;

    let acknowledgement = timeout(REMOTE_WS_REGISTER_TIMEOUT, receive_json(&mut socket))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;
    if acknowledgement.get("type").and_then(Value::as_str) != Some("registered")
        || acknowledgement.get("deviceId").and_then(Value::as_str) != Some(identity.device_id.as_str())
    {
        return Err(());
    }
    let expires_at = acknowledgement.get("expiresAt").and_then(Value::as_str).ok_or(())?;
    let parsed_expiry = chrono::DateTime::parse_from_rfc3339(expires_at).map_err(|_| ())?;
    if parsed_expiry.timestamp_millis() <= chrono::Utc::now().timestamp_millis() {
        return Err(());
    }
    Ok(socket)
}

async fn receive_json(socket: &mut ReverseWebSocket) -> Result<Value, ()> {
    loop {
        match socket.next().await {
            Some(Ok(Message::Text(text))) => return serde_json::from_str(text.as_ref()).map_err(|_| ()),
            Some(Ok(Message::Ping(payload))) => socket.send(Message::Pong(payload)).await.map_err(|_| ())?,
            Some(Ok(Message::Pong(_))) => continue,
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return Err(()),
            Some(Ok(Message::Binary(_))) | Some(Ok(Message::Frame(_))) => return Err(()),
        }
    }
}

async fn send_json(socket: &mut ReverseWebSocket, value: &Value) -> Result<(), ()> {
    let encoded = serde_json::to_string(value).map_err(|_| ())?;
    if encoded.len() > REMOTE_WS_MAX_PAYLOAD_BYTES {
        return Err(());
    }
    socket.send(Message::Text(encoded.into())).await.map_err(|_| ())
}

async fn run_session(
    socket: &mut ReverseWebSocket,
    client: Arc<FabushiCoreTargetClient>,
    catalog: Arc<dyn RemoteComputerToolCatalog>,
    authorizer: Arc<dyn RemoteComputerToolAuthorizer>,
    executor: Arc<dyn RemoteComputerNativeExecutor>,
    identity: ReverseWsIdentity,
    stop: Arc<AtomicBool>,
) -> Result<(), ()> {
    let (response_tx, mut response_rx) = mpsc::unbounded_channel::<PrivateRequestResponse>();
    let mut pending: HashMap<String, PendingPrivateRequest> = HashMap::new();
    let mut heartbeat = interval_at(Instant::now() + REMOTE_WS_HEARTBEAT_INTERVAL, REMOTE_WS_HEARTBEAT_INTERVAL);
    let mut refresh = interval_at(Instant::now() + REMOTE_WS_BINDING_REFRESH_INTERVAL, REMOTE_WS_BINDING_REFRESH_INTERVAL);
    let mut stop_poll = interval_at(Instant::now() + REMOTE_WS_STOP_POLL_INTERVAL, REMOTE_WS_STOP_POLL_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    refresh.set_missed_tick_behavior(MissedTickBehavior::Delay);
    stop_poll.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        let event = tokio::select! {
            _ = stop_poll.tick() => SessionEvent::Stop,
            _ = heartbeat.tick() => SessionEvent::Heartbeat,
            _ = refresh.tick() => SessionEvent::RefreshBinding,
            response = response_rx.recv() => SessionEvent::Response(response),
            message = socket.next() => SessionEvent::Message(message),
        };
        match event {
            SessionEvent::Stop => {
                if stop.load(Ordering::Acquire) || !fallback_enabled() {
                    break;
                }
            }
            SessionEvent::Heartbeat => {
                send_json(socket, &json!({"type":"heartbeat"})).await?;
            }
            SessionEvent::RefreshBinding => {
                let (_, current) = client.reverse_ws_context().map_err(|_| ())?;
                if !identity.same_connection(&current) {
                    break;
                }
            }
            SessionEvent::Response(Some(response)) => {
                let Some(request) = pending.remove(&response.outer_request_id) else {
                    continue;
                };
                match response.result {
                    Ok(result) if response_matches_pending(&request, &result) => {
                        let encoded = serde_json::to_vec(&result).map_err(|_| ())?;
                        if encoded.len() > REMOTE_WS_MAX_PAYLOAD_BYTES {
                            send_private_error(
                                socket,
                                &response.outer_request_id,
                                "outcome_unknown",
                                "private result exceeded the bounded transport size",
                            )
                            .await?;
                        } else {
                            send_json(socket, &json!({
                                "type":"fdnPrivateResult",
                                "requestId":response.outer_request_id,
                                "ok":true,
                                "result":result
                            })).await?;
                        }
                    }
                    Ok(_) => {
                        let dispatch_state = request.cancellation.phase_label();
                        let dispatch_state = if dispatch_state == "pending" || dispatch_state == "cancelled_before_dispatch" {
                            "not_dispatched"
                        } else {
                            "outcome_unknown"
                        };
                        send_private_error(
                            socket,
                            &response.outer_request_id,
                            dispatch_state,
                            "private response failed device/session/generation correlation",
                        )
                        .await?;
                    }
                    Err(error) => {
                        let dispatch_state = if error == RemoteComputerTargetError::OutcomeUnknown {
                            "outcome_unknown"
                        } else {
                            let phase = request.cancellation.phase_label();
                            if phase == "pending" || phase == "cancelled_before_dispatch" {
                                "not_dispatched"
                            } else {
                                "outcome_unknown"
                            }
                        };
                        send_private_error(
                            socket,
                            &response.outer_request_id,
                            dispatch_state,
                            "target rejected the private request",
                        )
                        .await?;
                    }
                }
            }
            SessionEvent::Response(None) => break,
            SessionEvent::Message(Some(Ok(Message::Text(text)))) => {
                let message: Value = match serde_json::from_str(text.as_ref()) {
                    Ok(message) => message,
                    Err(_) => break,
                };
                match message.get("type").and_then(Value::as_str) {
                    Some("fdnPrivate") => {
                        accept_private_request(
                            socket,
                            &message,
                            &identity,
                            &mut pending,
                            &response_tx,
                            &client,
                            &catalog,
                            &authorizer,
                            &executor,
                        ).await?;
                    }
                    Some("fdnPrivateCancel") => {
                        acknowledge_cancel(socket, &message, &identity, &pending).await?;
                    }
                    _ => break,
                }
            }
            SessionEvent::Message(Some(Ok(Message::Ping(payload)))) => {
                socket.send(Message::Pong(payload)).await.map_err(|_| ())?;
            }
            SessionEvent::Message(Some(Ok(Message::Pong(_)))) => {}
            SessionEvent::Message(Some(Ok(Message::Close(_))))
            | SessionEvent::Message(None)
            | SessionEvent::Message(Some(Err(_)))
            | SessionEvent::Message(Some(Ok(Message::Binary(_))))
            | SessionEvent::Message(Some(Ok(Message::Frame(_)))) => break,
        }
    }

    for request in pending.values() {
        let _ = request.cancellation.cancel_before_dispatch();
    }
    pending.clear();
    Ok(())
}

async fn accept_private_request(
    socket: &mut ReverseWebSocket,
    message: &Value,
    identity: &ReverseWsIdentity,
    pending: &mut HashMap<String, PendingPrivateRequest>,
    response_tx: &mpsc::UnboundedSender<PrivateRequestResponse>,
    client: &Arc<FabushiCoreTargetClient>,
    catalog: &Arc<dyn RemoteComputerToolCatalog>,
    authorizer: &Arc<dyn RemoteComputerToolAuthorizer>,
    executor: &Arc<dyn RemoteComputerNativeExecutor>,
) -> Result<(), ()> {
    let outer_request_id = message.get("requestId").and_then(Value::as_str).ok_or(())?.to_string();
    if !valid_identifier(&outer_request_id) || pending.contains_key(&outer_request_id) {
        return Err(());
    }
    let Some(payload) = message.get("payload").filter(|value| value.is_object()).cloned() else {
        send_private_error(socket, &outer_request_id, "not_dispatched", "invalid private payload").await?;
        return Ok(());
    };
    if validate_reverse_ws_payload_binding(&payload, identity).is_err() {
        send_private_error(socket, &outer_request_id, "not_dispatched", "private payload identity binding failed").await?;
        return Ok(());
    }
    if pending.len() >= REMOTE_WS_MAX_PENDING {
        send_private_error(socket, &outer_request_id, "not_dispatched", "target private request capacity reached").await?;
        return Ok(());
    }

    let cancellation = Arc::new(RemoteTargetCancellation::new());
    let request = PendingPrivateRequest {
        cancellation: Arc::clone(&cancellation),
        device_id: identity.device_id.clone(),
        network_generation: identity.network_generation,
        session_id: payload.get("sessionId").and_then(Value::as_str).map(str::to_string),
        payload_request_id: payload.get("requestId").and_then(Value::as_str).map(str::to_string),
    };
    pending.insert(outer_request_id.clone(), request);
    let client = Arc::clone(client);
    let catalog = Arc::clone(catalog);
    let authorizer = Arc::clone(authorizer);
    let executor = Arc::clone(executor);
    let response_tx = response_tx.clone();
    let identity = identity.clone();
    let request_id = outer_request_id.clone();
    let spawned = thread::Builder::new()
        .name("fabushi-fdn-private-request".into())
        .spawn(move || {
            let result = client.serve_private_payload(
                &payload,
                &identity,
                catalog.as_ref(),
                authorizer.as_ref(),
                executor.as_ref(),
                cancellation.as_ref(),
            );
            let _ = response_tx.send(PrivateRequestResponse { outer_request_id: request_id, result });
        });
    if spawned.is_err() {
        pending.remove(&outer_request_id);
        send_private_error(socket, &outer_request_id, "not_dispatched", "could not schedule private request").await?;
    }
    Ok(())
}

async fn acknowledge_cancel(
    socket: &mut ReverseWebSocket,
    message: &Value,
    identity: &ReverseWsIdentity,
    pending: &HashMap<String, PendingPrivateRequest>,
) -> Result<(), ()> {
    let request_id = message.get("requestId").and_then(Value::as_str).ok_or(())?.to_string();
    if !valid_identifier(&request_id) {
        return Err(());
    }
    let mut acknowledgement = json!({
        "type": "fdnPrivateCancelResult",
        "requestId": request_id,
        "deviceId": identity.device_id,
        "networkGeneration": identity.network_generation,
        "confirmed": false,
        "status": "not_found"
    });
    if let Some(request) = pending.get(&request_id) {
        let confirmed = request.cancellation.cancel_before_dispatch();
        acknowledgement["confirmed"] = Value::Bool(confirmed);
        acknowledgement["status"] = Value::String(if confirmed {
            "cancelled_before_dispatch".into()
        } else {
            "outcome_unknown".into()
        });
        acknowledgement["deviceId"] = Value::String(request.device_id.clone());
        acknowledgement["networkGeneration"] = json!(request.network_generation);
        if let Some(session_id) = &request.session_id {
            acknowledgement["sessionId"] = Value::String(session_id.clone());
        }
        if let Some(payload_request_id) = &request.payload_request_id {
            acknowledgement["payloadRequestId"] = Value::String(payload_request_id.clone());
        }
    }
    send_json(socket, &acknowledgement).await
}

async fn send_private_error(
    socket: &mut ReverseWebSocket,
    request_id: &str,
    dispatch_state: &str,
    error: &str,
) -> Result<(), ()> {
    send_json(socket, &json!({
        "type": "fdnPrivateResult",
        "requestId": request_id,
        "ok": false,
        "dispatchState": dispatch_state,
        "error": error
    })).await
}

fn valid_identifier(value: &str) -> bool {
    (8..=160).contains(&value.len())
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn response_matches_pending(pending: &PendingPrivateRequest, result: &Value) -> bool {
    if !result.is_object()
        || result.get("deviceId").and_then(Value::as_str) != Some(pending.device_id.as_str())
        || result.get("networkGeneration").and_then(Value::as_i64) != Some(pending.network_generation)
    {
        return false;
    }
    if let Some(session_id) = &pending.session_id {
        if result.get("sessionId").and_then(Value::as_str) != Some(session_id.as_str()) {
            return false;
        }
    }
    if let Some(request_id) = &pending.payload_request_id {
        if result.get("requestId").and_then(Value::as_str) != Some(request_id.as_str()) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cancellation_is_confirmed_only_before_dispatch() {
        let queued = RemoteTargetCancellation::new();
        assert!(queued.cancel_before_dispatch());
        assert!(!queued.try_begin());
        assert_eq!(queued.phase_label(), "cancelled_before_dispatch");

        let executing = RemoteTargetCancellation::new();
        assert!(executing.try_begin());
        assert!(!executing.cancel_before_dispatch());
        executing.finish(false);
        assert_eq!(executing.phase_label(), "completed");

        let uncertain = RemoteTargetCancellation::new();
        assert!(uncertain.try_begin());
        uncertain.finish_outcome_unknown();
        assert!(!uncertain.cancel_before_dispatch());
        assert_eq!(uncertain.phase_label(), "outcome_unknown");
    }

    #[test]
    fn reverse_websocket_requires_payload_and_response_identity_correlation() {
        let identity = ReverseWsIdentity {
            account_id: "account-1".into(),
            device_id: "device-1".into(),
            access_token_sha256: "token-digest".into(),
            network_generation: 7,
        };
        let payload = json!({
            "deviceId":"device-1",
            "networkGeneration":7,
            "sessionId":"session-123",
            "requestId":"request-123"
        });
        assert!(validate_reverse_ws_payload_binding(&payload, &identity).is_ok());
        let result = json!({
            "deviceId":"device-1",
            "networkGeneration":7,
            "sessionId":"session-123",
            "requestId":"request-123"
        });
        assert!(validate_reverse_ws_response_binding(&payload, &result));
        assert!(!validate_reverse_ws_response_binding(
            &payload,
            &json!({"deviceId":"device-2","networkGeneration":7,"sessionId":"session-123","requestId":"request-123"})
        ));
        assert!(validate_reverse_ws_payload_binding(
            &json!({"deviceId":"device-1","networkGeneration":8,"sessionId":"session-123","requestId":"request-123"}),
            &identity
        ).is_err());
    }

    #[test]
    fn reverse_websocket_url_is_tls_by_default_and_only_allows_plain_loopback() {
        let secure = url::Url::parse("https://api.example.test/base").expect("valid URL");
        let secure_ws = super::super::reverse_websocket_url(&secure).expect("secure WS URL");
        assert_eq!(secure_ws.scheme(), "wss");
        assert_eq!(secure_ws.path(), "/agent");

        let loopback = url::Url::parse("http://127.0.0.1:8000").expect("loopback URL");
        let loopback_ws = super::super::reverse_websocket_url(&loopback).expect("loopback WS URL");
        assert_eq!(loopback_ws.scheme(), "ws");

        let external = url::Url::parse("http://api.example.test").expect("external URL");
        assert!(super::super::reverse_websocket_url(&external).is_err());
    }
}
