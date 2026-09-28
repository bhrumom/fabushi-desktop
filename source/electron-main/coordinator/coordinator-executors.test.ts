import assert from "node:assert/strict";
import test from "node:test";

import { createCoordinatorControlExecutors } from "./coordinator-executors.js";

const EXPECTED_CONTROL_METHODS = [
  "resolveGatewayConnection",
  "listRoutedMcpTools",
  "executeRoutedMcpTool",
  "listHostMcpServers",
  "listHostMcpCatalog",
  "listHostEffectiveMcpPlugins",
  "installHostMcpPlugin",
  "uninstallHostMcpPlugin",
  "noteHostMcpAuthCompleted",
  "mintLocalExecDaemonCredential",
  "requestWebAuthnConsent",
  "requestWebAuthnPin",
  "updateWebAuthnConsent",
  "finishWebAuthnConsent",
  "spawnLocalExecDaemon",
  "terminateProcess",
  "isProcessAlive",
  "getProcessIdentity",
  "waitLocalExecDaemonExit",
  "getRpcTraceWindowTraceparent",
  "reportTransportStage",
  "reportGatewayCommandSpan",
  "reportGatewayReachability",
  "reportGatewayDnsDiagnostic",
  "reportProcessCrash",
] as const;

function makeDependencies(calls: string[]) {
  return {
    connector: {
      async connect() {
        calls.push("gateway");
        return { baseUrl: "https://127.0.0.1:43123" };
      },
      async issueLocalExecDaemonCredential() {
        calls.push("credential");
        return { token: "secret" };
      },
    },
    webauthnPrompt: {
      async requestConsent() {
        calls.push("consent");
        return { approved: true as const, promptId: "prompt-1", windowHandle: undefined };
      },
      async requestPin() {
        calls.push("pin");
        return { pin: "1234" };
      },
      update(status: string) {
        calls.push(`status:${status}`);
      },
      finish() {
        calls.push("finish");
      },
    },
    recordSendStage(report: { readonly name: string }) {
      calls.push(`stage:${report.name}`);
    },
    recordGatewayCommandSpan() {
      calls.push("command-span");
    },
    onReachability() {
      calls.push("reachability");
    },
    onDnsDiagnostic() {
      calls.push("dns");
    },
    onProcessCrash() {
      calls.push("crash");
    },
    getRpcTraceWindowTraceparent: () => "00-trace-span-01",
    async listRoutedMcpTools() {
      calls.push("mcp-list");
      return [{ name: "tool" }];
    },
    async executeRoutedMcpTool(request: unknown) {
      calls.push("mcp-call");
      return { request };
    },
    async listHostMcpServers() {
      calls.push("mcp-host-servers");
      return { servers: [{ id: "7" }] };
    },
    async listHostMcpCatalog(options: { readonly forceRefresh?: boolean }) {
      calls.push(`mcp-host-catalog:${options.forceRefresh === true}`);
      return [{ id: "plugin-7" }];
    },
    async listHostEffectiveMcpPlugins() {
      calls.push("mcp-host-effective");
      return [{ pluginId: "7" }];
    },
    async installHostMcpPlugin(request: unknown) {
      calls.push("mcp-host-install");
      return { request };
    },
    async uninstallHostMcpPlugin(pluginId: string) {
      calls.push(`mcp-host-uninstall:${pluginId}`);
      return { removed: true };
    },
    async noteHostMcpAuthCompleted(request: { readonly serverId: string; readonly accountKey: string }) {
      calls.push(`mcp-host-auth:${request.serverId}:${request.accountKey}`);
      return { noted: true };
    },
  };
}

test("control executor exposes the frozen Grok platform command surface", () => {
  const calls: string[] = [];
  const executors = createCoordinatorControlExecutors(makeDependencies(calls));
  assert.deepEqual(Object.keys(executors), EXPECTED_CONTROL_METHODS);
});

test("control executor routes gateway, MCP, WebAuthn, and telemetry without absorbing Host behavior", async () => {
  const calls: string[] = [];
  const executors = createCoordinatorControlExecutors(makeDependencies(calls));

  assert.deepEqual(await executors.resolveGatewayConnection(), {
    baseUrl: "https://127.0.0.1:43123",
  });
  assert.deepEqual(await executors.mintLocalExecDaemonCredential(), { token: "secret" });
  assert.deepEqual(await executors.listRoutedMcpTools(), [{ name: "tool" }]);
  assert.deepEqual(await executors.executeRoutedMcpTool({ id: 7 }), {
    request: { id: 7 },
  });
  assert.deepEqual(await executors.listHostMcpServers(), { servers: [{ id: "7" }] });
  assert.deepEqual(await executors.listHostMcpCatalog({ forceRefresh: true }), [{ id: "plugin-7" }]);
  assert.deepEqual(await executors.listHostEffectiveMcpPlugins(), [{ pluginId: "7" }]);
  assert.deepEqual(await executors.installHostMcpPlugin({ entryId: "7" }), { request: { entryId: "7" } });
  assert.deepEqual(await executors.uninstallHostMcpPlugin({ pluginId: "7" }), { removed: true });
  assert.deepEqual(await executors.noteHostMcpAuthCompleted({ serverId: "server-7", accountKey: "default" }), { noted: true });
  assert.deepEqual(await executors.requestWebAuthnConsent({
    origin: "https://example.test",
    rpId: "example.test",
  }), {
    approved: true,
    promptId: "prompt-1",
    windowHandle: undefined,
  });
  assert.deepEqual(await executors.requestWebAuthnPin({
    promptId: "prompt-1",
    invalid: false,
  }), { pin: "1234" });

  executors.updateWebAuthnConsent({ status: "Touch your security key now" });
  executors.finishWebAuthnConsent();
  assert.equal(executors.getRpcTraceWindowTraceparent(), "00-trace-span-01");
  executors.reportTransportStage({
    stage: "gateway.dispatch",
    traceparent: "00-trace-span-01",
    clientNonce: "nonce",
    startEpochMs: 10,
    durationMs: 5,
    attempt: 1,
    isError: false,
  });
  executors.reportGatewayCommandSpan({});
  executors.reportGatewayReachability({});
  executors.reportGatewayDnsDiagnostic({});
  executors.reportProcessCrash({ kind: "panic" });

  assert.deepEqual(calls, [
    "gateway",
    "credential",
    "mcp-list",
    "mcp-call",
    "mcp-host-servers",
    "mcp-host-catalog:true",
    "mcp-host-effective",
    "mcp-host-install",
    "mcp-host-uninstall:7",
    "mcp-host-auth:server-7:default",
    "consent",
    "pin",
    "status:Touch your security key now",
    "finish",
    "stage:gateway.dispatch",
    "command-span",
    "reachability",
    "dns",
    "crash",
  ]);
});

test("MCP control methods fail closed when routing is not configured", async () => {
  const calls: string[] = [];
  const deps = makeDependencies(calls);
  const executors = createCoordinatorControlExecutors({
    connector: deps.connector,
    webauthnPrompt: deps.webauthnPrompt,
    recordSendStage: deps.recordSendStage,
    recordGatewayCommandSpan: deps.recordGatewayCommandSpan,
    onReachability: deps.onReachability,
    onDnsDiagnostic: deps.onDnsDiagnostic,
    onProcessCrash: deps.onProcessCrash,
  });
  await assert.rejects(executors.listRoutedMcpTools(), /Desktop MCP routing is unavailable/);
  await assert.rejects(
    executors.executeRoutedMcpTool({}),
    /Desktop MCP routing is unavailable/,
  );
  await assert.rejects(executors.listHostMcpServers(), /Desktop MCP lifecycle routing is unavailable/);
  await assert.rejects(executors.listHostMcpCatalog({}), /Desktop MCP lifecycle routing is unavailable/);
  await assert.rejects(executors.listHostEffectiveMcpPlugins(), /Desktop MCP lifecycle routing is unavailable/);
  await assert.rejects(executors.installHostMcpPlugin({}), /Desktop MCP lifecycle routing is unavailable/);
  await assert.rejects(executors.uninstallHostMcpPlugin({ pluginId: "7" }), /Desktop MCP lifecycle routing is unavailable/);
  await assert.rejects(executors.noteHostMcpAuthCompleted({ serverId: "7", accountKey: "default" }), /Desktop MCP lifecycle routing is unavailable/);
});
