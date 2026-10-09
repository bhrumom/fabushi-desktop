export const TRANSCRIPT_FOLLOW_LATEST_THRESHOLD_PX = 96;

export interface TranscriptScrollMetrics {
  readonly scrollTop: number;
  readonly clientHeight: number;
  readonly scrollHeight: number;
}

export function isTranscriptNearBottom(
  metrics: TranscriptScrollMetrics,
  thresholdPx = TRANSCRIPT_FOLLOW_LATEST_THRESHOLD_PX,
): boolean {
  if (!Number.isFinite(thresholdPx) || thresholdPx < 0) return false;
  const scrollTop = Number.isFinite(metrics.scrollTop) ? Math.max(0, metrics.scrollTop) : 0;
  const clientHeight = Number.isFinite(metrics.clientHeight) ? Math.max(0, metrics.clientHeight) : 0;
  const scrollHeight = Number.isFinite(metrics.scrollHeight) ? Math.max(0, metrics.scrollHeight) : 0;
  return Math.max(0, scrollHeight - clientHeight - scrollTop) <= thresholdPx;
}
