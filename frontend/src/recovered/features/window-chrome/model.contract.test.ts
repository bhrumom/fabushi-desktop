import assert from "node:assert/strict";
import test from "node:test";

import {
  WINDOW_CHROME_METRICS,
  applyRootShellTheme,
  applyRootShellZoomFactor,
  scaledWindowChromeDimension,
  setWindowChromeVariables,
  shouldRefreshRootShellOnFocus,
  windowChromeBlock,
} from "./model.ts";

interface FakeStyle {
  colorScheme: string;
  values: Map<string, string>;
  setProperty(name: string, value: string): void;
  removeProperty(name: string): void;
}

function fakeDocument(): { documentElement: { dataset: Record<string, string>; style: FakeStyle } } {
  const style: FakeStyle = {
    colorScheme: "",
    values: new Map(),
    setProperty(name, value) {
      this.values.set(name, value);
    },
    removeProperty(name) {
      this.values.delete(name);
    },
  };
  return { documentElement: { dataset: {}, style } };
}

test("window chrome dimensions preserve platform and zoom contracts", () => {
  assert.equal(
    scaledWindowChromeDimension(WINDOW_CHROME_METRICS.controlsInset),
    "calc(140px / var(--sand-zoom-factor, 1))",
  );
  assert.equal(windowChromeBlock("win32"), "calc(51px / var(--sand-zoom-factor, 1))");
  assert.equal(windowChromeBlock("linux"), "52px");
  assert.equal(windowChromeBlock("darwin"), "52px");
});

test("window chrome DOM variables install and clean up only when applicable", () => {
  const prior = (globalThis as Record<string, unknown>).document;
  const doc = fakeDocument();
  (globalThis as Record<string, unknown>).document = doc;
  try {
    assert.equal(setWindowChromeVariables("darwin", false), undefined);
    assert.equal(setWindowChromeVariables("win32", true), undefined);

    const dispose = setWindowChromeVariables("win32", false);
    assert.equal(
      doc.documentElement.style.values.get("--sand-window-controls-inset"),
      "calc(140px / var(--sand-zoom-factor, 1))",
    );
    assert.equal(
      doc.documentElement.style.values.get("--sand-window-controls-block"),
      "calc(51px / var(--sand-zoom-factor, 1))",
    );

    applyRootShellTheme("dark");
    assert.equal(doc.documentElement.dataset.theme, "cursor-dark");
    assert.equal(doc.documentElement.style.colorScheme, "dark");

    applyRootShellTheme("light");
    assert.equal(doc.documentElement.dataset.theme, "cursor-light");
    assert.equal(doc.documentElement.style.colorScheme, "light");

    applyRootShellZoomFactor(1.25);
    assert.equal(doc.documentElement.style.values.get("--sand-zoom-factor"), "1.25");
    applyRootShellZoomFactor(Number.NaN);
    assert.equal(doc.documentElement.style.values.get("--sand-zoom-factor"), "1");

    dispose?.();
    assert.equal(doc.documentElement.style.values.has("--sand-window-controls-inset"), false);
    assert.equal(doc.documentElement.style.values.has("--sand-window-controls-block"), false);
  } finally {
    if (prior === undefined) delete (globalThis as Record<string, unknown>).document;
    else (globalThis as Record<string, unknown>).document = prior;
  }
});

test("focus refresh only occurs for visible connected renderer", () => {
  assert.equal(shouldRefreshRootShellOnFocus("connected", "visible"), true);
  assert.equal(shouldRefreshRootShellOnFocus("connected", "hidden"), false);
  assert.equal(shouldRefreshRootShellOnFocus("connecting", "visible"), false);
  assert.equal(shouldRefreshRootShellOnFocus("down", "visible"), false);
  assert.equal(shouldRefreshRootShellOnFocus("browser", "visible"), false);
});
