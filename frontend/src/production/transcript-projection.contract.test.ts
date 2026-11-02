import assert from "node:assert/strict";
import test from "node:test";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../recovered/features/conversation/workspace/model.ts";
import {
  createTranscriptPaginationController,
  mergeOlderTranscriptEntries,
  type TranscriptHistoryPage,
  type TranscriptHistoryRequest,
} from "../recovered/features/conversation/workspace/pagination.ts";
import {
  applyTranscriptReadUpdates,
  currentTranscriptReadBuffers,
  reconcileAuthoritativeTranscriptBaseline,
  reconcileLateInitialTranscriptPage,
  reconcileTranscriptFeedBaseline,
  reconcileTranscriptRefresh,
  reconcileTranscriptTail,
  type TranscriptReadUpdateBuffer,
} from "./transcript-projection.ts";

// NMR-26 / NMR-AC-17: these are the production projection and pagination
// functions consumed by ProductionRenderer, not a duplicate cache algorithm.
const message = (id: string, text = id): TranscriptMessage => ({
  id, kind: "message", role: "user", author: "You", text, timestampMs: 100,
});
const ids = (entries: readonly ConversationTranscriptEntry[]) => entries.map((entry) => entry.id);
function row(entries: readonly ConversationTranscriptEntry[], id: string): TranscriptMessage {
  const entry = entries.find((candidate) => candidate.id === id);
  assert.equal(entry?.kind, "message");
  return entry as TranscriptMessage;
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolveValue, rejectValue) => { resolve = resolveValue; reject = rejectValue; });
  return { promise, resolve, reject };
}

test("NMR-26 late initial tail keeps a same-ID edit, reaction, and newer append", () => {
  const newer = { ...message("m2", "edited"), myReactions: new Set(["👍"]), delivery: "sent" as const };
  // m2 was not loaded when its update arrived. A request-local buffer overlays
  // only returned IDs, so an unrelated old update cannot be appended out of order.
  const page = applyTranscriptReadUpdates([message("m1"), message("m2", "stale")],
    new Map([["m2", newer], ["outside-window", message("outside-window")]]));
  const result = reconcileLateInitialTranscriptPage([message("m3")], page);
  assert.deepEqual(ids(result), ["m1", "m2", "m3"]);
  assert.equal(row(result, "m2").text, "edited");
  assert.deepEqual(row(result, "m2").myReactions, new Set(["👍"]));
  assert.equal(row(result, "m2").delivery, "sent");
  assert.equal(row(reconcileLateInitialTranscriptPage([newer], [message("m2", "stale")]), "m2").text, "edited");
  // update of an unloaded ID -> stale feed baseline -> late initial response.
  // The response buffer has precedence over both snapshot projections.
  const staleBaseline = [message("m0"), message("m1"), message("m2", "baseline-old")];
  const settled = applyTranscriptReadUpdates(
    reconcileLateInitialTranscriptPage(staleBaseline, [message("m1"), message("m2", "page-old")]),
    new Map([["m2", newer]]),
  );
  assert.deepEqual(ids(settled), ["m0", "m1", "m2"]);
  assert.equal(row(settled, "m2").text, "edited");
});

test("NMR-26 duplicate initial IDs and nonce acknowledgement retain one canonical send", () => {
  const pending = { ...message("pending-n", "composed"), clientNonce: "n", delivery: "pending" as const,
    attachments: [{ path: "/resource", name: "saved" }] };
  const accepted = { ...message("remote-8", "accepted"), clientNonce: "n", delivery: "sent" as const };
  const result = reconcileLateInitialTranscriptPage([pending], [accepted, accepted]);
  assert.deepEqual(ids(result), ["remote-8"]);
  assert.equal(row(result, "remote-8").delivery, "sent");
  assert.deepEqual(row(result, "remote-8").attachments, pending.attachments);
  assert.deepEqual(ids(reconcileAuthoritativeTranscriptBaseline([pending, pending], [])), ["pending-n"]);
});

test("NMR-26 feed baseline cannot roll back a same-ID Agent terminal event", () => {
  const terminal = { ...message("agent-final", "complete output"), role: "assistant" as const, isStreaming: false };
  const result = reconcileTranscriptFeedBaseline([terminal], [
    { ...terminal, text: "intermediate", isStreaming: true },
  ], new Set([terminal.id]));
  assert.deepEqual(ids(result), [terminal.id]);
  assert.equal(row(result, terminal.id).text, terminal.text);
  assert.equal(row(result, terminal.id).isStreaming, false);
  assert.equal(row(result, terminal.id).role, "assistant");
});

test("NMR-26 tail merge preserves cached older updates in canonical order", () => {
  const original = [message("m1"), message("m2"), message("m3")];
  const edited = { ...original[0], text: "old row edited" };
  const current = [edited, original[1], original[2], message("m5", "live")];
  const result = reconcileTranscriptTail(current, [message("m3", "refreshed"), message("m4")],
    new Set(["m1", "m5"]), original);
  assert.deepEqual(ids(result), ["m1", "m2", "m3", "m4", "m5"]);
  assert.equal(row(result, "m1").text, "old row edited");
  assert.equal(row(result, "m3").text, "refreshed");
});

test("NMR-26 disjoint tail stays between cached history and events first seen during the read", () => {
  const original = [message("m1"), message("m2")];
  const current = [{ ...original[0], text: "updated old" }, original[1], message("m5")];
  const result = reconcileTranscriptTail(current, [message("m3"), message("m4")],
    new Set(["m1", "m5"]), original);
  assert.deepEqual(ids(result), ["m1", "m2", "m3", "m4", "m5"]);
  assert.equal(row(result, "m1").text, "updated old");
});

test("NMR-26 complete refresh updates unchanged rows but preserves post-request events", () => {
  const original = [message("m1", "old"), message("m2", "old")];
  const current = [original[0], { ...original[1], text: "new event" }, message("m3", "live")];
  const result = reconcileTranscriptRefresh(current, [message("m1", "fresh"), message("m2", "snapshot")], original);
  assert.deepEqual(ids(result), ["m1", "m2", "m3"]);
  assert.equal(row(result, "m1").text, "fresh");
  assert.equal(row(result, "m2").text, "new event");
});

test("NMR-26 older page overlap deduplicates IDs without reverting current row state", () => {
  const current = [message("m2", "current"), message("m3"), message("m3")];
  const result = mergeOlderTranscriptEntries(current, [message("m1"), message("m1"), message("m2", "stale")]);
  assert.deepEqual(ids(result), ["m1", "m2", "m3"]);
  assert.equal(row(result, "m2").text, "current");
});

test("NMR-26 accepted pages commit to the root cache and resume the exclusive cursor on reopen", async () => {
  const requests: TranscriptHistoryRequest[] = [];
  let cached: readonly ConversationTranscriptEntry[] = [message("m5"), message("m6")];
  let nextBeforeSeq: number | undefined = 5;
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", pageLimit: 2,
    fetchPage: async (request) => {
      requests.push(request);
      return request.beforeSeq === 5
        ? { entries: [message("m3"), message("m4")], nextBeforeSeq: 3 }
        : { entries: [message("m1"), message("m2")] };
    },
    onPageApplied: (scope, page) => {
      assert.deepEqual(scope, { accountSlot: "account", agentId: "conversation" });
      cached = mergeOlderTranscriptEntries(cached, page.entries);
      nextBeforeSeq = page.nextBeforeSeq;
    },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq });
  await controller.loadOlder();
  assert.deepEqual(ids(cached), ["m3", "m4", "m5", "m6"]);
  controller.setScope("account", "other");
  controller.installInitialPage({ entries: [message("other")] });
  controller.setScope("account", "conversation");
  controller.installInitialPage({ entries: [], nextBeforeSeq });
  await controller.loadOlder();
  assert.deepEqual(requests, [
    { id: "conversation", limit: 2, beforeSeq: 5 },
    { id: "conversation", limit: 2, beforeSeq: 3 },
  ]);
  assert.deepEqual(ids(cached), ["m1", "m2", "m3", "m4", "m5", "m6"]);
  assert.equal(controller.getSnapshot().hasOlder, false);
  controller.dispose();
});

test("NMR-26 update of an unloaded old row wins over an in-flight page", async () => {
  const read = deferred<TranscriptHistoryPage>();
  const commits: TranscriptHistoryPage[] = [];
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => read.promise,
    onPageApplied: (_scope, page) => { commits.push(page); },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 10 });
  const loading = controller.loadOlder();
  const updated = { ...message("m1", "edited old"), myReactions: new Set(["💙"]) };
  controller.ingestEntryUpdate("other-account", "conversation", message("m1", "foreign"));
  controller.ingestEntryUpdate("account", "other-conversation", message("m1", "foreign"));
  controller.ingestEntryUpdate("account", "conversation", updated);
  read.resolve({ entries: [message("m1", "stale"), message("m2")] });
  await loading;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m1", "m2"]);
  assert.deepEqual(row(controller.getSnapshot().entries, "m1"), updated);
  assert.equal(commits.length, 1);
  assert.deepEqual(commits[0].entries, controller.getSnapshot().entries);
  controller.dispose();
});

test("NMR-26 loaded older row refresh retains order and the next-page cursor", async () => {
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: async () => ({ entries: [message("m1"), message("m2")], nextBeforeSeq: 1 }),
  });
  controller.installInitialPage({ entries: [message("m3")], nextBeforeSeq: 3 });
  await controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", message("m1", "fresh"));
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m1", "m2", "m3"]);
  assert.equal(row(controller.getSnapshot().entries, "m1").text, "fresh");
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 1 });
  controller.dispose();
});

test("NMR-26 account and conversation changes fence stale rows and request completion", async () => {
  const first = deferred<TranscriptHistoryPage>();
  const second = deferred<TranscriptHistoryPage>();
  const commits: string[] = [];
  let calls = 0;
  const controller = createTranscriptPaginationController({
    accountSlot: "account-a", agentId: "conversation-a",
    fetchPage: () => ++calls === 1 ? first.promise : second.promise,
    onPageApplied: (scope) => { commits.push(`${scope.accountSlot}:${scope.agentId}`); },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 10 });
  const stale = controller.loadOlder();
  await Promise.resolve();
  controller.setScope("account-b", "conversation-b");
  controller.installInitialPage({ entries: [message("b2")], nextBeforeSeq: 2 });
  const fresh = controller.loadOlder();
  first.resolve({ entries: [message("foreign-a1")] });
  await stale;
  assert.equal(controller.getSnapshot().isLoadingOlder, true);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["b2"]);
  second.resolve({ entries: [message("b1")] });
  await fresh;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["b1", "b2"]);
  assert.deepEqual(commits, ["account-b:conversation-b"]);
  controller.dispose();

  // Initial/full-read buffers use the same scope fence as their commit path.
  let generation = 1;
  const expired: TranscriptReadUpdateBuffer = {
    accountSlot: "account", agentId: "conversation", isCurrent: () => generation === 1,
    updates: new Map([["m2", message("m2", "old event")]]),
  };
  const foreign: TranscriptReadUpdateBuffer = {
    accountSlot: "other-account", agentId: "conversation", isCurrent: () => true,
    updates: new Map([["m2", message("m2", "foreign event")]]),
  };
  const otherConversation: TranscriptReadUpdateBuffer = {
    accountSlot: "account", agentId: "other-conversation", isCurrent: () => true,
    updates: new Map([["m2", message("m2", "other conversation event")]]),
  };
  generation = 2;
  const freshBuffer: TranscriptReadUpdateBuffer = {
    accountSlot: "account", agentId: "conversation", isCurrent: () => generation === 2,
    updates: new Map(),
  };
  const reads = new Set([expired, foreign, otherConversation, freshBuffer]);
  const active = currentTranscriptReadBuffers(reads, "account", "conversation");
  assert.deepEqual(active, [freshBuffer]);
  assert.equal(reads.has(expired), false);
  const snapshot = [message("m2", "new snapshot")];
  assert.equal(row(applyTranscriptReadUpdates(snapshot, active[0].updates), "m2").text, "new snapshot");
  // Both onUpdated and onAppended record the latest event for an in-flight read.
  for (const read of active) read.updates.set("m2", message("m2", "updated"));
  for (const read of currentTranscriptReadBuffers(reads, "account", "conversation")) {
    read.updates.set("m2", message("m2", "later append"));
  }
  assert.equal(row(applyTranscriptReadUpdates(snapshot, freshBuffer.updates), "m2").text, "later append");
});

test("NMR-26 clear/reset prevents a late page from repopulating the transcript", async () => {
  const read = deferred<TranscriptHistoryPage>();
  let commits = 0;
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => read.promise,
    onPageApplied: () => { commits += 1; },
  });
  controller.installInitialPage({ entries: [message("m2")], nextBeforeSeq: 2 });
  const loading = controller.loadOlder();
  await Promise.resolve();
  controller.reset();
  read.resolve({ entries: [message("m1")] });
  await loading;
  assert.deepEqual(controller.getSnapshot().entries, []);
  assert.equal(controller.getSnapshot().isLoadingOlder, false);
  assert.equal(commits, 0);
  controller.dispose();
});

test("NMR-26 authoritative complete refresh supersedes an older request", async () => {
  const read = deferred<TranscriptHistoryPage>();
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => read.promise,
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 10 });
  const loading = controller.loadOlder();
  await Promise.resolve();
  controller.installInitialPage({ entries: [message("fresh")] });
  read.resolve({ entries: [message("stale")] });
  await loading;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["fresh"]);
  assert.equal(controller.getSnapshot().hasOlder, false);
  controller.dispose();
});

test("NMR-26 failed history load preserves rows and retries the same cursor once", async () => {
  let fail = true;
  const requests: number[] = [];
  const failure = new Error("offline");
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: async ({ beforeSeq }) => {
      requests.push(beforeSeq);
      if (fail) throw failure;
      return { entries: [message("m1")] };
    },
  });
  controller.installInitialPage({ entries: [message("m2")], nextBeforeSeq: 2 });
  const loading = controller.loadOlder();
  assert.equal(controller.loadOlder(), loading);
  await loading;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m2"]);
  assert.equal(controller.getSnapshot().olderFailure, failure);
  assert.equal(controller.getSnapshot().isLoadingOlder, false);
  fail = false;
  await controller.loadOlder();
  assert.deepEqual(requests, [2, 2]);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m1", "m2"]);
  assert.equal(controller.getSnapshot().olderFailure, null);
  controller.dispose();
});

test("NMR-26 synchronous fetch failure cannot leave pagination permanently busy", async () => {
  let fail = true;
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: () => {
      if (fail) throw new Error("synchronous transport refusal");
      return Promise.resolve({ entries: [message("m1")] });
    },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 2 });
  await controller.loadOlder();
  assert.equal(controller.getSnapshot().isLoadingOlder, false);
  assert.ok(controller.getSnapshot().olderFailure instanceof Error);
  fail = false;
  await controller.loadOlder();
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m1"]);
  controller.dispose();
});

test("NMR-26 viewport anchoring and disposal reject stale asynchronous effects", async () => {
  const read = deferred<TranscriptHistoryPage>();
  let committed = 0;
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => read.promise,
    onPageApplied: () => { committed += 1; },
  });
  const viewport = { scrollTop: 30, scrollHeight: 100 };
  controller.bindViewport(viewport);
  controller.installInitialPage({ entries: [message("m2")], nextBeforeSeq: 2 });
  const loading = controller.loadOlder();
  read.resolve({ entries: [message("m1")] });
  await loading;
  viewport.scrollHeight = 160;
  assert.equal(controller.onEntriesCommitted(), true);
  assert.equal(viewport.scrollTop, 90);
  assert.equal(controller.onEntriesCommitted(), false);
  controller.installInitialPage({ entries: [], nextBeforeSeq: 1 });
  const pending = controller.loadOlder();
  controller.dispose();
  await pending;
  assert.equal(committed, 1);
  assert.equal(controller.onEntriesCommitted(), false);
  assert.equal(controller.installInitialPage({ entries: [message("late")] }), false);
});
