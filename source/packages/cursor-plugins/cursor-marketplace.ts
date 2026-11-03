import { lstat, mkdir, mkdtemp, readdir, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { extract } from "tar";
import { sanitizeFilename } from "../utils/path-matchers.js";

export const PLUGINS_CACHE_ROOT = "plugins/cache";
export const PLUGIN_CACHE_SENTINEL = ".cache-complete";

export interface PluginCacheKey {
  readonly marketplaceSlug: string;
  readonly pluginId: string;
  readonly version?: string;
}

type CacheDirectoryState = "missing" | "complete" | "incomplete" | "not-directory";
type CacheIoPhase = "initial inspection" | "prepublication inspection" | "winner inspection"
  | "staging parent creation" | "staging allocation" | "completion marker creation"
  | "publication parent creation" | "directory publication";

function isMissingPathError(error: unknown): boolean {
  return typeof error === "object" && error !== null && "code" in error && error.code === "ENOENT";
}

function cacheIoCode(error: unknown): string {
  return typeof error === "object" && error !== null && "code" in error
    && typeof error.code === "string" && /^E[A-Z0-9]+$/.test(error.code) ? error.code : "unknown";
}

function cacheIoFailure(phase: CacheIoPhase, error: unknown): Error {
  return new Error(`Plugin cache ${phase} failed (I/O code: ${cacheIoCode(error)}).`, { cause: error });
}

async function cacheIo<T>(phase: CacheIoPhase, operation: () => Promise<T>): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    throw cacheIoFailure(phase, error);
  }
}

function sanitizePluginCacheMarketplaceSlug(raw: string): string {
  return raw.replace(/[^a-zA-Z0-9_-]/g, "_");
}

function sanitizePluginCachePluginId(raw: string): string {
  return raw.replace(/[^a-zA-Z0-9_-]/g, "_");
}

function sanitizePluginCacheVersion(raw: string): string {
  return sanitizeFilename(raw);
}

function getPluginCachePath(
  cacheRoot: string,
  marketplaceSlug: string,
  pluginId: string,
  version?: string,
): string {
  const parts = [cacheRoot, marketplaceSlug, pluginId];
  if (version !== undefined) parts.push(version);
  return join(...parts);
}

export function getPluginInstallCachePath(
  cacheRoot: string,
  args: PluginCacheKey,
): string {
  return getPluginCachePath(
    cacheRoot,
    sanitizePluginCacheMarketplaceSlug(args.marketplaceSlug),
    sanitizePluginCachePluginId(args.pluginId),
    args.version !== undefined
      ? sanitizePluginCacheVersion(args.version)
      : undefined,
  );
}

export class DefaultPluginCacheManager {
  readonly cacheRoot: string;

  constructor(
    userHomeDir?: string,
    options?: { readonly cacheRoot?: string },
  ) {
    const home = userHomeDir ?? process.env.HOME ?? "";
    this.cacheRoot = options?.cacheRoot
      ?? join(home, ".cursor", PLUGINS_CACHE_ROOT);
  }

  getCacheDir(args: PluginCacheKey): string {
    return getPluginInstallCachePath(this.cacheRoot, args);
  }

  async isCached(args: PluginCacheKey): Promise<boolean> {
    try {
      return await this.cacheDirectoryState(this.getCacheDir(args)) === "complete";
    } catch {
      return false;
    }
  }

  private async cacheDirectoryState(cacheDir: string): Promise<CacheDirectoryState> {
    try {
      if (!(await lstat(cacheDir)).isDirectory()) return "not-directory";
    } catch (error) {
      if (isMissingPathError(error)) return "missing";
      throw error;
    }
    try {
      return (await lstat(join(cacheDir, PLUGIN_CACHE_SENTINEL))).isFile()
        ? "complete"
        : "incomplete";
    } catch (error) {
      if (isMissingPathError(error)) return "incomplete";
      throw error;
    }
  }

  async materializeToCache(
    args: PluginCacheKey,
    install: (stagingDir: string) => Promise<void>,
  ): Promise<string> {
    const cacheDir = this.getCacheDir(args);
    const initial = await cacheIo("initial inspection", () => this.cacheDirectoryState(cacheDir));
    if (initial === "complete") return cacheDir;
    if (initial !== "missing") throw new Error(`Plugin cache target is ${initial} (phase: initial inspection).`, { cause: { cacheDir } });

    // Rust prunes inside cacheRoot. Keep an in-progress attempt outside that
    // tree, on its parent filesystem, and require atomic directory publication.
    const stagingParent = dirname(resolve(this.cacheRoot));
    await cacheIo("staging parent creation", () => mkdir(stagingParent, { recursive: true }));
    const stagingDir = await cacheIo("staging allocation", () => mkdtemp(join(stagingParent, ".plugin-cache-install-")));
    let ownsStaging = true;
    let primaryFailure: { readonly error: unknown } | undefined;
    try {
      // The backend callback's original error remains the existing auth signal.
      await install(stagingDir);
      // Plugin-supplied markers (including symlinks) are not completion proof.
      await cacheIo("completion marker creation", () => writeFile(join(stagingDir, PLUGIN_CACHE_SENTINEL), "", { flag: "wx" }));

      const current = await cacheIo("prepublication inspection", () => this.cacheDirectoryState(cacheDir));
      if (current === "complete") return cacheDir;
      if (current !== "missing") throw new Error(`Plugin cache target is ${current} (phase: prepublication inspection).`, { cause: { cacheDir } });
      await cacheIo("publication parent creation", () => mkdir(dirname(cacheDir), { recursive: true }));
      try {
        // Every cooperating SDK publisher renames a complete nonempty directory;
        // a concurrent winner is never merged into or recursively removed here.
        await rename(stagingDir, cacheDir);
      } catch (error) {
        let winner: CacheDirectoryState;
        try {
          winner = await this.cacheDirectoryState(cacheDir);
        } catch (inspectionError) {
          const publicationFailure = cacheIoFailure("directory publication", error);
          const inspectionFailure = cacheIoFailure("winner inspection", inspectionError);
          throw new AggregateError(
            [publicationFailure, inspectionFailure],
            `${publicationFailure.message}\n${inspectionFailure.message}`,
            { cause: publicationFailure },
          );
        }
        if (winner === "complete") return cacheDir;
        throw cacheIoFailure("directory publication", error);
      }
      ownsStaging = false;
      return cacheDir;
    } catch (error) {
      primaryFailure = { error };
      throw error;
    } finally {
      // After rename the old temporary name is no longer ours to clean.
      if (ownsStaging) {
        try {
          await rm(stagingDir, { recursive: true, force: true });
        } catch (cleanupError) {
          // Do not let a cleanup message or path create a new Git-auth denial.
          // The complete cleanup error remains available in errors/cause.
          const cleanupMessage = `Plugin cache staging cleanup failed; residual state unknown (I/O code: ${cacheIoCode(cleanupError)}).`;
          if (primaryFailure !== undefined) {
            const primary = primaryFailure.error;
            // Host and Rust classify canonical auth denials from the message.
            // Keep that failure visible as well as the unsuccessful cleanup.
            throw new AggregateError(
              [primary, cleanupError],
              `${primary instanceof Error ? primary.message : String(primary)}\n${cleanupMessage}`,
              { cause: primary },
            );
          }
          throw new Error(cleanupMessage, { cause: cleanupError });
        }
      }
    }
  }

  async extractToCache(
    args: PluginCacheKey & { readonly tarball: Uint8Array },
  ): Promise<string> {
    const cacheDir = this.getCacheDir(args);
    await mkdir(cacheDir, { recursive: true });
    const bufferStream = Readable.from(args.tarball);
    await pipeline(
      bufferStream,
      createGunzip(),
      extract({ cwd: cacheDir, strip: 1 }),
    );
    await this.markCacheComplete(args);
    return cacheDir;
  }

  async listCachedVersions(
    args: Omit<PluginCacheKey, "version">,
  ): Promise<string[]> {
    const safeSlug = sanitizePluginCacheMarketplaceSlug(args.marketplaceSlug);
    const safePluginId = sanitizePluginCachePluginId(args.pluginId);
    const pluginDir = join(this.cacheRoot, safeSlug, safePluginId);
    try {
      const entries = await readdir(pluginDir, { withFileTypes: true });
      return entries.filter(entry => entry.isDirectory()).map(entry => entry.name);
    } catch {
      return [];
    }
  }

  async removeVersion(args: PluginCacheKey): Promise<void> {
    try {
      await rm(this.getCacheDir(args), { recursive: true, force: true });
    } catch {
      // Cache cleanup is best effort in the immutable implementation.
    }
  }

  async removeAllVersions(
    args: Omit<PluginCacheKey, "version">,
  ): Promise<void> {
    await rm(getPluginInstallCachePath(this.cacheRoot, args), {
      recursive: true,
      force: true,
    });
  }

  async pruneOldVersions(
    args: Omit<PluginCacheKey, "version"> & {
      readonly keepVersions: readonly string[];
    },
  ): Promise<void> {
    const cachedVersions = await this.listCachedVersions(args);
    const keepSet = new Set(
      args.keepVersions.map(version => sanitizePluginCacheVersion(version)),
    );
    for (const version of cachedVersions) {
      if (!keepSet.has(version)) {
        await this.removeVersion({ ...args, version });
      }
    }
  }

  async markCacheComplete(args: PluginCacheKey): Promise<void> {
    await writeFile(join(this.getCacheDir(args), PLUGIN_CACHE_SENTINEL), "");
  }
}
