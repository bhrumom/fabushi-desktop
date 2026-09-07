import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { installDesktopAccountSessionSync } from './account-session-sync';
import { installDesktopAppAgentSurface } from './app-agent-surface';
import { installBotIdentityAliases } from './agent-identity-aliases';
import CredentialVault from './credential-vault';
import { installDurableAgentState, restoreDurableAgentState } from './durable-agent-state';
import { GrokChatParityRuntime, prepareGrokChatParityRuntime } from './grok-chat-parity-runtime';
import DesktopShellV2 from './messaging-shell-v2';
import { installMahayanaAgentInlineCompatibility } from './mahayana-agent-inline-compat';
import MahayanaAgentInlineReport from './mahayana-agent-inline-report';
import MahayanaAgentWorkbench from './mahayana-agent-workbench';
import { installMahayanaAgentTranscriptSemantics } from './mahayana-agent-transcript-semantics';
import { installDesktopMiniAppDiscoveryAliases } from './miniapp-discovery-aliases';
import { installDesktopMiniAppWebMcpHost } from './miniapp-webmcp-host';
import { installSelfHostedMahayanaInvocationBridge } from './selfhosted-mahayana-invocation-bridge';
import './messenger-layout-regressions.css';
import './grok-agent-ui-parity.css';
import './mahayana-agent-transcript-semantics.css';
import './credential-vault.css';
import './sidebar-contact-groups.css';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

async function bootstrapDesktop(rootElement: HTMLDivElement): Promise<void> {
  installDesktopAccountSessionSync();
  // Restore native persisted projections before transport/workbench reducers
  // read their first-frame local cache. This makes localStorage a projection;
  // canonical cloud/Rust authority is verified separately by GBF-601/602.
  await restoreDurableAgentState();
  prepareGrokChatParityRuntime();
  installBotIdentityAliases();
  installDesktopMiniAppDiscoveryAliases();
  installDurableAgentState();
  installDesktopMiniAppWebMcpHost();
  installDesktopAppAgentSurface();

  createRoot(rootElement).render(
    <StrictMode>
      <DesktopShellV2 />
      <GrokChatParityRuntime />
      <MahayanaAgentWorkbench />
      <MahayanaAgentInlineReport />
      <CredentialVault />
    </StrictMode>,
  );

  installMahayanaAgentInlineCompatibility();
  installMahayanaAgentTranscriptSemantics();
  installSelfHostedMahayanaInvocationBridge();
}

void bootstrapDesktop(root).catch((error: unknown) => {
  console.error('Fabushi desktop bootstrap failed', error);
});
