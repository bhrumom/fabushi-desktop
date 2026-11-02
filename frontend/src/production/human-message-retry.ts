import type { TranscriptAcknowledgementController } from "../recovered/features/conversation/workspace/acknowledgement";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../recovered/features/conversation/workspace/model";
import type { RosterSelectionState } from "../recovered/features/roster/selection-state";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import { isRecord, projectTranscriptEntry } from "./model";
import { reconcileTranscriptTail } from "./transcript-projection";

/** Captured from the existing account and roster-selection owners. */
export interface HumanMessageRetryScope {
  readonly accountGeneration: number;
  readonly selectionGeneration: number;
  readonly accountSlot: string | null;
  readonly selection: RosterSelectionState;
}

type TranscriptRows = Record<string, ConversationTranscriptEntry[]>;
type StateUpdater<T> = (update: (current: T) => T) => void;

interface HumanMessageRetryInput {
  readonly scope: HumanMessageRetryScope;
  readonly conversationId: string;
  readonly conversationName: string;
  readonly entry: TranscriptMessage;
}

interface HumanMessageRetryOptions {
  readonly client: Pick<ProductionCoordinatorClient, "call"> | null;
  readonly readScope: () => HumanMessageRetryScope | null;
  readonly acknowledgement: Pick<TranscriptAcknowledgementController, "reconcileEcho">;
  readonly setEntries: StateUpdater<TranscriptRows>;
  readonly setNotice: StateUpdater<string | null>;
  readonly onAuthoritativeEntry?: (
    scope: HumanMessageRetryScope,
    conversationId: string,
    entry: TranscriptMessage,
    isCurrent: () => boolean,
  ) => void;
}

interface RetryTarget {
  readonly conversationId: string;
  readonly clientNonce: string;
  readonly authorId?: string;
}

interface RetryAttempt {
  readonly scope: HumanMessageRetryScope;
  readonly target: RetryTarget;
  pending: boolean;
  invalidated: boolean;
  phase: "pending" | "settled" | "failed";
  completion: Promise<void>;
}

export interface HumanMessageRetryController {
  retry(input: HumanMessageRetryInput): Promise<void>;
  observeEntry(conversationId: string, entry: ConversationTranscriptEntry): void;
  dispose(): void;
}

function identity(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.trim() === value;
}

function nonce(value: unknown): value is string {
  return identity(value) && new TextEncoder().encode(value).byteLength <= 200;
}

function remoteMessageId(value: unknown): string | null {
  if (typeof value !== "string" && typeof value !== "number") return null;
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 && String(id) === String(value) ? String(id) : null;
}

function matchesTarget(entry: ConversationTranscriptEntry, target: RetryTarget): entry is TranscriptMessage {
  if (entry.kind !== "message" || entry.role !== "user" || entry.clientNonce !== target.clientNonce) return false;
  if (entry.authorKind === undefined) {
    return target.authorId === undefined && entry.id === `pending-${target.clientNonce}`
      && entry.remoteMessageId === undefined;
  }
  return entry.authorKind === "human" && identity(entry.authorId)
    && (target.authorId === undefined || entry.authorId === target.authorId)
    && (entry.remoteMessageId === undefined
      ? entry.id === `human-message:${target.clientNonce}` || entry.id === `pending-${target.clientNonce}`
      : remoteMessageId(entry.remoteMessageId) !== null && entry.id === `human-server-message:${entry.remoteMessageId}`);
}

function isSettled(entry: ConversationTranscriptEntry, target: RetryTarget): entry is TranscriptMessage {
  return matchesTarget(entry, target) && entry.authorKind === "human" && entry.delivery === "sent";
}

function projectRetryResult(value: unknown, input: HumanMessageRetryInput, target: RetryTarget): TranscriptMessage {
  if (!isRecord(value) || value.kind !== "message" || value.role !== "user" || value.authorKind !== "human"
    || !identity(value.authorId) || value.clientNonce !== target.clientNonce
    || (target.authorId !== undefined && value.authorId !== target.authorId)
    || typeof value.content !== "string" || typeof value.timestampMs !== "number" || !Number.isFinite(value.timestampMs)
    || (value.conversationId !== undefined && value.conversationId !== target.conversationId)
    || (value.agentId !== undefined && value.agentId !== target.conversationId)) {
    throw new Error("retryHumanMessage returned an invalid Human transcript entry");
  }
  const local = value.id === `human-message:${target.clientNonce}` && value.remoteMessageId === undefined;
  const remoteId = remoteMessageId(value.remoteMessageId);
  const canonical = remoteId !== null && value.id === `human-server-message:${remoteId}`;
  if (value.delivery === "sent" ? !local && !canonical
    : !local || (value.delivery !== "pending" && value.delivery !== "dispatching")) {
    throw new Error("retryHumanMessage returned a conflicting Human message identity");
  }
  const projected = projectTranscriptEntry(value, 0, input.conversationName, target.conversationId);
  if (projected?.kind !== "message" || !matchesTarget(projected, target)) {
    throw new Error("retryHumanMessage returned an invalid Human message projection");
  }
  return projected;
}

/**
 * UI attempt lifetime only. The Host reads the durable intent and owns dispatch,
 * resources, forwarding and idempotency. This controller has no composer or
 * attachment bridge, and sends only the two original durable identities.
 */
export function createHumanMessageRetryController(options: HumanMessageRetryOptions): HumanMessageRetryController {
  let disposed = false;
  let accountGeneration: number | undefined;
  // Settled tokens remain until superseded so deferred React updaters can fence
  // an earlier failure after a later retry. Only pending tokens hold a flight.
  const attempts = new Map<string, RetryAttempt>();
  const keyFor = (conversationId: string, clientNonce: string) => JSON.stringify([conversationId, clientNonce]);
  const scopeIsCurrent = (scope: HumanMessageRetryScope, conversationId: string): boolean => {
    const current = options.readScope();
    return !disposed && current != null && scope.accountSlot != null
      && current.accountGeneration === scope.accountGeneration
      && current.accountSlot === scope.accountSlot
      && current.selectionGeneration === scope.selectionGeneration
      && current.selection === scope.selection
      && current.selection.currentAgentId === conversationId;
  };

  return {
    retry(input) {
      const scope = { ...input.scope };
      const conversationId = input.conversationId;
      const client = options.client;
      if (client == null || !scopeIsCurrent(scope, conversationId)) return Promise.resolve();
      if (!identity(conversationId) || !nonce(input.entry.clientNonce) || input.entry.delivery !== "failed") {
        options.setNotice((current) => scopeIsCurrent(scope, conversationId)
          ? "Message retry unavailable: the failed message has no valid durable identity."
          : current);
        return Promise.resolve();
      }
      const target: RetryTarget = {
        conversationId,
        clientNonce: input.entry.clientNonce,
        ...(input.entry.authorKind === "human" ? { authorId: input.entry.authorId } : {}),
      };
      if (!matchesTarget(input.entry, target) || input.entry.remoteMessageId !== undefined) {
        options.setNotice((current) => scopeIsCurrent(scope, conversationId)
          ? "Message retry unavailable: the failed message is not a local Human send."
          : current);
        return Promise.resolve();
      }
      if (accountGeneration !== scope.accountGeneration) {
        attempts.clear();
        accountGeneration = scope.accountGeneration;
      }
      const key = keyFor(conversationId, target.clientNonce);
      const previous = attempts.get(key);
      if (previous?.pending) return previous.completion;
      if (previous != null) previous.invalidated = true;
      const attempt: RetryAttempt = {
        scope, target, pending: true, invalidated: false, phase: "pending", completion: Promise.resolve(),
      };
      attempts.set(key, attempt);
      const isCurrent = () => !attempt.invalidated && scopeIsCurrent(scope, conversationId);
      const setDelivery = (delivery: "pending" | "failed") => {
        options.setEntries((current) => {
          if (!isCurrent() || attempt.phase !== delivery) return current;
          const entries = current[conversationId];
          if (entries == null || entries.some((entry) => isSettled(entry, target))) return current;
          let changed = false;
          const next = entries.map((entry) => {
            if (!matchesTarget(entry, target) || entry.remoteMessageId !== undefined || entry.delivery === "sent"
              || entry.delivery === delivery) return entry;
            changed = true;
            return { ...entry, delivery };
          });
          return changed ? { ...current, [conversationId]: next } : current;
        });
      };
      const setNotice = (notice: string | null) => {
        const phase = attempt.phase;
        options.setNotice((current) => isCurrent() && attempt.phase === phase ? notice : current);
      };
      // Reserve the flight before any asynchronous dispatch or state callback.
      // The client rechecks isCurrent at the actual frame after waiting for ready.
      attempt.completion = Promise.resolve().then(async () => {
        try {
          if (!isCurrent()) return;
          setDelivery("pending");
          setNotice(null);
          if (!isCurrent()) return;
          const result = await client.call("retryHumanMessage", {
            conversationId, clientNonce: target.clientNonce,
          }, { isCurrent });
          if (!isCurrent()) return;
          const authoritative = projectRetryResult(result, input, target);
          attempt.phase = authoritative.delivery === "failed" ? "failed" : "settled";
          options.onAuthoritativeEntry?.(scope, conversationId, authoritative, isCurrent);
          if (!isCurrent()) return;
          const settledPhase = attempt.phase;
          options.setEntries((current) => {
            if (!isCurrent() || attempt.phase !== settledPhase) return current;
            const entries = current[conversationId];
            if (entries == null || !entries.some((entry) => matchesTarget(entry, target))) return current;
            const settledIds = new Set(entries.filter((entry) => isSettled(entry, target)).map((entry) => entry.id));
            return {
              ...current,
              [conversationId]: reconcileTranscriptTail(entries, [authoritative], settledIds),
            };
          });
          if (!isCurrent()) return;
          if (authoritative.delivery === "sent") {
            options.acknowledgement.reconcileEcho({
              accountSlot: scope.accountSlot, agentId: conversationId,
              nonce: target.clientNonce, echoedNonce: target.clientNonce,
            });
          }
          setNotice(authoritative.delivery === "failed"
            ? `Message retry failed: ${String((result as Record<string, unknown>).lastDispatchError)}`
            : null);
        } catch (error) {
          if (!isCurrent()) return;
          attempt.phase = "failed";
          setDelivery("failed");
          setNotice(`Message retry failed: ${error instanceof Error ? error.message : String(error)}`);
        } finally {
          attempt.pending = false;
        }
      });
      return attempt.completion;
    },

    observeEntry(conversationId, entry) {
      if (disposed || entry.kind !== "message" || !nonce(entry.clientNonce)) return;
      const attempt = attempts.get(keyFor(conversationId, entry.clientNonce));
      if (attempt != null && options.readScope()?.accountGeneration === attempt.scope.accountGeneration
        && isSettled(entry, attempt.target)) attempt.invalidated = true;
    },

    dispose() {
      disposed = true;
      attempts.clear();
    },
  };
}
