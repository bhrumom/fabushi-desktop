import assert from "node:assert/strict";
import * as crypto from "node:crypto";
import { promises as fs } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { runInNewContext } from "node:vm";

// NMR-31 / NMR-AC-22: execute the production account service, policy and
// credential-file writer and the secret-store functions imported by production.
// Electron's codec/userData boundary and the per-key API/HTTP are instrumented.
// Real private files exercise replacement, refresh and logout;
// these contracts do not attest an OS keychain or a deployed account service.
const desktopRequire = createRequire(new URL("../../../desktop/package.json", import.meta.url));
const { build } = desktopRequire("esbuild");
const serviceSource = new URL("./fabushi-account-service.ts", import.meta.url);
const compiled = await build({
  entryPoints: [fileURLToPath(serviceSource)],
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node24",
  write: false,
  logLevel: "silent",
  plugins: [{
    name: "instrument-account-secret-boundary",
    setup(builder) {
      builder.onResolve({ filter: /\/secrets\/secret-store\.js$/ }, () => ({
        path: "fixture:secret-store",
        external: true,
      }));
    },
  }],
});
const serviceCode = compiled.outputFiles[0].text;
const secretStoreSource = new URL("../../electron-main/secrets/secret-store.ts", import.meta.url);
const compiledStore = await build({
  stdin: {
    contents: `export { readSecret, writeSecret, deleteSecret } from ${JSON.stringify(fileURLToPath(secretStoreSource))};`,
    resolveDir: dirname(fileURLToPath(secretStoreSource)),
    sourcefile: "native-account-secret-store-fixture.ts",
    loader: "ts",
  },
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node24",
  write: false,
  logLevel: "silent",
  plugins: [{
    name: "isolate-unused-electron-diagnostics",
    setup(builder) {
      builder.onResolve({ filter: /desktop-edge-failures\.js$|\/telemetry\/sentry\.js$/ }, () => ({
        path: "fixture:diagnostics",
        external: true,
      }));
    },
  }],
});
const productionIssuer = "https://api.ombhrum.com";
const isolatedIssuer = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";
const legacyStoragePrefix = "fabushi-telegram-storage-key-v1:";
const legacyApplicationKey = "fabushi-telegram-application-credential-v1";
const nativeSessionKey = "fabushi-account-session-v1";
const isLegacyKey = (key) => key.startsWith(legacyStoragePrefix) || key === legacyApplicationKey;

async function fixture(t, legacy = {}) {
  const root = await fs.mkdtemp(join(tmpdir(), "fabushi-native-account-"));
  const credential = join(root, "private", "account.json");
  const remoteCredential = join(root, "private", "remote-computer.json");
  const secretFile = join(root, "secure", "sand-secrets.json");
  const secrets = new Map();
  const secretOperations = [];
  const randomByteRequests = [];
  const requests = [];
  const openedUrls = [];
  const runtimes = [];
  const tokens = new Map();
  let now = 1_793_610_000_000;
  let loginNumber = 0;
  let refreshNumber = 0;
  let refreshStatus = 200;
  let refreshBarrier;
  let selectedUserId = "native-a";
  let issuer = productionIssuer;

  function createSecretStore() {
    const module = { exports: {} };
    const electron = {
      app: {
        isPackaged: true,
        getPath(name) { assert.equal(name, "userData"); return dirname(secretFile); },
      },
      safeStorage: {
        isEncryptionAvailable() { return true; },
        getSelectedStorageBackend() { return "fixture-secure"; },
        encryptString(value) { return Buffer.from(`fixture-codec:${value}`, "utf8"); },
        decryptString(value) {
          const decoded = value.toString("utf8");
          assert.ok(decoded.startsWith("fixture-codec:"));
          return decoded.slice("fixture-codec:".length);
        },
      },
    };
    runInNewContext(compiledStore.outputFiles[0].text, {
      module,
      exports: module.exports,
      process,
      Buffer,
      URL,
      setTimeout,
      clearTimeout,
      require(name) {
        if (name === "electron") return electron;
        if (name === "fixture:diagnostics") return {
          reportDesktopEdgeFailure() {},
          captureSandSentryWarning() {},
        };
        return desktopRequire(name);
      },
    }, { filename: secretStoreSource.pathname });
    return module.exports;
  }
  let backingStore = createSecretStore();

  const secretStore = Object.fromEntries([
    ["readSecret", async (key) => {
      secretOperations.push({ operation: "read", key });
      assert.equal(isLegacyKey(key), false, "Native authentication must never read retained legacy secrets");
      const value = await backingStore.readSecret(key);
      if (value == null) secrets.delete(key);
      else secrets.set(key, value);
      return value;
    }],
    ["writeSecret", async (key, value) => {
      secretOperations.push({ operation: "write", key });
      assert.equal(isLegacyKey(key), false, "Native authentication must never create or rotate legacy secrets");
      await backingStore.writeSecret(key, value);
      secrets.set(key, value);
    }],
    ["deleteSecret", async (key) => {
      secretOperations.push({ operation: "delete", key });
      assert.equal(isLegacyKey(key), false, "Native authentication must never delete retained legacy secrets");
      await backingStore.deleteSecret(key);
      secrets.delete(key);
    }],
  ]);

  function session(userId = selectedUserId, sessionIssuer = issuer) {
    return {
      issuer: sessionIssuer,
      accessToken: `fixture-access-${userId}-${loginNumber}`,
      refreshToken: `fixture-refresh-${userId}-${loginNumber}`,
      accessTokenExpiresAt: now + 10 * 60_000,
      refreshTokenExpiresAt: now + 24 * 60 * 60_000,
      sessionId: `browser:${userId}-${loginNumber}`,
      deviceId: "native-fixture-device",
      username: `${userId}@example.test`,
      userId,
    };
  }

  const fetchImpl = async (url, init = {}) => {
    const parsed = new URL(url);
    assert.equal(parsed.origin, issuer);
    assert.equal(init.redirect, "error");
    const body = init.body == null ? null : JSON.parse(init.body);
    requests.push({ path: parsed.pathname, method: init.method ?? "GET", body });
    if (parsed.pathname === "/api/auth/browser/start") {
      loginNumber += 1;
      return Response.json({
        attemptId: `fixture-login-${loginNumber}`,
        loginUrl: `${issuer}/login?attempt=fixture-${loginNumber}`,
        pollSecret: "fixture-poll-only",
        expiresAt: now + 60_000,
        pollAfterMs: 250,
      });
    }
    if (parsed.pathname === `/api/auth/browser/attempts/fixture-login-${loginNumber}`) {
      const issued = session();
      tokens.set(issued.accessToken, issued);
      return Response.json({ status: "completed", session: issued });
    }
    if (parsed.pathname === "/api/auth/refresh") {
      const prior = JSON.parse(secrets.get(nativeSessionKey));
      assert.equal(body.refreshToken, prior.refreshToken);
      assert.equal(body.deviceId, prior.deviceId);
      if (refreshBarrier != null) {
        refreshBarrier.enter();
        await refreshBarrier.wait;
      }
      if (refreshStatus !== 200) return Response.json({ error: { code: "fixture-revoked" } }, { status: refreshStatus });
      refreshNumber += 1;
      const next = {
        ...prior,
        accessToken: `fixture-refreshed-access-${prior.userId}-${refreshNumber}`,
        refreshToken: `fixture-refreshed-refresh-${prior.userId}-${refreshNumber}`,
        accessTokenExpiresAt: now + 10 * 60_000,
      };
      tokens.set(next.accessToken, next);
      return Response.json(next);
    }
    const bearer = new Headers(init.headers).get("authorization")?.replace(/^Bearer /u, "");
    const authorized = tokens.get(bearer);
    assert.ok(authorized, `fixture account endpoint requires an issued Native bearer: ${parsed.pathname}`);
    if (parsed.pathname === "/api/auth/user-info") {
      return Response.json({ id: authorized.userId, username: authorized.username, displayName: "Native fixture" });
    }
    if (parsed.pathname === "/v1/computers/register") {
      assert.equal(body.deviceId, authorized.deviceId);
      return Response.json({ deviceId: authorized.deviceId });
    }
    if (parsed.pathname === "/v1/computers/heartbeat") return Response.json({ ok: true, lastSeenAt: now });
    if (parsed.pathname === "/api/auth/logout") {
      assert.equal(body.deviceId, authorized.deviceId);
      assert.equal(body.refreshToken, authorized.refreshToken);
      return Response.json({ ok: true });
    }
    throw new Error(`unexpected account request: ${parsed.pathname}`);
  };

  function stopTimers() {
    for (const runtime of runtimes) {
      for (const timer of runtime.timers) clearTimeout(timer);
      runtime.timers.clear();
    }
  }

  function createService() {
    const module = { exports: {} };
    const timers = new Set();
    runInNewContext(serviceCode, {
      module,
      exports: module.exports,
      process,
      Buffer,
      URL,
      AbortController,
      fetch: fetchImpl,
      setTimeout(callback, ms, ...args) {
        const timer = setTimeout(() => { timers.delete(timer); callback(...args); }, ms);
        timers.add(timer);
        return timer;
      },
      clearTimeout(timer) { timers.delete(timer); clearTimeout(timer); },
      require(name) {
        if (name === "fixture:secret-store") return secretStore;
        if (name === "node:crypto") return {
          ...crypto,
          randomBytes(size) { randomByteRequests.push(size); return crypto.randomBytes(size); },
        };
        return desktopRequire(name);
      },
    }, { filename: serviceSource.pathname });
    const service = new module.exports.FabushiAuthService({
      openExternal(url) { openedUrls.push(url); },
      env: { FABUSHI_API_BASE_URL: issuer },
      machineId: "native-fixture-device",
      hostAccessCredentialFile: credential,
      hostRemoteComputerCredentialFile: remoteCredential,
      fetchImpl,
      now: () => now,
    });
    runtimes.push({ timers, service });
    return { service, exports: module.exports };
  }

  const initial = createService();
  if (legacy.storage != null) {
    for (const retainedSession of [session(), session("native-b"), session("native-a", isolatedIssuer)]) {
      secrets.set(initial.exports.telegramStorageSecretName(retainedSession), legacy.storage);
    }
  }
  if (legacy.application != null) secrets.set(legacyApplicationKey, legacy.application);
  const legacySnapshot = [...secrets].filter(([key]) => isLegacyKey(key));
  const retainedCiphertexts = legacySnapshot.map(([key, value]) => [
    key, Buffer.from(`fixture-codec:${value}`, "utf8").toString("base64"),
  ]);
  await fs.mkdir(dirname(secretFile), { recursive: true, mode: 0o700 });
  await fs.writeFile(secretFile, JSON.stringify(Object.fromEntries(retainedCiphertexts)), { mode: 0o600 });

  t.after(async () => {
    stopTimers();
    await fs.rm(root, { recursive: true, force: true });
  });

  return {
    service: initial.service,
    secrets,
    credential,
    remoteCredential,
    requests,
    openedUrls,
    selectAccount(userId) { selectedUserId = userId; },
    restart(nextIssuer = issuer) {
      // A fresh service retains only disk/secure-store state; fixture-owned
      // timers are retired as they would be when the former process exits.
      stopTimers();
      issuer = nextIssuer;
      backingStore = createSecretStore();
      return createService().service;
    },
    enterRefreshWindow() { now = JSON.parse(secrets.get(nativeSessionKey)).accessTokenExpiresAt - 30_000; },
    rejectRefresh() { refreshStatus = 401; },
    holdRefresh() {
      let enter;
      let release;
      const entered = new Promise((resolve) => { enter = resolve; });
      const wait = new Promise((resolve) => { release = resolve; });
      refreshBarrier = { enter, wait };
      return { entered, release };
    },
    async assertDescriptor() {
      const stored = JSON.parse(secrets.get(nativeSessionKey));
      const actual = JSON.parse(await fs.readFile(credential, "utf8"));
      assert.deepEqual(actual, {
        version: 1,
        issuer: stored.issuer,
        accessToken: stored.accessToken,
        expiresAtMs: stored.accessTokenExpiresAt,
        sessionId: stored.sessionId,
        deviceId: stored.deviceId,
        userId: stored.userId,
      }, "Host handoff must contain exactly the seven Native v1 fields");
      if (process.platform !== "win32") {
        assert.equal((await fs.stat(credential)).mode & 0o777, 0o600);
        assert.equal((await fs.stat(dirname(credential))).mode & 0o777, 0o700);
      }
      assert.ok((await fs.readdir(dirname(credential))).every((name) => !name.endsWith(".tmp")));
      return actual;
    },
    async assertLegacyUntouched() {
      assert.deepEqual(secretOperations.filter(({ key }) => isLegacyKey(key)), []);
      assert.deepEqual([...secrets].filter(([key]) => isLegacyKey(key)), legacySnapshot);
      const persisted = JSON.parse(await fs.readFile(secretFile, "utf8"));
      assert.deepEqual(Object.entries(persisted).filter(([key]) => isLegacyKey(key)), retainedCiphertexts,
        "Native secure-store mutations must preserve every retained legacy ciphertext byte");
      assert.ok(!randomByteRequests.includes(32), "Native credentials must not generate a legacy storage key");
    },
    async assertLoggedOut() {
      assert.equal(secrets.has(nativeSessionKey), false);
      assert.equal(await backingStore.readSecret(nativeSessionKey), null);
      await assert.rejects(fs.stat(credential), { code: "ENOENT" });
      await assert.rejects(fs.stat(remoteCredential), { code: "ENOENT" });
    },
  };
}

const legacyCases = [
  { name: "valid retained storage/application secrets", storage: Buffer.alloc(32, 17).toString("base64url"), application: JSON.stringify({ apiId: 123456, apiHash: "abcdef0123456789abcdef0123456789" }) },
  { name: "corrupt retained storage and non-JSON application secret", storage: "old-key-must-not-be-rotated", application: "{not-json\n" },
  { name: "corrupt retained storage and invalid application credential", storage: "legacy\0opaque", application: JSON.stringify({ apiId: -1, apiHash: "invalid" }) },
  { name: "new account without legacy secrets" },
];

for (const scenario of legacyCases) {
  test(`NMR-NATIVE-ACCOUNT: login/refresh/restart/logout preserves ${scenario.name}`, async (t) => {
    const f = await fixture(t, scenario);
    const status = await f.service.login();
    assert.equal(status.kind, "logged-in");
    assert.equal(status.authId, "native-a");
    assert.equal(f.openedUrls.length, 1);
    await f.assertDescriptor();
    await f.assertLegacyUntouched();

    const previousBytes = await fs.readFile(f.credential, "utf8");
    const previousFile = await fs.open(f.credential, "r");
    try {
      f.enterRefreshWindow();
      assert.equal((await f.service.getStatus()).kind, "logged-in");
      const refreshed = await f.assertDescriptor();
      assert.match(refreshed.accessToken, /^fixture-refreshed-access-/u);
      assert.equal(await previousFile.readFile("utf8"), previousBytes,
        "refresh replaces the descriptor atomically without overwriting the previous open inode");
    } finally {
      await previousFile.close();
    }
    await f.assertLegacyUntouched();

    const restarted = f.restart();
    assert.equal((await restarted.getStatus()).authId, "native-a");
    assert.equal(await restarted.getValidAccessToken(), JSON.parse(f.secrets.get(nativeSessionKey)).accessToken);
    await f.assertDescriptor();
    await f.assertLegacyUntouched();
    assert.equal((await restarted.logout()).kind, "logged-out");
    await f.assertLoggedOut();
    assert.equal((await restarted.getStatus()).kind, "logged-out");
    await f.assertLegacyUntouched();
    assert.equal(f.requests.filter(({ path }) => path === "/api/auth/refresh").length, 1);
    assert.equal(f.requests.filter(({ path }) => path === "/api/auth/logout").length, 1);
  });
}

test("NMR-NATIVE-ACCOUNT: account switch replaces Native identity without touching either legacy account", async (t) => {
  const f = await fixture(t, legacyCases[0]);
  assert.equal((await f.service.login()).authId, "native-a");
  const before = await f.assertDescriptor();
  f.selectAccount("native-b");
  assert.equal((await f.service.login()).authId, "native-b");
  const after = await f.assertDescriptor();
  assert.equal(after.userId, "native-b");
  assert.notEqual(after.accessToken, before.accessToken);
  assert.notEqual(after.sessionId, before.sessionId);
  await f.assertLegacyUntouched();
  assert.equal((await f.service.logout()).kind, "logged-out");
  await f.assertLoggedOut();
  await f.assertLegacyUntouched();
});

test("NMR-NATIVE-ACCOUNT: restart denies another issuer before isolated login establishes its own Native identity", async (t) => {
  const f = await fixture(t, legacyCases[1]);
  assert.equal((await f.service.login()).authId, "native-a");
  const storedProduction = f.secrets.get(nativeSessionKey);
  const isolated = f.restart(isolatedIssuer);
  assert.equal((await isolated.getStatus()).kind, "logged-out");
  assert.equal(f.secrets.get(nativeSessionKey), storedProduction);
  await assert.rejects(fs.stat(f.credential), { code: "ENOENT" });
  const loggedIn = await isolated.login();
  assert.equal(loggedIn.kind, "logged-in");
  assert.match(loggedIn.authId, /^fabushi-ci-isolated:v1:[0-9a-f]{64}$/u);
  assert.equal((await f.assertDescriptor()).issuer, isolatedIssuer);
  await f.assertLegacyUntouched();
  assert.equal((await isolated.logout()).kind, "logged-out");
  await f.assertLoggedOut();
  await f.assertLegacyUntouched();
});

test("NMR-NATIVE-ACCOUNT: rejected refresh clears Native authority and preserves malformed legacy data", async (t) => {
  const f = await fixture(t, legacyCases[2]);
  assert.equal((await f.service.login()).kind, "logged-in");
  f.enterRefreshWindow();
  f.rejectRefresh();
  const status = await f.service.getStatus();
  assert.equal(status.kind, "logged-out");
  assert.match(status.errorMessage, /401 fixture-revoked/u);
  await f.assertLoggedOut();
  assert.equal((await f.service.getStatus()).kind, "logged-out");
  await f.assertLegacyUntouched();
});

test("NMR-NATIVE-ACCOUNT: logout fences a delayed refresh before it can republish any credential", async (t) => {
  const f = await fixture(t, legacyCases[0]);
  assert.equal((await f.service.login()).kind, "logged-in");
  f.enterRefreshWindow();
  const held = f.holdRefresh();
  const refreshing = f.service.getStatus();
  await held.entered;
  const loggingOut = f.service.logout();
  held.release();
  const [refreshResult, logoutResult] = await Promise.all([refreshing, loggingOut]);
  assert.equal(refreshResult.kind, "logged-out");
  assert.equal(logoutResult.kind, "logged-out");
  await f.assertLoggedOut();
  assert.equal((await f.service.getStatus()).kind, "logged-out");
  await f.assertLoggedOut();
  await f.assertLegacyUntouched();
});
