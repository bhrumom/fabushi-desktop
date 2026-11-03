use std::collections::{BTreeMap, VecDeque};
use std::io::{self, BufReader, Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use flate2::{Compression, write::GzEncoder};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use crate::gateway_command_error::{
    GatewayCommandErrorClassification, classify_gateway_command_error,
};
pub use crate::gateway_command_error::GatewayCommandError;
use crate::gateway_config::{GatewayServerConfig, GatewayTlsConfig, is_loopback_host};
use crate::device_pairing_owner::{
    DeviceTokenAuthorization, is_device_pairing_join_code, parse_device_pairing_join_request_path,
};
use crate::gateway_protocol::{
    GATEWAY_PREPARE_UPGRADE_PATH, is_grok_gateway_command, parse_command_args,
    slim_command_result, slim_event,
};
use crate::host_gateway_api::{
    FABUSHI_NATIVE_ATTACHMENTS_GATEWAY_METHODS, FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS,
};
use crate::extensions::mcp::coordinator_relay::{
    HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
};
use crate::runner::coordinator_tool_relay::{
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};

pub const GATEWAY_API_PREFIX: &str = "/api";
pub const GATEWAY_EVENTS_PATH: &str = "/events";
pub const GATEWAY_HEALTH_PATH: &str = "/health";
pub const GATEWAY_AUTH_SCHEME: &str = "Bearer";
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
pub const GZIP_MIN_BYTES: usize = 1_400;
pub const DISABLE_SSE_GZIP_ENV: &str = "SAND_DISABLE_GATEWAY_SSE_GZIP";
pub const MAX_REQUEST_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_BODY_BYTES: usize = MAX_REQUEST_PAYLOAD_BYTES * 4 / 3 + 64 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;

const FABUSHI_EXTENSION_COMMANDS: &[&str] = &[
    // Explicit Fabushi product extensions carried through the Grok Host
    // Gateway boundary. Keep these separate from the reconstructed Grok
    // command inventory so the 1:1 architecture remains auditable.
    "feature.info",
    "feature.execute",
    "feature.marketplace.browse",
    "feature.marketplace.release",
    "feature.plugin.install",
    "feature.plugin.uninstall",
    "feature.plugin.rollback",
    "feature.plugin.active",
    "feature.plugin.listInstalled",
    "feature.plugin.uiDocument",
    "feature.auth.status",
    "feature.auth.providers",
    "feature.auth.browserStart",
    "feature.auth.browserPoll",
    "feature.auth.browserCancel",
    "feature.auth.browserReopen",
    "feature.auth.passwordLogin",
    "feature.auth.oauthStart",
    "feature.auth.oauthPoll",
    "feature.auth.logout",
    "feature.interrupt",
    "feature.approval.resolve",
    // FBCP-001 Human communication commands. These remain explicit so the
    // existing Session/Transcript owner is reachable through the shipping
    // Coordinator -> Host gateway without opening a generic messaging namespace.
    "listHumanConversations",
    "syncHumanConversations",
    "syncHumanConversation",
    "createHumanConversation",
    "sendHumanMessage",
    "reactHumanMessage",
    "getHumanConversationTranscript",
    "getHumanConversationTranscriptTail",
    "searchHumanMessages",
    // ADR-003 CallSession/Signaling commands. Keep live call truth in the
    // canonical Host owner rather than widening Renderer/Electron ownership.
    "createCallSession",
    "getCallSession",
    "listCallSessions",
    "transitionCallSession",
    "updateCallMedia",
    "sendCallSignal",
    "listCallSignals",
    // Internal desktop MCP lifecycle settlement. The shipping desktop MCP owner
    // answers Host lifecycle requests through this single explicit callback.
    "mcp.resolveLifecycleRequest",
    // Internal Coordinator -> Host -> Runner methods. These are explicit
    // Fabushi architecture extensions and must never widen the runner.*
    // namespace generically.
    "runner.acceptRoutedPrompt",
    "runner.startRoutedProvider",
    "runner.cancelRoutedProvider",
    "runner.resolveRoutedToolRequest",
];



fn is_gateway_command(method: &str) -> bool {
    is_grok_gateway_command(method) || FABUSHI_EXTENSION_COMMANDS.contains(&method)
}



#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GatewayHealth {
    pub is_busy: bool,
    pub busy_only_awaiting_approval: Option<bool>,
    pub active_agent_id: Option<String>,
    pub last_busy_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayCommandReport {
    pub method: String,
    pub duration_ms: u64,
    pub request_id: Option<String>,
    pub traceparent: Option<String>,
    pub status: u16,
    pub error: Option<String>,
    pub reason: Option<String>,
    pub error_class: Option<String>,
    pub errno: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GatewayCommandContext {
    pub request_id: Option<String>,
    pub traceparent: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub authenticated: bool,
    pub dispatch_started: Instant,
}

pub trait GatewayApi: Send + Sync + 'static {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError>;

    /// Resolve a bearer token issued by the canonical device-pairing owner.
    /// Gateway ingress consumes the result but does not inspect pairing state.
    fn authenticate_device_token(
        &self,
        _token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        Ok(None)
    }

    /// Enforce request-level binding for an authenticated device token after
    /// the request body has been decoded. The pairing owner remains the sole
    /// authority for token identity; implementations only bind that identity
    /// to the command's addressed device.
    fn authorize_device_token_request(
        &self,
        _authorization: &DeviceTokenAuthorization,
        _method: &str,
        _args: &Value,
    ) -> Result<(), GatewayCommandError> {
        Ok(())
    }

    /// A recognized paired bearer must retain its owner-issued credential
    /// identity through the rotation transaction. Legacy adapters fail closed;
    /// they must not silently route this request through the native call path.
    fn rotate_paired_device_token(
        &self,
        _args: Value,
        _authorization: &DeviceTokenAuthorization,
        _scope_ceiling: Option<&str>,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        Err(GatewayCommandError::Forbidden(
            "paired token rotation is not supported by this gateway adapter".into(),
        ))
    }

    /// Removal must carry the sealed credential to the existing durable owner.
    /// An adapter without this hook must never use generic native dispatch.
    fn revoke_paired_device(
        &self,
        _args: Value,
        _authorization: &DeviceTokenAuthorization,
        _scope_ceiling: Option<&str>,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        Err(GatewayCommandError::Forbidden(
            "paired device revocation is not supported by this gateway adapter".into(),
        ))
    }

    /// Selected-role retirement keeps both paired proofs and native header
    /// ceilings bound to the durable owner. No caller uses generic dispatch.
    fn revoke_device_token(
        &self,
        _args: Value,
        _authorization: Option<&DeviceTokenAuthorization>,
        _scope_ceiling: Option<&str>,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        Err(GatewayCommandError::Forbidden(
            "device token revocation is not supported by this gateway adapter".into(),
        ))
    }

    /// Retained paired connections may only keep the credential admitted at
    /// ingress. The owner decides currentness; Gateway owns no permission state.
    fn validate_paired_device_authorization(
        &self,
        _authorization: &DeviceTokenAuthorization,
    ) -> Result<(), GatewayCommandError> {
        Err(GatewayCommandError::Forbidden(
            "paired connection validation is not supported by this gateway adapter".into(),
        ))
    }

    fn call_with_context(
        &self,
        method: &str,
        args: Value,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.call(method, args)
    }

    fn on_command_complete(&self, _report: GatewayCommandReport) {}

    fn on_command_error(&self, _report: GatewayCommandReport) {}

    fn health(&self) -> GatewayHealth {
        GatewayHealth::default()
    }
    fn prepare_for_upgrade(&self) -> Result<Value, GatewayCommandError> {
        Ok(json!({ "quiescing": false, "runningTurns": 0 }))
    }

    /// Redeem a public, single-use device-pairing join code. Implementations
    /// that do not own pairing storage keep the route disabled by default.
    fn redeem_device_pairing_join_code(
        &self,
        _shortcode: &str,
    ) -> Result<Option<Value>, GatewayCommandError> {
        Ok(None)
    }
}

pub type GatewayBridgeClose = Box<dyn FnOnce() + Send + 'static>;
type GatewayBridgeSubscribeHandler =
    Arc<dyn Fn(Sender<Value>) -> GatewayBridgeClose + Send + Sync + 'static>;
type GatewayBridgeResponseHandler = Arc<dyn Fn(Value) + Send + Sync + 'static>;

pub struct GatewayBridgeSubscription {
    receiver: Receiver<Value>,
    close: Option<GatewayBridgeClose>,
}

impl GatewayBridgeSubscription {
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Value, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}

impl Drop for GatewayBridgeSubscription {
    fn drop(&mut self) {
        if let Some(close) = self.close.take() {
            close();
        }
    }
}

#[derive(Clone)]
pub struct GatewayBridgeHub {
    request_subscribers: Arc<Mutex<Vec<Sender<Value>>>>,
    response_subscribers: Arc<Mutex<Vec<Sender<Value>>>>,
    on_request_subscribe: Option<GatewayBridgeSubscribeHandler>,
    on_response: Option<GatewayBridgeResponseHandler>,
}

impl Default for GatewayBridgeHub {
    fn default() -> Self {
        Self {
            request_subscribers: Arc::new(Mutex::new(Vec::new())),
            response_subscribers: Arc::new(Mutex::new(Vec::new())),
            on_request_subscribe: None,
            on_response: None,
        }
    }
}

impl GatewayBridgeHub {
    pub fn with_handlers<Subscribe, Response>(
        on_request_subscribe: Subscribe,
        on_response: Response,
    ) -> Self
    where
        Subscribe: Fn(Sender<Value>) -> GatewayBridgeClose + Send + Sync + 'static,
        Response: Fn(Value) + Send + Sync + 'static,
    {
        Self {
            on_request_subscribe: Some(Arc::new(on_request_subscribe)),
            on_response: Some(Arc::new(on_response)),
            ..Self::default()
        }
    }
    pub fn publish_request(&self, frame: Value) {
        if let Ok(mut subscribers) = self.request_subscribers.lock() {
            subscribers.retain(|subscriber| subscriber.send(frame.clone()).is_ok());
        }
    }

    pub fn subscribe_requests(&self) -> GatewayBridgeSubscription {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut subscribers) = self.request_subscribers.lock() {
            subscribers.push(sender.clone());
        }
        let close = self
            .on_request_subscribe
            .as_ref()
            .map(|handler| handler(sender));
        GatewayBridgeSubscription { receiver, close }
    }

    pub fn submit_responses(&self, batch: Value) {
        if let Some(handler) = self.on_response.as_ref() {
            handler(batch.clone());
        }
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

pub type GatewayEventHub = crate::host_event_bus::SandHostEventBus;

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

#[derive(Clone, Copy)]
struct JoinLimitPolicy {
    failures: usize,
    window: Duration,
    lockout: Duration,
    entries: usize,
}

impl Default for JoinLimitPolicy {
    fn default() -> Self {
        Self {
            failures: 10,
            window: Duration::from_secs(60),
            lockout: Duration::from_secs(300),
            entries: 10_000,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum JoinRejection {
    Limited(Duration),
    Unavailable,
}

#[derive(Default)]
struct JoinPeerState {
    failures: VecDeque<Instant>,
    locked_until: Option<Instant>,
}

impl JoinPeerState {
    fn refresh(&mut self, now: Instant, policy: JoinLimitPolicy) {
        if let Some(until) = self.locked_until {
            if now < until {
                return;
            }
            self.locked_until = None;
            self.failures.clear();
        }
        while self.failures.front().is_some_and(|at| {
            now.saturating_duration_since(*at) >= policy.window
        }) {
            self.failures.pop_front();
        }
    }

    fn check(&mut self, now: Instant, policy: JoinLimitPolicy) -> Result<(), JoinRejection> {
        self.refresh(now, policy);
        match self.locked_until {
            Some(until) => Err(JoinRejection::Limited(until.duration_since(now))),
            None => Ok(()),
        }
    }

    fn fail(&mut self, now: Instant, policy: JoinLimitPolicy) {
        self.refresh(now, policy);
        if self.locked_until.is_some() {
            return;
        }
        self.failures.push_back(now);
        if self.failures.len() == policy.failures {
            self.locked_until = Some(now + policy.lockout);
        }
    }

    fn reset(&mut self) {
        self.failures.clear();
        self.locked_until = None;
    }
}

fn join_peer_ip(peer: IpAddr) -> IpAddr {
    match peer {
        IpAddr::V6(ip) => ip.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(peer),
        _ => peer,
    }
}

fn join_retry_seconds(delay: Duration) -> u64 {
    (delay.as_secs() + u64::from(delay.subsec_nanos() != 0)).max(1)
}

#[derive(Default)]
struct JoinTurnState {
    serving: u64,
    failed: bool,
}

#[derive(Default)]
struct JoinPeer {
    state: Mutex<JoinPeerState>,
    turns: Mutex<JoinTurnState>,
    ready: Condvar,
}

#[derive(Default)]
struct JoinEntry {
    peer: Arc<JoinPeer>,
    next_ticket: u64,
}

impl JoinEntry {
    // Called only inside the short map critical section: observable reservation
    // order must not depend on which waiting thread wins a mutex next.
    fn reserve(&mut self) -> Result<JoinReservation, JoinRejection> {
        let ticket = self.next_ticket;
        self.next_ticket = ticket.checked_add(1).ok_or(JoinRejection::Unavailable)?;
        Ok(JoinReservation { peer: Arc::clone(&self.peer), ticket, has_turn: false })
    }
}

struct JoinReservation {
    peer: Arc<JoinPeer>,
    ticket: u64,
    has_turn: bool,
}

impl JoinReservation {
    fn wait_turn(&mut self) -> Result<(), JoinRejection> {
        let mut turns = match self.peer.turns.lock() {
            Ok(turns) => turns,
            Err(poisoned) => {
                poisoned.into_inner().failed = true;
                self.peer.ready.notify_all();
                return Err(JoinRejection::Unavailable);
            }
        };
        while !turns.failed && self.ticket > turns.serving {
            turns = match self.peer.ready.wait(turns) {
                Ok(turns) => turns,
                Err(poisoned) => {
                    poisoned.into_inner().failed = true;
                    self.peer.ready.notify_all();
                    return Err(JoinRejection::Unavailable);
                }
            };
        }
        if turns.failed || self.ticket != turns.serving {
            turns.failed = true;
            self.peer.ready.notify_all();
            return Err(JoinRejection::Unavailable);
        }
        self.has_turn = true;
        Ok(())
    }
}

impl Drop for JoinReservation {
    fn drop(&mut self) {
        {
            let mut turns = match self.peer.turns.lock() {
                Ok(turns) => turns,
                Err(poisoned) => {
                    let mut turns = poisoned.into_inner();
                    turns.failed = true;
                    turns
                }
            };
            if thread::panicking() || !self.has_turn || turns.serving != self.ticket {
                // An abandoned queue slot cannot leave a permanent hole, and
                // unwinding cannot let a waiter enter possibly poisoned state.
                turns.failed = true;
            } else if !turns.failed {
                match turns.serving.checked_add(1) {
                    Some(next) => turns.serving = next,
                    None => turns.failed = true,
                }
            }
        }
        self.peer.ready.notify_all();
    }
}

// This transient state belongs to one Gateway generation. Each reservation
// pins its peer until settlement; capacity cannot replace live or queued state.
struct JoinRateLimiter {
    policy: JoinLimitPolicy,
    peers: Mutex<BTreeMap<IpAddr, JoinEntry>>,
}

impl JoinRateLimiter {
    fn new(policy: JoinLimitPolicy) -> Self {
        Self { policy, peers: Mutex::new(BTreeMap::new()) }
    }

    fn reserve(&self, peer: IpAddr, now: Instant) -> Result<JoinReservation, JoinRejection> {
        let peer = join_peer_ip(peer);
        let mut peers = self.peers.lock().map_err(|_| JoinRejection::Unavailable)?;
        if let Some(entry) = peers.get_mut(&peer) {
            return entry.reserve();
        }
        if peers.len() >= self.policy.entries {
            let mut idle = None;
            let mut expired = None;
            for (key, entry) in peers.iter() {
                if Arc::strong_count(&entry.peer) != 1 || entry.next_ticket == u64::MAX {
                    continue;
                }
                // Never wait for a peer while holding the global map lock.
                let Ok(turns) = entry.peer.turns.try_lock() else { continue };
                if turns.failed || turns.serving != entry.next_ticket {
                    continue;
                }
                let Ok(mut state) = entry.peer.state.try_lock() else { continue };
                state.refresh(now, self.policy);
                if state.locked_until.is_some() {
                    continue;
                }
                if state.failures.is_empty() {
                    expired = Some(*key);
                    break;
                }
                idle.get_or_insert(*key);
            }
            if let Some(key) = expired.or(idle) {
                peers.remove(&key);
            } else {
                return Err(JoinRejection::Limited(Duration::from_secs(1)));
            }
        }
        let mut entry = JoinEntry::default();
        let reservation = entry.reserve()?;
        peers.insert(peer, entry);
        Ok(reservation)
    }

    fn attempt<F>(
        &self,
        peer: IpAddr,
        stop: &AtomicBool,
        operation: F,
    ) -> Result<Result<Option<Value>, GatewayCommandError>, JoinRejection>
    where F: FnOnce() -> Result<Option<Value>, GatewayCommandError> {
        self.attempt_with_clock(peer, stop, Instant::now, operation)
    }

    fn attempt_with_clock<F, C>(
        &self,
        peer: IpAddr,
        stop: &AtomicBool,
        clock: C,
        operation: F,
    ) -> Result<Result<Option<Value>, GatewayCommandError>, JoinRejection>
    where
        F: FnOnce() -> Result<Option<Value>, GatewayCommandError>,
        C: Fn() -> Instant,
    {
        if stop.load(Ordering::Acquire) {
            return Err(JoinRejection::Unavailable);
        }
        let mut reservation = self.reserve(peer, clock())?;
        reservation.wait_turn()?;
        let mut state = reservation.peer.state.lock().map_err(|_| JoinRejection::Unavailable)?;
        if stop.load(Ordering::Acquire) {
            return Err(JoinRejection::Unavailable);
        }
        state.check(clock(), self.policy)?;
        let outcome = operation();
        match &outcome {
            Ok(Some(_)) => state.reset(),
            Ok(None) => state.fail(clock(), self.policy),
            Err(_) => {}
        }
        // Both the per-peer guard and its reservation end before any HTTP I/O.
        Ok(outcome)
    }
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

fn build_tls_server_config(tls: &GatewayTlsConfig) -> io::Result<Arc<ServerConfig>> {
    let mut cert_reader = BufReader::new(Cursor::new(tls.cert.as_slice()));
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io::Error::other(format!("gateway TLS certificate parse failed: {error}")))?;
    if certs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "gateway TLS certificate file contains no certificates",
        ));
    }
    let mut key_reader = BufReader::new(Cursor::new(tls.key.as_slice()));
    let key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|error| io::Error::other(format!("gateway TLS private key parse failed: {error}")))?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "gateway TLS private key file contains no private key",
            )
        })?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|error| io::Error::other(format!("gateway TLS configuration failed: {error}")))?;
    Ok(Arc::new(config))
}

enum GatewayStream {
    Plain(TcpStream),
    Tls(StreamOwned<ServerConnection, TcpStream>),
}

impl GatewayStream {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.set_read_timeout(timeout),
            Self::Tls(stream) => stream.sock.set_read_timeout(timeout),
        }
    }

    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.set_write_timeout(timeout),
            Self::Tls(stream) => stream.sock.set_write_timeout(timeout),
        }
    }
}

impl Drop for GatewayStream {
    fn drop(&mut self) {
        let Self::Tls(stream) = self else {
            return;
        };
        stream.conn.send_close_notify();
        while stream.conn.wants_write() {
            match stream.conn.write_tls(&mut stream.sock) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
        let _ = stream.sock.flush();
    }
}

impl Read for GatewayStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for GatewayStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buffer),
            Self::Tls(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

pub fn start_gateway_server(deps: GatewayServerDeps) -> io::Result<GatewayServer> {
    let tls_config = deps
        .config
        .tls
        .as_ref()
        .map(build_tls_server_config)
        .transpose()?;
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
    let join_limiter = Arc::new(JoinRateLimiter::new(JoinLimitPolicy::default()));
    let deps = Arc::new(deps);
    let thread_deps = Arc::clone(&deps);
    let thread = thread::spawn(move || {
        while !thread_stop.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((stream, peer)) => {
                    // The listener is nonblocking so the accept loop can observe shutdown.
                    // On BSD/macOS accepted descriptors may retain nonblocking behavior, while
                    // rustls' StreamOwned performs a blocking handshake over Read/Write.
                    // Normalize every accepted socket before handing it to either HTTP or TLS.
                    if stream.set_nonblocking(false).is_err() {
                        continue;
                    }
                    let connection_deps = Arc::clone(&thread_deps);
                    let connection_stop = Arc::clone(&thread_stop);
                    let connection_join_limiter = Arc::clone(&join_limiter);
                    let connection_tls = tls_config.clone();
                    thread::spawn(move || {
                        let stream = match connection_tls {
                            Some(config) => match ServerConnection::new(config) {
                                Ok(connection) => GatewayStream::Tls(StreamOwned::new(connection, stream)),
                                Err(_) => return,
                            },
                            None => GatewayStream::Plain(stream),
                        };
                        let _ = handle_connection(
                            stream, &connection_deps, &connection_stop,
                            peer.ip(), &connection_join_limiter,
                        );
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

fn read_request(stream: &mut GatewayStream) -> io::Result<HttpRequest> {
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

fn check_device_timestamp_skew(timestamp_str: &str) -> Result<(), &'static str> {
    let ts = timestamp_str.parse::<u64>().map_err(|_| "invalid timestamp format")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let diff = if ts > now { ts - now } else { now - ts };
    if diff > 120 {
        return Err("TIMESTAMP_SKEW: clock skew exceeds +/-120s");
    }
    Ok(())
}

static NONCE_CACHE: std::sync::LazyLock<Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

fn check_and_record_nonce(nonce: &str) -> Result<(), &'static str> {
    if nonce.is_empty() {
        return Err("invalid nonce");
    }
    let mut lock = NONCE_CACHE.lock().unwrap();
    if lock.contains(nonce) {
        return Err("NONCE_REPLAY: nonce has already been used");
    }
    lock.insert(nonce.to_string());
    if lock.len() > 10000 {
        lock.clear();
        lock.insert(nonce.to_string());
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum NativeGatewayAccess {
    Read,
    Write,
}

impl NativeGatewayAccess {
    fn for_method(method: &str) -> Option<(Self, &'static str)> {
        match method {
            "getAgentMemories" => Some((Self::Read, "Memory")),
            "deleteAgentMemory" | "clearAgentMemories" => Some((Self::Write, "Memory")),
            "connectChannel" | "disconnectChannel" => Some((Self::Write, "Channel credential")),
            _ => None,
        }
    }

    fn satisfied_by(self, scope: &str) -> bool {
        match scope {
            "operator.admin" | "operator.write" => true,
            "operator.read" => matches!(self, Self::Read),
            _ => false,
        }
    }
}

fn is_mutating_openclaw_command(method: &str) -> bool {
    matches!(
        method,
        "device.pair.approve"
            | "device.pair.reject"
            | "device.pair.revoke"
            | "device.pair.rotateToken"
            | "device.token.revoke"
            | "device.pair.setupCode"
            | "chat.send"
            | "chat.abort"
            | "sessions.create"
            | "sessions.patch"
            | "sessions.providerReview.continue"
            | "sessions.delete"
            | "cron.trigger"
            | "config.set"
            | "uninstallMcpPlugin"
            | "refreshMcp"
            | "syncPluginSkills"
    )
}

fn is_operator_only_openclaw_command(method: &str) -> bool {
    matches!(
        method,
        "device.pair.approve"
            | "device.pair.reject"
            | "device.pair.list"
            | "device.pair.revoke"
            | "device.pair.rotateToken"
            | "device.token.revoke"
            | "device.pair.setupCode"
            | "chat.send"
            | "chat.abort"
            | "sessions.list"
            | "sessions.get"
            | "sessions.create"
            | "sessions.patch"
            | "sessions.providerReview.continue"
            | "sessions.delete"
            | "cron.list"
            | "cron.trigger"
            | "config.get"
            | "config.set"
            | "system.info"
            | "node.status"
            | "node.list"
            | "uninstallMcpPlugin"
            | "refreshMcp"
            | "syncPluginSkills"
            | "getPluginSyncStatus"
    )
}

fn bearer_token(request: &HttpRequest) -> Option<&str> {
    request
        .headers
        .get("authorization")
        .and_then(|value| value.strip_prefix(&format!("{GATEWAY_AUTH_SCHEME} ")))
        .filter(|token| !token.is_empty())
}

fn is_authorized_token(provided: &str, expected: &str) -> bool {
    let provided = provided.as_bytes();
    let expected = expected.as_bytes();
    provided.len() == expected.len() && bool::from(provided.ct_eq(expected))
}

fn write_response(
    stream: &mut GatewayStream,
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
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
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

fn respond_json(stream: &mut GatewayStream, status: u16, value: Value) -> io::Result<()> {
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

fn client_accepts_gzip(request: &HttpRequest) -> bool {
    request
        .headers
        .get("accept-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("gzip"))
}

fn respond_command_json(
    stream: &mut GatewayStream,
    status: u16,
    value: Value,
    request: &HttpRequest,
) -> io::Result<()> {
    let encoded = serde_json::to_vec(&value)
        .map_err(|error| io::Error::other(format!("gateway response serialization failed: {error}")))?;
    if encoded.len() < GZIP_MIN_BYTES || !client_accepts_gzip(request) {
        return write_response(
            stream,
            status,
            "application/json",
            &encoded,
            &[(GATEWAY_MINT_DEDUPE_HEADER, "1")],
        );
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&encoded)?;
    let zipped = encoder
        .finish()
        .map_err(|error| io::Error::other(format!("gateway gzip finish failed: {error}")))?;
    write_response(
        stream,
        status,
        "application/json",
        &zipped,
        &[
            ("Content-Encoding", "gzip"),
            ("Vary", "Accept-Encoding"),
            (GATEWAY_MINT_DEDUPE_HEADER, "1"),
        ],
    )
}

fn respond_error(stream: &mut GatewayStream, status: u16, message: impl Into<String>) -> io::Result<()> {
    let encoded = serde_json::to_vec(&json!({ "error": message.into() }))
        .map_err(|error| io::Error::other(format!("gateway error serialization failed: {error}")))?;
    write_response(stream, status, "application/json", &encoded, &[])
}

fn respond_pairing_error(
    stream: &mut GatewayStream,
    status: u16,
    message: impl Into<String>,
) -> io::Result<()> {
    let encoded = serde_json::to_vec(&json!({ "error": message.into() }))
        .map_err(|error| io::Error::other(format!("gateway error serialization failed: {error}")))?;
    write_response(
        stream,
        status,
        "application/json",
        &encoded,
        &[("Cache-Control", "no-store")],
    )
}

fn serve_pairing_join(
    stream: &mut GatewayStream,
    deps: &GatewayServerDeps,
    stop: &AtomicBool,
    limiter: &JoinRateLimiter,
    peer: IpAddr,
    request: &HttpRequest,
    shortcode: &str,
    query: Option<&str>,
) -> io::Result<()> {
    let outcome = limiter.attempt(peer, stop, || {
        if request.method != "GET" || query.is_some() || !is_device_pairing_join_code(shortcode) {
            return Ok(None);
        }
        deps.api.redeem_device_pairing_join_code(shortcode)
    });
    match outcome {
        Err(JoinRejection::Limited(delay)) => {
            let retry = join_retry_seconds(delay).to_string();
            write_response(
                stream, 429, "application/json", br#"{"error":"rate_limited"}"#,
                &[("Cache-Control", "no-store"), ("Retry-After", retry.as_str())],
            )
        }
        Err(JoinRejection::Unavailable) => respond_pairing_error(stream, 503, "unavailable"),
        Ok(Err(error)) => respond_pairing_error(stream, error.status(), error.to_string()),
        Ok(Ok(None)) => respond_pairing_error(stream, 404, "not_found"),
        Ok(Ok(Some(payload))) => write_response(
            stream, 200, "application/json",
            &serde_json::to_vec(&payload).map_err(|error| {
                io::Error::other(format!("gateway response serialization failed: {error}"))
            })?,
            &[("Cache-Control", "no-store")],
        ),
    }
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

fn valid_traceparent(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(version) = parts.next() else { return false };
    let Some(trace_id) = parts.next() else { return false };
    let Some(parent_id) = parts.next() else { return false };
    let Some(flags) = parts.next() else { return false };
    if parts.next().is_some()
        || version.len() != 2
        || trace_id.len() != 32
        || parent_id.len() != 16
        || flags.len() != 2
        || ![version, trace_id, parent_id, flags]
            .into_iter()
            .all(|part| part.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return false;
    }
    !trace_id.bytes().all(|byte| byte == b'0') && !parent_id.bytes().all(|byte| byte == b'0')
}

fn gateway_command_context(
    request: &HttpRequest,
    authenticated: bool,
    dispatch_started: Instant,
) -> GatewayCommandContext {
    let request_id = request
        .headers
        .get(GATEWAY_REQUEST_ID_HEADER)
        .filter(|value| !value.is_empty())
        .cloned();
    let traceparent = request
        .headers
        .get(GATEWAY_TRACEPARENT_HEADER)
        .filter(|value| valid_traceparent(value))
        .cloned();
    let (trace_id, span_id) = traceparent
        .as_deref()
        .and_then(|value| {
            let mut parts = value.split('-');
            let _version = parts.next()?;
            let trace_id = parts.next()?.to_string();
            let span_id = parts.next()?.to_string();
            Some((trace_id, span_id))
        })
        .map(|(trace_id, span_id)| (Some(trace_id), Some(span_id)))
        .unwrap_or((None, None));
    GatewayCommandContext {
        request_id,
        traceparent,
        trace_id,
        span_id,
        authenticated,
        dispatch_started,
    }
}

fn command_report(
    request: &HttpRequest,
    method: &str,
    started: Instant,
    status: u16,
    error: Option<String>,
    classification: Option<GatewayCommandErrorClassification>,
) -> GatewayCommandReport {
    let (reason, error_class, errno) = classification
        .map(|value| {
            (
                Some(value.reason.to_string()),
                Some(value.error_class),
                value.errno,
            )
        })
        .unwrap_or((None, None, None));
    GatewayCommandReport {
        method: method.to_string(),
        duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        request_id: request
            .headers
            .get(GATEWAY_REQUEST_ID_HEADER)
            .filter(|value| !value.is_empty())
            .cloned(),
        traceparent: request
            .headers
            .get(GATEWAY_TRACEPARENT_HEADER)
            .filter(|value| valid_traceparent(value))
            .cloned(),
        status,
        error,
        reason,
        error_class,
        errno,
    }
}

fn sse_gzip_enabled(request: &HttpRequest) -> bool {
    std::env::var(DISABLE_SSE_GZIP_ENV).ok().as_deref() != Some("1")
        && client_accepts_gzip(request)
}

fn write_sse_headers(stream: &mut GatewayStream, gzip: bool) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache, no-transform\r\nConnection: keep-alive\r\n"
    )?;
    if gzip {
        write!(
            stream,
            "Content-Encoding: gzip\r\nVary: Accept-Encoding\r\n"
        )?;
    }
    write!(stream, "\r\n")?;
    stream.flush()
}

#[derive(Clone, Copy)]
struct PairedRequestAdmission<'a> {
    api: &'a dyn GatewayApi,
    authorization: &'a DeviceTokenAuthorization,
}

impl PairedRequestAdmission<'_> {
    fn check(self) -> Result<(), GatewayCommandError> {
        self.api.validate_paired_device_authorization(self.authorization)
    }
}

fn check_paired_admission(
    admission: Option<PairedRequestAdmission<'_>>,
) -> Result<(), GatewayCommandError> {
    admission.map_or(Ok(()), PairedRequestAdmission::check)
}

fn serve_event_body<W: Write>(
    sink: &mut W,
    receiver: Receiver<Value>,
    stop: &AtomicBool,
    channels: Option<Vec<String>>,
    slim_avatars: bool,
    allow_internal_mcp_control: bool,
    admission: Option<PairedRequestAdmission<'_>>,
) -> io::Result<()> {
    if stop.load(Ordering::Acquire) || check_paired_admission(admission).is_err() {
        return Ok(());
    }
    sink.write_all(b"retry: 1000\n\n")?;
    sink.flush()?;
    let heartbeat = Duration::from_millis(SSE_HEARTBEAT_MS);
    while !stop.load(Ordering::Acquire) {
        let received = receiver.recv_timeout(heartbeat);
        // Admission, not write completion, is the frame boundary. Every wake
        // checks before filtering, so an excluded-event flood cannot retain access.
        if stop.load(Ordering::Acquire) || check_paired_admission(admission).is_err() {
            break;
        }
        match received {
            Ok(event) => {
                // Paired devices may watch ordinary Gateway events, but both
                // MCP relay request channels belong to the native control plane.
                // Filter before channel selection, slimming and serialization.
                if !allow_internal_mcp_control
                    && matches!(
                        event_channel(&event),
                        Some(HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL)
                            | Some(RUNNER_TOOL_REQUEST_EVENT_CHANNEL)
                    )
                {
                    continue;
                }
                if channels
                    .as_ref()
                    .is_some_and(|allowed| event_channel(&event).is_some_and(|channel| !allowed.iter().any(|value| value == channel)))
                {
                    continue;
                }
                let event = if slim_avatars { slim_event(event) } else { event };
                let encoded = serde_json::to_string(&event)
                    .map_err(|error| io::Error::other(format!("serialize gateway event: {error}")))?;
                write!(sink, "data: {encoded}\n\n")?;
                sink.flush()?;
            }
            Err(RecvTimeoutError::Timeout) => {
                sink.write_all(b":ping\n\n")?;
                sink.flush()?;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn serve_events(
    stream: &mut GatewayStream,
    deps: &GatewayServerDeps,
    stop: &AtomicBool,
    channels: Option<Vec<String>>,
    slim_avatars: bool,
    gzip: bool,
    allow_internal_mcp_control: bool,
    admission: Option<PairedRequestAdmission<'_>>,
) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    if let Err(error) = check_paired_admission(admission) {
        return respond_error(stream, error.status(), error.to_string());
    }
    let receiver = deps.events.subscribe();
    if let Err(error) = check_paired_admission(admission) {
        return respond_error(stream, error.status(), error.to_string());
    }
    write_sse_headers(stream, gzip)?;
    if gzip {
        let mut encoder = GzEncoder::new(stream, Compression::default());
        let result = serve_event_body(&mut encoder, receiver, stop, channels, slim_avatars, allow_internal_mcp_control, admission);
        let _ = encoder.try_finish();
        result
    } else {
        serve_event_body(stream, receiver, stop, channels, slim_avatars, allow_internal_mcp_control, admission)
    }
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
    stream: &mut GatewayStream,
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

fn serve_bridge_body<W: Write>(
    sink: &mut W,
    subscription: GatewayBridgeSubscription,
    stop: &AtomicBool,
    admission: Option<PairedRequestAdmission<'_>>,
) -> io::Result<()> {
    if stop.load(Ordering::Acquire) || check_paired_admission(admission).is_err() {
        return Ok(());
    }
    sink.write_all(b"retry: 1000\n\n")?;
    sink.flush()?;
    let heartbeat = Duration::from_millis(SSE_HEARTBEAT_MS);
    while !stop.load(Ordering::Acquire) {
        let received = subscription.recv_timeout(heartbeat);
        if stop.load(Ordering::Acquire) || check_paired_admission(admission).is_err() {
            break;
        }
        match received {
            Ok(frame) => {
                let encoded = serde_json::to_string(&frame)
                    .map_err(|error| io::Error::other(format!("serialize gateway bridge frame: {error}")))?;
                write!(sink, "data: {encoded}\n\n")?;
                sink.flush()?;
            }
            Err(RecvTimeoutError::Timeout) => {
                sink.write_all(b":ping\n\n")?;
                sink.flush()?;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn serve_bridge_requests(
    stream: &mut GatewayStream,
    bridge: &GatewayBridgeHub,
    stop: &AtomicBool,
    gzip: bool,
    admission: Option<PairedRequestAdmission<'_>>,
) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    if let Err(error) = check_paired_admission(admission) {
        return respond_error(stream, error.status(), error.to_string());
    }
    let receiver = bridge.subscribe_requests();
    if let Err(error) = check_paired_admission(admission) {
        return respond_error(stream, error.status(), error.to_string());
    }
    write_sse_headers(stream, gzip)?;
    if gzip {
        let mut encoder = GzEncoder::new(stream, Compression::default());
        let result = serve_bridge_body(&mut encoder, receiver, stop, admission);
        let _ = encoder.try_finish();
        result
    } else {
        serve_bridge_body(stream, receiver, stop, admission)
    }
}

fn submit_bridge_responses(
    stream: &mut GatewayStream,
    bridge: &GatewayBridgeHub,
    body: &[u8],
    admission: Option<PairedRequestAdmission<'_>>,
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
    if let Err(error) = check_paired_admission(admission) {
        return respond_error(stream, error.status(), error.to_string());
    }
    bridge.submit_responses(batch);
    respond_json(stream, 200, json!({ "ok": true }))
}

fn handle_connection(
    mut stream: GatewayStream,
    deps: &GatewayServerDeps,
    stop: &AtomicBool,
    peer: IpAddr,
    join_limiter: &JoinRateLimiter,
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
    // A join code is this public namespace's credential. Throttle and validate
    // before optional bearer/nonce/revision processing can touch another owner.
    if let Some(shortcode) = parse_device_pairing_join_request_path(path) {
        return serve_pairing_join(
            &mut stream, deps, stop, join_limiter, peer, &request, shortcode, query,
        );
    }
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
    let method = if command {
        path.strip_prefix(&format!("{GATEWAY_API_PREFIX}/")).unwrap_or_default()
    } else {
        ""
    };

    if command && !is_gateway_command(method) {
        return respond_error(&mut stream, 404, format!("unknown gateway method: {method}"));
    }

    if (local_requests || local_responses) && deps.config.auth_token.is_none() {
        return respond_error(&mut stream, 401, "local-exec requires gateway authentication");
    }
    if (webauthn_requests || webauthn_responses) && deps.config.auth_token.is_none() {
        return respond_error(&mut stream, 401, "webauthn requires gateway authentication");
    }

    let allows_unauthenticated = command && matches!(method, "device.pair.request" | "device.pair.status");
    // Credential absence is a native mode only on the actual loopback listener.
    // An explicit malformed, unknown or retired bearer never becomes absence.
    let mut authenticated = deps.config.auth_token.is_none()
        && !request.headers.contains_key("authorization")
        && is_loopback_host(&deps.config.host);
    let mut device_authorization = None;

    if let Some(token) = bearer_token(&request) {
        if deps
            .config
            .auth_token
            .as_deref()
            .is_some_and(|expected| is_authorized_token(token, expected))
        {
            authenticated = true;
        } else {
            match deps.api.authenticate_device_token(token) {
                Ok(Some(authorization)) => {
                    authenticated = true;
                    device_authorization = Some(authorization);
                }
                Ok(None) => {}
                Err(error) => {
                    return respond_error(&mut stream, error.status(), error.to_string());
                }
            }
        }
    }

    if let (Some(ts), Some(nonce)) = (
        request.headers.get("x-sand-device-timestamp"),
        request.headers.get("x-sand-device-nonce"),
    ) {
        if let Err(msg) = check_device_timestamp_skew(ts) {
            return respond_error(&mut stream, 401, msg);
        }
        // Timestamp/nonce are replay fences for an already authenticated
        // device request. They are not a device signature or token, so they
        // must never promote an unauthenticated request to authenticated.
        if authenticated {
            if let Err(msg) = check_and_record_nonce(nonce) {
                return respond_error(&mut stream, 401, msg);
            }
        }
    }

    if let Some(rev) = request.headers.get("x-sand-pairing-revision") {
        if rev == "stale" || rev.starts_with("stale-") {
            return respond_error(&mut stream, 401, "PAIRING_CHANGED: pairing revision is stale");
        }
    }

    if !authenticated && !allows_unauthenticated {
        return respond_error(&mut stream, 401, "unauthorized");
    }

    let requested_role = request.headers.get("x-sand-role").map(String::as_str);
    if method == "device.token.revoke"
        && device_authorization.is_none()
        && requested_role.is_some_and(|role| role != "operator")
    {
        return respond_error(
            &mut stream,
            403,
            "ROLE_MISMATCH: token revocation requires operator role",
        );
    }
    if let Some((access, family)) = NativeGatewayAccess::for_method(method) {
        if requested_role.is_some_and(|role| role != "operator")
            || device_authorization
                .as_ref()
                .is_some_and(|authorization| authorization.role != "operator")
        {
            return respond_error(
                &mut stream,
                403,
                format!("ROLE_MISMATCH: {family} methods require operator role"),
            );
        }
        let grant_allows = device_authorization.as_ref().is_none_or(|authorization| {
            authorization.scopes.iter().any(|scope| access.satisfied_by(scope))
        });
        // The singular scope header is only a ceiling, including for native
        // callers; it never supplies a missing paired-device permission.
        let ceiling_allows = request.headers.get("x-sand-scope")
            .is_none_or(|scope| access.satisfied_by(scope));
        if !grant_allows || !ceiling_allows {
            return respond_error(
                &mut stream,
                403,
                format!("INSUFFICIENT_SCOPE: {family} access is not granted for method: {method}"),
            );
        }
    }
    // Reuse Box's native authority for both MCP receipt routes and their SSE
    // requests. Common authentication admits the configured secret or explicit
    // credential-absent native loopback mode, never an unknown bearer.
    // Recognized device credentials never gain this authority through headers.
    let native_mcp_control = device_authorization.is_none()
        && requested_role.is_none_or(|role| role == "operator")
        && request
            .headers
            .get("x-sand-scope")
            .is_none_or(|scope| scope == "operator.write")
        && (deps.config.auth_token.is_some() || is_loopback_host(&deps.config.host));
    if method == MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD && !native_mcp_control {
        return respond_error(
            &mut stream,
            403,
            "MCP_LIFECYCLE_NATIVE_ONLY: Host MCP lifecycle receipts require the native control plane",
        );
    }
    if method == RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD && !native_mcp_control {
        return respond_error(
            &mut stream,
            403,
            "RUNNER_TOOL_NATIVE_ONLY: Routed MCP tool receipts require the native control plane",
        );
    }
    if FABUSHI_NATIVE_BOX_MCP_GATEWAY_METHODS.contains(&method) && !native_mcp_control {
        return respond_error(
            &mut stream,
            403,
            "MCP_BOX_NATIVE_ONLY: Box MCP requires the native control-plane gateway",
        );
    }
    // The desktop preview leg carries a native-selected cache directory.
    // Reuse the same existing control-plane authority as native Box routes;
    // a paired operator token must not turn this into an arbitrary-path RPC.
    if FABUSHI_NATIVE_ATTACHMENTS_GATEWAY_METHODS.contains(&method) && !native_mcp_control {
        return respond_error(
            &mut stream,
            403,
            "ATTACHMENTS_NATIVE_ONLY: Link preview requires the native control-plane gateway",
        );
    }
    if let Some(authorization) = device_authorization.as_ref() {
        if requested_role.is_some_and(|role| role != authorization.role) {
            return respond_error(
                &mut stream,
                403,
                "ROLE_MISMATCH: requested role differs from device token",
            );
        }
        if authorization.role == "operator"
            && is_mutating_openclaw_command(method)
            // The bound owner below applies the current role/approval and
            // operator implications atomically, including admin-only grants.
            && !matches!(method, "device.pair.rotateToken" | "device.pair.revoke" | "device.token.revoke")
            && !authorization
                .scopes
                .iter()
                .any(|scope| scope == "operator.write")
        {
            return respond_error(
                &mut stream,
                403,
                format!("INSUFFICIENT_SCOPE: device token cannot invoke mutating method: {method}"),
            );
        }
    }
    if (requested_role == Some("node")
        || device_authorization
            .as_ref()
            .is_some_and(|authorization| authorization.role == "node"))
        && is_operator_only_openclaw_command(method)
    {
        return respond_error(
            &mut stream,
            403,
            format!("ROLE_MISMATCH: node cannot invoke operator method: {method}"),
        );
    }

    if let Some(scope) = request.headers.get("x-sand-scope") {
        if scope == "operator.read" && is_mutating_openclaw_command(method) {
            return respond_error(
                &mut stream,
                403,
                format!("INSUFFICIENT_SCOPE: read-only scope cannot invoke mutating method: {method}"),
            );
        }
    }
    let paired_admission = device_authorization.as_ref().map(|authorization| {
        PairedRequestAdmission { api: deps.api.as_ref(), authorization }
    });
    if local_requests {
        return match deps.local_exec.as_ref() {
            Some(bridge) => serve_bridge_requests(&mut stream, bridge, stop, sse_gzip_enabled(&request), paired_admission),
            None => respond_error(&mut stream, 404, "local-exec channel not enabled"),
        };
    }
    if local_responses {
        return match deps.local_exec.as_ref() {
            Some(bridge) => submit_bridge_responses(&mut stream, bridge, &request.body, paired_admission),
            None => respond_error(&mut stream, 404, "local-exec channel not enabled"),
        };
    }
    if webauthn_requests {
        return match deps.webauthn.as_ref() {
            Some(bridge) => serve_bridge_requests(&mut stream, bridge, stop, sse_gzip_enabled(&request), paired_admission),
            None => respond_error(&mut stream, 404, "webauthn channel not enabled"),
        };
    }
    if webauthn_responses {
        return match deps.webauthn.as_ref() {
            Some(bridge) => submit_bridge_responses(&mut stream, bridge, &request.body, paired_admission),
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
        return serve_events(
            &mut stream,
            deps,
            stop,
            parse_channels(query),
            slim_avatars,
            sse_gzip_enabled(&request),
            native_mcp_control,
            paired_admission,
        );
    }
    if prepare {
        return match deps.api.prepare_for_upgrade() {
            Ok(value) => respond_json(&mut stream, 200, value),
            Err(error) => respond_error(&mut stream, error.status(), error.to_string()),
        };
    }
    let args = match parse_command_args(&request.body) {
        Ok(value) => value,
        Err(error) => {
            return respond_error(&mut stream, 400, format!("invalid JSON body: {error}"));
        }
    };
    if let Some(authorization) = device_authorization.as_ref() {
        if let Err(error) = deps
            .api
            .authorize_device_token_request(authorization, method, &args)
        {
            return respond_error(&mut stream, error.status(), error.to_string());
        }
    }
    let started = Instant::now();
    let command_context = gateway_command_context(&request, authenticated, started);
    let result = match device_authorization.as_ref() {
        authorization if method == "device.token.revoke" => {
            deps.api.revoke_device_token(
                args,
                authorization,
                request.headers.get("x-sand-scope").map(String::as_str),
                &command_context,
            )
        }
        Some(authorization) if method == "device.pair.rotateToken" => {
            deps.api.rotate_paired_device_token(
                args,
                authorization,
                request.headers.get("x-sand-scope").map(String::as_str),
                &command_context,
            )
        }
        Some(authorization) if method == "device.pair.revoke" => {
            deps.api.revoke_paired_device(
                args,
                authorization,
                request.headers.get("x-sand-scope").map(String::as_str),
                &command_context,
            )
        }
        _ => deps.api.call_with_context(method, args, &command_context),
    };
    match result {
        Ok(value) => {
            deps.api
                .on_command_complete(command_report(&request, method, started, 200, None, None));
            let slim = request
                .headers
                .get(GATEWAY_SLIM_AVATARS_HEADER)
                .is_some_and(|value| value == "1");
            respond_command_json(
                &mut stream,
                200,
                if slim { slim_command_result(method, value) } else { value },
                &request,
            )
        }
        Err(error) => {
            let status = error.status();
            if status >= 500 {
                let classification = classify_gateway_command_error(&error);
                deps.api.on_command_error(command_report(
                    &request,
                    method,
                    started,
                    status,
                    Some(error.to_string()),
                    Some(classification),
                ));
            }
            respond_error(&mut stream, status, error.to_string())
        }
    }
}

#[cfg(test)]
mod join_rate_limit_tests {
    use super::{JoinLimitPolicy, JoinPeerState, JoinRateLimiter, JoinRejection, join_peer_ip, join_retry_seconds};
    use crate::gateway_command_error::GatewayCommandError;
    use serde_json::{Value, json};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    fn peer(id: u8) -> IpAddr { IpAddr::V4(Ipv4Addr::new(198, 51, 100, id)) }

    fn seed(limiter: &JoinRateLimiter, ip: IpAddr, now: Instant, count: usize) {
        for _ in 0..count {
            assert!(limiter.attempt_with_clock(ip, &AtomicBool::new(false), || now, || Ok(None))
                .expect("admitted miss").expect("controlled miss").is_none());
        }
    }

    fn failures(limiter: &JoinRateLimiter, ip: IpAddr) -> Vec<Instant> {
        let peers = limiter.peers.lock().expect("inspect map");
        let state = peers.get(&join_peer_ip(ip)).expect("tracked peer").peer.state.lock().expect("inspect peer");
        state.failures.iter().copied().collect()
    }

    fn reserve_idle(limiter: &JoinRateLimiter, ip: IpAddr, now: Instant) {
        let mut reservation = limiter.reserve(ip, now).expect("reserve idle peer");
        reservation.wait_turn().expect("idle turn");
    }

    fn wait_for_reservations(limiter: &JoinRateLimiter, ip: IpAddr, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Ok(peers) = limiter.peers.try_lock() {
                if peers.get(&join_peer_ip(ip)).is_some_and(|entry| Arc::strong_count(&entry.peer) >= count) {
                    return;
                }
            }
            assert!(Instant::now() < deadline, "no causal queued peer-reservation witness");
            thread::yield_now();
        }
    }

    #[test]
    fn join_limit_defaults_window_lockout_and_retry_boundaries() {
        let policy = JoinLimitPolicy::default();
        assert_eq!((policy.failures, policy.entries), (10, 10_000));
        assert_eq!((policy.window, policy.lockout), (Duration::from_secs(60), Duration::from_secs(300)));
        let base = Instant::now();
        let mut state = JoinPeerState::default();
        for _ in 0..9 { state.fail(base, policy); }
        assert!(state.check(base + Duration::from_millis(59_999), policy).is_ok());
        assert_eq!(state.failures.len(), 9);
        assert!(state.check(base + policy.window, policy).is_ok());
        assert!(state.failures.is_empty(), "exact cutoff expires old misses");
        let threshold = base + policy.window;
        for _ in 0..10 { state.fail(threshold, policy); }
        let until = threshold + policy.lockout;
        assert_eq!(state.locked_until, Some(until));
        assert_eq!(state.check(threshold + Duration::from_millis(1), policy),
            Err(JoinRejection::Limited(Duration::from_millis(299_999))));
        assert_eq!(join_retry_seconds(Duration::from_millis(299_999)), 300);
        assert_eq!(join_retry_seconds(Duration::from_nanos(1)), 1);
        assert_eq!(join_retry_seconds(Duration::from_secs(1)), 1);
        for _ in 0..20 { state.fail(threshold + Duration::from_secs(1), policy); }
        assert_eq!(state.failures.len(), 10, "history never grows past threshold");
        assert_eq!(state.locked_until, Some(until), "denials cannot extend lockout");
        assert!(state.check(until, policy).is_ok());
        assert!(state.failures.is_empty());
        state.fail(until, policy);
        assert_eq!(state.failures.len(), 1, "served lockout starts a fresh history");
        assert!(state.locked_until.is_none());
    }

    #[test]
    fn join_limit_success_isolation_and_storage_error_accounting() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy::default());
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        seed(&limiter, peer(1), now, 9);
        seed(&limiter, peer(2), now, 9);
        assert_eq!(limiter.attempt_with_clock(peer(1), &stop, || now, || Ok(Some(Value::Null)))
            .expect("admitted legacy null").expect("actual success"), Some(Value::Null));
        assert!(failures(&limiter, peer(1)).is_empty());
        assert_eq!(failures(&limiter, peer(2)).len(), 9);
        let error = limiter.attempt_with_clock(peer(2), &stop, || now, || {
            Err(GatewayCommandError::Internal("storage refusal".into()))
        }).expect("error attempt admitted").expect_err("owner error remains an error");
        assert_eq!(error.to_string(), "storage refusal");
        assert_eq!(failures(&limiter, peer(2)).len(), 9, "error neither spends nor resets misses");
        seed(&limiter, peer(2), now, 1);
        assert!(matches!(limiter.attempt_with_clock(peer(2), &stop, || now, || panic!("blocked owner")),
            Err(JoinRejection::Limited(_))));
    }

    #[test]
    fn join_limit_mapped_peers_share_strict_loopback_budget() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy::default());
        let now = Instant::now();
        let ipv4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let mapped = IpAddr::V6(Ipv4Addr::LOCALHOST.to_ipv6_mapped());
        assert_eq!(join_peer_ip(mapped), ipv4);
        seed(&limiter, ipv4, now, 10);
        assert!(matches!(limiter.attempt_with_clock(mapped, &AtomicBool::new(false), || now, || panic!("mapped bypass")),
            Err(JoinRejection::Limited(_))));
        let ipv6 = IpAddr::V6(Ipv6Addr::LOCALHOST);
        seed(&limiter, ipv6, now, 10);
        assert!(matches!(limiter.attempt_with_clock(ipv6, &AtomicBool::new(false), || now, || panic!("loopback bypass")),
            Err(JoinRejection::Limited(_))));
        assert_eq!(limiter.peers.lock().unwrap().len(), 2);
    }

    #[test]
    fn join_limit_capacity_reclaims_expired_before_idle_and_keeps_lockouts() {
        let now = Instant::now();
        let limiter = JoinRateLimiter::new(JoinLimitPolicy { entries: 2, ..JoinLimitPolicy::default() });
        seed(&limiter, peer(1), now + Duration::from_secs(59), 1);
        seed(&limiter, peer(2), now, 1);
        reserve_idle(&limiter, peer(3), now + Duration::from_secs(60));
        {
            let peers = limiter.peers.lock().unwrap();
            assert!(peers.contains_key(&peer(1)) && peers.contains_key(&peer(3)));
            assert!(!peers.contains_key(&peer(2)));
        }
        let full = JoinRateLimiter::new(JoinLimitPolicy { entries: 1, ..JoinLimitPolicy::default() });
        seed(&full, peer(1), now, 10);
        assert!(matches!(full.reserve(peer(2), now + Duration::from_secs(1)), Err(JoinRejection::Limited(_))));
        assert_eq!(full.peers.lock().unwrap().len(), 1);
        assert!(full.peers.lock().unwrap().contains_key(&peer(1)), "live lockout is never evicted");
        reserve_idle(&full, peer(2), now + Duration::from_secs(300));
        assert!(!full.peers.lock().unwrap().contains_key(&peer(1)));
        seed(&full, peer(2), now + Duration::from_secs(301), 1);
        reserve_idle(&full, peer(3), now + Duration::from_secs(302));
        assert_eq!(full.peers.lock().unwrap().len(), 1);
        assert!(full.peers.lock().unwrap().contains_key(&peer(3)));
    }

    #[test]
    fn join_limit_live_and_queued_reservations_cannot_be_evicted() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy { entries: 1, ..JoinLimitPolicy::default() });
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        let mut reservation = limiter.reserve(peer(1), now).expect("pin original identity");
        reservation.wait_turn().unwrap();
        let entry = Arc::clone(&reservation.peer);
        let held = entry.state.lock().unwrap();
        thread::scope(|scope| {
            let queued = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || Ok(None)));
            wait_for_reservations(&limiter, peer(1), 4);
            assert!(matches!(limiter.reserve(peer(2), now), Err(JoinRejection::Limited(_))));
            assert!(Arc::ptr_eq(&limiter.peers.lock().unwrap().get(&peer(1)).unwrap().peer, &entry));
            drop(held);
            drop(reservation);
            assert!(queued.join().unwrap().expect("queued admission").expect("miss").is_none());
        });
        assert_eq!(failures(&limiter, peer(1)).len(), 1);
        assert!(matches!(limiter.reserve(peer(2), now), Err(JoinRejection::Limited(_))), "live retained guard still pins identity");
        drop(entry);
        reserve_idle(&limiter, peer(2), now);
        assert_eq!(limiter.peers.lock().unwrap().len(), 1);
    }

    #[test]
    fn join_limit_serializes_whole_attempt_without_blocking_other_peers() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy { failures: 1, ..JoinLimitPolicy::default() });
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        let queued_calls = AtomicUsize::new(0);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        thread::scope(|scope| {
            let limiter_ref = &limiter;
            let stop_ref = &stop;
            let first = scope.spawn(move || limiter_ref.attempt_with_clock(peer(1), stop_ref, || now, || {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).expect("release owned first attempt");
                Ok(None)
            }));
            entered_rx.recv_timeout(Duration::from_secs(3)).expect("first owner entered");
            let queued = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || {
                queued_calls.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            }));
            wait_for_reservations(&limiter, peer(1), 3);
            let (other_tx, other_rx) = mpsc::channel();
            let limiter_ref = &limiter;
            let stop_ref = &stop;
            scope.spawn(move || other_tx.send(limiter_ref.attempt_with_clock(peer(2), stop_ref, || now, || Ok(Some(json!(7))))).unwrap());
            assert_eq!(other_rx.recv_timeout(Duration::from_secs(3)).expect("independent peer progresses")
                .expect("other admission").expect("other result"), Some(json!(7)));
            release_tx.send(()).unwrap();
            assert!(first.join().unwrap().expect("first admission").expect("first miss").is_none());
            assert!(matches!(queued.join().unwrap(), Err(JoinRejection::Limited(_))));
        });
        assert_eq!(queued_calls.load(Ordering::SeqCst), 0);
        assert_eq!(failures(&limiter, peer(1)).len(), 1);
    }

    #[test]
    fn join_limit_reserved_turns_are_fifo_for_mixed_outcomes() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy { failures: 2, ..JoinLimitPolicy::default() });
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        seed(&limiter, peer(1), now, 1);
        let (order_tx, order_rx) = mpsc::channel();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        thread::scope(|scope| {
            let limiter_ref = &limiter;
            let stop_ref = &stop;
            let first_order = order_tx.clone();
            let first = scope.spawn(move || limiter_ref.attempt_with_clock(peer(1), stop_ref, || now, || {
                first_order.send(0).unwrap();
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).expect("release first reserved owner");
                Ok(Some(Value::Null))
            }));
            entered_rx.recv_timeout(Duration::from_secs(3)).expect("first owner entered");
            let second = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || {
                order_tx.send(1).unwrap(); Ok(None)
            }));
            wait_for_reservations(&limiter, peer(1), 3);
            let third = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || {
                order_tx.send(2).unwrap(); Ok(Some(json!(["legacy"])))
            }));
            wait_for_reservations(&limiter, peer(1), 4);
            let fourth = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || {
                order_tx.send(3).unwrap(); Ok(None)
            }));
            wait_for_reservations(&limiter, peer(1), 5);
            assert_eq!(order_rx.try_iter().collect::<Vec<_>>(), vec![0], "queued owners cannot enter early");
            release_tx.send(()).unwrap();
            assert_eq!(first.join().unwrap().expect("first admission").expect("first success"), Some(Value::Null));
            assert!(second.join().unwrap().expect("second admission").expect("second miss").is_none());
            assert_eq!(third.join().unwrap().expect("third admission").expect("third success"), Some(json!(["legacy"])));
            assert!(fourth.join().unwrap().expect("fourth admission").expect("fourth miss").is_none());
        });
        assert_eq!(order_rx.try_iter().collect::<Vec<_>>(), vec![1, 2, 3], "reservation order is observable");
        assert_eq!(failures(&limiter, peer(1)), vec![now], "mixed FIFO outcomes leave only the final miss");
    }

    #[test]
    fn join_limit_clock_is_sampled_after_queue_and_owner_settlement() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy::default());
        let base = Instant::now();
        let clock = AtomicU64::new(0);
        let stop = AtomicBool::new(false);
        let mut reservation = limiter.reserve(peer(1), base).unwrap();
        reservation.wait_turn().unwrap();
        let entry = Arc::clone(&reservation.peer);
        let mut held = entry.state.lock().unwrap();
        for _ in 0..10 { held.fail(base, limiter.policy); }
        thread::scope(|scope| {
            let queued = scope.spawn(|| limiter.attempt_with_clock(
                peer(1), &stop, || base + Duration::from_secs(clock.load(Ordering::SeqCst)), || Ok(Some(Value::Null)),
            ));
            wait_for_reservations(&limiter, peer(1), 4);
            clock.store(300, Ordering::SeqCst);
            drop(held);
            drop(reservation);
            assert_eq!(queued.join().unwrap().expect("fresh clock after queued turn").expect("success"), Some(Value::Null));
        });
        seed(&limiter, peer(2), base + Duration::from_secs(300), 9);
        clock.store(300, Ordering::SeqCst);
        assert!(limiter.attempt_with_clock(peer(2), &stop,
            || base + Duration::from_secs(clock.load(Ordering::SeqCst)),
            || { clock.store(360, Ordering::SeqCst); Ok(None) },
        ).expect("owner entered").expect("settled miss").is_none());
        assert_eq!(failures(&limiter, peer(2)), vec![base + Duration::from_secs(360)]);
    }

    #[test]
    fn join_limit_stop_refuses_queued_turn_but_settles_entered_result() {
        let limiter = JoinRateLimiter::new(JoinLimitPolicy::default());
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        seed(&limiter, peer(1), now, 9);
        let queued_calls = AtomicUsize::new(0);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        thread::scope(|scope| {
            let limiter_ref = &limiter;
            let stop_ref = &stop;
            let entered = scope.spawn(move || limiter_ref.attempt_with_clock(peer(1), stop_ref, || now, || {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).expect("release entered operation");
                Ok(Some(Value::Null))
            }));
            entered_rx.recv_timeout(Duration::from_secs(3)).expect("operation already entered");
            let queued = scope.spawn(|| limiter.attempt_with_clock(peer(1), &stop, || now, || {
                queued_calls.fetch_add(1, Ordering::SeqCst); Ok(None)
            }));
            wait_for_reservations(&limiter, peer(1), 3);
            stop.store(true, Ordering::Release);
            release_tx.send(()).unwrap();
            assert_eq!(entered.join().unwrap().expect("entered result still settles").expect("actual success"), Some(Value::Null));
            assert!(matches!(queued.join().unwrap(), Err(JoinRejection::Unavailable)));
        });
        assert_eq!(queued_calls.load(Ordering::SeqCst), 0);
        assert!(failures(&limiter, peer(1)).is_empty(), "actual success resets despite later stop");
        assert!(matches!(limiter.attempt(peer(2), &stop, || panic!("stopped admission")), Err(JoinRejection::Unavailable)));
        assert_eq!(limiter.peers.lock().unwrap().len(), 1, "stopped admission does not reserve a new peer");
    }

    #[test]
    fn join_limit_poisoned_admission_state_fails_closed() {
        let stop = AtomicBool::new(false);
        let map_poison = JoinRateLimiter::new(JoinLimitPolicy::default());
        thread::scope(|scope| {
            assert!(scope.spawn(|| {
                let _held = map_poison.peers.lock().unwrap();
                panic!("controlled map poison");
            }).join().is_err());
        });
        assert!(matches!(map_poison.attempt(peer(1), &stop, || panic!("poisoned map bypass")), Err(JoinRejection::Unavailable)));
        let peer_poison = JoinRateLimiter::new(JoinLimitPolicy::default());
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (queued_tx, queued_rx) = mpsc::channel();
        thread::scope(|scope| {
            let limiter_ref = &peer_poison;
            let stop_ref = &stop;
            let entered = scope.spawn(move || limiter_ref.attempt(peer(1), stop_ref, || {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).expect("release panicking owner");
                panic!("controlled owner unwind");
            }));
            entered_rx.recv_timeout(Duration::from_secs(3)).expect("owner entered before poison");
            let limiter_ref = &peer_poison;
            let stop_ref = &stop;
            let queued = scope.spawn(move || queued_tx.send(limiter_ref.attempt(peer(1), stop_ref, || {
                panic!("queued owner entered after unwind")
            })).unwrap());
            wait_for_reservations(&peer_poison, peer(1), 3);
            release_tx.send(()).unwrap();
            assert!(entered.join().is_err(), "actual owner unwind poisons its admission state");
            assert!(matches!(queued_rx.recv_timeout(Duration::from_secs(3)).expect("unwind wakes queued reservation"),
                Err(JoinRejection::Unavailable)));
            queued.join().unwrap();
        });
        assert!(matches!(peer_poison.attempt(peer(1), &stop, || panic!("poisoned peer bypass")), Err(JoinRejection::Unavailable)));
        assert!(peer_poison.attempt(peer(2), &stop, || Ok(None)).expect("independent healthy peer").expect("miss").is_none());

        let turns_poison = JoinRateLimiter::new(JoinLimitPolicy::default());
        reserve_idle(&turns_poison, peer(1), Instant::now());
        let entry = Arc::clone(&turns_poison.peers.lock().unwrap().get(&peer(1)).unwrap().peer);
        thread::scope(|scope| {
            assert!(scope.spawn(move || {
                let _held = entry.turns.lock().unwrap();
                panic!("controlled turn-state poison");
            }).join().is_err());
        });
        assert!(matches!(turns_poison.attempt(peer(1), &stop, || panic!("poisoned turn bypass")), Err(JoinRejection::Unavailable)));
    }

    #[test]
    fn join_limit_abandoned_or_exhausted_turns_fail_closed() {
        let now = Instant::now();
        let stop = AtomicBool::new(false);
        let abandoned = JoinRateLimiter::new(JoinLimitPolicy { entries: 1, ..JoinLimitPolicy::default() });
        let mut first = abandoned.reserve(peer(1), now).unwrap();
        first.wait_turn().unwrap();
        let omitted = abandoned.reserve(peer(1), now).unwrap();
        let (queued_tx, queued_rx) = mpsc::channel();
        thread::scope(|scope| {
            let limiter_ref = &abandoned;
            let stop_ref = &stop;
            let queued = scope.spawn(move || queued_tx.send(limiter_ref.attempt(peer(1), stop_ref, || {
                panic!("abandoned reservation bypass")
            })).unwrap());
            wait_for_reservations(&abandoned, peer(1), 4);
            drop(omitted);
            assert!(matches!(queued_rx.recv_timeout(Duration::from_secs(3)).expect("abandoned slot wakes later turn"),
                Err(JoinRejection::Unavailable)));
            queued.join().unwrap();
            drop(first);
        });
        assert!(matches!(abandoned.attempt(peer(1), &stop, || panic!("invalidated peer bypass")), Err(JoinRejection::Unavailable)));
        assert!(matches!(abandoned.reserve(peer(2), now), Err(JoinRejection::Limited(_))), "invalidated state cannot be recycled into a new budget");

        let exhausted = JoinRateLimiter::new(JoinLimitPolicy { entries: 1, ..JoinLimitPolicy::default() });
        reserve_idle(&exhausted, peer(1), now);
        {
            let mut peers = exhausted.peers.lock().unwrap();
            let entry = peers.get_mut(&peer(1)).unwrap();
            entry.next_ticket = u64::MAX - 1;
            entry.peer.turns.lock().unwrap().serving = u64::MAX - 1;
        }
        assert_eq!(exhausted.attempt_with_clock(peer(1), &stop, || now, || Ok(Some(Value::Null)))
            .expect("last representable reservation").expect("last result"), Some(Value::Null));
        assert!(matches!(exhausted.attempt(peer(1), &stop, || panic!("exhausted ticket bypass")), Err(JoinRejection::Unavailable)));
        assert!(matches!(exhausted.reserve(peer(2), now), Err(JoinRejection::Limited(_))), "exhausted identity cannot be evicted and restarted");
        let peers = exhausted.peers.lock().unwrap();
        let entry = peers.get(&peer(1)).unwrap();
        assert_eq!(entry.next_ticket, u64::MAX);
        assert_eq!(entry.peer.turns.lock().unwrap().serving, u64::MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_gateway_command, serve_event_body, HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL,
        RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
    };
    use serde_json::json;
    use std::sync::{mpsc, atomic::AtomicBool};


    #[test]
    fn paired_device_event_filter_hides_host_mcp_requests_even_without_channel_selection() {
        for allow_internal in [false, true] {
            let (sender, receiver) = mpsc::channel();
            sender.send(json!({
                "channel": HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL,
                "payload": {"requestId": "private-mcp-request", "method": "installHostMcpPlugin"}
            })).unwrap();
            sender.send(json!({
                "channel": RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
                "payload": {"requestId": "private-runner-request", "method": "executeRoutedMcpTool"}
            })).unwrap();
            sender.send(json!({"channel": "public-status", "payload": {"ok": true}})).unwrap();
            drop(sender);
            let mut rendered = Vec::new();
            serve_event_body(
                &mut rendered, receiver, &AtomicBool::new(false),
                None, false, allow_internal, None,
            ).expect("event stream body");
            let body = String::from_utf8(rendered).expect("UTF-8 events");
            assert!(body.contains("public-status"), "{body}");
            assert_eq!(body.contains("private-mcp-request"), allow_internal, "{body}");
            assert_eq!(body.contains("private-runner-request"), allow_internal, "{body}");
            assert_eq!(body.contains(RUNNER_TOOL_REQUEST_EVENT_CHANNEL), allow_internal, "{body}");
            assert_eq!(
                body.contains(HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL),
                allow_internal,
                "{body}"
            );
        }
    }

    #[test]
    fn fbcp_human_commands_are_explicit_gateway_extensions() {
        for method in [
            "listHumanConversations",
            "syncHumanConversations",
            "syncHumanConversation",
            "createHumanConversation",
            "sendHumanMessage",
            "reactHumanMessage",
            "getHumanConversationTranscript",
            "getHumanConversationTranscriptTail",
            "searchHumanMessages",
            "createCallSession",
            "getCallSession",
            "listCallSessions",
            "transitionCallSession",
            "updateCallMedia",
            "sendCallSignal",
            "listCallSignals",
        ] {
            assert!(is_gateway_command(method), "{method} must reach the production Session owner");
        }

        for method in [
            "human.anything",
            "telegram.sendMessage",
            "communication.dispatch",
        ] {
            assert!(
                !is_gateway_command(method),
                "{method} must not widen the explicit FBCP gateway boundary"
            );
        }
    }
}
