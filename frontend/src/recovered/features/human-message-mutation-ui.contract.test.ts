import assert from "node:assert/strict";
import test from "node:test";

import { projectHumanConversation, projectTranscriptEntry } from "../../production/model.ts";
import {
  beginHumanMessageDelete,
  beginHumanMessageEdit,
  canDeleteHumanMessage,
  canEditHumanMessage,
  updateHumanMessageEditDraft,
} from "./conversation/workspace/human-message-mutations.ts";

function projectedMessage(authorId: string, overrides: Record<string, unknown> = {}) {
  const entry = projectTranscriptEntry({
    id: "human-server-message:42",
    kind: "message",
    role: "user",
    authorId,
    content: "owner text",
    timestampMs: 1000,
    delivery: "sent",
    remoteMessageId: "42",
    remoteRevision: 3,
    ...overrides,
  }, 0, "Peer", "human-direct-x");
  assert.ok(entry != null && entry.kind === "message");
  return entry;
}

test("REG-MSG-MUTATION-OWNER-UI-001 exposes edit/delete only for the authoritative local sender", () => {
  const conversation = projectHumanConversation({
    id: "human-direct-x",
    kind: "human",
    title: "Peer",
    participantIds: ["human-local", "human-peer"],
    localHumanId: "human-local",
  });
  assert.ok(conversation != null);
  assert.equal(conversation.raw.localHumanId, "human-local");

  const owner = projectedMessage("human-local");
  const peer = projectedMessage("human-peer");
  assert.equal(canEditHumanMessage(owner, "human-local"), true);
  assert.equal(canDeleteHumanMessage(owner, "human-local"), true);
  assert.equal(canEditHumanMessage(peer, "human-local"), false);
  assert.equal(canDeleteHumanMessage(peer, "human-local"), false);
  assert.equal(canEditHumanMessage(projectedMessage("human-local", { delivery: "pending", remoteMessageId: undefined }), "human-local"), false);
  assert.equal(canDeleteHumanMessage(projectedMessage("human-local", { isDeleted: true }), "human-local"), false);
});

test("REG-MSG-EDIT-COMPOSER-STATE-001 keeps edit text in canonical Composer state without overwriting its normal draft", () => {
  const owner = projectedMessage("human-local");
  const edit = beginHumanMessageEdit("human-direct-x", "human-local", owner, "edit-operation-1");
  assert.ok(edit != null);
  assert.equal(edit.operationId, "edit-operation-1");
  assert.equal(edit.draft.prompt, "owner text");
  assert.deepEqual(edit.draft.attachments, []);

  const changed = updateHumanMessageEditDraft(edit, {
    prompt: "edited in composer",
    richText: "{\"type\":\"doc\"}",
    attachments: [{ path: "/should/not/stage", name: "ignored.txt" }],
  });
  assert.equal(changed.draft.prompt, "edited in composer");
  assert.equal(changed.draft.richText, "{\"type\":\"doc\"}");
  assert.deepEqual(changed.draft.attachments, []);

  const deleting = beginHumanMessageDelete("human-direct-x", "human-local", owner, "delete-operation-1");
  assert.ok(deleting != null);
  assert.equal(deleting.operationId, "delete-operation-1");
  assert.equal(deleting.entryId, owner.id);
  assert.equal(deleting.baseRevision, 3);
});
