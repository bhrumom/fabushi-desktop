import type {
  AppAlertController,
  AppAlertRequest,
} from "../../window-chrome/app-alert/controller.ts";

export const GROUP_MAX_MEMBERS = 6;

export interface GroupMemberAgent {
  readonly id: string;
  readonly name: string;
  readonly isGroup: boolean;
  readonly memberIds: readonly string[];
  readonly isSharedRoom?: boolean;
}

export interface GroupRosterSourceSnapshot {
  readonly accountGeneration: number;
  readonly agents: readonly unknown[];
}

export interface GroupRosterSource {
  getSnapshot(): GroupRosterSourceSnapshot;
  subscribe(listener: () => void): () => void;
  setGroupMembers(args: {
    readonly id: string;
    readonly memberAgentIds: readonly string[];
  }): Promise<unknown>;
}

export type GroupMembersPending =
  | {
      readonly kind: "add" | "remove";
      readonly agentId: string;
      readonly generation: number;
    }
  | null;

export interface GroupMembersSnapshot {
  readonly group: GroupMemberAgent | null;
  readonly members: readonly GroupMemberAgent[];
  readonly candidates: readonly GroupMemberAgent[];
  readonly canAdd: boolean;
  readonly canRemove: boolean;
  readonly pending: GroupMembersPending;
  readonly failure: unknown | null;
  readonly accountGeneration: number;
}

export interface GroupMembersProvider {
  getSnapshot(): GroupMembersSnapshot;
  subscribe(listener: () => void): () => void;
  setContext(agent: GroupMemberAgent | null, accountGeneration: number): void;
  addMember(agentId: string): Promise<boolean>;
  requestRemoveMember(agent: Pick<GroupMemberAgent, "id" | "name">): Promise<boolean>;
  reset(): void;
  dispose(): void;
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function nonEmptyStringArray(value: unknown): value is string[] {
  return Array.isArray(value)
    && value.every((item) => typeof item === "string" && item.length > 0);
}

export function projectGroupMemberAgent(value: unknown): GroupMemberAgent | null {
  const row = record(value);
  if (
    row == null
    || typeof row.id !== "string"
    || row.id.length === 0
    || typeof row.name !== "string"
    || typeof row.isGroup !== "boolean"
    || !nonEmptyStringArray(row.memberIds)
    || (row.isSharedRoom !== undefined && typeof row.isSharedRoom !== "boolean")
  ) {
    return null;
  }
  return {
    id: row.id,
    name: row.name,
    isGroup: row.isGroup,
    memberIds: [...row.memberIds],
    ...(row.isSharedRoom === undefined
      ? {}
      : { isSharedRoom: row.isSharedRoom }),
  };
}

const EMPTY_GROUP_SNAPSHOT: GroupMembersSnapshot = Object.freeze({
  group: null,
  members: [],
  candidates: [],
  canAdd: false,
  canRemove: false,
  pending: null,
  failure: null,
  accountGeneration: -1,
});

function sameAgent(
  left: GroupMemberAgent | null,
  right: GroupMemberAgent | null,
): boolean {
  if (left === right) return true;
  if (left == null || right == null) return false;
  return left.id === right.id
    && left.name === right.name
    && left.isGroup === right.isGroup
    && left.isSharedRoom === right.isSharedRoom
    && left.memberIds.length === right.memberIds.length
    && left.memberIds.every((id, index) => id === right.memberIds[index]);
}

function removalAlert(
  name: string,
  perform: () => Promise<void>,
): AppAlertRequest {
  return {
    title: `Remove ${name} from this conversation?`,
    confirmLabel: "Remove",
    pendingLabel: "Removing...",
    cancelLabel: "Cancel",
    destructive: true,
    perform: async () => {
      try {
        await perform();
        return null;
      } catch {
        return "Removing failed. Check your connection and try again.";
      }
    },
  };
}

export function createGroupMembersProvider(
  source: GroupRosterSource,
  alert: Pick<AppAlertController, "alert" | "reset">,
): GroupMembersProvider {
  let context: GroupMemberAgent | null = null;
  let contextGeneration = -1;
  let actionGeneration = 0;
  let pending: GroupMembersPending = null;
  let failure: unknown | null = null;
  let disposed = false;
  let snapshot = EMPTY_GROUP_SNAPSHOT;
  const listeners = new Set<() => void>();

  const roster = (): GroupMemberAgent[] =>
    source.getSnapshot().agents.flatMap((value) => {
      const projected = projectGroupMemberAgent(value);
      return projected == null ? [] : [projected];
    });

  const derive = (): GroupMembersSnapshot => {
    const sourceSnapshot = source.getSnapshot();
    const agents = roster();
    const group =
      context == null
      || !context.isGroup
      || context.isSharedRoom === true
        ? null
        : agents.find((agent) => agent.id === context?.id) ?? context;

    if (
      group == null
      || sourceSnapshot.accountGeneration !== contextGeneration
    ) {
      return {
        ...EMPTY_GROUP_SNAPSHOT,
        accountGeneration: sourceSnapshot.accountGeneration,
        pending,
        failure,
      };
    }

    const byId = new Map(agents.map((agent) => [agent.id, agent]));
    const members = group.memberIds.flatMap((id) => {
      const member = byId.get(id);
      return member == null ? [] : [member];
    });
    const memberIds = new Set(group.memberIds);
    const candidates = agents.filter(
      (agent) =>
        !agent.isGroup
        && agent.id !== group.id
        && !memberIds.has(agent.id),
    );

    return {
      group,
      members,
      candidates,
      canAdd:
        group.memberIds.length < GROUP_MAX_MEMBERS
        && candidates.length > 0
        && pending == null,
      canRemove: group.memberIds.length > 1 && pending == null,
      pending,
      failure,
      accountGeneration: sourceSnapshot.accountGeneration,
    };
  };

  const publish = (): void => {
    if (disposed) return;
    snapshot = derive();
    for (const listener of Array.from(listeners)) listener();
  };

  const currentGroup = (generation: number): GroupMemberAgent | null => {
    if (disposed || generation !== actionGeneration) return null;
    const sourceSnapshot = source.getSnapshot();
    if (sourceSnapshot.accountGeneration !== contextGeneration) return null;
    const group = roster().find((agent) => agent.id === context?.id) ?? null;
    return group?.isGroup === true && group.isSharedRoom !== true
      ? group
      : null;
  };

  const mutate = async (
    kind: "add" | "remove",
    groupId: string,
    agentId: string,
    memberIds: readonly string[],
    generation: number,
    rethrow: boolean,
  ): Promise<boolean> => {
    pending = { kind, agentId, generation };
    failure = null;
    publish();
    try {
      await source.setGroupMembers({
        id: groupId,
        memberAgentIds: [...memberIds],
      });
      return true;
    } catch (error) {
      if (!disposed && generation === actionGeneration) {
        failure = error;
        publish();
      }
      if (rethrow) throw error;
      return false;
    } finally {
      if (!disposed && generation === actionGeneration) {
        pending = null;
        publish();
      }
    }
  };

  const stopSource = source.subscribe(publish);

  return {
    getSnapshot() {
      return snapshot;
    },
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    setContext(agent, accountGeneration) {
      if (disposed) return;
      const changed =
        !sameAgent(context, agent)
        || contextGeneration !== accountGeneration;
      context = agent;
      contextGeneration = accountGeneration;
      if (changed) {
        actionGeneration += 1;
        pending = null;
        failure = null;
        alert.reset();
      }
      publish();
    },
    async addMember(agentId) {
      const generation = actionGeneration;
      const group = currentGroup(generation);
      if (
        group == null
        || pending != null
        || group.memberIds.length >= GROUP_MAX_MEMBERS
      ) {
        return false;
      }
      const candidate = roster().find((agent) => agent.id === agentId);
      if (
        candidate == null
        || candidate.isGroup
        || candidate.id === group.id
        || group.memberIds.includes(candidate.id)
      ) {
        return false;
      }
      return mutate(
        "add",
        group.id,
        candidate.id,
        [...group.memberIds, candidate.id],
        generation,
        false,
      );
    },
    async requestRemoveMember(agent) {
      const generation = actionGeneration;
      const group = currentGroup(generation);
      if (
        group == null
        || pending != null
        || group.memberIds.length <= 1
        || !group.memberIds.includes(agent.id)
      ) {
        return false;
      }

      return alert.alert(removalAlert(agent.name, async () => {
        const latest = currentGroup(generation);
        if (
          latest == null
          || latest.memberIds.length <= 1
          || !latest.memberIds.includes(agent.id)
        ) {
          return;
        }
        await mutate(
          "remove",
          latest.id,
          agent.id,
          latest.memberIds.filter((id) => id !== agent.id),
          generation,
          true,
        );
      }));
    },
    reset() {
      if (disposed) return;
      actionGeneration += 1;
      context = null;
      contextGeneration = -1;
      pending = null;
      failure = null;
      alert.reset();
      publish();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      actionGeneration += 1;
      stopSource();
      alert.reset();
      listeners.clear();
    },
  };
}
