use std::sync::{Arc, Mutex};

use crate::cloud_agents::cloud_agent_tool::{
    CloudAgentToolBridge, CloudAgentToolDependencies,
};
use crate::runner::box_tool_access::{
    BoxShellAutoReviewCallback, RUNNER_BOX_READ_TOOL_NAME, RUNNER_BOX_SHELL_TOOL_NAME,
    RunnerBoxResourcePort, RunnerBoxToolBridge,
};
use crate::sand_activity::{
    SAND_BOX_AWAIT_SHELL_TOOL_NAME, SAND_EXTERNAL_AWAIT_SHELL_TOOL_NAME,
    SAND_EXTERNAL_READ_TOOL_NAME, SAND_EXTERNAL_SHELL_TOOL_NAME,
};
use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolPermissionController, SandLocalToolScope,
};
use crate::extensions::inference::provider_session::{
    ProviderPartialToolCall, ProviderSessionError, RoutedMcpMetaToolDefinition,
    RoutedToolDefinition,
};
use crate::ports::mcp_state_executor::{
    McpStateExecResult, SandMcpToolProvider, execute_mcp_state as execute_canonical_mcp_state,
};
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge,
};
use crate::runner::subagent_runtime::SubagentRuntime;

use super::box_help_tool::BoxHelpToolBridge;
use super::mcp_meta_tools::McpMetaToolBridge;
use super::sand_await_shell_tool::SandAwaitShellToolBridge;
use super::sand_agent_management_tools::{
    AgentManagementSink, AgentManagementToolBridge,
};
use super::sand_browser_tools::{BrowserToolExecutor, SandBrowserToolBridge};
use super::sand_computer_tool::{ComputerToolExecutor, ComputerToolExposure, SandComputerToolBridge};
use super::sand_file_transfer_tools::{
    COPY_FROM_BOX_TOOL_NAME, COPY_TO_BOX_TOOL_NAME, FileTransferExecutor,
    SandFileTransferToolBridge,
};
use super::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalMachineToolBridge, ExternalShellAutoReviewCallback,
};
use super::sand_reaction_tool::{ReactionSink, ReactionToolBridge};
use super::sand_spotlight_tools::SpotlightedRoutedToolBridge;
use super::sand_state_tool::{
    RoutineAutoReviewCallback, RoutinePostWriteCallback, SandStateToolBridge, SandStateWriter,
    StateApprovalBarrier,
};
use super::sand_multitask_todo_tool::{
    MultitaskTodoState, SandMultitaskTodoToolBridge,
};
use super::sand_mcp_management_tools::{McpManagementSink, McpManagementToolBridge};
use super::send_message_tool::{
    SAND_SEND_MESSAGE_TOOL_NAME, SendMessageInteractionSink, SendMessageSink,
    SendMessageToolBridge,
};
use super::sand_task_subagent_tool::{
    SubagentTaskReviewCallback, SubagentTaskSink, SubagentTaskToolBridge,
};
use super::sand_subagent_management_tools::{
    SubagentManagementToolBridge, SubagentSteerReviewCallback,
};

struct TurnToolsetMcpProvider<'a> {
    bridge: &'a dyn RoutedToolBridge,
}

impl SandMcpToolProvider for TurnToolsetMcpProvider<'_> {
    fn get_tools(&self) -> Result<Vec<RoutedToolDefinition>, String> {
        self.bridge.list_tools().map_err(|error| error.to_string())
    }
}

pub struct TurnToolsetMcpProjectionBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    projected_tools: Option<Arc<Vec<RoutedToolDefinition>>>,
}

impl TurnToolsetMcpProjectionBridge {
    fn new(
        delegate: Arc<dyn RoutedToolBridge>,
        projected_tools: Option<Arc<Vec<RoutedToolDefinition>>>,
    ) -> Self {
        Self { delegate, projected_tools }
    }
}

impl RoutedToolBridge for TurnToolsetMcpProjectionBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        if let Some(projected_tools) = self.projected_tools.as_ref() {
            return Ok(projected_tools.as_ref().clone());
        }
        let McpStateExecResult::Success(state) = execute_canonical_mcp_state(
            &TurnToolsetMcpProvider { bridge: self.delegate.as_ref() },
        )
        .map_err(|error| ProviderSessionError::Tool(format!("MCP state projection failed: {error}")))?;
        Ok(state.servers.into_iter().flat_map(|server| server.tools).map(|tool| RoutedToolDefinition {
            name: tool.name,
            provider_identifier: tool.provider_identifier,
            tool_name: tool.tool_name,
            description: tool.description,
            input_schema: tool.input_schema,
        }).collect())
    }

    fn observe_partial_tool_call(&self, partial: &ProviderPartialToolCall) -> Result<(), ProviderSessionError> {
        self.delegate.observe_partial_tool_call(partial)
    }

    fn call_tool(&self, tool: &RoutedToolDefinition, args: serde_json::Value, tool_call_id: &str) -> Result<serde_json::Value, ProviderSessionError> {
        self.delegate.call_tool(tool, args, tool_call_id)
    }
}

pub fn project_turn_mcp_toolset(
    base: Arc<dyn RoutedToolBridge>,
    projected_tools: Option<Arc<Vec<RoutedToolDefinition>>>,
    projected_meta_tools: Option<Arc<Vec<RoutedMcpMetaToolDefinition>>>,
    mcp_meta_enabled: bool,
) -> Result<Arc<dyn RoutedToolBridge>, ProviderSessionError> {
    let source_meta_tools = if mcp_meta_enabled {
        Some(match projected_meta_tools {
            Some(tools) => tools.as_ref().clone(),
            None => base.list_mcp_meta_tools()?,
        })
    } else {
        None
    };
    let bridge: Arc<dyn RoutedToolBridge> = Arc::new(TurnToolsetMcpProjectionBridge::new(base, projected_tools));
    Ok(match source_meta_tools {
        Some(tools) => Arc::new(McpMetaToolBridge::new(bridge, tools)),
        None => bridge,
    })
}

/// Per-turn Runner tool dependency projection.
///
/// The Host provides concrete capabilities; this module owns which Runner
/// tool bridges are present and their composition order for one prepared turn.
/// Cross-cutting audit/observation wrappers stay outside this owner.
#[derive(Clone, Default)]
pub struct TurnToolsetDependencies {
    pub role: TurnToolsetRole,
    pub local_tool_permission: Option<TurnLocalToolPermissionBinding>,
    pub cancellation: RoutedProviderCancellation,
    pub box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    pub box_shell_review: Option<BoxShellAutoReviewCallback>,
    pub browser_executor: Option<Arc<dyn BrowserToolExecutor>>,
    pub computer_executor: Option<Arc<dyn ComputerToolExecutor>>,
    pub computer_exposure: ComputerToolExposure,
    pub file_transfer_executor: Option<Arc<dyn FileTransferExecutor>>,
    pub external_machine_executor: Option<Arc<dyn ExternalMachineExecutor>>,
    pub external_shell_review: Option<ExternalShellAutoReviewCallback>,
    pub mcp_management_sink: Option<Arc<dyn McpManagementSink>>,
    pub send_message_sink: Option<Arc<dyn SendMessageSink>>,
    pub send_message_interaction_sink: Option<Arc<dyn SendMessageInteractionSink>>,
    pub reaction_sink: Option<Arc<dyn ReactionSink>>,
    pub agent_management_sink: Option<Arc<dyn AgentManagementSink>>,
    pub subagent_task_sink: Option<Arc<dyn SubagentTaskSink>>,
    pub subagent_task_review: Option<SubagentTaskReviewCallback>,
    pub subagent_runtime: Option<Arc<Mutex<SubagentRuntime>>>,
    pub subagent_steer_review: Option<SubagentSteerReviewCallback>,
    pub state_writer: Option<Arc<dyn SandStateWriter>>,
    pub routine_auto_review: Option<RoutineAutoReviewCallback>,
    pub state_approval_barrier: Option<StateApprovalBarrier>,
    pub routine_post_write: Option<RoutinePostWriteCallback>,
    pub multitask_enabled: bool,
    pub multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
    pub cloud_agent_tool: Option<CloudAgentToolDependencies>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnToolsetRole {
    pub is_subagent_runner: bool,
    pub is_shared_room_runner: bool,
    pub is_box_scoped_subagent: bool,
    pub is_browser_use_subagent: bool,
    pub shared_room_box_tools_enabled: bool,
}

impl Default for TurnToolsetRole {
    fn default() -> Self {
        Self {
            is_subagent_runner: false,
            is_shared_room_runner: false,
            is_box_scoped_subagent: false,
            is_browser_use_subagent: false,
            shared_room_box_tools_enabled: true,
        }
    }
}

#[derive(Clone)]
pub struct TurnLocalToolPermissionBinding {
    pub controller: Arc<SandLocalToolPermissionController>,
    pub agent_id: String,
}

struct LocalToolScopeBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    binding: TurnLocalToolPermissionBinding,
}

impl LocalToolScopeBridge {
    fn scoped_action(tool: &RoutedToolDefinition) -> Option<Option<&'static str>> {
        let names = [tool.name.as_str(), tool.tool_name.as_str()];
        if names.contains(&SAND_EXTERNAL_SHELL_TOOL_NAME) {
            return Some(Some("run-command"));
        }
        if names.contains(&SAND_EXTERNAL_READ_TOOL_NAME)
            || names.contains(&SAND_EXTERNAL_AWAIT_SHELL_TOOL_NAME)
        {
            return Some(Some("read-file"));
        }
        if names.contains(&RUNNER_BOX_SHELL_TOOL_NAME)
            || names.contains(&RUNNER_BOX_READ_TOOL_NAME)
            || names.contains(&SAND_BOX_AWAIT_SHELL_TOOL_NAME)
            || names.contains(&COPY_TO_BOX_TOOL_NAME)
            || names.contains(&COPY_FROM_BOX_TOOL_NAME)
        {
            return Some(None);
        }
        None
    }
}

impl RoutedToolBridge for LocalToolScopeBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        self.delegate.list_tools()
    }

    fn list_mcp_meta_tools(
        &self,
    ) -> Result<Vec<RoutedMcpMetaToolDefinition>, ProviderSessionError> {
        self.delegate.list_mcp_meta_tools()
    }

    fn observe_partial_tool_call(
        &self,
        partial: &ProviderPartialToolCall,
    ) -> Result<(), ProviderSessionError> {
        self.delegate.observe_partial_tool_call(partial)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: serde_json::Value,
        tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        let Some(action) = Self::scoped_action(tool) else {
            return self.delegate.call_tool(tool, args, tool_call_id);
        };
        let scope = SandLocalToolScope {
            agent_id: self.binding.agent_id.clone(),
            tool_call_id: Some(tool_call_id.to_string()),
            action: action.map(str::to_string),
            direction_epoch: Some(
                self.binding
                    .controller
                    .direction_epoch(&self.binding.agent_id),
            ),
        };
        let result = self.delegate.call_tool(tool, args, tool_call_id);
        self.binding.controller.complete_scope(Some(&scope));
        result
    }
}

struct SharedRoomToolFilterBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    box_tools_enabled: bool,
}

impl SharedRoomToolFilterBridge {
    fn allows(&self, tool: &RoutedToolDefinition) -> bool {
        if tool.name == SAND_SEND_MESSAGE_TOOL_NAME || tool.tool_name == SAND_SEND_MESSAGE_TOOL_NAME {
            return true;
        }
        self.box_tools_enabled
            && [
                RUNNER_BOX_SHELL_TOOL_NAME,
                RUNNER_BOX_READ_TOOL_NAME,
                SAND_BOX_AWAIT_SHELL_TOOL_NAME,
                "Screenshot",
            ]
            .iter()
            .any(|allowed| tool.name == *allowed || tool.tool_name == *allowed)
    }
}

impl RoutedToolBridge for SharedRoomToolFilterBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(self
            .delegate
            .list_tools()?
            .into_iter()
            .filter(|tool| self.allows(tool))
            .collect())
    }

    fn list_mcp_meta_tools(
        &self,
    ) -> Result<Vec<RoutedMcpMetaToolDefinition>, ProviderSessionError> {
        Ok(self
            .delegate
            .list_mcp_meta_tools()?
            .into_iter()
            .filter(|meta| self.allows(&meta.tool))
            .collect())
    }

    fn observe_partial_tool_call(
        &self,
        partial: &ProviderPartialToolCall,
    ) -> Result<(), ProviderSessionError> {
        self.delegate.observe_partial_tool_call(partial)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: serde_json::Value,
        tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        if !self.allows(tool) {
            return Err(ProviderSessionError::Tool(format!(
                "{} is not available in a shared-room Runner",
                tool.name
            )));
        }
        self.delegate.call_tool(tool, args, tool_call_id)
    }
}

pub fn build_turn_toolset(
    base: Arc<dyn RoutedToolBridge>,
    dependencies: TurnToolsetDependencies,
) -> Arc<dyn RoutedToolBridge> {
    let role = dependencies.role;
    let await_box_resources = dependencies.box_resources.clone();
    let await_external_machine = dependencies.external_machine_executor.clone();
    let await_cancellation = dependencies.cancellation.clone();
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.mcp_management_sink) {
            (false, Some(management)) => Arc::new(McpManagementToolBridge::new(base, management)),
            _ => base,
        };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.box_resources {
        Some(box_resources) => {
            let mut box_bridge = RunnerBoxToolBridge::new(bridge, box_resources);
            if let Some(review) = dependencies.box_shell_review {
                box_bridge = box_bridge.with_shell_review(review);
            }
            Arc::new(box_bridge)
        }
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_browser_use_subagent, dependencies.browser_executor) {
            (true, Some(executor)) => Arc::new(SandBrowserToolBridge::new(bridge, executor)),
            _ => bridge,
        };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.computer_executor {
        Some(executor) => Arc::new(
            SandComputerToolBridge::new(bridge, executor)
                .with_exposure(dependencies.computer_exposure),
        ),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_box_scoped_subagent, dependencies.external_machine_executor) {
        (false, Some(executor)) => {
            let mut external = ExternalMachineToolBridge::new(bridge, executor);
            if let Some(review) = dependencies.external_shell_review {
                external = external.with_shell_review(review);
            }
            Arc::new(external)
        }
        _ => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_box_scoped_subagent, dependencies.file_transfer_executor) {
            (false, Some(executor)) => Arc::new(SandFileTransferToolBridge::new(bridge, executor)),
            _ => bridge,
        };
    let bridge: Arc<dyn RoutedToolBridge> = if !role.is_box_scoped_subagent
        && (await_box_resources.is_some() || await_external_machine.is_some())
    {
        Arc::new(SandAwaitShellToolBridge::new(
            bridge,
            await_box_resources,
            await_external_machine,
            await_cancellation,
        ))
    } else {
        bridge
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.reaction_sink) {
            (false, Some(sink)) => Arc::new(ReactionToolBridge::new(bridge, sink)),
            _ => bridge,
        };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.agent_management_sink) {
            (false, Some(sink)) => Arc::new(AgentManagementToolBridge::new(bridge, sink)),
            _ => bridge,
        };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.subagent_task_sink) {
        (false, Some(sink)) => {
            let mut task_bridge = SubagentTaskToolBridge::new(bridge, sink);
            if let Some(review) = dependencies.subagent_task_review {
                task_bridge = task_bridge.with_review(review);
            }
            Arc::new(task_bridge)
        }
        _ => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.subagent_runtime) {
        (false, Some(runtime)) => Arc::new(SubagentManagementToolBridge::new(
            bridge,
            runtime,
            dependencies.subagent_steer_review,
        )),
        _ => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, dependencies.state_writer) {
        (false, Some(state)) => {
            let mut state_bridge = SandStateToolBridge::new(bridge, state);
            if let Some(review) = dependencies.routine_auto_review {
                state_bridge = state_bridge.with_routine_auto_review(review);
            }
            if let Some(barrier) = dependencies.state_approval_barrier {
                state_bridge = state_bridge.with_approval_barrier(barrier);
            }
            if let Some(callback) = dependencies.routine_post_write {
                state_bridge = state_bridge.with_routine_post_write(callback);
            }
            Arc::new(state_bridge)
        }
        _ => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        if !role.is_subagent_runner && dependencies.multitask_enabled {
        match dependencies.multitask_todo_state {
            Some(state) => Arc::new(SandMultitaskTodoToolBridge::new(bridge, state)),
            None => bridge,
        }
    } else {
        bridge
    };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_box_scoped_subagent, dependencies.cloud_agent_tool) {
            (false, Some(deps)) => Arc::new(CloudAgentToolBridge::new(bridge, deps)),
            _ => bridge,
        };
    let bridge: Arc<dyn RoutedToolBridge> =
        match (role.is_subagent_runner, &dependencies.send_message_sink) {
        (false, Some(sink)) => Arc::new(BoxHelpToolBridge::new(
            bridge,
            Arc::clone(sink),
            dependencies.cancellation.clone(),
        )),
        _ => bridge,
    };
    let bridge = match (role.is_subagent_runner, dependencies.send_message_sink) {
        (false, Some(sink)) => {
            let mut send_bridge = SendMessageToolBridge::new(bridge, sink);
            if let Some(interaction_sink) = dependencies.send_message_interaction_sink {
                send_bridge = send_bridge.with_interaction_sink(interaction_sink);
            }
            Arc::new(send_bridge)
        }
        _ => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.local_tool_permission {
        Some(binding) => Arc::new(LocalToolScopeBridge {
            delegate: bridge,
            binding,
        }),
        None => bridge,
    };
    if role.is_shared_room_runner {
        Arc::new(SharedRoomToolFilterBridge {
            delegate: bridge,
            box_tools_enabled: role.shared_room_box_tools_enabled,
        })
    } else {
        bridge
    }
}

pub fn fence_turn_toolset(
    bridge: Arc<dyn RoutedToolBridge>,
    spotlight_enabled: bool,
) -> Arc<dyn RoutedToolBridge> {
    if spotlight_enabled {
        Arc::new(SpotlightedRoutedToolBridge::new(bridge))
    } else {
        bridge
    }
}
