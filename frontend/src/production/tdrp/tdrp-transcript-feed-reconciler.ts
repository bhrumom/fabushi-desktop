import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import { transcriptAttachmentIdentity } from "../../recovered/features/conversation/workspace/model";
import {
  applyTranscriptReadUpdates,
  currentTranscriptReadBuffers,
  localTranscriptClientNonce,
  recordTranscriptReadUpdate,
  reconcileForwardTargetMessage,
  reconcileTranscriptMutation,
  reconcileTranscriptTail,
  type TranscriptReadUpdateBuffer,
} from "../transcript-projection";
import {
  projectTranscriptEntry,
  projectTranscriptFeedEntries,
  type RendererAgent,
} from "../model";
import { UI_TEXT } from "../evidence";
import type { TranscriptOperationScope, TranscriptSpreadsheetSource } from "./tdrp-transcript-scope";
import type { ForwardTargetState } from "./TdrpForwardMessageModal";

export interface CreateTranscriptFeedObserversOptions {
  readonly isCurrent: () => boolean;
  readonly accountRef: React.MutableRefObject<{ readonly kind?: string; readonly authId?: string | null; readonly email?: string | null } | null>;
  readonly transcriptConversationGenerationsRef: React.MutableRefObject<Map<string, number>>;
  readonly agentsRef: React.MutableRefObject<RendererAgent[]>;
  readonly humanConversationsRef: React.MutableRefObject<RendererAgent[]>;
  readonly pendingTranscriptReadsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly pendingTranscriptReceiptsRef: React.MutableRefObject<Set<TranscriptReadUpdateBuffer>>;
  readonly transcriptAccountSlot: string | null;
  readonly spreadsheetViewerSourceRef: React.MutableRefObject<TranscriptSpreadsheetSource | null>;
  readonly spreadsheetEntriesRef: React.MutableRefObject<readonly ConversationTranscriptEntry[]>;
  readonly isTranscriptScopeCurrent: (scope: TranscriptOperationScope) => boolean;
  readonly setForwardTarget: React.Dispatch<React.SetStateAction<ForwardTargetState | null>>;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly acknowledgementController: any;
  readonly humanReactionSettlement: any;
  readonly humanMessageRetryController: any;
  readonly observeTranscriptSettlement: (agentId: string, entry: ConversationTranscriptEntry) => void;
  readonly setEntriesByAgent: React.Dispatch<React.SetStateAction<Record<string, readonly ConversationTranscriptEntry[]>>>;
  readonly entriesByAgentRef: React.MutableRefObject<Record<string, readonly ConversationTranscriptEntry[]>>;
  readonly invalidateActiveHumanSearch: (ownerId: string) => void;
  readonly transcriptPaginationController: any;
  readonly invalidateTranscriptConversation: (ownerId: string) => void;
  readonly sendJournalApprovalLifecycle: any;
  readonly transcriptHistoryCursorsRef: React.MutableRefObject<Map<string, { accountSlot: string }>>;
  readonly activeAgentIdRef: React.MutableRefObject<string>;
  readonly openAgentRequestGenerationRef: React.MutableRefObject<number>;
  readonly initialTranscriptRequestRef: React.MutableRefObject<any>;
  readonly openAgentRef: React.MutableRefObject<(agentId: string, options?: { background?: boolean }) => Promise<void>>;
}

export function createTranscriptFeedObservers({
  isCurrent,
  accountRef,
  transcriptConversationGenerationsRef,
  agentsRef,
  humanConversationsRef,
  pendingTranscriptReadsRef,
  pendingTranscriptReceiptsRef,
  transcriptAccountSlot,
  spreadsheetViewerSourceRef,
  spreadsheetEntriesRef,
  isTranscriptScopeCurrent,
  setForwardTarget,
  accountScopeGenerationRef,
  acknowledgementController,
  humanReactionSettlement,
  humanMessageRetryController,
  observeTranscriptSettlement,
  setEntriesByAgent,
  entriesByAgentRef,
  invalidateActiveHumanSearch,
  transcriptPaginationController,
  invalidateTranscriptConversation,
  sendJournalApprovalLifecycle,
  transcriptHistoryCursorsRef,
  activeAgentIdRef,
  openAgentRequestGenerationRef,
  initialTranscriptRequestRef,
  openAgentRef,
}: CreateTranscriptFeedObserversOptions) {
  const transcriptBaselineSeen = new Set<string>();
  const entriesAtFeedStart = entriesByAgentRef.current;
  const preBaselineFeedEntryIds = new Map<string, Set<string>>();

  const rememberPreBaselineFeedEntry = (ownerId: string, entryId: string) => {
    if (transcriptBaselineSeen.has(ownerId)) return;
    const ids = preBaselineFeedEntryIds.get(ownerId) ?? new Set<string>();
    ids.add(entryId);
    preBaselineFeedEntryIds.set(ownerId, ids);
  };

  const rememberTranscriptUpdate = (ownerId: string, entry: ConversationTranscriptEntry, previousId?: string) => {
    for (const reads of [pendingTranscriptReadsRef.current, pendingTranscriptReceiptsRef.current]) {
      for (const read of currentTranscriptReadBuffers(reads, transcriptAccountSlot, ownerId)) {
        recordTranscriptReadUpdate(read.updates, entry, previousId);
      }
    }
  };

  const retireChangedSpreadsheetSource = (ownerId: string, entry: ConversationTranscriptEntry, previousId?: string) => {
    const source = spreadsheetViewerSourceRef.current;
    if (source == null || source.agentId !== ownerId || !isTranscriptScopeCurrent(source)) return;
    const current = spreadsheetEntriesRef.current.find((candidate) => candidate.id === source.entryId);
    if (current == null) return;
    const isAgentAttachment = localTranscriptClientNonce(entry, undefined) != null
      && "sourceKind" in entry && entry.sourceKind === "user-attachment";
    const next = reconcileTranscriptMutation([current], entry, { previousId, preserveCurrentId: isAgentAttachment })
      .find((candidate) => candidate.id === source.entryId);
    if (next?.kind !== "message" || !next.attachments?.some((attachment) =>
      transcriptAttachmentIdentity(attachment) === source.attachmentIdentity)) source.mount.close();
  };

  const retireOrRefreshForwardTarget = (ownerId: string, entry: ConversationTranscriptEntry, previousId?: string) => {
    setForwardTarget((current) => {
      if (current == null || current.sourceConversationId !== ownerId) return current;
      if (current.accountScopeGeneration !== accountScopeGenerationRef.current) return null;
      const nextMessage = reconcileForwardTargetMessage(current.message, entry, { previousId });
      if (nextMessage == null) return null;
      return nextMessage === current.message ? current : { ...current, message: nextMessage };
    });
  };

  const resolveAgentRetryEcho = (ownerId: string, entry: ConversationTranscriptEntry): ConversationTranscriptEntry => {
    if (humanConversationsRef.current.some((conversation) => conversation.id === ownerId)) return entry;
    const nonce = localTranscriptClientNonce(entry, undefined);
    if (nonce == null || entry.kind !== "message") return entry;
    const record = acknowledgementController.getSnapshot().records.find((candidate: any) => candidate.agentId === ownerId
      && (candidate.nonce === nonce || candidate.priorNonces.includes(nonce)));
    return record == null || record.nonce === nonce ? entry : { ...entry, clientNonce: record.nonce };
  };

  return {
    onBaseline: ({ agentId: ownerId, entries: rawEntries }: { agentId: string; entries: readonly unknown[] }) => {
      if (!isCurrent() || accountRef.current?.kind !== "logged-in") return;
      const conversationGeneration = transcriptConversationGenerationsRef.current.get(ownerId) ?? 0;
      const owner = agentsRef.current.find((agent) => agent.id === ownerId);
      let projectedEntries = projectTranscriptFeedEntries(rawEntries, owner?.name ?? UI_TEXT.title, ownerId);
      for (const reads of [pendingTranscriptReadsRef.current, pendingTranscriptReceiptsRef.current]) {
        for (const read of currentTranscriptReadBuffers(reads, transcriptAccountSlot, ownerId)) {
          projectedEntries = applyTranscriptReadUpdates(projectedEntries, read.updates);
        }
      }
      for (const entry of projectedEntries) {
        retireChangedSpreadsheetSource(ownerId, entry);
        retireOrRefreshForwardTarget(ownerId, entry);
        humanReactionSettlement.observeEntry({ conversationId: ownerId, entryId: entry.id });
        humanMessageRetryController.observeEntry(ownerId, entry);
        observeTranscriptSettlement(ownerId, entry);
      }
      const preserveEntryIds = new Set(preBaselineFeedEntryIds.get(ownerId) ?? []);
      setEntriesByAgent((current) => !isCurrent()
        || (transcriptConversationGenerationsRef.current.get(ownerId) ?? 0) !== conversationGeneration ? current : ({
        ...current,
        [ownerId]: reconcileTranscriptTail(current[ownerId] ?? [], projectedEntries, preserveEntryIds, entriesAtFeedStart[ownerId] ?? [])
      }));
      transcriptBaselineSeen.add(ownerId);
      preBaselineFeedEntryIds.delete(ownerId);
      invalidateActiveHumanSearch(ownerId);
    },

    onUpdated: ({ agentId: ownerId, before, after: event }: { agentId: string; before: unknown; after: unknown }) => {
      if (!isCurrent() || accountRef.current?.kind !== "logged-in") return;
      const owner = agentsRef.current.find((agent) => agent.id === ownerId);
      const rawProjected = projectTranscriptEntry(event, entriesByAgentRef.current[ownerId]?.length ?? 0, owner?.name ?? UI_TEXT.title, ownerId);
      if (rawProjected == null || !ownerId) return;
      const projected = resolveAgentRetryEcho(ownerId, rawProjected);
      humanReactionSettlement.observeEntry({ conversationId: ownerId, entryId: projected.id });
      humanMessageRetryController.observeEntry(ownerId, projected);
      const conversationGeneration = transcriptConversationGenerationsRef.current.get(ownerId) ?? 0;
      rememberPreBaselineFeedEntry(ownerId, projected.id);
      const previousId = before != null && typeof before === "object" && "id" in before && typeof before.id === "string" ? before.id : undefined;
      if (previousId != null) {
        humanReactionSettlement.observeEntry({ conversationId: ownerId, entryId: previousId });
        rememberPreBaselineFeedEntry(ownerId, previousId);
      }
      rememberTranscriptUpdate(ownerId, projected, previousId);
      retireChangedSpreadsheetSource(ownerId, projected, previousId);
      retireOrRefreshForwardTarget(ownerId, projected, previousId);
      invalidateActiveHumanSearch(ownerId);
      observeTranscriptSettlement(ownerId, projected);
      transcriptPaginationController.ingestEntryUpdate(transcriptAccountSlot, ownerId, projected, previousId);
      setEntriesByAgent((current) => {
        if (!isCurrent() || (transcriptConversationGenerationsRef.current.get(ownerId) ?? 0) !== conversationGeneration) return current;
        const existing = current[ownerId] ?? [];
        const isAgentAttachment = localTranscriptClientNonce(projected, undefined) != null
          && "sourceKind" in projected && projected.sourceKind === "user-attachment";
        const next = reconcileTranscriptMutation(existing, projected, { previousId, preserveCurrentId: isAgentAttachment });
        return next === existing ? current : { ...current, [ownerId]: next };
      });
    },

    onCleared: (ownerId: string) => {
      if (!isCurrent() || accountRef.current?.kind !== "logged-in") return;
      humanReactionSettlement.clearConversation(ownerId);
      if (spreadsheetViewerSourceRef.current?.agentId === ownerId) spreadsheetViewerSourceRef.current.mount.close();
      setForwardTarget((current) => current?.sourceConversationId === ownerId ? null : current);
      invalidateActiveHumanSearch(ownerId);
      invalidateTranscriptConversation(ownerId);
      sendJournalApprovalLifecycle.reset();
      transcriptBaselineSeen.add(ownerId);
      preBaselineFeedEntryIds.delete(ownerId);
      transcriptHistoryCursorsRef.current.delete(ownerId);
      for (const read of pendingTranscriptReadsRef.current) {
        if (read.agentId === ownerId) pendingTranscriptReadsRef.current.delete(read);
      }
      setEntriesByAgent((current) => {
        if (!isCurrent()) return current;
        if (!(ownerId in current)) return current;
        const next = { ...current };
        delete next[ownerId];
        return next;
      });
      if (activeAgentIdRef.current !== ownerId && transcriptPaginationController.getSnapshot().agentId !== ownerId) return;
      openAgentRequestGenerationRef.current += 1;
      initialTranscriptRequestRef.current = null;
      acknowledgementController.reset();
      transcriptPaginationController.reset();
      if (activeAgentIdRef.current === ownerId) void openAgentRef.current(ownerId, { background: true });
    },

    onAppended: ({ agentId: ownerId, entry: event }: { agentId: string; entry: unknown }) => {
      if (!isCurrent() || accountRef.current?.kind !== "logged-in") return;
      const owner = agentsRef.current.find((agent) => agent.id === ownerId);
      const rawProjected = projectTranscriptEntry(event, entriesByAgentRef.current[ownerId]?.length ?? 0, owner?.name ?? UI_TEXT.title, ownerId);
      if (rawProjected == null || !ownerId) return;
      const projected = resolveAgentRetryEcho(ownerId, rawProjected);
      humanReactionSettlement.observeEntry({ conversationId: ownerId, entryId: projected.id });
      humanMessageRetryController.observeEntry(ownerId, projected);
      const conversationGeneration = transcriptConversationGenerationsRef.current.get(ownerId) ?? 0;
      rememberPreBaselineFeedEntry(ownerId, projected.id);
      rememberTranscriptUpdate(ownerId, projected);
      retireChangedSpreadsheetSource(ownerId, projected);
      retireOrRefreshForwardTarget(ownerId, projected);
      invalidateActiveHumanSearch(ownerId);
      observeTranscriptSettlement(ownerId, projected);
      transcriptPaginationController.ingestEntryUpdate(transcriptAccountSlot, ownerId, projected);
      setEntriesByAgent((current) => {
        if (!isCurrent() || (transcriptConversationGenerationsRef.current.get(ownerId) ?? 0) !== conversationGeneration) return current;
        const isAgentAttachment = localTranscriptClientNonce(projected, undefined) != null
          && "sourceKind" in projected && projected.sourceKind === "user-attachment";
        return { ...current, [ownerId]: reconcileTranscriptMutation(current[ownerId] ?? [], projected, {
          appendIfMissing: true, preserveCurrentId: isAgentAttachment,
        }) };
      });
    },
  };
}
