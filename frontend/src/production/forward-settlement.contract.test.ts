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
    recipientNames: new Map([["c", "Alice"], ["d", "Bob"]]),
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => ["c", "d"], applyRoster: h.apply,
  });
  assert.deepEqual(h.notices, ["Forwarded to 1; 1 destination still needs attention."]);
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
    recipientNames: new Map([["c", "Alice"]]),
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: () => roster.promise, applyRoster: h.apply,
  });
  h.invalidate();
  roster.reject(new Error("old account access denied"));
  await settlement;
  assert.deepEqual(h.notices, ["Forwarded to Alice."]);
  assert.deepEqual(h.rosters, []);
});

test("single and paired confirmations retain their distinct destination names", async () => {
  const cases = [
    { ids: ["c"], names: [["c", "  Alice  "]], notice: "Forwarded to Alice." },
    { ids: ["c", "d"], names: [["c", "Alice"], ["d", "Bob"]], notice: "Forwarded to Alice and Bob." },
    { ids: ["c", "d"], names: [["c", "Alex"], ["d", "Alex"]], notice: "Forwarded to Alex and Alex." },
  ] as const;
  for (const sample of cases) {
    const h = harness();
    await settleForwardProjection({
      settlement: sample.ids.map((conversationId) => ({ conversationId, status: "sent" as const })),
      recipientNames: new Map<string, string>(sample.names),
      accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
      setNotice: h.notice, readRoster: null, applyRoster: h.apply,
    });
    assert.deepEqual(h.notices, [sample.notice]);
    assert.deepEqual(h.rosters, []);
  }
});

test("missing or blank labels and larger fanout retain accurate count feedback", async () => {
  const cases = [
    { ids: ["c"], names: [], notice: "Forwarded to 1 conversation." },
    { ids: ["c"], names: [["c", " \t "]], notice: "Forwarded to 1 conversation." },
    { ids: ["c", "d"], names: [["c", "Alice"]], notice: "Forwarded to 2 conversations." },
    { ids: ["c", "d"], names: [["c", "Alice"], ["d", ""]], notice: "Forwarded to 2 conversations." },
    { ids: ["c", "d", "e"], names: [["c", "Alice"], ["d", "Bob"], ["e", "Carol"]], notice: "Forwarded to 3 conversations." },
  ] as const;
  for (const sample of cases) {
    const h = harness();
    await settleForwardProjection({
      settlement: sample.ids.map((conversationId) => ({ conversationId, status: "sent" as const })),
      recipientNames: new Map<string, string>(sample.names),
      accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
      setNotice: h.notice, readRoster: async () => sample.ids, applyRoster: h.apply,
    });
    assert.deepEqual(h.notices, [sample.notice]);
    assert.deepEqual(h.rosters, [sample.ids]);
  }
});

test("unconfirmed outcomes remain honest and still refresh the current roster", async () => {
  const cases = [
    { sent: [], failed: [], notice: "No forwarding destinations were confirmed." },
    { sent: [], failed: ["c"], notice: "No deliveries confirmed; 1 destination still needs attention." },
    { sent: [], failed: ["c", "d"], notice: "No deliveries confirmed; 2 destinations still need attention." },
    { sent: ["c"], failed: ["d", "e"], notice: "Forwarded to 1; 2 destinations still need attention." },
  ] as const;
  for (const sample of cases) {
    const h = harness();
    await settleForwardProjection({
      settlement: [
        ...sample.sent.map((conversationId) => ({ conversationId, status: "sent" as const })),
        ...sample.failed.map((conversationId) => ({ conversationId, status: "failed" as const })),
      ],
      recipientNames: new Map([["c", "Alice"], ["d", "Bob"], ["e", "Carol"]]),
      accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
      setNotice: h.notice, readRoster: async () => ["current-roster"], applyRoster: h.apply,
    });
    assert.deepEqual(h.notices, [sample.notice]);
    assert.deepEqual(h.rosters, [["current-roster"]]);
  }
});

test("returning to the same account cannot revive a named settlement from its old epoch", async () => {
  const h = harness();
  h.invalidate(); // A -> B
  h.invalidate(); // B -> A, with a new epoch
  let reads = 0;
  await settleForwardProjection({
    settlement: [{ conversationId: "c", status: "sent" }],
    recipientNames: new Map([["c", "Private old-account name"]]),
    accountScopeGeneration: 3, isAccountScopeCurrent: h.isCurrent,
    setNotice: h.notice, readRoster: async () => { reads += 1; return ["old-roster"]; },
    applyRoster: h.apply,
  });
  assert.equal(reads, 0);
  assert.deepEqual(h.notices, []);
  assert.deepEqual(h.rosters, []);
});

test("single self/Saved Messages destination uses Saved Messages default, custom self label, or optional notice suppression", async () => {
  const selfNotifications: string[] = [];
  const defaultSelf = harness();
  await settleForwardProjection({
    settlement: [{ conversationId: "human:self", status: "sent" }],
    selfConversationId: "human:self",
    onSelfForwarded: (id) => selfNotifications.push(`default:${id}`),
    accountScopeGeneration: 3,
    isAccountScopeCurrent: defaultSelf.isCurrent,
    setNotice: defaultSelf.notice,
    readRoster: async () => ["human:self"],
    applyRoster: defaultSelf.apply,
  });
  assert.deepEqual(defaultSelf.notices, ["Forwarded to Saved Messages."]);
  assert.deepEqual(defaultSelf.rosters, [["human:self"]]);

  const customSelf = harness();
  await settleForwardProjection({
    settlement: [{ conversationId: "human:self", status: "sent" }],
    recipientNames: new Map([["human:self", "Personal Vault"]]),
    selfConversationId: "human:self",
    onSelfForwarded: (id) => selfNotifications.push(`custom:${id}`),
    accountScopeGeneration: 3,
    isAccountScopeCurrent: customSelf.isCurrent,
    setNotice: customSelf.notice,
    readRoster: null,
    applyRoster: customSelf.apply,
  });
  assert.deepEqual(customSelf.notices, ["Forwarded to Personal Vault."]);

  const suppressedSelf = harness();
  await settleForwardProjection({
    settlement: [{ conversationId: "human:self", status: "sent" }],
    selfConversationId: "human:self",
    suppressSelfNotice: true,
    onSelfForwarded: (id) => selfNotifications.push(`suppressed:${id}`),
    accountScopeGeneration: 3,
    isAccountScopeCurrent: suppressedSelf.isCurrent,
    setNotice: suppressedSelf.notice,
    readRoster: async () => ["human:self"],
    applyRoster: suppressedSelf.apply,
  });
  assert.deepEqual(suppressedSelf.notices, []);
  assert.deepEqual(suppressedSelf.rosters, [["human:self"]]);
  assert.deepEqual(selfNotifications, [
    "default:human:self",
    "custom:human:self",
    "suppressed:human:self",
  ]);
});
