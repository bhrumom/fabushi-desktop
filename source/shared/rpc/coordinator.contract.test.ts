import assert from "node:assert/strict";
import test from "node:test";

import {
  COORDINATOR_METHOD_TABLE,
  parseCoordinatorHumanMessageRetryRequest,
  parseCoordinatorHumanMessageRetryResponse,
  parseCoordinatorHumanTranscriptPageRequest,
  parseCoordinatorHumanTranscriptPageResponse,
  parseCoordinatorTelegramAuthorizationProjection,
  validateCoordinatorReply,
} from "./coordinator.js";

test("Human retry command preserves the exact two identities including a 200-byte UTF-8 nonce", () => {
  const request = { conversationId: "human:1:2", clientNonce: "界".repeat(66) + "ab" };
  assert.deepEqual(parseCoordinatorHumanMessageRetryRequest(request), request);
  assert.deepEqual(COORDINATOR_METHOD_TABLE.retryHumanMessage, { args: "object", reply: "human-message-settlement" });
});

test("Human retry rejects replacement payload, padded identities and noncanonical argument objects", () => {
  const request = { conversationId: "human:1:2", clientNonce: "original" };
  const invalid: unknown[] = [null, [], {}, { conversationId: "human:1:2" }, { clientNonce: "original" },
    { ...request, conversationId: "" }, { ...request, conversationId: " human:1:2" },
    { ...request, conversationId: 1 }, { ...request, clientNonce: " original" },
    { ...request, clientNonce: "original\n" }, { ...request, clientNonce: "" },
    { ...request, clientNonce: null }, { ...request, clientNonce: "界".repeat(67) },
    Object.assign(Object.create({ conversationId: "human:1:2" }), { clientNonce: "original", text: "replacement" })];
  for (const field of ["text", "senderId", "attachments", "replyToId", "forwardContext", "composedAtMs", "unknown"]) {
    invalid.push({ ...request, [field]: undefined });
  }
  for (const value of invalid) assert.equal(parseCoordinatorHumanMessageRetryRequest(value), null);
});

function retrySettlement() {
  return { kind: "message", id: "human-server-message:42", role: "user", authorKind: "human", authorId: "local",
    clientNonce: "original", delivery: "sent", remoteMessageId: "42", content: "Original text", timestampMs: 100,
    composedAtMs: 80, replyToId: "human-server-message:3",
    attachments: [{ path: "/committed/original.txt", name: "original.txt", resourceId: "resource-1" }] };
}

test("Human retry settlement preserves the canonical row and its durable reply and resource projection", () => {
  const row = retrySettlement();
  assert.equal(parseCoordinatorHumanMessageRetryResponse(row), row);
  assert.equal(validateCoordinatorReply("retryHumanMessage", row), row);
  assert.ok(parseCoordinatorHumanMessageRetryResponse({ ...row, remoteMessageId: "9007199254740991", id: "human-server-message:9007199254740991" }));
  assert.ok(parseCoordinatorHumanMessageRetryResponse({ ...row, composedAtMs: null, replyToId: null, attachments: [] }));
});

test("Human retry settlement rejects malformed rows and local-only or conflicting server identities", () => {
  const row = retrySettlement();
  const invalid: unknown[] = [null, {}, [], { ...row, kind: "tool" }, { ...row, role: "assistant" },
    { ...row, authorKind: "agent" }, { ...row, authorId: " local" }, { ...row, clientNonce: "" },
    { ...row, clientNonce: "界".repeat(67) }, { ...row, delivery: "pending" },
    { ...row, remoteMessageId: undefined, id: "human-message:original" },
    { ...row, id: "human-server-message:43" }, { ...row, content: null },
    { ...row, timestampMs: Number.NaN }, { ...row, composedAtMs: Infinity }, { ...row, replyToId: 3 },
    { ...row, attachments: {} }, { ...row, attachments: [{ path: "/file", name: "file" }] }];
  for (const remoteMessageId of [42, "0", "-1", "042", "4.2", "1e2", " 42", "9007199254740992"]) {
    invalid.push({ ...row, remoteMessageId, id: `human-server-message:${remoteMessageId}` });
  }
  for (const value of invalid) {
    assert.equal(parseCoordinatorHumanMessageRetryResponse(value), null);
    assert.throws(() => validateCoordinatorReply("retryHumanMessage", value), /malformed Human message settlement/);
  }
});

test("Human transcript cursors preserve the opaque traversal beside the physical anchor", () => {
  const cursor = "htp1:8e51c11de4324995a6c6878a94e3d32f:4:17";
  const request = { conversationId: "human:1:2", beforeSeq: 17, beforeCursor: cursor, limit: 200 };
  assert.deepEqual(parseCoordinatorHumanTranscriptPageRequest(request), request);
  const page = { entries: [{ id: "human-server-message:6", kind: "message" }], nextBeforeSeq: 17, nextBeforeCursor: cursor };
  assert.deepEqual(parseCoordinatorHumanTranscriptPageResponse(page), page);
  assert.equal(validateCoordinatorReply("getHumanConversationTranscriptTail", page), page);
  assert.deepEqual(parseCoordinatorHumanTranscriptPageResponse({ entries: [], nextBeforeSeq: null, nextBeforeCursor: null }), { entries: [] });
  assert.deepEqual(parseCoordinatorHumanTranscriptPageResponse({ entries: [], nextBeforeSeq: 17 }), { entries: [], nextBeforeSeq: 17 });
});

test("Human transcript cursor admission rejects malformed scope and continuation without coercion", () => {
  for (const invalid of [
    { conversationId: "" },
    { conversationId: "human:1:2", beforeCursor: 3 },
    { conversationId: "human:1:2", beforeCursor: "" },
    { conversationId: "human:1:2", beforeCursor: "x".repeat(257) },
    { conversationId: "human:1:2", beforeSeq: Number.MAX_SAFE_INTEGER + 1 },
    { conversationId: "human:1:2", beforeSeq: -1 },
    { conversationId: "human:1:2", limit: 1.5 },
  ]) assert.equal(parseCoordinatorHumanTranscriptPageRequest(invalid), null);
  for (const invalid of [
    { entries: [], nextBeforeCursor: "opaque" },
    { entries: [], nextBeforeSeq: 1, nextBeforeCursor: {} },
    { entries: [], nextBeforeSeq: 1, nextBeforeCursor: "" },
    { entries: [], nextBeforeSeq: 0 },
    { entries: [{ id: "missing-kind" }], nextBeforeSeq: 1 },
  ]) {
    assert.equal(parseCoordinatorHumanTranscriptPageResponse(invalid), null);
    assert.throws(() => validateCoordinatorReply("getHumanConversationTranscriptTail", invalid), /malformed Human transcript page/);
  }
});

test("Telegram authorization projection accepts typed challenge, issue, and bounded backoff", () => {
  const value = {
    authorizationState: {
      type: "waitCode",
      phoneNumber: "+15551234567",
      deliveryType: "email",
      codeLength: 6,
      timeoutSeconds: 60,
    },
    selfUserId: null,
    error: {
      code: "FLOOD_WAIT_17",
      category: "rateLimit",
      action: "retryLater",
      recoverable: true,
      retryAfterSeconds: 17,
    },
    backoff: { retryAfterSeconds: 17 },
  };
  const parsed = parseCoordinatorTelegramAuthorizationProjection(value);
  assert.ok(parsed);
  assert.equal(parsed.authorizationState.type, "waitCode");
  assert.equal(parsed.error?.code, "FLOOD_WAIT_17");
  assert.equal(parsed.backoff?.retryAfterSeconds, 17);
  assert.equal(validateCoordinatorReply("feature.auth.telegramSignIn", value), value);
});

test("Telegram authorization projection rejects malformed or unbounded state", () => {
  assert.equal(parseCoordinatorTelegramAuthorizationProjection({
    authorizationState: { type: "unknownState" },
    selfUserId: null,
  }), null);
  assert.equal(parseCoordinatorTelegramAuthorizationProjection({
    authorizationState: { type: "ready" },
    selfUserId: -1,
  }), null);
  assert.equal(parseCoordinatorTelegramAuthorizationProjection({
    authorizationState: { type: "waitPassword", passwordHint: "", hasRecoveryEmail: true },
    error: {
      code: "FLOOD_WAIT_999999999",
      category: "rateLimit",
      action: "retryLater",
      recoverable: true,
      retryAfterSeconds: 999999999,
    },
  }), null);
  assert.throws(
    () => validateCoordinatorReply("feature.auth.telegramStatus", { authorizationState: { type: "bogus" } }),
    /malformed Telegram authorization projection/,
  );
});


test("remaining Telegram auth lifecycle methods share the canonical projection contract", () => {
  const value = {
    authorizationState: {
      type: "waitPassword",
      passwordHint: "hint",
      hasRecoveryEmail: true,
      recoveryEmailPattern: "m***@example.test",
    },
    selfUserId: null,
  };
  for (const method of [
    "feature.auth.telegramResendCode",
    "feature.auth.telegramCancelCode",
    "feature.auth.telegramRequestPasswordRecovery",
    "feature.auth.telegramCheckRecoveryPassword",
    "feature.auth.telegramSendVerifyEmailCode",
    "feature.auth.telegramVerifyEmail",
    "feature.auth.telegramRefreshPasswordChallenge",
  ]) {
    assert.equal(validateCoordinatorReply(method, value), value);
  }
});
