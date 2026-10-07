import assert from "node:assert/strict";
import test from "node:test";

import { bindConversationChildSelectionToPagination } from "./child-history-composition.ts";
import {
  conversationDestinationScopeKey,
  createConversationChildSelectionController
} from "./child-selection.ts";
import { createConversationChildPageRequestController } from "./child-pagination.ts";

test("shipping composition projects canonical child selection into child pagination scope", async () => {
  const requests: Array<{ scopeKey: string; direction: string; anchor: string | null }> = [];
  const commits: string[] = [];
  const selection = createConversationChildSelectionController({
    resolveLegacyTopicRoot: (_conversationId, legacyTopicId) =>
      legacyTopicId === "7" ? "message:700" : null
  });
  const pagination = createConversationChildPageRequestController({
    fetchPage: async (request) => {
      requests.push(request);
      return { messageIds: ["message:one"] };
    },
    getBoundary: () => null,
    commitPage: (_direction, page) => commits.push(...page.messageIds)
  });
  let changes = 0;
  const unbind = bindConversationChildSelectionToPagination(
    selection,
    pagination,
    conversationDestinationScopeKey,
    () => { changes += 1; }
  );

  assert.equal(pagination.getSnapshot().scopeKey, null);
  assert.equal(selection.selectLegacyTopic({
    conversationId: "channel:one",
    legacyTopicId: "missing"
  }), false);
  assert.equal(pagination.getSnapshot().scopeKey, null);

  assert.equal(selection.selectLegacyTopic({
    conversationId: "channel:one",
    legacyTopicId: "7"
  }), true);
  assert.equal(
    pagination.getSnapshot().scopeKey,
    "conversation:channel:one:topic:message:700"
  );

  await pagination.loadAround("message:700");
  assert.equal(requests.length, 1);
  assert.equal(requests[0]?.scopeKey, "conversation:channel:one:topic:message:700");
  assert.deepEqual(commits, ["message:one"]);
  assert.ok(changes >= 2);

  selection.clear();
  assert.equal(pagination.getSnapshot().scopeKey, null);

  unbind();
  pagination.dispose();
  selection.dispose();
});

test("destroying the selected child clears the shipping page scope instead of falling back to parent", () => {
  const selection = createConversationChildSelectionController();
  const pagination = createConversationChildPageRequestController({
    fetchPage: async () => ({ messageIds: [] }),
    getBoundary: () => null,
    commitPage: () => {}
  });
  const unbind = bindConversationChildSelectionToPagination(
    selection,
    pagination,
    conversationDestinationScopeKey
  );

  selection.select({
    conversationId: "channel:one",
    child: { kind: "topic", rootMessageId: "message:700" }
  });
  assert.notEqual(pagination.getSnapshot().scopeKey, null);

  assert.equal(selection.clearDestroyedChild(
    "channel:one",
    { kind: "topic", rootMessageId: "message:700" }
  ), true);
  assert.equal(selection.getSnapshot().selected, null);
  assert.equal(pagination.getSnapshot().scopeKey, null);

  unbind();
  pagination.dispose();
  selection.dispose();
});
