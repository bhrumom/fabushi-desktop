import assert from "node:assert/strict";
import { once } from "node:events";
import { mkdtempSync, rmSync } from "node:fs";
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test, { type TestContext } from "node:test";
import { Struct } from "@bufbuild/protobuf";
import { Code, ConnectError, createClient, createContextValues, type Transport } from "@connectrpc/connect";
import { runUnaryCall } from "@connectrpc/connect/protocol";

import { createCoordinatorControlServer } from "../../../electron-main/coordinator/coordinator-control-server.js";
import { createSandDesktopMcpManager, type DesktopMcpManagerFacade } from "../../../electron-main/mcp/desktop-mcp-manager.js";
import { McpResult, McpSuccess } from "../../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import { DashboardService } from "../../../packages/proto/generated/aiserver/v1/dashboard_connect.js";
import { ExecuteSandMcpToolRequest, ExecuteSandMcpToolResponse } from "../../../packages/proto/generated/aiserver/v1/dashboard_pb.js";
import { COORDINATOR_PROTOCOL_VERSION, type CoordinatorControlSourceContext, type CoordinatorFrame } from "../../rpc/coordinator-port.js";
import { pinMcpDiagnosticsReporter } from "../mcp/mcp-diagnostics.js";
import { generatedMcpResultFactory, MCP_ORIGIN_RETIRED, McpOriginRetiredError } from "../mcp/mcp-result-factory.js";
import { createMcpToolsDiscovery } from "../mcp/tools-discovery.js";
import { SandSettingsStore } from "../settings/sand-settings-store.js";
import { createDashboardSandBackendMcpExec, type DashboardMcpExecClient, type DashboardMcpExecClientOptions } from "./backend-mcp-exec.js";
import { clearSandPrivacyModeCacheForTesting, createSandInferenceInterceptor, type PrivacyLookupOptions } from "./cursor-inference.js";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

// Capture the native context from a real control-server dispatch. Wire input
// never manufactures a lifetime assertion, even in this transport fixture.
function sourceFixture(t: TestContext) {
  const sources: CoordinatorControlSourceContext[] = [];
  const frames: CoordinatorFrame[] = [];
  const problems: string[] = [];
  const control = createCoordinatorControlServer({
    post: (frame) => { frames.push(frame); },
    executors: { capture: (_args, source) => { sources.push(source); return null; } },
    onEvent: {},
    onProblem: (problem) => { problems.push(problem); },
  });
  t.after(() => control.dispose());
  control.handleMessage({ kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  control.handleMessage({ kind: "request", requestId: "capture-source", method: "capture", args: {} });
  assert.deepEqual(problems, []);
  assert.deepEqual(frames[0], { kind: "lifecycle", phase: "ready", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  assert.equal(sources.length, 1);
  const source = sources[0]!;
  assert.equal(source.assertCurrent(), undefined);
  return { source, retire: () => control.dispose() };
}

const routedRequest = {
  providerIdentifier: "origin-http",
  name: "public-write",
  toolName: "raw_write",
  args: { nested: { values: [1, false, null, "literal"] }, label: "unchanged" },
  toolCallId: "call-original",
  agentId: "agent-original",
};
const backendRequest = {
  serverIdentifier: routedRequest.providerIdentifier,
  toolName: routedRequest.name,
  args: routedRequest.args,
  toolCallId: routedRequest.toolCallId,
  agentId: routedRequest.agentId,
};

function resultMessage(isError = false): McpResult {
  return new McpResult({ result: { case: "success", value: new McpSuccess({
    content: [generatedMcpResultFactory.textItem(isError ? "server refused the operation" : "operation completed")],
    isError,
    structuredContent: Struct.fromJson({ nested: ["server-owned", 23, false] }),
  }) } });
}

function assertResult(actual: unknown, expected: McpResult): void {
  assert.ok(actual instanceof McpResult, "the generated result must survive the adapter");
  assert.deepEqual(actual.toJson(), expected.toJson());
  assert.deepEqual(structuredClone(actual), structuredClone(expected));
}

function assertRefused(actual: unknown): void {
  assertResult(actual, generatedMcpResultFactory.rejected(MCP_ORIGIN_RETIRED));
}

function assertWireRequest(request: ExecuteSandMcpToolRequest, expected = routedRequest): void {
  assert.deepEqual(request.toJson(), {
    serverIdentifier: expected.providerIdentifier,
    toolName: expected.name,
    args: expected.args,
    toolCallId: expected.toolCallId,
    agentId: expected.agentId,
  });
}

type TransportFixtureOptions = {
  readonly getAccessToken?: () => Promise<string>;
  readonly getMachineId?: () => Promise<string>;
  readonly privacy?: (lookup: PrivacyLookupOptions, clientIndex: number) => Promise<"true" | "false">;
  readonly onRequestId?: (requestId: string) => void;
  readonly execute?: (request: ExecuteSandMcpToolRequest) => Promise<ExecuteSandMcpToolResponse>;
  readonly timeoutMs?: number;
  readonly recordExecError?: () => void;
};

function transportFixture(options: TransportFixtureOptions = {}) {
  const clients: DashboardMcpExecClientOptions[] = [];
  const nextCalls: { clientIndex: number; method: string }[] = [];
  const executions: { request: ExecuteSandMcpToolRequest; header: Headers }[] = [];
  const transportErrors: unknown[] = [];
  const execErrors: unknown[] = [];
  let requestSequence = 0;
  const backend = createDashboardSandBackendMcpExec({
    getAccessToken: options.getAccessToken ?? (async () => "fixture-token"),
    getMachineId: options.getMachineId ?? (async () => "fixture-machine"),
    recordExecError: (_toolCallId, error) => { execErrors.push(error); options.recordExecError?.(); },
    connectErrorCode: (error) => error instanceof ConnectError
      ? { name: Code[error.code], deadlineExceeded: error.code === Code.DeadlineExceeded }
      : undefined,
    createClient(credentials) {
      const clientIndex = clients.length;
      clients.push(credentials);
      const inference = createSandInferenceInterceptor({
        getAccessToken: credentials.getAccessToken,
        getMachineId: credentials.getMachineId,
        beforeDispatch: credentials.beforeDispatch,
        backendUrl: "https://transport.example.test/",
        resolveGhostModeHeader: (lookup) => options.privacy?.(lookup, clientIndex) ?? Promise.resolve("true"),
        onRequestId: options.onRequestId,
        randomUUID: () => `request-${++requestSequence}`,
        env: {},
      });
      const transport: Transport = {
        unary(service, method, signal, timeoutMs, header, input, contextValues) {
          // Keep the generated client, request normalization, interceptor and
          // Connect error wrapping real; replace only the final network effect.
          return runUnaryCall({
            req: {
              service, method, stream: false,
              url: `https://transport.example.test/${service.typeName}/${method.name}`,
              init: {}, header: new Headers(header), message: input,
              contextValues: contextValues ?? createContextValues(),
            },
            signal,
            timeoutMs: options.timeoutMs ?? timeoutMs,
            interceptors: [inference],
            next: async (request) => {
              nextCalls.push({ clientIndex, method: method.name });
              let bytes: Uint8Array;
              if (method.name === DashboardService.methods.executeSandMcpTool.name) {
                const decoded = ExecuteSandMcpToolRequest.fromBinary(request.message.toBinary());
                executions.push({ request: decoded, header: new Headers(request.header) });
                const response = await (options.execute?.(decoded) ?? Promise.resolve(new ExecuteSandMcpToolResponse({ result: resultMessage() })));
                bytes = response.toBinary();
              } else if (method.name === DashboardService.methods.listSandMcpTools.name ||
                         method.name === DashboardService.methods.completeMcpOAuth.name) {
                bytes = new method.O().toBinary();
              } else {
                throw new Error(`unexpected fixture RPC: ${method.name}`);
              }
              return { service, method, stream: false, header: new Headers(), trailer: new Headers(), message: method.O.fromBinary(bytes) };
            },
          }).catch((error: unknown) => { transportErrors.push(error); throw error; });
        },
        stream() { throw new Error("unexpected streaming fixture RPC"); },
      };
      return createClient(DashboardService, transport) as unknown as DashboardMcpExecClient;
    },
  });
  return { backend, clients, nextCalls, executions, transportErrors, execErrors };
}

function discoveryFixture(backend: ReturnType<typeof createDashboardSandBackendMcpExec>) {
  let disabled: Record<string, string[]> = {};
  const reports: Record<string, unknown>[] = [];
  const discovery = createMcpToolsDiscovery({
    backendMcpExec: backend,
    definitionSource: { getServerUrlForIdentifier: async () => "https://connector.example.test/mcp" },
    lastAccountDisplayConfig: () => ({ servers: [{ id: "71", name: "Origin HTTP", serverIdentifier: "origin-http", config: { url: "https://connector.example.test/mcp" } }] }),
    settingsStore: () => ({
      getMcpDisabledToolsByServerId: () => disabled,
      getRawMcpCustomInstructionByServerId: () => undefined,
      getRawMcpCustomInstruction: () => undefined,
    }),
  }, { onConnectorAuth: (report) => { reports.push(report); } });
  return {
    reports,
    disableRawTool: (value: boolean) => { disabled = value ? { "71": [routedRequest.toolName] } : {}; },
    execute: (source: CoordinatorControlSourceContext, request = routedRequest) => discovery.executeTool(
      undefined, request, { agentId: request.agentId }, source.assertCurrent,
    ),
  };
}

test("MCP origin: current generated transport preserves public/raw names, nested args and disabled checks", async (t) => {
  const { source } = sourceFixture(t);
  const transport = transportFixture();
  const discovery = discoveryFixture(transport.backend);
  discovery.disableRawTool(true);
  const disabled = await discovery.execute(source);
  assert.equal(disabled.result.case, "error");
  assert.equal(transport.nextCalls.length, 0);
  const sibling = { ...routedRequest, name: "public-sibling", toolName: "raw_sibling", toolCallId: "call-sibling" };
  assertResult(await discovery.execute(source, sibling), resultMessage());
  assert.equal(transport.nextCalls.length, 1);
  assertWireRequest(transport.executions[0]!.request, sibling);
  assert.equal(transport.executions[0]!.header.get("authorization"), "Bearer fixture-token");
  assert.equal(transport.executions[0]!.header.get("x-ghost-mode"), "true");
  assert.equal(transport.executions[0]!.header.get("x-request-id"), "request-1");
  assert.deepEqual(discovery.reports, [{ phase: "first_call_ok", outcome: "ok", serverName: "Origin HTTP", serverId: "71" }]);
  assert.equal(transport.clients.length, 2);
  assert.equal(transport.clients[0]!.beforeDispatch, undefined);
  assert.equal(typeof transport.clients[1]!.beforeDispatch, "function");
});

test("MCP origin: held privacy retires before real Connect next and does not consume first-call diagnostics", { timeout: 10_000 }, async (t) => {
  const old = sourceFixture(t);
  const current = sourceFixture(t);
  const entered = deferred<PrivacyLookupOptions>();
  const privacy = deferred<"true">();
  t.after(() => privacy.resolve("true"));
  const transport = transportFixture({
    privacy: async (lookup) => { entered.resolve(lookup); return await privacy.promise; },
    recordExecError: () => { throw new Error("retirement must not enter execution-error diagnostics"); },
  });
  const discovery = discoveryFixture(transport.backend);
  const pending = discovery.execute(old.source);
  const lookup = await entered.promise;
  assert.equal(lookup.accessToken, "fixture-token");
  assert.equal(lookup.machineId, "fixture-machine");
  old.retire();
  privacy.resolve("true");
  assertRefused(await pending);
  assert.equal(transport.nextCalls.length, 0);
  assert.equal(transport.execErrors.length, 0);
  assert.equal(transport.transportErrors.length, 1);
  assert.ok(transport.transportErrors[0] instanceof ConnectError);
  assert.ok(transport.transportErrors[0].cause instanceof McpOriginRetiredError);
  assert.deepEqual(discovery.reports, []);
  assertResult(await discovery.execute(current.source), resultMessage());
  assert.equal(transport.nextCalls.length, 1);
  assert.equal(discovery.reports.length, 1);
  assert.equal(discovery.reports[0]!.outcome, "ok");
});

test("MCP origin: held token or machine ID cannot pass a retired source to next", { timeout: 10_000 }, async (t) => {
  for (const held of ["token", "machine"] as const) {
    const origin = sourceFixture(t);
    const bothStarted = [deferred<undefined>(), deferred<undefined>()];
    const credential = deferred<string>();
    t.after(() => credential.resolve(`fixture-${held}`));
    const transport = transportFixture({
      getAccessToken: async () => { bothStarted[0]!.resolve(undefined); return held === "token" ? await credential.promise : "fixture-token"; },
      getMachineId: async () => { bothStarted[1]!.resolve(undefined); return held === "machine" ? await credential.promise : "fixture-machine"; },
    });
    const pending = transport.backend.executeTool(backendRequest, origin.source.assertCurrent);
    await Promise.all(bothStarted.map((gate) => gate.promise));
    origin.retire();
    credential.resolve(`fixture-${held}`);
    assertRefused(await pending);
    assert.equal(transport.nextCalls.length, 0, held);
  }
});

test("MCP origin: final assertion follows the request-ID observer and survives Connect deadline replacement", async (t) => {
  for (const replaceWithDeadline of [false, true]) {
    const origin = sourceFixture(t);
    let observations = 0;
    const transport = transportFixture({
      // runUnaryCall's own nonpositive deadline is already expired. It still
      // invokes the interceptor, then substitutes its deadline in the catch.
      timeoutMs: replaceWithDeadline ? 0 : undefined,
      onRequestId: () => {
        observations += 1;
        origin.retire();
      },
    });
    assertRefused(await transport.backend.executeTool(backendRequest, origin.source.assertCurrent));
    assert.equal(observations, 1);
    assert.equal(transport.nextCalls.length, 0);
    assert.ok(transport.transportErrors[0] instanceof ConnectError);
    if (replaceWithDeadline) {
      assert.equal(transport.transportErrors[0].code, Code.DeadlineExceeded);
      assert.ok(!(transport.transportErrors[0].cause instanceof McpOriginRetiredError));
    }
  }
});

test("MCP origin: concurrent guarded requests retain separate sources and reverse-order completion", { timeout: 10_000 }, async (t) => {
  const old = sourceFixture(t);
  const current = sourceFixture(t);
  const entered = [deferred<undefined>(), deferred<undefined>()];
  const privacy = [deferred<"true">(), deferred<"true">()];
  t.after(() => { for (const gate of privacy) gate.resolve("true"); });
  const transport = transportFixture({ privacy: async (_lookup, clientIndex) => {
    assert.ok(clientIndex === 1 || clientIndex === 2);
    entered[clientIndex - 1]!.resolve(undefined);
    return await privacy[clientIndex - 1]!.promise;
  } });
  const oldCall = transport.backend.executeTool({ ...backendRequest, toolCallId: "old-call" }, old.source.assertCurrent);
  const currentCall = transport.backend.executeTool({ ...backendRequest, toolCallId: "current-call" }, current.source.assertCurrent);
  await Promise.all(entered.map((gate) => gate.promise));
  old.retire();
  privacy[1]!.resolve("true");
  assertResult(await currentCall, resultMessage());
  privacy[0]!.resolve("true");
  assertRefused(await oldCall);
  assert.equal(transport.nextCalls.length, 1);
  assert.equal(transport.nextCalls[0]!.clientIndex, 2);
  assertWireRequest(transport.executions[0]!.request, { ...routedRequest, toolCallId: "current-call" });
  assert.notEqual(transport.clients[1]!.beforeDispatch, transport.clients[2]!.beforeDispatch);
});

test("MCP origin: matching text and typed exceptions outside the final assertion are not known refusals", async () => {
  const plain = transportFixture();
  const ordinary = await plain.backend.executeTool(backendRequest, () => { throw new Error(MCP_ORIGIN_RETIRED); });
  assert.equal(ordinary.result.case, "error");
  assert.equal(plain.nextCalls.length, 0);
  assert.equal(plain.execErrors.length, 1);
  assert.ok(plain.transportErrors[0] instanceof ConnectError);
  assert.ok(!(plain.transportErrors[0].cause instanceof McpOriginRetiredError));

  let assertions = 0;
  const credential = transportFixture({ getAccessToken: async () => { throw new McpOriginRetiredError(); } });
  const early = await credential.backend.executeTool(backendRequest, () => { assertions += 1; return undefined; });
  assert.equal(early.result.case, "error");
  assert.equal(assertions, 0);
  assert.equal(credential.nextCalls.length, 0);
  assert.equal(credential.execErrors.length, 1);
});

test("MCP origin: entered late success, server failure and uncertain transport settlement retain custody", { timeout: 10_000 }, async (t) => {
  const outcomes = [
    { name: "success", result: resultMessage() },
    { name: "server-declared-failure", result: resultMessage(true) },
    { name: "server-error-result", result: generatedMcpResultFactory.error("server refused the operation") },
    { name: "unset-server-result", result: new McpResult() },
    { name: "transport-unavailable", error: new ConnectError("reply lost", Code.Unavailable) },
    { name: "transport-deadline", error: new ConnectError("reply timed out", Code.DeadlineExceeded) },
    { name: "typed-error-from-next", error: new McpOriginRetiredError() },
  ];
  for (const outcome of outcomes) {
    const origin = sourceFixture(t);
    const entered = deferred<undefined>();
    const reply = deferred<ExecuteSandMcpToolResponse>();
    t.after(() => reply.resolve(new ExecuteSandMcpToolResponse({ result: resultMessage() })));
    const transport = transportFixture({ execute: async () => { entered.resolve(undefined); return await reply.promise; } });
    const pending = transport.backend.executeTool(backendRequest, origin.source.assertCurrent);
    await entered.promise;
    assert.equal(transport.nextCalls.length, 1, outcome.name);
    origin.retire();
    if (outcome.result !== undefined) reply.resolve(new ExecuteSandMcpToolResponse({ result: outcome.result }));
    else reply.reject(outcome.error);
    const result = await pending;
    if (outcome.result !== undefined) assertResult(result, outcome.result);
    else {
      assert.equal(result.result.case, "error", outcome.name);
      assert.equal(transport.execErrors.length, 1, outcome.name);
    }
    assert.equal(transport.nextCalls.length, 1, "late settlement must never retry");
    assert.equal(transport.clients.length, 2, "late settlement must never acquire a replacement client");
  }
});

test("MCP origin: guarded refusal does not attach its assertion to shared listing, auth or unguarded execution", async (t) => {
  const origin = sourceFixture(t);
  const transport = transportFixture();
  origin.retire();
  assertRefused(await transport.backend.executeTool(backendRequest, origin.source.assertCurrent));
  assert.deepEqual(await transport.backend.listTools(["origin-http"]), []);
  await transport.backend.completeOAuth({ stateId: "fixture-state", code: "fixture-code" });
  assertResult(await transport.backend.executeTool(backendRequest), resultMessage());
  assert.equal(transport.clients.length, 2);
  assert.deepEqual(transport.nextCalls, [
    { clientIndex: 0, method: "ListSandMcpTools" },
    { clientIndex: 0, method: "CompleteMcpOAuth" },
    { clientIndex: 0, method: "ExecuteSandMcpTool" },
  ]);
});

test("MCP origin: default desktop facade forwards the assertion through its real HTTP client factory", { concurrency: false, timeout: 15_000 }, async (t) => {
  const old = sourceFixture(t);
  const current = sourceFixture(t);
  const folder = mkdtempSync(join(tmpdir(), "mcp-origin-http-"));
  const settings = new SandSettingsStore(join(folder, "settings.json"));
  const previousBackend = process.env.FABUSHI_API_BASE_URL;
  const privacyEntered = deferred<undefined>();
  const releasePrivacy = deferred<undefined>();
  const requests: ExecuteSandMcpToolRequest[] = [];
  const problems: unknown[] = [];
  const pending: Promise<unknown>[] = [];
  let holdPrivacy = false;
  let facade: DesktopMcpManagerFacade | undefined;

  async function handleRequest(request: IncomingMessage, response: ServerResponse): Promise<void> {
    const chunks: Buffer[] = [];
    for await (const chunk of request) chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    const body = Buffer.concat(chunks);
    const method = request.url?.split("/").at(-1);
    let bytes: Uint8Array;
    switch (method) {
      case "GetUserPrivacyMode":
        DashboardService.methods.getUserPrivacyMode.I.fromBinary(body);
        if (holdPrivacy) { privacyEntered.resolve(undefined); await releasePrivacy.promise; }
        bytes = DashboardService.methods.getUserPrivacyMode.O.fromJson({ privacyMode: 2 }).toBinary();
        break;
      case "GetAvailableMcpServers":
        DashboardService.methods.getAvailableMcpServers.I.fromBinary(body);
        bytes = DashboardService.methods.getAvailableMcpServers.O.fromJson({ servers: [{
          id: 71, name: "Origin HTTP", serverIdentifier: "origin-http", type: "http",
          url: "https://connector.example.test/mcp", enabled: true,
        }] }).toBinary();
        break;
      case "ListSandMcpTools": {
        const input = DashboardService.methods.listSandMcpTools.I.fromBinary(body);
        bytes = DashboardService.methods.listSandMcpTools.O.fromJson({ servers: input.serverIdentifiers.map((serverIdentifier) => ({
          serverIdentifier, status: "connected", rowServerIdentifier: serverIdentifier,
          accountLabel: "default", tools: serverIdentifier === "origin-http" ? [{
            name: routedRequest.name, toolName: routedRequest.toolName,
            providerIdentifier: "origin-http", description: "Fixture write tool", inputSchema: { type: "object" },
          }] : [],
        })) }).toBinary();
        break;
      }
      case "ExecuteSandMcpTool":
        requests.push(ExecuteSandMcpToolRequest.fromBinary(body));
        bytes = new ExecuteSandMcpToolResponse({ result: resultMessage() }).toBinary();
        break;
      default: throw new Error(`unexpected default-facade RPC: ${method}`);
    }
    if (response.destroyed) return;
    response.writeHead(200, { "content-type": "application/proto", "content-length": bytes.byteLength, "connect-protocol-version": "1" });
    response.end(Buffer.from(bytes));
  }

  const server = createServer((request, response) => {
    void handleRequest(request, response).catch((error: unknown) => {
      problems.push(error);
      if (response.destroyed) return;
      response.writeHead(500, { "content-type": "application/json" });
      response.end(JSON.stringify({ code: "internal", message: "fixture RPC failed" }));
    });
  });
  const unexpectedBox = async (): Promise<never> => { throw new Error("HTTP-only fixture entered Box"); };
  let cleanupPromise: Promise<void> | undefined;
  const cleanup = () => cleanupPromise ??= (async () => {
    old.retire();
    current.retire();
    releasePrivacy.resolve(undefined);
    const closed = server.listening
      ? new Promise<void>((resolve, reject) => server.close((error) => error === undefined ? resolve() : reject(error)))
      : Promise.resolve();
    server.closeAllConnections();
    try {
      await Promise.allSettled(pending);
      await facade?.dispose();
      await closed;
    } finally {
      clearSandPrivacyModeCacheForTesting();
      pinMcpDiagnosticsReporter(null);
      if (previousBackend === undefined) delete process.env.FABUSHI_API_BASE_URL;
      else process.env.FABUSHI_API_BASE_URL = previousBackend;
      rmSync(folder, { recursive: true, force: true });
    }
  })();
  t.after(cleanup);
  try {
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    const address = server.address();
    assert.ok(address !== null && typeof address === "object");
    process.env.FABUSHI_API_BASE_URL = `http://127.0.0.1:${address.port}`;
    clearSandPrivacyModeCacheForTesting();
    const token = `fixture.${Buffer.from(JSON.stringify({ sub: "origin-http-fixture" })).toString("base64url")}.signature`;
    facade = await createSandDesktopMcpManager({
      settingsStore: settings,
      onAccountScopeApplied: () => {},
      getAccessToken: async () => token,
      getMachineId: async () => "fixture-machine",
      loadBoxMcpServers: unexpectedBox,
      listBoxMcpServers: unexpectedBox,
      listBoxMcpToolsRaw: unexpectedBox,
      executeBoxMcpToolRaw: unexpectedBox,
      onConnectorAuth: () => {},
      onMcpDiscoveryFailed: (problem) => { problems.push(problem); },
      onMcpDiagnostic: (problem) => { problems.push(problem); },
    });
    const tools = await facade.listRoutedTools();
    assert.ok(Array.isArray(tools));
    assert.equal(tools.length, 1);
    assert.equal(tools[0].name, routedRequest.name);
    assert.equal(tools[0].toolName, routedRequest.toolName);

    clearSandPrivacyModeCacheForTesting();
    holdPrivacy = true;
    const stopped = facade.executeRoutedTool(routedRequest, old.source.assertCurrent);
    pending.push(stopped);
    await privacyEntered.promise;
    assert.equal(requests.length, 0);
    old.retire();
    releasePrivacy.resolve(undefined);
    assertRefused(await stopped);
    assert.equal(requests.length, 0, "the private generatedBackendClient must forward the final assertion");

    holdPrivacy = false;
    settings.setMcpDisabledToolsByServerId({ "71": [routedRequest.toolName] });
    const disabled = await facade.executeRoutedTool(routedRequest, current.source.assertCurrent);
    assert.ok(disabled instanceof McpResult);
    assert.equal(disabled.result.case, "error");
    assert.equal(requests.length, 0);
    settings.setMcpDisabledToolsByServerId({});
    assertResult(await facade.executeRoutedTool(routedRequest, current.source.assertCurrent), resultMessage());
    assert.equal(requests.length, 1);
    assertWireRequest(requests[0]!);
    assert.deepEqual(problems, []);
  } finally {
    await cleanup();
  }
});
