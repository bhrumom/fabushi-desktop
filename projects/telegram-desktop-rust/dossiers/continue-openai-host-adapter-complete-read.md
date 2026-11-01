# Complete read — fixed-U OpenAI continue host adapter

## Authority snapshot

- Desktop checkpoint parent: `d620e371dbeab832fc6983462fc2466c5abe111c`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursively verified `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`.
- Exact blob read completely: 291 characters / 4 lines.

## Exact source contract

The file is a declarative OpenAI host-adapter surface:

- `interface.display_name = "Continue AI Work"` supplies discoverability copy.
- `interface.short_description = "Resume active work, then run a specified batch"` projects the scheduler's active-first and explicit-scope semantics.
- `interface.default_prompt` invokes `$continue` and states the portable sequencing contract: put any active task first, freeze exactly the tasks selected by the caller's batch specification behind it, and process the batch one task at a time.

It does not define task state, durable ownership, queue persistence, source-lineage, split routing, discovery routing, consolidation, validation or acceptance. Those remain owned by the parent continue scheduler and canonical repository task/process state.

## Direct consumers and adapter boundary

At fixed U, the semantic target is `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`; the default prompt explicitly delegates to `$continue` rather than reproducing scheduler logic. The OpenAI host consumes this conventional `agents/openai.yaml` interface metadata as an invocation/discovery adapter. Repository-local searches found declarative `default_prompt` definitions, not a second in-repository parser or state owner.

The same fixed-U schema is used by adjacent host adapters, confirming the boundary is generic metadata rather than continue-specific runtime:

- `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`
- `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`
- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30`

Claude/Grok wrappers remain separate host adapters for the same scheduler and do not change this ownership boundary.

## Existing-owner resolution

Existing-owner-first maps this entry into the already-established **Fabushi repository development process authority + repository quality/test-evidence governance**. Root `AGENTS.md` already says host adapters may invoke repository scheduling but cannot become task authority; the existing Test Strategy and Evidence Contract already own qualification/evidence.

Rejected owners:

- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime;
- a copied OpenAI-skill runtime or Telegram-specific scheduler;
- a second queue, receipt, ownership or acceptance store.

Exact platform-core commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` has no repository host-adapter/task-state production ownership. No platform-core production delta and no ADR are required.

## Fabushi contract delta

The existing responsibility `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` is extended rather than duplicated. Spec-first qualification/evidence now explicitly requires the OpenAI adapter to preserve the canonical scheduler contract while remaining non-authoritative.

Authored, not run:

- `CONTRACT-TDRP-CONTINUE-OPENAI-HOST-METADATA-001`
- `INT-TDRP-CONTINUE-OPENAI-HOST-INVOCATION-001`

No build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, packaging, deployment, installation or runtime acceptance is executed in Stage A.

## Source-closure disposition

This dossier closes exactly one additional independent fixed-U source entry: `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`, by attaching it to the existing continue task/process responsibility. Responsibility count therefore does not increase.

The next independent unrepresented fixed-U blob in tree order is `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`.
