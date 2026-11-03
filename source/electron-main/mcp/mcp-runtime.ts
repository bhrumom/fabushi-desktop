import type { ConnectorAuthReport } from "../../shared/observability/connector-auth-telemetry.js";

type McpManagerOptions = {
  readonly settingsStore: unknown;
  readonly onAccountScopeApplied: () => void;
  readonly getAccessToken: (args: { backendUrl: string }) => Promise<string | null>;
  readonly getMachineId: () => string | Promise<string>;
  readonly loadBoxMcpServers: (configJson: string) => Promise<unknown>;
  readonly listBoxMcpServers: (
    serverIdentifiers: unknown,
  ) => Promise<readonly Record<string, unknown>[]>;
  readonly listBoxMcpToolsRaw: (payloadHex: string) => Promise<string>;
  readonly executeBoxMcpToolRaw: (payloadHex: string) => Promise<string>;
  readonly onConnectorAuth: (report: ConnectorAuthReport) => void;
  readonly onMcpDiscoveryFailed: (report: { readonly errorClass: string; readonly elapsedMs: number; readonly servedStale: boolean }) => void;
  readonly onMcpDiagnostic: (failure: {
    readonly leg: string;
    readonly errorClass: string;
  }) => void;
};

type CursorAuthService = {
  readonly getValidAccessToken: (args: {
    readonly backendUrl: string;
  }) => Promise<string | null>;
};

type McpRuntimeDependencies<TManager> = {
  readonly createManager: (options: McpManagerOptions) => Promise<TManager>;
  readonly settingsStore: unknown;
  readonly pushBoxSecrets: () => Promise<unknown>;
  readonly ensureCursorAuthService: () => Promise<CursorAuthService>;
  readonly getMachineId: () => string | Promise<string>;
  readonly loadBoxMcpServers: (configJson: string) => Promise<unknown>;
  readonly listBoxMcpServers: (
    serverIdentifiers: unknown,
  ) => Promise<readonly Record<string, unknown>[]>;
  readonly listBoxMcpToolsRaw: (payloadHex: string) => Promise<string>;
  readonly executeBoxMcpToolRaw: (payloadHex: string) => Promise<string>;
  readonly reportConnectorAuth: (report: ConnectorAuthReport) => void;
  readonly reportMcpDiscoveryFailed: (report: { readonly errorClass: string; readonly elapsedMs: number; readonly servedStale: boolean }) => void;
  readonly reportDiagnostic: (leg: string, errorClass: string) => void;
  readonly cleanupLegacyAuth: (root: string) => Promise<unknown>;
  readonly sandRootDir: () => string;
  readonly reportFailure: (
    subsystem: string,
    leg: string,
    error: unknown,
  ) => void;
  readonly broadcast: (channel: string, completion: unknown) => void;
  readonly refreshHostMcp: (completion: unknown) => void;
};

export function createMcpRuntime<
  TManager extends {
    dispose(): Promise<void> | void;
    setAuthCompletionObserver(listener: (completion: unknown) => void): void;
  },
>(deps: McpRuntimeDependencies<TManager>) {
  type Attempt = {
    readonly promise: Promise<TManager>;
    manager?: TManager;
    cleanup?: Promise<void>;
  };
  let current: Attempt | undefined;
  let disposed = false;
  let shutdownAttempt: Attempt | undefined;

  const isCurrent = (attempt: Attempt): boolean => !disposed && current === attempt;
  function requireCurrent(attempt: Attempt): void {
    if (!isCurrent(attempt)) throw new Error("MCP manager construction is retired.");
  }

  function disposeAttempt(attempt: Attempt | undefined): Promise<void> {
    if (attempt?.manager == null) return Promise.resolve();
    if (attempt.cleanup != null) return attempt.cleanup;
    let resolve!: () => void;
    let reject!: (error: unknown) => void;
    // Reserve cleanup before calling external code: dispose may reenter us.
    attempt.cleanup = new Promise<void>((onResolve, onReject) => {
      resolve = onResolve;
      reject = onReject;
    });
    try {
      Promise.resolve(attempt.manager.dispose()).then(resolve, reject);
    } catch (error) {
      reject(error);
    }
    return attempt.cleanup;
  }

  async function construct(attempt: Attempt): Promise<TManager> {
    requireCurrent(attempt);
    const manager = await deps.createManager({
      settingsStore: deps.settingsStore,
      onAccountScopeApplied: () => {
        if (isCurrent(attempt)) void deps.pushBoxSecrets();
      },
      getAccessToken: async ({ backendUrl }) => {
        requireCurrent(attempt);
        const service = await deps.ensureCursorAuthService();
        requireCurrent(attempt);
        const token = await service.getValidAccessToken({ backendUrl });
        requireCurrent(attempt);
        return token;
      },
      getMachineId: deps.getMachineId,
      loadBoxMcpServers: async (configJson) => {
        requireCurrent(attempt);
        return deps.loadBoxMcpServers(configJson);
      },
      listBoxMcpServers: async (serverIdentifiers) => {
        requireCurrent(attempt);
        return deps.listBoxMcpServers(serverIdentifiers);
      },
      listBoxMcpToolsRaw: async (payloadHex) => {
        requireCurrent(attempt);
        return deps.listBoxMcpToolsRaw(payloadHex);
      },
      executeBoxMcpToolRaw: async (payloadHex) => {
        requireCurrent(attempt);
        // Once handed to Host, keep its actual result even after retirement.
        return deps.executeBoxMcpToolRaw(payloadHex);
      },
      onConnectorAuth: (report) => {
        if (isCurrent(attempt)) deps.reportConnectorAuth(report);
      },
      onMcpDiscoveryFailed: (report) => {
        if (isCurrent(attempt)) deps.reportMcpDiscoveryFailed(report);
      },
      onMcpDiagnostic: (failure) => {
        if (isCurrent(attempt)) deps.reportDiagnostic(failure.leg, failure.errorClass);
      },
    });
    attempt.manager = manager;
    requireCurrent(attempt);
    const root = deps.sandRootDir();
    requireCurrent(attempt);
    void deps
      .cleanupLegacyAuth(root)
      .catch((error: unknown) =>
        deps.reportFailure("mcp-auth-cleanup", "startup-scrub", error),
      );
    requireCurrent(attempt);
    manager.setAuthCompletionObserver((completion) => {
      if (!isCurrent(attempt)) return;
      deps.broadcast("sand:mcp-auth-event", completion);
      if (isCurrent(attempt)) deps.refreshHostMcp(completion);
    });
    requireCurrent(attempt);
    return manager;
  }

  function ensureMcpManager(): Promise<TManager> {
    if (disposed) return Promise.reject(new Error("MCP runtime is disposed."));
    if (current != null) return current.promise;
    let resolve!: (manager: TManager) => void;
    let reject!: (error: unknown) => void;
    const attempt: Attempt = {
      promise: new Promise<TManager>((onResolve, onReject) => {
        resolve = onResolve;
        reject = onReject;
      }),
    };
    current = attempt;
    // Defer invocation so even a synchronous factory reentry shares the
    // reserved attempt, and reset can refuse a factory that has not started.
    void Promise.resolve().then(async () => {
      try {
        const manager = await construct(attempt);
        // Setup can queue a reset before this final publication continuation.
        requireCurrent(attempt);
        resolve(manager);
      } catch (error) {
        if (current === attempt) current = undefined;
        try {
          await disposeAttempt(attempt);
        } catch (cleanupError) {
          reject(new AggregateError([error, cleanupError], "MCP construction and cleanup failed."));
          return;
        }
        reject(error);
      }
    });
    return attempt.promise;
  }

  function resetMcpManager(): Promise<void> {
    const attempt = current;
    current = undefined;
    // An unresolved factory remains owned by its construction promise. Do not
    // make account transition wait for it; its late manager is disposed there.
    return disposeAttempt(attempt);
  }

  return {
    ensureMcpManager,
    resetMcpManager,
    dispose: (): Promise<void> => {
      if (!disposed) {
        disposed = true;
        shutdownAttempt = current;
        current = undefined;
      }
      return disposeAttempt(shutdownAttempt);
    },
  };
}
