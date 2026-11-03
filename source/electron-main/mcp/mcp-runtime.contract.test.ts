import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import test, { type TestContext } from "node:test";

import {
  McpResult,
  McpSuccess,
  McpTextContent,
  McpToolResultContentItem,
} from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import {
  fetchAccountMcpServers,
  type AccountMcpClient,
  type AvailableMcpServer,
} from "../../shared/node/cursor-backend/account-mcp.js";
import type { createDashboardSandBackendMcpExec } from "../../shared/node/cursor-backend/backend-mcp-exec.js";
import { accountCacheScope } from "../../shared/node/cursor-token.js";
import { SandMcpManager } from "../../shared/node/mcp/mcp-manager.js";
import { createMcpToolsDiscovery } from "../../shared/node/mcp/tools-discovery.js";
import { SandSettingsStore } from "../../shared/node/settings/sand-settings-store.js";
import { createProductionMcpOAuthAdapter, createProductionMcpOAuthPorts, createProductionMcpOAuthRootPortProvider } from "../adapters/mcp-oauth.js";
import type { ProductionServiceContext } from "../main-production-services.js";
import type { DesktopMcpManagerFacade } from "./desktop-mcp-manager.js";
import { createMcpRuntime } from "./mcp-runtime.js";
import { OfficialMcpService } from "./official-mcp-service.js";
import { SandUserSecretsStore } from "../secrets/user-secrets-store.js";
import { createCoordinatorControlServer } from "../coordinator/coordinator-control-server.js";
import {
  createCoordinatorAccountRuntime,
  type CoordinatorAccountBinding,
  type CoordinatorAuthStatus,
} from "../coordinator/coordinator-account-runtime.js";
import {
  parseMcpRoutedOrigin,
  type CoordinatorControlSourceContext,
  type McpRoutedOrigin,
} from "../../shared/rpc/coordinator-port.js";
import {
  generatedMcpResultFactory,
  MCP_ORIGIN_RETIRED,
  McpOriginRetiredError,
  type McpDispatchAssertion,
} from "../../shared/node/mcp/mcp-result-factory.js";

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
type BackendToolRequest = Parameters<ReturnType<typeof createDashboardSandBackendMcpExec>["executeTool"]>[0];
type RoutedToolRequest = Parameters<DesktopMcpManagerFacade["executeRoutedTool"]>[0];

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

// Use the concrete producer's context; wire arguments cannot mint this lease.
function nativeRoutedSource(t: TestContext) {
  let source: CoordinatorControlSourceContext | undefined;
  const server = createCoordinatorControlServer({
    post() {},
    executors: { capture: (_args, current) => { source = current; } },
    onEvent: {},
    onProblem: (detail) => assert.fail(detail),
  });
  t.after(() => server.dispose());
  server.handleMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
  server.handleMessage({ kind: "request", requestId: "capture", method: "capture", args: null });
  assert.ok(source);
  return { source, server };
}

function catalogOrigin(catalog: unknown): McpRoutedOrigin {
  assert.ok(Array.isArray(catalog));
  const origin = parseMcpRoutedOrigin(catalog[0]?.origin);
  assert.ok(origin, "the production adapter must stamp its original native source and Attempt");
  return origin;
}

function assertOriginRejected(value: unknown): void {
  assert.ok(value instanceof McpResult, "pre-dispatch refusal uses the actual generated result");
  assert.deepEqual(value.result, { case: "rejected", value: generatedMcpResultFactory.rejected(MCP_ORIGIN_RETIRED).result.value });
}

const constructionTool = { providerIdentifier: "connector", name: "tool", toolName: "tool" };

type ConstructionManager = Pick<DesktopMcpManagerFacade,
  "dispose" | "setAuthCompletionObserver" | "listRoutedTools" | "executeRoutedTool">;
type ConstructionDeps = Parameters<typeof createMcpRuntime<ConstructionManager>>[0];
type ConstructionOptions = Parameters<ConstructionDeps["createManager"]>[0];

function constructionDeps(
  createManager: ConstructionDeps["createManager"],
  overrides: Partial<Omit<ConstructionDeps, "createManager">> = {},
): ConstructionDeps {
  return {
    createManager,
    settingsStore: {},
    pushBoxSecrets: async () => {},
    ensureCursorAuthService: async () => ({ getValidAccessToken: async () => "current-token" }),
    getMachineId: () => "construction-machine",
    loadBoxMcpServers: async () => ({}),
    listBoxMcpServers: async () => [],
    listBoxMcpToolsRaw: async () => "00",
    executeBoxMcpToolRaw: async () => "00",
    reportConnectorAuth: () => {},
    reportMcpDiscoveryFailed: () => {},
    reportDiagnostic: () => {},
    cleanupLegacyAuth: async () => {},
    sandRootDir: () => "/tmp/mcp-construction",
    reportFailure: () => {},
    broadcast: () => {},
    refreshHostMcp: () => {},
    ...overrides,
  };
}

function constructionManager(hooks: {
  dispose?: () => void | Promise<void>;
  observe?: (listener: (completion: unknown) => void) => void;
} = {}) {
  const calls = { dispose: 0, observe: 0, list: 0, execute: 0 };
  let observer: ((completion: unknown) => void) | undefined;
  const manager: ConstructionManager = {
    dispose() {
      calls.dispose += 1;
      return hooks.dispose?.();
    },
    setAuthCompletionObserver(listener) {
      calls.observe += 1;
      observer = listener;
      hooks.observe?.(listener);
    },
    async listRoutedTools() { calls.list += 1; return [constructionTool]; },
    async executeRoutedTool() { calls.execute += 1; return "executed"; },
  };
  return { manager, calls, complete(completion: unknown) { observer?.(completion); } };
}

test("MC-01 concurrent ensures share the reserved construction and manager", async () => {
  const started = deferred<void>();
  const held = deferred<ConstructionManager>();
  const owned = constructionManager();
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(() => {
    factories += 1;
    started.resolve();
    return held.promise;
  }));
  const first = runtime.ensureMcpManager();
  const second = runtime.ensureMcpManager();
  assert.equal(first, second);
  await started.promise;
  assert.equal(factories, 1);
  held.resolve(owned.manager);
  assert.deepEqual(await Promise.all([first, second]), [owned.manager, owned.manager]);
  assert.equal(await runtime.ensureMcpManager(), owned.manager);
  assert.equal(owned.calls.observe, 1);
  await runtime.dispose();
  assert.equal(owned.calls.dispose, 1);
});

test("MC-01 a synchronous factory reentry sees the already reserved promise", async () => {
  const owned = constructionManager();
  let factories = 0;
  let reentrant: Promise<ConstructionManager> | undefined;
  const runtime = createMcpRuntime(constructionDeps(() => {
    factories += 1;
    reentrant = runtime.ensureMcpManager();
    return Promise.resolve(owned.manager);
  }));
  const first = runtime.ensureMcpManager();
  assert.equal(await first, owned.manager);
  assert.equal(reentrant, first);
  assert.equal(factories, 1);
  await runtime.dispose();
});

for (const action of ["reset", "dispose"] as const) {
  test("MC-01/03 " + action + " before invocation refuses the old factory", async () => {
    const owned = constructionManager();
    let factories = 0;
    const runtime = createMcpRuntime(constructionDeps(async () => {
      factories += 1;
      return owned.manager;
    }));
    const pending = runtime.ensureMcpManager();
    const rejected = assert.rejects(pending, /construction is retired/);
    await (action === "reset" ? runtime.resetMcpManager() : runtime.dispose());
    await rejected;
    assert.equal(factories, 0);
    if (action === "reset") {
      assert.equal(await runtime.ensureMcpManager(), owned.manager);
      assert.equal(factories, 1);
    } else {
      await runtime.resetMcpManager();
      await assert.rejects(runtime.ensureMcpManager(), /runtime is disposed/);
      assert.equal(factories, 0);
    }
    await runtime.dispose();
  });
}

for (const outcome of ["success", "failure"] as const) {
  test("MC-02 late old " + outcome + " cannot adopt or erase replacement B", async () => {
    const started = deferred<void>();
    const held = deferred<ConstructionManager>();
    const old = constructionManager();
    const replacement = constructionManager();
    const failure = new Error("Old factory failed");
    let factories = 0;
    const runtime = createMcpRuntime(constructionDeps(() => {
      factories += 1;
      if (factories > 1) return Promise.resolve(replacement.manager);
      started.resolve();
      return held.promise;
    }));
    const pending = runtime.ensureMcpManager();
    const rejected = assert.rejects(pending, outcome === "success" ? /construction is retired/ : (error) => error === failure);
    await started.promise;
    await runtime.resetMcpManager();
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    if (outcome === "success") held.resolve(old.manager);
    else held.reject(failure);
    await rejected;
    assert.equal(old.calls.dispose, outcome === "success" ? 1 : 0);
    assert.equal(old.calls.observe, 0);
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    assert.equal(replacement.calls.dispose, 0);
    assert.equal(factories, 2, "the old operation must not retry against B");
    await runtime.dispose();
  });
}

for (const failureMode of ["throw", "reject"] as const) {
  test("MC-02 current factory " + failureMode + " retires callbacks and allows a fresh retry", async () => {
    const replacement = constructionManager();
    const failure = new Error("Current factory failed");
    let oldOptions: ConstructionOptions | undefined;
    let reports = 0;
    let factories = 0;
    const runtime = createMcpRuntime(constructionDeps((options) => {
      factories += 1;
      if (factories > 1) return Promise.resolve(replacement.manager);
      oldOptions = options;
      options.onMcpDiagnostic({ leg: "construct", errorClass: "live" });
      if (failureMode === "throw") throw failure;
      return Promise.reject(failure);
    }, { reportDiagnostic: () => { reports += 1; } }));
    await assert.rejects(runtime.ensureMcpManager(), (error) => error === failure);
    assert.ok(oldOptions);
    assert.equal(reports, 1);
    oldOptions.onMcpDiagnostic({ leg: "construct", errorClass: "stale" });
    assert.equal(reports, 1);
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    oldOptions.onMcpDiagnostic({ leg: "construct", errorClass: "stale-after-B" });
    assert.equal(reports, 1);
    assert.equal(factories, 2);
    await runtime.dispose();
  });
}

test("MC-03 disposal returns while a factory is pending and owns its late cleanup", async () => {
  const started = deferred<void>();
  const held = deferred<ConstructionManager>();
  const old = constructionManager();
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(() => {
    factories += 1;
    started.resolve();
    return held.promise;
  }));
  const rejected = assert.rejects(runtime.ensureMcpManager(), /construction is retired/);
  await started.promise;
  await runtime.dispose();
  await runtime.dispose();
  await runtime.resetMcpManager();
  await assert.rejects(runtime.ensureMcpManager(), /runtime is disposed/);
  assert.equal(factories, 1);
  assert.equal(old.calls.dispose, 0);
  held.resolve(old.manager);
  await rejected;
  await runtime.dispose();
  assert.equal(old.calls.dispose, 1);
  assert.equal(old.calls.observe, 0);
});

test("MC-03 disposal shares observable cleanup before synchronous reentry", async () => {
  const cleanup = deferred<void>();
  let reentrant: Promise<void> | undefined;
  const owned = constructionManager({ dispose: () => {
    reentrant = runtime.dispose();
    return cleanup.promise;
  } });
  const runtime = createMcpRuntime(constructionDeps(async () => owned.manager));
  await runtime.ensureMcpManager();
  let settled = false;
  const first = runtime.dispose();
  void first.then(() => { settled = true; });
  const second = runtime.dispose();
  assert.equal(first, second);
  assert.equal(first, reentrant);
  assert.equal(owned.calls.dispose, 1);
  await assert.rejects(runtime.ensureMcpManager(), /runtime is disposed/);
  assert.equal(settled, false);
  cleanup.resolve();
  await first;
  assert.equal(settled, true);
});

test("MC-02 reset detaches before a disposer reentrantly creates B", async () => {
  const cleanup = deferred<void>();
  const replacement = constructionManager();
  let replacementPromise: Promise<ConstructionManager> | undefined;
  const old = constructionManager({ dispose: () => {
    replacementPromise = runtime.ensureMcpManager();
    return cleanup.promise;
  } });
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager));
  await runtime.ensureMcpManager();
  const reset = runtime.resetMcpManager();
  assert.ok(replacementPromise);
  assert.equal(await replacementPromise, replacement.manager);
  cleanup.resolve();
  await reset;
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  assert.equal(old.calls.dispose, 1);
  assert.equal(replacement.calls.dispose, 0);
  await runtime.dispose();
});

for (const cleanupMode of ["throw", "reject"] as const) {
  test("MC-02/03 reset observes cleanup " + cleanupMode + " without erasing B", async () => {
    const failure = new Error("Manager cleanup failed");
    const cleanup = deferred<void>();
    const old = constructionManager({ dispose: () => {
      if (cleanupMode === "throw") throw failure;
      return cleanup.promise;
    } });
    const replacement = constructionManager();
    let factories = 0;
    const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager));
    await runtime.ensureMcpManager();
    const rejected = assert.rejects(runtime.resetMcpManager(), (error) => error === failure);
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    if (cleanupMode === "reject") cleanup.reject(failure);
    await rejected;
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    assert.equal(old.calls.dispose, 1);
    assert.equal(replacement.calls.dispose, 0);
    await runtime.dispose();
  });
}

test("MC-02/03 late retired-manager cleanup failure remains in the old rejection", async () => {
  const started = deferred<void>();
  const held = deferred<ConstructionManager>();
  const failure = new Error("Late cleanup failed");
  const old = constructionManager({ dispose: async () => { throw failure; } });
  const replacement = constructionManager();
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(() => {
    if (++factories > 1) return Promise.resolve(replacement.manager);
    started.resolve();
    return held.promise;
  }));
  const rejected = assert.rejects(runtime.ensureMcpManager(), (error) => {
    assert.ok(error instanceof AggregateError);
    assert.match(error.errors[0].message, /construction is retired/);
    assert.equal(error.errors[1], failure);
    return true;
  });
  await started.promise;
  await runtime.resetMcpManager();
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  held.resolve(old.manager);
  await rejected;
  assert.equal(old.calls.dispose, 1);
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  await runtime.dispose();
});

for (const setup of ["root", "cleanup", "observer"] as const) {
  test("MC-04 reentrant " + setup + " setup cannot publish a retired manager", async () => {
    let replacementPromise: Promise<ConstructionManager> | undefined;
    let resetPromise: Promise<void> | undefined;
    let resetOnce = true;
    const reset = () => {
      if (!resetOnce) return;
      resetOnce = false;
      resetPromise = runtime.resetMcpManager();
      replacementPromise = runtime.ensureMcpManager();
    };
    const old = constructionManager({ observe: () => { if (setup === "observer") reset(); } });
    const replacement = constructionManager();
    let factories = 0;
    let scrubs = 0;
    const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager, {
      sandRootDir: () => {
        if (setup === "root") reset();
        return "/tmp/mcp-construction";
      },
      cleanupLegacyAuth: async () => {
        scrubs += 1;
        if (setup === "cleanup") reset();
      },
    }));
    await assert.rejects(runtime.ensureMcpManager(), /construction is retired/);
    await resetPromise;
    assert.ok(replacementPromise);
    assert.equal(await replacementPromise, replacement.manager);
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    assert.equal(old.calls.dispose, 1);
    assert.equal(old.calls.observe, setup === "observer" ? 1 : 0);
    assert.equal(scrubs, setup === "root" ? 1 : 2);
    assert.equal(replacement.calls.observe, 1);
    assert.equal(replacement.calls.dispose, 0);
    await runtime.dispose();
  });
}

test("MC-02 reset between setup and promise publication rejects the original waiter", async () => {
  const installed = deferred<void>();
  const old = constructionManager({ observe: () => { installed.resolve(); } });
  const replacement = constructionManager();
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager));
  const rejected = assert.rejects(runtime.ensureMcpManager(), /construction is retired/);
  // This waiter is released by setup before the construction continuation can
  // publish its facade. It is a causal promise boundary, not a timed delay.
  await installed.promise;
  await runtime.resetMcpManager();
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  await rejected;
  assert.equal(old.calls.dispose, 1);
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  await runtime.dispose();
});

test("MC-02/04 setup failure retires callbacks, retains cleanup errors and permits retry", async () => {
  const failure = new Error("Observer setup failed");
  const cleanupFailure = new Error("Failed setup cleanup failed");
  const old = constructionManager({
    observe: () => { throw failure; },
    dispose: () => { throw cleanupFailure; },
  });
  const replacement = constructionManager();
  let oldOptions: ConstructionOptions | undefined;
  let factories = 0;
  let reports = 0;
  const runtime = createMcpRuntime(constructionDeps(async (options) => {
    if (++factories > 1) return replacement.manager;
    oldOptions = options;
    return old.manager;
  }, { reportDiagnostic: () => { reports += 1; } }));
  await assert.rejects(runtime.ensureMcpManager(), (error) => {
    assert.ok(error instanceof AggregateError);
    assert.deepEqual(error.errors, [failure, cleanupFailure]);
    return true;
  });
  assert.equal(old.calls.dispose, 1);
  assert.ok(oldOptions);
  oldOptions.onMcpDiagnostic({ leg: "setup", errorClass: "retired" });
  assert.equal(reports, 0);
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  assert.equal(factories, 2);
  await runtime.dispose();
});

for (const trigger of ["installation", "completion"] as const) {
  test("MC-04 auth broadcast rechecks retirement during " + trigger, async () => {
    const broadcasts: unknown[] = [];
    const refreshes: unknown[] = [];
    let resetPromise: Promise<void> | undefined;
    const old = constructionManager({ observe: (listener) => {
      if (trigger === "installation") listener("old-completion");
    } });
    const replacement = constructionManager();
    let factories = 0;
    const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager, {
      broadcast: (channel, completion) => {
        assert.equal(channel, "sand:mcp-auth-event");
        broadcasts.push(completion);
        if (completion === "old-completion") resetPromise = runtime.resetMcpManager();
      },
      refreshHostMcp: (completion) => { refreshes.push(completion); },
    }));
    if (trigger === "installation") {
      await assert.rejects(runtime.ensureMcpManager(), /construction is retired/);
    } else {
      assert.equal(await runtime.ensureMcpManager(), old.manager);
      old.complete("old-completion");
    }
    assert.ok(resetPromise);
    await resetPromise;
    assert.deepEqual(broadcasts, ["old-completion"]);
    assert.deepEqual(refreshes, [], "reset during broadcast must prevent the Host refresh");
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    old.complete("stale-completion");
    replacement.complete("B-completion");
    assert.deepEqual(broadcasts, ["old-completion", "B-completion"]);
    assert.deepEqual(refreshes, ["B-completion"]);
    assert.equal(old.calls.dispose, 1);
    await runtime.dispose();
  });
}

test("MC-04 callbacks belong to the exact attempt while late scrub failures stay observable", async () => {
  const options: ConstructionOptions[] = [];
  const callbacks: unknown[] = [];
  const reports: unknown[] = [];
  const scrub = deferred<void>();
  const reported = deferred<void>();
  const failure = new Error("Old startup scrub failed");
  const old = constructionManager();
  const replacement = constructionManager();
  let scrubs = 0;
  const runtime = createMcpRuntime(constructionDeps(async (captured) => {
    options.push(captured);
    return options.length === 1 ? old.manager : replacement.manager;
  }, {
    pushBoxSecrets: async () => { callbacks.push("scope"); },
    reportConnectorAuth: (report) => { callbacks.push(report); },
    reportMcpDiscoveryFailed: (report) => { callbacks.push(report); },
    reportDiagnostic: (leg, errorClass) => { callbacks.push([leg, errorClass]); },
    cleanupLegacyAuth: () => ++scrubs === 1 ? scrub.promise : Promise.resolve(),
    reportFailure: (subsystem, leg, error) => {
      reports.push([subsystem, leg, error]);
      reported.resolve();
    },
  }));
  const auth = { phase: "flow_completed", outcome: "succeeded" };
  const discovery = { errorClass: "TimeoutError", elapsedMs: 12, servedStale: true };
  const emit = (captured: ConstructionOptions) => {
    captured.onAccountScopeApplied();
    captured.onConnectorAuth(auth);
    captured.onMcpDiscoveryFailed(discovery);
    captured.onMcpDiagnostic({ leg: "discovery", errorClass: "timeout" });
  };
  await runtime.ensureMcpManager();
  emit(options[0]!);
  const expected = ["scope", auth, discovery, ["discovery", "timeout"]];
  assert.deepEqual(callbacks, expected);
  await runtime.resetMcpManager();
  emit(options[0]!);
  assert.deepEqual(callbacks, expected);
  await runtime.ensureMcpManager();
  emit(options[0]!);
  assert.deepEqual(callbacks, expected);
  emit(options[1]!);
  assert.deepEqual(callbacks, [...expected, ...expected]);
  scrub.reject(failure);
  await reported.promise;
  assert.deepEqual(reports, [["mcp-auth-cleanup", "startup-scrub", failure]]);
  assert.equal(await runtime.ensureMcpManager(), replacement.manager);
  await runtime.dispose();
  emit(options[1]!);
  assert.deepEqual(callbacks, [...expected, ...expected]);
});

for (const boundary of ["entry", "service", "token"] as const) {
  test("MC-05 token port refuses retirement at the " + boundary + " boundary", async () => {
    type AuthService = Awaited<ReturnType<ConstructionDeps["ensureCursorAuthService"]>>;
    const serviceStarted = deferred<void>();
    const tokenStarted = deferred<void>();
    const factoryStarted = deferred<void>();
    const heldFactory = deferred<ConstructionManager>();
    const heldService = deferred<AuthService>();
    const heldToken = deferred<string | null>();
    const options: ConstructionOptions[] = [];
    const old = constructionManager();
    const replacement = constructionManager();
    let serviceCalls = 0;
    let tokenCalls = 0;
    let currentToken = "account-A-token";
    let holdService = boundary === "service";
    let holdToken = boundary === "token";
    const service: AuthService = { getValidAccessToken: ({ backendUrl }) => {
      assert.equal(backendUrl, "https://account.example.test");
      tokenCalls += 1;
      tokenStarted.resolve();
      return holdToken ? heldToken.promise : Promise.resolve(currentToken);
    } };
    const runtime = createMcpRuntime(constructionDeps((captured) => {
      options.push(captured);
      if (options.length > 1) return Promise.resolve(replacement.manager);
      factoryStarted.resolve();
      return heldFactory.promise;
    }, { ensureCursorAuthService: () => {
      serviceCalls += 1;
      serviceStarted.resolve();
      return holdService ? heldService.promise : Promise.resolve(service);
    } }));
    const constructionRejected = assert.rejects(runtime.ensureMcpManager(), /construction is retired/);
    await factoryStarted.promise;
    const request = { backendUrl: "https://account.example.test" };
    let rejected: Promise<void> | undefined;
    if (boundary !== "entry") {
      rejected = assert.rejects(options[0]!.getAccessToken(request), /construction is retired/);
      await (boundary === "service" ? serviceStarted.promise : tokenStarted.promise);
    }
    await runtime.resetMcpManager();
    currentToken = "account-B-token";
    assert.equal(await runtime.ensureMcpManager(), replacement.manager);
    holdService = false;
    holdToken = false;
    if (boundary === "entry") {
      await assert.rejects(options[0]!.getAccessToken(request), /construction is retired/);
    } else {
      if (boundary === "service") heldService.resolve(service);
      else heldToken.resolve(currentToken);
      await rejected;
    }
    assert.equal(serviceCalls, boundary === "entry" ? 0 : 1);
    assert.equal(tokenCalls, boundary === "token" ? 1 : 0);
    heldFactory.resolve(old.manager);
    await constructionRejected;
    assert.equal(old.calls.dispose, 1);
    assert.equal(old.calls.observe, 0);
    assert.equal(await options[1]!.getAccessToken(request), "account-B-token");
    assert.equal(serviceCalls, boundary === "entry" ? 1 : 2);
    assert.equal(tokenCalls, boundary === "token" ? 2 : 1);
    await runtime.dispose();
    await assert.rejects(options[1]!.getAccessToken(request), /construction is retired/);
  });
}

function constructionAdapter(deps: ConstructionDeps) {
  const service = createProductionMcpOAuthAdapter({
    resolveRuntimeDeps: () => ({
      ...deps,
      // This controlled factory implements only the facade methods exercised
      // through the real adapter; unused desktop operations must never run.
      createManager: async (options) => await deps.createManager(options) as DesktopMcpManagerFacade,
    }),
    resolveDesktopDeps: () => { throw new Error("Desktop IPC is outside this routed-operation test"); },
  }).create({ machineId: "construction-machine" } as ProductionServiceContext);
  assert.ok(!(service instanceof Promise));
  return service;
}

test("OD-02 captured Attempt is reserved once and stale lookup never constructs a replacement", async () => {
  const old = constructionManager();
  const replacement = constructionManager();
  let factories = 0;
  const runtime = createMcpRuntime(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager));
  try {
    const first = runtime.captureMcpAttempt();
    assert.match(first.id, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    assert.equal(runtime.captureMcpAttempt(), first);
    assert.equal(runtime.requireMcpAttempt(first.id), first);
    assert.equal(runtime.ensureMcpManager(), first.promise);
    assert.equal(first.assertCurrent(), undefined);
    assert.equal(await first.promise, old.manager);
    assert.equal(factories, 1);

    const resetting = runtime.resetMcpManager();
    assert.throws(first.assertCurrent, McpOriginRetiredError);
    assert.throws(() => runtime.requireMcpAttempt(first.id), McpOriginRetiredError);
    await resetting;
    assert.equal(factories, 1, "lookup must not reserve or invoke B after reset");
    const second = runtime.captureMcpAttempt();
    assert.notEqual(second.id, first.id);
    assert.equal(await second.promise, replacement.manager);
    assert.throws(() => runtime.requireMcpAttempt(first.id), McpOriginRetiredError);
    assert.equal(runtime.requireMcpAttempt(second.id), second);
    assert.equal(factories, 2);
    await runtime.dispose();
    assert.throws(second.assertCurrent, McpOriginRetiredError);
    assert.throws(() => runtime.requireMcpAttempt(second.id), McpOriginRetiredError);
    assert.equal(factories, 2);
  } finally { await runtime.dispose(); }
});

for (const boundary of ["factory", "catalog"] as const) {
  for (const retirement of ["source", "attempt"] as const) {
    test("OD-02/03 held " + boundary + " cannot publish origin after " + retirement + " retirement", { timeout: 10_000 }, async (t) => {
      const { source, server } = nativeRoutedSource(t);
      const entered = deferred<void>();
      const factory = deferred<ConstructionManager>();
      const catalog = deferred<unknown>();
      const old = constructionManager();
      const replacement = constructionManager();
      let factories = 0;
      let oldLists = 0;
      const oldManager: ConstructionManager = {
        ...old.manager,
        async listRoutedTools() {
          oldLists += 1;
          if (boundary === "catalog") { entered.resolve(); return await catalog.promise; }
          return [constructionTool];
        },
      };
      const adapter = constructionAdapter(constructionDeps(async () => {
        if (++factories > 1) return replacement.manager;
        if (boundary === "factory") { entered.resolve(); return await factory.promise; }
        return oldManager;
      }));
      t.after(async () => {
        factory.resolve(oldManager);
        catalog.resolve([constructionTool]);
        await adapter.dispose();
      });
      const pending = adapter.listRoutedTools(source);
      const refused = assert.rejects(pending, (error) => retirement === "source" || boundary === "catalog"
        ? error instanceof McpOriginRetiredError
        : error instanceof Error && error.message === "MCP manager construction is retired.");
      await entered.promise;
      if (retirement === "source") server.dispose();
      else await adapter.resetMcpManager();
      factory.resolve(oldManager);
      catalog.resolve([constructionTool]);
      await refused;
      assert.equal(oldLists, boundary === "catalog" ? 1 : 0);
      assert.equal(old.calls.execute, 0);
      assert.equal(factories, 1, "a stale listing cannot silently construct B");
      const freshSource = nativeRoutedSource(t).source;
      const fresh = await adapter.listRoutedTools(freshSource);
      const origin = catalogOrigin(fresh);
      assert.equal(origin.coordinatorSourceId, freshSource.sourceId);
      assert.equal(factories, retirement === "attempt" ? 2 : 1);
      assert.deepEqual(fresh, [{ ...constructionTool, origin }]);
    });
  }
}

test("OD-02/05 stale and malformed origins cannot construct or borrow manager B", { timeout: 10_000 }, async (t) => {
  const sourceA = nativeRoutedSource(t);
  const sourceB = nativeRoutedSource(t);
  const old = constructionManager();
  const replacement = constructionManager();
  const replacementStarted = deferred<void>();
  const replacementFactory = deferred<ConstructionManager>();
  let factories = 0;
  const adapter = constructionAdapter(constructionDeps(async () => {
    if (++factories === 1) return old.manager;
    replacementStarted.resolve();
    return await replacementFactory.promise;
  }));
  t.after(async () => { replacementFactory.resolve(replacement.manager); await adapter.dispose(); });
  const originA = catalogOrigin(await adapter.listRoutedTools(sourceA.source));
  const request = { ...constructionTool, args: { original: true }, toolCallId: "original-call", agentId: "original-agent", origin: originA };
  await adapter.resetMcpManager();
  assertOriginRejected(await adapter.executeRoutedTool(request, sourceA.source));
  assertOriginRejected(await adapter.executeRoutedTool(structuredClone(request), sourceB.source));
  for (const origin of [undefined, null, [], { ...originA, version: 2 }, { ...originA, mcpAttemptId: "invalid" }]) {
    assertOriginRejected(await adapter.executeRoutedTool({ ...request, origin }, sourceB.source));
  }
  assert.equal(factories, 1, "neither missing Attempt nor a foreign source can initiate manager B");
  assert.equal(old.calls.execute, 0);
  assert.equal(replacement.calls.execute, 0);

  const pendingB = adapter.listRoutedTools(sourceB.source);
  void pendingB.catch(() => {});
  await replacementStarted.promise;
  assertOriginRejected(await adapter.executeRoutedTool({ ...request, origin: { ...originA, coordinatorSourceId: sourceB.source.sourceId } }, sourceB.source));
  assert.equal(replacement.calls.execute, 0, "stale execute settles without awaiting the held B factory");
  replacementFactory.resolve(replacement.manager);
  const originB = catalogOrigin(await pendingB);
  assert.notEqual(originB.mcpAttemptId, originA.mcpAttemptId);
  assertOriginRejected(await adapter.executeRoutedTool({ ...request, origin: { ...originA, coordinatorSourceId: sourceB.source.sourceId } }, sourceB.source));
  sourceA.server.dispose();
  assertOriginRejected(await adapter.executeRoutedTool({ ...request, origin: { ...originB, coordinatorSourceId: sourceA.source.sourceId } }, sourceA.source));
  assert.equal(replacement.calls.execute, 0, "an existing B cannot be borrowed by A");
  assert.equal(await adapter.executeRoutedTool({ ...request, origin: originB }, sourceB.source), "executed");
  assert.equal(factories, 2);
  assert.equal(replacement.calls.execute, 1);
});

for (const retirement of ["source", "attempt"] as const) {
  test("OD-02 execute rechecks original " + retirement + " after the manager await", async (t) => {
    const { source, server } = nativeRoutedSource(t);
    const old = constructionManager();
    const replacement = constructionManager();
    let factories = 0;
    const adapter = constructionAdapter(constructionDeps(async () => ++factories === 1 ? old.manager : replacement.manager));
    t.after(async () => { await adapter.dispose(); });
    const origin = catalogOrigin(await adapter.listRoutedTools(source));
    const request = { ...constructionTool, args: {}, toolCallId: "post-manager-await", origin };
    // execute has reached its await of this already-resolved manager promise.
    // Retire synchronously before that continuation can invoke the facade.
    const pending = adapter.executeRoutedTool(request, source);
    if (retirement === "source") server.dispose();
    else await adapter.resetMcpManager();
    assertOriginRejected(await pending);
    assert.equal(old.calls.execute, 0);
    assert.equal(factories, 1);
    const freshSource = nativeRoutedSource(t).source;
    const freshOrigin = catalogOrigin(await adapter.listRoutedTools(freshSource));
    assert.equal(await adapter.executeRoutedTool({ ...request, origin: freshOrigin }, freshSource), "executed");
    assert.equal(retirement === "source" ? old.calls.execute : replacement.calls.execute, 1);
    assert.equal(factories, retirement === "source" ? 1 : 2);
  });
}

test("OD-03 native origin stamping preserves malformed received catalogs and permissive row values", async (t) => {
  const { source } = nativeRoutedSource(t);
  let returned: unknown;
  const owned = constructionManager();
  const adapter = constructionAdapter(constructionDeps(async () => ({
    ...owned.manager,
    listRoutedTools: async () => returned,
  })));
  t.after(async () => { await adapter.dispose(); });
  for (const malformed of [null, undefined, {}, "not-an-array", 7]) {
    returned = malformed;
    assert.equal(await adapter.listRoutedTools(source), malformed);
  }
  const valid = { name: "", providerIdentifier: "", toolName: "raw", inputSchema: null, extra: { keep: true } };
  const badRow = { providerIdentifier: "connector", toolName: "tool" };
  returned = [valid, null, "bad", 0, [], badRow, valid];
  const listed = await adapter.listRoutedTools(source) as unknown[];
  const origin = catalogOrigin(listed);
  assert.deepEqual(listed, [{ ...valid, origin }, null, "bad", 0, [], { ...badRow, origin }, { ...valid, origin }]);
  assert.equal((listed[0] as { extra: unknown }).extra, valid.extra);
  assert.equal((returned as unknown[])[0], valid, "the producer does not mutate the manager's descriptor");
  returned = [];
  assert.deepEqual(await adapter.listRoutedTools(source), []);
});

for (const outcome of ["success", "server-error", "failure", "typed-failure"] as const) {
  test("OD-05/07 native adapter preserves admitted late " + outcome + " after source and Attempt retirement", async (t) => {
    const { source, server } = nativeRoutedSource(t);
    const entered = deferred<void>();
    const result = deferred<unknown>();
    const owned = constructionManager();
    const received: unknown[] = [];
    let assertion: McpDispatchAssertion | undefined;
    let factories = 0;
    const adapter = constructionAdapter(constructionDeps(async () => {
      factories += 1;
      return {
        ...owned.manager,
        async executeRoutedTool(request, assertDispatchCurrent) {
          assert.ok(assertDispatchCurrent);
          assertion = assertDispatchCurrent;
          assert.equal(assertDispatchCurrent(), undefined);
          received.push(request);
          entered.resolve();
          return await result.promise;
        },
      };
    }));
    t.after(async () => { result.resolve(null); await adapter.dispose(); });
    const origin = catalogOrigin(await adapter.listRoutedTools(source));
    const request = { providerIdentifier: "provider", name: "public-name", toolName: "raw-name", args: { value: 7 }, toolCallId: "call-7", agentId: "agent-7", origin };
    const failure = outcome === "typed-failure" ? new McpOriginRetiredError() : new Error(MCP_ORIGIN_RETIRED);
    const pending = adapter.executeRoutedTool(request, source);
    const observed = outcome === "failure" || outcome === "typed-failure"
      ? assert.rejects(pending, (error) => error === failure)
      : pending;
    await entered.promise;
    assert.equal(received[0], request);
    server.dispose();
    await adapter.resetMcpManager();
    assert.ok(assertion);
    assert.throws(assertion, McpOriginRetiredError);
    const success = new McpResult({ result: { case: "success", value: new McpSuccess({ isError: outcome === "server-error" }) } });
    if (outcome === "failure" || outcome === "typed-failure") result.reject(failure);
    else result.resolve(success);
    const settled = await observed;
    if (outcome === "success" || outcome === "server-error") assert.equal(settled, success);
    assert.deepEqual(received, [request]);
    assert.equal(factories, 1, "a late outcome cannot re-list, retry, or construct a fresh manager");
  });
}

for (const action of ["reset", "dispose"] as const) {
  test("MC-06 shipping adapter " + action + " rejects pending routed operations before facade I/O", async (t) => {
    const { source } = nativeRoutedSource(t);
    const started = deferred<void>();
    const held = deferred<ConstructionManager>();
    const old = constructionManager();
    const replacement = constructionManager();
    let factories = 0;
    const adapter = constructionAdapter(constructionDeps(() => {
      if (++factories > 1) return Promise.resolve(replacement.manager);
      started.resolve();
      return held.promise;
    }));
    const request = { providerIdentifier: "connector", name: "tool", toolName: "tool", args: {}, toolCallId: "construction-call" };
    const listRejected = assert.rejects(adapter.listRoutedTools(source), /construction is retired/);
    // OD requires a completed catalog origin before execute may await an
    // Attempt; an unbound request must refuse even while listing is pending.
    const executeRejected = adapter.executeRoutedTool(request, source).then(assertOriginRejected);
    await started.promise;
    assert.equal(factories, 1);
    if (action === "reset") {
      await adapter.resetMcpManager();
      const catalog = await adapter.listRoutedTools(source);
      const origin = catalogOrigin(catalog);
      assert.deepEqual(catalog, [{ ...constructionTool, origin }]);
      assert.equal(await adapter.executeRoutedTool({ ...request, origin }, source), "executed");
    } else {
      await adapter.dispose();
      await assert.rejects(adapter.listRoutedTools(source), /adapter is disposed/);
    }
    held.resolve(old.manager);
    await Promise.all([listRejected, executeRejected]);
    assert.equal(old.calls.list, 0);
    assert.equal(old.calls.execute, 0);
    assert.equal(old.calls.dispose, 1);
    assert.equal(replacement.calls.list, action === "reset" ? 1 : 0);
    assert.equal(replacement.calls.execute, action === "reset" ? 1 : 0);
    assert.equal(factories, action === "reset" ? 2 : 1);
    await adapter.dispose();
  });
}

test("MC-03 shipping adapter awaits and propagates manager teardown failure", async (t) => {
  const { source } = nativeRoutedSource(t);
  const started = deferred<void>();
  const cleanup = deferred<void>();
  const failure = new Error("Shutdown cleanup failed");
  const owned = constructionManager({ dispose: () => {
    started.resolve();
    return cleanup.promise;
  } });
  const adapter = constructionAdapter(constructionDeps(async () => owned.manager));
  const catalog = await adapter.listRoutedTools(source);
  assert.deepEqual(catalog, [{ ...constructionTool, origin: catalogOrigin(catalog) }]);
  let settled = false;
  const disposal = Promise.resolve(adapter.dispose());
  const rejected = assert.rejects(disposal, (error) => error === failure);
  void disposal.then(() => { settled = true; }, () => { settled = true; });
  await started.promise;
  assert.equal(settled, false);
  cleanup.reject(failure);
  await rejected;
  assert.equal(settled, true);
  await adapter.dispose();
  assert.equal(owned.calls.dispose, 1);
});

type BoxBridgePorts = Pick<ConstructionDeps,
  "loadBoxMcpServers" | "listBoxMcpServers" | "listBoxMcpToolsRaw" | "executeBoxMcpToolRaw">;
type BoxBridgeName = keyof BoxBridgePorts;

function controlledBoxPorts(
  call: (name: BoxBridgeName, input: unknown) => Promise<unknown>,
): BoxBridgePorts {
  return {
    loadBoxMcpServers: (input) => call("loadBoxMcpServers", input),
    listBoxMcpServers: async (input) => await call("listBoxMcpServers", input) as readonly Record<string, unknown>[],
    listBoxMcpToolsRaw: async (input) => await call("listBoxMcpToolsRaw", input) as string,
    executeBoxMcpToolRaw: async (input) => await call("executeBoxMcpToolRaw", input) as string,
  };
}

function boxBridgeCases() {
  const config = '{"mcpServers":{"retirement":{"command":"fixture-command"}}}';
  const identifiers = ["retirement"];
  return [
    { name: "loadBoxMcpServers" as const, input: config, value: { loaded: true },
      invoke: (options: ConstructionOptions) => options.loadBoxMcpServers(config) },
    { name: "listBoxMcpServers" as const, input: identifiers, value: [{ serverIdentifier: "retirement" }],
      invoke: (options: ConstructionOptions) => options.listBoxMcpServers(identifiers) },
    { name: "listBoxMcpToolsRaw" as const, input: "cafe", value: "0011",
      invoke: (options: ConstructionOptions) => options.listBoxMcpToolsRaw("cafe") },
    { name: "executeBoxMcpToolRaw" as const, input: "babe", value: "2233",
      invoke: (options: ConstructionOptions) => options.executeBoxMcpToolRaw("babe") },
  ];
}

test("MB-02 current Box bridge ports preserve arguments and actual success or failure", async () => {
  const cases = boxBridgeCases();
  const calls: Array<{ name: BoxBridgeName; input: unknown }> = [];
  const failure = new Error("Box request failed");
  let fail = false;
  let options!: ConstructionOptions;
  const owned = constructionManager();
  const runtime = createMcpRuntime(constructionDeps(async (captured) => {
    options = captured;
    return owned.manager;
  }, controlledBoxPorts(async (name, input) => {
    calls.push({ name, input });
    if (fail) throw failure;
    return cases.find((item) => item.name === name)!.value;
  })));
  await runtime.ensureMcpManager();
  for (const item of cases) assert.equal(await item.invoke(options), item.value);
  fail = true;
  for (const item of cases) await assert.rejects(item.invoke(options), (error) => error === failure);
  assert.deepEqual(calls.map((call) => call.name), [...cases, ...cases].map((item) => item.name));
  calls.forEach((call, index) => assert.equal(call.input, cases[index % cases.length]!.input));
  await runtime.dispose();
});

for (const retirement of ["reset", "dispose", "construction failure"] as const) {
  test("MB-01 all captured Box ports refuse after " + retirement, async () => {
    const cases = boxBridgeCases();
    const calls: BoxBridgeName[] = [];
    const captured: ConstructionOptions[] = [];
    const old = constructionManager();
    const replacement = constructionManager();
    const failure = new Error("Factory refused before publication");
    const runtime = createMcpRuntime(constructionDeps(async (options) => {
      captured.push(options);
      if (captured.length === 1 && retirement === "construction failure") throw failure;
      return captured.length === 1 ? old.manager : replacement.manager;
    }, controlledBoxPorts(async (name) => {
      calls.push(name);
      return cases.find((item) => item.name === name)!.value;
    })));
    if (retirement === "construction failure") {
      await assert.rejects(runtime.ensureMcpManager(), (error) => error === failure);
    } else {
      await runtime.ensureMcpManager();
      await (retirement === "reset" ? runtime.resetMcpManager() : runtime.dispose());
    }
    for (const item of cases) await assert.rejects(item.invoke(captured[0]!), /construction is retired/);
    assert.deepEqual(calls, []);
    if (retirement !== "dispose") {
      assert.equal(await runtime.ensureMcpManager(), replacement.manager);
      for (const item of cases) assert.equal(await item.invoke(captured[1]!), item.value);
      for (const item of cases) await assert.rejects(item.invoke(captured[0]!), /construction is retired/);
      assert.deepEqual(calls, cases.map((item) => item.name));
    } else {
      await assert.rejects(runtime.ensureMcpManager(), /runtime is disposed/);
      assert.equal(captured.length, 1);
    }
    await runtime.dispose();
    assert.equal(old.calls.dispose, retirement === "construction failure" ? 0 : 1);
    assert.equal(replacement.calls.dispose, retirement === "dispose" ? 0 : 1);
  });
}

for (const outcome of ["success", "failure"] as const) {
  test("MB-03 already admitted Box requests keep their actual late " + outcome, async () => {
    const cases = boxBridgeCases();
    const held = new Map(cases.map((item) => [item.name, deferred<unknown>()]));
    const calls: BoxBridgeName[] = [];
    const captured: ConstructionOptions[] = [];
    const failure = new Error("Admitted Box request failed");
    const runtime = createMcpRuntime(constructionDeps(async (options) => {
      captured.push(options);
      return constructionManager().manager;
    }, controlledBoxPorts((name) => {
      calls.push(name);
      return held.get(name)!.promise;
    })));
    await runtime.ensureMcpManager();
    const pending = cases.map((item) => {
      const request = item.invoke(captured[0]!);
      return outcome === "success" ? request : assert.rejects(request, (error) => error === failure);
    });
    assert.deepEqual(calls, cases.map((item) => item.name));
    await runtime.resetMcpManager();
    await runtime.ensureMcpManager();
    for (const item of cases) await assert.rejects(item.invoke(captured[0]!), /construction is retired/);
    assert.deepEqual(calls, cases.map((item) => item.name));
    for (const item of cases) {
      if (outcome === "success") held.get(item.name)!.resolve(item.value);
      else held.get(item.name)!.reject(failure);
    }
    const results = await Promise.all(pending);
    if (outcome === "success") results.forEach((value, index) => assert.equal(value, cases[index]!.value));
    assert.equal(calls.length, cases.length, "retirement cannot replay admitted requests");
    await runtime.dispose();
  });
}

for (const action of ["reset", "dispose"] as const) {
  test("MB-04 shipping adapter " + action + " refuses a facade paused before Box dispatch", async (t) => {
    const { source } = nativeRoutedSource(t);
    const started = deferred<void>();
    const release = deferred<void>();
    const old = constructionManager();
    const replacement = constructionManager();
    const rawCalls: string[] = [];
    let factories = 0;
    let oldFacadeCalls = 0;
    const adapter = constructionAdapter(constructionDeps(async (options) => {
      const first = ++factories === 1;
      return {
        ...(first ? old.manager : replacement.manager),
        async executeRoutedTool() {
          if (first) {
            oldFacadeCalls += 1;
            started.resolve();
            await release.promise;
          }
          return options.executeBoxMcpToolRaw(first ? "aaaa" : "bbbb");
        },
      };
    }, { executeBoxMcpToolRaw: async (payload) => { rawCalls.push(payload); return "result-" + payload; } }));
    const origin = catalogOrigin(await adapter.listRoutedTools(source));
    const request = { providerIdentifier: "connector", name: "tool", toolName: "tool", args: {}, toolCallId: "box-retirement-call", origin };
    const refused = assert.rejects(adapter.executeRoutedTool(request, source), /construction is retired/);
    await started.promise;
    assert.equal(oldFacadeCalls, 1);
    assert.deepEqual(rawCalls, []);
    if (action === "reset") {
      await adapter.resetMcpManager();
      const replacementOrigin = catalogOrigin(await adapter.listRoutedTools(source));
      assert.equal(await adapter.executeRoutedTool({ ...request, origin: replacementOrigin }, source), "result-bbbb");
    } else {
      await adapter.dispose();
      assertOriginRejected(await adapter.executeRoutedTool(request, source));
    }
    release.resolve();
    await refused;
    assert.deepEqual(rawCalls, action === "reset" ? ["bbbb"] : []);
    assert.equal(factories, action === "reset" ? 2 : 1);
    assert.equal(old.calls.dispose, 1);
    await adapter.dispose();
    assert.equal(replacement.calls.dispose, action === "reset" ? 1 : 0);
  });
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
  ports: {
    readConfig?: AccountMcpClient["getMcpConfig"];
    executeTool?: (request: BackendToolRequest) => Promise<McpResult>;
    boxMcpExec?: {
      loadServers(configJson: string): Promise<unknown>;
      executeTool(request: RoutedToolRequest): Promise<McpResult>;
    };
  } = {},
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
    async executeTool(request: BackendToolRequest) {
      executions.push({ serverIdentifier: request.serverIdentifier, toolName: request.toolName });
      if (ports.executeTool != null) return ports.executeTool(request);
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
            getMcpConfig: ports.readConfig ?? unexpectedAccountRpc,
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
      }, {
        boxMcpExec: ports.boxMcpExec == null ? undefined : {
          ...ports.boxMcpExec,
          listTools: backend.listTools,
        },
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

test("MD-01/03/05 a cold current owner rechecks retained tools against persisted same-account disable", { timeout: 10_000 }, async () => {
  const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-dispatch-cold-"));
  const started = deferred<void>();
  const held = deferred<AccountServerResponse>();
  const requests: BackendToolRequest[] = [];
  const payload = new McpToolResultContentItem({
    content: { case: "text", value: new McpTextContent({ text: "connector payload" }) },
  });
  const success = new McpResult({
    result: { case: "success", value: new McpSuccess({ content: [payload] }) },
  });
  let holdAccount = false;
  const fixture = createAccountRetirementFixture(root, async (token) => {
    assert.equal(token, ACCOUNT_B);
    if (holdAccount) {
      started.resolve();
      return held.promise;
    }
    return accountResponse("b");
  }, { executeTool: async (request) => { requests.push(request); return success; } });
  fixture.selectAccount(ACCOUNT_B);
  const pending: Promise<unknown>[] = [];
  try {
    const previous = await fixture.runtime.ensureMcpManager();
    const advertised = await fixture.discovery(previous).getTools();
    const disabledTool = advertised.find((tool) => tool.toolName === DISABLED_TOOL);
    const enabledTool = advertised.find((tool) => tool.toolName === ENABLED_TOOL);
    assert.ok(disabledTool);
    assert.ok(enabledTool);
    await previous.toggleMcpToolDisabled({ serverId: "71", toolName: DISABLED_TOOL });
    const committed = readFileSync(fixture.store.settingsPath, "utf8");
    const reopened = new SandSettingsStore(fixture.store.settingsPath);
    assert.equal(reopened.getMcpCustomInstructionsAccountScope(), accountCacheScope(ACCOUNT_B));
    assert.deepEqual(reopened.getMcpDisabledToolsByServerId(), { "71": [DISABLED_TOOL] });

    await fixture.runtime.resetMcpManager();
    holdAccount = true;
    const current = await fixture.runtime.ensureMcpManager();
    await started.promise;
    assert.notEqual(current, previous);
    assert.equal(current.lastAccountDisplayConfigView(), null);
    assert.equal(current.definitionSourceView().peekHttpServerNames(), undefined);
    const discovery = fixture.discovery(current);
    const args = { nested: { enabled: true }, ids: [1, 2] };
    const denied = discovery.executeTool(undefined, {
      ...disabledTool, args, toolCallId: "cold-disabled",
    }, { agentId: "cold-agent" });
    const allowed = discovery.executeTool(undefined, {
      ...enabledTool, args, toolCallId: "cold-enabled",
    }, { agentId: "cold-agent" });
    pending.push(denied, allowed);
    assert.deepEqual(requests, [], "identity preparation is still held");
    held.resolve(accountResponse("b"));
    const [denial, result] = await Promise.all([denied, allowed]);
    assert.equal(denial.result.case, "error");
    assert.equal(denial.result.value.error, `Tool "${DISABLED_TOOL}" is disabled for "Retirement b".`);
    assert.equal(result, success);
    assert.deepEqual(requests, [{
      serverIdentifier: enabledTool.providerIdentifier, toolName: enabledTool.name,
      args, toolCallId: "cold-enabled", agentId: "cold-agent",
    }]);
    assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committed);
    assert.equal(reopened.getMcpCustomInstructionsAccountScope(), accountCacheScope(ACCOUNT_B));
    assert.deepEqual(reopened.getMcpDisabledToolsByServerId(), { "71": [DISABLED_TOOL] });

    // A warm denial uses toolName, while the backend still receives name.
    const warmDenial = await discovery.executeTool(undefined, {
      ...disabledTool, name: ENABLED_TOOL, args, toolCallId: "warm-disabled-alias",
    }, { agentId: "warm-agent" });
    assert.deepEqual(warmDenial, denial);
    assert.equal(requests.length, 1);
    fixture.store.setMcpCustomInstructionByServerId({
      serverId: "71", displayName: "Retirement b", value: "Preserve this configured note.", mirrorLegacyName: false,
    });
    const warmResult = await discovery.executeTool(undefined, {
      ...enabledTool, name: "wire-safe-alias", args, toolCallId: "warm-enabled-alias",
    }, { agentId: "warm-agent" });
    assert.deepEqual(requests[1], {
      serverIdentifier: enabledTool.providerIdentifier, toolName: "wire-safe-alias",
      args, toolCallId: "warm-enabled-alias", agentId: "warm-agent",
    });
    assert.equal(requests.length, 2);
    assert.equal(warmResult.result.case, "success");
    const content = (warmResult.result.value as McpSuccess).content;
    assert.equal(content.length, 2);
    assert.equal(content[0]!.content.case, "text");
    assert.equal((content[0]!.content.value as McpTextContent).text,
      'Custom instructions for the "Retirement b" connector (always follow them when using this tool):\nPreserve this configured note.');
    assert.deepEqual(content[1], payload);
  } finally {
    held.resolve(accountResponse("b"));
    await Promise.allSettled(pending);
    await fixture.runtime.dispose();
    rmSync(root, { recursive: true, force: true });
  }
});

const STDIO_ACCOUNT_CONFIG = '{"mcpServers":{"retirement-b":{"command":"fixture-command","args":["--stdio"]}}}';
const stdioAccountResponse = (): AccountServerResponse => ({
  servers: [{ ...accountServer("b"), type: "stdio", command: "fixture-command", args: ["--stdio"] }],
});
const readStdioAccountConfig: AccountMcpClient["getMcpConfig"] = async (request) => {
  assert.deepEqual(request, { teamScope: false, redactSecrets: false });
  return { configJson: STDIO_ACCOUNT_CONFIG, serverMetadataByName: { "retirement-b": { serverId: 71n } } };
};

for (const outcome of ["success", "failure"] as const) {
  test("MD-02/03/06 Box preparation " + outcome + " preserves a disable committed before execution", { timeout: 10_000 }, async () => {
    const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-dispatch-box-"));
    const started = deferred<void>();
    const held = deferred<void>();
    const configs: string[] = [];
    const requests: RoutedToolRequest[] = [];
    const failure = new Error("Controlled Box configuration failure");
    const success = new McpResult({ result: { case: "success", value: new McpSuccess({ content: [] }) } });
    let holdPush = true;
    const fixture = createAccountRetirementFixture(root, async () => stdioAccountResponse(), {
      readConfig: readStdioAccountConfig,
      boxMcpExec: {
        async loadServers(config) {
          configs.push(config);
          if (holdPush) { started.resolve(); await held.promise; }
        },
        async executeTool(request) { requests.push(request); return success; },
      },
    });
    fixture.selectAccount(ACCOUNT_B);
    let pending: Promise<unknown> | undefined;
    try {
      const manager = await fixture.runtime.ensureMcpManager();
      assert.deepEqual(await manager.definitionSourceView().getStdioServerConfigs(), {
        "retirement-b": { command: "fixture-command", args: ["--stdio"] },
      });
      const discovery = fixture.discovery(manager);
      const request = {
        providerIdentifier: "retirement-b", name: "wire-dangerous-alias", toolName: DISABLED_TOOL,
        args: { target: "held-preparation" }, toolCallId: "box-held-call",
      };
      const execution = discovery.executeTool(undefined, request, { agentId: "box-agent" });
      pending = execution;
      await started.promise;
      assert.deepEqual(configs, [STDIO_ACCOUNT_CONFIG]);
      assert.deepEqual(requests, []);
      fixture.store.setMcpDisabledToolsByServerId({ "71": [DISABLED_TOOL] });
      const committed = readFileSync(fixture.store.settingsPath, "utf8");
      if (outcome === "success") held.resolve();
      else held.reject(failure);
      const result = await execution;
      assert.equal(result.result.case, "error");
      assert.equal(result.result.value.error, outcome === "success"
        ? `Tool "${DISABLED_TOOL}" is disabled for "Retirement b".`
        : "Could not load MCP servers onto Fabushi's computer: Controlled Box configuration failure");
      assert.deepEqual(requests, [], "no Box execution may follow the held preparation");
      assert.deepEqual(fixture.executions, [], "stdio must not fall through to HTTP execution");
      assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committed);

      holdPush = false;
      const enabled = { ...request, name: "wire-safe-alias", toolName: ENABLED_TOOL, toolCallId: "box-enabled-call" };
      assert.equal(await discovery.executeTool(undefined, enabled, { agentId: "box-agent" }), success);
      assert.equal(requests.length, 1);
      assert.equal(requests[0], enabled, "Box receives the original request unchanged");
      assert.equal(configs.length, outcome === "success" ? 1 : 2);
      assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committed);
    } finally {
      held.resolve();
      await Promise.allSettled(pending == null ? [] : [pending]);
      await fixture.runtime.dispose();
      rmSync(root, { recursive: true, force: true });
    }
  });
}

for (const preparation of ["missing", "unavailable"] as const) {
  test("MD-03/06 cold " + preparation + " identity retains unavailable outcome and permissions", { timeout: 10_000 }, async () => {
    const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-dispatch-unavailable-"));
    const started = deferred<void>();
    const held = deferred<AccountServerResponse>();
    const fixture = createAccountRetirementFixture(root, async () => {
      started.resolve();
      return held.promise;
    });
    fixture.selectAccount(ACCOUNT_B);
    fixture.store.scopeToAccount(accountCacheScope(ACCOUNT_B));
    fixture.store.setMcpDisabledToolsByServerId({ "71": [DISABLED_TOOL] });
    const committed = readFileSync(fixture.store.settingsPath, "utf8");
    let pending: Promise<unknown> | undefined;
    try {
      const manager = await fixture.runtime.ensureMcpManager();
      await started.promise;
      assert.equal(manager.lastAccountDisplayConfigView(), null);
      const execution = executeFixtureTool(fixture.discovery(manager), DISABLED_TOOL);
      pending = execution;
      if (preparation === "missing") held.resolve({ servers: [] });
      else held.reject(new Error("Controlled account listing failure"));
      const result = await execution;
      assert.equal(result.result.case, "error");
      assert.match(result.result.value.error, /MCP server "retirement-b" is not available here/);
      assert.deepEqual(fixture.executions, []);
      assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committed);
      assert.equal(fixture.store.getMcpCustomInstructionsAccountScope(), accountCacheScope(ACCOUNT_B));
      assert.deepEqual(fixture.store.getMcpDisabledToolsByServerId(), { "71": [DISABLED_TOOL] });
    } finally {
      held.resolve({ servers: [] });
      await Promise.allSettled(pending == null ? [] : [pending]);
      await fixture.runtime.dispose();
      rmSync(root, { recursive: true, force: true });
    }
  });
}

for (const transport of ["http", "stdio"] as const) {
  for (const retirement of ["disable", "reset"] as const) {
    for (const outcome of ["success", "rejection"] as const) {
      test("MD-04/06 admitted " + transport + " keeps late " + outcome + " after " + retirement, { timeout: 10_000 }, async () => {
        const root = mkdtempSync(join(tmpdir(), "fabushi-mcp-dispatch-entered-"));
        const entered = deferred<void>();
        const held = deferred<McpResult>();
        const requests: unknown[] = [];
        const failure = new Error("Controlled entered execution failure");
        const success = new McpResult({ result: { case: "success", value: new McpSuccess({ content: [] }) } });
        const execute = (request: unknown) => {
          requests.push(request);
          entered.resolve();
          return held.promise;
        };
        const fixture = createAccountRetirementFixture(root,
          async () => transport === "http" ? accountResponse("b") : stdioAccountResponse(), {
            executeTool: execute,
            ...(transport === "stdio" ? {
              readConfig: readStdioAccountConfig,
              boxMcpExec: { loadServers: async () => {}, executeTool: execute },
            } : {}),
          });
        fixture.selectAccount(ACCOUNT_B);
        let completion: Promise<unknown> | undefined;
        try {
          const manager = await fixture.runtime.ensureMcpManager();
          const discovery = fixture.discovery(manager);
          const descriptor = (await discovery.getTools()).find((tool) => tool.toolName === DISABLED_TOOL);
          assert.ok(descriptor);
          const execution = discovery.executeTool(undefined, {
            ...descriptor, args: { target: "already-entered" }, toolCallId: "entered-call",
          }, { agentId: "entered-agent" });
          completion = outcome === "rejection" ? assert.rejects(execution, (error) => error === failure) : execution;
          await entered.promise;
          assert.equal(requests.length, 1);
          fixture.store.setMcpDisabledToolsByServerId({ "71": [DISABLED_TOOL] });
          const committed = readFileSync(fixture.store.settingsPath, "utf8");
          if (retirement === "reset") await fixture.runtime.resetMcpManager();
          if (outcome === "success") held.resolve(success);
          else held.reject(failure);
          const result = await completion;
          if (outcome === "success") assert.equal(result, success);
          assert.equal(requests.length, 1, "disable/retirement cannot replay an entered call");
          assert.equal(fixture.executions.length, transport === "http" ? 1 : 0);
          assert.equal(readFileSync(fixture.store.settingsPath, "utf8"), committed);
        } finally {
          held.resolve(success);
          await Promise.allSettled(completion == null ? [] : [completion]);
          await fixture.runtime.dispose();
          rmSync(root, { recursive: true, force: true });
        }
      });
    }
  }
}


function desktopUninstallFixture(options: {
  uninstall?: (request: unknown) => Promise<unknown>;
  syncSettings?: (settings: Record<string, unknown>) => Promise<Record<string, any> | null>;
} = {}) {
  const root = mkdtempSync(join(tmpdir(), "fabushi-desktop-uninstall-"));
  const settingsStore = new SandSettingsStore(join(root, "settings.json"));
  const warmed = deferred<void>();
  const owned = constructionManager();
  const calls = { directUninstall: 0, refresh: 0, settings: [] as Record<string, unknown>[], edges: [] as unknown[] };
  const manager = {
    ...owned.manager,
    async getCatalog() { warmed.resolve(); return []; },
    async uninstallPlugin() {
      calls.directUninstall += 1;
      throw new Error("Desktop intent bypassed the Rust MCP owner");
    },
  };
  const legs: Record<string, ((request: unknown) => Promise<unknown>) | undefined> = {
    refreshMcp: async () => { calls.refresh += 1; return null; },
    uninstallMcpPlugin: options.uninstall,
  };
  const context = {
    machineId: "desktop-uninstall-machine",
    settings: { settingsStore },
    coordinatorLegs: { legs },
    mcpHost: {
      refreshMcp: () => { calls.refresh += 1; },
      syncHostSettings: async (settings: Record<string, unknown>) => {
        calls.settings.push(settings);
        return options.syncSettings == null ? settings : await options.syncSettings(settings);
      },
    },
    broadcast() {},
    readTelemetry: () => undefined,
    requireAccount: () => ({ subscribe: () => () => {} }),
    getTrustedContents: () => undefined,
    native: {},
  } as unknown as ProductionServiceContext;
  const adapter = createProductionMcpOAuthAdapter({
    resolveRootPorts: createProductionMcpOAuthRootPortProvider,
    resolveRuntimeDeps: () => ({
      ...constructionDeps(async () => manager),
      // Only catalog warming and runtime lifecycle use this controlled facade.
      // The registered desktop handler and root port provider are production.
      createManager: async () => manager as DesktopMcpManagerFacade,
    }),
    resolveDesktopDeps: () => ({
      shell: { openExternal: async () => { throw new Error("Unexpected external URL"); } },
      parseAllowedExternalUrl: () => null,
      peekAccessToken: async () => null,
      onEdgeFailure: (failure) => { calls.edges.push(failure); },
    }),
  }).create(context);
  const handlers = new Map<string, (event: unknown, request?: unknown) => unknown>();
  adapter.registerDesktopIpc({
    handle(channel, handler) {
      assert.equal(handlers.has(channel), false, "duplicate IPC registration");
      handlers.set(channel, handler);
    },
  });
  const handler = handlers.get("sand:mcp-uninstall-plugin");
  assert.ok(handler, "the actual adapter must register desktop uninstall");
  return {
    calls, legs, settingsStore, warmed: warmed.promise,
    invoke: async (request: unknown) => await handler(undefined, request),
    async dispose() {
      try { await adapter.dispose(); }
      finally { rmSync(root, { recursive: true, force: true }); }
    },
  };
}

test("PS-DESKTOP-01/03/04 registered uninstall awaits one Rust intent and preserves its full response", { timeout: 10_000 }, async () => {
  const entered = deferred<void>();
  const held = deferred<unknown>();
  const requests: unknown[] = [];
  const fixture = desktopUninstallFixture({
    uninstall: async (request) => { requests.push(request); entered.resolve(); return await held.promise; },
  });
  const request = { pluginId: "7" };
  const response = {
    removed: true, reason: "original remote reason",
    state: { servers: [{ id: "unchanged", name: "Still installed", managedByTeamPluginPolicy: true }],
      warnings: ["original warning"], futureStateField: { keep: true } },
    futureOutcomeField: ["preserve", 7],
  };
  let pending: Promise<unknown> | undefined;
  try {
    await fixture.warmed;
    pending = fixture.invoke(request);
    await entered.promise;
    assert.deepEqual(requests, [request]);
    assert.equal(requests[0], request, "forward the original request object");
    assert.deepEqual(fixture.calls.settings, [], "settings cannot precede the Host result");
    assert.equal(fixture.calls.directUninstall, 0);
    held.resolve(response);
    assert.equal(await pending, response, "return the original same-response display projection");
    assert.equal(fixture.calls.settings.length, 1);
    assert.deepEqual(fixture.calls.settings[0], {
      mcpCustomInstructionsByServerId: fixture.settingsStore.getMcpCustomInstructionsByServerId(),
      mcpCustomInstructions: fixture.settingsStore.getMcpCustomInstructions(),
      mcpDisabledToolsByServerId: fixture.settingsStore.getMcpDisabledToolsByServerId(),
    });
    assert.equal(requests.length, 1);
    assert.equal(fixture.calls.directUninstall, 0);
    assert.equal(fixture.calls.refresh, 0, "the Host service owns the uninstall refresh");
    assert.deepEqual(fixture.calls.edges, []);
  } finally {
    held.resolve(response);
    await Promise.allSettled(pending == null ? [] : [pending]);
    await fixture.dispose();
  }
});

test("PS-DESKTOP-04 settings failure after confirmation does not retry, fall back, or refresh again", { timeout: 10_000 }, async () => {
  const order: string[] = [];
  const failure = new Error("Controlled settings synchronization refusal");
  const fixture = desktopUninstallFixture({
    uninstall: async () => { order.push("host-confirmed"); return { removed: true, state: { servers: [] } }; },
    syncSettings: async () => { order.push("settings"); throw failure; },
  });
  try {
    await fixture.warmed;
    await assert.rejects(fixture.invoke({ pluginId: "7" }), (error) => error === failure);
    assert.deepEqual(order, ["host-confirmed", "settings"]);
    assert.equal(fixture.calls.settings.length, 1);
    assert.equal(fixture.calls.directUninstall, 0);
    assert.equal(fixture.calls.refresh, 0);
  } finally { await fixture.dispose(); }
});

for (const outcome of ["not-removed", "unknown-result", "invalid-args", "missing-leg"] as const) {
  test("PS-DESKTOP-02/04 registered uninstall preserves " + outcome + " without local mutation", { timeout: 10_000 }, async () => {
    const requests: unknown[] = [];
    const failure = new Error(outcome === "invalid-args" ? "Host rejected unknown fields" : "Host execution outcome unknown");
    const response = { removed: false, state: { servers: [] }, reason: "policy denied", retained: true };
    const request = outcome === "invalid-args" ? { pluginId: "7", removed: true } : { pluginId: "7" };
    const fixture = desktopUninstallFixture({
      ...(outcome === "missing-leg" ? {} : {
        uninstall: async (input: unknown) => {
          requests.push(input);
          if (outcome === "not-removed") return response;
          throw failure;
        },
      }),
    });
    try {
      await fixture.warmed;
      if (outcome === "not-removed") assert.equal(await fixture.invoke(request), response);
      else if (outcome === "missing-leg") await assert.rejects(fixture.invoke(request), /Coordinator MCP uninstall port is unavailable/);
      else await assert.rejects(fixture.invoke(request), (error) => error === failure);
      assert.deepEqual(requests, outcome === "missing-leg" ? [] : [request]);
      if (outcome !== "missing-leg") assert.equal(requests[0], request, "unknown fields must reach Rust validation intact");
      assert.deepEqual(fixture.calls.settings, []);
      assert.equal(fixture.calls.directUninstall, 0);
      assert.equal(fixture.calls.refresh, 0);
    } finally { await fixture.dispose(); }
  });
}

test("PS-DESKTOP-06 replacement Coordinator leg cannot replay an admitted uninstall", { timeout: 10_000 }, async () => {
  const entered = deferred<void>();
  const held = deferred<unknown>();
  const oldRequests: unknown[] = [];
  const replacementRequests: unknown[] = [];
  const fixture = desktopUninstallFixture({
    uninstall: async (request) => { oldRequests.push(request); entered.resolve(); return await held.promise; },
  });
  const firstReply = { removed: true, state: { servers: [] }, original: true };
  let pending: Promise<unknown> | undefined;
  try {
    await fixture.warmed;
    pending = fixture.invoke({ pluginId: "7" });
    await entered.promise;
    fixture.legs.uninstallMcpPlugin = async (request) => {
      replacementRequests.push(request);
      return { removed: false, state: { servers: [] }, reason: "later request" };
    };
    assert.deepEqual(replacementRequests, []);
    held.resolve(firstReply);
    assert.equal(await pending, firstReply);
    assert.equal(oldRequests.length, 1);
    assert.deepEqual(replacementRequests, [], "a changed leg is not replay authorization");
    assert.equal(fixture.calls.settings.length, 1);
    assert.deepEqual(await fixture.invoke({ pluginId: "8" }), {
      removed: false, state: { servers: [] }, reason: "later request",
    });
    assert.deepEqual(replacementRequests, [{ pluginId: "8" }], "a new user invocation resolves the current leg");
    assert.equal(oldRequests.length, 1);
    assert.equal(fixture.calls.settings.length, 1);
    assert.equal(fixture.calls.directUninstall, 0);
    assert.equal(fixture.calls.refresh, 0);
  } finally {
    held.resolve(firstReply);
    await Promise.allSettled(pending == null ? [] : [pending]);
    await fixture.dispose();
  }
});

const OFFICIAL_ID = "fabushi-official-github";
const OFFICIAL_ORIGIN = "https://api.ombhrum.com";
const OFFICIAL_CHANGED = { status: "changed", provider: "fabushi-official" };
type OfficialAuthIo = {
  getValidAccessToken(args: { backendUrl: string }): Promise<string | null>;
  peekAccessToken(): Promise<string | null>;
};
type OfficialBrokerCall = { path: string; authorization: string | null; body: unknown };
type OfficialProviderCall = { url: string; init: RequestInit; payload: { method: string; id?: number; params?: any } };
type OfficialProviderTool = { name: string; description: string; inputSchema: Record<string, unknown> };

function officialProviderReply(call: OfficialProviderCall, tools: readonly OfficialProviderTool[]): Response {
  if (call.payload.method === "notifications/initialized") return new Response(null, { status: 202 });
  const result = call.payload.method === "initialize" ? { protocolVersion: "2025-03-26" }
    : call.payload.method === "tools/list" ? { tools }
    : call.payload.method === "tools/call" ? { content: [{ type: "text", text: "official provider result" }], isError: false }
    : assert.fail("unexpected official provider method");
  return new Response(JSON.stringify({ jsonrpc: "2.0", id: call.payload.id, result }), {
    headers: { "Content-Type": "application/json",
      ...(call.payload.method === "initialize" ? { "Mcp-Session-Id": "official-dispatch-session" } : {}) },
  });
}

// One event-loop turn drains the finite promise continuations released by an
// owned I/O gate. Poll timers are invoked explicitly below; no elapsed delay
// decides whether a production callback has completed.
const drainOfficialContinuations = () => new Promise<void>(resolve => setImmediate(resolve));

function officialAccountOwnerFixture(t: TestContext) {
  const controls: {
    transition?: (transition: { previousSlot: string; nextSlot: string | null }) => Promise<void>;
  } = {};
  const authorizations: string[] = [];
  const captures: CoordinatorAccountBinding[] = [];
  const problems: string[] = [];
  const events: string[] = [];
  let runtimeSequence = 0;
  const owner = createCoordinatorAccountRuntime<CoordinatorAuthStatus>({
    createRuntime() {
      const id = ++runtimeSequence;
      events.push("create:" + id);
      return {
        requestRendererPort() {},
        revokeRendererPortRequest() { events.push("revoke-renderer:" + id); },
        restart: async () => { events.push("restart:" + id); },
        dispose: async () => { events.push("dispose:" + id); },
      };
    },
    authorizeAccount: async slot => { authorizations.push(slot); return true; },
    revokeRefusedAccount: async () => ({ kind: "revoked", status: { kind: "logged-out" } }),
    prepareAccountTransition: async transition => { await controls.transition?.(transition); },
    resetAccountState() {},
    revokeMainDataPort() {},
    deliverStatus() {},
    onProblem: detail => { problems.push(detail); },
  });
  const ready = owner.start({ kind: "logged-in", authId: "official-lifetime-a" });
  t.after(async () => {
    await owner.dispose();
    assert.deepEqual(problems, [], "the real account owner completed its controlled lifecycle");
  });
  return {
    owner, controls, authorizations, captures, events, ready,
    observe: (slot: string | null) => owner.observe(slot == null
      ? { kind: "logged-out" } : { kind: "logged-in", authId: slot }),
    captureOutputAccountBinding(): McpDispatchAssertion {
      const binding = owner.captureCurrentBinding();
      assert.ok(binding, "the real Coordinator account owner must have a current binding");
      binding.assertCurrent();
      captures.push(binding);
      return () => { binding.assertCurrent(); return undefined; };
    },
  };
}

function officialProductionFixture(t: TestContext) {
  const account = officialAccountOwnerFixture(t);
  const root = mkdtempSync(join(tmpdir(), "fabushi-official-lifetime-"));
  const previousRoot = process.env.SAND_DATA_ROOT;
  const previousBackend = process.env.FABUSHI_API_BASE_URL;
  process.env.SAND_DATA_ROOT = root;
  // The real factory warms legacy discovery. Refuse its credential I/O before
  // Connect-node can use a socket; official OAuth keeps its pinned origin.
  process.env.FABUSHI_API_BASE_URL = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";
  const polls = new Map<object, () => void>();
  const pollHandles = new WeakSet<object>();
  let cleanupAdapter: { dispose(): Promise<void> | void } | undefined;
  let cleaned = false;
  const dispose = async () => {
    if (cleaned) return;
    cleaned = true;
    try {
      await cleanupAdapter?.dispose();
      await drainOfficialContinuations();
      assert.equal(polls.size, 0, "production disposal must clear its owned poll timer");
    } finally {
      t.mock.restoreAll();
      if (previousRoot === undefined) delete process.env.SAND_DATA_ROOT;
      else process.env.SAND_DATA_ROOT = previousRoot;
      if (previousBackend === undefined) delete process.env.FABUSHI_API_BASE_URL;
      else process.env.FABUSHI_API_BASE_URL = previousBackend;
      rmSync(root, { recursive: true, force: true });
    }
  };
  t.after(dispose);
  const vault = new Map<string, string>();
  const writes: Array<{ path: string; value: string }> = [];
  const broker: OfficialBrokerCall[] = [];
  const provider: OfficialProviderCall[] = [];
  const tools: OfficialProviderTool[] = [];
  const reads: string[] = [];
  const broadcasts: Array<{ channel: string; value: unknown }> = [];
  const refreshes: unknown[][] = [];
  const tokenRequests: string[] = [];
  const calls = { authService: 0, peek: 0 };
  let subject = "official-lifetime-a";
  const token = () => "header." + Buffer.from(JSON.stringify({ sub: subject })).toString("base64url") + ".signature";
  let sequence = 0;
  const controls: {
    nextService?: () => Promise<OfficialAuthIo>;
    nextToken?: () => Promise<string | null>;
    nextPeek?: () => Promise<string | null>;
    nextRead?: () => void | Promise<void>;
    nextWrite?: (write: { path: string; value: string }) => Promise<void>;
    broker?: (call: OfficialBrokerCall) => Promise<Response> | Response | undefined;
    provider?: (call: OfficialProviderCall) => Promise<Response> | Response | undefined;
    broadcast?: () => void;
  } = {};
  const auth: OfficialAuthIo = {
    getValidAccessToken: async ({ backendUrl }) => {
      if (backendUrl !== OFFICIAL_ORIGIN) throw new Error("Controlled legacy credential refusal");
      tokenRequests.push(backendUrl);
      const next = controls.nextToken;
      controls.nextToken = undefined;
      return next == null ? token() : await next();
    },
    peekAccessToken: async () => {
      calls.peek += 1;
      const next = controls.nextPeek;
      controls.nextPeek = undefined;
      return next == null ? token() : await next();
    },
  };
  const storePath = (store: SandUserSecretsStore): string => {
    // Observe only the native I/O destination. The real vault constructs the
    // scope-specific store; the fixture must not replace that selection.
    const path: unknown = Reflect.get(store, "storePath");
    assert.equal(typeof path, "string");
    assert.match(relative(join(root, "official-mcp"), path as string), /^[a-f0-9]{64}\.json$/);
    return path as string;
  };
  t.mock.method(SandUserSecretsStore.prototype, "reveal", async function (this: SandUserSecretsStore, key: string) {
    assert.equal(key, "OFFICIAL_MCP_INSTALLS");
    const path = storePath(this);
    reads.push(path);
    const value = vault.get(path) ?? null;
    const next = controls.nextRead;
    controls.nextRead = undefined;
    const completion = next?.();
    // Existing void hooks keep their original synchronous timing. Only a
    // deliberately returned promise adds an await to this native I/O port.
    if (completion !== undefined) await completion;
    return value;
  });
  t.mock.method(SandUserSecretsStore.prototype, "upsert", async function (this: SandUserSecretsStore, entries: Readonly<Record<string, string>>) {
    assert.deepEqual(Object.keys(entries), ["OFFICIAL_MCP_INSTALLS"]);
    const write = { path: storePath(this), value: entries.OFFICIAL_MCP_INSTALLS! };
    writes.push(write);
    const next = controls.nextWrite;
    controls.nextWrite = undefined;
    if (next != null) await next(write);
    vault.set(write.path, write.value);
  });
  const json = (value: unknown) => new Response(JSON.stringify(value), { headers: { "Content-Type": "application/json" } });
  t.mock.method(globalThis, "fetch", async (input: string | URL | Request, init?: RequestInit) => {
    const url = new URL(typeof input === "string" || input instanceof URL ? String(input) : input.url);
    assert.ok(init, "official network calls retain explicit request options");
    if (url.origin === OFFICIAL_ORIGIN) {
      assert.equal(init.method, "POST");
      assert.equal(init.redirect, "error");
      const call = { path: url.pathname, authorization: new Headers(init.headers).get("Authorization"),
        body: JSON.parse(String(init.body)) as unknown };
      broker.push(call);
      const controlled = controls.broker?.(call);
      if (controlled !== undefined) return await controlled;
      if (call.path === "/api/mcp/oauth/start") {
        const attemptId = "official-attempt-" + ++sequence;
        return json({ attemptId, authorizationUrl: OFFICIAL_ORIGIN + "/api/mcp/oauth/authorize?ticket=" + attemptId });
      }
      if (/\/api\/mcp\/oauth\/attempts\/[^/]+$/.test(call.path)) {
        return json({ status: "ready", credential: { token: "fixture-provider-access",
          refreshToken: "fixture-provider-refresh", connectionId: "fixture-connection", expiresAt: Date.now() + 3_600_000 } });
      }
      assert.match(call.path, /\/(?:ack|cancel|revoke)$/);
      return json({});
    }
    assert.equal(url.href, "https://api.githubcopilot.com/mcp/", "unexpected provider network I/O");
    const call: OfficialProviderCall = { url: url.href, init, payload: JSON.parse(String(init.body)) };
    provider.push(call);
    const controlled = controls.provider?.(call);
    if (controlled !== undefined) return await controlled;
    return officialProviderReply(call, tools);
  });
  const originalSetTimeout = globalThis.setTimeout;
  const originalClearTimeout = globalThis.clearTimeout;
  t.mock.method(globalThis, "setTimeout", ((callback: (...args: any[]) => void, ms?: number, ...args: any[]) => {
    if (ms !== 2_000) return originalSetTimeout(callback, ms, ...args);
    const handle = { unref() { return handle; } };
    pollHandles.add(handle);
    polls.set(handle, () => callback(...args));
    return handle as unknown as ReturnType<typeof setTimeout>;
  }) as typeof setTimeout);
  t.mock.method(globalThis, "clearTimeout", ((handle: Parameters<typeof clearTimeout>[0]) => {
    if (typeof handle === "object" && handle != null && pollHandles.has(handle)) {
      polls.delete(handle);
      return;
    }
    originalClearTimeout(handle);
  }) as typeof clearTimeout);

  const context = {
    machineId: "official-lifetime-machine",
    requireCoordinator: () => ({ getAccountRuntime: () => account.owner }),
    settings: { settingsStore: new SandSettingsStore(join(root, "settings.json")) },
    requireAccount: () => ({
      getAuthService: async () => {
        calls.authService += 1;
        const next = controls.nextService;
        controls.nextService = undefined;
        return next == null ? auth : await next();
      },
    }),
    requireExperiments: () => ({ checkFeatureGate: () => false }),
    coordinatorLegs: { legs: {} },
    secretsStores: { pushBoxSecrets: { push: async () => {} } },
    readTelemetry: () => undefined,
    broadcast: (channel: string, value: unknown) => {
      broadcasts.push({ channel, value });
      controls.broadcast?.();
    },
    mcpHost: {
      refreshMcp: async (...args: unknown[]) => { refreshes.push(args); },
      syncHostSettings: async () => ({}),
    },
    native: { shell: { openExternal: async () => {} } },
  } as unknown as ProductionServiceContext;
  // No resolver or manager replacement: exercise the actual default production
  // factory, runtime, facade, official service, OAuth adapter and vault mapping.
  const ports = createProductionMcpOAuthPorts();
  const adapter = createProductionMcpOAuthAdapter(ports).create(context);
  cleanupAdapter = adapter;
  const install = async (values?: Record<string, string>) => {
    await account.ready;
    await account.owner.whenIdle();
    return await adapter.installHostMcpPlugin({ entryId: OFFICIAL_ID, ...(values == null ? {} : { values }) });
  };
  const runPoll = () => {
    assert.equal(polls.size, 1, "one actual official authorization poll must be scheduled");
    const [handle, callback] = polls.entries().next().value!;
    polls.delete(handle);
    callback();
  };
  return {
    adapter, auth, controls, calls, broker, provider, tools, reads, broadcasts, refreshes, tokenRequests, writes, vault, runPoll, dispose, json, account,
    token, switchAccount: (nextSubject = "official-lifetime-b") => {
      subject = nextSubject;
      // Match auth wiring: observation retires the old owner synchronously.
      // Individual cases control transition I/O, including the MCP reset edge.
      account.observe(nextSubject);
    },
    fallbackRefresh: (...args: [] | [unknown]) => ports.resolveRuntimeDeps(context).refreshHostMcp(...args),
    install,
    authenticate: () => adapter.authenticateHostMcpServer({ serverId: OFFICIAL_ID, accountKey: "default", forceReauth: true }),
    async ready() { await install(); await drainOfficialContinuations(); },
  };
}

for (const boundary of ["service", "token"] as const) {
  test("MO-05 default official credential bridge refuses reset across the " + boundary + " await", { timeout: 10_000 }, async t => {
    const fixture = officialProductionFixture(t);
    const entered = deferred<void>();
    const service = deferred<OfficialAuthIo>();
    const token = deferred<string | null>();
    let operation: Promise<unknown> | undefined;
    try {
      await fixture.ready();
      if (boundary === "service") {
        // The real authentication snapshot finishes its vault read before the
        // broker asks for an auth service. Arm that actual I/O edge, not a
        // guessed number of microtasks or scope reads.
        fixture.controls.nextRead = () => {
          fixture.controls.nextService = () => { entered.resolve(); return service.promise; };
        };
      } else {
        fixture.controls.nextToken = () => { entered.resolve(); return token.promise; };
      }
      operation = fixture.authenticate();
      void operation.catch(() => {});
      await entered.promise;
      await fixture.adapter.resetMcpManager();
      fixture.switchAccount();
      await fixture.ready();
      service.resolve(fixture.auth);
      token.resolve(fixture.token());
      await assert.rejects(operation, /construction is retired/);
      assert.equal(fixture.tokenRequests.length, boundary === "service" ? 0 : 1);
      assert.deepEqual(fixture.broker, [], "retired token acquisition cannot start an official broker request");
      await fixture.authenticate();
      assert.equal(fixture.tokenRequests.at(-1), OFFICIAL_ORIGIN);
      assert.equal(fixture.broker.length, 1);
      assert.equal(fixture.broker[0]!.path, "/api/mcp/oauth/start");
      assert.equal(fixture.broker[0]!.authorization, "Bearer " + fixture.token());
    } finally {
      service.resolve(fixture.auth);
      token.resolve(fixture.token());
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}

for (const retirement of ["reset", "dispose"] as const) {
  test("MO-05 real official disposal cannot obtain credentials for cancel after " + retirement, { timeout: 10_000 }, async t => {
    const fixture = officialProductionFixture(t);
    try {
      await fixture.ready();
      await fixture.authenticate();
      const before = { services: fixture.calls.authService, tokens: fixture.tokenRequests.length, broker: fixture.broker.length };
      if (retirement === "reset") await fixture.adapter.resetMcpManager();
      else await fixture.adapter.dispose();
      await drainOfficialContinuations();
      assert.deepEqual({ services: fixture.calls.authService, tokens: fixture.tokenRequests.length, broker: fixture.broker.length }, before);
      assert.deepEqual(fixture.broadcasts, []);
      assert.deepEqual(fixture.refreshes, []);
      if (retirement === "reset") {
        await fixture.ready();
        await fixture.authenticate();
        assert.equal(fixture.broker.at(-1)!.path, "/api/mcp/oauth/start");
      } else {
        await assert.rejects(fixture.authenticate(), /disposed/);
      }
    } finally {
      await fixture.dispose();
    }
  });
}

test("MO-05 default official bridge retains current credential errors and unauthenticated message", { timeout: 10_000 }, async t => {
  const fixture = officialProductionFixture(t);
  try {
    await fixture.ready();
    const failure = new Error("Controlled official credential failure");
    fixture.controls.nextToken = async () => { throw failure; };
    await assert.rejects(fixture.authenticate(), error => error === failure);
    fixture.controls.nextToken = async () => null;
    await assert.rejects(fixture.authenticate(), /请先登录 Fabushi/);
    assert.deepEqual(fixture.broker, []);
    await fixture.authenticate();
    assert.equal(fixture.broker[0]!.authorization, "Bearer " + fixture.token());
  } finally {
    await fixture.dispose();
  }
});

for (const result of ["failure", "ready"] as const) {
  test("MO-06 actual official poll " + result + " after reset cannot notify the replacement", { timeout: 10_000 }, async t => {
    const fixture = officialProductionFixture(t);
    const entered = deferred<void>();
    const response = deferred<Response>();
    try {
      await fixture.ready();
      await fixture.authenticate();
      fixture.controls.broker = call => {
        if (/\/attempts\/[^/]+$/.test(call.path)) { entered.resolve(); return response.promise; }
      };
      fixture.runPoll();
      await entered.promise;
      await fixture.adapter.resetMcpManager();
      await fixture.ready();
      const before = [...fixture.vault.entries()];
      if (result === "failure") response.reject(new Error("Controlled held poll failure"));
      else response.resolve(fixture.json({ status: "ready", credential: {
        token: "stale-provider", connectionId: "stale-connection",
      } }));
      await drainOfficialContinuations();
      assert.deepEqual(fixture.broadcasts, []);
      assert.deepEqual(fixture.refreshes, []);
      assert.deepEqual([...fixture.vault.entries()], before);
      assert.equal(fixture.broker.filter(call => /\/(?:cancel|ack)$/.test(call.path)).length, 0);
    } finally {
      response.resolve(fixture.json({ status: "failed" }));
      await drainOfficialContinuations();
      await fixture.dispose();
    }
  });
}

test("MO-06 actual official acknowledgment after reset keeps its admitted outcome without notifying B", { timeout: 10_000 }, async t => {
  const fixture = officialProductionFixture(t);
  const entered = deferred<void>();
  const response = deferred<Response>();
  try {
    await fixture.ready();
    await fixture.authenticate();
    fixture.controls.broker = call => {
      if (call.path.endsWith("/ack")) { entered.resolve(); return response.promise; }
    };
    fixture.runPoll();
    await entered.promise;
    assert.equal(JSON.parse([...fixture.vault.values()][0]!)[OFFICIAL_ID].connectionId, "fixture-connection",
      "the real service persists credentials before acknowledging");
    await fixture.adapter.resetMcpManager();
    await fixture.ready();
    const before = [...fixture.vault.entries()];
    response.resolve(fixture.json({}));
    await drainOfficialContinuations();
    assert.deepEqual([...fixture.vault.entries()], before);
    assert.deepEqual(fixture.broadcasts, []);
    assert.deepEqual(fixture.refreshes, []);
    assert.equal(fixture.broker.filter(call => call.path.endsWith("/ack")).length, 1, "admitted ack is not replayed");
    assert.equal(fixture.broker.filter(call => call.path.endsWith("/cancel")).length, 0);
  } finally {
    response.resolve(fixture.json({}));
    await drainOfficialContinuations();
    await fixture.dispose();
  }
});

for (const result of ["success", "failure"] as const) {
  test("MO-06 actual official disconnect cleanup " + result + " after reset cannot notify B", { timeout: 10_000 }, async t => {
    const fixture = officialProductionFixture(t);
    const entered = deferred<void>();
    const response = deferred<Response>();
    let operation: Promise<unknown> | undefined;
    try {
      await fixture.ready();
      await fixture.authenticate();
      fixture.runPoll();
      await drainOfficialContinuations();
      assert.deepEqual(fixture.broadcasts, [{ channel: "sand:mcp-auth-event", value: OFFICIAL_CHANGED }]);
      assert.deepEqual(fixture.refreshes, [[]]);
      fixture.controls.broker = call => {
        if (call.path.endsWith("/revoke")) { entered.resolve(); return response.promise; }
      };
      operation = fixture.adapter.logoutHostMcpAccount({ serverId: OFFICIAL_ID, accountKey: "default" });
      void operation.catch(() => {});
      await entered.promise;
      assert.equal(JSON.parse([...fixture.vault.values()][0]!)[OFFICIAL_ID].token, undefined,
        "local credential removal precedes remote revocation");
      await fixture.adapter.resetMcpManager();
      await fixture.ready();
      if (result === "success") response.resolve(fixture.json({}));
      else response.reject(new Error("Controlled admitted revoke failure"));
      await assert.rejects(operation, result === "success" ? /disposed/ : /本机授权/);
      assert.equal(fixture.broadcasts.length, 1);
      assert.deepEqual(fixture.refreshes, [[]]);
      assert.equal(fixture.broker.filter(call => call.path.endsWith("/revoke")).length, 1);
    } finally {
      response.resolve(fixture.json({}));
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}

test("MO-06 default official completion preserves zero-argument refresh and rechecks broadcast reset", { timeout: 10_000 }, async t => {
  const fixture = officialProductionFixture(t);
  let reset: Promise<void> | undefined;
  try {
    await fixture.ready();
    // The default root overrides this fallback, so qualify its argument shape
    // as well as the actual root callback below.
    fixture.fallbackRefresh();
    const legacy = { status: "legacy-completion" };
    fixture.fallbackRefresh(legacy);
    fixture.fallbackRefresh(undefined);
    assert.deepEqual(fixture.refreshes, [[], [legacy], [undefined]]);
    fixture.refreshes.length = 0;
    fixture.controls.broadcast = () => { reset = fixture.adapter.resetMcpManager(); };
    await fixture.authenticate();
    fixture.runPoll();
    await drainOfficialContinuations();
    assert.ok(reset);
    await reset;
    assert.deepEqual(fixture.broadcasts, [{ channel: "sand:mcp-auth-event", value: OFFICIAL_CHANGED }]);
    assert.deepEqual(fixture.refreshes, [], "broadcast retirement prevents the subsequent Host refresh");
    fixture.controls.broadcast = undefined;
    await fixture.ready();
    await fixture.authenticate();
    fixture.runPoll();
    await drainOfficialContinuations();
    assert.deepEqual(fixture.broadcasts, [
      { channel: "sand:mcp-auth-event", value: OFFICIAL_CHANGED },
      { channel: "sand:mcp-auth-event", value: OFFICIAL_CHANGED },
    ]);
    assert.deepEqual(fixture.refreshes, [[]], "the current official path invokes refresh with zero arguments");
  } finally {
    await reset;
    await fixture.dispose();
  }
});

test("MO-07 default official mutation cannot write after its final scope await is retired", { timeout: 10_000 }, async t => {
  const fixture = officialProductionFixture(t);
  const entered = deferred<void>();
  const scope = deferred<string | null>();
  let operation: Promise<unknown> | undefined;
  try {
    await fixture.ready();
    fixture.controls.nextRead = () => {
      fixture.controls.nextPeek = () => { entered.resolve(); return scope.promise; };
    };
    operation = fixture.install({ ACCESS_TOKEN: "stale-provider" });
    void operation.catch(() => {});
    await entered.promise;
    const before = fixture.writes.length;
    await fixture.adapter.resetMcpManager();
    await fixture.install({ ACCESS_TOKEN: "replacement-provider" });
    assert.equal(fixture.writes.length, before + 1, "current B still commits once");
    const committed = [...fixture.vault.entries()];
    scope.resolve(fixture.token());
    await assert.rejects(operation, /disposed/);
    assert.equal(fixture.writes.length, before + 1, "A never enters the write port");
    assert.deepEqual([...fixture.vault.entries()], committed, "B's exact same-account bytes survive");
  } finally {
    scope.resolve(fixture.token());
    await Promise.allSettled(operation == null ? [] : [operation]);
    await fixture.dispose();
  }
});

test("MO-07 official pre-write admission refuses retirement after the final scope check fulfills", { timeout: 10_000 }, async () => {
  const accountScope = "a".repeat(64);
  const finalScope = deferred<string>();
  const entered = deferred<void>();
  const retirementObserved = deferred<void>();
  let holdFinalScope = false;
  let stored = "{}";
  const oldWrites: string[] = [];
  const replacementWrites: string[] = [];
  const unexpectedFetch = async (): Promise<Response> => { throw new Error("Unexpected official network I/O"); };
  const old = new OfficialMcpService({
    getScope: (): Promise<string> => {
      if (!holdFinalScope) return Promise.resolve(accountScope);
      holdFinalScope = false;
      // Register before returning the native promise to scope()'s await. On
      // release, this reaction queues retirement behind the scope continuation
      // but ahead of the mutation continuation that scope then makes runnable.
      void finalScope.promise.then(() => queueMicrotask(() => {
        old.dispose();
        retirementObserved.resolve();
      }));
      entered.resolve();
      return finalScope.promise;
    },
    read: async scope => {
      assert.equal(scope, accountScope);
      holdFinalScope = true;
      return stored;
    },
    write: async (scope, value) => {
      assert.equal(scope, accountScope);
      oldWrites.push(value);
      stored = value;
    },
    fetch: unexpectedFetch,
  });
  const replacement = new OfficialMcpService({
    getScope: () => Promise.resolve(accountScope),
    read: async scope => { assert.equal(scope, accountScope); return stored; },
    write: async (scope, value) => {
      assert.equal(scope, accountScope);
      replacementWrites.push(value);
      stored = value;
    },
    fetch: unexpectedFetch,
  });
  let operation: Promise<void> | undefined;
  try {
    operation = old.install(OFFICIAL_ID, { ACCESS_TOKEN: "stale-provider" });
    void operation.catch(() => {});
    await entered.promise;
    await replacement.install(OFFICIAL_ID, { ACCESS_TOKEN: "replacement-provider" });
    assert.equal(replacementWrites.length, 1, "the current same-account service writes once");
    assert.equal(JSON.parse(stored)[OFFICIAL_ID].token, "replacement-provider");
    const committed = stored;
    finalScope.resolve(accountScope);
    await retirementObserved.promise;
    await assert.rejects(operation, /disposed/);
    assert.deepEqual(oldWrites, [], "the retired mutation never enters its write port");
    assert.equal(stored, committed, "the replacement's exact stored bytes survive");
  } finally {
    finalScope.resolve(accountScope);
    await Promise.allSettled(operation == null ? [] : [operation]);
    old.dispose();
    replacement.dispose();
  }
});

for (const result of ["success", "failure"] as const) {
  test("MO-07 default official already-admitted write retains its " + result + " after reset", { timeout: 10_000 }, async t => {
    const fixture = officialProductionFixture(t);
    const entered = deferred<void>();
    const write = deferred<void>();
    const failure = new Error("Controlled admitted vault failure");
    let operation: Promise<unknown> | undefined;
    try {
      await fixture.ready();
      const before = [...fixture.vault.entries()];
      fixture.controls.nextWrite = async () => { entered.resolve(); await write.promise; };
      operation = fixture.install({ ACCESS_TOKEN: "admitted-provider" });
      void operation.catch(() => {});
      await entered.promise;
      const admittedWrites = fixture.writes.length;
      await fixture.adapter.resetMcpManager();
      if (result === "success") {
        write.resolve();
        // The real facade performs a later servers() read; that read may refuse
        // retirement even though its previously admitted write completed.
        await assert.rejects(operation, /disposed/);
        assert.equal(JSON.parse([...fixture.vault.values()][0]!)[OFFICIAL_ID].token, "admitted-provider");
        await fixture.ready();
        assert.equal(JSON.parse([...fixture.vault.values()][0]!)[OFFICIAL_ID].token, "admitted-provider");
        assert.equal(fixture.writes.length, admittedWrites + 1);
      } else {
        write.reject(failure);
        await assert.rejects(operation, error => error === failure);
        assert.deepEqual([...fixture.vault.entries()], before);
        assert.equal(fixture.writes.length, admittedWrites);
      }
    } finally {
      write.resolve();
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}


const OFFICIAL_DISPATCH_TOKEN = "official-dispatch-provider-token";
const OFFICIAL_DISPATCH_ERROR = "Official MCP call failed or authorization changed. The call was not retried; confirm its outcome before retrying a write.";
const OFFICIAL_DISPATCH_TOOLS: readonly OfficialProviderTool[] = [
  { name: "read_issue", description: "Read an issue", inputSchema: { type: "object" } },
  { name: "write_issue", description: "Write an issue", inputSchema: { type: "object" } },
];
const officialDispatchOptions = { concurrency: false, timeout: 10_000 };

function officialCallEntries(calls: readonly OfficialProviderCall[]) {
  return calls.filter(call => call.payload.method === "tools/call");
}
function assertOfficialDispatchError(value: unknown): void {
  assert.ok(value instanceof McpResult);
  assert.deepEqual(value.toJson(), generatedMcpResultFactory.error(OFFICIAL_DISPATCH_ERROR).toJson());
}
function assertOfficialDispatchSuccess(value: unknown, isError = false): asserts value is McpResult {
  assert.ok(value instanceof McpResult);
  assert.ok(value.result.case === "success");
  assert.equal(value.result.value.isError, isError);
}

async function prepareOfficialDispatch(fixture: ReturnType<typeof officialProductionFixture>): Promise<void> {
  fixture.tools.push(...OFFICIAL_DISPATCH_TOOLS);
  await fixture.install({ ACCESS_TOKEN: OFFICIAL_DISPATCH_TOKEN });
  await drainOfficialContinuations();
}
async function officialDispatchRequest(
  fixture: ReturnType<typeof officialProductionFixture>,
  source: CoordinatorControlSourceContext,
  toolName = "read_issue",
  args: unknown = { issue: 17, nested: { values: [null, false, 1.25, "literal"] } },
) {
  const catalog = await fixture.adapter.listRoutedTools(source);
  assert.ok(Array.isArray(catalog));
  const row = catalog.find((value: any) => value.providerIdentifier === OFFICIAL_ID && value.toolName === toolName);
  assert.ok(row, "the actual default production chain must publish the requested official descriptor");
  const origin = parseMcpRoutedOrigin(row.origin);
  assert.ok(origin);
  assert.equal(origin.coordinatorSourceId, source.sourceId);
  assert.equal(row.name, OFFICIAL_ID + "_" + toolName);
  return { providerIdentifier: row.providerIdentifier, name: row.name, toolName: row.toolName,
    args, toolCallId: "official-dispatch-call", agentId: "official-dispatch-agent", origin };
}

for (const boundary of ["vault", "initialize", "notification", "pagination"] as const) {
  test("OF-T1 default official source retirement at " + boundary + " refuses tools/call and permits a fresh source", officialDispatchOptions, async t => {
    const fixture = officialProductionFixture(t);
    const old = nativeRoutedSource(t);
    const fresh = nativeRoutedSource(t);
    const entered = deferred<void>();
    const release = deferred<void>();
    let operation: Promise<unknown> | undefined;
    try {
      await prepareOfficialDispatch(fixture);
      const request = await officialDispatchRequest(fixture, old.source);
      const before = fixture.provider.length;
      if (boundary === "vault") {
        fixture.controls.nextRead = async () => { entered.resolve(); await release.promise; };
      } else {
        let held = false;
        const method = boundary === "notification" ? "notifications/initialized"
          : boundary === "pagination" ? "tools/list" : "initialize";
        fixture.controls.provider = async call => {
          if (held || call.payload.method !== method) return officialProviderReply(call, fixture.tools);
          held = true;
          entered.resolve();
          await release.promise;
          if (boundary === "pagination") return fixture.json({
            jsonrpc: "2.0", id: call.payload.id,
            result: { tools: [OFFICIAL_DISPATCH_TOOLS[0]], nextCursor: "official-page-2" },
          });
          return officialProviderReply(call, fixture.tools);
        };
      }
      operation = fixture.adapter.executeRoutedTool(request, old.source);
      await entered.promise;
      old.server.dispose();
      assert.throws(old.source.assertCurrent, McpOriginRetiredError);
      release.resolve();
      assertOriginRejected(await operation);
      assert.equal(officialCallEntries(fixture.provider).length, 0, "refusal proves zero tool-call fetch entries");
      assert.equal(fixture.provider.length - before,
        boundary === "vault" ? 0 : boundary === "initialize" ? 1 : boundary === "notification" ? 2 : 3,
        "initialization/discovery entries are counted separately from tool effects");
      fixture.controls.provider = undefined;
      const current = await officialDispatchRequest(fixture, fresh.source);
      assert.equal(current.origin.mcpAttemptId, request.origin.mcpAttemptId, "source retirement does not replace the current manager");
      assert.notEqual(current.origin.coordinatorSourceId, request.origin.coordinatorSourceId);
      assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
      assert.equal(officialCallEntries(fixture.provider).length, 1);
      assert.deepEqual(officialCallEntries(fixture.provider)[0]!.payload.params, { name: current.toolName, arguments: current.args });
    } finally {
      release.resolve();
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}

test("OF-T1 default official Attempt reset cannot dispatch or borrow B after a held vault read", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const old = nativeRoutedSource(t);
  const fresh = nativeRoutedSource(t);
  const entered = deferred<void>();
  const release = deferred<void>();
  let operation: Promise<unknown> | undefined;
  try {
    await prepareOfficialDispatch(fixture);
    const request = await officialDispatchRequest(fixture, old.source);
    const before = fixture.provider.length;
    fixture.controls.nextRead = async () => { entered.resolve(); await release.promise; };
    operation = fixture.adapter.executeRoutedTool(request, old.source);
    await entered.promise;
    await fixture.adapter.resetMcpManager();
    assert.equal(old.source.assertCurrent(), undefined, "only the Attempt was retired");
    const nativeBeforeReplay = { ...fixture.calls, reads: fixture.reads.length, provider: fixture.provider.length };
    assertOriginRejected(await fixture.adapter.executeRoutedTool(request, old.source));
    assert.deepEqual({ ...fixture.calls, reads: fixture.reads.length, provider: fixture.provider.length }, nativeBeforeReplay,
      "stale intake cannot start native I/O or construct a replacement");
    release.resolve();
    assertOfficialDispatchError(await operation);
    assert.equal(fixture.provider.length, before, "the disposed old service makes no provider handoff");
    assert.equal(officialCallEntries(fixture.provider).length, 0);
    const current = await officialDispatchRequest(fixture, fresh.source);
    assert.notEqual(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
    const beforeSecondReplay = fixture.provider.length;
    assertOriginRejected(await fixture.adapter.executeRoutedTool(request, old.source));
    assert.equal(fixture.provider.length, beforeSecondReplay, "an existing B cannot authorize A's origin");
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
    assert.equal(officialCallEntries(fixture.provider).length, 1);
  } finally {
    release.resolve();
    await Promise.allSettled(operation == null ? [] : [operation]);
    await fixture.dispose();
  }
});

// Direct native I/O controls for the final resource boundary. The real service
// owns every snapshot, inventory request, fence, send and result conversion.
function officialDispatchServiceFixture(t: TestContext, captureOutputAccountBinding?: () => McpDispatchAssertion) {
  const scope = "c".repeat(64);
  let stored = "{}";
  const calls: OfficialProviderCall[] = [];
  const controls: {
    nextRead?: (value: string) => Promise<string | null>;
    provider?: (call: OfficialProviderCall) => Response | Promise<Response> | undefined;
  } = {};
  const service = new OfficialMcpService({
    ...(captureOutputAccountBinding == null ? {} : { captureOutputAccountBinding }),
    getScope: () => Promise.resolve(scope),
    read: current => {
      assert.equal(current, scope);
      const next = controls.nextRead;
      controls.nextRead = undefined;
      // Return the owned promise directly: the ordering controls below bind to
      // this actual read -> state -> fence -> request continuation chain.
      return next == null ? Promise.resolve(stored) : next(stored);
    },
    write: async (current, value) => { assert.equal(current, scope); stored = value; },
    fetch: (url, init) => {
      const call: OfficialProviderCall = { url, init, payload: JSON.parse(String(init.body)) };
      calls.push(call);
      return Promise.resolve(controls.provider?.(call) ?? officialProviderReply(call, OFFICIAL_DISPATCH_TOOLS));
    },
  });
  t.after(() => service.dispose());
  return { service, calls, controls,
    ready: () => service.install(OFFICIAL_ID, { ACCESS_TOKEN: OFFICIAL_DISPATCH_TOKEN }) };
}

test("OF-T2 official final send refuses retirement after its last awaited fence fulfills", officialDispatchOptions, async t => {
  const fixture = officialDispatchServiceFixture(t);
  const old = nativeRoutedSource(t);
  const fresh = nativeRoutedSource(t);
  const entered = deferred<void>();
  const read = deferred<string | null>();
  const retired = deferred<void>();
  let saved = "";
  let notifications = 0;
  let operation: Promise<unknown> | undefined;
  try {
    await fixture.ready();
    fixture.controls.provider = call => {
      if (call.payload.method === "notifications/initialized" && ++notifications === 2) {
        fixture.controls.nextRead = value => {
          saved = value;
          // Register this read reaction before state() awaits the promise.
          // It queues the first microtask behind state's continuation; that
          // queues the second behind fence's continuation and ahead of the
          // resumed request. Retirement therefore follows the last fence
          // fulfillment but precedes the final send, without elapsed delays.
          void read.promise.then(() => queueMicrotask(() => queueMicrotask(() => {
            old.server.dispose();
            retired.resolve();
          })));
          entered.resolve();
          return read.promise;
        };
      }
      return undefined;
    };
    operation = fixture.service.execute(OFFICIAL_ID, "read_issue", { edge: "last-fence" }, old.source.assertCurrent);
    await entered.promise;
    assert.equal(officialCallEntries(fixture.calls).length, 0);
    assert.equal(old.source.assertCurrent(), undefined);
    read.resolve(saved);
    await retired.promise;
    assertOriginRejected(await operation);
    assert.deepEqual(fixture.calls.map(call => call.payload.method), [
      "initialize", "notifications/initialized", "tools/list", "initialize", "notifications/initialized",
    ]);
    fixture.controls.provider = undefined;
    assertOfficialDispatchSuccess(await fixture.service.execute(OFFICIAL_ID, "read_issue", {}, fresh.source.assertCurrent));
    assert.equal(officialCallEntries(fixture.calls).length, 1);
  } finally {
    read.resolve(saved);
    await Promise.allSettled(operation == null ? [] : [operation]);
  }
});

test("OF-T2 official payload serialization retires the source before the final fetch guard", officialDispatchOptions, async t => {
  const fixture = officialDispatchServiceFixture(t);
  const old = nativeRoutedSource(t);
  const fresh = nativeRoutedSource(t);
  await fixture.ready();
  let serializations = 0;
  const args = { toJSON() {
    serializations += 1;
    old.server.dispose();
    return { nested: { original: [false, null, 7] } };
  } };
  assertOriginRejected(await fixture.service.execute(OFFICIAL_ID, "read_issue", args, old.source.assertCurrent));
  assert.equal(serializations, 1);
  assert.equal(officialCallEntries(fixture.calls).length, 0);
  assert.equal(fixture.calls.length, 5, "only the completed inventory and call preparation entered fetch");
  assertOfficialDispatchSuccess(await fixture.service.execute(OFFICIAL_ID, "read_issue", { current: true }, fresh.source.assertCurrent));
  assert.equal(officialCallEntries(fixture.calls).length, 1);
});


test("OF-T3 default official concurrent sources keep separate assertions and refusal witnesses", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const old = nativeRoutedSource(t);
  const current = nativeRoutedSource(t);
  const entered = deferred<void>();
  const release = deferred<void>();
  let operation: Promise<unknown> | undefined;
  try {
    await prepareOfficialDispatch(fixture);
    const requestA = await officialDispatchRequest(fixture, old.source, "read_issue", { invocation: "A" });
    const requestB = await officialDispatchRequest(fixture, current.source, "read_issue", { invocation: "B" });
    assert.equal(requestA.origin.mcpAttemptId, requestB.origin.mcpAttemptId);
    fixture.controls.nextRead = async () => { entered.resolve(); await release.promise; };
    operation = fixture.adapter.executeRoutedTool(requestA, old.source);
    await entered.promise;
    old.server.dispose();
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(requestB, current.source));
    release.resolve();
    assertOriginRejected(await operation);
    assert.deepEqual(officialCallEntries(fixture.provider).map(call => call.payload.params),
      [{ name: "read_issue", arguments: { invocation: "B" } }]);
    assert.equal(current.source.assertCurrent(), undefined);
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool({ ...requestB, args: { invocation: "B-again" } }, current.source));
    assert.deepEqual(officialCallEntries(fixture.provider).map(call => call.payload.params.arguments.invocation), ["B", "B-again"]);
  } finally {
    release.resolve();
    await Promise.allSettled(operation == null ? [] : [operation]);
    await fixture.dispose();
  }
});

for (const failure of ["owner-typed", "matching-text", "credential-typed", "serialization-typed", "entered-fetch-typed"] as const) {
  test("OF-T3 official refusal provenance distinguishes " + failure, officialDispatchOptions, async t => {
    const fixture = officialDispatchServiceFixture(t);
    const origin = nativeRoutedSource(t);
    await fixture.ready();
    let assertions = 0;
    const assertion: McpDispatchAssertion = () => {
      assertions += 1;
      if (failure === "matching-text") throw new Error(MCP_ORIGIN_RETIRED);
      return origin.source.assertCurrent();
    };
    let args: unknown = { invocation: failure };
    if (failure === "owner-typed") origin.server.dispose();
    if (failure === "credential-typed") {
      fixture.controls.nextRead = () => { throw new McpOriginRetiredError(); };
    }
    if (failure === "serialization-typed") {
      args = { toJSON() { throw new McpOriginRetiredError(); } };
    }
    if (failure === "entered-fetch-typed") {
      fixture.controls.provider = call => {
        if (call.payload.method === "tools/call") throw new McpOriginRetiredError();
        return undefined;
      };
    }
    const result = await fixture.service.execute(OFFICIAL_ID, "read_issue", args, assertion);
    if (failure === "owner-typed") assertOriginRejected(result);
    else assertOfficialDispatchError(result);
    assert.equal(officialCallEntries(fixture.calls).length, failure === "entered-fetch-typed" ? 1 : 0);
    assert.equal(assertions, failure === "credential-typed" ? 0
      : failure === "serialization-typed" ? 5 : failure === "entered-fetch-typed" ? 6 : 1);
    assert.equal(fixture.calls.length, failure === "serialization-typed" ? 5 : failure === "entered-fetch-typed" ? 6 : 0,
      "only the owner's final typed assertion proves a known pre-call refusal");
    fixture.controls.provider = undefined;
    const fresh = nativeRoutedSource(t);
    const before = officialCallEntries(fixture.calls).length;
    assertOfficialDispatchSuccess(await fixture.service.execute(OFFICIAL_ID, "read_issue", { recovered: true }, fresh.source.assertCurrent));
    assert.equal(officialCallEntries(fixture.calls).length, before + 1, "a refusal witness never leaks into the next invocation");
  });
}

for (const outcome of ["success", "server-error", "transport-failure", "http-failure", "jsonrpc-failure"] as const) {
  test("OF-T4 default official entered " + outcome + " retains custody after source-only retirement", officialDispatchOptions, async t => {
    const fixture = officialProductionFixture(t);
    const old = nativeRoutedSource(t);
    const entered = deferred<OfficialProviderCall>();
    const response = deferred<Response>();
    let operation: Promise<unknown> | undefined;
    try {
      await prepareOfficialDispatch(fixture);
      const request = await officialDispatchRequest(fixture, old.source);
      const vaultBefore = [...fixture.vault.entries()];
      const writesBefore = fixture.writes.length;
      const brokerBefore = fixture.broker.length;
      const start = fixture.provider.length;
      fixture.controls.provider = call => {
        if (call.payload.method === "tools/call") { entered.resolve(call); return response.promise; }
        return undefined;
      };
      operation = fixture.adapter.executeRoutedTool(request, old.source);
      const call = await entered.promise;
      assert.equal(officialCallEntries(fixture.provider).length, 1);
      assert.deepEqual(call.payload.params, { name: request.toolName, arguments: request.args });
      old.server.dispose();
      assert.equal(call.init.signal?.aborted, false, "retiring the source does not dispose the current Official service");
      if (outcome === "transport-failure") response.reject(new Error("lost provider reply " + OFFICIAL_DISPATCH_TOKEN));
      else if (outcome === "http-failure") response.resolve(new Response(OFFICIAL_DISPATCH_TOKEN, { status: 503 }));
      else if (outcome === "jsonrpc-failure") response.resolve(fixture.json({
        jsonrpc: "2.0", id: call.payload.id, error: { code: -32000, message: OFFICIAL_DISPATCH_TOKEN },
      }));
      else response.resolve(fixture.json({
        jsonrpc: "2.0", id: call.payload.id,
        result: { content: [{ type: "text", text: "original private result " + OFFICIAL_DISPATCH_TOKEN }],
          isError: outcome === "server-error" },
      }));
      const result = await operation;
      if (outcome === "success" || outcome === "server-error") {
        assertOfficialDispatchSuccess(result, outcome === "server-error");
        assert.match(JSON.stringify(result.toJson()), /original private result \[redacted\]/);
      } else assertOfficialDispatchError(result);
      assert.ok(result instanceof McpResult);
      assert.doesNotMatch(JSON.stringify(result.toJson()), new RegExp(OFFICIAL_DISPATCH_TOKEN));
      assert.deepEqual(fixture.provider.slice(start).map(entry => entry.payload.method),
        ["initialize", "notifications/initialized", "tools/list", "initialize", "notifications/initialized", "tools/call"],
        "entered settlement does not retry or start replacement preparation");
      assert.equal(officialCallEntries(fixture.provider).length, 1);
      assert.equal(fixture.writes.length, writesBefore);
      assert.equal(fixture.broker.length, brokerBefore);
      assert.deepEqual([...fixture.vault.entries()], vaultBefore);
      fixture.controls.provider = undefined;
      const fresh = nativeRoutedSource(t);
      const current = await officialDispatchRequest(fixture, fresh.source);
      assert.equal(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
      assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
      assert.equal(officialCallEntries(fixture.provider).length, 2, "only an explicit fresh invocation adds another call");
    } finally {
      response.resolve(fixture.json({ jsonrpc: "2.0", id: 2, result: { content: [] } }));
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}

for (const fence of ["account-switch", "uninstall", "credential-revision", "service-reset"] as const) {
  test("OF-T5 default official " + fence + " suppresses private output after tools/call entry", officialDispatchOptions, async t => {
    const fixture = officialProductionFixture(t);
    const source = nativeRoutedSource(t);
    const entered = deferred<OfficialProviderCall>();
    const response = deferred<Response>();
    let operation: Promise<unknown> | undefined;
    try {
      await prepareOfficialDispatch(fixture);
      const request = await officialDispatchRequest(fixture, source.source);
      fixture.controls.provider = call => {
        if (call.payload.method === "tools/call") { entered.resolve(call); return response.promise; }
        return undefined;
      };
      operation = fixture.adapter.executeRoutedTool(request, source.source);
      const call = await entered.promise;
      assert.equal(officialCallEntries(fixture.provider).length, 1);
      if (fence === "account-switch") fixture.switchAccount();
      else if (fence === "uninstall") await fixture.adapter.uninstallHostMcpPlugin(OFFICIAL_ID);
      else if (fence === "credential-revision") await fixture.install({ ACCESS_TOKEN: "official-replacement-provider-token" });
      else await fixture.adapter.resetMcpManager();
      assert.equal(source.source.assertCurrent(), undefined, "these controls exercise the independent account/install/service fences");
      assert.equal(call.init.signal?.aborted, fence === "service-reset");
      response.resolve(fixture.json({
        jsonrpc: "2.0", id: call.payload.id,
        result: { content: [{ type: "text", text: "must-suppress-private-official-output" }],
          structuredContent: { private: "must-suppress-private-structured-output" } },
      }));
      const result = await operation;
      assertOfficialDispatchError(result);
      assert.ok(result instanceof McpResult);
      assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-private/);
      assert.equal(officialCallEntries(fixture.provider).length, 1, "fencing cannot authorize replay");
      fixture.controls.provider = undefined;
      await fixture.install({ ACCESS_TOKEN: "official-replacement-provider-token" });
      const fresh = nativeRoutedSource(t);
      const current = await officialDispatchRequest(fixture, fresh.source);
      if (fence === "service-reset") assert.notEqual(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
      else assert.equal(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
      assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
      assert.equal(officialCallEntries(fixture.provider).length, 2);
      assert.equal(new Headers(officialCallEntries(fixture.provider)[1]!.init.headers).get("Authorization"),
        "Bearer official-replacement-provider-token");
    } finally {
      response.resolve(fixture.json({ jsonrpc: "2.0", id: 2, result: { content: [] } }));
      await Promise.allSettled(operation == null ? [] : [operation]);
      await fixture.dispose();
    }
  });
}

test("OF-T5 default official dispatch retains raw disable, provider credential, protocol and redaction contracts", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const source = nativeRoutedSource(t);
  try {
    await prepareOfficialDispatch(fixture);
    const disabled = await officialDispatchRequest(fixture, source.source);
    const [path, value] = [...fixture.vault.entries()][0]!;
    const stored = JSON.parse(value);
    // The native vault I/O supplies a persisted disabled record. The real
    // Official service still owns both listing filtration and call admission.
    stored[OFFICIAL_ID].disabledTools = ["read_issue"];
    stored[OFFICIAL_ID].revision = "official-disabled-fixture-revision";
    fixture.vault.set(path, JSON.stringify(stored));
    const before = fixture.provider.length;
    assertOfficialDispatchError(await fixture.adapter.executeRoutedTool(disabled, source.source));
    assert.equal(fixture.provider.length, before, "a retained descriptor cannot bypass its raw-name disable record");
    assert.equal(officialCallEntries(fixture.provider).length, 0);
    const catalog = await fixture.adapter.listRoutedTools(source.source);
    assert.ok(Array.isArray(catalog));
    assert.deepEqual(catalog.filter((row: any) => row.providerIdentifier === OFFICIAL_ID).map((row: any) => row.toolName), ["write_issue"]);
    const args = { nested: { list: [null, false, 3.25, { literal: "unchanged" }] }, url: "https://untrusted.example/argument-only" };
    const current = await officialDispatchRequest(fixture, source.source, "write_issue", args);
    assert.equal(current.origin.mcpAttemptId, disabled.origin.mcpAttemptId);
    fixture.controls.provider = call => call.payload.method === "tools/call" ? fixture.json({
      jsonrpc: "2.0", id: call.payload.id,
      result: { content: [{ type: "text", text: "echo " + OFFICIAL_DISPATCH_TOKEN }],
        structuredContent: { [OFFICIAL_DISPATCH_TOKEN]: { value: OFFICIAL_DISPATCH_TOKEN } } },
    }) : undefined;
    const result = await fixture.adapter.executeRoutedTool(current, source.source);
    assertOfficialDispatchSuccess(result);
    assert.equal(officialCallEntries(fixture.provider).length, 1);
    assert.deepEqual(officialCallEntries(fixture.provider)[0]!.payload, {
      jsonrpc: "2.0", id: 2, method: "tools/call", params: { name: "write_issue", arguments: args },
    });
    assert.doesNotMatch(JSON.stringify(result.toJson()), new RegExp(OFFICIAL_DISPATCH_TOKEN));
    assert.match(JSON.stringify(result.toJson()), /\[redacted\]/);
    for (const call of fixture.provider) {
      assert.equal(call.url, "https://api.githubcopilot.com/mcp/");
      assert.equal(call.init.method, "POST");
      assert.equal(call.init.redirect, "error");
      const headers = new Headers(call.init.headers);
      assert.equal(headers.get("Authorization"), "Bearer " + OFFICIAL_DISPATCH_TOKEN);
      assert.notEqual(headers.get("Authorization"), "Bearer " + fixture.token());
      if (call.payload.method !== "initialize") {
        assert.equal(headers.get("Mcp-Session-Id"), "official-dispatch-session");
        assert.equal(headers.get("MCP-Protocol-Version"), "2025-03-26");
      }
    }
    assert.deepEqual(fixture.broker, [], "manual provider credentials do not authorize a broker request");
  } finally {
    await fixture.dispose();
  }
});

test("OF-T5 default official reset during its final output read suppresses private settlement", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const source = nativeRoutedSource(t);
  const entered = deferred<void>();
  const release = deferred<void>();
  let postResponseReads = 0;
  let operation: Promise<unknown> | undefined;
  try {
    await prepareOfficialDispatch(fixture);
    const request = await officialDispatchRequest(fixture, source.source);
    const start = fixture.provider.length;
    const vaultBefore = [...fixture.vault.entries()];
    fixture.controls.provider = call => {
      if (call.payload.method !== "tools/call") return undefined;
      // The first read after this response belongs to request()'s final
      // fence. Arm the next hook there so only execute()'s output fence
      // is held, after request() has removed its controller from pending.
      fixture.controls.nextRead = () => {
        postResponseReads += 1;
        fixture.controls.nextRead = () => {
          postResponseReads += 1;
          entered.resolve();
          return release.promise;
        };
      };
      return fixture.json({
        jsonrpc: "2.0", id: call.payload.id,
        result: {
          content: [{ type: "text", text: "must-suppress-final-read-private-content" }],
          structuredContent: { private: "must-suppress-final-read-private-structured" },
        },
      });
    };
    operation = fixture.adapter.executeRoutedTool(request, source.source);
    await entered.promise;
    assert.equal(postResponseReads, 2);
    assert.equal(officialCallEntries(fixture.provider).length, 1);
    const call = officialCallEntries(fixture.provider)[0]!;
    assert.deepEqual(call.payload.params, { name: request.toolName, arguments: request.args });
    assert.ok(call.init.signal);
    assert.equal(call.init.signal.aborted, false);

    await fixture.adapter.resetMcpManager();
    assert.equal(source.source.assertCurrent(), undefined, "the native source itself remains current");
    assert.equal(call.init.signal.aborted, false,
      "request() already removed this completed call from pending; abort cannot mask the output fence");
    assert.deepEqual([...fixture.vault.entries()], vaultBefore,
      "reset leaves the original matching install revision intact");
    // The existing native read fixture captured its text before entering
    // this gate, so release returns the original matching revision.
    release.resolve();
    const result = await operation;
    assertOfficialDispatchError(result);
    assert.ok(result instanceof McpResult);
    assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-final-read-private/);
    assert.equal(postResponseReads, 2);
    assert.equal(officialCallEntries(fixture.provider).length, 1);
    assert.deepEqual(fixture.provider.slice(start).map(entry => entry.payload.method), [
      "initialize", "notifications/initialized", "tools/list",
      "initialize", "notifications/initialized", "tools/call",
    ], "late suppression cannot retry or prepare a replacement invocation");
    assert.equal(call.init.signal.aborted, false);

    fixture.controls.provider = undefined;
    const fresh = nativeRoutedSource(t);
    const current = await officialDispatchRequest(fixture, fresh.source);
    assert.notEqual(current.origin.coordinatorSourceId, request.origin.coordinatorSourceId);
    assert.notEqual(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
    assert.equal(officialCallEntries(fixture.provider).length, 1,
      "constructing and listing the replacement does not replay the original call");
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
    assert.equal(officialCallEntries(fixture.provider).length, 2,
      "only the explicit fresh invocation enters the replacement provider call");
    assert.deepEqual(officialCallEntries(fixture.provider)[1]!.payload.params,
      { name: current.toolName, arguments: current.args });
  } finally {
    release.resolve();
    await Promise.allSettled(operation == null ? [] : [operation]);
    await fixture.dispose();
  }
});

test("OF-T5 official disposal after final fence fulfillment suppresses private settlement", officialDispatchOptions, async t => {
  const fixture = officialDispatchServiceFixture(t);
  const source = nativeRoutedSource(t);
  const entered = deferred<void>();
  const read = deferred<string | null>();
  const disposed = deferred<void>();
  let saved: string | null = null;
  let postResponseReads = 0;
  let operation: Promise<unknown> | undefined;
  try {
    await fixture.ready();
    fixture.controls.provider = call => {
      if (call.payload.method !== "tools/call") return undefined;
      fixture.controls.nextRead = value => {
        postResponseReads += 1;
        fixture.controls.nextRead = value => {
          postResponseReads += 1;
          saved = value;
          // Register before state() awaits this exact port promise.
          // The first queued microtask follows state's continuation;
          // the second follows fence's continuation and precedes the
          // resumed execute(). A check only inside fence() is too early.
          void read.promise.then(() => queueMicrotask(() => queueMicrotask(() => {
            fixture.service.dispose();
            disposed.resolve();
          })));
          entered.resolve();
          return read.promise;
        };
        return Promise.resolve(value);
      };
      return new Response(JSON.stringify({
        jsonrpc: "2.0", id: call.payload.id,
        result: {
          content: [{ type: "text", text: "must-suppress-final-continuation-private-content" }],
          structuredContent: { private: "must-suppress-final-continuation-private-structured" },
        },
      }), { headers: { "Content-Type": "application/json" } });
    };
    operation = fixture.service.execute(
      OFFICIAL_ID, "read_issue", { edge: "final-output-continuation" }, source.source.assertCurrent,
    );
    await entered.promise;
    assert.equal(postResponseReads, 2);
    assert.equal(typeof saved, "string");
    assert.equal(source.source.assertCurrent(), undefined);
    assert.equal(officialCallEntries(fixture.calls).length, 1);
    const call = officialCallEntries(fixture.calls)[0]!;
    assert.deepEqual(call.payload.params,
      { name: "read_issue", arguments: { edge: "final-output-continuation" } });
    assert.ok(call.init.signal);
    assert.equal(call.init.signal.aborted, false);

    read.resolve(saved);
    await disposed.promise;
    assert.equal(source.source.assertCurrent(), undefined,
      "only the Official service lifetime ended; source currentness is not the output fence");
    assert.equal(call.init.signal.aborted, false,
      "the completed request controller is no longer pending when service disposal runs");
    const result = await operation;
    assertOfficialDispatchError(result);
    assert.ok(result instanceof McpResult);
    assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-final-continuation-private/);
    assert.equal(postResponseReads, 2);
    assert.equal(officialCallEntries(fixture.calls).length, 1);
    assert.deepEqual(fixture.calls.map(entry => entry.payload.method), [
      "initialize", "notifications/initialized", "tools/list",
      "initialize", "notifications/initialized", "tools/call",
    ], "output suppression preserves the entered call without replay or additional provider preparation");
    assert.equal(call.init.signal.aborted, false);
  } finally {
    read.resolve(saved);
    await Promise.allSettled(operation == null ? [] : [operation]);
  }
});


test("OF-T6 default official account departure during its final output read suppresses private settlement", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const source = nativeRoutedSource(t);
  const entered = deferred<void>();
  const release = deferred<void>();
  const transitionEntered = deferred<void>();
  const transitionRelease = deferred<void>();
  let postResponseReads = 0;
  let resets = 0;
  let operation: Promise<unknown> | undefined;
  try {
    await prepareOfficialDispatch(fixture);
    const request = await officialDispatchRequest(fixture, source.source);
    const bindingA = fixture.account.owner.captureCurrentBinding();
    assert.ok(bindingA);
    const start = fixture.provider.length;
    const vaultBefore = [...fixture.vault.entries()];
    fixture.account.controls.transition = async transition => {
      assert.deepEqual(transition, { previousSlot: "official-lifetime-a", nextSlot: "official-lifetime-b" });
      transitionEntered.resolve();
      await transitionRelease.promise;
      await fixture.adapter.resetMcpManager();
      resets += 1;
    };
    fixture.controls.provider = call => {
      if (call.payload.method !== "tools/call") return undefined;
      fixture.controls.nextRead = () => {
        postResponseReads += 1;
        fixture.controls.nextRead = () => {
          postResponseReads += 1;
          entered.resolve();
          return release.promise;
        };
      };
      return fixture.json({
        jsonrpc: "2.0", id: call.payload.id,
        result: {
          content: [{ type: "text", text: "must-suppress-account-private-content" }],
          structuredContent: { private: "must-suppress-account-private-structured" },
        },
      });
    };
    operation = fixture.adapter.executeRoutedTool(request, source.source);
    await entered.promise;
    assert.equal(postResponseReads, 2);
    assert.equal(officialCallEntries(fixture.provider).length, 1);
    const call = officialCallEntries(fixture.provider)[0]!;
    assert.ok(call.init.signal);
    fixture.switchAccount();
    assert.throws(() => bindingA.assertCurrent(), /no longer current/);
    assert.equal(fixture.account.owner.captureCurrentBinding(), null,
      "observe retires A synchronously before queued transition work");
    await transitionEntered.promise;
    assert.equal(resets, 0, "account cleanup is held before the actual MCP reset");
    assert.equal(source.source.assertCurrent(), undefined);
    assert.equal(call.init.signal.aborted, false);
    assert.deepEqual([...fixture.vault.entries()], vaultBefore);

    release.resolve();
    const result = await operation;
    assertOfficialDispatchError(result);
    assert.ok(result instanceof McpResult);
    assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-account-private/);
    assert.equal(resets, 0, "the still-live service must enforce the retired account binding");
    assert.equal(postResponseReads, 2);
    assert.equal(call.init.signal.aborted, false,
      "the completed request is no longer pending and no service disposal masked account retirement");
    assert.equal(officialCallEntries(fixture.provider).length, 1);
    assert.deepEqual(fixture.provider.slice(start).map(entry => entry.payload.method), [
      "initialize", "notifications/initialized", "tools/list",
      "initialize", "notifications/initialized", "tools/call",
    ], "late account suppression does not retry or prepare another invocation");

    fixture.controls.provider = undefined;
    transitionRelease.resolve();
    await fixture.account.owner.whenIdle();
    assert.equal(resets, 1);
    assert.throws(() => bindingA.assertCurrent(), /no longer current/);
    await fixture.install({ ACCESS_TOKEN: "official-account-b-provider-token" });
    const fresh = nativeRoutedSource(t);
    const current = await officialDispatchRequest(fixture, fresh.source);
    assert.notEqual(current.origin.mcpAttemptId, request.origin.mcpAttemptId);
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
    assert.equal(officialCallEntries(fixture.provider).length, 2);
    assert.equal(new Headers(officialCallEntries(fixture.provider)[1]!.init.headers).get("Authorization"),
      "Bearer official-account-b-provider-token");
    assert.deepEqual(fixture.account.authorizations, ["official-lifetime-a", "official-lifetime-b"]);
  } finally {
    release.resolve();
    transitionRelease.resolve();
    await Promise.allSettled(operation == null ? [] : [operation]);
    await fixture.account.owner.whenIdle();
    await fixture.dispose();
  }
});

test("OF-T6 official account retirement after final fence fulfillment suppresses private settlement", officialDispatchOptions, async t => {
  const account = officialAccountOwnerFixture(t);
  const fixture = officialDispatchServiceFixture(t, account.captureOutputAccountBinding);
  const source = nativeRoutedSource(t);
  const entered = deferred<void>();
  const read = deferred<string | null>();
  const retired = deferred<void>();
  const transitionRelease = deferred<void>();
  let saved: string | null = null;
  let postResponseReads = 0;
  let operation: Promise<unknown> | undefined;
  try {
    await account.ready;
    await fixture.ready();
    account.controls.transition = async () => { await transitionRelease.promise; };
    fixture.controls.provider = call => {
      if (call.payload.method !== "tools/call") return undefined;
      fixture.controls.nextRead = value => {
        postResponseReads += 1;
        fixture.controls.nextRead = value => {
          postResponseReads += 1;
          saved = value;
          // As in OF-T2, state -> fence -> execute are the real await
          // continuations. Observe after fence fulfills but before execute
          // resumes; do not dispose the service or retire the native source.
          void read.promise.then(() => queueMicrotask(() => queueMicrotask(() => {
            account.observe("official-lifetime-b");
            retired.resolve();
          })));
          entered.resolve();
          return read.promise;
        };
        return Promise.resolve(value);
      };
      return new Response(JSON.stringify({
        jsonrpc: "2.0", id: call.payload.id,
        result: {
          content: [{ type: "text", text: "must-suppress-account-continuation-private-content" }],
          structuredContent: { private: "must-suppress-account-continuation-private-structured" },
        },
      }), { headers: { "Content-Type": "application/json" } });
    };
    operation = fixture.service.execute(OFFICIAL_ID, "read_issue", { edge: "account-final-continuation" }, source.source.assertCurrent);
    assert.equal(account.captures.length, 1, "capture occurs before execute's first await");
    await entered.promise;
    assert.equal(postResponseReads, 2);
    const call = officialCallEntries(fixture.calls)[0]!;
    assert.ok(call.init.signal);
    read.resolve(saved);
    await retired.promise;
    assert.equal(source.source.assertCurrent(), undefined);
    assert.throws(() => account.captures[0]!.assertCurrent(), /no longer current/);
    const result = await operation;
    assertOfficialDispatchError(result);
    assert.ok(result instanceof McpResult);
    assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-account-continuation-private/);
    assert.equal(account.captures.length, 1, "publication cannot recapture the account owner");
    assert.equal(call.init.signal.aborted, false);
    assert.equal(officialCallEntries(fixture.calls).length, 1);
    assert.deepEqual(fixture.calls.map(entry => entry.payload.method), [
      "initialize", "notifications/initialized", "tools/list",
      "initialize", "notifications/initialized", "tools/call",
    ]);

    fixture.controls.provider = undefined;
    transitionRelease.resolve();
    await account.owner.whenIdle();
    assertOfficialDispatchSuccess(await fixture.service.execute(
      OFFICIAL_ID, "read_issue", { fresh: "account-b" }, source.source.assertCurrent,
    ));
    assert.equal(account.captures.length, 2);
    assert.equal(officialCallEntries(fixture.calls).length, 2,
      "the same undisposed service accepts only the explicit fresh-account invocation");
  } finally {
    read.resolve(saved);
    transitionRelease.resolve();
    await Promise.allSettled(operation == null ? [] : [operation]);
    await account.owner.whenIdle();
  }
});

for (const replacement of ["B", "A-after-B"] as const) {
  test("OF-T6 official captured account A cannot borrow " + replacement + " at private output publication", officialDispatchOptions, async t => {
    const account = officialAccountOwnerFixture(t);
    const fixture = officialDispatchServiceFixture(t, account.captureOutputAccountBinding);
    const source = nativeRoutedSource(t);
    const entered = deferred<void>();
    const read = deferred<string | null>();
    let saved: string | null = null;
    let postResponseReads = 0;
    let operation: Promise<unknown> | undefined;
    try {
      await account.ready;
      await fixture.ready();
      fixture.controls.provider = call => {
        if (call.payload.method !== "tools/call") return undefined;
        fixture.controls.nextRead = value => {
          postResponseReads += 1;
          fixture.controls.nextRead = value => {
            postResponseReads += 1;
            saved = value;
            entered.resolve();
            return read.promise;
          };
          return Promise.resolve(value);
        };
        return new Response(JSON.stringify({
          jsonrpc: "2.0", id: call.payload.id,
          result: {
            content: [{ type: "text", text: "must-suppress-replaced-account-private-content" }],
            structuredContent: { private: "must-suppress-replaced-account-private-structured" },
          },
        }), { headers: { "Content-Type": "application/json" } });
      };
      operation = fixture.service.execute(OFFICIAL_ID, "read_issue", { original: "account-a" }, source.source.assertCurrent);
      assert.equal(account.captures.length, 1, "capture precedes every asynchronous credential operation");
      await entered.promise;
      assert.equal(postResponseReads, 2);
      account.observe("official-lifetime-b");
      await account.owner.whenIdle();
      if (replacement === "A-after-B") {
        account.observe("official-lifetime-a");
        await account.owner.whenIdle();
      }
      const currentBinding = account.owner.captureCurrentBinding();
      assert.ok(currentBinding);
      currentBinding.assertCurrent();
      assert.notEqual(currentBinding.runtimeIdentity, account.captures[0]!.runtimeIdentity);
      assert.throws(() => account.captures[0]!.assertCurrent(), /no longer current/);
      assert.equal(source.source.assertCurrent(), undefined);
      const call = officialCallEntries(fixture.calls)[0]!;
      assert.ok(call.init.signal);
      assert.equal(call.init.signal.aborted, false);
      // Native vault/scope inputs remain the captured A snapshot. Only the
      // existing account lifetime distinguishes replacement from continuity.
      read.resolve(saved);
      const result = await operation;
      assertOfficialDispatchError(result);
      assert.ok(result instanceof McpResult);
      assert.doesNotMatch(JSON.stringify(result.toJson()), /must-suppress-replaced-account-private/);
      assert.equal(account.captures.length, 1, "late publication never acquires the replacement binding");
      assert.equal(officialCallEntries(fixture.calls).length, 1);
      assert.equal(fixture.calls.length, 6);
      assert.equal(call.init.signal.aborted, false);

      fixture.controls.provider = undefined;
      assertOfficialDispatchSuccess(await fixture.service.execute(
        OFFICIAL_ID, "read_issue", { fresh: replacement }, source.source.assertCurrent,
      ));
      assert.equal(account.captures.length, 2);
      assert.equal(officialCallEntries(fixture.calls).length, 2);
      assert.deepEqual(account.authorizations, replacement === "B"
        ? ["official-lifetime-a", "official-lifetime-b"]
        : ["official-lifetime-a", "official-lifetime-b", "official-lifetime-a"]);
    } finally {
      read.resolve(saved);
      await Promise.allSettled(operation == null ? [] : [operation]);
      await account.owner.whenIdle();
    }
  });
}

test("OF-T6 default official refuses execution while the canonical account binding is unavailable", officialDispatchOptions, async t => {
  const fixture = officialProductionFixture(t);
  const source = nativeRoutedSource(t);
  const entered = deferred<void>();
  const release = deferred<void>();
  try {
    await prepareOfficialDispatch(fixture);
    const request = await officialDispatchRequest(fixture, source.source);
    fixture.account.controls.transition = async () => { entered.resolve(); await release.promise; };
    fixture.switchAccount();
    await entered.promise;
    assert.equal(fixture.account.owner.captureCurrentBinding(), null);
    assert.equal(source.source.assertCurrent(), undefined);
    const before = { ...fixture.calls, reads: fixture.reads.length, provider: fixture.provider.length };
    assertOfficialDispatchError(await fixture.adapter.executeRoutedTool(request, source.source));
    assert.deepEqual({ ...fixture.calls, reads: fixture.reads.length, provider: fixture.provider.length }, before,
      "missing canonical binding fails before credential I/O or any provider preparation");
    assert.equal(officialCallEntries(fixture.provider).length, 0);
    release.resolve();
    await fixture.account.owner.whenIdle();
    await fixture.install({ ACCESS_TOKEN: "official-account-b-provider-token" });
    const fresh = nativeRoutedSource(t);
    const current = await officialDispatchRequest(fixture, fresh.source);
    assert.equal(current.origin.mcpAttemptId, request.origin.mcpAttemptId,
      "this missing-binding control never disposes or replaces the MCP service");
    assertOfficialDispatchSuccess(await fixture.adapter.executeRoutedTool(current, fresh.source));
    assert.equal(officialCallEntries(fixture.provider).length, 1);
  } finally {
    release.resolve();
    await fixture.account.owner.whenIdle();
    await fixture.dispose();
  }
});
