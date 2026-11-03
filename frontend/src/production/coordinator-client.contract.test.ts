import assert from "node:assert/strict";
import test, { type TestContext } from "node:test";
import { setImmediate as nextTurn } from "node:timers/promises";
import { createCoordinatorPortBroker } from "../../../source/electron-preload/coordinator-port-bridge.ts";
import type { TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge.ts";
import {
  COORDINATOR_PROTOCOL_VERSION,
  CoordinatorCallError,
  createCoordinatorClient,
} from "./coordinator-client.ts";

// NMR-50d: exercise the production client and broker. Only the transferred
// transport is controlled; no mock client may admit a request before ready.
// These cases distinguish local admission from already posted Host work,
// including malformed correlated reply recovery without retiring other calls.
interface RequestFrame {
  readonly kind: "request";
  readonly requestId: string;
  readonly method: string;
  readonly args: unknown;
}

type MessageListener = (event: { data: unknown }) => void;
type CloseListener = (event: Record<string, never>) => void;

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

class ControlledPort implements TransferredCoordinatorPort {
  readonly posted: unknown[] = [];
  readonly #messages = new Set<MessageListener>();
  readonly #closes = new Set<CloseListener>();
  starts = 0;
  closes = 0;
  failNextSendPost = false;

  postMessage(message: unknown): void {
    if (this.failNextSendPost && isRecord(message) && message.kind === "request" && message.method === "sendHumanMessage") {
      this.failNextSendPost = false;
      throw new Error("fixture request frame was not posted");
    }
    // A real transferred port cannot serialize the local admission callback.
    // Recording a clone also prevents later caller mutations hiding the frame.
    this.posted.push(structuredClone(message));
  }

  start(): void { this.starts += 1; }

  close(): void {
    this.closes += 1;
    this.disconnect();
  }

  addEventListener(type: "message", listener: MessageListener): void;
  addEventListener(type: "close", listener: CloseListener): void;
  addEventListener(type: "message" | "close", listener: MessageListener | CloseListener): void {
    if (type === "message") this.#messages.add(listener as MessageListener);
    else this.#closes.add(listener as CloseListener);
  }

  receive(frame: unknown): void {
    // Deliver even after close: the actual client must fence a retired port.
    for (const listener of this.#messages) listener({ data: structuredClone(frame) });
  }

  disconnect(): void {
    for (const listener of this.#closes) listener({});
  }

  ready(): void {
    this.receive({ kind: "lifecycle", phase: "ready", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  }

  succeed(request: RequestFrame, value: unknown): void {
    this.receive({ kind: "reply", requestId: request.requestId, outcome: { status: "ok", value } });
  }

  get requests(): RequestFrame[] {
    return this.posted
      .filter((frame): frame is Record<string, unknown> => isRecord(frame) && frame.kind === "request")
      .map((frame) => {
        assert.equal(typeof frame.requestId, "string", "request needs a correlation id");
        assert.equal(typeof frame.method, "string", "request needs a method");
        assert.ok(Object.hasOwn(frame, "args"), "request needs serializable arguments");
        return frame as unknown as RequestFrame;
      });
  }
}

function harness(t: TestContext) {
  let portRequests = 0;
  const broker = createCoordinatorPortBroker<TransferredCoordinatorPort>({
    invokeRequest() { portRequests += 1; },
  });
  const client = createCoordinatorClient(broker.bridge);
  assert.ok(client, "the actual broker must grant its first client claim");
  t.after(() => client.dispose());
  assert.equal(portRequests, 1, "claiming the client requests a transferred port");
  return {
    client,
    get portRequests() { return portRequests; },
    providePort() {
      const port = new ControlledPort();
      broker.deliver(port);
      assert.equal(port.starts, 1);
      assert.deepEqual(port.posted, [{
        kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION,
      }]);
      return port;
    },
  };
}

function forwardArgs(clientNonce = "forward-admission-1") {
  return {
    sourceConversationId: "human:source",
    sourceEntryId: "message:source",
    destinationConversationIds: ["human:destination"],
    clientNonce,
    dropSenderNames: true,
    dropCaptions: true,
  };
}

function forwardResult() {
  return { destinations: [{ conversationId: "human:destination", status: "sent" }] };
}

function observed<T>(promise: Promise<T>): Promise<T> {
  // A failed frame assertion still disposes the client in t.after. Observe that
  // cleanup rejection without replacing the original result asserted below.
  void promise.catch(() => {});
  return promise;
}

function onlyRequest(port: ControlledPort): RequestFrame {
  assert.equal(port.requests.length, 1, "exactly one request must be posted");
  return port.requests[0]!;
}

async function admitFreshForward(client: NonNullable<ReturnType<typeof createCoordinatorClient>>, port: ControlledPort) {
  const args = forwardArgs("fresh-live-scope");
  const result = observed(client.call("forwardHumanMessage", args, () => true));
  await nextTurn();
  const request = onlyRequest(port);
  assert.equal(request.method, "forwardHumanMessage");
  assert.deepEqual(request.args, args);
  const settlement = forwardResult();
  port.succeed(request, settlement);
  assert.deepEqual(await result, settlement);
}

test("Coordinator admission rejects an account retired while initial readiness is pending", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  let accountGeneration = 1;
  const originalGeneration = accountGeneration;
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    guardCalls += 1;
    return accountGeneration === originalGeneration;
  }));
  const rejected = assert.rejects(result, /scope expired before admission/);
  await nextTurn();
  assert.equal(guardCalls, 0);
  assert.equal(port.requests.length, 0);
  accountGeneration += 1;
  port.ready();
  await rejected;
  assert.equal(guardCalls, 1);
  assert.equal(port.requests.length, 0);
});

test("Coordinator admission rejects a closed dialog after the same source is reopened before ready", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  let dialog: { sourceEntryId: string } | null = { sourceEntryId: "message:source" };
  const originalDialog = dialog;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => dialog === originalDialog));
  const rejected = assert.rejects(result, /scope expired before admission/);
  await nextTurn();
  assert.equal(port.requests.length, 0);
  dialog = null;
  dialog = { sourceEntryId: "message:source" };
  port.ready();
  await rejected;
  assert.equal(port.requests.length, 0, "equal source ids cannot revive the old dialog lifetime");
});

test("Coordinator admission rejects an account A-B-A return with the original generation", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  let account = { id: "account-a", generation: 1 };
  const original = account;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () =>
    account.id === original.id && account.generation === original.generation));
  const rejected = assert.rejects(result, /scope expired before admission/);
  account = { id: "account-b", generation: 2 };
  account = { id: "account-a", generation: 3 };
  port.ready();
  await rejected;
  assert.equal(port.requests.length, 0);
});

test("Coordinator admission checks a live predicate only after ready and posts no callback", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  const args = forwardArgs();
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", args, () => {
    guardCalls += 1;
    return true;
  }));
  await nextTurn();
  assert.equal(guardCalls, 0);
  assert.equal(port.requests.length, 0);
  port.ready();
  await nextTurn();
  const request = onlyRequest(port);
  assert.equal(guardCalls, 1);
  assert.deepEqual(request, {
    kind: "request", requestId: request.requestId, method: "forwardHumanMessage", args,
  }, "only the ordinary gateway request is serialized");
  const settlement = forwardResult();
  port.succeed(request, settlement);
  assert.deepEqual(await result, settlement);
  assert.equal(guardCalls, 1, "reply handling does not reevaluate the local predicate");
});

test("Coordinator admission waits for replacement readiness after disconnect and rejects a retired scope", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const oldPort = h.providePort();
  oldPort.ready();
  await h.client.ready;
  oldPort.disconnect();
  assert.equal(h.portRequests, 2);
  await h.client.ready; // The public initial promise is already resolved.
  let current = true;
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    guardCalls += 1;
    return current;
  }));
  const rejected = assert.rejects(result, /scope expired before admission/);
  const replacement = h.providePort();
  oldPort.ready();
  await nextTurn();
  assert.equal(guardCalls, 0, "call uses currentReady, not the resolved initial promise");
  assert.equal(oldPort.requests.length, 0);
  assert.equal(replacement.requests.length, 0);
  current = false;
  replacement.ready();
  await rejected;
  assert.equal(guardCalls, 1);
  assert.equal(replacement.requests.length, 0);
  await admitFreshForward(h.client, replacement);
});

test("Coordinator port replacement rejects the old wait and requires the new port to become ready", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const oldPort = h.providePort();
  let oldGuardCalls = 0;
  const oldResult = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    oldGuardCalls += 1;
    return true;
  }));
  const rejected = assert.rejects(oldResult, /coordinator session replaced/);
  const replacement = h.providePort();
  await rejected;
  assert.equal(oldPort.closes, 1);
  assert.equal(oldGuardCalls, 0);
  assert.equal(oldPort.requests.length, 0);
  let freshGuardCalls = 0;
  const freshResult = observed(h.client.call("forwardHumanMessage", forwardArgs("replacement"), () => {
    freshGuardCalls += 1;
    return true;
  }));
  oldPort.ready();
  await nextTurn();
  assert.equal(freshGuardCalls, 0);
  assert.equal(replacement.requests.length, 0);
  replacement.ready();
  await nextTurn();
  const request = onlyRequest(replacement);
  const settlement = forwardResult();
  replacement.succeed(request, settlement);
  assert.deepEqual(await freshResult, settlement);
  assert.equal(freshGuardCalls, 1);
});

test("Coordinator admission requires strict true and a rejected predicate does not poison a fresh request", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  // Runtime misuse must fail closed even though the TypeScript API is boolean.
  for (const value of [false, undefined, null, 0, 1, "true", {}]) {
    await assert.rejects(
      observed(h.client.call("forwardHumanMessage", forwardArgs(), () => value as boolean)),
      /scope expired before admission/,
    );
    assert.equal(port.requests.length, 0);
  }
  await admitFreshForward(h.client, port);
});

test("Coordinator admission rejects a throwing predicate without posting and admits the next request", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  await assert.rejects(
    observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
      throw new Error("local scope inspection failed");
    })),
    /local scope inspection failed/,
  );
  assert.equal(port.requests.length, 0);
  await admitFreshForward(h.client, port);
});

test("Coordinator retirement after posting preserves the Host success without a cancellation frame", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  let current = true;
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    guardCalls += 1;
    return current;
  }));
  await nextTurn();
  const request = onlyRequest(port);
  const posted = structuredClone(port.posted);
  current = false;
  await nextTurn();
  assert.deepEqual(port.posted, posted, "retiring presentation does not send cancellation or another mutation");
  const settlement = forwardResult();
  port.succeed(request, settlement);
  assert.deepEqual(await result, settlement);
  assert.equal(guardCalls, 1);
  assert.deepEqual(port.posted, posted);
});

test("Coordinator retirement after posting preserves the Host failure code and transport kind", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  let current = true;
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    guardCalls += 1;
    return current;
  }));
  const rejected = assert.rejects(result, (error: unknown) => {
    assert.ok(error instanceof CoordinatorCallError);
    assert.equal(error.code, "forward.retryable");
    assert.equal(error.transportKind, "http");
    assert.equal(error.message, "forward.retryable: delivery confirmation unavailable");
    return true;
  });
  await nextTurn();
  const request = onlyRequest(port);
  current = false;
  port.receive({
    kind: "reply", requestId: request.requestId,
    outcome: {
      status: "failed",
      failure: { code: "forward.retryable", message: "delivery confirmation unavailable", transportKind: "http" },
    },
  });
  await rejected;
  assert.equal(guardCalls, 1);
  assert.equal(port.requests.length, 1);
});

test("Coordinator calls without a predicate retain readiness, default arguments and reply validation", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  const result = observed(h.client.call("listAgents"));
  await nextTurn();
  assert.equal(port.requests.length, 0);
  port.ready();
  await nextTurn();
  const request = onlyRequest(port);
  assert.deepEqual(request, {
    kind: "request", requestId: request.requestId, method: "listAgents", args: {},
  });
  const agents = [{ agentId: "agent-1" }];
  port.succeed(request, agents);
  assert.deepEqual(await result, agents);
  const malformed = observed(h.client.call("listAgents"));
  const rejected = assert.rejects(malformed, /listAgents returned a malformed array reply/);
  await nextTurn();
  assert.equal(port.requests.length, 2);
  port.succeed(port.requests[1]!, { agents: [] });
  await rejected;
});

test("Coordinator disposal rejects a readiness wait without evaluating its predicate or posting work", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  let guardCalls = 0;
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => {
    guardCalls += 1;
    return true;
  }));
  const rejected = assert.rejects(result, /coordinator source disposed/);
  h.client.dispose();
  await rejected;
  port.ready();
  await nextTurn();
  assert.equal(guardCalls, 0);
  assert.equal(port.requests.length, 0);
  assert.equal(port.closes, 1);
  assert.deepEqual(port.posted.at(-1), {
    kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null,
  });
  h.client.dispose();
  assert.equal(port.closes, 1, "disposal remains idempotent");
});

test("Coordinator malformed-frame recovery fences late old-port frames and admits a fresh request", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const oldPort = h.providePort();
  oldPort.ready();
  await h.client.ready;
  const original = observed(h.client.call("forwardHumanMessage", forwardArgs(), () => true));
  const rejected = assert.rejects(original, /coordinator posted a malformed frame/);
  await nextTurn();
  onlyRequest(oldPort);
  oldPort.receive(null);
  await rejected;
  assert.equal(h.portRequests, 2);
  assert.deepEqual(oldPort.posted.at(-1), {
    kind: "lifecycle", phase: "shutdown", reason: "protocol-error", detail: "coordinator posted a malformed frame",
  });
  const replacement = h.providePort();
  const result = observed(h.client.call("forwardHumanMessage", forwardArgs("after-malformed"), () => true));
  let settled = false;
  void result.then(() => { settled = true; }, () => { settled = true; });
  oldPort.ready();
  await nextTurn();
  assert.equal(replacement.requests.length, 0);
  replacement.ready();
  await nextTurn();
  const request = onlyRequest(replacement);
  oldPort.succeed(request, { destinations: [], stale: true });
  await nextTurn();
  assert.equal(settled, false, "even a matching reply id from a retired port cannot settle current work");
  const settlement = forwardResult();
  replacement.succeed(request, settlement);
  assert.deepEqual(await result, settlement);
});

function settledHumanRetry() {
  return {
    kind: "message", id: "human-server-message:42", role: "user",
    authorKind: "human", authorId: "local", clientNonce: "original",
    content: "Original text", delivery: "sent", remoteMessageId: "42",
    timestampMs: 101, replyToId: "human-server-message:7",
    attachments: [{ path: "/saved/original.pdf", name: "original.pdf", resourceId: "resource-1" }],
  };
}

test("Human retry waits for ready, fences account ABA and submits only durable identities", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  let generation = 1;
  const args = { conversationId: "human:local:peer", clientNonce: "original" };
  const stale = observed(h.client.call("retryHumanMessage", args, () => generation === 1));
  const refused = assert.rejects(stale, /scope expired before admission/);
  await nextTurn();
  assert.equal(port.requests.length, 0);
  generation = 2;
  generation = 3; // Same account identity after logout/relogin, new lifetime.
  port.ready();
  await refused;
  assert.equal(port.requests.length, 0);
  const fresh = observed(h.client.call("retryHumanMessage", args, () => generation === 3));
  await nextTurn();
  const request = onlyRequest(port);
  assert.equal(request.method, "retryHumanMessage");
  assert.deepEqual(request.args, args, "no replacement message, attachments or new nonce");
  const result = settledHumanRetry();
  port.succeed(request, result);
  assert.deepEqual(await fresh, result);
  assert.equal(port.requests.length, 1);
});

test("Human retry rejects payload changes before posting and malformed Host settlement after posting", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  const args = { conversationId: "human:local:peer", clientNonce: "original" };
  for (const candidate of [
    { ...args, text: "replacement" }, { ...args, attachments: [] },
    { ...args, replyToId: null }, { ...args, clientNonce: " original" },
    { ...args, clientNonce: "界".repeat(67) },
  ]) {
    await assert.rejects(observed(h.client.call("retryHumanMessage", candidate)),
      /original Human conversation and nonce identities/);
    assert.equal(port.requests.length, 0);
  }
  const invalid = observed(h.client.call("retryHumanMessage", args));
  const failed = assert.rejects(invalid, /malformed Human message settlement/);
  await nextTurn();
  const request = onlyRequest(port);
  port.succeed(request, { ...settledHumanRetry(), remoteMessageId: "43" });
  await failed;
  assert.equal(port.requests.length, 1);
  const again = observed(h.client.call("retryHumanMessage", args));
  await nextTurn();
  port.succeed(port.requests[1]!, settledHumanRetry());
  assert.deepEqual(await again, settledHumanRetry());
});

test("Human retry rejects malformed correlated envelopes once and preserves concurrent calls and same-intent recovery", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  const args = { conversationId: "human:local:peer", clientNonce: "original" };
  const malformed: ReadonlyArray<readonly [string, Record<string, unknown>]> = [
    ["missing outcome", {}],
    ["null outcome", { outcome: null }],
    ["array outcome", { outcome: [] }],
    ["primitive outcome", { outcome: "failed" }],
    ["missing status", { outcome: {} }],
    ["unknown status", { outcome: { status: "intermediate" } }],
    ["non-string status", { outcome: { status: null } }],
    ["missing failure", { outcome: { status: "failed" } }],
    ["null failure", { outcome: { status: "failed", failure: null } }],
    ["array failure", { outcome: { status: "failed", failure: [] } }],
    ["primitive failure", { outcome: { status: "failed", failure: "unavailable" } }],
  ];
  for (const [label, fields] of malformed) {
    const offset = port.requests.length;
    const original = observed(h.client.call("retryHumanMessage", args));
    let originalRejections = 0;
    void original.then(() => {}, () => { originalRejections += 1; });
    const rejected = assert.rejects(original, (error: unknown) => {
      assert.ok(error instanceof CoordinatorCallError, label);
      assert.equal(error.code, "protocol-error", label);
      assert.equal(error.transportKind, undefined, label);
      assert.match(error.message, /retryHumanMessage returned a malformed Coordinator reply outcome/, label);
      return true;
    });
    const unrelated = observed(h.client.call("listAgents"));
    let unrelatedSettled = false;
    void unrelated.then(() => { unrelatedSettled = true; }, () => { unrelatedSettled = true; });
    await nextTurn();
    assert.equal(port.requests.length, offset + 2, label);
    const originalRequest = port.requests[offset]!;
    const unrelatedRequest = port.requests[offset + 1]!;
    assert.equal(originalRequest.method, "retryHumanMessage", label);
    assert.equal(unrelatedRequest.method, "listAgents", label);
    port.receive({ kind: "reply", requestId: originalRequest.requestId, ...fields });
    await rejected;
    assert.equal(originalRejections, 1, label);
    assert.equal(unrelatedSettled, false, label);

    const retry = observed(h.client.call("retryHumanMessage", args));
    let retrySettled = false;
    void retry.then(() => { retrySettled = true; }, () => { retrySettled = true; });
    await nextTurn();
    assert.equal(port.requests.length, offset + 3, label);
    const retryRequest = port.requests[offset + 2]!;
    assert.deepEqual(retryRequest.args, args, "retry preserves the original durable identities");
    assert.notEqual(retryRequest.requestId, originalRequest.requestId, label);
    port.receive({ kind: "reply", requestId: originalRequest.requestId, ...fields });
    port.succeed(originalRequest, settledHumanRetry());
    await nextTurn();
    assert.equal(originalRejections, 1, label);
    assert.equal(unrelatedSettled, false, "duplicate old reply cannot consume another call");
    assert.equal(retrySettled, false, "late old success cannot settle the new request");
    const agents = [{ agentId: "unrelated-agent" }];
    port.succeed(unrelatedRequest, agents);
    assert.deepEqual(await unrelated, agents, label);
    assert.equal(retrySettled, false, label);
    port.succeed(retryRequest, settledHumanRetry());
    assert.deepEqual(await retry, settledHumanRetry(), label);
    assert.equal(h.portRequests, 1, "a correlated failure does not retire unrelated port work");
  }
});

test("Coordinator correlated failures preserve valid fields and existing fallback messages", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  const failures = [
    { failure: {}, code: "failed", message: "failed: Coordinator request failed", transportKind: undefined },
    {
      failure: { code: "forward.retryable", message: "confirmation unavailable", transportKind: "http" },
      code: "forward.retryable", message: "forward.retryable: confirmation unavailable", transportKind: "http",
    },
    {
      failure: { code: 7, message: null, transportKind: [] },
      code: "failed", message: "failed: Coordinator request failed", transportKind: undefined,
    },
  ];
  for (const expected of failures) {
    const result = observed(h.client.call("retryHumanMessage", {
      conversationId: "human:local:peer", clientNonce: "original",
    }));
    const rejected = assert.rejects(result, (error: unknown) => {
      assert.ok(error instanceof CoordinatorCallError);
      assert.equal(error.code, expected.code);
      assert.equal(error.message, expected.message);
      assert.equal(error.transportKind, expected.transportKind);
      return true;
    });
    await nextTurn();
    const request = port.requests.at(-1)!;
    port.receive({
      kind: "reply", requestId: request.requestId,
      outcome: { status: "failed", failure: expected.failure },
    });
    await rejected;
    assert.equal(h.portRequests, 1);
  }
});

test("Malformed replies with unknown identities or from a retired port cannot settle current Human retry work", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const retired = h.providePort();
  retired.ready();
  await h.client.ready;
  const args = { conversationId: "human:local:peer", clientNonce: "original" };
  const original = observed(h.client.call("retryHumanMessage", args));
  const replaced = assert.rejects(original, /coordinator session replaced/);
  await nextTurn();
  onlyRequest(retired);
  const current = h.providePort();
  await replaced;
  const retry = observed(h.client.call("retryHumanMessage", args));
  let retrySettled = false;
  void retry.then(() => { retrySettled = true; }, () => { retrySettled = true; });
  current.ready();
  await nextTurn();
  const request = onlyRequest(current);
  retired.receive({ kind: "reply", requestId: request.requestId, outcome: { status: "failed" } });
  retired.succeed(request, settledHumanRetry());
  current.receive({ kind: "reply", requestId: "not-a-pending-request", outcome: null });
  current.receive({ kind: "reply", outcome: { status: "failed" } });
  current.receive({ kind: "reply", requestId: null, outcome: [] });
  await nextTurn();
  assert.equal(retrySettled, false, "only this request on its current port may settle it");
  assert.equal(current.requests.length, 1);
  current.succeed(request, settledHumanRetry());
  assert.deepEqual(await retry, settledHumanRetry());
});

test("Human send admission records only a posted frame; a refused port preserves the original nonce for retry", { timeout: 5_000 }, async (t) => {
  const h = harness(t);
  const port = h.providePort();
  port.ready();
  await h.client.ready;
  const original = { conversationId: "human:local:peer", text: "same original", clientNonce: "post-failed-nonce", attachments: [] };
  let admissionChecks = 0;
  let posted = 0;
  port.failNextSendPost = true;
  const first = observed(h.client.call("sendHumanMessage", original, () => {
    admissionChecks += 1;
    return true;
  }, () => { posted += 1; }));
  await assert.rejects(first, /fixture request frame was not posted/);
  assert.equal(admissionChecks, 1, "readiness may check scope but does not imply Host admission");
  assert.equal(posted, 0, "failed frame serialization must not mark a Host-owned send");
  assert.equal(port.requests.length, 0, "a failed request is never transmitted");

  const retry = observed(h.client.call("sendHumanMessage", original, () => true, () => { posted += 1; }));
  await nextTurn();
  const request = onlyRequest(port);
  assert.deepEqual(request.args, original, "retry retains the original nonce and immutable intent");
  assert.equal(posted, 1, "only the successful frame records Host admission");
  assert.equal(request.requestId, "r-2", "failed frame correlation is not recycled");
  port.receive({ kind: "reply", requestId: "r-1", outcome: { status: "ok", value: { id: "forged-old" } } });
  const settled = { id: "human-server-message:201", clientNonce: original.clientNonce, delivery: "sent" };
  port.succeed(request, settled);
  assert.deepEqual(await retry, settled, "a late unknown reply cannot replace the surviving intent");
});
