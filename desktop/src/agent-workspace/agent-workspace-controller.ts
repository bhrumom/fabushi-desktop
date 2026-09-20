import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { AgentOperationRegistry, type AgentOperationSnapshot } from '../grok-runtime/agent-operation-registry';
import { AGENT_ATTACHMENT_LIMIT } from './agent-attachments';
import type { PersistedAgentDraft, PersistedAgentDrafts } from './agent-draft-store';
import type { AgentPromptReference, AgentReplyContext } from './prompt-context';

export interface AgentWorkspaceDraftSnapshot {
  readonly [peerKey: string]: string;
}

export interface AgentWorkspaceAttachmentSnapshot {
  readonly [peerKey: string]: readonly AttachmentContext[];
}

export interface AgentWorkspaceReplySnapshot {
  readonly [peerKey: string]: AgentReplyContext;
}

export interface AgentWorkspaceReferenceSnapshot {
  readonly [peerKey: string]: readonly AgentPromptReference[];
}

function normalizeDraft(draft: Partial<PersistedAgentDraft> | undefined): PersistedAgentDraft {
  return {
    text: typeof draft?.text === 'string' ? draft.text : '',
    attachments: Array.isArray(draft?.attachments)
      ? [...draft.attachments].slice(0, AGENT_ATTACHMENT_LIMIT)
      : [],
    ...(Array.isArray(draft?.references) && draft.references.length
      ? { references: draft.references.filter((reference) => reference?.kind === 'agent').slice(0, 32) }
      : {}),
    ...(draft?.replyTo ? { replyTo: draft.replyTo } : {}),
  };
}

function draftHasPayload(draft: PersistedAgentDraft): boolean {
  return Boolean(draft.text || draft.attachments.length || draft.references?.length || draft.replyTo);
}

/**
 * Renderer-facing state owner for the Agent workspace.
 *
 * The view layer must not know how request ids become runtime operation ids,
 * and it must not keep Agent drafts/attachments/replies in renderer-global
 * React maps. This controller owns those Agent-scoped lifecycles so opening
 * Agent B cannot mutate Agent A while Agent A is still working.
 */
export class AgentWorkspaceController {
  private readonly operations = new AgentOperationRegistry();
  private readonly drafts = new Map<string, PersistedAgentDraft>();
  private readonly uploadingPeers = new Set<string>();
  private readonly finishedOperations = new Set<string>();

  constructor(initialDrafts: PersistedAgentDrafts = {}) {
    this.hydratePersistedDrafts(initialDrafts);
  }

  beginRequest(peerKey: string, requestId: string): void {
    this.operations.beginRequest(peerKey, requestId);
  }

  cancelRequest(requestId: string): string | null {
    return this.operations.cancelRequest(requestId);
  }

  adoptOperation(
    requestId: string | null | undefined,
    operationId: string,
    fallbackPeerKey?: string | null,
  ): string | null {
    return this.operations.adoptOperation(requestId, operationId, fallbackPeerKey);
  }

  claimOperation(operationId: string, peerKey: string): string {
    return this.operations.claimOperation(operationId, peerKey);
  }

  /**
   * Adopt a runtime operation without consulting the currently visible Agent.
   * Legacy events that omit request ownership may only fall back when exactly
   * one Agent request is pending.
   */
  claimRuntimeOperation(operationId: string, fallbackPeerKey?: string | null): string | null {
    if (!operationId || this.finishedOperations.has(operationId)) return null;
    const peerKey = this.operations.peerForOperation(operationId)
      ?? fallbackPeerKey
      ?? this.operations.onlyPendingPeer();
    if (!peerKey) return null;
    const requestId = this.operations.requestForPeer(peerKey);
    this.operations.adoptOperation(requestId, operationId, peerKey);
    return peerKey;
  }

  finishOperation(operationId: string): string | null {
    return this.operations.finishOperation(operationId);
  }

  finishRuntimeOperation(operationId: string): string | null {
    const peerKey = this.operations.peerForOperation(operationId);
    if (!peerKey) return null;
    this.operations.finishOperation(operationId);
    this.finishedOperations.add(operationId);
    if (this.finishedOperations.size > 500) {
      const oldest = this.finishedOperations.values().next().value;
      if (oldest) this.finishedOperations.delete(oldest);
    }
    return peerKey;
  }

  isOperationFinished(operationId: string | null | undefined): boolean {
    return Boolean(operationId && this.finishedOperations.has(operationId));
  }

  peerForRuntimeId(runtimeId: string | null | undefined): string | null {
    return this.operations.peerForOperation(runtimeId)
      ?? this.operations.peerForRequest(runtimeId);
  }

  operationForPeer(peerKey: string | null | undefined): string | null {
    return this.operations.operationForPeer(peerKey);
  }

  requestForPeer(peerKey: string | null | undefined): string | null {
    return this.operations.requestForPeer(peerKey);
  }

  peerForOperation(operationId: string | null | undefined): string | null {
    return this.operations.peerForOperation(operationId);
  }

  peerForRequest(requestId: string | null | undefined): string | null {
    return this.operations.peerForRequest(requestId);
  }

  isBusy(peerKey: string | null | undefined): boolean {
    return this.operations.isBusy(peerKey);
  }

  snapshot(): AgentOperationSnapshot {
    return this.operations.snapshot();
  }

  requestSnapshot(): Readonly<Record<string, string>> {
    return this.operations.requestSnapshot();
  }

  onlyPendingPeer(): string | null {
    return this.operations.onlyPendingPeer();
  }

  clearPeer(peerKey: string): void {
    this.operations.clearPeer(peerKey);
    this.drafts.delete(peerKey);
    this.uploadingPeers.delete(peerKey);
  }

  clearOperations(options: { preserveFinished?: boolean } = {}): void {
    this.operations.clear();
    this.uploadingPeers.clear();
    if (!options.preserveFinished) this.finishedOperations.clear();
  }

  clear(): void {
    this.clearOperations();
    this.drafts.clear();
  }

  setDraft(peerKey: string, value: string): void {
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const next = { ...current, text: value };
    if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    else this.drafts.delete(peerKey);
  }

  hydrateDrafts(snapshot: Readonly<Record<string, string>>): void {
    for (const [peerKey, value] of Object.entries(snapshot)) {
      if (!peerKey) continue;
      const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
      const next = { ...current, text: value };
      if (draftHasPayload(next)) this.drafts.set(peerKey, next);
      else this.drafts.delete(peerKey);
    }
  }

  hydratePersistedDrafts(snapshot: PersistedAgentDrafts): void {
    this.drafts.clear();
    for (const [peerKey, value] of Object.entries(snapshot)) {
      if (!peerKey) continue;
      const next = normalizeDraft(value);
      if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    }
  }

  draftForPeer(peerKey: string | null | undefined): string {
    return peerKey ? this.drafts.get(peerKey)?.text ?? '' : '';
  }

  attachmentsForPeer(peerKey: string | null | undefined): readonly AttachmentContext[] {
    if (!peerKey) return [];
    return this.drafts.get(peerKey)?.attachments ?? [];
  }

  setAttachments(peerKey: string, attachments: readonly AttachmentContext[]): void {
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const next = {
      ...current,
      attachments: [...attachments].slice(0, AGENT_ATTACHMENT_LIMIT),
    };
    if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    else this.drafts.delete(peerKey);
  }

  appendAttachments(peerKey: string, attachments: readonly AttachmentContext[]): void {
    if (!attachments.length) return;
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const byId = new Map(
      [...current.attachments, ...attachments].map((attachment) => [attachment.id, attachment] as const),
    );
    this.setAttachments(peerKey, [...byId.values()]);
  }

  removeAttachment(peerKey: string, attachmentId: string): void {
    this.setAttachments(
      peerKey,
      this.attachmentsForPeer(peerKey).filter((attachment) => attachment.id !== attachmentId),
    );
  }

  referencesForPeer(peerKey: string | null | undefined): readonly AgentPromptReference[] {
    if (!peerKey) return [];
    return this.drafts.get(peerKey)?.references ?? [];
  }

  upsertReference(peerKey: string, reference: AgentPromptReference): void {
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const references = [
      ...(current.references ?? []).filter((candidate) => candidate.id !== reference.id),
      reference,
    ].slice(-32);
    const next = { ...current, references };
    if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    else this.drafts.delete(peerKey);
  }

  pruneReferences(peerKey: string, text: string): void {
    const current = this.drafts.get(peerKey);
    if (!current?.references?.length) return;
    const references = current.references.filter((reference) => text.includes(`@${reference.label}`));
    const next = { ...current, ...(references.length ? { references } : {}) };
    if (!references.length) delete next.references;
    if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    else this.drafts.delete(peerKey);
  }

  replyForPeer(peerKey: string | null | undefined): AgentReplyContext | undefined {
    return peerKey ? this.drafts.get(peerKey)?.replyTo : undefined;
  }

  setReply(peerKey: string, replyTo: AgentReplyContext | undefined): void {
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const next: PersistedAgentDraft = {
      ...current,
      ...(replyTo ? { replyTo } : {}),
    };
    if (!replyTo) delete next.replyTo;
    if (draftHasPayload(next)) this.drafts.set(peerKey, next);
    else this.drafts.delete(peerKey);
  }

  clearReply(peerKey: string): void {
    this.setReply(peerKey, undefined);
  }

  setUploading(peerKey: string, uploading: boolean): void {
    if (uploading) this.uploadingPeers.add(peerKey);
    else this.uploadingPeers.delete(peerKey);
  }

  isUploading(peerKey: string | null | undefined): boolean {
    return Boolean(peerKey && this.uploadingPeers.has(peerKey));
  }

  takeDraft(peerKey: string): PersistedAgentDraft {
    const current = normalizeDraft(this.drafts.get(peerKey));
    this.drafts.delete(peerKey);
    return current;
  }

  restoreDraft(peerKey: string, draft: Partial<PersistedAgentDraft>): void {
    const current = this.drafts.get(peerKey) ?? normalizeDraft(undefined);
    const attachmentById = new Map(
      [...current.attachments, ...(draft.attachments ?? [])]
        .map((attachment) => [attachment.id, attachment] as const),
    );
    const merged = normalizeDraft({
      // A failed send may settle after the user has already started the next
      // prompt. Never overwrite that newer text; only restore the submitted
      // prompt when the current Agent draft is still empty.
      text: current.text || draft.text || '',
      attachments: [...attachmentById.values()],
      references: [
        ...(current.references ?? []),
        ...(draft.references ?? []).filter((reference) => !(current.references ?? []).some((candidate) => candidate.id === reference.id)),
      ],
      replyTo: current.replyTo ?? draft.replyTo,
    });
    if (draftHasPayload(merged)) this.drafts.set(peerKey, merged);
    else this.drafts.delete(peerKey);
  }

  persistedDraftSnapshot(): PersistedAgentDrafts {
    return Object.fromEntries(
      [...this.drafts.entries()].map(([peerKey, draft]) => [
        peerKey,
        {
          text: draft.text,
          attachments: [...draft.attachments],
          ...(draft.references?.length ? { references: draft.references.map((reference) => ({ ...reference })) } : {}),
          ...(draft.replyTo ? { replyTo: { ...draft.replyTo } } : {}),
        },
      ]),
    );
  }

  draftSnapshot(): AgentWorkspaceDraftSnapshot {
    return Object.freeze(Object.fromEntries(
      [...this.drafts.entries()]
        .filter(([, draft]) => Boolean(draft.text))
        .map(([peerKey, draft]) => [peerKey, draft.text]),
    ));
  }

  attachmentSnapshot(): AgentWorkspaceAttachmentSnapshot {
    return Object.freeze(Object.fromEntries(
      [...this.drafts.entries()]
        .filter(([, draft]) => draft.attachments.length > 0)
        .map(([peerKey, draft]) => [peerKey, Object.freeze([...draft.attachments])]),
    ));
  }

  replySnapshot(): AgentWorkspaceReplySnapshot {
    return Object.freeze(Object.fromEntries(
      [...this.drafts.entries()]
        .filter(([, draft]) => Boolean(draft.replyTo))
        .map(([peerKey, draft]) => [peerKey, { ...draft.replyTo! }]),
    ));
  }

  referenceSnapshot(): AgentWorkspaceReferenceSnapshot {
    return Object.freeze(Object.fromEntries(
      [...this.drafts.entries()]
        .filter(([, draft]) => Boolean(draft.references?.length))
        .map(([peerKey, draft]) => [
          peerKey,
          Object.freeze((draft.references ?? []).map((reference) => Object.freeze({ ...reference }))),
        ]),
    ));
  }
}
