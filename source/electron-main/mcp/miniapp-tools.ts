/**
 * Declarative Mini App tool surface for the desktop Host.
 *
 * The renderer never receives paths, credentials or a shell. It asks for a
 * declared tool with validated arguments; this router decides whether the call
 * is allowed and hands it to the native owner.
 */
export const MINIAPP_TOOL_CONTRACT = "fabushi.miniapp.tools.v1";

export const MINIAPP_CAPABILITY = Object.freeze({
  OFFLINE: "offline",
  SELECTION: "selection",
  TRANSFER: "transfer",
  SCAN: "scan",
  STATE: "state",
  CONFIG: "config",
  PLATFORM: "platform",
  HOTSPOT: "hotspot",
  UPDATE: "update",
  DIAGNOSTICS: "diagnostics",
  COMPAT: "compat",
} as const);

export type MiniAppCapability = typeof MINIAPP_CAPABILITY[keyof typeof MINIAPP_CAPABILITY];

export interface MiniAppToolScope {
  readonly appId: string;
  readonly version: string;
  readonly packageDigest: string;
  readonly accountScope: string;
  readonly accountGeneration: number;
}

export interface MiniAppToolError { readonly code: string; readonly message: string }
export type MiniAppToolResult =
  | { readonly ok: true; readonly data?: unknown }
  | { readonly ok: false; readonly error: MiniAppToolError };

export interface MiniAppSelectionEntry { readonly name: string; readonly size: number }
export interface MiniAppTargetPool { readonly id: string; readonly label: string }
export interface MiniAppReceiveConfig { readonly targetPoolId: string; readonly once?: boolean }

/** Implemented by the native owner; never reachable from the renderer directly. */
export interface MiniAppToolBackend {
  capabilities(): Promise<{ readonly capabilities: MiniAppCapability[]; readonly platform: string; readonly targetPools: MiniAppTargetPool[] }>;
  readState(): Promise<unknown>;
  pick(mode: "files" | "directory", limits: { readonly maxFiles: number; readonly maxBytes: number }): Promise<{ readonly selectionId: string; readonly entries: MiniAppSelectionEntry[] } | null>;
  release(selectionId: string): Promise<void>;
  send(selectionId: string, targetPoolId: string): Promise<{ readonly operationId: string }>;
  receive?(config: MiniAppReceiveConfig): Promise<{ readonly operationId: string }>;
  status(operationId: string): Promise<unknown>;
  cancel(operationId: string): Promise<void>;
  scan(mode: "metadata" | "read", rootId: string): Promise<unknown>;
  state?(action: "inspect" | "import", args: Record<string, unknown>): Promise<unknown>;
  config?(action: "defaults" | "check"): Promise<unknown>;
  platform?(action: "inspect" | "mounts" | "time" | "watch"): Promise<unknown>;
  hotspot(action: "plan" | "apply" | "status" | "stop"): Promise<unknown>;
  update(action: "check" | "stage"): Promise<unknown>;
  diagnostics?(action: "selfTest" | "doctor" | "status"): Promise<unknown>;
  compat?(action: "packetInspect" | "countryLookup" | "linkExercise" | "linkRecover", args: Record<string, unknown>): Promise<unknown>;
}

const ID = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/;
const MAX_FILES_LIMIT = 128;
const MAX_BYTES_LIMIT = 1024 ** 3;

function fail(code: string, message: string): MiniAppToolResult {
  return { ok: false, error: { code, message } };
}

function requiredId(args: Record<string, unknown>, key: string): string {
  const value = args[key];
  if (typeof value !== "string" || !ID.test(value)) throw new TypeError(`INVALID_${key.toUpperCase()}`);
  return value;
}

function optionalLimits(args: Record<string, unknown>): { maxFiles: number; maxBytes: number } {
  const maxFiles = args.maxFiles === undefined ? MAX_FILES_LIMIT : Number(args.maxFiles);
  const maxBytes = args.maxBytes === undefined ? MAX_BYTES_LIMIT : Number(args.maxBytes);
  if (!Number.isSafeInteger(maxFiles) || maxFiles < 1 || maxFiles > MAX_FILES_LIMIT) throw new TypeError("INVALID_MAX_FILES");
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 1 || maxBytes > MAX_BYTES_LIMIT) throw new TypeError("INVALID_MAX_BYTES");
  return { maxFiles, maxBytes };
}

function optionalBoolean(args: Record<string, unknown>, key: string): boolean | undefined {
  if (args[key] === undefined) return undefined;
  if (typeof args[key] !== "boolean") throw new TypeError(`INVALID_${key.toUpperCase()}`);
  return args[key] as boolean;
}

function asRecord(args: unknown): Record<string, unknown> {
  if (args === undefined || args === null) return {};
  if (typeof args !== "object" || Array.isArray(args)) throw new TypeError("INVALID_ARGUMENTS");
  return args as Record<string, unknown>;
}

type Handler = (args: Record<string, unknown>) => Promise<MiniAppToolResult>;

function handlersFor(backend: MiniAppToolBackend): Map<string, { capability: MiniAppCapability; mutating: boolean; run: Handler }> {
  const entries: Array<[string, MiniAppCapability, boolean, Handler]> = [
    ["offline.capabilities", MINIAPP_CAPABILITY.OFFLINE, false, async () => {
      const announced = await backend.capabilities();
      return { ok: true, data: { contract: MINIAPP_TOOL_CONTRACT, platform: announced.platform, capabilities: announced.capabilities, targetPools: announced.targetPools } };
    }],
    ["offline.state.read", MINIAPP_CAPABILITY.OFFLINE, false, async () => ({ ok: true, data: await backend.readState() })],
    ["selection.pick", MINIAPP_CAPABILITY.SELECTION, false, async (args) => {
      const mode = args.mode === "directory" ? "directory" : "files";
      const picked = await backend.pick(mode, optionalLimits(args));
      return picked ? { ok: true, data: picked } : fail("SELECTION_CANCELLED", "用户取消了选择");
    }],
    ["selection.release", MINIAPP_CAPABILITY.SELECTION, false, async (args) => {
      await backend.release(requiredId(args, "selectionId"));
      return { ok: true };
    }],
    ["transfer.send", MINIAPP_CAPABILITY.TRANSFER, true, async (args) => {
      const sent = await backend.send(requiredId(args, "selectionId"), requiredId(args, "targetPoolId"));
      return { ok: true, data: sent };
    }],
    ["transfer.receive", MINIAPP_CAPABILITY.TRANSFER, true, async (args) => {
      if (!backend.receive) return fail("CAPABILITY_UNAVAILABLE", "宿主未提供接收能力");
      const targetPoolId = requiredId(args, "targetPoolId");
      const received = await backend.receive({ targetPoolId, once: optionalBoolean(args, "once") });
      return { ok: true, data: received };
    }],
    ["transfer.status", MINIAPP_CAPABILITY.TRANSFER, false, async (args) => ({ ok: true, data: await backend.status(requiredId(args, "operationId")) })],
    ["transfer.cancel", MINIAPP_CAPABILITY.TRANSFER, true, async (args) => {
      await backend.cancel(requiredId(args, "operationId"));
      return { ok: true };
    }],
    ["scan.metadata", MINIAPP_CAPABILITY.SCAN, false, async (args) => ({ ok: true, data: await backend.scan("metadata", requiredId(args, "rootId")) })],
    ["scan.read", MINIAPP_CAPABILITY.SCAN, false, async (args) => ({ ok: true, data: await backend.scan("read", requiredId(args, "rootId")) })],
    ["state.inspect", MINIAPP_CAPABILITY.STATE, false, async (args) => backend.state ? ({ ok: true, data: await backend.state("inspect", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供状态能力")],
    ["state.import", MINIAPP_CAPABILITY.STATE, true, async (args) => backend.state ? ({ ok: true, data: await backend.state("import", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供状态能力")],
    ["config.defaults", MINIAPP_CAPABILITY.CONFIG, false, async () => backend.config ? ({ ok: true, data: await backend.config("defaults") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供配置能力")],
    ["config.check", MINIAPP_CAPABILITY.CONFIG, false, async () => backend.config ? ({ ok: true, data: await backend.config("check") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供配置能力")],
    ["platform.inspect", MINIAPP_CAPABILITY.PLATFORM, false, async () => backend.platform ? ({ ok: true, data: await backend.platform("inspect") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供平台能力")],
    ["platform.mounts", MINIAPP_CAPABILITY.PLATFORM, false, async () => backend.platform ? ({ ok: true, data: await backend.platform("mounts") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供平台能力")],
    ["platform.time", MINIAPP_CAPABILITY.PLATFORM, false, async () => backend.platform ? ({ ok: true, data: await backend.platform("time") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供平台能力")],
    ["platform.watch", MINIAPP_CAPABILITY.PLATFORM, false, async () => backend.platform ? ({ ok: true, data: await backend.platform("watch") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供平台能力")],
    ["hotspot.plan", MINIAPP_CAPABILITY.HOTSPOT, false, async () => ({ ok: true, data: await backend.hotspot("plan") })],
    ["hotspot.apply", MINIAPP_CAPABILITY.HOTSPOT, true, async () => ({ ok: true, data: await backend.hotspot("apply") })],
    ["hotspot.status", MINIAPP_CAPABILITY.HOTSPOT, false, async () => ({ ok: true, data: await backend.hotspot("status") })],
    ["hotspot.stop", MINIAPP_CAPABILITY.HOTSPOT, true, async () => ({ ok: true, data: await backend.hotspot("stop") })],
    ["update.check", MINIAPP_CAPABILITY.UPDATE, false, async () => ({ ok: true, data: await backend.update("check") })],
    ["update.stage", MINIAPP_CAPABILITY.UPDATE, true, async () => ({ ok: true, data: await backend.update("stage") })],
    ["diagnostics.selfTest", MINIAPP_CAPABILITY.DIAGNOSTICS, false, async () => backend.diagnostics ? ({ ok: true, data: await backend.diagnostics("selfTest") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供诊断能力")],
    ["diagnostics.doctor", MINIAPP_CAPABILITY.DIAGNOSTICS, false, async () => backend.diagnostics ? ({ ok: true, data: await backend.diagnostics("doctor") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供诊断能力")],
    ["diagnostics.status", MINIAPP_CAPABILITY.DIAGNOSTICS, false, async () => backend.diagnostics ? ({ ok: true, data: await backend.diagnostics("status") }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供诊断能力")],
    ["compat.packetInspect", MINIAPP_CAPABILITY.COMPAT, false, async (args) => backend.compat ? ({ ok: true, data: await backend.compat("packetInspect", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供兼容能力")],
    ["compat.countryLookup", MINIAPP_CAPABILITY.COMPAT, false, async (args) => backend.compat ? ({ ok: true, data: await backend.compat("countryLookup", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供兼容能力")],
    ["compat.linkExercise", MINIAPP_CAPABILITY.COMPAT, true, async (args) => backend.compat ? ({ ok: true, data: await backend.compat("linkExercise", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供兼容能力")],
    ["compat.linkRecover", MINIAPP_CAPABILITY.COMPAT, true, async (args) => backend.compat ? ({ ok: true, data: await backend.compat("linkRecover", args) }) : fail("CAPABILITY_UNAVAILABLE", "宿主未提供兼容能力")],
  ];
  return new Map(entries.map(([name, capability, mutating, run]) => [name, { capability, mutating, run }]));
}

export interface MiniAppToolRouterOptions {
  readonly backend: MiniAppToolBackend;
  /** Called before a mutating tool runs; return false to reject. */
  readonly approve?: (tool: string, args: Record<string, unknown>) => Promise<boolean> | boolean;
  readonly now?: () => number;
}

export interface MiniAppToolRouter {
  invoke(scope: MiniAppToolScope, tool: unknown, rawArgs: unknown): Promise<MiniAppToolResult>;
}

const WINDOW_MS = 10_000;
const WINDOW_LIMIT = 40;

export function createMiniAppToolRouter(options: MiniAppToolRouterOptions): MiniAppToolRouter {
  const handlers = handlersFor(options.backend);
  const now = options.now ?? Date.now;
  const windows = new Map<string, { start: number; count: number }>();

  const throttle = (key: string): boolean => {
    const current = now();
    const record = windows.get(key);
    if (!record || current - record.start >= WINDOW_MS) { windows.set(key, { start: current, count: 1 }); return true; }
    record.count += 1;
    return record.count <= WINDOW_LIMIT;
  };

  return {
    async invoke(scope, tool, rawArgs) {
      if (typeof tool !== "string" || !handlers.has(tool)) {
        return fail("UNKNOWN_TOOL", "该工具未在契约中声明");
      }
      const handler = handlers.get(tool);
      if (!handler) return fail("UNKNOWN_TOOL", "该工具未在契约中声明");
      if (!throttle(scope.accountScope)) return fail("RATE_LIMITED", "调用过于频繁，请稍后重试");
      // Discovery is a policy boundary, not a UI hint. Re-check the native
      // announcement on every invocation so a renderer cannot call a hidden
      // capability by guessing its method name.
      let announced: Awaited<ReturnType<MiniAppToolBackend["capabilities"]>>;
      try { announced = await options.backend.capabilities(); }
      catch { return fail("CAPABILITY_UNAVAILABLE", "宿主能力探测失败"); }
      if (!announced.capabilities.includes(handler.capability)) {
        return fail("CAPABILITY_UNAVAILABLE", "该能力未获宿主授权");
      }
      let args: Record<string, unknown>;
      try { args = asRecord(rawArgs); }
      catch { return fail("INVALID_ARGUMENTS", "参数必须是对象"); }
      if (handler.mutating) {
        // Fail closed: a mutating tool without an approval owner is not runnable.
        if (!options.approve) return fail("NOT_APPROVED", "宿主未提供审批通道");
        let approved = false;
        try { approved = await options.approve(tool, args); } catch { approved = false; }
        if (!approved) return fail("NOT_APPROVED", "用户未批准该操作");
      }
      try {
        const result = await handler.run(args);
        return result;
      } catch (error) {
        const code = error instanceof TypeError ? String(error.message) : "TOOL_FAILED";
        const message = error instanceof Error ? error.message : String(error);
        return fail(code, message.slice(0, 240));
      }
    },
  };
}
