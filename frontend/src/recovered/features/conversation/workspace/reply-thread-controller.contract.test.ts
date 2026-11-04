import assert from "node:assert/strict";
import test from "node:test";
import { createReplyThreadController, isActionableReplyTargetEntry } from "./reply-thread-controller.ts";
import type { ComposerDraft, ConversationTranscriptEntry } from "./model.ts";
import type { ComposerSubmission } from "./submission.ts";

const scopeA = { accountSlot: "account-a", agentId: "agent-a" };
const scopeB = { accountSlot: "account-b", agentId: "agent-a" };
const attachment = { path: "/staging/original.txt", name: "original.txt", size: 17 };
const richText = '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Recovered input"}]}]}';
const draft: ComposerDraft = {
  prompt: "Recovered input", attachments: [attachment], richText,
  replyToId: "persisted-reply", isFork: true,
};
const input: ComposerSubmission = {
  nonce: "original", agentId: "agent-a", prompt: draft.prompt,
  attachments: [attachment], createdAtMs: 123, richText,
  replyToId: draft.replyToId, isFork: true,
};
const withoutReply = () => {
  const { replyToId: _replyToId, ...rest } = input;
  return rest;
};

test("NMR-DRAFT-01 scoped restored reply preserves complete submission input", () => {
  const controller = createReplyThreadController({ scope: scopeA });
  assert.equal(controller.getSelection(), null);
  assert.deepEqual(controller.applyReplyToDraft(draft, scopeA), draft);
  assert.deepEqual(controller.projectSubmission(input, scopeA), input);
  assert.deepEqual(controller.projectSubmission(input), withoutReply(),
    "The old unscoped caller still cannot inject a restored reply");
  assert.deepEqual(input.attachments, [attachment]);
});

test("NMR-DRAFT-01 restored reply rejects account conversation and absent scope identities", () => {
  const controller = createReplyThreadController({ scope: scopeA });
  assert.equal(controller.selectReply("current-selection"), true);
  for (const wrong of [
    scopeB,
    { accountSlot: "account-a", agentId: "another-agent" },
    { accountSlot: null, agentId: "agent-a" },
    { accountSlot: "account-a", agentId: null },
  ]) {
    assert.deepEqual(controller.projectSubmission(input, wrong), withoutReply(),
      "A retained current selection cannot bypass a supplied mismatched scope");
  }
  const anotherInput = { ...input, agentId: "another-agent" };
  const { replyToId: _replyToId, ...otherWithoutReply } = anotherInput;
  assert.deepEqual(controller.projectSubmission(anotherInput, scopeA), otherWithoutReply);
  assert.deepEqual(controller.projectSubmission({ ...input, replyToId: "" }, scopeB), withoutReply());
  assert.deepEqual(controller.getSelection()?.scope, scopeA);
});

test("NMR-DRAFT-01 current selection wins and an account switch cannot expose its old target", () => {
  const controller = createReplyThreadController({ scope: scopeA });
  assert.equal(controller.selectReply("selected-in-a"), true);
  assert.equal(controller.projectSubmission(input, scopeA).replyToId, "selected-in-a");
  assert.equal(controller.projectSubmission(input).replyToId, "selected-in-a");
  const otherDraft = { ...draft, replyToId: "persisted-in-b" };
  // The root's new render can arrive before its existing setScope effect.
  assert.deepEqual(controller.applyReplyToDraft(otherDraft, scopeB), otherDraft);
  assert.deepEqual(controller.projectSubmission({ ...input, replyToId: "persisted-in-b" }, scopeB), withoutReply());
  controller.setScope(scopeB);
  assert.equal(controller.getSelection(), null);
  assert.equal(controller.projectSubmission({ ...input, replyToId: "persisted-in-b" }, scopeB).replyToId, "persisted-in-b");
  assert.equal(controller.selectReply("new-selection-in-b"), true);
  assert.equal(controller.projectSubmission(input, scopeB).replyToId, "new-selection-in-b");
});

test("NMR-DRAFT-01 explicit reply cancellation preserves recovery content and prevents restored target reuse", () => {
  const controller = createReplyThreadController({ scope: scopeA });
  controller.selectReply("selected-in-a");
  const visible = controller.applyReplyToDraft(draft, scopeA);
  assert.equal(visible.replyToId, "selected-in-a");
  controller.clearReply();
  const cancelled = controller.clearReplyFromDraft(visible);
  assert.deepEqual(cancelled, {
    prompt: draft.prompt, attachments: [attachment], richText, isFork: true,
  });
  const { replyToId: _replyToId, ...baseInput } = input;
  const cancelledInput = { ...baseInput, ...cancelled };
  assert.deepEqual(controller.projectSubmission(cancelledInput, scopeA), cancelledInput);
  assert.equal(controller.projectSubmission(cancelledInput, scopeA).replyToId, undefined);
  assert.equal(draft.replyToId, "persisted-reply", "Cancellation does not mutate another retained input");
  assert.equal(controller.selectReply("explicit-new-selection"), true);
  assert.equal(controller.projectSubmission(cancelledInput, scopeA).replyToId, "explicit-new-selection");

  const activeEntry: ConversationTranscriptEntry = {
    kind: "message",
    id: "explicit-new-selection",
    role: "assistant",
    author: "Peer",
    text: "Actionable target",
    timestampMs: 100,
    attachments: [],
  };
  const emptiedEntry: ConversationTranscriptEntry = {
    ...activeEntry,
    text: "   ",
    attachments: [],
  };
  assert.equal(isActionableReplyTargetEntry(activeEntry), true);
  assert.equal(isActionableReplyTargetEntry(emptiedEntry), false);
  controller.replaceEntries([activeEntry]);
  assert.equal(controller.getSelection()?.targetId, "explicit-new-selection");
  assert.equal(controller.resolve("explicit-new-selection").isInScope, true);
  controller.replaceEntries([emptiedEntry]);
  assert.equal(controller.getSelection(), null);
  assert.equal(controller.resolve("explicit-new-selection").isInScope, false);
  assert.equal(controller.selectReply("explicit-new-selection"), false);
  assert.deepEqual(
    controller.applyReplyToDraft({ ...draft, replyToId: "explicit-new-selection" }, scopeA),
    cancelled,
  );
  assert.equal(
    controller.projectSubmission({ ...input, replyToId: "explicit-new-selection" }, scopeA).replyToId,
    undefined,
  );
});

test("NMR-DRAFT-01 a disposed reply controller cannot consume a persisted draft target", () => {
  const controller = createReplyThreadController({ scope: scopeA });
  controller.selectReply("retired-selection");
  controller.dispose();
  assert.equal(controller.selectReply("late-selection"), false);
  assert.equal(controller.getSelection(), null);
  assert.deepEqual(controller.applyReplyToDraft(draft, scopeA), draft);
  assert.deepEqual(controller.projectSubmission(input, scopeA), withoutReply());
  assert.deepEqual(controller.projectSubmission(input), withoutReply());
});
