//! Real Responses HTTP and private-file replacement at the Native auth boundary.
//! This proves per-transport binding, not identity pinning across Runner attempts.

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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
const MODEL: &str = "Vendor/Nested/CaseSensitive@Credential-Context";
const TOKEN_A: &str = "credential-context-private-token-a";
const TOKEN_B: &str = "credential-context-private-token-b";
const REFRESH_SECRET: &str = "credential-context-private-refresh-secret";
const BOUND: Duration = Duration::from_secs(20);
const REQUEST_PATH: &str = "/tenant-a/codex/v1/responses";

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("fabushi-credential-context-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).expect("create private test root");
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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

fn write_private(path: &Path, raw: &str) {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(&temporary).expect("create private replacement")
        .write_all(raw.as_bytes()).expect("write complete credential image");
    fs::rename(temporary, path).expect("atomically replace credential image");
}

fn install_image(path: &Path, image: Image) {
    match image {
        Image::Document(document) => write_private(path, &document.to_string()),
        Image::Raw(raw) => write_private(path, &raw),
        Image::Missing => { let _ = fs::remove_file(path); }
        Image::Directory => {
            let _ = fs::remove_file(path);
            fs::create_dir(path).expect("directory negative control");
        }
        #[cfg(unix)]
        Image::Public(document) => {
            use std::os::unix::fs::PermissionsExt;
            write_private(path, &document.to_string());
            fs::set_permissions(path, fs::Permissions::from_mode(0o644)).expect("public-file negative control");
        }
        #[cfg(unix)]
        Image::Symlink(document) => {
            let target = path.with_extension("target");
            write_private(&target, &document.to_string());
            let _ = fs::remove_file(path);
            std::os::unix::fs::symlink(target, path).expect("symlink negative control");
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

fn read_request(stream: &mut TcpStream) -> Result<CapturedRequest, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).map_err(|error| error.to_string())?;
    let mut content_length = None;
    let mut host = String::new();
    let mut authorization = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|error| error.to_string())? == 0 {
            return Err("HTTP headers ended early".into());
        }
        if line == "\r\n" { break; }
        let (name, value) = line.split_once(':').ok_or("invalid header")?;
        match name.to_ascii_lowercase().as_str() {
            "content-length" => content_length = Some(value.trim().parse::<usize>().map_err(|error| error.to_string())?),
            "authorization" => authorization = value.trim().to_string(),
            "host" => host = value.trim().to_string(),
            _ => {}
        }
    }
    let length = content_length.ok_or("missing body length")?;
    if length > 4 * 1024 * 1024 { return Err("oversized fixture request".into()); }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|error| error.to_string())?;
    Ok(CapturedRequest {
        request_line: request_line.trim_end().into(), host, authorization, body,
        response_status: String::new(),
    })
}

struct Loopback {
    endpoint: String,
    origin: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl Loopback {
    fn new(path: &Path, case: &str, initial_refusal: bool) -> Self {
        Self::start(path, case, initial_refusal, true, None)
    }

    fn http_error(path: &Path, status: &'static str, body: String, rotate: bool) -> Self {
        Self::start(path, "rotate", false, rotate, Some((status, body)))
    }

    fn start(
        path: &Path,
        case: &str,
        initial_refusal: bool,
        rotate: bool,
        http_error: Option<(&'static str, String)>,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind observable Responses fixture");
        listener.set_nonblocking(true).expect("pollable fixture");
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let next = if initial_refusal {
            install_image(path, replacement(case, &origin));
            None
        } else {
            install_image(path, Image::Document(credential(&origin, TOKEN_A)));
            rotate.then(|| replacement(case, &origin))
        };
        let repeated_401 = case == "second-401";
        let path = path.to_path_buf();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let capture = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + BOUND * 2;
            let mut next = next;
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
                    Err(error) => return Err(error.to_string()),
                };
                let mut request = read_request(&mut stream)?;
                let first_401 = if let Some(image) = next.take() {
                    install_image(&path, image);
                    true
                } else { false };
                let (status, content_type, body) = if first_401 || repeated_401 {
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
                let response = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                stream.write_all(response.as_bytes())
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        });
        Self { endpoint: format!("{origin}{REQUEST_PATH}"), origin, requests, stop, worker: Some(worker) }
    }

    fn finish(&mut self) -> Vec<CapturedRequest> {
        self.stop.store(true, Ordering::Release);
        self.worker.take().unwrap().join().expect("fixture thread").expect("fixture protocol");
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for Loopback {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}

struct OwnedChild(Child);

impl OwnedChild {
    fn wait(&mut self) -> ExitStatus {
        let deadline = Instant::now() + BOUND;
        loop {
            if let Some(status) = self.0.try_wait().expect("owned child status") { return status; }
            assert!(Instant::now() < deadline, "credential contract child exceeded its deadline");
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
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
            data_dir: root, typed_replay: None, fabushi_model: Some(MODEL), cursor_auth: None,
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
                data_dir: self.root, typed_replay: None, fabushi_model: Some(MODEL), cursor_auth: None,
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
                data_dir: self.root, typed_replay: None, fabushi_model: Some(MODEL), cursor_auth: None,
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
    assert_eq!(observed["kind"], expected, "typed provider result: {observed}");
    assert_eq!(observed["tools"], 0);
    assert_eq!(observed["checkpoints"], 0);
    if expected == "success" {
        assert_eq!(observed["message"], "done");
        assert_eq!(observed["deltas"], json!(["done"]));
        assert_eq!(observed["usages"], 1);
    } else {
        assert_eq!(observed["deltas"], json!([]));
        assert_eq!(observed["usages"], 0);
    }
    let rendered = observed.to_string();
    for secret in [TOKEN_A, TOKEN_B, REFRESH_SECRET] {
        assert!(!rendered.contains(secret), "provider diagnostics must redact credentials");
    }
    assert!(!rendered.contains("expiresAtMs"), "do not copy credential documents into diagnostics");
}

fn run_child(root: &Path, endpoint: &str, credential_path: &Path, expected: &str, new_transport: bool, control: &str) -> Value {
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout = root.join("child.stdout");
    let stderr = root.join("child.stderr");
    let mut child = OwnedChild(Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
        .env(CHILD_ROOT, root).env(CHILD_NONCE, &nonce).env(CHILD_EXPECT, expected)
        .env(CHILD_NEW, if new_transport { "1" } else { "0" })
        .env(CHILD_CONTROL, control)
        .env("FABUSHI_RESPONSES_URL", endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", credential_path)
        .env("NO_PROXY", "*").env("no_proxy", "*")
        .env_remove("FABUSHI_RESPONSES_MODEL")
        .env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env_remove("SAND_PACKAGED").env_remove("FABUSHI_PRODUCT_MODE")
        .stdout(fs::File::create(&stdout).unwrap()).stderr(fs::File::create(&stderr).unwrap())
        .spawn().expect("spawn isolated provider invocation"));
    let status = child.wait();
    assert!(status.success(), "provider child failed: {status}\n{}\n{}",
        fs::read_to_string(stdout).unwrap(), fs::read_to_string(stderr).unwrap());
    let receipt: Value = serde_json::from_slice(&fs::read(root.join("receipt.json")).expect("child execution receipt")).unwrap();
    assert_eq!(receipt["nonce"], nonce, "a zero-test child must not satisfy this oracle");
    assert_observation(&receipt["first"], expected);
    if new_transport { assert_observation(&receipt["new_transport"], "success"); }
    receipt
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
        ("403 Forbidden", echoed.clone(), "transport", false),
        ("500 Internal Server Error", json!({"error": {
            "code":"server_error", "message":echoed,
            "accessToken":TOKEN_B, "refreshToken":REFRESH_SECRET,
        }}).to_string(), "transport", true),
        ("403 Forbidden", refusal.clone(), "provider-refusal", false),
        ("500 Internal Server Error", refusal, "provider-refusal", false),
        ("403 Forbidden", clean_refusal, "provider-refusal", false),
    ];
    for (status, body, kind, retryable) in cases {
        for rotate in [false, true] {
            let root = TempRoot::new();
            let path = root.0.join("credential.json");
            let mut server = Loopback::http_error(&path, status, body.clone(), rotate);
            let receipt = run_child(&root.0, &server.endpoint, &path, kind, false, "http-error");
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
            let requests = server.finish();
            assert_eq!(requests.len(), attempts + if rotate { 1 } else { 0 });
            for (index, request) in requests.iter().enumerate() {
                assert_eq!(request.response_status, if rotate && index == 0 { "401 Unauthorized" } else { status });
                assert_eq!(request.authorization, format!("Bearer {}", if rotate && index > 0 { TOKEN_B } else { TOKEN_A }));
                assert_eq!(request.request_line, format!("POST {REQUEST_PATH} HTTP/1.1"));
                assert_eq!(request.host, server.origin.strip_prefix("http://").unwrap());
                assert_eq!(request.body, requests[0].body, "HTTP error retries must retain request/model bytes");
                let request_body: Value = serde_json::from_slice(&request.body).unwrap();
                assert_eq!(request_body["model"], MODEL);
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
        let first = match std::env::var(CHILD_CONTROL).unwrap().as_str() {
            "watchdog" => exercise_routed_watchdog(&root, false),
            "watchdog-user" => exercise_routed_watchdog(&root, true),
            "direct-user" => exercise_provider(&root, true),
            "http-error" => exercise_routed_http_error(&root),
            "" => exercise_provider(&root, false),
            _ => panic!("unknown child control"),
        };
        assert_observation(&first, &std::env::var(CHILD_EXPECT).unwrap());
        let later = if std::env::var(CHILD_NEW).unwrap() == "1" {
            let later = exercise_provider(&root, false);
            assert_observation(&later, "success");
            later
        } else { Value::Null };
        fs::write(root.join("receipt.json"), json!({
            "nonce":std::env::var(CHILD_NONCE).unwrap(), "first":first, "new_transport":later
        }).to_string()).unwrap();
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
        run_child(&root.0, &server.endpoint, &path, if success { "success" } else { "authentication" }, new_transport, "");
        let requests = server.finish();
        let expected = if success || *case == "second-401" || new_transport { 2 } else { 1 };
        assert_eq!(requests.len(), expected, "HTTP request count for {case}");
        assert_eq!(requests[0].authorization, format!("Bearer {TOKEN_A}"));
        for request in &requests {
            assert_eq!(request.request_line, format!("POST {REQUEST_PATH} HTTP/1.1"));
            assert_eq!(request.host, server.origin.strip_prefix("http://").unwrap());
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["model"], MODEL);
            assert!(!String::from_utf8_lossy(&request.body).contains("accessToken"));
            assert!(!String::from_utf8_lossy(&request.body).contains(TOKEN_A));
            assert!(!String::from_utf8_lossy(&request.body).contains(TOKEN_B));
        }
        if requests.len() == 2 {
            assert_eq!(requests[1].authorization, format!("Bearer {TOKEN_B}"));
            assert_eq!(requests[0].body, requests[1].body, "request/model bytes must survive rotation: {case}");
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
        run_child(&root.0, &server.endpoint, &path, "authentication", false, "");
        assert_eq!(server.finish().len(), 0, "initial credential refusal sent HTTP: {case}");
    }

    let root = TempRoot::new();
    let path = root.0.join("credential.json");
    let mut server = Loopback::new(&path, "rotate", false);
    run_child(&root.0, &format!("{}?redirect=forbidden", server.endpoint), &path, "configuration", false, "");
    assert_eq!(server.finish().len(), 0, "invalid endpoint must fail before HTTP");

    for permanent in ["authentication", "configuration"] {
        for control in ["watchdog", "watchdog-user", "direct-user"] {
            let root = TempRoot::new();
            let path = root.0.join("credential.json");
            let mut server = Loopback::new(&path, "missing-file", true);
            let endpoint = if permanent == "configuration" {
                format!("{}?invalid=true", server.endpoint)
            } else { server.endpoint.clone() };
            let expected = if control == "watchdog" { permanent } else { "cancelled" };
            run_child(&root.0, &endpoint, &path, expected, false, control);
            assert_eq!(server.finish().len(), 0, "routed refusal control sent HTTP");
        }
    }

    assert_native_http_error_projection();
}
