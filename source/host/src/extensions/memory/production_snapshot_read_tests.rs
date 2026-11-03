//! MSSA-v1 through the actual production Memory factory, Native HTTP and Session.
//! The peer accepts only post-repair requests with complete persisted facts.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
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
    format_memory_date, memory_id_for,
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
const MODEL: &str = "Native/MSSA/CaseSensitive";
const TOKEN: &str = "mssa-private-fixture-token";
const PROFILE: &str = "MSSA persisted profile sentinel";
const LOG: &str = "MSSA persisted monthly sentinel";
const EXPLICIT: &str = "MSSA explicit fact remains protected";
const RETIRED: &str = "MSSA retired fact stays forgotten";
const CREATED: &str = "MSSA repair completed through production inference";
const REFRESH: &str = ".dreaming/next-refresh-at";
const CHILD_TEST: &str = "FABUSHI_MSSA_NATIVE_CHILD_TEST";
const CHILD_ROOT: &str = "FABUSHI_MSSA_NATIVE_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_MSSA_NATIVE_CHILD_NONCE";

fn private_write(path: &Path, bytes: &[u8]) {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).expect("create owned private fixture file");
    file.write_all(bytes).expect("write owned private fixture file");
    file.sync_all().expect("sync owned private fixture file");
}

fn read_bounded_fixture_file(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
            "fixture file type or size is outside its diagnostic bound"));
    }
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
            "opened fixture file type or size is outside its diagnostic bound"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
            "fixture file grew beyond its diagnostic bound"));
    }
    Ok(bytes)
}

fn read_fixture_diagnostic(path: &Path) -> std::io::Result<String> {
    String::from_utf8(read_bounded_fixture_file(path, 1024 * 1024)?)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData,
            "fixture diagnostic is not UTF-8"))
}

fn memory_tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut pending = vec![root.to_path_buf()];
    let mut tree = BTreeMap::new();
    while let Some(path) = pending.pop() {
        let relative = path.strip_prefix(root).unwrap().to_owned();
        let kind = fs::symlink_metadata(&path).unwrap().file_type();
        if kind.is_dir() {
            tree.insert(relative, None);
            for entry in fs::read_dir(&path).unwrap() {
                pending.push(entry.unwrap().path());
            }
        } else {
            assert!(kind.is_file(), "fixture owns ordinary files and directories");
            tree.insert(relative, Some(fs::read(&path).unwrap()));
        }
    }
    tree
}

struct Request {
    evidence: String,
    verify: bool,
    payload: Value,
}

fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    stream.set_nonblocking(false).map_err(|_| "accepted socket mode")?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|_| "request timeout")?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|_| "response timeout")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut bytes = Vec::new();
    loop {
        if Instant::now() >= deadline { return Err("request deadline".into()); }
        let mut chunk = [0; 8192];
        let count = stream.read(&mut chunk).map_err(|_| "request read")?;
        if count == 0 { return Err("incomplete request".into()); }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() > 1024 * 1024 { return Err("request size bound".into()); }
        let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else { continue; };
        let headers = std::str::from_utf8(&bytes[..end]).map_err(|_| "header encoding")?;
        if headers.lines().next() != Some("POST /v1/responses HTTP/1.1") {
            return Err("unexpected method/path".into());
        }
        let mut length = None;
        let mut authorized = false;
        for header in headers.lines().skip(1) {
            let (name, value) = header.split_once(':').ok_or("header syntax")?;
            if name.eq_ignore_ascii_case("content-length") {
                if length.replace(value.trim().parse::<usize>().map_err(|_| "body length")?).is_some() {
                    return Err("duplicate length".into());
                }
            } else if name.eq_ignore_ascii_case("authorization") {
                if authorized || value.trim() != format!("Bearer {TOKEN}") {
                    return Err("Native fixture authorization".into());
                }
                authorized = true;
            }
        }
        let length = length.ok_or("missing body length")?;
        if length > 1024 * 1024 { return Err("body size bound".into()); }
        if bytes.len() < end + 4 + length { continue; }
        if !authorized { return Err("missing Native authorization".into()); }
        let body: Value = serde_json::from_slice(&bytes[end + 4..end + 4 + length])
            .map_err(|_| "request JSON")?;
        if body["model"] != MODEL { return Err("actual selected model".into()); }
        if body.get("tools").is_some_and(|value| value.as_array().is_none_or(|tools| !tools.is_empty())) {
            return Err("summary exposed tools".into());
        }
        let input = body["input"].as_array().ok_or("summary input")?;
        let user = input.iter().rev().find(|item| item["role"] == "user")
            .and_then(|item| item["content"].as_str()).ok_or("summary user message")?;
        let payload: Value = serde_json::from_str(user).map_err(|_| "Memory request JSON")?;
        let verify = payload.get("proposedChanges").is_some();
        let marker = if verify { MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER }
            else { MEMORY_SYNTHESIS_PROMPT_MARKER };
        if !body["instructions"].as_str().is_some_and(|text| text.contains(marker)) {
            return Err("actual Memory system prompt".into());
        }
        let facts = payload["currentMemories"].as_array().ok_or("current memories")?;
        for required in [PROFILE, LOG, EXPLICIT] {
            if !facts.iter().any(|fact| fact["content"] == required) {
                return Err("incomplete persisted snapshot reached actual transport".into());
            }
        }
        if facts.iter().any(|fact| fact["content"] == RETIRED) {
            return Err("forgotten fact reached transport".into());
        }
        let evidence_key = if verify { "evidence" } else { "newEvidence" };
        let evidence = payload[evidence_key].as_array().ok_or("evidence")?;
        if evidence.len() != 1 { return Err("captured evidence did not settle".into()); }
        let evidence = evidence[0]["id"].as_str().ok_or("evidence id")?.to_string();
        if !["snapshot-repair-profile", "snapshot-repair-log", "snapshot-repair-all-queued"]
            .contains(&evidence.as_str()) {
            return Err("a request was dispatched during refused acquisition".into());
        }
        return Ok(Request { evidence, verify, payload });
    }
}

fn respond(stream: &mut TcpStream, output: Value) -> Result<(), String> {
    let events = [
        json!({"type":"response.output_text.delta","delta":output.to_string()}),
        json!({"type":"response.completed","response":{
            "id":"mssa-response","usage":{"input_tokens":11,"output_tokens":2},"output":[],
        }}),
    ];
    let body = events.iter().map(|event| format!("data: {event}\n\n")).collect::<String>();
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|_| "response write".into())
}

struct Peer {
    endpoint: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl Peer {
    fn start(root: &Path) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("owned loopback listener");
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        private_write(&root.join("credential.json"), json!({
            "version":1,"issuer":origin,"accessToken":TOKEN,"expiresAtMs":4_102_444_800_000_u64,
            "sessionId":"mssa-session","deviceId":"mssa-device","userId":"mssa-user",
        }).to_string().as_bytes());
        let endpoint = format!("{origin}/v1/responses");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(60);
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= deadline { return Err("peer deadline".into()); }
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(_) => return Err("peer accept".into()),
                };
                let request = read_request(&mut stream)?;
                let output = if request.verify {
                    let changes = request.payload["proposedChanges"].as_array().ok_or("verification changes")?;
                    if changes.len() != 1 || changes[0]["content"] != CREATED {
                        return Err("verification did not observe actual repair proposal".into());
                    }
                    json!({"approved":true})
                } else {
                    json!({"changes":[{
                        "action":"create","content":CREATED,"kind":"profile",
                        "sourceEvidenceIds":[request.evidence],
                    }]})
                };
                captured.lock().unwrap().push(request);
                respond(&mut stream, output)?;
            }
            Ok(())
        });
        Self { endpoint, requests, stop, worker: Some(worker) }
    }

    fn finish(&mut self) -> Result<(), String> {
        self.stop.store(true, Ordering::Release);
        self.worker.take().expect("owned peer worker").join()
            .unwrap_or_else(|_| Err("peer panicked".into()))
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Failure cleanup joins only this fixture-owned, deadline-bounded worker.
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}

struct OwnedChild {
    process: Child,
    terminal: bool,
    retirement: Option<String>,
}

impl OwnedChild {
    fn retire_once(&mut self) -> String {
        if let Some(result) = &self.retirement { return result.clone(); }
        self.retirement = Some("retirement started; terminal unknown".into());
        let kill = self.process.kill();
        let deadline = Instant::now() + Duration::from_secs(3);
        let observed = loop {
            let state = match self.process.try_wait() {
                Ok(Some(status)) => { self.terminal = true; break format!("terminal={status:?}"); }
                Ok(None) => "still running".to_string(),
                Err(error) => format!("terminal observation failed: {error}"),
            };
            if Instant::now() >= deadline { break format!("terminal=false; {state}"); }
            thread::sleep(Duration::from_millis(10));
        };
        let result = format!("kill={kill:?}; {observed}");
        self.retirement = Some(result.clone());
        result
    }

    fn wait_bounded(&mut self) -> Result<std::process::ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            match self.process.try_wait() {
                Ok(Some(status)) => { self.terminal = true; return Ok(status); }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                observed => {
                    let retirement = self.retire_once();
                    return Err(format!("child wait failed: {observed:?}; {retirement}"));
                }
            }
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.terminal && self.retirement.is_none() {
            eprintln!("MSSA child retirement: {}", self.retire_once());
        }
    }
}

fn run_child(root: &Path) {
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    settings.set_host_settings(&json!({
        "inferenceProvider":"codex",
        "agentDefaultModel":{"provider":"fabushi","modelId":MODEL,"maxMode":true,"parameters":[]},
    })).unwrap();
    let auth = start_host_auth_extension_with_options(HostAuthServiceOptions {
        environment: HostAuthEnvironment::from_pairs([(
            "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE".into(),
            root.join("credential.json").to_string_lossy().into_owned(),
        )]),
        backend_url: Some("http://127.0.0.1:9".into()), backend: None,
        machine_id: Arc::new(|| Ok("mssa-device".into())),
        log: Arc::new(|_| {}), now_ms: Arc::new(|| 10_000),
    }, Arc::new(|_| Ok(None))).unwrap();
    let inference = Arc::new(start_production_inference_extension(Arc::new(auth),
        Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
            is_dev_build:false, env_gate_overrides:None,
        })), settings));
    assert_eq!(inference.route(), InferenceRoute::Routed(RoutedProvider::Fabushi));

    for case in ["profile", "log", "all-queued"] {
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join(case).join("agents"), 500));
        let sessions = SandAgentSessionStore::new(workers.clone());
        let agent = sessions.create_session(None, "user", None).unwrap();
        drop(sessions);
        workers.bind_memory_synthesis_gate();
        let memory = workers.memory_service();
        let target = ProductionSynthesisTarget {
            agent_id:agent.id.clone(), store:memory.synthesis_target_for_agent(&agent.id).unwrap(),
            service:memory.clone(),
        };
        let empty = target.prepare_synthesis().expect("healthy initial snapshot");
        assert_eq!(target.apply_synthesis(&empty, &[
            SynthesisChange::Create { content:PROFILE.into(), kind:MemoryKind::Profile },
            SynthesisChange::Create { content:LOG.into(), kind:MemoryKind::Log },
            SynthesisChange::Create { content:RETIRED.into(), kind:MemoryKind::Log },
        ], NOW), SynthesisApplyResult::Committed);
        workers.with_agent_admitted_filesystem_mutation(&agent.id, || {
            target.store.add_memory(EXPLICIT, NOW, MemoryKind::Profile).map_err(|error| error.to_string())?;
            target.store.remove_memory(&memory_id_for(RETIRED)).map_err(|error| error.to_string())?;
            target.store.mark_temporal_review(NOW - MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS - 1)
                .map_err(|error| error.to_string())
        }).unwrap();
        let healthy = target.prepare_synthesis().unwrap();
        let directory = target.store.get_location();
        let marker = fs::read(directory.join(REFRESH)).unwrap();
        let admission = workers.read_session_work_admission(&agent.id).unwrap();
        let reports = Arc::new(Mutex::new(Vec::<SynthesisReport>::new()));
        let reported = reports.clone();
        let allocations = Arc::new(AtomicUsize::new(0));
        let allocated = allocations.clone();
        let mut options = production_memory_synthesis_options(memory.clone(), inference.clone(), root.to_owned());
        let actual_factory = options.job_factory.clone();
        options.job_factory = Arc::new(move || {
            allocated.fetch_add(1, Ordering::SeqCst);
            actual_factory()
        });
        options.now = Arc::new(|| NOW);
        options.deadline = Duration::from_secs(5);
        options.retry_attempts = 1;
        options.report = Some(Arc::new(move |report| reported.lock().unwrap().push(report)));
        let synthesis = Arc::new(MemorySynthesisService::new(options));
        let bridge: Arc<dyn MemorySynthesisBridge> = synthesis.clone();
        memory.set_synthesis_bridge(Arc::downgrade(&bridge));
        if case == "all-queued" { synthesis.start(); }
        let profile = directory.join("profile.md");
        let log = directory.join("log").join(format!("{}.md", &format_memory_date(NOW)[..7]));
        let paths = match case {
            "profile" => vec![profile],
            "log" => vec![log],
            _ => vec![profile, log],
        };
        let mut repairs = Vec::new();
        for path in paths {
            let original = fs::read(&path).unwrap();
            let mut corrupt = original.clone();
            corrupt.extend_from_slice(&[0xff, 0xfe]);
            fs::write(&path, &corrupt).unwrap();
            assert_eq!(fs::read(&path).unwrap(), corrupt);
            assert_eq!(fs::read_to_string(&path).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
            repairs.push((path, original));
        }
        if case != "all-queued" { synthesis.start(); }
        // The wholly unreadable case has only the already queued temporal work.
        if case != "all-queued" {
            assert!(memory.record_memory_evidence(&agent.id, Some("snapshot-refused-evidence".into()),
                "refused input", "refused reply", NOW));
        }
        let before = memory_tree(&directory);
        assert_eq!(synthesis.run_now(), vec![SynthesisOutcome::Failed]);
        assert_eq!(allocations.load(Ordering::SeqCst), 0, "refused acquisition allocates no real job");
        assert_eq!(memory_tree(&directory), before);
        assert_eq!(fs::read(directory.join(REFRESH)).unwrap(), marker);
        assert!(target.is_temporal_review_due(NOW));
        assert_eq!(workers.read_session_work_admission(&agent.id).unwrap(), admission);
        assert_eq!(reports.lock().unwrap().as_slice(), &[SynthesisReport {
            outcome:SynthesisReportOutcome::Failed, agent_id:Some(agent.id.clone()),
            evidence_count:usize::from(case != "all-queued"), input_memory_count:0,
            change_count:0, duration_ms:0,
        }]);
        assert!(synthesis.run_now().is_empty(), "refused acquisition settles without immediate replay");

        for (path, bytes) in repairs { fs::write(path, bytes).expect("explicit owned repair"); }
        assert_eq!(target.prepare_synthesis().unwrap(), healthy);
        assert!(memory.record_memory_evidence(&agent.id, Some(format!("snapshot-repair-{case}")),
            "The readable snapshot may now be reviewed.", "Acknowledged.", NOW));
        assert_eq!(synthesis.run_now(), vec![SynthesisOutcome::Committed]);
        assert_eq!(allocations.load(Ordering::SeqCst), 1);
        assert_eq!(reports.lock().unwrap().len(), 2);
        assert_eq!(reports.lock().unwrap()[1].outcome, SynthesisReportOutcome::Committed);
        assert_eq!(reports.lock().unwrap()[1].evidence_count, 1);
        let final_snapshot = target.prepare_synthesis().unwrap();
        for original in &healthy.memories { assert!(final_snapshot.memories.contains(original)); }
        let created = final_snapshot.memories.iter().filter(|record| record.content == CREATED).collect::<Vec<_>>();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].origin, MemoryOrigin::Synthesis);
        assert!(directory.join(".dreaming/tombstones").join(format!("{}.deleted", memory_id_for(RETIRED))).is_file());
        assert_eq!(fs::read(directory.join(REFRESH)).unwrap(),
            format!("{}\n", NOW + MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS).into_bytes());
        assert_eq!(workers.read_session_work_admission(&agent.id).unwrap(), admission);
        assert!(synthesis.run_now().is_empty());
        memory.clear_synthesis_bridge();
        drop(bridge);
        synthesis.dispose();
        drop(synthesis);
        workers.shutdown();
        drop(target);
        drop(memory);
        let owner = Arc::downgrade(&workers);
        drop(workers);
        assert!(owner.upgrade().is_none(), "retire actual Session owners before terminal receipt");
    }
}

#[test]
fn production_native_synthesis_snapshot_refuses_unreadable_inputs_before_dispatch_and_recovers() {
    const NAME: &str = "production_native_synthesis_snapshot_refuses_unreadable_inputs_before_dispatch_and_recovers";
    let (_, module) = module_path!().split_once("::").unwrap();
    let exact = format!("{module}::{NAME}");
    if let Ok(selected) = std::env::var(CHILD_TEST) {
        assert_eq!(selected, exact);
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT).unwrap());
        run_child(&root);
        private_write(&root.join("completed.json"), json!({
            "test":exact,"nonce":std::env::var(CHILD_NONCE).unwrap(),
            "pid":std::process::id(),"completed":true,
        }).to_string().as_bytes());
        return;
    }
    let root = std::env::temp_dir().join(format!("mssa-native-{}", uuid::Uuid::new_v4()));
    let mut directory = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(&root).unwrap();
    for name in ["home", "profile", "codex", "tmp"] { fs::create_dir(root.join(name)).unwrap(); }
    let mut peer = Peer::start(&root);
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.join("stdout.log");
    let stderr_path = root.join("stderr.log");
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.env_clear();
    // Cargo's dynamic-library paths and native loader roots are the only
    // inherited inputs. Product/provider/root configuration is fixture-owned.
    for key in ["PATH", "SystemRoot", "WINDIR", "LD_LIBRARY_PATH",
        "DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH"] {
        if let Some(value) = std::env::var_os(key) { command.env(key, value); }
    }
    command.args(["--exact", &exact, "--nocapture", "--test-threads=1"])
        .env(CHILD_TEST, &exact).env(CHILD_ROOT, &root).env(CHILD_NONCE, &nonce)
        .env("HOME", root.join("home")).env("USERPROFILE", root.join("profile"))
        .env("CODEX_HOME", root.join("codex"))
        .env("TMPDIR", root.join("tmp")).env("TEMP", root.join("tmp")).env("TMP", root.join("tmp"))
        .env("FABUSHI_PRODUCT_MODE", "1").env("SAND_PACKAGED", "1")
        .env("FABUSHI_RESPONSES_URL", &peer.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.join("credential.json"))
        .env("NO_PROXY", "*").env("no_proxy", "*")
        .stdout(Stdio::from(File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(File::create(&stderr_path).unwrap()));
    let mut child = OwnedChild { process:command.spawn().unwrap(), terminal:false, retirement:None };
    drop(command);
    let pid = child.process.id();
    let status = child.wait_bounded();
    let peer_result = peer.finish();
    let stdout = read_fixture_diagnostic(&stdout_path).unwrap_or_else(|error| {
        panic!("MSSA stdout unavailable within bound; retained={}: {error}", root.display())
    });
    let stderr = read_fixture_diagnostic(&stderr_path).unwrap_or_else(|error| {
        panic!("MSSA stderr unavailable within bound; retained={}: {error}", root.display())
    });
    assert!(matches!(&status, Ok(status) if status.success()) && peer_result.is_ok(),
        "MSSA child/peer failed: child={status:?}; peer={peer_result:?}; retained={}\n{stdout}\n{stderr}",
        root.display());
    assert!(child.terminal);
    let receipt_bytes = read_bounded_fixture_file(&root.join("completed.json"), 16 * 1024)
        .unwrap_or_else(|error| {
            panic!("MSSA terminal receipt unavailable within bound; retained={}: {error}", root.display())
        });
    let receipt: Value = serde_json::from_slice(&receipt_bytes).expect("valid bounded terminal receipt");
    assert_eq!(receipt, json!({"test":exact,"nonce":nonce,"pid":pid,"completed":true}));
    for sensitive in [TOKEN, PROFILE, LOG, EXPLICIT] {
        assert!(!stdout.contains(sensitive) && !stderr.contains(sensitive),
            "successful fixture diagnostics must not expose private data");
    }
    let requests = peer.requests.lock().unwrap();
    assert_eq!(requests.len(), 6, "exactly proposal + verification after each of three repairs");
    for (index, case) in ["profile", "log", "all-queued"].iter().enumerate() {
        let proposal = &requests[index * 2];
        let verification = &requests[index * 2 + 1];
        assert!(!proposal.verify && verification.verify);
        assert_eq!(proposal.evidence, format!("snapshot-repair-{case}"));
        assert_eq!(verification.evidence, proposal.evidence);
        assert_eq!(proposal.payload["currentMemories"], verification.payload["currentMemories"]);
    }
    drop(requests);
    drop(peer);
    drop(child);
    fs::remove_dir_all(root).expect("strict cleanup after owned process and peer termination");
}
