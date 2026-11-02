use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryRecord, MemoryUnsubscribe, memory_id_for,
};
use mahayana_host_runtime::extensions::session::agent_db::PendingMemoryMutation;
use mahayana_host_runtime::extensions::session::agent_db_serde::MemoryPromptSnapshot;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayEventHub, GatewayServer, GatewayServerDeps,
    start_gateway_server,
};
use mahayana_host_runtime::runner::sand_memory::render_memory_system_prompt;
use serde_json::{Value, json};

const NATIVE_TOKEN: &str = "memory-contract-native-token";
const PROFILE: &str = "Private profile: prefers tea ☃";
const LOG: &str = "Private log: launch on Friday 界";
const OTHER: &str = "Other agent private sentinel";
const OUTSIDE: &str = "Outside memory sentinel";
const DAY_MS: i64 = 86_400_000;
const METHODS: [&str; 3] = ["getAgentMemories", "deleteAgentMemory", "clearAgentMemories"];
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

struct OwnedRoot(PathBuf);

impl OwnedRoot {
    fn new() -> Self {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "fabushi-memory-gateway-{}-{stamp}-{sequence}", std::process::id(),
        ));
        fs::create_dir_all(&path).expect("owned Memory fixture root");
        Self(path)
    }
}

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        let result = fs::remove_dir_all(&self.0);
        if !thread::panicking() {
            result.expect("remove closed Memory fixture");
        }
    }
}

struct Credential {
    bearer: String,
    token: String,
    authority: DeviceTokenAuthorization,
}

fn issue_credential(owner: &DevicePairingOwner, label: &str, role: &str, scopes: &[&str]) -> Credential {
    let device_id = format!("memory-{label}");
    let pending = owner.handle_pair_request(&json!({
        "deviceId": device_id,
        "publicKey": format!("memory-contract-key-{label}"),
        "role": role,
        "roles": [role],
        "scopes": scopes,
    })).expect("real pairing request");
    owner.handle_pair_approve(&json!({"requestId": pending["requestId"]}))
        .expect("real pairing approval");
    let rotated = owner.handle_pair_rotate_token(&json!({"deviceId": device_id, "role": role}))
        .expect("real token rotation");
    let token = rotated["token"].as_str().expect("issued token").to_string();
    let authority = owner.authenticate_device_token(&token)
        .expect("issued token lookup").expect("issued token is recognized");
    assert_eq!(authority.device_id, device_id);
    assert_eq!(authority.role, role);
    assert_eq!(authority.scopes, scopes.iter().map(|value| value.to_string()).collect::<Vec<_>>());
    Credential { bearer: format!("Bearer {token}"), token, authority }
}

struct MemoryApi {
    pairing: DevicePairingOwner,
    sessions: Arc<ProductionSessionWorkers>,
    manager: TranscriptManager,
    calls: Mutex<Vec<(String, Value)>>,
    authorizations: Mutex<Vec<Option<DeviceTokenAuthorization>>>,
}

impl MemoryApi {
    fn open(root: &Path) -> Self {
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        Self {
            pairing: DevicePairingOwner::new(root.join("pairing.sqlite")).expect("pairing DB"),
            manager: TranscriptManager::new(root, Arc::clone(&sessions)),
            sessions,
            calls: Mutex::new(Vec::new()),
            authorizations: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().expect("owner calls").clone()
    }
}

impl GatewayApi for MemoryApi {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let authority = self.pairing.authenticate_device_token(token)?;
        self.authorizations.lock().expect("authorization observations").push(authority.clone());
        Ok(authority)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls.lock().expect("record real owner boundary")
            .push((method.to_string(), args.clone()));
        if let Some(result) = self.manager.dispatch_memory_gateway_call(method, &args) {
            // The shipping UnifiedGatewayApi uses this same delegate and error mapping.
            return result.map_err(GatewayCommandError::Internal);
        }
        match method {
            // Ordinary-route controls exercise the unchanged ingress classifications.
            "getTranscript" | "config.get" | "config.set" => Ok(args),
            _ => Err(GatewayCommandError::UnknownMethod(method.to_string())),
        }
    }
}

impl Drop for MemoryApi {
    fn drop(&mut self) {
        self.manager.dispose();
    }
}

#[derive(Debug, PartialEq)]
struct AgentMemoryState {
    records: Vec<MemoryRecord>,
    snapshot: Option<MemoryPromptSnapshot>,
    journal: Option<PendingMemoryMutation>,
}

#[derive(Debug, PartialEq)]
struct MemoryState {
    files: BTreeMap<PathBuf, Vec<u8>>,
    agents: Vec<AgentMemoryState>,
    notifications: usize,
}

fn collect_files(root: &Path, path: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => panic!("read fixture directory {}: {error}", path.display()),
    };
    for entry in entries {
        let entry = entry.expect("fixture entry");
        let path = entry.path();
        if entry.file_type().expect("fixture file type").is_dir() {
            collect_files(root, &path, output);
        } else {
            output.insert(path.strip_prefix(root).expect("owned file").to_path_buf(),
                fs::read(&path).expect("fixture bytes"));
        }
    }
}

struct HttpReply {
    status: u16,
    body: Value,
    raw: String,
}

struct MemoryGateway {
    // Drop/close the listener and API owners before OwnedRoot removes their files.
    server: Option<GatewayServer>,
    api: Arc<MemoryApi>,
    root: OwnedRoot,
    config: GatewayServerConfig,
    agent_id: String,
    other_id: String,
    profile_id: String,
    log_id: String,
    read_control: Credential,
    notifications: Arc<AtomicUsize>,
    unsubscribe: Option<MemoryUnsubscribe>,
}

impl MemoryGateway {
    fn new(configured_token: bool) -> Self {
        Self::with_bind_host(configured_token, "127.0.0.1")
    }

    fn with_bind_host(configured_token: bool, host: &str) -> Self {
        let root = OwnedRoot::new();
        let api = Arc::new(MemoryApi::open(&root.0));
        let session_store = SandAgentSessionStore::new(Arc::clone(&api.sessions));
        let agent_id = session_store.create_session(None, "user", None).expect("agent A").id;
        let other_id = session_store.create_session(None, "user", None).expect("agent B").id;
        let service = api.sessions.memory_service();
        let store = service.store_for_agent(&agent_id);
        let profile_id = store.add_memory(PROFILE, DAY_MS, MemoryKind::Profile)
            .expect("profile write").expect("profile fact").id;
        let log_id = store.add_memory(LOG, 2 * DAY_MS, MemoryKind::Log)
            .expect("log write").expect("log fact").id;
        service.store_for_agent(&other_id).add_memory(OTHER, DAY_MS, MemoryKind::Profile)
            .expect("other write").expect("other fact");
        let outside = FileMemoryStore::new(root.0.join("outside/memory"));
        let sentinel = outside.add_memory(OUTSIDE, DAY_MS, MemoryKind::Profile)
            .expect("outside write").expect("outside fact");
        assert_eq!(outside.list_memories(100), vec![sentinel], "outside oracle is readable");
        let read_control = issue_credential(&api.pairing, "read-control", "operator", &["operator.read"]);
        let config = GatewayServerConfig {
            host: host.into(), port: None,
            auth_token: configured_token.then(|| NATIVE_TOKEN.to_string()), tls: None,
        };
        let mut fixture = Self {
            server: None, api, root, config, agent_id, other_id, profile_id, log_id, read_control,
            notifications: Arc::new(AtomicUsize::new(0)), unsubscribe: None,
        };
        fixture.freeze(&fixture.agent_id);
        fixture.freeze(&fixture.other_id);
        fixture.subscribe();
        fixture.api.sessions.memory_service().set_active_agent(Some(fixture.agent_id.clone()));
        assert_eq!(fixture.notifications.load(Ordering::SeqCst), 1, "notification oracle is live");
        fixture.start();
        fixture.assert_read(Some(&fixture.read_control.bearer), "", fixture.initial_projection());
        fixture
    }

    fn subscribe(&mut self) {
        let notifications = Arc::clone(&self.notifications);
        self.unsubscribe = Some(self.api.sessions.memory_service().subscribe(Arc::new(move || {
            notifications.fetch_add(1, Ordering::SeqCst);
        })));
    }

    fn start(&mut self) {
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: self.api.clone(), events: GatewayEventHub::default(),
            local_exec: None, webauthn: None, config: self.config.clone(), started_at: 1,
        }).expect("real Memory HTTP gateway"));
    }

    fn stop(&mut self) {
        if let Some(server) = self.server.take() {
            server.close();
        }
        if let Some(unsubscribe) = self.unsubscribe.take() {
            unsubscribe();
        }
        self.api.manager.dispose();
    }

    fn reopen(&mut self) {
        self.stop();
        self.api = Arc::new(MemoryApi::open(&self.root.0));
        self.subscribe();
        self.start();
    }

    fn freeze(&self, agent_id: &str) {
        let store = self.api.sessions.memory_service().store_for_agent(agent_id);
        let render = render_memory_system_prompt(&store.recall(30), None);
        assert!(!render.is_empty(), "snapshot contains real stored facts");
        self.api.sessions.set_agent_memory_prompt_snapshot(agent_id, &json!({
            "render": render, "compactionEpoch": 0,
        })).expect("persist frozen prompt");
        let snapshot = self.api.sessions.get_agent_memory_prompt_snapshot(agent_id)
            .expect("read frozen prompt").expect("snapshot was persisted");
        assert_eq!(snapshot.render, render);
        assert_eq!(snapshot.compaction_epoch, 0.0);
    }

    fn state(&self) -> MemoryState {
        let mut files = BTreeMap::new();
        let agents = [&self.agent_id, &self.other_id].into_iter().map(|id| {
            let store = self.api.sessions.memory_service().store_for_agent(id);
            collect_files(&self.root.0, &store.get_location(), &mut files);
            let owner = self.api.sessions.open_agent_db_owner(id).expect("canonical agent DB");
            AgentMemoryState {
                records: store.list_memories(usize::MAX),
                snapshot: owner.get_memory_prompt_snapshot().expect("read prompt snapshot"),
                journal: owner.read_pending_memory_mutation().expect("read mutation journal"),
            }
        }).collect();
        collect_files(&self.root.0, &self.root.0.join("outside/memory"), &mut files);
        MemoryState { files, agents, notifications: self.notifications.load(Ordering::SeqCst) }
    }

    fn args(&self, method: &str) -> Value {
        if method == "deleteAgentMemory" {
            json!({"id": self.agent_id, "memoryId": self.profile_id})
        } else {
            json!({"id": self.agent_id})
        }
    }

    fn initial_projection(&self) -> Value {
        json!([
            {"id": self.profile_id, "content": PROFILE, "createdAt": DAY_MS, "kind": "profile"},
            {"id": self.log_id, "content": LOG, "createdAt": 2 * DAY_MS, "kind": "log"},
        ])
    }

    fn post(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str) -> HttpReply {
        let mut stream = TcpStream::connect(("127.0.0.1", self.server.as_ref().expect("server").port()))
            .expect("connect actual gateway");
        stream.set_read_timeout(Some(Duration::from_secs(5))).expect("read timeout");
        stream.set_write_timeout(Some(Duration::from_secs(5))).expect("write timeout");
        let authorization = authorization.map(|value| format!("Authorization: {value}\r\n"))
            .unwrap_or_default();
        let request = format!(
            "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len(),
        );
        stream.write_all(request.as_bytes()).expect("write real request");
        stream.flush().expect("flush request");
        let mut raw = String::new();
        stream.read_to_string(&mut raw).expect("drain response to EOF");
        let status = raw.lines().next().expect("HTTP status line").split_whitespace()
            .nth(1).expect("HTTP status").parse().expect("numeric HTTP status");
        let body = serde_json::from_str(raw.split_once("\r\n\r\n").expect("HTTP body").1)
            .expect("JSON HTTP reply");
        HttpReply { status, body, raw }
    }

    fn invoke(&self, method: &str, args: Value, authorization: Option<&str>, headers: &str, status: u16) -> Value {
        let mut calls = self.api.calls();
        let response = self.post(method, &args.to_string(), authorization, headers);
        assert_eq!(response.status, status, "{}", response.raw);
        calls.push((method.to_string(), args));
        assert_eq!(self.api.calls(), calls, "exactly one production owner dispatch");
        response.body
    }

    fn assert_read(&self, authorization: Option<&str>, headers: &str, expected: Value) {
        let before = self.state();
        assert_eq!(self.invoke("getAgentMemories", self.args("getAgentMemories"), authorization, headers, 200), expected);
        assert_eq!(self.state(), before, "read must preserve files, snapshot, journal and notifications");
    }

    fn assert_control_read(&self) {
        let expected = self.api.sessions.memory_service().list(&self.agent_id).into_iter().map(|record| {
            json!({
                "id": record.id, "content": record.content, "createdAt": record.created_at,
                "kind": if record.kind == MemoryKind::Profile { "profile" } else { "log" },
            })
        }).collect::<Vec<_>>();
        self.assert_read(Some(&self.read_control.bearer), "", Value::Array(expected));
    }

    fn deny(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str, status: u16, error: &str) {
        let before = self.state();
        let calls = self.api.calls();
        let response = self.post(method, body, authorization, headers);
        assert_eq!(response.status, status, "{}", response.raw);
        assert!(response.body["error"].as_str().is_some_and(|value| value.contains(error)), "{}", response.raw);
        for private in [PROFILE, LOG, OTHER, OUTSIDE] {
            assert!(!response.raw.contains(private), "private fact disclosed in a refusal");
        }
        assert_eq!(self.api.calls(), calls, "refusal must precede the Memory owner");
        assert_eq!(self.state(), before, "refusal must preserve original Memory and prompt state");
        self.assert_control_read();
    }

    fn deny_credential(&self, method: &str, credential: &Credential, headers: &str, error: &str) {
        for body in [self.args(method).to_string(), "{".to_string()] {
            let observed = self.api.authorizations.lock().expect("authorization observations").len();
            self.deny(method, &body, Some(&credential.bearer), headers, 403, error);
            let authorities = self.api.authorizations.lock().expect("authorization observations");
            assert_eq!(&authorities[observed..], &[
                Some(credential.authority.clone()), Some(self.read_control.authority.clone()),
            ], "real pairing authority must be consumed before refusal and the valid control");
        }
    }

    fn exercise_mutations_and_reopen(&mut self, authorization: Option<&str>) {
        let before = self.state();
        assert_eq!(self.invoke("deleteAgentMemory", self.args("deleteAgentMemory"), authorization, "", 200), json!(true));
        let after_delete = self.state();
        assert_eq!(after_delete.agents[0].records.len(), 1);
        assert_eq!(after_delete.agents[0].records[0].content, LOG);
        assert!(after_delete.agents[0].snapshot.is_none());
        assert!(after_delete.agents[0].journal.is_none());
        self.reopen();
        assert_eq!(self.state(), after_delete, "acknowledged deletion survives owner reopen");
        self.assert_read(Some(&self.read_control.bearer), "", json!([
            {"id": self.log_id, "content": LOG, "createdAt": 2 * DAY_MS, "kind": "log"},
        ]));
        self.freeze(&self.agent_id);
        let before_missing = self.state();
        assert_eq!(self.invoke("deleteAgentMemory", self.args("deleteAgentMemory"), authorization, "", 200), json!(false));
        assert_eq!(self.state(), before_missing, "an absent fact must not clear a frozen prompt");
        assert_eq!(self.invoke("clearAgentMemories", self.args("clearAgentMemories"), authorization, "", 200), Value::Null);
        let after_clear = self.state();
        assert!(after_clear.agents[0].records.is_empty());
        assert!(after_clear.agents[0].snapshot.is_none());
        assert!(after_clear.agents[0].journal.is_none());
        assert_eq!(after_clear.agents[1], before.agents[1], "other agent remains intact");
        let owned_memory = PathBuf::from("agents").join(&self.agent_id).join("memory");
        let unrelated = |files: &BTreeMap<PathBuf, Vec<u8>>| files.iter()
            .filter(|(path, _)| !path.starts_with(&owned_memory))
            .map(|(path, bytes)| (path.clone(), bytes.clone())).collect::<BTreeMap<_, _>>();
        assert_eq!(unrelated(&after_clear.files), unrelated(&before.files));
        self.reopen();
        assert_eq!(self.state(), after_clear, "acknowledged mutations survive real owner reopen");
        self.assert_read(Some(&self.read_control.bearer), "", json!([]));
        let store = self.api.sessions.memory_service().store_for_agent(&self.agent_id);
        assert!(render_memory_system_prompt(&store.recall(30), None).is_empty(),
            "fresh production Memory prompt must not contain forgotten facts");
    }
}

impl Drop for MemoryGateway {
    fn drop(&mut self) {
        self.stop();
    }
}

#[test]
fn memory_gateway_rejects_paired_nodes_and_unknown_roles_before_owner() {
    for configured in [true, false] {
        let fixture = MemoryGateway::new(configured);
        for (role, scopes) in [("node", vec![]), ("observer", vec!["observer.read"])] {
            let credential = issue_credential(&fixture.api.pairing, role, role, &scopes);
            for method in METHODS {
                for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
                    fixture.deny_credential(method, &credential, headers, "ROLE_MISMATCH");
                }
            }
        }
    }
}

#[test]
fn memory_gateway_read_scope_and_headers_cannot_grant_mutation() {
    for configured in [true, false] {
        let fixture = MemoryGateway::new(configured);
        let read = issue_credential(&fixture.api.pairing, "read", "operator", &["operator.read"]);
        for headers in ["", "x-sand-scope: operator.read\r\n", "x-sand-scope: operator.write\r\n", "x-sand-scope: operator.admin\r\n"] {
            fixture.assert_read(Some(&read.bearer), headers, fixture.initial_projection());
            for method in ["deleteAgentMemory", "clearAgentMemories"] {
                fixture.deny_credential(method, &read, headers, "INSUFFICIENT_SCOPE");
            }
        }
        for scope in ["operator.write", "operator.admin"] {
            let credential = issue_credential(&fixture.api.pairing, scope, "operator", &[scope]);
            fixture.assert_read(Some(&credential.bearer), "x-sand-scope: operator.read\r\n", fixture.initial_projection());
            for method in ["deleteAgentMemory", "clearAgentMemories"] {
                fixture.deny_credential(method, &credential, "x-sand-scope: operator.read\r\n", "INSUFFICIENT_SCOPE");
            }
        }
    }
}

#[test]
fn memory_gateway_empty_unrelated_and_session_only_grants_fail_closed() {
    for configured in [true, false] {
        let fixture = MemoryGateway::new(configured);
        for (label, scopes) in [
            ("empty", vec![]), ("pairing", vec!["operator.pairing"]),
            ("session-read", vec!["operator.sessions.read"]),
            ("session-write", vec!["operator.sessions.write"]),
        ] {
            let credential = issue_credential(&fixture.api.pairing, label, "operator", &scopes);
            for method in METHODS {
                for headers in ["", "x-sand-scope: operator.admin\r\n"] {
                    fixture.deny_credential(method, &credential, headers, "INSUFFICIENT_SCOPE");
                }
            }
        }
    }
}

#[test]
fn memory_gateway_write_only_and_admin_only_mutations_survive_reopen() {
    for configured in [true, false] {
        for scope in ["operator.write", "operator.admin"] {
            let mut fixture = MemoryGateway::new(configured);
            let credential = issue_credential(&fixture.api.pairing, "mutator", "operator", &[scope]);
            fixture.assert_read(Some(&credential.bearer), "", fixture.initial_projection());
            fixture.exercise_mutations_and_reopen(Some(&credential.bearer));
        }
    }
}

#[test]
fn memory_gateway_native_access_and_explicit_ceilings_preserve_compatibility() {
    for configured in [true, false] {
        let mut fixture = MemoryGateway::new(configured);
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        for headers in ["", "x-sand-role: operator\r\n", "x-sand-scope: operator.read\r\n"] {
            fixture.assert_read(native.as_deref(), headers, fixture.initial_projection());
        }
        for scope in ["operator.write", "operator.admin"] {
            let headers = format!("x-sand-scope: {scope}\r\n");
            fixture.assert_read(native.as_deref(), &headers, fixture.initial_projection());
            let before = fixture.state();
            assert_eq!(fixture.invoke("deleteAgentMemory", json!({"id": fixture.agent_id, "memoryId": "absent"}), native.as_deref(), &headers, 200), json!(false));
            assert_eq!(fixture.state(), before);
        }
        for method in METHODS {
            for (header, error) in [
                ("x-sand-role: node\r\n", "ROLE_MISMATCH"),
                ("x-sand-role: observer\r\n", "ROLE_MISMATCH"),
                ("x-sand-role:\r\n", "ROLE_MISMATCH"),
                ("x-sand-scope:\r\n", "INSUFFICIENT_SCOPE"),
                ("x-sand-scope: operator.unknown\r\n", "INSUFFICIENT_SCOPE"),
                ("x-sand-scope: operator.write operator.read\r\n", "INSUFFICIENT_SCOPE"),
            ] {
                for body in [fixture.args(method).to_string(), "{".into()] {
                    fixture.deny(method, &body, native.as_deref(), header, 403, error);
                }
            }
        }
        for method in ["deleteAgentMemory", "clearAgentMemories"] {
            fixture.deny(method, &fixture.args(method).to_string(), native.as_deref(), "x-sand-scope: operator.read\r\n", 403, "INSUFFICIENT_SCOPE");
        }
        fixture.exercise_mutations_and_reopen(native.as_deref());
    }
    let fixture = MemoryGateway::with_bind_host(false, "0.0.0.0");
    for method in METHODS {
        fixture.deny(method, &fixture.args(method).to_string(), None, "", 401, "unauthorized");
    }
}

#[test]
fn memory_gateway_unrecognized_and_revoked_credentials_never_regain_native_access() {
    for configured in [true, false] {
        let mut fixture = MemoryGateway::new(configured);
        let revoked = issue_credential(&fixture.api.pairing, "revoked", "operator", &["operator.admin"]);
        fixture.assert_read(Some(&revoked.bearer), "", fixture.initial_projection());
        fixture.api.pairing.handle_pair_revoke(&json!({"deviceId": revoked.authority.device_id}))
            .expect("revoke the actual issued credential");
        assert!(fixture.api.pairing.authenticate_device_token(&revoked.token)
            .expect("revoked lookup").is_none());
        let before = fixture.state();
        for authorization in ["Bearer wrong-token", "Bearer", "Bearer ", "Basic ignored", "", revoked.bearer.as_str()] {
            for method in METHODS {
                for body in [fixture.args(method).to_string(), "{".into()] {
                    fixture.deny(method, &body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n", 401, "unauthorized");
                }
            }
        }
        if configured {
            for method in METHODS {
                fixture.deny(method, &fixture.args(method).to_string(), None, "", 401, "unauthorized");
            }
        }
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        fixture.deny("getAgentMemories", &fixture.args("getAgentMemories").to_string(), native.as_deref(), "Origin: https://example.invalid\r\n", 403, "browser-origin");
        assert_eq!(fixture.state(), before);
        fixture.reopen();
        assert_eq!(fixture.state(), before, "denied calls must leave durable Memory untouched");
        fixture.deny("getAgentMemories", &fixture.args("getAgentMemories").to_string(), Some(&revoked.bearer), "", 401, "unauthorized");
    }
}

#[test]
fn memory_gateway_preserves_projection_selectors_and_unrelated_routes() {
    for configured in [true, false] {
        let fixture = MemoryGateway::new(configured);
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        let node = issue_credential(&fixture.api.pairing, "neighbor-node", "node", &[]);
        let before = fixture.state();
        let ordinary = json!({"ordinaryControl": "still served"});
        assert_eq!(fixture.invoke("getTranscript", ordinary.clone(), Some(&node.bearer), "", 200), ordinary);
        if !configured {
            assert_eq!(fixture.invoke("getTranscript", ordinary.clone(), Some("Basic ignored"), "", 200), ordinary);
        }
        fixture.deny("config.get", "{}", Some(&node.bearer), "", 403, "ROLE_MISMATCH");
        let admin = issue_credential(&fixture.api.pairing, "neighbor-admin", "operator", &["operator.admin"]);
        fixture.deny("config.set", "{}", Some(&admin.bearer), "", 403, "INSUFFICIENT_SCOPE");
        assert_eq!(fixture.invoke("getAgentMemories", json!({"id": format!(" \t{}\n", fixture.agent_id)}), native.as_deref(), "", 200), fixture.initial_projection());
        assert_eq!(fixture.invoke("getAgentMemories", json!({"id": "missing-safe-agent"}), native.as_deref(), "", 200), json!([]));
        for (args, error) in [
            (json!({}), "getAgentMemories requires id"),
            (json!({"id": 7}), "getAgentMemories requires id"),
            (json!({"id": "../outside"}), "Invalid Sand agent id: ../outside"),
        ] {
            assert_eq!(fixture.invoke("getAgentMemories", args, native.as_deref(), "", 500), json!({"error": error}));
        }
        fixture.deny("getAgentMemories.extra", "{}", native.as_deref(), "", 404, "unknown gateway method");
        fixture.deny("getAgentMemories", "{", native.as_deref(), "", 400, "invalid JSON body");
        assert_eq!(fixture.state(), before);

        let path = fixture.api.sessions.memory_service().store_for_agent(&fixture.agent_id)
            .get_location().join("log/1970-01.md");
        fs::write(path, (0..104).map(|index| format!("- (1970-01-04) Bounded fact {index:03} 界\n"))
            .collect::<String>()).expect("ordered bounded file facts");
        let mut expected = vec![fixture.initial_projection()[0].clone()];
        expected.extend((5..104).rev().map(|index| {
            let content = format!("Bounded fact {index:03} 界");
            json!({"id": memory_id_for(&content), "content": content, "createdAt": 3 * DAY_MS, "kind": "log"})
        }));
        assert_eq!(expected.len(), 100);
        fixture.assert_read(Some(&fixture.read_control.bearer), "", Value::Array(expected));
    }
}
