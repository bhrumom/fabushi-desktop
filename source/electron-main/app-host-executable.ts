import { join, resolve } from "node:path";

export interface AppHostExecutableContext {
  readonly platform: NodeJS.Platform;
  readonly resourcesPath: string | undefined;
  readonly moduleDirectory: string;
}

/** Pure mapping shared by the Electron main and Coordinator carrier boundaries. */
export function resolveAppHostExecutable(
  env: NodeJS.ProcessEnv,
  context: AppHostExecutableContext,
): string {
  const override = env.MAHAYANA_APP_HOST_BIN;
  if (override != null && override.trim().length > 0) return override;
  const name = context.platform === "win32" ? "mahayana-app-host.exe" : "mahayana-app-host";
  if (env.SAND_PACKAGED === "1") {
    if (context.resourcesPath == null || context.resourcesPath.length === 0) {
      throw new Error("Packaged App Host resolution requires Electron resourcesPath.");
    }
    return join(context.resourcesPath, "bin", name);
  }
  return resolve(context.moduleDirectory, "..", "..", "resources", "bin", name);
}
