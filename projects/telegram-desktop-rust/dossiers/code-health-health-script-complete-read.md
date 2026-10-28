# Code-health health.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
- Path: `.agents/skills/code-health/scripts/health.py`
- Blob: `f08675ef7a03c016d85f91c60b8f29e99454abe9`
- Recursive authority position: array index 27, immediately after `cpp_text.py` at index 26.

## Symbols and command boundary

The module is the code-health command/orchestration boundary rather than a metric implementation. Its command surface is `scan`, `worktree`, `atlas`, `area`, `check`, `moves`, and `verify`. Durable-state helpers include `published_state`, `snapshot_cache`, `cache_snapshot`, `write_atomic`, and `load`; authority helpers are `fetch_dev`, `ensure_commit`, and `tree_for`.

`command_scan` rejects durable state for a mutable working-tree scan, binds exact-ref scans to source acquisition, composes `report.snapshot/findings/merge/markdown`, preserves ignored finding state, writes the canonical ledger atomically, appends history, maintains a local raw-snapshot cache, and emits explicit run/totals/open-finding output. The daily skip is based on canonical state unless forced.

`command_worktree` creates or reuses a detached inspection worktree at the exact requested or latest-published commit, verifies HEAD equality, initializes only pinned `Telegram/lib_*` dependencies discovered from that source tree, and restricts removal to the dedicated `code-health-source-` temporary path identity.

`command_atlas` delegates approved-atlas parsing/checking and skeleton generation to the atlas/report/include/build-model owners; `command_area` reads the canonical published snapshot (rebuilding it from the exact published revision when its cache is absent); `command_check` overlays staged content when requested and delegates warning/error semantics to review; `command_moves` delegates line-accounting/move-only adjudication; `command_verify` compares base and working-tree findings and preserves present/resolved/changed/unknown distinctions. The top-level command boundary converts an underlying `CalledProcessError` to an explicit failed outcome rather than a clean result.

## Real callers and dependencies

- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` directly specifies all health.py command invocations, scheduled exact-`origin/dev` scanning, state publication, area/atlas/survey inputs, and perform-task review use.
- `.agents/skills/code-health/scripts/health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` imports `health` directly and exercises recorded scan once-per-day behavior, mutable-worktree nonpublication, missing-cache rebuild from published state, exact detached worktree creation/reuse/removal, plus check/verify/moves/atlas behavior.
- `report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0`, `includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c`, and the already-read atlas/clone/build-model/text helpers are delegated evidence libraries, not command/state owners.
- The skill routes proposals through the existing process-inbox/perform-task lifecycle; health.py does not own a separate queue or product runtime.

## Traceability

The following IDs are the complete ledger traceability inventory for this source responsibility and are bound to the source observations and owner decision in this dossier.

- capability_ids: `CAP-REPOSITORY-QUALITY-GOVERNANCE`, `CAP-REPOSITORY-RESPONSIBILITY-GOVERNANCE`, `CAP-REPOSITORY-TASK-EVIDENCE`
- dependency_ids: `.agents/skills/code-health/SKILL.md`, `.agents/skills/code-health/scripts/atlas.py`, `.agents/skills/code-health/scripts/cmake_model.py`, `.agents/skills/code-health/scripts/cpp_text.py`, `.agents/skills/code-health/scripts/includes.py`, `.agents/skills/code-health/scripts/metrics.py`, `.agents/skills/code-health/scripts/report.py`, `.agents/skills/code-health/scripts/review.py`, `.agents/skills/code-health/scripts/sources.py`, `.agents/skills/code-health/scripts/health_test.py`, `.agents/skills/perform-task/SKILL.md`, `.agents/skills/process-inbox/SKILL.md`, `Fabushi repository AI development lifecycle`, `Fabushi Test Strategy`, `Fabushi Test Evidence Contract`
- requirement_ids: `TDRP-R9-CODE-HEALTH-COMMAND-ORCHESTRATION-001`
- oracle_ids: `ORA-TDRP-CODE-HEALTH-EXACT-DURABLE-SCAN-001`, `ORA-TDRP-CODE-HEALTH-NONCANONICAL-WORKTREE-001`, `ORA-TDRP-CODE-HEALTH-COMMAND-RESULTS-001`
- invariant_ids: `INV-TDRP-CODE-HEALTH-EXACT-AUTHORITY-001`, `INV-TDRP-CODE-HEALTH-STABLE-STATE-001`, `INV-TDRP-CODE-HEALTH-NO-SECOND-OWNER-001`
- unit_case_ids: `health_test.StateTests.test_recorded_scan_writes_state_once_a_day`, `health_test.StateTests.test_working_tree_scan_leaves_the_ledger_alone`, `health_test.StateTests.test_area_rebuilds_a_missing_snapshot_of_the_published_scan`, `health_test.StateTests.test_source_worktree_is_created_reused_and_removed`, `health_test.VerifyTests.test_deleted_candidate_is_resolved`, `health_test.MoveTests.test_extracting_a_function_into_a_new_file_is_move_only`
- contract_case_ids: `CONTRACT-TDRP-CODE-HEALTH-COMMAND-ORCHESTRATION-001`
- integration_case_ids: `INT-TDRP-CODE-HEALTH-SKILL-COMMANDS-001`
- fault_recovery_case_ids: `FAULT-TDRP-CODE-HEALTH-FETCH-FAILURE-001`, `FAULT-TDRP-CODE-HEALTH-HEAD-MISMATCH-001`, `FAULT-TDRP-CODE-HEALTH-MALFORMED-STATE-001`
- security_case_ids: `SEC-TDRP-CODE-HEALTH-AUTHORITY-BYPASS-001`
- exploratory_charter_ids: `EXP-TDRP-CODE-HEALTH-COMMAND-ORCHESTRATION-001`
- release_gate_ids: `G-EVIDENCE`, `G-INVENTORY`, `G-RELEASE`
- test_fixture_ids: `code-health-health-command-contract`, `repository-quality-health-orchestration-contract`, `exact-source-canonical-state-contract`
- test_environment_ids: `GitHub Actions hosted runners`

## Responsibility decision

Applicable responsibility: source-neutral repository quality-health command orchestration and durable evidence-state publication. It includes exact-source fencing, canonical-versus-temporary run separation, stable findings/history publication, cache/rebuild semantics, safe detached inspection, command result preservation, and delegation to existing quality owners.

This is workflow-equivalent, not a Python port. Current Fabushi already has the correct owner in repository quality/AI development governance. The implementation extends that owner in `AGENTS.md`, `quality/TEST_STRATEGY.md`, and `quality/evidence-contract.md`; no Telegram-specific health runtime, second scanner, second findings ledger, ownership graph, or task queue is introduced.

## Failure and lifecycle boundaries

Fail closed on stale/partial source, missing exact commit, failed fetch/Git/process execution, detached HEAD mismatch, malformed canonical state, atlas coverage negatives, review errors, unresolved/present/changed/unknown findings, or non-move-only evidence. A temporary report/cache is never durable authority. Metrics and clean scanner output remain supporting repository-quality evidence and cannot prove product/runtime/build/release acceptance.

## Live rebaseline fe4afa4fbe9f
The source blob changed to `4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`. The command now delegates harness inspection, emits `harness.json`/`harness.md`, and preserves prior retention findings as unverified when coverage is unavailable. This remains command orchestration under the existing quality/evidence owner; the new harness implementation file is a separate unread source authority and is not subsumed by this row.
