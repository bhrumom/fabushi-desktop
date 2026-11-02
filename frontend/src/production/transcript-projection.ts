import type { ConversationTranscriptEntry, DraftAttachment, TranscriptMessage } from "../recovered/features/conversation/workspace/model";

export type TranscriptReadUpdateBuffer = {
  readonly accountSlot: string | null;
  readonly agentId: string;
  readonly isCurrent: () => boolean;
  readonly updates: Map<string, ConversationTranscriptEntry>;
};

/** Expired reads must not project their buffered events into a newer snapshot. */
export function currentTranscriptReadBuffers(
  reads: Set<TranscriptReadUpdateBuffer>,
  accountSlot: string | null,
  agentId: string,
): TranscriptReadUpdateBuffer[] {
  const current: TranscriptReadUpdateBuffer[] = [];
  for (const read of reads) {
    if (!read.isCurrent()) {
      reads.delete(read);
    } else if (read.accountSlot === accountSlot && read.agentId === agentId) {
      current.push(read);
    }
  }
  return current;
}

function mergeTranscriptAttachments(
  current: readonly DraftAttachment[] | undefined,
  incoming: readonly DraftAttachment[] | undefined,
): DraftAttachment[] {
  const merged = [...(current ?? [])];
  for (const attachment of incoming ?? []) {
    const index = merged.findIndex((candidate) => candidate.path === attachment.path);
    if (index < 0) merged.push(attachment);
    else merged[index] = { ...merged[index], ...attachment };
  }
  return merged;
}

export function mergeAuthoritativeTranscriptMessage(
  current: TranscriptMessage,
  incoming: TranscriptMessage,
  options: { preserveCurrentId?: boolean } = {},
): TranscriptMessage {
  return {
    ...current,
    ...incoming,
    id: options.preserveCurrentId ? current.id : incoming.id,
    text: incoming.text.length > 0 ? incoming.text : current.text,
    attachments: mergeTranscriptAttachments(current.attachments, incoming.attachments),
    clientNonce: incoming.clientNonce ?? current.clientNonce,
    delivery: incoming.delivery ?? current.delivery,
  };
}

export function transcriptEntriesCorrelate(left: ConversationTranscriptEntry, right: ConversationTranscriptEntry): boolean {
  if (left.id === right.id) return true;
  return left.kind === "message" && right.kind === "message"
    && left.clientNonce != null && right.clientNonce != null
    && left.clientNonce === right.clientNonce;
}

/** Apply updates captured during this read only to rows in its returned window. */
export function applyTranscriptReadUpdates(
  entries: readonly ConversationTranscriptEntry[],
  updates: ReadonlyMap<string, ConversationTranscriptEntry>,
): ConversationTranscriptEntry[] {
  return entries.map((entry) => {
    const update = updates.get(entry.id);
    if (update == null) return entry;
    return entry.kind === "message" && update.kind === "message"
      ? mergeAuthoritativeTranscriptMessage(entry, update)
      : update;
  });
}

function entryIndex(entries: readonly ConversationTranscriptEntry[]) {
  const byId = new Map(entries.map((entry) => [entry.id, entry]));
  const byNonce = new Map<string, ConversationTranscriptEntry>();
  for (const entry of entries) {
    if (entry.kind === "message" && entry.clientNonce != null && !byNonce.has(entry.clientNonce)) {
      byNonce.set(entry.clientNonce, entry);
    }
  }
  return (entry: ConversationTranscriptEntry): ConversationTranscriptEntry | undefined =>
    byId.get(entry.id) ?? (entry.kind === "message" && entry.clientNonce != null ? byNonce.get(entry.clientNonce) : undefined);
}

/** Full Host snapshot in canonical order, retaining unresolved local sends. */
export function reconcileAuthoritativeTranscriptBaseline(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  const currentEntry = entryIndex(current);
  const seen = new Set<string>();
  const baseline: ConversationTranscriptEntry[] = [];
  for (const entry of incoming) {
    if (seen.has(entry.id)) continue;
    seen.add(entry.id);
    const previous = currentEntry(entry);
    baseline.push(previous?.kind === "message" && entry.kind === "message"
      ? mergeAuthoritativeTranscriptMessage(previous, entry)
      : entry);
  }
  const baselineEntry = entryIndex(baseline);
  for (const entry of current) {
    if (entry.kind === "message" && entry.role === "user" && entry.clientNonce != null
      && entry.id.startsWith("pending-") && baselineEntry(entry) == null && !seen.has(entry.id)) {
      seen.add(entry.id);
      baseline.push(entry);
    }
  }
  return baseline;
}

/** Live events received after snapshot acquisition win, including the same ID. */
export function reconcileTranscriptFeedBaseline(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
  liveEntryIds: ReadonlySet<string>,
): ConversationTranscriptEntry[] {
  const currentEntry = entryIndex(current);
  const baseline = reconcileAuthoritativeTranscriptBaseline(current, incoming).map((entry) => {
    const live = currentEntry(entry);
    if (live == null || !liveEntryIds.has(live.id)) return entry;
    // A real acknowledgement still replaces an optimistic nonce placeholder.
    if (live.id.startsWith("pending-") && !entry.id.startsWith("pending-")) return entry;
    return live.kind === "message" && entry.kind === "message"
      ? mergeAuthoritativeTranscriptMessage(entry, live)
      : live;
  });
  const baselineEntry = entryIndex(baseline);
  const seen = new Set(baseline.map((entry) => entry.id));
  for (const entry of current) {
    if (liveEntryIds.has(entry.id) && !seen.has(entry.id) && baselineEntry(entry) == null) {
      seen.add(entry.id);
      baseline.push(entry);
    }
  }
  return baseline;
}

export function reconcileLateInitialTranscriptPage(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  return reconcileTranscriptTail(current, incoming, new Set(current.map((entry) => entry.id)));
}

/** A tail/feed window does not delete or reorder previously loaded history. */
export function reconcileTranscriptTail(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
  liveEntryIds: ReadonlySet<string>,
  entriesAtRequest: readonly ConversationTranscriptEntry[] = [],
): ConversationTranscriptEntry[] {
  const resolved = reconcileTranscriptFeedBaseline(current, incoming, liveEntryIds);
  const resolvedEntry = entryIndex(resolved);
  const incomingEntry = entryIndex(incoming);
  const priorEntry = entryIndex(entriesAtRequest);
  const positions = new Map(current.map((entry, index) => [entry.id, index]));
  const currentEntry = entryIndex(current);
  const anchorPosition = (entry: ConversationTranscriptEntry): number | undefined => {
    const match = currentEntry(entry);
    return match == null ? undefined : positions.get(match.id);
  };
  const firstAnchor = incoming.map(anchorPosition).find((position) => position != null);
  const result: ConversationTranscriptEntry[] = [];
  const seen = new Set<string>();
  const append = (entry: ConversationTranscriptEntry) => {
    if (seen.has(entry.id)) return;
    seen.add(entry.id);
    result.push(entry);
  };
  const appendCached = (entry: ConversationTranscriptEntry) => {
    if (incomingEntry(entry) == null) append(entry);
  };
  if (firstAnchor == null) {
    // A disjoint tail is newer than the previously cached page, but events
    // first observed during this read may be newer than the tail itself.
    const appendedDuringRead = (entry: ConversationTranscriptEntry) =>
      (liveEntryIds.has(entry.id) && priorEntry(entry) == null)
      || (entry.kind === "message" && entry.id.startsWith("pending-"));
    current.filter((entry) => !appendedDuringRead(entry)).forEach(appendCached);
    incoming.forEach((entry) => append(resolvedEntry(entry) ?? entry));
    current.filter(appendedDuringRead).forEach(appendCached);
    return result;
  }
  let currentOffset = firstAnchor;
  current.slice(0, firstAnchor).forEach(appendCached);
  for (const entry of incoming) {
    const anchor = anchorPosition(entry);
    if (anchor != null && anchor >= currentOffset) {
      current.slice(currentOffset, anchor).forEach(appendCached);
      currentOffset = anchor + 1;
    }
    append(resolvedEntry(entry) ?? entry);
  }
  current.slice(currentOffset).forEach(appendCached);
  return result;
}

/** Unchanged cached rows refresh; rows changed during this read retain the event. */
export function reconcileTranscriptRefresh(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
  entriesAtRequest: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  const priorEntry = entryIndex(entriesAtRequest);
  const changedIds = new Set(current.filter((entry) => priorEntry(entry) !== entry).map((entry) => entry.id));
  return reconcileTranscriptFeedBaseline(current, incoming, changedIds);
}
