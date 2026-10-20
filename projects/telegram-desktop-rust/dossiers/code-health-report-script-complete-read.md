# Code-health report.py complete read

## Source authority
- Repository: `telegramdesktop/tdesktop`
- Commit: `78ceaa548ae0e055aa56b53fde22064d13f250ef`
- Path: `.agents/skills/code-health/scripts/report.py`
- Blob: `a1177613670e933694f545f387ca7e7ed8cf0ee0`
- Root tree: `76b967833b38eb7aa2cb4047008c07331390e61e`.
- Exact-head parent run `39462234441` proved the current 7,012-entry root tree, 6,679 blobs, 36 direct gitlinks, 16,201 reconciled recursive non-directory entries, recursive gitlink fixed point unresolved=0, and current acquisition/reachability evidence. This new mapping commit makes that parent-head artifact historical-only until its own exact-head workflow completes.
- Recursive authority index 32 is `report.py`, immediately after `metrics.py` at index 31. The next strict entry is `.agents/skills/code-health/scripts/review.py@070b55553052267e8fbc12d70985b447b24bbb02`.

## Complete source responsibility
`report.py` is the aggregation and durable finding-shape layer of upstream repository code-health tooling. It does not implement product behavior.

- `LIMITS` defines report-owned investigation thresholds for translation-unit/header size, large classes, long functions, clone length, compile-unit weight, high rebuild-share headers, churn, and the number of heavy headers retained.
- `CLASS_BY_KIND` maps finding kinds to task classes (Q0-Q5 or map) without itself approving or routing a task.
- `short(path)` and `file_limit(path)` normalize report presentation and choose the applicable file-size threshold.
- `snapshot(tree, queue, churn_days, cost_days)` composes the approved atlas when present, source-derived CMake model, include graph/PCH closure and build-cost facts, structural metrics, maintenance history, optional task cost history, clone evidence, layering evidence, unused candidates and tested-file attribution into one exact-source snapshot. It derives per-file area/churn/hotspot/target/test/build-cost facts, per-area rollups and global totals.
- `map_references(tree)` identifies stale C++/Objective-C++/style paths named by non-atlas `AGENTS.md` area maps.
- `findings(snap)` assigns stable identities `kind:key`, class, area, title, paths, score and data to thresholded large-file/class/function/build-cost findings, clones, layering, unused candidates, stale maps, unmapped/stale atlas coverage, and untested hot areas.
- `merge(previous, current, checked_at)` preserves `first_seen`, marks current findings open, resolves disappeared open/ignored findings, and retains recently resolved findings for 180 days.
- `markdown(snap, ledger, previous)` renders totals and deltas, new/resolved findings, hotspot-area summaries, top findings, finding-kind counts, rebuild-heavy headers and heavy compilation units.

## Callers and regression coverage
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` is the production command caller. Durable scan calls `report.snapshot`, `report.findings`, `report.merge`, and `report.markdown`; area/report/change paths also consume report snapshots/findings.
- `.agents/skills/code-health/scripts/health_test.py@a79addaaceb35848dbd17e94e01ea428aede7788` directly covers `MapTests.test_map_naming_a_missing_file_is_stale`, `AtlasTests.test_scan_names_areas_by_the_atlas`, `StateTests.test_recorded_scan_writes_state_once_a_day`, `StateTests.test_working_tree_scan_leaves_the_ledger_alone`, and related scan/verify flows.
- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` defines the human workflow around scans, findings, reports, checks and task classes.

## Existing-owner decision
Applicable responsibilities are exact-source report aggregation, stable finding identity/lifecycle, prioritization score presentation, and report serialization. Fabushi already owns these under source-neutral repository quality-health governance and evidence contracts. Reuse that owner rather than porting Telegram's Python report runtime or creating a second findings ledger, task queue, responsibility atlas, scanner, or release gate.

Existing owner candidates:
- `AGENTS.md` Repository quality-health governance.
- `AGENTS.md` Repository quality-health command orchestration governance.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`.
- `projects/fabushi-communication-platform/quality/evidence-contract.md`.

The canonical target symbol for this responsibility is the existing unique sentence: `Persistent findings use stable semantic identities and lifecycle state.`

## Boundaries and failure semantics
A report is valid only for one exact acquired source snapshot and the qualified inputs it composes. Missing source/dependency acquisition, stale/mutable input, parser/instrument failure, incomplete coverage, or an unavailable history source remains explicit incomplete/non-passing evidence. A zero count cannot be inferred from a failed input.

Finding scores and classes are prioritization aids, not proof of product correctness, safe deletion, ownership, build success, security, packaging or release acceptance. Atlas ownership remains with the responsibility-atlas owner; source/build/text/include/clone/history facts remain with their respective qualified instruments; command publication and task routing remain with existing orchestration/review owners.

## Traceability
- Requirement: `TDRP-R9-CODE-HEALTH-REPORT-AGGREGATION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-REPORT-EXACT-SNAPSHOT-001`, `ORA-TDRP-CODE-HEALTH-STABLE-FINDING-LIFECYCLE-001`, `ORA-TDRP-CODE-HEALTH-REPORT-PRESENTATION-BOUNDARY-001`
- Invariants: `INV-TDRP-CODE-HEALTH-REPORT-PROVENANCE-001`, `INV-TDRP-CODE-HEALTH-REPORT-NO-SECOND-OWNER-001`, `INV-TDRP-CODE-HEALTH-REPORT-NO-ACCEPTANCE-001`
- Direct regression cases: `MapTests.test_map_naming_a_missing_file_is_stale`, `AtlasTests.test_scan_names_areas_by_the_atlas`, `StateTests.test_recorded_scan_writes_state_once_a_day`, `StateTests.test_working_tree_scan_leaves_the_ledger_alone`.

This closes only `report.py`. Generated/resource/tool disposition and the remaining semantic migration remain open; `source_closure_ready` stays false.
