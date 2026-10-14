import assert from "node:assert/strict";
import test from "node:test";

import {
  conversationDestinationScopeKey,
  createConversationChildSelectionController
} from "./child-selection.ts";

test("typed child picker keeps topic saved-sublist and nested conversation identities distinct", () => {
  const controller = createConversationChildSelectionController();
  assert.equal(controller.select({
    conversationId: "conversation:parent",
    child: { kind: "topic", rootMessageId: "message:42" }
  }), true);
  assert.equal(
    conversationDestinationScopeKey(controller.getSnapshot().selected!),
    "conversation:conversation:parent:topic:message:42"
  );

  assert.equal(controller.select({
    conversationId: "conversation:parent",
    child: { kind: "savedSublist", participantId: "human:42" }
  }), true);
  assert.equal(
    conversationDestinationScopeKey(controller.getSnapshot().selected!),
    "conversation:conversation:parent:saved:human:42"
  );

  assert.equal(controller.select({
    conversationId: "conversation:parent",
    child: { kind: "conversation", conversationId: "conversation:child" }
  }), true);
  assert.equal(
    conversationDestinationScopeKey(controller.getSnapshot().selected!),
    "conversation:conversation:parent:child:conversation:child"
  );
});

test("legacy topic ids require an explicit canonical root mapping and never fall back to parent", () => {
  const controller = createConversationChildSelectionController({
    resolveLegacyTopicRoot: (conversationId, legacyTopicId) =>
      conversationId === "channel:1" && legacyTopicId === "7" ? "message:700" : null
  });

  assert.equal(controller.selectLegacyTopic({
    conversationId: "channel:1",
    legacyTopicId: "missing"
  }), false);
  assert.equal(controller.getSnapshot().selected, null);

  assert.equal(controller.selectLegacyTopic({
    conversationId: "channel:1",
    legacyTopicId: "7"
  }), true);
  assert.deepEqual(controller.getSnapshot().selected, {
    conversationId: "channel:1",
    child: { kind: "topic", rootMessageId: "message:700" }
  });
});

test("destroying the exact selected child clears it without silently selecting the parent", () => {
  const controller = createConversationChildSelectionController();
  controller.select({
    conversationId: "channel:1",
    child: { kind: "topic", rootMessageId: "message:700" }
  });

  assert.equal(controller.clearDestroyedChild(
    "channel:1",
    { kind: "savedSublist", participantId: "human:7" }
  ), false);
  assert.notEqual(controller.getSnapshot().selected, null);

  assert.equal(controller.clearDestroyedChild(
    "channel:1",
    { kind: "topic", rootMessageId: "message:700" }
  ), true);
  assert.equal(controller.getSnapshot().selected, null);
});

test("self-nested and malformed child destinations fail closed", () => {
  const controller = createConversationChildSelectionController();
  assert.equal(controller.select({
    conversationId: "conversation:same",
    child: { kind: "conversation", conversationId: "conversation:same" }
  }), false);
  assert.equal(controller.select({
    conversationId: "conversation:parent",
    child: { kind: "topic", rootMessageId: "   " }
  }), false);
  assert.equal(controller.getSnapshot().selected, null);
});
