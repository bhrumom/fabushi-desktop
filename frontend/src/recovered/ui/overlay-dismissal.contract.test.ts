import assert from "node:assert/strict";
import test from "node:test";

import { consumeOverlayBackdropPointer, scheduleDeferredOverlayDismiss, topmostOverlayLayerIndex } from "./overlay-dismissal.ts";

const flush = async (): Promise<void> => {
  await Promise.resolve();
  await Promise.resolve();
};

test("backdrop dismissal consumes the triggering pointer before closing", () => {
  const calls: string[] = [];
  consumeOverlayBackdropPointer({
    preventDefault: () => calls.push("preventDefault"),
    stopPropagation: () => calls.push("stopPropagation"),
    stopImmediatePropagation: () => calls.push("stopImmediatePropagation"),
  });
  assert.deepEqual(calls, ["preventDefault", "stopPropagation", "stopImmediatePropagation"]);
});

test("backdrop dismissal is deferred and can be cancelled during teardown", async () => {
  let closes = 0;
  const cancel = scheduleDeferredOverlayDismiss(() => { closes += 1; });
  assert.equal(closes, 0);
  cancel();
  await flush();
  assert.equal(closes, 0);

  scheduleDeferredOverlayDismiss(() => { closes += 1; });
  assert.equal(closes, 0);
  await flush();
  assert.equal(closes, 1);
});

test("topmost modal arbitration prefers higher z-index and the later layer on ties", () => {
  assert.equal(topmostOverlayLayerIndex([]), -1);
  assert.equal(topmostOverlayLayerIndex([3200]), 0);
  assert.equal(topmostOverlayLayerIndex([3200, 3200]), 1);
  assert.equal(topmostOverlayLayerIndex([4200, 3200, 4100]), 0);
  assert.equal(topmostOverlayLayerIndex([3200, 4200, 4200]), 2);
});
