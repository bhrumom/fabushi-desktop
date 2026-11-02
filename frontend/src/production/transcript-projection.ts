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
  if (isCanonicalHumanSend(current) && isLocalSendPlaceholder(incoming)
    && transcriptEntriesCorrelate(current, incoming)) {
    const attachments = current.attachments ?? incoming.attachments;
    return {
      ...incoming,
      ...current,
      ...(attachments === undefined ? {} : { attachments }),
    };
  }
  // A Human snapshot can explicitly clear content/resources. Omitted optional
  // fields retain their partial-update meaning; Agent attachment echoes extend.
  const human = incoming.authorKind === "human";
  const attachments = human
    ? incoming.attachments ?? current.attachments
    : current.attachments === undefined && incoming.attachments === undefined
      ? undefined : mergeTranscriptAttachments(current.attachments, incoming.attachments);
  const clientNonce = human && "clientNonce" in incoming ? incoming.clientNonce : incoming.clientNonce ?? current.clientNonce;
  const delivery = options.preserveCurrentId ? current.delivery : incoming.delivery ?? current.delivery;
  const merged: TranscriptMessage = {
    ...current,
    ...incoming,
    id: options.preserveCurrentId ? current.id : incoming.id,
    text: human || incoming.text.length > 0 ? incoming.text : current.text,
    ...(attachments === undefined ? {} : { attachments }),
    ...(clientNonce === undefined && !(human && "clientNonce" in incoming) ? {} : { clientNonce }),
    ...(delivery === undefined && incoming.delivery === undefined ? {} : { delivery }),
  };
  if (options.preserveCurrentId && !("delivery" in current)) delete merged.delivery;
  return merged;
}

export function transcriptEntriesCorrelate(left: ConversationTranscriptEntry, right: ConversationTranscriptEntry): boolean {
  if (left.id === right.id) return true;
  if (left.kind !== "message" || right.kind !== "message"
    || left.role !== "user" || right.role !== "user"
    || left.clientNonce == null || right.clientNonce == null
    || left.clientNonce !== right.clientNonce) return false;
  if (left.clientNonce.length === 0) return false;
  if (left.authorKind !== "human" && right.authorKind !== "human") {
    return left.authorId == null && right.authorId == null;
  }
  if (left.authorKind === "human" && right.authorKind === "human") {
    return typeof left.authorId === "string" && left.authorId.trim().length > 0
      && left.authorId === right.authorId;
  }
  // Existing local optimistic placeholders precede the Host's author binding.
  // Only their exact local ID may bridge to an admitted Host Human send; an
  // arbitrary Agent or remote message with the same nonce cannot do so.
  const human = left.authorKind === "human" ? left : right;
  const optimistic = left.authorKind === "human" ? right : left;
  return optimistic.authorKind === undefined && optimistic.authorId == null && optimistic.role === "user"
    && optimistic.id === `pending-${optimistic.clientNonce}`
    && typeof human.authorId === "string" && human.authorId.trim().length > 0
    && (isLocalSendPlaceholder(human) || isCanonicalHumanSend(human));
}

function isLocalSendPlaceholder(entry: ConversationTranscriptEntry): entry is TranscriptMessage {
  if (entry.kind !== "message" || entry.role !== "user" || entry.clientNonce == null || entry.clientNonce.length === 0) return false;
  if (entry.authorKind !== "human") return entry.id === `pending-${entry.clientNonce}`;
  return typeof entry.authorId === "string" && entry.authorId.trim().length > 0
    && entry.remoteMessageId === undefined
    && (entry.id === `human-message:${entry.clientNonce}` || entry.id === `pending-${entry.clientNonce}`)
    && (entry.delivery === "pending" || entry.delivery === "queued" || entry.delivery === "failed");
}

function isCanonicalHumanSend(entry: ConversationTranscriptEntry): entry is TranscriptMessage {
  return entry.kind === "message" && entry.role === "user" && entry.authorKind === "human"
    && typeof entry.authorId === "string" && entry.authorId.trim().length > 0
    && typeof entry.clientNonce === "string" && entry.clientNonce.length > 0
    && typeof entry.remoteMessageId === "string" && entry.remoteMessageId.length > 0
    && entry.id === `human-server-message:${entry.remoteMessageId}`;
}

/** Undefined author selects the existing Agent path; null is an unscoped Human. */
export function localTranscriptClientNonce(
  entry: ConversationTranscriptEntry,
  humanAuthorId: string | null | undefined,
): string | null {
  if (entry.kind !== "message" || entry.role !== "user" || !entry.clientNonce) return null;
  if (humanAuthorId !== undefined) {
    return humanAuthorId != null && humanAuthorId.trim().length > 0
      && entry.authorKind === "human" && entry.authorId === humanAuthorId ? entry.clientNonce : null;
  }
  return entry.authorKind == null && entry.authorId == null ? entry.clientNonce : null;
}

// A false settlement state still permits a message; this is not a kind guard.
export function isSettledTranscriptMessage(entry: ConversationTranscriptEntry): boolean {
  return entry.kind === "message" && entry.role === "user" && entry.delivery === "sent"
    && !entry.id.startsWith("pending-")
    && !("sourceKind" in entry && entry.sourceKind === "user-attachment")
    && (entry.authorKind === "human" ? isCanonicalHumanSend(entry) : entry.authorId == null);
}

/** Older pages share the root's ID merge and Human settlement identity rules. */
export function mergeOlderTranscriptEntries(
  currentEntries: readonly ConversationTranscriptEntry[],
  olderEntries: readonly ConversationTranscriptEntry[],
): ConversationTranscriptEntry[] {
  const unique = (entries: readonly ConversationTranscriptEntry[]) => {
    const seen = new Set<string>();
    return entries.filter((entry) => {
      if (seen.has(entry.id)) return false;
      seen.add(entry.id);
      return true;
    });
  };
  const current = unique(currentEntries);
  const positions = new Map(current.map((entry, index) => [entry.id, index]));
  const insertions = new Map<number, ConversationTranscriptEntry[]>();
  let pending: ConversationTranscriptEntry[] = [];
  let lastAnchor = -1;
  const insertAt = (index: number) => {
    if (pending.length > 0) insertions.set(index, [...(insertions.get(index) ?? []), ...pending]);
    pending = [];
  };
  for (const older of unique(olderEntries)) {
    const position = positions.get(older.id);
    if (position == null) pending.push(older);
    else {
      insertAt(position);
      lastAnchor = position;
    }
  }
  // Overlapping pages supply chronological anchors; prepending unseen IDs
  // would turn [1,2,3,4] plus cached [2,4,5] into [1,3,2,4,5].
  insertAt(lastAnchor >= 0 ? lastAnchor + 1 : 0);
  const ordered: ConversationTranscriptEntry[] = [];
  for (let index = 0; index <= current.length; index += 1) {
    ordered.push(...(insertions.get(index) ?? []));
    if (index < current.length) ordered.push(current[index]);
  }
  const merged = unique(ordered);
  const settled = new Map<string, TranscriptMessage[]>();
  for (const entry of merged) {
    if (!isCanonicalHumanSend(entry)) continue;
    const candidates = settled.get(entry.clientNonce!) ?? [];
    candidates.push(entry);
    settled.set(entry.clientNonce!, candidates);
  }
  return merged.filter((entry) => !isLocalSendPlaceholder(entry) || entry.authorKind !== "human"
    || !(settled.get(entry.clientNonce!) ?? []).some((canonical) => transcriptEntriesCorrelate(entry, canonical)));
}

function previousIdentityMatches(current: ConversationTranscriptEntry, incoming: ConversationTranscriptEntry): boolean {
  if ((current.kind === "message" && current.authorKind === "human")
    || (incoming.kind === "message" && incoming.authorKind === "human")) {
    return current.kind === "message" && incoming.kind === "message"
      && current.role === "user" && incoming.role === "user"
      && current.authorKind === "human" && incoming.authorKind === "human"
      && incoming.authorId != null && incoming.authorId.trim().length > 0 && current.authorId === incoming.authorId;
  }
  return true;
}

/** Preserve optional fields from earlier events while a snapshot is pending. */
export function recordTranscriptReadUpdate(
  updates: Map<string, ConversationTranscriptEntry>,
  incoming: ConversationTranscriptEntry,
  previousId?: string,
): void {
  const matches = (entry: ConversationTranscriptEntry) => transcriptEntriesCorrelate(entry, incoming)
    || (entry.id === previousId && previousIdentityMatches(entry, incoming));
  const current = updates.get(incoming.id) ?? (previousId == null ? undefined : updates.get(previousId))
    ?? [...updates.values()].find(matches);
  const merged = current != null && matches(current) && current.kind === "message" && incoming.kind === "message"
    ? mergeAuthoritativeTranscriptMessage(current, incoming)
    : incoming;
  // Keep existing aliases on the latest complete projection, including a
  // clear followed by a partial event that omits the now-empty resources.
  for (const [id, entry] of updates) {
    if (matches(entry)) updates.set(id, merged);
  }
  updates.set(incoming.id, merged);
  if (previousId != null) {
    const previous = updates.get(previousId);
    if (previous == null || matches(previous)) updates.set(previousId, merged);
  }
}

/** Apply one Host mutation or command receipt without keeping its old placeholder. */
export function reconcileTranscriptMutation(
  current: readonly ConversationTranscriptEntry[],
  incoming: ConversationTranscriptEntry,
  options: { previousId?: string; appendIfMissing?: boolean; preserveSettled?: boolean; preserveCurrentId?: boolean } = {},
): ConversationTranscriptEntry[] {
  const matches = (entry: ConversationTranscriptEntry) => transcriptEntriesCorrelate(entry, incoming)
    || (entry.id === options.previousId && previousIdentityMatches(entry, incoming));
  const previous = current.find((entry) => entry.id === incoming.id) ?? current.find(matches);
  if (previous == null) return options.appendIfMissing ? [...current, incoming] : current as ConversationTranscriptEntry[];
  // A send receipt can lag behind the feed/history row that settled the send.
  // Keep that canonical row, including empty content and its latest reactions.
  const settled = options.preserveSettled && current.find((entry) => matches(entry)
    && entry.kind === "message" && entry.delivery === "sent" && !entry.id.startsWith("pending-"));
  const next = settled || (previous.kind === "message" && incoming.kind === "message"
    ? mergeAuthoritativeTranscriptMessage(previous, incoming, options)
    : incoming);
  let emitted = false;
  return current.flatMap((entry) => {
    if (!matches(entry)) return [entry];
    if (emitted) return [];
    emitted = true;
    return [next];
  });
}

/** Apply updates captured during this read only to rows in its returned window. */
export function applyTranscriptReadUpdates(
  entries: readonly ConversationTranscriptEntry[],
  updates: ReadonlyMap<string, ConversationTranscriptEntry>,
): ConversationTranscriptEntry[] {
  const projected = entries.map((entry) => {
    let current = entry;
    const visited = new Set<string>();
    while (!visited.has(current.id)) {
      visited.add(current.id);
      const update = updates.get(current.id)
        ?? [...updates.values()].find((candidate) => transcriptEntriesCorrelate(current, candidate));
      if (update == null || (current.id !== update.id
        && !transcriptEntriesCorrelate(current, update) && !previousIdentityMatches(current, update))) break;
      const previousId = current.id;
      current = current.kind === "message" && update.kind === "message"
        ? mergeAuthoritativeTranscriptMessage(current, update)
        : update;
      if (current.id === previousId) break;
    }
    return current;
  });
  const seen = new Set<string>();
  return projected.filter((entry) => {
    if (seen.has(entry.id)) return false;
    seen.add(entry.id);
    return true;
  });
}

function entryIndex(entries: readonly ConversationTranscriptEntry[]) {
  const byId = new Map(entries.map((entry) => [entry.id, entry]));
  const byNonce = new Map<string, ConversationTranscriptEntry[]>();
  for (const entry of entries) {
    if (entry.kind !== "message" || entry.clientNonce == null) continue;
    const candidates = byNonce.get(entry.clientNonce) ?? [];
    candidates.push(entry);
    byNonce.set(entry.clientNonce, candidates);
  }
  return (entry: ConversationTranscriptEntry): ConversationTranscriptEntry | undefined => {
    const direct = byId.get(entry.id);
    if (direct != null || entry.kind !== "message" || entry.clientNonce == null) return direct;
    const matches = (byNonce.get(entry.clientNonce) ?? []).filter((candidate) => transcriptEntriesCorrelate(candidate, entry));
    return matches.find(isCanonicalHumanSend) ?? matches[0];
  };
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
    const previous = currentEntry(entry);
    const next = previous?.kind === "message" && entry.kind === "message"
      ? mergeAuthoritativeTranscriptMessage(previous, entry)
      : entry;
    seen.add(entry.id);
    if (next.id !== entry.id && seen.has(next.id)) continue;
    seen.add(next.id);
    baseline.push(next);
  }
  const baselineEntry = entryIndex(baseline);
  for (const entry of current) {
    if (entry.kind === "message" && entry.role === "user" && entry.clientNonce != null
      && isLocalSendPlaceholder(entry) && baselineEntry(entry) == null && !seen.has(entry.id)) {
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
    if (isLocalSendPlaceholder(live) && !isLocalSendPlaceholder(entry)) return entry;
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
      || isLocalSendPlaceholder(entry);
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

export interface HumanReactionTarget {
  readonly conversationId: string;
  readonly entryId: string;
}

interface HumanReactionCompletion {
  readonly accountGeneration: number;
  invalidated: boolean;
}

export interface HumanReactionSettlement {
  run<T>(
    target: HumanReactionTarget,
    request: () => Promise<T>,
    apply: (value: T, isCurrent: () => boolean) => void,
    onError: (error: unknown, isCurrent: () => boolean) => void,
  ): Promise<void>;
  observeEntry(target: HumanReactionTarget): void;
  clearConversation(conversationId: string): void;
  dispose(): void;
}

/** Projection lifetime only; Session still owns reaction intent and settlement. */
export function createHumanReactionSettlement(readAccountGeneration: () => number): HumanReactionSettlement {
  let accountGeneration = readAccountGeneration();
  let disposed = false;
  // Retain settled tokens so deferred React updaters can reject later events.
  const completions = new Map<string, Map<string, HumanReactionCompletion>>();

  const enterAccount = (): number => {
    const current = readAccountGeneration();
    if (current !== accountGeneration) {
      for (const entries of completions.values()) {
        for (const completion of entries.values()) completion.invalidated = true;
      }
      completions.clear();
      accountGeneration = current;
    }
    return current;
  };

  return {
    async run(target, request, apply, onError) {
      if (disposed) return;
      const generation = enterAccount();
      const entries = completions.get(target.conversationId) ?? new Map<string, HumanReactionCompletion>();
      const previous = entries.get(target.entryId);
      if (previous != null) previous.invalidated = true;
      const completion: HumanReactionCompletion = { accountGeneration: generation, invalidated: false };
      entries.set(target.entryId, completion);
      completions.set(target.conversationId, entries);
      const isCurrent = (): boolean => !disposed
        && !completion.invalidated
        && readAccountGeneration() === completion.accountGeneration;
      try {
        const value = await request();
        if (isCurrent()) apply(value, isCurrent);
      } catch (error) {
        if (isCurrent()) onError(error, isCurrent);
      }
    },

    observeEntry(target) {
      if (disposed) return;
      enterAccount();
      const completion = completions.get(target.conversationId)?.get(target.entryId);
      if (completion != null) completion.invalidated = true;
    },

    clearConversation(conversationId) {
      if (disposed) return;
      enterAccount();
      const entries = completions.get(conversationId);
      if (entries != null) {
        for (const completion of entries.values()) completion.invalidated = true;
        completions.delete(conversationId);
      }
    },

    dispose() {
      disposed = true;
      completions.clear();
    },
  };
}
