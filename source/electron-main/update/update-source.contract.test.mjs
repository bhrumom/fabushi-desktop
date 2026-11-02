import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { EventEmitter } from "node:events";
import { mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { runInThisContext } from "node:vm";

// EUS-01..06: execute the existing shipping service and its real wiring/consumers.
// Fetch, clock and installer ports are controlled; this is not signed-package acceptance.
const desktopRequire = createRequire(new URL("../../../desktop/package.json", import.meta.url));
const { buildSync } = desktopRequire("esbuild");
const code = buildSync({
  stdin: {
    contents: `
      export { SandUpdateService } from "./sand-update-service.ts";
      export { createUpdateServiceWiring } from "./update-wiring.ts";
      export { configuredUpdateFeedBaseUrl, resolveUpdateFeedSource } from "./update-feed.ts";
      export { createUpdateApplyMarkerStore } from "./apply-marker.ts";
      export { createReleaseMetadata } from "./release-metadata.ts";
      export { updateCheckToTelemetry } from "./update-telemetry.ts";
      export { updateStatusMessage } from "../../../frontend/src/recovered/features/settings/overlay/updates.ts";
    `,
    resolveDir: dirname(fileURLToPath(import.meta.url)), loader: "ts", sourcefile: "update-source-contract-entry.ts",
  },
  bundle: true, write: false, platform: "node", format: "cjs", target: "node24", packages: "external",
}).outputFiles[0].text;
const loaded = { exports: {} };
runInThisContext(`(function(require, module, exports) { ${code}\n})`, { filename: "update-source-contract-bundle.cjs" })(desktopRequire, loaded, loaded.exports);
const { SandUpdateService, createUpdateServiceWiring, configuredUpdateFeedBaseUrl, resolveUpdateFeedSource, createUpdateApplyMarkerStore, createReleaseMetadata, updateCheckToTelemetry, updateStatusMessage } = loaded.exports;
const approvedFeed = "https://api.ombhrum.com/updates";
const copy = (value) => JSON.parse(JSON.stringify(value));
const settled = () => new Promise((resolve) => setImmediate(resolve));

class Clock {
  time = 1_000_000;
  tasks = [];
  now = () => this.time;
  monotonicNow = () => this.time;
  schedule(delay, callback) {
    const task = { delay, callback, active: true };
    this.tasks.push(task);
    return { dispose: () => { task.active = false; } };
  }
  get activeCount() { return this.tasks.filter((task) => task.active).length; }
  fire(delay) {
    const task = this.tasks.find((item) => item.active && item.delay === delay);
    assert.ok(task, `no scheduled timer at ${delay}`);
    task.active = false;
    this.time += delay;
    task.callback();
  }
}

function harness(t) {
  const root = mkdtempSync(join(tmpdir(), "fabushi-update-source-"));
  const clock = new Clock();
  const requests = [], checks = [], statuses = [], applies = [], logs = [], verified = [], installed = [];
  const services = new Set();
  let track = null, optIn = false, quits = 0;
  let fetchResponse = async () => new Response(null, { status: 204 });
  const squirrel = Object.assign(new EventEmitter(), {
    setFeedURL() { assert.fail("unexpected Squirrel feed while source check is gated"); },
    checkForUpdates() { assert.fail("unexpected Squirrel check while source check is gated"); },
    quitAndInstall() { assert.fail("unexpected Squirrel install while source check is gated"); },
  });
  const windows = {
    getDisabledReason: () => null, // Explicitly authorized controlled port; native publisher cases are separate.
    async verifySignature(path) { verified.push({ path, bytes: readFileSync(path) }); },
    quit() { quits += 1; },
    installOnQuit(path, options) { installed.push({ path, options }); },
  };
  const settingsStore = {
    getUpdateTrackOverride: () => track,
    setUpdateTrackOverride: (value) => { track = value; },
    getAutoUpdateWhenIdleOptIn: () => optIn,
    setAutoUpdateWhenIdleOptIn: (value) => { optIn = value; },
  };
  const defaults = {
    currentVersion: "1.2.78", buildDefaultTrack: "stable", platform: "win32", arch: "x64",
    tempDir: root, machineId: "private-machine-fixture", windows, clock, now: clock.now,
    initialCheckDelayMs: 10, checkIntervalMs: 50, checkIntervalJitterMs: 0,
    getTrackOverride: settingsStore.getUpdateTrackOverride, persistTrackOverride: settingsStore.setUpdateTrackOverride,
    getAutoUpdateWhenIdleOptIn: settingsStore.getAutoUpdateWhenIdleOptIn,
    persistAutoUpdateWhenIdleOptIn: settingsStore.setAutoUpdateWhenIdleOptIn,
    onStatusChange: (value) => statuses.push(copy(value)), reportCheck: (value) => checks.push(copy(value)),
    reportApply: (value) => applies.push(copy(value)), log: (value) => logs.push(value),
    fetchImpl: async (url, init) => { requests.push({ url: String(url), init }); return await fetchResponse(url, init); },
  };
  const create = (options = {}) => {
    const service = new SandUpdateService({ ...defaults, ...options });
    services.add(service);
    return service;
  };
  t.after(() => { for (const service of services) service.dispose(); rmSync(root, { recursive: true, force: true }); });
  return {
    root, clock, requests, checks, statuses, applies, logs, verified, installed, squirrel, windows,
    settingsStore, defaults, create, get quits() { return quits; },
    setFetch(value) { fetchResponse = value; },
  };
}

test("EUS-01/03 no implicit feed for startup, manual, track, minimum-version or backend checks", async (t) => {
  for (const feedBaseUrl of [undefined, "", "  \t\n"]) {
    const h = harness(t);
    const service = h.create({ feedBaseUrl, getMinimumVersion: () => "99.0.0" });
    service.start();
    await service.checkForUpdates({ trigger: "explicit" });
    await service.setTrackOverride("dogfood");
    service.noteMinimumVersionMayHaveChanged();
    service.noteBackendUpdateRequirement(true);
    service.noteReleaseTrackGateMayHaveChanged();
    service.applyDisabledReason(null);
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "update-source-unconfigured" });
    assert.equal(service.getStatus().trackOverride, "dogfood");
    assert.equal(service.getStatus().isBelowMinimumVersion, false);
    assert.equal(service.quitAndInstall(), false);
    assert.equal(h.clock.activeCount, 0);
    assert.equal(h.requests.length, 0);
    assert.equal(h.verified.length, 0);
    assert.equal(h.installed.length, 0);
  }
});

test("EUS-02 invalid and foreign sources fail before requests without echoing configuration", async (t) => {
  for (const feedBaseUrl of [
    "not a URL", "https://api2.cursor.sh/updates", "https://updates.example.test/feed",
    "https://api.ombhrum.com.evil.test/updates", "http://api.ombhrum.com/updates",
    "https://secret-user:secret-password@api.ombhrum.com/updates",
    "https://api.ombhrum.com/updates?token=secret-query", "https://api.ombhrum.com/updates#secret-fragment",
    "https://api.ombhrum.com:8443/updates", "file:///tmp/update-feed", "data:text/plain,feed",
  ]) {
    const h = harness(t);
    const service = h.create({ feedBaseUrl });
    service.start();
    await service.checkForUpdates({ trigger: "explicit" });
    service.applyDisabledReason(null);
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "update-source-invalid" });
    assert.equal(h.requests.length, 0);
    assert.equal(h.clock.activeCount, 0);
    const observable = JSON.stringify([service.getStatus(), h.checks, h.statuses, h.logs]);
    assert.equal(observable.includes(feedBaseUrl), false);
    assert.equal(/secret-user|secret-password|secret-query|secret-fragment/.test(observable), false);
  }
});

test("EUS-02 approved production, isolated CI and loopback origins retain explicit paths", () => {
  for (const [input, expected] of [
    [" https://api.ombhrum.com:443/updates/// ", "https://api.ombhrum.com/updates"],
    ["https://fabushi-ci-isolated-auth.bhrumom.workers.dev/releases", "https://fabushi-ci-isolated-auth.bhrumom.workers.dev/releases"],
    ["http://127.0.0.1:42123/local-feed", "http://127.0.0.1:42123/local-feed"],
    ["http://localhost:42123/updates", "http://localhost:42123/updates"],
    ["http://[::1]:42123/updates", "http://[::1]:42123/updates"],
  ]) assert.deepEqual(resolveUpdateFeedSource(input), { baseUrl: expected, disabledReason: null });
});

test("EUS-01 primary configuration is authoritative including blank and invalid values", () => {
  assert.equal(configuredUpdateFeedBaseUrl({}), undefined);
  assert.equal(configuredUpdateFeedBaseUrl({ SAND_UPDATE_FEED_BASE_URL: approvedFeed }), approvedFeed);
  for (const primary of ["", " ", "invalid", approvedFeed]) {
    assert.equal(configuredUpdateFeedBaseUrl({ FABUSHI_UPDATE_FEED_BASE_URL: primary, SAND_UPDATE_FEED_BASE_URL: "http://127.0.0.1:42123/legacy" }), primary);
  }
});

test("EUS-01/05 actual production wiring uses the same source precedence and service", async (t) => {
  for (const [env, expectedUrl, expectedReason] of [
    [{}, null, "update-source-unconfigured"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: "", SAND_UPDATE_FEED_BASE_URL: approvedFeed }, null, "update-source-unconfigured"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: "https://api2.cursor.sh/updates", SAND_UPDATE_FEED_BASE_URL: approvedFeed }, null, "update-source-invalid"],
    [{ SAND_UPDATE_FEED_BASE_URL: approvedFeed }, approvedFeed, null],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, SAND_UPDATE_FEED_BASE_URL: "https://api2.cursor.sh/updates" }, approvedFeed, null],
  ]) {
    const h = harness(t);
    const service = createUpdateServiceWiring({
      currentVersion: "1.2.78", buildDefaultTrack: "stable", machineId: "private-machine-fixture", platform: "win32", env,
      electron: { app: { getPath: () => h.root, quit() {} }, autoUpdater: h.squirrel, powerMonitor: { getSystemIdleState: () => "active", getSystemIdleTime: () => 0 } },
      settingsStore: h.settingsStore, getExperimentService: () => null, getHostStatus: async () => ({ isBusy: false }),
      emitStatus: (status) => h.statuses.push(copy(status)), reportCheck: (report) => h.checks.push(copy(report)),
      createWindowsInstaller: () => h.windows, createService: (options) => h.create(options),
    });
    assert.equal(service.updatePlatform, "win32-x64-user");
    await service.checkForUpdates({ trigger: "explicit" });
    if (expectedReason != null) {
      assert.deepEqual(service.getStatus().state, { type: "disabled", reason: expectedReason });
      assert.equal(h.requests.length, 0);
      assert.equal(h.clock.activeCount, 0);
    } else {
      assert.equal(h.requests.length, 1);
      assert.equal(h.requests[0].url, `${expectedUrl}/api/update/win32-x64-user/sand/1.2.78/private-machine-fixture/stable`);
    }
  }
});

test("EUS-03 existing disabled reasons retain precedence and clearing them cannot bypass source gating", async (t) => {
  for (const disabledReason of ["disabled-by-env", "lab-build", "not-packaged", "unsupported-platform"]) {
    const h = harness(t);
    const service = h.create({ disabledReason });
    service.start();
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: disabledReason });
    service.applyDisabledReason(null);
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "update-source-unconfigured" });
    await service.checkForUpdates({ trigger: "explicit" });
    assert.equal(h.requests.length, 0);
    assert.equal(h.clock.activeCount, 0);
  }
  const devMetadata = (env) => createReleaseMetadata({
    isLabBuild: false, app: { getVersion: () => "1.2.78", isPackaged: false }, platform: "linux", env,
  });
  assert.equal(await devMetadata({ FABUSHI_UPDATE_FEED_BASE_URL: "http://127.0.0.1:42123/updates" }).computeUpdateDisabledReasonLive(), null);
  assert.equal(await devMetadata({ SAND_UPDATE_FEED_BASE_URL: "http://127.0.0.1:42123/updates" }).computeUpdateDisabledReasonLive(), null);
  assert.equal(await devMetadata({ FABUSHI_UPDATE_FEED_BASE_URL: "", SAND_UPDATE_FEED_BASE_URL: "http://127.0.0.1:42123/updates" }).computeUpdateDisabledReasonLive(), "not-packaged");
  assert.equal(await devMetadata({ FABUSHI_UPDATE_FEED_BASE_URL: " \t ", SAND_UPDATE_FEED_BASE_URL: "http://127.0.0.1:42123/updates" }).computeUpdateDisabledReasonLive(), "not-packaged");
});

test("EUS-05 source gating reaches the actual Settings projection and non-retryable telemetry", (t) => {
  for (const [feedBaseUrl, reason, code, expectedText] of [
    [undefined, "update-source-unconfigured", "SAND-E0513", "Updates are unavailable until an update source is configured"],
    ["https://api2.cursor.sh/updates", "update-source-invalid", "SAND-E0514", "The configured update source is invalid"],
  ]) {
    const h = harness(t);
    const service = h.create({ feedBaseUrl });
    service.start();
    const projected = updateStatusMessage(service.getStatus());
    assert.deepEqual(projected, { text: expectedText, tone: "default" });
    const telemetry = updateCheckToTelemetry(h.checks[0]);
    assert.equal(telemetry.metadata.error_code, code);
    assert.equal(telemetry.metadata.error_retryable, "false");
    assert.equal(telemetry.metadata.cause, reason);
  }
});

test("EUS-04 approved explicit feed preserves request contract and genuine 204 no-update settlement", async (t) => {
  const h = harness(t);
  const service = h.create({ feedBaseUrl: approvedFeed });
  service.start();
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  assert.equal(h.requests[0].url, `${approvedFeed}/api/update/win32-x64-user/sand/1.2.78/private-machine-fixture/stable`);
  assert.equal(h.requests[0].init.redirect, "error");
  assert.equal(h.requests[0].init.headers["cache-control"], "no-cache");
  assert.ok(h.requests[0].init.signal instanceof AbortSignal);
  assert.equal(service.getStatus().state.type, "idle");
  assert.equal(service.getStatus().state.lastCheck.result, "up-to-date");
  assert.equal(h.checks.at(-1).result, "no_update");
});

test("EUS-02/04 approved feed HTTP failure is an error with no fallback or private request URL", async (t) => {
  const h = harness(t);
  h.setFetch(async () => new Response(null, { status: 503 }));
  const service = h.create({ feedBaseUrl: approvedFeed });
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  assert.equal(service.getStatus().state.lastCheck.result, "error");
  assert.equal(service.getStatus().state.lastCheck.errorMessage, "Update server returned HTTP 503");
  assert.equal(h.checks.at(-1).error.code, "SAND-E0502");
  assert.equal(JSON.stringify([h.checks, h.statuses, h.logs]).includes("private-machine-fixture"), false);
  assert.equal(JSON.stringify([h.checks, h.statuses, h.logs]).includes(approvedFeed), false);
});

test("EUS-02 real loopback manifest redirect is refused before reaching the redirect target", async (t) => {
  let requests = 0, redirectedRequests = 0;
  const server = createServer((request, response) => {
    requests += 1;
    if (request.url === "/redirect-target") { redirectedRequests += 1; response.writeHead(204); response.end(); return; }
    response.writeHead(302, { location: "/redirect-target" }); response.end();
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  t.after(() => new Promise((resolve) => { server.close(resolve); server.closeAllConnections(); }));
  const h = harness(t);
  h.setFetch((url, init) => fetch(url, init));
  const service = h.create({ feedBaseUrl: `http://127.0.0.1:${server.address().port}/updates` });
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(requests, 1);
  assert.equal(redirectedRequests, 0);
  assert.equal(service.getStatus().state.lastCheck.result, "error");
  assert.equal(h.checks.at(-1).error.code, "SAND-E0501");
});

test("EUS-04 approved source retains startup and periodic timers", async (t) => {
  const h = harness(t);
  const service = h.create({ feedBaseUrl: approvedFeed });
  service.start();
  assert.equal(h.clock.activeCount, 2);
  h.clock.fire(10);
  await settled();
  assert.equal(h.requests.length, 1);
  assert.equal(h.checks.at(-1).trigger, "startup");
  h.clock.fire(50);
  await settled();
  assert.equal(h.requests.length, 2);
  assert.equal(h.checks.at(-1).trigger, "periodic");
  service.dispose();
  assert.equal(h.clock.activeCount, 0);
});

test("EUS-03/04 disabling an approved service aborts its exact in-flight check and allows explicit recovery", async (t) => {
  const h = harness(t);
  h.setFetch((_url, init) => new Promise((_resolve, reject) => {
    init.signal.addEventListener("abort", () => reject(new Error("controlled cancellation")), { once: true });
  }));
  const service = h.create({ feedBaseUrl: approvedFeed });
  service.start();
  const pending = service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  service.applyDisabledReason("disabled-by-env");
  assert.equal(h.requests[0].init.signal.aborted, true);
  await pending;
  assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "disabled-by-env" });
  assert.equal(h.clock.activeCount, 0);
  h.setFetch(async () => new Response(null, { status: 204 }));
  service.applyDisabledReason(null);
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 2);
  assert.equal(service.getStatus().state.lastCheck.result, "up-to-date");
});

test("EUS-03 disposal blocks later manual checks and startup timers", async (t) => {
  const h = harness(t);
  const service = h.create({ feedBaseUrl: approvedFeed });
  service.start();
  service.dispose();
  service.start();
  service.applyDisabledReason(null);
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 0);
  assert.equal(h.clock.activeCount, 0);
});

test("EUS-03/04 a new service recovers after explicit configuration is corrected", async (t) => {
  const h = harness(t);
  const blocked = h.create();
  blocked.start();
  await blocked.setTrackOverride("dogfood");
  blocked.setAutoUpdateWhenIdleOptIn(true);
  blocked.dispose();
  const recovered = h.create({ feedBaseUrl: approvedFeed });
  recovered.start();
  await recovered.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  assert.equal(recovered.getStatus().trackOverride, "dogfood");
  assert.equal(recovered.getStatus().autoUpdateWhenIdleOptIn, true);
  assert.equal(h.requests[0].url, `${approvedFeed}/api/update/win32-x64-user/sand-dogfood/1.2.78/private-machine-fixture/stable?trackChanged=1`);
});

test("EUS-04 real persisted apply markers settle on disabled restart without touching user data", (t) => {
  for (const [feedBaseUrl, currentVersion, phase, expectedOutcomes] of [
    [undefined, "1.2.79", "requested", ["confirmed"]],
    ["https://api2.cursor.sh/updates", "1.2.78", "spawned", ["spawned", "stale_version"]],
  ]) {
    const h = harness(t);
    const markerPath = join(h.root, "sand-update-apply-marker.json");
    const sentinelPath = join(h.root, "existing-session.json");
    const sentinel = Buffer.from('{"session":"existing-user-data","version":7}\n');
    writeFileSync(sentinelPath, sentinel);
    const previous = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    previous.recordRequested({ mechanism: "nsis", targetVersion: "1.2.79", fromVersion: "1.2.78" });
    if (phase === "spawned") previous.recordSpawned();
    assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).phase, phase);
    const restartedMarker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    const service = h.create({ feedBaseUrl, currentVersion, applyMarker: restartedMarker });
    service.start();
    assert.deepEqual(h.applies.map((report) => report.outcome), expectedOutcomes);
    assert.equal(restartedMarker.peek(), null);
    assert.equal(existsSync(markerPath), false);
    assert.deepEqual(readFileSync(sentinelPath), sentinel);
    assert.equal(service.getStatus().state.type, "disabled");
    assert.equal(h.requests.length, 0);
    assert.equal(h.installed.length, 0);
  }
});

test("EUS-04 approved explicit feed retains verified download, installer staging and apply marker owner", async (t) => {
  const h = harness(t);
  const bytes = Buffer.from("controlled installer payload; not a signed production package");
  const sha256hash = createHash("sha256").update(bytes).digest("hex");
  const installerUrl = "https://api.ombhrum.com/releases/fabushi-contract.exe";
  h.setFetch(async (url) => String(url) === installerUrl
    ? new Response(bytes, { status: 200, headers: { "content-length": String(bytes.length) } })
    : Response.json({ version: "1.2.79", url: installerUrl, sha256hash }));
  const markerPath = join(h.root, "sand-update-apply-marker.json");
  const marker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
  const service = h.create({ feedBaseUrl: approvedFeed, applyMarker: marker });
  service.start();
  await service.checkForUpdates({ trigger: "explicit" });
  assert.deepEqual(service.getStatus().state, { type: "ready", version: "1.2.79" });
  assert.equal(h.requests.length, 2);
  assert.equal(h.verified.length, 1);
  assert.deepEqual(h.verified[0].bytes, bytes);
  assert.equal(marker.peek().phase, "staged");
  assert.equal(service.quitAndInstall(), true);
  assert.equal(h.quits, 1);
  assert.equal(marker.peek().phase, "requested");
  service.applyStagedOnQuit();
  assert.equal(h.installed.length, 1);
  assert.equal(h.installed[0].path, h.verified[0].path);
  assert.equal(h.installed[0].options.forceRun, true);
  assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).phase, "spawned");
});

