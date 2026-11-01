export type RouterProviderId =
  | "cursor"
  | "claude-code"
  | "codex"
  | "openrouter";

export interface RouterProvider {
  readonly id: RouterProviderId;
  readonly label: string;
  readonly description: string;
  readonly usageDescription: string;
  readonly usageSource: "cursor" | "external";
}

export const DEFAULT_ROUTER_PROVIDER: RouterProviderId = "cursor";
export const ROUTER_PROVIDER_PERSISTENCE_KEY = "settings.router-provider.v1";

export const ROUTER_PROVIDERS: readonly RouterProvider[] = Object.freeze([
  {
    id: "cursor",
    label: "Cursor",
    description: "Use the signed-in Cursor account and hosted agent models.",
    usageDescription: "Included and on-demand usage from the Cursor account.",
    usageSource: "cursor",
  },
  {
    id: "claude-code",
    label: "Claude Code",
    description: "Route agent requests through Anthropic Claude Code.",
    usageDescription: "Usage is managed by the connected Anthropic account.",
    usageSource: "external",
  },
  {
    id: "codex",
    label: "Codex",
    description: "Route agent requests through OpenAI Codex.",
    usageDescription: "Usage is managed by the connected OpenAI account.",
    usageSource: "external",
  },
  {
    id: "openrouter",
    label: "OpenRouter",
    description: "Route requests through models and billing in OpenRouter.",
    usageDescription: "Usage and spend are managed in OpenRouter.",
    usageSource: "external",
  },
]);

const ROUTER_PROVIDER_IDS = new Set(
  ROUTER_PROVIDERS.map((provider) => provider.id),
);

export function isRouterProviderId(value: unknown): value is RouterProviderId {
  return typeof value === "string"
    && ROUTER_PROVIDER_IDS.has(value as RouterProviderId);
}

export function routerProviderById(id: RouterProviderId): RouterProvider {
  return ROUTER_PROVIDERS.find((provider) => provider.id === id)
    ?? ROUTER_PROVIDERS[0]!;
}

export function parseRouterProviderPreference(
  raw: string | null,
): RouterProviderId {
  if (raw == null) return DEFAULT_ROUTER_PROVIDER;
  try {
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      return DEFAULT_ROUTER_PROVIDER;
    }
    const candidate = parsed as Record<string, unknown>;
    return candidate.schemaVersion === 1 && isRouterProviderId(candidate.provider)
      ? candidate.provider
      : DEFAULT_ROUTER_PROVIDER;
  } catch {
    return DEFAULT_ROUTER_PROVIDER;
  }
}

export interface RouterProviderPersistence {
  read(key: string): Promise<string | null>;
  write(key: string, value: string): Promise<void>;
}

export async function loadRouterProvider(
  persistence: RouterProviderPersistence,
): Promise<RouterProviderId> {
  const raw = await persistence.read(ROUTER_PROVIDER_PERSISTENCE_KEY);
  return parseRouterProviderPreference(raw);
}

export async function saveRouterProvider(
  persistence: RouterProviderPersistence,
  provider: RouterProviderId,
): Promise<void> {
  if (!isRouterProviderId(provider)) {
    throw new Error("Unknown router provider.");
  }
  await persistence.write(
    ROUTER_PROVIDER_PERSISTENCE_KEY,
    JSON.stringify({ schemaVersion: 1, provider }),
  );
}
