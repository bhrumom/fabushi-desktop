import assert from "node:assert/strict";
import test from "node:test";

import {
  MCP_TOOLS_CACHE_TTL_MS,
  createMcpToolsDiscovery,
} from "./tools-discovery.js";

type Tool = {
  providerIdentifier: string;
  name: string;
  toolName: string;
};

function coreFor(
  listTools: (serverIdentifiers: string[]) => Promise<Array<{ serverIdentifier: string; tools: Tool[] }>>,
) {
  return {
    definitionSource: {
      peekHttpServerNames: () => ["server-a"],
      peekStdioServerNames: () => [],
      getUserServerConfigs: async () => ({
        "server-a": { url: "https://mcp.example" },
      }),
      getStdioServerConfigs: async () => ({}),
    },
    lastAccountDisplayConfig: () => ({ servers: [] }),
    settingsStore: () => ({
      getMcpDisabledToolsByServerId: () => ({}),
    }),
    backendMcpExec: { listTools },
  };
}

const deadline = {
  run<T>(operation: () => Promise<T>): Promise<T> {
    return operation();
  },
};

const tool: Tool = {
  providerIdentifier: "server-a",
  name: "tool-a",
  toolName: "tool-a",
};

test("successful MCP discovery returns tools without failure telemetry", async () => {
  const reports: unknown[] = [];
  const discovery = createMcpToolsDiscovery(
    coreFor(async () => [{ serverIdentifier: "server-a", tools: [tool] }]),
    {
      deadline: deadline as any,
      onDiscoveryFailed: (report) => reports.push(report),
    },
  );
  assert.deepEqual(await discovery.getToolsRaw(), [tool]);
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(reports, []);
});

test("cold MCP discovery failure reports elapsed time and does not claim stale service", async () => {
  const originalNow = Date.now;
  let now = 10_000;
  Date.now = () => now;
  try {
    const reports: Array<Record<string, unknown>> = [];
    let rejectList: ((error: unknown) => void) | undefined;
    const discovery = createMcpToolsDiscovery(
      coreFor(
        () =>
          new Promise((_resolve, reject) => {
            rejectList = reject;
          }),
      ),
      {
        deadline: deadline as any,
        onDiscoveryFailed: (report) => reports.push(report),
      },
    );
    const pending = discovery.getToolsRaw();
    while (rejectList == null) await new Promise<void>((resolve) => setImmediate(resolve));
    now += 37;
    const failure = new Error("deadline");
    failure.name = "TimeoutError";
    rejectList(failure);
    await assert.rejects(pending, /deadline/);
    await new Promise<void>((resolve) => setImmediate(resolve));
    assert.deepEqual(reports, [{
      errorClass: "TimeoutError",
      elapsedMs: 37,
      servedStale: false,
    }]);
  } finally {
    Date.now = originalNow;
  }
});

test("expired MCP cache serves stale tools while failed refresh reports servedStale and elapsed", async () => {
  const originalNow = Date.now;
  let now = 20_000;
  Date.now = () => now;
  try {
    const reports: Array<Record<string, unknown>> = [];
    let calls = 0;
    let rejectRefresh: ((error: unknown) => void) | undefined;
    const discovery = createMcpToolsDiscovery(
      coreFor(async () => {
        calls += 1;
        if (calls === 1) return [{ serverIdentifier: "server-a", tools: [tool] }];
        return await new Promise((_resolve, reject) => {
          rejectRefresh = reject;
        });
      }),
      {
        deadline: deadline as any,
        onDiscoveryFailed: (report) => reports.push(report),
      },
    );
    assert.deepEqual(await discovery.getToolsRaw(), [tool]);
    now += MCP_TOOLS_CACHE_TTL_MS + 1;
    assert.deepEqual(await discovery.getToolsRaw(), [tool]);
    while (rejectRefresh == null) await new Promise<void>((resolve) => setImmediate(resolve));
    now += 29;
    const failure = new Error("offline");
    failure.name = "NetworkError";
    rejectRefresh(failure);
    await new Promise<void>((resolve) => setImmediate(resolve));
    assert.deepEqual(reports, [{
      errorClass: "NetworkError",
      elapsedMs: 29,
      servedStale: true,
    }]);
    assert.deepEqual(await discovery.getToolsRaw(), [tool]);
  } finally {
    Date.now = originalNow;
  }
});
