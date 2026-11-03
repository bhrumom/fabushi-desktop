import assert from "node:assert/strict";
import test from "node:test";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../recovered/features/conversation/workspace/model.ts";
import { isHumanConversationAgent, projectHumanConversation, projectTranscriptEntry } from "./model.ts";
import {
  createTranscriptPaginationController,
  mergeOlderTranscriptEntries,
  type TranscriptHistoryPage,
  type TranscriptHistoryRequest,
} from "../recovered/features/conversation/workspace/pagination.ts";
import {
  applyTranscriptReadUpdates,
  createHumanReactionSettlement,
  currentTranscriptReadBuffers,
  mergeAuthoritativeTranscriptMessage,
  recordTranscriptReadUpdate,
  reconcileTranscriptMutation,
  reconcileAuthoritativeTranscriptBaseline,
  reconcileLateInitialTranscriptPage,
  reconcileTranscriptFeedBaseline,
  reconcileTranscriptRefresh,
  reconcileTranscriptTail,
  transcriptEntriesCorrelate,
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

test("FBCP Human roster projection preserves the persisted participantIds string-array contract", () => {
  const source = {
    id: "human:alice",
    kind: "human",
    title: "Alice",
    participantIds: ["1", "2"],
    updatedAt: 100,
  };
  const projected = projectHumanConversation(source, 0);
  assert.ok(projected);
  assert.equal(isHumanConversationAgent(projected), true);
  assert.deepEqual(projected.memberIds, ["1", "2"]);
  assert.deepEqual(projected.conversationPartnerIds, ["1", "2"]);
  assert.deepEqual(projected.raw.participantIds, ["1", "2"]);

  for (const participantIds of [
    '["1","2"]',
    ["1", 2],
    ["1", "1"],
    ["1", ""],
    ["1"],
    ["1", "2", "3"],
    null,
  ]) {
    assert.equal(projectHumanConversation({ ...source, participantIds }, 0), null);
  }
});

test("NMR-49 dispatching Human text and attachments stay pending until settlement", () => {
  const shapes = [
    { kind: "message", role: "user", content: "Awaiting acknowledgement",
      attachments: [{ path: "/saved/parent.txt", name: "parent.txt", byteSize: 12 }] },
    { kind: "attachment", path: "/saved/parent.txt", fileName: "parent.txt", byteSize: 12 },
    { kind: "user-attachment", file_path: "/saved/parent.txt", file_name: "parent.txt", byteSize: 12 },
  ];
  for (const shape of shapes) {
    for (const field of ["delivery", "status"] as const) {
      const input = {
        ...shape, id: "human-message:pending-parent", authorKind: "human", authorId: "self",
        clientNonce: "pending-parent", timestampMs: 100, replyToId: "earlier-parent", replyTo: "earlier-parent",
        [field]: "dispatching",
      };
      const original = structuredClone(input);
      const projected = projectTranscriptEntry(input, 0, "Fabushi", "human-conversation");
      assert.ok(projected?.kind === "message");
      assert.equal(projected.delivery, "pending", `${shape.kind} ${field}`);
      assert.equal(projected.id, input.id);
      assert.equal(projected.role, "user");
      assert.equal(projected.clientNonce, input.clientNonce);
      assert.equal(projected.replyToId, "earlier-parent");
      assert.equal(projected.text, shape.kind === "message" ? "Awaiting acknowledgement" : "");
      assert.deepEqual(projected.attachments?.map(({ path, name, size }) => ({ path, name, size })), [
        { path: "/saved/parent.txt", name: "parent.txt", size: 12 },
      ]);
      assert.deepEqual(input, original);

      const settled = projectTranscriptEntry({ ...input, [field]: "sent" }, 0, "Fabushi", "human-conversation");
      assert.ok(settled?.kind === "message");
      assert.equal(settled.delivery, "sent");
      assert.equal(settled.clientNonce, projected.clientNonce);
      assert.equal(settled.replyToId, projected.replyToId);
      assert.equal(settled.text, projected.text);
      assert.deepEqual(settled.attachments, projected.attachments);
    }
  }
});

test("NMR-49 existing Human and Agent delivery projections remain unchanged", () => {
  const deliveryCases: { fields: Record<string, unknown>; expected: TranscriptMessage["delivery"] }[] = [
    { fields: {}, expected: "sent" },
    { fields: { status: "running" }, expected: "sent" },
    { fields: { pending: true }, expected: "pending" },
    { fields: { isPending: true }, expected: "pending" },
    { fields: { failed: true }, expected: "failed" },
    { fields: { isFailed: true }, expected: "failed" },
    { fields: { delivery: "sent", status: "dispatching", pending: true }, expected: "sent" },
  ];
  for (const field of ["delivery", "status"] as const) {
    for (const state of ["sent", "pending", "queued", "failed"] as const) {
      deliveryCases.push({ fields: { [field]: state }, expected: state });
    }
  }
  for (const role of ["user", "assistant"] as const) {
    for (const { fields, expected } of deliveryCases) {
      const projected = projectTranscriptEntry({
        id: `${role}-entry`, kind: "message", role, content: "Existing content", authorName: "Existing Agent",
        timestampMs: 100, clientNonce: `${role}-nonce`, replyToId: "existing-parent", ...fields,
      }, 0, "Fallback Agent", "existing-conversation");
      assert.ok(projected?.kind === "message");
      assert.equal(projected.delivery, expected, `${role} ${JSON.stringify(fields)}`);
      assert.equal(projected.role, role);
      assert.equal(projected.author, role === "assistant" ? "Existing Agent" : "You");
      assert.equal(projected.text, "Existing content");
      assert.equal(projected.clientNonce, `${role}-nonce`);
      assert.equal(projected.replyToId, "existing-parent");
    }
  }
});

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

test("Human transcript correlation isolates authors while retaining Host and optimistic send settlement", () => {
  const pending: TranscriptMessage = {
    ...message("human-message:shared", "local draft"), authorKind: "human", authorId: "local",
    clientNonce: "shared", delivery: "pending", attachments: [{ path: "/local", name: "local attachment" }],
  };
  const peer: TranscriptMessage = {
    ...message("human-server-message:52", "peer text"), authorKind: "human", authorId: "peer",
    clientNonce: "shared", remoteMessageId: "52", delivery: "sent",
  };
  const accepted: TranscriptMessage = {
    ...message("human-server-message:51", "accepted local"), authorKind: "human", authorId: "local",
    clientNonce: "shared", remoteMessageId: "51", delivery: "sent",
  };
  assert.equal(transcriptEntriesCorrelate(pending, peer), false);
  assert.equal(transcriptEntriesCorrelate(pending, accepted), true);
  assert.equal(transcriptEntriesCorrelate({ ...pending, authorId: "" }, accepted), false);
  assert.equal(transcriptEntriesCorrelate({ ...message("unrelated-agent"), clientNonce: "shared" }, accepted), false);
  const optimistic: TranscriptMessage = { ...message("pending-shared"), clientNonce: "shared", delivery: "pending" };
  assert.equal(transcriptEntriesCorrelate(optimistic, accepted), true);
  const baseline = reconcileAuthoritativeTranscriptBaseline([pending], [peer, accepted]);
  assert.deepEqual(ids(baseline), [peer.id, accepted.id]);
  assert.equal(row(baseline, peer.id).text, peer.text);
  assert.equal(row(baseline, peer.id).attachments, undefined);
  assert.deepEqual(row(baseline, accepted.id).attachments, pending.attachments);
  assert.deepEqual(ids(reconcileLateInitialTranscriptPage([optimistic], [accepted])), [accepted.id]);
  assert.deepEqual(ids(reconcileLateInitialTranscriptPage([pending], [accepted])), [accepted.id]);
  const stale = reconcileAuthoritativeTranscriptBaseline([accepted], [pending, accepted]);
  assert.deepEqual(ids(stale), [accepted.id]);
  assert.equal(row(stale, accepted.id).delivery, "sent");
  assert.equal(row(stale, accepted.id).text, accepted.text);
  const agentPending = { ...message("pending-agent"), clientNonce: "agent" };
  const agentAccepted = { ...message("agent-accepted"), clientNonce: "agent" };
  assert.equal(transcriptEntriesCorrelate(agentPending, agentAccepted), true);
});

test("Human older pages cannot resurrect a settled pending row in the root cache", async () => {
  const pending: TranscriptMessage = {
    ...message("human-message:shared"), authorKind: "human", authorId: "local",
    clientNonce: "shared", delivery: "pending",
  };
  const accepted: TranscriptMessage = {
    ...message("human-server-message:51"), authorKind: "human", authorId: "local",
    clientNonce: "shared", remoteMessageId: "51", delivery: "sent",
  };
  const peer: TranscriptMessage = {
    ...message("human-server-message:52"), authorKind: "human", authorId: "peer",
    clientNonce: "shared", remoteMessageId: "52", delivery: "sent",
  };
  const unrelatedPending: TranscriptMessage = {
    ...pending, id: "human-message:unmatched", clientNonce: "unmatched",
  };
  const agentPending = { ...message("pending-shared"), clientNonce: "shared", delivery: "pending" as const };
  const agentAccepted = { ...message("agent-accepted"), clientNonce: "shared", delivery: "sent" as const };
  let cached: readonly ConversationTranscriptEntry[] = [accepted, agentAccepted];
  let cachedCursor: string | undefined;
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "human-room",
    fetchPage: async () => ({
      entries: [peer, pending, unrelatedPending, agentPending],
      nextBeforeSeq: 2, nextBeforeCursor: "opaque-walk:2",
    }),
    onPageApplied: (_scope, page) => {
      cached = mergeOlderTranscriptEntries(cached, page.entries);
      cachedCursor = page.nextBeforeCursor;
    },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 10, nextBeforeCursor: "opaque-walk:10" });
  await controller.loadOlder();
  const expected = [peer.id, unrelatedPending.id, agentPending.id, accepted.id, agentAccepted.id];
  assert.deepEqual(ids(cached), expected);
  assert.deepEqual(ids(mergeOlderTranscriptEntries(cached, controller.getSnapshot().entries)), expected);
  assert.equal(cached.some((entry) => entry.id === pending.id), false);
  assert.equal(cachedCursor, "opaque-walk:2");
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 2, beforeCursor: "opaque-walk:2" });
  controller.dispose();
});

test("NMR-26 an intermediate row commit cannot consume an older-page viewport anchor", async () => {
  const read = deferred<TranscriptHistoryPage>();
  const requests: TranscriptHistoryRequest[] = [];
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: (request) => {
      requests.push(request);
      return request.beforeSeq === 5
        ? Promise.resolve({ entries: [message("m3")], nextBeforeSeq: 3, nextBeforeCursor: "opaque-walk:3" })
        : read.promise;
    },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 5, nextBeforeCursor: "opaque-walk:5" });
  await controller.loadOlder();
  const viewport = { scrollTop: 40, scrollHeight: 400 };
  controller.bindViewport(viewport);
  const loading = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", message("m3", "updated"));
  assert.equal(controller.getSnapshot().isLoadingOlder, true);
  // The renderer commits this row update while the older read is still pending.
  assert.equal(controller.onEntriesCommitted(), false);
  assert.equal(viewport.scrollTop, 40);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 3, beforeCursor: "opaque-walk:3" });
  read.resolve({ entries: [message("m1"), message("m2"), message("m3", "stale overlap")], nextBeforeSeq: 1, nextBeforeCursor: "opaque-walk:1" });
  await loading;
  assert.deepEqual(requests, [
    { id: "conversation", limit: 200, beforeSeq: 5, beforeCursor: "opaque-walk:5" },
    { id: "conversation", limit: 200, beforeSeq: 3, beforeCursor: "opaque-walk:3" },
  ]);
  viewport.scrollHeight = 480;
  assert.equal(controller.onEntriesCommitted(), true);
  assert.equal(viewport.scrollTop, 120);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["m1", "m2", "m3"]);
  assert.equal(row(controller.getSnapshot().entries, "m3").text, "updated");
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 1, beforeCursor: "opaque-walk:1" });
  assert.equal(controller.onEntriesCommitted(), false);
  controller.dispose();
});

const reactionTarget = { conversationId: "human-a", entryId: "older-entry" };
const unexpectedReactionFailure = (error: unknown): never => { throw error; };

test("NMR-26 a late first reaction response cannot replace a later mutation settlement", async () => {
  const controller = createHumanReactionSettlement(() => 1);
  const first = deferred<string>();
  const second = deferred<string>();
  const applied: string[] = [];
  const a = controller.run(reactionTarget, () => first.promise, (value) => { applied.push(value); }, unexpectedReactionFailure);
  const b = controller.run(reactionTarget, () => second.promise, (value) => { applied.push(value); }, unexpectedReactionFailure);
  second.resolve("new reaction state");
  await b;
  first.resolve("old reaction state");
  await a;
  assert.deepEqual(applied, ["new reaction state"]);
  controller.dispose();
});

test("NMR-26 an authoritative event wins over an outstanding reaction RPC response", async () => {
  const controller = createHumanReactionSettlement(() => 1);
  const pending = deferred<string>();
  const applied: string[] = [];
  const request = controller.run(reactionTarget, () => pending.promise, (value) => { applied.push(value); }, unexpectedReactionFailure);
  controller.observeEntry(reactionTarget);
  applied.push("authoritative event");
  pending.resolve("obsolete response");
  await request;
  assert.deepEqual(applied, ["authoritative event"]);
  controller.dispose();
});

test("NMR-26 account generation fences a reaction across logout and return to the same identity", async () => {
  let generation = 1;
  const controller = createHumanReactionSettlement(() => generation);
  const pending = deferred<string>();
  const applied: string[] = [];
  const request = controller.run(reactionTarget, () => pending.promise, (value) => { applied.push(value); }, unexpectedReactionFailure);
  generation = 2;
  generation = 3;
  pending.resolve("departed scope");
  await request;
  await controller.run(reactionTarget, async () => "current scope", (value) => { applied.push(value); }, unexpectedReactionFailure);
  assert.deepEqual(applied, ["current scope"]);
  controller.dispose();
});

test("NMR-26 independent conversation and message reactions retain their completion scope", async () => {
  const controller = createHumanReactionSettlement(() => 1);
  const pending = deferred<string>();
  const applied: string[] = [];
  const first = controller.run(reactionTarget, () => pending.promise, (value) => { applied.push(value); }, unexpectedReactionFailure);
  await controller.run({ conversationId: "human-b", entryId: reactionTarget.entryId }, async () => "other conversation", (value) => { applied.push(value); }, unexpectedReactionFailure);
  await controller.run({ conversationId: reactionTarget.conversationId, entryId: "other-entry" }, async () => "other entry", (value) => { applied.push(value); }, unexpectedReactionFailure);
  pending.resolve("original entry");
  await first;
  assert.deepEqual(applied, ["other conversation", "other entry", "original entry"]);
  controller.dispose();
});

test("NMR-26 current reaction failures report once and obsolete failures cannot replace recovery feedback", async () => {
  const controller = createHumanReactionSettlement(() => 1);
  const errors: unknown[] = [];
  const applied: string[] = [];
  const failure = new Error("offline");
  await controller.run(reactionTarget, async () => { throw failure; }, () => assert.fail("failed request applied"), (error) => { errors.push(error); });
  const invalid = new Error("invalid settlement");
  await controller.run(reactionTarget, async () => "invalid", () => { throw invalid; }, (error) => { errors.push(error); });
  const pending = deferred<string>();
  const old = controller.run(reactionTarget, () => pending.promise, (value) => { applied.push(value); }, (error) => { errors.push(error); });
  await controller.run(reactionTarget, async () => "recovered", (value) => { applied.push(value); }, unexpectedReactionFailure);
  pending.reject(new Error("obsolete failure"));
  await old;
  assert.deepEqual(errors, [failure, invalid]);
  assert.deepEqual(applied, ["recovered"]);
  controller.dispose();
});

test("NMR-26 deferred reaction state and error updates recheck events, clear and account generation", async () => {
  let generation = 1;
  const controller = createHumanReactionSettlement(() => generation);
  const deferredApplications: Array<() => void> = [];
  const applied: string[] = [];
  const enqueue = (value: string, isCurrent: () => boolean) => {
    deferredApplications.push(() => { if (isCurrent()) applied.push(value); });
  };
  await controller.run(reactionTarget, async () => "old event", enqueue, unexpectedReactionFailure);
  controller.observeEntry(reactionTarget);
  deferredApplications.shift()!();
  await controller.run(reactionTarget, async () => "cleared conversation", enqueue, unexpectedReactionFailure);
  controller.clearConversation(reactionTarget.conversationId);
  deferredApplications.shift()!();
  await controller.run(reactionTarget, async () => "old account", enqueue, unexpectedReactionFailure);
  generation += 1;
  deferredApplications.shift()!();
  await controller.run(reactionTarget, async () => { throw new Error("old account error"); }, enqueue,
    (_error, isCurrent) => enqueue("stale error notice", isCurrent));
  generation += 1;
  deferredApplications.shift()!();
  await controller.run(reactionTarget, async () => "current state", enqueue, unexpectedReactionFailure);
  deferredApplications.shift()!();
  assert.deepEqual(applied, ["current state"]);
  controller.dispose();
});

test("NMR-26 Human reaction settlement and a later event refresh an older row without moving it", async () => {
  const pagination = createTranscriptPaginationController({
    accountSlot: "account", agentId: reactionTarget.conversationId,
    fetchPage: async ({ beforeSeq, beforeCursor }) => {
      assert.equal(beforeSeq, 2);
      assert.equal(beforeCursor, "opaque-walk:2");
      return { entries: [message(reactionTarget.entryId)], nextBeforeSeq: 1, nextBeforeCursor: "opaque-walk:1" };
    },
  });
  pagination.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 2, nextBeforeCursor: "opaque-walk:2" });
  await pagination.loadOlder();
  const cursor = pagination.getSnapshot().cursor;
  const controller = createHumanReactionSettlement(() => 1);
  const reacted = { ...message(reactionTarget.entryId), reactions: [{ emoji: "👍", by: "me" }] };
  const apply = (value: TranscriptMessage) => pagination.ingestEntryUpdate("account", reactionTarget.conversationId, value);
  await controller.run(reactionTarget, async () => reacted, apply, unexpectedReactionFailure);
  assert.deepEqual(pagination.getSnapshot().entries, [reacted, message("tail")]);
  const late = deferred<TranscriptMessage>();
  const pending = controller.run(reactionTarget, () => late.promise, apply, unexpectedReactionFailure);
  const refreshed = { ...message(reactionTarget.entryId), reactions: [{ emoji: "❤️", by: "me" }] };
  controller.observeEntry(reactionTarget);
  pagination.ingestEntryUpdate("account", reactionTarget.conversationId, refreshed);
  late.resolve(reacted);
  await pending;
  assert.deepEqual(pagination.getSnapshot().entries, [refreshed, message("tail")]);
  assert.equal(pagination.getSnapshot().cursor, cursor);
  assert.deepEqual(pagination.getSnapshot().cursor, { kind: "more", beforeSeq: 1, beforeCursor: "opaque-walk:1" });
  controller.dispose();
  pagination.dispose();
});

test("NMR-26 disposed reaction completion cannot mutate the renderer or show a stale error", async () => {
  const controller = createHumanReactionSettlement(() => 1);
  const pending = deferred<string>();
  const request = controller.run(reactionTarget, () => pending.promise, () => assert.fail("disposed result applied"), unexpectedReactionFailure);
  controller.dispose();
  pending.reject(new Error("disconnected after dispose"));
  await request;
  await controller.run(reactionTarget, async () => { assert.fail("disposed request sent"); }, () => assert.fail("disposed result applied"), unexpectedReactionFailure);
});

test("NMR-26 Human empty snapshots clear content and resources while Agent partial echoes retain them", () => {
  const human: TranscriptMessage = {
    ...message("human-1", "old body"), authorKind: "human", authorId: "local",
    attachments: [{ path: "/old-resource", name: "old.txt" }], richText: "old-rich-text", clientNonce: "old",
  };
  const partial = mergeAuthoritativeTranscriptMessage(human, {
    ...message("human-1", "changed body"), authorKind: "human", authorId: "local",
  });
  assert.deepEqual(partial.attachments, human.attachments);
  assert.equal(partial.richText, human.richText);
  assert.equal(partial.clientNonce, human.clientNonce);
  const cleared = mergeAuthoritativeTranscriptMessage(partial, {
    ...message("human-1", ""), authorKind: "human", authorId: "local", attachments: [], richText: "", reactions: [],
  });
  assert.equal(cleared.text, "");
  assert.deepEqual(cleared.attachments, []);
  assert.equal(cleared.richText, "");
  assert.equal(cleared.clientNonce, human.clientNonce);
  const agentTurn = { ...message("pending-agent", "Agent prompt"), attachments: human.attachments,
    delivery: "pending" as const, clientNonce: "agent-nonce" };
  const echo = mergeAuthoritativeTranscriptMessage(agentTurn, {
    ...message("attachment-echo", ""), attachments: [{ path: "/new-resource", name: "new.txt" }], delivery: "sent",
  }, { preserveCurrentId: true });
  assert.equal(echo.id, agentTurn.id);
  assert.equal(echo.text, agentTurn.text);
  assert.equal(echo.delivery, "pending");
  assert.equal(echo.clientNonce, "agent-nonce");
  assert.deepEqual(echo.attachments?.map((attachment) => attachment.path), ["/old-resource", "/new-resource"]);
});

test("NMR-26 peer and missing-author nonces cannot reconcile a local Human placeholder", () => {
  const pending: TranscriptMessage = { ...message("pending-n", "local draft"), authorKind: "human", authorId: "local", clientNonce: "n", delivery: "pending" };
  const peer = { ...pending, id: "peer-1", authorId: "peer", text: "peer body" };
  const missingAuthor = { ...pending, id: "unknown", authorId: undefined };
  assert.equal(transcriptEntriesCorrelate(pending, peer), false);
  assert.equal(transcriptEntriesCorrelate(pending, missingAuthor), false);
  assert.equal(transcriptEntriesCorrelate(pending, { ...pending, id: "assistant", role: "assistant" }), false);
  assert.deepEqual(ids(reconcileAuthoritativeTranscriptBaseline([pending], [peer])), ["peer-1", "pending-n"]);
  assert.deepEqual(ids(reconcileTranscriptMutation([pending], peer, { previousId: pending.id, appendIfMissing: true })), ["pending-n", "peer-1"]);
  const authorless = { ...message("authorless"), clientNonce: "n" };
  assert.deepEqual(ids(reconcileTranscriptMutation([pending], authorless, { previousId: pending.id })), [pending.id]);
  assert.deepEqual(applyTranscriptReadUpdates([pending], new Map([[pending.id, peer]])), [pending]);
  assert.deepEqual(applyTranscriptReadUpdates([pending], new Map([[pending.id, authorless]])), [pending]);
  const local = { ...pending, id: "local-1", delivery: "sent" as const };
  assert.deepEqual(ids(reconcileTranscriptMutation([pending, peer], local, { previousId: pending.id })), ["local-1", "peer-1"]);
});

test("NMR-26 before-ID canonicalization and a delayed receipt preserve one newer settled Human row", () => {
  const pending: TranscriptMessage = { ...message("pending-n", "composed"), authorKind: "human", authorId: "local", clientNonce: "n", delivery: "pending" };
  const accepted = { ...pending, id: "remote-1", delivery: "sent" as const };
  const afterEvent = reconcileTranscriptMutation([pending], accepted, { previousId: pending.id });
  assert.deepEqual(ids(afterEvent), [accepted.id]);
  const newer = { ...accepted, text: "", attachments: [], reactions: [{ emoji: "💙", by: "me" }] };
  const withNewer = reconcileTranscriptMutation(afterEvent, newer);
  const settled = reconcileTranscriptMutation([...withNewer, pending], accepted, { previousId: pending.id, preserveSettled: true });
  assert.deepEqual(ids(settled), [accepted.id]);
  assert.equal(row(settled, accepted.id).text, "");
  assert.deepEqual(row(settled, accepted.id).reactions, newer.reactions);
  const latePage = applyTranscriptReadUpdates([pending], new Map([[pending.id, accepted], [accepted.id, newer]]));
  assert.deepEqual(ids(latePage), [accepted.id]);
  assert.equal(row(latePage, accepted.id).text, "");
  assert.deepEqual(row(latePage, accepted.id).reactions, newer.reactions);
  const updates = new Map<string, ConversationTranscriptEntry>();
  recordTranscriptReadUpdate(updates, { ...newer, richText: "" }, pending.id);
  recordTranscriptReadUpdate(updates, { ...accepted, text: "later partial" });
  const clearThenPartial = applyTranscriptReadUpdates([{ ...pending, attachments: [{ path: "/old", name: "old.txt" }], richText: "old-rich-text" }], updates);
  assert.deepEqual(ids(clearThenPartial), [accepted.id]);
  assert.equal(row(clearThenPartial, accepted.id).text, "later partial");
  assert.deepEqual(row(clearThenPartial, accepted.id).attachments, []);
  assert.deepEqual(row(clearThenPartial, accepted.id).reactions, newer.reactions);
  assert.equal(row(clearThenPartial, accepted.id).richText, "");
  recordTranscriptReadUpdate(updates, { ...newer, id: "peer", authorId: "peer" }, pending.id);
  assert.equal(row(applyTranscriptReadUpdates([pending], updates), accepted.id).text, "later partial");
  const noNonce = { ...accepted, clientNonce: undefined, id: "rewritten-id" };
  assert.deepEqual(ids(reconcileTranscriptMutation(afterEvent, noNonce, { previousId: accepted.id })), [noNonce.id]);
});

test("NMR-26 overlapping older windows preserve canonical anchors and loaded/in-flight identity updates", async () => {
  const current = [message("m2", "newer"), message("m4"), message("m5")];
  const merged = mergeOlderTranscriptEntries(current, [message("m1"), message("m2", "stale"), message("m3"), message("m4")]);
  assert.deepEqual(ids(merged), ["m1", "m2", "m3", "m4", "m5"]);
  assert.equal(row(merged, "m2").text, "newer");
  const old: TranscriptMessage = { ...message("old"), authorKind: "human", authorId: "local" };
  const remapped = { ...old, id: "canonical", text: "edited", attachments: [] };
  const page = deferred<TranscriptHistoryPage>();
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => page.promise,
  });
  controller.installInitialPage({ entries: [old, message("tail")], nextBeforeSeq: 3 });
  controller.ingestEntryUpdate("account", "conversation", remapped, old.id);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["canonical", "tail"]);
  controller.ingestEntryUpdate("account", "conversation", { ...remapped, id: "peer", authorId: "peer" }, remapped.id);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["canonical", "tail"]);
  controller.installInitialPage({ entries: [], nextBeforeSeq: 3 });
  const loading = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", { ...remapped, id: "peer", authorId: "peer" }, old.id);
  controller.ingestEntryUpdate("account", "conversation", remapped, old.id);
  page.resolve({ entries: [old, message("tail")] });
  await loading;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["canonical", "tail"]);
  assert.equal(row(controller.getSnapshot().entries, "canonical").text, "edited");
  controller.dispose();
});

test("NMR-26 failed older pages coalesce repeated Human clears and partial updates before replay", async () => {
  const first = deferred<TranscriptHistoryPage>();
  const retry = deferred<TranscriptHistoryPage>();
  const requests: TranscriptHistoryRequest[] = [];
  const original: TranscriptMessage = {
    ...message("older", "stale body"), authorKind: "human", authorId: "local", clientNonce: "older-nonce",
    attachments: [{ path: "/old-resource", name: "old.txt" }], richText: "old-rich-text",
    reactions: [{ emoji: "👍", by: "me" }],
  };
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: (request) => {
      requests.push(request);
      return requests.length === 1 ? first.promise : retry.promise;
    },
  });
  controller.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 5, nextBeforeCursor: "opaque-walk:5" });
  const loading = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", {
    ...original, text: "", attachments: [], richText: "", reactions: [],
  });
  const failure = new Error("older page unavailable");
  first.reject(failure);
  await loading;
  assert.equal(controller.getSnapshot().olderFailure, failure);
  let measuringReplay = false;
  let supersededPayloadReads = 0;
  for (let index = 0; index < 256; index += 1) {
    controller.ingestEntryUpdate("account", "conversation", {
      ...message(original.id), authorKind: "human", authorId: "local",
      get text() {
        // Observe replay work directly, without a timing threshold or access
        // to private controller storage. Ingestion may read these payloads.
        if (measuringReplay) supersededPayloadReads += 1;
        return "edit-" + index;
      },
    });
  }
  assert.deepEqual(ids(controller.getSnapshot().entries), ["tail"]);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 5, beforeCursor: "opaque-walk:5" });
  const retried = controller.loadOlder();
  measuringReplay = true;
  retry.resolve({ entries: [original], nextBeforeSeq: 1, nextBeforeCursor: "opaque-walk:1" });
  await retried;
  measuringReplay = false;
  assert.ok(supersededPayloadReads <= 4, "one identity must not replay its superseded event log");
  assert.deepEqual(ids(controller.getSnapshot().entries), ["older", "tail"]);
  const refreshed = row(controller.getSnapshot().entries, original.id);
  assert.equal(refreshed.text, "edit-255");
  assert.deepEqual(refreshed.attachments, []);
  assert.equal(refreshed.richText, "");
  assert.deepEqual(refreshed.reactions, []);
  assert.equal(refreshed.clientNonce, original.clientNonce);
  assert.equal(controller.getSnapshot().olderFailure, null);
  assert.deepEqual(requests, [
    { id: "conversation", limit: 200, beforeSeq: 5, beforeCursor: "opaque-walk:5" },
    { id: "conversation", limit: 200, beforeSeq: 5, beforeCursor: "opaque-walk:5" },
  ]);
  controller.dispose();
});

test("NMR-26 two failed reads retain composed previous-ID aliases without injecting unrelated rows", async () => {
  const reads = [deferred<TranscriptHistoryPage>(), deferred<TranscriptHistoryPage>(), deferred<TranscriptHistoryPage>()] as const;
  let requestIndex = 0;
  const original: TranscriptMessage = {
    ...message("old", "stale body"), authorKind: "human", authorId: "local",
    attachments: [{ path: "/old-resource", name: "old.txt" }], richText: "old-rich-text",
  };
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation", fetchPage: () => {
      const read = reads[requestIndex++];
      assert.ok(read, "only the three explicit page requests may run");
      return read.promise;
    },
  });
  controller.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 5, nextBeforeCursor: "opaque-walk:5" });
  const first = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("middle", "first edit"), authorKind: "human", authorId: "local", attachments: [], richText: "",
  }, original.id);
  reads[0].reject(new Error("first failure"));
  await first;
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("canonical", "second edit"), authorKind: "human", authorId: "local",
  }, "middle");
  const second = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("canonical", ""), authorKind: "human", authorId: "local", reactions: [],
  });
  const failure = new Error("second failure");
  reads[1].reject(failure);
  await second;
  assert.equal(controller.getSnapshot().olderFailure, failure);
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("canonical", "latest edit"), authorKind: "human", authorId: "local",
  });
  controller.ingestEntryUpdate("account", "conversation", message("outside-page"));
  const third = controller.loadOlder();
  reads[2].resolve({ entries: [original, { ...original, id: "middle" }] });
  await third;
  assert.equal(requestIndex, 3);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["canonical", "tail"]);
  const refreshed = row(controller.getSnapshot().entries, "canonical");
  assert.equal(refreshed.text, "latest edit");
  assert.deepEqual(refreshed.attachments, []);
  assert.equal(refreshed.richText, "");
  assert.deepEqual(refreshed.reactions, []);
  assert.equal(controller.getSnapshot().olderFailure, null);
  controller.dispose();
});

test("NMR-26 failed-page coalescing rejects cross-author aliases with and without nonces in either event order", async () => {
  for (const [clientNonce, peerFirst] of [
    [undefined, false], [undefined, true], ["shared", false], ["shared", true],
  ] as const) {
    const first = deferred<TranscriptHistoryPage>();
    const retry = deferred<TranscriptHistoryPage>();
    let requestIndex = 0;
    const local: TranscriptMessage = {
      ...message(clientNonce == null ? "old" : "human-message:shared", "local draft"), authorKind: "human", authorId: "local",
      clientNonce, delivery: "pending",
    };
    const peer: TranscriptMessage = {
      ...local, id: "peer-row", authorId: "peer", text: "peer body", delivery: "sent", remoteMessageId: "41",
    };
    const canonical: TranscriptMessage = {
      ...local, id: "local-canonical", text: "local settled", delivery: "sent", remoteMessageId: "42", attachments: [],
    };
    const controller = createTranscriptPaginationController({
      accountSlot: "account", agentId: "conversation",
      fetchPage: () => ++requestIndex === 1 ? first.promise : retry.promise,
    });
    controller.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 5 });
    const loading = controller.loadOlder();
    const localEvent = () => controller.ingestEntryUpdate("account", "conversation", canonical, local.id);
    const foreignEvents = () => {
      controller.ingestEntryUpdate("account", "conversation", { ...peer, text: "peer updated" }, local.id);
      controller.ingestEntryUpdate("account", "conversation", {
        ...peer, id: "authorless-row", authorId: undefined, text: "unqualified identity",
      }, local.id);
    };
    if (peerFirst) { foreignEvents(); localEvent(); }
    else { localEvent(); foreignEvents(); }
    first.reject(new Error("retry required"));
    await loading;
    controller.ingestEntryUpdate("account", "conversation", { ...peer, text: "peer latest" }, local.id);
    const retried = controller.loadOlder();
    retry.resolve({ entries: [local, peer] });
    await retried;
    assert.deepEqual(ids(controller.getSnapshot().entries), ["local-canonical", "peer-row", "tail"]);
    assert.equal(row(controller.getSnapshot().entries, canonical.id).authorId, "local");
    assert.equal(row(controller.getSnapshot().entries, canonical.id).text, "local settled");
    assert.deepEqual(row(controller.getSnapshot().entries, canonical.id).attachments, []);
    assert.equal(row(controller.getSnapshot().entries, peer.id).authorId, "peer");
    assert.equal(row(controller.getSnapshot().entries, peer.id).text, "peer latest");
    controller.dispose();
  }
});

test("NMR-26 appended canonical nonce settlement preserves earlier clears without a previous-ID event", async () => {
  const first = deferred<TranscriptHistoryPage>();
  const retry = deferred<TranscriptHistoryPage>();
  let requestIndex = 0;
  const pending: TranscriptMessage = {
    ...message("human-message:original", "stale draft"), authorKind: "human", authorId: "local",
    clientNonce: "original", delivery: "pending",
    attachments: [{ path: "/old-resource", name: "old.txt" }], richText: "old-rich-text",
  };
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: () => ++requestIndex === 1 ? first.promise : retry.promise,
  });
  controller.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 5 });
  const loading = controller.loadOlder();
  controller.ingestEntryUpdate("account", "conversation", {
    ...pending, text: "", attachments: [], richText: "", reactions: [],
  });
  first.reject(new Error("retry required"));
  await loading;
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("canonical", "settled"), authorKind: "human", authorId: "local",
    clientNonce: pending.clientNonce, delivery: "sent", remoteMessageId: "42",
  });
  controller.ingestEntryUpdate("account", "conversation", {
    ...message("canonical", "latest partial"), authorKind: "human", authorId: "local",
  });
  const retried = controller.loadOlder();
  retry.resolve({ entries: [pending] });
  await retried;
  assert.deepEqual(ids(controller.getSnapshot().entries), ["canonical", "tail"]);
  const refreshed = row(controller.getSnapshot().entries, "canonical");
  assert.equal(refreshed.text, "latest partial");
  assert.equal(refreshed.clientNonce, pending.clientNonce);
  assert.equal(refreshed.delivery, "sent");
  assert.equal(refreshed.remoteMessageId, "42");
  assert.deepEqual(refreshed.attachments, []);
  assert.equal(refreshed.richText, "");
  assert.deepEqual(refreshed.reactions, []);
  controller.dispose();
});

test("NMR-26 the first unloaded-row update after a failed read survives its explicit retry", async () => {
  const retry = deferred<TranscriptHistoryPage>();
  const failure = new Error("failed before any events");
  let requestIndex = 0;
  const original: TranscriptMessage = {
    ...message("older", "stale"), authorKind: "human", authorId: "local",
    attachments: [{ path: "/old-resource", name: "old.txt" }], richText: "old-rich-text",
  };
  const controller = createTranscriptPaginationController({
    accountSlot: "account", agentId: "conversation",
    fetchPage: () => ++requestIndex === 1 ? Promise.reject(failure) : retry.promise,
  });
  controller.installInitialPage({ entries: [message("tail")], nextBeforeSeq: 5, nextBeforeCursor: "opaque-walk:5" });
  await controller.loadOlder();
  assert.equal(controller.getSnapshot().olderFailure, failure);
  controller.ingestEntryUpdate("account", "conversation", { ...original, text: "", attachments: [], richText: "" });
  assert.deepEqual(ids(controller.getSnapshot().entries), ["tail"]);
  const retried = controller.loadOlder();
  assert.equal(controller.loadOlder(), retried);
  retry.resolve({ entries: [original] });
  await retried;
  assert.equal(requestIndex, 2);
  assert.deepEqual(ids(controller.getSnapshot().entries), ["older", "tail"]);
  const refreshed = row(controller.getSnapshot().entries, original.id);
  assert.equal(refreshed.text, "");
  assert.deepEqual(refreshed.attachments, []);
  assert.equal(refreshed.richText, "");
  assert.equal(controller.getSnapshot().olderFailure, null);
  controller.dispose();
});

