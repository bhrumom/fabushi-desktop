export function committedAgentName(initialValue: string, draftValue: string): string | null {
  const trimmed = draftValue.trim();
  return trimmed.length > 0 && trimmed !== initialValue ? trimmed : null;
}
