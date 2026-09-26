use std::sync::Arc;

use crate::extensions::inference::provider_session::RoutedProvider;
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
    pub request_context: RunnerRequestContextSnapshot,
    pub cancellation: RoutedProviderCancellation,
    pub checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    pub retry_sink: Option<Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>>,
    pub retry_report_sink: Option<Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>>,
    pub box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    pub send_message_sink: Option<Arc<dyn SendMessageSink>>,
    pub reaction_sink: Option<Arc<dyn ReactionSink>>,
    pub cloud_agent_tool: Option<CloudAgentToolDependencies>,
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
    if let Some(retry_sink) = input.retry_sink {
        composition = composition.with_retry_sink(retry_sink);
    }
    if let Some(retry_report_sink) = input.retry_report_sink {
        composition = composition.with_retry_report_sink(retry_report_sink);
    }
    composition = composition.with_spotlight_enabled(input.spotlight_enabled);
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
