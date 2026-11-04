# Complete read — fixed-U OpenAI Dependency Watch host adapter

## Authority snapshot

- Desktop checkpoint parent: `46241ddf29ca947ce1dae3fafae56e27ba13bf97`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursively verified `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`, fixed-U tree index 58.
- Exact blob read completely: 260 characters / 4 lines.

## Exact source contract

The file is a declarative OpenAI host-adapter surface:

- `interface.display_name = "Dependency Watch"` supplies discovery copy.
- `interface.short_description = "Track dependency releases and missing upstream fixes"` summarizes the repository advisory purpose without owning advisory state.
- `interface.default_prompt` invokes `$dependency-watch`, requires auditing freshly fetched `origin/dev`, and requests pressing updates before the review-someday list.

The adapter therefore selects and frames the already-defined dependency-watch workflow; it does not own snapshot identity, recursive dependency inventory, release/advisory research, security applicability, fork/backport provenance, release-trust decisions, finding persistence, report publication, dependency mutation, product updater state, build/test execution, PR/release/deployment, or acceptance.

## Direct consumers and adapter boundary

At fixed U, the semantic target is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`; the default prompt explicitly delegates to `$dependency-watch` rather than reproducing the workflow. Repository search found the declarative adapter and skill-local helper/reference sources, but no second in-repository parser, shipping caller, or durable dependency-advisory state owner.

The same fixed-U OpenAI adapter schema is used by:

- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30`
- `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`
- `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`
- `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`

This confirms the boundary is generic host discovery/invocation metadata rather than a Dependency Watch runtime.

## Existing-owner resolution

Existing-owner-first attaches this entry to the already-established `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` responsibility under existing Fabushi repository quality/test-evidence governance.

Rejected owners remain:

- shipping `source/electron-main/update/*`, which owns signed product update/install/rollback;
- platform-core runtime;
- Shipping Product Tasks/Automations;
- Agent/Coordinator/Host/Runner product runtime;
- build-time acquisition inventory as decision authority;
- a copied OpenAI skill runtime or Telegram-specific dependency monitor.

No new responsibility, product/runtime owner, platform-core production delta, dependency pin, updater state, or ADR is required.

## Fabushi contract delta

The existing dependency-advisory responsibility is extended rather than duplicated. Qualification/evidence now explicitly requires the OpenAI adapter to preserve the canonical advisory boundary while remaining non-authoritative.

Authored, not run:

- `CONTRACT-TDRP-DEPENDENCY-OPENAI-HOST-METADATA-001`
- `INT-TDRP-DEPENDENCY-OPENAI-HOST-INVOCATION-001`

No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, packaging, deployment, installation or runtime acceptance is executed in Stage A.

## Source-closure disposition

This dossier closes exactly one additional independent fixed-U source entry: `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`, by attaching it to the existing dependency-advisory responsibility. Responsibility count therefore does not increase.

The next independent unrepresented fixed-U blob in tree order is `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`.
