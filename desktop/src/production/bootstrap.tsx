import { StrictMode, type ReactElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import type { GrokAgentBridge } from '../grok-types';

export const PACKAGED_INVARIANT_MESSAGE =
  'Invariant violation (message stripped in packaged builds; the stack identifies the site)';

export interface ProductionRendererRuntime {
  bridge: GrokAgentBridge;
}

function invariant(condition: unknown): asserts condition {
  if (!condition) throw new Error(PACKAGED_INVARIANT_MESSAGE);
}

export function acquireProductionRendererRuntime(windowValue: unknown): ProductionRendererRuntime {
  invariant(typeof windowValue === 'object' && windowValue != null);
  const candidate = windowValue as { grokAgent?: unknown };
  const bridge = candidate.grokAgent as Partial<GrokAgentBridge> | undefined;
  invariant(typeof bridge === 'object' && bridge != null);
  invariant(typeof bridge.listAgents === 'function');
  invariant(typeof bridge.getThread === 'function');
  invariant(typeof bridge.sendMessage === 'function');
  invariant(typeof bridge.subscribe === 'function');
  return { bridge: bridge as GrokAgentBridge };
}

export function requireProductionRendererMount(mount: HTMLElement | null): HTMLElement {
  invariant(mount != null);
  return mount;
}

export function mountProductionRenderer(mount: HTMLElement, renderer: ReactElement): Root {
  const root = createRoot(mount);
  root.render(<StrictMode>{renderer}</StrictMode>);
  return root;
}
