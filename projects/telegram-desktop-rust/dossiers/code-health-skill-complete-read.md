# Complete read — .agents/skills/code-health/SKILL.md

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097`

## Inventory identity

GitHub Actions run `39337255014`, source-authority artifact `12227067093`, digest `sha256:fcc1d9818c0f1f7c98d1376f194364169964cc5a65d0df3e7ee57645eda11227`, produced the most recent complete recursive authority artifact available before this write. Its `upstream-recursive-inventory.json` has 16,202 non-directory entries and directly identifies actual 1-based array entry 18 as `.agents/skills/code-health/SKILL.md`, object `6885938b3d186b55f5d69cb8b9b9a96bbca0e097`, size 12,726 bytes. Current Fabushi main `d8cafdccdf791208f1f604e991d06a4d378733c9` still has no parity-ledger row for that exact path/blob, while upstream `dev` still resolves to `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`.

Entry 19 is independently `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30`. The references and scripts read below are dependencies needed to understand entry18, but remain independent recursive inventory entries and are not counted closed by this dossier.

## Complete source read

The complete 12,726-byte skill was read. It is repository maintainability and task-generation governance, not Telegram product/runtime code.

Independent responsibilities carried by this entry are:

- run measurement against a freshly resolved exact `origin/dev` authority; a working-tree scan may be inspected but cannot update the durable findings ledger;
- persist a source-neutral health state containing stable findings, first-seen/resolved/ignored lifecycle, history, per-run reports, surveys, atlas drafts and audits;
- use measurements such as file/class/function size, churn/hotspots, duplicate logic, unused candidates, include/build reach, target layering, headless-test reach and map staleness to select work rather than to declare success;
- maintain a responsibility-oriented source atlas and narrower area maps, with exact/glob/prefix precedence, owner-approved placement/invariants and stale/unmapped detection;
- choose hot areas for deep independent surveys, verify claims before map/proposal publication, and keep incidental product defects separate from maintainability cleanup;
- apply a proportional per-change ratchet and finding verification before review, including cross-target dependencies, oversized growth, copy-paste, heavier headers/build reach, stranded definitions, last-use leftovers and move-only evidence;
- classify maintainability tasks Q0 delete, Q1 move, Q2 duplicate merge, Q3 characterization tests, Q4 semantic reshape and Q5 redesign, with progressively stronger proof and map/invariant preconditions;
- generate only bounded, non-conflicting ordinary task proposals; respect owner approval/trust and active-path conflicts; do not edit product code during a scan;
- audit prior landed code-health work and suspend automated routing for a class+area when an audit or later regression invalidates trust;
- report regressions/trends/coverage while treating health scores and scanner output as prioritization evidence, never product acceptance.

## References and script semantics read for entry18

All four normative references were read:

- `references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` defines the responsibility atlas, path specificity, candidate-area skeleton, independent description/consolidation/verification, owner approval and upkeep.
- `references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f` defines Modules/Placement/Invariants/Leave alone/Direction, dated approved invariants and narrow amendment lifecycle.
- `references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` defines independent model/duplication/coupling/testability/defect lenses, re-verification, ranked proposals and separate defect output.
- `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` defines Q0–Q5, behavior-preserving bounds, proportional proof, mutation-sensitive Q3 tests, Q4/Q5 preconditions and routing trust.

The complete script set was also read to disambiguate the skill's contracts:

- `health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` enforces exact-ref durable scans, once-per-UTC-day state writes unless forced, worktree/atlas/area/check/moves/verify modes and atomic ledger/history publication.
- `sources.py@688aa05c4223d0d45b8572cff89fbc5148254844` reads exact Git objects and pinned `lib_*` authorities without treating the mutable source checkout as canonical.
- `report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` assembles snapshot metrics and stable finding identities, carries `first_seen`, and marks vanished known findings resolved while retaining recent history.
- `review.py@070b55553052267e8fbc12d70985b447b24bbb02` owns the base-relative ratchet, platform/style/language stranded-orphan detection and move-only proof.
- `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43`, `metrics.py@ebab29607cb52782899d5424636f0bd5e0c126cd`, `includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c`, `cmake_model.py@e40ec0fc152958f96bd58d795f70d1a3a24ab1eb`, `clones.py@b74e6e939a3ccaae894d1afa5978f422aec8ee65`, `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` and `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4` supply atlas matching/skeletons, measurements, include/build-cost reach, CMake ownership, normalized clone identity, churn/task cost and source parsing.
- `health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` is the qualification oracle for exact state publication, atlas matching, finding verification, ratchet failures, move-only proof, platform references, style/language last-use detection and related controls.

Those script/reference entries remain independently unread for inventory accounting after entry18 unless already represented by another exact row.

## Real scheduled callers and dependencies

The real scheduled and task callers were read:

- `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` routes `scan`, `survey`, `check` and explicit `route`; the scheduled invocation is `scan route`.
- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` supplies the native skill surface and explicitly asks for freshly fetched `origin/dev`.
- `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` provides `health-ensure` and `health-publish`: it preserves dirty/unpublished state, updates/synchronizes only when safe, constrains publication to the health state and handles publication conflicts explicitly.
- `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` runs `health.py check --base refs/ai-tasks/<TASK_ID>/base` before the normal review fanout and preserves the complete output as review-readable evidence.
- `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` uses `verify`, `check`, `moves`, build/test proof and bounded independent review for Q0–Q2.
- `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` selects that mechanical lane only for Q0–Q2 and keeps all other tasks in the ordinary adaptive evidence/review lifecycle.
- upstream root `AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086` makes the source atlas/nearest area map part of placement authority and points code-health governance at the shared skill.

There is no shipping messaging, UI, Coordinator, Host, Runner, Marketplace, Computer, Automations, packaging or updater caller for this skill.

## Existing-owner-first resolution

Current Fabushi already has the source-neutral owners required to absorb the applicable contract:

1. root `AGENTS.md` owns repository development lifecycle, verification, review and task boundaries;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns quality risk, qualification, proportional proof and negative controls;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns exact-revision evidence, durable ledgers and audit/provenance records;
4. the ordinary Fabushi task/review lifecycle remains the consumer for routed work rather than a second quality queue owner.

This closure extends those owners with fresh exact-revision health observation, stable finding lifecycle, atlas/map/survey qualification, proportional change ratchets, class-sensitive proof, bounded task generation and routing-audit suspension.

Rejected owners:

- shipping Agent/Coordinator/Host/Runner and product-domain stores do not own repository maintainability scans;
- release/packaging consumes quality evidence but does not own finding measurement or task generation;
- a copied Telegram `health.py`, code-health runtime, parallel findings service, or Telegram-branded quality owner would duplicate the existing source-neutral governance.

No novel shipping capability or ADR is required.

## Fabushi-equivalent contract

`fresh exact source authority -> immutable health observation -> stable finding identities/history -> responsibility atlas + hot-area evidence -> verified proposal or ordinary defect -> ordinary task lifecycle -> class-proportional ratchet/proof -> independent review/evidence -> landed result -> later audit/trend`

Fail-closed branches:

- mutable or stale authority cannot update canonical findings;
- an unavailable pinned source is named unobserved, not silently healthy;
- unknown finding identity is reread/reconciled, not called resolved;
- scan metrics and clean static output are not behavior or release acceptance;
- map/invariant-dependent work does not route without the declared approval;
- active-path conflict or task cap prevents routing without deleting the finding;
- audit/regression failure suspends affected automated routing without rewriting history;
- survey defects become ordinary defects rather than being folded into behavior-preserving cleanup.

The upstream Python/CMake parsing implementation and its queue layout are instruments. Fabushi absorbs the source-neutral quality-governance responsibilities into existing owners and continues to use GitHub Actions for executable verification.

## Traceability IDs

- Requirement: `TDRP-R9-CODE-HEALTH-GOVERNANCE-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-FRESH-AUTHORITY-001`, `ORA-TDRP-CODE-HEALTH-STABLE-FINDINGS-001`, `ORA-TDRP-CODE-HEALTH-RATCHET-001`
- Invariants: `INV-TDRP-CODE-HEALTH-EXACT-REF-001`, `INV-TDRP-CODE-HEALTH-FINDING-ID-STABLE-001`, `INV-TDRP-CODE-HEALTH-SCAN-NOT-SUCCESS-001`, `INV-TDRP-CODE-HEALTH-PROOF-PROPORTIONAL-001`, `INV-TDRP-CODE-HEALTH-ROUTE-TRUST-001`

## Verification disposition

This source-entry closure changes repository governance/evidence contracts only and adds no shipping runtime. Executable acceptance is the TDRP Revision 9 source-authority workflow plus Rust desktop runtime on one final PR HEAD. The source-authority run must continue to reject Accepted while the baseline is open and reproduce the current recursive/root/archive/reachability accounting; Rust runtime evidence must prove the existing renderer/platform-worker/rust-host/messaging composition remains unchanged.

After this independent source-entry closure the counters become recursive=16,202, unknown=16,184, unread=16,157, omitted=0. Full source closure and baseline acceptance remain fail-closed.
