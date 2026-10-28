import assert from "node:assert/strict";
import test from "node:test";

import { commandPaletteRoutinesFromCoordinator } from "./command-palette-provider.ts";

test("routine projection deduplicates the same automation within one agent", () => {
  const routines = commandPaletteRoutinesFromCoordinator([
    { agentId: "agent-1", automation: { id: "routine-1", name: "Daily", triggerDescription: "every day", createdAt: 1, lastRunAt: null } },
    { agentId: "agent-1", automation: { id: "routine-1", name: "Daily (stale)", triggerDescription: "every day", createdAt: 1, lastRunAt: null } },
    { agentId: "agent-2", automation: { id: "routine-1", name: "Other", triggerDescription: "every day", createdAt: 1, lastRunAt: null } },
  ]);
  assert.deepEqual(routines.map((routine) => `${routine.agentId}:${routine.automation.id}`), ["agent-1:routine-1", "agent-2:routine-1"]);
});
