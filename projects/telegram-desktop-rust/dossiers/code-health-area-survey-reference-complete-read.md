# Complete read — .agents/skills/code-health/references/survey.md

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a`

## Inventory identity

The write lease for this closure is Fabushi `main@fb4d62d00ff63c36fcf5e422849e2bf8512d608d`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. The immediately preceding exact-head source-authority campaign is run `39348980018`; fresh authority artifact `12232346825` has digest `sha256:43adb3229ddfe89a0b49d67a1ddb39da52de489532d92bcc178719b008f582f3` and is bound to validation head `edfc9147b8437a804ba74af4805097f32dae85a1`. That validation head merged without content drift as Fabushi main `fb4d62d00ff63c36fcf5e422849e2bf8512d608d`. Its recursive authority contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 23 as this exact path/blob; entry 22 is `references/maps.md`.

Immediately before this write the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,180, unread=16,153, omitted=0. This closure accounts only entry 23. The skill, maps/atlas/classes references, scripts and task-routing callers read below remain separately inventoried unless already represented by their own independent rows.

## Complete source responsibility

`survey.md` defines a deep, agent-driven review of one responsibility area. Its purpose is not to rerun scanner metrics and not to implement cleanup; it converts exact-source reading into an evidence-backed area understanding, a draft map, ranked maintainability proposals and separately reported defects.

The work is intentionally expensive and therefore targeted at structural cost: high hotspot, high agent hours per task, or a large feature about to land. Those signals select where human/agent reading is worth spending; they do not determine the survey conclusions.

### Inputs

The survey consumes four authoritative inputs:

1. `health.py area <area>`, which already supplies files, sizes, churn, classes, long functions, clones, layering, unused candidates, build reach and agent cost. The survey must not manufacture a second measurement truth for facts this report already owns.
2. The previous survey and current area map, when present, so structural history and approved placement are not rediscovered from surrounding drift.
3. Open ledger findings for the area, preserving stable finding identity and unresolved state.
4. The exact source at the scanned commit. Every lens reads the detached worktree printed by `health.py worktree`, never the mutable source checkout.

The portable responsibility is exact-authority source review with explicit prior-state inputs, not Telegram's `health_state` directory or Python implementation.

### Independent lenses

The source requires independent reviewers that read code directly and do not see one another's reports:

- **Model and responsibilities** identifies what large files/classes actually own by line range, hidden state machines encoded in flags, and domain logic stranded in presentation containers.
- **Duplication** finds repeated logic inside the area and against existing shared mechanisms, preserving evidence for both sides.
- **Coupling and build** examines dependency direction, includes crossing targets, headers with excessive reach, and global/static state.
- **Testability** distinguishes pure logic already suitable for headless checks from seams that block testing and identifies the cheapest first characterization target.
- **Defects in passing** records suspected product bugs separately; they become ordinary defects, never evidence that a behavior-preserving cleanup task is complete.

Independence is a correctness property: one lens cannot turn another lens's conclusion into apparent corroboration.

### Separate verification

Before a claim is written into a proposal or map, a separate verifier re-checks it by re-running the count/search, re-reading the cited lines, or confirming an absence such as no callers. The upstream procedure prefers a different provider when available because same-model reviewers share blind spots, but the portable contract is independent re-verification rather than any particular provider.

A claim not rechecked is explicitly marked `(inferred)`. Agreement, confidence or scanner correlation cannot silently upgrade it to verified. Map/proposal evidence therefore has two traceable stages: originating lens evidence and independent verifier disposition.

### Outputs and side-effect boundary

The survey persists four distinct outputs:

- `survey.md`: responsibility/module understanding, hotspots, duplication, coupling, testability, leave-alone guidance and a dependency-ordered Q0–Q5 plan with tests before semantic reshapes;
- `map.md`: only a draft or amendment under the separate `maps.md` owner-approval lifecycle;
- `proposals.md`: at most ten bounded maintainability task proposals in the code-health proposal format;
- `defects.md`: defects noticed during reading, each with evidence as an ordinary task request.

The last sentence is a hard side-effect boundary: **a survey changes no source code and creates no queue records itself**. Publishing the evidence bundle is not source implementation, map approval or task routing. A later owner decision may approve a map or route allowed proposals/defects through the ordinary task lifecycle, but the survey cannot self-approve or self-route.

## Implementation and caller relationships read

The following exact upstream relationships were read to separate the portable survey responsibility from Telegram-specific instruments:

- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` selects at most one due area survey in the scheduled quality cycle, or accepts an explicitly named area; it requires surveys/audits to read the scanned commit worktree, publishes survey evidence to health state, and keeps routing as a separate authorized owner decision.
- `.agents/skills/code-health/references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f` consumes survey output only as a draft/amendment; the declared owner must approve or adjust it before it becomes repository instruction.
- `.agents/skills/code-health/references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` supplies stable responsibility-area identity so surveys follow product responsibility rather than directory depth.
- `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` defines the Q0–Q5 plan/proof semantics used to rank proposals; Q0–Q2 may later use the mechanical lane, while Q3–Q5 retain stronger pipeline/precondition rules.
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` supplies exact-source scan state, `worktree` and `area` observations. It has no autonomous `survey` command: survey orchestration is agent review around these exact-source instruments, which is additional evidence that copying a Telegram survey runtime would be the wrong owner.
- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` supplies stable findings and area measurements, including map/atlas findings; surveys consume those facts rather than replacing their measurement owner.
- `.agents/skills/code-health/scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` classifies responsibility areas and skeleton facts; it is not the semantic survey verifier.
- `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` exposes `survey [area]` as an invocation mode and `scan route` as the scheduled mode. The command explicitly routes proposals only when routing is requested; command syntax is an adapter, not ownership.
- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` is another native invocation surface pointing at the same skill contract.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, `pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` and `phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` consume an ordinary task only after proposal routing. They own implementation/proof/review lanes; a survey never calls them to mutate source directly.
- `.agents/skills/continue/SKILL.md` and the ordinary inbox/task routing lifecycle may schedule already-created tasks; they do not convert survey completion into a queue record by implication.
- upstream root `AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086` makes atlas/maps repository placement authority and keeps conversational work/task-queue boundaries explicit.

Thus the scheduled/task chain is: exact scan -> optional deep survey evidence -> owner/trust routing decision -> ordinary task record -> perform-task/continue execution. Survey completion alone stops before the queue boundary.

## Existing-owner-first resolution

Current Fabushi already has a source-neutral owner for every applicable responsibility:

1. root `AGENTS.md` owns exact-authority repository work, responsibility/area placement, quality-health governance and the ordinary task/review lifecycle;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns independent verification, negative controls and class-proportional qualification;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns exact-source provenance, lens/claim evidence, map approval provenance and task-routing receipts;
4. the ordinary Fabushi task/review/evidence lifecycle owns queue creation, implementation, conflict handling and acceptance.

The existing contracts already summarized hot-area survey intent, but did not yet make all entry23 boundaries independently auditable: the exact input set, lens isolation, separate claim verifier, inferred/verified distinction, four-output separation, map-draft approval boundary, proposal/defect separation, and the explicit no-source/no-queue side effect. This closure extends those existing owners only.

Rejected alternatives:

- shipping Agent/Coordinator/Host/Runner are product execution owners and must not become repository survey engines;
- product stores, UI roots, packaging/updater and release owners consume quality evidence but do not own source-review lenses;
- copying Telegram's code-health skill, health state, detached-worktree scripts or provider prompts would create a source-specific parallel workflow runtime;
- a new survey store, map owner, findings ledger or task queue would duplicate the canonical repository responsibility/quality/task-evidence lifecycle.

No novel shipping capability, new owner or ADR is required.

## Fabushi-equivalent contract

`fresh exact scan/area identity -> gather canonical area report + prior survey/map + open findings -> independent direct-source lenses -> separate verifier rechecks every map/proposal-driving claim -> mark unrechecked claims inferred -> emit survey narrative + draft map + bounded proposals + ordinary defects as separate evidence -> stop with zero source changes and zero queue records -> existing owner may later approve map or route requests under ordinary trust/conflict/proof gates`

## Fail-closed branches

- stale or mutable source cannot back a passing survey;
- scanner metrics may prioritize/decorate a claim but cannot replace direct responsibility reading;
- one lens cannot reuse another lens's conclusion as independent evidence;
- an unrechecked claim cannot be labeled verified or drive an authoritative map/proposal without its inferred status being visible;
- a survey cannot self-approve its draft map or semantic invariants;
- proposals cannot become queue records merely because the survey emitted them;
- defects cannot be hidden inside behavior-preserving maintainability work;
- the survey cannot change source code while claiming to remain observation-only;
- survey/map/proposal evidence cannot substitute for ordinary product, security, build, runtime, packaging or release proof;
- no source-specific survey runtime, second responsibility graph, map store, findings store or task queue may be introduced.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-AREA-SURVEY-001`
- Oracles: `ORA-TDRP-AREA-SURVEY-INPUTS-001`, `ORA-TDRP-AREA-SURVEY-INDEPENDENT-LENSES-001`, `ORA-TDRP-AREA-SURVEY-VERIFICATION-001`, `ORA-TDRP-AREA-SURVEY-OUTPUT-SEPARATION-001`
- Invariants: `INV-TDRP-AREA-SURVEY-EXACT-AUTHORITY-001`, `INV-TDRP-AREA-SURVEY-INDEPENDENT-REVIEW-001`, `INV-TDRP-AREA-SURVEY-INFERRED-LABEL-001`, `INV-TDRP-AREA-SURVEY-NO-SOURCE-MUTATION-001`, `INV-TDRP-AREA-SURVEY-NO-DIRECT-QUEUE-001`, `INV-TDRP-AREA-SURVEY-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-AREA-SURVEY-001`
- Integration: `INT-TDRP-AREA-SURVEY-MAP-AND-TASK-BOUNDARY-001`
- Fault controls: `FAULT-TDRP-AREA-SURVEY-STALE-SOURCE-001`, `FAULT-TDRP-AREA-SURVEY-LENS-CONTAMINATION-001`, `FAULT-TDRP-AREA-SURVEY-UNVERIFIED-CLAIM-001`, `FAULT-TDRP-AREA-SURVEY-DIRECT-QUEUE-001`
- Security control: `SEC-TDRP-AREA-SURVEY-EVIDENCE-BYPASS-001`

## Verification disposition

This source entry is repository responsibility/quality/task-evidence governance only and adds no shipping runtime. Exact-head verification remains the project-required GitHub Actions pair on one immutable PR HEAD.

TDRP Revision 9 source authority must remain fail-closed in Accepted mode while `upstream.accepted=false`, preserve the complete recursive authority proof (12 root shards, 24 child shards, 16 next shards, 202->203 fixed point with zero unresolved gitlinks, reachability/root-candidate accounting, root source archives 2/94,443/582 and Boost 582/582/open=0), and emit a fresh authority artifact bound to the final PR head.

Rust desktop runtime on that same head must retain renderer, platform-worker and rust-host terminal success and continue to execute the native messaging manifest at `native/mahayana-messaging/Cargo.toml`.

Only after both campaigns are terminal-success on the same exact head, PR/main/upstream/entry23 are reread unchanged and GitHub reports that exact head mergeable may this one source entry merge. Full TDRP source closure remains open after this entry at unknown=16,179, unread=16,152, omitted=0.
