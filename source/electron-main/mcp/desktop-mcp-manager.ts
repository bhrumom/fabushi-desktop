import { DashboardService } from "../../packages/proto/generated/aiserver/v1/dashboard_connect.js";
import {
  McpArgs,
  McpResult,
  McpStateExecArgs,
  McpStateExecResult,
} from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import { reportDesktopEdgeFailure } from "../desktop-edge-failures.js";
import type { ConnectorAuthReport } from "../../shared/observability/connector-auth-telemetry.js";
import { createSandCursorBackendClient, getSandInferenceBackendUrl } from "../../shared/node/cursor-backend/cursor-inference.js";
import {
  createAccountMcpWriter,
  backfillUserPluginInstalls,
  fetchAccountMcpServers,
  fetchEffectiveUserPlugins,
  type AccountMcpClient,
  type AccountMcpDependencies,
} from "../../shared/node/cursor-backend/account-mcp.js";
import {
  createDashboardSandBackendMcpExec,
  type DashboardMcpExecClient,
} from "../../shared/node/cursor-backend/backend-mcp-exec.js";
import { pinMcpDiagnosticsReporter } from "../../shared/node/mcp/mcp-diagnostics.js";
import { SandMcpManager } from "../../shared/node/mcp/mcp-manager.js";
import { createMcpToolsDiscovery } from "../../shared/node/mcp/tools-discovery.js";
import { OfficialMcpService, type OfficialMcpPorts } from "./official-mcp-service.js";
import { officialMcpCatalogPlugins } from "../../shared/node/mcp/fabushi-official-catalog.js";
import { marketplacePluginToView } from "../../shared/node/mcp/mcp-marketplace.js";

export interface DesktopMcpManagerFacade {
  listServers(): Promise<unknown>;
  listEffectivePlugins(): Promise<unknown>;
  getCatalog(getAccessToken: unknown, options?: { readonly forceRefresh?: boolean }): Promise<unknown>;
  resolvePluginLogo(url: string): Promise<unknown>;
  installEntry(request: unknown, getAccessToken: unknown): Promise<unknown>;
  updatePluginInstall(request: unknown, getAccessToken: unknown): Promise<unknown>;
  addServer(request: { name: string; configJson: string }): Promise<unknown>;
  removeServer(serverId: string): Promise<unknown>;
  reloadServers(): Promise<unknown>;
  uninstallPlugin(pluginId: string): Promise<unknown>;
  authenticateServer(serverId: string, accountKey: string, requestingAgentId?: string | null, forceReauth?: boolean, trigger?: string | null): Promise<unknown>;
  logoutAccount(args: { serverId: string; accountKey: string }): Promise<unknown>;
  renameAccount(args: { serverId: string; accountKey: string; newAccountKey: string }): Promise<unknown>;
  removeAccount(args: { serverId: string; accountKey: string }): Promise<unknown>;
  setServerCustomInstructions(request: unknown): Promise<unknown>;
  listServerTools(serverId: string): Promise<unknown>;
  listRoutedTools(): Promise<unknown>;
  executeRoutedTool(request: {
    readonly providerIdentifier: string;
    readonly name: string;
    readonly toolName: string;
    readonly args: unknown;
    readonly toolCallId: string;
    readonly agentId?: string;
  }): Promise<unknown>;
  toggleMcpToolDisabled(request: unknown): Promise<unknown>;
  setAuthCompletionObserver(observer: (completion: unknown) => void): void;
  noteAuthCompletedElsewhere(serverId: string, accountKey: string): unknown;
  dispose(): Promise<void> | void;
}

export interface DesktopMcpManagerOptions {
  readonly officialMcp?: OfficialMcpPorts;
  readonly settingsStore: unknown;
  readonly onAccountScopeApplied: () => void;
  readonly getAccessToken: (args: { backendUrl: string }) => Promise<string>;
  readonly getMachineId: () => string | Promise<string>;
  readonly loadBoxMcpServers: (configJson: string) => Promise<unknown>;
  readonly listBoxMcpServers: (serverIdentifiers: unknown) => Promise<readonly Record<string, unknown>[]>;
  readonly listBoxMcpToolsRaw: (payloadHex: string) => Promise<string>;
  readonly executeBoxMcpToolRaw: (payloadHex: string) => Promise<string>;
  readonly onConnectorAuth: (report: ConnectorAuthReport) => void;
  readonly onMcpDiscoveryFailed: (report: { readonly errorClass: string; readonly elapsedMs: number; readonly servedStale: boolean }) => void;
  readonly onMcpDiagnostic?: (failure: { readonly leg: string; readonly errorClass: string }) => void;
}

function generatedAccountClient(credentials: Pick<AccountMcpDependencies, "getAccessToken" | "getMachineId">): AccountMcpClient {
  return createSandCursorBackendClient(DashboardService, {
    getAccessToken: async (options) => await credentials.getAccessToken({ backendUrl: options?.backendUrl }),
    getMachineId: credentials.getMachineId,
  }) as unknown as AccountMcpClient;
}

function generatedBackendClient(credentials: Pick<AccountMcpDependencies, "getAccessToken" | "getMachineId">): DashboardMcpExecClient {
  return createSandCursorBackendClient(DashboardService, {
    getAccessToken: async (options) => await credentials.getAccessToken({ backendUrl: options?.backendUrl }),
    getMachineId: credentials.getMachineId,
  }) as unknown as DashboardMcpExecClient;
}

/** Artifact anchor: electron-main/main.cjs:497780, `async function createSandDesktopMcpManager(options)`. */
export async function createSandDesktopMcpManager(options: DesktopMcpManagerOptions): Promise<DesktopMcpManagerFacade> {
  pinMcpDiagnosticsReporter(options.onMcpDiagnostic ?? null);
  const official = options.officialMcp == null ? undefined : new OfficialMcpService(options.officialMcp);
  const warnings = new Set<string>();
  async function legacy<T>(operation: () => Promise<T>, fallback: T): Promise<T> {
    try { return await operation(); }
    catch (error) {
      if (!official) throw error;
      warnings.add("部分已安装或团队连接器暂时无法加载。官方目录仍可浏览，请稍后重试。");
      options.onMcpDiagnostic?.({ leg: "legacy-mcp-partial-load", errorClass: "unavailable" });
      return fallback;
    }
  }
  const record = (value: unknown): Record<string, any> => {
    if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid MCP request.");
    return value as Record<string, any>;
  };
  const accountMcpDeps: AccountMcpDependencies = {
    getAccessToken: async (request) => await options.getAccessToken({ backendUrl: request?.backendUrl ?? getSandInferenceBackendUrl() }),
    getMachineId: async () => await options.getMachineId(),
    getBackendUrl: getSandInferenceBackendUrl,
    createClient: generatedAccountClient,
  };
  const backendMcpExec = createDashboardSandBackendMcpExec({
    getAccessToken: accountMcpDeps.getAccessToken,
    getMachineId: accountMcpDeps.getMachineId,
    createClient: generatedBackendClient,
  });
  const manager = new SandMcpManager({
    settingsStore: options.settingsStore,
    onAccountScopeApplied: options.onAccountScopeApplied,
    accountServersProvider: () => fetchAccountMcpServers(accountMcpDeps),
    accountMcpWriter: createAccountMcpWriter(accountMcpDeps),
    effectivePluginsProvider: () => fetchEffectiveUserPlugins(accountMcpDeps),
    getMachineId: accountMcpDeps.getMachineId,
    backendMcpExec,
    onConnectorAuth: options.onConnectorAuth,
  });
  const discovery = createMcpToolsDiscovery({
    definitionSource: manager.definitionSourceView(),
    lastAccountDisplayConfig: () => manager.lastAccountDisplayConfigView(),
    settingsStore: () => manager.settingsStoreView(),
    backendMcpExec,
  }, {
    onDiscoveryFailed: options.onMcpDiscoveryFailed,
    boxMcpExec: {
      loadServers: async (configJson: string) => {
        await options.loadBoxMcpServers(configJson);
      },
      listTools: async (serverIdentifiers: string[], request?: { readonly kickOnly?: boolean }) => {
        const payload = new McpStateExecArgs({
          serverIdentifiers,
          kickOnly: request?.kickOnly === true,
        }).toBinary();
        const responseHex = await options.listBoxMcpToolsRaw(Buffer.from(payload).toString("hex"));
        const response = McpStateExecResult.fromBinary(Buffer.from(responseHex, "hex"));
        if (response.result.case === "error") throw new Error(response.result.value.error);
        if (response.result.case === "rejected") throw new Error(response.result.value.reason);
        if (response.result.case !== "success") throw new Error("Box MCP state returned no result.");
        return response.result.value.servers;
      },
      executeTool: async (args: {
        readonly name: string;
        readonly args?: unknown;
        readonly toolCallId?: string;
        readonly providerIdentifier?: string;
        readonly toolName?: string;
        readonly smartModeApprovalOnly?: boolean;
        readonly skipApproval?: boolean;
      }) => {
        const request = McpArgs.fromJson({
          name: args.name,
          args: args.args ?? {},
          toolCallId: args.toolCallId ?? "",
          providerIdentifier: args.providerIdentifier ?? "",
          toolName: args.toolName ?? args.name,
          smartModeApprovalOnly: args.smartModeApprovalOnly === true,
          skipApproval: args.skipApproval === true,
          serverIdentifier: args.providerIdentifier ?? "",
        });
        const responseHex = await options.executeBoxMcpToolRaw(
          Buffer.from(request.toBinary()).toString("hex"),
        );
        return McpResult.fromBinary(Buffer.from(responseHex, "hex"));
      },
    },
  });
  manager.setBoxRuntime(discovery);
  let routedToolsSnapshot: unknown[] = [];
  let routedToolsWarm: Promise<unknown[]> | null = null;
  const warmRoutedTools = (): Promise<unknown[]> => routedToolsWarm ??= discovery.getTools().then((tools: unknown[]) => (routedToolsSnapshot = tools), (error: unknown) => {
    routedToolsWarm = null;
    throw error;
  });
  void warmRoutedTools().catch((error: unknown) => reportDesktopEdgeFailure("mcp-manager", "routed-tools-warm", error));
  let hasKickedInstallBackfill = false;
  const kickInstallBackfillOnce = (): void => {
    if (hasKickedInstallBackfill) return;
    hasKickedInstallBackfill = true;
    void backfillUserPluginInstalls(accountMcpDeps).catch((error: unknown) => reportDesktopEdgeFailure("mcp-manager", "install-backfill", error));
  };
  return {
    listServers: async () => {
      kickInstallBackfillOnce();
      warnings.clear();
      const state = await legacy(() => manager.listServers(), { servers: [] });
      const native = await official?.servers() ?? [];
      return { ...state, servers: [...state.servers, ...native], warnings: [...warnings] };
    },
    listEffectivePlugins: async () => [
      ...await legacy(() => manager.listEffectivePlugins(), []),
      ...(await official?.effective() ?? []),
    ],
    getCatalog: async (getAccessToken, args) => {
      const native = official ? officialMcpCatalogPlugins().map(marketplacePluginToView) : [];
      const other = await legacy(() => manager.getCatalog(getAccessToken, args), []);
      return [...native, ...(other as unknown[])];
    },
    resolvePluginLogo: (url) => manager.resolvePluginLogo(url),
    installEntry: async (request, getAccessToken) => {
      const input = record(request);
      if (!official?.owns(input.entryId)) return manager.installEntry(request, getAccessToken);
      await official.install(input.entryId, input.values);
      return { servers: await official.servers() };
    },
    updatePluginInstall: async (request, getAccessToken) => {
      const input = record(request);
      if (!official?.owns(input.pluginId)) return manager.updatePluginInstall(request, getAccessToken);
      await official.install(input.pluginId, input.values);
      return { servers: await official.servers() };
    },
    addServer: (request) => manager.addServer(request),
    removeServer: async (serverId) => {
      if (!official?.owns(serverId)) return manager.removeServer(serverId);
      await official.remove(serverId); return { removed: true, state: { servers: await official.servers() } };
    },
    reloadServers: async () => {
      const state = await legacy(() => manager.reloadServers(), { servers: [] });
      return { ...state, servers: [...state.servers, ...(await official?.servers() ?? [])], warnings: [...warnings] };
    },
    uninstallPlugin: async (pluginId) => {
      if (!official?.owns(pluginId)) return manager.uninstallPlugin(pluginId);
      await official.remove(pluginId); return { removed: true, state: { servers: await official.servers() } };
    },
    authenticateServer: (serverId, accountKey, requestingAgentId, forceReauth, trigger) =>
      official?.owns(serverId) ? official.authenticate(serverId, accountKey, forceReauth === true)
        : manager.authenticateServer(serverId, accountKey, requestingAgentId ?? null, forceReauth === true, trigger ?? null),
    logoutAccount: async (args) => {
      if (!official?.owns(args.serverId)) return manager.logoutAccount(args.serverId, args.accountKey);
      await official.disconnect(args.serverId); return { servers: await official.servers() };
    },
    renameAccount: (args) => {
      if (official?.owns(args.serverId)) throw new Error("官方连接器多账号 OAuth 配置尚未完成。");
      return manager.renameAccount(args.serverId, args.accountKey, args.newAccountKey);
    },
    removeAccount: async (args) => {
      if (!official?.owns(args.serverId)) return manager.removeAccount(args.serverId, args.accountKey);
      await official.disconnect(args.serverId); return { servers: await official.servers() };
    },
    setServerCustomInstructions: (request) => {
      const input = record(request);
      return official?.owns(input.serverId) ? official.setInstructions(input.serverId, input.instructions) : manager.setServerCustomInstructions(request);
    },
    listServerTools: (serverId) => official?.owns(serverId) ? official.toolSummaries(serverId) : manager.listServerTools(serverId),
    listRoutedTools: async () => [
      ...await legacy(async () => routedToolsSnapshot.length > 0 ? routedToolsSnapshot : await warmRoutedTools(), []),
      ...(await official?.routedTools() ?? []),
    ],
    executeRoutedTool: (request) => official?.owns(request.providerIdentifier)
      ? official.execute(request.providerIdentifier, request.toolName, request.args)
      : discovery.executeTool(
      undefined,
      {
        providerIdentifier: request.providerIdentifier,
        name: request.name,
        toolName: request.toolName,
        args: request.args,
        toolCallId: request.toolCallId,
      },
      request.agentId == null ? undefined : { agentId: request.agentId },
    ),
    toggleMcpToolDisabled: (request) => {
      const input = record(request);
      return official?.owns(input.serverId) ? official.toggle(input.serverId, input.toolName) : manager.toggleMcpToolDisabled(request);
    },
    setAuthCompletionObserver: (observer) => manager.setAuthCompletionObserver(observer),
    noteAuthCompletedElsewhere: (serverId, accountKey) => manager.noteAuthCompletedElsewhere(serverId, accountKey),
    dispose: () => { official?.dispose(); return manager.dispose(); },
  };
}

