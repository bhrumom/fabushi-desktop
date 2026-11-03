import assert from "node:assert/strict";
import test from "node:test";
import { settleForwardProjection } from "./forward-settlement.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}
function harness(recipients: readonly { id: string; name: string }[] = []) {
  let generation = 3;
  const notices: (string | null)[] = [];
  const rosters: unknown[] = [];
  return {
    notices, rosters,
    isCurrent: (value: number) => value === generation,
    invalidate: () => { generation += 1; },
    apply: (value: unknown) => { rosters.push(value); return recipients; },
    notice: (value: string | null) => notices.push(value),
  };
}
const sent = (conversationId: string) => ({ conversationId, status: "sent" as const });
const failed = (conversationId: string) => ({ conversationId, status: "failed" as const });

test("one settled destination uses its canonical current roster name", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["canonical-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to Alice."]);
  assert.deepEqual(h.rosters, [["canonical-roster"]]);
});

test("two settled destinations use canonical names in the immutable settlement order", async () => {
  const h = harness([{ id: "b", name: "Beta" }, { id: "a", name: "Alpha" }]);
  await settleForwardProjection({
    settlement: [sent("a"), sent("b")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to Alpha and Beta."]);
  assert.deepEqual(h.rosters, [["fresh-roster"]]);
});

test("more than two settled destinations keep the bounded count notice", async () => {
  const h = harness([
    { id: "a", name: "Alpha" }, { id: "b", name: "Beta" }, { id: "c", name: "Gamma" },
  ]);
  await settleForwardProjection({
    settlement: [sent("a"), sent("b"), sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 3 conversations."]);
});

test("mixed settlement names confirmed destinations and retains the failed count", async () => {
  const h = harness([{ id: "c", name: "Alice" }, { id: "d", name: "Bob" }]);
  await settleForwardProjection({
    settlement: [sent("c"), failed("d")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to Alice; 1 destination still needs attention."]);
});

test("missing, blank, duplicate, or unrelated roster identities fail closed to a count without leaking ids", async (t) => {
  const cases = [
    { label: "missing", recipients: [] },
    { label: "blank name", recipients: [{ id: "c", name: "   " }] },
    { label: "duplicate id", recipients: [{ id: "c", name: "Alice" }, { id: "c", name: "Mallory" }] },
    { label: "unrelated id", recipients: [{ id: "other", name: "Secret recipient" }] },
  ];
  for (const entry of cases) {
    await t.test(entry.label, async () => {
      const h = harness(entry.recipients);
      await settleForwardProjection({
        settlement: [sent("human:private-destination")],
        accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
        setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
      });
      assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
      assert.doesNotMatch(h.notices[0] ?? "", /human:private-destination|Secret recipient|Mallory/);
    });
  }
});

test("two destinations fall back to a count if either canonical name is unresolved", async () => {
  const h = harness([{ id: "a", name: "Alpha" }]);
  await settleForwardProjection({
    settlement: [sent("a"), sent("b")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 2 conversations."]);
});

test("current roster read failure retains generic settlement feedback", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => { throw new Error("roster unavailable"); }, applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(h.rosters, []);
});

test("missing roster reader retains generic settlement feedback", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: null, applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(h.rosters, []);
});

test("account switch before settlement prevents every projection", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  h.invalidate();
  let reads = 0;
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => { reads += 1; return ["c"]; },
    applyRoster: h.apply,
  });
  assert.equal(reads, 0);
  assert.deepEqual(h.notices, []);
  assert.deepEqual(h.rosters, []);
});

test("late successful roster response cannot publish a notice or roster for a replacement account", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  const roster = deferred<unknown>();
  const settlement = settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: () => roster.promise, applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, []);
  h.invalidate();
  roster.resolve(["foreign-account"]);
  await settlement;
  assert.deepEqual(h.notices, []);
  assert.deepEqual(h.rosters, []);
});

test("late rejected roster request cannot publish feedback for a replacement account", async () => {
  const h = harness([{ id: "c", name: "Alice" }]);
  const roster = deferred<unknown>();
  const settlement = settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: () => roster.promise, applyRoster: h.apply,
  });
  h.invalidate();
  roster.reject(new Error("old account access denied"));
  await settlement;
  assert.deepEqual(h.notices, []);
  assert.deepEqual(h.rosters, []);
});
