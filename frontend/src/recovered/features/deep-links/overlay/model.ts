export type DeepLinkSource = "protocol" | "https";

export interface DeepLinkInfo {
  version: 1;
  source: DeepLinkSource;
  route: "info";
  topic: "deep-links";
}

export function deepLinkRoute(link: DeepLinkInfo): string {
  const params = new URLSearchParams({ topic: link.topic });
  return `sand://app/v${link.version}/${link.route}?${params.toString()}`;
}

export function deepLinkSourceLabel(source: DeepLinkSource): string {
  switch (source) {
    case "protocol":
      return "Custom protocol (sand://)";
    case "https":
      return "HTTPS link";
  }
}
