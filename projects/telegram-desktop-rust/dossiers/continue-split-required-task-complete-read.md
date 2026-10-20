# Complete read — .agents/skills/continue/references/split-required-task.md

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@78ceaa548ae0e055aa56b53fde22064d13f250ef`
Upstream tree: `76b967833b38eb7aa2cb4047008c07331390e61e`
Source: `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`

## Inventory identity

PR #154 and branch `tdrp-r9-entry31-includes-r27-20261020` were re-read at parent `91d7db2db7bdcdf5a13d1abd9659a46cbedc4f07` / tree `8d96fa822da6592f43559c858d198e691a690a59`; live upstream `dev` remained `78ceaa548ae0e055aa56b53fde22064d13f250ef` / tree `76b967833b38eb7aa2cb4047008c07331390e61e`.

Zero-based recursive authority index 38 is this file. The next strict entry is index 39, `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`.

Parent TDRP run `39495231120` completed successfully at the exact parent: 61/61 jobs and 506/506 steps succeeded. Source authority job `118878465787` revalidated root entries=7012, root blobs=6679, direct gitlinks=36, recursive non-directory entries=16201 with exact reconciliation, plus cppgir/expected-lite participation; artifact `12296714976`, digest `sha256:9447c3e071bb8aa578e9ff2d1ca6b4595296b9cbf2afa6a8f94875ad9bdb7305`.

Fixed-point job `118881711068` scanned 203 authorities and ended with `unresolved_gitlink_authorities=0`; artifact `12298126603`, digest `sha256:d86315b3bbc6519a44f26e96f00cc235fdd27835e8d80742b8ba4b8773445ae0`. Boost job `118878739418` reports 582 candidates / 582 dispositioned / open=0; artifact `12297411524`, digest `sha256:b29e28c2e4e634a1401992cc97e1f1235c604e7fdeea16c509e97f031870699d`.

The same TDRP run is not full accepted source authority under the current gate. Reachability job `118881879880` is explicitly `partial-audited-open`: 108 authorities are proven non-reachable, but remaining-open totals are authorities=95, gitlinks=133, acquisition_candidates=3865, generated_inputs=4674, resource_inputs=4602, tool_inputs=4255. Artifact `12298730361`, digest `sha256:cbb6ba947678e08e816ab85d724b9a29e37252c5eac12ae0a67e83cc0beb85fe`. The fixed-point report itself keeps `recursive_disposition_status=open` and `source_closure_ready=false`.

Parent Rust run `39495231072` completed successfully at the exact parent: 6/6 jobs, 79 successful steps and two expected platform-conditional skips. Artifact `12298695193`, `rust-messaging-responsibilities-91d7db2db7bdcdf5a13d1abd9659a46cbedc4f07`, digest `sha256:d087bd0bd7f834d370d259be40a238467b300bb5fcb4acc48c47c54527400228`.

## Complete source read

This reference owns one bounded repository-development transaction after a performer has already published `split-required`. It explicitly forbids the split worker from editing, resetting, stashing, committing, building or testing product source.

1. Read the full source task, project/dependency context, work context, plan, split proposal, result, carried-work seal, retained diff and adjacent source.
2. Refine into the minimum independently shippable/testable boundaries; preserve inseparable callers together.
3. Preserve project, relevant original dependencies, acceptance scope and durable lineage; derive replacement dependencies from shipped behavior.
4. If retained implementation exists, exactly one first replacement is the implementation carrier. It owns the retained diff coherently; later replacements consuming it depend on the carrier. Phase approvals are never copied.
5. If no coherent carrier can own all retained implementation, stop with source state untouched.
6. Create at least two pristine unclaimed todo replacements plus one receipt, rewrite live reverse dependencies/project indexes, and remove the retired split id from live edges.
7. Seal retained content and replace live `state.yaml` only with `split.yaml`; retain task/history content.
8. Publish only through `workspace.py split-publish`, which rechecks carried-work seal, source ownership, pristine replacements, receipt/project linkage, dependency/superseded/split graphs and explicit changed-path coverage.
9. With a carrier, source refs persist until scheduler selection and ordinary `start` transfer; without a carrier, obsolete source refs are deleted after successful publication.
10. Races retry through the helper; semantic conflicts, changed seals, incoherent carrier or unavailable remote remain visible/recoverable and fail closed.

## Real callers and tests

- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` owns the scheduler transaction: exactly one fresh split worker, ordered replacement insertion into the frozen batch, and first-carrier selection/transfer.
- `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e` and `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` expose the same shared scheduler behavior.
- `.grok/commands/perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509` and `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e` publish split-required and stop; replacement creation belongs to continue.
- `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` implements carried-work sealing, dependency/superseded/split validation, split.yaml, exactly-one carrier transfer and atomic split publication.
- `.agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b` directly covers `test_finish_publishes_split_required_with_carried_work`, `test_carried_work_snapshot_seals_owned_submodule_changes`, `test_split_publish_routes_and_starts_implementation_carrier`, plus shared dependency-cycle negatives.

## Existing-owner-first resolution

Fabushi exact-head already owns the applicable source-neutral responsibility in `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb`, and `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` through canonical task/status/handoff, ordinary task routing/conflict qualification, and fail-closed task-local evidence.

Shipping candidates were inspected and rejected as semantic owners: `run_scheduler.rs@eb8994dc45b3722e8bfcc672e9e610c662429650` owns product User/Agent/Background run lanes; `runner_prompt_glue.rs@fed229ea2c4226eba00466bd5413ec9b85c6aee5`, `system_prompt_assembly.rs@6d592794b835994a81a214049b9c5907df0cb4ff`, and `turn_agent_composition.rs@eefde7f77856070e62e4f8dbca7cfb7f28da0a17` own shipping provider/prompt/tool-turn composition. None owns repository task decomposition, retained-work digests, dependency/project rewrites, source refs or Git publication CAS.

No new product owner, second Task store, second scheduler, source-specific runtime or copied upstream Python workflow is required.

## Fabushi-equivalent contract

`ordinary repository task execution -> split-required + sealed retained work -> one fresh source-neutral split transaction -> complete task/context/dependency read -> minimum independent replacement boundaries -> preserved acceptance/project/dependencies + receipt -> optional exactly-one first carrier -> whole-graph validation -> atomic publication -> retired source split record -> ordinary carrier start/ref transfer -> fresh exact-head evidence`

Fail-closed invariants:
- no acceptance criterion, dependency, project relation or retained history is lost;
- retained implementation has zero or exactly one coherent carrier, always first;
- stale seals, dependency cycles, semantic conflicts and publication races cannot publish weakened state;
- split routing cannot mutate product source or convert prior phase output into approval;
- shipping Task/RunScheduler/Automation/Workflow state cannot become repository-development split truth;
- parent CI is supporting/historical after this mapping commit;
- source closure remains false until unknown=0, unread=0, omitted=0 and all recursive remaining-open dispositions close.

## Traceability

- Requirement: `TDRP-R9-CONTINUE-SPLIT-REQUIRED-ROUTING-001`
- Oracles: `ORA-TDRP-CONTINUE-SPLIT-BOUNDARY-001`, `ORA-TDRP-CONTINUE-SPLIT-CARRIER-TRANSFER-001`, `ORA-TDRP-CONTINUE-SPLIT-PUBLICATION-001`
- Invariants: `INV-TDRP-CONTINUE-SPLIT-NO-RETAINED-WORK-LOSS-001`, `INV-TDRP-CONTINUE-SPLIT-DEPENDENCY-GRAPH-CLOSED-001`, `INV-TDRP-CONTINUE-SPLIT-FAIL-VISIBLE-001`, `INV-TDRP-CONTINUE-SPLIT-NO-SHIPPING-QUEUE-COLLISION-001`

## Verification disposition

This closes only recursive index 38. Counters become recursive=16,201, unknown=16,160, unread=16,135, omitted=0. Baseline accepted=false, baseline_ready=false and source_closure_ready=false.

Exact-head acceptance of this mapping requires fresh TDRP Revision 9 source-authority and Rust desktop runtime workflows on the new commit. Full migration/source closure remains prohibited until all inventory counters and recursive remaining-open dispositions close and downstream production/release acceptance passes.
