import type { ComposerSubmission } from "../../recovered/features/conversation/workspace/submission";
import type { DesktopBridge } from "../../recovered/contracts/desktop-bridge";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import {
  applyTranscriptReadUpdates,
  isSettledTranscriptMessage,
  localTranscriptClientNonce,
  reconcileTranscriptMutation,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import { sendHumanComposerSubmission } from "../human-message-submission";
import { projectHumanConversations, projectTranscriptEntry, type RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { TranscriptSubmissionScope } from "./tdrp-transcript-scope";

export interface SendHumanTranscriptSubmissionOptions {
  readonly submission: ComposerSubmission;
  readonly scope: TranscriptSubmissionScope;
  readonly humanConversation: RendererAgent;
  readonly client: ProductionCoordinatorClient;
  readonly bridge: DesktopBridge;
  readonly isCurrent: () => boolean;
  readonly stagedPaths: React.MutableRefObject<Set<string>>;
  readonly pendingTranscriptReceiptsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly markTranscriptSubmissionSettled: (scope: TranscriptSubmissionScope) => void;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly setHumanConversations: React.Dispatch<React.SetStateAction<RendererAgent[]>>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly composerDraftStore: any;
}

export async function sendHumanTranscriptSubmission({
  submission,
  scope,
  humanConversation,
  client,
  bridge,
  isCurrent,
  stagedPaths,
  pendingTranscriptReceiptsRef,
  markTranscriptSubmissionSettled,
  setEntriesByAgent,
  humanConversationsRef,
  setHumanConversations,
  setNotice,
  composerDraftStore,
}: SendHumanTranscriptSubmissionOptions): Promise<void> {
  await composerDraftStore.bindHumanSubmission({
    accountSlot: scope.accountSlot,
    agentId: scope.agentId,
    authorId: scope.humanAuthorId,
    nonce: scope.nonce,
    draftId: scope.draftOwnership?.draftIdentity?.draftId ?? null,
    recoveryId: scope.draftOwnership?.recoveryIdentity?.recoveryId ?? null,
  });
  if (!isCurrent()) throw new Error("Transcript send scope was cleared");
  const receipt: TranscriptReadUpdateBuffer = {
    accountSlot: scope.accountSlot,
    agentId: scope.agentId,
    isCurrent,
    updates: new Map(),
  };
  pendingTranscriptReceiptsRef.current.add(receipt);
  try {
    const result = await sendHumanComposerSubmission(submission, {
      client,
      bridge,
      isCurrent,
      queuedOffline: scope.humanQueuedOffline,
      onCommitted: (originalPath) => {
        if (isCurrent()) stagedPaths.current.delete(originalPath);
      },
    });
    if (!receipt.isCurrent()) throw new Error("Transcript send scope was cleared");
    const authoritative = projectTranscriptEntry(result, 0, humanConversation.name, humanConversation.id);
    if (
      authoritative == null
      || !isSettledTranscriptMessage(authoritative)
      || localTranscriptClientNonce(authoritative, scope.humanAuthorId) !== submission.nonce
    ) {
      throw new Error("Human send settlement returned an invalid transcript entry");
    }
    const settled = applyTranscriptReadUpdates([authoritative], receipt.updates)[0];
    markTranscriptSubmissionSettled(scope);
    setEntriesByAgent((current) => {
      if (!receipt.isCurrent()) return current;
      return {
        ...current,
        [submission.agentId]: reconcileTranscriptMutation(current[submission.agentId] ?? [], settled, {
          previousId: `pending-${submission.nonce}`,
          appendIfMissing: true,
          preserveSettled: true,
        }),
      };
    });
    try {
      const projectedHumans = projectHumanConversations(await client.call("listHumanConversations"));
      if (!receipt.isCurrent()) throw new Error("Transcript send scope was cleared");
      humanConversationsRef.current = projectedHumans;
      setHumanConversations(projectedHumans);
    } catch (error) {
      if (!receipt.isCurrent()) throw new Error("Transcript send scope was cleared");
      setNotice((current) =>
        !receipt.isCurrent()
          ? current
          : error instanceof Error
          ? `Human roster refresh unavailable: ${error.message}`
          : `Human roster refresh unavailable: ${String(error)}`
      );
    }
  } finally {
    pendingTranscriptReceiptsRef.current.delete(receipt);
  }
}
