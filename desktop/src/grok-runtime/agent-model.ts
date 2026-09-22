/**
 * Compatibility aliases for the recovered Grok presentation layer.
 *
 * Canonical Agent navigation/projection ownership lives in
 * agent-workspace/agent-model.ts. New product code must import that module.
 */
export {
  agentWorkspaceKey as grokAgentKey,
  projectActiveAgentKey as projectActiveGrokAgentKey,
  projectAgentSidebarItems as projectGrokAgentSidebarItems,
  type AgentActivityProjection as GrokAgentActivityProjection,
  type AgentPeerProjection as GrokAgentPeerProjection,
  type AgentSidebarItem as GrokAgentSidebarItem,
} from '../agent-workspace/agent-model';
