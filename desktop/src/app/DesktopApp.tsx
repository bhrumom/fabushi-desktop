import React from 'react';
import AgentRootShell from '../agent-workspace/agent-root-shell';
import DesktopAuthBoundary from './desktop-auth-boundary';

export default function DesktopApp() {
  return <DesktopAuthBoundary>
    {({ transport, onLogout }) => <AgentRootShell transport={transport} onLogout={onLogout} />}
  </DesktopAuthBoundary>;
}
