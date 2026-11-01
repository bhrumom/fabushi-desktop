# Complete read — fixed-U code-health `review.py`

Status: Stage A mapped/implemented as source-neutral repository quality/review contract; executable validation not run; accepted=false  
Task round: 112  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)  
Source: `.agents/skills/code-health/scripts/review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` (21,693 bytes)

## Exact responsibility

The complete blob was read. It owns the change-scoped repository review ratchet and move-only proof. It has no shipping product behavior and owns no durable findings/task state.

- `tracked_path`, `untracked`, `changed`, `diff_lines`, `base_text` and `added_hunks` bind review to an explicit base plus staged/working-tree subject, include untracked in-scope files for working-tree review, and preserve added/removed hunk content.
- `check(tree, base, staged, strict)` composes the source-derived CMake model, include graph, report file/class limits, clone copied-from evidence and orphan/reference analysis. New oversized files and first cross-target include violations are errors. Oversized growth beyond 20 lines and copied-from additions are strict-escalatable warning/error rules. Existing target-pair crossings, large-class growth, heavy-header build-cost growth, stranded references and last-use orphans remain warnings.
- Fresh-header build-cost review is source-derived: it computes newly reachable header lines times existing reach and warns only when the work is at least `BUILD_SHARE=0.002` of the include-cost proxy. This is not configured-build or wall-clock proof.
- `definitions`, `mention_lines`, `identifiers`, `defines`, `orphaned`, `platform_only` and `stranded` conservatively distinguish removed definitions still referenced from symbols whose last use was removed. Platform files are scanned, methods are qualifier-scoped, style/language definitions are recognized, numeric-literal suffixes are excluded, and exemptions are reused from the metrics policy.
- `grown_classes` warns only when an already-limit-sized class gains methods. `heavier` warns on material fresh-header cost using the existing include graph and cost model.
- `moves`, `out_of_order`, `comparable` and `declaration_like` prove Q1-style move-only changes: all substantive removed lines reappear, added substantive lines are moved or declared scaffolding/declarations, and moved statements preserve source order. Whole functions/groups may reorder only at boundaries; changed logic/initializer values, lost lines or substantive novelty fail the proof.

## Direct callers / consumers at fixed U

- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` imports `review`. `command_check` optionally reconstructs staged content with `review.changed`, calls `review.check`, prints Errors/Warnings and exits 1 only when errors exist. `command_moves` calls `review.moves`, prints move metrics/diagnostics and exits 0 only for `MOVE_ONLY`.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` directly imports `review`. `CheckTests` covers clean changes, hard new target crossings, 20-line oversized growth tolerance and strict escalation, copied-from warning, new oversized untracked file, known-crossing warning, removed-definition/platform references, orphaned helpers/styles/language keys, complete deletion, moved definitions, CRLF/LF stability and false-positive controls. `MoveTests` covers staged/untracked extraction, changed initializer/logic, statement reordering and whole-function reordering.
- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` defines `health.py check [--strict]` and `moves` as the ordinary pre-review ratchet/proof surface and states the warning/error split.
- `clones.py`, `cmake_model.py`, `cpp_text.py`, `includes.py`, `metrics.py`, `report.py`, and `sources.py` are independently accounted provider entries; this row consumes their qualified facts and does not re-claim them.

## Existing-owner-first disposition

The portable responsibility maps to the existing Fabushi repository quality/test-evidence governance in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `quality/evidence-contract.md`, with the already-established repository quality-health command orchestration remaining the command/exit-code owner. Shipping Agent/Coordinator/Host/Runner, product performance telemetry, platform-core runtime, a copied upstream Python review runtime, a second scanner/findings/task queue, and configured-build evidence are rejected owners.

No platform-core production delta and no new ADR are required.

## Authored contracts — not run

- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SCOPE-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SEVERITY-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-ORPHAN-REFERENCE-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLONE-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-INCLUDE-LAYERING-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-BUILD-COST-001`
- `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLASS-GROWTH-001`
- `CONTRACT-TDRP-CODE-HEALTH-MOVE-ONLY-001`
- `INT-TDRP-CODE-HEALTH-REVIEW-HEALTH-COMPOSITION-001`

Stage B must execute these against the frozen exact-head candidate in GitHub Actions. Source review in Stage A is not passing execution evidence.
