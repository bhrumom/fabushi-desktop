//! CWB-v1 oracles below the real credential loader and application request
//! builder. Only the destination of a built Codex request is redirected.
//! Synthetic private files and the loopback peer belong to an exact child;
//! these cases do not establish real service or concurrent-writer acceptance.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};

use super::{
    CodexResponsesRequestBinding, ProviderMessage, ProviderSessionError, RoutedProvider,
    RoutedProviderCheckpoint, RoutedProviderOptions, RoutedToolDefinition,
    capture_codex_responses_request_binding, run_routed_provider_text_with_lifecycle,
};
use crate::agents::agent_messaging::AgentMessageImage;
use crate::cursor_backend::SandPrivacyMode;
use crate::extensions::auth::auth_service::{HostAuthEnvironment, HostAuthServiceOptions};
use crate::extensions::auth::extension::start_host_auth_extension_with_options;
use crate::extensions::experiments::extension::{HostExperimentsExtension, HostExperimentsOptions};
use crate::extensions::inference::codex_direct_responses::CodexDirectCheckpoint;
use crate::extensions::inference::inference_service::{
    InferenceRoute, authorize_routed_provider_request,
};
use crate::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
};
use crate::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::settings::settings_service::SettingsService;
use crate::extensions::transcript::turn_runtime::shape_reply_nudge_turn_input;
use crate::host_request_context::HostRequestContext;
use crate::host_runner_composition::HostRunnerComposition;
use crate::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use crate::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryEvent, ProviderRetryReport,
    RoutedProviderAttemptControl, RoutedProviderAttemptExecutor, RoutedProviderCheckpointStore,
};
use crate::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedProviderRun,
    RoutedToolBridge, RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};
use crate::runner::sand_agent_runner::SandAgentRunner;
use crate::runner::sand_memory::{MEMORY_EPISODE_PREFIX, MEMORY_EXTRACTION_PROMPT_MARKER};
use crate::runner::subagent_runtime::SubagentRuntime;
use crate::runner::tools::sand_agent_management_tools::{AgentManagementRecord, AgentManagementSink};
use crate::runner::tools::sand_computer_tool::ComputerToolExposure;
use crate::runner::tools::send_message_tool::SendMessageSink;
use crate::runner::turn_memory::{
    ProductionEpisodeProgress, TurnExchange, TurnMemoryMode, run_turn_memory_with,
};
use crate::runner::{StreamAttemptPolicy, StreamAttemptRuntime, ToolEffectCustodyObservation, TurnRunOptions};
use crate::runner_production_bridge::{
    ProductionRunnerCompositionHooks, ProductionRunnerCompositionInput,
};

const RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const REFRESH_URL: &str = "https://auth.openai.com/oauth/token";
const MODEL: &str = "Codex/CWB/Stable@Model";
const ACCOUNT_A: &str = "cwb-synthetic-workspace-A";
const ACCOUNT_B: &str = "cwb-synthetic-workspace-B";
const CLIENT_ID: &str = "cwb-synthetic-oauth-client";
const AGENT_ID: &str = "codex-workspace-contract-agent";
const CHILD_TEST: &str = "CWB_CONTRACT_CHILD_TEST";
const CHILD_SCENARIO: &str = "CWB_CONTRACT_CHILD_SCENARIO";
const CHILD_ROOT: &str = "CWB_CONTRACT_CHILD_ROOT";
const CHILD_NONCE: &str = "CWB_CONTRACT_CHILD_NONCE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestKind {
    Responses,
    Refresh,
}

type RequestObserver = dyn Fn(RequestKind, &reqwest::Request) -> reqwest::Url;

struct HookState {
    owner: ThreadId,
    alive: Cell<bool>,
    observe: Box<RequestObserver>,
}

thread_local! {
    static REQUEST_HOOK: RefCell<Option<Rc<HookState>>> = RefCell::new(None);
}

/// A transport captures its armed scope, rather than looking up a later scope
/// at send time. Rc makes this token and its owning guard non-Send/non-Sync.
#[derive(Clone)]
pub(super) struct CapturedRequestHook(Rc<HookState>);

pub(super) fn capture_request_hook_for_current_thread() -> Option<CapturedRequestHook> {
    let captured = REQUEST_HOOK.with(|slot| slot.borrow().clone());
    captured.map(CapturedRequestHook)
}

impl CapturedRequestHook {
    pub(super) fn apply(&self, kind: RequestKind, request: &mut reqwest::Request) {
        assert_eq!(thread::current().id(), self.0.owner, "CWB request changed threads");
        assert!(self.0.alive.get(), "CWB request outlived its fixture scope");
        let current = REQUEST_HOOK.with(|slot| {
            slot.borrow().as_ref().is_some_and(|active| Rc::ptr_eq(active, &self.0))
        });
        assert!(current, "CWB captured request reached a different active scope");
        assert_eq!(request.method(), reqwest::Method::POST);
        let expected = match kind {
            RequestKind::Responses => RESPONSES_URL,
            RequestKind::Refresh => REFRESH_URL,
        };
        assert_eq!(request.url().as_str(), expected, "unexpected CWB request destination");
        // The callback has an immutable Request and no outstanding TLS borrow.
        // It can only propose a loopback URL; it cannot rebuild headers/body.
        let destination = (self.0.observe)(kind, request);
        assert_eq!(destination.scheme(), "http");
        assert_eq!(destination.host_str(), Some("127.0.0.1"));
        assert!(destination.port().is_some_and(|port| port != 0));
        assert!(destination.username().is_empty() && destination.password().is_none());
        assert!(destination.fragment().is_none());
        *request.url_mut() = destination;
    }
}

struct RequestHookScope {
    state: Rc<HookState>,
    previous: Option<Rc<HookState>>,
}

impl RequestHookScope {
    fn install(observe: impl Fn(RequestKind, &reqwest::Request) -> reqwest::Url + 'static) -> Self {
        let state = Rc::new(HookState {
            owner: thread::current().id(), alive: Cell::new(true), observe: Box::new(observe),
        });
        let previous = REQUEST_HOOK.with(|slot| slot.replace(Some(state.clone())));
        Self { state, previous }
    }
}

impl Drop for RequestHookScope {
    fn drop(&mut self) {
        assert_eq!(thread::current().id(), self.state.owner, "CWB scope changed threads");
        self.state.alive.set(false);
        REQUEST_HOOK.with(|slot| {
            let mut slot = slot.borrow_mut();
            assert!(slot.as_ref().is_some_and(|active| Rc::ptr_eq(active, &self.state)),
                "CWB scope retirement must follow its owned stack");
            *slot = self.previous.take();
        });
    }
}

struct FixtureRoot { path: PathBuf, clean: bool }

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        if self.clean { let _ = fs::remove_dir_all(&self.path); }
    }
}

struct OwnedChild { process: Child, terminal: bool, retirement: Option<String> }

impl OwnedChild {
    fn retire_once(&mut self) -> String {
        if let Some(observed) = &self.retirement { return observed.clone(); }
        self.retirement = Some("retirement started; terminal not observed".into());
        let kill = self.process.kill();
        let deadline = Instant::now() + Duration::from_secs(3);
        let observed = loop {
            let observation = match self.process.try_wait() {
                Ok(Some(status)) => { self.terminal = true; break format!("terminal={status:?}"); }
                Ok(None) => "child still running".to_string(),
                Err(error) => format!("terminal observation failed: {error}"),
            };
            if Instant::now() >= deadline { break format!("terminal=false; {observation}"); }
            thread::sleep(Duration::from_millis(10));
        };
        let result = format!("kill={kill:?}; {observed}");
        self.retirement = Some(result.clone());
        result
    }

    fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            match self.process.try_wait() {
                Ok(Some(status)) => { self.terminal = true; return Ok(status); }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                observed => {
                    let retired = self.retire_once();
                    return Err(format!("owned child did not complete: {observed:?}; {retired}"));
                }
            }
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.terminal && self.retirement.is_none() {
            let observation = self.retire_once();
            eprintln!("CWB child retirement: {observation}");
        }
    }
}

fn isolated(name: &str, scenario: &str, body: impl FnOnce(&Path)) {
    let (_, module) = module_path!().split_once("::").expect("library test module");
    let exact = format!("{module}::{name}");
    if let Ok(selected) = std::env::var(CHILD_TEST) {
        assert_eq!(selected, exact);
        if std::env::var(CHILD_SCENARIO).ok().as_deref() != Some(scenario) { return; }
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("owned child root"));
        body(&root);
        let receipt = json!({
            "test":exact,"scenario":scenario,"pid":std::process::id(),
            "nonce":std::env::var(CHILD_NONCE).expect("owned child nonce"),"completed":true,
        });
        OpenOptions::new().create_new(true).write(true).open(root.join("completed.json"))
            .expect("unique completion receipt").write_all(receipt.to_string().as_bytes()).unwrap();
        return;
    }
    let mut root = FixtureRoot {
        path: std::env::temp_dir().join(format!("cwb-{scenario}-{}", uuid::Uuid::new_v4())),
        clean: false,
    };
    let mut directory = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(&root.path).expect("private owned fixture directory");
    for name in ["codex", "home", "profile"] { fs::create_dir(root.path.join(name)).unwrap(); }
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.path.join("stdout.log");
    let stderr_path = root.path.join("stderr.log");
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", &exact, "--nocapture", "--test-threads=1"])
        .env(CHILD_TEST, &exact).env(CHILD_SCENARIO, scenario)
        .env(CHILD_ROOT, &root.path).env(CHILD_NONCE, &nonce)
        .env("CODEX_HOME", root.path.join("codex"))
        .env("HOME", root.path.join("home")).env("USERPROFILE", root.path.join("profile"))
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.path.join("absent-native.json"))
        .env("SAND_MEMORY_EPISODE_INTERVAL", "1")
        .env("NO_PROXY", "*").env("no_proxy", "*")
        .stdout(Stdio::from(File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(File::create(&stderr_path).unwrap()));
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy",
        "SAND_CODEX_MODEL", "SAND_CODEX_REASONING_EFFORT", "FABUSHI_PRODUCT_MODE", "SAND_PACKAGED",
        "SAND_AGENT_MODEL", "SAND_AGENT_MOCK_RESPONSE", "OPENAI_API_KEY", "CODEX_API_KEY"] {
        command.env_remove(key);
    }
    let mut child = OwnedChild {
        process: command.spawn().expect("spawn exact isolated test"), terminal: false, retirement: None,
    };
    let pid = child.process.id();
    let status = child.wait_bounded();
    let stdout = fs::read_to_string(&stdout_path).unwrap();
    let stderr = fs::read_to_string(&stderr_path).unwrap();
    assert!(matches!(&status, Ok(status) if status.success()),
        "CWB child failed: {status:?}; terminal={}; retained={}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        child.terminal, root.path.display());
    assert!(child.terminal, "root cleanup requires observed terminal exit");
    let bytes = fs::read(root.path.join("completed.json")).unwrap_or_else(|error| {
        panic!("missing completion receipt: {error}; retained={}\n{stdout}\n{stderr}", root.path.display())
    });
    let receipt: Value = serde_json::from_slice(&bytes).expect("completion receipt JSON");
    assert_eq!(receipt, json!({
        "test":exact,"scenario":scenario,"pid":pid,"nonce":nonce,"completed":true,
    }));
    root.clean = true;
}

#[derive(Clone)]
struct CredentialImage { access: String, refresh: String, id: String, workspace: String }

fn synthetic_jwt(workspace: Option<&str>, marker: &str) -> String {
    let mut payload = json!({"fixture_marker":marker});
    if let Some(workspace) = workspace {
        payload["https://api.openai.com/auth"] = json!({"chatgpt_account_id":workspace});
    }
    format!("synthetic-header.{}.synthetic-signature-{marker}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap()))
}

impl CredentialImage {
    fn new(workspace: &str, marker: &str) -> Self {
        Self {
            access: synthetic_jwt(Some(workspace), marker),
            refresh: format!("cwb-synthetic-refresh-{marker}"),
            id: format!("synthetic-header.{}.synthetic-signature",
                URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"aud":CLIENT_ID})).unwrap())),
            workspace: workspace.into(),
        }
    }
}

fn write_image(path: &Path, image: &CredentialImage) {
    let document = json!({"auth_mode":"chatgpt","tokens":{
        "access_token":image.access,"refresh_token":image.refresh,
        "id_token":image.id,"account_id":image.workspace,
    }});
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).expect("open owned private credential")
        .write_all(&serde_json::to_vec(&document).unwrap()).unwrap();
}

#[derive(Clone, Debug)]
struct ObservedRequest {
    kind: RequestKind,
    original_url: String,
    redirected_path: String,
    headers: BTreeMap<String, Vec<String>>,
    body: Vec<u8>,
}

impl ObservedRequest {
    fn json(&self) -> Value { serde_json::from_slice(&self.body).expect("actual JSON request bytes") }
}

struct ExpectedRequest { kind: RequestKind, image: CredentialImage }

impl ExpectedRequest {
    fn responses(image: &CredentialImage) -> Self { Self { kind: RequestKind::Responses, image: image.clone() } }
    fn refresh(image: &CredentialImage) -> Self { Self { kind: RequestKind::Refresh, image: image.clone() } }
}

#[derive(Debug)]
struct WireRequest { method: String, target: String, headers: BTreeMap<String, Vec<String>>, body: Vec<u8> }

struct Reply {
    kind: RequestKind,
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
    before_response: Option<Box<dyn FnOnce(&WireRequest) + Send>>,
    cancelled_peer: bool,
}

impl Reply {
    fn events(events: Vec<Value>) -> Self {
        let body = events.into_iter().map(|event| format!("data: {event}\n\n")).collect::<String>();
        Self { kind: RequestKind::Responses, status: 200, content_type: "text/event-stream",
            body: body.into_bytes(), before_response: None, cancelled_peer: false }
    }

    fn terminal(text: &str) -> Self {
        Self::events(vec![json!({"type":"response.output_text.delta","delta":text}),
            json!({"type":"response.completed","response":{"id":"cwb-terminal","output":[]}})])
    }

    fn status(status: u16) -> Self {
        Self { kind: RequestKind::Responses, status, content_type: "application/json",
            body: b"{}".to_vec(), before_response: None, cancelled_peer: false }
    }

    fn refresh(payload: Value) -> Self {
        Self { kind: RequestKind::Refresh, status: 200, content_type: "application/json",
            body: serde_json::to_vec(&payload).unwrap(), before_response: None, cancelled_peer: false }
    }

    fn before(mut self, action: impl FnOnce(&WireRequest) + Send + 'static) -> Self {
        self.before_response = Some(Box::new(action));
        self
    }

    fn cancelling(mut self, action: impl FnOnce(&WireRequest) + Send + 'static) -> Self {
        self.before_response = Some(Box::new(action));
        self.cancelled_peer = true;
        self
    }
}

fn read_wire_request(stream: &mut TcpStream) -> Result<WireRequest, String> {
    stream.set_read_timeout(Some(Duration::from_millis(100))).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut header_end = None;
    let mut content_length = None;
    loop {
        if let (Some(end), Some(length)) = (header_end, content_length) {
            if bytes.len() >= end + length { break; }
        }
        if Instant::now() >= deadline { return Err("owned request read deadline exceeded".into()); }
        match stream.read(&mut buffer) {
            Ok(0) => return Err("owned request closed before its complete body".into()),
            Ok(count) => bytes.extend_from_slice(&buffer[..count]),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => continue,
            Err(error) => return Err(format!("owned request read failed: {error}")),
        }
        if bytes.len() > 2 * 1024 * 1024 { return Err("owned request exceeded capture bound".into()); }
        if header_end.is_none() {
            if let Some(position) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                let end = position + 4;
                let head = std::str::from_utf8(&bytes[..position]).map_err(|_| "invalid HTTP header bytes")?;
                let lengths = head.lines().skip(1).filter_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length").then_some(value.trim())
                }).collect::<Vec<_>>();
                if lengths.len() != 1 { return Err("one actual Content-Length required".into()); }
                let length: usize = lengths[0].parse().map_err(|_| "invalid Content-Length")?;
                if length > 1024 * 1024 { return Err("owned request body exceeded capture bound".into()); }
                header_end = Some(end);
                content_length = Some(length);
            } else if bytes.len() > 64 * 1024 { return Err("owned headers exceeded capture bound".into()); }
        }
    }
    let end = header_end.ok_or("missing HTTP header terminator")?;
    let length = content_length.ok_or("missing actual body length")?;
    if bytes.len() != end + length { return Err("unexpected pipelined/trailing request bytes".into()); }
    let head = std::str::from_utf8(&bytes[..end - 4]).map_err(|_| "invalid HTTP header bytes")?;
    let mut lines = head.lines();
    let first = lines.next().ok_or("missing request line")?.split_whitespace().collect::<Vec<_>>();
    if first.len() != 3 { return Err("invalid request line".into()); }
    let mut headers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in lines {
        let (name, value) = line.split_once(':').ok_or("invalid header line")?;
        headers.entry(name.to_ascii_lowercase()).or_default().push(value.trim().to_string());
    }
    Ok(WireRequest { method: first[0].into(), target: first[1].into(), headers,
        body: bytes[end..].to_vec() })
}

struct OwnedPeer {
    address: SocketAddr,
    nonce: String,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<Vec<WireRequest>, String>>>,
    terminal: bool,
    retirement_spent: bool,
}

impl OwnedPeer {
    fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("owned literal loopback listener");
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let nonce = uuid::Uuid::new_v4().to_string();
        let peer_nonce = nonce.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(45);
            let mut replies: VecDeque<_> = replies.into();
            let mut observed = Vec::new();
            loop {
                if worker_stop.load(Ordering::Acquire) {
                    return if replies.is_empty() { Ok(observed) } else { Err("scripted requests were not reached".into()) };
                }
                if Instant::now() >= deadline { return Err("owned peer lifetime exhausted".into()); }
                match listener.accept() {
                    Ok((mut stream, remote)) => {
                        if !remote.ip().is_loopback() { return Err("foreign peer reached owned fixture".into()); }
                        let request = read_wire_request(&mut stream)?;
                        let mut reply = replies.pop_front().ok_or("unexpected extra loopback request")?;
                        let suffix = match reply.kind { RequestKind::Responses => "responses", RequestKind::Refresh => "refresh" };
                        let expected = format!("/{peer_nonce}/{}/{suffix}", observed.len());
                        if request.target != expected || request.method != "POST" { return Err("unexpected owned request route".into()); }
                        if let Some(action) = reply.before_response.take() { action(&request); }
                        observed.push(request);
                        stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|e| e.to_string())?;
                        let reason = match reply.status { 200 => "OK", 401 => "Unauthorized", 503 => "Service Unavailable", _ => "Fixture" };
                        let head = format!("HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            reply.status, reason, reply.content_type, reply.body.len());
                        if let Err(error) = stream.write_all(head.as_bytes())
                            .and_then(|()| stream.write_all(&reply.body))
                        {
                            // Only an explicitly scripted cancellation may close
                            // this peer after its complete request was observed.
                            // Every ordinary response retains strict write proof.
                            if !reply.cancelled_peer || !matches!(error.kind(),
                                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted)
                            {
                                return Err(format!("owned response write failed: {error}"));
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(5)),
                    Err(error) => return Err(format!("owned accept failed: {error}")),
                }
            }
        });
        Self { address, nonce, stop, worker: Some(worker), terminal: false, retirement_spent: false }
    }

    fn retire(&mut self) -> Result<Vec<WireRequest>, String> {
        if self.retirement_spent { return Err("owned peer retirement budget already spent".into()); }
        self.retirement_spent = true;
        self.stop.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_secs(3);
        while !self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            if Instant::now() >= deadline { return Err("owned peer terminal state not observed".into()); }
            thread::sleep(Duration::from_millis(5));
        }
        let result = self.worker.take().expect("owned worker").join();
        self.terminal = true;
        result.map_err(|_| "owned peer panicked".to_string())?
    }

    fn finish(mut self) -> Vec<WireRequest> {
        let result = self.retire();
        assert!(self.terminal, "fixture completion requires actual peer retirement");
        result.expect("complete loopback request script")
    }
}

impl Drop for OwnedPeer {
    fn drop(&mut self) {
        if !self.terminal && !self.retirement_spent {
            let result = self.retire();
            if !thread::panicking() { panic!("owned peer was not explicitly finished: {result:?}"); }
        }
    }
}

struct RequestRecording {
    expected: RefCell<VecDeque<ExpectedRequest>>,
    observed: RefCell<Vec<ObservedRequest>>,
}

struct HttpFixture { peer: OwnedPeer, scope: RequestHookScope, recording: Rc<RequestRecording> }

impl HttpFixture {
    fn new(expected: Vec<ExpectedRequest>, replies: Vec<Reply>) -> Self {
        assert_eq!(expected.len(), replies.len());
        let peer = OwnedPeer::start(replies);
        let address = peer.address;
        let nonce = peer.nonce.clone();
        let recording = Rc::new(RequestRecording { expected: RefCell::new(expected.into()), observed: RefCell::new(Vec::new()) });
        let record = recording.clone();
        let scope = RequestHookScope::install(move |kind, request| {
            REQUEST_HOOK.with(|slot| { let _borrow_released_before_callback = slot.borrow_mut(); });
            let expected = record.expected.borrow_mut().pop_front().expect("unexpected built application request");
            assert_eq!(kind, expected.kind);
            let headers = request.headers().keys().map(|name| {
                (name.as_str().to_ascii_lowercase(), request.headers().get_all(name).iter()
                    .map(|value| value.to_str().expect("synthetic header text").to_string()).collect::<Vec<_>>())
            }).collect::<BTreeMap<_, _>>();
            let body = request.body().and_then(reqwest::Body::as_bytes).expect("existing reusable request body").to_vec();
            match kind {
                RequestKind::Responses => {
                    assert_eq!(headers.get("authorization"), Some(&vec![format!("Bearer {}", expected.image.access)]));
                    assert_eq!(headers.get("chatgpt-account-id"), Some(&vec![expected.image.workspace]));
                    assert_eq!(headers.get("content-type"), Some(&vec!["application/json".to_string()]));
                    assert_eq!(headers.get("accept"), Some(&vec!["text/event-stream".to_string()]));
                    assert_eq!(headers.get("user-agent"), Some(&vec!["fabushi-router/1".to_string()]));
                    let value: Value = serde_json::from_slice(&body).unwrap();
                    assert_eq!(value["model"], MODEL);
                    assert_eq!(value["stream"], true);
                    assert_eq!(value["store"], false);
                }
                RequestKind::Refresh => {
                    assert!(!headers.contains_key("authorization") && !headers.contains_key("chatgpt-account-id"));
                    assert_eq!(headers.get("content-type"), Some(&vec!["application/x-www-form-urlencoded".to_string()]));
                    let form = url::form_urlencoded::parse(&body).map(|(key, value)| (key.into_owned(), value.into_owned())).collect::<Vec<_>>();
                    assert_eq!(form, vec![("grant_type".into(), "refresh_token".into()),
                        ("refresh_token".into(), expected.image.refresh), ("client_id".into(), CLIENT_ID.into())]);
                }
            }
            let suffix = match kind { RequestKind::Responses => "responses", RequestKind::Refresh => "refresh" };
            let path = format!("/{nonce}/{}/{suffix}", record.observed.borrow().len());
            record.observed.borrow_mut().push(ObservedRequest {
                kind, original_url: request.url().to_string(), redirected_path: path.clone(), headers, body,
            });
            reqwest::Url::parse(&format!("http://{address}{path}")).unwrap()
        });
        Self { peer, scope, recording }
    }

    fn count(&self) -> usize { self.recording.observed.borrow().len() }

    fn finish(self) -> Vec<ObservedRequest> {
        let Self { peer, scope, recording } = self;
        assert!(recording.expected.borrow().is_empty(), "all expected production request boundaries must execute");
        drop(scope);
        let wire = peer.finish();
        let observed = recording.observed.borrow().clone();
        assert_eq!(wire.len(), observed.len());
        for (actual, original) in wire.iter().zip(&observed) {
            assert_eq!(actual.method, "POST");
            assert_eq!(actual.target, original.redirected_path);
            assert_eq!(actual.body, original.body, "destination change must preserve actual body bytes");
            for (name, values) in &original.headers {
                assert_eq!(actual.headers.get(name), Some(values), "actual application header changed: {name}");
            }
            assert_eq!(original.original_url, match original.kind { RequestKind::Responses => RESPONSES_URL, RequestKind::Refresh => REFRESH_URL });
        }
        observed
    }
}

#[derive(Default)]
struct EmptyBridge { lists: AtomicUsize, calls: AtomicUsize }

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.lists.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }
    fn call_tool(&self, _: &RoutedToolDefinition, _: Value, _: &str) -> Result<Value, ProviderSessionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        panic!("unexpected delegated tool")
    }
}

struct UnusedAgentManagement;
impl AgentManagementSink for UnusedAgentManagement {
    fn self_agent_id(&self) -> &str { AGENT_ID }
    fn send_to_agent(&self, _: &str, _: &str, _: &[AgentMessageImage], _: bool) -> Result<String, ProviderSessionError> { panic!("unexpected agent message") }
    fn create_agent(&self, _: &str, _: &str) -> Result<AgentManagementRecord, ProviderSessionError> { panic!("unexpected agent creation") }
    fn update_agent(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Result<Option<AgentManagementRecord>, ProviderSessionError> { panic!("unexpected agent update") }
}

#[derive(Default)]
struct CountedDelivery(Mutex<Vec<(String, Value)>>);
impl SendMessageSink for CountedDelivery {
    fn send_message(&self, value: Value, timestamp_ms: u64, call_id: &str) -> Result<Option<String>, ProviderSessionError> {
        assert!(timestamp_ms > 0);
        let mut delivered = self.0.lock().unwrap();
        assert!(delivered.is_empty(), "accepted effect must not execute twice");
        delivered.push((call_id.into(), value));
        Ok(Some("cwb-delivery-1".into()))
    }
}

#[derive(Default)]
struct RetryObservations { events: Mutex<Vec<ProviderRetryEvent>>, reports: Mutex<Vec<ProviderRetryReport>> }

struct Owners {
    inference: ProductionInferenceExtension,
    host: HostRunnerComposition,
    delivery: Arc<CountedDelivery>,
    retries: Arc<RetryObservations>,
}

fn codex_inference_owners(root: &Path) -> (Arc<SettingsService>, ProductionInferenceExtension) {
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    let before = settings.get_host_settings_snapshot();
    settings.set_host_settings_if_current(Some(&before.hash), json!({
        "inferenceProvider":"codex",
        "agentDefaultModel":{"provider":"codex","modelId":MODEL,"maxMode":true,"parameters":[]},
    })).unwrap();
    let auth = start_host_auth_extension_with_options(HostAuthServiceOptions {
        environment: HostAuthEnvironment::from_pairs([("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE".into(),
            root.join("absent-native.json").to_string_lossy().into_owned())]),
        backend_url: Some("http://127.0.0.1:9".into()), backend: None,
        machine_id: Arc::new(|| Ok("cwb-fixture-device".into())), log: Arc::new(|_| {}), now_ms: Arc::new(|| 0),
    }, Arc::new(|_| Ok(None))).unwrap();
    let inference = start_production_inference_extension(Arc::new(auth),
        Arc::new(HostExperimentsExtension::new(HostExperimentsOptions { is_dev_build: false, env_gate_overrides: None })),
        settings.clone());
    assert_eq!(inference.route(), InferenceRoute::Routed(RoutedProvider::Codex));
    (settings, inference)
}

impl Owners {
    fn new(root: &Path) -> Self {
        let (settings, inference) = codex_inference_owners(root);
        Self { inference, host: HostRunnerComposition::with_sink(
            Arc::new(SandLocalToolPermissionController::production(settings)), Arc::new(|_| {})),
            delivery: Arc::new(CountedDelivery::default()), retries: Arc::new(RetryObservations::default()) }
    }

    fn admit(&self) -> CodexResponsesRequestBinding {
        authorize_routed_provider_request(self.inference.route(), RoutedProvider::Codex).unwrap();
        assert_eq!(self.inference.port().resolve_codex_turn_model(None, None, None).unwrap(), MODEL);
        capture_codex_responses_request_binding().expect("actual private credential admission")
    }

    fn runner(&self, root: &Path, stream: &str, binding: CodexResponsesRequestBinding,
        store: Arc<ProductionRoutedProviderCheckpointStore>) -> SandAgentRunner {
        let events = self.retries.clone();
        let reports = self.retries.clone();
        let composition = self.host.compose_production_turn(ProductionRunnerCompositionInput {
            provider: RoutedProvider::Codex, fabushi_model: None, codex_model: Some(MODEL.into()),
            codex_request_binding: Some(binding), fabushi_request_binding: None,
            bridge: Arc::new(EmptyBridge::default()), cursor_auth: None, request_context: request_context(root),
            cancellation: Default::default(), checkpoint_store: store,
            retry_sink: Some(Arc::new(move |event| events.events.lock().unwrap().push(event.clone()))),
            retry_report_sink: Some(Arc::new(move |report| reports.reports.lock().unwrap().push(report.clone()))),
            usage_sink: None, box_resources: None, browser_executor: None, web_executor: None,
            generate_image_executor: None, computer_executor: None, computer_exposure: ComputerToolExposure::Full,
            file_transfer_executor: None, external_machine_executor: None, external_shell_review: None,
            mcp_management_sink: None, send_message_sink: Some(self.delivery.clone()), reaction_sink: None,
            cloud_agent_tool: None, multitask_enabled: false, spotlight_enabled: false,
            toolset_role: Default::default(), local_tool_permission: None, action_audit: None, observation: None,
        }, ProductionRunnerCompositionHooks {
            agent_management_sink: Arc::new(UnusedAgentManagement), state_writer: None,
            routine_auto_review: Arc::new(|_, _| Ok(None)), state_approval_barrier: Arc::new(|| Ok(())),
            box_shell_review: Arc::new(|_| Ok(None)), subagent_task_sink: None, subagent_task_allowed_types: None,
            subagent_task_review: None, subagent_management_runtime: None, subagent_steer_review: None,
            routine_post_write: None, multitask_todo_state: None,
        });
        self.host.compose_production_runner(composition, ProductionTurnAgentBuildBindings::new(
            ProductionTurnAgentStaticConfig { model_id: MODEL.into(), agent_token_limit: 200_000,
                conversation_id: AGENT_ID.into(), is_box_scoped_subagent: false, is_subagent_runner: false,
                is_shared_room_runner: false, sand_send_message_delivery_owed: true, transcripts_folder_available: true },
            Arc::new(|| Some(SandPrivacyMode::NoTraining)),
            Arc::new(|_, _, _| panic!("bounded turn does not need compaction"))),
            ProductionTurnAgentLifecycleBindings::new(AGENT_ID, stream), None, Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(SubagentRuntime::default())), None, None, None, None)
    }
}

#[path = "codex_synthesis_identity_tests.rs"]
mod synthesis_identity_tests;

fn request_context(root: &Path) -> RunnerRequestContextSnapshot {
    RunnerRequestContextSnapshot { context: HostRequestContext {
        os_version: "cwb-contract".into(), shell: None, time_zone: Some("UTC".into()),
        transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(), user_full_name: None,
    }, rules: None }
}

fn messages(label: &str) -> Vec<ProviderMessage> {
    vec![ProviderMessage { role: "user".into(), content: format!("cwb request {label}") }]
}

fn checkpoint_files(store: &ProductionRoutedProviderCheckpointStore) -> Vec<(PathBuf, Vec<u8>)> {
    let entries = match fs::read_dir(store.directory()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => panic!("checkpoint read failed: {error}"),
    };
    let mut files = entries.map(|entry| { let path = entry.unwrap().path(); let bytes = fs::read(&path).unwrap(); (path, bytes) }).collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn custody(runner: &SandAgentRunner, effects: u64) {
    let Some(ToolEffectCustodyObservation::Known { run_generation, snapshot }) = runner.last_tool_effect_custody() else {
        panic!("actual generated invocation custody required")
    };
    assert!(*run_generation > 0);
    assert_eq!(snapshot.started_revision, effects);
    assert_eq!(snapshot.checkpoint_revision, effects);
    assert_eq!(snapshot.pending_effects, 0);
    assert!(snapshot.unknown_result.is_none());
}

fn direct_runner(root: &Path, model: &str, binding: Option<CodexResponsesRequestBinding>,
    bridge: Arc<EmptyBridge>, cancellation: RoutedProviderCancellation,
    store: Arc<ProductionRoutedProviderCheckpointStore>, retries: Arc<RetryObservations>,
) -> Result<String, ProviderSessionError> {
    let reports = retries.clone();
    run_routed_provider_in_runner(RoutedProviderRun {
        provider: RoutedProvider::Codex, fabushi_model: None, codex_model: Some(model.into()),
        codex_request_binding: binding, fabushi_request_binding: None, data_dir: root,
        messages: &messages("direct"), typed_replay: None, bridge, cursor_auth: None,
        request_context: request_context(root), cancellation, checkpoint_store: store,
        retry_sink: Some(Arc::new(move |event| retries.events.lock().unwrap().push(event.clone()))),
        retry_report_sink: Some(Arc::new(move |report| reports.reports.lock().unwrap().push(report.clone()))),
        stream_attempt_runtime: None, usage_sink: None, cloud_agents_enabled: false, multitask_enabled: false,
        is_computer_use_subagent: false, max_steps: 8, tool_step_reminder: None,
    }, &mut |_, _| {})
}

#[derive(Default)]
struct AdapterEffects { tools: Cell<usize>, checkpoints: Cell<usize>, deltas: RefCell<Vec<(String, String)>> }

impl AdapterEffects {
    fn assert_zero(&self) {
        assert_eq!(self.tools.get(), 0); assert_eq!(self.checkpoints.get(), 0); assert!(self.deltas.borrow().is_empty());
    }
}

fn adapter(root: &Path, model: &str, binding: Option<&CodexResponsesRequestBinding>,
    resume: Option<&RoutedProviderCheckpoint>, cancelled: bool, effects: &AdapterEffects,
) -> Result<String, ProviderSessionError> {
    let mut tool = |_: &RoutedToolDefinition, _: Value, _: &str| {
        effects.tools.set(effects.tools.get() + 1);
        Err(ProviderSessionError::Tool("unexpected oracle tool".into()))
    };
    let mut delta = |part: &str, total: &str| effects.deltas.borrow_mut().push((part.into(), total.into()));
    let mut options = RoutedProviderOptions {
        data_dir: root, typed_replay: None, fabushi_model: None, codex_model: Some(model),
        codex_request_binding: binding, fabushi_request_binding: None, cursor_auth: None, tools: &[],
        mcp_server_url: None, execute_tool: &mut tool, on_text_delta: &mut delta,
        on_partial_tool_call: None, should_cancel: &|| cancelled,
    };
    run_routed_provider_text_with_lifecycle(RoutedProvider::Codex, &messages("adapter"), &mut options, resume,
        &mut |_| { effects.checkpoints.set(effects.checkpoints.get() + 1); Ok(()) })
}

fn known_checkpoint(model: Option<&str>) -> CodexDirectCheckpoint {
    CodexDirectCheckpoint { model: model.map(str::to_string), input: vec![
        json!({"role":"user","content":"previous workspace A history"}),
        json!({"type":"reasoning","encrypted_content":"cwb-accepted-reasoning"}),
        json!({"type":"function_call","name":"AlreadyApplied","call_id":"already-applied","arguments":"{}"}),
        json!({"type":"function_call_output","call_id":"already-applied","output":"{\"applied\":true}"}),
    ], text: "prior ".into(), response_id: "cwb-prior-response".into(), usage: Default::default(), completed_steps: 1, tool_calls_completed: 1 }
}

fn assert_redacted(error: &ProviderSessionError, images: &[&CredentialImage]) {
    let message = format!("{error:?} {error}");
    for image in images {
        for secret in [&image.access, &image.refresh, &image.id] {
            assert!(!message.contains(secret.as_str()), "credential material leaked into error");
        }
    }
}

#[test]
fn codex_workspace_real_http_retry_refuses_replaced_account_and_new_turn_binds_current() {
    isolated("codex_workspace_real_http_retry_refuses_replaced_account_and_new_turn_binds_current", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "first-A");
        let b = CredentialImage::new(ACCOUNT_B, "later-B");
        let auth_path = root.join("codex/auth.json");
        write_image(&auth_path, &a);
        let owners = Owners::new(root);
        let admitted_a = owners.admit();
        let replace_path = auth_path.clone();
        let replacement = b.clone();
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::responses(&b)], vec![
            Reply::status(503).before(move |_| write_image(&replace_path, &replacement)),
            Reply::terminal("fresh B accepted"),
        ]);
        let store_a = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "old-A"));
        let mut runner_a = owners.runner(root, "old-A", admitted_a, store_a.clone());
        let original = messages("old A turn");
        let mut deltas = Vec::new();
        let error = runner_a.run_routed_provider(root, &original,
            &mut |part, total| deltas.push((part.to_string(), total.to_string())))
            .expect_err("B must not receive A's retried request");
        assert!(matches!(error, ProviderSessionError::Authentication(_)));
        assert_redacted(&error, &[&a, &b]);
        assert!(deltas.is_empty());
        assert_eq!(http.count(), 1, "the retry must refuse before request two");
        custody(&runner_a, 0);
        assert!(checkpoint_files(&store_a).is_empty());
        let retries = owners.retries.events.lock().unwrap().clone();
        assert_eq!(retries.len(), 1);
        assert_eq!((retries[0].attempt, retries[0].next_attempt), (1, 2));
        assert!(!retries[0].resume_from_checkpoint && !retries[0].watchdog_expired);
        assert_eq!(owners.retries.reports.lock().unwrap().last().unwrap().error_type, "AuthenticationError");

        // A new generated invocation on the same composed worker must retain
        // A too. A nudge is not a fresh login/account admission.
        let nudge = shape_reply_nudge_turn_input(&original, &original, &TurnRunOptions::default());
        let error = runner_a.run_routed_provider_with_projected_messages(root,
            &nudge.lifecycle_messages, &nudge.provider_messages, nudge.options, &mut |_, _| {})
            .expect_err("same-worker nudge keeps the original binding");
        assert!(matches!(error, ProviderSessionError::Authentication(_)));
        assert_eq!(http.count(), 1);
        assert_eq!(owners.retries.events.lock().unwrap().len(), 1, "permanent account refusal is not retried");
        custody(&runner_a, 0);
        assert!(checkpoint_files(&store_a).is_empty());

        let store_b = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "fresh-B"));
        let mut runner_b = owners.runner(root, "fresh-B", owners.admit(), store_b.clone());
        assert_eq!(runner_b.run_routed_provider(root, &messages("fresh B turn"), &mut |_, _| {}).unwrap(), "fresh B accepted");
        custody(&runner_b, 0);
        assert!(checkpoint_files(&store_b).is_empty());
        assert!(owners.delivery.0.lock().unwrap().is_empty());
        let requests = http.finish();
        assert_eq!(requests.len(), 2);
        assert_ne!(requests[0].json()["input"], requests[1].json()["input"]);
        assert!(requests[1].json()["input"].to_string().contains("fresh B turn"));
    });
}

#[test]
fn codex_workspace_real_http_retry_accepts_same_workspace_rotation() {
    isolated("codex_workspace_real_http_retry_accepts_same_workspace_rotation", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "rotation-before");
        let rotated = CredentialImage::new(ACCOUNT_A, "rotation-after");
        let path = root.join("codex/auth.json");
        write_image(&path, &a);
        let changed_path = path.clone();
        let changed_image = rotated.clone();
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::responses(&rotated)], vec![
            Reply::status(503).before(move |_| write_image(&changed_path, &changed_image)),
            Reply::terminal("rotated A accepted"),
        ]);
        let bridge = Arc::new(EmptyBridge::default());
        let retries = Arc::new(RetryObservations::default());
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "rotation"));
        // No App binding is supplied: the actual top-level Runner must create
        // one empty identity, initialize it once and retain it across attempts.
        assert_eq!(direct_runner(root, MODEL, None, bridge.clone(), Default::default(),
            store.clone(), retries.clone()).unwrap(), "rotated A accepted");
        assert_eq!(bridge.lists.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert!(checkpoint_files(&store).is_empty());
        let events = retries.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!((events[0].attempt, events[0].next_attempt), (1, 2));
        assert!(!events[0].resume_from_checkpoint && !events[0].watchdog_expired);
        let requests = http.finish();
        assert_eq!(requests[0].body, requests[1].body, "rotation changes credentials, not the retried model or history");
        let persisted: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(persisted["tokens"]["access_token"], rotated.access);
        assert_eq!(persisted["tokens"]["account_id"], ACCOUNT_A);
    });
}

#[test]
fn codex_workspace_real_http_checkpoint_refusal_preserves_history_and_effect_custody() {
    isolated("codex_workspace_real_http_checkpoint_refusal_preserves_history_and_effect_custody", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "checkpoint-A");
        let b = CredentialImage::new(ACCOUNT_B, "checkpoint-B");
        let path = root.join("codex/auth.json");
        write_image(&path, &a);
        let owners = Owners::new(root);
        let binding = owners.admit();
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "checkpoint"));
        let before_retry = Arc::new(Mutex::new(None));
        let capture = before_retry.clone();
        let observed_store = store.clone();
        let changed_path = path.clone();
        let replacement = b.clone();
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::responses(&a)], vec![
            Reply::events(vec![
                json!({"type":"response.output_text.delta","delta":"working "}),
                json!({"type":"response.completed","response":{"id":"cwb-tool-boundary","output":[
                    {"type":"reasoning","id":"cwb-reasoning","encrypted_content":"cwb-opaque-reasoning","summary":[]},
                    {"type":"function_call","name":"SendMessage","call_id":"cwb-send-once",
                        "arguments":json!({"type":"text","content":"one workspace A effect"}).to_string()}
                ]}}),
            ]),
            Reply::status(503).before(move |wire| {
                let files = checkpoint_files(&observed_store);
                assert_eq!(files.len(), 1, "real accepted tool checkpoint precedes continuation HTTP");
                let checkpoint: RoutedProviderCheckpoint = serde_json::from_slice(&files[0].1).unwrap();
                let RoutedProviderCheckpoint::Codex(checkpoint) = checkpoint else { panic!("wrong checkpoint provider") };
                assert_eq!(checkpoint.model.as_deref(), Some(MODEL));
                assert_eq!((checkpoint.completed_steps, checkpoint.tool_calls_completed), (1, 1));
                assert_eq!(checkpoint.text, "working ");
                let body: Value = serde_json::from_slice(&wire.body).unwrap();
                assert_eq!(body["input"], json!(checkpoint.input));
                *capture.lock().unwrap() = Some(files);
                write_image(&changed_path, &replacement);
            }),
        ]);
        let mut runner = owners.runner(root, "checkpoint", binding, store.clone());
        let mut deltas = Vec::new();
        let error = runner.run_routed_provider(root, &messages("accepted tool checkpoint"),
            &mut |part, total| deltas.push((part.to_string(), total.to_string())))
            .expect_err("checkpoint continuation must refuse replacement workspace B");
        assert!(matches!(error, ProviderSessionError::Authentication(_)));
        assert_redacted(&error, &[&a, &b]);
        assert_eq!(deltas, vec![("working ".into(), "working ".into())]);
        custody(&runner, 1);
        let files = checkpoint_files(&store);
        assert_eq!(files, before_retry.lock().unwrap().as_ref().unwrap().clone(),
            "permanent identity refusal cannot replace accepted checkpoint bytes/history");
        assert_eq!(owners.delivery.0.lock().unwrap().as_slice(), &[(
            "cwb-send-once".to_string(), json!({"type":"text","content":"one workspace A effect"}),
        )]);
        let events = owners.retries.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].resume_from_checkpoint);
        assert_eq!((events[0].attempt, events[0].next_attempt), (1, 2));
        assert!(!events[0].watchdog_expired);
        assert_eq!(owners.retries.reports.lock().unwrap().last().unwrap().error_type, "AuthenticationError");
        let requests = http.finish();
        assert_eq!(requests.len(), 2, "no B inference after the accepted checkpoint");
        let persisted: RoutedProviderCheckpoint = serde_json::from_slice(&files[0].1).unwrap();
        let RoutedProviderCheckpoint::Codex(checkpoint) = persisted else { panic!("wrong checkpoint provider") };
        assert_eq!(json!(checkpoint.input), requests[1].json()["input"]);
        assert_eq!(checkpoint.input.iter().filter(|item| item["type"] == "reasoning").count(), 1);
        assert_eq!(checkpoint.input.iter().filter(|item| item["type"] == "function_call").count(), 1);
        let outputs = checkpoint.input.iter().filter(|item| item["type"] == "function_call_output").collect::<Vec<_>>();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0]["call_id"], "cwb-send-once");
        let receipt: Value = serde_json::from_str(outputs[0]["output"].as_str().unwrap()).unwrap();
        assert_eq!(receipt["sent"], true);
        assert_eq!(receipt["messageId"], "cwb-delivery-1");
        let encoded = String::from_utf8(files[0].1.clone()).unwrap();
        for forbidden in [&a.access, &a.refresh, &a.id, &b.access, &a.workspace, &b.workspace] {
            assert!(!encoded.contains(forbidden.as_str()), "binding and credential image are not checkpoint state");
        }
    });
}

#[test]
fn codex_workspace_real_http_refresh_same_workspace_persists_and_resends() {
    isolated("codex_workspace_real_http_refresh_same_workspace_persists_and_resends", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "refresh-before");
        let mut rotated = a.clone();
        rotated.access = synthetic_jwt(Some(ACCOUNT_A), "refresh-after");
        let path = root.join("codex/auth.json");
        write_image(&path, &a);
        let before = fs::read(&path).unwrap();
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::refresh(&a),
            ExpectedRequest::responses(&rotated)], vec![Reply::status(401),
            Reply::refresh(json!({"access_token":rotated.access})), Reply::terminal("refreshed A accepted")]);
        let retries = Arc::new(RetryObservations::default());
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "refresh"));
        let bridge = Arc::new(EmptyBridge::default());
        assert_eq!(direct_runner(root, MODEL, None, bridge.clone(), Default::default(),
            store.clone(), retries.clone()).unwrap(), "refreshed A accepted");
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert!(retries.events.lock().unwrap().is_empty(), "one-401 refresh is the existing transport's owner");
        assert!(retries.reports.lock().unwrap().is_empty());
        assert!(checkpoint_files(&store).is_empty());
        let after = fs::read(&path).unwrap();
        assert_ne!(after, before);
        let document: Value = serde_json::from_slice(&after).unwrap();
        assert_eq!(document["tokens"]["account_id"], ACCOUNT_A);
        assert_eq!(document["tokens"]["access_token"], rotated.access);
        assert_eq!(document["tokens"]["refresh_token"], a.refresh, "existing omitted-token fallback remains");
        assert_eq!(document["tokens"]["id_token"], a.id);
        let requests = http.finish();
        assert_eq!(requests.iter().map(|request| request.kind).collect::<Vec<_>>(),
            vec![RequestKind::Responses, RequestKind::Refresh, RequestKind::Responses]);
        assert_eq!(requests[0].body, requests[2].body, "refresh resends the exact same request body");
    });
}

#[test]
fn codex_workspace_real_http_refresh_refuses_missing_or_changed_workspace_without_write() {
    let name = "codex_workspace_real_http_refresh_refuses_missing_or_changed_workspace_without_write";
    for (scenario, workspace) in [("different", Some(ACCOUNT_B)), ("missing", None)] {
        isolated(name, scenario, |root| {
            let a = CredentialImage::new(ACCOUNT_A, "refused-refresh-A");
            let path = root.join("codex/auth.json");
            write_image(&path, &a);
            let before = fs::read(&path).unwrap();
            let returned = synthetic_jwt(workspace, "refused-refresh-response");
            let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::refresh(&a)],
                vec![Reply::status(401), Reply::refresh(json!({"access_token":returned,
                    "refresh_token":"cwb-refused-new-refresh","id_token":"cwb-refused-new-id"}))]);
            let retries = Arc::new(RetryObservations::default());
            let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "refused-refresh"));
            let error = direct_runner(root, MODEL, None, Arc::new(EmptyBridge::default()),
                Default::default(), store.clone(), retries.clone()).expect_err("invalid refreshed workspace");
            assert!(matches!(error, ProviderSessionError::Authentication(_)));
            assert_redacted(&error, &[&a]);
            assert!(!error.to_string().contains(returned.as_str()));
            assert_eq!(fs::read(&path).unwrap(), before, "invalid refresh cannot publish any new credential image");
            assert_eq!(fs::read_dir(root.join("codex")).unwrap().count(), 1, "refusal creates no temporary write");
            assert!(checkpoint_files(&store).is_empty());
            assert!(retries.events.lock().unwrap().is_empty(), "Authentication is permanently ineligible");
            let reports = retries.reports.lock().unwrap();
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].error_type, "AuthenticationError");
            assert!(!reports[0].watchdog_expired);
            let requests = http.finish();
            assert_eq!(requests.len(), 2, "one original request and refresh, with no resend");
        });
    }
}

#[test]
fn codex_workspace_real_http_refusals_precede_credentials_transport_and_tools() {
    isolated("codex_workspace_real_http_refusals_precede_credentials_transport_and_tools", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "bound-legacy");
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a)], vec![Reply::terminal("legacy done")]);
        let effects = AdapterEffects::default();
        let known = RoutedProviderCheckpoint::Codex(known_checkpoint(Some(MODEL)));
        let legacy_inner = known_checkpoint(None);
        let legacy = RoutedProviderCheckpoint::Codex(legacy_inner.clone());
        let wrong_provider = RoutedProviderCheckpoint::Fabushi(known_checkpoint(Some(MODEL)));
        let uninitialized = CodexResponsesRequestBinding::for_runner_invocation();
        let store = ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "guard-history");
        store.persist(&legacy).unwrap();
        let before = checkpoint_files(&store);
        assert!(matches!(adapter(root, "bad\nmodel", None, Some(&wrong_provider), true, &effects),
            Err(ProviderSessionError::Cancelled(_))));
        assert!(matches!(adapter(root, "Other/Model", None, Some(&known), false, &effects),
            Err(ProviderSessionError::Protocol(_))));
        assert!(matches!(adapter(root, MODEL, None, Some(&wrong_provider), false, &effects),
            Err(ProviderSessionError::Protocol(_))));
        assert!(matches!(adapter(root, MODEL, None, Some(&known), false, &effects),
            Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(adapter(root, MODEL, Some(&uninitialized), Some(&known), false, &effects),
            Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(adapter(root, MODEL, None, Some(&legacy), false, &effects),
            Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(adapter(root, "bad\nmodel", None, None, false, &effects),
            Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(super::run_codex_provider_text_with_lifecycle(
            &messages("unbound convenience wrapper"), &[],
            &mut |_, _, _| panic!("unbound wrapper cannot execute tools"),
            &mut |_, _| panic!("unbound wrapper cannot emit text"), &|| false,
            Some(&legacy_inner), &mut |_| panic!("unbound wrapper cannot accept checkpoints"),
        ), Err(ProviderSessionError::Configuration(_))));
        effects.assert_zero();
        assert_eq!(http.count(), 0);
        assert!(!root.join("codex/auth.json").exists(), "typed refusal won over deliberately missing credentials");
        assert_eq!(checkpoint_files(&store), before);

        let cancellation = RoutedProviderCancellation::default();
        assert!(cancellation.cancel("cwb-owned-cancellation"));
        let bridge = Arc::new(EmptyBridge::default());
        let retries = Arc::new(RetryObservations::default());
        let cancelled_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "cancelled"));
        let error = direct_runner(root, "bad\nmodel", None, bridge.clone(), cancellation,
            cancelled_store.clone(), retries.clone()).expect_err("already cancelled actual Runner");
        assert!(matches!(error, ProviderSessionError::Transport(ref reason) if reason == "cwb-owned-cancellation"));
        assert_eq!(bridge.lists.load(Ordering::SeqCst), 0);
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert!(retries.events.lock().unwrap().is_empty() && retries.reports.lock().unwrap().is_empty());
        assert!(checkpoint_files(&cancelled_store).is_empty());
        assert_eq!(http.count(), 0);

        write_image(&root.join("codex/auth.json"), &a);
        let binding = capture_codex_responses_request_binding().unwrap();
        assert_eq!(adapter(root, MODEL, Some(&binding), Some(&legacy), false, &effects).unwrap(), "prior legacy done");
        assert_eq!(effects.tools.get(), 0);
        assert_eq!(effects.checkpoints.get(), 0);
        assert_eq!(effects.deltas.borrow().as_slice(), &[("legacy done".into(), "prior legacy done".into())]);
        assert_eq!(checkpoint_files(&store), before, "bound compatibility does not rewrite legacy evidence");
        let requests = http.finish();
        assert_eq!(requests[0].json()["input"], json!(legacy_inner.input));
    });
}

#[test]
fn codex_workspace_real_http_memory_keeps_bound_workspace_and_summary_captures_current() {
    isolated("codex_workspace_real_http_memory_keeps_bound_workspace_and_summary_captures_current", "run", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "memory-A");
        let b = CredentialImage::new(ACCOUNT_B, "summary-B");
        let path = root.join("codex/auth.json");
        write_image(&path, &a);
        let owners = Owners::new(root);
        let binding = owners.admit();
        let http = HttpFixture::new(vec![ExpectedRequest::responses(&a), ExpectedRequest::responses(&a),
            ExpectedRequest::responses(&b)], vec![
            Reply::terminal("profile: The completed workspace A turn stays private"),
            Reply::terminal("Workspace A completed its admitted memory episode."),
            Reply::terminal("independent workspace B summary"),
        ]);
        assert!(matches!(owners.inference.run_turn_memory_prompt(root, RoutedProvider::Codex,
            None, Some(MODEL), None, None, "memory", "missing binding", &|| false),
            Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(owners.inference.run_turn_memory_prompt(root, RoutedProvider::Codex,
            None, None, None, None, "memory", "cancel first", &|| true),
            Err(ProviderSessionError::Cancelled(_))));
        assert_eq!(http.count(), 0);
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("memory-agents"), 500));
        let session = workers.materialize_session_with_active(None, "user", None, None).unwrap();
        let agent_id = session.record.id.clone();
        let progress = ProductionEpisodeProgress::new(workers.clone(), agent_id.clone());
        let mut calls = 0;
        let report = run_turn_memory_with(&session.memory, Some(&progress), 1_900_000_000_000,
            TurnExchange { user: "remember workspace A's completed turn".into(), agent: "complete".into() },
            TurnMemoryMode::Extract, |system, user| {
                calls += 1;
                owners.inference.run_turn_memory_prompt(root, RoutedProvider::Codex, None, Some(MODEL),
                    Some(&binding), None, system, user, &|| false).map_err(|error| error.to_string())
            });
        assert_eq!(calls, 2);
        assert!(report.extraction_attempted && report.episode_recorded && report.episode_summary_attempted);
        assert_eq!(report.added_memories, 1);
        assert!(report.episode_memory_added);
        assert!(workers.get_agent_pending_episode_turns(&agent_id).unwrap().is_empty());
        let memory = session.memory.list_memories(100);
        assert!(memory.iter().any(|row| row.content == "The completed workspace A turn stays private"));
        assert!(memory.iter().any(|row| row.content == format!("{MEMORY_EPISODE_PREFIX}Workspace A completed its admitted memory episode.")));
        assert_eq!(http.count(), 2);

        write_image(&path, &b);
        let b_bytes = fs::read(&path).unwrap();
        let error = owners.inference.run_turn_memory_prompt(root, RoutedProvider::Codex,
            None, Some(MODEL), Some(&binding), None, "memory", "old A private continuation", &|| false)
            .expect_err("admitted turn-memory must not silently capture B");
        assert!(matches!(error, ProviderSessionError::Authentication(_)));
        assert_redacted(&error, &[&a, &b]);
        assert_eq!(http.count(), 2, "B receives no request for the old admitted memory call");
        assert_eq!(fs::read(&path).unwrap(), b_bytes);
        assert_eq!(owners.inference.run_summarization_prompt(root, "independent summary",
            "new authorized workspace B material", &|| false).unwrap(), "independent workspace B summary");
        let requests = http.finish();
        assert_eq!(requests.len(), 3);
        assert!(requests[0].json()["instructions"].as_str().unwrap().contains(MEMORY_EXTRACTION_PROMPT_MARKER));
        for request in &requests { assert!(request.json().get("tools").is_none()); }
        assert!(requests[2].json()["input"].to_string().contains("new authorized workspace B material"));
        drop(progress);
        drop(session);
        workers.shutdown();
        drop(workers);
        let reopened = ProductionSessionWorkers::with_agents_root(root.join("memory-agents"), 500);
        let session = reopened.open_materialized_session(&agent_id).unwrap().unwrap();
        assert!(session.memory.list_memories(100).iter().any(|row| row.content == "The completed workspace A turn stays private"));
        assert!(reopened.get_agent_pending_episode_turns(&agent_id).unwrap().is_empty());
        drop(session);
        reopened.shutdown();
    });
}

#[test]
fn codex_workspace_request_hook_restores_scope_and_rejects_stale_or_unexpected_targets() {
    isolated("codex_workspace_request_hook_restores_scope_and_rejects_stale_or_unexpected_targets", "run", |_| {
        assert!(capture_request_hook_for_current_thread().is_none());
        let peer = OwnedPeer::start(Vec::new());
        let address = peer.address;
        let calls = Rc::new(Cell::new(0));
        let outer_calls = calls.clone();
        let outer = RequestHookScope::install(move |_, request| {
            REQUEST_HOOK.with(|slot| { let _released_before_callback = slot.borrow_mut(); });
            assert!(request.body().and_then(reqwest::Body::as_bytes).is_some());
            outer_calls.set(outer_calls.get() + 1);
            reqwest::Url::parse(&format!("http://{address}/scope-only")).unwrap()
        });
        let captured = capture_request_hook_for_current_thread().unwrap();
        let client = super::responses_http_client().unwrap();
        let request = || client.post(RESPONSES_URL).header("authorization", "Bearer cwb-synthetic-probe")
            .header("ChatGPT-Account-Id", ACCOUNT_A).json(&json!({"probe":"scope-only"})).build().unwrap();
        let mut first = request();
        let before_headers = first.headers().clone();
        let before_body = first.body().unwrap().as_bytes().unwrap().to_vec();
        captured.clone().apply(RequestKind::Responses, &mut first);
        assert_eq!(first.headers(), &before_headers);
        assert_eq!(first.body().unwrap().as_bytes().unwrap(), before_body.as_slice());
        assert_eq!(calls.get(), 1);
        let sibling = thread::spawn(|| capture_request_hook_for_current_thread().is_none());
        let deadline = Instant::now() + Duration::from_secs(2);
        while !sibling.is_finished() && Instant::now() < deadline { thread::sleep(Duration::from_millis(5)); }
        assert!(sibling.is_finished(), "scope-isolation observer must terminate");
        assert!(sibling.join().unwrap());
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _inner = RequestHookScope::install(move |_, _| reqwest::Url::parse(&format!("http://{address}/inner")).unwrap());
            let wrong_scope = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| captured.apply(RequestKind::Responses, &mut request())));
            assert!(wrong_scope.is_err());
            let inner = capture_request_hook_for_current_thread().unwrap();
            inner.apply(RequestKind::Responses, &mut request());
            panic!("controlled CWB scope unwind");
        }));
        assert!(unwind.is_err());
        captured.apply(RequestKind::Responses, &mut request());
        assert_eq!(calls.get(), 2, "unwind restored the original scope");
        let wrong_kind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| captured.apply(RequestKind::Refresh, &mut request())));
        assert!(wrong_kind.is_err());
        let mut unexpected = request();
        *unexpected.url_mut() = reqwest::Url::parse("https://unexpected.invalid/cwb").unwrap();
        let wrong_target = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| captured.apply(RequestKind::Responses, &mut unexpected)));
        assert!(wrong_target.is_err());
        assert_eq!(calls.get(), 2);
        drop(outer);
        assert!(capture_request_hook_for_current_thread().is_none());
        let stale = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| captured.apply(RequestKind::Responses, &mut request())));
        assert!(stale.is_err());
        assert_eq!(calls.get(), 2);
        assert!(peer.finish().is_empty(), "scope probes never execute HTTP");
    });
}

#[test]
fn codex_workspace_built_request_errors_keep_cancellation_precedence() {
    isolated("codex_workspace_built_request_errors_keep_cancellation_precedence", "run", |_| {
        let http = HttpFixture::new(Vec::new(), Vec::new());
        let captured = capture_request_hook_for_current_thread().unwrap();
        let client = super::responses_http_client().unwrap();
        for cancelled in [true, false] {
            // Exercise the actual build_split/execute future, not a fixture
            // version of it. The invalid header remains a reqwest builder error.
            let builder = client.post(RESPONSES_URL).header("authorization", "synthetic\ninvalid-header").json(&json!({"probe":true}));
            let result = super::provider_io_runtime().unwrap().block_on(super::await_reqwest(
                super::execute_responses_http_request(builder, Some(&captured), RequestKind::Responses),
                &|| cancelled, "CWB builder-error probe",
            ));
            if cancelled { assert!(matches!(result, Err(ProviderSessionError::Cancelled(_)))); }
            else { assert!(matches!(result, Err(ProviderSessionError::Transport(_)))); }
            assert_eq!(http.count(), 0, "a cancelled or failed build cannot enter the request hook");
        }
        assert!(http.finish().is_empty(), "no builder-error probe reached any peer");
    });
}

struct CredentialStopExecutor {
    root: PathBuf,
    binding: CodexResponsesRequestBinding,
    cancellation: RoutedProviderCancellation,
    cancel_after_load: bool,
    attempts: usize,
    preparations: Cell<usize>,
}

impl RoutedProviderAttemptExecutor for CredentialStopExecutor {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        attempt.check()?;
        self.attempts += 1;
        assert!(resume_from.is_none());
        let prepared = Cell::new(false);
        // Deliver the real shell stop at the exact completed-load boundary.
        // This neither counts callback polls nor guesses when a sleep expires.
        let deliver_stop = || prepared.get() && should_cancel();
        let after_load = || {
            prepared.set(true);
            self.preparations.set(self.preparations.get() + 1);
            assert!(!attempt.is_cancelled());
            let deadline = Instant::now() + Duration::from_secs(3);
            while !should_cancel() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(1));
            }
            assert!(should_cancel(), "observe the actual shell watchdog, not elapsed fixture time");
            assert!(!attempt.is_cancelled(), "watchdog-only control is not user cancellation");
            if self.cancel_after_load {
                assert!(self.cancellation.cancel("cwb-user-cancel-after-real-credential-load"));
                assert!(attempt.is_cancelled());
            }
        };
        let mut tool = |_: &RoutedToolDefinition, _: Value, _: &str| {
            panic!("a refused credential image cannot execute tools")
        };
        let mut options = RoutedProviderOptions {
            data_dir: &self.root, typed_replay: None, fabushi_model: None, codex_model: Some(MODEL),
            codex_request_binding: Some(&self.binding), fabushi_request_binding: None,
            cursor_auth: None, tools: &[], mcp_server_url: None, execute_tool: &mut tool,
            on_text_delta, on_partial_tool_call: None, should_cancel: &deliver_stop,
        };
        let result = super::run_routed_provider_text_with_lifecycle_impl(
            RoutedProvider::Codex, &messages("actual watchdog credential refusal"), &mut options,
            resume_from, on_checkpoint, &mut |_| panic!("no credential-refused usage"),
            8, None, true, Some(&after_load),
        );
        assert!(prepared.get(), "the actual load result must reach its observer");
        result
    }
}

#[test]
fn codex_workspace_actual_watchdog_preserves_credential_refusal_and_user_cancellation() {
    let name = "codex_workspace_actual_watchdog_preserves_credential_refusal_and_user_cancellation";
    for cancel_after_load in [false, true] {
        let scenario = if cancel_after_load { "user-cancel" } else { "watchdog-only" };
        isolated(name, scenario, |root| {
            let a = CredentialImage::new(ACCOUNT_A, "watchdog-admitted-a");
            let b = CredentialImage::new(ACCOUNT_B, "watchdog-replacement-b");
            let path = root.join("codex/auth.json");
            write_image(&path, &a);
            let binding = capture_codex_responses_request_binding().unwrap();
            write_image(&path, &b);
            let before = fs::read(&path).unwrap();
            let http = HttpFixture::new(Vec::new(), Vec::new());
            let adapter = ProductionTurnRunShellAdapter {
                policy: StreamAttemptPolicy {
                    first_output_timeout: Duration::from_millis(10), max_attempts: 2,
                    initial_backoff: Duration::from_millis(1), max_backoff: Duration::from_millis(1),
                    max_retry_after: Duration::from_millis(1),
                },
                watchdog_poll_interval: Duration::from_millis(1),
            };
            let runtime = Arc::new(StreamAttemptRuntime::new(adapter.policy.clone()));
            let cancellation = RoutedProviderCancellation::default();
            let mut executor = CredentialStopExecutor {
                root: root.to_path_buf(), binding, cancellation: cancellation.clone(), cancel_after_load,
                attempts: 0, preparations: Cell::new(0),
            };
            let store = ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, scenario);
            let mut retries = Vec::new();
            let mut reports = Vec::new();
            let error = adapter.run_with_attempt_runtime_reporting(
                runtime.clone(), &cancellation, &store, &mut executor,
                &mut |_, _| panic!("no credential-refused text"),
                &mut |event| retries.push(event.clone()), &mut |report| reports.push(report.clone()),
            ).expect_err("identity refusal and actual cancellation are both terminal");
            assert_eq!(executor.attempts, 1, "a permanent credential refusal cannot become a watchdog retry");
            assert_eq!(executor.preparations.get(), 1);
            assert!(retries.is_empty());
            if cancel_after_load {
                assert!(matches!(&error, ProviderSessionError::Cancelled(reason)
                    if reason == "cwb-user-cancel-after-real-credential-load"));
                assert!(reports.is_empty(), "actual cancellation settles before failure classification");
            } else {
                assert!(matches!(&error, ProviderSessionError::Authentication(_)));
                assert_eq!(reports.len(), 1);
                assert_eq!(reports[0].error_type, "AuthenticationError");
                assert!(reports[0].watchdog_expired, "the owner must report actual expiry");
                assert!(!cancellation.is_cancelled());
            }
            assert_redacted(&error, &[&a, &b]);
            let custody = runtime.effect_custody();
            assert_eq!(custody.started_revision, 0);
            assert_eq!(custody.checkpoint_revision, 0);
            assert_eq!(custody.pending_effects, 0);
            assert!(custody.unknown_result.is_none());
            assert!(checkpoint_files(&store).is_empty());
            assert_eq!(fs::read(&path).unwrap(), before);
            assert!(http.finish().is_empty(), "the refused load must never dispatch HTTP");
        });
    }
    isolated(name, "ordinary-call-cancellation", |root| {
        let a = CredentialImage::new(ACCOUNT_A, "ordinary-admitted-a");
        let b = CredentialImage::new(ACCOUNT_B, "ordinary-replacement-b");
        let path = root.join("codex/auth.json");
        write_image(&path, &a);
        let binding = capture_codex_responses_request_binding().unwrap();
        let http = HttpFixture::new(Vec::new(), Vec::new());
        for image in [&b, &a] {
            write_image(&path, image);
            let before = fs::read(&path).unwrap();
            let cancelled = Cell::new(false);
            let should_cancel = || cancelled.get();
            let after_load = || { assert!(!cancelled.replace(true)); };
            let mut tool = |_: &RoutedToolDefinition, _: Value, _: &str| panic!("no cancelled tool");
            let mut delta = |_: &str, _: &str| panic!("no cancelled text");
            let mut options = RoutedProviderOptions {
                data_dir: root, typed_replay: None, fabushi_model: None, codex_model: Some(MODEL),
                codex_request_binding: Some(&binding), fabushi_request_binding: None,
                cursor_auth: None, tools: &[], mcp_server_url: None, execute_tool: &mut tool,
                on_text_delta: &mut delta, on_partial_tool_call: None, should_cancel: &should_cancel,
            };
            let result = super::run_routed_provider_text_with_lifecycle_impl(
                RoutedProvider::Codex, &messages("ordinary cancellation after load"), &mut options,
                None, &mut |_| panic!("no cancelled checkpoint"), &mut |_| panic!("no cancelled usage"),
                8, None, false, Some(&after_load),
            );
            assert!(cancelled.get());
            assert!(matches!(result, Err(ProviderSessionError::Cancelled(_))),
                "ordinary calls keep cancellation priority for either successful or refused loading");
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        assert!(http.finish().is_empty());
    });
}

#[test]
fn codex_workspace_admitted_summary_keeps_identity_across_compaction_phases() {
    let name = "codex_workspace_admitted_summary_keeps_identity_across_compaction_phases";
    for scenario in ["same-workspace-rotation", "replacement-before-primary", "replacement-before-prefix"] {
        isolated(name, scenario, |root| {
            let a = CredentialImage::new(ACCOUNT_A, "summary-admitted-A");
            let rotated = CredentialImage::new(ACCOUNT_A, "summary-rotated-A");
            let b = CredentialImage::new(ACCOUNT_B, "summary-replacement-B");
            let path = root.join("codex/auth.json");
            write_image(&path, &a);
            let owners = Owners::new(root);
            let admitted_model = owners.inference.port().resolve_codex_turn_model(None, None, None).unwrap();
            assert_eq!(admitted_model, MODEL);
            let binding = owners.admit();
            let settings = SettingsService::new(root.join("settings.json"));
            let previous = settings.get_host_settings_snapshot();
            let later_model = "Codex/CWB/Later@Summary-Model";
            settings.set_host_settings_if_current(Some(&previous.hash), json!({
                "agentDefaultModel":{"provider":"codex","modelId":later_model,"maxMode":true,"parameters":[]},
            })).unwrap();
            assert_eq!(owners.inference.port().resolve_codex_turn_model(None, None, None).unwrap(), later_model);
            let (expected, replies) = match scenario {
                "same-workspace-rotation" => (
                    vec![ExpectedRequest::responses(&a), ExpectedRequest::responses(&rotated)],
                    vec![Reply::terminal("admitted primary summary"), Reply::terminal("admitted prefix summary")],
                ),
                "replacement-before-primary" => (Vec::new(), Vec::new()),
                "replacement-before-prefix" => (
                    vec![ExpectedRequest::responses(&a)], vec![Reply::terminal("admitted primary summary")],
                ),
                _ => unreachable!(),
            };
            let http = HttpFixture::new(expected, replies);

            // Model/binding admission and explicit cancellation are earlier
            // boundaries than a conflicting credential image. These calls
            // must not capture the current B identity or fall back to defaults.
            write_image(&path, &b);
            let replacement_bytes = fs::read(&path).unwrap();
            for (model, request_binding) in [
                (None, Some(&binding)), (Some(admitted_model.as_str()), None), (None, None),
            ] {
                let result = owners.inference.run_turn_summarization_prompt(
                    root, RoutedProvider::Codex, None, model, request_binding, None,
                    "CWB missing summary identity", "must not dispatch", &|| false,
                );
                assert!(matches!(result, Err(ProviderSessionError::Configuration(_))));
                assert_eq!(http.count(), 0);
                assert_eq!(fs::read(&path).unwrap(), replacement_bytes);
            }
            for (model, request_binding) in [
                (None, None), (Some("bad\nmodel"), Some(&binding)),
                (Some(admitted_model.as_str()), Some(&binding)),
            ] {
                let result = owners.inference.run_turn_summarization_prompt(
                    root, RoutedProvider::Codex, None, model, request_binding, None,
                    "CWB cancelled summary", "must not dispatch", &|| true,
                );
                assert!(matches!(result, Err(ProviderSessionError::Cancelled(_))));
                assert_eq!(http.count(), 0);
                assert_eq!(fs::read(&path).unwrap(), replacement_bytes);
            }

            // Both phases consume the same admitted values through the real
            // production method. The App compaction producer is covered by
            // its separate oracle; this is the real HTTP consumer boundary.
            let summarize = |system: &str, user: &str| {
                owners.inference.run_turn_summarization_prompt(
                    root, RoutedProvider::Codex, None, Some(&admitted_model), Some(&binding), None,
                    system, user, &|| false,
                )
            };
            if scenario == "replacement-before-primary" {
                let error = summarize("CWB admitted primary phase", "workspace A private primary history")
                    .expect_err("primary must refuse replacement B before HTTP");
                assert!(matches!(error, ProviderSessionError::Authentication(_)));
                assert_redacted(&error, &[&a, &rotated, &b]);
                assert_eq!(http.count(), 0);
                assert_eq!(fs::read(&path).unwrap(), replacement_bytes);
            } else {
                write_image(&path, &a);
                let primary_bytes = fs::read(&path).unwrap();
                assert_eq!(summarize("CWB admitted primary phase", "workspace A private primary history")
                    .unwrap(), "admitted primary summary");
                assert_eq!(http.count(), 1, "the primary phase must execute one real A request");
                assert_eq!(fs::read(&path).unwrap(), primary_bytes);

                write_image(&path, if scenario == "same-workspace-rotation" { &rotated } else { &b });
                let prefix_bytes = fs::read(&path).unwrap();
                let prefix = summarize("CWB admitted prefix phase", "workspace A private split prefix history");
                if scenario == "same-workspace-rotation" {
                    assert_eq!(prefix.unwrap(), "admitted prefix summary");
                    assert_eq!(http.count(), 2);
                } else {
                    let error = prefix.expect_err("prefix must refuse B after the real A primary completed");
                    assert!(matches!(error, ProviderSessionError::Authentication(_)));
                    assert_redacted(&error, &[&a, &rotated, &b]);
                    assert_eq!(http.count(), 1, "the refused prefix cannot send A history under B");
                }
                assert_eq!(fs::read(&path).unwrap(), prefix_bytes);
            }
            assert_eq!(fs::read_dir(root.join("codex")).unwrap().count(), 1,
                "summary calls create no credential replacement or temporary file");
            assert!(owners.delivery.0.lock().unwrap().is_empty());
            assert!(owners.retries.events.lock().unwrap().is_empty());
            assert!(owners.retries.reports.lock().unwrap().is_empty());
            let requests = http.finish();
            let expected_count = match scenario {
                "same-workspace-rotation" => 2, "replacement-before-primary" => 0, _ => 1,
            };
            assert_eq!(requests.len(), expected_count);
            assert_eq!(settings.get_agent_default_model_checked().unwrap(), Some(json!({
                "provider":"codex","modelId":later_model,"maxMode":true,"parameters":[],
            })));
            assert_eq!(settings.get_inference_provider(), "codex");
            let usage = settings.get_inference_router_usage();
            assert_eq!(usage["providers"]["codex"]["requests"], json!(expected_count),
                "only successful actual summaries retain the existing usage accounting");
            for (index, request) in requests.iter().enumerate() {
                let body = request.json();
                assert_eq!(body["model"], admitted_model);
                assert!(body.get("tools").is_none(), "admitted summaries expose no tools");
                let (phase, history) = if index == 0 {
                    ("CWB admitted primary phase", "workspace A private primary history")
                } else {
                    ("CWB admitted prefix phase", "workspace A private split prefix history")
                };
                assert!(body["instructions"].as_str().unwrap().contains(phase));
                assert!(!body["instructions"].as_str().unwrap().contains(later_model));
                assert!(body["input"].as_array().unwrap().iter().any(|item| {
                    item["role"] == "user" && item["content"] == history
                }));
            }
        });
    }
}
