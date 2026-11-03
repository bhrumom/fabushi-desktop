import assert from "node:assert/strict";
import test from "node:test";

import type { CoordinatorPortBridge, CursorAuthStatus, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
import { createCoordinatorConnectionController } from "../recovered/features/root-resilience/connection-state";
import { createCoordinatorClient, COORDINATOR_PROTOCOL_VERSION, type ProductionCoordinatorClient } from "./coordinator-client";

function fixture() {
  let consumer: { onPort(port: TransferredCoordinatorPort): void } | null = null;
  const bridge: CoordinatorPortBridge = {
    claim(next) { consumer = next; return { request() {}, release() { consumer = null; } }; },
  };
  const client = createCoordinatorClient(bridge)!;
  assert.ok(client);
  const connect = () => {
    const posted: Record<string, unknown>[] = [];
    const listeners = new Map<string, (event: any) => void>();
    const port: TransferredCoordinatorPort = {
      postMessage(value) { posted.push(value as Record<string, unknown>); },
      close() {}, start() {},
      addEventListener(type: string, listener: (event: any) => void) { listeners.set(type, listener); },
    };
    consumer!.onPort(port);
    const emit = (value: unknown) => listeners.get("message")!({ data: value });
    return {
      posted, emit,
      requests: () => posted.filter((frame) => frame.kind === "request"),
      ready: () => emit({ kind: "lifecycle", phase: "ready", protocolVersion: COORDINATOR_PROTOCOL_VERSION }),
      disconnect: () => listeners.get("close")!({}),
      reply: (requestId: unknown, value: unknown) => emit({ kind: "reply", requestId, outcome: { status: "ok", value } }),
      fail: (requestId: unknown, code = "fixture/unavailable") => emit({ kind: "reply", requestId,
        outcome: { status: "failed", failure: { code, message: "Controlled fixture failure" } } }),
    };
  };
  return { client, connect };
}

const args = { conversationId: "human:1:2", clientNonce: "original-nonce" };
const settled = { id: "human-server-message:42", kind: "message", role: "user", authorKind: "human",
  authorId: "1", clientNonce: args.clientNonce, remoteMessageId: "42", delivery: "sent", content: "original" };

test("actual Coordinator retry frame carries only durable identities and validates canonical replies", async () => {
  const { client, connect } = fixture();
  const port = connect(); port.ready();
  await client.ready;
  for (const extra of ["text", "senderId", "attachments", "replyToId", "forwardContext"]) {
    await assert.rejects(client.call("retryHumanMessage", { ...args, [extra]: null }), /requires only/);
  }
  assert.equal(port.requests().length, 0);
  const sending = client.call("retryHumanMessage", args, { isCurrent: () => true });
  await Promise.resolve();
  const frame = port.requests()[0];
  assert.deepEqual(frame, { kind: "request", requestId: "r-1", method: "retryHumanMessage", args });
  port.reply(frame.requestId, settled);
  assert.deepEqual(await sending, settled);
  const malformed = client.call("retryHumanMessage", args);
  await Promise.resolve();
  port.reply(port.requests()[1].requestId, { ...settled, delivery: "pending" });
  await assert.rejects(malformed, /malformed canonical Human message/);
  const numeric = { ...settled, remoteMessageId: 42 };
  const repeated = client.call("retryHumanMessage", args);
  await Promise.resolve();
  port.reply(port.requests()[2].requestId, numeric);
  assert.deepEqual(await repeated, numeric);
  client.dispose();
});

test("queued initial readiness rechecks account and selection lifetime before posting a retry frame", async () => {
  for (const replacement of ["account", "selection-aba"]) {
    const { client, connect } = fixture();
    const port = connect();
    let generation = 7;
    const captured = generation;
    const sending = client.call("retryHumanMessage", args, { isCurrent: () => generation === captured });
    const rejected = assert.rejects(sending, /scope expired/);
    generation += replacement === "account" ? 1 : 2;
    port.ready();
    await rejected;
    assert.equal(port.requests().length, 0, replacement);
    client.dispose();
  }
});

test("resolved initial ready does not admit an expired retry queued on reconnect readiness", async () => {
  const { client, connect } = fixture();
  const original = connect(); original.ready(); await client.ready;
  original.disconnect();
  const replacement = connect();
  let current = true;
  const sending = client.call("retryHumanMessage", args, { isCurrent: () => current });
  const rejected = assert.rejects(sending, /scope expired/);
  current = false;
  replacement.ready();
  await rejected;
  assert.equal(original.requests().length, 0);
  assert.equal(replacement.requests().length, 0);
  client.dispose();
});

test("expired retry refuses immediately while an existing unguarded Agent call keeps its transport", async () => {
  const { client, connect } = fixture();
  const port = connect();
  await assert.rejects(client.call("retryHumanMessage", args, { isCurrent: () => false }), /scope expired/);
  assert.equal(port.requests().length, 0);
  port.ready(); await client.ready;
  const agentArgs = { id: "agent-one" };
  const listing = client.call("listAgents", agentArgs);
  await Promise.resolve();
  assert.deepEqual(port.requests()[0], { kind: "request", requestId: "r-1", method: "listAgents", args: agentArgs });
  port.reply("r-1", [{ id: "agent-one" }]);
  assert.deepEqual(await listing, [{ id: "agent-one" }]);
  client.dispose();
});

function runtimeFrame(type: string, lifecycle: string, generation: unknown, sequence: unknown) {
  return { kind: "event", family: "runtime", payload: {
    type, lifecycle, generation, sequence, timestamp: "2026-11-03T00:00:00.000Z",
    ...(type === "host.lifecycle" ? { state: lifecycle === "running" ? "running" : "stopped", recoverable: true } : {}),
  } };
}

async function flushMicrotasks() {
  for (let index = 0; index < 12; index += 1) await Promise.resolve();
}

function recoveryController(
  client: ProductionCoordinatorClient,
  retry: () => Promise<unknown>,
  initialAccountStatus?: Promise<CursorAuthStatus>,
) {
  let account: CursorAuthStatus = { kind: "logged-in", authId: "fixture-a", email: "a@example.test" };
  const accountListeners = new Set<(status: CursorAuthStatus) => void>();
  const controller = createCoordinatorConnectionController({
    ready: client.ready,
    getAccountStatus: () => initialAccountStatus ?? Promise.resolve(account),
    subscribeAccount(listener) {
      accountListeners.add(listener);
      return () => accountListeners.delete(listener);
    },
    subscribeTransport: (listener) => client.subscribeTransport(listener),
    retry,
  });
  return {
    controller,
    setAccount(status: CursorAuthStatus) {
      account = status;
      for (const listener of [...accountListeners]) listener(status);
    },
  };
}

test("runtime Host replacement waits for the current Gateway transport before notifying recovery", async () => {
  const { client, connect } = fixture();
  const port = connect();
  const states: string[] = [];
  const payloads: unknown[] = [];
  const stopTransport = client.subscribeTransport((state) => states.push(state));
  const stopRuntime = client.subscribe("runtime", (payload) => payloads.push(payload));
  assert.deepEqual(states, [], "unknown transport is not replayed as connected");
  port.ready();
  await client.ready;
  assert.deepEqual(states, ["connected"], "protocol ready retains the initial bootstrap");

  const frames = [
    runtimeFrame("host.lifecycle", "running", 1, 1),
    runtimeFrame("gateway.lifecycle", "discovered", 1, 2),
    runtimeFrame("gateway.lifecycle", "transport-connected", 1, 3),
    runtimeFrame("host.lifecycle", "stopped", 1, 4),
    runtimeFrame("host.lifecycle", "running", 2, 5),
    runtimeFrame("gateway.lifecycle", "transport-connected", 2, 6),
  ];
  port.emit(frames[0]);
  port.emit(frames[1]);
  assert.deepEqual(states, ["connected", "down"], "spawn and discovery do not admit a Gateway refresh");
  port.emit(frames[2]);
  port.emit(frames[3]);
  port.emit(frames[4]);
  assert.deepEqual(states, ["connected", "down", "connected", "down", "down"]);
  port.emit(frames[5]);
  assert.deepEqual(states, ["connected", "down", "connected", "down", "down", "connected"]);
  assert.deepEqual(payloads, frames.map((frame) => frame.payload), "runtime subscribers still receive the original payload");
  assert.equal(port.requests().length, 0, "lifecycle delivery does not replay a business command");
  stopTransport(); stopRuntime(); client.dispose();

  for (const lifecycle of ["discovery-timeout", "rediscovery-failed"]) {
    const next = fixture();
    const nextPort = next.connect();
    nextPort.ready(); await next.client.ready;
    const observed: string[] = [];
    next.client.subscribeTransport((state) => observed.push(state));
    nextPort.emit(runtimeFrame("gateway.lifecycle", lifecycle, 1, 1));
    nextPort.emit(runtimeFrame("gateway.lifecycle", lifecycle, 1, 2));
    assert.deepEqual(observed, ["connected", "down"], lifecycle);
    nextPort.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 3));
    assert.deepEqual(observed, ["connected", "down", "connected"], lifecycle);
    next.client.dispose();
  }
});

test("runtime same-generation reconnects once and rejects stale or terminal-generation transitions", async () => {
  const { client, connect } = fixture();
  const port = connect(); port.ready(); await client.ready;
  const states: string[] = [];
  client.subscribeTransport((state) => states.push(state));
  assert.deepEqual(states, ["connected"], "late subscribers synchronously receive current transport");
  states.length = 0;
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 7, 20));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 7, 20));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 7, 21));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 6, 1000));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 7, 19));
  assert.deepEqual(states, ["connected"], "neither duplicates nor older generations add recovery");
  port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 7, 22));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 7, 23));
  port.emit(runtimeFrame("host.lifecycle", "stopped", 7, 24));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 7, 10000));
  port.emit(runtimeFrame("host.lifecycle", "running", 7, 10001));
  assert.deepEqual(states, ["connected", "down", "connected", "down"], "terminal Host generations cannot revive");
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 8, 1));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 9, 1));
  assert.deepEqual(states, ["connected", "down", "connected", "down", "connected", "connected"],
    "new Host generations recover even when down was missed and their sequence is smaller");
  assert.equal(port.requests().length, 0);
  client.dispose();

  for (const lifecycle of ["stopped", "circuit-open", "spawn-failed"]) {
    const next = fixture();
    const nextPort = next.connect(); nextPort.ready(); await next.client.ready;
    nextPort.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 1));
    const observed: string[] = [];
    next.client.subscribeTransport((state) => observed.push(state));
    nextPort.emit(runtimeFrame("host.lifecycle", lifecycle, 1, 2));
    nextPort.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 3));
    nextPort.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 2, 1));
    assert.deepEqual(observed, ["connected", "down", "connected"], lifecycle);
    next.client.dispose();
  }
});

test("runtime informational and malformed frames cannot advance the generation or sequence fence", async () => {
  const { client, connect } = fixture();
  const port = connect(); port.ready(); await client.ready;
  const states: string[] = [];
  const payloads: unknown[] = [];
  client.subscribeTransport((state) => states.push(state));
  client.subscribe("runtime", (payload) => payloads.push(payload));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 5, 1));
  const beforeNoise = [...states];
  const noise: unknown[] = [
    runtimeFrame("host.lifecycle", "protocol-error", 900, 900),
    runtimeFrame("gateway.lifecycle", "discovered", 901, 901),
    runtimeFrame("host.lifecycle", "unknown", 902, 902),
    runtimeFrame("unknown.lifecycle", "transport-connected", 903, 903),
  ];
  for (const invalid of ["9", 0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1, NaN, null, undefined]) {
    noise.push(runtimeFrame("gateway.lifecycle", "transport-down", invalid, 999));
    noise.push(runtimeFrame("gateway.lifecycle", "transport-down", 999, invalid));
  }
  noise.push({ kind: "event", family: "runtime", payload: null });
  noise.push({ kind: "event", family: "runtime", payload: [] });
  for (const frame of noise) port.emit(frame);
  assert.deepEqual(states, beforeNoise, "only recognized transitions with positive safe integer counters affect transport");
  assert.equal(payloads.length, noise.length + 1, "ignored transport observations remain ordinary runtime events");
  port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 5, 2));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 5, 3));
  port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 6, 1));
  assert.deepEqual(states, ["connected", "connected", "down", "connected", "connected"],
    "a large informational or malformed counter cannot hide later meaningful current observations");
  assert.equal(port.requests().length, 0);
  client.dispose();
});

test("Coordinator port replacement resets runtime epochs while lifecycle events preserve pending RPC settlement", async () => {
  const { client, connect } = fixture();
  const original = connect(); original.ready(); await client.ready;
  original.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 8, 1));
  const sending = client.call("retryHumanMessage", args);
  let mutationSettled = false;
  void sending.then(() => { mutationSettled = true; }, () => { mutationSettled = true; });
  await flushMicrotasks();
  const mutationFrame = original.requests()[0];
  assert.ok(mutationFrame);
  assert.equal(mutationFrame.method, "retryHumanMessage");
  const nested: string[] = [];
  let nestedSubscribed = false;
  client.subscribeTransport((state) => {
    if (state === "down" && !nestedSubscribed) {
      nestedSubscribed = true;
      client.subscribeTransport((next) => nested.push(next));
    }
  });
  original.emit(runtimeFrame("host.lifecycle", "stopped", 8, 2));
  const late: string[] = [];
  client.subscribeTransport((state) => late.push(state));
  original.ready();
  await flushMicrotasks();
  assert.equal(mutationSettled, false, "runtime loss does not cancel an already posted mutation promise");
  assert.deepEqual(late, ["down"], "a repeated protocol ready cannot overwrite a newer runtime down");
  assert.deepEqual(nested, ["down"], "a subscriber added during fanout receives one snapshot replay");
  assert.equal(original.requests().length, 1, "recovery does not replay the held mutation");
  original.reply(mutationFrame.requestId, settled);
  assert.deepEqual(await sending, settled, "the existing request still receives its canonical settlement");

  const oldListing = client.call("listAgents");
  const oldRejected = assert.rejects(oldListing, /coordinator session replaced/);
  await flushMicrotasks();
  const oldFrame = original.requests()[1];
  assert.ok(oldFrame);
  const replacement = connect();
  await oldRejected;
  original.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 999, 999));
  original.reply(oldFrame.requestId, []);
  assert.deepEqual(late, ["down"], "frames from an old port cannot revive its runtime epoch");
  const listing = client.call("listAgents");
  await flushMicrotasks();
  assert.equal(replacement.requests().length, 0, "new requests wait for the replacement port handshake");
  replacement.ready();
  await flushMicrotasks();
  const newFrame = replacement.requests()[0];
  assert.ok(newFrame);
  assert.equal(newFrame.method, "listAgents");
  replacement.reply(newFrame.requestId, []);
  assert.deepEqual(await listing, []);
  replacement.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 1));
  replacement.emit(runtimeFrame("gateway.lifecycle", "transport-down", 1, 2));
  assert.deepEqual(late, ["down", "connected", "connected", "down"], "a new port starts a fresh runtime epoch");
  const afterReplacement: string[] = [];
  client.subscribeTransport((state) => afterReplacement.push(state));
  assert.deepEqual(afterReplacement, ["down"]);
  client.dispose();
  replacement.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 2, 1));
  original.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1000, 1000));
  client.subscribeTransport((state) => afterReplacement.push(state));
  assert.deepEqual(afterReplacement, ["down"], "disposed clients neither replay nor emit state");
});

test("an in-flight roster retry is replaced after same-generation transport recovery and ignores its late result", async () => {
  for (const recovery of ["same-generation", "new-generation"]) {
    for (const oldOutcome of ["ok", "failed"]) {
      const { client, connect } = fixture();
      const port = connect(); port.ready(); await client.ready;
      port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 4, 1));
      let rootGeneration = 0;
      const starts: number[] = [];
      const { controller } = recoveryController(client, () => {
        starts.push(rootGeneration);
        return client.call("listAgents");
      });
      // This is the real registration order: the controller listens before Root.
      controller.start();
      const rootNotices: Array<{ state: string; generation: number; retrying: boolean; attempts: number }> = [];
      const stopRoot = client.subscribeTransport((state) => {
        rootGeneration += 1;
        rootNotices.push({ state, generation: rootGeneration, retrying: controller.get().isRetrying, attempts: starts.length });
      });
      await flushMicrotasks();
      controller.setReadiness({ hasReachedBox: true, isPrivacyBlocked: false, failureCode: "roster-load-failed" });
      const obligation = controller.retry();
      assert.equal(controller.retry(), obligation, "concurrent Retry callers share one current obligation");
      let obligationSettled = false;
      void obligation.then(() => { obligationSettled = true; });
      assert.equal(controller.get().isRetrying, true);
      await flushMicrotasks();
      assert.deepEqual(starts, [rootGeneration]);
      const oldFrame = port.requests()[0];
      assert.ok(oldFrame);
      assert.equal(oldFrame.method, "listAgents");

      if (recovery === "same-generation") {
        port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 4, 2));
        port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 4, 3));
      } else {
        port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 5, 1));
      }
      const currentRootGeneration = rootGeneration;
      assert.equal(starts.length, 1, "the controller must not start a retry inside the earlier transport listener");
      assert.deepEqual(rootNotices[rootNotices.length - 1], {
        state: "connected", generation: currentRootGeneration, retrying: true, attempts: 1,
      }, "Root observes the retry obligation before it advances its refresh scope");
      await flushMicrotasks();
      assert.deepEqual(starts, [1, currentRootGeneration], "the replacement retry captures Root's completed fanout generation");
      assert.equal(port.requests().length, 2);
      const newFrame = port.requests()[1];
      assert.ok(newFrame);
      assert.notEqual(newFrame.requestId, oldFrame.requestId);
      assert.equal(newFrame.method, "listAgents");
      assert.equal(controller.get().isRetrying, true);
      assert.equal(controller.retry(), obligation, "transport replacement retains the original user waiter");

      if (oldOutcome === "ok") port.reply(oldFrame.requestId, []);
      else port.fail(oldFrame.requestId);
      await flushMicrotasks();
      assert.equal(obligationSettled, false, oldOutcome + ": a late old result must not settle the replacement waiter");
      assert.equal(controller.get().isRetrying, true, oldOutcome + ": old completion/finally must not release the new slot");
      assert.equal(controller.get().failureCode, "roster-load-failed", oldOutcome + ": old completion must not clear or replace failure state");
      assert.equal(port.requests().length, 2, oldOutcome + ": old finally must not start a third attempt");
      port.reply(newFrame.requestId, []);
      assert.equal(await obligation, true);
      await flushMicrotasks();
      assert.equal(controller.get().isRetrying, false);
      assert.equal(controller.get().failureCode, null);
      assert.equal(port.requests().length, 2);
      stopRoot(); controller.dispose(); client.dispose();
    }
  }
});

test("transport retry handoff is account-scoped, single-flight and inert after disposal", async () => {
  {
    const { client, connect } = fixture();
    const port = connect();
    let calls = 0;
    const { controller } = recoveryController(client, async () => { calls += 1; });
    controller.start();
    await flushMicrotasks();
    port.ready();
    port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 1, 1));
    await flushMicrotasks();
    assert.equal(controller.get().transport, "down", "a late initial ready continuation cannot overwrite observed runtime down");
    const late = recoveryController(client, async () => { calls += 1; });
    late.controller.start();
    await flushMicrotasks();
    assert.equal(late.controller.get().transport, "down", "a new subscriber receives current down even when initial ready already resolved");
    assert.equal(calls, 0);
    late.controller.dispose(); controller.dispose(); client.dispose();
  }


  for (const lateStatus of [
    { kind: "logged-out" } as const,
    { kind: "logged-in", authId: "fixture-a", email: "a@example.test" } as const,
  ]) {
    const { client, connect } = fixture();
    const port = connect(); port.ready(); await client.ready;
    let resolveInitial!: (status: CursorAuthStatus) => void;
    const initialAccountStatus = new Promise<CursorAuthStatus>((resolve) => { resolveInitial = resolve; });
    const { controller, setAccount } = recoveryController(client, () => client.call("listAgents"), initialAccountStatus);
    controller.start();
    setAccount({ kind: "logged-in", authId: "fixture-b", email: "b@example.test" });
    assert.equal(controller.get().accountIdentity, "logged-in:fixture-b");
    const current = controller.retry();
    await flushMicrotasks();
    assert.equal(port.requests().length, 1);
    resolveInitial(lateStatus);
    await flushMicrotasks();
    assert.equal(controller.get().accountIdentity, "logged-in:fixture-b", "the initial account read cannot overwrite a newer first status event");
    assert.equal(controller.get().isRetrying, true, "a late initial read cannot retire the current account's retry");
    setAccount({ kind: "logged-in", authId: "fixture-b", email: "updated@example.test" });
    assert.equal(controller.retry(), current, "a repeated observation of the same account does not cancel its active waiter");
    port.reply(port.requests()[0].requestId, []);
    assert.equal(await current, true);
    await flushMicrotasks();
    assert.equal(controller.get().accountIdentity, "logged-in:fixture-b");
    assert.equal(port.requests().length, 1);
    controller.dispose(); client.dispose();
  }

  for (const [retirement, afterAdmission] of [
    ["account", false], ["account", true], ["dispose", false], ["dispose", true],
  ] as const) {
    const { client, connect } = fixture();
    const port = connect(); port.ready(); await client.ready;
    port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 1));
    let activeAccount = "fixture-a";
    const starts: string[] = [];
    const { controller, setAccount } = recoveryController(client, () => {
      starts.push(activeAccount);
      return client.call("listAgents");
    });
    controller.start(); await flushMicrotasks();
    const obligation = controller.retry();
    await flushMicrotasks();
    const oldFrame = port.requests()[0];
    assert.ok(oldFrame);
    port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 1, 2));
    port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 3));
    assert.equal(starts.length, 1, "a handoff is scheduled but has not crossed the account/dispose boundary");
    if (afterAdmission) {
      await Promise.resolve();
      assert.equal(starts.length, 1, "the admitted retry has not invoked its source before the next microtask");
    }
    if (retirement === "account") {
      activeAccount = "fixture-b";
      setAccount({ kind: "logged-in", authId: activeAccount, email: "b@example.test" });
      assert.equal(await obligation, false, "the old account's waiter retires without replay");
      await flushMicrotasks();
      assert.deepEqual(starts, ["fixture-a"]);
      assert.equal(controller.get().isRetrying, false);
      const current = controller.retry();
      assert.equal(controller.retry(), current);
      await flushMicrotasks();
      assert.deepEqual(starts, ["fixture-a", "fixture-b"]);
      const newFrame = port.requests()[1];
      assert.ok(newFrame);
      port.fail(oldFrame.requestId, "fixture/old-account");
      await flushMicrotasks();
      assert.equal(controller.get().isRetrying, true);
      assert.equal(controller.get().failureCode, null, "old-account failure cannot poison the new account");
      port.reply(newFrame.requestId, []);
      assert.equal(await current, true);
      await flushMicrotasks();
      assert.equal(controller.get().accountIdentity, "logged-in:fixture-b");
      assert.equal(controller.get().isRetrying, false);
      assert.equal(port.requests().length, 2);
      controller.dispose();
    } else {
      controller.dispose();
      const disposedSnapshot = controller.get();
      assert.equal(await obligation, false);
      port.emit(runtimeFrame("gateway.lifecycle", "transport-down", 1, 4));
      port.emit(runtimeFrame("gateway.lifecycle", "transport-connected", 1, 5));
      port.fail(oldFrame.requestId, "fixture/disposed");
      await flushMicrotasks();
      assert.equal(controller.get(), disposedSnapshot, "late callbacks cannot mutate a disposed snapshot");
      assert.deepEqual(starts, ["fixture-a"]);
      assert.equal(port.requests().length, 1);
      assert.equal(await controller.retry(), false);
    }
    client.dispose();
  }

  {
    const { client, connect } = fixture();
    const port = connect(); port.ready(); await client.ready;
    const { controller } = recoveryController(client, () => client.call("listAgents"));
    controller.start(); await flushMicrotasks();
    const failed = controller.retry();
    assert.equal(controller.retry(), failed);
    await flushMicrotasks();
    assert.equal(port.requests().length, 1);
    port.fail(port.requests()[0].requestId, "fixture/denied");
    assert.equal(await failed, false);
    await flushMicrotasks();
    assert.equal(controller.get().failureCode, "fixture/denied");
    assert.equal(controller.get().isRetrying, false);
    assert.equal(port.requests().length, 1, "a current failure does not create an automatic retry loop");
    const retried = controller.retry();
    await flushMicrotasks();
    assert.equal(port.requests().length, 2);
    port.reply(port.requests()[1].requestId, []);
    assert.equal(await retried, true);
    await flushMicrotasks();
    assert.equal(controller.get().failureCode, null);
    controller.dispose(); client.dispose();
  }

  {
    const { client, connect } = fixture();
    const port = connect(); port.ready(); await client.ready;
    let attempts = 0;
    const { controller } = recoveryController(client, () => {
      attempts += 1;
      throw Object.assign(new Error("Controlled synchronous roster failure"), { code: "fixture/synchronous" });
    });
    controller.start(); await flushMicrotasks();
    assert.equal(await controller.retry(), false, "a synchronous source failure settles through the same retry owner");
    await flushMicrotasks();
    assert.equal(controller.get().failureCode, "fixture/synchronous");
    assert.equal(controller.get().isRetrying, false);
    assert.equal(attempts, 1);
    assert.equal(port.requests().length, 0);
    controller.dispose(); client.dispose();
  }
});
