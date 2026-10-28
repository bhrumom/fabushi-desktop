import assert from "node:assert/strict";
import test from "node:test";

import { createCoordinatorMainPortClient } from "./coordinator-main-port-client.js";
import { createCoordinatorMainLegs } from "./coordinator-main-legs.js";

test("a request post failure settles the pending call and closes the main-data port", async () => {
  const frames: unknown[] = [];
  let closed = 0;
  const client = createCoordinatorMainPortClient({
    post(frame) {
      frames.push(frame);
      if ((frame as { kind?: string }).kind === "request") throw new Error("port is gone");
    },
    close() {
      closed += 1;
    },
  });

  await assert.rejects(client.legs.setHostSettings({ settings: {} }), /post-failed/);
  assert.equal(closed, 1);
  assert.equal((await client.settled).outcome, "post-failed");
  assert.equal(frames.length, 2);
});

test("a hello post failure does not replace the current main-data generation", async () => {
  const failures: string[] = [];
  let closed = 0;
  const legs = createCoordinatorMainLegs({
    onProblem() {},
    reportFailure(leg) {
      failures.push(leg);
    },
  });
  legs.adoptPort({
    postMessage() {
      throw new Error("hello post failed");
    },
    close() {
      closed += 1;
    },
    on() {},
    start() {},
  });

  assert.deepEqual(failures, ["leg-post"]);
  assert.equal(closed, 1);
  await assert.rejects(legs.legs.getHostSettings!(), /no live coordinator session/);
});

test("a protocol breach still settles when the shutdown frame cannot be posted", async () => {
  let closed = 0;
  const client = createCoordinatorMainPortClient({
    post(frame) {
      if ((frame as { phase?: string }).phase === "shutdown") throw new Error("port is gone");
    },
    close() {
      closed += 1;
    },
  });

  client.handleMessage({ kind: "invalid-frame" });
  assert.equal((await client.settled).outcome, "protocol-breach");
  assert.equal(closed, 1);
});
