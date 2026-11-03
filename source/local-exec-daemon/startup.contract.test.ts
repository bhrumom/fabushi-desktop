import assert from "node:assert/strict";
import { spawn, type ChildProcess } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { createReadStream } from "node:fs";
import { chmod, copyFile, cp, mkdir, mkdtemp, open, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";
import type { Socket } from "node:net";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import test, { type TestContext } from "node:test";

import { resolveAppHostExecutable } from "../electron-main/app-host-executable.js";
import { spawnLocalExecDaemon } from "../electron-main/local-exec/local-exec-native.js";

// The byte-transfer contract uses the production native launcher and a controlled
// Electron resource context, under ordinary Node with its explicit Node stage.
// Real Electron ABI loading has separate contracts. Never import main.ts here:
// its entry guard would see this bundle's URL and start a second daemon.
const STARTUP_TIMEOUT_MS = 10_000;
const CLOSE_TIMEOUT_MS = 5_000;
const OUTPUT_LIMIT_BYTES = 64 * 1024;
const PROVIDER_ID = "ldb-real-provider";
const TOKEN = "ldb-loopback-only-token";
const DISCOVERY_FILENAME = "local-exec-daemon.json";
const CONNECTION_FILENAME = "local-exec-daemon-connection.json";

type JsonRecord = Record<string, unknown>;
interface ChildOutcome { readonly code: number | null; readonly signal: NodeJS.Signals | null }
interface RuntimePaths { readonly entry: string; readonly host: string; readonly nodeStage: string }
interface ReceivedFrame { readonly providerId: unknown; readonly frame: JsonRecord }

function isRecord(value: unknown): value is JsonRecord {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

async function within<T>(operation: Promise<T>, timeoutMs: number, label: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      operation,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(label + " timed out")), timeoutMs);
      }),
    ]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

async function requiredRuntimePaths(): Promise<RuntimePaths> {
  const entry = process.env.FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY;
  const host = process.env.FABUSHI_LOCAL_EXEC_HOST_TEST_BIN;
  assert.ok(entry, "FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY must identify the actual emitted daemon");
  assert.ok(host, "FABUSHI_LOCAL_EXEC_HOST_TEST_BIN must identify the same-H Host binary");
  const canonicalEntry = await realpath(entry);
  return {
    entry: canonicalEntry,
    host: await realpath(host),
    nodeStage: await realpath(join(dirname(canonicalEntry), "..", "node-deps")),
  };
}

class OwnedChild {
  readonly child: ChildProcess;
  readonly closed: Promise<ChildOutcome>;
  outcome: ChildOutcome | undefined;
  spawnError: Error | undefined;
  stdout = Buffer.alloc(0);
  stderr = Buffer.alloc(0);
  outputOverflow = false;

  static spawn(args: readonly string[], env: NodeJS.ProcessEnv): OwnedChild {
    return new OwnedChild(spawn(process.execPath, [...args], {
      env,
      stdio: ["ignore", "pipe", "pipe"],
    }));
  }

  static fromNativeLaunch(child: ChildProcess, logPath: string): OwnedChild {
    return new OwnedChild(child, logPath);
  }

  private constructor(child: ChildProcess, private readonly logPath?: string) {
    // Keep the actual child handle, including the one returned by the production
    // native launcher. No PID discovery or global process killing.
    this.child = child;
    this.child.on("error", (error: Error) => { this.spawnError = error; });
    this.child.stdout?.on("error", (error: Error) => { this.spawnError ??= error; });
    this.child.stderr?.on("error", (error: Error) => { this.spawnError ??= error; });
    this.child.stdout?.on("data", (chunk: Buffer) => this.capture("stdout", chunk));
    this.child.stderr?.on("data", (chunk: Buffer) => this.capture("stderr", chunk));
    this.closed = new Promise((resolve) => {
      this.child.once("close", (code, signal) => {
        this.outcome = { code, signal };
        resolve(this.outcome);
      });
    });
  }

  private capture(channel: "stdout" | "stderr", chunk: Buffer): void {
    const remaining = OUTPUT_LIMIT_BYTES - this[channel].length;
    if (chunk.length > remaining) this.outputOverflow = true;
    if (remaining > 0) this[channel] = Buffer.concat([this[channel], chunk.subarray(0, remaining)]);
  }

  diagnostics(): string {
    return JSON.stringify({
      pid: this.child.pid,
      outcome: this.outcome,
      spawnError: this.spawnError?.message,
      outputOverflow: this.outputOverflow,
      logPath: this.logPath,
      stdout: this.stdout.toString("utf8"),
      stderr: this.stderr.toString("utf8"),
    });
  }

  private async captureNativeLog(): Promise<void> {
    if (this.logPath === undefined) return;
    const handle = await open(this.logPath, "r");
    try {
      const { size } = await handle.stat();
      if (size > OUTPUT_LIMIT_BYTES) this.outputOverflow = true;
      const bytes = Buffer.alloc(Math.min(size, OUTPUT_LIMIT_BYTES));
      const { bytesRead } = await handle.read(bytes, 0, bytes.length, 0);
      this.stderr = bytes.subarray(0, bytesRead);
    } finally { await handle.close(); }
  }

  async waitForClose(timeoutMs = CLOSE_TIMEOUT_MS): Promise<ChildOutcome> {
    try {
      const outcome = await within(this.closed, timeoutMs, "owned child close");
      await this.captureNativeLog();
      return outcome;
    } catch (error) { throw new Error(String(error) + "\n" + this.diagnostics()); }
  }

  signal(signal: NodeJS.Signals): void {
    assert.equal(this.outcome, undefined, "only an unsettled owned child may be signalled");
    assert.equal(this.child.kill(signal), true, "owned child signal must be delivered");
  }

  async cleanup(): Promise<void> {
    if (this.outcome !== undefined) return;
    // This is failure cleanup, never evidence of a successful graceful stop.
    this.child.kill("SIGTERM");
    try { await this.waitForClose(); }
    catch {
      if (this.outcome === undefined) this.child.kill("SIGKILL");
      await this.waitForClose();
    }
  }

  assertExit(code: number): void {
    assert.equal(this.spawnError, undefined, this.diagnostics());
    assert.equal(this.outputOverflow, false, this.diagnostics());
    assert.deepEqual(this.outcome, { code, signal: null }, this.diagnostics());
  }
}

async function createGateway() {
  const streams = new Set<ServerResponse>();
  const sockets = new Set<Socket>();
  const received: ReceivedFrame[] = [];
  const errors: string[] = [];
  let requestStreams = 0;
  let responsePosts = 0;
  let closing = false;
  let providerDisconnectExpected = false;
  const disconnects: string[] = [];
  const recordSocketError = (error: Error): void => {
    if (closing) return;
    if (providerDisconnectExpected && isRecord(error) && (error.code === "ECONNRESET" || error.code === "EPIPE")) {
      disconnects.push(error.message);
    } else errors.push(error.message);
  };
  const server = createServer((request, response) => {
    response.on("error", recordSocketError);
    void (async () => {
      if (request.headers.authorization !== "Bearer " + TOKEN) {
        throw new Error("gateway request did not carry the configured token");
      }
      if (request.method === "GET" && request.url === "/local-exec/requests") {
        assert.equal(request.headers.accept, "text/event-stream");
        requestStreams += 1;
        streams.add(response);
        response.on("close", () => streams.delete(response));
        response.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache" });
        response.flushHeaders();
        response.write("data: " + JSON.stringify({ kind: "welcome", providerId: PROVIDER_ID }) + "\n\n");
        return;
      }
      if (request.method === "POST" && request.url === "/local-exec/responses") {
        responsePosts += 1;
        assert.equal(request.headers["content-type"], "application/json");
        const chunks: Buffer[] = [];
        let size = 0;
        for await (const value of request) {
          const chunk = Buffer.isBuffer(value) ? value : Buffer.from(value);
          size += chunk.length;
          assert.ok(size <= 512 * 1024, "fixture response body must remain bounded");
          chunks.push(chunk);
        }
        const body: unknown = JSON.parse(Buffer.concat(chunks).toString("utf8"));
        assert.ok(isRecord(body));
        assert.ok(Array.isArray(body.frames));
        for (const frame of body.frames) {
          assert.ok(isRecord(frame));
          received.push({ providerId: body.providerId, frame });
        }
        response.writeHead(204);
        response.end();
        return;
      }
      throw new Error("unexpected gateway route: " + request.method + " " + request.url);
    })().catch((error: unknown) => {
      if (!closing) errors.push(String(error));
      if (!response.headersSent) response.writeHead(500);
      response.end();
    });
  });
  server.on("connection", (socket) => {
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
    socket.on("error", recordSocketError);
  });
  server.on("error", (error: Error) => { if (!closing) errors.push(error.message); });
  const close = async (): Promise<void> => {
    closing = true;
    const closed = new Promise<void>((resolve, reject) => {
      server.close((error) => {
        if (error === undefined || (isRecord(error) && error.code === "ERR_SERVER_NOT_RUNNING")) resolve();
        else reject(error);
      });
    });
    for (const response of streams) response.destroy();
    for (const socket of sockets) socket.destroy();
    await within(closed, CLOSE_TIMEOUT_MS, "loopback gateway close");
  };
  try {
    await within(new Promise<void>((resolve, reject) => {
      const failed = (error: Error): void => { server.off("listening", listening); reject(error); };
      const listening = (): void => { server.off("error", failed); resolve(); };
      server.once("error", failed);
      server.once("listening", listening);
      server.listen(0, "127.0.0.1");
    }), CLOSE_TIMEOUT_MS, "loopback gateway listen");
  } catch (error) {
    try { await close(); }
    catch (cleanupError) { throw new AggregateError([error, cleanupError], "gateway startup and cleanup failed"); }
    throw error;
  }
  const address = server.address();
  assert.ok(address !== null && typeof address !== "string");

  return {
    baseUrl: "http://127.0.0.1:" + address.port,
    received,
    errors,
    disconnects,
    expectProviderDisconnect(): void { providerDisconnectExpected = true; },
    get requestStreams() { return requestStreams; },
    get responsePosts() { return responsePosts; },
    get openStreams() { return streams.size; },
    send(frame: JsonRecord): void {
      assert.equal(streams.size, 1, "send only through the real provider's single SSE connection");
      const response = [...streams][0];
      assert.ok(response !== undefined && !response.destroyed);
      response.write("data: " + JSON.stringify(frame) + "\n\n");
    },
    close,
  };
}

async function createFixture() {
  const runtime = await requiredRuntimePaths();
  const directory = await mkdtemp(join(tmpdir(), "ldb-emitted-daemon-"));
  const dataRoot = join(directory, "data");
  const fileRoot = join(directory, "files");
  let gateway: Awaited<ReturnType<typeof createGateway>> | undefined;
  try {
    await Promise.all([mkdir(dataRoot), mkdir(fileRoot)]);
    gateway = await createGateway();
    const generation = randomUUID();
    await writeFile(join(dataRoot, CONNECTION_FILENAME), JSON.stringify({
      baseUrl: gateway.baseUrl, token: TOKEN,
    }), { mode: 0o600 });
    await writeFile(join(dataRoot, "settings.json"), JSON.stringify({
      version: 1,
      localToolPermission: "always",
      settingsMigrations: ["downgrade-persisted-max-fast"],
    }));
    await writeFile(join(dataRoot, "local-exec-supervisor.json"), JSON.stringify({
      pid: process.pid, at: Date.now(),
    }));
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      SAND_PACKAGED: "1",
      SAND_DATA_ROOT: dataRoot,
      SAND_LOCAL_EXEC_ROOT: fileRoot,
      SAND_LOCAL_EXEC_GENERATION_TOKEN: generation,
      SAND_TREE_SITTER_NODE_DEPS: runtime.nodeStage,
      FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY: runtime.entry,
    };
    // Ordinary Node must not be misclassified as Electron by node-gyp-build.
    // Exclude inherited native-loader, preload and external telemetry overrides.
    for (const name of [
      "ELECTRON_RUN_AS_NODE", "TREE_SITTER_PREBUILD", "TREE_SITTER_BASH_PREBUILD",
      "PREBUILDS_ONLY", "NODE_OPTIONS", "NODE_PATH",
      "SAND_SENTRY_ENVIRONMENT", "SAND_SENTRY_RELEASE", "MAHAYANA_APP_HOST_BIN",
    ]) delete env[name];

    return {
      runtime, directory, dataRoot, fileRoot, gateway, generation, env,
      discoveryPath: join(dataRoot, DISCOVERY_FILENAME),
    };
  } catch (error) {
    try { await gateway?.close(); }
    catch (cleanupError) { throw new AggregateError([error, cleanupError], "fixture setup failed; retained " + directory); }
    throw new Error("fixture setup failed; retained " + directory, { cause: error });
  }
}
type Fixture = Awaited<ReturnType<typeof createFixture>>;

async function sha256File(path: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

async function stageFixtureHost(fixture: Fixture) {
  const resourcesPath = join(fixture.directory, "resources");
  const bin = join(resourcesPath, "bin");
  const host = join(bin, process.platform === "win32" ? "mahayana-app-host.exe" : "mahayana-app-host");
  await mkdir(bin, { recursive: true });
  const sourceHash = await sha256File(fixture.runtime.host);
  await copyFile(fixture.runtime.host, host);
  if (process.platform !== "win32") await chmod(host, 0o755);
  assert.equal(await sha256File(host), sourceHash, "staged Host must retain the same-H executable bytes");
  assert.equal(await sha256File(fixture.runtime.host), sourceHash, "Host source must remain unchanged while staging");
  return { resourcesPath, host, sha256: sourceHash };
}

async function waitUntil(
  fixture: Fixture,
  child: OwnedChild,
  label: string,
  predicate: () => boolean | Promise<boolean>,
): Promise<void> {
  const deadline = performance.now() + STARTUP_TIMEOUT_MS;
  while (performance.now() < deadline) {
    assert.deepEqual(fixture.gateway.errors, [], label + "\n" + child.diagnostics());
    if (await predicate()) return;
    assert.equal(child.outcome, undefined, label + ": child closed early\n" + child.diagnostics());
    assert.equal(child.spawnError, undefined, child.diagnostics());
    await delay(10);
  }
  assert.fail(label + " timed out\n" + child.diagnostics());
}

async function readDiscovery(path: string): Promise<JsonRecord | undefined> {
  try {
    const value: unknown = JSON.parse(await readFile(path, "utf8"));
    assert.ok(isRecord(value), "discovery must be an object");
    return value;
  } catch (error) {
    if (isRecord(error) && error.code === "ENOENT") return undefined;
    throw error;
  }
}

async function fileReply(fixture: Fixture, child: OwnedChild, request: JsonRecord): Promise<JsonRecord> {
  fixture.gateway.send(request);
  await waitUntil(fixture, child, "file reply " + String(request.requestId), () =>
    fixture.gateway.received.some(({ frame }) => frame.requestId === request.requestId));
  const matches = fixture.gateway.received.filter(({ frame }) => frame.requestId === request.requestId);
  assert.equal(matches.length, 1, "one response per file request");
  const reply = matches[0];
  assert.ok(reply);
  assert.equal(reply.providerId, PROVIDER_ID);
  assert.equal(reply.frame.kind, "file", JSON.stringify(reply.frame) + "\n" + child.diagnostics());
  return reply.frame;
}

async function finishFixture(
  t: TestContext,
  fixture: Fixture,
  child: OwnedChild | undefined,
  passed: boolean,
): Promise<void> {
  let cleanupError: unknown;
  try { await child?.cleanup(); } catch (error) { cleanupError = error; }
  try { await fixture.gateway.close(); } catch (error) { cleanupError ??= error; }
  const terminalObserved = child === undefined || child.outcome !== undefined;
  if (passed && terminalObserved && cleanupError === undefined) {
    await rm(fixture.directory, { recursive: true, force: true });
  } else {
    await writeFile(join(fixture.directory, "child-observation.json"), child?.diagnostics() ?? "no child");
    await writeFile(join(fixture.directory, "gateway-observation.json"), JSON.stringify({
      errors: fixture.gateway.errors,
      disconnects: fixture.gateway.disconnects,
      received: fixture.gateway.received,
      requestStreams: fixture.gateway.requestStreams,
      responsePosts: fixture.gateway.responsePosts,
    }, null, 2));
    t.diagnostic("retained LDB fixture: " + fixture.directory);
  }
  if (cleanupError !== undefined) throw cleanupError;
}

test("ldb Host resolver preserves packaged development and explicit override mappings", () => {
  const root = resolve(tmpdir(), "ldb-host-path-contract");
  const resourcesPath = join(root, "installed", "resources");
  const moduleDirectory = join(root, "desktop", "dist", "electron-main");
  for (const platform of ["darwin", "linux", "win32"] as const) {
    const context = { platform, resourcesPath, moduleDirectory };
    const name = platform === "win32" ? "mahayana-app-host.exe" : "mahayana-app-host";
    assert.equal(resolveAppHostExecutable({ SAND_PACKAGED: "1" }, context), join(resourcesPath, "bin", name));
    assert.equal(resolveAppHostExecutable({ SAND_PACKAGED: "0" }, context), join(root, "desktop", "resources", "bin", name));
    assert.equal(resolveAppHostExecutable({}, {
      ...context, moduleDirectory: join(root, "desktop", "dist", "node-agent-coordinator"),
    }), join(root, "desktop", "resources", "bin", name), "carrier and main share development staging");
    assert.equal(resolveAppHostExecutable({ SAND_PACKAGED: "1", MAHAYANA_APP_HOST_BIN: "  " }, context), join(resourcesPath, "bin", name));
    const override = join(root, "developer host", name);
    for (const packaged of ["0", "1"]) {
      assert.equal(resolveAppHostExecutable({
        SAND_PACKAGED: packaged, MAHAYANA_APP_HOST_BIN: override,
      }, { ...context, resourcesPath: undefined }), override, "explicit developer override takes precedence");
    }
    assert.throws(() => resolveAppHostExecutable({ SAND_PACKAGED: "1" }, {
      ...context, resourcesPath: undefined,
    }), /requires Electron resourcesPath/, "packaged launch cannot fall back to development resources");
  }
});

test("ldb emitted daemon transfers bytes through real provider and retires owned discovery", { timeout: 90_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    assert.equal(process.env.MAHAYANA_APP_HOST_BIN, undefined, "native launch must start without an inherited Host binding");
    assert.equal(Object.hasOwn(fixture.env, "MAHAYANA_APP_HOST_BIN"), false, "fixture must not supply the Host binding");
    const stagedHost = await stageFixtureHost(fixture);
    const logPath = join(fixture.directory, "native-launch.log");
    const launched = await spawnLocalExecDaemon({
      logPath,
      env: fixture.env,
      mainPath: fixture.runtime.entry,
      generationToken: fixture.generation,
      appHostContext: {
        platform: process.platform,
        resourcesPath: stagedHost.resourcesPath,
        moduleDirectory: join(fixture.directory, "dist", "electron-main"),
      },
    });
    child = OwnedChild.fromNativeLaunch(launched.child, logPath);
    assert.equal(launched.entryRealpath, fixture.runtime.entry);
    assert.equal(launched.generationToken, fixture.generation);
    assert.equal(process.env.MAHAYANA_APP_HOST_BIN, undefined, "binding stays in the child environment");
    assert.equal(Object.hasOwn(fixture.env, "MAHAYANA_APP_HOST_BIN"), false, "launcher must not mutate caller environment");
    await waitUntil(fixture, child, "matching discovery and real provider hello", async () => {
      const discovery = await readDiscovery(fixture.discoveryPath);
      return discovery !== undefined
        && fixture.gateway.received.some(({ frame }) => frame.kind === "hello");
    });
    const discovery = await readDiscovery(fixture.discoveryPath);
    assert.ok(discovery);
    assert.equal(discovery.pid, child.child.pid);
    assert.equal(discovery.entryRealpath, fixture.runtime.entry);
    assert.equal(discovery.generationToken, fixture.generation);
    assert.ok(typeof discovery.startedAt === "number");
    assert.ok(Number.isFinite(discovery.startedAt) && discovery.startedAt <= Date.now());
    assert.equal(discovery.inflightCount, 0);
    const hello = fixture.gateway.received.find(({ frame }) => frame.kind === "hello")?.frame;
    assert.ok(hello);
    assert.equal(hello.localRoot, fixture.fileRoot);
    assert.equal(hello.terminalsFolder, join(fixture.fileRoot, "terminals"));
    assert.equal(hello.supervised, true);
    assert.equal(fixture.gateway.requestStreams, 1);

    const bytes = Buffer.concat([Buffer.from("LDB real emitted provider\n"), Buffer.from([0, 1, 127, 128, 255])]);
    const hash = createHash("sha256").update(bytes).digest("hex");
    const target = join(fixture.fileRoot, "published.bin");
    const transferId = "ldb-" + randomUUID();
    const start = await fileReply(fixture, child, {
      kind: "upload-start", requestId: "ldb-start", transferId, path: target,
      totalBytes: bytes.length, sha256: hash, overwrite: false,
    });
    assert.equal(start.size, 0);
    const chunk = await fileReply(fixture, child, {
      kind: "upload-chunk", requestId: "ldb-chunk", transferId, path: target,
      offset: 0, bytesBase64: bytes.toString("base64"),
    });
    assert.equal(chunk.size, bytes.length);
    const committed = await fileReply(fixture, child, {
      kind: "upload-commit", requestId: "ldb-commit", transferId, path: target,
    });
    assert.equal(committed.size, bytes.length);
    assert.equal(committed.sha256, hash);
    assert.ok(typeof committed.version === "string" && committed.version.length > 0);
    assert.deepEqual(await readFile(target), bytes, "independent destination observation");
    const downloaded = await fileReply(fixture, child, {
      kind: "download", requestId: "ldb-download", path: target,
    });
    assert.equal(downloaded.bytesBase64, bytes.toString("base64"));
    assert.equal(downloaded.sha256, hash);
    assert.equal(downloaded.version, committed.version);
    assert.equal(downloaded.size, bytes.length);

    fixture.gateway.expectProviderDisconnect();
    child.signal("SIGTERM");
    await child.waitForClose();
    child.assertExit(0);
    assert.equal(await readDiscovery(fixture.discoveryPath), undefined, "owned discovery retired before exit");
    const streamCloseDeadline = performance.now() + CLOSE_TIMEOUT_MS;
    while (fixture.gateway.openStreams !== 0 && performance.now() < streamCloseDeadline) await delay(10);
    assert.equal(fixture.gateway.openStreams, 0, "provider SSE closed with its owned process");
    assert.deepEqual(fixture.gateway.errors, []);
    for (const id of ["ldb-start", "ldb-chunk", "ldb-commit", "ldb-download"]) {
      assert.equal(fixture.gateway.received.filter(({ frame }) => frame.requestId === id).length, 1);
    }
    t.diagnostic(JSON.stringify({
      domain: "packaged-node", entry: fixture.runtime.entry, host: stagedHost.host,
      sourceHost: fixture.runtime.host, hostSha256: stagedHost.sha256,
      resourcesPath: stagedHost.resourcesPath, hostBinding: "production-native-launch",
      inheritedHostBinding: false, fixtureHostBinding: false, logPath,
      nodeStage: fixture.runtime.nodeStage, pid: child.child.pid, generation: fixture.generation,
      transferredBytes: bytes.length, sha256: hash, close: child.outcome,
    }));
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});

test("ldb emitted daemon refuses mismatched generation before registration", { timeout: 30_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    child = OwnedChild.spawn([
      fixture.runtime.entry, "--sand-local-exec-generation=" + fixture.generation + "-wrong",
    ], fixture.env);
    await child.waitForClose(STARTUP_TIMEOUT_MS);
    child.assertExit(1);
    assert.equal(await readDiscovery(fixture.discoveryPath), undefined);
    assert.equal(fixture.gateway.requestStreams, 0);
    assert.equal(fixture.gateway.responsePosts, 0);
    assert.deepEqual(fixture.gateway.errors, []);
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});

test("ldb emitted daemon refuses missing configured Node parser domain without fallback", { timeout: 30_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    const unavailableStage = join(fixture.directory, "unavailable-node-stage");
    child = OwnedChild.spawn([
      fixture.runtime.entry, "--sand-local-exec-generation=" + fixture.generation,
    ], { ...fixture.env, SAND_TREE_SITTER_NODE_DEPS: unavailableStage });
    await child.waitForClose(STARTUP_TIMEOUT_MS);
    child.assertExit(1);
    const error = child.stderr.toString("utf8");
    assert.ok(error.includes("SAND_TREE_SITTER_RUNTIME_UNAVAILABLE"), child.diagnostics());
    assert.ok(error.includes("node dist/node-deps"), child.diagnostics());
    assert.ok(error.includes(unavailableStage), child.diagnostics());
    assert.equal(await readDiscovery(fixture.discoveryPath), undefined);
    assert.equal(fixture.gateway.requestStreams, 0);
    assert.equal(fixture.gateway.responsePosts, 0);
    assert.deepEqual(fixture.gateway.errors, []);
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});

async function copyQualifiedNodeStage(fixture: Fixture): Promise<{ root: string; manifest: JsonRecord }> {
  const root = join(fixture.directory, "copied-node-stage");
  await cp(fixture.runtime.nodeStage, root, { recursive: true, dereference: false, errorOnExist: true, force: false });
  const original = await readFile(join(fixture.runtime.nodeStage, "local-exec-dependencies.json"), "utf8");
  assert.equal(await readFile(join(root, "local-exec-dependencies.json"), "utf8"), original);
  const manifest: unknown = JSON.parse(original);
  assert.ok(isRecord(manifest));
  assert.equal(manifest.contract, "LDB-v1");
  assert.equal(manifest.domain, "node");
  assert.deepEqual(manifest.runtime, {
    version: process.versions.node, node: process.versions.node, modules: process.versions.modules,
    platform: process.platform, arch: process.arch,
  });
  assert.ok(Array.isArray(manifest.native_outputs) && manifest.native_outputs.length === 2);
  assert.deepEqual(manifest.native_outputs.map(output => {
    assert.ok(isRecord(output) && typeof output.path === "string");
    return output.path;
  }).sort(), ["tree-sitter-bash/build/Release/tree_sitter_bash_binding.node",
    "tree-sitter/build/Release/tree_sitter_runtime_binding.node"].sort());
  for (const output of manifest.native_outputs) {
    assert.ok(isRecord(output) && typeof output.path === "string" && typeof output.sha256 === "string");
    assert.equal(await sha256File(join(root, output.path)), output.sha256, "copied native bytes retain their qualified identity");
    assert.equal(await sha256File(join(fixture.runtime.nodeStage, output.path)), output.sha256, "original stage is unchanged");
  }
  return { root, manifest };
}

async function assertNoParserRegistration(fixture: Fixture, child: OwnedChild, stage: string, marker: string): Promise<void> {
  await child.waitForClose(STARTUP_TIMEOUT_MS);
  child.assertExit(1);
  const error = child.stderr.toString("utf8");
  for (const expected of ["SAND_TREE_SITTER_RUNTIME_UNAVAILABLE", "node dist/node-deps", stage, marker]) {
    assert.ok(error.includes(expected), child.diagnostics());
  }
  assert.equal(await readDiscovery(fixture.discoveryPath), undefined);
  assert.equal(fixture.gateway.requestStreams, 0);
  assert.equal(fixture.gateway.responsePosts, 0);
  assert.deepEqual(fixture.gateway.received, []);
  assert.deepEqual(fixture.gateway.errors, []);
}

test("ldb emitted daemon refuses wrong-domain parser manifest before registration", { timeout: 30_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    const copied = await copyQualifiedNodeStage(fixture);
    // Binaries and every other receipt field are still the qualified Node stage;
    // an incidental Electron/Node ABI failure cannot decide this negative.
    await writeFile(join(copied.root, "local-exec-dependencies.json"), JSON.stringify({ ...copied.manifest, domain: "electron" }));
    child = OwnedChild.spawn([
      fixture.runtime.entry, "--sand-local-exec-generation=" + fixture.generation,
    ], { ...fixture.env, SAND_TREE_SITTER_NODE_DEPS: copied.root });
    await assertNoParserRegistration(fixture, child, copied.root, "LDB-v1 stage domain mismatch");
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});

test("ldb emitted daemon refuses mismatched parser runtime before registration", { timeout: 90_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    const copied = await copyQualifiedNodeStage(fixture);
    assert.ok(isRecord(copied.manifest.runtime));
    const runtime = copied.manifest.runtime;
    const changes: Record<string, string> = {
      version: "0.0.0", node: "0.0.0", modules: "0",
      platform: process.platform === "linux" ? "darwin" : "linux",
      arch: process.arch === "x64" ? "arm64" : "x64",
    };
    for (const [field, value] of Object.entries(changes)) {
      assert.notEqual(runtime[field], value);
      await writeFile(join(copied.root, "local-exec-dependencies.json"), JSON.stringify({
        ...copied.manifest, runtime: { ...runtime, [field]: value },
      }));
      child = OwnedChild.spawn([
        fixture.runtime.entry, "--sand-local-exec-generation=" + fixture.generation,
      ], { ...fixture.env, SAND_TREE_SITTER_NODE_DEPS: copied.root });
      await assertNoParserRegistration(fixture, child, copied.root, "LDB-v1 stage runtime mismatch (" + field + ")");
      t.diagnostic("actual emitted-entry runtime refusal: " + field);
    }
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});

// Require the emitted entry without an argv[1], so its direct-entry condition
// stays false. The callbacks below are the real process EventEmitter listeners;
// the probe never imports a source substitute or injects a provider/executor.
const EVENT_ADAPTER_PROBE = [
  'const assert = require("node:assert/strict");',
  'const fs = require("node:fs");',
  'const path = require("node:path");',
  'const { createRequire } = require("node:module");',
  'const entry = process.env.FABUSHI_LOCAL_EXEC_DAEMON_TEST_ENTRY;',
  'assert.equal(process.argv[1], undefined);',
  'const fromEntry = createRequire(entry);',
  'const { installLocalExecDaemonProcessCrashGuards } = fromEntry(entry);',
  'assert.equal(typeof installLocalExecDaemonProcessCrashGuards, "function");',
  'const libraryRoot = fs.realpathSync(path.join(path.dirname(entry), "..", "node_modules", "jschardet"));',
  'const libraryEntry = fs.realpathSync(fromEntry.resolve("jschardet"));',
  'assert.ok(libraryEntry.startsWith(libraryRoot + path.sep), "jschardet must resolve from the separately retained dist library");',
  'assert.equal(fromEntry(path.join(libraryRoot, "package.json")).version, "3.1.4");',
  'assert.ok(require.cache[libraryEntry], "requiring the real main must have loaded its external jschardet dependency");',
  'const originalConsoleError = console.error;',
  'const logs = [];',
  'console.error = (...args) => logs.push(args);',
  'const originalCalls = [];',
  'const replacementCalls = [];',
  'const guard = installLocalExecDaemonProcessCrashGuards({ scope: "ldb-events", onError: (error, kind) => originalCalls.push({ error, kind }) });',
  'const original = new Error("original-error");',
  'assert.equal(process.emit("uncaughtException", original), true);',
  'assert.equal(originalCalls.length, 1);',
  'assert.equal(originalCalls[0].error, original);',
  'assert.equal(originalCalls[0].kind, "uncaughtException");',
  'guard.setReporter((error, kind) => replacementCalls.push({ error, kind }));',
  'assert.equal(process.emit("unhandledRejection", 17), true);',
  'assert.equal(originalCalls.length, 1);',
  'assert.equal(replacementCalls.length, 1);',
  'assert.ok(replacementCalls[0].error instanceof Error);',
  'assert.equal(replacementCalls[0].error.message, "17");',
  'assert.equal(replacementCalls[0].kind, "unhandledRejection");',
  'let throwingCalls = 0;',
  'guard.setReporter(() => { throwingCalls += 1; throw new Error("reporter-failure"); });',
  'assert.doesNotThrow(() => process.emit("uncaughtException", "reporter-contained"));',
  'assert.equal(throwingCalls, 1);',
  'guard.setReporter(undefined);',
  'assert.doesNotThrow(() => process.emit("unhandledRejection", null));',
  'assert.equal(throwingCalls, 1, "cleared reporter must not be called");',
  'assert.equal(originalCalls.length, 1);',
  'assert.equal(replacementCalls.length, 1);',
  'assert.equal(logs.length, 4);',
  'assert.deepEqual(logs.map((args) => args[0]), [',
  '  "[ldb-events] uncaughtException (kept alive):",',
  '  "[ldb-events] unhandledRejection (kept alive):",',
  '  "[ldb-events] uncaughtException (kept alive):",',
  '  "[ldb-events] unhandledRejection (kept alive):",',
  ']);',
  'assert.equal(logs[0][1], original);',
  'assert.deepEqual(logs.map((args) => args[1].message), ["original-error", "17", "reporter-contained", "null"]);',
  'console.error = originalConsoleError;',
  'setImmediate(() => process.stdout.write("LDB_EVENT_ADAPTER_OK\\n"));',
].join("\n");

test("ldb Node event adapter preserves normalization reporter replacement and clearing", { timeout: 30_000 }, async (t) => {
  const fixture = await createFixture();
  let child: OwnedChild | undefined;
  let passed = false;
  try {
    child = OwnedChild.spawn(["-e", EVENT_ADAPTER_PROBE], fixture.env);
    await child.waitForClose(STARTUP_TIMEOUT_MS);
    child.assertExit(0);
    assert.equal(child.stdout.toString("utf8"), "LDB_EVENT_ADAPTER_OK\n");
    assert.equal(child.stderr.toString("utf8"), "");
    assert.equal(await readDiscovery(fixture.discoveryPath), undefined);
    assert.equal(fixture.gateway.requestStreams, 0);
    assert.equal(fixture.gateway.responsePosts, 0);
    assert.deepEqual(fixture.gateway.errors, []);
    passed = true;
  } finally { await finishFixture(t, fixture, child, passed); }
});
