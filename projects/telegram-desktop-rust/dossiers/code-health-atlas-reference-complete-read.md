# Complete read — .agents/skills/code-health/references/atlas.md

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0`

## Inventory identity

The immediately prior merged exact-head source-authority run `39341704852` produced fresh authority artifact `12229022687` for head `e49174f77ac651cd66a7136db6e16e305d519566` (digest `sha256:cc0861876d3421e085ea6c09c32b53d871f6b888de272b78bbd558fca5025d3f`). That artifact identifies actual 1-based recursive array entry 20 as `.agents/skills/code-health/references/atlas.md`, blob `7f43dd75fcfb670c327bc3e71c1de0589e7506f0`. Before this write, Fabushi main is exactly `5d445687a2ae58a38f7934a9d1dc68c11f908ca3`, upstream `dev` is exactly `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, and the merged parity ledger has zero rows for this exact path/blob.

## Complete source responsibility

The reference defines the whole-tree source atlas as a responsibility map, not a directory inventory. The canonical atlas lives at `Telegram/SourceFiles/AGENTS.md`; it gives every area one clear responsibility and is the nearest placement map for code that has no deeper area map. Health metrics, agent cost, proposals and surveys use these responsibility identities so they follow product boundaries instead of arbitrary directory depth.

Its schema is explicit:

- area names are short, unique, stable slugs; renaming resets ledger history;
- paths are relative to `Telegram/`;
- exact-file patterns outrank globs, globs outrank directory prefixes, and longer patterns win within a kind;
- every in-scope source file must resolve to exactly one area;
- layers order foundations through model/network/storage/UI/features/platform/tooling/tests so dependency direction is visible;
- known boundary violations are named instead of widening responsibility;
- an area that is too large or mixes unrelated responsibilities is split, while tiny directories join the responsibility they work with.

## Generation, verification and upkeep

The draft lifecycle binds to a fresh exact source authority through `health.py atlas --ref origin/dev --fetch --state <health_state>`. It writes `skeleton.md` and `skeleton.json` under an exact-commit atlas evidence directory. Candidate boundaries are informed by directory/line size, largest files, naming groups, CMake target, dependency direction, co-change, churn, agent cost and tested lines.

Those measurements do not decide ownership. Independent readers describe groups of candidate areas by reading the skeleton, headers and largest implementation openings; consolidation accepts or rejects proposed split/merge/move judgments against the source; a separate verifier samples responsibility claims against code. `health.py atlas --check <draft>` is fail-closed on unmapped files, equal-specificity ambiguity or unused path patterns. Draft, verifier report and judgment calls are published for owner review. Only text approved by the owner becomes `Telegram/SourceFiles/AGENTS.md`.

Upkeep is narrow: scans surface unmapped files and stale patterns; a new directory or reshaped area proposes the smallest amendment for owner approval rather than rewriting the map. Deeper area maps must agree with the atlas.

## Implementation relationships read

`.agents/skills/code-health/scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` implements the upstream instrument: it parses headings and `Paths:` patterns, classifies exact/glob/prefix specificity, checks unmapped/ambiguous/unused patterns, derives candidate folders using 30K/3K line heuristics, and builds skeleton rows from file metrics, include edges, co-change, task cost and tested lines. These mechanics explain the reference contract but are not copied into Fabushi.

`.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` wires the `atlas` command. Its check branch returns non-zero for coverage failures; its draft branch builds snapshot/include-graph evidence and writes exact-commit skeleton artifacts. Durable scan state separately requires an exact `--ref`.

`.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` loads the approved atlas to name every file/area, includes `atlas.check` results in the scan snapshot, and emits stable `stale-atlas`/unmapped findings rather than treating the map as decoration. `.agents/skills/code-health/scripts/health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` covers specificity, unmapped/unused reporting, scan naming, stale-atlas findings and large-directory candidate splitting.

The complete `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` invokes the atlas when the owner asks or no canonical atlas exists, then makes scan/survey/proposal behavior consume it. `.agents/skills/code-health/references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` deep-reads one atlas area from the exact scanned worktree and drafts its deeper map. `maps.md@89dd78f48283b15d6ad959b2d450895258a0980f` makes each area map refine the atlas responsibility/placement boundary and requires owner approval for invariants/direction. `classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` makes task placement follow the nearest map and gates Q4/Q5 on approved maps/invariants.

The scheduled invocation remains `scan route` through `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`; that scan consumes the approved atlas via report/snapshot rather than regenerating ownership automatically. Ordinary perform-task verification consumes the resulting map/placement and quality-ratchet rules through the existing task/review/evidence lifecycle. Upstream root `AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086` likewise tells developers to read the atlas entry and nearest area map before placement.

## Existing-owner-first resolution

Current Fabushi already has the applicable source-neutral owner:

1. root `AGENTS.md` owns repository development/quality governance;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` already qualifies atlas specificity, one-area coverage, stale patterns, responsibility boundaries and surveys;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` already requires exact revision, ownership coverage, split/merge facts, verifier disposition and owner approval;
4. the ordinary task/review/evidence lifecycle owns placement, review, routing and acceptance.

This closure therefore extends those same owners with the missing stable-identity and draft/verify/approve/amend lifecycle details. It does not add an atlas service, scanner, Telegram adapter, repository ownership database, product runtime, second task queue or second product owner.

Rejected alternatives:

- shipping Agent/Coordinator/Host/Runner are product execution owners, not repository source placement governance;
- copying `atlas.py`/Telegram code-health machinery would duplicate upstream instrumentation rather than rewrite an applicable product responsibility;
- a second ownership graph would conflict with current canonical repository and product owner-first rules.

No novel shipping capability or ADR is required.

## Fabushi-equivalent contract

`fresh exact source -> evidence-backed candidate skeleton -> independent responsibility read -> consolidate split/merge/layer judgments -> independent verifier + zero unmapped/ambiguous/unused patterns -> explicit owner approval -> canonical atlas -> scans/surveys/maps/tasks consume stable area identity -> narrow source-drift amendment -> re-approval`

Fail-closed branches:

- mutable/stale authority cannot become canonical atlas evidence;
- generated metrics/thresholds cannot decide semantic responsibility by themselves;
- unmapped, ambiguous or stale patterns cannot pass;
- unapproved drafts cannot become repository instruction;
- boundary violations are recorded, not normalized by widening an area;
- atlas/map contradictions remain non-passing;
- renames record an identity reset instead of silently carrying history;
- atlas metadata cannot become a second scanner, task owner or shipping runtime.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-RESPONSIBILITY-ATLAS-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-ATLAS-COVERAGE-001`, `ORA-TDRP-CODE-HEALTH-ATLAS-OWNER-APPROVAL-001`, `ORA-TDRP-CODE-HEALTH-ATLAS-LIFECYCLE-001`
- Invariants: `INV-TDRP-CODE-HEALTH-ATLAS-ONE-AREA-001`, `INV-TDRP-CODE-HEALTH-ATLAS-STABLE-IDENTITY-001`, `INV-TDRP-CODE-HEALTH-ATLAS-EXACT-AUTHORITY-001`, `INV-TDRP-CODE-HEALTH-ATLAS-NO-SECOND-OWNER-001`

## Verification disposition

This closure changes only repository governance/evidence contracts plus the independent migration ledger/dossier. It adds no shipping runtime. Exact-head acceptance remains the same required GitHub Actions pair: TDRP Revision 9 source authority and Rust desktop runtime on one final PR HEAD. Source-authority must remain Accepted fail-closed while full baseline closure is open, reproduce 12 root / 24 child / 16 next shards, reach the 202 -> 203 fixed point with zero unresolved gitlink authorities, preserve reachability/root-candidate and root-archive evidence, and keep Boost 582/582/open=0. Rust runtime must preserve renderer/platform-worker/rust-host and `native/mahayana-messaging/Cargo.toml` composition.

After this one independent source entry, counters become recursive=16,202, unknown=16,182, unread=16,155, omitted=0. Full source closure remains open.
