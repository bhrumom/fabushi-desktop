import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  ROUTINE_RUN_HISTORY_CLOCK_INTERVAL_MS,
  ROUTINE_RUN_HISTORY_CLOCK_NAME,
  createRoutineRunHistoryClockOwner,
} from "./automations/routines/run-history-clock.ts";
import { loadWithSingleCacheHitRecovery } from "../../../../source/packages/cursor-plugins/cache-hit-recovery.ts";
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


test("marketplace cache-hit load failure invalidates and reinstalls exactly once", async () => {
  const sequence: string[] = [];
  let attempts = 0;
  const result = await loadWithSingleCacheHitRecovery({
    wasCacheHit: true,
    async load() {
      sequence.push("load");
      attempts += 1;
      if (attempts === 1) throw new Error("cached payload is unreadable");
      return "recovered";
    },
    async invalidate() { sequence.push("invalidate"); },
    async reinstall() { sequence.push("reinstall"); },
    onRecoveryAttempt(error) {
      sequence.push(`recover:${error instanceof Error ? error.message : String(error)}`);
    },
  });
  assert.equal(result, "recovered");
  assert.deepEqual(sequence, [
    "load",
    "recover:cached payload is unreadable",
    "invalidate",
    "reinstall",
    "load",
  ]);
});

test("marketplace cache recovery never retries a miss or loops after a failed reinstall load", async () => {
  let invalidations = 0;
  let reinstalls = 0;
  await assert.rejects(
    () => loadWithSingleCacheHitRecovery({
      wasCacheHit: false,
      async load() { throw new Error("fresh install load failed"); },
      async invalidate() { invalidations += 1; },
      async reinstall() { reinstalls += 1; },
    }),
    /fresh install load failed/,
  );
  assert.equal(invalidations, 0);
  assert.equal(reinstalls, 0);

  let loads = 0;
  await assert.rejects(
    () => loadWithSingleCacheHitRecovery({
      wasCacheHit: true,
      async load() { loads += 1; throw new Error(`load-${loads}`); },
      async invalidate() { invalidations += 1; },
      async reinstall() { reinstalls += 1; },
    }),
    /load-2/,
  );
  assert.equal(loads, 2);
  assert.equal(invalidations, 1);
  assert.equal(reinstalls, 1);
});

test("shipping marketplace loader binds cache-hit recovery to invalidation and reinstall", () => {
  const loaderSource = readFileSync(
    new URL("../../../../source/packages/cursor-plugins/loader.ts", import.meta.url),
    "utf8",
  );
  assert.match(loaderSource, /loadWithSingleCacheHitRecovery\(\{/);
  assert.match(loaderSource, /wasCacheHit: cacheHit/);
  assert.match(loaderSource, /invalidate: \(\) => cacheManager\.removeVersion\(cacheArgs\)/);
  assert.match(loaderSource, /reinstall: installIntoCache/);
  assert.match(loaderSource, /marketplace\.cache_hit_load_recovery_attempt/);
});
