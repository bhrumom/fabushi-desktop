# Desktop source dependency closure

The desktop TypeScript/Vite source imports a small set of shared monorepo modules outside `desktop/`. They were restored verbatim from `bhrumom/fabushi@5d75920308fa17f5000308df1a0153309eb8e9ab` so this repository can typecheck and bundle without reaching into another checkout.

Restored files:

- `chatgpt-vps-control/lib/app-agent-surface-client.js`
- `contracts/automation/cross-platform-journeys.json`
- `frontend/apps/web/src/app/host/agent-workflow-panel.tsx`
- `frontend/apps/web/src/app/host/bot-mark.tsx`
- `frontend/apps/web/src/app/host/extension-studio.tsx`
- `frontend/apps/web/src/app/host/fabushi-avatar-runtime.tsx`
- `frontend/apps/web/src/app/host/fabushi-bot-mark-engine.tsx`
- `frontend/apps/web/src/app/host/group-chat-panel.tsx`
- `frontend/apps/web/src/app/host/host-client.tsx`
- `frontend/apps/web/src/app/host/host.module.css`
- `frontend/apps/web/src/app/host/rich-transcript.tsx`
- `frontend/apps/web/src/lib/app-agent-surface/dom-agent-surface.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/agent-notifications.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/agent-utils.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/capability-provider.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/collaboration.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/error-trays.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/group-mentions.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/interactions.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/memory-store.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/native-desktop.ts`
- `frontend/apps/web/src/lib/fabushi-runtime/widget-interactions.ts`
- `frontend/apps/web/src/lib/mahayana-host/contracts.ts`
- `frontend/apps/web/src/lib/mahayana-host/coordinator.ts`
- `frontend/apps/web/src/lib/mahayana-host/electron-transport.ts`
- `frontend/apps/web/src/lib/mahayana-host/gateway-events.ts`
- `frontend/apps/web/src/lib/mahayana-host/mock-transport.ts`
- `frontend/apps/web/src/lib/mahayana-host/transport.ts`
- `frontend/apps/web/src/lib/mahayana-host/wasm-transport.ts`
- `frontend/apps/web/src/lib/marketplace-install-contract.ts`
- `frontend/apps/web/src/lib/marketplace-install-state.ts`
- `frontend/apps/web/src/lib/marketplace.ts`
- `frontend/apps/web/src/lib/remote-computer/desktop-peer.ts`
- `frontend/apps/web/src/lib/site-url.ts`
- `frontend/packages/mcp-app-sdk/src/app-surface-webmcp.ts`
- `frontend/packages/mcp-app-sdk/src/app-surface.ts`
- `frontend/packages/mcp-app-sdk/src/bridge.ts`
- `frontend/packages/mcp-app-sdk/src/commands.ts`
- `frontend/packages/mcp-app-sdk/src/index.ts`
- `frontend/packages/mcp-app-sdk/src/types.ts`
- `frontend/packages/mcp-app-sdk/src/webmcp.ts`
- `frontend/packages/shared/src/app-experience.ts`
- `frontend/packages/shared/src/brand.ts`
- `frontend/packages/shared/src/business.ts`
- `frontend/packages/shared/src/chat-experience.ts`
- `frontend/packages/shared/src/copy.ts`
- `frontend/packages/shared/src/index.ts`
- `frontend/packages/shared/src/mahayana-host-features.ts`
