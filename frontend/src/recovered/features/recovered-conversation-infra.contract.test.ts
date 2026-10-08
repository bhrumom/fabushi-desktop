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
import { projectRichMessageAction, projectRichMessageActionAffordance } from "./conversation/cards/transcript-card/url-card.ts";
import { createWidgetInteractionAdapter } from "./conversation/cards/transcript-card/widget-interactions.ts";
import { createAssistantMathMarkupCache, type KatexRuntime } from "./conversation/workspace/math.tsx";
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


test("rich widget button actions project fail-closed URL/copy semantics", () => {
  assert.deepEqual(projectRichMessageAction({ kind: "open-url", data: "https://example.com/docs?q=1" }), {
    kind: "open-url",
    data: "https://example.com/docs?q=1",
  });
  assert.deepEqual(projectRichMessageAction({ kind: "authorize-url", data: "https://example.com/auth?token=opaque" }), {
    kind: "authorize-url",
    data: "https://example.com/auth?token=opaque",
  });
  assert.deepEqual(projectRichMessageAction({ kind: "copy-text", data: "ABC-123" }), {
    kind: "copy-text",
    data: "ABC-123",
  });
  assert.equal(projectRichMessageAction({ kind: "run-script", data: "opaque" }), null);
  assert.equal(projectRichMessageAction({ kind: "open-url", data: "   " }), null);

  assert.deepEqual(
    projectRichMessageActionAffordance({ kind: "open-url", data: "https://example.com/docs?q=1" }),
    {
      tooltip: "https://example.com/docs?q=1",
      copyText: "https://example.com/docs?q=1",
      copyLabel: "Copy Link",
      normalizedUrl: "https://example.com/docs?q=1",
    },
  );
  assert.equal(
    projectRichMessageActionAffordance(
      { kind: "authorize-url", data: "https://example.com/auth" },
      "Long authorization label",
    ).tooltip,
    "Long authorization label\n\nhttps://example.com/auth",
  );
  assert.deepEqual(
    projectRichMessageActionAffordance({ kind: "copy-text", data: "ABC-123" }),
    {
      tooltip: "Copy text:\nABC-123",
      copyText: "ABC-123",
      copyLabel: "Copy Text",
      normalizedUrl: null,
    },
  );
  assert.equal(
    projectRichMessageActionAffordance({ kind: "open-url", data: "javascript:alert(1)" }).normalizedUrl,
    null,
  );
});

test("widget action lifecycle fences duplicate and stale rich-button settlement", async () => {
  let resolveResponse!: (value: { accepted: boolean }) => void;
  const response = new Promise<{ accepted: boolean }>((resolve) => { resolveResponse = resolve; });
  const adapter = createWidgetInteractionAdapter({
    scope: { accountSlot: "slot-1", agentId: "agent-1" },
    transport: {
      respondToWidget: async () => response,
      dismissWidget: async () => ({ accepted: true }),
    },
  });
  adapter.replaceEntries([{
    kind: "send-message",
    id: "widget-1",
    message: {
      type: "widget",
      widget: {
        prompt: "Choose",
        options: [{ label: "Open", action: { kind: "open-url", data: "https://example.com" } }],
      },
    },
  }]);

  const pending = adapter.respond("widget-1", "Open");
  assert.equal(adapter.getSnapshot("widget-1").state, "pending");
  assert.deepEqual(await adapter.respond("widget-1", "Open"), { kind: "ignored", reason: "settled" });

  adapter.setScope({ accountSlot: "slot-1", agentId: "agent-2" });
  resolveResponse({ accepted: true });
  const stale = await pending;
  assert.equal(stale.kind, "stale");
  assert.equal(adapter.getSnapshot("widget-1").state, "idle");

  adapter.dispose();
  assert.deepEqual(await adapter.dismiss("widget-1"), { kind: "ignored", reason: "unavailable" });
});


test("assistant math cache shares renders, isolates loaders, and retries load failure", async () => {
  const cache = createAssistantMathMarkupCache();
  let loadsA = 0;
  let rendersA = 0;
  const runtimeA: KatexRuntime = {
    renderToString(expression, options) {
      rendersA += 1;
      return `<span>${options.displayMode ? "D" : "I"}:${expression}</span>`;
    },
  };
  const loaderA = async () => {
    loadsA += 1;
    return runtimeA;
  };

  const [first, second] = await Promise.all([
    cache.load(loaderA, "x^2", false),
    cache.load(loaderA, "x^2", false),
  ]);
  assert.equal(first, "<span>I:x^2</span>");
  assert.equal(second, first);
  assert.equal(loadsA, 1);
  assert.equal(rendersA, 1);

  await cache.load(loaderA, "x^2", true);
  assert.equal(loadsA, 2);
  assert.equal(rendersA, 2);

  let loadsB = 0;
  const loaderB = async () => {
    loadsB += 1;
    return runtimeA;
  };
  await cache.load(loaderB, "x^2", false);
  assert.equal(loadsB, 1);

  cache.invalidate(loaderA);
  await cache.load(loaderA, "x^2", false);
  assert.equal(loadsA, 3);

  let attempts = 0;
  const healingLoader = async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("temporary chunk failure");
    return runtimeA;
  };
  await assert.rejects(() => cache.load(healingLoader, "y", false));
  assert.equal(await cache.load(healingLoader, "y", false), "<span>I:y</span>");
  assert.equal(attempts, 2);
});
