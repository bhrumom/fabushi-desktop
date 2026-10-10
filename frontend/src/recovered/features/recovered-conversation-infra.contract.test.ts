import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
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
import { projectRichMessageAction, projectRichMessageActionAffordance, projectTranscriptExternalLink } from "./conversation/cards/transcript-card/url-card.ts";
import { createWidgetInteractionAdapter } from "./conversation/cards/transcript-card/widget-interactions.ts";
import { createAssistantMathMarkupCache, type KatexRuntime } from "./conversation/workspace/math-runtime.ts";
import { resolveWithSingleRetry } from "./conversation/workspace/media-runtime.ts";
import { accumulateWheelZoomSteps, normalizeWheelZoomDelta } from "./conversation/workspace/media-zoom.ts";
import { DERIVED_MEDIA_PRELOAD_ROOT_MARGIN, DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN, isDerivedMediaNearViewport, isVisibilityBoundDerivedMedia, observeDerivedMediaVisibility, shouldResolveDerivedMedia, shouldResolveDerivedThumbnail } from "./conversation/workspace/media-visibility.ts";
import { TRANSCRIPT_FOLLOW_LATEST_THRESHOLD_PX, isTranscriptNearBottom } from "./conversation/workspace/transcript-follow-state.ts";
import { isTranscriptDeliveryActionable, isTranscriptDeliveryBusy, normalizeTranscriptDelivery } from "./conversation/workspace/transcript-delivery-state.ts";
import { beginHorizontalScrollPointer, captureHorizontalScroll, clampHorizontalScrollOffset, normalizeHorizontalScrollWheelDelta, restoreHorizontalScrollOffset, updateHorizontalScrollPointer, updateHorizontalScrollWheelLock } from "./conversation/workspace/horizontal-scroll-state.ts";
import {
  areAssistantProjectionCandidatesCompatible,
  reconcileAssistantContentProjection,
  type AssistantProjectionCandidate,
} from "./conversation/workspace/assistant-content-projection.ts";
import {
  createHiddenChatsMutationController,
} from "./hidden-chats/overlay/mutation-controller.ts";
import {
  createFindInChatController,
  findInChatSearchText,
  includeFindInChatDisclosure,
} from "./conversation/workspace/find-in-chat-controller.ts";
import { createFindHighlightRefreshRuntime } from "./conversation/workspace/find-highlight-runtime.ts";
import { formatTranscriptToolCallName } from "./conversation/workspace/tool-call-label.ts";
import { copyTranscriptCodeText } from "./conversation/workspace/code-copy.ts";
import {
  formatRoutineRunTimestamp,
  presentRoutineRunHistory,
} from "./automations/routines/run-history.ts";
import {
  createComposerDraftStateStore,
  type ComposerDraftPersistence,
} from "./conversation/workspace/draft-state.ts";
import { createTranscriptAcknowledgementController } from "./conversation/workspace/acknowledgement.ts";
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



test("UNIT-FBCP-HUMAN-REMOTE-IDENTITY-SETTLEMENT-001 acknowledgement settlement fails closed on missing, foreign, and ambiguous nonce evidence", () => {
  const controller = createTranscriptAcknowledgementController();
  controller.setScope("account-a", "human-a");
  assert.equal(controller.insertOptimistic({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-a",
    entries: [{ id: "pending-nonce-a", kind: "message", clientNonce: "nonce-a" }],
    phase: "dispatching",
  }), true);

  assert.equal(controller.reconcileEcho({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-a",
  }), false, "a missing remote nonce cannot settle local optimistic state");
  assert.equal(controller.reconcileEcho({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-a",
    echoedNonce: "foreign-nonce",
  }), false, "a foreign remote nonce cannot settle local optimistic state");
  assert.equal(controller.getSnapshot().records.length, 1);

  assert.equal(controller.reconcileEcho({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-a",
    echoedNonce: "nonce-a",
  }), true);
  assert.equal(controller.getSnapshot().records.length, 0);

  assert.equal(controller.insertOptimistic({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-shared",
    entries: [{ id: "pending-human-a", kind: "message", clientNonce: "nonce-shared" }],
    phase: "dispatching",
  }), true);
  assert.equal(controller.insertOptimistic({
    accountSlot: "account-a",
    agentId: "human-b",
    nonce: "nonce-shared",
    entries: [{ id: "pending-human-b", kind: "message", clientNonce: "nonce-shared" }],
    phase: "dispatching",
  }), true);
  assert.equal(controller.reconcileEcho({
    accountSlot: "account-a",
    agentId: "human-a",
    nonce: "nonce-shared",
    echoedNonce: "nonce-shared",
  }), false, "same-account nonce ambiguity must fail closed instead of choosing iteration order");
  controller.setScope("account-a", "human-a");
  assert.equal(controller.getSnapshot().records.length, 1);
  controller.setScope("account-a", "human-b");
  assert.equal(controller.getSnapshot().records.length, 1);
  controller.dispose();
});

test("CONTRACT-FBCP-HUMAN-REMOTE-IDENTITY-SETTLEMENT-001 shipping Human send requires exact durable id and remote nonce before replacing pending state", () => {
  const source = readFileSync(new URL("../../production/ProductionRenderer.tsx", import.meta.url), "utf8");
  const acknowledgement = readFileSync(new URL("./conversation/workspace/acknowledgement.ts", import.meta.url), "utf8");
  assert.match(source, /authoritative\.clientNonce !== submission\.nonce/);
  assert.match(source, /authoritative\.id === "entry-0"/);
  assert.match(source, /authoritative\.id === `pending-\$\{submission\.nonce\}`/);
  assert.match(source, /correlated\.length !== 1/);
  assert.match(source, /echoedNonce: authoritative\.clientNonce/);
  assert.match(source, /if \(!reconciled\)/);
  assert.match(acknowledgement, /input\.echoedNonce == null \|\| input\.echoedNonce\.length === 0/);
  assert.match(acknowledgement, /if \(matched != null\) return null/);
  assert.match(acknowledgement, /!recordMatches\(match\[1\], input\.echoedNonce\)/);
});

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

test("UNIT-TDRP-IV-ARTICLE-HIDDEN-SEARCH-001 hidden transcript details share the expanded presentation search corpus", () => {
  assert.equal(formatTranscriptToolCallName("ReadFileToolCall"), "Read File");
  assert.equal(formatTranscriptToolCallName("ToolCall"), "ToolCall");

  const thinkingText = findInChatSearchText({
    id: "thinking-1",
    kind: "thinking",
    text: "First line\nHidden needle",
  });
  assert.equal(thinkingText, "Thinking\nFirst line\nHidden needle");

  const toolText = findInChatSearchText({
    id: "tool-1",
    kind: "tool-call",
    name: "ReadFileToolCall",
    summary: "Outer hidden needle",
    toolResult: {
      kind: "file-edit",
      toolCallId: "tool-1",
      status: "success",
      path: "/tmp/example.ts",
      command: null,
      workingDirectory: "/tmp",
      summary: "Result hidden needle",
      output: "Not rendered while summary is present",
      diff: "@@ hidden needle diff",
      isStreaming: false,
      isBackground: false,
    },
  });
  assert.equal(toolText, [
    "Read File",
    "Outer hidden needle",
    "/tmp/example.ts",
    "success",
    "/tmp",
    "Result hidden needle",
    "@@ hidden needle diff",
  ].join("\n"));
  assert.equal(toolText.includes("Not rendered while summary is present"), false);

  const controller = createFindInChatController();
  controller.replaceEntries([
    { id: "thinking-1", kind: "thinking", text: "Hidden needle twice: hidden needle" },
    {
      id: "tool-1",
      kind: "tool-call",
      name: "ReadFileToolCall",
      summary: "Hidden needle",
    },
  ]);
  controller.setQuery("hidden needle");
  assert.deepEqual(controller.getSnapshot().matches, [
    { entryId: "thinking-1", occurrence: 0 },
    { entryId: "thinking-1", occurrence: 1 },
    { entryId: "tool-1", occurrence: 0 },
  ]);
  controller.dispose();

  const existing = new Set(["thinking-1"]);
  assert.equal(includeFindInChatDisclosure(existing, "thinking-1"), existing);
  const revealed = includeFindInChatDisclosure(existing, "tool-1");
  assert.notEqual(revealed, existing);
  assert.deepEqual([...revealed], ["thinking-1", "tool-1"]);
});

test("CONTRACT-TDRP-IV-ARTICLE-HIDDEN-REVEAL-001 find navigation expands hidden transcript owners before highlight refresh", () => {
  const transcriptSource = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const controllerSource = readFileSync(new URL("./conversation/workspace/find-in-chat-controller.ts", import.meta.url), "utf8");
  assert.match(transcriptSource, /data-entry-id=\{entry\.id\} data-kind="tool-call"/);
  assert.match(transcriptSource, /data-entry-id=\{entry\.id\} data-kind="thinking"/);
  assert.match(transcriptSource, /revealFindEntryRef\.current\(entryId, disclosureKind\)/);
  assert.match(transcriptSource, /setExpandedThinking\(\(current\) => includeFindInChatDisclosure\(current, entryId\)\)/);
  assert.match(transcriptSource, /setExpandedToolCalls\(\(current\) => includeFindInChatDisclosure\(current, entryId\)\)/);
  assert.match(transcriptSource, /\[expandedThinking, expandedToolCalls, transcriptHandleRef\]/);
  assert.match(transcriptSource, /preview\.length > 0 && !expanded/);
  assert.match(controllerSource, /entry\.kind === "thinking"/);
  assert.match(controllerSource, /entry\.kind === "tool-call"/);
  assert.match(controllerSource, /toolResultSearchText\(toolCall\.toolResult\)/);
  assert.match(controllerSource, /from "\.\/tool-call-label\.ts"/);
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



test("accepted-send editor clear preserves user-owned focus outside the composer", async () => {
  const source = await readFile(new URL("./conversation/workspace/rich-text-editor.tsx", import.meta.url), "utf8");
  assert.match(source, /const preserveEditorFocus = editor\.view\.hasFocus\(\);/);
  assert.match(source, /if \(preserveEditorFocus\) clear\.focus\("end"\);/);
  assert.doesNotMatch(source, /clearContent\(false\)\.focus\("end"\)\.run\(\)/);
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


test("composer stash atomically exchanges rich drafts and prepared attachments per scope", async () => {
  const writes: unknown[] = [];
  const persistence: ComposerDraftPersistence = {
    async read() { return null; },
    async write(_accountSlot, value) { writes.push(value); },
    async clear() {},
  };
  const store = createComposerDraftStateStore(persistence);
  await store.restore("account-1");
  const first = {
    prompt: "first",
    richText: "{\"type\":\"doc\",\"content\":[]}",
    replyToId: "reply-1",
    attachments: [{ path: "/tmp/a.pdf", name: "a.pdf" }],
  };
  const second = {
    prompt: "second",
    attachments: [{ path: "/tmp/b.png", name: "b.png" }],
  };
  store.setDraft("conversation:topic:child", first);
  assert.equal(store.exchangeDraft("conversation:topic:child"), true);
  assert.equal(store.snapshotsFor("conversation:topic:child").get().draft, null);
  assert.deepEqual(store.snapshotsFor("conversation:topic:child").get().stash, first);
  store.setDraft("conversation:topic:child", second);
  assert.equal(store.exchangeDraft("conversation:topic:child"), true);
  assert.deepEqual(store.snapshotsFor("conversation:topic:child").get().draft, first);
  assert.deepEqual(store.snapshotsFor("conversation:topic:child").get().stash, second);
  assert.equal(store.removeStashIfMatches("conversation:topic:child", first), false, "stale send completion must not remove a newer stash");
  assert.deepEqual(store.snapshotsFor("conversation:topic:child").get().stash, second);
  assert.equal(store.removeStashIfMatches("conversation:topic:child", second), true);
  assert.equal(store.snapshotsFor("conversation:topic:child").get().stash, null);
  await flush();
  assert.ok(writes.length > 0, "draft mutations remain owned by the existing persistence store");
  store.dispose();
});

test("composer stash revalidates before restore and child-scope cleanup removes draft plus stash", async () => {
  const persistence: ComposerDraftPersistence = {
    async read() { return null; },
    async write() {},
    async clear() {},
  };
  const store = createComposerDraftStateStore(persistence);
  await store.restore("account-1");
  const scope = "conversation:topic:child";
  store.setDraft(scope, { prompt: "stashed", attachments: [{ path: "/tmp/file", name: "file" }] });
  assert.equal(store.exchangeDraft(scope), true);
  store.setDraft(scope, { prompt: "current", attachments: [] });
  assert.equal(store.exchangeDraft(scope, () => false), false, "invalid stash must not displace current composer state");
  assert.equal(store.snapshotsFor(scope).get().draft?.prompt, "current");
  assert.equal(store.snapshotsFor(scope).get().stash?.prompt, "stashed");
  store.clearScope(scope);
  assert.deepEqual(store.snapshotsFor(scope).get(), { draft: null, recovery: null, stash: null });
  assert.equal(store.canExchangeDraft(scope), false);
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

test("CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001 transcript external links separate open target from typed copy payload", () => {
  assert.deepEqual(projectTranscriptExternalLink("https://example.com/docs?q=1"), {
    href: "https://example.com/docs?q=1",
    copyText: "https://example.com/docs?q=1",
    copyLabel: "Copy Link",
    tooltip: null,
  });
  assert.deepEqual(projectTranscriptExternalLink("mailto:reader%2Bnotes@example.com?subject=Ignored"), {
    href: "mailto:reader+notes@example.com",
    copyText: "reader+notes@example.com",
    copyLabel: "Copy Email",
    tooltip: null,
  });
  assert.equal(projectTranscriptExternalLink("javascript:alert(1)"), null);
  assert.equal(projectTranscriptExternalLink("file:///tmp/secret.md"), null);
  assert.equal(projectTranscriptExternalLink("../relative.md"), null);
  assert.equal(projectTranscriptExternalLink("mailto:%ZZ"), null);
  assert.equal(
    projectTranscriptExternalLink("https://example.com/docs", "Documentation").tooltip,
    "https://example.com/docs",
  );
  assert.equal(
    projectTranscriptExternalLink("https://example.com/docs", "https://example.com/docs").tooltip,
    null,
  );
  assert.equal(
    projectTranscriptExternalLink("mailto:reader@example.com", "Contact us").tooltip,
    "reader@example.com",
  );
  assert.equal(
    projectTranscriptExternalLink("mailto:reader@example.com", "reader@example.com").tooltip,
    null,
  );

  const transcript = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const actions = readFileSync(new URL("./conversation/cards/transcript-card/message-actions.tsx", import.meta.url), "utf8");
  assert.match(transcript, /projectTranscriptExternalLink\(mark\.attrs\.href, node\.text \?\? undefined\)/);
  assert.match(transcript, /data-transcript-copy-label=\{external\.copyLabel\}/);
  assert.match(transcript, /data-transcript-copy-text=\{external\.copyText\}/);
  assert.match(transcript, /title=\{external\.tooltip \?\? undefined\}/);
  assert.match(transcript, /projectTranscriptInlineCopyTarget\(event\.target\)/);
  assert.match(transcript, /inlineCopy\?\.label \?\? "Copy"/);
  assert.match(transcript, /\(\(\?:https\?:\\\/\\\/\|mailto:\)\[\^\\s\)\]\+\)/);
  assert.match(actions, /"Copy Link" \| "Copy Email" \| "Copy Text"/);
  assert.match(actions, /label !== "Copy Link" && label !== "Copy Email" && label !== "Copy Text"/);
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


test("UNIT-TDRP-MATH-RESOURCE-BOUND-001 oversized assistant math fails before runtime loading or parsing", async () => {
  const cache = createAssistantMathMarkupCache({ maxExpressionLength: 4 });
  let loads = 0;
  const loader = async (): Promise<KatexRuntime> => {
    loads += 1;
    return {
      renderToString(expression) {
        return `<span>${expression}</span>`;
      },
    };
  };

  await assert.rejects(
    () => cache.load(loader, "12345", false),
    /exceeds the 4-code-unit parsing limit/,
  );
  assert.equal(loads, 0, "oversized untrusted math must be rejected before loading KaTeX");
});

test("UNIT-TDRP-MATH-RESOURCE-BOUND-002 assistant math cache evicts least-recently-used markup under a byte budget", async () => {
  const cache = createAssistantMathMarkupCache({ maxBytes: 500 });
  let loads = 0;
  const loader = async (): Promise<KatexRuntime> => {
    loads += 1;
    return {
      renderToString(expression, options) {
        return `<span>${options.displayMode ? "D" : "I"}:${expression}</span>`;
      },
    };
  };

  assert.equal(await cache.load(loader, "a", false), "<span>I:a</span>");
  assert.equal(await cache.load(loader, "a", false), "<span>I:a</span>");
  assert.equal(loads, 1, "a cache hit must reuse the resolved markup and refresh its LRU position");

  assert.equal(await cache.load(loader, "b", false), "<span>I:b</span>");
  assert.equal(loads, 2);
  assert.equal(await cache.load(loader, "a", false), "<span>I:a</span>");
  assert.equal(loads, 3, "the oldest entry must be rendered again after budget eviction");
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


test("transcript rich links remain behind the canonical URL owner", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  assert.match(source, /openExternal\?\: TranscriptExternalLinkOpener/);
  assert.match(source, /event\.preventDefault\(\); if \(!transcriptSelectionBlocksActivation\(\)\) openExternal\(external\.href\);/);
  assert.match(source, /messageUrlCards\.openExternal\(url\)/);
  assert.doesNotMatch(source, /assistant-link-[\s\S]{0,700}target="_blank"/);
  assert.match(source, /external == null \|\| openExternal == null \? current/);
  assert.match(source, /selection != null && !selection\.isCollapsed && selection\.toString\(\)\.length > 0/);
});


test("CONTRACT-TDRP-IV-MESSAGE-CONTEXT-MENU-CANONICAL-001 all transcript message-action seams consume the source-neutral canonical menu owner", () => {
  const transcript = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const cardActions = readFileSync(new URL("./conversation/cards/transcript-card/message-actions.tsx", import.meta.url), "utf8");
  const floating = readFileSync(new URL("../ui/sand-floating-primitives.tsx", import.meta.url), "utf8");
  for (const source of [transcript, cardActions]) {
    assert.match(source, /SandContextMenu/);
    assert.match(source, /SandMenuRoot/);
    assert.match(source, /SandMenuContent/);
    assert.match(source, /SandMenuItem/);
    assert.match(source, /shouldOpen=\{\(event\) => \{/);
    assert.match(source, /isMessageContextTargetExcluded\(event\.target\)/);
    assert.doesNotMatch(source, /<button[^>]*role="menuitem"/);
    assert.doesNotMatch(source, /document\.addEventListener\("pointerdown"/);
  }
  assert.match(floating, /readonly shouldOpen\?: \(event: ReactMouseEvent\) => boolean/);
  assert.match(floating, /<MenuContext\.Provider value=\{menu\}>/);
  assert.match(floating, /<SandMenuContent ariaLabel=\{ariaLabel\}/);
  assert.match(floating, /function mergeRefs<T>\(\.\.\.refs: readonly \(Ref<T> \| undefined\)\[\]\)/);
  assert.match(floating, /typeof ref === "function"\) ref\(node\)/);
  assert.match(floating, /ref\.current = node/);
});

test("CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001 content replacement invalidates stale transcript pointer actions", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  assert.match(source, /pointerActivationRevisionRef = useRef\(0\)/);
  assert.match(source, /pointerActivationIntentRef = useRef<\{ revision: number; target: HTMLElement; pointerId: number; startX: number; startY: number \} \| null>\(null\)/);
  assert.match(source, /pointerActivationRevisionRef\.current \+= 1;[\s\S]{0,100}pointerActivationIntentRef\.current = null;[\s\S]{0,220}\}, \[entries, transcriptHandleRef\]\);/);
  assert.match(source, /onPointerDownCapture=\{\(event\) => \{[\s\S]{0,700}revision: pointerActivationRevisionRef\.current, target/);
  assert.match(source, /window\.addEventListener\("blur", retirePointerActivationIntent\)/);
  assert.match(source, /document\.addEventListener\("visibilitychange", handleVisibilityChange\)/);
  assert.match(source, /document\.visibilityState !== "visible"[\s\S]{0,120}retirePointerActivationIntent\(\)/);
  assert.match(source, /onBlurCapture=\{\(event\) => \{[\s\S]{0,220}transcriptRef\.current\?\.contains\(next\)[\s\S]{0,120}pointerActivationIntentRef\.current = null/);
  assert.match(source, /onPointerCancelCapture=\{\(\) => \{[\s\S]{0,120}pointerActivationIntentRef\.current = null/);
  assert.match(source, /onPointerLeave=\{\(\) => \{[\s\S]{0,120}pointerActivationIntentRef\.current = null/);
  assert.match(source, /onClickCapture=\{\(event\) => \{[\s\S]{0,120}event\.detail === 0[\s\S]{0,420}transcriptSelectionBlocksActivation\(\)[\s\S]{0,260}pointerActivationIntentRef\.current = null[\s\S]{0,180}event\.preventDefault\(\);[\s\S]{0,100}event\.stopPropagation\(\);[\s\S]{0,520}intent\?\.revision === pointerActivationRevisionRef\.current && intent\.target === target/);
  assert.match(source, /closest<HTMLElement>\("a\[href\], button"\)/);
  assert.match(source, /TRANSCRIPT_POINTER_ACTIVATION_DRAG_THRESHOLD = 4/);
  assert.match(source, /onPointerMoveCapture=\{\(event\) => \{[\s\S]{0,260}intent\.pointerId !== event\.pointerId[\s\S]{0,360}Math\.hypot\(event\.clientX - intent\.startX, event\.clientY - intent\.startY\) > TRANSCRIPT_POINTER_ACTIVATION_DRAG_THRESHOLD[\s\S]{0,180}pointerActivationIntentRef\.current = null/);
});


test("CONTRACT-TDRP-MEDIAVIEW-WINDOW-CHROME-REPLACEMENT-001 media preview replaces detached OS window chrome with one accessible in-app dialog", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  const styles = readFileSync(new URL("./conversation/workspace/view.css", import.meta.url), "utf8");
  assert.match(source, /aria-modal="true"/);
  assert.match(source, /ref=\{viewerRef\} role="dialog"/);
  assert.match(source, /import \{ SandIconButton \} from "\.\.\/\.\.\/\.\.\/ui\/sand-kit-primitives"/);
  assert.match(source, /<SandIconButton aria-label="Close media preview"[\s\S]{0,240}icon="close"[\s\S]{0,240}onClick=\{onClose\}[\s\S]{0,160}variant="ghost"/);
  assert.match(source, /event\.key === "Escape"[\s\S]{0,160}onClose\(\)/);
  assert.match(source, /const previousOverflow = document\.body\.style\.overflow;[\s\S]{0,140}document\.body\.style\.overflow = "hidden"/);
  assert.match(source, /document\.body\.style\.overflow = previousOverflow;[\s\S]{0,100}restoreFocus\(\)/);
  assert.match(source, /data-media-source=\{attachment\.path\}/);
  assert.match(source, /trigger == null \|\| !trigger\.isConnected[\s\S]{0,420}querySelectorAll<HTMLButtonElement>\("button\[data-media-source\]"\)[\s\S]{0,220}candidate\.dataset\.mediaSource === source/);
  assert.match(source, /requestAnimationFrame\(focusCurrentTrigger\)/);
  assert.match(styles, /\.sand-message-action-anchor--menu-open \{ z-index: 5001; \}/);
  assert.match(styles, /\.sand-message-hover-actions \{[^}]*z-index: 5001;/);
  assert.doesNotMatch(source, /BrowserWindow|window\.(?:minimize|maximize|unmaximize|restore)\s*\(/);
  assert.match(styles, /\.sand-media-viewer \{ position: fixed; inset: 0;[\s\S]{0,180}overflow: hidden;/);
  assert.match(styles, /\.sand-media-viewer__close \{ pointer-events: auto; \}/);
  const closeRule = styles.match(/\.sand-media-viewer__close \{[^}]*\}/)?.[0] ?? "";
  assert.doesNotMatch(closeRule, /#[0-9a-f]{3,8}|color\s*:|background\s*:|border\s*:|outline\s*:/i);
  const primitiveStyles = readFileSync(new URL("../ui/sand-kit-primitives.css", import.meta.url), "utf8");
  assert.match(primitiveStyles, /\.sand-kit-icon-button:focus-visible[\s\S]{0,120}var\(--cursor-stroke-focused\)/);
  assert.match(styles, /\.sand-media-viewer__image \{[\s\S]{0,100}max-width: 92vw;[\s\S]{0,100}max-height: calc\(100vh - 145px\)/);
});

test("CONTRACT-TDRP-MEDIAVIEW-NATIVE-PLAYBACK-001 canonical media preview delegates video playback to Chromium without a second renderer owner", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  assert.match(source, /const videoRef = useRef<HTMLVideoElement \| null>\(null\);/);
  const videoMarkup = source.match(/<video aria-label=\{caption\.length > 0 \? caption : "Media preview"\}[\s\S]*?src=\{source\} \/>/)?.[0] ?? "";
  assert.match(videoMarkup, /className="sand-media-viewer__image"/);
  assert.match(videoMarkup, /\bcontrols\b/);
  assert.match(videoMarkup, /\bplaysInline\b/);
  assert.match(videoMarkup, /preload="metadata"/);
  assert.match(videoMarkup, /ref=\{videoRef\}/);
  assert.doesNotMatch(videoMarkup, /controls=\{false\}/);
  assert.match(source, /const outgoingVideo = videoRef\.current;[\s\S]{0,320}outgoingVideo\.pause\(\);[\s\S]{0,160}setMedia\(null\)/);
  const cleanupStart = source.indexOf("useEffect(() => () => {");
  const cleanupEnd = source.indexOf("}, []);", cleanupStart);
  assert.ok(cleanupStart >= 0 && cleanupEnd > cleanupStart);
  const cleanupBlock = source.slice(cleanupStart, cleanupEnd);
  assert.match(cleanupBlock, /const activeVideo = videoRef\.current;/);
  assert.match(cleanupBlock, /persistMediaPlaybackPreferences\(activeVideo\);/);
  assert.match(cleanupBlock, /persistMediaPlaybackPosition\(activeVideo, videoPersistenceRef\.current\.source\);/);
  assert.match(cleanupBlock, /activeVideo\.pause\(\);/);
  assert.match(source, /event\.key === "Escape"[\s\S]{0,100}document\.fullscreenElement != null[\s\S]{0,140}onClose\(\)/);
  assert.match(source, /target\?\.closest\("video, audio, button, input, select, textarea, \[role='slider'\], \[contenteditable='true'\]"\) != null\) return;/);
  assert.match(source, /const onWheel = \(event: React\.WheelEvent<HTMLDivElement>\) => \{[\s\S]{0,100}media\?\.kind === "video"[\s\S]{0,60}return;/);
  assert.match(source, /const onPointerDown = \(event: React\.PointerEvent<HTMLDivElement>\) => \{[\s\S]{0,120}media\?\.kind === "video"[\s\S]{0,120}return;/);
  assert.match(source, /onDoubleClick=\{media\?\.kind === "video" \? undefined : fit\}/);
  assert.doesNotMatch(source, /RendererGL|RendererRhi|TelegramMediaViewer|TelegramVideo/);
});

test("CONTRACT-TDRP-MEDIAVIEW-PLAYBACK-PERSISTENCE-001 canonical media preview persists source-neutral volume and resumable position without leaking source paths", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  assert.match(source, /MEDIA_PLAYBACK_PREFERENCES_KEY = "fabushi\.mediaViewer\.playback\.v1"/);
  assert.match(source, /MEDIA_PLAYBACK_POSITION_PREFIX = "fabushi\.mediaViewer\.position\.v1\."/);
  assert.match(source, /function mediaPlaybackSourceKey\(source: string\)[\s\S]{0,380}Math\.imul\(hash, 0x01000193\)[\s\S]{0,180}MEDIA_PLAYBACK_POSITION_PREFIX/);
  assert.doesNotMatch(source, /MEDIA_PLAYBACK_POSITION_PREFIX[^\n]{0,300}storage\.(?:setItem|getItem)\([^\n]*source/);
  assert.match(source, /persistMediaPlaybackPreferences\(outgoingVideo\);[\s\S]{0,160}persistMediaPlaybackPosition\(outgoingVideo, videoPersistenceRef\.current\.source\)[\s\S]{0,100}outgoingVideo\.pause\(\)/);
  assert.match(source, /onLoadedMetadata=\{restoreVideoPlaybackState\}/);
  assert.match(source, /onPause=\{persistVideoPlaybackState\}/);
  assert.match(source, /onTimeUpdate=\{persistVideoPlaybackProgress\}/);
  assert.match(source, /onVolumeChange=\{persistVideoPlaybackState\}/);
  assert.match(source, /onEnded=\{clearCompletedVideoPlaybackPosition\}/);
  assert.match(source, /MEDIA_POSITION_WRITE_INTERVAL_MS = 1_000/);
  assert.match(source, /readMediaPlaybackPosition\(sourceKey, video\.duration\)[\s\S]{0,100}video\.currentTime = position/);
  assert.match(source, /persistMediaPlaybackPosition\(event\.currentTarget, videoPersistenceRef\.current\.source, true\)/);
  assert.match(source, /catch \{[\s\S]{0,120}Playback remains functional when persistent storage is unavailable/);
});

test("CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001 media replacement releases stale viewer pointer ownership before the new resource settles", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  const replacementStart = source.indexOf("setMedia(null);");
  const resolverStart = source.indexOf("void resolveWithSingleRetry(resolveMedia, current.path)", replacementStart);
  assert.ok(replacementStart >= 0 && resolverStart > replacementStart);
  const replacementBlock = source.slice(replacementStart, resolverStart);
  assert.match(replacementBlock, /setTransform\(\{ scale: MIN_ZOOM, x: 0, y: 0 \}\);/);
  assert.match(replacementBlock, /activePointerId = pointerRef\.current\.id;[\s\S]{0,260}viewer\?\.hasPointerCapture\(activePointerId\)[\s\S]{0,120}viewer\.releasePointerCapture\(activePointerId\)/);
  assert.match(replacementBlock, /pointerRef\.current = \{ id: null, startX: 0, startY: 0, originX: 0, originY: 0, moved: false \};/);
  const cleanupStart = source.indexOf("useEffect(() => () => {");
  const cleanupEnd = source.indexOf("}, []);", cleanupStart);
  assert.ok(cleanupStart >= 0 && cleanupEnd > cleanupStart);
  const cleanupBlock = source.slice(cleanupStart, cleanupEnd);
  const releaseIndex = cleanupBlock.indexOf("viewer.releasePointerCapture(activePointerId)");
  const clearIndex = cleanupBlock.indexOf("pointerRef.current.id = null");
  assert.ok(releaseIndex >= 0 && clearIndex > releaseIndex);
  assert.match(source, /event\.target\.closest\("button, a\[href\], input, select, textarea, \[role='button'\]"\) != null\) return;/);
  assert.match(source, /onLostPointerCapture=\{onLostPointerCapture\}/);
  assert.match(source, /const onLostPointerCapture = \(event: React\.PointerEvent<HTMLDivElement>\) => \{[\s\S]{0,140}pointerRef\.current\.id === event\.pointerId[\s\S]{0,80}pointerRef\.current\.id = null/);
  assert.match(source, /ref=\{viewerRef\} role="dialog"/);
  assert.match(source, /if \(pointer\.id !== event\.pointerId\) return;/);
});


test("CONTRACT-TDRP-IV-VIEW-CODE-COPY-SANITIZED-001 code copy crosses only the clipboard writer boundary", async () => {
  const writes: string[] = [];
  assert.equal(await copyTranscriptCodeText("const x = 1;\n", {
    async writeText(text) {
      writes.push(text);
    },
  }), true);
  assert.deepEqual(writes, ["const x = 1;\n"]);
  assert.equal(await copyTranscriptCodeText("secret", null), false);
  assert.equal(await copyTranscriptCodeText("secret", {
    async writeText() {
      throw new Error("clipboard denied");
    },
  }), false);

  const source = readFileSync(new URL("./conversation/workspace/code-copy.ts", import.meta.url), "utf8");
  assert.doesNotMatch(source, /session|window|controller|host|coordinator/i);
});


test("UNIT-TDRP-IV-VIEW-CONTROL-WHEEL-ZOOM-001 accumulates deterministic control-wheel steps", () => {
  assert.equal(normalizeWheelZoomDelta(-120, 0), 120);
  assert.equal(normalizeWheelZoomDelta(3, 1), -48);
  assert.equal(normalizeWheelZoomDelta(1, 2), -100);
  assert.deepEqual(accumulateWheelZoomSteps(0, 60), { remainder: 60, steps: 0 });
  assert.deepEqual(accumulateWheelZoomSteps(60, 60), { remainder: 0, steps: 1 });
  assert.deepEqual(accumulateWheelZoomSteps(0, -250), { remainder: -10, steps: -2 });
  assert.deepEqual(accumulateWheelZoomSteps(Number.NaN, 120), { remainder: 0, steps: 1 });
});


test("media missing-resource recovery is bounded to one retry", async () => {
  let attempts = 0;
  const recovered = await resolveWithSingleRetry(async () => {
    attempts += 1;
    return attempts === 1 ? null : { kind: "ok" };
  }, "media://recover");
  assert.deepEqual(recovered, { kind: "ok" });
  assert.equal(attempts, 2);

  attempts = 0;
  const missing = await resolveWithSingleRetry(async () => {
    attempts += 1;
    return null;
  }, "media://missing");
  assert.equal(missing, null);
  assert.equal(attempts, 2);

  attempts = 0;
  const afterFailure = await resolveWithSingleRetry(async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("transient");
    return { kind: "ok" };
  }, "media://transient");
  assert.deepEqual(afterFailure, { kind: "ok" });
  assert.equal(attempts, 2);

  attempts = 0;
  await assert.rejects(() => resolveWithSingleRetry(async () => {
    attempts += 1;
    throw new Error(`failure-${attempts}`);
  }, "media://broken"), /failure-2/);
  assert.equal(attempts, 2);
});


test("UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001 rich-content horizontal scroll restores only compatible owners", () => {
  const snapshot = captureHorizontalScroll("message-1:code:0:typescript", 240);
  assert.deepEqual(snapshot, { ownerId: "message-1:code:0:typescript", offset: 240 });
  assert.equal(restoreHorizontalScrollOffset(snapshot, snapshot.ownerId, 1_000, 400), 240);
  assert.equal(restoreHorizontalScrollOffset(snapshot, snapshot.ownerId, 500, 400), 100);
  assert.equal(restoreHorizontalScrollOffset(snapshot, "message-2:code:0:typescript", 1_000, 400), 0);
  assert.deepEqual(captureHorizontalScroll("owner", -12), { ownerId: "owner", offset: 0 });
  assert.equal(clampHorizontalScrollOffset(Number.POSITIVE_INFINITY, 600, 500), 0);
  assert.equal(clampHorizontalScrollOffset(20, 100, 200), 0);
});

test("UNIT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-LIFECYCLE-001 horizontal pointer intent commits only after directional arbitration", () => {
  const initial = beginHorizontalScrollPointer(7, 100, 20);
  assert.deepEqual(initial, { pointerId: 7, startX: 100, startY: 20, lastX: 100, lastY: 20, active: false });

  const pending = updateHorizontalScrollPointer(initial, 7, 102, 21);
  assert.equal(pending.decision, "pending");
  assert.equal(pending.deltaX, 0);
  assert.equal(pending.gesture?.active, false);

  const horizontal = updateHorizontalScrollPointer(pending.gesture, 7, 110, 22);
  assert.equal(horizontal.decision, "horizontal");
  assert.equal(horizontal.deltaX, -10);
  assert.equal(horizontal.gesture?.active, true);

  const continued = updateHorizontalScrollPointer(horizontal.gesture, 7, 116, 22);
  assert.equal(continued.decision, "horizontal");
  assert.equal(continued.deltaX, -6);

  const vertical = updateHorizontalScrollPointer(beginHorizontalScrollPointer(8, 20, 20), 8, 22, 32);
  assert.equal(vertical.decision, "vertical");
  assert.equal(vertical.gesture, null);

  const mismatched = updateHorizontalScrollPointer(initial, 99, 200, 200);
  assert.equal(mismatched.decision, "ignored");
  assert.equal(mismatched.gesture, initial);
});

test("UNIT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001 wheel gestures keep their first resolved axis until idle settlement", () => {
  const first = updateHorizontalScrollWheelLock(null, 18, 4, 100);
  assert.equal(first.axis, "horizontal");
  assert.deepEqual(first.lock, { axis: "horizontal", expiresAtMs: 260 });

  const jitter = updateHorizontalScrollWheelLock(first.lock, 1, 24, 180);
  assert.equal(jitter.axis, "horizontal");
  assert.deepEqual(jitter.lock, { axis: "horizontal", expiresAtMs: 340 });

  const settled = updateHorizontalScrollWheelLock(jitter.lock, 1, 24, 501);
  assert.equal(settled.axis, "vertical");
  assert.deepEqual(settled.lock, { axis: "vertical", expiresAtMs: 661 });

  assert.deepEqual(updateHorizontalScrollWheelLock(null, 0, 0, 700), { axis: null, lock: null });
});

test("UNIT-TDRP-IV-ARTICLE-WHEEL-DELTA-NORMALIZATION-001 wheel line and page units project into pixel-space scroll deltas", () => {
  assert.deepEqual(normalizeHorizontalScrollWheelDelta(2, -3, 0, 640), { x: 2, y: -3 });
  assert.deepEqual(normalizeHorizontalScrollWheelDelta(2, -3, 1, 640), { x: 32, y: -48 });
  assert.deepEqual(normalizeHorizontalScrollWheelDelta(0.5, -1, 2, 640), { x: 320, y: -640 });
});

test("CONTRACT-TDRP-IV-VIEW-STATIC-DATE-WORK-001 canonical transcript timestamps require no background formatted-date timer", () => {
  const transcript = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const timeline = readFileSync(new URL("./conversation/cards/timeline-event.tsx", import.meta.url), "utf8");
  const permission = readFileSync(new URL("./conversation/cards/permission-request/view.tsx", import.meta.url), "utf8");

  assert.match(transcript, /new Date\(entry\.timestampMs\)\.toLocaleString\(\)/);
  assert.match(timeline, /new Intl\.DateTimeFormat\([^\n]+hour:\s*"numeric"[\s\S]{0,120}minute:\s*"2-digit"/);
  assert.match(permission, /data-timestamp-ms=\{timestampMs\}/);

  for (const source of [transcript, timeline, permission]) {
    assert.doesNotMatch(source, /\bsetInterval\b/);
    assert.doesNotMatch(source, /\bRelativeTimeFormat\b/);
  }
});

test("CONTRACT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001 rich-content overflow consumes only horizontally locked wheel gestures", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  assert.match(source, /if \(event\.ctrlKey\) \{\s*wheelLockRef\.current = null;\s*return;/);
  assert.match(source, /normalizeHorizontalScrollWheelDelta\([\s\S]{0,220}event\.deltaMode/);
  assert.match(source, /updateHorizontalScrollWheelLock\([\s\S]{0,220}event\.timeStamp/);
  assert.match(source, /if \(update\.axis !== "horizontal"[\s\S]{0,180}return;/);
  assert.match(source, /event\.currentTarget\.scrollLeft \+ delta\.x/);
  assert.match(source, /event\.preventDefault\(\)/);
});

test("CONTRACT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-BALANCE-001 rich-content regions balance touch-like pointer ownership", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const css = readFileSync(new URL("./conversation/workspace/view.css", import.meta.url), "utf8");
  assert.match(source, /beginHorizontalScrollPointer\(event\.pointerId, event\.clientX, event\.clientY\)/);
  assert.match(source, /updateHorizontalScrollPointer\(pointerGestureRef\.current, event\.pointerId, event\.clientX, event\.clientY\)/);
  assert.match(source, /setPointerCapture\(event\.pointerId\)/);
  assert.match(source, /releasePointerCapture\(gesture\.pointerId\)/);
  assert.match(source, /onPointerCancel=\{\(event\) => retirePointerGesture\(event\.pointerId\)\}/);
  assert.match(source, /onPointerUp=\{\(event\) => retirePointerGesture\(event\.pointerId\)\}/);
  assert.match(source, /onLostPointerCapture=\{\(event\) => retirePointerGesture\(event\.pointerId\)\}/);
  assert.match(css, /touch-action:\s*pan-y/);
});

test("CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001 assistant code and table scroll regions bind to canonical message identity", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const sendMessageTextSource = readFileSync(new URL("./conversation/cards/transcript-card/views/send-message-text.tsx", import.meta.url), "utf8");
  assert.match(source, /<AssistantMessageContent[\s\S]{0,700}ownerId=\{entry\.id\}[\s\S]{0,200}text=\{entry\.text\}/);
  assert.match(sendMessageTextSource, /<AssistantMessageContent[\s\S]{0,400}ownerId=\{entry\.id\}[\s\S]{0,120}text=\{message\.content\}/);
  assert.match(source, /const tableOwnerId = `\$\{ownerId\}:table:\$\{JSON\.stringify\(block\.headers\)\}`;/);
  assert.match(source, /restoreHorizontalScrollOffset\(snapshotRef\.current, ownerId, region\.scrollWidth, region\.clientWidth\)/);
  assert.match(source, /className="sand-code-scroll"/);
  assert.match(source, /className="fabushi-rich-content-scroll-region"/);
});


test("UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001 assistant content projection preserves only compatible committed identities", () => {
  const paragraph = (value: string): AssistantProjectionCandidate => ({
    path: "content:0:text:0",
    kind: "paragraph",
    structuralIdentity: "paragraph",
    revision: { mode: "text-prefix", value },
  });
  const code = (value: string): AssistantProjectionCandidate => ({
    path: "content:1:code",
    kind: "code",
    structuralIdentity: "code:typescript",
    revision: { mode: "text-prefix", value },
  });

  const initial = reconcileAssistantContentProjection(null, {
    ownerId: "message-1",
    streaming: true,
    candidates: [paragraph("Hello")],
  });
  assert.equal(initial.mode, "initial");
  assert.equal(initial.generation, 0);
  const paragraphKey = initial.entries[0]?.key;
  assert.ok(paragraphKey);

  const appended = reconcileAssistantContentProjection(initial, {
    ownerId: "message-1",
    streaming: true,
    candidates: [paragraph("Hello world"), code("const value")],
  });
  assert.equal(appended.mode, "patch");
  assert.equal(appended.generation, 0);
  assert.equal(appended.entries[0]?.key, paragraphKey);
  const codeKey = appended.entries[1]?.key;
  assert.ok(codeKey);

  const settled = reconcileAssistantContentProjection(appended, {
    ownerId: "message-1",
    streaming: false,
    candidates: [paragraph("Hello world!"), code("const value = 1;")],
  });
  assert.equal(settled.mode, "patch");
  assert.equal(settled.entries[0]?.key, paragraphKey);
  assert.equal(settled.entries[1]?.key, codeKey);

  const settledBase = reconcileAssistantContentProjection(null, {
    ownerId: "settled-message",
    streaming: false,
    candidates: [paragraph("Complete")],
  });
  const settledTailAppend = reconcileAssistantContentProjection(settledBase, {
    ownerId: "settled-message",
    streaming: false,
    candidates: [paragraph("Complete"), code("const late = true;")],
  });
  assert.equal(settledTailAppend.mode, "replace");
  assert.equal(settledTailAppend.generation, 1);

  const rewritten = reconcileAssistantContentProjection(settled, {
    ownerId: "message-1",
    streaming: false,
    candidates: [paragraph("Rewritten text"), code("const value = 1;")],
  });
  assert.equal(rewritten.mode, "replace");
  assert.equal(rewritten.generation, 1);
  assert.notEqual(rewritten.entries[0]?.key, paragraphKey);
  assert.notEqual(rewritten.entries[1]?.key, codeKey);

  const shortened = reconcileAssistantContentProjection(rewritten, {
    ownerId: "message-1",
    streaming: false,
    candidates: [paragraph("Rewritten text")],
  });
  assert.equal(shortened.mode, "replace");
  assert.equal(shortened.generation, 2);

  const newOwner = reconcileAssistantContentProjection(shortened, {
    ownerId: "message-2",
    streaming: false,
    candidates: [paragraph("Rewritten text")],
  });
  assert.equal(newOwner.mode, "replace");
  assert.equal(newOwner.ownerId, "message-2");
  assert.notEqual(newOwner.entries[0]?.key, shortened.entries[0]?.key);
});

test("UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001 list and table growth fails closed when an earlier unit changes", () => {
  const list = (values: readonly string[]): AssistantProjectionCandidate => ({
    path: "content:0:text:0",
    kind: "list",
    structuralIdentity: JSON.stringify({ ordered: false, start: null }),
    revision: { mode: "append-sequence", values },
  });
  assert.equal(areAssistantProjectionCandidatesCompatible(
    list(["item:open:first"]),
    list(["item:open:first", "item:open:second"]),
    true,
  ), true);
  assert.equal(areAssistantProjectionCandidatesCompatible(
    list(["item:open:first", "item:open:sec"]),
    list(["item:open:first", "item:open:second"]),
    true,
  ), true);
  assert.equal(areAssistantProjectionCandidatesCompatible(
    list(["item:open:first", "item:open:second"]),
    list(["item:open:changed", "item:open:second"]),
    true,
  ), false);
  assert.equal(areAssistantProjectionCandidatesCompatible(
    list(["item:open:first"]),
    { ...list(["item:open:first"]), structuralIdentity: JSON.stringify({ ordered: true, start: 1 }) },
    true,
  ), false);

  const table = (values: readonly string[]): AssistantProjectionCandidate => ({
    path: "content:0:text:1",
    kind: "table",
    structuralIdentity: JSON.stringify(["Name", "Value"]),
    revision: { mode: "append-sequence", values },
  });
  assert.equal(areAssistantProjectionCandidatesCompatible(
    table(["first\u001f1"]),
    table(["first\u001f1", "second\u001f2"]),
    true,
  ), true);
  assert.equal(areAssistantProjectionCandidatesCompatible(
    table(["first\u001f1", "second\u001f2"]),
    table(["changed\u001f1", "second\u001f2"]),
    true,
  ), false);
  assert.equal(areAssistantProjectionCandidatesCompatible(
    table(["first\u001f1"]),
    { ...table(["first\u001f1"]), structuralIdentity: JSON.stringify(["Changed", "Value"]) },
    true,
  ), false);
});

test("CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001 assistant projection patches compatible growth and remounts incompatible structure", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  const projectionSource = readFileSync(new URL("./conversation/workspace/assistant-content-projection.ts", import.meta.url), "utf8");
  assert.match(source, /reconcileAssistantContentProjection\(committedProjectionRef\.current/);
  assert.match(source, /useLayoutEffect\(\(\) => \{\s*committedProjectionRef\.current = projectedContent\.projection;/);
  assert.match(source, /key=\{projectionEntry\.key\}/);
  assert.doesNotMatch(source, /key=\{`code-\$\{index\}`\}/);
  assert.match(projectionSource, /allowGrowth && input\.candidates\.length > previous\.entries\.length/);
  assert.match(projectionSource, /previous\.ownerId === input\.ownerId/);
  assert.match(projectionSource, /mode: "replace"/);
  assert.match(projectionSource, /previous\.generation \+ 1/);
});

test("UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001 find highlight refresh supersedes and cancels stale frames", () => {
  const frames: Array<{ active: boolean; callback: () => void }> = [];
  const runtime = createFindHighlightRefreshRuntime((callback) => {
    const frame = { active: true, callback };
    frames.push(frame);
    return () => { frame.active = false; };
  });
  const settled: string[] = [];

  runtime.schedule(() => settled.push("first"));
  runtime.schedule(() => settled.push("second"));
  assert.equal(frames.length, 2);
  assert.equal(frames[0]?.active, false);
  frames[0]?.callback();
  assert.deepEqual(settled, []);
  frames[1]?.callback();
  assert.deepEqual(settled, ["second"]);

  runtime.schedule(() => settled.push("invalidated"));
  const invalidatedFrame = frames.at(-1);
  runtime.invalidate();
  assert.equal(invalidatedFrame?.active, false);
  invalidatedFrame?.callback();
  assert.deepEqual(settled, ["second"]);

  runtime.schedule(() => settled.push("disposed"));
  const disposedFrame = frames.at(-1);
  runtime.dispose();
  assert.equal(disposedFrame?.active, false);
  disposedFrame?.callback();
  runtime.schedule(() => settled.push("after-dispose"));
  assert.equal(frames.length, 4);
  assert.deepEqual(settled, ["second"]);

  let synchronousSettlements = 0;
  let staleSynchronousCancels = 0;
  const synchronousRuntime = createFindHighlightRefreshRuntime((callback) => {
    callback();
    return () => { staleSynchronousCancels += 1; };
  });
  synchronousRuntime.schedule(() => { synchronousSettlements += 1; });
  synchronousRuntime.invalidate();
  assert.equal(synchronousSettlements, 1);
  assert.equal(staleSynchronousCancels, 0);
});

test("CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001 find highlights cannot settle into a replaced or disposed transcript", () => {
  const source = readFileSync(new URL("./conversation/workspace/find-in-chat.tsx", import.meta.url), "utf8");
  const runtimeSource = readFileSync(new URL("./conversation/workspace/find-highlight-runtime.ts", import.meta.url), "utf8");
  assert.match(source, /createFindHighlightRefreshRuntime\(\)/);
  assert.match(source, /highlightRefreshRuntime\.schedule\(\(\) => \{/);
  assert.match(source, /controllerRef\.current\.getSnapshot\(\)/);
  assert.match(source, /applyFindHighlights\(transcriptContainerRef\.current/);
  assert.match(source, /highlightRefreshRuntime\.invalidate\(\);\s*controllerRef\.current = controller;\s*transcriptContainerRef\.current = transcriptContainer/);
  assert.match(source, /return \(\) => highlightRefreshRuntime\.invalidate\(\);/);
  assert.match(source, /return \(\) => clearFindHighlights\(\);/);
  assert.match(source, /highlightRefreshRuntime\.dispose\(\);/);
  assert.doesNotMatch(source, /requestAnimationFrame\(refresh\)/);
  assert.match(runtimeSource, /cancelPending\?\.\(\);/);
  assert.match(runtimeSource, /disposed \|\| generation !== scheduledGeneration/);
  assert.match(runtimeSource, /queueMicrotask/);
  assert.match(runtimeSource, /cancelAnimationFrame/);
});

test("UNIT-FBCP-HUMAN-DISPATCHING-SETTLEMENT-001 Human dispatching remains unsettled until remote identity settlement", () => {
  assert.equal(normalizeTranscriptDelivery("dispatching"), "dispatching");
  assert.equal(normalizeTranscriptDelivery("scheduled"), "scheduled");
  assert.equal(isTranscriptDeliveryBusy("dispatching"), true);
  assert.equal(isTranscriptDeliveryActionable("dispatching"), false);
  assert.equal(isTranscriptDeliveryBusy("sent"), false);
  assert.equal(isTranscriptDeliveryActionable("sent"), true);
});

test("CONTRACT-FBCP-HUMAN-DISPATCHING-SETTLEMENT-001 dispatching stays busy and cannot expose settled message actions", () => {
  const state = readFileSync(new URL("./conversation/workspace/transcript-delivery-state.ts", import.meta.url), "utf8");
  const model = readFileSync(new URL("./conversation/workspace/model.ts", import.meta.url), "utf8");
  const projection = readFileSync(new URL("../../production/model.ts", import.meta.url), "utf8");
  const transcript = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  assert.match(state, /candidate === "dispatching"/);
  assert.match(state, /candidate === "scheduled"/);
  assert.match(state, /delivery === "pending" \|\| delivery === "queued" \|\| delivery === "dispatching"/);
  assert.match(model, /TranscriptDelivery = TranscriptDeliveryState/);
  assert.match(projection, /normalizeTranscriptDelivery\(/);
  assert.match(transcript, /isTranscriptDeliveryActionable\(entry\.delivery\)/);
  assert.match(transcript, /isTranscriptDeliveryBusy\(entry\.delivery\)/);
});

test("UNIT-TDRP-IV-ARTICLE-FOLLOW-LATEST-001 transcript follow state stays pinned only near the latest message", () => {
  assert.equal(TRANSCRIPT_FOLLOW_LATEST_THRESHOLD_PX, 96);
  assert.equal(isTranscriptNearBottom({ scrollTop: 0, clientHeight: 600, scrollHeight: 600 }), true);
  assert.equal(isTranscriptNearBottom({ scrollTop: 304, clientHeight: 600, scrollHeight: 1_000 }), true);
  assert.equal(isTranscriptNearBottom({ scrollTop: 303, clientHeight: 600, scrollHeight: 1_000 }), false);
  assert.equal(isTranscriptNearBottom({ scrollTop: -20, clientHeight: 600, scrollHeight: 640 }), true);
});

test("CONTRACT-TDRP-IV-ARTICLE-FOLLOW-LATEST-LIFECYCLE-001 transcript follows appended content only while the user remains near the bottom", () => {
  const source = readFileSync(new URL("./conversation/workspace/transcript.tsx", import.meta.url), "utf8");
  assert.match(source, /const followLatestRef = useRef\(true\)/);
  assert.match(source, /if \(transcript == null \|\| !followLatestRef\.current\) return;[\s\S]{0,180}transcript\.scrollTop = Math\.max\(0, transcript\.scrollHeight - transcript\.clientHeight\)/);
  assert.match(source, /followLatestRef\.current = isTranscriptNearBottom\([\s\S]{0,240}scrollHeight: transcript\.scrollHeight/);
  assert.match(source, /addEventListener\("scroll", syncFollowLatest, \{ passive: true \}\)/);
  assert.match(source, /removeEventListener\("scroll", syncFollowLatest\)/);
});

test("UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001 derived transcript media follows a bounded visibility budget", () => {
  assert.equal(isVisibilityBoundDerivedMedia("image"), true);
  assert.equal(isVisibilityBoundDerivedMedia("video"), true);
  assert.equal(isVisibilityBoundDerivedMedia("audio"), false);
  assert.equal(shouldResolveDerivedMedia("image", false), false);
  assert.equal(shouldResolveDerivedMedia("video", true), true);
  assert.equal(shouldResolveDerivedMedia("audio", false), true);
  assert.equal(shouldResolveDerivedThumbnail(false, false), false);
  assert.equal(shouldResolveDerivedThumbnail(false, true), true);
  assert.equal(shouldResolveDerivedThumbnail(true, false), true);
  assert.equal(isDerivedMediaNearViewport({ top: 100, right: 220, bottom: 180, left: 120 }, 800, 600, DERIVED_MEDIA_PRELOAD_ROOT_MARGIN), true);
  assert.equal(isDerivedMediaNearViewport({ top: 1_241, right: 220, bottom: 1_300, left: 120 }, 800, 600, DERIVED_MEDIA_PRELOAD_ROOT_MARGIN), false);
  assert.equal(isDerivedMediaNearViewport({ top: 100, right: 1_180, bottom: 180, left: 1_100 }, 800, 600, DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN), true);
  assert.equal(isDerivedMediaNearViewport({ top: 100, right: 1_240, bottom: 180, left: 1_121 }, 800, 600, DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN), false);
  assert.equal(isDerivedMediaNearViewport({ top: 5_000, right: 5_100, bottom: 5_100, left: 5_000 }, 800, 600, "25%"), true);

  const previousObserver = globalThis.IntersectionObserver;
  const instances: FakeIntersectionObserver[] = [];
  class FakeIntersectionObserver {
    readonly root = null;
    readonly thresholds = [0];
    readonly observed = new Set<Element>();
    readonly callback: IntersectionObserverCallback;
    readonly options: IntersectionObserverInit | undefined;
    disconnected = false;
    constructor(callback: IntersectionObserverCallback, options?: IntersectionObserverInit) {
      this.callback = callback;
      this.options = options;
      instances.push(this);
    }
    observe(target: Element) { this.observed.add(target); }
    unobserve(target: Element) { this.observed.delete(target); }
    disconnect() { this.disconnected = true; this.observed.clear(); }
    takeRecords(): IntersectionObserverEntry[] { return []; }
  }
  Object.defineProperty(globalThis, "IntersectionObserver", { configurable: true, writable: true, value: FakeIntersectionObserver as unknown as typeof IntersectionObserver });
  try {
    const first = {} as Element;
    const second = {} as Element;
    const firstStates: boolean[] = [];
    const secondStates: boolean[] = [];
    const releaseFirst = observeDerivedMediaVisibility(first, DERIVED_MEDIA_PRELOAD_ROOT_MARGIN, (value) => firstStates.push(value));
    const releaseSecond = observeDerivedMediaVisibility(second, DERIVED_MEDIA_PRELOAD_ROOT_MARGIN, (value) => secondStates.push(value));
    assert.equal(instances.length, 1);
    assert.equal(instances[0]?.options?.rootMargin, DERIVED_MEDIA_PRELOAD_ROOT_MARGIN);
    instances[0]?.callback([
      { target: first, isIntersecting: true } as IntersectionObserverEntry,
      { target: second, isIntersecting: false } as IntersectionObserverEntry,
    ], instances[0] as unknown as IntersectionObserver);
    assert.deepEqual(firstStates, [true]);
    assert.deepEqual(secondStates, [false]);
    releaseFirst();
    assert.equal(instances[0]?.disconnected, false);
    releaseSecond();
    assert.equal(instances[0]?.disconnected, true);
  } finally {
    if (previousObserver == null) Reflect.deleteProperty(globalThis, "IntersectionObserver");
    else Object.defineProperty(globalThis, "IntersectionObserver", { configurable: true, writable: true, value: previousObserver });
  }
  assert.equal(DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN, "0px 320px");
});

test("CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001 attachment cards and filmstrip thumbnails release offscreen derived media without changing canonical metadata", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  const visibilitySource = readFileSync(new URL("./conversation/workspace/media-visibility.ts", import.meta.url), "utf8");
  assert.match(visibilitySource, /const visibilityBuckets = new Map/);
  assert.match(visibilitySource, /new IntersectionObserver/);
  assert.match(source, /initialDerivedMediaVisibility\(next, rootMargin\)/);
  assert.match(source, /observeDerivedMediaVisibility\(element, rootMargin, setIsNearViewport\)/);
  assert.match(source, /const shouldResolve = resolveMedia != null && supportedMediaKind && shouldResolveDerivedMedia\(kind, isNearViewport\);/);
  assert.match(source, /if \(!shouldResolve \|\| resolveMedia == null\) \{[\s\S]{0,120}setMedia\(null\);[\s\S]{0,120}setLoading\(false\);/);
  assert.match(source, /const shouldResolve = shouldResolveDerivedThumbnail\(isNearViewport, isActive\);/);
  assert.match(source, /<MediaThumbnail isActive=\{attachmentIndex === index\}/);
  assert.match(source, /<MediaCard[\s\S]{0,400}observe=\{observeMediaCard\}/);
  assert.match(source, /if \(isPreviewable\(kind\) && onOpen != null\) \{/);
  assert.match(source, /return <button aria-label="Media preview"[\s\S]{0,600}data-media-state=\{loading \? "loading" : media == null \? "unavailable" : "ready"\}[\s\S]{0,300}ref=\{observe\}[\s\S]{0,180}>\{preview\}<\/button>/);
  assert.doesNotMatch(source, /if \(media\?\.kind === "image" && onOpen != null\) return <button/);
  assert.doesNotMatch(source, /if \(media\?\.kind === "video" && onOpen != null\) return <button/);
  assert.match(source, /<MediaViewer attachments=\{mediaAttachments\}/);
});
