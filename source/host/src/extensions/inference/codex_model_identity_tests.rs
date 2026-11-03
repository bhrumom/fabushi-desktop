//! CMI-v1: actual Settings/Inference/composition/Runner/serializer contracts.
//! Only the final Codex network transport is replaced. These are not account,
//! service, UI, package or interrupted-process recovery acceptance tests.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedProviderOptions, RoutedToolDefinition, run_routed_provider_text_with_lifecycle,
};
use crate::agents::agent_messaging::AgentMessageImage;
use crate::cursor_backend::SandPrivacyMode;
use crate::extensions::auth::auth_service::{HostAuthEnvironment, HostAuthServiceOptions};
use crate::extensions::auth::extension::start_host_auth_extension_with_options;
use crate::extensions::experiments::extension::{HostExperimentsExtension, HostExperimentsOptions};
use crate::extensions::inference::codex_direct_responses::{
    CodexDirectCheckpoint, CodexDirectError, CodexDirectTransport,
};
use crate::extensions::inference::cursor_session::SAND_SUMMARIZATION_MODEL_ID;
use crate::extensions::inference::inference_service::{
    InferenceRoute, authorize_routed_provider_request,
};
use crate::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
    summarization_session_options,
};
use crate::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::settings::settings_service::SettingsService;
use crate::extensions::transcript::turn_runtime::{
    shape_closing_send_nudge_turn_input, shape_reply_nudge_turn_input,
};
use crate::host_request_context::HostRequestContext;
use crate::host_runner_composition::HostRunnerComposition;
use crate::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use crate::runner::production_turn_run_shell_adapter::{
    ProviderRetryEvent, RoutedProviderCheckpointStore,
};
use crate::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedProviderRun,
    RoutedToolBridge, RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};
use crate::runner::sand_agent_runner::SandAgentRunner;
use crate::runner::sand_memory::{
    MEMORY_EPISODE_PREFIX, MEMORY_EPISODE_PROMPT_MARKER, MEMORY_EXTRACTION_PROMPT_MARKER,
};
use crate::runner::subagent_runtime::SubagentRuntime;
use crate::runner::tools::sand_agent_management_tools::{AgentManagementRecord, AgentManagementSink};
use crate::runner::tools::sand_computer_tool::ComputerToolExposure;
use crate::runner::tools::send_message_tool::SendMessageSink;
use crate::runner::turn_memory::{
    ProductionEpisodeProgress, TurnExchange, TurnMemoryMode, run_turn_memory_with,
};
use crate::runner::{ToolEffectCustodyObservation, TurnRunOptions};
use crate::runner_production_bridge::{
    ProductionRunnerCompositionHooks, ProductionRunnerCompositionInput,
};

const MODEL_A: &str = "Codex/Vendor/Nested/Case@Model-A";
const MODEL_B: &str = "Codex/Vendor/Nested/Case@Model-B";
const MODEL_C: &str = "Codex/Vendor/Nested/Case@Model-C";
const AGENT_ID: &str = "codex-model-identity-agent";
const CHILD_TEST: &str = "CMI_CONTRACT_CHILD_TEST";
const CHILD_SCENARIO: &str = "CMI_CONTRACT_CHILD_SCENARIO";
const CHILD_ROOT: &str = "CMI_CONTRACT_CHILD_ROOT";
const CHILD_NONCE: &str = "CMI_CONTRACT_CHILD_NONCE";

type TransportFactory = Rc<dyn Fn() -> Box<dyn CodexDirectTransport>>;

thread_local! {
    static TRANSPORT_FACTORY: RefCell<Option<TransportFactory>> = RefCell::new(None);
}

/// Called by the production adapter only at its final transport boundary.
/// Release the slot borrow before constructing a transport or invoking callbacks.
pub(super) fn transport_for_current_thread() -> Option<Box<dyn CodexDirectTransport>> {
    let factory = TRANSPORT_FACTORY.with(|slot| slot.borrow().clone());
    factory.map(|factory| factory())
}

struct TransportScope {
    previous: Option<TransportFactory>,
    owner: ThreadId,
    not_send: PhantomData<Rc<()>>,
}

impl TransportScope {
    fn install(factory: TransportFactory) -> Self {
        let previous = TRANSPORT_FACTORY.with(|slot| slot.replace(Some(factory)));
        Self { previous, owner: thread::current().id(), not_send: PhantomData }
    }
}

impl Drop for TransportScope {
    fn drop(&mut self) {
        assert_eq!(thread::current().id(), self.owner, "transport scope changed threads");
        TRANSPORT_FACTORY.with(|slot| {
            *slot.borrow_mut() = self.previous.take();
        });
    }
}

type ScriptStep = Box<dyn FnOnce(&Value) -> Result<Vec<Value>, CodexDirectError>>;

struct Script {
    owner: ThreadId,
    steps: RefCell<VecDeque<ScriptStep>>,
    requests: RefCell<Vec<Value>>,
    factories: Cell<usize>,
    live_transports: Cell<usize>,
}

impl Script {
    fn new(steps: Vec<ScriptStep>) -> Rc<Self> {
        Rc::new(Self {
            owner: thread::current().id(),
            steps: RefCell::new(steps.into()),
            requests: RefCell::new(Vec::new()),
            factories: Cell::new(0),
            live_transports: Cell::new(0),
        })
    }

    fn install(self: &Rc<Self>) -> TransportScope {
        let script = Rc::clone(self);
        TransportScope::install(Rc::new(move || {
            assert_eq!(thread::current().id(), script.owner, "attempt moved threads");
            TRANSPORT_FACTORY.with(|slot| {
                let _released_before_factory_callback = slot.borrow_mut();
            });
            script.factories.set(script.factories.get() + 1);
            script.live_transports.set(script.live_transports.get() + 1);
            Box::new(ScriptTransport(Rc::clone(&script)))
        }))
    }

    fn requests(&self) -> Vec<Value> {
        self.requests.borrow().clone()
    }

    fn assert_finished(&self, expected_factories: usize) {
        assert!(self.steps.borrow().is_empty(), "an actual request was not reached");
        assert_eq!(self.factories.get(), expected_factories);
        assert_eq!(self.live_transports.get(), 0, "actual adapter retained a transport");
    }
}

struct ScriptTransport(Rc<Script>);

impl Drop for ScriptTransport {
    fn drop(&mut self) {
        assert_eq!(thread::current().id(), self.0.owner);
        self.0.live_transports.set(self.0.live_transports.get() - 1);
    }
}

impl CodexDirectTransport for ScriptTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), CodexDirectError> {
        assert_eq!(thread::current().id(), self.0.owner, "serializer moved threads");
        assert!(!should_cancel(), "cancelled dispatch reached transport");
        TRANSPORT_FACTORY.with(|slot| {
            let _released_before_transport_callback = slot.borrow_mut();
        });
        self.0.requests.borrow_mut().push(request.clone());
        let step = self.0.steps.borrow_mut().pop_front().expect("unexpected actual request");
        // In particular, callbacks may observe captured requests and commit real
        // Settings while the actual request/attempt remains in progress.
        for event in step(request)? {
            on_event(event)?;
        }
        Ok(())
    }
}

fn terminal(text: &str) -> ScriptStep {
    let text = text.to_string();
    Box::new(move |_| Ok(vec![
        json!({"type":"response.output_text.delta","delta":text}),
        json!({"type":"response.completed","response":{"id":"cmi-terminal","output":[]}}),
    ]))
}

fn transient(before_error: impl FnOnce() + 'static) -> ScriptStep {
    Box::new(move |_| {
        before_error();
        Err(CodexDirectError::Transport("CMI controlled transient (503)".into()))
    })
}

struct FixtureRoot {
    path: PathBuf,
    clean: bool,
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        if self.clean {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

struct OwnedChild {
    process: Child,
    terminal: bool,
    retirement: Option<String>,
}

impl OwnedChild {
    fn retire_once_bounded(&mut self) -> String {
        if let Some(observation) = self.retirement.as_ref() {
            return observation.clone();
        }
        // Mark this one budget as spent before requesting termination. Drop
        // cannot start a second retirement campaign if observation fails.
        self.retirement = Some("retirement started; terminal outcome not observed".into());
        let kill = self.process.kill();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut observations = 0_u32;
        let outcome = loop {
            observations += 1;
            let observation = match self.process.try_wait() {
                Ok(Some(status)) => {
                    self.terminal = true;
                    break format!("terminal exit observed: {status:?}");
                }
                Ok(None) => "child still running".to_string(),
                Err(error) => format!("terminal observation failed: {error}"),
            };
            if Instant::now() >= deadline {
                break format!("retirement observation budget exhausted; {observation}; terminal=false");
            }
            thread::sleep(Duration::from_millis(10));
        };
        let observation = format!("kill request={kill:?}; observations={observations}; {outcome}");
        self.retirement = Some(observation.clone());
        observation
    }

    fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    self.terminal = true;
                    return Ok(status);
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                result => {
                    let retirement = self.retire_once_bounded();
                    return Err(format!(
                        "owned child did not complete: observed={result:?}; {retirement}"
                    ));
                }
            }
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.terminal && self.retirement.is_none() {
            let observation = self.retire_once_bounded();
            eprintln!("CMI owned child retirement during Drop: {observation}");
        }
    }
}

fn isolated(
    name: &str,
    scenario: &str,
    environment: &[(&str, &str)],
    body: impl FnOnce(&Path),
) {
    let (_, module) = module_path!().split_once("::").expect("library unit module path");
    let exact = format!("{module}::{name}");
    if let Ok(child_test) = std::env::var(CHILD_TEST) {
        assert_eq!(child_test, exact, "child must run only its exact selected test");
        if std::env::var(CHILD_SCENARIO).ok().as_deref() != Some(scenario) {
            return;
        }
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("owned child root"));
        body(&root);
        let receipt = json!({
            "test":exact, "scenario":scenario,
            "nonce":std::env::var(CHILD_NONCE).expect("child nonce"),
            "pid":std::process::id(), "completed":true,
        });
        OpenOptions::new().write(true).create_new(true)
            .open(root.join("completed.json")).expect("create unique completion receipt")
            .write_all(receipt.to_string().as_bytes()).expect("write completion receipt");
        return;
    }
    let mut root = FixtureRoot {
        path: std::env::temp_dir().join(format!("cmi-{scenario}-{}", uuid::Uuid::new_v4())),
        clean: false,
    };
    fs::create_dir_all(root.path.join("codex")).expect("create isolated Codex home");
    let nonce = uuid::Uuid::new_v4().to_string();
    let stdout_path = root.path.join("stdout.log");
    let stderr_path = root.path.join("stderr.log");
    let mut command = Command::new(std::env::current_exe().expect("unit test executable"));
    command.args(["--exact", &exact, "--nocapture", "--test-threads=1"])
        .env(CHILD_TEST, &exact).env(CHILD_SCENARIO, scenario)
        .env(CHILD_ROOT, &root.path).env(CHILD_NONCE, &nonce)
        .env("CODEX_HOME", root.path.join("codex"))
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.path.join("absent-native-credential.json"))
        .env("SAND_MEMORY_EPISODE_INTERVAL", "1")
        .env_remove("SAND_CODEX_MODEL").env_remove("SAND_CODEX_REASONING_EFFORT")
        .env_remove("FABUSHI_PRODUCT_MODE").env_remove("SAND_PACKAGED")
        .env_remove("SAND_AGENT_MODEL").env_remove("SAND_AGENT_MOCK_RESPONSE")
        .env_remove("OPENAI_API_KEY").env_remove("CODEX_API_KEY")
        .stdout(Stdio::from(File::create(&stdout_path).expect("child stdout")))
        .stderr(Stdio::from(File::create(&stderr_path).expect("child stderr")));
    for (key, value) in environment {
        command.env(key, value);
    }
    let mut child = OwnedChild {
        process: command.spawn().expect("start exact owned child"),
        terminal: false,
        retirement: None,
    };
    let pid = child.process.id();
    let status = child.wait_bounded();
    let stdout = fs::read_to_string(&stdout_path).expect("read child stdout snapshot");
    let stderr = fs::read_to_string(&stderr_path).expect("read child stderr snapshot");
    assert!(matches!(&status, Ok(status) if status.success()),
        "CMI child failed: {status:?}; terminal={}; retained at {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        child.terminal, root.path.display());
    assert!(child.terminal, "cleanup requires observed child exit");
    let receipt_bytes = fs::read(root.path.join("completed.json")).unwrap_or_else(|error| {
        panic!("selected child did not leave a terminal receipt: {error}; retained at {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            root.path.display())
    });
    let receipt: Value = serde_json::from_slice(&receipt_bytes).unwrap_or_else(|error| {
        panic!("invalid terminal receipt: {error}; retained at {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            root.path.display())
    });
    assert_eq!(receipt, json!({
        "test":exact, "scenario":scenario, "nonce":nonce, "pid":pid, "completed":true,
    }));
    root.clean = true;
}

fn bound_model(provider: &str, model: &str) -> Value {
    json!({"provider":provider,"modelId":model,"maxMode":true,"parameters":[]})
}

fn save_default(settings: &SettingsService, model: &str) {
    let before = settings.get_host_settings_snapshot();
    let committed = settings.set_host_settings_if_current(
        Some(&before.hash), json!({"agentDefaultModel":bound_model("codex", model)}),
    ).expect("real Settings compare-and-commit");
    assert_eq!(committed.settings["agentDefaultModel"], bound_model("codex", model));
    assert_ne!(committed.hash, before.hash, "the selected value really changed");
}

fn inference(settings: Arc<SettingsService>, root: &Path) -> ProductionInferenceExtension {
    let auth = start_host_auth_extension_with_options(
        HostAuthServiceOptions {
            environment: HostAuthEnvironment::from_pairs([(
                "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE".to_string(),
                root.join("absent-native-credential.json").to_string_lossy().into_owned(),
            )]),
            backend_url: Some("http://127.0.0.1:9".into()),
            backend: None,
            machine_id: Arc::new(|| Ok("cmi-contract-device".into())),
            log: Arc::new(|_| {}),
            now_ms: Arc::new(|| 0),
        },
        Arc::new(|_| Ok(None)),
    ).expect("real Host auth owner without a credential");
    start_production_inference_extension(
        Arc::new(auth),
        Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
            is_dev_build: false, env_gate_overrides: None,
        })),
        settings,
    )
}

#[derive(Default)]
struct EmptyBridge {
    lists: AtomicUsize,
    calls: AtomicUsize,
    change_config_on_first_list: Option<PathBuf>,
}

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        if self.lists.fetch_add(1, Ordering::SeqCst) == 0 {
            if let Some(path) = self.change_config_on_first_list.as_ref() {
                fs::write(path, format!("model = \"{MODEL_B}\"\n"))
                    .expect("change fallback at actual tool listing");
            }
        }
        Ok(Vec::new())
    }

    fn call_tool(&self, _: &RoutedToolDefinition, _: Value, _: &str) -> Result<Value, ProviderSessionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        panic!("unexpected delegated tool execution")
    }
}

struct UnusedAgentManagement;
impl AgentManagementSink for UnusedAgentManagement {
    fn self_agent_id(&self) -> &str { AGENT_ID }
    fn send_to_agent(&self, _: &str, _: &str, _: &[AgentMessageImage], _: bool) -> Result<String, ProviderSessionError> {
        panic!("unrelated agent-message owner must not be reached")
    }
    fn create_agent(&self, _: &str, _: &str) -> Result<AgentManagementRecord, ProviderSessionError> {
        panic!("unrelated agent creation must not be reached")
    }
    fn update_agent(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Result<Option<AgentManagementRecord>, ProviderSessionError> {
        panic!("unrelated agent update must not be reached")
    }
}

struct RecordedDelivery {
    settings: Arc<SettingsService>,
    delivered: Mutex<Vec<(String, Value)>>,
}

impl SendMessageSink for RecordedDelivery {
    fn send_message(&self, message: Value, timestamp_ms: u64, call_id: &str) -> Result<Option<String>, ProviderSessionError> {
        assert!(timestamp_ms > 0);
        let mut delivered = self.delivered.lock().expect("counted delivery");
        assert!(delivered.is_empty(), "accepted tool effect must not run twice");
        save_default(&self.settings, MODEL_B);
        delivered.push((call_id.to_string(), message));
        Ok(Some("cmi-delivery-1".into()))
    }
}

struct Owners {
    settings: Arc<SettingsService>,
    inference: ProductionInferenceExtension,
    host: HostRunnerComposition,
    provider: RoutedProvider,
    deliveries: Arc<RecordedDelivery>,
    retries: Arc<Mutex<Vec<ProviderRetryEvent>>>,
}

impl Owners {
    fn new(root: &Path, default: Option<&str>) -> Self {
        let path = root.join("settings.json");
        let writer = SettingsService::new(&path);
        let snapshot = writer.get_host_settings_snapshot();
        let mut settings = json!({"inferenceProvider":"codex"});
        if let Some(model) = default {
            settings["agentDefaultModel"] = bound_model("codex", model);
        }
        let committed = writer.set_host_settings_if_current(Some(&snapshot.hash), settings)
            .expect("save real Codex route and default");
        drop(writer);
        let settings = Arc::new(SettingsService::new(&path));
        assert_eq!(settings.get_host_settings_snapshot(), committed, "fresh owner reads acknowledged bytes");
        let inference = inference(settings.clone(), root);
        assert_eq!(inference.route(), InferenceRoute::Routed(RoutedProvider::Codex));
        let provider = authorize_routed_provider_request(inference.route(), RoutedProvider::Codex)
            .expect("route independently authorizes Codex before admission");
        let host = HostRunnerComposition::with_sink(
            Arc::new(SandLocalToolPermissionController::production(settings.clone())),
            Arc::new(|_| {}),
        );
        Self {
            settings: settings.clone(), inference, host, provider,
            deliveries: Arc::new(RecordedDelivery { settings, delivered: Mutex::new(Vec::new()) }),
            retries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn admit(&self) -> String {
        authorize_routed_provider_request(self.inference.route(), self.provider)
            .expect("selection never grants provider authority");
        self.inference.port().resolve_codex_turn_model(None, None, None)
            .expect("real Codex admission resolver")
    }

    fn runner(&self, root: &Path, stream: &str, model: String, store: Arc<ProductionRoutedProviderCheckpointStore>) -> SandAgentRunner {
        let retries = self.retries.clone();
        let composition = self.host.compose_production_turn(
            ProductionRunnerCompositionInput {
                provider: self.provider,
                fabushi_model: None,
                codex_model: Some(model.clone()),
                fabushi_request_binding: None,
                bridge: Arc::new(EmptyBridge::default()),
                cursor_auth: None,
                request_context: request_context(root),
                cancellation: RoutedProviderCancellation::default(),
                checkpoint_store: store,
                retry_sink: Some(Arc::new(move |event| retries.lock().expect("retry events").push(event.clone()))),
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
                send_message_sink: Some(self.deliveries.clone()),
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
        self.host.compose_production_runner(
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
                Arc::new(|_, _, _| panic!("bounded turn must not need compaction summarization")),
            ),
            ProductionTurnAgentLifecycleBindings::new(AGENT_ID, stream),
            None, Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(SubagentRuntime::default())),
            None, None, None, None,
        )
    }
}

fn request_context(root: &Path) -> RunnerRequestContextSnapshot {
    RunnerRequestContextSnapshot {
        context: HostRequestContext {
            os_version: "cmi-contract".into(), shell: None,
            time_zone: Some("UTC".into()),
            transcripts_folder: root.join("transcripts").to_string_lossy().into_owned(),
            user_full_name: None,
        },
        rules: None,
    }
}

fn messages(label: &str) -> Vec<ProviderMessage> {
    vec![ProviderMessage { role: "user".into(), content: format!("execute {label}") }]
}

fn custody_generation(runner: &SandAgentRunner, effects: u64) -> u64 {
    let Some(ToolEffectCustodyObservation::Known { run_generation, snapshot }) = runner.last_tool_effect_custody() else {
        panic!("actual generated invocation and custody observation are required")
    };
    assert!(*run_generation > 0);
    assert_eq!(snapshot.started_revision, effects);
    assert_eq!(snapshot.checkpoint_revision, effects);
    assert_eq!(snapshot.pending_effects, 0);
    assert!(snapshot.unknown_result.is_none());
    *run_generation
}

type CheckpointFiles = Vec<(PathBuf, Vec<u8>)>;

fn checkpoint_files(store: &ProductionRoutedProviderCheckpointStore) -> CheckpointFiles {
    let entries = match fs::read_dir(store.directory()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => panic!("cannot inspect actual checkpoint storage: {error}"),
    };
    let mut files = entries.map(|entry| {
        let path = entry.expect("checkpoint entry").path();
        assert_eq!(path.extension().and_then(|value| value.to_str()), Some("json"));
        let bytes = fs::read(&path).expect("durable checkpoint bytes");
        (path, bytes)
    }).collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn assert_models(requests: &[Value], models: &[&str]) {
    assert_eq!(requests.len(), models.len());
    for (request, model) in requests.iter().zip(models) {
        assert_eq!(request["model"], *model);
        assert_eq!(request["stream"], true);
        assert_eq!(request["store"], false);
    }
}

#[test]
fn cmi_saved_model_reaches_production_retry_and_later_admission() {
    isolated("cmi_saved_model_reaches_production_retry_and_later_admission", "run", &[], |root| {
        let owners = Owners::new(root, Some(MODEL_A));
        let selected_a = owners.admit();
        assert_eq!(selected_a, MODEL_A);
        let settings = owners.settings.clone();
        let script = Script::new(vec![
            transient(move || save_default(&settings, MODEL_B)),
            terminal("turn A"), terminal("reply nudge A"), terminal("closing nudge A"),
            terminal("turn B"),
        ]);
        let scope = script.install();
        let store_a = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "a"));
        let mut runner_a = owners.runner(root, "a", selected_a, store_a.clone());
        let turn_messages = messages("a");
        let options = TurnRunOptions { inference_request_id: Some("a".into()), ..Default::default() };
        let mut deltas = Vec::new();
        assert_eq!(runner_a.run_routed_provider_with_options(root, &turn_messages, options.clone(),
            &mut |delta, total| deltas.push((delta.to_string(), total.to_string()))).unwrap(), "turn A");
        assert_eq!(deltas, vec![("turn A".into(), "turn A".into())]);
        let mut generation = custody_generation(&runner_a, 0);
        assert_eq!(owners.settings.get_agent_default_model_checked().unwrap(), Some(bound_model("codex", MODEL_B)));
        for (closing, expected) in [(false, "reply nudge A"), (true, "closing nudge A")] {
            let nudge = if closing {
                shape_closing_send_nudge_turn_input(&turn_messages, &turn_messages, &options)
            } else {
                shape_reply_nudge_turn_input(&turn_messages, &turn_messages, &options)
            };
            assert_eq!(runner_a.run_routed_provider_with_projected_messages(
                root, &nudge.lifecycle_messages, &nudge.provider_messages, nudge.options,
                &mut |_, _| {},
            ).unwrap(), expected);
            let next = custody_generation(&runner_a, 0);
            assert!(next > generation, "nudge must enter the real generated lifecycle");
            generation = next;
        }
        let selected_b = owners.admit();
        assert_eq!(selected_b, MODEL_B);
        let store_b = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "b"));
        let mut runner_b = owners.runner(root, "b", selected_b, store_b.clone());
        assert_eq!(runner_b.run_routed_provider(root, &messages("b"), &mut |_, _| {}).unwrap(), "turn B");
        assert_models(&script.requests(), &[MODEL_A, MODEL_A, MODEL_A, MODEL_A, MODEL_B]);
        assert_eq!(script.requests()[0], script.requests()[1], "effect-free retry keeps the same complete request");
        let retries = owners.retries.lock().unwrap();
        assert_eq!(retries.len(), 1);
        assert_eq!((retries[0].attempt, retries[0].next_attempt), (1, 2));
        assert!(!retries[0].resume_from_checkpoint);
        assert!(!retries[0].watchdog_expired);
        assert!(checkpoint_files(&store_a).is_empty());
        assert!(checkpoint_files(&store_b).is_empty());
        assert!(owners.deliveries.delivered.lock().unwrap().is_empty());
        script.assert_finished(5);
        drop(scope);
        assert!(transport_for_current_thread().is_none());
    });
}

#[test]
fn cmi_checkpoint_retry_retains_model_history_and_single_effect() {
    isolated("cmi_checkpoint_retry_retains_model_history_and_single_effect", "run", &[], |root| {
        let owners = Owners::new(root, Some(MODEL_A));
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "tool-turn"));
        let before_retry = Rc::new(RefCell::new(None));
        let capture = before_retry.clone();
        let observed_store = store.clone();
        let settings = owners.settings.clone();
        let reasoning = json!({"type":"reasoning","id":"cmi-reasoning","encrypted_content":"opaque-model-a-reasoning","summary":[]});
        let call = json!({"type":"function_call","name":"SendMessage","call_id":"cmi-send-1",
            "arguments":json!({"type":"text","content":"one admitted effect"}).to_string()});
        let tool_step: ScriptStep = Box::new(move |_| Ok(vec![
            json!({"type":"response.output_text.delta","delta":"working "}),
            json!({"type":"response.completed","response":{"id":"cmi-tool-boundary","output":[reasoning,call]}}),
        ]));
        let script = Script::new(vec![tool_step, transient(move || {
            let files = checkpoint_files(&observed_store);
            assert_eq!(files.len(), 1, "the real tool boundary must be durable before continuation dispatch");
            let persisted: RoutedProviderCheckpoint = serde_json::from_slice(&files[0].1).unwrap();
            let RoutedProviderCheckpoint::Codex(checkpoint) = persisted else { panic!("wrong checkpoint provider") };
            assert_eq!(checkpoint.model.as_deref(), Some(MODEL_A));
            assert_eq!(checkpoint.tool_calls_completed, 1);
            assert_eq!(settings.get_agent_default_model_checked().unwrap(), Some(bound_model("codex", MODEL_B)));
            *capture.borrow_mut() = Some(files);
        }), terminal("done")]);
        let _scope = script.install();
        let mut runner = owners.runner(root, "tool-turn", owners.admit(), store.clone());
        let mut deltas = Vec::new();
        assert_eq!(runner.run_routed_provider(root, &messages("tool-turn"),
            &mut |delta, total| deltas.push((delta.to_string(), total.to_string()))).unwrap(), "working done");
        assert_eq!(deltas, vec![("working ".into(), "working ".into()), ("done".into(), "working done".into())]);
        custody_generation(&runner, 1);
        let requests = script.requests();
        assert_models(&requests, &[MODEL_A, MODEL_A, MODEL_A]);
        assert_eq!(requests[1], requests[2], "accepted full context must be byte-equivalent as JSON on retry");
        let files = checkpoint_files(&store);
        assert_eq!(files, before_retry.borrow().as_ref().unwrap().clone(), "retry may not rewrite accepted checkpoint bytes or paths");
        let persisted: RoutedProviderCheckpoint = serde_json::from_slice(&files[0].1).unwrap();
        let RoutedProviderCheckpoint::Codex(checkpoint) = persisted else { panic!("wrong checkpoint provider") };
        assert_eq!(checkpoint.input, requests[1]["input"].as_array().unwrap().clone());
        assert_eq!(checkpoint.completed_steps, 1);
        assert_eq!(checkpoint.text, "working ");
        let accepted = requests[1]["input"].as_array().unwrap();
        assert_eq!(accepted.iter().filter(|item| item["type"] == "reasoning").count(), 1);
        assert_eq!(accepted.iter().filter(|item| item["type"] == "function_call").count(), 1);
        let results = accepted.iter().filter(|item| item["type"] == "function_call_output").collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["call_id"], "cmi-send-1");
        let receipt: Value = serde_json::from_str(results[0]["output"].as_str().unwrap()).unwrap();
        assert_eq!(receipt["sent"], true);
        assert_eq!(receipt["messageId"], "cmi-delivery-1");
        assert_eq!(owners.deliveries.delivered.lock().unwrap().as_slice(), &[
            ("cmi-send-1".to_string(), json!({"type":"text","content":"one admitted effect"})),
        ]);
        let retries = owners.retries.lock().unwrap();
        assert_eq!(retries.len(), 1);
        assert!(retries[0].resume_from_checkpoint);
        assert!(!retries[0].watchdog_expired);
        script.assert_finished(2);
    });
}

fn direct_runner(
    root: &Path,
    model: Option<String>,
    bridge: Arc<EmptyBridge>,
    cancellation: RoutedProviderCancellation,
    store: Arc<ProductionRoutedProviderCheckpointStore>,
    retries: Arc<Mutex<Vec<ProviderRetryEvent>>>,
) -> Result<String, ProviderSessionError> {
    run_routed_provider_in_runner(RoutedProviderRun {
        provider: RoutedProvider::Codex, fabushi_model: None, codex_model: model,
        fabushi_request_binding: None, data_dir: root, messages: &messages("direct"), typed_replay: None,
        bridge, cursor_auth: None, request_context: request_context(root), cancellation,
        checkpoint_store: store,
        retry_sink: Some(Arc::new(move |event| retries.lock().unwrap().push(event.clone()))),
        retry_report_sink: None, stream_attempt_runtime: None, usage_sink: None,
        cloud_agents_enabled: false, multitask_enabled: false, is_computer_use_subagent: false,
        max_steps: 8, tool_step_reminder: None,
    }, &mut |_, _| {})
}

#[test]
fn cmi_direct_runner_fallback_is_frozen_before_tools_and_retry() {
    isolated("cmi_direct_runner_fallback_is_frozen_before_tools_and_retry", "run", &[], |root| {
        let config = root.join("codex/config.toml");
        fs::write(&config, format!("model = \"{MODEL_A}\"\n")).unwrap();
        let change_at_failure = config.clone();
        let script = Script::new(vec![transient(move || {
            assert!(fs::read_to_string(&change_at_failure).unwrap().contains(MODEL_B));
            fs::write(change_at_failure, format!("model = \"{MODEL_C}\"\n")).unwrap();
        }), terminal("frozen A"), terminal("new C")]);
        let _scope = script.install();
        let bridge = Arc::new(EmptyBridge { change_config_on_first_list: Some(config), ..Default::default() });
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "direct-a"));
        let retries = Arc::new(Mutex::new(Vec::new()));
        assert_eq!(direct_runner(root, None, bridge.clone(), Default::default(), store.clone(), retries.clone()).unwrap(), "frozen A");
        assert_eq!(bridge.lists.load(Ordering::SeqCst), 1, "model admission must precede actual tool listing");
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert_eq!(retries.lock().unwrap().len(), 1);
        assert!(!retries.lock().unwrap()[0].resume_from_checkpoint);
        assert!(checkpoint_files(&store).is_empty());
        let next_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "direct-c"));
        assert_eq!(direct_runner(root, None, Arc::new(EmptyBridge::default()), Default::default(),
            next_store.clone(), Arc::new(Mutex::new(Vec::new()))).unwrap(), "new C");
        assert!(checkpoint_files(&next_store).is_empty());
        assert_models(&script.requests(), &[MODEL_A, MODEL_A, MODEL_C]);
        script.assert_finished(3);
    });
}

#[test]
fn cmi_model_resolution_preserves_precedence_provider_isolation_and_fallbacks() {
    let name = "cmi_model_resolution_preserves_precedence_provider_isolation_and_fallbacks";
    isolated(name, "priority", &[], |root| {
        let owners = Owners::new(root, Some(MODEL_A));
        let owner = owners.inference.port();
        let selected = owner.resolve_codex_turn_model(None, None, None).unwrap();
        assert_eq!(selected, MODEL_A);
        assert_eq!(owner.resolve_codex_turn_model(None, None, Some(&Value::Null)).unwrap(), MODEL_A);
        let explicit = json!("  Explicit/Nested/Case@Preview  ");
        assert_eq!(owner.resolve_codex_turn_model(None, None, Some(&explicit)).unwrap(), "Explicit/Nested/Case@Preview");
        assert_eq!(owner.resolve_codex_turn_model(
            Some(" Session/Nested/Case@Selected "), Some("codex"), Some(&explicit),
        ).unwrap(), "Session/Nested/Case@Selected");
        save_default(&owners.settings, MODEL_B);
        assert_eq!(selected, MODEL_A, "the admission is an owned frozen value");
        assert_eq!(owner.resolve_codex_turn_model(None, None, None).unwrap(), MODEL_B);
        let reopened = inference(Arc::new(SettingsService::new(root.join("settings.json"))), root);
        assert_eq!(reopened.port().resolve_codex_turn_model(None, None, None).unwrap(), MODEL_B);
        let summary_options = summarization_session_options();
        assert_eq!(reopened.cursor_auth_for_session(Some(&summary_options)).requested_model().model_id,
            SAND_SUMMARIZATION_MODEL_ID, "bound Codex default must not replace Cursor summary selection");
        assert!(authorize_routed_provider_request(reopened.route(), RoutedProvider::Fabushi).is_err());

        fs::write(root.join("codex/config.toml"), format!("model = \"{MODEL_C}\"\n")).unwrap();
        owners.settings.set_agent_default_model(Some(&bound_model("fabushi", MODEL_B))).unwrap();
        assert_eq!(owner.resolve_codex_turn_model(None, None, None).unwrap(), MODEL_C,
            "a valid bound default for another provider is not a Codex selection");
        owners.settings.set_agent_default_model(Some(&json!({
            "modelId":"Cursor/LegacyDefault", "maxMode":false, "parameters":[],
        }))).unwrap();
        assert_eq!(owner.resolve_codex_turn_model(None, None, None).unwrap(), MODEL_C,
            "a legacy unbound default remains Cursor-only");
        assert_eq!(owners.inference.route(), InferenceRoute::Routed(RoutedProvider::Codex));
        assert!(transport_for_current_thread().is_none(), "resolution does not require transport");
    });
    isolated(name, "builtin-config", &[], |root| {
        let owners = Owners::new(root, None);
        assert_eq!(owners.admit(), "gpt-5.4");
        let config = root.join("codex/config.toml");
        fs::write(&config, format!("# selected config\nmodel = '{MODEL_C}'\n")).unwrap();
        let before = fs::read(&config).unwrap();
        assert_eq!(owners.admit(), MODEL_C);
        assert_eq!(fs::read(&config).unwrap(), before, "selection does not rewrite Codex config");
        fs::write(&config, "model = '   '\n").unwrap();
        assert_eq!(owners.admit(), "gpt-5.4", "blank optional config preserves compatibility fallback");
    });
    isolated(name, "environment", &[("SAND_CODEX_MODEL", "  Environment/Case@Model  ")], |root| {
        fs::write(root.join("codex/config.toml"), format!("model = \"{MODEL_C}\"\n")).unwrap();
        assert_eq!(Owners::new(root, None).admit(), "Environment/Case@Model");
    });
    isolated(name, "blank-environment", &[("SAND_CODEX_MODEL", "   ")], |root| {
        fs::write(root.join("codex/config.toml"), format!("model = \"{MODEL_C}\"\n")).unwrap();
        assert_eq!(Owners::new(root, None).admit(), MODEL_C);
    });
}

#[test]
fn cmi_model_resolution_refuses_malformed_sources_pairs_and_raw_values() {
    let name = "cmi_model_resolution_refuses_malformed_sources_pairs_and_raw_values";
    isolated(name, "selection-source", &[], |root| {
        let owners = Owners::new(root, Some(MODEL_A));
        let owner = owners.inference.port();
        let script = Script::new(Vec::new());
        let _scope = script.install();
        let explicit = json!("Explicit/Valid");
        for (model, provider) in [
            (Some("Session/A"), None), (None, Some("codex")),
            (Some(""), Some("codex")), (Some("  "), Some("codex")),
            (Some("Session\nA"), Some("codex")), (Some("Session/A"), Some("")),
            (Some("Session/A"), Some("fabushi")), (Some("Session/A"), Some("Codex")),
            (Some("Session/A"), Some(" codex ")),
        ] {
            assert!(matches!(owner.resolve_codex_turn_model(model, provider, Some(&explicit)),
                Err(ProviderSessionError::Configuration(_))), "invalid session pair used an alternative model");
        }
        for raw in [json!(false), json!(2), json!([]), json!({}), json!(""), json!("  "),
            json!("Model\nA"), json!("\tModel"), json!("Model\u{7f}")] {
            assert!(matches!(owner.resolve_codex_turn_model(None, None, Some(&raw)),
                Err(ProviderSessionError::Configuration(_))));
            assert!(matches!(owner.resolve_codex_turn_model(Some("Session/Valid"), Some("codex"), Some(&raw)),
                Err(ProviderSessionError::Configuration(_))), "session precedence must not hide malformed explicit input");
        }
        let path = root.join("settings.json");
        let valid = fs::read(&path).unwrap();
        for bytes in [b"invalid JSON".as_slice(), b"{}", b"[]", br#"{"version":2}"#, &[0xff, 0xfe],
            br#"{"version":1,"agentDefaultModel":{"provider":"codex","modelId":""}}"#] {
            fs::write(&path, bytes).unwrap();
            assert!(matches!(owner.resolve_codex_turn_model(None, None, None),
                Err(ProviderSessionError::Configuration(_))), "invalid checked settings are not absence");
            assert_eq!(fs::read(&path).unwrap(), bytes, "refusal may not repair or overwrite the source");
            assert_eq!(owner.resolve_codex_turn_model(None, None, Some(&explicit)).unwrap(), "Explicit/Valid");
            assert_eq!(owner.resolve_codex_turn_model(Some("Session/Valid"), Some("codex"), None).unwrap(), "Session/Valid");
        }
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(matches!(owner.resolve_codex_turn_model(None, None, None),
            Err(ProviderSessionError::Configuration(_))), "read failure must not use fallback");
        assert!(path.is_dir());
        fs::remove_dir(&path).unwrap();
        fs::write(&path, &valid).unwrap();
        assert_eq!(owner.resolve_codex_turn_model(None, None, None).unwrap(), MODEL_A);
        assert_eq!(fs::read(&path).unwrap(), valid);
        script.assert_finished(0);
    });
    isolated(name, "bad-environment", &[("SAND_CODEX_MODEL", "Model\u{7f}Control")], |root| {
        let owners = Owners::new(root, None);
        assert!(matches!(owners.inference.port().resolve_codex_turn_model(None, None, None),
            Err(ProviderSessionError::Configuration(_))));
        assert_eq!(owners.inference.port().resolve_codex_turn_model(None, None, Some(&json!(MODEL_A))).unwrap(), MODEL_A);
    });
    isolated(name, "bad-config", &[], |root| {
        fs::write(root.join("codex/config.toml"), "model = 'Model\u{7f}Control'\n").unwrap();
        let owners = Owners::new(root, None);
        assert!(matches!(owners.inference.port().resolve_codex_turn_model(None, None, None),
            Err(ProviderSessionError::Configuration(_))));
    });
}

#[derive(Default)]
struct AdapterEffects {
    tools: Cell<usize>,
    deltas: RefCell<Vec<(String, String)>>,
    checkpoints: Cell<usize>,
}

impl AdapterEffects {
    fn assert_zero(&self) {
        assert_eq!(self.tools.get(), 0);
        assert!(self.deltas.borrow().is_empty());
        assert_eq!(self.checkpoints.get(), 0);
    }
}

fn adapter(
    root: &Path,
    model: Option<&str>,
    resume: Option<&RoutedProviderCheckpoint>,
    cancelled: bool,
    effects: &AdapterEffects,
) -> Result<String, ProviderSessionError> {
    let mut execute_tool = |_: &RoutedToolDefinition, _: Value, _: &str| {
        effects.tools.set(effects.tools.get() + 1);
        panic!("refusal or text-only legacy continuation dispatched an effect")
    };
    let mut delta = |part: &str, total: &str| {
        effects.deltas.borrow_mut().push((part.to_string(), total.to_string()));
    };
    let mut options = RoutedProviderOptions {
        data_dir: root, typed_replay: None,
        fabushi_model: None, codex_model: model, fabushi_request_binding: None,
        cursor_auth: None, tools: &[], mcp_server_url: None,
        execute_tool: &mut execute_tool, on_text_delta: &mut delta,
        on_partial_tool_call: None, should_cancel: &|| cancelled,
    };
    run_routed_provider_text_with_lifecycle(
        RoutedProvider::Codex, &messages("actual adapter"), &mut options, resume,
        &mut |_| {
            effects.checkpoints.set(effects.checkpoints.get() + 1);
            panic!("refusal or text-only legacy continuation accepted a tool checkpoint")
        },
    )
}

fn known_checkpoint(model: &str) -> CodexDirectCheckpoint {
    CodexDirectCheckpoint {
        model: Some(model.into()), input: vec![json!({"role":"user","content":"existing turn"})],
        text: String::new(), response_id: "prior-response".into(), usage: Default::default(),
        completed_steps: 1, tool_calls_completed: 0,
    }
}

#[test]
fn cmi_known_checkpoint_mismatch_and_invalid_identity_have_no_effects() {
    isolated("cmi_known_checkpoint_mismatch_and_invalid_identity_have_no_effects", "run", &[], |root| {
        let script = Script::new(Vec::new());
        let _scope = script.install();
        let effects = AdapterEffects::default();
        let store = ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "known");
        let checkpoint = RoutedProviderCheckpoint::Codex(known_checkpoint(MODEL_A));
        store.persist(&checkpoint).unwrap();
        let before = checkpoint_files(&store);
        assert!(matches!(adapter(root, Some(MODEL_B), Some(&checkpoint), false, &effects),
            Err(ProviderSessionError::Protocol(_))));
        for bad in ["", " ", "\tModel", "Model\u{7f}"] {
            assert!(matches!(adapter(root, Some(bad), None, false, &effects),
                Err(ProviderSessionError::Configuration(_))));
        }
        for bad in ["", " Model ", "Model\nA"] {
            let malformed = RoutedProviderCheckpoint::Codex(known_checkpoint(bad));
            assert!(matches!(adapter(root, Some(MODEL_A), Some(&malformed), false, &effects),
                Err(ProviderSessionError::Protocol(_))));
        }
        let wrong_provider = RoutedProviderCheckpoint::Fabushi(known_checkpoint(MODEL_A));
        assert!(matches!(adapter(root, Some(MODEL_A), Some(&wrong_provider), false, &effects),
            Err(ProviderSessionError::Protocol(_))), "provider fence must remain independent of model equality");
        effects.assert_zero();
        script.assert_finished(0);
        assert_eq!(checkpoint_files(&store), before);
        assert!(!root.join("codex/auth.json").exists(), "typed refusal must precede missing authentication");

        let bridge = Arc::new(EmptyBridge::default());
        let retries = Arc::new(Mutex::new(Vec::new()));
        let run_store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "invalid-direct"));
        assert!(matches!(direct_runner(root, Some("Model\nBad".into()), bridge.clone(), Default::default(), run_store.clone(), retries.clone()),
            Err(ProviderSessionError::Configuration(_))));
        assert_eq!(bridge.lists.load(Ordering::SeqCst), 0, "invalid direct admission precedes tool catalog access");
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert!(retries.lock().unwrap().is_empty());
        assert!(checkpoint_files(&run_store).is_empty());
        script.assert_finished(0);
    });
}

#[test]
fn cmi_cancelled_dispatch_precedes_model_transport_and_tools() {
    isolated("cmi_cancelled_dispatch_precedes_model_transport_and_tools", "run", &[], |root| {
        let script = Script::new(Vec::new());
        let _scope = script.install();
        let effects = AdapterEffects::default();
        let wrong_provider = RoutedProviderCheckpoint::Fabushi(known_checkpoint(MODEL_B));
        assert!(matches!(adapter(root, Some("bad\nmodel"), Some(&wrong_provider), true, &effects),
            Err(ProviderSessionError::Cancelled(_))), "cancellation precedes provider/model refusals");
        effects.assert_zero();
        let bridge = Arc::new(EmptyBridge::default());
        let cancellation = RoutedProviderCancellation::default();
        assert!(cancellation.cancel("cmi-owned-cancellation"));
        let store = Arc::new(ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "cancelled"));
        let retries = Arc::new(Mutex::new(Vec::new()));
        let error = direct_runner(root, Some("bad\nmodel".into()), bridge.clone(), cancellation,
            store.clone(), retries.clone()).expect_err("cancelled direct runner");
        assert!(matches!(error, ProviderSessionError::Transport(ref message) if message == "cmi-owned-cancellation"),
            "retain the existing direct Runner cancellation error contract");
        assert_eq!(bridge.lists.load(Ordering::SeqCst), 0);
        assert_eq!(bridge.calls.load(Ordering::SeqCst), 0);
        assert!(retries.lock().unwrap().is_empty());
        assert!(checkpoint_files(&store).is_empty());
        script.assert_finished(0);
    });
}

#[test]
fn cmi_legacy_checkpoint_preserves_history_without_replaying_tools() {
    isolated("cmi_legacy_checkpoint_preserves_history_without_replaying_tools", "run", &[], |root| {
        let legacy: CodexDirectCheckpoint = serde_json::from_value(json!({
            "input":[
                {"role":"user","content":"existing legacy turn"},
                {"type":"reasoning","encrypted_content":"opaque-legacy-reasoning"},
                {"type":"function_call","name":"AlreadyApplied","call_id":"legacy-call","arguments":"{}"},
                {"type":"function_call_output","call_id":"legacy-call","output":"{\"applied\":true}"}
            ],
            "text":"prior ", "response_id":"legacy-response",
            "usage":{"input_tokens":7,"output_tokens":1,"cache_read_tokens":0,"cache_write_tokens":0},
            "completed_steps":1,"tool_calls_completed":1,
        })).unwrap();
        assert_eq!(legacy.model, None);
        assert!(serde_json::to_value(&legacy).unwrap().get("model").is_none());
        let store = ProductionRoutedProviderCheckpointStore::new(root, AGENT_ID, "legacy");
        let checkpoint = RoutedProviderCheckpoint::Codex(legacy.clone());
        store.persist(&checkpoint).unwrap();
        let before = checkpoint_files(&store);
        let script = Script::new(vec![terminal("resumed")]);
        let _scope = script.install();
        let effects = AdapterEffects::default();
        assert_eq!(adapter(root, Some(MODEL_B), Some(&checkpoint), false, &effects).unwrap(), "prior resumed");
        assert_eq!(effects.tools.get(), 0);
        assert_eq!(effects.checkpoints.get(), 0);
        assert_eq!(effects.deltas.borrow().as_slice(), &[("resumed".into(), "prior resumed".into())]);
        let requests = script.requests();
        assert_models(&requests, &[MODEL_B]);
        assert_eq!(requests[0]["input"], json!(legacy.input));
        assert_eq!(checkpoint_files(&store), before, "legacy positive does not mutate old durable evidence");
        script.assert_finished(1);
    });
}

#[test]
fn cmi_memory_and_summary_consume_the_correct_admitted_model() {
    isolated("cmi_memory_and_summary_consume_the_correct_admitted_model", "run", &[], |root| {
        let owners = Owners::new(root, Some(MODEL_A));
        let admitted = owners.admit();
        save_default(&owners.settings, MODEL_B);
        let script = Script::new(vec![
            terminal("profile: The admitted Codex model remains fixed"),
            terminal("The model A turn completed its memory episode."),
            terminal("standalone summary B"),
        ]);
        let _scope = script.install();
        assert!(matches!(owners.inference.run_turn_memory_prompt(
            root, owners.provider, None, None, None, "memory", "must refuse", &|| false,
        ), Err(ProviderSessionError::Configuration(_))));
        assert!(matches!(owners.inference.run_turn_memory_prompt(
            root, owners.provider, None, None, None, "memory", "must cancel", &|| true,
        ), Err(ProviderSessionError::Cancelled(_))));
        assert_eq!(script.factories.get(), 0, "missing identity/cancellation must precede transport construction");
        assert!(script.requests().is_empty());

        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("memory-agents"), 500));
        let session = workers.materialize_session_with_active(None, "user", None, None).unwrap();
        let agent_id = session.record.id.clone();
        let progress = ProductionEpisodeProgress::new(workers.clone(), agent_id.clone());
        let mut calls = 0;
        let report = run_turn_memory_with(
            &session.memory, Some(&progress), 1_900_000_000_000,
            TurnExchange { user: "remember the completed Codex turn".into(), agent: "complete".into() },
            TurnMemoryMode::Extract,
            |system, user| {
                calls += 1;
                owners.inference.run_turn_memory_prompt(
                    root, owners.provider, None, Some(&admitted), None, system, user, &|| false,
                ).map_err(|error| error.to_string())
            },
        );
        assert_eq!(calls, 2, "actual extraction and episode consumers must both run");
        assert!(report.extraction_attempted && report.episode_recorded && report.episode_summary_attempted);
        assert_eq!(report.added_memories, 1);
        assert!(report.episode_memory_added);
        assert_eq!(report.removed_memories, 0);
        assert!(workers.get_agent_pending_episode_turns(&agent_id).unwrap().is_empty());
        let memory = session.memory.list_memories(100);
        assert!(memory.iter().any(|row| row.content == "The admitted Codex model remains fixed"));
        assert!(memory.iter().any(|row| row.content == format!("{MEMORY_EPISODE_PREFIX}The model A turn completed its memory episode.")));
        let summary_options = summarization_session_options();
        assert_eq!(owners.inference.cursor_auth_for_session(Some(&summary_options)).requested_model().model_id,
            SAND_SUMMARIZATION_MODEL_ID);
        assert_eq!(owners.inference.run_summarization_prompt(root, "summarize", "completed turn", &|| false).unwrap(),
            "standalone summary B");
        let requests = script.requests();
        assert_models(&requests, &[MODEL_A, MODEL_A, MODEL_B]);
        assert!(requests[0]["instructions"].as_str().unwrap().contains(MEMORY_EXTRACTION_PROMPT_MARKER));
        assert!(requests[1]["instructions"].as_str().unwrap().contains(MEMORY_EPISODE_PROMPT_MARKER));
        for request in &requests {
            assert!(request.get("tools").is_none(), "memory/summary do not gain a tool catalog");
        }
        assert_ne!(requests[2]["model"], SAND_SUMMARIZATION_MODEL_ID);
        script.assert_finished(3);
        drop(progress);
        drop(session);
        workers.shutdown();
        drop(workers);
        let observer = ProductionSessionWorkers::with_agents_root(root.join("memory-agents"), 500);
        let reopened = observer.open_materialized_session(&agent_id).unwrap().unwrap();
        let reopened_memory = reopened.memory.list_memories(100);
        assert!(reopened_memory.iter().any(|row| row.content == "The admitted Codex model remains fixed"));
        assert!(reopened_memory.iter().any(|row| row.content == format!("{MEMORY_EPISODE_PREFIX}The model A turn completed its memory episode.")));
        assert!(observer.get_agent_pending_episode_turns(&agent_id).unwrap().is_empty());
        drop(reopened);
        observer.shutdown();
    });
}

#[test]
fn cmi_transport_factory_is_thread_scoped_borrow_free_and_restored() {
    assert!(transport_for_current_thread().is_none());
    let outer = Script::new(vec![terminal("outer")]);
    let outer_scope = outer.install();
    let first = transport_for_current_thread().expect("outer factory");
    drop(first);
    assert_eq!(outer.factories.get(), 1);
    assert!(thread::spawn(|| transport_for_current_thread().is_none()).join().unwrap(),
        "override must not cross into a sibling or future shipping worker thread");
    let inner = Script::new(Vec::new());
    let inner_for_unwind = inner.clone();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _scope = inner_for_unwind.install();
        drop(transport_for_current_thread().expect("inner factory"));
        panic!("controlled CMI scope unwind");
    }));
    assert!(unwind.is_err());
    assert_eq!(inner.factories.get(), 1);
    assert_eq!(inner.live_transports.get(), 0);
    let mut restored = transport_for_current_thread().expect("outer restored after unwind");
    let mut received = Vec::new();
    restored.stream_response(&json!({"probe":"scope-only"}), &mut |event| {
        TRANSPORT_FACTORY.with(|slot| {
            let _released_before_event_callback = slot.borrow_mut();
        });
        received.push(event);
        Ok(())
    }, &|| false).unwrap();
    assert_eq!(received.len(), 2);
    drop(restored);
    outer.assert_finished(2);
    drop(outer_scope);
    assert!(transport_for_current_thread().is_none());
    assert_eq!(inner.live_transports.get(), 0);
}
