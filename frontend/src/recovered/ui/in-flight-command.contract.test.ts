import assert from "node:assert/strict";
import test from "node:test";

import { createInFlightCommandFence } from "./in-flight-command.ts";

test("command fence refuses duplicate pointer, keyboard, programmatic and connected-handler entry", () => {
  const fence = createInFlightCommandFence();
  let effects = 0;
  const invoke = () => {
    const lease = fence.acquire("create-agent");
    if (lease == null) return null;
    effects += 1;
    return lease;
  };

  const pointer = invoke();
  assert.ok(pointer);
  assert.equal(fence.isInFlight("create-agent"), true);
  assert.equal(invoke(), null, "keyboard entry must be refused while pointer entry is in flight");
  assert.equal(invoke(), null, "programmatic entry must be refused while pointer entry is in flight");
  assert.equal(invoke(), null, "connected handler entry must be refused while pointer entry is in flight");
  assert.equal(effects, 1);

  pointer.release();
  assert.equal(fence.isInFlight("create-agent"), false);
  const restored = invoke();
  assert.ok(restored);
  assert.equal(effects, 2);
  restored.release();
});

test("lease restoration is idempotent after failed command settlement", () => {
  const fence = createInFlightCommandFence();
  const first = fence.acquire("human-handoff");
  assert.ok(first);
  first.release();
  first.release();

  const retry = fence.acquire("human-handoff");
  assert.ok(retry);
  retry.release();
});

test("dispose fails closed and stale leases cannot reopen the command", () => {
  const fence = createInFlightCommandFence();
  const lease = fence.acquire("create-agent");
  assert.ok(lease);
  fence.dispose();
  lease.release();
  assert.equal(fence.isInFlight("create-agent"), false);
  assert.equal(fence.acquire("create-agent"), null);
});
