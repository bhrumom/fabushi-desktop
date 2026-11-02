//! Real Responses HTTP and private-file replacement at the Native auth boundary.
//! Covers per-transport refresh and the public Runner's automatic-attempt binding.

#[path = "fabushi_responses_credential_context_contract/automatic_attempt_binding.rs"]
mod automatic_attempt_binding;

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc::{self, SyncSender}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::auth::auth_service::EXPIRY_LEEWAY_MS;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint, RoutedProviderOptions,
    run_routed_provider_text_with_lifecycle_reporting_usage,
    run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProductionTurnRunShellAdapter, ProviderRetryOutcome, RoutedProviderAttemptControl,
    RoutedProviderAttemptExecutor, RoutedProviderCheckpointStore,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedProviderCancellation;
use mahayana_host_runtime::runner::StreamAttemptPolicy;
use serde_json::{Value, json};

const TEST_NAME: &str = "fabushi_responses_bind_refresh_to_the_original_credential_context";
const CHILD_ROOT: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_NONCE";
const CHILD_EXPECT: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_EXPECT";
const CHILD_NEW: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_NEW_TRANSPORT";
const CHILD_CONTROL: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_CONTROL";
const CHILD_CASE: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_CASE";
const CHILD_PHASE: &str = "FABUSHI_CONTEXT_CONTRACT_CHILD_PHASE";
const MODEL: &str = "Vendor/Nested/CaseSensitive@Credential-Context";
const TOKEN_A: &str = "credential-context-private-token-a";
const TOKEN_B: &str = "credential-context-private-token-b";
const REFRESH_SECRET: &str = "credential-context-private-refresh-secret";
const BOUND: Duration = Duration::from_secs(20);
const REQUEST_PATH: &str = "/tenant-a/codex/v1/responses";

struct TempRoot(PathBuf, Arc<AtomicUsize>);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("fabushi-credential-context-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).expect("exclusively create private test root");
        Self(path, Arc::new(AtomicUsize::new(0)))
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let outstanding = self.1.load(Ordering::Acquire);
        if outstanding == 0 {
            let _ = fs::remove_dir_all(&self.0);
        } else {
            eprintln!("credential fixture root retained path={} outstanding_owned_children={outstanding}", self.0.display());
        }
    }
}

enum Image {
    Document(Value),
    Raw(String),
    Missing,
    Directory,
    #[cfg(unix)]
    Public(Value),
    #[cfg(unix)]
    Symlink(Value),
}

fn fixture_io_error(stage: &str, error: io::Error) -> String {
    format!("{stage}: kind={:?}, os={:?}", error.kind(), error.raw_os_error())
}

fn write_private(path: &Path, raw: &str) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(&temporary).map_err(|error| fixture_io_error("replacement-create", error))?
        .write_all(raw.as_bytes()).map_err(|error| fixture_io_error("replacement-write", error))?;
    fs::rename(temporary, path).map_err(|error| fixture_io_error("replacement-rename", error))
}

fn remove_image_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(fixture_io_error("replacement-remove", error)),
    }
}

fn install_image(path: &Path, image: Image) -> Result<(), String> {
    match image {
        Image::Document(document) => write_private(path, &document.to_string()),
        Image::Raw(raw) => write_private(path, &raw),
        Image::Missing => remove_image_file(path),
        Image::Directory => {
            remove_image_file(path)?;
            fs::create_dir(path).map_err(|error| fixture_io_error("replacement-directory", error))
        }
        #[cfg(unix)]
        Image::Public(document) => {
            use std::os::unix::fs::PermissionsExt;
            write_private(path, &document.to_string())?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o644))
                .map_err(|error| fixture_io_error("replacement-permissions", error))
        }
        #[cfg(unix)]
        Image::Symlink(document) => {
            let target = path.with_extension("target");
            write_private(&target, &document.to_string())?;
            remove_image_file(path)?;
            std::os::unix::fs::symlink(target, path)
                .map_err(|error| fixture_io_error("replacement-symlink", error))
        }
    }
}

fn credential(origin: &str, token: &str) -> Value {
    json!({
        "version": 1, "issuer": origin, "accessToken": token,
        "expiresAtMs": 4_102_444_800_000_u64,
        "sessionId": "session-a", "deviceId": "device-a", "userId": 42
    })
}

fn replacement(case: &str, origin: &str) -> Image {
    let mut document = credential(origin, TOKEN_B);
    // Token and expiry are deliberately different even in the positive case.
    document["expiresAtMs"] = json!(4_102_444_860_000_u64);
    match case {
        "rotate" | "second-401" => {}
        "numeric-to-string" => document["userId"] = json!("42"),
        "canonical-issuer" => document["issuer"] = json!(format!("{origin}/")),
        "user" => document["userId"] = json!("account-b"),
        "user-not-numeric-normalized" => document["userId"] = json!("042"),
        "user-whitespace" => document["userId"] = json!("42 "),
        "device" => document["deviceId"] = json!("device-b"),
        "session" => document["sessionId"] = json!("new-login-for-the-same-user"),
        "issuer" => document["issuer"] = json!("https://api.ombhrum.com"),
        "unsupported-issuer" => document["issuer"] = json!("https://untrusted.example"),
        "malformed-issuer" => document["issuer"] = json!(false),
        "version" => document["version"] = json!(2),
        "malformed-user" => document["userId"] = json!({"id": 42}),
        "blank-user" => document["userId"] = json!(" \t"),
        "blank-device" => document["deviceId"] = json!(" "),
        "blank-session" => document["sessionId"] = json!("\t"),
        "expired" => document["expiresAtMs"] = json!(1),
        "expiry-leeway" => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_millis() as u64;
            document["expiresAtMs"] = json!(now + EXPIRY_LEEWAY_MS - 1_000);
        }
        "malformed" => return Image::Raw(format!("{{\"accessToken\":\"{TOKEN_B}\",not-json")),
        "secret-type-error" => document["expiresAtMs"] = json!(TOKEN_B),
        "unknown-refresh-field" => document["refreshToken"] = json!(REFRESH_SECRET),
        "token-only" => return Image::Document(json!({"accessToken": TOKEN_B, "issuer": origin})),
        "missing-file" => return Image::Missing,
        "directory" => return Image::Directory,
        #[cfg(unix)]
        "public-file" => return Image::Public(document),
        #[cfg(unix)]
        "symlink" => return Image::Symlink(document),
        missing if missing.starts_with("missing-") => {
            let field = &missing["missing-".len()..];
            assert!(document.as_object_mut().unwrap().remove(field).is_some());
        }
        _ => panic!("unknown credential case: {case}"),
    }
    Image::Document(document)
}

#[derive(Clone)]
struct CapturedRequest {
    request_line: String,
    host: String,
    authorization: String,
    body: Vec<u8>,
    response_status: String,
}

fn read_request(
    stream: &mut TcpStream, mut entered: Option<SyncSender<TcpStream>>,
    progress: &Mutex<Vec<String>>, connection: usize,
) -> Result<CapturedRequest, String> {
    record_progress(progress, connection, "configure-stream")?;
    read_request_with_progress(stream, |stream, phase| {
        record_progress(progress, connection, phase)?;
        if phase == "io-ready" {
            if let Some(entered) = entered.take() {
                let observer = stream.try_clone().map_err(|error| fixture_io_error("request-reader-observer", error))?;
                entered.send(observer).map_err(|_| "request-reader entry receiver closed")?;
            }
        }
        Ok(())
    })
}

fn read_request_with_progress(
    stream: &mut TcpStream,
    mut on_progress: impl FnMut(&TcpStream, &'static str) -> Result<(), String>,
) -> Result<CapturedRequest, String> {
    // Accepted sockets can inherit O_NONBLOCK on macOS. Timeouts do not clear it.
    stream.set_nonblocking(false).map_err(|error| fixture_io_error("request-blocking", error))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|error| fixture_io_error("request-read-timeout", error))?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|error| fixture_io_error("request-write-timeout", error))?;
    on_progress(stream, "io-ready")?;
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).map_err(|error| fixture_io_error("request-line", error))?;
    on_progress(reader.get_ref(), "request-line")?;
    let mut content_length = None;
    let mut host = String::new();
    let mut authorization = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|error| fixture_io_error("request-header", error))? == 0 {
            return Err("HTTP headers ended early".into());
        }
        if line == "\r\n" { break; }
        let (name, value) = line.split_once(':').ok_or("invalid header")?;
        match name.to_ascii_lowercase().as_str() {
            "content-length" => content_length = Some(value.trim().parse::<usize>().map_err(|_| "invalid content-length")?),
            "authorization" => authorization = value.trim().to_string(),
            "host" => host = value.trim().to_string(),
            _ => {}
        }
    }
    let length = content_length.ok_or("missing body length")?;
    if length > 4 * 1024 * 1024 { return Err("oversized fixture request".into()); }
    on_progress(reader.get_ref(), "headers")?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|error| fixture_io_error("request-body", error))?;
    Ok(CapturedRequest {
        request_line: request_line.trim_end().into(), host, authorization, body,
        response_status: String::new(),
    })
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) -> Result<(), String> {
    let response = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    stream.write_all(response.as_bytes()).map_err(|error| fixture_io_error("response-write", error))
}

fn record_progress(progress: &Mutex<Vec<String>>, connection: usize, stage: &str) -> Result<(), String> {
    progress.lock().map_err(|_| "progress lock poisoned")?.push(format!("connection={connection} {stage}"));
    Ok(())
}

struct ServerReport {
    result: Result<(), String>,
    progress: Vec<String>,
    requests: Vec<CapturedRequest>,
}

impl ServerReport {
    fn evidence(&self) -> Value {
        json!({
            "capturedRequests": self.requests.len(),
            "accepted": self.progress.iter().filter(|event| event.ends_with(" accepted")).count(),
            "parsed": self.progress.iter().filter(|event| event.ends_with(" request-complete")).count(),
            "responded": self.progress.iter().filter(|event| event.ends_with(" response-written")).count(),
            "phase": self.progress.last(),
            "serverOutcome": match &self.result {
                Ok(()) => "ok",
                Err(error) => error.as_str(),
            },
        })
    }
}

struct Loopback {
    case: String,
    phase: String,
    label: String,
    endpoint: String,
    origin: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    progress: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl Loopback {
    fn new(path: &Path, case: &str, initial_refusal: bool) -> Self {
        Self::start(path, case, initial_refusal, true, None, None, None)
    }

    fn http_error(path: &Path, case: &str, status: &'static str, body: String, rotate: bool) -> Self {
        let mut server = Self::start(path, "rotate", false, rotate, Some((status, body)), None, None);
        server.case = case.into();
        server.phase = if rotate { "http-error-after-401" } else { "initial-http-error" }.into();
        server
    }

    fn start(
        path: &Path,
        case: &str,
        initial_refusal: bool,
        rotate: bool,
        http_error: Option<(&'static str, String)>,
        mut reader_entered: Option<SyncSender<TcpStream>>,
        mut automatic_plan: Option<automatic_attempt_binding::ResponsePlan>,
    ) -> Self {
        let label = format!("case={case} initial_refusal={initial_refusal} rotate={rotate} http_status={}",
            http_error.as_ref().map(|(status, _)| *status).unwrap_or("none"));
        eprintln!("[credential-fixture] start {label}");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind observable Responses fixture");
        listener.set_nonblocking(true).expect("pollable fixture");
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let response_origin = origin.clone();
        let next = if initial_refusal {
            install_image(path, replacement(case, &origin)).expect("install initial negative credential");
            None
        } else {
            install_image(path, Image::Document(credential(&origin, TOKEN_A))).expect("install initial credential");
            rotate.then(|| replacement(case, &origin))
        };
        let repeated_401 = case == "second-401";
        let path = path.to_path_buf();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let capture = Arc::clone(&requests);
        let progress = Arc::new(Mutex::new(vec!["connection=0 listen".into()]));
        let server_progress = Arc::clone(&progress);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + BOUND * 2;
            let mut next = next;
            let mut connections = 0;
            // The listener survives every 401 and returns a complete response
            // to any illicit resend, making absence of a resend observable.
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() > deadline { return Err("fixture deadline".into()); }
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => return Err(fixture_io_error("accept", error)),
                };
                connections += 1;
                record_progress(&server_progress, connections, "accepted")?;
                if reader_entered.is_some() {
                    // The client has connected but cannot send until reader entry.
                    stream.set_nonblocking(true).map_err(|error| fixture_io_error("control-nonblocking", error))?;
                    match stream.peek(&mut [0_u8; 1]) {
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                        Err(error) => return Err(fixture_io_error("control-empty-socket", error)),
                        Ok(_) => return Err("control request arrived before reader entry".into()),
                    }
                    record_progress(&server_progress, connections, "control-nonblocking-empty")?;
                }
                let mut request = read_request(&mut stream, reader_entered.take(), &server_progress, connections)?;
                record_progress(&server_progress, connections, "request-complete")?;
                record_progress(&server_progress, connections, "install-image")?;
                let first_401 = if let Some(image) = next.take() {
                    install_image(&path, image)?;
                    record_progress(&server_progress, connections, "replacement-installed")?;
                    true
                } else { false };
                let (status, content_type, body) = if let Some(plan) = automatic_plan.as_mut() {
                    plan.response(&request, connections, &response_origin, &server_progress)?
                } else if first_401 || repeated_401 {
                    // A credential-bearing response body must not leak into
                    // the terminal 401 diagnostic, including on retry.
                    ("401 Unauthorized", "application/json", json!({
                        "error": "authentication failed", "accessToken": TOKEN_B,
                        "refreshToken": REFRESH_SECRET
                    }).to_string())
                } else if let Some((status, body)) = &http_error {
                    (*status, "application/json", body.clone())
                } else {
                    ("200 OK", "text/event-stream", concat!(
                        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"done\"}\n\n",
                        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"context-response\",\"output\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n",
                        "data: [DONE]\n\n"
                    ).to_string())
                };
                request.response_status = status.into();
                capture.lock().map_err(|_| "capture lock")?.push(request);
                record_progress(&server_progress, connections, &format!("response-planned status={status}"))?;
                record_progress(&server_progress, connections, "write-response")?;
                write_response(&mut stream, status, content_type, &body)?;
                record_progress(&server_progress, connections, "response-written")?;
            }
            Ok(())
        });
        Self {
            case: case.into(), phase: if initial_refusal { "initial-credential" } else { "credential-reload" }.into(),
            label, endpoint: format!("{origin}{REQUEST_PATH}"), origin, requests, progress, stop, worker: Some(worker),
        }
    }

    fn finish(&mut self) -> ServerReport {
        self.stop.store(true, Ordering::Release);
        let mut result = match self.worker.take() {
            Some(worker) => worker.join().unwrap_or_else(|_| Err("fixture worker panicked; payload omitted".into())),
            None => Err("fixture worker already joined".into()),
        };
        let progress = self.progress.lock().unwrap_or_else(|poisoned| {
            result = Err(format!("prior server={result:?}; fixture progress lock poisoned"));
            poisoned.into_inner()
        }).clone();
        let requests = self.requests.lock().unwrap_or_else(|poisoned| {
            result = Err(format!("prior server={result:?}; fixture request capture lock poisoned"));
            poisoned.into_inner()
        }).clone();
        ServerReport { result, progress, requests }
    }
}

impl Drop for Loopback {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let report = self.finish();
            eprintln!("fixture cleanup case={} phase={} label={} server={} progress={:?}",
                self.case, self.phase, self.label, report.evidence(), report.progress);
        }
    }
}

struct OwnedChild(Child, Option<String>, Option<Arc<AtomicUsize>>);

impl OwnedChild {
    fn new(child: Child, root: &TempRoot) -> Self {
        root.1.fetch_add(1, Ordering::AcqRel);
        Self(child, None, Some(Arc::clone(&root.1)))
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.0.try_wait()?;
        if status.is_some() {
            // Only the actual owned handle's exit/reap observation releases
            // deletion permission, and taking this token makes release once-only.
            if let Some(outstanding) = self.2.take() {
                outstanding.fetch_sub(1, Ordering::AcqRel);
            }
        }
        Ok(status)
    }

    fn exit_observed(&self) -> bool {
        self.2.is_none()
    }

    fn stop_and_reap(&mut self) -> Result<ExitStatus, String> {
        let before = self.try_wait().map_err(|error| fixture_io_error("child-reap-status", error));
        if let Ok(Some(status)) = &before {
            self.1.get_or_insert_with(|| format!("already-exited={status:?}"));
            return Ok(status.clone());
        }
        if let Some(prior) = &self.1 {
            return Err(format!("owned child remains unresolved; prior cleanup={prior}; status={before:?}"));
        }
        let deadline = Instant::now() + BOUND;
        let kill = self.0.kill().map_err(|error| fixture_io_error("child-kill", error));
        let reap = loop {
            match self.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if Instant::now() < deadline => {}
                Ok(None) => break Err("owned child reap deadline; root retained".to_string()),
                Err(error) => break Err(fixture_io_error("child-reap-status", error)),
            }
            thread::sleep(Duration::from_millis(5));
        };
        self.1 = Some(format!("status-before={before:?}; kill={kill:?}; reap={reap:?}"));
        match reap {
            Ok(status) if before.is_ok() && kill.is_ok() => Ok(status),
            _ => Err(format!("owned child cleanup failed: {}", self.1.as_deref().unwrap_or("unavailable"))),
        }
    }

    fn wait(&mut self) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + BOUND;
        loop {
            match self.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) if Instant::now() < deadline => {}
                outcome => {
                    let reason = match outcome {
                        Err(error) => fixture_io_error("child-status", error),
                        _ => "credential contract child exceeded its deadline".into(),
                    };
                    let cleanup = self.stop_and_reap();
                    return Err(format!("{reason}; owned cleanup={cleanup:?}"));
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.exit_observed() {
            if let Err(error) = self.stop_and_reap() {
                eprintln!("owned child cleanup pid={} exit_observed={}: {error}", self.0.id(), self.exit_observed());
            }
        }
    }
}

fn exercise_provider(root: &Path, explicit_cancel: bool) -> Value {
    let mut deltas = Vec::new();
    let mut tools = 0;
    let mut checkpoints = 0;
    let mut usages = 0;
    let checked = Cell::new(false);
    let should_cancel = || explicit_cancel && checked.replace(true);
    let result = run_routed_provider_text_with_lifecycle_reporting_usage(
        RoutedProvider::Fabushi,
        &[ProviderMessage { role: "user".into(), content: "Keep exact body and model.".into() }],
        &mut RoutedProviderOptions {
            data_dir: root, typed_replay: None, fabushi_model: Some(MODEL),
            fabushi_request_binding: None, cursor_auth: None,
            tools: &[], mcp_server_url: None,
            execute_tool: &mut |_, _, _| { tools += 1; Ok(json!({})) },
            on_text_delta: &mut |delta, _| deltas.push(delta.to_string()),
            on_partial_tool_call: None, should_cancel: &should_cancel,
        },
        None,
        &mut |_| { checkpoints += 1; Ok(()) },
        &mut |_| { usages += 1; },
    );
    provider_observation(result, deltas, tools, checkpoints, usages)
}

fn provider_observation(
    result: Result<String, ProviderSessionError>,
    deltas: Vec<String>,
    tools: usize,
    checkpoints: usize,
    usages: usize,
) -> Value {
    let (kind, message, debug, refusal) = match result {
        Ok(text) => ("success", text, String::new(), Value::Null),
        Err(error) => {
            let kind = match &error {
                ProviderSessionError::Authentication(_) => "authentication",
                ProviderSessionError::Configuration(_) => "configuration",
                ProviderSessionError::Cancelled(_) => "cancelled",
                ProviderSessionError::Transport(_) => "transport",
                ProviderSessionError::ProviderRefusal(_) => "provider-refusal",
                _ => "unexpected-error",
            };
            let refusal = match &error {
                ProviderSessionError::ProviderRefusal(refusal) => json!(refusal),
                _ => Value::Null,
            };
            (kind, error.to_string(), format!("{error:?}"), refusal)
        }
    };
    json!({"kind":kind, "message":message, "debug":debug, "refusal":refusal,
        "deltas":deltas, "tools":tools, "checkpoints":checkpoints, "usages":usages})
}

struct RefusalCheckpointStore;

impl RoutedProviderCheckpointStore for RefusalCheckpointStore {
    fn persist(&self, _: &RoutedProviderCheckpoint) -> Result<String, ProviderSessionError> {
        panic!("credential/configuration refusal cannot persist a checkpoint")
    }
}

struct RoutedRefusalExecutor<'a> {
    root: &'a Path,
    cancellation: RoutedProviderCancellation,
    cancel_user: bool,
    attempts: usize,
}

impl RoutedProviderAttemptExecutor for RoutedRefusalExecutor<'_> {
    fn run_attempt(
        &mut self,
        attempt: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        assert!(resume_from.is_none());
        let checked = Cell::new(false);
        let controlled_cancel = || {
            if !checked.replace(true) { return false; }
            // These fixtures refuse before HTTP. The second check is the real
            // routed entry's final result projection: hold it until the real
            // adapter watchdog retires this attempt, then return its signal.
            let deadline = Instant::now() + BOUND;
            while !should_cancel() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(1));
            }
            assert!(should_cancel(), "actual adapter watchdog must fire");
            assert!(!attempt.is_current());
            if self.cancel_user { self.cancellation.cancel("explicit user cancellation"); }
            true
        };
        run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps(
            RoutedProvider::Fabushi, &[],
            &mut RoutedProviderOptions {
                data_dir: self.root, typed_replay: None, fabushi_model: Some(MODEL),
                fabushi_request_binding: None, cursor_auth: None,
                tools: &[], mcp_server_url: None,
                execute_tool: &mut |_, _, _| panic!("refused tool execution"),
                on_text_delta, on_partial_tool_call: None, should_cancel: &controlled_cancel,
            },
            None, on_checkpoint, &mut |_| panic!("refused usage"), 8, None,
        )
    }
}

fn exercise_routed_watchdog(root: &Path, cancel_user: bool) -> Value {
    let adapter = ProductionTurnRunShellAdapter {
        policy: StreamAttemptPolicy {
            first_output_timeout: Duration::from_millis(25), max_attempts: 3,
            initial_backoff: Duration::ZERO, max_backoff: Duration::ZERO, max_retry_after: Duration::ZERO,
        },
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let cancellation = RoutedProviderCancellation::default();
    let mut executor = RoutedRefusalExecutor {
        root, cancellation: cancellation.clone(), cancel_user, attempts: 0,
    };
    let mut reports = Vec::new();
    let error = adapter.run_with_retry_reporting(
        &cancellation, &RefusalCheckpointStore, &mut executor,
        &mut |_, _| panic!("refused output"), &mut |_| panic!("permanent failure retried"),
        &mut |report| reports.push(report.clone()),
    ).expect_err("real routed refusal must survive watchdog");
    assert_eq!(executor.attempts, 1);
    let (kind, error_type) = match &error {
        ProviderSessionError::Authentication(_) => ("authentication", "AuthenticationError"),
        ProviderSessionError::Configuration(_) => ("configuration", "ConfigurationError"),
        ProviderSessionError::Cancelled(_) if cancel_user => ("cancelled", "CancelledError"),
        _ => panic!("lost typed failure: {error:?}"),
    };
    if cancel_user {
        assert_eq!(kind, "cancelled");
        assert!(reports.is_empty());
    } else {
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
        assert_eq!(reports[0].error_type, error_type);
        assert_eq!(reports[0].error_code, "SAND-E0405");
        assert!(reports[0].watchdog_expired);
        assert!(!reports[0].resume_from_checkpoint);
    }
    json!({"kind":kind, "message":error.to_string(), "debug":format!("{error:?}"),
        "deltas":[], "tools":0, "checkpoints":0, "usages":0})
}

struct RoutedHttpErrorExecutor<'a> {
    root: &'a Path,
    attempts: usize,
    tools: usize,
    checkpoints: usize,
    usages: usize,
}

impl RoutedProviderAttemptExecutor for RoutedHttpErrorExecutor<'_> {
    fn run_attempt(
        &mut self,
        _: &RoutedProviderAttemptControl,
        resume_from: Option<&RoutedProviderCheckpoint>,
        on_text_delta: &mut dyn FnMut(&str, &str),
        on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        self.attempts += 1;
        assert!(resume_from.is_none());
        let tools = &mut self.tools;
        let checkpoints = &mut self.checkpoints;
        let usages = &mut self.usages;
        run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps(
            RoutedProvider::Fabushi,
            &[ProviderMessage { role: "user".into(), content: "Keep exact body and model.".into() }],
            &mut RoutedProviderOptions {
                data_dir: self.root, typed_replay: None, fabushi_model: Some(MODEL),
                fabushi_request_binding: None, cursor_auth: None,
                tools: &[], mcp_server_url: None,
                execute_tool: &mut |_, _, _| { *tools += 1; Ok(json!({})) },
                on_text_delta, on_partial_tool_call: None, should_cancel,
            },
            None,
            &mut |checkpoint| { *checkpoints += 1; on_checkpoint(checkpoint) },
            &mut |_| { *usages += 1; },
            8, None,
        )
    }
}

fn exercise_routed_http_error(root: &Path) -> Value {
    let adapter = ProductionTurnRunShellAdapter {
        policy: StreamAttemptPolicy {
            first_output_timeout: BOUND, max_attempts: 2,
            initial_backoff: Duration::ZERO, max_backoff: Duration::ZERO, max_retry_after: Duration::ZERO,
        },
        watchdog_poll_interval: Duration::from_millis(1),
    };
    let mut executor = RoutedHttpErrorExecutor { root, attempts: 0, tools: 0, checkpoints: 0, usages: 0 };
    let mut deltas = Vec::new();
    let mut retries = 0;
    let mut reports = Vec::new();
    let error = adapter.run_with_retry_reporting(
        &RoutedProviderCancellation::default(), &RefusalCheckpointStore, &mut executor,
        &mut |delta, _| deltas.push(delta.to_string()),
        &mut |_| { retries += 1; },
        &mut |report| reports.push(json!({
            "outcome":format!("{:?}", report.outcome), "attempt":report.attempt,
            "error_type":report.error_type, "error_code":report.error_code,
            "cause":report.cause, "error":report.error,
            "watchdog_expired":report.watchdog_expired,
            "resume_from_checkpoint":report.resume_from_checkpoint,
        })),
    ).expect_err("real HTTP fixture must return a provider error");
    let mut observed = provider_observation(
        Err(error), deltas, executor.tools, executor.checkpoints, executor.usages,
    );
    observed["attempts"] = json!(executor.attempts);
    observed["retries"] = json!(retries);
    observed["reports"] = json!(reports);
    observed
}

fn assert_observation(observed: &Value, expected: &str) {
    let rendered = observed.to_string();
    for secret in [TOKEN_A, TOKEN_B, REFRESH_SECRET] {
        assert!(!rendered.contains(secret), "provider diagnostics must redact credentials");
    }
    assert!(!rendered.contains("expiresAtMs"), "do not copy credential documents into diagnostics");
    assert_eq!(observed["kind"], expected, "typed provider result kind; full sanitized observation is in the child receipt");
    assert_eq!(observed["tools"], 0);
    assert_eq!(observed["checkpoints"], 0);
    if expected == "success" {
        assert!(observed["message"] == "done", "success text must be done");
        assert!(observed["deltas"] == json!(["done"]), "success delta must be done");
        assert_eq!(observed["usages"], 1);
    } else {
        assert!(observed["deltas"] == json!([]), "refusal must not publish deltas");
        assert_eq!(observed["usages"], 0);
    }
}

fn contains_private_diagnostic(text: &str) -> bool {
    [TOKEN_A, TOKEN_B, REFRESH_SECRET, "expiresAtMs", "native-http-error-body"]
        .iter().any(|marker| text.contains(marker))
}

fn sanitized_text(text: &str) -> String {
    if contains_private_diagnostic(text) {
        "<credential or response-body diagnostic omitted>".into()
    } else {
        text.into()
    }
}

fn sanitized_observation(value: &Value) -> Value {
    match value {
        Value::String(text) => Value::String(sanitized_text(text)),
        Value::Array(values) => Value::Array(values.iter().map(sanitized_observation).collect()),
        Value::Object(values) => Value::Object(values.iter().map(|(key, value)| {
            (key.clone(), sanitized_observation(value))
        }).collect()),
        value => value.clone(),
    }
}

fn write_child_receipt(root: &Path, receipt: &Value) {
    write_private(&root.join("receipt.json"), &receipt.to_string()).expect("atomically record child execution phase");
}

fn record_child_observation(root: &Path, receipt: &mut Value, field: &str, observed: &Value) {
    let sanitized = sanitized_observation(observed);
    let redacted = sanitized != *observed || receipt["diagnostics_redacted"] == true;
    receipt[field] = sanitized;
    receipt["diagnostics_redacted"] = json!(redacted);
    receipt["phase"] = json!(format!("{field}-observed"));
    write_child_receipt(root, receipt);
}

fn read_child_output(path: &Path, stage: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| fixture_io_error(stage, error))
}

fn child_phase_evidence(receipt: &Value) -> Value {
    // Project only typed observations and counts from the single atomic receipt.
    let observed = &receipt[receipt["observation_field"].as_str().unwrap_or("")];
    json!({
        "case": receipt["case"], "control": receipt["control"],
        "expected": receipt["phase_expected"], "phase": receipt["phase"],
        "observation": receipt["observation_field"], "kind": observed["kind"],
        "deltas": observed["deltas"].as_array().map(Vec::len),
        "tools": observed["tools"], "checkpoints": observed["checkpoints"], "usages": observed["usages"],
        "completed": receipt["completed"], "diagnosticsRedacted": receipt["diagnostics_redacted"],
    })
}

fn child_panic_location(output: &Result<String, String>) -> Option<Value> {
    // Extract only numeric source location; raw redirected output is never
    // part of parent diagnostics, even when it contains no known sentinel.
    output.as_ref().ok().and_then(|output| {
        output.lines().find_map(|line| {
            let location = [
                "panicked at tests/fabushi_responses_credential_context_contract.rs:",
                "panicked at tests/fabushi_responses_credential_context_contract/automatic_attempt_binding.rs:",
            ].into_iter().find_map(|prefix| line.split_once(prefix).map(|(_, location)| location))?;
            let mut numbers = location.split(':');
            let line = numbers.next()?.parse::<u64>().ok()?;
            let column = numbers.next()?.parse::<u64>().ok()?;
            Some(json!({"line": line, "column": column}))
        })
    })
}

fn run_child(
    root_owner: &TempRoot, server: &mut Loopback, endpoint: Option<String>, credential_path: &Path,
    expected: &str, new_transport: bool, control: &str,
) -> (Value, Vec<CapturedRequest>) {
    let root = root_owner.0.as_path();
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout = root.join("child.stdout");
    let stderr = root.join("child.stderr");
    let endpoint = endpoint.unwrap_or_else(|| server.endpoint.clone());
    let mut child_cleanup = None;
    // Spawn, wait and timeout errors are values until the owned server is joined.
    let status = (|| -> Result<ExitStatus, String> {
        let executable = std::env::current_exe().map_err(|error| fixture_io_error("child-executable", error))?;
        let mut child = OwnedChild::new(Command::new(executable)
            .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
            .env(CHILD_ROOT, root).env(CHILD_NONCE, &nonce).env(CHILD_EXPECT, expected)
            .env(CHILD_NEW, if new_transport { "1" } else { "0" })
            .env(CHILD_CONTROL, control).env(CHILD_CASE, &server.case).env(CHILD_PHASE, &server.phase)
            .env("FABUSHI_RESPONSES_URL", endpoint)
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", credential_path)
            .env("NO_PROXY", "*").env("no_proxy", "*")
            .env_remove("FABUSHI_RESPONSES_MODEL")
            .env_remove("SAND_AGENT_MOCK_RESPONSE")
            .env_remove("SAND_PACKAGED").env_remove("FABUSHI_PRODUCT_MODE")
            .stdout(fs::File::create(&stdout).map_err(|error| fixture_io_error("child-stdout-create", error))?)
            .stderr(fs::File::create(&stderr).map_err(|error| fixture_io_error("child-stderr-create", error))?)
            .spawn().map_err(|error| fixture_io_error("child-spawn", error))?, root_owner);
        eprintln!("[credential-fixture] child_pid={} expected={expected} control={control:?} new_transport={new_transport} case={} phase={}",
            child.0.id(), server.case, server.phase);
        let result = child.wait();
        child_cleanup = child.1.clone();
        result
    })();
    let report = server.finish();
    let receipt = fs::read(root.join("receipt.json"))
        .map_err(|error| fixture_io_error("child-receipt-read", error))
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).map_err(|error| format!(
            "child receipt JSON: category={:?}, line={}, column={}",
            error.classify(), error.line(), error.column(),
        )));
    let stdout = read_child_output(&stdout, "child-stdout-read");
    let stderr = read_child_output(&stderr, "child-stderr-read");
    let evidence = json!({
        "case": server.case, "scenario_phase": server.phase, "control": control, "expected": expected,
        "label": server.label, "server": report.evidence(),
        "child": receipt.as_ref().map(child_phase_evidence).unwrap_or_else(|_| json!({"phase":"unavailable"})),
        "childPanic": child_panic_location(&stderr),
        "childResult": format!("{status:?}"), "childCleanup": child_cleanup,
        "stdout": stdout.as_ref().map(String::len), "stderr": stderr.as_ref().map(String::len),
        "receipt": receipt.as_ref().map(|value| json!({
            "nonceMatches": value["nonce"] == nonce,
            "caseMatches": value["case"] == server.case,
            "controlMatches": value["control"] == control,
            "scenarioPhaseMatches": value["scenario_phase"] == server.phase,
        })),
    });
    eprintln!("credential-context evidence: {evidence}");
    let diagnostics = format!("evidence={evidence} progress={:?}", report.progress);
    assert!(status.as_ref().is_ok_and(|status| status.success()) && report.result.is_ok()
        && stdout.is_ok() && stderr.is_ok() && receipt.is_ok(), "{diagnostics}");
    let receipt = receipt.expect("checked receipt");
    assert!(receipt["nonce"] == nonce, "a zero-test child must not satisfy this oracle");
    assert_eq!(receipt["case"], server.case);
    assert_eq!(receipt["control"], control, "child receipt control");
    assert_eq!(receipt["expected"], expected, "child receipt initial expected kind");
    assert_eq!(receipt["scenario_phase"], server.phase);
    assert_eq!(receipt["completed"], true, "{diagnostics}");
    assert_eq!(receipt["diagnostics_redacted"], false, "sanitization is not permission to pass: {diagnostics}");
    assert_observation(&receipt["first"], expected);
    if new_transport { assert_observation(&receipt["new_transport"], "success"); }
    (receipt, report.requests)
}

fn assert_native_http_error_projection() {
    let echoed = format!("native-http-error-body: {TOKEN_A} {TOKEN_B} {REFRESH_SECRET}");
    let refusal = json!({"error": {
        "code":"misalignment_policy_violation", "message":echoed,
        "misalignment": {
            "detailed_explanation":format!("explanation {echoed}"),
            "steer":{"message":format!("continuation {echoed}")},
            "error_type":format!("error-type {echoed}"),
        },
    }}).to_string();
    let clean_refusal = json!({"error": {
        "code":"misalignment_policy_violation",
        "misalignment": {
            "detailed_explanation":"native-http-error-body: This request requires review.",
            "steer":{"message":"Continue only the requested task."},
            "error_type":"policy_review",
        },
    }}).to_string();
    let cases = [
        ("generic-403", "403 Forbidden", echoed.clone(), "transport", false),
        ("generic-500", "500 Internal Server Error", json!({"error": {
            "code":"server_error", "message":echoed,
            "accessToken":TOKEN_B, "refreshToken":REFRESH_SECRET,
        }}).to_string(), "transport", true),
        ("secret-refusal-403", "403 Forbidden", refusal.clone(), "provider-refusal", false),
        ("secret-refusal-500", "500 Internal Server Error", refusal, "provider-refusal", false),
        ("clean-refusal-403", "403 Forbidden", clean_refusal, "provider-refusal", false),
    ];
    for (case, status, body, kind, retryable) in cases {
        for rotate in [false, true] {
            let root = TempRoot::new();
            let path = root.0.join("credential.json");
            let mut server = Loopback::http_error(&path, case, status, body.clone(), rotate);
            let (receipt, requests) = run_child(&root, &mut server, None, &path, kind, false, "http-error");
            let observed = &receipt["first"];
            let attempts = if retryable { 2 } else { 1 };
            assert_eq!(observed["attempts"], attempts);
            assert_eq!(observed["retries"], if retryable { 1 } else { 0 });
            assert!(!observed.to_string().contains("native-http-error-body"));
            let diagnostic = format!("Responses request failed ({status}).");
            if kind == "transport" {
                assert_eq!(observed["refusal"], Value::Null);
                assert_eq!(observed["message"], diagnostic);
                assert_eq!(observed["debug"], format!("Transport({diagnostic:?})"));
            } else {
                let explanation = format!("Fabushi Responses request refused ({status}).");
                assert_eq!(observed["refusal"], json!({
                    "category":"misalignment", "explanation":explanation,
                    "continuation_message":null, "error_type":null,
                }));
                let projection = format!(
                    "CodexProviderRefusal {{ category: \"misalignment\", explanation: Some({explanation:?}), continuation_message: None, error_type: None }}",
                );
                assert_eq!(observed["message"], format!("provider refusal: {projection}"));
                assert_eq!(observed["debug"], format!("ProviderRefusal({projection})"));
            }
            let reports = observed["reports"].as_array().unwrap();
            assert_eq!(reports.len(), if retryable { 2 } else { 0 });
            for (index, report) in reports.iter().enumerate() {
                assert_eq!(report["outcome"], if index == 0 { "Retried" } else { "Exhausted" });
                assert_eq!(report["attempt"], index + 1);
                assert_eq!(report["error_type"], "TransportError");
                assert_eq!(report["error_code"], "SAND-E0406");
                assert_eq!(report["cause"], diagnostic);
                assert_eq!(report["error"], diagnostic);
                assert_eq!(report["watchdog_expired"], false);
                assert_eq!(report["resume_from_checkpoint"], false);
            }
            assert_eq!(requests.len(), attempts + if rotate { 1 } else { 0 });
            for (index, request) in requests.iter().enumerate() {
                assert_eq!(request.response_status, if rotate && index == 0 { "401 Unauthorized" } else { status });
                assert!(request.authorization == format!("Bearer {}", if rotate && index > 0 { TOKEN_B } else { TOKEN_A }), "HTTP error bearer identity");
                assert_eq!(request.request_line, format!("POST {REQUEST_PATH} HTTP/1.1"));
                assert_eq!(request.host, server.origin.strip_prefix("http://").unwrap());
                assert!(request.body == requests[0].body, "HTTP error retries must retain request/model bytes");
                let request_body: Value = serde_json::from_slice(&request.body).unwrap();
                assert!(request_body["model"] == MODEL, "HTTP error request model");
                let raw = String::from_utf8_lossy(&request.body);
                for secret in [TOKEN_A, TOKEN_B, REFRESH_SECRET, "accessToken", "refreshToken"] {
                    assert!(!raw.contains(secret));
                }
            }
        }
    }
}

#[test]
fn fabushi_responses_bind_refresh_to_the_original_credential_context() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        let control = std::env::var(CHILD_CONTROL).unwrap();
        let expected = std::env::var(CHILD_EXPECT).unwrap();
        let mut receipt = json!({
            "nonce":std::env::var(CHILD_NONCE).unwrap(),
            "case":std::env::var(CHILD_CASE).unwrap(),
            "control":control, "expected":expected, "phase_expected":expected,
            "scenario_phase":std::env::var(CHILD_PHASE).unwrap(),
            "phase":"first-running", "observation_field":"first", "completed":false, "diagnostics_redacted":false,
            "first":null, "new_transport":null,
        });
        write_child_receipt(&root, &receipt);
        let first = match control.as_str() {
            "watchdog" => exercise_routed_watchdog(&root, false),
            "watchdog-user" => exercise_routed_watchdog(&root, true),
            "direct-user" => exercise_provider(&root, true),
            "http-error" => exercise_routed_http_error(&root),
            "" => exercise_provider(&root, false),
            _ => panic!("unknown child control"),
        };
        record_child_observation(&root, &mut receipt, "first", &first);
        assert_observation(&first, &expected);
        if std::env::var(CHILD_NEW).unwrap() == "1" {
            receipt["phase"] = json!("new-transport-running");
            receipt["observation_field"] = json!("new_transport");
            receipt["phase_expected"] = json!("success");
            write_child_receipt(&root, &receipt);
            let later = exercise_provider(&root, false);
            record_child_observation(&root, &mut receipt, "new_transport", &later);
            assert_observation(&later, "success");
        }
        assert_eq!(receipt["diagnostics_redacted"], false, "credential/HTTP body diagnostic was redacted from the receipt");
        receipt["phase"] = json!("complete");
        receipt["completed"] = json!(true);
        write_child_receipt(&root, &receipt);
        return;
    }

    let mut cases = vec![
        "rotate", "numeric-to-string", "canonical-issuer", "second-401",
        "user", "user-not-numeric-normalized", "user-whitespace", "device", "session", "issuer",
        "unsupported-issuer", "malformed-issuer", "version", "malformed-user",
        "blank-user", "blank-device", "blank-session", "expired", "expiry-leeway", "malformed",
        "secret-type-error", "unknown-refresh-field", "token-only", "missing-file", "directory",
        "missing-version", "missing-accessToken", "missing-expiresAtMs",
        "missing-userId", "missing-deviceId", "missing-sessionId",
    ];
    #[cfg(unix)]
    cases.extend(["public-file", "symlink"]);
    for case in &cases {
        let root = TempRoot::new();
        let path = root.0.join("credential.json");
        let mut server = Loopback::new(&path, case, false);
        let success = matches!(*case, "rotate" | "numeric-to-string" | "canonical-issuer");
        let new_transport = matches!(*case, "user" | "device" | "session");
        let (_, requests) = run_child(&root, &mut server, None, &path, if success { "success" } else { "authentication" }, new_transport, "");
        let expected = if success || *case == "second-401" || new_transport { 2 } else { 1 };
        assert_eq!(requests.len(), expected, "HTTP request count for {case}");
        assert!(requests[0].authorization == format!("Bearer {TOKEN_A}"), "initial bearer identity");
        for request in &requests {
            assert_eq!(request.request_line, format!("POST {REQUEST_PATH} HTTP/1.1"));
            assert_eq!(request.host, server.origin.strip_prefix("http://").unwrap());
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert!(body["model"] == MODEL, "credential request model");
            assert!(!String::from_utf8_lossy(&request.body).contains("accessToken"));
            assert!(!String::from_utf8_lossy(&request.body).contains(TOKEN_A));
            assert!(!String::from_utf8_lossy(&request.body).contains(TOKEN_B));
        }
        if requests.len() == 2 {
            assert!(requests[1].authorization == format!("Bearer {TOKEN_B}"), "replacement bearer identity");
            assert!(requests[0].body == requests[1].body, "request/model bytes must survive rotation: {case}");
        }
    }

    // Invalid initial images must fail before the same observable server sees
    // any HTTP. Keep missing issuer out: Auth intentionally supports its legacy
    // production-origin default, covered by the existing Auth contract.
    for case in cases.iter().filter(|case| !matches!(**case,
        "rotate" | "numeric-to-string" | "canonical-issuer" | "second-401"
            | "user" | "user-not-numeric-normalized" | "user-whitespace" | "device" | "session"
    )) {
        let root = TempRoot::new();
        let path = root.0.join("credential.json");
        let mut server = Loopback::new(&path, case, true);
        let (_, requests) = run_child(&root, &mut server, None, &path, "authentication", false, "");
        assert_eq!(requests.len(), 0, "initial credential refusal sent HTTP: {case}");
    }

    let root = TempRoot::new();
    let path = root.0.join("credential.json");
    let mut server = Loopback::new(&path, "rotate", false);
    server.phase = "invalid-endpoint".into();
    let endpoint = format!("{}?redirect=forbidden", server.endpoint);
    let (_, requests) = run_child(&root, &mut server, Some(endpoint), &path, "configuration", false, "");
    assert_eq!(requests.len(), 0, "invalid endpoint must fail before HTTP");

    for permanent in ["authentication", "configuration"] {
        for control in ["watchdog", "watchdog-user", "direct-user"] {
            let root = TempRoot::new();
            let path = root.0.join("credential.json");
            let mut server = Loopback::new(&path, "missing-file", true);
            server.case = permanent.into();
            server.phase = control.into();
            let endpoint = if permanent == "configuration" {
                format!("{}?invalid=true", server.endpoint)
            } else { server.endpoint.clone() };
            let expected = if control == "watchdog" { permanent } else { "cancelled" };
            let (_, requests) = run_child(&root, &mut server, Some(endpoint), &path, expected, false, control);
            assert_eq!(requests.len(), 0, "routed refusal control sent HTTP");
        }
    }

    assert_native_http_error_projection();
}

#[test]
fn credential_http_fixture_reads_a_delayed_nonblocking_connection() {
    let root = TempRoot::new();
    let path = root.0.join("credential.json");
    let (entered, reader_entry) = mpsc::sync_channel(1);
    let mut server = Loopback::start(&path, "rotate", false, false, None, Some(entered), None);
    server.case = "delayed-request".into();
    server.phase = "explicit-nonblocking-accepted-stream".into();
    let body = json!({"model":MODEL, "input":[]}).to_string();
    let client_result = (|| -> Result<Vec<u8>, String> {
        let address = server.origin.strip_prefix("http://").expect("fixture origin")
            .parse().map_err(|_| "invalid fixture address")?;
        let mut stream = TcpStream::connect_timeout(&address, BOUND)
            .map_err(|error| fixture_io_error("control-connect", error))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| fixture_io_error("control-read-timeout", error))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| fixture_io_error("control-write-timeout", error))?;
        let observer = reader_entry.recv_timeout(BOUND).map_err(|error| format!("control-reader-entry: {error:?}"))?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            // The clone shares the accepted socket's file status flags. No
            // request bytes are released until its actual mode is observed.
            let flags = unsafe { libc::fcntl(observer.as_raw_fd(), libc::F_GETFL) };
            if flags < 0 { return Err(fixture_io_error("control-reader-flags", io::Error::last_os_error())); }
            if flags & libc::O_NONBLOCK != 0 { return Err("shared request reader retained O_NONBLOCK".into()); }
        }
        // This observer must not keep the connection alive during read_to_end.
        drop(observer);
        // No request bytes exist before the shared reader has entered. This
        // reaches the same parser and response writer used by credential cases.
        let request = format!(
            "POST {REQUEST_PATH} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {TOKEN_A}\r\nContent-Length: {}\r\n\r\n{body}",
            server.origin.strip_prefix("http://").unwrap(), body.len(),
        );
        stream.write_all(request.as_bytes()).map_err(|error| fixture_io_error("control-request-write", error))?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).map_err(|error| fixture_io_error("control-response-read", error))?;
        Ok(response)
    })();
    let report = server.finish();
    assert!(client_result.is_ok() && report.result.is_ok(),
        "case={} phase={} client={} server={:?} progress={:?}", server.case, server.phase,
        client_result.as_ref().err().map(String::as_str).unwrap_or("complete"), report.result, report.progress);
    let response = client_result.expect("checked control response");
    assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"), "control response status");
    assert!(response.ends_with(b"data: [DONE]\n\n"), "control response completed");
    assert_eq!(report.requests.len(), 1);
    assert!(report.requests[0].body.as_slice() == body.as_bytes(), "control preserves complete request bytes");
    assert_eq!(report.requests[0].response_status, "200 OK");
    assert!(report.progress.iter().any(|event| event.ends_with("control-nonblocking-empty")));
    assert!(report.progress.iter().any(|event| event.ends_with("response-written")));
}

#[cfg(unix)]
fn assert_socket_nonblocking(stream: &TcpStream, expected: bool) {
    use std::os::fd::AsRawFd;
    // F_GETFL only inspects the flags of this live, owned socket descriptor.
    let flags = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
    assert_ne!(flags, -1, "inspect fixture socket flags");
    assert_eq!(flags & libc::O_NONBLOCK != 0, expected, "fixture socket mode");
}

#[test]
fn credential_fixture_reads_split_http_after_clearing_nonblocking() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind split-request control");
    listener.set_nonblocking(true).expect("pollable control listener");
    let mut client = TcpStream::connect_timeout(&listener.local_addr().unwrap(), BOUND)
        .expect("connect split-request control");
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    client.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
    let deadline = Instant::now() + BOUND;
    let (mut accepted, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "split-request accept deadline");
                thread::yield_now();
            }
            Err(error) => panic!("{}", fixture_io_error("control accept", error)),
        }
    };
    // Establish the same precondition on every platform, independent of
    // whether that platform inherits listener flags at accept time.
    accepted.set_nonblocking(true).unwrap();
    #[cfg(unix)]
    assert_socket_nonblocking(&accepted, true);

    let (progress_tx, progress_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let (client_result, worker_result) = thread::scope(|scope| {
        let worker = scope.spawn(move || {
            let result = read_request_with_progress(&mut accepted, |stream, phase| {
                if phase == "io-ready" {
                    #[cfg(unix)]
                    assert_socket_nonblocking(stream, false);
                    assert_eq!(stream.read_timeout().unwrap(), Some(Duration::from_secs(5)));
                    assert_eq!(stream.write_timeout().unwrap(), Some(Duration::from_secs(5)));
                }
                progress_tx.send(phase).map_err(|_| "control reader progress receiver closed".into())
            }).and_then(|request| {
                accepted.write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .map_err(|error| fixture_io_error("control response", error))?;
                Ok(request)
            });
            let _ = result_tx.send(result);
        });
        // Keep all client predicates as results until the owned worker has
        // joined, including early failure before a complete request is sent.
        let client_result = (|| -> Result<(CapturedRequest, String), String> {
            let await_phase = |expected| -> Result<(), String> {
                let observed = progress_rx.recv_timeout(BOUND)
                    .map_err(|error| format!("split-reader-progress: {error:?}"))?;
                if observed != expected {
                    return Err(format!("split reader phase: expected={expected}, observed={observed}"));
                }
                Ok(())
            };
            await_phase("io-ready")?;
            client.write_all(b"POST /split-request HTTP/1.1\r\nHost: 127.")
                .map_err(|error| fixture_io_error("control partial header", error))?;
            await_phase("request-line")?;
            client.write_all(b"0.0.1\r\nContent-Length: 7\r\nConnection: close\r\n\r\n")
                .map_err(|error| fixture_io_error("control complete headers", error))?;
            await_phase("headers")?;
            // Complete headers must be parsed while no body has been sent.
            if !matches!(result_rx.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                return Err("split reader completed before body release".into());
            }
            client.write_all(b"pay").map_err(|error| fixture_io_error("control body prefix", error))?;
            client.write_all(b"load").map_err(|error| fixture_io_error("control body suffix", error))?;
            let captured = result_rx.recv_timeout(BOUND)
                .map_err(|error| format!("bounded control result: {error:?}"))??;
            let mut response = String::new();
            client.read_to_string(&mut response)
                .map_err(|error| fixture_io_error("bounded control response", error))?;
            Ok((captured, response))
        })();
        drop(client);
        let worker_result = worker.join().map_err(|_| "split-request worker panicked; payload omitted".to_string());
        (client_result, worker_result)
    });
    assert!(client_result.is_ok() && worker_result.is_ok(),
        "split-request client={} worker={worker_result:?}",
        client_result.as_ref().err().map(String::as_str).unwrap_or("complete"));
    let (captured, response) = client_result.expect("checked split-request result");
    assert_eq!(captured.request_line, "POST /split-request HTTP/1.1");
    assert_eq!(captured.host, "127.0.0.1");
    assert!(captured.authorization.is_empty());
    assert_eq!(captured.body.as_slice(), b"payload");
    assert_eq!(response, "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
}

#[test]
fn credential_fixture_root_retention_requires_actual_child_exit() {
    const NAME: &str = "credential_fixture_root_retention_requires_actual_child_exit";
    const ROOT: &str = "FABUSHI_FIXTURE_CUSTODY_ROOT";
    const NONCE: &str = "FABUSHI_FIXTURE_CUSTODY_NONCE";
    const MODE: &str = "FABUSHI_FIXTURE_CUSTODY_MODE";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        let mode = std::env::var(MODE).expect("custody child mode");
        write_private(&root.join("ready.json"), &json!({
            "nonce":std::env::var(NONCE).expect("custody child nonce"),
            "pid":std::process::id(), "mode":mode,
        }).to_string()).expect("publish actual child readiness");
        if mode == "exited" { return; }
        assert!(mode == "live", "known custody control mode");
        let deadline = Instant::now() + BOUND;
        while Instant::now() < deadline { thread::sleep(Duration::from_millis(5)); }
        panic!("live custody control child exceeded its parent-owned deadline");
    }

    for mode in ["live", "exited"] {
        let mut root = Some(TempRoot::new());
        let path = root.as_ref().unwrap().0.clone();
        let outstanding = Arc::clone(&root.as_ref().unwrap().1);
        let nonce = uuid::Uuid::new_v4().to_string();
        let mut child = OwnedChild::new(Command::new(std::env::current_exe().expect("custody test executable"))
            .args(["--exact", NAME, "--nocapture", "--test-threads=1"])
            .env(ROOT, &path).env(NONCE, &nonce).env(MODE, mode)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn().expect("spawn actual custody control child"), root.as_ref().unwrap());
        let observation = (|| -> Result<(), String> {
            let deadline = Instant::now() + BOUND;
            let ready: Value = loop {
                match fs::read(path.join("ready.json")) {
                    Ok(raw) => break serde_json::from_slice(&raw).map_err(|_| "invalid custody readiness JSON")?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound && Instant::now() < deadline => {}
                    Err(error) => return Err(fixture_io_error("custody-readiness", error)),
                }
                thread::sleep(Duration::from_millis(5));
            };
            if ready["nonce"] != nonce || ready["pid"] != child.0.id() || ready["mode"] != mode {
                return Err("custody readiness must match the owned nonce, PID and mode".into());
            }
            if mode == "live" {
                if child.try_wait().map_err(|error| fixture_io_error("custody-live-status", error))?.is_some() {
                    return Err("custody control child exited before root drop".into());
                }
                if outstanding.load(Ordering::Acquire) != 1 {
                    return Err("live owned child must retain one root permission".into());
                }
                drop(root.take());
                if !path.is_dir() { return Err("live owned child's root was removed".into()); }
                if child.try_wait().map_err(|error| fixture_io_error("custody-retained-status", error))?.is_some() {
                    return Err("custody control child must still be live after root drop".into());
                }
            } else {
                let status = child.wait()?;
                if !status.success() { return Err("ordinary custody control child must succeed".into()); }
                if !child.exit_observed() || outstanding.load(Ordering::Acquire) != 0 {
                    return Err("actual exit must release the owned root permission".into());
                }
                drop(root.take());
                if path.exists() { return Err("observed-exit root must be removed normally".into()); }
            }
            Ok(())
        })();
        // Always retire this exact owned child before any manual directory
        // cleanup. A failed control/cleanup never becomes success after reaping.
        let retirement = child.stop_and_reap();
        let exit_observed = child.exit_observed();
        let cleanup_observation = child.1.clone();
        drop(child);
        let remaining = outstanding.load(Ordering::Acquire);
        drop(root.take());
        let cleanup = if remaining == 0 {
            if path.exists() {
                fs::remove_dir_all(&path).map_err(|error| fixture_io_error("custody-control-root-cleanup", error))
            } else { Ok(()) }
        } else {
            Err("owned child retirement unresolved; control root retained".into())
        };
        assert!(observation.is_ok() && retirement.is_ok() && exit_observed && remaining == 0 && cleanup.is_ok(),
            "custody mode={mode} observation={observation:?} retirement={retirement:?} exit_observed={exit_observed} outstanding={remaining} child_cleanup={cleanup_observation:?} root_cleanup={cleanup:?}");
    }
}
