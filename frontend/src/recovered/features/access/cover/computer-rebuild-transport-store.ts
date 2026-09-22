import {
  reduceComputerRebuildState,
  type ComputerRebuildState,
} from "./computer-rebuild-model.ts";

export type ComputerRebuildTransportState = "connected" | "down";

export interface ComputerRebuildTransportSource {
  readonly ready: Promise<void>;
  subscribeTransport(listener: (state: unknown) => void): () => void;
}

export interface ComputerRebuildTransportStore {
  get(): ComputerRebuildState;
  getTransportState(): ComputerRebuildTransportState;
  isHydrating(): boolean;
  subscribe(listener: () => void): () => void;
  connect(): Promise<void>;
  reset(): void;
  dispose(): void;
}

function transportState(
  value: unknown,
): ComputerRebuildTransportState | null {
  return value === "connected" || value === "down" ? value : null;
}

export function createComputerRebuildTransportStore(input: {
  source: ComputerRebuildTransportSource;
  initialState: ComputerRebuildState;
  now: () => number;
}): ComputerRebuildTransportStore {
  let state = input.initialState;
  let transport: ComputerRebuildTransportState =
    state.isConnected ? "connected" : "down";
  let connected = false;
  let hydrating = false;
  let disposed = false;
  let generation = 0;
  let pending: Promise<void> | null = null;
  let stop: (() => void) | null = null;
  const listeners = new Set<() => void>();

  const notify = (): void => {
    for (const listener of Array.from(listeners)) listener();
  };

  const ingest = (next: ComputerRebuildTransportState): void => {
    if (disposed) return;
    generation += 1;
    hydrating = false;
    transport = next;
    state = reduceComputerRebuildState(state, {
      type: "connection",
      isConnected: next === "connected",
      at: input.now(),
    });
    notify();
  };

  const hydrate = (): Promise<void> => {
    if (disposed || !connected) return Promise.resolve();
    if (pending != null) return pending;

    const attempt = ++generation;
    hydrating = true;
    notify();
    const request = input.source.ready.then(
      () => {
        if (!disposed && connected && attempt === generation) {
          ingest("connected");
        }
      },
      () => {
        if (!disposed && connected && attempt === generation) {
          ingest("down");
        }
      },
    ).finally(() => {
      if (pending === request) pending = null;
      if (!disposed && attempt === generation) {
        hydrating = false;
        notify();
      }
    });
    pending = request;
    return request;
  };

  return {
    get: () => state,
    getTransportState: () => transport,
    isHydrating: () => hydrating,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    connect() {
      if (disposed) return Promise.resolve();
      if (!connected) {
        connected = true;
        stop = input.source.subscribeTransport((value) => {
          const next = transportState(value);
          if (next != null) ingest(next);
        });
      }
      return hydrate();
    },
    reset() {
      if (disposed) return;
      generation += 1;
      connected = false;
      hydrating = false;
      pending = null;
      stop?.();
      stop = null;
      state = input.initialState;
      transport = state.isConnected ? "connected" : "down";
      notify();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      connected = false;
      hydrating = false;
      pending = null;
      stop?.();
      stop = null;
      listeners.clear();
    },
  };
}
