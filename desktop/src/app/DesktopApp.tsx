import React from 'react';
import AgentRootShell from '../agent-workspace/agent-root-shell';
import DesktopAuthBoundary from './desktop-auth-boundary';

export default function DesktopApp() {
  return <DesktopAuthBoundary>
    {({ onLogout }) => <AgentRootShell onLogout={onLogout} />}
  </DesktopAuthBoundary>;
}
