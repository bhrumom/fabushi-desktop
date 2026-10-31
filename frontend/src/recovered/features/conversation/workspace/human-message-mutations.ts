import type { ComposerDraft, ConversationTranscriptEntry, TranscriptMessage } from "./model";

export type HumanMessageMutationKind = "edit" | "delete";

export interface HumanMessageMutationTarget {
  readonly conversationId: string;
  readonly entryId: string;
  readonly ownerId: string;
  readonly remoteMessageId: string;
  readonly baseRevision: number;
  readonly operationId: string;
}

export interface HumanMessageEditState extends HumanMessageMutationTarget {
  readonly originalText: string;
  readonly draft: ComposerDraft;
  readonly phase: "editing" | "saving";
}

export interface HumanMessageDeleteTarget extends HumanMessageMutationTarget {
  readonly preview: string;
}

function isSettledOwnerMessage(entry: TranscriptMessage, ownerId: string | null | undefined): boolean {
  return ownerId != null
    && ownerId.length > 0
    && entry.authorId === ownerId
    && entry.remoteMessageId != null
    && entry.remoteMessageId.length > 0
    && entry.remoteRevision != null
    && Number.isInteger(entry.remoteRevision)
    && entry.remoteRevision >= 1
    && entry.delivery === "sent"
    && entry.isDeleted !== true;
}

export function canEditHumanMessage(entry: TranscriptMessage, ownerId: string | null | undefined): boolean {
  return isSettledOwnerMessage(entry, ownerId) && entry.text.trim().length > 0;
}

export function canDeleteHumanMessage(entry: TranscriptMessage, ownerId: string | null | undefined): boolean {
  return isSettledOwnerMessage(entry, ownerId);
}

export function beginHumanMessageEdit(
  conversationId: string,
  ownerId: string | null | undefined,
  entry: TranscriptMessage,
  operationId: string,
): HumanMessageEditState | null {
  const normalizedOperationId = operationId.trim();
  if (!canEditHumanMessage(entry, ownerId) || ownerId == null || entry.remoteMessageId == null || entry.remoteRevision == null || normalizedOperationId.length === 0) return null;
  return {
    conversationId,
    entryId: entry.id,
    ownerId,
    remoteMessageId: entry.remoteMessageId,
    baseRevision: entry.remoteRevision,
    operationId: normalizedOperationId,
    originalText: entry.text,
    draft: { prompt: entry.text, attachments: [] },
    phase: "editing",
  };
}

export function updateHumanMessageEditDraft(
  state: HumanMessageEditState,
  draft: ComposerDraft,
): HumanMessageEditState {
  if (state.phase === "saving") return state;
  return {
    ...state,
    draft: {
      prompt: draft.prompt,
      ...(draft.richText == null ? {} : { richText: draft.richText }),
      attachments: [],
    },
  };
}

export function markHumanMessageEditSaving(state: HumanMessageEditState): HumanMessageEditState {
  return state.phase === "saving" ? state : { ...state, phase: "saving" };
}

export function markHumanMessageEditEditing(state: HumanMessageEditState): HumanMessageEditState {
  return state.phase === "editing" ? state : { ...state, phase: "editing" };
}

export function beginHumanMessageDelete(
  conversationId: string,
  ownerId: string | null | undefined,
  entry: TranscriptMessage,
  operationId: string,
): HumanMessageDeleteTarget | null {
  const normalizedOperationId = operationId.trim();
  if (!canDeleteHumanMessage(entry, ownerId) || ownerId == null || entry.remoteMessageId == null || entry.remoteRevision == null || normalizedOperationId.length === 0) return null;
  const normalized = entry.text.replace(/\s+/gu, " ").trim();
  return {
    conversationId,
    entryId: entry.id,
    ownerId,
    remoteMessageId: entry.remoteMessageId,
    baseRevision: entry.remoteRevision,
    operationId: normalizedOperationId,
    preview: normalized.length <= 80 ? normalized : `${normalized.slice(0, 79).trimEnd()}…`,
  };
}

function sameRemoteMessage(
  target: HumanMessageMutationTarget,
  entry: TranscriptMessage,
): boolean {
  return entry.id === target.entryId
    && entry.authorId === target.ownerId
    && entry.remoteMessageId === target.remoteMessageId;
}

export function isAuthoritativeHumanMessageMutationSettlement(
  target: HumanMessageMutationTarget,
  entry: TranscriptMessage,
  kind: HumanMessageMutationKind,
): boolean {
  if (!sameRemoteMessage(target, entry)) return false;
  if (entry.remoteRevision == null || !Number.isInteger(entry.remoteRevision) || entry.remoteRevision <= target.baseRevision) return false;
  if (kind === "delete") return entry.isDeleted === true && entry.deletedAt != null;
  return entry.isDeleted !== true && entry.editedAt != null;
}

export function settleHumanMessageMutation(
  entries: ConversationTranscriptEntry[],
  authoritative: TranscriptMessage,
): ConversationTranscriptEntry[] {
  const index = entries.findIndex((entry) => entry.kind === "message"
    && (entry.id === authoritative.id
      || (authoritative.remoteMessageId != null && entry.remoteMessageId === authoritative.remoteMessageId)));
  if (index < 0) return entries;
  const current = entries[index];
  if (current == null || current.kind !== "message") return entries;
  if (current.remoteRevision != null && authoritative.remoteRevision != null
    && authoritative.remoteRevision < current.remoteRevision) return entries;
  const next = entries.slice();
  next[index] = authoritative;
  return next;
}
