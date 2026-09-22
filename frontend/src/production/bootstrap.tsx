import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { installDesktopAccountSessionSync } from '../../../desktop/src/account-session-sync';
import { installDesktopAppAgentSurface } from '../../../desktop/src/app-agent-surface';
import { installFabAvatarIdentityAliases } from '../../../desktop/src/agent-identity-aliases';
import { installDurableAgentState, restoreDurableAgentState } from '../../../desktop/src/durable-agent-state';
import { installMiniAppComposerOpenBridge } from '../../../desktop/src/miniapp-composer-open-bridge';
import { installDesktopMiniAppDiscoveryAliases } from '../../../desktop/src/miniapp-discovery-aliases';
import { installDesktopMiniAppWebMcpHost } from '../../../desktop/src/miniapp-webmcp-host';
import { installSelfHostedMahayanaInvocationBridge } from '../../../desktop/src/selfhosted-mahayana-invocation-bridge';
import '../../../desktop/src/messenger-layout-regressions.css';
import '../../../desktop/src/grok-agent-ui-parity.css';
import '../../../desktop/src/openbot-ui-parity.css';
import '../../../desktop/src/mahayana-assistant-turn.css';
import '../../../desktop/src/credential-vault.css';
import '../../../desktop/src/sidebar-contact-groups.css';
import '../../../desktop/src/ui/tokens.css';
import ProductionRenderer from './ProductionRenderer';

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

export function bootstrapProductionRenderer(rootElement: HTMLDivElement): void {
  createRoot(rootElement).render(
    <StrictMode>
      <ProductionRenderer />
    </StrictMode>,
  );

  installOptionalBridge('account session sync', installDesktopAccountSessionSync);
  installOptionalBridge('FabAvatar identity alias', installFabAvatarIdentityAliases);
  installOptionalBridge('Mini App discovery alias', installDesktopMiniAppDiscoveryAliases);
  installOptionalBridge('Mini App WebMCP host', installDesktopMiniAppWebMcpHost);
  installOptionalBridge('Agent surface', installDesktopAppAgentSurface);
  installOptionalBridge('Mini App composer', () => installMiniAppComposerOpenBridge(rootElement));
  installOptionalBridge('self-hosted Mahayana invocation', installSelfHostedMahayanaInvocationBridge);

  // Compatibility-only projections restore after first paint while migration
  // continues. The canonical renderer root itself now lives under frontend/**.
  void hydrateCompatibilityProjection();
}
