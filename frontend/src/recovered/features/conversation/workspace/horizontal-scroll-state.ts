export interface HorizontalScrollSnapshot {
  readonly ownerId: string;
  readonly offset: number;
}

function finiteNonNegative(value: number): number {
  return Number.isFinite(value) ? Math.max(0, value) : 0;
}

export function captureHorizontalScroll(ownerId: string, offset: number): HorizontalScrollSnapshot {
  return { ownerId, offset: finiteNonNegative(offset) };
}

export function clampHorizontalScrollOffset(
  offset: number,
  scrollWidth: number,
  clientWidth: number,
): number {
  const maximum = Math.max(0, finiteNonNegative(scrollWidth) - finiteNonNegative(clientWidth));
  return Math.min(finiteNonNegative(offset), maximum);
}

export function restoreHorizontalScrollOffset(
  snapshot: HorizontalScrollSnapshot | null,
  ownerId: string,
  scrollWidth: number,
  clientWidth: number,
): number {
  if (snapshot == null || snapshot.ownerId !== ownerId) return 0;
  return clampHorizontalScrollOffset(snapshot.offset, scrollWidth, clientWidth);
}
