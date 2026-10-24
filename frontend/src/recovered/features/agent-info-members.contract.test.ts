import assert from "node:assert/strict";
import test from "node:test";

import {
  GROUP_MAX_MEMBERS,
  createGroupMembersProvider,
  projectGroupMemberAgent,
} from "./agent-info/group-members/model.ts";
import {
  createGenerationFencedGroupRosterSource,
  typedCoordinatorSetGroupMembers,
} from "./agent-info/group-members/bridge.ts";

test("group member projection is strict and provider derives candidates/mutations", async () => {
  assert.equal(projectGroupMemberAgent(null), null);
  assert.equal(projectGroupMemberAgent({
    id: "bad",
    name: "Bad",
    isGroup: false,
    memberIds: [""],
  }), null);

  let generation = 3;
  let rows: unknown[] = [
    { id: "g", name: "Group", isGroup: true, memberIds: ["a", "b"] },
    { id: "a", name: "A", isGroup: false, memberIds: [] },
    { id: "b", name: "B", isGroup: false, memberIds: [] },
    { id: "c", name: "C", isGroup: false, memberIds: [] },
  ];
  const mutations: unknown[] = [];
  let sourceListener: (() => void) | null = null;
  let alertResets = 0;
  const source = {
    getSnapshot: () => ({ accountGeneration: generation, agents: rows }),
    subscribe(listener: () => void) {
      sourceListener = listener;
      return () => { sourceListener = null; };
    },
    async setGroupMembers(args: unknown) {
      mutations.push(args);
      const a = args as { id: string; memberAgentIds: string[] };
      rows = rows.map((row) =>
        (row as { id?: string }).id === a.id
          ? { ...(row as object), memberIds: a.memberAgentIds }
          : row
      );
      sourceListener?.();
      return {};
    },
  };
  const provider = createGroupMembersProvider(source, {
    reset: () => { alertResets += 1; },
    alert: async (request) => {
      const result = await request.perform?.();
      return result == null;
    },
  });
  provider.setContext(
    { id: "g", name: "Group", isGroup: true, memberIds: ["a", "b"] },
    generation,
  );
  let snapshot = provider.getSnapshot();
  assert.deepEqual(snapshot.members.map((member) => member.id), ["a", "b"]);
  assert.deepEqual(snapshot.candidates.map((member) => member.id), ["c"]);
  assert.equal(snapshot.canAdd, true);
  assert.equal(snapshot.canRemove, true);

  assert.equal(await provider.addMember("c"), true);
  snapshot = provider.getSnapshot();
  assert.deepEqual(snapshot.group?.memberIds, ["a", "b", "c"]);
  assert.equal(mutations.length, 1);

  assert.equal(
    await provider.requestRemoveMember({ id: "a", name: "A" }),
    true,
  );
  assert.deepEqual(provider.getSnapshot().group?.memberIds, ["b", "c"]);
  assert.equal(alertResets >= 1, true);

  provider.setContext({
    id: "g",
    name: "Group",
    isGroup: true,
    memberIds: Array.from({ length: GROUP_MAX_MEMBERS }, (_, i) => `m${i}`),
  }, generation);
  assert.equal(await provider.addMember("c"), false);

  generation += 1;
  sourceListener?.();
  assert.equal(provider.getSnapshot().group, null);
  provider.dispose();
});

test("group bridge validates RPC payloads/replies and drops stale generations", async () => {
  const calls: unknown[] = [];
  const client = {
    async call(method: string, args?: unknown) {
      calls.push([method, args]);
      return { ok: true };
    },
  };
  assert.deepEqual(
    await typedCoordinatorSetGroupMembers(client, {
      id: "g",
      memberAgentIds: ["a", "b"],
    }),
    { ok: true },
  );
  await assert.rejects(
    () => typedCoordinatorSetGroupMembers(client, {
      id: "",
      memberAgentIds: [],
    }),
    /non-empty/,
  );

  let ownerListener: (() => void) | null = null;
  let ownerGeneration = 4;
  const source = createGenerationFencedGroupRosterSource({
    getAgents: () => [],
    getAccountGeneration: () => ownerGeneration,
    subscribe(listener) {
      ownerListener = listener;
      return () => { ownerListener = null; };
    },
  }, client);

  let notices = 0;
  source.subscribe(() => { notices += 1; });
  ownerListener?.();
  assert.equal(notices, 1);
  assert.equal(source.getSnapshot().accountGeneration, 4);

  source.reset();
  ownerGeneration = 5;
  ownerListener?.();
  assert.equal(notices, 1);

  source.dispose();
});
