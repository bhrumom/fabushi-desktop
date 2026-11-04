import type { ComposerSubmission } from "./submission";
import type { ComposerDraft, ConversationTranscriptEntry, DraftAttachment, TranscriptMessage, TranscriptReplyPreview } from "./model";
import { inferAttachmentKind } from "./model";

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5323918
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5306234

export interface ReplyThreadScope {
  accountSlot: string | null;
  agentId: string | null;
}

export interface ReplyTargetResolution {
  targetId: string;
  preview: TranscriptReplyPreview;
  isInScope: boolean;
  status: "resolved" | "missing";
}

export interface ReplySelection extends ReplyTargetResolution {
  scope: ReplyThreadScope;
}

export interface ReplyThreadControllerOptions {
  scope?: ReplyThreadScope;
  onSelectionChange?(selection: ReplySelection | null): void;
  onNavigate?(targetId: string, isInScope: boolean): void;
  onRestoreFocus?(): void;
}

export interface ReplyThreadController {
  getScope(): ReplyThreadScope;
  getSelection(): ReplySelection | null;
  replaceEntries(entries: readonly ConversationTranscriptEntry[]): void;
  prependEntries(entries: readonly ConversationTranscriptEntry[]): void;
  appendEntries(entries: readonly ConversationTranscriptEntry[]): void;
  resolve(targetId: string): ReplyTargetResolution;
  selectReply(targetId: string): boolean;
  restoreReplyFromDraft(draft: ComposerDraft): boolean;
  clearReply(): void;
  navigate(targetId: string): ReplyTargetResolution;
  applyReplyToDraft(draft: ComposerDraft): ComposerDraft;
  clearReplyFromDraft(draft: ComposerDraft): ComposerDraft;
  projectSubmission(submission: ComposerSubmission): ComposerSubmission;
  setScope(scope: ReplyThreadScope): void;
  dispose(): void;
}

function sameScope(left: ReplyThreadScope, right: ReplyThreadScope): boolean {
  return left.accountSlot === right.accountSlot && left.agentId === right.agentId;
}

function cloneScope(scope: ReplyThreadScope): ReplyThreadScope {
  return { accountSlot: scope.accountSlot, agentId: scope.agentId };
}

function attachmentPreview(attachment: DraftAttachment): TranscriptReplyPreview {
  const kind = inferAttachmentKind({ mimeType: attachment.mimeType, fileName: attachment.name, urlOrPath: attachment.path });
  if (kind === "image") return { kind: "image", url: attachment.path };
  if (attachment.name.length === 0 && /^https?:\/\//iu.test(attachment.path)) return { kind: "link", url: attachment.path };
  return { kind: "file", url: attachment.path, ...(attachment.name.length === 0 ? {} : { name: attachment.name }) };
}

function messagePreview(entry: TranscriptMessage): TranscriptReplyPreview {
  if (entry.text.length > 0) return { kind: entry.role === "user" ? "user-text" : "assistant-text", text: entry.text };
  const attachment = entry.attachments?.[0];
  return attachment == null ? { kind: "missing" } : attachmentPreview(attachment);
}

function previewForEntry(entry: ConversationTranscriptEntry): TranscriptReplyPreview {
  return entry.kind === "message" ? messagePreview(entry) : { kind: "missing" };
}

const HUMAN_LOCAL_MESSAGE_PREFIX = "human-message:";
const HUMAN_SERVER_MESSAGE_PREFIX = "human-server-message:";

interface ReplyEntryIndex {
  entries: Map<string, ConversationTranscriptEntry>;
  aliases: Map<string, string>;
  ambiguousAliases: Set<string>;
}

function indexEntries(
  sourceEntries: readonly ConversationTranscriptEntry[],
  accountSlot: string | null,
): ReplyEntryIndex {
  const indexed = new Map<string, ConversationTranscriptEntry>();
  const candidates = new Map<string, Set<string>>();
  for (const entry of sourceEntries) {
    indexed.set(entry.id, entry);
    if (accountSlot == null || entry.kind !== "message" || entry.authorKind !== "human"
      || entry.authorId !== accountSlot || entry.delivery !== "sent"
      || typeof entry.clientNonce !== "string" || entry.clientNonce.length === 0
      || typeof entry.remoteMessageId !== "string" || entry.remoteMessageId.length === 0
      || entry.id !== `${HUMAN_SERVER_MESSAGE_PREFIX}${entry.remoteMessageId}`) continue;
    const alias = `${HUMAN_LOCAL_MESSAGE_PREFIX}${entry.clientNonce}`;
    const ids = candidates.get(alias) ?? new Set<string>();
    ids.add(entry.id);
    candidates.set(alias, ids);
  }
  const aliases = new Map<string, string>();
  const ambiguousAliases = new Set<string>();
  for (const [alias, ids] of candidates) {
    if (ids.size === 1 && !indexed.has(alias)) aliases.set(alias, [...ids][0]);
    else ambiguousAliases.add(alias);
  }
  return { entries: indexed, aliases, ambiguousAliases };
}

function isValidLocalHumanReplyIntent(
  entry: ConversationTranscriptEntry | undefined,
  nonce: string,
  accountSlot: string | null,
): entry is TranscriptMessage {
  if (entry == null || entry.kind !== "message" || accountSlot == null) return false;
  return entry.authorKind === "human"
    && entry.authorId === accountSlot
    && entry.clientNonce === nonce
    && entry.remoteMessageId == null
    && (entry.delivery === "pending" || entry.delivery === "sent");
}

export function createReplyThreadController(options: ReplyThreadControllerOptions = {}): ReplyThreadController {
  let scope = cloneScope(options.scope ?? { accountSlot: null, agentId: null });
  let entries = new Map<string, ConversationTranscriptEntry>();
  let aliases = new Map<string, string>();
  let ambiguousAliases = new Set<string>();
  let selection: ReplySelection | null = null;
  let disposed = false;

  const notify = () => options.onSelectionChange?.(selection == null ? null : { ...selection, scope: cloneScope(selection.scope) });
  const currentResolution = (requestedTargetId: string): ReplyTargetResolution => {
    if (requestedTargetId.startsWith(HUMAN_LOCAL_MESSAGE_PREFIX)) {
      const nonce = requestedTargetId.slice(HUMAN_LOCAL_MESSAGE_PREFIX.length);
      if (nonce.length === 0 || ambiguousAliases.has(requestedTargetId)) {
        return { targetId: requestedTargetId, preview: { kind: "missing" }, isInScope: false, status: "missing" };
      }
      const canonicalId = aliases.get(requestedTargetId);
      if (canonicalId != null) {
        const canonical = entries.get(canonicalId);
        return canonical == null
          ? { targetId: requestedTargetId, preview: { kind: "missing" }, isInScope: false, status: "missing" }
          : { targetId: canonicalId, preview: previewForEntry(canonical), isInScope: true, status: "resolved" };
      }
      const local = entries.get(requestedTargetId);
      return isValidLocalHumanReplyIntent(local, nonce, scope.accountSlot)
        ? { targetId: requestedTargetId, preview: previewForEntry(local), isInScope: true, status: "resolved" }
        : { targetId: requestedTargetId, preview: { kind: "missing" }, isInScope: false, status: "missing" };
    }
    const entry = entries.get(requestedTargetId);
    return entry == null
      ? { targetId: requestedTargetId, preview: { kind: "missing" }, isInScope: false, status: "missing" }
      : { targetId: requestedTargetId, preview: previewForEntry(entry), isInScope: true, status: "resolved" };
  };

  const replaceEntries = (nextEntries: readonly ConversationTranscriptEntry[]) => {
    if (disposed) return;
    const indexed = indexEntries(nextEntries, scope.accountSlot);
    entries = indexed.entries;
    aliases = indexed.aliases;
    ambiguousAliases = indexed.ambiguousAliases;
    if (selection != null) {
      const next = currentResolution(selection.targetId);
      selection = { ...next, scope: cloneScope(scope) };
      notify();
    }
  };
  const mergeEntries = (nextEntries: readonly ConversationTranscriptEntry[], prepend: boolean) => {
    if (disposed) return;
    const incoming = indexEntries(nextEntries, scope.accountSlot).entries;
    const merged = new Map<string, ConversationTranscriptEntry>();
    if (prepend) for (const [id, entry] of incoming) merged.set(id, entry);
    for (const [id, entry] of entries) merged.set(id, entry);
    if (!prepend) for (const [id, entry] of incoming) merged.set(id, entry);
    const indexed = indexEntries([...merged.values()], scope.accountSlot);
    entries = indexed.entries;
    aliases = indexed.aliases;
    ambiguousAliases = indexed.ambiguousAliases;
    if (selection != null) {
      const next = currentResolution(selection.targetId);
      selection = { ...next, scope: cloneScope(scope) };
      notify();
    }
  };

  return {
    getScope: () => cloneScope(scope),
    getSelection: () => selection == null ? null : { ...selection, scope: cloneScope(selection.scope) },
    replaceEntries,
    prependEntries: (nextEntries) => mergeEntries(nextEntries, true),
    appendEntries: (nextEntries) => mergeEntries(nextEntries, false),
    resolve(targetId) {
      if (disposed || targetId.length === 0) return { targetId, preview: { kind: "missing" }, isInScope: false, status: "missing" };
      return currentResolution(targetId);
    },
    selectReply(targetId) {
      if (disposed || scope.accountSlot == null || scope.agentId == null || targetId.length === 0) return false;
      const resolution = currentResolution(targetId);
      selection = { ...resolution, scope: cloneScope(scope) };
      notify();
      return true;
    },
    restoreReplyFromDraft(draft) {
      if (disposed || selection != null || scope.accountSlot == null || scope.agentId == null) return false;
      const targetId = draft.replyToId;
      if (targetId == null || targetId.length === 0) return false;
      selection = { ...currentResolution(targetId), scope: cloneScope(scope) };
      notify();
      return true;
    },
    clearReply() {
      if (disposed) return;
      if (selection == null) return;
      selection = null;
      notify();
      options.onRestoreFocus?.();
    },
    navigate(targetId) {
      const resolution = disposed || targetId.length === 0
        ? { targetId, preview: { kind: "missing" } as const, isInScope: false, status: "missing" as const }
        : currentResolution(targetId);
      if (!disposed) {
        options.onNavigate?.(resolution.targetId, resolution.isInScope);
        options.onRestoreFocus?.();
      }
      return resolution;
    },
    applyReplyToDraft(draft) {
      if (selection == null || !sameScope(selection.scope, scope)) return { ...draft };
      return { ...draft, replyToId: selection.targetId };
    },
    clearReplyFromDraft(draft) {
      const { replyToId: _replyToId, ...withoutReply } = draft;
      return withoutReply;
    },
    projectSubmission(submission) {
      const { replyToId: _replyToId, ...withoutReply } = submission;
      return selection != null && sameScope(selection.scope, scope) && submission.agentId === scope.agentId
        ? { ...withoutReply, replyToId: selection.targetId }
        : withoutReply;
    },
    setScope(nextScope) {
      if (disposed || sameScope(scope, nextScope)) return;
      scope = cloneScope(nextScope);
      entries = new Map();
      aliases = new Map();
      ambiguousAliases = new Set();
      selection = null;
      notify();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      entries.clear();
      aliases.clear();
      ambiguousAliases.clear();
      selection = null;
    }
  };
}
