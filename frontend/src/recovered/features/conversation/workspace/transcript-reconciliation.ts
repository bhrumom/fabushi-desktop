import type { ConversationTranscriptEntry, TranscriptMessage } from "./model.ts";

/**
 * Canonical Transcript reconciliation helpers.
 *
 * These functions stay under the existing Conversation/Transcript owner. They
 * are used by the shipping renderer for baseline/live/pagination settlement;
 * they are not a second transcript store.
 */

function mergeTranscriptAttachments(
  current: TranscriptMessage["attachments"],
  incoming: TranscriptMessage["attachments"],
): NonNullable<TranscriptMessage["attachments"]> {
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
    // A durable/final row settles streaming. Absence of isStreaming must not
    // inherit a stale true value from an earlier optimistic/live projection.
    isStreaming: incoming.isStreaming === true,
  };
}

function mergeMonotonicLiveTranscriptMessage(
  current: TranscriptMessage,
  incoming: TranscriptMessage,
  options: { preserveCurrentId?: boolean } = {},
): TranscriptMessage {
  if (current.isStreaming !== true && incoming.isStreaming === true) {
    const merged = mergeAuthoritativeTranscriptMessage(incoming, current, {
      preserveCurrentId: options.preserveCurrentId,
    });
    return options.preserveCurrentId ? { ...merged, id: current.id } : merged;
  }
  return mergeAuthoritativeTranscriptMessage(current, incoming, options);
}

export function transcriptEntriesCorrelate(
  left: ConversationTranscriptEntry,
  right: ConversationTranscriptEntry,
): boolean {
  if (left.id === right.id) return true;
  return left.kind === "message"
    && right.kind === "message"
    && left.clientNonce != null
    && right.clientNonce != null
    && left.clientNonce === right.clientNonce;
}

function correlatedCurrentMessage(
  current: readonly ConversationTranscriptEntry[],
  incoming: TranscriptMessage,
): TranscriptMessage | undefined {
  return current.find((candidate): candidate is TranscriptMessage =>
    candidate.kind === "message"
    && (
      candidate.id === incoming.id
      || (incoming.clientNonce != null && candidate.clientNonce === incoming.clientNonce)
    )
  );
}

function mergeObservedLiveOverBaseline(
  live: TranscriptMessage,
  baseline: TranscriptMessage,
): TranscriptMessage {
  // Terminal is monotonic: a stale streaming snapshot can never resurrect a
  // terminal live row. If both rows have the same lifecycle phase, the live
  // event was observed after this feed subscription began and therefore wins
  // semantic fields while still inheriting baseline-only metadata.
  if (live.isStreaming === true && baseline.isStreaming !== true) {
    return mergeAuthoritativeTranscriptMessage(live, baseline);
  }
  return mergeAuthoritativeTranscriptMessage(baseline, live);
}

function reconcileBaseline(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
  preferObservedLiveIds: ReadonlySet<string>,
): ConversationTranscriptEntry[] {
  const baseline = incoming.map((entry) => {
    if (entry.kind !== "message") {
      const previous = current.find((candidate) => candidate.id === entry.id);
      return previous != null && preferObservedLiveIds.has(previous.id) ? previous : entry;
    }
    const previous = correlatedCurrentMessage(current, entry);
    if (previous == null) return entry;
    return preferObservedLiveIds.has(previous.id)
      ? mergeObservedLiveOverBaseline(previous, entry)
      : mergeAuthoritativeTranscriptMessage(previous, entry);
  });

  const unresolvedOptimistic = current.filter((entry): entry is TranscriptMessage =>
    entry.kind === "message"
    && entry.role === "user"
    && entry.clientNonce != null
    && entry.id.startsWith("pending-")
    && !incoming.some((incomingEntry) => transcriptEntriesCorrelate(incomingEntry, entry))
  );

  return [...baseline, ...unresolvedOptimistic];
}

export function reconcileAuthoritativeTranscriptBaseline(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  return reconcileBaseline(current, incoming, new Set());
}

/**
 * A feed append/update may race its initial snapshot. Entries observed before
 * the baseline are explicitly fenced so that a stale correlated baseline
 * cannot overwrite newer live semantic state merely because it shares an id.
 */
export function reconcileTranscriptFeedBaseline(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
  preBaselineEntryIds: ReadonlySet<string>,
): ConversationTranscriptEntry[] {
  const baseline = reconcileBaseline(current, incoming, preBaselineEntryIds);
  const arrivedAfterSnapshot = current.filter((entry) =>
    preBaselineEntryIds.has(entry.id)
    && !baseline.some((candidate) => transcriptEntriesCorrelate(candidate, entry))
  );
  return [...baseline, ...arrivedAfterSnapshot];
}

export function reconcileLateInitialTranscriptPage(
  current: readonly ConversationTranscriptEntry[],
  incoming: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  const baseline = reconcileAuthoritativeTranscriptBaseline(current, incoming);
  const arrivedWhileLoading = current.filter((currentEntry) =>
    !incoming.some((incomingEntry) => transcriptEntriesCorrelate(incomingEntry, currentEntry))
  );
  return [...baseline, ...arrivedWhileLoading];
}

export function reconcileTranscriptLiveUpdate(
  current: readonly ConversationTranscriptEntry[],
  incoming: ConversationTranscriptEntry,
  options: { allowInsert?: boolean } = {},
): readonly ConversationTranscriptEntry[] {
  const index = current.findIndex((entry) => entry.id === incoming.id);
  if (index < 0) return options.allowInsert === true ? [...current, incoming] : current;

  const previous = current[index];
  const next = previous.kind === "message" && incoming.kind === "message"
    ? mergeMonotonicLiveTranscriptMessage(previous, incoming)
    : incoming;
  return current.map((entry, entryIndex) => entryIndex === index ? next : entry);
}

export function reconcileTranscriptLiveAppend(
  current: readonly ConversationTranscriptEntry[],
  incoming: ConversationTranscriptEntry,
  options: { resolvedNonce?: string | null } = {},
): readonly ConversationTranscriptEntry[] {
  const incomingMessage = incoming.kind === "message" ? incoming : null;
  const resolvedNonce = options.resolvedNonce ?? null;
  const pendingId = resolvedNonce == null ? null : `pending-${resolvedNonce}`;
  const correlated = incomingMessage == null ? null : current.find((entry): entry is TranscriptMessage =>
    entry.kind === "message"
    && (
      entry.id === incoming.id
      || (pendingId != null && entry.id === pendingId)
      || (resolvedNonce != null && entry.clientNonce === resolvedNonce)
    )
  );

  if (correlated != null && incomingMessage != null) {
    const isAttachmentEcho = "sourceKind" in incomingMessage && incomingMessage.sourceKind === "user-attachment";
    const merged = mergeMonotonicLiveTranscriptMessage(correlated, incomingMessage, {
      preserveCurrentId: isAttachmentEcho,
    });
    const reconciled: TranscriptMessage = {
      ...merged,
      ...(isAttachmentEcho ? { delivery: correlated.delivery } : { delivery: "sent" }),
    };
    return current.map((entry) =>
      entry.kind === "message" && entry.id === correlated.id ? reconciled : entry
    );
  }

  if (current.some((entry) => entry.id === incoming.id)) return current;
  return [...current, incoming];
}
