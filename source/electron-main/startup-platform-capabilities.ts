export type StartupApplicationFolderMethod =
  | "isInApplicationsFolder"
  | "moveToApplicationsFolder";

/**
 * Electron exposes Applications-folder startup APIs only on macOS.
 * Keep this platform decision dependency-free so contract tests can exercise it
 * without loading the full production Electron composition graph.
 */
export function startupApplicationFolderMethodsForPlatform(
  platform: string,
): readonly StartupApplicationFolderMethod[] {
  return platform === "darwin"
    ? ["isInApplicationsFolder", "moveToApplicationsFolder"]
    : [];
}
