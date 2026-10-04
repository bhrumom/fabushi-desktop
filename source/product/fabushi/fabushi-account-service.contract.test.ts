import assert from "node:assert/strict";
import test from "node:test";

import {
  DEFAULT_FABUSHI_API_BASE_URL,
  normalizeFabushiApiBaseUrl,
  normalizeFabushiCiSession,
  normalizeFabushiSession,
} from "./fabushi-account-service.js";

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
});
