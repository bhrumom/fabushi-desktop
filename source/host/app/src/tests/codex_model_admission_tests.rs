//! CMI-v1 admission controls use the actual Settings, Session, Transcript lease
//! and permission owners. Provider transport/retry evidence lives in the
//! separately selected private Codex consumer tests, not in this fixture.

use super::super as shipping;
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
use mahayana_host_runtime::extensions::local_tool_permission::local_tool_permission_controller::SandLocalToolPermissionController;
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use serde_json::{Value, json};
use shipping::{GatewayEventHub, ProductionSessionWorkers, ProductionTranscriptRuntime, RoutedProvider, SettingsService};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

struct CodexAdmissionFixture {
    root: PathBuf,
    settings: Arc<SettingsService>,
    inference: ProductionInferenceExtension,
    sessions: Arc<ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    permission: SandLocalToolPermissionController,
    events: GatewayEventHub,
    agent_id: String,
}

impl CodexAdmissionFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("cmi-admission-{}", uuid::Uuid::new_v4()));
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
        let inference = start_production_inference_extension(
            auth,
            Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
                is_dev_build: false, env_gate_overrides: None,
            })),
            Arc::clone(&settings),
        );
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent_id = sessions.materialize_new_session(None, "user", None).unwrap().id;
        let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
        let permission = SandLocalToolPermissionController::production(Arc::clone(&settings));
        Self { root, settings, inference, sessions, runtime, permission,
            events: GatewayEventHub::default(), agent_id }
    }

    fn accept(&self, stream: &str) -> PersistedSendContext {
        let args = json!({
            "agentId":&self.agent_id, "prompt":"Codex admission", "streamId":stream,
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
        if !std::thread::panicking() {
            fs::remove_dir_all(&self.root).expect("retired admission fixture cleanup");
        }
    }
}

#[test]
fn cmi_shipping_admission_freezes_saved_model_and_keeps_native_recovery_isolated() {
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
}

#[test]
fn cmi_shipping_refusal_releases_lease_without_permission_or_settings_effects() {
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
}

#[test]
fn cmi_shipping_session_precedence_preserves_raw_validation_and_other_routes() {
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
}

#[test]
fn cmi_shipping_cancellation_before_and_after_model_capture_preserves_admission_owners() {
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
}
