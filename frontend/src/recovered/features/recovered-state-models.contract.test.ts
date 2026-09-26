import assert from "node:assert/strict";
import test from "node:test";

import {
  createAsyncTasksProvider,
  formatAsyncTaskTime,
} from "./agent-info/async-tasks/provider.ts";
import {
  AVATAR_MAX_ZOOM,
  AVATAR_MIN_ZOOM,
  avatarCropRect,
  clampAvatarCrop,
  clampAvatarZoom,
  initialAvatarCrop,
  panAvatarCrop,
  visibleAvatarSide,
} from "./agent-info/avatar-editor/model.ts";

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

async function settle(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
  await Promise.resolve();
}

test("async-task provider validates replies, retains previous data on failure, and fences stale reconnect replies", async () => {
  const calls: Array<{ id: string; gate: ReturnType<typeof deferred<unknown>> }> = [];
  const provider = createAsyncTasksProvider({
    getAsyncTasks({ id }) {
      const gate = deferred<unknown>();
      calls.push({ id, gate });
      return gate.promise;
    },
  });

  const handle = provider.snapshotsFor("agent-a");
  const unsubscribe = handle.subscribe(() => {});
  assert.deepEqual(handle.get(), { status: "loading" });

  provider.connect();
  assert.equal(calls.length, 1);
  assert.equal(calls[0]!.id, "agent-a");

  calls[0]!.gate.resolve([{
    kind: "shell",
    id: "task-1",
    label: "Build",
    status: "running",
    startedAtMs: 1_000,
  }]);
  await settle();
  let snapshot = handle.get();
  assert.equal(snapshot.status, "ready");
  if (snapshot.status === "ready") assert.equal(snapshot.value[0]?.id, "task-1");

  provider.refresh("agent-a");
  assert.equal(calls.length, 2);
  calls[1]!.gate.reject(new Error("offline"));
  await settle();
  snapshot = handle.get();
  assert.equal(snapshot.status, "failed");
  if (snapshot.status === "failed" && "previous" in snapshot) {
    assert.equal(snapshot.previous[0]?.id, "task-1");
  } else {
    assert.fail("expected failed snapshot to retain previous tasks");
  }

  provider.refresh("agent-a");
  assert.equal(calls.length, 3);
  provider.noteReconnect();
  assert.equal(calls.length, 4);

  calls[2]!.gate.resolve([]);
  await settle();
  assert.notEqual(handle.get().status, "empty");

  calls[3]!.gate.resolve([]);
  await settle();
  assert.equal(handle.get().status, "empty");

  provider.ingestAsyncTasksEvent({
    parentAgentId: "agent-a",
    tasks: [{
      kind: "cloud-agent",
      id: "task-event",
      label: "Remote",
      status: "running",
      startedAtMs: 2_000,
    }],
  });
  snapshot = handle.get();
  assert.equal(snapshot.status, "ready");
  if (snapshot.status === "ready") assert.equal(snapshot.value[0]?.id, "task-event");

  unsubscribe();
  provider.dispose();
});

test("async-task provider exposes capability-unavailable and the recovered relative-time buckets", async () => {
  const provider = createAsyncTasksProvider({
    async getAsyncTasks() {
      throw { code: "source/capability-unavailable" };
    },
  });
  const handle = provider.snapshotsFor("agent-b");
  handle.subscribe(() => {});
  provider.connect();
  await settle();

  assert.deepEqual(handle.get(), {
    status: "unavailable",
    reason: "source/capability-unavailable",
  });
  assert.equal(formatAsyncTaskTime(10_000, 10_500), "now");
  assert.equal(formatAsyncTaskTime(10_000, 130_000), "2m ago");
  assert.equal(formatAsyncTaskTime(10_000, 7_210_000), "2h ago");
  assert.equal(formatAsyncTaskTime(Number.NaN, 1), "");
});

test("avatar crop model clamps zoom, crop bounds, and stage-space panning", () => {
  assert.equal(clampAvatarZoom(Number.NaN), AVATAR_MIN_ZOOM);
  assert.equal(clampAvatarZoom(-100), AVATAR_MIN_ZOOM);
  assert.equal(clampAvatarZoom(100), AVATAR_MAX_ZOOM);

  const source = { dataUrl: "data:image/png;base64,x", width: 800, height: 400 };
  assert.deepEqual(initialAvatarCrop(source.width, source.height), {
    zoom: 1,
    centerX: 400,
    centerY: 200,
  });
  assert.equal(visibleAvatarSide(source, 2), 200);

  const bounded = clampAvatarCrop(source, {
    zoom: 2,
    centerX: -20,
    centerY: 999,
  });
  assert.deepEqual(bounded, { zoom: 2, centerX: 100, centerY: 300 });
  assert.deepEqual(avatarCropRect(source, bounded), {
    x: 0,
    y: 200,
    width: 200,
    height: 200,
  });

  const panned = panAvatarCrop(
    source,
    { zoom: 2, centerX: 400, centerY: 200 },
    130,
    -130,
  );
  assert.deepEqual(panned, { zoom: 2, centerX: 300, centerY: 300 });
});
