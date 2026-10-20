# Complete read — .agents/skills/continue/references/consolidate-pending-tasks.md

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@78ceaa548ae0e055aa56b53fde22064d13f250ef`
Upstream tree: `76b967833b38eb7aa2cb4047008c07331390e61e`
Source: `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`

## Inventory identity

The live upstream `dev` ref was re-read immediately before this write and still resolves to `78ceaa548ae0e055aa56b53fde22064d13f250ef` / tree `76b967833b38eb7aa2cb4047008c07331390e61e`. PR #154 and branch `tdrp-r9-entry31-includes-r27-20261020` both still resolve to parent `423d90406948a0e8b06a5c17a8aece30ce687346` / tree `5ef0df15d0b8b68f4df7543d878c2486f7684734`.

Zero-based recursive authority index 37 is `.agents/skills/continue/references/consolidate-pending-tasks.md`, blob `bd301d93f8f670ad041b3db72330e17cbb50eb5c`, immediately after `.agents/skills/continue/agents/openai.yaml` index 36. The next strict entry is index 38, `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`.

Parent Rust evidence is successful: run `39492471969` at exact parent `423d90406948a0e8b06a5c17a8aece30ce687346` completed 6/6 jobs successfully with 79 successful steps and two expected platform-conditional skips. Artifact `12296766732`, `rust-messaging-responsibilities-423d90406948a0e8b06a5c17a8aece30ce687346`, is bound by GitHub metadata to the same head SHA and has digest `sha256:fa685dbeff5ec979fa07ad913e4c8cda01b0f8c1f1108c0b0f74e705b07b622a`.

Parent TDRP run `39492472036` is explicitly incomplete/failed evidence and is not accepted parent authority. Attempt 2 ended with 15 successful jobs, one failed job and seven dependency-skipped jobs. The failed job is `118870253577` (Recursive root authority shard 7); its live log records `git clone https://chromium.googlesource.com/breakpad/breakpad` failing with `HTTP 503`, exit 128. As a consequence the recursive root accounting, recursive child/next layers, fixed-point and recursive build-reachability gates were skipped.

The successful Source authority / recursive inventory job `118870251376` remains only partial supporting evidence. Its live log binds target `423d90406948a0e8b06a5c17a8aece30ce687346` to upstream/tree `78ceaa548ae0e055aa56b53fde22064d13f250ef` / `76b967833b38eb7aa2cb4047008c07331390e61e`, reports root entries=7012, root blobs=6679, direct gitlinks=36, recursive non-directory=16201, exact recursive count reconciliation and cppgir/expected-lite participation. Attempt-2 authority artifact `12296064250` has GitHub digest `sha256:37ac4fe26a6244d47c5e5b61c05b3bd2dc626a370aad34b19ce71b3e69ccb58e`.

The independent Boost job `118870320373` also succeeded and logged candidates=582, dispositioned=582, open=0 while keeping source_closure_ready=false. Artifact `12296446919` has digest `sha256:493aa3266201c07f240c79764e580bc0467bd614b195e972431a50b9dedf0c70`. These partial successes do not repair the failed TDRP run and do not provide exact-parent fixed-point unresolved=0 or recursive remaining-open reachability evidence.

## Complete source read

The reference defines one bounded repository-development optimization: after discovery routing creates follow-up tasks, a fresh non-delegating leaf may merge truly duplicative pristine pending tasks without changing product source or task semantics.

Its responsibility boundary and state machine are explicit:

1. It receives the source checkout, AI slot/routing worktree, checkout tag, one durable `work/consolidation-pending.md` marker and the effective ordered batch ids.
2. It may inspect only repository task/project/receipt state. It must not touch Telegram source, build, tests, inbox processing, claims, task execution, approval or blocking.
3. Inventory is bounded to unfinished task state. Approved terminal history is opaque to merge reasoning, so work does not grow with shipped history.
4. Eligibility is fail-closed: candidates must be pristine unclaimed `todo` tasks with null claim/phase fields and no tracked execution/evidence artifacts. Code-health work is never a consolidation candidate.
5. Mergeability requires the same project and scheduler-membership semantics, preservation of acceptance scope and the revert boundary, compatible inputs and a simulated acyclic dependency graph.
6. Before retirement, each source task's committed retained content is sealed with `task-content-digest`. Retirement removes only live state, preserves task/input history, and adds a durable `superseded.yaml` alias containing the replacement, receipt, type, project and content digest.
7. Replacement tasks are pristine unclaimed `todo`; external dependencies are the stable union after old-to-new rewriting. Eligible todo dependents and project indexes are rewritten atomically, while blocked/in-progress/claimed reverse dependents veto the merge.
8. The receipt preserves every old-to-new mapping, criterion accounting, dependency rewrite and changed path. Old receipts and retained source content are immutable.
9. Immediately before mutation the worker re-reads the snapshot. A race returns RACED with no writes. A no-merge outcome still replaces the pending marker with a durable complete marker so the optimization is not repeated after restart.
10. Publication is only through `workspace.py consolidate-publish` (and `--routing` in the scheduler lane). The helper stages only explicit paths, validates retained aliases and the complete dependency graph, rebases, revalidates and pushes with the repository publication boundary. A semantic conflict, dependency cycle, changed source content or unavailable publication leaves visible incomplete state; force-push is forbidden.

The file owns no user-facing Fabushi feature, product Task store, shipping Agent queue, messaging state, runtime scheduler, model prompt, network protocol or release verdict.

## Real callers, state machine and tests

The same upstream commit was re-read through the real caller chain:

- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` writes/recoveries durable pending markers, launches exactly one fresh consolidation worker, uses the current frozen batch, applies published mappings back to invocation-local batch ids, and treats repeated pre-commit races as non-blocking optimization misses rather than product-task failure.
- `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e` requires a fresh blocking worker that reads this full reference and never runs consolidation inside a performer or discovery-routing worker.
- `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` binds the same transaction to the routing worktree and allows it beside active task-local phase state while keeping at most one consolidation in flight.
- `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` is the durable implementation owner for `CONSOLIDATION_PENDING`, `CONSOLIDATION_COMPLETE`, committed-blob `retained_task_digest`, superseded aliases, alias-cycle resolution, pending-marker discovery, full dependency/superseded/split graph validation, consolidation-head revalidation, `task-content-digest` and `consolidate-publish`.
- `.agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b` directly covers durable alias resolution, dependency cycles, pending-marker inventory, generic revalidation of consolidation commits, no-merge durable completion, merged publication including directory deletion, alias/dependency validation, rejection of late retained-content changes, and rejection of racing dependents.

This traces the document to executable repository workflow behavior instead of treating the prose reference as a standalone runtime.

## Existing-owner-first resolution

Fabushi exact-head already owns the applicable source-neutral responsibility in repository governance:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` requires quality findings and transformations to use ordinary repository task/review/evidence lifecycle rather than a parallel queue and requires stale/partial/failed authority to remain explicit non-passing evidence.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` qualifies repository task classes, immutable exact-head proof and negative controls; test fixtures/helpers never become a second task queue or source authority.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` records exact-base task evidence, atomic publication outcome, routing/conflict negatives and explicit non-passing/unknown results for Git/process failure or incomplete proof.

Exact-head shipping candidates were inspected and rejected as semantic owners: `source/host/src/extensions/transcript/run_scheduler.rs@eb8994dc45b3722e8bfcc672e9e610c662429650` owns product User/Agent/Background run lanes, watchdog and settlement; `runner_prompt_glue.rs@fed229ea2c4226eba00466bd5413ec9b85c6aee5`, `system_prompt_assembly.rs@6d592794b835994a81a214049b9c5907df0cb4ff` and `turn_agent_composition.rs@eefde7f77856070e62e4f8dbca7cfb7f28da0a17` own shipping provider/prompt/tool-turn composition. They do not own repository-development task retirement, retained-content digests, dependency graph rewriting, routing-worktree publication or Git CAS evidence.

No new product owner, second Task store, second scheduler, Telegram-specific runtime or copied Python workflow is required.

## Fabushi-equivalent contract

`ordinary repository discovery/routing -> durable pending marker -> bounded exact-state scan of unfinished tasks -> fail-closed semantic merge decision -> preserved retained content + durable aliases + rewritten acyclic dependencies -> explicit receipt -> atomic repository publication/revalidation -> durable complete marker or visible race/block`

Fail-closed invariants:

- no semantic criterion, dependency, input or retained history may disappear through consolidation;
- no claimed, started, blocked-with-dependent-risk or otherwise non-pristine task may be merged;
- stale snapshots cannot publish; a race is visible and retry-bounded;
- partial/external CI failure stays incomplete evidence and cannot be promoted to accepted authority;
- shipping Task/RunScheduler/Automation/Workflow state must not be repurposed as repository-development queue truth;
- parent CI is historical/supporting only after a new mapping commit; exact-head verification is required again.

## Traceability

- Requirement: `TDRP-R9-CONTINUE-PENDING-CONSOLIDATION-001`
- Oracles: `ORA-TDRP-CONTINUE-CONSOLIDATION-ELIGIBILITY-001`, `ORA-TDRP-CONTINUE-CONSOLIDATION-TRACEABILITY-001`, `ORA-TDRP-CONTINUE-CONSOLIDATION-CAS-001`
- Invariants: `INV-TDRP-CONTINUE-CONSOLIDATION-NO-SEMANTIC-LOSS-001`, `INV-TDRP-CONTINUE-CONSOLIDATION-RETAINED-CONTENT-IMMUTABLE-001`, `INV-TDRP-CONTINUE-CONSOLIDATION-RACE-FAIL-VISIBLE-001`, `INV-TDRP-CONTINUE-CONSOLIDATION-NO-SHIPPING-QUEUE-COLLISION-001`

## Verification disposition

This closes only recursive index 37. Counters become recursive=16,201, unknown=16,161, unread=16,136, omitted=0. Baseline identity remains unaccepted and `source_closure_ready=false`.

The parent Rust run is successful supporting regression evidence. Parent TDRP run `39492472036` remains explicitly incomplete/failed because external Chromium googlesource HTTP 503 prevented shard 7 and therefore prevented exact-parent recursive fixed-point and remaining-open reachability gates. Its successful authority and Boost jobs are partial evidence only, never accepted parent authority.

Exact-head acceptance of this mapping requires fresh TDRP Revision 9 source-authority and Rust desktop runtime workflows on the new commit. A later TDRP run may restore the recursive-evidence threshold only if it actually proves fixed-point `unresolved=0`, all nested gitlinks, cppgir/expected-lite, exact recursive reconciliation, Boost `582/582/open=0`, and recursive remaining-open disposition/reachability. Full source closure remains prohibited until unknown=0, unread=0, omitted=0 and all remaining-open generated/resource/tool/acquisition disposition is closed.
