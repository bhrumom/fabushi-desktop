export type ForwardRecipientNavigationKey =
  | "ArrowDown"
  | "ArrowUp"
  | "PageDown"
  | "PageUp"
  | "Home"
  | "End";

export function nextForwardRecipientIndex(
  currentIndex: number,
  recipientCount: number,
  key: ForwardRecipientNavigationKey,
  pageSize = 6,
): number | null {
  if (!Number.isInteger(recipientCount) || recipientCount <= 0) return null;
  const last = recipientCount - 1;
  const current = Math.min(Math.max(Number.isInteger(currentIndex) ? currentIndex : 0, 0), last);
  const page = Math.max(1, Math.trunc(pageSize));
  switch (key) {
    case "ArrowDown":
      return Math.min(current + 1, last);
    case "ArrowUp":
      return Math.max(current - 1, 0);
    case "PageDown":
      return Math.min(current + page, last);
    case "PageUp":
      return Math.max(current - page, 0);
    case "Home":
      return 0;
    case "End":
      return last;
  }
}

export function isForwardRecipientNavigationKey(
  key: string,
): key is ForwardRecipientNavigationKey {
  return key === "ArrowDown"
    || key === "ArrowUp"
    || key === "PageDown"
    || key === "PageUp"
    || key === "Home"
    || key === "End";
}

export function isForwardSubmitShortcut(input: {
  readonly key: string;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
}): boolean {
  return input.key === "Enter" && (input.ctrlKey || input.metaKey);
}

export function isForwardToggleShortcut(input: {
  readonly key: string;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
}): boolean {
  return (input.key === "Enter" && !input.ctrlKey && !input.metaKey)
    || input.key === " ";
}
