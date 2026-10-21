import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import {
  createBrowserWorkspaceService,
  type BrowserWorkspaceDownloadItem,
  type BrowserWorkspaceHostWindowPort,
  type BrowserWorkspaceIpcPort,
  type BrowserWorkspacePersistencePort,
  type BrowserWorkspaceSessionPort,
  type BrowserWorkspaceViewPort,
  type BrowserWorkspaceWebContentsPort,
} from "./browser-workspace.js";
import { LOCAL_BROWSER_AGENT_LIMITS } from "../shared/local-browser-agent.js";
import {
  BROWSER_WORKSPACE_INVOKE_CHANNEL,
  normalizeBrowserWorkspaceAddress,
  type BrowserWorkspaceBounds,
  type BrowserWorkspaceCommand,
  type BrowserWorkspaceSnapshot,
} from "../shared/browser-workspace.js";

class FakeSession implements BrowserWorkspaceSessionPort {
  readonly loadedExtensions = new Map<string, string>();
  readonly extensions = {
    loadExtension: async (path: string) => {
      const id = `extension-${this.loadedExtensions.size + 1}`;
      this.loadedExtensions.set(id, path);
      return { id, name: "Fabushi Test Extension", version: "1.0.0" };
    },
    removeExtension: (id: string) => { this.loadedExtensions.delete(id); },
  };
  permissionRequest: ((contents: BrowserWorkspaceWebContentsPort, permission: string, callback: (allowed: boolean) => void) => void) | null = null;
  permissionCheck: ((contents: BrowserWorkspaceWebContentsPort, permission: string) => boolean) | null = null;
  download: ((event: unknown, item: BrowserWorkspaceDownloadItem, contents: BrowserWorkspaceWebContentsPort) => void) | null = null;
  setPermissionRequestHandler(handler: FakeSession["permissionRequest"]): void { this.permissionRequest = handler; }
  setPermissionCheckHandler(handler: FakeSession["permissionCheck"]): void { this.permissionCheck = handler; }
  on(_event: "will-download", listener: NonNullable<FakeSession["download"]>): void { this.download = listener; }
}

class FakeDebugger {
  attached = false;
  readonly commands: Array<{ readonly method: string; readonly params?: Record<string, unknown> }> = [];
  readonly listeners = new Map<string, Array<(event: unknown, method: string, params: unknown, sessionId?: string) => void>>();
  isAttached(): boolean { return this.attached; }
  attach(_protocolVersion?: string): void { this.attached = true; }
  detach(): void { this.attached = false; }
  async sendCommand(method: string, params?: Record<string, unknown>): Promise<unknown> {
    this.commands.push({ method, ...(params === undefined ? {} : { params: structuredClone(params) }) });
    return {};
  }
  on(event: "message", listener: (event: unknown, method: string, params: unknown, sessionId?: string) => void): void {
    const listeners = this.listeners.get(event) ?? [];
    listeners.push(listener);
    this.listeners.set(event, listeners);
  }
  removeListener(event: "message", listener: (event: unknown, method: string, params: unknown, sessionId?: string) => void): void {
    this.listeners.set(event, (this.listeners.get(event) ?? []).filter((candidate) => candidate !== listener));
  }
  emitMessage(method: string, params: unknown): void {
    for (const listener of this.listeners.get("message") ?? []) listener({}, method, params);
  }
}

class FakeContents implements BrowserWorkspaceWebContentsPort {
  readonly debugger = new FakeDebugger();
  readonly mainFrame: import("./local-browser-agent-page.js").LocalBrowserFramePort = {
    url: "https://example.com/",
    name: "",
    frameTreeNodeId: 1,
    detached: false,
    framesInSubtree: [],
    executeJavaScript: async (script: string) => await this.executeJavaScript(script),
  };
  readonly listeners = new Map<string, Array<(...args: any[]) => void>>();
  popup: ((details: { readonly url: string }) => { action: "deny" }) | null = null;
  url = "about:blank";
  title = "New Tab";
  zoom = 1;
  destroyed = false;
  stopped = false;
  reloaded = false;
  back = false;
  forward = false;
  readonly inputEvents: Array<{ readonly type: string; readonly keyCode?: string; readonly x?: number; readonly y?: number; readonly button?: "left"; readonly clickCount?: number }> = [];
  readonly executedScripts: string[] = [];
  snapshotElements: unknown[] = [{
    selector: "html > body > button",
    tag: "button",
    role: null,
    name: "Submit",
    text: "Submit",
    href: null,
    inputType: null,
    disabled: false,
    editable: false,
    checked: null,
    selected: null,
    frameUrl: null,
  }];
  constructor(readonly session: FakeSession) {}
  on(event: string, listener: (...args: any[]) => void): void {
    const list = this.listeners.get(event) ?? [];
    list.push(listener);
    this.listeners.set(event, list);
  }
  emit(event: string, ...args: unknown[]): void {
    for (const listener of this.listeners.get(event) ?? []) listener(...args);
  }
  async loadURL(url: string): Promise<unknown> {
    this.emit("did-start-loading");
    this.url = url;
    this.title = `Title ${new URL(url).hostname}`;
    this.emit("did-navigate", {}, url);
    this.emit("page-title-updated", {}, this.title);
    this.emit("did-finish-load");
    this.emit("did-stop-loading");
    return undefined;
  }
  getURL(): string { return this.url; }
  getTitle(): string { return this.title; }
  canGoBack(): boolean { return this.back; }
  canGoForward(): boolean { return this.forward; }
  goBack(): void { this.back = false; }
  goForward(): void { this.forward = false; }
  reload(): void { this.reloaded = true; this.emit("did-start-loading"); this.emit("did-stop-loading"); }
  stop(): void { this.stopped = true; }
  getZoomFactor(): number { return this.zoom; }
  setZoomFactor(factor: number): void { this.zoom = factor; }
  async executeJavaScript(script: string): Promise<unknown> {
    this.executedScripts.push(script);
    if (script.includes("MAX_ELEMENTS") && script.includes("document.querySelectorAll")) {
      return {
        url: this.url,
        title: this.title,
        text: "Synthetic page Submit",
        elements: this.snapshotElements,
      };
    }
    if (script.includes("getBoundingClientRect")) return { ok: true, x: 40, y: 50 };
    if (script.includes("document.querySelector(locator.selector)")) return { ok: true };
    if (script.includes("document.body && document.body.innerText")) return true;
    if (script.includes("window.scrollBy")) return { ok: true };
    return { ok: true };
  }
  async capturePage(): Promise<{ toPNG(): Buffer }> {
    return { toPNG: () => Buffer.from("synthetic-png") };
  }
  sendInputEvent(event: { readonly type: "keyDown" | "keyUp" | "char" | "mouseMove" | "mouseDown" | "mouseUp"; readonly keyCode?: string; readonly x?: number; readonly y?: number; readonly button?: "left"; readonly clickCount?: number }): void {
    this.inputEvents.push(event);
  }
  isDestroyed(): boolean { return this.destroyed; }
  destroy(): void { this.destroyed = true; this.emit("destroyed"); }
  setWindowOpenHandler(handler: NonNullable<FakeContents["popup"]>): void { this.popup = handler; }
}

class FakeView implements BrowserWorkspaceViewPort {
  bounds: BrowserWorkspaceBounds = { x: 0, y: 0, width: 1, height: 1 };
  constructor(readonly webContents: FakeContents) {}
  setBounds(bounds: BrowserWorkspaceBounds): void { this.bounds = bounds; }
}

class FakeHostWindow implements BrowserWorkspaceHostWindowPort {
  destroyed = false;
  mounted: BrowserWorkspaceViewPort[] = [];
  readonly listeners = new Map<string, () => void>();
  readonly contentView = {
    addChildView: (view: BrowserWorkspaceViewPort) => { if (!this.mounted.includes(view)) this.mounted.push(view); },
    removeChildView: (view: BrowserWorkspaceViewPort) => { this.mounted = this.mounted.filter((candidate) => candidate !== view); },
  };
  isDestroyed(): boolean { return this.destroyed; }
  on(event: "resize" | "closed", listener: () => void): void { this.listeners.set(event, listener); }
}

class FakeIpc implements BrowserWorkspaceIpcPort {
  handler: ((event: unknown, request?: unknown) => unknown) | null = null;
  channel: string | null = null;
  handle(channel: string, listener: (event: unknown, request?: unknown) => unknown): void {
    this.channel = channel;
    this.handler = listener;
  }
  removeHandler(channel: string): void {
    if (this.channel === channel) {
      this.channel = null;
      this.handler = null;
    }
  }
}

class MemoryPersistence implements BrowserWorkspacePersistencePort {
  state: unknown = null;
  readonly paths: string[] = [];
  load(): unknown { return this.state; }
  save(value: unknown): void { this.state = structuredClone(value); }
  prepareDownloadPath(accountHash: string, downloadId: string, fileName: string): string {
    const path = `/downloads/${accountHash}/${downloadId}-${fileName}`;
    this.paths.push(path);
    return path;
  }
  finalizeDownload(_path: string, maxBytes: number): { readonly sizeBytes: number; readonly sha256: string } {
    const bytes = Buffer.from("synthetic-download", "utf8");
    assert.ok(bytes.byteLength <= maxBytes);
    return { sizeBytes: bytes.byteLength, sha256: "synthetic-sha256" };
  }
}

function harness(
  initialState: unknown = null,
  pickUploadFile?: () => Promise<string | null>,
  pickExtensionDirectory?: () => Promise<string | null>,
  nowOverride?: () => number,
) {
  const ipc = new FakeIpc();
  const persistence = new MemoryPersistence();
  persistence.state = initialState;
  const sessions = new Map<string, FakeSession>();
  const views: FakeView[] = [];
  const trustedSession = new FakeSession();
  const trusted = new FakeContents(trustedSession);
  let accountScope = "account-a";
  let id = 0;
  const broadcasts: unknown[] = [];
  const service = createBrowserWorkspaceService({
    ipcMain: ipc,
    createView: ({ partition }) => {
      const session = sessions.get(partition) ?? new FakeSession();
      sessions.set(partition, session);
      const view = new FakeView(new FakeContents(session));
      views.push(view);
      return view;
    },
    getTrustedContents: () => trusted,
    getAccountScope: () => accountScope,
    persistence,
    ...(pickUploadFile == null ? {} : { pickUploadFile }),
    ...(pickExtensionDirectory == null ? {} : { pickExtensionDirectory }),
    broadcast: (_channel, payload) => broadcasts.push(payload),
    id: () => `id-${++id}`,
    now: nowOverride ?? (() => 1_700_000_000_000 + id),
  });
  const invoke = async (command: BrowserWorkspaceCommand): Promise<BrowserWorkspaceSnapshot> => {
    assert.equal(ipc.channel, BROWSER_WORKSPACE_INVOKE_CHANNEL);
    assert.ok(ipc.handler);
    return await ipc.handler({ sender: trusted, senderFrame: trusted.mainFrame }, command) as BrowserWorkspaceSnapshot;
  };
  return {
    service, ipc, persistence, sessions, views, trusted, broadcasts, invoke,
    setAccountScope(value: string) { accountScope = value; },
  };
}

test("navigation policy permits HTTPS/search/loopback and rejects privileged or credential URLs", () => {
  assert.equal(normalizeBrowserWorkspaceAddress("example.com"), "https://example.com/");
  assert.equal(normalizeBrowserWorkspaceAddress("https://example.com/a?q=1"), "https://example.com/a?q=1");
  assert.equal(normalizeBrowserWorkspaceAddress("http://127.0.0.1:8080/test"), "http://127.0.0.1:8080/test");
  assert.equal(normalizeBrowserWorkspaceAddress("find cats"), "https://www.google.com/search?q=find%20cats");
  assert.equal(normalizeBrowserWorkspaceAddress("http://example.com"), null);
  assert.equal(normalizeBrowserWorkspaceAddress("file:///etc/passwd"), null);
  assert.equal(normalizeBrowserWorkspaceAddress("javascript:alert(1)"), null);
  assert.equal(normalizeBrowserWorkspaceAddress("https://user:pass@example.com"), null);
});

test("trusted renderer activates a real view boundary with account-scoped session and denied page permissions", async () => {
  const h = harness();
  const host = new FakeHostWindow();
  h.service.attachWindow(host);
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 500.8, y: 50.2, width: 700.9, height: 800.1 } });
  assert.equal(state.visible, true);
  assert.equal(state.tabs.length, 1);
  assert.equal(host.mounted.length, 1);
  assert.deepEqual(h.views[0]?.bounds, { x: 500, y: 50, width: 700, height: 800 });
  assert.equal(h.sessions.size, 1);
  const [partition, session] = [...h.sessions.entries()][0]!;
  assert.match(partition, /^persist:fabushi-browser-[a-f0-9]{64}$/);
  assert.ok(session.permissionRequest);
  let allowed = true;
  session.permissionRequest!(h.views[0]!.webContents, "geolocation", (result) => { allowed = result; });
  assert.equal(allowed, false);
  assert.equal(session.permissionCheck!(h.views[0]!.webContents, "geolocation"), false);
});

test("tabs stay identity-isolated across conversations and wrong-tab mutations are rejected", async () => {
  const h = harness();
  const host = new FakeHostWindow();
  h.service.attachWindow(host);
  const a = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 500, y: 50, width: 700, height: 800 } });
  const aTab = a.activeTabId;
  const a2 = await h.invoke({ type: "new-tab", conversationId: "bot-a", address: "https://example.com/a" });
  assert.equal(a2.tabs.length, 2);
  assert.notEqual(a2.activeTabId, aTab);

  const b = await h.invoke({ type: "activate", conversationId: "bot-b", bounds: { x: 400, y: 50, width: 800, height: 800 } });
  assert.equal(b.tabs.length, 1);
  assert.notEqual(b.workspaceId, a.workspaceId);
  assert.equal(host.mounted.length, 1);
  await assert.rejects(() => h.invoke({ type: "select-tab", conversationId: "bot-b", tabId: a2.activeTabId }), /BROWSER_TAB_NOT_FOUND/);
  const after = h.service.snapshot("bot-b");
  assert.equal(after?.tabs.length, 1);
});

test("select/close/zoom/navigation operations preserve the selected workspace", async () => {
  const h = harness();
  const host = new FakeHostWindow();
  h.service.attachWindow(host);
  const first = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 600, y: 60, width: 600, height: 700 } });
  const firstTab = first.activeTabId;
  const withSecond = await h.invoke({ type: "new-tab", conversationId: "bot-a", address: "https://example.com/path" });
  const secondTab = withSecond.activeTabId;
  assert.equal(withSecond.tabs.find((tab) => tab.id === secondTab)?.url, "https://example.com/path");
  await h.invoke({ type: "set-zoom", conversationId: "bot-a", tabId: secondTab, factor: 99 });
  assert.equal(h.views.at(-1)?.webContents.zoom, 3);
  await h.invoke({ type: "select-tab", conversationId: "bot-a", tabId: firstTab });
  const closed = await h.invoke({ type: "close-tab", conversationId: "bot-a", tabId: secondTab });
  assert.equal(closed.tabs.length, 1);
  assert.equal(closed.activeTabId, firstTab);
});

test("popup policy denies Chromium windows and converts only allowed URLs into managed tabs", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const owner = h.views[0]!.webContents;
  assert.ok(owner.popup);
  assert.deepEqual(owner.popup!({ url: "javascript:alert(1)" }), { action: "deny" });
  assert.equal(h.service.snapshot("bot-a")?.tabs.length, 1);
  assert.deepEqual(owner.popup!({ url: "https://example.com/popup" }), { action: "deny" });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(h.service.snapshot("bot-a")?.tabs.length, 2);
  assert.equal(h.service.snapshot("bot-a")?.tabs.some((tab) => tab.url === "https://example.com/popup"), true);
});

test("history/bookmarks persist URLs only and restore bounded tabs after service recreation", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  let state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  state = await h.invoke({ type: "navigate", conversationId: "bot-a", tabId: state.activeTabId, address: "https://example.com/persisted" });
  state = await h.invoke({ type: "add-bookmark", conversationId: "bot-a", tabId: state.activeTabId });
  assert.equal(state.bookmarks.length, 1);
  assert.equal(state.history.at(-1)?.url, "https://example.com/persisted");
  const persisted = h.persistence.state;
  h.service.dispose();

  const restored = harness(persisted);
  restored.service.attachWindow(new FakeHostWindow());
  const next = await restored.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  assert.equal(next.tabs.length, 1);
  assert.equal(next.tabs[0]?.url, "https://example.com/persisted");
  assert.equal(next.bookmarks[0]?.url, "https://example.com/persisted");
});

test("untrusted renderer cannot reach browser commands", async () => {
  const h = harness();
  assert.ok(h.ipc.handler);
  await assert.rejects(
    async () => await h.ipc.handler!({ sender: new FakeContents(new FakeSession()), senderFrame: {} }, { type: "hide", conversationId: "bot-a" }),
    /BROWSER_UNAUTHORIZED_SENDER/,
  );
  assert.equal(h.views.length, 0);
});

test("account revocation destroys runtime tabs and the next account gets a distinct workspace/profile", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const a = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const oldContents = h.views[0]!.webContents;
  h.service.revokeAccount();
  assert.equal(oldContents.destroyed, true);
  h.setAccountScope("account-b");
  const b = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  assert.notEqual(a.workspaceId, b.workspaceId);
  assert.equal(h.sessions.size, 2);
});

test("agent action snapshot mints opaque generation refs and fixed actions reject stale generations", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "op-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  assert.equal(snap.snapshot?.elements.length, 1);
  assert.equal(snap.snapshot?.elements[0]?.ref, `g${tab.generation}-e1`);

  const clicked = await h.service.executeAgentAction({
    type: "click",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "op-click",
    ref: snap.snapshot!.elements[0]!.ref,
  });
  assert.equal(clicked.ok, true);

  await h.invoke({ type: "navigate", conversationId: "bot-a", tabId: tab.id, address: "https://example.com/next" });
  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "click",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "op-stale",
      ref: snap.snapshot!.elements[0]!.ref,
    }),
    /BROWSER_STALE_GENERATION/,
  );
});

test("agent select-option is generation-fenced and rejects invalid payloads before page mutation", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "op-select-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;

  const selected = await h.service.executeAgentAction({
    type: "select-option",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "op-select",
    ref: snap.snapshot!.elements[0]!.ref,
    values: ["ready"],
  });
  assert.equal(selected.ok, true);

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "select-option",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "op-invalid-select",
      ref: snap.snapshot!.elements[0]!.ref,
      values: [],
    }),
    /BROWSER_AGENT_INVALID_ACTION/,
  );
});

test("agent actions fence workspace identity and expose only bounded fixed keyboard events", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "snapshot",
      conversationId: "bot-a",
      workspaceId: "wrong-workspace",
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "op-wrong",
    }),
    /BROWSER_WORKSPACE_MISMATCH/,
  );

  const pressed = await h.service.executeAgentAction({
    type: "press-key",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "op-key",
    key: "Enter",
  });
  assert.equal(pressed.ok, true);
  assert.deepEqual(h.views[0]!.webContents.inputEvents, [
    { type: "keyDown", keyCode: "Enter" },
    { type: "keyUp", keyCode: "Enter" },
  ]);

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "press-key",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "op-key-denied",
      key: "Meta+A",
    }),
    /BROWSER_KEY_DENIED/,
  );
});

test("agent target discovery and tab management stay scoped to the conversation workspace", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });

  const resolved = await h.service.executeAgentAction({
    type: "resolve-target",
    conversationId: "bot-a",
    operationId: "resolve-a",
  });
  assert.equal(resolved.ok, true);
  if (!resolved.ok) return;
  assert.equal(resolved.workspaceId, state.workspaceId);
  assert.equal(resolved.tabs?.length, 1);
  assert.equal(resolved.tabs?.[0]?.active, true);

  const created = await h.service.executeAgentAction({
    type: "tabs",
    action: "new",
    conversationId: "bot-a",
    operationId: "new-a",
    expectedControlEpoch: resolved.controlEpoch, expectedTaskId: resolved.taskId,
    address: "https://example.com/local",
  });
  assert.equal(created.ok, true);
  if (!created.ok) return;
  assert.equal(created.tabs?.length, 2);
  const newTab = created.tabs?.find((tab) => tab.active);
  assert.equal(newTab?.url, "https://example.com/local");

  const selected = await h.service.executeAgentAction({
    type: "tabs",
    action: "select",
    conversationId: "bot-a",
    operationId: "select-a",
    expectedControlEpoch: resolved.controlEpoch, expectedTaskId: resolved.taskId,
    tabId: state.activeTabId,
  });
  assert.equal(selected.ok, true);
  if (!selected.ok) return;
  assert.equal(selected.tabId, state.activeTabId);

  const closed = await h.service.executeAgentAction({
    type: "tabs",
    action: "close",
    conversationId: "bot-a",
    operationId: "close-a",
    expectedControlEpoch: resolved.controlEpoch, expectedTaskId: resolved.taskId,
    tabId: newTab!.tabId,
  });
  assert.equal(closed.ok, true);
  if (!closed.ok) return;
  assert.equal(closed.tabs?.length, 1);
  assert.equal(closed.tabs?.[0]?.tabId, state.activeTabId);
});

test("snapshot includes frame identity and routes ref actions to the owning frame", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const contents = h.views[0]!.webContents;
  let childActions = 0;
  const childFrame: import("./local-browser-agent-page.js").LocalBrowserFramePort = {
    url: "https://frame.example/child",
    name: "child",
    frameTreeNodeId: 22,
    detached: false,
    framesInSubtree: [],
    executeJavaScript: async (script: string) => {
      if (script.includes("MAX_ELEMENTS") && script.includes("document.querySelectorAll")) {
        return {
          url: "https://frame.example/child", title: "Child", text: "Child Frame Submit",
          elements: [{
            selector: "html > body > button", tag: "button", role: null, name: "Child Submit", text: "Child Submit",
            href: null, inputType: null, disabled: false, editable: false, checked: null, selected: null, frameUrl: null,
          }],
        };
      }
      childActions += 1;
      return { ok: true };
    },
  };
  const mainFrame = contents.mainFrame!;
  (mainFrame as { framesInSubtree: readonly import("./local-browser-agent-page.js").LocalBrowserFramePort[] }).framesInSubtree = [mainFrame, childFrame];

  const snap = await h.service.executeAgentAction({
    type: "snapshot", conversationId: "bot-a", workspaceId: state.workspaceId, tabId: state.tabs[0]!.id,
    expectedGeneration: state.tabs[0]!.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "frame-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const child = snap.snapshot!.elements.find((element) => element.frameId === 22);
  assert.equal(child?.frameName, "child");
  assert.equal(child?.frameUrl, "https://frame.example/child");
  assert.match(child!.ref, /^g\d+-f22-e\d+$/);

  const clicked = await h.service.executeAgentAction({
    type: "click", conversationId: "bot-a", workspaceId: state.workspaceId, tabId: state.tabs[0]!.id,
    expectedGeneration: state.tabs[0]!.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "frame-click", ref: child!.ref,
  });
  assert.equal(clicked.ok, true);
  assert.ok(childActions > 0);
});

test("agent drag uses generation-scoped refs and bounded native mouse input", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "drag-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const sourceRef = snap.snapshot!.elements[0]!.ref;
  const dragged = await h.service.executeAgentAction({
    type: "drag",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "drag-bounded",
    sourceRef,
    targetX: 120,
    targetY: 160,
  });
  assert.equal(dragged.ok, true);
  const pointerEvents = h.views[0]!.webContents.inputEvents.filter((event) => event.type.startsWith("mouse"));
  assert.equal(pointerEvents[0]?.type, "mouseMove");
  assert.equal(pointerEvents[1]?.type, "mouseDown");
  assert.equal(pointerEvents.at(-1)?.type, "mouseUp");

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "drag",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "drag-invalid",
      sourceRef,
      targetX: 20_000,
      targetY: 1,
    }),
    /BROWSER_AGENT_INVALID_ACTION/,
  );
});

test("user takeover rotates the Main-owned mutation lease and requires an explicit resume", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-a", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const resolved = await h.service.executeAgentAction({
    type: "resolve-target",
    conversationId: "bot-a",
    operationId: "lease-resolve",
  });
  assert.equal(resolved.ok, true);
  if (!resolved.ok) return;
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: resolved.controlEpoch, expectedTaskId: resolved.taskId,
    operationId: "lease-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const ref = snap.snapshot!.elements[0]!.ref;

  const taken = await h.invoke({ type: "agent-control", conversationId: "bot-a", action: "takeover" });
  assert.equal(taken.agentControl.state, "taken-over");
  assert.ok(taken.agentControl.epoch > resolved.controlEpoch);

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "click",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: resolved.controlEpoch, expectedTaskId: resolved.taskId,
      operationId: "lease-stale-click",
      ref,
    }),
    /BROWSER_AGENT_CONTROL_STALE/,
  );

  const fresh = await h.service.executeAgentAction({
    type: "resolve-target",
    conversationId: "bot-a",
    operationId: "lease-fresh-resolve",
  });
  assert.equal(fresh.ok, true);
  if (!fresh.ok) return;
  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "click",
      conversationId: "bot-a",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: fresh.controlEpoch, expectedTaskId: fresh.taskId,
      operationId: "lease-revoked-click",
      ref,
    }),
    /BROWSER_AGENT_CONTROL_REVOKED/,
  );

  const resumed = await h.invoke({ type: "agent-control", conversationId: "bot-a", action: "resume" });
  assert.equal(resumed.agentControl.state, "available");
  const resumedTarget = await h.service.executeAgentAction({
    type: "resolve-target",
    conversationId: "bot-a",
    operationId: "lease-resume-resolve",
  });
  assert.equal(resumedTarget.ok, true);
  if (!resumedTarget.ok) return;
  const clicked = await h.service.executeAgentAction({
    type: "click",
    conversationId: "bot-a",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: resumedTarget.controlEpoch, expectedTaskId: resumedTarget.taskId,
    operationId: "lease-resumed-click",
    ref,
  });
  assert.equal(clicked.ok, true);
});

test("local Agent upload uses a trusted user selection and never exposes a local path to the page or tool request", async () => {
  const selectedPath = fileURLToPath(import.meta.url);
  let pickerCalls = 0;
  const h = harness(null, async () => {
    pickerCalls += 1;
    return selectedPath;
  });
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-upload", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents;
  contents.snapshotElements = [{
    selector: "html > body > input",
    tag: "input",
    role: null,
    name: "Attachment",
    text: "",
    href: null,
    inputType: "file",
    disabled: false,
    editable: true,
    checked: null,
    selected: null,
    frameUrl: null,
  }];
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-upload",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "upload-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const uploaded = await h.service.executeAgentAction({
    type: "upload-file",
    conversationId: "bot-upload",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "upload-file",
    ref: snap.snapshot!.elements[0]!.ref,
  });
  assert.equal(uploaded.ok, true);
  if (!uploaded.ok) return;
  assert.equal(pickerCalls, 1);
  assert.equal(uploaded.uploadedFile?.name, "browser-workspace.contract.test.js");
  assert.ok((uploaded.uploadedFile?.sizeBytes ?? 0) > 0);
  const script = contents.executedScripts.find((candidate) =>
    candidate.includes("new DataTransfer()") && candidate.includes("HTMLInputElement.prototype")
  );
  assert.ok(script);
  assert.equal(script.includes(selectedPath), false, "page script must never contain the selected local path");
});

test("local Agent upload cancellation is explicit and has no page side effect", async () => {
  const h = harness(null, async () => null);
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-upload-cancel", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const snap = await h.service.executeAgentAction({
    type: "snapshot",
    conversationId: "bot-upload-cancel",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "upload-cancel-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const beforeScripts = h.views[0]!.webContents.executedScripts.length;
  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "upload-file",
      conversationId: "bot-upload-cancel",
      workspaceId: state.workspaceId,
      tabId: tab.id,
      expectedGeneration: tab.generation,
      expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
      operationId: "upload-cancel",
      ref: snap.snapshot!.elements[0]!.ref,
    }),
    /BROWSER_UPLOAD_CANCELLED/,
  );
  assert.equal(h.views[0]!.webContents.executedScripts.length, beforeScripts);
});

test("local Agent download is lease-fenced, bounded, and returns integrity without exposing a path", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-download-agent", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents;
  const session = contents.session as FakeSession;
  let cancelled = false;
  const originalExecute = contents.executeJavaScript.bind(contents);
  contents.executeJavaScript = async (script: string): Promise<unknown> => {
    const result = await originalExecute(script);
    if (script.includes("element.click()")) {
      const listeners = new Map<"updated" | "done", (_event?: unknown, state?: string) => void>();
      const item: BrowserWorkspaceDownloadItem = {
        getURL: () => "https://example.com/export.bin", getFilename: () => "export.bin", getTotalBytes: () => 18, getReceivedBytes: () => 18,
        setSavePath: () => undefined, cancel: () => { cancelled = true; }, on: (event, listener) => { listeners.set(event, listener); },
      };
      session.download?.({}, item, contents);
      listeners.get("done")?.({}, "completed");
    }
    return result;
  };
  const snapshot = await h.service.executeAgentAction({ type: "snapshot", conversationId: "bot-download-agent", workspaceId: state.workspaceId, tabId: tab.id, expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "download-snapshot" });
  assert.equal(snapshot.ok, true);
  if (!snapshot.ok) return;
  const downloaded = await h.service.executeAgentAction({ type: "download", conversationId: "bot-download-agent", workspaceId: state.workspaceId, tabId: tab.id, expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "download-file", ref: snapshot.snapshot!.elements[0]!.ref });
  assert.equal(downloaded.ok, true);
  if (!downloaded.ok) return;
  assert.match(downloaded.downloadedFile!.downloadId, /^id-/);
  assert.equal(downloaded.downloadedFile!.name, "export.bin");
  assert.equal(downloaded.downloadedFile!.sizeBytes, Buffer.byteLength("synthetic-download"));
  assert.equal(downloaded.downloadedFile!.sha256, "synthetic-sha256");
  assert.equal(JSON.stringify(downloaded).includes("/downloads/"), false);
  assert.equal(cancelled, false);
});

test("takeover cancels an in-flight local Agent download and revokes the operation lease", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-download-cancel", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents;
  const session = contents.session as FakeSession;
  let signalDownload!: () => void;
  const downloadStarted = new Promise<void>((resolve) => { signalDownload = resolve; });
  let cancelled = false;
  const originalExecute = contents.executeJavaScript.bind(contents);
  contents.executeJavaScript = async (script: string): Promise<unknown> => {
    const result = await originalExecute(script);
    if (script.includes("element.click()")) {
      const item: BrowserWorkspaceDownloadItem = {
        getURL: () => "https://example.com/slow.bin",
        getFilename: () => "slow.bin",
        getTotalBytes: () => 1024,
        getReceivedBytes: () => 1,
        setSavePath: () => undefined,
        cancel: () => { cancelled = true; },
        on: () => undefined,
      };
      session.download?.({}, item, contents);
      signalDownload();
    }
    return result;
  };
  const snapshot = await h.service.executeAgentAction({
    type: "snapshot", conversationId: "bot-download-cancel", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "cancel-snapshot",
  });
  assert.equal(snapshot.ok, true);
  if (!snapshot.ok) return;
  const pending = h.service.executeAgentAction({
    type: "download", conversationId: "bot-download-cancel", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId,
    operationId: "cancel-download", ref: snapshot.snapshot!.elements[0]!.ref,
  });
  await downloadStarted;
  const taken = await h.invoke({ type: "agent-control", conversationId: "bot-download-cancel", action: "takeover" });
  assert.equal(taken.agentControl.state, "taken-over");
  await assert.rejects(() => pending, /BROWSER_AGENT_CONTROL_STALE/);
  assert.equal(cancelled, true, "takeover must cancel the owned in-flight download");
});

test("workspace agent actions serialize and takeover invalidates queued mutations before side effects", async () => {
  let releasePicker!: (path: string | null) => void;
  let pickerStarted!: () => void;
  const pickerStartedPromise = new Promise<void>((resolve) => { pickerStarted = resolve; });
  const pickerPromise = new Promise<string | null>((resolve) => { releasePicker = resolve; });
  const selectedPath = fileURLToPath(import.meta.url);
  const h = harness(null, async () => {
    pickerStarted();
    return await pickerPromise;
  });
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-queue", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents;
  contents.snapshotElements = [{
    selector: "html > body > input",
    tag: "input",
    role: null,
    name: "Attachment",
    text: "",
    href: null,
    inputType: "file",
    disabled: false,
    editable: true,
    checked: null,
    selected: null,
    frameUrl: null,
  }];
  const snap = await h.service.executeAgentAction({
    type: "snapshot", conversationId: "bot-queue", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "queue-snapshot",
  });
  assert.equal(snap.ok, true);
  if (!snap.ok) return;
  const ref = snap.snapshot!.elements[0]!.ref;
  const upload = h.service.executeAgentAction({
    type: "upload-file", conversationId: "bot-queue", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "queue-upload", ref,
  });
  await pickerStartedPromise;
  const scriptsBeforeQueued = contents.executedScripts.length;
  const queuedClick = h.service.executeAgentAction({
    type: "click", conversationId: "bot-queue", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch, expectedTaskId: state.agentControl.taskId, operationId: "queue-click", ref,
  });
  await Promise.resolve();
  assert.equal(contents.executedScripts.length, scriptsBeforeQueued, "queued action must not execute while the prior operation owns the workspace lane");

  const taken = await h.invoke({ type: "agent-control", conversationId: "bot-queue", action: "takeover" });
  assert.equal(taken.agentControl.state, "taken-over");
  releasePicker(selectedPath);
  await assert.rejects(() => upload, /BROWSER_AGENT_CONTROL_STALE/);
  await assert.rejects(() => queuedClick, /BROWSER_AGENT_CONTROL_STALE/);
  assert.equal(
    contents.executedScripts.some((script) => script.includes("element.click()")),
    false,
    "a queued mutation from the revoked lease must never reach the page",
  );
});

test("takeover plus explicit reauthorization rotates task identity and rejects the old task lease", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-task", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const first = await h.service.executeAgentAction({
    type: "snapshot", conversationId: "bot-task", workspaceId: state.workspaceId, tabId: tab.id,
    expectedGeneration: tab.generation, expectedControlEpoch: state.agentControl.epoch,
    expectedTaskId: state.agentControl.taskId, operationId: "task-first",
  });
  assert.equal(first.ok, true);
  if (!first.ok) return;
  const taken = await h.invoke({ type: "agent-control", conversationId: "bot-task", action: "takeover" });
  assert.equal(taken.agentControl.taskStatus, "taken-over");
  const allowed = await h.invoke({ type: "agent-control", conversationId: "bot-task", action: "resume" });
  assert.equal(allowed.agentControl.taskStatus, "idle");
  assert.notEqual(allowed.agentControl.taskId, first.taskId);

  await assert.rejects(
    () => h.service.executeAgentAction({
      type: "snapshot", conversationId: "bot-task", workspaceId: state.workspaceId, tabId: tab.id,
      expectedGeneration: tab.generation, expectedControlEpoch: allowed.agentControl.epoch,
      expectedTaskId: first.taskId, operationId: "task-stale",
    }),
    /BROWSER_AGENT_TASK_STALE/,
  );
});

test("unpacked browser extension requires user confirmation, enforces integrity, and supports disable update uninstall", async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-extension-"));
  try {
    writeFileSync(join(root, "manifest.json"), JSON.stringify({
      manifest_version: 3,
      name: "Fabushi Test Extension",
      version: "1.0.0",
      permissions: ["storage"],
      host_permissions: ["https://example.com/*"],
      content_scripts: [{ matches: ["https://example.com/*"], js: ["content.js"] }],
    }));
    writeFileSync(join(root, "content.js"), "document.documentElement.dataset.fabushiExtension = 'loaded';");

    const h = harness(null, undefined, async () => root);
    h.service.attachWindow(new FakeHostWindow());
    const state = await h.invoke({ type: "activate", conversationId: "bot-extension", bounds: { x: 1, y: 1, width: 500, height: 500 } });
    const candidate = await h.invoke({ type: "extension-choose", conversationId: "bot-extension" });
    assert.equal(candidate.extensions.length, 0);
    assert.equal(candidate.extensionCandidate?.name, "Fabushi Test Extension");
    assert.deepEqual(candidate.extensionCandidate?.permissions, ["storage"]);
    assert.deepEqual(candidate.extensionCandidate?.hostPermissions, ["https://example.com/*"]);
    assert.equal(JSON.stringify(candidate).includes(root), false, "renderer snapshot must not expose extension source path");
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 0, "choosing must not load before confirmation");

    const installed = await h.invoke({
      type: "extension-install",
      conversationId: "bot-extension",
      candidateId: candidate.extensionCandidate!.candidateId,
    });
    assert.equal(installed.extensions.length, 1);
    assert.equal(installed.extensions[0]?.enabled, true);
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 1);

    const disabled = await h.invoke({
      type: "extension-set-enabled",
      conversationId: "bot-extension",
      extensionKey: installed.extensions[0]!.key,
      enabled: false,
    });
    assert.equal(disabled.extensions[0]?.enabled, false);
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 0);

    writeFileSync(join(root, "content.js"), "document.documentElement.dataset.fabushiExtension = 'updated';");
    await assert.rejects(
      () => h.invoke({
        type: "extension-set-enabled",
        conversationId: "bot-extension",
        extensionKey: installed.extensions[0]!.key,
        enabled: true,
      }),
      /BROWSER_EXTENSION_INTEGRITY_CHANGED/,
    );

    const updated = await h.invoke({
      type: "extension-update",
      conversationId: "bot-extension",
      extensionKey: installed.extensions[0]!.key,
    });
    assert.notEqual(updated.extensions[0]?.integrity, installed.extensions[0]?.integrity);
    assert.equal(updated.extensions[0]?.enabled, false);

    const enabled = await h.invoke({
      type: "extension-set-enabled",
      conversationId: "bot-extension",
      extensionKey: installed.extensions[0]!.key,
      enabled: true,
    });
    assert.equal(enabled.extensions[0]?.enabled, true);
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 1);

    const removed = await h.invoke({
      type: "extension-remove",
      conversationId: "bot-extension",
      extensionKey: installed.extensions[0]!.key,
    });
    assert.equal(removed.extensions.length, 0);
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 0);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("browser extension policy rejects privileged permissions before Electron load", async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-extension-reject-"));
  try {
    writeFileSync(join(root, "manifest.json"), JSON.stringify({
      manifest_version: 3,
      name: "Fabushi Test Extension",
      version: "1.0.0",
      permissions: ["nativeMessaging"],
    }));
    const h = harness(null, undefined, async () => root);
    h.service.attachWindow(new FakeHostWindow());
    await h.invoke({ type: "activate", conversationId: "bot-extension-reject", bounds: { x: 1, y: 1, width: 500, height: 500 } });
    await assert.rejects(
      () => h.invoke({ type: "extension-choose", conversationId: "bot-extension-reject" }),
      /BROWSER_EXTENSION_PERMISSION_UNSUPPORTED:nativeMessaging/,
    );
    assert.equal(h.sessions.values().next().value?.loadedExtensions.size ?? 0, 0);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("download navigation accepts Chromium ERR_FAILED only after the owned session confirms a download", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const initial = await h.invoke({ type: "activate", conversationId: "bot-download", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const contents = h.views[0]!.webContents;
  const downloadUrl = "https://example.com/export.bin";

  contents.loadURL = async (url: string): Promise<unknown> => {
    contents.emit("did-start-loading");
    const listeners = new Map<"updated" | "done", (_event?: unknown, state?: string) => void>();
    const item: BrowserWorkspaceDownloadItem = {
      getURL: () => url,
      getFilename: () => "export.bin",
      getTotalBytes: () => 7,
      getReceivedBytes: () => 7,
      setSavePath: () => undefined,
      cancel: () => undefined,
      on: (event, listener) => { listeners.set(event, listener); },
    };
    contents.session.download?.({}, item, contents);
    contents.emit("did-fail-load", {}, -2, "ERR_FAILED", url, true);
    listeners.get("done")?.({}, "completed");
    throw Object.assign(new Error(`ERR_FAILED (-2) loading '${url}'`), { code: "ERR_FAILED", url });
  };

  const downloaded = await h.invoke({
    type: "navigate",
    conversationId: "bot-download",
    tabId: initial.activeTabId,
    address: downloadUrl,
  });
  assert.equal(downloaded.downloads.length, 1);
  assert.equal(downloaded.downloads[0]?.fileName, "export.bin");
  assert.equal(downloaded.downloads[0]?.state, "completed");
  assert.equal(h.persistence.paths.length, 1);

  contents.loadURL = async (url: string): Promise<unknown> => {
    throw Object.assign(new Error(`ERR_FAILED (-2) loading '${url}'`), { code: "ERR_FAILED", url });
  };
  await assert.rejects(
    () => h.invoke({
      type: "navigate",
      conversationId: "bot-download",
      tabId: initial.activeTabId,
      address: "https://example.com/not-a-download",
    }),
    /ERR_FAILED/,
  );
});

test("invalid commands are rejected before any view is created", async () => {
  const h = harness();
  assert.ok(h.ipc.handler);
  await assert.rejects(
    async () => await h.ipc.handler!({ sender: h.trusted, senderFrame: h.trusted.mainFrame }, { type: "activate", conversationId: "", bounds: {} }),
    /BROWSER_INVALID_COMMAND/,
  );
  assert.equal(h.views.length, 0);
});


test("Agent JavaScript dialogs use only the fixed Main-owned CDP handler and are control-fenced", async () => {
  const h = harness();
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-dialog", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents as FakeContents;
  await new Promise((resolve) => setTimeout(resolve, 0));
  const target = {
    conversationId: "bot-dialog",
    workspaceId: state.workspaceId,
    tabId: tab.id,
    expectedGeneration: tab.generation,
    expectedControlEpoch: state.agentControl.epoch,
    expectedTaskId: state.agentControl.taskId,
  };

  contents.debugger.emitMessage("Page.javascriptDialogOpening", {
    type: "prompt",
    message: "Enter a one-time value",
    url: "https://example.com/login?secret=must-not-be-recorded",
    defaultPrompt: "",
  });
  const accepted = await h.service.executeAgentAction({
    ...target, type: "dialog", decision: "accept", promptText: "approved", operationId: "dialog-accept",
  });
  assert.equal(accepted.ok, true);
  if (accepted.ok) {
    assert.deepEqual(accepted.dialogResult, { type: "prompt", decision: "accept" });
    assert.equal(accepted.dialog, null);
    assert.equal(JSON.stringify(accepted).includes("must-not-be-recorded"), false);
  }
  assert.deepEqual(contents.debugger.commands.filter((command) => command.method === "Page.handleJavaScriptDialog").at(-1), {
    method: "Page.handleJavaScriptDialog",
    params: { accept: true, promptText: "approved" },
  });

  contents.debugger.emitMessage("Page.javascriptDialogOpening", {
    type: "confirm", message: "Continue?", url: "https://example.com/confirm",
  });
  const dismissed = await h.service.executeAgentAction({
    ...target, type: "dialog", decision: "dismiss", operationId: "dialog-dismiss",
  });
  assert.equal(dismissed.ok, true);
  assert.deepEqual(contents.debugger.commands.filter((command) => command.method === "Page.handleJavaScriptDialog").at(-1), {
    method: "Page.handleJavaScriptDialog",
    params: { accept: false },
  });

  await assert.rejects(() => h.service.executeAgentAction({
    ...target, type: "dialog", decision: "accept", promptText: "x".repeat(LOCAL_BROWSER_AGENT_LIMITS.maxPromptTextLength + 1), operationId: "dialog-too-long",
  }), /BROWSER_AGENT_INVALID_ACTION/);

  contents.debugger.emitMessage("Page.javascriptDialogOpening", {
    type: "alert", message: "Do not accept after takeover", url: "https://example.com/",
  });
  await h.invoke({ type: "agent-control", conversationId: "bot-dialog", action: "takeover" });
  const commandCount = contents.debugger.commands.filter((command) => command.method === "Page.handleJavaScriptDialog").length;
  await assert.rejects(() => h.service.executeAgentAction({
    ...target, type: "dialog", decision: "accept", operationId: "dialog-stale-control",
  }), /BROWSER_AGENT_CONTROL_STALE/);
  assert.equal(contents.debugger.commands.filter((command) => command.method === "Page.handleJavaScriptDialog").length, commandCount);
  h.service.dispose();
});

test("page/load/network-idle events have bounded cursors, origin-only evidence, and finite waits", async () => {
  let clock = 0;
  const h = harness(null, undefined, undefined, () => 1_700_000_000_000 + (clock += 100));
  h.service.attachWindow(new FakeHostWindow());
  const state = await h.invoke({ type: "activate", conversationId: "bot-events", bounds: { x: 1, y: 1, width: 500, height: 500 } });
  const tab = state.tabs[0]!;
  const contents = h.views[0]!.webContents as FakeContents;
  await new Promise((resolve) => setTimeout(resolve, 0));
  await h.invoke({ type: "navigate", conversationId: "bot-events", tabId: tab.id, address: "https://example.com/start" });
  contents.debugger.emitMessage("Network.requestWillBeSent", {
    requestId: "request-1",
    type: "Fetch",
    request: { url: "https://example.com/api?token=private" },
  });
  contents.debugger.emitMessage("Network.loadingFinished", { requestId: "request-1" });

  const initialState = h.service.snapshot("bot-events")!;
  const initial = await h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: initialState.workspaceId, tabId: tab.id,
    expectedGeneration: initialState.tabs[0]!.generation,
    expectedControlEpoch: initialState.agentControl.epoch,
    expectedTaskId: initialState.agentControl.taskId,
    operationId: "events-initial", type: "events", afterSequence: 0, limit: 32,
  });
  assert.equal(initial.ok, true);
  if (!initial.ok) return;
  assert.ok(initial.events?.some((event) => event.type === "page"));
  assert.ok(initial.events?.some((event) => event.type === "load"));
  assert.ok(initial.events?.some((event) => event.type === "network-request"));
  assert.ok(initial.events?.some((event) => event.type === "network-finished"));
  assert.ok((initial.events?.length ?? 0) <= LOCAL_BROWSER_AGENT_LIMITS.maxEventBatch);
  assert.equal(JSON.stringify(initial).includes("token=private"), false);

  for (let index = 0; index < 80; index += 1) {
    contents.emit("did-navigate", {}, `https://example.com/page/${index}?token=private`);
  }
  const boundedState = h.service.snapshot("bot-events")!;
  const bounded = await h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: boundedState.workspaceId, tabId: tab.id,
    expectedGeneration: boundedState.tabs[0]!.generation,
    expectedControlEpoch: boundedState.agentControl.epoch,
    expectedTaskId: boundedState.agentControl.taskId,
    operationId: "events-bounded", type: "events", afterSequence: 0, limit: 32,
  });
  assert.equal(bounded.ok, true);
  if (!bounded.ok) return;
  assert.ok((bounded.events?.length ?? 0) <= 32);
  assert.ok((bounded.eventSequence ?? 0) > LOCAL_BROWSER_AGENT_LIMITS.maxEvents);
  assert.equal(JSON.stringify(bounded).includes("token=private"), false);
  assert.ok(bounded.events?.every((event) => event.urlOrigin === "https://example.com"));

  const cursor = bounded.eventSequence!;
  const waited = await h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: boundedState.workspaceId, tabId: tab.id,
    expectedGeneration: boundedState.tabs[0]!.generation,
    expectedControlEpoch: boundedState.agentControl.epoch,
    expectedTaskId: boundedState.agentControl.taskId,
    operationId: "events-wait-page", type: "wait-event", event: "page", afterSequence: cursor - 1, timeoutMs: 500,
  });
  assert.equal(waited.ok, true);
  if (waited.ok) assert.equal(waited.event?.type, "page");

  const latest = h.service.snapshot("bot-events")!;
  const latestGeneration = latest.tabs[0]!.generation;
  const latestSequence = bounded.eventSequence!;
  await assert.rejects(() => h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: latest.workspaceId, tabId: tab.id,
    expectedGeneration: latestGeneration, expectedControlEpoch: latest.agentControl.epoch,
    expectedTaskId: latest.agentControl.taskId, operationId: "events-invalid-cursor", type: "wait-event",
    event: "page", afterSequence: latestSequence + 1, timeoutMs: 100,
  }), /BROWSER_EVENT_CURSOR_INVALID/);
  await assert.rejects(() => h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: latest.workspaceId, tabId: tab.id,
    expectedGeneration: latestGeneration, expectedControlEpoch: latest.agentControl.epoch,
    expectedTaskId: latest.agentControl.taskId, operationId: "events-timeout", type: "wait-event",
    event: "page", afterSequence: latestSequence, timeoutMs: 100,
  }), /BROWSER_EVENT_WAIT_TIMEOUT/);
  await assert.rejects(() => h.service.executeAgentAction({
    conversationId: "bot-events", workspaceId: latest.workspaceId, tabId: tab.id,
    expectedGeneration: latestGeneration, expectedControlEpoch: latest.agentControl.epoch,
    expectedTaskId: latest.agentControl.taskId, operationId: "events-timeout-bounded", type: "wait-event",
    event: "page", afterSequence: latestSequence, timeoutMs: LOCAL_BROWSER_AGENT_LIMITS.maxWaitMs + 1,
  }), /BROWSER_AGENT_INVALID_ACTION/);
  h.service.dispose();
});
