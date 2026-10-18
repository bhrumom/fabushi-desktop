import type { ConversationTranscriptEntry, DraftAttachment, TranscriptMessage } from "./model.ts";

/**
 * Canonical Transcript reconciliation helpers.
 *
 * These functions stay under the existing Conversation/Transcript owner. They
 * are used by the shipping renderer for baseline/live/pagination settlement;
 * they are not a second transcript store.
 */

function mergeAttachmentMetadata(current: DraftAttachment, incoming: DraftAttachment): DraftAttachment {
  const merged: DraftAttachment = {
    path: incoming.path.length > 0 ? incoming.path : current.path,
    name: incoming.name.length > 0 ? incoming.name : current.name,
  };
  const size = incoming.size ?? current.size;
  if (size !== undefined) merged.size = size;
  const mimeType = incoming.mimeType != null && incoming.mimeType.length > 0
    ? incoming.mimeType
    : current.mimeType;
  if (mimeType !== undefined) merged.mimeType = mimeType;
  return merged;
}

function mergeTranscriptAttachments(
  current: readonly DraftAttachment[] | undefined,
  incoming: readonly DraftAttachment[] | undefined,
): NonNullable<TranscriptMessage["attachments"]> {
  const merged = [...(current ?? [])];
  for (const attachment of incoming ?? []) {
    const index = merged.findIndex((candidate) =>
      candidate.path === attachment.path
      && (attachment.path.length > 0 || candidate.name === attachment.name)
    );
    if (index < 0) merged.push({ ...attachment });
    else merged[index] = mergeAttachmentMetadata(merged[index]!, attachment);
  }
  return merged;
}

function mergeTranscriptText(primary: string, secondary: string, preservePrimary: boolean): string {
  if (primary.length === 0) return secondary;
  if (secondary.length === 0) return primary;
  if (primary.startsWith(secondary)) return primary;
  if (secondary.startsWith(primary)) return preservePrimary ? primary : secondary;
  return primary;
}

export function mergeAuthoritativeTranscriptMessage(
  current: TranscriptMessage,
  incoming: TranscriptMessage,
  options: { preserveCurrentId?: boolean } = {},
): TranscriptMessage {
  // Lifecycle is monotonic. A previously observed terminal row outranks a
  // stale streaming replay, regardless of whether the caller is reconciling a
  // baseline, update, or append. Missing/empty fields may still be filled from
  // the other projection, but the stale replay cannot restore streaming state.
  const staleStreamingCannotResurrect = current.isStreaming !== true && incoming.isStreaming === true;
  const primary = staleStreamingCannotResurrect ? current : incoming;
  const secondary = staleStreamingCannotResurrect ? incoming : current;
  const merged: TranscriptMessage = {
    ...secondary,
    ...primary,
    id: options.preserveCurrentId || staleStreamingCannotResurrect ? current.id : incoming.id,
    text: mergeTranscriptText(primary.text, secondary.text, staleStreamingCannotResurrect),
    attachments: mergeTranscriptAttachments(secondary.attachments, primary.attachments),
    clientNonce: primary.clientNonce ?? secondary.clientNonce,
    delivery: staleStreamingCannotResurrect ? current.delivery : incoming.delivery ?? current.delivery,
    isStreaming: staleStreamingCannotResurrect ? false : incoming.isStreaming === true,
  };

  // Empty or truncated projections are not an explicit redaction protocol.
  // Keep richer serialized content until a versioned redaction contract exists.
  if ((primary.richText == null || primary.richText.length === 0) && secondary.richText != null && secondary.richText.length > 0) {
    merged.richText = secondary.richText;
  } else if (
    !staleStreamingCannotResurrect
    && primary.richText != null
    && secondary.richText != null
    && secondary.richText.startsWith(primary.richText)
  ) {
    merged.richText = secondary.richText;
  }
  if ((primary.images?.length ?? 0) === 0 && (secondary.images?.length ?? 0) > 0) {
    merged.images = secondary.images;
  }
  return merged;
}

export function transcriptEntriesCorrelate(
  left: ConversationTranscriptEntry,
  right: ConversationTranscriptEntry,
): boolean {
  if (left.id === right.id) return true;
  return left.kind === "message"
    && right.kind === "message"
    && left.role === right.role
    && left.clientNonce != null
    && left.clientNonce.length > 0
    && left.clientNonce === right.clientNonce;
}

function findCorrelatedEntryIndex(
  entries: readonly ConversationTranscriptEntry[],
  incoming: ConversationTranscriptEntry,
): number {
  const exactIndex = entries.findIndex((candidate) => candidate.id === incoming.id);
  if (exactIndex >= 0) return exactIndex;
  if (incoming.kind !== "message" || incoming.clientNonce == null || incoming.clientNonce.length === 0) return -1;

  const matches: number[] = [];
  entries.forEach((candidate, index) => {
    if (
      candidate.kind === "message"
      && candidate.role === incoming.role
      && candidate.clientNonce === incoming.clientNonce
    ) matches.push(index);
  });
  return matches.length === 1 ? matches[0]! : -1;
}

function mergeObservedLiveOverBaseline(
  live: TranscriptMessage,
  baseline: TranscriptMessage,
): TranscriptMessage {
  // The live event was observed after this feed subscription began. Terminal
  // state is monotonic; within the same lifecycle phase the live event wins
  // semantic fields while inheriting baseline-only metadata.
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
  const baseline: ConversationTranscriptEntry[] = [];
  for (const entry of incoming) {
    const duplicateIndex = findCorrelatedEntryIndex(baseline, entry);
    if (duplicateIndex >= 0) {
      const duplicate = baseline[duplicateIndex]!;
      if (duplicate.kind === "message" && entry.kind === "message") {
        baseline[duplicateIndex] = mergeAuthoritativeTranscriptMessage(duplicate, entry);
      } else if (duplicate.id === entry.id) {
        baseline[duplicateIndex] = entry;
      }
      continue;
    }

    const previousIndex = findCorrelatedEntryIndex(current, entry);
    if (previousIndex < 0) {
      baseline.push(entry);
      continue;
    }

    const previous = current[previousIndex]!;
    const preferObservedLive = preferObservedLiveIds.has(previous.id) || preferObservedLiveIds.has(entry.id);
    if (previous.kind === "message" && entry.kind === "message") {
      baseline.push(
        preferObservedLive
          ? mergeObservedLiveOverBaseline(previous, entry)
          : mergeAuthoritativeTranscriptMessage(previous, entry),
      );
    } else {
      baseline.push(preferObservedLive ? previous : entry);
    }
  }

  const unresolvedOptimistic = current.filter((entry): entry is TranscriptMessage =>
    entry.kind === "message"
    && entry.role === "user"
    && entry.clientNonce != null
    && entry.clientNonce.length > 0
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
 * the baseline are explicitly fenced so a stale correlated baseline cannot
 * overwrite newer live semantic state merely because it shares an id/nonce.
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
  const index = findCorrelatedEntryIndex(current, incoming);
  if (index < 0) {
    return options.allowInsert === true && incoming.kind === "message" ? [...current, incoming] : current;
  }

  const previous = current[index]!;
  const next = previous.kind === "message" && incoming.kind === "message"
    ? mergeAuthoritativeTranscriptMessage(previous, incoming)
    : incoming;
  const reconciled = [...current];
  reconciled[index] = next;
  return reconciled;
}

export function reconcileTranscriptLiveAppend(
  current: readonly ConversationTranscriptEntry[],
  incoming: ConversationTranscriptEntry,
  options: { resolvedNonce?: string | null } = {},
): readonly ConversationTranscriptEntry[] {
  const incomingMessage = incoming.kind === "message" ? incoming : null;
  const resolvedNonce = options.resolvedNonce ?? incomingMessage?.clientNonce ?? null;
  const pendingId = resolvedNonce == null ? null : "pending-" + resolvedNonce;
  let correlatedIndex = -1;

  if (incomingMessage != null) {
    correlatedIndex = current.findIndex((entry) =>
      entry.kind === "message" && entry.id === incomingMessage.id
    );
    if (correlatedIndex < 0 && pendingId != null) {
      correlatedIndex = current.findIndex((entry) =>
        entry.kind === "message" && entry.id === pendingId
      );
    }
    if (correlatedIndex < 0 && resolvedNonce != null && resolvedNonce.length > 0) {
      const matches: number[] = [];
      current.forEach((entry, index) => {
        if (
          entry.kind === "message"
          && entry.role === incomingMessage.role
          && entry.clientNonce === resolvedNonce
        ) matches.push(index);
      });
      if (matches.length > 1) return current;
      if (matches.length === 1) correlatedIndex = matches[0]!;
    }
  }

  if (correlatedIndex >= 0 && incomingMessage != null) {
    const correlated = current[correlatedIndex] as TranscriptMessage;
    const isAttachmentEcho = "sourceKind" in incomingMessage && incomingMessage.sourceKind === "user-attachment";
    const merged = mergeAuthoritativeTranscriptMessage(correlated, incomingMessage, {
      preserveCurrentId: isAttachmentEcho,
    });
    const reconciled: TranscriptMessage = {
      ...merged,
      ...(isAttachmentEcho ? { delivery: correlated.delivery ?? merged.delivery } : { delivery: "sent" as const }),
    };
    const next = [...current];
    next[correlatedIndex] = reconciled;
    return next;
  }

  if (current.some((entry) => entry.id === incoming.id)) return current;
  return [...current, incoming];
}
