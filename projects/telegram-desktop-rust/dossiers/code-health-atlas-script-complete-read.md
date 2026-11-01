# Complete read — .agents/skills/code-health/scripts/atlas.py

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
Source: `.agents/skills/code-health/scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43`

## Inventory identity

The write lease for this closure is Fabushi `main@0204e6730cd79baac9149dd3560efdd0dd974d38`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. Fresh authority artifact `12233497396` from run `39351424999` has digest `sha256:15ee3666d411728ef5a205536fc3d4905483edec558d4acee5af80abfba6094f` and is bound to validation head `1fca285aed86266fc9ee7c2c862dd3a1b7e489a3`; that validation head merged without content drift as Fabushi main `0204e6730cd79baac9149dd3560efdd0dd974d38`.

The fresh recursive authority contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 24 as this exact path/blob. Immediately before this write the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,179, unread=16,152, omitted=0. This closure accounts only entry 24.

## Complete source responsibility

`atlas.py` is a library of source-neutral atlas parsing, deterministic path classification, atlas coverage validation and candidate-skeleton evidence generation. It does not own the command-line surface, owner approval, product runtime, task queue or semantic responsibility judgments.

### Parse and classification

- `ATLAS_PATH` points at the approved source atlas text; `SPLIT_LINES=30000` and `MIN_LINES=3000` are drafting thresholds, not ownership policy.
- `relative(path)` strips the repository's source-root prefix before pattern comparison.
- `match(pattern, path)` defines deterministic specificity: exact file (rank 3) beats glob (rank 2), glob beats directory prefix (rank 1), and longer patterns beat shorter patterns within the same class.
- `Atlas.__init__` reads level-3 area headings and backtick path patterns from `Path:`/`Paths:` lines, preserving the area-to-pattern mapping and an in-memory classification cache.
- `Atlas.classify` computes the best specificity per area, returns the winning area plus every other area with the exact same best score, and caches that result. Equal-best claims are surfaced as ambiguity rather than hidden by iteration order.
- `Atlas.area_of` returns only the selected area for consumers that do not need ambiguity detail.
- `load` reads explicit atlas text or the canonical atlas file from the exact source tree; missing text or a parsed atlas with no areas yields no canonical classifier.

### Coverage validation

`mapped_file` bounds this instrument to C/C++ translation units, headers and `.style` files under `Telegram/SourceFiles/` and `Telegram/lib_*`. `check` enumerates that complete declared scope and returns three independent negative sets:

- `unmapped`: no atlas pattern owns the file;
- `ambiguous`: multiple areas have equal best specificity for the file;
- `unused`: a configured atlas pattern matches no mapped file.

The library reports facts; the caller decides the command result. An approved atlas is non-passing when any of these coverage negatives exists.

### Candidate skeleton

`candidates(files)` accumulates line totals for every ancestor, always retains `Telegram/SourceFiles` as a source root, recognizes each `Telegram/lib_*` source root, and recursively splits a candidate over 30,000 lines only into children of at least 3,000 lines. `owner(path, folders)` assigns residual descendants to the nearest selected ancestor. These thresholds suggest review units; they do not establish semantic ownership.

`name_groups` summarizes direct-file stem groups by source lines after removing a common underscore prefix. `co_change` reads the exact commit's recent non-merge history for `Telegram/SourceFiles`, measures per-area commit counts, same-commit Jaccard partners and one-area-only commits; it is repository-history evidence, not a responsibility oracle.

`skeleton(tree, snap, graph)` combines the exact source tree, the canonical health snapshot and include graph. It preserves mapped file lines (including style files), candidate membership, cross-candidate dependency edges, recent co-change, task cost, churn and tested-line facts. Each row records folder, lines/files, dominant build target, largest files, optional direct-file name groups, outgoing/incoming dependency counts, meaningful co-change partners, commit/solo counts, churn, task count/median duration and tested lines.

`markdown(rows)` renders the same candidate evidence for human review. It explicitly treats independent `lib_*` history differently because co-change is not measured across repositories. It also marks candidates below the minimum threshold as merge-review candidates rather than silently changing ownership.

### CLI and error boundary

The actual CLI owner is `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9`.

For `health.py atlas --check <draft>`, `command_atlas` constructs an `Atlas`, calls `atlas.check`, prints area count plus unmapped/ambiguous/unused totals, detailed negatives, empty areas and area line totals, and returns 1 if any unmapped/ambiguous/unused set is non-empty; otherwise it returns 0.

For skeleton generation, `command_atlas` resolves one source tree, canonical snapshot and include graph, calls `atlas.skeleton`, then writes `skeleton.json` and `skeleton.md` under `<state>/atlas/<UTC-stamp>-<commit-prefix>/` or an explicit temporary directory when no durable state is supplied. Its result JSON binds commit, directory, candidate count and total lines.

The `health.py` top-level command dispatcher catches `subprocess.CalledProcessError`, reports the underlying Git failure and returns 2. Coverage failure and execution failure therefore remain distinct. `atlas.py` itself does not swallow repository failures or turn a failed Git history read into clean coverage.

## Real consumers and routing relationship

- `report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` loads the approved atlas and uses its area identity before the coarse `metrics.area_of` fallback; it also retains `atlas.check` results in the scan snapshot.
- `health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` exercises exact/glob/prefix specificity, unmapped/unused coverage, scan area naming/stale-atlas integration, large-directory splitting and nearest candidate ownership.
- `sources.py@688aa05c4223d0d45b8572cff89fbc5148254844` owns exact-ref source-tree and Git process access; `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4` supplies the mapped C/C++ extension sets.
- `code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` and `references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` define the human draft -> describe -> consolidate -> independently verify -> owner approve lifecycle. A generated skeleton is never self-authorizing.
- `references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f` refines an approved atlas area; `references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` consumes atlas identity for deep source review; neither is owned by this Python library.
- `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e`, `perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, `pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` and `phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` consume ordinary routed tasks only after class/trust/placement checks. Atlas instrumentation never inserts a task directly.
- `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` and `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` are invocation adapters, not responsibility owners.
- `.agents/skills/continue/SKILL.md` and the ordinary queue lifecycle schedule already-created tasks; atlas evidence does not cross that boundary by implication.
- upstream root `AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086` makes atlas/maps placement authority and keeps browsing/survey evidence distinct from task execution.

Thus the real chain is: exact source/tree + canonical health facts -> atlas classifier/check or candidate skeleton -> human responsibility review and independent verification -> explicit owner approval -> ordinary scans/maps/surveys consume stable area identity -> separately authorized quality proposals may enter the normal task lifecycle.

## Existing-owner-first resolution

Current Fabushi already has the applicable source-neutral owner:

1. root `AGENTS.md` owns exact-authority repository work, responsibility-atlas/map governance and ordinary task lifecycle;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns independent qualification and negative controls;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns reproducible exact-source, classification, coverage and command evidence;
4. the ordinary Fabushi task/review/evidence lifecycle owns any later implementation, review and acceptance.

The atlas reference entry was already mapped independently, but `atlas.py` carries additional executable responsibility that must remain auditable: parse rules, exact/glob/prefix specificity, equal-best ambiguity, missing-atlas behavior, mapped scope, coverage negative sets, threshold-based candidate generation, nearest-candidate assignment, source-fact aggregation, cross-repository history boundary, output formats and distinct check/execution result semantics. This closure extends those existing owners only.

Rejected alternatives:

- shipping Agent/Coordinator/Host/Runner and product-domain stores do not own repository responsibility classification;
- copying Telegram's Python atlas/health runtime would create a source-specific parallel workflow implementation;
- a second atlas store, ownership graph, map owner, findings store or task queue would conflict with current canonical repository governance;
- treating the generated candidate skeleton as the canonical atlas would bypass independent source reading and explicit owner approval.

No novel shipping capability, new owner or ADR is required.

## Fabushi-equivalent contract

`exact source authority + approved responsibility text -> deterministic source-root-relative classification (exact > glob > prefix; equal-best => ambiguity) -> complete declared-scope coverage negatives -> optional candidate skeleton from exact source/health/dependency/history facts -> human semantic responsibility review + independent verification -> explicit existing-owner approval -> stable atlas identity consumed by ordinary quality/task evidence`

Missing approved atlas text leaves no canonical classifier. Candidate thresholds and metrics remain drafting evidence. Coverage failure and Git/process execution failure remain distinct. No atlas instrumentation result may approve itself, mutate product source, create a queue record or create a second ownership runtime.

## Fail-closed branches

- missing/empty approved atlas text cannot be presented as canonical classification;
- equal-best area claims remain ambiguous;
- unmapped files and unused configured patterns are non-passing coverage;
- generated candidate boundaries cannot become ownership by threshold or metric alone;
- stale/mutable source cannot back exact-source evidence;
- cross-repository histories cannot be presented as directly comparable co-change evidence;
- a Git/process failure cannot be rewritten as coverage success;
- skeleton generation cannot approve atlas text or satisfy owner approval;
- fallback directory measurement cannot become repository ownership;
- no source-specific atlas runtime, second ownership graph, map owner or task queue may be introduced.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-ATLAS-INSTRUMENT-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-ATLAS-CLASSIFICATION-001`, `ORA-TDRP-CODE-HEALTH-ATLAS-COVERAGE-001`, `ORA-TDRP-CODE-HEALTH-ATLAS-SKELETON-001`, `ORA-TDRP-CODE-HEALTH-ATLAS-CLI-001`
- Invariants: `INV-TDRP-CODE-HEALTH-ATLAS-SPECIFICITY-001`, `INV-TDRP-CODE-HEALTH-ATLAS-AMBIGUITY-001`, `INV-TDRP-CODE-HEALTH-ATLAS-NONAUTHORITATIVE-SKELETON-001`, `INV-TDRP-CODE-HEALTH-ATLAS-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-ATLAS-INSTRUMENT-001`
- Integration: `INT-TDRP-CODE-HEALTH-ATLAS-CONSUMERS-001`
- Fault controls: `FAULT-TDRP-CODE-HEALTH-ATLAS-MISSING-001`, `FAULT-TDRP-CODE-HEALTH-ATLAS-AMBIGUOUS-001`, `FAULT-TDRP-CODE-HEALTH-ATLAS-GIT-FAILURE-001`
- Security control: `SEC-TDRP-CODE-HEALTH-ATLAS-AUTHORITY-BYPASS-001`

## Verification disposition

This source entry is repository responsibility/quality/task-evidence tooling and adds no shipping runtime. Exact-head verification remains the project-required GitHub Actions pair on one immutable PR HEAD.

TDRP Revision 9 source authority must remain fail-closed in Accepted mode while `upstream.accepted=false`, preserve the complete recursive authority proof (12 root shards, 24 child shards, 16 next shards, 202->203 fixed point with zero unresolved gitlinks, reachability/root-candidate accounting, root source archives 2/94,443/582 and Boost 582/582/open=0), and emit a fresh authority artifact bound to the final PR head.

Rust desktop runtime on that same head must retain renderer, platform-worker and rust-host terminal success and continue to execute the native messaging manifest at `native/mahayana-messaging/Cargo.toml`.

Only after both campaigns are terminal-success on the same exact head, PR/main/upstream/entry24 are reread unchanged and GitHub reports that exact head mergeable may this one source entry merge. Full TDRP source closure remains open after this entry at unknown=16,178, unread=16,151, omitted=0.
