import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import DesktopShellV2 from './messaging-shell-v2';
import MahayanaAgentWorkbench from './mahayana-agent-workbench';
import { installMahayanaAgentTranscriptSemantics } from './mahayana-agent-transcript-semantics';
import { installSelfHostedMahayanaInvocationBridge } from './selfhosted-mahayana-invocation-bridge';
import './messenger-layout-regressions.css';
import './grok-agent-ui-parity.css';
import './mahayana-agent-transcript-semantics.css';

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

installMahayanaAgentTranscriptSemantics();
installSelfHostedMahayanaInvocationBridge();
