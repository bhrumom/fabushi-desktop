import assert from "node:assert/strict";
import test from "node:test";
import { settleForwardProjection } from "./forward-settlement.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}
function harness() {
  let generation = 3;
  const notices: (string | null)[] = [];
  const rosters: unknown[] = [];
  return {
    notices, rosters,
    isCurrent: (value: number) => value === generation,
    invalidate: () => { generation += 1; },
    apply: (value: unknown) => rosters.push(value),
    notice: (value: string | null) => notices.push(value),
  };
}
test("current forward success projects notice and roster", async () => {
  const h = harness();
  await settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["c"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(h.rosters, [["c"]]);
});
test("mixed settlement retains failed count and current roster", async () => {
  const h = harness();
  await settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }, { conversationId: "d", status: "failed" }],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["c", "d"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1; 1 destination still need attention."]);
  assert.deepEqual(h.rosters, [["c", "d"]]);
});
test("account switch before settlement prevents every projection", async () => {
  const h = harness();
  h.invalidate();
  let reads = 0;
  await settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => { reads += 1; return ["c"]; },
    applyRoster: h.apply,
  });
  assert.equal(reads, 0);
  assert.deepEqual(h.notices, []);
  assert.deepEqual(h.rosters, []);
});
test("late successful roster response cannot repopulate a new account", async () => {
  const h = harness();
  const roster = deferred<unknown>();
  const settlement = settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: () => roster.promise, applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
  h.invalidate();
  roster.resolve(["foreign-account"]);
  await settlement;
  assert.deepEqual(h.rosters, []);
});
test("late rejected roster request cannot alter a new account or rollback notice", async () => {
  const h = harness();
  const roster = deferred<unknown>();
  const settlement = settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: () => roster.promise, applyRoster: h.apply,
  });
  h.invalidate();
  roster.reject(new Error("old account access denied"));
  await settlement;
  assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(h.rosters, []);
});
