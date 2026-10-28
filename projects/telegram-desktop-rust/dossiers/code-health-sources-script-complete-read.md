# Code-health sources.py complete read

## Source authority
- Repository: `telegramdesktop/tdesktop`
- Commit: `78ceaa548ae0e055aa56b53fde22064d13f250ef`
- Tree: `76b967833b38eb7aa2cb4047008c07331390e61e`
- Path: `.agents/skills/code-health/scripts/sources.py`
- Blob: `688aa05c4223d0d45b8572cff89fbc5148254844`
- Recursive authority index: 34, immediately after `review.py` at index 33.
- The next strict source entry is `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`.

## Complete source responsibility
`sources.py` is the source-snapshot acquisition boundary for the upstream code-health instruments. It does not own product behavior, source-completeness acceptance, findings, review policy, task routing, build truth, or release acceptance.

- `SOURCE_ROOT`, `LIB_PREFIX`, `LANG_FILE`, `TOP_LEVEL`, `EXTENSIONS`, and `wanted` define a deliberately narrow code-health text scope: SourceFiles, Telegram/lib_* inputs, Telegram/cmake/CMakeLists, and lang.strings, excluding ThirdParty/.git and admitting SourceFiles AGENTS.md.
- `git` is a checked subprocess boundary with captured output and a 600-second timeout.
- `SourceTree` binds repository, file-text map, exact commit, requested ref, acquisition notes, include roots, and top-level submodule heads; `lines` and `root_of` are read-only projections.
- `from_worktree` walks only the declared Telegram top-level scope, filters ThirdParty/.git, reads text plus lang.strings, records HEAD, records Telegram dirty state, and captures current Telegram/lib_* submodule heads.
- `from_ref` resolves `ref^{commit}`, recursively lists the Telegram tree, records top-level `Telegram/lib_*` gitlinks, batch-reads immutable root blobs, and then reads each selected lib repository at the exact gitlink object. A child checkout unavailable locally is explicitly noted and skipped by the upstream instrument.
- `list_tree` uses NUL-delimited recursive `git ls-tree`; `read_blobs` uses `git cat-file --batch` against immutable object IDs.
- `submodule_heads` projects worktree submodule status; `decode` rejects NUL-containing binary content, prefers UTF-8, and falls back to Latin-1.

The narrow scope is correct for upstream code-health measurement but is not sufficient as Fabushi's migration inventory. Fabushi's TDRP authority intentionally covers the complete recursive non-directory source range, direct/nested gitlinks, external nested authorities, build-time acquisitions, archives, generated/resource/tool disposition, and provenance.

## Real callers
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9`
  - `tree_for` selects `sources.from_ref` for an explicit ref or `sources.from_worktree` otherwise.
  - `command_scan`, `command_atlas`, `command_area`, `command_check`, and `command_verify` consume those source snapshots.
  - `command_worktree` reuses `sources.git` for detached exact-commit worktree creation/removal and selected lib_* submodule initialization.
- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` consumes `tree.files`, `tree.ref`, `tree.commit`, and `tree.notes` when producing snapshots/findings.
- `.agents/skills/code-health/scripts/review.py@070b55553052267e8fbc12d70985b447b24bbb02` consumes `tree.repo`, `tree.files`, and `tree.lines` for exact-base change ratchets and move proof.

## Direct regression coverage
`.agents/skills/code-health/scripts/health_test.py@a79addaaceb35848dbd17e94e01ea428aede7788` directly consumes `sources.from_worktree` in:
- `UnusedTests.test_declared_but_never_called`
- `UnusedTests.test_initializer_list_call_is_a_use`
- `MapTests.test_map_naming_a_missing_file_is_stale`
- `AtlasTests.test_check_reports_unmapped_and_unused`
- `AtlasTests.test_scan_names_areas_by_the_atlas`

Its state/command regressions also exercise the source boundary through `health.py`, including `test_working_tree_scan_leaves_the_ledger_alone`, `test_area_rebuilds_a_missing_snapshot_of_the_published_scan`, and `test_source_worktree_is_created_reused_and_removed`.

There is no dedicated direct upstream unit case isolating `from_ref` plus gitlink/unavailable-child behavior. That gap cannot be rewritten as upstream proof. Fabushi's existing exact-head TDRP workflow supplies stronger production evidence by asserting exact target checkout and recursively validating direct/nested gitlinks, acquisition authority, archives, and fail-closed closure.

## Existing-owner decision
Applicable responsibility is source-neutral repository source acquisition/provenance for quality and migration evidence. Fabushi already owns it:
- `scripts/validate-tdrp-revision9-authority.mjs` validates exact target/upstream identity, complete recursive inventory, direct/nested gitlinks, build-time acquisitions, recursive reconciliation, and fail-closed closure.
- `scripts/discover-tdrp-root-open-authority.mjs` discovers live build-time mutable/tool source authorities from the accepted upstream source.
- `.github/workflows/tdrp-revision9-authority.yml` binds execution to exact target SHA and publishes authority/recursive/archive/reachability artifacts.
- `AGENTS.md` keeps source acquisition and process failure in the existing repository quality-health command/evidence lifecycle.
- `TEST_STRATEGY.md` and `evidence-contract.md` require exact-source provenance and stronger evidence for build/runtime/release claims.

No Telegram Python `SourceTree`, second inventory, second gitlink resolver, second acquisition ledger, or second repository-authority root is introduced.

## Parent evidence is historical-only
For parent target `a93967589685ae8bbfa7e80fa0d1b8aaa2a6b2d7`:
- TDRP source authority run `39478280087`, job `118819855616`, artifact `12291075807`, GitHub digest and downloaded ZIP SHA-256 `9dc2c056af519a0a3ab50995a688759297b8c3dd7a412288724e08c31c020d72`.
- Rust desktop runtime run `39478280157`, rust-host job `118819857405`, artifact `12290181212`, GitHub digest and downloaded ZIP SHA-256 `a05ed26500434b01836bb4223ebe2fc5085e1e10dcd62e3d986d99d1b31f6e89`.

These establish parent behavior/provenance only. Creating the sources.py mapping commit demotes them to historical-only; only the new exact HEAD may satisfy current evidence.

## Closure accounting
This commit closes only recursive index 34:
- unknown: 16,165 -> 16,164
- unread: 16,140 -> 16,139
- omitted: 0 -> 0

`source_closure_ready`, baseline acceptance, and release acceptance remain false/open. Recursive generated/resource/tool disposition remains open.

## Traceability
- Requirement: `TDRP-R9-CODE-HEALTH-SOURCE-ACQUISITION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-SOURCE-EXACT-REF-001`, `ORA-TDRP-CODE-HEALTH-SOURCE-GITLINK-PROVENANCE-001`, `ORA-TDRP-CODE-HEALTH-SOURCE-SCOPE-001`
- Invariants: `INV-TDRP-CODE-HEALTH-SOURCE-NO-MUTABLE-AS-EXACT-001`, `INV-TDRP-CODE-HEALTH-SOURCE-NO-PARTIAL-AS-COMPLETE-001`, `INV-TDRP-CODE-HEALTH-SOURCE-NO-SECOND-AUTHORITY-001`
