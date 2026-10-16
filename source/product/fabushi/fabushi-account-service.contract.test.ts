import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import {
  DEFAULT_FABUSHI_API_BASE_URL,
  DEFAULT_FABUSHI_RESPONSES_URL,
  FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
  normalizeFabushiApiBaseUrl,
  normalizeFabushiCiSession,
  normalizeFabushiSession,
  nativeAuthIdForIssuer,
  nativeRemoteComputerDeviceScope,
  isProductionFabushiIssuer,
} from "./fabushi-account-policy.js";

function boundedCiSession(overrides: Record<string, unknown> = {}) {
  return {
    accessToken: "access-only-token",
    accessTokenExpiresAt: Date.now() + 10 * 60_000,
    sessionId: "ci-runner:run-123",
    deviceId: "gha-123",
    username: "ci@example.com",
    userId: "ci-user",
    provider: "github-actions",
    ciRunner: true,
    ...overrides,
  };
}

test("Fabushi CI session accepts only the bounded refresh-token-free GitHub Actions shape", () => {
  const accepted = normalizeFabushiCiSession(boundedCiSession());
  assert.ok(accepted);
  assert.equal(accepted.provider, "github-actions");
  assert.equal(accepted.ciRunner, true);
  assert.equal(accepted.refreshToken, undefined);

  assert.equal(normalizeFabushiCiSession(boundedCiSession({ provider: "browser" })), null);
  assert.equal(normalizeFabushiCiSession(boundedCiSession({ ciRunner: false })), null);
  assert.equal(normalizeFabushiCiSession(boundedCiSession({ sessionId: "browser:123" })), null);
  assert.equal(normalizeFabushiCiSession(boundedCiSession({
    refreshToken: "must-not-enter-packaged-acceptance",
    refreshTokenExpiresAt: Date.now() + 60 * 60_000,
  })), null);
});

test("Fabushi durable sessions require a refresh token but CI normalization deliberately does not", () => {
  assert.equal(normalizeFabushiSession(boundedCiSession()), null);
  assert.ok(normalizeFabushiSession(boundedCiSession(), { allowRefreshless: true }));
  assert.ok(normalizeFabushiSession({
    ...boundedCiSession({ provider: "browser", ciRunner: false, sessionId: "browser:123" }),
    refreshToken: "refresh-token",
    refreshTokenExpiresAt: Date.now() + 60 * 60_000,
  }));
});

test("Fabushi API origin policy is HTTPS-first and rejects credential/query smuggling", () => {
  assert.equal(normalizeFabushiApiBaseUrl(undefined), DEFAULT_FABUSHI_API_BASE_URL);
  assert.equal(normalizeFabushiApiBaseUrl("https://api.ombhrum.com/"), DEFAULT_FABUSHI_API_BASE_URL);
  assert.equal(normalizeFabushiApiBaseUrl("http://127.0.0.1:8787/"), "http://127.0.0.1:8787");
  assert.throws(() => normalizeFabushiApiBaseUrl("http://api.ombhrum.com"), /HTTPS/);
  assert.throws(() => normalizeFabushiApiBaseUrl("https://user:secret@api.ombhrum.com"), /credentials/);
  assert.throws(() => normalizeFabushiApiBaseUrl("https://api.ombhrum.com?token=secret"), /query/);
  assert.throws(() => normalizeFabushiApiBaseUrl("https://api.ombhrum.com/#secret"), /fragment/);
  assert.equal(normalizeFabushiApiBaseUrl(FABUSHI_CI_ISOLATED_AUTH_BASE_URL), FABUSHI_CI_ISOLATED_AUTH_BASE_URL);
  assert.throws(() => normalizeFabushiApiBaseUrl("https://fabushi-ci-isolated-auth.bhrumom.workers.dev.evil.test"), /approved/);
  assert.throws(() => normalizeFabushiApiBaseUrl("https://untrusted.example"), /approved/);
});

test("Native authId preserves production identity and issuer-qualifies equal isolated user IDs", () => {
  const rawUserId = "123";
  const prodAuthId = nativeAuthIdForIssuer(rawUserId, DEFAULT_FABUSHI_API_BASE_URL);
  const isolatedAuthId = nativeAuthIdForIssuer(rawUserId, FABUSHI_CI_ISOLATED_AUTH_BASE_URL);
  const expectedIsolatedAuthId = `fabushi-ci-isolated:v1:${createHash("sha256")
    .update(`fabushi-native-auth-id-v1\0${FABUSHI_CI_ISOLATED_AUTH_BASE_URL}\0${rawUserId}`, "utf8")
    .digest("hex")}`;
  assert.equal(prodAuthId, rawUserId);
  assert.equal(isolatedAuthId, expectedIsolatedAuthId);
  assert.match(isolatedAuthId, /^fabushi-ci-isolated:v1:[0-9a-f]{64}$/u);
  assert.equal(isolatedAuthId, nativeAuthIdForIssuer(rawUserId, FABUSHI_CI_ISOLATED_AUTH_BASE_URL));
  assert.notEqual(
    createHash("sha256").update(prodAuthId).digest("hex"),
    createHash("sha256").update(isolatedAuthId).digest("hex"),
  );
});

test("FDN device-secret scope preserves production keys and separates equal IDs by issuer", () => {
  const previousProductionScope = createHash("sha256")
    .update("123\0device-a", "utf8")
    .digest("hex");
  assert.equal(isProductionFabushiIssuer(DEFAULT_FABUSHI_API_BASE_URL), true);
  assert.equal(isProductionFabushiIssuer(FABUSHI_CI_ISOLATED_AUTH_BASE_URL), false);
  assert.equal(
    nativeRemoteComputerDeviceScope("123", DEFAULT_FABUSHI_API_BASE_URL, "device-a"),
    previousProductionScope,
  );
  assert.notEqual(
    nativeRemoteComputerDeviceScope("123", DEFAULT_FABUSHI_API_BASE_URL, "device-a"),
    nativeRemoteComputerDeviceScope("123", FABUSHI_CI_ISOLATED_AUTH_BASE_URL, "device-a"),
  );
});

test("session issuer is bound by the Native owner and missing stored issuer remains production-only", () => {
  assert.equal(normalizeFabushiCiSession(boundedCiSession(), {
    issuer: FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
    bindMissingIssuer: true,
  })?.issuer, FABUSHI_CI_ISOLATED_AUTH_BASE_URL);
  assert.equal(normalizeFabushiCiSession(boundedCiSession({ issuer: DEFAULT_FABUSHI_API_BASE_URL }), {
    issuer: FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
    bindMissingIssuer: true,
  }), null);
  assert.equal(normalizeFabushiSession(boundedCiSession(), {
    issuer: FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
  }), null);
});


test("Fabushi production Responses URL targets the deployed Codex-compatible first-party adapter", () => {
  assert.equal(
    DEFAULT_FABUSHI_RESPONSES_URL,
    "https://api.ombhrum.com/codex-deepseek/v1/responses",
  );
  const endpoint = new URL(DEFAULT_FABUSHI_RESPONSES_URL);
  assert.equal(endpoint.origin, new URL(DEFAULT_FABUSHI_API_BASE_URL).origin);
  assert.equal(endpoint.pathname, "/codex-deepseek/v1/responses");
});
