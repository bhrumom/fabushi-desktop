use std::path::Path;
use std::sync::Arc;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider,
};
use crate::cloud_agents::cloud_agent_tool::CloudAgentToolDependencies;

use super::box_tool_access::RunnerBoxResourcePort;
use super::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryReport, RoutedProviderCheckpointStore,
};
use super::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedProviderRun, RoutedToolBridge,
    RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};
use super::sand_action_audit::{AuditedRoutedToolBridge, RoutedMcpAuditConfig};
use super::turn_observation::{ObservedRoutedToolBridge, TurnObservationHandle};
use super::tools::send_message_tool::SendMessageSink;
use super::tools::sand_reaction_tool::ReactionSink;
use super::tools::sand_agent_management_tools::AgentManagementSink;
use super::tools::sand_state_tool::SandStateWriter;
use super::tools::turn_toolset::{
    TurnToolsetDependencies, build_turn_toolset, fence_turn_toolset,
};

/// Shipping Runner composition for one provider-backed turn.
///
/// This module deliberately owns the dependency assembly that used to be
/// hand-built by the Host entrypoint: provider selection, Host tool bridge,
/// request-context projection, cancellation, and the durable checkpoint store.
/// Provider execution itself remains in the Runner-owned routed-provider
/// runtime so the Host process does not absorb Runner responsibilities.
#[derive(Clone)]
pub struct TurnAgentComposition {
    provider: RoutedProvider,
    bridge: Arc<dyn RoutedToolBridge>,
    request_context: RunnerRequestContextSnapshot,
    cancellation: RoutedProviderCancellation,
    checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    retry_sink: Option<Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>>,
    retry_report_sink: Option<Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>>,
    box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    send_message_sink: Option<Arc<dyn SendMessageSink>>,
    reaction_sink: Option<Arc<dyn ReactionSink>>,
    agent_management_sink: Option<Arc<dyn AgentManagementSink>>,
    state_writer: Option<Arc<dyn SandStateWriter>>,
    cloud_agent_tool: Option<CloudAgentToolDependencies>,
    spotlight_enabled: bool,
    action_audit: Option<RoutedMcpAuditConfig>,
    observation: Option<TurnObservationHandle>,
}

impl TurnAgentComposition {
    pub fn new(
        provider: RoutedProvider,
        bridge: Arc<dyn RoutedToolBridge>,
        request_context: RunnerRequestContextSnapshot,
        cancellation: RoutedProviderCancellation,
        checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    ) -> Self {
        Self {
            provider,
            bridge,
            request_context,
            cancellation,
            checkpoint_store,
            retry_sink: None,
            retry_report_sink: None,
            box_resources: None,
            send_message_sink: None,
            reaction_sink: None,
            agent_management_sink: None,
            state_writer: None,
            cloud_agent_tool: None,
            spotlight_enabled: false,
            action_audit: None,
            observation: None,
        }
    }

    pub fn with_retry_sink(
        mut self,
        sink: Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>,
    ) -> Self {
        self.retry_sink = Some(sink);
        self
    }

    pub fn with_retry_report_sink(
        mut self,
        sink: Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>,
    ) -> Self {
        self.retry_report_sink = Some(sink);
        self
    }

    pub fn with_box_resources(
        mut self,
        box_resources: Arc<dyn RunnerBoxResourcePort>,
    ) -> Self {
        self.box_resources = Some(box_resources);
        self
    }

    pub fn has_box_resources(&self) -> bool {
        self.box_resources.is_some()
    }

    pub fn with_send_message_sink(
        mut self,
        sink: Arc<dyn SendMessageSink>,
    ) -> Self {
        self.send_message_sink = Some(sink);
        self
    }

    pub fn has_send_message_sink(&self) -> bool {
        self.send_message_sink.is_some()
    }

    pub fn with_reaction_sink(
        mut self,
        sink: Arc<dyn ReactionSink>,
    ) -> Self {
        self.reaction_sink = Some(sink);
        self
    }

    pub fn has_reaction_sink(&self) -> bool {
        self.reaction_sink.is_some()
    }

    pub fn with_agent_management_sink(
        mut self,
        sink: Arc<dyn AgentManagementSink>,
    ) -> Self {
        self.agent_management_sink = Some(sink);
        self
    }

    pub fn has_agent_management_sink(&self) -> bool {
        self.agent_management_sink.is_some()
    }

    pub fn with_state_writer(
        mut self,
        state: Arc<dyn SandStateWriter>,
    ) -> Self {
        self.state_writer = Some(state);
        self
    }

    pub fn has_state_writer(&self) -> bool {
        self.state_writer.is_some()
    }

    pub fn with_cloud_agent_tool(
        mut self,
        deps: CloudAgentToolDependencies,
    ) -> Self {
        self.cloud_agent_tool = Some(deps);
        self
    }

    pub fn has_cloud_agent_tool(&self) -> bool {
        self.cloud_agent_tool.is_some()
    }

    pub fn with_spotlight_enabled(mut self, enabled: bool) -> Self {
        self.spotlight_enabled = enabled;
        self
    }

    pub fn has_spotlight_enabled(&self) -> bool {
        self.spotlight_enabled
    }

    pub fn with_action_audit(mut self, config: RoutedMcpAuditConfig) -> Self {
        self.action_audit = Some(config);
        self
    }

    pub fn has_action_audit(&self) -> bool {
        self.action_audit.is_some()
    }

    pub fn with_observation(
        mut self,
        observation: TurnObservationHandle,
    ) -> Self {
        self.observation = Some(observation);
        self
    }

    pub fn has_observation(&self) -> bool {
        self.observation.is_some()
    }

    pub fn provider(&self) -> RoutedProvider {
        self.provider
    }

    pub fn cancellation(&self) -> RoutedProviderCancellation {
        self.cancellation.clone()
    }

    pub fn run(
        &self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        let bridge: Arc<dyn RoutedToolBridge> = match &self.action_audit {
            Some(config) => Arc::new(AuditedRoutedToolBridge::new(
                Arc::clone(&self.bridge),
                config.clone(),
            )),
            None => Arc::clone(&self.bridge),
        };
        let bridge = build_turn_toolset(
            bridge,
            TurnToolsetDependencies {
                cancellation: self.cancellation.clone(),
                box_resources: self.box_resources.clone(),
                send_message_sink: self.send_message_sink.clone(),
                reaction_sink: self.reaction_sink.clone(),
                agent_management_sink: self.agent_management_sink.clone(),
                state_writer: self.state_writer.clone(),
                cloud_agent_tool: self.cloud_agent_tool.clone(),
            },
        );
        let bridge: Arc<dyn RoutedToolBridge> = match &self.observation {
            Some(observation) => Arc::new(ObservedRoutedToolBridge::new(
                bridge,
                Arc::clone(observation),
            )),
            None => bridge,
        };
        let bridge = fence_turn_toolset(bridge, self.spotlight_enabled);
        run_routed_provider_in_runner(
            RoutedProviderRun {
                provider: self.provider,
                data_dir,
                messages,
                bridge,
                request_context: self.request_context.clone(),
                cancellation: self.cancellation.clone(),
                checkpoint_store: Arc::clone(&self.checkpoint_store),
                retry_sink: self.retry_sink.clone(),
                retry_report_sink: self.retry_report_sink.clone(),
            },
            on_text_delta,
        )
    }
}
