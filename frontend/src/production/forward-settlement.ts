import type { ForwardSettlement } from "./ForwardMessageDialog";

// An asynchronously obtained roster is only a projection. The account
// generation used for the successful forward must still be current when
// the list read resolves; the Host/Session remains authoritative.
export async function settleForwardProjection(input: {
  readonly settlement: readonly ForwardSettlement[];
  readonly recipientNames?: ReadonlyMap<string, string>;
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly setNotice: (notice: string | null) => void;
  readonly readRoster: (() => Promise<unknown>) | null;
  readonly applyRoster: (value: unknown) => void;
}): Promise<void> {
  const { settlement, accountScopeGeneration, isAccountScopeCurrent, setNotice, readRoster, applyRoster } = input;
  if (!isAccountScopeCurrent(accountScopeGeneration)) return;
  const sent = settlement.filter((item) => item.status === "sent");
  const failed = settlement.length - sent.length;
  const attention = `${failed} destination${failed === 1 ? "" : "s"} still need${failed === 1 ? "s" : ""} attention.`;
  if (settlement.length === 0) {
    setNotice("No forwarding destinations were confirmed.");
  } else if (sent.length === 0) {
    setNotice(`No deliveries confirmed; ${attention}`);
  } else if (failed > 0) {
    setNotice(`Forwarded to ${sent.length}; ${attention}`);
  } else {
    // Names explain this accepted settlement; they never choose or authorize
    // destinations. Missing labels retain the truthful count projection.
    const names = sent.map((item) => input.recipientNames?.get(item.conversationId)?.trim());
    const hasNamedRecipients = sent.length <= 2 && names.every((name) => name != null && name.length > 0);
    setNotice(hasNamedRecipients
      ? `Forwarded to ${names.join(" and ")}.`
      : `Forwarded to ${sent.length} conversation${sent.length === 1 ? "" : "s"}.`);
  }
  if (readRoster == null) return;
  try {
    const value = await readRoster();
    if (isAccountScopeCurrent(accountScopeGeneration)) applyRoster(value);
  } catch {
    // Preserve the original successful-forward result. The next normal roster
    // convergence can repair a failed read, but a stale read cannot publish.
  }
}
