import React from 'react';
import AgentRootShell from '../agent-workspace/agent-root-shell';

/**
 * Single desktop product entry.
 *
 * AgentRootShell owns the Agent Sidebar, conversation workspace, transcript,
 * composer, Context/Computer surfaces and Agent controllers. Compatibility
 * features are mounted by explicit adapters from inside that product boundary.
 */
export default function DesktopApp() {
  return <AgentRootShell />;
}
