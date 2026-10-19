export async function settleOptionalRemoteComputerTarget(
  ensureTarget: () => Promise<void>,
  onUnavailable: () => Promise<void>,
): Promise<"ready" | "unavailable"> {
  try {
    await ensureTarget();
    return "ready";
  } catch {
    await onUnavailable();
    return "unavailable";
  }
}
