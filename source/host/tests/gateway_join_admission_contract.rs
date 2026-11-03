//! DJL-v1: real TCP Gateway and durable pairing owner. Private limiter tests
//! separately witness exact queue ownership, clock, capacity, stop and poison.

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayEventHub, GatewayHealth,
    GatewayServer, GatewayServerDeps, start_gateway_server,
};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts { auth: usize, authorize: usize, redeem: usize, dispatch: usize }

struct HeldRedemption {
    code: String,
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

struct Hold {
    entered: mpsc::Receiver<()>,
    release: Option<mpsc::Sender<()>>,
}

impl Hold {
    fn wait_entered(&self) {
        self.entered.recv_timeout(Duration::from_secs(5)).expect("actual owner callback entered");
    }
    fn release(mut self) {
        self.release.take().unwrap().send(()).expect("release exactly the owned callback");
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() { let _ = release.send(()); }
    }
}

struct Controls {
    auth: AtomicUsize,
    authorize: AtomicUsize,
    redeem: AtomicUsize,
    dispatch: AtomicUsize,
    held: Mutex<Option<HeldRedemption>>,
    settled: mpsc::Sender<(String, &'static str)>,
}

impl Controls {
    fn counts(&self) -> Counts {
        Counts {
            auth: self.auth.load(Ordering::SeqCst), authorize: self.authorize.load(Ordering::SeqCst),
            redeem: self.redeem.load(Ordering::SeqCst), dispatch: self.dispatch.load(Ordering::SeqCst),
        }
    }
    fn hold(&self, code: &str) -> Hold {
        let (entered, entered_rx) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let mut held = self.held.lock().unwrap();
        assert!(held.is_none(), "one explicit held callback per fixture generation");
        *held = Some(HeldRedemption { code: code.into(), entered, release: release_rx });
        Hold { entered: entered_rx, release: Some(release) }
    }
}

struct PairingApi {
    owner: Option<Arc<DevicePairingOwner>>,
    controls: Arc<Controls>,
    retired: mpsc::Sender<()>,
}

impl GatewayApi for PairingApi {
    fn call(&self, method: &str, _args: Value) -> Result<Value, GatewayCommandError> {
        self.controls.dispatch.fetch_add(1, Ordering::SeqCst);
        Err(GatewayCommandError::UnknownMethod(method.into()))
    }
    fn health(&self) -> GatewayHealth { GatewayHealth::default() }
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.controls.auth.fetch_add(1, Ordering::SeqCst);
        self.owner.as_ref().unwrap().authenticate_device_token(token)
    }
    fn authorize_device_token_request(&self, _authorization: &DeviceTokenAuthorization, _method: &str, _args: &Value) -> Result<(), GatewayCommandError> {
        self.controls.authorize.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn redeem_device_pairing_join_code(&self, code: &str) -> Result<Option<Value>, GatewayCommandError> {
        self.controls.redeem.fetch_add(1, Ordering::SeqCst);
        let held = {
            let mut gate = self.controls.held.lock().unwrap();
            if gate.as_ref().is_some_and(|held| held.code == code) { gate.take() } else { None }
        };
        if let Some(held) = held {
            held.entered.send(()).expect("observe owned callback admission");
            held.release.recv_timeout(Duration::from_secs(10)).expect("bounded owned callback release");
        }
        let result = self.owner.as_ref().unwrap().redeem_device_pairing_join_code(code);
        let kind = match &result { Ok(Some(_)) => "payload", Ok(None) => "miss", Err(_) => "error" };
        let _ = self.controls.settled.send((code.into(), kind));
        result
    }
}

impl Drop for PairingApi {
    fn drop(&mut self) {
        drop(self.owner.take());
        let _ = self.retired.send(());
    }
}

struct Generation {
    server: Option<GatewayServer>,
    port: u16,
    controls: Arc<Controls>,
    settled: mpsc::Receiver<(String, &'static str)>,
    retired: mpsc::Receiver<()>,
    is_retired: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredCode {
    payload: String,
    created_at_ms: i64,
    expires_at_ms: i64,
    redeemed_at_ms: Option<i64>,
    revision: (String, i64),
}

struct Fixture {
    root: PathBuf,
    db_path: PathBuf,
    owner: Option<Arc<DevicePairingOwner>>,
    generations: Vec<Generation>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-join-ingress-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let db_path = root.join("device-pairing.db");
        let owner = DevicePairingOwner::new(&db_path).expect("open actual durable pairing owner");
        let fixture = Self { root, db_path, owner: Some(Arc::new(owner)), generations: Vec::new() };
        fixture.execute("INSERT INTO config_machine_state(state_key,value_json,updated_at_ms) VALUES ('devicePairing.revision','\"7\"',1234)");
        fixture
    }
    fn execute(&self, sql: &str) {
        let connection = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE).expect("open fixture writer");
        connection.execute_batch(sql).expect("explicit fixture fault/repair");
        connection.close().expect("close fixture writer");
    }
    fn register(&self, value: &Value) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        self.owner.as_ref().unwrap().register_device_pairing_join_code(value, now + 600_000).expect("register real live code")
    }
    fn stored(&self, code: &str, payload_column: &str) -> StoredCode {
        assert!(matches!(payload_column, "payload_json" | "payload_obstacle"));
        let connection = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("open independent state observer");
        let (payload, created_at_ms, expires_at_ms, redeemed_at_ms) = connection.query_row(
            &format!("SELECT {payload_column},created_at_ms,expires_at_ms,redeemed_at_ms FROM device_pairing_join_codes WHERE shortcode=?1"),
            params![code], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).expect("observe unchanged durable code columns");
        let revision = connection.query_row("SELECT value_json,updated_at_ms FROM config_machine_state WHERE state_key='devicePairing.revision'", [],
            |row| Ok((row.get(0)?, row.get(1)?))).expect("observe actual pairing revision");
        connection.close().expect("close independent observer");
        StoredCode { payload, created_at_ms, expires_at_ms, redeemed_at_ms, revision }
    }
    fn start(&mut self) -> usize {
        let (settled, settled_rx) = mpsc::channel();
        let controls = Arc::new(Controls {
            auth: AtomicUsize::new(0), authorize: AtomicUsize::new(0), redeem: AtomicUsize::new(0), dispatch: AtomicUsize::new(0),
            held: Mutex::new(None), settled,
        });
        let (retired, retired_rx) = mpsc::channel();
        let server = start_gateway_server(GatewayServerDeps {
            api: Arc::new(PairingApi { owner: Some(Arc::clone(self.owner.as_ref().unwrap())), controls: Arc::clone(&controls), retired }),
            events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: GatewayServerConfig { host: "127.0.0.1".into(), port: None, auth_token: Some("fixture-native-secret".into()), tls: None },
            started_at: 1,
        }).expect("start actual Gateway generation");
        let index = self.generations.len();
        self.generations.push(Generation { port: server.port(), server: Some(server), controls,
            settled: settled_rx, retired: retired_rx, is_retired: false });
        index
    }
    fn port(&self, generation: usize) -> u16 { self.generations[generation].port }
    fn controls(&self, generation: usize) -> Arc<Controls> { Arc::clone(&self.generations[generation].controls) }
    fn request(&self, generation: usize, method: &str, path: &str, headers: &str) -> String {
        request(self.port(generation), method, path, headers)
    }
    fn miss(&self, generation: usize, index: usize) {
        let response = self.request(generation, "GET", &format!("/j/{}", missing_code(index)), "");
        assert_eq!(assert_join_response(&response, 404), json!({"error":"not_found"}));
    }
    fn stop_listener(&mut self, generation: usize) {
        if let Some(server) = self.generations[generation].server.take() { server.close(); }
    }
    fn retire(&mut self, generation: usize) {
        self.stop_listener(generation);
        if !self.generations[generation].is_retired {
            self.generations[generation].retired.recv_timeout(Duration::from_secs(10)).expect("all owned connection/API references retired");
            self.generations[generation].is_retired = true;
        }
    }
    fn wait_settled(&self, generation: usize, code: &str) -> &'static str {
        loop {
            let (settled_code, kind) = self.generations[generation].settled.recv_timeout(Duration::from_secs(5)).expect("actual pairing settlement");
            if settled_code == code { return kind; }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for index in 0..self.generations.len() { self.stop_listener(index); }
        let mut retired = true;
        for generation in &mut self.generations {
            if !generation.is_retired {
                generation.is_retired = generation.retired.recv_timeout(Duration::from_secs(10)).is_ok();
                retired &= generation.is_retired;
            }
        }
        drop(self.owner.take());
        if thread::panicking() {
            eprintln!("DJL fixture retained: {}", self.root.display());
            return;
        }
        assert!(retired, "unretired owned connections retain fixture evidence");
        fs::remove_dir_all(&self.root).expect("remove only retired owned fixture directory");
    }
}

fn missing_code(index: usize) -> String { format!("absent{index:016}") }

fn send(port: u16, method: &str, path: &str, headers: &str) -> TcpStream {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect real socket");
    stream.set_read_timeout(Some(Duration::from_secs(10))).expect("bounded response read");
    stream.set_write_timeout(Some(Duration::from_secs(5))).expect("bounded request write");
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{headers}Content-Length: 0\r\nConnection: close\r\n\r\n").expect("send complete request");
    stream.flush().expect("flush request");
    stream
}

fn receive(mut stream: TcpStream) -> String {
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("receive completed actual HTTP response");
    response
}

fn request(port: u16, method: &str, path: &str, headers: &str) -> String { receive(send(port, method, path, headers)) }

fn assert_join_response(response: &str, status: u16) -> Value {
    let reason = match status { 200 => "OK", 404 => "Not Found", 429 => "Too Many Requests", 500 => "Internal Server Error", 503 => "Service Unavailable", _ => panic!("unexpected expected status") };
    assert!(response.starts_with(&format!("HTTP/1.1 {status} {reason}\r\n")), "{response}");
    let (headers, body) = response.split_once("\r\n\r\n").expect("HTTP boundary");
    assert!(headers.lines().any(|line| line.trim_end_matches('\r').eq_ignore_ascii_case("Cache-Control: no-store")), "{headers}");
    if status == 429 {
        let retry = headers.lines().find_map(|line| line.strip_prefix("Retry-After: "))
            .expect("rate rejection has Retry-After").trim().parse::<u64>().expect("integer Retry-After");
        assert!((1..=300).contains(&retry), "positive ceiling delay within default lockout");
    }
    serde_json::from_str(body).expect("JSON response")
}

fn assert_only_consumed(before: &StoredCode, after: &StoredCode) {
    assert!(after.redeemed_at_ms.is_some_and(|at| at > 0));
    let mut expected = before.clone();
    expected.redeemed_at_ms = after.redeemed_at_ms;
    assert_eq!(*after, expected, "only the existing durable consumption marker changes");
}

#[test]
fn join_http_default_budget_ignores_headers_and_blocks_before_owner_callbacks() {
    let mut fixture = Fixture::new("default-budget");
    let payload = json!({"bootstrapToken":"private-live-join-material"});
    let code = fixture.register(&payload);
    let before = fixture.stored(&code, "payload_json");
    let generation = fixture.start();
    let controls = fixture.controls(generation);
    let mut source_ports = BTreeSet::new();
    for index in 0..10 {
        let headers = format!("X-Forwarded-For: 203.0.113.{index}\r\nX-Real-IP: 192.0.2.{index}\r\nForwarded: for=198.51.100.{index}\r\nTailscale-User-Login: forged-{index}\r\n");
        let stream = send(fixture.port(generation), "GET", &format!("/j/{}", missing_code(index)), &headers);
        source_ports.insert(stream.local_addr().unwrap().port());
        assert_eq!(assert_join_response(&receive(stream), 404), json!({"error":"not_found"}));
    }
    assert!(source_ports.len() > 1, "actual changing source ports share one socket-IP budget");
    assert_eq!(controls.counts(), Counts { redeem: 10, ..Counts::default() });
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_json TO payload_obstacle");
    for index in 10..13 {
        let headers = format!("Authorization: Bearer invalid-{index}\r\nx-sand-device-timestamp: 0\r\nx-sand-device-nonce: stale-nonce\r\nx-sand-pairing-revision: stale\r\nx-sand-role: node\r\nX-Forwarded-For: 203.0.113.{index}\r\n");
        let observed = controls.counts();
        let response = fixture.request(generation, "GET", &format!("/j/{code}"), &headers);
        assert_eq!(assert_join_response(&response, 429), json!({"error":"rate_limited"}));
        assert_eq!(controls.counts(), observed, "429 precedes every authentication/authorization/redemption callback");
        assert_eq!(fixture.stored(&code, "payload_obstacle"), before);
        assert!(!response.contains("private-live-join-material") && !response.contains(&code));
    }
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_obstacle TO payload_json");
    fixture.retire(generation);
    assert_eq!(fixture.owner.as_ref().unwrap().redeem_device_pairing_join_code(&code).expect("blocked code remains real and usable"), Some(payload));
    assert_only_consumed(&before, &fixture.stored(&code, "payload_json"));
}

#[test]
fn join_http_namespace_failures_count_without_owner_access() {
    let mut fixture = Fixture::new("namespace");
    let code = fixture.register(&json!({"legacy":"untouched"}));
    let before = fixture.stored(&code, "payload_json");
    let generation = fixture.start();
    let controls = fixture.controls(generation);
    let cases = [
        ("POST", format!("/j/{code}")), ("HEAD", format!("/context/j/{code}")),
        ("GET", "/j".into()), ("GET", "/context/j".into()),
        ("GET", "/j/invalid".into()), ("GET", format!("/j/{code}/extra")),
        ("GET", format!("/j/{code}?query=refused")), ("GET", format!("/j/{code}?")),
        ("PUT", format!("/context/j/{code}")), ("GET", "/j/".into()),
    ];
    for (method, path) in cases {
        let response = fixture.request(generation, method, &path,
            "Authorization: Bearer ignored\r\nx-sand-device-timestamp: 0\r\nx-sand-device-nonce: not-a-credential\r\nx-sand-pairing-revision: stale\r\n");
        assert_eq!(assert_join_response(&response, 404), json!({"error":"not_found"}));
        assert_eq!(controls.counts(), Counts::default());
        assert_eq!(fixture.stored(&code, "payload_json"), before);
    }
    assert_eq!(assert_join_response(&fixture.request(generation, "POST", &format!("/j/{code}"), ""), 429), json!({"error":"rate_limited"}));
    assert_eq!(controls.counts(), Counts::default());
    assert_eq!(fixture.stored(&code, "payload_json"), before);
}

#[test]
fn join_http_success_resets_budget_for_legacy_payloads() {
    let mut fixture = Fixture::new("success-reset");
    let null_code = fixture.register(&Value::Null);
    let original = fixture.register(&json!(["legacy", 7, true]));
    let j_code = format!("j{}", &original[1..]);
    let connection = Connection::open_with_flags(&fixture.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
    assert_eq!(connection.execute("UPDATE device_pairing_join_codes SET shortcode=?1 WHERE shortcode=?2", params![j_code, original]).unwrap(), 1);
    connection.close().unwrap();
    let generation = fixture.start();
    for index in 0..9 { fixture.miss(generation, index); }
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/j/{null_code}"),
        "Authorization: Bearer ignored\r\nx-sand-pairing-revision: stale\r\n"), 200), Value::Null);
    for index in 10..19 { fixture.miss(generation, index); }
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/proxy/j/app/j/{j_code}"), ""), 200), json!(["legacy", 7, true]));
    for index in 20..30 { fixture.miss(generation, index); }
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/j/{}", missing_code(31)), ""), 429), json!({"error":"rate_limited"}));
    assert_eq!(fixture.controls(generation).counts(), Counts { redeem: 30, ..Counts::default() });
    assert!(fixture.stored(&null_code, "payload_json").redeemed_at_ms.is_some());
    assert!(fixture.stored(&j_code, "payload_json").redeemed_at_ms.is_some());
}

#[test]
fn join_http_storage_errors_neither_spend_nor_reset_budget() {
    let mut fixture = Fixture::new("storage-accounting");
    let code = fixture.register(&json!({"credential":"keep-private"}));
    let before = fixture.stored(&code, "payload_json");
    let generation = fixture.start();
    for index in 0..9 { fixture.miss(generation, index); }
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_json TO payload_obstacle");
    for _ in 0..3 {
        let response = fixture.request(generation, "GET", &format!("/j/{code}"), "Authorization: Bearer ignored\r\n");
        assert_eq!(assert_join_response(&response, 500), json!({"error":"device pairing join-code storage unavailable"}));
        assert_eq!(fixture.stored(&code, "payload_obstacle"), before);
    }
    assert_eq!(fixture.controls(generation).counts(), Counts { redeem: 12, ..Counts::default() });
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_obstacle TO payload_json");
    fixture.miss(generation, 20);
    let observed = fixture.controls(generation).counts();
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/j/{code}"), ""), 429), json!({"error":"rate_limited"}));
    assert_eq!(fixture.controls(generation).counts(), observed);
    assert_eq!(observed, Counts { redeem: 13, ..Counts::default() });
    assert_eq!(fixture.stored(&code, "payload_json"), before);
}

#[test]
fn join_http_last_slot_concurrency_settles_before_next_admission() {
    let mut fixture = Fixture::new("last-slot");
    let code = fixture.register(&json!({"valid":"must-remain-live"}));
    let before = fixture.stored(&code, "payload_json");
    let generation = fixture.start();
    for index in 0..9 { fixture.miss(generation, index); }
    let controls = fixture.controls(generation);
    let held_code = missing_code(900);
    let hold = controls.hold(&held_code);
    let port = fixture.port(generation);
    thread::scope(|scope| {
        let held_path = format!("/j/{held_code}");
        let first = scope.spawn(move || request(port, "GET", &held_path, ""));
        hold.wait_entered();
        assert_eq!(controls.counts(), Counts { redeem: 10, ..Counts::default() });
        let (sent, sent_rx) = mpsc::channel();
        let mut queued = Vec::new();
        for index in 0..3 {
            let sent = sent.clone();
            let path = format!("/j/{code}");
            queued.push(scope.spawn(move || {
                let stream = send(port, "GET", &path, &format!("X-Forwarded-For: 203.0.113.{index}\r\n"));
                sent.send(()).expect("observe complete competing request write");
                receive(stream)
            }));
        }
        drop(sent);
        for _ in 0..3 {
            sent_rx.recv_timeout(Duration::from_secs(5)).expect("bounded competing-request admission barrier");
        }
        // All competing TCP requests have been sent while the first real owner
        // callback is held. A live health round trip witnesses listener progress;
        // the private unit oracle separately proves exact queued Arc ownership.
        assert!(request(port, "GET", "/health", "").starts_with("HTTP/1.1 200 OK"));
        assert_eq!(controls.counts(), Counts { redeem: 10, ..Counts::default() });
        hold.release();
        assert_eq!(assert_join_response(&first.join().expect("first owned TCP client"), 404), json!({"error":"not_found"}));
        for client in queued {
            assert_eq!(assert_join_response(&client.join().expect("competing owned TCP client"), 429), json!({"error":"rate_limited"}));
        }
    });
    assert_eq!(controls.counts(), Counts { redeem: 10, ..Counts::default() });
    assert_eq!(fixture.stored(&code, "payload_json"), before);
}

#[test]
fn join_http_new_generation_keeps_old_inflight_state_separate() {
    let mut fixture = Fixture::new("generation");
    let payload = json!({"live":"across-generations"});
    let code = fixture.register(&payload);
    let before = fixture.stored(&code, "payload_json");
    let old = fixture.start();
    for index in 0..9 { fixture.miss(old, index); }
    let old_controls = fixture.controls(old);
    let held_code = missing_code(901);
    let hold = old_controls.hold(&held_code);
    let old_port = fixture.port(old);
    thread::scope(|scope| {
        let path = format!("/j/{held_code}");
        let old_request = scope.spawn(move || request(old_port, "GET", &path, ""));
        hold.wait_entered();
        fixture.stop_listener(old);
        let new = fixture.start();
        for index in 100..109 { fixture.miss(new, index); }
        assert_eq!(fixture.controls(new).counts(), Counts { redeem: 9, ..Counts::default() });
        assert_eq!(fixture.stored(&code, "payload_json"), before);
        hold.release();
        assert_eq!(assert_join_response(&old_request.join().expect("old entered request settles"), 404), json!({"error":"not_found"}));
        fixture.retire(old);
        assert_eq!(old_controls.counts(), Counts { redeem: 10, ..Counts::default() });
        assert_eq!(assert_join_response(&fixture.request(new, "GET", &format!("/j/{code}"), ""), 200), payload);
        assert_eq!(fixture.controls(new).counts(), Counts { redeem: 10, ..Counts::default() });
        assert_only_consumed(&before, &fixture.stored(&code, "payload_json"));
    });
}

#[test]
fn join_http_lost_response_consumes_once_without_replay() {
    let mut fixture = Fixture::new("lost-response");
    let code = fixture.register(&json!({"secret":"consumed-even-without-reader"}));
    let before = fixture.stored(&code, "payload_json");
    let generation = fixture.start();
    for index in 0..9 { fixture.miss(generation, index); }
    let controls = fixture.controls(generation);
    let hold = controls.hold(&code);
    let socket = send(fixture.port(generation), "GET", &format!("/j/{code}"), "");
    hold.wait_entered();
    socket.shutdown(Shutdown::Both).expect("caller disconnects before owner settlement");
    drop(socket);
    hold.release();
    assert_eq!(fixture.wait_settled(generation, &code), "payload");
    assert_eq!(controls.counts(), Counts { redeem: 10, ..Counts::default() });
    let consumed = fixture.stored(&code, "payload_json");
    assert_only_consumed(&before, &consumed);
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/j/{code}"), ""), 404), json!({"error":"not_found"}));
    assert_eq!(fixture.stored(&code, "payload_json"), consumed);
    for index in 100..109 { fixture.miss(generation, index); }
    assert_eq!(assert_join_response(&fixture.request(generation, "GET", &format!("/j/{}", missing_code(999)), ""), 429), json!({"error":"rate_limited"}));
    assert_eq!(controls.counts(), Counts { redeem: 20, ..Counts::default() }, "exact explicit requests, no automatic redemption replay");
    fixture.retire(generation);
    assert_eq!(fixture.stored(&code, "payload_json"), consumed);
}
