import assert from "node:assert/strict";
import test from "node:test";

import type { ComposerDraft, ConversationTranscriptEntry } from "./model.ts";
import { createReplyThreadController, type ReplySelection } from "./reply-thread-controller.ts";
import type { ComposerSubmission } from "./submission.ts";

function draft(replyToId: string): ComposerDraft {
  return { prompt: "Reply draft", attachments: [], replyToId };
}

function message(id: string, text: string, patch: Record<string, unknown> = {}): ConversationTranscriptEntry {
  return {
    kind: "message",
    id,
    role: "user",
    author: "Alice",
    text,
    timestampMs: 1,
    attachments: [],
    reactions: [],
    ...patch,
  } as unknown as ConversationTranscriptEntry;
}

function localHumanParent(nonce: string, text: string, authorId = "account-a"): ConversationTranscriptEntry {
  return message(`human-message:${nonce}`, text, {
    authorKind: "human", authorId, clientNonce: nonce, delivery: "pending",
  });
}

function canonicalHumanParent(
  nonce: string,
  remoteMessageId: string,
  text: string,
  authorId = "account-a",
  delivery = "sent",
  id = `human-server-message:${remoteMessageId}`,
): ConversationTranscriptEntry {
  return message(id, text, {
    authorKind: "human", authorId, clientNonce: nonce, remoteMessageId, delivery,
  });
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


test("an active local Human reply selection follows Host canonical rekey and refreshes its preview", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  controller.replaceEntries([localHumanParent("parent-nonce", "Local parent before Host settlement")]);
  assert.equal(controller.selectReply(alias), true);
  assert.equal(controller.getSelection()?.targetId, alias);
  assert.deepEqual(controller.getSelection()?.preview, {
    kind: "user-text",
    text: "Local parent before Host settlement",
  });

  controller.replaceEntries([canonicalHumanParent("parent-nonce", "42", "Canonical parent after Host rekey")]);

  assert.equal(controller.getSelection()?.status, "resolved");
  assert.equal(controller.getSelection()?.targetId, canonicalId);
  assert.deepEqual(controller.getSelection()?.preview, {
    kind: "user-text",
    text: "Canonical parent after Host rekey",
  });
  assert.equal(controller.projectSubmission(submission("human:alice")).replyToId, canonicalId);
  controller.dispose();
});

test("append reconciliation resolves a canonical alias while the local outbox row is retained", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  controller.replaceEntries([localHumanParent("parent-nonce", "Pending local parent")]);
  assert.equal(controller.selectReply(alias), true);
  controller.appendEntries([canonicalHumanParent("parent-nonce", "42", "Canonical parent after append")]);

  assert.equal(controller.resolve(alias).status, "resolved");
  assert.equal(controller.resolve(alias).targetId, canonicalId);
  assert.equal(controller.getSelection()?.targetId, canonicalId);
  assert.deepEqual(controller.getSelection()?.preview, {
    kind: "user-text",
    text: "Canonical parent after append",
  });
  assert.equal(controller.projectSubmission(submission("human:alice")).replyToId, canonicalId);
  controller.dispose();
});

test("prepend reconciliation resolves a canonical alias while the local outbox row is retained", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  controller.replaceEntries([localHumanParent("parent-nonce", "Pending local parent")]);
  assert.equal(controller.selectReply(alias), true);
  controller.prependEntries([canonicalHumanParent("parent-nonce", "42", "Canonical parent after prepend")]);

  assert.equal(controller.resolve(alias).status, "resolved");
  assert.equal(controller.resolve(alias).targetId, canonicalId);
  assert.equal(controller.getSelection()?.targetId, canonicalId);
  assert.deepEqual(controller.getSelection()?.preview, {
    kind: "user-text",
    text: "Canonical parent after prepend",
  });
  assert.equal(controller.projectSubmission(submission("human:alice")).replyToId, canonicalId);
  controller.dispose();
});

test("a persisted local Human reply draft restores through its unique canonical Host parent", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  assert.equal(controller.restoreReplyFromDraft(draft(alias)), true);
  assert.equal(controller.getSelection()?.status, "missing");
  controller.replaceEntries([canonicalHumanParent("parent-nonce", "42", "Canonical restored parent")]);

  assert.equal(controller.getSelection()?.status, "resolved");
  assert.equal(controller.getSelection()?.targetId, canonicalId);
  assert.deepEqual(controller.getSelection()?.preview, {
    kind: "user-text",
    text: "Canonical restored parent",
  });
  assert.equal(controller.applyReplyToDraft({ prompt: "Reply draft", attachments: [] }).replyToId, canonicalId);
  assert.equal(controller.projectSubmission(submission("human:alice", alias)).replyToId, canonicalId);
  controller.dispose();
});


test("removing a reply parent clears only the matching live selection, including a Host-rekeyed canonical alias", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });

  controller.replaceEntries([localHumanParent("parent-nonce", "Pending local parent")]);
  assert.equal(controller.selectReply(alias), true);
  controller.replaceEntries([canonicalHumanParent("parent-nonce", "42", "Canonical parent")]);
  assert.equal(controller.getSelection()?.targetId, canonicalId);

  assert.equal(controller.clearReplyIfMatches("human-server-message:99"), false);
  assert.equal(controller.getSelection()?.targetId, canonicalId);
  assert.equal(controller.clearReplyIfMatches(canonicalId), true);
  assert.equal(controller.getSelection(), null);
  controller.dispose();
});

test("removing a canonical reply parent clears only its matching persisted alias and preserves the rest of the draft", () => {
  const alias = "human-message:parent-nonce";
  const canonicalId = "human-server-message:42";
  const controller = createReplyThreadController({
    scope: { accountSlot: "account-a", agentId: "human:alice" },
  });
  controller.replaceEntries([canonicalHumanParent("parent-nonce", "42", "Canonical parent")]);

  const original: ComposerDraft = {
    prompt: "Keep this reply text",
    attachments: [{ path: "/tmp/kept.png", name: "kept.png" }],
    richText: "formatted reply",
    replyToId: alias,
    isFork: true,
  };
  const cleared = controller.clearReplyFromDraftIfMatches(original, canonicalId);
  assert.deepEqual(cleared, {
    prompt: "Keep this reply text",
    attachments: [{ path: "/tmp/kept.png", name: "kept.png" }],
    richText: "formatted reply",
    isFork: true,
  });
  assert.equal(Object.hasOwn(cleared, "replyToId"), false);

  const unrelated: ComposerDraft = { ...original, replyToId: "human-server-message:99" };
  assert.equal(controller.clearReplyFromDraftIfMatches(unrelated, canonicalId), unrelated);
  controller.dispose();
});

test("historical local Human reply aliases fail closed for foreign, ambiguous and non-canonical rows", () => {
  const alias = "human-message:parent-nonce";
  const cases: Array<{ name: string; entries: ConversationTranscriptEntry[] }> = [
    {
      name: "foreign canonical parent",
      entries: [canonicalHumanParent("parent-nonce", "42", "Foreign parent", "account-b")],
    },
    {
      name: "ambiguous canonical parents",
      entries: [
        canonicalHumanParent("parent-nonce", "42", "First parent"),
        canonicalHumanParent("parent-nonce", "43", "Second parent"),
      ],
    },
    {
      name: "canonical ID disagrees with remote identity",
      entries: [canonicalHumanParent("parent-nonce", "43", "Mismatched parent", "account-a", "sent", "human-server-message:42")],
    },
    {
      name: "canonical parent is unsettled",
      entries: [canonicalHumanParent("parent-nonce", "42", "Unsettled parent", "account-a", "pending")],
    },
    {
      name: "direct local alias belongs to another account",
      entries: [localHumanParent("parent-nonce", "Foreign local alias", "account-b")],
    },
    {
      name: "malformed retained local alias conflicts with a canonical parent",
      entries: [
        localHumanParent("parent-nonce", "Foreign local alias", "account-b"),
        canonicalHumanParent("parent-nonce", "42", "Canonical parent"),
      ],
    },
  ];

  for (const scenario of cases) {
    const controller = createReplyThreadController({
      scope: { accountSlot: "account-a", agentId: "human:alice" },
    });
    assert.equal(controller.restoreReplyFromDraft(draft(alias)), true, scenario.name);
    controller.replaceEntries(scenario.entries);
    assert.equal(controller.getSelection()?.status, "missing", scenario.name);
    assert.equal(controller.getSelection()?.targetId, alias, scenario.name);
    assert.equal(controller.resolve(alias).status, "missing", scenario.name);
    assert.equal(controller.projectSubmission(submission("human:alice")).replyToId, alias, scenario.name);
    controller.dispose();
  }
});
