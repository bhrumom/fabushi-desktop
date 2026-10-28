import assert from "node:assert/strict";
import test from "node:test";

import { createCommandPaletteFileProvider } from "./command-palette-search-provider.ts";

test("file search treats blank input as an empty query without calling the coordinator", async () => {
  let calls = 0;
  const provider = createCommandPaletteFileProvider({
    searchMedia: async () => {
      calls += 1;
      return [];
    },
  });
  provider.setAvailable(true);
  const snapshot = await provider.setQuery("   ");
  assert.equal(snapshot.status, "empty");
  assert.equal(calls, 0);
  assert.equal(provider.getSnapshot().status, "empty");
  provider.dispose();
});

test("file search still forwards normalized non-empty queries", async () => {
  const queries: string[] = [];
  const provider = createCommandPaletteFileProvider({
    searchMedia: async ({ query }) => {
      queries.push(query);
      return [{ agentId: "agent-1", entryId: "entry-1", fileName: "notes.md", ext: "md", mime: "text/markdown", kind: "markdown", timestampMs: 1, width: null, height: null }];
    },
  });
  provider.setAvailable(true);
  const snapshot = await provider.setQuery("  notes  ");
  assert.equal(snapshot.status, "ready");
  assert.deepEqual(queries, ["notes"]);
  provider.dispose();
});
