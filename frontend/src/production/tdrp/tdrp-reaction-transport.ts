import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import {
  currentTranscriptReadBuffers,
  mergeAuthoritativeTranscriptMessage,
  recordTranscriptReadUpdate,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import { projectTranscriptEntry, type RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { createHumanMessageRetryController } from "../human-message-retry";

export interface CreateHumanReactionTransportOptions {
  readonly activeIsHuman: boolean;
  readonly client: ProductionCoordinatorClient | null;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly humanReactionSettlement: any;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly transcriptPaginationController: any;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
}

export function createHumanReactionTransport({
  activeIsHuman,
  client,
  accountRef,
  humanReactionSettlement,
  humanConversationsRef,
  transcriptPaginationController,
  setEntriesByAgent,
  setNotice,
}: CreateHumanReactionTransportOptions) {
  if (!activeIsHuman || client == null) return undefined;
  return {
    async reactToMessage(input: { entryId: string; emoji: string; agentId: string }) {
      const currentAccount = accountRef.current;
      if (currentAccount?.kind !== "logged-in") return;
      const accountSlot = currentAccount.authId ?? currentAccount.email ?? "account";
      await humanReactionSettlement.run(
        { conversationId: input.agentId, entryId: input.entryId },
        async () => {
          const conversation = humanConversationsRef.current.find((candidate) => candidate.id === input.agentId);
          if (conversation == null) throw new Error("Human reaction conversation is unavailable");
          const value = await client.call("reactHumanMessage", {
            conversationId: input.agentId,
            entryId: input.entryId,
            emoji: input.emoji,
          });
          const projected = projectTranscriptEntry(value, 0, conversation.name, conversation.id);
          if (projected == null || projected.id !== input.entryId) throw new Error("Human reaction settlement returned an invalid transcript entry");
          return projected;
        },
        (projected: ConversationTranscriptEntry, isCurrent: () => boolean) => {
          transcriptPaginationController.ingestEntryUpdate(accountSlot, input.agentId, projected);
          setEntriesByAgent((current) => {
            if (!isCurrent()) return current;
            const entries = current[input.agentId];
            if (entries == null || !entries.some((entry) => entry.id === input.entryId)) return current;
            return {
              ...current,
              [input.agentId]: entries.map((entry) =>
                entry.id !== input.entryId
                  ? entry
                  : entry.kind === "message" && projected.kind === "message"
                  ? mergeAuthoritativeTranscriptMessage(entry, projected)
                  : projected
              ),
            };
          });
        },
        (error: unknown, isCurrent: () => boolean) =>
          setNotice((current) =>
            isCurrent()
              ? error instanceof Error
                ? `Human reaction unavailable: ${error.message}`
                : `Human reaction unavailable: ${String(error)}`
              : current
          )
      );
    },
  };
}

export interface CreateProductionHumanMessageRetryControllerOptions {
  readonly client: ProductionCoordinatorClient | null;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly accountIdentityRef: React.MutableRefObject<string | null>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly openAgentRequestGenerationRef: React.MutableRefObject<number>;
  readonly selectionStore: any;
  readonly acknowledgementController: any;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly setNotice: React.Dispatch<React.SetStateAction<string | null>>;
  readonly pendingTranscriptReadsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly pendingTranscriptReceiptsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly observeTranscriptSettlement: (agentId: string, entry: ConversationTranscriptEntry) => void;
  readonly transcriptPaginationController: any;
}

export function createProductionHumanMessageRetryController({
  client,
  accountRef,
  accountIdentityRef,
  accountScopeGenerationRef,
  openAgentRequestGenerationRef,
  selectionStore,
  acknowledgementController,
  setEntriesByAgent,
  setNotice,
  pendingTranscriptReadsRef,
  pendingTranscriptReceiptsRef,
  observeTranscriptSettlement,
  transcriptPaginationController,
}: CreateProductionHumanMessageRetryControllerOptions) {
  return createHumanMessageRetryController({
    client,
    readScope: () => {
      const status = accountRef.current;
      if (status?.kind !== "logged-in") return null;
      const accountSlot = status.authId ?? status.email ?? "account";
      if (accountIdentityRef.current !== `logged-in:${accountSlot}`) return null;
      return {
        accountGeneration: accountScopeGenerationRef.current,
        selectionGeneration: openAgentRequestGenerationRef.current,
        accountSlot,
        selection: selectionStore.get(),
      };
    },
    acknowledgement: acknowledgementController,
    setEntries: setEntriesByAgent,
    setNotice,
    onAuthoritativeEntry: (scope: any, conversationId: string, entry: ConversationTranscriptEntry, isCurrent: () => boolean) => {
      if (!isCurrent()) return;
      for (const reads of [pendingTranscriptReadsRef.current, pendingTranscriptReceiptsRef.current]) {
        for (const read of currentTranscriptReadBuffers(reads, scope.accountSlot, conversationId)) {
          recordTranscriptReadUpdate(read.updates, entry);
        }
      }
      if (!isCurrent()) return;
      observeTranscriptSettlement(conversationId, entry);
      transcriptPaginationController.ingestEntryUpdate(scope.accountSlot, conversationId, entry);
    },
  });
}
