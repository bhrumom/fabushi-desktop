//! MSI-v1 through the actual production Memory factory, Native HTTP and Session store.
//! Synthetic loopback credentials establish this boundary, not real-service compatibility.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::{ProductionSynthesisTarget, production_memory_synthesis_options};
use crate::extensions::auth::auth_service::{HostAuthEnvironment, HostAuthServiceOptions};
use crate::extensions::auth::extension::start_host_auth_extension_with_options;
use crate::extensions::experiments::extension::{HostExperimentsExtension, HostExperimentsOptions};
use crate::extensions::inference::inference_service::InferenceRoute;
use crate::extensions::inference::production::start_production_inference_extension;
use crate::extensions::inference::provider_session::RoutedProvider;
use crate::extensions::memory::memory_service::{
    MemoryKind, MemoryOrigin, MemorySynthesisBridge, SynthesisApplyResult, SynthesisChange,
    memory_id_for,
};
use crate::extensions::memory::memory_synthesis_service::{
    MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS, MEMORY_SYNTHESIS_PROMPT_MARKER,
    MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER, MemorySynthesisService, SynthesisOutcome,
    SynthesisReport, SynthesisReportOutcome, SynthesisTarget,
};
use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::settings::settings_service::SettingsService;

const NOW: i64 = 1_700_000_000_000;
const MODEL_A: &str = "Native/Memory/CaseSensitive-A";
const MODEL_B: &str = "Native/Memory/CaseSensitive-B";
const TOKEN_A: &str = "msi-private-native-token-A";
const TOKEN_ROTATED: &str = "msi-private-native-token-A-rotated";
const TOKEN_B: &str = "msi-private-native-token-B";
const SOURCE: &str = "MSI historical synthesized fact";
const UPDATED: &str = "MSI reviewed synthesized fact";
const EXPLICIT: &str = "MSI explicit fact must survive";
const RETIRED: &str = "MSI forgotten fact must stay forgotten";
const EVIDENCE: &str = "MSI current evidence sentinel";
const REFRESH: &str = ".dreaming/next-refresh-at";
const CHILD_TEST: &str = "FABUSHI_MSI_NATIVE_CHILD_TEST";
const CHILD_ROOT: &str = "FABUSHI_MSI_NATIVE_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_MSI_NATIVE_CHILD_NONCE";

#[derive(Clone, Copy)]
enum Case {
    Rotate,
    Replace,
    TemporalReplace,
    RetryRotate,
    RetryReplace,
    CancelBeforeApply,
}

impl Case {
    fn refuses(self) -> bool {
        matches!(self, Self::Replace | Self::TemporalReplace | Self::RetryReplace)
    }

    fn retries(self) -> bool {
        matches!(self, Self::RetryRotate | Self::RetryReplace)
    }

    fn first_request_count(self) -> usize {
        if self.refuses() { 1 } else if self.retries() { 3 } else { 2 }
    }
}

fn bound_model(model: &str) -> Value {
    json!({"provider":"fabushi","modelId":model,"maxMode":true,"parameters":[]})
}

fn private_write(path: &Path, bytes: &[u8]) {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).expect("create owned private file");
    file.write_all(bytes).expect("write owned private file");
    file.sync_all().expect("sync owned private file");
}

fn credential(path: &Path, origin: &str, token: &str, context: &str) {
    let next = path.with_extension(format!("{}.next", uuid::Uuid::new_v4()));
    let image = json!({
        "version":1,"issuer":origin,"accessToken":token,"expiresAtMs":4_102_444_800_000_u64,
        "sessionId":format!("msi-session-{context}"),
        "deviceId":format!("msi-device-{context}"),"userId":format!("msi-user-{context}"),
    });
    private_write(&next, image.to_string().as_bytes());
    fs::rename(next, path).expect("publish fixture-owned credential image");
}

struct OwnedRoot {
    path: PathBuf,
    clean: bool,
}

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        if self.clean { let _ = fs::remove_dir_all(&self.path); }
    }
}

struct OwnedChild {
    process: Child,
    terminal: bool,
    retirement: Option<String>,
}

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
            eprintln!("MSI child retirement: {observation}");
        }
    }
}

struct Captured {
    model: String,
    authorization: String,
    payload: Value,
    verify: bool,
}

fn read_request(stream: &mut TcpStream) -> Result<Captured, String> {
    stream.set_nonblocking(false).map_err(|_| "blocking accepted socket")?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|_| "read timeout")?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|_| "write timeout")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut bytes = Vec::new();
    loop {
        if Instant::now() >= deadline { return Err("request deadline".into()); }
        let mut chunk = [0; 8192];
        let count = stream.read(&mut chunk).map_err(|_| "read request")?;
        if count == 0 { return Err("incomplete request".into()); }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() > 1024 * 1024 { return Err("request size limit".into()); }
        let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else { continue; };
        let headers = std::str::from_utf8(&bytes[..end]).map_err(|_| "header encoding")?;
        if headers.lines().next() != Some("POST /v1/responses HTTP/1.1") {
            return Err("unexpected method/path".into());
        }
        let mut length = None;
        let mut authorization = None;
        for header in headers.lines().skip(1) {
            let (name, value) = header.split_once(':').ok_or("malformed header")?;
            if name.eq_ignore_ascii_case("content-length") {
                if length.replace(value.trim().parse::<usize>().map_err(|_| "content length")?).is_some() {
                    return Err("duplicate content length".into());
                }
            } else if name.eq_ignore_ascii_case("authorization") {
                if authorization.replace(value.trim().to_owned()).is_some() {
                    return Err("duplicate authorization".into());
                }
            } else if name.eq_ignore_ascii_case("chatgpt-account-id") {
                return Err("Native request carried a Codex account header".into());
            }
        }
        let length = length.ok_or("missing content length")?;
        if length > 1024 * 1024 { return Err("body size limit".into()); }
        if bytes.len() < end + 4 + length { continue; }
        let body: Value = serde_json::from_slice(&bytes[end + 4..end + 4 + length])
            .map_err(|_| "body JSON")?;
        let input = body["input"].as_array().ok_or("request input")?;
        let user = input.iter().rev().find(|item| item["role"] == "user")
            .and_then(|item| item["content"].as_str()).ok_or("summary user message")?;
        let payload: Value = serde_json::from_str(user).map_err(|_| "Memory JSON payload")?;
        let verify = payload.get("proposedChanges").is_some();
        let marker = if verify { MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER } else { MEMORY_SYNTHESIS_PROMPT_MARKER };
        if !body["instructions"].as_str().is_some_and(|text| text.contains(marker)) {
            return Err("missing real Memory system prompt".into());
        }
        if body.get("tools").is_some_and(|tools| tools.as_array().is_none_or(|tools| !tools.is_empty())) {
            return Err("summary exposed tools".into());
        }
        return Ok(Captured {
            model: body["model"].as_str().ok_or("request model")?.into(),
            authorization: authorization.ok_or("missing authorization")?,
            payload,
            verify,
        });
    }
}

fn proposal(payload: &Value) -> Result<Value, String> {
    let memory = payload["currentMemories"].as_array().ok_or("historical memories")?
        .iter().find(|memory| memory["content"] == SOURCE || memory["content"] == UPDATED)
        .ok_or("missing historical fact sentinel")?;
    let evidence = payload["newEvidence"].as_array().ok_or("proposal evidence")?;
    let evidence_id = if evidence.is_empty() { "clock" } else {
        evidence[0]["id"].as_str().ok_or("evidence id")?
    };
    Ok(json!({"changes":[{
        "action":"update","id":memory["id"],"content":UPDATED,"kind":"profile",
        "sourceEvidenceIds":[evidence_id],
    }]}))
}

fn response(stream: &mut TcpStream, text: &str, retry: bool) -> Result<(), String> {
    let (status, content_type, body) = if retry {
        ("503 Service Unavailable", "application/json", json!({"error":"fixture retry"}).to_string())
    } else {
        let events = [
            json!({"type":"response.output_text.delta","delta":text}),
            json!({"type":"response.completed","response":{
                "id":"msi-native-response","usage":{"input_tokens":11,"output_tokens":2},"output":[],
            }}),
        ];
        ("200 OK", "text/event-stream", events.iter()
            .map(|event| format!("data: {event}\n\n")).collect::<String>())
    };
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|_| "write response".into())
}

struct Server {
    endpoint: String,
    requests: Arc<Mutex<Vec<Captured>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl Server {
    fn new(root: &Path, case: Case) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("owned loopback listener");
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        credential(&root.join("credential.json"), &origin, TOKEN_A, "A");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let owned_root = root.to_path_buf();
        let endpoint = format!("{origin}/v1/responses");
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(60);
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= deadline { return Err("server deadline".into()); }
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(_) => return Err("accept request".into()),
                };
                let request = read_request(&mut stream)?;
                let index = captured.lock().unwrap().len();
                let output = if request.verify {
                    let changes = request.payload["proposedChanges"].as_array().ok_or("verification changes")?;
                    if changes.len() != 1 || changes[0]["content"] != UPDATED {
                        return Err("verification lacks the proposal sentinel".into());
                    }
                    json!({"approved":true})
                } else {
                    proposal(&request.payload)?
                };
                captured.lock().unwrap().push(request);
                if index == 0 {
                    // The first actual proposal is observed before either mutation.
                    // Returning its response is the deterministic between-phase barrier.
                    SettingsService::new(owned_root.join("settings.json"))
                        .set_agent_default_model(Some(&bound_model(MODEL_B)))
                        .map_err(|_| "persist model B at response barrier")?;
                    let (token, context) = if case.refuses() { (TOKEN_B, "B") } else { (TOKEN_ROTATED, "A") };
                    credential(&owned_root.join("credential.json"), &origin, token, context);
                }
                response(&mut stream, &output.to_string(), index == 0 && case.retries())?;
            }
            Ok(())
        });
        Self { endpoint, requests, stop, worker: Some(worker) }
    }

    fn finish(&mut self) -> Result<(), String> {
        self.stop.store(true, Ordering::Release);
        self.worker.take().expect("owned server worker").join()
            .unwrap_or_else(|_| Err("server panicked".into()))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}

fn memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, tree: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        tree.insert(path.strip_prefix(root).unwrap().to_owned(), None);
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, tree);
            } else {
                assert!(entry.file_type().unwrap().is_file());
                tree.insert(path.strip_prefix(root).unwrap().to_owned(), Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

struct CancelAtWorkerExit(Weak<MemorySynthesisService>);

impl Drop for CancelAtWorkerExit {
    fn drop(&mut self) {
        self.0.upgrade().expect("live consumer cancellation fixture").dispose();
    }
}

fn run_child(root: &Path, case: Case) {
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    settings.set_host_settings(&json!({
        "inferenceProvider":"codex","agentDefaultModel":bound_model(MODEL_A),
    })).unwrap();
    let auth = start_host_auth_extension_with_options(HostAuthServiceOptions {
        environment: HostAuthEnvironment::from_pairs([(
            "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE".into(),
            root.join("credential.json").to_string_lossy().into_owned(),
        )]),
        backend_url: Some("http://127.0.0.1:9".into()), backend: None,
        machine_id: Arc::new(|| Ok("msi-native-device".into())),
        log: Arc::new(|_| {}), now_ms: Arc::new(|| 10_000),
    }, Arc::new(|_| Ok(None))).unwrap();
    let inference = Arc::new(start_production_inference_extension(Arc::new(auth),
        Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
            is_dev_build:false, env_gate_overrides:None,
        })), Arc::clone(&settings)));
    assert_eq!(settings.get_inference_provider(), "codex");
    assert_eq!(inference.route(), InferenceRoute::Routed(RoutedProvider::Fabushi),
        "product mode must select Native for the actual production factory");

    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
    let agent = sessions.create_session(None, "user", None).unwrap();
    workers.bind_memory_synthesis_gate();
    let memory = workers.memory_service();
    let target = ProductionSynthesisTarget {
        agent_id:agent.id.clone(),
        store:memory.synthesis_target_for_agent(&agent.id).unwrap(),
        service:Arc::clone(&memory),
    };
    assert_eq!(target.apply_synthesis(&target.prepare_synthesis(), &[
        SynthesisChange::Create { content:SOURCE.into(), kind:MemoryKind::Profile },
        SynthesisChange::Create { content:RETIRED.into(), kind:MemoryKind::Log },
    ], NOW), SynthesisApplyResult::Committed);
    workers.with_agent_admitted_filesystem_mutation(&agent.id, || {
        target.store.add_memory(EXPLICIT, NOW, MemoryKind::Profile).map_err(|error| error.to_string())?;
        target.store.remove_memory(&memory_id_for(RETIRED)).map_err(|error| error.to_string())?;
        target.store.mark_temporal_review(NOW - MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS - 1)
            .map_err(|error| error.to_string())
    }).unwrap();
    let before = memory_tree(&target.store.get_location());
    let admission = workers.read_session_work_admission(&agent.id).unwrap();
    let reports = Arc::new(Mutex::new(Vec::<SynthesisReport>::new()));
    let reported = Arc::clone(&reports);
    let mut options = production_memory_synthesis_options(memory.clone(), inference, root.to_owned());
    options.now = Arc::new(|| NOW);
    options.deadline = Duration::from_secs(5);
    options.retry_attempts = 3;
    options.retry_initial = Duration::from_millis(1);
    options.retry_max = Duration::from_millis(1);
    options.report = Some(Arc::new(move |report| reported.lock().unwrap().push(report)));
    let consumer = Arc::new(Mutex::new(None::<Weak<MemorySynthesisService>>));
    if matches!(case, Case::CancelBeforeApply) {
        let consumer = Arc::clone(&consumer);
        options.worker_entry = Some(Arc::new(move || {
            Box::new(CancelAtWorkerExit(consumer.lock().unwrap().as_ref().unwrap().clone()))
        }));
    }
    let synthesis = Arc::new(MemorySynthesisService::new(options));
    *consumer.lock().unwrap() = Some(Arc::downgrade(&synthesis));
    synthesis.start();
    let bridge: Arc<dyn MemorySynthesisBridge> = synthesis.clone();
    memory.set_synthesis_bridge(Arc::downgrade(&bridge));
    if !matches!(case, Case::TemporalReplace) {
        assert!(memory.record_memory_evidence(&agent.id, Some("native-evidence-A".into()),
            EVIDENCE, "Acknowledged", NOW));
    }
    let expected = if matches!(case, Case::CancelBeforeApply) {
        SynthesisOutcome::NoWork
    } else if case.refuses() {
        SynthesisOutcome::Failed
    } else {
        SynthesisOutcome::Committed
    };
    assert_eq!(synthesis.run_now(), vec![expected]);
    assert_eq!(workers.read_session_work_admission(&agent.id).unwrap(), admission);
    if case.refuses() || matches!(case, Case::CancelBeforeApply) {
        assert_eq!(memory_tree(&target.store.get_location()), before,
            "identity/cancellation refusal must preserve the entire memory tree");
        assert!(target.is_temporal_review_due(NOW));
    }
    assert!(synthesis.run_now().is_empty(), "settled snapshots must not replay implicitly");
    if matches!(case, Case::CancelBeforeApply) {
        assert!(reports.lock().unwrap().is_empty());
    } else {
        let first_report = reports.lock().unwrap()[0].clone();
        assert_eq!(first_report.outcome, if case.refuses() {
            SynthesisReportOutcome::Failed
        } else { SynthesisReportOutcome::Committed });
        assert_eq!(first_report.evidence_count, usize::from(!matches!(case, Case::TemporalReplace)));
        assert!(memory.record_memory_evidence(&agent.id, Some("native-evidence-B".into()),
            "MSI later independent evidence", "Acknowledged", NOW));
        assert_eq!(synthesis.run_now(), vec![SynthesisOutcome::Committed]);
        assert_eq!(reports.lock().unwrap().len(), 2);
        assert_eq!(reports.lock().unwrap()[1].evidence_count, 1,
            "the refused snapshot evidence must settle without erasing later evidence");
        let facts = target.prepare_synthesis().memories;
        assert_eq!(facts.iter().filter(|fact| fact.content == UPDATED).count(), 1);
        assert!(!facts.iter().any(|fact| fact.content == SOURCE || fact.content == RETIRED));
        assert!(facts.iter().any(|fact| fact.content == EXPLICIT && fact.origin == MemoryOrigin::Explicit));
        assert!(target.store.get_location().join(".dreaming/tombstones")
            .join(format!("{}.deleted", memory_id_for(RETIRED))).is_file());
        assert_eq!(fs::read(target.store.get_location().join(REFRESH)).unwrap(),
            format!("{}\n", NOW + MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS).into_bytes());
    }
    memory.clear_synthesis_bridge();
    synthesis.dispose();
    workers.shutdown();
}

fn isolated(name: &str, case: Case) {
    let (_, module) = module_path!().split_once("::").unwrap();
    let exact = format!("{module}::{name}");
    if let Ok(selected) = std::env::var(CHILD_TEST) {
        assert_eq!(selected, exact);
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT).unwrap());
        run_child(&root, case);
        private_write(&root.join("completed.json"), json!({
            "test":exact,"nonce":std::env::var(CHILD_NONCE).unwrap(),
            "pid":std::process::id(),"completed":true,
        }).to_string().as_bytes());
        return;
    }
    let mut root = OwnedRoot {
        path:std::env::temp_dir().join(format!("msi-native-{}", uuid::Uuid::new_v4())),
        clean:false,
    };
    let mut directory = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(&root.path).unwrap();
    for name in ["home", "profile", "codex"] { fs::create_dir(root.path.join(name)).unwrap(); }
    let mut server = Server::new(&root.path, case);
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.path.join("stdout.log");
    let stderr_path = root.path.join("stderr.log");
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", &exact, "--nocapture", "--test-threads=1"])
        .env(CHILD_TEST, &exact).env(CHILD_ROOT, &root.path).env(CHILD_NONCE, &nonce)
        .env("HOME", root.path.join("home")).env("USERPROFILE", root.path.join("profile"))
        .env("CODEX_HOME", root.path.join("codex"))
        .env("FABUSHI_PRODUCT_MODE", "1").env("SAND_PACKAGED", "1")
        .env("FABUSHI_RESPONSES_URL", &server.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.path.join("credential.json"))
        .env("NO_PROXY", "*").env("no_proxy", "*")
        .stdout(Stdio::from(File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(File::create(&stderr_path).unwrap()));
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy",
        "SAND_AGENT_MODEL", "SAND_AGENT_MOCK_RESPONSE", "SAND_CODEX_MODEL",
        "FABUSHI_RESPONSES_MODEL", "OPENAI_API_KEY", "CODEX_API_KEY"] {
        command.env_remove(key);
    }
    let mut child = OwnedChild {
        process:command.spawn().unwrap(), terminal:false, retirement:None,
    };
    let pid = child.process.id();
    let status = child.wait_bounded();
    let server_result = server.finish();
    let stdout = fs::read_to_string(&stdout_path).unwrap();
    let stderr = fs::read_to_string(&stderr_path).unwrap();
    assert!(matches!(&status, Ok(status) if status.success()) && server_result.is_ok(),
        "MSI child/server failed: child={status:?}; server={server_result:?}; retained={}\n{stdout}\n{stderr}",
        root.path.display());
    assert!(child.terminal);
    let receipt: Value = serde_json::from_slice(&fs::read(root.path.join("completed.json")).unwrap()).unwrap();
    assert_eq!(receipt, json!({"test":exact,"nonce":nonce,"pid":pid,"completed":true}));
    for sensitive in [TOKEN_A, TOKEN_ROTATED, TOKEN_B, SOURCE, EVIDENCE] {
        assert!(!stdout.contains(sensitive) && !stderr.contains(sensitive),
            "successful production diagnostics must not contain private fixture data");
    }
    let requests = server.requests.lock().unwrap();
    let first_count = case.first_request_count();
    let later_count = if matches!(case, Case::CancelBeforeApply) { 0 } else { 2 };
    assert_eq!(requests.len(), first_count + later_count, "exact model dispatch count");
    for (index, request) in requests.iter().enumerate() {
        assert_eq!(request.model, if index < first_count { MODEL_A } else { MODEL_B });
        let token = if index == 0 { TOKEN_A } else if case.refuses() { TOKEN_B } else { TOKEN_ROTATED };
        assert!(request.authorization == format!("Bearer {token}"), "actual request credential identity");
        assert!(request.payload["currentMemories"].as_array().unwrap()
            .iter().any(|memory| memory["content"] == SOURCE || memory["content"] == UPDATED));
    }
    assert!(!requests[0].verify, "first observed dispatch must be a real proposal");
    if matches!(case, Case::TemporalReplace) {
        assert!(requests[0].payload["newEvidence"].as_array().unwrap().is_empty());
    } else {
        assert!(requests[0].payload["newEvidence"].to_string().contains(EVIDENCE));
    }
    if !case.refuses() {
        let verify = &requests[first_count - 1];
        assert!(verify.verify);
        assert_eq!(verify.payload["currentMemories"], requests[0].payload["currentMemories"]);
        assert_eq!(verify.payload["evidence"], requests[0].payload["newEvidence"]);
        if case.retries() {
            assert!(!requests[1].verify);
            assert_eq!(requests[1].payload, requests[0].payload, "retry uses the same snapshot");
        }
    }
    if later_count != 0 {
        assert!(!requests[first_count].verify);
        assert!(requests[first_count + 1].verify);
        assert_eq!(requests[first_count].payload["newEvidence"][0]["id"], "native-evidence-B");
    }
    drop(requests);
    root.clean = true;
}

#[test]
fn production_native_synthesis_pins_model_and_rotates_token() {
    isolated("production_native_synthesis_pins_model_and_rotates_token", Case::Rotate);
}

#[test]
fn production_native_synthesis_refuses_changed_context_without_writes() {
    isolated("production_native_synthesis_refuses_changed_context_without_writes", Case::Replace);
}

#[test]
fn production_native_temporal_synthesis_refuses_changed_context_without_review() {
    isolated("production_native_temporal_synthesis_refuses_changed_context_without_review", Case::TemporalReplace);
}

#[test]
fn production_native_synthesis_retry_keeps_snapshot_identity() {
    isolated("production_native_synthesis_retry_keeps_snapshot_identity", Case::RetryRotate);
}

#[test]
fn production_native_synthesis_retry_refuses_changed_context() {
    isolated("production_native_synthesis_retry_refuses_changed_context", Case::RetryReplace);
}

#[test]
fn production_native_synthesis_cancellation_after_worker_success_prevents_apply() {
    isolated("production_native_synthesis_cancellation_after_worker_success_prevents_apply", Case::CancelBeforeApply);
}
