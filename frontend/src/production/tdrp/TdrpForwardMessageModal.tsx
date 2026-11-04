import React from "react";
import { ForwardMessageDialog, type ForwardRecipient, type ForwardSettlement } from "../ForwardMessageDialog";
import type { TranscriptMessage } from "../../recovered/features/conversation/workspace/model";

export interface ForwardTargetState {
  readonly sourceConversationId: string;
  readonly message: TranscriptMessage;
  readonly accountScopeGeneration: number;
}

export interface TdrpForwardMessageModalProps {
  readonly forwardTarget: ForwardTargetState | null;
  readonly fallbackAccountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly forwardHumanMessage: (input: any, admission?: any) => Promise<readonly ForwardSettlement[]>;
  readonly searchForwardRecipients: (input: any) => Promise<readonly ForwardRecipient[]>;
  readonly settleForward: (
    settlement: readonly ForwardSettlement[],
    accountScopeGeneration: number,
    recipientNames: ReadonlyMap<string, string>
  ) => void;
  readonly onClose: () => void;
  readonly shareUrl?: string;
}

export function TdrpForwardMessageModal({
  forwardTarget,
  fallbackAccountScopeGeneration,
  isAccountScopeCurrent,
  forwardHumanMessage,
  searchForwardRecipients,
  settleForward,
  onClose,
  shareUrl,
}: TdrpForwardMessageModalProps): React.JSX.Element {
  return (
    <ForwardMessageDialog
      accountScopeGeneration={forwardTarget?.accountScopeGeneration ?? fallbackAccountScopeGeneration}
      forwardMessage={forwardHumanMessage}
      isAccountScopeCurrent={isAccountScopeCurrent}
      message={forwardTarget?.message ?? null}
      onClose={onClose}
      onSettled={settleForward}
      open={forwardTarget != null && isAccountScopeCurrent(forwardTarget.accountScopeGeneration)}
      searchRecipients={searchForwardRecipients}
      sourceConversationId={forwardTarget?.sourceConversationId ?? null}
      shareUrl={shareUrl}
    />
  );
}
