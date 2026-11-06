import { createHash, randomUUID } from "node:crypto";
import { dirname, join } from "node:path";
import type { ProductionServiceContext } from "../main-production-services.js";
import { registerMiniAppManagement, type NativeMiniAppEntry } from "./miniapp-management.js";
import { createInstalledNativeRuntime, parseNativeRuntime, type NativeRuntimeHandle } from "./miniapp-native-runtime.js";
import { createMiniAppWindowFactory, type MiniAppIpc, type MiniAppLaunchContext, type MiniAppToolBridgePorts, type MiniAppWindowFactoryPorts, type VerifiedMiniAppResource } from "./miniapp-window.js";
import { createMiniAppToolRouter, MINIAPP_TOOL_CONTRACT, type MiniAppToolBackend } from "./miniapp-tools.js";

type Method = "listApprovedMiniApps" | "listInstalledMiniApps" | "installApprovedMiniApp" | "getInstalledMiniAppResources" | "uninstallMiniApp" | "issueMiniAppLaunchSession" | "miniAppPostToConversation";
function record(value: unknown): Record<string, unknown> {
  if (value == null || typeof value !== "object" || Array.isArray(value)) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
  return value as Record<string, unknown>;
}
function entries(value: unknown): NativeMiniAppEntry[] {
  const items = record(value).items;
  if (!Array.isArray(items) || items.length > 128) throw new Error("MINIAPP_DESCRIPTOR_LIMIT");
  return items.map((item) => record(item) as unknown as NativeMiniAppEntry);
}

/** MainData only: no launch resources, credentials or paths cross desktop IPC. */
export function registerProductionMiniApps(context: ProductionServiceContext, ipc: Parameters<typeof registerMiniAppManagement>[0]["ipc"]): { dispose(): void } {
  const account = context.requireAccount();
  let generation = 1;
  let current: { accountScope: string; generation: number } | null = null;
  let disposed = false;
  const handles = new Map<string, { close(): void }>();
  const runtimes = new Map<string, NativeRuntimeHandle>();
  const windows = context.native.session == null ? null : createMiniAppWindowFactory({
    BrowserWindow: context.native.BrowserWindow as unknown as MiniAppWindowFactoryPorts["BrowserWindow"],
    session: context.native.session,
    currentAccount: () => current,
  });
  const closeAll = () => { for (const runtime of runtimes.values()) runtime.cancelAll(); runtimes.clear(); windows?.closeAll(); handles.clear(); };
  const unsubscribe = account.subscribe(() => { generation += 1; current = null; closeAll(); });
  const scope = async (): Promise<string | null> => {
    const ticket = generation;
    const status = await account.getStatus();
    if (disposed || generation !== ticket) throw new Error("MINIAPP_SCOPE_CHANGED");
    const authId = status.kind === "logged-in" && typeof status.authId === "string" ? status.authId : null;
    const next = authId ? createHash("sha256").update(authId).digest("hex") : null;
    if (current && next !== current.accountScope) { generation += 1; closeAll(); }
    current = next ? { accountScope: next, generation } : null;
    return next;
  };
  const call = async (method: Method, args: Record<string, unknown> = {}): Promise<unknown> => {
    const leg = context.coordinatorLegs.legs[method];
    if (typeof leg !== "function") throw new Error("MINIAPP_UNSUPPORTED_HOST");
    return await leg(args);
  };

  const launchWithContext = async (appId: string, accountScope: string, stillCurrent: () => Promise<boolean>, conversationContext?: { conversationId: string; startParameter?: string }) => {
    if (!windows || context.native.app.commandLine.hasSwitch?.("no-sandbox") === true) throw new Error("MINIAPP_UNSUPPORTED_HOST");
    const ticket = generation;
    const loaded = record(await call("getInstalledMiniAppResources", { appId, operationId: randomUUID() }));
    if (!await stillCurrent() || ticket !== generation || loaded.sameScopeHash !== accountScope) throw new Error("MINIAPP_SCOPE_CHANGED");
    const descriptor = record(loaded.descriptor);
    if (descriptor.appId !== appId || typeof descriptor.version !== "string" || typeof descriptor.digest !== "string" || typeof descriptor.entrypoint !== "string") throw new Error("MINIAPP_DESCRIPTOR_INVALID");
    if (!Array.isArray(loaded.resources) || loaded.resources.length > 10_000) throw new Error("MINIAPP_DESCRIPTOR_LIMIT");
    const resources = new Map<string, VerifiedMiniAppResource>();
    let total = 0;
    for (const raw of loaded.resources) {
      const item = record(raw);
      if (typeof item.path !== "string" || typeof item.mime !== "string" || typeof item.bytesBase64 !== "string" || item.bytesBase64.length > 350 * 1024 * 1024 || resources.has(item.path)) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
      const bytes = Buffer.from(item.bytesBase64, "base64");
      if (bytes.toString("base64") !== item.bytesBase64) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
      total += bytes.length;
      if (total > 250 * 1024 * 1024) throw new Error("MINIAPP_DESCRIPTOR_LIMIT");
      resources.set(item.path, { bytes, mime: item.mime });
    }
    if (!await stillCurrent() || ticket !== generation) throw new Error("MINIAPP_SCOPE_CHANGED");

    let launchContext: MiniAppLaunchContext | undefined;
    if (conversationContext) {
      const issued = record(await call("issueMiniAppLaunchSession", {
        appId,
        conversationId: conversationContext.conversationId,
        startParameter: conversationContext.startParameter,
        version: descriptor.version,
        packageDigest: descriptor.digest,
      }));
      const launchSessionId = typeof issued.launchSessionId === "string" ? issued.launchSessionId : "";
      if (!launchSessionId) throw new Error("MINIAPP_OPERATION_FAILED");
      launchContext = {
        launchSessionId,
        conversationId: conversationContext.conversationId,
        startParameter: conversationContext.startParameter,
      };
    }

    const launch = {
      appId, accountScope, accountGeneration: ticket, version: descriptor.version,
      packageDigest: descriptor.digest, entrypoint: descriptor.entrypoint, resources,
      ...(launchContext ? { launchContext } : {}),
    };
    let tools: MiniAppToolBridgePorts | undefined;
    let runtime: NativeRuntimeHandle | undefined;
    const primary = (context.resources as unknown as { preloadPath?: string } | undefined)?.preloadPath;
    if (descriptor.nativeRuntime != null) {
      const definition = parseNativeRuntime(descriptor.nativeRuntime, resources);
      const app = context.native.app as unknown as { getPath(name: string): string };
      const dialog = context.native.dialog as unknown as {
        showMessageBox(options: { type: string; buttons: string[]; defaultId: number; cancelId: number; message: string; detail: string }): Promise<{ response: number }>;
        showOpenDialog(options: { properties: string[] }): Promise<{ canceled: boolean; filePaths: string[] }>;
      } | undefined;
      if (!dialog?.showMessageBox || !dialog.showOpenDialog || !primary || typeof app.getPath !== "function") throw new Error("MINIAPP_UNSUPPORTED_HOST");
      const currentScope = () => !disposed && generation === ticket && current?.accountScope === accountScope;
      runtime = await createInstalledNativeRuntime({
        carrierPath: join(dirname(primary), "..", "electron-main", "miniapp-native-carrier.cjs"),
        codeRoot: join(app.getPath("userData"), "native-miniapp-runtimes"), resources, definition,
        scope: { appId, version: descriptor.version, packageDigest: descriptor.digest, accountScope, accountGeneration: ticket },
        current: currentScope,
        confirm: async detail => {
          if (!currentScope()) return false;
          const answer = await dialog.showMessageBox({ type: "question", buttons: ["取消", "允许"], defaultId: 0, cancelId: 0, message: "小程序请求本机执行权限", detail });
          return currentScope() && answer.response === 1;
        },
        pick: async mode => {
          if (!currentScope()) return null;
          const result = await dialog.showOpenDialog({ properties: [mode === "directory" ? "openDirectory" : "openFile"] });
          return currentScope() && !result.canceled ? result.filePaths.slice(0, 1).map(path => ({ path, name: path, size: 0 })) : null;
        },
        selfUpdate: async action => {
          if (!currentScope()) throw new Error("MINIAPP_SCOPE_CHANGED");
          const approved = entries(await call("listApprovedMiniApps")).find(item => item.appId === appId && item.supported);
          if (!currentScope() || !approved) throw new Error("MINIAPP_POLICY_DENIED");
          const available = approved.packageDigest !== descriptor.digest;
          if (action === "stage" && available) {
            await call("installApprovedMiniApp", { appId, operationId: randomUUID() });
            if (!currentScope()) throw new Error("MINIAPP_SCOPE_CHANGED");
            const timer = setTimeout(() => { handles.get(appId)?.close(); handles.delete(appId); runtimes.get(appId)?.cancelAll(); runtimes.delete(appId); }, 250);
            timer.unref();
          }
          return { installedVersion: descriptor.version, availableVersion: approved.version, availableDigest: approved.packageDigest, updateAvailable: available, staged: action === "stage" && available, requiresReopen: action === "stage" && available };
        },
      });
      if (!await stillCurrent() || ticket !== generation) { runtime.cancelAll(); throw new Error("MINIAPP_SCOPE_CHANGED"); }
      runtimes.get(appId)?.cancelAll(); runtimes.set(appId, runtime);
      tools = { appId, router: runtime.router, ipc: ipc as MiniAppIpc, preloadPath: join(dirname(primary), "preload-miniapp-tools.cjs"), cancelAll: () => runtime?.cancelAll() };
      // Native resources never become WebView-addressable resources.
      for (const name of [...resources.keys()]) if (name.startsWith("native/") || name === "native-runtime.json") resources.delete(name);
    } else if (primary) {
      // Standard web Mini App: construct Host conversation tool bridge
      const conversationBackend: MiniAppToolBackend = {
        capabilities: async () => ({ contract: MINIAPP_TOOL_CONTRACT, platform: "desktop", capabilities: ["offline", "conversation"], targetPools: [] }),
        readState: async () => ({}),
        pick: async () => null,
        release: async () => {},
        send: async () => { throw new Error("UNSUPPORTED"); },
        status: async () => ({}),
        cancel: async () => {},
        scan: async () => ({}),
        hotspot: async () => ({}),
        update: async () => ({}),
        conversation: async (action, args) => {
          let request: Record<string, unknown>;
          if (action === "sendMessage") {
            request = { type: "sendMessage", text: args.text };
          } else if (action === "sendData") {
            request = { type: "sendData", data: args.data };
          } else if (action === "openInvoice") {
            request = { type: "openInvoice", invoiceId: args.invoiceId };
          } else if (action === "startServiceCall") {
            request = { type: "startServiceCall", callId: args.callId, mode: args.mode };
          } else if (action === "submitServiceCallInput") {
            request = { type: "submitServiceCallInput", callId: args.callId, input: args.input };
          } else if (action === "endServiceCall") {
            request = { type: "endServiceCall", callId: args.callId };
          } else {
            throw new Error(`Unsupported conversation action: ${action}`);
          }
          return await call("miniAppPostToConversation", {
            launchSessionId: args.launchSessionId,
            requestId: args.requestId,
            request,
          });
        },
      };
      const conversationRouter = createMiniAppToolRouter({
        backend: conversationBackend,
        allowedAppId: appId,
        approve: () => true,
      });
      tools = {
        appId,
        router: conversationRouter,
        ipc: ipc as MiniAppIpc,
        preloadPath: join(dirname(primary), "preload-miniapp-tools.cjs"),
        cancelAll: () => {},
      };
    }
    try {
      const handle = await windows.open(launch, tools);
      if (!await stillCurrent() || ticket !== generation) { handle.close(); throw new Error("MINIAPP_SCOPE_CHANGED"); }
      handles.set(appId, handle);
    } catch (error) { runtime?.cancelAll(); runtimes.delete(appId); throw error; }
  };

  const management = registerMiniAppManagement({
    ipc, trustedSender: context.getTrustedContents, accountScope: scope,
    subscribeAccount: (listener) => account.subscribe(listener),
    host: {
      list: async () => {
        const installed = entries(await call("listInstalledMiniApps"));
        let approved: NativeMiniAppEntry[];
        try { approved = entries(await call("listApprovedMiniApps")); }
        catch { return installed; } // Host still revalidates installed code on every launch.
        const merged = new Map(approved.map((item) => [item.appId, item]));
        // The catalog cannot replace installed/quarantined Host truth.
        for (const item of installed) {
          const latest = merged.get(item.appId);
          merged.set(item.appId, { ...item, ...(latest?.supported && latest.packageDigest !== item.packageDigest
            ? { availableUpdate: { version: latest.version, packageDigest: latest.packageDigest } } : {}) });
        }
        return [...merged.values()];
      },
      installApproved: async (appId, operationId) => { handles.get(appId)?.close(); handles.delete(appId); await call("installApprovedMiniApp", { appId, operationId }); },
      removeCode: async (appId, operationId) => { runtimes.get(appId)?.cancelAll(); runtimes.delete(appId); await call("uninstallMiniApp", { appId, operationId }); },
    },
    windows: {
      closeAll,
      closeApp: (appId) => { runtimes.get(appId)?.cancelAll(); runtimes.delete(appId); handles.get(appId)?.close(); handles.delete(appId); },
      launch: async (appId, accountScope, stillCurrent) => {
        await launchWithContext(appId, accountScope, stillCurrent);
      },
      launchInConversation: async (appId, accountScope, conversationId, startParameter, stillCurrent) => {
        await launchWithContext(appId, accountScope, stillCurrent, { conversationId, startParameter });
      },
    },
  });
  return { dispose() { if (disposed) return; disposed = true; generation += 1; current = null; unsubscribe(); management.dispose(); closeAll(); } };
}
