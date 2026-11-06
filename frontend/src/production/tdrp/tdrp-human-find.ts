import { useEffect } from "react";
import type {
  FindInChatMatch,
  FindInChatTranscriptHandle,
} from "../../recovered/features/conversation/workspace/find-in-chat-controller";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import type { CursorAuthStatus } from "../../recovered/contracts/desktop-bridge";
import { projectHumanConversationTranscript, type RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";

export interface UseTdrpHumanFindOptions {
  readonly findNavigateHandlerRef: React.MutableRefObject<((match: FindInChatMatch) => void) | null>;
  readonly transcriptHandleRef: React.MutableRefObject<FindInChatTranscriptHandle | null>;
  readonly activeAgentIdRef: React.MutableRefObject<string>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly transcriptAccountSlot: string | null;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly humanFindGenerationRef: React.MutableRefObject<number>;
  readonly humanFindUpdateGenerationRef: React.MutableRefObject<number>;
  readonly humanFindUpdateGeneration: number;
  readonly accountIdentityRef: React.MutableRefObject<string | null>;
  readonly selectionStore: any;
  readonly findInChatController: any;
  readonly findInChatOpenRef: React.MutableRefObject<boolean>;
  readonly transcriptPaginationController: any;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly activeIsHuman: boolean;
  readonly activeAgent: RendererAgent | null | undefined;
  readonly activeAgentId: string;
  readonly account: CursorAuthStatus | null;
  readonly setFindInChatOpen: React.Dispatch<React.SetStateAction<boolean>>;
  readonly entries: readonly ConversationTranscriptEntry[];
  readonly findInChatOpen: boolean;
  readonly findInChatSnapshot: any;
  readonly client: ProductionCoordinatorClient | null;
  readonly renderedAccountScopeGeneration: number;
  readonly scrollToFindMatch: (match: FindInChatMatch) => void;
}

export function useTdrpHumanFind({
  findNavigateHandlerRef,
  transcriptHandleRef,
  activeAgentIdRef,
  humanConversationsRef,
  transcriptAccountSlot,
  accountScopeGenerationRef,
  humanFindGenerationRef,
  humanFindUpdateGenerationRef,
  humanFindUpdateGeneration,
  accountIdentityRef,
  selectionStore,
  findInChatController,
  findInChatOpenRef,
  transcriptPaginationController,
  setNotice,
  activeIsHuman,
  activeAgent,
  activeAgentId,
  account,
  setFindInChatOpen,
  entries,
  findInChatOpen,
  findInChatSnapshot,
  client,
  renderedAccountScopeGeneration: _renderedAccountScopeGeneration,
  scrollToFindMatch,
}: UseTdrpHumanFindOptions): void {
  findNavigateHandlerRef.current = (match: FindInChatMatch) => {
    if (transcriptHandleRef.current?.scrollToEntryWithoutHighlight(match.entryId) === true) return;
    const conversationId = activeAgentIdRef.current;
    const isHumanConversation = humanConversationsRef.current.some((conversation) => conversation.id === conversationId);
    if (!isHumanConversation) {
      scrollToFindMatch(match);
      return;
    }
    const accountSlot = transcriptAccountSlot;
    void (async () => {
      for (let page = 0; page < 32; page += 1) {
        if (
          activeAgentIdRef.current !== conversationId
          || transcriptAccountSlot !== accountSlot
          || findInChatController.getSnapshot().scope.agentId !== conversationId
        ) return;
        if (transcriptHandleRef.current?.scrollToEntryWithoutHighlight(match.entryId) === true) return;
        const pagination = transcriptPaginationController.getSnapshot();
        if (pagination.agentId !== conversationId || pagination.accountSlot !== accountSlot || !pagination.hasOlder || pagination.olderFailure != null) return;
        await transcriptPaginationController.loadOlder();
        if (transcriptPaginationController.getSnapshot().olderFailure != null) return;
      }
      transcriptHandleRef.current?.scrollToEntryWithoutHighlight(match.entryId);
    })().catch((error: unknown) => {
      setNotice(error instanceof Error ? `Human search navigation unavailable: ${error.message}` : `Human search navigation unavailable: ${String(error)}`);
    });
  };

  useEffect(() => {
    findInChatController.setScope(transcriptAccountSlot, activeAgentId.length > 0 ? activeAgentId : null);
    if (account?.kind !== "logged-in" || activeAgentId.length === 0) setFindInChatOpen(false);
  }, [account?.kind, activeAgentId, findInChatController, setFindInChatOpen, transcriptAccountSlot]);

  useEffect(() => {
    const query = findInChatSnapshot.query.trim();
    if (!activeIsHuman || query.length === 0) findInChatController.replaceEntries(entries);
  }, [activeIsHuman, entries, findInChatController, findInChatSnapshot.query]);

  useEffect(() => {
    const query = findInChatSnapshot.query.trim();
    if (!findInChatOpen || !activeIsHuman || query.length === 0 || client == null || activeAgent == null || transcriptAccountSlot == null) return;
    const generation = ++humanFindGenerationRef.current;
    const updateGeneration = humanFindUpdateGenerationRef.current;
    const accountGeneration = accountScopeGenerationRef.current;
    const accountSlot = transcriptAccountSlot;
    const conversationId = activeAgent.id;
    const conversationName = activeAgent.name;
    const isCurrent = () => {
      const current = findInChatController.getSnapshot();
      return humanFindGenerationRef.current === generation
        && humanFindUpdateGenerationRef.current === updateGeneration
        && accountScopeGenerationRef.current === accountGeneration
        && accountIdentityRef.current === `logged-in:${accountSlot}`
        && selectionStore.get().currentAgentId === conversationId
        && current.scope.accountSlot === accountSlot && current.scope.agentId === conversationId
        && current.query.trim() === query && findInChatOpenRef.current;
    };
    // setScope can clear the controller query earlier in this same effect pass.
    if (!isCurrent()) return;
    void client.call("searchHumanMessages", {
      conversationId,
      query,
      limit: 200,
    }).then((value) => {
      if (!isCurrent()) return;
      findInChatController.replaceEntries(
        projectHumanConversationTranscript(value, conversationName, conversationId)
      );
      const settledSearch = findInChatController.getSnapshot();
      if (settledSearch.current == null && settledSearch.matches.length > 0) {
        findInChatController.step(1);
      }
    }).catch((error: unknown) => {
      if (!isCurrent()) return;
      setNotice(error instanceof Error ? `Human search unavailable: ${error.message}` : `Human search unavailable: ${String(error)}`);
    });
    return () => {
      if (humanFindGenerationRef.current === generation) humanFindGenerationRef.current += 1;
    };
  }, [accountIdentityRef, accountScopeGenerationRef, activeAgent, activeIsHuman, client, findInChatController, findInChatOpen, findInChatOpenRef, findInChatSnapshot.query, humanFindGenerationRef, humanFindUpdateGeneration, humanFindUpdateGenerationRef, selectionStore, setNotice, transcriptAccountSlot]);
}
