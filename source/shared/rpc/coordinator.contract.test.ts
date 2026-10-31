import assert from "node:assert/strict";
import test from "node:test";

import {
  parseCoordinatorTelegramAuthorizationProjection,
  validateCoordinatorReply,
} from "./coordinator.js";

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
