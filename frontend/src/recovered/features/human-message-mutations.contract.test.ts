import assert from "node:assert/strict";
import test from "node:test";

import { projectTranscriptEntry } from "../../production/model.ts";

test("REG-MSG-EDIT-DELETE-PROJECTION-001 keeps canonical Human mutation identity and revision", () => {
  const projected = projectTranscriptEntry({
    id: "human-server-message:42",
    kind: "message",
    role: "user",
    authorId: "human-local",
    content: "edited text",
    timestampMs: 1000,
    delivery: "sent",
    remoteMessageId: "42",
    remoteRevision: 3,
    editedAt: "2026-10-31T12:00:00.000Z",
    isDeleted: false,
    reactions: [],
  }, 0, "Peer", "human-direct-x");
  assert.ok(projected != null && projected.kind === "message");
  assert.equal(projected.id, "human-server-message:42");
  assert.equal(projected.text, "edited text");
  assert.equal(projected.authorId, "human-local");
  assert.equal(projected.remoteMessageId, "42");
  assert.equal(projected.remoteRevision, 3);
  assert.equal(projected.editedAt, "2026-10-31T12:00:00.000Z");
  assert.equal(projected.isDeleted, undefined);
});

test("REG-MSG-DELETE-TOMBSTONE-PROJECTION-001 preserves a deleted Human row for ordering and provenance", () => {
  const projected = projectTranscriptEntry({
    id: "human-server-message:42",
    kind: "message",
    role: "user",
    authorId: "human-local",
    content: "",
    timestampMs: 1000,
    delivery: "sent",
    remoteMessageId: "42",
    remoteRevision: 4,
    deletedAt: "2026-10-31T12:01:00.000Z",
    isDeleted: true,
    reactions: [],
  }, 0, "Peer", "human-direct-x");
  assert.ok(projected != null && projected.kind === "message");
  assert.equal(projected.id, "human-server-message:42");
  assert.equal(projected.text, "");
  assert.equal(projected.remoteRevision, 4);
  assert.equal(projected.deletedAt, "2026-10-31T12:01:00.000Z");
  assert.equal(projected.isDeleted, true);
  assert.deepEqual(projected.reactions, []);
});
