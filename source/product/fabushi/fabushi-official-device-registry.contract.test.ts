import assert from "node:assert/strict";
import test from "node:test";

import type { FabushiDeviceAgentSession } from "../../electron-main/account/auth-service-contract.js";
import {
  FABUSHI_APP_AGENT_STATUS_TOOL,
  FABUSHI_APP_AGENT_TOOLS,
  FabushiOfficialDeviceRegistryService,
  type OfficialDeviceRegistrySocket,
  resolveFabushiDeviceGatewayUrl,
} from "./fabushi-official-device-registry.js";

class FakeSocket implements OfficialDeviceRegistrySocket {
  readyState = 0;
  readonly sent: Array<Record<string, unknown>> = [];
  readonly listeners = new Map<string, Array<(...args: unknown[]) => void>>();
  closed = false;

  send(data: string, callback?: (error?: Error) => void): void {
    this.sent.push(JSON.parse(data) as Record<string, unknown>);
    callback?.();
  }

  close(): void {
    if (this.closed) return;
    this.closed = true;
    this.readyState = 3;
    this.emit("close");
  }

  terminate(): void { this.close(); }

  on(event: string, listener: (...args: unknown[]) => void): this {
    const list = this.listeners.get(event) ?? [];
    list.push(listener);
    this.listeners.set(event, list);
    return this;
  }

  open(): void {
    this.readyState = 1;
    this.emit("open");
  }

  message(value: unknown): void {
    this.emit("message", Buffer.from(JSON.stringify(value), "utf8"));
  }

  private emit(event: string, ...args: unknown[]): void {
    for (const listener of this.listeners.get(event) ?? []) listener(...args);
  }
}

function session(label: string): FabushiDeviceAgentSession {
  return {
    accessToken: ("fabushi-account-token-" + label + "-").padEnd(48, "x"),
    accessTokenExpiresAt: Date.now() + 10 * 60_000,
    deviceId: "device-" + label,
    sessionId: "browser:" + label,
    username: label + "@example.com",
  };
}

async function settle(): Promise<void> {
  await new Promise<void>((resolve) => setImmediate(resolve));
}

test("packaged registry is pinned to the official WSS /agent origin and dev overrides fail closed", () => {
  assert.equal(
    resolveFabushiDeviceGatewayUrl(true, { FABUSHI_REMOTE_DEVICE_GATEWAY_URL: "wss://evil.example/agent" }),
    "wss://fabushi-mcp.ombhrum.com/agent",
  );
  assert.equal(resolveFabushiDeviceGatewayUrl(false, {}), null);
  assert.equal(
    resolveFabushiDeviceGatewayUrl(false, { FABUSHI_REMOTE_DEVICE_GATEWAY_URL: "wss://localhost/agent" }),
    "wss://localhost/agent",
  );
  assert.throws(
    () => resolveFabushiDeviceGatewayUrl(false, { FABUSHI_REMOTE_DEVICE_GATEWAY_URL: "wss://localhost/agent?token=secret" }),
    /clean wss/,
  );
});

test("account switch and revoke close the previous registry socket without putting Fabushi bearer material in registration payloads", async () => {
  let current: FabushiDeviceAgentSession | null = session("a");
  const sockets: FakeSocket[] = [];
  const bearerHeaders: string[] = [];
  const service = new FabushiOfficialDeviceRegistryService({
    isPackaged: true,
    env: {},
    userDataPath: "/private/fabushi",
    getDeviceAgentSession: async () => current,
    connect: (_url, accessToken) => {
      bearerHeaders.push(accessToken);
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    },
    surfaceStatus: async () => ({ available: true }),
    pollMs: 60_000,
    heartbeatMs: 60_000,
  });

  await service.syncNow();
  assert.equal(sockets.length, 1);
  sockets[0]!.open();
  await settle();
  const firstSession = current;
  assert.ok(firstSession);
  const registerA = sockets[0]!.sent.find((entry) => entry.type === "register");
  assert.ok(registerA);
  assert.deepEqual(registerA.capabilities, [...FABUSHI_APP_AGENT_TOOLS]);
  assert.equal(JSON.stringify(registerA).includes(firstSession.accessToken), false);
  assert.equal(bearerHeaders[0], firstSession.accessToken);

  current = session("b");
  const secondSession = current;
  await service.syncNow();
  assert.equal(sockets[0]!.closed, true);
  assert.equal(sockets.length, 2);
  assert.notEqual(bearerHeaders[1], bearerHeaders[0]);

  sockets[1]!.open();
  await settle();
  const registerB = sockets[1]!.sent.find((entry) => entry.type === "register");
  assert.ok(registerB);
  assert.equal(JSON.stringify(registerB).includes(firstSession.accessToken), false);
  assert.equal(JSON.stringify(registerB).includes(secondSession.accessToken), false);

  current = null;
  await service.syncNow();
  assert.equal(sockets[1]!.closed, true);
  await service.stop();
});

test("device_call delegates only the advertised read-only status tool and never retries write-shaped or stale-account outcomes", async () => {
  let current: FabushiDeviceAgentSession | null = session("call");
  let surfaceCalls = 0;
  const sockets: FakeSocket[] = [];
  const service = new FabushiOfficialDeviceRegistryService({
    isPackaged: true,
    env: {},
    userDataPath: "/private/fabushi",
    getDeviceAgentSession: async () => current,
    connect: () => {
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    },
    surfaceStatus: async () => {
      surfaceCalls += 1;
      return { version: 1, appId: "fabushi.desktop", available: true, secretProbe: current?.accessToken };
    },
    pollMs: 60_000,
    heartbeatMs: 60_000,
  });

  await service.syncNow();
  const socket = sockets[0]!;
  socket.open();
  await settle();

  socket.message({
    type: "call",
    requestId: "read-1",
    toolName: FABUSHI_APP_AGENT_STATUS_TOOL,
    arguments: {},
    diskPreflight: { impact: "none", reason: "read only", targets: [] },
  });
  await settle();
  assert.equal(surfaceCalls, 1);
  const activeSession = current;
  assert.ok(activeSession);
  const readResult = socket.sent.find((entry) => entry.type === "result" && entry.requestId === "read-1");
  assert.equal(readResult?.ok, true);
  assert.equal(JSON.stringify(readResult).includes(activeSession.accessToken), false, "Fabushi account bearer must never enter a device result");

  socket.message({
    type: "call",
    requestId: "write-1",
    toolName: FABUSHI_APP_AGENT_STATUS_TOOL,
    arguments: {},
    diskPreflight: { impact: "writes", reason: "not read only", targets: ["/tmp/write"] },
  });
  await settle();
  assert.equal(surfaceCalls, 1, "a denied write-shaped request must not execute or retry the local Host surface");
  const writeResult = socket.sent.find((entry) => entry.type === "result" && entry.requestId === "write-1");
  assert.equal(writeResult?.ok, false);

  let release!: () => void;
  const gate = new Promise<void>((resolve) => { release = resolve; });
  const staleServiceSockets: FakeSocket[] = [];
  let staleSurfaceCalls = 0;
  let staleCurrent: FabushiDeviceAgentSession | null = session("stale-a");
  const staleService = new FabushiOfficialDeviceRegistryService({
    isPackaged: true,
    env: {},
    userDataPath: "/private/fabushi",
    getDeviceAgentSession: async () => staleCurrent,
    connect: () => {
      const next = new FakeSocket();
      staleServiceSockets.push(next);
      return next;
    },
    surfaceStatus: async () => {
      staleSurfaceCalls += 1;
      await gate;
      return { available: true };
    },
    pollMs: 60_000,
    heartbeatMs: 60_000,
  });

  await staleService.syncNow();
  const staleSocket = staleServiceSockets[0]!;
  staleSocket.open();
  await settle();
  staleSocket.message({
    type: "call",
    requestId: "stale-1",
    toolName: FABUSHI_APP_AGENT_STATUS_TOOL,
    arguments: {},
    diskPreflight: { impact: "none", reason: "read only", targets: [] },
  });
  await settle();
  staleCurrent = session("stale-b");
  release();
  await settle();
  assert.equal(staleSurfaceCalls, 1);
  assert.equal(staleSocket.closed, true);
  assert.equal(
    staleSocket.sent.some((entry) => entry.type === "result" && entry.requestId === "stale-1"),
    false,
    "an account-switch race must not return an A result into B state",
  );

  await staleService.stop();
  await service.stop();
});

test("private transport rejects missing routes, fences revoked sessions before dispatch, and redacts bearer results", async () => {
  const first = session("private");
  let current: FabushiDeviceAgentSession | null = first;
  let dispatches = 0;
  const sockets: FakeSocket[] = [];
  const route = { nodeId: "100.64.0.12", cellId: "cell-123456", generation: 2, expiresAt: Math.floor(Date.now() / 1_000) + 300 };
  const service = new FabushiOfficialDeviceRegistryService({
    isPackaged: true, env: {}, userDataPath: "/private/fabushi",
    getDeviceAgentSession: async () => current,
    connect: () => { const socket = new FakeSocket(); sockets.push(socket); return socket; },
    privateTransportRequest: async ({ networkRoute }) => {
      dispatches += 1;
      assert.deepEqual(networkRoute, route);
      return { safe: true, accidentalBearer: first.accessToken };
    },
  });
  await service.syncNow();
  const socket = sockets[0]!;
  socket.open();
  await settle();
  socket.message({ type: "fdnPrivate", requestId: "missing-route", payload: { requestId: "request-valid-1", deviceId: "device-a" } });
  await settle();
  assert.equal(dispatches, 0, "a missing route must not execute even with a supplied dispatch adapter");
  assert.equal(socket.sent.find((item) => item.requestId === "missing-route")?.ok, false);
  socket.message({ type: "fdnPrivate", requestId: "private-success", networkRoute: route, payload: { requestId: "request-valid-2", deviceId: "device-a" } });
  await settle();
  assert.equal(dispatches, 1);
  const successful = socket.sent.find((item) => item.requestId === "private-success");
  assert.equal(successful?.ok, true);
  assert.equal(JSON.stringify(successful).includes(first.accessToken), false, "first-party bearer must never cross private result WSS");
  assert.equal((successful?.result as Record<string, unknown>)?.accidentalBearer, "[redacted]");
  current = null;
  socket.message({ type: "fdnPrivate", requestId: "revoked", networkRoute: route, payload: { requestId: "request-valid-3", deviceId: "device-a" } });
  await settle();
  assert.equal(dispatches, 1, "revoked account must not dispatch a private operation");
  assert.equal(socket.closed, true);
  await service.stop();
});

test("private dispatch never sends an in-flight result on a replaced account socket", async () => {
  let current: FabushiDeviceAgentSession | null = session("private-before");
  const sockets: FakeSocket[] = [];
  let dispatches = 0;
  let finish!: (value: unknown) => void;
  const pending = new Promise<unknown>((resolve) => { finish = resolve; });
  const service = new FabushiOfficialDeviceRegistryService({
    isPackaged: true, env: {}, userDataPath: "/private/fabushi",
    getDeviceAgentSession: async () => current,
    connect: () => { const socket = new FakeSocket(); sockets.push(socket); return socket; },
    privateTransportRequest: async () => { dispatches += 1; return await pending; },
  });
  await service.syncNow();
  const oldSocket = sockets[0]!;
  oldSocket.open();
  await settle();
  oldSocket.message({ type: "fdnPrivate", requestId: "late-private", networkRoute: {
    nodeId: "100.64.0.13", cellId: "cell-987654", generation: 4, expiresAt: Math.floor(Date.now() / 1_000) + 300,
  }, payload: { requestId: "request-valid-4", deviceId: "device-b" } });
  await settle();
  assert.equal(dispatches, 1);
  current = session("private-after");
  await service.syncNow();
  finish({ executed: true });
  await settle();
  assert.equal(oldSocket.closed, true);
  assert.equal(oldSocket.sent.some((item) => item.type === "fdnPrivateResult" && item.requestId === "late-private"), false);
  await service.stop();
});
