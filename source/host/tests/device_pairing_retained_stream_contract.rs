//! DVR-v1 retained HTTP/SSE contracts against the real durable pairing owner.
//! Spec: projects/openclaw-rust/docs/specs/device-pairing-revocation-continuity.md.
//! Real event bus, TCP and local-exec/WebAuthn provider registrations are exercised.
//! These cases do not claim to cancel an already dispatched process or ceremony,
//! prove physical-device teardown, or replace the separate shipping App contracts.
//! All executable verification belongs to exact-H GitHub Actions.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use flate2::read::GzDecoder;
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::local_exec::local_exec_bridge::SandLocalExecBridge;
use mahayana_host_runtime::extensions::mcp::coordinator_relay::HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL;
use mahayana_host_runtime::extensions::webauthn_proxy::webauthn_proxy_bridge::{
    SAND_WEBAUTHN_LIVENESS_WINDOW_MS, SandWebAuthnBridge, SandWebAuthnBridgeOptions,
};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayBridgeClose, GatewayBridgeHub, GatewayCommandError, GatewayCommandReport,
    GatewayEventHub, GatewayServer, GatewayServerDeps, SSE_HEARTBEAT_MS, start_gateway_server,
};
use mahayana_host_runtime::runner::coordinator_tool_relay::RUNNER_TOOL_REQUEST_EVENT_CHANNEL;
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};

const CALLER: &str = "dvr-stream-caller";
const OTHER: &str = "dvr-stream-other";
const SECRET: &str = "dvr-stream-native-secret";
const BARRIER_WAIT: Duration = Duration::from_secs(15);
const IO_WAIT: Duration = Duration::from_secs(35);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase { Before, After }

struct Gate {
    device: String,
    role: String,
    phase: Phase,
    registered: Option<Arc<AtomicUsize>>,
    claimed: AtomicBool,
    entered: SyncSender<Instant>,
    released: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    fn claim(&self, proof: &DeviceTokenAuthorization) -> bool {
        if self.registered.as_ref().is_some_and(|count| count.load(Ordering::SeqCst) == 0) {
            return false;
        }
        proof.device_id == self.device && proof.role == self.role
            && !self.claimed.swap(true, Ordering::SeqCst)
    }

    fn pause(&self) -> Result<(), GatewayCommandError> {
        self.entered.try_send(Instant::now()).map_err(|_| {
            GatewayCommandError::Internal("DVR stream fixture arrival failed".into())
        })?;
        let released = self.released.lock().expect("gate release lock");
        let (released, _) = self.changed.wait_timeout_while(released, BARRIER_WAIT, |value| !*value)
            .expect("bounded admission barrier");
        if !*released {
            return Err(GatewayCommandError::Internal("DVR stream fixture barrier expired".into()));
        }
        Ok(())
    }

    fn release(&self) {
        *self.released.lock().unwrap_or_else(|error| error.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct GateControl { gate: Arc<Gate>, entered: Receiver<Instant> }

impl GateControl {
    fn new(device: &str, role: &str, phase: Phase, registered: Option<Arc<AtomicUsize>>) -> Self {
        let (entered, receiver) = mpsc::sync_channel(1);
        Self {
            gate: Arc::new(Gate {
                device: device.into(), role: role.into(), phase, registered, claimed: AtomicBool::new(false),
                entered, released: Mutex::new(false), changed: Condvar::new(),
            }),
            entered: receiver,
        }
    }

    fn wait(&self) -> Instant {
        self.entered.recv_timeout(BARRIER_WAIT).expect("actual currentness or registration boundary reached")
    }

    fn release(&self) { self.gate.release(); }
}

impl Drop for GateControl {
    fn drop(&mut self) { self.release(); }
}

#[derive(Clone, Debug)]
struct Check {
    device: String,
    role: String,
    status: u16,
    error: Option<String>,
    at: Instant,
}

#[derive(Default)]
struct ApiObservations {
    checks: Mutex<Vec<Check>>,
    calls: AtomicUsize,
    reports: AtomicUsize,
}

impl ApiObservations {
    fn last(&self, device: &str, role: &str) -> Check {
        self.checks.lock().expect("currentness observations").iter().rev()
            .find(|check| check.device == device && check.role == role).expect("observed actual owner check").clone()
    }

    fn len(&self) -> usize { self.checks.lock().expect("checks").len() }
}

struct Api {
    owner: Option<DevicePairingOwner>,
    gate: Mutex<Option<Arc<Gate>>>,
    observed: Arc<ApiObservations>,
    closed: SyncSender<()>,
}

impl Api {
    fn owner(&self) -> &DevicePairingOwner { self.owner.as_ref().expect("open durable owner") }
}

impl GatewayApi for Api {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.observed.calls.fetch_add(1, Ordering::SeqCst);
        self.owner().handle_pairing_command(method, &args)
    }

    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.owner().authenticate_device_token(token)
    }

    fn validate_paired_device_authorization(&self, proof: &DeviceTokenAuthorization) -> Result<(), GatewayCommandError> {
        // Clone and release the instrumentation mutex before either barrier or
        // owner call. The returned policy result always comes from the owner.
        let gate = self.gate.lock().expect("optional admission gate").clone()
            .filter(|gate| gate.claim(proof));
        if let Some(gate) = gate.as_ref().filter(|gate| gate.phase == Phase::Before) {
            gate.pause()?;
        }
        let result = self.owner().validate_current_device_authorization(proof);
        self.observed.checks.lock().expect("check observations").push(Check {
            device: proof.device_id.clone(), role: proof.role.clone(),
            status: result.as_ref().map(|_| 200).unwrap_or_else(|error| error.status()),
            error: result.as_ref().err().map(ToString::to_string), at: Instant::now(),
        });
        if let Some(gate) = gate.as_ref().filter(|gate| gate.phase == Phase::After) {
            gate.pause()?;
        }
        result
    }

    fn on_command_complete(&self, _report: GatewayCommandReport) {
        self.observed.reports.fetch_add(1, Ordering::SeqCst);
    }

    fn on_command_error(&self, _report: GatewayCommandReport) {
        self.observed.reports.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        drop(self.owner.take());
        let _ = self.closed.try_send(());
    }
}

struct LegacyApi(Arc<Api>);

impl GatewayApi for LegacyApi {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> { self.0.call(method, args) }
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.0.authenticate_device_token(token)
    }
    // Deliberately inherit the fail-closed retained-currentness hook.
}

#[derive(Clone, Copy, Debug)]
enum Bridge { LocalExec, WebAuthn }

impl Bridge {
    fn requests(self) -> &'static str {
        match self { Self::LocalExec => "/local-exec/requests", Self::WebAuthn => "/webauthn/requests" }
    }
    fn responses(self) -> &'static str {
        match self { Self::LocalExec => "/local-exec/responses", Self::WebAuthn => "/webauthn/responses" }
    }
}

#[derive(Default)]
struct BridgeObservations {
    registered: Arc<AtomicUsize>,
    closed: AtomicUsize,
    submitted: Mutex<Vec<Value>>,
    registration_gate: Mutex<Option<Arc<Gate>>>,
}

struct Bridges {
    local: SandLocalExecBridge,
    web: SandWebAuthnBridge,
    web_now: Arc<AtomicU64>,
    local_hub: GatewayBridgeHub,
    web_hub: GatewayBridgeHub,
    local_observed: Arc<BridgeObservations>,
    web_observed: Arc<BridgeObservations>,
    local_closed: Receiver<()>,
    web_closed: Receiver<()>,
}

impl Bridges {
    fn new() -> Self {
        let local = SandLocalExecBridge::production();
        let web_now = Arc::new(AtomicU64::new(1000));
        let clock = Arc::clone(&web_now);
        let mut web_options = SandWebAuthnBridgeOptions::production(Arc::new(|_| {}));
        web_options.now_ms = Arc::new(move || clock.load(Ordering::SeqCst));
        let web = SandWebAuthnBridge::new(web_options);
        let local_observed = Arc::new(BridgeObservations::default());
        let web_observed = Arc::new(BridgeObservations::default());
        let (local_closed_tx, local_closed) = mpsc::sync_channel(8);
        let (web_closed_tx, web_closed) = mpsc::sync_channel(8);
        let local_registration = local.clone();
        let local_response = local.clone();
        let registration_observed = Arc::clone(&local_observed);
        let response_observed = Arc::clone(&local_observed);
        let local_hub = GatewayBridgeHub::with_handlers(move |send| {
            let registration = local_registration.register_provider(send);
            registration_observed.registered.fetch_add(1, Ordering::SeqCst);
            let gate = registration_observed.registration_gate.lock().expect("registration gate").clone();
            if let Some(gate) = gate { gate.pause().expect("bounded real registration barrier"); }
            let observed = Arc::clone(&registration_observed);
            let closed = local_closed_tx.clone();
            Box::new(move || {
                drop(registration);
                observed.closed.fetch_add(1, Ordering::SeqCst);
                let _ = closed.try_send(());
            }) as GatewayBridgeClose
        }, move |batch| {
            local_response.submit_responses(batch.clone());
            response_observed.submitted.lock().expect("local response observations").push(batch);
        });
        let web_registration = web.clone();
        let web_response = web.clone();
        let registration_observed = Arc::clone(&web_observed);
        let response_observed = Arc::clone(&web_observed);
        let web_hub = GatewayBridgeHub::with_handlers(move |send| {
            let registration = web_registration.register_provider(send);
            registration_observed.registered.fetch_add(1, Ordering::SeqCst);
            let gate = registration_observed.registration_gate.lock().expect("registration gate").clone();
            if let Some(gate) = gate { gate.pause().expect("bounded real registration barrier"); }
            let observed = Arc::clone(&registration_observed);
            let closed = web_closed_tx.clone();
            Box::new(move || {
                drop(registration);
                observed.closed.fetch_add(1, Ordering::SeqCst);
                let _ = closed.try_send(());
            }) as GatewayBridgeClose
        }, move |batch| {
            web_response.submit_responses(batch.clone());
            response_observed.submitted.lock().expect("WebAuthn response observations").push(batch);
        });
        Self { local, web, web_now, local_hub, web_hub, local_observed, web_observed, local_closed, web_closed }
    }

    fn hub(&self, bridge: Bridge) -> &GatewayBridgeHub {
        match bridge { Bridge::LocalExec => &self.local_hub, Bridge::WebAuthn => &self.web_hub }
    }

    fn observed(&self, bridge: Bridge) -> &BridgeObservations {
        match bridge { Bridge::LocalExec => &self.local_observed, Bridge::WebAuthn => &self.web_observed }
    }

    fn provider_count(&self, bridge: Bridge) -> usize {
        match bridge { Bridge::LocalExec => self.local.provider_count(), Bridge::WebAuthn => self.web.provider_counts().0 }
    }

    fn wait_closed(&self, bridge: Bridge, count: usize) {
        let closed = match bridge { Bridge::LocalExec => &self.local_closed, Bridge::WebAuthn => &self.web_closed };
        for _ in 0..count {
            closed.recv_timeout(BARRIER_WAIT).expect("actual provider registration was dropped");
        }
    }

    fn assert_counts(&self, bridge: Bridge, registered: usize, closed: usize, live: usize) {
        let observed = self.observed(bridge);
        assert_eq!(observed.registered.load(Ordering::SeqCst), registered);
        assert_eq!(observed.closed.load(Ordering::SeqCst), closed, "close callback runs exactly once per retired subscription");
        assert_eq!(self.provider_count(bridge), live, "actual execution-owner provider registry");
    }

    fn release_gates(&self) {
        for observed in [&self.local_observed, &self.web_observed] {
            if let Some(gate) = observed.registration_gate.lock().unwrap_or_else(|error| error.into_inner()).as_ref() {
                gate.release();
            }
        }
    }

    fn assert_response_effect(&self, bridge: Bridge, expected: bool) {
        match bridge {
            Bridge::LocalExec => assert_eq!(self.local.get_provider_info().is_some(), expected),
            Bridge::WebAuthn => assert_eq!(self.web.provider_counts().1, usize::from(expected)),
        }
    }

    fn prepare_response_effect(&self, bridge: Bridge, provider_id: &Value) {
        if matches!(bridge, Bridge::WebAuthn) {
            // A provider with no heartbeat is intentionally live. Establish a
            // real heartbeat, then expire it through the existing clock seam.
            // Only the later candidate HTTP handoff may refresh this provider.
            assert!(provider_id.as_str().is_some());
            self.web.submit_responses(json!({
                "providerId": provider_id, "frames": [{"kind": "ping"}],
            }));
            assert_eq!(self.web.provider_counts(), (1, 1));
            self.web_now.fetch_add(SAND_WEBAUTHN_LIVENESS_WINDOW_MS + 1, Ordering::SeqCst);
            assert_eq!(self.web.provider_counts(), (1, 0));
        }
        self.assert_response_effect(bridge, false);
    }
}

#[derive(Debug, PartialEq)]
struct Snapshot { paired: Vec<Vec<SqlValue>>, pending: Vec<Vec<SqlValue>>, config: Vec<Vec<SqlValue>> }

fn rows(conn: &Connection, sql: &str) -> Vec<Vec<SqlValue>> {
    let mut statement = conn.prepare(sql).expect("prepare independent snapshot");
    let count = statement.column_count();
    statement.query_map([], |row| (0..count).map(|index| row.get(index)).collect::<rusqlite::Result<Vec<SqlValue>>>())
        .expect("query complete raw SQL rows").collect::<rusqlite::Result<Vec<_>>>().expect("all SQL values")
}

struct Fixture {
    root: PathBuf,
    db: PathBuf,
    server: Option<GatewayServer>,
    api: Option<Arc<Api>>,
    closed: Option<Receiver<()>>,
    gates: Vec<Arc<Gate>>,
    observed: Arc<ApiObservations>,
    events: GatewayEventHub,
    bridges: Bridges,
    shutdown_failed: bool,
}

impl Fixture {
    fn new(configured: bool, host: &str, legacy: bool) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-dvr-stream-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let db = root.join("pairing.sqlite");
        let (closed_tx, closed) = mpsc::sync_channel(1);
        let observed = Arc::new(ApiObservations::default());
        let api = Arc::new(Api {
            owner: Some(DevicePairingOwner::new(&db).expect("real file-backed owner")),
            gate: Mutex::new(None), observed: Arc::clone(&observed), closed: closed_tx,
        });
        let dispatch: Arc<dyn GatewayApi> = if legacy { Arc::new(LegacyApi(Arc::clone(&api))) } else { api.clone() };
        let events = GatewayEventHub::default();
        let bridges = Bridges::new();
        let server = start_gateway_server(GatewayServerDeps {
            api: dispatch, events: events.clone(), local_exec: Some(bridges.local_hub.clone()),
            webauthn: Some(bridges.web_hub.clone()),
            config: GatewayServerConfig {
                host: host.into(), port: None, auth_token: configured.then(|| SECRET.into()), tls: None,
            }, started_at: 1,
        }).expect("start actual TCP Gateway");
        let fixture = Self {
            root, db, server: Some(server), api: Some(api), closed: Some(closed), gates: Vec::new(),
            observed, events, bridges, shutdown_failed: false,
        };
        fixture.pair(CALLER, &["operator", "node"], &["operator.read", "node.invoke"]);
        fixture.pair(OTHER, &["operator"], &["operator.read"]);
        fixture
    }

    fn owner(&self) -> &DevicePairingOwner { self.api.as_ref().expect("live API").owner() }
    fn port(&self) -> u16 { self.server.as_ref().expect("owned server").port() }

    fn pair(&self, device: &str, roles: &[&str], scopes: &[&str]) {
        let pending = self.owner().handle_pair_request(&json!({
            "deviceId": device, "publicKey": format!("dvr-stream-key-{device}"),
            "role": roles[0], "roles": roles, "scopes": scopes,
        })).expect("real pairing request");
        self.owner().handle_pair_approve(&json!({"requestId": pending["requestId"]})).expect("real approval");
    }

    fn token(&self, device: &str, role: &str) -> String {
        let conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("read-only credential observer");
        let raw: String = conn.query_row("SELECT tokens_json FROM device_pairing_paired WHERE device_id = ?1", params![device], |row| row.get(0))
            .expect("observe real issued token");
        conn.close().expect("close credential observer");
        let value: Value = serde_json::from_str(&raw).expect("stored token map");
        value[role]["token"].as_str().expect("actual role credential").into()
    }

    fn snapshot(&self) -> Snapshot {
        let mut conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("independent SQL observer");
        let tx = conn.transaction().expect("consistent snapshot");
        let state = Snapshot {
            paired: rows(&tx, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: rows(&tx, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            config: rows(&tx, "SELECT *, typeof(value_json), typeof(updated_at_ms) FROM config_machine_state ORDER BY state_key"),
        };
        tx.commit().expect("finish snapshot");
        conn.close().expect("close observer before mutation or cleanup");
        state
    }

    fn sql(&self, sql: &str) {
        let conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_WRITE).expect("explicit fixture mutation connection");
        conn.execute_batch(sql).expect("controlled storage mutation");
        conn.close().expect("close fixture writer");
    }

    fn revoke(&self, device: &str) {
        self.owner().handle_pair_revoke(&json!({"deviceId": device})).expect("commit real whole-device removal");
    }

    fn gate(&mut self, device: &str, role: &str, phase: Phase) -> GateControl {
        let control = GateControl::new(device, role, phase, None);
        *self.api.as_ref().expect("live API").gate.lock().expect("install admission gate") = Some(Arc::clone(&control.gate));
        self.gates.push(Arc::clone(&control.gate));
        control
    }

    fn registration_gate(&mut self, bridge: Bridge) -> GateControl {
        let control = GateControl::new(CALLER, "operator", Phase::Before, None);
        *self.bridges.observed(bridge).registration_gate.lock().expect("install registration gate") = Some(Arc::clone(&control.gate));
        self.gates.push(Arc::clone(&control.gate));
        control
    }

    fn admitted_header_gate(&mut self, bridge: Bridge) -> GateControl {
        let registered = Arc::clone(&self.bridges.observed(bridge).registered);
        let control = GateControl::new(CALLER, "operator", Phase::After, Some(registered));
        *self.api.as_ref().expect("live API").gate.lock().expect("install post-registration gate") = Some(Arc::clone(&control.gate));
        self.gates.push(Arc::clone(&control.gate));
        control
    }

    fn open(&self, path: &str, token: Option<&str>, headers: &str, gzip: bool) -> Sse {
        let mut stream = Sse::connect(self.port(), path, token, headers, gzip).expect("actual SSE connection");
        assert_eq!(stream.frame(), "retry: 1000\n", "initial admitted retry frame");
        stream
    }

    fn ready_event(&self, stream: &mut Sse, marker: &str) {
        let value = json!({"channel": "runtime", "payload": {"marker": marker}});
        self.events.publish(value.clone());
        assert_eq!(stream.data(), value, "positive progress proves the subscription is live");
    }

    fn stop_listener(&mut self) {
        for gate in &self.gates { gate.release(); }
        self.bridges.release_gates();
        if let Some(server) = self.server.take() { server.close(); }
        // Existing receive wakes make owned connection teardown prompt. Tests
        // separately require actual EOF; publishing alone is never closure proof.
        self.events.publish(json!({"channel": "dvr-cleanup"}));
        self.bridges.local_hub.publish_request(json!({"kind": "dvr-cleanup"}));
        self.bridges.web_hub.publish_request(json!({"kind": "dvr-cleanup"}));
    }

    fn finish(&mut self) {
        self.stop_listener();
        drop(self.api.take());
        if let Some(closed) = self.closed.take() {
            self.shutdown_failed |= closed.recv_timeout(IO_WAIT).is_err();
        }
        assert!(!self.shutdown_failed, "connection/API/SQLite owner teardown must be acknowledged");
        assert_eq!(self.bridges.local.provider_count(), 0);
        assert_eq!(self.bridges.web.provider_counts().0, 0);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop_listener();
        drop(self.api.take());
        if let Some(closed) = self.closed.take() {
            self.shutdown_failed |= closed.recv_timeout(IO_WAIT).is_err();
        }
        if !self.shutdown_failed {
            let result = fs::remove_dir_all(&self.root);
            if !thread::panicking() { result.expect("remove only the closed owned fixture directory"); }
        } else {
            eprintln!("DVR stream fixture retained after unacknowledged teardown: {}", self.root.display());
            assert!(thread::panicking(), "bounded owned connection/API teardown failed");
        }
    }
}

fn connect(port: u16) -> Result<TcpStream, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = TcpStream::connect_timeout(&address, IO_WAIT).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    Ok(stream)
}

// A socket timeout alone restarts on each successful read: a broken stream
// emitting a heartbeat forever could otherwise hang an EOF oracle forever.
// This bound also applies inside BufReader and the incremental gzip decoder.
struct DeadlineReader { stream: TcpStream, deadline: Arc<Mutex<Instant>> }

impl Read for DeadlineReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() { return Ok(0); }
        let deadline = *self.deadline.lock().expect("I/O deadline");
        let remaining = deadline.checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "DVR absolute I/O deadline expired"))?;
        self.stream.set_read_timeout(Some(remaining))?;
        self.stream.read(buffer)
    }
}

struct Sse {
    reader: Box<dyn BufRead + Send>,
    socket: TcpStream,
    deadline: Arc<Mutex<Instant>>,
}

impl Sse {
    fn connect(port: u16, path: &str, token: Option<&str>, headers: &str, gzip: bool) -> Result<Self, String> {
        let mut stream = connect(port)?;
        let socket = stream.try_clone().map_err(|error| error.to_string())?;
        let authorization = token.map(|token| format!("Authorization: Bearer {token}\r\n")).unwrap_or_default();
        let encoding = if gzip { "Accept-Encoding: gzip\r\n" } else { "" };
        write!(stream, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{headers}{encoding}Connection: keep-alive\r\n\r\n")
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
        let deadline = Arc::new(Mutex::new(Instant::now() + IO_WAIT));
        let mut reader = BufReader::new(DeadlineReader { stream, deadline: Arc::clone(&deadline) });
        let mut response = String::new();
        loop {
            let mut line = String::new();
            let count = reader.read_line(&mut line).map_err(|error| error.to_string())?;
            if count == 0 { return Err("EOF before SSE headers".into()); }
            let complete = line == "\r\n";
            response.push_str(&line);
            if response.len() > 8192 { return Err("oversized SSE headers".into()); }
            if complete { break; }
        }
        if !response.starts_with("HTTP/1.1 200 OK") { return Err(response); }
        if !response.to_ascii_lowercase().contains("content-type: text/event-stream") {
            return Err(format!("missing SSE content type: {response}"));
        }
        if response.to_ascii_lowercase().contains("content-encoding: gzip") != gzip {
            return Err(format!("unexpected gzip negotiation: {response}"));
        }
        let reader: Box<dyn BufRead + Send> = if gzip { Box::new(BufReader::new(GzDecoder::new(reader))) } else { Box::new(reader) };
        Ok(Self { reader, socket, deadline })
    }

    fn frame(&mut self) -> String {
        *self.deadline.lock().expect("frame deadline") = Instant::now() + IO_WAIT;
        let mut frame = String::new();
        loop {
            let mut line = String::new();
            assert!(self.reader.read_line(&mut line).expect("bounded SSE frame read") > 0, "EOF before expected admitted frame");
            if line == "\n" { return frame; }
            frame.push_str(&line);
            assert!(frame.len() < 128 * 1024, "bounded frame size");
        }
    }

    fn data(&mut self) -> Value {
        let frame = self.frame();
        serde_json::from_str(frame.strip_prefix("data: ").expect("expected application frame").trim_end())
            .expect("actual SSE JSON")
    }

    fn empty_eof(&mut self) {
        *self.deadline.lock().expect("EOF deadline") = Instant::now() + IO_WAIT;
        let mut tail = String::new();
        self.reader.read_to_string(&mut tail).expect("actual EOF before the I/O deadline; timeout is failure");
        assert!(tail.trim().is_empty(), "no unadmitted frame, heartbeat or second HTTP response: {tail}");
    }
}

impl Drop for Sse {
    fn drop(&mut self) { let _ = self.socket.shutdown(Shutdown::Both); }
}

#[derive(Debug)]
struct Reply { status: u16, body: Value, raw: String }

fn request(port: u16, verb: &str, path: &str, body: &str, authorization: Option<&str>) -> Result<Reply, String> {
    let mut stream = connect(port)?;
    let header = authorization.map(|value| format!("Authorization: {value}\r\n")).unwrap_or_default();
    write!(stream, "{verb} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{header}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let mut raw = String::new();
    DeadlineReader { stream, deadline: Arc::new(Mutex::new(Instant::now() + IO_WAIT)) }
        .read_to_string(&mut raw).map_err(|error| error.to_string())?;
    let status = raw.lines().next().and_then(|line| line.split_whitespace().nth(1)).ok_or("missing HTTP status")?
        .parse::<u16>().map_err(|error| error.to_string())?;
    let body = serde_json::from_str(raw.split_once("\r\n\r\n").ok_or("missing HTTP body")?.1)
        .map_err(|error| error.to_string())?;
    Ok(Reply { status, body, raw })
}

struct Pending<T> { result: Receiver<Result<T, String>>, thread: Option<JoinHandle<()>>, gate: Arc<Gate> }

impl<T: Send + 'static> Pending<T> {
    fn start(gate: &GateControl, work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Self {
        let (sender, result) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || { let _ = sender.send(work()); });
        Self { result, thread: Some(handle), gate: Arc::clone(&gate.gate) }
    }

    fn finish(mut self) -> T {
        let result = self.result.recv_timeout(IO_WAIT).expect("bounded owned client completion");
        self.thread.take().expect("owned client").join().expect("owned client terminal");
        result.expect("actual TCP transport")
    }
}

impl<T> Drop for Pending<T> {
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.gate.release();
            if self.result.recv_timeout(IO_WAIT).is_ok() {
                let _ = self.thread.take().expect("owned client").join();
            }
        }
    }
}

fn event_gate_arrived(gate: &GateControl, last_check: Instant) {
    let arrived = gate.wait();
    // The stream has consumed a positive marker, and exactly the tested frame
    // was then queued. A heartbeat cannot have woken before this lower bound.
    // Missing checks on filtered frames therefore cannot pass at a later timer.
    assert!(arrived.duration_since(last_check) < Duration::from_millis(SSE_HEARTBEAT_MS),
        "queued-frame admission must be observed before a possible quiet heartbeat");
}

fn finish_streams(fixture: &mut Fixture, streams: &mut [&mut Sse]) {
    fixture.stop_listener();
    for stream in streams { stream.empty_eof(); }
    fixture.finish();
}

#[test]
fn retained_initial_admission_refuses_before_headers_subscription_or_provider_registration() {
    for path in ["/events", "/local-exec/requests", "/webauthn/requests"] {
        let mut fixture = Fixture::new(true, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        let gate = fixture.gate(CALLER, "operator", Phase::Before);
        let port = fixture.port();
        let bearer = format!("Bearer {token}");
        let pending = Pending::start(&gate, move || request(port, "GET", path, "", Some(&bearer)));
        gate.wait();
        assert_eq!(fixture.events.subscriber_count(), 0);
        for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
            fixture.bridges.assert_counts(bridge, 0, 0, 0);
        }
        fixture.revoke(CALLER);
        let committed = fixture.snapshot();
        gate.release();
        let reply = pending.finish();
        assert_eq!(reply.status, 401, "{}", reply.raw);
        assert!(reply.body["error"].as_str().is_some());
        assert!(!reply.raw.contains("text/event-stream"));
        assert!(!reply.raw.contains("retry:"));
        assert!(!reply.raw.contains(&token));
        assert_eq!(fixture.snapshot(), committed);
        assert_eq!(fixture.events.subscriber_count(), 0);
        for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
            fixture.bridges.assert_counts(bridge, 0, 0, 0);
        }
        fixture.finish();
    }
}

#[test]
fn retained_default_hook_refuses_paired_streams_and_preserves_native_streams() {
    for bridge in [None, Some(Bridge::LocalExec), Some(Bridge::WebAuthn)] {
        let mut fixture = Fixture::new(true, "127.0.0.1", true);
        let path = bridge.map(Bridge::requests).unwrap_or("/events");
        let token = fixture.token(CALLER, "operator");
        let before = fixture.snapshot();
        let refused = request(fixture.port(), "GET", path, "", Some(&format!("Bearer {token}"))).expect("default-hook HTTP response");
        assert_eq!(refused.status, 403, "{}", refused.raw);
        assert!(!refused.raw.contains("retry:"));
        assert_eq!(fixture.events.subscriber_count(), 0);
        if let Some(bridge) = bridge { fixture.bridges.assert_counts(bridge, 0, 0, 0); }
        let mut native = fixture.open(path, Some(SECRET), "", false);
        if let Some(bridge) = bridge {
            assert_eq!(native.data()["kind"], "welcome");
            fixture.bridges.assert_counts(bridge, 1, 0, 1);
        } else {
            fixture.ready_event(&mut native, "native-default-hook-control");
        }
        assert_eq!(fixture.snapshot(), before);
        assert_eq!(fixture.observed.len(), 0, "native streams never invoke paired currentness");
        finish_streams(&mut fixture, &mut [&mut native]);
        if let Some(bridge) = bridge {
            fixture.bridges.wait_closed(bridge, 1);
            fixture.bridges.assert_counts(bridge, 1, 1, 0);
        }
    }
}

#[test]
fn retained_bridges_recheck_after_registration_before_headers_and_drop_the_provider_once() {
    for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
        let mut fixture = Fixture::new(true, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        let gate = fixture.registration_gate(bridge);
        let port = fixture.port();
        let bearer = format!("Bearer {token}");
        let pending = Pending::start(&gate, move || request(port, "GET", bridge.requests(), "", Some(&bearer)));
        gate.wait();
        fixture.bridges.assert_counts(bridge, 1, 0, 1);
        // The initial admission succeeded and a real provider now exists; its
        // welcome is queued. Revoke before the post-registration header check.
        fixture.revoke(CALLER);
        let committed = fixture.snapshot();
        gate.release();
        let reply = pending.finish();
        assert_eq!(reply.status, 401, "{}", reply.raw);
        assert!(!reply.raw.contains("text/event-stream"));
        assert!(!reply.raw.contains("retry:"));
        assert!(!reply.raw.contains("welcome"));
        assert!(!reply.raw.contains(&token));
        fixture.bridges.wait_closed(bridge, 1);
        fixture.bridges.assert_counts(bridge, 1, 1, 0);
        assert_eq!(fixture.snapshot(), committed);
        fixture.bridges.hub(bridge).publish_request(json!({"kind": "after-closed-registration"}));
        assert_eq!(fixture.bridges.hub(bridge).request_subscriber_count(), 0);
        fixture.finish();
    }
}

#[test]
fn retained_bridges_recheck_initial_retry_after_the_header_admission_has_succeeded() {
    for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
        let mut fixture = Fixture::new(true, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        // This is armed by real registration, not a hard-coded number of API
        // checks. Pre-subscription checks cannot claim it.
        let gate = fixture.admitted_header_gate(bridge);
        let port = fixture.port();
        let pending = Pending::start(&gate, move || Sse::connect(port, bridge.requests(), Some(&token), "", false));
        gate.wait();
        fixture.bridges.assert_counts(bridge, 1, 0, 1);
        assert_eq!(fixture.observed.last(CALLER, "operator").status, 200);
        fixture.revoke(CALLER);
        let committed = fixture.snapshot();
        gate.release();
        let mut stream = pending.finish();
        // Already-admitted headers may finish; the retry and queued provider
        // welcome need their own currentness admission after this commit.
        stream.empty_eof();
        fixture.bridges.wait_closed(bridge, 1);
        fixture.bridges.assert_counts(bridge, 1, 1, 0);
        assert_eq!(fixture.observed.last(CALLER, "operator").status, 401);
        assert_eq!(fixture.snapshot(), committed);
        fixture.finish();
    }
}

#[test]
fn retained_events_revoke_before_queued_admission_closes_with_unchanged_and_unrelated_controls() {
    for gzip in [false, true] {
        for action in ["unchanged", "unrelated", "remove"] {
            let mut fixture = Fixture::new(false, "127.0.0.1", false);
            let token = fixture.token(CALLER, "operator");
            let mut stream = fixture.open("/events", Some(&token), "", gzip);
            fixture.ready_event(&mut stream, "before-queued-frame");
            let ready_check = fixture.observed.last(CALLER, "operator");
            assert_eq!(ready_check.status, 200);
            let before = fixture.snapshot();
            let gate = fixture.gate(CALLER, "operator", Phase::Before);
            let queued = json!({"channel": "runtime", "payload": {"marker": "queued-before-currentness"}});
            fixture.events.publish(queued.clone());
            event_gate_arrived(&gate, ready_check.at);
            assert_eq!(fixture.snapshot(), before, "readiness and admission instrumentation do not mutate storage");
            match action {
                "unrelated" => { fixture.owner().handle_pair_rotate_token(&json!({"deviceId": OTHER})).expect("independent role replacement"); }
                "remove" => fixture.revoke(CALLER),
                "unchanged" => {}
                _ => unreachable!(),
            }
            let committed = fixture.snapshot();
            gate.release();
            if action == "remove" {
                stream.empty_eof();
                assert_eq!(fixture.observed.last(CALLER, "operator").status, 401);
                assert_eq!(fixture.snapshot(), committed);
            } else {
                assert_eq!(stream.data(), queued, "live credential survives unrelated revision changes");
                assert_eq!(fixture.snapshot(), committed);
                fixture.revoke(CALLER);
                fixture.events.publish(json!({"channel": "runtime", "payload": {"marker": "later-refused"}}));
                stream.empty_eof();
            }
            assert_eq!(fixture.observed.calls.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.observed.reports.load(Ordering::SeqCst), 0);
            // Sender pruning is lazy in the real bus; EOF is established first.
            fixture.events.publish(json!({"channel": "prune-closed-sender"}));
            assert_eq!(fixture.events.subscriber_count(), 0);
            fixture.finish();
        }
    }
}

#[test]
fn retained_event_admitted_before_commit_may_finish_but_the_next_frame_is_refused() {
    let mut fixture = Fixture::new(false, "127.0.0.1", false);
    let token = fixture.token(CALLER, "operator");
    let mut stream = fixture.open("/events", Some(&token), "", false);
    fixture.ready_event(&mut stream, "before-admission-linearization");
    let ready_check = fixture.observed.last(CALLER, "operator");
    let gate = fixture.gate(CALLER, "operator", Phase::After);
    let admitted = json!({"channel": "runtime", "payload": {"marker": "admitted-before-commit"}});
    fixture.events.publish(admitted.clone());
    event_gate_arrived(&gate, ready_check.at);
    assert_eq!(fixture.observed.last(CALLER, "operator").status, 200);
    // Returning from the real owner check released its database mutex. This
    // commit must complete while the network consumer is still held at the gate.
    fixture.revoke(CALLER);
    let committed = fixture.snapshot();
    gate.release();
    assert_eq!(stream.data(), admitted, "already-admitted I/O may finish after COMMIT");
    fixture.events.publish(json!({"channel": "runtime", "payload": {"marker": "must-not-be-admitted"}}));
    stream.empty_eof();
    assert_eq!(fixture.observed.last(CALLER, "operator").status, 401);
    assert_eq!(fixture.snapshot(), committed);
    fixture.finish();
}

#[test]
fn retained_quiet_events_retire_on_the_real_fifteen_second_heartbeat_with_native_control() {
    assert_eq!(SSE_HEARTBEAT_MS, 15_000, "the existing receive cadence is part of this contract");
    let mut fixture = Fixture::new(false, "127.0.0.1", false);
    let token = fixture.token(CALLER, "operator");
    let mut paired = fixture.open("/events", Some(&token), "", false);
    let mut native = fixture.open("/events", None, "", false);
    let marker = json!({"channel": "runtime", "payload": {"marker": "last-event-before-quiet"}});
    fixture.events.publish(marker.clone());
    assert_eq!(paired.data(), marker);
    assert_eq!(native.data(), marker);
    let last_live = fixture.observed.last(CALLER, "operator");
    assert_eq!(last_live.status, 200);
    fixture.revoke(CALLER);
    let committed = fixture.snapshot();
    // No publish, injected clock, shortened heartbeat or sleep follows removal.
    // A timeout in empty_eof is a failure, never retirement evidence.
    paired.empty_eof();
    let retired = fixture.observed.last(CALLER, "operator");
    assert_eq!(retired.status, 401);
    assert!(retired.at.duration_since(last_live.at) >= Duration::from_millis(SSE_HEARTBEAT_MS),
        "the real quiet recv_timeout must produce this currentness check");
    assert_eq!(native.frame(), ":ping\n", "native quiet stream continues its actual heartbeat");
    assert_eq!(fixture.snapshot(), committed, "quiet checks do not touch last-seen, revision or any row");
    assert_eq!(fixture.observed.calls.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.observed.reports.load(Ordering::SeqCst), 0);
    finish_streams(&mut fixture, &mut [&mut native]);
}

#[test]
fn retained_role_replacement_closes_only_that_role_and_keeps_node_other_device_and_fresh_token_live() {
    let mut fixture = Fixture::new(true, "127.0.0.1", false);
    let operator_token = fixture.token(CALLER, "operator");
    let node_token = fixture.token(CALLER, "node");
    let other_token = fixture.token(OTHER, "operator");
    let mut operator = fixture.open("/events", Some(&operator_token), "", false);
    let mut node = fixture.open("/events", Some(&node_token), "", false);
    let mut other = fixture.open("/events", Some(&other_token), "", false);
    let ready = json!({"channel": "runtime", "payload": {"marker": "three-current-proofs"}});
    fixture.events.publish(ready.clone());
    for stream in [&mut operator, &mut node, &mut other] { assert_eq!(stream.data(), ready); }
    let before_checks = fixture.snapshot();
    assert_eq!(fixture.observed.last(CALLER, "node").status, 200);
    fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER, "role": "operator"})).expect("replace selected role only");
    let committed = fixture.snapshot();
    assert_ne!(committed, before_checks);
    assert_eq!(fixture.token(CALLER, "node"), node_token);
    let after_rotation = json!({"channel": "runtime", "payload": {"marker": "after-operator-rotation"}});
    fixture.events.publish(after_rotation.clone());
    operator.empty_eof();
    assert_eq!(node.data(), after_rotation);
    assert_eq!(other.data(), after_rotation);
    assert_eq!(fixture.snapshot(), committed);
    let refused = request(fixture.port(), "GET", "/events", "", Some(&format!("Bearer {operator_token}"))).expect("old reconnect result");
    assert_eq!(refused.status, 401);
    let replacement = fixture.token(CALLER, "operator");
    assert_ne!(replacement, operator_token);
    let mut fresh = fixture.open("/events", Some(&replacement), "", false);
    let next = json!({"channel": "runtime", "payload": {"marker": "fresh-proof-and-survivors"}});
    fixture.events.publish(next.clone());
    for stream in [&mut fresh, &mut node, &mut other] { assert_eq!(stream.data(), next); }
    operator.empty_eof();
    assert_eq!(fixture.snapshot(), committed);
    finish_streams(&mut fixture, &mut [&mut fresh, &mut node, &mut other]);
}

#[test]
fn retained_approval_narrowing_live_grant_narrowing_corruption_and_storage_failure_close_without_writes() {
    for change in ["approval", "live-grant", "malformed", "storage"] {
        let mut fixture = Fixture::new(false, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        let mut stream = fixture.open("/events", Some(&token), "", false);
        fixture.ready_event(&mut stream, "before-authority-change");
        let ready_check = fixture.observed.last(CALLER, "operator");
        let before = fixture.snapshot();
        let gate = fixture.gate(CALLER, "operator", Phase::Before);
        fixture.events.publish(json!({"channel": "runtime", "payload": {"marker": "refuse-after-authority-change"}}));
        event_gate_arrived(&gate, ready_check.at);
        match change {
            "approval" => fixture.sql("UPDATE device_pairing_paired SET approved_scopes_json = '[\"node.invoke\"]' WHERE device_id = 'dvr-stream-caller'"),
            "live-grant" => fixture.sql("UPDATE device_pairing_paired SET tokens_json = json_set(tokens_json, '$.operator.scopes', json('[]')) WHERE device_id = 'dvr-stream-caller'"),
            "malformed" => fixture.sql("UPDATE device_pairing_paired SET approved_scopes_json = 'dvr-private-invalid-authority' WHERE device_id = 'dvr-stream-caller'"),
            "storage" => fixture.sql("ALTER TABLE device_pairing_paired RENAME TO dvr_saved_pairings"),
            _ => unreachable!(),
        }
        let expected = if change == "storage" { before } else { fixture.snapshot() };
        gate.release();
        stream.empty_eof();
        let check = fixture.observed.last(CALLER, "operator");
        assert_eq!(check.status, if matches!(change, "storage" | "malformed") { 500 } else { 401 });
        let error = check.error.expect("observed owner refusal");
        for secret in [token.as_str(), "dvr-private-invalid-authority", "dvr_saved_pairings"] {
            assert!(!error.contains(secret), "retained refusal must not disclose raw authority or storage diagnostics");
        }
        if change == "storage" { fixture.sql("ALTER TABLE dvr_saved_pairings RENAME TO device_pairing_paired"); }
        assert_eq!(fixture.snapshot(), expected, "currentness/error path changes no data");
        fixture.finish();
    }
}

#[test]
fn retained_filtered_flood_checks_currentness_before_channel_and_both_internal_mcp_filters() {
    for first_channel in ["unselected", HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, RUNNER_TOOL_REQUEST_EVENT_CHANNEL] {
        let mut fixture = Fixture::new(false, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        let mut stream = fixture.open("/events?channels=runtime", Some(&token), "x-sand-slim-avatars: 1\r\n", false);
        fixture.ready_event(&mut stream, "before-filtered-flood");
        let ready_check = fixture.observed.last(CALLER, "operator");
        let gate = fixture.gate(CALLER, "operator", Phase::Before);
        fixture.events.publish(json!({"channel": first_channel, "payload": {"marker": "first-filtered-frame"}}));
        event_gate_arrived(&gate, ready_check.at);
        for sequence in 0..256 {
            fixture.events.publish(json!({"channel": first_channel, "payload": {"sequence": sequence}}));
        }
        fixture.revoke(CALLER);
        let committed = fixture.snapshot();
        gate.release();
        stream.empty_eof();
        assert_eq!(fixture.observed.last(CALLER, "operator").status, 401);
        assert_eq!(fixture.snapshot(), committed);
        fixture.events.publish(json!({"channel": "prune-after-filtered-eof"}));
        assert_eq!(fixture.events.subscriber_count(), 0);
        fixture.finish();
    }
}

#[test]
fn retained_streams_preserve_native_channel_slim_gzip_and_paired_internal_mcp_exclusion() {
    for configured in [false, true] {
        for gzip in [false, true] {
            let mut fixture = Fixture::new(configured, "127.0.0.1", false);
            let operator_token = fixture.token(CALLER, "operator");
            let node_token = fixture.token(CALLER, "node");
            let target = format!("/events?channels=agent-upserted,{HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL},{RUNNER_TOOL_REQUEST_EVENT_CHANNEL}");
            let slim = if gzip { "x-sand-slim-avatars: 1\r\n" } else { "" };
            let mut operator = fixture.open(&target, Some(&operator_token), &format!("{slim}x-sand-scope: operator.write\r\n"), gzip);
            let mut node = fixture.open(&target, Some(&node_token), slim, gzip);
            let mut native = fixture.open(&target, configured.then_some(SECRET), slim, gzip);
            let before = fixture.snapshot();
            fixture.events.publish(json!({"channel": "unselected", "payload": {"marker": "not-selected"}}));
            let host = json!({"channel": HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, "payload": {"requestId": "private-host-id"}});
            let runner = json!({"channel": RUNNER_TOOL_REQUEST_EVENT_CHANNEL, "payload": {"requestId": "private-runner-id"}});
            let marker = json!({"channel": "agent-upserted", "payload": {
                "marker": "after-internal-frames", "agent": {"id": "stream-fixture", "avatarDataUrl": "data:image/png;base64,Zml4dHVyZQ=="},
            }});
            fixture.events.publish(host.clone());
            fixture.events.publish(runner.clone());
            fixture.events.publish(marker.clone());
            let mut projected = marker;
            if gzip { projected["payload"]["agent"]["avatarDataUrl"] = Value::Null; }
            assert_eq!(operator.data(), projected, "read token cannot acquire native MCP authority via a write header");
            assert_eq!(node.data(), projected);
            assert_eq!(native.data(), host);
            assert_eq!(native.data(), runner);
            assert_eq!(native.data(), projected);
            assert_eq!(fixture.snapshot(), before, "successful retained checks are read-only");
            assert_eq!(fixture.observed.calls.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.observed.reports.load(Ordering::SeqCst), 0);
            finish_streams(&mut fixture, &mut [&mut operator, &mut node, &mut native]);
        }
    }
}

#[test]
fn retained_reconnects_refuse_removed_malformed_credentials_and_wildcard_ambient_authority() {
    for configured in [false, true] {
        let mut fixture = Fixture::new(configured, "127.0.0.1", false);
        let token = fixture.token(CALLER, "operator");
        let mut removed_stream = fixture.open("/events", Some(&token), "", false);
        fixture.ready_event(&mut removed_stream, "before-reconnect-retirement");
        fixture.revoke(CALLER);
        fixture.events.publish(json!({"channel": "runtime", "payload": {"marker": "removed-wake"}}));
        removed_stream.empty_eof();
        fixture.events.publish(json!({"channel": "prune-before-reconnect"}));
        assert_eq!(fixture.events.subscriber_count(), 0);
        let committed = fixture.snapshot();
        for authorization in ["Bearer unknown", "Bearer", "Bearer ", "", "Basic ignored", &format!("Bearer {token}")] {
            for path in ["/events", "/local-exec/requests", "/webauthn/requests"] {
                let refused = request(fixture.port(), "GET", path, "", Some(authorization)).expect("failed credential response");
                assert_eq!(refused.status, 401, "{}", refused.raw);
                assert!(!refused.raw.contains("retry:"));
                assert!(!refused.raw.contains(&token));
            }
            for path in ["/local-exec/responses", "/webauthn/responses"] {
                let refused = request(fixture.port(), "POST", path, "{}", Some(authorization)).expect("failed response credential");
                assert_eq!(refused.status, 401, "{}", refused.raw);
            }
        }
        for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
            fixture.bridges.assert_counts(bridge, 0, 0, 0);
            assert!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").is_empty());
        }
        assert_eq!(fixture.events.subscriber_count(), 0);
        let mut native = fixture.open("/events", configured.then_some(SECRET), "", false);
        fixture.ready_event(&mut native, "native-after-invalid-reconnects");
        assert_eq!(fixture.snapshot(), committed);
        removed_stream.empty_eof();
        finish_streams(&mut fixture, &mut [&mut native]);
    }
    let mut fixture = Fixture::new(false, "0.0.0.0", false);
    // Connecting over 127.0.0.1 and sending a loopback Host cannot change the
    // actual listener's wildcard classification.
    let refused = request(fixture.port(), "GET", "/events", "", None).expect("wildcard response");
    assert_eq!(refused.status, 401);
    assert_eq!(fixture.events.subscriber_count(), 0);
    let token = fixture.token(CALLER, "node");
    let mut paired = fixture.open("/events", Some(&token), "", false);
    fixture.ready_event(&mut paired, "paired-wildcard-positive");
    finish_streams(&mut fixture, &mut [&mut paired]);
}

#[test]
fn retained_bridge_requests_drop_each_owned_provider_once_and_keep_native_provider_live() {
    for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
        for role in ["operator", "node"] {
            let mut fixture = Fixture::new(true, "127.0.0.1", false);
            let token = fixture.token(CALLER, role);
            let mut paired = fixture.open(bridge.requests(), Some(&token), "", false);
            assert_eq!(paired.data()["kind"], "welcome");
            let mut native = fixture.open(bridge.requests(), Some(SECRET), "", false);
            assert_eq!(native.data()["kind"], "welcome");
            fixture.bridges.assert_counts(bridge, 2, 0, 2);
            let ready_check = fixture.observed.last(CALLER, role);
            let gate = fixture.gate(CALLER, role, Phase::Before);
            let queued = json!({"kind": "fixture-request", "marker": "bridge-before-admission"});
            fixture.bridges.hub(bridge).publish_request(queued.clone());
            event_gate_arrived(&gate, ready_check.at);
            assert_eq!(native.data(), queued);
            fixture.revoke(CALLER);
            let committed = fixture.snapshot();
            gate.release();
            paired.empty_eof();
            fixture.bridges.wait_closed(bridge, 1);
            fixture.bridges.assert_counts(bridge, 2, 1, 1);
            let next = json!({"kind": "fixture-request", "marker": "native-after-paired-retirement"});
            fixture.bridges.hub(bridge).publish_request(next.clone());
            assert_eq!(native.data(), next);
            paired.empty_eof();
            assert!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").is_empty());
            assert_eq!(fixture.snapshot(), committed);
            fixture.stop_listener();
            native.empty_eof();
            fixture.bridges.wait_closed(bridge, 1);
            fixture.bridges.assert_counts(bridge, 2, 2, 0);
            fixture.bridges.hub(bridge).publish_request(json!({"kind": "prune-owned-closed-registrations"}));
            assert_eq!(fixture.bridges.hub(bridge).request_subscriber_count(), 0);
            fixture.finish();
        }
    }
}

#[test]
fn retained_bridge_responses_admit_after_decode_before_effect_and_keep_native_and_prior_admission_controls() {
    for bridge in [Bridge::LocalExec, Bridge::WebAuthn] {
        for change in ["remove-before", "unchanged", "unrelated", "remove-after"] {
            let mut fixture = Fixture::new(true, "127.0.0.1", false);
            let token = fixture.token(CALLER, "operator");
            // Keep this provider native so only response admission is retired.
            let mut stream = fixture.open(bridge.requests(), Some(SECRET), "", false);
            let welcome = stream.data();
            assert_eq!(welcome["kind"], "welcome");
            fixture.bridges.prepare_response_effect(bridge, &welcome["providerId"]);
            let body = json!({"providerId": welcome["providerId"], "frames": [
                {"kind": "hello", "localRoot": "/fixture", "terminalsFolder": "terminals", "computerId": "fixture-computer", "label": "DVR fixture"},
                {"kind": "ping"},
            ]}).to_string();
            fixture.bridges.assert_response_effect(bridge, false);
            let checks_before_decode = fixture.observed.len();
            let malformed = request(fixture.port(), "POST", bridge.responses(), "{", Some(&format!("Bearer {token}")))
                .expect("malformed response JSON result");
            assert_eq!(malformed.status, 400);
            assert_eq!(fixture.observed.len(), checks_before_decode, "retained response check belongs after successful decoding");
            assert!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").is_empty());
            let phase = if change == "remove-after" { Phase::After } else { Phase::Before };
            let gate = fixture.gate(CALLER, "operator", phase);
            let port = fixture.port();
            let bearer = format!("Bearer {token}");
            let pending_body = body.clone();
            let pending = Pending::start(&gate, move || request(port, "POST", bridge.responses(), &pending_body, Some(&bearer)));
            gate.wait();
            fixture.bridges.assert_response_effect(bridge, false);
            assert!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").is_empty());
            match change {
                "remove-before" | "remove-after" => fixture.revoke(CALLER),
                "unrelated" => { fixture.owner().handle_pair_rotate_token(&json!({"deviceId": OTHER})).expect("unrelated response-control change"); }
                "unchanged" => {}
                _ => unreachable!(),
            }
            let committed = fixture.snapshot();
            gate.release();
            let reply = pending.finish();
            let accepted = change != "remove-before";
            assert_eq!(reply.status, if accepted { 200 } else { 401 }, "{}", reply.raw);
            assert!(!reply.raw.contains(&token));
            if accepted { assert_eq!(reply.body, json!({"ok": true})); }
            fixture.bridges.assert_response_effect(bridge, accepted);
            assert_eq!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").len(), usize::from(accepted));
            assert_eq!(fixture.snapshot(), committed, "response admission is read-only in pairing storage");
            let checks_before_native = fixture.observed.len();
            let native = request(fixture.port(), "POST", bridge.responses(), &body, Some(&format!("Bearer {SECRET}")))
                .expect("native bridge response");
            assert_eq!(native.status, 200);
            assert_eq!(fixture.observed.len(), checks_before_native);
            fixture.bridges.assert_response_effect(bridge, true);
            assert_eq!(fixture.bridges.observed(bridge).submitted.lock().expect("responses").len(), usize::from(accepted) + 1);
            assert_eq!(fixture.observed.calls.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.observed.reports.load(Ordering::SeqCst), 0);
            fixture.stop_listener();
            stream.empty_eof();
            fixture.bridges.wait_closed(bridge, 1);
            fixture.bridges.assert_counts(bridge, 1, 1, 0);
            fixture.finish();
        }
    }
}
