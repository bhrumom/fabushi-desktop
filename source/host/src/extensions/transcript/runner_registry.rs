use std::sync::Arc;

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedProviderTaskRegistry,
};

pub const RUN_WATCHDOG_INTERRUPT_REASON: &str =
    "run-queue watchdog: releasing a wedged predecessor";

#[derive(Clone, Default)]
pub struct TranscriptRunnerRegistry {
    routed_provider_tasks: Arc<RoutedProviderTaskRegistry>,
}

impl TranscriptRunnerRegistry {
    pub fn new(routed_provider_tasks: Arc<RoutedProviderTaskRegistry>) -> Self {
        Self {
            routed_provider_tasks,
        }
    }

    pub fn register_routed_provider(
        &self,
        agent_id: &str,
        stream_id: &str,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        self.routed_provider_tasks
            .register_for_agent(agent_id, stream_id)
    }

    pub fn finish_routed_provider(&self, stream_id: &str) {
        self.routed_provider_tasks.finish(stream_id);
    }

    pub fn cancel_stream(&self, stream_id: &str, reason: impl Into<String>) -> bool {
        self.routed_provider_tasks.cancel(stream_id, reason)
    }

    pub fn interrupt_wedged_run_for_watchdog(&self, agent_id: &str) -> bool {
        self.routed_provider_tasks
            .cancel_agent(agent_id, RUN_WATCHDOG_INTERRUPT_REASON)
            > 0
    }

    pub fn active_stream_ids_for_agent(&self, agent_id: &str) -> Vec<String> {
        self.routed_provider_tasks
            .active_stream_ids_for_agent(agent_id)
    }

    pub fn active_count(&self) -> usize {
        self.routed_provider_tasks.active_count()
    }

    pub fn cancel_all(&self, reason: &str) {
        self.routed_provider_tasks.cancel_all(reason);
    }
}
