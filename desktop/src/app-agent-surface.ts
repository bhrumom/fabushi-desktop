import { installFabushiDomAppSurface } from '../../frontend/apps/web/src/lib/app-agent-surface/dom-agent-surface';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { AppSurfaceOperation } from '@fabushi/mcp-app-sdk';

const ALLOWED_OPERATIONS = new Set<AppSurfaceOperation>([
  'status', 'snapshot', 'find', 'action', 'wait', 'assert',
]);
const REQUEST_ID = /^app-agent-[a-f0-9]{32}$/;
const INSTALL_KEY = '__fabushiDesktopAppAgentDisposeV1' as const;

type AppAgentRequest = {
  version?: unknown;
  requestId?: unknown;
  operation?: unknown;
  input?: unknown;
  deadlineAt?: unknown;
};

type DesktopGlobal = Window & { __fabushiDesktopAppAgentDisposeV1?: () => void };

function safeError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return message
    .replace(/Bearer\s+[A-Za-z0-9._~-]+/giu, 'Bearer <redacted>')
    .replace(/(token|password|secret|credential)=([^&\s]+)/giu, '$1=<redacted>')
    .slice(0, 1000);
}

export function installDesktopAppAgentSurface(): () => void {
  const globalObject = window as DesktopGlobal;
  if (globalObject[INSTALL_KEY]) return globalObject[INSTALL_KEY];
  document.body.dataset.agentScreen = 'messenger';
  const installed = installFabushiDomAppSurface({ appId: 'fabushi.desktop', platform: 'electron' });
  const unsubscribe = subscribeNativeDesktopEvents({
    'app-agent-surface-request': (payload) => {
      const request = (payload && typeof payload === 'object' && !Array.isArray(payload)
        ? payload
        : {}) as AppAgentRequest;
      const requestId = String(request.requestId ?? '');
      const operation = String(request.operation ?? '') as AppSurfaceOperation;
      const deadlineAt = Number(request.deadlineAt ?? 0);
      if (!REQUEST_ID.test(requestId) || !ALLOWED_OPERATIONS.has(operation)) return;
      const input = request.input && typeof request.input === 'object' && !Array.isArray(request.input)
        ? request.input as Record<string, unknown>
        : {};
      const controller = new AbortController();
      const remaining = Number.isFinite(deadlineAt) ? Math.max(1, Math.min(35_000, deadlineAt - Date.now())) : 35_000;
      const timer = window.setTimeout(() => controller.abort(new Error('app_surface_renderer_timeout')), remaining);
      void installed.surface.call(operation, input, { signal: controller.signal })
        .then((result) => invokeNativeDesktop('respondAppAgentSurfaceRequest', { requestId, ok: true, result }))
        .catch((error: unknown) => invokeNativeDesktop('respondAppAgentSurfaceRequest', {
          requestId,
          ok: false,
          error: safeError(error),
        }))
        .catch(() => undefined)
        .finally(() => window.clearTimeout(timer));
    },
  });
  const dispose = () => {
    unsubscribe();
    installed.dispose();
    delete document.body.dataset.agentScreen;
    if (globalObject[INSTALL_KEY] === dispose) delete globalObject[INSTALL_KEY];
  };
  Object.defineProperty(globalObject, INSTALL_KEY, { configurable: true, value: dispose });
  return dispose;
}
