# Continue skill complete read

## Source authority
- Repository: `telegramdesktop/tdesktop`
- Commit: `78ceaa548ae0e055aa56b53fde22064d13f250ef`
- Tree: `76b967833b38eb7aa2cb4047008c07331390e61e`
- Path: `.agents/skills/continue/SKILL.md`
- Blob: `aeb81dd57018abbc8f03ea3dbd34f91952128db5`
- Complete read: 722 lines.
- Recursive authority index: 35, immediately after `.agents/skills/code-health/scripts/sources.py@688aa05c4223d0d45b8572cff89fbc5148254844` at index 34.
- The next strict source entry is `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`.

## Complete responsibility
The continue skill is a repository-development scheduler, not Telegram product runtime. Its job is to drain one explicitly requested batch of durable AI development tasks without silently taking unrelated queue work.

The 722-line contract requires an explicit batch specification before mutation, otherwise queue inspection is read-only. It freezes the invocation batch, prioritizes already-active work, admits only user-selected unfinished work plus strictly scoped same-invocation discovery/split successors, and refuses to treat a soft batch file as a reservation. It rechecks source lineage before Start/Retry and before a pre-Phase-1 resume, prepares source/submodules without destroying local ownership, runs exactly one stateful performer at a time, and treats Approve, genuine Block, Split-required, or a global environment stop as distinct boundaries.

Split-required work is rerouted through an isolated deep-split transaction. Retained implementation may transfer to exactly one first replacement carrier; source task history remains durable. After terminal task results, discovery routing is scope- and lineage-filtered; broad or unrelated discovery does not enter the frozen batch. Pending-task consolidation is a separate single-writer transaction limited to safe unclaimed pristine todo work with project/batch/dependency/content-digest guards. A blocker is task-local: unrelated frozen-batch work continues. Housekeeping may archive old fully completed projects, and reporting stays compact/reconstructable rather than treating internal commit hashes as user state.

## Concrete workflow/script authority
The actual state transitions are implemented by `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c`.

Relevant complete command/helper blocks read:
- `prepare_source` / `command_source_prepare`: reject source-local dirt, update only mismatched recorded submodule pins, then require clean source.
- `command_queue`: classify own in-progress/todo/blocked/split-required and unclaimed todo work, report soft batch state, pending consolidations, dirty/unpublished state, and structural violations.
- `batch_task_order`: started task first, preserves initial order, drops only terminal batch statuses and rejects unknown ids.
- `command_start`: synchronizes canonical AI state, validates todo ownership/readiness/source lineage, forbids another active stateful task, atomically writes in-progress state plus batch, and handles a lost Start without publishing a stale batch.
- `source_lineage_report` / `command_source_lineage`: resolves approved retained-source prerequisites to source commits, identifies missing lineage and compatible local branches, and distinguishes unavailable history.
- `command_retry`: reopens only checkout-owned blocked work after readiness/lineage checks, remains local, and clears stale discovery/consolidation markers.
- `command_split_publish`: validates split-required ownership, at least two unique replacements, exact publication paths and retained-work snapshot, assigns at most one first implementation carrier, replaces source state with durable split record, validates and publishes.
- `command_task_content_digest` / `command_consolidate_publish`: bind consolidation to retained task content; publish only explicit source/replacement/receipt paths; validate aliases/dependencies and no-merge completion; refuse unexpected state.
- `command_route_ensure` / `command_route_publish`: isolate discovery routing publication and reject changes outside explicit paths.
- `command_archive_stale`: archive only old projects with no unfinished tasks.
- `command_publish`: resume generic canonical publication with consolidation/split validation and source-ref recovery.

## Real callers and host adapters
Direct callers/registrations were read:
- `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e`: loads the shared skill, supplies the explicit batch, preserves frozen batch/split/consolidation state through compaction, and uses blocking Grok workers.
- `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`: same scheduler owner with Claude/Codex review-provider mechanics and source-lineage handling.
- `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`: native invocation registration; this is the next independent source entry and is not closed by this dossier.
- `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154`: maps shared child lifecycle onto Grok, forbids duplicate/invented wait controls, keeps the continue parent as scheduler only, and preserves frozen-batch state across compaction.
- `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952` and `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652`: effort selection plus result-driven child completion/recovery and non-duplication.
- `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`: isolated deep split transaction and exactly-one carrier rule.
- `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`: unfinished-only safe consolidation with project/batch partitioning, digest/dependency checks and no source mutation.
- `.agents/skills/process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db` and `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`: routing/planning and exactly-one-task implementation/review/evidence publication boundaries used by continue.

## Regression evidence read
`.agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b` provides direct state-machine coverage for the helper that makes the continue policy concrete.

Read positives/negatives include:
- `test_start_atomically_assigns_unclaimed_todo`
- `test_start_refuses_source_dependency_absent_from_branch`
- `test_source_lineage_finds_compatible_local_branch`
- `test_retry_reopens_owned_blocked_task_and_resets_routing_marker`
- `test_retry_refuses_to_compete_with_active_task`
- `test_normal_lifecycle_publishes_only_start_and_approve`
- `test_split_publish_routes_and_starts_implementation_carrier`
- `test_start_writes_the_batch_into_its_start_commit`
- `test_start_rejects_unknown_batch_ids_before_changing_anything`
- `test_start_without_a_batch_marks_only_the_started_task`
- `test_a_batch_marks_tasks_without_claiming_them`
- `test_a_lost_start_leaves_no_batch_published`
- `test_queue_lists_an_absent_batch_as_empty`
- `test_batch_option_collects_every_id_in_order`
- `test_reading_a_batch_ignores_what_it_cannot_use`
- `test_queue_inventory_finds_pending_consolidation_markers`
- `test_no_merge_consolidation_publishes_durable_completion`
- `test_merged_consolidation_publishes_a_removed_directory`
- `test_merged_consolidation_validates_aliases_and_dependencies`
- `test_archive_stale_moves_only_old_fully_approved_projects`
- `test_archive_stale_without_candidates_reports_nothing`
- `test_preparation_updates_recursive_recorded_pins_without_following_remote` and adjacent local-change/worktree safety cases.

There is no single direct unit test that interprets the complete 722-line Markdown scheduler policy or the host command text. That is a policy/caller gap, not passing evidence. Concrete helper transitions are unit/integration tested upstream; Fabushi's mapping is governed and revalidated by exact-head repository evidence rather than by copying Telegram's developer scheduler.

## Fabushi existing-owner audit
The applicable responsibility is repository-development task/evidence orchestration. Fabushi already has a source-neutral owner:
- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` requires exact live task/source context, traceable tasks, proportional verification, fail-closed evidence classification, compact reconstructable handoff, and explicitly defines **Repository quality-health command orchestration governance** as the boundary composing source acquisition, measurements/findings, review policy, proposal/routing/approval and the ordinary task lifecycle.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` qualifies repository command orchestration separately from product runtime, plus AI worker lifecycle/adapter behavior.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` binds task/context/worker evidence to exact source, durable terminal results, owned process/session identity, serialized atomic publication, resume lineage and append-only recovery history.

Shipping orchestration owners were also inspected and deliberately rejected:
- `source/host/src/extensions/transcript/run_scheduler.rs@eb8994dc45b3722e8bfcc672e9e610c662429650` owns per-agent user/background run lanes, watchdog, settlement and diagnostics.
- `source/host/src/runner/background_work.rs@85df5a9b8636e77e08678dac1de4756073b4da91` owns background wakeups plus cloud/shell watch cancellation and completion.
- `source/host/src/sand_multitask.rs@45e570098583a21e058e91b40d57ef45335caed3` owns the end-user durable todo prompt and intentionally allows several independent streams in progress.
- `source/host/src/extensions/transcript/workflow_commands.rs@35d5863225b4de40e6f321873f33fba3e7a2e43d` + `source/host/src/workflows/workflow_store.rs@5132f6889bfa31372b3f1a9faf604d5874f2e1d4` own user/agent workflow commands and definitions.
- `source/host/src/extensions/transcript/automation_runtime.rs@2a30406f2dd138a29a9bb90cf9f5324aa944e731` + `source/host/src/automations/automation_store.rs@8ffd2175c7afb65611f65a1d56f3368b79881220` own scheduled/event/manual automations and run history.

Those shipping owners must not absorb repository branch/source-lineage/task-publication semantics. No second shipping Task store, Automation scheduler, Workflow store, repository queue or worker-lifecycle owner is introduced.

## Parent evidence is historical-only
Parent target `fb04df3f5e3b5f3d077a1c216205001970981fa8` was re-read before this write:
- TDRP Revision 9 source authority run `39481285146`: completed success at exact parent SHA; 61/61 jobs successful, 506/506 steps successful, every job had a non-empty GitHub runner id/name.
- Main authority artifact `12290068816`, GitHub digest and parent downloaded ZIP SHA-256 `64ebf5a260fa6931ba5a69a21fb8b78aae06e65f9eeaa9e28b49492e67ae1371`.
- Rust desktop runtime run `39481285124`: completed success, 6/6 jobs successful, 79 successful steps and 2 platform-conditional skipped steps.
- Rust artifact `12291662749`, GitHub digest and parent downloaded ZIP SHA-256 `df1178f0ba508dafb8aebf85e15c896e6bb01995739e97e1fa41cc19e0d017a0`.
- TDRP published 60 artifacts and Rust published 1; all 61 artifact records bind `workflow_run.head_sha=fb04df3f5e3b5f3d077a1c216205001970981fa8`, and the parent round downloaded all 61 ZIPs and matched each actual SHA-256 to its GitHub digest.

Creating this mapping commit demotes every one of those parent artifacts to historical-only. Only the new exact HEAD can satisfy current evidence.

## Closure accounting
This commit closes only recursive index 35:
- unknown: 16,164 -> 16,163
- unread: 16,139 -> 16,138
- omitted: 0 -> 0

Baseline acceptance remains false, `source_closure_ready=false`, generated/resource/tool disposition remains open, and independent release acceptance remains open.

## Traceability
- Requirement: `TDRP-R9-CONTINUE-WORKFLOW-001`
- Oracles: `ORA-TDRP-CONTINUE-FROZEN-BATCH-001`, `ORA-TDRP-CONTINUE-SOURCE-LINEAGE-001`, `ORA-TDRP-CONTINUE-SINGLE-WRITER-001`, `ORA-TDRP-CONTINUE-DURABLE-TERMINAL-EVIDENCE-001`
- Invariants: `INV-TDRP-CONTINUE-NO-SILENT-BATCH-GROWTH-001`, `INV-TDRP-CONTINUE-NO-DUPLICATE-STATEFUL-WRITER-001`, `INV-TDRP-CONTINUE-NO-PRODUCT-QUEUE-COLLISION-001`
