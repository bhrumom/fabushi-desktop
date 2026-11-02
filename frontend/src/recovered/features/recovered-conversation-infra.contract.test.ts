import assert from "node:assert/strict";
import test from "node:test";

import {
  TIMELINE_EVENT_REGISTRY,
  createTimelineEventRegistry,
  projectTimelineEvent,
  timelineEventActionVerb,
  timelineEventDescription,
  timelineEventProtocolKey,
} from "./conversation/cards/timeline-event-registry.ts";
import {
  createTranscriptFeedFanout,
  type TranscriptClientEventSource,
} from "./conversation/cards/transcript-card/transcript-feed-source.ts";
import {
  createHiddenChatsMutationController,
} from "./hidden-chats/overlay/mutation-controller.ts";
import {
  createFindInChatController,
  findInChatSearchText,
} from "./conversation/workspace/find-in-chat-controller.ts";
import {
  formatRoutineRunTimestamp,
  presentRoutineRunHistory,
} from "./automations/routines/run-history.ts";
import {
  createComposerDraftStateStore,
  type ComposerDraftPersistence,
} from "./conversation/workspace/draft-state.ts";
import {
  isForwardRecipientNavigationKey,
  isForwardSubmitShortcut,
  isForwardToggleShortcut,
  nextForwardRecipientIndex,
} from "../../production/forward-recipient-navigation.ts";
import { projectTranscriptPageResult } from "../../production/model.ts";
import type { ConversationTranscriptEntry } from "./conversation/workspace/model.ts";
import {
  createTranscriptPaginationController,
  mergeOlderTranscriptEntries,
  type TranscriptHistoryPage,
  type TranscriptHistoryRequest,
} from "./conversation/workspace/pagination.ts";

const flush = async (): Promise<void> => {
  await Promise.resolve();
  await Promise.resolve();
};

test("timeline event registry validates protocol rows and descriptions", () => {
  const event = projectTimelineEvent({
    type: "automation-changed",
    automationId: "auto-1",
    action: "enabled",
    automationName: "Daily",
  });
  assert.ok(event);
  assert.equal(timelineEventProtocolKey(event), "event:automation-changed");
  assert.equal(timelineEventActionVerb(event), "Enabled");
  assert.equal(
    timelineEventDescription(event),
    'Enabled automation "Daily"',
  );
  assert.equal(projectTimelineEvent({
    type: "automation-changed",
    automationId: "",
    action: "enabled",
    automationName: "Daily",
  }), null);

  assert.equal(
    createTimelineEventRegistry()
      .metadata("event:automation-changed")
      .placeholderHeight,
    32,
  );
  assert.equal(
    TIMELINE_EVENT_REGISTRY["event:automation-changed"].icon,
    "calendar",
  );
});

test("transcript feed fanout shares one subscription and validates entry identity", () => {
  let subscribed = 0;
  let released = 0;
  let listener: ((value: unknown) => void) | null = null;
  const source: TranscriptClientEventSource = {
    subscribe(family, next) {
      assert.equal(family, "transcript");
      subscribed += 1;
      listener = next;
      return () => { released += 1; };
    },
  };
  const fanout = createTranscriptFeedFanout(source);
  assert.ok(fanout);
  assert.equal(subscribed, 1);

  const events: string[] = [];
  const observe = () => fanout.observeEntriesFeed({
    onBaseline: ({ entries }) => events.push(`baseline:${entries.length}`),
    onAppended: ({ entry }) => events.push(`append:${(entry as { id: string }).id}`),
    onUpdated: ({ after }) => events.push(`update:${(after as { id: string }).id}`),
    onCleared: () => events.push("clear"),
  });
  const releaseA = observe();
  const releaseB = observe();

  listener?.({
    type: "snapshot",
    activeAgentId: "a",
    entries: [{ id: "1" }, { id: "2" }],
  });
  listener?.({
    type: "snapshot",
    activeAgentId: "a",
    entries: [{ id: "1" }, { id: "1" }],
  });
  listener?.({ type: "appended", agentId: "a", entry: { id: "3" } });
  listener?.({
    type: "updated",
    owningAgentId: "a",
    before: { id: "3" },
    entry: { id: "3" },
  });
  listener?.({ type: "cleared", agentId: "a" });

  assert.deepEqual(events, [
    "baseline:2", "baseline:2",
    "append:3", "append:3",
    "update:3", "update:3",
    "clear", "clear",
  ]);
  releaseA();
  releaseB();
  fanout.dispose();
  assert.equal(released, 1);
});

test("hidden chat mutations rollback permanent failures and hold transport failures for reconnect", async () => {
  let current = { id: "a", isHidden: false, updatedAt: 1 };
  const optimistic: unknown[] = [];
  const rollback: unknown[] = [];
  let fail: unknown = null;
  const controller = createHiddenChatsMutationController({
    call: async () => {
      if (fail != null) throw fail;
      return {};
    },
    readAgent: () => current,
    onOptimisticChange: (id, value) => {
      optimistic.push([id, value]);
      current = { ...current, isHidden: value };
    },
    onRollback: (id, optimisticValue, previousValue) => {
      rollback.push([id, optimisticValue, previousValue]);
      current = { ...current, isHidden: previousValue };
    },
  });
  controller.setScope("slot", "a");

  fail = { code: "validation" };
  await assert.rejects(
    () => controller.setAgentHiddenFromSidebar("a", true),
  );
  assert.deepEqual(rollback, [["a", true, false]]);
  assert.equal(controller.isPending("a"), false);

  fail = { code: "source/transport-failure" };
  await assert.rejects(
    () => controller.setAgentHiddenFromSidebar("a", true),
  );
  assert.equal(controller.isPending("a"), true);

  fail = null;
  controller.noteReconnect();
  await flush();
  assert.equal(controller.isPending("a"), false);

  controller.ingestAgents([current]);
  assert.equal(controller.isPending("a"), false);
  assert.equal(optimistic.length, 2);
  controller.dispose();
});

test("find-in-chat searches visible text, wraps navigation and resets on scope", () => {
  const navigated: unknown[] = [];
  const controller = createFindInChatController({
    scope: { accountSlot: "slot", agentId: "a" },
    onNavigate: (match, index) => navigated.push([match, index]),
  });
  controller.replaceEntries([
    { id: "1", kind: "message", text: "Alpha beta alpha" },
    {
      id: "2",
      kind: "send-message",
      message: {
        type: "email-draft",
        draft: { subject: "Alpha", body: "Body" },
      },
    },
    { id: "3", kind: "other" },
  ]);
  controller.setQuery("alpha");
  assert.equal(controller.getSnapshot().matches.length, 3);
  assert.deepEqual(controller.step(1), { entryId: "1", occurrence: 0 });
  assert.deepEqual(controller.step(-1), { entryId: "2", occurrence: 0 });
  assert.equal(navigated.length, 2);
  assert.equal(
    findInChatSearchText({
      id: "2",
      kind: "send-message",
      message: {
        type: "slack-draft",
        draft: { body: "Hello" },
      },
    }),
    "Hello",
  );

  const priorGeneration = controller.getSnapshot().generation;
  controller.setScope("slot", "b");
  assert.equal(controller.getSnapshot().generation, priorGeneration + 1);
  assert.equal(controller.getSnapshot().query, "");
  assert.equal(controller.getSnapshot().matches.length, 0);
  controller.dispose();
});

test("routine history formats status and relative/zoned timestamps", () => {
  const now = Date.UTC(2026, 0, 15, 20, 0, 0);
  assert.equal(formatRoutineRunTimestamp(now - 20_000, now, "UTC"), "Just now");
  assert.equal(formatRoutineRunTimestamp(now - 5 * 60_000, now, "UTC"), "5 min ago");
  assert.equal(formatRoutineRunTimestamp(now + 5 * 60_000, now, "UTC"), "In 5 min");

  const history = presentRoutineRunHistory([
    { id: "1", status: "running", startedAt: now - 1000, detail: "Run" },
    { id: "2", status: "ok", startedAt: now - 1000 },
    { id: "3", status: "error", startedAt: now - 1000 },
  ], now, "UTC");
  assert.equal(history.empty, false);
  assert.deepEqual(history.rows.map((row) => row.ariaLabel), [
    "Running", "Succeeded", "Failed",
  ]);
  assert.equal(presentRoutineRunHistory([], now, "UTC").empty, true);
});



test("composer draft restore does not overwrite a staged attachment added while persistence is loading", async () => {
  let releaseRead!: () => void;
  let noteReadStarted!: () => void;
  const readStarted = new Promise<void>((resolve) => { noteReadStarted = resolve; });
  const readGate = new Promise<void>((resolve) => { releaseRead = resolve; });
  const persisted = JSON.stringify({
    schemaVersion: 1,
    value: {
      agents: {
        "agent-1": {
          draft: { prompt: "persisted draft", attachments: [] },
          draftId: "persisted-draft-id",
          recovery: null,
        },
      },
    },
  });
  const persistence: ComposerDraftPersistence = {
    async read(accountSlot) {
      assert.equal(accountSlot, "account-1");
      noteReadStarted();
      await readGate;
      return persisted;
    },
    async write() {},
    async clear() {},
  };
  const store = createComposerDraftStateStore(persistence);
  const restoring = store.restore("account-1");
  await readStarted;

  store.setDraft("agent-1", {
    prompt: "",
    attachments: [{ path: "/tmp/agent-notes.txt", name: "agent-notes.txt" }],
  });
  releaseRead();
  await restoring;

  assert.deepEqual(store.snapshotsFor("agent-1").get().draft, {
    prompt: "",
    attachments: [{ path: "/tmp/agent-notes.txt", name: "agent-notes.txt" }],
  });
  store.dispose();
});


test("forward recipient keyboard policy preserves picker boundaries and submit semantics", () => {
  assert.equal(nextForwardRecipientIndex(0, 0, "ArrowDown"), null);
  assert.equal(nextForwardRecipientIndex(0, 8, "ArrowUp"), 0);
  assert.equal(nextForwardRecipientIndex(0, 8, "ArrowDown"), 1);
  assert.equal(nextForwardRecipientIndex(1, 8, "PageDown", 3), 4);
  assert.equal(nextForwardRecipientIndex(4, 8, "PageUp", 3), 1);
  assert.equal(nextForwardRecipientIndex(4, 8, "Home"), 0);
  assert.equal(nextForwardRecipientIndex(4, 8, "End"), 7);
  assert.equal(nextForwardRecipientIndex(7, 8, "ArrowDown"), 7);

  assert.equal(isForwardRecipientNavigationKey("PageDown"), true);
  assert.equal(isForwardRecipientNavigationKey("Enter"), false);
  assert.equal(isForwardSubmitShortcut({ key: "Enter", ctrlKey: true, metaKey: false }), true);
  assert.equal(isForwardSubmitShortcut({ key: "Enter", ctrlKey: false, metaKey: true }), true);
  assert.equal(isForwardSubmitShortcut({ key: "Enter", ctrlKey: false, metaKey: false }), false);
  assert.equal(isForwardToggleShortcut({ key: "Enter", ctrlKey: false, metaKey: false }), true);
  assert.equal(isForwardToggleShortcut({ key: " ", ctrlKey: false, metaKey: false }), true);
  assert.equal(isForwardToggleShortcut({ key: "Enter", ctrlKey: true, metaKey: false }), false);
});

const historyEntry = (id: string): ConversationTranscriptEntry => ({
  id, kind: "message", role: "user", author: "Peer", text: id, timestampMs: 1000,
});

function deferredHistoryPage() {
  let resolve!: (page: TranscriptHistoryPage) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<TranscriptHistoryPage>((resolvePage, rejectPage) => {
    resolve = resolvePage;
    reject = rejectPage;
  });
  return { promise, resolve, reject };
}

test("Human opaque history cursors survive projection, root-cache commits and cached conversation reopen", async () => {
  const firstToken = "opaque-history:walk-A/+=:2:5";
  const secondToken = "opaque-history:walk-A/+=:4:3";
  const initial = projectTranscriptPageResult({
    entries: ["m5", "m6"].map((id) => ({ id, kind: "message", content: id, timestampMs: 1000 })),
    nextBeforeSeq: 5,
    nextBeforeCursor: firstToken,
  }, "Peer", "human-room");
  assert.equal(initial.nextBeforeCursor, firstToken);
  const dispatching = projectTranscriptPageResult({ entries: [{
    id: "human-message:local-nonce", kind: "message", authorKind: "human", authorId: "local",
    clientNonce: "local-nonce", delivery: "dispatching", content: "sending", timestampMs: 1000,
  }, {
    id: "human-server-message:42", kind: "message", authorKind: "human", authorId: "peer",
    remoteMessageId: "42", delivery: "sent", content: "received", timestampMs: 1000,
  }] }, "Peer", "human-room").entries;
  assert.ok(dispatching[0].kind === "message" && dispatching[1].kind === "message");
  assert.equal(dispatching[0].delivery, "pending");
  assert.equal(dispatching[0].authorKind, "human");
  assert.equal(dispatching[0].authorId, "local");
  assert.equal(dispatching[1].remoteMessageId, "42");
  let cached = initial.entries as readonly ConversationTranscriptEntry[];
  let cachedPage: TranscriptHistoryPage = { ...initial, entries: [] };
  const requests: TranscriptHistoryRequest[] = [];
  const controller = createTranscriptPaginationController({
    accountSlot: "account-A", agentId: "human-room", pageLimit: 2,
    fetchPage: async (request) => {
      requests.push(request);
      return requests.length === 1
        ? { entries: [historyEntry("m3"), historyEntry("m4")], nextBeforeSeq: 3, nextBeforeCursor: secondToken }
        : { entries: [historyEntry("m1"), historyEntry("m2")] };
    },
    onPageApplied: (scope, page) => {
      assert.deepEqual(scope, { accountSlot: "account-A", agentId: "human-room" });
      cached = mergeOlderTranscriptEntries(cached, page.entries);
      cachedPage = { ...page, entries: [] };
    },
  });
  controller.installInitialPage(cachedPage);
  await controller.loadOlder();
  assert.equal(cachedPage.nextBeforeCursor, secondToken);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 3, beforeCursor: secondToken });
  controller.setScope("account-A", "other-room");
  controller.installInitialPage({ entries: [historyEntry("other")] });
  controller.setScope("account-A", "human-room");
  controller.installInitialPage(cachedPage);
  await controller.loadOlder();
  assert.deepEqual(requests, [
    { id: "human-room", limit: 2, beforeSeq: 5, beforeCursor: firstToken },
    { id: "human-room", limit: 2, beforeSeq: 3, beforeCursor: secondToken },
  ]);
  assert.deepEqual(cached.map((entry) => entry.id), ["m1", "m2", "m3", "m4", "m5", "m6"]);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "exhausted" });
  assert.equal(cachedPage.nextBeforeCursor, undefined);
  controller.dispose();
});

test("Human failed history retries preserve the exact token and the latest committed row overlay", async () => {
  const pages = [deferredHistoryPage(), deferredHistoryPage()];
  const requests: TranscriptHistoryRequest[] = [];
  const commits: TranscriptHistoryPage[] = [];
  const token = "opaque-history:retry/+=:2:3";
  const controller = createTranscriptPaginationController({
    accountSlot: "account-A", agentId: "human-room",
    fetchPage: (request) => {
      requests.push(request);
      return pages[requests.length - 1].promise;
    },
    onPageApplied: (_scope, page) => { commits.push(page); },
  });
  const initial = historyEntry("m3");
  controller.installInitialPage({ entries: [initial], nextBeforeSeq: 3, nextBeforeCursor: token });
  const first = controller.loadOlder();
  await flush();
  const updated = { ...historyEntry("m1"), reactions: [{ emoji: "❤️", by: "me" } as const] };
  controller.ingestEntryUpdate("account-A", "human-room", updated);
  const failure = new Error("older-page transport interrupted");
  pages[0].reject(failure);
  await first;
  assert.deepEqual(controller.getSnapshot().entries, [initial]);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 3, beforeCursor: token });
  assert.equal(controller.getSnapshot().olderFailure, failure);
  assert.equal(controller.getSnapshot().isLoadingOlder, false);
  assert.equal(commits.length, 0);
  const latest = { ...historyEntry("m1"), reactions: [{ emoji: "🎉", by: "me" } as const] };
  // Updates between failure and retry must also replace that read's overlay.
  controller.ingestEntryUpdate("account-A", "human-room", latest);
  const retry = controller.loadOlder();
  await flush();
  pages[1].resolve({ entries: [historyEntry("m1"), historyEntry("m2"), historyEntry("m3")] });
  await retry;
  assert.deepEqual(requests, [
    { id: "human-room", limit: 200, beforeSeq: 3, beforeCursor: token },
    { id: "human-room", limit: 200, beforeSeq: 3, beforeCursor: token },
  ]);
  assert.deepEqual(controller.getSnapshot().entries, [latest, historyEntry("m2"), initial]);
  assert.deepEqual(commits[0].entries, controller.getSnapshot().entries);
  assert.equal(controller.getSnapshot().olderFailure, null);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "exhausted" });
  controller.dispose();
});

test("Human opaque history rejects stale account ABA results and their buffered updates", async () => {
  for (const outcome of ["success", "failure"] as const) {
    const stalePage = deferredHistoryPage();
    const freshPage = deferredHistoryPage();
    const requests: TranscriptHistoryRequest[] = [];
    const commits: TranscriptHistoryPage[] = [];
    const controller = createTranscriptPaginationController({
      accountSlot: "account-A", agentId: "human-room",
      fetchPage: (request) => {
        requests.push(request);
        return requests.length === 1 ? stalePage.promise : freshPage.promise;
      },
      onPageApplied: (_scope, page) => { commits.push(page); },
    });
    controller.installInitialPage({ entries: [], nextBeforeSeq: 10, nextBeforeCursor: "retired-token" });
    const stale = controller.loadOlder();
    await flush();
    controller.ingestEntryUpdate("account-A", "human-room", { ...historyEntry("m1"), text: "retired update" });
    controller.setScope("account-B", "human-room");
    assert.deepEqual(controller.getSnapshot().cursor, { kind: "unprobed" });
    controller.setScope("account-A", "human-room");
    controller.installInitialPage({ entries: [], nextBeforeSeq: 8, nextBeforeCursor: "current-token" });
    const fresh = controller.loadOlder();
    await flush();
    if (outcome === "success") stalePage.resolve({ entries: [historyEntry("must-not-leak")] });
    else stalePage.reject({ code: "source/capability-unavailable" });
    await stale;
    assert.equal(controller.getSnapshot().isLoadingOlder, true);
    assert.deepEqual(controller.getSnapshot().entries, []);
    assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 8, beforeCursor: "current-token" });
    assert.equal(commits.length, 0);
    freshPage.resolve({ entries: [historyEntry("m1")] });
    await fresh;
    assert.deepEqual(controller.getSnapshot().entries, [historyEntry("m1")]);
    assert.deepEqual(requests[1], { id: "human-room", limit: 200, beforeSeq: 8, beforeCursor: "current-token" });
    assert.equal(commits.length, 1);
    controller.dispose();
  }
});

test("Human replacement initial history clears the prior token and failed-read overlays", async () => {
  const retiredPage = deferredHistoryPage();
  const requests: TranscriptHistoryRequest[] = [];
  const controller = createTranscriptPaginationController({
    accountSlot: "account-A", agentId: "human-room",
    fetchPage: (request) => {
      requests.push(request);
      return requests.length === 1 ? retiredPage.promise : Promise.resolve({ entries: [historyEntry("m1")] });
    },
  });
  controller.installInitialPage({ entries: [], nextBeforeSeq: 10, nextBeforeCursor: "retired-token" });
  const retired = controller.loadOlder();
  await flush();
  controller.ingestEntryUpdate("account-A", "human-room", { ...historyEntry("m1"), text: "retired update" });
  retiredPage.reject(new Error("failed old read"));
  await retired;
  controller.installInitialPage({ entries: [historyEntry("m2")], nextBeforeSeq: 2, nextBeforeCursor: "replacement-token" });
  await controller.loadOlder();
  assert.deepEqual(requests[1], { id: "human-room", limit: 200, beforeSeq: 2, beforeCursor: "replacement-token" });
  assert.deepEqual(controller.getSnapshot().entries, [historyEntry("m1"), historyEntry("m2")]);
  assert.equal(controller.getSnapshot().olderFailure, null);
  controller.reset();
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "unprobed" });
  controller.dispose();
});

test("Agent pagination retains its sequence-only request and viewport anchor", async () => {
  const requests: TranscriptHistoryRequest[] = [];
  const commits: TranscriptHistoryPage[] = [];
  const viewport = { scrollTop: 45, scrollHeight: 500 };
  const controller = createTranscriptPaginationController({
    accountSlot: "account-A", agentId: "agent-A", pageLimit: 50,
    fetchPage: async (request) => {
      requests.push(request);
      return { entries: [historyEntry("a1"), historyEntry("a2")], nextBeforeSeq: 1 };
    },
    onPageApplied: (_scope, page) => { commits.push(page); },
  });
  controller.installInitialPage({ entries: [historyEntry("a2"), historyEntry("a3")], nextBeforeSeq: 2 });
  controller.bindViewport(viewport);
  await controller.loadOlder();
  assert.deepEqual(requests, [{ id: "agent-A", limit: 50, beforeSeq: 2 }]);
  assert.equal(Object.hasOwn(requests[0], "beforeCursor"), false);
  assert.equal(Object.hasOwn(commits[0], "nextBeforeCursor"), false);
  assert.deepEqual(controller.getSnapshot().cursor, { kind: "more", beforeSeq: 1 });
  assert.deepEqual(controller.getSnapshot().entries.map((entry) => entry.id), ["a1", "a2", "a3"]);
  viewport.scrollHeight = 700;
  assert.equal(controller.onEntriesCommitted(), true);
  assert.equal(viewport.scrollTop, 245);
  assert.equal(controller.onEntriesCommitted(), false);
  controller.dispose();
});

