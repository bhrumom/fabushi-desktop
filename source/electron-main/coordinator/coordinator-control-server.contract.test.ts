import assert from "node:assert/strict";
import test from "node:test";

import { createCoordinatorControlServer } from "./coordinator-control-server.js";

test("a throwing problem reporter cannot prevent protocol settlement", async () => {
  const server = createCoordinatorControlServer({
    post() {},
    executors: {},
    onEvent: {},
    onProblem() {
      throw new Error("diagnostics failed");
    },
  });

  server.handleMessage({
    kind: "lifecycle",
    phase: "hello",
    protocolVersion: 1,
  });
  server.handleMessage({
    kind: "event",
    family: "undeclared-family",
    payload: {},
  });

  assert.equal((await server.settled).outcome, "protocol-breach");
});
