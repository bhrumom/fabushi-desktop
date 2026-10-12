export type UpdateDisabledReason = "disabled-by-env" | "lab-build" | "not-packaged" | "unsupported-platform";
export function computeUpdateDisabledReason(input: { readonly envDisabled: boolean; readonly isLabBuild: boolean; readonly hasDevFeedOverride: boolean; readonly releaseUpdaterAvailable?: boolean; readonly supportsLinuxAppImage?: boolean; readonly isPackaged: boolean; readonly platform: NodeJS.Platform }): UpdateDisabledReason | null {
  if (input.envDisabled) return "disabled-by-env";
  if (input.isLabBuild) return "lab-build";
  const legacySupported = input.hasDevFeedOverride && (input.platform === "darwin" || input.platform === "win32");
  const releaseSupported = input.releaseUpdaterAvailable === true
    && (input.platform === "darwin" || input.platform === "win32" || input.platform === "linux" && input.supportsLinuxAppImage === true);
  if (!legacySupported && !releaseSupported) return "unsupported-platform";
  if (input.hasDevFeedOverride) return null;
  if (!input.isPackaged) return "not-packaged";
  return null;
}
