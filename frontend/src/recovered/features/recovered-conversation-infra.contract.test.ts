import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
import { DERIVED_MEDIA_PRELOAD_ROOT_MARGIN, DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN, isVisibilityBoundDerivedMedia, observeDerivedMediaVisibility, shouldResolveDerivedMedia, shouldResolveDerivedThumbnail } from "./conversation/workspace/media-visibility.ts";
import { beginHorizontalScrollPointer, captureHorizontalScroll, clampHorizontalScrollOffset, restoreHorizontalScrollOffset, updateHorizontalScrollPointer } from "./conversation/workspace/horizontal-scroll-state.ts";
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


test("CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001 media replacement releases stale viewer pointer ownership before the new resource settles", () => {
  const source = readFileSync(new URL("./conversation/workspace/media-viewer.tsx", import.meta.url), "utf8");
  const replacementStart = source.indexOf("setMedia(null);");
  const resolverStart = source.indexOf("void resolveWithSingleRetry(resolveMedia, current.path)", replacementStart);
  assert.ok(replacementStart >= 0 && resolverStart > replacementStart);
  const replacementBlock = source.slice(replacementStart, resolverStart);
  assert.match(replacementBlock, /setTransform\(\{ scale: MIN_ZOOM, x: 0, y: 0 \}\);/);
  assert.match(replacementBlock, /activePointerId = pointerRef\.current\.id;[\s\S]{0,260}viewer\?\.hasPointerCapture\(activePointerId\)[\s\S]{0,120}viewer\.releasePointerCapture\(activePointerId\)/);
  assert.match(replacementBlock, /pointerRef\.current = \{ id: null, startX: 0, startY: 0, originX: 0, originY: 0, moved: false \};/);
  assert.match(source, /useEffect\(\(\) => \(\) => \{[\s\S]{0,360}releasePointerCapture\(activePointerId\)[\s\S]{0,160}pointerRef\.current\.id = null/);
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
  assert.match(source, /observeDerivedMediaVisibility\(element, rootMargin, setIsNearViewport\)/);
  assert.match(source, /const shouldResolve = resolveMedia != null && supportedMediaKind && shouldResolveDerivedMedia\(kind, isNearViewport\);/);
  assert.match(source, /if \(!shouldResolve \|\| resolveMedia == null\) \{[\s\S]{0,120}setMedia\(null\);[\s\S]{0,120}setLoading\(false\);/);
  assert.match(source, /const shouldResolve = shouldResolveDerivedThumbnail\(isNearViewport, isActive\);/);
  assert.match(source, /<MediaThumbnail isActive=\{attachmentIndex === index\}/);
  assert.match(source, /<MediaCard[\s\S]{0,400}observe=\{observeMediaCard\}/);
  assert.match(source, /<MediaViewer attachments=\{mediaAttachments\}/);
});
