import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, realpath, rename, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { createDandelionCliBackend } from "./miniapp-cli-backend.js";
import { registerProductionMiniApps } from "./miniapp-production.js";
import { createMiniAppToolRouter, MINIAPP_TOOL_CONTRACT, type MiniAppToolBackend, type MiniAppToolScope } from "./miniapp-tools.js";
import type { MiniAppIpcEvent, MiniAppSession, MiniAppWindow, MiniAppWindowOptions } from "./miniapp-window.js";
import { MINIAPP_TOOL_CHANNEL } from "../../shared/miniapp-tool-contract.js";
import type { ProductionServiceContext } from "../main-production-services.js";

const scope: MiniAppToolScope = {
  appId: "org.fabushi.global-dandelion", version: "0.2.0",
  packageDigest: "a".repeat(64), accountScope: "b".repeat(64), accountGeneration: 1,
};

function fixture() {
  let sent = 0;
  const backend: MiniAppToolBackend = {
    capabilities: async () => ({ capabilities: ["offline", "selection", "transfer"], platform: "desktop", targetPools: [{ id: "lan", label: "LAN" }] }),
    readState: async () => ({ selections: 0 }),
    pick: async () => ({ selectionId: "sel-1", entries: [{ name: "root", size: 0 }] }),
    release: async () => {},
    send: async () => { sent += 1; return { operationId: "op-1" }; },
    status: async () => ({ state: "running" }),
    cancel: async () => {},
    scan: async () => ({}),
    hotspot: async () => ({}),
    update: async () => ({}),
  };
  return { backend, sent: () => sent };
}

test("capabilities answer with the declared contract", async () => {
  const { backend } = fixture();
  const result = await createMiniAppToolRouter({ backend }).invoke(scope, "offline.capabilities", {});
  assert.equal(result.ok, true);
  assert.equal(result.ok && (result.data as { contract: string }).contract, MINIAPP_TOOL_CONTRACT);
});

test("undeclared tools and malformed arguments fail closed", async () => {
  const { backend } = fixture();
  const router = createMiniAppToolRouter({ backend });
  assert.equal((await router.invoke(scope, "shell.exec", {})).ok, false);
  assert.equal((await router.invoke(scope, "scan.metadata", "not-an-object")).ok, false);
  assert.equal((await router.invoke(scope, "scan.metadata", { rootId: "../../etc" })).ok, false);
});

test("a mutating tool never runs without an approval owner", async () => {
  const { backend, sent } = fixture();
  const refused = createMiniAppToolRouter({ backend, approve: () => false });
  const ownerless = createMiniAppToolRouter({ backend });
  const args = { selectionId: "sel-1", targetPoolId: "lan" };
  assert.equal((await refused.invoke(scope, "transfer.send", args)).ok, false);
  assert.equal((await ownerless.invoke(scope, "transfer.send", args)).ok, false);
  assert.equal(sent(), 0);
  const approved = createMiniAppToolRouter({ backend, approve: () => true });
  assert.equal((await approved.invoke(scope, "transfer.send", args)).ok, true);
  assert.equal(sent(), 1);
});

test("a foreign app cannot inspect native state and a late approval cannot start work", async () => {
  const { backend, sent } = fixture();
  let current = true;
  let approve!: (allowed: boolean) => void;
  const router = createMiniAppToolRouter({
    backend, allowedAppId: scope.appId,
    approve: () => new Promise<boolean>((settle) => { approve = settle; }),
  });
  const foreign = await router.invoke({ ...scope, appId: "org.other.app" }, "offline.state.read", {});
  assert.equal(foreign.ok, false);
  assert.equal(!foreign.ok && foreign.error.code, "APP_NOT_AUTHORIZED");
  const pending = router.invoke(scope, "transfer.send", { selectionId: "sel-1", targetPoolId: "lan" }, () => current);
  while (!approve) await new Promise((resolve) => setImmediate(resolve));
  current = false;
  approve(true);
  const result = await pending;
  assert.equal(result.ok, false);
  assert.equal(!result.ok && result.error.code, "STALE_SCOPE");
  assert.equal(sent(), 0);
});

test("calls are throttled per account scope and recover after the window", async () => {
  const { backend } = fixture();
  let now = 0;
  const router = createMiniAppToolRouter({ backend, now: () => now });
  let last = null;
  for (let index = 0; index < 45; index += 1) last = await router.invoke(scope, "offline.state.read", {});
  assert.equal(last?.ok, false);
  now = 10_000;
  assert.equal((await router.invoke(scope, "offline.state.read", {})).ok, true);
});

test("owned operation control remains available when a native probe cannot obtain a slot", async () => {
  const { backend } = fixture();
  backend.capabilities = async () => { throw new Error("NATIVE_OPERATION_LIMIT"); };
  let cancelled = 0;
  backend.cancel = async (id) => { assert.equal(id, "op-1"); cancelled += 1; };
  const router = createMiniAppToolRouter({ backend, allowedAppId: scope.appId, approve: () => true });
  assert.equal((await router.invoke(scope, "transfer.status", { operationId: "op-1" })).ok, true);
  assert.equal((await router.invoke(scope, "transfer.cancel", { operationId: "op-1" })).ok, true);
  assert.equal(cancelled, 1);
  assert.equal((await router.invoke(scope, "transfer.send", { selectionId: "sel-1", targetPoolId: "lan" })).ok, false);
});

test("CLI backend enforces directory-only selection, canonical root boundaries and handle release", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "fabushi-dandelion-cli-"));
  try {
    const allowedRoot = join(workspace, "authorized");
    const subDir = join(allowedRoot, "dataset");
    const outsideDir = join(workspace, "outside");
    const escapedLink = join(allowedRoot, "escape-link");
    await mkdir(subDir, { recursive: true });
    await mkdir(outsideDir, { recursive: true });
    await symlink(outsideDir, escapedLink);

    let nextPick: string | null = subDir;
    const spawned: string[][] = [];
    const backend = createDandelionCliBackend(
      {
        platform: "linux",
        binaryPath: "/nonexistent/dandelion",
        configPath: "/nonexistent/config.toml",
        allowedRoots: [allowedRoot],
        targetPools: [{ id: "lan", label: "LAN" }],
      },
      {
        pick: async () => (nextPick ? [{ path: nextPick, name: "picked", size: 2048 }] : null),
        spawn: (argv) => {
          spawned.push([...argv]);
          return {
            done: Promise.resolve({ code: 0, stdout: JSON.stringify({ files: 2, bytes: 2048 }), stderr: "" }),
            kill: () => {},
          };
        },
      },
    );

    await assert.rejects(() => backend.pick("files", { maxFiles: 1, maxBytes: 1024 }), /DIRECTORY_SELECTION_REQUIRED/);

    nextPick = outsideDir;
    await assert.rejects(() => backend.pick("directory", { maxFiles: 1, maxBytes: 1024 }), /ROOT_NOT_AUTHORIZED/);

    nextPick = escapedLink;
    await assert.rejects(() => backend.pick("directory", { maxFiles: 1, maxBytes: 1024 }), /ROOT_NOT_AUTHORIZED/);

    nextPick = subDir;
    const picked = await backend.pick("directory", { maxFiles: 1, maxBytes: 1024 });
    assert.ok(picked);
    assert.equal(picked.entries[0]?.name, "dataset");
    const canonicalSubDir = await realpath(subDir);
    const scanned = await backend.scan("metadata", picked.selectionId);
    assert.deepEqual(scanned, { files: 2, bytes: 2048 });
    assert.deepEqual(spawned[0], ["scan", "metadata", "--source", canonicalSubDir, "--json"]);

    await backend.release(picked.selectionId);
    await assert.rejects(() => backend.scan("metadata", picked.selectionId), /UNKNOWN_SELECTION/);
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("CLI backend spawns real argv-only binary for scan, async transfer, cancellation, hotspot and update", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "fabushi-dandelion-bin-"));
  try {
    const allowedRoot = join(workspace, "root");
    await mkdir(allowedRoot, { recursive: true });
    const configPath = join(workspace, "dandelion.toml");
    await writeFile(configPath, "# test config\n", "utf8");
    const binPath = join(workspace, "dandelion-mock.py");
    await writeFile(
      binPath,
      [
        "#!/usr/bin/env python3",
        "import json, sys, time",
        "args = sys.argv[1:]",
        "assert args[:2] == ['--config', " + JSON.stringify(configPath) + "], f'unexpected config prefix: {args}'",
        "cmd = args[2:]",
        "if cmd[:2] == ['scan', 'metadata']:",
        "    print(json.dumps({'mode': 'metadata', 'files': 3, 'bytes': 4096, 'source': cmd[3]}))",
        "elif cmd[:1] == ['run']:",
        "    if 'slow' in cmd[2]:",
        "        time.sleep(10)",
        "    print(json.dumps({'packets': 8, 'verifiedBytes': 4096, 'source': cmd[2]}))",
        "elif cmd[:2] == ['hotspot', 'plan']:",
        "    print(json.dumps({'profileId': 'hs-preview-1'}))",
        "elif cmd[:2] == ['hotspot', 'apply']:",
        "    print(json.dumps({'applied': True}))",
        "elif cmd[:2] == ['hotspot', 'status']:",
        "    print(json.dumps({'active': False}))",
        "elif cmd[:2] == ['hotspot', 'stop']:",
        "    print(json.dumps({'stopped': True}))",
        "elif cmd[:2] == ['update', 'check']:",
        "    print(json.dumps({'upToDate': True}))",
        "elif cmd[:2] == ['config', 'init']:",
        "    print(json.dumps({'defaults': True}))",
        "else:",
        "    sys.stderr.write('unknown args: ' + ' '.join(cmd))",
        "    sys.exit(2)",
      ].join("\n"),
      "utf8",
    );
    await chmod(binPath, 0o755);

    const fastDir = join(allowedRoot, "fast");
    const slowDir = join(allowedRoot, "slow");
    await mkdir(fastDir, { recursive: true });
    await mkdir(slowDir, { recursive: true });

    let chosenDir = fastDir;
    const disabledBackend = createDandelionCliBackend(
      { platform: "linux", binaryPath: binPath, configPath, allowedRoots: [allowedRoot], targetPools: [{ id: "lan", label: "LAN" }] },
      { pick: async () => [{ path: chosenDir, name: "dir", size: 4096 }] },
    );
    await assert.rejects(() => disabledBackend.hotspot("plan"), /HOTSPOT_DISABLED/);
    await assert.rejects(() => disabledBackend.update("check"), /UPDATE_DISABLED/);

    const backend = createDandelionCliBackend(
      {
        platform: "linux",
        binaryPath: binPath,
        configPath,
        allowedRoots: [allowedRoot],
        targetPools: [{ id: "lan", label: "LAN" }],
        enableHotspot: true,
        hotspotJournalDirectory: join(workspace, "hotspot-journal"),
        hotspotInterface: "wlan0",
        hotspotPasswordFile: join(workspace, "hotspot-password"),
        updateTrust: {
          manifest: join(workspace, "release.json"),
          signature: join(workspace, "release.sig"),
          publicKey: join(workspace, "trusted-key.raw"),
        },
        enableUpdate: true,
      },
      { pick: async () => [{ path: chosenDir, name: "dir", size: 4096 }] },
    );

    const fastPicked = await backend.pick("directory", { maxFiles: 1, maxBytes: 1024 ** 3 });
    assert.ok(fastPicked);
    const scanOut = (await backend.scan("metadata", fastPicked.selectionId)) as { files: number; bytes: number };
    assert.equal(scanOut.files, 3);
    assert.equal(scanOut.bytes, 4096);

    await assert.rejects(() => backend.send({ selectionId: fastPicked.selectionId, targetPoolId: "unknown-pool" }), /UNKNOWN_TARGET_POOL/);
    const started = await backend.send({ selectionId: fastPicked.selectionId, targetPoolId: "lan" });
    let finalState: { state?: string; result?: unknown } = { state: "running" };
    for (let attempt = 0; attempt < 40; attempt += 1) {
      finalState = (await backend.status(started.operationId)) as { state?: string; result?: unknown };
      if (finalState.state !== "running") break;
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
    assert.equal(finalState.state, "complete");
    assert.equal((finalState.result as { packets: number }).packets, 8);

    chosenDir = slowDir;
    const slowPicked = await backend.pick("directory", { maxFiles: 1, maxBytes: 1024 ** 3 });
    assert.ok(slowPicked);
    const slowOp = await backend.send({ selectionId: slowPicked.selectionId, targetPoolId: "lan" });
    await backend.cancel(slowOp.operationId);
    const cancelledState = (await backend.status(slowOp.operationId)) as { state?: string };
    assert.equal(cancelledState.state, "cancelled");

    assert.deepEqual(await backend.hotspot("plan"), { profileId: "hs-preview-1" });
    assert.deepEqual(await backend.hotspot("apply"), { applied: true });
    assert.deepEqual(await backend.hotspot("status"), { active: false });
    assert.deepEqual(await backend.update("check"), { upToDate: true });
    assert.deepEqual(await backend.config("defaults"), { defaults: true });
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("an unproven Apple binary never advertises native capabilities", async () => {
  const backend = createDandelionCliBackend(
    {
      binaryPath: "/reviewed/dandelion",
      configPath: "/reviewed/dandelion.toml",
      allowedRoots: ["/reviewed/root"],
      targetPools: [{ id: "lan", label: "LAN" }],
      platform: "darwin",
    },
    { pick: async () => null },
  );
  const announced = await backend.capabilities();
  assert.deepEqual(announced.capabilities, ["offline", "selection"]);
  assert.equal(announced.platform, "desktop");
});

test("Apple native contract is probed again and Linux-only operations stay unavailable", async () => {
  let valid = true;
  let probes = 0;
  const backend = createDandelionCliBackend({
    binaryPath: "/reviewed/dandelion", configPath: "/reviewed/config.toml",
    allowedRoots: ["/reviewed/root"], targetPools: [], platform: "darwin", enableHotspot: true,
  }, { pick: async () => null, spawn: (argv) => {
    if (argv[0] === "doctor") probes += 1;
    return { kill() {}, done: Promise.resolve({ code: 0, stderr: "", stdout: JSON.stringify(
      argv[0] === "doctor" ? { native_contract: valid ? "dandelion.native.apple.v1" : "legacy", os: "macos", target: "aarch64-apple-darwin" } : { defaults: true },
    ) }) };
  } });
  assert.deepEqual((await backend.capabilities()).capabilities, ["offline", "selection", "scan", "config", "diagnostics"]);
  assert.deepEqual(await backend.config("defaults"), { defaults: true });
  await assert.rejects(() => backend.platform("mounts"), /PLATFORM_UNSUPPORTED/);
  await assert.rejects(() => backend.hotspot("plan"), /PLATFORM_UNSUPPORTED/);
  valid = false;
  await assert.rejects(() => backend.config("defaults"), /NATIVE_RUNTIME_INCOMPATIBLE/);
  assert.ok(probes >= 3);
});

test("receive consumes readiness and final receipt; readiness alone and malformed trailers fail", async () => {
  let stdout = "";
  const backend = createDandelionCliBackend({
    binaryPath: "/reviewed/dandelion", configPath: "/reviewed/config.toml", allowedRoots: [], targetPools: [], platform: "linux",
    receive: { targetPoolId: "local", caFile: "/private/ca", certificateFile: "/private/cert", privateKeyFile: "/private/key" },
  }, { pick: async () => null, spawn: () => ({ kill() {}, done: Promise.resolve({ code: 0, stderr: "", stdout }) }) });
  const ready = { event: "ready", mutual_tls: true, address: "127.0.0.1:10000" };
  const receipt = { sessions: 1, failed_sessions: 0, verified_payload_bytes: 42, cancelled: false };
  for (const [payload, expected] of [
    [JSON.stringify(ready) + "\n" + JSON.stringify(receipt) + "\n", "complete"],
    [JSON.stringify(ready) + "\n", "failed"],
    [JSON.stringify(ready) + "\n" + JSON.stringify(receipt) + "\ninvalid\n", "failed"],
  ] as const) {
    stdout = payload;
    const operation = await backend.receive({ targetPoolId: "local" });
    await new Promise((resolve) => setTimeout(resolve, 0));
    const status = await backend.status(operation.operationId) as { state: string; result?: unknown };
    assert.equal(status.state, expected);
    if (expected === "complete") assert.deepEqual(status.result, receipt);
  }
});

test("all native calls share bounded admission and teardown fences direct results", async () => {
  const pending: Array<{ resolve: (result: { code: number; stdout: string; stderr: string }) => void; killed: boolean }> = [];
  const backend = createDandelionCliBackend({
    binaryPath: "/reviewed/dandelion", configPath: "/reviewed/config.toml", allowedRoots: [], targetPools: [], platform: "linux",
  }, { pick: async () => null, spawn: () => {
    let resolve!: (result: { code: number; stdout: string; stderr: string }) => void;
    const done = new Promise<{ code: number; stdout: string; stderr: string }>((settle) => { resolve = settle; });
    const entry = { resolve, killed: false };
    pending.push(entry);
    return { done, kill() { entry.killed = true; } };
  } });
  const requests = Array.from({ length: 8 }, () => backend.config("defaults").then(
    () => "unexpected success", (error: Error) => error.message,
  ));
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(pending.length, 8);
  await assert.rejects(() => backend.config("defaults"), /NATIVE_OPERATION_LIMIT/);
  backend.cancelAll();
  assert.ok(pending.every((entry) => entry.killed));
  await assert.rejects(() => backend.config("defaults"), /NATIVE_OPERATION_LIMIT/);
  for (const entry of pending) entry.resolve({ code: 0, stdout: "{}", stderr: "" });
  assert.deepEqual(await Promise.all(requests), Array(8).fill("DANDELION_CANCELLED"));
});

test("native failures return stable errors without paths or credential stderr", async () => {
  const backend = createDandelionCliBackend({
    binaryPath: "/reviewed/dandelion", configPath: "/reviewed/config.toml", allowedRoots: [], targetPools: [], platform: "linux",
  }, { pick: async () => null, spawn: () => ({ kill() {}, done: Promise.resolve({ code: 2, stdout: "", stderr: "/private/key SECRET" }) }) });
  await assert.rejects(() => backend.config("defaults"), /^Error: DANDELION_EXIT_2$/);
});

test("bundled picker grants only the selected directory and rejects replacement before invocation", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "fabushi-picked-root-"));
  try {
    const root = join(workspace, "selected");
    await mkdir(root);
    let spawned = 0;
    const backend = createDandelionCliBackend({
      binaryPath: "/bundle/dandelion", configPath: "/bundle/default.toml", allowedRoots: [],
      authorizePickedRoots: true, targetPools: [{ id: "loopback", label: "Local" }], platform: "linux",
    }, { pick: async () => [{ path: root, name: "selected", size: 0 }], spawn: () => {
      spawned += 1;
      return { kill() {}, done: Promise.resolve({ code: 0, stdout: "{}", stderr: "" }) };
    } });
    const picked = await backend.pick("directory", { maxFiles: 1, maxBytes: 1024 });
    assert.ok(picked);
    await backend.scan("metadata", picked.selectionId);
    assert.equal(spawned, 1);
    await rename(root, join(workspace, "original"));
    await mkdir(root);
    await assert.rejects(() => backend.scan("metadata", picked.selectionId), /SELECTION_SOURCE_CHANGED/);
    assert.equal(spawned, 1);
  } finally { await rm(workspace, { recursive: true, force: true }); }
});

test("Host invokes the pinned Global Dandelion source engine through the real CLI", {
  skip: !process.env.FABUSHI_DANDELION_REAL_BIN,
  timeout: 120_000,
}, async () => {
  const binaryPath = process.env.FABUSHI_DANDELION_REAL_BIN!;
  const configPath = process.env.FABUSHI_DANDELION_REAL_CONFIG!;
  const root = process.env.FABUSHI_DANDELION_REAL_ROOT!;
  const stateDirectory = process.env.FABUSHI_DANDELION_REAL_STATE!;
  const importFile = process.env.FABUSHI_DANDELION_REAL_IMPORT!;
  const backend = createDandelionCliBackend(
    {
      binaryPath,
      configPath,
      allowedRoots: [root],
      stateDirectory,
      targetPools: [{ id: "loopback", label: "授权本机回环" }],
      platform: process.platform,
    },
    { pick: async (mode) => [{ path: mode === "files" ? importFile : root, name: "real-fixture", size: 32 }] },
  );

  const capabilities = await backend.capabilities();
  assert.deepEqual(capabilities.capabilities, [
    "offline", "selection", "transfer", "scan", "state", "config",
    ...(process.platform === "linux" ? ["platform"] : []), "diagnostics",
  ]);

  const picked = await backend.pick("directory", { maxFiles: 1, maxBytes: 1024 ** 3 });
  assert.ok(picked);

  const metadata = await backend.scan("metadata", picked.selectionId) as Record<string, unknown>;
  assert.equal(metadata.payload_bytes_read, 0);
  assert.ok(Number(metadata.regular_files) >= 1);

  const read = await backend.scan("read", picked.selectionId) as Record<string, unknown>;
  assert.ok(Number(read.verified_payload_bytes) > 0);

  const defaults = await backend.config("defaults") as Record<string, unknown>;
  assert.equal(typeof defaults, "object");
  const checked = await backend.config("check") as Record<string, unknown>;
  assert.equal(typeof checked, "object");

  if (process.platform === "linux") {
    const platform = await backend.platform("inspect") as Record<string, unknown>;
    assert.ok(Array.isArray(platform.mounts));
  } else {
    await assert.rejects(() => backend.platform("inspect"), /PLATFORM_UNSUPPORTED/);
  }
  const diagnostics = await backend.diagnostics("selfTest") as Record<string, unknown>;
  assert.equal(diagnostics.status, "completed");

  const operation = await backend.send({ selectionId: picked.selectionId, targetPoolId: "loopback" });
  let status: Record<string, unknown> = { state: "running" };
  for (let attempt = 0; attempt < 240; attempt += 1) {
    status = await backend.status(operation.operationId) as Record<string, unknown>;
    if (status.state !== "running") break;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.equal(status.state, "complete");
  assert.ok(Number((status.result as Record<string, unknown>).verified_payload_bytes) > 0);

  const imported = await backend.state("import", {}) as Record<string, unknown>;
  assert.equal(imported.imported, true);
  const persisted = await backend.state("inspect", {}) as Record<string, unknown>;
  assert.equal(persisted.charged_bytes, 1234);
  const repeated = await backend.state("import", {}) as Record<string, unknown>;
  assert.equal(repeated.imported, false);
  const watched = await backend.send({ selectionId: picked.selectionId, targetPoolId: "loopback", watch: true });
  await backend.cancel(watched.operationId);
  assert.equal((await backend.status(watched.operationId) as Record<string, unknown>).state, "cancelled");

  await backend.release(picked.selectionId);
  await assert.rejects(() => backend.scan("metadata", picked.selectionId), /UNKNOWN_SELECTION/);
  backend.cancelAll();
});

test("legacy Host environment cannot activate domain tools in an offline installed package", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "fabushi-desktop-bridge-e2e-"));
  const previousEnv = {
    FABUSHI_DANDELION_BIN: process.env.FABUSHI_DANDELION_BIN,
    FABUSHI_DANDELION_CONFIG: process.env.FABUSHI_DANDELION_CONFIG,
    FABUSHI_DANDELION_ROOTS: process.env.FABUSHI_DANDELION_ROOTS,
    FABUSHI_DANDELION_POOLS: process.env.FABUSHI_DANDELION_POOLS,
    FABUSHI_MINIAPP_PRELOAD: process.env.FABUSHI_MINIAPP_PRELOAD,
    FABUSHI_DANDELION_HOTSPOT: process.env.FABUSHI_DANDELION_HOTSPOT,
    FABUSHI_DANDELION_UPDATE: process.env.FABUSHI_DANDELION_UPDATE,
  };
  try {
    const allowedRoot = join(workspace, "authorized-root");
    const datasetDir = join(allowedRoot, "batch-01");
    await mkdir(datasetDir, { recursive: true });
    const configPath = join(workspace, "dandelion.toml");
    const preloadPath = join(workspace, "preload-miniapp-tools.cjs");
    const binPath = join(workspace, "dandelion");
    await writeFile(configPath, "# e2e config\n", "utf8");
    await writeFile(preloadPath, "// compiled preload stub\n", "utf8");
    await writeFile(
      binPath,
      [
        "#!/usr/bin/env python3",
        "import json, sys",
        "args = sys.argv[1:]",
        "assert args[:2] == ['--config', " + JSON.stringify(configPath) + "]",
        "cmd = args[2:]",
        "if cmd[:1] == ['doctor']:",
        "    print(json.dumps({'native_contract': 'dandelion.native.apple.v1', 'os': 'macos', 'target': 'aarch64-apple-darwin'}))",
        "elif cmd[:2] == ['scan', 'metadata']:",
        "    print(json.dumps({'files': 5, 'bytes': 8192}))",
        "elif cmd[:1] == ['run']:",
        "    print(json.dumps({'packets': 16, 'verifiedBytes': 8192}))",
        "elif cmd == ['hotspot', 'plan', '--json']:",
        "    print(json.dumps({'profileId': 'dandelion-lan-01'}))",
        "elif cmd == ['hotspot', 'apply', '--confirm', '--json']:",
        "    print(json.dumps({'active': True}))",
        "else:",
        "    sys.exit(2)",
      ].join("\n"),
      "utf8",
    );
    await chmod(binPath, 0o755);

    process.env.FABUSHI_DANDELION_BIN = binPath;
    process.env.FABUSHI_DANDELION_CONFIG = configPath;
    process.env.FABUSHI_DANDELION_ROOTS = [allowedRoot].join(delimiter);
    process.env.FABUSHI_DANDELION_POOLS = JSON.stringify([{ id: "lan-pool", label: "局域网目标" }]);
    process.env.FABUSHI_MINIAPP_PRELOAD = preloadPath;
    process.env.FABUSHI_DANDELION_HOTSPOT = "1";
    process.env.FABUSHI_DANDELION_UPDATE = "0";

    const trustedManagementSender = {};
    const accountListeners = new Set<() => void>();
    const ipcHandlers = new Map<string, (...args: unknown[]) => Promise<unknown>>();
    let approveNext = false;
    let openedWindow: { webContents: MiniAppWindow["webContents"]; options: MiniAppWindowOptions; closed: boolean } | null = null;

    const fakeSession: MiniAppSession = {
      protocol: { handle: async () => {}, unhandle: () => {} },
      webRequest: { onBeforeRequest: () => {} },
      setProxy: async () => {},
      setPermissionRequestHandler: () => {},
      setPermissionCheckHandler: () => {},
      on: () => {},
      removeListener: () => {},
    };

    class HeadlessMiniAppWindow implements MiniAppWindow {
      readonly webContents: MiniAppWindow["webContents"];
      closed = false;
      constructor(readonly options: MiniAppWindowOptions) {
        this.webContents = {
          session: fakeSession,
          setWebRTCIPHandlingPolicy: async () => {},
          setWindowOpenHandler: () => ({ action: "deny" as const }),
          on: () => {},
          loadURL: async () => {},
        };
        openedWindow = this;
      }
      isDestroyed() { return this.closed; }
      show() {}
      focus() {}
      close() { this.closed = true; }
      destroy() { this.closed = true; }
      on() {}
    }

    const authId = "user-desktop-e2e";
    const expectedAccountScope = createHash("sha256").update(authId).digest("hex");
    const appId = "org.fabushi.global-dandelion";
    const entry = {
      appId,
      publisherId: "bhrumom",
      displayName: "Global Dandelion 离线工作台",
      version: "0.2.0",
      packageDigest: "f".repeat(64),
      sizeBytes: 479376,
      offlineCore: true,
      installed: true,
      supported: true,
    };

    const context = {
      native: {
        app: { commandLine: { hasSwitch: () => false } },
        session: { fromPartition: () => fakeSession },
        BrowserWindow: HeadlessMiniAppWindow,
        dialog: {
          showOpenDialog: async () => ({ canceled: false, filePaths: [datasetDir] }),
          showMessageBox: async () => ({ response: approveNext ? 1 : 0 }),
        },
      },
      getTrustedContents: () => trustedManagementSender,
      requireAccount: () => ({
        getStatus: async () => ({ kind: "logged-in", authId }),
        subscribe: (listener: () => void) => {
          accountListeners.add(listener);
          return () => { accountListeners.delete(listener); };
        },
      }),
      coordinatorLegs: {
        legs: {
          listInstalledMiniApps: async () => ({ items: [entry] }),
          listApprovedMiniApps: async () => ({ items: [entry] }),
          installApprovedMiniApp: async () => {},
          uninstallMiniApp: async () => {},
          getInstalledMiniAppResources: async () => ({
            sameScopeHash: expectedAccountScope,
            descriptor: { appId, version: "0.2.0", digest: "f".repeat(64), entrypoint: "index.html" },
            resources: [
              { path: "index.html", mime: "text/html", bytesBase64: Buffer.from("<!doctype html><title>GD</title>").toString("base64") },
            ],
          }),
        },
      },
    } as unknown as ProductionServiceContext;

    const registration = registerProductionMiniApps(context, {
      handle: (channel, listener) => {
        ipcHandlers.set(channel, listener as (...args: unknown[]) => Promise<unknown>);
      },
    });

    try {
      await ipcHandlers.get("sand:miniapp-launch")!({ sender: trustedManagementSender }, appId);
      assert.ok(openedWindow, "window must be created on launch");
      assert.equal(openedWindow.options.webPreferences.preload, undefined);
      assert.equal(openedWindow.options.webPreferences.sandbox, true);
      assert.equal(openedWindow.options.webPreferences.contextIsolation, true);
      assert.equal(openedWindow.options.webPreferences.nodeIntegration, false);

      assert.equal(ipcHandlers.has(MINIAPP_TOOL_CHANNEL), false, "old domain env must not create a bridge for an offline package");
      for (const listener of accountListeners) listener();
      assert.equal(openedWindow.closed, true);
    } finally {
      registration.dispose();
    }
  } finally {
    for (const [key, value] of Object.entries(previousEnv)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    await rm(workspace, { recursive: true, force: true });
  }
});
