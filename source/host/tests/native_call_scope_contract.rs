use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::memory::memory_service::MemoryService;
use mahayana_host_runtime::extensions::session::agent_db::SandAgentDb;
use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
};
use mahayana_host_runtime::extensions::session::native_messaging::FabushiNativeMessagingClient;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct HttpRequest {
    method: String,
    target: String,
    body: Value,
}

struct RemoteCall {
    value: Value,
    events: Vec<Value>,
}

#[derive(Default)]
struct ServiceState {
    accepted_connections: usize,
    requests: Vec<HttpRequest>,
    calls: BTreeMap<String, RemoteCall>,
    refuse_next_creation: bool,
}

struct CallService {
    base_url: String,
    state: Arc<Mutex<ServiceState>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<(), String>>>,
}

impl CallService {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind call scope service");
        let base_url = format!("http://{}", listener.local_addr().expect("service address"));
        listener.set_nonblocking(true).expect("bounded service accept");
        let state = Arc::new(Mutex::new(ServiceState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                if Instant::now() >= deadline {
                    return Err("call scope fixture exceeded its accept deadline".into());
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false)
                            .map_err(|error| format!("restore blocking call fixture stream: {error}"))?;
                        worker_state.lock().map_err(|_| "service state poisoned")?
                            .accepted_connections += 1;
                        let request = read_request(&mut stream, deadline)?;
                        let (status, body) = {
                            let mut state = worker_state.lock().map_err(|_| "service state poisoned")?;
                            state.requests.push(request.clone());
                            reply(&mut state, &request)?
                        };
                        stream.set_write_timeout(Some(Duration::from_secs(5)))
                            .map_err(|error| error.to_string())?;
                        let body = body.to_string();
                        write!(
                            stream,
                            "HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(), body,
                        ).map_err(|error| format!("write call fixture response: {error}"))?;
                        stream.flush().map_err(|error| error.to_string())?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        // Drain any already queued connection before honoring stop.
                        if worker_stop.load(Ordering::SeqCst) {
                            return Ok(());
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => return Err(format!("accept call fixture request: {error}")),
                }
            }
        });
        Self { base_url, state, stop, worker: Some(worker) }
    }

    fn finish(&mut self) -> (usize, Vec<HttpRequest>) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("call fixture thread").expect("call fixture protocol");
        }
        let state = self.state.lock().expect("service state");
        (state.accepted_connections, state.requests.clone())
    }
}

impl Drop for CallService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            // Assertions may unwind early; the bounded accept/read path still joins.
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
        .map_err(|error| format!("request headers: {error}"))?.to_string();
    let mut lines = headers.split("\r\n");
    let mut first = lines.next().ok_or("request line missing")?.split_whitespace();
    let method = first.next().ok_or("request method missing")?.to_string();
    let target = first.next().ok_or("request target missing")?.to_string();
    let headers = lines.filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_string()))
        .collect::<BTreeMap<_, _>>();
    if headers.get("authorization").map(String::as_str) != Some("Bearer call-scope-fixture")
        || headers.get("x-fabushi-device-id").map(String::as_str) != Some("device-a")
    {
        return Err("fixture did not receive the Native bearer/device headers".into());
    }
    let length = headers.get("content-length")
        .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
        .transpose()?.unwrap_or(0);
    if length > 64 * 1024 {
        return Err("call fixture request body exceeded its bound".into());
    }
    while bytes.len() < header_end + length {
        read_more(stream, &mut bytes, deadline)?;
    }
    let body = if length == 0 {
        Value::Null
    } else {
        serde_json::from_slice(&bytes[header_end..header_end + length])
            .map_err(|error| format!("request JSON: {error}"))?
    };
    Ok(HttpRequest { method, target, body })
}

fn read_more(stream: &mut TcpStream, bytes: &mut Vec<u8>, deadline: Instant) -> Result<(), String> {
    let remaining = deadline.checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero()).ok_or("call fixture request read deadline")?;
    stream.set_read_timeout(Some(remaining)).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 4096];
    let count = stream.read(&mut buffer).map_err(|error| format!("read call request: {error}"))?;
    if count == 0 {
        return Err("call fixture request ended before its complete frame".into());
    }
    bytes.extend_from_slice(&buffer[..count]);
    if bytes.len() > 128 * 1024 {
        return Err("call fixture request exceeded its frame bound".into());
    }
    Ok(())
}

fn reply(state: &mut ServiceState, request: &HttpRequest) -> Result<(u16, Value), String> {
    let url = url::Url::parse(&format!("http://fixture{}", request.target))
        .map_err(|error| error.to_string())?;
    if request.method == "POST" && url.path() == "/api/social/calls" {
        let call_id = request.body.get("callId").and_then(Value::as_str)
            .ok_or("callId missing")?;
        let peer = request.body.get("targetUserId").and_then(Value::as_str)
            .ok_or("targetUserId missing")?;
        if std::mem::take(&mut state.refuse_next_creation) {
            return Ok((403, json!({"success": false, "error": "fixture refused call creation"})));
        }
        let value = json!({
            "callId": call_id, "creatorUserId": 1, "peerUserId": peer,
            "creatorDeviceId": "device-a", "peerDeviceId": null,
            "state": "invited", "generation": 0, "eventSeq": 0, "terminalState": null,
            "createdAt": "2026-11-02T00:00:00Z", "updatedAt": "2026-11-02T00:00:00Z"
        });
        state.calls.insert(call_id.to_string(), RemoteCall { value: value.clone(), events: Vec::new() });
        return Ok((200, json!({"success": true, "call": value})));
    }
    if request.method == "GET" && url.path() == "/api/social/calls" {
        let calls = state.calls.values().map(|call| call.value.clone()).collect::<Vec<_>>();
        return Ok((200, json!({"success": true, "calls": calls})));
    }
    let suffix = url.path().strip_prefix("/api/social/calls/")
        .ok_or_else(|| format!("unexpected call fixture path: {}", request.target))?;
    if request.method == "POST" && suffix.ends_with("/events") {
        let call_id = suffix.trim_end_matches("/events");
        let call = state.calls.get_mut(call_id).ok_or("unknown fixture call")?;
        let kind = request.body.get("kind").and_then(Value::as_str).ok_or("event kind missing")?;
        let generation = request.body.get("generation").and_then(Value::as_u64)
            .ok_or("event generation missing")?;
        let client_event_id = request.body.get("clientEventId").and_then(Value::as_str)
            .ok_or("clientEventId missing")?;
        let payload = request.body.get("payload").ok_or("event payload missing")?;
        if kind != "transition" {
            return Err("call scope fixture supports only lifecycle transitions".into());
        }
        let next_state = payload.get("state").and_then(Value::as_str).ok_or("transition state missing")?;
        let seq = call.events.len() as u64 + 1;
        let event = json!({
            "callId": call_id, "seq": seq, "generation": generation,
            "userId": 1, "deviceId": "device-a", "clientEventId": client_event_id,
            "kind": kind, "payload": payload, "createdAt": "2026-11-02T00:00:01Z"
        });
        call.events.push(event.clone());
        call.value["state"] = json!(next_state);
        call.value["generation"] = json!(generation);
        call.value["eventSeq"] = json!(seq);
        return Ok((200, json!({"success": true, "call": call.value, "event": event})));
    }
    if request.method == "GET" {
        let call = state.calls.get(suffix).ok_or("unknown fixture call")?;
        let after_seq = url.query_pairs().find(|(key, _)| key == "afterSeq")
            .ok_or("afterSeq missing")?.1.parse::<u64>().map_err(|error| error.to_string())?;
        let events = call.events.iter().filter(|event| event["seq"].as_u64().unwrap() > after_seq)
            .cloned().collect::<Vec<_>>();
        let next = events.last().and_then(|event| event["seq"].as_u64()).unwrap_or(after_seq);
        return Ok((200, json!({"success": true, "call": call.value, "events": events, "nextAfterSeq": next})));
    }
    Err(format!("unexpected call fixture request: {} {}", request.method, request.target))
}

struct Fixture {
    root: PathBuf,
    agents_root: PathBuf,
    runtime: Option<Arc<ProductionSessionWorkers>>,
    client: Arc<FabushiNativeMessagingClient>,
    service: CallService,
    scope12: String,
    scope13: String,
    conversation12_path: PathBuf,
    conversation13_path: PathBuf,
    prior_call_id: String,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-native-call-scope-{}", Uuid::new_v4()));
        let agents_root = root.join("agents");
        fs::create_dir_all(&root).expect("call scope root");
        let service = CallService::start();
        let credential_path = root.join("host-credential.json");
        fs::write(&credential_path, json!({
            "accessToken": "call-scope-fixture", "deviceId": "device-a",
            "userId": 1, "issuer": service.base_url,
        }).to_string()).expect("fixture credential");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&credential_path, fs::Permissions::from_mode(0o600))
                .expect("private credential");
        }
        let client = Arc::new(FabushiNativeMessagingClient::new(&service.base_url, credential_path)
            .expect("Native call client"));
        let local = workers(&agents_root, None);
        let scope12 = local.create_human_conversation("2", "Peer two").expect("scope 12")
            ["id"].as_str().expect("scope id").to_string();
        let scope13 = local.create_human_conversation("3", "Peer three").expect("scope 13")
            ["id"].as_str().expect("scope id").to_string();
        for (scope, nonce) in [(&scope12, "scope-12-existing-message"), (&scope13, "scope-13-existing-message")] {
            local.append_human_message(scope, "Existing conversation content", nonce, Some(1_000.0), None, &[])
                .expect("seed existing transcript through Session");
        }
        let prior_call = dispatch(&local, "createCallSession", json!({
            "scopeId": scope12, "participantIds": ["1", "2"]
        })).expect("seed existing local call");
        let prior_call_id = prior_call["id"].as_str().expect("prior call id").to_string();
        let conversation12_path = local.open_human_conversation_db_owner(&scope12).expect("owner 12")
            .db_path().to_path_buf();
        let conversation13_path = local.open_human_conversation_db_owner(&scope13).expect("owner 13")
            .db_path().to_path_buf();
        local.shutdown();
        drop(local);
        let runtime = Some(workers(&agents_root, Some(Arc::clone(&client))));
        Self {
            root, agents_root, runtime, client, service, scope12, scope13,
            conversation12_path, conversation13_path, prior_call_id,
        }
    }

    fn runtime(&self) -> &Arc<ProductionSessionWorkers> {
        self.runtime.as_ref().expect("live fixture runtime")
    }

    fn reopen(&mut self, native: bool) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown();
            drop(runtime);
        }
        self.runtime = Some(workers(&self.agents_root, native.then(|| Arc::clone(&self.client))));
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
        owner.get_transcript_entries().expect("persisted transcript"),
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

#[test]
fn native_call_scope_rejects_wrong_peer_and_invalid_requests_without_effects() {
    let mut fixture = Fixture::new();
    let before12 = conversation_snapshot(&fixture.conversation12_path);
    let before13 = conversation_snapshot(&fixture.conversation13_path);
    let before_calls = call_snapshot(fixture.runtime());
    assert_eq!(before_calls.0.len(), 1, "negative controls include an existing call");
    let cases = [
        (fixture.scope12.clone(), json!(["1", "3"])),
        (fixture.scope13.clone(), json!(["1", "2"])),
        (fixture.scope12.clone(), json!([])),
        (fixture.scope12.clone(), json!(["1"])),
        (fixture.scope12.clone(), json!(["1", "1"])),
        (fixture.scope12.clone(), json!(["2", "3"])),
        (fixture.scope12.clone(), json!(["1", "2", "3"])),
        (fixture.scope12.clone(), json!(["1", " "])),
        (fixture.scope12.clone(), json!(["1", 2])),
        (fixture.scope12.clone(), json!("1,2")),
    ];
    for (scope, participants) in cases {
        let result = dispatch(fixture.runtime(), "createCallSession", json!({
            "scopeId": scope, "participantIds": participants,
        }));
        assert!(result.is_err(), "invalid scope/pair must fail: {scope} {participants}");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls, "no call mutation after {participants}");
        assert_eq!(conversation_snapshot(&fixture.conversation12_path), before12);
        assert_eq!(conversation_snapshot(&fixture.conversation13_path), before13);
    }
    fixture.reopen(true);
    assert_eq!(call_snapshot(fixture.runtime()), before_calls, "reopen preserves prior calls only");
    assert_eq!(conversation_snapshot(&fixture.conversation12_path), before12);
    assert_eq!(conversation_snapshot(&fixture.conversation13_path), before13);
    let (connections, requests) = fixture.service.finish();
    assert_eq!(connections, 0, "invalid pairs cannot reach authenticated transport");
    assert!(requests.is_empty());
}

#[test]
fn native_call_scope_rejects_malformed_canonical_metadata_and_preserves_reopen_state() {
    let mut fixture = Fixture::new();
    let original = conversation_snapshot(&fixture.conversation12_path);
    let before_calls = call_snapshot(fixture.runtime());
    let cases = [
        ("participantIds", Value::Null),
        ("participantIds", json!({})),
        ("participantIds", json!([])),
        ("participantIds", json!(["1"])),
        ("participantIds", json!(["1", "1"])),
        ("participantIds", json!(["1", 2])),
        ("participantIds", json!(["1", ""])),
        ("participantIds", json!(["1", " "])),
        ("participantIds", json!(["1", " 2"])),
        ("participantIds", json!(["1", "2 "])),
        ("participantIds", json!(["1", "2", "2"])),
        ("participantIds", json!(["1", "2", "3"])),
        ("participantIds", json!(["2", "3"])),
        ("conversationKind", json!("agent")),
        ("conversationId", json!("foreign-conversation-id")),
    ];
    for (key, invalid) in cases {
        let owner = fixture.runtime().open_human_conversation_db_owner(&fixture.scope12)
            .expect("canonical owner before deliberate corruption");
        assert!(owner.set_metadata(key, invalid.clone()).expect("write corrupt fixture metadata"));
        drop(owner);
        let before = conversation_snapshot(&fixture.conversation12_path);
        let result = dispatch(fixture.runtime(), "createCallSession", json!({
            "scopeId": fixture.scope12, "participantIds": ["1", "2"],
        }));
        assert!(result.is_err(), "invalid stored {key} must fail: {invalid}");
        assert_eq!(conversation_snapshot(&fixture.conversation12_path), before);
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
        fixture.reopen(true);
        assert_eq!(conversation_snapshot(&fixture.conversation12_path), before, "no repair-by-discard on reopen");
        assert_eq!(call_snapshot(fixture.runtime()), before_calls);
        let owner = SandAgentDb::open_conversation_store(&fixture.conversation12_path, 500)
            .expect("restore fixture through existing SQLite owner");
        assert!(owner.set_metadata(key, original.0[key].clone()).expect("restore fixture field"));
        owner.close(false);
    }
    assert_eq!(conversation_snapshot(&fixture.conversation12_path), original);
    let (connections, requests) = fixture.service.finish();
    assert_eq!(connections, 0, "invalid canonical identity cannot reach transport");
    assert!(requests.is_empty());
}

#[test]
fn native_call_scope_valid_creation_lifecycle_and_discovery_survive_reopen() {
    let mut fixture = Fixture::new();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation12_path);
    let created = dispatch(fixture.runtime(), "createCallSession", json!({
        "scopeId": fixture.scope12, "participantIds": [" 2 ", "1", "1", ""],
    })).expect("normalized matching pair remains compatible");
    let call_id = created["id"].as_str().expect("created call id").to_string();
    assert_ne!(call_id, fixture.prior_call_id);
    assert_eq!(created["scopeId"], fixture.scope12);
    assert_eq!(created["participantIds"], json!(["1", "2"]));
    assert_eq!(created["state"], "invited");
    let ringing = dispatch(fixture.runtime(), "transitionCallSession", json!({
        "callId": call_id, "generation": 0, "action": "ring",
    })).expect("existing Native lifecycle");
    assert_eq!(ringing["state"], "ringing");
    fixture.reopen(true);
    let restored = dispatch(fixture.runtime(), "getCallSession", json!({"callId": call_id}))
        .expect("reopen and reconcile existing Native call");
    assert_eq!(restored, ringing);
    let discovered = dispatch(fixture.runtime(), "syncHumanCalls", json!({}))
        .expect("discovery retains matching canonical scope");
    assert_eq!(discovered.as_array().map(Vec::len), Some(1));
    assert_eq!(discovered[0], restored);
    assert_eq!(conversation_snapshot(&fixture.conversation12_path), before_conversation);
    let after_calls = call_snapshot(fixture.runtime());
    assert_eq!(after_calls.0.len(), before_calls.0.len() + 1);
    assert!(after_calls.0.contains(&before_calls.0[0]), "preexisting call stays intact");
    let (connections, requests) = fixture.service.finish();
    assert_eq!(connections, requests.len());
    let creates = requests.iter().filter(|request| request.method == "POST" && request.target == "/api/social/calls")
        .collect::<Vec<_>>();
    assert_eq!(creates.len(), 1, "reopen/discovery cannot duplicate creation");
    assert_eq!(creates[0].body["callId"], call_id);
    assert_eq!(creates[0].body["targetUserId"], "2");
    assert!(creates[0].body.get("scopeId").is_none(), "existing service schema stays unchanged");
}

#[test]
fn native_call_scope_backend_refusal_has_no_phantom_call_and_allows_retry() {
    let mut fixture = Fixture::new();
    let before_calls = call_snapshot(fixture.runtime());
    let before_conversation = conversation_snapshot(&fixture.conversation12_path);
    fixture.service.state.lock().expect("service state").refuse_next_creation = true;
    let refused = dispatch(fixture.runtime(), "createCallSession", json!({
        "scopeId": fixture.scope12, "participantIds": ["1", "2"],
    })).expect_err("Native refusal must propagate");
    assert!(matches!(refused, SessionGatewayError::Internal(message)
        if message.contains("rejected (HTTP 403)")));
    assert_eq!(call_snapshot(fixture.runtime()), before_calls);
    assert_eq!(conversation_snapshot(&fixture.conversation12_path), before_conversation);
    fixture.reopen(true);
    assert_eq!(call_snapshot(fixture.runtime()), before_calls, "no refused call after reopen");
    let created = dispatch(fixture.runtime(), "createCallSession", json!({
        "scopeId": fixture.scope12, "participantIds": ["1", "2"],
    })).expect("valid retry after service recovery");
    assert_eq!(created["scopeId"], fixture.scope12);
    assert_eq!(call_snapshot(fixture.runtime()).0.len(), before_calls.0.len() + 1);
    assert_eq!(conversation_snapshot(&fixture.conversation12_path), before_conversation);
    let (connections, requests) = fixture.service.finish();
    assert_eq!(connections, 2);
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| request.method == "POST"
        && request.target == "/api/social/calls" && request.body["targetUserId"] == "2"));
    assert_eq!(requests[1].body["callId"], created["id"]);
}

#[test]
fn native_call_scope_leaves_non_native_participant_and_lifecycle_contract_unchanged() {
    let mut fixture = Fixture::new();
    fixture.reopen(false);
    // The local-only owner already permits multiple participants. Native direct
    // admission must not silently redefine that separate compatibility contract.
    let created = dispatch(fixture.runtime(), "createCallSession", json!({
        "scopeId": fixture.scope12, "participantIds": ["3", "1", "2"],
    })).expect("existing non-Native participant contract");
    let call_id = created["id"].as_str().expect("local call id").to_string();
    assert_eq!(created["participantIds"], json!(["1", "2", "3"]));
    let ended = dispatch(fixture.runtime(), "transitionCallSession", json!({
        "callId": call_id, "generation": 0, "action": "hangup", "terminalReason": "local-finished",
    })).expect("existing non-Native lifecycle");
    assert_eq!(ended["state"], "ended");
    fixture.reopen(false);
    let restored = dispatch(fixture.runtime(), "getCallSession", json!({"callId": call_id}))
        .expect("existing non-Native reopen");
    assert_eq!(restored, ended);
    let (connections, requests) = fixture.service.finish();
    assert_eq!(connections, 0);
    assert!(requests.is_empty());
}
