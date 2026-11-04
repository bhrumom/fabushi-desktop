import { createElement, useSyncExternalStore } from "react";
import { RosterReconnectNotice } from "../roster/reconnect-notice";
import { RosterStatus } from "../roster/status";
import type { CursorAuthStatus, DesktopBridge } from "../../contracts/desktop-bridge";
import type { ProductionCoordinatorClient } from "../../../production/coordinator-client";

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2553043 (reconnect host)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5577358 (account status fencing)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5579109 (account request fencing)
// The root host reuses the shipped RosterStatus sand-agents-state projection.

export type CoordinatorTransportState = "connecting" | "connected" | "down";
export type CoordinatorConnectionPhase = "hidden" | "loading" | "connected" | "reconnecting" | "unreachable";

export interface CoordinatorConnectionReadiness {
  readonly hasReachedBox: boolean;
  readonly isPrivacyBlocked: boolean;
  readonly failureCode: string | null;
}

export interface CoordinatorConnectionSnapshot extends CoordinatorConnectionReadiness {
  readonly accountIdentity: string | null;
  readonly accountKind: CursorAuthStatus["kind"];
  readonly transport: CoordinatorTransportState;
  readonly phase: CoordinatorConnectionPhase;
  readonly isRetrying: boolean;
}

export interface CoordinatorConnectionSource {
  readonly ready: Promise<void>;
  getAccountStatus(): Promise<CursorAuthStatus>;
  subscribeAccount(listener: (status: CursorAuthStatus) => void): () => void;
  subscribeTransport(listener: (state: CoordinatorTransportState) => void): () => void;
  retry(): Promise<unknown>;
}

/** Adapter for the existing coordinator/account-session edges; no new bridge API. */
export function createCoordinatorConnectionSource(
  client: Pick<ProductionCoordinatorClient, "ready" | "subscribeTransport">,
  bridge: Pick<DesktopBridge, "cursorAccount">,
  retry: () => Promise<unknown>
): CoordinatorConnectionSource {
  return {
    ready: client.ready,
    getAccountStatus: () => bridge.cursorAccount.getStatus(),
    subscribeAccount: (listener) => bridge.cursorAccount.onStatusChanged(listener),
    subscribeTransport: (listener) => client.subscribeTransport(listener),
    retry
  };
}

export interface CoordinatorConnectionController {
  get(): CoordinatorConnectionSnapshot;
  subscribe(listener: () => void): () => void;
  start(): void;
  ingestAccount(status: CursorAuthStatus): void;
  setReadiness(readiness: CoordinatorConnectionReadiness): void;
  retry(): Promise<boolean>;
  dispose(): void;
}

export interface CoordinatorConnectionHostProps {
  readonly controller: CoordinatorConnectionController;
}

/** Typed fallback for bundlers that resolve the same-stem controller module. */
export function CoordinatorConnectionHost({ controller }: CoordinatorConnectionHostProps) {
  const snapshot = useSyncExternalStore(controller.subscribe, controller.get, controller.get);
  if (snapshot.phase === "hidden" || snapshot.phase === "connected") return null;
  if (snapshot.phase === "loading") return createElement(RosterStatus, { kind: "loading" });
  if (snapshot.phase === "unreachable") return createElement(RosterStatus, { isRetrying: snapshot.isRetrying, kind: "error", onRetry: () => void controller.retry() });
  return createElement(RosterReconnectNotice, { isRetrying: snapshot.isRetrying, onRetry: () => void controller.retry() });
}

const INITIAL_READINESS: CoordinatorConnectionReadiness = {
  hasReachedBox: false,
  isPrivacyBlocked: false,
  failureCode: null
};

const INITIAL_SNAPSHOT: CoordinatorConnectionSnapshot = {
  ...INITIAL_READINESS,
  accountIdentity: null,
  accountKind: "logged-out",
  transport: "connecting",
  phase: "hidden",
  isRetrying: false
};

function accountIdentity(status: CursorAuthStatus): string {
  return status.kind === "logged-in"
    ? `logged-in:${status.authId ?? status.email ?? "account"}`
    : status.kind;
}

function phaseFor(snapshot: Omit<CoordinatorConnectionSnapshot, "phase">): CoordinatorConnectionPhase {
  if (snapshot.accountKind !== "logged-in" || snapshot.isPrivacyBlocked) return "hidden";
  if (snapshot.transport === "connecting") return "loading";
  if (snapshot.transport === "connected" && snapshot.failureCode == null) return "connected";
  return snapshot.hasReachedBox ? "reconnecting" : "unreachable";
}

function failureCode(reason: unknown): string {
  if (typeof reason === "object" && reason !== null && "code" in reason && typeof reason.code === "string" && reason.code.length > 0) return reason.code;
  return "roster-load-failed";
}

function projectSnapshot(snapshot: Omit<CoordinatorConnectionSnapshot, "phase">): CoordinatorConnectionSnapshot {
  return { ...snapshot, phase: phaseFor(snapshot) };
}

interface RetryWaiter {
  readonly generation: number;
  readonly promise: Promise<boolean>;
  resolve(value: boolean): void;
}

export function createCoordinatorConnectionController(
  source: CoordinatorConnectionSource,
  initialReadiness: CoordinatorConnectionReadiness = INITIAL_READINESS
): CoordinatorConnectionController {
  let snapshot = projectSnapshot({ ...INITIAL_SNAPSHOT, ...initialReadiness });
  let generation = 0;
  let transportGeneration = 0;
  let observedTransport = false;
  let accountObservationGeneration = 0;
  let started = false;
  let disposed = false;
  let retryInFlight: Promise<boolean> | null = null;
  let retryWaiter: RetryWaiter | null = null;
  let retryStartQueued = false;
  const listeners = new Set<() => void>();
  let stopAccount: (() => void) | null = null;
  let stopTransport: (() => void) | null = null;

  const notify = () => { for (const listener of [...listeners]) listener(); };
  const isCurrent = (expectedGeneration: number) => !disposed && expectedGeneration === generation;
  const setSnapshot = (next: Omit<CoordinatorConnectionSnapshot, "phase">) => {
    if (disposed) return;
    snapshot = projectSnapshot(next);
    notify();
  };
  const resolveQueuedRetry = (value: boolean) => {
    const waiter = retryWaiter;
    retryWaiter = null;
    waiter?.resolve(value);
  };
  const queueRetry = (): Promise<boolean> => {
    if (retryWaiter != null && retryWaiter.generation === generation) return retryWaiter.promise;
    let resolve!: (value: boolean) => void;
    const promise = new Promise<boolean>((nextResolve) => { resolve = nextResolve; });
    retryWaiter = { generation, promise, resolve };
    return promise;
  };
  const scheduleRetry = () => {
    if (retryStartQueued || disposed || snapshot.transport !== "connected" || retryWaiter == null) return;
    retryStartQueued = true;
    // The Root transport listener runs after this controller's listener. Its
    // generation must advance before refreshRoster captures the current scope.
    queueMicrotask(() => {
      retryStartQueued = false;
      if (!disposed && snapshot.accountKind === "logged-in" && snapshot.transport === "connected"
        && retryWaiter?.generation === generation && retryInFlight == null) void runRetry();
    });
  };
  const runRetry = (): Promise<boolean> => {
    if (disposed || snapshot.accountKind !== "logged-in" || snapshot.transport !== "connected") return Promise.resolve(false);
    if (retryInFlight != null) return retryInFlight;
    const expectedGeneration = generation;
    const expectedTransportGeneration = transportGeneration;
    const isCurrentRetry = (): boolean => isCurrent(expectedGeneration)
      && expectedTransportGeneration === transportGeneration && retryInFlight === request;
    const request: Promise<boolean> = Promise.resolve().then(() => {
      if (isCurrentRetry()) return source.retry();
    }).then(() => {
      if (!isCurrentRetry()) return false;
      resolveQueuedRetry(true);
      setSnapshot({ ...snapshot, failureCode: null, isRetrying: false });
      return true;
    }, (reason: unknown) => {
      if (!isCurrentRetry()) return false;
      resolveQueuedRetry(false);
      setSnapshot({ ...snapshot, failureCode: failureCode(reason), isRetrying: false });
      return false;
    }).finally(() => {
      if (retryInFlight !== request) return;
      retryInFlight = null;
      scheduleRetry();
    });
    retryInFlight = request;
    return request;
  };
  const onTransport = (state: CoordinatorTransportState) => {
    if (disposed) return;
    observedTransport = true;
    transportGeneration += 1;
    const hadRetry = retryInFlight != null || retryWaiter != null;
    // The underlying RPC still settles normally. Only its ownership of the
    // current retry slot ends here; its completion is fenced by both epochs.
    retryInFlight = null;
    if (hadRetry && snapshot.accountKind === "logged-in") queueRetry();
    setSnapshot({ ...snapshot, transport: state, isRetrying: retryWaiter != null });
    scheduleRetry();
  };
  const retireRetry = () => {
    generation += 1;
    retryInFlight = null;
    resolveQueuedRetry(false);
  };
  const onAccount = (status: CursorAuthStatus) => {
    if (disposed) return;
    accountObservationGeneration += 1;
    const nextIdentity = accountIdentity(status);
    const identityChanged = snapshot.accountIdentity != null && snapshot.accountIdentity !== nextIdentity;
    if (status.kind !== "logged-in") {
      retireRetry();
      setSnapshot({ ...snapshot, accountIdentity: nextIdentity, accountKind: status.kind, failureCode: null, isRetrying: false });
      return;
    }
    if (identityChanged) retireRetry();
    setSnapshot({ ...snapshot, accountIdentity: nextIdentity, accountKind: status.kind, failureCode: identityChanged ? null : snapshot.failureCode, isRetrying: identityChanged ? false : snapshot.isRetrying });
  };

  return {
    get: () => snapshot,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    start() {
      if (disposed || started) return;
      started = true;
      const expectedAccountObservation = accountObservationGeneration;
      stopAccount = source.subscribeAccount(onAccount);
      stopTransport = source.subscribeTransport(onTransport);
      const expectedGeneration = generation;
      void source.getAccountStatus().then((status) => {
        if (isCurrent(expectedGeneration) && accountObservationGeneration === expectedAccountObservation) onAccount(status);
      }, () => {});
      source.ready.then(() => {
        if (isCurrent(expectedGeneration) && !observedTransport) onTransport("connected");
      }, () => {
        if (isCurrent(expectedGeneration) && !observedTransport) onTransport("down");
      });
    },
    ingestAccount: onAccount,
    setReadiness(readiness) {
      if (disposed) return;
      setSnapshot({ ...snapshot, ...readiness });
    },
    retry() {
      if (disposed || snapshot.accountKind !== "logged-in") return Promise.resolve(false);
      const promise = queueRetry();
      setSnapshot({ ...snapshot, isRetrying: true });
      scheduleRetry();
      return promise;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      retryInFlight = null;
      stopAccount?.();
      stopTransport?.();
      stopAccount = null;
      stopTransport = null;
      retryWaiter?.resolve(false);
      retryWaiter = null;
      listeners.clear();
    }
  };
}
