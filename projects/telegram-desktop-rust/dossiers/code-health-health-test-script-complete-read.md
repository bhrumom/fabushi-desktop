# Code-health health_test.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
- Path: `.agents/skills/code-health/scripts/health_test.py`
- Blob: `2176aea672fd9ed16c01342939fbffb41490ae73`
- Recursive authority position: array index 28, immediately after `health.py` at index 27.

## Complete responsibility read

The file is a Python `unittest` integration/qualification harness. It imports the existing code-health instruments directly: `atlas`, `clones`, `cmake_model`, `cpp_text`, `health`, `includes`, `metrics`, `report`, `review`, and `sources`. Its temporary `Repo` helper creates disposable Git repositories and fixture source/build/map state for falsifiable tests; those helpers are test mechanics, not a shipping or durable owner.

The test classes cover:
- `TextTests`: function span/initializer-list recognition, false-positive rejection, member counting, and block-comment/code-line behavior.
- `GraphTests`: source-build target/PCH relationships, layering violations, include reach and weight.
- `CloneTests`: substantive repeated-block detection and declaration-mirroring rejection.
- `UnusedTests`: declared-but-never-called detection and initializer-list usage.
- `MapTests`: stale map references.
- `CheckTests`: clean change, new layering error, size ratchet, copied code, stranded references, untracked files, known-crossing warning, platform references, orphaned helper/style/key detection, complete deletion and moved-definition negatives.
- `VerifyTests`: `health.py verify` preserves present/resolved/unknown exit/result semantics.
- `AtlasTests`: specificity, unmapped/unused pattern coverage, atlas area naming and large-area split heuristics.
- `StateTests`: durable scan state, once-per-day skip, forced append-only history, mutable working-tree nonpublication, missing snapshot rebuild from published exact revision, and exact source worktree create/reuse/safe removal.
- `MoveTests`: move-only extraction/new-file/reordering positives and changed initializer/order/logic negatives.

## Real caller and execution boundary

The file is self-executing through `unittest.main()` and directly invokes the code-health libraries plus `health.py` subprocess/handler boundaries. Repository code search for the literal file name has no separate product/runtime caller; the applicable responsibility is therefore the regression/qualification contract embodied by its cases, not preservation of Python test wiring.

`.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` defines the code-health operational workflow that these tests qualify. The immediately preceding `health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` is the command/state owner for the State/Verify command cases; text/build/include/clone/atlas/report/review modules retain their own facts and semantics.

## Existing-owner-first decision

Applicable responsibility: integrated repository-quality regression qualification with positive and negative controls across the existing source-text, source-build, include/dependency, clone, atlas, report, review and health-command owners. Fabushi already owns these contracts in repository quality governance, `quality/TEST_STRATEGY.md`, and `quality/evidence-contract.md`.

The equivalent implementation is source-neutral test strategy/evidence, not a copy of Telegram's Python unittest harness, temporary fixture repository, sample C++ strings, or a second scanner/findings/ownership/task runtime. The new integrated-regression sections make the cross-owner qualification boundary explicit while retaining each underlying semantic owner.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-INTEGRATED-REGRESSION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-INTEGRATED-POSITIVE-NEGATIVE-001`, `ORA-TDRP-CODE-HEALTH-STATE-LIFECYCLE-001`, `ORA-TDRP-CODE-HEALTH-RESULT-DISTINCTIONS-001`
- Invariants: `INV-TDRP-CODE-HEALTH-TEST-NO-SECOND-OWNER-001`, `INV-TDRP-CODE-HEALTH-TEST-EXACT-EVIDENCE-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-INTEGRATED-REGRESSION-001`
- Integration: `INT-TDRP-CODE-HEALTH-INTEGRATED-REGRESSION-001`
- Fault recovery: `FAULT-TDRP-CODE-HEALTH-TEST-NEGATIVE-CONTROLS-001`
- Security: `SEC-TDRP-CODE-HEALTH-TEST-AUTHORITY-BYPASS-001`
- Exploratory: `EXP-TDRP-CODE-HEALTH-INTEGRATED-REGRESSION-001`
- Release gates: `G-EVIDENCE`, `G-INVENTORY`, `G-RELEASE`

## Failure boundaries

A harness pass cannot establish production/runtime behavior, configured-build truth, packaging, or release acceptance. Missing negative controls, mutable/non-exact authority used as canonical evidence, a test helper mutating canonical state, collapsed warning/error or present/resolved/unknown semantics, unsafe temporary-worktree removal, or copying upstream Python test infrastructure as a Fabushi product owner are non-passing.

## Live rebaseline fe4afa4fbe9f
The source blob changed to `ae0255931760d6164992e525a502fccca37f5507`. The added regression asserts harness report files are produced and records unavailable coverage in the fixture. Existing integrated-regression qualification remains the owner. The new harness implementation/test sources remain separate unread inventory entries.
