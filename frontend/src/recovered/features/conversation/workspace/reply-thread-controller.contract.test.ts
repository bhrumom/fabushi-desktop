import assert from "node:assert/strict";
import test from "node:test";

import type { ComposerDraft, ConversationTranscriptEntry } from "./model.ts";
import { createReplyThreadController, type ReplySelection } from "./reply-thread-controller.ts";
import type { ComposerSubmission } from "./submission.ts";

function draft(replyToId: string): ComposerDraft {
  return { prompt: "Reply draft", attachments: [], replyToId };
}

function message(id: string, text: string): ConversationTranscriptEntry {
  return {
    kind: "message",
    id,
    role: "user",
    author: "Alice",
    text,
    timestampMs: 1,
    attachments: [],
    reactions: [],
  } as unknown as ConversationTranscriptEntry;
}

function submission(agentId: string, replyToId?: string): ComposerSubmission {
  return {
    nonce: "stable-reply-nonce",
    agentId,
    prompt: "Reply draft",
    attachments: [],
    createdAtMs: 1,
    ...(replyToId == null ? {} : { replyToId }),
  };
}

test("reply draft restoration rehydrates the scoped target, resolves its preview and projects the same reply identity", () => {
  const changes: Array<ReplySelection | null> = [];
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
    onSelectionChange: (selection) => changes.push(selection),
  });

  assert.equal(controller.restoreReplyFromDraft(draft("m1")), true);
  assert.equal(controller.getSelection()?.status, "missing");

  controller.replaceEntries([message("m1", "Original human message")]);

  assert.equal(controller.getSelection()?.status, "resolved");
  assert.deepEqual(controller.getSelection()?.preview, { kind: "user-text", text: "Original human message" });
  const projected = controller.projectSubmission(submission("human:alice", "stale-reply"));
  assert.equal(projected.replyToId, "m1");
  assert.equal(projected.nonce, "stable-reply-nonce");
  assert.equal(changes.length, 2);

  controller.dispose();
});

test("a deleted or not-yet-loaded reply target keeps its persisted ID until the user clears it", () => {
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  assert.equal(controller.restoreReplyFromDraft(draft("deleted-parent")), true);
  assert.equal(controller.getSelection()?.status, "missing");
  assert.equal(controller.projectSubmission(submission("human:alice")).replyToId, "deleted-parent");

  controller.clearReply();

  assert.equal(controller.getSelection(), null);
  assert.equal(controller.projectSubmission(submission("human:alice", "stale-reply")).replyToId, undefined);
  controller.dispose();
});

test("account or conversation scope replacement clears a restored reply before any new submission", () => {
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  assert.equal(controller.restoreReplyFromDraft(draft("m1")), true);
  controller.setScope({ accountSlot: "account-b", agentId: "human:alice" });
  assert.equal(controller.getSelection(), null);
  assert.equal(controller.projectSubmission(submission("human:alice", "stale-reply")).replyToId, undefined);

  controller.setScope({ accountSlot: "account-b", agentId: "human:bob" });
  assert.equal(controller.getSelection(), null);
  assert.equal(controller.projectSubmission(submission("human:bob", "stale-reply")).replyToId, undefined);
  controller.dispose();
});

test("persisted draft restoration never overwrites a newer live reply selection", () => {
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  assert.equal(controller.selectReply("live-selection"), true);
  assert.equal(controller.restoreReplyFromDraft(draft("older-persisted-selection")), false);
  assert.equal(controller.getSelection()?.targetId, "live-selection");

  controller.dispose();
  assert.equal(controller.restoreReplyFromDraft(draft("after-dispose")), false);
});
