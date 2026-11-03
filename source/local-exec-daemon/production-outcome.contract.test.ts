import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setImmediate as nextTurn } from "node:timers/promises";
import test from "node:test";

import type { JsonValue } from "@bufbuild/protobuf";
import { SandLocalExecProvider, type LocalExecExecutor, type LocalExecResponseFrame } from "../host/local-exec/local-exec-provider.js";
import { createContext } from "../packages/context/core.js";
import { ControlledExecDisposedError, SimpleControlledExecManager } from "../packages/agent-exec/controlled.js";
import { ExecClientControlMessage, ExecServerMessage } from "../packages/proto/generated/agent/v1/exec_pb.js";
import { ExecServerAbort, ExecServerControlMessage } from "../packages/proto/generated/agent/v1/agent_service_pb.js";
import { LOCAL_EXEC_PRE_ENTRY_REFUSED } from "../shared/local-exec-gateway.js";
import { createDefaultProductionLocalExecExecutor } from "./production-executor.js";

async function collect<T>(values: AsyncIterable<T>): Promise<T[]> {
  const result: T[] = [];
  for await (const value of values) result.push(value);
  return result;
}

function throwValue(message: JsonValue): { error: string; errorCode?: string } {
  const decoded = ExecClientControlMessage.fromJson(message);
  assert.equal(decoded.message.case, "throw");
  return decoded.message.value as { error: string; errorCode?: string };
}

async function providerFixture(blocked?: string) {
  const root = await mkdtemp(join(tmpdir(), "lxo-provider-"));
  const production = createDefaultProductionLocalExecExecutor({ root });
  let entered = 0;
  const executor: LocalExecExecutor = {
    ...production,
    execute: async function* (message, signal) {
      entered += 1;
      yield* production.execute(message, signal);
    },
  };
  const frames: LocalExecResponseFrame[] = [];
  const provider = new SandLocalExecProvider({
    root,
    executor,
    resolveConnection: async () => ({ baseUrl: "http://127.0.0.1:1", token: "fixture-only" }),
    isLocalUseBlocked: () => blocked,
    fetch: async (_input, init) => {
      const batch = JSON.parse(String(init?.body)) as { frames: LocalExecResponseFrame[] };
      frames.push(...batch.frames);
      return new Response(null, { status: 204 });
    },
  });
  async function waitFor(count: number): Promise<void> {
    const until = performance.now() + 2_000;
    while (frames.length < count && performance.now() < until) await nextTurn();
    assert.equal(frames.length, count, "all actual provider frames arrived once");
  }
  return {
    provider, frames, entered: () => entered, waitFor,
    close: async () => { provider.close(); await rm(root, { recursive: true, force: true }); },
  };
}

test("lxo production codec retains optional preentry code without inventing it", { timeout: 5_000 }, async () => {
  const root = await mkdtemp(join(tmpdir(), "lxo-codec-"));
  try {
    const executor = createDefaultProductionLocalExecExecutor({ root });
    assert.equal(throwValue(executor.throwControl("denied", LOCAL_EXEC_PRE_ENTRY_REFUSED)).errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);
    const opaque = throwValue(executor.throwControl("opaque"));
    assert.equal(opaque.error, "opaque");
    assert.equal(opaque.errorCode, undefined);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("lxo provider permission refusal is tagged before real executor entry", { timeout: 5_000 }, async () => {
  const fixture = await providerFixture("permission denied");
  try {
    await fixture.provider.handleRequest({
      kind: "exec", requestId: "permission-request",
      serverMessage: { id: 3, shellStreamArgs: { command: "must-not-run", workingDirectory: "/tmp", toolCallId: "permission-call" } },
    });
    await fixture.waitFor(1);
    assert.equal(fixture.entered(), 0);
    const frame = fixture.frames[0];
    assert.equal(frame.kind, "control");
    assert.equal("requestId" in frame && frame.requestId, "permission-request");
    assert.ok("message" in frame);
    const thrown = throwValue(frame.message);
    assert.equal(thrown.error, "permission denied");
    assert.equal(thrown.errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);
  } finally { await fixture.close(); }
});

test("lxo provider generated decode refusal is tagged before executor entry", { timeout: 5_000 }, async () => {
  const fixture = await providerFixture();
  try {
    await fixture.provider.handleRequest({
      kind: "exec", requestId: "decode-request", serverMessage: ["not", "a", "protobuf-message"],
    });
    await fixture.waitFor(1);
    assert.equal(fixture.entered(), 0);
    const frame = fixture.frames[0];
    assert.equal(frame.kind, "control");
    assert.equal("requestId" in frame && frame.requestId, "decode-request");
    assert.ok("message" in frame);
    assert.equal(throwValue(frame.message).errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);
  } finally { await fixture.close(); }
});

test("lxo real default provider preserves no-handler preentry witness", { timeout: 5_000 }, async () => {
  const fixture = await providerFixture();
  try {
    await fixture.provider.handleRequest({
      kind: "exec", requestId: "no-handler-request", serverMessage: { id: 4, execId: "no-handler" },
    });
    await fixture.waitFor(2);
    assert.equal(fixture.entered(), 1, "actual production adapter was called once");
    assert.ok(fixture.frames.every(frame => frame.kind === "control" && frame.requestId === "no-handler-request"));
    const first = fixture.frames[0];
    assert.ok("message" in first);
    assert.equal(throwValue(first.message).errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);
    const last = fixture.frames[1];
    assert.ok("message" in last);
    assert.equal(ExecClientControlMessage.fromJson(last.message).message.case, "streamClose");
  } finally { await fixture.close(); }
});

test("lxo controlled no-handler witness is distinct from entered exceptions", { timeout: 5_000 }, async () => {
  const empty = new SimpleControlledExecManager();
  const refused = await collect(empty.handle(createContext(), new ExecServerMessage({ id: 20 })));
  assert.equal(refused.length, 2);
  assert.ok(refused[0] instanceof ExecClientControlMessage);
  assert.equal(throwValue(refused[0].toJson()).errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);

  for (const [error, expectedCode] of [
    [new Error("opaque entered failure"), undefined],
    [new ControlledExecDisposedError("entered backend disposed"), "EXEC_BACKEND_UNAVAILABLE"],
    [Object.assign(new Error("entered stream timeout"), { name: "AgentExecStreamStartTimeoutError" }), "AGENT_EXEC_STREAM_START_TIMEOUT"],
  ] as const) {
    const manager = new SimpleControlledExecManager();
    let entered = 0;
    manager.register({ handle: () => (async function* () {
      entered += 1;
      throw error;
    })() });
    const messages = await collect(manager.handle(createContext(), new ExecServerMessage({ id: 21 })));
    assert.equal(entered, 1);
    assert.equal(messages.length, 1);
    assert.ok(messages[0] instanceof ExecClientControlMessage);
    const thrown = throwValue(messages[0].toJson());
    assert.equal(thrown.errorCode, expectedCode);
    assert.notEqual(thrown.errorCode, LOCAL_EXEC_PRE_ENTRY_REFUSED);
    assert.match(thrown.error, /entered/);
  }
});

test("lxo controlled cancellation can close stream without a shell Exit", { timeout: 5_000 }, async () => {
  const manager = new SimpleControlledExecManager();
  let entered = 0;
  let cancellationObserved = false;
  manager.register({ handle: (ctx) => (async function* () {
    entered += 1;
    if (!ctx.signal.aborted) await new Promise<void>(resolve => {
      ctx.signal.addEventListener("abort", () => resolve(), { once: true });
    });
    cancellationObserved = ctx.signal.aborted;
  })() });
  const values = manager.handle(createContext(), new ExecServerMessage({ id: 22 }));
  assert.equal(entered, 1);
  manager.handleControlMessage(new ExecServerControlMessage({
    message: { case: "abort", value: new ExecServerAbort({ id: 22 }) },
  }));
  const messages = await collect(values);
  assert.equal(cancellationObserved, true);
  assert.equal(messages.length, 1);
  assert.ok(messages[0] instanceof ExecClientControlMessage);
  assert.equal(messages[0].message.case, "streamClose");
});
