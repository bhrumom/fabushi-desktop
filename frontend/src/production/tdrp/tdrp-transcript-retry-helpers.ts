import { useCallback } from "react";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../../recovered/features/conversation/workspace/model";
import { isSettledTranscriptMessage, localTranscriptClientNonce } from "../transcript-projection";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { UI_TEXT } from "../evidence";
import type { RendererAgent } from "../model";
import { submissionScope, type ScopedComposerSubmission, type TranscriptOperationScope, type TranscriptSubmissionScope } from "./tdrp-transcript-scope";

export interface UseTdrpTranscriptRetryHelpersOptions {
  readonly activeAgentIdRef: React.MutableRefObject<string>;
  readonly captureTranscriptScope: (agentId: string) => TranscriptOperationScope | null;
  readonly entriesByAgentRef: React.MutableRefObject<Record<string, readonly ConversationTranscriptEntry[]>>;
  readonly submissionScopesRef: React.MutableRefObject<Map<string, TranscriptSubmissionScope>>;
  readonly composerDraftStore: any;
  readonly composerSubmissionQueueRef: React.MutableRefObject<any>;
  readonly acknowledgementController: any;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly isTranscriptScopeCurrent: (scope: TranscriptOperationScope) => boolean;
  readonly activeIsHuman: boolean;
  readonly activeAgentId: string;
  readonly activeAgent: RendererAgent | null;
  readonly composerSubmissionQueue: any;
  readonly renderedAccountScopeGeneration: number;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly transcriptAccountSlot: string | null;
  readonly openAgentRequestGenerationRef: React.MutableRefObject<number>;
  readonly renderedRetrySelectionGeneration: number;
  readonly selectionStore: any;
  readonly renderedRetrySelection: any;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly humanMessageRetryController: any;
  readonly client: ProductionCoordinatorClient | null;
  readonly scopeComposerSubmission: (submission: any) => ScopedComposerSubmission | null;
  readonly sendJournalApprovalLifecycle: any;
  readonly makeClientNonce: () => string;
  readonly optimisticAcknowledgementEntries: (nonce: string, attachments: readonly { path: string; name: string }[]) => any[];
}

export function useTdrpTranscriptRetryHelpers({
  activeAgentIdRef,
  captureTranscriptScope,
  entriesByAgentRef,
  submissionScopesRef,
  composerDraftStore,
  composerSubmissionQueueRef,
  acknowledgementController,
  setEntriesByAgent,
  isTranscriptScopeCurrent,
  activeIsHuman,
  activeAgentId,
  activeAgent,
  composerSubmissionQueue,
  renderedAccountScopeGeneration,
  accountScopeGenerationRef,
  accountRef,
  transcriptAccountSlot,
  openAgentRequestGenerationRef,
  renderedRetrySelectionGeneration,
  selectionStore,
  renderedRetrySelection,
  setNotice,
  humanMessageRetryController,
  client,
  scopeComposerSubmission,
  sendJournalApprovalLifecycle,
  makeClientNonce,
  optimisticAcknowledgementEntries,
}: UseTdrpTranscriptRetryHelpersOptions) {
  const removeTranscriptMessage = useCallback((entry: TranscriptMessage) => {
    const agentId = activeAgentIdRef.current;
    const operation = captureTranscriptScope(agentId);
    if (!agentId || operation == null || entry.delivery !== "failed") return;
    const nonce = localTranscriptClientNonce(entry, operation.humanAuthorId);
    if (nonce == null) return;
    const currentEntry = (entriesByAgentRef.current[agentId] ?? []).find((candidate) => candidate.id === entry.id);
    if (currentEntry == null || isSettledTranscriptMessage(currentEntry)) return;
    const scope = submissionScopesRef.current.get(nonce);
    if (scope?.canonicalSettled === true) return;
    if (scope != null && scope.agentId === agentId) { scope.invalidate(); submissionScopesRef.current.delete(nonce); }
    composerDraftStore.forgetHumanSubmissions({ accountSlot: operation.accountSlot, agentId, nonce });
    composerSubmissionQueueRef.current?.discard(nonce);
    acknowledgementController.removeOptimistic({ accountSlot: operation.accountSlot, nonce });
    setEntriesByAgent((current) => !isTranscriptScopeCurrent(operation) ? current : ({
      ...current,
      [agentId]: (current[agentId] ?? []).filter((candidate) => candidate.kind !== "message" || candidate.id !== entry.id)
    }));
  }, [acknowledgementController, activeAgentIdRef, captureTranscriptScope, composerDraftStore, composerSubmissionQueueRef, entriesByAgentRef, isTranscriptScopeCurrent, setEntriesByAgent, submissionScopesRef]);

  const resendFailedSend = useCallback(async (entry: TranscriptMessage) => {
    if (activeIsHuman) {
      const operation = captureTranscriptScope(activeAgentId);
      if (operation == null || localTranscriptClientNonce(entry, operation.humanAuthorId) == null
        || isSettledTranscriptMessage(entry)) return;
      const original = composerSubmissionQueue.snapshot().find((record: any) => record.agentId === activeAgentId
        && record.nonce === entry.clientNonce && (record.phase === "pending" || record.phase === "queued"
          || record.humanDispatch?.requestPosted !== true));
      const originalScope = entry.clientNonce == null ? null : submissionScopesRef.current.get(entry.clientNonce);
      const retryIsCurrent = () => originalScope != null && !originalScope.invalidated
        && isTranscriptScopeCurrent(originalScope)
        && originalScope.accountGeneration === renderedAccountScopeGeneration
        && renderedAccountScopeGeneration === accountScopeGenerationRef.current
        && accountRef.current?.kind === "logged-in"
        && (accountRef.current.authId ?? accountRef.current.email ?? "account") === transcriptAccountSlot
        && openAgentRequestGenerationRef.current === renderedRetrySelectionGeneration
        && selectionStore.get() === renderedRetrySelection
        && renderedRetrySelection.currentAgentId === activeAgentId;
      if (original != null && entry.delivery === "failed" && retryIsCurrent()) {
        setNotice((current) => retryIsCurrent() ? null : current);
        const retry = composerSubmissionQueue.retryFailed(original.nonce, { isCurrent: retryIsCurrent });
        if (retry != null) { await retry.completion; return; }
      }
      try {
        await composerDraftStore.bindHumanSubmission({
          accountSlot: operation.accountSlot, agentId: activeAgentId, authorId: operation.humanAuthorId!, nonce: entry.clientNonce!,
          draftId: originalScope?.draftOwnership?.draftIdentity?.draftId ?? null,
          recoveryId: originalScope?.draftOwnership?.recoveryIdentity?.recoveryId ?? null,
        });
      } catch (error) {
        if (isTranscriptScopeCurrent(operation)) setNotice(error instanceof Error ? error.message : String(error));
        return;
      }
      if (!isTranscriptScopeCurrent(operation)) return;
      await humanMessageRetryController.retry({
        scope: {
          accountGeneration: renderedAccountScopeGeneration,
          selectionGeneration: renderedRetrySelectionGeneration,
          accountSlot: transcriptAccountSlot,
          selection: renderedRetrySelection,
        },
        conversationId: activeAgentId,
        conversationName: activeAgent?.name ?? UI_TEXT.title,
        entry,
      });
      return;
    }
    const agentId = activeAgentIdRef.current;
    if (client == null || !agentId) return;
    const operation = captureTranscriptScope(agentId);
    if (operation == null || entry.delivery !== "failed") return;
    const failedNonce = localTranscriptClientNonce(entry, operation.humanAuthorId);
    if (failedNonce == null) return;
    const currentEntry = (entriesByAgentRef.current[agentId] ?? []).find((candidate) => candidate.id === entry.id);
    if (currentEntry == null || isSettledTranscriptMessage(currentEntry)) return;
    const previousScope = submissionScopesRef.current.get(failedNonce);
    if (previousScope?.canonicalSettled === true) return;
    const retained = previousScope != null && !previousScope.invalidated && isTranscriptScopeCurrent(previousScope)
      ? composerSubmissionQueue.snapshot().find((record: any) => record.nonce === failedNonce && record.agentId === agentId
        && record.phase === "failed" && submissionScope(record) === previousScope)
      : undefined;
    const clientNonce = makeClientNonce();
    const pendingId = `pending-${clientNonce}`;
    const enteredAt = Date.now();
    const retryAttachments = (retained?.attachments ?? entry.attachments ?? []).map((attachment: any) => ({ ...attachment }));
    const retryRichText = retained == null ? entry.richText : retained.richText;
    const retryReplyToId = retained == null ? entry.replyToId : retained.replyToId;
    const retrySubmission = scopeComposerSubmission({
      nonce: clientNonce,
      agentId,
      prompt: retained?.prompt ?? entry.text.trim(),
      attachments: retryAttachments,
      createdAtMs: enteredAt,
      ...(retryRichText === undefined ? {} : { richText: retryRichText }),
      ...(retryReplyToId === undefined ? {} : { replyToId: retryReplyToId }),
      ...(retained?.isFork === undefined ? {} : { isFork: retained.isFork }),
    });
    if (retrySubmission == null) return;
    const retryScope = retrySubmission.transcriptScope;
    if (previousScope?.agentId === agentId && !previousScope.invalidated && isTranscriptScopeCurrent(previousScope)) {
      retryScope.draftOwnership = previousScope.draftOwnership;
      retryScope.agentCommittedAttachments = previousScope.agentCommittedAttachments?.map((attachment: any) => ({ ...attachment }));
    }
    if (retained == null) {
      retryScope.agentCommittedAttachments = retryAttachments.map((attachment: any) => ({ ...attachment }));
    }
    if (previousScope != null && previousScope.agentId === agentId) {
      previousScope.invalidate();
      if (submissionScopesRef.current.get(failedNonce) === previousScope) submissionScopesRef.current.delete(failedNonce);
    }
    composerSubmissionQueue.discard(failedNonce);
    const retried = acknowledgementController.retryFailed({
      accountSlot: retryScope.accountSlot,
      agentId,
      nonce: failedNonce,
      freshNonce: clientNonce,
      entries: optimisticAcknowledgementEntries(clientNonce, retryAttachments)
    });
    if (!retried) acknowledgementController.insertOptimistic({
      accountSlot: retryScope.accountSlot,
      agentId,
      nonce: clientNonce,
      entries: optimisticAcknowledgementEntries(clientNonce, retryAttachments),
      phase: "pending"
    });
    setEntriesByAgent((current) => retryScope.invalidated || !isTranscriptScopeCurrent(retryScope) ? current : ({
      ...current,
      [agentId]: (current[agentId] ?? []).map((candidate) => candidate.kind === "message" && candidate.id === entry.id
        ? { ...candidate, id: pendingId, clientNonce, delivery: "pending", composedAtMs: undefined, remoteMessageId: undefined }
        : candidate)
    }));
    const journaledRetry = sendJournalApprovalLifecycle.resendFailed({ submission: retrySubmission, onJournaled: () => {} });
    void journaledRetry.then((phase: string) => {
      if (phase === "sent") retryScope.onSettled?.();
    });
  }, [accountRef, accountScopeGenerationRef, acknowledgementController, activeAgent?.name, activeAgentId, activeAgentIdRef, activeIsHuman, captureTranscriptScope, client, composerDraftStore, composerSubmissionQueue, entriesByAgentRef, humanMessageRetryController, isTranscriptScopeCurrent, makeClientNonce, openAgentRequestGenerationRef, optimisticAcknowledgementEntries, renderedAccountScopeGeneration, renderedRetrySelection, renderedRetrySelectionGeneration, scopeComposerSubmission, selectionStore, sendJournalApprovalLifecycle, setEntriesByAgent, setNotice, submissionScopesRef, transcriptAccountSlot]);

  const cancelQueuedSend = useCallback((entry: TranscriptMessage) => {
    const agentId = activeAgentIdRef.current;
    const operation = captureTranscriptScope(agentId);
    if (operation == null) return;
    const nonce = localTranscriptClientNonce(entry, operation.humanAuthorId);
    if (nonce == null) return;
    const currentEntry = (entriesByAgentRef.current[agentId] ?? []).find((candidate) => candidate.id === entry.id);
    if (currentEntry == null || isSettledTranscriptMessage(currentEntry)) return;
    if (submissionScopesRef.current.get(nonce)?.canonicalSettled === true) return;
    const queuedSubmission = composerSubmissionQueue.snapshot().find((submission: any) =>
      submission.nonce === nonce && submission.agentId === agentId && submission.phase === "queued");
    if (composerSubmissionQueue.cancelQueued(nonce)) {
      composerDraftStore.forgetHumanSubmissions({ accountSlot: operation.accountSlot, agentId, nonce });
      acknowledgementController.removeOptimistic({ accountSlot: operation.accountSlot, nonce });
      if (agentId.length > 0) composerDraftStore.recoverDraft(agentId, {
        prompt: queuedSubmission?.prompt ?? entry.text,
        attachments: queuedSubmission == null
          ? (entry.attachments ?? []).map(({ path, name }) => ({ path, name }))
          : [...queuedSubmission.attachments],
        ...(queuedSubmission?.richText === undefined ? {} : { richText: queuedSubmission.richText }),
        ...(queuedSubmission?.replyToId === undefined ? {} : { replyToId: queuedSubmission.replyToId }),
        ...(queuedSubmission?.isFork === undefined ? {} : { isFork: queuedSubmission.isFork }),
      });
      return;
    }
    removeTranscriptMessage(entry);
  }, [acknowledgementController, activeAgentIdRef, captureTranscriptScope, composerDraftStore, composerSubmissionQueue, entriesByAgentRef, removeTranscriptMessage, submissionScopesRef]);

  return {
    removeTranscriptMessage,
    resendFailedSend,
    cancelQueuedSend,
  };
}
