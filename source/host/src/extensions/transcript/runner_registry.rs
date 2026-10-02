use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedProviderTaskRegistry,
};

pub const RUN_WATCHDOG_INTERRUPT_REASON: &str =
    "run-queue watchdog: releasing a wedged predecessor";

#[derive(Clone, Default)]
pub struct TranscriptRunnerRegistry {
    routed_provider_tasks: Arc<RoutedProviderTaskRegistry>,
    group_member_tasks: Arc<RoutedProviderTaskRegistry>,
    dm_preempted_group_members: Arc<Mutex<HashSet<String>>>,
}

impl TranscriptRunnerRegistry {
    pub fn new(routed_provider_tasks: Arc<RoutedProviderTaskRegistry>) -> Self {
        Self {
            routed_provider_tasks,
            group_member_tasks: Arc::new(RoutedProviderTaskRegistry::default()),
            dm_preempted_group_members: Arc::new(Mutex::new(HashSet::new())),
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

    pub fn register_group_member(
        &self,
        agent_id: &str,
        stream_id: &str,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        self.group_member_tasks
            .register_for_agent(agent_id, stream_id)
    }

    pub fn finish_routed_provider(&self, stream_id: &str) {
        self.routed_provider_tasks.finish(stream_id);
        self.group_member_tasks.finish(stream_id);
    }

    pub fn cancel_stream(&self, stream_id: &str, reason: impl Into<String>) -> bool {
        let reason = reason.into();
        self.routed_provider_tasks.cancel(stream_id, reason.clone())
            || self.group_member_tasks.cancel(stream_id, reason)
    }

    pub fn agent_id_for_stream(&self, stream_id: &str) -> Option<String> {
        self.routed_provider_tasks
            .agent_id_for_stream(stream_id)
            .or_else(|| self.group_member_tasks.agent_id_for_stream(stream_id))
    }

    pub fn interrupt_wedged_run_for_watchdog(&self, agent_id: &str) -> bool {
        self.routed_provider_tasks
            .cancel_agent(agent_id, RUN_WATCHDOG_INTERRUPT_REASON)
            > 0
    }

    pub fn cancel_agent(&self, agent_id: &str, reason: impl Into<String>) -> usize {
        self.routed_provider_tasks.cancel_agent(agent_id, reason)
    }

    pub fn cancel_group_member_agent(
        &self,
        agent_id: &str,
        reason: impl Into<String>,
    ) -> usize {
        self.group_member_tasks.cancel_agent(agent_id, reason)
    }

    pub fn preempt_group_member_agent(
        &self,
        agent_id: &str,
        reason: impl Into<String>,
    ) -> usize {
        if self
            .group_member_tasks
            .active_stream_ids_for_agent(agent_id)
            .is_empty()
        {
            return 0;
        }
        self.dm_preempted_group_members
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(agent_id.to_string());
        let cancelled = self.group_member_tasks.cancel_agent(agent_id, reason);
        if cancelled == 0 {
            self.clear_group_member_preempted(agent_id);
        }
        cancelled
    }

    pub fn take_group_member_preempted(&self, agent_id: &str) -> bool {
        self.dm_preempted_group_members
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(agent_id)
    }

    pub fn clear_group_member_preempted(&self, agent_id: &str) {
        self.dm_preempted_group_members
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(agent_id);
    }

    pub fn active_stream_ids_for_agent(&self, agent_id: &str) -> Vec<String> {
        let mut streams = self
            .routed_provider_tasks
            .active_stream_ids_for_agent(agent_id);
        streams.extend(
            self.group_member_tasks
                .active_stream_ids_for_agent(agent_id),
        );
        streams.sort();
        streams
    }

    pub fn active_group_member_stream_ids_for_agent(&self, agent_id: &str) -> Vec<String> {
        self.group_member_tasks
            .active_stream_ids_for_agent(agent_id)
    }

    pub fn active_count(&self) -> usize {
        self.routed_provider_tasks.active_count() + self.group_member_tasks.active_count()
    }

    pub fn cancel_all(&self, reason: &str) {
        self.routed_provider_tasks.cancel_all(reason);
        self.group_member_tasks.cancel_all(reason);
    }
}
