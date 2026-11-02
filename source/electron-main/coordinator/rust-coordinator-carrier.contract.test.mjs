import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import * as path from "node:path";
import test from "node:test";
import { runInNewContext } from "node:vm";

// NMR-27 / NMR-AC-17: execute the shipping carrier itself. Only Electron's
// transferred ports and the child-process/readline boundary are controlled.
// These are carrier contracts, not live Host/service or packaged acceptance.
const desktopRequire = createRequire(new URL("../../../desktop/package.json", import.meta.url));
const { transformSync } = desktopRequire("esbuild");
const carrierSource = new URL("./rust-coordinator-carrier.ts", import.meta.url);
const carrierCode = transformSync(readFileSync(carrierSource, "utf8"), {
  loader: "ts", format: "cjs", target: "node24", sourcefile: carrierSource.pathname,
}).code;
const channels = ["coordinator-control", "coordinator-data", "coordinator-main-data"];
const copy = (value) => JSON.parse(JSON.stringify(value));

class Port extends EventEmitter {
  received = [];
  starts = 0;
  closes = 0;
  postMessage(value) { this.received.push(copy(value)); }
  start() { this.starts += 1; }
  close() { this.closes += 1; this.emit("close"); }
  send(value) { this.emit("message", { data: value }); }
}

function fixture() {
  const ports = channels.map(() => new Port());
  const requests = [];
  const writes = [];
  const diagnostics = [];
  const output = new EventEmitter();
  let outputCloses = 0;
  output.close = () => { outputCloses += 1; };
  const child = Object.assign(new EventEmitter(), {
    stdin: { write(line) { writes.push(line); return true; } },
    stdout: {}, stderr: new EventEmitter(),
    exitCode: null, signalCode: null, kills: 0,
    kill() { this.kills += 1; this.signalCode = "SIGTERM"; return true; },
  });
  let receiveBootstrap;
  const process = Object.assign(new EventEmitter(), {
    platform: "linux", resourcesPath: "/product/resources", exitCode: undefined,
    env: {
      SAND_PACKAGED: "1",
      SAND_COORDINATOR_BIN: "/native/coordinator",
      MAHAYANA_APP_HOST_BIN: "/native/app-host",
    },
    stderr: { write() {} },
    parentPort: {
      once(event, listener) { assert.equal(event, "message"); receiveBootstrap = listener; },
    },
  });
  runInNewContext(carrierCode, {
    process, __dirname: "/product/dist/node-agent-coordinator",
    console: { error(...args) { diagnostics.push(args); } },
    require(name) {
      if (name === "node:path") return path;
      if (name === "node:child_process") return {
        spawn(executable, args, options) {
          requests.push(copy({ executable, args, options }));
          return child;
        },
      };
      if (name === "node:readline") return {
        createInterface({ input }) { assert.equal(input, child.stdout); return output; },
      };
      throw new Error(`unexpected carrier dependency: ${name}`);
    },
  }, { filename: carrierSource.pathname });
  return {
    ports, child, process, writes, requests, diagnostics,
    get outputCloses() { return outputCloses; },
    bootstrap(data = { bootstrap: { processConfig: { appVersion: "contract", isPackaged: true, dataDir: "/data/account-a" } } }, transferred = ports) {
      receiveBootstrap({ data, ports: transferred });
    },
    stdout(value) { output.emit("line", JSON.stringify(value)); },
    rawStdout(line) { output.emit("line", line); },
    frames() { return writes.map((line) => { assert.ok(line.endsWith("\n")); return JSON.parse(line); }); },
    exit(code, signal = null) {
      child.exitCode = code;
      child.signalCode = signal;
      child.emit("exit", code, signal);
    },
  };
}

function message(overrides = {}) {
  return {
    id: "human-server-message:21", kind: "message", role: "user",
    authorKind: "human", authorId: "local-human", content: "fixture message",
    clientNonce: "nonce-21", remoteMessageId: "21", delivery: "sent",
    timestampMs: 1793610000000, remoteCreatedAt: "2026-11-02T09:00:00Z",
    reactions: [], ...overrides,
  };
}

test("NMR-ELECTRON-01: bootstrap binds one native child and all three existing ports", () => {
  const f = fixture();
  assert.equal(f.requests.length, 0);
  f.bootstrap();
  assert.deepEqual(f.requests, [{
    executable: "/native/coordinator",
    args: ['--bootstrap={"processConfig":{"appVersion":"contract","isPackaged":true,"dataDir":"/data/account-a"}}'],
    options: { stdio: ["pipe", "pipe", "pipe"], env: f.process.env, windowsHide: true },
  }]);
  assert.deepEqual(f.ports.map((port) => port.starts), [1, 1, 1]);
  assert.equal(f.writes.length, 0, "carrier must not invent a message/history request");
});

test("NMR-ELECTRON-02: transcript windows preserve cursor, stable IDs and channel identity", () => {
  const f = fixture();
  f.bootstrap();
  const first = { kind: "request", requestId: "page-1", method: "getHumanConversationTranscriptTail", args: { conversationId: "human-a", limit: 200 } };
  const older = { kind: "request", requestId: "page-2", method: "getHumanConversationTranscriptTail", args: { conversationId: "human-a", beforeSeq: 6401, limit: 200 } };
  const agent = { kind: "request", requestId: "page-agent", method: "getAgentTranscriptWindow", args: { id: "agent-a", beforeSeq: 41, limit: 200 } };
  f.ports[1].send(first);
  f.ports[1].send(older);
  f.ports[1].send(agent);
  const page = { kind: "reply", requestId: "page-2", outcome: { status: "ok", value: {
    entries: [message({ id: "human-server-message:6201", remoteMessageId: "6201" }), message({ id: "human-server-message:6400", remoteMessageId: "6400" })],
    nextBeforeSeq: 6201,
  } } };
  const agentPage = { kind: "reply", requestId: "page-agent", outcome: { status: "ok", value: {
    entries: [{ id: "agent-result-40", kind: "send-message", content: "Agent result" }],
    nextBeforeSeq: 40, threadCounts: { "agent-result-40": 2 },
  } } };
  f.stdout({ channel: "coordinator-data", frame: page });
  f.stdout({ channel: "coordinator-data", frame: agentPage });
  const control = { kind: "event", family: "transport-connected", payload: { id: "human-a" } };
  const main = { kind: "reply", requestId: "main-1", outcome: { status: "ok", value: { active: true } } };
  f.stdout({ channel: "coordinator-control", frame: control });
  f.stdout({ channel: "coordinator-main-data", frame: main });
  assert.deepEqual(f.frames(), [first, older, agent].map((frame) => ({ channel: "coordinator-data", frame })));
  assert.deepEqual(f.ports.map((port) => port.received), [[control], [page, agentPage], [main]]);
});

test("NMR-ELECTRON-03: opaque native replies/events retain identity, reactions and receipts; unsupported edits/deletes stay errors", () => {
  const f = fixture();
  f.bootstrap();
  const operations = [
    ["sendHumanMessage", { conversationId: "human-a", text: "fixture message", clientNonce: "nonce-21", composedAtMs: 21, attachments: [], replyToId: "human-server-message:2" }],
    ["editHumanMessage", { conversationId: "human-a", entryId: "human-server-message:21", text: "edited" }],
    ["deleteHumanMessage", { conversationId: "human-a", entryId: "human-server-message:21" }],
    ["reactHumanMessage", { conversationId: "human-a", entryId: "human-server-message:21", emoji: "👍🏽" }],
  ].map(([method, args], index) => ({ kind: "request", requestId: `mutation-${index}`, method, args }));
  operations.forEach((frame) => f.ports[1].send(frame));
  const states = [
    message(),
    // The current production Session gateway does not implement these two
    // methods. A carrier contract must not imply that a fixture implements them.
    null,
    null,
    message({ reactions: [{ emoji: "👍🏽", by: "me" }], remoteReactions: [{ emoji: "👍🏽", count: 1, reactedByMe: true }], readAt: "2026-11-02T09:02:00Z" }),
  ];
  const frames = [];
  states.forEach((entry, index) => {
    const outcome = entry == null
      ? { status: "failed", failure: { code: "unknown-method", message: "fixture unsupported native mutation" } }
      : { status: "ok", value: entry };
    const reply = { kind: "reply", requestId: `mutation-${index}`, outcome };
    frames.push(reply);
    f.stdout({ channel: "coordinator-data", frame: reply });
    if (entry == null) return;
    const event = { kind: "event", family: "transcript", payload: { type: index === 0 ? "appended" : "updated", agentId: "human-a", entry } };
    frames.push(event);
    f.stdout({ channel: "coordinator-data", frame: event });
  });
  assert.deepEqual(f.frames(), operations.map((frame) => ({ channel: "coordinator-data", frame })));
  assert.deepEqual(f.ports[1].received, frames);
  assert.equal(f.ports[0].received.length + f.ports[2].received.length, 0);
});

test("NMR-ELECTRON-04: cancellation and native failure outcomes remain explicit", () => {
  const f = fixture();
  f.bootstrap();
  const cancel = { kind: "cancel", requestId: "pending-send" };
  f.ports[1].send(cancel);
  const errors = ["cancelled", "permission-denied", "unknown-method", "network-failed"].map((code) => ({
    kind: "reply", requestId: "pending-send", outcome: { status: "failed", failure: { code, message: "fixture refusal", transportKind: "host" } },
  }));
  errors.forEach((frame) => f.stdout({ channel: "coordinator-data", frame }));
  assert.deepEqual(f.frames(), [{ channel: "coordinator-data", frame: cancel }]);
  assert.deepEqual(f.ports[1].received, errors, "carrier must not replace failure with fallback success");
});

test("NMR-ELECTRON-05: invalid output cannot reach a subscriber or log private payloads", () => {
  const f = fixture();
  f.bootstrap();
  const privateMarker = "fixture-private-message-and-token";
  f.rawStdout(`invalid-json ${privateMarker}`);
  f.stdout({ channel: "not-a-coordinator-channel", frame: { text: privateMarker } });
  f.stdout({ channel: "coordinator-data", text: privateMarker });
  f.stdout([{ channel: "coordinator-data", frame: privateMarker }]);
  f.rawStdout("  ");
  assert.deepEqual(f.ports.map((port) => port.received), [[], [], []]);
  assert.equal(f.diagnostics.length, 4);
  assert.ok(!JSON.stringify(f.diagnostics).includes(privateMarker));
  const valid = { kind: "event", family: "transcript", payload: { type: "cleared", agentId: "human-a" } };
  f.stdout({ channel: "coordinator-data", frame: valid });
  assert.deepEqual(f.ports[1].received, [valid], "malformed records cannot consume a later valid event");
});

test("NMR-ELECTRON-06: closed subscriptions fence late traffic and all-port disposal retires the child once", () => {
  const f = fixture();
  f.bootstrap();
  f.ports[1].emit("close");
  f.ports[1].emit("close");
  const late = { kind: "event", family: "transcript", payload: { type: "updated", agentId: "human-a", entry: message() } };
  f.stdout({ channel: "coordinator-data", frame: late });
  f.ports[1].send({ kind: "request", requestId: "late", method: "sendHumanMessage", args: {} });
  assert.equal(f.ports[1].received.length, 0);
  assert.equal(f.writes.length, 0);
  assert.equal(f.child.kills, 0, "one closed subscription must not retire live control/main ports");
  const control = { kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null };
  f.ports[0].send(control);
  assert.deepEqual(f.frames(), [{ channel: "coordinator-control", frame: control }]);
  f.ports[0].emit("close");
  assert.equal(f.child.kills, 0);
  f.ports[2].emit("close");
  assert.equal(f.child.kills, 1);
  assert.equal(f.outputCloses, 1);
  assert.equal(f.process.exitCode, 0);
  assert.deepEqual(f.ports.map((port) => port.closes), [1, 1, 1]);
  for (let index = 0; index < channels.length; index += 1) {
    f.stdout({ channel: channels[index], frame: late });
    f.ports[index].send(control);
  }
  assert.deepEqual(f.ports.map((port) => port.received), [[], [], []]);
  assert.equal(f.writes.length, 1);
  assert.equal(f.child.kills, 1);
});

test("NMR-ELECTRON-07: child errors close subscriptions and reject late delivery", () => {
  const f = fixture();
  f.bootstrap();
  f.child.emit("error", new Error("fixture child failure"));
  assert.equal(f.process.exitCode, 1);
  assert.equal(f.child.kills, 1);
  assert.equal(f.outputCloses, 1);
  assert.deepEqual(f.ports.map((port) => port.closes), [1, 1, 1]);
  f.stdout({ channel: "coordinator-data", frame: { kind: "reply", requestId: "late", outcome: { status: "ok", value: message() } } });
  assert.equal(f.ports[1].received.length, 0);
  f.exit(1);
  assert.equal(f.outputCloses, 1, "duplicate termination cannot repeat disposal");
});

test("NMR-ELECTRON-08: native child exit preserves failure status without killing an exited process", () => {
  const f = fixture();
  f.bootstrap();
  f.exit(7);
  assert.equal(f.process.exitCode, 7);
  assert.equal(f.child.kills, 0);
  assert.equal(f.outputCloses, 1);
  assert.deepEqual(f.ports.map((port) => port.closes), [1, 1, 1]);
});

test("NMR-ELECTRON-09: signal termination remains non-successful", () => {
  const f = fixture();
  f.bootstrap();
  f.exit(null, "SIGTERM");
  assert.equal(f.process.exitCode, 1);
  assert.equal(f.child.kills, 0);
  assert.deepEqual(f.ports.map((port) => port.closes), [1, 1, 1]);
});

test("NMR-ELECTRON-10: absent bootstrap or incomplete port handoff cannot launch a child", () => {
  const missing = fixture();
  assert.throws(() => missing.bootstrap({}), /no bootstrap payload/);
  assert.equal(missing.requests.length, 0);
  const incomplete = fixture();
  assert.throws(() => incomplete.bootstrap({ bootstrap: {} }, incomplete.ports.slice(0, 2)), /requires 3 transferred ports/);
  assert.equal(incomplete.requests.length, 0);
});
