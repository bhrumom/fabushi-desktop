import assert from "node:assert/strict";
import test from "node:test";

import {
  createCoordinatorRuntime,
  type CoordinatorRuntimeDependencies,
} from "./coordinator-runtime.js";
import type {
  CoordinatorLaunchHandle,
  CoordinatorMessagePort,
} from "./coordinator-launcher.js";

class FakePort implements CoordinatorMessagePort {
  constructor(readonly name: string) {}
  postMessage(_value: unknown): void {}
  on(
    _event: "message" | "close",
    _listener: ((event: { readonly data: unknown }) => void) | (() => void),
  ): void {}
  start(): void {}
  close(): void {}
}

type FakeHandle = CoordinatorLaunchHandle & {
  disposeCount: number;
  exit(code?: number | null): void;
};

function fakeHandle(index: number): FakeHandle {
  let resolveExit!: (value: { readonly code: number | null }) => void;
  const processExited = new Promise<{ readonly code: number | null }>((resolve) => {
    resolveExit = resolve;
  });
  return {
    rendererDataPort: new FakePort(`renderer-${index}`),
    mainDataPort: new FakePort(`main-${index}`),
    controlSettled: Promise.resolve(),
    processExited,
    disposeCount: 0,
    dispose() {
      this.disposeCount += 1;
    },
    exit(code = 0) {
      resolveExit({ code });
    },
  };
}

test("renderer port replacement and restart retire the old generation before launching the next", async () => {
  const launches: FakeHandle[] = [];
  let relaunchSchedules = 0;
  const dependencies: CoordinatorRuntimeDependencies = {
    fork() {
      throw new Error("test launch override must own coordinator creation");
    },
    artifactPath: "/runtime/coordinator",
    createChannel() {
      throw new Error("test launch override must own coordinator channels");
    },
    executors: {},
    onEvent: {
      "transport-connected": () => {},
      "transport-down": () => {},
    },
    onProblem: () => {},
    processConfig: {},
    monotonicNow: () => 0,
    onMainDataPort: () => {},
    onLifecycle: () => {},
    relaunchBackoff: {
      schedule() {
        relaunchSchedules += 1;
        return {
          elapsed: new Promise<void>(() => {}),
          dispose() {},
        };
      },
    },
    launch() {
      const handle = fakeHandle(launches.length + 1);
      launches.push(handle);
      return handle;
    },
  };

  const runtime = createCoordinatorRuntime(dependencies);
  assert.equal(launches.length, 1);

  let firstPort: unknown;
  runtime.requestRendererPort((port) => {
    firstPort = port;
  });
  assert.equal(firstPort, launches[0]?.rendererDataPort);

  let secondPort: unknown;
  runtime.requestRendererPort((port) => {
    secondPort = port;
  });
  assert.equal(launches.length, 1, "new generation must wait for old process exit");
  assert.equal(launches[0]?.disposeCount, 1);
  assert.equal(secondPort, undefined);

  launches[0]?.exit(0);
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(launches.length, 2);
  assert.equal(secondPort, launches[1]?.rendererDataPort);
  assert.equal(relaunchSchedules, 0, "intentional handoff must not schedule crash relaunch");

  const restart = runtime.restart();
  assert.equal(launches.length, 2, "explicit restart must also wait for old process exit");
  assert.equal(launches[1]?.disposeCount, 1);
  launches[1]?.exit(0);
  await restart;
  assert.equal(launches.length, 3);
  assert.equal(relaunchSchedules, 0);

  const disposal = runtime.dispose();
  launches[2]?.exit(0);
  await disposal;
});
