import {
  composerSubmissionBusyState,
  createComposerSubmissionQueue,
  type ComposerSubmission,
  type ComposerSubmissionQueue,
  type ComposerSubmissionAdmission,
} from "../../recovered/features/conversation/workspace/submission";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import { isSettledTranscriptMessage, localTranscriptClientNonce } from "../transcript-projection";
import { observeHumanOfflineQueueOrigin } from "../human-message-submission";
import {
  submissionScope,
  type TranscriptOperationScope,
  type TranscriptSubmissionScope,
} from "./tdrp-transcript-scope";

export interface CreateProductionComposerSubmissionQueueOptions {
  readonly transportRef: React.MutableRefObject<string>;
  readonly submissionScopesRef: React.MutableRefObject<Map<string, TranscriptSubmissionScope>>;
  readonly isTranscriptScopeCurrent: (scope: TranscriptOperationScope) => boolean;
  readonly resendSubmissionNoncesRef: React.MutableRefObject<Set<string>>;
  readonly sendComposerPrompt: (submission: ComposerSubmission, admission?: ComposerSubmissionAdmission) => Promise<void>;
  readonly sendJournalApprovalLifecycle: any;
  readonly setPendingSendScopes: React.Dispatch<React.SetStateAction<ReadonlyMap<string, TranscriptSubmissionScope>>>;
  readonly acknowledgementController: any;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly composerSubmissionQueueRef: React.MutableRefObject<ComposerSubmissionQueue | null>;
  readonly markTranscriptSubmissionSettled: (scope: TranscriptSubmissionScope) => void;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly entriesByAgentRef: React.MutableRefObject<Record<string, readonly ConversationTranscriptEntry[]>>;
  readonly composerDraftStore: any;
}

export function createProductionComposerSubmissionQueue({
  transportRef,
  submissionScopesRef,
  isTranscriptScopeCurrent,
  resendSubmissionNoncesRef,
  sendComposerPrompt,
  sendJournalApprovalLifecycle,
  setPendingSendScopes,
  acknowledgementController,
  setEntriesByAgent,
  composerSubmissionQueueRef,
  markTranscriptSubmissionSettled,
  setNotice,
  entriesByAgentRef,
  composerDraftStore,
}: CreateProductionComposerSubmissionQueueOptions): ComposerSubmissionQueue {
  return createComposerSubmissionQueue({
    isTransportDown: () => transportRef.current !== "connected",
    send: (submission, admission) => {
      const scope = submissionScope(submission);
      if (scope == null || scope.invalidated || !isTranscriptScopeCurrent(scope)) return Promise.resolve();
      const sending = admission != null || resendSubmissionNoncesRef.current.has(submission.nonce)
        ? sendComposerPrompt(submission, admission)
        : sendJournalApprovalLifecycle.sendPrompt(submission);
      return Promise.race([
        sending,
        scope.cancelled.then(() => {
          throw new Error("Transcript send scope was cleared");
        }),
      ]);
    },
    onPhase: (submission) => {
      const scope = submissionScope(submission);
      if (scope == null || scope.invalidated || !isTranscriptScopeCurrent(scope) || submission.isCurrentAttempt?.() === false) return;
      if (typeof scope.humanAuthorId === "string" && !scope.canonicalSettled) {
        scope.humanQueuedOffline = observeHumanOfflineQueueOrigin(
          scope.humanQueuedOffline,
          submission.phase,
          transportRef.current !== "connected",
          submission.humanDispatch?.requestPosted === true
        );
      }
      const composerBusy = composerSubmissionBusyState(submission);
      setPendingSendScopes((current) => {
        if (composerBusy === undefined || submission.isCurrentAttempt?.() === false) return current;
        if (composerBusy) {
          if (scope.invalidated || !isTranscriptScopeCurrent(scope) || current.get(scope.agentId) === scope) return current;
          return new Map(current).set(scope.agentId, scope);
        }
        if (current.get(scope.agentId) !== scope) return current;
        const next = new Map(current);
        next.delete(scope.agentId);
        return next;
      });
      if (!scope.canonicalSettled) {
        if (submission.phase === "pending") acknowledgementController.markDispatching(scope.accountSlot, submission.nonce);
        if (submission.phase === "sent") acknowledgementController.markAcceptedAwaitingEcho(scope.accountSlot, submission.nonce);
        if (submission.phase === "failed") acknowledgementController.markFailed(scope.accountSlot, submission.nonce, Date.now());
        if (submission.phase === "cancelled") acknowledgementController.removeOptimistic({ accountSlot: scope.accountSlot, nonce: submission.nonce });
      }
      setEntriesByAgent((current) => {
        if (scope.invalidated || !isTranscriptScopeCurrent(scope) || submission.isCurrentAttempt?.() === false) return current;
        const matches = (entry: ConversationTranscriptEntry) => localTranscriptClientNonce(entry, scope.humanAuthorId) === submission.nonce;
        if (submission.phase === "cancelled") {
          return {
            ...current,
            [submission.agentId]: (current[submission.agentId] ?? []).filter((entry) => !matches(entry) || isSettledTranscriptMessage(entry)),
          };
        }
        const delivery: "pending" | "queued" | "failed" | "sent" =
          submission.phase === "pending" || submission.phase === "queued" || submission.phase === "failed" || submission.phase === "sent"
            ? submission.phase
            : "pending";
        return {
          ...current,
          [submission.agentId]: (current[submission.agentId] ?? []).map((entry) =>
            entry.kind === "message" && matches(entry) && !scope.canonicalSettled && !isSettledTranscriptMessage(entry)
              ? { ...entry, delivery }
              : entry
          ),
        };
      });
      if (submission.phase === "sent" || submission.phase === "cancelled") {
        if (submissionScopesRef.current.get(submission.nonce) === scope) submissionScopesRef.current.delete(submission.nonce);
      }
    },
    onFailure: (submission, error) => {
      if (submission.isCurrentAttempt?.() === false) return;
      const scope = submissionScope(submission);
      if (scope == null || scope.invalidated || !isTranscriptScopeCurrent(scope) || scope.canonicalSettled) {
        composerSubmissionQueueRef.current?.discard(submission.nonce);
        composerSubmissionQueueRef.current?.flush();
        if (scope != null && !scope.invalidated && isTranscriptScopeCurrent(scope) && scope.canonicalSettled) {
          markTranscriptSubmissionSettled(scope);
          if (submissionScopesRef.current.get(submission.nonce) === scope) submissionScopesRef.current.delete(submission.nonce);
          if (!submission.isRetry) setNotice(error instanceof Error ? error.message : String(error));
        }
        return;
      }
      if (submission.isRetry) {
        setNotice((current) => {
          if (
            scope.invalidated
            || !isTranscriptScopeCurrent(scope)
            || submission.isCurrentAttempt?.() === false
            || scope.canonicalSettled
            || entriesByAgentRef.current[submission.agentId]?.some(
              (entry) => localTranscriptClientNonce(entry, scope.humanAuthorId) === submission.nonce && isSettledTranscriptMessage(entry)
            )
          ) {
            return current;
          }
          return error instanceof Error ? error.message : String(error);
        });
        return;
      }
      const beforeRecovery = composerDraftStore.snapshotsFor(submission.agentId).get();
      composerDraftStore.recoverDraft(
        submission.agentId,
        {
          prompt: submission.prompt,
          attachments: [...submission.attachments],
          ...(submission.richText === undefined ? {} : { richText: submission.richText }),
          ...(submission.replyToId === undefined ? {} : { replyToId: submission.replyToId }),
          ...(submission.isFork === undefined ? {} : { isFork: submission.isFork }),
        },
        typeof scope.humanAuthorId === "string"
          ? { accountSlot: scope.accountSlot, authorId: scope.humanAuthorId, nonce: submission.nonce }
          : undefined
      );
      const recovered = composerDraftStore.snapshotsFor(submission.agentId).get();
      const ownership = (scope.draftOwnership ??= {
        draft: null,
        draftIdentity: null,
        recovery: null,
        recoveryIdentity: null,
        settled: false,
      });
      if (beforeRecovery.draft == null || beforeRecovery.draft === ownership.draft) {
        ownership.draft = recovered.draft;
        ownership.draftIdentity =
          recovered.draft == null
            ? null
            : composerDraftStore.identifyDraft({ agentId: submission.agentId, draft: recovered.draft });
      }
      if (beforeRecovery.draft != null) {
        ownership.recovery = recovered.recovery;
        ownership.recoveryIdentity =
          recovered.recovery == null
            ? null
            : composerDraftStore.identifyRecovery({ agentId: submission.agentId, draft: recovered.recovery });
      }
    },
  });
}
