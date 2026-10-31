import assert from "node:assert/strict";
import test from "node:test";

import type { TranscriptMessage } from "./conversation/workspace/model.ts";
import {
  beginHumanMessageDelete,
  beginHumanMessageEdit,
  isAuthoritativeHumanMessageMutationSettlement,
  settleHumanMessageMutation,
} from "./conversation/workspace/human-message-mutations.ts";

function message(revision: number, text: string, overrides: Partial<TranscriptMessage> = {}): TranscriptMessage {
  return {
    kind: "message",
    id: "human-server-message:42",
    role: "user",
    author: "You",
    authorId: "human-local",
    text,
    timestampMs: 1000,
    delivery: "sent",
    remoteMessageId: "42",
    remoteRevision: revision,
    ...overrides,
  };
}

test("REG-MSG-MUTATION-STALE-001 stale authoritative settlement cannot replace a newer canonical revision", () => {
  const current = [message(5, "newer")];
  const stale = message(4, "stale", { editedAt: "2026-10-31T12:00:00.000Z" });
  const settled = settleHumanMessageMutation(current, stale);
  assert.equal(settled, current);
  assert.equal((settled[0] as TranscriptMessage).text, "newer");
});

test("REG-MSG-EDIT-AUTHORITATIVE-001 edit completes only on a newer owner-matching authoritative revision", () => {
  const base = message(3, "before");
  const state = beginHumanMessageEdit("human-direct-x", "human-local", base, "edit-operation-1");
  assert.ok(state != null);
  const authoritative = message(4, "after", { editedAt: "2026-10-31T12:01:00.000Z" });
  assert.equal(isAuthoritativeHumanMessageMutationSettlement(state, authoritative, "edit"), true);
  assert.equal(isAuthoritativeHumanMessageMutationSettlement(state, message(3, "replay", { editedAt: "2026-10-31T12:01:00.000Z" }), "edit"), false);
  assert.equal(isAuthoritativeHumanMessageMutationSettlement(state, { ...authoritative, authorId: "human-peer" }, "edit"), false);
  const currentForEdit = [base];
  const settled = settleHumanMessageMutation(currentForEdit, authoritative);
  assert.notEqual(settled, currentForEdit);
  assert.equal((settled[0] as TranscriptMessage).text, "after");
  assert.equal((settled[0] as TranscriptMessage).remoteRevision, 4);
});

test("REG-MSG-DELETE-AUTHORITATIVE-001 delete settles in place only as a newer tombstone", () => {
  const base = message(4, "delete me");
  const target = beginHumanMessageDelete("human-direct-x", "human-local", base, "delete-operation-1");
  assert.ok(target != null);
  const tombstone = message(5, "", {
    isDeleted: true,
    deletedAt: "2026-10-31T12:02:00.000Z",
  });
  assert.equal(isAuthoritativeHumanMessageMutationSettlement(target, tombstone, "delete"), true);
  const settled = settleHumanMessageMutation([base], tombstone);
  assert.equal(settled.length, 1);
  assert.equal((settled[0] as TranscriptMessage).id, base.id);
  assert.equal((settled[0] as TranscriptMessage).isDeleted, true);
  assert.equal((settled[0] as TranscriptMessage).remoteRevision, 5);
});
