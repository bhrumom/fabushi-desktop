// Shared origin authority for native account and product service configuration.
export const DEFAULT_FABUSHI_API_BASE_URL = "https://api.ombhrum.com";
export const FABUSHI_CI_ISOLATED_AUTH_BASE_URL = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";

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
  if (!loopback && (url.pathname !== "/" || url.port.length > 0
    || ![DEFAULT_FABUSHI_API_BASE_URL, FABUSHI_CI_ISOLATED_AUTH_BASE_URL].includes(url.origin))) {
    throw new Error("Fabushi account API must use an approved Native issuer origin.");
  }
  return loopback ? url.toString().replace(/\/+$/, "") : url.origin;
}

