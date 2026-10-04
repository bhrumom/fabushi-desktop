import assert from "node:assert/strict";
import test from "node:test";

import {
  ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS,
  ROUTINE_RUN_HISTORY_CLOCK_NAME,
  createRoutineRunHistoryClockOwner,
} from "./automations/routines/run-history-clock.ts";
import {
  createPluginServerToolsController,
  togglePluginServerToolOptimistically,
  type McpToolSummary,
} from "./plugins/overlay/server-tools.ts";

const flush = async (): Promise<void> => {
  await Promise.resolve();
  await Promise.resolve();
};

test("routine history clock is lazy, shared and timezone-aware", () => {
  let callback: (() => void) | null = null;
  let timerDisposed = 0;
  const clock = createRoutineRunHistoryClockOwner({
    initialTimeZone: {
      detectedTimeZone: "America/Los_Angeles",
      overrideTimeZone: null,
    },
    now: () => 123,
    scheduler: {
      schedule(input) {
        assert.equal(input.name, ROUTINE_RUN_HISTORY_CLOCK_NAME);
        assert.equal(input.intervalMs, ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS);
        callback = input.callback;
        return { dispose: () => { timerDisposed += 1; } };
      },
    },
  });

  assert.equal(clock.now(), 123);
  assert.equal(clock.timeZone?.(), "America/Los_Angeles");

  let notices = 0;
  const releaseA = clock.subscribe?.(() => { notices += 1; })!;
  const releaseB = clock.subscribe?.(() => { notices += 1; })!;
  callback?.();
  assert.equal(notices, 2);

  clock.ingestTimeZone({
    detectedTimeZone: "America/Los_Angeles",
    overrideTimeZone: "UTC",
  });
  assert.equal(clock.timeZone?.(), "UTC");
  assert.equal(notices, 4);

  releaseA();
  assert.equal(timerDisposed, 0);
  releaseB();
  assert.equal(timerDisposed, 1);
  clock.dispose();
});

test("server tools optimistically toggle and fence stale open/dispose state", async () => {
  const initial: McpToolSummary[] = [
    { name: "read", isDisabled: false },
    { name: "write", isDisabled: true },
  ];
  assert.deepEqual(
    togglePluginServerToolOptimistically(initial, "read")
      .map((tool) => [tool.name, tool.isDisabled]),
    [["read", true], ["write", true]],
  );

  let loadResolve: ((value: readonly McpToolSummary[]) => void) | null = null;
  const controller = createPluginServerToolsController(
    () => new Promise((resolve) => { loadResolve = resolve; }),
    async (name) => initial.map((tool) =>
      tool.name === name ? { ...tool, isDisabled: !tool.isDisabled } : tool
    ),
  );

  let notices = 0;
  controller.subscribe(() => { notices += 1; });
  controller.open();
  assert.equal(controller.getSnapshot().status, "loading");
  loadResolve?.(initial);
  await flush();
  assert.equal(controller.getSnapshot().status, "ready");

  const togglePromise = controller.toggle("read");
  assert.equal(controller.getSnapshot().pendingTool, "read");
  assert.equal(controller.getSnapshot().tools[0]?.isDisabled, true);
  await togglePromise;
  assert.equal(controller.getSnapshot().pendingTool, null);

  controller.dispose();
  assert.equal(notices > 0, true);
});
