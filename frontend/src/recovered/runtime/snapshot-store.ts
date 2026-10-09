export interface SnapshotStore<Value> {
  get(): Value;
  subscribe(listener: () => void): () => void;
  set(value: Value): void;
  update(updater: (current: Value) => Value): void;
}

export function createSnapshotStore<Value>(initialValue: Value): SnapshotStore<Value> {
  let value = initialValue;
  const listeners = new Set<() => void>();

  const commit = (next: Value): void => {
    if (Object.is(value, next)) return;
    value = next;
    for (const listener of Array.from(listeners)) listener();
  };

  return {
    get() {
      return value;
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    set(next) {
      commit(next);
    },
    update(updater) {
      commit(updater(value));
    },
  };
}
