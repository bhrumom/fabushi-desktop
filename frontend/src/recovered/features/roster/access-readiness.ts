export type RosterTransportState =
  | "browser"
  | "connecting"
  | "connected"
  | "down";
export type RosterLoadState = "loading" | "ready" | "error";

export interface RosterFailureSnapshot {
  readonly code: string;
  readonly transportKind?: string;
}

export interface RosterAccessReadinessInput {
  readonly accountKey: string | null;
  readonly transport: RosterTransportState;
  readonly loadState: RosterLoadState;
  readonly hasLoadedAgents: boolean;
  readonly agentIds: readonly string[];
  readonly selectedAgentId: string | null;
  readonly failure: RosterFailureSnapshot | null;
  readonly isShowingRestoredRoster: boolean;
  readonly isPrivacyBlocked: boolean;
}

export interface RosterAccessReadiness {
  readonly accountKey: string | null;
  readonly isAccountBound: boolean;
  readonly isConnected: boolean;
  readonly isLoaded: boolean;
  readonly hasReachedBox: boolean;
  readonly hasSelectedAgent: boolean;
  readonly isSelectionReady: boolean;
  readonly rosterFailureCode: string | null;
  readonly rosterFailureTransportKind: string | null;
  readonly isShowingRestoredRoster: boolean;
  readonly isPrivacyBlocked: boolean;
}

function sourceRecord(value: unknown): Record<string, unknown> | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

export function projectRosterFailure(value: unknown): RosterFailureSnapshot | null {
  const candidate = sourceRecord(value);
  if (candidate == null || typeof candidate.code !== "string" || candidate.code.length === 0) {
    return null;
  }
  return typeof candidate.transportKind === "string"
    ? { code: candidate.code, transportKind: candidate.transportKind }
    : { code: candidate.code };
}

export function selectRosterAccessReadiness(
  input: RosterAccessReadinessInput,
): RosterAccessReadiness {
  const isAccountBound = input.accountKey != null;
  const hasSelectedAgent =
    isAccountBound
    && input.selectedAgentId != null
    && input.agentIds.includes(input.selectedAgentId);
  const isLoaded =
    isAccountBound
    && input.hasLoadedAgents
    && input.loadState === "ready";
  const isConnected =
    isAccountBound
    && input.transport === "connected";

  return {
    accountKey: input.accountKey,
    isAccountBound,
    isConnected,
    isLoaded,
    hasReachedBox: isAccountBound && input.hasLoadedAgents,
    hasSelectedAgent,
    isSelectionReady: isLoaded && isConnected && hasSelectedAgent,
    rosterFailureCode: input.failure?.code ?? null,
    rosterFailureTransportKind: input.failure?.transportKind ?? null,
    isShowingRestoredRoster: input.isShowingRestoredRoster,
    isPrivacyBlocked: input.isPrivacyBlocked,
  };
}
