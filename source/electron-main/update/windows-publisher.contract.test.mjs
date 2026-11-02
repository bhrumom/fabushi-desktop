import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { EventEmitter } from "node:events";
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { runInThisContext } from "node:vm";

// EUS-WIN-01..06: execute the real owners with explicitly artificial certificate
// records and a controlled native probe. No fixture is a Fabushi signer approval,
// a Windows Authenticode execution, or signed-package/install acceptance.
const desktopRequire = createRequire(new URL("../../../desktop/package.json", import.meta.url));
const { buildSync } = desktopRequire("esbuild");
const code = buildSync({
  stdin: {
    contents: [
      'export { SandWindowsInstaller, SandInstallerSignatureError, resolveWindowsUpdatePublisherPolicy } from "./win32-installer.ts";',
      'export { SandUpdateService } from "./sand-update-service.ts";',
      'export { createUpdateServiceWiring } from "./update-wiring.ts";',
      'export { createUpdateApplyMarkerStore } from "./apply-marker.ts";',
      'export { updateCheckToTelemetry } from "./update-telemetry.ts";',
      'export { updateStatusMessage } from "../../../frontend/src/recovered/features/settings/overlay/updates.ts";',
    ].join("\n"),
    resolveDir: dirname(fileURLToPath(import.meta.url)), loader: "ts", sourcefile: "windows-publisher-contract-entry.ts",
  },
  bundle: true, write: false, platform: "node", format: "cjs", target: "node24", packages: "external",
}).outputFiles[0].text;
const loaded = { exports: {} };
runInThisContext("(function(require, module, exports) { " + code + "\n})", { filename: "windows-publisher-contract-bundle.cjs" })(desktopRequire, loaded, loaded.exports);
const {
  SandWindowsInstaller, SandInstallerSignatureError, resolveWindowsUpdatePublisherPolicy,
  SandUpdateService, createUpdateServiceWiring, createUpdateApplyMarkerStore,
  updateCheckToTelemetry, updateStatusMessage,
} = loaded.exports;

const fixtureA = Object.freeze({ subject: "CN=Contract Fixture Alpha, O=Updater Tests, C=ZZ", certificateSha256: "a".repeat(64) });
const fixtureB = Object.freeze({ subject: "CN=Contract Fixture Beta, O=Updater Tests, C=ZZ", certificateSha256: "b".repeat(64) });
const policy = (...publishers) => JSON.stringify(publishers);
const approvedPolicy = policy(fixtureA);
const approvedFeed = "https://api.ombhrum.com/updates";
const copy = (value) => JSON.parse(JSON.stringify(value));
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const probe = (publisher = fixtureA, overrides = {}) => ({
  stdout: JSON.stringify({
    status: "Valid", signatureType: "Authenticode",
    signerSubject: publisher.subject, signerCertificateSha256: publisher.certificateSha256,
    ...overrides,
  }),
});

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
}

function harness(t) {
  const root = mkdtempSync(join(tmpdir(), "fabushi-windows-publisher-"));
  const clock = new Clock();
  const requests = [], checks = [], statuses = [], applies = [], logs = [], probes = [], spawns = [];
  const services = new Set();
  let track = null, optIn = false, quits = 0;
  let nativeProbe = async () => probe();
  let spawnAction = () => {};
  let fetchResponse = async () => new Response(null, { status: 204 });
  const runPowershell = async (args) => { probes.push([...args]); return await nativeProbe(args); };
  const spawnDetached = (command, args, onError) => {
    spawns.push({ command, args: [...args] });
    spawnAction(command, args, onError);
  };
  const installerOptions = {
    quit: () => { quits += 1; },
    log: (value) => logs.push(value),
    runPowershell,
    spawnDetached,
  };
  const createInstaller = (overrides = {}) => new SandWindowsInstaller({
    ...installerOptions, publisherPolicyJson: approvedPolicy, ...overrides,
  });
  const windows = createInstaller();
  const squirrel = Object.assign(new EventEmitter(), {
    setFeedURL() { assert.fail("unexpected Squirrel staging in Windows publisher contract"); },
    checkForUpdates() { assert.fail("unexpected Squirrel staging in Windows publisher contract"); },
    quitAndInstall() { assert.fail("unexpected Squirrel install in Windows publisher contract"); },
  });
  const settingsStore = {
    getUpdateTrackOverride: () => track,
    setUpdateTrackOverride: (value) => { track = value; },
    getAutoUpdateWhenIdleOptIn: () => optIn,
    setAutoUpdateWhenIdleOptIn: (value) => { optIn = value; },
  };
  const createService = (overrides = {}) => {
    const service = new SandUpdateService({
      currentVersion: "1.2.78", buildDefaultTrack: "stable", platform: "win32", arch: "x64",
      tempDir: root, machineId: "private-machine-fixture", feedBaseUrl: approvedFeed,
      windows, clock, now: clock.now,
      initialCheckDelayMs: 10, checkIntervalMs: 50, checkIntervalJitterMs: 0,
      getTrackOverride: settingsStore.getUpdateTrackOverride,
      persistTrackOverride: settingsStore.setUpdateTrackOverride,
      getAutoUpdateWhenIdleOptIn: settingsStore.getAutoUpdateWhenIdleOptIn,
      persistAutoUpdateWhenIdleOptIn: settingsStore.setAutoUpdateWhenIdleOptIn,
      onStatusChange: (value) => statuses.push(copy(value)),
      reportCheck: (value) => checks.push(copy(value)),
      reportApply: (value) => applies.push(copy(value)),
      log: (value) => logs.push(value),
      fetchImpl: async (url, init) => {
        requests.push({ url: String(url), init });
        return await fetchResponse(url, init);
      },
      ...overrides,
    });
    services.add(service);
    return service;
  };
  const wire = (env, extra = {}) => createUpdateServiceWiring({
    currentVersion: "1.2.78", buildDefaultTrack: "stable", machineId: "private-machine-fixture",
    platform: "win32", env,
    electron: {
      app: { getPath: () => root, quit: installerOptions.quit },
      autoUpdater: squirrel,
      powerMonitor: { getSystemIdleState: () => "active", getSystemIdleTime: () => 0 },
    },
    settingsStore, getExperimentService: () => null, getHostStatus: async () => ({ isBusy: false }),
    emitStatus: (value) => statuses.push(copy(value)),
    reportCheck: (value) => checks.push(copy(value)),
    reportApply: (value) => applies.push(copy(value)),
    log: (value) => logs.push(value),
    // Preserve the actual wiring's options. In particular there is no fixture
    // policy default here that could hide absent production configuration.
    createWindowsInstaller: (options) => new SandWindowsInstaller({ ...options, runPowershell, spawnDetached }),
    createService,
    ...extra,
  });
  const writeCandidate = (name = "candidate.exe", bytes = Buffer.from("artificial installer bytes")) => {
    const path = join(root, name);
    writeFileSync(path, bytes);
    return path;
  };
  const candidatePath = (version) => {
    for (const name of readdirSync(root)) {
      if (!name.startsWith("sand-update-")) continue;
      const path = join(root, name, "sand-update-" + version + ".exe");
      if (existsSync(path)) return path;
    }
    return null;
  };
  const offer = (version, bytes = Buffer.from("controlled package " + version)) => {
    const url = "https://api.ombhrum.com/releases/fixture-" + version + ".exe";
    fetchResponse = async (requested) => String(requested) === url
      ? new Response(bytes, { status: 200, headers: { "content-length": String(bytes.length) } })
      : Response.json({ version, url, sha256hash: digest(bytes) });
    return bytes;
  };
  t.after(() => {
    for (const service of services) service.dispose();
    rmSync(root, { recursive: true, force: true });
  });
  return {
    root, clock, requests, checks, statuses, applies, logs, probes, spawns, windows, squirrel,
    installerOptions, settingsStore, createInstaller, createService, wire, writeCandidate, candidatePath, offer,
    get quits() { return quits; },
    setProbe: (value) => { nativeProbe = value; },
    setSpawn: (value) => { spawnAction = value; },
  };
}

async function rejectsSignature(action, forbidden = []) {
  await assert.rejects(action, (error) => {
    assert.ok(error instanceof SandInstallerSignatureError);
    assert.ok(error.message.length > 0);
    for (const value of forbidden) assert.equal(error.message.includes(value), false);
    return true;
  });
}

function refusedLaunch(installer, path) {
  const errors = [];
  assert.doesNotThrow(() => installer.installOnQuit(path, { forceRun: true, onSpawnError: (error) => errors.push(error) }));
  assert.equal(errors.length, 1);
  assert.ok(errors[0] instanceof SandInstallerSignatureError);
  return errors[0];
}

test("EUS-WIN-01 absent and blank authority are distinct from every malformed policy", () => {
  for (const raw of [undefined, "", " \t\r\n"]) {
    assert.deepEqual(resolveWindowsUpdatePublisherPolicy(raw), { publishers: [], disabledReason: "windows-publisher-unconfigured" });
  }
  const invalid = [
    "not-json", "null", "[]", "{}", "true", "1", '"Anysphere"', '["Anysphere, Inc."]',
    policy(null), policy({}), policy({ subject: fixtureA.subject }),
    policy({ certificateSha256: fixtureA.certificateSha256 }),
    policy({ ...fixtureA, signerCommonName: "Anysphere" }),
    policy({ ...fixtureA, subject: "" }), policy({ ...fixtureA, subject: " " }),
    policy({ ...fixtureA, subject: " " + fixtureA.subject }),
    policy({ ...fixtureA, subject: fixtureA.subject + " " }),
    policy({ ...fixtureA, subject: "CN=private\nsubject" }),
    policy({ ...fixtureA, subject: "CN=private\u0000subject" }),
    policy({ ...fixtureA, subject: "CN=private\u007fsubject" }),
    policy({ ...fixtureA, subject: "CN=private\u0085subject" }),
    policy({ ...fixtureA, subject: "CN=private\u009fsubject" }),
    policy({ ...fixtureA, subject: 42 }),
    policy({ ...fixtureA, certificateSha256: null }),
    policy({ ...fixtureA, certificateSha256: "a".repeat(63) }),
    policy({ ...fixtureA, certificateSha256: "a".repeat(65) }),
    policy({ ...fixtureA, certificateSha256: "g".repeat(64) }),
    policy({ ...fixtureA, certificateSha256: " " + fixtureA.certificateSha256 }),
    policy(fixtureA, { ...fixtureB, certificateSha256: fixtureA.certificateSha256.toUpperCase() }),
    policy(fixtureA, { ...fixtureB, certificateSha256: "invalid" }),
    policy(...Array.from({ length: 17 }, (_, i) => ({ subject: "CN=Fixture " + i, certificateSha256: (i + 1).toString(16).padStart(64, "0") }))),
  ];
  for (const raw of invalid) {
    assert.deepEqual(resolveWindowsUpdatePublisherPolicy(raw), { publishers: [], disabledReason: "windows-publisher-invalid" }, raw);
  }
});

test("EUS-WIN-01 explicit rotation and the 16-entry bound normalize only certificate hexadecimal case", () => {
  const rotated = { subject: fixtureA.subject, certificateSha256: fixtureB.certificateSha256 };
  assert.deepEqual(resolveWindowsUpdatePublisherPolicy(" \n" + policy({ ...fixtureA, certificateSha256: fixtureA.certificateSha256.toUpperCase() }, rotated) + "\n "), {
    publishers: [fixtureA, rotated], disabledReason: null,
  });
  const sixteen = Array.from({ length: 16 }, (_, i) => ({ subject: "CN=Fixture " + i, certificateSha256: (i + 1).toString(16).padStart(64, "0") }));
  assert.deepEqual(resolveWindowsUpdatePublisherPolicy(policy(...sixteen)), { publishers: sixteen, disabledReason: null });
});

test("EUS-WIN-01/03 an installer snapshots startup authority and denies absent authority before the native probe", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  for (const [raw, reason] of [[undefined, "windows-publisher-unconfigured"], ["", "windows-publisher-unconfigured"], ["private-invalid-policy", "windows-publisher-invalid"]]) {
    const installer = h.createInstaller({ publisherPolicyJson: raw });
    assert.equal(installer.getDisabledReason(), reason);
    await rejectsSignature(() => installer.verifySignature(path), ["private-invalid-policy"]);
    refusedLaunch(installer, path);
  }
  assert.equal(h.probes.length, 0);
  assert.equal(h.spawns.length, 0);
  const mutableOptions = { ...h.installerOptions, publisherPolicyJson: approvedPolicy };
  const installer = new SandWindowsInstaller(mutableOptions);
  mutableOptions.publisherPolicyJson = policy(fixtureB);
  assert.equal(installer.getDisabledReason(), null);
  await installer.verifySignature(path);
  assert.equal(h.probes.length, 1);
});

test("EUS-WIN-02 valid native trust requires the subject and certificate pin from one authorized entry", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  const installer = h.createInstaller({ publisherPolicyJson: policy(fixtureA, fixtureB) });
  for (const publisher of [fixtureA, fixtureB]) {
    h.setProbe(async () => probe(publisher));
    await installer.verifySignature(path);
  }
  for (const mixed of [
    { subject: fixtureA.subject, certificateSha256: fixtureB.certificateSha256 },
    { subject: fixtureB.subject, certificateSha256: fixtureA.certificateSha256 },
    { subject: fixtureA.subject.toLowerCase(), certificateSha256: fixtureA.certificateSha256 },
  ]) {
    h.setProbe(async () => probe(mixed));
    await rejectsSignature(() => installer.verifySignature(path), [mixed.subject]);
    refusedLaunch(installer, path);
  }
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-02 explicit certificate rotation accepts each approved pin without learning a new pin", async (t) => {
  const h = harness(t);
  const rotated = { subject: fixtureA.subject, certificateSha256: fixtureB.certificateSha256 };
  const installer = h.createInstaller({ publisherPolicyJson: policy(fixtureA, rotated) });
  const path = h.writeCandidate();
  for (const publisher of [fixtureA, rotated]) {
    h.setProbe(async () => probe(publisher));
    await installer.verifySignature(path);
    installer.installOnQuit(path, { forceRun: false, onSpawnError: assert.fail });
  }
  assert.equal(h.spawns.length, 2);
  assert.deepEqual(h.spawns[0].args, ["--updated", "/S"]);
  h.setProbe(async () => probe({ subject: fixtureA.subject, certificateSha256: "c".repeat(64) }));
  await rejectsSignature(() => installer.verifySignature(path));
  refusedLaunch(installer, path);
  assert.equal(h.spawns.length, 2);
});

test("EUS-WIN-02 historical Anysphere common-name and certificate-only shortcuts never authorize an installer", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  for (const output of [
    { status: "Valid", signerCommonName: "Anysphere, Inc." },
    { status: "Valid", signerCommonName: "Anysphere" },
    { status: "Valid", signatureType: "Authenticode", signerSubject: "CN=Anysphere, Inc.", signerCertificateSha256: fixtureA.certificateSha256 },
    { status: "Valid", signatureType: "Authenticode", signerSubject: fixtureA.subject, signerCertificateThumbprint: fixtureA.certificateSha256 },
    { status: "Valid", signatureType: "Authenticode", signerSubject: fixtureA.subject, timestampCertificateSha256: fixtureA.certificateSha256 },
  ]) {
    h.setProbe(async () => ({ stdout: JSON.stringify(output) }));
    await rejectsSignature(() => h.windows.verifySignature(path), ["Anysphere"]);
    refusedLaunch(h.windows, path);
  }
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-02 malformed, unsigned, untrusted and catalog probe output fails with fixed non-disclosing errors", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  const privateValue = "PRIVATE_NATIVE_PROBE_VALUE";
  const outputs = [
    "", privateValue, "null", "[]", "true", "{}",
    probe(fixtureA, { status: "NotSigned" }).stdout,
    probe(fixtureA, { status: "HashMismatch" }).stdout,
    probe(fixtureA, { status: "NotTrusted" }).stdout,
    probe(fixtureA, { status: privateValue }).stdout,
    probe(fixtureA, { signatureType: "Catalog" }).stdout,
    probe(fixtureA, { signatureType: "None" }).stdout,
    probe(fixtureA, { signatureType: privateValue }).stdout,
    probe(fixtureA, { signerSubject: null }).stdout,
    probe(fixtureA, { signerSubject: privateValue }).stdout,
    probe(fixtureA, { signerCertificateSha256: privateValue }).stdout,
    probe(fixtureA, { signerCertificateSha256: 123 }).stdout,
    JSON.stringify([JSON.parse(probe().stdout)]),
    probe().stdout + "\n" + privateValue,
  ];
  for (const stdout of outputs) {
    h.setProbe(async () => ({ stdout }));
    await rejectsSignature(() => h.windows.verifySignature(path), [privateValue, fixtureA.subject]);
    refusedLaunch(h.windows, path);
  }
  assert.equal(h.spawns.length, 0);
  assert.equal(JSON.stringify(h.logs).includes(privateValue), false);
});

test("EUS-WIN-02 native timeout, native failure and file-read failure deny receipts without disclosing native output", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  const privateValue = "PRIVATE_POWERSHELL_STDERR";
  for (const code of ["ETIMEDOUT", "ENOENT", "EACCES"]) {
    h.setProbe(async () => { throw Object.assign(new Error(privateValue), { code, stderr: privateValue, stdout: privateValue }); });
    await rejectsSignature(() => h.windows.verifySignature(path), [privateValue]);
    refusedLaunch(h.windows, path);
  }
  const previousProbeCount = h.probes.length;
  await rejectsSignature(() => h.windows.verifySignature(join(h.root, "missing-private-file.exe")), ["missing-private-file"]);
  assert.equal(h.probes.length, previousProbeCount);
  h.setProbe(async () => { rmSync(path); return probe(); });
  await rejectsSignature(() => h.windows.verifySignature(path), [path]);
  refusedLaunch(h.windows, path);
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-02 the native command treats apostrophes and metacharacters as a literal file path", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate("fixture'; PRIVATE_LITERAL_PATH; '.exe");
  await h.windows.verifySignature(path);
  const args = h.probes[0];
  assert.equal(args.includes("-NoProfile"), true);
  assert.equal(args.includes("-NonInteractive"), true);
  const commandIndex = args.indexOf("-Command");
  assert.ok(commandIndex >= 0);
  assert.equal(args[commandIndex + 1].includes("[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)"), true);
  assert.equal(args[commandIndex + 1].includes("-LiteralPath '" + path.replace(/'/g, "''") + "'"), true);
});

test("EUS-WIN-03 exact-path receipts belong to one installer instance and permit only one spawn", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  const otherPath = h.writeCandidate("identical-other-file.exe");
  refusedLaunch(h.windows, path);
  await h.windows.verifySignature(path);
  refusedLaunch(h.windows, otherPath);
  refusedLaunch(h.createInstaller(), path);
  h.windows.installOnQuit(path, { forceRun: true, onSpawnError: assert.fail });
  assert.deepEqual(h.spawns, [{ command: path, args: ["--updated", "/S", "--force-run"] }]);
  refusedLaunch(h.windows, path);
  assert.equal(h.spawns.length, 1);
});

test("EUS-WIN-04 a file changed during the native probe cannot receive an install receipt", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  h.setProbe(async () => { writeFileSync(path, "different bytes during native probe"); return probe(); });
  await rejectsSignature(() => h.windows.verifySignature(path));
  refusedLaunch(h.windows, path);
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-04 failed re-verification revokes that path even when its bytes remain unchanged", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  await h.windows.verifySignature(path);
  h.setProbe(async () => probe(fixtureB));
  await rejectsSignature(() => h.windows.verifySignature(path));
  refusedLaunch(h.windows, path);
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-03/04 an older in-flight probe cannot resurrect a receipt after a newer verification fails", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  let enterOld, releaseOld, calls = 0;
  const entered = new Promise((resolve) => { enterOld = resolve; });
  const released = new Promise((resolve) => { releaseOld = resolve; });
  h.setProbe(async () => {
    calls += 1;
    if (calls === 1) { enterOld(); await released; return probe(); }
    return probe(fixtureB);
  });
  const older = h.windows.verifySignature(path);
  const olderRejection = rejectsSignature(() => older);
  await Promise.race([entered, older.then(() => assert.fail("older verification did not enter the probe"))]);
  try {
    await rejectsSignature(() => h.windows.verifySignature(path));
    refusedLaunch(h.windows, path);
  } finally {
    releaseOld();
  }
  await olderRejection;
  refusedLaunch(h.windows, path);
  assert.equal(calls, 2);
  assert.equal(h.spawns.length, 0);
});

test("EUS-WIN-03/04 an older superseded probe cannot revoke a newer successful path receipt", async (t) => {
  const h = harness(t);
  const path = h.writeCandidate();
  let enterOld, releaseOld, calls = 0;
  const entered = new Promise((resolve) => { enterOld = resolve; });
  const released = new Promise((resolve) => { releaseOld = resolve; });
  h.setProbe(async () => {
    calls += 1;
    if (calls === 1) { enterOld(); await released; }
    return probe();
  });
  const older = h.windows.verifySignature(path);
  const olderRejection = rejectsSignature(() => older);
  await Promise.race([entered, older.then(() => assert.fail("older verification did not enter the probe"))]);
  try {
    await h.windows.verifySignature(path);
  } finally {
    releaseOld();
  }
  await olderRejection;
  h.windows.installOnQuit(path, { forceRun: true, onSpawnError: assert.fail });
  assert.equal(calls, 2);
  assert.equal(h.spawns.length, 1);
  refusedLaunch(h.windows, path);
});

test("EUS-WIN-04 changed or missing bytes at launch fail through the callback and consume the old receipt", async (t) => {
  for (const change of ["replace", "remove"]) {
    const h = harness(t);
    const original = Buffer.from("verified fixture bytes");
    const path = h.writeCandidate("candidate.exe", original);
    await h.windows.verifySignature(path);
    if (change === "replace") writeFileSync(path, "unverified replacement bytes");
    else rmSync(path);
    const error = refusedLaunch(h.windows, path);
    assert.equal(error.message.includes(path), false);
    writeFileSync(path, original);
    refusedLaunch(h.windows, path);
    assert.equal(h.spawns.length, 0);
  }
});

test("EUS-WIN-03/05 spawn errors use the existing callback and cannot reuse a consumed receipt", async (t) => {
  for (const mode of ["throw", "callback"]) {
    const h = harness(t);
    const path = h.writeCandidate();
    const spawnError = Object.assign(new Error("controlled spawn failure"), { code: "EACCES" });
    const errors = [];
    h.setSpawn((_command, _args, onError) => {
      if (mode === "throw") throw spawnError;
      onError(spawnError);
    });
    await h.windows.verifySignature(path);
    h.windows.installOnQuit(path, { forceRun: true, onSpawnError: (error) => errors.push(error) });
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, "EACCES");
    refusedLaunch(h.windows, path);
    assert.equal(h.spawns.length, 1);
  }
});

test("EUS-WIN-03 service admission and all later gates cannot bypass missing or invalid publisher authority", async (t) => {
  for (const [raw, reason] of [[undefined, "windows-publisher-unconfigured"], ["[]", "windows-publisher-invalid"]]) {
    const h = harness(t);
    const service = h.createService({ windows: h.createInstaller({ publisherPolicyJson: raw }), getMinimumVersion: () => "99.0.0" });
    service.start();
    await service.checkForUpdates({ trigger: "explicit" });
    await service.setTrackOverride("dogfood");
    service.setAutoUpdateWhenIdleOptIn(true);
    service.noteMinimumVersionMayHaveChanged();
    service.noteBackendUpdateRequirement(true);
    service.noteReleaseTrackGateMayHaveChanged();
    service.applyDisabledReason(null);
    await service.checkForUpdates({ trigger: "periodic" });
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason });
    assert.equal(service.getStatus().trackOverride, "dogfood");
    assert.equal(service.getStatus().autoUpdateWhenIdleOptIn, true);
    assert.equal(service.getStatus().isBelowMinimumVersion, false);
    assert.equal(service.quitAndInstall(), false);
    service.applyStagedOnQuit();
    assert.equal(h.clock.activeCount, 0);
    assert.equal(h.requests.length, 0);
    assert.equal(h.probes.length, 0);
    assert.equal(h.spawns.length, 0);
    assert.equal(h.quits, 0);
  }
});

test("EUS-WIN-03 Windows ports without an authority method cannot silently opt into update admission", async (t) => {
  for (const windows of [null, {
    verifySignature: async () => assert.fail("unauthorized legacy port verified a candidate"),
    quit: () => assert.fail("unauthorized legacy port quit"),
    installOnQuit: () => assert.fail("unauthorized legacy port spawned an installer"),
  }]) {
    const h = harness(t);
    const service = h.createService({ windows });
    service.start();
    service.applyDisabledReason(null);
    await service.checkForUpdates({ trigger: "explicit" });
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "windows-publisher-unconfigured" });
    assert.equal(h.clock.activeCount, 0);
    assert.equal(h.requests.length, 0);
  }
});

test("EUS-WIN-03 runtime and feed gates retain precedence while non-Windows behavior remains unchanged", async (t) => {
  for (const disabledReason of ["disabled-by-env", "lab-build", "not-packaged", "unsupported-platform"]) {
    const h = harness(t);
    const service = h.createService({ disabledReason, windows: h.createInstaller({ publisherPolicyJson: undefined }) });
    service.start();
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: disabledReason });
    service.applyDisabledReason(null);
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason: "windows-publisher-unconfigured" });
  }
  for (const [feedBaseUrl, reason] of [[undefined, "update-source-unconfigured"], ["https://api2.cursor.sh/updates", "update-source-invalid"]]) {
    const h = harness(t);
    const service = h.createService({ feedBaseUrl, windows: h.createInstaller({ publisherPolicyJson: "[]" }) });
    service.start();
    service.applyDisabledReason(null);
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason });
    assert.equal(h.requests.length, 0);
  }
  const h = harness(t);
  const darwin = h.createService({ platform: "darwin", windows: h.createInstaller({ publisherPolicyJson: "[]" }), squirrel: h.squirrel });
  darwin.start();
  await darwin.checkForUpdates({ trigger: "explicit" });
  assert.equal(darwin.getStatus().state.type, "idle");
  assert.equal(h.requests.length, 1);
  assert.equal(h.clock.activeCount, 2);
});

test("EUS-WIN-01/03 actual production wiring reads only the explicit publisher variable and preserves its raw value", async (t) => {
  const raw = " \n" + policy({ ...fixtureA, certificateSha256: fixtureA.certificateSha256.toUpperCase() }) + "\n ";
  for (const [env, reason] of [
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed }, "windows-publisher-unconfigured"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, SAND_WINDOWS_UPDATE_PUBLISHERS_JSON: approvedPolicy, WINDOWS_INSTALLER_SIGNER_ALLOWLIST: "Anysphere" }, "windows-publisher-unconfigured"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON: "", SAND_WINDOWS_UPDATE_PUBLISHERS_JSON: approvedPolicy }, "windows-publisher-unconfigured"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON: "[]", SAND_WINDOWS_UPDATE_PUBLISHERS_JSON: approvedPolicy }, "windows-publisher-invalid"],
    [{ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON: policy({ ...fixtureA, subject: "CN=private\u0085subject" }) }, "windows-publisher-invalid"],
  ]) {
    const h = harness(t);
    const service = h.wire(env);
    await service.checkForUpdates({ trigger: "explicit" });
    assert.deepEqual(service.getStatus().state, { type: "disabled", reason });
    assert.equal(h.requests.length, 0);
  }
  const h = harness(t);
  let receivedOptions;
  const service = h.wire({ FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed, FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON: raw }, {
    createWindowsInstaller: (options) => {
      receivedOptions = options;
      return new SandWindowsInstaller({ ...options, runPowershell: h.installerOptions.runPowershell, spawnDetached: h.installerOptions.spawnDetached });
    },
  });
  assert.equal(receivedOptions.publisherPolicyJson, raw);
  assert.equal(Object.hasOwn(receivedOptions, "signerAllowlist"), false);
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  assert.equal(service.getStatus().state.type, "idle");
});

test("EUS-WIN-01/03 changing startup environment requires reconstruction and preserves user Settings", async (t) => {
  const h = harness(t);
  const env = { FABUSHI_UPDATE_FEED_BASE_URL: approvedFeed };
  const blocked = h.wire(env);
  await blocked.setTrackOverride("dogfood");
  blocked.setAutoUpdateWhenIdleOptIn(true);
  env.FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON = approvedPolicy;
  blocked.applyDisabledReason(null);
  await blocked.checkForUpdates({ trigger: "explicit" });
  assert.deepEqual(blocked.getStatus().state, { type: "disabled", reason: "windows-publisher-unconfigured" });
  assert.equal(h.requests.length, 0);
  blocked.dispose();
  const recovered = h.wire(env);
  env.FABUSHI_WINDOWS_UPDATE_PUBLISHERS_JSON = "private-invalid-replacement";
  await recovered.checkForUpdates({ trigger: "explicit" });
  assert.equal(recovered.getStatus().state.type, "idle");
  assert.equal(recovered.getStatus().trackOverride, "dogfood");
  assert.equal(recovered.getStatus().autoUpdateWhenIdleOptIn, true);
  assert.equal(h.requests.length, 1);
  recovered.dispose();
  recovered.start();
  recovered.applyDisabledReason(null);
  await recovered.checkForUpdates({ trigger: "explicit" });
  assert.equal(h.requests.length, 1);
  assert.equal(h.clock.activeCount, 0);
});

test("EUS-WIN-05 publisher gating reaches actual Settings and distinct non-retryable telemetry without raw policy", (t) => {
  for (const [raw, reason, code, expectedText] of [
    [undefined, "windows-publisher-unconfigured", "SAND-E0515", "Windows updates are unavailable until a publisher is authorized"],
    ["PRIVATE_INVALID_POLICY", "windows-publisher-invalid", "SAND-E0516", "The Windows update publisher configuration is invalid"],
  ]) {
    const h = harness(t);
    const service = h.createService({ windows: h.createInstaller({ publisherPolicyJson: raw }) });
    service.start();
    assert.deepEqual(updateStatusMessage(service.getStatus()), { text: expectedText, tone: "default" });
    const telemetry = updateCheckToTelemetry(h.checks[0]);
    assert.equal(telemetry.metadata.error_code, code);
    assert.equal(telemetry.metadata.error_retryable, "false");
    assert.equal(telemetry.metadata.cause, reason);
    assert.equal(JSON.stringify([service.getStatus(), h.checks, h.statuses, h.logs, telemetry]).includes("PRIVATE_INVALID_POLICY"), false);
  }
});

test("EUS-WIN-03/05 disabled restart settles persisted apply markers before the publisher gate without touching sessions", (t) => {
  for (const [raw, phase, currentVersion, expectedOutcomes] of [
    [undefined, "staged", "1.2.78", []],
    [undefined, "requested", "1.2.79", ["confirmed"]],
    ["[]", "spawned", "1.2.78", ["spawned", "stale_version"]],
    ["[]", "spawn_failed", "1.2.78", ["stale_version"]],
  ]) {
    const h = harness(t);
    const markerPath = join(h.root, "sand-update-apply-marker.json");
    const sessionPath = join(h.root, "existing-session.json");
    const session = Buffer.from('{"session":"existing-user-data","version":7}\n');
    writeFileSync(sessionPath, session);
    const previous = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    const markerInput = { mechanism: "nsis", targetVersion: "1.2.79", fromVersion: "1.2.78" };
    if (phase === "staged") previous.recordStaged(markerInput);
    else previous.recordRequested(markerInput);
    if (phase === "spawned") previous.recordSpawned();
    if (phase === "spawn_failed") previous.recordSpawnFailed(Object.assign(new Error("fixture"), { code: "EACCES" }));
    assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).version, 1);
    assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).phase, phase);
    const marker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    const sequence = [];
    const service = h.createService({
      currentVersion, applyMarker: marker, windows: h.createInstaller({ publisherPolicyJson: raw }),
      reportApply: (value) => { sequence.push("apply"); h.applies.push(copy(value)); },
      reportCheck: (value) => { sequence.push("check"); h.checks.push(copy(value)); },
    });
    service.start();
    assert.deepEqual(h.applies.map((report) => report.outcome), expectedOutcomes);
    assert.deepEqual(sequence, [...expectedOutcomes.map(() => "apply"), "check"]);
    assert.equal(marker.peek(), null);
    assert.equal(existsSync(markerPath), false);
    assert.deepEqual(readFileSync(sessionPath), session);
    assert.equal(service.getStatus().state.type, "disabled");
    assert.equal(h.requests.length, 0);
    assert.equal(h.probes.length, 0);
    assert.equal(h.spawns.length, 0);
    assert.equal(h.clock.activeCount, 0);
  }
});

test("EUS-WIN-04/05 real installer authorization preserves service staging, one-use launch and marker ownership", async (t) => {
  const h = harness(t);
  const bytes = h.offer("1.2.79");
  const markerPath = join(h.root, "sand-update-apply-marker.json");
  const marker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
  const service = h.createService({ applyMarker: marker });
  service.start();
  await service.checkForUpdates({ trigger: "explicit" });
  assert.deepEqual(service.getStatus().state, { type: "ready", version: "1.2.79" });
  const path = h.candidatePath("1.2.79");
  assert.ok(path);
  assert.deepEqual(readFileSync(path), bytes);
  assert.equal(h.probes.length, 1);
  assert.equal(marker.peek().phase, "staged");
  assert.equal(service.quitAndInstall(), true);
  assert.equal(h.quits, 1);
  assert.equal(marker.peek().phase, "requested");
  service.applyStagedOnQuit();
  service.applyStagedOnQuit();
  assert.deepEqual(h.spawns, [{ command: path, args: ["--updated", "/S", "--force-run"] }]);
  assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).phase, "spawned");
  refusedLaunch(h.windows, path);
  assert.equal(h.spawns.length, 1);
});

test("EUS-WIN-02/05 a rejected downloaded publisher cannot become ready or quit and cleanup preserves user data", async (t) => {
  const h = harness(t);
  h.offer("1.2.79");
  h.setProbe(async () => probe({ subject: "CN=Anysphere, Inc.", certificateSha256: fixtureA.certificateSha256 }));
  const marker = createUpdateApplyMarkerStore(join(h.root, "sand-update-apply-marker.json"), h.clock.now);
  const sessionPath = join(h.root, "existing-session.json");
  const session = Buffer.from('{"existing":"session-data"}\n');
  writeFileSync(sessionPath, session);
  const service = h.createService({ applyMarker: marker });
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(service.getStatus().state.type, "idle");
  assert.equal(service.getStatus().state.lastCheck.result, "error");
  assert.equal(service.getStatus().state.lastCheck.errorMessage.includes("Anysphere"), false);
  assert.equal(h.candidatePath("1.2.79"), null);
  assert.equal(marker.peek(), null);
  assert.equal(service.quitAndInstall(), false);
  service.applyStagedOnQuit();
  assert.equal(h.quits, 0);
  assert.equal(h.spawns.length, 0);
  assert.deepEqual(readFileSync(sessionPath), session);
});

test("EUS-WIN-04/05 a newer candidate failing authorization preserves the previously ready candidate receipt", async (t) => {
  const h = harness(t);
  const marker = createUpdateApplyMarkerStore(join(h.root, "sand-update-apply-marker.json"), h.clock.now);
  const service = h.createService({ applyMarker: marker });
  h.offer("1.2.79");
  await service.checkForUpdates({ trigger: "explicit" });
  const priorPath = h.candidatePath("1.2.79");
  assert.ok(priorPath);
  h.offer("1.2.80");
  h.setProbe(async () => probe(fixtureB));
  await service.checkForUpdates({ trigger: "explicit" });
  assert.equal(service.getStatus().state.type, "ready");
  assert.equal(service.getStatus().state.version, "1.2.79");
  assert.equal(service.getStatus().state.lastCheck.result, "error");
  assert.equal(marker.peek().targetVersion, "1.2.79");
  assert.equal(h.candidatePath("1.2.80"), null);
  assert.equal(existsSync(priorPath), true);
  assert.equal(service.quitAndInstall(), true);
  service.applyStagedOnQuit();
  assert.equal(h.spawns.length, 1);
  assert.equal(h.spawns[0].command, priorPath);
  assert.equal(marker.peek().phase, "spawned");
});

test("EUS-WIN-04/05 cancelling a newly verified track candidate preserves the previous candidate receipt", async (t) => {
  const h = harness(t);
  const marker = createUpdateApplyMarkerStore(join(h.root, "sand-update-apply-marker.json"), h.clock.now);
  const service = h.createService({ applyMarker: marker });
  h.offer("1.2.79");
  await service.checkForUpdates({ trigger: "explicit" });
  const priorPath = h.candidatePath("1.2.79");
  assert.ok(priorPath);
  h.offer("1.2.80");
  let enterProbe, releaseProbe;
  const entered = new Promise((resolve) => { enterProbe = resolve; });
  const released = new Promise((resolve) => { releaseProbe = resolve; });
  h.setProbe(async () => { enterProbe(); await released; return probe(); });
  const checking = service.checkForUpdates({ trigger: "explicit" });
  await Promise.race([entered, checking.then(() => assert.fail("candidate did not reach the native probe"))]);
  try {
    assert.equal(service.getStatus().state.type, "staging");
    await service.setTrackOverride("dogfood");
  } finally {
    releaseProbe();
  }
  await checking;
  assert.deepEqual(service.getStatus().state, { type: "ready", version: "1.2.79" });
  assert.equal(h.candidatePath("1.2.80"), null);
  assert.equal(marker.peek().targetVersion, "1.2.79");
  assert.equal(service.quitAndInstall(), true);
  service.applyStagedOnQuit();
  assert.equal(h.spawns.length, 1);
  assert.equal(h.spawns[0].command, priorPath);
});

test("EUS-WIN-04/05 rejected launch bytes and spawn failures retain spawn_failed restart evidence", async (t) => {
  for (const failure of ["replaced-bytes", "spawn-error"]) {
    const h = harness(t);
    const markerPath = join(h.root, "sand-update-apply-marker.json");
    const sessionPath = join(h.root, "existing-session.json");
    const session = Buffer.from('{"preserve":"user-session"}\n');
    writeFileSync(sessionPath, session);
    const marker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    const service = h.createService({ applyMarker: marker });
    h.offer("1.2.79");
    await service.checkForUpdates({ trigger: "explicit" });
    const path = h.candidatePath("1.2.79");
    assert.ok(path);
    if (failure === "replaced-bytes") writeFileSync(path, "unsigned substituted bytes");
    else h.setSpawn(() => { throw Object.assign(new Error("controlled spawn failure"), { code: "EACCES" }); });
    assert.equal(service.quitAndInstall(), true);
    service.applyStagedOnQuit();
    assert.equal(marker.peek().phase, "spawn_failed");
    assert.equal(JSON.parse(readFileSync(markerPath, "utf8")).phase, "spawn_failed");
    assert.equal(h.spawns.length, failure === "replaced-bytes" ? 0 : 1);
    service.dispose();
    h.applies.length = 0;
    const restartedMarker = createUpdateApplyMarkerStore(markerPath, h.clock.now);
    const restarted = h.createService({
      windows: h.createInstaller({ publisherPolicyJson: undefined }), applyMarker: restartedMarker,
    });
    restarted.start();
    assert.equal(h.applies.length, 1);
    assert.equal(h.applies[0].outcome, "stale_version");
    assert.equal(h.applies[0].error.code, "SAND-E0511");
    assert.equal(h.applies[0].error.errno, failure === "replaced-bytes" ? "E_OTHER" : "EACCES");
    assert.equal(existsSync(markerPath), false);
    assert.deepEqual(readFileSync(sessionPath), session);
  }
});
