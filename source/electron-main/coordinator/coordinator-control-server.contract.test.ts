import assert from "node:assert/strict";
import { once } from "node:events";
import { access, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";
import { MessageChannel, Worker } from "node:worker_threads";
import { Struct } from "@bufbuild/protobuf";

import { createCoordinatorControlServer } from "./coordinator-control-server.js";
import { McpResult, McpSuccess } from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import {
  createDashboardSandBackendMcpExec,
  type DashboardMcpExecClient,
} from "../../shared/node/cursor-backend/backend-mcp-exec.js";
import {
  generatedMcpResultFactory,
  MCP_ORIGIN_RETIRED,
  McpOriginRetiredError,
} from "../../shared/node/mcp/mcp-result-factory.js";
import type { CoordinatorControlSourceContext, CoordinatorFrame } from "../../shared/rpc/coordinator-port.js";

test("a throwing problem reporter cannot prevent protocol settlement", async () => {
  const server = createCoordinatorControlServer({
    post() {},
    executors: {},
    onEvent: {},
    onProblem() {
      throw new Error("diagnostics failed");
    },
  });

  server.handleMessage({
    kind: "lifecycle",
    phase: "hello",
    protocolVersion: 1,
  });
  server.handleMessage({
    kind: "event",
    family: "undeclared-family",
    payload: {},
  });

  assert.equal((await server.settled).outcome, "protocol-breach");
});

for (const retirement of ["dispose", "port-close", "shutdown", "protocol-breach"] as const) {
  test("OD-01 concrete control source retires before callbacks on " + retirement, async () => {
    const captures: CoordinatorControlSourceContext[] = [];
    const posted: CoordinatorFrame[] = [];
    const callbackStates: string[] = [];
    let retirementCallbacks = 0;
    let server!: ReturnType<typeof createCoordinatorControlServer>;
    const request = { kind: "request", requestId: "capture", method: "capture", args: {
      sourceId: "00000000-0000-0000-0000-000000000000",
      source: { sourceId: "00000000-0000-0000-0000-000000000000" },
    } };
    const duringRetirement = () => {
      retirementCallbacks += 1;
      try { captures[0]!.assertCurrent(); callbackStates.push("live"); }
      catch (error) { callbackStates.push(error instanceof McpOriginRetiredError ? "retired" : "other-error"); }
      server.handleMessage({ ...request, requestId: "reentrant" });
    };
    server = createCoordinatorControlServer({
      post(frame) {
        posted.push(frame);
        if (frame.kind === "lifecycle" && frame.phase === "shutdown") duringRetirement();
      },
      executors: { capture(args, source) {
        assert.deepEqual(args, request.args);
        captures.push(source);
        return source.sourceId;
      } },
      onEvent: {},
      onProblem: duringRetirement,
    });
    server.handleMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
    server.handleMessage(request);
    assert.equal(captures.length, 1);
    const source = captures[0]!;
    assert.notEqual(source.sourceId, request.args.sourceId);
    assert.match(source.sourceId, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    assert.equal(source.assertCurrent(), undefined);
    if (retirement === "dispose") server.dispose();
    else if (retirement === "port-close") server.handlePortClosed();
    else if (retirement === "shutdown") server.handleMessage({ kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null });
    else server.handleMessage({ kind: "event", family: "undeclared", payload: null });
    await server.settled;
    assert.throws(source.assertCurrent, McpOriginRetiredError);
    server.handleMessage({ ...request, requestId: "after-retirement" });
    server.dispose();
    assert.equal(captures.length, 1);
    assert.equal(posted.filter((frame) => frame.kind === "reply").length, 0, "a retired port cannot claim delivery of the pending result");
    assert.equal(retirementCallbacks, retirement === "dispose" ? 1 : retirement === "protocol-breach" ? 2 : 0);
    assert.deepEqual(callbackStates, Array(retirementCallbacks).fill("retired"), "retirement precedes diagnostics and post callbacks");

    let replacement: CoordinatorControlSourceContext | undefined;
    const next = createCoordinatorControlServer({
      post() {}, executors: { capture: (_args, current) => { replacement = current; } },
      onEvent: {}, onProblem: (detail) => assert.fail(detail),
    });
    next.handleMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
    next.handleMessage(request);
    assert.ok(replacement);
    assert.notEqual(replacement.sourceId, source.sourceId);
    assert.equal(replacement.assertCurrent(), undefined);
    assert.throws(source.assertCurrent, McpOriginRetiredError);
    next.dispose();
  });
}

function bounded<T>(promise: Promise<T>, label: string): Promise<T> {
  return new Promise<T>((resolvePromise, reject) => {
    const timer = setTimeout(() => reject(new Error(`${label} did not settle`)), 10_000);
    promise.then(
      (value) => { clearTimeout(timer); resolvePromise(value); },
      (error) => { clearTimeout(timer); reject(error); },
    );
  });
}

// Only the Electron event-wrapper shape is adapted. Native MessagePorts still
// perform the real structured clone, and the shipping carrier owns JSON I/O.
const CARRIER_WORKER = String.raw`
const { parentPort, workerData } = require("node:worker_threads");
const childProcess = require("node:child_process");
const spawn = childProcess.spawn;
let child;
let transferred = [];
let stopping = false;
childProcess.spawn = (...args) => {
  child = spawn(...args);
  child.once("exit", () => { if (stopping) parentPort.close(); });
  if (stopping) child.kill();
  return child;
};
const adapt = (port) => ({
  postMessage: (value) => port.postMessage(value),
  on: (event, listener) => port.on(event, event === "message"
    ? (data) => listener({ data }) : listener),
  start: () => port.start(),
  close: () => port.close(),
});
Object.defineProperty(process, "parentPort", { value: {
  once: (_event, listener) => parentPort.once("message", (message) => {
    transferred = message.ports ?? [];
    listener({ data: message.data, ports: transferred.map(adapt) });
  }),
} });
parentPort.on("message", (message) => {
  if (message?.kind !== "stop") return;
  stopping = true;
  for (const port of transferred) port.close();
  if (child != null && child.exitCode == null && child.signalCode == null) child.kill();
  else parentPort.close();
});
require(workerData.carrierPath);
parentPort.postMessage({ kind: "loaded" });
`;

// This peer does not implement MCP or classify results. It records the exact
// stdin line emitted by the actual carrier and returns it on its other port.
const COORDINATOR_ECHO_PEER = String.raw`
const readline = require("node:readline");
const labels = JSON.parse(process.env.FABUSHI_CARRIER_CASES);
const emit = (channel, frame) => process.stdout.write(JSON.stringify({ channel, frame }) + "\n");
emit("coordinator-control", { kind: "lifecycle", phase: "hello", protocolVersion: 1 });
readline.createInterface({ input: process.stdin }).on("line", (line) => {
  const envelope = JSON.parse(line);
  if (envelope.channel !== "coordinator-control") throw new Error("unexpected input channel");
  const frame = envelope.frame;
  if (frame.kind === "lifecycle" && frame.phase === "ready") {
    for (const label of labels) emit("coordinator-control", {
      kind: "request", requestId: label, method: "executeRoutedMcpTool", args: { label },
    });
  } else if (frame.kind === "reply") {
    emit("coordinator-main-data", { wireLine: line });
  } else if (frame.kind === "lifecycle" && frame.phase === "shutdown") {
    process.exit(0);
  } else throw new Error("unexpected control frame");
});
`;

test("MCP results retain their discriminants through native ports and the shipping Rust carrier", {
  timeout: 45_000,
}, async () => {
  const artifact = process.env.FABUSHI_COORDINATOR_CARRIER_ARTIFACT;
  assert.ok(artifact, "Actions must supply the shipping carrier built for this exact H");
  const carrierPath = resolve(artifact);
  await access(carrierPath);
  assert.notEqual(process.platform, "win32", "this local peer fixture requires the scoped Unix runner");

  const nested = { error: "user field", result: { case: "error", value: { error: "payload only" } } };
  const text = `timeout-like user content: ${JSON.stringify(nested)}`;
  const typedSuccess = (isError: boolean) => new McpResult({ result: {
    case: "success",
    value: new McpSuccess({
      content: [generatedMcpResultFactory.textItem(text)],
      isError,
      structuredContent: Struct.fromJson(nested),
    }),
  } });
  const officialError = generatedMcpResultFactory.error("official transport result is unknown");
  // The generated prototype serializes to canonical protobuf JSON before a
  // clone. The native port must remove that prototype before carrier JSON I/O.
  assert.deepEqual(JSON.parse(JSON.stringify(officialError)), {
    error: { error: "official transport result is unknown" },
  });

  let backendCalls = 0;
  let backendFailures = 0;
  const unusedBackendMethod = async (): Promise<never> => { throw new Error("unexpected backend method"); };
  const backendClient: DashboardMcpExecClient = {
    listSandMcpTools: unusedBackendMethod,
    executeSandMcpTool: async () => { backendCalls += 1; throw new Error("deadline after dispatch"); },
    checkHttpMcpStatus: unusedBackendMethod,
    completeMcpOAuth: unusedBackendMethod,
    validateMcpOAuthTokens: unusedBackendMethod,
    deleteMcpOAuthToken: unusedBackendMethod,
    renameMcpOAuthAccount: unusedBackendMethod,
    deleteMcpOAuthAccount: unusedBackendMethod,
  };
  const backend = createDashboardSandBackendMcpExec({
    getAccessToken: async () => "unused-test-token",
    getMachineId: async () => "wire-contract-machine",
    createClient: () => backendClient,
    connectErrorCode: () => ({ name: "DeadlineExceeded", deadlineExceeded: true }),
    recordExecError: () => { backendFailures += 1; },
  });
  const rawResult = { content: [{ type: "text", text }], isError: true, structuredContent: nested };
  const producers = new Map<string, () => unknown | Promise<unknown>>([
    ["official-error", () => officialError],
    ["origin-retired", () => generatedMcpResultFactory.rejected(MCP_ORIGIN_RETIRED)],
    ["backend-deadline", () => backend.executeTool({
      serverIdentifier: "fixture", toolName: "write", args: {}, toolCallId: "wire-deadline",
    })],
    ["typed-success", () => typedSuccess(false)],
    ["typed-server-error", () => typedSuccess(true)],
    ["empty-success", () => new McpResult({ result: { case: "success", value: new McpSuccess() } })],
    ["raw-server-error", () => rawResult],
    ["canonical-error", () => officialError.toJson()],
    ["canonical-server-error", () => typedSuccess(true).toJson()],
    ["executor-failure", () => { throw new Error("executor failed after dispatch"); }],
  ]);

  const root = await mkdtemp(join(tmpdir(), "fabushi-runner-carrier-"));
  const peer = join(root, "coordinator-echo.cjs");
  await writeFile(peer, `#!${process.execPath}\n${COORDINATOR_ECHO_PEER}`, { mode: 0o700 });
  const channels = [new MessageChannel(), new MessageChannel(), new MessageChannel()] as const;
  const control = channels[0].port2;
  const wireOutput = channels[2].port2;
  const worker = new Worker(CARRIER_WORKER, {
    eval: true,
    workerData: { carrierPath },
    env: {
      ...process.env,
      SAND_COORDINATOR_BIN: peer,
      MAHAYANA_APP_HOST_BIN: process.execPath,
      FABUSHI_CARRIER_CASES: JSON.stringify([...producers.keys()]),
    },
  });
  const exited = new Promise<number>((resolveExit) => worker.once("exit", resolveExit));
  let rejectReplies!: (error: unknown) => void;
  const replies = new Promise<Map<string, any>>((resolveReplies, reject) => {
    rejectReplies = reject;
    const observed = new Map<string, any>();
    wireOutput.on("message", (message: unknown) => {
      try {
        assert.equal(typeof (message as { wireLine?: unknown }).wireLine, "string");
        const envelope = JSON.parse((message as { wireLine: string }).wireLine);
        assert.equal(envelope.channel, "coordinator-control");
        assert.equal(envelope.frame.kind, "reply");
        assert.ok(producers.has(envelope.frame.requestId));
        assert.ok(!observed.has(envelope.frame.requestId), "one reply per control request");
        observed.set(envelope.frame.requestId, envelope.frame.outcome);
        if (observed.size === producers.size) resolveReplies(observed);
      } catch (error) { reject(error); }
    });
    worker.on("error", reject);
    worker.on("exit", (code) => {
      if (observed.size !== producers.size) reject(new Error(`carrier exited before replies: ${code}`));
    });
  });
  // Install the rejection consumer before startup so a failed carrier cannot
  // turn an early worker error into an unhandled promise rejection.
  const boundedReplies = bounded(replies, "actual carrier replies");
  void boundedReplies.catch(() => {});
  const server = createCoordinatorControlServer({
    post: (frame) => control.postMessage(frame),
    executors: { executeRoutedMcpTool: (args) => {
      const label = (args as { label: string }).label;
      const produce = producers.get(label);
      assert.ok(produce, "the child can invoke only a declared result producer");
      return produce();
    } },
    onEvent: {},
    onProblem: (detail) => rejectReplies(new Error(detail)),
  });
  control.on("message", (frame) => server.handleMessage(frame));

  try {
    const [loaded] = await bounded(once(worker, "message"), "carrier module loading");
    assert.deepEqual(loaded, { kind: "loaded" });
    worker.postMessage({
      data: { bootstrap: { processConfig: { appVersion: "contract", isPackaged: false, dataDir: root } } },
      ports: channels.map((channel) => channel.port1),
    }, channels.map((channel) => channel.port1));
    const received = await boundedReplies;
    for (const [label, outcome] of received) {
      assert.equal(outcome.status, label === "executor-failure" ? "failed" : "ok", label);
    }
    assert.deepEqual(received.get("official-error").value, {
      result: { case: "error", value: { error: "official transport result is unknown" } },
    });
    assert.deepEqual(received.get("origin-retired").value, {
      result: { case: "rejected", value: { reason: MCP_ORIGIN_RETIRED, isReadonly: false } },
    });
    const deadline = received.get("backend-deadline").value;
    assert.equal(deadline.result.case, "error");
    assert.match(deadline.result.value.error, /connector may still have applied it/);
    assert.equal(backendCalls, 1);
    assert.equal(backendFailures, 1);
    for (const label of ["typed-success", "typed-server-error"]) {
      const value = received.get(label).value;
      assert.equal(value.result.case, "success");
      assert.equal(value.result.value.isError, label === "typed-server-error");
      assert.equal(value.result.value.content[0].content.case, "text");
      assert.equal(value.result.value.content[0].content.value.text, text);
      assert.deepEqual(new Struct(value.result.value.structuredContent).toJson(), nested);
    }
    assert.deepEqual(received.get("empty-success").value, {
      result: { case: "success", value: { content: [], isError: false } },
    });
    assert.deepEqual(received.get("raw-server-error").value, rawResult);
    assert.deepEqual(received.get("canonical-error").value, {
      error: { error: "official transport result is unknown" },
    });
    const canonical = received.get("canonical-server-error").value;
    assert.equal(canonical.success.isError, true);
    assert.deepEqual(canonical.success.structuredContent, nested);
    assert.equal(canonical.success.content[0].text.text, text);
    assert.deepEqual(received.get("executor-failure").failure, {
      code: "main-execution-failure", message: "executor failed after dispatch",
    });
  } finally {
    server.dispose();
    try { worker.postMessage({ kind: "stop" }); } catch { /* Already exited; still join it below. */ }
    try {
      await bounded(exited, "owned carrier and echo peer shutdown");
    } finally {
      await worker.terminate();
      for (const channel of channels) { channel.port1.close(); channel.port2.close(); }
      await rm(root, { recursive: true, force: true });
    }
  }
});
