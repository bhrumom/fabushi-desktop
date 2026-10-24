export const ROUTINE_RUN_HISTORY_CLOCK_NAME = "agents-now-tick" as const;
export const ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS = 30_000;

export interface DesktopTimeZoneState {
  detectedTimeZone: string | null;
  overrideTimeZone: string | null;
}

export interface RoutineRunHistoryClock {
  now(): number;
  timeZone?(): string | undefined;
  subscribe?(listener: () => void): () => void;
}

export interface RoutineRunHistoryClockTimer {
  dispose(): void;
}

export interface RoutineRunHistoryClockScheduler {
  schedule(input: {
    readonly name: typeof ROUTINE_RUN_HISTORY_CLOCK_NAME;
    readonly intervalMs: typeof ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS;
    readonly callback: () => void;
  }): RoutineRunHistoryClockTimer;
}

export interface RoutineRunHistoryClockOwner extends RoutineRunHistoryClock {
  ingestTimeZone(state: DesktopTimeZoneState): void;
  dispose(): void;
}

export const browserRoutineRunHistoryScheduler: RoutineRunHistoryClockScheduler = {
  schedule({ intervalMs, callback }) {
    if (typeof window === "undefined") return { dispose() {} };
    const handle = window.setInterval(callback, intervalMs);
    return { dispose: () => window.clearInterval(handle) };
  },
};

export function detectRoutineRunHistoryTimeZone(): DesktopTimeZoneState {
  try {
    return {
      detectedTimeZone:
        Intl.DateTimeFormat().resolvedOptions().timeZone || null,
      overrideTimeZone: null,
    };
  } catch {
    return { detectedTimeZone: null, overrideTimeZone: null };
  }
}

function effectiveTimeZone(state: DesktopTimeZoneState): string {
  return state.overrideTimeZone ?? state.detectedTimeZone ?? "UTC";
}

export function createRoutineRunHistoryClockOwner(options: {
  readonly initialTimeZone: DesktopTimeZoneState;
  readonly now?: () => number;
  readonly scheduler: RoutineRunHistoryClockScheduler;
}): RoutineRunHistoryClockOwner {
  const currentTime = options.now ?? Date.now;
  let timeZone = effectiveTimeZone(options.initialTimeZone);
  let disposed = false;
  let timer: RoutineRunHistoryClockTimer | null = null;
  const listeners = new Set<() => void>();

  const notify = (): void => {
    if (disposed) return;
    for (const listener of Array.from(listeners)) listener();
  };

  const stop = (): void => {
    timer?.dispose();
    timer = null;
  };

  const ensureTimer = (): void => {
    if (disposed || timer != null || listeners.size === 0) return;
    timer = options.scheduler.schedule({
      name: ROUTINE_RUN_HISTORY_CLOCK_NAME,
      intervalMs: ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS,
      callback: notify,
    });
  };

  return {
    now: currentTime,
    timeZone: () => timeZone,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      ensureTimer();
      return () => {
        listeners.delete(listener);
        if (listeners.size === 0) stop();
      };
    },
    ingestTimeZone(state) {
      if (disposed) return;
      const next = effectiveTimeZone(state);
      if (next === timeZone) return;
      timeZone = next;
      notify();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      stop();
      listeners.clear();
    },
  };
}
