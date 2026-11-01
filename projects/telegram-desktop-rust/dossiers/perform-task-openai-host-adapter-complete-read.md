# Complete read — fixed-U OpenAI perform-task host adapter

## Authority snapshot

- Desktop checkpoint parent: `56a503be59d4fe1378e7475708d7002dc8f7ec09`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursively verified `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`, fixed-U recursive-tree index 69 (1-based, including tree entries).
- Exact blob read completely: 229 bytes / 4 logical lines.

## Exact source contract

The file is a declarative OpenAI host-adapter surface:

- `interface.display_name = "Perform AI Task"` supplies host discovery copy.
- `interface.short_description = "Implement or resume one Telegram AI task"` projects the exactly-one-task performer boundary.
- `interface.default_prompt = "Use $perform-task to implement, adaptively review and test, and publish one named ai-tdesktop task."` delegates execution to the shared performer and names its implementation/review/test/publication lifecycle.

The adapter does not define task resolution, Start/Retry/Resume ownership, source lineage, planning, phase state, mechanical-lane eligibility, review/convergence, evidence recovery, final AI state, queue scheduling, split routing, consolidation, validation, acceptance or publication authority.

## Direct consumers and host-adapter boundary

The fixed-U semantic target is `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`; the default prompt explicitly invokes `$perform-task` rather than reproducing its state machine.

Fixed-U peer host entrypoints confirm the boundary:

- `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e` reads the shared skill, performs only the named task, and leaves post-result queue/routing/consolidation to `continue`.
- `.grok/commands/perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509` does the same while adapting Grok-specific delegation.
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` is the scheduler that selects/spawns exactly one performer and owns post-result split/routing/consolidation.
- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` consumes the performer's bounded mechanical lane and pre-review checks without becoming the performer owner.
- Root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` exposes the repository-level `perform-task <slug or full id>` invocation boundary.

The same four-line `agents/openai.yaml` schema is used by adjacent Continue, Dependency Watch, Code Health and Process Inbox host adapters. This source is therefore provider discovery/invocation metadata, not an OpenAI-specific task runtime.

## Existing-owner resolution

Existing-owner-first attaches this source entry to the already-established `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` responsibility under Fabushi repository development process authority + repository quality/test-evidence governance.

Rejected owners:

- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime;
- a Telegram/OpenAI-specific performer or queue;
- a second task, review, evidence or publication store.

Exact platform-core `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` has no repository performer host-adapter ownership. No new responsibility, ADR, shipping runtime or platform-core production delta is required.

## Fabushi contract delta

The existing performer responsibility is extended rather than duplicated. Repository governance now makes the OpenAI metadata boundary explicit, while Test Strategy and Evidence Contract qualify the exact metadata and its delegation to the canonical single-task performer.

Authored, not run:

- `CONTRACT-TDRP-PERFORM-TASK-OPENAI-HOST-METADATA-001`
- `INT-TDRP-PERFORM-TASK-OPENAI-HOST-INVOCATION-001`

No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, runtime evidence campaign or acceptance is executed in Stage A.

## Source-closure disposition

This dossier closes exactly one additional independent fixed-U source entry: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`, by attaching it to the existing performer responsibility. Responsibility count therefore does not increase.

The next independent fixed-U blob in tree order is `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`.
