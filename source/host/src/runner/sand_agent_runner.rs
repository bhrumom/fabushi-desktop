use std::path::Path;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};

use super::production_turn_agent_owner::ProductionTurnAgentOwner;
use super::routed_provider_runtime::RoutedProviderCancellation;
use super::{TurnRunFinished, TurnRunOptions};

/// Runner-owned shipping facade for provider-backed turns.
///
/// This is intentionally separate from Host supervision. The Host owns the
/// process/thread boundary; this type owns the turn Agent owner and therefore
/// the provider execution plus terminal settlement for the turn.
pub struct SandAgentRunner {
    owner: ProductionTurnAgentOwner,
}

impl SandAgentRunner {
    pub fn new(owner: ProductionTurnAgentOwner) -> Self {
        Self { owner }
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
