import {
  getMarketplaceRelease,
  type MarketplaceReleaseInfo,
} from "./marketplace";
import {
  marketplaceInstallAction,
  type MarketplaceInstallAction,
} from "./marketplace-install-contract";

export const MARKETPLACE_INSTALL_STATE_KEY = "fabushi.marketplace.installs.v2";
export const LEGACY_MARKETPLACE_INSTALL_STATE_KEY = "fabushi.installed-miniapps";

export type MarketplaceInstallVerification = "github-sha256" | "github-metadata" | "legacy";

export type MarketplaceInstallRecord = {
  id: string;
  version: string;
  repository: string;
  sourceRef: string;
  releaseUrl: string;
  manifestUrl: string;
  artifactId: string;
  artifactUrl: string;
  artifactSha256: string;
  artifactSize: number;
  format: string;
  runtime: string;
  platforms: string[];
  verification: MarketplaceInstallVerification;
  installedAt: string;
};

export type MarketplaceAppInstallAction = MarketplaceInstallAction | "unavailable";

type UnknownRecord = Record<string, unknown>;

function isRecord(value: unknown): value is UnknownRecord {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function text(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

function readJson(key: string): unknown {
  if (typeof window === "undefined") return undefined;
  try {
    return JSON.parse(window.localStorage.getItem(key) ?? "null");
  } catch {
    return undefined;
  }
}

function readLegacyIds(): string[] {
  const value = readJson(LEGACY_MARKETPLACE_INSTALL_STATE_KEY);
  return Array.isArray(value)
    ? [...new Set(value.filter((item): item is string => typeof item === "string" && Boolean(item.trim())).map((item) => item.trim()))]
    : [];
}

function normalizeRecord(id: string, value: unknown): MarketplaceInstallRecord | undefined {
  if (!isRecord(value)) return undefined;
  const normalizedId = text(value.id) || id;
  if (!normalizedId) return undefined;
  const platforms = Array.isArray(value.platforms)
    ? value.platforms.filter((item): item is string => typeof item === "string" && Boolean(item.trim())).map((item) => item.trim())
    : [];
  const verification = value.verification === "github-sha256"
    || value.verification === "github-metadata"
    || value.verification === "legacy"
    ? value.verification
    : "legacy";
  return {
    id: normalizedId,
    version: text(value.version) || "0.0.0",
    repository: text(value.repository),
    sourceRef: text(value.sourceRef),
    releaseUrl: text(value.releaseUrl),
    manifestUrl: text(value.manifestUrl),
    artifactId: text(value.artifactId),
    artifactUrl: text(value.artifactUrl),
    artifactSha256: text(value.artifactSha256).toLocaleLowerCase(),
    artifactSize: Number.isSafeInteger(value.artifactSize) && Number(value.artifactSize) >= 0
      ? Number(value.artifactSize)
      : 0,
    format: text(value.format),
    runtime: text(value.runtime),
    platforms,
    verification,
    installedAt: text(value.installedAt),
  };
}

function legacyRecord(id: string): MarketplaceInstallRecord {
  return {
    id,
    version: "0.0.0",
    repository: "",
    sourceRef: "",
    releaseUrl: "",
    manifestUrl: "",
    artifactId: "",
    artifactUrl: "",
    artifactSha256: "",
    artifactSize: 0,
    format: "",
    runtime: "",
    platforms: [],
    verification: "legacy",
    installedAt: "",
  };
}

export function readMarketplaceInstallRecords(): Record<string, MarketplaceInstallRecord> {
  const records: Record<string, MarketplaceInstallRecord> = {};
  const value = readJson(MARKETPLACE_INSTALL_STATE_KEY);
  if (isRecord(value)) {
    for (const [id, candidate] of Object.entries(value)) {
      const record = normalizeRecord(id, candidate);
      if (record) records[record.id] = record;
    }
  } else if (Array.isArray(value)) {
    for (const candidate of value) {
      const record = normalizeRecord(text(isRecord(candidate) ? candidate.id : ""), candidate);
      if (record) records[record.id] = record;
    }
  }
  for (const id of readLegacyIds()) {
    if (!records[id]) records[id] = legacyRecord(id);
  }
  return records;
}

export function marketplaceInstalledIds(records: Record<string, MarketplaceInstallRecord> = readMarketplaceInstallRecords()): string[] {
  return Object.keys(records).sort();
}

function emitInstallState(records: Record<string, MarketplaceInstallRecord>): void {
  if (typeof window === "undefined") return;
  const ids = marketplaceInstalledIds(records);
  window.localStorage.setItem(MARKETPLACE_INSTALL_STATE_KEY, JSON.stringify(records));
  window.localStorage.setItem(LEGACY_MARKETPLACE_INSTALL_STATE_KEY, JSON.stringify(ids));
  window.dispatchEvent(new CustomEvent("fabushi:marketplace-installed", { detail: { ids, records } }));
}

function recordFromRelease(
  appId: string,
  release: MarketplaceReleaseInfo,
  verification: MarketplaceInstallVerification,
): MarketplaceInstallRecord {
  return {
    id: appId,
    version: release.version,
    repository: release.repository,
    sourceRef: release.sourceRef,
    releaseUrl: release.releaseUrl,
    manifestUrl: release.manifestUrl ?? "",
    artifactId: release.artifactId,
    artifactUrl: release.artifactUrl,
    artifactSha256: release.artifactSha256,
    artifactSize: release.artifactSize,
    format: release.format,
    runtime: release.runtime,
    platforms: [...release.platforms],
    verification,
    installedAt: new Date().toISOString(),
  };
}

/** Record a version selected from the immutable GitHub release metadata. */
export function installMarketplaceApp(
  appId: string,
  verification: MarketplaceInstallVerification = "github-metadata",
): MarketplaceInstallRecord | undefined {
  const release = getMarketplaceRelease(appId);
  if (!release) return undefined;
  const records = readMarketplaceInstallRecords();
  const previous = records[appId];
  const action = marketplaceInstallAction(
    installItem(appId),
    previous,
  );
  if (action === "blocked") return undefined;
  if (action === "current") return previous;
  const record = recordFromRelease(appId, release, verification);
  emitInstallState({ ...records, [appId]: record });
  return record;
}

export function uninstallMarketplaceApp(appId: string): void {
  const records = readMarketplaceInstallRecords();
  if (!records[appId]) return;
  delete records[appId];
  emitInstallState(records);
}

function installItem(appId: string) {
  const release = getMarketplaceRelease(appId);
  if (!release) return undefined;
  return {
    id: appId,
    version: release.version,
    latestVersion: release.version,
    install: {
      protocol: release.protocol,
      strategy: "github-immutable",
      pluginId: appId,
      version: release.version,
      source: {
        repository: release.repository,
        sourceRef: release.sourceRef,
        ...(release.manifestUrl ? { manifestUrl: release.manifestUrl } : {}),
        marketplaceHostsPackage: false,
      },
      artifacts: [{
        id: release.artifactId,
        runtime: release.runtime,
        platforms: [...release.platforms],
        source: { type: "https", url: release.artifactUrl },
        sha256: release.artifactSha256,
        size: release.artifactSize,
        format: release.format,
      }],
      update: {
        check: "marketplace-release",
        comparison: "version-then-artifact-sha256",
        allowDowngrade: false,
        rollback: "previous-active",
      },
    },
  };
}

export function marketplaceAppInstallAction(
  appId: string,
  installed: MarketplaceInstallRecord | undefined = readMarketplaceInstallRecords()[appId],
): MarketplaceAppInstallAction {
  const item = installItem(appId);
  if (!item) return "unavailable";
  return marketplaceInstallAction(item, installed);
}

export function marketplaceInstallActionLabel(action: MarketplaceAppInstallAction): string {
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
    case "unavailable":
      return "等待 GitHub 发布";
  }
}

export function subscribeMarketplaceInstallState(onChange: (records: Record<string, MarketplaceInstallRecord>) => void): () => void {
  if (typeof window === "undefined") return () => undefined;
  const refresh = () => onChange(readMarketplaceInstallRecords());
  window.addEventListener("storage", refresh);
  window.addEventListener("fabushi:marketplace-installed", refresh);
  return () => {
    window.removeEventListener("storage", refresh);
    window.removeEventListener("fabushi:marketplace-installed", refresh);
  };
}
