use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::browser_ua::extension::StopSubscription;
use crate::extensions::experiments::HostExperimentsExtension;
use crate::extensions::settings::settings_service::SettingsService;

use super::cursor_inference_transport::CursorInferenceAuth;
use super::cursor_session::{
    RequestLineage, RequestedModel, ResolveRequestedModelInputs, SandAttachedMediaUrlProvider,
    SAND_SUMMARIZATION_MODEL_ID, SandSessionOptions, resolve_sand_requested_model,
};
use super::generated_inference_codec::InferenceReason;
use super::sand_labeling::{
    FollowupLabelingTracker, LabelMessage, SandLabelingClient, prepare_post_turn_labeling,
};
use super::sand_model_experiment::{
    SandAgentModelParameter, SandAgentModelSelection, select_sand_experiment_turn_model,
};
use super::cursor_web_tools::{
    CursorWebBackend, CursorWebBackendOptions, CursorWebFetchService, CursorWebSearchService,
    create_cursor_web_backend,
};
use super::extension::{AgentInferenceOwner, InferenceExtensionRuntime};
use super::inference_service::{
    HostInferenceService, InferenceRoute, InferenceSettings, InferenceUsage,
};
use super::provider_session::{
    CodexResponsesRequestBinding, FabushiResponsesRequestBinding,
    ProviderMessage, ProviderSessionError,
    ProviderTokenUsage, RoutedProvider,
    RoutedProviderCheckpoint, RoutedProviderOptions, RoutedToolDefinition,
    capture_codex_responses_request_binding,
    resolve_codex_request_model, resolve_fabushi_request_model, run_routed_provider_text,
    run_routed_provider_text_with_lifecycle_reporting_usage,
};

#[path = "summary_job.rs"]
mod summary_job;
pub use summary_job::SummaryInferenceJob;
#[cfg(test)]
pub(crate) use summary_job::SummaryAdmissionHookScope;

pub type RequestIdObserver = Arc<dyn Fn(&str) + Send + Sync>;
pub type ModelExperimentApplied = Arc<dyn Fn() + Send + Sync>;

pub fn summarization_session_options() -> SandSessionOptions {
    SandSessionOptions {
        model_id: Some(SAND_SUMMARIZATION_MODEL_ID.to_string()),
        is_summarization_session: true,
        skip_labeling: true,
        ..SandSessionOptions::default()
    }
}

pub trait InferenceAuth: Send + Sync {
    fn access_token(&self) -> Result<String, String>;
    fn machine_id(&self) -> String;
}

pub trait InferencePortFactory: Send + Sync {
    fn create_host_inference(
        &self,
        auth: Arc<dyn InferenceAuth>,
        on_model_experiment_applied: ModelExperimentApplied,
    ) -> Arc<dyn AgentInferenceOwner>;
}

pub trait CursorWebBackendFactory: Send + Sync {
    fn create_backend(
        &self,
        auth: Arc<dyn InferenceAuth>,
        on_request_id: Option<RequestIdObserver>,
    ) -> Arc<dyn CursorWebBackend>;
}

#[derive(Clone)]
pub struct InferenceProductionExtras {
    auth: Arc<dyn InferenceAuth>,
    port_factory: Arc<dyn InferencePortFactory>,
    web_backend_factory: Arc<dyn CursorWebBackendFactory>,
}

impl InferenceProductionExtras {
    pub fn new(
        auth: Arc<dyn InferenceAuth>,
        port_factory: Arc<dyn InferencePortFactory>,
        web_backend_factory: Arc<dyn CursorWebBackendFactory>,
    ) -> Self {
        Self { auth, port_factory, web_backend_factory }
    }

    pub fn create_port(
        &self,
        on_model_experiment_applied: ModelExperimentApplied,
    ) -> Arc<dyn AgentInferenceOwner> {
        self.port_factory.create_host_inference(
            Arc::clone(&self.auth),
            on_model_experiment_applied,
        )
    }

    pub fn create_web_search(
        &self,
        model_id: impl Into<String>,
        on_request_id: Option<RequestIdObserver>,
    ) -> CursorWebSearchService {
        CursorWebSearchService::new(
            self.web_backend_factory.create_backend(Arc::clone(&self.auth), on_request_id),
            model_id,
        )
    }

    pub fn create_web_fetch(
        &self,
        on_request_id: Option<RequestIdObserver>,
    ) -> CursorWebFetchService {
        CursorWebFetchService::new(
            self.web_backend_factory.create_backend(Arc::clone(&self.auth), on_request_id),
        )
    }
}

/// Canonical production inference owner installed in the Host extension registry.
///
/// The Runner remains the owner of turn execution.  This object only keeps the
/// frozen Host inference dependencies together and exposes the live provider /
/// experiment state needed by Host-side inference surfaces.
#[derive(Clone)]
pub struct ProductionInferenceSettings {
    settings: Arc<SettingsService>,
}

impl ProductionInferenceSettings {
    pub fn new(settings: Arc<SettingsService>) -> Self {
        Self { settings }
    }
}

fn resolve_production_inference_provider(
    fabushi_product_mode: Option<&str>,
    packaged: Option<&str>,
    configured: &str,
) -> RoutedProvider {
    if fabushi_product_mode == Some("1") || packaged == Some("1") {
        return RoutedProvider::Fabushi;
    }
    RoutedProvider::parse(configured).unwrap_or(RoutedProvider::Cursor)
}

impl InferenceSettings for ProductionInferenceSettings {
    fn inference_provider(&self) -> RoutedProvider {
        resolve_production_inference_provider(
            std::env::var("FABUSHI_PRODUCT_MODE").ok().as_deref(),
            std::env::var("SAND_PACKAGED").ok().as_deref(),
            &self.settings.get_inference_provider(),
        )
    }

    fn record_inference_usage(&self, provider: RoutedProvider, usage: InferenceUsage) {
        if let Err(error) = self.settings.record_inference_usage(
            provider.as_str(),
            usage.input_tokens.map(|value| value as f64),
            usage.output_tokens.map(|value| value as f64),
            usage.cache_read_tokens.map(|value| value as f64),
            usage.cache_write_tokens.map(|value| value as f64),
        ) {
            eprintln!(
                "mahayana-host inference_usage_persist_failed provider={} error={error}",
                provider.as_str(),
            );
        }
    }
}

pub struct ProductionAgentInferenceOwner {
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
}

fn model_selection_from_value(value: Option<serde_json::Value>) -> Option<SandAgentModelSelection> {
    let value = value?;
    let object = value.as_object()?;
    let model_id = object.get("modelId")?.as_str()?.trim();
    if model_id.is_empty() {
        return None;
    }
    let max_mode = object
        .get("maxMode")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let parameters = object
        .get("parameters")
        .and_then(serde_json::Value::as_array)
        .map(|parameters| {
            parameters
                .iter()
                .filter_map(|parameter| {
                    let parameter = parameter.as_object()?;
                    let id = parameter.get("id")?.as_str()?.trim();
                    let value = parameter.get("value")?.as_str()?.trim();
                    if id.is_empty() || value.is_empty() {
                        return None;
                    }
                    Some(SandAgentModelParameter {
                        id: id.to_string(),
                        value: value.to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(SandAgentModelSelection {
        model_id: model_id.to_string(),
        max_mode,
        parameters,
    })
}

fn cursor_default_model(value: Option<serde_json::Value>) -> Option<SandAgentModelSelection> {
    let value = value?;
    match value.get("provider") {
        None => model_selection_from_value(Some(value)),
        Some(serde_json::Value::String(provider)) if provider == "cursor" => {
            model_selection_from_value(Some(value))
        }
        _ => None,
    }
}

impl ProductionAgentInferenceOwner {
    pub fn configured_provider(&self) -> String {
        self.settings.get_inference_provider()
    }

    /// Resolve the Fabushi identity once at turn admission. The selected route
    /// remains the caller's independently authorized provider.
    pub fn resolve_fabushi_turn_model(
        &self,
        session_model_id: Option<&str>,
        session_provider: Option<&str>,
        explicit_model_id: Option<&serde_json::Value>,
    ) -> Result<String, ProviderSessionError> {
        let explicit = match explicit_model_id {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(value)) => {
                Some(resolve_fabushi_request_model(Some(value), None)?)
            }
            Some(_) => return Err(ProviderSessionError::Configuration(
                "Fabushi modelId must be a nonempty string when supplied".into(),
            )),
        };
        match (session_model_id, session_provider) {
            (Some(model), Some("fabushi")) => {
                return resolve_fabushi_request_model(Some(model), None);
            }
            (None, None) => {}
            _ => return Err(ProviderSessionError::Configuration(
                "persisted session model must contain a complete Fabushi provider/model pair".into(),
            )),
        }
        if let Some(model) = explicit {
            return Ok(model);
        }
        let stored = self.settings.get_agent_default_model_checked().map_err(|error| {
            ProviderSessionError::Configuration(format!(
                "could not read the persisted default model: {error}"
            ))
        })?;
        if let Some(value) = stored.as_ref().filter(|value| {
            value.get("provider").and_then(serde_json::Value::as_str) == Some("fabushi")
        }) {
            let model = value.get("modelId").and_then(serde_json::Value::as_str)
                .ok_or_else(|| ProviderSessionError::Configuration(
                    "persisted Fabushi default is missing modelId".into(),
                ))?;
            return resolve_fabushi_request_model(Some(model), None);
        }
        resolve_fabushi_request_model(None, None)
    }

    /// Resolve the already-authorized Codex route's identity once at admission.
    /// A persisted model selection never changes the caller's provider route.
    pub fn resolve_codex_turn_model(
        &self,
        session_model_id: Option<&str>,
        session_provider: Option<&str>,
        explicit_model_id: Option<&serde_json::Value>,
    ) -> Result<String, ProviderSessionError> {
        let explicit = match explicit_model_id {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(value)) => {
                Some(resolve_codex_request_model(Some(value), None)?)
            }
            Some(_) => return Err(ProviderSessionError::Configuration(
                "Codex modelId must be a nonempty string when supplied".into(),
            )),
        };
        match (session_model_id, session_provider) {
            (Some(model), Some("codex")) => {
                return resolve_codex_request_model(Some(model), None);
            }
            (None, None) => {}
            _ => return Err(ProviderSessionError::Configuration(
                "persisted session model must contain a complete Codex provider/model pair".into(),
            )),
        }
        if let Some(model) = explicit {
            return Ok(model);
        }
        let stored = self.settings.get_agent_default_model_checked().map_err(|error| {
            ProviderSessionError::Configuration(format!(
                "could not read the persisted default model: {error}"
            ))
        })?;
        if let Some(value) = stored.as_ref().filter(|value| {
            value.get("provider").and_then(serde_json::Value::as_str) == Some("codex")
        }) {
            let model = value.get("modelId").and_then(serde_json::Value::as_str)
                .ok_or_else(|| ProviderSessionError::Configuration(
                    "persisted Codex default is missing modelId".into(),
                ))?;
            return resolve_codex_request_model(Some(model), None);
        }
        resolve_codex_request_model(None, None)
    }

    pub fn resolve_requested_model(
        &self,
        session_options: Option<&SandSessionOptions>,
    ) -> RequestedModel {
        let stored_default_model = cursor_default_model(self.settings.get_agent_default_model());
        let configured_computer_use_model = self
            .experiments
            .get_computer_use_model_override()
            .and_then(|config| {
                model_selection_from_value(Some(serde_json::Value::Object(
                    config.into_iter().collect(),
                )))
            });
        let stored_computer_use_model =
            model_selection_from_value(self.settings.get_computer_use_model())
                .or(configured_computer_use_model);
        let configured_browser_use_model = self
            .experiments
            .get_browser_use_model_override()
            .and_then(|config| {
                model_selection_from_value(Some(serde_json::Value::Object(
                    config.into_iter().collect(),
                )))
            });
        let env_model_override = std::env::var("SAND_AGENT_MODEL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let experiment_state = self.experiments.get_sand_model_experiment_state();
        let request_source = session_options
            .and_then(|options| options.request_source.as_deref());
        let experiment_model_override = select_sand_experiment_turn_model(
            experiment_state.as_ref(),
            request_source,
            || self.experiments.get_configured_default_model(),
            || self.experiments.get_configured_automations_model(),
        );
        if experiment_model_override.is_some() {
            let _ = self.experiments.log_sand_model_experiment_exposure();
        }
        resolve_sand_requested_model(ResolveRequestedModelInputs {
            session_options,
            env_model_override: env_model_override.as_deref(),
            stored_default_model: stored_default_model.as_ref(),
            stored_computer_use_model: stored_computer_use_model.as_ref(),
            stored_browser_use_model: configured_browser_use_model.as_ref(),
            experiment_model_override: experiment_model_override.as_ref(),
        })
    }

    pub fn is_agent_network_enabled(&self) -> bool {
        self.experiments.is_agent_network_enabled()
    }

    pub fn has_cached_access_token(&self) -> bool {
        self.auth.peek_access_token().is_some()
    }
}

impl AgentInferenceOwner for ProductionAgentInferenceOwner {}

#[derive(Clone)]
struct ProductionCursorInferenceAuth {
    auth: Arc<HostAuthExtension>,
    requested_model: RequestedModel,
    session_options: SandSessionOptions,
    labeling_client: Arc<SandLabelingClient>,
    labeling_tracker: Arc<Mutex<FollowupLabelingTracker>>,
    last_request_id: Arc<Mutex<Option<String>>>,
}

impl CursorInferenceAuth for ProductionCursorInferenceAuth {
    fn access_token(&self) -> Result<String, String> {
        self.auth.get_access_token().map_err(|error| error.to_string())
    }

    fn machine_id(&self) -> Result<String, String> {
        self.auth.get_machine_id().map_err(|error| error.to_string())
    }

    fn requested_model(&self) -> RequestedModel {
        self.requested_model.clone()
    }

    fn conversation_id(&self) -> Option<String> {
        self.session_options.conversation_id.clone()
    }

    fn inference_reason(&self) -> Option<InferenceReason> {
        self.session_options.inference_reason
    }

    fn lineage(&self) -> Option<RequestLineage> {
        self.session_options.lineage.clone()
    }

    fn on_request_id(&self, request_id: &str, messages: &[super::provider_session::ProviderMessage]) {
        *self
            .last_request_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(request_id.to_string());
        if self.session_options.skip_labeling
            || self.session_options.is_summarization_session
            || self.session_options.is_computer_use_subagent
        {
            return;
        }
        let Some(conversation_id) = self.session_options.conversation_id.as_deref() else {
            return;
        };
        let markers = messages
            .iter()
            .map(|_| LabelMessage::default())
            .collect::<Vec<_>>();
        let classification = self
            .labeling_tracker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_stream(
                conversation_id,
                request_id,
                &self.requested_model.model_id,
                &markers,
            );
        if let Some(classification) = classification {
            self.labeling_client
                .spawn_followup_classification(classification, messages.to_vec());
        }
    }

    fn record_post_turn_labeling(&self, messages: &[super::provider_session::ProviderMessage]) {
        if self.session_options.skip_labeling
            || self.session_options.is_summarization_session
            || self.session_options.is_computer_use_subagent
        {
            return;
        }
        let Some(conversation_id) = self.session_options.conversation_id.as_deref() else {
            return;
        };
        let request_id = self
            .last_request_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let Some(request_id) = request_id else {
            return;
        };
        let markers = messages
            .iter()
            .map(|_| LabelMessage::default())
            .collect::<Vec<_>>();
        if let Some(labeling) = prepare_post_turn_labeling(
            conversation_id,
            &request_id,
            &self.requested_model.model_id,
            &markers,
        ) {
            self.labeling_client
                .spawn_post_turn_labeling(labeling, messages.to_vec());
        }
    }
}

/// Shipping Fabushi inference extension surface.
///
/// Web search/fetch are constructed from the same authenticated Host auth owner
/// used by the rest of the production extension graph. No renderer-side or
/// compatibility runtime is introduced here.
pub struct ProductionInferenceExtension {
    runtime: InferenceExtensionRuntime<Arc<ProductionAgentInferenceOwner>>,
    auth: Arc<HostAuthExtension>,
    service: HostInferenceService<ProductionInferenceSettings>,
    labeling_client: Arc<SandLabelingClient>,
    labeling_tracker: Arc<Mutex<FollowupLabelingTracker>>,
    attached_media: SandAttachedMediaUrlProvider,
}

impl ProductionInferenceExtension {
    pub fn is_ready(&self) -> bool {
        self.runtime
            .is_ready(std::env::var_os("SAND_AGENT_MOCK_RESPONSE").is_some())
    }

    pub fn port(&self) -> &Arc<ProductionAgentInferenceOwner> {
        self.runtime.port()
    }

    pub fn on_model_experiment_applied(
        &self,
        listener: ModelExperimentApplied,
    ) -> StopSubscription {
        self.runtime.on_model_experiment_applied(listener)
    }

    pub fn notify_model_experiment_applied(&self) {
        self.runtime.notify_model_experiment_applied();
    }

    pub fn route(&self) -> InferenceRoute {
        self.service.route()
    }

    /// One checked Settings snapshot and the shipping inference route decide
    /// whether a native main-session reset has a supported configured default.
    /// Unsupported routes remain unsupported; this is not a model catalog.
    pub fn checked_session_model_default(&self) -> Result<Option<(String, String)>, String> {
        let (configured, default) = self.port().settings.get_session_model_source_checked()?;
        let route = resolve_production_inference_provider(
            std::env::var("FABUSHI_PRODUCT_MODE").ok().as_deref(),
            std::env::var("SAND_PACKAGED").ok().as_deref(),
            &configured,
        );
        let provider = match route {
            RoutedProvider::Fabushi => "fabushi",
            RoutedProvider::Codex => "codex",
            _ => return Ok(None),
        };
        let stored = default.as_ref()
            .filter(|value| value.get("provider").and_then(serde_json::Value::as_str) == Some(provider))
            .and_then(|value| value.get("modelId").and_then(serde_json::Value::as_str));
        let model = match route {
            RoutedProvider::Fabushi => resolve_fabushi_request_model(stored, None),
            RoutedProvider::Codex => resolve_codex_request_model(stored, None),
            _ => unreachable!("supported provider already matched"),
        }.map_err(|error| error.to_string())?;
        Ok(Some((provider.to_string(), model)))
    }

    pub fn record_usage(&self, provider: RoutedProvider, usage: InferenceUsage) {
        self.service.record_usage(provider, usage);
    }

    pub fn cursor_auth_for_session(
        &self,
        session_options: Option<&SandSessionOptions>,
    ) -> Arc<dyn CursorInferenceAuth> {
        let session_options = session_options.cloned().unwrap_or_default();
        let owner = self.runtime.port();
        let requested_model = owner.resolve_requested_model(Some(&session_options));
        Arc::new(ProductionCursorInferenceAuth {
            auth: Arc::clone(&self.auth),
            requested_model,
            session_options,
            labeling_client: Arc::clone(&self.labeling_client),
            labeling_tracker: Arc::clone(&self.labeling_tracker),
            last_request_id: Arc::new(Mutex::new(None)),
        })
    }

    pub fn attached_media_provider(&self) -> SandAttachedMediaUrlProvider {
        self.attached_media.clone()
    }

    /// Turn-memory follows the already-admitted route, model and account
    /// binding. It must not resolve a later default or create a new turn.
    pub fn run_turn_memory_prompt(
        &self,
        data_dir: &Path,
        provider: RoutedProvider,
        fabushi_model: Option<&str>,
        codex_model: Option<&str>,
        codex_request_binding: Option<&CodexResponsesRequestBinding>,
        fabushi_request_binding: Option<&FabushiResponsesRequestBinding>,
        system_prompt: &str,
        user_prompt: &str,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled(
                "Runner cancelled before provider dispatch".into(),
            ));
        }
        if provider == RoutedProvider::Fabushi && fabushi_request_binding.is_none() {
            return Err(ProviderSessionError::Configuration(
                "Native turn-memory inference requires its admitted request binding.".into(),
            ));
        }
        if provider == RoutedProvider::Codex && codex_model.is_none() {
            return Err(ProviderSessionError::Configuration(
                "Codex turn-memory inference requires its admitted model identity.".into(),
            ));
        }
        if provider == RoutedProvider::Codex && codex_request_binding.is_none() {
            return Err(ProviderSessionError::Configuration(
                "Codex turn-memory inference requires its admitted request binding.".into(),
            ));
        }
        let messages = vec![
            ProviderMessage { role: "system".into(), content: system_prompt.to_string() },
            ProviderMessage { role: "user".into(), content: user_prompt.to_string() },
        ];
        let mut reject_tool = |
            _tool: &RoutedToolDefinition,
            _args: serde_json::Value,
            _tool_call_id: &str,
        | -> Result<serde_json::Value, ProviderSessionError> {
            Err(ProviderSessionError::Tool(
                "turn-memory inference exposes no tools".into(),
            ))
        };
        let mut ignore_delta = |_delta: &str, _accumulated: &str| {};
        let mut options = RoutedProviderOptions {
            data_dir,
            typed_replay: None,
            fabushi_model,
            codex_model,
            codex_request_binding,
            fabushi_request_binding,
            cursor_auth: Some(self.auth.clone()),
            tools: &[],
            mcp_server_url: None,
            execute_tool: &mut reject_tool,
            on_text_delta: &mut ignore_delta,
            on_partial_tool_call: None,
            should_cancel,
        };
        run_routed_provider_text(provider, &messages, &mut options)
    }

    /// Automatic summaries belong to the already-admitted turn. Keep that
    /// caller's route, model and credential identity through every phase.
    pub fn run_turn_summarization_prompt(
        &self,
        data_dir: &Path,
        provider: RoutedProvider,
        fabushi_model: Option<&str>,
        codex_model: Option<&str>,
        codex_request_binding: Option<&CodexResponsesRequestBinding>,
        fabushi_request_binding: Option<&FabushiResponsesRequestBinding>,
        system_prompt: &str,
        user_prompt: &str,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled("summarization cancelled".into()));
        }
        if provider == RoutedProvider::Codex
            && (codex_model.is_none() || codex_request_binding.is_none())
        {
            return Err(ProviderSessionError::Configuration(
                "Codex turn summarization requires its admitted model and request binding.".into(),
            ));
        }
        if provider == RoutedProvider::Fabushi
            && (fabushi_model.is_none() || fabushi_request_binding.is_none())
        {
            return Err(ProviderSessionError::Configuration(
                "Native turn summarization requires its admitted model and request binding.".into(),
            ));
        }
        self.run_summarization_prompt_with_identity(
            data_dir, provider, fabushi_model, codex_model,
            codex_request_binding, fabushi_request_binding,
            system_prompt, user_prompt, should_cancel,
        )
    }

    /// An independent summary has its own current route/model/credential capture.
    pub fn run_summarization_prompt(
        &self,
        data_dir: &Path,
        system_prompt: &str,
        user_prompt: &str,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled("summarization cancelled".into()));
        }
        let provider = match self.route() {
            InferenceRoute::Cursor => RoutedProvider::Cursor,
            InferenceRoute::Routed(provider) => provider,
        };
        let fabushi_model = if provider == RoutedProvider::Fabushi {
            Some(self.runtime.port().resolve_fabushi_turn_model(None, None, None)?)
        } else {
            None
        };
        let codex_model = if provider == RoutedProvider::Codex {
            Some(self.runtime.port().resolve_codex_turn_model(None, None, None)?)
        } else {
            None
        };
        let codex_request_binding = if provider == RoutedProvider::Codex {
            if should_cancel() {
                return Err(ProviderSessionError::Cancelled("summarization cancelled".into()));
            }
            let binding = capture_codex_responses_request_binding();
            if should_cancel() {
                return Err(ProviderSessionError::Cancelled("summarization cancelled".into()));
            }
            Some(binding?)
        } else {
            None
        };
        self.run_summarization_prompt_with_identity(
            data_dir, provider, fabushi_model.as_deref(), codex_model.as_deref(),
            codex_request_binding.as_ref(), None, system_prompt, user_prompt, should_cancel,
        )
    }

    fn run_summarization_prompt_with_identity(
        &self,
        data_dir: &Path,
        provider: RoutedProvider,
        fabushi_model: Option<&str>,
        codex_model: Option<&str>,
        codex_request_binding: Option<&CodexResponsesRequestBinding>,
        fabushi_request_binding: Option<&FabushiResponsesRequestBinding>,
        system_prompt: &str,
        user_prompt: &str,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled("summarization cancelled".into()));
        }
        let session_options = summarization_session_options();
        let cursor_auth = self.cursor_auth_for_session(Some(&session_options));
        let messages = vec![
            ProviderMessage {
                role: "system".into(),
                content: system_prompt.to_string(),
            },
            ProviderMessage {
                role: "user".into(),
                content: user_prompt.to_string(),
            },
        ];
        let mut reject_tool = |
            _tool: &RoutedToolDefinition,
            _args: serde_json::Value,
            _tool_call_id: &str,
        | -> Result<serde_json::Value, ProviderSessionError> {
            Err(ProviderSessionError::Tool(
                "summarization inference exposes no tools".into(),
            ))
        };
        let mut ignore_delta = |_delta: &str, _accumulated: &str| {};
        let mut options = RoutedProviderOptions {
            data_dir,
            typed_replay: None,
            fabushi_model,
            codex_model,
            codex_request_binding,
            fabushi_request_binding,
            cursor_auth: Some(cursor_auth),
            tools: &[],
            mcp_server_url: None,
            execute_tool: &mut reject_tool,
            on_text_delta: &mut ignore_delta,
            on_partial_tool_call: None,
            should_cancel,
        };
        let mut ignore_checkpoint =
            |_checkpoint: &RoutedProviderCheckpoint| Ok(());
        let mut record_usage = |usage: ProviderTokenUsage| {
            self.record_usage(
                provider,
                InferenceUsage {
                    input_tokens: Some(usage.input_tokens),
                    output_tokens: Some(usage.output_tokens),
                    cache_read_tokens: Some(usage.cache_read_tokens),
                    cache_write_tokens: Some(usage.cache_write_tokens),
                },
            );
        };
        run_routed_provider_text_with_lifecycle_reporting_usage(
            provider,
            &messages,
            &mut options,
            None,
            &mut ignore_checkpoint,
            &mut record_usage,
        )
    }

    pub fn create_web_search(
        &self,
        model_id: impl Into<String>,
        on_request_id: Option<RequestIdObserver>,
    ) -> Result<CursorWebSearchService, String> {
        let auth: Arc<dyn super::cursor_web_tools::CursorWebAuth> = self.auth.clone();
        let options = CursorWebBackendOptions::production(auth, on_request_id)?;
        Ok(CursorWebSearchService::new(
            create_cursor_web_backend(options),
            model_id,
        ))
    }

    pub fn create_web_fetch(
        &self,
        on_request_id: Option<RequestIdObserver>,
    ) -> Result<CursorWebFetchService, String> {
        let auth: Arc<dyn super::cursor_web_tools::CursorWebAuth> = self.auth.clone();
        let options = CursorWebBackendOptions::production(auth, on_request_id)?;
        Ok(CursorWebFetchService::new(create_cursor_web_backend(options)))
    }
}

pub fn start_production_inference_extension(
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
) -> ProductionInferenceExtension {
    let owner = Arc::new(ProductionAgentInferenceOwner {
        auth: Arc::clone(&auth),
        experiments,
        settings: Arc::clone(&settings),
    });
    let ready_auth = Arc::clone(&auth);
    let labeling_client = Arc::new(SandLabelingClient::production(Arc::clone(&auth)));
    let attached_media = SandAttachedMediaUrlProvider::production(Arc::clone(&auth));
    let service = HostInferenceService::new(ProductionInferenceSettings::new(settings));
    ProductionInferenceExtension {
        runtime: InferenceExtensionRuntime::new(
            owner,
            Arc::new(move || ready_auth.peek_access_token()),
        ),
        auth,
        service,
        labeling_client,
        labeling_tracker: Arc::new(Mutex::new(FollowupLabelingTracker::default())),
        attached_media,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::auth::auth_service::{HostAuthEnvironment, HostAuthServiceOptions};
    use crate::extensions::auth::extension::start_host_auth_extension_with_options;
    use crate::extensions::experiments::extension::HostExperimentsOptions;
    use serde_json::json;

    fn inference_fixture(label: &str) -> (
        std::path::PathBuf,
        Arc<SettingsService>,
        ProductionInferenceExtension,
    ) {
        let root = std::env::temp_dir().join(format!("fabushi-model-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("fixture directory");
        let settings = Arc::new(SettingsService::new(root.join("settings.json")));
        let auth = Arc::new(start_host_auth_extension_with_options(
            HostAuthServiceOptions {
                environment: HostAuthEnvironment::from_pairs(std::iter::empty()),
                backend_url: Some("http://127.0.0.1:9".into()),
                backend: None,
                machine_id: Arc::new(|| Ok("model-contract-machine".into())),
                log: Arc::new(|_| {}),
                now_ms: Arc::new(|| 1),
            },
            Arc::new(|_| Err("model resolution must not fetch account identity".into())),
        ).expect("auth owner"));
        let experiments = Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
            is_dev_build: false,
            env_gate_overrides: None,
        }));
        let inference = start_production_inference_extension(auth, experiments, Arc::clone(&settings));
        (root, settings, inference)
    }

    fn save_default(settings: &SettingsService, provider: Option<&str>, model: &str) {
        let mut value = json!({"modelId": model, "maxMode": false, "parameters": []});
        if let Some(provider) = provider {
            value["provider"] = json!(provider);
        }
        assert!(settings.set_agent_default_model(Some(&value)).expect("save model"));
    }

    #[test]
    fn packaged_fabushi_is_pinned_to_first_party_inference() {
        for configured in ["cursor", "codex", "claude-code", "openrouter", "unknown"] {
            assert_eq!(
                resolve_production_inference_provider(None, Some("1"), configured),
                RoutedProvider::Fabushi,
            );
        }
    }

    #[test]
    fn explicit_product_mode_is_pinned_to_first_party_inference() {
        assert_eq!(
            resolve_production_inference_provider(Some("1"), None, "cursor"),
            RoutedProvider::Fabushi,
        );
    }

    #[test]
    fn development_mode_preserves_explicit_router_selection() {
        assert_eq!(
            resolve_production_inference_provider(None, None, "openrouter"),
            RoutedProvider::OpenRouter,
        );
    }
    #[test]
    fn native_responses_preserves_unconfigured_default_and_binds_explicit_turn_models() {
        let (root, settings, inference) = inference_fixture("selection");
        let owner = inference.port();
        let configured_route = owner.configured_provider();
        let fallback = resolve_fabushi_request_model(None, None).expect("configured fallback");
        assert_eq!(owner.resolve_fabushi_turn_model(None, None, None).unwrap(), fallback);
        save_default(&settings, None, "legacy-cursor-model");
        assert_eq!(owner.resolve_fabushi_turn_model(None, None, None).unwrap(), fallback);
        assert_eq!(cursor_default_model(settings.get_agent_default_model()).unwrap().model_id,
            "legacy-cursor-model");
        for provider in ["cursor", "codex", "claude-code", "openrouter"] {
            save_default(&settings, Some(provider), "Other/Model@Version");
            assert_eq!(owner.resolve_fabushi_turn_model(None, None, None).unwrap(), fallback);
        }
        save_default(&settings, Some("fabushi"), "  Vendor/Nested/Model@Version  ");
        assert_eq!(owner.resolve_fabushi_turn_model(None, None, None).unwrap(),
            "Vendor/Nested/Model@Version");
        assert!(cursor_default_model(settings.get_agent_default_model()).is_none());
        let explicit = json!("  Explicit/Case@2 ");
        assert_eq!(owner.resolve_fabushi_turn_model(None, None, Some(&explicit)).unwrap(),
            "Explicit/Case@2");
        assert_eq!(owner.resolve_fabushi_turn_model(
            Some(" Session/Model@Q "), Some("fabushi"), Some(&explicit),
        ).unwrap(), "Session/Model@Q");
        assert_eq!(owner.configured_provider(), configured_route);
        assert_eq!(inference.cursor_auth_for_session(Some(&summarization_session_options()))
            .requested_model().model_id, SAND_SUMMARIZATION_MODEL_ID);
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn fabushi_model_resolution_rejects_malformed_pairs_and_raw_fields() {
        let (root, settings, inference) = inference_fixture("refusal");
        save_default(&settings, Some("fabushi"), "Valid/Default");
        let owner = inference.port();
        for (model, provider) in [
            (Some("model"), None), (None, Some("fabushi")),
            (Some("model"), Some("cursor")), (Some("model"), Some("FABUSHI")),
            (Some(""), Some("fabushi")), (Some("model\n"), Some("fabushi")),
        ] {
            assert!(matches!(owner.resolve_fabushi_turn_model(model, provider, None),
                Err(ProviderSessionError::Configuration(_))));
        }
        for raw in [json!(false), json!(17), json!([]), json!({}), json!("  "), json!("\nmodel")] {
            assert!(matches!(owner.resolve_fabushi_turn_model(None, None, Some(&raw)),
                Err(ProviderSessionError::Configuration(_))));
            assert!(matches!(owner.resolve_fabushi_turn_model(Some("valid"), Some("fabushi"), Some(&raw)),
                Err(ProviderSessionError::Configuration(_))), "persisted pair must not hide malformed input");
        }
        assert_eq!(owner.resolve_fabushi_turn_model(None, None, Some(&serde_json::Value::Null)).unwrap(),
            "Valid/Default");
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn fabushi_model_resolution_distinguishes_bad_settings_from_absence_and_unused_default() {
        let (root, settings, inference) = inference_fixture("source");
        save_default(&settings, Some("fabushi"), "Valid/Default");
        let path = root.join("settings.json");
        let mut document: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["agentDefaultModel"]["provider"] = serde_json::Value::Null;
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(matches!(inference.port().resolve_fabushi_turn_model(None, None, None),
            Err(ProviderSessionError::Configuration(_))));
        std::fs::write(&path, "{").unwrap();
        assert!(matches!(inference.port().resolve_fabushi_turn_model(None, None, None),
            Err(ProviderSessionError::Configuration(_))));
        assert_eq!(inference.port().resolve_fabushi_turn_model(None, None, Some(&json!("Explicit"))).unwrap(),
            "Explicit");
        assert_eq!(inference.port().resolve_fabushi_turn_model(Some("Session"), Some("fabushi"), None).unwrap(),
            "Session");
        assert!(matches!(inference.run_summarization_prompt(&root, "system", "user", &|| true),
            Err(ProviderSessionError::Cancelled(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{");
        std::fs::remove_dir_all(root).expect("cleanup");
    }

}
