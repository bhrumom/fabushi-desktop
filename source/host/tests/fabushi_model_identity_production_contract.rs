//! Loopback protocol wiring through the real Settings, Inference and Runner owners.
//! This does not establish real-service compatibility or process-restart checkpoint resume.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
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
    start_production_inference_extension, summarization_session_options,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolPermissionController,
};
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::host_runner_composition::HostRunnerComposition;
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::ProviderRetryEvent;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::subagent_runtime::SubagentRuntime;
use mahayana_host_runtime::runner::tools::sand_agent_management_tools::{
    AgentManagementRecord, AgentManagementSink,
};
use mahayana_host_runtime::runner::tools::sand_computer_tool::ComputerToolExposure;
use mahayana_host_runtime::runner::tools::send_message_tool::SendMessageSink;
use mahayana_host_runtime::runner::TurnRunOptions;
use mahayana_host_runtime::runner_production_bridge::{
    ProductionRunnerCompositionHooks, ProductionRunnerCompositionInput,
};
use serde_json::{Value, json};

const MODEL_A: &str = "Vendor/Nested/CaseSensitive@Model-A";
const MODEL_B: &str = "Vendor/Nested/CaseSensitive@Model-B";
const AGENT_ID: &str = "model-identity-agent";
const CHILD_ROOT_ENV: &str = "FABUSHI_MODEL_IDENTITY_CONTRACT_CHILD_ROOT";
const TEST_NAME: &str = "persisted_fabushi_model_is_frozen_across_tools_retry_and_summarization";

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
            stop,
            server: Some(server),
        }
    }

    fn finish(&mut self) -> Vec<Value> {
        self.stop.store(true, Ordering::Release);
        self.server
            .take()
            .expect("owned fixture server")
            .join()
            .expect("fixture server must not panic")
            .expect("fixture server must complete without protocol errors");
        self.requests.lock().expect("captured requests").clone()
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

fn read_request(stream: &mut TcpStream) -> Result<Value, String> {
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
        if headers.lines().next() != Some("POST /v1/responses HTTP/1.1") {
            return Err("unexpected fixture request method or path".into());
        }
        let headers = headers.to_ascii_lowercase();
        if !headers.lines().any(|line| {
            line == "authorization: bearer model-identity-fixture-token"
        }) || headers.contains("chatgpt-account-id:") {
            return Err("request did not use the fixture Fabushi credential".into());
        }
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length:"))
            .ok_or("fixture request is missing content-length")?
            .trim()
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        if length > 4 * 1024 * 1024 {
            return Err("fixture request content-length exceeded its bound".into());
        }
        let body_start = header_end + 4;
        if bytes.len() >= body_start + length {
            return serde_json::from_slice(&bytes[body_start..body_start + length])
                .map_err(|error| error.to_string());
        }
    }
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
    fn wait_bounded(&mut self) -> ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            if let Some(status) = self.0.try_wait().expect("observe owned test child") {
                return status;
            }
            assert!(Instant::now() < deadline, "isolated production owner flow timed out");
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

fn assert_durable_provider_checkpoints(store: &ProductionRoutedProviderCheckpointStore) {
    let mut checkpoints = fs::read_dir(store.directory())
        .expect("production provider checkpoint directory")
        .map(|entry| {
            let path = entry.expect("checkpoint entry").path();
            assert_eq!(path.extension().and_then(|value| value.to_str()), Some("json"));
            let bytes = fs::read(path).expect("read durably written checkpoint");
            match serde_json::from_slice::<RoutedProviderCheckpoint>(&bytes)
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
    }
}

fn run_isolated_production_owners(root: &Path) {
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
    let retry_events = Arc::new(Mutex::new(Vec::<ProviderRetryEvent>::new()));

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
        let retry_sink = Arc::clone(&retry_events);
        let composition = host_composition.compose_production_turn(
            ProductionRunnerCompositionInput {
                provider,
                fabushi_model: Some(model.clone()),
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
                retry_report_sink: None,
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
                send_message_sink: Some(delivery_sink.clone()),
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
        let mut runner = host_composition.compose_production_runner(
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
            assert_durable_provider_checkpoints(&checkpoint_store);
            assert_eq!(
                settings.get_agent_default_model_checked().expect("default after delivery"),
                Some(bound_model(MODEL_B))
            );
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
        return;
    }

    let mut fixture = LoopbackFixture::new();
    // A fresh test process isolates the transport environment from other tests and
    // the process-owned Tokio runtime. No process-global environment is mutated.
    // Both turns and the accepted-checkpoint retry execute in this same child.
    let mut child = OwnedChild(Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
        .env(CHILD_ROOT_ENV, &fixture.root.0)
        .env("FABUSHI_RESPONSES_URL", &fixture.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &fixture.credential_path)
        .env("FABUSHI_RESPONSES_MODEL", "environment-model-must-not-win")
        .env_remove("FABUSHI_PRODUCT_MODE")
        .env_remove("SAND_PACKAGED")
        .env_remove("SAND_AGENT_MODEL")
        .env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env("NO_PROXY", "*")
        .env("no_proxy", "*")
        .spawn()
        .expect("start isolated production owner test"));
    let status = child.wait_bounded();
    let requests = fixture.finish();
    assert!(
        status.success(),
        "isolated owner flow failed: {status}; captured {} requests",
        requests.len()
    );
    assert_eq!(requests.len(), 6);
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
    assert!(requests[0]["tools"].as_array().expect("native tools")
        .iter().any(|tool| tool["name"] == "SendMessage"));
    assert!(
        requests[5].get("tools").is_none(),
        "summarization does not acquire the native toolset"
    );
    assert!(!requests[5]["instructions"].as_str().unwrap()
        .contains(SAND_SUMMARIZATION_MODEL_ID));
}
