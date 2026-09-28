use std::sync::Arc;

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::experiments::HostExperimentsExtension;
use crate::extensions::settings::settings_service::SettingsService;

use super::cursor_web_tools::{
    CursorWebBackend, CursorWebBackendOptions, CursorWebFetchService, CursorWebSearchService,
    create_cursor_web_backend,
};
use super::extension::{AgentInferenceOwner, InferenceExtensionRuntime};
use super::inference_service::{
    HostInferenceService, InferenceRoute, InferenceSettings, InferenceUsage,
};
use super::provider_session::RoutedProvider;

pub type RequestIdObserver = Arc<dyn Fn(&str) + Send + Sync>;
pub type ModelExperimentApplied = Arc<dyn Fn() + Send + Sync>;

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

impl InferenceSettings for ProductionInferenceSettings {
    fn inference_provider(&self) -> RoutedProvider {
        RoutedProvider::parse(&self.settings.get_inference_provider())
            .unwrap_or(RoutedProvider::Cursor)
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

impl ProductionAgentInferenceOwner {
    pub fn configured_provider(&self) -> String {
        self.settings.get_inference_provider()
    }

    pub fn is_agent_network_enabled(&self) -> bool {
        self.experiments.is_agent_network_enabled()
    }

    pub fn has_cached_access_token(&self) -> bool {
        self.auth.peek_access_token().is_some()
    }
}

impl AgentInferenceOwner for ProductionAgentInferenceOwner {}

/// Shipping Grok-shaped inference extension surface.
///
/// Web search/fetch are constructed from the same authenticated Host auth owner
/// used by the rest of the production extension graph. No renderer-side or
/// compatibility runtime is introduced here.
pub struct ProductionInferenceExtension {
    runtime: InferenceExtensionRuntime<Arc<ProductionAgentInferenceOwner>>,
    auth: Arc<HostAuthExtension>,
    service: HostInferenceService<ProductionInferenceSettings>,
}

impl ProductionInferenceExtension {
    pub fn is_ready(&self) -> bool {
        self.runtime
            .is_ready(std::env::var_os("SAND_AGENT_MOCK_RESPONSE").is_some())
    }

    pub fn port(&self) -> &Arc<ProductionAgentInferenceOwner> {
        self.runtime.port()
    }

    pub fn on_model_experiment_applied(&self, listener: ModelExperimentApplied) {
        self.runtime.on_model_experiment_applied(listener);
    }

    pub fn notify_model_experiment_applied(&self) {
        self.runtime.notify_model_experiment_applied();
    }

    pub fn route(&self) -> InferenceRoute {
        self.service.route()
    }

    pub fn record_usage(&self, provider: RoutedProvider, usage: InferenceUsage) {
        self.service.record_usage(provider, usage);
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
    let service = HostInferenceService::new(ProductionInferenceSettings::new(settings));
    ProductionInferenceExtension {
        runtime: InferenceExtensionRuntime::new(
            owner,
            Arc::new(move || ready_auth.peek_access_token()),
        ),
        auth,
        service,
    }
}
