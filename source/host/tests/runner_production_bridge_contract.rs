use std::sync::Arc;

use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedProvider, RoutedProviderCheckpoint,
    RoutedToolDefinition,
};
use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderCheckpointStore;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner_production_bridge::{
    ProductionRunnerCompositionInput, create_production_runner_composition,
};
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
        Err(ProviderSessionError::Tool("no tools".into()))
    }
}

struct MemoryCheckpointStore;

impl RoutedProviderCheckpointStore for MemoryCheckpointStore {
    fn persist(
        &self,
        _checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        Ok("checkpoint".into())
    }
}

fn request_context() -> RunnerRequestContextSnapshot {
    RunnerRequestContextSnapshot {
        context: HostRequestContext {
            os_version: "test".into(),
            shell: None,
            time_zone: Some("UTC".into()),
            transcripts_folder: "/tmp/transcripts".into(),
            user_full_name: None,
        },
        rules: None,
    }
}

#[test]
fn production_bridge_preserves_provider_and_cancellation_identity() {
    let cancellation = RoutedProviderCancellation::default();
    let composition = create_production_runner_composition(
        ProductionRunnerCompositionInput {
            provider: RoutedProvider::OpenRouter,
            bridge: Arc::new(EmptyBridge),
            request_context: request_context(),
            cancellation: cancellation.clone(),
            checkpoint_store: Arc::new(MemoryCheckpointStore),
            retry_sink: None,
            retry_report_sink: None,
            box_resources: None,
            send_message_sink: None,
            reaction_sink: None,
            action_audit: None,
            observation: None,
        },
    );

    assert_eq!(composition.provider(), RoutedProvider::OpenRouter);
    assert!(!cancellation.is_cancelled());
    assert!(composition.cancellation().cancel("contract"));
    assert!(cancellation.is_cancelled());
    assert_eq!(cancellation.reason().as_deref(), Some("contract"));
    assert!(!composition.has_box_resources());
    assert!(!composition.has_send_message_sink());
    assert!(!composition.has_reaction_sink());
    assert!(!composition.has_action_audit());
}
