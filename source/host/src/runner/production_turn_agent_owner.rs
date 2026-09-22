use std::path::Path;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};

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
}

impl ProductionTurnAgentOwner {
    pub fn new(composition: TurnAgentComposition) -> Self {
        Self {
            composition,
            shell: TurnRunShell::default(),
            last_finished: None,
        }
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
        let Self {
            composition,
            shell,
            last_finished,
        } = self;
        run_owned_turn(shell, last_finished, messages, || {
            composition.run(data_dir, messages, on_text_delta)
        })
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
        run_owned_turn(
            &mut self.shell,
            &mut self.last_finished,
            messages,
            execute,
        )
    }
}

fn run_owned_turn<Execute>(
    shell: &mut TurnRunShell,
    last_finished: &mut Option<TurnRunFinished>,
    messages: &[ProviderMessage],
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
    let started = shell
        .begin_run(prompt, TurnRunOptions::default())
        .map_err(turn_shell_error)?;
    shell
        .mark_dispatched(&started.owner)
        .map_err(turn_shell_error)?;

    let result = execute();
    let settlement = match &result {
        Ok(_) => shell.finish_completed(&started.owner),
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
