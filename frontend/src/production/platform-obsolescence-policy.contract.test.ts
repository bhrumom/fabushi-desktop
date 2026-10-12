import assert from "node:assert/strict";
import test from "node:test";

import {
  createPlatformObsolescencePersistence,
  platformObsolescenceSnoozeDays,
  projectPlatformObsolescence,
} from "./platform-obsolescence-policy.ts";

test("platform obsolescence preserves accepted 7/30/90-day cutoff windows", () => {
  assert.equal(platformObsolescenceSnoozeDays("2026-10-10", "2026-10-11", "2026-10-10"), 7);
  assert.equal(platformObsolescenceSnoozeDays("2026-10-10", "2026-10-10", "2026-10-01"), 30);
  assert.equal(platformObsolescenceSnoozeDays("2026-10-10", "2026-11-20", "2026-10-11"), 90);
});

test("dismissal before cutoff gets only the minimal post-cutoff grace", () => {
  assert.equal(projectPlatformObsolescence({
    cutoffDate: "2026-10-10",
    today: "2026-10-11",
    lastDismissedDate: "2026-10-09",
  }).hidden, true);
  assert.equal(projectPlatformObsolescence({
    cutoffDate: "2026-10-10",
    today: "2026-10-16",
    lastDismissedDate: "2026-10-09",
  }).hidden, false);
});

test("future and corrupt persistence fail open", () => {
  for (const lastDismissedDate of ["2026-12-01", "not-a-date", "2026-02-30"]) {
    const snapshot = projectPlatformObsolescence({
      cutoffDate: "2026-10-10",
      today: "2026-10-20",
      lastDismissedDate,
    });
    assert.equal(snapshot.hidden, false);
    assert.equal(snapshot.lastDismissedDate, null);
  }
});

test("pre-cutoff dismissal uses 30 days and post-cutoff dismissal uses 90 days", () => {
  assert.equal(projectPlatformObsolescence({
    cutoffDate: "2026-12-01",
    today: "2026-10-20",
    lastDismissedDate: "2026-10-01",
  }).hidden, true);
  assert.equal(projectPlatformObsolescence({
    cutoffDate: "2026-10-01",
    today: "2026-12-29",
    lastDismissedDate: "2026-10-02",
  }).hidden, true);
  assert.equal(projectPlatformObsolescence({
    cutoffDate: "2026-10-01",
    today: "2027-01-01",
    lastDismissedDate: "2026-10-02",
  }).hidden, false);
});

test("client persistence adapter writes only validated civil dates", async () => {
  const writes: Array<[string, string]> = [];
  const store = createPlatformObsolescencePersistence({
    read: async () => "2026-10-09",
    write: async (key, value) => { writes.push([key, value]); },
  });
  assert.equal(await store.readLastDismissedDate(), "2026-10-09");
  await store.writeLastDismissedDate("2026-10-10");
  assert.deepEqual(writes, [["ui/platform-support/outdated-hidden", "2026-10-10"]]);
  await assert.rejects(() => store.writeLastDismissedDate("2026-13-40"), TypeError);
});
