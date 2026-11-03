import { join } from "node:path";

import {
  createBackendMarketplaceClient,
  normalizeEffectiveUserPluginsResponse,
  type BackendMarketplaceClientOptions,
  type EffectivePluginsResponse,
} from "../../packages/cursor-plugins/backend-marketplace-client.js";
import { DefaultPluginCacheManager } from "../../packages/cursor-plugins/cursor-marketplace.js";
import { classifyCloneError } from "../../packages/cursor-plugins/marketplace-cache.js";
import { loadFromMarketplaceSource, type MarketplaceLoaderClient } from "../../packages/cursor-plugins/loader.js";
import { buildOriginTokenGitConfig } from "../../packages/cursor-plugins/origin-git-auth.js";
import { DashboardService } from "../../packages/proto/generated/aiserver/v1/dashboard_connect.js";
import {
  createSandCursorBackendClient,
  getSandInferenceBackendUrl,
} from "../../shared/node/cursor-backend/cursor-inference.js";
import { accountCacheScope, parseJwtPayload } from "../../shared/node/cursor-token.js";

export const HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS = 15_000;
export const HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS = 10_000;

export interface HostPluginSkillsDashboardClient {
  readonly getEffectiveUserPlugins: (
    request: Record<string, never>,
    options: { readonly timeoutMs: number },
  ) => Promise<unknown>;
  readonly getMe: (
    request: Record<string, never>,
    options: { readonly timeoutMs: number },
  ) => Promise<{ readonly userId?: unknown } | null | undefined>;
}

export interface HostPluginSkillsLoaderDeps {
  readonly sandRootDir: string;
  readonly getAccessToken: (args: { readonly backendUrl: string }) => Promise<string>;
  readonly getMachineId: () => string | Promise<string>;
  readonly peekAccessToken: () => Promise<string | null>;
  readonly isSparsePluginClonesEnabled: () => boolean;
  readonly log?: (message: string) => void;
  readonly createDashboardClient?: (
    credentials: Pick<HostPluginSkillsLoaderDeps, "getAccessToken" | "getMachineId">,
  ) => HostPluginSkillsDashboardClient;
  readonly createMarketplaceClient?: (
    getEffectiveUserPlugins: () => Promise<EffectivePluginsResponse>,
    options: BackendMarketplaceClientOptions,
  ) => MarketplaceLoaderClient;
}

export interface HostPluginSkillsLoadResult {
  readonly plugins: readonly {
    readonly identifier: {
      readonly source: string;
      readonly name: string;
      readonly pluginDbId?: string;
      readonly version?: string;
    };
    readonly displayName?: string;
    readonly loadError?: string;
    readonly installPath: string;
    readonly skills: readonly {
      readonly name?: string;
      readonly description?: string;
      readonly path: string;
    }[];
  }[];
  readonly authBlocked: readonly {
    readonly pluginId: string;
    readonly pluginName: string;
    readonly marketplaceName?: string;
  }[];
  readonly listedPluginIds: readonly string[];
  readonly listedCacheKeys: readonly {
    readonly marketplaceSlug: string;
    readonly pluginId: string;
  }[];
  readonly publisherFacts: Readonly<Record<string, {
    readonly publisherUserId: number | null;
    readonly marketplaceTeamId: number | null;
  }>>;
  readonly currentUserId: number | null;
}

function positiveNumber(value: unknown): number | null {
  if (typeof value === "bigint") {
    const number = Number(value);
    return Number.isSafeInteger(number) && number > 0 ? number : null;
  }
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0 ? value : null;
}

function canonicalPluginDbId(value: unknown): string | undefined {
  if (typeof value === "string") {
    return value.length > 0 && value.trim() === value ? value : undefined;
  }
  if (typeof value === "bigint") return value > 0n ? String(value) : undefined;
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0
    ? String(value)
    : undefined;
}

function authBlocksFromFailures(
  failures: readonly {
    readonly pluginDbId?: string;
    readonly pluginName: string;
    readonly marketplaceName?: string;
    readonly errorMessage: string;
  }[],
): HostPluginSkillsLoadResult["authBlocked"] {
  const seen = new Set<string>();
  const blocks: Array<{
    pluginId: string;
    pluginName: string;
    marketplaceName?: string;
  }> = [];
  for (const failure of failures) {
    // Only clone/install transport failures can represent a user's remote Git
    // authorization refusal. Local cache I/O can contain the same words
    // (for example EACCES: permission denied) and must never become an
    // authorization decision.
    if (failure.errorType !== "clone" || classifyCloneError(failure.errorMessage) !== "user_git_access") continue;
    const id = failure.pluginDbId;
    if (typeof id !== "string" || canonicalPluginDbId(id) === undefined) {
      throw new Error("Plugin Skills authorization failure has no valid canonical plugin ID.");
    }
    if (seen.has(id)) continue;
    seen.add(id);
    blocks.push({
      pluginId: id,
      pluginName: failure.pluginName,
      ...(failure.marketplaceName == null ? {} : { marketplaceName: failure.marketplaceName }),
    });
  }
  return blocks;
}

function metadataFromEffectiveListing(response: unknown): {
  response: EffectivePluginsResponse;
  listedPluginIds: readonly string[];
  publisherFacts: HostPluginSkillsLoadResult["publisherFacts"];
} {
  if (typeof response !== "object" || response == null || Array.isArray(response)) {
    throw new Error("Plugin Skills effective listing is not an object.");
  }
  const plugins = Reflect.get(response, "plugins");
  const marketplaces = Reflect.get(response, "marketplaces");
  if (!Array.isArray(plugins) || (marketplaces !== undefined && !Array.isArray(marketplaces))) {
    throw new Error("Plugin Skills effective listing has invalid collections.");
  }
  const listedPluginIds = new Set<string>();
  const facts = new Map<string, {
    publisherUserId: number | null;
    marketplaceTeamId: number | null;
  }>();
  for (const effective of plugins) {
    if (typeof effective !== "object" || effective == null || Array.isArray(effective)
      || typeof Reflect.get(effective, "isEnabled") !== "boolean") {
      throw new Error("Plugin Skills effective listing has an invalid enabled state.");
    }
    if (!Reflect.get(effective, "isEnabled")) continue;
    const plugin = Reflect.get(effective, "plugin");
    if (typeof plugin !== "object" || plugin == null || Array.isArray(plugin)) {
      throw new Error("Plugin Skills effective listing has an enabled entry without a plugin.");
    }
    const id = canonicalPluginDbId(Reflect.get(plugin, "id"));
    if (id === undefined || listedPluginIds.has(id)) {
      throw new Error("Plugin Skills effective listing has an invalid or duplicate enabled plugin ID.");
    }
    listedPluginIds.add(id);
    const publisher = Reflect.get(plugin, "publisher");
    const marketplace = Reflect.get(plugin, "marketplace");
    facts.set(id, {
      publisherUserId: typeof publisher === "object" && publisher != null
        ? positiveNumber(Reflect.get(publisher, "ownerUserId"))
        : null,
      marketplaceTeamId: typeof marketplace === "object" && marketplace != null
        ? positiveNumber(Reflect.get(marketplace, "teamId"))
        : null,
    });
  }
  return {
    response: response as EffectivePluginsResponse,
    listedPluginIds: [...listedPluginIds],
    publisherFacts: Object.fromEntries(facts),
  };
}

/**
 * Frozen Grok 0.18 installed-plugin loader adapted as an Electron SDK edge.
 * The Mahayana Host remains the owner of plugin-skills lifecycle/cache/sync;
 * this function only uses the existing Cursor marketplace + Dashboard SDKs to
 * materialize installed plugins and returns a strictly serializable snapshot.
 */
export function createHostPluginSkillsLoader(
  deps: HostPluginSkillsLoaderDeps,
): () => Promise<HostPluginSkillsLoadResult> {
  const log = deps.log ?? ((message: string) => console.info(message));
  const pluginsRoot = join(deps.sandRootDir, "plugins");
  const createDashboardClient: NonNullable<HostPluginSkillsLoaderDeps["createDashboardClient"]> = deps.createDashboardClient ?? ((credentials) =>
    createSandCursorBackendClient(DashboardService, credentials) as unknown as HostPluginSkillsDashboardClient);
  const createMarketplaceClient = deps.createMarketplaceClient ?? createBackendMarketplaceClient;

  return async () => {
    const listedCacheKeys: Array<{ marketplaceSlug: string; pluginId: string }> = [];
    let listedPluginIds: readonly string[] = [];
    let publisherFacts: HostPluginSkillsLoadResult["publisherFacts"] = {};
    let currentUserIdForPass: number | null = null;

    const backendUrl = getSandInferenceBackendUrl();
    const [accessToken, machineId] = await Promise.all([
      deps.getAccessToken({ backendUrl }),
      deps.getMachineId(),
    ]);
    if (accessToken.trim().length === 0) {
      throw new Error("Plugin Skills require an authenticated account.");
    }
    if (getSandInferenceBackendUrl() !== backendUrl) {
      throw new Error("Plugin Skills account context changed or became unavailable during refresh.");
    }
    const accountScope = accountCacheScope(accessToken);
    const accountSubject = parseJwtPayload(accessToken)?.sub;
    // The adapter survives account-manager resets. One refresh must use one
    // credential for both identity facts and materialization; no user identity
    // or credential is retained across passes.
    const dashboard = createDashboardClient({
      getAccessToken: async () => accessToken,
      getMachineId: () => machineId,
    });
    const client = createMarketplaceClient(
      async () => {
        const response = await dashboard.getEffectiveUserPlugins(
          {},
          { timeoutMs: HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS },
        );
        // Only the raw authoritative listing can retire an installed ID. SDK
        // filtering and version/materialization failures must not erase it.
        const metadata = metadataFromEffectiveListing(response);
        listedPluginIds = metadata.listedPluginIds;
        publisherFacts = metadata.publisherFacts;
        try {
          const currentUser = await dashboard.getMe(
            {},
            { timeoutMs: HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS },
          );
          currentUserIdForPass = positiveNumber(currentUser?.userId);
        } catch {
          currentUserIdForPass = null;
          // Unknown identity may not inherit an earlier publisher's rights.
          // Do not log arbitrary transport errors that could contain secrets.
          log("[sand:plugin-skills] could not resolve the signed-in user for this refresh");
        }
        return normalizeEffectiveUserPluginsResponse(metadata.response);
      },
      {
        marketplaceCacheRoot: join(pluginsRoot, "marketplaces"),
        listOptions: { enableInlinePlugins: true },
        sparsePluginClones: deps.isSparsePluginClonesEnabled(),
        extraGitConfig: buildOriginTokenGitConfig(accessToken),
      },
    );

    const result = await loadFromMarketplaceSource({
      client,
      userId: "sand",
      cacheManager: new DefaultPluginCacheManager(undefined, {
        cacheRoot: join(pluginsRoot, "cache"),
      }),
      // Rust prunes only after it commits an authoritative snapshot.
      pruneOldVersions: false,
      onPluginsListed: async (entries) => {
        for (const entry of entries) {
          const slug = entry.marketplace?.name;
          if (slug != null && slug.length > 0) {
            listedCacheKeys.push({
              marketplaceSlug: slug,
              pluginId: entry.pluginId,
            });
          }
        }
      },
    });

    if (result.sourceUnavailable) {
      throw new Error("Plugin Skills marketplace source is unavailable.");
    }
    let currentContextMatches = false;
    try {
      const currentAccessToken = await deps.peekAccessToken();
      const currentSubject = currentAccessToken == null ? undefined : parseJwtPayload(currentAccessToken)?.sub;
      const sameKnownAccount = !!accountSubject && accountSubject.trim() === accountSubject
        && !!currentSubject && currentSubject.trim() === currentSubject
        && currentAccessToken != null && accountCacheScope(currentAccessToken) === accountScope;
      currentContextMatches = currentAccessToken != null
        && currentAccessToken.trim().length > 0
        && (currentAccessToken === accessToken || sameKnownAccount)
        && getSandInferenceBackendUrl() === backendUrl;
    } catch {
      // An unreadable current account cannot authorize a completed snapshot.
    }
    if (!currentContextMatches) {
      throw new Error("Plugin Skills account context changed or became unavailable during refresh.");
    }
    for (const failure of result.failures) {
      log(`[sand:plugin-skills] plugin ${failure.pluginName} failed to load: ${failure.errorMessage}`);
    }

    return {
      plugins: result.plugins.map((plugin) => ({
        identifier: {
          source: plugin.identifier.source,
          name: plugin.identifier.sourceInfo.name,
          ...(plugin.identifier.sourceInfo.pluginDbId == null
            ? {}
            : { pluginDbId: plugin.identifier.sourceInfo.pluginDbId }),
          ...(plugin.identifier.sourceInfo.version == null
            ? {}
            : { version: plugin.identifier.sourceInfo.version }),
        },
        ...(plugin.displayName == null ? {} : { displayName: plugin.displayName }),
        ...(plugin.loadError == null ? {} : { loadError: String(plugin.loadError) }),
        installPath: plugin.installPath,
        skills: plugin.skills.map((skill) => ({
          ...(skill.name == null ? {} : { name: skill.name }),
          ...(skill.description == null ? {} : { description: skill.description }),
          path: skill.path,
        })),
      })),
      authBlocked: authBlocksFromFailures(result.failures),
      listedPluginIds,
      listedCacheKeys,
      publisherFacts,
      currentUserId: currentUserIdForPass,
    };
  };
}
