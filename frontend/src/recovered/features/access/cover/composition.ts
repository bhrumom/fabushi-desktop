import {
  isComputerRebuildLocked,
  type ComputerRebuildState,
} from "./computer-rebuild-model.ts";
import {
  shouldShowAccessCover,
  type SandAccess,
} from "./model.ts";
import type { FirstBoxGateState } from "./first-box-gate.ts";

export interface AccessRosterSnapshot {
  readonly failure: { readonly code: string } | null;
  readonly isShowingRestoredRoster: boolean;
  readonly loadState: "loading" | "ready" | "error";
  readonly isFetching: boolean;
}

export interface AccessCoverCompositionInput {
  readonly access: SandAccess;
  readonly roster: AccessRosterSnapshot;
  readonly firstBox: FirstBoxGateState;
  readonly rebuildStates: readonly Pick<ComputerRebuildState, "kind">[];
}

export interface AccessCoverCompositionState {
  readonly access: SandAccess;
  readonly rosterFailureCode: string | null;
  readonly hasReachedBox: boolean;
  readonly isShowingRestoredRoster: boolean;
  readonly isComputerRebuildLocked: boolean;
  readonly isLoading: boolean;
  readonly isError: boolean;
  readonly isVisible: boolean;
}

export function projectAccessCoverComposition(
  input: AccessCoverCompositionInput,
): AccessCoverCompositionState {
  const rebuildLocked = input.rebuildStates.some(
    isComputerRebuildLocked,
  );
  const rosterFailureCode = input.roster.failure?.code ?? null;
  const isLoading =
    input.access.state === "checking"
    || input.roster.loadState === "loading"
    || input.roster.isFetching;
  const isError =
    input.roster.loadState === "error"
    || input.access.state === "unknown";

  return {
    access: input.access,
    rosterFailureCode,
    hasReachedBox: input.firstBox.hasReachedBox,
    isShowingRestoredRoster: input.roster.isShowingRestoredRoster,
    isComputerRebuildLocked: rebuildLocked,
    isLoading,
    isError,
    isVisible: shouldShowAccessCover({
      rosterFailureCode,
      hasReachedBox: input.firstBox.hasReachedBox,
      isShowingRestoredRoster: input.roster.isShowingRestoredRoster,
      isComputerRebuildLocked: rebuildLocked,
    }),
  };
}
