import { ACCESS_BLOCKED_FAILURE_CODE } from "./model.ts";

export type FirstBoxRosterLoadState = "loading" | "ready" | "error";

export interface FirstBoxRosterSnapshot {
  readonly loadState: FirstBoxRosterLoadState;
  readonly isShowingRestoredRoster: boolean;
  readonly failureCode: string | null;
  readonly failureTransportKind: string | null;
}

export interface FirstBoxGateState {
  readonly isAwaitingFirstBox: boolean;
  readonly hasReachedBox: boolean;
}

export const INITIAL_FIRST_BOX_GATE: FirstBoxGateState = Object.freeze({
  isAwaitingFirstBox: false,
  hasReachedBox: false,
});

function connectivityFailure(kind: string | null): boolean {
  return kind === "network" || kind === "dns";
}

export function projectFirstBoxGate(
  previous: FirstBoxGateState,
  roster: FirstBoxRosterSnapshot,
): FirstBoxGateState {
  const hasReachedBox = previous.hasReachedBox || roster.loadState === "ready";
  const suppressed =
    roster.isShowingRestoredRoster
    || roster.failureCode === ACCESS_BLOCKED_FAILURE_CODE
    || connectivityFailure(roster.failureTransportKind);

  return {
    hasReachedBox,
    isAwaitingFirstBox:
      roster.loadState !== "loading"
      && !hasReachedBox
      && !suppressed,
  };
}

export function resetFirstBoxGate(): FirstBoxGateState {
  return INITIAL_FIRST_BOX_GATE;
}
