import assert from "node:assert/strict";
import test from "node:test";

import type { CoordinatorFrame, CoordinatorReplyOutcome } from "../../shared/rpc/coordinator-port.js";
import { CoordinatorMainPortCallError, createCoordinatorMainPortClient } from "./coordinator-main-port-client.js";
import { createCoordinatorMainLegs, type CapturedCoordinatorMainSession, type CoordinatorMainPortOwnership } from "./coordinator-main-legs.js";

test("a request post failure settles the pending call and closes the main-data port", async () => {
  const frames: unknown[] = [];
  let closed = 0;
  const client = createCoordinatorMainPortClient({
    post(frame) {
      frames.push(frame);
      if ((frame as { kind?: string }).kind === "request") throw new Error("port is gone");
    },
    close() {
      closed += 1;
    },
  });

  client.handleMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  await assert.rejects(client.legs.setHostSettings({ settings: {} }), /post-failed/);
  assert.equal(closed, 1);
  assert.equal((await client.settled).outcome, "post-failed");
  assert.equal(frames.length, 2);
});

test("a request waits for main-data ready before it is posted", async () => {
  const frames: unknown[] = [];
  const client = createCoordinatorMainPortClient({
    post(frame) { frames.push(frame); },
    close() {},
  });

  const pending = client.legs.setHostSettings({ settings: {} });
  assert.equal(frames.length, 1);
  client.handleMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  assert.equal(frames.length, 2);
  client.handleMessage({ kind: "reply", requestId: "m-1", outcome: { status: "ok", value: { accepted: true } } });
  assert.deepEqual(await pending, { accepted: true });
});

test("a queued request is rejected when the main-data port settles before ready", async () => {
  const client = createCoordinatorMainPortClient({ post() {}, close() {} });
  const pending = client.legs.setHostSettings({ settings: {} });
  client.handlePortClosed();
  await assert.rejects(pending, /port-closed/);
});

test("a hello post failure does not replace the current main-data generation", async () => {
  const failures: string[] = [];
  let closed = 0;
  const legs = createCoordinatorMainLegs({
    onProblem() {},
    reportFailure(leg) {
      failures.push(leg);
    },
  });
  legs.adoptPort({
    postMessage() {
      throw new Error("hello post failed");
    },
    close() {
      closed += 1;
    },
    on() {},
    start() {},
  });

  assert.deepEqual(failures, ["leg-post"]);
  assert.equal(closed, 1);
  await assert.rejects(legs.legs.getHostSettings!(), /no live coordinator session/);
});

test("a protocol breach still settles when the shutdown frame cannot be posted", async () => {
  let closed = 0;
  const client = createCoordinatorMainPortClient({
    post(frame) {
      if ((frame as { phase?: string }).phase === "shutdown") throw new Error("port is gone");
    },
    close() {
      closed += 1;
    },
  });

  client.handleMessage({ kind: "invalid-frame" });
  assert.equal((await client.settled).outcome, "protocol-breach");
  assert.equal(closed, 1);
});

test("dispose settles pending calls when the shutdown frame cannot be posted", async () => {
  let closed = 0;
  const client = createCoordinatorMainPortClient({
    post(frame) {
      if ((frame as { phase?: string }).phase === "shutdown") throw new Error("port is gone");
    },
    close() {
      closed += 1;
    },
  });

  const pending = client.legs.setHostSettings({ settings: {} });
  client.dispose();
  await assert.rejects(pending, /shutdown-requested/);
  assert.equal((await client.settled).outcome, "shutdown-requested");
  assert.equal(closed, 1);
});

test("settlement survives an endpoint close failure", async () => {
  const client = createCoordinatorMainPortClient({
    post() {},
    close() {
      throw new Error("close failed");
    },
  });

  client.handlePortClosed();
  assert.equal((await client.settled).outcome, "port-closed");
});

type MainRequest = Extract<CoordinatorFrame, { kind: "request" }>;
type PortEvent = "message" | "close";

function createPortCarrier(hooks: {
  post?(frame: CoordinatorFrame): void;
  close?(): void;
  on?(event: PortEvent): void;
  start?(): void;
} = {}) {
  const frames: CoordinatorFrame[] = [];
  const listeners: Record<PortEvent, Array<(event: any) => void>> = { message: [], close: [] };
  let closeCount = 0;
  const receive = (value: unknown) => {
    for (const listener of [...listeners.message]) listener({ data: value });
  };
  return {
    frames,
    get closeCount() { return closeCount; },
    postMessage(value: unknown) {
      const frame = value as CoordinatorFrame;
      frames.push(frame);
      hooks.post?.(frame);
    },
    close() { closeCount += 1; hooks.close?.(); },
    on(event: PortEvent, listener: (event: any) => void) {
      listeners[event].push(listener);
      hooks.on?.(event);
    },
    start() { hooks.start?.(); },
    receive,
    ready() { receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 }); },
    reply(request: MainRequest, outcome: CoordinatorReplyOutcome) {
      receive({ kind: "reply", requestId: request.requestId, outcome });
    },
    emitClose() { for (const listener of [...listeners.close]) listener(undefined); },
    requests() { return frames.filter((frame): frame is MainRequest => frame.kind === "request"); },
  };
}

test("MAB transport denied admission stays request-local and uses bounded error text", async () => {
  const port = createPortCarrier();
  const client = createCoordinatorMainPortClient({ post: port.postMessage, close: port.close });
  const guarded = client.bindLifetime(() => { throw new Error("private account and attachment details"); });

  await assert.rejects(guarded.readAttachmentText!({ path: "private-path" }), {
    message: "coordinator main data call lifetime is no longer current",
  });
  client.handleMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
  assert.deepEqual(port.requests(), []);
  assert.equal(port.closeCount, 0);
  assert.doesNotThrow(client.assertLive);

  const cleanup = client.legs.setHostSettings!({ settings: {} });
  const request = port.requests()[0]!;
  assert.equal(request.method, "setHostSettings");
  client.handleMessage({ kind: "reply", requestId: request.requestId, outcome: { status: "ok", value: "cleared" } });
  assert.equal(await cleanup, "cleared");
  client.dispose();
});

test("MAB transport denied queued work leaves ordinary cleanup in the same ready flush", async () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const port = createPortCarrier();
  const ownership = owner.adoptPort(port);
  assert.ok(ownership);
  const lifetime = new AbortController();
  const captured = owner.captureCurrent(() => lifetime.signal.throwIfAborted());
  assert.ok(captured);
  const attachment = captured.legs.readAttachmentChunk!({ path: "a", offset: 0, length: 4 });
  const refused = assert.rejects(attachment, /call lifetime is no longer current/);
  const cleanup = owner.legs.setHostSettings!({ settings: {} });
  assert.deepEqual(port.requests(), []);

  lifetime.abort();
  assert.throws(captured.assertCurrent, /no longer current/);
  assert.doesNotThrow(ownership.assertCurrent);
  assert.equal(owner.captureCurrent(() => lifetime.signal.throwIfAborted()), null);
  port.ready();
  const requests = port.requests();
  assert.deepEqual(requests.map((request) => request.method), ["setHostSettings"]);
  port.reply(requests[0]!, { status: "ok", value: { cleared: true } });
  await refused;
  assert.deepEqual(await cleanup, { cleared: true });
  assert.equal(port.closeCount, 0);
  assert.doesNotThrow(ownership.assertCurrent);
  owner.dispose();
});

test("MAB transport unchanged guard completes queued work on its captured client", async () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const port = createPortCarrier();
  owner.adoptPort(port);
  const lifetime = new AbortController();
  const captured = owner.captureCurrent(() => lifetime.signal.throwIfAborted());
  assert.ok(captured);
  const args = { path: "a", offset: 0, length: 4 };
  const attachment = captured.legs.readAttachmentChunk!(args);
  assert.deepEqual(port.requests(), []);
  assert.doesNotThrow(captured.assertCurrent);

  port.ready();
  const requests = port.requests();
  assert.equal(requests.length, 1);
  assert.equal(requests[0]!.method, "readAttachmentChunk");
  assert.deepEqual(requests[0]!.args, args);
  port.reply(requests[0]!, { status: "ok", value: { bytes: [1, 2, 3, 4] } });
  assert.deepEqual(await attachment, { bytes: [1, 2, 3, 4] });
  assert.doesNotThrow(captured.assertCurrent);
  assert.equal(port.closeCount, 0);
  const deniedByHost = captured.legs.getHostSettings!();
  const noArgsRequest = port.requests()[1]!;
  assert.deepEqual(noArgsRequest.args, {});
  port.reply(noArgsRequest, { status: "failed", failure: { code: "host-failure", message: "unavailable" } });
  await assert.rejects(deniedByHost, (error: unknown) => error instanceof CoordinatorMainPortCallError && error.code === "host-failure");
  owner.dispose();
});

test("MAB transport ready flush rechecks each call and fences an already posted reply", async () => {
  const lifetime = new AbortController();
  const port = createPortCarrier({
    post(frame) {
      if (frame.kind === "request" && frame.method === "readAttachmentText") lifetime.abort();
    },
  });
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  owner.adoptPort(port);
  const captured = owner.captureCurrent(() => lifetime.signal.throwIfAborted());
  assert.ok(captured);
  const first = captured.legs.readAttachmentText!({ path: "first" });
  const second = captured.legs.readAttachmentChunk!({ path: "second", offset: 0, length: 4 });
  const firstRefused = assert.rejects(first, /call lifetime is no longer current/);
  const secondRefused = assert.rejects(second, /call lifetime is no longer current/);
  const cleanup = owner.legs.setHostSettings!({ settings: {} });

  port.ready();
  const requests = port.requests();
  assert.deepEqual(requests.map((request) => request.method), ["readAttachmentText", "setHostSettings"]);
  port.reply(requests[0]!, { status: "ok", value: "old bytes" });
  port.reply(requests[1]!, { status: "ok", value: "cleared" });
  await Promise.all([firstRefused, secondRefused]);
  assert.equal(await cleanup, "cleared");
  assert.equal(port.closeCount, 0);
  owner.dispose();
});

test("MAB transport retired pending calls reject both reply outcomes without consuming other replies", async () => {
  for (const outcome of [
    { status: "ok", value: "old bytes" },
    { status: "failed", failure: { code: "private-code", message: "private-message" } },
  ] satisfies CoordinatorReplyOutcome[]) {
    const lifetime = new AbortController();
    const port = createPortCarrier();
    const owner = createCoordinatorMainLegs({ onProblem() {} });
    owner.adoptPort(port);
    port.ready();
    const captured = owner.captureCurrent(() => lifetime.signal.throwIfAborted());
    assert.ok(captured);
    const attachment = captured.legs.readAttachmentText!({ path: "old" });
    const refused = assert.rejects(attachment, { message: "coordinator main data call lifetime is no longer current" });
    const cleanup = owner.legs.setHostSettings!({ settings: {} });
    const requests = port.requests();
    assert.equal(requests.length, 2);

    lifetime.abort();
    port.receive({ kind: "reply", requestId: "missing-request", outcome });
    port.reply(requests[0]!, outcome);
    port.reply(requests[0]!, outcome);
    port.reply(requests[1]!, { status: "ok", value: "cleared" });
    await refused;
    assert.equal(await cleanup, "cleared");
    assert.equal(port.closeCount, 0);

    const ordinary = owner.legs.getHostSettings!();
    port.reply(port.requests()[2]!, { status: "failed", failure: { code: "host-failure", message: "unavailable" } });
    await assert.rejects(ordinary, (error: unknown) => error instanceof CoordinatorMainPortCallError && error.code === "host-failure");
    owner.dispose();
  }
});

test("MAB transport client live phase is terminal before shutdown and close callbacks", async () => {
  for (const terminal of ["close", "post-failure", "breach", "dispose", "peer-shutdown"] as const) {
    let client: ReturnType<typeof createCoordinatorMainPortClient>;
    let checkedClose = false;
    let checkedShutdown = false;
    let observedSettlement = false;
    client = createCoordinatorMainPortClient({
      post(frame) {
        if (frame.kind === "lifecycle" && frame.phase === "shutdown") {
          assert.throws(client.assertLive, /no longer live/);
          checkedShutdown = true;
        }
        if (terminal === "post-failure" && frame.kind === "request") throw new Error("post failed");
      },
      close() {
        assert.throws(client.assertLive, /no longer live/);
        checkedClose = true;
      },
    });
    void client.settled.then(() => { observedSettlement = true; });
    client.handleMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
    const pending = client.legs.getHostSettings!();
    const refused = assert.rejects(pending, /settled/);
    if (terminal === "close") client.handlePortClosed();
    if (terminal === "breach") client.handleMessage({ kind: "invalid-frame" });
    if (terminal === "dispose") client.dispose();
    if (terminal === "peer-shutdown") client.handleMessage({ kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null });

    assert.equal(observedSettlement, false);
    assert.equal(checkedClose, true);
    assert.equal(checkedShutdown, terminal === "breach" || terminal === "dispose");
    assert.throws(client.assertLive, /no longer live/);
    client.handleMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
    assert.throws(client.assertLive, /no longer live/);
    await refused;
    await client.settled;
  }
});

test("MAB transport captures and ownership fail synchronously at every terminal client phase", async () => {
  for (const terminal of ["close", "post-failure", "breach", "dispose", "peer-shutdown"] as const) {
    const owner = createCoordinatorMainLegs({ onProblem() {} });
    const port = createPortCarrier({
      post(frame) { if (terminal === "post-failure" && frame.kind === "request") throw new Error("post failed"); },
    });
    const ownership = owner.adoptPort(port);
    assert.ok(ownership);
    const captured = owner.captureCurrent(() => {});
    assert.ok(captured);
    port.ready();
    let microtaskRan = false;
    void Promise.resolve().then(() => { microtaskRan = true; });
    const pending = captured.legs.readAttachmentText!({ path: "a" });
    const refused = assert.rejects(pending, /settled/);
    if (terminal === "close") port.emitClose();
    if (terminal === "breach") port.receive({ kind: "invalid-frame" });
    if (terminal === "dispose") ownership.revoke();
    if (terminal === "peer-shutdown") port.receive({ kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null });

    assert.equal(microtaskRan, false);
    assert.throws(captured.assertCurrent, /no longer/);
    assert.throws(ownership.assertCurrent, /no longer/);
    assert.equal(owner.captureCurrent(() => {}), null);
    port.ready();
    assert.equal(owner.captureCurrent(() => {}), null);
    await refused;
    owner.dispose();
  }
});

test("MAB transport same lifetime port replacement never rebinds an old capture", async () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const first = createPortCarrier();
  const second = createPortCarrier();
  const assertLifetime = () => {};
  const firstOwnership = owner.adoptPort(first);
  assert.ok(firstOwnership);
  first.ready();
  const captured = owner.captureCurrent(assertLifetime);
  assert.ok(captured);
  const oldRequest = captured.legs.readAttachmentText!({ path: "a" });
  const oldRefused = assert.rejects(oldRequest, /shutdown-requested/);
  const secondOwnership = owner.adoptPort(second);
  assert.ok(secondOwnership);

  assert.throws(firstOwnership.assertCurrent, /no longer current/);
  assert.throws(captured.assertCurrent, /no longer current/);
  assert.doesNotThrow(secondOwnership.assertCurrent);
  const next = owner.captureCurrent(assertLifetime);
  assert.ok(next);
  const newRequest = next.legs.readAttachmentText!({ path: "a" });
  first.reply(first.requests()[0]!, { status: "ok", value: "stale" });
  first.ready();
  first.emitClose();
  second.ready();
  assert.equal(first.requests().length, 1);
  assert.equal(second.requests().length, 1);
  second.reply(second.requests()[0]!, { status: "ok", value: "current" });
  await oldRefused;
  assert.equal(await newRequest, "current");
  await assert.rejects(captured.legs.readAttachmentText!({ path: "a" }), /call lifetime is no longer current/);
  assert.equal(second.requests().length, 1);
  assert.doesNotThrow(next.assertCurrent);
  assert.doesNotThrow(secondOwnership.assertCurrent);
  owner.dispose();
});

test("MAB transport failed hello preserves a valid previous client", async () => {
  const failures: string[] = [];
  const owner = createCoordinatorMainLegs({ onProblem() {}, reportFailure: (leg) => failures.push(leg) });
  const previous = createPortCarrier();
  const previousOwnership = owner.adoptPort(previous);
  assert.ok(previousOwnership);
  previous.ready();
  const captured = owner.captureCurrent(() => {});
  assert.ok(captured);
  const pending = captured.legs.readAttachmentText!({ path: "a" });
  const candidate = createPortCarrier({ post() { throw new Error("hello failed"); } });

  assert.equal(owner.adoptPort(candidate), null);
  assert.deepEqual(failures, ["leg-post"]);
  assert.equal(candidate.closeCount, 1);
  assert.equal(previous.closeCount, 0);
  assert.doesNotThrow(previousOwnership.assertCurrent);
  assert.doesNotThrow(captured.assertCurrent);
  previous.reply(previous.requests()[0]!, { status: "ok", value: "still live" });
  assert.equal(await pending, "still live");
  owner.dispose();
});

async function exerciseAdoptionInterruption(
  boundary: "hello" | "previous-close" | "message-listener" | "close-listener" | "start",
  interruption: "revoke" | "dispose" | "nested-adopt",
) {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const successor = createPortCarrier();
  const observed: {
    successorOwnership: CoordinatorMainPortOwnership | null;
    candidateCapture: CapturedCoordinatorMainSession | null;
  } = { successorOwnership: null, candidateCapture: null };
  let interrupted = false;
  const interrupt = () => {
    assert.equal(interrupted, false);
    interrupted = true;
    if (boundary === "previous-close") {
      observed.candidateCapture = owner.captureCurrent(() => {});
      assert.ok(observed.candidateCapture);
    }
    if (interruption === "revoke") owner.revoke();
    if (interruption === "dispose") owner.dispose();
    if (interruption === "nested-adopt") observed.successorOwnership = owner.adoptPort(successor);
  };
  const previous = createPortCarrier({
    close() { if (boundary === "previous-close") interrupt(); },
  });
  const previousOwnership = owner.adoptPort(previous);
  assert.ok(previousOwnership);
  previous.ready();
  const previousCapture = owner.captureCurrent(() => {});
  assert.ok(previousCapture);
  const candidate = createPortCarrier({
    post(frame) {
      if (boundary === "hello" && frame.kind === "lifecycle" && frame.phase === "hello") interrupt();
    },
    on(event) {
      if ((boundary === "message-listener" && event === "message") || (boundary === "close-listener" && event === "close")) interrupt();
    },
    start() { if (boundary === "start") interrupt(); },
  });

  assert.equal(owner.adoptPort(candidate), null);
  assert.equal(interrupted, true);
  assert.equal(candidate.closeCount, 1);
  assert.equal(previous.closeCount, 1);
  assert.throws(previousOwnership.assertCurrent, /no longer/);
  assert.throws(previousCapture.assertCurrent, /no longer/);
  if (observed.candidateCapture != null) assert.throws(observed.candidateCapture.assertCurrent, /no longer/);
  previous.emitClose();
  candidate.emitClose();
  candidate.ready();

  if (interruption === "nested-adopt") {
    assert.ok(observed.successorOwnership);
    const ownership = observed.successorOwnership;
    assert.doesNotThrow(ownership.assertCurrent);
    const current = owner.captureCurrent(() => {});
    assert.ok(current);
    successor.ready();
    const pending = current.legs.readAttachmentText!({ path: "successor" });
    assert.equal(successor.requests().length, 1);
    successor.reply(successor.requests()[0]!, { status: "ok", value: "successor bytes" });
    assert.equal(await pending, "successor bytes");
    // Old settlement callbacks have now run; they cannot clear the successor.
    assert.doesNotThrow(current.assertCurrent);
    assert.doesNotThrow(ownership.assertCurrent);
    assert.equal(successor.closeCount, 0);
  } else {
    assert.equal(owner.captureCurrent(() => {}), null);
    await assert.rejects(owner.legs.getHostSettings!(), /no live coordinator session/);
  }
  assert.deepEqual(previous.requests(), []);
  assert.deepEqual(candidate.requests(), []);
  owner.dispose();
}

test("MAB transport hello-time revoke cannot resurrect the outer candidate", async () => {
  await exerciseAdoptionInterruption("hello", "revoke");
});

test("MAB transport hello-time disposal cannot resurrect the outer candidate", async () => {
  await exerciseAdoptionInterruption("hello", "dispose");
});

test("MAB transport hello-time nested adoption keeps only the successor current", async () => {
  await exerciseAdoptionInterruption("hello", "nested-adopt");
});

test("MAB transport previous-close revoke retires the published candidate", async () => {
  await exerciseAdoptionInterruption("previous-close", "revoke");
});

test("MAB transport previous-close disposal retires the published candidate", async () => {
  await exerciseAdoptionInterruption("previous-close", "dispose");
});

test("MAB transport previous-close nested adoption cannot be cleared by old settlement", async () => {
  await exerciseAdoptionInterruption("previous-close", "nested-adopt");
});

test("MAB transport listener and start reentrancy never publishes a retired candidate", async () => {
  for (const boundary of ["message-listener", "close-listener", "start"] as const) {
    for (const interruption of ["revoke", "dispose", "nested-adopt"] as const) {
      await exerciseAdoptionInterruption(boundary, interruption);
    }
  }
});

test("MAB transport listener and start failures close only their owned candidate", async () => {
  for (const boundary of ["message", "close", "start"] as const) {
    for (const nested of [false, true]) {
      const failures: string[] = [];
      const owner = createCoordinatorMainLegs({ onProblem() {}, reportFailure: (leg) => failures.push(leg) });
      const previous = createPortCarrier();
      const successor = createPortCarrier();
      const previousOwnership = owner.adoptPort(previous);
      assert.ok(previousOwnership);
      previous.ready();
      const previousCapture = owner.captureCurrent(() => {});
      assert.ok(previousCapture);
      const observed: { successorOwnership: CoordinatorMainPortOwnership | null } = { successorOwnership: null };
      const fail = () => {
        if (nested) observed.successorOwnership = owner.adoptPort(successor);
        throw new Error("native setup failed");
      };
      const candidate = createPortCarrier({
        on(event) { if (event === boundary) fail(); },
        start() { if (boundary === "start") fail(); },
      });

      assert.equal(owner.adoptPort(candidate), null);
      assert.deepEqual(failures, ["leg-adopt"]);
      assert.equal(candidate.closeCount, 1);
      assert.deepEqual(candidate.requests(), []);
      candidate.emitClose();
      candidate.ready();
      if (nested) {
        assert.ok(observed.successorOwnership);
        assert.doesNotThrow(observed.successorOwnership.assertCurrent);
        assert.throws(previousOwnership.assertCurrent, /no longer/);
        assert.throws(previousCapture.assertCurrent, /no longer/);
        successor.ready();
      } else {
        assert.equal(previous.closeCount, 0);
        assert.doesNotThrow(previousOwnership.assertCurrent);
        assert.doesNotThrow(previousCapture.assertCurrent);
      }
      const current = owner.captureCurrent(() => {});
      assert.ok(current);
      const pending = current.legs.getHostSettings!();
      const livePort = nested ? successor : previous;
      livePort.reply(livePort.requests()[0]!, { status: "ok", value: "live settings" });
      assert.equal(await pending, "live settings");
      assert.doesNotThrow(current.assertCurrent);
      assert.equal(livePort.closeCount, 0);
      owner.dispose();
    }
  }
});

test("MAB transport exact ownership revoke is idempotent and cannot revoke a successor", async () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const first = createPortCarrier();
  const second = createPortCarrier();
  const firstOwnership = owner.adoptPort(first);
  assert.ok(firstOwnership);
  const firstCapture = owner.captureCurrent(firstOwnership.assertCurrent);
  assert.ok(firstCapture);
  const secondOwnership = owner.adoptPort(second);
  assert.ok(secondOwnership);
  const secondCapture = owner.captureCurrent(secondOwnership.assertCurrent);
  assert.ok(secondCapture);
  assert.equal(owner.captureCurrent(firstOwnership.assertCurrent), null);

  firstOwnership.revoke();
  firstOwnership.revoke();
  assert.equal(first.closeCount, 1);
  assert.equal(second.closeCount, 0);
  assert.throws(firstOwnership.assertCurrent, /no longer/);
  assert.throws(firstCapture.assertCurrent, /no longer/);
  assert.doesNotThrow(secondOwnership.assertCurrent);
  const pending = secondCapture.legs.readAttachmentText!({ path: "second" });
  second.ready();
  second.reply(second.requests()[0]!, { status: "ok", value: "second bytes" });
  assert.equal(await pending, "second bytes");

  secondOwnership.revoke();
  secondOwnership.revoke();
  assert.equal(second.closeCount, 1);
  assert.throws(secondOwnership.assertCurrent, /no longer/);
  assert.throws(secondCapture.assertCurrent, /no longer/);
  assert.equal(owner.captureCurrent(() => {}), null);
  const third = createPortCarrier();
  const thirdOwnership = owner.adoptPort(third);
  assert.ok(thirdOwnership);
  firstOwnership.revoke();
  secondOwnership.revoke();
  assert.doesNotThrow(thirdOwnership.assertCurrent);
  assert.equal(third.closeCount, 0);
  owner.dispose();
  assert.throws(thirdOwnership.assertCurrent, /no longer/);
});

test("MAB transport an old ownership revoke during hello cannot retire the new candidate", () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const first = createPortCarrier();
  const firstOwnership = owner.adoptPort(first);
  assert.ok(firstOwnership);
  const second = createPortCarrier({
    post(frame) {
      if (frame.kind === "lifecycle" && frame.phase === "hello") firstOwnership.revoke();
    },
  });
  const secondOwnership = owner.adoptPort(second);
  assert.ok(secondOwnership);
  assert.equal(first.closeCount, 1);
  assert.equal(second.closeCount, 0);
  assert.throws(firstOwnership.assertCurrent, /no longer/);
  assert.doesNotThrow(secondOwnership.assertCurrent);
  owner.dispose();
});

test("MAB transport capture rechecks identity after its opaque guard reenters", () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  const first = createPortCarrier();
  const second = createPortCarrier();
  owner.adoptPort(first);
  const observed: { replacement: CoordinatorMainPortOwnership | null } = { replacement: null };
  const captured = owner.captureCurrent(() => { observed.replacement = owner.adoptPort(second); });
  assert.equal(captured, null);
  assert.ok(observed.replacement);
  assert.doesNotThrow(observed.replacement.assertCurrent);
  assert.equal(first.closeCount, 1);
  assert.equal(second.closeCount, 0);
  assert.deepEqual(first.requests(), []);
  assert.deepEqual(second.requests(), []);
  owner.dispose();
});

test("MAB transport disposed owner closes incoming ports without sending hello", () => {
  const owner = createCoordinatorMainLegs({ onProblem() {} });
  owner.dispose();
  const port = createPortCarrier();
  assert.equal(owner.adoptPort(port), null);
  assert.equal(port.closeCount, 1);
  assert.deepEqual(port.frames, []);
  assert.equal(owner.captureCurrent(() => {}), null);
});
