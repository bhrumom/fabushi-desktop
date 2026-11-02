import assert from "node:assert/strict";
import test from "node:test";

import type { CoordinatorPortBridge, DesktopBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
import { createComposerSubmissionQueue, composerSubmissionBusyState, type ComposerSubmission, type ComposerSubmissionPhase,
  type ComposerSubmissionRecord } from "../recovered/features/conversation/workspace/submission";
import { createCoordinatorClient, COORDINATOR_PROTOCOL_VERSION } from "./coordinator-client";
import { sendHumanComposerSubmission } from "./human-message-submission";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

type RequestFrame = { kind: "request"; requestId: string; method: string; args: Record<string, unknown> };

function portFixture() {
  let consumer: { onPort(port: TransferredCoordinatorPort): void } | null = null;
  const bridge: CoordinatorPortBridge = {
    claim(next) { consumer = next; return { request() {}, release() { consumer = null; } }; },
  };
  const client = createCoordinatorClient(bridge)!;
  assert.ok(client);
  const connect = ({ throwRequest = false }: { throwRequest?: boolean } = {}) => {
    const requests: RequestFrame[] = [];
    const waiting: ((frame: RequestFrame) => void)[] = [];
    const unread: RequestFrame[] = [];
    const listeners = new Map<string, (event: any) => void>();
    const port: TransferredCoordinatorPort = {
      close() {}, start() {},
      addEventListener(type: string, listener: (event: any) => void) { listeners.set(type, listener); },
      postMessage(value) {
        const frame = value as RequestFrame;
        if (frame.kind !== "request") return;
        requests.push(frame);
        const next = waiting.shift();
        if (next == null) unread.push(frame); else next(frame);
        if (throwRequest) throw new Error("port post failed after admission boundary");
      },
    };
    consumer!.onPort(port);
    const emit = (value: unknown) => listeners.get("message")!({ data: value });
    return {
      requests,
      nextRequest: () => unread.length > 0 ? Promise.resolve(unread.shift()!)
        : new Promise<RequestFrame>((resolve) => waiting.push(resolve)),
      ready: () => emit({ kind: "lifecycle", phase: "ready", protocolVersion: COORDINATOR_PROTOCOL_VERSION }),
      disconnect: () => listeners.get("close")!({}),
      reply: (request: RequestFrame, value: unknown) => emit({ kind: "reply", requestId: request.requestId,
        outcome: { status: "ok", value } }),
      fail: (request: RequestFrame) => emit({ kind: "reply", requestId: request.requestId,
        outcome: { status: "failed", failure: { code: "internal", message: "retained durable failure" } } }),
    };
  };
  return { client, connect };
}

function originalSubmission(attachments: ComposerSubmission["attachments"] = []): ComposerSubmission {
  return {
    nonce: "same-original-nonce", agentId: "human-original-conversation", prompt: "Original body",
    attachments, createdAtMs: 1234, replyToId: "human-server-message:8", richText: "original-rich-text",
  };
}

function canonical(submission: ComposerSubmission) {
  return { id: "human-server-message:9", remoteMessageId: "9", kind: "message", role: "user",
    authorKind: "human", authorId: "1", clientNonce: submission.nonce, delivery: "sent",
    content: "Original body", timestampMs: 1235, replyToId: "human-server-message:8" };
}

function queueFixture(
  client: NonNullable<ReturnType<typeof createCoordinatorClient>>,
  bridge: Pick<DesktopBridge, "commitStagedAttachments"> | null,
) {
  let transportDown = false;
  let ownerCurrent = true;
  const phases: { nonce: string; phase: string }[] = [];
  const failures: unknown[] = [];
  const inputs: ComposerSubmission[] = [];
  const results: unknown[] = [];
  const committed: string[] = [];
  const queue = createComposerSubmissionQueue({
    isTransportDown: () => transportDown,
    send: async (input, admission) => {
      inputs.push(input);
      results.push(await sendHumanComposerSubmission(input, {
        client, bridge,
        isCurrent: () => ownerCurrent && admission?.isCurrent?.() !== false,
        onCommitted: (path) => committed.push(path),
      }));
    },
    onPhase: (input) => phases.push({ nonce: input.nonce, phase: input.phase }),
    onFailure: (_input, error) => failures.push(error),
  });
  return { queue, phases, failures, inputs, results, committed,
    setTransportDown: (value: boolean) => { transportDown = value; },
    setOwnerCurrent: (value: boolean) => { ownerCurrent = value; },
  };
}

test("Human Composer partial commit failure resumes the same queue input and never recommits acknowledged files", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const port = ports.connect(); port.ready();
  const commits: { paths: readonly string[]; names: readonly string[] }[] = [];
  let failSecond = true;
  const f = queueFixture(ports.client, {
    async commitStagedAttachments(paths, names) {
      commits.push({ paths: [...paths], names: [...names] });
      if (paths[0] === "/staged/second" && failSecond) { failSecond = false; return null; }
      return [`/committed/${names[0]}`];
    },
  });
  const input = originalSubmission([{ path: "/staged/first", name: "first.txt" }, { path: "/staged/second", name: "second.txt" }]);
  const initial = f.queue.submit(input);
  assert.equal(await initial.completion, "failed");
  assert.equal(port.requests.length, 0);
  assert.equal(input.humanDispatch?.requestPosted, false);
  assert.deepEqual(input.humanDispatch?.committedAttachments, [{ path: "/committed/first.txt", name: "first.txt" }]);
  assert.deepEqual(f.committed, ["/staged/first"]);
  const retry = f.queue.retryFailed(input.nonce, { isCurrent: () => true });
  assert.ok(retry);
  const duplicate = f.queue.retryFailed(input.nonce, { isCurrent: () => true });
  assert.ok(duplicate);
  assert.equal(duplicate.completion, retry.completion, "same pending input has one queue flight");
  const request = await port.nextRequest();
  assert.equal(request.method, "sendHumanMessage");
  assert.deepEqual(request.args, { conversationId: input.agentId, text: "Original body", clientNonce: input.nonce,
    composedAtMs: 1234, replyToId: "human-server-message:8", attachments: [
      { path: "/committed/first.txt", name: "first.txt" }, { path: "/committed/second.txt", name: "second.txt" },
    ] });
  assert.deepEqual(commits.map((call) => call.paths), [["/staged/first"], ["/staged/second"], ["/staged/second"]]);
  assert.deepEqual(commits.map((call) => call.names), [["first.txt"], ["second.txt"], ["second.txt"]]);
  assert.equal(input.humanDispatch?.requestPosted, true);
  const afterPostDuplicate = f.queue.retryFailed(input.nonce, { isCurrent: () => true });
  assert.ok(afterPostDuplicate);
  assert.equal(afterPostDuplicate.completion, retry.completion, "posting the frame does not open a second queue flight");
  assert.equal(port.requests.length, 1);
  const reply = canonical(input); port.reply(request, reply);
  assert.equal(await retry.completion, "sent");
  assert.equal(await duplicate.completion, "sent");
  assert.equal(f.inputs.length, 2);
  assert.ok(f.inputs.every((value) => value === input), "existing queue retains the original object");
  assert.deepEqual(f.results, [reply]);
  assert.deepEqual(input.attachments, [{ path: "/staged/first", name: "first.txt" }, { path: "/staged/second", name: "second.txt" }]);
  assert.equal(input.richText, "original-rich-text");
  assert.equal(f.queue.retryFailed(input.nonce), null, "settled queue record is retired");
  f.queue.dispose(); ports.client.dispose();
});

test("Human Composer disconnect before the real initial frame retains non-admission and recovers the same first send", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const firstPort = ports.connect();
  const f = queueFixture(ports.client, null); const input = originalSubmission();
  const initial = f.queue.submit(input);
  await Promise.resolve();
  firstPort.disconnect();
  assert.equal(await initial.completion, "failed");
  assert.equal(input.humanDispatch?.requestPosted, false);
  assert.equal(firstPort.requests.length, 0);
  const secondPort = ports.connect(); secondPort.ready();
  const retry = f.queue.retryFailed(input.nonce, { isCurrent: () => true }); assert.ok(retry);
  const request = await secondPort.nextRequest();
  assert.equal(request.method, "sendHumanMessage");
  assert.deepEqual(request.args, { conversationId: input.agentId, text: input.prompt, clientNonce: input.nonce,
    composedAtMs: input.createdAtMs, attachments: [], replyToId: input.replyToId });
  secondPort.reply(request, canonical(input));
  assert.equal(await retry.completion, "sent");
  assert.deepEqual(f.inputs, [input, input]);
  f.queue.dispose(); ports.client.dispose();
});

test("Human Composer lost reply or throwing post cannot downgrade a possibly admitted message to payload resend", { timeout: 10_000 }, async () => {
  for (const cause of ["disconnect", "throw"] as const) {
    const ports = portFixture(); const firstPort = ports.connect({ throwRequest: cause === "throw" }); firstPort.ready();
    let commits = 0;
    const f = queueFixture(ports.client, { async commitStagedAttachments() { commits += 1; return ["/committed/original.txt"]; } });
    const input = originalSubmission([{ path: "/staged/original", name: "original.txt" }]);
    const initial = f.queue.submit(input);
    const firstRequest = await firstPort.nextRequest();
    assert.equal(firstRequest.method, "sendHumanMessage");
    assert.equal(input.humanDispatch?.requestPosted, true, cause);
    if (cause === "disconnect") firstPort.disconnect();
    assert.equal(await initial.completion, "failed");
    if (cause === "throw") firstPort.disconnect();
    const secondPort = ports.connect(); secondPort.ready();
    const retry = f.queue.retryFailed(input.nonce, { isCurrent: () => true }); assert.ok(retry);
    const request = await secondPort.nextRequest();
    assert.equal(request.method, "retryHumanMessage", cause);
    assert.deepEqual(request.args, { conversationId: input.agentId, clientNonce: input.nonce });
    assert.equal(commits, 1, "already committed resources never return to staging");
    secondPort.reply(request, canonical(input));
    assert.equal(await retry.completion, "sent");
    assert.equal(firstPort.requests.length, 1); assert.equal(secondPort.requests.length, 1);
    assert.equal(input.prompt, "Original body"); assert.equal(input.replyToId, "human-server-message:8");
    f.queue.dispose(); ports.client.dispose();
  }
});

test("Human Composer scope expiry during retry commit retains progress without stale queue callbacks or frame", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const port = ports.connect(); port.ready();
  const entered = deferred<void>(); const commit = deferred<string[] | null>();
  let commits = 0;
  const f = queueFixture(ports.client, {
    async commitStagedAttachments() {
      commits += 1;
      if (commits === 1) return null;
      entered.resolve(); return await commit.promise;
    },
  });
  const input = originalSubmission([{ path: "/staged/original", name: "original.txt" }]);
  assert.equal(await f.queue.submit(input).completion, "failed");
  let selection = 1;
  const retry = f.queue.retryFailed(input.nonce, { isCurrent: () => selection === 1 }); assert.ok(retry);
  await entered.promise;
  const phases = [...f.phases]; const failures = [...f.failures];
  selection = 2;
  commit.resolve(["/committed/original.txt"]);
  assert.notEqual(await retry.completion, "sent");
  assert.equal(port.requests.length, 0);
  assert.equal(input.humanDispatch?.requestPosted, false);
  assert.deepEqual(input.humanDispatch?.committedAttachments, [{ path: "/committed/original.txt", name: "original.txt" }]);
  assert.deepEqual(f.committed, [], "obsolete commit reply cannot update renderer staging state");
  assert.deepEqual(f.phases, phases); assert.deepEqual(f.failures, failures);
  const currentRetry = f.queue.retryFailed(input.nonce, { isCurrent: () => selection === 2 }); assert.ok(currentRetry);
  const request = await port.nextRequest();
  assert.equal(request.method, "sendHumanMessage"); assert.equal(commits, 2, "acknowledged commit survived expiry");
  assert.deepEqual(request.args.attachments, [{ path: "/committed/original.txt", name: "original.txt" }]);
  port.reply(request, canonical(input)); assert.equal(await currentRetry.completion, "sent");
  f.queue.dispose(); ports.client.dispose();
});

test("Human Composer offline retry joins its queued flight and expired admission cannot flush a replacement frame", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const port = ports.connect(); port.ready();
  let commits = 0;
  const f = queueFixture(ports.client, {
    async commitStagedAttachments() { commits += 1; return commits === 1 ? null : ["/committed/original.txt"]; },
  });
  const input = originalSubmission([{ path: "/staged/original", name: "original.txt" }]);
  assert.equal(await f.queue.submit(input).completion, "failed");
  f.setTransportDown(true);
  let selection = 1;
  const retry = f.queue.retryFailed(input.nonce, { isCurrent: () => selection === 1 }); assert.ok(retry);
  const duplicate = f.queue.retryFailed(input.nonce, { isCurrent: () => selection === 1 }); assert.ok(duplicate);
  assert.equal(duplicate.completion, retry.completion);
  assert.equal(f.queue.snapshot()[0].phase, "queued"); assert.equal(commits, 1);
  const phases = [...f.phases]; const failures = [...f.failures];
  selection = 2; f.setTransportDown(false); f.queue.flush();
  assert.notEqual(await retry.completion, "sent");
  assert.deepEqual(f.phases, phases); assert.deepEqual(f.failures, failures);
  assert.equal(port.requests.length, 0); assert.equal(commits, 1);
  const currentRetry = f.queue.retryFailed(input.nonce, { isCurrent: () => selection === 2 }); assert.ok(currentRetry);
  const request = await port.nextRequest();
  assert.equal(request.method, "sendHumanMessage"); assert.equal(request.args.clientNonce, input.nonce);
  port.reply(request, canonical(input)); assert.equal(await currentRetry.completion, "sent");
  assert.equal(commits, 2); assert.equal(f.queue.snapshot().length, 0);
  f.queue.dispose(); ports.client.dispose();
});

test("Human Composer durable retry fences reconnect readiness and late settlement without forgetting possible admission", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const firstPort = ports.connect(); firstPort.ready();
  const f = queueFixture(ports.client, null); const input = originalSubmission();
  const initial = f.queue.submit(input); const initialFrame = await firstPort.nextRequest();
  firstPort.fail(initialFrame); assert.equal(await initial.completion, "failed");
  assert.equal(input.humanDispatch?.requestPosted, true);
  firstPort.disconnect(); const nextPort = ports.connect();
  let generation = 1;
  const waiting = f.queue.retryFailed(input.nonce, { isCurrent: () => generation === 1 }); assert.ok(waiting);
  const waitingPhases = [...f.phases]; const waitingFailures = [...f.failures];
  generation = 2; nextPort.ready();
  assert.notEqual(await waiting.completion, "sent");
  assert.equal(nextPort.requests.length, 0);
  assert.deepEqual(f.phases, waitingPhases); assert.deepEqual(f.failures, waitingFailures);
  const late = f.queue.retryFailed(input.nonce, { isCurrent: () => generation === 2 }); assert.ok(late);
  const lateFrame = await nextPort.nextRequest();
  assert.equal(lateFrame.method, "retryHumanMessage");
  assert.deepEqual(lateFrame.args, { conversationId: input.agentId, clientNonce: input.nonce });
  const latePhases = [...f.phases]; const lateFailures = [...f.failures];
  generation = 3; nextPort.reply(lateFrame, canonical(input));
  assert.notEqual(await late.completion, "sent");
  assert.deepEqual(f.phases, latePhases); assert.deepEqual(f.failures, lateFailures);
  assert.equal(input.humanDispatch?.requestPosted, true);
  const recovery = f.queue.retryFailed(input.nonce, { isCurrent: () => generation === 3 }); assert.ok(recovery);
  const recoveredFrame = await nextPort.nextRequest();
  assert.equal(recoveredFrame.method, "retryHumanMessage");
  nextPort.reply(recoveredFrame, canonical(input));
  assert.equal(await recovery.completion, "sent");
  assert.equal(firstPort.requests.length, 1); assert.equal(nextPort.requests.length, 2);
  assert.equal(f.queue.snapshot().length, 0);
  f.queue.dispose(); ports.client.dispose();
});

test("Human retry callbacks carry their explicit attempt while original Agent queue callbacks and order remain unchanged", { timeout: 10_000 }, async () => {
  const ports = portFixture(); const port = ports.connect(); port.ready();
  const phases: (ComposerSubmission & { phase: ComposerSubmissionPhase; isRetry?: boolean; isCurrentAttempt?: () => boolean })[] = [];
  const failures: ComposerSubmissionRecord[] = [];
  const queue = createComposerSubmissionQueue({
    isTransportDown: () => false,
    send: async (input, admission) => {
      await sendHumanComposerSubmission(input, { client: ports.client, bridge: null,
        isCurrent: () => admission?.isCurrent?.() !== false });
    },
    onPhase: (input) => phases.push(input),
    onFailure: (input) => failures.push(input),
  });
  const input = originalSubmission();
  const initial = queue.submit(input); const initialFrame = await port.nextRequest();
  port.fail(initialFrame); assert.equal(await initial.completion, "failed");
  assert.equal(Object.hasOwn(failures[0], "isRetry"), false);
  assert.equal(Object.hasOwn(failures[0], "isCurrentAttempt"), false);
  assert.ok(phases.every((event) => !Object.hasOwn(event, "isCurrentAttempt")));
  let current = true;
  const isCurrent = () => current;
  const retry = queue.retryFailed(input.nonce, { isCurrent }); assert.ok(retry);
  const request = await port.nextRequest();
  assert.equal(request.method, "retryHumanMessage");
  port.fail(request); assert.equal(await retry.completion, "failed");
  assert.equal(failures.length, 2);
  assert.equal(failures[1].isRetry, true);
  assert.equal(phases.at(-1)?.isRetry, true);
  assert.equal(composerSubmissionBusyState(phases.at(-1)!), undefined, "retry terminal feedback does not own global busy");
  assert.equal(failures[1].isCurrentAttempt, isCurrent);
  assert.equal(phases.at(-1)?.isCurrentAttempt, isCurrent);
  assert.equal(failures[1].isCurrentAttempt?.(), true);
  current = false;
  assert.equal(failures[1].isCurrentAttempt?.(), false, "a deferred consumer receives the live attempt predicate");
  assert.equal(phases.at(-1)?.isCurrentAttempt?.(), false);
  queue.dispose(); ports.client.dispose();

  const first = deferred<void>(); const second = deferred<void>(); const secondEntered = deferred<void>();
  const agentA = { ...originalSubmission(), agentId: "agent-existing", nonce: "agent-a", prompt: "Agent first" };
  const agentB = { ...originalSubmission(), agentId: "agent-existing", nonce: "agent-b", prompt: "Agent second" };
  const sends: ComposerSubmission[] = []; const agentPhases: unknown[] = [];
  const agentFailures: { input: ComposerSubmissionRecord; error: unknown }[] = [];
  const agentQueue = createComposerSubmissionQueue({
    isTransportDown: () => false,
    send: (submission) => {
      sends.push(submission);
      if (submission === agentA) return first.promise;
      assert.equal(submission, agentB); secondEntered.resolve(); return second.promise;
    },
    onPhase: (event) => agentPhases.push(event),
    onFailure: (event, error) => agentFailures.push({ input: event, error }),
  });
  const a = agentQueue.submit(agentA); const b = agentQueue.submit(agentB);
  assert.deepEqual(sends, [agentA]);
  assert.deepEqual(agentPhases, [{ ...agentA, phase: "pending" }, { ...agentB, phase: "queued" }]);
  first.resolve(); assert.equal(await a.completion, "sent"); await secondEntered.promise;
  assert.deepEqual(sends, [agentA, agentB]);
  const failure = new Error("original Agent send failure"); second.reject(failure);
  assert.equal(await b.completion, "failed");
  assert.deepEqual(agentPhases, [
    { ...agentA, phase: "pending" }, { ...agentB, phase: "queued" },
    { ...agentA, phase: "sent" }, { ...agentB, phase: "pending" }, { ...agentB, phase: "failed" },
  ]);
  assert.deepEqual(agentFailures, [{ input: { ...agentB, phase: "failed" }, error: failure }]);
  assert.deepEqual(agentQueue.snapshot(), [{ ...agentB, phase: "failed" }]);
  assert.deepEqual(agentPhases.map((event) => composerSubmissionBusyState(event as { phase: ComposerSubmissionPhase })),
    [true, false, false, true, false], "original Agent pending/queued/terminal busy decisions remain unchanged");
  assert.equal(Object.hasOwn(agentA, "humanDispatch"), false);
  assert.equal(Object.hasOwn(agentB, "humanDispatch"), false);
  agentQueue.dispose();

  for (const outcome of ["late-success", "late-failure"] as const) {
    const abandoned = deferred<void>(); const current = deferred<void>();
    const oldInput = { ...originalSubmission(), agentId: "agent-reset", nonce: "same-reset-nonce", prompt: "Old account" };
    const nextInput = { ...oldInput, prompt: "Current account" };
    const resetSends: ComposerSubmission[] = []; const resetPhases: unknown[] = []; const resetFailures: unknown[] = [];
    const resetQueue = createComposerSubmissionQueue({
      isTransportDown: () => false,
      send: (input) => { resetSends.push(input); return input === oldInput ? abandoned.promise : current.promise; },
      onPhase: (input) => resetPhases.push(input),
      onFailure: (input) => resetFailures.push(input),
    });
    const oldFlight = resetQueue.submit(oldInput);
    const beforeReset = [...resetPhases]; resetQueue.reset();
    assert.equal(await oldFlight.completion, "cancelled");
    assert.deepEqual(resetPhases, beforeReset, "reset does not emit a stale account callback");
    assert.deepEqual(resetQueue.snapshot(), []);
    const nextFlight = resetQueue.submit(nextInput);
    const afterReplacement = [...resetPhases];
    assert.deepEqual(resetSends, [oldInput, nextInput], "reset releases the prior active lane for the current account");
    if (outcome === "late-success") abandoned.resolve(); else abandoned.reject(new Error("late abandoned send failure"));
    await abandoned.promise.catch(() => {});
    assert.deepEqual(resetPhases, afterReplacement, outcome);
    assert.deepEqual(resetFailures, [], outcome);
    assert.deepEqual(resetQueue.snapshot(), [{ ...nextInput, phase: "pending" }], "old completion cannot retire a reused nonce");
    current.resolve(); assert.equal(await nextFlight.completion, "sent");
    assert.deepEqual(resetQueue.snapshot(), []);
    assert.deepEqual(resetPhases.at(-1), { ...nextInput, phase: "sent" });
    resetQueue.dispose();
  }
});

test("explicit Human retry never captures or releases the current composer busy state across selection replacement", { timeout: 10_000 }, async () => {
  for (const startsCurrentOperation of [false, true]) {
    const ports = portFixture(); const port = ports.connect(); port.ready();
    let busy = false; let selection = "human-a"; let commits = 0;
    const phases: { nonce: string; phase: ComposerSubmissionPhase; isRetry?: boolean; busy: boolean | undefined }[] = [];
    const failures: string[] = [];
    const queue = createComposerSubmissionQueue({
      isTransportDown: () => false,
      send: async (input, admission) => {
        await sendHumanComposerSubmission(input, {
          client: ports.client,
          bridge: { async commitStagedAttachments() { commits += 1; return commits === 1 ? null : ["/committed/original.txt"]; } },
          isCurrent: () => admission?.isCurrent?.() !== false,
        });
      },
      onPhase: (input) => {
        const decision = composerSubmissionBusyState(input);
        phases.push({ nonce: input.nonce, phase: input.phase, ...(input.isRetry ? { isRetry: true } : {}), busy: decision });
        if (decision !== undefined) busy = decision;
      },
      onFailure: (input) => failures.push(input.nonce),
    });
    const original = { ...originalSubmission([{ path: "/staged/original", name: "original.txt" }]), agentId: "human-a" };
    const first = queue.submit(original);
    assert.equal(busy, true, "ordinary initial submission still occupies the composer");
    assert.equal(await first.completion, "failed");
    assert.equal(busy, false); assert.equal(port.requests.length, 0);
    const retry = queue.retryFailed(original.nonce, { isCurrent: () => selection === "human-a" }); assert.ok(retry);
    assert.equal(busy, false, "explicit retry pending belongs only to its transcript row");
    assert.deepEqual(phases.at(-1), { nonce: original.nonce, phase: "pending", isRetry: true, busy: undefined });
    const retryRequest = await port.nextRequest();
    assert.equal(retryRequest.method, "sendHumanMessage", "the original pre-Host failure resumes its first request");
    selection = "human-b";
    assert.equal(busy, false, "selecting B cannot inherit a stuck retry busy flag");

    const next = { ...originalSubmission(), agentId: "human-b", nonce: "current-b-operation" };
    const currentFlight = startsCurrentOperation ? queue.submit(next) : null;
    const currentRequest = startsCurrentOperation ? await port.nextRequest() : null;
    assert.equal(busy, startsCurrentOperation, "a new B operation owns its own busy state");
    const beforeLateReply = [...phases]; const failuresBeforeLateReply = [...failures];
    port.reply(retryRequest, canonical(original));
    assert.notEqual(await retry.completion, "sent");
    assert.equal(busy, startsCurrentOperation, "old terminal feedback cannot capture or clear B's busy state");
    assert.deepEqual(phases, beforeLateReply);
    assert.deepEqual(failures, failuresBeforeLateReply);
    if (currentFlight != null && currentRequest != null) {
      port.reply(currentRequest, { ...canonical(next), id: "human-server-message:10", remoteMessageId: "10" });
      assert.equal(await currentFlight.completion, "sent");
      assert.equal(busy, false, "only the current ordinary operation releases its busy state");
    }
    queue.dispose(); ports.client.dispose();
  }
});
