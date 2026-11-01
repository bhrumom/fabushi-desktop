import assert from "node:assert/strict";
import test from "node:test";

import { settleOptionalRemoteComputerTarget } from "./remote-target-availability.js";

test("remote target success remains visible without cleanup", async () => {
  let cleanup = 0;
  const result = await settleOptionalRemoteComputerTarget(
    async () => {},
    async () => { cleanup += 1; },
  );
  assert.equal(result, "ready");
  assert.equal(cleanup, 0);
});

test("remote target failure is isolated from the valid account session", async () => {
  let cleanup = 0;
  const result = await settleOptionalRemoteComputerTarget(
    async () => { throw new Error("coordinator unavailable"); },
    async () => { cleanup += 1; },
  );
  assert.equal(result, "unavailable");
  assert.equal(cleanup, 1);
});

test("target cleanup failure remains fail-closed and is not silently swallowed", async () => {
  await assert.rejects(
    () => settleOptionalRemoteComputerTarget(
      async () => { throw new Error("registration failed"); },
      async () => { throw new Error("credential cleanup failed"); },
    ),
    /credential cleanup failed/,
  );
});
