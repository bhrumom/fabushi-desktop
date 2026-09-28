use std::sync::Arc;

use crate::cloud_agents::cloud_agent_tool::{
    CloudAgentToolBridge, CloudAgentToolDependencies,
};
use crate::runner::box_tool_access::{RunnerBoxResourcePort, RunnerBoxToolBridge};
use crate::runner::routed_provider_runtime::{
    RoutedProviderCancellation, RoutedToolBridge,
};

use super::box_help_tool::BoxHelpToolBridge;
use super::sand_agent_management_tools::{
    AgentManagementSink, AgentManagementToolBridge,
};
use super::sand_browser_tools::{BrowserToolExecutor, SandBrowserToolBridge};
use super::sand_computer_tool::{ComputerToolExecutor, SandComputerToolBridge};
use super::sand_file_transfer_tools::{FileTransferExecutor, SandFileTransferToolBridge};
use super::sand_reaction_tool::{ReactionSink, ReactionToolBridge};
use super::sand_spotlight_tools::SpotlightedRoutedToolBridge;
use super::sand_state_tool::{SandStateToolBridge, SandStateWriter};
use super::sand_multitask_todo_tool::{
    MultitaskTodoState, SandMultitaskTodoToolBridge,
};
use super::send_message_tool::{SendMessageSink, SendMessageToolBridge};

/// Per-turn Runner tool dependency projection.
///
/// The Host provides concrete capabilities; this module owns which Runner
/// tool bridges are present and their composition order for one prepared turn.
/// Cross-cutting audit/observation wrappers stay outside this owner.
#[derive(Clone, Default)]
pub struct TurnToolsetDependencies {
    pub cancellation: RoutedProviderCancellation,
    pub box_resources: Option<Arc<dyn RunnerBoxResourcePort>>,
    pub browser_executor: Option<Arc<dyn BrowserToolExecutor>>,
    pub computer_executor: Option<Arc<dyn ComputerToolExecutor>>,
    pub file_transfer_executor: Option<Arc<dyn FileTransferExecutor>>,
    pub send_message_sink: Option<Arc<dyn SendMessageSink>>,
    pub reaction_sink: Option<Arc<dyn ReactionSink>>,
    pub agent_management_sink: Option<Arc<dyn AgentManagementSink>>,
    pub state_writer: Option<Arc<dyn SandStateWriter>>,
    pub multitask_enabled: bool,
    pub multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
    pub cloud_agent_tool: Option<CloudAgentToolDependencies>,
}

pub fn build_turn_toolset(
    base: Arc<dyn RoutedToolBridge>,
    dependencies: TurnToolsetDependencies,
) -> Arc<dyn RoutedToolBridge> {
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.box_resources {
        Some(box_resources) => Arc::new(RunnerBoxToolBridge::new(
            base,
            box_resources,
        )),
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
    let bridge: Arc<dyn RoutedToolBridge> = match dependencies.state_writer {
        Some(state) => Arc::new(SandStateToolBridge::new(bridge, state)),
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
