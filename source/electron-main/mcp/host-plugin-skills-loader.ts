import { join } from "node:path";

import {
  createBackendMarketplaceClient,
  normalizeEffectiveUserPluginsResponse,
} from "../../packages/cursor-plugins/backend-marketplace-client.js";
import { DefaultPluginCacheManager } from "../../packages/cursor-plugins/cursor-marketplace.js";
import { classifyCloneError } from "../../packages/cursor-plugins/marketplace-cache.js";
import { loadFromMarketplaceSource } from "../../packages/cursor-plugins/loader.js";
import { buildOriginTokenGitConfig } from "../../packages/cursor-plugins/origin-git-auth.js";
import { DashboardService } from "../../packages/proto/generated/aiserver/v1/dashboard_connect.js";
import {
  GetEffectiveUserPluginsRequest,
  GetMeRequest,
} from "../../packages/proto/generated/aiserver/v1/dashboard_pb.js";
import {
  createSandCursorBackendClient,
  getSandInferenceBackendUrl,
} from "../../shared/node/cursor-backend/cursor-inference.js";

export const HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS = 15_000;
export const HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS = 10_000;

export interface HostPluginSkillsLoaderDeps {
  readonly sandRootDir: string;
  readonly getAccessToken: (args: { readonly backendUrl: string }) => Promise<string>;
  readonly getMachineId: () => string | Promise<string>;
  readonly peekAccessToken: () => Promise<string | null>;
  readonly isSparsePluginClonesEnabled: () => boolean;
  readonly log?: (message: string) => void;
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

function pluginDbId(plugin: {
  readonly identifier: {
    readonly source: string;
    readonly sourceInfo: { readonly pluginDbId?: string };
  };
}): string | undefined {
  return plugin.identifier.source === "cursor-first-party"
    || plugin.identifier.source === "cursor-third-party"
    ? plugin.identifier.sourceInfo.pluginDbId
    : undefined;
}

function authBlocksFromFailures(
  failures: readonly {
    readonly pluginDbId?: string;
    readonly pluginId?: string;
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
    if (classifyCloneError(failure.errorMessage) !== "user_git_access") continue;
    const id = failure.pluginDbId ?? failure.pluginId ?? "";
    const key = id || `name:${failure.pluginName}`;
    if (seen.has(key)) continue;
    seen.add(key);
    blocks.push({
      pluginId: id,
      pluginName: failure.pluginName,
      ...(failure.marketplaceName == null ? {} : { marketplaceName: failure.marketplaceName }),
    });
  }
  return blocks;
}

function publisherFactsFromListing(response: unknown): Map<string, {
  publisherUserId: number | null;
  marketplaceTeamId: number | null;
}> {
  const facts = new Map<string, {
    publisherUserId: number | null;
    marketplaceTeamId: number | null;
  }>();
  if (typeof response !== "object" || response == null) return facts;
  const plugins = Reflect.get(response, "plugins");
  if (!Array.isArray(plugins)) return facts;
  for (const effective of plugins) {
    if (typeof effective !== "object" || effective == null) continue;
    const plugin = Reflect.get(effective, "plugin");
    if (typeof plugin !== "object" || plugin == null) continue;
    const rawId = Reflect.get(plugin, "id");
    if (rawId == null) continue;
    const publisher = Reflect.get(plugin, "publisher");
    const marketplace = Reflect.get(plugin, "marketplace");
    facts.set(String(rawId), {
      publisherUserId: typeof publisher === "object" && publisher != null
        ? positiveNumber(Reflect.get(publisher, "ownerUserId"))
        : null,
      marketplaceTeamId: typeof marketplace === "object" && marketplace != null
        ? positiveNumber(Reflect.get(marketplace, "teamId"))
        : null,
    });
  }
  return facts;
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
  const backendUrl = getSandInferenceBackendUrl();
  const dashboard: any = createSandCursorBackendClient(DashboardService, {
    getAccessToken: async () => await deps.getAccessToken({ backendUrl }),
    getMachineId: deps.getMachineId,
  });
  let currentUserId: number | null = null;

  const resolveCurrentUserId = async (): Promise<number | null> => {
    if (currentUserId != null) return currentUserId;
    try {
      const response = await dashboard.getMe(
        new GetMeRequest(),
        { timeoutMs: HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS },
      );
      currentUserId = positiveNumber(response?.userId);
    } catch (error) {
      log(`[sand:plugin-skills] could not resolve the signed-in user: ${error instanceof Error ? error.message : String(error)}`);
    }
    return currentUserId;
  };

  return async () => {
    const listedCacheKeys: Array<{ marketplaceSlug: string; pluginId: string }> = [];
    const listedPluginIds = new Set<string>();
    let publisherFacts = new Map<string, {
      publisherUserId: number | null;
      marketplaceTeamId: number | null;
    }>();
    let currentUserIdForPass: number | null = null;

    const accessToken = await deps.peekAccessToken();
    const client = createBackendMarketplaceClient(
      async () => {
        const response = await dashboard.getEffectiveUserPlugins(
          new GetEffectiveUserPluginsRequest(),
          { timeoutMs: HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS },
        );
        publisherFacts = publisherFactsFromListing(response);
        currentUserIdForPass = await resolveCurrentUserId();
        return normalizeEffectiveUserPluginsResponse(response);
      },
      {
        marketplaceCacheRoot: join(pluginsRoot, "marketplaces"),
        listOptions: { enableInlinePlugins: true },
        sparsePluginClones: deps.isSparsePluginClonesEnabled(),
        extraGitConfig: buildOriginTokenGitConfig(accessToken ?? undefined),
      },
    );

    const result = await loadFromMarketplaceSource({
      client,
      userId: "sand",
      cacheManager: new DefaultPluginCacheManager(undefined, {
        cacheRoot: join(pluginsRoot, "cache"),
      }),
      pruneOldVersions: true,
      onPluginsListed: async (entries) => {
        for (const entry of entries) {
          if (entry.pluginDbId != null && entry.pluginDbId.length > 0) {
            listedPluginIds.add(entry.pluginDbId);
          }
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

    for (const failure of result.failures) {
      log(`[sand:plugin-skills] plugin ${failure.pluginName} failed to load: ${failure.errorMessage}`);
    }
    for (const plugin of result.plugins) {
      const id = pluginDbId(plugin);
      if (id != null && id.length > 0) listedPluginIds.add(id);
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
      listedPluginIds: [...listedPluginIds],
      listedCacheKeys,
      publisherFacts: Object.fromEntries(publisherFacts.entries()),
      currentUserId: currentUserIdForPass,
    };
  };
}
