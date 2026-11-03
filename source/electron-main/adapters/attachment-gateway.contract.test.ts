import assert from "node:assert/strict";
import { mkdtemp, mkdir, open, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test, { type TestContext } from "node:test";
import { pathToFileURL } from "node:url";

import {
  createProductionAttachmentGatewayBinding,
  type AttachmentGatewayFileSystemPorts,
  type ElectronAttachmentGatewayCompositionPorts,
} from "./attachment-gateway.js";
import {
  GATEWAY_READ_CHUNK_BYTES,
  type AttachmentDownloadFile,
  type createAttachmentEdgePort,
} from "../attachments/attachments.js";
import { createEgressConnectionObserver } from "../box/remote-connector-egress.js";
import type { CoordinatorAccountRuntime } from "../coordinator/coordinator-account-runtime.js";
import {
  createCoordinatorMainLegs,
  type CoordinatorMainPortOwnership,
} from "../coordinator/coordinator-main-legs.js";
import type {
  CoordinatorChildProcess,
  CoordinatorMessageChannel,
  CoordinatorMessagePort,
} from "../coordinator/coordinator-launcher.js";
import {
  createProductionCoordinatorAdapter,
  type ProductionCoordinatorAuthStatus,
} from "../coordinator/production-provider.js";
import type {
  ProductionAccountService,
  ProductionAccountStatus,
  ProductionCoordinatorService,
  ProductionServiceContext,
} from "../main-production-services.js";
import type { BoxConnectionInfo } from "../../shared/node/egress-tunnel/box-connection.js";
import {
  COORDINATOR_PROTOCOL_VERSION,
  type CoordinatorFrame,
} from "../../shared/rpc/coordinator-port.js";

// MAB-v1 composes the real provider, account runtime, launcher, MainData owner,
// client and attachment edge. Only the external Electron/connector/account ABI
// is controlled. File seams below always perform the real Node filesystem I/O.
// A kill request never manufactures process exit. Every retirement explicitly
// releases the child exit event, and every delay is a causal promise barrier.
const SERIAL = { concurrency: false, timeout: 20_000 } as const;
const PNG = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a5r8AAAAASUVORK5CYII=", "base64");
type Request = Extract<CoordinatorFrame, { kind: "request" }>;
type Edge = ReturnType<typeof createAttachmentEdgePort>;

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

function barrier() {
  const entered = deferred<void>();
  const released = deferred<void>();
  return {
    entered: entered.promise,
    release: () => released.resolve(),
    async hold() { entered.resolve(); await released.promise; },
  };
}

/** Native MessagePort ABI, with native-style delivery queued until start(). */
class NativePort implements CoordinatorMessagePort {
  peer!: NativePort;
  closed = false;
  failPost = false;
  readonly sent: unknown[] = [];
  private started = false;
  private readonly queue: unknown[] = [];
  private readonly messages: Array<(event: { readonly data: unknown }) => void> = [];
  private readonly closes: Array<() => void> = [];

  postMessage(value: unknown): void {
    if (this.failPost) throw new Error("fixture native post failed");
    if (this.closed) return;
    const copy: unknown = structuredClone(value);
    this.sent.push(copy);
    this.peer.receive(copy);
  }
  on(event: "message", listener: (event: { readonly data: unknown }) => void): void;
  on(event: "close", listener: () => void): void;
  on(event: "message" | "close", listener: ((event: { readonly data: unknown }) => void) | (() => void)): void {
    if (event === "message") this.messages.push(listener as (event: { readonly data: unknown }) => void);
    else this.closes.push(listener as () => void);
  }
  start(): void {
    this.started = true;
    while (!this.closed && this.queue.length > 0) this.deliver(this.queue.shift());
  }
  close(): void {
    if (this.closed) return;
    this.closed = true;
    for (const listener of this.closes) listener();
    this.peer.close();
  }
  private receive(value: unknown): void {
    if (this.closed) return;
    if (this.started) this.deliver(value);
    else this.queue.push(value);
  }
  private deliver(data: unknown): void {
    for (const listener of this.messages) listener({ data });
  }
}

class NativeChannel implements CoordinatorMessageChannel {
  readonly port1 = new NativePort();
  readonly port2 = new NativePort();
  constructor() { this.port1.peer = this.port2; this.port2.peer = this.port1; }
}

/** Controlled remote endpoint; production owns all protocol/lifetime policy. */
class MainPeer {
  readonly requests: Request[] = [];
  readonly hello = deferred<void>();
  autoCleanup = true;
  onRequest: ((request: Request) => void) | undefined;
  private readonly waiters: Array<{ method: string; index: number; resolve: (request: Request) => void }> = [];
  constructor(readonly port: NativePort, autoReady = true) {
    port.on("message", ({ data }) => {
      const frame = data as CoordinatorFrame;
      if (frame.kind === "lifecycle" && frame.phase === "hello") {
        this.hello.resolve();
        if (autoReady) this.ready();
      } else if (frame.kind === "request") {
        this.requests.push(frame);
        for (const waiter of [...this.waiters]) {
          const request = this.matching(waiter.method)[waiter.index];
          if (request !== undefined) {
            this.waiters.splice(this.waiters.indexOf(waiter), 1);
            waiter.resolve(request);
          }
        }
        if (frame.method === "setHostSettings" && this.autoCleanup) this.reply(frame, null);
        else this.onRequest?.(frame);
      }
    });
    port.start();
  }
  ready(): void { this.port.postMessage({ kind: "lifecycle", phase: "ready", protocolVersion: COORDINATOR_PROTOCOL_VERSION }); }
  matching(method: string): Request[] { return this.requests.filter((request) => request.method === method); }
  request(method: string, index = 0): Promise<Request> {
    const request = this.matching(method)[index];
    if (request !== undefined) return Promise.resolve(request);
    return new Promise((resolve) => this.waiters.push({ method, index, resolve }));
  }
  reply(request: Request, value: unknown): void {
    this.port.postMessage({ kind: "reply", requestId: request.requestId, outcome: { status: "ok", value } });
  }
  serveBytes(bytes: Buffer): void {
    this.onRequest = (request) => {
      assert.equal(request.method, "readAttachmentChunk");
      const { offset, length } = chunkArgs(request);
      this.reply(request, { totalSize: bytes.length, bytesBase64: bytes.subarray(offset, offset + length).toString("base64") });
    };
  }
}

class NativeChild implements CoordinatorChildProcess {
  readonly killed = deferred<void>();
  readonly bootstrapped = deferred<void>();
  killCount = 0;
  exited = false;
  slot: string | undefined;
  main!: MainPeer;
  private readonly exitListeners: Array<(code: number | null) => void> = [];
  constructor(private readonly autoReady: boolean, private readonly failMainHello: boolean) {}
  postMessage(value: unknown, ports: CoordinatorMessagePort[]): void {
    assert.equal(ports.length, 3, "shipping launcher transfers control, renderer and MainData ports");
    const bootstrap = value as { bootstrap: { processConfig: { localHumanId: string } } };
    this.slot = bootstrap.bootstrap.processConfig.localHumanId;
    const control = ports[0] as NativePort;
    control.start();
    control.postMessage({ kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
    const main = ports[2] as NativePort;
    main.peer.failPost = this.failMainHello;
    this.main = new MainPeer(main, this.autoReady);
    this.bootstrapped.resolve();
  }
  on(_event: "exit", listener: (code: number | null) => void): void { this.exitListeners.push(listener); }
  kill(): void { this.killCount += 1; this.killed.resolve(); }
  exit(code = 0): void {
    if (this.exited) return;
    this.exited = true;
    for (const listener of this.exitListeners) listener(code);
  }
}

function auth(slot: string): ProductionCoordinatorAuthStatus { return { kind: "logged-in", authId: slot, email: `${slot}@fixture.invalid` }; }
function chunkArgs(request: Request): { path: string; offset: number; length: number } {
  assert.equal(request.method, "readAttachmentChunk");
  return request.args as { path: string; offset: number; length: number };
}
function unexpected(): never { throw new Error("unexpected external native operation"); }

interface FixtureOptions {
  readonly autoReady?: boolean;
  readonly failMainHello?: boolean;
  readonly fileSystem?: AttachmentGatewayFileSystemPorts;
  readonly releases?: ReadonlyArray<() => void>;
  readonly onLaunched?: (fixture: Fixture) => void;
  readonly beforeStart?: (fixture: Fixture) => void;
}

interface Fixture {
  readonly root: string;
  readonly children: NativeChild[];
  readonly legs: ReturnType<typeof createCoordinatorMainLegs>;
  readonly service: ProductionCoordinatorService;
  readonly runtime: CoordinatorAccountRuntime<ProductionCoordinatorAuthStatus>;
  readonly edge: Edge;
  readonly problems: string[];
  readonly edgeFailures: Array<{ leg: string; errorClass: string }>;
  readonly connectorCalls: string[];
  readonly cleanupStarted: ReturnType<typeof deferred<void>>;
  readonly launches: ReturnType<typeof deferred<NativeChild>>[];
  saveDialog: () => Promise<{ canceled: boolean; filePath?: string }>;
  imageContinuation: (() => void) | undefined;
  start(slot?: string): Promise<NativeChild>;
  observe(slot: string | null): void;
  finishSwitch(previous: NativeChild, slot?: string): Promise<NativeChild>;
  child(index: number): Promise<NativeChild>;
  requestRenderer(sink: (port: CoordinatorMessagePort) => void): void;
  disposeService(): Promise<void>;
}

async function fixture(t: TestContext, options: FixtureOptions = {}): Promise<Fixture> {
  const root = await mkdtemp(join(tmpdir(), "mab-attachment-"));
  const previousRoot = process.env.SAND_DATA_ROOT;
  process.env.SAND_DATA_ROOT = join(root, "sand-data");
  let teardown = async (): Promise<void> => {};
  t.after(async () => {
    for (const release of options.releases ?? []) release();
    try { await teardown(); }
    finally {
      if (previousRoot === undefined) delete process.env.SAND_DATA_ROOT;
      else process.env.SAND_DATA_ROOT = previousRoot;
      await rm(root, { recursive: true, force: true });
    }
  });
  const artifactPath = join(root, "dist", "node-agent-coordinator", "main.cjs");
  await mkdir(join(root, "dist", "node-agent-coordinator"), { recursive: true });
  // This inert file satisfies the production stat check. No subprocess runs it.
  await writeFile(artifactPath, "// Native utilityProcess is controlled by the contract fixture.\n");
  await mkdir(join(root, "downloads"));
  await mkdir(join(root, "user-data"));
  const children: NativeChild[] = [];
  const launches: ReturnType<typeof deferred<NativeChild>>[] = [];
  const problems: string[] = [];
  const edgeFailures: Array<{ leg: string; errorClass: string }> = [];
  const connectorCalls: string[] = [];
  const cleanupStarted = deferred<void>();
  const legs = createCoordinatorMainLegs({ onProblem: (detail) => problems.push(detail) });
  let status = { kind: "logged-out" } as ProductionCoordinatorAuthStatus;
  let service!: ProductionCoordinatorService;
  let ownedDispose: Promise<void> | undefined;
  let f!: Fixture;
  const account = {
    getStatus: async () => status as ProductionAccountStatus,
    subscribe: () => () => {},
  } as Pick<ProductionAccountService, "getStatus" | "subscribe"> as ProductionAccountService;
  // These are the actual fields consumed by this composition. Unrelated root
  // services are intentionally absent and cannot provide a fallback owner.
  const context = {
    native: { app: { isPackaged: false } },
    resources: { metadata: { version: "mab-contract" } },
    coordinatorLegs: legs,
    connectorEgress: createEgressConnectionObserver<BoxConnectionInfo>(),
    requireAccount: () => account,
    requireCoordinator: () => service,
    requireMcp: unexpected,
    getMainWindow: () => null,
    readTelemetry: () => ({ telemetry: { reportAttachmentEdgeFailure: (failure: { leg: string; errorClass: string }) => edgeFailures.push(failure) } }),
  } as unknown as ProductionServiceContext;
  const adapter = createProductionCoordinatorAdapter({
    electronMainModuleUrl: pathToFileURL(join(root, "dist", "electron-main", "main.cjs")).href,
    utilityProcess: {
      fork(path, args, options_) {
        assert.equal(path, artifactPath);
        assert.deepEqual(args, []);
        assert.equal(options_.serviceName, "sand-node-agent-coordinator");
        const child = new NativeChild(options.autoReady ?? true, options.failMainHello ?? false);
        const index = children.push(child) - 1;
        launches[index]?.resolve(child);
        return child;
      },
    },
    MessageChannelMain: NativeChannel,
    net: { isOnline: () => false },
    powerMonitor: { on() {} },
    createWebAuthnPromptWindow: unexpected,
    localExecNative: {
      spawnLocalExecDaemon: unexpected,
      terminateProcess: unexpected,
      isProcessAlive: unexpected,
      readProcessIdentity: unexpected,
    },
    createGatewayConnector: () => ({
      connect() { connectorCalls.push("connect"); return unexpected(); },
      issueLocalExecDaemonCredential() { connectorCalls.push("credential"); return unexpected(); },
    }),
    getDataDir: () => join(root, "sand-data"),
    account: {
      authorizeAccount: async () => true,
      revokeRefusedAccount: async () => ({ kind: "completed", status: { kind: "logged-out" } }),
      async prepareAccountTransition() {
        // Ordinary Host cleanup deliberately uses the real shared MainData
        // method without attachment's retired-lifetime guard.
        cleanupStarted.resolve();
        await legs.legs.setHostSettings!({ mcpCustomInstructionsByServerId: {}, mcpDisabledToolsByServerId: {} });
      },
      resetAccountState() {},
      deliverStatus(next) { status = next; },
    },
    resync: {
      getMcpCustomInstructionsAccountScope: () => null,
      getMcpCustomInstructionsByServerId: () => ({}),
      getMcpDisabledToolsByServerId: () => ({}),
      setMcpCustomInstructionsByServerId() {},
      setMcpDisabledToolsByServerId() {},
      detectTimeZone: () => "UTC",
      getUserTimeZoneOverride: () => null,
      getComputerUseModel: () => null,
      getAutoReviewInstructions: () => null,
      getLocalToolPermission: () => null,
      getWebauthnProxyEnabled: () => false,
      getFeatureFlagOverrides: () => ({}),
      pushBoxSecrets: async () => {},
      onHostSettingsTransportConnected() {},
      onHostSettingsTransportDown() {},
    },
    events: {
      onAgentsEvent() {},
      onAgentsRosterSeed() {},
      onCoordinatorLaunched() { options.onLaunched?.(f); },
    },
    telemetry: {
      getUploader: () => null,
      isQuitting: () => false,
      isRebuildInFlight: () => false,
      reportHandoff() {},
      recordSendStage() {},
      recordGatewayCommandSpan() {},
      reportProcessCrash() {},
      getRpcTraceWindowTraceparent: () => undefined,
      flushNow() {},
    },
    reportProblem: (area, detail) => problems.push(`${area}: ${detail}`),
    reportFailure: (area, leg) => problems.push(`${area}: ${leg}`),
  });
  service = adapter.create(context);
  const runtime = service.getAccountRuntime!() as CoordinatorAccountRuntime<ProductionCoordinatorAuthStatus>;
  const electron: ElectronAttachmentGatewayCompositionPorts = {
    app: { getPath: (name) => join(root, name === "userData" ? "user-data" : "downloads") },
    BrowserWindow: class { constructor(_options: { readonly show: false }) {} },
    dialog: {
      showSaveDialog: async () => await f.saveDialog(),
      showMessageBox: async () => undefined,
    },
    nativeImage: {
      createFromBuffer: (bytes) => ({ isEmpty: () => bytes.length === 0, getSize: () => ({ width: 1, height: 1 }) }),
      createFromDataURL() {
        f.imageContinuation?.();
        return { isEmpty: () => false, resize: () => ({ isEmpty: () => false, toJPEG: () => PNG, toPNG: () => PNG }) };
      },
    },
  };
  const edge = createProductionAttachmentGatewayBinding(electron, options.fileSystem).create(context) as Edge;
  f = {
    root, children, launches, legs, service, runtime, edge, problems, edgeFailures, connectorCalls, cleanupStarted,
    saveDialog: async () => ({ canceled: true }),
    imageContinuation: undefined,
    async start(slot = "account-A") {
      const index = children.length;
      status = auth(slot);
      await service.start(status as ProductionAccountStatus);
      return await f.child(index);
    },
    observe(slot) {
      status = slot == null ? { kind: "logged-out" } : auth(slot);
      runtime.observe(status);
    },
    async finishSwitch(previous, slot) {
      if (slot !== undefined) f.observe(slot);
      await previous.killed.promise;
      assert.equal(previous.exited, false, "kill is not process exit");
      const index = children.length;
      previous.exit();
      await runtime.whenIdle();
      return await f.child(index);
    },
    async child(index) {
      let child = children[index];
      if (child === undefined) {
        const pending = launches[index] ??= deferred<NativeChild>();
        child = await pending.promise;
      }
      await child.bootstrapped.promise;
      return child;
    },
    requestRenderer: (sink) => adapter.requestRendererPort(sink),
    disposeService() { return ownedDispose ??= Promise.resolve(service.dispose()); },
  };
  teardown = async () => {
    const disposing = f.disposeService();
    // The fixture owns every native endpoint. Settle outstanding ordinary
    // cleanup before waiting, then deliver actual exit for every owned child.
    legs.dispose();
    for (const child of children) child.exit();
    await disposing;
  };
  options.beforeStart?.(f);
  return f;
}

function assertNoAttachmentRequests(peer: MainPeer): void {
  assert.deepEqual(peer.requests.filter((request) => request.method !== "setHostSettings"), []);
}

function chunkReply(peer: MainPeer, request: Request, bytes: Buffer): void {
  const { offset, length } = chunkArgs(request);
  peer.reply(request, { totalSize: bytes.length, bytesBase64: bytes.subarray(offset, offset + length).toString("base64") });
}

async function assertMissing(path: string): Promise<void> {
  await assert.rejects(stat(path), { code: "ENOENT" });
}

test("MAB attachment multichunk read returns actual bytes through one production session", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const bytes = Buffer.concat([Buffer.alloc(GATEWAY_READ_CHUNK_BYTES, 0x41), Buffer.from("final native chunk")]);
  a.main.serveBytes(bytes);
  const result = await f.edge.readBytes("/remote/report.pdf");
  assert.equal(result?.kind, "bytes");
  if (result?.kind !== "bytes") assert.fail("expected byte preview");
  assert.deepEqual(Buffer.from(result.bytes), bytes);
  assert.deepEqual(a.main.requests.map(chunkArgs), [
    { path: "/remote/report.pdf", offset: 0, length: 0 },
    { path: "/remote/report.pdf", offset: 0, length: GATEWAY_READ_CHUNK_BYTES },
    { path: "/remote/report.pdf", offset: GATEWAY_READ_CHUNK_BYTES, length: GATEWAY_READ_CHUNK_BYTES },
  ]);
  assert.equal(a.slot, "account-A");
  assert.equal(f.children.length, 1);
  assert.deepEqual(f.edgeFailures, []);
});

test("MAB attachment same account refresh preserves a multichunk read lifetime", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const bytes = Buffer.concat([Buffer.alloc(GATEWAY_READ_CHUNK_BYTES, 0x61), Buffer.from("same-account-tail")]);
  const session = f.service.captureMainDataSession();
  assert.ok(session);
  const reading = f.edge.readBytes("/remote/refresh.pdf");
  chunkReply(a.main, await a.main.request("readAttachmentChunk"), bytes);
  const first = await a.main.request("readAttachmentChunk", 1);
  f.observe("account-A");
  await f.runtime.whenIdle();
  session.assertCurrent();
  chunkReply(a.main, first, bytes);
  chunkReply(a.main, await a.main.request("readAttachmentChunk", 2), bytes);
  const result = await reading;
  assert.equal(result?.kind, "bytes");
  if (result?.kind !== "bytes") assert.fail("expected byte preview after same-account refresh");
  assert.deepEqual(Buffer.from(result.bytes), bytes);
  assert.equal(f.children.length, 1);
  assert.equal(a.killCount, 0);
});

async function rejectReadAtBoundary(t: TestContext, boundary: "probe" | "first" | "last" | "zero"): Promise<void> {
  const f = await fixture(t);
  const a = await f.start();
  const bytes = boundary === "zero" ? Buffer.alloc(0) : Buffer.concat([Buffer.alloc(GATEWAY_READ_CHUNK_BYTES, 0x41), Buffer.from("account-A-only")]);
  const reading = f.edge.readBytes("/remote/private.pdf");
  let held = await a.main.request("readAttachmentChunk");
  let count = 1;
  if (boundary === "first" || boundary === "last") {
    chunkReply(a.main, held, bytes);
    held = await a.main.request("readAttachmentChunk", count++);
  }
  if (boundary === "last") {
    chunkReply(a.main, held, bytes);
    held = await a.main.request("readAttachmentChunk", count++);
  }
  // Resolve the real MainData reply, then retire A before any Promise
  // continuation can expose its value or send the next chunk.
  chunkReply(a.main, held, bytes);
  f.observe("account-B");
  assert.equal(await reading, null);
  const b = await f.finishSwitch(a);
  assert.equal(b.slot, "account-B");
  assert.equal(a.main.matching("readAttachmentChunk").length, count);
  assertNoAttachmentRequests(b.main);
  assert.ok(f.edgeFailures.some((failure) => failure.leg === "read-bytes"));
}

test("MAB attachment switch at probe rejects before the first data chunk", SERIAL, async (t) => {
  await rejectReadAtBoundary(t, "probe");
});

test("MAB attachment switch at first chunk cannot append successor bytes", SERIAL, async (t) => {
  await rejectReadAtBoundary(t, "first");
});

test("MAB attachment switch at last chunk cannot return a stale completed preview", SERIAL, async (t) => {
  await rejectReadAtBoundary(t, "last");
});

test("MAB attachment zero byte probe still checks its captured lifetime", SERIAL, async (t) => {
  await rejectReadAtBoundary(t, "zero");
});

test("MAB attachment zero byte positive returns an empty preview without a data request", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  a.main.serveBytes(Buffer.alloc(0));
  const result = await f.edge.readBytes("/remote/empty.pdf");
  assert.equal(result?.kind, "bytes");
  if (result?.kind !== "bytes") assert.fail("expected empty byte preview");
  assert.equal(result.bytes.byteLength, 0);
  assert.equal(a.main.requests.length, 1);
  assert.equal(chunkArgs(a.main.requests[0]!).length, 0);
});

test("MAB attachment queued before ready is rejected while ordinary Host cleanup flushes", SERIAL, async (t) => {
  const f = await fixture(t, { autoReady: false });
  const a = await f.start();
  a.main.autoCleanup = false;
  const reading = f.edge.readText("/remote/queued.txt");
  f.observe("account-B");
  // Wait until the real account owner has invoked its ordinary Host cleanup.
  // Both that call and attachment's call are queued on the same real client.
  await f.cleanupStarted.promise;
  assert.equal(a.main.requests.length, 0);
  a.main.ready();
  const departure = await a.main.request("setHostSettings");
  assert.equal(await reading, null);
  assert.equal(a.main.matching("readAttachmentText").length, 0);
  assert.equal(a.main.port.closed, false);
  assert.equal(a.killCount, 0);
  a.main.reply(departure, null);
  const b = await f.finishSwitch(a);
  b.main.ready();
  assertNoAttachmentRequests(b.main);
  assert.deepEqual(f.problems, []);
});

test("MAB attachment late text reply after native retirement is rejected without replay", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const reading = f.edge.readText("/remote/private.txt");
  const request = await a.main.request("readAttachmentText");
  f.observe("account-B");
  a.main.reply(request, "account-A private text");
  assert.equal(await reading, null);
  const b = await f.finishSwitch(a);
  assertNoAttachmentRequests(b.main);
  assert.equal(a.main.matching("readAttachmentText").length, 1);
});

test("MAB attachment text reply continuation cannot escape a just retired session", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const reading = f.edge.readText("/remote/continuation.txt");
  a.main.reply(await a.main.request("readAttachmentText"), "already resolved A reply");
  f.observe("account-B");
  assert.equal(await reading, null);
  assertNoAttachmentRequests((await f.finishSwitch(a)).main);
});

test("MAB attachment image fallback reply continuation rejects stale remote pixels", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const reading = f.edge.resolveMedia(join(f.root, "missing-remote.png"));
  const request = await a.main.request("readAttachmentImage");
  a.main.reply(request, { dataUrl: `data:image/png;base64,${PNG.toString("base64")}`, width: 1, height: 1 });
  f.observe("account-B");
  assert.equal(await reading, null);
  assertNoAttachmentRequests((await f.finishSwitch(a)).main);
});

test("MAB attachment link reply continuation rejects metadata from a retired session", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const reading = f.edge.getLinkMetadata("https://fixture.invalid/private");
  const rejected = assert.rejects(reading, /session.*current/);
  const request = await a.main.request("fetchLinkMetadata");
  assert.deepEqual(request.args, { cacheDir: join(f.root, "user-data", "link-preview-cache"), url: "https://fixture.invalid/private" });
  a.main.reply(request, { title: "A-only metadata", imageDataUrl: null, faviconDataUrl: null });
  f.observe("account-B");
  await rejected;
  assertNoAttachmentRequests((await f.finishSwitch(a)).main);
});

test("MAB attachment link image continuation rechecks lifetime after native image work", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  // The real encoded-image bounds reader uses this PNG's IHDR dimensions;
  // Electron's native image decoding/resizing is the controlled native ABI.
  const widePng = Buffer.from(PNG);
  widePng.writeUInt32BE(1600, 16);
  f.imageContinuation = () => f.observe("account-B");
  const reading = f.edge.getLinkMetadata("https://fixture.invalid/wide-image");
  const rejected = assert.rejects(reading, /session.*current/);
  a.main.reply(await a.main.request("fetchLinkMetadata"), {
    title: "A-only image", imageDataUrl: `data:image/png;base64,${widePng.toString("base64")}`, faviconDataUrl: null,
  });
  await rejected;
  assertNoAttachmentRequests((await f.finishSwitch(a)).main);
});

test("MAB attachment text image and link positive controls use the production session", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const text = f.edge.readText("/remote/positive.txt");
  a.main.reply(await a.main.request("readAttachmentText"), "positive text");
  assert.equal(await text, "positive text");
  const image = f.edge.resolveMedia(join(f.root, "positive-remote.png"));
  const imageValue = { dataUrl: `data:image/png;base64,${PNG.toString("base64")}`, width: 1, height: 1 };
  a.main.reply(await a.main.request("readAttachmentImage"), imageValue);
  assert.deepEqual(await image, { kind: "image", ...imageValue });
  const metadata = f.edge.getLinkMetadata("https://fixture.invalid/positive");
  const metadataValue = { title: "positive metadata", imageDataUrl: imageValue.dataUrl, faviconDataUrl: null };
  a.main.reply(await a.main.request("fetchLinkMetadata"), metadataValue);
  assert.deepEqual(await metadata, metadataValue);
  assert.deepEqual(f.edgeFailures, []);
});

async function replaceSameAccount(t: TestContext, renderer: boolean): Promise<void> {
  const f = await fixture(t);
  const a = await f.start();
  const oldReading = f.edge.readText("/remote/old-generation.txt");
  await a.main.request("readAttachmentText");
  let replacement: Promise<void> | undefined;
  const delivered: CoordinatorMessagePort[] = [];
  if (renderer) {
    f.requestRenderer((port) => delivered.push(port));
    assert.equal(delivered.length, 1);
    f.requestRenderer((port) => delivered.push(port));
  } else replacement = f.runtime.restart();
  await a.killed.promise;
  assert.equal(a.exited, false);
  assert.equal(f.children.length, 1, "same-account replacement must wait for process exit");
  assert.equal(a.main.port.closed, false, "old MainData can remain usable until actual close/replacement");
  const admittedBeforeExit = f.edge.readText("/remote/before-exit.txt");
  a.main.reply(await a.main.request("readAttachmentText", 1), "still the live old port");
  assert.equal(await admittedBeforeExit, "still the live old port");
  a.exit();
  await replacement;
  const next = await f.child(1);
  assert.equal(next.slot, "account-A");
  assert.equal(await oldReading, null);
  assertNoAttachmentRequests(next.main);
  if (renderer) assert.equal(delivered.length, 2);
  const fresh = f.edge.readText("/remote/new-generation.txt");
  next.main.reply(await next.main.request("readAttachmentText"), "new generation only");
  assert.equal(await fresh, "new generation only");
  assert.equal(next.main.matching("readAttachmentText").length, 1);
}

test("MAB attachment same account restart waits for exit and never migrates an old operation", SERIAL, async (t) => {
  await replaceSameAccount(t, false);
});

test("MAB attachment renderer replacement waits for exit and never migrates an old operation", SERIAL, async (t) => {
  await replaceSameAccount(t, true);
});

test("MAB attachment runtime identity join rejects a foreign current port and disposal preserves it", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const captured = f.service.captureMainDataSession();
  assert.ok(captured);
  const foreignChannel = new NativeChannel();
  const foreign = new MainPeer(foreignChannel.port1);
  const ownership = f.legs.adoptPort(foreignChannel.port2);
  assert.ok(ownership);
  ownership.assertCurrent();
  assert.ok(f.runtime.captureCurrentBinding(), "the real account runtime remains active");
  assert.equal(f.service.captureMainDataSession(), null, "an unrelated current port must not join this runtime identity");
  assert.throws(() => captured.assertCurrent(), /session.*current/);
  assert.equal(await f.edge.readText("/remote/must-not-reach-foreign.txt"), null);
  assertNoAttachmentRequests(foreign);
  let disposed = false;
  const disposing = f.disposeService().then(() => { disposed = true; });
  await a.killed.promise;
  assert.equal(a.exited, false);
  assert.equal(disposed, false);
  ownership.assertCurrent();
  assert.equal(foreign.port.closed, false, "shipping disposal only withdraws its owned adoption");
  const ordinary = f.legs.legs.setHostSettings!({ fixtureForeignOwner: true });
  await ordinary;
  assert.equal(foreign.matching("setHostSettings").length, 1);
  a.exit();
  await disposing;
  ownership.assertCurrent();
  assert.equal(foreign.port.closed, false);
});

test("MAB attachment failed production adoption cannot capture or revoke a prior foreign port", SERIAL, async (t) => {
  const foreignChannel = new NativeChannel();
  const foreign = new MainPeer(foreignChannel.port1);
  let ownership: CoordinatorMainPortOwnership | null = null;
  const f = await fixture(t, {
    failMainHello: true,
    beforeStart(next) { ownership = next.legs.adoptPort(foreignChannel.port2); },
  });
  const a = await f.start();
  assert.ok(ownership);
  (ownership as CoordinatorMainPortOwnership).assertCurrent();
  assert.equal(a.main.port.closed, true, "the failed native candidate is closed");
  assert.ok(f.runtime.captureCurrentBinding());
  assert.equal(f.service.captureMainDataSession(), null, "failed adoption leaves no runtime-owned join");
  assert.equal(await f.edge.readText("/remote/failed-adoption.txt"), null);
  assertNoAttachmentRequests(foreign);
  let disposed = false;
  const disposing = f.disposeService().then(() => { disposed = true; });
  await a.killed.promise;
  assert.equal(disposed, false);
  assert.equal(foreign.port.closed, false);
  a.exit();
  await disposing;
  (ownership as CoordinatorMainPortOwnership).assertCurrent();
  assert.equal(foreign.port.closed, false);
});

test("MAB attachment shipping disposal withdraws its own port before actual child exit", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const reading = f.edge.readText("/remote/disposal.txt");
  await a.main.request("readAttachmentText");
  let disposed = false;
  const disposing = f.disposeService().then(() => { disposed = true; });
  assert.equal(f.service.captureMainDataSession(), null);
  assert.equal(a.main.port.closed, true);
  assert.equal(await reading, null);
  await a.killed.promise;
  assert.equal(a.exited, false);
  assert.equal(disposed, false, "disposal must still await the actual process exit");
  a.exit();
  await disposing;
  assert.equal(disposed, true);
  assert.equal(f.children.length, 1);
});

test("MAB attachment download positive writes real multichunk and zero size files", SERIAL, async (t) => {
  let opens = 0;
  let closes = 0;
  const f = await fixture(t, { fileSystem: { async openDownloadFile(path) {
    opens += 1;
    const handle = await open(path, "w");
    return { write: (bytes, offset, length, position) => handle.write(bytes, offset, length, position), async close() { closes += 1; await handle.close(); } };
  } } });
  const a = await f.start();
  const bytes = Buffer.concat([Buffer.alloc(GATEWAY_READ_CHUNK_BYTES, 0x5a), Buffer.from("download tail")]);
  const destination = join(f.root, "downloads", "positive.bin");
  f.saveDialog = async () => ({ canceled: false, filePath: destination });
  a.main.serveBytes(bytes);
  assert.equal(await f.edge.download("/remote/positive.bin", "positive.bin"), true);
  assert.deepEqual(await readFile(destination), bytes);
  assert.deepEqual(a.main.requests.map((request) => chunkArgs(request).offset), [0, 0, GATEWAY_READ_CHUNK_BYTES]);
  const empty = join(f.root, "downloads", "empty.bin");
  f.saveDialog = async () => ({ canceled: false, filePath: empty });
  a.main.serveBytes(Buffer.alloc(0));
  assert.equal(await f.edge.download("/remote/empty.bin", "empty.bin"), true);
  assert.equal((await stat(empty)).size, 0);
  assert.equal(a.main.requests.length, 4, "zero size download only probes");
  assert.equal(opens, 2);
  assert.equal(closes, 2);
  assert.deepEqual(f.edgeFailures, []);
});

async function rejectDownloadDialog(t: TestContext, preexisting: boolean): Promise<void> {
  const dialog = barrier();
  let opens = 0;
  const f = await fixture(t, { releases: [dialog.release], fileSystem: { async openDownloadFile(path) { opens += 1; return await open(path, "w"); } } });
  const a = await f.start();
  const destination = join(f.root, "downloads", preexisting ? "keep.bin" : "never-created.bin");
  const original = Buffer.from("preexisting destination bytes");
  if (preexisting) await writeFile(destination, original);
  f.saveDialog = async () => { await dialog.hold(); return { canceled: false, filePath: destination }; };
  a.main.serveBytes(Buffer.from("A-only download"));
  const downloading = f.edge.download("/remote/private.bin", "private.bin");
  await dialog.entered;
  const b = await f.finishSwitch(a, "account-B");
  dialog.release();
  assert.equal(await downloading, false);
  assert.equal(opens, 0, "stale dialog continuation must fail before opening with w");
  if (preexisting) assert.deepEqual(await readFile(destination), original);
  else await assertMissing(destination);
  assert.equal(a.main.matching("readAttachmentChunk").length, 1);
  assertNoAttachmentRequests(b.main);
}

test("MAB attachment download account switch during dialog preserves existing destination bytes", SERIAL, async (t) => {
  await rejectDownloadDialog(t, true);
});

test("MAB attachment download account switch during dialog does not create a destination", SERIAL, async (t) => {
  await rejectDownloadDialog(t, false);
});

async function rejectDownloadIo(t: TestContext, boundary: "open" | "between-chunks" | "final-write" | "final-close"): Promise<void> {
  const io = barrier();
  let writes = 0;
  let closes = 0;
  let nativeHandle: Awaited<ReturnType<typeof open>> | undefined;
  const bytes = Buffer.concat([Buffer.alloc(GATEWAY_READ_CHUNK_BYTES, 0x44), Buffer.from("final write")]);
  const f = await fixture(t, { releases: [io.release], fileSystem: { async openDownloadFile(path): Promise<AttachmentDownloadFile> {
    const handle = nativeHandle = await open(path, "w");
    if (boundary === "open") await io.hold();
    return {
      async write(buffer, offset, length, position) {
        const result = await handle.write(buffer, offset, length, position);
        writes += 1;
        if ((boundary === "between-chunks" && writes === 1) || (boundary === "final-write" && writes === 2)) await io.hold();
        return result;
      },
      async close() {
        closes += 1;
        await handle.close();
        if (boundary === "final-close") await io.hold();
      },
    };
  } } });
  const a = await f.start();
  a.main.serveBytes(bytes);
  const destination = join(f.root, "downloads", `${boundary}.bin`);
  f.saveDialog = async () => ({ canceled: false, filePath: destination });
  const downloading = f.edge.download("/remote/private.bin", "private.bin");
  await io.entered;
  const expectedBeforeRelease = boundary === "open" ? Buffer.alloc(0) : boundary === "between-chunks" ? bytes.subarray(0, GATEWAY_READ_CHUNK_BYTES) : bytes;
  assert.deepEqual(await readFile(destination), expectedBeforeRelease, "the barrier follows a real filesystem effect");
  const b = await f.finishSwitch(a, "account-B");
  io.release();
  assert.equal(await downloading, false);
  assert.equal(closes, 1, "the opened handle remains reachable for exactly one close");
  assert.ok(nativeHandle);
  await assert.rejects(nativeHandle.stat(), "the real FileHandle must be closed");
  await assertMissing(destination);
  assert.equal(writes, boundary === "open" ? 0 : boundary === "between-chunks" ? 1 : 2);
  assert.equal(a.main.matching("readAttachmentChunk").length, boundary === "open" ? 1 : boundary === "between-chunks" ? 2 : 3);
  assertNoAttachmentRequests(b.main);
}

test("MAB attachment download switch after real open closes and cleans the destination", SERIAL, async (t) => {
  await rejectDownloadIo(t, "open");
});

test("MAB attachment download switch between real chunk writes closes and cleans", SERIAL, async (t) => {
  await rejectDownloadIo(t, "between-chunks");
});

test("MAB attachment download switch after the final real write cannot return success", SERIAL, async (t) => {
  await rejectDownloadIo(t, "final-write");
});

test("MAB attachment download switch during final close cannot return success", SERIAL, async (t) => {
  await rejectDownloadIo(t, "final-close");
});

test("MAB attachment staged read switch sends no upload and preserves the real original", SERIAL, async (t) => {
  const stagedRead = barrier();
  const reads: string[] = [];
  const f = await fixture(t, { releases: [stagedRead.release], fileSystem: { async readStagedFile(path) {
    const bytes = await readFile(path);
    reads.push(path);
    await stagedRead.hold();
    return bytes;
  } } });
  const a = await f.start();
  const bytes = Buffer.from("actual staged private bytes");
  const staged = await f.edge.stageBytes("private.txt", bytes);
  assert.equal(staged.ok, true);
  if (!staged.ok) assert.fail("real staging failed");
  const committing = f.edge.commitStaged([staged.path], ["private.txt"]);
  await stagedRead.entered;
  assert.deepEqual(reads, [staged.path]);
  assert.deepEqual(await readFile(staged.path), bytes);
  const b = await f.finishSwitch(a, "account-B");
  stagedRead.release();
  assert.equal(await committing, null);
  assert.equal(a.main.matching("uploadAttachment").length, 0);
  assertNoAttachmentRequests(b.main);
  assert.deepEqual(await readFile(staged.path), bytes, "failed commit does not discard its local original");
});

test("MAB attachment staged positive uploads exact real bytes and retains originals until discard", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const files = [
    { filename: "first.txt", bytes: Buffer.from("first actual payload\n") },
    { filename: "second.bin", bytes: Buffer.from([0, 1, 2, 127, 128, 254, 255]) },
  ];
  const paths: string[] = [];
  for (const file of files) {
    const staged = await f.edge.stageBytes(file.filename, file.bytes);
    assert.equal(staged.ok, true);
    if (!staged.ok) assert.fail("real staging failed");
    paths.push(staged.path);
    assert.deepEqual(await readFile(staged.path), file.bytes);
  }
  a.main.onRequest = (request) => {
    assert.equal(request.method, "uploadAttachment");
    const args = request.args as { filename: string; bytesBase64: string };
    a.main.reply(request, { path: `/remote/committed/${args.filename}` });
  };
  assert.deepEqual(await f.edge.commitStaged(paths, files.map((file) => file.filename)), files.map((file) => `/remote/committed/${file.filename}`));
  assert.deepEqual(a.main.matching("uploadAttachment").map((request) => request.args), files.map((file) => ({ filename: file.filename, bytesBase64: file.bytes.toString("base64") })));
  for (let index = 0; index < paths.length; index += 1) {
    assert.deepEqual(await readFile(paths[index]!), files[index]!.bytes);
    await f.edge.discardStaged(paths[index]);
    await assertMissing(paths[index]!);
  }
  assert.deepEqual(f.edgeFailures, []);
});

test("MAB attachment upload reply continuation rejects stale paths and keeps all staged originals", SERIAL, async (t) => {
  const f = await fixture(t);
  const a = await f.start();
  const first = await f.edge.stageBytes("first.txt", Buffer.from("first staged bytes"));
  const second = await f.edge.stageBytes("second.txt", Buffer.from("second staged bytes"));
  if (!first.ok || !second.ok) assert.fail("real staging failed");
  const committing = f.edge.commitStaged([first.path, second.path], ["first.txt", "second.txt"]);
  a.main.reply(await a.main.request("uploadAttachment"), { path: "/remote/A-already-admitted-upload" });
  f.observe("account-B");
  assert.equal(await committing, null);
  const b = await f.finishSwitch(a);
  assert.equal(a.main.matching("uploadAttachment").length, 1, "the second file must not be uploaded after retirement");
  assertNoAttachmentRequests(b.main);
  assert.deepEqual(await readFile(first.path), Buffer.from("first staged bytes"));
  assert.deepEqual(await readFile(second.path), Buffer.from("second staged bytes"));
  // The first upload was already admitted. This oracle makes no rollback claim.
});

test("MAB attachment offline local image staging and discard need no account or network", SERIAL, async (t) => {
  const f = await fixture(t);
  assert.equal(f.runtime.captureCurrentBinding(), null);
  assert.equal(f.service.captureMainDataSession(), null);
  const local = join(f.root, "local.png");
  await writeFile(local, PNG);
  assert.deepEqual(await f.edge.resolveMedia(local), { kind: "image", dataUrl: `data:image/png;base64,${PNG.toString("base64")}`, width: 1, height: 1 });
  const staged = await f.edge.stageBytes("offline.png", PNG);
  assert.equal(staged.ok, true);
  if (!staged.ok) assert.fail("offline real staging failed");
  assert.deepEqual(await readFile(staged.path), PNG);
  await f.edge.discardStaged(staged.path);
  await assertMissing(staged.path);
  assert.equal((await f.edge.resolveMedia("/local/audio.mp3"))?.kind, "audio");
  assert.equal((await f.edge.resolveMedia("/local/video.mp4"))?.kind, "video");
  assert.equal(f.children.length, 0);
  assert.deepEqual(f.connectorCalls, []);
  assert.deepEqual(f.edgeFailures, []);
});

test("MAB attachment failed local read cannot capture an account that appeared during I O", SERIAL, async (t) => {
  const localRead = barrier();
  const f = await fixture(t, { releases: [localRead.release], fileSystem: { async readImageFile(path) {
    try { return await readFile(path); }
    catch (error) { await localRead.hold(); throw error; }
  } } });
  const resolving = f.edge.resolveMedia(join(f.root, "missing-before-account.png"));
  await localRead.entered;
  const a = await f.start();
  assert.ok(f.service.captureMainDataSession(), "the later account really is ready");
  localRead.release();
  assert.equal(await resolving, null);
  assertNoAttachmentRequests(a.main);
  assert.deepEqual(f.connectorCalls, []);
});

test("MAB attachment failed local read preserves its original optional fallback identity", SERIAL, async (t) => {
  const localRead = barrier();
  const f = await fixture(t, { releases: [localRead.release], fileSystem: { async readImageFile(path) {
    try { return await readFile(path); }
    catch (error) { await localRead.hold(); throw error; }
  } } });
  const a = await f.start();
  const resolving = f.edge.resolveMedia(join(f.root, "missing-during-switch.png"));
  await localRead.entered;
  const b = await f.finishSwitch(a, "account-B");
  localRead.release();
  assert.equal(await resolving, null);
  assert.equal(a.main.matching("readAttachmentImage").length, 0);
  assertNoAttachmentRequests(b.main);
});

test("MAB attachment production onLaunched same account refresh publishes only after construction", SERIAL, async (t) => {
  let callbacks = 0;
  const f = await fixture(t, { onLaunched(current) {
    callbacks += 1;
    assert.equal(current.service.captureMainDataSession(), null, "constructing runtime is not yet a capturable account binding");
    current.observe("account-A");
  } });
  const a = await f.start();
  await f.runtime.whenIdle();
  assert.equal(callbacks, 1);
  assert.equal(a.killCount, 0);
  assert.ok(f.service.captureMainDataSession());
  const reading = f.edge.readText("/remote/constructed.txt");
  a.main.reply(await a.main.request("readAttachmentText"), "published after same-account callback");
  assert.equal(await reading, "published after same-account callback");
  assert.deepEqual(f.problems, [], "production must not swallow a failed launch callback assertion");
});

test("MAB attachment production onLaunched account switch fences the unpublished child until exit", SERIAL, async (t) => {
  let callbacks = 0;
  const f = await fixture(t, { onLaunched(current) {
    callbacks += 1;
    assert.equal(current.service.captureMainDataSession(), null);
    if (callbacks === 1) current.observe("account-B");
  } });
  const starting = f.service.start(auth("account-A") as ProductionAccountStatus);
  const a = await f.child(0);
  await a.killed.promise;
  assert.equal(a.main.port.closed, true, "unpublished production runtime withdraws its owned MainData");
  assert.equal(a.exited, false);
  assert.equal(f.children.length, 1);
  assert.equal(f.service.captureMainDataSession(), null);
  a.exit();
  await starting;
  await f.runtime.whenIdle();
  const b = await f.child(1);
  assert.equal(b.slot, "account-B");
  assert.equal(callbacks, 2);
  assert.ok(f.service.captureMainDataSession());
  assertNoAttachmentRequests(a.main);
  assertNoAttachmentRequests(b.main);
  assert.deepEqual(f.problems, []);
});

test("MAB attachment production onLaunched disposal withdraws an unpublished own port and waits for exit", SERIAL, async (t) => {
  let disposed = false;
  const f = await fixture(t, { onLaunched(current) {
    assert.equal(current.service.captureMainDataSession(), null);
    void current.disposeService().then(() => { disposed = true; });
  } });
  const starting = f.service.start(auth("account-A") as ProductionAccountStatus);
  const a = await f.child(0);
  await a.killed.promise;
  assert.equal(a.main.port.closed, true);
  assert.equal(a.exited, false);
  assert.equal(disposed, false);
  assert.equal(f.service.captureMainDataSession(), null);
  a.exit();
  await starting;
  await f.disposeService();
  assert.equal(disposed, true);
  assert.equal(f.children.length, 1);
  assert.deepEqual(f.problems, []);
});

test("MAB attachment production onLaunched foreign adoption never becomes the runtime owned session", SERIAL, async (t) => {
  const channel = new NativeChannel();
  const foreign = new MainPeer(channel.port1);
  let foreignOwnership: CoordinatorMainPortOwnership | null = null;
  const f = await fixture(t, { onLaunched(current) {
    assert.equal(current.service.captureMainDataSession(), null);
    foreignOwnership = current.legs.adoptPort(channel.port2);
    assert.ok(foreignOwnership);
    foreignOwnership.assertCurrent();
  } });
  const a = await f.start();
  assert.ok(f.runtime.captureCurrentBinding());
  assert.equal(f.service.captureMainDataSession(), null);
  assert.equal(await f.edge.readText("/remote/reentrant-foreign.txt"), null);
  assertNoAttachmentRequests(foreign);
  const disposing = f.disposeService();
  await a.killed.promise;
  assert.equal(foreign.port.closed, false);
  assert.equal(a.exited, false);
  a.exit();
  await disposing;
  assert.ok(foreignOwnership);
  (foreignOwnership as CoordinatorMainPortOwnership).assertCurrent();
  assert.equal(foreign.port.closed, false);
  assert.deepEqual(f.problems, []);
});
