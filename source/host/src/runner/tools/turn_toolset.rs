use std::sync::{Arc, Mutex};

use crate::cloud_agents::cloud_agent_tool::{
    CloudAgentToolBridge, CloudAgentToolDependencies,
};
use crate::runner::box_tool_access::{
    BoxShellAutoReviewCallback, RunnerBoxResourcePort, RunnerBoxToolBridge,
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
use super::sand_file_transfer_tools::{FileTransferExecutor, SandFileTransferToolBridge};
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
use super::send_message_tool::{SendMessageInteractionSink, SendMessageSink, SendMessageToolBridge};
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

pub fn build_turn_toolset(
    base: Arc<dyn RoutedToolBridge>,
    dependencies: TurnToolsetDependencies,
) -> Arc<dyn RoutedToolBridge> {
    let await_box_resources = dependencies.box_resources.clone();
    let await_external_machine = dependencies.external_machine_executor.clone();
    let await_cancellation = dependencies.cancellation.clone();
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.mcp_management_sink {
        Some(management) => Arc::new(McpManagementToolBridge::new(base, management)),
        None => base,
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
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.browser_executor {
        Some(executor) => Arc::new(SandBrowserToolBridge::new(bridge, executor)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.computer_executor {
        Some(executor) => Arc::new(
            SandComputerToolBridge::new(bridge, executor)
                .with_exposure(dependencies.computer_exposure),
        ),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.external_machine_executor {
        Some(executor) => {
            let mut external = ExternalMachineToolBridge::new(bridge, executor);
            if let Some(review) = dependencies.external_shell_review {
                external = external.with_shell_review(review);
            }
            Arc::new(external)
        }
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.file_transfer_executor {
        Some(executor) => Arc::new(SandFileTransferToolBridge::new(bridge, executor)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = if await_box_resources.is_some() || await_external_machine.is_some() {
        Arc::new(SandAwaitShellToolBridge::new(
            bridge,
            await_box_resources,
            await_external_machine,
            await_cancellation,
        ))
    } else {
        bridge
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.reaction_sink {
        Some(sink) => Arc::new(ReactionToolBridge::new(bridge, sink)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.agent_management_sink {
        Some(sink) => Arc::new(AgentManagementToolBridge::new(bridge, sink)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.subagent_task_sink {
        Some(sink) => {
            let mut task_bridge = SubagentTaskToolBridge::new(bridge, sink);
            if let Some(review) = dependencies.subagent_task_review {
                task_bridge = task_bridge.with_review(review);
            }
            Arc::new(task_bridge)
        }
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.subagent_runtime {
        Some(runtime) => Arc::new(SubagentManagementToolBridge::new(
            bridge,
            runtime,
            dependencies.subagent_steer_review,
        )),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.state_writer {
        Some(state) => {
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
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = if dependencies.multitask_enabled {
        match dependencies.multitask_todo_state {
            Some(state) => Arc::new(SandMultitaskTodoToolBridge::new(bridge, state)),
            None => bridge,
        }
    } else {
        bridge
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.cloud_agent_tool {
        Some(deps) => Arc::new(CloudAgentToolBridge::new(bridge, deps)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match &dependencies.send_message_sink {
        Some(sink) => Arc::new(BoxHelpToolBridge::new(
            bridge,
            Arc::clone(sink),
            dependencies.cancellation.clone(),
        )),
        None => bridge,
    };
    match dependencies.send_message_sink {
        Some(sink) => {
            let mut send_bridge = SendMessageToolBridge::new(bridge, sink);
            if let Some(interaction_sink) = dependencies.send_message_interaction_sink {
                send_bridge = send_bridge.with_interaction_sink(interaction_sink);
            }
            Arc::new(send_bridge)
        }
        None => bridge,
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
