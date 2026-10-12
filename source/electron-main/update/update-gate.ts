export type UpdateDisabledReason = "disabled-by-env" | "lab-build" | "not-packaged" | "unsupported-platform";
export function computeUpdateDisabledReason(input: { readonly envDisabled: boolean; readonly isLabBuild: boolean; readonly hasDevFeedOverride: boolean; readonly isPackaged: boolean; readonly platform: NodeJS.Platform }): UpdateDisabledReason | null {
  if (input.envDisabled) return "disabled-by-env";
  if (input.isLabBuild) return "lab-build";
  // A development feed may change where a supported updater reads offers from,
  // but it must never manufacture an apply mechanism for an unsupported platform.
  if (input.platform !== "darwin" && input.platform !== "win32") return "unsupported-platform";
  if (input.hasDevFeedOverride) return null;
  if (!input.isPackaged) return "not-packaged";
  return null;
}
