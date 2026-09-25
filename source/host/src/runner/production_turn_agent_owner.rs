use std::path::Path;
use std::sync::Arc;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};

use super::production_agent_checkpoint::AgentStateCheckpointSink;
use super::tools::box_help_tool::WAITING_USER_CANCELLATION_PREFIX;
use super::routed_provider_runtime::RoutedProviderCancellation;
use super::turn_agent_composition::TurnAgentComposition;
use super::{
    TerminalOutcome, TurnRunFinished, TurnRunOptions, TurnRunShell,
    TurnRunShellError,
};

/// Owns one production turn's terminal lifecycle independently from the Host.
///
/// The Host may supervise the worker thread, but begin/dispatched/terminal
/// settlement belongs here so a Host failure or a provider failure cannot
/// produce two final states for the same Runner turn.
pub struct ProductionTurnAgentOwner {
    composition: TurnAgentComposition,
    shell: TurnRunShell,
    last_finished: Option<TurnRunFinished>,
    agent_state_checkpoint_sink: Option<Arc<dyn AgentStateCheckpointSink>>,
}

impl ProductionTurnAgentOwner {
    pub fn new(composition: TurnAgentComposition) -> Self {
        Self {
            composition,
            shell: TurnRunShell::default(),
            last_finished: None,
            agent_state_checkpoint_sink: None,
        }
    }

    pub fn with_agent_state_checkpoint_sink(
        mut self,
        sink: Arc<dyn AgentStateCheckpointSink>,
    ) -> Self {
        self.agent_state_checkpoint_sink = Some(sink);
        self
    }

    pub fn cancellation(&self) -> RoutedProviderCancellation {
        self.composition.cancellation()
    }

    pub fn last_finished(&self) -> Option<&TurnRunFinished> {
        self.last_finished.as_ref()
    }

    pub fn run_routed_provider(
        &mut self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.run_routed_provider_with_options(
            data_dir,
            messages,
            TurnRunOptions::default(),
            on_text_delta,
        )
    }

    pub fn run_routed_provider_with_options(
        &mut self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        options: TurnRunOptions,
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.run_routed_provider_with_projected_messages(
            data_dir,
            messages,
            messages,
            options,
            on_text_delta,
        )
    }

    pub fn run_routed_provider_with_projected_messages(
        &mut self,
        data_dir: &Path,
        lifecycle_messages: &[ProviderMessage],
        provider_messages: &[ProviderMessage],
        options: TurnRunOptions,
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        let Self {
            composition,
            shell,
            last_finished,
            agent_state_checkpoint_sink,
        } = self;
        run_owned_turn(
            shell,
            last_finished,
            agent_state_checkpoint_sink.as_deref(),
            lifecycle_messages,
            options,
            || composition.run(data_dir, provider_messages, on_text_delta),
        )
    }

    /// Contract seam used by independent lifecycle tests. The shipping path
    /// above supplies the real TurnAgentComposition executor.
    pub fn run_with<Execute>(
        &mut self,
        messages: &[ProviderMessage],
        execute: Execute,
    ) -> Result<String, ProviderSessionError>
    where
        Execute: FnOnce() -> Result<String, ProviderSessionError>,
    {
        self.run_with_options(messages, TurnRunOptions::default(), execute)
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
        run_owned_turn(
            &mut self.shell,
            &mut self.last_finished,
            self.agent_state_checkpoint_sink.as_deref(),
            messages,
            options,
            execute,
        )
    }
}

fn run_owned_turn<Execute>(
    shell: &mut TurnRunShell,
    last_finished: &mut Option<TurnRunFinished>,
    agent_state_checkpoint_sink: Option<&dyn AgentStateCheckpointSink>,
    messages: &[ProviderMessage],
    options: TurnRunOptions,
    execute: Execute,
) -> Result<String, ProviderSessionError>
where
    Execute: FnOnce() -> Result<String, ProviderSessionError>,
{
    let prompt = latest_non_empty_user_prompt(messages).ok_or_else(|| {
        ProviderSessionError::Configuration(
            "Runner production turn requires a non-empty user prompt.".into(),
        )
    })?;
    let checkpoint_options = options.clone();
    let started = shell
        .begin_run(prompt, options)
        .map_err(turn_shell_error)?;
    shell
        .mark_dispatched(&started.owner)
        .map_err(turn_shell_error)?;

    let mut result = execute();
    if let (Ok(content), Some(sink)) = (&result, agent_state_checkpoint_sink) {
        if let Err(error) = sink.checkpoint_text_turn(
            messages,
            &checkpoint_options,
            content,
        ) {
            result = Err(error);
        }
    }
    let settlement = match &result {
        Ok(_) => shell.finish_completed(&started.owner),
        Err(ProviderSessionError::Cancelled(reason))
            if reason.starts_with(WAITING_USER_CANCELLATION_PREFIX) =>
        {
            shell
                .end_turn_awaiting_user(&started.owner, reason.clone())
                .and_then(|()| shell.finish_cancelled(&started.owner))
        }
        Err(ProviderSessionError::Cancelled(_)) => {
            shell.finish_cancelled(&started.owner)
        }
        Err(error) => shell.finish_failed(
            &started.owner,
            matches!(error, ProviderSessionError::Transport(_)),
            error.to_string(),
        ),
    }
    .map_err(turn_shell_error)?;
    *last_finished = Some(settlement);
    result
}

fn latest_non_empty_user_prompt(messages: &[ProviderMessage]) -> Option<&str> {
    messages
        .iter()
        .rev()
        .find(|message| {
            message.role == "user" && !message.content.trim().is_empty()
        })
        .map(|message| message.content.as_str())
}

fn turn_shell_error(error: TurnRunShellError) -> ProviderSessionError {
    ProviderSessionError::Protocol(format!(
        "Runner turn lifecycle failed: {error}"
    ))
}

#[allow(dead_code)]
fn _terminal_outcome_type_anchor(outcome: &TerminalOutcome) -> bool {
    matches!(
        outcome,
        TerminalOutcome::Completed
            | TerminalOutcome::Cancelled
            | TerminalOutcome::WaitingUser
            | TerminalOutcome::Failed { .. }
    )
}
