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

function installOptionalBridge(name: string, install: () => unknown): void {
  try {
    install();
  } catch (error) {
    console.error(`Fabushi desktop ${name} bridge failed to install`, error);
  }
}

async function hydrateCompatibilityProjection(): Promise<void> {
  try {
    await restoreDurableAgentState();
  } catch (error) {
    console.error('Fabushi desktop compatibility projection restore failed', error);
  }
  installOptionalBridge('durable Agent state', installDurableAgentState);
}

function bootstrapDesktop(rootElement: HTMLDivElement): void {
  // Product shell first: compatibility hydration, Mini Apps, and auxiliary
  // bridges must never be able to leave a packaged build on a blank first
  // frame. Runtime/Auth readiness is rendered explicitly by DesktopAuthBoundary.
  createRoot(rootElement).render(
    <StrictMode>
      <DesktopApp />
      <CredentialVault />
    </StrictMode>,
  );

  installOptionalBridge('account session sync', installDesktopAccountSessionSync);
  installOptionalBridge('FabAvatar identity alias', installFabAvatarIdentityAliases);
  installOptionalBridge('Mini App discovery alias', installDesktopMiniAppDiscoveryAliases);
  installOptionalBridge('Mini App WebMCP host', installDesktopMiniAppWebMcpHost);
  installOptionalBridge('Agent surface', installDesktopAppAgentSurface);
  installOptionalBridge('Mini App composer', () => installMiniAppComposerOpenBridge(rootElement));
  installOptionalBridge('self-hosted Mahayana invocation', installSelfHostedMahayanaInvocationBridge);

  // Compatibility-only local projections restore after the shell mounts.
  // Rust RuntimeStore remains authoritative for Agent-owned durable state.
  void hydrateCompatibilityProjection();
}

bootstrapDesktop(root);
