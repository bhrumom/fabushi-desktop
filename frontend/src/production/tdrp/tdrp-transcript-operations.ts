import { useCallback } from "react";
import {
  isSettledTranscriptMessage,
  localTranscriptClientNonce,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import {
  transcriptAttachmentIdentity,
  type ConversationTranscriptEntry,
} from "../../recovered/features/conversation/workspace/model";
import type { ComposerSubmission } from "../../recovered/features/conversation/workspace/submission";
import type {
  ScopedComposerSubmission,
  TranscriptOperationScope,
  TranscriptSpreadsheetSource,
  TranscriptSubmissionScope,
} from "./tdrp-transcript-scope";
import type { RendererAgent } from "../model";

export interface UseTdrpTranscriptOperationsOptions {
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly accountIdentityRef: React.MutableRefObject<string | null>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly transcriptConversationGenerationsRef: React.MutableRefObject<Map<string, number>>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly spreadsheetViewerSourceRef: React.MutableRefObject<TranscriptSpreadsheetSource | null>;
  readonly spreadsheetEntriesRef: React.MutableRefObject<readonly ConversationTranscriptEntry[]>;
  readonly selectionStore: any;
  readonly humanFindGenerationRef: React.MutableRefObject<number>;
  readonly humanFindUpdateGenerationRef: React.MutableRefObject<number>;
  readonly setHumanFindUpdateGeneration: React.Dispatch<React.SetStateAction<number>>;
  readonly composerDraftStore: any;
  readonly activeAgentIdRef: React.MutableRefObject<string>;
  readonly setComposerClearGeneration: React.Dispatch<React.SetStateAction<number>>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly submissionScopesRef: React.MutableRefObject<Map<string, TranscriptSubmissionScope>>;
  readonly acknowledgementController: any;
  readonly humanCoverageContinuationInFlightRef: React.MutableRefObject<Set<string>>;
  readonly composerSubmissionQueueRef: React.MutableRefObject<any>;
  readonly pendingTranscriptReceiptsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly setPendingSendScopes: React.Dispatch<React.SetStateAction<ReadonlyMap<string, TranscriptSubmissionScope>>>;
}

export function useTdrpTranscriptOperations({
  accountRef,
  accountIdentityRef,
  accountScopeGenerationRef,
  transcriptConversationGenerationsRef,
  humanConversationsRef,
  spreadsheetViewerSourceRef,
  spreadsheetEntriesRef,
  selectionStore,
  humanFindGenerationRef,
  humanFindUpdateGenerationRef,
  setHumanFindUpdateGeneration,
  composerDraftStore,
  activeAgentIdRef,
  setComposerClearGeneration,
  setNotice,
  submissionScopesRef,
  acknowledgementController,
  humanCoverageContinuationInFlightRef,
  composerSubmissionQueueRef,
  pendingTranscriptReceiptsRef,
  setPendingSendScopes,
}: UseTdrpTranscriptOperationsOptions) {
  const isTranscriptScopeCurrent = useCallback((scope: TranscriptOperationScope) => {
    const current = accountRef.current;
    return current?.kind === "logged-in"
      && (current.authId ?? current.email ?? "account") === scope.accountSlot
      && accountIdentityRef.current === `logged-in:${scope.accountSlot}`
      && accountScopeGenerationRef.current === scope.accountGeneration
      && (transcriptConversationGenerationsRef.current.get(scope.agentId) ?? 0) === scope.conversationGeneration;
  }, [accountIdentityRef, accountRef, accountScopeGenerationRef, transcriptConversationGenerationsRef]);

  const captureTranscriptScope = useCallback((agentId: string): TranscriptOperationScope | null => {
    const current = accountRef.current;
    if (current?.kind !== "logged-in") return null;
    if (accountIdentityRef.current !== `logged-in:${current.authId ?? current.email ?? "account"}`) return null;
    return {
      accountSlot: current.authId ?? current.email ?? "account",
      accountGeneration: accountScopeGenerationRef.current,
      agentId,
      conversationGeneration: transcriptConversationGenerationsRef.current.get(agentId) ?? 0,
      humanAuthorId: humanConversationsRef.current.some((conversation) => conversation.id === agentId)
        ? current.authId || null : undefined,
    };
  }, [accountIdentityRef, accountRef, accountScopeGenerationRef, humanConversationsRef, transcriptConversationGenerationsRef]);

  const isSpreadsheetSourceCurrent = useCallback((source: TranscriptSpreadsheetSource) => {
    if (spreadsheetViewerSourceRef.current !== source || !isTranscriptScopeCurrent(source)
      || selectionStore.get().currentAgentId !== source.agentId) return false;
    const entry = spreadsheetEntriesRef.current.find((candidate) => candidate.id === source.entryId);
    return entry?.kind === "message" && entry.attachments?.some((attachment) =>
      transcriptAttachmentIdentity(attachment) === source.attachmentIdentity) === true;
  }, [isTranscriptScopeCurrent, selectionStore, spreadsheetEntriesRef, spreadsheetViewerSourceRef]);

  const invalidateActiveHumanSearch = useCallback((ownerId: string) => {
    if (selectionStore.get().currentAgentId !== ownerId
      || !humanConversationsRef.current.some((conversation) => conversation.id === ownerId)) return;
    humanFindGenerationRef.current += 1;
    humanFindUpdateGenerationRef.current += 1;
    setHumanFindUpdateGeneration(humanFindUpdateGenerationRef.current);
  }, [humanConversationsRef, humanFindGenerationRef, humanFindUpdateGenerationRef, selectionStore, setHumanFindUpdateGeneration]);

  const settlePersistedHumanDraft = useCallback((operation: TranscriptOperationScope, nonce: string) => {
    if (typeof operation.humanAuthorId !== "string" || !isTranscriptScopeCurrent(operation)) return;
    void composerDraftStore.settleHumanSubmission({
      accountSlot: operation.accountSlot, agentId: operation.agentId, authorId: operation.humanAuthorId, nonce,
    }).then(({ clearedDraft, clearedRecovery }: { clearedDraft: boolean; clearedRecovery: boolean }) => {
      if (!isTranscriptScopeCurrent(operation) || (!clearedDraft && !clearedRecovery)) return;
      const current = composerDraftStore.snapshotsFor(operation.agentId).get();
      if (current.draft != null || current.recovery != null || activeAgentIdRef.current !== operation.agentId) return;
      setComposerClearGeneration((generation) => {
        const latest = composerDraftStore.snapshotsFor(operation.agentId).get();
        return isTranscriptScopeCurrent(operation) && activeAgentIdRef.current === operation.agentId
          && latest.draft == null && latest.recovery == null ? generation + 1 : generation;
      });
    }).catch((error: unknown) => {
      if (isTranscriptScopeCurrent(operation)) setNotice(`Human draft recovery unavailable: ${error instanceof Error ? error.message : String(error)}`);
    });
  }, [activeAgentIdRef, composerDraftStore, isTranscriptScopeCurrent, setComposerClearGeneration, setNotice]);

  const settleTranscriptDraft = useCallback((scope: TranscriptSubmissionScope) => {
    if (scope.invalidated || !isTranscriptScopeCurrent(scope)) return;
    if (typeof scope.humanAuthorId === "string") {
      if (scope.canonicalSettled) settlePersistedHumanDraft(scope, scope.nonce);
      return;
    }
    const ownership = scope.draftOwnership;
    if (ownership == null || ownership.settled || scope.invalidated || !isTranscriptScopeCurrent(scope)) return;
    ownership.settled = true;
    const currentDraft = composerDraftStore.snapshotsFor(scope.agentId).get();
    const clearsRecovery = ownership.recovery != null && currentDraft.recovery === ownership.recovery;
    if (clearsRecovery) composerDraftStore.clearRecovery(scope.agentId);
    const cleared = ownership.draftIdentity != null
      ? composerDraftStore.clearDraftIfCurrent(ownership.draftIdentity)
      : ownership.draft != null && currentDraft.draft === ownership.draft
        && composerDraftStore.clearDraftIfMatches({ agentId: scope.agentId, draft: ownership.draft });
    if ((cleared || clearsRecovery && currentDraft.draft == null) && activeAgentIdRef.current === scope.agentId) {
      setComposerClearGeneration((current) => scope.invalidated || !isTranscriptScopeCurrent(scope) ? current : current + 1);
    }
  }, [activeAgentIdRef, composerDraftStore, isTranscriptScopeCurrent, setComposerClearGeneration, settlePersistedHumanDraft]);

  const scopeComposerSubmission = useCallback((submission: ComposerSubmission): ScopedComposerSubmission | null => {
    const operation = captureTranscriptScope(submission.agentId);
    if (operation == null || operation.humanAuthorId === null) return null;
    let cancel = () => {};
    const scope: TranscriptSubmissionScope = {
      ...operation,
      nonce: submission.nonce,
      cancelled: new Promise<void>((resolve) => { cancel = resolve; }),
      invalidated: false,
      canonicalSettled: false,
      humanQueuedOffline: false,
      invalidate() { scope.invalidated = true; cancel(); },
      onSettled() { settleTranscriptDraft(scope); },
    };
    submissionScopesRef.current.set(submission.nonce, scope);
    return { ...submission, transcriptScope: scope };
  }, [captureTranscriptScope, settleTranscriptDraft, submissionScopesRef]);

  const markTranscriptSubmissionSettled = useCallback((scope: TranscriptSubmissionScope) => {
    if (scope.invalidated || !isTranscriptScopeCurrent(scope)) return;
    scope.canonicalSettled = true;
    acknowledgementController.reconcileEcho({
      accountSlot: scope.accountSlot, agentId: scope.agentId, nonce: scope.nonce, echoedNonce: scope.nonce,
    });
    scope.onSettled?.();
  }, [acknowledgementController, isTranscriptScopeCurrent]);

  const observeTranscriptSettlement = useCallback((agentId: string, entry: ConversationTranscriptEntry) => {
    const current = accountRef.current;
    if (current?.kind !== "logged-in") return;
    const scope = entry.kind === "message" && entry.clientNonce != null
      ? submissionScopesRef.current.get(entry.clientNonce) : undefined;
    const human = humanConversationsRef.current.some((conversation) => conversation.id === agentId)
      || (scope?.agentId === agentId && scope.humanAuthorId !== undefined)
      || (entry.kind === "message" && entry.authorKind === "human");
    const nonce = localTranscriptClientNonce(entry, human ? current.authId || null : undefined);
    if (nonce == null) return;
    if (scope?.agentId === agentId && (scope.invalidated || !isTranscriptScopeCurrent(scope)
      || localTranscriptClientNonce(entry, scope.humanAuthorId) !== scope.nonce)) return;
    const attachmentEcho = "sourceKind" in entry && entry.sourceKind === "user-attachment";
    if (!isSettledTranscriptMessage(entry) && (human || !attachmentEcho)) return;
    if (scope != null && scope.agentId === agentId
      && localTranscriptClientNonce(entry, scope.humanAuthorId) === scope.nonce
      && isSettledTranscriptMessage(entry)) markTranscriptSubmissionSettled(scope);
    else if (human && isSettledTranscriptMessage(entry)) {
      const operation = captureTranscriptScope(agentId);
      if (operation != null) settlePersistedHumanDraft({ ...operation, humanAuthorId: current.authId || null }, nonce);
    }
    acknowledgementController.ingestTranscriptEvent({
      accountSlot: current.authId ?? current.email ?? "account", agentId,
      entry: { id: entry.id, kind: attachmentEcho ? "user-attachment" : "message", clientNonce: nonce },
    });
  }, [accountRef, acknowledgementController, captureTranscriptScope, humanConversationsRef, isTranscriptScopeCurrent, markTranscriptSubmissionSettled, settlePersistedHumanDraft, submissionScopesRef]);

  const invalidateTranscriptConversation = useCallback((agentId: string) => {
    const operation = captureTranscriptScope(agentId);
    if (operation != null) composerDraftStore.forgetHumanSubmissions({ accountSlot: operation.accountSlot, agentId });
    humanCoverageContinuationInFlightRef.current.delete(agentId);
    transcriptConversationGenerationsRef.current.set(agentId, (transcriptConversationGenerationsRef.current.get(agentId) ?? 0) + 1);
    for (const [nonce, scope] of submissionScopesRef.current) {
      if (scope.agentId !== agentId) continue;
      scope.invalidate();
      acknowledgementController.removeOptimistic({ accountSlot: scope.accountSlot, nonce });
      composerSubmissionQueueRef.current?.cancelQueued(nonce);
      composerSubmissionQueueRef.current?.discard(nonce);
      submissionScopesRef.current.delete(nonce);
    }
    for (const read of pendingTranscriptReceiptsRef.current) {
      if (read.agentId === agentId) pendingTranscriptReceiptsRef.current.delete(read);
    }
    setPendingSendScopes((current) => {
      const scope = current.get(agentId);
      if (scope == null || (!scope.invalidated && isTranscriptScopeCurrent(scope))) return current;
      const next = new Map(current);
      next.delete(agentId);
      return next;
    });
  }, [acknowledgementController, captureTranscriptScope, composerDraftStore, composerSubmissionQueueRef, humanCoverageContinuationInFlightRef, isTranscriptScopeCurrent, pendingTranscriptReceiptsRef, setPendingSendScopes, submissionScopesRef, transcriptConversationGenerationsRef]);

  return {
    isTranscriptScopeCurrent,
    captureTranscriptScope,
    isSpreadsheetSourceCurrent,
    invalidateActiveHumanSearch,
    settlePersistedHumanDraft,
    settleTranscriptDraft,
    scopeComposerSubmission,
    markTranscriptSubmissionSettled,
    observeTranscriptSettlement,
    invalidateTranscriptConversation,
  };
}
