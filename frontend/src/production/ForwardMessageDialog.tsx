/**
 * Forward message dialog and headless forward state machine.
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`forwardMessages`
 *   and `Data::ForwardOptions`: `dropSenderNames` / `dropCaptions` normalization
 *   and per-peer topic routing).
 * - PR #189 (`origin/checkpoint/tdrp-forward-privacy-task20bf-r1`): Forward privacy
 *   option coupling (`dropCaptions` implies `dropSenderNames`), `forcedForwardInfo`
 *   guard, and topic eligibility filtering (`topicClosed` / `topicDeleted`).
 * - PR #190 (`origin/checkpoint/tdrp-nmr50-recovery-20bf9475-r1`): Account-scoped
 *   `ForwardDispatchScope` fencing and NMR-50i cumulative retry settlement
 *   (`confirmedRecipientIds` / `confirmedSettlements`) so retrying failed targets
 *   never re-sends to confirmed targets or drops them from the final settlement.
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`).
 */

import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import type { TranscriptMessage } from "../recovered/features/conversation/workspace/model";
import { CoordinatorCallError, type CoordinatorCallAdmission, type CoordinatorClient } from "./coordinator-client";
import {
  captureComposerDispatchOrigin,
  type ComposerDispatchOrigin,
  type ComposerDispatchScope,
} from "./composer-dispatch-scope";
import { OverlayDialog } from "../recovered/ui/overlay-primitives";
import { SandButton } from "../recovered/ui/sand-kit-primitives";
import {
  isForwardRecipientNavigationKey,
  isForwardSubmitShortcut,
  isForwardToggleShortcut,
  nextForwardRecipientIndex,
} from "./forward-recipient-navigation";

export type ForwardDispatchScope = ComposerDispatchScope;

export interface ForwardSourceSummary {
  readonly messageId: string;
  readonly fromPeerId: string;
  readonly authorLabel: string;
  readonly previewText: string;
  readonly hasMediaCaption?: boolean;
  readonly forwardRestricted?: boolean;
}

export interface ForwardRecipientOption {
  readonly id: string;
  readonly title: string;
  readonly subtitle?: string;
  readonly kind: "human" | "agent" | "group" | "channel";
  readonly writeForbidden?: boolean;
}

export type ForwardCommentAttributionMode =
  | "preserve-author"
  | "hide-author"
  | "hide-author-and-caption";

export interface ForwardExecutionSettlement {
  readonly toPeerId: string;
  readonly messageId: string;
  readonly commentMessageId?: string;
  readonly createdAt: string;
}

export interface ForwardDialogSnapshot {
  readonly source: ForwardSourceSummary;
  readonly query: string;
  readonly visibleRecipients: readonly ForwardRecipientOption[];
  readonly selectedRecipientIds: readonly string[];
  readonly confirmedRecipientIds: readonly string[];
  readonly comment: string;
  readonly attributionMode: ForwardCommentAttributionMode;
  readonly phase: "idle" | "submitting" | "failed" | "completed";
  readonly errorMessage: string | null;
  readonly settlements: readonly ForwardExecutionSettlement[];
}

export interface ForwardDialogModelOptions {
  readonly source: ForwardSourceSummary;
  readonly recipients: readonly ForwardRecipientOption[];
  readonly client: Pick<CoordinatorClient, "call">;
  readonly initialRecipientIds?: readonly string[];
  readonly readDispatchScope?: () => ForwardDispatchScope;
  readonly onSettled?: (settlements: readonly ForwardExecutionSettlement[]) => void;
  readonly onClosed?: () => void;
}

export interface ForwardDialogModel {
  snapshot(): ForwardDialogSnapshot;
  setQuery(query: string): ForwardDialogSnapshot;
  toggleRecipient(peerId: string): ForwardDialogSnapshot;
  setComment(comment: string): ForwardDialogSnapshot;
  setAttributionMode(mode: ForwardCommentAttributionMode): ForwardDialogSnapshot;
  submit(): Promise<readonly ForwardExecutionSettlement[]>;
  subscribe(listener: (snapshot: ForwardDialogSnapshot) => void): () => void;
}

function normalizeHeadlessQuery(value: string): string {
  return value.trim().toLowerCase();
}

function filterHeadlessRecipients(
  recipients: readonly ForwardRecipientOption[],
  query: string
): readonly ForwardRecipientOption[] {
  const normalized = normalizeHeadlessQuery(query);
  return recipients.filter((recipient) => {
    if (recipient.writeForbidden) return false;
    if (!normalized) return true;
    const haystack = `${recipient.title} ${recipient.subtitle ?? ""} ${recipient.id}`.toLowerCase();
    return haystack.includes(normalized);
  });
}

function retiredForwardError(): CoordinatorCallError {
  return new CoordinatorCallError("request-retired", "Forward dispatch retired after account scope change.");
}

function requireCurrentForwardOrigin(origin: ComposerDispatchOrigin | null): void {
  if (origin != null && !origin.isCurrent()) {
    throw retiredForwardError();
  }
}

export function createForwardDialogModel(options: ForwardDialogModelOptions): ForwardDialogModel {
  if (options.source.forwardRestricted) {
    throw new Error("Protected or restricted messages cannot be forwarded.");
  }

  const eligibleIds = new Set(
    options.recipients.filter((recipient) => !recipient.writeForbidden).map((recipient) => recipient.id)
  );
  const listeners = new Set<(snapshot: ForwardDialogSnapshot) => void>();

  let query = "";
  let selectedRecipientIds = (options.initialRecipientIds ?? []).filter((id) => eligibleIds.has(id));
  const confirmedSettlementsByPeer = new Map<string, ForwardExecutionSettlement>();
  let targetOrderIds = [...selectedRecipientIds];
  let comment = "";
  let attributionMode: ForwardCommentAttributionMode = "preserve-author";
  let phase: ForwardDialogSnapshot["phase"] = "idle";
  let errorMessage: string | null = null;
  let settlements: readonly ForwardExecutionSettlement[] = [];

  const orderedConfirmedSettlements = (): ForwardExecutionSettlement[] => {
    const ordered: ForwardExecutionSettlement[] = [];
    const seen = new Set<string>();
    for (const peerId of targetOrderIds) {
      const settlement = confirmedSettlementsByPeer.get(peerId);
      if (settlement != null) {
        ordered.push(settlement);
        seen.add(peerId);
      }
    }
    for (const [peerId, settlement] of confirmedSettlementsByPeer.entries()) {
      if (!seen.has(peerId)) {
        ordered.push(settlement);
      }
    }
    return ordered;
  };

  const buildSnapshot = (): ForwardDialogSnapshot => ({
    source: options.source,
    query,
    visibleRecipients: filterHeadlessRecipients(options.recipients, query),
    selectedRecipientIds: [...selectedRecipientIds],
    confirmedRecipientIds: [...confirmedSettlementsByPeer.keys()],
    comment,
    attributionMode,
    phase,
    errorMessage,
    settlements,
  });

  const emit = (): ForwardDialogSnapshot => {
    const next = buildSnapshot();
    for (const listener of listeners) {
      listener(next);
    }
    return next;
  };

  return {
    snapshot: buildSnapshot,
    setQuery(nextQuery) {
      query = nextQuery;
      return emit();
    },
    toggleRecipient(peerId) {
      if (!eligibleIds.has(peerId) || phase === "submitting") {
        return buildSnapshot();
      }
      if (selectedRecipientIds.includes(peerId)) {
        selectedRecipientIds = selectedRecipientIds.filter((id) => id !== peerId);
      } else {
        selectedRecipientIds = [...selectedRecipientIds, peerId];
        if (!targetOrderIds.includes(peerId)) {
          targetOrderIds = [...targetOrderIds, peerId];
        }
      }
      errorMessage = null;
      if (phase === "failed") phase = "idle";
      return emit();
    },
    setComment(nextComment) {
      if (phase === "submitting") return buildSnapshot();
      comment = nextComment;
      return emit();
    },
    setAttributionMode(nextMode) {
      if (phase === "submitting") return buildSnapshot();
      attributionMode = nextMode;
      return emit();
    },
    async submit() {
      if (phase === "submitting") return settlements;
      const pendingTargets = selectedRecipientIds.filter((peerId) => !confirmedSettlementsByPeer.has(peerId));
      if (pendingTargets.length === 0) {
        errorMessage = "Select at least one recipient.";
        phase = "failed";
        emit();
        return [];
      }

      for (const peerId of pendingTargets) {
        if (!targetOrderIds.includes(peerId)) {
          targetOrderIds = [...targetOrderIds, peerId];
        }
      }

      const dispatchOrigin = options.readDispatchScope
        ? captureComposerDispatchOrigin(options.readDispatchScope)
        : null;
      const callScoped = <T = unknown>(method: string, params: Record<string, unknown>): Promise<T> => {
        requireCurrentForwardOrigin(dispatchOrigin);
        return options.client.call<T>(
          method,
          params,
          dispatchOrigin
            ? {
                isCurrent: dispatchOrigin.isCurrent,
                isDispatchCurrent: dispatchOrigin.isCurrent,
              }
            : undefined
        );
      };

      phase = "submitting";
      errorMessage = null;
      emit();

      const trimmedComment = comment.trim();
      const dropAuthor = attributionMode !== "preserve-author";
      const dropCaption = attributionMode === "hide-author-and-caption";

      try {
        for (const toPeerId of pendingTargets) {
          requireCurrentForwardOrigin(dispatchOrigin);
          let commentMessageId: string | undefined;
          if (trimmedComment.length > 0) {
            const commentResponse = await callScoped<{ message_id?: string }>("messages.send", {
              peer_id: toPeerId,
              text: trimmedComment,
              client_nonce: `fwd-comment:${options.source.messageId}:${toPeerId}`,
              attachments: [],
            });
            requireCurrentForwardOrigin(dispatchOrigin);
            commentMessageId =
              typeof commentResponse?.message_id === "string" ? commentResponse.message_id : undefined;
          }

          const forwardResponse = await callScoped<{
            message?: { id?: string; peer_id?: string; text?: string; created_at?: string };
          }>("messages.forward", {
            from_peer_id: options.source.fromPeerId,
            to_peer_id: toPeerId,
            message_id: options.source.messageId,
            drop_author: dropAuthor,
            drop_media_captions: dropCaption,
          });
          requireCurrentForwardOrigin(dispatchOrigin);

          const forwardedId =
            typeof forwardResponse?.message?.id === "string" && forwardResponse.message.id.length > 0
              ? forwardResponse.message.id
              : `fwd:${options.source.messageId}:${toPeerId}`;
          const createdAt =
            typeof forwardResponse?.message?.created_at === "string" &&
            forwardResponse.message.created_at.length > 0
              ? forwardResponse.message.created_at
              : new Date().toISOString();

          confirmedSettlementsByPeer.set(toPeerId, {
            toPeerId,
            messageId: forwardedId,
            ...(commentMessageId ? { commentMessageId } : {}),
            createdAt,
          });
          selectedRecipientIds = selectedRecipientIds.filter((id) => id !== toPeerId);
        }

        requireCurrentForwardOrigin(dispatchOrigin);
        settlements = orderedConfirmedSettlements();
        phase = "completed";
        errorMessage = null;
        emit();
        options.onSettled?.(settlements);
        options.onClosed?.();
        return settlements;
      } catch (error) {
        if (dispatchOrigin != null && !dispatchOrigin.isCurrent()) {
          phase = "failed";
          errorMessage = "Forward cancelled because the active account changed.";
          emit();
          return [];
        }
        settlements = orderedConfirmedSettlements();
        phase = "failed";
        errorMessage = error instanceof Error ? error.message : String(error ?? "Forward failed");
        emit();
        return settlements;
      }
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

export interface ForwardRecipient {
  readonly id: string;
  readonly name: string;
  readonly topicId?: string;
  readonly topicTitle?: string;
  readonly topicClosed?: boolean;
  readonly topicDeleted?: boolean;
}

export interface ForwardSettlement {
  readonly conversationId: string;
  readonly status: "sent" | "failed";
  readonly error?: string;
}

export interface ForwardMessageDialogProps {
  readonly open: boolean;
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly message: TranscriptMessage | null;
  readonly sourceConversationId: string | null;
  readonly onClose: () => void;
  readonly searchRecipients: (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    query: string;
    limit: number;
  }) => Promise<readonly ForwardRecipient[]>;
  readonly forwardMessage?: (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    destinationConversationIds: readonly string[];
    clientNonce: string;
    dropSenderNames: boolean;
    dropCaptions: boolean;
    comment?: { text: string; entities?: readonly unknown[] };
    destinationTopics?: Readonly<Record<string, string>>;
    shareUrl?: string;
  }, admission?: CoordinatorCallAdmission & (() => boolean)) => Promise<readonly ForwardSettlement[]>;
  readonly onForward?: (payload: {
    sourceConversationId: string;
    sourceEntryId: string;
    destinationConversationIds: readonly string[];
    clientNonce: string;
    dropSenderNames: boolean;
    dropCaptions: boolean;
    comment?: { text: string; entities?: readonly unknown[] };
    destinationTopics?: Readonly<Record<string, string>>;
    shareUrl?: string;
  }) => void | Promise<unknown>;
  readonly shareUrl?: string;
  readonly onSettled?: (
    settlement: readonly ForwardSettlement[],
    accountScopeGeneration: number,
    recipientNames: ReadonlyMap<string, string>,
  ) => void;
}

interface ForwardDialogLifetime {
  readonly accountScopeGeneration: number;
  readonly sourceConversationId: string;
  readonly sourceEntryId: string;
  readonly requestId: string;
  readonly recipientNames: Map<string, string>;
  readonly recipientTopics: Map<string, string>;
}

type ForwardRequest = Parameters<NonNullable<ForwardMessageDialogProps["forwardMessage"]>>[0];

interface ForwardDialogIntentSnapshot {
  readonly request: ForwardRequest;
  readonly recipientNames: ReadonlyMap<string, string>;
  readonly confirmedSettlements: Map<string, ForwardSettlement>;
}

// Match the existing Session request bound before freezing a retry intent.
// The Host still rechecks the bound and each destination's authorization.
const MAX_FORWARD_DESTINATIONS = 32;

function createForwardRequestId(sourceConversationId: string, sourceEntryId: string): string {
  const suffix = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `forward:${sourceConversationId}:${sourceEntryId}:${suffix}`;
}

export function ForwardMessageDialog({
  open,
  accountScopeGeneration,
  isAccountScopeCurrent,
  message,
  sourceConversationId,
  onClose,
  searchRecipients,
  forwardMessage,
  onForward,
  shareUrl,
  onSettled,
}: ForwardMessageDialogProps) {
  const searchRef = useRef<HTMLInputElement | null>(null);
  const recipientListRef = useRef<HTMLDivElement | null>(null);
  const searchGenerationRef = useRef(0);
  const lifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const submittingLifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const snapshotRef = useRef<ForwardDialogIntentSnapshot | null>(null);
  const selectedRef = useRef<ReadonlySet<string>>(new Set());
  const [query, setQuery] = useState("");
  const [recipients, setRecipients] = useState<readonly ForwardRecipient[]>([]);
  const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
  const [dropSenderNames, setDropSenderNames] = useState(false);
  const [dropCaptions, setDropCaptions] = useState(false);
  const [searching, setSearching] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [snapshotLocked, setSnapshotLocked] = useState(false);
  const [searchFailure, setSearchFailure] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [activeRecipientIndex, setActiveRecipientIndex] = useState(0);
  const [commentText, setCommentText] = useState("");

  const sourceEntryId = message?.id ?? null;
  const forcePreserveForwardInfo = message?.forcedForwardInfo === true;
  const effectiveDropCaptions = !forcePreserveForwardInfo && dropCaptions;
  const effectiveDropSenderNames = !forcePreserveForwardInfo && (dropSenderNames || effectiveDropCaptions);
  const scopeKey = open && sourceEntryId != null && sourceConversationId != null
    ? JSON.stringify([accountScopeGeneration, sourceConversationId, sourceEntryId])
    : null;

  const isCurrent = (lifetime: ForwardDialogLifetime): boolean =>
    lifetimeRef.current === lifetime && isAccountScopeCurrent(lifetime.accountScopeGeneration);
  const canInteract = (): boolean => {
    const lifetime = lifetimeRef.current;
    return lifetime != null && isCurrent(lifetime) && submittingLifetimeRef.current !== lifetime;
  };
  const canEditIntent = (): boolean => canInteract() && snapshotRef.current == null;
  const updateSelected = (next: ReadonlySet<string>) => {
    selectedRef.current = next;
    setSelected(next);
  };

  // A committed close/source/account change invalidates both asynchronous
  // lanes before paint. Object-only message refreshes preserve the same intent.
  useLayoutEffect(() => {
    lifetimeRef.current = null;
    submittingLifetimeRef.current = null;
    snapshotRef.current = null;
    searchGenerationRef.current += 1;
    setQuery("");
    setRecipients([]);
    updateSelected(new Set());
    setDropSenderNames(false);
    setDropCaptions(false);
    setSearching(false);
    setSubmitting(false);
    setSnapshotLocked(false);
    setSearchFailure(null);
    setFailure(null);
    setActiveRecipientIndex(0);
    setCommentText("");
    if (scopeKey == null || sourceEntryId == null || sourceConversationId == null) return;
    const lifetime: ForwardDialogLifetime = {
      accountScopeGeneration,
      sourceConversationId,
      sourceEntryId,
      requestId: createForwardRequestId(sourceConversationId, sourceEntryId),
      recipientNames: new Map(),
      recipientTopics: new Map(),
    };
    lifetimeRef.current = lifetime;
    return () => {
      lifetime.recipientNames.clear();
      lifetime.recipientTopics.clear();
      if (lifetimeRef.current === lifetime) {
        lifetimeRef.current = null;
        snapshotRef.current = null;
      }
      if (submittingLifetimeRef.current === lifetime) submittingLifetimeRef.current = null;
      searchGenerationRef.current += 1;
    };
  }, [accountScopeGeneration, scopeKey, sourceConversationId, sourceEntryId]);

  useLayoutEffect(() => {
    if (!forcePreserveForwardInfo || snapshotRef.current != null) return;
    if (dropSenderNames) setDropSenderNames(false);
    if (dropCaptions) setDropCaptions(false);
  }, [dropCaptions, dropSenderNames, forcePreserveForwardInfo]);

  useEffect(() => {
    const lifetime = lifetimeRef.current;
    if (lifetime == null || !isCurrent(lifetime)) return;
    const generation = ++searchGenerationRef.current;
    const isSearchCurrent = () => isCurrent(lifetime) && generation === searchGenerationRef.current;
    const timer = window.setTimeout(() => {
      if (!isSearchCurrent()) return;
      setSearching(true);
      void searchRecipients({
        sourceConversationId: lifetime.sourceConversationId,
        sourceEntryId: lifetime.sourceEntryId,
        query,
        limit: 50,
      })
        .then((rawNext) => {
          if (!isSearchCurrent()) return;
          // Destroyed / Closed Child Fencing (TDRP-R9-SHARE-THREAD-DESTINATION-001 & TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001)
          if (snapshotRef.current == null) {
            const unreachableIds = new Set<string>();
            for (const r of rawNext) {
              if (r.topicDeleted === true || r.topicClosed === true) {
                unreachableIds.add(r.id);
              }
            }
            if (unreachableIds.size > 0) {
              const hasUnreachable = [...unreachableIds].some((id) => selectedRef.current.has(id));
              if (hasUnreachable) {
                const nextSelected = new Set(selectedRef.current);
                for (const id of unreachableIds) {
                  nextSelected.delete(id);
                }
                updateSelected(nextSelected);
              }
            }
          }
          const next = rawNext.filter((r) => r.topicDeleted !== true && r.topicClosed !== true);
          // Search visibility may change without removing a selected recipient.
          for (const id of lifetime.recipientNames.keys()) {
            if (!selectedRef.current.has(id)) lifetime.recipientNames.delete(id);
          }
          for (const id of lifetime.recipientTopics.keys()) {
            if (!selectedRef.current.has(id)) lifetime.recipientTopics.delete(id);
          }
          for (const recipient of next) {
            lifetime.recipientNames.set(recipient.id, recipient.name);
            const trimmedTopicId = recipient.topicId?.trim();
            if (trimmedTopicId != null && trimmedTopicId.length > 0) {
              lifetime.recipientTopics.set(recipient.id, trimmedTopicId);
            } else {
              lifetime.recipientTopics.delete(recipient.id);
            }
          }
          setRecipients(next);
          setActiveRecipientIndex((current) => next.length === 0 ? 0 : Math.min(current, next.length - 1));
          setSearchFailure(null);
        })
        .catch((error: unknown) => {
          if (!isSearchCurrent()) return;
          setRecipients([]);
          setSearchFailure(error instanceof Error ? error.message : String(error));
        })
        .finally(() => {
          if (isSearchCurrent()) setSearching(false);
        });
    }, query.length === 0 ? 0 : 150);
    return () => {
      window.clearTimeout(timer);
      if (generation === searchGenerationRef.current) searchGenerationRef.current += 1;
    };
  }, [query, scopeKey, searchRecipients]);

  const focusActiveRecipient = (index: number) => {
    const lifetime = lifetimeRef.current;
    window.requestAnimationFrame(() => {
      if (lifetime == null || !isCurrent(lifetime)) return;
      recipientListRef.current
        ?.querySelector<HTMLElement>(`[data-forward-recipient-index="${index}"]`)
        ?.scrollIntoView({ block: "nearest" });
    });
  };

  const handleSearchKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (!canInteract()) return;
    if (isForwardSubmitShortcut(event)) {
      event.preventDefault();
      void submit();
      return;
    }
    if (recipients.length === 0) return;
    if (
      event.key === "ArrowDown"
      || event.key === "ArrowUp"
      || event.key === "PageDown"
      || event.key === "PageUp"
    ) {
      event.preventDefault();
      const next = nextForwardRecipientIndex(activeRecipientIndex, recipients.length, event.key);
      if (next != null) {
        setActiveRecipientIndex(next);
        focusActiveRecipient(next);
      }
      return;
    }
    if (event.key === "Enter" && !event.ctrlKey && !event.metaKey) {
      event.preventDefault();
      const recipient = recipients[activeRecipientIndex];
      if (recipient != null) toggleRecipient(recipient.id);
    }
  };

  const handleRecipientKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!canInteract() || recipients.length === 0) return;
    if (isForwardSubmitShortcut(event)) {
      event.preventDefault();
      void submit();
      return;
    }
    if (isForwardRecipientNavigationKey(event.key)) {
      event.preventDefault();
      const next = nextForwardRecipientIndex(activeRecipientIndex, recipients.length, event.key);
      if (next != null) {
        setActiveRecipientIndex(next);
        focusActiveRecipient(next);
      }
      return;
    }
    if (isForwardToggleShortcut(event)) {
      event.preventDefault();
      const recipient = recipients[activeRecipientIndex];
      if (recipient != null) toggleRecipient(recipient.id);
    }
  };

  const toggleRecipient = (id: string) => {
    if (!canEditIntent()) return;
    const next = new Set(selectedRef.current);
    if (next.has(id)) next.delete(id);
    else if (next.size < MAX_FORWARD_DESTINATIONS) next.add(id);
    else {
      setFailure(`Select up to ${MAX_FORWARD_DESTINATIONS} conversations. Remove one before adding another.`);
      return;
    }
    updateSelected(next);
    setFailure(null);
  };

  const submit = async () => {
    const lifetime = lifetimeRef.current;
    if (
      lifetime == null
      || !isCurrent(lifetime)
      || submittingLifetimeRef.current === lifetime
      || selectedRef.current.size === 0
    ) return;
    if (selectedRef.current.size > MAX_FORWARD_DESTINATIONS) {
      setFailure(`Select up to ${MAX_FORWARD_DESTINATIONS} conversations before forwarding.`);
      return;
    }
    // One accepted nonce identifies one immutable intent. The Host's durable
    // journal allows retries to narrow to unconfirmed destinations, never to
    // broaden the original set or change its normalized privacy options.
    let snapshot = snapshotRef.current;
    if (snapshot == null) {
      const comment = commentText.trim().length > 0
        ? { text: commentText, entities: [] }
        : undefined;
      const destinationTopics: Record<string, string> = {};
      for (const id of selectedRef.current) {
        const topicId = lifetime.recipientTopics.get(id);
        if (topicId != null && topicId.trim().length > 0) {
          destinationTopics[id] = topicId.trim();
        }
      }
      const hasDestinationTopics = Object.keys(destinationTopics).length > 0;
      const forwardShareUrl = typeof shareUrl === "string" && shareUrl.trim().length > 0
        ? shareUrl.trim()
        : undefined;
      const request: ForwardRequest = {
        sourceConversationId: lifetime.sourceConversationId,
        sourceEntryId: lifetime.sourceEntryId,
        destinationConversationIds: [...selectedRef.current],
        clientNonce: lifetime.requestId,
        dropSenderNames: effectiveDropSenderNames,
        dropCaptions: effectiveDropCaptions,
        ...(comment ? { comment } : {}),
        ...(hasDestinationTopics ? { destinationTopics } : {}),
        ...(forwardShareUrl ? { shareUrl: forwardShareUrl } : {}),
      };
      const recipientNames = new Map<string, string>();
      for (const id of request.destinationConversationIds) {
        const name = lifetime.recipientNames.get(id);
        if (name != null) recipientNames.set(id, name);
      }
      snapshot = { request, recipientNames, confirmedSettlements: new Map() };
      snapshotRef.current = snapshot;
    }
    const destinationConversationIds = snapshot.request.destinationConversationIds
      .filter((id) => selectedRef.current.has(id) && !snapshot.confirmedSettlements.has(id));
    let retryDestinationTopics: Record<string, string> | undefined;
    if (snapshot.request.destinationTopics != null) {
      const filtered: Record<string, string> = {};
      for (const [id, topicId] of Object.entries(snapshot.request.destinationTopics)) {
        if (destinationConversationIds.includes(id)) {
          filtered[id] = topicId;
        }
      }
      retryDestinationTopics = filtered;
    }
    // React may batch two submit events before repainting `submitting`.
    submittingLifetimeRef.current = lifetime;
    setSnapshotLocked(true);
    setSubmitting(true);
    setFailure(null);
    try {
      const forwardShareUrl = typeof shareUrl === "string" && shareUrl.trim().length > 0
        ? shareUrl.trim()
        : undefined;
      const forwardPayload = {
        ...snapshot.request,
        destinationConversationIds,
        ...(retryDestinationTopics != null ? { destinationTopics: retryDestinationTopics } : {}),
        ...(forwardShareUrl ? { shareUrl: forwardShareUrl } : {}),
      };
      if (typeof onForward === "function") {
        try {
          await onForward(forwardPayload);
        } catch (error) {
          if (forwardMessage == null) throw error;
        }
      }
      // Readiness can outlive this dialog. Recheck the same lifetime at the
      // request frame; after posting, the Host still owns the durable intent.
      const callAdmission: CoordinatorCallAdmission & (() => boolean) & { readonly isDispatchCurrent: () => boolean } = Object.assign(
        () => isCurrent(lifetime),
        {
          isCurrent: () => isCurrent(lifetime),
          isDispatchCurrent: () => isCurrent(lifetime),
        },
      );
      const settlement = forwardMessage != null
        ? await forwardMessage(forwardPayload, callAdmission)
        : destinationConversationIds.map((conversationId): ForwardSettlement => ({
          conversationId,
          status: "sent",
        }));
      if (!isCurrent(lifetime)) return;
      const hasUnexpectedSettlement = settlement.some((item) => !destinationConversationIds.includes(item.conversationId));
      const completed = destinationConversationIds.map((conversationId): ForwardSettlement => {
        const results = settlement.filter((item) => item.conversationId === conversationId);
        return !hasUnexpectedSettlement && results.length === 1 && results[0] != null
          ? results[0]
          : { conversationId, status: "failed", error: "Delivery was not confirmed. Retry keeps your forwarding choices." };
      });
      for (const item of completed) {
        if (item.status === "sent") {
          snapshot.confirmedSettlements.set(item.conversationId, item);
        }
      }
      const failed = completed.filter((item) => item.status !== "sent");
      updateSelected(new Set(failed.map((item) => item.conversationId)));
      // A consumer receives only this attempt's targets and cannot mutate the
      // names frozen with the original request or the lifetime search cache.
      const recipientNames = new Map<string, string>();
      for (const { conversationId } of completed) {
        const name = snapshot.recipientNames.get(conversationId);
        if (name != null) recipientNames.set(conversationId, name);
      }
      onSettled?.(completed, lifetime.accountScopeGeneration, recipientNames);
      if (!isCurrent(lifetime)) return;
      if (failed.length === 0) {
        closeLifetime(lifetime);
        return;
      }
      setFailure(
        failed.length === 1
          ? (failed[0]?.error ?? "One destination could not be forwarded.")
          : `${failed.length} destinations could not be forwarded. Retry sends only to these destinations.`,
      );
    } catch (error) {
      if (isCurrent(lifetime)) setFailure(error instanceof Error ? error.message : String(error));
    } finally {
      if (isCurrent(lifetime) && submittingLifetimeRef.current === lifetime) {
        submittingLifetimeRef.current = null;
        setSubmitting(false);
      }
    }
  };

  const closeLifetime = (lifetime: ForwardDialogLifetime) => {
    if (!isCurrent(lifetime)) return;
    lifetime.recipientNames.clear();
    lifetime.recipientTopics.clear();
    lifetimeRef.current = null;
    submittingLifetimeRef.current = null;
    snapshotRef.current = null;
    searchGenerationRef.current += 1;
    onClose();
  };
  const close = () => {
    const lifetime = lifetimeRef.current;
    // Closing retires presentation callbacks; an already accepted Host intent
    // remains durable and may settle after this dialog has gone away.
    if (lifetime != null) closeLifetime(lifetime);
  };

  return <OverlayDialog
    className="fabushi-forward-dialog"
    closeOnBackdrop={selected.size === 0 && !snapshotLocked && !submitting}
    initialFocusRef={searchRef}
    label="Forward message"
    onClose={close}
    open={open}
  >
    <form onSubmit={(event) => { event.preventDefault(); void submit(); }}>
      <header>
        <h2>Forward message</h2>
        <p>Select one or more conversations. Send eligibility is checked before results are exposed and again at send time.</p>
      </header>
      <div className="fabushi-forward-dialog__body">
        <label className="fabushi-forward-dialog__search">
          <span>Recipients</span>
          <input
            aria-activedescendant={recipients.length === 0 ? undefined : `fabushi-forward-recipient-${activeRecipientIndex}`}
            aria-busy={searching || undefined}
            aria-controls="fabushi-forward-recipient-list"
            autoComplete="off"
            onChange={(event) => {
              const lifetime = lifetimeRef.current;
              if (lifetime != null && isCurrent(lifetime)) setQuery(event.currentTarget.value);
            }}
            onKeyDown={handleSearchKeyDown}
            placeholder="Search conversations"
            ref={searchRef}
            type="search"
            value={query}
          />
        </label>
        <div
          aria-activedescendant={recipients.length === 0 ? undefined : `fabushi-forward-recipient-${activeRecipientIndex}`}
          aria-busy={searching || undefined}
          aria-label="Forward recipients"
          aria-multiselectable="true"
          className="fabushi-forward-dialog__recipients"
          id="fabushi-forward-recipient-list"
          onKeyDown={handleRecipientKeyDown}
          ref={recipientListRef}
          role="listbox"
          tabIndex={0}
        >
          {recipients.length === 0 && !searching
            ? <p role="status">{query.length === 0 ? "No eligible conversations." : "No eligible conversations match this search."}</p>
            : recipients.map((recipient, index) => <label
              aria-selected={selected.has(recipient.id)}
              className="fabushi-forward-dialog__recipient"
              data-active={activeRecipientIndex === index ? "true" : undefined}
              data-forward-recipient-index={index}
              id={`fabushi-forward-recipient-${index}`}
              key={recipient.id}
              onMouseEnter={() => { if (canInteract()) setActiveRecipientIndex(index); }}
              role="option"
            >
              <input
                checked={selected.has(recipient.id)}
                disabled={snapshotLocked}
                onChange={() => toggleRecipient(recipient.id)}
                tabIndex={-1}
                type="checkbox"
              />
              <span>{recipient.topicTitle != null && recipient.topicTitle.trim().length > 0 ? `${recipient.name} › ${recipient.topicTitle.trim()}` : recipient.name}</span>
            </label>)}
        </div>
        <fieldset className="fabushi-forward-dialog__privacy">
          <legend>Forward options</legend>
          <label>
            <input
              checked={effectiveDropSenderNames}
              disabled={snapshotLocked || forcePreserveForwardInfo}
              onChange={(event) => {
                if (!canEditIntent() || forcePreserveForwardInfo) return;
                const checked = event.currentTarget.checked;
                setDropSenderNames(checked);
                if (!checked) setDropCaptions(false);
              }}
              type="checkbox"
            />
            <span>Hide sender name</span>
          </label>
          <label>
            <input
              checked={effectiveDropCaptions}
              disabled={snapshotLocked || forcePreserveForwardInfo}
              onChange={(event) => {
                if (!canEditIntent() || forcePreserveForwardInfo) return;
                const checked = event.currentTarget.checked;
                setDropCaptions(checked);
                if (checked) setDropSenderNames(true);
              }}
              type="checkbox"
            />
            <span>Hide media caption</span>
          </label>
        </fieldset>
        <label className="fabushi-forward-dialog__comment">
          <span>Add a comment</span>
          <input
            autoComplete="off"
            disabled={snapshotLocked}
            onChange={(event) => {
              if (canEditIntent()) setCommentText(event.currentTarget.value);
            }}
            placeholder="Add a comment…"
            type="text"
            value={commentText}
          />
        </label>
        {failure == null && searchFailure == null ? null : <p className="fabushi-forward-dialog__error" role="alert">{failure ?? searchFailure}</p>}
        {snapshotLocked ? <p>Retry keeps your forwarding choices and skips confirmed recipients. Messages already being sent may still arrive after you close.</p> : null}
      </div>
      <footer>
        <span aria-live="polite">{selected.size === 0 ? "No recipients selected" : `${selected.size} selected`}</span>
        <SandButton onClick={close} size="sm" variant="secondary">{snapshotLocked ? "Close" : "Cancel"}</SandButton>
        <SandButton disabled={selected.size === 0} pending={submitting} size="sm" type="submit">
          {submitting ? "Forwarding…" : "Forward"}
        </SandButton>
      </footer>
    </form>
  </OverlayDialog>;
}
