export type TranscriptDeliveryState = "sent" | "scheduled" | "pending" | "queued" | "dispatching" | "failed";

export function normalizeTranscriptDelivery(candidate: unknown, pending = false, failed = false): TranscriptDeliveryState | undefined {
  if (
    candidate === "pending"
    || candidate === "queued"
    || candidate === "dispatching"
    || candidate === "failed"
    || candidate === "sent"
    || candidate === "scheduled"
  ) return candidate;
  if (pending) return "pending";
  if (failed) return "failed";
  return undefined;
}

export function isTranscriptDeliveryBusy(delivery: TranscriptDeliveryState | undefined): boolean {
  return delivery === "pending" || delivery === "queued" || delivery === "dispatching";
}

export function isTranscriptDeliveryActionable(delivery: TranscriptDeliveryState | undefined): boolean {
  return delivery !== "failed" && !isTranscriptDeliveryBusy(delivery);
}
