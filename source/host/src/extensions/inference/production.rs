use std::sync::Arc;

use super::cursor_web_tools::{CursorWebBackend, CursorWebFetchService, CursorWebSearchService};
use super::extension::AgentInferenceOwner;

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
