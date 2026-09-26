import { homedir } from "node:os";
import { isAbsolute, join, resolve } from "node:path";

import { getSandVariant } from "./sand-variant.js";
import { isPathWithin } from "./paths.js";

export const SAND_DATA_ROOT_ENV = "SAND_DATA_ROOT";
export const SAND_PRODUCTION_DATA_DIRNAME = ".grokbot";
export const SAND_USER_DATA_DIR_ENV = "SAND_USER_DATA_DIR";
export const SAND_DATA_DIRNAME = "sand-data";
export const USER_DATA_DIR_FLAG = "--user-data-dir";

export function readUserDataDirArg(argv: readonly string[]): string | null {
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === USER_DATA_DIR_FLAG) {
      const next = argv[index + 1];
      return next != null && !next.startsWith("--") ? next : null;
    }
    const prefix = `${USER_DATA_DIR_FLAG}=`;
    if (arg?.startsWith(prefix)) return arg.slice(prefix.length);
  }
  return null;
}

export function resolveSandUserDataDir(
  argv: readonly string[] = [],
  env: NodeJS.ProcessEnv = process.env,
  cwd = process.cwd(),
): string | null {
  const raw = readUserDataDirArg(argv) ?? env[SAND_USER_DATA_DIR_ENV];
  const trimmed = raw?.trim();
  if (trimmed == null || trimmed.length === 0) return null;
  return isAbsolute(trimmed) ? trimmed : resolve(cwd, trimmed);
}

export function getSandProductionRootDir(homeDir = homedir()): string {
  return join(homeDir, SAND_PRODUCTION_DATA_DIRNAME);
}

export function resolveSandDataRootOverride(
  env: NodeJS.ProcessEnv = process.env,
): string | null {
  const override = env[SAND_DATA_ROOT_ENV]?.trim();
  return override != null && override.length > 0 && isAbsolute(override)
    ? override
    : null;
}

export function getSandRootDir(homeDir = homedir()): string {
  const override = resolveSandDataRootOverride();
  if (override != null) return override;
  const userDataDir = resolveSandUserDataDir([], process.env);
  if (userDataDir != null) return join(userDataDir, SAND_DATA_DIRNAME);
  const variant = getSandVariant();
  return variant === "sand"
    ? getSandProductionRootDir(homeDir)
    : join(homeDir, ".cursor", variant);
}

export function reanchorSandPath(storedPath: string): string {
  const root = getSandRootDir();
  if (isPathWithin(root, storedPath, { isInclusive: true })) return storedPath;
  const match = /(?:[/\\]\.cursor[/\\]sand(?:-[^/\\]+)?|[/\\]\.grokbot)[/\\](.+)$/.exec(storedPath);
  if (match?.[1] == null) return storedPath;
  const segments = match[1].split(/[/\\]+/);
  if (segments.some((segment) => segment === "." || segment === "..")) return storedPath;
  return join(root, ...segments);
}
