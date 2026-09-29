use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};

use super::production_turn_agent_owner::ProductionTurnAgentOwner;
use super::routed_provider_runtime::RoutedProviderCancellation;
use super::subagent_runtime::{
    PendingWake, RunOutcome, SettleResult, SubagentLineage, SubagentRuntime,
    SubagentSessionSnapshot,
};
use super::{TurnRunFinished, TurnRunOptions};

/// Runner-owned shipping facade for provider-backed turns.
///
/// This is intentionally separate from Host supervision. The Host owns the
/// process/thread boundary; this type owns the turn Agent owner and therefore
/// the provider execution plus terminal settlement for the turn.
pub struct SandAgentRunner {
    owner: ProductionTurnAgentOwner,
    generated_agent_runtime: Option<Arc<Mutex<SubagentRuntime>>>,
}

impl SandAgentRunner {
    pub fn new(owner: ProductionTurnAgentOwner) -> Self {
        Self {
            owner,
            generated_agent_runtime: None,
        }
    }

    pub fn with_generated_agent_runtime(
        mut self,
        runtime: Arc<Mutex<SubagentRuntime>>,
    ) -> Self {
        self.generated_agent_runtime = Some(runtime);
        self
    }

    pub fn generated_agent_runtime(&self) -> Option<Arc<Mutex<SubagentRuntime>>> {
        self.generated_agent_runtime.as_ref().map(Arc::clone)
    }

    pub fn begin_generated_subagent(
        &self,
        parent_agent_id: &str,
        box_id: &str,
        subagent_agent_id: &str,
        subagent_type: &str,
        tool_call_id: &str,
        prompt: &str,
        lineage: Option<SubagentLineage>,
        now_ms: u64,
    ) -> Result<Option<PendingWake>, String> {
        let Some(runtime) = self.generated_agent_runtime.as_ref() else {
            return Ok(None);
        };
        let mut runtime = runtime
            .lock()
            .map_err(|_| "generated Agent runtime lock poisoned".to_string())?;
        runtime.register_session(subagent_agent_id, SubagentSessionSnapshot::default());
        Ok(runtime.dispatch_background_subagent(
            parent_agent_id,
            box_id,
            subagent_agent_id,
            subagent_type,
            tool_call_id,
            prompt,
            lineage,
            None,
            now_ms,
        ))
    }

    pub fn settle_generated_subagent(
        &self,
        subagent_agent_id: &str,
        outcome: RunOutcome,
        now_ms: u64,
    ) -> Result<SettleResult, String> {
        let Some(runtime) = self.generated_agent_runtime.as_ref() else {
            return Ok(SettleResult::default());
        };
        runtime
            .lock()
            .map_err(|_| "generated Agent runtime lock poisoned".to_string())
            .map(|mut runtime| {
                runtime.settle_background_subagent_turn(subagent_agent_id, outcome, now_ms)
            })
    }

    pub fn cancellation(&self) -> RoutedProviderCancellation {
        self.owner.cancellation()
    }

    pub fn last_finished(&self) -> Option<&TurnRunFinished> {
        self.owner.last_finished()
    }

    pub fn run_routed_provider(
        &mut self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.owner
            .run_routed_provider(data_dir, messages, on_text_delta)
    }

    pub fn run_routed_provider_with_options(
        &mut self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        options: TurnRunOptions,
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.owner
            .run_routed_provider_with_options(data_dir, messages, options, on_text_delta)
    }

    pub fn run_routed_provider_with_projected_messages(
        &mut self,
        data_dir: &Path,
        lifecycle_messages: &[ProviderMessage],
        provider_messages: &[ProviderMessage],
        options: TurnRunOptions,
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.owner.run_routed_provider_with_projected_messages(
            data_dir,
            lifecycle_messages,
            provider_messages,
            options,
            on_text_delta,
        )
    }

    pub fn run_with<Execute>(
        &mut self,
        messages: &[ProviderMessage],
        execute: Execute,
    ) -> Result<String, ProviderSessionError>
    where
        Execute: FnOnce() -> Result<String, ProviderSessionError>,
    {
        self.owner.run_with(messages, execute)
    }

    pub fn run_with_options<Execute>(
        &mut self,
        messages: &[ProviderMessage],
        options: TurnRunOptions,
        execute: Execute,
    ) -> Result<String, ProviderSessionError>
    where
        Execute: FnOnce() -> Result<String, ProviderSessionError>,
    {
        self.owner.run_with_options(messages, options, execute)
    }
}
