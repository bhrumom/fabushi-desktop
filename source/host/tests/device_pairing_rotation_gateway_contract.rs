//! DPA-v1: real HTTP ingress, the production pairing owner and durable SQLite.
//! Contract: projects/openclaw-rust/docs/specs/device-pairing-authenticated-rotation.md.
//! These adapters contain no authorization policy and do not represent a full App
//! or physical-device runtime. Executable verification belongs to exact-H Actions.

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
use std::time::Duration;

use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandContext, GatewayCommandError, GatewayCommandReport,
    GatewayEventHub, GatewayServer, GatewayServerDeps, start_gateway_server,
};
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};

const ROTATE: &str = "device.pair.rotateToken";
const CALLER: &str = "dpa-http-caller";
const TARGET: &str = "dpa-http-target";
const NATIVE_TOKEN: &str = "dpa-http-native-secret";
const WAIT: Duration = Duration::from_secs(15);
const IO_WAIT: Duration = Duration::from_secs(30);
type StoredRow = BTreeMap<String, SqlValue>;

#[derive(Debug, Clone, PartialEq)]
struct StoredState {
    paired: Vec<StoredRow>,
    pending: Vec<StoredRow>,
    revision: Vec<StoredRow>,
}

fn read_rows(conn: &Connection, sql: &str) -> Vec<StoredRow> {
    let mut statement = conn.prepare(sql).expect("prepare independent SQL observation");
    let columns: Vec<_> = statement.column_names().into_iter().map(str::to_owned).collect();
    statement.query_map([], |row| {
        columns.iter().enumerate()
            .map(|(index, name)| Ok((name.clone(), row.get::<_, SqlValue>(index)?)))
            .collect::<rusqlite::Result<StoredRow>>()
    }).expect("query durable rows").collect::<rusqlite::Result<Vec<_>>>()
        .expect("retain complete raw SQL rows")
}

fn sql_text<'a>(row: &'a StoredRow, column: &str) -> &'a str {
    match row.get(column) {
        Some(SqlValue::Text(value)) => value,
        _ => panic!("expected SQL TEXT in {column}"),
    }
}

impl StoredState {
    fn device(&self, device_id: &str) -> &StoredRow {
        self.paired.iter().find(|row| sql_text(row, "device_id") == device_id)
            .expect("observed paired device")
    }

    fn tokens(&self, device_id: &str) -> Value {
        serde_json::from_str(sql_text(self.device(device_id), "tokens_json"))
            .expect("durable role token map")
    }

    fn token(&self, device_id: &str, role: &str) -> String {
        self.tokens(device_id)[role]["token"].as_str().expect("issued role token").to_owned()
    }

    fn revision_number(&self) -> u64 {
        assert_eq!(self.revision.len(), 1);
        assert_eq!(sql_text(&self.revision[0], "value_type"), "text");
        assert_eq!(sql_text(&self.revision[0], "timestamp_type"), "integer");
        let raw = sql_text(&self.revision[0], "value_json");
        let value = serde_json::from_str::<String>(raw).expect("quoted decimal revision");
        let number: u64 = value.parse().expect("unsigned revision");
        assert_eq!(raw, format!("\"{number}\""));
        number
    }

    fn assert_no_secrets_in(&self, reply: &HttpReply) {
        for row in &self.paired {
            let tokens: Value = serde_json::from_str(sql_text(row, "tokens_json"))
                .expect("stored token object");
            for entry in tokens.as_object().expect("role map").values() {
                assert!(!reply.raw.contains(entry["token"].as_str().expect("stored token")),
                    "HTTP reply disclosed a stored credential");
            }
        }
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
    contexts: Mutex<Vec<(bool, Option<String>)>>,
    completed: Mutex<Vec<GatewayCommandReport>>,
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

// A bounded, releasable barrier: arrival is inside the actual bound hook, after
// real Gateway authentication. No sleep or scheduling order establishes a race.
struct HookGate {
    remaining: AtomicUsize,
    entered: SyncSender<()>,
    released: Mutex<bool>,
    changed: Condvar,
}

impl HookGate {
    fn pause(&self) -> Result<(), GatewayCommandError> {
        if self.remaining.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)).is_err() {
            return Ok(());
        }
        self.entered.try_send(()).map_err(|_| {
            GatewayCommandError::Internal("HTTP fixture admission channel unavailable".into())
        })?;
        let released = self.released.lock().expect("hook release lock");
        let (released, _) = self.changed.wait_timeout_while(released, WAIT, |value| !*value)
            .expect("bounded hook release wait");
        if !*released {
            return Err(GatewayCommandError::Internal("HTTP fixture release deadline exceeded".into()));
        }
        Ok(())
    }

    fn release(&self) {
        *self.released.lock().unwrap_or_else(|error| error.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct GateControl {
    gate: Arc<HookGate>,
    entered: Receiver<()>,
}

impl GateControl {
    fn wait_for_entries(&self, count: usize) {
        for _ in 0..count {
            self.entered.recv_timeout(WAIT).expect("request reached actual bound rotation hook");
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

struct PairingApi {
    owner: Option<DevicePairingOwner>,
    observations: Arc<Observations>,
    gate: Mutex<Option<Arc<HookGate>>>,
    closed: SyncSender<()>,
}

impl PairingApi {
    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("live file-backed owner")
    }
}

impl GatewayApi for PairingApi {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let authorization = self.owner().authenticate_device_token(token)?;
        if authorization.is_some() {
            self.observations.recognized.fetch_add(1, Ordering::SeqCst);
        }
        Ok(authorization)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.observations.native.fetch_add(1, Ordering::SeqCst);
        self.owner().handle_pairing_command(method, &args)
    }

    fn rotate_paired_device_token(
        &self,
        args: Value,
        authorization: &DeviceTokenAuthorization,
        scope_ceiling: Option<&str>,
        context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.observations.bound.fetch_add(1, Ordering::SeqCst);
        self.observations.contexts.lock().expect("hook context observations")
            .push((context.authenticated, context.request_id.clone()));
        let gate = self.gate.lock().expect("optional hook barrier").clone();
        if let Some(gate) = gate {
            gate.pause()?;
        }
        self.owner().rotate_token_for_authenticated_device(&args, authorization, scope_ceiling)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        self.observations.completed.lock().expect("command reports").push(report);
    }
}

impl Drop for PairingApi {
    fn drop(&mut self) {
        // A listener close alone does not join its detached connection threads.
        // Acknowledge only after their final API reference and SQLite owner close.
        drop(self.owner.take());
        let _ = self.closed.try_send(());
    }
}

struct LegacyApi(Arc<PairingApi>);

impl GatewayApi for LegacyApi {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.0.call(method, args)
    }

    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.0.authenticate_device_token(token)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        self.0.on_command_complete(report);
    }
    // Deliberately inherit GatewayApi's default bound rotation implementation.
}

#[derive(Debug)]
struct HttpReply {
    status: u16,
    body: Value,
    raw: String,
}

fn post_to_port(port: u16, method: &str, body: &str, authorization: Option<&str>, headers: &str) -> Result<HttpReply, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, IO_WAIT).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(IO_WAIT)).map_err(|error| error.to_string())?;
    let authorization = authorization.map(|value| format!("Authorization: {value}\r\n")).unwrap_or_default();
    let request = format!(
        "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    stream.write_all(request.as_bytes()).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw).map_err(|error| error.to_string())?;
    let status = raw.lines().next().and_then(|line| line.split_whitespace().nth(1))
        .ok_or("missing HTTP status")?.parse::<u16>().map_err(|error| error.to_string())?;
    let body = serde_json::from_str(raw.split_once("\r\n\r\n").ok_or("missing HTTP body")?.1)
        .map_err(|error| format!("non-JSON HTTP body: {error}"))?;
    Ok(HttpReply { status, body, raw })
}

struct PendingHttp {
    result: Receiver<Result<HttpReply, String>>,
    thread: Option<JoinHandle<()>>,
    gate: Arc<HookGate>,
}

impl PendingHttp {
    fn finish(mut self) -> HttpReply {
        let result = self.result.recv_timeout(IO_WAIT).expect("bounded HTTP completion");
        self.thread.take().expect("owned HTTP client").join().expect("HTTP client completed");
        result.expect("actual HTTP transport")
    }
}

impl Drop for PendingHttp {
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.gate.release();
            // Panic cleanup releases the hook before joining the exact client.
            // A deadline never becomes an unbounded join or successful evidence.
            if self.result.recv_timeout(IO_WAIT).is_ok() {
                let _ = self.thread.take().expect("owned client").join();
            }
        }
    }
}

struct PairingGateway {
    server: Option<GatewayServer>,
    api: Option<Arc<PairingApi>>,
    closed: Option<Receiver<()>>,
    gate: Option<Arc<HookGate>>,
    observations: Arc<Observations>,
    config: GatewayServerConfig,
    legacy: bool,
    shutdown_failed: bool,
    root: PathBuf,
    db_path: PathBuf,
}

impl PairingGateway {
    fn new(configured: bool, host: &str, legacy: bool) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-dpa-http-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned HTTP fixture directory");
        let mut fixture = Self {
            server: None, api: None, closed: None, gate: None,
            observations: Arc::new(Observations::default()),
            config: GatewayServerConfig {
                host: host.into(), port: None,
                auth_token: configured.then(|| NATIVE_TOKEN.into()), tls: None,
            },
            legacy, shutdown_failed: false, db_path: root.join("pairing.sqlite"), root,
        };
        fixture.open();
        fixture
    }

    fn seeded(role: &str, scopes: &[&str]) -> Self {
        let fixture = Self::new(false, "127.0.0.1", false);
        fixture.seed(role, scopes);
        fixture
    }

    fn open(&mut self) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let api = Arc::new(PairingApi {
            owner: Some(DevicePairingOwner::new(&self.db_path).expect("open actual durable pairing owner")),
            observations: Arc::clone(&self.observations), gate: Mutex::new(None), closed: sender,
        });
        let dispatch: Arc<dyn GatewayApi> = if self.legacy {
            Arc::new(LegacyApi(Arc::clone(&api)))
        } else {
            api.clone()
        };
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: dispatch, events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: self.config.clone(), started_at: 1,
        }).expect("start real TCP Gateway"));
        self.api = Some(api);
        self.closed = Some(receiver);
    }

    fn stop(&mut self) -> bool {
        if let Some(gate) = self.gate.take() {
            gate.release();
        }
        if let Some(server) = self.server.take() {
            server.close();
        }
        drop(self.api.take());
        if let Some(closed) = self.closed.take() {
            self.shutdown_failed |= closed.recv_timeout(IO_WAIT).is_err();
        }
        // Keep a failed shutdown terminal: a later Drop must not mistake a
        // consumed timeout receiver for proof that an old owner has closed.
        !self.shutdown_failed
    }

    fn reopen(&mut self) {
        assert!(self.stop(), "all owned connection references and SQLite owner must close before reopen");
        self.open();
    }

    fn owner(&self) -> &DevicePairingOwner {
        self.api.as_ref().expect("live adapter").owner()
    }

    fn observe(&self) -> StoredState {
        let mut conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("independent read-only SQLite observer");
        let tx = conn.transaction().expect("consistent read-only snapshot");
        let state = StoredState {
            paired: read_rows(&tx, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: read_rows(&tx, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            revision: read_rows(&tx,
                "SELECT state_key, typeof(value_json) AS value_type, value_json,
                        typeof(updated_at_ms) AS timestamp_type, updated_at_ms
                 FROM config_machine_state WHERE state_key = 'devicePairing.revision'"),
        };
        tx.commit().expect("finish read-only snapshot");
        conn.close().expect("close observer before mutation or reopen");
        state
    }

    fn approve(&self, device_id: &str, roles: &[&str], scopes: &[&str]) {
        let request = self.owner().handle_pair_request(&json!({
            "deviceId": device_id, "publicKey": format!("dpa-key-{device_id}"),
            "role": roles[0], "roles": roles, "scopes": scopes,
            "displayName": format!("DPA {device_id}"), "platform": "linux",
        })).expect("real pending pairing");
        self.owner().handle_pair_approve(&json!({"requestId": request["requestId"]}))
            .expect("production approval issues actual credentials");
    }

    fn seed(&self, role: &str, scopes: &[&str]) {
        let mut caller_scopes = scopes.to_vec();
        let caller_roles = if role == "operator" {
            caller_scopes.push("node.invoke");
            vec!["operator", "node"]
        } else {
            vec![role]
        };
        self.approve(CALLER, &caller_roles, &caller_scopes);
        self.approve(TARGET, &["operator", "node"], &["operator.read", "operator.write", "node.invoke"]);
        for device_id in [CALLER, TARGET] {
            self.owner().handle_pair_request(&json!({
                "deviceId": device_id, "publicKey": format!("dpa-repair-{device_id}"),
                "roles": ["operator", "node"], "scopes": ["operator.read", "node.invoke"],
            })).expect("fresh pending repair remains independently observable");
        }
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("explicit fixture-only retained-surface writer");
        for device_id in [CALLER, TARGET] {
            assert_eq!(conn.execute(
                "UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2
                 WHERE device_id = ?3",
                params!["{\"generation\":7,\"commands\":[\"system.run\"]}",
                    "{\"generation\":8,\"commands\":[\"system.notify\"]}", device_id],
            ).expect("seed retained and pending node surfaces"), 1);
        }
        conn.close().expect("close fixture writer");
        let state = self.observe();
        assert_eq!(state.paired.len(), 2);
        assert_eq!(state.pending.len(), 2);
        assert_eq!(state.revision_number(), 2);
        self.assert_all_current_tokens(&state);
    }

    fn assert_authorized(&self, token: &str, device_id: &str, role: &str, scopes: &Value) {
        let authorization = self.owner().authenticate_device_token(token)
            .expect("production token lookup").expect("current credential recognized");
        assert_eq!(authorization.device_id, device_id);
        assert_eq!(authorization.role, role);
        assert_eq!(serde_json::to_value(authorization.scopes).expect("scope projection"), *scopes);
    }

    fn assert_retired(&self, token: &str) {
        assert!(self.owner().authenticate_device_token(token).expect("retired credential lookup").is_none());
    }

    fn assert_all_current_tokens(&self, state: &StoredState) {
        for row in &state.paired {
            let device_id = sql_text(row, "device_id");
            for (role, entry) in state.tokens(device_id).as_object().expect("role tokens") {
                self.assert_authorized(entry["token"].as_str().expect("token"), device_id, role, &entry["scopes"]);
            }
        }
    }

    fn post(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str) -> HttpReply {
        post_to_port(self.server.as_ref().expect("owned server").port(), method, body, authorization, headers)
            .expect("real TCP HTTP response")
    }

    fn rotate(&self, args: &Value, token: &str, headers: &str) -> HttpReply {
        self.post(ROTATE, &args.to_string(), Some(&format!("Bearer {token}")), headers)
    }

    fn deny(&self, body: &str, authorization: Option<&str>, headers: &str, status: u16, bound: Option<usize>, recognized: usize) -> HttpReply {
        let before = self.observe();
        let counts = self.observations.counts();
        let reply = self.post(ROTATE, body, authorization, headers);
        assert_refused(&reply, status, &before);
        assert_eq!(self.observe(), before, "refusal preserves every caller/target/pending/revision SQL value");
        let after = self.observations.counts();
        assert_eq!(after.native, counts.native, "paired or invalid credentials must never fall back to native call");
        assert_eq!(after.recognized, counts.recognized + recognized);
        if let Some(bound) = bound {
            assert_eq!(after.bound, counts.bound + bound);
        } else {
            assert!((counts.bound..=counts.bound + 1).contains(&after.bound), "at most one bound dispatch");
        }
        self.assert_all_current_tokens(&before);
        reply
    }

    fn assert_rotation(&self, before: &StoredState, reply: &HttpReply, device_id: &str, role: &str, delivery: &str, scopes: &[&str]) -> String {
        assert_eq!(reply.status, 200, "{}", reply.raw);
        assert_eq!(reply.body["deviceId"], device_id);
        assert_eq!(reply.body["role"], role);
        assert_eq!(reply.body["tokenDelivery"], delivery);
        assert!(reply.body["rotatedAtMs"].as_i64().is_some());
        let after = self.observe();
        assert_eq!(after.revision_number(), before.revision_number() + 1, "one committed replacement");
        assert_eq!(reply.body["revision"].as_u64(), Some(after.revision_number()));
        assert_eq!(after.pending, before.pending, "rotation preserves pending pairing rows");
        assert_eq!(after.paired.len(), before.paired.len());
        for old_row in &before.paired {
            let id = sql_text(old_row, "device_id");
            if id != device_id {
                assert_eq!(after.device(id), old_row, "unaddressed device remains byte-for-byte intact");
            }
        }
        let mut identity_before = before.device(device_id).clone();
        let mut identity_after = after.device(device_id).clone();
        for column in ["tokens_json", "node_surface_json", "pending_node_surface_json"] {
            identity_before.remove(column);
            identity_after.remove(column);
        }
        assert_eq!(identity_after, identity_before, "roles, approval baseline, identity and timestamps remain intact");
        for column in ["node_surface_json", "pending_node_surface_json"] {
            assert_eq!(after.device(device_id).get(column), Some(&SqlValue::Null));
        }
        let old_tokens = before.tokens(device_id);
        let new_tokens = after.tokens(device_id);
        assert_eq!(new_tokens.as_object().expect("new token map").len(), old_tokens.as_object().expect("old token map").len());
        for (other_role, entry) in old_tokens.as_object().expect("old token map") {
            if other_role != role {
                assert_eq!(&new_tokens[other_role], entry, "nonselected role token and metadata remain intact");
            }
        }
        let token = after.token(device_id, role);
        assert!(token.starts_with("sand-device-tok-"));
        assert_ne!(token, before.token(device_id, role));
        assert_eq!(new_tokens[role]["role"], role);
        assert_eq!(new_tokens[role]["scopes"], json!(scopes));
        assert_eq!(new_tokens[role]["rotatedAtMs"], reply.body["rotatedAtMs"]);
        before.assert_no_secrets_in(reply);
        if delivery == "in-band" {
            assert_eq!(reply.body["token"].as_str(), Some(token.as_str()));
        } else {
            assert!(reply.body.get("token").is_none());
            after.assert_no_secrets_in(reply);
        }
        self.assert_retired(&before.token(device_id, role));
        self.assert_all_current_tokens(&after);
        token
    }

    fn install_gate(&mut self, count: usize) -> GateControl {
        let (sender, entered) = mpsc::sync_channel(count);
        let gate = Arc::new(HookGate {
            remaining: AtomicUsize::new(count), entered: sender,
            released: Mutex::new(false), changed: Condvar::new(),
        });
        *self.api.as_ref().expect("adapter").gate.lock().expect("install hook barrier") = Some(Arc::clone(&gate));
        self.gate = Some(Arc::clone(&gate));
        GateControl { gate, entered }
    }

    fn spawn_rotation(&self, gate: &GateControl, args: Value, token: &str) -> PendingHttp {
        let port = self.server.as_ref().expect("owned server").port();
        let body = args.to_string();
        let bearer = format!("Bearer {token}");
        let (sender, result) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let reply = post_to_port(port, ROTATE, &body, Some(&bearer), "");
            let _ = sender.send(reply);
        });
        PendingHttp { result, thread: Some(handle), gate: Arc::clone(&gate.gate) }
    }
}

impl Drop for PairingGateway {
    fn drop(&mut self) {
        if self.stop() {
            let result = fs::remove_dir_all(&self.root);
            if !thread::panicking() {
                result.expect("remove only the closed owned HTTP fixture directory");
            }
        } else {
            eprintln!("DPA fixture retained because owner shutdown was not acknowledged: {}", self.root.display());
            assert!(thread::panicking(), "bounded owned Gateway shutdown failed");
        }
    }
}

fn assert_refused(reply: &HttpReply, status: u16, state: &StoredState) {
    assert_eq!(reply.status, status, "{}", reply.raw);
    assert!(reply.body["error"].as_str().is_some_and(|error| !error.is_empty()), "HTTP error projection");
    assert!(reply.body.get("token").is_none());
    assert!(reply.body.get("revision").is_none(), "a refusal cannot acknowledge a mutation");
    state.assert_no_secrets_in(reply);
}

fn assert_bound_once(fixture: &PairingGateway, before: Counts) {
    assert_eq!(fixture.observations.counts(), Counts {
        native: before.native, bound: before.bound + 1, recognized: before.recognized + 1,
    });
}

#[test]
fn paired_http_self_rotation_derives_delivery_from_identity_and_survives_reopen() {
    for (cross_device, desired) in [(None, None), (Some(false), Some(vec!["operator.read"])), (Some(true), Some(vec![]))] {
        let mut fixture = PairingGateway::seeded("operator", &["operator.write"]);
        let before = fixture.observe();
        let old = before.token(CALLER, "operator");
        let mut args = json!({"deviceId": CALLER});
        if let Some(value) = cross_device {
            args["crossDevice"] = json!(value);
        }
        if let Some(scopes) = desired.as_ref() {
            args["scopes"] = json!(scopes);
        }
        let counts = fixture.observations.counts();
        let reply = fixture.rotate(&args, &old, "x-sand-request-id: dpa-self\r\n");
        assert_bound_once(&fixture, counts);
        let expected_scopes = desired.as_deref().unwrap_or(&["operator.write"]);
        let new_token = fixture.assert_rotation(&before, &reply, CALLER, "operator", "in-band", expected_scopes);
        assert_eq!(&*fixture.observations.contexts.lock().expect("contexts"), &[(true, Some("dpa-self".to_string()))]);
        {
            let reports = fixture.observations.completed.lock().expect("reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].method, ROTATE);
            assert_eq!(reports[0].status, 200);
            assert_eq!(reports[0].request_id.as_deref(), Some("dpa-self"));
        }
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        fixture.assert_all_current_tokens(&committed);
        fixture.assert_retired(&old);
        fixture.assert_authorized(&new_token, CALLER, "operator", &json!(expected_scopes));
        fixture.deny(&args.to_string(), Some(&format!("Bearer {old}")), "", 401, Some(0), 0);
    }
}

#[test]
fn paired_http_write_cannot_rotate_other_devices_roles_or_forge_authority() {
    let fixture = PairingGateway::seeded("operator", &["operator.write"]);
    let bearer = format!("Bearer {}", fixture.observe().token(CALLER, "operator"));
    let ordinary = fixture.deny(&json!({"deviceId": TARGET}).to_string(), Some(&bearer), "", 403, Some(1), 1);
    let absent = fixture.deny(&json!({"deviceId": "dpa-unknown-target"}).to_string(), Some(&bearer), "", 403, Some(1), 1);
    assert_eq!(ordinary.body, absent.body, "unauthorized cross-device requests must not reveal target existence");
    for args in [
        json!({"deviceId": TARGET, "crossDevice": false}),
        json!({"deviceId": TARGET, "crossDevice": true, "callerDeviceId": TARGET,
            "callerRole": "operator", "isAdmin": true, "admin": true,
            "authorization": {"deviceId": TARGET, "role": "operator", "scopes": ["operator.admin"]}}),
        json!({"deviceId": CALLER, "role": "node", "callerRole": "operator", "isAdmin": true}),
        json!({"deviceId": TARGET, "role": "node"}),
        json!({"deviceId": CALLER, "scopes": ["operator.admin"]}),
    ] {
        for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
            fixture.deny(&args.to_string(), Some(&bearer), headers, 403, Some(1), 1);
        }
    }
    for args in [
        json!({"deviceId": CALLER, "role": 7}),
        json!({"deviceId": CALLER, "scopes": "operator.write"}),
        json!({"deviceId": CALLER, "scopes": [null]}),
        json!({"deviceId": CALLER, "crossDevice": "false"}),
    ] {
        fixture.deny(&args.to_string(), Some(&bearer), "", 400, Some(1), 1);
    }
    fixture.deny("{", Some(&bearer), "", 400, Some(0), 1);
}

#[test]
fn paired_http_admin_only_rotates_operator_and_node_with_identity_bound_disclosure() {
    let mut fixture = PairingGateway::seeded("operator", &["operator.admin"]);
    let admin = fixture.observe().token(CALLER, "operator");
    let bearer = format!("Bearer {admin}");
    for headers in [
        "x-sand-scope: operator.write\r\n", "x-sand-scope: operator.read\r\n",
        "x-sand-scope:\r\n", "x-sand-scope: operator.write operator.admin\r\n",
        "x-sand-role: node\r\nx-sand-scope: operator.admin\r\n",
    ] {
        fixture.deny(&json!({"deviceId": TARGET}).to_string(), Some(&bearer), headers, 403, None, 1);
    }
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&bearer),
        "x-sand-scope: operator.write\r\n", 403, Some(1), 1);
    for (role, scopes) in [("operator", vec!["operator.read", "operator.write"]), ("node", vec!["node.invoke"])] {
        let before = fixture.observe();
        let counts = fixture.observations.counts();
        let reply = fixture.rotate(&json!({
            "deviceId": TARGET, "role": role, "crossDevice": false, "callerDeviceId": TARGET,
        }), &admin, "x-sand-scope: operator.admin\r\n");
        assert_bound_once(&fixture, counts);
        fixture.assert_rotation(&before, &reply, TARGET, role, "withheld-cross-device", &scopes);
    }
    let before = fixture.observe();
    let counts = fixture.observations.counts();
    let reply = fixture.rotate(&json!({"deviceId": CALLER, "crossDevice": true}), &admin, "");
    assert_bound_once(&fixture, counts);
    fixture.assert_rotation(&before, &reply, CALLER, "operator", "in-band", &["operator.admin"]);
    let committed = fixture.observe();
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);
    fixture.assert_all_current_tokens(&committed);
    fixture.assert_retired(&admin);
}

#[test]
fn paired_http_read_node_unknown_and_unrelated_grants_cannot_gain_rotation_authority() {
    for (role, scopes) in [
        ("operator", vec!["operator.read"]), ("operator", vec![]),
        ("operator", vec!["operator.sessions.write"]),
        ("node", vec!["node.invoke"]), ("observer", vec!["observer.read"]),
    ] {
        let mut fixture = PairingGateway::seeded(role, &scopes);
        let bearer = format!("Bearer {}", fixture.observe().token(CALLER, role));
        let body = json!({"deviceId": CALLER, "isAdmin": true, "callerRole": "operator"}).to_string();
        for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
            fixture.deny(&body, Some(&bearer), headers, 403, None, 1);
        }
        let before = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), before);
        fixture.deny(&body, Some(&bearer), "", 403, None, 1);
    }
}

#[test]
fn paired_http_explicit_failed_credentials_never_become_native_and_public_controls_remain() {
    for configured in [false, true] {
        let fixture = PairingGateway::new(configured, "127.0.0.1", false);
        fixture.seed("operator", &["operator.write"]);
        let retired = fixture.observe().token(CALLER, "operator");
        fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER})).expect("retire old caller by replacement");
        fixture.approve("dpa-removed", &["operator"], &["operator.write"]);
        let removed = fixture.observe().token("dpa-removed", "operator");
        fixture.owner().handle_pair_revoke(&json!({"deviceId": "dpa-removed"})).expect("remove another real credential");
        let retired_bearer = format!("Bearer {retired}");
        let removed_bearer = format!("Bearer {removed}");
        let body = json!({"deviceId": TARGET}).to_string();
        for authorization in ["Bearer wrong-token", "Bearer", "Bearer ", "", "Basic ignored", &retired_bearer, &removed_bearer] {
            for request_body in [body.as_str(), "{"] {
                fixture.deny(request_body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n",
                    401, Some(0), 0);
            }
        }
        if configured {
            fixture.deny(&body, None, "", 401, Some(0), 0);
        }
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        for cross_device in [false, true] {
            let before = fixture.observe();
            let counts = fixture.observations.counts();
            let reply = fixture.post(ROTATE, &json!({"deviceId": TARGET, "crossDevice": cross_device}).to_string(), native.as_deref(), "");
            assert_eq!(fixture.observations.counts(), Counts { native: counts.native + 1, ..counts });
            fixture.assert_rotation(&before, &reply, TARGET, "operator",
                if cross_device { "withheld-cross-device" } else { "in-band" }, &["operator.read", "operator.write"]);
        }
        let before = fixture.observe();
        let current = before.token(CALLER, "operator");
        let counts = fixture.observations.counts();
        let reply = fixture.rotate(&json!({"deviceId": CALLER}), &current, "");
        assert_bound_once(&fixture, counts);
        fixture.assert_rotation(&before, &reply, CALLER, "operator", "in-band", &["operator.write"]);
        let before_public = fixture.observe();
        let counts = fixture.observations.counts();
        let pending = fixture.post("device.pair.request", &json!({
            "deviceId": "dpa-public", "publicKey": "dpa-public-key", "scopes": ["operator.read"],
        }).to_string(), None, "");
        assert_eq!(pending.status, 200);
        assert_eq!(pending.body["status"], "pending");
        assert!(pending.body["requestId"].as_str().is_some());
        assert!(pending.body.get("token").is_none());
        before_public.assert_no_secrets_in(&pending);
        let after_public = fixture.observe();
        assert_eq!(after_public.paired, before_public.paired);
        assert_eq!(after_public.revision, before_public.revision);
        assert_eq!(after_public.pending.len(), before_public.pending.len() + 1);
        for row in &before_public.pending {
            assert!(after_public.pending.contains(row));
        }
        let status = fixture.post("device.pair.status", &json!({"requestId": pending.body["requestId"]}).to_string(), None, "");
        assert_eq!(status.status, 200);
        assert_eq!(status.body, json!({"status": "pending", "requestId": pending.body["requestId"]}));
        after_public.assert_no_secrets_in(&status);
        assert_eq!(fixture.observe(), after_public);
        assert_eq!(fixture.observations.counts(), Counts { native: counts.native + 2, ..counts });
    }
    let fixture = PairingGateway::new(false, "0.0.0.0", false);
    fixture.seed("operator", &["operator.write"]);
    // post() sends a loopback Host header and connects through loopback; neither
    // may turn an actual wildcard listener into credential-absent native mode.
    fixture.deny(&json!({"deviceId": TARGET}).to_string(), None, "", 401, Some(0), 0);
    let before = fixture.observe();
    let counts = fixture.observations.counts();
    let reply = fixture.rotate(&json!({"deviceId": CALLER}), &before.token(CALLER, "operator"), "");
    assert_bound_once(&fixture, counts);
    fixture.assert_rotation(&before, &reply, CALLER, "operator", "in-band", &["operator.write"]);
}

#[test]
fn paired_http_default_bound_hook_refuses_without_legacy_native_dispatch() {
    let fixture = PairingGateway::new(false, "127.0.0.1", true);
    fixture.seed("operator", &["operator.write"]);
    let before = fixture.observe();
    let bearer = format!("Bearer {}", before.token(CALLER, "operator"));
    let args = json!({"deviceId": CALLER});
    fixture.deny(&args.to_string(), Some(&bearer), "", 403, Some(0), 1);
    let counts = fixture.observations.counts();
    let reply = fixture.post(ROTATE, &args.to_string(), None, "");
    assert_eq!(fixture.observations.counts(), Counts { native: counts.native + 1, ..counts });
    let current = fixture.assert_rotation(&before, &reply, CALLER, "operator", "in-band", &["operator.write"]);
    fixture.deny(&args.to_string(), Some(&format!("Bearer {current}")), "", 403, Some(0), 1);
}

#[test]
fn paired_http_rechecks_retirement_after_authentication_with_live_and_unrelated_controls() {
    for retirement in ["unchanged", "replace", "remove", "unrelated"] {
        let mut fixture = PairingGateway::seeded("operator", &["operator.admin"]);
        let before = fixture.observe();
        let caller_token = before.token(CALLER, "operator");
        let counts = fixture.observations.counts();
        let gate = fixture.install_gate(1);
        let request = fixture.spawn_rotation(&gate, json!({"deviceId": TARGET}), &caller_token);
        gate.wait_for_entries(1);
        assert_bound_once(&fixture, counts);
        assert_eq!(fixture.observe(), before, "actual authenticated request is paused before owner admission");
        match retirement {
            "replace" => {
                fixture.owner().handle_pair_rotate_token(&json!({"deviceId": CALLER}))
                    .expect("replacement commits while HTTP hook is paused");
                fixture.assert_retired(&caller_token);
            }
            "remove" => {
                fixture.owner().handle_pair_revoke(&json!({"deviceId": CALLER}))
                    .expect("caller removal commits while HTTP hook is paused");
                fixture.assert_retired(&caller_token);
            }
            "unrelated" => {
                fixture.owner().handle_pair_rotate_token(&json!({"deviceId": TARGET, "role": "node"}))
                    .expect("unrelated role replacement advances revision without retiring caller");
            }
            "unchanged" => {}
            _ => unreachable!(),
        }
        let after_retirement = fixture.observe();
        if matches!(retirement, "replace" | "remove") {
            assert_eq!(after_retirement.device(TARGET), before.device(TARGET));
            assert_eq!(after_retirement.revision_number(), before.revision_number() + 1);
        }
        gate.release();
        let reply = request.finish();
        assert_bound_once(&fixture, counts);
        if matches!(retirement, "replace" | "remove") {
            assert_refused(&reply, 401, &after_retirement);
            assert!(!reply.raw.contains(&caller_token));
            assert_eq!(fixture.observe(), after_retirement,
                "retired admitted request cannot touch target, pending rows or post-retirement revision");
            fixture.assert_all_current_tokens(&after_retirement);
        } else {
            fixture.assert_rotation(&after_retirement, &reply, TARGET, "operator", "withheld-cross-device",
                &["operator.read", "operator.write"]);
        }
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        fixture.assert_all_current_tokens(&committed);
        if matches!(retirement, "replace" | "remove") {
            fixture.assert_retired(&caller_token);
        }
    }
}

#[test]
fn paired_http_two_admitted_self_rotations_commit_exactly_one_replacement() {
    let mut fixture = PairingGateway::seeded("operator", &["operator.write"]);
    let before = fixture.observe();
    let original = before.token(CALLER, "operator");
    let counts = fixture.observations.counts();
    let gate = fixture.install_gate(2);
    let first = fixture.spawn_rotation(&gate, json!({"deviceId": CALLER}), &original);
    let second = fixture.spawn_rotation(&gate, json!({"deviceId": CALLER, "crossDevice": true}), &original);
    gate.wait_for_entries(2);
    assert_eq!(fixture.observations.counts(), Counts {
        native: counts.native, bound: counts.bound + 2, recognized: counts.recognized + 2,
    });
    assert_eq!(fixture.observe(), before, "both old credentials were admitted before either mutation");
    gate.release();
    let replies = [first.finish(), second.finish()];
    let mut statuses = [replies[0].status, replies[1].status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 401]);
    let success = replies.iter().find(|reply| reply.status == 200).expect("one success");
    let refused = replies.iter().find(|reply| reply.status == 401).expect("one retired request");
    assert_eq!(fixture.observations.counts(), Counts {
        native: counts.native, bound: counts.bound + 2, recognized: counts.recognized + 2,
    });
    let replacement = fixture.assert_rotation(&before, success, CALLER, "operator", "in-band", &["operator.write"]);
    let committed = fixture.observe();
    assert_refused(refused, 401, &committed);
    assert!(!refused.raw.contains(&original));
    assert_eq!(fixture.observations.completed.lock().expect("success reports").len(), 1);
    fixture.reopen();
    assert_eq!(fixture.observe(), committed);
    fixture.assert_all_current_tokens(&committed);
    fixture.assert_retired(&original);
    fixture.assert_authorized(&replacement, CALLER, "operator", &json!(["operator.write"]));
    fixture.deny(&json!({"deviceId": CALLER}).to_string(), Some(&format!("Bearer {original}")), "", 401, Some(0), 0);
}
