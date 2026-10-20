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
import { realpath, stat } from "node:fs/promises";
import { basename, isAbsolute, relative, sep } from "node:path";
import type {
  MiniAppCapability,
  MiniAppSelectionEntry,
  MiniAppTargetPool,
  MiniAppTransferSendConfig,
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
  /** Trusted native state directory; never supplied by the renderer. */
  readonly stateDirectory?: string;
  /** Trusted native compatibility data directory; never supplied by the renderer. */
  readonly compatDirectory?: string;
  /** Optional authenticated receive settings owned by the Host. */
  readonly receive?: { readonly targetPoolId: string; readonly caFile: string; readonly certificateFile: string; readonly privateKeyFile: string; readonly bind?: string; readonly maxSessions?: number; readonly trafficLimitBytes?: number; readonly rateBytesPerSecond?: number; readonly timeoutMs?: number };
  /** Trusted journal directory used by hotspot status/apply/stop. */
  readonly hotspotJournalDirectory?: string;
  readonly hotspotInterface?: string;
  readonly hotspotSsid?: string;
  readonly hotspotBand?: string;
  readonly hotspotPasswordFile?: string;
  readonly updateTrust?: {
    readonly manifest?: string;
    readonly signature?: string;
    readonly publicKey: string;
    readonly directory?: string;
    readonly artifact?: string;
    readonly allowedHost?: string;
  };
  readonly enableHotspot?: boolean;
  readonly enableUpdate?: boolean;
  readonly timeoutMs?: number;
  /** The host platform, injectable for contract tests. The Rust CLI is Linux-only. */
  readonly platform?: NodeJS.Platform;
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

const NATIVE_CLI_PLATFORM: NodeJS.Platform = "linux";

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
  const platform = config.platform ?? process.platform;
  const nativeCliSupported = platform === NATIVE_CLI_PLATFORM;
  const spawnCommand = ports.spawn ?? defaultSpawn(config.binaryPath, config.configPath);
  const selections = new Map<string, { root: string; expiresAt: number }>();
  const operations = new Map<string, Operation>();
  const roots = config.allowedRoots.filter((root) => isAbsolute(root));
  const stateDirectory = config.stateDirectory && isAbsolute(config.stateDirectory) ? config.stateDirectory : null;
  const compatDirectory = config.compatDirectory && isAbsolute(config.compatDirectory) ? config.compatDirectory : null;
  const compatPath = async (value: unknown, fallback: string): Promise<string> => {
    if (!compatDirectory) throw new Error("COMPAT_DISABLED");
    const raw = typeof value === "string" && value.length > 0 ? value : fallback;
    const candidate = isAbsolute(raw) ? raw : `${compatDirectory}/${raw}`;
    const resolved = await realpath(candidate);
    const root = await realpath(compatDirectory);
    if (!insideRoot(root, resolved)) throw new Error("COMPAT_PATH_NOT_AUTHORIZED");
    return resolved;
  };
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
      if (nativeCliSupported) {
        if (config.targetPools.length > 0 && roots.length > 0) capabilities.push("transfer");
        if (roots.length > 0) capabilities.push("scan");
        if (stateDirectory) capabilities.push("state");
        capabilities.push("config", "platform", "diagnostics");
        if (compatDirectory) capabilities.push("compat");
        if (config.enableHotspot === true) capabilities.push("hotspot");
        if (config.enableUpdate === true) capabilities.push("update");
      }
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
    async pick(mode, limits) {
      // The engine works on a source root; per-file selection would widen it.
      if (mode !== "directory") throw new Error("DIRECTORY_SELECTION_REQUIRED");
      const picked = await ports.pick(mode, limits);
      if (!picked || picked.length === 0) return null;
      const root = await realpath(picked[0]!.path);
      const info = await stat(root);
      if (mode === "directory" && !info.isDirectory()) throw new Error("DIRECTORY_SELECTION_REQUIRED");
      if (!(await canonicalRoots()).some((allowed) => insideRoot(allowed, root))) throw new Error("ROOT_NOT_AUTHORIZED");
      // Directory stat size is filesystem metadata, not the recursive payload
      // size. The Dandelion scanner enforces the configured recursive budget.
      if (picked.length > limits.maxFiles || (!info.isDirectory() && info.size > limits.maxBytes)) throw new Error("SELECTION_LIMIT_EXCEEDED");
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
    async send(sendConfig: MiniAppTransferSendConfig) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      const root = requireSelection(sendConfig.selectionId);
      if (!config.targetPools.some((pool) => pool.id === sendConfig.targetPoolId)) throw new Error("UNKNOWN_TARGET_POOL");
      pruneOperations();
      const operationId = randomUUID();
      const argv = ["run", "--source", root];
      if (sendConfig.repeat !== undefined) argv.push("--repeat", String(sendConfig.repeat));
      if (sendConfig.watch === true) argv.push("--watch");
      if (sendConfig.intervalMs !== undefined) argv.push("--interval-ms", String(sendConfig.intervalMs));
      if (sendConfig.resume === true) argv.push("--resume");
      if (sendConfig.rateBytesPerSecond !== undefined) argv.push("--rate-bytes-per-second", String(sendConfig.rateBytesPerSecond));
      if (sendConfig.trafficLimitBytes !== undefined) argv.push("--traffic-limit-bytes", String(sendConfig.trafficLimitBytes));
      if (sendConfig.queueCapacity !== undefined) argv.push("--queue-capacity", String(sendConfig.queueCapacity));
      if (sendConfig.chunkBytes !== undefined) argv.push("--chunk-bytes", String(sendConfig.chunkBytes));
      argv.push("--json");
      const handle = spawnCommand(argv);
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
    async receive(receiveConfig) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      const settings = config.receive;
      if (!settings || settings.targetPoolId !== receiveConfig.targetPoolId) throw new Error("RECEIVE_DISABLED");
      pruneOperations();
      const operationId = randomUUID();
      const argv = ["receive", "--bind", settings.bind ?? "127.0.0.1:20000", "--ca-file", settings.caFile, "--certificate-file", settings.certificateFile, "--private-key-file", settings.privateKeyFile];
      const maxSessions = receiveConfig.maxSessions ?? settings.maxSessions;
      const trafficLimitBytes = receiveConfig.trafficLimitBytes ?? settings.trafficLimitBytes;
      const rateBytesPerSecond = receiveConfig.rateBytesPerSecond ?? settings.rateBytesPerSecond;
      const timeoutMs = receiveConfig.timeoutMs ?? settings.timeoutMs;
      if (maxSessions !== undefined) argv.push("--max-sessions", String(maxSessions));
      if (trafficLimitBytes !== undefined) argv.push("--traffic-limit-bytes", String(trafficLimitBytes));
      if (rateBytesPerSecond !== undefined) argv.push("--rate-bytes-per-second", String(rateBytesPerSecond));
      if (timeoutMs !== undefined) argv.push("--timeout-ms", String(timeoutMs));
      if (receiveConfig.once !== false) argv.push("--once");
      argv.push("--json");
      const handle = spawnCommand(argv);
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
    cancelAll() {
      for (const operation of operations.values()) {
        if (operation.state === "running") {
          operation.state = "cancelled";
          clearTimeout(operation.timer);
          operation.handle.kill();
        }
      }
      selections.clear();
    },
    async scan(mode, rootId) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      return execute(["scan", mode, "--source", requireSelection(rootId), "--json"]);
    },
    async state(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      if (!stateDirectory) throw new Error("STATE_DISABLED");
      if (action === "inspect") return execute(["state", "inspect", "--directory", stateDirectory, "--json"]);
      const picked = await ports.pick("files", { maxFiles: 1, maxBytes: 1024 * 1024 });
      if (!picked || picked.length === 0) throw new Error("STATE_IMPORT_CANCELLED");
      const file = await realpath(picked[0]!.path);
      const fileInfo = await stat(file);
      if (!fileInfo.isFile()) throw new Error("STATE_IMPORT_FILE_REQUIRED");
      if (fileInfo.size > 1024 * 1024) throw new Error("STATE_IMPORT_TOO_LARGE");
      return execute(["state", "import", "--directory", stateDirectory, "--file", file]);
    },
    async config(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      return execute(["config", action === "defaults" ? "init" : "check", "--json"]);
    },
    async platform(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      const argv = action === "watch"
        ? ["platform", "watch", "--samples", "5", "--interval-ms", "1000", "--json"]
        : ["platform", action, "--json"];
      return execute(argv);
    },
    async hotspot(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      if (config.enableHotspot !== true) throw new Error("HOTSPOT_DISABLED");
      const iface = config.hotspotInterface;
      if (action === "plan") {
        if (!iface) throw new Error("HOTSPOT_INTERFACE_NOT_CONFIGURED");
        return execute(["hotspot", "plan", "--interface", iface, "--ssid", config.hotspotSsid ?? "Global-Dandelion", "--band", config.hotspotBand ?? "a", "--json"]);
      }
      const journalDirectory = config.hotspotJournalDirectory;
      if (!journalDirectory || !isAbsolute(journalDirectory)) throw new Error("HOTSPOT_JOURNAL_NOT_CONFIGURED");
      if (action === "status") return execute(["hotspot", "status", "--journal-directory", journalDirectory, "--json"]);
      if (action === "stop") return execute(["hotspot", "stop", "--journal-directory", journalDirectory, "--confirm", "--json"]);
      if (!iface || !config.hotspotPasswordFile) throw new Error("HOTSPOT_APPLY_NOT_CONFIGURED");
      return execute(["hotspot", "apply", "--interface", iface, "--ssid", config.hotspotSsid ?? "Global-Dandelion", "--band", config.hotspotBand ?? "a", "--journal-directory", journalDirectory, "--password-file", config.hotspotPasswordFile, "--confirm", "--json"]);
    },
    async update(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      if (config.enableUpdate !== true) throw new Error("UPDATE_DISABLED");
      const trust = config.updateTrust;
      if (!trust?.publicKey) throw new Error("UPDATE_TRUST_NOT_CONFIGURED");
      const argv = ["update", action];
      if (trust.manifest) argv.push("--manifest", trust.manifest);
      if (trust.signature) argv.push("--signature", trust.signature);
      argv.push("--public-key", trust.publicKey);
      if (action === "stage") {
        if (!trust.directory) throw new Error("UPDATE_DIRECTORY_NOT_CONFIGURED");
        argv.push("--directory", trust.directory);
        if (trust.artifact) argv.push("--artifact", trust.artifact);
        else if (trust.allowedHost) argv.push("--allowed-host", trust.allowedHost);
        else throw new Error("UPDATE_SOURCE_NOT_CONFIGURED");
      }
      argv.push("--json");
      return execute(argv);
    },
    async diagnostics(action) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      if (action === "status") {
        if (!stateDirectory) throw new Error("STATE_DISABLED");
        return execute(["status", "--directory", stateDirectory, "--json"]);
      }
      return execute([action === "selfTest" ? "self-test" : "doctor", "--json"]);
    },
    async compat(action, args) {
      if (!nativeCliSupported) throw new Error("PLATFORM_UNSUPPORTED");
      if (!compatDirectory) throw new Error("COMPAT_DISABLED");
      if (action === "packetInspect") return execute(["compat", "packet-inspect", "--file", await compatPath(args.file, "packet.bin"), "--json"]);
      if (action === "countryLookup") {
        const address = typeof args.address === "string" ? args.address : null;
        if (!address) throw new Error("COMPAT_ADDRESS_REQUIRED");
        return execute(["compat", "country-lookup", "--locations", await compatPath(args.locations, "locations.csv"), "--blocks", await compatPath(args.blocks, "blocks.csv"), "--address", address, "--json"]);
      }
      if (action === "linkExercise") {
        return execute(["compat", "link-exercise", "--file", await compatPath(args.file, "link-target"), "--cycles", "10", "--interval-ms", "50", "--confirm", "--json"]);
      }
      return execute(["compat", "link-recover", "--journal-directory", await compatPath(args.journalDirectory, "."), "--confirm", "--json"]);
    },
  };
}
