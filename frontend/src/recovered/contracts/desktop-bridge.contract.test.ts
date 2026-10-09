import assert from "node:assert/strict";
import test from "node:test";

import {
  DESKTOP_BRIDGE_TOP_LEVEL_KEYS,
  hasDesktopBridge,
  requireDesktopBridge,
} from "./desktop-bridge.ts";

test("desktop bridge contract exposes the frozen top-level preload surface exactly once", () => {
  assert.equal(new Set(DESKTOP_BRIDGE_TOP_LEVEL_KEYS).size, DESKTOP_BRIDGE_TOP_LEVEL_KEYS.length);
  for (const key of [
    "mcp",
    "cursorAccount",
    "experiments",
    "foreverBox",
    "telemetry",
    "localToolPermission",
    "theme",
    "secrets",
    "agent",
    "update",
    "attachProdBox",
  ]) {
    assert.equal(DESKTOP_BRIDGE_TOP_LEVEL_KEYS.includes(key as never), true);
  }
  assert.equal(DESKTOP_BRIDGE_TOP_LEVEL_KEYS[0], "resolveAttachmentMedia");
  assert.equal(DESKTOP_BRIDGE_TOP_LEVEL_KEYS.at(-1), "attachProdBox");
});

test("desktop bridge guard is deliberately structural and fail-closed on missing ownership boundaries", () => {
  const minimal = {
    openExternal() {},
    getWindowState() {},
    mcp: {},
    cursorAccount: {},
    update: {},
  };
  assert.equal(hasDesktopBridge(minimal), true);
  assert.equal(requireDesktopBridge(minimal), minimal);

  for (const key of ["openExternal", "getWindowState", "mcp", "cursorAccount", "update"] as const) {
    const broken = { ...minimal } as Record<string, unknown>;
    delete broken[key];
    assert.equal(hasDesktopBridge(broken), false, `missing ${key}`);
  }

  assert.equal(hasDesktopBridge(null), false);
  assert.equal(hasDesktopBridge([]), false);
  assert.throws(() => requireDesktopBridge({}), /desktop preload bridge is unavailable/i);
});
