export type ComputerRebuildKind =
  | "update"
  | "reset"
  | "recover"
  | "reconnecting";
export type ComputerRebuildUpdateSource =
  | "auto"
  | "request"
  | "migration";
export type ComputerRebuildTeardown = "none" | "transport" | "box";
export type ComputerRebuildResolution =
  | "settled"
  | "failed"
  | "cancelled";

export interface ComputerRebuildOperationId {
  readonly value: string;
}

export interface ComputerRebuildState {
  readonly kind: ComputerRebuildKind | null;
  readonly operationId: ComputerRebuildOperationId | null;
  readonly updateSource: ComputerRebuildUpdateSource | null;
  readonly lockBoxId: string | null;
  readonly isPending: boolean;
  readonly hasRequestAcknowledgement: boolean;
  readonly imageUpdateAvailable: boolean | undefined;
  readonly expectsImageUpgrade: boolean;
  readonly boxPhase: string | null;
  readonly observedBoxId: string | null;
  readonly lastHealthyBoxId: string | null;
  readonly hasLeftHealthy: boolean;
  readonly teardownObserved: ComputerRebuildTeardown;
  readonly reconnectedSinceLeft: boolean;
  readonly readySince: number | null;
  readonly isConnected: boolean;
  readonly connectedSince: number | null;
  readonly resetOperationId: ComputerRebuildOperationId | null;
  readonly hasTerminalMigration: boolean;
  readonly lastResolution: ComputerRebuildResolution | null;
  readonly lastResolutionKind: ComputerRebuildKind | null;
}

export type ComputerRebuildEvent =
  | {
      readonly type: "request";
      readonly kind: ComputerRebuildKind;
      readonly operationId?: ComputerRebuildOperationId | null;
      readonly source?: ComputerRebuildUpdateSource | null;
      readonly at: number;
    }
  | { readonly type: "pending"; readonly isPending: boolean; readonly at: number }
  | { readonly type: "acknowledged"; readonly at: number }
  | {
      readonly type: "image-update";
      readonly available: boolean | undefined;
      readonly at: number;
    }
  | {
      readonly type: "box";
      readonly boxId: string | null;
      readonly phase: string | null;
      readonly at: number;
    }
  | {
      readonly type: "migration";
      readonly operationId: ComputerRebuildOperationId | null;
      readonly phase:
        | "backing-up"
        | "creating"
        | "moving"
        | "cleaning-up"
        | "wiping"
        | "done"
        | "failed";
      readonly at: number;
    }
  | {
      readonly type: "connection";
      readonly isConnected: boolean;
      readonly at: number;
    }
  | { readonly type: "error"; readonly at: number }
  | { readonly type: "deactivate"; readonly at: number }
  | { readonly type: "tick"; readonly at: number };

function resetLike(kind: ComputerRebuildKind | null): boolean {
  return kind === "reset" || kind === "recover";
}

function healthy(phase: string | null): boolean {
  return phase === "running" || phase === "local";
}

function sameOperation(
  a: ComputerRebuildOperationId | null,
  b: ComputerRebuildOperationId | null,
): boolean {
  return a != null && b != null && a.value === b.value;
}

function sameLockedBox(state: ComputerRebuildState): boolean {
  return state.lockBoxId == null
    || state.observedBoxId == null
    || state.lockBoxId === state.observedBoxId;
}

function clearOperation(
  state: ComputerRebuildState,
  resolution: ComputerRebuildResolution | null,
): ComputerRebuildState {
  const previousKind = state.kind;
  return {
    ...state,
    kind: null,
    operationId: null,
    updateSource: null,
    lockBoxId: null,
    hasLeftHealthy: false,
    teardownObserved: "none",
    reconnectedSinceLeft: false,
    readySince: null,
    resetOperationId: null,
    hasTerminalMigration: false,
    hasRequestAcknowledgement: false,
    expectsImageUpgrade: false,
    lastResolution: resolution,
    lastResolutionKind: previousKind,
  };
}

export function initialComputerRebuildState(
  boxPhase: string | null,
  imageUpdateAvailable?: boolean,
): ComputerRebuildState {
  return {
    kind: null,
    operationId: null,
    updateSource: null,
    lockBoxId: null,
    isPending: false,
    hasRequestAcknowledgement: false,
    imageUpdateAvailable,
    expectsImageUpgrade: false,
    boxPhase,
    observedBoxId: null,
    lastHealthyBoxId: null,
    hasLeftHealthy: false,
    teardownObserved: "none",
    reconnectedSinceLeft: false,
    readySince: null,
    isConnected: true,
    connectedSince: null,
    resetOperationId: null,
    hasTerminalMigration: false,
    lastResolution: null,
    lastResolutionKind: null,
  };
}

function begin(
  state: ComputerRebuildState,
  event: Extract<ComputerRebuildEvent, { type: "request" }>,
): ComputerRebuildState {
  const operationId = resetLike(event.kind)
    ? event.operationId ?? null
    : null;
  const updateSource =
    event.kind === "update" ? event.source ?? null : null;

  if (state.kind != null) {
    if (
      resetLike(event.kind)
      && (
        !resetLike(state.kind)
        || (
          event.operationId != null
          && !sameOperation(state.resetOperationId, event.operationId)
        )
      )
    ) {
      return {
        ...state,
        kind: event.kind,
        operationId,
        resetOperationId: operationId,
        updateSource: null,
        lockBoxId: state.observedBoxId,
        hasLeftHealthy: true,
        reconnectedSinceLeft: false,
        readySince: null,
        hasTerminalMigration: false,
        hasRequestAcknowledgement: false,
        expectsImageUpgrade: false,
        lastResolution: null,
        lastResolutionKind: null,
      };
    }
    if (
      state.kind === "reconnecting"
      && event.kind !== "reconnecting"
    ) {
      return {
        ...state,
        kind: event.kind,
        operationId,
        resetOperationId: operationId,
        updateSource,
        lockBoxId: state.observedBoxId,
        expectsImageUpgrade:
          event.kind === "update"
          && state.imageUpdateAvailable === true,
      };
    }
    if (
      state.kind === "update"
      && event.kind === "update"
      && state.updateSource === "auto"
      && event.source != null
      && event.source !== "auto"
    ) {
      return { ...state, updateSource: event.source };
    }
    return state;
  }

  const leavesHealthy =
    event.kind === "reconnecting" || resetLike(event.kind);
  return {
    ...state,
    kind: event.kind,
    operationId,
    resetOperationId: operationId,
    updateSource,
    lockBoxId: state.observedBoxId,
    hasLeftHealthy: leavesHealthy,
    reconnectedSinceLeft: false,
    readySince:
      !leavesHealthy && healthy(state.boxPhase) ? event.at : null,
    connectedSince:
      state.isConnected ? state.connectedSince ?? event.at : null,
    hasTerminalMigration: false,
    hasRequestAcknowledgement: false,
    expectsImageUpgrade:
      event.kind === "update" && state.imageUpdateAvailable === true,
    lastResolution: null,
    lastResolutionKind: null,
  };
}

export function reduceComputerRebuildState(
  state: ComputerRebuildState,
  event: ComputerRebuildEvent,
): ComputerRebuildState {
  switch (event.type) {
    case "request":
      return begin(state, event);

    case "pending":
      if (!event.isPending && state.kind == null) {
        return { ...clearOperation(state, null), isPending: false };
      }
      return { ...state, isPending: event.isPending };

    case "acknowledged":
      return state.kind === "update" || resetLike(state.kind)
        ? { ...state, hasRequestAcknowledgement: true }
        : state;

    case "image-update":
      return { ...state, imageUpdateAvailable: event.available };

    case "box": {
      if (
        state.observedBoxId === event.boxId
        && state.boxPhase === event.phase
      ) {
        return state;
      }

      let next: ComputerRebuildState = {
        ...state,
        observedBoxId: event.boxId,
        boxPhase: event.phase,
        lastHealthyBoxId: healthy(event.phase)
          ? event.boxId
          : state.lastHealthyBoxId,
      };

      if (
        next.kind == null
        && !next.isPending
        && event.phase === "pulling"
        && event.boxId != null
        && next.lastHealthyBoxId === event.boxId
      ) {
        next = begin(next, {
          type: "request",
          kind: "update",
          source: "auto",
          at: event.at,
        });
      }

      if (next.kind == null) return next;
      if (next.lockBoxId == null && event.boxId != null) {
        next = { ...next, lockBoxId: event.boxId };
      }
      if (!sameLockedBox(next)) {
        return { ...next, readySince: null };
      }
      return healthy(event.phase)
        ? {
            ...next,
            readySince: next.readySince ?? event.at,
          }
        : {
            ...next,
            hasLeftHealthy: true,
            teardownObserved: "box",
            readySince: null,
          };
    }

    case "migration": {
      if (event.phase === "failed") {
        return state.kind == null
          ? state
          : clearOperation(state, "failed");
      }

      if (event.phase === "done") {
        const eligible =
          resetLike(state.kind)
          || (
            state.kind === "update"
            && state.updateSource === "migration"
          );
        if (!eligible || state.hasTerminalMigration) return state;
        if (
          state.operationId != null
          && event.operationId != null
          && !sameOperation(state.operationId, event.operationId)
        ) {
          return state;
        }
        return {
          ...state,
          hasTerminalMigration: true,
          hasLeftHealthy: true,
          readySince:
            state.isConnected
            && healthy(state.boxPhase)
            && sameLockedBox(state)
              ? event.at
              : null,
        };
      }

      if (event.phase === "wiping") {
        return begin(state, {
          type: "request",
          kind: "reset",
          operationId: event.operationId,
          at: event.at,
        });
      }
      return begin(state, {
        type: "request",
        kind: "update",
        operationId: event.operationId,
        source: "migration",
        at: event.at,
      });
    }

    case "connection":
      if (event.isConnected) {
        return {
          ...state,
          isConnected: true,
          connectedSince: state.connectedSince ?? event.at,
          reconnectedSinceLeft:
            state.hasLeftHealthy || state.reconnectedSinceLeft,
          readySince:
            state.kind != null
            && state.hasLeftHealthy
            && healthy(state.boxPhase)
            && sameLockedBox(state)
              ? state.readySince ?? event.at
              : state.readySince,
        };
      }
      return {
        ...state,
        isConnected: false,
        connectedSince: null,
        hasLeftHealthy:
          state.kind == null ? state.hasLeftHealthy : true,
        teardownObserved:
          state.kind != null && state.teardownObserved === "none"
            ? "transport"
            : state.teardownObserved,
        readySince: state.kind == null ? state.readySince : null,
      };

    case "error":
      return state.kind == null
        ? state
        : clearOperation(state, "failed");

    case "deactivate":
      return state.kind == null
        ? state
        : clearOperation(
            state,
            state.hasTerminalMigration ? "settled" : "cancelled",
          );

    case "tick":
      return state;
  }
}

export function isComputerRebuildLocked(
  state: Pick<ComputerRebuildState, "kind">,
): boolean {
  return state.kind != null;
}
