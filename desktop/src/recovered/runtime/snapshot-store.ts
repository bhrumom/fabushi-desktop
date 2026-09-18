export interface SnapshotStore<Value> {
  get(): Value;
  subscribe(listener: () => void): () => void;
  set(value: Value): void;
  update(updater: (current: Value) => Value): void;
}

export function createSnapshotStore<Value>(initial: Value): SnapshotStore<Value> {
  let current = initial;
  const listeners = new Set<() => void>();
  const publish = (next: Value): void => {
    if (Object.is(next, current)) return;
    current = next;
    for (const listener of [...listeners]) listener();
  };
  return {
    get: () => current,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set: publish,
    update(updater) {
      publish(updater(current));
    }
  };
}
