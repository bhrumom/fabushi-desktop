import type { ForwardSettlement } from "./ForwardMessageDialog";

export interface ForwardRosterRecipient {
  readonly id: string;
  readonly name: string;
}

function recipientNamesForSettlement(
  settlement: readonly ForwardSettlement[],
  roster: readonly ForwardRosterRecipient[],
): readonly string[] | null {
  const sent = settlement.filter((item) => item.status === "sent");
  if (sent.length === 0 || sent.length > 2) return null;

  const sentIds = sent.map((item) => item.conversationId);
  if (new Set(sentIds).size !== sentIds.length) return null;

  const occurrences = new Map<string, number>();
  const names = new Map<string, string>();
  for (const recipient of roster) {
    const id = typeof recipient.id === "string" ? recipient.id : "";
    if (id.trim().length === 0) continue;
    occurrences.set(id, (occurrences.get(id) ?? 0) + 1);
    const name = typeof recipient.name === "string" ? recipient.name.trim() : "";
    if (name.length > 0 && !names.has(id)) names.set(id, name);
  }

  const resolved: string[] = [];
  for (const id of sentIds) {
    if (occurrences.get(id) !== 1) return null;
    const name = names.get(id);
    if (name == null) return null;
    resolved.push(name);
  }
  return resolved;
}

export function formatForwardNotice(
  settlement: readonly ForwardSettlement[],
  roster: readonly ForwardRosterRecipient[] = [],
): string {
  const sentCount = settlement.filter((item) => item.status === "sent").length;
  const failedCount = settlement.filter((item) => item.status === "failed").length;
  const names = recipientNamesForSettlement(settlement, roster);

  let summary: string;
  if (sentCount === 0) {
    summary = "No destinations were forwarded.";
  } else if (sentCount === 1) {
    summary = names == null ? "Forwarded to 1 conversation." : `Forwarded to ${names[0]}.`;
  } else if (sentCount === 2) {
    summary = names == null ? "Forwarded to 2 conversations." : `Forwarded to ${names[0]} and ${names[1]}.`;
  } else {
    summary = `Forwarded to ${sentCount} conversations.`;
  }

  if (failedCount === 0) return summary;
  const suffix = `${failedCount} destination${failedCount === 1 ? "" : "s"} still need${failedCount === 1 ? "s" : ""} attention.`;
  return `${summary.slice(0, -1)}; ${suffix}`;
}

export async function settleForwardProjection({
  settlement,
  accountScopeGeneration,
  isAccountScopeCurrent,
  setNotice,
  readRoster,
  applyRoster,
}: {
  readonly settlement: readonly ForwardSettlement[];
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly setNotice: (notice: string | null) => void;
  readonly readRoster?: (() => Promise<unknown>) | null;
  readonly applyRoster: (value: unknown) => readonly ForwardRosterRecipient[];
}): Promise<void> {
  const fallbackNotice = formatForwardNotice(settlement);
  if (!isAccountScopeCurrent(accountScopeGeneration)) return;
  if (readRoster == null) {
    setNotice(fallbackNotice);
    return;
  }

  try {
    const value = await readRoster();
    if (!isAccountScopeCurrent(accountScopeGeneration)) return;
    const recipients = applyRoster(value);
    if (!isAccountScopeCurrent(accountScopeGeneration)) return;
    setNotice(formatForwardNotice(settlement, recipients));
  } catch {
    if (isAccountScopeCurrent(accountScopeGeneration)) setNotice(fallbackNotice);
  }
}
