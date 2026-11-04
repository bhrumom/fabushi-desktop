import { useCallback } from "react";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import {
  applyTranscriptReadUpdates,
  reconcileLateInitialTranscriptPage,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import {
  projectTranscriptPageResult,
  type RendererAgent,
} from "../model";
import { UI_TEXT } from "../evidence";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { TranscriptLoadErrorState } from "./tdrp-transcript-scope";

export interface UseTdrpOpenAgentOptions {
  readonly client: ProductionCoordinatorClient | null;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly transcriptConversationGenerationsRef: React.MutableRefObject<Map<string, number>>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly transportScopeGenerationRef: React.MutableRefObject<number>;
  readonly selectionStore: any;
  readonly setOverlay: React.Dispatch<React.SetStateAction<any>>;
  readonly setWorkspaceRoute: React.Dispatch<React.SetStateAction<any>>;
  readonly setCommandPaletteOpen: React.Dispatch<React.SetStateAction<boolean>>;
  readonly setPaletteMessageTarget: React.Dispatch<React.SetStateAction<any>>;
  readonly paletteMessagePagesLoadedRef: React.MutableRefObject<number>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly initialTranscriptRequestRef: React.MutableRefObject<any>;
  readonly openAgentRequestGenerationRef: React.MutableRefObject<number>;
  readonly transcriptHistoryCursorsRef: React.MutableRefObject<Map<string, { accountSlot: string; nextBeforeSeq?: number; nextBeforeCursor?: string }>>;
  readonly entriesByAgentRef: React.MutableRefObject<Record<string, readonly ConversationTranscriptEntry[]>>;
  readonly transcriptAccountSlot: string | null;
  readonly transcriptPaginationController: any;
  readonly activeAgentIdRef: React.MutableRefObject<string>;
  readonly continueHumanHistoryCoverage: (conversationId: string) => void;
  readonly pendingTranscriptReadsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly setTranscriptLoadError: React.Dispatch<React.SetStateAction<TranscriptLoadErrorState | null>>;
  readonly agentsRef: React.MutableRefObject<RendererAgent[]>;
  readonly transportRef: React.MutableRefObject<string>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly observeTranscriptSettlement: (agentId: string, entry: ConversationTranscriptEntry) => void;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, ConversationTranscriptEntry[]>>>;
  readonly completeRosterAgentIdsRef: React.MutableRefObject<readonly string[]>;
  readonly hasLoadedAgentsRef: React.MutableRefObject<boolean>;
  readonly setHumanConversations: React.Dispatch<React.SetStateAction<RendererAgent[]>>;
}

export function useTdrpOpenAgent({
  client,
  accountRef,
  transcriptConversationGenerationsRef,
  accountScopeGenerationRef,
  transportScopeGenerationRef,
  selectionStore,
  setOverlay,
  setWorkspaceRoute,
  setCommandPaletteOpen,
  setPaletteMessageTarget,
  paletteMessagePagesLoadedRef,
  humanConversationsRef,
  initialTranscriptRequestRef,
  openAgentRequestGenerationRef,
  transcriptHistoryCursorsRef,
  entriesByAgentRef,
  transcriptAccountSlot,
  transcriptPaginationController,
  activeAgentIdRef,
  continueHumanHistoryCoverage,
  pendingTranscriptReadsRef,
  setTranscriptLoadError,
  agentsRef,
  transportRef,
  setNotice,
  observeTranscriptSettlement,
  setEntriesByAgent,
  completeRosterAgentIdsRef,
  hasLoadedAgentsRef,
  setHumanConversations,
}: UseTdrpOpenAgentOptions) {
  const openAgent = useCallback(async (agentId: string, options?: { background?: boolean }) => {
    const conversationGeneration = transcriptConversationGenerationsRef.current.get(agentId) ?? 0;
    const accountScopeGeneration = accountScopeGenerationRef.current;
    const transportScopeGeneration = transportScopeGenerationRef.current;
    if (accountRef.current?.kind !== "logged-in") return;
    if (options?.background === true && selectionStore.get().currentAgentId !== agentId) return;
    if (options?.background !== true) {
      setOverlay(null);
      setWorkspaceRoute(null);
      setCommandPaletteOpen(false);
      setPaletteMessageTarget((current: any) => {
        if (current == null || current.agentId === agentId) return current;
        paletteMessagePagesLoadedRef.current = 0;
        return null;
      });
    }
    const isHumanTranscript = humanConversationsRef.current.some((conversation) => conversation.id === agentId);
    if (initialTranscriptRequestRef.current?.agentId === agentId
      && initialTranscriptRequestRef.current.accountGeneration === accountScopeGeneration
      && initialTranscriptRequestRef.current.requestGeneration === openAgentRequestGenerationRef.current
      && initialTranscriptRequestRef.current.isHuman === isHumanTranscript
      && (isHumanTranscript || initialTranscriptRequestRef.current.transportGeneration === transportScopeGeneration)) return;
    const cachedCursor = transcriptHistoryCursorsRef.current.get(agentId);
    const hasLoadedEntries = entriesByAgentRef.current[agentId] != null
      && cachedCursor != null && cachedCursor.accountSlot === transcriptAccountSlot;
    const paginationScope = transcriptPaginationController.getSnapshot();
    if (activeAgentIdRef.current === agentId && hasLoadedEntries
      && paginationScope.accountSlot === transcriptAccountSlot && paginationScope.agentId === agentId) {
      setOverlay(null);
      setWorkspaceRoute(null);
      setCommandPaletteOpen(false);
      if (isHumanTranscript
        && humanConversationsRef.current.find((conversation) => conversation.id === agentId)?.raw.historyCoverageComplete === false) {
        continueHumanHistoryCoverage(agentId);
      }
      return;
    }
    const requestGeneration = ++openAgentRequestGenerationRef.current;
    pendingTranscriptReadsRef.current.clear();
    setTranscriptLoadError((current) => current?.agentId === agentId ? null : current);
    transcriptPaginationController.setScope(transcriptAccountSlot, agentId);
    selectionStore.select(agentId);
    if (hasLoadedEntries) {
      if (transcriptPaginationController.getSnapshot().cursor.kind === "unprobed") {
        transcriptPaginationController.installInitialPage({
          entries: [],
          nextBeforeSeq: cachedCursor?.nextBeforeSeq,
          nextBeforeCursor: cachedCursor?.nextBeforeCursor,
        });
      }
      selectionStore.settle(agentId);
    }
    if (hasLoadedEntries || client == null) {
      if (client == null) selectionStore.settle(agentId);
      else if (isHumanTranscript
        && humanConversationsRef.current.find((conversation) => conversation.id === agentId)?.raw.historyCoverageComplete === false) {
        continueHumanHistoryCoverage(agentId);
      }
      return;
    }
    const humanConversation = humanConversationsRef.current.find((conversation) => conversation.id === agentId);
    const agentName = humanConversation?.name ?? agentsRef.current.find((agent) => agent.id === agentId)?.name ?? UI_TEXT.title;
    const pendingRead: TranscriptReadUpdateBuffer = {
      accountSlot: transcriptAccountSlot,
      agentId,
      isCurrent: () => accountScopeGenerationRef.current === accountScopeGeneration
        && openAgentRequestGenerationRef.current === requestGeneration
        && (transcriptConversationGenerationsRef.current.get(agentId) ?? 0) === conversationGeneration
        && selectionStore.get().currentAgentId === agentId
        && (humanConversation != null || transportScopeGenerationRef.current === transportScopeGeneration)
        && accountRef.current?.kind === "logged-in",
      updates: new Map(),
    };
    initialTranscriptRequestRef.current = {
      agentId, accountGeneration: accountScopeGeneration, requestGeneration,
      transportGeneration: transportScopeGeneration, isHuman: humanConversation != null,
    };
    try {
      let page: unknown;
      if (humanConversation == null) {
        pendingTranscriptReadsRef.current.add(pendingRead);
        page = await client.call("openAgentTail", { id: agentId, limit: 200 });
      } else {
        if (transportRef.current === "connected") {
          try {
            await client.call("syncHumanConversation", { conversationId: agentId });
          } catch (error) {
            if (pendingRead.isCurrent()) setNotice(error instanceof Error ? `Human sync unavailable: ${error.message}` : `Human sync unavailable: ${String(error)}`);
          }
        }
        if (!pendingRead.isCurrent()) return;
        pendingTranscriptReadsRef.current.add(pendingRead);
        page = await client.call("getHumanConversationTranscriptTail", { conversationId: agentId, limit: 200 });
      }
      if (!pendingRead.isCurrent()) return;
      const projectedPage = projectTranscriptPageResult(page, agentName, agentId);
      const projectedEntries = applyTranscriptReadUpdates(projectedPage.entries, pendingRead.updates);
      for (const entry of projectedEntries) observeTranscriptSettlement(agentId, entry);
      setEntriesByAgent((current) => !pendingRead.isCurrent() ? current : ({
        ...current,
        [agentId]: applyTranscriptReadUpdates(reconcileLateInitialTranscriptPage(current[agentId] ?? [], projectedEntries), pendingRead.updates)
      }));
      if (transcriptAccountSlot != null) transcriptHistoryCursorsRef.current.set(agentId, {
        accountSlot: transcriptAccountSlot,
        nextBeforeSeq: projectedPage.nextBeforeSeq,
        nextBeforeCursor: projectedPage.nextBeforeCursor,
      });
      transcriptPaginationController.installInitialPage({
        entries: [],
        nextBeforeSeq: projectedPage.nextBeforeSeq,
        nextBeforeCursor: projectedPage.nextBeforeCursor,
      });
      selectionStore.settle(agentId);
      selectionStore.reconcile({ agentIds: completeRosterAgentIdsRef.current, isRosterComplete: hasLoadedAgentsRef.current });
      setTranscriptLoadError((current) => current?.agentId === agentId ? null : current);
      if (humanConversation != null) {
        const tailCoverageIncomplete = typeof page === "object" && page != null
          && (page as { historyCoverageComplete?: unknown }).historyCoverageComplete === false;
        const previousIncomplete = humanConversationsRef.current.find((candidate) => candidate.id === agentId)?.raw.historyCoverageComplete === false;
        if (tailCoverageIncomplete !== previousIncomplete) {
          const updatedHumans = humanConversationsRef.current.map((candidate) => {
            if (candidate.id !== agentId) return candidate;
            const nextRaw = { ...candidate.raw };
            if (tailCoverageIncomplete) nextRaw.historyCoverageComplete = false;
            else delete nextRaw.historyCoverageComplete;
            return { ...candidate, raw: nextRaw };
          });
          humanConversationsRef.current = updatedHumans;
          setHumanConversations(updatedHumans);
        }
        if (tailCoverageIncomplete) {
          continueHumanHistoryCoverage(agentId);
        }
      }
    } catch {
      if (!pendingRead.isCurrent()) return;
      selectionStore.settle(agentId);
      selectionStore.reconcile({ agentIds: completeRosterAgentIdsRef.current, isRosterComplete: hasLoadedAgentsRef.current });
      setTranscriptLoadError({ agentId, accountScopeGeneration });
    } finally {
      pendingTranscriptReadsRef.current.delete(pendingRead);
      if (initialTranscriptRequestRef.current?.requestGeneration === requestGeneration) initialTranscriptRequestRef.current = null;
    }
  }, [accountRef, accountScopeGenerationRef, activeAgentIdRef, client, completeRosterAgentIdsRef, continueHumanHistoryCoverage, entriesByAgentRef, hasLoadedAgentsRef, humanConversationsRef, initialTranscriptRequestRef, observeTranscriptSettlement, openAgentRequestGenerationRef, paletteMessagePagesLoadedRef, pendingTranscriptReadsRef, selectionStore, setCommandPaletteOpen, setEntriesByAgent, setHumanConversations, setNotice, setOverlay, setPaletteMessageTarget, setTranscriptLoadError, setWorkspaceRoute, transcriptAccountSlot, transcriptConversationGenerationsRef, transcriptHistoryCursorsRef, transcriptPaginationController, transportRef, transportScopeGenerationRef]);

  return { openAgent };
}
