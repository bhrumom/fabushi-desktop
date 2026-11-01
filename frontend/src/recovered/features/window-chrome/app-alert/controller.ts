export type AppAlertFailure = string;
export type AppAlertCompletion = string | null | undefined | void;

export interface AppAlertSecondaryAction {
  readonly label: string;
  readonly destructive?: boolean;
  readonly perform?: () => Promise<AppAlertCompletion>;
}

export interface AppAlertRequest {
  readonly title: string;
  readonly description?: string;
  readonly body?: string;
  readonly warning?: string;
  readonly confirmLabel: string;
  readonly pendingLabel?: string;
  readonly cancelLabel?: string;
  readonly destructive?: boolean;
  readonly confirmLeadingIcon?: string;
  readonly secondary?: AppAlertSecondaryAction;
  readonly width?: "regular" | "wide";
  readonly perform?: () => Promise<AppAlertCompletion>;
}

export interface AppAlertState {
  readonly request: AppAlertRequest;
  readonly isPerforming: boolean;
  readonly failure: AppAlertFailure | null;
}

export interface AppAlertController {
  readonly getSnapshot: () => AppAlertState | null;
  readonly subscribe: (listener: () => void) => () => void;
  readonly alert: (request: AppAlertRequest) => Promise<boolean>;
  readonly confirm: () => void;
  readonly confirmSecondary: () => void;
  readonly cancel: () => void;
  readonly reset: () => void;
  readonly dispose: () => void;
}

type Settler = (accepted: boolean) => void;

interface PendingAlert {
  request: AppAlertRequest;
  settle: Settler;
}

function failureText(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

export function createAppAlertController(): AppAlertController {
  const subscribers = new Set<() => void>();
  let active: PendingAlert | null = null;
  let waiting: PendingAlert | null = null;
  let snapshot: AppAlertState | null = null;
  let epoch = 0;
  let disposed = false;

  const publish = (next: AppAlertState | null): void => {
    snapshot = next;
    for (const subscriber of Array.from(subscribers)) subscriber();
  };

  const show = (entry: PendingAlert): void => {
    active = entry;
    publish({ request: entry.request, isPerforming: false, failure: null });
  };

  const promoteWaiting = (): void => {
    if (disposed || waiting == null) return;
    const next = waiting;
    waiting = null;
    show(next);
  };

  const settleActive = (accepted: boolean): void => {
    if (active == null) return;
    const settle = active.settle;
    active = null;
    epoch += 1;
    publish(null);
    promoteWaiting();
    settle(accepted);
  };

  const performAction = (
    pick: (request: AppAlertRequest) => (() => Promise<AppAlertCompletion>) | undefined,
  ): void => {
    if (active == null || snapshot == null || snapshot.isPerforming) return;
    const action = pick(active.request);
    if (action == null) {
      settleActive(true);
      return;
    }

    const actionEpoch = epoch;
    publish({ ...snapshot, isPerforming: true, failure: null });
    void action()
      .then((result) => {
        if (actionEpoch !== epoch || active == null) return;
        const reportedFailure = result == null ? null : String(result);
        if (reportedFailure == null) {
          settleActive(true);
          return;
        }
        const current = snapshot;
        if (current != null) {
          publish({ ...current, isPerforming: false, failure: reportedFailure });
        }
      })
      .catch((error: unknown) => {
        if (actionEpoch !== epoch || active == null) return;
        const current = snapshot;
        if (current != null) {
          publish({ ...current, isPerforming: false, failure: failureText(error) });
        }
      });
  };

  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      subscribers.add(listener);
      return () => {
        subscribers.delete(listener);
      };
    },
    alert(request) {
      if (disposed) return Promise.resolve(false);
      return new Promise<boolean>((settle) => {
        const entry = { request, settle };
        if (active == null) {
          show(entry);
          return;
        }

        // A cancellable alert represents an explicit modal decision and must not
        // be silently superseded. Only one non-cancellable follow-up is retained.
        if (request.cancelLabel != null || waiting != null) {
          settle(false);
          return;
        }
        waiting = entry;
      });
    },
    confirm() {
      performAction((request) => request.perform);
    },
    confirmSecondary() {
      performAction((request) => request.secondary?.perform);
    },
    cancel() {
      if (snapshot?.isPerforming === true) return;
      settleActive(false);
    },
    reset() {
      const queued = waiting;
      waiting = null;
      queued?.settle(false);
      settleActive(false);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      const queued = waiting;
      waiting = null;
      queued?.settle(false);
      settleActive(false);
      subscribers.clear();
    },
  };
}
