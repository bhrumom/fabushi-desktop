import assert from "node:assert/strict";
import test from "node:test";

import {
  COORDINATOR_CONTROL_METHODS,
  createCoordinatorControlExecutors,
} from "./coordinator-executors.js";

test("control executor exposes the frozen Grok platform command surface", () => {
  assert.deepEqual(COORDINATOR_CONTROL_METHODS, [
    "resolveGatewayConnection",
    "listRoutedMcpTools",
    "executeRoutedMcpTool",
    "mintLocalExecDaemonCredential",
    "requestWebAuthnConsent",
    "requestWebAuthnPin",
    "updateWebAuthnConsent",
    "finishWebAuthnConsent",
    "spawnLocalExecDaemon",
    "terminateProcess",
    "isProcessAlive",
    "getProcessIdentity",
    "waitLocalExecDaemonExit",
    "getRpcTraceWindowTraceparent",
    "reportTransportStage",
    "reportGatewayCommandSpan",
    "reportGatewayReachability",
    "reportGatewayDnsDiagnostic",
    "reportProcessCrash",
  ]);
});

test("control executor routes platform work without absorbing Host or Runner behavior", async () => {
  const calls: string[] = [];
  const identity = {
    pid: 77,
    startEpochMs: 1_000,
    command:
      "/bin/node /app/local-exec --sand-local-exec-generation=g-1",
    entryRealpath: "/app/local-exec",
    generationToken: "g-1",
  };
  const executors = createCoordinatorControlExecutors({
    gateway: {
      resolveConnection: () => {
        calls.push("gateway");
        return { baseUrl: "https://127.0.0.1:43123" };
      },
      mintLocalExecDaemonCredential: () => ({ token: "secret" }),
    },
    mcp: {
      listRoutedMcpTools: () => [{ name: "tool" }],
      executeRoutedMcpTool: (request) => ({ request }),
    },
    webauthnPrompt: {
      requestConsent: (args) => ({ approved: true, args }),
      requestPin: () => "1234",
      update: (status) => calls.push(`status:${status}`),
      finish: () => calls.push("finish"),
    },
    localExec: {
      spawnLocalExecDaemon: ({ logPath, env }) => {
        assert.equal(logPath, "/tmp/local-exec.log");
        assert.equal(env.SAND_PACKAGED, "1");
        return identity;
      },
      terminateProcess: (expected) => expected.pid === identity.pid,
      isProcessAlive: (pid) => pid === identity.pid,
      getProcessIdentity: (expected) =>
        expected.pid === identity.pid ? identity : null,
      waitLocalExecDaemonExit: (expected) => ({
        identity: expected,
        exitCode: 0,
      }),
    },
    telemetry: {
      getRpcTraceWindowTraceparent: () => "00-trace-span-01",
      reportProcessCrash: () => calls.push("crash"),
    },
  });

  assert.deepEqual(await executors.resolveGatewayConnection({}), {
    baseUrl: "https://127.0.0.1:43123",
  });
  assert.deepEqual(
    await executors.mintLocalExecDaemonCredential({}),
    { token: "secret" },
  );
  assert.deepEqual(
    await executors.spawnLocalExecDaemon({
      logPath: "/tmp/local-exec.log",
      env: { SAND_PACKAGED: "1" },
    }),
    identity,
  );
  assert.equal(await executors.isProcessAlive({ pid: 77 }), true);
  assert.deepEqual(
    await executors.getProcessIdentity({
      pid: 77,
      entryRealpath: "/app/local-exec",
      generationToken: "g-1",
    }),
    identity,
  );
  assert.deepEqual(
    await executors.terminateProcess({ identity }),
    { terminated: true },
  );
  assert.deepEqual(
    await executors.waitLocalExecDaemonExit(identity),
    { identity, exitCode: 0 },
  );
  await executors.updateWebAuthnConsent({
    status: "Touch your security key now",
  });
  await executors.finishWebAuthnConsent({});
  await executors.reportProcessCrash({ kind: "panic" });
  assert.equal(
    await executors.getRpcTraceWindowTraceparent({}),
    "00-trace-span-01",
  );
  assert.deepEqual(calls, [
    "gateway",
    "status:Touch your security key now",
    "finish",
    "crash",
  ]);
});

test("control executor rejects malformed native process requests before delegation", async () => {
  const executors = createCoordinatorControlExecutors({
    gateway: { resolveConnection: () => ({}) },
    mcp: {
      listRoutedMcpTools: () => [],
      executeRoutedMcpTool: () => null,
    },
    webauthnPrompt: {
      requestConsent: () => ({ approved: false }),
      requestPin: () => null,
      update: () => undefined,
      finish: () => undefined,
    },
    localExec: {
      spawnLocalExecDaemon: () => {
        throw new Error("must not delegate");
      },
      terminateProcess: () => false,
      isProcessAlive: () => false,
      getProcessIdentity: () => null,
    },
  });

  await assert.rejects(
    async () => await executors.spawnLocalExecDaemon({
      logPath: "",
      env: { SAND_PACKAGED: "1" },
    }),
    /logPath must be a non-empty string/,
  );
  await assert.rejects(
    async () => await executors.terminateProcess({
      identity: { pid: 0 },
    }),
    /pid must be a positive integer/,
  );
});
