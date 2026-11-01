# Complete read — .agents/skills/code-health/scripts/clones.py

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
Source: `.agents/skills/code-health/scripts/clones.py@b74e6e939a3ccaae894d1afa5978f422aec8ee65`

## Inventory identity

The write lease for this closure is Fabushi `main@96de945ef88fe7e11e1579cc333e6eb723d078ee`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. Fresh authority artifact `12234032396` is the current source-authority input. The recursive authority contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 25 as this exact path/blob. Immediately before this write the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,178, unread=16,151, omitted=0. This closure accounts only entry 25.

## Complete source responsibility

`clones.py` is a source-neutral text-level duplicate-evidence library. It has no CLI, no persistence, no task routing, no refactoring authority and no product runtime.

### Source normalization and sequence construction

- `WINDOW=6`, `MIN_LINES=10`, and `MAX_BUCKET=6` are detector thresholds, not ownership policy.
- `Sequence(path, text)` enumerates `cpp_text.code_lines` with original 1-based line numbers, applies `cpp_text.normalize`, drops empty normalized lines, preserves the surviving normalized text and original line coordinates, and hashes each normalized line for candidate lookup.
- `sequences(files)` accepts only paths ending in the declared C/C++ translation-unit or header extensions from `cpp_text.EXTENSIONS_TU + EXTENSIONS_HEADER`.
- `cpp_text.code_lines` blanks block comments while preserving line count. `cpp_text.normalize` collapses whitespace and omits lines that carry no logic. Clone evidence therefore compares normalized logical lines while reporting original coordinates.

### Clone discovery

- `window_index` indexes every contiguous normalized window by the hash of its tuple and records path plus normalized offset.
- `find` ignores buckets with fewer than two positions and buckets larger than `MAX_BUCKET`, preventing very common windows from exploding into low-signal pair combinations.
- Candidate position pairs are canonicalized by sorting. Same-file candidates closer than one full window are skipped.
- Only a pair without a predecessor pair is treated as a run start. The detector extends through consecutive window matches to a maximal normalized run and computes `count = matched_windows + window - 1`.
- Runs below `MIN_LINES` are rejected.
- A same-file run that would overlap itself is clipped at the second start and rejected if the clipped result falls below the minimum.
- `declarations_only` rejects blocks where at least 80% of normalized lines look like declarations/interfaces rather than executable duplicated logic; pimpl/interface mirroring therefore does not become a clone finding.
- Clone identity is the first 12 hex characters of SHA-1 over the newline-joined normalized duplicated block. Output preserves normalized line count and both original source ranges, then sorts longest-first and by left path/start for deterministic presentation.

The hash/index stages are candidate acceleration only. A collision can at most introduce a candidate; maximal run construction still depends on the indexed normalized-line hashes. The result is heuristic quality evidence and never a semantic equivalence proof.

### Review-time copy detection

`copied_from(added, files, window=6, min_lines=8)` reuses exact-current source sequences and the same window index to detect added hunks that repeat existing code:

- each added hunk is normalized while retaining its original added-line numbers;
- empty normalized lines are removed;
- for each added window, the detector finds existing indexed positions and explicitly excludes the exact same path/line location;
- matching added indices are accumulated into one run beginning at the first source candidate;
- `flush` emits only runs with at least the review minimum, preserving added path/start/end/count and existing source path/start.

This is a review guard against introducing copy-paste, not a source transform. It reports the first source candidate for the matched run and leaves refactor choice to the existing review/task owner.

## Real consumers and routing relationship

- `report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` calls `clones.find(tree.files)` for every exact-source snapshot, adds each side's clone lines to the existing responsibility-area metrics, records total clone pairs/lines and raw clones, groups clone findings by stable content ID, scores them with churn and maps kind `clone` to class Q2.
- `review.py@070b55553052267e8fbc12d70985b447b24bbb02` builds changed-file added hunks and calls `clones.copied_from`. The existing health-check owner formats source provenance and chooses warning vs error according to strictness.
- `health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` owns scan/check/area/verify CLI boundaries. `area` renders clones touching an area; `verify` compares before/after canonical findings and may report a clone resolved or changed when a successor clone remains in the same files.
- `health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` directly proves repeated blocks are found once and mirrored declarations are excluded, and the wider check/verify tests exercise canonical command behavior.
- `code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` treats duplicated code as a measured quality signal and Q2 proposal input; scan state remains in the existing quality ledger.
- `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` defines Q2 as merging copies that are identical apart from names/literals and explicitly sends logic-divergent copies to Q4.
- `perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` owns the actual Q2 edit/proof/review lifecycle, including `health.py verify`, Debug build, check, bounded review and drop-on-nonmechanical behavior.
- atlas/maps/survey references consume duplication only as evidence within the existing responsibility/area lifecycle; survey duplication is an independent source-reading lens and can compare against existing mechanisms without granting `clones.py` ownership.
- command/OpenAI adapters only invoke the code-health skill; scheduled/continue/task routing remains outside this library.

Thus the real chain is: exact source snapshot -> normalized sequence/window evidence -> scan/report stable clone finding and/or review-time copied-from warning -> human/task-class judgment -> separately authorized Q2 mechanical task when applicable -> ordinary proof/review/acceptance.

## Existing-owner-first resolution

Current Fabushi already has the applicable source-neutral owner:

1. root `AGENTS.md` owns repository quality/task lifecycle and fail-closed evidence boundaries;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns qualification and negative controls;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns reproducible exact-source evidence;
4. the ordinary Fabushi task/review/evidence lifecycle owns any later deduplication change.

The applicable responsibility is duplication evidence, not the Telegram Python implementation. Copying `clones.py`, creating a second clone findings ledger, a duplicate ownership graph, or a clone-specific task queue would duplicate canonical governance. Shipping Agent/Coordinator/Host/Runner, product-domain stores, UI roots and packaging/updater owners do not own repository clone analysis.

No novel shipping capability, new owner or ADR is required.

## Fabushi-equivalent contract

`exact source authority -> source-kind filter -> comment/logic-free removal + whitespace normalization with original coordinates -> bounded window candidate index -> maximal non-overlapping substantive clone evidence -> stable content-derived identity -> existing scan/report ledger and area metrics`

and independently:

`exact current source + added hunks -> same normalization/window comparison excluding identical location -> thresholded copied-from provenance -> existing review/check strictness -> ordinary task/review lifecycle`.

Detector thresholds select investigation evidence. They do not prove semantic equivalence or authorize a merge. No detector result may edit source, route a task, approve Q2, create ownership, or replace exact-head acceptance.

## Fail-closed branches

- unsupported source kinds are outside this detector and cannot be claimed scanned;
- comments-only/logic-free/declaration-only similarity cannot be promoted to a substantive clone;
- same-file overlap is clipped, not double-counted;
- below-threshold and over-common buckets are detector exclusions, not proof of no duplication;
- review copy detection excludes the exact same source location;
- stale or partial source identity cannot back canonical clone evidence;
- a detector/process failure cannot be rewritten as zero clones;
- a detected clone cannot self-authorize Q2 or bypass behavioral review;
- no source-specific Python clone runtime, second findings ledger, ownership graph or task queue may be introduced.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-CLONE-EVIDENCE-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-CLONE-NORMALIZATION-001`, `ORA-TDRP-CODE-HEALTH-CLONE-MAXIMAL-001`, `ORA-TDRP-CODE-HEALTH-COPIED-FROM-001`
- Invariants: `INV-TDRP-CODE-HEALTH-CLONE-PROVENANCE-001`, `INV-TDRP-CODE-HEALTH-CLONE-NONAUTHORITATIVE-001`, `INV-TDRP-CODE-HEALTH-CLONE-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-CLONE-EVIDENCE-001`
- Integration: `INT-TDRP-CODE-HEALTH-CLONE-CONSUMERS-001`
- Fault controls: `FAULT-TDRP-CODE-HEALTH-CLONE-STALE-SOURCE-001`, `FAULT-TDRP-CODE-HEALTH-CLONE-PROCESS-001`
- Security control: `SEC-TDRP-CODE-HEALTH-CLONE-AUTHORITY-BYPASS-001`

## Verification disposition

This source entry is repository quality/task-evidence tooling and adds no shipping runtime. Exact-head verification remains the project-required GitHub Actions pair on one immutable PR HEAD.

TDRP Revision 9 source authority must remain fail-closed in Accepted mode while `upstream.accepted=false`, preserve the complete recursive authority proof (12 root shards, 24 child shards, 16 next shards, 202->203 fixed point with zero unresolved gitlinks, reachability/root-candidate accounting, root source archives 2/94,443/582 and Boost 582/582/open=0), and emit a fresh authority artifact bound to the final PR head.

Rust desktop runtime on that same head must retain renderer, platform-worker and rust-host terminal success and continue to execute the native messaging manifest at `native/mahayana-messaging/Cargo.toml`.

Only after both campaigns are terminal-success on the same exact head, PR/main/upstream/entry25 are reread unchanged and GitHub reports that exact head mergeable may this one source entry merge. Full TDRP source closure remains open after this entry at unknown=16,177, unread=16,150, omitted=0.
