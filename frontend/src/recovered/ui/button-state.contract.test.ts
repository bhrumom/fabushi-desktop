import assert from "node:assert/strict";
import test from "node:test";

import { resolveButtonInteractionState } from "./button-state.ts";

test("pending projects as aria-busy and disabled without changing permanent applicability", () => {
  assert.deepEqual(resolveButtonInteractionState({ disabled: false, pending: true }), {
    ariaBusy: true,
    disabled: true,
    pending: true,
  });
  assert.deepEqual(resolveButtonInteractionState({ disabled: true, pending: false }), {
    ariaBusy: undefined,
    disabled: true,
    pending: false,
  });
});

test("pending restoration returns an otherwise applicable button to interactive state", () => {
  const before = resolveButtonInteractionState({ disabled: false, pending: false });
  const during = resolveButtonInteractionState({ disabled: false, pending: true });
  const after = resolveButtonInteractionState({ disabled: false, pending: false });
  assert.equal(before.disabled, false);
  assert.equal(during.disabled, true);
  assert.equal(during.ariaBusy, true);
  assert.deepEqual(after, before);
});
