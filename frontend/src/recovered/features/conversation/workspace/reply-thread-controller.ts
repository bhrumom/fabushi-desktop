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
  clearReply(): void;
  navigate(targetId: string): ReplyTargetResolution;
  applyReplyToDraft(draft: ComposerDraft, draftScope?: ReplyThreadScope): ComposerDraft;
  clearReplyFromDraft(draft: ComposerDraft): ComposerDraft;
  projectSubmission(submission: ComposerSubmission, draftScope?: ReplyThreadScope): ComposerSubmission;
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

export function isActionableReplyTargetEntry(entry: ConversationTranscriptEntry): boolean {
  return entry.kind === "message" && (entry.text.trim().length > 0 || (entry.attachments?.length ?? 0) > 0);
}

function previewForEntry(entry: ConversationTranscriptEntry): TranscriptReplyPreview {
  return entry.kind === "message" ? messagePreview(entry) : { kind: "missing" };
}

function indexEntries(entries: readonly ConversationTranscriptEntry[]): Map<string, ConversationTranscriptEntry> {
  const indexed = new Map<string, ConversationTranscriptEntry>();
  for (const entry of entries) indexed.set(entry.id, entry);
  return indexed;
}

export function createReplyThreadController(options: ReplyThreadControllerOptions = {}): ReplyThreadController {
  let scope = cloneScope(options.scope ?? { accountSlot: null, agentId: null });
  let entries = new Map<string, ConversationTranscriptEntry>();
  let selection: ReplySelection | null = null;
  let disposed = false;

  const notify = () => options.onSelectionChange?.(selection == null ? null : { ...selection, scope: cloneScope(selection.scope) });
  const isTargetClearedInEntries = (targetId: string): boolean => {
    const entry = entries.get(targetId);
    return entry != null && !isActionableReplyTargetEntry(entry);
  };
  const currentResolution = (targetId: string): ReplyTargetResolution => {
    const entry = entries.get(targetId);
    return entry == null || !isActionableReplyTargetEntry(entry)
      ? { targetId, preview: { kind: "missing" }, isInScope: false, status: "missing" }
      : { targetId, preview: previewForEntry(entry), isInScope: true, status: "resolved" };
  };

  const replaceEntries = (nextEntries: readonly ConversationTranscriptEntry[]) => {
    if (disposed) return;
    entries = indexEntries(nextEntries);
    if (selection != null) {
      if (isTargetClearedInEntries(selection.targetId)) {
        selection = null;
        notify();
      } else {
        const next = currentResolution(selection.targetId);
        selection = { ...next, scope: cloneScope(scope) };
        notify();
      }
    }
  };
  const mergeEntries = (nextEntries: readonly ConversationTranscriptEntry[], prepend: boolean) => {
    if (disposed) return;
    const incoming = indexEntries(nextEntries);
    const merged = new Map<string, ConversationTranscriptEntry>();
    if (prepend) for (const [id, entry] of incoming) merged.set(id, entry);
    for (const [id, entry] of entries) merged.set(id, entry);
    if (!prepend) for (const [id, entry] of incoming) merged.set(id, entry);
    entries = merged;
    if (selection != null) {
      if (isTargetClearedInEntries(selection.targetId)) {
        selection = null;
        notify();
      } else {
        const next = currentResolution(selection.targetId);
        selection = { ...next, scope: cloneScope(scope) };
        notify();
      }
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
      if (
        disposed
        || scope.accountSlot == null
        || scope.agentId == null
        || targetId.length === 0
        || isTargetClearedInEntries(targetId)
      ) return false;
      const resolution = currentResolution(targetId);
      selection = { ...resolution, scope: cloneScope(scope) };
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
        options.onNavigate?.(targetId, resolution.isInScope);
        options.onRestoreFocus?.();
      }
      return resolution;
    },
    applyReplyToDraft(draft, draftScope) {
      // A new render can precede setScope. Never overlay the retired account's
      // in-memory selection on an explicitly scoped persisted draft.
      if (disposed || draftScope != null && !sameScope(draftScope, scope)) return { ...draft };
      if (selection != null && sameScope(selection.scope, scope)) {
        return { ...draft, replyToId: selection.targetId };
      }
      if (typeof draft.replyToId === "string" && isTargetClearedInEntries(draft.replyToId)) {
        const { replyToId: _replyToId, ...withoutReply } = draft;
        return withoutReply;
      }
      return { ...draft };
    },
    clearReplyFromDraft(draft) {
      const { replyToId: _replyToId, ...withoutReply } = draft;
      return withoutReply;
    },
    projectSubmission(submission, draftScope) {
      const { replyToId, ...withoutReply } = submission;
      if (disposed || draftScope != null && (draftScope.accountSlot == null || draftScope.agentId == null
        || !sameScope(draftScope, scope) || submission.agentId !== draftScope.agentId)) return withoutReply;
      if (selection != null && sameScope(selection.scope, scope) && submission.agentId === scope.agentId) {
        return { ...withoutReply, replyToId: selection.targetId };
      }
      // Legacy callers still require a current selection. Only the mounted
      // owner may provide the exact account/conversation of its restored draft.
      return draftScope != null && typeof replyToId === "string" && replyToId.trim().length > 0
        && !isTargetClearedInEntries(replyToId)
        ? { ...withoutReply, replyToId } : withoutReply;
    },
    setScope(nextScope) {
      if (disposed || sameScope(scope, nextScope)) return;
      scope = cloneScope(nextScope);
      entries = new Map();
      selection = null;
      notify();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      entries.clear();
      selection = null;
    }
  };
}
