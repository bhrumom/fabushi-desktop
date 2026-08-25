import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import DesktopShellV2 from './messaging-shell-v2';
import MahayanaAgentWorkbench from './mahayana-agent-workbench';
import './messenger-layout-regressions.css';
import './grok-agent-ui-parity.css';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

createRoot(root).render(
  <StrictMode>
    <DesktopShellV2 />
    <MahayanaAgentWorkbench />
  </StrictMode>,
);
