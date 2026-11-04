# Fixed-U complete read — code-health sources.py

Task round: 113  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`  
Exact blob size: 6,085 bytes

## Complete-read result

The exact blob was read in full from the fixed-U Git object. It owns the source-set boundary and Git/source provenance carrier used by code-health instrumentation; it does not own product behavior, configured build truth, durable findings, task routing, mutation, or acceptance.

### Scope and filtering

- `LANG_FILE` explicitly admits `Telegram/Resources/langs/lang.strings`.
- Ordinary scan scope is limited to `Telegram/SourceFiles/**`, `Telegram/lib_*/**`, `Telegram/cmake/**`, and `Telegram/CMakeLists.txt`.
- `ThirdParty` and `.git` path components are excluded.
- `AGENTS.md` is admitted only under `Telegram/SourceFiles/**`.
- Declared source/style/CMake extensions and any `CMakeLists.txt` are eligible.
- Working-tree discovery starts only from top-level `SourceFiles`, `cmake`, `CMakeLists.txt`, and `lib_*` entries beneath `Telegram`.
- `lib_roots` derives source include roots from observed `Telegram/lib_*` paths; `SourceTree.roots` is `Telegram/SourceFiles` plus those library roots.

### Exact-ref authority

`from_ref` resolves the supplied ref with `rev-parse --verify <ref>^{commit}`, recursively enumerates the tracked `Telegram` tree, separates eligible parent-repository blobs from top-level `Telegram/lib_*` gitlinks, reads parent blobs by object ID, and records every gitlink path/pinned OID. For each gitlink it tries to read the exact pinned tree from the local submodule object database. If the pinned object is unavailable, it appends `<path> at <oid> unavailable locally, skipped` and continues.

That continuation is partial diagnostic behavior, not completeness authority. Fabushi's existing fail-closed repository governance requires the path/OID and missing-object note to remain explicit unknown coverage; consumers must not turn a skipped dependency into a complete scan or zero-finding claim.

### Working-tree authority

`from_worktree` walks the current eligible checkout roots, excludes `ThirdParty` and `.git`, normalizes readable text, adds the language catalog, records current `HEAD`, detects Telegram-local dirtiness via `git status --porcelain -- Telegram`, labels the subject `WORKTREE`, and records current `lib_*` submodule status.

This is intentionally mutable developer evidence. It cannot become canonical durable quality state solely because a `HEAD` exists, and concurrent working-tree/index/submodule changes stale the observation.

### Git/blob/text contracts

- `git` runs `git -C <repo>` with captured output, `check=True`, and a 600-second timeout; process failures propagate.
- `list_tree` uses recursive NUL-delimited `ls-tree` and returns mode/kind/OID/path tuples.
- `read_blobs` batches exact object IDs through `cat-file --batch` and binds decoded content to the enumerated path.
- `decode` rejects NUL-bearing data as non-text, prefers UTF-8, and falls back to Latin-1.
- `read_text` returns no text on filesystem read failure and normalizes CRLF to LF so working-tree representation matches Git diff/storage line endings.
- `SourceTree` carries repository, decoded file map, resolved commit, subject ref, notes, source roots and gitlink pins; `lines` and `root_of` are read-only derived helpers.

A NUL-bearing eligible blob or failed working-file read can therefore make the decoded file map smaller than the eligible enumeration. Absence from the decoded map is not proof the tracked input does not exist; completeness-dependent evidence must preserve that distinction.

## Exact fixed-U consumers

Direct module consumers verified from fixed-U blobs:

- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`: imports `sources`; selects `from_ref` vs `from_worktree`; uses the Git runner for fetch/ref/worktree/submodule operations; composes scan, area, check and verify subjects.
- `harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1`: imports `sources`; reuses `read_blobs`, `list_tree` and `git` for exact helper/reference/history/queue evidence.
- `harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74`: creates both `from_ref` and `from_worktree` harness subjects.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794`: creates working-tree source carriers for quality/report contracts.
- `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32`: imports `LANG_FILE`, `git` and `wanted` for diff-scope review.
- `metrics.py@42f42b1100f88bd85977b66dfb6bf13e9dc88e46`: imports `LANG_FILE`.
- `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` and `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df`: import the common Git runner.

The fixed-U `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` is the command-level consumer contract: durable scan uses an exact ref; pinned submodule objects missing locally are skipped and named in notes; failed fetch is a failed run; only exact-ref scans may be recorded in canonical state.

Downstream report/metrics/include/clone/review/atlas/harness logic consumes the `SourceTree` result but retains its own previously accounted semantics.

## Existing-owner-first mapping

Current Fabushi root governance already states that the source-neutral quality-health command delegates source acquisition to repository authority and that mutable/incomplete source evidence must not be promoted into durable clean state. Therefore this source file does not justify a new runtime, scanner, findings store, source index, task queue, ownership graph, or platform-core production component.

Applicable owner: existing Fabushi repository authority plus repository quality/test-evidence governance.  
Command/lifecycle owner: existing repository quality-health command orchestration.  
Target contracts: `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `projects/fabushi-communication-platform/quality/evidence-contract.md`.  
Platform-core production delta: none.  
New ADR: none.

## Authored Stage-A contracts — not run

- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-SCOPE-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-EXACT-REF-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-WORKTREE-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GITLINK-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-DECODE-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GIT-FAILURE-001`
- `CONTRACT-TDRP-CODE-HEALTH-SOURCE-PROVENANCE-001`
- `INT-TDRP-CODE-HEALTH-SOURCE-CONSUMER-COMPOSITION-001`

No build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, runtime acceptance, Actions/check/status/workflow query, or workflow trigger was performed.
