import type { ConversationTranscriptEntry } from "./model";
import { mergeOlderTranscriptEntries } from "../../../../production/transcript-projection.ts";
export { mergeOlderTranscriptEntries } from "../../../../production/transcript-projection.ts";

/**
 * Cursor and viewport state for the shipped transcript older-history flow.
 * The coordinator returns entries in chronological order and uses
 * `nextBeforeSeq` as the exclusive older-page cursor. Human pages also carry
 * an opaque `nextBeforeCursor` for a stable walk across message updates.
 *
 * @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5692943 (cursor/page merge helpers)
 * @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5705748 (loadOlder request fencing and error state)
 * @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5312453 (root hasOlder/isLoadingOlder/loadOlder handoff)
 */

export type TranscriptHistoryCursor =
  | { kind: "unprobed" }
  | { kind: "more"; beforeSeq: number; beforeCursor?: string }
  | { kind: "exhausted" }
  | { kind: "unavailable" };

export interface TranscriptHistoryPage {
  entries: readonly ConversationTranscriptEntry[];
  nextBeforeSeq?: number;
  nextBeforeCursor?: string;
}

export interface TranscriptHistoryRequest {
  id: string;
  limit: number;
  beforeSeq: number;
  beforeCursor?: string;
}

export type TranscriptHistoryFetcher = (request: TranscriptHistoryRequest) => Promise<TranscriptHistoryPage>;

export interface TranscriptViewport {
  scrollTop: number;
  scrollHeight: number;
}

export interface TranscriptPaginationSnapshot {
  generation: number;
  accountSlot: string | null;
  agentId: string | null;
  entries: readonly ConversationTranscriptEntry[];
  cursor: TranscriptHistoryCursor;
  hasOlder: boolean;
  isLoadingOlder: boolean;
  olderFailure: unknown | null;
}

/**
 * Typed boundary for the root owner. The root installs the authoritative
 * initial page, subscribes to this snapshot, passes its fields to the
 * transcript view, and calls onEntriesCommitted from its layout-commit path.
 */
export interface TranscriptPaginationRootHandoff {
  getSnapshot(): TranscriptPaginationSnapshot;
  subscribe(listener: () => void): () => void;
  setScope(accountSlot: string | null, agentId: string | null): void;
  installInitialPage(page: TranscriptHistoryPage): boolean;
  ingestEntryUpdate(accountSlot: string | null, agentId: string, entry: ConversationTranscriptEntry): void;
  loadOlder(): Promise<void>;
  bindViewport(viewport: TranscriptViewport | null): void;
  onEntriesCommitted(): boolean;
  reset(): void;
  dispose(): void;
}

export interface TranscriptPaginationControllerOptions {
  fetchPage: TranscriptHistoryFetcher;
  pageLimit?: number;
  accountSlot?: string | null;
  agentId?: string | null;
  /** Commit accepted older rows to the root's existing conversation cache. */
  onPageApplied?(scope: { accountSlot: string | null; agentId: string }, page: TranscriptHistoryPage): void;
}

const DEFAULT_PAGE_LIMIT = 200;

const EMPTY_CURSOR: TranscriptHistoryCursor = { kind: "unprobed" };

function cursorFromPage({ nextBeforeSeq, nextBeforeCursor }: TranscriptHistoryPage): TranscriptHistoryCursor {
  return typeof nextBeforeSeq === "number" && Number.isFinite(nextBeforeSeq)
    ? {
      kind: "more",
      beforeSeq: nextBeforeSeq,
      ...(typeof nextBeforeCursor === "string" && nextBeforeCursor.length > 0
        ? { beforeCursor: nextBeforeCursor }
        : {}),
    }
    : { kind: "exhausted" };
}

function isCapabilityUnavailableFailure(error: unknown): boolean {
  if (typeof error !== "object" || error == null) return false;
  const candidate = error as { code?: unknown; failure?: { code?: unknown } };
  return candidate.code === "source/capability-unavailable"
    || candidate.failure?.code === "source/capability-unavailable";
}

function uniqueEntries(entries: readonly ConversationTranscriptEntry[], excludedIds = new Set<string>()): ConversationTranscriptEntry[] {
  const seen = new Set(excludedIds);
  const result: ConversationTranscriptEntry[] = [];
  for (const entry of entries) {
    if (seen.has(entry.id)) continue;
    seen.add(entry.id);
    result.push(entry);
  }
  return result;
}

export function createTranscriptPaginationController(
  options: TranscriptPaginationControllerOptions
): TranscriptPaginationRootHandoff {
  const listeners = new Set<() => void>();
  const pageLimit = Number.isSafeInteger(options.pageLimit) && (options.pageLimit ?? 0) > 0
    ? options.pageLimit as number
    : DEFAULT_PAGE_LIMIT;
  let accountSlot = options.accountSlot ?? null;
  let agentId = options.agentId ?? null;
  let entries: readonly ConversationTranscriptEntry[] = [];
  let cursor: TranscriptHistoryCursor = EMPTY_CURSOR;
  let olderFailure: unknown | null = null;
  let generation = 0;
  let pageGeneration = 0;
  let requestGeneration = 0;
  let inFlight: Promise<void> | null = null;
  const updatesDuringRequest = new Map<string, ConversationTranscriptEntry>();
  let viewport: TranscriptViewport | null = null;
  let pendingAnchor: { generation: number; pageGeneration: number; scrollTop: number; scrollHeight: number; ready: boolean } | null = null;
  let disposed = false;

  const buildSnapshot = (): TranscriptPaginationSnapshot => ({
    generation,
    accountSlot,
    agentId,
    entries,
    cursor,
    hasOlder: cursor.kind === "unprobed" || cursor.kind === "more",
    isLoadingOlder: inFlight != null,
    olderFailure
  });
  let cachedSnapshot = buildSnapshot();

  const emit = () => {
    if (disposed) return;
    cachedSnapshot = buildSnapshot();
    for (const listener of [...listeners]) listener();
  };

  const isCurrentRequest = (
    requestId: number,
    requestPageGeneration: number,
    requestAccountSlot: string | null,
    requestAgentId: string | null,
  ): boolean => (
    !disposed
    && requestId === requestGeneration
    && requestPageGeneration === pageGeneration
    && requestAccountSlot === accountSlot
    && requestAgentId === agentId
  );

  const clearPageState = () => {
    entries = [];
    cursor = EMPTY_CURSOR;
    olderFailure = null;
    pendingAnchor = null;
    updatesDuringRequest.clear();
  };

  return {
    getSnapshot: () => cachedSnapshot,

    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },

    setScope(nextAccountSlot, nextAgentId) {
      if (disposed || (accountSlot === nextAccountSlot && agentId === nextAgentId)) return;
      generation += 1;
      pageGeneration += 1;
      requestGeneration += 1;
      accountSlot = nextAccountSlot;
      agentId = nextAgentId;
      inFlight = null;
      clearPageState();
      emit();
    },

    installInitialPage(page) {
      if (disposed) return false;
      pageGeneration += 1;
      requestGeneration += 1;
      inFlight = null;
      updatesDuringRequest.clear();
      entries = uniqueEntries(page.entries);
      cursor = cursorFromPage(page);
      olderFailure = null;
      pendingAnchor = null;
      emit();
      return true;
    },

    ingestEntryUpdate(updateAccountSlot, ownerId, entry) {
      if (disposed || updateAccountSlot !== accountSlot || ownerId !== agentId) return;
      // A failed page can be retried with an older snapshot. Keep the latest
      // committed overlay for that read until a successful page consumes it.
      if (inFlight != null || updatesDuringRequest.has(entry.id)) updatesDuringRequest.set(entry.id, entry);
      if (!entries.some((candidate) => candidate.id === entry.id)) return;
      entries = entries.map((candidate) => candidate.id === entry.id ? entry : candidate);
      emit();
    },

    loadOlder() {
      if (disposed || agentId == null || cursor.kind !== "more") return Promise.resolve();
      if (inFlight != null) return inFlight;

      const requestId = ++requestGeneration;
      const requestPageGeneration = pageGeneration;
      const requestAgentId = agentId;
      const requestAccountSlot = accountSlot;
      const beforeSeq = cursor.beforeSeq;
      const beforeCursor = cursor.beforeCursor;
      if (viewport != null && Number.isFinite(viewport.scrollTop) && Number.isFinite(viewport.scrollHeight)) {
        pendingAnchor = {
          generation,
          pageGeneration,
          scrollTop: viewport.scrollTop,
          scrollHeight: viewport.scrollHeight,
          ready: false
        };
      } else pendingAnchor = null;
      olderFailure = null;

      const request = (async () => {
        // Defer invocation until inFlight is assigned, including synchronous
        // fetch failures; a rejected read must remain retryable.
        await Promise.resolve();
        try {
          if (!isCurrentRequest(requestId, requestPageGeneration, requestAccountSlot, requestAgentId)) return;
          const page = await options.fetchPage({
            id: requestAgentId,
            limit: pageLimit,
            beforeSeq,
            ...(beforeCursor === undefined ? {} : { beforeCursor }),
          });
          if (!isCurrentRequest(requestId, requestPageGeneration, requestAccountSlot, requestAgentId)) return;
          const refreshedPage = page.entries.map((entry) => updatesDuringRequest.get(entry.id) ?? entry);
          entries = mergeOlderTranscriptEntries(entries, refreshedPage);
          if (pendingAnchor != null) pendingAnchor.ready = true;
          cursor = cursorFromPage(page);
          olderFailure = null;
          options.onPageApplied?.({ accountSlot: requestAccountSlot, agentId: requestAgentId }, {
            entries,
            nextBeforeSeq: page.nextBeforeSeq,
            ...(page.nextBeforeCursor === undefined ? {} : { nextBeforeCursor: page.nextBeforeCursor }),
          });
          updatesDuringRequest.clear();
          emit();
        } catch (error) {
          if (!isCurrentRequest(requestId, requestPageGeneration, requestAccountSlot, requestAgentId)) return;
          if (isCapabilityUnavailableFailure(error)) {
            cursor = { kind: "unavailable" };
            olderFailure = null;
            updatesDuringRequest.clear();
          } else olderFailure = error;
          pendingAnchor = null;
          emit();
        } finally {
          if (requestId === requestGeneration) {
            inFlight = null;
            if (!disposed) emit();
          }
        }
      })();
      inFlight = request;
      emit();
      return request;
    },

    bindViewport(nextViewport) {
      if (disposed) return;
      if (viewport !== nextViewport && pendingAnchor != null) pendingAnchor = null;
      viewport = nextViewport;
    },

    onEntriesCommitted() {
      if (disposed || pendingAnchor == null || !pendingAnchor.ready || viewport == null) return false;
      const anchor = pendingAnchor;
      pendingAnchor = null;
      if (anchor.generation !== generation || anchor.pageGeneration !== pageGeneration) return false;
      if (!Number.isFinite(viewport.scrollHeight) || !Number.isFinite(viewport.scrollTop)) return false;
      viewport.scrollTop = anchor.scrollTop + (viewport.scrollHeight - anchor.scrollHeight);
      return true;
    },

    reset() {
      if (disposed) return;
      generation += 1;
      pageGeneration += 1;
      requestGeneration += 1;
      inFlight = null;
      clearPageState();
      emit();
    },

    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      pageGeneration += 1;
      requestGeneration += 1;
      inFlight = null;
      clearPageState();
      viewport = null;
      listeners.clear();
    }
  };
}

