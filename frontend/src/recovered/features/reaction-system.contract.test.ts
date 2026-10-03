import assert from "node:assert/strict";
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
