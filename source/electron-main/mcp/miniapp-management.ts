import { randomUUID } from "node:crypto";

export interface NativeMiniAppEntry {
  appId: string; displayName: string; publisherId: string; version: string;
  packageDigest: string; sizeBytes: number; offlineCore: boolean;
  installed: boolean; supported: boolean;
  availableUpdate?: { version: string; packageDigest: string };
}
export interface MiniAppManagementPorts {
  ipc: { handle(name: string, handler: (event: unknown, appId?: unknown) => Promise<unknown>): void };
  trustedSender(): unknown;
  accountScope(): Promise<string | null>;
  subscribeAccount(listener: () => void): () => void;
  host: {
    list(): Promise<readonly NativeMiniAppEntry[]>;
    installApproved(appId: string, operationId: string): Promise<void>;
    removeCode(appId: string, operationId: string): Promise<void>;
  };
  windows: {
    launch(appId: string, accountScope: string, stillCurrent: () => Promise<boolean>): Promise<void>;
    closeApp(appId: string): void;
    closeAll(): void;
  };
}
const ID = /^[a-z0-9][a-z0-9._-]{0,127}$/;
const SAFE_ERRORS = new Set(["MINIAPP_ID_INVALID", "MINIAPP_DESCRIPTOR_INVALID",
  "MINIAPP_SENDER_DENIED", "MINIAPP_CLOSED", "MINIAPP_ACCOUNT_REQUIRED",
  "MINIAPP_SCOPE_CHANGED", "MINIAPP_DESCRIPTOR_LIMIT", "MINIAPP_BUSY",
  "MINIAPP_OPERATION_FAILED", "MINIAPP_NOT_INSTALLED", "MINIAPP_POLICY_DENIED",
  "MINIAPP_UNSUPPORTED_HOST", "MINIAPP_VERIFICATION_FAILED"]);
function identity(value: unknown): string {
  if (typeof value !== "string" || !ID.test(value)) throw new Error("MINIAPP_ID_INVALID");
  return value;
}
function publicEntry(entry: NativeMiniAppEntry): NativeMiniAppEntry {
  if (!/^[0-9a-f]{64}$/.test(entry.packageDigest)
    || !Number.isSafeInteger(entry.sizeBytes) || entry.sizeBytes <= 0 || entry.sizeBytes > 100 * 1024 * 1024
    || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(entry.version)
    || typeof entry.displayName !== "string" || entry.displayName.length > 160
    || [entry.offlineCore, entry.installed, entry.supported].some((field) => typeof field !== "boolean")) {
    throw new Error("MINIAPP_DESCRIPTOR_INVALID");
  }
  const update = entry.availableUpdate;
  if (update != null && (typeof update !== "object" || Array.isArray(update)
    || Object.keys(update).sort().join() !== "packageDigest,version"
    || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(update.version)
    || !/^[0-9a-f]{64}$/.test(update.packageDigest)
    || update.packageDigest === entry.packageDigest || !entry.installed)) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
  return { appId: identity(entry.appId), publisherId: identity(entry.publisherId),
    displayName: entry.displayName, version: entry.version, packageDigest: entry.packageDigest,
    sizeBytes: entry.sizeBytes, offlineCore: entry.offlineCore,
    installed: entry.installed, supported: entry.supported,
    ...(update ? { availableUpdate: { version: update.version, packageDigest: update.packageDigest } } : {}) };
}
export function registerMiniAppManagement(ports: MiniAppManagementPorts): { dispose(): void } {
  let disposed = false, generation = 1;
  const pending = new Set<string>();
  const unsubscribe = ports.subscribeAccount(() => {
    generation += 1;
    ports.windows.closeAll();
  });
  const begin = async (event: unknown) => {
    const sender = typeof event === "object" && event != null ? Reflect.get(event, "sender") : undefined;
    if (!sender || sender !== ports.trustedSender()) throw new Error("MINIAPP_SENDER_DENIED");
    if (disposed) throw new Error("MINIAPP_CLOSED");
    const ticket = generation;
    const account = await ports.accountScope();
    if (!account) throw new Error("MINIAPP_ACCOUNT_REQUIRED");
    const stillCurrent = async () => {
      const current = await ports.accountScope();
      return !disposed && generation === ticket && current === account;
    };
    const fence = async () => { if (!await stillCurrent()) throw new Error("MINIAPP_SCOPE_CHANGED"); };
    await fence();
    return { account, fence, stillCurrent };
  };
  const handle = (name: string, action: (event: unknown, appId?: unknown) => Promise<unknown>) => {
    ports.ipc.handle(name, async (event, appId) => {
      try { return await action(event, appId); }
      catch (error) {
        const message = error instanceof Error ? error.message : "";
        throw new Error(SAFE_ERRORS.has(message) ? message : "MINIAPP_OPERATION_FAILED");
      }
    });
  };
  handle("sand:miniapp-list", async (event) => {
    const { fence } = await begin(event);
    const entries = await ports.host.list();
    await fence();
    if (!Array.isArray(entries) || entries.length > 128) throw new Error("MINIAPP_DESCRIPTOR_LIMIT");
    return { entries: entries.map(publicEntry) };
  });
  const mutation = async (event: unknown, raw: unknown, action: "install" | "launch" | "remove") => {
    const { account, fence, stillCurrent } = await begin(event);
    const appId = identity(raw);
    if (pending.has(appId)) throw new Error("MINIAPP_BUSY");
    pending.add(appId);
    try {
      await fence();
      if (action === "install") await ports.host.installApproved(appId, randomUUID());
      else if (action === "launch") await ports.windows.launch(appId, account, stillCurrent);
      else { ports.windows.closeApp(appId); await ports.host.removeCode(appId, randomUUID()); }
      await fence();
    } finally { pending.delete(appId); }
  };
  handle("sand:miniapp-install", (event, appId) => mutation(event, appId, "install"));
  handle("sand:miniapp-launch", (event, appId) => mutation(event, appId, "launch"));
  handle("sand:miniapp-remove-code", (event, appId) => mutation(event, appId, "remove"));
  return { dispose() {
    if (disposed) return;
    disposed = true; generation += 1; unsubscribe(); ports.windows.closeAll();
  } };
}
