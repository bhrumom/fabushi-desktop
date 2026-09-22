use std::sync::Arc;

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider,
    RoutedProviderCheckpoint, RoutedToolDefinition,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::production_turn_agent_owner::ProductionTurnAgentOwner;
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::sand_agent_runner::SandAgentRunner;
use mahayana_host_runtime::runner::turn_agent_composition::TurnAgentComposition;
use mahayana_host_runtime::runner::TerminalOutcome;
use serde_json::Value;

struct EmptyBridge;

impl RoutedToolBridge for EmptyBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool(
            "contract bridge has no tools".into(),
        ))
    }
}

struct MemoryCheckpointStore;

impl RoutedProviderCheckpointStore for MemoryCheckpointStore {
    fn persist(
        &self,
        _checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        Ok("memory-checkpoint".into())
    }
}

fn runner() -> SandAgentRunner {
    let composition = TurnAgentComposition::new(
        RoutedProvider::OpenRouter,
        Arc::new(EmptyBridge),
        RunnerRequestContextSnapshot {
            context: HostRequestContext {
                os_version: "test".into(),
                shell: None,
                time_zone: Some("UTC".into()),
                transcripts_folder: "/tmp/transcripts".into(),
                user_full_name: None,
            },
            rules: None,
        },
        RoutedProviderCancellation::default(),
        Arc::new(MemoryCheckpointStore),
    );
    SandAgentRunner::new(ProductionTurnAgentOwner::new(composition))
}

fn user_messages() -> Vec<ProviderMessage> {
    vec![ProviderMessage {
        role: "user".into(),
        content: "hello".into(),
    }]
}

#[test]
fn shipping_sand_agent_owner_settles_completed_turn_once() {
    let mut runner = runner();
    let result = runner
        .run_with(&user_messages(), || Ok("done".into()))
        .expect("completed turn");
    assert_eq!(result, "done");
    assert!(matches!(
        runner.last_finished().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Completed)
    ));
}

#[test]
fn shipping_sand_agent_owner_preserves_retryable_transport_failure() {
    let mut runner = runner();
    let error = runner
        .run_with(&user_messages(), || {
            Err(ProviderSessionError::Transport(
                "upstream reset".into(),
            ))
        })
        .expect_err("transport failure");
    assert!(error.to_string().contains("upstream reset"));
    assert!(matches!(
        runner.last_finished().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Failed {
            retryable: true,
            message,
        }) if message.contains("upstream reset")
    ));
}

#[test]
fn shipping_sand_agent_owner_settles_provider_cancellation() {
    let mut runner = runner();
    let error = runner
        .run_with(&user_messages(), || {
            Err(ProviderSessionError::Cancelled(
                "user cancelled".into(),
            ))
        })
        .expect_err("cancelled turn");
    assert!(matches!(error, ProviderSessionError::Cancelled(_)));
    assert!(matches!(
        runner.last_finished().map(|finished| &finished.outcome),
        Some(TerminalOutcome::Cancelled)
    ));
}

#[test]
fn shipping_sand_agent_owner_fails_closed_without_user_prompt() {
    let mut runner = runner();
    let messages = vec![ProviderMessage {
        role: "system".into(),
        content: "system only".into(),
    }];
    let mut executed = false;
    let error = runner
        .run_with(&messages, || {
            executed = true;
            Ok("unexpected".into())
        })
        .expect_err("missing user prompt");
    assert!(matches!(error, ProviderSessionError::Configuration(_)));
    assert!(!executed);
    assert!(runner.last_finished().is_none());
}
