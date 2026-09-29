use std::sync::{Arc, Mutex};

use crate::cloud_agents::cloud_agent_tool::{
    CloudAgentToolBridge, CloudAgentToolDependencies,
};
use crate::runner::box_tool_access::{
    BoxShellAutoReviewCallback, RunnerBoxResourcePort, RunnerBoxToolBridge,
};
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge,
};
use crate::runner::subagent_runtime::SubagentRuntime;

use super::box_help_tool::BoxHelpToolBridge;
use super::sand_agent_management_tools::{
    AgentManagementSink, AgentManagementToolBridge,
};
use super::sand_browser_tools::{BrowserToolExecutor, SandBrowserToolBridge};
use super::sand_computer_tool::{ComputerToolExecutor, SandComputerToolBridge};
use super::sand_file_transfer_tools::{FileTransferExecutor, SandFileTransferToolBridge};
use super::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalMachineToolBridge, ExternalShellAutoReviewCallback,
};
use super::sand_reaction_tool::{ReactionSink, ReactionToolBridge};
use super::sand_spotlight_tools::SpotlightedRoutedToolBridge;
use super::sand_state_tool::{
    RoutineAutoReviewCallback, RoutinePostWriteCallback, SandStateToolBridge, SandStateWriter,
};
use super::sand_multitask_todo_tool::{
    MultitaskTodoState, SandMultitaskTodoToolBridge,
};
use super::send_message_tool::{SendMessageSink, SendMessageToolBridge};
use super::sand_task_subagent_tool::{
    SubagentTaskReviewCallback, SubagentTaskSink, SubagentTaskToolBridge,
};
use super::sand_subagent_management_tools::{
    SubagentManagementToolBridge, SubagentSteerReviewCallback,
};

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
    pub file_transfer_executor: Option<Arc<dyn FileTransferExecutor>>,
    pub external_machine_executor: Option<Arc<dyn ExternalMachineExecutor>>,
    pub external_shell_review: Option<ExternalShellAutoReviewCallback>,
    pub send_message_sink: Option<Arc<dyn SendMessageSink>>,
    pub reaction_sink: Option<Arc<dyn ReactionSink>>,
    pub agent_management_sink: Option<Arc<dyn AgentManagementSink>>,
    pub subagent_task_sink: Option<Arc<dyn SubagentTaskSink>>,
    pub subagent_task_review: Option<SubagentTaskReviewCallback>,
    pub subagent_runtime: Option<Arc<Mutex<SubagentRuntime>>>,
    pub subagent_steer_review: Option<SubagentSteerReviewCallback>,
    pub state_writer: Option<Arc<dyn SandStateWriter>>,
    pub routine_auto_review: Option<RoutineAutoReviewCallback>,
    pub routine_post_write: Option<RoutinePostWriteCallback>,
    pub multitask_enabled: bool,
    pub multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
    pub cloud_agent_tool: Option<CloudAgentToolDependencies>,
}

pub fn build_turn_toolset(
    base: Arc<dyn RoutedToolBridge>,
    dependencies: TurnToolsetDependencies,
) -> Arc<dyn RoutedToolBridge> {
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.box_resources {
        Some(box_resources) => {
            let mut box_bridge = RunnerBoxToolBridge::new(base, box_resources);
            if let Some(review) = dependencies.box_shell_review {
                box_bridge = box_bridge.with_shell_review(review);
            }
            Arc::new(box_bridge)
        }
        None => base,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.browser_executor {
        Some(executor) => Arc::new(SandBrowserToolBridge::new(bridge, executor)),
        None => bridge,
    };
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.computer_executor {
        Some(executor) => Arc::new(SandComputerToolBridge::new(bridge, executor)),
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
        Some(sink) => Arc::new(SendMessageToolBridge::new(bridge, sink)),
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
