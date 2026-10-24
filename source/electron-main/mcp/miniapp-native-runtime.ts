/** Generic, explicitly authorized native package runner. No application methods. */
import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { chmod, lstat, mkdir, mkdtemp, rm, statfs, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import type { MiniAppToolRouter, MiniAppToolScope, MiniAppToolResult } from "./miniapp-tools.js";
import type { VerifiedMiniAppResource } from "./miniapp-window.js";

const LIMIT = 1024 * 1024;
const DENIED: MiniAppToolResult = { ok: false, error: { code: "NATIVE_RUNTIME_UNAVAILABLE", message: "小程序原生运行时不可用" } };
export interface NativeRuntimeDefinition {
  schema: "fabushi.native-runtime.v1";
  platform: "darwin";
  architecture: "arm64";
  entrypoint: string;
  executables: string[];
  hostCapabilities: string[];
}
function resourcePath(value: unknown): value is string {
  return typeof value === "string" && value.startsWith("native/") && value.length <= 256
    && value.split("/").every(part => /^[A-Za-z0-9._-]+$/.test(part) && part !== "." && part !== "..");
}
export function parseNativeRuntime(raw: unknown, resources: ReadonlyMap<string, VerifiedMiniAppResource>): NativeRuntimeDefinition {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw new Error("MINIAPP_POLICY_DENIED");
  const d = raw as Record<string, unknown>;
  if (Object.keys(d).sort().join() !== "architecture,entrypoint,executables,hostCapabilities,platform,schema"
    || d.schema !== "fabushi.native-runtime.v1" || d.platform !== "darwin" || d.architecture !== "arm64"
    || !resourcePath(d.entrypoint) || !d.entrypoint.endsWith(".cjs")
    || resources.get(d.entrypoint)?.mime !== "text/javascript"
    || !Array.isArray(d.executables) || d.executables.length < 1 || d.executables.length > 8
    || new Set(d.executables).size !== d.executables.length
    || d.executables.some(p => !resourcePath(p) || !p.endsWith(".bin") || resources.get(p)?.mime !== "application/octet-stream")
    || !Array.isArray(d.hostCapabilities) || new Set(d.hostCapabilities).size !== d.hostCapabilities.length
    || d.hostCapabilities.some(p => p !== "file-picker" && p !== "confirmation" && p !== "package-update")) throw new Error("MINIAPP_POLICY_DENIED");
  return d as unknown as NativeRuntimeDefinition;
}
export interface NativeRuntimePorts {
  carrierPath: string;
  codeRoot: string;
  resources: ReadonlyMap<string, VerifiedMiniAppResource>;
  definition: NativeRuntimeDefinition;
  scope: MiniAppToolScope;
  current(): boolean;
  confirm(detail: string): Promise<boolean>;
  pick(mode: "files" | "directory"): Promise<{ path: string; name: string; size: number }[] | null>;
  selfUpdate?(action: "check" | "stage"): Promise<unknown>;
  /** Generic Electron Node carrier, never a package-supplied interpreter path. */
  interpreter?: string;
}
export interface NativeRuntimeHandle {
  router: MiniAppToolRouter;
  cancelAll(): void;
  directory: string;
}
export async function createInstalledNativeRuntime(ports: NativeRuntimePorts): Promise<NativeRuntimeHandle> {
  if (process.platform !== "darwin" || process.arch !== "arm64") throw new Error("MINIAPP_UNSUPPORTED_HOST");
  const definition = parseNativeRuntime(ports.definition, ports.resources);
  if (!ports.current() || !await ports.confirm("此小程序包含原生代码，将在本机执行。仅在信任发布者和此安装版本时允许。\n" + ports.scope.appId + "\n" + ports.scope.version)
    || !ports.current()) throw new Error("MINIAPP_POLICY_DENIED");
  await mkdir(ports.codeRoot, { recursive: true, mode: 0o700 });
  const root = await lstat(ports.codeRoot);
  if (!root.isDirectory() || root.isSymbolicLink() || (root.mode & 0o077) !== 0) throw new Error("MINIAPP_POLICY_DENIED");
  const nativeResources = [...ports.resources].filter(([name]) => name.startsWith("native/"));
  const size = nativeResources.reduce((sum, [, r]) => sum + r.bytes.length, 0);
  const disk = await statfs(ports.codeRoot, { bigint: true });
  const capacity = disk.blocks * disk.bsize;
  const reserve = (capacity + 19n) / 20n > 1073741824n ? (capacity + 19n) / 20n : 1073741824n;
  if (capacity <= 0n || disk.bavail * disk.bsize < reserve + BigInt(size * 2 + LIMIT)) throw new Error("MINIAPP_POLICY_DENIED");
  const directory = await mkdtemp(join(ports.codeRoot, "runtime-"));
  try {
    await chmod(directory, 0o700);
    const dataBase = join(ports.codeRoot, "data");
    await mkdir(dataBase, { recursive: true, mode: 0o700 });
    const dataInfo = await lstat(dataBase);
    if (!dataInfo.isDirectory() || dataInfo.isSymbolicLink() || (dataInfo.mode & 0o077) !== 0) throw new Error("MINIAPP_POLICY_DENIED");
    const identity = createHash("sha256").update(ports.scope.accountScope).update("\0").update(ports.scope.appId).digest("hex");
    const dataDirectory = join(dataBase, identity);
    await mkdir(dataDirectory, { recursive: true, mode: 0o700 });
    const privateInfo = await lstat(dataDirectory);
    if (!privateInfo.isDirectory() || privateInfo.isSymbolicLink() || (privateInfo.mode & 0o077) !== 0) throw new Error("MINIAPP_POLICY_DENIED");
    for (const [name, resource] of nativeResources) {
      if (!resourcePath(name)) throw new Error("MINIAPP_DESCRIPTOR_INVALID");
      const destination = join(directory, name);
      await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
      await writeFile(destination, resource.bytes, { flag: "wx", mode: definition.executables.includes(name) ? 0o500 : 0o400 });
    }
    if (!ports.current()) throw new Error("MINIAPP_SCOPE_CHANGED");
    // Avoid inheriting authentication, Node injection options or domain overrides.
    const env: NodeJS.ProcessEnv = { ELECTRON_RUN_AS_NODE: "1", LANG: "en_US.UTF-8", PATH: "/usr/bin:/bin", TMPDIR: directory, FABUSHI_NATIVE_DATA_DIRECTORY: dataDirectory };
    const child = spawn(ports.interpreter ?? process.execPath, [ports.carrierPath, join(directory, definition.entrypoint), directory], {
      cwd: directory, env, detached: true, stdio: ["pipe", "pipe", "pipe"],
    });
    let closed = false, buffer = "", stderrBytes = 0;
    const pending = new Map<string, { resolve(value: MiniAppToolResult): void; timer: ReturnType<typeof setTimeout>; current(): boolean; hostCalls: number }>();
    const send = (value: unknown) => {
      const line = JSON.stringify(value);
      if (Buffer.byteLength(line) > LIMIT || closed || !child.stdin.writable) throw new Error("MINIAPP_CLOSED");
      child.stdin.write(line + "\n");
    };
    const signalGroup = (signal: NodeJS.Signals) => { if (child.pid) { try { process.kill(-child.pid, signal); } catch { /* already exited */ } } };
    const close = () => {
      if (closed) return;
      closed = true;
      for (const p of pending.values()) { clearTimeout(p.timer); p.resolve(DENIED); }
      pending.clear();
      signalGroup("SIGTERM");
      const timer = setTimeout(() => {
        if (child.exitCode === null && child.signalCode === null) signalGroup("SIGKILL");
        void rm(directory, { recursive: true, force: true });
      }, 4000);
      timer.unref();
    };
    child.on("error", close);
    child.on("exit", close);
    child.stdin.on("error", close);
    child.stderr.on("data", (chunk: Buffer) => { stderrBytes += chunk.length; if (stderrBytes > 65536) close(); });
    const handleLine = async (line: string) => {
      let message: Record<string, unknown>;
      try { message = JSON.parse(line) as Record<string, unknown>; } catch { close(); return; }
      if (!message || typeof message !== "object" || typeof message.id !== "string") { close(); return; }
      const request = pending.get(message.id);
      if (!request || !ports.current() || !request.current()) { close(); return; }
      if (message.kind === "host") {
        if (typeof message.callId !== "string" || !/^[a-zA-Z0-9-]{1,80}$/.test(message.callId) || ++request.hostCalls > 8) { close(); return; }
        let result: unknown = null;
        if (message.method === "confirmation" && definition.hostCapabilities.includes("confirmation") && typeof message.detail === "string" && message.detail.length <= 256) {
          result = await ports.confirm(message.detail);
        } else if (message.method === "file-picker" && definition.hostCapabilities.includes("file-picker") && (message.mode === "files" || message.mode === "directory")) {
          result = await ports.pick(message.mode);
        } else if (message.method === "package-update" && definition.hostCapabilities.includes("package-update") && ports.selfUpdate && (message.action === "check" || message.action === "stage")) {
          try { result = await ports.selfUpdate(message.action); } catch {
            if (!closed && ports.current() && request.current()) send({ kind: "host-result", callId: message.callId, error: "HOST_OPERATION_FAILED" });
            return;
          }
        } else { close(); return; }
        if (!closed && ports.current() && request.current()) send({ kind: "host-result", callId: message.callId, result });
        return;
      }
      if (message.kind !== "result" || !message.result || typeof message.result !== "object" || typeof (message.result as { ok?: unknown }).ok !== "boolean") { close(); return; }
      clearTimeout(request.timer); pending.delete(message.id);
      request.resolve(message.result as MiniAppToolResult);
    };
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      buffer += chunk;
      if (Buffer.byteLength(buffer) > LIMIT) { close(); return; }
      let end: number;
      while ((end = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, end); buffer = buffer.slice(end + 1);
        void handleLine(line).catch(close);
      }
    });
    return { directory, cancelAll: close, router: {
      async invoke(scope, method, args, isCurrent = () => true) {
        if (closed || !ports.current() || !isCurrent() || pending.size >= 8 || typeof method !== "string" || !/^[A-Za-z][A-Za-z0-9.]{0,127}$/.test(method)
          || ["appId", "version", "packageDigest", "accountScope", "accountGeneration"].some(key => Reflect.get(scope, key) !== Reflect.get(ports.scope, key))) return DENIED;
        const id = randomUUID();
        return new Promise<MiniAppToolResult>(resolve => {
          const timer = setTimeout(() => { close(); }, 180000); timer.unref();
          pending.set(id, { resolve, timer, current: isCurrent, hostCalls: 0 });
          try { send({ kind: "invoke", id, method, args, scope }); } catch { close(); }
        });
      },
    } };
  } catch (error) { await rm(directory, { recursive: true, force: true }); throw error; }
}
