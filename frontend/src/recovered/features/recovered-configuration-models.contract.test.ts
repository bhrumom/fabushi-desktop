import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_INSTRUCTIONS_PER_BEHAVIOR,
  instructionRows,
  parseCursorAuthId,
  reconcileInstructionRow,
  removeInstruction,
  saveInstruction,
} from "./settings/overlay/model.ts";
import {
  DEFAULT_ROUTER_PROVIDER,
  ROUTER_PROVIDER_PERSISTENCE_KEY,
  isRouterProviderId,
  loadRouterProvider,
  parseRouterProviderPreference,
  routerProviderById,
  saveRouterProvider,
} from "./settings/overlay/router.ts";
import {
  buildOrgChartEdges,
  getAgentActivity,
  getEdgeActivity,
  reconcileOrgChartAgentSelection,
  toggleOrgChartAgentSelection,
  type OrgChartAgent,
} from "./org-chart/workspace/model.ts";
import {
  AGENTS_SECTION_ID,
  projectSidebarSectionHeader,
  projectSidebarSections,
} from "./conversation/workspace/sidebar-section-projection.ts";

test("settings instruction model preserves list identity, duplicate rejection and cross-list edits", () => {
  assert.deepEqual(parseCursorAuthId("auth0|subject-7"), { subject: "subject-7" });
  assert.equal(parseCursorAuthId("invalid"), null);

  const base = {
    allowInstructions: ["read files", "run tests"],
    blockInstructions: ["delete files"],
  };
  assert.deepEqual(instructionRows(base), [
    { behavior: "allow", text: "read files", listIndex: 0 },
    { behavior: "allow", text: "run tests", listIndex: 1 },
    { behavior: "ask", text: "delete files", listIndex: 0 },
  ]);

  assert.equal(
    saveInstruction(base, "run tests", "allow", null),
    null,
  );
  assert.deepEqual(
    saveInstruction(
      base,
      "run checks",
      "allow",
      { behavior: "allow", text: "run tests", listIndex: 1 },
    ),
    {
      allowInstructions: ["read files", "run checks"],
      blockInstructions: ["delete files"],
    },
  );
  assert.deepEqual(
    saveInstruction(
      base,
      "delete files",
      "allow",
      { behavior: "ask", text: "delete files", listIndex: 0 },
    ),
    {
      allowInstructions: ["read files", "run tests", "delete files"],
      blockInstructions: [],
    },
  );

  const removed = removeInstruction(base, {
    behavior: "allow",
    text: "read files",
    listIndex: 0,
  });
  assert.deepEqual(removed.allowInstructions, ["run tests"]);

  assert.deepEqual(
    reconcileInstructionRow(
      { ...base, allowInstructions: ["new", "run tests"] },
      { behavior: "allow", text: "run tests", listIndex: 0 },
    ),
    { behavior: "allow", text: "run tests", listIndex: 1 },
  );

  const full = {
    allowInstructions: Array.from(
      { length: MAX_INSTRUCTIONS_PER_BEHAVIOR },
      (_, index) => `allow-${index}`,
    ),
    blockInstructions: [],
  };
  assert.equal(saveInstruction(full, "overflow", "allow", null), null);
});

test("router provider preference fails closed and persists versioned provider state", async () => {
  assert.equal(isRouterProviderId("codex"), true);
  assert.equal(isRouterProviderId("missing"), false);
  assert.equal(routerProviderById("openrouter").usageSource, "external");
  assert.equal(parseRouterProviderPreference(null), DEFAULT_ROUTER_PROVIDER);
  assert.equal(parseRouterProviderPreference("{broken"), DEFAULT_ROUTER_PROVIDER);
  assert.equal(
    parseRouterProviderPreference(JSON.stringify({ schemaVersion: 2, provider: "codex" })),
    DEFAULT_ROUTER_PROVIDER,
  );
  assert.equal(
    parseRouterProviderPreference(JSON.stringify({ schemaVersion: 1, provider: "codex" })),
    "codex",
  );

  const store = new Map<string, string>();
  const persistence = {
    read: async (key: string) => store.get(key) ?? null,
    write: async (key: string, value: string) => {
      store.set(key, value);
    },
  };
  await saveRouterProvider(persistence, "claude-code");
  assert.equal(store.has(ROUTER_PROVIDER_PERSISTENCE_KEY), true);
  assert.equal(await loadRouterProvider(persistence), "claude-code");
});

function agent(input: Partial<OrgChartAgent> & Pick<OrgChartAgent, "id">): OrgChartAgent {
  return {
    id: input.id,
    isGroup: input.isGroup ?? false,
    memberIds: input.memberIds ?? [],
    conversationPartnerIds: input.conversationPartnerIds ?? [],
    awaitingUserResponse: input.awaitingUserResponse ?? null,
    isRunning: input.isRunning ?? false,
    updatedAt: input.updatedAt ?? 0,
    ...input,
  };
}

test("org chart model deduplicates membership/message edges and projects activity", () => {
  const agents = [
    agent({ id: "group", isGroup: true, memberIds: ["a", "b", "missing"] }),
    agent({ id: "a", conversationPartnerIds: ["b"], isRunning: true, updatedAt: 900 }),
    agent({ id: "b", conversationPartnerIds: ["a"], isRunning: true, updatedAt: 950 }),
  ];
  const edges = buildOrgChartEdges(agents);
  assert.deepEqual(edges.map((edge) => edge.key), [
    "member::group::a",
    "member::group::b",
    "msg::a::b",
  ]);

  const byId = new Map(agents.map((value) => [value.id, value]));
  const message = edges.find((edge) => edge.kind === "message")!;
  assert.equal(getEdgeActivity(message, byId, 1000), "talking");

  const waiting = agent({
    id: "wait",
    awaitingUserResponse: { kind: "question" },
    isRunning: true,
  });
  assert.equal(getAgentActivity(waiting, () => true, () => true), "waiting");
  assert.equal(getAgentActivity(agent({ id: "type" }), () => true, () => false), "typing");
  assert.equal(getAgentActivity(agent({ id: "work" }), () => false, () => true), "working");

  const selected = toggleOrgChartAgentSelection(null, "a");
  assert.deepEqual(selected, { kind: "agent", id: "a" });
  assert.equal(toggleOrgChartAgentSelection(selected, "a"), null);
  assert.equal(reconcileOrgChartAgentSelection(selected, [{ id: "b" }]), null);
});

test("sidebar projection excludes pinned agents, preserves section order and omits empty synthetic section", () => {
  const agents = [{ id: "a" }, { id: "b" }, { id: "c" }, { id: "pinned" }];
  const projected = projectSidebarSections({
    agents,
    pinnedIds: ["pinned"],
    sections: [
      {
        id: "work",
        name: "Work",
        agentIds: ["b", "missing"],
        isCollapsed: false,
      },
      {
        id: AGENTS_SECTION_ID,
        name: "Unassigned",
        agentIds: [],
        isCollapsed: true,
      },
      {
        id: "empty",
        name: "Empty",
        agentIds: [],
        isCollapsed: false,
      },
    ],
  });
  assert.deepEqual(projected.map((section) => [
    section.id,
    section.agents.map((value) => value.id),
  ]), [
    ["work", ["b"]],
    [AGENTS_SECTION_ID, ["a", "c"]],
    ["empty", []],
  ]);
  assert.deepEqual(projectSidebarSectionHeader(projected[1]!), {
    id: AGENTS_SECTION_ID,
    label: "Unassigned",
    count: 2,
    isFolded: true,
    isLocked: true,
    ariaExpanded: false,
    dataSectionId: AGENTS_SECTION_ID,
  });
});
