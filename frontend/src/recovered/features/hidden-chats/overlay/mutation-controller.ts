export interface HiddenChatsMutationAgent {
  readonly id: string;
  readonly isHidden: boolean;
  readonly updatedAt: number;
}

export interface HiddenChatsMutationController {
  setScope(accountSlot: string | null, activeAgentId: string | null): void;
  ingestAgents(agents: readonly HiddenChatsMutationAgent[]): void;
  setAgentHiddenFromSidebar(agentId: string, isHidden: boolean): Promise<void>;
  noteReconnect(): void;
  isPending(agentId: string): boolean;
  reset(): void;
  dispose(): void;
}

export interface HiddenChatsMutationControllerOptions {
  call(input: { id: string; isHidden: boolean }): Promise<unknown>;
  readAgent(agentId: string): HiddenChatsMutationAgent | null;
  onOptimisticChange(agentId: string, isHidden: boolean): void;
  onRollback(
    agentId: string,
    optimisticValue: boolean,
    previousValue: boolean,
  ): void;
}

interface Mutation {
  id: string;
  value: boolean;
  previousValue: boolean;
  inFlight: number;
  confirmed: boolean | null;
}

function transportFailure(error: unknown): boolean {
  if (typeof error !== "object" || error === null) return false;
  const row = error as Record<string, unknown>;
  return row.code === "source/transport-failure"
    || typeof row.transportKind === "string";
}

export function createHiddenChatsMutationController(
  options: HiddenChatsMutationControllerOptions,
): HiddenChatsMutationController {
  const mutations = new Map<string, Mutation>();
  const held = new Map<string, boolean>();
  let accountSlot: string | null = null;
  let activeAgentId: string | null = null;
  let generation = 0;
  let disposed = false;

  const current = (g: number): boolean =>
    !disposed && generation === g;

  const complete = (
    agentId: string,
    g: number,
    outcome: "confirm" | "rollback",
  ): void => {
    if (!current(g)) return;
    const mutation = mutations.get(agentId);
    if (mutation == null) return;

    if (mutation.inFlight > 1) {
      mutation.inFlight -= 1;
      return;
    }
    if (outcome === "rollback") {
      mutations.delete(agentId);
      held.delete(agentId);
      options.onRollback(
        agentId,
        mutation.value,
        mutation.previousValue,
      );
      return;
    }
    mutation.inFlight = 0;
    mutation.confirmed = mutation.value;
  };

  const execute = async (mutation: Mutation, g: number): Promise<void> => {
    try {
      await options.call({ id: mutation.id, isHidden: mutation.value });
      if (!current(g)) return;
      complete(mutation.id, g, "confirm");
      held.delete(mutation.id);
    } catch (error) {
      if (!current(g)) return;
      if (transportFailure(error)) {
        complete(mutation.id, g, "confirm");
        held.set(mutation.id, mutation.value);
      } else {
        complete(mutation.id, g, "rollback");
      }
      throw error;
    }
  };

  const retryHeld = (): void => {
    const g = generation;
    for (const [id, value] of Array.from(held)) {
      const existing = mutations.get(id);
      if (existing == null || existing.inFlight > 0) continue;
      const retry = { ...existing, value, inFlight: 1 };
      mutations.set(id, retry);
      void execute(retry, g).catch(() => {});
    }
  };

  return {
    setScope(nextAccountSlot, nextActiveAgentId) {
      if (
        disposed
        || (
          accountSlot === nextAccountSlot
          && activeAgentId === nextActiveAgentId
        )
      ) {
        return;
      }
      generation += 1;
      accountSlot = nextAccountSlot;
      activeAgentId = nextActiveAgentId;
      mutations.clear();
      held.clear();
    },
    ingestAgents(agents) {
      if (disposed) return;
      const byId = new Map(agents.map((agent) => [agent.id, agent]));
      for (const [id, mutation] of Array.from(mutations)) {
        if (mutation.inFlight > 0 || mutation.confirmed == null) continue;
        if (byId.get(id)?.isHidden === mutation.confirmed) {
          mutations.delete(id);
          held.delete(id);
        }
      }
    },
    async setAgentHiddenFromSidebar(agentId, isHidden) {
      if (
        disposed
        || accountSlot == null
        || agentId.length === 0
        || held.has(agentId)
        || (mutations.get(agentId)?.inFlight ?? 0) > 0
      ) {
        return;
      }
      const agent = options.readAgent(agentId);
      if (agent == null) return;

      const mutation: Mutation = {
        id: agentId,
        value: isHidden,
        previousValue: agent.isHidden,
        inFlight: 1,
        confirmed: null,
      };
      mutations.set(agentId, mutation);
      options.onOptimisticChange(agentId, isHidden);
      await execute(mutation, generation);
    },
    noteReconnect() {
      if (!disposed) retryHeld();
    },
    isPending(agentId) {
      const mutation = mutations.get(agentId);
      return !disposed
        && (
          held.has(agentId)
          || (mutation != null && mutation.inFlight > 0)
        );
    },
    reset() {
      generation += 1;
      accountSlot = null;
      activeAgentId = null;
      mutations.clear();
      held.clear();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      accountSlot = null;
      activeAgentId = null;
      mutations.clear();
      held.clear();
    },
  };
}
