import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test, { type TestContext } from "node:test";
import { pathToFileURL } from "node:url";
import { MessageChannel, type MessagePort } from "node:worker_threads";

import {
  createCoordinatorAccountRuntime,
  type CoordinatorAccountBinding,
  type CoordinatorAccountRuntime,
  type CoordinatorAccountTransition,
  type CoordinatorAuthStatus,
} from "./coordinator-account-runtime.js";
import type { CoordinatorRuntime } from "./coordinator-runtime.js";
import { createProductionCoordinatorAdapter } from "./production-provider.js";
import { createCoordinatorMainLegs } from "./coordinator-main-legs.js";
import type { CoordinatorChildProcess, CoordinatorMessageChannel, CoordinatorMessagePort } from "./coordinator-launcher.js";
import { createProductionMcpOAuthAdapter } from "../adapters/mcp-oauth.js";
import { createEgressConnectionObserver } from "../box/remote-connector-egress.js";
import type { DesktopMcpManagerFacade } from "../mcp/desktop-mcp-manager.js";
import type { ProductionCoordinatorService, ProductionMcpService, ProductionServiceContext } from "../main-production-services.js";
import { McpResult, McpSuccess } from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import {
  parseMcpRoutedOrigin,
  type CoordinatorControlSourceContext,
  type CoordinatorFrame,
  type CoordinatorReplyOutcome,
} from "../../shared/rpc/coordinator-port.js";
import { MCP_ORIGIN_RETIRED, McpOriginRetiredError } from "../../shared/node/mcp/mcp-result-factory.js";

interface Status extends CoordinatorAuthStatus {
  readonly kind: "logged-in" | "logged-out";
}

const loggedOut: Status = { kind: "logged-out" };
const loggedIn = (slot: string, email = `${slot}@example.test`): Status => ({
  kind: "logged-in",
  authId: slot,
  email,
});
const options = { timeout: 10_000 };
const turn = () => new Promise<void>((resolve) => setImmediate(resolve));

class ControlledRuntime implements CoordinatorRuntime {
  readonly disposeStarted = Promise.withResolvers<void>();
  readonly port: { readonly runtime: number };
  exit: Promise<void> = Promise.resolve();
  disposeError: Error | undefined;
  onRevoke: (() => void) | undefined;
  onDispose: (() => void) | undefined;
  disposeCalls = 0;
  revokeCalls = 0;
  rendererRequests = 0;
  restartCalls = 0;

  constructor(readonly index: number, readonly events: string[]) {
    this.port = { runtime: index };
  }

  requestRendererPort(sink: (port: unknown) => void): void {
    this.rendererRequests += 1;
    this.events.push(`renderer:${this.index}`);
    sink(this.port);
  }

  revokeRendererPortRequest(): void {
    this.revokeCalls += 1;
    this.events.push(`revoke-renderer:${this.index}`);
    this.onRevoke?.();
  }

  restart(): Promise<void> {
    this.restartCalls += 1;
    this.events.push(`restart:${this.index}`);
    return Promise.resolve();
  }

  dispose(): Promise<void> {
    this.disposeCalls += 1;
    this.events.push(`dispose:${this.index}`);
    this.disposeStarted.resolve();
    this.onDispose?.();
    if (this.disposeError !== undefined) throw this.disposeError;
    return this.exit;
  }
}

interface Hooks {
  onCreate?(runtime: ControlledRuntime): void;
  authorize?(slot: string, transition: CoordinatorAccountTransition): Promise<boolean>;
  prepare?(transition: { previousSlot: string; nextSlot: string | null }): Promise<void>;
  onRevokeMain?(): void;
  onReset?(): void;
  onProblem?(): void;
}

function createHarness() {
  const hooks: Hooks = {};
  const events: string[] = [];
  const runtimes: ControlledRuntime[] = [];
  const authorizations: string[] = [];
  const revokedRuntimes: CoordinatorRuntime[] = [];
  const statuses: Status[] = [];
  const problems: string[] = [];
  const timeouts: number[] = [];
  const deadlines: Array<{ readonly signal: AbortSignal; expire(): void }> = [];
  const owner = createCoordinatorAccountRuntime<Status>({
    createRuntime() {
      const runtime = new ControlledRuntime(runtimes.length + 1, events);
      runtimes.push(runtime);
      events.push(`create:${runtime.index}`);
      hooks.onCreate?.(runtime);
      return runtime;
    },
    async authorizeAccount(slot, transition) {
      authorizations.push(slot);
      events.push(`authorize:${slot}`);
      return await (hooks.authorize?.(slot, transition) ?? Promise.resolve(true));
    },
    async revokeRefusedAccount() {
      events.push("revoke-refused");
      return { kind: "revoked", status: loggedOut };
    },
    async prepareAccountTransition(transition) {
      events.push(`prepare:${transition.previousSlot}->${transition.nextSlot}`);
      await hooks.prepare?.(transition);
      events.push("prepared");
    },
    resetAccountState() {
      events.push("reset");
      hooks.onReset?.();
    },
    revokeMainDataPort(runtime) {
      revokedRuntimes.push(runtime);
      events.push("revoke-main");
      hooks.onRevokeMain?.();
    },
    deliverStatus(status) {
      statuses.push(status);
      events.push(`status:${status.authId ?? status.kind}`);
    },
    onProblem(problem) {
      problems.push(problem);
      hooks.onProblem?.();
    },
    onExitTimeout(ms) {
      timeouts.push(ms);
    },
    exitTimeoutMs: 25,
    delay(_ms, signal) {
      const completion = Promise.withResolvers<void>();
      deadlines.push({ signal, expire: () => completion.resolve() });
      if (signal.aborted) completion.resolve();
      else signal.addEventListener("abort", () => completion.resolve(), { once: true });
      return completion.promise;
    },
  });
  return { owner, hooks, runtimes, events, authorizations, revokedRuntimes, statuses, problems, timeouts, deadlines };
}

function requireBinding(owner: CoordinatorAccountRuntime<Status>): CoordinatorAccountBinding {
  const binding = owner.captureCurrentBinding();
  assert.ok(binding, "the authorized active runtime must supply a binding");
  return binding;
}

function assertRetired(binding: CoordinatorAccountBinding): void {
  assert.throws(() => binding.assertCurrent(), {
    name: "Error",
    message: "Coordinator account binding is no longer current.",
  });
}

function bindingFailure(binding: CoordinatorAccountBinding): unknown {
  try {
    binding.assertCurrent();
    return undefined;
  } catch (error) {
    return error;
  }
}

test("MAB account capture waits for authorization and identifies the exact active runtime", options, async () => {
  const h = createHarness();
  const authorization = Promise.withResolvers<boolean>();
  const requested = Promise.withResolvers<void>();
  h.hooks.authorize = () => {
    requested.resolve();
    return authorization.promise;
  };
  assert.equal(h.owner.captureCurrentBinding(), null);
  const startup = h.owner.start(loggedIn("A"));
  await requested.promise;
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 0);
  authorization.resolve(true);
  await startup;
  const binding = requireBinding(h.owner);
  assert.equal(binding.runtimeIdentity, h.runtimes[0]);
  binding.assertCurrent();
  assert.deepEqual(h.authorizations, ["A"]);
  let disposalCapture: CoordinatorAccountBinding | null | undefined;
  let disposalFailure: unknown;
  h.hooks.onRevokeMain = () => {
    disposalCapture = h.owner.captureCurrentBinding();
    disposalFailure = bindingFailure(binding);
  };
  await h.owner.dispose();
  assert.deepEqual(h.revokedRuntimes, [h.runtimes[0]]);
  assert.equal(disposalCapture, null);
  assert.ok(disposalFailure instanceof Error);
  assertRetired(binding);
});

test("MAB account same-account refresh and restart preserve the captured lifetime", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const binding = requireBinding(h.owner);
  h.owner.observe(loggedIn("A", "changed@example.test"));
  binding.assertCurrent();
  assert.equal(requireBinding(h.owner).runtimeIdentity, binding.runtimeIdentity);
  await h.owner.whenIdle();
  h.owner.requestRendererPort(() => {});
  await h.owner.restart();
  binding.assertCurrent();
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.runtimes[0]!.restartCalls, 1);
  assert.deepEqual(h.authorizations, ["A"]);
  await h.owner.dispose();
});

test("MAB account departure retires before callbacks and scopes port revocation after preparation", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const first = h.runtimes[0]!;
  const binding = requireBinding(h.owner);
  const preparation = Promise.withResolvers<void>();
  const preparing = Promise.withResolvers<void>();
  const exit = Promise.withResolvers<void>();
  first.exit = exit.promise;
  let callbackCapture: CoordinatorAccountBinding | null | undefined;
  let callbackFailure: unknown;
  first.onRevoke = () => {
    callbackCapture = h.owner.captureCurrentBinding();
    callbackFailure = bindingFailure(binding);
  };
  h.hooks.prepare = () => {
    preparing.resolve();
    return preparation.promise;
  };
  h.owner.observe(loggedIn("B"));
  assertRetired(binding);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(callbackCapture, null);
  assert.ok(callbackFailure instanceof Error);
  await preparing.promise;
  assert.equal(first.disposeCalls, 0);
  assert.deepEqual(h.revokedRuntimes, []);
  assert.equal(h.events.includes("revoke-main"), false);
  preparation.resolve();
  await first.disposeStarted.promise;
  assert.equal(h.runtimes.length, 1);
  assert.deepEqual(h.revokedRuntimes, [first]);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.ok(h.events.indexOf("prepared") < h.events.indexOf("revoke-main"));
  assert.ok(h.events.indexOf("revoke-main") < h.events.indexOf("dispose:1"));
  h.events.push("exit:1");
  exit.resolve();
  await h.owner.whenIdle();
  assert.equal(requireBinding(h.owner).runtimeIdentity, h.runtimes[1]);
  assert.ok(h.events.indexOf("exit:1") < h.events.indexOf("authorize:B"));
  assertRetired(binding);
  await h.owner.dispose();
  assert.deepEqual(h.revokedRuntimes, [first, h.runtimes[1]]);
});

test("MAB account logout retires immediately and settles inactive after preparation", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const binding = requireBinding(h.owner);
  const preparation = Promise.withResolvers<void>();
  const preparing = Promise.withResolvers<void>();
  h.hooks.prepare = () => {
    preparing.resolve();
    return preparation.promise;
  };
  h.owner.observe(loggedOut);
  assertRetired(binding);
  assert.equal(h.owner.captureCurrentBinding(), null);
  await preparing.promise;
  assert.equal(h.events.includes("revoke-main"), false);
  assert.equal(h.runtimes[0]!.disposeCalls, 0);
  preparation.resolve();
  assert.equal((await h.owner.whenIdle()).kind, "logged-out");
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 1);
  assertRetired(binding);
  await h.owner.dispose();
});

test("MAB account queued A to B to A never revives the original binding", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const original = requireBinding(h.owner);
  h.owner.observe(loggedIn("B"));
  assertRetired(original);
  h.owner.observe(loggedIn("A"));
  assertRetired(original);
  assert.equal(h.owner.captureCurrentBinding(), null);
  await h.owner.whenIdle();
  const replacement = requireBinding(h.owner);
  assert.notEqual(replacement.runtimeIdentity, original.runtimeIdentity);
  assert.equal(h.runtimes.length, 2);
  assert.equal(h.runtimes[0]!.disposeCalls, 1);
  assert.equal(h.statuses.at(-1)?.authId, "A");
  assertRetired(original);
  await h.owner.dispose();
});

test("MAB account completed A to B to A creates distinct permanent lifetimes", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const firstA = requireBinding(h.owner);
  h.owner.observe(loggedIn("B"));
  await h.owner.whenIdle();
  const bindingB = requireBinding(h.owner);
  assertRetired(firstA);
  h.owner.observe(loggedIn("A"));
  await h.owner.whenIdle();
  const nextA = requireBinding(h.owner);
  assert.equal(h.runtimes.length, 3);
  assert.notEqual(nextA.runtimeIdentity, firstA.runtimeIdentity);
  assert.notEqual(nextA.runtimeIdentity, bindingB.runtimeIdentity);
  assertRetired(firstA);
  assertRetired(bindingB);
  await h.owner.dispose();
});

test("MAB account reentrant observation preserves the latest claim after renderer revocation", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const firstA = requireBinding(h.owner);
  let capturedDuringRevoke: CoordinatorAccountBinding | null | undefined;
  let failureDuringRevoke: unknown;
  h.runtimes[0]!.onRevoke = () => {
    capturedDuringRevoke = h.owner.captureCurrentBinding();
    failureDuringRevoke = bindingFailure(firstA);
    h.owner.observe(loggedIn("A"));
  };
  h.owner.observe(loggedIn("B"));
  assert.equal(capturedDuringRevoke, null);
  assert.ok(failureDuringRevoke instanceof Error);
  await h.owner.whenIdle();
  assert.equal(h.statuses.at(-1)?.authId, "A");
  assert.equal(h.runtimes.length, 2);
  assert.notEqual(requireBinding(h.owner).runtimeIdentity, firstA.runtimeIdentity);
  assertRetired(firstA);
  await h.owner.dispose();
});

test("MAB account logged-out and refused states cannot furnish a capture", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedOut);
  assert.equal(h.owner.captureCurrentBinding(), null);
  h.hooks.authorize = async () => false;
  h.owner.observe(loggedIn("A"));
  assert.equal((await h.owner.whenIdle()).kind, "logged-out");
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 0);
  assert.equal(h.events.includes("revoke-refused"), true);
  await h.owner.dispose();
  h.owner.observe(loggedIn("B"));
  await h.owner.start(loggedIn("B"));
  assert.deepEqual(h.authorizations, ["A"]);
  assert.equal(h.owner.captureCurrentBinding(), null);
});

test("MAB account reentrant start during construction cannot launch a second runtime", options, async () => {
  const h = createHarness();
  const constructionCaptures: Array<CoordinatorAccountBinding | null> = [];
  const deliveredPorts: unknown[] = [];
  let nestedStart: Promise<void> | undefined;
  h.owner.requestRendererPort((port) => deliveredPorts.push(port));
  h.hooks.onCreate = () => {
    constructionCaptures.push(h.owner.captureCurrentBinding());
    nestedStart = h.owner.start(loggedIn("B"));
  };
  await h.owner.start(loggedIn("A"));
  await nestedStart;
  await h.owner.whenIdle();
  assert.deepEqual(constructionCaptures, [null]);
  assert.deepEqual(h.authorizations, ["A"]);
  assert.equal(h.runtimes.length, 1);
  assert.deepEqual(deliveredPorts, [h.runtimes[0]!.port]);
  assert.equal(requireBinding(h.owner).runtimeIdentity, h.runtimes[0]);
  await h.owner.dispose();
});

test("MAB account same-account construction refresh preserves the pending lifetime", options, async () => {
  const h = createHarness();
  const constructionCaptures: Array<CoordinatorAccountBinding | null> = [];
  h.hooks.onCreate = (runtime) => {
    constructionCaptures.push(h.owner.captureCurrentBinding());
    if (runtime.index === 1) h.owner.observe(loggedIn("A", "refreshed@example.test"));
  };
  await h.owner.start(loggedIn("A"));
  await h.owner.whenIdle();
  assert.deepEqual(constructionCaptures, [null]);
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.runtimes[0]!.disposeCalls, 0);
  assert.equal(requireBinding(h.owner).runtimeIdentity, h.runtimes[0]);
  assert.equal(h.statuses.at(-1)?.email, "refreshed@example.test");
  await h.owner.dispose();
});

test("MAB account stale construction waits for exit without invoking departure port revoke", options, async () => {
  const h = createHarness();
  const created = Promise.withResolvers<ControlledRuntime>();
  const exit = Promise.withResolvers<void>();
  const deliveredPorts: unknown[] = [];
  h.owner.requestRendererPort((port) => deliveredPorts.push(port));
  h.hooks.onCreate = (runtime) => {
    if (runtime.index !== 1) return;
    runtime.exit = exit.promise;
    created.resolve(runtime);
    h.owner.observe(loggedIn("B"));
  };
  const startup = h.owner.start(loggedIn("A"));
  const stale = await created.promise;
  await stale.disposeStarted.promise;
  await turn();
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.events.includes("revoke-main"), false);
  assert.deepEqual(deliveredPorts, []);
  exit.resolve();
  await startup;
  await h.owner.whenIdle();
  assert.equal(stale.disposeCalls, 1);
  assert.equal(stale.rendererRequests, 0);
  assert.equal(h.runtimes.length, 2);
  assert.equal(requireBinding(h.owner).runtimeIdentity, h.runtimes[1]);
  assert.deepEqual(deliveredPorts, [h.runtimes[1]!.port]);
  assert.equal(h.events.includes("revoke-main"), false);
  assert.deepEqual(h.revokedRuntimes, []);
  await h.owner.dispose();
});

test("MAB account construction A to B to A cannot publish the retired candidate", options, async () => {
  const h = createHarness();
  const created = Promise.withResolvers<ControlledRuntime>();
  const exit = Promise.withResolvers<void>();
  h.owner.requestRendererPort(() => {});
  h.hooks.onCreate = (runtime) => {
    if (runtime.index !== 1) return;
    runtime.exit = exit.promise;
    created.resolve(runtime);
    h.owner.observe(loggedIn("B"));
    h.owner.observe(loggedIn("A"));
  };
  const startup = h.owner.start(loggedIn("A"));
  const stale = await created.promise;
  await stale.disposeStarted.promise;
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(stale.rendererRequests, 0);
  exit.resolve();
  await startup;
  await h.owner.whenIdle();
  assert.equal(h.runtimes.length, 2);
  assert.equal(requireBinding(h.owner).runtimeIdentity, h.runtimes[1]);
  assert.equal(h.statuses.at(-1)?.authId, "A");
  assert.equal(stale.disposeCalls, 1);
  await h.owner.dispose();
});

test("MAB account reentrant construction disposal waits for the owned exit and stays terminal", options, async () => {
  const h = createHarness();
  const created = Promise.withResolvers<ControlledRuntime>();
  const exit = Promise.withResolvers<void>();
  let disposal: Promise<void> | undefined;
  let disposalFinished = false;
  h.owner.requestRendererPort(() => {});
  h.hooks.onCreate = (runtime) => {
    runtime.exit = exit.promise;
    created.resolve(runtime);
    disposal = h.owner.dispose();
    void disposal.then(() => { disposalFinished = true; });
    h.owner.observe(loggedIn("B"));
  };
  const startup = h.owner.start(loggedIn("A"));
  const stale = await created.promise;
  await stale.disposeStarted.promise;
  assert.ok(disposal);
  assert.equal(h.owner.dispose(), disposal);
  await turn();
  assert.equal(disposalFinished, false);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(stale.rendererRequests, 0);
  assert.equal(h.events.includes("revoke-main"), false);
  exit.resolve();
  await Promise.all([startup, disposal]);
  await h.owner.whenIdle();
  assert.equal(h.runtimes.length, 1);
  assert.equal(stale.disposeCalls, 1);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.deepEqual(h.authorizations, ["A"]);
});

test("MAB account synchronous stale disposal failure stays blocked even if reporting throws", options, async () => {
  const h = createHarness();
  h.hooks.onProblem = () => { throw new Error("controlled reporter failure"); };
  h.hooks.onCreate = (runtime) => {
    runtime.disposeError = new Error("controlled synchronous disposal failure");
    h.owner.observe(loggedIn("B"));
  };
  await h.owner.start(loggedIn("A"));
  assert.equal((await h.owner.whenIdle()).authId, "B");
  assert.equal(h.owner.captureCurrentBinding(), null);
  h.owner.observe(loggedIn("C"));
  await h.owner.whenIdle();
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.runtimes[0]!.disposeCalls, 1);
  assert.deepEqual(h.authorizations, ["A"]);
  assert.equal(h.events.includes("revoke-main"), false);
  assert.ok(h.problems.some((problem) => problem.includes("exit could not be confirmed")));
  await h.owner.dispose();
});

test("MAB account rejected stale exit stays blocked across later claims", options, async () => {
  const h = createHarness();
  h.hooks.onCreate = (runtime) => {
    runtime.exit = Promise.reject(new Error("controlled exit rejection"));
    h.owner.observe(loggedIn("B"));
  };
  await h.owner.start(loggedIn("A"));
  await h.owner.whenIdle();
  h.owner.observe(loggedIn("A"));
  await h.owner.whenIdle();
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.runtimes[0]!.disposeCalls, 1);
  assert.deepEqual(h.authorizations, ["A"]);
  assert.ok(h.problems.some((problem) => problem.includes("controlled exit rejection")));
  await h.owner.dispose();
});

test("MAB account timed-out stale exit recovers only the latest observed account", options, async () => {
  const h = createHarness();
  const created = Promise.withResolvers<ControlledRuntime>();
  const recovered = Promise.withResolvers<ControlledRuntime>();
  const exit = Promise.withResolvers<void>();
  h.hooks.onCreate = (runtime) => {
    if (runtime.index === 1) {
      runtime.exit = exit.promise;
      created.resolve(runtime);
      h.owner.observe(loggedIn("B"));
    } else recovered.resolve(runtime);
  };
  const startup = h.owner.start(loggedIn("A"));
  const stale = await created.promise;
  await stale.disposeStarted.promise;
  assert.ok(h.deadlines[0]);
  h.deadlines[0].expire();
  await startup;
  assert.equal((await h.owner.whenIdle()).authId, "B");
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.deepEqual(h.timeouts, [25]);
  assert.equal(h.runtimes.length, 1);
  h.owner.observe(loggedIn("C"));
  h.owner.observe(loggedIn("D"));
  await h.owner.whenIdle();
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.events.includes("revoke-main"), false);
  exit.resolve();
  const replacement = await recovered.promise;
  await h.owner.whenIdle();
  assert.equal(requireBinding(h.owner).runtimeIdentity, replacement);
  assert.deepEqual(h.authorizations, ["A", "D"]);
  assert.equal(h.runtimes.length, 2);
  assert.equal(stale.disposeCalls, 1);
  await h.owner.dispose();
});

test("MAB account normal exit timeout retains cleanup ordering and recovers the latest claim", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const binding = requireBinding(h.owner);
  const first = h.runtimes[0]!;
  const exit = Promise.withResolvers<void>();
  const recovered = Promise.withResolvers<ControlledRuntime>();
  first.exit = exit.promise;
  h.hooks.onCreate = (runtime) => recovered.resolve(runtime);
  h.owner.observe(loggedIn("B"));
  await first.disposeStarted.promise;
  assert.ok(h.deadlines[0]);
  h.deadlines[0].expire();
  await h.owner.whenIdle();
  assertRetired(binding);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.ok(h.events.indexOf("prepared") < h.events.indexOf("revoke-main"));
  assert.ok(h.events.indexOf("revoke-main") < h.events.indexOf("dispose:1"));
  h.owner.observe(loggedIn("C"));
  await h.owner.whenIdle();
  assert.equal(h.runtimes.length, 1);
  exit.resolve();
  const replacement = await recovered.promise;
  await h.owner.whenIdle();
  assert.equal(requireBinding(h.owner).runtimeIdentity, replacement);
  assert.deepEqual(h.authorizations, ["A", "C"]);
  assertRetired(binding);
  await h.owner.dispose();
});

test("MAB account disposal from shared-port revocation cannot restore an active state", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const binding = requireBinding(h.owner);
  let disposal: Promise<void> | undefined;
  h.hooks.onRevokeMain = () => { disposal = h.owner.dispose(); };
  h.owner.observe(loggedIn("B"));
  await h.owner.whenIdle();
  assert.ok(disposal);
  await disposal;
  assertRetired(binding);
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 1);
  assert.equal(h.runtimes[0]!.disposeCalls, 1);
  assert.deepEqual(h.authorizations, ["A"]);
});

test("MAB account timed-out retirement cannot recover after disposal", options, async () => {
  const h = createHarness();
  await h.owner.start(loggedIn("A"));
  const first = h.runtimes[0]!;
  const exit = Promise.withResolvers<void>();
  first.exit = exit.promise;
  h.owner.observe(loggedIn("B"));
  await first.disposeStarted.promise;
  assert.ok(h.deadlines[0]);
  h.deadlines[0].expire();
  await h.owner.whenIdle();
  await h.owner.dispose();
  h.owner.observe(loggedIn("C"));
  exit.resolve();
  await turn();
  await h.owner.whenIdle();
  assert.equal(h.owner.captureCurrentBinding(), null);
  assert.equal(h.runtimes.length, 1);
  assert.deepEqual(h.authorizations, ["A"]);
});

test("MAB account disposal during authorization prevents later account effects", options, async () => {
  for (const authorized of [true, false]) {
    const h = createHarness();
    const authorization = Promise.withResolvers<boolean>();
    const requested = Promise.withResolvers<void>();
    h.hooks.authorize = () => {
      requested.resolve();
      return authorization.promise;
    };
    const startup = h.owner.start(loggedIn("A"));
    await requested.promise;
    const disposal = h.owner.dispose();
    assert.equal(h.owner.captureCurrentBinding(), null);
    authorization.resolve(authorized);
    await Promise.all([startup, disposal]);
    h.owner.observe(loggedIn("B"));
    await h.owner.whenIdle();
    assert.equal(h.runtimes.length, 0);
    assert.deepEqual(h.authorizations, ["A"]);
    assert.equal(h.events.includes("revoke-refused"), false);
    assert.equal(h.owner.captureCurrentBinding(), null);
  }
});

// The origin integration controls the Electron child ABI and manager resource
// ports. Production owns account binding, runtime/launcher, control dispatch,
// source creation, Attempt capture and generated result classification. Native
// MessageChannels retain the real structured-clone boundary.
class OriginNativePort implements CoordinatorMessagePort {
  constructor(private readonly port: MessagePort) {}
  postMessage(value: unknown): void { this.port.postMessage(value); }
  on(event: "message", listener: (event: { readonly data: unknown }) => void): void;
  on(event: "close", listener: () => void): void;
  on(event: "message" | "close", listener: ((event: { readonly data: unknown }) => void) | (() => void)): void {
    if (event === "message") this.port.on(event, (data: unknown) => (listener as (event: { data: unknown }) => void)({ data }));
    else this.port.on(event, listener);
  }
  start(): void { this.port.start(); }
  close(): void { this.port.close(); }
}

class OriginChild implements CoordinatorChildProcess {
  readonly ready = Promise.withResolvers<void>();
  readonly killed = Promise.withResolvers<void>();
  private readonly exits: Array<(code: number | null) => void> = [];
  private readonly replies = new Map<string, { resolve(value: CoordinatorReplyOutcome): void; reject(error: unknown): void }>();
  private control!: CoordinatorMessagePort;
  private sequence = 0;
  private exited = false;

  postMessage(_value: unknown, ports: CoordinatorMessagePort[]): void {
    assert.equal(ports.length, 3);
    this.control = ports[0]!;
    this.control.on("message", ({ data }) => {
      const frame = data as CoordinatorFrame;
      if (frame.kind === "lifecycle" && frame.phase === "ready") this.ready.resolve();
      if (frame.kind === "reply") {
        const waiter = this.replies.get(frame.requestId);
        this.replies.delete(frame.requestId);
        waiter?.resolve(frame.outcome);
      }
    });
    this.control.on("close", () => {
      for (const waiter of this.replies.values()) waiter.reject(new Error("original control reply is unavailable"));
      this.replies.clear();
    });
    this.control.start();
    this.control.postMessage({ kind: "lifecycle", phase: "hello", protocolVersion: 1 });
    const main = ports[2]!;
    main.on("message", ({ data }) => {
      const frame = data as CoordinatorFrame;
      if (frame.kind === "lifecycle" && frame.phase === "hello") {
        main.postMessage({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
      }
    });
    main.start();
  }
  on(_event: "exit", listener: (code: number | null) => void): void { this.exits.push(listener); }
  kill(): void { this.killed.resolve(); }
  exit(): void {
    if (this.exited) return;
    this.exited = true;
    for (const listener of this.exits) listener(0);
  }
  request(method: string, args: unknown): Promise<CoordinatorReplyOutcome> {
    const requestId = `origin-${++this.sequence}`;
    const reply = new Promise<CoordinatorReplyOutcome>((resolve, reject) => this.replies.set(requestId, { resolve, reject }));
    this.control.postMessage({ kind: "request", requestId, method, args });
    return reply;
  }
  async call(method: string, args: unknown = {}): Promise<unknown> {
    const reply = await this.request(method, args);
    assert.equal(reply.status, "ok");
    if (reply.status !== "ok") throw new Error(reply.failure.message);
    return reply.value;
  }
}

function originBarrier() {
  const entered = Promise.withResolvers<void>();
  const released = Promise.withResolvers<void>();
  return { entered: entered.promise, release: () => released.resolve(), hold: async () => { entered.resolve(); await released.promise; } };
}

async function originProductionFixture(t: TestContext) {
  const root = await mkdtemp(join(tmpdir(), "mcp-origin-account-"));
  const artifact = join(root, "dist", "node-agent-coordinator", "main.cjs");
  await mkdir(join(root, "dist", "node-agent-coordinator"), { recursive: true });
  await writeFile(artifact, "// Controlled native child ABI; this file is never executed.\n");
  const channels: CoordinatorMessageChannel[] = [];
  const children: OriginChild[] = [];
  const sources: CoordinatorControlSourceContext[] = [];
  const executed: Array<{ manager: number; request: unknown }> = [];
  const problems: string[] = [];
  const calls = { factories: 0, requireMcp: 0, managerDisposals: 0 };
  const controls: { transition?: ReturnType<typeof originBarrier>; list?: ReturnType<typeof originBarrier> } = {};
  const heldLists = new Set<ReturnType<typeof originBarrier>>();
  const tool = { providerIdentifier: "origin-provider", name: "public-tool", toolName: "raw-tool", inputSchema: null };
  const unexpected = (): never => { throw new Error("unexpected external operation in origin fixture"); };
  const legs = createCoordinatorMainLegs({ onProblem: (detail) => problems.push(detail) });
  let status: Status = loggedOut;
  let coordinator!: ProductionCoordinatorService;
  let mcp!: ProductionMcpService;
  const context = {
    machineId: "origin-machine",
    native: { app: { isPackaged: false } },
    resources: { metadata: { version: "origin-contract" } },
    coordinatorLegs: legs,
    connectorEgress: createEgressConnectionObserver(),
    requireAccount: () => ({ getStatus: async () => status, subscribe: () => () => {} }),
    requireMcp: () => {
      calls.requireMcp += 1;
      return {
        ...mcp,
        listRoutedTools(source: CoordinatorControlSourceContext) {
          sources.push(source);
          return mcp.listRoutedTools(source);
        },
      };
    },
  } as unknown as ProductionServiceContext;
  mcp = await createProductionMcpOAuthAdapter({
    resolveRuntimeDeps: () => ({
      createManager: async () => {
        const manager = ++calls.factories;
        return {
          dispose() { calls.managerDisposals += 1; },
          setAuthCompletionObserver() {},
          async listRoutedTools() {
            const held = controls.list;
            controls.list = undefined;
            if (held != null) {
              heldLists.add(held);
              try { await held.hold(); } finally { heldLists.delete(held); }
            }
            return [tool];
          },
          async executeRoutedTool(request, assertDispatchCurrent) {
            assert.ok(assertDispatchCurrent);
            assertDispatchCurrent();
            executed.push({ manager, request });
            return new McpResult({ result: { case: "success", value: new McpSuccess() } });
          },
        } as DesktopMcpManagerFacade;
      },
      settingsStore: {}, pushBoxSecrets: async () => {},
      ensureCursorAuthService: async () => ({ getValidAccessToken: async () => "origin-token" }),
      getMachineId: () => "origin-machine",
      loadBoxMcpServers: async () => ({}), listBoxMcpServers: async () => [],
      listBoxMcpToolsRaw: async () => "00", executeBoxMcpToolRaw: async () => "00",
      reportConnectorAuth() {}, reportMcpDiscoveryFailed() {}, reportDiagnostic() {},
      cleanupLegacyAuth: async () => {}, sandRootDir: () => root,
      reportFailure() {}, broadcast() {}, refreshHostMcp() {},
    }),
    resolveDesktopDeps: unexpected,
  }).create(context);
  const adapter = createProductionCoordinatorAdapter<Status>({
    electronMainModuleUrl: pathToFileURL(join(root, "dist", "electron-main", "main.cjs")).href,
    utilityProcess: { fork(path) { assert.equal(path, artifact); const child = new OriginChild(); children.push(child); return child; } },
    MessageChannelMain: class implements CoordinatorMessageChannel {
      readonly port1: CoordinatorMessagePort;
      readonly port2: CoordinatorMessagePort;
      constructor() {
        const channel = new MessageChannel();
        this.port1 = new OriginNativePort(channel.port1);
        this.port2 = new OriginNativePort(channel.port2);
        channels.push(this);
      }
    },
    net: { isOnline: () => false }, powerMonitor: { on() {} },
    createWebAuthnPromptWindow: unexpected,
    localExecNative: { spawnLocalExecDaemon: unexpected, terminateProcess: unexpected, isProcessAlive: unexpected, readProcessIdentity: unexpected },
    createGatewayConnector: () => ({ connect: unexpected, issueLocalExecDaemonCredential: unexpected }),
    getDataDir: () => root,
    account: {
      authorizeAccount: async () => true,
      revokeRefusedAccount: async () => ({ kind: "completed", status: loggedOut }),
      async prepareAccountTransition() { await controls.transition?.hold(); await mcp.resetMcpManager(); },
      resetAccountState() {}, deliverStatus(next) { status = next; },
    },
    resync: {
      getMcpCustomInstructionsAccountScope: () => null,
      getMcpCustomInstructionsByServerId: () => ({}), getMcpDisabledToolsByServerId: () => ({}),
      setMcpCustomInstructionsByServerId() {}, setMcpDisabledToolsByServerId() {},
      detectTimeZone: () => "UTC", getUserTimeZoneOverride: () => null,
      getComputerUseModel: () => null, getAutoReviewInstructions: () => null,
      getLocalToolPermission: () => null, getWebauthnProxyEnabled: () => false,
      getFeatureFlagOverrides: () => ({}), pushBoxSecrets: async () => {},
      onHostSettingsTransportConnected() {}, onHostSettingsTransportDown() {},
    },
    events: { onAgentsEvent() {}, onAgentsRosterSeed() {}, onCoordinatorLaunched() {} },
    telemetry: {
      getUploader: () => null, isQuitting: () => false, isRebuildInFlight: () => false,
      reportHandoff() {}, recordSendStage() {}, recordGatewayCommandSpan() {}, reportProcessCrash() {},
      getRpcTraceWindowTraceparent: () => undefined, flushNow() {},
    },
    reportProblem: (area, detail) => problems.push(`${area}: ${detail}`),
    reportFailure: (area, leg) => problems.push(`${area}: ${leg}`),
  });
  coordinator = adapter.create(context);
  const runtime = coordinator.getAccountRuntime!() as CoordinatorAccountRuntime<Status>;
  t.after(async () => {
    controls.transition?.release(); controls.list?.release();
    for (const held of heldLists) held.release();
    const disposal = Promise.resolve(coordinator.dispose());
    for (const child of children) child.exit();
    try { await disposal; await mcp.dispose(); }
    finally {
      legs.dispose();
      for (const channel of channels) { channel.port1.close(); channel.port2.close(); }
      await rm(root, { recursive: true, force: true });
    }
  });
  return {
    runtime, mcp, children, sources, executed, calls, controls, tool, problems,
    async start() {
      await coordinator.start(loggedIn("origin-A"));
      await children[0]!.ready.promise;
      return children[0]!;
    },
  };
}

function requireOriginRow(value: unknown) {
  assert.ok(Array.isArray(value));
  assert.equal(value.length, 1);
  const row = value[0] as Record<string, unknown>;
  const origin = parseMcpRoutedOrigin(row.origin);
  assert.ok(origin);
  return { ...row, origin };
}

function assertWireOriginRetired(value: unknown): void {
  assert.deepEqual(value, { result: { case: "rejected", value: { reason: MCP_ORIGIN_RETIRED } } });
}

test("OD-T2 production Coordinator refuses departed account and serialized origin replay before manager B", options, async (t) => {
  const f = await originProductionFixture(t);
  const childA = await f.start();
  const rowA = requireOriginRow(await childA.call("listRoutedMcpTools"));
  const requestA = { ...rowA, args: { retained: true }, toolCallId: "origin-call", agentId: "origin-agent" };
  const sourceA = f.sources[0]!;
  const preparation = originBarrier();
  f.controls.transition = preparation;
  const resolutions = f.calls.requireMcp;
  f.runtime.observe(loggedIn("origin-B"));
  assert.throws(sourceA.assertCurrent, McpOriginRetiredError);
  await preparation.entered;
  assertWireOriginRetired(await childA.call("executeRoutedMcpTool", requestA));
  assert.equal(f.calls.requireMcp, resolutions, "the old account cannot resolve a replacement MCP service");
  assert.equal(f.calls.factories, 1);
  assert.deepEqual(f.executed, []);
  preparation.release();
  await childA.killed.promise;
  childA.exit();
  await f.runtime.whenIdle();
  const childB = f.children[1]!;
  await childB.ready.promise;
  assertWireOriginRetired(await childB.call("executeRoutedMcpTool", JSON.parse(JSON.stringify(requestA))));
  assert.equal(f.calls.factories, 1, "serialized A origin cannot construct manager B on the new native source");
  assert.deepEqual(f.executed, []);
  const rowB = requireOriginRow(await childB.call("listRoutedMcpTools"));
  assert.notEqual(rowB.origin.coordinatorSourceId, rowA.origin.coordinatorSourceId);
  assert.notEqual(rowB.origin.mcpAttemptId, rowA.origin.mcpAttemptId);
  const requestB = { ...requestA, ...rowB };
  assert.deepEqual(await childB.call("executeRoutedMcpTool", requestB), { result: { case: "success", value: { content: [], isError: false } } });
  assert.deepEqual(f.executed, [{ manager: 2, request: requestB }]);
  assert.equal(f.calls.factories, 2);
  assert.throws(sourceA.assertCurrent, McpOriginRetiredError);
  assert.deepEqual(f.problems, []);
});

test("OD-T2 same-account child restart retires its source while retaining the current MCP Attempt", options, async (t) => {
  const f = await originProductionFixture(t);
  const childA = await f.start();
  const rowA = requireOriginRow(await childA.call("listRoutedMcpTools"));
  const sourceA = f.sources[0]!;
  const account = requireBinding(f.runtime);
  const requestA = { ...rowA, args: { retained: true }, toolCallId: "restart-call", agentId: "restart-agent" };
  const restart = f.runtime.restart();
  account.assertCurrent();
  assert.throws(sourceA.assertCurrent, McpOriginRetiredError);
  await childA.killed.promise;
  childA.exit();
  await restart;
  const childB = f.children[1]!;
  await childB.ready.promise;
  assert.equal(requireBinding(f.runtime).runtimeIdentity, account.runtimeIdentity);
  assertWireOriginRetired(await childB.call("executeRoutedMcpTool", structuredClone(requestA)));
  assert.equal(f.calls.factories, 1);
  assert.equal(f.calls.managerDisposals, 0);
  assert.deepEqual(f.executed, []);
  const rowB = requireOriginRow(await childB.call("listRoutedMcpTools"));
  assert.notEqual(rowB.origin.coordinatorSourceId, rowA.origin.coordinatorSourceId);
  assert.equal(rowB.origin.mcpAttemptId, rowA.origin.mcpAttemptId);
  const requestB = { ...requestA, ...rowB };
  await childB.call("executeRoutedMcpTool", requestB);
  assert.deepEqual(f.executed, [{ manager: 1, request: requestB }]);
  assert.throws(sourceA.assertCurrent, McpOriginRetiredError);
  assert.deepEqual(f.problems, []);
});

test("OD-T2/3 held production account catalog cannot publish its origin after departure", options, async (t) => {
  const f = await originProductionFixture(t);
  const childA = await f.start();
  const listing = originBarrier();
  const preparation = originBarrier();
  f.controls.list = listing;
  f.controls.transition = preparation;
  t.after(() => listing.release());
  const pending = childA.request("listRoutedMcpTools", {});
  await listing.entered;
  f.runtime.observe(loggedIn("origin-B"));
  await preparation.entered;
  listing.release();
  assert.deepEqual(await pending, { status: "failed", failure: { code: "main-execution-failure", message: MCP_ORIGIN_RETIRED } });
  assert.equal(f.calls.factories, 1);
  assert.deepEqual(f.executed, []);
  preparation.release();
  await childA.killed.promise;
  childA.exit();
  await f.runtime.whenIdle();
  const childB = f.children[1]!;
  await childB.ready.promise;
  const rowB = requireOriginRow(await childB.call("listRoutedMcpTools"));
  assert.equal(rowB.origin.coordinatorSourceId, f.sources.at(-1)!.sourceId);
  assert.equal(f.calls.factories, 2);
  assert.deepEqual(f.problems, []);
});
