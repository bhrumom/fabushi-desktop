import type { SpreadsheetViewerMount } from "../../recovered/features/conversation/workspace/spreadsheet-viewer-provider";
import type { ComposerDraft, DraftAttachment } from "../../recovered/features/conversation/workspace/model";
import type { ComposerSubmission } from "../../recovered/features/conversation/workspace/submission";

export type TranscriptLoadErrorState = {
  readonly agentId: string;
  readonly accountScopeGeneration: number;
};

export type TranscriptOperationScope = {
  readonly accountSlot: string;
  readonly accountGeneration: number;
  readonly agentId: string;
  readonly conversationGeneration: number;
  readonly humanAuthorId: string | null | undefined;
};

export type TranscriptSpreadsheetSource = TranscriptOperationScope & {
  readonly entryId: string;
  readonly attachmentIdentity: string;
  mount: SpreadsheetViewerMount;
};

export type TranscriptDraftOwnership = {
  draft: ComposerDraft | null;
  draftIdentity: { agentId: string; draftId: string } | null;
  recovery: ComposerDraft | null;
  recoveryIdentity: { agentId: string; recoveryId: string } | null;
  settled: boolean;
};

export type TranscriptSubmissionScope = TranscriptOperationScope & {
  readonly nonce: string;
  readonly cancelled: Promise<void>;
  invalidate(): void;
  invalidated: boolean;
  canonicalSettled: boolean;
  /** Proven by this existing queue scope before possible Human admission. */
  humanQueuedOffline: boolean;
  draftOwnership?: TranscriptDraftOwnership;
  /** Acknowledged Agent attachment commits stay with the existing queue scope. */
  agentCommittedAttachments?: DraftAttachment[];
  onSettled?(): void;
};

export type ScopedComposerSubmission = ComposerSubmission & {
  readonly transcriptScope: TranscriptSubmissionScope;
};

export function submissionScope(submission: ComposerSubmission): TranscriptSubmissionScope | undefined {
  return (submission as Partial<ScopedComposerSubmission>).transcriptScope;
}
