//! Loopback protocol wiring through the real Settings, Inference and Runner owners.
//! This does not establish real-service compatibility or process-restart checkpoint resume.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use mahayana_host_runtime::agents::agent_messaging::AgentMessageImage;
use mahayana_host_runtime::cursor_backend::SandPrivacyMode;
use mahayana_host_runtime::extensions::auth::auth_service::{
    FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV, HostAuthEnvironment, HostAuthServiceOptions,
};
use mahayana_host_runtime::extensions::auth::extension::start_host_auth_extension_with_options;
use mahayana_host_runtime::extensions::experiments::extension::{
    HostExperimentsExtension, HostExperimentsOptions,
};
use mahayana_host_runtime::extensions::inference::cursor_session::SAND_SUMMARIZATION_MODEL_ID;
use mahayana_host_runtime::extensions::inference::inference_service::{
    InferenceRoute, authorize_routed_provider_request,
};
use mahayana_host_runtime::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
    summarization_session_options,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    FabushiResponsesRequestBinding, ProviderMessage, ProviderSessionError, RoutedProvider,
    RoutedProviderCheckpoint, RoutedToolDefinition, capture_fabushi_responses_request_binding,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolPermissionController,
};
use mahayana_host_runtime::extensions::session::agent_db_serde::EpisodeTurn;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::extensions::transcript::turn_runtime::{
    shape_closing_send_nudge_turn_input, shape_reply_nudge_turn_input,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryOutcome, ProviderRetryReport,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::sand_memory::{
    MEMORY_EPISODE_PREFIX, MEMORY_EPISODE_PROMPT_MARKER, MEMORY_EXTRACTION_PROMPT_MARKER,
};
use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
use mahayana_host_runtime::runner::tools::sand_agent_management_tools::{
    AgentManagementRecord, AgentManagementSink,
};
use mahayana_host_runtime::runner::tools::sand_computer_tool::ComputerToolExposure;
use mahayana_host_runtime::runner::tools::send_message_tool::SendMessageSink;
use mahayana_host_runtime::runner::turn_memory::{
    TurnExchange, TurnMemoryMode, run_turn_memory_with,
};
use mahayana_host_runtime::runner::{ToolEffectCustodyObservation, TurnRunOptions};
use mahayana_host_runtime::runner_production_bridge::{
    ProductionRunnerCompositionHooks, ProductionRunnerCompositionInput,
};
use serde_json::{Value, json};

const MODEL_A: &str = "Vendor/Nested/CaseSensitive@Model-A";
const MODEL_B: &str = "Vendor/Nested/CaseSensitive@Model-B";
const AGENT_ID: &str = "model-identity-agent";
const CHILD_ROOT_ENV: &str = "FABUSHI_MODEL_IDENTITY_CONTRACT_CHILD_ROOT";
const CHILD_NONCE_ENV: &str = "FABUSHI_MODEL_IDENTITY_CONTRACT_CHILD_NONCE";
const TEST_NAME: &str = "persisted_fabushi_model_is_frozen_across_tools_retry_and_summarization";
const BINDING_ROOT_ENV: &str = "FABUSHI_TURN_BINDING_CONTRACT_CHILD_ROOT";
const BINDING_CASE_ENV: &str = "FABUSHI_TURN_BINDING_CONTRACT_CHILD_CASE";
const COMPLETION_FILE: &str = "child-completed.json";
const BINDING_TOKEN_A: &str = "TurnBinding-Account-A-Token-Sentinel";
const BINDING_TOKEN_ROTATED: &str = "TurnBinding-Account-A-Rotated-Token-Sentinel";
const BINDING_TOKEN_B: &str = "TurnBinding-Account-B-Token-Sentinel";

type CheckpointFiles = Vec<(PathBuf, Vec<u8>)>;

struct FixtureDirectory(PathBuf);

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct LoopbackFixture {
    root: FixtureDirectory,
    endpoint: String,
    credential_path: PathBuf,
    requests: Arc<Mutex<Vec<Value>>>,
    checkpoint_files_before_503: Arc<Mutex<Option<CheckpointFiles>>>,
    stop: Arc<AtomicBool>,
    server: Option<JoinHandle<Result<(), String>>>,
}

impl LoopbackFixture {
    fn new() -> Self {
        let root = FixtureDirectory(std::env::temp_dir().join(format!(
            "fabushi-model-identity-production-{}",
            uuid::Uuid::new_v4()
        )));
        fs::create_dir_all(&root.0).expect("create isolated fixture directory");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind Responses loopback");
        listener.set_nonblocking(true).expect("pollable listener");
        let origin = format!("http://{}", listener.local_addr().expect("listener address"));
        let credential_path = root.0.join("credential.json");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options
            .open(&credential_path)
            .expect("create private fixture credential")
            .write_all(
                json!({
                    "version": 1,
                    "issuer": origin,
                    "accessToken": "model-identity-fixture-token",
                    "expiresAtMs": 4_102_444_800_000_u64,
                    "sessionId": "model-identity-session",
                    "deviceId": "model-identity-device",
                    "userId": "model-identity-account"
                })
                .to_string()
                .as_bytes(),
            )
            .expect("write fixture credential");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let checkpoint_files_before_503 = Arc::new(Mutex::new(None));
        let captured_checkpoint_files = Arc::clone(&checkpoint_files_before_503);
        let checkpoint_store = ProductionRoutedProviderCheckpointStore::new(
            &root.0, AGENT_ID, "turn-a",
        );
        let settings_path = root.0.join("settings.json");
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(60);
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    return Err("Responses fixture exceeded its deadline".into());
                }
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => return Err(format!("fixture accept failed: {error}")),
                };
                let request = read_request(&mut stream)?;
                let index = {
                    let mut requests = captured.lock().map_err(|_| "capture lock poisoned")?;
                    let index = requests.len();
                    requests.push(request);
                    index
                };
                if index == 2 {
                    // The third request must follow both durable accepted
                    // boundaries before this server releases the retryable 503.
                    let requests = captured.lock().map_err(|_| "capture lock poisoned")?;
                    let files = assert_durable_provider_checkpoints(
                        &checkpoint_store,
                        Some(requests.as_slice()),
                    );
                    assert_eq!(
                        SettingsService::new(&settings_path)
                            .get_agent_default_model_checked()
                            .expect("read committed default before 503"),
                        Some(bound_model(MODEL_B)),
                        "the first delivery must persist B before the accepted continuation fails"
                    );
                    *captured_checkpoint_files.lock()
                        .map_err(|_| "checkpoint capture lock poisoned")? = Some(files);
                }
                let (status, content_type, body) = scripted_response(index)?;
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .map_err(|error| format!("fixture response failed: {error}"))?;
            }
            Ok(())
        });
        Self {
            root,
            endpoint: format!("{origin}/v1/responses"),
            credential_path,
            requests,
            checkpoint_files_before_503,
            stop,
            server: Some(server),
        }
    }

    fn finish(&mut self) -> (Vec<Value>, Option<CheckpointFiles>, Result<(), String>) {
        self.stop.store(true, Ordering::Release);
        let outcome = self.server
            .take()
            .expect("owned fixture server")
            .join()
            .unwrap_or_else(|_| Err("fixture server panicked".into()));
        let requests = self.requests.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
        let files = self.checkpoint_files_before_503
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        (requests, files, outcome)
    }
}

impl Drop for LoopbackFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
    }
}

#[derive(Clone)]
struct CapturedRequest {
    request_line: String,
    method: String,
    path: String,
    authorization: Option<String>,
    has_chatgpt_account_id: bool,
    raw_body: Vec<u8>,
    json: Value,
}

fn read_captured_request(stream: &mut TcpStream) -> Result<CapturedRequest, String> {
    // Accepted sockets can inherit O_NONBLOCK on macOS. Both directions stay bounded.
    stream.set_nonblocking(false).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut bytes = Vec::new();
    loop {
        if Instant::now() >= deadline {
            return Err("fixture request read exceeded its deadline".into());
        }
        let mut buffer = [0_u8; 8192];
        let count = stream.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("fixture connection closed before the complete request".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("fixture request exceeded its size bound".into());
        }
        let Some(header_end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
            continue;
        };
        let headers = std::str::from_utf8(&bytes[..header_end])
            .map_err(|error| error.to_string())?;
        let request_line = headers.lines().next().ok_or("fixture request has no request line")?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().ok_or("fixture request has no method")?;
        let path = request_parts.next().ok_or("fixture request has no path")?;
        let mut authorization = None;
        let has_chatgpt_account_id = headers.to_ascii_lowercase().contains("chatgpt-account-id:");
        let mut length = None;
        for line in headers.lines().skip(1) {
            let Some((name, value)) = line.split_once(':') else {
                return Err("fixture request has a malformed header".into());
            };
            if name.eq_ignore_ascii_case("authorization") {
                if authorization.replace(value.to_string()).is_some() {
                    return Err("fixture request has duplicate authorization".into());
                }
            } else if name.eq_ignore_ascii_case("content-length") {
                length = Some(value.trim().parse::<usize>()
                    .map_err(|_| "fixture request has invalid content-length")?);
            }
        }
        let length = length.ok_or("fixture request is missing content-length")?;
        if length > 4 * 1024 * 1024 {
            return Err("fixture request content-length exceeded its bound".into());
        }
        let body_start = header_end + 4;
        if bytes.len() >= body_start + length {
            let raw_body = bytes[body_start..body_start + length].to_vec();
            let json = serde_json::from_slice(&raw_body)
                .map_err(|_| "fixture request has invalid JSON")?;
            return Ok(CapturedRequest {
                request_line: request_line.to_string(),
                method: method.to_string(),
                path: path.to_string(),
                authorization,
                has_chatgpt_account_id,
                raw_body,
                json,
            });
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Result<Value, String> {
    let request = read_captured_request(stream)?;
    if request.request_line != "POST /v1/responses HTTP/1.1" {
        return Err("unexpected fixture request method or path".into());
    }
    if request.authorization.as_deref().map(str::to_ascii_lowercase).as_deref()
        != Some(" bearer model-identity-fixture-token")
        || request.has_chatgpt_account_id
    {
        return Err("request did not use the fixture Fabushi credential".into());
    }
    Ok(request.json)
}

fn reasoning_item() -> Value {
    json!({
        "id": "reasoning-model-a",
        "type": "reasoning",
        "encrypted_content": "opaque-reasoning-for-model-a",
        "summary": [{"type": "summary_text", "text": "Prepare both deliveries."}]
    })
}

fn scripted_response(index: usize) -> Result<(&'static str, &'static str, String), String> {
    if index == 2 {
        return Ok((
            "503 Service Unavailable",
            "application/json",
            json!({"error": "fixture failure after the second accepted tool boundary"}).to_string(),
        ));
    }
    let mut events = Vec::new();
    let mut output = Vec::new();
    let delta = match index {
        0 => "working ",
        1 => "",
        3 => "done",
        4 => "next turn",
        5 => "summary on model B",
        _ => return Err(format!("unexpected Responses request {}", index + 1)),
    };
    if !delta.is_empty() {
        events.push(json!({"type": "response.output_text.delta", "delta": delta}));
    }
    if index == 0 {
        output.push(reasoning_item());
    }
    if index < 2 {
        output.push(json!({
            "type": "function_call",
            "name": "SendMessage",
            "call_id": format!("send-{}", index + 1),
            "arguments": json!({
                "type": "text",
                "content": format!("native delivery {}", index + 1)
            }).to_string()
        }));
    }
    events.push(json!({
        "type": "response.completed",
        "response": {
            "id": format!("fixture-response-{}", index + 1),
            "usage": {"input_tokens": 11, "output_tokens": 2},
            "output": output
        }
    }));
    let body = events.iter().map(|event| format!("data: {event}\n\n")).collect();
    Ok(("200 OK", "text/event-stream", body))
}

struct OwnedChild(Child);

impl OwnedChild {
    fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            match self.0.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) if Instant::now() < deadline => {}
                outcome => {
                    let _ = self.0.kill();
                    let _ = self.0.wait();
                    return Err(if outcome.is_err() {
                        "could not observe owned test child"
                    } else {
                        "isolated production owner flow timed out"
                    }.into());
                }
            }
            thread::sleep(Duration::from_millis(10));
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

fn write_completion_receipt(root: &Path, test_name: &str) {
    let nonce = std::env::var(CHILD_NONCE_ENV).expect("child invocation nonce");
    let receipt = json!({
        "nonce": nonce,
        "pid": std::process::id(),
        "test": test_name,
        "completed": true,
    });
    OpenOptions::new().write(true).create_new(true)
        .open(root.join(COMPLETION_FILE)).expect("create completion receipt")
        .write_all(receipt.to_string().as_bytes()).expect("write completion receipt");
}

fn assert_completion_receipt(root: &Path, nonce: &str, pid: u32, test_name: &str) {
    let bytes = fs::read(root.join(COMPLETION_FILE)).expect("child must finish its exact test");
    let receipt: Value = serde_json::from_slice(&bytes).expect("decode completion receipt");
    assert_eq!(receipt, json!({
        "nonce": nonce,
        "pid": pid,
        "test": test_name,
        "completed": true,
    }));
}

struct EmptyBridge;

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected non-native tool dispatch".into()))
    }
}

struct UnusedAgentManagement;

impl AgentManagementSink for UnusedAgentManagement {
    fn self_agent_id(&self) -> &str {
        AGENT_ID
    }

    fn send_to_agent(
        &self,
        _target_id: &str,
        _message: &str,
        _images: &[AgentMessageImage],
        _priority: bool,
    ) -> Result<String, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected agent message".into()))
    }

    fn create_agent(
        &self,
        _name: &str,
        _description: &str,
    ) -> Result<AgentManagementRecord, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected agent creation".into()))
    }

    fn update_agent(
        &self,
        _agent_id: &str,
        _name: Option<&str>,
        _description: Option<&str>,
    ) -> Result<Option<AgentManagementRecord>, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected agent update".into()))
    }
}

fn bound_model(model: &str) -> Value {
    json!({"provider": "fabushi", "modelId": model, "maxMode": true, "parameters": []})
}

struct ChangeDefaultOnFirstDelivery {
    settings: Arc<SettingsService>,
    deliveries: Mutex<Vec<(String, Value)>>,
}

impl SendMessageSink for ChangeDefaultOnFirstDelivery {
    fn send_message(
        &self,
        message: Value,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<Option<String>, ProviderSessionError> {
        assert!(timestamp_ms > 0);
        let mut deliveries = self.deliveries.lock().expect("native deliveries");
        if deliveries.is_empty() {
            let current = self.settings.get_host_settings_snapshot();
            assert_eq!(current.settings["agentDefaultModel"], bound_model(MODEL_A));
            let committed = self
                .settings
                .set_host_settings_if_current(
                    Some(&current.hash),
                    json!({"agentDefaultModel": bound_model(MODEL_B)}),
                )
                .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
            assert_ne!(committed.hash, current.hash);
            assert_eq!(committed.settings["agentDefaultModel"], bound_model(MODEL_B));
        }
        deliveries.push((tool_call_id.to_string(), message));
        Ok(Some(format!("native-message-{}", deliveries.len())))
    }
}

fn assert_accepted_history(input: &[Value], tool_count: usize) {
    assert_eq!(
        input.iter().filter(|item| **item == reasoning_item()).count(),
        1
    );
    assert_eq!(
        input.iter().filter(|item| item["type"] == "function_call").count(),
        tool_count
    );
    let results = input
        .iter()
        .filter(|item| item["type"] == "function_call_output")
        .collect::<Vec<_>>();
    assert_eq!(results.len(), tool_count);
    for (index, result) in results.iter().enumerate() {
        assert_eq!(result["call_id"], format!("send-{}", index + 1));
        let output: Value = serde_json::from_str(
            result["output"].as_str().expect("tool result JSON"),
        )
        .expect("decode native SendMessage result");
        assert_eq!(output["sent"], true);
        assert_eq!(output["messageId"], format!("native-message-{}", index + 1));
        assert!(output["timestampMs"].as_u64().is_some_and(|value| value > 0));
    }
}

fn read_checkpoint_files(store: &ProductionRoutedProviderCheckpointStore) -> CheckpointFiles {
    let mut files = fs::read_dir(store.directory())
        .expect("production provider checkpoint directory")
        .map(|entry| {
            let path = entry.expect("checkpoint entry").path();
            assert_eq!(path.extension().and_then(|value| value.to_str()), Some("json"));
            let bytes = fs::read(&path).expect("read durably written checkpoint");
            (path, bytes)
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn assert_durable_provider_checkpoints(
    store: &ProductionRoutedProviderCheckpointStore,
    accepted_requests: Option<&[Value]>,
) -> CheckpointFiles {
    let files = read_checkpoint_files(store);
    let mut checkpoints = files.iter()
        .map(|(_, bytes)| {
            match serde_json::from_slice::<RoutedProviderCheckpoint>(bytes)
                .expect("decode actual production checkpoint JSON")
            {
                RoutedProviderCheckpoint::Fabushi(checkpoint) => checkpoint,
                other => panic!("wrong persisted checkpoint provider: {:?}", other.provider()),
            }
        })
        .collect::<Vec<_>>();
    checkpoints.sort_by_key(|checkpoint| checkpoint.completed_steps);
    assert_eq!(checkpoints.len(), 2, "retry must not repeat accepted tool boundaries");
    for (index, checkpoint) in checkpoints.iter().enumerate() {
        assert_eq!(checkpoint.model.as_deref(), Some(MODEL_A));
        assert_eq!(checkpoint.completed_steps, index + 1);
        assert_eq!(checkpoint.tool_calls_completed, index + 1);
        assert_eq!(checkpoint.text, "working ");
        assert_accepted_history(&checkpoint.input, index + 1);
        if let Some(requests) = accepted_requests {
            assert_eq!(
                checkpoint.input.as_slice(),
                requests[index + 1]["input"].as_array()
                    .expect("captured accepted continuation input").as_slice(),
                "the persisted checkpoint must contain the complete input sent after its boundary"
            );
        }
    }
    files
}

fn assert_no_provider_checkpoints(store: &ProductionRoutedProviderCheckpointStore) {
    match fs::read_dir(store.directory()) {
        Ok(mut entries) => assert!(
            entries.next().is_none(),
            "a turn without accepted tools must not create a provider checkpoint"
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("could not inspect the later turn's checkpoint directory: {error}"),
    }
}

struct ProductionFixtureOwners {
    settings: Arc<SettingsService>,
    inference: ProductionInferenceExtension,
    provider: RoutedProvider,
    host_composition: HostRunnerComposition,
    delivery_sink: Arc<ChangeDefaultOnFirstDelivery>,
    retry_events: Arc<Mutex<Vec<ProviderRetryEvent>>>,
    retry_reports: Arc<Mutex<Vec<ProviderRetryReport>>>,
}

fn production_fixture_owners(root: &Path) -> ProductionFixtureOwners {
    let settings_path = root.join("settings.json");
    let initial_owner = SettingsService::new(&settings_path);
    let initial = initial_owner.get_host_settings_snapshot();
    let committed = initial_owner
        .set_host_settings_if_current(
            Some(&initial.hash),
            json!({"inferenceProvider": "fabushi", "agentDefaultModel": bound_model(MODEL_A)}),
        )
        .expect("CAS persist provider-bound model A");
    drop(initial_owner);
    let settings = Arc::new(SettingsService::new(&settings_path));
    assert_eq!(settings.get_host_settings_snapshot(), committed);
    assert_eq!(
        settings.get_agent_default_model_checked().expect("checked reopened settings"),
        Some(bound_model(MODEL_A))
    );

    let auth = Arc::new(start_host_auth_extension_with_options(
        HostAuthServiceOptions {
            environment: HostAuthEnvironment::from_pairs([(
                FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV.to_string(),
                root.join("credential.json").to_string_lossy().into_owned(),
            )]),
            backend_url: Some("http://127.0.0.1:9".into()),
            backend: None,
            machine_id: Arc::new(|| Ok("model-identity-device".into())),
            log: Arc::new(|_| {}),
            now_ms: Arc::new(|| 10_000),
        },
        Arc::new(|_| Ok(None)),
    ).expect("construct existing Host auth owner without Cursor credentials"));
    let experiments = Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: false,
        env_gate_overrides: None,
    }));
    let inference = start_production_inference_extension(auth, experiments, Arc::clone(&settings));
    assert_eq!(inference.route(), InferenceRoute::Routed(RoutedProvider::Fabushi));
    let provider = authorize_routed_provider_request(inference.route(), RoutedProvider::Fabushi)
        .expect("existing route authorizes Fabushi");
    let host_composition = HostRunnerComposition::with_sink(
        Arc::new(SandLocalToolPermissionController::production(Arc::clone(&settings))),
        Arc::new(|_| {}),
    );
    let delivery_sink = Arc::new(ChangeDefaultOnFirstDelivery {
        settings: Arc::clone(&settings),
        deliveries: Mutex::new(Vec::new()),
    });
    ProductionFixtureOwners {
        settings,
        inference,
        provider,
        host_composition,
        delivery_sink,
        retry_events: Arc::new(Mutex::new(Vec::new())),
        retry_reports: Arc::new(Mutex::new(Vec::new())),
    }
}

impl ProductionFixtureOwners {
    fn runner(
        &self,
        root: &Path,
        stream_id: &str,
        model: String,
        fabushi_request_binding: Option<FabushiResponsesRequestBinding>,
        checkpoint_store: Arc<ProductionRoutedProviderCheckpointStore>,
    ) -> SandAgentRunner {
        let retry_sink = Arc::clone(&self.retry_events);
        let retry_report_sink = Arc::clone(&self.retry_reports);
        let composition = self.host_composition.compose_production_turn(
            ProductionRunnerCompositionInput {
                provider: self.provider,
                fabushi_model: Some(model.clone()),
                fabushi_request_binding,
                bridge: Arc::new(EmptyBridge),
                cursor_auth: None,
                request_context: RunnerRequestContextSnapshot {
                    context: HostRequestContext {
                        os_version: "contract-fixture".into(),
                        shell: None,
                        time_zone: Some("UTC".into()),
                        transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
                        user_full_name: None,
                    },
                    rules: None,
                },
                cancellation: RoutedProviderCancellation::default(),
                checkpoint_store: checkpoint_store.clone(),
                retry_sink: Some(Arc::new(move |event| {
                    retry_sink.lock().expect("retry events").push(event.clone());
                })),
                retry_report_sink: Some(Arc::new(move |report| {
                    retry_report_sink.lock().expect("retry reports").push(report.clone());
                })),
                usage_sink: None,
                box_resources: None,
                browser_executor: None,
                web_executor: None,
                generate_image_executor: None,
                computer_executor: None,
                computer_exposure: ComputerToolExposure::Full,
                file_transfer_executor: None,
                external_machine_executor: None,
                external_shell_review: None,
                mcp_management_sink: None,
                send_message_sink: Some(self.delivery_sink.clone()),
                reaction_sink: None,
                cloud_agent_tool: None,
                multitask_enabled: false,
                spotlight_enabled: false,
                toolset_role: Default::default(),
                local_tool_permission: None,
                action_audit: None,
                observation: None,
            },
            ProductionRunnerCompositionHooks {
                agent_management_sink: Arc::new(UnusedAgentManagement),
                state_writer: None,
                routine_auto_review: Arc::new(|_, _| Ok(None)),
                state_approval_barrier: Arc::new(|| Ok(())),
                box_shell_review: Arc::new(|_| Ok(None)),
                subagent_task_sink: None,
                subagent_task_allowed_types: None,
                subagent_task_review: None,
                subagent_management_runtime: None,
                subagent_steer_review: None,
                routine_post_write: None,
                multitask_todo_state: None,
            },
        );
        self.host_composition.compose_production_runner(
            composition,
            ProductionTurnAgentBuildBindings::new(
                ProductionTurnAgentStaticConfig {
                    model_id: model,
                    agent_token_limit: 200_000,
                    conversation_id: AGENT_ID.into(),
                    is_box_scoped_subagent: false,
                    is_subagent_runner: false,
                    is_shared_room_runner: false,
                    sand_send_message_delivery_owed: true,
                    transcripts_folder_available: true,
                },
                Arc::new(|| Some(SandPrivacyMode::NoTraining)),
                Arc::new(|_, _, _| Err(ProviderSessionError::Protocol(
                    "unexpected Runner summarization in bounded fixture".into(),
                ))),
            ),
            ProductionTurnAgentLifecycleBindings::new(AGENT_ID, stream_id),
            None,
            Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(SubagentRuntime::default())),
            None,
            None,
            None,
            None,
        )
    }
}

fn run_isolated_production_owners(root: &Path) {
    let owners = production_fixture_owners(root);
    let settings = &owners.settings;
    let inference = &owners.inference;
    let delivery_sink = &owners.delivery_sink;
    let retry_events = &owners.retry_events;
    let mut turn_a_checkpoint_files = None;

    for (stream_id, expected_model, expected_text) in [
        ("turn-a", MODEL_A, "working done"),
        ("turn-b", MODEL_B, "next turn"),
    ] {
        let model = inference
            .port()
            .resolve_fabushi_turn_model(None, None, None)
            .expect("resolve model through the real inference owner");
        assert_eq!(model, expected_model);
        let checkpoint_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(
            root, AGENT_ID, stream_id,
        ));
        let binding = capture_fabushi_responses_request_binding()
            .expect("capture each independently admitted production turn");
        let mut runner = owners.runner(
            root, stream_id, model, Some(binding), checkpoint_store.clone(),
        );
        let messages = [ProviderMessage {
            role: "user".into(),
            content: format!("execute {stream_id}"),
        }];
        let mut deltas = Vec::new();
        let text = runner
            .run_routed_provider_with_options(
                root,
                &messages,
                TurnRunOptions {
                    inference_request_id: Some(stream_id.into()),
                    ..TurnRunOptions::default()
                },
                &mut |delta, accumulated| {
                    deltas.push((delta.to_string(), accumulated.to_string()));
                },
            )
            .expect("real production Runner completes the loopback turn");
        assert_eq!(text, expected_text);
        if stream_id == "turn-a" {
            assert_eq!(deltas, vec![
                ("working ".to_string(), "working ".to_string()),
                ("done".to_string(), "working done".to_string()),
            ]);
            turn_a_checkpoint_files = Some(assert_durable_provider_checkpoints(
                &checkpoint_store, None,
            ));
            assert_eq!(
                settings.get_agent_default_model_checked().expect("default after delivery"),
                Some(bound_model(MODEL_B))
            );
        } else {
            let turn_a_store = ProductionRoutedProviderCheckpointStore::new(
                root, AGENT_ID, "turn-a",
            );
            assert_eq!(
                read_checkpoint_files(&turn_a_store),
                *turn_a_checkpoint_files.as_ref().expect("turn A checkpoint snapshot"),
                "turn B must leave turn A checkpoint paths and bytes unchanged"
            );
            assert_no_provider_checkpoints(&checkpoint_store);
        }
    }

    let deliveries = delivery_sink.deliveries.lock().expect("native deliveries");
    assert_eq!(deliveries.as_slice(), &[
        ("send-1".to_string(), json!({"type": "text", "content": "native delivery 1"})),
        ("send-2".to_string(), json!({"type": "text", "content": "native delivery 2"})),
    ]);
    drop(deliveries);
    let retries = retry_events.lock().expect("retry events");
    assert_eq!(retries.len(), 1);
    assert_eq!(retries[0].attempt, 1);
    assert_eq!(retries[0].next_attempt, 2);
    assert!(retries[0].resume_from_checkpoint);
    assert!(!retries[0].watchdog_expired);
    drop(retries);

    let summary_options = summarization_session_options();
    assert_eq!(
        inference
            .cursor_auth_for_session(Some(&summary_options))
            .requested_model()
            .model_id,
        SAND_SUMMARIZATION_MODEL_ID,
        "the separate Cursor summarization input keeps its existing model"
    );
    let summary = inference
        .run_summarization_prompt(
            root,
            "Summarize this fixture turn.",
            "Both native messages were delivered.",
            &|| false,
        )
        .expect("real inference summarization producer completes");
    assert_eq!(summary, "summary on model B");
}

#[test]
fn persisted_fabushi_model_is_frozen_across_tools_retry_and_summarization() {
    if let Some(root) = std::env::var_os(CHILD_ROOT_ENV) {
        run_isolated_production_owners(Path::new(&root));
        write_completion_receipt(Path::new(&root), TEST_NAME);
        return;
    }

    let mut fixture = LoopbackFixture::new();
    let nonce = uuid::Uuid::new_v4().to_string();
    // A fresh test process isolates the transport environment from other tests and
    // the process-owned Tokio runtime. No process-global environment is mutated.
    // Both turns and the accepted-checkpoint retry execute in this same child.
    let mut child = OwnedChild(Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
        .env(CHILD_ROOT_ENV, &fixture.root.0)
        .env(CHILD_NONCE_ENV, &nonce)
        .env("FABUSHI_RESPONSES_URL", &fixture.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &fixture.credential_path)
        .env("FABUSHI_RESPONSES_MODEL", "environment-model-must-not-win")
        .env_remove("FABUSHI_PRODUCT_MODE")
        .env_remove("SAND_PACKAGED")
        .env_remove("SAND_AGENT_MODEL")
        .env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env("NO_PROXY", "*")
        .env("no_proxy", "*")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start isolated production owner test"));
    let child_pid = child.0.id();
    let status = child.wait_bounded();
    let (requests, checkpoint_files_before_503, server_outcome) = fixture.finish();
    assert!(
        matches!(&status, Ok(status) if status.success()) && server_outcome.is_ok(),
        "isolated owner flow failed: child={status:?}; server={server_outcome:?}; captured {} requests",
        requests.len()
    );
    assert_completion_receipt(&fixture.root.0, &nonce, child_pid, TEST_NAME);
    let checkpoint_files_before_503 = checkpoint_files_before_503
        .expect("checkpoint files were observed before the 503");
    assert_eq!(requests.len(), 6);
    let turn_a_store = ProductionRoutedProviderCheckpointStore::new(
        &fixture.root.0, AGENT_ID, "turn-a",
    );
    assert_eq!(
        read_checkpoint_files(&turn_a_store),
        checkpoint_files_before_503,
        "retry, turn B and summarization must preserve the paths and bytes captured before the 503"
    );
    assert_no_provider_checkpoints(&ProductionRoutedProviderCheckpointStore::new(
        &fixture.root.0, AGENT_ID, "turn-b",
    ));
    let turn_models = requests[..5]
        .iter()
        .map(|request| request["model"].as_str().expect("actual serialized model"))
        .collect::<Vec<_>>();
    assert_eq!(turn_models, [MODEL_A, MODEL_A, MODEL_A, MODEL_A, MODEL_B]);
    assert_eq!(
        requests[5]["model"], MODEL_B,
        "summarization resolves the bound default separately"
    );
    for (index, request) in requests.iter().enumerate() {
        let model = if index < 4 { MODEL_A } else { MODEL_B };
        let instructions = request["instructions"].as_str().expect("runtime instructions");
        assert!(instructions.contains(&format!(
            "Runtime inference metadata: provider=Fabushi first-party Responses API; model={model}."
        )));
        assert!(!instructions.contains("environment-model-must-not-win"));
        assert_eq!(request["store"], false);
        assert_eq!(request["stream"], true);
    }
    assert_accepted_history(requests[1]["input"].as_array().expect("second step input"), 1);
    assert_accepted_history(requests[2]["input"].as_array().expect("third step input"), 2);
    assert_eq!(
        requests[2], requests[3],
        "retry reuses the complete accepted reasoning/tool context under model A"
    );
    let turn_b_input = requests[4]["input"].as_array().expect("later turn input");
    assert!(turn_b_input.iter().all(|item| !matches!(
        item.get("type").and_then(Value::as_str),
        Some("reasoning" | "function_call" | "function_call_output"),
    )), "turn B must not inherit turn A reasoning, calls or accepted tool results");
    assert!(turn_b_input.iter().any(|item| {
        item["role"] == "user" && item["content"] == "execute turn-b"
    }), "turn B must contain its new user request");
    assert!(requests[0]["tools"].as_array().expect("native tools")
        .iter().any(|tool| tool["name"] == "SendMessage"));
    assert!(
        requests[5].get("tools").is_none(),
        "summarization does not acquire the native toolset"
    );
    assert!(!requests[5]["instructions"].as_str().unwrap()
        .contains(SAND_SUMMARIZATION_MODEL_ID));
}

#[derive(Clone, Copy)]
enum BindingCase {
    PreEffectAccountSwap,
    PreEffectRotation,
    AcceptedCheckpointAccountSwap,
    AcceptedCheckpointRotation,
    FollowupAccountSwap,
    FollowupRotation,
}

impl BindingCase {
    fn test_name(self) -> &'static str {
        match self {
            Self::PreEffectAccountSwap => "direct_runner_retry_refuses_a_changed_fabushi_account",
            Self::PreEffectRotation => "direct_runner_retry_accepts_same_context_token_rotation",
            Self::AcceptedCheckpointAccountSwap => "accepted_checkpoint_retry_refuses_a_changed_fabushi_account",
            Self::AcceptedCheckpointRotation => "accepted_checkpoint_retry_accepts_same_context_token_rotation",
            Self::FollowupAccountSwap => "same_runner_nudges_and_turn_memory_refuse_a_changed_fabushi_account",
            Self::FollowupRotation => "same_runner_nudges_and_turn_memory_accept_same_context_token_rotation",
        }
    }

    fn rotates(self) -> bool {
        matches!(self, Self::PreEffectRotation | Self::AcceptedCheckpointRotation | Self::FollowupRotation)
    }

    fn has_checkpoint(self) -> bool {
        matches!(self, Self::AcceptedCheckpointAccountSwap | Self::AcceptedCheckpointRotation)
    }

    fn has_followups(self) -> bool {
        matches!(self, Self::FollowupAccountSwap | Self::FollowupRotation)
    }

    fn expected_requests(self) -> usize {
        match self {
            Self::PreEffectAccountSwap => 1,
            Self::PreEffectRotation => 2,
            Self::AcceptedCheckpointAccountSwap => 3,
            Self::AcceptedCheckpointRotation => 4,
            Self::FollowupAccountSwap => 3,
            Self::FollowupRotation => 7,
        }
    }
}

#[derive(Clone, Copy)]
enum FixtureCredential {
    AccountA,
    RotatedA,
    AccountB,
}

impl FixtureCredential {
    fn token(self) -> &'static str {
        match self {
            Self::AccountA => BINDING_TOKEN_A,
            Self::RotatedA => BINDING_TOKEN_ROTATED,
            Self::AccountB => BINDING_TOKEN_B,
        }
    }
}

fn replace_binding_credential(
    path: &Path,
    origin: &str,
    credential: FixtureCredential,
) -> Result<(), String> {
    let suffix = if matches!(credential, FixtureCredential::AccountB) { "b" } else { "a" };
    let next = path.with_file_name(format!("credential-next-{}.json", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let bytes = json!({
        "version": 1,
        "issuer": origin,
        "accessToken": credential.token(),
        "expiresAtMs": if matches!(credential, FixtureCredential::RotatedA) {
            4_102_444_900_000_u64
        } else {
            4_102_444_800_000_u64
        },
        "sessionId": format!("turn-binding-session-{suffix}"),
        "deviceId": format!("turn-binding-device-{suffix}"),
        "userId": format!("turn-binding-account-{suffix}"),
    }).to_string();
    let mut file = options.open(&next).map_err(|_| "create private binding credential")?;
    file.write_all(bytes.as_bytes()).map_err(|_| "write private binding credential")?;
    file.sync_all().map_err(|_| "sync private binding credential")?;
    drop(file);
    fs::rename(&next, path).map_err(|_| "replace private binding credential".into())
}

fn binding_success_response(text: &str) -> (&'static str, &'static str, String) {
    let events = [
        json!({"type": "response.output_text.delta", "delta": text}),
        json!({"type": "response.completed", "response": {
            "id": "turn-binding-response",
            "usage": {"input_tokens": 11, "output_tokens": 2},
            "output": [],
        }}),
    ];
    ("200 OK", "text/event-stream", events.iter()
        .map(|event| format!("data: {event}\n\n")).collect())
}

fn binding_response(case: BindingCase, index: usize) -> (&'static str, &'static str, String) {
    if case.has_followups() {
        let text = if case.rotates() {
            match index {
                0 => "main complete",
                1 => "reply nudge complete",
                2 => "closing nudge complete",
                3 => "profile: User prefers frozen turn memory",
                4 => "The bound turn completed its memory episode.",
                5 => "fresh account complete",
                6 => "summary on model B",
                _ => "unexpected extra dispatch",
            }
        } else {
            match index {
                0 => "main complete",
                1 => "fresh account complete",
                2 => "summary on model B",
                _ => "unexpected extra dispatch",
            }
        };
        return binding_success_response(text);
    }
    if case.has_checkpoint() && index < 4 {
        return scripted_response(index).expect("retained accepted-checkpoint script");
    }
    if !case.has_checkpoint() && index == 0 {
        return (
            "503 Service Unavailable",
            "application/json",
            json!({"error": "pre-effect fixture failure"}).to_string(),
        );
    }
    binding_success_response(if index < case.expected_requests() {
        "retry complete"
    } else {
        "unexpected extra dispatch"
    })
}

#[derive(Default)]
struct BindingServerState {
    accepted: usize,
    requests: Vec<CapturedRequest>,
    responded: usize,
    checkpoint_files_before_503: Option<CheckpointFiles>,
}

struct BindingFixture {
    root: FixtureDirectory,
    endpoint: String,
    credential_path: PathBuf,
    state: Arc<Mutex<BindingServerState>>,
    stop: Arc<AtomicBool>,
    server: Option<JoinHandle<Result<(), String>>>,
}

impl BindingFixture {
    fn new(case: BindingCase) -> Self {
        let root = FixtureDirectory(std::env::temp_dir().join(format!(
            "fabushi-turn-binding-{}", uuid::Uuid::new_v4(),
        )));
        fs::create_dir_all(&root.0).expect("create binding fixture directory");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind binding Responses fixture");
        listener.set_nonblocking(true).expect("pollable binding fixture listener");
        let origin = format!("http://{}", listener.local_addr().expect("binding listener address"));
        let endpoint = format!("{origin}/v1/responses");
        let credential_path = root.0.join("credential.json");
        replace_binding_credential(&credential_path, &origin, FixtureCredential::AccountA)
            .expect("initial actual account-bound credential");
        let checkpoint_store = ProductionRoutedProviderCheckpointStore::new(
            &root.0, AGENT_ID, "turn-a",
        );
        let settings_path = root.0.join("settings.json");
        let state = Arc::new(Mutex::new(BindingServerState::default()));
        let captured = Arc::clone(&state);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let replaced_path = credential_path.clone();
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(60);
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    return Err("binding Responses fixture exceeded its deadline".into());
                }
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(_) => return Err("binding fixture accept failed".into()),
                };
                captured.lock().map_err(|_| "binding capture lock poisoned")?.accepted += 1;
                let request = read_captured_request(&mut stream)?;
                let index = {
                    let mut state = captured.lock().map_err(|_| "binding capture lock poisoned")?;
                    let index = state.requests.len();
                    state.requests.push(request);
                    index
                };
                let before_retry = !case.has_followups()
                    && index == if case.has_checkpoint() { 2 } else { 0 };
                if before_retry {
                    if case.has_checkpoint() {
                        // This observation is before releasing the 503, after
                        // the real store has durably accepted both tool effects.
                        let files = assert_durable_provider_checkpoints(&checkpoint_store, None);
                        let mut state = captured.lock()
                            .map_err(|_| "binding capture lock poisoned")?;
                        for (_, bytes) in &files {
                            let RoutedProviderCheckpoint::Fabushi(checkpoint) =
                                serde_json::from_slice(bytes).expect("actual bound checkpoint")
                            else {
                                panic!("bound checkpoint must retain its provider");
                            };
                            assert!(checkpoint.input.as_slice() == state.requests[checkpoint.completed_steps]
                                .json["input"].as_array().expect("accepted request input").as_slice(),
                                "checkpoint must retain the exact accepted request input");
                            assert_no_binding_secrets(&String::from_utf8_lossy(bytes));
                        }
                        assert_eq!(
                            SettingsService::new(&settings_path)
                                .get_agent_default_model_checked().expect("default before bound retry"),
                            Some(bound_model(MODEL_B)),
                        );
                        state.checkpoint_files_before_503 = Some(files);
                    }
                    replace_binding_credential(
                        &replaced_path,
                        &origin,
                        if case.rotates() { FixtureCredential::RotatedA } else { FixtureCredential::AccountB },
                    )?;
                }
                let (status, content_type, body) = binding_response(case, index);
                write!(stream,
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len(),
                ).map_err(|_| "binding fixture response failed")?;
                captured.lock().map_err(|_| "binding capture lock poisoned")?.responded += 1;
            }
            Ok(())
        });
        Self { root, endpoint, credential_path, state, stop, server: Some(server) }
    }

    fn finish(&mut self) -> (BindingServerState, Result<(), String>) {
        self.stop.store(true, Ordering::Release);
        let outcome = self.server.take().expect("owned binding server").join()
            .unwrap_or_else(|_| Err("binding fixture server panicked".into()));
        let state = std::mem::take(&mut *self.state.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()));
        (state, outcome)
    }
}

impl Drop for BindingFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
    }
}

fn assert_no_binding_secrets(value: &str) {
    for secret in [
        BINDING_TOKEN_A, BINDING_TOKEN_ROTATED, BINDING_TOKEN_B,
        "accessToken", "expiresAtMs", "turn-binding-session-a", "turn-binding-session-b",
        "turn-binding-device-a", "turn-binding-device-b", "turn-binding-account-a",
        "turn-binding-account-b",
    ] {
        assert!(!value.contains(secret), "binding context or credential leaked into persisted data or diagnostics");
    }
}

fn assert_authentication_refusal(result: &Result<String, ProviderSessionError>) {
    let Err(error) = result else {
        panic!("changed identity must fail before HTTP");
    };
    assert!(matches!(error, ProviderSessionError::Authentication(_)),
        "identity refusal must retain typed Authentication");
    assert_no_binding_secrets(&error.to_string());
    assert_no_binding_secrets(&format!("{error:?}"));
}

fn generated_invocation_generation(runner: &SandAgentRunner, effects: u64) -> u64 {
    let Some(ToolEffectCustodyObservation::Known { run_generation, snapshot }) =
        runner.last_tool_effect_custody()
    else {
        panic!("test must enter the actual generated Runner invocation");
    };
    assert!(*run_generation > 0);
    assert_eq!(snapshot.started_revision, effects);
    assert_eq!(snapshot.checkpoint_revision, effects);
    assert_eq!(snapshot.pending_effects, 0);
    assert!(snapshot.unknown_result.is_none());
    *run_generation
}

fn run_binding_retry_child(root: &Path, case: BindingCase) {
    let owners = production_fixture_owners(root);
    let model = owners.inference.port().resolve_fabushi_turn_model(None, None, None)
        .expect("resolve the original turn model");
    assert_eq!(model, MODEL_A);
    let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "turn-a"));
    // None deliberately exercises the actual executor's single lazy binding.
    // A new transport on its retry must not capture another account.
    let mut runner = owners.runner(root, "turn-a", model, None, store.clone());
    let messages = [ProviderMessage { role: "user".into(), content: "execute turn-a".into() }];
    let mut deltas = Vec::new();
    let result = runner.run_routed_provider_with_options(
        root,
        &messages,
        TurnRunOptions { inference_request_id: Some("turn-a".into()), ..TurnRunOptions::default() },
        &mut |delta, accumulated| deltas.push((delta.to_string(), accumulated.to_string())),
    );
    if case.rotates() {
        assert!(result.as_ref().is_ok_and(|text| text == if case.has_checkpoint() {
            "working done"
        } else {
            "retry complete"
        }), "same-context rotation must complete the actual retry");
    } else {
        assert_authentication_refusal(&result);
    }
    let expected_deltas = if case.has_checkpoint() {
        let mut expected = vec![("working ".to_string(), "working ".to_string())];
        if case.rotates() {
            expected.push(("done".to_string(), "working done".to_string()));
        }
        expected
    } else if case.rotates() {
        vec![("retry complete".to_string(), "retry complete".to_string())]
    } else {
        Vec::new()
    };
    assert!(deltas == expected_deltas, "refused retry must publish no new output");
    let deliveries = owners.delivery_sink.deliveries.lock().expect("bound tool deliveries");
    if case.has_checkpoint() {
        assert!(deliveries.as_slice() == &[
            ("send-1".to_string(), json!({"type": "text", "content": "native delivery 1"})),
            ("send-2".to_string(), json!({"type": "text", "content": "native delivery 2"})),
        ], "accepted effects must occur exactly once");
        assert_durable_provider_checkpoints(&store, None);
        assert_eq!(owners.settings.get_agent_default_model_checked().expect("persisted later default"),
            Some(bound_model(MODEL_B)));
    } else {
        assert!(deliveries.is_empty());
        assert_no_provider_checkpoints(&store);
    }
    drop(deliveries);
    generated_invocation_generation(&runner, if case.has_checkpoint() { 2 } else { 0 });
    let retries = owners.retry_events.lock().expect("bound retry events");
    assert_eq!(retries.len(), 1);
    assert_eq!(retries[0].attempt, 1);
    assert_eq!(retries[0].next_attempt, 2);
    assert_eq!(retries[0].resume_from_checkpoint, case.has_checkpoint());
    assert!(!retries[0].watchdog_expired);
    let reports = owners.retry_reports.lock().expect("bound retry reports");
    assert_eq!(reports.len(), if case.rotates() { 1 } else { 2 });
    assert_eq!(reports[0].outcome, ProviderRetryOutcome::Retried);
    if !case.rotates() {
        assert_eq!(reports[1].outcome, ProviderRetryOutcome::GaveUpIneligible);
        assert_eq!(reports[1].attempt, 2);
        assert_eq!(reports[1].error_type, "AuthenticationError");
        assert_eq!(reports[1].resume_from_checkpoint, case.has_checkpoint());
        assert!(!reports[1].watchdog_expired);
    }
    for report in reports.iter() {
        assert_no_binding_secrets(&format!("{report:?}"));
    }
}

fn run_bound_memory_consumers(
    root: &Path,
    owners: &ProductionFixtureOwners,
    model: &str,
    binding: &FabushiResponsesRequestBinding,
    case: BindingCase,
) {
    let workers = ProductionSessionWorkers::with_agents_root(root.join("memory-agents"), 500);
    let session = workers.materialize_session_with_active(None, "user", None, None)
        .expect("real memory Session owner");
    let exchange = TurnExchange {
        user: "Remember the original turn identity while recording this completed exchange.".into(),
        agent: "main complete".into(),
    };
    let mut calls = 0;
    let report = run_turn_memory_with(
        &session.memory,
        Some(session.db.as_ref()),
        1_900_000_000_000,
        exchange.clone(),
        TurnMemoryMode::Extract,
        |system, user| {
            calls += 1;
            let result = owners.inference.run_turn_memory_prompt(
                root, owners.provider, Some(model), Some(binding), system, user, &|| false,
            );
            if case.rotates() {
                assert!(result.is_ok(), "actual turn-memory consumer must permit same-context rotation");
            } else {
                // Check the actual typed boundary before the Host memory
                // callback's established String error projection.
                assert_authentication_refusal(&result);
            }
            result.map_err(|_| "bound turn-memory prompt refused".to_string())
        },
    );
    assert_eq!(calls, 2, "the child-fixed episode interval must reach both real memory prompts");
    assert!(report.extraction_attempted);
    assert!(report.episode_recorded);
    assert!(report.episode_summary_attempted);
    assert!(!report.evidence_recorded);
    assert_eq!(report.removed_memories, 0);
    let pending = session.db.get_pending_episode_turns().expect("pending episode turns");
    let memories = session.memory.list_memories(100);
    if case.rotates() {
        assert!(pending.is_empty());
        assert_eq!(report.added_memories, 1);
        assert!(report.episode_memory_added);
        assert!(memories.iter().any(|memory| memory.content == "User prefers frozen turn memory"));
        assert!(memories.iter().any(|memory| memory.content == format!(
            "{MEMORY_EPISODE_PREFIX}The bound turn completed its memory episode."
        )));
    } else {
        assert_eq!(pending, vec![EpisodeTurn {
            ts: 1_900_000_000_000.0,
            user: exchange.user.clone(),
            agent: exchange.agent.clone(),
        }], "refused summary must retain the original completed exchange");
        assert_eq!(report.added_memories, 0);
        assert!(!report.episode_memory_added);
        assert!(memories.is_empty(), "refused memory prompts must not invent extracted memories");
    }

    session.db.record_episode_turn(&EpisodeTurn {
        ts: 1_900_000_000_001.0,
        user: "pending evidence user".into(),
        agent: "pending evidence agent".into(),
    }).expect("seed pending episode evidence");
    let mut evidence = Vec::new();
    let mut record = |exchange: &TurnExchange| evidence.push(exchange.clone());
    let mut evidence_inference_calls = 0;
    let evidence_report = run_turn_memory_with(
        &session.memory,
        Some(session.db.as_ref()),
        1_900_000_000_002,
        exchange.clone(),
        TurnMemoryMode::RecordEvidence(&mut record),
        |system, user| {
            evidence_inference_calls += 1;
            owners.inference.run_turn_memory_prompt(
                root, owners.provider, Some(model), Some(binding), system, user, &|| false,
            ).map_err(|_| "unexpected evidence-mode inference".to_string())
        },
    );
    assert_eq!(evidence_inference_calls, 0);
    assert!(evidence_report.evidence_recorded);
    assert!(!evidence_report.extraction_attempted);
    assert!(!evidence_report.episode_summary_attempted);
    assert_eq!(evidence, vec![exchange]);
    assert!(session.db.get_pending_episode_turns().expect("evidence clears pending turns").is_empty());
    assert!(session.memory.list_memories(100) == memories, "RecordEvidence must preserve stored memories");
    workers.shutdown();
}

fn run_binding_followup_child(root: &Path, case: BindingCase) {
    let owners = production_fixture_owners(root);
    let model = owners.inference.port().resolve_fabushi_turn_model(None, None, None)
        .expect("resolve admitted main model");
    assert_eq!(model, MODEL_A);
    let binding = capture_fabushi_responses_request_binding().expect("eager main admission binding");
    let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "bound-main"));
    let mut runner = owners.runner(root, "bound-main", model.clone(), Some(binding.clone()), store.clone());
    let messages = [ProviderMessage {
        role: "user".into(),
        content: "execute bound-main with its original admitted identity".into(),
    }];
    let options = TurnRunOptions {
        inference_request_id: Some("bound-main".into()),
        ..TurnRunOptions::default()
    };
    let mut main_deltas = Vec::new();
    let result = runner.run_routed_provider_with_options(
        root, &messages, options.clone(),
        &mut |delta, accumulated| main_deltas.push((delta.to_string(), accumulated.to_string())),
    );
    assert!(result.as_ref().is_ok_and(|text| text == "main complete"), "main invocation must complete");
    assert!(main_deltas == vec![("main complete".to_string(), "main complete".to_string())]);
    let mut generation = generated_invocation_generation(&runner, 0);
    let snapshot = owners.settings.get_host_settings_snapshot();
    owners.settings.set_host_settings_if_current(
        Some(&snapshot.hash), json!({"agentDefaultModel": bound_model(MODEL_B)}),
    ).expect("persist later default without changing admitted model");
    let endpoint = std::env::var("FABUSHI_RESPONSES_URL").expect("child-fixed endpoint");
    let origin = endpoint.strip_suffix("/v1/responses").expect("fixed fixture endpoint path");
    replace_binding_credential(
        &root.join("credential.json"), origin,
        if case.rotates() { FixtureCredential::RotatedA } else { FixtureCredential::AccountB },
    ).expect("change actual credential after the main invocation");

    for (closing, expected_text) in [(false, "reply nudge complete"), (true, "closing nudge complete")] {
        let nudge = if closing {
            shape_closing_send_nudge_turn_input(&messages, &messages, &options)
        } else {
            shape_reply_nudge_turn_input(&messages, &messages, &options)
        };
        let mut deltas = Vec::new();
        let result = runner.run_routed_provider_with_projected_messages(
            root, &nudge.lifecycle_messages, &nudge.provider_messages, nudge.options,
            &mut |delta, accumulated| deltas.push((delta.to_string(), accumulated.to_string())),
        );
        if case.rotates() {
            assert!(result.as_ref().is_ok_and(|text| text == expected_text),
                "same-runner nudge must permit same-context token rotation");
            assert!(deltas == vec![(expected_text.to_string(), expected_text.to_string())]);
        } else {
            assert_authentication_refusal(&result);
            assert!(deltas.is_empty(), "refused nudge must publish no text");
        }
        let next_generation = generated_invocation_generation(&runner, 0);
        assert!(next_generation > generation, "each nudge must enter a new actual generated invocation");
        generation = next_generation;
    }
    let missing_binding = owners.inference.run_turn_memory_prompt(
        root, owners.provider, Some(model.as_str()), None,
        "Check Native turn-memory binding admission.", "This prompt must not be dispatched.",
        &|| false,
    );
    assert!(matches!(missing_binding, Err(ProviderSessionError::Configuration(_))),
        "Native turn-memory must refuse a missing admitted binding before transport");
    let cancelled_without_binding = owners.inference.run_turn_memory_prompt(
        root, owners.provider, Some(model.as_str()), None,
        "Check Native turn-memory binding admission.", "This prompt must not be dispatched.",
        &|| true,
    );
    assert!(matches!(cancelled_without_binding, Err(ProviderSessionError::Cancelled(_))),
        "explicit turn-memory cancellation must precede missing-binding validation");
    run_bound_memory_consumers(root, &owners, &model, &binding, case);
    assert_no_provider_checkpoints(&store);
    assert!(owners.delivery_sink.deliveries.lock().expect("followup effects").is_empty());

    // A later, independently admitted turn is allowed to bind account B.
    replace_binding_credential(&root.join("credential.json"), origin, FixtureCredential::AccountB)
        .expect("select the next independently admitted account");
    let later_model = owners.inference.port().resolve_fabushi_turn_model(None, None, None)
        .expect("resolve next turn's persisted default");
    assert_eq!(later_model, MODEL_B);
    let later_binding = capture_fabushi_responses_request_binding().expect("fresh account B admission");
    let later_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "fresh-bound-turn"));
    let mut later_runner = owners.runner(
        root, "fresh-bound-turn", later_model, Some(later_binding), later_store.clone(),
    );
    let later_messages = [ProviderMessage { role: "user".into(), content: "execute fresh-bound-turn".into() }];
    let mut later_deltas = Vec::new();
    let result = later_runner.run_routed_provider_with_options(
        root, &later_messages,
        TurnRunOptions { inference_request_id: Some("fresh-bound-turn".into()), ..TurnRunOptions::default() },
        &mut |delta, accumulated| later_deltas.push((delta.to_string(), accumulated.to_string())),
    );
    assert!(result.as_ref().is_ok_and(|text| text == "fresh account complete"),
        "a new admitted turn must accept its newly captured account");
    assert!(later_deltas == vec![("fresh account complete".to_string(), "fresh account complete".to_string())]);
    generated_invocation_generation(&later_runner, 0);
    assert_no_provider_checkpoints(&later_store);
    let summary = owners.inference.run_summarization_prompt(
        root,
        "Summarize this fixture turn after changing accounts.",
        "The independently admitted account B turn completed.",
        &|| false,
    );
    assert!(summary.as_ref().is_ok_and(|text| text == "summary on model B"),
        "standalone summarization retains its independent current-default policy");
    assert!(owners.retry_events.lock().expect("followup retry events").is_empty());
    let reports = owners.retry_reports.lock().expect("followup retry reports");
    assert_eq!(reports.len(), if case.rotates() { 0 } else { 2 });
    for report in reports.iter() {
        assert_eq!(report.outcome, ProviderRetryOutcome::GaveUpIneligible);
        assert_eq!(report.attempt, 1);
        assert_eq!(report.error_type, "AuthenticationError");
        assert!(!report.watchdog_expired);
        assert!(!report.resume_from_checkpoint);
        assert_no_binding_secrets(&format!("{report:?}"));
    }
}

fn assert_binding_request(request: &CapturedRequest, token: &str, model: &str) {
    assert!(request.method == "POST" && request.path == "/v1/responses"
        && request.request_line == "POST /v1/responses HTTP/1.1", "bound request method/path");
    assert!(request.authorization.as_deref() == Some(format!(" Bearer {token}").as_str()),
        "bound request must use the expected current token, without logging it");
    assert!(!request.has_chatgpt_account_id);
    assert!(request.json["model"] == model, "bound request must retain its admitted model");
    let instructions = request.json["instructions"].as_str().expect("bound runtime instructions");
    assert!(instructions.contains(&format!(
        "Runtime inference metadata: provider=Fabushi first-party Responses API; model={model}."
    )));
    assert!(!instructions.contains("environment-model-must-not-win"));
    assert_eq!(request.json["store"], false);
    assert_eq!(request.json["stream"], true);
    let body = std::str::from_utf8(&request.raw_body).expect("serialized request UTF-8");
    for token in [BINDING_TOKEN_A, BINDING_TOKEN_ROTATED, BINDING_TOKEN_B] {
        assert!(!body.contains(token), "credentials must not enter request JSON");
    }
}

fn assert_binding_http_outcome(root: &Path, case: BindingCase, state: &BindingServerState) {
    assert_eq!(state.accepted, case.expected_requests(), "exact bound dispatch count");
    assert_eq!(state.requests.len(), state.accepted, "every accepted request must parse completely");
    assert_eq!(state.responded, state.accepted, "every accepted request must receive its complete response");
    let requests = &state.requests;
    if case.has_followups() {
        assert_binding_request(&requests[0], BINDING_TOKEN_A, MODEL_A);
        let later_index = if case.rotates() { 5 } else { 1 };
        if case.rotates() {
            for request in &requests[1..5] {
                assert_binding_request(request, BINDING_TOKEN_ROTATED, MODEL_A);
            }
            for request in &requests[1..3] {
                assert!(request.json["input"].as_array().expect("nudge request input").iter().any(|item| {
                    item["role"] == "user" && item["content"].as_str().is_some_and(|text| {
                        text.contains("BEGIN PENDING USER REQUEST")
                            && text.contains("execute bound-main with its original admitted identity")
                    })
                }), "real shaped nudge must retain the pending main request");
            }
            for (index, marker) in [(3, MEMORY_EXTRACTION_PROMPT_MARKER), (4, MEMORY_EPISODE_PROMPT_MARKER)] {
                assert!(requests[index].json.get("tools").is_none(), "turn-memory exposes no tools");
                assert!(requests[index].json["instructions"].as_str().expect("memory instructions")
                    .contains(marker), "actual memory processing must reach the expected prompt consumer");
            }
        }
        assert_binding_request(&requests[later_index], BINDING_TOKEN_B, MODEL_B);
        assert!(requests[later_index].json["input"].as_array().expect("fresh turn input").iter().any(|item| {
            item["role"] == "user" && item["content"] == "execute fresh-bound-turn"
        }), "the B request must belong to the separately admitted new turn");
        let summary = &requests[later_index + 1];
        assert_binding_request(summary, BINDING_TOKEN_B, MODEL_B);
        assert!(summary.json.get("tools").is_none(), "standalone summary exposes no native toolset");
        assert!(summary.json["instructions"].as_str().expect("summary instructions")
            .contains("Summarize this fixture turn after changing accounts."));
        assert!(!summary.json["instructions"].as_str().unwrap().contains(SAND_SUMMARIZATION_MODEL_ID));
        assert_no_provider_checkpoints(&ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "bound-main"));
        assert_no_provider_checkpoints(&ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "fresh-bound-turn"));
    } else {
        let first_attempt_requests = if case.has_checkpoint() { 3 } else { 1 };
        for request in &requests[..first_attempt_requests] {
            assert_binding_request(request, BINDING_TOKEN_A, MODEL_A);
        }
        if case.rotates() {
            assert_binding_request(&requests[first_attempt_requests], BINDING_TOKEN_ROTATED, MODEL_A);
            assert!(requests[first_attempt_requests - 1].raw_body == requests[first_attempt_requests].raw_body,
                "successful retry must send byte-identical accepted request context");
        }
        let store = ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "turn-a");
        if case.has_checkpoint() {
            assert_accepted_history(requests[1].json["input"].as_array().expect("first accepted history"), 1);
            assert_accepted_history(requests[2].json["input"].as_array().expect("second accepted history"), 2);
            let before = state.checkpoint_files_before_503.as_ref().expect("accepted bytes before account change");
            assert!(read_checkpoint_files(&store) == *before,
                "refused or permitted retry must preserve the exact accepted checkpoint paths and bytes");
            for (_, bytes) in before {
                assert_no_binding_secrets(&String::from_utf8_lossy(bytes));
            }
        } else {
            assert!(state.checkpoint_files_before_503.is_none());
            assert_no_provider_checkpoints(&store);
        }
    }
}

fn run_binding_case(case: BindingCase) {
    if let Some(root) = std::env::var_os(BINDING_ROOT_ENV) {
        assert_eq!(std::env::var(BINDING_CASE_ENV).expect("exact binding child case"), case.test_name());
        if case.has_followups() {
            run_binding_followup_child(Path::new(&root), case);
        } else {
            run_binding_retry_child(Path::new(&root), case);
        }
        write_completion_receipt(Path::new(&root), case.test_name());
        return;
    }
    let mut fixture = BindingFixture::new(case);
    let nonce = uuid::Uuid::new_v4().to_string();
    let mut child = OwnedChild(Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", case.test_name(), "--nocapture", "--test-threads=1"])
        .env_remove(CHILD_ROOT_ENV)
        .env(BINDING_ROOT_ENV, &fixture.root.0)
        .env(BINDING_CASE_ENV, case.test_name())
        .env(CHILD_NONCE_ENV, &nonce)
        .env("FABUSHI_RESPONSES_URL", &fixture.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &fixture.credential_path)
        .env("FABUSHI_RESPONSES_MODEL", "environment-model-must-not-win")
        .env("SAND_MEMORY_EPISODE_INTERVAL", "1")
        .env_remove("FABUSHI_PRODUCT_MODE")
        .env_remove("SAND_PACKAGED")
        .env_remove("SAND_AGENT_MODEL")
        .env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env("NO_PROXY", "*")
        .env("no_proxy", "*")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn().expect("start isolated bound-turn test"));
    let pid = child.0.id();
    let child_outcome = child.wait_bounded();
    // Preserve both terminal outcomes and the actual socket counters before
    // adjudication. A fixture failure cannot become a successful no-HTTP case.
    let (state, server_outcome) = fixture.finish();
    assert!(matches!(&child_outcome, Ok(status) if status.success()) && server_outcome.is_ok(),
        "{}: child={child_outcome:?}; server={server_outcome:?}; accepted={}; parsed={}; responded={}",
        case.test_name(), state.accepted, state.requests.len(), state.responded);
    assert_completion_receipt(&fixture.root.0, &nonce, pid, case.test_name());
    assert_binding_http_outcome(&fixture.root.0, case, &state);
}

#[test]
fn direct_runner_retry_refuses_a_changed_fabushi_account() {
    run_binding_case(BindingCase::PreEffectAccountSwap);
}

#[test]
fn direct_runner_retry_accepts_same_context_token_rotation() {
    run_binding_case(BindingCase::PreEffectRotation);
}

#[test]
fn accepted_checkpoint_retry_refuses_a_changed_fabushi_account() {
    run_binding_case(BindingCase::AcceptedCheckpointAccountSwap);
}

#[test]
fn accepted_checkpoint_retry_accepts_same_context_token_rotation() {
    run_binding_case(BindingCase::AcceptedCheckpointRotation);
}

#[test]
fn same_runner_nudges_and_turn_memory_refuse_a_changed_fabushi_account() {
    run_binding_case(BindingCase::FollowupAccountSwap);
}

#[test]
fn same_runner_nudges_and_turn_memory_accept_same_context_token_rotation() {
    run_binding_case(BindingCase::FollowupRotation);
}
