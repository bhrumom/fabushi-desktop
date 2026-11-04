# Complete read — fixed-U continue task/process scheduler

## Authority snapshot

- Desktop checkpoint parent: `304c467453dcea45be3ed7897444256a8f41a44e`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursively reread with `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`.
- Exact blob read completely: 39,712 characters / 722 lines.

## Source responsibility

The source is a checkout-level development scheduler, not Telegram product runtime. Its portable contracts are:

1. resolve one checkout/workspace and treat current canonical AI/repository task state as authority;
2. recover pre-existing exclusive active, split-required and pending-consolidation state before selecting new shared work;
3. require an explicit batch specification, freeze exactly that initial scope, and never drain unrelated tasks observed later;
4. preserve one active task/owner at a time and establish ownership only through canonical Start-like state publication;
5. keep receipts and soft batch marks non-authoritative for ownership/reservation;
6. gate new execution on approved source-lineage prerequisites and safe source preparation;
7. delegate exactly one task performer and accept only approved, genuine blocked, split-required or explicit hard-stop boundaries;
8. route split-required work as one atomic replacement transaction with complete requirement/dependency coverage and at most one retained-work carrier;
9. route discovered follow-ups separately from execution, only when attributable to the current result;
10. consolidate compatible unclaimed unfinished tasks in a separate transaction, preserving project/batch boundaries, criteria, inputs, dependencies and durable aliases;
11. treat publication races/conflicts fail-closed, preserve recoverable work, and never force-push away semantic state;
12. report scope/result state without turning implementation into verification or acceptance.

## Direct fixed-U callers and consumers

Host invocation wrappers:

- `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` requires the shared continue skill completely and applies only Claude adapter mechanics.
- `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e` requires the same shared skill completely and applies only Grok adapter mechanics.
- `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61` is an OpenAI-facing invocation/default-prompt surface; it owns no durable task state.

Direct composed workflow authorities:

- `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d` defines pristine unclaimed task creation, dependency lineage and inbox publication semantics. It explicitly leaves performer-discovered splitting to the continue scheduler.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` owns exactly one task through approval/block/split-required/hard-stop and returns control to continue for routing/consolidation.
- `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846` defines the atomic split transaction, replacement mapping and retained-work carrier.
- `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c` defines eligible unfinished-work consolidation, old-id durable aliases, no-repeat completion markers and CAS publication.
- `.agents/shared/phase-effort.md@8d7b385dbb526e87fc090774ee35191e1619bb08` and `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652` provide host/worker execution policy but do not own canonical task state.

## Existing-owner resolution

Current Fabushi root `AGENTS.md` already owns repository-wide development lifecycle, source/spec authority, task context/handoff and repository quality governance. The communication-platform `quality/TEST_STRATEGY.md` and `quality/evidence-contract.md` already own source-neutral qualification and durable evidence rules for repository tooling.

Existing-owner-first therefore maps this source to **Fabushi repository development process authority + repository quality/test-evidence governance**.

Rejected owners:

- shipping Product `Tasks` / `Automations`: user/runtime task execution is a different state domain;
- Agent/Coordinator/Host/Runner: shipping execution owners must not own repository-development queue state;
- platform-core runtime: exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` has no repository task/split/consolidation authority surface and requires no production delta;
- a Telegram-specific task queue/scheduler or copied Python/Markdown workflow runtime: would duplicate canonical repository governance;
- a second findings/receipt/ownership store: would split task truth.

No new ADR is required because an applicable source-neutral owner already exists.

## Fabushi contract delta

Spec-first changes extend:

- root `AGENTS.md` with repository task/process authority governance;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` with task/process qualification;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` with durable evidence requirements.

Authored, not run:

- `CONTRACT-TDRP-CONTINUE-BATCH-FREEZE-001`
- `CONTRACT-TDRP-CONTINUE-TASK-STATE-AUTHORITY-001`
- `CONTRACT-TDRP-CONTINUE-OWNERSHIP-001`
- `CONTRACT-TDRP-CONTINUE-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-CONTINUE-SPLIT-TRANSACTION-001`
- `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-001`
- `CONTRACT-TDRP-CONTINUE-RECOVERY-001`
- `INT-TDRP-CONTINUE-HOST-ADAPTER-COMPOSITION-001`

No build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, packaging, deployment, installation or runtime acceptance is executed in Stage A.

## Source-closure disposition

Exactly one independent source entry is closed by this dossier: `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`. Adjacent references were read to understand this source's contracts but remain independently unclosed ledger entries.

The next independent unrepresented fixed-U blob in tree order is `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`.
