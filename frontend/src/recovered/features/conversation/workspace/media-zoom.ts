export interface WheelZoomAccumulation {
  readonly remainder: number;
  readonly steps: number;
}

export function normalizeWheelZoomDelta(deltaY: number, deltaMode: number): number {
  const multiplier = deltaMode === 1 ? 16 : deltaMode === 2 ? 100 : 1;
  const normalized = Number.isFinite(deltaY) ? deltaY * multiplier : 0;
  return -normalized;
}

export function accumulateWheelZoomSteps(
  previousRemainder: number,
  delta: number,
  threshold = 120,
): WheelZoomAccumulation {
  const boundedThreshold = Number.isFinite(threshold) && threshold > 0 ? threshold : 120;
  let remainder = (Number.isFinite(previousRemainder) ? previousRemainder : 0)
    + (Number.isFinite(delta) ? delta : 0);
  let steps = 0;
  while (Math.abs(remainder) >= boundedThreshold) {
    if (remainder > 0) {
      remainder -= boundedThreshold;
      steps += 1;
    } else {
      remainder += boundedThreshold;
      steps -= 1;
    }
  }
  return { remainder, steps };
}
