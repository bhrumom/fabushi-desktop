export type ConversationChildPageDirection = "around" | "before" | "after";

export interface ConversationChildPageRequest {
  scopeKey: string;
  direction: ConversationChildPageDirection;
  anchor: string | null;
}

export interface ConversationChildPage {
  readonly messageIds: readonly string[];
  readonly skippedBefore?: number;
  readonly skippedAfter?: number;
  readonly fullCount?: number;
}

export type ConversationChildPageFetcher = (
  request: ConversationChildPageRequest,
  signal: AbortSignal
) => Promise<ConversationChildPage>;

export interface ConversationChildPageRequestSnapshot {
  readonly generation: number;
  readonly scopeKey: string | null;
  readonly inFlight: Readonly<Record<ConversationChildPageDirection, boolean>>;
  readonly lastFailure: unknown | null;
}

export interface ConversationChildPageRequestController {
  getSnapshot(): ConversationChildPageRequestSnapshot;
  subscribe(listener: () => void): () => void;
  setScope(scopeKey: string | null): void;
  loadAround(anchor: string | null): Promise<void>;
  loadBefore(): Promise<void>;
  loadAfter(): Promise<void>;
  reset(): void;
  dispose(): void;
}

export interface ConversationChildPageRequestControllerOptions {
  fetchPage: ConversationChildPageFetcher;
  getBoundary(direction: "before" | "after"): string | null;
  commitPage(direction: ConversationChildPageDirection, page: ConversationChildPage): void;
}

interface InFlightRequest {
  readonly serial: number;
  readonly scopeGeneration: number;
  readonly scopeKey: string;
  readonly direction: ConversationChildPageDirection;
  readonly anchor: string | null;
  readonly controller: AbortController;
  readonly promise: Promise<void>;
}

const DIRECTIONS: readonly ConversationChildPageDirection[] = ["around", "before", "after"];

function isAbortFailure(error: unknown): boolean {
  return error instanceof DOMException
    ? error.name === "AbortError"
    : typeof error === "object" && error != null && "name" in error
      && (error as { name?: unknown }).name === "AbortError";
}

/**
 * Source-neutral lifecycle owner for asynchronous child-history page requests.
 *
 * The canonical Conversation state remains in the messaging domain. This
 * controller owns only transport/request timing: direction-scoped in-flight
 * fencing, supersede cancellation, same-around retry coalescing and stale
 * boundary retries. Callers commit only accepted pages into the canonical
 * ConversationChildPaginationState.
 */
export function createConversationChildPageRequestController(
  options: ConversationChildPageRequestControllerOptions
): ConversationChildPageRequestController {
  const listeners = new Set<() => void>();
  const inFlight: Partial<Record<ConversationChildPageDirection, InFlightRequest>> = {};
  let generation = 0;
  let requestSerial = 0;
  let scopeKey: string | null = null;
  let lastFailure: unknown | null = null;
  let queuedAroundRetry: string | null | undefined;
  let disposed = false;

  const buildSnapshot = (): ConversationChildPageRequestSnapshot => ({
    generation,
    scopeKey,
    inFlight: {
      around: inFlight.around != null,
      before: inFlight.before != null,
      after: inFlight.after != null
    },
    lastFailure
  });
  let cachedSnapshot = buildSnapshot();

  const emit = () => {
    if (disposed) return;
    cachedSnapshot = buildSnapshot();
    for (const listener of [...listeners]) listener();
  };

  const cancelDirection = (direction: ConversationChildPageDirection) => {
    const request = inFlight[direction];
    if (request == null) return;
    delete inFlight[direction];
    request.controller.abort();
  };

  const cancelAll = () => {
    for (const direction of DIRECTIONS) cancelDirection(direction);
    queuedAroundRetry = undefined;
  };

  const current = (request: Pick<InFlightRequest, "serial" | "scopeGeneration" | "scopeKey" | "direction">) => (
    !disposed
    && scopeKey === request.scopeKey
    && generation === request.scopeGeneration
    && inFlight[request.direction]?.serial === request.serial
  );

  const start = (
    direction: ConversationChildPageDirection,
    anchor: string | null
  ): Promise<void> => {
    if (disposed || scopeKey == null) return Promise.resolve();

    const requestScope = scopeKey;
    const requestGeneration = generation;
    const serial = ++requestSerial;
    const controller = new AbortController();
    let retryAfterSettlement = false;

    const promise = (async () => {
      try {
        const page = await options.fetchPage({
          scopeKey: requestScope,
          direction,
          anchor
        }, controller.signal);
        const request = { serial, scopeGeneration: requestGeneration, scopeKey: requestScope, direction };
        if (!current(request)) return;

        if (direction === "before" || direction === "after") {
          const boundaryNow = options.getBoundary(direction);
          if (boundaryNow !== anchor) {
            retryAfterSettlement = true;
          } else {
            options.commitPage(direction, page);
            lastFailure = null;
          }
        } else {
          options.commitPage(direction, page);
          lastFailure = null;
        }
      } catch (error) {
        const request = { serial, scopeGeneration: requestGeneration, scopeKey: requestScope, direction };
        if (!current(request) || controller.signal.aborted || isAbortFailure(error)) return;
        lastFailure = error;
      } finally {
        const request = { serial, scopeGeneration: requestGeneration, scopeKey: requestScope, direction };
        if (!current(request)) return;
        delete inFlight[direction];

        if (direction === "around" && queuedAroundRetry !== undefined) {
          const retryAnchor = queuedAroundRetry;
          queuedAroundRetry = undefined;
          emit();
          await start("around", retryAnchor);
          return;
        }

        if (retryAfterSettlement) {
          emit();
          const freshBoundary = options.getBoundary(direction as "before" | "after");
          if (freshBoundary != null) await start(direction, freshBoundary);
          return;
        }

        emit();
      }
    })();

    inFlight[direction] = {
      serial,
      scopeGeneration: requestGeneration,
      scopeKey: requestScope,
      direction,
      anchor,
      controller,
      promise
    };
    emit();
    return promise;
  };

  return {
    getSnapshot: () => cachedSnapshot,

    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },

    setScope(nextScopeKey) {
      if (disposed || scopeKey === nextScopeKey) return;
      generation += 1;
      cancelAll();
      scopeKey = nextScopeKey;
      lastFailure = null;
      emit();
    },

    loadAround(anchor) {
      if (disposed || scopeKey == null) return Promise.resolve();
      const active = inFlight.around;
      if (active != null && active.anchor === anchor) {
        queuedAroundRetry = anchor;
        return active.promise;
      }

      cancelDirection("around");
      cancelDirection("before");
      cancelDirection("after");
      queuedAroundRetry = undefined;
      lastFailure = null;
      emit();
      return start("around", anchor);
    },

    loadBefore() {
      if (disposed || scopeKey == null) return Promise.resolve();
      if (inFlight.around != null) {
        cancelDirection("around");
        queuedAroundRetry = undefined;
      }
      if (inFlight.before != null) return inFlight.before.promise;
      const boundary = options.getBoundary("before");
      if (boundary == null) return Promise.resolve();
      lastFailure = null;
      emit();
      return start("before", boundary);
    },

    loadAfter() {
      if (disposed || scopeKey == null) return Promise.resolve();
      if (inFlight.after != null) return inFlight.after.promise;
      const boundary = options.getBoundary("after");
      if (boundary == null) return Promise.resolve();
      lastFailure = null;
      emit();
      return start("after", boundary);
    },

    reset() {
      if (disposed) return;
      generation += 1;
      cancelAll();
      lastFailure = null;
      emit();
    },

    dispose() {
      if (disposed) return;
      generation += 1;
      cancelAll();
      disposed = true;
      listeners.clear();
    }
  };
}
