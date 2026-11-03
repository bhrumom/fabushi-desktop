//! DRT-v1: actual HTTP admission and selected-role retirement in the durable owner.
//! Spec: projects/openclaw-rust/docs/specs/device-role-token-revocation.md.
//! The adapter delegates every policy decision to DevicePairingOwner. Its
//! ordinary-route probe proves authentication, not shipping transcript behavior.
//! These contracts do not prove App wiring or physical-device cancellation.
//! Execute this complete target only in exact-H GitHub Actions.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandContext, GatewayCommandError, GatewayCommandReport,
    GatewayEventHub, GatewayServer, GatewayServerDeps, start_gateway_server,
};
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REVOKE: &str = "device.token.revoke";
const ROTATE: &str = "device.pair.rotateToken";
const CALLER: &str = "drt-http-caller";
const TARGET: &str = "drt-http-target";
const OTHER: &str = "drt-http-other";
const SECRET: &str = "drt-http-native-secret";
const WAIT: Duration = Duration::from_secs(15);
const IO_WAIT: Duration = Duration::from_secs(35);
type Row = BTreeMap<String, SqlValue>;

#[derive(Clone, Debug, PartialEq)]
struct Snapshot { paired: Vec<Row>, pending: Vec<Row>, config: Vec<Row> }

fn rows(conn: &Connection, sql: &str) -> Vec<Row> {
    let mut statement = conn.prepare(sql).expect("prepare independent SQL snapshot");
    let names: Vec<_> = statement.column_names().into_iter().map(str::to_owned).collect();
    statement.query_map([], |row| {
        names.iter().enumerate().map(|(index, name)| {
            Ok((name.clone(), row.get::<_, SqlValue>(index)?))
        }).collect::<rusqlite::Result<Row>>()
    }).expect("query complete rows").collect::<rusqlite::Result<Vec<_>>>().expect("all raw SQL values")
}

fn text<'a>(row: &'a Row, column: &str) -> &'a str {
    match row.get(column) {
        Some(SqlValue::Text(value)) => value,
        _ => panic!("expected SQL TEXT in {column}"),
    }
}

// Capture these before corrupting an entry. A malformed-storage refusal must
// reach the wire/redaction oracle, not panic while rebuilding its witnesses.
struct Secrets(Vec<String>);

impl Secrets {
    fn assert_redacted(&self, reply: &Reply) {
        for token in &self.0 {
            assert!(!reply.raw.contains(token), "response disclosed a fixture credential");
            let digest = Sha256::digest(token.as_bytes());
            for fingerprint in [
                format!("{digest:x}"), format!("{digest:X}"), format!("{:?}", digest.to_vec()),
                base64::engine::general_purpose::STANDARD.encode(&digest),
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest),
            ] {
                assert!(!reply.raw.contains(&fingerprint), "response disclosed sealed credential evidence");
            }
        }
        assert!(!reply.raw.contains(SECRET));
        for field in ["token", "scopes", "tokens", "fingerprint", "tokenDelivery", "revision"] {
            assert!(reply.body.get(field).is_none(), "unexpected response authority field: {field}");
        }
    }
}

impl Snapshot {
    fn device(&self, device: &str) -> &Row {
        self.paired.iter().find(|row| text(row, "device_id") == device).expect("paired fixture device")
    }

    fn tokens(&self, device: &str) -> Value {
        serde_json::from_str(text(self.device(device), "tokens_json")).expect("fixture token map")
    }

    fn token(&self, device: &str, role: &str) -> String {
        self.tokens(device)[role]["token"].as_str().expect("real owner-issued role credential").to_owned()
    }

    fn secrets(&self) -> Secrets {
        let mut tokens = Vec::new();
        for row in &self.paired {
            // Additional original witnesses are retained explicitly by cases
            // that replace a map or entry with malformed data.
            if let Some(SqlValue::Text(raw)) = row.get("tokens_json") {
                if let Ok(Value::Object(entries)) = serde_json::from_str::<Value>(raw) {
                    for entry in entries.values() {
                        if let Some(token) = entry.get("token").and_then(Value::as_str).filter(|token| !token.is_empty()) {
                            tokens.push(token.to_owned());
                        }
                    }
                }
            }
        }
        Secrets(tokens)
    }

    fn revision(&self) -> u64 {
        let row = self.config.iter().find(|row| text(row, "state_key") == "devicePairing.revision")
            .expect("existing revision row");
        assert_eq!(text(row, "value_type"), "text");
        assert_eq!(text(row, "timestamp_type"), "integer");
        let raw = text(row, "value_json");
        let decimal: String = serde_json::from_str(raw).expect("quoted decimal revision");
        let value = decimal.parse::<u64>().expect("unsigned revision");
        assert_eq!(raw, format!("\"{value}\""));
        value
    }

    fn assert_redacted(&self, reply: &Reply) {
        self.secrets().assert_redacted(reply);
        let error = reply.body["error"].as_str().unwrap_or_default();
        for row in self.paired.iter().chain(&self.pending).chain(&self.config) {
            for (column, value) in row {
                if column.ends_with("_json") {
                    if let SqlValue::Text(raw) = value {
                        if raw.len() > 2 {
                            assert!(!error.contains(raw), "error disclosed raw stored authority");
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Counts { generic: usize, bound: usize, recognized: usize, rotations: usize }

#[derive(Clone)]
struct Admission {
    paired: bool,
    device: Option<String>,
    role: Option<String>,
    ceiling: Option<String>,
    context: GatewayCommandContext,
}

#[derive(Default)]
struct Observations {
    generic: AtomicUsize,
    bound: AtomicUsize,
    recognized: AtomicUsize,
    rotations: AtomicUsize,
    admissions: Mutex<Vec<Admission>>,
    successes: Mutex<Vec<GatewayCommandReport>>,
    errors: Mutex<Vec<GatewayCommandReport>>,
}

impl Observations {
    fn counts(&self) -> Counts {
        Counts {
            generic: self.generic.load(Ordering::SeqCst), bound: self.bound.load(Ordering::SeqCst),
            recognized: self.recognized.load(Ordering::SeqCst), rotations: self.rotations.load(Ordering::SeqCst),
        }
    }
}

// The barrier is reached after real HTTP authentication, before the owner
// transaction. No instrumentation or SQLite mutex is held while paused.
struct Gate {
    method: &'static str,
    remaining: AtomicUsize,
    entered: SyncSender<()>,
    released: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    fn pause(&self, method: &str) -> Result<(), GatewayCommandError> {
        if self.method != method
            || self.remaining.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)).is_err()
        {
            return Ok(());
        }
        self.entered.try_send(()).map_err(|_| GatewayCommandError::Internal("DRT fixture arrival failed".into()))?;
        let released = self.released.lock().expect("gate lock");
        let (released, _) = self.changed.wait_timeout_while(released, WAIT, |value| !*value)
            .expect("bounded authenticated barrier");
        if !*released {
            return Err(GatewayCommandError::Internal("DRT fixture barrier expired".into()));
        }
        Ok(())
    }

    fn release(&self) {
        *self.released.lock().unwrap_or_else(|error| error.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct GateControl { gate: Arc<Gate>, entered: Receiver<()> }

impl GateControl {
    fn wait(&self, count: usize) {
        for _ in 0..count {
            self.entered.recv_timeout(WAIT).expect("actual authenticated bound hook reached");
        }
    }
    fn release(&self) { self.gate.release(); }
}

impl Drop for GateControl { fn drop(&mut self) { self.release(); } }

struct Api {
    owner: Option<DevicePairingOwner>,
    observed: Arc<Observations>,
    gate: Mutex<Option<Arc<Gate>>>,
    closed: SyncSender<()>,
}

impl Api {
    fn owner(&self) -> &DevicePairingOwner { self.owner.as_ref().expect("open durable owner") }
    fn pause(&self, method: &str) -> Result<(), GatewayCommandError> {
        let gate = self.gate.lock().expect("optional hook gate").clone();
        if let Some(gate) = gate { gate.pause(method)?; }
        Ok(())
    }
}

impl GatewayApi for Api {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let result = self.owner().authenticate_device_token(token)?;
        if result.is_some() { self.observed.recognized.fetch_add(1, Ordering::SeqCst); }
        Ok(result)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.observed.generic.fetch_add(1, Ordering::SeqCst);
        if method == "getTranscript" {
            return Ok(json!({"ordinaryAuthenticationProbe": true}));
        }
        self.owner().handle_pairing_command(method, &args)
    }

    fn revoke_device_token(
        &self,
        args: Value,
        authorization: Option<&DeviceTokenAuthorization>,
        scope_ceiling: Option<&str>,
        context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.observed.bound.fetch_add(1, Ordering::SeqCst);
        self.observed.admissions.lock().expect("admission observations").push(Admission {
            paired: authorization.is_some(), device: authorization.map(|proof| proof.device_id.clone()),
            role: authorization.map(|proof| proof.role.clone()), ceiling: scope_ceiling.map(str::to_owned),
            context: context.clone(),
        });
        self.pause(REVOKE)?;
        self.owner().revoke_token_with_authority(&args, authorization, scope_ceiling)
    }

    fn rotate_paired_device_token(
        &self,
        args: Value,
        authorization: &DeviceTokenAuthorization,
        scope_ceiling: Option<&str>,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.observed.rotations.fetch_add(1, Ordering::SeqCst);
        self.pause(ROTATE)?;
        self.owner().rotate_token_for_authenticated_device(&args, authorization, scope_ceiling)
    }

    fn redeem_device_pairing_join_code(&self, shortcode: &str) -> Result<Option<Value>, GatewayCommandError> {
        self.owner().redeem_device_pairing_join_code(shortcode)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        self.observed.successes.lock().expect("success reports").push(report);
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        self.observed.errors.lock().expect("error reports").push(report);
    }
}

impl Drop for Api {
    fn drop(&mut self) { drop(self.owner.take()); let _ = self.closed.try_send(()); }
}

struct LegacyApi(Arc<Api>);

impl GatewayApi for LegacyApi {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> { self.0.call(method, args) }
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.0.authenticate_device_token(token)
    }
    // Inherit the fixed fail-closed DRT hook, including for native requests.
}

#[derive(Debug)]
struct Reply { status: u16, body: Value, raw: String }

fn request(port: u16, verb: &str, path: &str, body: &str, authorization: Option<&str>, headers: &str) -> Result<Reply, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, IO_WAIT).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    let authorization = authorization.map(|value| format!("Authorization: {value}\r\n")).unwrap_or_default();
    write!(stream, "{verb} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let deadline = Instant::now() + IO_WAIT;
    let mut bytes = Vec::new();
    loop {
        let remaining = deadline.checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero()).ok_or("absolute HTTP deadline expired")?;
        stream.set_read_timeout(Some(remaining)).map_err(|error| error.to_string())?;
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 { break; }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > 1024 * 1024 { return Err("oversized HTTP reply".into()); }
    }
    let raw = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    let status = raw.lines().next().and_then(|line| line.split_whitespace().nth(1))
        .ok_or("missing HTTP status")?.parse::<u16>().map_err(|error| error.to_string())?;
    let body = serde_json::from_str(raw.split_once("\r\n\r\n").ok_or("missing HTTP body")?.1)
        .map_err(|error| error.to_string())?;
    Ok(Reply { status, body, raw })
}

struct Pending { result: Receiver<Result<Reply, String>>, thread: Option<JoinHandle<()>>, gate: Arc<Gate> }

impl Pending {
    fn finish(mut self) -> Reply {
        let result = self.result.recv_timeout(IO_WAIT).expect("bounded client completion");
        self.thread.take().expect("owned client").join().expect("completed TCP client");
        result.expect("actual HTTP transport")
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.gate.release();
            if self.result.recv_timeout(IO_WAIT).is_ok() {
                let _ = self.thread.take().expect("owned client").join();
            }
        }
    }
}

struct Fixture {
    root: PathBuf,
    db: PathBuf,
    config: GatewayServerConfig,
    legacy: bool,
    server: Option<GatewayServer>,
    api: Option<Arc<Api>>,
    closed: Option<Receiver<()>>,
    gate: Option<Arc<Gate>>,
    observed: Arc<Observations>,
    shutdown_failed: bool,
}

impl Fixture {
    fn new(configured: bool, host: &str, legacy: bool) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-drt-http-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let mut fixture = Self {
            db: root.join("pairing.sqlite"), root,
            config: GatewayServerConfig { host: host.into(), port: None, auth_token: configured.then(|| SECRET.into()), tls: None },
            legacy, server: None, api: None, closed: None, gate: None,
            observed: Arc::new(Observations::default()), shutdown_failed: false,
        };
        fixture.open();
        fixture
    }

    fn seeded(roles: &[&str], scopes: &[&str]) -> Self {
        let fixture = Self::new(false, "127.0.0.1", false);
        fixture.seed(roles, scopes);
        fixture
    }

    fn open(&mut self) {
        let (closed_tx, closed) = mpsc::sync_channel(1);
        let api = Arc::new(Api {
            owner: Some(DevicePairingOwner::new(&self.db).expect("real durable owner")),
            observed: Arc::clone(&self.observed), gate: Mutex::new(None), closed: closed_tx,
        });
        let dispatch: Arc<dyn GatewayApi> = if self.legacy { Arc::new(LegacyApi(Arc::clone(&api))) } else { api.clone() };
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: dispatch, events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: self.config.clone(), started_at: 1,
        }).expect("bind actual Gateway listener"));
        self.api = Some(api);
        self.closed = Some(closed);
    }

    fn stop(&mut self) -> bool {
        if let Some(gate) = self.gate.take() { gate.release(); }
        if let Some(server) = self.server.take() { server.close(); }
        drop(self.api.take());
        if let Some(closed) = self.closed.take() { self.shutdown_failed |= closed.recv_timeout(IO_WAIT).is_err(); }
        !self.shutdown_failed
    }

    fn reopen(&mut self) { assert!(self.stop(), "acknowledged API/SQLite teardown before reopen"); self.open(); }
    fn owner(&self) -> &DevicePairingOwner { self.api.as_ref().expect("live adapter").owner() }
    fn port(&self) -> u16 { self.server.as_ref().expect("owned server").port() }
    fn native(&self) -> Option<String> { self.config.auth_token.as_ref().map(|token| format!("Bearer {token}")) }

    fn observe(&self) -> Snapshot {
        let mut conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("independent read-only observer");
        let tx = conn.transaction().expect("consistent snapshot");
        let snapshot = Snapshot {
            paired: rows(&tx, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: rows(&tx, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            config: rows(&tx, "SELECT *, typeof(value_json) AS value_type, typeof(updated_at_ms) AS timestamp_type FROM config_machine_state ORDER BY state_key"),
        };
        tx.commit().expect("complete observation");
        conn.close().expect("close observer before mutations or cleanup");
        snapshot
    }

    fn set(&self, sql: &str, parameters: impl rusqlite::Params) {
        let conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_WRITE).expect("independent controlled writer");
        assert_eq!(conn.execute(sql, parameters).expect("explicit fixture mutation"), 1);
        conn.close().expect("commit and close independent writer");
    }

    fn edit_tokens(&self, device: &str, change: impl FnOnce(&mut Value)) {
        let mut entries = self.observe().tokens(device);
        change(&mut entries);
        self.set("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2", params![entries.to_string(), device]);
    }

    fn pair(&self, device: &str, roles: &[&str], scopes: &[&str]) {
        let pending = self.owner().handle_pair_request(&json!({
            "deviceId": device, "publicKey": format!("drt-key-{device}"), "displayName": device,
            "role": roles[0], "roles": roles, "scopes": scopes,
        })).expect("real pairing request");
        self.owner().handle_pair_approve(&json!({"requestId": pending["requestId"]})).expect("real approval");
    }

    fn seed(&self, roles: &[&str], scopes: &[&str]) {
        self.pair(CALLER, roles, scopes);
        self.pair(TARGET, &["operator", "node"], &["operator.pairing", "node.invoke"]);
        self.pair(OTHER, &["operator"], &["operator.read"]);
        for device in [CALLER, TARGET, OTHER] {
            self.owner().handle_pair_request(&json!({
                "deviceId": device, "publicKey": format!("drt-repair-{device}"),
                "roles": ["operator"], "scopes": ["operator.read"],
            })).expect("pending repair preservation witness");
            self.set("UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2 WHERE device_id = ?3",
                params![" {\"generation\":7,\"commands\":[\"system.run\"]} ", " {\"generation\":8} ", device]);
            self.edit_tokens(device, |entries| {
                for entry in entries.as_object_mut().expect("seed role map").values_mut() {
                    entry["drtUnknownMetadata"] = json!({"retain": [1, "sentinel", null]});
                }
            });
        }
        self.set("INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES ('drt.unrelated', '\"preserve\"', 7)", []);
    }

    fn http(&self, verb: &str, path: &str, body: &str, authorization: Option<&str>, headers: &str) -> Reply {
        request(self.port(), verb, path, body, authorization, headers).expect("actual TCP response")
    }

    fn revoke(&self, args: &Value, authorization: Option<&str>, headers: &str) -> Reply {
        self.http("POST", "/api/device.token.revoke", &args.to_string(), authorization, headers)
    }

    fn deny(&self, body: &str, authorization: Option<&str>, headers: &str, status: u16, bound: usize) -> Reply {
        let before = self.observe();
        let counts = self.observed.counts();
        let reply = self.http("POST", "/api/device.token.revoke", body, authorization, headers);
        assert_refused(&reply, status, &before);
        assert_eq!(self.observe(), before, "refusal preserves every SQL row and revision byte/type");
        let after = self.observed.counts();
        assert_eq!(after.generic, counts.generic, "no DRT fallback into generic native dispatch");
        assert_eq!(after.bound, counts.bound + bound);
        assert_eq!(after.rotations, counts.rotations);
        reply
    }

    fn assert_retirement(&self, before: &Snapshot, reply: &Reply, device: &str, role: &str) {
        assert_eq!(reply.status, 200, "{}", reply.raw);
        let timestamp = reply.body["revokedAtMs"].as_i64().filter(|value| *value > 0).expect("positive signed committed timestamp");
        assert_eq!(reply.body, json!({"deviceId": device, "role": role, "revokedAtMs": timestamp}), "closed three-field receipt");
        let after = self.observe();
        let mut expected_tokens = before.tokens(device);
        expected_tokens[role]["revokedAtMs"] = json!(timestamp);
        assert_eq!(after.tokens(device), expected_tokens, "only the selected marker changes; no credential is issued");
        let mut expected_rows = before.paired.clone();
        let expected = expected_rows.iter_mut().find(|row| text(row, "device_id") == device).expect("existing target");
        expected.insert("tokens_json".into(), after.device(device)["tokens_json"].clone());
        if role == "node" {
            expected.insert("node_surface_json".into(), SqlValue::Null);
            expected.insert("pending_node_surface_json".into(), SqlValue::Null);
        }
        assert_eq!(after.paired, expected_rows, "all other columns, entries and devices retain exact stored values");
        assert_eq!(after.pending, before.pending);
        assert_eq!(after.revision(), before.revision() + 1);
        let unrelated = |state: &Snapshot| state.config.iter()
            .filter(|row| text(row, "state_key") != "devicePairing.revision").cloned().collect::<Vec<_>>();
        assert_eq!(unrelated(&after), unrelated(before));
        assert!(self.owner().authenticate_device_token(&before.token(device, role)).expect("fresh retired-token lookup").is_none());
        before.assert_redacted(reply);
    }

    fn gate(&mut self, method: &'static str, count: usize) -> GateControl {
        let (entered, receiver) = mpsc::sync_channel(count);
        let gate = Arc::new(Gate { method, remaining: AtomicUsize::new(count), entered, released: Mutex::new(false), changed: Condvar::new() });
        *self.api.as_ref().expect("adapter").gate.lock().expect("install bound hook gate") = Some(Arc::clone(&gate));
        self.gate = Some(Arc::clone(&gate));
        GateControl { gate, entered: receiver }
    }

    fn spawn(&self, gate: &GateControl, method: &'static str, args: Value, token: &str) -> Pending {
        let port = self.port();
        let body = args.to_string();
        let bearer = format!("Bearer {token}");
        let (sender, result) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let _ = sender.send(request(port, "POST", &format!("/api/{method}"), &body, Some(&bearer), ""));
        });
        Pending { result, thread: Some(handle), gate: Arc::clone(&gate.gate) }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.stop() {
            let result = fs::remove_dir_all(&self.root);
            if !thread::panicking() { result.expect("remove only acknowledged closed fixture"); }
        } else {
            eprintln!("DRT fixture retained after unacknowledged teardown: {}", self.root.display());
            assert!(thread::panicking(), "bounded connection/API teardown failed");
        }
    }
}

fn assert_refused(reply: &Reply, expected: u16, before: &Snapshot) {
    assert_eq!(reply.status, expected, "{}", reply.raw);
    assert!(reply.body["error"].as_str().is_some_and(|error| !error.is_empty()));
    assert!(!reply.raw.contains("DRT fixture"), "fixture failure cannot satisfy a policy-refusal oracle");
    before.assert_redacted(reply);
}

fn assert_bound_once(fixture: &Fixture, before: Counts, paired: bool) {
    assert_eq!(fixture.observed.counts(), Counts {
        bound: before.bound + 1, recognized: before.recognized + usize::from(paired), ..before
    });
}

#[test]
fn token_revoke_http_self_pairing_on_mixed_device_reports_exact_receipt_and_context() {
    for role_header in ["", "x-sand-role: operator\r\n"] {
        let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "node.invoke"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "operator");
        let node = before.token(CALLER, "node");
        let counts = fixture.observed.counts();
        let reply = fixture.revoke(&json!({"deviceId": format!(" {CALLER} "), "role": " operator "}),
            Some(&format!("Bearer {token}")),
            &format!("{role_header}x-sand-scope: operator.pairing\r\nx-sand-request-id: drt-self\r\ntraceparent: 00-0123456789abcdef0123456789abcdef-0123456789abcdef-01\r\n"));
        assert_bound_once(&fixture, counts, true);
        fixture.assert_retirement(&before, &reply, CALLER, "operator");
        assert_eq!(fixture.observe().token(CALLER, "node"), node);
        assert!(fixture.owner().authenticate_device_token(&node).expect("unaffected role authentication").is_some());
        {
            let admissions = fixture.observed.admissions.lock().expect("admissions");
            assert_eq!(admissions.len(), 1);
            let admission = &admissions[0];
            assert!(admission.paired);
            assert_eq!(admission.device.as_deref(), Some(CALLER));
            assert_eq!(admission.role.as_deref(), Some("operator"));
            assert_eq!(admission.ceiling.as_deref(), Some("operator.pairing"));
            assert!(admission.context.authenticated);
            assert_eq!(admission.context.request_id.as_deref(), Some("drt-self"));
            assert_eq!(admission.context.trace_id.as_deref(), Some("0123456789abcdef0123456789abcdef"));
            assert_eq!(admission.context.span_id.as_deref(), Some("0123456789abcdef"));
            let reports = fixture.observed.successes.lock().expect("reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].method, REVOKE);
            assert_eq!(reports[0].status, 200);
            assert_eq!(reports[0].request_id, admission.context.request_id);
            assert_eq!(reports[0].traceparent, admission.context.traceparent);
            assert!(fixture.observed.errors.lock().expect("errors").is_empty());
        }
        let committed = fixture.observe();
        fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&format!("Bearer {token}")), "", 401, 0);
        fixture.reopen();
        assert_eq!(fixture.observe(), committed, "role retirement survives closing every original owner/connection");
        fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&format!("Bearer {token}")), "", 401, 0);
        assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", Some(&format!("Bearer {node}")), "").status, 200);
        assert_eq!(fixture.observe(), committed);
    }
}

#[test]
fn token_revoke_http_admin_management_and_scope_intersection_preserve_unselected_state() {
    let fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "node.invoke"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    let existing = fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&bearer), "", 403, 1);
    let missing = fixture.deny(&json!({"deviceId": "drt-absent", "role": "operator"}).to_string(), Some(&bearer), "", 403, 1);
    assert_eq!(existing.body, missing.body, "unauthorized cross-device addressing discloses no existence");
    let existing_role = fixture.deny(&json!({"deviceId": CALLER, "role": "node"}).to_string(), Some(&bearer), "", 403, 1);
    let missing_role = fixture.deny(&json!({"deviceId": CALLER, "role": "observer"}).to_string(), Some(&bearer), "", 403, 1);
    assert_eq!(existing_role.body, missing_role.body, "unauthorized role addressing precedes lookup");

    let fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&bearer),
        "x-sand-scope: operator.pairing\r\n", 403, 1);
    fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&bearer),
        "x-sand-scope: operator.pairing\r\n", 403, 1);
    fixture.deny(&json!({"deviceId": "drt-authorized-absent", "role": "operator"}).to_string(), Some(&bearer), "", 400, 1);
    // A separately current administrator must be able to retire an existing
    // target after its issuance approval was withdrawn. Node grants are not
    // another issuance prerequisite, and unselected malformed values survive.
    fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[]' WHERE device_id = ?1", params![TARGET]);
    fixture.edit_tokens(TARGET, |entries| entries["node"]["scopes"] = json!({"oldUnsupportedGrant": true}));
    for role in ["operator", "operator", "node", "node"] {
        let before = fixture.observe();
        let counts = fixture.observed.counts();
        let reply = fixture.revoke(&json!({"deviceId": TARGET, "role": role}), Some(&bearer), "x-sand-scope: operator.admin\r\n");
        assert_bound_once(&fixture, counts, true);
        fixture.assert_retirement(&before, &reply, TARGET, role);
    }
    // A repeated retirement still enforces the entire persisted operator grant.
    fixture.edit_tokens(TARGET, |entries| entries["operator"]["scopes"] = json!(["operator.pairing", "node.invoke"]));
    fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&bearer), "", 403, 1);
}

#[test]
fn token_revoke_http_native_ceiling_is_bound_for_secret_and_absent_loopback() {
    for configured in [false, true] {
        for ceiling in [None, Some("operator.admin"), Some("operator.pairing")] {
            let fixture = Fixture::new(configured, "127.0.0.1", false);
            fixture.seed(&["operator"], &["operator.pairing"]);
            let native = fixture.native();
            let header = ceiling.map(|scope| format!("x-sand-scope: {scope}\r\n")).unwrap_or_default();
            let before = fixture.observe();
            let counts = fixture.observed.counts();
            let reply = fixture.revoke(&json!({"deviceId": TARGET, "role": "operator"}), native.as_deref(),
                &format!("x-sand-role: operator\r\n{header}"));
            assert_bound_once(&fixture, counts, false);
            fixture.assert_retirement(&before, &reply, TARGET, "operator");
            let admission = fixture.observed.admissions.lock().expect("native admission").last().expect("bound native call").clone();
            assert!(!admission.paired);
            assert!(admission.device.is_none() && admission.role.is_none(), "native admission has no fabricated paired identity");
            assert_eq!(admission.ceiling.as_deref(), ceiling);
            assert!(admission.context.authenticated);
            if ceiling == Some("operator.pairing") {
                fixture.deny(&json!({"deviceId": TARGET, "role": "node"}).to_string(), native.as_deref(), &header, 403, 1);
                fixture.edit_tokens(TARGET, |entries| entries["operator"]["scopes"] = json!(["operator.pairing", "operator.read"]));
                fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), native.as_deref(), &header, 403, 1);
            } else {
                let before = fixture.observe();
                let reply = fixture.revoke(&json!({"deviceId": TARGET, "role": "node"}), native.as_deref(), &header);
                fixture.assert_retirement(&before, &reply, TARGET, "node");
            }
        }
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let native = fixture.native();
        for (ceiling, bound) in [("operator.write", 1), ("operator.read", 0), ("", 1),
            ("operator.pairing operator.admin", 1), ("operator.pairing,operator.admin", 1), ("node.invoke", 1)] {
            fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), native.as_deref(),
                &format!("x-sand-scope: {ceiling}\r\n"), 403, bound);
        }
        for role in ["node", "observer", "Operator", ""] {
            fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), native.as_deref(),
                &format!("x-sand-role: {role}\r\n"), 403, 0);
        }
        fixture.edit_tokens(TARGET, |entries| entries["operator"]["scopes"] = json!([]));
        let before = fixture.observe();
        let reply = fixture.revoke(&json!({"deviceId": TARGET, "role": "operator"}), native.as_deref(), "x-sand-scope: operator.pairing\r\n");
        fixture.assert_retirement(&before, &reply, TARGET, "operator");
    }
}

#[test]
fn token_revoke_http_closed_request_and_storage_errors_are_redacted_and_atomic() {
    let mut fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
    let original = fixture.observe();
    let secrets = original.secrets();
    let bearer = format!("Bearer {}", original.token(CALLER, "operator"));
    for args in [json!(null), json!([]), json!("request"), json!(7), json!({}),
        json!({"deviceId": TARGET}), json!({"role": "operator"}),
        json!({"deviceId": null, "role": "operator"}), json!({"deviceId": 7, "role": "operator"}),
        json!({"deviceId": "  ", "role": "operator"}), json!({"deviceId": TARGET, "role": null}),
        json!({"deviceId": TARGET, "role": 7}), json!({"deviceId": TARGET, "role": " "})] {
        fixture.deny(&args.to_string(), Some(&bearer), "", 400, 1);
    }
    for (key, value) in [("scopes", json!([])), ("crossDevice", json!(true)), ("callerDeviceId", json!(TARGET)),
        ("callerRole", json!("operator")), ("isAdmin", json!(true)), ("authorization", json!({"scopes": ["operator.admin"]}))] {
        let mut args = json!({"deviceId": TARGET, "role": "operator"});
        args[key] = value;
        fixture.deny(&args.to_string(), Some(&bearer), "", 400, 1);
    }
    fixture.deny("{", Some(&bearer), "", 400, 0);
    fixture.deny(&json!({"deviceId": TARGET, "role": "Operator"}).to_string(), Some(&bearer), "", 400, 1);
    fixture.deny(&json!({"deviceId": "drt-missing-target", "role": "operator"}).to_string(), Some(&bearer), "", 400, 1);
    let args = json!({"deviceId": TARGET, "role": "operator"}).to_string();
    for invalid in [None, Some(json!(null)), Some(json!(7)), Some(json!({"privateGrant": "drt-private-grant"})),
        Some(json!(["operator.pairing", 7]))] {
        fixture.set("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
            params![original.device(TARGET)["tokens_json"].clone(), TARGET]);
        fixture.edit_tokens(TARGET, |entries| {
            let entry = entries["operator"].as_object_mut().expect("selected entry");
            match invalid { Some(value) => { entry.insert("scopes".into(), value); }, None => { entry.remove("scopes"); } }
        });
        let errors = fixture.observed.errors.lock().expect("errors").len();
        let reply = fixture.deny(&args, Some(&bearer), "x-sand-request-id: drt-invalid-grant\r\n", 500, 1);
        assert_eq!(reply.body["error"], "stored pairing authority is invalid");
        secrets.assert_redacted(&reply);
        assert!(!reply.raw.contains("drt-private-grant"));
        let reports = fixture.observed.errors.lock().expect("normal error reports");
        assert_eq!(reports.len(), errors + 1);
        let report = reports.last().unwrap();
        assert_eq!(report.method, REVOKE);
        assert_eq!(report.status, 500);
        assert_eq!(report.request_id.as_deref(), Some("drt-invalid-grant"));
        assert_eq!(report.error.as_deref(), Some("stored pairing authority is invalid"));
    }
    fixture.set("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
        params![original.device(TARGET)["tokens_json"].clone(), TARGET]);
    for invalid in [SqlValue::Null, SqlValue::Blob(vec![0xff, 0]), SqlValue::Text("drt-private-roles".into()), SqlValue::Text("[7]".into())] {
        fixture.set("UPDATE device_pairing_paired SET roles_json = ?1 WHERE device_id = ?2", params![invalid, TARGET]);
        let reply = fixture.deny(&args, Some(&bearer), "", 500, 1);
        assert_eq!(reply.body["error"], "stored pairing authority is invalid");
        secrets.assert_redacted(&reply);
    }
    fixture.set("UPDATE device_pairing_paired SET roles_json = ?1 WHERE device_id = ?2",
        params![original.device(TARGET)["roles_json"].clone(), TARGET]);
    fixture.edit_tokens(TARGET, |entries| { entries.as_object_mut().unwrap().remove("operator"); });
    fixture.deny(&args, Some(&bearer), "", 400, 1);
    fixture.set("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
        params![original.device(TARGET)["tokens_json"].clone(), TARGET]);
    fixture.edit_tokens(TARGET, |entries| entries["operator"]["token"] = json!(""));
    let reply = fixture.deny(&args, Some(&bearer), "", 500, 1);
    secrets.assert_redacted(&reply);
    fixture.set("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
        params![original.device(TARGET)["tokens_json"].clone(), TARGET]);
    fixture.set("UPDATE config_machine_state SET value_json = 'drt-private-revision' WHERE state_key = 'devicePairing.revision'", []);
    let reply = fixture.deny(&args, Some(&bearer), "x-sand-request-id: drt-invalid-revision\r\n", 500, 1);
    secrets.assert_redacted(&reply);
    assert!(!reply.raw.contains("drt-private-revision"));
    let revision = original.config.iter().find(|row| text(row, "state_key") == "devicePairing.revision").unwrap();
    fixture.set("UPDATE config_machine_state SET value_json = ?1 WHERE state_key = 'devicePairing.revision'", params![revision["value_json"].clone()]);
    assert_eq!(fixture.observe(), original, "explicit fixture repair restores all prior raw values");
    fixture.reopen();
    assert_eq!(fixture.observe(), original, "all refused operations remain absent after reopen");
}

#[test]
fn token_revoke_http_pairing_permission_and_common_role_scope_guards_remain_distinct() {
    for (role, scopes, bound) in [("operator", vec!["operator.write"], 1), ("operator", vec!["operator.read"], 1),
        ("operator", vec![], 1), ("operator", vec!["operator.sessions.write"], 1),
        ("observer", vec!["observer.read"], 1), ("node", vec!["node.invoke"], 0)] {
        let fixture = Fixture::seeded(&[role], &scopes);
        let bearer = format!("Bearer {}", fixture.observe().token(CALLER, role));
        let args = json!({"deviceId": CALLER, "role": "operator"}).to_string();
        for headers in ["", "x-sand-scope: operator.admin\r\n"] {
            fixture.deny(&args, Some(&bearer), headers, 403, bound);
        }
        fixture.deny(&args, Some(&bearer), "x-sand-role: different-role\r\n", 403, 0);
        fixture.deny(&args, Some(&bearer), "x-sand-scope: operator.read\r\n", 403, 0);
    }
    for scope in ["operator.pairing", "operator.admin"] {
        let fixture = Fixture::seeded(&["operator"], &[scope]);
        let before = fixture.observe();
        let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
        let counts = fixture.observed.counts();
        let reply = fixture.revoke(&json!({"deviceId": CALLER, "role": "operator"}), Some(&bearer), "");
        assert_bound_once(&fixture, counts, true);
        fixture.assert_retirement(&before, &reply, CALLER, "operator");
    }
}

#[test]
fn token_revoke_http_failed_credentials_and_listener_guards_never_dispatch_as_native() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let replaced = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER, "role": "operator"})).expect("explicit native rotation");
        let retired = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_token_revoke(&json!({"deviceId": CALLER, "role": "operator"})).expect("selected retirement");
        fixture.pair("drt-removed", &["operator"], &["operator.pairing"]);
        let removed = fixture.observe().token("drt-removed", "operator");
        fixture.owner().handle_pair_revoke(&json!({"deviceId": "drt-removed"})).expect("whole-pair control");
        for authorization in ["Bearer unknown", "Bearer", "Bearer ", "", "Basic ignored",
            &format!("Bearer {replaced}"), &format!("Bearer {retired}"), &format!("Bearer {removed}")] {
            for body in [json!({"deviceId": TARGET, "role": "operator"}).to_string(), "{".into()] {
                fixture.deny(&body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n", 401, 0);
            }
            let counts = fixture.observed.counts();
            let ordinary = fixture.http("POST", "/api/getTranscript", "{}", Some(authorization), "");
            assert_eq!(ordinary.status, 401);
            assert_eq!(fixture.observed.counts(), counts, "failed explicit credentials never become native absence");
        }
        let args = json!({"deviceId": TARGET, "role": "operator"});
        if configured { fixture.deny(&args.to_string(), None, "", 401, 0); }
        let native = fixture.native();
        fixture.deny(&args.to_string(), native.as_deref(), "Origin: https://untrusted.invalid\r\n", 403, 0);
        fixture.deny(&args.to_string(), native.as_deref(), "x-sand-pairing-revision: stale-drt\r\n", 401, 0);
        fixture.deny(&args.to_string(), native.as_deref(), "x-sand-device-timestamp: 1\r\nx-sand-device-nonce: drt-expired\r\n", 401, 0);
        if !configured {
            fixture.deny(&args.to_string(), native.as_deref(), "Host: untrusted.invalid\r\n", 403, 0);
        }
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_secs();
        let nonce = uuid::Uuid::new_v4();
        let replay_headers = format!("x-sand-device-timestamp: {timestamp}\r\nx-sand-device-nonce: {nonce}\r\n");
        assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", native.as_deref(), &replay_headers).status, 200);
        fixture.deny(&args.to_string(), native.as_deref(), &replay_headers, 401, 0);
        let before = fixture.observe();
        let counts = fixture.observed.counts();
        let reply = fixture.revoke(&args, native.as_deref(), "");
        assert_bound_once(&fixture, counts, false);
        fixture.assert_retirement(&before, &reply, TARGET, "operator");
        fixture.pair("drt-live", &["operator"], &["operator.pairing"]);
        let before = fixture.observe();
        let reply = fixture.revoke(&json!({"deviceId": "drt-live", "role": "operator"}),
            Some(&format!("Bearer {}", before.token("drt-live", "operator"))), "");
        fixture.assert_retirement(&before, &reply, "drt-live", "operator");
        let before_public = fixture.observe();
        let pending = fixture.http("POST", "/api/device.pair.request", &json!({
            "deviceId": "drt-public", "publicKey": "drt-public-key", "scopes": ["operator.read"],
        }).to_string(), Some("Basic deliberately-public"), "");
        assert_eq!(pending.status, 200);
        assert_eq!(pending.body["status"], "pending");
        let status = fixture.http("POST", "/api/device.pair.status", &json!({"requestId": pending.body["requestId"]}).to_string(), None, "");
        assert_eq!(status.status, 200);
        assert_eq!(status.body["status"], "pending");
        assert_eq!(fixture.http("GET", "/health", "", Some("Bearer unknown"), "").status, 200);
        let payload = json!({"gatewayUrl": "https://fixture.invalid", "bootstrapToken": "drt-public-redemption"});
        let expires = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_millis() as i64 + 60_000;
        let code = fixture.owner().register_device_pairing_join_code(&payload, expires).expect("real public join code");
        let joined = fixture.http("GET", &format!("/j/{code}"), "", None, "");
        assert_eq!(joined.status, 200);
        assert_eq!(joined.body, payload);
        assert_eq!(fixture.http("GET", &format!("/j/{code}"), "", None, "").status, 404);
        assert_eq!(fixture.observe().paired, before_public.paired);
        assert_eq!(fixture.observe().config, before_public.config);
    }
    let fixture = Fixture::new(false, "0.0.0.0", false);
    fixture.seed(&["operator"], &["operator.pairing"]);
    fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), None, "", 401, 0);
    let before = fixture.observe();
    let reply = fixture.revoke(&json!({"deviceId": CALLER, "role": "operator"}),
        Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
    fixture.assert_retirement(&before, &reply, CALLER, "operator");
}

#[test]
fn token_revoke_http_default_hook_fails_closed_for_paired_and_native() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", true);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let before = fixture.observe();
        let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
        let native = fixture.native();
        for authorization in [Some(bearer.as_str()), native.as_deref()] {
            let reply = fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), authorization, "", 403, 0);
            assert_eq!(reply.body["error"], "forbidden: device token revocation is not supported by this gateway adapter");
        }
        let counts = fixture.observed.counts();
        let ordinary = fixture.http("POST", "/api/getTranscript", "{}", native.as_deref(), "");
        assert_eq!(ordinary.status, 200);
        assert_eq!(ordinary.body, json!({"ordinaryAuthenticationProbe": true}));
        assert_eq!(fixture.observed.counts(), Counts { generic: counts.generic + 1, ..counts });
        assert_eq!(fixture.observe(), before);
    }
}

#[test]
fn token_revoke_http_rechecks_current_authority_after_authentication_with_controls() {
    for change in ["unchanged", "unrelated", "retire", "rotate", "remove", "withdraw", "malformed-grant", "malformed-baseline"] {
        let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
        let before = fixture.observe();
        let secrets = before.secrets();
        let token = before.token(CALLER, "operator");
        let counts = fixture.observed.counts();
        let gate = fixture.gate(REVOKE, 1);
        let pending = fixture.spawn(&gate, REVOKE, json!({"deviceId": TARGET, "role": "operator"}), &token);
        gate.wait(1);
        assert_bound_once(&fixture, counts, true);
        assert_eq!(fixture.observe(), before, "genuine authentication precedes the independent mutation");
        match change {
            "unchanged" => {}
            "unrelated" => { fixture.owner().handle_token_revoke(&json!({"deviceId": OTHER, "role": "operator"})).expect("unrelated retirement"); }
            "retire" => { fixture.owner().handle_token_revoke(&json!({"deviceId": CALLER, "role": "operator"})).expect("caller retirement"); }
            "rotate" => { fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER, "role": "operator"})).expect("caller replacement"); }
            "remove" => { fixture.owner().handle_pair_revoke(&json!({"deviceId": CALLER})).expect("whole caller removal"); }
            "withdraw" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[\"node.invoke\"]' WHERE device_id = ?1", params![CALLER]),
            "malformed-grant" => fixture.edit_tokens(CALLER, |entries| entries["operator"]["scopes"] = json!(["operator.admin", 7])),
            "malformed-baseline" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = 'drt-private-caller-baseline' WHERE device_id = ?1", params![CALLER]),
            _ => unreachable!(),
        }
        let after_change = fixture.observe();
        gate.release();
        let reply = pending.finish();
        assert_bound_once(&fixture, counts, true);
        if matches!(change, "unchanged" | "unrelated") {
            fixture.assert_retirement(&after_change, &reply, TARGET, "operator");
        } else {
            let expected = if change.starts_with("malformed") { 500 } else { 401 };
            assert_refused(&reply, expected, &after_change);
            if expected == 500 { assert_eq!(reply.body["error"], "stored pairing authority is invalid"); }
            assert_eq!(fixture.observe(), after_change);
            assert_eq!(after_change.device(TARGET), before.device(TARGET), "stale caller cannot affect target state");
        }
        secrets.assert_redacted(&reply);
        assert!(!reply.raw.contains("drt-private-caller-baseline"));
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
    }
    // A node proof is rejected by preserved HTTP guards before this hook.
    // Observer proofs reach atomic owner admission and can test its ordering.
    for (change, expected) in [("unchanged", 403), ("unrelated", 403), ("withdraw", 401),
        ("partial-approval", 401), ("malformed-grant", 500), ("malformed-baseline", 500)] {
        let mut fixture = Fixture::seeded(&["observer"], &["observer.read", "observer.listen"]);
        let before = fixture.observe();
        let secrets = before.secrets();
        let token = before.token(CALLER, "observer");
        let counts = fixture.observed.counts();
        let gate = fixture.gate(REVOKE, 1);
        let pending = fixture.spawn(&gate, REVOKE, json!({"deviceId": TARGET, "role": "operator"}), &token);
        gate.wait(1);
        assert_bound_once(&fixture, counts, true);
        assert_eq!(fixture.observe(), before);
        match change {
            "unchanged" => {}
            "unrelated" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[]' WHERE device_id = ?1", params![TARGET]),
            "withdraw" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[]' WHERE device_id = ?1", params![CALLER]),
            "partial-approval" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[\"observer.read\"]' WHERE device_id = ?1", params![CALLER]),
            "malformed-grant" => fixture.edit_tokens(CALLER, |entries| entries["observer"]["scopes"] = json!(["observer.read", false])),
            "malformed-baseline" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = ?1 WHERE device_id = ?2", params![SqlValue::Blob(vec![0xff, 8]), CALLER]),
            _ => unreachable!(),
        }
        let after_change = fixture.observe();
        gate.release();
        let reply = pending.finish();
        assert_refused(&reply, expected, &after_change);
        if expected == 500 { assert_eq!(reply.body["error"], "stored pairing authority is invalid"); }
        secrets.assert_redacted(&reply);
        assert_bound_once(&fixture, counts, true);
        assert_eq!(fixture.observe(), after_change);
        assert_eq!(after_change.config, before.config, "controlled authority changes did not create a revision");
        fixture.set("UPDATE device_pairing_paired SET tokens_json = ?1, approved_scopes_json = ?2 WHERE device_id = ?3",
            params![before.device(CALLER)["tokens_json"].clone(), before.device(CALLER)["approved_scopes_json"].clone(), CALLER]);
        fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&format!("Bearer {token}")), "", 403, 1);
    }
}

#[test]
fn token_revoke_http_two_self_admissions_and_rotation_races_have_serial_outcomes() {
    let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "node.invoke"]);
    let before = fixture.observe();
    let token = before.token(CALLER, "operator");
    let counts = fixture.observed.counts();
    let gate = fixture.gate(REVOKE, 2);
    let first = fixture.spawn(&gate, REVOKE, json!({"deviceId": CALLER, "role": "operator"}), &token);
    let second = fixture.spawn(&gate, REVOKE, json!({"deviceId": CALLER, "role": "operator"}), &token);
    gate.wait(2);
    assert_eq!(fixture.observe(), before);
    assert_eq!(fixture.observed.counts(), Counts { bound: counts.bound + 2, recognized: counts.recognized + 2, ..counts });
    gate.release();
    let replies = [first.finish(), second.finish()];
    let mut statuses = [replies[0].status, replies[1].status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 401]);
    fixture.assert_retirement(&before, replies.iter().find(|reply| reply.status == 200).unwrap(), CALLER, "operator");
    let committed = fixture.observe();
    assert_refused(replies.iter().find(|reply| reply.status == 401).unwrap(), 401, &committed);
    assert_eq!(fixture.observed.successes.lock().expect("reports").len(), 1);
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);

    for first_method in [ROTATE, REVOKE] {
        let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "operator.write", "node.invoke"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "operator");
        let node = before.token(CALLER, "node");
        let later_method = if first_method == ROTATE { REVOKE } else { ROTATE };
        let gate = fixture.gate(later_method, 1);
        let pending = fixture.spawn(&gate, later_method, json!({"deviceId": CALLER, "role": "operator"}), &token);
        gate.wait(1);
        assert_eq!(fixture.observe(), before, "the losing operation already holds a genuine original proof");
        let first = fixture.http("POST", &format!("/api/{first_method}"),
            &json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&format!("Bearer {token}")), "");
        assert_eq!(first.status, 200, "{}", first.raw);
        if first_method == REVOKE {
            fixture.assert_retirement(&before, &first, CALLER, "operator");
        } else {
            assert_eq!(fixture.observe().token(CALLER, "operator"), first.body["token"].as_str().expect("paired self-rotation receipt"));
            assert_ne!(fixture.observe().token(CALLER, "operator"), token);
            assert!(fixture.observe().tokens(CALLER)["operator"].get("revokedAtMs").is_none());
        }
        let committed = fixture.observe();
        gate.release();
        let losing = pending.finish();
        assert_refused(&losing, 401, &committed);
        before.secrets().assert_redacted(&losing);
        assert_eq!(fixture.observe(), committed, "no replacement is issued after DRT and no successor is retired by the stale proof");
        assert_eq!(committed.revision(), before.revision() + 1);
        assert_eq!(committed.token(CALLER, "node"), node);
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        assert!(fixture.owner().authenticate_device_token(&token).expect("old raw bearer").is_none());
        assert!(fixture.owner().authenticate_device_token(&node).expect("unaffected role").is_some());
        if first_method == ROTATE {
            assert!(fixture.owner().authenticate_device_token(&committed.token(CALLER, "operator")).expect("explicit successor").is_some());
        }
    }
}

#[test]
fn token_revoke_http_fresh_marker_rule_denies_retired_bearers_and_preserves_live_controls() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator", "node"], &["operator.read", "node.invoke"]);
        let original = fixture.observe();
        let selected = original.token(CALLER, "operator");
        let node = original.token(CALLER, "node");
        let other = original.token(OTHER, "operator");
        let native = fixture.native();
        let variants = [
            (None, true), (Some(json!(null)), true), (Some(json!(0)), true),
            (Some(json!(1)), false), (Some(json!(-1)), false), (Some(json!(0.0)), false),
            (Some(json!(1.5)), false), (Some(json!("0")), false), (Some(json!("retired")), false),
            (Some(json!(false)), false), (Some(json!(true)), false), (Some(json!([])), false),
            (Some(json!([0])), false), (Some(json!({})), false), (Some(json!({"at": 0})), false),
        ];
        for (marker, active) in variants {
            fixture.edit_tokens(CALLER, |entries| {
                let entry = entries["operator"].as_object_mut().unwrap();
                match marker { Some(value) => { entry.insert("revokedAtMs".into(), value); }, None => { entry.remove("revokedAtMs"); } }
            });
            let state = fixture.observe();
            let counts = fixture.observed.counts();
            let reply = fixture.http("POST", "/api/getTranscript", "{}", Some(&format!("Bearer {selected}")), "");
            assert_eq!(reply.status, if active { 200 } else { 401 }, "{}", reply.raw);
            if active { assert_eq!(reply.body, json!({"ordinaryAuthenticationProbe": true})); }
            assert_eq!(fixture.observed.counts(), Counts {
                generic: counts.generic + usize::from(active), recognized: counts.recognized + usize::from(active), ..counts
            });
            for authorization in [Some(format!("Bearer {node}")), Some(format!("Bearer {other}")), native.clone()] {
                let control = fixture.http("POST", "/api/getTranscript", "{}", authorization.as_deref(), "");
                assert_eq!(control.status, 200, "unselected role/device and genuine native controls stay live");
                assert_eq!(control.body, json!({"ordinaryAuthenticationProbe": true}));
            }
            assert_eq!(fixture.observe(), state, "authentication and ordinary read admission never rewrite timestamps or markers");
        }
    }
}
