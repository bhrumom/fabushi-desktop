//! CMI-v1/CWB-v1 controls use the actual Settings, Session, Transcript lease,
//! credential parser and permission owners. Memory and automatic compaction
//! workspace refusals reach the real Inference consumers; successful provider
//! requests and retries remain in the separately selected private Codex tests.

use super::super as shipping;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use mahayana_host_runtime::extensions::auth::auth_service::{
    HostAuthEnvironment, HostAuthServiceOptions,
};
use mahayana_host_runtime::extensions::auth::extension::start_host_auth_extension_with_options;
use mahayana_host_runtime::extensions::experiments::{HostExperimentsExtension, HostExperimentsOptions};
use mahayana_host_runtime::extensions::inference::inference_service::{
    InferenceRoute, authorize_routed_provider_request,
};
use mahayana_host_runtime::extensions::inference::model_picker_service::SessionModelEntryState;
use mahayana_host_runtime::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
};
use mahayana_host_runtime::extensions::inference::provider_session::ProviderSessionError;
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use serde_json::{Value, json};
use shipping::{GatewayEventHub, ProductionSessionWorkers, ProductionTranscriptRuntime, RoutedProvider, SettingsService};
use std::fs;
use std::io::{ErrorKind, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

const CHILD_CASE: &str = "FABUSHI_CWB_APP_CHILD_CASE";
const CHILD_ROOT: &str = "FABUSHI_CWB_APP_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_CWB_APP_CHILD_NONCE";
const WORKSPACE_A: &str = "CWB/App-Workspace-A";
const WORKSPACE_B: &str = "CWB/App-Workspace-B";

fn private_directory(path: &Path) {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).expect("exclusive private admission directory");
}

fn write_private(path: &Path, bytes: &[u8]) {
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).expect("owned private fixture file")
        .write_all(bytes).expect("complete fixture image");
}

fn write_credentials(path: &Path, workspace: &str, access_workspace: &str) {
    let jwt = |payload: Value| format!(
        "e30.{}.synthetic-cwb-signature",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap()),
    );
    let image = json!({
        "auth_mode":"chatgpt",
        "tokens":{
            "access_token":jwt(json!({
                "https://api.openai.com/auth":{"chatgpt_account_id":access_workspace},
                "exp":4_102_444_800_u64, "fixture":uuid::Uuid::new_v4().to_string(),
            })),
            "refresh_token":"synthetic-cwb-refresh",
            "id_token":jwt(json!({"aud":"synthetic-cwb-client"})),
            "account_id":workspace,
        },
    });
    write_private(path, &serde_json::to_vec(&image).unwrap());
}

fn child_root() -> PathBuf {
    PathBuf::from(std::env::var_os(CHILD_ROOT).expect("fixture requires its exact owned child"))
}

fn credential_path() -> PathBuf { child_root().join("codex/auth.json") }

fn credential_image() -> Option<Vec<u8>> {
    match fs::read(credential_path()) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("cannot observe owned credential image: {error}"),
    }
}

fn reject_proxy_connections(proxy: &TcpListener) -> usize {
    let mut count = 0;
    // Keep even an unexpected request flood inside the child's wait budget.
    for _ in 0..16 {
        match proxy.accept() {
            Ok((stream, _)) => {
                count += 1;
                // Never forward an unexpected request or read a credential.
                drop(stream);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => return count,
            Err(error) => panic!("owned proxy observation failed: {error}"),
        }
    }
    count
}

struct OwnedAdmissionChild {
    process: Child,
    terminal: bool,
    retirement: Option<String>,
}

impl OwnedAdmissionChild {
    fn retire_once_bounded(&mut self) -> String {
        if let Some(outcome) = &self.retirement { return outcome.clone(); }
        self.retirement = Some("retirement started; terminal outcome unknown".into());
        let kill = self.process.kill();
        let deadline = Instant::now() + Duration::from_secs(3);
        let outcome = loop {
            let observation = match self.process.try_wait() {
                Ok(Some(status)) => {
                    self.terminal = true;
                    break format!("terminal exit observed: {status}");
                }
                Ok(None) => "owned child still running".to_string(),
                Err(error) => format!("terminal observation failed: {error}"),
            };
            if Instant::now() >= deadline { break observation; }
            std::thread::sleep(Duration::from_millis(10));
        };
        let outcome = format!("kill={kill:?}; terminal={}; {outcome}", self.terminal);
        self.retirement = Some(outcome.clone());
        outcome
    }

    fn wait_bounded(&mut self, proxy: &TcpListener) -> Result<(ExitStatus, usize), String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut connections = 0;
        loop {
            connections += reject_proxy_connections(proxy);
            if connections != 0 {
                let retirement = self.retire_once_bounded();
                return Err(format!("unexpected HTTP connections={connections}; {retirement}"));
            }
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    self.terminal = true;
                    connections += reject_proxy_connections(proxy);
                    return Ok((status, connections));
                }
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
                result => {
                    let retirement = self.retire_once_bounded();
                    return Err(format!("owned child did not complete: {result:?}; {retirement}"));
                }
            }
        }
    }
}

impl Drop for OwnedAdmissionChild {
    fn drop(&mut self) {
        if !self.terminal && self.retirement.is_none() {
            eprintln!("CWB App owned child retirement: {}", self.retire_once_bounded());
        }
    }
}

fn isolated(case: &str, body: impl FnOnce()) {
    if let Some(selected) = std::env::var_os(CHILD_CASE) {
        assert_eq!(selected.to_str(), Some(case), "only the selected exact child may enter");
        let root = child_root();
        assert!(root.is_absolute());
        let nonce = std::env::var(CHILD_NONCE).expect("owned child nonce");
        let allocation: Value = serde_json::from_slice(&fs::read(root.join("allocation.json")).unwrap()).unwrap();
        assert_eq!(allocation["case"], case);
        assert_eq!(allocation["nonce"], nonce);
        assert_eq!(std::env::var_os("CODEX_HOME"), Some(root.join("codex").into_os_string()));
        body();
        fs::OpenOptions::new().write(true).create_new(true)
            .open(root.join("completed.json")).expect("unique assertion completion receipt")
            .write_all(json!({
                "case":case, "nonce":nonce, "pid":std::process::id(), "phase":"assertions-returned",
            }).to_string().as_bytes()).unwrap();
        return;
    }
    let base = std::env::var_os("FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT")
        .map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&base).expect("owned fixture artifact parent");
    let root = base.join(format!("cwb-app-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
    private_directory(&root);
    let root = fs::canonicalize(root).expect("absolute owned child root");
    private_directory(&root.join("codex"));
    private_directory(&root.join("tmp"));
    write_credentials(&root.join("codex/auth.json"), WORKSPACE_A, WORKSPACE_A);
    let nonce = uuid::Uuid::new_v4().to_string();
    fs::write(root.join("allocation.json"), json!({
        "case":case, "nonce":nonce, "parentPid":std::process::id(),
    }).to_string()).unwrap();
    let proxy = TcpListener::bind("127.0.0.1:0").expect("owned no-forward HTTP observation");
    proxy.set_nonblocking(true).unwrap();
    let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
    let stdout_path = root.join("stdout.log");
    let stderr_path = root.join("stderr.log");
    let mut command = Command::new(std::env::current_exe().expect("actual App test executable"));
    command.args(["--exact", case, "--nocapture", "--test-threads=1"]).env_clear();
    for key in ["PATH", "SYSTEMROOT", "WINDIR", "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH"] {
        if let Some(value) = std::env::var_os(key) { command.env(key, value); }
    }
    command.env(CHILD_CASE, case).env(CHILD_ROOT, &root).env(CHILD_NONCE, &nonce)
        .env("CODEX_HOME", root.join("codex"))
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.join("absent-native-credential.json"))
        .env("FABUSHI_RESPONSES_URL", format!("{proxy_url}/v1/responses"))
        .env("TMPDIR", root.join("tmp")).env("TEMP", root.join("tmp")).env("TMP", root.join("tmp"))
        .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()));
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"] {
        command.env(key, &proxy_url);
    }
    let mut child = OwnedAdmissionChild {
        process: command.spawn().expect("start exact isolated App child"),
        terminal: false, retirement: None,
    };
    drop(command);
    let pid = child.process.id();
    let outcome = child.wait_bounded(&proxy);
    if !matches!(&outcome, Ok((status, 0)) if status.success()) {
        let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        panic!("CWB App child failed or attempted HTTP: {outcome:?}; terminal={}; retained={}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            child.terminal, root.display());
    }
    assert!(child.terminal, "cleanup requires the actual owned child terminal");
    let receipt: Value = serde_json::from_slice(&fs::read(root.join("completed.json"))
        .expect("actual selected child completion receipt")).unwrap();
    assert_eq!(receipt, json!({
        "case":case, "nonce":nonce, "pid":pid, "phase":"assertions-returned",
    }));
    // Successful child stdout/stderr stay silent: the existing CMI parent oracle
    // must retain its sole one-passed summary. Failed roots are never removed.
    fs::remove_dir_all(&root).expect("retired exact child fixture cleanup");
}

struct CodexAdmissionFixture {
    root: PathBuf,
    settings: Arc<SettingsService>,
    inference: Arc<ProductionInferenceExtension>,
    sessions: Arc<ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    permission: SandLocalToolPermissionController,
    events: GatewayEventHub,
    agent_id: String,
}

impl CodexAdmissionFixture {
    fn new() -> Self {
        Self::new_named("owners")
    }

    fn new_named(name: &str) -> Self {
        let root = child_root().join(name);
        fs::create_dir(&root).unwrap();
        let settings_path = root.join("settings.json");
        {
            let writer = SettingsService::new(&settings_path);
            assert!(writer.set_agent_default_model(Some(&json!({
                "provider":"codex", "modelId":"Codex/Admitted@A", "maxMode":true, "parameters":[],
            }))).unwrap());
        }
        let settings = Arc::new(SettingsService::new(&settings_path));
        let auth = Arc::new(start_host_auth_extension_with_options(
            HostAuthServiceOptions {
                environment: HostAuthEnvironment::from_pairs(std::iter::empty()),
                backend_url: Some("http://127.0.0.1:9".into()),
                backend: None,
                machine_id: Arc::new(|| Ok("cmi-admission-machine".into())),
                log: Arc::new(|_| {}),
                now_ms: Arc::new(|| 1),
            },
            Arc::new(|_| Err("Codex admission must not fetch Native identity".into())),
        ).unwrap());
        let inference = Arc::new(start_production_inference_extension(
            auth,
            Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
                is_dev_build: false, env_gate_overrides: None,
            })),
            Arc::clone(&settings),
        ));
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = sessions.materialize_new_session(None, "user", None).unwrap().id;
        let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
        let permission = SandLocalToolPermissionController::production(Arc::clone(&settings));
        Self { root, settings, inference, sessions, runtime, permission,
            events: GatewayEventHub::default(), agent_id }
    }

    fn accept(&self, stream: &str) -> PersistedSendContext {
        self.accept_with_prompt(stream, "Codex admission")
    }

    fn accept_with_prompt(&self, stream: &str, prompt: &str) -> PersistedSendContext {
        let args = json!({
            "agentId":&self.agent_id, "prompt":prompt, "streamId":stream,
            "sessionKey":format!("agent:{}:main", self.agent_id),
            "clientNonce":stream,
        });
        let accepted = self.runtime.accept_routed_send(&args, |accepted| {
            shipping::persist_accepted_send_prompt_context(&self.sessions, &args, accepted)
                .map_err(|error| shipping::ProductionSendError::Internal(format!("{error:?}")))
        }).expect("actual accepted durable send and lease");
        assert!(accepted.context.accepted_durably);
        assert!(accepted.context.user_message_id.as_ref().is_some_and(|id| !id.is_empty()));
        assert_eq!(accepted.context.echo_entry_id, accepted.context.user_message_id);
        assert!(self.runtime.has_routed_turn_lease(&self.agent_id, stream));
        accepted.context
    }

    fn admit(&self, stream: &str, provider: RoutedProvider, explicit: Option<&Value>, new_direction: bool)
        -> shipping::RoutedTurnModelAdmission
    {
        shipping::prepare_routed_turn_model_admission(
            &self.runtime, &self.events, &self.sessions, &self.inference, &self.permission,
            provider, &self.agent_id, stream, explicit, new_direction,
        ).expect("actual routed model admission")
    }

    fn assert_idle(&self, stream: &str, direction: u64) {
        assert!(!self.runtime.has_routed_turn_lease(&self.agent_id, stream));
        assert_eq!(self.runtime.in_flight_run_count(&self.agent_id), 0);
        assert!(self.runtime.is_turn_dispatch_idle(&self.agent_id));
        assert_eq!(self.permission.direction_epoch(&self.agent_id), direction);
    }

    fn refuse(&self, stream: &str, raw: Option<&Value>) {
        let direction = self.permission.direction_epoch(&self.agent_id);
        self.accept(stream);
        let accepted_entries = self.sessions.read_agent_transcript_entries(&self.agent_id).unwrap();
        let result = shipping::prepare_routed_turn_model_admission(
            &self.runtime, &self.events, &self.sessions, &self.inference, &self.permission,
            RoutedProvider::Codex, &self.agent_id, stream, raw, true,
        );
        assert!(result.is_err(), "invalid Codex admission must be refused");
        assert!(matches!(&result, Err(shipping::RoutedTurnAdmissionError::Refused(
            shipping::GatewayCommandError::BadRequest(_) | shipping::GatewayCommandError::Internal(_)
        ))), "model/source refusal must not become cancellation or Native authentication");
        drop(result);
        self.assert_idle(stream, direction);
        assert_eq!(self.sessions.read_agent_transcript_entries(&self.agent_id).unwrap(), accepted_entries,
            "model refusal must preserve the already accepted user transcript");
    }

    fn put_session_pair(&self, model: Option<&str>, provider: Option<&str>) {
        // Materialize source state through the real DB owner. This fixture is
        // not a new caller-authorized session-picker mutation API.
        let state = SessionModelEntryState {
            session_id: self.agent_id.clone(),
            model_override: model.map(str::to_owned),
            provider_override: provider.map(str::to_owned),
            ..SessionModelEntryState::default()
        };
        assert!(self.sessions.open_agent_db_owner(&self.agent_id).unwrap()
            .write_kv("sessionModelSelection", &serde_json::to_string(&state).unwrap()).unwrap());
    }
}

impl Drop for CodexAdmissionFixture {
    fn drop(&mut self) {
        self.sessions.shutdown();
        // The parent removes these files only after observing this exact child
        // exit and its nonce/PID-bound receipt. Failure keeps every owned input.
    }
}

#[test]
fn cmi_shipping_admission_freezes_saved_model_and_keeps_native_recovery_isolated() {
    isolated("tests::codex_model_admission_tests::cmi_shipping_admission_freezes_saved_model_and_keeps_native_recovery_isolated", || {
        let fixture = CodexAdmissionFixture::new();
        let context = fixture.accept("native-recovery-precondition");
        let session_admission = fixture.sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
        let proof = fixture.runtime.bind_original_turn_model(
            &fixture.agent_id, "native-recovery-precondition", &session_admission, "native-positive-model",
        ).unwrap().expect("the durable fixture must be eligible for actual Native binding");
        assert_eq!(proof.provider, "fabushi");
        assert_eq!(Some(proof.user_message_id), context.user_message_id);
        assert_eq!(Some(proof.echo_entry_id), context.echo_entry_id);
        assert!(fixture.runtime.settle_routed_turn(&fixture.agent_id, "native-recovery-precondition", 1)
            .unwrap().is_some());
        fixture.assert_idle("native-recovery-precondition", 0);
        let provider = authorize_routed_provider_request(
            InferenceRoute::Routed(RoutedProvider::Codex), RoutedProvider::Codex,
        ).unwrap();
        assert!(authorize_routed_provider_request(
            InferenceRoute::Routed(RoutedProvider::Fabushi), RoutedProvider::Codex,
        ).is_err(), "a saved Codex default cannot grant a different route");
        fixture.accept("model-a");
        let first = fixture.admit("model-a", provider, None, true);
        assert_eq!(first.codex_model.as_deref(), Some("Codex/Admitted@A"));
        assert!(first.fabushi_model.is_none());
        assert!(first.fabushi_request_binding.is_none());
        assert!(first.original_turn.is_none(), "Codex must not acquire Native recovery proof");
        let direction = first.direction_epoch;
        assert!(fixture.settings.set_agent_default_model(Some(&json!({
            "provider":"codex", "modelId":"Codex/Admitted@B", "maxMode":true, "parameters":[],
        }))).unwrap());
        assert_eq!(first.codex_model.as_deref(), Some("Codex/Admitted@A"));
        drop(first);
        fixture.assert_idle("model-a", direction);
        fixture.accept("model-b");
        let second = fixture.admit("model-b", provider, None, false);
        assert_eq!(second.codex_model.as_deref(), Some("Codex/Admitted@B"));
        assert_eq!(second.direction_epoch, direction);
        assert!(second.original_turn.is_none());
        drop(second);
        fixture.assert_idle("model-b", direction);
    });
}

#[test]
fn cmi_shipping_refusal_releases_lease_without_permission_or_settings_effects() {
    isolated("tests::codex_model_admission_tests::cmi_shipping_refusal_releases_lease_without_permission_or_settings_effects", || {
        let fixture = CodexAdmissionFixture::new();
        let settings_path = fixture.root.join("settings.json");
        let good = fs::read(&settings_path).unwrap();
        for (index, raw) in [json!(false), json!(17), json!(" "), json!("\nModel")].iter().enumerate() {
            fixture.refuse(&format!("bad-raw-{index}"), Some(raw));
            assert_eq!(fs::read(&settings_path).unwrap(), good);
        }
        fs::write(&settings_path, b"{").unwrap();
        fixture.refuse("bad-source", None);
        assert_eq!(fs::read(&settings_path).unwrap(), b"{");
        fs::write(&settings_path, &good).unwrap();
        {
            let db = fixture.sessions.open_agent_db_owner(&fixture.agent_id).unwrap();
            assert!(db.write_kv("sessionModelSelection", "{").unwrap());
            fixture.refuse("bad-session-json", None);
            assert_eq!(db.read_kv("sessionModelSelection").unwrap().as_deref(), Some("{"));
            assert!(db.delete_kv("sessionModelSelection").unwrap());
        }
        for (index, (model, provider)) in [
            (Some("Codex/Session"), None), (None, Some("codex")),
            (Some("Codex/Session"), Some("fabushi")), (Some(" "), Some("codex")),
        ].into_iter().enumerate() {
            fixture.put_session_pair(model, provider);
            fixture.refuse(&format!("bad-pair-{index}"), Some(&json!("Codex/Explicit")));
        }
        fixture.put_session_pair(None, None);
        fixture.accept("valid-after-refusal");
        let accepted = fixture.admit("valid-after-refusal", RoutedProvider::Codex,
            Some(&json!("  Codex/Explicit@Case  ")), true);
        assert_eq!(accepted.codex_model.as_deref(), Some("Codex/Explicit@Case"));
        let direction = accepted.direction_epoch;
        drop(accepted);
        fixture.assert_idle("valid-after-refusal", direction);
    });
}

#[test]
fn cmi_shipping_session_precedence_preserves_raw_validation_and_other_routes() {
    isolated("tests::codex_model_admission_tests::cmi_shipping_session_precedence_preserves_raw_validation_and_other_routes", || {
        let fixture = CodexAdmissionFixture::new();
        fixture.put_session_pair(Some("Codex/Session@Case"), Some("codex"));
        fixture.refuse("malformed-explicit-not-hidden", Some(&json!({"model":"invalid"})));
        fixture.accept("session-wins");
        let selected = fixture.admit("session-wins", RoutedProvider::Codex,
            Some(&json!("Codex/Explicit")), false);
        assert_eq!(selected.codex_model.as_deref(), Some("Codex/Session@Case"));
        assert_eq!(selected.persisted_session_model.provider_override.as_deref(), Some("codex"));
        assert!(selected.original_turn.is_none());
        let direction = selected.direction_epoch;
        drop(selected);
        fixture.assert_idle("session-wins", direction);
        fixture.put_session_pair(None, None);
        fs::write(fixture.root.join("settings.json"), b"{").unwrap();
        fixture.accept("other-route");
        let other = fixture.admit("other-route", RoutedProvider::OpenRouter, None, false);
        assert!(other.codex_model.is_none());
        assert!(other.fabushi_model.is_none());
        assert!(other.fabushi_request_binding.is_none());
        assert!(other.original_turn.is_none());
        drop(other);
        fixture.assert_idle("other-route", direction);
        assert_eq!(fs::read(fixture.root.join("settings.json")).unwrap(), b"{");
    });
}

#[test]
fn cmi_shipping_cancellation_before_and_after_model_capture_preserves_admission_owners() {
    isolated("tests::codex_model_admission_tests::cmi_shipping_cancellation_before_and_after_model_capture_preserves_admission_owners", || {
        let fixture = CodexAdmissionFixture::new();
        let direction = fixture.permission.direction_epoch(&fixture.agent_id);
        let settings_path = fixture.root.join("settings.json");
        let settings_bytes = fs::read(&settings_path).unwrap();
        fixture.accept("cancel-before-model");
        let before_entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        let db = fixture.sessions.open_agent_db_owner(&fixture.agent_id).unwrap();
        assert!(db.write_kv("sessionModelSelection", "{").unwrap());
        assert_eq!(fixture.runtime.request_routed_turn_cancellation(
            "cancel-before-model", Some(&fixture.agent_id), "CMI before model",
        ), Some(true));
        assert_eq!(fixture.runtime.request_routed_turn_cancellation(
            "cancel-before-model", Some(&fixture.agent_id), "must retain the first cancellation",
        ), Some(false));
        assert!(fixture.runtime.has_routed_turn_lease(&fixture.agent_id, "cancel-before-model"));
        assert!(!fixture.runtime.is_turn_dispatch_idle(&fixture.agent_id));
        let mut observed = Vec::new();
        let result = shipping::prepare_routed_turn_model_admission_observed(
            &fixture.runtime, &fixture.events, &fixture.sessions, &fixture.inference,
            &fixture.permission, RoutedProvider::Codex, &fixture.agent_id,
            "cancel-before-model", None, true, |point| observed.push(point),
        );
        assert!(matches!(result, Err(shipping::RoutedTurnAdmissionError::Cancelled)),
            "cancellation must win before the deliberately invalid Session model read");
        assert_eq!(observed, vec![shipping::RoutedTurnAdmissionPoint::BeforeModel]);
        fixture.assert_idle("cancel-before-model", direction);
        assert_eq!(db.read_kv("sessionModelSelection").unwrap().as_deref(), Some("{"));
        assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), before_entries);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
        assert!(db.delete_kv("sessionModelSelection").unwrap());

        fixture.accept("cancel-after-model");
        let captured_entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        observed.clear();
        let result = shipping::prepare_routed_turn_model_admission_observed(
            &fixture.runtime, &fixture.events, &fixture.sessions, &fixture.inference,
            &fixture.permission, RoutedProvider::Codex, &fixture.agent_id,
            "cancel-after-model", None, true, |point| {
                observed.push(point);
                if point == shipping::RoutedTurnAdmissionPoint::ModelCaptured {
                    assert_eq!(fixture.permission.direction_epoch(&fixture.agent_id), direction);
                    assert_eq!(fixture.runtime.request_routed_turn_cancellation(
                        "cancel-after-model", Some(&fixture.agent_id), "CMI after model",
                    ), Some(true));
                    assert!(fixture.runtime.has_routed_turn_lease(&fixture.agent_id, "cancel-after-model"));
                    assert!(!fixture.runtime.is_turn_dispatch_idle(&fixture.agent_id));
                }
            },
        );
        assert!(matches!(result, Err(shipping::RoutedTurnAdmissionError::Cancelled)));
        assert_eq!(observed, vec![shipping::RoutedTurnAdmissionPoint::BeforeModel,
            shipping::RoutedTurnAdmissionPoint::ModelCaptured]);
        fixture.assert_idle("cancel-after-model", direction);
        assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), captured_entries);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
        assert!(db.read_kv("sessionModelSelection").unwrap().is_none());

        assert!(fixture.settings.set_agent_default_model(Some(&json!({
            "provider":"codex", "modelId":"Codex/AfterCancel@B", "maxMode":true, "parameters":[],
        }))).unwrap());
        fixture.accept("valid-after-cancellation");
        let later = fixture.admit("valid-after-cancellation", RoutedProvider::Codex, None, true);
        assert_eq!(later.codex_model.as_deref(), Some("Codex/AfterCancel@B"));
        assert_eq!(later.direction_epoch, direction + 1);
        assert!(later.original_turn.is_none());
        assert!(!later.lease_guard.control.cancellation().is_cancelled());
        drop(later);
        fixture.assert_idle("valid-after-cancellation", direction + 1);
    });
}

#[test]
fn cwb_shipping_credential_refusal_preserves_accepted_input_settings_and_exact_lease() {
    isolated("tests::codex_model_admission_tests::cwb_shipping_credential_refusal_preserves_accepted_input_settings_and_exact_lease", || {
        let fixture = CodexAdmissionFixture::new();
        let direction = fixture.permission.direction_epoch(&fixture.agent_id);
        let settings_path = fixture.root.join("settings.json");
        let settings_bytes = fs::read(&settings_path).unwrap();
        let other_agent = fixture.sessions.materialize_new_session(None, "user", None).unwrap().id;
        let other_stream = "cwb-unrelated-accepted-owner";
        let other_args = json!({
            "agentId":&other_agent, "prompt":"unrelated accepted input", "streamId":other_stream,
            "sessionKey":format!("agent:{other_agent}:main"), "clientNonce":other_stream,
        });
        let other_accepted = fixture.runtime.accept_routed_send(&other_args, |accepted| {
            shipping::persist_accepted_send_prompt_context(&fixture.sessions, &other_args, accepted)
                .map_err(|error| shipping::ProductionSendError::Internal(format!("{error:?}")))
        }).unwrap();
        assert!(other_accepted.context.accepted_durably);
        let other_guard = shipping::RoutedTurnLeaseGuard::new(
            Arc::clone(&fixture.runtime), other_agent.clone(), other_stream.to_string(), fixture.events.clone(),
        ).unwrap();
        let other_entries = fixture.sessions.read_agent_transcript_entries(&other_agent).unwrap();

        for scenario in ["missing", "malformed", "conflicting-workspace"] {
            let auth_path = credential_path();
            match scenario {
                "missing" => fs::remove_file(&auth_path).unwrap(),
                "malformed" => write_private(&auth_path, b"{"),
                "conflicting-workspace" => write_credentials(&auth_path, WORKSPACE_A, WORKSPACE_B),
                _ => unreachable!(),
            }
            let auth_before = credential_image();
            let stream = format!("cwb-credential-refusal-{scenario}");
            fixture.accept(&stream);
            let entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
            let result = shipping::prepare_routed_turn_model_admission(
                &fixture.runtime, &fixture.events, &fixture.sessions, &fixture.inference,
                &fixture.permission, RoutedProvider::Codex, &fixture.agent_id, &stream, None, true,
            );
            match result {
                Err(shipping::RoutedTurnAdmissionError::Refused(shipping::GatewayCommandError::Unauthorized(message))) => {
                    for secret in ["synthetic-cwb", WORKSPACE_A, WORKSPACE_B] {
                        assert!(!message.contains(secret), "credential refusal must stay redacted");
                    }
                }
                Err(error) => panic!("credential admission returned the wrong typed refusal: {error:?}"),
                Ok(admission) => {
                    drop(admission);
                    panic!("invalid workspace credentials acquired preparation authority");
                }
            }
            fixture.assert_idle(&stream, direction);
            assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), entries);
            assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
            assert_eq!(credential_image(), auth_before, "refusal must not repair or replace the login image");
            assert!(fixture.runtime.has_routed_turn_lease(&other_agent, other_stream));
            assert!(!fixture.runtime.is_turn_dispatch_idle(&other_agent));
            assert!(!other_guard.control.cancellation().is_cancelled());
            assert_eq!(fixture.sessions.read_agent_transcript_entries(&other_agent).unwrap(), other_entries);
        }
        write_credentials(&credential_path(), WORKSPACE_A, WORKSPACE_A);
        fixture.accept("cwb-valid-after-credential-repair");
        let admitted = fixture.admit("cwb-valid-after-credential-repair", RoutedProvider::Codex, None, true);
        assert_eq!(admitted.codex_model.as_deref(), Some("Codex/Admitted@A"));
        assert_eq!(admitted.direction_epoch, direction + 1);
        assert!(admitted.original_turn.is_none());
        drop(admitted);
        fixture.assert_idle("cwb-valid-after-credential-repair", direction + 1);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
        assert!(fixture.runtime.has_routed_turn_lease(&other_agent, other_stream));
        drop(other_guard);
        assert!(!fixture.runtime.has_routed_turn_lease(&other_agent, other_stream));
        assert!(fixture.runtime.is_turn_dispatch_idle(&other_agent));
    });
}

#[test]
fn cwb_shipping_bound_memory_refuses_changed_workspace_before_transport_and_later_admission() {
    isolated("tests::codex_model_admission_tests::cwb_shipping_bound_memory_refuses_changed_workspace_before_transport_and_later_admission", || {
        let fixture = CodexAdmissionFixture::new();
        let settings_path = fixture.root.join("settings.json");
        let settings_bytes = fs::read(&settings_path).unwrap();
        fixture.accept("cwb-admitted-workspace-a");
        let first = fixture.admit("cwb-admitted-workspace-a", RoutedProvider::Codex, None, true);
        let model = first.codex_model.clone().expect("actual admitted Codex model");
        let original_binding = first.codex_request_binding.clone().expect("actual captured workspace binding");
        let direction = first.direction_epoch;
        let first_entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        let missing_binding = fixture.inference.run_turn_memory_prompt(
            &fixture.root, RoutedProvider::Codex, None, Some(&model), None, None,
            "workspace-bound turn memory", "accepted user input", &|| false,
        );
        assert!(matches!(missing_binding, Err(ProviderSessionError::Configuration(_))));

        write_credentials(&credential_path(), WORKSPACE_B, WORKSPACE_B);
        let login_b = credential_image();
        let refused = fixture.inference.run_turn_memory_prompt(
            &fixture.root, RoutedProvider::Codex, first.fabushi_model.as_deref(), first.codex_model.as_deref(),
            first.codex_request_binding.as_ref(), first.fabushi_request_binding.as_ref(),
            "workspace-bound turn memory", "accepted user input",
            &|| first.lease_guard.control.cancellation().is_cancelled(),
        );
        assert!(matches!(&refused, Err(ProviderSessionError::Authentication(_))),
            "the actual Inference consumer must refuse B before transport: {refused:?}");
        let rendered = refused.unwrap_err().to_string();
        for secret in ["synthetic-cwb", WORKSPACE_A, WORKSPACE_B] {
            assert!(!rendered.contains(secret));
        }
        assert_eq!(credential_image(), login_b);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
        assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), first_entries);
        assert!(fixture.runtime.has_routed_turn_lease(&fixture.agent_id, "cwb-admitted-workspace-a"));
        assert_eq!(fixture.permission.direction_epoch(&fixture.agent_id), direction);
        drop(first);
        fixture.assert_idle("cwb-admitted-workspace-a", direction);

        fixture.accept("cwb-later-workspace-b");
        let later = fixture.admit("cwb-later-workspace-b", RoutedProvider::Codex, None, true);
        assert_eq!(later.codex_model.as_deref(), Some(model.as_str()));
        assert_eq!(later.direction_epoch, direction + 1);
        assert!(later.original_turn.is_none());
        let later_entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
        assert!(later_entries.starts_with(&first_entries));
        let old_turn = fixture.inference.run_turn_memory_prompt(
            &fixture.root, RoutedProvider::Codex, None, Some(&model), Some(&original_binding), None,
            "original turn memory", "original accepted input", &|| false,
        );
        assert!(matches!(old_turn, Err(ProviderSessionError::Authentication(_))),
            "a new admission must not replace the original turn's workspace identity");

        // The new admission has a real B binding, not merely Some or a global A
        // value: the reverse replacement must be refused by its actual consumer.
        write_credentials(&credential_path(), WORKSPACE_A, WORKSPACE_A);
        let login_a = credential_image();
        let reverse = fixture.inference.run_turn_memory_prompt(
            &fixture.root, RoutedProvider::Codex, later.fabushi_model.as_deref(), later.codex_model.as_deref(),
            later.codex_request_binding.as_ref(), later.fabushi_request_binding.as_ref(),
            "later turn memory", "later accepted input",
            &|| later.lease_guard.control.cancellation().is_cancelled(),
        );
        assert!(matches!(reverse, Err(ProviderSessionError::Authentication(_))));
        assert_eq!(credential_image(), login_a);
        assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), later_entries);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
        drop(later);
        fixture.assert_idle("cwb-later-workspace-b", direction + 1);
    });
}

#[test]
fn cwb_shipping_cancellation_precedes_credentials_and_other_routes_stay_isolated() {
    isolated("tests::codex_model_admission_tests::cwb_shipping_cancellation_precedes_credentials_and_other_routes_stay_isolated", || {
        let fixture = CodexAdmissionFixture::new();
        let direction = fixture.permission.direction_epoch(&fixture.agent_id);
        let settings_path = fixture.root.join("settings.json");
        let settings_bytes = fs::read(&settings_path).unwrap();
        for malformed in [false, true] {
            if malformed {
                write_private(&credential_path(), b"{");
            } else {
                fs::remove_file(credential_path()).unwrap();
            }
            let auth_before = credential_image();
            let stream = if malformed { "cwb-cancel-malformed-login" } else { "cwb-cancel-missing-login" };
            fixture.accept(stream);
            let entries = fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap();
            assert_eq!(fixture.runtime.request_routed_turn_cancellation(
                stream, Some(&fixture.agent_id), "cancel before credential admission",
            ), Some(true));
            let mut observed = Vec::new();
            let result = shipping::prepare_routed_turn_model_admission_observed(
                &fixture.runtime, &fixture.events, &fixture.sessions, &fixture.inference,
                &fixture.permission, RoutedProvider::Codex, &fixture.agent_id, stream, None, true,
                |point| observed.push(point),
            );
            assert!(matches!(result, Err(shipping::RoutedTurnAdmissionError::Cancelled)),
                "captured cancellation must win over missing or malformed login");
            assert_eq!(observed, vec![shipping::RoutedTurnAdmissionPoint::BeforeModel]);
            fixture.assert_idle(stream, direction);
            assert_eq!(credential_image(), auth_before);
            assert_eq!(fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(), entries);
            assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
            let memory = fixture.inference.run_turn_memory_prompt(
                &fixture.root, RoutedProvider::Codex, None, None, None, None,
                "cancelled memory", "accepted input", &|| true,
            );
            assert!(matches!(memory, Err(ProviderSessionError::Cancelled(_))),
                "Memory cancellation must precede missing model and binding admission");
        }
        let invalid_login = credential_image();
        fixture.accept("cwb-other-route-with-invalid-codex-login");
        let other = fixture.admit("cwb-other-route-with-invalid-codex-login", RoutedProvider::OpenRouter, None, false);
        assert!(other.codex_model.is_none());
        assert!(other.codex_request_binding.is_none());
        assert!(other.fabushi_model.is_none());
        assert!(other.fabushi_request_binding.is_none());
        assert!(other.original_turn.is_none());
        assert_eq!(other.direction_epoch, direction);
        drop(other);
        fixture.assert_idle("cwb-other-route-with-invalid-codex-login", direction);
        assert_eq!(credential_image(), invalid_login);
        assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);

        write_credentials(&credential_path(), WORKSPACE_B, WORKSPACE_B);
        fixture.accept("cwb-valid-after-cancellation");
        let admitted = fixture.admit("cwb-valid-after-cancellation", RoutedProvider::Codex, None, true);
        assert_eq!(admitted.codex_model.as_deref(), Some("Codex/Admitted@A"));
        assert_eq!(admitted.direction_epoch, direction + 1);
        assert!(!admitted.lease_guard.control.cancellation().is_cancelled());
        drop(admitted);
        fixture.assert_idle("cwb-valid-after-cancellation", direction + 1);
    });
}

#[test]
fn cwb_shipping_compaction_uses_admitted_identity_and_preserves_refused_history() {
    isolated("tests::codex_model_admission_tests::cwb_shipping_compaction_uses_admitted_identity_and_preserves_refused_history", || {
        use mahayana_host_runtime::extensions::transcript::send_message_shaping::{
            UserMessageOptions, create_user_message,
        };
        use std::mem::discriminant;
        use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};

        #[derive(Clone, Copy)]
        enum Refusal { Workspace, MissingModel, MissingBinding, Cancelled }

        // Each phase has its own real Session and accepted lease. Prefix cases
        // supply only the successful first summary; their second callback is
        // the same production Inference consumer used by the primary cases.
        for phase in [0, 1] {
            for (label, refusal) in [
                ("workspace", Refusal::Workspace),
                ("missing-model", Refusal::MissingModel),
                ("missing-binding", Refusal::MissingBinding),
                ("cancelled", Refusal::Cancelled),
            ] {
                write_credentials(&credential_path(), WORKSPACE_A, WORKSPACE_A);
                let login_a = credential_image().unwrap();
                let fixture = CodexAdmissionFixture::new_named(&format!("owners-compaction-{label}-{phase}"));
                let history = [
                    create_user_message("history-user", "Establish the history fact.", UserMessageOptions::default(), 1.0),
                    json!({
                        "id":"history-assistant", "kind":"message", "role":"assistant",
                        "content":"The history fact is established.",
                    }),
                ];
                assert_eq!(fixture.sessions.append_agent_transcript_entries(&fixture.agent_id, &history).unwrap(), 2);
                let stream = format!("{}-{label}-{phase}", super::COMPACTION_TEST_TURN);
                let accepted = fixture.accept_with_prompt(&stream, super::COMPACTION_TEST_REQUEST);
                let admitted = fixture.admit(&stream, RoutedProvider::Codex, None, true);
                assert_eq!(admitted.codex_model.as_deref(), Some("Codex/Admitted@A"));
                assert!(admitted.codex_request_binding.is_some());
                assert!(admitted.fabushi_model.is_none() && admitted.fabushi_request_binding.is_none());
                let cancellation = admitted.lease_guard.control.cancellation();
                assert!(!cancellation.is_cancelled());
                assert_eq!(fixture.sessions.append_agent_transcript_entries(&fixture.agent_id, &[json!({
                    "id":"retained-assistant", "kind":"message", "role":"assistant",
                    // This crosses the shipping 8,000-token retained-tail
                    // budget after the actually accepted current user entry.
                    "content":"r".repeat(32_004),
                })]).unwrap(), 1);

                // Current settings deliberately disagree with the admitted
                // route/model. Missing identity must not recapture either.
                assert!(fixture.settings.set_inference_provider("openrouter").unwrap());
                assert!(fixture.settings.set_agent_default_model(Some(&json!({
                    "provider":"codex", "modelId":"Codex/Current@B", "maxMode":true, "parameters":[],
                }))).unwrap());
                let settings_path = fixture.root.join("settings.json");
                let settings_bytes = fs::read(&settings_path).unwrap();
                let owner = fixture.sessions.open_agent_db_owner(&fixture.agent_id).unwrap();
                let snapshot = || super::PostTurnCompactionSnapshot {
                    entries: owner.get_transcript_entries().unwrap(),
                    metadata: owner.read_kv("metadata").unwrap(),
                    admission: owner.read_kv("sessionWorkAdmission").unwrap(),
                    write_generation: super::get_sand_agent_db_write_generation(owner.db_path()),
                };
                let before = snapshot();
                let plan = shipping::prepare_compaction_plan(
                    &fixture.sessions.read_agent_transcript_entries(&fixture.agent_id).unwrap(),
                    shipping::SAND_AGENT_TOKEN_LIMIT as u64, shipping::SAND_AGENT_TOKEN_LIMIT as u64,
                    shipping::CompactionSettings { enabled:true, reserve_tokens:16_000, keep_recent_tokens:8_000 },
                    None,
                ).expect("actual accepted history must qualify for shipping compaction");
                assert!(plan.is_split_turn);
                assert_eq!(plan.first_kept_entry_id, "retained-assistant");
                assert!(plan.turn_prefix_entries.iter().any(|entry|
                    entry.get("id").and_then(Value::as_str) == accepted.user_message_id.as_deref()
                ), "the split prefix must contain this admission's durable user entry");

                let counts: Arc<[AtomicUsize; 3]> = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
                let publications = super::PostTurnCompactionPublications {
                    counts: Arc::clone(&counts),
                    _metadata: std::array::from_fn(|index| {
                        let counts = Arc::clone(&counts);
                        owner.subscribe_metadata(["lifecycleRevision", "sessionWorkAdmission"][index], Arc::new(move || {
                            counts[index].fetch_add(1, Ordering::SeqCst);
                        }))
                    }),
                    transcript: Some(super::subscribe_transcript_mutations({
                        let counts = Arc::clone(&counts);
                        let agent_id = fixture.agent_id.clone();
                        move |mutation| {
                            if mutation.get("agentId").and_then(Value::as_str) == Some(agent_id.as_str()) {
                                counts[2].fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    })),
                };
                let callback = shipping::compose_admitted_turn_summarization_prompt(
                    Arc::clone(&fixture.inference), fixture.root.clone(), RoutedProvider::Codex,
                    admitted.fabushi_model.clone(),
                    if matches!(refusal, Refusal::MissingModel | Refusal::Cancelled) { None } else { admitted.codex_model.clone() },
                    if matches!(refusal, Refusal::MissingBinding | Refusal::Cancelled) { None } else { admitted.codex_request_binding.clone() },
                    admitted.fabushi_request_binding.clone(),
                );
                let expected_login = match refusal {
                    Refusal::Workspace => {
                        write_credentials(&credential_path(), WORKSPACE_B, WORKSPACE_B);
                        let login_b = credential_image();
                        if phase == 1 {
                            // Prepare B's exact synthetic image, then restore A
                            // until the controlled primary has succeeded.
                            write_private(&credential_path(), &login_a);
                        }
                        login_b
                    }
                    Refusal::Cancelled => {
                        if phase == 0 { fs::remove_file(credential_path()).unwrap(); }
                        None
                    }
                    Refusal::MissingModel | Refusal::MissingBinding => Some(login_a.clone()),
                };
                let errors = Arc::new(Mutex::new(Vec::new()));
                let observed_errors = Arc::clone(&errors);
                let synthetic_primaries = Arc::new(AtomicUsize::new(0));
                let observed_primaries = Arc::clone(&synthetic_primaries);
                let observed_cancellation = cancellation.clone();
                let callback_login = expected_login.clone();
                let (runner, calls) = super::post_turn_compaction_runner_for_context(
                    &fixture.root, &fixture.agent_id, &cancellation,
                    move |index, system, user, should_cancel| {
                        if index < phase {
                            assert_eq!(index, 0);
                            assert_eq!(credential_image(), Some(login_a.clone()));
                            observed_primaries.fetch_add(1, Ordering::SeqCst);
                            match refusal {
                                Refusal::Workspace => write_private(&credential_path(), callback_login.as_deref().unwrap()),
                                Refusal::Cancelled => fs::remove_file(credential_path()).unwrap(),
                                Refusal::MissingModel | Refusal::MissingBinding => {},
                            }
                            // This one controlled response only qualifies the
                            // real split-prefix consumer; it is not provider HTTP.
                            return Ok(super::COMPACTION_HISTORY_SUMMARY.into());
                        }
                        assert_eq!(index, phase);
                        if matches!(refusal, Refusal::Cancelled) {
                            assert!(observed_cancellation.cancel("cancel admitted compaction before credential lookup"));
                        }
                        let result = callback(system, user, should_cancel);
                        let error = result.as_ref().expect_err("the actual admitted-identity consumer must refuse");
                        for secret in ["synthetic-cwb", WORKSPACE_A, WORKSPACE_B] {
                            assert!(!error.to_string().contains(secret));
                        }
                        observed_errors.lock().unwrap().push((index, discriminant(error)));
                        // Record the typed error, then pass the original Result
                        // unchanged through Runner into the shipping producer.
                        result
                    },
                );
                shipping::run_post_turn_compaction(
                    &runner, &fixture.sessions, &fixture.agent_id, &stream,
                    shipping::SAND_AGENT_TOKEN_LIMIT as u64, &|| cancellation.is_cancelled(),
                );
                let expected_error = match refusal {
                    Refusal::Workspace => ProviderSessionError::Authentication(String::new()),
                    Refusal::MissingModel | Refusal::MissingBinding => ProviderSessionError::Configuration(String::new()),
                    Refusal::Cancelled => ProviderSessionError::Cancelled(String::new()),
                };
                assert_eq!(*errors.lock().unwrap(), vec![(phase, discriminant(&expected_error))]);
                assert_eq!(calls.load(Ordering::SeqCst), phase + 1);
                assert_eq!(synthetic_primaries.load(Ordering::SeqCst), phase);
                assert_eq!(snapshot(), before, "a refused summary must not commit or replace accepted history");
                assert_eq!(publications.counts(), [0, 0, 0]);
                assert_eq!(credential_image(), expected_login);
                assert_eq!(fs::read(&settings_path).unwrap(), settings_bytes);
                assert!(fixture.runtime.has_routed_turn_lease(&fixture.agent_id, &stream));
                assert_eq!(fixture.runtime.in_flight_run_count(&fixture.agent_id), 1);
                assert!(!fixture.runtime.is_turn_dispatch_idle(&fixture.agent_id));
                assert_eq!(fixture.permission.direction_epoch(&fixture.agent_id), admitted.direction_epoch);
                assert_eq!(cancellation.is_cancelled(), matches!(refusal, Refusal::Cancelled));
                let direction = admitted.direction_epoch;
                drop(admitted);
                fixture.assert_idle(&stream, direction);
                assert_eq!(snapshot(), before);
                assert_eq!(publications.counts(), [0, 0, 0]);
                // The owned parent additionally requires zero attempted proxy
                // connections before accepting this child's terminal receipt.
            }
        }
    });
}
