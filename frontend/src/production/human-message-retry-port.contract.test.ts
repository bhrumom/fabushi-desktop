import assert from "node:assert/strict";
import test from "node:test";

import type { CoordinatorPortBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
import { createCoordinatorClient, COORDINATOR_PROTOCOL_VERSION } from "./coordinator-client";

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
