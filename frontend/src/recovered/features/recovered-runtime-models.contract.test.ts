import assert from "node:assert/strict";
import test from "node:test";

import {
  ASYNC_TASKS_CLOCK_INTERVAL_MS,
  createStableAsyncTasksClock,
} from "./agent-info/async-tasks/clock.ts";
import {
  normalizeTerminalOutput,
  projectTerminalOutput,
} from "./terminal/output/model.ts";
import {
  projectRosterFailure,
  selectRosterAccessReadiness,
} from "./roster/access-readiness.ts";

test("stable async-task clock starts lazily, shares a snapshot, and stops on last unsubscribe", () => {
  let now = 100;
  let scheduled: (() => void) | null = null;
  let disposed = 0;
  const clock = createStableAsyncTasksClock(
    { now: () => now },
    {
      start(listener, intervalMs) {
        assert.equal(intervalMs, ASYNC_TASKS_CLOCK_INTERVAL_MS);
        scheduled = listener;
        return { dispose: () => { disposed += 1; } };
      },
    },
  );

  assert.equal(clock.now(), 100);
  let notifications = 0;
  const releaseA = clock.subscribe(() => { notifications += 1; });
  const releaseB = clock.subscribe(() => { notifications += 1; });
  now = 130;
  scheduled?.();
  assert.equal(clock.now(), 130);
  assert.equal(notifications, 2);

  releaseA();
  assert.equal(disposed, 0);
  releaseB();
  assert.equal(disposed, 1);
  releaseB();
  assert.equal(disposed, 1);
});

test("stable async-task clock delegates to an injected source subscription when present", () => {
  let now = 1;
  let sourceListener: (() => void) | null = null;
  let sourceDisposed = 0;
  const clock = createStableAsyncTasksClock({
    now: () => now,
    subscribe(listener) {
      sourceListener = listener;
      return () => { sourceDisposed += 1; };
    },
  });

  const release = clock.subscribe(() => {});
  now = 2;
  sourceListener?.();
  assert.equal(clock.now(), 2);
  release();
  assert.equal(sourceDisposed, 1);
});

test("terminal projection normalizes legacy field variants and status priority", () => {
  assert.equal(normalizeTerminalOutput("a\r\nb\rc"), "a\nb\nc");
  assert.equal(projectTerminalOutput(null), null);
  assert.equal(projectTerminalOutput({ sessionId: "s" }), null);

  assert.deepEqual(
    projectTerminalOutput({
      terminal_instance_id: 42,
      metadata: {
        cwd: "/repo",
        currentCommand: { command: "npm test" },
      },
      output_raw: "one\r\ntwo",
      exit_code: 0,
    }),
    {
      sessionId: "42",
      command: "npm test",
      cwd: "/repo",
      output: "one\ntwo",
      status: "exited",
      exitCode: 0,
    },
  );

  assert.equal(
    projectTerminalOutput({
      sessionId: "run",
      command: "build",
      status: "running",
      exitCode: 7,
    })?.status,
    "running",
  );
  assert.equal(
    projectTerminalOutput({
      sessionId: "run",
      command: "build",
      exitCode: 7,
    })?.status,
    "error",
  );
});

test("roster readiness is account-bound and requires loaded connected selected state", () => {
  assert.deepEqual(projectRosterFailure({ code: "NETWORK", transportKind: "dns" }), {
    code: "NETWORK",
    transportKind: "dns",
  });
  assert.equal(projectRosterFailure({ code: "" }), null);
  assert.equal(projectRosterFailure([]), null);

  const ready = selectRosterAccessReadiness({
    accountKey: "account-1",
    transport: "connected",
    loadState: "ready",
    hasLoadedAgents: true,
    agentIds: ["a", "b"],
    selectedAgentId: "b",
    failure: null,
    isShowingRestoredRoster: false,
    isPrivacyBlocked: false,
  });
  assert.equal(ready.isAccountBound, true);
  assert.equal(ready.isConnected, true);
  assert.equal(ready.isLoaded, true);
  assert.equal(ready.hasReachedBox, true);
  assert.equal(ready.hasSelectedAgent, true);
  assert.equal(ready.isSelectionReady, true);

  const disconnected = selectRosterAccessReadiness({
    accountKey: "account-1",
    transport: "down",
    loadState: "error",
    hasLoadedAgents: true,
    agentIds: ["a"],
    selectedAgentId: "a",
    failure: { code: "DOWN", transportKind: "network" },
    isShowingRestoredRoster: true,
    isPrivacyBlocked: false,
  });
  assert.equal(disconnected.hasReachedBox, true);
  assert.equal(disconnected.isSelectionReady, false);
  assert.equal(disconnected.rosterFailureCode, "DOWN");

  const loggedOut = selectRosterAccessReadiness({
    accountKey: null,
    transport: "connected",
    loadState: "ready",
    hasLoadedAgents: true,
    agentIds: ["a"],
    selectedAgentId: "a",
    failure: null,
    isShowingRestoredRoster: false,
    isPrivacyBlocked: true,
  });
  assert.equal(loggedOut.isAccountBound, false);
  assert.equal(loggedOut.hasReachedBox, false);
  assert.equal(loggedOut.hasSelectedAgent, false);
  assert.equal(loggedOut.isSelectionReady, false);
});
