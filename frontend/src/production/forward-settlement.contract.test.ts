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

test("one and two settled destinations use canonical current roster names in settlement order", async () => {
  const one = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: one.isCurrent,
    setNotice: one.notice, readRoster: async () => ["canonical-roster"], applyRoster: one.apply,
  });
  assert.deepEqual(one.notices, ["Forwarded to Alice."]);
  assert.deepEqual(one.rosters, [["canonical-roster"]]);

  const two = harness([{ id: "b", name: "Beta" }, { id: "a", name: "Alpha" }]);
  await settleForwardProjection({
    settlement: [sent("a"), sent("b")],
    accountScopeGeneration: 3, isAccountScopeCurrent: two.isCurrent,
    setNotice: two.notice, readRoster: async () => ["fresh-roster"], applyRoster: two.apply,
  });
  assert.deepEqual(two.notices, ["Forwarded to Alpha and Beta."]);
  assert.deepEqual(two.rosters, [["fresh-roster"]]);
});

test("more than two destinations use a bounded count and mixed settlement names only confirmed targets", async () => {
  const many = harness([
    { id: "a", name: "Alpha" }, { id: "b", name: "Beta" }, { id: "c", name: "Gamma" },
  ]);
  await settleForwardProjection({
    settlement: [sent("a"), sent("b"), sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: many.isCurrent,
    setNotice: many.notice, readRoster: async () => ["fresh-roster"], applyRoster: many.apply,
  });
  assert.deepEqual(many.notices, ["Forwarded to 3 conversations."]);

  const mixed = harness([{ id: "c", name: "Alice" }, { id: "d", name: "Bob" }]);
  await settleForwardProjection({
    settlement: [sent("c"), failed("d")],
    accountScopeGeneration: 3, isAccountScopeCurrent: mixed.isCurrent,
    setNotice: mixed.notice, readRoster: async () => ["fresh-roster"], applyRoster: mixed.apply,
  });
  assert.deepEqual(mixed.notices, ["Forwarded to Alice; 1 destination still needs attention."]);
});

test("missing, blank, duplicate, unrelated, or partially unresolved roster identities fail closed", async () => {
  const cases = [
    { roster: [], settlement: [sent("c")], expected: "Forwarded to 1 conversation." },
    { roster: [{ id: "c", name: "   " }], settlement: [sent("c")], expected: "Forwarded to 1 conversation." },
    { roster: [{ id: "c", name: "Alice" }, { id: "c", name: "Mallory" }], settlement: [sent("c")], expected: "Forwarded to 1 conversation." },
    { roster: [{ id: "other", name: "Secret recipient" }], settlement: [sent("human:private-destination")], expected: "Forwarded to 1 conversation." },
    { roster: [{ id: "a", name: "Alpha" }], settlement: [sent("a"), sent("b")], expected: "Forwarded to 2 conversations." },
    { roster: [{ id: "c", name: "Alice" }], settlement: [sent("c"), sent("c")], expected: "Forwarded to 2 conversations." },
  ];
  for (const entry of cases) {
    const h = harness(entry.roster);
    await settleForwardProjection({
      settlement: entry.settlement,
      accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
      setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
    });
    assert.deepEqual(h.notices, [entry.expected]);
    assert.doesNotMatch(h.notices[0] ?? "", /human:private-destination|Secret recipient|Mallory/);
  }
});

test("roster read, projection, and missing-reader failures retain generic settlement feedback", async () => {
  const readFailure = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: readFailure.isCurrent,
    setNotice: readFailure.notice, readRoster: async () => { throw new Error("roster unavailable"); },
    applyRoster: readFailure.apply,
  });
  assert.deepEqual(readFailure.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(readFailure.rosters, []);

  const noReader = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: noReader.isCurrent,
    setNotice: noReader.notice, readRoster: null, applyRoster: noReader.apply,
  });
  assert.deepEqual(noReader.notices, ["Forwarded to 1 conversation."]);
  assert.deepEqual(noReader.rosters, []);

  const projectionFailure = harness([{ id: "c", name: "Alice" }]);
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: projectionFailure.isCurrent,
    setNotice: projectionFailure.notice, readRoster: async () => ["fresh-roster"],
    applyRoster: () => { throw new Error("projection unavailable"); },
  });
  assert.deepEqual(projectionFailure.notices, ["Forwarded to 1 conversation."]);
});

test("account scope replacement fences pre-read, late successful, and late rejected roster results", async () => {
  const before = harness([{ id: "c", name: "Alice" }]);
  before.invalidate();
  let reads = 0;
  await settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: before.isCurrent,
    setNotice: before.notice, readRoster: async () => { reads += 1; return ["old"]; },
    applyRoster: before.apply,
  });
  assert.equal(reads, 0);
  assert.deepEqual(before.notices, []);
  assert.deepEqual(before.rosters, []);

  const lateSuccess = harness([{ id: "c", name: "Alice" }]);
  const successRoster = deferred<unknown>();
  const success = settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: lateSuccess.isCurrent,
    setNotice: lateSuccess.notice, readRoster: () => successRoster.promise,
    applyRoster: lateSuccess.apply,
  });
  lateSuccess.invalidate();
  successRoster.resolve(["foreign-account"]);
  await success;
  assert.deepEqual(lateSuccess.notices, []);
  assert.deepEqual(lateSuccess.rosters, []);

  const lateFailure = harness([{ id: "c", name: "Alice" }]);
  const failureRoster = deferred<unknown>();
  const failure = settleForwardProjection({
    settlement: [sent("c")],
    accountScopeGeneration: 3, isAccountScopeCurrent: lateFailure.isCurrent,
    setNotice: lateFailure.notice, readRoster: () => failureRoster.promise,
    applyRoster: lateFailure.apply,
  });
  lateFailure.invalidate();
  failureRoster.reject(new Error("old account access denied"));
  await failure;
  assert.deepEqual(lateFailure.notices, []);
  assert.deepEqual(lateFailure.rosters, []);
});

test("unknown runtime settlement statuses fail closed without exposing unconfirmed names", async () => {
  const h = harness([{ id: "c", name: "Alice" }, { id: "x", name: "Private target" }]);
  const unknown = { conversationId: "x", status: "queued" } as unknown as ReturnType<typeof failed>;
  await settleForwardProjection({
    settlement: [sent("c"), unknown],
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["fresh-roster"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to Alice; 1 destination still needs attention."]);
  assert.doesNotMatch(h.notices[0] ?? "", /Private target|queued/);
});
