export const ROSTER_SELECTION_SLICE = Object.freeze({
  slice: "selection.last-agent",
  schemaVersion: 1,
  scope: "client-persisted",
  accountSensitive: true,
} as const);

export interface RosterSelectionState {
  readonly currentAgentId: string | null;
  readonly isLoadPending: boolean;
}

export interface RosterSelectionPersistence {
  read(accountSlot: string): Promise<
    | { kind: "absent" }
    | { kind: "corrupt" }
    | { kind: "envelope"; schemaVersion: number; value: unknown }
  >;
  write(accountSlot: string, value: RosterSelectionState): Promise<void>;
  clear(accountSlot: string): Promise<void>;
}

export interface ClientPersistence {
  read(key: string): Promise<string | null>;
  write(key: string, value: string): Promise<void>;
  remove(key: string): Promise<void>;
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function parseEnvelope(value: string | null):
  | { kind: "absent" }
  | { kind: "corrupt" }
  | { kind: "envelope"; schemaVersion: number; value: unknown } {
  if (value == null) return { kind: "absent" };
  try {
    const parsed = record(JSON.parse(value));
    if (
      parsed == null
      || typeof parsed.schemaVersion !== "number"
      || !("value" in parsed)
    ) {
      return { kind: "corrupt" };
    }
    return {
      kind: "envelope",
      schemaVersion: parsed.schemaVersion,
      value: parsed.value,
    };
  } catch {
    return { kind: "corrupt" };
  }
}

function selection(value: unknown): RosterSelectionState | null {
  const row = record(value);
  return typeof row?.agentId === "string" && row.agentId.length > 0
    ? { currentAgentId: row.agentId, isLoadPending: false }
    : null;
}

function encodeAccountSlot(accountSlot: string): string {
  return encodeURIComponent(accountSlot).replaceAll(".", "%2E");
}

export function rosterSelectionPersistenceKey(
  accountSlot: string,
): string {
  if (accountSlot.length === 0) {
    throw new Error("accountSlot must not be empty");
  }
  return `sand.client.slice.account.${encodeAccountSlot(accountSlot)}.${ROSTER_SELECTION_SLICE.slice}`;
}

export function createRosterSelectionPersistence(
  client: ClientPersistence,
): RosterSelectionPersistence {
  return {
    async read(accountSlot) {
      return parseEnvelope(
        await client.read(rosterSelectionPersistenceKey(accountSlot)),
      );
    },
    async write(accountSlot, value) {
      await client.write(
        rosterSelectionPersistenceKey(accountSlot),
        JSON.stringify({
          schemaVersion: ROSTER_SELECTION_SLICE.schemaVersion,
          value: { agentId: value.currentAgentId },
        }),
      );
    },
    async clear(accountSlot) {
      await client.remove(rosterSelectionPersistenceKey(accountSlot));
    },
  };
}

export interface RosterSelectionStore {
  get(): RosterSelectionState;
  subscribe(listener: () => void): () => void;
  select(agentId: string | null): boolean;
  settle(agentId: string): void;
  reconcile(input: {
    agentIds: readonly string[];
    isRosterComplete: boolean;
  }): void;
  restore(accountSlot: string | null): Promise<void>;
  reset(): void;
  dispose(): void;
}

export function createRosterSelectionStore(
  persistence: RosterSelectionPersistence,
): RosterSelectionStore {
  let state: RosterSelectionState = {
    currentAgentId: null,
    isLoadPending: false,
  };
  let completeRosterAgentIds: readonly string[] | null = null;
  let accountSlot: string | null = null;
  let generation = 0;
  let disposed = false;
  let writes = Promise.resolve();
  const listeners = new Set<() => void>();

  const emit = (): void => {
    for (const listener of Array.from(listeners)) listener();
  };

  const replace = (next: RosterSelectionState): void => {
    if (
      next.currentAgentId === state.currentAgentId
      && next.isLoadPending === state.isLoadPending
    ) {
      return;
    }
    state = next;
    emit();
  };

  const persist = (): void => {
    if (accountSlot == null) return;
    const slot = accountSlot;
    const value = state;
    writes = writes
      .then(() => persistence.write(slot, value))
      .catch(() => {});
  };

  const current = (g: number, slot: string): boolean =>
    !disposed && generation === g && accountSlot === slot;

  return {
    get: () => state,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    select(agentId) {
      if (disposed) return false;
      const next =
        agentId == null || agentId.length === 0 ? null : agentId;
      if (next === state.currentAgentId) {
        return next != null && !state.isLoadPending;
      }
      replace({
        currentAgentId: next,
        isLoadPending: next != null,
      });
      persist();
      return next != null;
    },
    settle(agentId) {
      if (
        disposed
        || state.currentAgentId !== agentId
        || !state.isLoadPending
      ) {
        return;
      }
      const next =
        completeRosterAgentIds == null
        || completeRosterAgentIds.includes(agentId)
          ? agentId
          : completeRosterAgentIds[0] ?? null;
      replace({ currentAgentId: next, isLoadPending: false });
      if (next !== agentId) persist();
    },
    reconcile({ agentIds, isRosterComplete }) {
      if (disposed || !isRosterComplete) return;
      completeRosterAgentIds = [...agentIds];
      const available = new Set(agentIds);

      if (
        state.currentAgentId != null
        && !available.has(state.currentAgentId)
        && state.isLoadPending
      ) {
        return;
      }
      if (
        state.currentAgentId != null
        && available.has(state.currentAgentId)
      ) {
        return;
      }

      const next = agentIds[0] ?? null;
      replace({ currentAgentId: next, isLoadPending: false });
      persist();
    },
    async restore(nextAccountSlot) {
      generation += 1;
      const g = generation;
      accountSlot = nextAccountSlot;
      completeRosterAgentIds = null;
      replace({ currentAgentId: null, isLoadPending: false });
      if (disposed || nextAccountSlot == null) return;

      await writes;
      if (!current(g, nextAccountSlot)) return;
      const stored = await persistence.read(nextAccountSlot);
      if (!current(g, nextAccountSlot)) return;

      if (stored.kind === "absent") return;
      const restored =
        stored.kind === "envelope"
        && stored.schemaVersion === ROSTER_SELECTION_SLICE.schemaVersion
          ? selection(stored.value)
          : null;
      if (restored == null) {
        await persistence.clear(nextAccountSlot);
        return;
      }
      replace(restored);
    },
    reset() {
      generation += 1;
      accountSlot = null;
      completeRosterAgentIds = null;
      replace({ currentAgentId: null, isLoadPending: false });
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      accountSlot = null;
      completeRosterAgentIds = null;
      listeners.clear();
    },
  };
}
