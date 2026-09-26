export const ASYNC_TASKS_CLOCK_INTERVAL_MS = 30_000;

export interface AsyncTasksClockInput {
  now(): number;
  subscribe?(listener: () => void): () => void;
}

export interface AsyncTasksClock {
  now(): number;
  subscribe(listener: () => void): () => void;
}

export interface AsyncTasksClockScheduler {
  start(listener: () => void, intervalMs: number): { dispose(): void };
}

const intervalScheduler: AsyncTasksClockScheduler = {
  start(listener, intervalMs) {
    const handle = setInterval(listener, intervalMs);
    return {
      dispose() {
        clearInterval(handle);
      },
    };
  },
};

export function createStableAsyncTasksClock(
  input: AsyncTasksClockInput,
  scheduler: AsyncTasksClockScheduler = intervalScheduler,
): AsyncTasksClock {
  let snapshot = input.now();
  const listeners = new Set<() => void>();
  let releaseSource: (() => void) | null = null;
  let releaseTimer: (() => void) | null = null;

  const refresh = (): void => {
    snapshot = input.now();
    for (const listener of Array.from(listeners)) listener();
  };

  const start = (): void => {
    if (input.subscribe != null) {
      releaseSource = input.subscribe(refresh);
      return;
    }
    const timer = scheduler.start(refresh, ASYNC_TASKS_CLOCK_INTERVAL_MS);
    releaseTimer = () => timer.dispose();
  };

  const stop = (): void => {
    releaseSource?.();
    releaseSource = null;
    releaseTimer?.();
    releaseTimer = null;
  };

  return {
    now() {
      return snapshot;
    },
    subscribe(listener) {
      const first = listeners.size === 0;
      listeners.add(listener);
      if (first) start();

      let released = false;
      return () => {
        if (released) return;
        released = true;
        listeners.delete(listener);
        if (listeners.size === 0) stop();
      };
    },
  };
}

export const DEFAULT_ASYNC_TASKS_CLOCK = createStableAsyncTasksClock({
  now: () => Date.now(),
});
