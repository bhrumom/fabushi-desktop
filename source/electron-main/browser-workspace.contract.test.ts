import assert from "node:assert/strict";
import test from "node:test";
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
import {
  BROWSER_WORKSPACE_INVOKE_CHANNEL,
  normalizeBrowserWorkspaceAddress,
  type BrowserWorkspaceBounds,
  type BrowserWorkspaceCommand,
  type BrowserWorkspaceSnapshot,
} from "../shared/browser-workspace.js";

class FakeSession implements BrowserWorkspaceSessionPort {
  permissionRequest: ((contents: BrowserWorkspaceWebContentsPort, permission: string, callback: (allowed: boolean) => void) => void) | null = null;
  permissionCheck: ((contents: BrowserWorkspaceWebContentsPort, permission: string) => boolean) | null = null;
  download: ((event: unknown, item: BrowserWorkspaceDownloadItem, contents: BrowserWorkspaceWebContentsPort) => void) | null = null;
  setPermissionRequestHandler(handler: FakeSession["permissionRequest"]): void { this.permissionRequest = handler; }
  setPermissionCheckHandler(handler: FakeSession["permissionCheck"]): void { this.permissionCheck = handler; }
  on(_event: "will-download", listener: NonNullable<FakeSession["download"]>): void { this.download = listener; }
}

class FakeContents implements BrowserWorkspaceWebContentsPort {
  readonly mainFrame = {};
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
    if (script.includes("MAX_ELEMENTS") && script.includes("document.querySelectorAll")) {
      return {
        url: this.url,
        title: this.title,
        text: "Synthetic page Submit",
        elements: [{
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
        }],
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
}

function harness(initialState: unknown = null) {
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
    broadcast: (_channel, payload) => broadcasts.push(payload),
    id: () => `id-${++id}`,
    now: () => 1_700_000_000_000 + id,
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
    expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: state.agentControl.epoch,
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
      expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: state.agentControl.epoch,
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
      expectedControlEpoch: state.agentControl.epoch,
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
      expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: state.agentControl.epoch,
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
      expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: resolved.controlEpoch,
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
    expectedControlEpoch: resolved.controlEpoch,
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
    expectedControlEpoch: resolved.controlEpoch,
    tabId: newTab!.tabId,
  });
  assert.equal(closed.ok, true);
  if (!closed.ok) return;
  assert.equal(closed.tabs?.length, 1);
  assert.equal(closed.tabs?.[0]?.tabId, state.activeTabId);
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
    expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: state.agentControl.epoch,
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
      expectedControlEpoch: state.agentControl.epoch,
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
    expectedControlEpoch: resolved.controlEpoch,
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
      expectedControlEpoch: resolved.controlEpoch,
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
      expectedControlEpoch: fresh.controlEpoch,
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
    expectedControlEpoch: resumedTarget.controlEpoch,
    operationId: "lease-resumed-click",
    ref,
  });
  assert.equal(clicked.ok, true);
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
