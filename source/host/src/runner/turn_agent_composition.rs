use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::extensions::inference::cursor_inference_transport::CursorInferenceAuth;
use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, ProviderTokenUsage, RoutedProvider,
    RoutedProviderCheckpoint, RoutedToolDefinition,
};
use crate::cloud_agents::cloud_agent_tool::CloudAgentToolDependencies;
use crate::ports::mcp_state_executor::{
    McpStateExecResult, SandMcpToolProvider, execute_mcp_state as execute_canonical_mcp_state,
};

use super::box_tool_access::{BoxShellAutoReviewCallback, RunnerBoxResourcePort};
use super::production_turn_run_shell_adapter::{
    ProviderRetryEvent, ProviderRetryReport, RoutedProviderCheckpointStore,
};
use super::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedProviderRun, RoutedToolBridge,
    RunnerRequestContextSnapshot, run_routed_provider_in_runner,
};
use super::sand_action_audit::{AuditedRoutedToolBridge, RoutedMcpAuditConfig};
use super::subagent_runtime::SubagentRuntime;
use super::turn_shape::checkpoint_ended_on_silent_tool_calls;
use super::turn_observation::{
    McpObservedRoutedToolBridge, ObservedRoutedToolBridge, TurnObservationHandle,
};
use super::tools::communicate_tool::{
    CommunicateInteractionSink, CommunicateRoutedToolBridge,
    TurnObservationCommunicateSink,
};
use super::tools::send_message_tool::SendMessageSink;
use super::tools::sand_reaction_tool::ReactionSink;
use super::tools::sand_agent_management_tools::AgentManagementSink;
use super::tools::sand_task_subagent_tool::{SubagentTaskReviewCallback, SubagentTaskSink};
use super::tools::sand_subagent_management_tools::SubagentSteerReviewCallback;
use super::tools::sand_browser_tools::BrowserToolExecutor;
use super::tools::sand_computer_tool::{ComputerToolExecutor, ComputerToolExposure};
use super::tools::sand_file_transfer_tools::FileTransferExecutor;
use super::tools::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalShellAutoReviewCallback,
};
use super::tools::sand_mcp_management_tools::{McpManagementSink, McpManagementToolBridge};
use super::tools::sand_state_tool::{
    RoutineAutoReviewCallback, RoutinePostWriteCallback, SandStateWriter,
};
use super::tools::sand_multitask_todo_tool::MultitaskTodoState;
use super::tools::turn_toolset::{
    TurnToolsetDependencies, build_turn_toolset, fence_turn_toolset,
};

struct RoutedBridgeMcpToolProvider<'a> {
    bridge: &'a dyn RoutedToolBridge,
}

impl SandMcpToolProvider for RoutedBridgeMcpToolProvider<'_> {
    fn get_tools(&self) -> Result<Vec<RoutedToolDefinition>, String> {
        self.bridge.list_tools().map_err(|error| error.to_string())
    }
}

struct ObservedRoutedProviderCheckpointStore {
    delegate: Arc<dyn RoutedProviderCheckpointStore>,
    latest: Arc<Mutex<Option<RoutedProviderCheckpoint>>>,
}

impl RoutedProviderCheckpointStore for ObservedRoutedProviderCheckpointStore {
    fn persist(
        &self,
        checkpoint: &RoutedProviderCheckpoint,
    ) -> Result<String, ProviderSessionError> {
        let cursor = self.delegate.persist(checkpoint)?;
        *self
            .latest
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(checkpoint.clone());
        Ok(cursor)
    }
}

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
    cursor_auth: Option<Arc<dyn CursorInferenceAuth>>,
    request_context: RunnerRequestContextSnapshot,
    cancellation: RoutedProviderCancellation,
    checkpoint_store: Arc<dyn RoutedProviderCheckpointStore>,
    latest_provider_checkpoint: Arc<Mutex<Option<RoutedProviderCheckpoint>>>,
    retry_sink: Option<Arc<dyn Fn(&ProviderRetryEvent) + Send + Sync>>,
    retry_report_sink: Option<Arc<dyn Fn(&ProviderRetryReport) + Send + Sync>>,
    usage_sink: Option<Arc<dyn Fn(ProviderTokenUsage) + Send + Sync>>,
    box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    box_shell_review: Option<BoxShellAutoReviewCallback>,
    browser_executor: Option<Arc<dyn BrowserToolExecutor>>,
    computer_executor: Option<Arc<dyn ComputerToolExecutor>>,
    computer_exposure: ComputerToolExposure,
    file_transfer_executor: Option<Arc<dyn FileTransferExecutor>>,
    external_machine_executor: Option<Arc<dyn ExternalMachineExecutor>>,
    external_shell_review: Option<ExternalShellAutoReviewCallback>,
    mcp_management_sink: Option<Arc<dyn McpManagementSink>>,
    send_message_sink: Option<Arc<dyn SendMessageSink>>,
    reaction_sink: Option<Arc<dyn ReactionSink>>,
    agent_management_sink: Option<Arc<dyn AgentManagementSink>>,
    subagent_task_sink: Option<Arc<dyn SubagentTaskSink>>,
    subagent_task_review: Option<SubagentTaskReviewCallback>,
    subagent_runtime: Option<Arc<Mutex<SubagentRuntime>>>,
    subagent_steer_review: Option<SubagentSteerReviewCallback>,
    state_writer: Option<Arc<dyn SandStateWriter>>,
    routine_auto_review: Option<RoutineAutoReviewCallback>,
    routine_post_write: Option<RoutinePostWriteCallback>,
    multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
    cloud_agent_tool: Option<CloudAgentToolDependencies>,
    multitask_enabled: bool,
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
        let latest_provider_checkpoint = Arc::new(Mutex::new(None));
        let checkpoint_store: Arc<dyn RoutedProviderCheckpointStore> =
            Arc::new(ObservedRoutedProviderCheckpointStore {
                delegate: checkpoint_store,
                latest: Arc::clone(&latest_provider_checkpoint),
            });
        Self {
            provider,
            bridge,
            cursor_auth: None,
            request_context,
            cancellation,
            checkpoint_store,
            latest_provider_checkpoint,
            retry_sink: None,
            retry_report_sink: None,
            usage_sink: None,
            box_resources: None,
            box_shell_review: None,
            browser_executor: None,
            computer_executor: None,
            computer_exposure: ComputerToolExposure::Full,
            file_transfer_executor: None,
            external_machine_executor: None,
            external_shell_review: None,
            mcp_management_sink: None,
            send_message_sink: None,
            reaction_sink: None,
            agent_management_sink: None,
            subagent_task_sink: None,
            subagent_task_review: None,
            subagent_runtime: None,
            subagent_steer_review: None,
            state_writer: None,
            routine_auto_review: None,
            routine_post_write: None,
            multitask_todo_state: None,
            cloud_agent_tool: None,
            multitask_enabled: false,
            spotlight_enabled: false,
            action_audit: None,
            observation: None,
        }
    }

    pub fn with_cursor_auth(
        mut self,
        cursor_auth: Arc<dyn CursorInferenceAuth>,
    ) -> Self {
        self.cursor_auth = Some(cursor_auth);
        self
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

    pub fn with_usage_sink(
        mut self,
        sink: Arc<dyn Fn(ProviderTokenUsage) + Send + Sync>,
    ) -> Self {
        self.usage_sink = Some(sink);
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

    pub fn with_box_shell_review(mut self, review: BoxShellAutoReviewCallback) -> Self {
        self.box_shell_review = Some(review);
        self
    }

    pub fn with_browser_executor(
        mut self,
        executor: Arc<dyn BrowserToolExecutor>,
    ) -> Self {
        self.browser_executor = Some(executor);
        self
    }

    pub fn has_browser_executor(&self) -> bool {
        self.browser_executor.is_some()
    }

    pub fn with_computer_executor(
        mut self,
        executor: Arc<dyn ComputerToolExecutor>,
    ) -> Self {
        self.computer_executor = Some(executor);
        self
    }

    pub fn has_computer_executor(&self) -> bool {
        self.computer_executor.is_some()
    }

    pub fn with_computer_exposure(mut self, exposure: ComputerToolExposure) -> Self {
        self.computer_exposure = exposure;
        self
    }


    pub fn with_file_transfer_executor(
        mut self,
        executor: Arc<dyn FileTransferExecutor>,
    ) -> Self {
        self.file_transfer_executor = Some(executor);
        self
    }

    pub fn has_file_transfer_executor(&self) -> bool {
        self.file_transfer_executor.is_some()
    }

    pub fn with_external_machine_executor(
        mut self,
        executor: Arc<dyn ExternalMachineExecutor>,
    ) -> Self {
        self.external_machine_executor = Some(executor);
        self
    }

    pub fn has_external_machine_executor(&self) -> bool {
        self.external_machine_executor.is_some()
    }

    pub fn with_external_shell_review(
        mut self,
        review: ExternalShellAutoReviewCallback,
    ) -> Self {
        self.external_shell_review = Some(review);
        self
    }

    pub fn has_external_shell_review(&self) -> bool {
        self.external_shell_review.is_some()
    }

    pub fn with_mcp_management_sink(
        mut self,
        sink: Arc<dyn McpManagementSink>,
    ) -> Self {
        self.mcp_management_sink = Some(sink);
        self
    }

    pub fn has_mcp_management_sink(&self) -> bool {
        self.mcp_management_sink.is_some()
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

    pub fn with_subagent_task_sink(mut self, sink: Arc<dyn SubagentTaskSink>) -> Self {
        self.subagent_task_sink = Some(sink);
        self
    }

    pub fn has_subagent_task_sink(&self) -> bool {
        self.subagent_task_sink.is_some()
    }

    pub fn with_subagent_task_review(mut self, review: SubagentTaskReviewCallback) -> Self {
        self.subagent_task_review = Some(review);
        self
    }

    pub fn with_subagent_management(
        mut self,
        runtime: Arc<Mutex<SubagentRuntime>>,
        review: Option<SubagentSteerReviewCallback>,
    ) -> Self {
        self.subagent_runtime = Some(runtime);
        self.subagent_steer_review = review;
        self
    }

    pub fn has_subagent_management(&self) -> bool {
        self.subagent_runtime.is_some()
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

    pub fn with_routine_auto_review(
        mut self,
        review: RoutineAutoReviewCallback,
    ) -> Self {
        self.routine_auto_review = Some(review);
        self
    }

    pub fn has_routine_auto_review(&self) -> bool {
        self.routine_auto_review.is_some()
    }

    pub fn with_routine_post_write(
        mut self,
        callback: RoutinePostWriteCallback,
    ) -> Self {
        self.routine_post_write = Some(callback);
        self
    }

    pub fn has_routine_post_write(&self) -> bool {
        self.routine_post_write.is_some()
    }

    pub fn with_multitask_todo_state(
        mut self,
        state: Arc<dyn MultitaskTodoState>,
    ) -> Self {
        self.multitask_todo_state = Some(state);
        self
    }

    pub fn has_multitask_todo_state(&self) -> bool {
        self.multitask_todo_state.is_some()
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

    pub fn with_multitask_enabled(mut self, enabled: bool) -> Self {
        self.multitask_enabled = enabled;
        self
    }

    pub fn has_multitask_enabled(&self) -> bool {
        self.multitask_enabled
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

    /// Execute the canonical per-turn MCP state projection from the exact
    /// shipping routed-tool bridge already owned by this Runner composition.
    ///
    /// This is the Rust equivalent of Grok's mcpStateExecutorResource and
    /// deliberately avoids creating a second MCP discovery owner.
    pub fn execute_mcp_state(&self) -> Result<McpStateExecResult, String> {
        execute_canonical_mcp_state(&RoutedBridgeMcpToolProvider {
            bridge: self.bridge.as_ref(),
        })
    }

    pub fn provider(&self) -> RoutedProvider {
        self.provider
    }

    pub fn cancellation(&self) -> RoutedProviderCancellation {
        self.cancellation.clone()
    }

    pub fn last_run_ended_on_silent_tool_calls(&self, final_text: &str) -> bool {
        self.latest_provider_checkpoint
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .is_some_and(|checkpoint| {
                checkpoint_ended_on_silent_tool_calls(checkpoint, final_text)
            })
    }

    fn reset_latest_provider_checkpoint(&self) {
        *self
            .latest_provider_checkpoint
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    pub fn run(
        &self,
        data_dir: &Path,
        messages: &[ProviderMessage],
        on_text_delta: &mut dyn FnMut(&str, &str),
    ) -> Result<String, ProviderSessionError> {
        self.reset_latest_provider_checkpoint();
        let bridge: Arc<dyn RoutedToolBridge> = match &self.observation {
            Some(observation) => Arc::new(McpObservedRoutedToolBridge::new(
                Arc::clone(&self.bridge),
                Arc::clone(observation),
            )),
            None => Arc::clone(&self.bridge),
        };
        let bridge: Arc<dyn RoutedToolBridge> = match &self.action_audit {
            Some(config) => Arc::new(AuditedRoutedToolBridge::new(
                bridge,
                config.clone(),
            )),
            None => bridge,
        };
        let bridge: Arc<dyn RoutedToolBridge> = match &self.mcp_management_sink {
            Some(management) => Arc::new(McpManagementToolBridge::new(
                bridge,
                Arc::clone(management),
            )),
            None => bridge,
        };
        let bridge = build_turn_toolset(
            bridge,
            TurnToolsetDependencies {
                cancellation: self.cancellation.clone(),
                box_resources: self.box_resources.clone(),
                box_shell_review: self.box_shell_review.clone(),
                browser_executor: self.browser_executor.clone(),
                computer_executor: self.computer_executor.clone(),
                computer_exposure: self.computer_exposure,
                file_transfer_executor: self.file_transfer_executor.clone(),
                external_machine_executor: self.external_machine_executor.clone(),
                external_shell_review: self.external_shell_review.clone(),
                send_message_sink: self.send_message_sink.clone(),
                reaction_sink: self.reaction_sink.clone(),
                agent_management_sink: self.agent_management_sink.clone(),
                subagent_task_sink: self.subagent_task_sink.clone(),
                subagent_task_review: self.subagent_task_review.clone(),
                subagent_runtime: self.subagent_runtime.clone(),
                subagent_steer_review: self.subagent_steer_review.clone(),
                state_writer: self.state_writer.clone(),
                routine_auto_review: self.routine_auto_review.clone(),
                routine_post_write: self.routine_post_write.clone(),
                multitask_enabled: self.multitask_enabled,
                multitask_todo_state: self.multitask_todo_state.clone(),
                cloud_agent_tool: self.cloud_agent_tool.clone(),
            },
        );
        let communicate_sink: Option<Arc<dyn CommunicateInteractionSink>> =
            self.observation.as_ref().map(|observation| {
                Arc::new(TurnObservationCommunicateSink::new(Arc::clone(observation)))
                    as Arc<dyn CommunicateInteractionSink>
            });
        let bridge: Arc<dyn RoutedToolBridge> =
            Arc::new(CommunicateRoutedToolBridge::new(bridge, communicate_sink));
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
                cursor_auth: self.cursor_auth.clone(),
                request_context: self.request_context.clone(),
                cancellation: self.cancellation.clone(),
                checkpoint_store: Arc::clone(&self.checkpoint_store),
                retry_sink: self.retry_sink.clone(),
                retry_report_sink: self.retry_report_sink.clone(),
                usage_sink: self.usage_sink.clone(),
                cloud_agents_enabled: self.cloud_agent_tool.is_some(),
                multitask_enabled: self.multitask_enabled,
            },
            on_text_delta,
        )
    }
}
