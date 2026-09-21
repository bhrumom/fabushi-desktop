import React from 'react';
import AgentRootShell from '../agent-workspace/agent-root-shell';

/**
 * Single desktop product entry.
 *
 * AgentRootShell is selected by the app layer. Legacy messaging, contacts,
 * Telegram and MiniApp compatibility can render inside it, but adapters cannot
 * replace the root product boundary.
 */
export default function DesktopApp() {
  return <AgentRootShell />;
}
