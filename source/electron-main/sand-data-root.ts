import { homedir } from "node:os";
import { isAbsolute, join, relative, resolve } from "node:path";

import { getSandVariant } from "../shared/node/sand-variant.js";
import { isPathWithin } from "../shared/node/paths.js";

export const SAND_DATA_ROOT_ENV = "SAND_DATA_ROOT";
export const SAND_USER_DATA_DIR_ENV = "SAND_USER_DATA_DIR";
export const SAND_DATA_DIRNAME = "sand-data";
export const USER_DATA_DIR_FLAG = "--user-data-dir";
export const SAND_PRODUCTION_DATA_DIRNAME = ".grokbot";

export function readDesktopUserDataDirArg(argv: readonly string[]): string | null {
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

export function resolveDesktopSandUserDataDir(
  argv: readonly string[] = process.argv,
  env: NodeJS.ProcessEnv = process.env,
  cwd = process.cwd(),
): string | null {
  const raw = readDesktopUserDataDirArg(argv) ?? env[SAND_USER_DATA_DIR_ENV];
  const trimmed = raw?.trim();
  if (trimmed == null || trimmed.length === 0) return null;
  return isAbsolute(trimmed) ? trimmed : resolve(cwd, trimmed);
}

export function getDesktopSandRootDir(homeDir = homedir()): string {
  const override = process.env[SAND_DATA_ROOT_ENV]?.trim();
  if (override != null && override.length > 0 && isAbsolute(override)) return override;
  const userDataDir = resolveDesktopSandUserDataDir();
  if (userDataDir != null) return join(userDataDir, SAND_DATA_DIRNAME);
  const variant = getSandVariant();
  return variant === "sand"
    ? join(homeDir, SAND_PRODUCTION_DATA_DIRNAME)
    : join(homeDir, ".cursor", variant);
}

export function reanchorDesktopSandPath(storedPath: string): string {
  const root = getDesktopSandRootDir();
  if (isPathWithin(root, storedPath, { isInclusive: true })) return storedPath;
  const match = /(?:[/\\]\.cursor[/\\]sand(?:-[^/\\]+)?|[/\\]\.grokbot)[/\\](.+)$/.exec(storedPath);
  if (match?.[1] == null) return storedPath;
  const segments = match[1].split(/[/\\]+/);
  if (segments.some((segment) => segment === "." || segment === "..")) return storedPath;
  return join(root, ...segments);
}

export function desktopSandRelativePath(path: string): string | null {
  const root = getDesktopSandRootDir();
  if (!isPathWithin(root, path, { isInclusive: true })) return null;
  return relative(root, path);
}
