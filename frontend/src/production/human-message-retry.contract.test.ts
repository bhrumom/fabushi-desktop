import assert from "node:assert/strict";
import test from "node:test";
import type { TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge.ts";
import { createTranscriptAcknowledgementController } from "../recovered/features/conversation/workspace/acknowledgement.ts";
import type { ConversationTranscriptEntry, TranscriptMessage } from "../recovered/features/conversation/workspace/model.ts";
import { createRosterSelectionStore } from "../recovered/features/roster/selection-state.ts";
import { createCoordinatorClient, type ProductionCoordinatorClient } from "./coordinator-client.ts";
import { createHumanMessageRetryController, type HumanMessageRetryScope } from "./human-message-retry.ts";
import { projectTranscriptEntry } from "./model.ts";
import { createComposerSubmissionQueue, type ComposerSubmission } from "../recovered/features/conversation/workspace/submission.ts";
import { sendHumanComposerSubmission } from "./human-message-submission.ts";
import { mergeAuthoritativeTranscriptMessage, reconcileTranscriptTail } from "./transcript-projection.ts";

// HRR-05/06, HRR-AC-03: exercise the controller mounted by ProductionRenderer,
// the actual transcript projector/acknowledgement/selection owners and client.
// These cases are authored here; executable evidence comes only from Actions.
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

const localRaw = {
  kind: "message", role: "user", id: "human-message:original-nonce", authorKind: "human", authorId: "local-human",
  clientNonce: "original-nonce", content: "Original text", timestampMs: 100, composedAtMs: 90,
  replyToId: "human-message:parent", attachments: [{ path: "/committed/file.txt", name: "file.txt" }],
  delivery: "pending", lastDispatchError: "ambiguous dispatch",
};
const sentRaw = {
  ...localRaw, id: "human-server-message:51", remoteMessageId: "51", delivery: "sent",
  content: "Durable Host text", replyToId: "human-server-message:50", lastDispatchError: undefined,
  attachments: [{ path: "/resource-cache/resource-51/file.txt", name: "file.txt" }],
};

function project(raw: unknown): TranscriptMessage {
  const entry = projectTranscriptEntry(raw, 0, "Human chat", "human-a");
  assert.ok(entry?.kind === "message");
  return entry;
}

type Rows = Record<string, ConversationTranscriptEntry[]>;
type Client = Pick<ProductionCoordinatorClient, "call">;

function harness({ client, deferUpdates = false }: { client?: Client; deferUpdates?: boolean } = {}) {
  const selection = createRosterSelectionStore({
    read: async () => ({ kind: "absent" }), write: async () => {}, clear: async () => {},
  });
  selection.select("human-a");
  selection.settle("human-a");
  let accountGeneration = 1;
  let selectionGeneration = 1;
  let accountSlot = "account-a";
  let signedIn = true;
  const readScope = (): HumanMessageRetryScope | null => signedIn ? {
    accountGeneration, selectionGeneration, accountSlot, selection: selection.get(),
  } : null;
  const failed = project(localRaw);
  const acknowledgement = createTranscriptAcknowledgementController();
  acknowledgement.setScope(accountSlot, "human-a");
  acknowledgement.insertOptimistic({
    accountSlot, agentId: "human-a", nonce: failed.clientNonce!, phase: "failed", failedAtMs: 101,
    entries: [{ id: failed.id, kind: "message", clientNonce: failed.clientNonce! }],
  });
  let rows: Rows = { "human-a": [failed] };
  let notice: string | null = "Previous failure";
  const rowUpdates: Array<(current: Rows) => Rows> = [];
  const noticeUpdates: Array<(current: string | null) => string | null> = [];
  const requests: Array<{
    method: string;
    args: unknown;
    admission: Parameters<Client["call"]>[2];
    result: ReturnType<typeof deferred<unknown>>;
  }> = [];
  const authoritativeEntries: TranscriptMessage[] = [];
  const controller = createHumanMessageRetryController({
    client: client ?? {
      call(method, args, admission) {
        const result = deferred<unknown>();
        requests.push({ method, args, admission, result });
        return result.promise;
      },
    },
    readScope,
    acknowledgement,
    setEntries(update) {
      if (deferUpdates) rowUpdates.push(update);
      else rows = update(rows);
    },
    setNotice(update) {
      if (deferUpdates) noticeUpdates.push(update);
      else notice = update(notice);
    },
    onAuthoritativeEntry(_scope, _conversationId, entry, isCurrent) {
      assert.equal(isCurrent(), true);
      authoritativeEntries.push(entry);
    },
  });
  const input = (entry = failed, scope = readScope()!) => ({
    scope, conversationId: scope.selection.currentAgentId!, conversationName: "Human chat", entry,
  });
  return {
    controller, selection, acknowledgement, failed, requests, authoritativeEntries, rowUpdates, noticeUpdates, input, readScope,
    get rows() { return rows; },
    get notice() { return notice; },
    install(nextRows: Rows, nextNotice: string | null = notice) { rows = nextRows; notice = nextNotice; },
    select(id: string) {
      selectionGeneration += 1;
      selection.select(id);
      selection.settle(id);
      acknowledgement.setScope(accountSlot, id);
    },
    account(slot: string) {
      accountGeneration += 1;
      accountSlot = slot;
      signedIn = true;
      acknowledgement.reset();
      acknowledgement.setScope(slot, selection.get().currentAgentId);
    },
    logout() { accountGeneration += 1; signedIn = false; acknowledgement.reset(); },
    layoutUnmount() { accountGeneration += 1; },
    flush() {
      while (rowUpdates.length > 0) rows = rowUpdates.shift()!(rows);
      while (noticeUpdates.length > 0) notice = noticeUpdates.shift()!(notice);
    },
    dispose() { controller.dispose(); selection.dispose(); acknowledgement.dispose(); },
  };
}

test("HRR-05 retry sends only the original identities and settles through the existing projection and acknowledgement", async () => {
  const h = harness();
  const original = structuredClone(h.failed);
  const acknowledgementBefore = h.acknowledgement.getSnapshot().records;
  const attempt = h.controller.retry(h.input());
  await Promise.resolve();
  assert.equal(h.requests.length, 1);
  assert.equal(h.requests[0].method, "retryHumanMessage");
  assert.deepEqual(h.requests[0].args, { conversationId: "human-a", clientNonce: "original-nonce" });
  assert.equal(h.requests[0].admission?.isCurrent?.(), true);
  assert.deepEqual(h.rows["human-a"], [{ ...original, delivery: "pending" }]);
  assert.deepEqual(h.acknowledgement.getSnapshot().records, acknowledgementBefore);
  assert.deepEqual(h.failed, original);
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  assert.equal(h.rows["human-a"].length, 1);
  assert.equal(h.notice, null);
  assert.equal(h.acknowledgement.getSnapshot().records.length, 0);
  assert.equal(h.authoritativeEntries.length, 1);
  h.dispose();
});

test("HRR-05 canonical retry replaces committed attachment paths with the complete Host resource collection", async () => {
  for (const optimistic of [false, true]) {
    const h = harness();
    const local: TranscriptMessage = optimistic
      ? { ...h.failed, id: "pending-original-nonce", authorKind: undefined, authorId: undefined }
      : h.failed;
    h.install({ "human-a": [local] });
    const attempt = h.controller.retry(h.input(local));
    await Promise.resolve();
    assert.deepEqual((h.rows["human-a"][0] as TranscriptMessage).attachments, localRaw.attachments);
    h.requests[0].result.resolve(sentRaw);
    await attempt;
    assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
    const settled = h.rows["human-a"][0] as TranscriptMessage;
    assert.equal(settled.attachments?.length, 1);
    assert.equal(settled.attachments?.[0].path, "/resource-cache/resource-51/file.txt");
    assert.equal(settled.attachments?.some((attachment) => attachment.path === "/committed/file.txt"), false);
    assert.deepEqual(local.attachments, localRaw.attachments);
    const lateLocalRead = reconcileTranscriptTail(h.rows["human-a"], [local], new Set());
    assert.deepEqual(lateLocalRead, [project(sentRaw)]);
    h.dispose();
  }
});

test("HRR-05 omitted or empty canonical retry attachments clear the local collection and stay empty after stale reads", async () => {
  const withoutAttachments: Record<string, unknown> = { ...sentRaw };
  delete withoutAttachments.attachments;
  for (const result of [withoutAttachments, { ...withoutAttachments, attachments: [] }]) {
    const h = harness({ client: { call: async () => result } });
    await h.controller.retry(h.input());
    const settled = h.rows["human-a"][0] as TranscriptMessage;
    assert.equal(settled.id, sentRaw.id);
    assert.equal(settled.delivery, "sent");
    assert.deepEqual(settled.attachments, []);
    assert.deepEqual(h.failed.attachments, localRaw.attachments);
    assert.equal(h.acknowledgement.getSnapshot().records.length, 0);
    const lateLocalRead = reconcileTranscriptTail(h.rows["human-a"], [h.failed], new Set());
    assert.equal(lateLocalRead.length, 1);
    assert.equal(lateLocalRead[0].id, sentRaw.id);
    assert.deepEqual((lateLocalRead[0] as TranscriptMessage).attachments, []);
    h.dispose();
  }
});

test("HRR-05 Human attachment replacement preserves Agent incremental and missing-attachment merges", () => {
  const local = project({ ...localRaw, authorKind: "agent", id: "pending-original-nonce" });
  const incoming = project({ ...sentRaw, authorKind: "agent", id: "agent-accepted" });
  const merged = mergeAuthoritativeTranscriptMessage(local, incoming);
  assert.deepEqual(merged.attachments, [...localRaw.attachments, ...sentRaw.attachments]);
  const withoutAttachments = { ...incoming };
  delete withoutAttachments.attachments;
  assert.deepEqual(mergeAuthoritativeTranscriptMessage(local, withoutAttachments).attachments, localRaw.attachments);
});

test("HRR-05 canonical numeric remote identity is projected without changing the authoritative response", async () => {
  const result = { ...sentRaw, remoteMessageId: 51 };
  const original = structuredClone(result);
  const h = harness({ client: { call: async () => result } });
  await h.controller.retry(h.input());
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  assert.equal((h.rows["human-a"][0] as TranscriptMessage).remoteMessageId, "51");
  assert.deepEqual(result, original);
  assert.equal(h.acknowledgement.getSnapshot().records.length, 0);
  h.dispose();
});

test("HRR-05 rapid duplicate clicks share one pending UI attempt before and after dispatch", async () => {
  const h = harness();
  const first = h.controller.retry(h.input());
  assert.equal(h.controller.retry(h.input()), first);
  await Promise.resolve();
  assert.equal(h.controller.retry(h.input()), first);
  assert.equal(h.requests.length, 1);
  h.requests[0].result.resolve(sentRaw);
  await first;
  h.dispose();
});

test("HRR-05 failure retains the same row, reply, attachment and nonce for a subsequent retry", async () => {
  const h = harness();
  const first = h.controller.retry(h.input());
  await Promise.resolve();
  h.requests[0].result.reject(new Error("permission denied"));
  await first;
  assert.deepEqual(h.rows["human-a"], [h.failed]);
  assert.match(h.notice!, /permission denied/);
  assert.equal(h.acknowledgement.getSnapshot().records[0].nonce, "original-nonce");
  const second = h.controller.retry(h.input());
  assert.notEqual(second, first);
  await Promise.resolve();
  assert.equal(h.requests.length, 2);
  assert.deepEqual(h.requests[1].args, h.requests[0].args);
  h.requests[1].result.resolve(sentRaw);
  await second;
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  h.dispose();
});

test("HRR-05 synchronous client failure releases the attempt for retry", async () => {
  let fail = true;
  let calls = 0;
  const h = harness({ client: { call() {
    calls += 1;
    if (fail) throw new Error("synchronous refusal");
    return Promise.resolve(sentRaw);
  } } });
  await h.controller.retry(h.input());
  assert.deepEqual(h.rows["human-a"], [h.failed]);
  assert.match(h.notice!, /synchronous refusal/);
  fail = false;
  await h.controller.retry(h.input());
  assert.equal(calls, 2);
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  h.dispose();
});

test("HRR-05 malformed or mismatched settlements retain failure and cannot reconcile an echo", async () => {
  const malformed = [
    null, [], {}, { ...sentRaw, kind: "notice" }, { ...sentRaw, role: "assistant" },
    { ...sentRaw, authorKind: "agent" }, { ...sentRaw, authorId: "other-human" },
    { ...sentRaw, clientNonce: "replacement" }, { ...sentRaw, id: "unrelated-entry" },
    { ...sentRaw, conversationId: "human-b" }, { ...sentRaw, agentId: "human-b" },
    { ...sentRaw, content: undefined }, { ...sentRaw, timestampMs: undefined },
    { ...sentRaw, timestampMs: Infinity }, { ...sentRaw, remoteMessageId: null },
    { ...sentRaw, remoteMessageId: 0 }, { ...sentRaw, remoteMessageId: -51 },
    { ...sentRaw, remoteMessageId: 51.5 }, { ...sentRaw, remoteMessageId: Number.MAX_SAFE_INTEGER + 1 },
    { ...sentRaw, remoteMessageId: "051" }, { ...sentRaw, remoteMessageId: " 51" },
    { ...sentRaw, remoteMessageId: "51.0" }, { ...sentRaw, remoteMessageId: "remote" },
    { ...sentRaw, delivery: "pending" }, { ...localRaw, delivery: "failed" },
    { ...localRaw, id: "pending-original-nonce" },
  ];
  for (const value of malformed) {
    const h = harness({ client: { call: async () => value } });
    await h.controller.retry(h.input());
    assert.deepEqual(h.rows["human-a"], [h.failed]);
    assert.match(h.notice!, /retryHumanMessage returned/);
    assert.equal(h.acknowledgement.getSnapshot().records.length, 1);
    assert.equal(h.authoritativeEntries.length, 0);
    h.dispose();
  }
});

test("HRR-05 a current pending or dispatching Host result is not falsely acknowledged as sent", async () => {
  for (const delivery of ["pending", "dispatching"]) {
    const h = harness({ client: { call: async () => ({ ...localRaw, delivery, lastDispatchError: undefined }) } });
    await h.controller.retry(h.input());
    assert.equal((h.rows["human-a"][0] as TranscriptMessage).delivery, "pending");
    assert.equal(h.acknowledgement.getSnapshot().records.length, 1);
    assert.equal(h.notice, null);
    h.dispose();
  }
  const h = harness({ client: { call: async () => localRaw } });
  await h.controller.retry(h.input());
  assert.deepEqual(h.rows["human-a"], [h.failed]);
  assert.match(h.notice!, /ambiguous dispatch/);
  assert.equal(h.acknowledgement.getSnapshot().records.length, 1);
  h.dispose();
});

test("HRR-05 an original optimistic Human failure retries without reading projected content or attachment paths", async () => {
  const h = harness();
  const optimistic: TranscriptMessage = { ...h.failed, id: "pending-original-nonce", authorKind: undefined, authorId: undefined };
  h.install({ "human-a": [optimistic] });
  const clicked = { ...optimistic };
  for (const field of ["text", "attachments", "replyToId", "composedAtMs"] as const) {
    Object.defineProperty(clicked, field, { get() { assert.fail(`Retry read projected ${field}`); } });
  }
  const attempt = h.controller.retry(h.input(clicked));
  await Promise.resolve();
  assert.deepEqual(h.requests[0].args, { conversationId: "human-a", clientNonce: "original-nonce" });
  assert.deepEqual(h.rows["human-a"], [{ ...optimistic, delivery: "pending" }]);
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  h.dispose();
});

test("HRR-05 invalid local identities never dispatch or mint replacement identities", async () => {
  const mutations: Partial<TranscriptMessage>[] = [
    { clientNonce: undefined }, { clientNonce: "" }, { clientNonce: " padded" },
    { clientNonce: "é".repeat(101), id: `human-message:${"é".repeat(101)}` },
    { role: "assistant" }, { id: "arbitrary-message" }, { authorId: "" },
    { authorId: " local-human" }, { remoteMessageId: "51" }, { delivery: "sent" },
  ];
  for (const change of mutations) {
    const h = harness();
    await h.controller.retry(h.input({ ...h.failed, ...change }));
    assert.equal(h.requests.length, 0);
    assert.deepEqual(h.rows["human-a"], [h.failed]);
    assert.equal(h.acknowledgement.getSnapshot().records[0].nonce, "original-nonce");
    h.dispose();
  }
});

test("HRR-06 changing conversation while retry awaits preserves the replacement view", async () => {
  const h = harness();
  const attempt = h.controller.retry(h.input());
  await Promise.resolve();
  h.select("human-b");
  const replacement: Rows = { "human-b": [project({ ...sentRaw, id: "human-server-message:71", remoteMessageId: "71" })] };
  h.install(replacement, "B notice");
  assert.equal(h.requests[0].admission?.isCurrent?.(), false);
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  assert.equal(h.rows, replacement);
  assert.equal(h.notice, "B notice");
  assert.equal(h.authoritativeEntries.length, 0);
  h.dispose();
});

test("HRR-06 the actual selection owner fences A to B to A even without a render or open-request increment", async () => {
  const h = harness();
  const staleInput = h.input();
  const attempt = h.controller.retry(staleInput);
  await Promise.resolve();
  h.selection.select("human-b");
  h.selection.select("human-a");
  h.selection.settle("human-a");
  assert.equal(h.readScope()!.selectionGeneration, staleInput.scope.selectionGeneration);
  assert.notEqual(h.readScope()!.selection, staleInput.scope.selection);
  const replacement: Rows = { "human-a": [h.failed] };
  h.install(replacement, "A replacement notice");
  assert.equal(h.controller.retry(h.input()), attempt);
  assert.equal(h.requests.length, 1);
  h.requests[0].result.reject(new Error("departed selection failure"));
  await attempt;
  await h.controller.retry(staleInput);
  assert.equal(h.requests.length, 1);
  assert.equal(h.rows, replacement);
  assert.equal(h.notice, "A replacement notice");
  h.dispose();
});

test("HRR-06 reopening the same selected conversation fences the prior open-request generation", async () => {
  const h = harness();
  const originalSelection = h.selection.get();
  const attempt = h.controller.retry(h.input());
  await Promise.resolve();
  h.select("human-a");
  assert.equal(h.selection.get(), originalSelection);
  const replacement: Rows = { "human-a": [h.failed] };
  h.install(replacement, "new selection request");
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  assert.equal(h.rows, replacement);
  assert.equal(h.notice, "new selection request");
  h.dispose();
});

test("HRR-06 account A to B to A cannot publish a late result into a replacement lifetime", async () => {
  const h = harness();
  const staleInput = h.input();
  const attempt = h.controller.retry(staleInput);
  await Promise.resolve();
  h.account("account-b");
  h.account("account-a");
  const replacement: Rows = { "human-a": [h.failed] };
  h.install(replacement, "replacement account");
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  await h.controller.retry(staleInput);
  assert.equal(h.rows, replacement);
  assert.equal(h.notice, "replacement account");
  assert.equal(h.requests.length, 1);
  const current = h.controller.retry(h.input());
  await Promise.resolve();
  assert.equal(h.requests.length, 2);
  h.requests[1].result.resolve(sentRaw);
  await current;
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  h.dispose();
});

test("HRR-06 logout and layout-unmount fences are immediate, before passive disposal", async () => {
  for (const endLifetime of ["logout", "layoutUnmount"] as const) {
    const h = harness();
    const attempt = h.controller.retry(h.input());
    await Promise.resolve();
    h[endLifetime]();
    const replacement: Rows = {};
    h.install(replacement, "closed view");
    assert.equal(h.requests[0].admission?.isCurrent?.(), false);
    h.requests[0].result.reject(new Error("late disconnect"));
    await attempt;
    assert.equal(h.rows, replacement);
    assert.equal(h.notice, "closed view");
    h.dispose();
  }
});

test("HRR-06 cancellation before the dispatch microtask releases its flight without effects", async () => {
  const h = harness();
  const first = h.controller.retry(h.input());
  h.select("human-b");
  h.select("human-a");
  await first;
  assert.equal(h.requests.length, 0);
  assert.deepEqual(h.rows["human-a"], [h.failed]);
  assert.equal(h.notice, "Previous failure");
  const next = h.controller.retry(h.input());
  await Promise.resolve();
  assert.equal(h.requests.length, 1);
  h.requests[0].result.resolve(sentRaw);
  await next;
  h.dispose();
});

class RetryTestPort implements TransferredCoordinatorPort {
  readonly frames: unknown[] = [];
  private onMessage?: (event: { data: unknown }) => void;
  private requests: Record<string, unknown>[] = [];
  private requestWaiters: ((request: Record<string, unknown>) => void)[] = [];
  postMessage(message: unknown) {
    this.frames.push(message);
    if (typeof message === "object" && message != null && (message as { kind?: unknown }).kind === "request") {
      const next = this.requestWaiters.shift();
      if (next == null) this.requests.push(message as Record<string, unknown>);
      else next(message as Record<string, unknown>);
    }
  }
  nextRequest(): Promise<Record<string, unknown>> {
    return this.requests.length > 0 ? Promise.resolve(this.requests.shift()!)
      : new Promise((resolve) => this.requestWaiters.push(resolve));
  }
  start() {}
  close() {}
  addEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  addEventListener(type: "close", listener: (event: Record<string, never>) => void): void;
  addEventListener(type: "message" | "close", listener: ((event: { data: unknown }) => void) | ((event: Record<string, never>) => void)) {
    if (type === "message") this.onMessage = listener as (event: { data: unknown }) => void;
  }
  receive(data: unknown) { this.onMessage?.({ data }); }
}

test("HRR-07 a possibly posted author-bound optimistic Human row retries through the real durable controller", { timeout: 10_000 }, async () => {
  const firstPort = new RetryTestPort();
  let nextPort = firstPort;
  const client = createCoordinatorClient({ claim: (consumer) => ({ request: () => consumer.onPort(nextPort), release() {} }) });
  assert.ok(client);
  firstPort.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  const submission: ComposerSubmission = { nonce: localRaw.clientNonce, agentId: "human-a", prompt: localRaw.content,
    attachments: localRaw.attachments, createdAtMs: localRaw.composedAtMs, replyToId: localRaw.replyToId };
  const queue = createComposerSubmissionQueue({ isTransportDown: () => false,
    send: async (input) => { await sendHumanComposerSubmission(input, { client, bridge: null, isCurrent: () => true }); } });
  const original = queue.submit(submission);
  const initial = await firstPort.nextRequest();
  assert.equal(initial.method, "sendHumanMessage");
  assert.equal(submission.humanDispatch?.requestPosted, true);
  nextPort = new RetryTestPort();
  firstPort.receive({ kind: "lifecycle", phase: "shutdown" });
  assert.equal(await original.completion, "failed");
  assert.equal(queue.snapshot()[0].humanDispatch?.requestPosted, true);

  const h = harness({ client });
  const optimistic: TranscriptMessage = { ...h.failed, id: "pending-original-nonce" };
  assert.equal(optimistic.authorKind, "human");
  assert.equal(optimistic.authorId, "local-human");
  h.install({ "human-a": [optimistic] });
  const retry = h.controller.retry(h.input(optimistic));
  nextPort.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  const frame = await nextPort.nextRequest();
  assert.equal(frame.method, "retryHumanMessage");
  assert.deepEqual(frame.args, { conversationId: "human-a", clientNonce: "original-nonce" });
  nextPort.receive({ kind: "reply", requestId: frame.requestId, outcome: { status: "ok", value: sentRaw } });
  await retry;
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  assert.equal(h.acknowledgement.getSnapshot().records.length, 0);
  assert.equal(firstPort.frames.filter((frame: any) => frame.kind === "request").length, 1);
  assert.equal(nextPort.frames.filter((frame: any) => frame.kind === "request").length, 1);
  h.dispose(); queue.dispose(); client.dispose();
});

test("HRR-06 a queued-ready production client admits no retry frame after account or selection replacement", async () => {
  for (const replacement of ["account", "selection", "unmount"] as const) {
    const port = new RetryTestPort();
    const client = createCoordinatorClient({ claim: (consumer) => ({ request: () => consumer.onPort(port), release() {} }) });
    assert.ok(client);
    const h = harness({ client });
    const attempt = h.controller.retry(h.input());
    await Promise.resolve();
    assert.equal(port.frames.length, 1); // hello, still awaiting ready
    if (replacement === "account") { h.account("account-b"); h.account("account-a"); }
    else if (replacement === "selection") { h.select("human-b"); h.select("human-a"); }
    else h.layoutUnmount();
    const rows: Rows = { "human-a": [h.failed] };
    h.install(rows, "new lifetime");
    port.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
    await attempt;
    assert.deepEqual(port.frames, [{ kind: "lifecycle", phase: "hello", protocolVersion: 1 }]);
    assert.equal(h.rows, rows);
    assert.equal(h.notice, "new lifetime");
    h.dispose();
    client.dispose();
  }
});

test("HRR-06 deferred success and notice updaters recheck account and selection lifetimes", async () => {
  for (const replacement of ["account", "selection", "unmount"] as const) {
    const h = harness({ deferUpdates: true, client: { call: async () => sentRaw } });
    await h.controller.retry(h.input());
    assert.ok(h.rowUpdates.length > 0);
    if (replacement === "account") { h.account("account-b"); h.account("account-a"); }
    else if (replacement === "selection") { h.select("human-b"); h.select("human-a"); }
    else h.layoutUnmount();
    const rows: Rows = { "human-a": [h.failed] };
    h.install(rows, "replacement notice");
    h.flush();
    assert.equal(h.rows, rows);
    assert.equal(h.notice, "replacement notice");
    h.dispose();
  }
});

test("HRR-06 deferred failure and notice cannot overwrite a later successful retry", async () => {
  const h = harness({ deferUpdates: true });
  const first = h.controller.retry(h.input());
  await Promise.resolve();
  h.requests[0].result.reject(new Error("earlier failure"));
  await first;
  const obsoleteRows = h.rowUpdates.splice(0);
  const obsoleteNotices = h.noticeUpdates.splice(0);
  const second = h.controller.retry(h.input());
  await Promise.resolve();
  h.requests[1].result.resolve(sentRaw);
  await second;
  h.flush();
  const settled = h.rows;
  for (const update of obsoleteRows) assert.equal(update(settled), settled);
  for (const update of obsoleteNotices) assert.equal(update(null), null);
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  assert.equal(h.notice, null);
  h.dispose();
});

test("HRR-06 deferred pending feedback cannot regress a completed current attempt", async () => {
  const h = harness({ deferUpdates: true });
  const attempt = h.controller.retry(h.input());
  await Promise.resolve();
  const pendingUpdate = h.rowUpdates.shift()!;
  const pendingNotice = h.noticeUpdates.shift()!;
  h.requests[0].result.resolve(sentRaw);
  await attempt;
  h.flush();
  const settled = h.rows;
  assert.equal(pendingUpdate(settled), settled);
  assert.equal(pendingNotice("new notice"), "new notice");
  assert.deepEqual(h.rows["human-a"], [project(sentRaw)]);
  h.dispose();
});

test("HRR-05 a canonical transcript event wins over a late retry error and deferred retry feedback", async () => {
  const h = harness({ deferUpdates: true });
  const attempt = h.controller.retry(h.input());
  await Promise.resolve();
  const canonical = project(sentRaw);
  h.controller.observeEntry("human-a", canonical);
  h.acknowledgement.reconcileEcho({ accountSlot: "account-a", agentId: "human-a", nonce: "original-nonce" });
  const rows: Rows = { "human-a": [canonical] };
  h.install(rows, "event notice");
  h.requests[0].result.reject(new Error("obsolete error after server acceptance"));
  await attempt;
  h.flush();
  assert.equal(h.rows, rows);
  assert.equal(h.notice, "event notice");
  assert.equal(h.authoritativeEntries.length, 0);
  h.dispose();
});

test("HRR-05 same nonce in different conversations has separate flights and no cross-view settlement", async () => {
  const h = harness();
  const first = h.controller.retry(h.input());
  await Promise.resolve();
  h.select("human-b");
  h.install({ "human-b": [h.failed] });
  const second = h.controller.retry(h.input());
  await Promise.resolve();
  assert.equal(h.requests.length, 2);
  assert.deepEqual(h.requests.map((request) => request.args), [
    { conversationId: "human-a", clientNonce: "original-nonce" },
    { conversationId: "human-b", clientNonce: "original-nonce" },
  ]);
  h.requests[1].result.resolve(sentRaw);
  await second;
  const settled = h.rows;
  h.requests[0].result.reject(new Error("A failed after B settled"));
  await first;
  assert.equal(h.rows, settled);
  assert.equal(h.notice, null);
  h.dispose();
});

test("HRR-06 disposal drops late results, errors and deferred updaters without another request", async () => {
  for (const reject of [false, true]) {
    const h = harness({ deferUpdates: true });
    const input = h.input();
    const attempt = h.controller.retry(input);
    await Promise.resolve();
    const rows = h.rows;
    h.controller.dispose();
    if (reject) h.requests[0].result.reject(new Error("disposed failure"));
    else h.requests[0].result.resolve(sentRaw);
    await attempt;
    h.flush();
    await h.controller.retry(input);
    assert.equal(h.requests.length, 1);
    assert.equal(h.rows, rows);
    assert.equal(h.notice, "Previous failure");
    assert.equal(h.authoritativeEntries.length, 0);
    h.dispose();
  }
});

test("HRR-05 persisted pending Human errors expose Retry while preserving all projected content", () => {
  const original = structuredClone(localRaw);
  const failed = project(localRaw);
  assert.equal(failed.delivery, "failed");
  assert.equal(failed.id, localRaw.id);
  assert.equal(failed.clientNonce, localRaw.clientNonce);
  assert.equal(failed.text, localRaw.content);
  assert.equal(failed.replyToId, localRaw.replyToId);
  assert.equal(failed.composedAtMs, localRaw.composedAtMs);
  assert.deepEqual(failed.attachments, localRaw.attachments);
  assert.deepEqual(localRaw, original);
  assert.equal(project({ ...localRaw, delivery: "dispatching" }).delivery, "pending");
  assert.equal(project({ ...localRaw, delivery: "sent" }).delivery, "sent");
  assert.equal(project({ ...sentRaw, lastDispatchError: "stale error" }).delivery, "sent");
  assert.deepEqual(project({ ...sentRaw, remoteMessageId: 51 }), project(sentRaw));
  assert.equal(project({ ...localRaw, delivery: undefined, status: "pending" }).delivery, "failed");
});

test("HRR-05 malformed, foreign, canonical and Agent shapes do not acquire Human Retry from error metadata", () => {
  const shapes: Record<string, unknown>[] = [
    { ...localRaw, authorKind: "agent" }, { ...localRaw, authorKind: undefined },
    { ...localRaw, authorId: "" }, { ...localRaw, authorId: " padded" },
    { ...localRaw, role: "assistant" }, { ...localRaw, clientNonce: "" },
    { ...localRaw, clientNonce: " padded", id: "human-message: padded" },
    { ...localRaw, clientNonce: "é".repeat(101), id: `human-message:${"é".repeat(101)}` },
    { ...localRaw, id: "arbitrary-message" }, { ...localRaw, remoteMessageId: "51" },
    { ...localRaw, remoteMessageId: null }, { ...localRaw, id: "human-server-message:51" },
    { ...localRaw, content: undefined, text: "legacy Agent shape" },
    { ...localRaw, lastDispatchError: "" }, { ...localRaw, lastDispatchError: "  " },
    { ...localRaw, lastDispatchError: { message: "not an error string" } },
    { ...localRaw, kind: "attachment", path: "/saved/file.txt", name: "file.txt" },
    { ...localRaw, kind: "user-attachment", file_path: "/saved/file.txt", file_name: "file.txt" },
  ];
  for (const shape of shapes) assert.equal(project(shape).delivery, "pending");
  for (const delivery of ["queued", "failed", "sent"] as const) {
    assert.equal(project({ ...localRaw, authorKind: "agent", delivery }).delivery, delivery);
  }
});
