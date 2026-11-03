import assert from "node:assert/strict";
import test from "node:test";

import {
  createCoordinatorAccountRuntime,
  type CoordinatorAccountBinding,
  type CoordinatorAccountRuntime,
  type CoordinatorAccountTransition,
  type CoordinatorAuthStatus,
} from "./coordinator-account-runtime.js";
import type { CoordinatorRuntime } from "./coordinator-runtime.js";

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
