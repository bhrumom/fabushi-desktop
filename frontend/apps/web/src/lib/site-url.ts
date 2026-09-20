import { brand } from "@fabushi/shared";

const rawBasePath = process.env.NEXT_PUBLIC_SITE_BASE_PATH?.trim() ?? "";
export const siteBasePath =
  rawBasePath && rawBasePath !== "/" ? `/${rawBasePath.replace(/^\/+|\/+$/g, "")}` : "";

const rootHref = siteBasePath ? `${siteBasePath}/` : "/";
const defaultOrigin = `https://${brand.domain}`;
const configuredOrigin = (process.env.NEXT_PUBLIC_SITE_ORIGIN?.trim() || defaultOrigin).replace(/\/+$/, "");
const legacyOfficialOrigins = new Set([
  `https://www.${brand.domain}`,
  `https://fabushi.${brand.domain}`,
]);
// The public website is canonical on the root domain. Keep stale deployment
// variables from making a legacy hostname canonical again after the host split.
const siteOrigin = legacyOfficialOrigins.has(configuredOrigin) ? defaultOrigin : configuredOrigin;

function isExternalHref(path: string) {
  return /^(?:[a-z]+:)?\/\//i.test(path) || path.startsWith("mailto:") || path.startsWith("tel:");
}

function hasFileExtension(pathname: string) {
  const lastSegment = pathname.split("/").filter(Boolean).pop() ?? "";
  return lastSegment.includes(".");
}

export function siteHref(path: string) {
  if (!path || path === "/") {
    return rootHref;
  }

  if (isExternalHref(path)) {
    return path;
  }

  if (path.startsWith("#")) {
    return `${rootHref}${path}`;
  }

  const [pathnameWithQuery, hash = ""] = path.split("#", 2);
  const [pathname, query = ""] = pathnameWithQuery.split("?", 2);
  const normalizedPathname = pathname === "/" ? rootHref : `/${pathname.replace(/^\/+|\/+$/g, "")}`;
  const needsTrailingSlash = !hasFileExtension(normalizedPathname);
  const hrefPath =
    normalizedPathname === rootHref
      ? rootHref
      : `${siteBasePath}${normalizedPathname}${needsTrailingSlash ? "/" : ""}`;
  const querySuffix = query ? `?${query}` : "";
  const hashSuffix = hash ? `#${hash}` : "";
  return `${hrefPath}${querySuffix}${hashSuffix}`;
}

export function siteUrl(path: string) {
  return new URL(siteHref(path), `${siteOrigin}/`).toString();
}
