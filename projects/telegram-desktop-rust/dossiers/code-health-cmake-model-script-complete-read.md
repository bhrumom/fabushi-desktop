# Complete read — .agents/skills/code-health/scripts/cmake_model.py

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
Source: `.agents/skills/code-health/scripts/cmake_model.py@e40ec0fc152958f96bd58d795f70d1a3a24ab1eb`

## Inventory identity

The write lease for this closure is Fabushi `main@49ff58f5a93de8548acd0a576f597255eaaa8d56`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. Fresh authority artifact `12234149790` from the immediately preceding source-authority campaign is the candidate recursive authority input. The recursive authority contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 26 as this exact path/blob. Immediately before this write the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,177, unread=16,150, omitted=0. This closure accounts only entry 26.

## Complete source responsibility

`cmake_model.py` is a read-only, source-derived CMake topology model used by Telegram's repository quality tooling. It does not run CMake, configure a build directory, read a CMake cache or Ninja manifest, execute build commands, persist state, own findings or route tasks.

### Recognizer and normalization

- `CALL` recognizes only `nice_target_sources`, `target_link_libraries`, `target_precompile_headers`, `add_library` and `add_executable`, case-insensitively.
- `calls` removes hash comments, scans from each recognized call through balanced parentheses, and whitespace-splits the body. It is deliberately not a CMake evaluator: variables, conditions, helper-function semantics, quoting and generator expressions are not generally interpreted.
- `base_for` maps CMake files under `Telegram/lib_*` to that library root; all other recognized build files map source paths under `Telegram/SourceFiles`.
- `local_name` strips only `desktop-app::` and `tdesktop::` target namespaces. `PRIVATE`, `PUBLIC` and `INTERFACE` are scope tokens.
- `build` walks `.cmake` and `CMakeLists.txt` paths in sorted order, making the approximation deterministic for a fixed source snapshot.

### Target/source/link/PCH model

`CMakeModel` owns only in-memory derived facts: `file_target`, `sources`, `links`, `pch`, `executables`, `libraries` and a dependency-closure cache.

- `add_library` contributes a concrete library unless the token list contains `ALIAS`; `add_executable` contributes an executable.
- `target_link_libraries` records normalized link tokens after dropping scope tokens, command-line-like tokens beginning with a dash, variable tokens beginning with a dollar sign and quoted tokens.
- `target_precompile_headers` recognizes only header tokens containing the literal source-root pattern and records the derived header path for that target; later recognized PCH tokens replace earlier ones for the same target.
- `nice_target_sources` is accepted only when its source-root argument is the literal expected source token. Scope/variable tokens are skipped; each remaining token becomes a source path. `file_target` uses first-recognized assignment, so sorted CMake-file traversal makes duplicate membership deterministic.
- `target_of` is a direct file-to-target lookup.
- `dependencies` returns the target itself plus its transitive recognized link closure and caches the result; cycles terminate through the visited set.
- `tests` classifies executables whose target name contains lowercase `test`.
- `tested_files` includes sources compiled directly by each such test executable plus sources of directly linked project libraries whose normalized target begins `td_`. It deliberately does not infer transitive tested coverage or count unrelated toolkit/support libraries.

These are repository-quality approximations. A missing modeled edge is not proof that an evaluated CMake graph lacks that edge.

## Real consumers and routing relationship

- `scripts/includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c` consumes `file_target`, `target_of`, `pch` and `dependencies`. It limits translation units to modeled targets when mappings exist, subtracts target PCH closure from per-unit include weight, computes reach/rebuild/build-work estimates, and labels an include as a source-level layering violation when the included file's target is outside the source target's recognized transitive links.
- `scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` builds this model for each source snapshot, records target/tested facts per file, PCH-adjusted include cost and total build-work lines, and turns downstream heavy-unit/heavy-header/layering/untested-hot-area evidence into Q1/Q5/Q3 findings through the existing report owner.
- `scripts/review.py@070b55553052267e8fbc12d70985b447b24bbb02` builds the current source model and include graph. New cross-target includes become errors when no same target-pair violation already exists and warnings when the repository already contains such crossings; new header reach can also produce build-weight warnings. Review owns strictness and result semantics, not `cmake_model.py`.
- `scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` owns scan/check/area/verify/moves/atlas CLI, exact-ref durable scan rules and Git/process error handling. Its atlas command combines the canonical snapshot with an IncludeGraph built from `cmake_model` for candidate-skeleton evidence. `cmake_model.py` has no CLI or durable output.
- `scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` uses the report/include-derived target, tested-line and cross-area dependency facts only as candidate-skeleton measurements. Atlas owner approval remains separate.
- `scripts/health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` supplies a miniature CMake fixture and directly proves source target, PCH and dependency direction; adjacent graph tests prove upward-layering detection and PCH-adjusted reach/weight behavior.
- `scripts/clones.py@b74e6e939a3ccaae894d1afa5978f422aec8ee65` is an independent quality signal combined by report; it has no build-model ownership.
- `code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` treats include/build/test/layering facts as quality observations. `references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f`, `classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e`, `survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` and `atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` keep placement, Q-class proof, source-review and owner-approval boundaries outside the model.
- `perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` dispatches Q0-Q2 through `mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` and Q3-Q5 through `pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` / `phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba`. Mechanical proof may edit CMake source lists within task bounds, but source-derived model evidence never self-authorizes those edits.
- `process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db` and `continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` retain ordinary routing/scheduling. A scan proposal is routed through the existing task lifecycle; the model cannot create, claim or complete a task.

The real chain is: exact source snapshot -> bounded source CMake recognition -> approximate target/source/link/PCH/test facts -> include-cost/layering/test/atlas quality evidence -> existing findings/class/placement owner -> ordinary task routing and proportional proof. Configured build claims leave this chain and use Fabushi's existing configured-build graph provenance owner.

## Existing-owner-first resolution

Current Fabushi already has the applicable source-neutral owners:

1. root `AGENTS.md` owns exact-authority repository quality governance, responsibility/placement and ordinary task/review lifecycle;
2. the existing **Configured-build graph provenance evidence** contract owns stronger evaluated build-graph claims and prevents this approximation from becoming build authority;
3. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns qualification and negative controls;
4. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns reproducible exact-source and configured-build provenance boundaries;
5. the ordinary Fabushi task/review/evidence lifecycle owns any later quality change, approval, scheduling and acceptance.

The missing durable detail was the explicit source-derived build-model boundary: what may be recognized, deterministic normalization/closure semantics, how quality consumers may use it, what missing recognition cannot prove, and when evidence must escalate to configured-build graph provenance. This closure extends only those existing owners.

Rejected alternatives: shipping Agent/Coordinator/Host/Runner do not own repository CMake quality analysis; product-domain stores/UI roots/packaging/updater do not own source build topology; copying Telegram `cmake_model.py`/`includes.py`/`health.py` would create a source-specific parallel quality runtime; a new build-model findings store, ownership graph or task queue would duplicate canonical governance; treating the source parser as configured-build authority would weaken the existing stronger provenance contract.

No novel shipping capability, new owner or ADR is required.

## Fabushi-equivalent contract

`fresh exact source authority -> deterministic bounded declarative-build recognition -> normalized approximate target/source/link/PCH/executable/library/test facts -> source-level quality consumers for include cost/layering/test attribution/atlas drafting -> existing findings and class/placement owners -> ordinary task lifecycle; any evaluated configured-build claim -> existing configured-build graph provenance evidence instead`.

## Failure and authority boundaries

- stale, partial or mutable source cannot back durable source-model evidence;
- unsupported/unevaluated CMake syntax is an explicit limitation, never proof of absence;
- parser failure or malformed/incomplete input cannot be rewritten as a clean quality result;
- source-derived links cannot prove evaluated configured target membership, PCH consumption, link operands, compiler definitions or generated inputs;
- test-name/direct-link heuristics are coarse attribution and cannot prove behavioral test coverage;
- approximate include/build-work metrics select investigation and cannot prove release/build success;
- scan/report/review/atlas callers keep findings, strictness, owner approval and routing semantics;
- no Telegram Python quality runtime, second build-model/findings store, ownership graph or task queue may be introduced.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-SOURCE-BUILD-MODEL-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-CMAKE-RECOGNITION-001`, `ORA-TDRP-CODE-HEALTH-BUILD-NORMALIZATION-001`, `ORA-TDRP-CODE-HEALTH-BUILD-CONSUMERS-001`
- Invariants: `INV-TDRP-CODE-HEALTH-SOURCE-MODEL-NONCONFIGURED-001`, `INV-TDRP-CODE-HEALTH-BUILD-AUTHORITY-SEPARATION-001`, `INV-TDRP-CODE-HEALTH-BUILD-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-SOURCE-BUILD-MODEL-001`
- Integration: `INT-TDRP-CODE-HEALTH-SOURCE-BUILD-MODEL-CONSUMERS-001`
- Fault controls: `FAULT-TDRP-CODE-HEALTH-CMAKE-UNRECOGNIZED-001`, `FAULT-TDRP-CODE-HEALTH-CMAKE-PARSER-001`, `FAULT-TDRP-CODE-HEALTH-STALE-SOURCE-001`
- Security control: `SEC-TDRP-CODE-HEALTH-BUILD-AUTHORITY-BYPASS-001`

## Verification disposition

This source entry is repository quality/build-evidence tooling and adds no shipping runtime. Exact-head verification remains the project-required GitHub Actions pair on one immutable PR HEAD.

TDRP Revision 9 source authority must remain fail-closed in Accepted mode while `upstream.accepted=false`, preserve the complete recursive authority proof (12 root shards, 24 child shards, 16 next shards, 202->203 fixed point with zero unresolved gitlinks, reachability/root-candidate accounting, root source archives 2/94,443/582 and Boost 582/582/open=0), and emit a fresh authority artifact bound to the final PR head.

Rust desktop runtime on that same head must retain renderer, platform-worker and rust-host terminal success and continue to execute the native messaging manifest at `native/mahayana-messaging/Cargo.toml`.

Only after both campaigns are terminal-success on the same exact head, PR/main/upstream/entry26 are reread unchanged and GitHub reports that exact head mergeable may this one source entry merge. Full TDRP source closure remains open after this entry at unknown=16,176, unread=16,149, omitted=0.
