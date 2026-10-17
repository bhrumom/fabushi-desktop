import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import test from "node:test";
const require = createRequire(new URL("../desktop/package.json", import.meta.url));
const ts = require("typescript");
const transpile = (source, fileName) => "data:text/javascript;base64," + Buffer.from(ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 }, fileName }).outputText).toString("base64");
const modelSource = await readFile(new URL("../frontend/src/recovered/features/onboarding/signed-in/model.ts", import.meta.url), "utf8");
const modelUrl = "data:text/javascript;base64," + Buffer.from(ts.transpileModule(modelSource, { compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 }, fileName: "model.ts" }).outputText).toString("base64");
const model = await import(modelUrl);
const readinessSource = (await readFile(new URL("../frontend/src/recovered/features/onboarding/signed-in/computer-readiness.ts", import.meta.url), "utf8")).replace('from "./model";', `from "${modelUrl}";`);
const readiness = await import(transpile(readinessSource, "computer-readiness.ts"));
const viewSource = await readFile(new URL("../frontend/src/recovered/features/onboarding/signed-in/view.tsx", import.meta.url), "utf8");
class FakeClock {
  nextId = 1; timers = new Map();
  setTimeout(callback, delay) { const id = this.nextId++; this.timers.set(id, { callback, delay }); return id; }
  clearTimeout(id) { this.timers.delete(id); }
  fireDelay(delay) { const entry = [...this.timers.entries()].find(([, timer]) => timer.delay === delay); assert.ok(entry, `missing ${delay}ms timer`); this.timers.delete(entry[0]); entry[1].callback(); }
}
test("readiness success is explicit", async () => {
  const source = readiness.createOnboardingComputerReadiness(async () => "ignored");
  const unsubscribe = source.subscribe(() => {});
  assert.equal(await source.waitForComputer(), "ready");
  unsubscribe(); source.dispose();
});
test("60-second timeout preserves fail-open behavior", async () => {
  const clock = new FakeClock();
  const source = readiness.createOnboardingComputerReadiness(() => new Promise(() => {}), { clock });
  const unsubscribe = source.subscribe(() => {});
  const waiting = source.waitForComputer(); clock.fireDelay(model.ONBOARDING_BOX_WAIT_TIMEOUT_MS);
  const result = await waiting;
  assert.equal(result, "timeout"); assert.equal(model.shouldContinueAfterComputerWait(result), true);
  unsubscribe(); source.dispose();
});
test("readiness disposal cancels", async () => {
  const source = readiness.createOnboardingComputerReadiness(() => new Promise(() => {}));
  const waiting = source.waitForComputer(); source.dispose();
  assert.equal(await waiting, "cancelled"); assert.equal(model.shouldContinueAfterComputerWait("cancelled"), false);
});
test("handoff dwell distinguishes completion", async () => {
  const clock = new FakeClock(), dwell = model.createOnboardingHandOffDwell(clock);
  const waiting = dwell.wait(); clock.fireDelay(model.ONBOARDING_HAND_OFF_DWELL_MS);
  assert.equal(await waiting, "completed"); dwell.dispose();
});
test("handoff cancellation and disposal never report success", async () => {
  const first = model.createOnboardingHandOffDwell(new FakeClock()), cancelled = first.wait();
  first.cancel(); assert.equal(await cancelled, "cancelled"); first.dispose();
  const second = model.createOnboardingHandOffDwell(new FakeClock()), disposed = second.wait();
  second.dispose(); assert.equal(await disposed, "cancelled"); assert.equal(await second.wait(), "cancelled");
});
test("only current generation can complete", async () => {
  const fence = model.createOnboardingRequestGenerationFence(), old = fence.begin();
  let resolveDwell, calls = 0;
  const pending = model.completeOnboardingRequestAfterDwell(() => new Promise(resolve => { resolveDwell = resolve; }), () => fence.isCurrent(old), () => { calls++; });
  fence.invalidate(); resolveDwell("completed");
  assert.equal(await pending, false); assert.equal(calls, 0);
  const current = fence.begin();
  assert.equal(await model.completeOnboardingRequestAfterDwell(async () => "completed", () => fence.isCurrent(current), () => { calls++; }), true);
  assert.equal(calls, 1);
});
test("A to B to A does not revive old generation", async () => {
  const fence = model.createOnboardingRequestGenerationFence(), a1 = fence.begin();
  fence.invalidate(); const b = fence.begin(); fence.invalidate(); const a2 = fence.begin();
  assert.notEqual(a1, a2); assert.equal(fence.isCurrent(a1), false); assert.equal(fence.isCurrent(b), false); assert.equal(fence.isCurrent(a2), true);
  let calls = 0;
  assert.equal(await model.completeOnboardingRequestAfterDwell(async () => "completed", () => fence.isCurrent(a1), () => { calls++; }), false);
  assert.equal(calls, 0);
});
test("production view binds async onboarding to generations and explicit results", () => {
  assert.match(viewSource, /createOnboardingRequestGenerationFence/);
  assert.match(viewSource, /shouldContinueAfterComputerWait\(readinessResult\)/);
  assert.match(viewSource, /completeOnboardingRequestAfterDwell/);
  assert.match(viewSource, /if \(!isRequestCurrent\(\)\) return;/);
});
