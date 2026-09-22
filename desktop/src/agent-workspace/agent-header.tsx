import React from 'react';
import GrokAgentHeader from '../grok-shell/grok-agent-header';

export type AgentHeaderProps = React.ComponentProps<typeof GrokAgentHeader>;

/**
 * Agent-owned header boundary. Presentation can continue tracking the pinned
 * Fabu/Grok reference without coupling the workspace controller to grok-shell.
 */
export default function AgentHeader(props: AgentHeaderProps) {
  return <GrokAgentHeader {...props} />;
}
