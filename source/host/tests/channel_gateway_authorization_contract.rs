//! CGWA-v1 uses actual HTTP ingress, pairing grants and Session credential
//! writers. The adapter observes dispatch; it does not decide authorization or
//! manufacture credential effects. External channel connectivity is not tested.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::memory::memory_service::{MemoryKind, MemoryRecord};
use mahayana_host_runtime::extensions::session::agent_db::PendingMemoryMutation;
use mahayana_host_runtime::extensions::session::agent_db_serde::MemoryPromptSnapshot;
use mahayana_host_runtime::extensions::session::channel_store::ChannelConnection;
use mahayana_host_runtime::extensions::session::gateway::SessionGatewayError;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::transcript_manager::TranscriptManager;
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayEventHub, GatewayServer, GatewayServerDeps,
    start_gateway_server,
};
use serde_json::{Value, json};

const NATIVE_TOKEN: &str = "cgwa-native-fixture-token";
const ORIGINAL_TOKEN: &str = "cgwa-original-slack-secret";
const REPLACEMENT_TOKEN: &str = "cgwa-replacement-slack-secret";
const OTHER_PLATFORM_TOKEN: &str = "cgwa-discord-sentinel-secret";
const OTHER_AGENT_TOKEN: &str = "cgwa-other-agent-secret";
const PRIVATE_MEMORY: &str = "CGWA private memory sentinel ☃";
const OTHER_MEMORY: &str = "CGWA other Agent memory sentinel 界";
const METHODS: [&str; 2] = ["connectChannel", "disconnectChannel"];
const ROLE_ERROR: &str = "ROLE_MISMATCH: Channel credential methods require operator role";

fn scope_error(method: &str) -> String {
    format!("INSUFFICIENT_SCOPE: Channel credential access is not granted for method: {method}")
}

struct Credential {
    bearer: String,
    token: String,
    authority: DeviceTokenAuthorization,
}

fn issue(owner: &DevicePairingOwner, label: &str, role: &str, scopes: &[&str]) -> Credential {
    let device_id = format!("cgwa-{label}");
    let pending = owner.handle_pair_request(&json!({
        "deviceId": device_id, "publicKey": format!("cgwa-key-{label}"),
        "role": role, "roles": [role], "scopes": scopes,
    })).expect("actual pairing request");
    owner.handle_pair_approve(&json!({"requestId": pending["requestId"]}))
        .expect("actual approved grant");
    let rotated = owner.handle_pair_rotate_token(&json!({"deviceId": device_id, "role": role}))
        .expect("actual token rotation");
    let token = rotated["token"].as_str().expect("issued bearer").to_string();
    let authority = owner.authenticate_device_token(&token).unwrap().expect("recognized issued token");
    assert_eq!(authority.device_id, device_id);
    assert_eq!(authority.role, role);
    assert_eq!(authority.scopes, scopes.iter().map(|scope| scope.to_string()).collect::<Vec<_>>());
    Credential { bearer: format!("Bearer {token}"), token, authority }
}

struct CredentialApi {
    pairing: DevicePairingOwner,
    sessions: Arc<ProductionSessionWorkers>,
    manager: TranscriptManager,
    authorizations: Mutex<Vec<Option<DeviceTokenAuthorization>>>,
    bindings: Mutex<Vec<(DeviceTokenAuthorization, String, Value)>>,
    calls: Mutex<Vec<(String, Value)>>,
}

impl CredentialApi {
    fn open(root: &Path) -> Self {
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        Self {
            pairing: DevicePairingOwner::new(root.join("pairing.sqlite")).unwrap(),
            manager: TranscriptManager::new(root, Arc::clone(&sessions)), sessions,
            authorizations: Mutex::new(Vec::new()), bindings: Mutex::new(Vec::new()),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<(String, Value)> { self.calls.lock().unwrap().clone() }
}

impl GatewayApi for CredentialApi {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let authorization = self.pairing.authenticate_device_token(token)?;
        self.authorizations.lock().unwrap().push(authorization.clone());
        Ok(authorization)
    }

    fn authorize_device_token_request(
        &self, authorization: &DeviceTokenAuthorization, method: &str, args: &Value,
    ) -> Result<(), GatewayCommandError> {
        // These native channel/Memory methods have no addressed-device binding
        // in UnifiedGatewayApi. Observe this post-body boundary without adding
        // a test permission decision; the real pairing owner supplied the grant.
        self.bindings.lock().unwrap().push((authorization.clone(), method.into(), args.clone()));
        Ok(())
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls.lock().unwrap().push((method.into(), args.clone()));
        if let Some(result) = self.manager.dispatch_memory_gateway_call(method, &args) {
            return result.map_err(GatewayCommandError::Internal);
        }
        if let Some(result) = self.manager.dispatch_session_gateway_call(method, &args) {
            return result.map_err(|error| match error {
                SessionGatewayError::BadRequest(message) => GatewayCommandError::BadRequest(message),
                SessionGatewayError::Internal(message) => GatewayCommandError::Internal(message),
            });
        }
        match method {
            // Ordinary-route controls only; credential and Memory methods above
            // always execute their actual production owners.
            "getTranscript" | "config.get" | "config.set" => Ok(args),
            _ => Err(GatewayCommandError::UnknownMethod(method.into())),
        }
    }
}

impl Drop for CredentialApi {
    fn drop(&mut self) { self.manager.dispose(); }
}

#[derive(Debug, PartialEq)]
struct AgentState {
    channels: Vec<ChannelConnection>,
    secrets: Vec<Value>,
    transcript: Vec<Value>,
    memories: Vec<MemoryRecord>,
    snapshot: Option<MemoryPromptSnapshot>,
    journal: Option<PendingMemoryMutation>,
}

#[derive(Debug, PartialEq)]
struct DurableState {
    files: BTreeMap<PathBuf, Vec<u8>>,
    agents: Vec<AgentState>,
}

fn collect_files(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => panic!("read owned fixture directory {}: {error}", path.display()),
    };
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            collect_files(root, &path, files);
        } else {
            files.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read(path).unwrap());
        }
    }
}

struct HttpReply { status: u16, body: Value, raw: String }

struct OwnedRoot(PathBuf);

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        let removed = fs::remove_dir_all(&self.0);
        if !thread::panicking() { removed.expect("remove only the closed owned fixture root"); }
    }
}

struct CredentialGateway {
    // Fields drop in this order after stop: listener, API/SQLite owners, root.
    server: Option<GatewayServer>,
    api: Arc<CredentialApi>,
    root: OwnedRoot,
    config: GatewayServerConfig,
    agent_id: String,
    other_id: String,
    memory_id: String,
    write_control: Credential,
}

impl CredentialGateway {
    fn new(configured_token: bool) -> Self { Self::with_bind_host(configured_token, "127.0.0.1") }

    fn with_bind_host(configured_token: bool, host: &str) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-channel-gateway-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let api = Arc::new(CredentialApi::open(&root));
        let agent_id = api.sessions.materialize_new_session(None, "user", None).unwrap().id;
        let other_id = api.sessions.materialize_new_session(None, "user", None).unwrap().id;
        for (id, platform, token) in [
            (&agent_id, "slack", ORIGINAL_TOKEN), (&agent_id, "discord", OTHER_PLATFORM_TOKEN),
            (&other_id, "slack", OTHER_AGENT_TOKEN), (&other_id, "discord", OTHER_AGENT_TOKEN),
        ] {
            assert!(api.sessions.store_connector_credential(id, platform, "token", token).unwrap());
        }
        let memory_id = api.sessions.memory_service().store_for_agent(&agent_id)
            .add_memory(PRIVATE_MEMORY, 86_400_000, MemoryKind::Profile).unwrap().unwrap().id;
        api.sessions.memory_service().store_for_agent(&other_id)
            .add_memory(OTHER_MEMORY, 86_400_000, MemoryKind::Profile).unwrap().unwrap();
        for (id, render) in [(&agent_id, PRIVATE_MEMORY), (&other_id, OTHER_MEMORY)] {
            api.sessions.set_agent_memory_prompt_snapshot(id, &json!({"render":render,"compactionEpoch":0})).unwrap();
        }
        let write_control = issue(&api.pairing, "write-control", "operator", &["operator.write"]);
        let config = GatewayServerConfig {
            host: host.into(), port: None,
            auth_token: configured_token.then(|| NATIVE_TOKEN.into()), tls: None,
        };
        let mut fixture = Self { server: None, api, root: OwnedRoot(root), config, agent_id, other_id, memory_id, write_control };
        fixture.start();
        let initial = fixture.state();
        assert_eq!(initial.agents[0].channels.len(), 2);
        assert_eq!(initial.agents[1].channels.len(), 2);
        assert_eq!(initial.agents[0].secrets[0]["token"], ORIGINAL_TOKEN);
        assert!(initial.agents.iter().all(|agent| agent.snapshot.is_some() && agent.journal.is_none()));
        fixture
    }

    fn start(&mut self) {
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: self.api.clone(), events: GatewayEventHub::default(), local_exec: None,
            webauthn: None, config: self.config.clone(), started_at: 1,
        }).expect("actual channel credential HTTP gateway"));
    }

    fn stop(&mut self) {
        if let Some(server) = self.server.take() { server.close(); }
        self.api.manager.dispose();
    }

    fn reopen(&mut self) {
        self.stop();
        self.api = Arc::new(CredentialApi::open(&self.root.0));
        self.start();
    }

    fn state(&self) -> DurableState {
        let mut files = BTreeMap::new();
        collect_files(&self.root.0, self.api.sessions.connector_secret_store().secrets_root(), &mut files);
        let agents = [&self.agent_id, &self.other_id].into_iter().map(|id| {
            let channel_store = self.api.sessions.open_channel_store(id).unwrap();
            collect_files(&self.root.0, channel_store.get_location(), &mut files);
            let memory = self.api.sessions.memory_service().store_for_agent(id);
            collect_files(&self.root.0, &memory.get_location(), &mut files);
            let db = self.api.sessions.open_agent_db_owner(id).unwrap();
            AgentState {
                channels: self.api.sessions.list_agent_channels(id).unwrap(),
                secrets: ["slack", "discord"].iter().map(|platform| {
                    Value::Object(self.api.sessions.connector_secret_store().read(id, platform))
                }).collect(),
                transcript: self.api.sessions.read_agent_transcript_entries(id).unwrap(),
                memories: memory.list_memories(usize::MAX),
                snapshot: db.get_memory_prompt_snapshot().unwrap(),
                journal: db.read_pending_memory_mutation().unwrap(),
            }
        }).collect();
        DurableState { files, agents }
    }

    fn args(&self, method: &str, token: &str) -> Value {
        match method {
            "connectChannel" => json!({"id":self.agent_id,"platform":"slack","token":token}),
            "disconnectChannel" => json!({"id":self.agent_id,"platform":"slack"}),
            "deleteAgentMemory" => json!({"id":self.agent_id,"memoryId":self.memory_id}),
            _ => json!({"id":self.agent_id}),
        }
    }

    fn post_with_host(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str, host: &str) -> HttpReply {
        let mut stream = TcpStream::connect(("127.0.0.1", self.server.as_ref().unwrap().port())).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let authorization = authorization.map(|value| format!("Authorization: {value}\r\n")).unwrap_or_default();
        let request = format!(
            "POST /api/{method} HTTP/1.1\r\nHost: {host}\r\n{authorization}{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len(),
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();
        let mut raw = String::new();
        stream.read_to_string(&mut raw).expect("complete HTTP response");
        let status = raw.lines().next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = serde_json::from_str(raw.split_once("\r\n\r\n").unwrap().1).expect("JSON HTTP response");
        HttpReply { status, body, raw }
    }

    fn post(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str) -> HttpReply {
        self.post_with_host(method, body, authorization, headers, "127.0.0.1")
    }

    fn assert_no_secret(reply: &HttpReply) {
        for secret in [NATIVE_TOKEN, ORIGINAL_TOKEN, REPLACEMENT_TOKEN, OTHER_PLATFORM_TOKEN, OTHER_AGENT_TOKEN] {
            assert!(!reply.raw.contains(secret), "credential material must not occur in an HTTP reply");
        }
    }

    fn invoke(&self, method: &str, args: Value, authorization: Option<&str>, headers: &str) -> Value {
        let mut calls = self.api.calls();
        let mut bindings = self.api.bindings.lock().unwrap().clone();
        let observed = self.api.authorizations.lock().unwrap().len();
        let response = self.post(method, &args.to_string(), authorization, headers);
        assert_eq!(response.status, 200, "{}", response.raw);
        Self::assert_no_secret(&response);
        let authorities = self.api.authorizations.lock().unwrap();
        assert!(authorities[observed..].len() <= 1, "at most one pairing authentication for this HTTP request");
        if let Some(Some(authority)) = authorities[observed..].first() {
            bindings.push((authority.clone(), method.into(), args.clone()));
        }
        drop(authorities);
        assert_eq!(*self.api.bindings.lock().unwrap(), bindings, "admitted paired requests prove the binding observer is live");
        calls.push((method.into(), args));
        assert_eq!(self.api.calls(), calls, "one actual business owner invocation per admitted request");
        response.body
    }

    fn expected_connections(connected: bool) -> Value {
        let mut result = vec![json!({"platform":"discord","label":"Discord","status":"configured"})];
        if connected { result.push(json!({"platform":"slack","label":"Slack","status":"configured"})); }
        Value::Array(result)
    }

    fn valid_after_refusal(&self, method: &str) {
        // Capture denial invariants before entering this deliberately admitted
        // control. Disconnect executes a real delete, followed by a real connect
        // restoring the fixture so later refusal cases have readable sentinels.
        let before = self.state();
        let authorization = Some(self.write_control.bearer.as_str());
        let reply = self.invoke(method, self.args(method, ORIGINAL_TOKEN), authorization, "");
        assert_eq!(reply, Self::expected_connections(method == "connectChannel"));
        if method == "disconnectChannel" {
            assert_eq!(self.api.sessions.get_connector_secret(&self.agent_id, "slack", "token").unwrap(), None);
            assert_eq!(self.invoke("connectChannel", self.args("connectChannel", ORIGINAL_TOKEN), authorization, ""), Self::expected_connections(true));
        }
        assert_eq!(self.state(), before, "the real admitted control restores only the addressed connection");
    }

    fn deny(&self, method: &str, body: &str, authorization: Option<&str>, headers: &str, status: u16, error: &str) {
        let before = self.state();
        let calls = self.api.calls();
        let bindings = self.api.bindings.lock().unwrap().clone();
        let response = self.post(method, body, authorization, headers);
        assert_eq!(response.status, status, "{}", response.raw);
        if status == 400 {
            assert!(response.body["error"].as_str().is_some_and(|message| message.starts_with(error)), "{}", response.raw);
        } else {
            assert_eq!(response.body, json!({"error":error}), "fixed refusal text");
        }
        Self::assert_no_secret(&response);
        for private in [PRIVATE_MEMORY, OTHER_MEMORY] {
            assert!(!response.raw.contains(private), "no private Memory in a refusal");
        }
        assert_eq!(self.api.calls(), calls, "denial precedes actual Session/Memory dispatch");
        assert_eq!(*self.api.bindings.lock().unwrap(), bindings, "denial precedes post-body device binding");
        assert_eq!(self.state(), before, "denied request preserves raw bytes and real owner projections");
    }

    fn deny_credential(&self, method: &str, credential: &Credential, headers: &str, error: &str) {
        for body in [self.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
            let observed = self.api.authorizations.lock().unwrap().len();
            self.deny(method, &body, Some(&credential.bearer), headers, 403, error);
            assert_eq!(&self.api.authorizations.lock().unwrap()[observed..], &[Some(credential.authority.clone())],
                "the actual issued pairing grant, not headers, reaches the authorization gate");
            self.valid_after_refusal(method);
        }
    }

    fn assert_untouched_except_target(&self, before: &DurableState, after: &DurableState) {
        let target_secret = PathBuf::from("connector-secrets").join(&self.agent_id).join("slack.json");
        let target_metadata = PathBuf::from("agents").join(&self.agent_id).join("channels/slack");
        let unrelated = |state: &DurableState| state.files.iter()
            .filter(|(path, _)| *path != &target_secret && !path.starts_with(&target_metadata))
            .map(|(path, bytes)| (path.clone(), bytes.clone())).collect::<BTreeMap<_, _>>();
        assert_eq!(unrelated(after), unrelated(before), "other platform, other Agent and Memory bytes are unchanged");
        assert_eq!(after.agents[1], before.agents[1]);
        assert_eq!(after.agents[0].secrets[1], before.agents[0].secrets[1]);
        assert_eq!(after.agents[0].transcript, before.agents[0].transcript);
        assert_eq!(after.agents[0].memories, before.agents[0].memories);
        assert_eq!(after.agents[0].snapshot, before.agents[0].snapshot);
        assert_eq!(after.agents[0].journal, before.agents[0].journal);
    }

    fn exercise_mutations_and_reopen(&mut self, authorization: Option<&str>, headers: &str) {
        let before = self.state();
        assert_eq!(self.invoke("connectChannel", self.args("connectChannel", REPLACEMENT_TOKEN), authorization, headers), Self::expected_connections(true));
        let secret = self.api.sessions.connector_secret_store().file_path(&self.agent_id, "slack");
        assert_eq!(serde_json::from_slice::<Value>(&fs::read(&secret).unwrap()).unwrap(), json!({"token":REPLACEMENT_TOKEN}));
        assert_eq!(self.api.sessions.get_connector_secret(&self.agent_id, "slack", "token").unwrap().as_deref(), Some(REPLACEMENT_TOKEN));
        let connected = self.state();
        assert_ne!(connected.files, before.files, "accepted connect really changed credential bytes");
        self.assert_untouched_except_target(&before, &connected);
        self.reopen();
        assert_eq!(self.state(), connected, "accepted credential and metadata survive actual owner reopen");
        assert_eq!(self.invoke("disconnectChannel", self.args("disconnectChannel", ""), authorization, headers), Self::expected_connections(false));
        assert!(!secret.exists());
        assert!(!self.api.sessions.open_channel_store(&self.agent_id).unwrap().config_path("slack").exists());
        assert_eq!(self.api.sessions.get_connector_secret(&self.agent_id, "slack", "token").unwrap(), None);
        let disconnected = self.state();
        self.assert_untouched_except_target(&before, &disconnected);
        self.reopen();
        assert_eq!(self.state(), disconnected, "accepted credential deletion survives actual owner reopen");
        assert_eq!(self.invoke("disconnectChannel", self.args("disconnectChannel", ""), authorization, headers), Self::expected_connections(false));
        assert_eq!(self.state(), disconnected, "existing absent-platform response remains idempotent");
    }
}

impl Drop for CredentialGateway {
    fn drop(&mut self) { self.stop(); }
}

#[test]
fn channel_gateway_rejects_read_node_and_unknown_roles_before_owner() {
    for configured in [true, false] {
        let fixture = CredentialGateway::new(configured);
        let read = issue(&fixture.api.pairing, "readonly", "operator", &["operator.read"]);
        for method in METHODS {
            fixture.deny_credential(method, &read, "", &scope_error(method));
        }
        for (role, scopes) in [("node", vec![]), ("observer", vec!["observer.read"])] {
            let credential = issue(&fixture.api.pairing, role, role, &scopes);
            for method in METHODS {
                for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n"] {
                    fixture.deny_credential(method, &credential, headers, ROLE_ERROR);
                }
            }
        }
        for headers in ["x-sand-role: node\r\n", "x-sand-role: observer\r\n", "x-sand-role:\r\n"] {
            for method in METHODS {
                fixture.deny_credential(method, &fixture.write_control, headers, ROLE_ERROR);
            }
        }
    }
}

#[test]
fn channel_gateway_headers_and_unrelated_grants_cannot_authorize_credential_writes() {
    for configured in [true, false] {
        let fixture = CredentialGateway::new(configured);
        for (label, scopes) in [
            ("read", vec!["operator.read"]), ("empty", vec![]),
            ("pairing", vec!["operator.pairing"]), ("sessions-read", vec!["operator.sessions.read"]),
            ("sessions-write", vec!["operator.sessions.write"]), ("unrelated", vec!["operator.other"]),
        ] {
            let credential = issue(&fixture.api.pairing, label, "operator", &scopes);
            for method in METHODS {
                for header in ["", "x-sand-scope: operator.write\r\n", "x-sand-scope: operator.admin\r\n"] {
                    fixture.deny_credential(method, &credential, header, &scope_error(method));
                }
            }
        }
        for scope in ["operator.write", "operator.admin"] {
            let credential = issue(&fixture.api.pairing, scope, "operator", &[scope]);
            for header in [
                "x-sand-scope: operator.read\r\n", "x-sand-scope:\r\n",
                "x-sand-scope: operator.unknown\r\n", "x-sand-scope: operator.write operator.admin\r\n",
                "x-sand-scope: operator.write,operator.admin\r\n",
            ] {
                for method in METHODS {
                    fixture.deny_credential(method, &credential, header, &scope_error(method));
                }
            }
        }
    }
}

#[test]
fn channel_gateway_write_and_admin_credentials_persist_through_reopen() {
    for configured in [true, false] {
        for scope in ["operator.write", "operator.admin"] {
            for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n", "x-sand-scope: operator.admin\r\n"] {
                let mut fixture = CredentialGateway::new(configured);
                let credential = issue(&fixture.api.pairing, "mutator", "operator", &[scope]);
                fixture.exercise_mutations_and_reopen(Some(&credential.bearer), headers);
                let reopened = fixture.api.pairing.authenticate_device_token(&credential.token)
                    .expect("authenticate the persisted credential after reopen")
                    .expect("the same persisted token must remain valid");
                assert_eq!(reopened.device_id, credential.authority.device_id);
                assert_eq!(reopened.role, credential.authority.role);
                assert_eq!(reopened.scopes, credential.authority.scopes);
                // Durable identity/grants survive restart; an authentication
                // proof belongs to the owner instance that actually issued it.
                assert_ne!(reopened, credential.authority);
            }
        }
    }
}

#[test]
fn channel_gateway_native_credentials_and_header_ceilings_preserve_compatibility() {
    for configured in [true, false] {
        let fixture = CredentialGateway::new(configured);
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        for method in METHODS {
            for (headers, error) in [
                ("x-sand-role: node\r\n", ROLE_ERROR.to_string()),
                ("x-sand-role: observer\r\n", ROLE_ERROR.to_string()),
                ("x-sand-role:\r\n", ROLE_ERROR.to_string()),
                ("x-sand-scope: operator.read\r\n", scope_error(method)),
                ("x-sand-scope:\r\n", scope_error(method)),
                ("x-sand-scope: operator.unknown\r\n", scope_error(method)),
                ("x-sand-scope: operator.write operator.read\r\n", scope_error(method)),
            ] {
                for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
                    fixture.deny(method, &body, native.as_deref(), headers, 403, &error);
                    fixture.valid_after_refusal(method);
                }
            }
        }
        for headers in ["", "x-sand-role: operator\r\n", "x-sand-scope: operator.write\r\n", "x-sand-scope: operator.admin\r\n"] {
            let mut accepted = CredentialGateway::new(configured);
            accepted.exercise_mutations_and_reopen(native.as_deref(), headers);
        }
    }
    let fixture = CredentialGateway::with_bind_host(false, "0.0.0.0");
    for method in METHODS {
        for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
            fixture.deny(method, &body, None, "", 401, "unauthorized");
            fixture.valid_after_refusal(method);
        }
    }
}

#[test]
fn channel_gateway_explicit_bad_or_revoked_credentials_never_regain_ambient_authority() {
    for configured in [true, false] {
        let mut fixture = CredentialGateway::new(configured);
        let revoked = issue(&fixture.api.pairing, "revoked", "operator", &["operator.admin"]);
        assert_eq!(fixture.invoke("connectChannel", fixture.args("connectChannel", ORIGINAL_TOKEN), Some(&revoked.bearer), ""), CredentialGateway::expected_connections(true));
        fixture.api.pairing.handle_pair_revoke(&json!({"deviceId":revoked.authority.device_id})).unwrap();
        assert!(fixture.api.pairing.authenticate_device_token(&revoked.token).unwrap().is_none());
        let before = fixture.state();
        for authorization in ["Bearer wrong-fixture-token", "Bearer", "Bearer ", "Basic ignored", "", revoked.bearer.as_str()] {
            for method in METHODS {
                for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
                    fixture.deny(method, &body, Some(authorization), "x-sand-role: operator\r\nx-sand-scope: operator.admin\r\n", 401, "unauthorized");
                    fixture.valid_after_refusal(method);
                }
            }
        }
        if configured {
            for method in METHODS {
                for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
                    fixture.deny(method, &body, None, "", 401, "unauthorized");
                    fixture.valid_after_refusal(method);
                }
            }
        }
        assert_eq!(fixture.state(), before);
        fixture.reopen();
        assert_eq!(fixture.state(), before, "refused requests leave durable credential/metadata and Memory state intact");
        assert!(fixture.api.pairing.authenticate_device_token(&revoked.token).unwrap().is_none());
        for method in METHODS {
            fixture.deny(method, &fixture.args(method, REPLACEMENT_TOKEN).to_string(), Some(&revoked.bearer), "", 401, "unauthorized");
            fixture.valid_after_refusal(method);
        }
    }
}

#[test]
fn channel_gateway_registry_origin_and_memory_controls_remain_intact() {
    for configured in [true, false] {
        let fixture = CredentialGateway::new(configured);
        let native = configured.then(|| format!("Bearer {NATIVE_TOKEN}"));
        let node = issue(&fixture.api.pairing, "neighbor-node", "node", &[]);
        let read = issue(&fixture.api.pairing, "neighbor-read", "operator", &["operator.read"]);
        let admin = issue(&fixture.api.pairing, "neighbor-admin", "operator", &["operator.admin"]);
        let before = fixture.state();
        for method in ["getAgentChannels", "refreshChannel"] {
            assert_eq!(fixture.invoke(method, fixture.args(method, ""), Some(&node.bearer), ""), CredentialGateway::expected_connections(true));
        }
        for method in ["connectChannel.extra", "disconnectChannel.extra", "channels.status", "channels.start", "channels.stop", "channels.logout"] {
            fixture.deny(method, "{", native.as_deref(), "", 404, &format!("unknown gateway method: {method}"));
        }
        for method in METHODS {
            for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
                fixture.deny(method, &body, native.as_deref(), "Origin: https://example.invalid\r\n", 403, "browser-origin gateway requests are not allowed");
                fixture.valid_after_refusal(method);
            }
            fixture.deny(method, "{", Some(&admin.bearer), "", 400, "invalid JSON body:");
            fixture.valid_after_refusal(method);
            if !configured {
                for body in [fixture.args(method, REPLACEMENT_TOKEN).to_string(), "{".into()] {
                    let calls = fixture.api.calls();
                    let bindings = fixture.api.bindings.lock().unwrap().clone();
                    let state = fixture.state();
                    let response = fixture.post_with_host(method, &body, None, "", "example.invalid");
                    assert_eq!(response.status, 403);
                    assert_eq!(response.body, json!({"error":"untrusted gateway host"}));
                    CredentialGateway::assert_no_secret(&response);
                    assert_eq!(fixture.api.calls(), calls);
                    assert_eq!(*fixture.api.bindings.lock().unwrap(), bindings);
                    assert_eq!(fixture.state(), state);
                    fixture.valid_after_refusal(method);
                }
            }
        }
        let memory = json!([{"id":fixture.memory_id,"content":PRIVATE_MEMORY,"createdAt":86_400_000,"kind":"profile"}]);
        for credential in [&read, &fixture.write_control, &admin] {
            assert_eq!(fixture.invoke("getAgentMemories", fixture.args("getAgentMemories", ""), Some(&credential.bearer), ""), memory);
        }
        for credential in [&fixture.write_control, &admin] {
            assert_eq!(fixture.invoke("deleteAgentMemory", json!({"id":fixture.agent_id,"memoryId":"cgwa-absent-memory"}),
                Some(&credential.bearer), ""), json!(false), "Memory write/admin still reaches the real mutation owner");
        }
        for method in ["getAgentMemories", "deleteAgentMemory", "clearAgentMemories"] {
            fixture.deny(method, &fixture.args(method, "").to_string(), Some(&node.bearer), "", 403,
                "ROLE_MISMATCH: Memory methods require operator role");
        }
        for method in ["deleteAgentMemory", "clearAgentMemories"] {
            fixture.deny(method, &fixture.args(method, "").to_string(), Some(&read.bearer), "", 403,
                &format!("INSUFFICIENT_SCOPE: Memory access is not granted for method: {method}"));
        }
        let ordinary = json!({"ordinaryControl":"still served"});
        assert_eq!(fixture.invoke("getTranscript", ordinary.clone(), Some(&node.bearer), ""), ordinary);
        assert_eq!(fixture.invoke("getTranscript", ordinary.clone(), native.as_deref(), ""), ordinary);
        fixture.deny("getTranscript", &ordinary.to_string(), Some("Basic ignored"), "", 401, "unauthorized");
        fixture.deny("config.get", "{}", Some(&node.bearer), "", 403, "ROLE_MISMATCH: node cannot invoke operator method: config.get");
        fixture.deny("config.set", "{}", Some(&admin.bearer), "", 403, "INSUFFICIENT_SCOPE: device token cannot invoke mutating method: config.set");
        fixture.deny("mcp.resolveLifecycleRequest", "{}", Some(&admin.bearer), "", 403,
            "MCP_LIFECYCLE_NATIVE_ONLY: Host MCP lifecycle receipts require the native control plane");
        fixture.deny("fetchLinkMetadata", "{}", Some(&admin.bearer), "", 403,
            "ATTACHMENTS_NATIVE_ONLY: Link preview requires the native control-plane gateway");
        assert_eq!(fixture.state(), before, "control requests leave credential, metadata, transcript and Memory state unchanged");
    }
}
