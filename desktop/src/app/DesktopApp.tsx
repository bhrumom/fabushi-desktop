import React from 'react';
import AgentRootShell from '../agent-workspace/agent-root-shell';

/**
 * Single desktop product entry.
 *
 * AgentRootShell owns the Agent product layout and lifecycle. Compatibility
 * features plug into that boundary but cannot replace it.
 */
export default function DesktopApp() {
  return <AgentRootShell />;
}
