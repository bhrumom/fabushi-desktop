import assert from "node:assert/strict";
import { spawn, type ChildProcess } from "node:child_process";
import { randomUUID } from "node:crypto";
import { EventEmitter } from "node:events";
import { mkdtemp, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";
import test, { type TestContext } from "node:test";

import { createCoordinatorControlServer } from "./coordinator-control-server.js";
import {
  createCoordinatorControlExecutors,
  type CoordinatorControlExecutorDependencies,
} from "./coordinator-executors.js";
import type {
  CoordinatorControlExecutor,
  CoordinatorControlSourceContext,
  CoordinatorFrame,
} from "../../shared/rpc/coordinator-port.js";
import type {
  ExpectedLocalExecProcessIdentity,
  LocalExecProcessIdentity,
} from "../../shared/local-exec-process-identity.js";

// LPI-v1: the producer is the normal exact-H Rust executable. Only the native
// process observation and daemon handle are controlled; no Rust request or
// identity predicate is reproduced here.
const observationTimeoutMs = 12_000;
const childCloseTimeoutMs = 5_000;

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((settle) => { resolve = settle; });
  return { promise, resolve };
}

async function bounded<T>(promise: Promise<T>, label: string, milliseconds: number): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(label + " timed out")), milliseconds);
      }),
    ]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

type NativeObservation = Pick<LocalExecProcessIdentity, "pid" | "startEpochMs" | "command">;
type IdentityCall = {
  readonly requestId: string;
  readonly args: Record<string, unknown>;
  readonly result: LocalExecProcessIdentity | null;
};
type Executors = ReturnType<typeof createCoordinatorControlExecutors>;
type Peer = {
  readonly calls: IdentityCall[];
  readonly requestIds: string[];
  readonly posts: CoordinatorFrame[];
  readonly executors: Executors;
  waitFor(predicate: () => boolean, label: string): Promise<void>;
  assertHealthy(): void;
  stop(): Promise<void>;
  closeInputBeforeIdentityReply(): Promise<void>;
};
type Environment = {
  readonly binary: string;
  readonly root: string;
  readonly identity: LocalExecProcessIdentity;
  readonly publicationTime: number;
  readonly state: {
    observed: NativeObservation | null;
    canonicalEntry: string;
    spawnCount: number;
    terminations: number[];
    handleSignals: NodeJS.Signals[];
  };
  readonly native: NonNullable<CoordinatorControlExecutorDependencies["native"]>;
  publishDiscovery(): Promise<void>;
  readonly peers: Peer[];
};

async function createEnvironment(t: TestContext, mode: "adopt" | "spawn"): Promise<Environment> {
  const binary = process.env.FABUSHI_COORDINATOR_TEST_BIN;
  assert.ok(binary && binary.trim().length > 0, "FABUSHI_COORDINATOR_TEST_BIN must identify the exact-H normal binary");
  assert.ok((await stat(binary)).isFile(), "the required Coordinator binary must be a regular file");
  const root = await mkdtemp(join(tmpdir(), "ocrm-lpi-"));
  const generationToken = randomUUID();
  const identity: LocalExecProcessIdentity = {
    pid: 64_001,
    startEpochMs: Date.now() - 1_000,
    command: process.execPath + " /ocrm-owned/lpi-daemon.cjs --sand-local-exec-generation=" + generationToken,
    entryRealpath: "/ocrm-owned/lpi-daemon.cjs",
    generationToken,
  };
  const publicationTime = identity.startEpochMs + 500;
  const peers: Peer[] = [];
  const state = {
    observed: { pid: identity.pid, startEpochMs: identity.startEpochMs, command: identity.command } as NativeObservation | null,
    canonicalEntry: identity.entryRealpath,
    spawnCount: 0,
    terminations: [] as number[],
    handleSignals: [] as NodeJS.Signals[],
  };
  const daemonHandle = Object.assign(new EventEmitter(), {
    pid: identity.pid,
    exitCode: null as number | null,
    signalCode: null as NodeJS.Signals | null,
  });
  Object.assign(daemonHandle, {
    kill(signal: NodeJS.Signals = "SIGTERM") {
      state.handleSignals.push(signal);
      daemonHandle.signalCode = signal;
      daemonHandle.emit("exit", null, signal);
      return true;
    },
  });
  async function publishDiscovery() {
    await writeFile(join(root, "local-exec-daemon.json"), JSON.stringify({
      pid: identity.pid,
      startedAt: publicationTime,
      entryRealpath: identity.entryRealpath,
      generationToken: identity.generationToken,
      inflightCount: mode === "adopt" ? 2 : 0,
    }) + "\n", { mode: 0o600 });
  }
  const native: NonNullable<CoordinatorControlExecutorDependencies["native"]> = {
    async spawnLocalExecDaemon(args) {
      state.spawnCount += 1;
      assert.equal(mode, "spawn", "a busy adopted daemon must not be replaced");
      assert.equal(state.spawnCount, 1, "readiness must not start a duplicate daemon");
      assert.equal(args.env.ELECTRON_RUN_AS_NODE, "1");
      assert.equal(args.env.SAND_PACKAGED, "0");
      return {
        child: daemonHandle as unknown as ChildProcess,
        entryRealpath: identity.entryRealpath,
        generationToken: identity.generationToken,
      };
    },
    async terminateProcess(pid) {
      state.terminations.push(pid);
      if (state.observed?.pid === pid) {
        state.observed = null;
        daemonHandle.exitCode = 0;
        daemonHandle.emit("exit", 0, null);
      }
    },
    isProcessAlive: (pid) => state.observed?.pid === pid,
    readProcessIdentity: (pid) => state.observed?.pid === pid ? { ...state.observed } : null,
    resolveLocalExecDaemonEntryRealpath: () => state.canonicalEntry,
  };
  const environment = { binary, root, identity, publicationTime, state, native, publishDiscovery, peers };
  // One owner joins every child before removing any state the child can read.
  t.after(async () => {
    const errors: unknown[] = [];
    for (const peer of [...peers].reverse()) {
      try { await peer.stop(); } catch (error) { errors.push(error); }
    }
    await rm(root, { recursive: true, force: true });
    if (errors.length > 0) throw new AggregateError(errors, "LPI owned child teardown failed");
  });
  if (mode === "adopt") await publishDiscovery();
  return environment;
}

function makeExecutors(environment: Environment): Executors {
  return createCoordinatorControlExecutors({
    connector: {
      connect: () => ({ baseUrl: "http://127.0.0.1:1", headers: {} }),
      issueLocalExecDaemonCredential: () => null,
    },
    webauthnPrompt: {
      requestConsent: async () => ({ approved: false as const, promptId: null }),
      requestPin: async () => ({ pin: null }),
      update() {},
      finish() {},
    },
    recordSendStage() {},
    recordGatewayCommandSpan() {},
    onReachability() {},
    onDnsDiagnostic() {},
    onProcessCrash() {},
    native: environment.native,
  });
}

async function startPeer(
  environment: Environment,
  options: { publishAfterFirstIdentity?: boolean; holdFirstIdentityReply?: boolean } = {},
): Promise<Peer> {
  const calls: IdentityCall[] = [];
  const requestIds: string[] = [];
  const posts: CoordinatorFrame[] = [];
  const executors = makeExecutors(environment);
  const gate = deferred<void>();
  if (!options.holdFirstIdentityReply) gate.resolve(undefined);
  let stopping = false;
  let inputClosed = false;
  let forcedCleanup = false;
  let firstFailure: Error | undefined;
  let stderr = "";
  let closed: { code: number | null; signal: NodeJS.Signals | null } | undefined;
  let stopPromise: Promise<void> | undefined;
  const exit = deferred<{ code: number | null; signal: NodeJS.Signals | null }>();
  const fail = (error: unknown) => {
    firstFailure ??= error instanceof Error ? error : new Error(String(error));
  };
  const child = spawn(environment.binary, ["--bootstrap=" + JSON.stringify({
    processConfig: { appVersion: "lpi-contract", isPackaged: false, dataDir: environment.root },
  })], {
    env: { ...process.env, MAHAYANA_APP_HOST_BIN: process.execPath },
    stdio: ["pipe", "pipe", "pipe"],
  });
  const dispatchers: Record<string, CoordinatorControlExecutor> = Object.fromEntries(
    Object.entries(executors).map(([method, executor]) => [
      method, (args: unknown, source: CoordinatorControlSourceContext) => (executor as CoordinatorControlExecutor)(args, source),
    ]),
  );
  dispatchers.getProcessIdentity = async (raw) => {
    const requestId = requestIds[calls.length];
    assert.ok(requestId, "identity execution must come from a recorded real Rust request");
    const result = executors.getProcessIdentity(raw as ExpectedLocalExecProcessIdentity);
    calls.push({
      requestId,
      args: structuredClone(raw) as Record<string, unknown>,
      result: result === null ? null : { ...result },
    });
    if (calls.length === 1 && options.publishAfterFirstIdentity) await environment.publishDiscovery();
    if (calls.length === 1) await gate.promise;
    return result;
  };
  const server = createCoordinatorControlServer({
    executors: dispatchers,
    post(frame) {
      assert.equal(inputClosed, false, "the control server must not publish after its carrier closed");
      posts.push(structuredClone(frame));
      child.stdin.write(JSON.stringify({ channel: "coordinator-control", frame }) + "\n");
    },
    onEvent: {},
    onProblem: fail,
    reportFailure: (_leg, error) => fail(error),
  });
  const lines = createInterface({ input: child.stdout, crlfDelay: Infinity });
  lines.on("line", (line) => {
    try {
      const envelope: unknown = JSON.parse(line);
      assert.ok(envelope && typeof envelope === "object" && !Array.isArray(envelope));
      const carrier = envelope as { channel?: unknown; frame?: unknown };
      assert.equal(carrier.channel, "coordinator-control", "no data-channel peer or Host is part of this contract");
      assert.ok(carrier.frame && typeof carrier.frame === "object");
      const frame = carrier.frame as Record<string, unknown>;
      if (frame.kind === "request" && frame.method === "getProcessIdentity") {
        assert.equal(typeof frame.requestId, "string");
        requestIds.push(frame.requestId as string);
      }
      server.handleMessage(carrier.frame);
    } catch (error) { fail(error); }
  });
  lines.on("error", fail);
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk: string) => { stderr = (stderr + chunk).slice(-16_384); });
  child.stdin.on("error", (error) => { if (!forcedCleanup) fail(error); });
  child.stdout.on("error", fail);
  child.stderr.on("error", fail);
  child.on("error", fail);
  child.once("close", (code, signal) => {
    closed = { code, signal };
    server.handlePortClosed();
    if (!stopping) fail(new Error("Coordinator closed outside its owner: " + String(code) + "/" + String(signal)));
    exit.resolve(closed);
  });
  function assertHealthy() {
    if (firstFailure !== undefined) throw new Error(firstFailure.message + (stderr ? "\nCoordinator stderr:\n" + stderr : ""), { cause: firstFailure });
  }
  async function waitFor(predicate: () => boolean, label: string) {
    const deadline = Date.now() + observationTimeoutMs;
    while (Date.now() < deadline) {
      assertHealthy();
      if (predicate()) return;
      assert.equal(closed, undefined, "Coordinator closed before " + label);
      await new Promise<void>((resolve) => setTimeout(resolve, 10));
    }
    assert.fail(label + " was not observed" + (stderr ? "\nCoordinator stderr:\n" + stderr : ""));
  }
  async function awaitCleanClose() {
    const result = await bounded(exit.promise, "owned Coordinator close", childCloseTimeoutMs);
    assert.deepEqual(result, { code: 0, signal: null }, "control closure must terminate the Coordinator normally");
    assertHealthy();
  }
  function stop(): Promise<void> {
    stopPromise ??= (async () => {
      stopping = true;
      server.dispose();
      gate.resolve(undefined);
      if (!inputClosed && closed === undefined) {
        inputClosed = true;
        child.stdin.end();
      }
      try { await awaitCleanClose(); }
      catch (error) {
        // This child was created above and is the only process this cleanup can signal.
        forcedCleanup = true;
        if (closed === undefined) {
          child.kill("SIGKILL");
          await bounded(exit.promise, "forced owned Coordinator close", childCloseTimeoutMs);
        }
        throw error;
      } finally { lines.close(); }
    })();
    return stopPromise;
  }
  const peer = {
    calls, requestIds, posts, executors, waitFor, assertHealthy, stop,
    async closeInputBeforeIdentityReply() {
      assert.ok(options.holdFirstIdentityReply);
      assert.equal(calls.length, 1);
      stopping = true;
      inputClosed = true;
      child.stdin.end();
      await awaitCleanClose();
      const postsAtClose = posts.length;
      gate.resolve(undefined);
      // The asynchronous executor completion must observe the retired real server.
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
      assert.equal(posts.length, postsAtClose, "late identity results must not publish into a closed control carrier");
      assert.equal(posts.some((frame) => frame.kind === "reply" && frame.requestId === calls[0]!.requestId), false);
      assertHealthy();
    },
  };
  environment.peers.push(peer);
  return peer;
}

function assertAdoptionRequest(environment: Environment, call: IdentityCall) {
  assert.deepEqual(Object.keys(call.args).sort(), ["discoveryStartedAt", "entryRealpath", "generationToken", "pid"]);
  assert.equal(Object.hasOwn(call.args, "startEpochMs"), false);
  assert.equal(Object.hasOwn(call.args, "command"), false);
  assert.deepEqual(call.args, {
    pid: environment.identity.pid,
    entryRealpath: environment.identity.entryRealpath,
    generationToken: environment.identity.generationToken,
    discoveryStartedAt: environment.publicationTime,
  });
  assert.deepEqual(call.result, environment.identity);
}

function assertFullRequest(environment: Environment, call: IdentityCall) {
  assert.deepEqual(call.args, { ...environment.identity, discoveryStartedAt: environment.publicationTime });
  assert.deepEqual(call.result, environment.identity);
}

function assertNoDaemonMutation(environment: Environment, expectedSpawns: number) {
  assert.equal(environment.state.spawnCount, expectedSpawns);
  assert.deepEqual(environment.state.terminations, []);
  assert.deepEqual(environment.state.handleSignals, []);
}

test("LPI actual Coordinator adopts busy discovery and re-adopts after control restart", { timeout: 40_000 }, async (t) => {
  const environment = await createEnvironment(t, "adopt");
  for (let restart = 0; restart < 2; restart += 1) {
    const peer = await startPeer(environment);
    await peer.waitFor(() => peer.calls.length >= 2, "adoption followed by full liveness");
    assertAdoptionRequest(environment, peer.calls[0]!);
    assertFullRequest(environment, peer.calls[1]!);
    assertNoDaemonMutation(environment, 0);
    await peer.stop();
  }
  assertNoDaemonMutation(environment, 0);
});

test("LPI actual Coordinator preserves pre-discovery spawn and full liveness identity", { timeout: 30_000 }, async (t) => {
  const environment = await createEnvironment(t, "spawn");
  const peer = await startPeer(environment, { publishAfterFirstIdentity: true });
  await peer.waitFor(() => peer.calls.length >= 3, "pre-discovery readiness, published discovery and full liveness");
  assert.deepEqual(peer.calls[0]!.args, environment.identity);
  assert.equal(Object.hasOwn(peer.calls[0]!.args, "discoveryStartedAt"), false);
  assert.deepEqual(peer.calls[0]!.result, environment.identity);
  assertFullRequest(environment, peer.calls[1]!);
  assertFullRequest(environment, peer.calls[2]!);
  assertNoDaemonMutation(environment, 1);
  await peer.stop();
  assertNoDaemonMutation(environment, 1);
});

test("LPI actual native identity consumer rejects null types and mismatched identities", { timeout: 30_000 }, async (t) => {
  const environment = await createEnvironment(t, "adopt");
  const peer = await startPeer(environment);
  await peer.waitFor(() => peer.calls.length >= 2, "actual Rust adoption and constrained liveness arguments");
  assertAdoptionRequest(environment, peer.calls[0]!);
  assertFullRequest(environment, peer.calls[1]!);
  await peer.stop();
  const actualArgs = peer.calls[1]!.args;
  const getIdentity = (args: Record<string, unknown>) =>
    peer.executors.getProcessIdentity(args as unknown as ExpectedLocalExecProcessIdentity);
  assert.deepEqual(getIdentity(actualArgs), environment.identity);

  for (const field of ["startEpochMs", "command", "discoveryStartedAt"] as const) {
    const wrongTypes = field === "command"
      ? [null, 0, false, [], {}]
      : [null, "", String(actualArgs[field]), false, [], {}];
    for (const value of wrongTypes) {
      assert.equal(getIdentity({ ...actualArgs, [field]: value }), null, field + " must not treat " + JSON.stringify(value) + " as an absent constraint");
    }
  }
  const mismatches: [string, unknown][] = [
    ["pid", environment.identity.pid + 1],
    ["pid", String(environment.identity.pid)],
    ["entryRealpath", environment.identity.entryRealpath + ".foreign"],
    ["generationToken", environment.identity.generationToken + "-foreign"],
    ["startEpochMs", 0],
    ["startEpochMs", environment.identity.startEpochMs + 1],
    ["command", ""],
    ["command", environment.identity.command + " --foreign"],
    ["discoveryStartedAt", 0],
    ["discoveryStartedAt", environment.identity.startEpochMs - 1],
    ["discoveryStartedAt", environment.identity.startEpochMs + 60_001],
    ["discoveryStartedAt", Date.now() + 60_000],
  ];
  for (const [field, value] of mismatches) {
    assert.equal(getIdentity({ ...actualArgs, [field]: value }), null, "supplied " + field + " must constrain the observed process");
  }

  const original = { ...environment.state.observed! };
  environment.state.observed = { ...original, startEpochMs: original.startEpochMs + 1 };
  assert.equal(getIdentity(actualArgs), null, "a registered PID with a new start time is not the owned process");
  environment.state.observed = { ...original, command: original.command + " --changed" };
  assert.equal(getIdentity(actualArgs), null, "a registered PID with a changed command is not the owned process");
  environment.state.observed = null;
  assert.equal(getIdentity(actualArgs), null, "a lost native identity cannot be reconstructed from registration");
  environment.state.observed = original;
  assert.deepEqual(getIdentity(actualArgs), environment.identity);

  // These use the actual Rust adoption request with fresh native owner maps.
  // They prevent a prior successful registration from masking admission checks.
  const adoptionArgs = peer.calls[0]!.args as unknown as ExpectedLocalExecProcessIdentity;
  environment.state.canonicalEntry += ".foreign";
  assert.equal(makeExecutors(environment).getProcessIdentity(adoptionArgs), null);
  environment.state.canonicalEntry = environment.identity.entryRealpath;
  environment.state.observed = { ...original, command: original.command + "-foreign" };
  const fresh = makeExecutors(environment);
  assert.equal(fresh.getProcessIdentity(adoptionArgs), null, "generation argument prefixes must not confer process ownership");
  environment.state.observed = original;
  assert.deepEqual(fresh.getProcessIdentity(adoptionArgs), environment.identity);
  assertNoDaemonMutation(environment, 0);
});

test("LPI control EOF settles the owned Coordinator without daemon mutation", { timeout: 30_000 }, async (t) => {
  const environment = await createEnvironment(t, "adopt");
  const peer = await startPeer(environment, { holdFirstIdentityReply: true });
  await peer.waitFor(() => peer.calls.length === 1, "actual identity request held before control reply");
  assertAdoptionRequest(environment, peer.calls[0]!);
  await peer.closeInputBeforeIdentityReply();
  assertNoDaemonMutation(environment, 0);
  await peer.stop();
});
