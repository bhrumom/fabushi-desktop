import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, realpath, rm, symlink, writeFile } from "node:fs/promises";
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
      { binaryPath: binPath, configPath, allowedRoots: [allowedRoot], targetPools: [{ id: "lan", label: "LAN" }] },
      { pick: async () => [{ path: chosenDir, name: "dir", size: 4096 }] },
    );
    await assert.rejects(() => disabledBackend.hotspot("plan"), /HOTSPOT_DISABLED/);
    await assert.rejects(() => disabledBackend.update("check"), /UPDATE_DISABLED/);

    const backend = createDandelionCliBackend(
      {
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

    await assert.rejects(() => backend.send(fastPicked.selectionId, "unknown-pool"), /UNKNOWN_TARGET_POOL/);
    const started = await backend.send(fastPicked.selectionId, "lan");
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
    const slowOp = await backend.send(slowPicked.selectionId, "lan");
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

test("production desktop bridge runs end-to-end without Electron from preload IPC through router, approval gate and dandelion CLI", async () => {
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
        "if cmd[:2] == ['scan', 'metadata']:",
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
      assert.equal(openedWindow.options.webPreferences.preload, preloadPath);
      assert.equal(openedWindow.options.webPreferences.sandbox, true);
      assert.equal(openedWindow.options.webPreferences.contextIsolation, true);
      assert.equal(openedWindow.options.webPreferences.nodeIntegration, false);

      const toolIpc = ipcHandlers.get(MINIAPP_TOOL_CHANNEL);
      assert.ok(toolIpc, "preload IPC handler must be registered");

      const hostHash = createHash("sha256").update(expectedAccountScope).update("\0").update(appId).digest("hex");
      const validOrigin = `fabushi-miniapp://${hostHash}.app`;
      const invokeFromRenderer = (tool: string, args: Record<string, unknown> = {}, frameUrl = `${validOrigin}/index.html`) =>
        toolIpc({ sender: openedWindow!.webContents, senderFrame: { url: frameUrl } } satisfies MiniAppIpcEvent, tool, args) as Promise<{
          ok: boolean;
          data?: Record<string, unknown>;
          error?: { code: string; message: string };
        }>;

      // 1. Capabilities probe matches host-tools.js expectations.
      const caps = await invokeFromRenderer("offline.capabilities", { contract: MINIAPP_TOOL_CONTRACT });
      assert.equal(caps.ok, true);
      assert.equal(caps.data?.contract, MINIAPP_TOOL_CONTRACT);
      assert.equal(caps.data?.platform, "desktop");
      assert.deepEqual(caps.data?.capabilities, ["offline", "selection", "transfer", "scan", "config", "platform", "diagnostics", "hotspot"]);
      assert.deepEqual(caps.data?.targetPools, [{ id: "lan-pool", label: "局域网目标" }]);

      // 2. Directory selection via native dialog and canonical root validation.
      const picked = await invokeFromRenderer("selection.pick", { mode: "directory" });
      assert.equal(picked.ok, true);
      const selectionId = String(picked.data?.selectionId ?? "");
      assert.match(selectionId, /^[0-9a-f-]{36}$/);

      // 3. Metadata scan executes real dandelion binary.
      const scanned = await invokeFromRenderer("scan.metadata", { rootId: selectionId });
      assert.equal(scanned.ok, true);
      assert.deepEqual(scanned.data, { files: 5, bytes: 8192 });

      // 4. Mutating transfer.send is denied when user cancels the approval dialog.
      approveNext = false;
      const deniedSend = await invokeFromRenderer("transfer.send", { selectionId, targetPoolId: "lan-pool" });
      assert.equal(deniedSend.ok, false);
      assert.equal(deniedSend.error?.code, "NOT_APPROVED");

      // 5. Mutating transfer.send runs when user approves and completes asynchronously.
      approveNext = true;
      const approvedSend = await invokeFromRenderer("transfer.send", { selectionId, targetPoolId: "lan-pool" });
      assert.equal(approvedSend.ok, true);
      const operationId = String(approvedSend.data?.operationId ?? "");
      let statusOut: { ok: boolean; data?: Record<string, unknown> } = { ok: true, data: { state: "running" } };
      for (let attempt = 0; attempt < 40; attempt += 1) {
        statusOut = await invokeFromRenderer("transfer.status", { operationId });
        if (statusOut.data?.state !== "running") break;
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
      assert.equal(statusOut.ok, true);
      assert.equal(statusOut.data?.state, "complete");
      assert.deepEqual(statusOut.data?.result, { packets: 16, verifiedBytes: 8192 });

      // 6. Forged frame origin or untrusted sender webContents is rejected before reaching router.
      const badOrigin = await invokeFromRenderer("offline.capabilities", {}, "https://evil.example/index.html");
      assert.equal(badOrigin.ok, false);
      assert.equal(badOrigin.error?.code, "UNAUTHORIZED_SENDER");

      const badSender = (await toolIpc(
        { sender: {} as MiniAppWindow["webContents"], senderFrame: { url: `${validOrigin}/index.html` } },
        "offline.capabilities",
        {},
      )) as { ok: boolean; error?: { code: string } };
      assert.equal(badSender.ok, false);
      assert.equal(badSender.error?.code, "UNAUTHORIZED_SENDER");

      // 7. Account change closes the window and fences any late tool calls.
      for (const listener of accountListeners) listener();
      assert.equal(openedWindow.closed, true);
      const afterLogout = await invokeFromRenderer("offline.capabilities", {});
      assert.equal(afterLogout.ok, false);
      assert.equal(afterLogout.error?.code, "UNAUTHORIZED_SENDER");
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
