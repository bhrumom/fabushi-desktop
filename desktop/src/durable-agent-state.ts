import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import {
  MAHAYANA_ACCOUNT_SESSION_RESET_EVENT,
  MAHAYANA_COMMAND_EVENT_NAME,
  MAHAYANA_RUNTIME_EVENT_NAME,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

export const AGENT_WORKBENCH_STORAGE_KEY = 'fabushi.desktop.mahayana-agent-workbench.v1';
export const CONVERSATION_JOURNAL_STORAGE_KEY = 'fabushi.desktop.mahayana-conversation-journal.v1';
export const SELFHOSTED_INVOCATION_CLAIMS_KEY = 'fabushi.desktop.selfhosted-mahayana-invocations.v1';

export const DURABLE_AGENT_STATE_KEYS = [
  AGENT_WORKBENCH_STORAGE_KEY,
  CONVERSATION_JOURNAL_STORAGE_KEY,
  SELFHOSTED_INVOCATION_CLAIMS_KEY,
] as const;

type DurableAgentStateKey = typeof DURABLE_AGENT_STATE_KEYS[number];

function parseLocalValue(value: string): unknown {
  try {
    return JSON.parse(value) as unknown;
  } catch {
    return null;
  }
}

async function readNativeValue(key: DurableAgentStateKey): Promise<unknown> {
  try {
    return await invokeNativeDesktop<unknown>('readClientPersistence', { key });
  } catch {
    return null;
  }
}

/**
 * Restore renderer projections from the native client-persistence store before
 * React constructs transport/workbench state. localStorage remains a first-frame
 * cache only; when it is absent, the native store supplies the restart-safe copy.
 */
export async function restoreDurableAgentState(): Promise<void> {
  if (typeof window === 'undefined') return;
  await Promise.all(DURABLE_AGENT_STATE_KEYS.map(async (key) => {
    try {
      if (window.localStorage.getItem(key) !== null) return;
      const nativeValue = await readNativeValue(key);
      if (nativeValue === null || nativeValue === undefined) return;
      window.localStorage.setItem(key, JSON.stringify(nativeValue));
    } catch {
      // Native persistence is unavailable in browser-only development. The
      // renderer cache still lets local development continue without lying
      // about durable production state.
    }
  }));
}

/**
 * Mirror the three account-scoped Mahayana projections to native persistence.
 * The existing renderer owners continue to update their in-memory/local cache;
 * this bridge observes those projections and makes Rust/native persistence the
 * restart boundary. Values are deduplicated so the idle poll performs no disk
 * writes when state has not changed.
 */
export function installDurableAgentState(): () => void {
  if (typeof window === 'undefined') return () => {};

  const lastSerialized = new Map<DurableAgentStateKey, string | null>();
  const inFlight = new Set<DurableAgentStateKey>();
  let disposed = false;
  let scheduledTimer: number | null = null;

  const flushKey = async (key: DurableAgentStateKey) => {
    if (disposed || inFlight.has(key)) return;
    const serialized = window.localStorage.getItem(key);
    if (lastSerialized.has(key) && lastSerialized.get(key) === serialized) return;
    inFlight.add(key);
    try {
      if (serialized === null) {
        await invokeNativeDesktop<boolean>('removeClientPersistence', { key });
        lastSerialized.set(key, null);
        return;
      }
      const value = parseLocalValue(serialized);
      if (value === null) return;
      await invokeNativeDesktop<boolean>('writeClientPersistence', { key, value });
      lastSerialized.set(key, serialized);
    } catch {
      // Keep lastSerialized unchanged so a later runtime event or idle poll
      // retries after a temporarily unavailable native edge.
    } finally {
      inFlight.delete(key);
    }
  };

  const flushAll = () => {
    DURABLE_AGENT_STATE_KEYS.forEach((key) => void flushKey(key));
  };

  const scheduleFlush = () => {
    if (scheduledTimer !== null) window.clearTimeout(scheduledTimer);
    // React effects and the self-hosted invocation bridge commit their local
    // projections after the runtime event callback. A short trailing debounce
    // captures the completed projection rather than the previous frame.
    scheduledTimer = window.setTimeout(() => {
      scheduledTimer = null;
      flushAll();
    }, 120);
  };

  const clearAccountState = () => {
    if (scheduledTimer !== null) {
      window.clearTimeout(scheduledTimer);
      scheduledTimer = null;
    }
    DURABLE_AGENT_STATE_KEYS.forEach((key) => {
      try {
        window.localStorage.removeItem(key);
      } catch {
        // Continue with native deletion.
      }
      lastSerialized.set(key, null);
      void invokeNativeDesktop<boolean>('removeClientPersistence', { key }).catch(() => {});
    });
  };

  window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, scheduleFlush);
  window.addEventListener(MAHAYANA_RUNTIME_EVENT_NAME, scheduleFlush);
  window.addEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, clearAccountState);
  const interval = window.setInterval(flushAll, 1_000);
  flushAll();

  return () => {
    disposed = true;
    if (scheduledTimer !== null) window.clearTimeout(scheduledTimer);
    window.clearInterval(interval);
    window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, scheduleFlush);
    window.removeEventListener(MAHAYANA_RUNTIME_EVENT_NAME, scheduleFlush);
    window.removeEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, clearAccountState);
  };
}
