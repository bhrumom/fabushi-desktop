import { useCallback } from "react";
import {
  applyTranscriptReadUpdates,
  reconcileTranscriptRefresh,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import {
  projectHumanConversations,
  projectHumanConversationTranscript,
  type RendererAgent,
} from "../model";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import type { ProductionCoordinatorClient } from "../coordinator-client";

export interface UseTdrpHistoryCoverageOptions {
  readonly client: ProductionCoordinatorClient | null;
  readonly transportRef: React.MutableRefObject<string>;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly selectionStore: any;
  readonly humanCoverageContinuationInFlightRef: React.MutableRefObject<Set<string>>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly transportScopeGenerationRef: React.MutableRefObject<number>;
  readonly transcriptConversationGenerationsRef: React.MutableRefObject<Map<string, number>>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly setHumanConversations: React.Dispatch<React.SetStateAction<RendererAgent[]>>;
  readonly entriesByAgentRef: React.MutableRefObject<Record<string, readonly ConversationTranscriptEntry[]>>;
  readonly openAgentRequestGenerationRef: React.MutableRefObject<number>;
  readonly pendingTranscriptReadsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly observeTranscriptSettlement: (agentId: string, entry: ConversationTranscriptEntry) => void;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, ConversationTranscriptEntry[]>>>;
  readonly transcriptHistoryCursorsRef: React.MutableRefObject<Map<string, { accountSlot: string }>>;
  readonly transcriptPaginationController: any;
}

export function useTdrpHistoryCoverage({
  client,
  transportRef,
  accountRef,
  selectionStore,
  humanCoverageContinuationInFlightRef,
  accountScopeGenerationRef,
  transportScopeGenerationRef,
  transcriptConversationGenerationsRef,
  humanConversationsRef,
  setNotice,
  setHumanConversations,
  entriesByAgentRef,
  openAgentRequestGenerationRef,
  pendingTranscriptReadsRef,
  observeTranscriptSettlement,
  setEntriesByAgent,
  transcriptHistoryCursorsRef,
  transcriptPaginationController,
}: UseTdrpHistoryCoverageOptions) {
  const continueHumanHistoryCoverage = useCallback((conversationId: string) => {
    if (client == null || transportRef.current !== "connected" || accountRef.current?.kind !== "logged-in") return;
    if (selectionStore.get().currentAgentId !== conversationId) return;
    if (humanCoverageContinuationInFlightRef.current.has(conversationId)) return;
    const accountScopeGeneration = accountScopeGenerationRef.current;
    const transportScopeGeneration = transportScopeGenerationRef.current;
    const conversationGeneration = transcriptConversationGenerationsRef.current.get(conversationId) ?? 0;
    const isCurrent = () => accountScopeGenerationRef.current === accountScopeGeneration
      && transportScopeGenerationRef.current === transportScopeGeneration
      && transportRef.current === "connected"
      && accountRef.current?.kind === "logged-in"
      && selectionStore.get().currentAgentId === conversationId
      && (transcriptConversationGenerationsRef.current.get(conversationId) ?? 0) === conversationGeneration;
    humanCoverageContinuationInFlightRef.current.add(conversationId);
    void (async () => {
      try {
        const MAX_CONTINUATION_WINDOWS = 16;
        for (let windowIndex = 0; windowIndex < MAX_CONTINUATION_WINDOWS; windowIndex += 1) {
          if (!isCurrent()) return;
          const currentConversation = humanConversationsRef.current.find((candidate) => candidate.id === conversationId);
          if (currentConversation == null || currentConversation.raw.historyCoverageComplete !== false) return;
          try {
            await client.call("syncHumanConversation", { conversationId });
          } catch (error) {
            if (isCurrent()) {
              setNotice(error instanceof Error
                ? `Human sync unavailable: ${error.message}`
                : `Human sync unavailable: ${String(error)}`);
            }
            return;
          }
          if (!isCurrent()) return;
          let refreshedRosterValue: unknown;
          try {
            refreshedRosterValue = await client.call("listHumanConversations");
          } catch {
            return;
          }
          if (!isCurrent()) return;
          const refreshedHumans = projectHumanConversations(refreshedRosterValue);
          humanConversationsRef.current = refreshedHumans;
          setHumanConversations(refreshedHumans);
          const refreshedConversation = refreshedHumans.find((candidate) => candidate.id === conversationId);
          if (refreshedConversation == null) return;
          if (entriesByAgentRef.current[conversationId] != null) {
            const accountNow = accountRef.current;
            const requestGeneration = openAgentRequestGenerationRef.current;
            const pendingRead: TranscriptReadUpdateBuffer = {
              accountSlot: accountNow?.kind === "logged-in" ? accountNow.authId ?? accountNow.email ?? "account" : null,
              agentId: conversationId,
              isCurrent: () => isCurrent() && openAgentRequestGenerationRef.current === requestGeneration,
              updates: new Map(),
            };
            pendingTranscriptReadsRef.current.add(pendingRead);
            try {
              const entriesAtRequest = entriesByAgentRef.current[conversationId] ?? [];
              const transcriptValue = await client.call("getHumanConversationTranscript", { conversationId });
              if (pendingRead.isCurrent()) {
                const projectedEntries = projectHumanConversationTranscript(
                  transcriptValue,
                  refreshedConversation.name,
                  conversationId
                );
                for (const entry of projectedEntries) observeTranscriptSettlement(conversationId, entry);
                setEntriesByAgent((current) => !pendingRead.isCurrent() ? current : ({
                  ...current,
                  [conversationId]: applyTranscriptReadUpdates(reconcileTranscriptRefresh(
                    current[conversationId] ?? [],
                    projectedEntries,
                    entriesAtRequest,
                  ), pendingRead.updates)
                }));
                const latestAccount = accountRef.current;
                if (latestAccount?.kind === "logged-in") {
                  transcriptHistoryCursorsRef.current.set(conversationId, {
                    accountSlot: latestAccount.authId ?? latestAccount.email ?? "account",
                  });
                }
                openAgentRequestGenerationRef.current += 1;
                pendingTranscriptReadsRef.current.clear();
                transcriptPaginationController.installInitialPage({ entries: [] });
              }
            } catch (error) {
              if (pendingRead.isCurrent()) {
                setNotice(error instanceof Error
                  ? `Human transcript refresh unavailable: ${error.message}`
                  : `Human transcript refresh unavailable: ${String(error)}`);
              }
              return;
            } finally {
              pendingTranscriptReadsRef.current.delete(pendingRead);
            }
          }
          if (refreshedConversation.raw.historyCoverageComplete !== false) return;
        }
      } finally {
        humanCoverageContinuationInFlightRef.current.delete(conversationId);
      }
    })();
  }, [accountRef, accountScopeGenerationRef, client, entriesByAgentRef, humanConversationsRef, humanCoverageContinuationInFlightRef, observeTranscriptSettlement, openAgentRequestGenerationRef, pendingTranscriptReadsRef, selectionStore, setEntriesByAgent, setHumanConversations, setNotice, transcriptConversationGenerationsRef, transcriptHistoryCursorsRef, transcriptPaginationController, transportRef, transportScopeGenerationRef]);

  return { continueHumanHistoryCoverage };
}
