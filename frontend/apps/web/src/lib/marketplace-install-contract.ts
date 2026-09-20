export const MARKETPLACE_INSTALL_PROTOCOL = "fabushi.marketplace.install.v1" as const;

export type MarketplaceInstallArtifact = {
  id?: string;
  runtime?: string;
  platforms?: string[];
  source?: Record<string, unknown>;
  sha256?: string;
  artifactSha256?: string;
  size?: number;
  sizeBytes?: number;
  format?: string;
  archiveFormat?: string;
  entry?: string;
};

export type MarketplaceInstallContract = {
  protocol?: string;
  strategy?: string;
  pluginId?: string;
  version?: string;
  source?: {
    repository?: string;
    sourceRef?: string;
    manifestUrl?: string;
    marketplaceHostsPackage?: boolean;
    [key: string]: unknown;
  };
  artifacts?: MarketplaceInstallArtifact[];
  update?: {
    check?: string;
    comparison?: string;
    allowDowngrade?: boolean;
    rollback?: string;
    [key: string]: unknown;
  };
  permissions?: string[];
  [key: string]: unknown;
};

export type MarketplaceInstallItem = {
  pluginId?: string;
  id?: string;
  latestVersion?: string;
  version?: string;
  installMode?: string;
  install?: MarketplaceInstallContract | Record<string, unknown>;
  releaseManifest?: Record<string, unknown>;
};

export type MarketplaceInstallAction =
  | "install"
  | "update"
  | "reinstall"
  | "current"
  | "blocked";

const VERSION_PART_PATTERN = /[.+-]/;

function asInstallContract(value: unknown): MarketplaceInstallContract | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as MarketplaceInstallContract
    : undefined;
}
function text(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

export function marketplaceInstallContract(item: MarketplaceInstallItem | undefined): MarketplaceInstallContract | undefined {
  if (!item) return undefined;
  return asInstallContract(item.install) ?? asInstallContract(item.releaseManifest?.install);
}

export function marketplaceInstallArtifact(
  item: MarketplaceInstallItem | undefined,
): MarketplaceInstallArtifact | undefined {
  const contract = marketplaceInstallContract(item);
  return Array.isArray(contract?.artifacts)
    ? contract.artifacts.find((artifact) => Boolean(artifact && typeof artifact === "object"))
    : undefined;
}

export function marketplaceArtifactDigest(item: MarketplaceInstallItem | undefined): string {
  const artifact = marketplaceInstallArtifact(item);
  return text(artifact?.sha256 ?? artifact?.artifactSha256).toLocaleLowerCase();
}

export function compareMarketplaceVersions(left: unknown, right: unknown): number {
  const a = text(left).replace(/^v/i, "");
  const b = text(right).replace(/^v/i, "");
  if (!a && !b) return 0;
  if (!a) return -1;
  if (!b) return 1;
  const tokenize = (value: string) => value.split(VERSION_PART_PATTERN).flatMap((part) => {
    const numbers = part.match(/\d+/g);
    return numbers ? numbers.map((number) => Number(number)) : [0];
  });
  const aParts = tokenize(a);
  const bParts = tokenize(b);
  const length = Math.max(aParts.length, bParts.length);
  for (let index = 0; index < length; index += 1) {
    const aPart = aParts[index] ?? 0;
    const bPart = bParts[index] ?? 0;
    if (aPart !== bPart) return aPart > bPart ? 1 : -1;
  }
  const aPrerelease = a.includes("-");
  const bPrerelease = b.includes("-");
  if (aPrerelease !== bPrerelease) return aPrerelease ? -1 : 1;
  return 0;
}

export function marketplaceItemVersion(item: MarketplaceInstallItem | undefined): string {
  const contract = marketplaceInstallContract(item);
  return text(item?.latestVersion ?? item?.version ?? contract?.version);
}

export function marketplaceInstallAction(
  item: MarketplaceInstallItem | undefined,
  installed: { version?: string; artifactSha256?: string } | undefined,
): MarketplaceInstallAction {
  if (!installed) return "install";
  const available = marketplaceItemVersion(item);
  const current = text(installed.version);
  if (available && current && compareMarketplaceVersions(current, available) > 0) return "blocked";
  if (!current || (available && compareMarketplaceVersions(current, available) < 0)) return "update";
  const expectedDigest = marketplaceArtifactDigest(item);
  const installedDigest = text(installed.artifactSha256).toLocaleLowerCase();
  if (expectedDigest && installedDigest && expectedDigest !== installedDigest) return "reinstall";
  return "current";
}

export function marketplaceInstallActionLabel(action: MarketplaceInstallAction): string {
  switch (action) {
    case "install":
      return "安装";
    case "update":
      return "更新";
    case "reinstall":
      return "重新安装";
    case "blocked":
      return "阻止降级";
    case "current":
      return "已是最新";
  }
}
