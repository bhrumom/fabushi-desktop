import type { ForwardSettlement } from "./ForwardMessageDialog";

// An asynchronously obtained roster is only a projection. The account
// generation used for the successful forward must still be current when
// the list read resolves; the Host/Session remains authoritative.
export async function settleForwardProjection(input: {
  readonly settlement: readonly ForwardSettlement[];
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly setNotice: (notice: string | null) => void;
  readonly readRoster: (() => Promise<unknown>) | null;
  readonly applyRoster: (value: unknown) => void;
}): Promise<void> {
  const { settlement, accountScopeGeneration, isAccountScopeCurrent, setNotice, readRoster, applyRoster } = input;
  if (!isAccountScopeCurrent(accountScopeGeneration)) return;
  const sent = settlement.filter((item) => item.status === "sent").length;
  const failed = settlement.length - sent;
  setNotice(failed === 0
    ? `Forwarded to ${sent} conversation${sent === 1 ? "" : "s"}.`
    : `Forwarded to ${sent}; ${failed} destination${failed === 1 ? "" : "s"} still need attention.`);
  if (readRoster == null) return;
  try {
    const value = await readRoster();
    if (isAccountScopeCurrent(accountScopeGeneration)) applyRoster(value);
  } catch {
    // Preserve the original successful-forward result. The next normal roster
    // convergence can repair a failed read, but a stale read cannot publish.
  }
}
