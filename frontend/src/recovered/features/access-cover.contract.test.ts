import assert from "node:assert/strict";
import test from "node:test";

import {
  ACCESS_BLOCKED_FAILURE_CODE,
  ACCESS_ONBOARDING_URL,
  SAND_ACCESS_UNKNOWN,
  accessNoticeCopy,
  isSandAccess,
  openAccessOnboarding,
  projectSandAccess,
  readFreshSandAccess,
  shouldShowAccessCover,
} from "./access/cover/model.ts";
import {
  INITIAL_FIRST_BOX_GATE,
  projectFirstBoxGate,
  resetFirstBoxGate,
} from "./access/cover/first-box-gate.ts";
import {
  createDevBoxRebuildSignalStore,
} from "./access/cover/rebuild-signal.ts";

test("access model validates states and only shows the blocking cover before first box", async () => {
  assert.equal(isSandAccess({ state: "granted", reason: "none" }), true);
  assert.equal(isSandAccess({ state: "bad", reason: "none" }), false);
  assert.equal(isSandAccess([]), false);
  assert.equal(projectSandAccess({ state: "bad", reason: "none" }), SAND_ACCESS_UNKNOWN);

  const fresh = await readFreshSandAccess({
    getSandAccessFresh: async () => ({
      state: "paymentRequired",
      reason: "paywallIndividual",
    }),
  });
  assert.equal(fresh.state, "paymentRequired");
  assert.equal(accessNoticeCopy(fresh)?.action, "Upgrade");

  assert.equal(shouldShowAccessCover({
    rosterFailureCode: ACCESS_BLOCKED_FAILURE_CODE,
    hasReachedBox: false,
    isShowingRestoredRoster: false,
    isComputerRebuildLocked: false,
  }), true);
  assert.equal(shouldShowAccessCover({
    rosterFailureCode: ACCESS_BLOCKED_FAILURE_CODE,
    hasReachedBox: true,
    isShowingRestoredRoster: false,
    isComputerRebuildLocked: false,
  }), false);

  let opened: string | null = null;
  await openAccessOnboarding({
    openExternal: async (url) => {
      opened = url;
    },
  });
  assert.equal(opened, ACCESS_ONBOARDING_URL);
});

test("first-box gate latches ready and suppresses access/network/restored false alarms", () => {
  const afterError = projectFirstBoxGate(INITIAL_FIRST_BOX_GATE, {
    loadState: "error",
    isShowingRestoredRoster: false,
    failureCode: "OTHER",
    failureTransportKind: null,
  });
  assert.equal(afterError.isAwaitingFirstBox, true);
  assert.equal(afterError.hasReachedBox, false);

  for (const snapshot of [
    {
      loadState: "error" as const,
      isShowingRestoredRoster: true,
      failureCode: "OTHER",
      failureTransportKind: null,
    },
    {
      loadState: "error" as const,
      isShowingRestoredRoster: false,
      failureCode: ACCESS_BLOCKED_FAILURE_CODE,
      failureTransportKind: null,
    },
    {
      loadState: "error" as const,
      isShowingRestoredRoster: false,
      failureCode: "OTHER",
      failureTransportKind: "network",
    },
  ]) {
    assert.equal(
      projectFirstBoxGate(INITIAL_FIRST_BOX_GATE, snapshot).isAwaitingFirstBox,
      false,
    );
  }

  const ready = projectFirstBoxGate(INITIAL_FIRST_BOX_GATE, {
    loadState: "ready",
    isShowingRestoredRoster: false,
    failureCode: null,
    failureTransportKind: null,
  });
  assert.equal(ready.hasReachedBox, true);
  const laterDown = projectFirstBoxGate(ready, {
    loadState: "error",
    isShowingRestoredRoster: false,
    failureCode: "DOWN",
    failureTransportKind: "network",
  });
  assert.equal(laterDown.hasReachedBox, true);
  assert.equal(laterDown.isAwaitingFirstBox, false);
  assert.equal(resetFirstBoxGate(), INITIAL_FIRST_BOX_GATE);
});

test("rebuild signal increments generations, acknowledges only current generation, and disposes bridge", () => {
  let bridgeListener: (() => void) | null = null;
  let bridgeDisposed = 0;
  const store = createDevBoxRebuildSignalStore({
    onDevBoxRebuild(listener) {
      bridgeListener = listener;
      return () => {
        bridgeDisposed += 1;
      };
    },
  });

  let notifications = 0;
  const release = store.subscribe(() => {
    notifications += 1;
  });
  bridgeListener?.();
  assert.deepEqual(store.get(), { generation: 1, isPending: true });
  store.acknowledge(0);
  assert.equal(store.get().isPending, true);
  store.acknowledge(1);
  assert.deepEqual(store.get(), { generation: 1, isPending: false });

  bridgeListener?.();
  assert.deepEqual(store.get(), { generation: 2, isPending: true });
  assert.equal(notifications, 3);

  release();
  store.dispose();
  assert.equal(bridgeDisposed, 1);
  store.acknowledge(2);
  assert.equal(store.get().isPending, true);
});
