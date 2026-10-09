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


export type HorizontalScrollWheelAxis = "horizontal" | "vertical";

export interface HorizontalScrollWheelLock {
  readonly axis: HorizontalScrollWheelAxis;
  readonly expiresAtMs: number;
}

export interface HorizontalScrollWheelUpdate {
  readonly lock: HorizontalScrollWheelLock | null;
  readonly axis: HorizontalScrollWheelAxis | null;
}

export function normalizeHorizontalScrollWheelDelta(
  deltaX: number,
  deltaY: number,
  deltaMode: number,
  pageWidth: number,
): { readonly x: number; readonly y: number } {
  const x = Number.isFinite(deltaX) ? deltaX : 0;
  const y = Number.isFinite(deltaY) ? deltaY : 0;
  const mode = Number.isFinite(deltaMode) ? deltaMode : 0;
  const page = Number.isFinite(pageWidth) && pageWidth > 0 ? pageWidth : 1;
  const scale = mode === 1 ? 16 : mode === 2 ? page : 1;
  return { x: x * scale, y: y * scale };
}

export function updateHorizontalScrollWheelLock(
  lock: HorizontalScrollWheelLock | null,
  deltaX: number,
  deltaY: number,
  nowMs: number,
  idleWindowMs = 160,
): HorizontalScrollWheelUpdate {
  const x = Number.isFinite(deltaX) ? deltaX : 0;
  const y = Number.isFinite(deltaY) ? deltaY : 0;
  const now = Number.isFinite(nowMs) ? nowMs : 0;
  const idle = Number.isFinite(idleWindowMs) && idleWindowMs > 0 ? idleWindowMs : 160;
  const live = lock != null && now <= lock.expiresAtMs ? lock : null;
  if (live != null) {
    return {
      axis: live.axis,
      lock: { axis: live.axis, expiresAtMs: now + idle },
    };
  }
  if (x === 0 && y === 0) return { axis: null, lock: null };
  const axis: HorizontalScrollWheelAxis = Math.abs(x) > Math.abs(y) ? "horizontal" : "vertical";
  return {
    axis,
    lock: { axis, expiresAtMs: now + idle },
  };
}
