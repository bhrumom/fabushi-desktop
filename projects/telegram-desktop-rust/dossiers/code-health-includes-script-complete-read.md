# Code-health includes.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Fixed commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)
- Path: `.agents/skills/code-health/scripts/includes.py`
- Blob: `2c02b4e7eb821f123588e0a80056edef86fc084c`
- Size: 5,150 bytes

## Exact responsibility

This source is the repository-quality include graph and source-line build-cost proxy instrument. It owns no shipping product state and no configured-build truth. It resolves project-owned quoted C/C++ include/import and style-import edges; preserves unresolved counts; computes memoized cycle-safe closure and target PCH closure; derives TU non-PCH header-line weight, header reach/rebuild and total build-work proxies; and detects cross-target layering against transitive link dependencies.

Resolution order is local path, including-file source root, then other declared roots. Generated `styles/style_*.h` can map to the matching `.style` source. Translation units come from `cpp_text.is_tu`, and where build ownership exists units outside the model are excluded.

## Direct callers/consumers

- `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12`: constructs the graph, consumes costs/PCH/layering, projects weight/reach/rebuild, and emits heavy-unit/heavy-header findings.
- `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32`: resolves existing/new includes, detects new target crossings and estimates added header work from reach/closure/cost.
- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`: constructs the graph for atlas generation.
- `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43`: consumes graph edges for cross-area dependency counts.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794`: `GraphTests` cover target/PCH, upward layering, reach and weight.
- Adjacent providers: `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`, `cmake_model.py@e40ec0fc152958f96bd58d795f70d1a3a24ab1eb`, `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`.

No independent product/runtime consumer was found in the fixed-U code-health script set.

## Existing-owner-first

Map to the existing Fabushi repository quality/test-evidence governance in root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, and `quality/evidence-contract.md`. No platform-core production delta, second scanner/build model/findings/task/runtime owner, upstream Python port or ADR is required.

## Traceability

Requirement: `TDRP-R9-CODE-HEALTH-INCLUDE-GRAPH-BUILD-COST-001`.

Oracles: `ORA-TDRP-CODE-HEALTH-INCLUDE-RESOLUTION-001`, `ORA-TDRP-CODE-HEALTH-INCLUDE-CLOSURE-PCH-001`, `ORA-TDRP-CODE-HEALTH-INCLUDE-BUILD-COST-001`, `ORA-TDRP-CODE-HEALTH-INCLUDE-LAYERING-001`.

Contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-RESOLUTION-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-CLOSURE-PCH-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-STYLE-IMPORT-001`, `INT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-001`.

Stage A: `implemented / validation_status=not-run / accepted=false`.
