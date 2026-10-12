import type { AgentDesktopBridge } from "../../../contracts/desktop-bridge";
import { areComposerDraftsEqual, parseComposerDraft, type ComposerDraft } from "./model.ts";

// Immutable root: ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4769359 (composer-drafts slice metadata)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4771060 (per-agent draft/recovery store)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4773604 (mounted draft lifecycle)

export const COMPOSER_DRAFT_SLICE = {
  slice: "composer-drafts",
  schemaVersion: 2,
  scope: "client-persisted",
  accountSensitive: true
} as const;

export type ComposerDraftScope =
  | { readonly kind: "agent"; readonly agentId: string }
  | { readonly kind: "conversation"; readonly conversationId: string }
  | { readonly kind: "topic"; readonly conversationId: string; readonly topicId: string }
  | { readonly kind: "savedSublist"; readonly conversationId: string; readonly participantId: string }
  | { readonly kind: "conversationChild"; readonly conversationId: string; readonly childConversationId: string };

export type ComposerDraftScopeInput = ComposerDraftScope | string;

export interface ComposerDraftSnapshot {
  readonly draft: ComposerDraft | null;
  readonly recovery: ComposerDraft | null;
  /** Transient keyed exchange slot. It is intentionally not persisted across account restore. */
  readonly stash: ComposerDraft | null;
}

export interface ComposerDraftSnapshotStore {
  get(): ComposerDraftSnapshot;
  subscribe(listener: () => void): () => void;
}

export interface ComposerDraftPersistence {
  read(accountSlot: string): Promise<string | null>;
  write(accountSlot: string, value: ComposerDraftEnvelope): Promise<void>;
  clear(accountSlot: string): Promise<void>;
}

export interface ComposerDraftEnvelope {
  scopes: Record<string, {
    readonly draft: ComposerDraft | null;
    readonly draftId: string | null;
    readonly recovery: ComposerDraft | null;
  }>;
}

export interface ComposerDraftStateStore {
  snapshotsFor(scope: ComposerDraftScopeInput): ComposerDraftSnapshotStore;
  setDraft(scope: ComposerDraftScopeInput, draft: ComposerDraft): void;
  identifyDraft(input: { agentId: string; draft: ComposerDraft }): { agentId: string; draftId: string } | null;
  clearDraftIfCurrent(input: { agentId: string; draftId: string }): boolean;
  clearDraftIfMatches(input: { agentId: string; draft: ComposerDraft }): boolean;
  clearDraft(scope: ComposerDraftScopeInput): void;
  recoverDraft(scope: ComposerDraftScopeInput, draft: ComposerDraft): void;
  clearRecovery(scope: ComposerDraftScopeInput): void;
  canExchangeDraft(scope: ComposerDraftScopeInput): boolean;
  exchangeDraft(scope: ComposerDraftScopeInput, validate?: (draft: ComposerDraft) => boolean): boolean;
  removeStash(scope: ComposerDraftScopeInput): boolean;
  removeStashIfMatches(scope: ComposerDraftScopeInput, draft: ComposerDraft): boolean;
  clearScope(scope: ComposerDraftScopeInput): void;
  restore(accountSlot: string | null): Promise<void>;
  reset(): void;
  dispose(): void;
}

const EMPTY_SNAPSHOT: ComposerDraftSnapshot = { draft: null, recovery: null, stash: null };
const EMPTY_RECORD = { draft: null, draftId: null, recovery: null } as const;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function encodeAccountSlot(accountSlot: string): string {
  return encodeURIComponent(accountSlot).replaceAll(".", "%2E");
}

function encodeScopePart(value: string, label: string): string {
  if (value.trim().length === 0) throw new Error(`${label} must not be empty`);
  return encodeURIComponent(value);
}

export function composerDraftScopeKey(scope: ComposerDraftScopeInput): string {
  if (typeof scope === "string") {
    return `agent:${encodeScopePart(scope, "agentId")}`;
  }
  switch (scope.kind) {
    case "agent":
      return `agent:${encodeScopePart(scope.agentId, "agentId")}`;
    case "conversation":
      return `conversation:${encodeScopePart(scope.conversationId, "conversationId")}`;
    case "topic":
      return `topic:${encodeScopePart(scope.conversationId, "conversationId")}:${encodeScopePart(scope.topicId, "topicId")}`;
    case "savedSublist":
      return `saved-sublist:${encodeScopePart(scope.conversationId, "conversationId")}:${encodeScopePart(scope.participantId, "participantId")}`;
    case "conversationChild":
      return `conversation-child:${encodeScopePart(scope.conversationId, "conversationId")}:${encodeScopePart(scope.childConversationId, "childConversationId")}`;
  }
}

function isPersistedComposerDraftScopeKey(value: string): boolean {
  return /^(?:agent|conversation):[^:]+$/u.test(value)
    || /^(?:topic|saved-sublist|conversation-child):[^:]+:[^:]+$/u.test(value);
}

export function composerDraftPersistenceKey(accountSlot: string): string {
  if (accountSlot.length === 0) throw new Error("accountSlot must not be empty");
  return `sand.client.slice.account.${encodeAccountSlot(accountSlot)}.${COMPOSER_DRAFT_SLICE.slice}`;
}

function cloneDraft(draft: ComposerDraft | null): ComposerDraft | null {
  if (draft == null) return null;
  return {
    prompt: draft.prompt,
    attachments: draft.attachments.map(({ path, name, size, mimeType }) => ({
      path,
      name,
      ...(size === undefined ? {} : { size }),
      ...(mimeType === undefined ? {} : { mimeType }),
    })),
    ...(draft.richText === undefined ? {} : { richText: draft.richText }),
    ...(draft.replyToId === undefined ? {} : { replyToId: draft.replyToId }),
    ...(draft.isFork === undefined ? {} : { isFork: draft.isFork })
  };
}

function cloneSnapshot(snapshot: ComposerDraftSnapshot): ComposerDraftSnapshot {
  return { draft: cloneDraft(snapshot.draft), recovery: cloneDraft(snapshot.recovery), stash: cloneDraft(snapshot.stash) };
}

function parseEnvelope(value: string | null):
  | { kind: "absent" }
  | { kind: "corrupt" }
  | { kind: "envelope"; schemaVersion: number; value: unknown } {
  if (value == null) return { kind: "absent" };
  try {
    const parsed: unknown = JSON.parse(value);
    if (!isRecord(parsed) || typeof parsed.schemaVersion !== "number" || !("value" in parsed)) return { kind: "corrupt" };
    return { kind: "envelope", schemaVersion: parsed.schemaVersion, value: parsed.value };
  } catch {
    return { kind: "corrupt" };
  }
}

function parsePersistedValue(
  value: unknown,
  schemaVersion: number,
): Map<string, { draft: ComposerDraft | null; draftId: string | null; recovery: ComposerDraft | null }> | null {
  if (!isRecord(value)) return null;
  const rawScopes = schemaVersion === 1 ? value.agents : schemaVersion === 2 ? value.scopes : null;
  if (!isRecord(rawScopes)) return null;
  const result = new Map<string, { draft: ComposerDraft | null; draftId: string | null; recovery: ComposerDraft | null }>();
  for (const [persistedKey, raw] of Object.entries(rawScopes)) {
    const scopeKey = schemaVersion === 1
      ? composerDraftScopeKey(persistedKey)
      : isPersistedComposerDraftScopeKey(persistedKey) ? persistedKey : null;
    if (scopeKey == null || !isRecord(raw)) return null;
    const draft = raw.draft == null ? null : parseComposerDraft(raw.draft);
    const recovery = raw.recovery == null ? null : parseComposerDraft(raw.recovery);
    if ((raw.draft != null && draft == null) || (raw.recovery != null && recovery == null)) return null;
    const draftId = raw.draftId == null ? null : typeof raw.draftId === "string" && raw.draftId.length > 0 ? raw.draftId : null;
    if (raw.draftId != null && draftId == null || draft == null && draftId != null) return null;
    if (draft == null && recovery == null) continue;
    result.set(scopeKey, { draft, draftId, recovery });
  }
  return result;
}

function draftId(): string {
  const crypto = globalThis.crypto;
  return crypto?.randomUUID != null
    ? crypto.randomUUID()
    : `d-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

export function createComposerDraftPersistence(clientPersistence: AgentDesktopBridge["clientPersistence"]): ComposerDraftPersistence {
  return {
    async read(accountSlot) {
      return clientPersistence.read(composerDraftPersistenceKey(accountSlot));
    },
    async write(accountSlot, value) {
      await clientPersistence.write(
        composerDraftPersistenceKey(accountSlot),
        JSON.stringify({ schemaVersion: COMPOSER_DRAFT_SLICE.schemaVersion, value })
      );
    },
    clear: (accountSlot) => clientPersistence.remove(composerDraftPersistenceKey(accountSlot))
  };
}

export function createComposerDraftStateStore(persistence: ComposerDraftPersistence): ComposerDraftStateStore {
  const records = new Map<string, { draft: ComposerDraft | null; draftId: string | null; recovery: ComposerDraft | null }>();
  const snapshots = new Map<string, { snapshot: ComposerDraftSnapshot; listeners: Set<() => void> }>();
  const stashes = new Map<string, ComposerDraft>();
  let accountSlot: string | null = null;
  let generation = 0;
  let mutationRevision = 0;
  let disposed = false;
  let writes = Promise.resolve();

  const snapshotFor = (scopeKey: string) => {
    const existing = snapshots.get(scopeKey);
    if (existing != null) return existing;
    const created = { snapshot: EMPTY_SNAPSHOT, listeners: new Set<() => void>() };
    snapshots.set(scopeKey, created);
    return created;
  };
  const notify = (scopeKey: string, next: ComposerDraftSnapshot) => {
    const state = snapshotFor(scopeKey);
    state.snapshot = cloneSnapshot(next);
    for (const listener of [...state.listeners]) listener();
  };
  const currentRecord = (scopeKey: string) => records.get(scopeKey) ?? EMPTY_RECORD;
  const serialize = (): ComposerDraftEnvelope => {
    const scopes: ComposerDraftEnvelope["scopes"] = {};
    for (const [scopeKey, record] of records) {
      if (record.draft == null && record.recovery == null) continue;
      scopes[scopeKey] = { draft: cloneDraft(record.draft), draftId: record.draftId, recovery: cloneDraft(record.recovery) };
    }
    return { scopes };
  };
  const enqueueWrite = () => {
    if (accountSlot == null || disposed) return;
    const slot = accountSlot;
    const value = serialize();
    writes = writes.then(() => persistence.write(slot, value)).catch(() => {});
  };
  const replace = (scopeKey: string, next: { draft: ComposerDraft | null; draftId: string | null; recovery: ComposerDraft | null }) => {
    mutationRevision += 1;
    if (next.draft == null && next.recovery == null) records.delete(scopeKey);
    else records.set(scopeKey, { draft: cloneDraft(next.draft), draftId: next.draftId, recovery: cloneDraft(next.recovery) });
    notify(scopeKey, { draft: next.draft, recovery: next.recovery, stash: stashes.get(scopeKey) ?? null });
    enqueueWrite();
  };
  const isCurrent = (expectedGeneration: number, expectedAccount: string): boolean =>
    !disposed && generation === expectedGeneration && accountSlot === expectedAccount;

  return {
    snapshotsFor(scope) {
      const scopeKey = composerDraftScopeKey(scope);
      const state = snapshotFor(scopeKey);
      return {
        get: () => state.snapshot,
        subscribe(listener) {
          state.listeners.add(listener);
          return () => state.listeners.delete(listener);
        }
      };
    },
    setDraft(scope, draft) {
      if (disposed) return;
      const scopeKey = composerDraftScopeKey(scope);
      const current = currentRecord(scopeKey);
      if (draft.prompt.trim().length === 0 && draft.attachments.length === 0) {
        if (current.draft == null) return;
        replace(scopeKey, { draft: null, draftId: null, recovery: current.recovery });
        return;
      }
      if (current.draft != null && areComposerDraftsEqual(current.draft, draft)) return;
      replace(scopeKey, { draft, draftId: draftId(), recovery: current.recovery });
    },
    identifyDraft({ agentId, draft }) {
      if (disposed || agentId.length === 0) return null;
      const scopeKey = composerDraftScopeKey(agentId);
      const current = currentRecord(scopeKey);
      return current.draft != null && current.draftId != null && areComposerDraftsEqual(current.draft, draft)
        ? { agentId, draftId: current.draftId }
        : null;
    },
    clearDraftIfCurrent({ agentId, draftId: currentDraftId }) {
      if (disposed || agentId.length === 0) return false;
      const scopeKey = composerDraftScopeKey(agentId);
      const current = currentRecord(scopeKey);
      if (current.draft == null || current.draftId !== currentDraftId) return false;
      replace(scopeKey, { draft: null, draftId: null, recovery: current.recovery });
      return true;
    },
    clearDraftIfMatches({ agentId, draft }) {
      if (disposed || agentId.length === 0) return false;
      const scopeKey = composerDraftScopeKey(agentId);
      const current = currentRecord(scopeKey);
      if (current.draft == null || !areComposerDraftsEqual(current.draft, draft)) return false;
      replace(scopeKey, { draft: null, draftId: null, recovery: current.recovery });
      return true;
    },
    clearDraft(scope) {
      if (disposed) return;
      const scopeKey = composerDraftScopeKey(scope);
      const current = currentRecord(scopeKey);
      if (current.draft != null) replace(scopeKey, { draft: null, draftId: null, recovery: current.recovery });
    },
    recoverDraft(scope, recovery) {
      if (disposed || recovery.prompt.trim().length === 0 && recovery.attachments.length === 0) return;
      const scopeKey = composerDraftScopeKey(scope);
      const current = currentRecord(scopeKey);
      replace(scopeKey, current.draft == null
        ? { draft: recovery, draftId: draftId(), recovery: current.recovery }
        : { draft: current.draft, draftId: current.draftId, recovery });
    },
    clearRecovery(scope) {
      if (disposed) return;
      const scopeKey = composerDraftScopeKey(scope);
      const current = currentRecord(scopeKey);
      if (current.recovery != null) replace(scopeKey, { draft: current.draft, draftId: current.draftId, recovery: null });
    },
    canExchangeDraft(scope) {
      if (disposed) return false;
      const scopeKey = composerDraftScopeKey(scope);
      return currentRecord(scopeKey).draft != null || stashes.has(scopeKey);
    },
    exchangeDraft(scope, validate) {
      if (disposed) return false;
      const scopeKey = composerDraftScopeKey(scope);
      const current = currentRecord(scopeKey);
      const stashed = stashes.get(scopeKey) ?? null;
      if (stashed != null && validate != null && !validate(stashed)) return false;
      if (current.draft == null && stashed == null) return false;
      if (current.draft == null) stashes.delete(scopeKey);
      else stashes.set(scopeKey, cloneDraft(current.draft)!);
      const nextDraft = cloneDraft(stashed);
      replace(scopeKey, {
        draft: nextDraft,
        draftId: nextDraft == null ? null : draftId(),
        recovery: current.recovery
      });
      return true;
    },
    removeStash(scope) {
      if (disposed) return false;
      const scopeKey = composerDraftScopeKey(scope);
      if (!stashes.delete(scopeKey)) return false;
      const current = currentRecord(scopeKey);
      notify(scopeKey, { draft: current.draft, recovery: current.recovery, stash: null });
      return true;
    },
    removeStashIfMatches(scope, draft) {
      if (disposed) return false;
      const scopeKey = composerDraftScopeKey(scope);
      const stashed = stashes.get(scopeKey);
      if (stashed == null || !areComposerDraftsEqual(stashed, draft)) return false;
      stashes.delete(scopeKey);
      const current = currentRecord(scopeKey);
      notify(scopeKey, { draft: current.draft, recovery: current.recovery, stash: null });
      return true;
    },
    clearScope(scope) {
      if (disposed) return;
      const scopeKey = composerDraftScopeKey(scope);
      const hadStash = stashes.delete(scopeKey);
      const hadRecord = records.delete(scopeKey);
      if (!hadStash && !hadRecord) return;
      mutationRevision += 1;
      notify(scopeKey, EMPTY_SNAPSHOT);
      enqueueWrite();
    },
    async restore(nextAccountSlot) {
      generation += 1;
      const expectedGeneration = generation;
      const expectedMutationRevision = mutationRevision;
      accountSlot = nextAccountSlot;
      records.clear();
      stashes.clear();
      for (const agentKey of snapshots.keys()) notify(agentKey, EMPTY_SNAPSHOT);
      if (disposed || nextAccountSlot == null) return;
      await writes;
      if (!isCurrent(expectedGeneration, nextAccountSlot) || mutationRevision !== expectedMutationRevision) return;
      const stored = parseEnvelope(await persistence.read(nextAccountSlot));
      if (!isCurrent(expectedGeneration, nextAccountSlot) || mutationRevision !== expectedMutationRevision) return;
      if (stored.kind === "absent") return;
      if (stored.kind === "corrupt" || stored.schemaVersion !== 1 && stored.schemaVersion !== COMPOSER_DRAFT_SLICE.schemaVersion) {
        writes = writes.then(() => persistence.clear(nextAccountSlot)).catch(() => {});
        return;
      }
      const restored = parsePersistedValue(stored.value, stored.schemaVersion);
      if (restored == null) {
        writes = writes.then(() => persistence.clear(nextAccountSlot)).catch(() => {});
        return;
      }
      for (const [scopeKey, record] of restored) {
        records.set(scopeKey, record);
        notify(scopeKey, { draft: record.draft, recovery: record.recovery, stash: null });
      }
      if (stored.schemaVersion === 1) enqueueWrite();
    },
    reset() {
      generation += 1;
      accountSlot = null;
      records.clear();
      stashes.clear();
      for (const agentKey of snapshots.keys()) notify(agentKey, EMPTY_SNAPSHOT);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      accountSlot = null;
      records.clear();
      stashes.clear();
      for (const state of snapshots.values()) state.listeners.clear();
      snapshots.clear();
    }
  };
}
