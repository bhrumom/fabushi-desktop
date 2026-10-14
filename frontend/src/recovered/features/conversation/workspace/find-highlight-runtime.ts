export type FindHighlightFrameScheduler = (callback: () => void) => () => void;

export interface FindHighlightRefreshRuntime {
  schedule(refresh: () => void): void;
  invalidate(): void;
  dispose(): void;
}

export function scheduleFindHighlightFrame(callback: () => void): () => void {
  if (typeof requestAnimationFrame === "function") {
    const frame = requestAnimationFrame(callback);
    return () => {
      if (typeof cancelAnimationFrame === "function") cancelAnimationFrame(frame);
    };
  }
  let active = true;
  queueMicrotask(() => {
    if (active) callback();
  });
  return () => {
    active = false;
  };
}

export function createFindHighlightRefreshRuntime(
  scheduleFrame: FindHighlightFrameScheduler = scheduleFindHighlightFrame,
): FindHighlightRefreshRuntime {
  let generation = 0;
  let disposed = false;
  let cancelPending: (() => void) | null = null;

  const invalidate = (): void => {
    generation += 1;
    cancelPending?.();
    cancelPending = null;
  };

  return {
    schedule(refresh) {
      if (disposed) return;
      invalidate();
      const scheduledGeneration = generation;
      let settled = false;
      let cancelFrame: (() => void) | null = null;
      const cancelScheduled = () => {
        if (settled) return;
        settled = true;
        cancelFrame?.();
      };
      cancelPending = cancelScheduled;
      cancelFrame = scheduleFrame(() => {
        if (settled) return;
        settled = true;
        if (cancelPending === cancelScheduled) cancelPending = null;
        if (disposed || generation !== scheduledGeneration) return;
        refresh();
      });
      if (settled) cancelFrame = null;
    },
    invalidate() {
      if (disposed) return;
      invalidate();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      invalidate();
    },
  };
}
