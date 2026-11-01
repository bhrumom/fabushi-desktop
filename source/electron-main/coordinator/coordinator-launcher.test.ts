import assert from "node:assert/strict";
import test from "node:test";

import {
  COORDINATOR_SERVICE_NAME,
  launchCoordinator,
  type CoordinatorChildProcess,
  type CoordinatorMessageChannel,
  type CoordinatorMessagePort,
} from "./coordinator-launcher.js";

class FakePort implements CoordinatorMessagePort {
  readonly posted: unknown[] = [];
  private readonly messageListeners: Array<(event: { readonly data: unknown }) => void> = [];
  private readonly closeListeners: Array<() => void> = [];
  startCount = 0;
  closeCount = 0;

  postMessage(value: unknown): void { this.posted.push(value); }
  on(event: "message" | "close", listener: ((event: { readonly data: unknown }) => void) | (() => void)): void {
    if (event === "message") this.messageListeners.push(listener as (event: { readonly data: unknown }) => void);
    else this.closeListeners.push(listener as () => void);
  }
  start(): void { this.startCount += 1; }
  close(): void {
    this.closeCount += 1;
    for (const listener of this.closeListeners) listener();
  }
  emitMessage(data: unknown): void {
    for (const listener of this.messageListeners) listener({ data });
  }
}

class FakeChild implements CoordinatorChildProcess {
  readonly posts: Array<{ value: unknown; ports: CoordinatorMessagePort[] }> = [];
  private readonly exitListeners: Array<(code: number | null) => void> = [];
  killCount = 0;

  postMessage(value: unknown, ports: CoordinatorMessagePort[]): void {
    this.posts.push({ value, ports });
  }
  on(event: "exit", listener: (code: number | null) => void): void {
    assert.equal(event, "exit");
    this.exitListeners.push(listener);
  }
  kill(): void { this.killCount += 1; }
  exit(code: number | null): void {
    for (const listener of this.exitListeners) listener(code);
  }
}

function fakeChannel(): CoordinatorMessageChannel {
  return { port1: new FakePort(), port2: new FakePort() };
}

test("launcher transfers three ports and completes Grok control handshake", async () => {
  const child = new FakeChild();
  const channels: CoordinatorMessageChannel[] = [];
  const problems: string[] = [];
  const handle = launchCoordinator({
    fork(artifactPath, options) {
      assert.equal(artifactPath, "/runtime/coordinator");
      assert.equal(options.serviceName, COORDINATOR_SERVICE_NAME);
      return child;
    },
    artifactPath: "/runtime/coordinator",
    createChannel() {
      const value = fakeChannel();
      channels.push(value);
      return value;
    },
    executors: {
      echo: (args) => ({ args }),
    },
    onEvent: {
      "transport-connected": () => {},
      "transport-down": () => {},
      "agents-event": () => {},
      "agents-roster-seed": () => {},
    },
    onProblem: (problem) => problems.push(problem),
    processConfig: { appVersion: "1.2.75", isPackaged: true, dataDir: "/tmp/fabushi" },
  });

  assert.equal(channels.length, 3);
  assert.equal(child.posts.length, 1);
  assert.deepEqual(child.posts[0]?.value, {
    bootstrap: { processConfig: { appVersion: "1.2.75", isPackaged: true, dataDir: "/tmp/fabushi" } },
  });
  assert.deepEqual(child.posts[0]?.ports, [channels[0]?.port1, channels[1]?.port1, channels[2]?.port1]);
  assert.equal(handle.rendererDataPort, channels[1]?.port2);
  assert.equal(handle.mainDataPort, channels[2]?.port2);

  const control = channels[0]?.port2 as FakePort;
  assert.equal(control.startCount, 1);
  control.emitMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
  assert.deepEqual(control.posted[0], { kind: "lifecycle", phase: "ready", protocolVersion: 1 });

  control.emitMessage({ kind: "request", requestId: "r1", method: "echo", args: { value: 7 } });
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(control.posted[1], {
    kind: "reply",
    requestId: "r1",
    outcome: { status: "ok", value: { args: { value: 7 } } },
  });
  assert.deepEqual(problems, []);

  child.exit(17);
  assert.deepEqual(await handle.processExited, { code: 17 });
  assert.equal((channels[0]?.port2 as FakePort).closeCount, 1);
  assert.equal((channels[1]?.port2 as FakePort).closeCount, 1);
  assert.equal((channels[2]?.port2 as FakePort).closeCount, 1);
});

test("launcher fails closed on repeated hello and dispose kills the child", async () => {
  const child = new FakeChild();
  const channels: CoordinatorMessageChannel[] = [];
  const problems: string[] = [];
  const handle = launchCoordinator({
    fork: () => child,
    artifactPath: "/runtime/coordinator",
    createChannel() {
      const value = fakeChannel();
      channels.push(value);
      return value;
    },
    executors: {},
    onEvent: {
      "transport-connected": () => {},
      "transport-down": () => {},
      "agents-event": () => {},
      "agents-roster-seed": () => {},
    },
    onProblem: (problem) => problems.push(problem),
    processConfig: {},
  });
  const control = channels[0]?.port2 as FakePort;
  control.emitMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
  control.emitMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
  assert.match(problems[0] ?? "", /repeated hello/);
  assert.deepEqual(control.posted.at(-1), {
    kind: "lifecycle",
    phase: "shutdown",
    reason: "protocol-error",
    detail: "coordinator repeated hello on the control port",
  });

  handle.dispose();
  await handle.controlSettled;
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(child.killCount, 1);
});
