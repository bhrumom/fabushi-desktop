import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

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
import { createProductionMcpOAuthAdapter, createProductionMcpOAuthRootPortProvider } from "../adapters/mcp-oauth.js";
import type { ProductionServiceContext } from "../main-production-services.js";
import type { DesktopMcpManagerFacade } from "./desktop-mcp-manager.js";
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
    async listRoutedTools() { calls.list += 1; return ["current-tool"]; },
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
  return createProductionMcpOAuthAdapter({
    resolveRuntimeDeps: () => ({
      ...deps,
      // This controlled factory implements only the facade methods exercised
      // through the real adapter; unused desktop operations must never run.
      createManager: async (options) => await deps.createManager(options) as DesktopMcpManagerFacade,
    }),
    resolveDesktopDeps: () => { throw new Error("Desktop IPC is outside this routed-operation test"); },
  }).create({ machineId: "construction-machine" } as ProductionServiceContext);
}

for (const action of ["reset", "dispose"] as const) {
  test("MC-06 shipping adapter " + action + " rejects pending routed operations before facade I/O", async () => {
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
    const listRejected = assert.rejects(adapter.listRoutedTools(), /construction is retired/);
    const executeRejected = assert.rejects(adapter.executeRoutedTool(request), /construction is retired/);
    await started.promise;
    assert.equal(factories, 1);
    if (action === "reset") {
      await adapter.resetMcpManager();
      assert.deepEqual(await adapter.listRoutedTools(), ["current-tool"]);
      assert.equal(await adapter.executeRoutedTool(request), "executed");
    } else {
      await adapter.dispose();
      await assert.rejects(adapter.listRoutedTools(), /adapter is disposed/);
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

test("MC-03 shipping adapter awaits and propagates manager teardown failure", async () => {
  const started = deferred<void>();
  const cleanup = deferred<void>();
  const failure = new Error("Shutdown cleanup failed");
  const owned = constructionManager({ dispose: () => {
    started.resolve();
    return cleanup.promise;
  } });
  const adapter = constructionAdapter(constructionDeps(async () => owned.manager));
  assert.deepEqual(await adapter.listRoutedTools(), ["current-tool"]);
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
  test("MB-04 shipping adapter " + action + " refuses a facade paused before Box dispatch", async () => {
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
    const request = { providerIdentifier: "connector", name: "tool", toolName: "tool", args: {}, toolCallId: "box-retirement-call" };
    const refused = assert.rejects(adapter.executeRoutedTool(request), /construction is retired/);
    await started.promise;
    assert.equal(oldFacadeCalls, 1);
    assert.deepEqual(rawCalls, []);
    if (action === "reset") {
      await adapter.resetMcpManager();
      assert.equal(await adapter.executeRoutedTool(request), "result-bbbb");
    } else {
      await adapter.dispose();
      await assert.rejects(adapter.executeRoutedTool(request), /adapter is disposed/);
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
