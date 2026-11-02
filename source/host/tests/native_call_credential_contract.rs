// Authored contract coverage for FBCP-CALL-CREDENTIAL-001. Runtime validation belongs to Actions.
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::memory::memory_service::MemoryService;
use mahayana_host_runtime::extensions::session::agent_db::SandAgentDb;
use mahayana_host_runtime::extensions::session::call_session::CallSessionStore;
use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
};
use mahayana_host_runtime::extensions::session::native_messaging::FabushiNativeMessagingClient;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use uuid::Uuid;

const SERVICE_TIMEOUT: Duration = Duration::from_secs(45);
const BARRIER_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
struct HttpRequest {
    method: String,
    target: String,
    body: Value,
    bearer: String,
    device_id: String,
}

struct RemoteCall {
    value: Value,
    events: Vec<Value>,
}

struct HoldPlan {
    method: String,
    path: String,
    reached: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

#[derive(Default)]
struct ServiceState {
    accepted_connections: usize,
    requests: Vec<HttpRequest>,
    calls: BTreeMap<String, RemoteCall>,
    hold_next: Option<HoldPlan>,
}

struct ResponseBarrier {
    reached: mpsc::Receiver<()>,
    release: Option<mpsc::Sender<()>>,
}

impl ResponseBarrier {
    fn wait_until_held(&self) {
        self.reached.recv_timeout(BARRIER_TIMEOUT)
            .expect("fixture withheld the final response-body byte");
    }

    fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

impl Drop for ResponseBarrier {
    fn drop(&mut self) {
        self.release();
    }
}

struct PendingGatewayCall {
    barrier: ResponseBarrier,
    worker: Option<thread::JoinHandle<Result<Value, SessionGatewayError>>>,
}

impl PendingGatewayCall {
    fn start(
        runtime: Arc<ProductionSessionWorkers>,
        method: &'static str,
        args: Value,
        barrier: ResponseBarrier,
    ) -> Self {
        let worker = thread::spawn(move || dispatch(&runtime, method, args));
        Self { barrier, worker: Some(worker) }
    }

    fn wait_until_held(&self) {
        self.barrier.wait_until_held();
    }

    fn complete(mut self) -> Result<Value, SessionGatewayError> {
        self.barrier.release();
        self.worker.take().expect("pending gateway worker")
            .join().expect("gateway worker returned without panic")
    }
}

impl Drop for PendingGatewayCall {
    fn drop(&mut self) {
        // Assertions can unwind while a response is held. Always release before joining.
        self.barrier.release();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct CallService {
    base_url: String,
    state: Arc<Mutex<ServiceState>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<(), String>>>,
}

impl CallService {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind call credential fixture");
        let base_url = format!("http://{}", listener.local_addr().expect("fixture address"));
        listener.set_nonblocking(true).expect("bounded fixture accept");
        let state = Arc::new(Mutex::new(ServiceState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + SERVICE_TIMEOUT;
            loop {
                if Instant::now() >= deadline {
                    return Err("call credential fixture exceeded its accept deadline".into());
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false)
                            .map_err(|error| format!("restore blocking fixture stream: {error}"))?;
                        worker_state.lock().map_err(|_| "fixture state poisoned")?
                            .accepted_connections += 1;
                        let request = read_request(&mut stream, deadline)?;
                        let (body, hold) = {
                            let mut state = worker_state.lock().map_err(|_| "fixture state poisoned")?;
                            state.requests.push(request.clone());
                            let body = reply(&mut state, &request)?;
                            let path = request.target.split('?').next().unwrap_or(&request.target);
                            let matches_hold = state.hold_next.as_ref()
                                .map(|hold| hold.method == request.method && hold.path == path)
                                .unwrap_or(false);
                            let hold = if matches_hold { state.hold_next.take() } else { None };
                            (body, hold)
                        };
                        write_response(&mut stream, &body, hold)?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        // Drain connections already admitted by the listener before stopping.
                        if worker_stop.load(Ordering::SeqCst) {
                            return Ok(());
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => return Err(format!("accept credential fixture request: {error}")),
                }
            }
        });
        Self { base_url, state, stop, worker: Some(worker) }
    }

    fn hold(&self, method: &str, path: &str) -> ResponseBarrier {
        let (reached_tx, reached_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let mut state = self.state.lock().expect("fixture state");
        assert!(state.hold_next.is_none(), "only one response barrier may be armed");
        state.hold_next = Some(HoldPlan {
            method: method.into(), path: path.into(), reached: reached_tx, release: release_rx,
        });
        ResponseBarrier { reached: reached_rx, release: Some(release_tx) }
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.state.lock().expect("fixture state").requests.clone()
    }

    fn accepted_connections(&self) -> usize {
        self.state.lock().expect("fixture state").accepted_connections
    }

    fn event_posts(&self) -> usize {
        self.requests().iter().filter(|request| {
            request.method == "POST" && request.target.ends_with("/events")
        }).count()
    }

    fn create_posts(&self) -> usize {
        self.requests().iter().filter(|request| {
            request.method == "POST" && request.target == "/api/social/calls"
        }).count()
    }

    fn finish(&mut self) -> usize {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("credential fixture thread")
                .expect("credential fixture protocol and cleanup");
        }
        let state = self.state.lock().expect("fixture state");
        assert!(state.hold_next.is_none(), "every armed response barrier was reached");
        state.accepted_connections
    }
}

impl Drop for CallService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn read_request(stream: &mut TcpStream, service_deadline: Instant) -> Result<HttpRequest, String> {
    let deadline = service_deadline.min(Instant::now() + Duration::from_secs(5));
    let mut bytes = Vec::new();
    let header_end = loop {
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        read_more(stream, &mut bytes, deadline)?;
    };
    let headers = std::str::from_utf8(&bytes[..header_end])
        .map_err(|error| format!("fixture request headers: {error}"))?.to_string();
    let mut lines = headers.split("\r\n");
    let mut first = lines.next().ok_or("request line missing")?.split_whitespace();
    let method = first.next().ok_or("request method missing")?.to_string();
    let target = first.next().ok_or("request target missing")?.to_string();
    let headers = lines.filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_string()))
        .collect::<BTreeMap<_, _>>();
    let bearer = headers.get("authorization").ok_or("Native bearer header missing")?.clone();
    let device_id = headers.get("x-fabushi-device-id").ok_or("Native device header missing")?.clone();
    let length = headers.get("content-length")
        .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
        .transpose()?.unwrap_or(0);
    if length > 64 * 1024 {
        return Err("credential fixture body exceeded its bound".into());
    }
    while bytes.len() < header_end + length {
        read_more(stream, &mut bytes, deadline)?;
    }
    let body = if length == 0 {
        Value::Null
    } else {
        serde_json::from_slice(&bytes[header_end..header_end + length])
            .map_err(|error| format!("fixture request JSON: {error}"))?
    };
    Ok(HttpRequest { method, target, body, bearer, device_id })
}

fn read_more(stream: &mut TcpStream, bytes: &mut Vec<u8>, deadline: Instant) -> Result<(), String> {
    let remaining = deadline.checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero()).ok_or("credential fixture read deadline")?;
    stream.set_read_timeout(Some(remaining)).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 4096];
    let count = stream.read(&mut buffer).map_err(|error| format!("read credential fixture: {error}"))?;
    if count == 0 {
        return Err("credential fixture request ended before its complete frame".into());
    }
    bytes.extend_from_slice(&buffer[..count]);
    if bytes.len() > 128 * 1024 {
        return Err("credential fixture request exceeded its frame bound".into());
    }
    Ok(())
}

fn write_response(stream: &mut TcpStream, body: &Value, hold: Option<HoldPlan>) -> Result<(), String> {
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|error| error.to_string())?;
    let body = body.to_string();
    write!(stream,
        "HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len(),
    ).map_err(|error| format!("write credential fixture headers: {error}"))?;
    let bytes = body.as_bytes();
    if let Some(hold) = hold {
        let split = bytes.len().checked_sub(1).ok_or("held response body is empty")?;
        stream.write_all(&bytes[..split]).map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
        hold.reached.send(()).map_err(|_| "response barrier controller disappeared")?;
        hold.release.recv_timeout(BARRIER_TIMEOUT)
            .map_err(|error| format!("bounded response barrier: {error}"))?;
        stream.write_all(&bytes[split..]).map_err(|error| error.to_string())?;
    } else {
        stream.write_all(bytes).map_err(|error| error.to_string())?;
    }
    stream.flush().map_err(|error| error.to_string())
}

fn remote_call(call_id: &str, creator: &str, peer: &str, device_id: &str) -> Value {
    json!({
        "callId": call_id, "creatorUserId": creator, "peerUserId": peer,
        "creatorDeviceId": device_id, "peerDeviceId": null,
        "state": "invited", "generation": 0, "eventSeq": 0, "terminalState": null,
        "createdAt": "2026-11-02T00:00:00Z", "updatedAt": "2026-11-02T00:00:00Z",
    })
}

fn reply(state: &mut ServiceState, request: &HttpRequest) -> Result<Value, String> {
    let url = url::Url::parse(&format!("http://fixture{}", request.target))
        .map_err(|error| error.to_string())?;
    if request.method == "GET" && url.path() == "/api/social/calls/ice" {
        return Ok(json!({
            "success": true,
            "iceServers": [{
                "urls": ["turn:relay.example.invalid:3478"],
                "username": "fixture-ice-user", "credential": "fixture-ice-secret",
            }],
            "ttlSeconds": 3600,
        }));
    }
    if request.method == "GET" && url.path() == "/api/social/calls" {
        let calls = state.calls.values().map(|call| call.value.clone()).collect::<Vec<_>>();
        return Ok(json!({"success": true, "calls": calls}));
    }
    if request.method == "POST" && url.path() == "/api/social/calls" {
        let call_id = request.body.get("callId").and_then(Value::as_str)
            .ok_or("create callId missing")?;
        let peer = request.body.get("targetUserId").and_then(Value::as_str)
            .ok_or("create targetUserId missing")?;
        if state.calls.contains_key(call_id) {
            return Err("fixture creation unexpectedly reused a call id".into());
        }
        let value = remote_call(call_id, "1", peer, &request.device_id);
        // The remote mutation precedes a held response; releasing it cannot roll this back.
        state.calls.insert(call_id.into(), RemoteCall { value: value.clone(), events: Vec::new() });
        return Ok(json!({"success": true, "call": value}));
    }
    let suffix = url.path().strip_prefix("/api/social/calls/")
        .ok_or_else(|| format!("unexpected credential fixture path: {}", request.target))?;
    if request.method == "POST" && suffix.ends_with("/events") {
        let call_id = suffix.trim_end_matches("/events");
        let call = state.calls.get_mut(call_id).ok_or("unknown fixture call")?;
        let kind = request.body.get("kind").and_then(Value::as_str).ok_or("event kind missing")?;
        let generation = request.body.get("generation").and_then(Value::as_u64)
            .ok_or("event generation missing")?;
        let client_event_id = request.body.get("clientEventId").and_then(Value::as_str)
            .ok_or("clientEventId missing")?;
        let payload = request.body.get("payload").ok_or("event payload missing")?;
        match kind {
            "transition" => {
                let next = payload.get("state").and_then(Value::as_str)
                    .ok_or("transition state missing")?;
                if payload.get("action").and_then(Value::as_str) != Some("ring") || next != "ringing" {
                    return Err("fixture only admits the expected ring transition".into());
                }
                call.value["state"] = json!(next);
                call.value["generation"] = json!(generation);
            }
            "signal" => {
                if payload.get("senderDeviceId").and_then(Value::as_str)
                    != Some(request.device_id.as_str())
                {
                    return Err("signal body differs from the Native dispatch device".into());
                }
            }
            _ => return Err(format!("unexpected posted event kind: {kind}")),
        }
        let seq = call.events.len() as u64 + 1;
        let event = json!({
            "callId": call_id, "seq": seq, "generation": generation,
            "userId": "1", "deviceId": request.device_id, "clientEventId": client_event_id,
            "kind": kind, "payload": payload, "createdAt": "2026-11-02T00:00:01Z",
        });
        call.events.push(event.clone());
        call.value["eventSeq"] = json!(seq);
        return Ok(json!({"success": true, "call": call.value, "event": event}));
    }
    if request.method == "GET" {
        let call = state.calls.get(suffix).ok_or("unknown fixture call")?;
        let after_seq = url.query_pairs().find(|(key, _)| key == "afterSeq")
            .ok_or("afterSeq missing")?.1.parse::<u64>().map_err(|error| error.to_string())?;
        let limit = url.query_pairs().find(|(key, _)| key == "limit")
            .ok_or("limit missing")?.1.parse::<usize>().map_err(|error| error.to_string())?;
        let events = call.events.iter()
            .filter(|event| event["seq"].as_u64().unwrap() > after_seq)
            .take(limit).cloned().collect::<Vec<_>>();
        let next = events.last().and_then(|event| event["seq"].as_u64()).unwrap_or(after_seq);
        return Ok(json!({"success": true, "call": call.value, "events": events, "nextAfterSeq": next}));
    }
    Err(format!("unexpected credential fixture request: {} {}", request.method, request.target))
}

struct Fixture {
    root: PathBuf,
    agents_root: PathBuf,
    credential_path: PathBuf,
    original_credential: Value,
    runtime: Option<Arc<ProductionSessionWorkers>>,
    client: Arc<FabushiNativeMessagingClient>,
    service: CallService,
    scope_id: String,
    conversation_path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-native-call-credential-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("credential fixture root");
        let agents_root = root.join("agents");
        let service = CallService::start();
        let credential_path = root.join("host-credential.json");
        let original_credential = json!({
            "accessToken": "call-credential-fixture", "deviceId": "device-a", "userId": 1,
            "sessionId": "fixture-session-a", "expiresAtMs": 4_102_444_800_000_u64,
            "issuer": service.base_url,
        });
        write_credential(&credential_path, &original_credential);
        let client = Arc::new(FabushiNativeMessagingClient::new(&service.base_url, credential_path.clone())
            .expect("Native call client"));
        let local = workers(&agents_root, None);
        let scope_id = local.create_human_conversation("2", "Peer two").expect("canonical scope")
            ["id"].as_str().expect("scope id").to_string();
        local.append_human_message(
            &scope_id, "Preserved conversation content", "credential-fixture-message", Some(1_000.0), None, &[],
        ).expect("seed actual local conversation transcript");
        let conversation_path = local.open_human_conversation_db_owner(&scope_id)
            .expect("conversation owner").db_path().to_path_buf();
        local.shutdown();
        drop(local);
        let runtime = Some(workers(&agents_root, Some(Arc::clone(&client))));
        Self {
            root, agents_root, credential_path, original_credential, runtime, client,
            service, scope_id, conversation_path,
        }
    }

    fn runtime(&self) -> &Arc<ProductionSessionWorkers> {
        self.runtime.as_ref().expect("live credential fixture runtime")
    }

    fn create_call(&self) -> String {
        let call = dispatch(self.runtime(), "createCallSession", json!({
            "scopeId": self.scope_id, "participantIds": ["1", "2"],
        })).expect("create admitted call through the shipping gateway");
        assert_eq!(call["state"], "invited");
        assert_eq!(call["generation"], 0);
        call["id"].as_str().expect("created call id").to_string()
    }

    fn rotate(&self, field: &str, value: Value) {
        let mut credential = self.original_credential.clone();
        credential[field] = value;
        write_credential(&self.credential_path, &credential);
    }

    fn restore(&self) {
        write_credential(&self.credential_path, &self.original_credential);
    }

    fn reopen(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown();
            drop(runtime);
        }
        self.runtime = Some(workers(&self.agents_root, Some(Arc::clone(&self.client))));
    }

    fn pending(&self, method: &'static str, args: Value, http_method: &str, path: &str) -> PendingGatewayCall {
        let barrier = self.service.hold(http_method, path);
        PendingGatewayCall::start(Arc::clone(self.runtime()), method, args, barrier)
    }

    fn remote_call(&self, call_id: &str) -> Value {
        self.service.state.lock().expect("fixture state").calls.get(call_id)
            .expect("accepted remote call").value.clone()
    }

    fn insert_incoming_call(&self, call_id: &str) {
        self.service.state.lock().expect("fixture state").calls.insert(call_id.into(), RemoteCall {
            value: remote_call(call_id, "3", "1", "device-peer"), events: Vec::new(),
        });
    }

    fn append_remote_media(&self, call_id: &str) {
        let mut state = self.service.state.lock().expect("fixture state");
        let call = state.calls.get_mut(call_id).expect("media event call");
        let seq = call.events.len() as u64 + 1;
        call.events.push(json!({
            "callId": call_id, "seq": seq, "generation": 0,
            "userId": "2", "deviceId": "device-peer", "clientEventId": "remote-media-fixture",
            "kind": "media", "payload": {
                "mediaCapabilities": {"audio": true, "video": true},
                "deviceSelection": {"microphoneId": "fixture-remote-microphone"},
            },
            "createdAt": "2026-11-02T00:00:02Z",
        }));
        call.value["eventSeq"] = json!(seq);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown();
            drop(runtime);
        }
        self.service.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.service.worker.take() {
            let _ = worker.join();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_credential(path: &Path, credential: &Value) {
    // All rewrites happen before dispatch or behind an acknowledged response-body barrier.
    fs::write(path, credential.to_string()).expect("write synthetic private credential");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture credential");
    }
}

fn workers(agents_root: &Path, client: Option<Arc<FabushiNativeMessagingClient>>) -> Arc<ProductionSessionWorkers> {
    let native_required = client.is_some();
    Arc::new(ProductionSessionWorkers::with_agents_root_identity_messaging_and_dependencies(
        agents_root, 500, Some("1".into()), client, native_required, None,
        Arc::new(|| None), Arc::new(MemoryService::new(agents_root.to_path_buf())),
    ))
}

fn dispatch(runtime: &Arc<ProductionSessionWorkers>, method: &str, args: Value) -> Result<Value, SessionGatewayError> {
    dispatch_production_session_gateway_call(runtime, method, &args).expect("registered Session method")
}

fn conversation_snapshot(path: &Path) -> (Value, Vec<Value>) {
    let owner = SandAgentDb::open_conversation_store(path, 500).expect("reopen conversation SQLite");
    let snapshot = (
        owner.read_metadata().expect("persisted conversation metadata"),
        owner.get_transcript_entries().expect("persisted conversation transcript"),
    );
    owner.close(false);
    snapshot
}

type Rows = Vec<Vec<SqlValue>>;

fn call_snapshot(runtime: &Arc<ProductionSessionWorkers>) -> (Rows, Rows, Rows) {
    let connection = Connection::open_with_flags(
        runtime.call_session_store_path().expect("call store path"), OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).expect("read existing call SQLite");
    let rows = |sql: &str| {
        let mut statement = connection.prepare(sql).expect("call snapshot query");
        let columns = statement.column_count();
        let values = statement.query_map([], |row| {
            (0..columns).map(|index| row.get::<_, SqlValue>(index)).collect::<Result<Vec<_>, _>>()
        }).expect("call snapshot rows");
        values.collect::<Result<Vec<_>, _>>().expect("call snapshot values")
    };
    (
        rows("SELECT * FROM call_sessions ORDER BY id"),
        rows("SELECT * FROM call_remote_sync ORDER BY call_id"),
        rows("SELECT * FROM call_signals ORDER BY call_id, generation, seq"),
    )
}

fn stored_cursor(runtime: &Arc<ProductionSessionWorkers>, call_id: &str) -> u64 {
    CallSessionStore::open(runtime.agents_root(), 500)
        .expect("existing call owner").remote_event_seq(call_id).expect("persisted replay cursor")
}

fn assert_fixture_headers(requests: &[HttpRequest], device_id: &str) {
    assert!(requests.iter().all(|request| request.bearer == "Bearer call-credential-fixture"));
    assert!(requests.iter().all(|request| request.device_id == device_id));
}

fn assert_stale_response(result: Result<Value, SessionGatewayError>, field: &str) {
    let error = format!("{:?}", result.expect_err("a changed credential must reject the held response"));
    if field == "issuer" {
        assert!(error.contains("issuer does not match the configured service origin"), "{error}");
    } else {
        assert!(error.contains("credential changed during Human request"), "{field}: {error}");
    }
}

#[test]
fn native_call_credentials_bind_all_http_families_before_transport() {
    let mut fixture = Fixture::new();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation_path);
    assert!(before_calls.0.is_empty());
    fixture.rotate("userId", json!(2));
    let cases = [
        ("ICE", fixture.client.get_human_call_ice_servers_for_user("1").map(|_| ())),
        ("discovery", fixture.client.list_human_calls_for_user(200, "1").map(|_| ())),
        ("creation", fixture.client.create_human_call_for_user("wrong-account-call", "2", "1").map(|_| ())),
        ("retrieval", fixture.client.get_human_call_for_user("wrong-account-call", 0, 200, "1").map(|_| ())),
        ("append", fixture.client.append_human_call_event_for_user(
            "wrong-account-call", "wrong-account-event", 0, "transition",
            &json!({"action": "ring", "state": "ringing"}), "1",
        ).map(|_| ())),
    ];
    for (family, result) in cases {
        assert_eq!(
            result.expect_err("fixed Human admission must reject a different credential account"),
            "Fabushi native messaging credential changed Human account", "{family}",
        );
        assert_eq!(call_snapshot(fixture.runtime()), before_calls, "{family} durable state");
    }
    fixture.reopen();
    assert_eq!(call_snapshot(fixture.runtime()), before_calls);
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    assert_eq!(fixture.service.finish(), 0, "all five families reject before any HTTP connection");
    assert!(fixture.service.requests().is_empty());
}

#[test]
fn native_call_signal_binds_sender_to_dispatch_credential() {
    let mut fixture = Fixture::new();
    let earlier_device = fixture.client.identity().expect("earlier production identity sample").device_id;
    fixture.rotate("deviceId", json!("device-b"));
    let call_id = fixture.create_call();
    let baseline_connections = fixture.service.accepted_connections();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation_path);
    let payloads = [
        json!({"signalKind": "offer", "signal": {"sdp": "fixture-offer"}}),
        json!({"senderDeviceId": earlier_device, "signalKind": "offer", "signal": {"sdp": "fixture-offer"}}),
    ];
    for payload in payloads {
        let error = fixture.client.append_human_call_event_for_user(
            &call_id, "signal-device-negative", 0, "signal", &payload, "1",
        ).expect_err("missing or stale sender cannot use the newly loaded credential");
        assert_eq!(error, "call signal senderDeviceId must match the authenticated Host device");
        assert_eq!(fixture.service.accepted_connections(), baseline_connections);
        assert_eq!(fixture.service.event_posts(), 0);
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
    }
    let matching = json!({
        "senderDeviceId": "device-b", "signalKind": "offer", "signal": {"sdp": "fixture-offer"},
    });
    let (_, event) = fixture.client.append_human_call_event_for_user(
        &call_id, "signal-device-positive", 0, "signal", &matching, "1",
    ).expect("matching dispatch snapshot admits signaling");
    assert_eq!(event.device_id, "device-b");
    assert_eq!(event.payload, matching);
    let call = dispatch(fixture.runtime(), "getCallSession", json!({"callId": call_id}))
        .expect("gateway replays the accepted matching signal");
    assert_eq!(call["signalSeq"], 1);
    assert_eq!(stored_cursor(fixture.runtime(), &call_id), 1);
    let store = CallSessionStore::open(&fixture.agents_root, 500).expect("durable call owner");
    let signals = store.list_signals(&call_id, 0, 0, 200).expect("durable signals");
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].sender_device_id, "device-b");
    assert_eq!(signals[0].payload, json!({"sdp": "fixture-offer"}));
    drop(store);
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    assert_eq!(fixture.service.event_posts(), 1);
    assert_eq!(fixture.service.finish(), baseline_connections + 2,
        "the two sender negatives add no HTTP; only matching POST and replay GET are sent");
    assert_fixture_headers(&fixture.service.requests(), "device-b");
}

#[test]
fn native_call_credentials_fence_every_held_get_field_before_lifecycle_post() {
    let rotations = [
        ("userId", json!(2)),
        ("deviceId", json!("device-b")),
        ("accessToken", json!("rotated-fixture-token")),
        ("sessionId", json!("fixture-session-b")),
        ("expiresAtMs", json!(4_102_444_800_001_u64)),
        ("issuer", json!("http://localhost:1")),
    ];
    for (field, rotated) in rotations {
        let mut fixture = Fixture::new();
        let call_id = fixture.create_call();
        let seed_signal = dispatch(fixture.runtime(), "sendCallSignal", json!({
            "callId": call_id, "generation": 0, "seq": 1, "senderDeviceId": "device-a",
            "kind": "offer", "payload": {"sdp": "seed-signal"},
        })).expect("seed a real persisted signal through the gateway");
        assert_eq!(seed_signal["seq"], 1);
        assert_eq!(seed_signal["senderDeviceId"], "device-a");
        fixture.append_remote_media(&call_id);
        let before_calls = call_snapshot(fixture.runtime());
        let before_conversation = conversation_snapshot(&fixture.conversation_path);
        assert_eq!(before_calls.0.len(), 1);
        assert_eq!(before_calls.1.len(), 1);
        assert_eq!(before_calls.2.len(), 1, "signals must be nonempty in the negative oracle");
        assert_eq!(stored_cursor(fixture.runtime(), &call_id), 1);
        let posts_before = fixture.service.event_posts();
        let connections_before = fixture.service.accepted_connections();
        let pending = fixture.pending("transitionCallSession", json!({
            "callId": call_id, "generation": 0, "action": "ring",
        }), "GET", &format!("/api/social/calls/{call_id}"));
        pending.wait_until_held();
        fixture.rotate(field, rotated);
        assert_stale_response(pending.complete(), field);
        assert_eq!(fixture.service.event_posts(), posts_before,
            "{field}: a stale valid GET must not authorize any lifecycle POST");
        assert_eq!(fixture.service.accepted_connections(), connections_before + 1,
            "{field}: only the held preflight GET was dispatched");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls,
            "{field}: call, unconsumed media, cursor and signal must remain unchanged");
        assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
        assert_eq!(fixture.remote_call(&call_id)["eventSeq"], 2);
        assert_eq!(fixture.remote_call(&call_id)["state"], "invited");
        fixture.reopen();
        assert_eq!(call_snapshot(fixture.runtime()), before_calls,
            "{field}: reopening cannot conceal stale durable settlement");
        fixture.restore();
        let replayed = dispatch(fixture.runtime(), "getCallSession", json!({"callId": call_id}))
            .expect("restored credential replays the previously held remote media event");
        assert_eq!(replayed["state"], "invited");
        assert_eq!(replayed["mediaCapabilities"], json!({"audio": true, "video": true}));
        assert_eq!(replayed["deviceSelection"], json!({"microphoneId": "fixture-remote-microphone"}));
        assert_eq!(stored_cursor(fixture.runtime(), &call_id), 2);
        let settled = dispatch(fixture.runtime(), "transitionCallSession", json!({
            "callId": call_id, "generation": 0, "action": "ring",
        })).expect("restored credential retries the originally rejected lifecycle command");
        assert_eq!(settled["state"], "ringing");
        assert_eq!(settled["generation"], 0);
        assert_eq!(settled["signalSeq"], 1, "media and ring do not create signal rows");
        assert_eq!(stored_cursor(fixture.runtime(), &call_id), 3);
        assert_eq!(call_snapshot(fixture.runtime()).2, before_calls.2,
            "the existing signal survives rejection, reopen, replay and retry");
        assert_eq!(fixture.service.event_posts(), posts_before + 1);
        assert_eq!(fixture.service.create_posts(), 1);
        assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
        assert_eq!(fixture.service.finish(), connections_before + 5,
            "held GET plus recovery GET and retry GET/POST/GET are the entire suffix");
        assert_fixture_headers(&fixture.service.requests(), "device-a");
    }
}

#[test]
fn native_call_credentials_reject_stale_ice_and_discovery_bodies() {
    {
        let mut fixture = Fixture::new();
        let before_calls = call_snapshot(fixture.runtime());
        let before_conversation = conversation_snapshot(&fixture.conversation_path);
        let pending = fixture.pending("getCallIceServers", json!({}), "GET", "/api/social/calls/ice");
        pending.wait_until_held();
        fixture.rotate("accessToken", json!("rotated-fixture-token"));
        assert_stale_response(pending.complete(), "accessToken");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
        assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
        assert_eq!(fixture.service.accepted_connections(), 1);
        fixture.restore();
        let ice = dispatch(fixture.runtime(), "getCallIceServers", json!({}))
            .expect("restored credential accepts valid ICE data");
        assert_eq!(ice["ttlSeconds"], 3600);
        assert_eq!(ice["iceServers"][0]["urls"], json!(["turn:relay.example.invalid:3478"]));
        assert_eq!(ice["iceServers"][0]["username"], "fixture-ice-user");
        assert_eq!(ice["iceServers"][0]["credential"], "fixture-ice-secret");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
        assert_eq!(fixture.service.finish(), 2);
        assert_fixture_headers(&fixture.service.requests(), "device-a");
    }
    {
        let mut fixture = Fixture::new();
        let call_id = Uuid::new_v4().to_string();
        fixture.insert_incoming_call(&call_id);
        let before_calls = call_snapshot(fixture.runtime());
        let before_conversation = conversation_snapshot(&fixture.conversation_path);
        let before_scopes = fixture.runtime().list_human_conversations().expect("local conversation list");
        assert_eq!(before_scopes.len(), 1);
        let pending = fixture.pending("syncHumanCalls", json!({}), "GET", "/api/social/calls");
        pending.wait_until_held();
        fixture.rotate("accessToken", json!("rotated-fixture-token"));
        assert_stale_response(pending.complete(), "accessToken");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls,
            "rejected discovery cannot materialize its previously unknown call");
        assert_eq!(fixture.runtime().list_human_conversations().unwrap(), before_scopes,
            "rejected discovery cannot create the remote peer's canonical scope");
        assert_eq!(fixture.service.accepted_connections(), 1,
            "rejected list data cannot authorize a detail GET");
        fixture.reopen();
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
        assert_eq!(fixture.runtime().list_human_conversations().unwrap(), before_scopes);
        fixture.restore();
        let discovered = dispatch(fixture.runtime(), "syncHumanCalls", json!({}))
            .expect("restored credential discovers and replays the existing incoming call");
        let calls = discovered.as_array().expect("discovered call array");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0]["id"], call_id);
        assert_eq!(calls[0]["creatorId"], "3");
        assert_eq!(calls[0]["participantIds"], json!(["1", "3"]));
        assert_ne!(calls[0]["scopeId"], fixture.scope_id);
        assert_eq!(fixture.runtime().list_human_conversations().unwrap().len(), 2);
        assert_eq!(call_snapshot(fixture.runtime()).0.len(), 1);
        assert_eq!(stored_cursor(fixture.runtime(), &call_id), 0);
        assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
        assert_eq!(fixture.service.create_posts(), 0);
        assert_eq!(fixture.service.event_posts(), 0);
        assert_eq!(fixture.service.finish(), 3,
            "held list GET is followed only by restored list and detail GETs");
        assert_fixture_headers(&fixture.service.requests(), "device-a");
    }
}

#[test]
fn native_call_credentials_recover_admitted_create_after_rejected_response() {
    let mut fixture = Fixture::new();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation_path);
    assert!(before_calls.0.is_empty());
    let pending = fixture.pending("createCallSession", json!({
        "scopeId": fixture.scope_id, "participantIds": ["1", "2"],
    }), "POST", "/api/social/calls");
    pending.wait_until_held();
    let requests = fixture.service.requests();
    assert_eq!(requests.len(), 1);
    let call_id = requests[0].body["callId"].as_str().expect("admitted creation id").to_string();
    assert_eq!(fixture.remote_call(&call_id)["state"], "invited",
        "the service has accepted the creation before releasing its response");
    fixture.rotate("accessToken", json!("rotated-fixture-token"));
    assert_stale_response(pending.complete(), "accessToken");
    assert_eq!(call_snapshot(fixture.runtime()), before_calls,
        "a rejected create response cannot persist a local call row");
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    assert_eq!(fixture.service.create_posts(), 1,
        "response rejection does not claim the already admitted mutation was undone");
    assert_eq!(fixture.remote_call(&call_id)["eventSeq"], 0);
    fixture.reopen();
    assert_eq!(call_snapshot(fixture.runtime()), before_calls);
    fixture.restore();
    let recovered = dispatch(fixture.runtime(), "syncHumanCalls", json!({}))
        .expect("restored account discovers the admitted creation");
    let calls = recovered.as_array().expect("recovered call array");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["id"], call_id);
    assert_eq!(calls[0]["scopeId"], fixture.scope_id);
    assert_eq!(calls[0]["state"], "invited");
    assert_eq!(calls[0]["participantIds"], json!(["1", "2"]));
    assert_eq!(stored_cursor(fixture.runtime(), &call_id), 0);
    assert_eq!(call_snapshot(fixture.runtime()).0.len(), 1);
    fixture.reopen();
    let store = CallSessionStore::open(&fixture.agents_root, 500).expect("reopened durable call owner");
    let persisted = store.get(&call_id).expect("recovered call lookup").expect("recovered call survives reopen");
    assert_eq!(persisted.scope_id, fixture.scope_id);
    assert_eq!(persisted.creator_id, "1");
    assert_eq!(persisted.state, "invited");
    drop(store);
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    assert_eq!(fixture.service.create_posts(), 1, "discovery recovers the same id without a second creation");
    assert_eq!(fixture.service.event_posts(), 0);
    assert_eq!(fixture.service.finish(), 3, "one admitted POST, discovery GET, and detail GET");
    assert_fixture_headers(&fixture.service.requests(), "device-a");
}

#[test]
fn native_call_credentials_recover_admitted_event_after_rejected_response() {
    let mut fixture = Fixture::new();
    let call_id = fixture.create_call();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation_path);
    let connections_before = fixture.service.accepted_connections();
    let pending = fixture.pending("transitionCallSession", json!({
        "callId": call_id, "generation": 0, "action": "ring",
    }), "POST", &format!("/api/social/calls/{call_id}/events"));
    pending.wait_until_held();
    assert_eq!(fixture.remote_call(&call_id)["state"], "ringing",
        "the transition is accepted remotely before its response is released");
    assert_eq!(fixture.remote_call(&call_id)["eventSeq"], 1);
    fixture.rotate("accessToken", json!("rotated-fixture-token"));
    assert_stale_response(pending.complete(), "accessToken");
    assert_eq!(call_snapshot(fixture.runtime()), before_calls,
        "a rejected append response cannot advance the local call or replay cursor");
    assert_eq!(fixture.service.accepted_connections(), connections_before + 2,
        "preflight GET and admitted POST occur, with no stale follow-up sync");
    assert_eq!(fixture.service.event_posts(), 1);
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    fixture.reopen();
    assert_eq!(call_snapshot(fixture.runtime()), before_calls);
    fixture.restore();
    let recovered = dispatch(fixture.runtime(), "getCallSession", json!({"callId": call_id}))
        .expect("restored credential replays the already accepted event");
    assert_eq!(recovered["id"], call_id);
    assert_eq!(recovered["scopeId"], fixture.scope_id);
    assert_eq!(recovered["state"], "ringing");
    assert_eq!(recovered["generation"], 0);
    assert_eq!(recovered["signalSeq"], 0);
    assert_eq!(stored_cursor(fixture.runtime(), &call_id), 1);
    assert_eq!(call_snapshot(fixture.runtime()).2, before_calls.2);
    fixture.reopen();
    let store = CallSessionStore::open(&fixture.agents_root, 500).expect("reopened durable call owner");
    assert_eq!(store.get(&call_id).unwrap().unwrap().state, "ringing");
    assert_eq!(store.remote_event_seq(&call_id).unwrap(), 1);
    drop(store);
    assert_eq!(conversation_snapshot(&fixture.conversation_path), before_conversation);
    assert_eq!(fixture.service.event_posts(), 1,
        "replay recovers the accepted ring; it must not send the now-invalid ring action again");
    assert_eq!(fixture.service.create_posts(), 1);
    assert_eq!(fixture.service.finish(), connections_before + 3,
        "preflight GET, admitted POST and restored replay GET are the entire suffix");
    assert_fixture_headers(&fixture.service.requests(), "device-a");
}
