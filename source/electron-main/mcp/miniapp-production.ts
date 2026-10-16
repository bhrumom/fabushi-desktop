import { createHash, randomUUID } from "node:crypto";
import type { ProductionServiceContext } from "../main-production-services.js";
import { registerMiniAppManagement, type NativeMiniAppEntry } from "./miniapp-management.js";
import { createMiniAppWindowFactory, type MiniAppWindowFactoryPorts, type VerifiedMiniAppResource } from "./miniapp-window.js";

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

/** MainData only: no launch resources, credentials or paths cross desktop IPC. */
export function registerProductionMiniApps(context: ProductionServiceContext, ipc: Parameters<typeof registerMiniAppManagement>[0]["ipc"]): { dispose(): void } {
  const account = context.requireAccount();
  let generation = 1;
  let current: { accountScope: string; generation: number } | null = null;
  let disposed = false;
  const handles = new Map<string, { close(): void }>();
  const windows = context.native.session == null ? null : createMiniAppWindowFactory({
    BrowserWindow: context.native.BrowserWindow as unknown as MiniAppWindowFactoryPorts["BrowserWindow"],
    session: context.native.session,
    currentAccount: () => current,
  });
  const closeAll = () => { windows?.closeAll(); handles.clear(); };
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
