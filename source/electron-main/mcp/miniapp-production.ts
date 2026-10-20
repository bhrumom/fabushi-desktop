import { createHash, randomUUID } from "node:crypto";
import { delimiter } from "node:path";
import type { ProductionServiceContext } from "../main-production-services.js";
import { registerMiniAppManagement, type NativeMiniAppEntry } from "./miniapp-management.js";
import { createDandelionCliBackend, type MiniAppPickPort } from "./miniapp-cli-backend.js";
import { createMiniAppToolRouter } from "./miniapp-tools.js";
import { createMiniAppWindowFactory, type MiniAppIpc, type MiniAppToolBridgePorts, type MiniAppWindowFactoryPorts, type VerifiedMiniAppResource } from "./miniapp-window.js";

type Method = "listApprovedMiniApps" | "listInstalledMiniApps" | "installApprovedMiniApp" | "getInstalledMiniAppResources" | "uninstallMiniApp";
function record(value: unknown): Record<string, unknown> {
  if (value == null || typeof value !== "object" || Array.isArray(value)) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
  return value as Record<string, unknown>;
}
function entries(value: unknown): NativeMiniAppEntry[] {
  const items = record(value).items;
  if (!Array.isArray(items) || items.length > 128) throw new Error("MINIAPP_DESCRIPTOR_LIMIT");
  return items.map((item) => record(item) as unknown as NativeMiniAppEntry);
}

/** Opt-in native tool bridge. Absent configuration keeps the window render-only. */
interface MiniAppDialogPort {
  showOpenDialog(options: { properties: string[] }): Promise<{ canceled: boolean; filePaths: string[] }>;
  showMessageBox?(options: { type?: string; buttons: string[]; defaultId?: number; cancelId?: number; message: string; detail?: string }): Promise<{ response: number }>;
}

function miniAppTooling(context: ProductionServiceContext, ipc: unknown): MiniAppToolBridgePorts | undefined {
  const env = process.env;
  const binaryPath = env.FABUSHI_DANDELION_BIN?.trim() ?? "";
  const configPath = env.FABUSHI_DANDELION_CONFIG?.trim() ?? "";
  const preloadPath = env.FABUSHI_MINIAPP_PRELOAD?.trim() ?? "";
  const roots = (env.FABUSHI_DANDELION_ROOTS ?? "").split(delimiter).map((item) => item.trim()).filter(Boolean);
  const stateDirectory = env.FABUSHI_DANDELION_STATE_DIR?.trim() || undefined;
  const compatDirectory = env.FABUSHI_DANDELION_COMPAT_DIR?.trim() || undefined;
  const hotspotJournalDirectory = env.FABUSHI_DANDELION_HOTSPOT_JOURNAL_DIR?.trim() || undefined;
  const hotspotInterface = env.FABUSHI_DANDELION_HOTSPOT_INTERFACE?.trim() || undefined;
  const hotspotSsid = env.FABUSHI_DANDELION_HOTSPOT_SSID?.trim() || undefined;
  const hotspotBand = env.FABUSHI_DANDELION_HOTSPOT_BAND?.trim() || undefined;
  const hotspotPasswordFile = env.FABUSHI_DANDELION_HOTSPOT_PASSWORD_FILE?.trim() || undefined;
  const updateTrust = env.FABUSHI_DANDELION_UPDATE_TRUST?.trim();
  let receive: { targetPoolId: string; caFile: string; certificateFile: string; privateKeyFile: string; bind?: string; maxSessions?: number; trafficLimitBytes?: number; rateBytesPerSecond?: number; timeoutMs?: number } | undefined;
  if (!binaryPath || !configPath || !preloadPath || roots.length === 0) return undefined;
  let pools: { id: string; label: string }[];
  try {
    const parsed: unknown = JSON.parse(env.FABUSHI_DANDELION_POOLS ?? "[]");
    if (!Array.isArray(parsed)) return undefined;
    pools = parsed.filter((item): item is { id: string; label: string } =>
      typeof item === "object" && item !== null
      && typeof (item as { id?: unknown }).id === "string" && typeof (item as { label?: unknown }).label === "string");
  } catch { return undefined; }
  let parsedUpdateTrust: { manifest?: string; signature?: string; publicKey: string; directory?: string; artifact?: string; allowedHost?: string } | undefined;
  if (updateTrust) {
    try {
      const value: unknown = JSON.parse(updateTrust);
      if (value && typeof value === "object" && !Array.isArray(value)
        && typeof (value as { publicKey?: unknown }).publicKey === "string") {
        parsedUpdateTrust = value as typeof parsedUpdateTrust;
      }
    } catch { return undefined; }
  }
  try {
    const parsed: unknown = JSON.parse(env.FABUSHI_DANDELION_RECEIVE ?? "null");
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      const value = parsed as Record<string, unknown>;
      if (typeof value.targetPoolId === "string" && typeof value.caFile === "string" && typeof value.certificateFile === "string" && typeof value.privateKeyFile === "string") {
        receive = {
          targetPoolId: value.targetPoolId,
          caFile: value.caFile,
          certificateFile: value.certificateFile,
          privateKeyFile: value.privateKeyFile,
          bind: typeof value.bind === "string" ? value.bind : undefined,
          maxSessions: typeof value.maxSessions === "number" ? value.maxSessions : undefined,
          trafficLimitBytes: typeof value.trafficLimitBytes === "number" ? value.trafficLimitBytes : undefined,
          rateBytesPerSecond: typeof value.rateBytesPerSecond === "number" ? value.rateBytesPerSecond : undefined,
          timeoutMs: typeof value.timeoutMs === "number" ? value.timeoutMs : undefined,
        };
      }
    }
  } catch { return undefined; }
  const dialog = (context.native as unknown as { dialog?: MiniAppDialogPort }).dialog;
  if (!dialog) return undefined;
  const pick: MiniAppPickPort = async (mode) => {
    const selected = await dialog.showOpenDialog({ properties: [mode === "directory" ? "openDirectory" : "openFile"] });
    if (selected.canceled || selected.filePaths.length === 0) return null;
    return selected.filePaths.slice(0, 1).map((path) => ({ path, name: path, size: 0 }));
  };
  const backend = createDandelionCliBackend({
    binaryPath, configPath, allowedRoots: roots, targetPools: pools,
    stateDirectory, compatDirectory, receive,
    hotspotJournalDirectory,
    hotspotInterface, hotspotSsid, hotspotBand, hotspotPasswordFile,
    updateTrust: parsedUpdateTrust,
    enableHotspot: env.FABUSHI_DANDELION_HOTSPOT === "1",
    enableUpdate: env.FABUSHI_DANDELION_UPDATE === "1",
  }, { pick });
  const router = createMiniAppToolRouter({
    backend,
    approve: async (tool) => {
      if (!dialog.showMessageBox) return false;
      const answer = await dialog.showMessageBox({
        type: "question", buttons: ["取消", "允许"], defaultId: 0, cancelId: 0,
        message: "小程序请求执行受控操作", detail: tool,
      });
      return answer.response === 1;
    },
  });
  return { router, preloadPath, ipc: ipc as MiniAppIpc, cancelAll: () => backend.cancelAll?.() };
}

/** MainData only: no launch resources, credentials or paths cross desktop IPC. */
export function registerProductionMiniApps(context: ProductionServiceContext, ipc: Parameters<typeof registerMiniAppManagement>[0]["ipc"]): { dispose(): void } {
  const account = context.requireAccount();
  let generation = 1;
  let current: { accountScope: string; generation: number } | null = null;
  let disposed = false;
  const handles = new Map<string, { close(): void }>();
  const tooling = miniAppTooling(context, ipc);
  const windows = context.native.session == null ? null : createMiniAppWindowFactory({
    BrowserWindow: context.native.BrowserWindow as unknown as MiniAppWindowFactoryPorts["BrowserWindow"],
    session: context.native.session,
    currentAccount: () => current,
    ...(tooling ? { tools: tooling } : {}),
  });
  const closeAll = () => { tooling?.cancelAll?.(); windows?.closeAll(); handles.clear(); };
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
        for (const item of installed) merged.set(item.appId, item);
        return [...merged.values()];
      },
      installApproved: async (appId, operationId) => { handles.get(appId)?.close(); handles.delete(appId); await call("installApprovedMiniApp", { appId, operationId }); },
      removeCode: async (appId, operationId) => { await call("uninstallMiniApp", { appId, operationId }); },
    },
    windows: {
      closeAll,
      closeApp: (appId) => { handles.get(appId)?.close(); handles.delete(appId); },
      launch: async (appId, accountScope, stillCurrent) => {
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
        const handle = await windows.open({ appId, accountScope, accountGeneration: ticket, version: descriptor.version, packageDigest: descriptor.digest, entrypoint: descriptor.entrypoint, resources });
        if (!await stillCurrent() || ticket !== generation) { handle.close(); throw new Error("MINIAPP_SCOPE_CHANGED"); }
        handles.set(appId, handle);
      },
    },
  });
  return { dispose() { if (disposed) return; disposed = true; generation += 1; current = null; unsubscribe(); management.dispose(); closeAll(); } };
}
