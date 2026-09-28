use std::sync::{Arc, Mutex};

use crate::extensions::extension_ids_generated::HostExtensionId;

pub const INFERENCE_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Experiments,
    HostExtensionId::Settings,
];

pub fn inference_extension_id() -> HostExtensionId {
    HostExtensionId::Inference
}

pub trait AgentInferenceOwner: Send + Sync {}

pub type ModelExperimentListener = Arc<dyn Fn() + Send + Sync>;

pub struct InferenceExtensionRuntime<P> {
    port: P,
    peek_access_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    listeners: Mutex<Vec<ModelExperimentListener>>,
}

impl<P> InferenceExtensionRuntime<P> {
    pub fn new(
        port: P,
        peek_access_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    ) -> Self {
        Self {
            port,
            peek_access_token,
            listeners: Mutex::new(Vec::new()),
        }
    }

    pub fn is_ready(&self, mock_response_configured: bool) -> bool {
        mock_response_configured || (self.peek_access_token)().is_some()
    }

    pub fn port(&self) -> &P {
        &self.port
    }

    pub fn on_model_experiment_applied(&self, listener: ModelExperimentListener) {
        self.listeners
            .lock()
            .expect("inference experiment listeners")
            .push(listener);
    }

    pub fn notify_model_experiment_applied(&self) {
        let listeners = self
            .listeners
            .lock()
            .expect("inference experiment listeners")
            .clone();
        for listener in listeners {
            listener();
        }
    }
}
