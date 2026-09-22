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
