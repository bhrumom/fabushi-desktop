import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import test from "node:test";

import { getSimulatedGatewayLatencyMs, setSimulatedGatewayLatencyMs, SIMULATED_GATEWAY_LATENCY_MAX_MS } from "./dev/dev-network-latency.js";
import { DEFAULT_DEV_CONTROL_PORT, isDevControlsEnabled, resolveDevControlPort } from "./dev/dev-controls-gate.js";
import { SAND_DEV_PRELOAD_FILENAME, SAND_PRIMARY_PRELOAD_FILENAME, resolveSandMainWindowPreload } from "./dev/dev-capability.js";
import { createDevGatewayOfflineControl } from "./dev/dev-gateway-offline.js";
import { registerExperimentsIpc } from "./experiments/experiments-ipc.js";
import { computeDockBadgeTotal } from "./notifications/dock-badge.js";
import { SandOsNotificationManager } from "./notifications/os-notification-manager.js";
import { resolveScanRoots } from "./process-metrics/wiring.js";
import { computeUpdateDisabledReason } from "./update/update-gate.js";
import { isSafeToRelaunchForUpdate } from "./update/safe-relaunch-gate.js";
import { SandWindowsInstaller, WINDOWS_PARENT_EXIT_GRACE_MS, buildWindowsInstallerParentHandoffScript } from "./update/win32-installer.js";
import { createProductionWindowBroadcaster } from "./window-broadcast.js";
import { unavailableOnePasswordProvisioningSink, OnePasswordProvisioningError } from "./onepassword/onepassword-provisioning-contract.js";
import { requireDisposable, requireFunction, requireObject } from "./adapters/provider-guards.js";
import { registerSettingsIpc } from "./prefs/settings-ipc.js";
import { isValidSendLatencyReport, sendLatencyReportToTelemetry } from "./telemetry/send-telemetry.js";
import { isValidAgentsUnreachableReport, agentsUnreachableReportToTelemetry } from "./telemetry/agents-unreachable-telemetry.js";
import { createBoxVncHandlers, createBoxVncTrust } from "./vnc/vnc-edge.js";
import { classifyWindowShortcut } from "./window-shortcuts.js";
import { hashProcessName, sanitizeProcessName } from "./process-metrics/redaction.js";
import { RECENT_WAKE_WINDOW_MS, connectivityStamps, createDesktopConnectivity } from "./coordinator/desktop-connectivity.js";
import { resolveDefaultDownloadDir, resolveDefaultDownloadPath, resolveSuggestedDownloadName } from "./downloads/download-path.js";
import { assertTrustedClientPersistenceSender, assertTrustedSecretsSender, isTrustedSecretsSender, UntrustedClientPersistenceSenderError, UntrustedSecretsSenderError } from "./secrets/secrets-ipc-guard.js";
import { createIdleRelaunchSignals, isScreensaverRunning } from "./update/idle-relaunch-signals.js";
import { createDesktopAccountAuthorizer } from "./account/account-authorization.js";
import { createSandRecreateCommands, type RecreateOperationId } from "./box/box-recreate-commands.js";
import { createDesktopHostSettingsFields } from "./prefs/host-settings-fields.js";
import { createReleaseMetadata } from "./update/release-metadata.js";
import { SandSettingsStore } from "../shared/node/settings/sand-settings-store.js";
import { normalizeSandCallMediaPreferences, normalizeSandUiPreferences, resolveSandUiDirection } from "../shared/desktop.js";
import { createDesktopCallMediaPort } from "./call-media.js";
import { normalizeCommitStagedAttachmentsEdgeRequest, normalizeStageAttachmentEdgeRequest } from "./attachments/attachment-edge-wire.js";
import { createAttachmentEdgePort, type AttachmentEdgeDeps } from "./attachments/attachments.js";

test("main edge accepts the shipping preload attachment wire contract", () => {
  const staged = normalizeStageAttachmentEdgeRequest({
    filename: "phase1-human-reply.txt",
    bytesBase64: Buffer.from("Fabushi Human attachment exact-head evidence.", "utf8").toString("base64"),
  });
  assert.equal(staged.filename, "phase1-human-reply.txt");
  assert.equal(Buffer.from(staged.bytes as Uint8Array).toString("utf8"), "Fabushi Human attachment exact-head evidence.");
  assert.equal(normalizeStageAttachmentEdgeRequest({ filename: "bad.txt", bytesBase64: "***" }).bytes, undefined);

  assert.deepEqual(normalizeCommitStagedAttachmentsEdgeRequest({ items: [
    { path: "/staging/a.txt", name: "a.txt" },
    { path: "/staging/b.png", name: "b.png" },
  ] }), {
    paths: ["/staging/a.txt", "/staging/b.png"],
    filenames: ["a.txt", "b.png"],
  });
  assert.deepEqual(normalizeCommitStagedAttachmentsEdgeRequest({
    paths: ["/legacy/a.txt"],
    filenames: ["a.txt"],
  }), {
    paths: ["/legacy/a.txt"],
    filenames: ["a.txt"],
  });
});

test("attachment staging uses the production filesystem owner and Node UUID source", async () => {
  const dir = mkdtempSync(join(tmpdir(), "fabushi-attachment-stage-"));
  const failures: Array<{ leg: string; errorClass: string }> = [];
  try {
    const edge = createAttachmentEdgePort({
      byteLimitForName: () => 1024,
      getStagingDir: () => dir,
      onEdgeFailure: (failure: { leg: string; errorClass: string }) => failures.push(failure),
      now: () => 1_700_000_000_000,
    } as unknown as AttachmentEdgeDeps);
    const bytes = new Uint8Array(Buffer.from("Fabushi Human attachment exact-head evidence.", "utf8"));
    const staged = await edge.stageBytes("phase1-human-reply.txt", bytes);
    if (!staged.ok) assert.fail("Expected production attachment staging to succeed.");
    assert.match(basename(staged.path), /^1700000000000-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.txt$/u);
    assert.equal(readFileSync(staged.path, "utf8"), "Fabushi Human attachment exact-head evidence.");
    assert.deepEqual(failures, []);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("UI accessibility preferences normalize, persist, and resolve RTL", () => {
  const dir=mkdtempSync(join(tmpdir(),"fabushi-ui-prefs-"));
  try {
    const store=new SandSettingsStore(join(dir,"settings.json"));
    store.setUiPreferences({ locale:"ar-SA", direction:"auto", reducedMotion:true, highContrast:true, textScale:1.25 });
    assert.deepEqual(store.getUiPreferences(), { locale:"ar-SA", direction:"auto", reducedMotion:true, highContrast:true, textScale:1.25 });
    assert.equal(resolveSandUiDirection(store.getUiPreferences()), "rtl");
    assert.deepEqual(normalizeSandUiPreferences({ locale:"../../invalid", direction:"sideways", reducedMotion:"yes", highContrast:true, textScale:9 }), { locale:"system", direction:"auto", reducedMotion:false, highContrast:true, textScale:2 });
  } finally { rmSync(dir,{recursive:true,force:true}); }
});

test("call media device preferences normalize and persist in the canonical settings store", () => {
  const dir=mkdtempSync(join(tmpdir(),"fabushi-call-media-prefs-"));
  try {
    const store=new SandSettingsStore(join(dir,"settings.json"));
    store.setCallMediaPreferences({ microphoneId:" mic-primary ", cameraId:"camera-primary" });
    assert.deepEqual(store.getCallMediaPreferences(), { microphoneId:"mic-primary", cameraId:"camera-primary" });
    assert.deepEqual(normalizeSandCallMediaPreferences({ microphoneId:"", cameraId:"bad\nvalue" }), { microphoneId:null, cameraId:null });
    store.setCallMediaPreferences({ microphoneId:null, cameraId:null });
    assert.deepEqual(store.getCallMediaPreferences(), { microphoneId:null, cameraId:null });
  } finally { rmSync(dir,{recursive:true,force:true}); }
});

test("system notification preferences remain disabled by the canonical settings owner", () => {
  const dir=mkdtempSync(join(tmpdir(),"fabushi-notification-prefs-"));
  try {
    const store=new SandSettingsStore(join(dir,"settings.json"));
    store.setNotificationConfig({ isEnabled:true, allowedApps:["example"] });
    const notification=store.getNotificationConfig();
    assert.equal(notification.isEnabled,false);
    assert.deepEqual(notification.allowedApps,[]);
  } finally { rmSync(dir,{recursive:true,force:true}); }
});

test("dev gates and latency clamps match Grok behavior", () => {
  assert.equal(setSimulatedGatewayLatencyMs(25.9), 25);
  assert.equal(getSimulatedGatewayLatencyMs(), 25);
  assert.equal(setSimulatedGatewayLatencyMs(Infinity), 0);
  assert.equal(setSimulatedGatewayLatencyMs(SIMULATED_GATEWAY_LATENCY_MAX_MS + 99), SIMULATED_GATEWAY_LATENCY_MAX_MS);

  assert.equal(isDevControlsEnabled({ isPackaged: false }), true);
  assert.equal(isDevControlsEnabled({ isPackaged: true }), false);
  assert.equal(resolveDevControlPort({ SAND_DEV_CONTROL_PORT: "62001" }), 62001);
  assert.equal(resolveDevControlPort({ SAND_DEV_CONTROL_PORT: "70000" }), DEFAULT_DEV_CONTROL_PORT);
  assert.equal(resolveSandMainWindowPreload({ isPackaged: false, env: { SAND_DEV_CAPABILITY: "1" } }), SAND_DEV_PRELOAD_FILENAME);
  assert.equal(resolveSandMainWindowPreload({ isPackaged: true, env: { SAND_DEV_CAPABILITY: "1" } }), SAND_PRIMARY_PRELOAD_FILENAME);
});

test("dev gateway offline control serializes and reapplies state", async () => {
  const calls: boolean[] = [];
  const control = createDevGatewayOfflineControl(async (next) => {
    calls.push(next);
    return { induced: next };
  });
  assert.deepEqual(await Promise.all([control.apply(true), control.apply(false)]), [{ induced: true }, { induced: false }]);
  assert.deepEqual(calls, [true, false]);
  assert.equal(control.isInduced(), false);
  await control.apply(true);
  assert.deepEqual(await control.reapplyAfterCoordinatorLaunch(), { induced: true });
  assert.deepEqual(calls, [true, false, true, true]);
});

test("sync IPC helpers expose experiment and settings snapshots", () => {
  const listeners = new Map<string, (event: { returnValue: unknown }) => void>();
  const ipcMain = { on(channel: string, listener: (event: { returnValue: unknown }) => void) { listeners.set(channel, listener); } };
  registerExperimentsIpc({ ipcMain, getExperimentService: () => ({ getSnapshot: () => ({ flag: true }) }) });
  registerSettingsIpc({
    ipcMain,
    settingsStore: { getEgressTunnelEnabled: () => true, getWebauthnProxyEnabled: () => false },
    themeController: { getState: () => ({ theme: "dark" }) },
    egressTunnelController: { getStatus: () => ({ connected: true }) },
  });
  const invoke = (channel: string) => {
    const event = { returnValue: undefined as unknown };
    listeners.get(channel)?.(event);
    return event.returnValue;
  };
  assert.deepEqual(invoke("sand:experiments-snapshot-sync"), { flag: true });
  assert.deepEqual(invoke("sand:theme-get-sync"), { theme: "dark" });
  assert.equal(invoke("sand:egress-tunnel-get-sync"), true);
  assert.equal(invoke("sand:webauthn-proxy-get-sync"), false);
  assert.deepEqual(invoke("sand:egress-tunnel-status-get-sync"), { connected: true });
});

test("dock badge, process roots, and window broadcast preserve production semantics", async () => {
  assert.equal(computeDockBadgeTotal([
    { hasUnread: true },
    { hasUnread: true, unreadCount: 3.9 },
    { hasUnread: true, unreadCount: 0 },
    { hasUnread: true, unreadCount: 9, isHiddenFromSidebar: true },
    { hasUnread: false, unreadCount: 5 },
  ]), 5);

  assert.deepEqual(await resolveScanRoots(10, async () => ({ pid: 20 }), (pid) => pid === 20), [10, 20]);
  assert.deepEqual(await resolveScanRoots(10, async () => { throw new Error("missing"); }, () => true), [10]);

  const received: Array<[string, unknown]> = [];
  const broadcast = createProductionWindowBroadcaster({
    getAllWindows: () => [
      { webContents: { send: (channel, payload) => received.push([channel, payload]) } },
      { webContents: { send: (channel, payload) => received.push([channel, payload]) } },
    ],
  });
  broadcast("event", { value: 7 });
  assert.deepEqual(received, [["event", { value: 7 }], ["event", { value: 7 }]]);
});

test("update gates fail closed unless every safe-relaunch condition holds", () => {
  assert.equal(computeUpdateDisabledReason({ envDisabled: true, isLabBuild: false, hasDevFeedOverride: false, isPackaged: true, platform: "darwin" }), "disabled-by-env");
  assert.equal(computeUpdateDisabledReason({ envDisabled: false, isLabBuild: true, hasDevFeedOverride: false, isPackaged: true, platform: "darwin" }), "lab-build");
  assert.equal(computeUpdateDisabledReason({ envDisabled: false, isLabBuild: false, hasDevFeedOverride: true, isPackaged: false, platform: "linux" }), null);
  assert.equal(computeUpdateDisabledReason({ envDisabled: false, isLabBuild: false, hasDevFeedOverride: false, isPackaged: false, platform: "darwin" }), "not-packaged");
  assert.equal(computeUpdateDisabledReason({ envDisabled: false, isLabBuild: false, hasDevFeedOverride: false, isPackaged: true, platform: "linux" }), "unsupported-platform");

  const safe = {
    optInEnabled: true,
    gateEnabled: true,
    updateStaged: true,
    hostIdle: { kind: "confirmed-idle" },
    screenLocked: true,
    screensaverActive: false,
    systemIdleSeconds: 900,
    idleThresholdSeconds: 600,
  };
  assert.equal(isSafeToRelaunchForUpdate(safe), true);
  assert.equal(isSafeToRelaunchForUpdate({ ...safe, hostIdle: { kind: "busy" } }), false);
  assert.equal(isSafeToRelaunchForUpdate({ ...safe, screenLocked: false }), false);
});


test("Windows updater handoff waits for the exact parent identity before NSIS apply", () => {
  const script = buildWindowsInstallerParentHandoffScript({
    installerPath: "C:\\Fabushi's Updates\\fabushi setup.exe",
    installerArgs: ["--updated", "/S", "--force-run"],
    parentProcessId: 4242,
  });
  assert.match(script, /\$parentId = 4242/);
  assert.match(script, new RegExp(`\\$graceMs = ${WINDOWS_PARENT_EXIT_GRACE_MS}`));
  assert.match(script, /StartTime\.ToUniversalTime\(\)\.Ticks/);
  assert.match(script, /WaitForExit\(\$graceMs\)/);
  assert.match(script, /\$currentCreated -eq \$parentCreated/);
  assert.match(script, /Stop-Process -Id \$parentId -Force/);
  assert.match(script, /C:\\Fabushi''s Updates\\fabushi setup\.exe/);
  assert.match(script, /--force-run/);

  const spawned: Array<{ command: string; args: readonly string[] }> = [];
  const installer = new SandWindowsInstaller({
    quit: () => undefined,
    parentProcessId: 777,
    spawnDetached: (command, args) => { spawned.push({ command, args }); },
  });
  installer.installOnQuit("C:\\updates\\fabushi.exe", { forceRun: true });
  assert.equal(spawned.length, 1);
  assert.equal(spawned[0]?.command, "powershell.exe");
  assert.deepEqual(spawned[0]?.args.slice(0, 4), [
    "-NoProfile",
    "-NonInteractive",
    "-WindowStyle",
    "Hidden",
  ]);
  const handoff = spawned[0]?.args.at(-1) ?? "";
  assert.match(handoff, /\$parentId = 777/);
  assert.match(handoff, /Start-Process -FilePath 'C:\\updates\\fabushi\.exe'/);
});

test("provider guards and unavailable 1Password sink fail closed", async () => {
  assert.deepEqual(requireObject({ ok: true }, "object"), { ok: true });
  assert.throws(() => requireObject(null, "object"), /Missing Electron production adapter port/);
  const fn = () => 1;
  assert.equal(requireFunction(fn, "fn"), fn);
  assert.throws(() => requireFunction(undefined, "fn"), /Missing Electron production adapter port/);
  const disposable = { dispose() {} };
  assert.equal(requireDisposable(disposable, "resource"), disposable);
  assert.throws(() => requireDisposable({} as { dispose(): void }, "resource"), /resource\.dispose/);
  await assert.rejects(
    () => unavailableOnePasswordProvisioningSink.accept({}),
    (error: unknown) => error instanceof OnePasswordProvisioningError && error.code === "sink-unavailable",
  );
});

test("send and agents telemetry validators reject malformed input", () => {
  assert.equal(isValidSendLatencyReport({ durationMs: 12.5, attachmentCount: 2, isFork: false }), true);
  assert.equal(isValidSendLatencyReport({ durationMs: -1, attachmentCount: 2, isFork: false }), false);
  assert.deepEqual(sendLatencyReportToTelemetry({ durationMs: 12.5, attachmentCount: 2, isFork: false }), {
    level: "info",
    metadata: {
      duration_ms: "13",
      commit_ms: undefined,
      attachment_count: "2",
      is_fork: "false",
      trace_id: undefined,
      span_id: undefined,
      conversation_id: undefined,
    },
  });
  assert.equal(isValidAgentsUnreachableReport({ phase: "entered", isManual: false, msSinceLastReachedMs: 10 }), true);
  assert.equal(isValidAgentsUnreachableReport({ phase: "bad", isManual: false }), false);
  assert.equal(agentsUnreachableReportToTelemetry({ phase: "entered", isManual: false }).level, "warn");
  assert.equal(agentsUnreachableReportToTelemetry({ phase: "recovered", isManual: false }).level, "info");
});

test("VNC trust and handlers expose only the trusted box webview", () => {
  const trust = createBoxVncTrust((url) => url === "https://trusted/");
  assert.equal(trust.boxDesktopWebview.test({ isBoxVncPartition: true, frameUrl: "https://trusted/" }), true);
  assert.equal(trust.boxDesktopWebview.test({ isBoxVncPartition: false, frameUrl: "https://trusted/" }), false);
  assert.equal(trust.boxDesktopWebview.test({ isBoxVncPartition: true, frameUrl: "https://evil/" }), false);

  let clipboard = "initial";
  let presence = false;
  const handlers = createBoxVncHandlers({
    readClipboardText: () => clipboard,
    writeClipboardText: (text) => { clipboard = text; },
    onUserPresence: (value) => { presence = value; },
  });
  assert.equal(handlers.boxDesktopWebview.readClipboard(), "initial");
  handlers.boxDesktopWebview.writeClipboard({ text: "next" });
  handlers.boxDesktopWebview.reportUserPresence({ isPresent: true });
  assert.equal(clipboard, "next");
  assert.equal(presence, true);
});

test("window shortcut classifier matches platform chords", () => {
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "F11" }, "linux"), "fullscreen");
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "f", meta: true, control: true }, "darwin"), "fullscreen");
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "i", meta: true, alt: true }, "darwin"), "toggledevtools");
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "i", control: true, shift: true }, "win32"), "toggledevtools");
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "r", control: true }, "win32"), "reload");
  assert.equal(classifyWindowShortcut({ type: "keyDown", key: "q", meta: true }, "darwin"), "quit");
  assert.equal(classifyWindowShortcut({ type: "keyUp", key: "F11" }, "linux"), null);
});


test("process redaction keeps only Grok helper labels and hashes every original name", () => {
  const helper = sanitizeProcessName("/Applications/Fabushi Helper (GPU)");
  assert.equal(helper.name, "Fabushi Helper (GPU)");
  assert.equal(helper.nameHash, hashProcessName("/Applications/Fabushi Helper (GPU)"));
  const legacyHelper = sanitizeProcessName("/Applications/Grok Bot Helper (GPU)");
  assert.equal(legacyHelper.name, "Grok Bot Helper (GPU)");
  assert.equal(legacyHelper.nameHash, hashProcessName("/Applications/Grok Bot Helper (GPU)"));
  const foreign = sanitizeProcessName("/tmp/secret-app --token=abc");
  assert.equal(foreign.name, "secret-app");
  assert.equal(foreign.nameHash.length, 64);
  assert.notEqual(foreign.nameHash, foreign.name);
});

test("desktop connectivity tracks only recent resumes and exposes telemetry stamps", () => {
  let now = 1_000;
  let resume: (() => void) | undefined;
  let online = true;
  const connectivity = createDesktopConnectivity({
    isOnline: () => online,
    onResume: (listener) => { resume = listener; },
    monotonicNow: () => now,
  });
  assert.equal(connectivity.recentWake(), false);
  resume?.();
  assert.equal(connectivity.recentWake(), true);
  now += RECENT_WAKE_WINDOW_MS;
  assert.equal(connectivity.recentWake(), false);
  online = false;
  assert.deepEqual(connectivityStamps(connectivity), { client_online: "false", recent_wake: "false" });
});

test("download path resolution rejects relative configured roots and unsafe suggested names", () => {
  assert.equal(resolveDefaultDownloadDir({ configuredDir: "/safe/downloads", osDownloadsDir: "/os/downloads" }), "/safe/downloads");
  assert.equal(resolveDefaultDownloadDir({ configuredDir: "relative", osDownloadsDir: "/os/downloads" }), "/os/downloads");
  assert.equal(resolveDefaultDownloadPath({ configuredDir: "/safe", osDownloadsDir: "/os", fileName: "../nested/file.txt" }), "/safe/file.txt");
  assert.equal(resolveSuggestedDownloadName({ sourcePath: "/tmp/report.pdf", suggestedName: "../../renamed.pdf" }), "renamed.pdf");
  assert.equal(resolveSuggestedDownloadName({ sourcePath: "/tmp/report.pdf", suggestedName: "renamed.exe" }), "report.pdf");
});

test("secrets IPC guard requires both trusted webContents and exact main frame", () => {
  const contents = {};
  const frame = {};
  const trusted = { sender: contents, senderFrame: frame, trustedContents: contents, trustedMainFrame: frame };
  assert.equal(isTrustedSecretsSender(trusted), true);
  assert.doesNotThrow(() => assertTrustedSecretsSender(trusted));
  assert.doesNotThrow(() => assertTrustedClientPersistenceSender(trusted));
  assert.throws(
    () => assertTrustedSecretsSender({ ...trusted, sender: {} }),
    (error: unknown) => error instanceof UntrustedSecretsSenderError,
  );
  assert.throws(
    () => assertTrustedClientPersistenceSender({ ...trusted, senderFrame: {} }),
    (error: unknown) => error instanceof UntrustedClientPersistenceSenderError,
  );
});

test("idle relaunch signals preserve platform and power-monitor semantics", async () => {
  assert.equal(
    await isScreensaverRunning(
      "linux",
      ((
        _file: string,
        _args: readonly string[],
        _options: { timeout: number },
        callback: (error: null) => void,
      ) => callback(null)) as any,
    ),
    false,
  );
  const signals = createIdleRelaunchSignals({
    powerMonitor: {
      getSystemIdleState: () => "locked",
      getSystemIdleTime: () => 777,
    },
    probeHostIdle: async () => ({ kind: "confirmed-idle" }),
    screensaverProbe: async () => true,
  });
  assert.equal(signals.getScreenLocked(), true);
  assert.equal(await signals.getScreensaverActive(), true);
  assert.equal(signals.getSystemIdleSeconds(), 777);
  assert.deepEqual(await signals.probeHostIdle(), { kind: "confirmed-idle" });
});


test("account authorization scopes durable state only after approval", async () => {
  let scoped: string | undefined;
  let abandoned = 0;
  let storedScope = "foreign";
  const authorizeCalls: unknown[] = [];
  const authorize = createDesktopAccountAuthorizer({
    binding: {
      async authorize(args) {
        authorizeCalls.push(args);
        return args.accountSlot !== "deny";
      },
    },
    descriptorUrl: "https://gateway.example/descriptor",
    hasExistingDurableData: () => true,
    store: {
      getMcpCustomInstructionsAccountScope: () => storedScope,
      scopeToAccount(scope) {
        scoped = scope;
        storedScope = scope;
      },
    },
    abandonForeignOnboardingMirror: () => { abandoned += 1; },
  });

  assert.equal(await authorize("deny", { isStartup: true }), false);
  assert.equal(scoped, undefined);
  assert.equal(await authorize("account-token", { isStartup: true }), true);
  assert.match(scoped ?? "", /^[0-9a-f]{64}$/);
  assert.equal(abandoned, 1);
  assert.equal(authorizeCalls.length, 2);
});


test("box recreate commands preserve tracked, untrackable, fallback and rejected outcomes", async () => {
  const accepted: Array<RecreateOperationId | null> = [];
  const tracked = createSandRecreateCommands<{ reason: string }>({
    connector: {
      recreate: async () => ({ status: "started", operationId: "operation-1" as any }),
      forceRecreate: async () => ({ status: "started-untrackable" }),
    },
    noteRecreateAccepted: (operationId) => accepted.push(operationId),
  });
  assert.deepEqual(await tracked.recreateComputer({ reason: "test" }), {
    status: "started",
    operationId: "operation-1",
  });
  assert.deepEqual(await tracked.forceRecreateComputer(), { status: "started-untrackable" });
  assert.deepEqual(accepted, ["operation-1", null]);

  const unavailable = createSandRecreateCommands<{}>({
    connector: {},
    noteRecreateAccepted: () => assert.fail("unavailable connector must not record acceptance"),
  });
  assert.deepEqual(await unavailable.recreateComputer({}), { status: "dev-fallback" });
  assert.deepEqual(await unavailable.forceRecreateComputer(), {
    status: "rejected",
    reason: "Reset Fabushi's Computer is unavailable without a backend connection.",
  });
});

test("host settings field hydrates from live box state and clears on account departure", async () => {
  let local: boolean | undefined = false;
  let cleared = 0;
  const persistence: Array<Record<string, string>> = [];
  const fields = createDesktopHostSettingsFields({
    read: async () => ({ hasSeenOnboarding: true }),
    write: async (update) => ({ ...update }),
    store: {
      getHasSeenOnboarding: () => local,
      setHasSeenOnboarding(value) { local = value; },
      clearHasSeenOnboarding() { local = undefined; cleared += 1; },
    },
    reportPersistence: (event) => persistence.push(event),
  });

  assert.equal(await fields.onboardingSeen.reconcile(), false);
  fields.onTransportConnected();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(local, true);
  assert.deepEqual(persistence, [{
    kind: "client_persistence",
    op: "writeback",
    outcome: "ok",
    slice: "host-settings.onboarding",
  }]);

  fields.onAccountDeparted();
  assert.equal(local, undefined);
  assert.equal(cleared, 1);
});


test("release metadata resolves package identity and live update gates", async () => {
  const metadata = createReleaseMetadata({
    packageVersion: "1.2.75",
    packageTrack: "not-a-track",
    isLabBuild: false,
    app: { getVersion: () => "fallback-version", isPackaged: true },
    env: { SAND_DISABLE_UPDATES: "1" },
    platform: "darwin",
  });
  assert.deepEqual(metadata.readAppReleaseMetadata(), {
    version: "1.2.75",
    buildDefaultTrack: null,
  });
  assert.equal(await metadata.computeUpdateDisabledReasonLive(), "disabled-by-env");

  const fallback = createReleaseMetadata({
    isLabBuild: false,
    app: { getVersion: () => "9.9.9", isPackaged: true },
    env: {},
    platform: "darwin",
  });
  assert.equal(fallback.readAppReleaseMetadata().version, "9.9.9");
  assert.equal(await fallback.computeUpdateDisabledReasonLive(), null);
});


test("call media binding preserves native permission and desktopCapturer authority", async () => {
  const requested: string[] = [];
  const port = createDesktopCallMediaPort({
    systemPreferences: {
      getMediaAccessStatus: (kind) => kind === "microphone" ? "granted" : "not-determined",
      askForMediaAccess: async (kind) => { requested.push(kind); return kind === "camera"; },
    },
    desktopCapturer: {
      async getSources(options) {
        assert.deepEqual(options.types, ["screen", "window"]);
        return [
          { id: "screen:1:0", name: "Display 1", display_id: "1" },
          { id: "window:2:0", name: "Editor" },
          { id: "window:2:0", name: "Duplicate" },
          { id: "", name: "Invalid" },
        ];
      },
    },
  }, "darwin");

  assert.deepEqual(await port.requestPermissions({ audio: true, video: true }), {
    microphone: "granted",
    camera: "granted",
  });
  assert.deepEqual(requested, ["camera"]);
  assert.deepEqual(await port.listDisplaySources(), [
    { id: "screen:1:0", name: "Display 1", displayId: "1" },
    { id: "window:2:0", name: "Editor" },
  ]);

  const chromiumPrompt = createDesktopCallMediaPort({}, "linux");
  assert.deepEqual(await chromiumPrompt.requestPermissions({ audio: true, video: false }), {
    microphone: "prompt",
    camera: "not-requested",
  });
  await assert.rejects(() => chromiumPrompt.listDisplaySources(), /desktopCapturer is unavailable/);
});


test("MAS packaging declares camera and microphone authority for Human calls", () => {
  const desktopRoot = basename(process.cwd()) === "desktop" ? process.cwd() : join(process.cwd(), "desktop");
  const entitlements = readFileSync(join(desktopRoot, "resources/mas/entitlements.mas.plist"), "utf8");
  assert.match(entitlements, /com\.apple\.security\.device\.microphone/);
  assert.match(entitlements, /com\.apple\.security\.device\.camera/);
  const desktopPackage = JSON.parse(readFileSync(join(desktopRoot, "package.json"), "utf8")) as {
    build?: { mas?: { extendInfo?: Record<string, unknown> } };
  };
  const extendInfo = desktopPackage.build?.mas?.extendInfo ?? {};
  assert.equal(typeof extendInfo.NSMicrophoneUsageDescription, "string");
  assert.equal(typeof extendInfo.NSCameraUsageDescription, "string");
});


test("OS notification manager retires only the exact canonical scope", () => {
  type FakeNotification = {
    readonly closeCount: { value: number };
    click(): void;
  };
  const created: FakeNotification[] = [];
  let focusCount = 0;
  let activationCount = 0;
  const manager = new SandOsNotificationManager({
    getWindow: () => ({
      isFocused: () => false,
      isMinimized: () => false,
      restore: () => undefined,
      show: () => undefined,
      focus: () => { focusCount += 1; },
    }),
    isSupported: () => true,
    createNotification: () => {
      let click: (() => void) | undefined;
      let close: (() => void) | undefined;
      const closeCount = { value: 0 };
      const fake: FakeNotification = {
        closeCount,
        click: () => click?.(),
      };
      created.push(fake);
      return {
        on: (_event, listener) => { click = listener; },
        once: (_event, listener) => { close = listener; },
        show: () => undefined,
        close: () => {
          closeCount.value += 1;
          close?.();
        },
      };
    },
    openAgent: () => undefined,
  });

  const topicScope = { kind: "conversation", conversationId: "conversation-1", childId: "topic-7" } as const;
  const savedScope = { kind: "conversation", conversationId: "conversation-1", childId: "saved-9" } as const;
  assert.equal(manager.showScoped({
    scope: topicScope,
    title: "Topic",
    body: "Message",
    silent: false,
    urgency: "normal",
    onActivate: () => { activationCount += 1; },
  }), true);
  assert.equal(manager.showScoped({
    scope: savedScope,
    title: "Saved",
    body: "Message",
    silent: false,
    urgency: "normal",
    onActivate: () => { activationCount += 1; },
  }), true);

  manager.clearScope(topicScope);
  assert.equal(created[0]?.closeCount.value, 1);
  assert.equal(created[1]?.closeCount.value, 0);

  created[1]?.click();
  assert.equal(focusCount, 1);
  assert.equal(activationCount, 1);

  manager.reset();
  assert.equal(created[1]?.closeCount.value, 1);
});
