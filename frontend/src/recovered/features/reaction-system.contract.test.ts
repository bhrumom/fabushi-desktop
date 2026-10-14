import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  QUICK_REACTION_EMOJIS,
  createReactionActionController,
  createReactToMessageTransport,
  projectTranscriptReactions,
} from "./conversation/cards/transcript-card/reaction-actions.ts";
import {
  createReactionFeedAdapter,
  type TranscriptFeedHandlers,
} from "./conversation/cards/transcript-card/reaction-feed.ts";
import {
  createScopedReactionWorkspacePair,
} from "./conversation/cards/transcript-card/reaction-workspace-handoff.ts";
import {
  projectReactionPickerExpansionHandoff,
} from "./conversation/cards/transcript-card/reaction-picker-handoff.ts";

const flush = async (): Promise<void> => {
  await Promise.resolve();
  await Promise.resolve();
};

test("reaction projection keeps only valid rows and identifies self reactions", () => {
  const projected = projectTranscriptReactions([
    { emoji: "👍", by: "me" },
    { emoji: "🎉", by: "agent-2" },
    { emoji: "", by: "me" },
    null,
  ]);
  assert.deepEqual(projected.reactions, [
    { emoji: "👍", by: "me" },
    { emoji: "🎉", by: "agent-2" },
  ]);
  assert.equal(projected.myReactions.has("👍"), true);
  assert.equal(projected.myReactions.has("🎉"), false);
  assert.equal(QUICK_REACTION_EMOJIS.length, 6);
});

test("reaction action controller is optimistic, scoped, authoritative, and silent on transport failure", async () => {
  const calls: Array<{ method: string; args: unknown }> = [];
  const optimistic: unknown[] = [];
  const authoritative: unknown[] = [];
  const cleared: unknown[] = [];

  const source = {
    async call(method: "reactToMessage", args: unknown) {
      calls.push({ method, args });
      throw new Error("offline");
    },
  };
  const controller = createReactionActionController({
    scope: { accountSlot: "slot-a", agentId: "agent-a" },
    transport: createReactToMessageTransport(source),
    onReacted: (input) => optimistic.push(input),
    onAuthoritativeReactions: (update) => authoritative.push(update),
    onAuthoritativeCleared: (scope) => cleared.push(scope),
  });

  assert.equal(controller.react("entry-1", "👍"), true);
  await flush();
  assert.equal(calls.length, 1);
  assert.equal(optimistic.length, 1);

  assert.equal(controller.reconcile("entry-1", [{ emoji: "👍", by: "me" }]), true);
  assert.equal(authoritative.length, 1);
  assert.equal(controller.clearAuthoritative(), true);
  assert.deepEqual(cleared, [{ accountSlot: "slot-a", agentId: "agent-a" }]);

  controller.setScope({ accountSlot: "slot-b", agentId: null });
  assert.equal(controller.react("entry-2", "❤️"), false);
  assert.equal(controller.reconcile("entry-2", []), false);
  controller.dispose();
  assert.equal(controller.react("entry-3", "🎉"), false);
});

test("reaction feed fences agents/generations and deduplicates unchanged reaction fingerprints", () => {
  let handlers: TranscriptFeedHandlers | null = null;
  let releases = 0;
  const feed = {
    observeEntriesFeed(next: TranscriptFeedHandlers) {
      handlers = next;
      return () => {
        releases += 1;
      };
    },
  };

  const reconciled: Array<[string, unknown]> = [];
  let clears = 0;
  const scopes: unknown[] = [];
  const controller = {
    reconcile(entryId: string, raw: unknown) {
      reconciled.push([entryId, raw]);
      return true;
    },
    clearAuthoritative() {
      clears += 1;
      return true;
    },
    setScope(scope: unknown) {
      scopes.push(scope);
    },
  };

  const adapter = createReactionFeedAdapter({
    scope: { accountSlot: "slot", agentId: "agent-a" },
    feed,
    controller,
  });

  handlers!.onBaseline({
    agentId: "agent-a",
    entries: [{
      id: "entry-1",
      reactions: [{ emoji: "👍", by: "me" }],
    }],
  });
  handlers!.onUpdated({
    agentId: "agent-a",
    after: {
      id: "entry-1",
      reactions: [{ emoji: "👍", by: "me" }],
    },
  });
  assert.equal(reconciled.length, 1);

  handlers!.onAppended({
    agentId: "agent-b",
    entry: { id: "entry-b", reactions: [] },
  });
  assert.equal(reconciled.length, 1);

  handlers!.onCleared("agent-a");
  assert.equal(clears, 1);

  adapter.setScope({ accountSlot: "slot", agentId: "agent-b" });
  assert.equal(releases, 1);
  assert.deepEqual(adapter.getScope(), { accountSlot: "slot", agentId: "agent-b" });

  adapter.reconnect();
  assert.equal(releases, 2);
  adapter.reset();
  assert.equal(releases, 3);
  adapter.dispose();
  assert.equal(releases, 3);
  assert.equal(scopes.length >= 2, true);
});

test("workspace and expanded picker handoffs fail closed without required owners", () => {
  assert.equal(createScopedReactionWorkspacePair({
    scope: { accountSlot: null, agentId: "a" },
    feed: null,
    source: null,
    onReacted: () => {},
  }), null);

  const sourceCalls: unknown[] = [];
  const pair = createScopedReactionWorkspacePair({
    scope: { accountSlot: "slot", agentId: "a" },
    feed: { observeEntriesFeed: () => () => {} },
    source: {
      async call(method, args) {
        sourceCalls.push([method, args]);
      },
    },
    onReacted: () => {},
  });
  assert.ok(pair);
  assert.equal(pair.controller.react("entry", "👍"), true);

  const handoff = projectReactionPickerExpansionHandoff({
    scope: { accountSlot: "slot", agentId: "a" },
    entryId: "entry",
    myReactions: new Set(["👍"]),
    onReacted: () => {},
    Content: "div",
    contentRef: { current: null },
    onExpandPicker: () => {},
    onOpenChange: () => {},
  });
  assert.ok(handoff);
  assert.deepEqual(handoff.scope, { accountSlot: "slot", agentId: "a" });

  assert.equal(projectReactionPickerExpansionHandoff({
    ...handoff,
    scope: { accountSlot: "slot", agentId: null },
  } as never), null);
});


test("conversation floating controls keep pointer ownership while visible titlebars retain drag", () => {
  const workspaceCss = readFileSync(new URL("./conversation/workspace/view.css", import.meta.url), "utf8");
  const chromeCss = readFileSync(new URL("./window-chrome/view.css", import.meta.url), "utf8");
  const pickerSource = readFileSync(new URL("./conversation/cards/transcript-card/reaction-picker.tsx", import.meta.url), "utf8");
  const floatingSource = readFileSync(new URL("../ui/sand-floating-primitives.tsx", import.meta.url), "utf8");
  const rendererSource = readFileSync(new URL("../../production/ProductionRenderer.tsx", import.meta.url), "utf8");

  const composerLayer = Number(workspaceCss.match(/\.sand-chat-input-dock\s*\{[^}]*z-index:\s*(\d+);/s)?.[1]);
  const dragLayer = Number(chromeCss.match(/\.sand-cover-drag\s*\{[^}]*z-index:\s*(\d+);/s)?.[1]);
  const reactionLayer = Number(pickerSource.match(/MESSAGE_REACTION_POPOVER_Z_INDEX\s*=\s*(\d+);/)?.[1]);
  assert.equal(reactionLayer > composerLayer, true);
  assert.equal(reactionLayer > dragLayer, true);
  assert.match(pickerSource, /<SandPopover[\s\S]{0,800}contentStyle=\{\{ zIndex: MESSAGE_REACTION_POPOVER_Z_INDEX \}\}[\s\S]{0,600}open=\{open\}[\s\S]{0,300}returnFocus/);
  assert.match(floatingSource, /if \(!context\.open\) return null;[\s\S]{0,1200}createPortal\(surface, document\.body\)/);

  assert.match(chromeCss, /\.sand-cover-drag\s*\{[^}]*pointer-events:\s*none;/s);
  assert.match(chromeCss, /\.sand-agents-sidebar__header,[\s\S]{0,120}\.sand-chat-header\s*\{[^}]*app-region:\s*drag;/s);
  assert.match(chromeCss, /\.sand-chat-header :is\([^)]*button[^)]*\)[\s\S]{0,100}app-region:\s*no-drag;/s);
  assert.match(chromeCss, /\.sand-window-controls button\s*\{[^}]*app-region:\s*no-drag;/s);
  assert.doesNotMatch(chromeCss, /\.sand-window-controls\s*\{[^}]*pointer-events:\s*none;/s);
  assert.match(workspaceCss, /\.sand-chat-find\s*\{[^}]*z-index:\s*4;[^}]*flex:\s*0 0 auto;[^}]*app-region:\s*no-drag;/s);
  assert.doesNotMatch(workspaceCss, /\.sand-chat-find\s*\{[^}]*pointer-events:\s*none/s);
  assert.doesNotMatch(workspaceCss, /\.sand-chat-input-dock\s*\{[^}]*pointer-events:\s*none/s);

  const flexFill = 'style={{ display: "flex", flex: "1 1 0", flexDirection: "column", minHeight: 0, minWidth: 0, width: "100%" }}';
  const collapsingColumn = 'style={{ display: "flex", flexDirection: "column", minHeight: 0, minWidth: 0, width: "100%" }}';
  assert.equal(rendererSource.split(flexFill).length - 1, 2);
  assert.equal(rendererSource.includes(collapsingColumn), false);
  assert.match(chromeCss, /\.sand-cover-drag\s*\{[^}]*pointer-events:\s*none;[^}]*app-region:\s*drag;/s);
});
