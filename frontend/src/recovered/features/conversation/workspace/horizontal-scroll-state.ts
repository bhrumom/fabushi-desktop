export interface HorizontalScrollSnapshot {
  readonly ownerId: string;
  readonly offset: number;
}

export interface HorizontalScrollPointerGesture {
  readonly pointerId: number;
  readonly startX: number;
  readonly startY: number;
  readonly lastX: number;
  readonly lastY: number;
  readonly active: boolean;
}

export interface HorizontalScrollPointerUpdate {
  readonly gesture: HorizontalScrollPointerGesture | null;
  readonly deltaX: number;
  readonly decision: "pending" | "horizontal" | "vertical" | "ignored";
}

function finiteNonNegative(value: number): number {
  return Number.isFinite(value) ? Math.max(0, value) : 0;
}

function finiteCoordinate(value: number): number {
  return Number.isFinite(value) ? value : 0;
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

export function beginHorizontalScrollPointer(
  pointerId: number,
  x: number,
  y: number,
): HorizontalScrollPointerGesture {
  const startX = finiteCoordinate(x);
  const startY = finiteCoordinate(y);
  return {
    pointerId,
    startX,
    startY,
    lastX: startX,
    lastY: startY,
    active: false,
  };
}

export function updateHorizontalScrollPointer(
  gesture: HorizontalScrollPointerGesture | null,
  pointerId: number,
  x: number,
  y: number,
  threshold = 4,
): HorizontalScrollPointerUpdate {
  if (gesture == null || gesture.pointerId !== pointerId) {
    return { gesture, deltaX: 0, decision: "ignored" };
  }
  const nextX = finiteCoordinate(x);
  const nextY = finiteCoordinate(y);
  if (gesture.active) {
    return {
      gesture: { ...gesture, lastX: nextX, lastY: nextY },
      deltaX: gesture.lastX - nextX,
      decision: "horizontal",
    };
  }
  const deltaX = nextX - gesture.startX;
  const deltaY = nextY - gesture.startY;
  const boundedThreshold = Number.isFinite(threshold) && threshold > 0 ? threshold : 4;
  if (Math.abs(deltaX) + Math.abs(deltaY) < boundedThreshold) {
    return {
      gesture: { ...gesture, lastX: nextX, lastY: nextY },
      deltaX: 0,
      decision: "pending",
    };
  }
  if (Math.abs(deltaX) <= Math.abs(deltaY)) {
    return { gesture: null, deltaX: 0, decision: "vertical" };
  }
  return {
    gesture: { ...gesture, lastX: nextX, lastY: nextY, active: true },
    deltaX: gesture.startX - nextX,
    decision: "horizontal",
  };
}
