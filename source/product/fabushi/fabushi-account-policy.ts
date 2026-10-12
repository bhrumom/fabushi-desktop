export const DEFAULT_FABUSHI_API_BASE_URL = "https://api.ombhrum.com";
export const DEFAULT_FABUSHI_RESPONSES_URL = `${DEFAULT_FABUSHI_API_BASE_URL}/codex-deepseek/v1/responses`;

export interface FabushiSession {
  readonly accessToken: string;
  readonly refreshToken?: string;
  readonly accessTokenExpiresAt: number;
  readonly refreshTokenExpiresAt?: number;
  readonly sessionId: string;
  readonly deviceId: string;
  readonly username: string;
  readonly userId: string | number;
  readonly user?: Readonly<Record<string, unknown>>;
  readonly provider?: string;
  readonly ciRunner?: boolean;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function boundedText(value: unknown, max: number): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 && trimmed.length <= max ? trimmed : undefined;
}

export function normalizeExpiryMs(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) return undefined;
  return value < 10_000_000_000 ? Math.trunc(value * 1_000) : Math.trunc(value);
}

export function normalizeFabushiApiBaseUrl(raw: string | undefined): string {
  const candidate = (raw ?? DEFAULT_FABUSHI_API_BASE_URL).trim().replace(/\/+$/, "");
  const url = new URL(candidate);
  const loopback = ["localhost", "127.0.0.1", "::1", "[::1]"].includes(url.hostname);
  if (url.protocol !== "https:" && !(loopback && url.protocol === "http:")) {
    throw new Error("Fabushi account API must use HTTPS outside loopback development.");
  }
  if (url.username || url.password || url.search || url.hash) {
    throw new Error("Fabushi account API base URL must not contain credentials, query, or fragment.");
  }
  return url.toString().replace(/\/+$/, "");
}

export function normalizeFabushiSession(
  value: unknown,
  options: { readonly allowRefreshless?: boolean } = {},
): FabushiSession | null {
  if (!isRecord(value)) return null;
  const accessToken = boundedText(value.accessToken, 16_384);
  const refreshToken = boundedText(value.refreshToken, 16_384);
  const accessTokenExpiresAt = normalizeExpiryMs(value.accessTokenExpiresAt ?? value.expiresAtMs);
  const refreshTokenExpiresAt = normalizeExpiryMs(value.refreshTokenExpiresAt);
  const sessionId = boundedText(value.sessionId, 200);
  const deviceId = boundedText(value.deviceId, 200);
  const username = boundedText(value.username, 320);
  const userId = typeof value.userId === "string" && value.userId.length > 0 && value.userId.length <= 200
    ? value.userId
    : typeof value.userId === "number" && Number.isFinite(value.userId)
      ? value.userId
      : undefined;
  if (
    accessToken == null
    || accessTokenExpiresAt == null
    || sessionId == null
    || deviceId == null
    || username == null
    || userId == null
  ) return null;
  if (refreshToken == null && options.allowRefreshless !== true) return null;
  if (refreshToken != null && refreshTokenExpiresAt == null) return null;
  const user = isRecord(value.user) ? value.user : undefined;
  const provider = boundedText(value.provider, 100);
  const ciRunner = typeof value.ciRunner === "boolean" ? value.ciRunner : undefined;
  return {
    accessToken,
    ...(refreshToken == null ? {} : { refreshToken }),
    accessTokenExpiresAt,
    ...(refreshTokenExpiresAt == null ? {} : { refreshTokenExpiresAt }),
    sessionId,
    deviceId,
    username,
    userId,
    ...(user == null ? {} : { user }),
    ...(provider == null ? {} : { provider }),
    ...(ciRunner == null ? {} : { ciRunner }),
  };
}

export function normalizeFabushiCiSession(value: unknown): FabushiSession | null {
  const session = normalizeFabushiSession(value, { allowRefreshless: true });
  if (
    session == null
    || session.provider !== "github-actions"
    || session.ciRunner !== true
    || session.refreshToken != null
    || !session.sessionId.startsWith("ci-runner:")
  ) return null;
  return session;
}
