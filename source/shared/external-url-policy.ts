export const ALLOWED_EXTERNAL_PROTOCOLS = new Set([
  "http:",
  "https:",
  "mailto:",
  "obsidian:",
  "tel:",
]);

function externalUrl(value: unknown): URL | null {
  if (typeof value !== "string" || value.length === 0) return null;
  try {
    return new URL(value);
  } catch {
    return null;
  }
}

function decodedParameterName(value: string): string {
  let result = value;
  for (let index = 0; index < 4; index += 1) {
    let next: string;
    try {
      next = decodeURIComponent(result);
    } catch {
      break;
    }
    if (next === result) break;
    result = next;
  }
  while (result.startsWith("?")) result = result.slice(1);
  return result;
}

function isForeignWebAuthTokenPart(part: string): boolean {
  const separator = part.indexOf("=");
  const encodedName = separator === -1 ? part : part.slice(0, separator);
  const name = decodedParameterName(encodedName).toLowerCase();
  return name.startsWith("tgwebauth") || name === "autologin_token";
}

function withoutForeignWebAuthTokenParams(encoded: string): string | null {
  let removed = false;
  const kept: string[] = [];
  for (const part of encoded.split("&")) {
    if (isForeignWebAuthTokenPart(part)) {
      removed = true;
    } else {
      kept.push(part);
    }
  }
  return removed ? kept.join("&") : null;
}

function withoutForeignWebAuthTokenFragment(encoded: string): string | null {
  const question = encoded.indexOf("?");
  if (question < 0) return withoutForeignWebAuthTokenParams(encoded);
  const routePart = encoded.slice(0, question);
  const paramsPart = encoded.slice(question + 1);
  const route = withoutForeignWebAuthTokenParams(routePart);
  const params = withoutForeignWebAuthTokenParams(paramsPart);
  if (route == null && params == null) return null;
  const nextRoute = route ?? routePart;
  const nextParams = params ?? paramsPart;
  return nextParams.length === 0 ? nextRoute : `${nextRoute}?${nextParams}`;
}

export function stripForeignWebAuthTokens(value: URL): URL {
  if (value.protocol !== "http:" && value.protocol !== "https:") return value;
  const sanitized = new URL(value.href);
  const query = withoutForeignWebAuthTokenParams(sanitized.search.startsWith("?") ? sanitized.search.slice(1) : sanitized.search);
  const fragment = withoutForeignWebAuthTokenFragment(sanitized.hash.startsWith("#") ? sanitized.hash.slice(1) : sanitized.hash);
  if (query != null) sanitized.search = query.length === 0 ? "" : `?${query}`;
  if (fragment != null) sanitized.hash = fragment.length === 0 ? "" : `#${fragment}`;
  return sanitized;
}

export function parseAllowedExternalUrl(value: unknown): string | null {
  const url = externalUrl(value);
  if (url == null || !ALLOWED_EXTERNAL_PROTOCOLS.has(url.protocol)) return null;
  return stripForeignWebAuthTokens(url).href;
}

export function parseServerAcceptedAuthExternalUrl(value: unknown, expectedOrigin: string): string | null {
  const url = externalUrl(value);
  const expected = externalUrl(expectedOrigin);
  if (
    url == null
    || expected == null
    || (url.protocol !== "http:" && url.protocol !== "https:")
    || (expected.protocol !== "http:" && expected.protocol !== "https:")
    || url.username !== ""
    || url.password !== ""
    || expected.username !== ""
    || expected.password !== ""
    || url.origin !== expected.origin
  ) return null;
  return url.href;
}

export function isHttpExternalUrl(value: unknown): boolean {
  const url = externalUrl(value);
  return url?.protocol === "http:" || url?.protocol === "https:";
}
