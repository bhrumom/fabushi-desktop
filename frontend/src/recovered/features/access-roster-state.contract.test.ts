import assert from "node:assert/strict";
import test from "node:test";

import {
  initialComputerRebuildState,
  isComputerRebuildLocked,
  reduceComputerRebuildState,
} from "./access/cover/computer-rebuild-model.ts";
import {
  createComputerRebuildTransportStore,
} from "./access/cover/computer-rebuild-transport-store.ts";
import {
  projectAccessCoverComposition,
} from "./access/cover/composition.ts";
import {
  ACCESS_BLOCKED_FAILURE_CODE,
} from "./access/cover/model.ts";
import {
  createRosterSelectionPersistence,
  createRosterSelectionStore,
  rosterSelectionPersistenceKey,
} from "./roster/selection-state.ts";

test("computer rebuild reducer tracks lock, teardown, reconnect, migration and resolution", () => {
  let state = initialComputerRebuildState("running", true);
  state = reduceComputerRebuildState(state, {
    type: "box",
    boxId: "box-1",
    phase: "running",
    at: 10,
  });
  state = reduceComputerRebuildState(state, {
    type: "request",
    kind: "update",
    source: "request",
    at: 20,
  });
  assert.equal(isComputerRebuildLocked(state), true);
  assert.equal(state.expectsImageUpgrade, true);
  assert.equal(state.lockBoxId, "box-1");

  state = reduceComputerRebuildState(state, {
    type: "connection",
    isConnected: false,
    at: 30,
  });
  assert.equal(state.hasLeftHealthy, true);
  assert.equal(state.teardownObserved, "transport");

  state = reduceComputerRebuildState(state, {
    type: "connection",
    isConnected: true,
    at: 40,
  });
  assert.equal(state.reconnectedSinceLeft, true);

  state = reduceComputerRebuildState(state, {
    type: "deactivate",
    at: 50,
  });
  assert.equal(isComputerRebuildLocked(state), false);
  assert.equal(state.lastResolution, "cancelled");

  const op = { value: "op-1" };
  state = reduceComputerRebuildState(state, {
    type: "request",
    kind: "reset",
    operationId: op,
    at: 60,
  });
  state = reduceComputerRebuildState(state, {
    type: "migration",
    operationId: op,
    phase: "done",
    at: 70,
  });
  assert.equal(state.hasTerminalMigration, true);
  state = reduceComputerRebuildState(state, {
    type: "deactivate",
    at: 80,
  });
  assert.equal(state.lastResolution, "settled");
});

test("computer rebuild transport store hydrates ready and follows transport events", async () => {
  let listener: ((state: unknown) => void) | null = null;
  let released = 0;
  const store = createComputerRebuildTransportStore({
    source: {
      ready: Promise.resolve(),
      subscribeTransport(next) {
        listener = next;
        return () => { released += 1; };
      },
    },
    initialState: {
      ...initialComputerRebuildState("running"),
      isConnected: false,
    },
    now: () => 100,
  });

  let notices = 0;
  store.subscribe(() => { notices += 1; });
  await store.connect();
  assert.equal(store.getTransportState(), "connected");
  assert.equal(store.get().isConnected, true);

  listener?.("down");
  assert.equal(store.getTransportState(), "down");
  assert.equal(store.get().isConnected, false);
  listener?.("invalid");
  assert.equal(store.getTransportState(), "down");

  store.reset();
  assert.equal(released, 1);
  assert.equal(notices > 0, true);
  store.dispose();
});

test("access composition hides while rebuild is locked and otherwise honors access gate", () => {
  const unlocked = projectAccessCoverComposition({
    access: { state: "paymentRequired", reason: "paywallIndividual" },
    roster: {
      failure: { code: ACCESS_BLOCKED_FAILURE_CODE },
      isShowingRestoredRoster: false,
      loadState: "error",
      isFetching: false,
    },
    firstBox: { hasReachedBox: false, isAwaitingFirstBox: false },
    rebuildStates: [{ kind: null }],
  });
  assert.equal(unlocked.isVisible, true);
  assert.equal(unlocked.isError, true);

  const locked = projectAccessCoverComposition({
    ...{
      access: { state: "checking", reason: "unspecified" } as const,
      roster: {
        failure: { code: ACCESS_BLOCKED_FAILURE_CODE },
        isShowingRestoredRoster: false,
        loadState: "loading" as const,
        isFetching: true,
      },
      firstBox: { hasReachedBox: false, isAwaitingFirstBox: false },
    },
    rebuildStates: [{ kind: "update" }],
  });
  assert.equal(locked.isVisible, false);
  assert.equal(locked.isLoading, true);
  assert.equal(locked.isComputerRebuildLocked, true);
});

test("roster selection persists per account and fences restore/reconcile state", async () => {
  const storage = new Map<string, string>();
  const persistence = createRosterSelectionPersistence({
    read: async (key) => storage.get(key) ?? null,
    write: async (key, value) => { storage.set(key, value); },
    remove: async (key) => { storage.delete(key); },
  });
  assert.equal(
    rosterSelectionPersistenceKey("account.one"),
    "sand.client.slice.account.account%2Eone.selection.last-agent",
  );

  const store = createRosterSelectionStore(persistence);
  await store.restore("account.one");
  assert.equal(store.select("agent-2"), true);
  assert.deepEqual(store.get(), {
    currentAgentId: "agent-2",
    isLoadPending: true,
  });
  store.reconcile({
    agentIds: ["agent-1", "agent-2"],
    isRosterComplete: true,
  });
  store.settle("agent-2");
  assert.equal(store.get().isLoadPending, false);

  await Promise.resolve();
  await Promise.resolve();

  const restored = createRosterSelectionStore(persistence);
  await restored.restore("account.one");
  assert.equal(restored.get().currentAgentId, "agent-2");

  restored.select("missing");
  restored.reconcile({
    agentIds: ["agent-1"],
    isRosterComplete: true,
  });
  assert.equal(restored.get().currentAgentId, "missing");
  restored.settle("missing");
  assert.equal(restored.get().currentAgentId, "agent-1");

  store.dispose();
  restored.dispose();
});
