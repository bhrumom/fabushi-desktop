import { installFabushiDomAppSurface } from '../../frontend/apps/web/src/lib/app-agent-surface/dom-agent-surface';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { AppSurfaceOperation } from '@fabushi/mcp-app-sdk';

const ALLOWED_OPERATIONS = new Set<AppSurfaceOperation>([
  'status', 'snapshot', 'find', 'action', 'wait', 'assert',
]);
const REQUEST_ID = /^app-agent-[a-f0-9]{32}$/;
const INSTALL_KEY = '__fabushiDesktopAppAgentDisposeV1' as const;
const MAX_STABLE_ACTION_LEASES = 32;
const MAX_STABLE_REBASE_ATTEMPTS = 3;

type AppAgentRequest = {
  version?: unknown;
  requestId?: unknown;
  operation?: unknown;
  input?: unknown;
  deadlineAt?: unknown;
};

type StableActionLease = {
  route: string;
  screen: string;
  targets: Map<string, string>;
};

type DesktopGlobal = Window & { __fabushiDesktopAppAgentDisposeV1?: () => void };

function safeError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return message
    .replace(/Bearer\s+[A-Za-z0-9._~-]+/giu, 'Bearer <redacted>')
    .replace(/(token|password|secret|credential)=([^&\s]+)/giu, '$1=<redacted>')
    .slice(0, 1000);
}

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function integer(value: unknown): number | null {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : null;
}

function stableTargetFingerprint(value: unknown): string {
  const target = record(value);
  return JSON.stringify({
    agentId: String(target.agentId ?? ''),
    role: String(target.role ?? ''),
    name: String(target.name ?? ''),
    description: String(target.description ?? ''),
    visible: target.visible === true,
    enabled: target.enabled === true,
    checked: target.checked ?? null,
    selected: target.selected ?? null,
    expanded: target.expanded ?? null,
    sensitive: target.sensitive === true,
    valuePresent: target.valuePresent ?? null,
    valueLength: target.valueLength ?? null,
    placeholder: String(target.placeholder ?? ''),
    tag: String(target.tag ?? ''),
  });
}

function leaseFromSnapshot(value: unknown): { generation: number; lease: StableActionLease } | null {
  const snapshot = record(value);
  const generation = integer(snapshot.generation);
  if (generation == null || !Array.isArray(snapshot.elements)) return null;
  const targets = new Map<string, string>();
  for (const candidate of snapshot.elements) {
    const target = record(candidate);
    const agentId = String(target.agentId ?? '').trim();
    if (!agentId || target.stable !== true || targets.has(agentId)) continue;
    targets.set(agentId, stableTargetFingerprint(target));
  }
  return {
    generation,
    lease: {
      route: String(snapshot.route ?? ''),
      screen: String(snapshot.screen ?? ''),
      targets,
    },
  };
}

function rememberStableActionLease(leases: Map<number, StableActionLease>, snapshot: unknown): void {
  const captured = leaseFromSnapshot(snapshot);
  if (!captured) return;
  leases.set(captured.generation, captured.lease);
  while (leases.size > MAX_STABLE_ACTION_LEASES) {
    const oldest = leases.keys().next().value as number | undefined;
    if (oldest == null) break;
    leases.delete(oldest);
  }
}

function isStaleGeneration(error: unknown): boolean {
  return /stale_app_surface_generation/u.test(error instanceof Error ? error.message : String(error));
}

async function callSurfaceWithStableTargetRebase(
  surface: ReturnType<typeof installFabushiDomAppSurface>['surface'],
  operation: AppSurfaceOperation,
  input: Record<string, unknown>,
  signal: AbortSignal,
  leases: Map<number, StableActionLease>,
): Promise<unknown> {
  try {
    const result = await surface.call(operation, input, { signal });
    if (operation === 'snapshot') rememberStableActionLease(leases, result);
    return result;
  } catch (initialError) {
    if (operation !== 'action' || !isStaleGeneration(initialError)) throw initialError;

    const requestedGeneration = integer(input.generation);
    const agentId = String(input.agentId ?? '').trim();
    const ref = String(input.ref ?? '').trim();
    const requestedLease = requestedGeneration == null ? null : leases.get(requestedGeneration);
    const requestedFingerprint = agentId && requestedLease ? requestedLease.targets.get(agentId) : undefined;
    if (
      requestedGeneration == null
      || !agentId
      || (ref && ref !== `agent:${agentId}`)
      || !requestedLease
      || !requestedFingerprint
    ) {
      throw initialError;
    }

    let latestError: unknown = initialError;
    for (let attempt = 0; attempt < MAX_STABLE_REBASE_ATTEMPTS; attempt += 1) {
      const currentSnapshot = await surface.call('snapshot', { maxElements: 500, includeText: true }, { signal });
      rememberStableActionLease(leases, currentSnapshot);
      const current = record(currentSnapshot);
      const currentGeneration = integer(current.generation);
      if (
        currentGeneration == null
        || String(current.route ?? '') !== requestedLease.route
        || String(current.screen ?? '') !== requestedLease.screen
        || !Array.isArray(current.elements)
      ) {
        throw initialError;
      }
      const currentMatches = current.elements
        .map(record)
        .filter((candidate) => candidate.stable === true && String(candidate.agentId ?? '') === agentId);
      if (
        currentMatches.length !== 1
        || stableTargetFingerprint(currentMatches[0]) !== requestedFingerprint
      ) {
        throw initialError;
      }

      try {
        return await surface.call('action', { ...input, generation: currentGeneration }, { signal });
      } catch (retryError) {
        latestError = retryError;
        if (!isStaleGeneration(retryError)) throw retryError;
      }
    }
    throw latestError;
  }
}

export function installDesktopAppAgentSurface(): () => void {
  const globalObject = window as DesktopGlobal;
  if (globalObject[INSTALL_KEY]) return globalObject[INSTALL_KEY];
  document.body.dataset.agentScreen = 'messenger';
  const installed = installFabushiDomAppSurface({ appId: 'fabushi.desktop', platform: 'electron' });
  const stableActionLeases = new Map<number, StableActionLease>();
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
      void callSurfaceWithStableTargetRebase(installed.surface, operation, input, controller.signal, stableActionLeases)
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
    stableActionLeases.clear();
    installed.dispose();
    delete document.body.dataset.agentScreen;
    if (globalObject[INSTALL_KEY] === dispose) delete globalObject[INSTALL_KEY];
  };
  Object.defineProperty(globalObject, INSTALL_KEY, { configurable: true, value: dispose });
  return dispose;
}
