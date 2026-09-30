import assert from "node:assert/strict";
import test from "node:test";

import { createMcpRuntime } from "./mcp-runtime.js";

test("shipping MCP runtime forwards discovery failure through the dedicated callback", async () => {
  let managerOptions: any;
  const forwarded: unknown[] = [];
  const manager = {
    dispose() {},
    setAuthCompletionObserver() {},
  };
  const runtime = createMcpRuntime({
    createManager: async (options) => {
      managerOptions = options;
      return manager;
    },
    settingsStore: {},
    pushBoxSecrets: async () => {},
    ensureCursorAuthService: async () => ({
      getValidAccessToken: async () => "token",
    }),
    getMachineId: () => "machine",
    loadBoxMcpServers: async () => ({}),
    listBoxMcpServers: async () => [],
    listBoxMcpToolsRaw: async () => "00",
    executeBoxMcpToolRaw: async () => "00",
    reportConnectorAuth: () => {},
    reportMcpDiscoveryFailed: (report) => forwarded.push(report),
    reportDiagnostic: () => {},
    cleanupLegacyAuth: async () => {},
    sandRootDir: () => "/tmp/sand",
    reportFailure: () => {},
    broadcast: () => {},
    refreshHostMcp: () => {},
  });
  await runtime.ensureMcpManager();
  const report = {
    errorClass: "TimeoutError",
    elapsedMs: 19.4,
    servedStale: true,
  };
  managerOptions.onMcpDiscoveryFailed(report);
  assert.deepEqual(forwarded, [report]);
});
