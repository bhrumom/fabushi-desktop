import { useCallback, useState } from "react";
import type { ForwardRecipient, ForwardSettlement } from "../ForwardMessageDialog";
import { settleForwardProjection } from "../forward-settlement";
import { projectHumanConversations, type RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { ForwardTargetState } from "./TdrpForwardMessageModal";

export interface UseTdrpForwardControllerOptions {
  readonly client: ProductionCoordinatorClient | null;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly accountIdentityRef: React.MutableRefObject<string | null>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly agentsRef: React.MutableRefObject<RendererAgent[]>;
  readonly setHumanConversations: React.Dispatch<React.SetStateAction<RendererAgent[]>>;
  readonly reconcileCompleteRosterSelection: (projected: readonly RendererAgent[]) => void;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
}

export interface TdrpForwardControllerResult {
  readonly forwardTarget: ForwardTargetState | null;
  readonly setForwardTarget: React.Dispatch<React.SetStateAction<ForwardTargetState | null>>;
  readonly isForwardAccountScopeCurrent: (generation: number) => boolean;
  readonly searchForwardRecipients: (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    query: string;
    limit: number;
  }) => Promise<readonly ForwardRecipient[]>;
  readonly forwardHumanMessage: (
    input: {
      sourceConversationId: string;
      sourceEntryId: string;
      destinationConversationIds: readonly string[];
      clientNonce: string;
      dropSenderNames: boolean;
      dropCaptions: boolean;
      comment?: { text: string; entities?: readonly unknown[] };
      destinationTopics?: Readonly<Record<string, string>>;
    },
    admission?: Parameters<ProductionCoordinatorClient["call"]>[2]
  ) => Promise<readonly ForwardSettlement[]>;
  readonly settleForward: (
    settlement: readonly ForwardSettlement[],
    accountScopeGeneration: number,
    recipientNames: ReadonlyMap<string, string>
  ) => void;
}

export function useTdrpForwardController({
  client,
  accountRef,
  accountIdentityRef,
  accountScopeGenerationRef,
  humanConversationsRef,
  agentsRef,
  setHumanConversations,
  reconcileCompleteRosterSelection,
  setNotice,
}: UseTdrpForwardControllerOptions): TdrpForwardControllerResult {
  const [forwardTarget, setForwardTarget] = useState<ForwardTargetState | null>(null);

  const isForwardAccountScopeCurrent = useCallback(
    (generation: number) =>
      accountScopeGenerationRef.current === generation
      && accountIdentityRef.current?.startsWith("logged-in:") === true,
    [accountIdentityRef, accountScopeGenerationRef]
  );

  const searchForwardRecipients = useCallback(async (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    query: string;
    limit: number;
  }): Promise<readonly ForwardRecipient[]> => {
    if (client == null) throw new Error("coordinator is unavailable for recipient search");
    const raw = await client.call("searchHumanRecipients", input);
    return projectHumanConversations(raw).map((conversation) => {
      const rawItem = conversation.raw as Record<string, unknown>;
      const topicId = typeof rawItem.topicId === "string" ? rawItem.topicId : undefined;
      const topicTitle = typeof rawItem.topicTitle === "string" ? rawItem.topicTitle : undefined;
      const topicClosed = typeof rawItem.topicClosed === "boolean" ? rawItem.topicClosed : undefined;
      const topicDeleted = typeof rawItem.topicDeleted === "boolean" ? rawItem.topicDeleted : undefined;
      return {
        id: conversation.id,
        name: conversation.name,
        ...(topicId !== undefined ? { topicId } : {}),
        ...(topicTitle !== undefined ? { topicTitle } : {}),
        ...(topicClosed !== undefined ? { topicClosed } : {}),
        ...(topicDeleted !== undefined ? { topicDeleted } : {}),
      };
    });
  }, [client]);

  const forwardHumanMessage = useCallback(async (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    destinationConversationIds: readonly string[];
    clientNonce: string;
    dropSenderNames: boolean;
    dropCaptions: boolean;
    comment?: { text: string; entities?: readonly unknown[] };
    destinationTopics?: Readonly<Record<string, string>>;
  }, admission?: Parameters<ProductionCoordinatorClient["call"]>[2]): Promise<readonly ForwardSettlement[]> => {
    if (client == null) throw new Error("coordinator is unavailable for forwarding");
    const result = await client.call("forwardHumanMessage", input, admission);
    if (result == null || typeof result !== "object" || !("destinations" in result) || !Array.isArray(result.destinations)) {
      throw new Error("forwardHumanMessage returned an invalid settlement");
    }
    return result.destinations.map((candidate) => {
      if (candidate == null || typeof candidate !== "object") {
        throw new Error("forwardHumanMessage returned an invalid destination settlement");
      }
      const conversationId = "conversationId" in candidate && typeof candidate.conversationId === "string"
        ? candidate.conversationId
        : null;
      const status = "status" in candidate && (candidate.status === "sent" || candidate.status === "failed")
        ? candidate.status
        : null;
      if (conversationId == null || status == null) {
        throw new Error("forwardHumanMessage returned an invalid destination settlement");
      }
      return {
        conversationId,
        status,
        ...("error" in candidate && typeof candidate.error === "string" ? { error: candidate.error } : {}),
      };
    });
  }, [client]);

  const settleForward = useCallback((
    settlement: readonly ForwardSettlement[],
    accountScopeGeneration: number,
    recipientNames: ReadonlyMap<string, string>
  ) => {
    const currentAccount = accountRef.current;
    const selfAuthorId = currentAccount?.kind === "logged-in" && typeof currentAccount.authId === "string" && currentAccount.authId.trim().length > 0
      ? currentAccount.authId.trim()
      : null;
    void settleForwardProjection({
      settlement,
      recipientNames,
      selfConversationId: selfAuthorId == null ? null : `human:${selfAuthorId}`,
      accountScopeGeneration,
      isAccountScopeCurrent: isForwardAccountScopeCurrent,
      setNotice,
      readRoster: client == null ? null : () => client.call("listHumanConversations"),
      applyRoster: (value) => {
        const projected = projectHumanConversations(value);
        humanConversationsRef.current = projected;
        setHumanConversations(projected);
        reconcileCompleteRosterSelection([...projected, ...agentsRef.current].sort((left, right) => right.updatedAt - left.updatedAt));
      },
    });
  }, [accountRef, agentsRef, client, humanConversationsRef, isForwardAccountScopeCurrent, reconcileCompleteRosterSelection, setHumanConversations, setNotice]);

  return {
    forwardTarget,
    setForwardTarget,
    isForwardAccountScopeCurrent,
    searchForwardRecipients,
    forwardHumanMessage,
    settleForward,
  };
}
