import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";

export const BROWSER_EXTENSION_POLICY = Object.freeze({
  maxFiles: 512,
  maxTotalBytes: 32 * 1024 * 1024,
  maxManifestBytes: 256 * 1024,
  maxPermissions: 100,
  maxHostPermissions: 100,
});

const FORBIDDEN_PERMISSIONS = new Set([
  "debugger",
  "management",
  "nativeMessaging",
  "proxy",
  "webRequestBlocking",
]);

export interface BrowserExtensionInspection {
  readonly sourcePath: string;
  readonly integrity: string;
  readonly name: string;
  readonly version: string;
  readonly permissions: readonly string[];
  readonly hostPermissions: readonly string[];
}

function strings(value: unknown, max: number): string[] {
  if (value == null) return [];
  if (!Array.isArray(value) || value.length > max) throw new Error("BROWSER_EXTENSION_MANIFEST_INVALID");
  const output = value.map((item) => typeof item === "string" ? item.trim() : "");
  if (output.some((item) => item.length === 0 || item.length > 512)) throw new Error("BROWSER_EXTENSION_MANIFEST_INVALID");
  return [...new Set(output)].sort();
}

function inside(root: string, path: string): boolean {
  const rel = relative(root, path);
  return rel === "" || (!rel.startsWith(".."+sep) && rel !== ".." && !rel.startsWith(sep));
}

export function inspectUnpackedBrowserExtension(directory: string): BrowserExtensionInspection {
  const root = resolve(directory);
  const rootStat = lstatSync(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) throw new Error("BROWSER_EXTENSION_SOURCE_INVALID");

  const manifestPath = join(root, "manifest.json");
  const manifestStat = lstatSync(manifestPath);
  if (!manifestStat.isFile() || manifestStat.isSymbolicLink() || manifestStat.size > BROWSER_EXTENSION_POLICY.maxManifestBytes) {
    throw new Error("BROWSER_EXTENSION_MANIFEST_INVALID");
  }

  const hasher = createHash("sha256");
  let files = 0;
  let totalBytes = 0;
  const walk = (directoryPath: string): void => {
    const entries = readdirSync(directoryPath, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
      const path = join(directoryPath, entry.name);
      if (!inside(root, path)) throw new Error("BROWSER_EXTENSION_SOURCE_ESCAPE");
      const stat = lstatSync(path);
      if (stat.isSymbolicLink()) throw new Error("BROWSER_EXTENSION_SYMLINK_BLOCKED");
      if (stat.isDirectory()) {
        walk(path);
        continue;
      }
      if (!stat.isFile()) throw new Error("BROWSER_EXTENSION_SPECIAL_FILE_BLOCKED");
      files += 1;
      totalBytes += stat.size;
      if (files > BROWSER_EXTENSION_POLICY.maxFiles || totalBytes > BROWSER_EXTENSION_POLICY.maxTotalBytes) {
        throw new Error("BROWSER_EXTENSION_SOURCE_TOO_LARGE");
      }
      const bytes = readFileSync(path);
      const rel = relative(root, path).split(sep).join("/");
      hasher.update(String(Buffer.byteLength(rel, "utf8"))).update(":").update(rel).update(":");
      hasher.update(String(bytes.byteLength)).update(":").update(bytes);
    }
  };
  walk(root);

  let manifest: Record<string, unknown>;
  try {
    const parsed = JSON.parse(readFileSync(manifestPath, "utf8")) as unknown;
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) throw new Error();
    manifest = parsed as Record<string, unknown>;
  } catch {
    throw new Error("BROWSER_EXTENSION_MANIFEST_INVALID");
  }

  if (manifest.manifest_version !== 3) throw new Error("BROWSER_EXTENSION_MANIFEST_VERSION_UNSUPPORTED");
  const name = typeof manifest.name === "string" ? manifest.name.trim() : "";
  const version = typeof manifest.version === "string" ? manifest.version.trim() : "";
  if (name.length === 0 || name.length > 256 || version.length === 0 || version.length > 64) {
    throw new Error("BROWSER_EXTENSION_MANIFEST_INVALID");
  }
  if (manifest.options_page != null || manifest.options_ui != null) throw new Error("BROWSER_EXTENSION_OPTIONS_UNSUPPORTED");
  const action = typeof manifest.action === "object" && manifest.action !== null && !Array.isArray(manifest.action)
    ? manifest.action as Record<string, unknown>
    : null;
  if (action?.default_popup != null) throw new Error("BROWSER_EXTENSION_POPUP_UNSUPPORTED");

  const permissions = strings(manifest.permissions, BROWSER_EXTENSION_POLICY.maxPermissions);
  const hostPermissions = strings(manifest.host_permissions, BROWSER_EXTENSION_POLICY.maxHostPermissions);
  const forbidden = permissions.find((permission) => FORBIDDEN_PERMISSIONS.has(permission));
  if (forbidden != null) throw new Error(`BROWSER_EXTENSION_PERMISSION_UNSUPPORTED:${forbidden}`);

  return {
    sourcePath: root,
    integrity: `sha256:${hasher.digest("hex")}`,
    name,
    version,
    permissions,
    hostPermissions,
  };
}
