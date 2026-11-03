//! DVR-v1: real TCP admission, the sealed owner proof and durable whole-row removal.
//! Contract: projects/openclaw-rust/docs/specs/device-pairing-revocation-continuity.md.
//! The API adapter delegates policy to DevicePairingOwner. It is not shipping App
//! wiring or physical-device evidence. Execute this target only in exact-H Actions.

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

const REVOKE: &str = "device.pair.revoke";
const CALLER: &str = "dvr-http-caller";
const TARGET: &str = "dvr-http-target";
const NATIVE_SECRET: &str = "dvr-http-native-secret";
const WAIT: Duration = Duration::from_secs(15);
const IO_WAIT: Duration = Duration::from_secs(30);
type Row = BTreeMap<String, SqlValue>;

#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    paired: Vec<Row>,
    pending: Vec<Row>,
    config: Vec<Row>,
}

fn rows(conn: &Connection, sql: &str) -> Vec<Row> {
    let mut statement = conn.prepare(sql).expect("prepare independent observation");
    let names: Vec<_> = statement.column_names().into_iter().map(str::to_owned).collect();
    statement.query_map([], |row| {
        names.iter().enumerate().map(|(index, name)| {
            Ok((name.clone(), row.get::<_, SqlValue>(index)?))
        }).collect::<rusqlite::Result<Row>>()
    }).expect("query complete SQL rows").collect::<rusqlite::Result<Vec<_>>>()
        .expect("retain all raw SQL values")
}

fn text<'a>(row: &'a Row, key: &str) -> &'a str {
    match row.get(key) {
        Some(SqlValue::Text(value)) => value,
        _ => panic!("expected SQL TEXT in {key}"),
    }
}

impl Snapshot {
    fn device(&self, id: &str) -> &Row {
        self.paired.iter().find(|row| text(row, "device_id") == id).expect("paired fixture device")
    }

    fn token(&self, id: &str, role: &str) -> String {
        let tokens: Value = serde_json::from_str(text(self.device(id), "tokens_json")).expect("token map");
        tokens[role]["token"].as_str().expect("actual issued credential").to_owned()
    }

    fn revision(&self) -> u64 {
        let row = self.config.iter().find(|row| text(row, "state_key") == "devicePairing.revision")
            .expect("revision row");
        assert_eq!(text(row, "value_type"), "text");
        assert_eq!(text(row, "timestamp_type"), "integer");
        let raw = text(row, "value_json");
        let decimal: String = serde_json::from_str(raw).expect("quoted decimal revision");
        let value = decimal.parse::<u64>().expect("unsigned revision");
        assert_eq!(raw, format!("\"{value}\""));
        value
    }

    fn assert_redacted(&self, reply: &Reply) {
        for row in &self.paired {
            let tokens: Value = serde_json::from_str(text(row, "tokens_json")).expect("stored token map");
            for entry in tokens.as_object().expect("role map").values() {
                let token = entry["token"].as_str().expect("stored credential");
                assert!(!reply.raw.contains(token),
                    "wire response disclosed a stored credential");
                let digest = Sha256::digest(token.as_bytes());
                for fingerprint in [
                    format!("{digest:x}"), format!("{digest:X}"), format!("{:?}", digest.to_vec()),
                    base64::engine::general_purpose::STANDARD.encode(&digest),
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest),
                ] {
                    assert!(!reply.raw.contains(&fingerprint), "wire response disclosed a sealed fingerprint");
                }
            }
        }
        let diagnostic = reply.body["error"].as_str().unwrap_or_default();
        for row in self.paired.iter().chain(&self.pending).chain(&self.config) {
            for (column, value) in row {
                if column.ends_with("_json") {
                    if let SqlValue::Text(raw) = value {
                        if raw.len() > 2 {
                            assert!(!diagnostic.contains(raw), "wire error disclosed raw persisted authority");
                        }
                    }
                }
            }
        }
        assert!(!reply.raw.contains(NATIVE_SECRET));
        assert!(reply.body.get("token").is_none());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Counts {
    native: usize,
    bound: usize,
    recognized: usize,
}

#[derive(Default)]
struct Observations {
    native: AtomicUsize,
    bound: AtomicUsize,
    recognized: AtomicUsize,
    contexts: Mutex<Vec<GatewayCommandContext>>,
    successes: Mutex<Vec<GatewayCommandReport>>,
    errors: Mutex<Vec<GatewayCommandReport>>,
}

impl Observations {
    fn counts(&self) -> Counts {
        Counts {
            native: self.native.load(Ordering::SeqCst),
            bound: self.bound.load(Ordering::SeqCst),
            recognized: self.recognized.load(Ordering::SeqCst),
        }
    }
}

// Arrival occurs in the real bound hook, after Gateway authentication but
// before owner admission. Neither the fixture lock nor an owner lock is held
// while the other thread commits a retirement.
struct Gate {
    remaining: AtomicUsize,
    entered: SyncSender<()>,
    released: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    fn pause(&self) -> Result<(), GatewayCommandError> {
        if self.remaining.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)).is_err() {
            return Ok(());
        }
        self.entered.try_send(()).map_err(|_| GatewayCommandError::Internal("DVR fixture arrival failed".into()))?;
        let released = self.released.lock().expect("gate release lock");
        let (released, _) = self.changed.wait_timeout_while(released, WAIT, |value| !*value)
            .expect("bounded hook barrier");
        if !*released {
            return Err(GatewayCommandError::Internal("DVR fixture barrier expired".into()));
        }
        Ok(())
    }

    fn release(&self) {
        *self.released.lock().unwrap_or_else(|error| error.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct GateControl {
    gate: Arc<Gate>,
    entered: Receiver<()>,
}

impl GateControl {
    fn wait(&self, count: usize) {
        for _ in 0..count {
            self.entered.recv_timeout(WAIT).expect("authenticated request reached bound revoke hook");
        }
    }

    fn release(&self) {
        self.gate.release();
    }
}

impl Drop for GateControl {
    fn drop(&mut self) {
        self.release();
    }
}

struct Api {
    owner: Option<DevicePairingOwner>,
    observed: Arc<Observations>,
    gate: Mutex<Option<Arc<Gate>>>,
    closed: SyncSender<()>,
}

impl Api {
    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("open production owner")
    }
}

impl GatewayApi for Api {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let proof = self.owner().authenticate_device_token(token)?;
        if proof.is_some() {
            self.observed.recognized.fetch_add(1, Ordering::SeqCst);
        }
        Ok(proof)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.observed.native.fetch_add(1, Ordering::SeqCst);
        self.owner().handle_pairing_command(method, &args)
    }

    fn revoke_paired_device(
        &self,
        args: Value,
        authorization: &DeviceTokenAuthorization,
        scope_ceiling: Option<&str>,
        context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.observed.bound.fetch_add(1, Ordering::SeqCst);
        self.observed.contexts.lock().expect("contexts").push(context.clone());
        let gate = self.gate.lock().expect("optional gate").clone();
        if let Some(gate) = gate {
            gate.pause()?;
        }
        self.owner().revoke_pairing_for_authenticated_device(&args, authorization, scope_ceiling)
    }

    fn redeem_device_pairing_join_code(&self, shortcode: &str) -> Result<Option<Value>, GatewayCommandError> {
        self.owner().redeem_device_pairing_join_code(shortcode)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        self.observed.successes.lock().expect("successful command reports").push(report);
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        self.observed.errors.lock().expect("error command reports").push(report);
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
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.0.call(method, args)
    }

    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.0.authenticate_device_token(token)
    }
    // Deliberately inherit the fail-closed revoke hook; no generic fallback.
}

#[derive(Debug)]
struct Reply {
    status: u16,
    body: Value,
    raw: String,
}

fn request(port: u16, verb: &str, path: &str, body: &str, bearer: Option<&str>, headers: &str) -> Result<Reply, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, IO_WAIT).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    let authorization = bearer.map(|value| format!("Authorization: {value}\r\n")).unwrap_or_default();
    write!(stream,
        "{verb} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    ).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let deadline = Instant::now() + IO_WAIT;
    let mut bytes = Vec::new();
    loop {
        let remaining = deadline.checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero()).ok_or("absolute HTTP response deadline exceeded")?;
        stream.set_read_timeout(Some(remaining)).map_err(|error| error.to_string())?;
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 { break; }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > 1024 * 1024 { return Err("unexpectedly large HTTP response".into()); }
    }
    let raw = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    let status = raw.lines().next().and_then(|line| line.split_whitespace().nth(1))
        .ok_or("missing HTTP status")?.parse::<u16>().map_err(|error| error.to_string())?;
    let body = serde_json::from_str(raw.split_once("\r\n\r\n").ok_or("missing HTTP body")?.1)
        .map_err(|error| format!("non-JSON response: {error}"))?;
    Ok(Reply { status, body, raw })
}

struct PendingRequest {
    result: Receiver<Result<Reply, String>>,
    thread: Option<JoinHandle<()>>,
    gate: Arc<Gate>,
}

impl PendingRequest {
    fn finish(mut self) -> Reply {
        let reply = self.result.recv_timeout(IO_WAIT).expect("bounded request completion");
        self.thread.take().expect("owned client").join().expect("completed TCP client");
        reply.expect("actual HTTP transport")
    }
}

impl Drop for PendingRequest {
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
        let root = std::env::temp_dir().join(format!("fabushi-dvr-http-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("create exclusively owned fixture directory");
        let mut fixture = Self {
            db: root.join("pairing.sqlite"), root,
            config: GatewayServerConfig {
                host: host.into(), port: None, auth_token: configured.then(|| NATIVE_SECRET.into()), tls: None,
            },
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
        let (sender, closed) = mpsc::sync_channel(1);
        let api = Arc::new(Api {
            owner: Some(DevicePairingOwner::new(&self.db).expect("open durable owner")),
            observed: Arc::clone(&self.observed), gate: Mutex::new(None), closed: sender,
        });
        let dispatch: Arc<dyn GatewayApi> = if self.legacy { Arc::new(LegacyApi(Arc::clone(&api))) } else { api.clone() };
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: dispatch, events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: self.config.clone(), started_at: 1,
        }).expect("bind actual TCP Gateway"));
        self.api = Some(api);
        self.closed = Some(closed);
    }

    fn stop(&mut self) -> bool {
        if let Some(gate) = self.gate.take() { gate.release(); }
        if let Some(server) = self.server.take() { server.close(); }
        drop(self.api.take());
        if let Some(closed) = self.closed.take() {
            self.shutdown_failed |= closed.recv_timeout(IO_WAIT).is_err();
        }
        !self.shutdown_failed
    }

    fn reopen(&mut self) {
        assert!(self.stop(), "await every connection/API/SQLite drop before reopen");
        self.open();
    }

    fn owner(&self) -> &DevicePairingOwner { self.api.as_ref().expect("live adapter").owner() }
    fn port(&self) -> u16 { self.server.as_ref().expect("owned listener").port() }

    fn observe(&self) -> Snapshot {
        let mut conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_ONLY).expect("independent observer");
        let tx = conn.transaction().expect("one observation snapshot");
        let state = Snapshot {
            paired: rows(&tx, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: rows(&tx, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            config: rows(&tx, "SELECT *, typeof(value_json) AS value_type, typeof(updated_at_ms) AS timestamp_type FROM config_machine_state ORDER BY state_key"),
        };
        tx.commit().expect("finish read transaction");
        conn.close().expect("close observer before mutation or reopen");
        state
    }

    fn pair(&self, id: &str, roles: &[&str], scopes: &[&str]) {
        let pending = self.owner().handle_pair_request(&json!({
            "deviceId": id, "publicKey": format!("dvr-key-{id}"), "displayName": id,
            "role": roles[0], "roles": roles, "scopes": scopes,
        })).expect("actual pairing request");
        self.owner().handle_pair_approve(&json!({"requestId": pending["requestId"]})).expect("actual approval");
    }

    fn seed(&self, roles: &[&str], scopes: &[&str]) {
        self.pair(CALLER, roles, scopes);
        self.pair(TARGET, &["operator", "node"], &["operator.write", "node.invoke"]);
        for id in [CALLER, TARGET] {
            self.owner().handle_pair_request(&json!({
                "deviceId": id, "publicKey": format!("dvr-repair-{id}"),
                "role": "operator", "roles": ["operator"], "scopes": ["operator.read"],
            })).expect("preserved pending repair");
        }
        let conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_WRITE).expect("fixture writer");
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2",
            params!["{\"generation\":7,\"commands\":[\"system.run\"]}", "{\"generation\":8}"])
            .expect("seed complete retained surfaces"), 2);
        conn.execute("INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES ('dvr.unrelated', '\"preserve\"', 7)", [])
            .expect("unrelated configuration witness");
        conn.close().expect("close fixture writer");
    }

    fn http(&self, verb: &str, path: &str, body: &str, bearer: Option<&str>, headers: &str) -> Reply {
        request(self.port(), verb, path, body, bearer, headers).expect("real TCP response")
    }

    fn revoke(&self, args: &Value, token: &str, headers: &str) -> Reply {
        self.http("POST", "/api/device.pair.revoke", &args.to_string(), Some(&format!("Bearer {token}")), headers)
    }

    fn deny(&self, body: &str, bearer: Option<&str>, headers: &str, status: u16, expected_bound: Option<usize>) -> Reply {
        let before = self.observe();
        let counts = self.observed.counts();
        let reply = self.http("POST", "/api/device.pair.revoke", body, bearer, headers);
        assert_refused(&reply, status, &before);
        assert_eq!(self.observe(), before, "refusal preserves all paired/pending/config SQL values");
        let after = self.observed.counts();
        assert_eq!(after.native, counts.native, "no generic paired or invalid-credential fallback");
        if let Some(expected) = expected_bound {
            assert_eq!(after.bound, counts.bound + expected);
        } else {
            assert!((counts.bound..=counts.bound + 1).contains(&after.bound));
        }
        reply
    }

    fn assert_removal(&self, before: &Snapshot, reply: &Reply, id: &str, receipt_role: &str) {
        assert_eq!(reply.status, 200, "{}", reply.raw);
        let after = self.observe();
        assert_eq!(reply.body["deviceId"], id);
        assert_eq!(reply.body["role"], receipt_role);
        assert!(reply.body["revokedAtMs"].as_i64().is_some_and(|value| value > 0));
        assert_eq!(after.revision(), before.revision() + 1);
        assert_eq!(reply.body["revision"].as_u64(), Some(after.revision()));
        assert_eq!(after.pending, before.pending, "native removal preserves every pending repair");
        let retained: Vec<_> = before.paired.iter().filter(|row| text(row, "device_id") != id).cloned().collect();
        assert_eq!(after.paired, retained, "exactly the whole target row disappears, including all role tokens and surfaces");
        let unrelated = |state: &Snapshot| state.config.iter().filter(|row| text(row, "state_key") != "devicePairing.revision").cloned().collect::<Vec<_>>();
        assert_eq!(unrelated(&after), unrelated(before));
        let old_tokens: Value = serde_json::from_str(text(before.device(id), "tokens_json")).expect("removed token map");
        for entry in old_tokens.as_object().expect("role map").values() {
            assert!(self.owner().authenticate_device_token(entry["token"].as_str().expect("old token"))
                .expect("removed credential lookup").is_none());
        }
        before.assert_redacted(reply);
    }

    fn install_gate(&mut self, count: usize) -> GateControl {
        let (sender, entered) = mpsc::sync_channel(count);
        let gate = Arc::new(Gate {
            remaining: AtomicUsize::new(count), entered: sender, released: Mutex::new(false), changed: Condvar::new(),
        });
        *self.api.as_ref().expect("adapter").gate.lock().expect("install gate") = Some(Arc::clone(&gate));
        self.gate = Some(Arc::clone(&gate));
        GateControl { gate, entered }
    }

    fn spawn_revoke(&self, gate: &GateControl, args: Value, token: &str) -> PendingRequest {
        let port = self.port();
        let body = args.to_string();
        let bearer = format!("Bearer {token}");
        let (sender, result) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let _ = sender.send(request(port, "POST", "/api/device.pair.revoke", &body, Some(&bearer), ""));
        });
        PendingRequest { result, thread: Some(handle), gate: Arc::clone(&gate.gate) }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.stop() {
            let removed = fs::remove_dir_all(&self.root);
            if !thread::panicking() { removed.expect("remove only the fully closed owned fixture"); }
        } else {
            eprintln!("DVR HTTP fixture retained after unacknowledged shutdown: {}", self.root.display());
            assert!(thread::panicking(), "bounded connection/API shutdown failed");
        }
    }
}

fn assert_refused(reply: &Reply, status: u16, state: &Snapshot) {
    assert_eq!(reply.status, status, "{}", reply.raw);
    assert!(reply.body["error"].as_str().is_some_and(|error| !error.is_empty()));
    assert!(reply.body.get("revision").is_none());
    state.assert_redacted(reply);
}

fn assert_bound_once(fixture: &Fixture, before: Counts) {
    assert_eq!(fixture.observed.counts(), Counts { bound: before.bound + 1, recognized: before.recognized + 1, ..before });
}

#[test]
fn paired_revoke_self_operator_only_delivers_final_reply_and_reports_bound_context_once() {
    for role in [None, Some("operator")] {
        let mut fixture = Fixture::seeded(&["operator"], &["operator.write"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "operator");
        let mut args = json!({"deviceId": CALLER});
        if let Some(role) = role { args["role"] = json!(role); }
        let counts = fixture.observed.counts();
        let reply = fixture.revoke(&args, &token,
            "x-sand-request-id: dvr-self\r\ntraceparent: 00-0123456789abcdef0123456789abcdef-0123456789abcdef-01\r\n");
        assert_bound_once(&fixture, counts);
        fixture.assert_removal(&before, &reply, CALLER, role.unwrap_or("all"));
        {
            let contexts = fixture.observed.contexts.lock().expect("contexts");
            assert_eq!(contexts.len(), 1);
            assert!(contexts[0].authenticated);
            assert_eq!(contexts[0].request_id.as_deref(), Some("dvr-self"));
            assert_eq!(contexts[0].trace_id.as_deref(), Some("0123456789abcdef0123456789abcdef"));
            assert_eq!(contexts[0].span_id.as_deref(), Some("0123456789abcdef"));
            let reports = fixture.observed.successes.lock().expect("reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].method, REVOKE);
            assert_eq!(reports[0].status, 200);
            assert_eq!(reports[0].request_id, contexts[0].request_id);
            assert_eq!(reports[0].traceparent, contexts[0].traceparent);
            assert!(fixture.observed.errors.lock().expect("errors").is_empty());
        }
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        fixture.deny(&args.to_string(), Some(&format!("Bearer {token}")), "", 401, Some(0));
    }
}

#[test]
fn paired_revoke_write_rejects_cross_device_and_mixed_self_despite_body_or_header_forgery() {
    for mixed in [false, true] {
        let roles: &[&str] = if mixed { &["operator", "node"] } else { &["operator"] };
        let fixture = Fixture::seeded(roles, &["operator.write", "node.invoke"]);
        let token = fixture.observe().token(CALLER, "operator");
        let bearer = format!("Bearer {token}");
        let present = fixture.deny(&json!({"deviceId": TARGET}).to_string(), Some(&bearer), "", 403, Some(1));
        let absent = fixture.deny(&json!({"deviceId": "dvr-unknown"}).to_string(), Some(&bearer), "", 403, Some(1));
        assert_eq!(present.body, absent.body, "cross-device denial must not disclose target existence");
        for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
            fixture.deny(&json!({"deviceId": TARGET, "role": "operator", "callerDeviceId": TARGET,
                "isAdmin": true, "admin": true, "authorization": {"scopes": ["operator.admin"]}}).to_string(),
                Some(&bearer), headers, 403, Some(1));
            if mixed {
                fixture.deny(&json!({"deviceId": CALLER, "role": "operator", "roles": ["operator"]}).to_string(),
                    Some(&bearer), headers, 403, Some(1));
            }
        }
        if !mixed {
            for args in [json!({}), json!({"deviceId": CALLER, "role": ""}), json!({"deviceId": CALLER, "role": 7})] {
                fixture.deny(&args.to_string(), Some(&bearer), "", 400, Some(1));
            }
            fixture.deny("{", Some(&bearer), "", 400, Some(0));
        }
    }
}

#[test]
fn paired_revoke_admin_implication_ceilings_and_storage_error_reporting_use_real_owner() {
    let fixture = Fixture::seeded(&["operator", "node"], &["operator.admin", "node.invoke"]);
    let token = fixture.observe().token(CALLER, "operator");
    let bearer = format!("Bearer {token}");
    for headers in ["x-sand-scope: operator.write\r\n", "x-sand-scope: operator.read\r\n",
        "x-sand-scope:\r\n", "x-sand-scope: operator.write operator.admin\r\n", "x-sand-role: node\r\n"] {
        fixture.deny(&json!({"deviceId": TARGET}).to_string(), Some(&bearer), headers, 403, None);
    }
    fixture.deny(&json!({"deviceId": CALLER, "role": "operator"}).to_string(), Some(&bearer),
        "x-sand-scope: operator.write\r\n", 403, Some(1));
    fixture.deny(&json!({"deviceId": "dvr-authorized-absent"}).to_string(), Some(&bearer), "", 400, Some(1));
    let before = fixture.observe();
    let counts = fixture.observed.counts();
    let reply = fixture.revoke(&json!({"deviceId": TARGET, "role": "operator"}), &token,
        "x-sand-scope: operator.admin\r\n");
    assert_bound_once(&fixture, counts);
    fixture.assert_removal(&before, &reply, TARGET, "operator");

    let conn = Connection::open_with_flags(&fixture.db, OpenFlags::SQLITE_OPEN_READ_WRITE).expect("fixture corruption writer");
    conn.execute("UPDATE config_machine_state SET value_json = 'dvr-private-corrupt-revision' WHERE state_key = 'devicePairing.revision'", [])
        .expect("inject checked revision failure");
    conn.close().expect("close corruption writer");
    let reply = fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&bearer),
        "x-sand-request-id: dvr-storage-error\r\n", 500, Some(1));
    assert!(!reply.raw.contains("dvr-private-corrupt-revision"));
    let reports = fixture.observed.errors.lock().expect("normal error report path");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].method, REVOKE);
    assert_eq!(reports[0].status, 500);
    assert_eq!(reports[0].request_id.as_deref(), Some("dvr-storage-error"));
    assert!(!reports[0].error.as_deref().unwrap_or_default().contains(&token));
    assert!(!reports[0].error.as_deref().unwrap_or_default().contains("dvr-private-corrupt-revision"));
}

#[test]
fn paired_revoke_read_node_empty_and_unrelated_grants_cannot_gain_management_authority() {
    for (role, scopes) in [("operator", vec!["operator.read"]), ("operator", vec![]),
        ("operator", vec!["operator.sessions.write"]), ("node", vec!["node.invoke"]), ("observer", vec!["observer.read"])] {
        let fixture = Fixture::seeded(&[role], &scopes);
        let bearer = format!("Bearer {}", fixture.observe().token(CALLER, role));
        for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
            fixture.deny(&json!({"deviceId": CALLER, "callerRole": "operator", "isAdmin": true}).to_string(),
                Some(&bearer), headers, 403, None);
        }
    }
    let fixture = Fixture::seeded(&["operator"], &["operator.write"]);
    let before = fixture.observe();
    let token = before.token(CALLER, "operator");
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&format!("Bearer {token}")),
        "x-sand-scope:\r\n", 403, Some(1));
    let reply = fixture.revoke(&json!({"deviceId": CALLER}), &token, "x-sand-scope: operator.write\r\n");
    fixture.assert_removal(&before, &reply, CALLER, "all");
}

#[test]
fn paired_revoke_failed_credentials_listener_spoofing_and_public_native_controls_are_distinct() {
    for configured in [false, true] {
        let fixture = Fixture::new(configured, "127.0.0.1", false);
        fixture.seed(&["operator"], &["operator.write"]);
        let replaced = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER})).expect("replace real token");
        fixture.pair("dvr-removed", &["operator"], &["operator.write"]);
        let removed = fixture.observe().token("dvr-removed", "operator");
        fixture.owner().handle_pair_revoke(&json!({"deviceId": "dvr-removed"})).expect("retire real credential");
        for authorization in ["Bearer unknown", "Bearer", "Bearer ", "", "Basic ignored",
            &format!("Bearer {replaced}"), &format!("Bearer {removed}")] {
            for body in [json!({"deviceId": TARGET}).to_string(), "{".into()] {
                fixture.deny(&body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n", 401, Some(0));
            }
            let counts = fixture.observed.counts();
            assert_eq!(fixture.http("POST", "/api/device.pair.list", "{}", Some(authorization), "").status, 401);
            assert_eq!(fixture.observed.counts(), counts, "failed credential never reaches generic native methods");
        }
        if configured {
            fixture.deny(&json!({"deviceId": TARGET}).to_string(), None, "", 401, Some(0));
        }
        let native = configured.then(|| format!("Bearer {NATIVE_SECRET}"));
        let before = fixture.observe();
        let counts = fixture.observed.counts();
        let reply = fixture.http("POST", "/api/device.pair.revoke", &json!({"deviceId": TARGET, "role": "operator"}).to_string(), native.as_deref(), "");
        assert_eq!(fixture.observed.counts(), Counts { native: counts.native + 1, ..counts });
        fixture.assert_removal(&before, &reply, TARGET, "operator");
        let before_public = fixture.observe();
        let pending = fixture.http("POST", "/api/device.pair.request", &json!({
            "deviceId": "dvr-public", "publicKey": "dvr-public-key", "scopes": ["operator.read"],
        }).to_string(), Some("Basic deliberately-public"), "");
        assert_eq!(pending.status, 200);
        assert_eq!(pending.body["status"], "pending");
        let status = fixture.http("POST", "/api/device.pair.status", &json!({"requestId": pending.body["requestId"]}).to_string(), None, "");
        assert_eq!(status.status, 200);
        assert_eq!(status.body["status"], "pending");
        let health = fixture.http("GET", "/health", "", Some("Bearer removed-or-unknown"), "");
        assert_eq!(health.status, 200);
        assert_eq!(health.body["ok"], true);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_millis() as i64;
        let payload = json!({"gatewayUrl": "https://fixture.invalid", "bootstrapToken": "fixture-public-redemption"});
        let code = fixture.owner().register_device_pairing_join_code(&payload, now + 60_000).expect("real public join code");
        let joined = fixture.http("GET", &format!("/j/{code}"), "", None, "");
        assert_eq!(joined.status, 200);
        assert_eq!(joined.body, payload);
        assert_eq!(fixture.http("GET", &format!("/j/{code}"), "", None, "").status, 404);
        assert_eq!(fixture.observe().paired, before_public.paired);
        assert_eq!(fixture.observe().config, before_public.config);
        assert_eq!(fixture.http("GET", "/health", "", None, "Origin: https://untrusted.invalid\r\n").status, 403);
    }
    let fixture = Fixture::new(false, "0.0.0.0", false);
    fixture.seed(&["operator"], &["operator.write"]);
    // Actual connection and Host are loopback; the listener itself is wildcard.
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), None, "", 401, Some(0));
    let before = fixture.observe();
    let reply = fixture.revoke(&json!({"deviceId": CALLER}), &before.token(CALLER, "operator"), "");
    fixture.assert_removal(&before, &reply, CALLER, "all");
}

#[test]
fn paired_revoke_default_hook_refuses_without_generic_dispatch_and_native_still_works() {
    let fixture = Fixture::new(false, "127.0.0.1", true);
    fixture.seed(&["operator"], &["operator.write"]);
    let before = fixture.observe();
    let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&bearer), "", 403, Some(0));
    let counts = fixture.observed.counts();
    let native = fixture.http("POST", "/api/device.pair.revoke", &json!({"deviceId": CALLER}).to_string(), None, "");
    assert_eq!(fixture.observed.counts(), Counts { native: counts.native + 1, ..counts });
    fixture.assert_removal(&before, &native, CALLER, "all");
}

#[test]
fn paired_revoke_rechecks_after_authentication_with_unchanged_and_unrelated_controls() {
    for retirement in ["unchanged", "replace", "remove", "unrelated"] {
        let mut fixture = Fixture::seeded(&["operator"], &["operator.admin"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "operator");
        let counts = fixture.observed.counts();
        let gate = fixture.install_gate(1);
        let pending = fixture.spawn_revoke(&gate, json!({"deviceId": TARGET}), &token);
        gate.wait(1);
        assert_bound_once(&fixture, counts);
        assert_eq!(fixture.observe(), before, "authenticated call has not entered the mutation owner");
        match retirement {
            "replace" => { fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER})).expect("caller replacement"); }
            "remove" => { fixture.owner().handle_pair_revoke(&json!({"deviceId": CALLER})).expect("caller removal"); }
            "unrelated" => { fixture.owner().handle_pair_rotate_token(&json!({"deviceId": TARGET, "role": "node"})).expect("unrelated role change"); }
            "unchanged" => {}
            _ => unreachable!(),
        }
        let after_retirement = fixture.observe();
        gate.release();
        let reply = pending.finish();
        assert_bound_once(&fixture, counts);
        if matches!(retirement, "replace" | "remove") {
            assert_refused(&reply, 401, &after_retirement);
            assert_eq!(fixture.observe(), after_retirement, "stale bound request cannot change the post-retirement revision or target");
            assert_eq!(after_retirement.device(TARGET), before.device(TARGET));
            assert_eq!(after_retirement.revision(), before.revision() + 1);
            assert!(!reply.raw.contains(&token));
        } else {
            fixture.assert_removal(&after_retirement, &reply, TARGET, "all");
        }
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
    }

    // DVR-008: observer reaches the bound hook; node intentionally retains its
    // earlier HTTP ingress refusal. Edit storage only after a real proof exists.
    for (change, expected) in [
        ("unchanged", 403), ("unrelated", 403),
        ("approval-withdrawn", 401), ("partial-approval", 401),
        ("malformed-live-grant", 500), ("malformed-present-baseline", 500),
        ("wrong-type-present-baseline", 500),
    ] {
        let mut fixture = Fixture::seeded(&["observer"], &["observer.read", "observer.listen"]);
        let before = fixture.observe();
        let token = before.token(CALLER, "observer");
        let counts = fixture.observed.counts();
        let gate = fixture.install_gate(1);
        let pending = fixture.spawn_revoke(&gate, json!({"deviceId": CALLER}), &token);
        gate.wait(1);
        assert_bound_once(&fixture, counts);
        assert!(fixture.observe() == before, "real observer authentication precedes the independent state edit");

        let conn = Connection::open_with_flags(&fixture.db, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("independent post-authentication authority writer");
        match change {
            "unchanged" => {}
            "malformed-live-grant" => {
                let mut entries: Value = serde_json::from_str(text(before.device(CALLER), "tokens_json")).expect("original role map");
                entries["observer"]["scopes"] = json!(["observer.read", 7]);
                assert_eq!(conn.execute(
                    "UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
                    params![entries.to_string(), CALLER],
                ).expect("corrupt only the captured role's live grant"), 1);
            }
            _ => {
                let (device, baseline) = match change {
                    "unrelated" => (TARGET, SqlValue::Text("[]".into())),
                    "approval-withdrawn" => (CALLER, SqlValue::Text("[]".into())),
                    "partial-approval" => (CALLER, SqlValue::Text(json!(["observer.read"]).to_string())),
                    "malformed-present-baseline" => (CALLER, SqlValue::Text("DVR-private-nonoperator-baseline".into())),
                    "wrong-type-present-baseline" => (CALLER, SqlValue::Blob(vec![0xff, 0, 8])),
                    _ => unreachable!(),
                };
                assert_eq!(conn.execute(
                    "UPDATE device_pairing_paired SET approved_scopes_json = ?1 WHERE device_id = ?2",
                    params![baseline, device],
                ).expect("edit exactly one independent approval baseline"), 1);
            }
        }
        conn.close().expect("commit and close the independent writer before bound owner admission");
        let after_edit = fixture.observe();
        assert!(after_edit.config == before.config, "authority edit preserves raw revision value, type and timestamp");
        assert!(after_edit.pending == before.pending);
        assert!(after_edit.token(CALLER, "observer") == token, "the originally authenticated bytes remain unchanged");
        gate.release();
        let reply = pending.finish();
        after_edit.assert_redacted(&reply);
        assert!(!reply.raw.contains("DVR fixture"), "a barrier failure is never an authority-refusal oracle");
        assert_refused(&reply, expected, &after_edit);
        if expected == 500 {
            assert!(reply.body["error"].as_str().is_some_and(|error| error.contains("stored pairing authority is invalid")),
                "500 must be the real owner's decisive-authority refusal");
        }
        assert_bound_once(&fixture, counts);
        assert!(fixture.observe() == after_edit, "bound refusal preserves every post-edit row, token, surface and raw revision");

        let conn = Connection::open_with_flags(&fixture.db, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("independent explicit authority repair");
        for device in [CALLER, TARGET] {
            assert_eq!(conn.execute(
                "UPDATE device_pairing_paired SET tokens_json = ?1, approved_scopes_json = ?2 WHERE device_id = ?3",
                params![before.device(device).get("tokens_json").unwrap(),
                    before.device(device).get("approved_scopes_json").unwrap(), device],
            ).expect("restore the exact original authority values"), 1);
        }
        conn.close().expect("close explicit repair writer");
        assert!(fixture.observe() == before, "repair restores every original raw state byte without a revision");
        let repaired_counts = fixture.observed.counts();
        let repair_gate = fixture.install_gate(1);
        let repaired = fixture.spawn_revoke(&repair_gate, json!({"deviceId": CALLER}), &token);
        repair_gate.wait(1);
        assert_bound_once(&fixture, repaired_counts);
        assert!(fixture.observe() == before);
        repair_gate.release();
        let reply = repaired.finish();
        before.assert_redacted(&reply);
        assert_refused(&reply, 403, &before);
        assert_bound_once(&fixture, repaired_counts);
        assert!(fixture.observe() == before, "a repaired current observer still has no management authority");
    }
}

#[test]
fn paired_revoke_two_authenticated_self_requests_commit_one_removal_and_one_stale_result() {
    let mut fixture = Fixture::seeded(&["operator"], &["operator.write"]);
    let before = fixture.observe();
    let token = before.token(CALLER, "operator");
    let counts = fixture.observed.counts();
    let gate = fixture.install_gate(2);
    let first = fixture.spawn_revoke(&gate, json!({"deviceId": CALLER}), &token);
    let second = fixture.spawn_revoke(&gate, json!({"deviceId": CALLER}), &token);
    gate.wait(2);
    assert_eq!(fixture.observe(), before, "both real proofs precede either mutation");
    assert_eq!(fixture.observed.counts(), Counts { bound: counts.bound + 2, recognized: counts.recognized + 2, ..counts });
    gate.release();
    let replies = [first.finish(), second.finish()];
    let mut statuses = [replies[0].status, replies[1].status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 401]);
    fixture.assert_removal(&before, replies.iter().find(|reply| reply.status == 200).expect("one success"), CALLER, "all");
    let committed = fixture.observe();
    assert_refused(replies.iter().find(|reply| reply.status == 401).expect("one stale request"), 401, &committed);
    assert_eq!(fixture.observed.successes.lock().expect("reports").len(), 1);
    assert_eq!(fixture.observed.counts(), Counts { bound: counts.bound + 2, recognized: counts.recognized + 2, ..counts });
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&format!("Bearer {token}")), "", 401, Some(0));
}
