use std::sync::Arc;

use crate::extensions::inference::cursor_inference_transport::CursorInferenceAuth;
use crate::extensions::inference::provider_session::{ProviderTokenUsage, RoutedProvider};
use crate::cloud_agents::cloud_agent_tool::CloudAgentToolDependencies;
use crate::runner::box_tool_access::RunnerBoxResourcePort;
use crate::runner::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryReport, RoutedProviderCheckpointStore,
};
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge, RunnerRequestContextSnapshot,
};
use crate::runner::sand_action_audit::{ActionAuditSink, RoutedMcpAuditConfig};
use crate::runner::tools::sand_reaction_tool::ReactionSink;
use crate::runner::tools::sand_browser_tools::BrowserToolExecutor;
use crate::runner::tools::sand_computer_tool::ComputerToolExecutor;
use crate::runner::tools::sand_file_transfer_tools::FileTransferExecutor;
use crate::runner::tools::sand_mcp_management_tools::McpManagementSink;
use crate::runner::tools::send_message_tool::SendMessageSink;
use crate::runner::turn_agent_composition::TurnAgentComposition;
use crate::runner::turn_observation::TurnObservationHandle;

/// Typed production half of the frozen Host -> Runner turn bridge.
///
/// The Host owns concrete platform/session services. The Runner bridge owns
/// one immutable per-turn dependency projection and creates the Runner
/// composition. This keeps provider/cancellation/checkpoint/tool identities
/// out of ad-hoc Host assembly while the full generated Agent turn engine is
/// still being ported behind the same boundary.
pub struct ProductionActionAuditInput {
    pub agent_id: String,
    pub turn_id: Option<String>,
    pub sink: Arc<dyn ActionAuditSink>,
}

pub struct ProductionRunnerCompositionInput {
    pub provider: RoutedProvider,
    pub bridge: Arc<dyn RoutedToolBridge>,
    pub cursor_auth: Option<Arc<dyn CursorInferenceAuth>>,
    pub request_context: RunnerRequestContextSnapshot,
    pub cancellation: RoutedProviderCancellation,
    pub checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    pub retry_sink: Option<Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>>,
    pub retry_report_sink: Option<Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>>,
    pub usage_sink: Option<Arc<dyn Fn(ProviderTokenUsage) + Send + Sync>>,
    pub box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    pub browser_executor: Option<Arc<dyn BrowserToolExecutor>>,
    pub computer_executor: Option<Arc<dyn ComputerToolExecutor>>,
    pub file_transfer_executor: Option<Arc<dyn FileTransferExecutor>>,
    pub mcp_management_sink: Option<Arc<dyn McpManagementSink>>,
    pub send_message_sink: Option<Arc<dyn SendMessageSink>>,
    pub reaction_sink: Option<Arc<dyn ReactionSink>>,
    pub cloud_agent_tool: Option<CloudAgentToolDependencies>,
    pub multitask_enabled: bool,
    pub spotlight_enabled: bool,
    pub action_audit: Option<ProductionActionAuditInput>,
    pub observation: Option<TurnObservationHandle>,
}

pub fn create_production_runner_composition(
    input: ProductionRunnerCompositionInput,
) -> TurnAgentComposition {
    let mut composition = TurnAgentComposition::new(
        input.provider,
        input.bridge,
        input.request_context,
        input.cancellation,
        input.checkpoint_store,
    );
    if let Some(cursor_auth) = input.cursor_auth {
        composition = composition.with_cursor_auth(cursor_auth);
    }
    if let Some(retry_sink) = input.retry_sink {
        composition = composition.with_retry_sink(retry_sink);
    }
    if let Some(retry_report_sink) = input.retry_report_sink {
        composition = composition.with_retry_report_sink(retry_report_sink);
    }
    if let Some(usage_sink) = input.usage_sink {
        composition = composition.with_usage_sink(usage_sink);
    }
    composition = composition
        .with_multitask_enabled(input.multitask_enabled)
        .with_spotlight_enabled(input.spotlight_enabled);
    if let Some(action_audit) = input.action_audit {
        composition = composition.with_action_audit(RoutedMcpAuditConfig::new(
            action_audit.agent_id,
            action_audit.turn_id,
            action_audit.sink,
        ));
    }
    if let Some(box_resources) = input.box_resources {
        composition = composition.with_box_resources(box_resources);
    }
    if let Some(browser_executor) = input.browser_executor {
        composition = composition.with_browser_executor(browser_executor);
    }
    if let Some(computer_executor) = input.computer_executor {
        composition = composition.with_computer_executor(computer_executor);
    }
    if let Some(file_transfer_executor) = input.file_transfer_executor {
        composition = composition.with_file_transfer_executor(file_transfer_executor);
    }
    if let Some(mcp_management_sink) = input.mcp_management_sink {
        composition = composition.with_mcp_management_sink(mcp_management_sink);
    }
    if let Some(reaction_sink) = input.reaction_sink {
        composition = composition.with_reaction_sink(reaction_sink);
    }
    if let Some(cloud_agent_tool) = input.cloud_agent_tool {
        composition = composition.with_cloud_agent_tool(cloud_agent_tool);
    }
    if let Some(send_message_sink) = input.send_message_sink {
        composition = composition.with_send_message_sink(send_message_sink);
    }
    if let Some(observation) = input.observation {
        composition = composition.with_observation(observation);
    }
    composition
}
