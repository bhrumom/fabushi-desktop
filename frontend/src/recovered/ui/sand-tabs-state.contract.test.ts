import assert from "node:assert/strict";
import test from "node:test";
import { resolveSandTabNavigation, resolveSandTabReorderTarget } from "./sand-tabs-state.ts";
const items = [
  { id: "all" },
  { id: "pinned", reorderLocked: true },
  { id: "work" },
  { id: "later", disabled: true },
  { id: "other" },
] as const;
test("tab navigation wraps and skips disabled items in both orientations", () => {
  assert.equal(resolveSandTabNavigation(items, 2, "ArrowRight"), "other");
  assert.equal(resolveSandTabNavigation(items, 4, "ArrowRight"), "all");
  assert.equal(resolveSandTabNavigation(items, 0, "ArrowLeft"), "other");
  assert.equal(resolveSandTabNavigation(items, 2, "ArrowDown", "vertical"), "other");
  assert.equal(resolveSandTabNavigation(items, 2, "Home"), "all");
  assert.equal(resolveSandTabNavigation(items, 2, "End"), "other");
});
test("reorder target cannot cross a locked tab interval", () => {
  assert.equal(resolveSandTabReorderTarget(items, 0, 4), 0);
  assert.equal(resolveSandTabReorderTarget(items, 4, 0), 2);
  assert.equal(resolveSandTabReorderTarget(items, 2, 4), 4);
});
test("locked or disabled source tabs cannot initiate reorder and targets clamp", () => {
  assert.equal(resolveSandTabReorderTarget(items, 1, 4), 1);
  assert.equal(resolveSandTabReorderTarget(items, 3, 0), 3);
  assert.equal(resolveSandTabReorderTarget([{ id: "a" }, { id: "b" }], 0, 99), 1);
});
