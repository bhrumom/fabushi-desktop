import assert from "node:assert/strict";
import test from "node:test";
import {
  COMPOSER_DRAFT_SLICE,
  composerDraftPersistenceKey,
  createComposerDraftPersistence,
  createComposerDraftStateStore,
  type ComposerDraftStateStore,
} from "./draft-state.ts";
import type { ComposerDraft } from "./model.ts";

// NMR-DRAFT-01: exercise the existing store and persistence adapter. Only the
// external durable read/write boundary is controlled; no draft owner is copied.
const ACCOUNT = "draft-account.a";
const CONVERSATION = "human-conversation";
const AUTHOR = "local-human";
const NONCE = "original-submission";
const TIMEOUT = { timeout: 10_000 };
const identity = { accountSlot: ACCOUNT, agentId: CONVERSATION, authorId: AUTHOR, nonce: NONCE };
const originalDraft: ComposerDraft = {
  prompt: "Original Human draft",
  attachments: [{ path: "/staged/report.txt", name: "report.txt", size: 17 }],
  richText: '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Original Human draft"}]}]}',
  replyToId: "human-server-message:41",
  isFork: true,
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((accept, fail) => { resolve = accept; reject = fail; });
  return { promise, resolve, reject };
}

interface StoredSubmission {
  nonce: string;
  authorId: string;
  draftId: string | null;
  recoveryId: string | null;
}
interface StoredAgent {
  draft: ComposerDraft | null;
  draftId: string | null;
  recovery: ComposerDraft | null;
  recoveryId?: string | null;
  submissions?: StoredSubmission[];
}
interface StoredEnvelope {
  schemaVersion: number;
  value: { agents: Record<string, StoredAgent> };
}
interface WriteAttempt {
  key: string;
  raw: string;
  envelope: StoredEnvelope;
}

function persistenceHarness() {
  const durable = new Map<string, string>();
  const attempts: WriteAttempt[] = [];
  const commits: WriteAttempt[] = [];
  const removed: string[] = [];
  const readGates: { key: string; entered: ReturnType<typeof deferred<void>>; release: ReturnType<typeof deferred<void>>; reply?: { raw: string | null } }[] = [];
  const writeGates: {
    matches(attempt: WriteAttempt): boolean;
    entered: ReturnType<typeof deferred<WriteAttempt>>;
    release: ReturnType<typeof deferred<void>>;
  }[] = [];
  const commitWaiters: { after: number; completion: ReturnType<typeof deferred<WriteAttempt>> }[] = [];
  const persistence = createComposerDraftPersistence({
    async read(key) {
      const index = readGates.findIndex((gate) => gate.key === key);
      if (index >= 0) {
        const [gate] = readGates.splice(index, 1);
        gate.entered.resolve();
        await gate.release.promise;
        if (gate.reply !== undefined) return gate.reply.raw;
      }
      return durable.get(key) ?? null;
    },
    async write(key, raw) {
      const attempt = { key, raw, envelope: JSON.parse(raw) as StoredEnvelope };
      attempts.push(attempt);
      const index = writeGates.findIndex((gate) => gate.matches(attempt));
      if (index >= 0) {
        const [gate] = writeGates.splice(index, 1);
        gate.entered.resolve(attempt);
        await gate.release.promise;
      }
      durable.set(key, raw);
      commits.push(attempt);
      for (let index = commitWaiters.length - 1; index >= 0; index -= 1) {
        const waiter = commitWaiters[index];
        if (commits.length > waiter.after) {
          commitWaiters.splice(index, 1);
          waiter.completion.resolve(attempt);
        }
      }
    },
    async remove(key) { removed.push(key); durable.delete(key); },
    async listKeys(prefix) { return [...durable.keys()].filter((key) => key.startsWith(prefix)); },
    async migrateFromLocalStorage() { return false; },
  });
  return {
    persistence, attempts, commits, removed,
    createStore: () => createComposerDraftStateStore(persistence),
    seed(accountSlot: string, agents: Record<string, unknown>) {
      durable.set(composerDraftPersistenceKey(accountSlot), JSON.stringify({ schemaVersion: 1, value: { agents } }));
    },
    stored(accountSlot = ACCOUNT): StoredEnvelope | null {
      const raw = durable.get(composerDraftPersistenceKey(accountSlot));
      return raw == null ? null : JSON.parse(raw) as StoredEnvelope;
    },
    holdRead(accountSlot = ACCOUNT, reply?: { raw: string | null }) {
      const entered = deferred<void>();
      const release = deferred<void>();
      readGates.push({ key: composerDraftPersistenceKey(accountSlot), entered, release, ...(reply === undefined ? {} : { reply }) });
      return { entered: entered.promise, release: () => release.resolve() };
    },
    holdWrite(matches: (attempt: WriteAttempt) => boolean) {
      const entered = deferred<WriteAttempt>();
      const release = deferred<void>();
      writeGates.push({ matches, entered, release });
      return { entered: entered.promise, release: () => release.resolve(), fail: (error: unknown) => release.reject(error) };
    },
    committedAfter(after: number): Promise<WriteAttempt> {
      if (commits.length > after) return Promise.resolve(commits[after]);
      const completion = deferred<WriteAttempt>();
      commitWaiters.push({ after, completion });
      return completion.promise;
    },
  };
}

function hasOriginalBinding(attempt: WriteAttempt): boolean {
  return attempt.key === composerDraftPersistenceKey(ACCOUNT)
    && attempt.envelope.value.agents[CONVERSATION]?.submissions?.some((binding) => binding.nonce === NONCE) === true;
}

function draftIdentity(store: ComposerDraftStateStore, agentId = CONVERSATION) {
  const draft = store.snapshotsFor(agentId).get().draft;
  assert.ok(draft);
  const result = store.identifyDraft({ agentId, draft });
  assert.ok(result);
  return result;
}

function recoveryIdentity(store: ComposerDraftStateStore, agentId = CONVERSATION) {
  const draft = store.snapshotsFor(agentId).get().recovery;
  assert.ok(draft);
  const result = store.identifyRecovery({ agentId, draft });
  assert.ok(result);
  return result;
}

async function seedOriginal(store: ComposerDraftStateStore) {
  await store.restore(ACCOUNT);
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const original = draftIdentity(store);
  const binding = { ...identity, draftId: original.draftId, recoveryId: null };
  await store.bindHumanSubmission(binding);
  return binding;
}

test("NMR-DRAFT-01 binding durability precedes the caller's first dispatch", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  await store.restore(ACCOUNT);
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const original = draftIdentity(store);
  const held = disk.holdWrite(hasOriginalBinding);
  let dispatched = 0;
  // This is the public store barrier's caller contract. The renderer/IPC
  // dispatch wiring is separately exercised by its real UI/Electron gates.
  const sending = store.bindHumanSubmission({ ...identity, draftId: original.draftId, recoveryId: null })
    .then(() => { dispatched += 1; });
  const attempted = await held.entered;
  assert.equal(dispatched, 0);
  assert.equal(attempted.envelope.schemaVersion, 1);
  assert.equal(COMPOSER_DRAFT_SLICE.schemaVersion, 1);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.length ?? 0, 0);
  held.release();
  await sending;
  assert.equal(dispatched, 1);
  assert.deepEqual(disk.stored()?.value.agents[CONVERSATION]?.submissions, [{
    nonce: NONCE, authorId: AUTHOR, draftId: original.draftId, recoveryId: null,
  }]);
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get().draft, originalDraft);
  store.dispose();
});

test("NMR-DRAFT-01 a failed binding write rejects dispatch and can be retried durably", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  await store.restore(ACCOUNT);
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const binding = { ...identity, draftId: draftIdentity(store).draftId, recoveryId: null };
  const held = disk.holdWrite(hasOriginalBinding);
  let dispatched = 0;
  const sending = store.bindHumanSubmission(binding).then(() => { dispatched += 1; });
  const rejected = assert.rejects(sending, /binding write unavailable/);
  await held.entered;
  held.fail(new Error("binding write unavailable"));
  await rejected;
  assert.equal(dispatched, 0);
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get().draft, originalDraft);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.length ?? 0, 0);
  await store.bindHumanSubmission(binding);
  dispatched += 1;
  assert.equal(dispatched, 1);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.[0].draftId, binding.draftId);
  store.dispose();
});

test("NMR-DRAFT-01 a newly instantiated store retires only the persisted original draft once", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const beforeRestart = disk.createStore();
  const binding = await seedOriginal(beforeRestart);
  beforeRestart.dispose();
  const restored = disk.createStore();
  await restored.restore(ACCOUNT);
  assert.equal(draftIdentity(restored).draftId, binding.draftId);
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: true, clearedRecovery: false });
  assert.deepEqual(restored.snapshotsFor(CONVERSATION).get(), { draft: null, recovery: null });
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  restored.dispose();
  const reopened = disk.createStore();
  await reopened.restore(ACCOUNT);
  assert.deepEqual(reopened.snapshotsFor(CONVERSATION).get(), { draft: null, recovery: null });
  assert.deepEqual(await reopened.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  reopened.dispose();
});

test("NMR-DRAFT-01 repeating a nonce cannot retarget an edited then identical new draft", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  const binding = await seedOriginal(store);
  store.setDraft(CONVERSATION, { ...structuredClone(originalDraft), prompt: "An intervening edit" });
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const replacement = draftIdentity(store);
  assert.notEqual(replacement.draftId, binding.draftId);
  await store.bindHumanSubmission({ ...binding, draftId: replacement.draftId });
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.[0].draftId, binding.draftId);
  store.dispose();
  const restored = disk.createStore();
  await restored.restore(ACCOUNT);
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.equal(draftIdentity(restored).draftId, replacement.draftId);
  assert.deepEqual(restored.snapshotsFor(CONVERSATION).get().draft, originalDraft);
  assert.deepEqual(disk.stored()?.value.agents[CONVERSATION]?.draft, originalDraft);
  restored.dispose();
});

test("NMR-DRAFT-01 wrong author account conversation and nonce cannot retire ownership", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  const binding = await seedOriginal(store);
  for (const foreign of [
    { ...identity, authorId: "peer-human" },
    { ...identity, accountSlot: "another-account" },
    { ...identity, agentId: "another-conversation" },
    { ...identity, nonce: "another-submission" },
  ]) {
    assert.deepEqual(await store.settleHumanSubmission(foreign), { clearedDraft: false, clearedRecovery: false });
    assert.equal(draftIdentity(store).draftId, binding.draftId);
    assert.deepEqual(store.snapshotsFor(CONVERSATION).get().draft, originalDraft);
  }
  assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: true, clearedRecovery: false });
  store.dispose();
});

test("NMR-DRAFT-01 restore preserves pending canonical observation and independent conversation edits", TIMEOUT, async () => {
  const disk = persistenceHarness();
  disk.seed(ACCOUNT, { [CONVERSATION]: {
    draft: originalDraft, draftId: "persisted-original", recovery: null, recoveryId: null,
    submissions: [{ nonce: NONCE, authorId: AUTHOR, draftId: "persisted-original", recoveryId: null }],
  } });
  const held = disk.holdRead();
  const store = disk.createStore();
  const restoring = store.restore(ACCOUNT);
  await held.entered;
  let settled = false;
  const settling = store.settleHumanSubmission(identity).then((result) => { settled = true; return result; });
  await Promise.resolve();
  assert.equal(settled, false);
  held.release();
  await restoring;
  assert.deepEqual(await settling, { clearedDraft: true, clearedRecovery: false });
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: null, recovery: null });
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.length ?? 0, 0);
  store.dispose();

  const editedDisk = persistenceHarness();
  const otherConversation = "edited-during-restore";
  const recovery = { ...structuredClone(originalDraft), prompt: "Persisted recovery" };
  editedDisk.seed(ACCOUNT, {
    [CONVERSATION]: { draft: originalDraft, draftId: "unrelated-original", recovery, recoveryId: "unrelated-recovery",
      submissions: [{ nonce: NONCE, authorId: AUTHOR, draftId: "unrelated-original", recoveryId: "unrelated-recovery" }] },
    [otherConversation]: { draft: { prompt: "Old other draft", attachments: [] }, draftId: "old-other", recovery: null },
  });
  const delayedRead = editedDisk.holdRead();
  const editing = editedDisk.createStore();
  const editingRestore = editing.restore(ACCOUNT);
  await delayedRead.entered;
  const newer = { ...structuredClone(originalDraft), prompt: "User edited another conversation during restore" };
  editing.setDraft(otherConversation, newer);
  const newerIdentity = draftIdentity(editing, otherConversation);
  delayedRead.release();
  await editingRestore;
  assert.deepEqual(editing.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery });
  assert.equal(draftIdentity(editing).draftId, "unrelated-original");
  assert.equal(recoveryIdentity(editing).recoveryId, "unrelated-recovery");
  assert.deepEqual(editing.snapshotsFor(otherConversation).get().draft, newer);
  assert.equal(draftIdentity(editing, otherConversation).draftId, newerIdentity.draftId);
  // Await the real owner barrier to drain the merged persistence write before
  // reopening; it cannot reconstruct the lost unrelated record for the store.
  await editing.bindHumanSubmission({ ...identity, agentId: otherConversation, nonce: "other-edit",
    draftId: newerIdentity.draftId, recoveryId: null });
  editing.dispose();
  const editingReopened = editedDisk.createStore();
  await editingReopened.restore(ACCOUNT);
  assert.deepEqual(editingReopened.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery });
  assert.deepEqual(editingReopened.snapshotsFor(otherConversation).get().draft, newer);
  assert.equal(draftIdentity(editingReopened, otherConversation).draftId, newerIdentity.draftId);
  assert.deepEqual(await editingReopened.settleHumanSubmission(identity), { clearedDraft: true, clearedRecovery: true });
  assert.deepEqual(editingReopened.snapshotsFor(otherConversation).get().draft, newer);
  editingReopened.dispose();
});

test("NMR-DRAFT-01 legacy and malformed binding metadata preserve valid V1 draft and recovery content", TIMEOUT, async () => {
  const valid = { nonce: NONCE, authorId: AUTHOR, draftId: "legacy-draft", recoveryId: "legacy-recovery" };
  const cases: { label: string; metadata: Record<string, unknown> }[] = [
    { label: "legacy without ownership", metadata: {} },
    { label: "non-array submissions", metadata: { submissions: { ...valid } } },
    { label: "empty nonce", metadata: { submissions: [{ ...valid, nonce: "" }] } },
    { label: "empty author", metadata: { submissions: [{ ...valid, authorId: "" }] } },
    { label: "invalid draft identity", metadata: { submissions: [{ ...valid, draftId: 42 }] } },
    { label: "invalid recovery identity", metadata: { submissions: [{ ...valid, recoveryId: 42 }] } },
    { label: "duplicate nonce", metadata: { submissions: [valid, { ...valid, draftId: "another-draft" }] } },
    { label: "malformed metadata beside valid content", metadata: { recoveryId: 42, submissions: [{ ...valid, authorId: null }] } },
  ];
  const recovery = { ...structuredClone(originalDraft), prompt: "Preserved legacy recovery" };
  for (const fixture of cases) {
    const disk = persistenceHarness();
    disk.seed(ACCOUNT, { [CONVERSATION]: {
      draft: originalDraft, draftId: "legacy-draft", recovery, ...fixture.metadata,
    } });
    const store = disk.createStore();
    await store.restore(ACCOUNT);
    assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery }, fixture.label);
    assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false }, fixture.label);
    assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery }, fixture.label);
    assert.equal(disk.removed.length, 0, fixture.label);
    store.dispose();
  }
});

test("NMR-DRAFT-01 account ABA rejects an old binding completion before dispatch", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  await store.restore(ACCOUNT);
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const binding = { ...identity, draftId: draftIdentity(store).draftId, recoveryId: null };
  const held = disk.holdWrite(hasOriginalBinding);
  let dispatched = 0;
  const sending = store.bindHumanSubmission(binding).then(() => { dispatched += 1; });
  const rejected = assert.rejects(sending);
  await held.entered;
  const away = store.restore("another-account");
  const returned = store.restore(ACCOUNT);
  held.release();
  await Promise.all([rejected, away, returned]);
  assert.equal(dispatched, 0);
  const newDraft = { ...structuredClone(originalDraft), prompt: "New account lifetime draft" };
  store.setDraft(CONVERSATION, newDraft);
  const current = draftIdentity(store);
  assert.notEqual(current.draftId, binding.draftId);
  assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get().draft, newDraft);
  store.dispose();
});

test("NMR-DRAFT-01 disposal fences pending bindings and old restore writes from a newer store", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  await store.restore(ACCOUNT);
  store.setDraft(CONVERSATION, structuredClone(originalDraft));
  const binding = { ...identity, draftId: draftIdentity(store).draftId, recoveryId: null };
  let publications = 0;
  store.snapshotsFor(CONVERSATION).subscribe(() => { publications += 1; });
  const held = disk.holdWrite(hasOriginalBinding);
  let dispatched = 0;
  const sending = store.bindHumanSubmission(binding).then(() => { dispatched += 1; });
  const rejected = assert.rejects(sending);
  await held.entered;
  store.dispose();
  const atDisposal = publications;
  held.release();
  await rejected;
  assert.equal(dispatched, 0);
  assert.equal(publications, atDisposal);
  assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.equal(publications, atDisposal);

  for (const staleRead of ["valid", "corrupt"] as const) {
    const sharedDisk = persistenceHarness();
    const otherConversation = "preserved-next-store-conversation";
    const unrelated = { prompt: "Other conversation remains durable", attachments: [] };
    sharedDisk.seed(ACCOUNT, {
      [CONVERSATION]: { draft: originalDraft, draftId: "disk-before-old-restore", recovery: null },
      [otherConversation]: { draft: unrelated, draftId: "unrelated-persisted-id", recovery: null },
    });
    // Freeze only the old external read's result. A second real store reads
    // and updates the same durable key before that obsolete callback resumes.
    const staleRaw = staleRead === "valid" ? JSON.stringify(sharedDisk.stored()) : "{invalid-old-envelope";
    const heldOldRead = sharedDisk.holdRead(ACCOUNT, { raw: staleRaw });
    const obsolete = sharedDisk.createStore();
    const obsoleteRestore = obsolete.restore(ACCOUNT);
    await heldOldRead.entered;
    obsolete.setDraft(CONVERSATION, { ...structuredClone(originalDraft), prompt: "Old lifetime edit during restore" });
    obsolete.dispose();

    const current = sharedDisk.createStore();
    await current.restore(ACCOUNT);
    const newer = { ...structuredClone(originalDraft), prompt: "New store B", replyToId: "human-server-message:77", isFork: false };
    current.setDraft(CONVERSATION, newer);
    const newerIdentity = draftIdentity(current);
    await current.bindHumanSubmission({ ...identity, nonce: "new-store-submission",
      draftId: newerIdentity.draftId, recoveryId: null });
    const durableBeforeRelease = structuredClone(sharedDisk.stored());
    const attemptsBeforeRelease = sharedDisk.attempts.length;
    const commitsBeforeRelease = sharedDisk.commits.length;
    const clearsBeforeRelease = sharedDisk.removed.length;

    heldOldRead.release();
    await obsoleteRestore;
    assert.equal(sharedDisk.attempts.length, attemptsBeforeRelease, staleRead);
    assert.equal(sharedDisk.commits.length, commitsBeforeRelease, staleRead);
    assert.equal(sharedDisk.removed.length, clearsBeforeRelease, staleRead);
    assert.deepEqual(sharedDisk.stored(), durableBeforeRelease, staleRead);
    assert.deepEqual(current.snapshotsFor(CONVERSATION).get().draft, newer, staleRead);
    assert.equal(draftIdentity(current).draftId, newerIdentity.draftId, staleRead);
    assert.deepEqual(current.snapshotsFor(otherConversation).get().draft, unrelated, staleRead);
    current.dispose();

    const reopened = sharedDisk.createStore();
    await reopened.restore(ACCOUNT);
    assert.deepEqual(reopened.snapshotsFor(CONVERSATION).get().draft, newer, staleRead);
    assert.equal(draftIdentity(reopened).draftId, newerIdentity.draftId, staleRead);
    assert.deepEqual(reopened.snapshotsFor(otherConversation).get().draft, unrelated, staleRead);
    assert.deepEqual(sharedDisk.stored(), durableBeforeRelease, staleRead);
    reopened.dispose();
  }
});

test("NMR-DRAFT-01 failure recovery keeps a stable persisted ID and cannot adopt the newer composer", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  const binding = await seedOriginal(store);
  const newer = { ...structuredClone(originalDraft), prompt: "Newer composer draft", replyToId: "human-server-message:99", isFork: false };
  store.setDraft(CONVERSATION, newer);
  const newerIdentity = draftIdentity(store);
  store.recoverDraft(CONVERSATION, structuredClone(originalDraft), identity);
  const recovery = recoveryIdentity(store);
  await store.bindHumanSubmission(binding);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.recoveryId, recovery.recoveryId);
  store.dispose();
  const restored = disk.createStore();
  await restored.restore(ACCOUNT);
  assert.equal(recoveryIdentity(restored).recoveryId, recovery.recoveryId);
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: true });
  assert.equal(draftIdentity(restored).draftId, newerIdentity.draftId);
  assert.deepEqual(restored.snapshotsFor(CONVERSATION).get(), { draft: newer, recovery: null });
  restored.dispose();
});

test("NMR-DRAFT-01 failure-created draft ownership survives restart without capturing an unrelated draft", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  await store.restore(ACCOUNT);
  const binding = { ...identity, draftId: null, recoveryId: null };
  await store.bindHumanSubmission(binding);
  store.recoverDraft(CONVERSATION, structuredClone(originalDraft), identity);
  const created = draftIdentity(store);
  await store.bindHumanSubmission(binding);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.[0].draftId, created.draftId);
  store.dispose();
  const restored = disk.createStore();
  await restored.restore(ACCOUNT);
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: true, clearedRecovery: false });
  assert.deepEqual(restored.snapshotsFor(CONVERSATION).get(), { draft: null, recovery: null });
  restored.dispose();
});

test("NMR-DRAFT-01 failed settlement persistence remains retryable across restart and newer edits", TIMEOUT, async () => {
  for (const restart of [false, true]) {
    const disk = persistenceHarness();
    let store = disk.createStore();
    const binding = await seedOriginal(store);
    const held = disk.holdWrite((attempt) => attempt.key === composerDraftPersistenceKey(ACCOUNT) && !hasOriginalBinding(attempt));
    const settling = store.settleHumanSubmission(identity);
    const rejected = assert.rejects(settling, /settlement write unavailable/);
    await held.entered;
    held.fail(new Error("settlement write unavailable"));
    await rejected;
    assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.[0].draftId, binding.draftId);
    if (restart) {
      store.dispose();
      store = disk.createStore();
      await store.restore(ACCOUNT);
      assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: true, clearedRecovery: false });
      assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: null, recovery: null });
    } else {
      const newer = { ...structuredClone(originalDraft), prompt: "Edited after settlement write failed" };
      store.setDraft(CONVERSATION, newer);
      const newerIdentity = draftIdentity(store);
      await store.bindHumanSubmission(binding);
      assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
      assert.equal(draftIdentity(store).draftId, newerIdentity.draftId);
      assert.deepEqual(store.snapshotsFor(CONVERSATION).get().draft, newer);
    }
    assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.length ?? 0, 0);
    store.dispose();
  }
});

test("NMR-DRAFT-01 empty drafts retain bindings but explicit forget cannot revive them during restore or reopen", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  const binding = await seedOriginal(store);
  const otherConversation = "another-human-conversation";
  store.setDraft(otherConversation, { prompt: "Other conversation", attachments: [] });
  await store.bindHumanSubmission({ ...identity, agentId: otherConversation, nonce: "other", draftId: draftIdentity(store, otherConversation).draftId, recoveryId: null });
  store.clearDraft(CONVERSATION);
  await store.bindHumanSubmission(binding);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.draft, null);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.[0].nonce, NONCE);
  const beforeForget = disk.commits.length;
  store.forgetHumanSubmissions({ accountSlot: ACCOUNT, agentId: CONVERSATION });
  await disk.committedAfter(beforeForget);
  assert.equal(disk.stored()?.value.agents[CONVERSATION]?.submissions?.length ?? 0, 0);
  assert.equal(disk.stored()?.value.agents[otherConversation]?.submissions?.[0].nonce, "other");
  store.dispose();
  const restored = disk.createStore();
  await restored.restore(ACCOUNT);
  restored.setDraft(CONVERSATION, structuredClone(originalDraft));
  const recreated = draftIdentity(restored);
  assert.notEqual(recreated.draftId, binding.draftId);
  assert.deepEqual(await restored.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.deepEqual(restored.snapshotsFor(CONVERSATION).get().draft, originalDraft);
  assert.equal(draftIdentity(restored).draftId, recreated.draftId);
  restored.dispose();

  const delayedDisk = persistenceHarness();
  const recovery = { ...structuredClone(originalDraft), prompt: "Retained after explicit forget" };
  delayedDisk.seed(ACCOUNT, {
    [CONVERSATION]: { draft: originalDraft, draftId: "forgotten-draft", recovery, recoveryId: "forgotten-recovery",
      submissions: [{ nonce: NONCE, authorId: AUTHOR, draftId: "forgotten-draft", recoveryId: "forgotten-recovery" }] },
    [otherConversation]: { draft: originalDraft, draftId: "retained-draft", recovery: null,
      submissions: [{ nonce: NONCE, authorId: AUTHOR, draftId: "retained-draft", recoveryId: null }] },
  });
  const heldRead = delayedDisk.holdRead();
  const forgetting = delayedDisk.createStore();
  const forgottenRestore = forgetting.restore(ACCOUNT);
  await heldRead.entered;
  let forgottenDispatches = 0;
  const obsoleteBinding = forgetting.bindHumanSubmission({ ...identity,
    draftId: "forgotten-draft", recoveryId: "forgotten-recovery" }).then(() => { forgottenDispatches += 1; });
  const obsoleteRejected = assert.rejects(obsoleteBinding);
  forgetting.forgetHumanSubmissions({ ...identity });
  heldRead.release();
  await Promise.all([forgottenRestore, obsoleteRejected]);
  assert.equal(forgottenDispatches, 0);
  assert.deepEqual(forgetting.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery });
  assert.deepEqual(await forgetting.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.deepEqual(forgetting.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery });
  assert.deepEqual(forgetting.snapshotsFor(otherConversation).get().draft, originalDraft);
  assert.deepEqual(await forgetting.settleHumanSubmission({ ...identity, agentId: otherConversation }),
    { clearedDraft: true, clearedRecovery: false });
  forgetting.dispose();
  const forgottenReopened = delayedDisk.createStore();
  await forgottenReopened.restore(ACCOUNT);
  assert.deepEqual(await forgottenReopened.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.deepEqual(forgottenReopened.snapshotsFor(CONVERSATION).get(), { draft: originalDraft, recovery });
  forgottenReopened.dispose();
});

test("NMR-DRAFT-01 an older settlement cannot clear a replacement recovery or bind a foreign failure", TIMEOUT, async () => {
  const disk = persistenceHarness();
  const store = disk.createStore();
  const binding = await seedOriginal(store);
  const newer = { ...structuredClone(originalDraft), prompt: "New draft and failure" };
  store.setDraft(CONVERSATION, newer);
  const newerIdentity = draftIdentity(store);
  const beforeForeignFailure = store.snapshotsFor(CONVERSATION).get();
  store.recoverDraft(CONVERSATION, structuredClone(originalDraft), { ...identity, authorId: "peer-human" });
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), beforeForeignFailure);
  await store.bindHumanSubmission(binding);
  assert.deepEqual(await store.settleHumanSubmission(identity), { clearedDraft: false, clearedRecovery: false });
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: newer, recovery: null });

  const first = { ...identity, nonce: "first-recovery" };
  await store.bindHumanSubmission({ ...first, draftId: null, recoveryId: null });
  store.recoverDraft(CONVERSATION, structuredClone(originalDraft), first);
  const firstRecovery = recoveryIdentity(store);
  const second = { ...identity, nonce: "replacement-recovery" };
  await store.bindHumanSubmission({ ...second, draftId: newerIdentity.draftId, recoveryId: null });
  store.recoverDraft(CONVERSATION, newer, second);
  const secondRecovery = recoveryIdentity(store);
  assert.notEqual(secondRecovery.recoveryId, firstRecovery.recoveryId);
  await store.bindHumanSubmission({ ...second, draftId: newerIdentity.draftId, recoveryId: null });
  assert.deepEqual(await store.settleHumanSubmission(first), { clearedDraft: false, clearedRecovery: false });
  assert.equal(recoveryIdentity(store).recoveryId, secondRecovery.recoveryId);
  assert.deepEqual(store.snapshotsFor(CONVERSATION).get(), { draft: newer, recovery: newer });
  assert.deepEqual(await store.settleHumanSubmission(second), { clearedDraft: true, clearedRecovery: true });
  store.dispose();
});
