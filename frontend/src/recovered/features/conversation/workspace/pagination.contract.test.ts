import assert from "node:assert/strict";
import test from "node:test";

import { mergeOlderTranscriptEntries } from "./pagination.ts";
import type { TranscriptMessage } from "./model.ts";

function message(input: Partial<TranscriptMessage> & Pick<TranscriptMessage, "id" | "role" | "text">): TranscriptMessage {
  return { kind: "message", author: input.role === "user" ? "You" : "Fabushi", timestampMs: input.timestampMs ?? 1, delivery: "sent", ...input };
}

test("older-page merge deduplicates an echo by client nonce even when server IDs differ", () => {
  const optimistic = message({ id: "pending-nonce-1", role: "user", text: "hello", clientNonce: "nonce-1", delivery: "pending" });
  const echoed = message({ id: "server-nonce-1", role: "user", text: "hello", clientNonce: "nonce-1" });
  const older = message({ id: "older-1", role: "assistant", text: "older", timestampMs: 0 });
  const merged = mergeOlderTranscriptEntries([optimistic], [older, echoed]);
  assert.deepEqual(merged.map((entry) => entry.id), ["older-1", "pending-nonce-1"]);
});

test("older-page merge removes duplicate client nonces within a fetched page", () => {
  const first = message({ id: "server-1", role: "user", text: "first", clientNonce: "nonce-1" });
  const duplicate = message({ id: "server-2", role: "user", text: "duplicate", clientNonce: "nonce-1" });
  const merged = mergeOlderTranscriptEntries([], [first, duplicate]);
  assert.deepEqual(merged.map((entry) => entry.id), ["server-1"]);
});
