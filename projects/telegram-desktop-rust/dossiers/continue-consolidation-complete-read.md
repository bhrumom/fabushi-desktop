# Complete read — fixed-U continue pending-task consolidation

## Authority snapshot

- Desktop checkpoint parent/live head before spec publication: `b1ecb2452f4fe8c9d215c6e2731573cac735a95f`.
- Desktop main reread this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` in `bhrumom/fabushi-platform-core`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U recursive root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (previously established non-truncated 7,194 entries / 6,859 blobs; no rebaseline in this round).
- Source entry: `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`.
- Exact blob read completely: 14,614 characters / 285 lines.

## Responsibility classification

This source is repository-development process governance. It optimizes canonical pending AI work after discovery routing; it is not Telegram shipping product behavior, not product Tasks/Automations, and not a second scheduler/runtime owner.

Existing-owner-first result:

- existing owner: root `AGENTS.md` repository task/process authority governance;
- qualification owner: `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- evidence owner: `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- same responsibility: `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`;
- no new owner, responsibility, ADR, platform-core runtime, Telegram-specific queue, or shipping product delta.

Rejected owners remain Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a copied upstream AI scheduler, and any second receipt/ownership store because none owns repository-development queue authority.

## Exact consolidation boundary

The pass runs only after discovery-routing publication and receives the source checkout, clean checkout-specific AI slot, checkout tag, source task pending marker, and effective frozen batch ids. It is one optimization pass, not discovery planning or task execution. It must not read or mutate Telegram source/build/test/inbox/claim/execution state, invent work, move projects, alter approved history, or change request meaning.

The current frozen `batch_task_ids` are authoritative. Marker batch contents are recovery input only before a new batch is frozen. Project reads and index edits follow the shared project-context policy.

## Eligibility and dependency safety

Only unfinished canonical work participates in the inventory. Approved/retired work is terminal history and remains opaque except as dependency targets.

A candidate is eligible only when all of these hold:

- `status: todo`;
- `claimed_by`, `claimed_at`, `claim_order`, `lease_until`, and `phase` are null;
- no tracked `work/`, `evidence/`, or unexpected task-local artifact shows execution began;
- every needed input can be copied and relinked without loss;
- it shares the exact project class with the cluster (same project slug, or all standalone);
- it shares the exact frozen-batch-membership class with the cluster;
- it is not a Code-health-class task whose own policy sizes the batch;
- no active/claimed/blocked/otherwise non-mergeable unfinished dependent would be stranded.

Eligible unclaimed `todo` dependents may be rewritten only atomically with the replacement. Before selection, simulate the replacement in the unfinished dependency graph: remove the cluster, add the stable union of external dependencies, rewrite eligible dependents, and reject direct/transitive self-dependency or any cycle.

## Merge and no-merge semantics

The merge bias exists only when one normal execution can reuse a material part of context, implementation, fixture/account/process setup, measurement instrumentation, review, build, or test work. Same project alone is not sufficient. Keep tasks separate when they require independent designs/fixtures, materially interfere, cross a real sequential product boundary, or no longer fit one adaptive implementation pass.

A merged replacement preserves every unique acceptance criterion, prerequisite, dependency, input, safety/fixture fact, project lineage, and source-specific coverage boundary. Exact duplicate criteria may collapse only when the receipt maps them to an equally strong replacement criterion.

A no-merge result is still durable: remove the pending marker, write the completion marker with `STATUS: NO_MERGE`, and change no unrelated task/project/receipt state. This is the no-repeat boundary after restart.

## Replacement, aliases, and fingerprints

For each selected cluster:

- create one pristine unclaimed `todo` replacement with shared project identity and stable external dependency union;
- compute each source task retained-content fingerprint before retirement;
- retire each source by removing only live `state.yaml`;
- retain original `task.md` and `input/`;
- add `superseded.yaml` with exact replacement id, receipt, type/project, and content fingerprint;
- copy/relink supplied inputs with collision-safe paths and account for every mapping;
- rewrite eligible external dependents and project navigation from old ids to the live replacement;
- keep execution artifacts out of retired directories.

The durable alias is not runnable state. Workspace resolution follows `superseded.yaml` chains to canonical live work. A late retained-content change invalidates the fingerprint and blocks safe publication.

## Publication, CAS, and recovery

Immediately before tracked writes the pass refreshes canonical state and rereads every selected source and rewritten dependent. Any change to status, owner, project, dependency, or tracked task content is a pre-commit race: write nothing, return `RACED`, and leave the pending marker for a later invocation.

Tracked publication is explicit-path and atomic. The publication helper verifies:

- pending marker replaced by the matching completion marker;
- exact old-to-new alias set matches the requested mappings;
- retired source content was not modified except state removal plus alias creation;
- every merged replacement is newly created pristine `todo`;
- merged receipts exist and no-merge publishes no receipt;
- aliases/fingerprints, splits, and the complete live dependency graph validate;
- no project index still links a retired id;
- no changes exist outside the explicit publication path set.

The helper commits once and publishes through compare-and-swap semantics. After every fetch/rebase, the generic publish path recognizes the consolidation commit and repeats semantic validation before push. A committed late edit, new dependency cycle, semantic conflict, or remote outage returns `BLOCKED` while preserving the recoverable commit. Force-push is forbidden.

## Direct fixed-U callers and consumers

- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`: direct scheduler caller. It starts one fresh non-delegating consolidation leaf after routed discoveries, attempts pending markers at most once per invocation, rewrites effective batch/discovered ids after a published mapping, treats pre-commit race as non-blocking optimization failure, and hard-stops on unsafe committed publication state.
- `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`: consumes retired `superseded.yaml` ids as durable aliases during deduplication/routing and requires new dependencies/project links to name final live tasks.
- `.agents/shared/project-context.md@53c8657eb7949a5a2fba7e7adc8d1073f3079ead`: governs selective project/index reads and narrow reconciled edits.
- `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`: concrete helper owner for `retained_task_digest`, superseded alias loading/resolution, dependency/split/alias validation, `validate_consolidation_tree`, `consolidation_validation_for_head`, `task-content-digest`, `consolidate-publish`, and generic publish revalidation.
- `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`: fixed-U regression consumer covering durable alias resolution, pending-marker discovery, no-merge completion, merged publication, alias/dependency integrity, and generic publish recognition.

## Fabushi implementation delta

The existing source-neutral repository process owner is extended, not duplicated:

- root `AGENTS.md` now states fail-closed eligibility, partition/dependency simulation, merge/no-merge preservation, durable alias/fingerprint, final reread, explicit-path atomic publication, CAS, and recovery rules;
- Test Strategy now defines qualification contracts for eligibility, merge/no-merge, alias/fingerprint, publish, and race/recovery;
- Test Evidence Contract now defines the evidence required to distinguish safe merge/no-merge, `RACED`, and committed `BLOCKED` outcomes and records the exact fixed-U consumers.

No production platform-core or shipping Desktop runtime code is changed because this responsibility governs repository development, not product execution.

## Authored verification contracts

Authored in Stage A and deliberately not run:

- `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ELIGIBILITY-001`;
- `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-MERGE-NO-MERGE-001`;
- `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ALIAS-FINGERPRINT-001`;
- `INT-TDRP-CONTINUE-CONSOLIDATION-PUBLISH-001`;
- `FAULT-TDRP-CONTINUE-CONSOLIDATION-RECOVERY-001`.

Validation status remains `not-run`; acceptance remains false. Stage-B exact-head executable verification is still pending.

## Stage-A attestation

This round performs no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deploy, install, or runtime acceptance, and does not query, trigger, queue, wait for, or rerun GitHub Actions/check/status/workflow state.
