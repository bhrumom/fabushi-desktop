import type { CoordinatorRuntime } from "./coordinator-runtime.js";

export interface CoordinatorAuthStatus {
  readonly kind: string;
  readonly authId?: string;
  readonly email?: string;
  readonly [key: string]: unknown;
}

export interface CoordinatorAccountTransition {
  readonly isStartup: boolean;
  readonly previousSlot: string | null | undefined;
}

export interface CoordinatorRefusedAccountResult<Status extends CoordinatorAuthStatus> {
  readonly kind: string;
  readonly status: Status;
  readonly error?: unknown;
}

export interface CoordinatorAccountRuntimeDependencies<Status extends CoordinatorAuthStatus> {
  readonly createRuntime: () => CoordinatorRuntime;
  readonly authorizeAccount: (
    slot: string,
    context: CoordinatorAccountTransition,
  ) => Promise<boolean>;
  readonly revokeRefusedAccount: () => Promise<CoordinatorRefusedAccountResult<Status>>;
  readonly prepareAccountTransition: (transition: {
    readonly previousSlot: string;
    readonly nextSlot: string | null;
  }) => Promise<void>;
  readonly resetAccountState: () => void;
  readonly revokeMainDataPort: (runtime: CoordinatorRuntime) => void;
  readonly deliverStatus: (status: Status) => void;
  readonly onProblem: (problem: string) => void;
  readonly onExitTimeout?: (timeoutMs: number) => void;
  readonly exitTimeoutMs?: number;
  readonly delay?: (ms: number, signal: AbortSignal) => Promise<void>;
}

export interface CoordinatorAccountBinding {
  /** Native composition identity only; no runtime methods or account claim. */
  readonly runtimeIdentity: object;
  assertCurrent(): void;
}

export interface CoordinatorAccountRuntime<Status extends CoordinatorAuthStatus> {
  start(status: Status): Promise<void>;
  observe(status: Status): void;
  whenIdle(): Promise<Status>;
  captureCurrentBinding(): CoordinatorAccountBinding | null;
  requestRendererPort(sink: (port: unknown) => void): void;
  restart(): Promise<void>;
  dispose(): Promise<void>;
}

interface RuntimeLifetime {
  retired: boolean;
}

interface RuntimeLaunchAttempt {
  readonly slot: string;
  readonly lifetime: RuntimeLifetime;
}

interface RuntimeRetirement {
  readonly session: CoordinatorRuntime;
  confirmed: boolean;
}

type StopOutcome =
  | { readonly kind: "exited" }
  | { readonly kind: "unconfirmed"; readonly retirement: RuntimeRetirement };

type RuntimeState =
  | { readonly kind: "unstarted" }
  | { readonly kind: "inactive" }
  | {
      readonly kind: "active";
      readonly slot: string;
      readonly session: CoordinatorRuntime;
      readonly lifetime: RuntimeLifetime;
      readonly appliedVersion: number;
      readonly rendererRequestRevoked: boolean;
    }
  | {
      readonly kind: "blocked";
      readonly slot: string | null;
      readonly previousSlot: string | null | undefined;
      readonly retirement: RuntimeRetirement;
    }
  | { readonly kind: "disposed" };

type StartOutcome<Status extends CoordinatorAuthStatus> =
  | { readonly kind: "accepted" }
  | { readonly kind: "disposed" }
  | { readonly kind: "launch-failed" }
  | { readonly kind: "superseded" }
  | { readonly kind: "blocked" }
  | { readonly kind: "refused"; readonly status: Status };

const DEFAULT_EXIT_TIMEOUT_MS = 10_000;

export class SandCoordinatorExitTimeoutError extends Error {}

function abortableDelay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted) {
      resolve();
      return;
    }
    const timeout = setTimeout(resolve, ms);
    signal.addEventListener(
      "abort",
      () => {
        clearTimeout(timeout);
        resolve();
      },
      { once: true },
    );
  });
}

function cursorAccountSlot(status: CoordinatorAuthStatus): string | null {
  if (status.kind !== "logged-in") return null;
  const slot = status.authId ?? status.email;
  return slot == null || slot.length === 0 ? null : slot;
}

/**
 * Owns the one coordinator process allowed for the currently settled account.
 * Observations are serialized and version-fenced so an older authorization or
 * shutdown cannot launch a coordinator after a newer auth status has arrived.
 */
export function createCoordinatorAccountRuntime<Status extends CoordinatorAuthStatus>(
  dependencies: CoordinatorAccountRuntimeDependencies<Status>,
): CoordinatorAccountRuntime<Status> {
  const exitTimeoutMs = dependencies.exitTimeoutMs ?? DEFAULT_EXIT_TIMEOUT_MS;
  const delay = dependencies.delay ?? abortableDelay;
  const loggedOut = { kind: "logged-out" } as Status;

  let state: RuntimeState = { kind: "unstarted" };
  let requester: ((port: unknown) => void) | null = null;
  let settledStatus = loggedOut;
  let observedVersion = 0;
  let chain: Promise<void> = Promise.resolve();
  let startWork: Promise<void> = Promise.resolve();
  let startRequested = false;
  let pendingLaunch: RuntimeLaunchAttempt | undefined;
  let disposeWork: Promise<void> | undefined;
  let releaseStart: () => void = () => {};
  const startArrived = new Promise<void>((resolve) => {
    releaseStart = resolve;
  });

  const isDisposed = (): boolean => state.kind === "disposed";

  const reportProblem = (detail: string): void => {
    try {
      dependencies.onProblem(detail);
    } catch {
      // Reporting must not abandon an owned runtime or skip its exit fence.
    }
  };

  const accountSlot = (): string | null | undefined => {
    switch (state.kind) {
      case "unstarted":
      case "disposed":
        return undefined;
      case "inactive":
        return null;
      case "active":
      case "blocked":
        return state.slot;
    }
  };

  const revokeActiveRendererPortRequest = (): void => {
    if (state.kind !== "active" || state.rendererRequestRevoked) return;
    const active = state;
    state = { ...active, rendererRequestRevoked: true };
    try {
      active.session.revokeRendererPortRequest();
    } catch (error) {
      reportProblem(`coordinator renderer request revocation failed: ${String(error)}`);
    }
  };

  const detachActive = (disposing = false): CoordinatorRuntime | undefined => {
    const previous = state.kind === "active" ? state : undefined;
    if (previous !== undefined) previous.lifetime.retired = true;
    if (previous !== undefined || disposing) {
      state = disposing ? { kind: "disposed" } : { kind: "inactive" };
    }
    if (previous === undefined) return undefined;
    try {
      dependencies.revokeMainDataPort(previous.session);
    } catch (error) {
      reportProblem(`coordinator main-data revocation failed: ${String(error)}`);
    }
    return previous.session;
  };

  const activateSession = (args: {
    readonly slot: string;
    readonly session: CoordinatorRuntime;
    readonly lifetime: RuntimeLifetime;
    readonly version: number;
  }): void => {
    state = {
      kind: "active",
      slot: args.slot,
      session: args.session,
      lifetime: args.lifetime,
      appliedVersion: args.version,
      rendererRequestRevoked: false,
    };
    if (requester !== null) args.session.requestRendererPort(requester);
  };

  const stop = async (
    previous: CoordinatorRuntime | undefined,
    onEventualExit?: (retirement: RuntimeRetirement) => void,
  ): Promise<StopOutcome> => {
    if (previous === undefined) return { kind: "exited" };
    const retirement: RuntimeRetirement = { session: previous, confirmed: false };
    const deadlineController = new AbortController();
    let exit: Promise<void> | undefined;
    let timedOut = false;
    try {
      exit = previous.dispose();
      await Promise.race([
        exit,
        delay(exitTimeoutMs, deadlineController.signal).then(() => {
          if (deadlineController.signal.aborted) return;
          timedOut = true;
          throw new SandCoordinatorExitTimeoutError(
            `coordinator exit timed out after ${exitTimeoutMs}ms`,
          );
        }),
      ]);
      retirement.confirmed = true;
      return { kind: "exited" };
    } catch (error) {
      reportProblem(
        `coordinator exit could not be confirmed: ${String(error)}`,
      );
      if (timedOut) {
        try {
          dependencies.onExitTimeout?.(exitTimeoutMs);
        } catch (reportError) {
          reportProblem(`coordinator exit timeout reporting failed: ${String(reportError)}`);
        }
      }
      if (exit !== undefined) {
        void exit.then(
          () => {
            retirement.confirmed = true;
            onEventualExit?.(retirement);
          },
          (eventualError) => {
            if (timedOut) {
              reportProblem(
                `coordinator exit remained unconfirmed: ${String(eventualError)}`,
              );
            }
          },
        );
      }
      return { kind: "unconfirmed", retirement };
    } finally {
      deadlineController.abort();
    }
  };

  const startForAccount = async (
    slot: string,
    context: CoordinatorAccountTransition,
    version: number,
  ): Promise<StartOutcome<Status>> => {
    if (isDisposed()) return { kind: "disposed" };
    let authorized = false;
    try {
      authorized = await dependencies.authorizeAccount(slot, context);
    } catch (error) {
      reportProblem(
        `account authorization for coordinator failed: ${String(error)}`,
      );
    }
    if (isDisposed()) return { kind: "disposed" };
    if (version !== observedVersion) return { kind: "superseded" };
    if (!authorized) {
      reportProblem(
        "coordinator kept unavailable because the host is not bound to this account",
      );
      if (isDisposed()) return { kind: "disposed" };
      let status = loggedOut;
      try {
        const revocation = await dependencies.revokeRefusedAccount();
        if (revocation.kind === "failed") {
          reportProblem(
            `refused account credentials could not be removed: ${String(revocation.error)}`,
          );
        }
        status = revocation.status;
      } catch (error) {
        reportProblem(
          `refused account credentials could not be removed: ${String(error)}`,
        );
      }
      if (version !== observedVersion) return { kind: "superseded" };
      return { kind: "refused", status };
    }
    if (isDisposed()) return { kind: "disposed" };
    const attempt: RuntimeLaunchAttempt = { slot, lifetime: { retired: false } };
    pendingLaunch = attempt;
    let session: CoordinatorRuntime;
    try {
      session = dependencies.createRuntime();
    } catch (error) {
      attempt.lifetime.retired = true;
      if (pendingLaunch === attempt) pendingLaunch = undefined;
      reportProblem(`coordinator launch failed: ${String(error)}`);
      return { kind: "launch-failed" };
    }
    const canActivate =
      pendingLaunch === attempt && !attempt.lifetime.retired && !isDisposed();
    if (pendingLaunch === attempt) pendingLaunch = undefined;
    if (!canActivate) {
      attempt.lifetime.retired = true;
      // The returned runtime owns port withdrawal for an unpublished candidate;
      // the active-runtime departure callback does not belong on this path.
      const stopped = await stop(session, enqueueReplacementRecovery);
      if (isDisposed()) return { kind: "disposed" };
      if (stopped.kind === "unconfirmed") {
        state = {
          kind: "blocked",
          slot,
          previousSlot: context.previousSlot,
          retirement: stopped.retirement,
        };
        return { kind: "blocked" };
      }
      return { kind: "superseded" };
    }
    activateSession({ slot, session, lifetime: attempt.lifetime, version });
    return { kind: "accepted" };
  };

  const deliver = (status: Status): void => {
    try {
      dependencies.deliverStatus(status);
    } catch (error) {
      reportProblem(`auth status delivery failed: ${String(error)}`);
    }
  };

  const settleObservedStatus = (status: Status): void => {
    settledStatus = status;
    deliver(status);
  };

  function enqueueReplacementRecovery(retirement: RuntimeRetirement): void {
    chain = chain.then(async () => {
      await startArrived;
      if (
        state.kind !== "blocked" ||
        state.retirement !== retirement ||
        !retirement.confirmed
      ) return;
      const blocked = state;
      const version = observedVersion;
      const nextSlot = cursorAccountSlot(settledStatus);
      if (nextSlot === null || nextSlot !== blocked.slot) {
        state = { kind: "inactive" };
        return;
      }
      const outcome = await startForAccount(
        nextSlot,
        { isStartup: false, previousSlot: blocked.previousSlot },
        version,
      );
      if (isDisposed() || outcome.kind === "disposed") return;
      if (outcome.kind === "accepted" || outcome.kind === "blocked") return;
      state = { kind: "inactive" };
      if (outcome.kind === "refused") settleObservedStatus(outcome.status);
    });
  }

  const applyClaim = async (
    nextSlot: string | null,
    status: Status,
    isStartup: boolean,
    version: number,
    settle: (status: Status) => void,
  ): Promise<void> => {
    if (isDisposed()) return;
    const previousSlot = accountSlot();
    if (state.kind === "blocked") {
      state = { ...state, slot: nextSlot };
      settle(status);
      return;
    }
    if (nextSlot === previousSlot) {
      if (state.kind === "active") state = { ...state, appliedVersion: version };
      settle(status);
      return;
    }
    if (state.kind === "active") state.lifetime.retired = true;
    revokeActiveRendererPortRequest();
    if (isDisposed()) return;
    if (previousSlot != null) {
      try {
        await dependencies.prepareAccountTransition({ previousSlot, nextSlot });
      } catch (error) {
        reportProblem(
          `account transition preparation failed: ${String(error)}`,
        );
      }
    }
    if (isDisposed()) return;
    const stopping = stop(detachActive(), enqueueReplacementRecovery);
    if (isDisposed()) {
      await stopping;
      return;
    }
    if (previousSlot !== undefined) {
      try {
        dependencies.resetAccountState();
      } catch (error) {
        reportProblem(`account state reset failed: ${String(error)}`);
      }
    }
    if (isDisposed()) {
      await stopping;
      return;
    }
    if (nextSlot === null) {
      state = { kind: "inactive" };
      settle(status);
      const stopped = await stopping;
      if (stopped.kind === "unconfirmed" && !isDisposed()) {
        state = {
          kind: "blocked", slot: null, previousSlot, retirement: stopped.retirement,
        };
      }
      return;
    }
    const stopped = await stopping;
    if (isDisposed()) return;
    if (stopped.kind === "unconfirmed") {
      state = {
        kind: "blocked", slot: nextSlot, previousSlot, retirement: stopped.retirement,
      };
      settle(status);
      return;
    }
    const outcome = await startForAccount(
      nextSlot,
      { isStartup, previousSlot },
      version,
    );
    if (isDisposed() || outcome.kind === "disposed") return;
    if (outcome.kind === "accepted") {
      settle(status);
      return;
    }
    if (outcome.kind === "blocked") return;
    state = { kind: "inactive" };
    if (outcome.kind === "superseded") return;
    settle(outcome.kind === "refused" ? outcome.status : status);
  };

  const waitForIdle = async (): Promise<Status> => {
    await startWork;
    while (true) {
      const pending = chain;
      await pending;
      if (pending === chain) return settledStatus;
    }
  };

  return {
    async start(status) {
      if (state.kind !== "unstarted" || startRequested) return;
      startRequested = true;
      // Publish the startup promise before invoking authorization/construction
      // collaborators, so reentrant start/dispose sees this exact owned work.
      startWork = Promise.resolve().then(async () => {
        try {
          await applyClaim(cursorAccountSlot(status), status, true, 0, (settled) => {
            settledStatus = settled;
          });
        } finally {
          releaseStart();
        }
      });
      await startWork;
    },
    observe(status) {
      if (isDisposed()) return;
      const version = ++observedVersion;
      const nextSlot = cursorAccountSlot(status);
      const changesSlot = nextSlot !== accountSlot();
      if (state.kind === "active" && nextSlot !== state.slot) {
        state.lifetime.retired = true;
      }
      if (pendingLaunch !== undefined && nextSlot !== pendingLaunch.slot) {
        pendingLaunch.lifetime.retired = true;
      }
      // Reserve observation order before renderer revocation can reenter observe.
      chain = chain.then(async () => {
        await startArrived;
        if (isDisposed()) return;
        await applyClaim(nextSlot, status, false, version, settleObservedStatus);
      });
      if (changesSlot) revokeActiveRendererPortRequest();
    },
    whenIdle: waitForIdle,
    captureCurrentBinding() {
      if (state.kind !== "active" || state.lifetime.retired) return null;
      const active = state;
      return {
        runtimeIdentity: active.session,
        assertCurrent() {
          if (
            active.lifetime.retired ||
            state.kind !== "active" ||
            state.lifetime !== active.lifetime ||
            state.session !== active.session
          ) {
            throw new Error("Coordinator account binding is no longer current.");
          }
        },
      };
    },
    requestRendererPort(sink) {
      requester = sink;
      if (
        state.kind === "active" &&
        !state.lifetime.retired &&
        state.appliedVersion === observedVersion
      ) {
        const active = state;
        state = { ...active, rendererRequestRevoked: false };
        active.session.requestRendererPort(sink);
      }
    },
    restart() {
      return state.kind === "active" && !state.lifetime.retired && state.appliedVersion === observedVersion
        ? state.session.restart()
        : Promise.resolve();
    },
    dispose() {
      if (disposeWork !== undefined) return disposeWork;
      const completion = Promise.withResolvers<void>();
      disposeWork = completion.promise;
      if (pendingLaunch !== undefined) pendingLaunch.lifetime.retired = true;
      const previous = detachActive(true);
      releaseStart();
      const stopping = stop(previous);
      void Promise.all([
        stopping,
        startWork.catch((error) => {
          reportProblem(
            `coordinator startup failed during disposal: ${String(error)}`,
          );
        }),
        chain.catch((error) => {
          reportProblem(
            `coordinator transition failed during disposal: ${String(error)}`,
          );
        }),
      ]).then(() => completion.resolve(), completion.reject);
      return disposeWork;
    },
  };
}
