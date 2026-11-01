import type { SandUsageSummary } from "../../shared/usage.js";
import { normalizeFabushiApiBaseUrl } from "../../shared/node/fabushi-service-origin.js";

function integer(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

/** Native token budgets have no implied vendor trial, weekly cycle or dollar spend. */
export function nativeUsageSummary(value: unknown): SandUsageSummary {
  if (value == null || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid Fabushi usage response.");
  const row = value as Record<string, unknown>;
  if (!["windowStart", "windowEnd", "tokenLimit", "usedTokens", "reservedTokens", "remainingTokens"].every(key => integer(row[key]))
      || typeof row.unlimited !== "boolean"
      || (row.windowEnd as number) <= (row.windowStart as number)
      || (row.windowEnd as number) > Math.floor(Number.MAX_SAFE_INTEGER / 1000)) {
    throw new Error("Invalid Fabushi usage response.");
  }
  const limit = row.tokenLimit as number;
  const used = row.usedTokens as number;
  return {
    isEnterprise: false,
    sandUsagePercent: row.unlimited ? null : limit > 0 ? Math.min(100, used / limit * 100) : 0,
    sandUsageResetTimestampMs: (row.windowEnd as number) * 1000,
    hasAvailableUsage: row.unlimited || (row.remainingTokens as number) > 0,
    hasNonZeroIncludedLimit: row.unlimited || limit > 0,
    isSandTrial: false, hasEndedSandTrial: false, canCancelSandTrial: false,
    onDemand: null, upgradeCta: null,
  };
}

export async function fetchNativeUsageSummary(options: {
  readonly baseUrl?: string;
  readonly getAccessToken: (options: { readonly backendUrl: string }) => Promise<string>;
  readonly fetchImpl?: typeof fetch;
}): Promise<SandUsageSummary> {
  const origin = normalizeFabushiApiBaseUrl(options.baseUrl);
  const token = await options.getAccessToken({ backendUrl: origin });
  const response = await (options.fetchImpl ?? fetch)(new URL("/v1/ai/usage", origin), {
    headers: { authorization: `Bearer ${token}`, accept: "application/json" },
    redirect: "error", cache: "no-store", signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) throw new Error("Fabushi usage is unavailable.");
  return nativeUsageSummary(await response.json());
}
