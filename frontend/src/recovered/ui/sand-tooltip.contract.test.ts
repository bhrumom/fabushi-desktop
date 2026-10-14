import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const source = readFileSync(new URL("./sand-floating-primitives.tsx", import.meta.url), "utf8");

test("canonical tooltip owns optional transient dismissal policy without a second owner", () => {
  assert.match(source, /readonly autoDismissMs\?: number;/);
  assert.match(source, /readonly closeOnOutsidePress\?: boolean;/);
  assert.match(source, /readonly closeOnEscape\?: boolean;/);
  assert.match(source, /readonly returnFocus\?: boolean;/);
  assert.match(source, /const autoDismissTimerRef = useRef<ReturnType<typeof setTimeout> \| null>\(null\);/);
  assert.match(source, /if \(!open \|\| disabled \|\| autoDismissMs == null \|\| autoDismissMs <= 0\) return;/);
  assert.match(source, /setTimeout\(\(\) => setOpen\(false\), autoDismissMs\)/);
  assert.match(source, /closeOnOutsidePress=\{closeOnOutsidePress\}/);
  assert.match(source, /closeOnEscape=\{closeOnEscape\}/);
  assert.match(source, /returnFocus=\{returnFocus\}/);
});

test("canonical tooltip keeps existing passive hover defaults", () => {
  assert.match(source, /closeOnOutsidePress = false/);
  assert.match(source, /closeOnEscape = false/);
  assert.match(source, /returnFocus = false/);
  assert.match(source, /openDelay = 30/);
  assert.match(source, /closeDelay = 300/);
});
