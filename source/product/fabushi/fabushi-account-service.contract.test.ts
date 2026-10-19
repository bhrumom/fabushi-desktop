import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import { FabushiAuthService } from "./fabushi-account-service.js";
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
    issuer: DEFAULT_FABUSHI_API_BASE_URL,
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

test("Fabushi CI session rejects missing issuer even when a caller offers to bind it", () => {
  const missingIssuer = boundedCiSession();
  delete (missingIssuer as Record<string, unknown>).issuer;

  assert.equal(normalizeFabushiCiSession(missingIssuer), null);
  assert.equal(normalizeFabushiCiSession(missingIssuer, {
    issuer: DEFAULT_FABUSHI_API_BASE_URL,
    bindMissingIssuer: true,
  }), null);
  assert.equal(normalizeFabushiCiSession(missingIssuer, {
    issuer: FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
    bindMissingIssuer: true,
  }), null);
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
  }), null);
  assert.equal(normalizeFabushiCiSession(boundedCiSession({ issuer: FABUSHI_CI_ISOLATED_AUTH_BASE_URL }), {
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

test("logout fences an in-flight current-session read before stale status or Host credential write", async () => {
  interface RaceSession {
    issuer: string;
    accessToken: string;
    accessTokenExpiresAt: number;
    sessionId: string;
    deviceId: string;
    username: string;
    userId: string;
  }
  interface ServiceProbe {
    operationEpoch: number;
    ciSessionDisabled: boolean;
    logoutInProgress: boolean;
    readCiSession(): Promise<null>;
    readStoredSession(): Promise<RaceSession | null>;
    settleRemoteComputerTarget(session: RaceSession): Promise<void>;
    writeHostCredential(session: RaceSession): Promise<void>;
    clearStoredSession(): Promise<void>;
    clearHostCredential(): Promise<void>;
    clearHostRemoteComputerCredential(): Promise<void>;
    stopRemoteComputerHeartbeat(): void;
  }

  const session: RaceSession = {
    issuer: DEFAULT_FABUSHI_API_BASE_URL,
    accessToken: "race-test-access-token",
    accessTokenExpiresAt: Date.now() + 10 * 60_000,
    sessionId: "browser:race-test",
    deviceId: "device-race-test",
    username: "race@example.com",
    userId: "race-user",
  };
  const service = new FabushiAuthService({
    openExternal: () => {},
    env: {},
    machineId: "device-race-test",
    hostAccessCredentialFile: "/tmp/fabushi-auth-race-contract.json",
  });
  const probe = service as unknown as ServiceProbe;
  let storedSession: RaceSession | null = session;
  let credentialWrites = 0;
  let credentialClears = 0;
  let clearStarted = false;
  let enterSettlement!: () => void;
  let releaseSettlement!: () => void;
  const settlementEntered = new Promise<void>((resolve) => { enterSettlement = resolve; });
  const settlementGate = new Promise<void>((resolve) => { releaseSettlement = resolve; });
  probe.readCiSession = async () => null;
  probe.readStoredSession = async () => storedSession;
  probe.settleRemoteComputerTarget = async () => {
    enterSettlement();
    await settlementGate;
  };
  probe.writeHostCredential = async () => { credentialWrites += 1; };
  probe.clearStoredSession = async () => { clearStarted = true; storedSession = null; };
  probe.clearHostCredential = async () => { credentialClears += 1; };
  probe.clearHostRemoteComputerCredential = async () => {};
  probe.stopRemoteComputerHeartbeat = () => {};

  const statusPromise = service.getStatus();
  await settlementEntered;
  const tokenPromise = service.getValidAccessToken().then(() => "unexpected-token", () => "denied");
  const epochBeforeLogout = probe.operationEpoch;
  const logoutPromise = service.logout();

  assert.equal(probe.ciSessionDisabled, true);
  assert.equal(probe.operationEpoch, epochBeforeLogout + 1);
  assert.equal(probe.logoutInProgress, true);
  assert.equal(clearStarted, false, "logout must drain pre-fence work before final cleanup");
  releaseSettlement();

  const [status, logoutStatus, tokenResult] = await Promise.all([statusPromise, logoutPromise, tokenPromise]);
  assert.equal(status.kind, "logged-out");
  assert.equal(logoutStatus.kind, "logged-out");
  assert.equal(tokenResult, "denied");
  assert.equal(credentialWrites, 0, "stale currentSession must not write a Host credential");
  assert.ok(credentialClears > 0, "logout must perform final Host credential cleanup");
  assert.equal((await service.getStatus()).kind, "logged-out");
});


test("CI logout never selects an unrelated stored session for server revocation", async () => {
  const service = new FabushiAuthService({
    openExternal: () => {},
    env: { FABUSHI_CI_ACCOUNT_SESSION_FILE: "/private-ram/ordinary.pipe" },
    machineId: "device-ci-revocation-contract",
    hostAccessCredentialFile: "/private-ram/credential.json",
  });
  const probe = service as unknown as {
    readStoredSession(): Promise<null>;
    clearStoredSession(): Promise<void>;
    clearHostCredential(): Promise<void>;
    clearHostRemoteComputerCredential(): Promise<void>;
    stopRemoteComputerHeartbeat(): void;
  };
  let storedReads = 0;
  probe.readStoredSession = async () => { storedReads += 1; return null; };
  probe.clearStoredSession = async () => {};
  probe.clearHostCredential = async () => {};
  probe.clearHostRemoteComputerCredential = async () => {};
  probe.stopRemoteComputerHeartbeat = () => {};
  assert.equal((await service.logout()).kind, "logged-out");
  assert.equal(storedReads, 0, "an inactive stored session must not be selected for CI logout");
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
