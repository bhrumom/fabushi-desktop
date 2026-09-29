use std::collections::BTreeMap;
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
use mahayana_host_runtime::runner::tools::sand_mcp_management_tools::{
    McpAuthenticationResult, McpInstalledServer, McpManagementSink, McpPluginDetail,
    McpPluginSummary, McpRemoveServerResult, McpUninstallPluginResult,
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

struct EmptyMcpManagementSink;

impl McpManagementSink for EmptyMcpManagementSink {
    fn list_plugins(&self) -> Result<Vec<McpPluginSummary>, ProviderSessionError> { Ok(Vec::new()) }
    fn get_plugin(&self, _plugin_id: &str) -> Result<Option<McpPluginDetail>, ProviderSessionError> { Ok(None) }
    fn install_plugin(&self, _plugin_id: &str, _values: Option<&BTreeMap<String, String>>) -> Result<(), ProviderSessionError> { Ok(()) }
    fn add_server(&self, _name: &str, _config_json: &str) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
    fn list_installed(&self) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
    fn remove_server(&self, _server_id: &str) -> Result<McpRemoveServerResult, ProviderSessionError> {
        Ok(McpRemoveServerResult { removed: false, reason: None, servers: Vec::new() })
    }
    fn uninstall_plugin(&self, _plugin_id: &str) -> Result<McpUninstallPluginResult, ProviderSessionError> {
        Ok(McpUninstallPluginResult { removed: false, reason: None })
    }
    fn set_instructions(&self, _server_id: &str, _instructions: &str) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
    fn restart(&self) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
    fn authenticate(
        &self,
        _server_id: &str,
        _account_key: &str,
        _requesting_agent_id: Option<&str>,
        _force_reauth: bool,
    ) -> Result<McpAuthenticationResult, ProviderSessionError> {
        Ok(McpAuthenticationResult::NotConfigured { server_name: "test".into() })
    }
    fn remove_account(&self, _server_id: &str, _account_key: &str) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
    fn rename_account(&self, _server_id: &str, _account_key: &str, _new_account_key: &str) -> Result<Vec<McpInstalledServer>, ProviderSessionError> { Ok(Vec::new()) }
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
            cursor_auth: None,
            request_context: request_context(),
            cancellation: cancellation.clone(),
            checkpoint_store: Arc::new(MemoryCheckpointStore),
            retry_sink: None,
            retry_report_sink: None,
            usage_sink: None,
            spotlight_enabled: true,
            box_resources: None,
            browser_executor: None,
            computer_executor: None,
            file_transfer_executor: None,
            external_machine_executor: None,
            external_shell_review: None,
            mcp_management_sink: None,
            send_message_sink: None,
            reaction_sink: None,
            cloud_agent_tool: None,
            multitask_enabled: true,
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
    assert!(!composition.has_browser_executor());
    assert!(!composition.has_computer_executor());
    assert!(!composition.has_external_machine_executor());
    assert!(!composition.has_external_shell_review());
    assert!(!composition.has_mcp_management_sink());
    assert!(!composition.has_send_message_sink());
    assert!(!composition.has_reaction_sink());
    assert!(!composition.has_cloud_agent_tool());
    assert!(composition.has_multitask_enabled());
    assert!(!composition.has_action_audit());
    assert!(composition.has_spotlight_enabled());
}


#[test]
fn production_bridge_projects_mcp_management_into_canonical_turn_composition() {
    let composition = create_production_runner_composition(
        ProductionRunnerCompositionInput {
            provider: RoutedProvider::OpenRouter,
            bridge: Arc::new(EmptyBridge),
            cursor_auth: None,
            request_context: request_context(),
            cancellation: RoutedProviderCancellation::default(),
            checkpoint_store: Arc::new(MemoryCheckpointStore),
            retry_sink: None,
            retry_report_sink: None,
            usage_sink: None,
            spotlight_enabled: false,
            box_resources: None,
            browser_executor: None,
            computer_executor: None,
            file_transfer_executor: None,
            external_machine_executor: None,
            external_shell_review: None,
            mcp_management_sink: Some(Arc::new(EmptyMcpManagementSink)),
            send_message_sink: None,
            reaction_sink: None,
            cloud_agent_tool: None,
            multitask_enabled: false,
            action_audit: None,
            observation: None,
        },
    );

    assert!(composition.has_mcp_management_sink());
}
