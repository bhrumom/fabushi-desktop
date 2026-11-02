import assert from "node:assert/strict";
import test from "node:test";
import type { CoordinatorPortBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge.ts";
import { createComposerSubmissionQueue, type ComposerSubmissionPhase } from "../recovered/features/conversation/workspace/submission.ts";
import { createSendJournalApprovalLifecycle } from "../recovered/features/conversation/workspace/send-journal-approval-lifecycle.ts";
import { createComposerDraftStateStore, type ComposerDraftEnvelope } from "../recovered/features/conversation/workspace/draft-state.ts";
import { commitComposerAttachments } from "../recovered/features/conversation/workspace/desktop.ts";
import { CoordinatorCallError, createCoordinatorClient } from "./coordinator-client.ts";
import { createComposerDispatchOrigin, reactScopedHumanMessage, sendScopedComposerSubmission, type ScopedComposerSubmission } from "./composer-dispatch-scope.ts";

const bounded = { timeout: 3_000 };
const conversation = { id: "same-human-pair", name: "Original peer" };

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}

interface RequestFrame {
  kind: "request";
  requestId: string;
  method: string;
  args: Record<string, unknown>;
}
interface PostedRequest { frame: RequestFrame; port: ControlledPort }

class ControlledPort implements TransferredCoordinatorPort {
  private readonly messages = new Set<(event: { data: unknown }) => void>();
  private readonly closures = new Set<(event: Record<string, never>) => void>();
  constructor(private readonly post: (port: ControlledPort, value: unknown) => void) {}
  postMessage(value: unknown) { this.post(this, value); }
  start() {}
  close() { for (const listener of this.closures) listener({}); }
  addEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  addEventListener(type: "close", listener: (event: Record<string, never>) => void): void;
  addEventListener(type: "message" | "close", listener: ((event: { data: unknown }) => void) | ((event: Record<string, never>) => void)) {
    if (type === "message") this.messages.add(listener as (event: { data: unknown }) => void);
    else this.closures.add(listener as (event: Record<string, never>) => void);
  }
  receive(value: unknown) { for (const listener of this.messages) listener({ data: value }); }
}

function transportHarness() {
  let consumer!: Parameters<CoordinatorPortBridge["claim"]>[0];
  const requests: PostedRequest[] = [];
  const frames: unknown[] = [];
  const observers = new Set<() => void>();
  const client = createCoordinatorClient({
    claim(value) {
      consumer = value;
      return { request() {}, release() {} };
    },
  });
  assert.ok(client);
  let port: ControlledPort;
  const connect = () => {
    port = new ControlledPort((source, value) => {
      frames.push(value);
      if (typeof value === "object" && value != null && "kind" in value && value.kind === "request") {
        requests.push({ port: source, frame: value as RequestFrame });
        for (const observer of [...observers]) observer();
      }
    });
    consumer.onPort(port);
    port.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  };
  connect();
  const request = (method: string, index = 0): Promise<PostedRequest> => {
    const found = () => requests.filter((entry) => entry.frame.method === method)[index];
    const current = found();
    if (current != null) return Promise.resolve(current);
    return new Promise((resolve) => {
      const observer = () => { const next = found(); if (next != null) { observers.delete(observer); resolve(next); } };
      observers.add(observer);
    });
  };
  return {
    client, requests, frames, connect, request,
    disconnect: () => port.receive({ kind: "lifecycle", phase: "shutdown" }),
    reply: (posted: PostedRequest, value: unknown) => posted.port.receive({ kind: "reply", requestId: posted.frame.requestId, outcome: { status: "ok", value } }),
    fail: (posted: PostedRequest) => posted.port.receive({ kind: "reply", requestId: posted.frame.requestId, outcome: { status: "failed", failure: { code: "denied", message: "old account failure" } } }),
  };
}

function accountHarness() {
  let generation = 1;
  let accountSlot: string | null = "A";
  let mounted = true;
  return {
    origin: (human = true) => createComposerDispatchOrigin({
      accountScopeGeneration: generation,
      accountSlot,
      humanConversation: human ? conversation : null,
      isAccountScopeCurrent: (expected) => mounted && accountSlot != null && expected === generation,
    }),
    change(next: string | null) { generation += 1; accountSlot = next; },
    unmount() { generation += 1; mounted = false; },
  };
}

function submission(origin: ScopedComposerSubmission["origin"], nonce = "original-nonce"): ScopedComposerSubmission {
  return {
    origin,
    nonce,
    agentId: conversation.id,
    prompt: "Original account text",
    attachments: [{ path: "/staging/original.png", name: "original.png" }],
    createdAtMs: 1234,
    replyToId: "human-server-message:7",
  };
}

function projections() {
  const committed: unknown[] = [];
  const results: unknown[] = [];
  const rosters: unknown[] = [];
  return {
    committed, results, rosters,
    onAttachmentsCommitted: (input: ScopedComposerSubmission, attachments: unknown) => { committed.push({ input, attachments }); },
    onHumanResult: (input: ScopedComposerSubmission, result: unknown) => { results.push({ input, result }); },
    onHumanRoster: (input: ScopedComposerSubmission, value: unknown) => { rosters.push({ input, value }); },
  };
}

const identityCommit = async (attachments: ScopedComposerSubmission["attachments"]) => attachments;
const retired = (error: unknown) => error instanceof CoordinatorCallError && error.code === "request-retired";

async function settleHuman(h: ReturnType<typeof transportHarness>, index = 0) {
  const send = await h.request("sendHumanMessage", index);
  h.reply(send, { id: `human-server-message:${index + 8}` });
  const roster = await h.request("listHumanConversations", index);
  h.reply(roster, [conversation]);
  return send;
}

test("admission freezes the original account and Human lane through phase clones", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  let current = 3;
  const source = { ...conversation };
  const origin = createComposerDispatchOrigin({ accountScopeGeneration: 3, accountSlot: "A", humanConversation: source, isAccountScopeCurrent: (value) => value === current });
  source.id = "replacement-agent";
  source.name = "replacement name";
  assert.equal(Object.isFrozen(origin), true);
  assert.equal(origin.kind, "human");
  if (origin.kind !== "human") throw new Error("expected Human origin");
  assert.equal(Object.isFrozen(origin.conversation), true);
  assert.deepEqual(origin.conversation, conversation);
  const phases: (ScopedComposerSubmission & { phase: ComposerSubmissionPhase })[] = [];
  const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({ isTransportDown: () => true, send: async () => {}, onPhase: (value) => phases.push(value) });
  t.after(() => queue.dispose());
  const input = submission(origin);
  const pending = queue.submit(input);
  assert.notEqual(phases[0], input);
  assert.equal(phases[0].origin, origin);
  assert.equal(queue.snapshot()[0].origin, origin);
  current = 4;
  assert.equal(phases[0].origin.isCurrent(), false);
  queue.reset();
  assert.equal(await pending.completion, "cancelled");
  assert.equal(phases.length, 1);
  assert.deepEqual(h.requests, []);
});

test("offline A intent resets before B sends to the same conversation and never becomes Agent work", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  let down = true;
  const seen = projections();
  const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
    isTransportDown: () => down,
    send: (input) => sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit, ...seen }),
  });
  t.after(() => queue.dispose());
  const old = queue.submit(submission(account.origin()));
  account.change("B"); queue.reset();
  assert.equal(await old.completion, "cancelled");
  down = false;
  queue.flush();
  const fresh = queue.submit(submission(account.origin(), "B-nonce"));
  const sent = await settleHuman(h);
  assert.equal(await fresh.completion, "sent");
  assert.equal(sent.frame.args.clientNonce, "B-nonce");
  assert.equal(h.requests.filter((value) => value.frame.method === "sendHumanMessage").length, 1);
  assert.equal(h.requests.some((value) => value.frame.method === "sendPrompt"), false);
  assert.equal(seen.results.length, 1);
});

for (const lateOutcome of ["success", "failure"] as const) {
  test(`reset releases an active attachment wait and queued A work; late ${lateOutcome} cannot affect reused B nonce`, bounded, async (t) => {
    const h = transportHarness(); t.after(() => h.client.dispose());
    const account = accountHarness();
    const attachment = deferred<ScopedComposerSubmission["attachments"]>();
    const oldOrigin = account.origin();
    const seen = projections();
    const phases: string[] = [];
    const failures: unknown[] = [];
    const work: Promise<void>[] = [];
    const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
      isTransportDown: () => false,
      send(input) {
        const result = sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: input.origin === oldOrigin ? () => attachment.promise : identityCommit, ...seen });
        work.push(result);
        return result;
      },
      onPhase: (input) => { if (input.origin.isCurrent()) phases.push(`${input.origin.accountSlot}:${input.nonce}:${input.phase}`); },
      onFailure: (input, error) => { if (input.origin.isCurrent()) failures.push(error); },
    });
    t.after(() => queue.dispose());
    const old = queue.submit(submission(oldOrigin, "reused-nonce"));
    const behind = queue.submit(submission(oldOrigin, "behind-active"));
    account.change("B"); queue.reset();
    assert.deepEqual(await Promise.all([old.completion, behind.completion]), ["cancelled", "cancelled"]);
    const fresh = queue.submit(submission(account.origin(), "reused-nonce"));
    await settleHuman(h);
    assert.equal(await fresh.completion, "sent");
    const settled = phases.slice();
    if (lateOutcome === "success") attachment.resolve([{ path: "/committed/old.png", name: "original.png" }]);
    else attachment.reject(new Error("old copy failure"));
    await assert.rejects(work[0]);
    assert.deepEqual(phases, settled);
    assert.deepEqual(failures, []);
    assert.equal(seen.committed.length, 1);
    assert.equal(seen.results.length, 1);
    assert.equal(seen.rosters.length, 1);
    assert.equal(h.requests.filter((value) => value.frame.method === "sendHumanMessage").length, 1);
    assert.deepEqual(queue.snapshot(), []);
  });
}

test("A B A retirement rejects a copied attachment result and also refuses work before copy begins", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const attachment = deferred<ScopedComposerSubmission["attachments"]>();
  const input = submission(account.origin());
  const seen = projections();
  let copies = 0;
  const send = sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: () => { copies += 1; return attachment.promise; }, ...seen });
  account.change("B"); account.change("A");
  attachment.resolve([{ path: "/committed/copy.png", name: "original.png" }]);
  await assert.rejects(send, retired);
  await assert.rejects(sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: async (values) => { copies += 1; return values; }, ...seen }), retired);
  assert.equal(copies, 1);
  assert.deepEqual(input.attachments, [{ path: "/staging/original.png", name: "original.png" }]);
  assert.deepEqual(seen.committed, []);
  assert.deepEqual(seen.results, []);
  assert.deepEqual(seen.rosters, []);
  assert.deepEqual(h.requests, []);
});

test("actual client cancels an unposted send after readiness crosses A B A without serializing its origin", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  h.disconnect();
  const copied = deferred<void>();
  const input = submission(account.origin());
  const send = sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit, onAttachmentsCommitted: () => copied.resolve() });
  await copied.promise;
  account.change("B"); account.change("A");
  h.connect();
  await assert.rejects(send, retired);
  assert.deepEqual(h.requests, []);
  const fresh = submission(account.origin(), "current-nonce");
  const current = sendScopedComposerSubmission({ client: h.client, submission: fresh, commitAttachments: identityCommit });
  const posted = await settleHuman(h);
  await current;
  assert.deepEqual(posted.frame.args, { conversationId: conversation.id, text: fresh.prompt, clientNonce: "current-nonce", composedAtMs: 1234, attachments: fresh.attachments, replyToId: "human-server-message:7" });
  assert.deepEqual(Object.keys(posted.frame).sort(), ["args", "kind", "method", "requestId"]);
});

for (const boundary of ["logout", "root retirement"] as const) {
  test(`${boundary} fences queued Human send and reaction at the real frame boundary`, bounded, async (t) => {
    const h = transportHarness(); t.after(() => h.client.dispose());
    const account = accountHarness();
    h.disconnect();
    const origin = account.origin();
    const copied = deferred<void>();
    const seen = projections();
    const send = sendScopedComposerSubmission({ client: h.client, submission: submission(origin), commitAttachments: identityCommit, ...seen, onAttachmentsCommitted: () => copied.resolve() });
    const reacted: unknown[] = [];
    const reaction = reactScopedHumanMessage({ client: h.client, origin, input: { agentId: conversation.id, entryId: "human-server-message:7", emoji: "👍" }, onResult: (value) => reacted.push(value) });
    await copied.promise;
    if (boundary === "logout") account.change(null); else account.unmount();
    h.connect();
    await Promise.all([assert.rejects(send, retired), assert.rejects(reaction, retired)]);
    assert.deepEqual(h.requests, []);
    assert.deepEqual(seen.results, []);
    assert.deepEqual(seen.rosters, []);
    assert.deepEqual(reacted, []);
  });
}

test("reaction uses the original Human entry through reconnect and navigation but ignores a retired posted result", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const origin = account.origin();
  const results: unknown[] = [];
  let activeConversation = conversation.id;
  h.disconnect();
  const reaction = reactScopedHumanMessage({ client: h.client, origin, input: { agentId: activeConversation, entryId: "human-server-message:7", emoji: "👍" }, onResult: (value, owner) => results.push({ value, owner }) });
  activeConversation = "another-conversation";
  h.connect();
  const posted = await h.request("reactHumanMessage");
  assert.deepEqual(posted.frame.args, { conversationId: conversation.id, entryId: "human-server-message:7", emoji: "👍" });
  h.reply(posted, { id: "human-server-message:7", reactions: ["👍"] });
  await reaction;
  assert.equal(activeConversation, "another-conversation");
  assert.equal(results.length, 1);
  const late = reactScopedHumanMessage({ client: h.client, origin, input: { agentId: conversation.id, entryId: "human-server-message:7", emoji: "❤️" }, onResult: (value) => results.push(value) });
  const lateFrame = await h.request("reactHumanMessage", 1);
  account.change("B");
  h.reply(lateFrame, { id: "human-server-message:7", reactions: ["❤️"] });
  await late;
  assert.equal(results.length, 1);
  assert.equal(h.requests.length, 2);
});

test("same-pair reaction waiting for readiness cannot cross A B A before its frame", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const results: unknown[] = [];
  h.disconnect();
  const action = { agentId: conversation.id, entryId: "human-server-message:7", emoji: "👍" };
  const old = reactScopedHumanMessage({ client: h.client, origin: account.origin(), input: action, onResult: (value) => results.push(value) });
  account.change("B"); account.change("A"); h.connect();
  await assert.rejects(old, retired);
  assert.deepEqual(h.requests, []);
  const current = reactScopedHumanMessage({ client: h.client, origin: account.origin(), input: action, onResult: (value) => results.push(value) });
  const posted = await h.request("reactHumanMessage");
  h.reply(posted, { id: action.entryId });
  await current;
  assert.deepEqual(posted.frame.args, { conversationId: conversation.id, entryId: action.entryId, emoji: action.emoji });
  assert.equal(results.length, 1);
});

for (const outcome of ["success", "failure"] as const) {
  test(`posted Human ${outcome} survives queue retirement without cancellation or new-account callbacks`, bounded, async (t) => {
    const h = transportHarness(); t.after(() => h.client.dispose());
    const account = accountHarness();
    const seen = projections();
    const failures: unknown[] = [];
    const work: Promise<void>[] = [];
    const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
      isTransportDown: () => false,
      send(input) { const pending = sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit, ...seen }); work.push(pending); return pending; },
      onFailure: (_input, error) => failures.push(error),
    });
    t.after(() => queue.dispose());
    const old = queue.submit(submission(account.origin()));
    const posted = await h.request("sendHumanMessage");
    account.change("B"); queue.reset();
    assert.equal(await old.completion, "cancelled");
    if (outcome === "success") { h.reply(posted, { id: "durable-original-message" }); await work[0]; }
    else { h.fail(posted); await assert.rejects(work[0]); }
    assert.deepEqual(seen.results, []);
    assert.deepEqual(seen.rosters, []);
    assert.deepEqual(failures, []);
    assert.equal(h.requests.length, 1);
    assert.equal(h.requests[0], posted);
    assert.equal(h.frames.some((value) => typeof value === "object" && value != null && "method" in value && String(value.method).toLowerCase().includes("cancel")), false);
  });
}

test("a delayed Human roster cannot publish across A B A, and a projection retirement prevents the read", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const seen = projections();
  const send = sendScopedComposerSubmission({ client: h.client, submission: submission(account.origin()), commitAttachments: identityCommit, ...seen });
  h.reply(await h.request("sendHumanMessage"), { id: "first" });
  const roster = await h.request("listHumanConversations");
  account.change("B"); account.change("A");
  h.reply(roster, ["OLD_PRIVATE_ROSTER"]);
  await send;
  assert.equal(seen.results.length, 1);
  assert.deepEqual(seen.rosters, []);
  const second = sendScopedComposerSubmission({ client: h.client, submission: submission(account.origin(), "second"), commitAttachments: identityCommit, onHumanResult: () => account.change("B"), ...{ onHumanRoster: seen.onHumanRoster } });
  h.reply(await h.request("sendHumanMessage", 1), { id: "second" });
  await second;
  assert.equal(h.requests.filter((value) => value.frame.method === "listHumanConversations").length, 1);
  assert.deepEqual(seen.rosters, []);
});

test("a follow-up roster waiting for readiness rechecks the original account at its own frame", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const projected = deferred<void>();
  const rosters: unknown[] = [];
  const send = sendScopedComposerSubmission({
    client: h.client, submission: submission(account.origin()), commitAttachments: identityCommit,
    onHumanResult: () => { h.disconnect(); projected.resolve(); },
    onHumanRoster: (_input, value) => rosters.push(value),
  });
  h.reply(await h.request("sendHumanMessage"), { id: "accepted-original-message" });
  await projected.promise;
  account.change("B"); h.connect();
  await assert.rejects(send, retired);
  assert.deepEqual(h.requests.map((value) => value.frame.method), ["sendHumanMessage"]);
  assert.deepEqual(rosters, []);
});

test("same-account offline queue and conversation navigation retain original Human nonces and lane", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  let down = true;
  let visibleConversation = conversation.id;
  const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
    isTransportDown: () => down,
    send: (input) => sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit }),
  });
  t.after(() => queue.dispose());
  const old = submission(account.origin(), "same-account-nonce");
  const pending = queue.submit(old);
  visibleConversation = "agent-now-visible";
  h.disconnect(); h.connect();
  down = false; queue.flush();
  const posted = await settleHuman(h);
  assert.equal(await pending.completion, "sent");
  assert.equal(visibleConversation, "agent-now-visible");
  assert.equal(posted.frame.args.clientNonce, "same-account-nonce");
  assert.equal(posted.frame.args.conversationId, conversation.id);
  assert.equal(h.requests.some((value) => value.frame.method === "sendPrompt"), false);
});

test("Agent send and failed-send retry preserve wire fields and successful approval cleanup", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  let clears = 0;
  const failures: ScopedComposerSubmission[] = [];
  const lifecycle = createSendJournalApprovalLifecycle<ScopedComposerSubmission, { submission: ScopedComposerSubmission; onJournaled?: (event: string) => void }, string, string, string, void, ComposerSubmissionPhase, void, boolean>({
    sendPrompt: (input) => sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit }),
    resendFailed: async (input) => { const result = queue.submit(input.submission); const phase = await result.completion; if (phase === "sent") input.onJournaled?.(input.submission.nonce); return phase; },
    deleteFailed: async (nonce) => queue.discard(nonce),
    cancelQueued: async (nonce) => queue.cancelQueued(nonce),
  }, { clearApprovals: async () => { clears += 1; } });
  let resending = false;
  const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
    isTransportDown: () => false,
    send: (input) => resending
      ? sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit })
      : lifecycle.sendPrompt(input),
    onFailure: (input) => failures.push(input),
  });
  t.after(() => { queue.dispose(); lifecycle.dispose(); });
  const first = { ...submission(account.origin(false), "agent-failed"), agentId: "agent-one", richText: '{"type":"doc"}', isFork: true };
  const failed = queue.submit(first);
  h.fail(await h.request("sendPrompt"));
  assert.equal(await failed.completion, "failed");
  assert.equal(clears, 0);
  assert.equal(failures[0].origin, first.origin);
  assert.deepEqual(failures[0].attachments, first.attachments);
  queue.discard(first.nonce);
  const retried = { ...first, nonce: "fresh-agent-retry", origin: account.origin(false) };
  resending = true;
  const retry = lifecycle.resendFailed({ submission: retried });
  const posted = await h.request("sendPrompt", 1);
  assert.deepEqual(posted.frame.args, { agentId: "agent-one", prompt: first.prompt, directAddressedAcceptance: true, attachmentPaths: ["/staging/original.png"], attachmentNames: ["original.png"], clientNonce: "fresh-agent-retry", enterEpochMs: 1234, composedAtMs: 1234, richText: '{"type":"doc"}', replyToId: "human-server-message:7", isFork: true });
  h.reply(posted, {});
  assert.equal(await retry, "sent");
  assert.equal(clears, 1);
  assert.equal(h.requests.some((value) => value.frame.method === "sendHumanMessage"), false);
  assert.equal(h.requests.some((value) => value.frame.method === "listHumanConversations"), false);
});

test("account retirement fences approval cleanup for an Agent request already posted", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  let clears = 0;
  const lifecycle = createSendJournalApprovalLifecycle<ScopedComposerSubmission, { onJournaled?: (event: string) => void }, string, string, string>({
    sendPrompt: (input) => sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: identityCommit }),
    resendFailed: async () => {}, deleteFailed: async () => {}, cancelQueued: async () => {},
  }, { clearApprovals: async () => { clears += 1; } });
  t.after(() => lifecycle.dispose());
  const pending = lifecycle.sendPrompt({ ...submission(account.origin(false)), agentId: "agent-one" });
  const posted = await h.request("sendPrompt");
  account.change("B"); lifecycle.reset();
  h.reply(posted, {});
  await pending;
  assert.equal(clears, 0);
  assert.equal(h.requests.length, 1);
});

test("retiring a copied attachment send preserves the real saved draft and original staged bytes for a later account restore", bounded, async (t) => {
  const h = transportHarness(); t.after(() => h.client.dispose());
  const account = accountHarness();
  const saved = new Map<string, string>();
  const files = new Map<string, string>([["/staging/original.png", "original-private-bytes"]]);
  const persistence = {
    read: async (slot: string) => saved.get(slot) ?? null,
    write: async (slot: string, value: ComposerDraftEnvelope) => { saved.set(slot, JSON.stringify({ schemaVersion: 1, value })); },
    clear: async (slot: string) => { saved.delete(slot); },
  };
  const store = createComposerDraftStateStore(persistence);
  await store.restore("A");
  const old = submission(account.origin());
  store.setDraft(old.agentId, { prompt: old.prompt, attachments: [...old.attachments] });
  const copied = deferred<void>();
  const copyStarted = deferred<void>();
  const seen = projections();
  let discardCalls = 0;
  const bridge = {
    commitStagedAttachments: async (paths: readonly string[], names: readonly string[]) => {
      copyStarted.resolve();
      await copied.promise;
      return paths.map((path, index) => {
        const bytes = files.get(path); assert.ok(bytes);
        const target = `/committed/${names[index]}`; files.set(target, bytes); return target;
      });
    },
    discardStagedAttachment: async (path: string) => { discardCalls += 1; files.delete(path); },
  };
  const work: Promise<void>[] = [];
  const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
    isTransportDown: () => false,
    send(input) {
      const pending = sendScopedComposerSubmission({ client: h.client, submission: input, commitAttachments: (values) => commitComposerAttachments(bridge, values), ...seen });
      work.push(pending); return pending;
    },
  });
  t.after(() => queue.dispose());
  const pending = queue.submit(old);
  await copyStarted.promise;
  account.unmount(); queue.reset(); store.dispose();
  assert.equal(await pending.completion, "cancelled");
  copied.resolve();
  await assert.rejects(work[0], retired);
  assert.equal(discardCalls, 0);
  assert.equal(files.get("/staging/original.png"), "original-private-bytes");
  assert.deepEqual(h.requests, []);
  assert.deepEqual(seen.committed, []);
  const restored = createComposerDraftStateStore(persistence); t.after(() => restored.dispose());
  await restored.restore("A");
  const draft = restored.snapshotsFor(old.agentId).get().draft;
  assert.ok(draft);
  assert.equal(draft.prompt, old.prompt);
  assert.deepEqual(draft.attachments, old.attachments);
  assert.equal(files.get(draft.attachments[0].path), "original-private-bytes");
  const newAccount = accountHarness();
  const fresh = { ...old, origin: newAccount.origin(), nonce: "restored-account-retry", attachments: draft.attachments };
  const retry = queue.submit(fresh);
  const posted = await settleHuman(h);
  assert.equal(await retry.completion, "sent");
  assert.deepEqual(posted.frame.args.attachments, [{ path: "/committed/original.png", name: "original.png" }]);
  assert.equal(files.get("/staging/original.png"), "original-private-bytes");
  assert.equal(discardCalls, 0);
});
