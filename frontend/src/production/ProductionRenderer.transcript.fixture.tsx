import type { CoordinatorPortBridge, CursorAuthStatus, DesktopBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
import { ProductionRenderer } from "./ProductionRenderer";
import { mountProductionRenderer } from "./bootstrap";
import { createCoordinatorPortBroker } from "../../../source/electron-preload/coordinator-port-bridge";

// NMR-26a: only the external desktop/service boundary is controlled. The real
// renderer, Coordinator client, transcript feed, projection, cache and controller
// remain mounted. No fixture copy of their merge or invalidation rules exists.
type Frame = Record<string, unknown>;
type Request = { id: number; port: number; requestId: string; method: string; args: Record<string, unknown>; account: string; settled: boolean };
const requests: Request[] = [];
const unexpected: string[] = [];
const pending = new Map<number, { resolve(value: unknown): void; reject(reason: string): void }>();
let account: CursorAuthStatus = { kind: "logged-in", authId: "fixture-a", email: "fixture-a@example.invalid", displayName: "Fixture A" };
let sequence = 0;
let portSequence = 0;
const ports = new Set<FixturePort>();
function signal<T>() {
  const listeners = new Set<(value: T) => void>();
  return {
    subscribe(listener: (value: T) => void) { listeners.add(listener); return () => { listeners.delete(listener); }; },
    emit(value: T) { for (const listener of [...listeners]) listener(value); },
  };
}
const auth = signal<CursorAuthStatus>();
const focus = signal<unknown>();
const themeEvents = signal<{ preference: "light"; resolved: "light" }>();
const emptyEvent = () => () => {};
const noop = () => {};
const done = async () => {};
function forbidden(name: string): never {
  unexpected.push(`desktop:${name}`);
  throw new Error(`Unexpected desktop operation: ${name}`);
}
// Match the Host's persisted participantIds contract: exactly two distinct
// string identities, with the currently authenticated local account first.
function humanConversationsForAccount(accountId: string) {
  return [
    { kind: "human", id: "human:alpha", title: "Alpha", participantIds: [accountId, "peer-alpha"], updatedAt: 2 },
    { kind: "human", id: "human:beta", title: "Beta", participantIds: [accountId, "peer-beta"], updatedAt: 1 },
  ];
}
const controlled = new Set(["getHumanConversationTranscriptTail", "getHumanConversationTranscript", "searchHumanMessages"]);
const fixedReplies: Record<string, unknown> = {
  listAgents: [], searchAgents: [], countAgents: 0,
  syncHumanConversation: null, syncHumanCalls: null, listCallSessions: [],
  listAllAutomations: [], getAgentAutomations: [], getAgentWorkflows: [],
  getTrays: [], getListenerIntegrations: { integrations: [] }, getAsyncTasks: [],
  isAgentNetworkEnabled: false, isGlobalSearchEnabled: false,
  getForeverBoxStatus: null, getSubagents: [], isEgressTunnelAvailable: false,
};
class FixturePort implements TransferredCoordinatorPort {
  readonly id = ++portSequence;
  readonly listeners = new Set<(event: { data: unknown }) => void>();
  closed = false;
  start() {}
  close() { this.closed = true; }
  addEventListener(type: "message" | "close", listener: (event: any) => void) {
    if (type === "message") this.listeners.add(listener);
  }
  receive(frame: unknown) {
    // Also deliver controlled late replies to the original endpoint. The real
    // disposed/generation-fenced consumer, not the fixture, must reject them.
    for (const listener of [...this.listeners]) listener({ data: frame });
  }
  postMessage(value: unknown) {
    const frame = value as Frame;
    if (frame.kind === "lifecycle") {
      if (frame.phase === "hello") queueMicrotask(() => this.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 }));
      return;
    }
    if (frame.kind !== "request" || typeof frame.requestId !== "string" || typeof frame.method !== "string") {
      unexpected.push(`frame:${JSON.stringify(frame)}`);
      return;
    }
    const request: Request = {
      id: ++sequence, port: this.id, requestId: frame.requestId, method: frame.method,
      args: frame.args as Record<string, unknown>, account: account.kind === "logged-in" ? account.authId! : "signed-out", settled: false,
    };
    requests.push(request);
    const respond = (outcome: unknown) => {
      request.settled = true;
      this.receive({ kind: "reply", requestId: request.requestId, outcome });
    };
    if (controlled.has(request.method)) {
      const response = new Promise<unknown>((resolve, reject) => pending.set(request.id, { resolve, reject }));
      void response.then(
        (value) => respond({ status: "ok", value }),
        (message) => respond({ status: "failed", failure: { code: "fixture/read-failed", message: String(message) } }),
      );
    } else if (request.method === "listHumanConversations" || request.method === "syncHumanConversations") {
      queueMicrotask(() => respond({ status: "ok", value: humanConversationsForAccount(request.account) }));
    } else if (Object.prototype.hasOwnProperty.call(fixedReplies, request.method)) {
      queueMicrotask(() => respond({ status: "ok", value: fixedReplies[request.method] }));
    } else {
      unexpected.push(`rpc:${request.method}`);
      queueMicrotask(() => respond({ status: "failed", failure: { code: "fixture/unexpected-rpc", message: request.method } }));
    }
  }
}
const broker = createCoordinatorPortBroker<TransferredCoordinatorPort>({
  invokeRequest() {
    const port = new FixturePort();
    ports.add(port);
    queueMicrotask(() => broker.deliver(port));
  },
});
const coordinatorPort: CoordinatorPortBridge = broker.bridge;
const storage = new Map<string, string>();
const theme = { preference: "light", resolved: "light" } as const;
const update = {
  state: { type: "disabled", reason: "not-packaged" }, currentVersion: "0.0.0-fixture",
  currentTrack: "stable", trackOverride: null, buildDefaultTrack: "stable", availableTracks: ["stable"],
  isTrackManagedByPolicy: false, isBelowMinimumVersion: false,
  autoUpdateWhenIdleOptIn: false, autoUpdateWhenIdleGateEnabled: false,
};
// Explicit inert bootstrap DTOs. Unsupported product actions throw and are
// reported by every test; there is deliberately no permissive Proxy fallback.
const bridge = {
  platform: "linux", isDev: false,
  resolveAttachmentMedia: async () => null, readAttachmentText: async () => null,
  readAttachmentBytes: async () => null, downloadAttachment: async () => false,
  getLinkMetadata: async () => null,
  openExternal: () => forbidden("openExternal"), openCloudAgent: () => forbidden("openCloudAgent"),
  stageAttachmentBytes: () => forbidden("stageAttachmentBytes"), commitStagedAttachments: () => forbidden("commitStagedAttachments"), discardStagedAttachment: done,
  mcp: {
    list: async () => ({ servers: [] }), catalog: async () => [], effectivePlugins: async () => [],
    teamPopularity: async () => ({}), pluginLogo: async () => null, onAuthCompleted: emptyEvent,
    install: () => forbidden("mcp.install"), updatePluginInstall: () => forbidden("mcp.updatePluginInstall"),
    remove: () => forbidden("mcp.remove"), uninstallPlugin: () => forbidden("mcp.uninstallPlugin"),
    authenticate: () => forbidden("mcp.authenticate"), renameAccount: () => forbidden("mcp.renameAccount"),
    removeAccount: () => forbidden("mcp.removeAccount"), setCustomInstructions: () => forbidden("mcp.setCustomInstructions"),
    listServerTools: () => forbidden("mcp.listServerTools"), toggleToolDisabled: () => forbidden("mcp.toggleToolDisabled"),
  },
  callMedia: {
    getPreferences: async () => ({ microphoneId: null, cameraId: null }),
    setPreferences: () => forbidden("callMedia.setPreferences"), requestPermissions: () => forbidden("callMedia.requestPermissions"), listDisplaySources: () => forbidden("callMedia.listDisplaySources"),
  },
  forceGatewayReconnect: done, pickAvatarSource: () => forbidden("pickAvatarSource"), pickAvatarFile: () => forbidden("pickAvatarFile"), generateAgentAvatarImage: () => forbidden("generateAgentAvatarImage"),
  onFocusAgent: focus.subscribe, onDeepLink: emptyEvent, deepLinksReady: done,
  getBoxMigrationStatus: async () => null, onBoxMigration: emptyEvent, onDevBoxRebuild: emptyEvent,
  onOpenFeedback: emptyEvent, onOpenAbout: emptyEvent, onWidgetGallery: emptyEvent, onForceOnboarding: emptyEvent,
  submitFeedback: () => forbidden("submitFeedback"), transcribeAudio: () => forbidden("transcribeAudio"),
  cursorAccount: {
    getStatus: async () => account, onStatusChanged: auth.subscribe,
    getSandAccessFresh: async () => ({ state: "granted", reason: "none" }), getSandAccess: async () => ({ state: "granted", reason: "none" }),
    getAvatar: async () => null, getWeeklyUsage: async () => null, getUsageSummary: async () => null,
    getPrReviewPreferences: async () => ({}), getPrivacyModeEnabled: async () => false,
    login: () => forbidden("login"), logout: () => forbidden("logout"), cancelLogin: () => forbidden("cancelLogin"),
    updateName: () => forbidden("updateName"), invokeDashboardAction: () => forbidden("invokeDashboardAction"), cancelTrial: () => forbidden("cancelTrial"),
  },
  experiments: { initialSnapshot: {}, getSnapshot: async () => ({}), onChanged: emptyEvent, refresh: done, applyFeatureFlagOverride: done, startRpcTraceWindow: async () => false },
  getWindowState: async () => ({ isFullscreen: false, isMaximized: false }), onWindowStateEvent: emptyEvent,
  getZoomFactor: () => 1, onZoomFactorEvent: emptyEvent,
  windowControls: { minimize: done, toggleMaximize: done, close: done, setTitleBarOverlayTone: done, resizeWidth: async () => 0 },
  foreverBox: {
    forceRecreate: () => forbidden("foreverBox.forceRecreate"), update: () => forbidden("foreverBox.update"), onVncUserPresence: emptyEvent, onDevBoxPullProgress: emptyEvent,
    egressTunnel: { initial: false, initialStatus: null, get: async () => false, set: () => forbidden("egressTunnel.set"), onChanged: emptyEvent, getStatus: async () => null, onStatusChanged: emptyEvent },
    webauthnProxy: { initial: false, get: async () => false, set: () => forbidden("webauthnProxy.set"), onChanged: emptyEvent },
  },
  onboarding: { getSeen: async () => true, setSeen: done, onSkip: emptyEvent },
  telemetry: {
    reportAgentLoad: noop, reportBoxVisibility: noop, reportSendLatency: noop, reportHeapMetrics: noop,
    reportSendAck: noop, reportReactionAck: noop, reportRenderTtfr: noop, reportRenderStream: noop,
    reportAgentsUnreachable: noop, reportAccessBlocked: noop, reportRecoveryAction: noop, reportRebuildLifecycle: noop,
    reportReconciliation: noop, reportVncSession: noop, reportVncLiveness: noop, reportOpenComputer: noop,
    reportUpdatePrompt: noop, reportSigninGate: noop, reportOnboardingStep: noop, reportClientFailure: noop, noteSentryConversation: noop,
  },
  timeZone: { get: async () => ({ detectedTimeZone: "UTC", overrideTimeZone: null }), setOverride: () => forbidden("timeZone.setOverride") },
  networkProxy: { get: () => forbidden("networkProxy.get"), mutate: () => forbidden("networkProxy.mutate") },
  uiPreferences: { get: async () => ({ locale: "en", direction: "ltr", reducedMotion: true, highContrast: false, textScale: 1 }), set: () => forbidden("uiPreferences.set") },
  autoReviewInstructions: { get: async () => ({ isEnabled: false, allowInstructions: [], blockInstructions: [] }), set: () => forbidden("autoReviewInstructions.set") },
  localToolPermission: { get: async () => "ask", ceiling: async () => null, set: () => forbidden("localToolPermission.set"), recordApproval: done, clearApprovals: done },
  theme: { initial: theme, get: async () => theme, set: () => forbidden("theme.set"), onChanged: themeEvents.subscribe },
  secrets: { list: async () => ({ keys: [], isPersistent: false }), reveal: () => forbidden("secrets.reveal"), upsert: () => forbidden("secrets.upsert"), remove: () => forbidden("secrets.remove") },
  agent: {
    getPinnedAgents: async () => [], setPinnedAgents: async (value: unknown) => value,
    getSidebarSections: async () => [], setSidebarSections: async (value: unknown) => value,
    getDefaultModel: async () => null, setDefaultModel: () => forbidden("setDefaultModel"),
    getComputerUseModel: async () => null, setComputerUseModel: () => forbidden("setComputerUseModel"), getAvailableModels: async () => [],
    clientPersistence: {
      read: async (key: string) => storage.get(key) ?? null,
      write: async (key: string, value: string) => { storage.set(key, value); },
      remove: async (key: string) => { storage.delete(key); },
      listKeys: async (prefix: string) => [...storage.keys()].filter((key) => key.startsWith(prefix)),
      migrateFromLocalStorage: async () => false,
    },
  },
  update: { getStatus: async () => update, onStatusEvent: emptyEvent, check: () => forbidden("update.check"), setTrack: () => forbidden("update.setTrack"), quitAndInstall: () => forbidden("update.quitAndInstall"), setAutoUpdateWhenIdleOptIn: () => forbidden("update.setAutoUpdateWhenIdleOptIn") },
  attachProdBox: { getStatus: async () => ({ enabled: false }), setEnabled: () => forbidden("attachProdBox.setEnabled") },
} as unknown as DesktopBridge;
function emit(family: string, payload: unknown) {
  for (const port of ports) if (!port.closed) port.receive({ kind: "event", family, payload });
}
function settle(id: number, value: unknown, failed: boolean) {
  const request = pending.get(id);
  if (request == null) throw new Error(`Unknown pending request ${id}`);
  pending.delete(id);
  if (failed) request.reject(String(value)); else request.resolve(value);
}
// Disable browser-native scroll anchoring in this fixture so the viewport
// test observes the production pagination owner's layout commit, not a UA fixup.
const style = document.createElement("style");
style.textContent = ".sand-virtual-transcript { overflow-anchor: none; }";
document.head.append(style);
const mount = document.getElementById("root")!;
const root = mountProductionRenderer(mount, <ProductionRenderer bridge={bridge} coordinatorPort={coordinatorPort} />);
let mounted = true;
Object.assign(window, {
  __transcriptHarness: {
    requests: () => requests.map((request) => ({ ...request })),
    unexpected: () => [...unexpected],
    ports: () => [...ports].map((port) => ({ id: port.id, closed: port.closed })),
    resolve: (id: number, value: unknown) => settle(id, value, false),
    reject: (id: number, reason: string) => settle(id, reason, true),
    event: emit,
    switchAccount: (id: string) => {
      account = { kind: "logged-in", authId: id, email: `${id}@example.invalid`, displayName: id };
      auth.emit(account);
    },
    unmount: () => { if (mounted) { mounted = false; root.unmount(); } },
  },
});
