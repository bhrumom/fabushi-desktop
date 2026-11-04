# Complete read — fixed-U OpenAI process-inbox host adapter

## Authority snapshot

- Desktop checkpoint parent: `a3a83766535bf3470f86eaefd17db67c9ff4ab0a`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursively verified `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`, recursive-tree index 77 (zero-based; 78th entry).
- Exact blob read completely: 234 Git bytes / 234 ASCII characters / 4 newline-delimited lines.

## Exact source contract

The file is a declarative OpenAI host-adapter surface with exactly three interface fields:

- `interface.display_name = "Process AI Inbox"` supplies discoverability copy.
- `interface.short_description = "Route inbox notes into project-aware AI tasks"` summarizes the route-and-plan boundary.
- `interface.default_prompt = "Use $process-inbox to route and plan the local AI inbox while preserving existing project lineage."` invokes the canonical process-inbox skill and preserves its project-continuity bias.

It does not define inbox acquisition, immutable snapshot/digest state, project/task/source lineage, superseded/split resolution, task-state schema, publication/finalization, helper execution, validation or acceptance. Those remain owned by the parent process-inbox skill and the canonical repository task/process authority.

## Direct consumers and adapter boundary

At fixed U, the semantic target is `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`; the default prompt delegates through `$process-inbox` instead of reproducing router/planner logic. The OpenAI host consumes the conventional `agents/openai.yaml` interface metadata as its discovery/invocation adapter. Repository-local search found declarative `default_prompt` definitions rather than a second in-repository OpenAI parser or durable state owner.

Peer fixed-U adapters confirm that this is a generic metadata boundary:

- `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`
- `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`
- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30`
- `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`

Claude `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21` and Grok `.grok/commands/process-inbox.md@5fcd9055de9b14a5163dc96a6c3551096898de18` are peer host adapters for the same skill. They reinforce route-and-plan-only semantics, project-continuity, isolated inbox publication, and no implementation/build/test behavior; they do not consume or supersede OpenAI metadata and do not become task/process owners.

## Existing-owner resolution

Existing-owner-first maps this entry into the already-established `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus repository quality/test-evidence governance. The prior complete read of process-inbox `SKILL.md` already established acquisition, routing, lineage, task schema, split/consolidation interoperability, publication, finalization and recovery under that owner.

Rejected owners remain:

- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime;
- a copied OpenAI-skill runtime or Telegram-specific inbox scheduler;
- a second queue, receipt, ownership, project-lineage or acceptance store.

Exact platform-core commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` has no repository host-adapter/task-process production ownership. No platform-core production delta and no ADR are required.

## Fabushi contract delta

The existing responsibility is extended, not duplicated. Qualification/evidence now explicitly requires the Process AI Inbox OpenAI adapter to preserve the parent process-inbox route-and-plan/project-lineage contract while remaining non-authoritative.

Authored, not run:

- `CONTRACT-TDRP-PROCESS-INBOX-OPENAI-HOST-METADATA-001`
- `INT-TDRP-PROCESS-INBOX-OPENAI-HOST-INVOCATION-001`

No build, typecheck, lint, validator, generator, dependency resolution, process-inbox helper/inbox transaction/source-lineage command, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, packaging, deployment, installation, runtime evidence or acceptance is executed in Stage A.

## Source-closure disposition

This dossier qualifies exactly one additional independent fixed-U source entry: `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`, under the existing repository process responsibility. Responsibility count therefore does not increase.

The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3`.
