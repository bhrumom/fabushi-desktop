//! DTR-v1: real HTTP bound rotation, secret delivery and durable currentness.
//! Spec: projects/openclaw-rust/docs/specs/device-token-rotation-wire.md.
//! The adapter delegates every policy decision to DevicePairingOwner. Its
//! ordinary-route probe proves authentication, not shipping transcript behavior.
//! These contracts do not prove App wiring or physical-device cancellation.
//! Execute this complete target only in exact-H GitHub Actions.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
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

const WIRE: &str = "device.token.rotate";
const COMMITTED: &str = "dtr-after-commit";
const ROTATE: &str = "device.pair.rotateToken";
const CALLER: &str = "dtr-http-caller";
const TARGET: &str = "dtr-http-target";
const OTHER: &str = "dtr-http-other";
const SECRET: &str = "dtr-http-native-secret";
const WAIT: Duration = Duration::from_secs(15);
const IO_WAIT: Duration = Duration::from_secs(35);
type Row = BTreeMap<String, SqlValue>;

#[derive(Clone, PartialEq)]
struct Snapshot { paired: Vec<Row>, pending: Vec<Row>, config: Vec<Row> }

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Snapshot(<redacted complete SQL rows>)")
    }
}

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
    fn assert_text_redacted(&self, raw: &str) {
        for token in &self.0 {
            assert!(!raw.contains(token), "response disclosed a fixture credential");
            let digest = Sha256::digest(token.as_bytes());
            for fingerprint in [
                format!("{digest:x}"), format!("{digest:X}"), format!("{:?}", digest.to_vec()),
                base64::engine::general_purpose::STANDARD.encode(&digest),
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest),
            ] {
                assert!(!raw.contains(&fingerprint), "response disclosed sealed credential evidence");
            }
        }
        assert!(!raw.contains(SECRET));
    }

    fn assert_reports_redacted(&self, observed: &Observations) {
        for reports in [&observed.successes, &observed.errors] {
            for report in reports.lock().expect("command reports").iter() {
                self.assert_text_redacted(&format!("{report:?}"));
            }
        }
    }

    fn assert_redacted(&self, reply: &Reply) {
        self.assert_text_redacted(&reply.raw);
        for field in ["tokens", "fingerprint", "revision"] {
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
struct Counts { generic: usize, bound: usize, recognized: usize, rotations: usize, revocations: usize }

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
    revocations: AtomicUsize,
    admissions: Mutex<Vec<Admission>>,
    successes: Mutex<Vec<GatewayCommandReport>>,
    errors: Mutex<Vec<GatewayCommandReport>>,
}

impl Observations {
    fn counts(&self) -> Counts {
        Counts {
            generic: self.generic.load(Ordering::SeqCst), bound: self.bound.load(Ordering::SeqCst),
            recognized: self.recognized.load(Ordering::SeqCst), rotations: self.rotations.load(Ordering::SeqCst),
            revocations: self.revocations.load(Ordering::SeqCst),
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
        self.entered.try_send(()).map_err(|_| GatewayCommandError::Internal("DTR fixture arrival failed".into()))?;
        let released = self.released.lock().expect("gate lock");
        let (released, _) = self.changed.wait_timeout_while(released, WAIT, |value| !*value)
            .expect("bounded authenticated barrier");
        if !*released {
            return Err(GatewayCommandError::Internal("DTR fixture barrier expired".into()));
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

    fn rotate_device_token(
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
        self.pause(WIRE)?;
        let result = self.owner().rotate_token_with_authority(&args, authorization, scope_ceiling);
        if result.is_ok() { self.pause(COMMITTED)?; }
        result
    }

    fn revoke_device_token(
        &self,
        args: Value,
        authorization: Option<&DeviceTokenAuthorization>,
        scope_ceiling: Option<&str>,
        _context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.observed.revocations.fetch_add(1, Ordering::SeqCst);
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
    // Inherit the fixed fail-closed DTR hook, including for native requests.
}

struct Reply { status: u16, body: Value, raw: String }

impl std::fmt::Debug for Reply {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Reply").field("status", &self.status).field("body", &"<redacted>").finish()
    }
}

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
            // Every work closure has bounded socket/deadline I/O. Join the
            // owned client even during unwinding; do not detach it and let the
            // fixture remove its directory while a client is still running.
            let _ = self.thread.take().expect("owned client").join();
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
        let root = std::env::temp_dir().join(format!("fabushi-dtr-http-{}", uuid::Uuid::new_v4()));
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
            "deviceId": device, "publicKey": format!("dtr-key-{device}"), "displayName": device,
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
                "deviceId": device, "publicKey": format!("dtr-repair-{device}"),
                "roles": ["operator"], "scopes": ["operator.read"],
            })).expect("pending repair preservation witness");
            self.set("UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2 WHERE device_id = ?3",
                params![" {\"generation\":7,\"commands\":[\"system.run\"]} ", " {\"generation\":8} ", device]);
            self.edit_tokens(device, |entries| {
                for entry in entries.as_object_mut().expect("seed role map").values_mut() {
                    entry["dtrUnknownMetadata"] = json!({"retain": [1, "sentinel", null]});
                    entry["createdAtMs"] = json!(11);
                    entry["lastUsedAtMs"] = json!(17);
                    entry["issuer"] = json!({"kind": "shared-gateway-auth", "generation": "fixture-generation"});
                }
            });
        }
        self.set("INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES ('dtr.unrelated', '\"preserve\"', 7)", []);
    }

    fn http(&self, verb: &str, path: &str, body: &str, authorization: Option<&str>, headers: &str) -> Reply {
        request(self.port(), verb, path, body, authorization, headers).expect("actual TCP response")
    }

    fn rotate(&self, args: &Value, authorization: Option<&str>, headers: &str) -> Reply {
        self.http("POST", "/api/device.token.rotate", &args.to_string(), authorization, headers)
    }

    fn deny(&self, body: &str, authorization: Option<&str>, headers: &str, status: u16, bound: usize) -> Reply {
        let before = self.observe();
        let counts = self.observed.counts();
        let reply = self.http("POST", "/api/device.token.rotate", body, authorization, headers);
        assert_refused(&reply, status, &before);
        assert_eq!(self.observe(), before, "refusal preserves every SQL row and revision byte/type");
        let after = self.observed.counts();
        assert_eq!(after.generic, counts.generic, "no DTR fallback into generic native dispatch");
        assert_eq!(after.bound, counts.bound + bound);
        assert_eq!(after.rotations, counts.rotations);
        reply
    }

    fn assert_rotation(&self, before: &Snapshot, reply: &Reply, device: &str, role: &str, scopes: Value, delivery: bool) -> String {
        assert_eq!(reply.status, 200, "rotation response status");
        let timestamp = reply.body["rotatedAtMs"].as_i64()
            .filter(|value| *value > 0 && *value <= 9_007_199_254_740_991)
            .expect("positive safe integer committed timestamp");
        let after = self.observe();
        let token = after.token(device, role);
        assert!(!token.is_empty());
        if let Some(old) = before.tokens(device)[role].get("token").and_then(Value::as_str) {
            assert!(token != old, "fresh selected credential differs without printing either secret");
            assert!(self.owner().authenticate_device_token(old).expect("old bearer lookup").is_none());
        }
        let mut expected = json!({"deviceId": device, "role": role, "scopes": scopes,
            "rotatedAtMs": timestamp, "tokenDelivery": if delivery { "in-band" } else { "withheld-cross-device" }});
        if delivery { expected["token"] = json!(token); }
        assert!(reply.body == expected, "exact closed identity/grant/time/delivery response, without secret diagnostics");
        let mut expected_tokens = before.tokens(device);
        if expected_tokens.get(role).is_none() { expected_tokens[role] = json!({}); }
        let entry = expected_tokens[role].as_object_mut().expect("selected old object or new entry");
        if entry.get("createdAtMs").is_none_or(Value::is_null) { entry.insert("createdAtMs".into(), json!(timestamp)); }
        entry.insert("token".into(), json!(token));
        entry.insert("role".into(), json!(role));
        entry.insert("scopes".into(), scopes);
        entry.insert("rotatedAtMs".into(), json!(timestamp));
        entry.remove("revokedAtMs");
        assert!(after.tokens(device) == expected_tokens, "only selected generation fields change; all lifecycle/issuer/unknown metadata survives");
        let mut expected_rows = before.paired.clone();
        let row = expected_rows.iter_mut().find(|row| text(row, "device_id") == device).expect("target row");
        row.insert("tokens_json".into(), after.device(device)["tokens_json"].clone());
        if role == "node" {
            row.insert("node_surface_json".into(), SqlValue::Null);
            row.insert("pending_node_surface_json".into(), SqlValue::Null);
        }
        assert!(after.paired == expected_rows, "all unselected entries, rows and surface bytes remain unchanged");
        assert!(after.pending == before.pending, "pending requests remain unchanged");
        assert_eq!(after.revision(), before.revision() + 1);
        let unrelated = |state: &Snapshot| state.config.iter()
            .filter(|row| text(row, "state_key") != "devicePairing.revision").cloned().collect::<Vec<_>>();
        assert!(unrelated(&after) == unrelated(before), "unrelated configuration remains unchanged");
        assert!(self.owner().authenticate_device_token(&token).expect("fresh successor lookup").is_some());
        before.assert_redacted(reply);
        if !delivery { after.assert_redacted(reply); }
        for report in self.observed.successes.lock().expect("success reports").iter() {
            assert!(!format!("{report:?}").contains(&token), "common report must not carry the replacement");
        }
        token
    }

    fn assert_primary_rotation(
        &self, before: &Snapshot, reply: &Reply, device: &str, role: &str,
        scopes: Value, expected_roles: Value,
    ) -> String {
        let after = self.observe();
        let raw_roles = after.device(device)["roles_json"].clone();
        let roles: Value = serde_json::from_str(text(after.device(device), "roles_json"))
            .expect("strict committed role array");
        assert!(roles == expected_roles, "only the selected primary role is appended; original strings and order survive");
        let mut expected = before.clone();
        expected.paired.iter_mut().find(|row| text(row, "device_id") == device).expect("target")
            .insert("roles_json".into(), raw_roles);
        // Retain the complete original rotation oracle. Its only permitted extra
        // column is the independently asserted selected-role materialization.
        self.assert_rotation(&expected, reply, device, role, scopes, false)
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
            eprintln!("DTR fixture retained after unacknowledged teardown: {}", self.root.display());
            assert!(thread::panicking(), "bounded connection/API teardown failed");
        }
    }
}

fn assert_refused(reply: &Reply, expected: u16, before: &Snapshot) {
    assert_eq!(reply.status, expected, "refusal status; response body redacted");
    assert!(reply.body["error"].as_str().is_some_and(|error| !error.is_empty()));
    assert!(!reply.raw.contains("DTR fixture"), "fixture failure cannot satisfy a policy-refusal oracle");
    assert!(reply.body.get("token").is_none() && reply.body.get("tokenDelivery").is_none() && reply.body.get("scopes").is_none());
    before.assert_redacted(reply);
}

fn assert_bound_once(fixture: &Fixture, before: Counts, paired: bool) {
    assert_eq!(fixture.observed.counts(), Counts {
        bound: before.bound + 1, recognized: before.recognized + usize::from(paired), ..before
    });
}

#[test]
fn token_rotate_wire_http_self_delivers_committed_successor_with_context_and_reopen() {
    for configured in [false, true] {
        let mut fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator", "node"], &["operator.pairing", "node.invoke"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "operator");
        let node = before.token(CALLER, "node");
        let counts = fixture.observed.counts();
        let reply = fixture.rotate(&json!({"deviceId": format!(" {CALLER} "), "role": " operator "}),
            Some(&format!("Bearer {token}")),
            "x-sand-role: operator\r\nx-sand-scope: operator.pairing\r\nx-sand-request-id: dtr-self\r\ntraceparent: 00-0123456789abcdef0123456789abcdef-0123456789abcdef-01\r\n");
        assert_bound_once(&fixture, counts, true);
        let successor = fixture.assert_rotation(&before, &reply, CALLER, "operator", json!(["operator.pairing"]), true);
        assert!(fixture.observe().token(CALLER, "node") == node);
        {
            let admissions = fixture.observed.admissions.lock().expect("admissions");
            assert_eq!(admissions.len(), 1);
            let admission = &admissions[0];
            assert!(admission.paired && admission.context.authenticated);
            assert_eq!(admission.device.as_deref(), Some(CALLER));
            assert_eq!(admission.role.as_deref(), Some("operator"));
            assert_eq!(admission.ceiling.as_deref(), Some("operator.pairing"));
            assert_eq!(admission.context.request_id.as_deref(), Some("dtr-self"));
            assert_eq!(admission.context.trace_id.as_deref(), Some("0123456789abcdef0123456789abcdef"));
            assert_eq!(admission.context.span_id.as_deref(), Some("0123456789abcdef"));
            let reports = fixture.observed.successes.lock().expect("reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].method, WIRE);
            assert_eq!(reports[0].status, 200);
            assert_eq!(reports[0].request_id, admission.context.request_id);
            assert_eq!(reports[0].traceparent, admission.context.traceparent);
            assert!(fixture.observed.errors.lock().expect("errors").is_empty());
        }
        let committed = fixture.observe();
        fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&format!("Bearer {token}")), "", 401, 0);
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        for (credential, status) in [(&token, 401), (&successor, 200), (&node, 200)] {
            assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", Some(&format!("Bearer {credential}")), "").status, status);
        }
        assert_eq!(fixture.observe(), committed, "fresh admission performs no pairing writes");
    }
}

#[test]
fn token_rotate_wire_http_native_delivery_and_singular_ceiling_bind_with_and_without_secret() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let native = fixture.native();
        for ceiling in [None, Some("operator.admin"), Some("operator.pairing")] {
            let header = ceiling.map(|scope| format!("x-sand-scope: {scope}\r\n")).unwrap_or_default();
            let before = fixture.observe();
            let counts = fixture.observed.counts();
            let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator"}), native.as_deref(),
                &format!("x-sand-role: operator\r\nx-sand-device-id: {TARGET}\r\n{header}"));
            assert_bound_once(&fixture, counts, false);
            fixture.assert_rotation(&before, &reply, TARGET, "operator", json!(["operator.pairing"]), false);
            let admission = fixture.observed.admissions.lock().expect("admissions").last().unwrap().clone();
            assert!(!admission.paired && admission.device.is_none() && admission.role.is_none());
            assert_eq!(admission.ceiling.as_deref(), ceiling);
            assert!(admission.context.authenticated);
        }
        for (ceiling, bound) in [("operator.write", 1), ("operator.read", 0), ("", 1),
            ("operator.pairing operator.admin", 1), ("operator.pairing,operator.admin", 1), ("node.invoke", 1)] {
            fixture.deny(&json!({"deviceId": TARGET, "role": "operator", "scopes": []}).to_string(), native.as_deref(),
                &format!("x-sand-scope: {ceiling}\r\n"), 403, bound);
        }
        for role in ["node", "observer", "Operator", ""] {
            fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), native.as_deref(),
                &format!("x-sand-role: {role}\r\n"), 403, 0);
        }
        fixture.deny(&json!({"deviceId": TARGET, "role": "node", "scopes": []}).to_string(), native.as_deref(),
            "x-sand-scope: operator.pairing\r\n", 403, 1);
        for (key, claim) in [("callerDeviceId", json!(TARGET)), ("crossDevice", json!(false)),
            ("tokenDelivery", json!("in-band")), ("isAdmin", json!(true))] {
            let mut args = json!({"deviceId": TARGET, "role": "operator"});
            args[key] = claim;
            fixture.deny(&args.to_string(), native.as_deref(), "", 400, 1);
        }
        let before = fixture.observe();
        let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": []}), native.as_deref(),
            "x-sand-scope: operator.pairing\r\n");
        fixture.assert_rotation(&before, &reply, TARGET, "operator", json!([]), false);
    }
}

#[test]
fn token_rotate_wire_http_addressing_hides_targets_and_admin_delivery_is_device_bound() {
    let fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "node.invoke"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    let existing = fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&bearer), "", 403, 1);
    let missing = fixture.deny(&json!({"deviceId": "dtr-absent", "role": "operator"}).to_string(), Some(&bearer), "", 403, 1);
    assert_eq!(existing.body, missing.body, "cross-device refusal precedes lookup");
    let existing = fixture.deny(&json!({"deviceId": CALLER, "role": "node"}).to_string(), Some(&bearer), "", 403, 1);
    let missing = fixture.deny(&json!({"deviceId": CALLER, "role": "observer"}).to_string(), Some(&bearer), "", 403, 1);
    assert_eq!(existing.body, missing.body, "nonoperator refusal precedes lookup");

    let fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    for (device, role, scopes, delivers) in [
        (TARGET, "operator", json!(["operator.pairing"]), false),
        (TARGET, "node", json!(["node.invoke"]), false),
        (CALLER, "node", json!(["node.invoke"]), true),
    ] {
        let before = fixture.observe();
        let counts = fixture.observed.counts();
        let reply = fixture.rotate(&json!({"deviceId": device, "role": role}), Some(&bearer), "");
        assert_bound_once(&fixture, counts, true);
        fixture.assert_rotation(&before, &reply, device, role, scopes, delivers);
    }
    fixture.deny(&json!({"deviceId": TARGET, "role": "operator", "scopes": []}).to_string(), Some(&bearer),
        "x-sand-scope: operator.pairing\r\n", 403, 1);
    fixture.deny(&json!({"deviceId": CALLER, "role": "node", "scopes": []}).to_string(), Some(&bearer),
        "x-sand-scope: operator.pairing\r\n", 403, 1);
    fixture.deny(&json!({"deviceId": "dtr-authorized-missing", "role": "operator"}).to_string(), Some(&bearer), "", 400, 1);
}

#[test]
fn token_rotate_wire_http_scope_omission_empty_implied_and_legacy_repair_use_resulting_grant() {
    let fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    fixture.set("UPDATE device_pairing_paired SET scopes_json = '[\"operator.admin\",\"node.invoke\"]', approved_scopes_json = '[\"operator.admin\",\"node.invoke\"]' WHERE device_id = ?1", params![TARGET]);
    // Omission preserves the narrower selected grant, never the broader baseline.
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator"}), Some(&bearer), "");
    fixture.assert_rotation(&before, &reply, TARGET, "operator", json!(["operator.pairing"]), false);
    for (requested, expected) in [
        (json!([" operator.write ", "operator.write", "\u{feff}", "\t"]), json!(["operator.read", "operator.write"])),
        (json!([]), json!([])),
    ] {
        let before = fixture.observe();
        let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": requested}), Some(&bearer), "");
        fixture.assert_rotation(&before, &reply, TARGET, "operator", expected, false);
    }
    fixture.deny(&json!({"deviceId": TARGET, "role": "operator", "scopes": ["\u{85}operator.write\u{85}"]}).to_string(), Some(&bearer), "", 403, 1);
    fixture.edit_tokens(TARGET, |entries| entries["node"]["scopes"] = json!(["operator.admin", "node.old-unsupported"]));
    fixture.deny(&json!({"deviceId": TARGET, "role": "node"}).to_string(), Some(&bearer), "", 403, 1);
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "node", "scopes": []}), Some(&bearer), "");
    fixture.assert_rotation(&before, &reply, TARGET, "node", json!([]), false);
    fixture.edit_tokens(TARGET, |entries| {
        entries["operator"]["scopes"] = json!({"privateCorruptGrant": true});
        entries["operator"]["revokedAtMs"] = json!(123);
        entries["operator"]["role"] = json!("obsolete-role");
        entries["operator"]["token"] = json!("");
    });
    fixture.deny(&json!({"deviceId": TARGET, "role": "operator"}).to_string(), Some(&bearer), "", 500, 1);
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": []}), Some(&bearer), "");
    fixture.assert_rotation(&before, &reply, TARGET, "operator", json!([]), false);
    fixture.edit_tokens(TARGET, |entries| { entries.as_object_mut().unwrap().remove("operator"); });
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": ["operator.pairing"]}), Some(&bearer), "");
    fixture.assert_rotation(&before, &reply, TARGET, "operator", json!(["operator.pairing"]), false);
}

#[test]
fn token_rotate_wire_http_closed_request_storage_refusal_and_explicit_repair_are_atomic() {
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
    for scopes in [json!(null), json!("operator.pairing"), json!({}), json!(["operator.pairing", 7]), json!([""])] {
        fixture.deny(&json!({"deviceId": TARGET, "role": "operator", "scopes": scopes}).to_string(), Some(&bearer), "", 400, 1);
    }
    for key in ["callerRole", "callerDeviceId", "authorization", "crossDevice", "revision", "token", "tokenDelivery"] {
        let mut args = json!({"deviceId": TARGET, "role": "operator"});
        args[key] = json!("forged");
        fixture.deny(&args.to_string(), Some(&bearer), "", 400, 1);
    }
    fixture.deny("{", Some(&bearer), "", 400, 0);
    fixture.deny(&json!({"deviceId": TARGET, "role": "Operator"}).to_string(), Some(&bearer), "", 400, 1);
    let args = json!({"deviceId": TARGET, "role": "operator", "scopes": []}).to_string();
    for (column, invalid) in [
        ("roles_json", SqlValue::Null),
        ("roles_json", SqlValue::Text("[7]".into())),
        ("tokens_json", SqlValue::Text("dtr-private-token-map".into())),
        ("approved_scopes_json", SqlValue::Text("null".into())),
        ("approved_scopes_json", SqlValue::Blob(vec![0xff, 7])),
    ] {
        fixture.set(&format!("UPDATE device_pairing_paired SET {column} = ?1 WHERE device_id = ?2"), params![invalid, TARGET]);
        let reports_before = fixture.observed.errors.lock().expect("reports").len();
        let reply = fixture.deny(&args, Some(&bearer), "x-sand-request-id: dtr-corruption\r\n", 500, 1);
        secrets.assert_redacted(&reply);
        assert_eq!(reply.body["error"], "stored pairing authority is invalid");
        let reports = fixture.observed.errors.lock().expect("reports");
        assert_eq!(reports.len(), reports_before + 1);
        let report = reports.last().unwrap();
        assert_eq!(report.method, WIRE);
        assert_eq!(report.status, 500);
        assert_eq!(report.request_id.as_deref(), Some("dtr-corruption"));
        assert_eq!(report.error.as_deref(), Some("stored pairing authority is invalid"));
        drop(reports);
        fixture.set(&format!("UPDATE device_pairing_paired SET {column} = ?1 WHERE device_id = ?2"), params![original.device(TARGET)[column].clone(), TARGET]);
    }
    fixture.set("UPDATE config_machine_state SET value_json = 'dtr-private-revision' WHERE state_key = 'devicePairing.revision'", []);
    let reply = fixture.deny(&args, Some(&bearer), "", 500, 1);
    secrets.assert_redacted(&reply);
    assert!(!reply.raw.contains("dtr-private-revision"));
    let revision = original.config.iter().find(|row| text(row, "state_key") == "devicePairing.revision").unwrap();
    fixture.set("UPDATE config_machine_state SET value_json = ?1 WHERE state_key = 'devicePairing.revision'", params![revision["value_json"].clone()]);
    assert_eq!(fixture.observe(), original);
    fixture.reopen();
    assert_eq!(fixture.observe(), original);
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": []}), Some(&bearer), "");
    fixture.assert_rotation(&original, &reply, TARGET, "operator", json!([]), false);
}

#[test]
fn token_rotate_wire_http_pairing_permission_and_existing_role_scope_guards_stay_distinct() {
    for (role, scopes, bound) in [("operator", vec!["operator.write"], 1), ("operator", vec!["operator.read"], 1),
        ("operator", vec![], 1), ("observer", vec!["observer.read"], 1), ("node", vec!["node.invoke"], 0)] {
        let fixture = Fixture::seeded(&[role], &scopes);
        let bearer = format!("Bearer {}", fixture.observe().token(CALLER, role));
        let args = json!({"deviceId": CALLER, "role": "operator", "scopes": []}).to_string();
        for headers in ["", "x-sand-scope: operator.admin\r\n"] { fixture.deny(&args, Some(&bearer), headers, 403, bound); }
        fixture.deny(&args, Some(&bearer), "x-sand-role: different-role\r\n", 403, 0);
        fixture.deny(&args, Some(&bearer), "x-sand-scope: operator.read\r\n", 403, 0);
    }
    for grant in ["operator.pairing", "operator.admin"] {
        let fixture = Fixture::seeded(&["operator"], &[grant]);
        let before = fixture.observe();
        let reply = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator", "scopes": []}),
            Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
        fixture.assert_rotation(&before, &reply, CALLER, "operator", json!([]), true);
    }
    let fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&bearer), "x-sand-scope: operator.pairing\r\n", 403, 1);
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator", "scopes": ["operator.pairing"]}), Some(&bearer), "x-sand-scope: operator.pairing\r\n");
    fixture.assert_rotation(&before, &reply, CALLER, "operator", json!(["operator.pairing"]), true);
}

#[test]
fn token_rotate_wire_http_large_distinct_scope_request_reaches_owner_and_preserves_read_only_authority() {
    let mut fixture = Fixture::seeded(&["operator"], &["operator.read"]);
    let before = fixture.observe();
    let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
    let scopes: Vec<String> = (0..100_000).map(|index| format!("operator.s{index:06}")).collect();
    let body = json!({"deviceId": CALLER, "role": "operator", "scopes": scopes}).to_string();
    assert!(body.len() < mahayana_host_runtime::gateway_server::MAX_BODY_BYTES);
    let counts = fixture.observed.counts();
    // No narrowing header: this is the actual paired read-only path through
    // closed-request parsing, not a generic write/read-header prefilter. The
    // existing I/O deadline bounds cleanup; no elapsed-time benchmark is used.
    let reply = fixture.deny(&body, Some(&bearer), "", 403, 1);
    assert_bound_once(&fixture, counts, true);
    assert_eq!(reply.body["error"], "forbidden: token rotation requires operator pairing authority");
    let admission = fixture.observed.admissions.lock().expect("admission witness").last().unwrap().clone();
    assert!(admission.paired && admission.context.authenticated);
    assert_eq!(admission.device.as_deref(), Some(CALLER));
    assert_eq!(admission.role.as_deref(), Some("operator"));
    assert!(admission.ceiling.is_none());
    fixture.reopen();
    assert_eq!(fixture.observe(), before, "large refusal survives owner/listener reopen without changing storage");
    let proof = fixture.owner().authenticate_device_token(&before.token(CALLER, "operator"))
        .expect("unchanged bearer lookup").expect("read-only credential remains recognized");
    fixture.owner().validate_current_device_authorization(&proof).expect("read-only credential remains current");
    assert_eq!(proof.scopes, ["operator.read"]);

    // A genuinely paired operator can still rotate on the same reopened owner.
    let counts = fixture.observed.counts();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "operator", "scopes": ["operator.pairing"]}),
        Some(&format!("Bearer {}", before.token(TARGET, "operator"))), "");
    assert_bound_once(&fixture, counts, true);
    let successor = fixture.assert_rotation(&before, &reply, TARGET, "operator", json!(["operator.pairing"]), true);
    let proof = fixture.owner().authenticate_device_token(&successor)
        .expect("positive successor lookup").expect("positive successor recognized");
    fixture.owner().validate_current_device_authorization(&proof).expect("positive successor current");
}

#[test]
fn token_rotate_wire_http_bad_credentials_and_common_ingress_never_become_native() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let predecessor = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_token_rotate(&json!({"deviceId": CALLER, "role": "operator"})).expect("explicit rotation");
        let retired = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_token_revoke(&json!({"deviceId": CALLER, "role": "operator"})).expect("retirement control");
        for authorization in ["Bearer unknown", "Bearer", "Bearer ", "", "Basic ignored",
            &format!("Bearer {predecessor}"), &format!("Bearer {retired}")] {
            for body in [json!({"deviceId": TARGET, "role": "operator"}).to_string(), "{".into()] {
                fixture.deny(&body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n", 401, 0);
            }
            let counts = fixture.observed.counts();
            assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", Some(authorization), "").status, 401);
            assert_eq!(fixture.observed.counts(), counts);
        }
        let args = json!({"deviceId": TARGET, "role": "operator"});
        let native = fixture.native();
        if configured { fixture.deny(&args.to_string(), None, "", 401, 0); }
        fixture.deny(&args.to_string(), native.as_deref(), "Origin: https://untrusted.invalid\r\n", 403, 0);
        fixture.deny(&args.to_string(), native.as_deref(), "x-sand-pairing-revision: stale-dtr\r\n", 401, 0);
        fixture.deny(&args.to_string(), native.as_deref(), "x-sand-device-timestamp: 1\r\nx-sand-device-nonce: dtr-expired\r\n", 401, 0);
        if !configured { fixture.deny(&args.to_string(), None, "Host: untrusted.invalid\r\n", 403, 0); }
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_secs();
        let nonce = uuid::Uuid::new_v4();
        let headers = format!("x-sand-device-timestamp: {timestamp}\r\nx-sand-device-nonce: {nonce}\r\n");
        assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", native.as_deref(), &headers).status, 200);
        fixture.deny(&args.to_string(), native.as_deref(), &headers, 401, 0);
        let before = fixture.observe();
        let reply = fixture.rotate(&args, native.as_deref(), "");
        fixture.assert_rotation(&before, &reply, TARGET, "operator", json!(["operator.pairing"]), false);
        let pending = fixture.http("POST", "/api/device.pair.request", &json!({
            "deviceId": "dtr-public", "publicKey": "dtr-public-key", "scopes": ["operator.read"],
        }).to_string(), Some("Basic deliberately-public"), "");
        assert_eq!(pending.status, 200);
        assert_eq!(pending.body["status"], "pending");
        assert_eq!(fixture.http("POST", "/api/device.pair.status", &json!({"requestId": pending.body["requestId"]}).to_string(), None, "").status, 200);
        assert_eq!(fixture.http("GET", "/health", "", Some("Bearer unknown"), "").status, 200);
        let payload = json!({"gatewayUrl": "https://fixture.invalid", "bootstrapToken": "dtr-public-redemption"});
        let expires = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_millis() as i64 + 60_000;
        let code = fixture.owner().register_device_pairing_join_code(&payload, expires).expect("public join code");
        assert!(fixture.http("GET", &format!("/j/{code}"), "", None, "").body == payload);
        assert_eq!(fixture.http("GET", &format!("/j/{code}"), "", None, "").status, 404);
    }
    let fixture = Fixture::new(false, "0.0.0.0", false);
    fixture.seed(&["operator"], &["operator.pairing"]);
    fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), None, "", 401, 0);
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator"}), Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
    fixture.assert_rotation(&before, &reply, CALLER, "operator", json!(["operator.pairing"]), true);
}

#[test]
fn token_rotate_wire_http_default_hook_fails_closed_and_dpa_keeps_its_distinct_contract() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", true);
        fixture.seed(&["operator"], &["operator.pairing"]);
        let before = fixture.observe();
        let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
        let native = fixture.native();
        for authorization in [Some(bearer.as_str()), native.as_deref()] {
            let reply = fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), authorization, "", 403, 0);
            assert_eq!(reply.body["error"], "forbidden: device token rotation is not supported by this gateway adapter");
        }
        assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", native.as_deref(), "").status, 200);
        assert_eq!(fixture.observe(), before);
    }
    let fixture = Fixture::seeded(&["operator"], &["operator.write"]);
    let before = fixture.observe();
    let counts = fixture.observed.counts();
    let reply = fixture.http("POST", "/api/device.pair.rotateToken", &json!({"deviceId": CALLER}).to_string(),
        Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
    assert_eq!(reply.status, 200);
    assert!(reply.body["token"].as_str().is_some_and(|token| token == fixture.observe().token(CALLER, "operator")));
    assert!(reply.body.get("revision").is_some(), "old DPA retains its native result");
    assert_eq!(fixture.observed.counts(), Counts { rotations: counts.rotations + 1, recognized: counts.recognized + 1, ..counts });
    assert_eq!(fixture.observe().device(CALLER)["node_surface_json"], SqlValue::Null, "DPA retains any-role invalidation");
    assert_eq!(fixture.observe().device(CALLER)["pending_node_surface_json"], SqlValue::Null);
    let fixture = Fixture::seeded(&["operator"], &["operator.pairing"]);
    let before = fixture.observe();
    let reply = fixture.http("POST", "/api/device.pair.rotateToken", &json!({"deviceId": CALLER, "role": "operator"}).to_string(),
        Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
    assert_refused(&reply, 403, &before);
    assert_eq!(fixture.observe(), before, "DTR pairing permission never widens the old DPA write requirement");
}

#[test]
fn token_rotate_wire_http_authenticated_barrier_rechecks_current_sealed_authority() {
    for change in ["unchanged", "unrelated", "retire", "rotate", "withdraw", "malformed-grant", "malformed-baseline"] {
        let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
        let before = fixture.observe();
        let secrets = before.secrets();
        let token = before.token(CALLER, "operator");
        let counts = fixture.observed.counts();
        let gate = fixture.gate(WIRE, 1);
        let pending = fixture.spawn(&gate, WIRE, json!({"deviceId": TARGET, "role": "operator"}), &token);
        gate.wait(1);
        assert_bound_once(&fixture, counts, true);
        assert_eq!(fixture.observe(), before, "barrier follows genuine ingress authentication");
        match change {
            "unchanged" => {}
            "unrelated" => { fixture.owner().handle_token_rotate(&json!({"deviceId": OTHER, "role": "operator"})).expect("unrelated commit"); }
            "retire" => { fixture.owner().handle_token_revoke(&json!({"deviceId": CALLER, "role": "operator"})).expect("caller retirement"); }
            "rotate" => { fixture.owner().handle_token_rotate(&json!({"deviceId": CALLER, "role": "operator"})).expect("caller replacement"); }
            "withdraw" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[\"node.invoke\"]' WHERE device_id = ?1", params![CALLER]),
            "malformed-grant" => fixture.edit_tokens(CALLER, |entries| entries["operator"]["scopes"] = json!(["operator.admin", 7])),
            "malformed-baseline" => fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = 'dtr-private-caller-baseline' WHERE device_id = ?1", params![CALLER]),
            _ => unreachable!(),
        }
        let competing = fixture.observe();
        gate.release();
        let reply = pending.finish();
        assert_bound_once(&fixture, counts, true);
        if matches!(change, "unchanged" | "unrelated") {
            fixture.assert_rotation(&competing, &reply, TARGET, "operator", json!(["operator.pairing"]), false);
        } else {
            assert_refused(&reply, if change.starts_with("malformed") { 500 } else { 401 }, &competing);
            assert_eq!(fixture.observe(), competing, "refusal preserves the independently committed state");
            assert!(competing.device(TARGET) == before.device(TARGET));
        }
        secrets.assert_redacted(&reply);
        assert!(!reply.raw.contains("dtr-private-caller-baseline"));
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
    }
}

#[test]
fn token_rotate_wire_http_two_self_admissions_have_one_winner_and_two_manager_rotations_commit() {
    let mut fixture = Fixture::seeded(&["operator", "node"], &["operator.pairing", "node.invoke"]);
    let before = fixture.observe();
    let token = before.token(CALLER, "operator");
    let counts = fixture.observed.counts();
    let gate = fixture.gate(WIRE, 2);
    let first = fixture.spawn(&gate, WIRE, json!({"deviceId": CALLER, "role": "operator"}), &token);
    let second = fixture.spawn(&gate, WIRE, json!({"deviceId": CALLER, "role": "operator"}), &token);
    gate.wait(2);
    assert_eq!(fixture.observe(), before);
    gate.release();
    let replies = [first.finish(), second.finish()];
    let mut statuses = [replies[0].status, replies[1].status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 401]);
    fixture.assert_rotation(&before, replies.iter().find(|reply| reply.status == 200).unwrap(), CALLER, "operator", json!(["operator.pairing"]), true);
    let committed = fixture.observe();
    assert_refused(replies.iter().find(|reply| reply.status == 401).unwrap(), 401, &committed);
    assert_eq!(fixture.observed.counts(), Counts { bound: counts.bound + 2, recognized: counts.recognized + 2, ..counts });
    assert_eq!(fixture.observed.successes.lock().expect("reports").len(), 1);
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);

    let mut fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
    let before = fixture.observe();
    let manager = before.token(CALLER, "operator");
    let gate = fixture.gate(WIRE, 2);
    let first = fixture.spawn(&gate, WIRE, json!({"deviceId": TARGET, "role": "operator"}), &manager);
    let second = fixture.spawn(&gate, WIRE, json!({"deviceId": TARGET, "role": "operator"}), &manager);
    gate.wait(2);
    assert_eq!(fixture.observe(), before);
    gate.release();
    for reply in [first.finish(), second.finish()] {
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body["tokenDelivery"], "withheld-cross-device");
        assert!(reply.body.get("token").is_none());
        before.assert_redacted(&reply);
    }
    let committed = fixture.observe();
    assert_eq!(committed.revision(), before.revision() + 2, "outside manager remains current for both serial commits");
    assert!(committed.token(TARGET, "operator") != before.token(TARGET, "operator"));
    assert!(committed.token(TARGET, "node") == before.token(TARGET, "node"));
    assert!(committed.device(CALLER) == before.device(CALLER));
    assert!(fixture.owner().authenticate_device_token(&manager).expect("manager remains current").is_some());
    assert!(fixture.owner().authenticate_device_token(&committed.token(TARGET, "operator")).expect("final generation").is_some());
    assert!(fixture.owner().authenticate_device_token(&before.token(TARGET, "operator")).expect("original generation").is_none());
    assert_eq!(fixture.observed.successes.lock().expect("reports").len(), 2);
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);
}

#[test]
fn token_rotate_wire_http_socket_closed_after_commit_never_retries_or_resurrects_the_predecessor() {
    let mut fixture = Fixture::seeded(&["operator"], &["operator.pairing"]);
    let before = fixture.observe();
    let predecessor = before.token(CALLER, "operator");
    let counts = fixture.observed.counts();
    let gate = fixture.gate(COMMITTED, 1);
    let mut socket = TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], fixture.port())), IO_WAIT).expect("owned TCP client");
    socket.set_write_timeout(Some(IO_WAIT)).expect("bounded request write");
    let body = json!({"deviceId": CALLER, "role": "operator"}).to_string();
    write!(socket, "POST /api/device.token.rotate HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {predecessor}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).expect("write one actual request");
    socket.flush().expect("flush one request");
    gate.wait(1);
    let committed = fixture.observe();
    assert_eq!(committed.revision(), before.revision() + 1, "independent observer sees the commit while the hook withholds its return");
    assert!(committed.token(CALLER, "operator") != predecessor);
    assert_bound_once(&fixture, counts, true);
    // The client closes before the adapter returns to the HTTP writer and reads
    // no response. This proves committed response loss to this client, not an
    // observed OS write error or a protocol-level confirmed-delivery signal.
    socket.shutdown(Shutdown::Both).expect("close owned client before response");
    drop(socket);
    gate.release();
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);
    assert_eq!(fixture.observed.counts().bound, counts.bound + 1, "no hidden replay after lost response");
    fixture.deny(&body, Some(&format!("Bearer {predecessor}")), "", 401, 0);
    assert_eq!(fixture.observe(), committed);
    let repaired = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator"}), None, "");
    fixture.assert_rotation(&committed, &repaired, CALLER, "operator", json!(["operator.pairing"]), false);
    assert_eq!(fixture.observed.counts().bound, counts.bound + 2, "only explicit newly authorized repair adds a generation");
}

#[test]
fn token_rotate_wire_http_issued_nel_literal_never_becomes_pairing_or_admin_at_adjacent_consumers() {
    let literal = "operator.admin\u{85}";
    let fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
    let before = fixture.observe();
    let original = before.token(CALLER, "operator");
    let receipt = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator", "scopes": [literal]}),
        Some(&format!("Bearer {original}")), "");
    let successor = fixture.assert_rotation(&before, &receipt, CALLER, "operator", json!([literal]), true);
    assert_eq!(receipt.body["scopes"], json!([literal]), "no canonical admin/read/write implications");
    let proof = fixture.owner().authenticate_device_token(&successor).expect("fresh successor authentication")
        .expect("literal credential is valid");
    assert_eq!(proof.scopes, vec![literal.to_owned()]);
    fixture.owner().validate_current_device_authorization(&proof).expect("literal grant is current, not blanket refused");
    let committed = fixture.observe();
    assert_eq!(fixture.http("POST", "/api/getTranscript", "{}", Some(&format!("Bearer {successor}")), "").status, 200);
    assert_eq!(fixture.observe(), committed);
    for method in ["device.token.revoke", ROTATE, WIRE] {
        for (device, role) in [(TARGET, "operator"), (CALLER, "node"), (TARGET, "node")] {
            let state = fixture.observe();
            let counts = fixture.observed.counts();
            let reports = fixture.observed.errors.lock().expect("reports").len();
            let successes = fixture.observed.successes.lock().expect("success reports").len();
            let reply = fixture.http("POST", &format!("/api/{method}"),
                &json!({"deviceId": device, "role": role}).to_string(), Some(&format!("Bearer {successor}")),
                "x-sand-request-id: dtr-nel-literal-refusal\r\n");
            assert_refused(&reply, 403, &state);
            before.secrets().assert_redacted(&reply);
            assert_eq!(fixture.observe(), state, "every target, selected grant, role, surface and revision remains unchanged");
            assert_eq!(fixture.observed.counts(), Counts {
                bound: counts.bound + usize::from(method == WIRE),
                rotations: counts.rotations + usize::from(method == ROTATE),
                revocations: counts.revocations + usize::from(method == "device.token.revoke"),
                recognized: counts.recognized + 1, ..counts
            }, "real bound owner hook refuses; a default adapter refusal is insufficient");
            let errors = fixture.observed.errors.lock().expect("normal error reports");
            assert_eq!(errors.len(), reports, "ordinary 403 is not a 5xx command-error report");
            drop(errors);
            assert_eq!(fixture.observed.successes.lock().expect("success reports").len(), successes,
                "refused commands never emit a success report");
            if method == WIRE {
                let admissions = fixture.observed.admissions.lock().expect("bound context witness");
                let admission = admissions.last().unwrap();
                assert!(admission.paired && admission.context.authenticated);
                assert_eq!(admission.device.as_deref(), Some(CALLER));
                assert_eq!(admission.context.request_id.as_deref(), Some("dtr-nel-literal-refusal"));
            }
            state.secrets().assert_reports_redacted(&fixture.observed);
            before.secrets().assert_reports_redacted(&fixture.observed);
        }
    }
    fixture.owner().validate_current_device_authorization(&proof).expect("refusals do not retire literal credential");
    assert_eq!(fixture.observe(), committed);

    // Genuine admin positives traverse the same three hooks. Existing DPA
    // request normalization remains an adapter, not credential permission truth.
    let fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    fixture.set("UPDATE device_pairing_paired SET approved_scopes_json = '[\"operator.admin\",\"node.invoke\"]' WHERE device_id = ?1", params![TARGET]);
    let before = fixture.observe();
    let reply = fixture.http("POST", "/api/device.token.revoke", &json!({"deviceId": TARGET, "role": "node"}).to_string(), Some(&bearer), "");
    assert_eq!(reply.status, 200);
    assert_eq!(fixture.observe().revision(), before.revision() + 1);
    assert_eq!(fixture.observed.counts().revocations, 1);
    let before = fixture.observe();
    let reply = fixture.http("POST", "/api/device.pair.rotateToken", &json!({
        "deviceId": TARGET, "role": "operator", "scopes": ["\u{85}operator.read\u{85}"],
    }).to_string(), Some(&bearer), "");
    assert_eq!(reply.status, 200, "legacy DPA request adapter still trims NEL");
    assert_eq!(fixture.observe().tokens(TARGET)["operator"]["scopes"], json!(["operator.read"]));
    assert_eq!(fixture.observe().revision(), before.revision() + 1);
    assert!(reply.body.get("revision").is_some() && reply.body.get("scopes").is_none());
    assert!(reply.body.get("token").is_none());
    let before = fixture.observe();
    let reply = fixture.http("POST", "/api/device.pair.rotateToken", &json!({
        "deviceId": TARGET, "role": "operator", "scopes": ["\u{feff}operator.read\u{feff}"],
    }).to_string(), Some(&bearer), "");
    assert_refused(&reply, 403, &before);
    assert_eq!(fixture.observe(), before, "legacy BOM refusal preserves all state");
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": TARGET, "role": "node", "scopes": ["node.invoke"]}), Some(&bearer), "");
    fixture.assert_rotation(&before, &reply, TARGET, "node", json!(["node.invoke"]), false);
    fixture.observe().secrets().assert_reports_redacted(&fixture.observed);

    let fixture = Fixture::seeded(&["operator"], &["operator.pairing"]);
    let before = fixture.observe();
    let reply = fixture.rotate(&json!({"deviceId": CALLER, "role": "operator"}),
        Some(&format!("Bearer {}", before.token(CALLER, "operator"))), "");
    let successor = fixture.assert_rotation(&before, &reply, CALLER, "operator", json!(["operator.pairing"]), true);
    let proof = fixture.owner().authenticate_device_token(&successor).unwrap().unwrap();
    fixture.owner().validate_current_device_authorization(&proof).expect("canonical pairing successor remains current");
}

#[test]
fn token_rotate_wire_http_primary_only_successors_materialize_roles_atomically_and_withdraw_after_reopen() {
    for (role, original_roles, expected_roles, scopes) in [
        ("operator", " [\"node\"] ", json!(["node", "operator"]), json!(["operator.pairing"])),
        ("node", " [] ", json!(["node"]), json!(["node.invoke"])),
    ] {
        let mut fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
        let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
        fixture.set("UPDATE device_pairing_paired SET roles_json = ?1, role = ?2 WHERE device_id = ?3", params![original_roles, role, TARGET]);
        fixture.edit_tokens(TARGET, |entries| { entries.as_object_mut().unwrap().remove(role); });
        let before = fixture.observe();
        let args = json!({"deviceId": TARGET, "role": role, "scopes": scopes});
        // The known revision refusal must roll back primary materialization as
        // well as token creation and node surface invalidation.
        fixture.set("UPDATE config_machine_state SET value_json = 'dtr-r2-invalid-revision' WHERE state_key = 'devicePairing.revision'", []);
        let refused = fixture.deny(&args.to_string(), Some(&bearer), "", 500, 1);
        assert!(!refused.raw.contains("dtr-r2-invalid-revision"));
        assert_eq!(text(fixture.observe().device(TARGET), "roles_json"), original_roles);
        let revision = before.config.iter().find(|row| text(row, "state_key") == "devicePairing.revision").unwrap();
        fixture.set("UPDATE config_machine_state SET value_json = ?1 WHERE state_key = 'devicePairing.revision'", params![revision["value_json"].clone()]);
        assert_eq!(fixture.observe(), before);
        let reply = fixture.rotate(&args, Some(&bearer), "");
        let successor = fixture.assert_primary_rotation(&before, &reply, TARGET, role, scopes.clone(), expected_roles);
        let proof = fixture.owner().authenticate_device_token(&successor).unwrap().unwrap();
        fixture.owner().validate_current_device_authorization(&proof).expect("primary-only successor must be immediately current");
        let committed = fixture.observe();
        assert_eq!(text(committed.device(TARGET), "role"), role, "primary label is not rewritten");
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        assert_eq!(fixture.owner().validate_current_device_authorization(&proof).unwrap_err().status(), 401, "old owner proof never transfers across reopen");
        let fresh = fixture.owner().authenticate_device_token(&successor).unwrap().unwrap();
        fixture.owner().validate_current_device_authorization(&fresh).expect("freshly authenticated persisted successor remains current after reopen");
        fixture.set("UPDATE device_pairing_paired SET roles_json = ?1 WHERE device_id = ?2", params![original_roles, TARGET]);
        let withdrawn = fixture.observe();
        assert_eq!(text(withdrawn.device(TARGET), "role"), role);
        assert!(withdrawn.token(TARGET, role) == successor, "withdrawal changes approval, not the bearer identity");
        assert_eq!(fixture.owner().validate_current_device_authorization(&fresh).unwrap_err().status(), 401, "primary label cannot resurrect role-array withdrawal");
        fixture.reopen();
        assert_eq!(fixture.observe(), withdrawn);
        if let Some(proof) = fixture.owner().authenticate_device_token(&successor).expect("withdrawn lookup") {
            assert_eq!(fixture.owner().validate_current_device_authorization(&proof).unwrap_err().status(), 401);
        }
        assert_eq!(fixture.observe(), withdrawn);
        before.secrets().assert_reports_redacted(&fixture.observed);
        withdrawn.secrets().assert_reports_redacted(&fixture.observed);
    }
}
