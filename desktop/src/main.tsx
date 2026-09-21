import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { installDesktopAccountSessionSync } from './account-session-sync';
import { installDesktopAppAgentSurface } from './app-agent-surface';
import { installFabAvatarIdentityAliases } from './agent-identity-aliases';
import CredentialVault from './credential-vault';
import { installDurableAgentState, restoreDurableAgentState } from './durable-agent-state';
import DesktopApp from './app/DesktopApp';
import { installMiniAppComposerOpenBridge } from './miniapp-composer-open-bridge';
import { installDesktopMiniAppDiscoveryAliases } from './miniapp-discovery-aliases';
import { installDesktopMiniAppWebMcpHost } from './miniapp-webmcp-host';
import { installSelfHostedMahayanaInvocationBridge } from './selfhosted-mahayana-invocation-bridge';
import './messenger-layout-regressions.css';
import './grok-agent-ui-parity.css';
import './openbot-ui-parity.css';
import './mahayana-assistant-turn.css';
import './credential-vault.css';
import './sidebar-contact-groups.css';
import './ui/tokens.css';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

async function bootstrapDesktop(rootElement: HTMLDivElement): Promise<void> {
  installDesktopAccountSessionSync();
  // Restore native persisted projections before Agent/runtime reducers read
  // their first-frame local cache. This makes localStorage a projection;
  // canonical cloud/Rust authority is verified separately by GBF-601/602.
  await restoreDurableAgentState();
  installFabAvatarIdentityAliases();
  installDesktopMiniAppDiscoveryAliases();
  installDurableAgentState();
  installDesktopMiniAppWebMcpHost();
  installDesktopAppAgentSurface();

  createRoot(rootElement).render(
    <StrictMode>
      <DesktopApp />
      <CredentialVault />
    </StrictMode>,
  );

  installMiniAppComposerOpenBridge(rootElement);
  installSelfHostedMahayanaInvocationBridge();
}

void bootstrapDesktop(root).catch((error: unknown) => {
  console.error('Fabushi desktop bootstrap failed', error);
});
