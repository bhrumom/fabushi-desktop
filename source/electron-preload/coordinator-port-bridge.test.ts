import assert from "node:assert/strict";
import test from "node:test";

import {
  createCoordinatorPortBroker,
  wrapTransferredCoordinatorPort,
} from "./coordinator-port-bridge.js";

test("coordinator port broker enforces single renderer ownership", () => {
  let requests = 0;
  const delivered: unknown[] = [];
  const broker = createCoordinatorPortBroker<unknown>({
    invokeRequest: () => { requests += 1; },
  });
  const firstConsumer = { onPort: (port: unknown) => delivered.push(port) };
  const secondConsumer = { onPort: (_port: unknown) => {} };

  const first = broker.bridge.claim(firstConsumer);
  assert.ok(first);
  assert.equal(broker.bridge.claim(secondConsumer), null);

  first.request();
  assert.equal(requests, 1);
  broker.deliver({ id: "port-1" });
  assert.deepEqual(delivered, [{ id: "port-1" }]);

  first.release();
  first.request();
  assert.equal(requests, 1, "released owner cannot request another port");
  broker.deliver({ id: "ignored" });
  assert.equal(delivered.length, 1);

  const second = broker.bridge.claim(secondConsumer);
  assert.ok(second);
  second.release();
});

test("transferred coordinator port preserves message and close semantics", () => {
  const listeners = new Map<string, (...args: any[]) => void>();
  const posted: unknown[] = [];
  let started = 0;
  let closed = 0;
  const raw = {
    postMessage(message: unknown) { posted.push(message); },
    close() { closed += 1; },
    start() { started += 1; },
    addEventListener(type: "message" | "close", listener: (...args: any[]) => void) {
      listeners.set(type, listener);
    },
  };

  const port = wrapTransferredCoordinatorPort(raw);
  const messages: unknown[] = [];
  let closeEvents = 0;
  port.addEventListener("message", (event) => {
    if ("data" in event) messages.push(event.data);
  });
  port.addEventListener("close", () => { closeEvents += 1; });
  port.start();
  port.postMessage({ kind: "request" });

  listeners.get("message")?.({ data: { kind: "reply" } });
  listeners.get("close")?.();
  port.close();

  assert.equal(started, 1);
  assert.equal(closed, 1);
  assert.deepEqual(posted, [{ kind: "request" }]);
  assert.deepEqual(messages, [{ kind: "reply" }]);
  assert.equal(closeEvents, 1);
});
