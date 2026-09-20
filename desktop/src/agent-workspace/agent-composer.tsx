import React from 'react';
import GrokAgentComposer from '../grok-shell/grok-agent-composer';

export type AgentComposerProps = React.ComponentProps<typeof GrokAgentComposer>;

/**
 * Agent-owned composer boundary. Draft, attachment, reply and voice state enter
 * the primary workspace through this component rather than through Messenger.
 */
export default function AgentComposer(props: AgentComposerProps) {
  return <GrokAgentComposer {...props} />;
}
