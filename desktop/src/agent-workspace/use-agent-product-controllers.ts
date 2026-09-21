import type { BotSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';
import { useAgentCommandPaletteController } from './use-agent-command-palette-controller';
import { useAgentSidebarController } from './use-agent-sidebar-controller';
import {
  useAgentNetworkController,
  type AgentNetworkController,
} from './use-agent-network-controller';
import {
  useAgentWorkflowController,
  type AgentWorkflowController,
  type AgentWorkflowControllerOptions,
} from './use-agent-workflow-controller';
import {
  useAgentMcpController,
  type AgentMcpController,
  type AgentMcpControllerOptions,
} from './use-agent-mcp-controller';
import {
  useAgentPrSuggestionController,
  type AgentPrSuggestionController,
} from './use-agent-pr-suggestion-controller';
import {
  useAgentStoreSyncController,
  type AgentStoreSyncController,
  type AgentStoreSyncControllerOptions,
} from './use-agent-store-sync-controller';
import {
  useAgentDirectoryController,
  type AgentDirectoryController,
  type AgentDirectoryControllerOptions,
} from './use-agent-directory-controller';
import type { AgentCommandPaletteController } from './use-agent-command-palette-controller';
import type { AgentSidebarController } from './use-agent-sidebar-controller';

export interface AgentProductControllersOptions {
  readonly client: AgentCoordinatorClient;
  readonly accountScope: string | null | undefined;
  readonly initialAgents?: readonly BotSummary[];
  readonly onError?: (message: string) => void;
  readonly workflow?: AgentWorkflowControllerOptions;
  readonly mcp?: AgentMcpControllerOptions;
  readonly storeSync: AgentStoreSyncControllerOptions;
  readonly directory?: AgentDirectoryControllerOptions;
}

export interface AgentProductControllers {
  readonly palette: AgentCommandPaletteController;
  readonly sidebar: AgentSidebarController;
  readonly network: AgentNetworkController;
  readonly workflow: AgentWorkflowController;
  readonly mcp: AgentMcpController;
  readonly pullRequests: AgentPrSuggestionController;
  readonly storeSync: AgentStoreSyncController;
  readonly directory: AgentDirectoryController;
}

/**
 * Product-owned composition boundary for the Agent control plane.
 *
 * Compatibility adapters may consume these projections and callbacks, but
 * cannot individually construct the Agent Sidebar/Network/Workflow/MCP/store
 * state machines. This keeps their lifecycle and cross-controller dependency
 * graph inside the Agent domain.
 */
export function useAgentProductControllers(
  options: AgentProductControllersOptions,
): AgentProductControllers {
  const palette = useAgentCommandPaletteController();
  const sidebar = useAgentSidebarController(options.accountScope);
  const network = useAgentNetworkController(options.client, options.onError);
  const workflow = useAgentWorkflowController(options.client, options.workflow);
  const mcp = useAgentMcpController(options.client, options.mcp);
  const pullRequests = useAgentPrSuggestionController(options.client, mcp.pullRequestTool);
  const storeSync = useAgentStoreSyncController(options.client, options.storeSync);
  const directory = useAgentDirectoryController(
    options.client,
    options.initialAgents ?? [],
    options.directory,
  );

  return {
    palette,
    sidebar,
    network,
    workflow,
    mcp,
    pullRequests,
    storeSync,
    directory,
  };
}
