import assert from "node:assert/strict";
import test from "node:test";

import {
  parseCoordinatorHumanTranscriptPageRequest,
  parseCoordinatorHumanTranscriptPageResponse,
  parseCoordinatorTelegramAuthorizationProjection,
  validateCoordinatorReply,
} from "./coordinator.js";

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
