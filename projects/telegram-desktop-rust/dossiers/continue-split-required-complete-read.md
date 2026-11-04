# Complete read — fixed-U continue split-required routing

## Authority snapshot

- Desktop checkpoint/live head before spec publication: `7700fd41ec787c08bb945a6f6c3660f9cde22930`.
- Desktop main reread this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` in `bhrumom/fabushi-platform-core`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U recursive root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, reread `truncated=false` with 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`.
- Exact blob read completely: 5,813 characters / 112 lines; recursive tree index 54.

## Responsibility classification

This source is repository-development process governance for rescoping one already-published oversized task. It is not Telegram shipping behavior, not product Tasks/Automations, and not a second scheduler/runtime owner.

Existing-owner-first result:

- existing owner: root `AGENTS.md` repository task/process authority governance;
- qualification owner: `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- evidence owner: `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- same responsibility: `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`;
- no new responsibility, ADR, platform-core runtime, Telegram-specific queue, Product Tasks/Automations owner, or shipping product delta.

Rejected owners remain Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, copied upstream scheduler code, and any second task/receipt/ownership store because none owns canonical repository-development queue authority.

## Split eligibility and decomposition

The transaction runs only after the performer has canonically published `status: split-required`. It rereads the complete source task, project context, dependencies, context/plan, split proposal/result and carried-work seal. The proposal is input, not titles to copy mechanically.

A replacement boundary must be independently shippable and independently testable with one useful product outcome and a direct acceptance oracle. Network, persistence, engine, ownership/lifetime, lifecycle, UI, platform and evidence seams are valid only when they can stand independently; inseparable APIs and their only callers stay together. A final integration task exists only for integration behavior not already proved by component tasks.

The source project is preserved by default. A successor can leave it only when coherent and testable with the project changes absent. Existing source dependencies remain wherever their shipped code is required, and successor dependency order follows actual code/behavior prerequisites.

## Retained implementation carrier

`work/carried-work.json` seals whether source implementation exists and the owned worktree state.

When `implementation: retained`:

- exactly one first successor is the implementation carrier;
- the complete retained diff must fit that successor's coherent review/test boundary;
- publication copies the source `owned-paths.txt`, durable checkout ownership metadata and `carried_from` lineage to the carrier while leaving it `todo`;
- later successors depend on the carrier whenever they consume retained code;
- retained phase conclusions are not copied as approval;
- the later canonical `Start` revalidates the retained worktree seal, transfers source refs to the carrier, then removes the obsolete source refs.

When `implementation: none`, no carrier exists, all replacements remain unclaimed, and obsolete source refs can be retired after split publication.

If no coherent carrier can own retained implementation, the transaction stops with the source task, source refs and implementation untouched. Resetting, stashing, discarding, hiding the patch or spreading it across unreviewed successors is forbidden.

## Replacement state and dependency rewrite

The transaction creates at least two dated replacement tasks. Before publication they are pristine unclaimed `todo`, `type: implement`, name the split receipt, and have no model/carried-from value. The publication helper alone performs the carrier ownership handoff.

The receipt accounts for the split reason, every ordered successor and oracle, dependencies, carrier or none, every original acceptance criterion and retained source path, and every changed dependent/project link.

The source project index replaces the retired source link with successors in dependency/routing order. Every live task dependency on the source is rewritten to the exact successor or successors providing the needed behavior. The retired source id remains in neither a live dependency nor a project index.

The source directory itself remains retained history. Publication removes only its runnable `state.yaml` and writes a sealed `split.yaml` carrying the ordered successors, optional carrier, receipt, original task metadata and retained-content digest. Split/superseded resolution must be cycle-safe and terminate at live canonical work.

## Atomic publication and recovery

The split worker does not edit/reset/stash/commit/build/test Telegram source. It publishes AI task/project/dependency/receipt state through the explicit-path `split-publish` transaction.

The fixed-U workspace helper requires:

- a canonical split-required source owned by the checkout with performer identity present;
- at least two unique successor ids and a first-position carrier when retained work exists;
- explicit path coverage for source, every successor, receipt and every changed project/dependent;
- no worker-authored source-task content changes before retirement;
- a matching carried-work seal and exact carrier/no-carrier disposition;
- pristine successor state before handoff;
- source retirement limited to `state.yaml -> split.yaml`;
- receipt coverage, no stale project links, complete dependency/superseded/split graph validity and retained-content digest validity.

The commit is `Split <source-id>` and publication is compare-and-swap. A no-carrier split retires obsolete source refs only after successful publication. A carrier split deliberately retains those refs until the carrier's canonical Start validates the seal and transfers them.

An ordinary publication race is retried through refresh plus the same semantic validator. A semantic conflict, unavailable remote, changed worktree seal or incoherent carrier preserves the unpublished split/source state and is a global hard stop. Force-push and partial retirement are forbidden.

## Direct fixed-U callers and consumers

- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`: direct scheduler caller; launches one fresh non-delegating split worker, validates the canonical split result, replaces the source id in the frozen batch in place, prioritizes the carrier and hard-stops on unsafe handoff/publication state.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`: producer boundary; the performer may publish split-required with retained source state but cannot create replacements or mutate the queue.
- `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`: consumes retained `split.yaml` as durable history, resolves live successors and forbids new dependencies/project links from naming the retired split id.
- `.agents/shared/project-context.md@53c8657eb7949a5a2fba7e7adc8d1073f3079ead`: governs project lineage and narrow reconciled project-index edits.
- `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`: concrete helper owner for split-result sealing, `carried-work.json`, split graph validation, `split-publish`, retained-content digests, carrier state handoff, worktree-seal validation and source-ref transfer at canonical Start.
- `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`: regression consumer covering split publication, retained carrier routing/start, split resolution, source-ref preservation/transfer and recovery safety.

## Fabushi implementation delta

The existing source-neutral repository process owner is extended, not duplicated:

- root `AGENTS.md` now specifies split eligibility, independently shippable/testable replacement construction, exclusive retained-work carrier semantics, dependency/project rewrite, sealed history, explicit-path CAS publication and fail-closed recovery;
- Test Strategy now defines detailed split eligibility/replacement/carrier/publication/recovery contracts;
- Test Evidence Contract now defines the evidence required to prove criterion/path ownership, carrier handoff, dependency rewrite, atomic publication and recoverable hard stops.

No production platform-core or shipping Desktop runtime code changes because this responsibility governs repository development, not product execution.

## Authored verification contracts

Authored in Stage A and deliberately not run:

- `CONTRACT-TDRP-CONTINUE-SPLIT-ELIGIBILITY-001`;
- `CONTRACT-TDRP-CONTINUE-SPLIT-REPLACEMENT-001`;
- `CONTRACT-TDRP-CONTINUE-SPLIT-CARRIER-HANDOFF-001`;
- `INT-TDRP-CONTINUE-SPLIT-PUBLISH-001`;
- `FAULT-TDRP-CONTINUE-SPLIT-RECOVERY-001`.

Validation status remains `not-run`; acceptance remains false. Stage-B exact-head executable verification is pending.

## Stage-A attestation

This round performs no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance, and does not query, trigger, queue, wait for or rerun GitHub Actions/check/status/workflow state.
