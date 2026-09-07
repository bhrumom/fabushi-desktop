import {
  ElectronMahayanaHostTransport,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import type { MarketplaceBrowseResult } from '../../frontend/apps/web/src/lib/mahayana-host/transport';

const INSTALL_MARKER = Symbol.for('fabushi.desktop.miniapp-discovery-aliases.v1');
const MINI_APP_DISCOVERY_ALIASES = new Set([
  '小程序',
  'mini app',
  'mini apps',
  'miniapp',
  'miniapps',
]);
const MINI_APP_DISCOVERY_LABEL = '小程序 · Mini App';

type MarketplaceBrowse = (query?: string) => Promise<MarketplaceBrowseResult>;
type PatchableTransportPrototype = {
  marketplaceBrowse: MarketplaceBrowse;
  [INSTALL_MARKER]?: boolean;
};

function normalizedDiscoveryTerm(query?: string): string | undefined {
  const trimmed = query?.normalize('NFKC').trim();
  return trimmed ? trimmed.toLocaleLowerCase().replace(/\s+/g, ' ') : undefined;
}

export function isMiniAppDiscoveryAlias(query?: string): boolean {
  const normalized = normalizedDiscoveryTerm(query);
  return Boolean(normalized && MINI_APP_DISCOVERY_ALIASES.has(normalized));
}

export function normalizeMiniAppDiscoveryQuery(query?: string): string | undefined {
  const trimmed = query?.normalize('NFKC').trim();
  if (!trimmed) return undefined;
  return isMiniAppDiscoveryAlias(trimmed) ? undefined : trimmed;
}

function projectMiniAppCategoryForSearch(result: MarketplaceBrowseResult): MarketplaceBrowseResult {
  return {
    ...result,
    plugins: result.plugins.map((app) => {
      const existing = `${app.displayName} ${app.description} ${app.pluginId}`.toLocaleLowerCase();
      if (existing.includes('小程序') && existing.includes('mini app')) return app;
      return {
        ...app,
        description: `${app.description} · ${MINI_APP_DISCOVERY_LABEL}`,
      };
    }),
  };
}

/**
 * The Marketplace endpoint performs content search. Generic category phrases
 * such as "小程序" / "Mini App" instead mean "show Mini Apps" in the desktop
 * Apps search tab. Normalize only those category aliases at the existing
 * Mahayana transport boundary; title/id/description searches remain untouched
 * and the Host remains authoritative for which apps are discoverable.
 *
 * GlobalSearchWorkspace performs a second client-side text filter over the
 * returned summaries, and install/uninstall refreshes can re-browse with an
 * empty Marketplace query while the global search box still contains 小程序.
 * Preserve the true Mini App category on every Marketplace summary so both the
 * initial category discovery and the post-install refresh survive that second
 * filter. Production descriptions that already carry the category stay intact.
 */
export function installDesktopMiniAppDiscoveryAliases(): void {
  const prototype = ElectronMahayanaHostTransport.prototype as PatchableTransportPrototype;
  if (prototype[INSTALL_MARKER]) return;
  const browse = prototype.marketplaceBrowse;
  Object.defineProperty(prototype, INSTALL_MARKER, {
    configurable: false,
    enumerable: false,
    value: true,
    writable: false,
  });
  prototype.marketplaceBrowse = async function marketplaceBrowseWithDiscoveryAliases(query?: string) {
    const result = await browse.call(this, normalizeMiniAppDiscoveryQuery(query));
    return projectMiniAppCategoryForSearch(result);
  };
}
