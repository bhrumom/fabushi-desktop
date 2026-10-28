import assert from "node:assert/strict";
import test from "node:test";

import {
  mergeAuthoritativeTranscriptMessage,
  reconcileAuthoritativeTranscriptBaseline,
  reconcileLateInitialTranscriptPage,
  reconcileTranscriptFeedBaseline,
  reconcileTranscriptFeedUpdate,
  transcriptEntriesCorrelate,
} from "./model.ts";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../recovered/features/conversation/workspace/model.ts";

function message(input: Partial<TranscriptMessage> & Pick<TranscriptMessage, "id" | "role" | "text">): TranscriptMessage {
  return { kind: "message", author: input.role === "user" ? "You" : "Fabushi", timestampMs: input.timestampMs ?? 1, delivery: "sent", ...input };
}

test("authoritative echo replaces optimistic message without losing attachments", () => {
  const optimistic = message({ id: "pending-nonce-1", role: "user", text: "", clientNonce: "nonce-1", attachments: [{ path: "/tmp/a", name: "a" }], delivery: "pending" });
  const echoed = message({ id: "server-1", role: "user", text: "hello", clientNonce: "nonce-1", attachments: [{ path: "/tmp/a", name: "a", size: 4 }] });
  const merged = mergeAuthoritativeTranscriptMessage(optimistic, echoed);
  assert.equal(merged.id, "server-1");
  assert.equal(merged.text, "hello");
  assert.deepEqual(merged.attachments, [{ path: "/tmp/a", name: "a", size: 4 }]);
  assert.equal(transcriptEntriesCorrelate(optimistic, echoed), true);
});

test("late baseline preserves unresolved optimistic entries and canonical page order", () => {
  const optimistic = message({ id: "pending-nonce-2", role: "user", text: "pending", clientNonce: "nonce-2", delivery: "pending" });
  const older = message({ id: "server-older", role: "assistant", text: "older", timestampMs: 1 });
  const newer = message({ id: "server-newer", role: "assistant", text: "newer", timestampMs: 3 });
  assert.deepEqual(reconcileLateInitialTranscriptPage([optimistic], [older, newer]), [older, newer, optimistic]);
  assert.deepEqual(reconcileAuthoritativeTranscriptBaseline([optimistic], [message({ id: "server-2", role: "user", text: "accepted", clientNonce: "nonce-2" })]).map((entry) => entry.id), ["server-2"]);
});

test("feed baseline keeps an event observed before the baseline exactly once", () => {
  const live = message({ id: "live-1", role: "assistant", text: "final", timestampMs: 4 });
  const baseline = message({ id: "history-1", role: "user", text: "prompt", timestampMs: 1 });
  const result = reconcileTranscriptFeedBaseline([live], [baseline], new Set(["live-1"]));
  assert.deepEqual(result.map((entry) => entry.id), ["history-1", "live-1"]);
  const repeated = reconcileTranscriptFeedBaseline(result, [baseline], new Set(["live-1"]));
  assert.equal(repeated.filter((entry) => entry.id === "live-1").length, 1);
});

test("feed updates arriving before the baseline are retained and later merged", () => {
  const update = message({ id: "live-2", role: "assistant", text: "draft", timestampMs: 2 });
  const revised = message({ id: "live-2", role: "assistant", text: "final", timestampMs: 2, delivery: "sent" });
  const first = reconcileTranscriptFeedUpdate([], update);
  assert.deepEqual(first.map((entry) => entry.id), ["live-2"]);
  assert.equal(reconcileTranscriptFeedUpdate(first, revised)[0]?.kind, "message");
  assert.equal((reconcileTranscriptFeedUpdate(first, revised)[0] as TranscriptMessage).text, "final");
});

test("feed updates correlate optimistic and authoritative IDs by client nonce", () => {
  const optimistic = message({ id: "pending-nonce-3", role: "user", text: "draft", clientNonce: "nonce-3", delivery: "pending" });
  const authoritative = message({ id: "server-nonce-3", role: "user", text: "accepted", clientNonce: "nonce-3" });
  const result = reconcileTranscriptFeedUpdate([optimistic], authoritative);
  assert.deepEqual(result.map((entry) => entry.id), ["server-nonce-3"]);
  assert.equal((result[0] as TranscriptMessage).text, "accepted");
});

test("non-message entries remain untouched during baseline reconciliation", () => {
  const notice: ConversationTranscriptEntry = { kind: "notice", id: "notice-1", text: "connected", timestampMs: 1 };
  assert.deepEqual(reconcileAuthoritativeTranscriptBaseline([notice], [notice]), [notice]);
});
