export interface SandTabStateItem {
  readonly id: string;
  readonly disabled?: boolean;
  readonly reorderLocked?: boolean;
}
export type SandTabReorderState = "started" | "applied" | "cancelled";
export interface SandTabReorderUpdate {
  readonly id: string;
  readonly oldPosition: number;
  readonly newPosition: number;
  readonly state: SandTabReorderState;
}
export function resolveSandTabNavigation(items: readonly SandTabStateItem[], index: number, key: string, orientation: "horizontal" | "vertical" = "horizontal"): string | null {
  if (items.length === 0) return null;
  if (key === "Home") return items.find((item) => !item.disabled)?.id ?? null;
  if (key === "End") return [...items].reverse().find((item) => !item.disabled)?.id ?? null;
  const forward = orientation === "vertical" ? "ArrowDown" : "ArrowRight";
  const backward = orientation === "vertical" ? "ArrowUp" : "ArrowLeft";
  const direction = key === forward ? 1 : key === backward ? -1 : 0;
  if (!direction) return null;
  for (let step = 1; step <= items.length; step += 1) {
    const next = (index + direction * step + items.length) % items.length;
    if (!items[next]?.disabled) return items[next]!.id;
  }
  return null;
}
export function resolveSandTabReorderTarget(items: readonly SandTabStateItem[], from: number, requested: number): number {
  if (from < 0 || from >= items.length || items[from]?.reorderLocked || items[from]?.disabled) return from;
  const target = Math.max(0, Math.min(requested, items.length - 1));
  if (target === from) return from;
  const direction = target > from ? 1 : -1;
  let lastAllowed = from;
  for (let index = from + direction; direction > 0 ? index <= target : index >= target; index += direction) {
    const item = items[index];
    if (item == null || item.reorderLocked) break;
    lastAllowed = index;
  }
  return lastAllowed;
}
