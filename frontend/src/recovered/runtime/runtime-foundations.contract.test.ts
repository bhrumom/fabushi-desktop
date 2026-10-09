import assert from "node:assert/strict";
import test from "node:test";

import {
  SourceFailure,
  isSourceRecord,
} from "./source-boundary.ts";
import {
  alwaysAvailable,
  defineEntrypoint,
} from "./define-entrypoint.ts";
import { createSnapshotStore } from "./snapshot-store.ts";
import { recoveredEntrypoints } from "./entrypoints.ts";

test("source boundary keeps structured failures and excludes arrays", () => {
  const failure = new SourceFailure({ code: "host/down", retryable: true });
  assert.equal(failure.name, "SourceFailure");
  assert.equal(failure.message, "host/down");
  assert.equal(failure.failure.code, "host/down");
  assert.equal(failure.failure.retryable, true);

  assert.equal(isSourceRecord({ ok: true }), true);
  assert.equal(isSourceRecord([]), false);
  assert.equal(isSourceRecord(null), false);
  assert.equal(isSourceRecord("value"), false);
});

test("entrypoint definitions are immutable and preserve availability contract", async () => {
  const entry = defineEntrypoint<Record<string, never>>({
    availability: alwaysAvailable,
    loadView: async () => ({ default: (() => null) as never }),
  });
  assert.equal(Object.isFrozen(entry), true);
  assert.deepEqual(entry.availability({ gates: new Set(), hasAgents: false }), {
    kind: "available",
  });
});

test("snapshot store suppresses Object.is no-ops and unsubscribes cleanly", () => {
  const store = createSnapshotStore({ count: 0 });
  let notifications = 0;
  const unsubscribe = store.subscribe(() => {
    notifications += 1;
  });

  const initial = store.get();
  store.set(initial);
  assert.equal(notifications, 0);

  store.update((current) => ({ count: current.count + 1 }));
  assert.equal(store.get().count, 1);
  assert.equal(notifications, 1);

  unsubscribe();
  store.set({ count: 2 });
  assert.equal(store.get().count, 2);
  assert.equal(notifications, 1);
});

test("recovered entrypoint inventory has unique stable ids and known surfaces", () => {
  const ids = recoveredEntrypoints.map((entry) => entry.id);
  assert.equal(new Set(ids).size, ids.length);
  assert.deepEqual(ids, [
    "overlay:computer",
    "overlay:hidden-chats",
    "view:org-chart",
    "overlay:plugins",
    "overlay:settings",
  ]);
  assert.ok(recoveredEntrypoints.every((entry) =>
    entry.surface === "overlay" || entry.surface === "workspace"
  ));
});
