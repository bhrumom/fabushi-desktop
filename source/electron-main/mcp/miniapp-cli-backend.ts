/**
 * Native owner for the Mini App tool contract.
 *
 * Execution is delegated to the already-installed `dandelion` binary with
 * argv arrays only (no shell). Every path handed to it must sit under a root
 * the Host authorized; the page can never name a path itself.
 *
 * Selections are directories on purpose: the engine's unit of work is a source
 * root, and accepting individual files would silently widen the sent set.
 */
import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { realpath } from "node:fs/promises";
import { basename, isAbsolute, relative, sep } from "node:path";
import type {
  MiniAppCapability,
  MiniAppSelectionEntry,
  MiniAppTargetPool,
  MiniAppToolBackend,
} from "./miniapp-tools.js";

export interface DandelionCliConfig {
  /** Absolute path to the installed `dandelion` binary. */
  readonly binaryPath: string;
  /** Absolute path to the reviewed service configuration. */
  readonly configPath: string;
  /** Roots the Host owner explicitly authorized for scanning and transfer. */
  readonly allowedRoots: readonly string[];
  /** Authorized destination pools; empty disables transfer. */
  readonly targetPools: readonly MiniAppTargetPool[];
  readonly enableHotspot?: boolean;
  readonly enableUpdate?: boolean;
  readonly timeoutMs?: number;
}

export interface MiniAppPickPort {
  (mode: "files" | "directory", limits: { readonly maxFiles: number; readonly maxBytes: number }): Promise<{ readonly path: string; readonly name: string; readonly size: number }[] | null>;
}

export interface MiniAppChildHandle {
  readonly done: Promise<{ readonly code: number; readonly stdout: string; readonly stderr: string }>;
  kill(): void;
}

export interface MiniAppSpawnPort {
  (argv: readonly string[]): MiniAppChildHandle;
}

const SELECTION_TTL_MS = 10 * 60_000;
const MAX_SELECTIONS = 16;
const MAX_OPERATIONS = 8;

function insideRoot(root: string, target: string): boolean {
  const rel = relative(root, target);
  return rel === "" || (!rel.startsWith(`..${sep}`) && rel !== ".." && !isAbsolute(rel));
}

function defaultSpawn(binaryPath: string, configPath: string): MiniAppSpawnPort {
  return (argv) => {
    const child = spawn(binaryPath, ["--config", configPath, ...argv], {
      shell: false,
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"],
      env: { PATH: "/usr/local/bin:/usr/bin:/bin", LC_ALL: "C" },
    });
    let stdout = "", stderr = "";
    child.stdout.on("data", (chunk: Buffer) => { if (stdout.length < 1_048_576) stdout += chunk.toString("utf8"); });
    child.stderr.on("data", (chunk: Buffer) => { if (stderr.length < 65_536) stderr += chunk.toString("utf8"); });
    const done = new Promise<{ code: number; stdout: string; stderr: string }>((settle) => {
      child.on("error", (error) => settle({ code: -1, stdout, stderr: String(error.message) }));
      child.on("close", (code) => settle({ code: code ?? -1, stdout, stderr }));
    });
    return { done, kill: () => { if (!child.killed) child.kill("SIGKILL"); } };
  };
}

interface Operation {
  readonly handle: MiniAppChildHandle;
  readonly timer: ReturnType<typeof setTimeout>;
  state: "running" | "complete" | "failed" | "cancelled";
  result?: unknown;
  error?: string;
}

export function createDandelionCliBackend(
  config: DandelionCliConfig,
  ports: { readonly pick: MiniAppPickPort; readonly spawn?: MiniAppSpawnPort },
): MiniAppToolBackend {
  const timeoutMs = config.timeoutMs ?? 30 * 60_000;
  const spawnCommand = ports.spawn ?? defaultSpawn(config.binaryPath, config.configPath);
  const selections = new Map<string, { root: string; expiresAt: number }>();
  const operations = new Map<string, Operation>();
  const roots = config.allowedRoots.filter((root) => isAbsolute(root));
  // macOS resolves /tmp to /private/tmp; compare real paths on both sides.
  let resolvedRoots: Promise<string[]> | null = null;
  const canonicalRoots = (): Promise<string[]> => {
    resolvedRoots ??= Promise.all(roots.map(async (root) => {
      try { return await realpath(root); } catch { return root; }
    }));
    return resolvedRoots;
  };

  const pruneSelections = (): void => {
    const now = Date.now();
    for (const [id, record] of selections) if (record.expiresAt <= now) selections.delete(id);
    while (selections.size > MAX_SELECTIONS) {
      const oldest = selections.keys().next().value;
      if (oldest === undefined) break;
      selections.delete(oldest);
    }
  };

  const pruneOperations = (): void => {
    while (operations.size > MAX_OPERATIONS) {
      const oldest = [...operations].find(([, operation]) => operation.state !== "running")?.[0];
      if (oldest === undefined) break;
      operations.delete(oldest);
    }
  };

  const requireSelection = (selectionId: string): string => {
    pruneSelections();
    const record = selections.get(selectionId);
    if (!record) throw new Error("UNKNOWN_SELECTION");
    if (record.expiresAt <= Date.now()) { selections.delete(selectionId); throw new Error("SELECTION_EXPIRED"); }
    return record.root;
  };

  const execute = async (argv: readonly string[]): Promise<unknown> => {
    const handle = spawnCommand(argv);
    const timer = setTimeout(() => handle.kill(), timeoutMs);
    const result = await handle.done;
    clearTimeout(timer);
    if (result.code !== 0) {
      const detail = (result.stderr || result.stdout).trim().slice(0, 240);
      throw new Error(`DANDELION_EXIT_${result.code}${detail ? `: ${detail}` : ""}`);
    }
    const text = result.stdout.trim();
    if (!text) return {};
    try { return JSON.parse(text); }
    catch { throw new Error("DANDELION_INVALID_JSON"); }
  };

  return {
    capabilities() {
      const capabilities: MiniAppCapability[] = ["offline", "selection"];
      if (config.targetPools.length > 0 && roots.length > 0) capabilities.push("transfer");
      if (roots.length > 0) capabilities.push("scan");
      if (config.enableHotspot === true) capabilities.push("hotspot");
      if (config.enableUpdate === true) capabilities.push("update");
      return Promise.resolve({ capabilities, platform: "desktop", targetPools: [...config.targetPools] });
    },
    readState() {
      return Promise.resolve({
        selections: selections.size,
        authorizedRoots: roots.length,
        targetPools: config.targetPools.length,
        operations: [...operations].map(([id, operation]) => ({ id, state: operation.state })),
      });
    },
    async pick(mode) {
      // The engine works on a source root; per-file selection would widen it.
      if (mode !== "directory") throw new Error("DIRECTORY_SELECTION_REQUIRED");
      const picked = await ports.pick(mode, { maxFiles: 1, maxBytes: 1024 ** 3 });
      if (!picked || picked.length === 0) return null;
      const root = await realpath(picked[0]!.path);
      if (!(await canonicalRoots()).some((allowed) => insideRoot(allowed, root))) throw new Error("ROOT_NOT_AUTHORIZED");
      pruneSelections();
      const selectionId = randomUUID();
      selections.set(selectionId, { root, expiresAt: Date.now() + SELECTION_TTL_MS });
      const entries: MiniAppSelectionEntry[] = [{ name: basename(root), size: picked[0]!.size }];
      return { selectionId, entries };
    },
    release(selectionId) {
      selections.delete(selectionId);
      return Promise.resolve();
    },
    async send(selectionId, targetPoolId) {
      const root = requireSelection(selectionId);
      if (!config.targetPools.some((pool) => pool.id === targetPoolId)) throw new Error("UNKNOWN_TARGET_POOL");
      pruneOperations();
      const operationId = randomUUID();
      const handle = spawnCommand(["run", "--source", root, "--json"]);
      const timer = setTimeout(() => handle.kill(), timeoutMs);
      const operation: Operation = { handle, timer, state: "running" };
      operations.set(operationId, operation);
      void handle.done.then((result) => {
        clearTimeout(timer);
        if (operation.state === "cancelled") return;
        if (result.code !== 0) {
          operation.state = "failed";
          operation.error = (result.stderr || result.stdout).trim().slice(0, 240) || `DANDELION_EXIT_${result.code}`;
          return;
        }
        operation.state = "complete";
        try { operation.result = result.stdout.trim() ? JSON.parse(result.stdout) : {}; }
        catch { operation.error = "DANDELION_INVALID_JSON"; operation.state = "failed"; }
      });
      return { operationId };
    },
    async status(operationId) {
      const operation = operations.get(operationId);
      if (!operation) throw new Error("UNKNOWN_OPERATION");
      return { state: operation.state, result: operation.result, error: operation.error };
    },
    async cancel(operationId) {
      const operation = operations.get(operationId);
      if (!operation) throw new Error("UNKNOWN_OPERATION");
      operation.state = "cancelled";
      clearTimeout(operation.timer);
      operation.handle.kill();
    },
    async scan(mode, rootId) {
      return execute(["scan", mode, "--source", requireSelection(rootId), "--json"]);
    },
    async hotspot(action) {
      if (config.enableHotspot !== true) throw new Error("HOTSPOT_DISABLED");
      return execute(action === "plan" ? ["hotspot", "plan", "--json"] : ["hotspot", action, "--confirm", "--json"]);
    },
    async update(action) {
      if (config.enableUpdate !== true) throw new Error("UPDATE_DISABLED");
      return execute(["update", action, "--json"]);
    },
  };
}
