export interface DevBoxRebuildSnapshot {
  readonly generation: number;
  readonly isPending: boolean;
}

export interface DevBoxRebuildSignalStore {
  get(): DevBoxRebuildSnapshot;
  subscribe(listener: () => void): () => void;
  acknowledge(generation: number): void;
  dispose(): void;
}

export interface DevBoxRebuildBridge {
  onDevBoxRebuild(listener: () => void): () => void;
}

const EMPTY_REBUILD: DevBoxRebuildSnapshot = Object.freeze({
  generation: 0,
  isPending: false,
});

export function createDevBoxRebuildSignalStore(
  bridge: DevBoxRebuildBridge,
): DevBoxRebuildSignalStore {
  let snapshot = EMPTY_REBUILD;
  let disposed = false;
  const listeners = new Set<() => void>();

  const publish = (next: DevBoxRebuildSnapshot): void => {
    if (
      disposed
      || (
        next.generation === snapshot.generation
        && next.isPending === snapshot.isPending
      )
    ) {
      return;
    }
    snapshot = next;
    for (const listener of Array.from(listeners)) listener();
  };

  const stopBridge = bridge.onDevBoxRebuild(() => {
    publish({ generation: snapshot.generation + 1, isPending: true });
  });

  return {
    get() {
      return snapshot;
    },
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    acknowledge(generation) {
      if (disposed || !snapshot.isPending || generation !== snapshot.generation) return;
      publish({ generation, isPending: false });
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      stopBridge();
      listeners.clear();
    },
  };
}
