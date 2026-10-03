import type {
  TranscriptFeedHandlers,
  TranscriptFeedSource,
} from "./reaction-feed.ts";

export type TranscriptClientUnsubscribe =
  | (() => void)
  | { dispose(): void };

export interface TranscriptClientEventSource {
  subscribe(
    family: "transcript",
    listener: (value: unknown) => void,
  ): TranscriptClientUnsubscribe;
}

export interface TranscriptFeedFanout extends TranscriptFeedSource {
  dispose(): void;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function nonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function entryId(value: unknown): string | null {
  const row = asRecord(value);
  return nonEmptyString(row?.id) ? row.id : null;
}

function validEntries(value: unknown): readonly unknown[] | null {
  if (!Array.isArray(value)) return null;
  const seen = new Set<string>();
  for (const entry of value) {
    const id = entryId(entry);
    if (id == null || seen.has(id)) return null;
    seen.add(id);
  }
  return value;
}

function eventAgentId(row: Record<string, unknown>): string | null {
  if (nonEmptyString(row.agentId)) return row.agentId;
  if (nonEmptyString(row.owningAgentId)) return row.owningAgentId;
  return null;
}

function release(subscription: TranscriptClientUnsubscribe): void {
  if (typeof subscription === "function") subscription();
  else subscription.dispose();
}

export function createTranscriptFeedFanout(
  source: TranscriptClientEventSource | null | undefined,
): TranscriptFeedFanout | null {
  if (typeof source?.subscribe !== "function") return null;

  const observers = new Set<TranscriptFeedHandlers>();
  let disposed = false;
  let subscription: TranscriptClientUnsubscribe;

  const dispatch = (value: unknown): void => {
    if (disposed) return;
    const row = asRecord(value);
    if (row == null || typeof row.type !== "string") return;

    if (row.type === "snapshot") {
      if (!nonEmptyString(row.activeAgentId)) return;
      const entries = validEntries(row.entries);
      if (entries == null) return;
      for (const observer of Array.from(observers)) {
        observer.onBaseline({ agentId: row.activeAgentId, entries });
      }
      return;
    }

    const agentId = eventAgentId(row);
    if (agentId == null) return;

    if (row.type === "appended") {
      if (entryId(row.entry) == null) return;
      for (const observer of Array.from(observers)) {
        observer.onAppended({ agentId, entry: row.entry });
      }
      return;
    }

    if (row.type === "updated") {
      if (entryId(row.entry) == null) return;
      if (row.before !== undefined && entryId(row.before) == null) return;
      for (const observer of Array.from(observers)) {
        observer.onUpdated({
          agentId,
          ...(row.before === undefined ? {} : { before: row.before }),
          after: row.entry,
        });
      }
      return;
    }

    if (row.type === "cleared") {
      for (const observer of Array.from(observers)) {
        observer.onCleared(agentId);
      }
    }
  };

  try {
    subscription = source.subscribe("transcript", dispatch);
  } catch {
    return null;
  }

  return {
    observeEntriesFeed(handlers) {
      if (disposed) return () => {};
      observers.add(handlers);
      return () => {
        observers.delete(handlers);
      };
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      release(subscription);
      observers.clear();
    },
  };
}
