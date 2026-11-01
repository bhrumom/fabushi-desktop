import assert from "node:assert/strict";
import test from "node:test";

import { createAppAlertController } from "./controller.ts";

const flush = async (): Promise<void> => {
  await Promise.resolve();
  await Promise.resolve();
};

test("app alert settles confirm, cancel, queue, and failure states deterministically", async () => {
  const controller = createAppAlertController();
  let changes = 0;
  const unsubscribe = controller.subscribe(() => {
    changes += 1;
  });

  const first = controller.alert({
    title: "Delete conversation",
    confirmLabel: "Delete",
    cancelLabel: "Cancel",
    perform: async () => undefined,
  });
  assert.equal(controller.getSnapshot()?.request.title, "Delete conversation");

  const rejectedWhileCancellable = await controller.alert({
    title: "Should not queue",
    confirmLabel: "OK",
    cancelLabel: "Cancel",
  });
  assert.equal(rejectedWhileCancellable, false);

  controller.confirm();
  await flush();
  assert.equal(await first, true);
  assert.equal(controller.getSnapshot(), null);

  const active = controller.alert({
    title: "First",
    confirmLabel: "Continue",
  });
  const queued = controller.alert({
    title: "Second",
    confirmLabel: "Continue",
  });
  const third = await controller.alert({
    title: "Third",
    confirmLabel: "Continue",
  });
  assert.equal(third, false);

  controller.confirm();
  assert.equal(await active, true);
  assert.equal(controller.getSnapshot()?.request.title, "Second");
  controller.cancel();
  assert.equal(await queued, false);

  const failed = controller.alert({
    title: "Retryable action",
    confirmLabel: "Retry",
    pendingLabel: "Working",
    perform: async () => "provider unavailable",
  });
  controller.confirm();
  assert.equal(controller.getSnapshot()?.isPerforming, true);
  await flush();
  assert.equal(controller.getSnapshot()?.isPerforming, false);
  assert.equal(controller.getSnapshot()?.failure, "provider unavailable");
  controller.cancel();
  assert.equal(await failed, false);

  const thrown = controller.alert({
    title: "Throwing action",
    confirmLabel: "Try",
    perform: async () => {
      throw new Error("network down");
    },
  });
  controller.confirm();
  await flush();
  assert.equal(controller.getSnapshot()?.failure, "network down");
  controller.reset();
  assert.equal(await thrown, false);

  assert.ok(changes >= 8);
  unsubscribe();
  controller.dispose();
});

test("dispose rejects active and queued work and prevents future alerts", async () => {
  const controller = createAppAlertController();
  const active = controller.alert({ title: "A", confirmLabel: "A" });
  const queued = controller.alert({ title: "B", confirmLabel: "B" });
  controller.dispose();
  assert.equal(await active, false);
  assert.equal(await queued, false);
  assert.equal(
    await controller.alert({ title: "C", confirmLabel: "C" }),
    false,
  );
});
