import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHostClipboardPasteScript,
  resolveHostToBoxSync,
} from "./box-vnc-clipboard-paste.js";
import { createViewerVisibilityGate } from "./box-vnc-visibility-gate.js";
import {
  EDGE_HANDLER_FAILED,
  EDGE_UNKNOWN_METHOD,
  EdgeCallFailure,
  bridgeRpcEdge,
  eventChannel,
  methodChannel,
} from "./rpc-edge-runtime.js";
import {
  MAIN_RPC_CONTRACT_NAME,
  MAIN_RPC_METHOD_TABLE,
  isMainMethod,
} from "./main-rpc-runtime.js";

test("RPC edge uses Grok channel names, envelopes, and event cleanup", async () => {
  const invoked: Array<{ channel: string; payload: unknown }> = [];
  const subscriptions = new Map<string, (payload: unknown) => void>();
  const disposed: string[] = [];
  const bridge = bridgeRpcEdge(
    "main",
    { ping: { args: "none" }, echo: { args: "object" } },
    {
      async invoke(channel, payload) {
        invoked.push({ channel, payload });
        if (channel.endsWith(":ping")) return { ok: true, value: "pong" };
        return { ok: true, value: payload };
      },
      on(channel, listener) {
        subscriptions.set(channel, listener);
        return () => disposed.push(channel);
      },
    },
    true,
  );

  assert.equal(await bridge.ping(), "pong");
  assert.deepEqual(await bridge.echo({ value: 7 }), { value: 7 });
  assert.deepEqual(invoked, [
    { channel: "sand-rpc:main:m:ping", payload: {} },
    { channel: "sand-rpc:main:m:echo", payload: { value: 7 } },
  ]);

  const events: unknown[] = [];
  const unsubscribe = bridge.subscribe({ changed: (payload: unknown) => events.push(payload) });
  subscriptions.get("sand-rpc:main:e:changed")?.({ value: 9 });
  assert.deepEqual(events, [{ value: 9 }]);
  unsubscribe();
  assert.deepEqual(disposed, ["sand-rpc:main:e:changed"]);
  assert.equal(methodChannel("main", "echo"), "sand-rpc:main:m:echo");
  assert.equal(eventChannel("main", "changed"), "sand-rpc:main:e:changed");
});

test("RPC edge fails closed on transport and malformed envelopes", async () => {
  const unknown = bridgeRpcEdge(
    "main",
    { ping: { args: "none" } },
    {
      async invoke() { throw new Error("missing"); },
      on() { return () => {}; },
    },
  );
  await assert.rejects(
    () => unknown.ping(),
    (error: unknown) => error instanceof EdgeCallFailure && error.code === EDGE_UNKNOWN_METHOD,
  );

  const malformed = bridgeRpcEdge(
    "main",
    { ping: { args: "none" } },
    {
      async invoke() { return { value: "outside-envelope" }; },
      on() { return () => {}; },
    },
  );
  await assert.rejects(
    () => malformed.ping(),
    (error: unknown) => error instanceof EdgeCallFailure && error.code === EDGE_HANDLER_FAILED,
  );
});

test("VNC clipboard helper preserves text literally and only reports successful sync", () => {
  const text = "quote \" slash \\ newline\n<>&";
  const script = buildHostClipboardPasteScript(text);
  assert.match(script, /clipboardPasteFrom/);
  assert.ok(script.includes(JSON.stringify(text)));
  assert.equal(resolveHostToBoxSync(text, true), text);
  assert.equal(resolveHostToBoxSync(text, false), null);
  assert.equal(resolveHostToBoxSync("", true), null);
});

test("viewer visibility gate reports only hidden-to-visible transitions", () => {
  const gate = createViewerVisibilityGate();
  assert.equal(gate.isVisible(), false);
  assert.equal(gate.update(true), true);
  assert.equal(gate.isVisible(), true);
  assert.equal(gate.update(true), false);
  assert.equal(gate.update(false), false);
  assert.equal(gate.isVisible(), false);
  assert.equal(gate.update(true), true);
});

test("main RPC runtime exposes the frozen shared registry", () => {
  assert.equal(MAIN_RPC_CONTRACT_NAME, "main");
  assert.equal(isMainMethod("openExternal"), true);
  assert.equal(isMainMethod("definitelyNotMain"), false);
  assert.equal(MAIN_RPC_METHOD_TABLE.openExternal.args, "object");
  assert.equal(MAIN_RPC_METHOD_TABLE.getWindowState.args, "none");
});
