import assert from "node:assert/strict";
import test from "node:test";

import {
  COMMAND_PALETTE_MESSAGE_SEARCH_DEBOUNCE_MS,
  createCommandPaletteMessageProvider,
} from "./command-palette-message-provider.ts";

test("message search treats blank input as empty without invoking the coordinator", async () => {
  let calls = 0;
  const provider = createCommandPaletteMessageProvider({
    searchAgents: async () => {
      calls += 1;
      return [];
    },
  });
  provider.setAvailable(true);
  const snapshot = await provider.setQuery("  ");
  assert.equal(snapshot.status, "empty");
  assert.equal(calls, 0);
  provider.dispose();
});

test("message search debounces and forwards normalized non-empty queries", async () => {
  const queries: string[] = [];
  const provider = createCommandPaletteMessageProvider({
    searchAgents: async ({ query }) => {
      queries.push(query);
      return [{ agentId: "agent-1", entryId: "entry-1", role: "assistant", timestampMs: 1, snippet: "hello" }];
    },
  });
  provider.setAvailable(true);
  const startedAt = Date.now();
  const snapshot = await provider.setQuery("  hello  ");
  assert.equal(snapshot.status, "ready");
  assert.deepEqual(queries, ["hello"]);
  assert.ok(Date.now() - startedAt >= COMMAND_PALETTE_MESSAGE_SEARCH_DEBOUNCE_MS - 10);
  provider.dispose();
});

test("a cancelled message search cannot publish stale results", async () => {
  let resolveSearch: ((value: unknown) => void) | null = null;
  const provider = createCommandPaletteMessageProvider({
    searchAgents: async () => new Promise((resolve) => { resolveSearch = resolve; }),
  });
  provider.setAvailable(true);
  const pending = provider.setQuery("stale");
  provider.cancel();
  resolveSearch?.([{ agentId: "agent-1", entryId: "entry-1", role: "assistant", timestampMs: 1, snippet: "stale" }]);
  const result = await pending;
  assert.equal(result.status, "cancelled");
  assert.equal(provider.getSnapshot().status, "cancelled");
  provider.dispose();
});
