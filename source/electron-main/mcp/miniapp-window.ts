import { createHash } from "node:crypto";
import type { MiniAppToolRouter, MiniAppToolScope } from "./miniapp-tools.js";
import { MINIAPP_TOOL_CHANNEL } from "../../shared/miniapp-tool-contract.js";

export const FABUSHI_MINI_APP_SCHEME = "fabushi-miniapp";
/** Must stay byte-identical to source/electron-preload/preload-miniapp-tools.ts. */
const MAX_RESOURCE_BYTES = 250 * 1024 * 1024;
const MAX_RESOURCE_COUNT = 10_000;
const MAX_REPORT_BYTES = 1024 * 1024;
const OFFLINE_PROXY = "http=127.0.0.1:9;https=127.0.0.1:9;socks5=127.0.0.1:9";

export interface PrivilegedSchemeDeclaration { readonly scheme: string; readonly privileges: Readonly<Record<string, boolean>> }
/** Add this declaration to Electron's single global registerSchemesAsPrivileged call. */
export const FABUSHI_MINI_APP_PRIVILEGED_SCHEME: PrivilegedSchemeDeclaration = Object.freeze({
  scheme: FABUSHI_MINI_APP_SCHEME,
  privileges: Object.freeze({ standard: true, secure: true, supportFetchAPI: true, corsEnabled: false, allowServiceWorkers: false }),
});

export interface VerifiedMiniAppResource {
  readonly bytes: Uint8Array;
  readonly mime: string;
}

/** Constructed only from the existing Rust Host's freshly reverified launch result. */
export interface VerifiedMiniAppLaunch {
  readonly appId: string;
  readonly version: string;
  readonly packageDigest: string;
  readonly entrypoint: string;
  /** Stable pseudonymous native account scope, never an access token or renderer value. */
  readonly accountScope: string;
  /** Monotonic generation supplied by the existing native account owner. */
  readonly accountGeneration: number;
  readonly resources: ReadonlyMap<string, VerifiedMiniAppResource>;
}

export interface MiniAppWindow {
  readonly webContents: MiniAppWebContents;
  isDestroyed(): boolean;
  show(): void;
  focus(): void;
  close(): void;
  destroy(): void;
  on(event: "closed", listener: () => void): void;
}

export interface MiniAppWebContents {
  readonly session: MiniAppSession;
  setWebRTCIPHandlingPolicy(policy: "default_public_interface_only" | "default_public_and_private_interfaces" | "disable_non_proxied_udp"): Promise<void>;
  setWindowOpenHandler(handler: () => { action: "deny" }): void;
  on(event: "will-navigate", listener: (event: { preventDefault(): void }, url: string) => void): void;
  on(event: "will-redirect", listener: (event: { preventDefault(): void }, url: string) => void): void;
  on(event: "will-attach-webview", listener: (event: { preventDefault(): void }) => void): void;
  on(event: "render-process-gone", listener: () => void): void;
  loadURL(url: string): Promise<unknown>;
}

export interface MiniAppSession {
  readonly protocol: {
    handle(scheme: string, handler: (request: Request) => Response | Promise<Response>): void | Promise<void>;
    unhandle(scheme: string): void;
  };
  readonly webRequest: {
    onBeforeRequest(filter: { urls: string[] }, listener: ((details: { url: string }, callback: (result: { cancel: boolean }) => void) => void) | null): void;
  };
  setProxy(config: { mode: "fixed_servers"; proxyRules: string; proxyBypassRules: string }): Promise<void>;
  setPermissionRequestHandler(handler: ((webContents: MiniAppWebContents, permission: string, callback: (allowed: boolean) => void) => void) | null): void;
  setPermissionCheckHandler(handler: ((webContents: MiniAppWebContents, permission: string) => boolean) | null): void;
  on(event: "will-download", listener: (event: { preventDefault(): void }, item: MiniAppDownloadItem, contents: MiniAppWebContents) => void): void;
  removeListener(event: "will-download", listener: (event: { preventDefault(): void }, item: MiniAppDownloadItem, contents: MiniAppWebContents) => void): void;
}

export interface MiniAppDownloadItem {
  getURL(): string;
  getMimeType(): string;
  getFilename(): string;
  getTotalBytes(): number;
  getReceivedBytes(): number;
  hasUserGesture(): boolean;
  cancel(): void;
  on(event: "updated" | "done", listener: () => void): void;
}

export interface MiniAppWindowOptions {
  readonly width: number;
  readonly height: number;
  readonly minWidth: number;
  readonly minHeight: number;
  readonly show: false;
  readonly title: string;
  readonly autoHideMenuBar: true;
  readonly webPreferences: {
    readonly partition: string;
    readonly nodeIntegration: false;
    readonly contextIsolation: true;
    readonly sandbox: true;
    readonly webSecurity: true;
    readonly webviewTag: false;
    readonly preload?: string;
  };
}

export interface MiniAppIpcEvent {
  readonly sender: MiniAppWebContents;
  readonly senderFrame?: { readonly url: string } | null;
}

export interface MiniAppIpc {
  handle(channel: string, listener: (event: MiniAppIpcEvent, tool: unknown, args: unknown) => Promise<unknown>): void;
  removeHandler?(channel: string): void;
}

export interface MiniAppToolBridgePorts {
  readonly appId?: string;
  readonly router: MiniAppToolRouter;
  /** Absolute path to the compiled sandboxed preload. */
  readonly preloadPath: string;
  readonly ipc: MiniAppIpc;
  /** Called when account ownership changes or the Host is disposed. */
  readonly cancelAll?: () => void;
}

export interface MiniAppWindowFactoryPorts {
  readonly BrowserWindow: new(options: MiniAppWindowOptions) => MiniAppWindow;
  readonly session: { fromPartition(partition: string): MiniAppSession };
  /** Existing native Auth/Account owner, based on getStatus().authId. */
  readonly currentAccount: () => { readonly accountScope: string; readonly generation: number } | null;
  readonly showError?: (error: "MINIAPP_STALE_ACCOUNT" | "MINIAPP_ALREADY_OPEN" | "MINIAPP_INVALID_PACKAGE") => void;
  /** Omit to keep the window render-only (current default). */
  readonly tools?: MiniAppToolBridgePorts;
}

export interface MiniAppWindowFactory {
  open(launch: VerifiedMiniAppLaunch, tools?: MiniAppToolBridgePorts): Promise<{ close(): void }>;
  closeAccountScope(accountScope: string, nextGeneration: number): void;
  closeAll(): void;
}

function denyResponse(status = 404): Response {
  return new Response(null, { status, headers: { "cache-control": "no-store", "content-security-policy": "default-src 'none'" } });
}

function safePath(path: string): boolean {
  if (!path || path.startsWith("/") || path.includes("\\") || path.includes("\0")) return false;
  return path.split("/").every((part) => part.length > 0 && part !== "." && part !== ".." && /^[A-Za-z0-9._-]+$/.test(part));
}

function identityHash(accountScope: string, appId: string): string {
  return createHash("sha256").update(accountScope).update("\0").update(appId).digest("hex");
}

function matchesOrigin(url: URL, origin: string): boolean {
  const expected = new URL(origin);
  return url.protocol === expected.protocol && url.hostname === expected.hostname
    && url.port === "" && url.username === "" && url.password === "";
}

function snapshotResources(launch: VerifiedMiniAppLaunch): Map<string, VerifiedMiniAppResource> {
  if (!/^[a-z0-9][a-z0-9.-]{2,127}$/.test(launch.appId) || !/^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/.test(launch.version) || !/^[a-f0-9]{64}$/.test(launch.packageDigest)
    || typeof launch.accountScope !== "string" || !/^[a-f0-9]{64}$/.test(launch.accountScope)
    || !Number.isSafeInteger(launch.accountGeneration) || launch.accountGeneration < 0 || !safePath(launch.entrypoint)
    || !(launch.resources instanceof Map) || launch.resources.size === 0 || launch.resources.size > MAX_RESOURCE_COUNT) throw new TypeError("MINIAPP_INVALID_PACKAGE");
  const resources = new Map<string, VerifiedMiniAppResource>();
  let totalBytes = 0;
  for (const [path, resource] of launch.resources) {
    if (!safePath(path) || !(resource.bytes instanceof Uint8Array) || !Number.isSafeInteger(resource.bytes.byteLength)
      || !/^(text\/(?:html|css|javascript|plain)|application\/(?:javascript|wasm|json)|image\/(?:png|jpeg|gif|webp|svg\+xml)|font\/(?:woff2?|ttf|otf))$/.test(resource.mime)) throw new TypeError("MINIAPP_INVALID_PACKAGE");
    totalBytes += resource.bytes.byteLength;
    if (totalBytes > MAX_RESOURCE_BYTES) throw new TypeError("MINIAPP_INVALID_PACKAGE");
    resources.set(path, Object.freeze({ bytes: new Uint8Array(resource.bytes), mime: resource.mime }));
  }
  if (!resources.has(launch.entrypoint)) throw new TypeError("MINIAPP_INVALID_PACKAGE");
  return resources;
}

function makeResourceHandler(origin: string, entrypoint: string, resources: ReadonlyMap<string, VerifiedMiniAppResource>) {
  const csp = [
    "default-src 'none'", "base-uri 'none'", "object-src 'none'", "frame-src 'none'", "child-src 'none'",
    "worker-src 'none'", "connect-src 'self'", "form-action 'none'", "script-src 'self' 'wasm-unsafe-eval'",
    "style-src 'self'", "img-src 'self'", "font-src 'self'", "media-src 'self'",
  ].join("; ");
  return (request: Request): Response => {
    let url: URL;
    try { url = new URL(request.url); } catch { return denyResponse(400); }
    if (!matchesOrigin(url, origin) || !["GET", "HEAD"].includes(request.method)) return denyResponse();
    let path: string;
    try { path = decodeURIComponent(url.pathname.replace(/^\//, "") || entrypoint); } catch { return denyResponse(400); }
    if (!safePath(path)) return denyResponse(400);
    const resource = resources.get(path);
    if (!resource) return denyResponse();
    const headers = new Headers({
      "content-type": resource.mime,
      "content-length": String(resource.bytes.byteLength),
      "cache-control": "no-store",
      "x-content-type-options": "nosniff",
      "content-security-policy": csp,
      "cross-origin-resource-policy": "same-origin",
      "referrer-policy": "no-referrer",
    });
    return new Response(request.method === "HEAD" ? null : new Uint8Array(resource.bytes), { status: 200, headers });
  };
}

function isAllowedReportExport(appId: string, item: MiniAppDownloadItem, contents: MiniAppWebContents, owner: MiniAppWebContents, origin: string): boolean {
  try {
    const blob = new URL(item.getURL());
    const filename = item.getFilename();
    const expectedFilename = appId === "org.fabushi.global-dandelion"
      ? "global-dandelion-local-report.json"
      : appId === "org.fabushi.chatgpt-auto-confirm"
        ? "chatgpt-auto-confirm-local-report.json"
        : null;
    const compatibleFilename = filename === expectedFilename
      || (appId === "org.fabushi.global-dandelion" && filename === "global-dandelion-session.json");
    return expectedFilename !== null && compatibleFilename && contents === owner && item.hasUserGesture() && blob.protocol === "blob:" && matchesOrigin(new URL(blob.pathname), origin)
      && item.getMimeType().split(";")[0]?.trim().toLowerCase() === "application/json"
      && filename === expectedFilename
      // Chromium may report zero for a blob download before the first update;
      // the updated handler enforces the bounded size once bytes arrive.
      && item.getTotalBytes() >= 0 && item.getTotalBytes() <= MAX_REPORT_BYTES
      && item.getReceivedBytes() >= 0 && item.getReceivedBytes() <= MAX_REPORT_BYTES;
  } catch { return false; }
}

export function createMiniAppWindowFactory(ports: MiniAppWindowFactoryPorts): MiniAppWindowFactory {
  const windows = new Map<string, { toolScope: MiniAppToolScope; tools?: MiniAppToolBridgePorts; origin: string; window: MiniAppWindow; disposeSession(): void }>();
  const minimumGeneration = new Map<string, number>();
  const opening = new Set<string>();
  const keyFor = (scope: string, appId: string) => identityHash(scope, appId);

  let toolBridgeInstalled = false;
  const ensureToolBridge = (tools?: MiniAppToolBridgePorts): void => {
    if (!tools || toolBridgeInstalled) return;
    toolBridgeInstalled = true;
    const denied = { ok: false, error: { code: "UNAUTHORIZED_SENDER", message: "该窗口无权调用宿主工具" } };
    tools.ipc.handle(MINIAPP_TOOL_CHANNEL, async (event, tool, args) => {
      for (const record of windows.values()) {
        if (record.window.webContents !== event.sender) continue;
        if (!record.tools || (record.tools.appId && record.tools.appId !== record.toolScope.appId)) return denied;
        const current = ports.currentAccount();
        if (!current || current.accountScope !== record.toolScope.accountScope
          || current.generation !== record.toolScope.accountGeneration) return denied;
        let frameUrl: URL;
        try { frameUrl = new URL(event.senderFrame?.url ?? ""); } catch { return denied; }
        if (!matchesOrigin(frameUrl, record.origin)) return denied;
        return record.tools.router.invoke(record.toolScope, tool, args, () => {
          const latest = ports.currentAccount();
          return [...windows.values()].includes(record)
            && latest?.accountScope === record.toolScope.accountScope
            && latest.generation === record.toolScope.accountGeneration;
        });
      }
      return denied;
    });
  };

  const closeKey = (key: string): void => {
    const record = windows.get(key);
    if (!record) return;
    windows.delete(key); // Fence callbacks before Electron tears down the renderer.
    if (!record.tools?.appId || record.tools.appId === record.toolScope.appId) record.tools?.cancelAll?.();
    if (!record.window.isDestroyed()) record.window.destroy();
    record.disposeSession();
  };

  return {
    async open(launch, toolOverride) {
      const tools = toolOverride ?? ports.tools;
      const account = ports.currentAccount();
      const floor = minimumGeneration.get(launch.accountScope) ?? 0;
      if (!account || account.accountScope !== launch.accountScope || account.generation !== launch.accountGeneration || launch.accountGeneration < floor) {
        ports.showError?.("MINIAPP_STALE_ACCOUNT");
        throw new Error("MINIAPP_STALE_ACCOUNT");
      }
      const resources = snapshotResources(launch);
      const key = keyFor(launch.accountScope, launch.appId);
      const prior = windows.get(key);
      if (prior && !prior.window.isDestroyed()) {
        ports.showError?.("MINIAPP_ALREADY_OPEN");
        prior.window.focus();
        throw new Error("MINIAPP_ALREADY_OPEN");
      }
      if (opening.has(key)) throw new Error("MINIAPP_ALREADY_OPEN");
      opening.add(key);

      let cleanupPartialSession = () => {};
      try {
      const partition = `persist:fabushi-miniapp-${key}`;
      const session = ports.session.fromPartition(partition);
      const origin = `${FABUSHI_MINI_APP_SCHEME}://${key}.app`;
      const toolScope: MiniAppToolScope = {
        appId: launch.appId, version: launch.version, packageDigest: launch.packageDigest,
        accountScope: launch.accountScope, accountGeneration: launch.accountGeneration,
      };
      const handler = makeResourceHandler(origin, launch.entrypoint, resources);
      let protocolRegistered = false;
      let requestListener: ((details: { url: string }, callback: (result: { cancel: boolean }) => void) => void) | undefined;
      let permissionInstalled = false;
      let handleDownload: ((event: { preventDefault(): void }, item: MiniAppDownloadItem, contents: MiniAppWebContents) => void) | undefined;
      let activeReport: MiniAppDownloadItem | undefined;
      let disposed = false;
      cleanupPartialSession = () => {
        if (disposed) return;
        disposed = true;
        if (activeReport) { activeReport.cancel(); activeReport = undefined; }
        if (requestListener) session.webRequest.onBeforeRequest({ urls: ["<all_urls>"] }, (_details, callback) => callback({ cancel: true }));
        if (permissionInstalled) {
          session.setPermissionRequestHandler((_contents, _permission, callback) => callback(false));
          session.setPermissionCheckHandler(() => false);
        }
        if (handleDownload) session.removeListener("will-download", handleDownload);
        if (protocolRegistered) session.protocol.unhandle(FABUSHI_MINI_APP_SCHEME);
      };
      await session.setProxy({ mode: "fixed_servers", proxyRules: OFFLINE_PROXY, proxyBypassRules: "<-loopback>" });
      const afterProxy = ports.currentAccount();
      if (afterProxy?.accountScope !== launch.accountScope || afterProxy.generation !== launch.accountGeneration || launch.accountGeneration < (minimumGeneration.get(launch.accountScope) ?? 0)) throw new Error("MINIAPP_STALE_ACCOUNT");
      await session.protocol.handle(FABUSHI_MINI_APP_SCHEME, handler);
      protocolRegistered = true;
      requestListener = (details: { url: string }, callback: (result: { cancel: boolean }) => void) => {
        let allowed = false;
        try { allowed = matchesOrigin(new URL(details.url), origin); } catch { /* deny */ }
        callback({ cancel: !allowed });
      };
      session.webRequest.onBeforeRequest({ urls: ["*://*/*", "ws://*/*", "wss://*/*", "http://*/*", "https://*/*", `${FABUSHI_MINI_APP_SCHEME}://*/*`] }, requestListener);
      const denyPermission = (_contents: MiniAppWebContents, _permission: string, callback: (allowed: boolean) => void) => callback(false);
      session.setPermissionRequestHandler(denyPermission);
      session.setPermissionCheckHandler(() => false);
      permissionInstalled = true;
      ensureToolBridge(tools);
      const window = new ports.BrowserWindow({
        width: 960, height: 720, minWidth: 480, minHeight: 360, show: false,
        title: "Fabushi Mini App", autoHideMenuBar: true,
        webPreferences: {
          partition, nodeIntegration: false, contextIsolation: true, sandbox: true, webSecurity: true, webviewTag: false,
          ...(tools ? { preload: tools.preloadPath } : {}),
        },
      });
      let reportExportActive = false;
      handleDownload = (event, item, contents) => {
        const owner = ports.currentAccount();
        if (disposed || owner?.accountScope !== launch.accountScope || owner.generation !== launch.accountGeneration
          || reportExportActive || !isAllowedReportExport(launch.appId, item, contents, window.webContents, origin)) {
          event.preventDefault();
          return;
        }
        // Do not set a filesystem path: Electron's normal user-facing save flow owns it.
        reportExportActive = true;
        activeReport = item;
        const release = () => { reportExportActive = false; activeReport = undefined; };
        item.on("updated", () => {
          if (item.getReceivedBytes() > MAX_REPORT_BYTES || item.getTotalBytes() > MAX_REPORT_BYTES) {
            item.cancel();
            release();
          }
        });
        item.on("done", release);
      };
      session.on("will-download", handleDownload);
      const disposeSession = () => {
        cleanupPartialSession();
      };
      const record = { toolScope, tools, origin, window, disposeSession };
      windows.set(key, record);
      window.on("closed", () => { if (windows.get(key) === record) {
        windows.delete(key);
        if (!record.tools?.appId || record.tools.appId === record.toolScope.appId) record.tools?.cancelAll?.();
        disposeSession();
      } });
      window.webContents.setWindowOpenHandler(() => ({ action: "deny" }));
      // Constrain only this Mini App; Host calls, Computer and browser networking stay unchanged.
      await window.webContents.setWebRTCIPHandlingPolicy("disable_non_proxied_udp");
      const preventExternal = (event: { preventDefault(): void }, rawUrl: string) => {
        try { if (!matchesOrigin(new URL(rawUrl), origin)) event.preventDefault(); } catch { event.preventDefault(); }
      };
      window.webContents.on("will-navigate", preventExternal);
      window.webContents.on("will-redirect", preventExternal);
      window.webContents.on("will-attach-webview", (event) => event.preventDefault());
      window.webContents.on("render-process-gone", () => closeKey(key));
      try {
        const beforeLoad = ports.currentAccount();
        if (beforeLoad?.accountScope !== launch.accountScope || beforeLoad.generation !== launch.accountGeneration
          || !windows.has(key)) throw new Error("MINIAPP_STALE_ACCOUNT");
        await window.webContents.loadURL(`${origin}/${encodeURI(launch.entrypoint)}`);
        const stillCurrent = ports.currentAccount();
        if (!windows.has(key) || stillCurrent?.accountScope !== launch.accountScope || stillCurrent.generation !== launch.accountGeneration) {
          closeKey(key);
          throw new Error("MINIAPP_STALE_ACCOUNT");
        }
        window.show();
      } catch (error) {
        closeKey(key);
        throw error;
      }
      return { close: () => closeKey(key) };
      } catch (error) {
        if (windows.has(key)) closeKey(key);
        else cleanupPartialSession();
        throw error;
      } finally {
        opening.delete(key);
      }
    },
    closeAccountScope(accountScope, nextGeneration) {
      if (!Number.isSafeInteger(nextGeneration) || nextGeneration < 0) throw new TypeError("MINIAPP_INVALID_GENERATION");
      minimumGeneration.set(accountScope, Math.max(minimumGeneration.get(accountScope) ?? 0, nextGeneration));
      for (const [windowKey, record] of windows) if (record.toolScope.accountScope === accountScope) closeKey(windowKey);
    },
    closeAll() { for (const key of windows.keys()) closeKey(key); },
  };
}
