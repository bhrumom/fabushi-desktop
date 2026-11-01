# Complete read — fixed-U code-health `report.py`

Status: Stage A mapped/implemented as source-neutral repository quality contract; executable validation not run; accepted=false  
Task round: 111  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)  
Source: `.agents/skills/code-health/scripts/report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` (19,769 bytes)

## Exact responsibility

The complete blob was read. It is the repository-quality aggregation, stable-finding lifecycle and report-projection layer; it is not shipping product behavior and does not own exact-ref acquisition or durable publication.

- `snapshot(tree, queue, churn_days=180, cost_days=120)` composes approved atlas classification with fallback area grouping, source-derived CMake/include costs and PCH closure, source metrics, history churn/task cost, clones, layering, unused candidates, coarse tested-file attribution and helper inventory. It attaches area/churn/hotspot/target/test/build-cost observations to files, aggregates areas, computes task medians/review means and totals, and returns one schema-versioned snapshot carrying `ref`, `commit`, `checked_at_utc`, source notes, limits and raw component evidence.
- `map_references(tree)` scans non-atlas nested `AGENTS.md` files for backtick C++/style paths and reports only names for which no local, `Telegram/SourceFiles/`, or repository-root candidate exists. Atlas patterns are deliberately handled separately by atlas checking.
- `findings(snap)` creates stable `kind:key` identities, class/area/title/paths/raw-data and a crude priority `score = impact * (1 + churn/10)`. It applies the declared file/class/function/build-cost/clone/layering/unused/map/atlas/hot-area/helper thresholds. Scores/classes are routing signals; they are not deletion, architecture or release authority.
- `merge(previous, current, checked_at, unavailable=())` preserves prior `first_seen` for recurring identities, marks current findings open, marks vanished unresolved findings resolved with `resolved_at`, retains unavailable unresolved kinds as ignored/unverified rather than falsely resolving them, preserves already-resolved findings, and drops resolved history older than 180 days.
- `markdown(snap, ledger, previous)` renders exact source identity/notes, totals and prior deltas, new/resolved summaries, bounded hotspot areas, top open findings, per-kind counts, build-cost tables and helper-lifecycle coverage wording.

## Direct callers / consumers at fixed U

- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` is the primary caller. `command_scan` calls `snapshot -> findings -> merge -> markdown`, applies ignored overrides, writes `report.md` plus helper evidence, and when durable state is selected writes the exact commit/ref/totals/findings ledger atomically and appends history. It also uses `snapshot` for atlas drafting, published-snapshot reconstruction/area reports and before/after finding verification. This confirms command orchestration/publication remains outside `report.py`.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` directly exercises `map_references`, atlas-over-fallback snapshot classification, stale-atlas findings, durable-vs-working-tree scan placement and published snapshot reconstruction.
- `harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74` directly exercises `merge`: unavailable helper-retention evidence changes an unresolved finding to `unverified`, recurring identity retains `first_seen`, ordinary disappearance resolves, and already-resolved remains resolved even if that evidence kind later becomes unavailable.
- `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` imports `LIMITS` and `file_limit`, reusing the same threshold policy in diff-time review.
- The upstream skill documentation `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` defines the durable report/ledger/history/run directory contract consumed by the command layer.

Snapshot providers are the already-independent fixed-U `atlas.py`, `clones.py`, `cmake_model.py`, `cpp_text.py`, `history.py`, `harness.py`, `includes.py`, and `metrics.py` entries. Their responsibilities are not re-claimed by this row.

## Existing-owner-first disposition

The portable responsibility maps to the existing Fabushi repository quality/test-evidence governance in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `quality/evidence-contract.md`. The already-declared repository quality-health command orchestration remains the publication owner. Shipping Agent/Coordinator/Host/Runner, product analytics/performance telemetry, platform-core runtime, a second findings ledger/task queue and a copied upstream Python report runtime are rejected owners.

No platform-core production delta and no new ADR are required.

## Authored contracts — not run

- `CONTRACT-TDRP-CODE-HEALTH-REPORT-SNAPSHOT-001`
- `CONTRACT-TDRP-CODE-HEALTH-REPORT-FINDINGS-001`
- `CONTRACT-TDRP-CODE-HEALTH-REPORT-MERGE-001`
- `CONTRACT-TDRP-CODE-HEALTH-REPORT-MAP-REFERENCES-001`
- `CONTRACT-TDRP-CODE-HEALTH-REPORT-MARKDOWN-001`
- `CONTRACT-TDRP-CODE-HEALTH-REPORT-UNAVAILABLE-EVIDENCE-001`
- `INT-TDRP-CODE-HEALTH-REPORT-HEALTH-COMPOSITION-001`

Stage B must execute these against the frozen exact-head candidate in GitHub Actions. Source review in Stage A is not passing execution evidence.
