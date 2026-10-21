import { createHash, randomUUID } from "node:crypto";
import { mkdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import { inspectUnpackedBrowserExtension, type BrowserExtensionInspection } from "./browser-extension-policy.js";
import { basename, dirname, join } from "node:path";
import {
  LOCAL_BROWSER_AGENT_LIMITS,
  parseLocalBrowserAgentAction,
  type LocalBrowserAgentActionResult,
  type LocalBrowserAgentDialog,
  type LocalBrowserAgentEvent,
  type LocalBrowserStructuredSnapshot,
} from "../shared/local-browser-agent.js";
import {
  captureLocalBrowserScreenshotBase64,
  captureLocalBrowserSnapshot,
  clickLocalBrowserRef,
  fillLocalBrowserRef,
  dragLocalBrowserRef,
  localBrowserPageContainsText,
  localBrowserRefStillMatches,
  pressLocalBrowserKey,
  scrollLocalBrowserPage,
  selectLocalBrowserOptions,
  uploadLocalBrowserFile,
  type LocalBrowserElementLocator,
  type LocalBrowserFramePort,
} from "./local-browser-agent-page.js";
import {
  BROWSER_WORKSPACE_INVOKE_CHANNEL,
  BROWSER_WORKSPACE_LIMITS,
  BROWSER_WORKSPACE_STATE_CHANNEL,
  normalizeBrowserWorkspaceAddress,
  parseBrowserWorkspaceCommand,
  type BrowserDownloadState,
  type BrowserWorkspaceBookmark,
  type BrowserWorkspaceBounds,
  type BrowserWorkspaceCommand,
  type BrowserWorkspaceDownload,
  type BrowserWorkspaceHistoryEntry,
  type BrowserWorkspaceExtension,
  type BrowserWorkspaceSnapshot,
  type BrowserWorkspaceTabSnapshot,
} from "../shared/browser-workspace.js";

const PERSISTENCE_VERSION = 1;
const MAX_PERSISTED_BYTES = 4 * 1024 * 1024;
const DEFAULT_TITLE = "New Tab";
const MAX_CONCURRENT_AGENT_TASKS = 4;

export interface BrowserWorkspaceDownloadItem {
  getURL(): string;
  getFilename(): string;
  getTotalBytes(): number;
  getReceivedBytes(): number;
  setSavePath(path: string): void;
  cancel(): void;
  on(event: "updated" | "done", listener: (_event?: unknown, state?: string) => void): void;
}

export interface BrowserWorkspaceLoadedExtension {
  readonly id: string;
  readonly name: string;
  readonly version: string;
}

export interface BrowserWorkspaceSessionPort {
  setPermissionRequestHandler(handler: ((webContents: BrowserWorkspaceWebContentsPort, permission: string, callback: (allowed: boolean) => void) => void) | null): void;
  setPermissionCheckHandler(handler: ((webContents: BrowserWorkspaceWebContentsPort, permission: string) => boolean) | null): void;
  on(event: "will-download", listener: (event: unknown, item: BrowserWorkspaceDownloadItem, webContents: BrowserWorkspaceWebContentsPort) => void): void;
  readonly extensions?: {
    loadExtension(path: string, options?: { readonly allowFileAccess?: boolean }): Promise<BrowserWorkspaceLoadedExtension>;
    removeExtension(extensionId: string): void;
  };
}

export type BrowserWorkspaceDebuggerMessageListener = (
  event: unknown,
  method: string,
  params: unknown,
  sessionId?: string,
) => void;

export interface BrowserWorkspaceDebuggerPort {
  isAttached(): boolean;
  attach(protocolVersion?: string): void;
  detach(): void;
  sendCommand(method: string, commandParams?: Record<string, unknown>): Promise<unknown>;
  on(event: "message", listener: BrowserWorkspaceDebuggerMessageListener): void;
  removeListener?(event: "message", listener: BrowserWorkspaceDebuggerMessageListener): void;
}

export interface BrowserWorkspaceWebContentsPort {
  readonly session: BrowserWorkspaceSessionPort;
  readonly debugger?: BrowserWorkspaceDebuggerPort;
  readonly mainFrame?: LocalBrowserFramePort;
  loadURL(url: string): Promise<unknown>;
  getURL(): string;
  getTitle(): string;
  canGoBack(): boolean;
  canGoForward(): boolean;
  goBack(): void;
  goForward(): void;
  reload(): void;
  stop(): void;
  getZoomFactor(): number;
  setZoomFactor(factor: number): void;
  executeJavaScript(script: string, userGesture?: boolean): Promise<unknown>;
  capturePage(): Promise<{ toPNG(): Buffer }>;
  sendInputEvent(event: { readonly type: "keyDown" | "keyUp" | "char" | "mouseMove" | "mouseDown" | "mouseUp"; readonly keyCode?: string; readonly x?: number; readonly y?: number; readonly button?: "left"; readonly clickCount?: number }): void;
  isDestroyed(): boolean;
  destroy(): void;
  setWindowOpenHandler(handler: (details: { readonly url: string }) => { action: "deny" }): void;
  on(event: string, listener: (...args: any[]) => void): void;
}

export interface BrowserWorkspaceViewPort {
  readonly webContents: BrowserWorkspaceWebContentsPort;
  setBounds(bounds: BrowserWorkspaceBounds): void;
}

export interface BrowserWorkspaceHostWindowPort {
  readonly contentView: {
    addChildView(view: BrowserWorkspaceViewPort): void;
    removeChildView(view: BrowserWorkspaceViewPort): void;
  };
  isDestroyed(): boolean;
  on?(event: "resize" | "closed", listener: () => void): void;
}

export interface BrowserWorkspaceIpcPort {
  handle(channel: string, listener: (event: unknown, request?: unknown) => unknown): void;
  removeHandler?(channel: string): void;
}

export interface BrowserWorkspacePersistencePort {
  load(): unknown;
  save(value: unknown): void;
  prepareDownloadPath(accountHash: string, downloadId: string, fileName: string): string;
  finalizeDownload(path: string, maxBytes: number): { readonly sizeBytes: number; readonly sha256: string };
}

export interface BrowserWorkspaceServicePorts {
  readonly ipcMain: BrowserWorkspaceIpcPort;
  readonly createView: (options: { readonly partition: string }) => BrowserWorkspaceViewPort;
  readonly getTrustedContents: () => BrowserWorkspaceWebContentsPort | undefined;
  readonly getAccountScope: () => string | undefined;
  readonly persistence: BrowserWorkspacePersistencePort;
  readonly pickUploadFile?: () => Promise<string | null>;
  readonly pickExtensionDirectory?: () => Promise<string | null>;
  readonly broadcast: (channel: string, payload: unknown) => void;
  readonly now?: () => number;
  readonly id?: () => string;
  readonly reportFailure?: (leg: string, error: unknown) => void;
}

export interface BrowserWorkspaceService {
  attachWindow(window: BrowserWorkspaceHostWindowPort): void;
  revokeAccount(): void;
  snapshot(conversationId: string): BrowserWorkspaceSnapshot | null;
  executeAgentAction(request: unknown): Promise<LocalBrowserAgentActionResult>;
  dispose(): void;
}

interface PersistedTab { readonly url: string }
interface PersistedWorkspace {
  readonly activeTabIndex: number;
  readonly tabs: readonly PersistedTab[];
}
interface PersistedExtension {
  key: string;
  sourcePath: string;
  integrity: string;
  name: string;
  version: string;
  enabled: boolean;
  permissions: readonly string[];
  hostPermissions: readonly string[];
}
interface PersistedAccount {
  readonly bookmarks: readonly BrowserWorkspaceBookmark[];
  readonly history: readonly BrowserWorkspaceHistoryEntry[];
  readonly extensions: readonly PersistedExtension[];
  readonly workspaces: Readonly<Record<string, PersistedWorkspace>>;
}
interface PersistedRoot {
  readonly version: 1;
  readonly accounts: Readonly<Record<string, PersistedAccount>>;
}

interface MutableExtension extends PersistedExtension {
  runtimeId: string | null;
  error: string | null;
}
interface MutableAccount {
  bookmarks: BrowserWorkspaceBookmark[];
  history: BrowserWorkspaceHistoryEntry[];
  extensions: MutableExtension[];
  workspaces: Record<string, PersistedWorkspace>;
}

interface RuntimeTab {
  readonly id: string;
  readonly view: BrowserWorkspaceViewPort;
  title: string;
  url: string;
  faviconUrl: string | null;
  loading: boolean;
  generation: number;
  crashed: boolean;
  error: { code: string; description: string } | null;
  downloadSequence: number;
  lastDownloadUrl: string | null;
  pendingAgentDownloadOperationId: string | null;
  agentRefs: Map<string, LocalBrowserElementLocator>;
  pendingDialog: LocalBrowserAgentDialog | null;
  events: LocalBrowserAgentEvent[];
  eventSequence: number;
  activeNetworkRequests: Set<string>;
  lastNetworkActivityAt: number;
  networkIdleToken: number;
  networkIdleEmittedToken: number;
  networkIdleTimer: ReturnType<typeof setTimeout> | null;
  debuggerListener: BrowserWorkspaceDebuggerMessageListener | null;
  debuggerReady: boolean;
  debuggerOwned: boolean;
}

interface RuntimeWorkspace {
  readonly id: string;
  readonly accountHash: string;
  readonly conversationId: string;
  readonly conversationHash: string;
  readonly partition: string;
  tabs: RuntimeTab[];
  activeTabId: string;
  bounds: BrowserWorkspaceBounds;
  visible: boolean;
  downloads: BrowserWorkspaceDownload[];
  agentControlState: "available" | "paused" | "taken-over" | "stopped";
  agentControlEpoch: number;
  agentTaskId: string;
  agentTaskStatus: "idle" | "running" | "paused" | "taken-over" | "stopped";
  activeAgentOperationId: string | null;
  extensionCandidate: { readonly candidateId: string; readonly inspection: BrowserExtensionInspection } | null;
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? value as Record<string, unknown> : null;
}

function boundedString(value: unknown, max: number): string | null {
  return typeof value === "string" && value.length > 0 && value.length <= max ? value : null;
}

function normalizePersistedRoot(value: unknown): PersistedRoot {
  const root = record(value);
  if (root?.version !== PERSISTENCE_VERSION) return { version: PERSISTENCE_VERSION, accounts: {} };
  const rawAccounts = record(root.accounts);
  if (rawAccounts == null) return { version: PERSISTENCE_VERSION, accounts: {} };
  const accounts: Record<string, PersistedAccount> = {};
  for (const [accountHash, raw] of Object.entries(rawAccounts)) {
    if (!/^[a-f0-9]{64}$/.test(accountHash)) continue;
    const account = record(raw);
    if (account == null) continue;
    const bookmarks = Array.isArray(account.bookmarks) ? account.bookmarks.flatMap((item): BrowserWorkspaceBookmark[] => {
      const entry = record(item);
      const id = boundedString(entry?.id, 128), title = boundedString(entry?.title, 512), url = boundedString(entry?.url, BROWSER_WORKSPACE_LIMITS.maxAddressLength);
      const createdAtMs = entry?.createdAtMs;
      return id != null && title != null && url != null && normalizeBrowserWorkspaceAddress(url) != null && typeof createdAtMs === "number" && Number.isFinite(createdAtMs)
        ? [{ id, title, url, createdAtMs }] : [];
    }).slice(-BROWSER_WORKSPACE_LIMITS.maxBookmarks) : [];
    const history = Array.isArray(account.history) ? account.history.flatMap((item): BrowserWorkspaceHistoryEntry[] => {
      const entry = record(item);
      const id = boundedString(entry?.id, 128), title = boundedString(entry?.title, 512), url = boundedString(entry?.url, BROWSER_WORKSPACE_LIMITS.maxAddressLength);
      const visitedAtMs = entry?.visitedAtMs;
      return id != null && title != null && url != null && normalizeBrowserWorkspaceAddress(url) != null && typeof visitedAtMs === "number" && Number.isFinite(visitedAtMs)
        ? [{ id, title, url, visitedAtMs }] : [];
    }).slice(-BROWSER_WORKSPACE_LIMITS.maxHistory) : [];
    const extensions = Array.isArray(account.extensions) ? account.extensions.flatMap((item): PersistedExtension[] => {
      const entry = record(item);
      const key = boundedString(entry?.key, 128);
      const sourcePath = boundedString(entry?.sourcePath, 8192);
      const integrity = boundedString(entry?.integrity, 128);
      const name = boundedString(entry?.name, 256);
      const version = boundedString(entry?.version, 64);
      const enabled = entry?.enabled;
      const permissions = Array.isArray(entry?.permissions) ? entry.permissions.filter((value): value is string => typeof value === "string" && value.length > 0 && value.length <= 512).slice(0, 100) : [];
      const hostPermissions = Array.isArray(entry?.hostPermissions) ? entry.hostPermissions.filter((value): value is string => typeof value === "string" && value.length > 0 && value.length <= 512).slice(0, 100) : [];
      return key != null && sourcePath != null && integrity?.startsWith("sha256:") && name != null && version != null && typeof enabled === "boolean"
        ? [{ key, sourcePath, integrity, name, version, enabled, permissions, hostPermissions }]
        : [];
    }).slice(0, 64) : [];
    const workspaces: Record<string, PersistedWorkspace> = {};
    const rawWorkspaces = record(account.workspaces);
    if (rawWorkspaces != null) {
      for (const [conversationHash, rawWorkspace] of Object.entries(rawWorkspaces)) {
        if (!/^[a-f0-9]{64}$/.test(conversationHash)) continue;
        const workspace = record(rawWorkspace);
        if (workspace == null || !Array.isArray(workspace.tabs)) continue;
        const tabs = workspace.tabs.flatMap((item): PersistedTab[] => {
          const tab = record(item);
          const url = typeof tab?.url === "string" ? normalizeBrowserWorkspaceAddress(tab.url) : null;
          return url == null ? [] : [{ url }];
        }).slice(0, BROWSER_WORKSPACE_LIMITS.maxTabs);
        const activeTabIndex = typeof workspace.activeTabIndex === "number" && Number.isSafeInteger(workspace.activeTabIndex)
          ? Math.max(0, Math.min(tabs.length - 1, workspace.activeTabIndex))
          : 0;
        workspaces[conversationHash] = { activeTabIndex, tabs };
      }
    }
    accounts[accountHash] = { bookmarks, history, extensions, workspaces };
  }
  return { version: PERSISTENCE_VERSION, accounts };
}

function hashIdentity(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function safeDownloadFilename(value: string): string {
  const base = value.replace(/[\\/\0<>:"|?*]/g, "_").trim().replace(/^\.+/, "").slice(0, 180);
  return base.length > 0 ? base : "download";
}

export function createBrowserWorkspaceFilePersistence(rootDir: string): BrowserWorkspacePersistencePort {
  const statePath = join(rootDir, "state-v1.json");
  return {
    load() {
      try {
        if (statSync(statePath).size > MAX_PERSISTED_BYTES) return null;
        return JSON.parse(readFileSync(statePath, "utf8")) as unknown;
      } catch {
        return null;
      }
    },
    save(value) {
      mkdirSync(dirname(statePath), { recursive: true });
      const encoded = JSON.stringify(value);
      if (Buffer.byteLength(encoded, "utf8") > MAX_PERSISTED_BYTES) throw new Error("BROWSER_STATE_TOO_LARGE");
      const temporary = `${statePath}.tmp-${process.pid}`;
      writeFileSync(temporary, encoded, { encoding: "utf8", mode: 0o600 });
      renameSync(temporary, statePath);
    },
    prepareDownloadPath(accountHash, downloadId, fileName) {
      const directory = join(rootDir, "downloads", accountHash);
      mkdirSync(directory, { recursive: true });
      return join(directory, `${downloadId}-${safeDownloadFilename(fileName)}`);
    },
    finalizeDownload(path, maxBytes) {
      const info = statSync(path);
      if (!info.isFile() || info.size < 0 || info.size > maxBytes) throw new Error("BROWSER_DOWNLOAD_SIZE_INVALID");
      const bytes = readFileSync(path);
      if (bytes.byteLength !== info.size || bytes.byteLength > maxBytes) throw new Error("BROWSER_DOWNLOAD_SIZE_CHANGED");
      return { sizeBytes: bytes.byteLength, sha256: createHash("sha256").update(bytes).digest("hex") };
    },
  };
}

function downloadState(value: string | undefined): BrowserDownloadState {
  return value === "cancelled" || value === "interrupted" || value === "completed" ? value : "progressing";
}

function isExpectedDownloadNavigationError(error: unknown): boolean {
  const value = record(error);
  return value?.code === "ERR_FAILED"
    || (error instanceof Error && error.message.includes("ERR_FAILED"));
}

export function createBrowserWorkspaceService(ports: BrowserWorkspaceServicePorts): BrowserWorkspaceService {
  const now = ports.now ?? Date.now;
  const nextId = ports.id ?? randomUUID;
  let persisted = normalizePersistedRoot(ports.persistence.load());
  const accounts = new Map<string, MutableAccount>();
  const workspaces = new Map<string, RuntimeWorkspace>();
  const agentQueues = new Map<string, Promise<void>>();
  const sessionConfigured = new WeakSet<object>();
  const activeAgentDownloads = new Map<string, { readonly workspaceId: string; readonly item: BrowserWorkspaceDownloadItem }>();
  let hostWindow: BrowserWorkspaceHostWindowPort | null = null;
  let activeWorkspaceId: string | null = null;
  let mountedView: BrowserWorkspaceViewPort | null = null;
  let disposed = false;

  const fail = (leg: string, error: unknown): void => {
    ports.reportFailure?.(leg, error);
  };

  const safeBrowserOrigin = (value: unknown): string | null => {
    if (typeof value !== "string") return null;
    try {
      const parsed = new URL(value);
      return parsed.protocol === "http:" || parsed.protocol === "https:" ? parsed.origin : null;
    } catch {
      return null;
    }
  };

  const appendAgentEvent = (
    tab: RuntimeTab,
    type: LocalBrowserAgentEvent["type"],
  ): LocalBrowserAgentEvent => {
    let currentUrl = tab.url;
    try { currentUrl = tab.view.webContents.getURL() || currentUrl; } catch {}
    const event: LocalBrowserAgentEvent = {
      sequence: ++tab.eventSequence,
      type,
      atMs: now(),
      urlOrigin: safeBrowserOrigin(currentUrl),
    };
    tab.events.push(event);
    if (tab.events.length > LOCAL_BROWSER_AGENT_LIMITS.maxEvents) {
      tab.events.splice(0, tab.events.length - LOCAL_BROWSER_AGENT_LIMITS.maxEvents);
    }
    return event;
  };

  const scheduleNetworkIdle = (tab: RuntimeTab): void => {
    if (tab.networkIdleTimer != null) clearTimeout(tab.networkIdleTimer);
    tab.networkIdleTimer = null;
    if (!tab.debuggerReady || tab.activeNetworkRequests.size !== 0) return;
    const token = tab.networkIdleToken;
    const delay = Math.max(0, LOCAL_BROWSER_AGENT_LIMITS.networkIdleMs - (now() - tab.lastNetworkActivityAt));
    tab.networkIdleTimer = setTimeout(() => {
      tab.networkIdleTimer = null;
      if (disposed || tab.view.webContents.isDestroyed() || !tab.debuggerReady) return;
      if (tab.activeNetworkRequests.size !== 0 || tab.networkIdleToken !== token) return;
      if (tab.networkIdleEmittedToken === token) return;
      tab.networkIdleEmittedToken = token;
      appendAgentEvent(tab, "network-idle");
    }, delay);
  };

  const disposeTabDebugger = (tab: RuntimeTab): void => {
    if (tab.networkIdleTimer != null) clearTimeout(tab.networkIdleTimer);
    tab.networkIdleTimer = null;
    const debuggerPort = tab.view.webContents.debugger;
    if (debuggerPort != null && tab.debuggerListener != null) {
      try { debuggerPort.removeListener?.("message", tab.debuggerListener); } catch {}
    }
    tab.debuggerListener = null;
    if (debuggerPort != null && tab.debuggerOwned) {
      try { if (debuggerPort.isAttached()) debuggerPort.detach(); } catch {}
    }
    tab.debuggerOwned = false;
    tab.debuggerReady = false;
    tab.activeNetworkRequests.clear();
  };

  const configureTabDebugger = async (tab: RuntimeTab): Promise<void> => {
    const debuggerPort = tab.view.webContents.debugger;
    if (debuggerPort == null) return;
    const listener: BrowserWorkspaceDebuggerMessageListener = (_event, method, rawParams) => {
      const params = record(rawParams);
      if (method === "Page.javascriptDialogOpening") {
        const rawType = params?.type;
        const type: LocalBrowserAgentDialog["type"] =
          rawType === "alert" || rawType === "confirm" || rawType === "prompt" || rawType === "beforeunload"
            ? rawType
            : "alert";
        const message = typeof params?.message === "string"
          ? params.message.slice(0, LOCAL_BROWSER_AGENT_LIMITS.maxDialogMessageLength)
          : "";
        const defaultPrompt = typeof params?.defaultPrompt === "string"
          ? params.defaultPrompt.slice(0, LOCAL_BROWSER_AGENT_LIMITS.maxPromptTextLength)
          : undefined;
        tab.pendingDialog = {
          type,
          message,
          urlOrigin: safeBrowserOrigin(params?.url),
          ...(defaultPrompt === undefined ? {} : { defaultPrompt }),
        };
        appendAgentEvent(tab, "dialog-opened");
        return;
      }
      if (method === "Page.javascriptDialogClosed") {
        if (tab.pendingDialog != null) {
          tab.pendingDialog = null;
          appendAgentEvent(tab, "dialog-closed");
        }
        return;
      }
      if (method === "Network.requestWillBeSent") {
        const requestId = boundedString(params?.requestId, 256);
        if (requestId == null) return;
        appendAgentEvent(tab, "network-request");
        const resourceType = params?.type;
        if (resourceType === "WebSocket" || resourceType === "EventSource") return;
        tab.activeNetworkRequests.add(requestId);
        tab.networkIdleToken += 1;
        tab.lastNetworkActivityAt = now();
        if (tab.networkIdleTimer != null) clearTimeout(tab.networkIdleTimer);
        tab.networkIdleTimer = null;
        return;
      }
      if (method === "Network.loadingFinished" || method === "Network.loadingFailed") {
        const requestId = boundedString(params?.requestId, 256);
        if (requestId == null || !tab.activeNetworkRequests.delete(requestId)) return;
        tab.lastNetworkActivityAt = now();
        tab.networkIdleToken += 1;
        appendAgentEvent(tab, method === "Network.loadingFailed" ? "network-failed" : "network-finished");
        scheduleNetworkIdle(tab);
      }
    };
    tab.debuggerListener = listener;
    debuggerPort.on("message", listener);
    try {
      if (!debuggerPort.isAttached()) {
        debuggerPort.attach("1.3");
        tab.debuggerOwned = true;
      }
      await debuggerPort.sendCommand("Page.enable");
      await debuggerPort.sendCommand("Network.enable");
      tab.debuggerReady = true;
    } catch (error) {
      tab.debuggerReady = false;
      fail("browser-debugger", error);
      try { debuggerPort.removeListener?.("message", listener); } catch {}
      tab.debuggerListener = null;
      if (tab.debuggerOwned) {
        try { if (debuggerPort.isAttached()) debuggerPort.detach(); } catch {}
        tab.debuggerOwned = false;
      }
    }
  };

  const persist = (): void => {
    const nextAccounts: Record<string, PersistedAccount> = { ...persisted.accounts };
    for (const [accountHash, account] of accounts) {
      nextAccounts[accountHash] = {
        bookmarks: account.bookmarks,
        history: account.history,
        extensions: account.extensions.map(({ runtimeId: _runtimeId, error: _error, ...extension }) => extension),
        workspaces: account.workspaces,
      };
    }
    persisted = { version: PERSISTENCE_VERSION, accounts: nextAccounts };
    try { ports.persistence.save(persisted); } catch (error) { fail("persistence", error); }
  };

  const accountFor = (accountHash: string): MutableAccount => {
    const existing = accounts.get(accountHash);
    if (existing != null) return existing;
    const saved = persisted.accounts[accountHash];
    const created: MutableAccount = {
      bookmarks: [...(saved?.bookmarks ?? [])],
      history: [...(saved?.history ?? [])],
      extensions: (saved?.extensions ?? []).map((extension) => ({ ...extension, runtimeId: null, error: null })),
      workspaces: { ...(saved?.workspaces ?? {}) },
    };
    accounts.set(accountHash, created);
    return created;
  };

  const currentAccount = (): { scope: string; hash: string } => {
    const scope = ports.getAccountScope()?.trim();
    if (scope == null || scope.length === 0 || scope.length > 1024) throw new Error("BROWSER_ACCOUNT_UNAVAILABLE");
    return { scope, hash: hashIdentity(scope) };
  };

  const workspaceIdentity = (accountHash: string, conversationId: string): { id: string; conversationHash: string } => {
    const conversationHash = hashIdentity(conversationId);
    return { id: hashIdentity(`${accountHash}\0${conversationHash}`), conversationHash };
  };

  const tabSnapshot = (tab: RuntimeTab): BrowserWorkspaceTabSnapshot => {
    const contents = tab.view.webContents;
    let canGoBack = false, canGoForward = false, zoomFactor = 1;
    try {
      if (!contents.isDestroyed()) {
        canGoBack = contents.canGoBack();
        canGoForward = contents.canGoForward();
        zoomFactor = contents.getZoomFactor();
      }
    } catch {}
    return {
      id: tab.id,
      title: tab.title,
      url: tab.url,
      faviconUrl: tab.faviconUrl,
      loading: tab.loading,
      canGoBack,
      canGoForward,
      zoomFactor,
      generation: tab.generation,
      crashed: tab.crashed,
      error: tab.error,
    };
  };

  const snapshot = (workspace: RuntimeWorkspace): BrowserWorkspaceSnapshot => {
    const account = accountFor(workspace.accountHash);
    return {
      workspaceId: workspace.id,
      conversationId: workspace.conversationId,
      visible: workspace.visible,
      activeTabId: workspace.activeTabId,
      tabs: workspace.tabs.map(tabSnapshot),
      bookmarks: account.bookmarks,
      history: account.history,
      downloads: workspace.downloads,
      extensions: account.extensions.map((extension): BrowserWorkspaceExtension => ({
        key: extension.key,
        name: extension.name,
        version: extension.version,
        enabled: extension.enabled,
        permissions: extension.permissions,
        hostPermissions: extension.hostPermissions,
        integrity: extension.integrity,
        error: extension.error,
      })),
      extensionCandidate: workspace.extensionCandidate == null ? null : {
        candidateId: workspace.extensionCandidate.candidateId,
        name: workspace.extensionCandidate.inspection.name,
        version: workspace.extensionCandidate.inspection.version,
        permissions: workspace.extensionCandidate.inspection.permissions,
        hostPermissions: workspace.extensionCandidate.inspection.hostPermissions,
        integrity: workspace.extensionCandidate.inspection.integrity,
      },
      agentControl: {
        state: workspace.agentControlState,
        epoch: workspace.agentControlEpoch,
        taskId: workspace.agentTaskId,
        taskStatus: workspace.agentTaskStatus,
        activeOperationId: workspace.activeAgentOperationId,
      },
    };
  };

  const publish = (workspace: RuntimeWorkspace): BrowserWorkspaceSnapshot => {
    const value = snapshot(workspace);
    ports.broadcast(BROWSER_WORKSPACE_STATE_CHANNEL, value);
    return value;
  };

  const saveWorkspace = (workspace: RuntimeWorkspace): void => {
    const account = accountFor(workspace.accountHash);
    const activeIndex = Math.max(0, workspace.tabs.findIndex((tab) => tab.id === workspace.activeTabId));
    account.workspaces[workspace.conversationHash] = {
      activeTabIndex: activeIndex,
      tabs: workspace.tabs.flatMap((tab): PersistedTab[] => {
        const url = normalizeBrowserWorkspaceAddress(tab.url);
        return url == null ? [] : [{ url }];
      }),
    };
    persist();
  };

  const addHistory = (workspace: RuntimeWorkspace, tab: RuntimeTab, url: string): void => {
    const safeUrl = normalizeBrowserWorkspaceAddress(url);
    if (safeUrl == null) return;
    const account = accountFor(workspace.accountHash);
    const previous = account.history.at(-1);
    if (previous?.url === safeUrl && previous.title === tab.title) return;
    account.history.push({ id: nextId(), title: tab.title || safeUrl, url: safeUrl, visitedAtMs: now() });
    if (account.history.length > BROWSER_WORKSPACE_LIMITS.maxHistory) account.history.splice(0, account.history.length - BROWSER_WORKSPACE_LIMITS.maxHistory);
    persist();
  };

  const unmount = (): void => {
    if (mountedView == null) return;
    try { hostWindow?.contentView.removeChildView(mountedView); } catch (error) { fail("remove-view", error); }
    mountedView = null;
  };

  const activeTab = (workspace: RuntimeWorkspace): RuntimeTab => {
    const tab = workspace.tabs.find((candidate) => candidate.id === workspace.activeTabId);
    if (tab == null) throw new Error("BROWSER_TAB_NOT_FOUND");
    return tab;
  };

  const layout = (workspace: RuntimeWorkspace): void => {
    if (hostWindow == null || hostWindow.isDestroyed() || !workspace.visible || activeWorkspaceId !== workspace.id) {
      unmount();
      return;
    }
    const tab = activeTab(workspace);
    if (tab.view.webContents.isDestroyed()) return;
    if (mountedView !== tab.view) {
      unmount();
      try {
        hostWindow.contentView.addChildView(tab.view);
        mountedView = tab.view;
      } catch (error) {
        fail("add-view", error);
        return;
      }
    }
    try { tab.view.setBounds(workspace.bounds); } catch (error) { fail("bounds", error); }
  };

  const findTabByContents = (contents: BrowserWorkspaceWebContentsPort): { workspace: RuntimeWorkspace; tab: RuntimeTab } | null => {
    for (const workspace of workspaces.values()) {
      const tab = workspace.tabs.find((candidate) => candidate.view.webContents === contents);
      if (tab != null) return { workspace, tab };
    }
    return null;
  };

  const publishAccount = (accountHash: string): void => {
    for (const workspace of workspaces.values()) {
      if (workspace.accountHash === accountHash) publish(workspace);
    }
  };

  const loadExtension = async (
    session: BrowserWorkspaceSessionPort,
    accountHash: string,
    extension: MutableExtension,
    inspection: BrowserExtensionInspection,
  ): Promise<void> => {
    if (session.extensions == null) throw new Error("BROWSER_EXTENSIONS_UNAVAILABLE");
    const loaded = await session.extensions.loadExtension(inspection.sourcePath, { allowFileAccess: false });
    if (loaded.name !== inspection.name || loaded.version !== inspection.version) {
      try { session.extensions.removeExtension(loaded.id); } catch {}
      throw new Error("BROWSER_EXTENSION_LOAD_MISMATCH");
    }
    extension.runtimeId = loaded.id;
    extension.error = null;
    publishAccount(accountHash);
  };

  const unloadExtension = (session: BrowserWorkspaceSessionPort, extension: MutableExtension): void => {
    if (extension.runtimeId != null && session.extensions != null) {
      try { session.extensions.removeExtension(extension.runtimeId); } catch {}
    }
    extension.runtimeId = null;
  };

  const restoreExtensions = async (session: BrowserWorkspaceSessionPort, accountHash: string): Promise<void> => {
    const account = accountFor(accountHash);
    for (const extension of account.extensions) {
      if (!extension.enabled || disposed) continue;
      try {
        const inspection = inspectUnpackedBrowserExtension(extension.sourcePath);
        if (inspection.integrity !== extension.integrity) throw new Error("BROWSER_EXTENSION_INTEGRITY_CHANGED");
        await loadExtension(session, accountHash, extension, inspection);
      } catch (error) {
        extension.runtimeId = null;
        extension.error = error instanceof Error ? error.message.slice(0, 256) : "BROWSER_EXTENSION_LOAD_FAILED";
        fail("extension-restore", error);
        publishAccount(accountHash);
      }
    }
  };

  const configureSession = (session: BrowserWorkspaceSessionPort, accountHash: string): void => {
    if (sessionConfigured.has(session as object)) return;
    sessionConfigured.add(session as object);
    void restoreExtensions(session, accountHash);
    session.setPermissionRequestHandler((_contents, _permission, callback) => callback(false));
    session.setPermissionCheckHandler(() => false);
    session.on("will-download", (_event, item, contents) => {
      const owner = findTabByContents(contents);
      if (owner == null || disposed) {
        try { item.cancel(); } catch {}
        return;
      }
      const { workspace, tab } = owner;
      tab.downloadSequence += 1;
      tab.lastDownloadUrl = item.getURL();
      const id = nextId();
      const agentOperationId = tab.pendingAgentDownloadOperationId;
      tab.pendingAgentDownloadOperationId = null;
      if (agentOperationId != null) activeAgentDownloads.set(id, { workspaceId: workspace.id, item });
      const fileName = safeDownloadFilename(item.getFilename());
      const entry: BrowserWorkspaceDownload = {
        id,
        tabId: tab.id,
        fileName,
        url: item.getURL().slice(0, BROWSER_WORKSPACE_LIMITS.maxAddressLength),
        receivedBytes: Math.max(0, item.getReceivedBytes()),
        totalBytes: Math.max(0, item.getTotalBytes()),
        state: "progressing",
        sha256: null,
        startedAtMs: now(),
      };
      workspace.downloads = [...workspace.downloads.slice(-99), entry];
      let savePath: string;
      try {
        const announcedBytes = Math.max(0, item.getTotalBytes());
        if (announcedBytes > BROWSER_WORKSPACE_LIMITS.maxDownloadBytes) throw new Error("BROWSER_DOWNLOAD_TOO_LARGE");
        savePath = ports.persistence.prepareDownloadPath(workspace.accountHash, id, fileName);
        item.setSavePath(savePath);
      } catch (error) {
        try { item.cancel(); } catch {}
        activeAgentDownloads.delete(id);
        fail("download-path", error);
        workspace.downloads = workspace.downloads.map((download) => download.id === id ? { ...download, state: "cancelled" } : download);
        publish(workspace);
        return;
      }
      const update = (_event?: unknown, state?: string): void => {
        let nextState = downloadState(state);
        let sha256: string | null = null;
        let receivedBytes = Math.max(0, item.getReceivedBytes());
        const totalBytes = Math.max(0, item.getTotalBytes());
        if (receivedBytes > BROWSER_WORKSPACE_LIMITS.maxDownloadBytes || totalBytes > BROWSER_WORKSPACE_LIMITS.maxDownloadBytes) {
          try { item.cancel(); } catch {}
          nextState = "cancelled";
        } else if (nextState === "completed") {
          try {
            const evidence = ports.persistence.finalizeDownload(savePath, BROWSER_WORKSPACE_LIMITS.maxDownloadBytes);
            receivedBytes = evidence.sizeBytes;
            sha256 = evidence.sha256;
          } catch (error) {
            nextState = "interrupted";
            fail("download-evidence", error);
          }
        }
        workspace.downloads = workspace.downloads.map((download) => download.id === id ? { ...download, receivedBytes, totalBytes, state: nextState, sha256 } : download);
        if (nextState !== "progressing") activeAgentDownloads.delete(id);
        publish(workspace);
      };
      item.on("updated", update);
      item.on("done", update);
      publish(workspace);
    });
  };

  const createTab = (workspace: RuntimeWorkspace, requestedAddress?: string, select = true): RuntimeTab => {
    if (workspace.tabs.length >= BROWSER_WORKSPACE_LIMITS.maxTabs) throw new Error("BROWSER_TAB_LIMIT");
    const view = ports.createView({ partition: workspace.partition });
    const contents = view.webContents;
    configureSession(contents.session, workspace.accountHash);
    const tab: RuntimeTab = {
      id: nextId(),
      view,
      title: DEFAULT_TITLE,
      url: "about:blank",
      faviconUrl: null,
      loading: false,
      generation: 0,
      crashed: false,
      error: null,
      downloadSequence: 0,
      lastDownloadUrl: null,
      pendingAgentDownloadOperationId: null,
      agentRefs: new Map(),
      pendingDialog: null,
      events: [],
      eventSequence: 0,
      activeNetworkRequests: new Set(),
      lastNetworkActivityAt: now(),
      networkIdleToken: 0,
      networkIdleEmittedToken: -1,
      networkIdleTimer: null,
      debuggerListener: null,
      debuggerReady: false,
      debuggerOwned: false,
    };
    workspace.tabs.push(tab);
    if (select) workspace.activeTabId = tab.id;

    const updateState = (): void => {
      if (contents.isDestroyed() || disposed) return;
      try {
        tab.url = contents.getURL() || tab.url;
        tab.title = contents.getTitle() || tab.title || DEFAULT_TITLE;
      } catch {}
      if (workspace.visible && activeWorkspaceId === workspace.id && workspace.activeTabId === tab.id) layout(workspace);
      publish(workspace);
    };

    contents.setWindowOpenHandler(({ url }) => {
      const safe = normalizeBrowserWorkspaceAddress(url);
      if (safe != null && !disposed) {
        try {
          createTab(workspace, safe, true);
          saveWorkspace(workspace);
          layout(workspace);
          publish(workspace);
        } catch (error) { fail("popup", error); }
      }
      return { action: "deny" };
    });
    contents.on("will-navigate", (event: { preventDefault(): void }, url: string) => {
      if (normalizeBrowserWorkspaceAddress(url) != null) return;
      event.preventDefault();
      tab.error = { code: "BROWSER_NAVIGATION_BLOCKED", description: "Fabushi blocked a non-HTTP(S) navigation." };
      publish(workspace);
    });
    contents.on("did-start-loading", () => {
      tab.loading = true;
      tab.generation += 1;
      tab.agentRefs.clear();
      tab.crashed = false;
      tab.error = null;
      updateState();
    });
    contents.on("did-stop-loading", () => {
      tab.loading = false;
      updateState();
    });
    contents.on("did-finish-load", () => {
      appendAgentEvent(tab, "load");
      scheduleNetworkIdle(tab);
    });
    contents.on("page-title-updated", (_event: unknown, title: string) => {
      if (typeof title === "string" && title.trim().length > 0) tab.title = title.slice(0, 512);
      updateState();
    });
    contents.on("page-favicon-updated", (_event: unknown, favicons: unknown) => {
      const candidate = Array.isArray(favicons) ? favicons.find((value) => typeof value === "string" && value.length <= 2048 && normalizeBrowserWorkspaceAddress(value) != null) : null;
      tab.faviconUrl = typeof candidate === "string" ? candidate : null;
      publish(workspace);
    });
    const didNavigate = (_event: unknown, url: string) => {
      const safe = normalizeBrowserWorkspaceAddress(url);
      if (safe != null) tab.url = safe;
      appendAgentEvent(tab, "page");
      updateState();
      addHistory(workspace, tab, url);
      saveWorkspace(workspace);
    };
    contents.on("did-navigate", didNavigate);
    contents.on("did-navigate-in-page", didNavigate);
    contents.on("did-fail-load", (_event: unknown, code: number, description: string, _validatedUrl: string, isMainFrame: boolean) => {
      if (isMainFrame === false || code === -3) return;
      tab.loading = false;
      tab.error = { code: `LOAD_${code}`, description: String(description || "Navigation failed").slice(0, 512) };
      updateState();
    });
    contents.on("render-process-gone", (_event: unknown, details: { reason?: string }) => {
      tab.loading = false;
      tab.crashed = true;
      tab.generation += 1;
      tab.error = { code: "RENDERER_GONE", description: String(details?.reason ?? "renderer process exited").slice(0, 256) };
      updateState();
    });
    contents.on("destroyed", () => {
      disposeTabDebugger(tab);
      if (disposed) return;
      tab.loading = false;
      tab.crashed = true;
      tab.generation += 1;
      tab.error = { code: "TAB_DESTROYED", description: "The browser tab was destroyed." };
      publish(workspace);
    });

    const debuggerSetup = configureTabDebugger(tab);
    if (requestedAddress != null) {
      const safe = normalizeBrowserWorkspaceAddress(requestedAddress);
      if (safe == null) {
        workspace.tabs = workspace.tabs.filter((candidate) => candidate !== tab);
        if (!contents.isDestroyed()) contents.destroy();
        throw new Error("BROWSER_NAVIGATION_BLOCKED");
      }
      tab.url = safe;
      void debuggerSetup.then(() => {
        if (contents.isDestroyed() || disposed) return undefined;
        return contents.loadURL(safe);
      }).catch((error: unknown) => {
        tab.loading = false;
        tab.error = { code: "LOAD_REJECTED", description: error instanceof Error ? error.message.slice(0, 512) : "Navigation failed" };
        publish(workspace);
      });
    }
    return tab;
  };

  const ensureWorkspace = (conversationId: string): RuntimeWorkspace => {
    const account = currentAccount();
    const identity = workspaceIdentity(account.hash, conversationId);
    const existing = workspaces.get(identity.id);
    if (existing != null) return existing;
    const saved = accountFor(account.hash).workspaces[identity.conversationHash];
    const workspace: RuntimeWorkspace = {
      id: identity.id,
      accountHash: account.hash,
      conversationId,
      conversationHash: identity.conversationHash,
      partition: `persist:fabushi-browser-${account.hash}`,
      tabs: [],
      activeTabId: "",
      bounds: { x: 0, y: 0, width: 1, height: 1 },
      visible: false,
      downloads: [],
      agentControlState: "available",
      agentControlEpoch: 1,
      agentTaskId: nextId(),
      agentTaskStatus: "idle",
      activeAgentOperationId: null,
      extensionCandidate: null,
    };
    workspaces.set(workspace.id, workspace);
    for (const savedTab of saved?.tabs ?? []) {
      try { createTab(workspace, savedTab.url, false); } catch (error) { fail("restore-tab", error); }
    }
    if (workspace.tabs.length === 0) createTab(workspace, undefined, false);
    const index = Math.max(0, Math.min(workspace.tabs.length - 1, saved?.activeTabIndex ?? 0));
    workspace.activeTabId = workspace.tabs[index]!.id;
    return workspace;
  };

  const requireOwnedWorkspace = (conversationId: string): RuntimeWorkspace => {
    const workspace = ensureWorkspace(conversationId);
    const account = currentAccount();
    if (workspace.accountHash !== account.hash) throw new Error("BROWSER_WORKSPACE_STALE_ACCOUNT");
    return workspace;
  };

  const requireExistingOwnedWorkspace = (conversationId: string): RuntimeWorkspace => {
    const account = currentAccount();
    const identity = workspaceIdentity(account.hash, conversationId);
    const workspace = workspaces.get(identity.id);
    if (workspace == null) throw new Error("BROWSER_WORKSPACE_NOT_OPEN");
    if (workspace.accountHash !== account.hash) throw new Error("BROWSER_WORKSPACE_STALE_ACCOUNT");
    return workspace;
  };

  const requireTab = (workspace: RuntimeWorkspace, tabId: string): RuntimeTab => {
    const tab = workspace.tabs.find((candidate) => candidate.id === tabId);
    if (tab == null || tab.view.webContents.isDestroyed()) throw new Error("BROWSER_TAB_NOT_FOUND");
    return tab;
  };

  const execute = async (command: BrowserWorkspaceCommand): Promise<BrowserWorkspaceSnapshot> => {
    if (disposed) throw new Error("BROWSER_WORKSPACE_DISPOSED");
    const workspace = requireOwnedWorkspace(command.conversationId);
    switch (command.type) {
      case "activate": {
        if (activeWorkspaceId != null && activeWorkspaceId !== workspace.id) {
          const previous = workspaces.get(activeWorkspaceId);
          if (previous != null) previous.visible = false;
        }
        activeWorkspaceId = workspace.id;
        workspace.visible = true;
        workspace.bounds = command.bounds;
        layout(workspace);
        return publish(workspace);
      }
      case "hide":
        workspace.visible = false;
        if (activeWorkspaceId === workspace.id) {
          activeWorkspaceId = null;
          unmount();
        }
        return publish(workspace);
      case "set-bounds":
        workspace.bounds = command.bounds;
        layout(workspace);
        return publish(workspace);
      case "new-tab":
        createTab(workspace, command.address, true);
        saveWorkspace(workspace);
        layout(workspace);
        return publish(workspace);
      case "select-tab":
        requireTab(workspace, command.tabId);
        workspace.activeTabId = command.tabId;
        saveWorkspace(workspace);
        layout(workspace);
        return publish(workspace);
      case "close-tab": {
        const tab = requireTab(workspace, command.tabId);
        const index = workspace.tabs.indexOf(tab);
        if (mountedView === tab.view) unmount();
        workspace.tabs.splice(index, 1);
        disposeTabDebugger(tab);
        if (!tab.view.webContents.isDestroyed()) tab.view.webContents.destroy();
        if (workspace.tabs.length === 0) createTab(workspace, undefined, true);
        else if (workspace.activeTabId === tab.id) workspace.activeTabId = workspace.tabs[Math.min(index, workspace.tabs.length - 1)]!.id;
        saveWorkspace(workspace);
        layout(workspace);
        return publish(workspace);
      }
      case "navigate": {
        const tab = requireTab(workspace, command.tabId);
        const safe = normalizeBrowserWorkspaceAddress(command.address);
        if (safe == null) throw new Error("BROWSER_NAVIGATION_BLOCKED");
        tab.url = safe;
        tab.error = null;
        const downloadSequence = tab.downloadSequence;
        try {
          await tab.view.webContents.loadURL(safe);
        } catch (error) {
          const failureUrl = record(error)?.url;
          const matchesConfirmedDownload = tab.downloadSequence !== downloadSequence
            && (tab.lastDownloadUrl === safe || (typeof failureUrl === "string" && failureUrl === tab.lastDownloadUrl));
          if (!matchesConfirmedDownload || !isExpectedDownloadNavigationError(error)) throw error;
          tab.loading = false;
          tab.error = null;
        }
        saveWorkspace(workspace);
        return publish(workspace);
      }
      case "back": {
        const tab = requireTab(workspace, command.tabId);
        if (tab.view.webContents.canGoBack()) tab.view.webContents.goBack();
        return publish(workspace);
      }
      case "forward": {
        const tab = requireTab(workspace, command.tabId);
        if (tab.view.webContents.canGoForward()) tab.view.webContents.goForward();
        return publish(workspace);
      }
      case "reload": {
        const tab = requireTab(workspace, command.tabId);
        tab.crashed = false;
        tab.error = null;
        tab.view.webContents.reload();
        return publish(workspace);
      }
      case "stop":
        requireTab(workspace, command.tabId).view.webContents.stop();
        return publish(workspace);
      case "set-zoom": {
        const tab = requireTab(workspace, command.tabId);
        const factor = Math.max(BROWSER_WORKSPACE_LIMITS.minZoomFactor, Math.min(BROWSER_WORKSPACE_LIMITS.maxZoomFactor, command.factor));
        tab.view.webContents.setZoomFactor(factor);
        return publish(workspace);
      }
      case "add-bookmark": {
        const tab = requireTab(workspace, command.tabId);
        const url = normalizeBrowserWorkspaceAddress(tab.url);
        if (url == null) return publish(workspace);
        const account = accountFor(workspace.accountHash);
        if (!account.bookmarks.some((bookmark) => bookmark.url === url)) {
          account.bookmarks.push({ id: nextId(), title: tab.title || url, url, createdAtMs: now() });
          if (account.bookmarks.length > BROWSER_WORKSPACE_LIMITS.maxBookmarks) account.bookmarks.splice(0, account.bookmarks.length - BROWSER_WORKSPACE_LIMITS.maxBookmarks);
          persist();
        }
        return publish(workspace);
      }
      case "remove-bookmark": {
        const account = accountFor(workspace.accountHash);
        account.bookmarks = account.bookmarks.filter((bookmark) => bookmark.id !== command.bookmarkId);
        persist();
        return publish(workspace);
      }
      case "extension-choose": {
        if (ports.pickExtensionDirectory == null) throw new Error("BROWSER_EXTENSIONS_UNAVAILABLE");
        const selected = await ports.pickExtensionDirectory();
        if (selected == null) {
          workspace.extensionCandidate = null;
          return publish(workspace);
        }
        const inspection = inspectUnpackedBrowserExtension(selected);
        workspace.extensionCandidate = { candidateId: nextId(), inspection };
        return publish(workspace);
      }
      case "extension-install": {
        const candidate = workspace.extensionCandidate;
        if (candidate == null || candidate.candidateId !== command.candidateId) throw new Error("BROWSER_EXTENSION_CANDIDATE_STALE");
        const inspection = inspectUnpackedBrowserExtension(candidate.inspection.sourcePath);
        if (inspection.integrity !== candidate.inspection.integrity) throw new Error("BROWSER_EXTENSION_INTEGRITY_CHANGED");
        const account = accountFor(workspace.accountHash);
        const key = hashIdentity(inspection.sourcePath);
        if (account.extensions.some((extension) => extension.key === key)) throw new Error("BROWSER_EXTENSION_ALREADY_INSTALLED");
        const extension: MutableExtension = {
          key,
          sourcePath: inspection.sourcePath,
          integrity: inspection.integrity,
          name: inspection.name,
          version: inspection.version,
          enabled: true,
          permissions: inspection.permissions,
          hostPermissions: inspection.hostPermissions,
          runtimeId: null,
          error: null,
        };
        await loadExtension(activeTab(workspace).view.webContents.session, workspace.accountHash, extension, inspection);
        account.extensions.push(extension);
        workspace.extensionCandidate = null;
        persist();
        return publish(workspace);
      }
      case "extension-set-enabled": {
        const account = accountFor(workspace.accountHash);
        const extension = account.extensions.find((item) => item.key === command.extensionKey);
        if (extension == null) throw new Error("BROWSER_EXTENSION_NOT_FOUND");
        const session = activeTab(workspace).view.webContents.session;
        if (!command.enabled) {
          unloadExtension(session, extension);
          extension.enabled = false;
          extension.error = null;
        } else {
          const inspection = inspectUnpackedBrowserExtension(extension.sourcePath);
          if (inspection.integrity !== extension.integrity) throw new Error("BROWSER_EXTENSION_INTEGRITY_CHANGED");
          await loadExtension(session, workspace.accountHash, extension, inspection);
          extension.enabled = true;
        }
        persist();
        return publish(workspace);
      }
      case "extension-update": {
        const account = accountFor(workspace.accountHash);
        const extension = account.extensions.find((item) => item.key === command.extensionKey);
        if (extension == null) throw new Error("BROWSER_EXTENSION_NOT_FOUND");
        const session = activeTab(workspace).view.webContents.session;
        const inspection = inspectUnpackedBrowserExtension(extension.sourcePath);
        unloadExtension(session, extension);
        try {
          if (extension.enabled) await loadExtension(session, workspace.accountHash, extension, inspection);
          extension.integrity = inspection.integrity;
          extension.name = inspection.name;
          extension.version = inspection.version;
          extension.permissions = inspection.permissions;
          extension.hostPermissions = inspection.hostPermissions;
          extension.error = null;
          persist();
          return publish(workspace);
        } catch (error) {
          extension.error = error instanceof Error ? error.message.slice(0, 256) : "BROWSER_EXTENSION_UPDATE_FAILED";
          persist();
          publish(workspace);
          throw error;
        }
      }
      case "extension-remove": {
        const account = accountFor(workspace.accountHash);
        const index = account.extensions.findIndex((item) => item.key === command.extensionKey);
        if (index < 0) throw new Error("BROWSER_EXTENSION_NOT_FOUND");
        unloadExtension(activeTab(workspace).view.webContents.session, account.extensions[index]!);
        account.extensions.splice(index, 1);
        persist();
        return publish(workspace);
      }
      case "agent-control": {
        const previousState = workspace.agentControlState;
        workspace.agentControlEpoch += 1;
        if (command.action !== "resume") {
          for (const [downloadId, active] of activeAgentDownloads) {
            if (active.workspaceId !== workspace.id) continue;
            try { active.item.cancel(); } catch {}
            activeAgentDownloads.delete(downloadId);
          }
          for (const tab of workspace.tabs) tab.pendingAgentDownloadOperationId = null;
        }
        workspace.activeAgentOperationId = null;
        workspace.agentControlState = command.action === "resume"
          ? "available"
          : command.action === "pause"
            ? "paused"
            : command.action === "takeover"
              ? "taken-over"
              : "stopped";
        if (command.action === "pause") {
          if (workspace.agentTaskStatus === "running") workspace.agentTaskStatus = "paused";
        } else if (command.action === "takeover") {
          workspace.agentTaskStatus = "taken-over";
        } else if (command.action === "stop") {
          workspace.agentTaskStatus = "stopped";
        } else if (previousState === "paused") {
          if (workspace.agentTaskStatus === "paused") workspace.agentTaskStatus = "running";
        } else {
          workspace.agentTaskId = nextId();
          workspace.agentTaskStatus = "idle";
        }
        return publish(workspace);
      }
    }
  };

  const agentResult = (
    operationId: string,
    workspace: RuntimeWorkspace,
    tab: RuntimeTab,
    extra: Partial<Extract<LocalBrowserAgentActionResult, { readonly ok: true }>> = {},
  ): LocalBrowserAgentActionResult => ({
    ok: true,
    operationId,
    workspaceId: workspace.id,
    tabId: tab.id,
    generation: tab.generation,
    controlEpoch: workspace.agentControlEpoch,
    taskId: workspace.agentTaskId,
    observedUrl: tab.view.webContents.getURL() || tab.url,
    observedTitle: tab.view.webContents.getTitle() || tab.title,
    eventSequence: tab.eventSequence,
    dialog: tab.pendingDialog,
    ...extra,
  });

  const requireAgentRef = (tab: RuntimeTab, ref: string): LocalBrowserElementLocator => {
    const locator = tab.agentRefs.get(ref);
    if (locator == null) throw new Error("BROWSER_REF_NOT_FOUND");
    return locator;
  };

  const agentTabs = (workspace: RuntimeWorkspace) => workspace.tabs.map((candidate) => ({
    tabId: candidate.id,
    title: candidate.title,
    url: candidate.url,
    generation: candidate.generation,
    active: candidate.id === workspace.activeTabId,
  }));

  const ensureAgentTaskRunning = (workspace: RuntimeWorkspace): void => {
    if (workspace.agentTaskStatus === "running") return;
    if (workspace.agentTaskStatus === "paused" || workspace.agentTaskStatus === "taken-over" || workspace.agentTaskStatus === "stopped") {
      throw new Error("BROWSER_AGENT_CONTROL_REVOKED");
    }
    const activeTasks = [...workspaces.values()].filter((candidate) =>
      candidate.id !== workspace.id
      && (candidate.agentTaskStatus === "running" || candidate.agentTaskStatus === "paused")
    ).length;
    if (activeTasks >= MAX_CONCURRENT_AGENT_TASKS) throw new Error("BROWSER_AGENT_TASK_LIMIT");
    workspace.agentTaskStatus = "running";
    publish(workspace);
  };

  const executeAgentActionNow = async (
    action: NonNullable<ReturnType<typeof parseLocalBrowserAgentAction>>,
  ): Promise<LocalBrowserAgentActionResult> => {
    if (disposed) throw new Error("BROWSER_WORKSPACE_DISPOSED");
    const workspace = requireExistingOwnedWorkspace(action.conversationId);

    if (action.type === "resolve-target") {
      if (workspace.agentControlState === "available") ensureAgentTaskRunning(workspace);
      const tab = action.tabId == null ? activeTab(workspace) : requireTab(workspace, action.tabId);
      return agentResult(action.operationId, workspace, tab, { tabs: agentTabs(workspace) });
    }

    const requireControlLease = (expectedControlEpoch: number, mutation: boolean): void => {
      if (workspace.agentControlEpoch !== expectedControlEpoch) throw new Error("BROWSER_AGENT_CONTROL_STALE");
      if (mutation && workspace.agentControlState !== "available") throw new Error("BROWSER_AGENT_CONTROL_REVOKED");
    };

    if (action.type === "tabs") {
      if (action.action !== "list") {
        if (workspace.agentTaskId !== action.expectedTaskId) throw new Error("BROWSER_AGENT_TASK_STALE");
        requireControlLease(action.expectedControlEpoch!, true);
        ensureAgentTaskRunning(workspace);
      }
      switch (action.action) {
        case "list":
          break;
        case "new":
          createTab(workspace, action.address, true);
          saveWorkspace(workspace);
          layout(workspace);
          break;
        case "select":
          requireTab(workspace, action.tabId!);
          workspace.activeTabId = action.tabId!;
          saveWorkspace(workspace);
          layout(workspace);
          break;
        case "close": {
          const closing = requireTab(workspace, action.tabId!);
          const index = workspace.tabs.indexOf(closing);
          if (mountedView === closing.view) unmount();
          workspace.tabs.splice(index, 1);
          disposeTabDebugger(closing);
          if (!closing.view.webContents.isDestroyed()) closing.view.webContents.destroy();
          if (workspace.tabs.length === 0) createTab(workspace, undefined, true);
          else if (workspace.activeTabId === closing.id) {
            workspace.activeTabId = workspace.tabs[Math.min(index, workspace.tabs.length - 1)]!.id;
          }
          saveWorkspace(workspace);
          layout(workspace);
          break;
        }
      }
      const tab = activeTab(workspace);
      publish(workspace);
      return agentResult(action.operationId, workspace, tab, { tabs: agentTabs(workspace) });
    }

    if (workspace.id !== action.workspaceId) throw new Error("BROWSER_WORKSPACE_MISMATCH");
    if (workspace.agentTaskId !== action.expectedTaskId) throw new Error("BROWSER_AGENT_TASK_STALE");
    const mutation = !["snapshot", "screenshot", "wait", "events", "wait-event"].includes(action.type);
    requireControlLease(action.expectedControlEpoch, mutation);
    ensureAgentTaskRunning(workspace);
    const tab = requireTab(workspace, action.tabId);
    if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
    const page = tab.view.webContents;
    if (tab.pendingDialog != null && !["dialog", "events", "wait-event"].includes(action.type)) {
      throw new Error("BROWSER_DIALOG_OPEN");
    }
    workspace.activeAgentOperationId = action.operationId;
    publish(workspace);

    try {
      switch (action.type) {
      case "snapshot": {
        const captured = await captureLocalBrowserSnapshot(page, tab.generation);
        if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
        tab.agentRefs = new Map(captured.locators);
        const snapshot: LocalBrowserStructuredSnapshot = {
          workspaceId: workspace.id,
          tabId: tab.id,
          generation: tab.generation,
          url: captured.url,
          title: captured.title,
          text: captured.text,
          elements: captured.elements,
        };
        return agentResult(action.operationId, workspace, tab, { snapshot });
      }
      case "click":
        await clickLocalBrowserRef(page, requireAgentRef(tab, action.ref));
        return agentResult(action.operationId, workspace, tab);
      case "download": {
        const before = new Set(workspace.downloads.map((download) => download.id));
        tab.pendingAgentDownloadOperationId = action.operationId;
        try {
          await clickLocalBrowserRef(page, requireAgentRef(tab, action.ref));
          const attempts = Math.ceil(LOCAL_BROWSER_AGENT_LIMITS.maxWaitMs / 25);
          for (let attempt = 0; attempt < attempts; attempt += 1) {
            if (workspace.agentControlEpoch !== action.expectedControlEpoch) throw new Error("BROWSER_AGENT_CONTROL_STALE");
            if (workspace.agentTaskId !== action.expectedTaskId) throw new Error("BROWSER_AGENT_TASK_STALE");
            const download = workspace.downloads.find((candidate) => candidate.tabId === tab.id && !before.has(candidate.id));
            if (download != null) {
              if (download.state === "cancelled") throw new Error("BROWSER_DOWNLOAD_CANCELLED");
              if (download.state === "interrupted") throw new Error("BROWSER_DOWNLOAD_INTERRUPTED");
              if (download.state === "completed" && download.sha256 != null) {
                return agentResult(action.operationId, workspace, tab, { downloadedFile: { downloadId: download.id, name: download.fileName, sizeBytes: download.receivedBytes, sha256: download.sha256 } });
              }
            }
            await new Promise((resolve) => setTimeout(resolve, 25));
          }
          throw new Error("BROWSER_DOWNLOAD_TIMEOUT");
        } finally {
          if (tab.pendingAgentDownloadOperationId === action.operationId) tab.pendingAgentDownloadOperationId = null;
        }
      }
      case "upload-file": {
        if (ports.pickUploadFile == null) throw new Error("BROWSER_UPLOAD_UNAVAILABLE");
        const selectedPath = await ports.pickUploadFile();
        if (selectedPath == null) throw new Error("BROWSER_UPLOAD_CANCELLED");
        if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
        requireControlLease(action.expectedControlEpoch, true);
        let info;
        try {
          info = statSync(selectedPath);
        } catch {
          throw new Error("BROWSER_UPLOAD_FILE_UNAVAILABLE");
        }
        if (!info.isFile()) throw new Error("BROWSER_UPLOAD_FILE_INVALID");
        if (info.size > LOCAL_BROWSER_AGENT_LIMITS.maxUploadBytes) throw new Error("BROWSER_UPLOAD_FILE_TOO_LARGE");
        let bytes: Buffer;
        try {
          bytes = readFileSync(selectedPath);
        } catch {
          throw new Error("BROWSER_UPLOAD_FILE_UNAVAILABLE");
        }
        if (bytes.byteLength !== info.size || bytes.byteLength > LOCAL_BROWSER_AGENT_LIMITS.maxUploadBytes) {
          throw new Error("BROWSER_UPLOAD_FILE_CHANGED");
        }
        const name = (basename(selectedPath) || "upload.bin").slice(0, 255);
        await uploadLocalBrowserFile(page, requireAgentRef(tab, action.ref), {
          name,
          bytesBase64: bytes.toString("base64"),
          sizeBytes: bytes.byteLength,
        });
        if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
        requireControlLease(action.expectedControlEpoch, true);
        return agentResult(action.operationId, workspace, tab, { uploadedFile: { name, sizeBytes: bytes.byteLength } });
      }
      case "fill":
        await fillLocalBrowserRef(page, requireAgentRef(tab, action.ref), action.value, false);
        return agentResult(action.operationId, workspace, tab);
      case "select-option":
        await selectLocalBrowserOptions(page, requireAgentRef(tab, action.ref), action.values);
        return agentResult(action.operationId, workspace, tab);
      case "type":
        await fillLocalBrowserRef(page, requireAgentRef(tab, action.ref), action.text, true);
        return agentResult(action.operationId, workspace, tab);
      case "press-key":
        pressLocalBrowserKey(page, action.key);
        return agentResult(action.operationId, workspace, tab);
      case "scroll":
        await scrollLocalBrowserPage(page, action.deltaX, action.deltaY);
        return agentResult(action.operationId, workspace, tab);
      case "drag":
        await dragLocalBrowserRef(
          page,
          requireAgentRef(tab, action.sourceRef),
          action.targetRef != null ? requireAgentRef(tab, action.targetRef) : { x: action.targetX!, y: action.targetY! },
        );
        return agentResult(action.operationId, workspace, tab);
      case "screenshot":
        return agentResult(action.operationId, workspace, tab, {
          screenshotBase64: await captureLocalBrowserScreenshotBase64(page),
        });
      case "wait": {
        const deadline = now() + Math.min(action.timeoutMs ?? 5_000, LOCAL_BROWSER_AGENT_LIMITS.maxWaitMs);
        const locator = action.ref == null ? null : requireAgentRef(tab, action.ref);
        let matched = false;
        do {
          if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
          requireControlLease(action.expectedControlEpoch, false);
          const refReady = locator == null || await localBrowserRefStillMatches(page, locator);
          const textReady = action.text == null || await localBrowserPageContainsText(page, action.text);
          matched = refReady && textReady;
          if (matched) break;
          await new Promise((resolve) => setTimeout(resolve, 50));
        } while (now() < deadline);
        if (!matched) throw new Error("BROWSER_WAIT_TIMEOUT");
        return agentResult(action.operationId, workspace, tab, { waited: true });
      }
      case "dialog": {
        const pending = tab.pendingDialog;
        if (pending == null) throw new Error("BROWSER_DIALOG_NOT_OPEN");
        const debuggerPort = page.debugger;
        if (!tab.debuggerReady || debuggerPort == null || !debuggerPort.isAttached()) {
          throw new Error("BROWSER_DIALOG_CONTROL_UNAVAILABLE");
        }
        if (action.promptText !== undefined && pending.type !== "prompt") {
          throw new Error("BROWSER_DIALOG_PROMPT_TEXT_NOT_APPLICABLE");
        }
        await debuggerPort.sendCommand("Page.handleJavaScriptDialog", {
          accept: action.decision === "accept",
          ...(action.promptText === undefined ? {} : { promptText: action.promptText }),
        });
        if (tab.pendingDialog === pending) {
          tab.pendingDialog = null;
          appendAgentEvent(tab, "dialog-closed");
        }
        return agentResult(action.operationId, workspace, tab, {
          dialogResult: { type: pending.type, decision: action.decision },
        });
      }
      case "events": {
        const afterSequence = action.afterSequence ?? 0;
        if (afterSequence > tab.eventSequence) throw new Error("BROWSER_EVENT_CURSOR_INVALID");
        const limit = action.limit ?? LOCAL_BROWSER_AGENT_LIMITS.maxEventBatch;
        const events = tab.events
          .filter((event) => event.sequence > afterSequence)
          .slice(-limit);
        return agentResult(action.operationId, workspace, tab, { events });
      }
      case "wait-event": {
        const afterSequence = action.afterSequence ?? tab.eventSequence;
        if (afterSequence > tab.eventSequence) throw new Error("BROWSER_EVENT_CURSOR_INVALID");
        if (action.event === "network-idle" && !tab.debuggerReady) {
          throw new Error("BROWSER_NETWORK_OBSERVATION_UNAVAILABLE");
        }
        const deadline = now() + Math.min(action.timeoutMs ?? 5_000, LOCAL_BROWSER_AGENT_LIMITS.maxWaitMs);
        do {
          if (tab.generation !== action.expectedGeneration) throw new Error("BROWSER_STALE_GENERATION");
          requireControlLease(action.expectedControlEpoch, false);
          const observed = tab.events.find((event) => event.sequence > afterSequence && event.type === action.event);
          if (observed != null) return agentResult(action.operationId, workspace, tab, { event: observed, waited: true });
          if (action.event === "network-idle"
            && tab.activeNetworkRequests.size === 0
            && now() - tab.lastNetworkActivityAt >= LOCAL_BROWSER_AGENT_LIMITS.networkIdleMs) {
            const event = appendAgentEvent(tab, "network-idle");
            tab.networkIdleEmittedToken = tab.networkIdleToken;
            return agentResult(action.operationId, workspace, tab, { event, waited: true });
          }
          await new Promise((resolve) => setTimeout(resolve, 25));
        } while (now() < deadline);
        throw new Error("BROWSER_EVENT_WAIT_TIMEOUT");
      }
      case "navigate": {
        const safe = normalizeBrowserWorkspaceAddress(action.address);
        if (safe == null) throw new Error("BROWSER_NAVIGATION_BLOCKED");
        tab.error = null;
        const downloadSequence = tab.downloadSequence;
        try {
          await page.loadURL(safe);
        } catch (error) {
          const failureUrl = record(error)?.url;
          const matchesConfirmedDownload = tab.downloadSequence !== downloadSequence
            && (tab.lastDownloadUrl === safe || (typeof failureUrl === "string" && failureUrl === tab.lastDownloadUrl));
          if (!matchesConfirmedDownload || !isExpectedDownloadNavigationError(error)) throw error;
          tab.loading = false;
          tab.error = null;
        }
        saveWorkspace(workspace);
        return agentResult(action.operationId, workspace, tab);
      }
      case "back":
        if (page.canGoBack()) page.goBack();
        return agentResult(action.operationId, workspace, tab);
      case "forward":
        if (page.canGoForward()) page.goForward();
        return agentResult(action.operationId, workspace, tab);
      case "reload":
        tab.error = null;
        page.reload();
        return agentResult(action.operationId, workspace, tab);
      }
    } finally {
      if (workspace.activeAgentOperationId === action.operationId) {
        workspace.activeAgentOperationId = null;
        publish(workspace);
      }
    }
  };

  const executeAgentAction = async (raw: unknown): Promise<LocalBrowserAgentActionResult> => {
    const action = parseLocalBrowserAgentAction(raw);
    if (action == null) throw new TypeError("BROWSER_AGENT_INVALID_ACTION");
    if (disposed) throw new Error("BROWSER_WORKSPACE_DISPOSED");
    const workspace = requireExistingOwnedWorkspace(action.conversationId);
    const queueKey = workspace.id;
    const previous = agentQueues.get(queueKey) ?? Promise.resolve();
    let release!: () => void;
    const gate = new Promise<void>((resolve) => { release = resolve; });
    const tail = previous.catch(() => undefined).then(() => gate);
    agentQueues.set(queueKey, tail);
    await previous.catch(() => undefined);
    try {
      if (disposed) throw new Error("BROWSER_WORKSPACE_DISPOSED");
      return await executeAgentActionNow(action);
    } finally {
      release();
      if (agentQueues.get(queueKey) === tail) {
        void tail.finally(() => {
          if (agentQueues.get(queueKey) === tail) agentQueues.delete(queueKey);
        });
      }
    }
  };

  ports.ipcMain.handle(BROWSER_WORKSPACE_INVOKE_CHANNEL, async (event, raw) => {
    const trusted = ports.getTrustedContents();
    const eventRecord = record(event);
    if (trusted == null || eventRecord?.sender !== trusted || eventRecord.senderFrame !== trusted.mainFrame) throw new Error("BROWSER_UNAUTHORIZED_SENDER");
    const command = parseBrowserWorkspaceCommand(raw);
    if (command == null) throw new Error("BROWSER_INVALID_COMMAND");
    return await execute(command);
  });

  const destroyAllRuntime = (): void => {
    unmount();
    activeWorkspaceId = null;
    for (const workspace of workspaces.values()) {
      workspace.visible = false;
      for (const tab of workspace.tabs) {
        disposeTabDebugger(tab);
        try { if (!tab.view.webContents.isDestroyed()) tab.view.webContents.destroy(); } catch (error) { fail("destroy-tab", error); }
      }
    }
    workspaces.clear();
    activeAgentDownloads.clear();
  };

  return {
    attachWindow(window) {
      if (disposed) return;
      unmount();
      hostWindow = window;
      window.on?.("resize", () => {
        if (activeWorkspaceId == null) return;
        const workspace = workspaces.get(activeWorkspaceId);
        if (workspace != null) layout(workspace);
      });
      window.on?.("closed", () => {
        unmount();
        hostWindow = null;
      });
      if (activeWorkspaceId != null) {
        const workspace = workspaces.get(activeWorkspaceId);
        if (workspace != null) layout(workspace);
      }
    },
    revokeAccount() {
      if (disposed) return;
      destroyAllRuntime();
    },
    snapshot(conversationId) {
      if (disposed) return null;
      try { return snapshot(requireOwnedWorkspace(conversationId)); } catch { return null; }
    },
    executeAgentAction,
    dispose() {
      if (disposed) return;
      disposed = true;
      agentQueues.clear();
      ports.ipcMain.removeHandler?.(BROWSER_WORKSPACE_INVOKE_CHANNEL);
      destroyAllRuntime();
      hostWindow = null;
    },
  };
}
