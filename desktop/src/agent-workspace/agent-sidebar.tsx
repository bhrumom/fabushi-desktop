import React from 'react';
import GrokAgentSidebar, {
  type GrokAgentSidebarItem,
  type GrokAgentSidebarProps,
} from '../grok-shell/grok-agent-sidebar';

export type AgentSidebarItem = GrokAgentSidebarItem;
export type AgentSidebarProps = GrokAgentSidebarProps;

/**
 * Stable Agent-workspace boundary for the primary desktop navigation.
 *
 * The recovered Grok component remains the visual implementation for now, but
 * Messenger code imports this boundary so the product shell no longer depends
 * on a Grok-specific presentation module.
 */
export default function AgentSidebar(props: AgentSidebarProps) {
  return <GrokAgentSidebar {...props} />;
}
