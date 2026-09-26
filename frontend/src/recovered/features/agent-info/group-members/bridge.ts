import type {
  GroupRosterSource,
  GroupRosterSourceSnapshot,
} from "./model.ts";

export interface CoordinatorCallClient {
  call(method: string, args?: unknown): Promise<unknown>;
}

export interface SetGroupMembersArgs {
  readonly id: string;
  readonly memberAgentIds: readonly string[];
}

export type SetGroupMembersReply = Record<string, unknown> | null;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export async function typedCoordinatorSetGroupMembers(
  client: CoordinatorCallClient,
  args: SetGroupMembersArgs,
): Promise<SetGroupMembersReply> {
  if (
    args.id.length === 0
    || args.memberAgentIds.some((id) => id.length === 0)
  ) {
    throw new TypeError("setGroupMembers requires non-empty agent ids");
  }

  const reply = await client.call("setGroupMembers", {
    id: args.id,
    memberAgentIds: [...args.memberAgentIds],
  });
  if (reply === null || isRecord(reply)) return reply;
  throw new TypeError("setGroupMembers returned a malformed record reply");
}

export interface SynchronizedRosterOwner {
  getAgents(): readonly unknown[];
  getAccountGeneration(): number;
  subscribe(listener: () => void): () => void;
}

export interface GenerationFencedGroupRosterSource extends GroupRosterSource {
  reset(): void;
  dispose(): void;
}

export function createGenerationFencedGroupRosterSource(
  owner: SynchronizedRosterOwner,
  client: CoordinatorCallClient,
): GenerationFencedGroupRosterSource {
  let lifecycleGeneration = 0;
  let disposed = false;
  const releases = new Set<() => void>();

  const getSnapshot = (): GroupRosterSourceSnapshot => ({
    accountGeneration: owner.getAccountGeneration(),
    agents: owner.getAgents(),
  });

  return {
    getSnapshot,
    subscribe(listener) {
      if (disposed) return () => {};
      const subscribedGeneration = lifecycleGeneration;
      const release = owner.subscribe(() => {
        if (
          !disposed
          && subscribedGeneration === lifecycleGeneration
        ) {
          listener();
        }
      });
      releases.add(release);
      return () => {
        releases.delete(release);
        release();
      };
    },
    async setGroupMembers(args) {
      if (disposed) return null;
      const requestGeneration = lifecycleGeneration;
      try {
        const reply = await typedCoordinatorSetGroupMembers(client, args);
        return !disposed && requestGeneration === lifecycleGeneration
          ? reply
          : null;
      } catch (error) {
        if (disposed || requestGeneration !== lifecycleGeneration) return null;
        throw error;
      }
    },
    reset() {
      if (!disposed) lifecycleGeneration += 1;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      lifecycleGeneration += 1;
      for (const release of Array.from(releases)) release();
      releases.clear();
    },
  };
}
