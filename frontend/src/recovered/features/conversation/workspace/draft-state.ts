import type { AgentDesktopBridge } from "../../../contracts/desktop-bridge";
import { areComposerDraftsEqual, parseComposerDraft, type ComposerDraft } from "./model.ts";

// Immutable root: ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4769359 (composer-drafts slice metadata)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4771060 (per-agent draft/recovery store)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4773604 (mounted draft lifecycle)

export const COMPOSER_DRAFT_SLICE = {
  slice: "composer-drafts",
  schemaVersion: 1,
  scope: "client-persisted",
  accountSensitive: true
} as const;

export interface ComposerDraftSnapshot {
  readonly draft: ComposerDraft | null;
  readonly recovery: ComposerDraft | null;
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
  agents: Record<string, ComposerDraftSnapshot & {
    readonly draftId: string | null;
    readonly recoveryId?: string | null;
    readonly submissions?: readonly ComposerHumanDraftBinding[];
  }>;
}

export interface ComposerHumanDraftSubmissionKey {
  readonly accountSlot: string;
  readonly agentId: string;
  readonly authorId: string;
  readonly nonce: string;
}

export interface ComposerHumanDraftBinding {
  readonly authorId: string;
  readonly nonce: string;
  readonly draftId: string | null;
  readonly recoveryId: string | null;
}

type ComposerDraftRecord = ComposerDraftSnapshot & {
  readonly draftId: string | null;
  readonly recoveryId: string | null;
  readonly submissions: readonly ComposerHumanDraftBinding[];
};

type ComposerRestoreContext = {
  slot: string;
  generation: number;
  loading: boolean;
  needsWrite: boolean;
  done: Promise<void>;
  edits: Map<string, { record: ComposerDraftRecord; draftChanged: boolean; recoveryChanged: boolean }>;
  forgotten: Map<string, Set<string> | "all">;
};

export interface ComposerDraftStateStore {
  snapshotsFor(agentKey: string): ComposerDraftSnapshotStore;
  setDraft(agentKey: string, draft: ComposerDraft): void;
  identifyDraft(input: { agentId: string; draft: ComposerDraft }): { agentId: string; draftId: string } | null;
  identifyRecovery(input: { agentId: string; draft: ComposerDraft }): { agentId: string; recoveryId: string } | null;
  bindHumanSubmission(input: ComposerHumanDraftSubmissionKey & { draftId: string | null; recoveryId: string | null }): Promise<void>;
  settleHumanSubmission(input: ComposerHumanDraftSubmissionKey): Promise<{ clearedDraft: boolean; clearedRecovery: boolean }>;
  forgetHumanSubmissions(input: { accountSlot: string; agentId: string; nonce?: string }): void;
  clearDraftIfCurrent(input: { agentId: string; draftId: string }): boolean;
  clearDraftIfMatches(input: { agentId: string; draft: ComposerDraft }): boolean;
  clearDraft(agentKey: string): void;
  recoverDraft(agentKey: string, draft: ComposerDraft, submission?: Omit<ComposerHumanDraftSubmissionKey, "agentId">): void;
  clearRecovery(agentKey: string): void;
  restore(accountSlot: string | null): Promise<void>;
  reset(): void;
  dispose(): void;
}

const EMPTY_SNAPSHOT: ComposerDraftSnapshot = { draft: null, recovery: null };
const EMPTY_RECORD: ComposerDraftRecord = { draft: null, draftId: null, recovery: null, recoveryId: null, submissions: [] };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function encodeAccountSlot(accountSlot: string): string {
  return encodeURIComponent(accountSlot).replaceAll(".", "%2E");
}

export function composerDraftPersistenceKey(accountSlot: string): string {
  if (accountSlot.length === 0) throw new Error("accountSlot must not be empty");
  return `sand.client.slice.account.${encodeAccountSlot(accountSlot)}.${COMPOSER_DRAFT_SLICE.slice}`;
}

function cloneDraft(draft: ComposerDraft | null): ComposerDraft | null {
  if (draft == null) return null;
  return {
    prompt: draft.prompt,
    attachments: draft.attachments.map(({ path, name, size }) => ({ path, name, ...(size === undefined ? {} : { size }) })),
    ...(draft.richText === undefined ? {} : { richText: draft.richText }),
    ...(draft.replyToId === undefined ? {} : { replyToId: draft.replyToId }),
    ...(draft.isFork === undefined ? {} : { isFork: draft.isFork })
  };
}

function cloneSnapshot(snapshot: ComposerDraftSnapshot): ComposerDraftSnapshot {
  return { draft: cloneDraft(snapshot.draft), recovery: cloneDraft(snapshot.recovery) };
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

function nonemptyIdentity(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function parseSubmissionBindings(value: unknown): readonly ComposerHumanDraftBinding[] {
  if (value == null) return [];
  if (!Array.isArray(value)) return [];
  const result: ComposerHumanDraftBinding[] = [];
  const nonces = new Set<string>();
  for (const raw of value) {
    // Invalid new metadata never makes otherwise valid V1 draft text disposable.
    if (!isRecord(raw) || Object.keys(raw).some((key) => !["nonce", "authorId", "draftId", "recoveryId"].includes(key))
      || !nonemptyIdentity(raw.nonce) || !nonemptyIdentity(raw.authorId)
      || !(raw.draftId === null || nonemptyIdentity(raw.draftId))
      || !(raw.recoveryId === null || nonemptyIdentity(raw.recoveryId))
      || raw.draftId === null && raw.recoveryId === null || nonces.has(raw.nonce)) return [];
    nonces.add(raw.nonce);
    result.push({ nonce: raw.nonce, authorId: raw.authorId, draftId: raw.draftId, recoveryId: raw.recoveryId });
  }
  return result;
}

function parsePersistedValue(value: unknown): Map<string, ComposerDraftRecord> | null {
  if (!isRecord(value) || !isRecord(value.agents)) return null;
  const result = new Map<string, ComposerDraftRecord>();
  for (const [agentKey, raw] of Object.entries(value.agents)) {
    if (agentKey.length === 0 || !isRecord(raw)) return null;
    const draft = raw.draft == null ? null : parseComposerDraft(raw.draft);
    const recovery = raw.recovery == null ? null : parseComposerDraft(raw.recovery);
    if ((raw.draft != null && draft == null) || (raw.recovery != null && recovery == null)) return null;
    const draftId = raw.draftId == null ? null : typeof raw.draftId === "string" && raw.draftId.length > 0 ? raw.draftId : null;
    if (raw.draftId != null && draftId == null || draft == null && draftId != null) return null;
    const submissions = parseSubmissionBindings(raw.submissions);
    const recoveryId = recovery == null ? null : nonemptyIdentity(raw.recoveryId) ? raw.recoveryId : newDraftIdentity();
    if (draft == null && recovery == null && submissions.length === 0) continue;
    result.set(agentKey, { draft, draftId: draft == null ? null : draftId ?? newDraftIdentity(), recovery, recoveryId, submissions });
  }
  return result;
}

function newDraftIdentity(): string {
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
  const records = new Map<string, ComposerDraftRecord>();
  const snapshots = new Map<string, { snapshot: ComposerDraftSnapshot; listeners: Set<() => void> }>();
  let accountSlot: string | null = null;
  let generation = 0;
  let disposed = false;
  let writes = Promise.resolve();
  let restoring = Promise.resolve();
  let activeRestore: ComposerRestoreContext | null = null;
  let writeBlock: unknown = null;
  const priorAccountRestores = new Map<string, Promise<void>>();
  const bindingEpochs = new Map<string, number>();
  const nonceEpochs = new Map<string, Map<string, number>>();
  const bindingEpoch = (agentId: string, nonce: string) =>
    `${bindingEpochs.get(agentId) ?? 0}:${nonceEpochs.get(agentId)?.get(nonce) ?? 0}`;

  const snapshotFor = (agentKey: string) => {
    const existing = snapshots.get(agentKey);
    if (existing != null) return existing;
    const created = { snapshot: EMPTY_SNAPSHOT, listeners: new Set<() => void>() };
    snapshots.set(agentKey, created);
    return created;
  };
  const notify = (agentKey: string, next: ComposerDraftSnapshot) => {
    const state = snapshotFor(agentKey);
    state.snapshot = cloneSnapshot(next);
    for (const listener of [...state.listeners]) listener();
  };
  const currentRecord = (agentKey: string) => records.get(agentKey) ?? EMPTY_RECORD;
  const serialize = (source: ReadonlyMap<string, ComposerDraftRecord> = records): ComposerDraftEnvelope => {
    const agents: ComposerDraftEnvelope["agents"] = {};
    for (const [agentKey, record] of source) {
      if (record.draft == null && record.recovery == null && record.submissions.length === 0) continue;
      agents[agentKey] = {
        draft: cloneDraft(record.draft), draftId: record.draftId,
        recovery: cloneDraft(record.recovery), recoveryId: record.recoveryId,
        ...(record.submissions.length === 0 ? {} : { submissions: record.submissions.map((binding) => ({ ...binding })) })
      };
    }
    return { agents };
  };
  const enqueueSnapshot = (slot: string, value: ComposerDraftEnvelope): Promise<void> => {
    const write = writes.then(() => disposed ? undefined : persistence.write(slot, value));
    writes = write.catch(() => {});
    return write;
  };
  const enqueueWrite = (): Promise<void> => {
    if (accountSlot == null || disposed) return Promise.resolve();
    if (activeRestore?.loading && activeRestore.generation === generation) {
      activeRestore.needsWrite = true;
      return activeRestore.done;
    }
    if (writeBlock != null) {
      const failed = Promise.reject<void>(writeBlock);
      void failed.catch(() => {});
      return failed;
    }
    return enqueueSnapshot(accountSlot, serialize());
  };
  const replace = (agentKey: string, next: ComposerDraftRecord): Promise<void> => {
    const previous = currentRecord(agentKey);
    if (activeRestore?.loading && activeRestore.generation === generation) {
      const prior = activeRestore.edits.get(agentKey);
      activeRestore.edits.set(agentKey, {
        record: next,
        draftChanged: prior?.draftChanged === true || previous.draftId !== next.draftId || previous.draft !== next.draft,
        recoveryChanged: prior?.recoveryChanged === true || previous.recoveryId !== next.recoveryId || previous.recovery !== next.recovery,
      });
    }
    if (next.draft == null && next.recovery == null && next.submissions.length === 0) records.delete(agentKey);
    else records.set(agentKey, { ...next, draft: cloneDraft(next.draft), recovery: cloneDraft(next.recovery), submissions: next.submissions.map((binding) => ({ ...binding })) });
    notify(agentKey, { draft: next.draft, recovery: next.recovery });
    return enqueueWrite();
  };
  const isCurrent = (expectedGeneration: number, expectedAccount: string): boolean =>
    !disposed && generation === expectedGeneration && accountSlot === expectedAccount;

  return {
    snapshotsFor(agentKey) {
      const state = snapshotFor(agentKey);
      return {
        get: () => state.snapshot,
        subscribe(listener) {
          state.listeners.add(listener);
          return () => state.listeners.delete(listener);
        }
      };
    },
    setDraft(agentKey, draft) {
      if (disposed || agentKey.length === 0) return;
      const current = currentRecord(agentKey);
      if (draft.prompt.trim().length === 0 && draft.attachments.length === 0) {
        if (current.draft == null) return;
        void replace(agentKey, { ...current, draft: null, draftId: null });
        return;
      }
      if (current.draft != null && areComposerDraftsEqual(current.draft, draft)) return;
      void replace(agentKey, { ...current, draft, draftId: newDraftIdentity() });
    },
    identifyDraft({ agentId, draft }) {
      if (disposed || agentId.length === 0) return null;
      const current = currentRecord(agentId);
      return current.draft != null && current.draftId != null && areComposerDraftsEqual(current.draft, draft)
        ? { agentId, draftId: current.draftId }
        : null;
    },
    identifyRecovery({ agentId, draft }) {
      if (disposed || agentId.length === 0) return null;
      const current = currentRecord(agentId);
      return current.recovery != null && current.recoveryId != null && areComposerDraftsEqual(current.recovery, draft)
        ? { agentId, recoveryId: current.recoveryId } : null;
    },
    async bindHumanSubmission(input) {
      const expectedGeneration = generation;
      const expectedEpoch = bindingEpoch(input.agentId, input.nonce);
      if (!nonemptyIdentity(input.accountSlot) || !nonemptyIdentity(input.agentId)
        || !nonemptyIdentity(input.authorId) || !nonemptyIdentity(input.nonce)
        || !(input.draftId === null || nonemptyIdentity(input.draftId))
        || !(input.recoveryId === null || nonemptyIdentity(input.recoveryId))
        || !isCurrent(expectedGeneration, input.accountSlot)) throw new Error("Human draft submission scope is unavailable");
      await restoring;
      if (!isCurrent(expectedGeneration, input.accountSlot) || bindingEpoch(input.agentId, input.nonce) !== expectedEpoch) throw new Error("Human draft submission scope changed");
      const current = currentRecord(input.agentId);
      const previous = current.submissions.find((binding) => binding.nonce === input.nonce);
      if (previous != null && previous.authorId !== input.authorId) throw new Error("Human draft submission author mismatch");
      // Retry keeps its original IDs even if the user has since recreated equal text.
      const binding = previous ?? { nonce: input.nonce, authorId: input.authorId, draftId: input.draftId, recoveryId: input.recoveryId };
      if (binding.draftId != null || binding.recoveryId != null) {
        await replace(input.agentId, { ...current, submissions: previous == null ? [...current.submissions, binding] : current.submissions });
      } else await writes;
      if (!isCurrent(expectedGeneration, input.accountSlot) || bindingEpoch(input.agentId, input.nonce) !== expectedEpoch) throw new Error("Human draft submission scope changed");
    },
    async settleHumanSubmission(input) {
      const unchanged = { clearedDraft: false, clearedRecovery: false };
      const expectedGeneration = generation;
      const expectedEpoch = bindingEpoch(input.agentId, input.nonce);
      if (!isCurrent(expectedGeneration, input.accountSlot) || !nonemptyIdentity(input.agentId)
        || !nonemptyIdentity(input.authorId) || !nonemptyIdentity(input.nonce)) return unchanged;
      await restoring;
      if (!isCurrent(expectedGeneration, input.accountSlot) || bindingEpoch(input.agentId, input.nonce) !== expectedEpoch) return unchanged;
      const current = currentRecord(input.agentId);
      const binding = current.submissions.find((entry) => entry.nonce === input.nonce && entry.authorId === input.authorId);
      if (binding == null) return unchanged;
      const clearedDraft = binding.draftId != null && current.draft != null && current.draftId === binding.draftId;
      const clearedRecovery = binding.recoveryId != null && current.recovery != null && current.recoveryId === binding.recoveryId;
      try {
        await replace(input.agentId, {
          ...current,
          ...(clearedDraft ? { draft: null, draftId: null } : {}),
          ...(clearedRecovery ? { recovery: null, recoveryId: null } : {}),
          submissions: current.submissions.filter((entry) => entry !== binding),
        });
      } catch (error) {
        // The disk may still contain A. Keep its identity available for a later
        // canonical observation, without restoring A over a new in-memory draft.
        if (isCurrent(expectedGeneration, input.accountSlot) && bindingEpoch(input.agentId, input.nonce) === expectedEpoch) {
          const latest = currentRecord(input.agentId);
          if (!latest.submissions.some((entry) => entry.nonce === binding.nonce)) {
            records.set(input.agentId, { ...latest, submissions: [...latest.submissions, binding] });
          }
        }
        throw error;
      }
      return isCurrent(expectedGeneration, input.accountSlot) && bindingEpoch(input.agentId, input.nonce) === expectedEpoch
        ? { clearedDraft, clearedRecovery } : unchanged;
    },
    forgetHumanSubmissions({ accountSlot: expectedAccount, agentId, nonce }) {
      if (!isCurrent(generation, expectedAccount) || agentId.length === 0) return;
      if (nonce == null) bindingEpochs.set(agentId, (bindingEpochs.get(agentId) ?? 0) + 1);
      else {
        const epochs = nonceEpochs.get(agentId) ?? new Map<string, number>();
        epochs.set(nonce, (epochs.get(nonce) ?? 0) + 1);
        nonceEpochs.set(agentId, epochs);
      }
      if (activeRestore?.loading && activeRestore.generation === generation) {
        const prior = activeRestore.forgotten.get(agentId);
        activeRestore.forgotten.set(agentId, nonce == null || prior === "all" ? "all" : new Set([...(prior ?? []), nonce]));
        activeRestore.needsWrite = true;
      }
      const current = currentRecord(agentId);
      const submissions = nonce == null ? [] : current.submissions.filter((binding) => binding.nonce !== nonce);
      if (submissions.length !== current.submissions.length) void replace(agentId, { ...current, submissions });
    },
    clearDraftIfCurrent({ agentId, draftId: currentDraftId }) {
      if (disposed || agentId.length === 0) return false;
      const current = currentRecord(agentId);
      if (current.draft == null || current.draftId !== currentDraftId) return false;
      void replace(agentId, { ...current, draft: null, draftId: null });
      return true;
    },
    clearDraftIfMatches({ agentId, draft }) {
      if (disposed || agentId.length === 0) return false;
      const current = currentRecord(agentId);
      if (current.draft == null || !areComposerDraftsEqual(current.draft, draft)) return false;
      void replace(agentId, { ...current, draft: null, draftId: null });
      return true;
    },
    clearDraft(agentKey) {
      if (disposed || agentKey.length === 0) return;
      const current = currentRecord(agentKey);
      if (current.draft != null) void replace(agentKey, { ...current, draft: null, draftId: null });
    },
    recoverDraft(agentKey, recovery, submission) {
      if (disposed || agentKey.length === 0 || recovery.prompt.trim().length === 0 && recovery.attachments.length === 0) return;
      if (submission != null && (!isCurrent(generation, submission.accountSlot)
        || !nonemptyIdentity(submission.authorId) || !nonemptyIdentity(submission.nonce))) return;
      const current = currentRecord(agentKey);
      const previous = submission == null ? undefined : current.submissions.find((binding) => binding.nonce === submission.nonce);
      if (submission != null && previous != null && previous.authorId !== submission.authorId) return;
      let next: ComposerDraftRecord = current.draft == null
        ? { ...current, draft: recovery, draftId: newDraftIdentity() }
        : { ...current, recovery, recoveryId: newDraftIdentity() };
      if (submission != null) {
        const binding = {
          nonce: submission.nonce, authorId: submission.authorId,
          draftId: current.draft == null ? next.draftId : previous?.draftId ?? null,
          recoveryId: current.draft == null ? previous?.recoveryId ?? null : next.recoveryId,
        };
        next = { ...next, submissions: [...current.submissions.filter((entry) => entry.nonce !== submission.nonce), binding] };
      }
      void replace(agentKey, next);
    },
    clearRecovery(agentKey) {
      if (disposed || agentKey.length === 0) return;
      const current = currentRecord(agentKey);
      if (current.recovery != null) void replace(agentKey, { ...current, recovery: null, recoveryId: null });
    },
    restore(nextAccountSlot) {
      generation += 1;
      const expectedGeneration = generation;
      accountSlot = nextAccountSlot;
      records.clear();
      bindingEpochs.clear();
      nonceEpochs.clear();
      writeBlock = null;
      let resolveRestore!: () => void;
      let rejectRestore!: (error: unknown) => void;
      const done = new Promise<void>((resolve, reject) => { resolveRestore = resolve; rejectRestore = reject; });
      void done.catch(() => {});
      restoring = done;
      const context: ComposerRestoreContext | null = disposed || nextAccountSlot == null ? null : {
        slot: nextAccountSlot, generation: expectedGeneration, loading: true, needsWrite: false,
        done, edits: new Map(), forgotten: new Map(),
      };
      activeRestore = context;
      for (const agentKey of snapshots.keys()) notify(agentKey, EMPTY_SNAPSHOT);
      if (context == null) { resolveRestore(); return done; }
      const previousRestore = priorAccountRestores.get(context.slot) ?? Promise.resolve();
      priorAccountRestores.set(context.slot, done);
      const precedingWrites = writes;
      void (async () => {
        await previousRestore.catch(() => {});
        await precedingWrites;
        if (disposed) { context.loading = false; return; }
        const stored = parseEnvelope(await persistence.read(context.slot));
        // A retired Renderer may finish its disk read after a replacement owner
        // has already persisted a new draft. It must neither clear nor rewrite it.
        if (disposed) { context.loading = false; return; }
        let restored = stored.kind === "envelope" && stored.schemaVersion === COMPOSER_DRAFT_SLICE.schemaVersion
          ? parsePersistedValue(stored.value) : stored.kind === "absent" ? new Map<string, ComposerDraftRecord>() : null;
        if (restored == null) {
          const clear = writes.then(() => disposed ? undefined : persistence.clear(context.slot));
          writes = clear.catch(() => {});
          await clear;
          if (disposed) { context.loading = false; return; }
          restored = new Map();
        }
        // While the read waits, edits belong to individual fields, not a new
        // empty account snapshot. Preserve untouched conversations and recovery.
        for (const [agentKey, edit] of context.edits) {
          const prior = restored.get(agentKey) ?? EMPTY_RECORD;
          const bindings = new Map(prior.submissions.map((binding) => [binding.nonce, binding]));
          for (const binding of edit.record.submissions) if (!bindings.has(binding.nonce)) bindings.set(binding.nonce, binding);
          restored.set(agentKey, {
            draft: edit.draftChanged ? edit.record.draft : prior.draft,
            draftId: edit.draftChanged ? edit.record.draftId : prior.draftId,
            recovery: edit.recoveryChanged ? edit.record.recovery : prior.recovery,
            recoveryId: edit.recoveryChanged ? edit.record.recoveryId : prior.recoveryId,
            submissions: [...bindings.values()],
          });
        }
        for (const [agentKey, forgotten] of context.forgotten) {
          const record = restored.get(agentKey);
          if (record != null) restored.set(agentKey, { ...record,
            submissions: forgotten === "all" ? [] : record.submissions.filter((binding) => !forgotten.has(binding.nonce)),
          });
        }
        context.loading = false;
        // Queue the merged snapshot before notifying consumers; an edit made
        // by a subscriber must be persisted after this snapshot, never before it.
        const write = context.needsWrite ? enqueueSnapshot(context.slot, serialize(restored)) : Promise.resolve();
        if (isCurrent(context.generation, context.slot)) {
          records.clear();
          for (const [agentKey, record] of restored) records.set(agentKey, record);
          for (const agentKey of new Set([...snapshots.keys(), ...restored.keys()])) notify(agentKey, currentRecord(agentKey));
        }
        await write;
      })().then(resolveRestore, (error: unknown) => {
        context.loading = false;
        if (isCurrent(context.generation, context.slot)) writeBlock = error;
        rejectRestore(error);
      });
      return done;
    },
    reset() {
      generation += 1;
      accountSlot = null;
      records.clear();
      bindingEpochs.clear();
      nonceEpochs.clear();
      activeRestore = null;
      for (const agentKey of snapshots.keys()) notify(agentKey, EMPTY_SNAPSHOT);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      accountSlot = null;
      records.clear();
      bindingEpochs.clear();
      nonceEpochs.clear();
      activeRestore = null;
      for (const state of snapshots.values()) state.listeners.clear();
      snapshots.clear();
    }
  };
}
