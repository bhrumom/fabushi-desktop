use std::path::Path;
use std::sync::Arc;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider,
};

use super::box_tool_access::{RunnerBoxResourcePort, RunnerBoxToolBridge};
use super::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use super::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedProviderRun, RoutedToolBridge,
    RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};

/// Shipping Runner composition for one provider-backed turn.
///
/// This module deliberately owns the dependency assembly that used to be
/// hand-built by the Host entrypoint: provider selection, Host tool bridge,
/// request-context projection, cancellation, and the durable checkpoint store.
/// Provider execution itself remains in the Runner-owned routed-provider
/// runtime so the Host process does not absorb Runner responsibilities.
#[derive(Clone)]
pub struct TurnAgentComposition {
    provider: RoutedProvider,
    bridge: Arc<dyn RoutedToolBridge>,
    request_context: RunnerRequestContextSnapshot,
    cancellation: RoutedProviderCancellation,
    checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
}

impl TurnAgentComposition {
    pub fn new(
        provider: RoutedProvider,
        bridge: Arc<dyn RoutedToolBridge>,
        request_context: RunnerRequestContextSnapshot,
        cancellation: RoutedProviderCancellation,
        checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    ) -> Self {
        Self {
            provider,
            bridge,
            request_context,
            cancellation,
            checkpoint_store,
            box_resources: None,
        }
    }

    pub fn with_box_resources(
        mut self,
        box_resources: Arc<dyn RunnerBoxResourcePort>,
    ) -> Self {
        self.box_resources = Some(box_resources);
        self
    }

    pub fn has_box_resources(&self) -> bool {
        self.box_resources.is_some()
    }

    pub fn provider(&self) -> RoutedProvider {
        self.provider
    }

    pub fn cancellation(&self) -> RoutedProviderCancellation {
        self.cancellation.clone()
    }

    pub fn run(
        &self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        let bridge: Arc<dyn RoutedToolBridge> = match &self.box_resources {
            Some(box_resources) => Arc::new(RunnerBoxToolBridge::new(
                Arc::clone(&self.bridge),
                Arc::clone(box_resources),
            )),
            None => Arc::clone(&self.bridge),
        };
        run_routed_provider_in_runner(
            RoutedProviderRun {
                provider: self.provider,
                data_dir,
                messages,
                bridge,
                request_context: self.request_context.clone(),
                cancellation: self.cancellation.clone(),
                checkpoint_store: Arc::clone(&self.checkpoint_store),
            },
            on_text_delta,
        )
    }
}
