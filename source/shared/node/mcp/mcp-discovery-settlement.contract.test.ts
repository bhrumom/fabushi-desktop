import assert from "node:assert/strict";
import test from "node:test";

import {
  settleMcpDiscoveryFailure,
  settleMcpDiscoverySuccess,
} from "./mcp-discovery-settlement.js";

test("successful MCP discovery settlement carries resolution and emits no failure report", () => {
  const resolution = { tools: ["tool-a"], resolvedKey: "server-a" };
  const settled = settleMcpDiscoverySuccess(resolution);
  assert.deepEqual(settled, { resolution });
  assert.equal(Object.hasOwn(settled, "report"), false);
});

test("cold MCP discovery failure clears cache and preserves elapsed semantics", () => {
  const failure = new Error("deadline");
  failure.name = "TimeoutError";
  const settled = settleMcpDiscoveryFailure(failure, 37, undefined);
  assert.equal(settled.clearCache, true);
  assert.equal(settled.staleTools, undefined);
  assert.deepEqual(settled.report, {
    errorClass: "TimeoutError",
    elapsedMs: 37,
    servedStale: false,
  });
});

test("failed refresh preserves stale tools and marks servedStale", () => {
  const stale = [{ providerIdentifier: "server-a", name: "tool-a" }];
  const settled = settleMcpDiscoveryFailure("offline", 29, stale);
  assert.equal(settled.clearCache, false);
  assert.equal(settled.staleTools, stale);
  assert.deepEqual(settled.report, {
    errorClass: "string",
    elapsedMs: 29,
    servedStale: true,
  });
});

test("Error with empty name keeps frozen Error fallback", () => {
  const failure = new Error("boom");
  Object.defineProperty(failure, "name", { value: "", configurable: true });
  assert.equal(
    settleMcpDiscoveryFailure(failure, 0, undefined).report.errorClass,
    "Error",
  );
});
