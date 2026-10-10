export interface InFlightCommandLease {
  readonly key: string;
  release(): void;
}

export interface InFlightCommandFence {
  acquire(key: string): InFlightCommandLease | null;
  isInFlight(key: string): boolean;
  dispose(): void;
}

export function createInFlightCommandFence(): InFlightCommandFence {
  const active = new Map<string, number>();
  let token = 0;
  let disposed = false;

  return {
    acquire(key) {
      if (disposed || active.has(key)) return null;
      const leaseToken = ++token;
      active.set(key, leaseToken);
      let released = false;
      return {
        key,
        release() {
          if (released) return;
          released = true;
          if (active.get(key) === leaseToken) active.delete(key);
        },
      };
    },
    isInFlight(key) {
      return active.has(key);
    },
    dispose() {
      disposed = true;
      active.clear();
    },
  };
}
