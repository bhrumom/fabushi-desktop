import assert from "node:assert/strict";
import test from "node:test";

import { createAgentSettingsController } from "./agent-info/settings/model.ts";

const status = (id: string, canReset: boolean, override: unknown) => ({
  key: `agent:${id}:main`,
  sessionId: id,
  lifecycleRevision: "rev-11",
  modelSelection: {
    version: 1,
    supported: true,
    override,
    default: { provider: "fabushi", modelId: "configured" },
    locked: false,
    canReset,
  },
});

const agent = (id: string) => ({
  id, name: id, description: "", isGroup: false,
  notifyOnUpdatesEnabled: false,
});

test("Agent Settings uses only server-projected reset authority and observed revision", async () => {
  const writes: unknown[] = [];
  let current = status("a", true, { provider: "fabushi", modelId: "overridden" });
  const source = {
    async updateAgent() { return {}; },
    async setAgentNotifyOnUpdates() {},
    async getSessionModelSelection() { return current; },
    async resetSessionModelSelection(input: unknown) {
      writes.push(input);
      return { ok: true, ...status("a", false, null) };
    },
  };
  const controller = createAgentSettingsController(source, agent("a"));
  await controller.refreshSessionModelSelection();
  assert.equal(controller.getSnapshot().modelSelection?.canReset, true);
  assert.equal(await controller.resetSessionModelSelection(), true);
  assert.deepEqual(writes, [{
    key: "agent:a:main",
    expectedSessionId: "a",
    expectedLifecycleRevision: "rev-11",
    modelSelection: { resetToDefault: true },
  }]);
  assert.equal(controller.getSnapshot().modelSelection?.override, null);
  assert.equal(await controller.resetSessionModelSelection(), false);
  assert.equal(writes.length, 1);

  current = status("a", false, { provider: "fabushi", modelId: "overridden" });
  await controller.refreshSessionModelSelection();
  assert.equal(await controller.resetSessionModelSelection(), false);
  assert.equal(writes.length, 1);
  controller.dispose();
});

test("Agent Settings ignores stale status after agent switch and fails closed on malformed reply", async () => {
  const pending: { release: ((value: unknown) => void) | null } = { release: null };
  let first = true;
  const source = {
    async updateAgent() { return {}; },
    async setAgentNotifyOnUpdates() {},
    getSessionModelSelection(args: { key: string }): Promise<unknown> {
      if (first) {
        first = false;
        return new Promise(resolve => { pending.release = resolve; });
      }
      return Promise.resolve(args.key === "agent:b:main"
        ? status("b", false, null)
        : { sessionId: "other", lifecycleRevision: "rev-11", modelSelection: {} });
    },
    async resetSessionModelSelection() { throw new Error("must not be called"); },
  };
  const controller = createAgentSettingsController(source, agent("a"));
  controller.setAgent(agent("b"));
  await controller.refreshSessionModelSelection();
  pending.release?.(status("a", true, { provider: "fabushi", modelId: "unsafe" }));
  await Promise.resolve();
  assert.equal(controller.getSnapshot().agent.id, "b");
  assert.equal(controller.getSnapshot().modelSelection?.sessionId, "b");
  assert.equal(controller.getSnapshot().modelSelection?.canReset, false);
  controller.dispose();
});
