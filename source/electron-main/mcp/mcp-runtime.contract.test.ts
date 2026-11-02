import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { McpResult, McpSuccess } from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import {
  fetchAccountMcpServers,
  type AccountMcpClient,
  type AvailableMcpServer,
} from "../../shared/node/cursor-backend/account-mcp.js";
import { accountCacheScope } from "../../shared/node/cursor-token.js";
import { SandMcpManager } from "../../shared/node/mcp/mcp-manager.js";
import { createMcpToolsDiscovery } from "../../shared/node/mcp/tools-discovery.js";
import { SandSettingsStore } from "../../shared/node/settings/sand-settings-store.js";
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


test("shipping MCP runtime preserves connector-auth reports through the dedicated callback", async () => {
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
    reportConnectorAuth: (report) => forwarded.push(report),
    reportMcpDiscoveryFailed: () => {},
    reportDiagnostic: () => {},
    cleanupLegacyAuth: async () => {},
    sandRootDir: () => "/tmp/sand",
    reportFailure: () => {},
    broadcast: () => {},
    refreshHostMcp: () => {},
  });
  await runtime.ensureMcpManager();
  const report = {
    phase: "flow_started",
    outcome: "failed",
    serverName: "GitHub",
    serverId: "srv_1",
    reauth: true,
    error: { code: "SAND-E0203", reason: "admin_blocked" },
  };
  managerOptions.onConnectorAuth(report);
  assert.deepEqual(forwarded, [report]);
  assert.equal(forwarded[0], report, "runtime must relay the producer report without remapping it");
});

type AccountServerResponse = Awaited<ReturnType<AccountMcpClient["getAvailableMcpServers"]>>;
type Discovery = ReturnType<typeof createMcpToolsDiscovery>;

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

const ACCOUNT_A = "mcp-retirement-account-a";
const ACCOUNT_B = "mcp-retirement-account-b";
const DISABLED_TOOL = "dangerous_tool";
const ENABLED_TOOL = "safe_tool";

function accountServer(account: "a" | "b"): AvailableMcpServer {
  return {
    id: account === "a" ? 70n : 71n,
    name: "Retirement " + account,
    serverIdentifier: "retirement-" + account,
    type: "http",
    url: "https://connector.example.test/" + account,
    args: [],
    enabled: true,
    isTeamServer: false,
    disabledByTeamAdminPolicy: false,
    isRequired: false,
    managedByTeamPluginPolicy: false,
  };
}

const accountResponse = (account: "a" | "b"): AccountServerResponse => ({
  servers: [accountServer(account)],
});

function createAccountRetirementFixture(
  root: string,
  readServers: (token: string) => Promise<AccountServerResponse>,
) {
  const store = new SandSettingsStore(join(root, "settings.json"));
  const rpcTokens: string[] = [];
  const scopeApplications: Array<string | undefined> = [];
  const executions: Array<{ serverIdentifier: string; toolName: string }> = [];
  const discoveries = new Map<SandMcpManager, Discovery>();
  let currentToken = ACCOUNT_A;
  let scopeHook: (() => void) | undefined;
  let beforeToolList: (() => Promise<void>) | undefined;
  const unexpectedAccountRpc = async (): Promise<never> => {
    throw new Error("Unexpected account RPC in retirement control");
  };
  const backend = {
    async listTools(serverIdentifiers: string[]) {
      await beforeToolList?.();
      return serverIdentifiers.map((providerIdentifier) => ({
        serverIdentifier: providerIdentifier,
        rowServerIdentifier: providerIdentifier,
        accountLabel: "default",
        status: "connected",
        tools: [DISABLED_TOOL, ENABLED_TOOL].map((toolName) => ({
          providerIdentifier,
          name: toolName,
          toolName,
          inputSchema: { type: "object" },
        })),
      }));
    },
    async executeTool(request: { serverIdentifier: string; toolName: string }) {
      executions.push({ serverIdentifier: request.serverIdentifier, toolName: request.toolName });
      return new McpResult({
        result: { case: "success", value: new McpSuccess({ content: [] }) },
      });
    },
  };
  const runtime = createMcpRuntime<SandMcpManager>({
    createManager: async (options) => {
      const manager = new SandMcpManager({
        includeBuiltins: false,
        settingsStore: options.settingsStore,
        onAccountScopeApplied: options.onAccountScopeApplied,
        backendMcpExec: backend,
        accountServersProvider: () => fetchAccountMcpServers({
          getAccessToken: async () => {
            const token = await options.getAccessToken({ backendUrl: "https://account.example.test" });
            if (token == null) throw new Error("No account credential");
            return token;
          },
          getMachineId: async () => options.getMachineId(),
          getBackendUrl: () => "https://account.example.test",
          createClient: (credentials): AccountMcpClient => ({
            getAvailableMcpServers: async () => {
              const token = await credentials.getAccessToken();
              rpcTokens.push(token);
              return readServers(token);
            },
            getMcpConfig: unexpectedAccountRpc,
            getEffectiveUserPlugins: unexpectedAccountRpc,
            setMcpConfig: unexpectedAccountRpc,
            installUserPlugin: unexpectedAccountRpc,
            uninstallUserPlugin: unexpectedAccountRpc,
            updateUserPluginInstall: unexpectedAccountRpc,
          }),
        }),
      });
      const discovery = createMcpToolsDiscovery({
        definitionSource: manager.definitionSourceView(),
        lastAccountDisplayConfig: () => manager.lastAccountDisplayConfigView(),
        settingsStore: () => manager.settingsStoreView(),
        backendMcpExec: backend,
      });
      manager.setBoxRuntime(discovery);
      discoveries.set(manager, discovery);
      return manager;
    },
    settingsStore: store,
    pushBoxSecrets: async () => {
      scopeApplications.push(store.getMcpCustomInstructionsAccountScope());
      scopeHook?.();
    },
    ensureCursorAuthService: async () => ({
      getValidAccessToken: async () => currentToken,
    }),
    getMachineId: () => "retirement-machine",
    loadBoxMcpServers: unexpectedAccountRpc,
    listBoxMcpServers: unexpectedAccountRpc,
    listBoxMcpToolsRaw: unexpectedAccountRpc,
    executeBoxMcpToolRaw: unexpectedAccountRpc,
    reportConnectorAuth: () => {},
    reportMcpDiscoveryFailed: () => {},
    reportDiagnostic: () => {},
    cleanupLegacyAuth: async () => {},
    sandRootDir: () => root,
    reportFailure: () => {},
    broadcast: () => {},
    refreshHostMcp: () => {},
  });
  return {
    runtime,
    store,
    rpcTokens,
    scopeApplications,
    executions,
    selectAccount(token: string) { currentToken = token; },
    onScopeApplied(hook: () => void) { scopeHook = hook; },
    beforeToolList(hook: () => Promise<void>) { beforeToolList = hook; },
    discovery(manager: SandMcpManager): Discovery {
      const discovery = discoveries.get(manager);
      assert.ok(discovery);
      return discovery;
    },
  };
}

function executeFixtureTool(discovery: Discovery, toolName: string) {
  return discovery.executeTool(undefined, {
    providerIdentifier: "retirement-b",
    name: toolName,
    toolName,
    args: {},
    toolCallId: "retirement-" + toolName,
  }, { agentId: "retirement-agent" });
}

for (const outcome of ["success", "unavailable"] as const) {
  test("MCP account retirement preserves B permissions after late A " + outcome, { timeout: 10_000 }, async () => {
    const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-account-retirement-"));
    const started = deferred<void>();
    const held = deferred<AccountServerResponse>();
    const fixture = createAccountRetirementFixture(root, async (token) => {
      if (token === ACCOUNT_A) {
        started.resolve();
        return held.promise;
      }
      assert.equal(token, ACCOUNT_B);
      return accountResponse("b");
    });
    const pending: Promise<unknown>[] = [];
    try {
      const retired = await fixture.runtime.ensureMcpManager();
      await started.promise;
      const listing = retired.listServers();
      const resolution = retired.resolveDisplayServer("70");
      const tools = fixture.discovery(retired).getTools();
      pending.push(listing, resolution, tools);
      assert.deepEqual(fixture.rpcTokens, [ACCOUNT_A]);

      await fixture.runtime.resetMcpManager();
      await fixture.runtime.resetMcpManager();
      await retired.dispose();

      fixture.selectAccount(ACCOUNT_B);
      const current = await fixture.runtime.ensureMcpManager();
      assert.notEqual(current, retired);
      const live = await current.listServers();
      assert.equal(live.servers[0]?.id, "71");
      const discovery = fixture.discovery(current);
      assert.deepEqual((await discovery.getTools()).map((tool) => tool.toolName), [DISABLED_TOOL, ENABLED_TOOL]);
      await current.toggleMcpToolDisabled({ serverId: "71", toolName: DISABLED_TOOL });
      const denial = await executeFixtureTool(discovery, DISABLED_TOOL);
      assert.equal(denial.result.case, "error");
      assert.match((denial.result.value as { error: string }).error, /is disabled/);
      assert.deepEqual(fixture.executions, []);
      const scopeB = accountCacheScope(ACCOUNT_B);
      assert.equal(fixture.store.getMcpCustomInstructionsAccountScope(), scopeB);
      assert.deepEqual(fixture.store.getMcpDisabledToolsByServerId(), { "71": [DISABLED_TOOL] });
      const committedB = readFileSync(fixture.store.settingsPath, "utf8");

      if (outcome === "success") held.resolve(accountResponse("a"));
      else held.reject(new Error("Account A listing unavailable"));
      const [retiredListing, retiredResolution, retiredTools] = await Promise.all([listing, resolution, tools]);
      assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committedB);
      assert.deepEqual(retiredListing, { servers: [] });
      assert.equal(retiredResolution, undefined);
      assert.deepEqual(retiredTools, []);
      assert.equal(retired.lastAccountDisplayConfigView(), null);
      assert.deepEqual(await retired.definitionSourceView().getUserServerConfigs(), {});
      assert.deepEqual(await retired.listConnectedBackendTools(), []);
      const persisted = new SandSettingsStore(fixture.store.settingsPath);
      assert.equal(persisted.getMcpCustomInstructionsAccountScope(), scopeB);
      assert.deepEqual(persisted.getMcpDisabledToolsByServerId(), { "71": [DISABLED_TOOL] });
      assert.deepEqual(fixture.scopeApplications, [scopeB], "late A cannot invoke the scope-applied callback");
      assert.equal((await executeFixtureTool(discovery, DISABLED_TOOL)).result.case, "error");
      assert.deepEqual(fixture.executions, [], "the disabled tool must never reach execution I/O");
      assert.equal(fixture.rpcTokens.filter((token) => token === ACCOUNT_A).length, 1);

      // Check refusal only after releasing the held response, so removing a
      // retirement guard fails on observable state/I/O rather than hanging.
      const retiredFetches = fixture.rpcTokens.length;
      await retired.reload();
      retired.refreshAccountConfigInBackground();
      assert.deepEqual(await retired.reloadServers(), { servers: [] });
      assert.deepEqual(await retired.listServers(), { servers: [] });
      assert.equal(await retired.resolveDisplayServer("70"), undefined);
      assert.equal(await retired.resolveDisplayServer("70", { requireFreshRead: true }), undefined);
      await retired.definitionSourceView().ensureConfigLoaded();
      assert.equal(fixture.rpcTokens.length, retiredFetches, "retired resolution must not start account I/O");
      assert.deepEqual(fixture.scopeApplications, [scopeB]);

      // The replacement manager remains live: refresh preserves its denial and
      // an enabled tool still reaches the actual discovery execution boundary.
      const refreshed = await current.reloadServers();
      assert.equal(refreshed.servers[0]?.id, "71");
      assert.equal(refreshed.servers[0]?.disabledToolCount, 1);
      assert.deepEqual((await discovery.getTools()).map((tool) => tool.toolName), [ENABLED_TOOL]);
      assert.equal((await executeFixtureTool(discovery, ENABLED_TOOL)).result.case, "success");
      assert.deepEqual(fixture.executions, [{ serverIdentifier: "retirement-b", toolName: ENABLED_TOOL }]);
      assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committedB);
    } finally {
      held.resolve(accountResponse("a"));
      await Promise.allSettled(pending);
      await fixture.runtime.resetMcpManager();
      rmSync(root, { recursive: true, force: true });
    }
  });
}

test("MCP account retirement drops an established cache and a held fresh resolution", { timeout: 10_000 }, async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-account-cache-"));
  const started = deferred<void>();
  const held = deferred<AccountServerResponse>();
  let holdFresh = false;
  const fixture = createAccountRetirementFixture(root, async (token) => {
    assert.equal(token, ACCOUNT_A);
    if (holdFresh) {
      started.resolve();
      return held.promise;
    }
    return accountResponse("a");
  });
  let pending: Promise<unknown> | undefined;
  try {
    const manager = await fixture.runtime.ensureMcpManager();
    assert.equal((await manager.listServers()).servers[0]?.id, "70");
    assert.equal((await manager.resolveDisplayServer("70"))?.id, "70");
    assert.equal((await fixture.discovery(manager).getTools()).length, 2);
    holdFresh = true;
    pending = manager.resolveDisplayServer("70", { requireFreshRead: true });
    await started.promise;
    await fixture.runtime.resetMcpManager();
    await manager.dispose();
    assert.equal(manager.lastAccountDisplayConfigView(), null);
    assert.deepEqual(await manager.definitionSourceView().getUserServerConfigs(), {});
    held.resolve(accountResponse("a"));
    assert.equal(await pending, undefined);
    assert.deepEqual(await manager.listConnectedBackendTools(), []);
    assert.deepEqual(await fixture.discovery(manager).getTools(), []);
    const retiredFetches = fixture.rpcTokens.length;
    await manager.reload();
    manager.refreshAccountConfigInBackground();
    assert.equal(await manager.resolveDisplayServer("70", { requireFreshRead: true }), undefined);
    assert.equal(fixture.rpcTokens.length, retiredFetches);
  } finally {
    held.resolve(accountResponse("a"));
    await Promise.allSettled(pending == null ? [] : [pending]);
    await fixture.runtime.resetMcpManager();
    rmSync(root, { recursive: true, force: true });
  }
});

test("MCP account retirement fences a held backend listing before tool-cache publication", { timeout: 10_000 }, async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-list-retirement-"));
  const started = deferred<void>();
  const held = deferred<void>();
  const fixture = createAccountRetirementFixture(root, async () => accountResponse("a"));
  fixture.beforeToolList(async () => {
    started.resolve();
    await held.promise;
  });
  let pending: Promise<unknown> | undefined;
  try {
    const manager = await fixture.runtime.ensureMcpManager();
    pending = manager.listServers();
    await started.promise;
    await fixture.runtime.resetMcpManager();
    held.resolve();
    assert.deepEqual(await pending, { servers: [] });
    assert.equal(manager.lastAccountDisplayConfigView(), null);
    assert.deepEqual(await manager.listConnectedBackendTools(), []);
  } finally {
    held.resolve();
    await Promise.allSettled(pending == null ? [] : [pending]);
    await fixture.runtime.resetMcpManager();
    rmSync(root, { recursive: true, force: true });
  }
});

test("MCP account retirement during the scope callback cannot restore account projections", { timeout: 10_000 }, async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-scope-retirement-"));
  const held = deferred<AccountServerResponse>();
  const fixture = createAccountRetirementFixture(root, async () => held.promise);
  let pending: Promise<unknown> | undefined;
  try {
    const manager = await fixture.runtime.ensureMcpManager();
    fixture.onScopeApplied(() => { void manager.dispose(); });
    pending = manager.listServers();
    held.resolve(accountResponse("a"));
    assert.deepEqual(await pending, { servers: [] });
    assert.equal(manager.lastAccountDisplayConfigView(), null);
    assert.deepEqual(await manager.definitionSourceView().getUserServerConfigs(), {});
    const retiredFetches = fixture.rpcTokens.length;
    await manager.reload();
    manager.refreshAccountConfigInBackground();
    assert.deepEqual(await manager.listServers(), { servers: [] });
    assert.equal(fixture.rpcTokens.length, retiredFetches);
    assert.equal(fixture.scopeApplications.length, 1);
  } finally {
    held.resolve(accountResponse("a"));
    await Promise.allSettled(pending == null ? [] : [pending]);
    await fixture.runtime.resetMcpManager();
    rmSync(root, { recursive: true, force: true });
  }
});
