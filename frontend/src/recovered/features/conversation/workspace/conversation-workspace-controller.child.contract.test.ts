import assert from "node:assert/strict";
import test from "node:test";

import { createConversationWorkspaceController } from "./conversation-workspace-controller.ts";

const emptyTranscriptPage = {
  entries: [],
  hasMore: false,
  cursor: null
};

test("workspace composes canonical child selection with child pagination scope", async () => {
  const requests: Array<{ scopeKey: string; direction: string; anchor: string | null }> = [];
  const commits: string[] = [];
  const controller = createConversationWorkspaceController({
    accountSlot: "account:one",
    agentId: "agent:one",
    fetchTranscriptPage: async () => emptyTranscriptPage,
    childHistory: {
      fetchPage: async (request) => {
        requests.push(request);
        return { messageIds: ["message:one"] };
      },
      getBoundary: () => null,
      commitPage: (_direction, page) => commits.push(...page.messageIds),
      resolveLegacyTopicRoot: (_conversationId, legacyTopicId) =>
        legacyTopicId === "7" ? "message:700" : null
    }
  });

  assert.equal(controller.selectLegacyTopicChild({
    conversationId: "channel:one",
    legacyTopicId: "missing"
  }), false);
  assert.equal(controller.getChildSelection()?.getSnapshot().selected, null);
  assert.equal(controller.getChildPagination()?.getSnapshot().scopeKey, null);

  assert.equal(controller.selectLegacyTopicChild({
    conversationId: "channel:one",
    legacyTopicId: "7"
  }), true);
  assert.equal(
    controller.getChildPagination()?.getSnapshot().scopeKey,
    "conversation:channel:one:topic:message:700"
  );

  await controller.getChildPagination()?.loadAround("message:700");
  assert.equal(requests.length, 1);
  assert.equal(requests[0]?.scopeKey, "conversation:channel:one:topic:message:700");
  assert.deepEqual(commits, ["message:one"]);

  controller.setScope("account:two", "agent:two");
  assert.equal(controller.getChildSelection()?.getSnapshot().selected, null);
  assert.equal(controller.getChildPagination()?.getSnapshot().scopeKey, null);

  controller.dispose();
});

test("workspace without child-history capability preserves the existing Agent path", () => {
  const controller = createConversationWorkspaceController({
    fetchTranscriptPage: async () => emptyTranscriptPage
  });
  assert.equal(controller.getChildSelection(), null);
  assert.equal(controller.getChildPagination(), null);
  assert.equal(controller.selectConversationChild({
    conversationId: "channel:one",
    child: { kind: "topic", rootMessageId: "message:1" }
  }), false);
  controller.dispose();
});
