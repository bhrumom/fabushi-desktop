# Complete read — fixed-U Claude AI workflow host adapter

## Authority snapshot

- Desktop checkpoint parent: `8705bf5da06690626271f1cf7dc5ea8fdb5888b9`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`, recursive-tree index 86 zero-based.
- Exact source read completely: 13,188 UTF-8 bytes / 13,166 decoded characters / 215 newline characters / trailing newline / 216 split lines.

## Exact host-adapter contract

The adapter is opt-in: it applies only when a command explicitly loads it. It says the shared `.agents/skills/` workflow remains authoritative for task selection, artifacts, source changes, builds/testing, commits, resumability and AI publication. Claude-specific content therefore adapts harness mechanics rather than creating another durable process owner.

Its delegation rules preserve one stateful performer and single-writer artifacts. Fresh leaves receive self-contained prompts; disposable phase leaves do not delegate or commit; normal phase leaves are synchronous foreground Agent calls whose return is the completion signal. A leaf report is its return value, not a SendMessage side channel. Watchers, Monitor/poll loops and progress-only termination are forbidden. Independent leaves of one step may run concurrently, while rerun/resume is bounded and must prove the earlier writer stopped before another writer targets the same artifact.

Review/assessment may use Codex CLI leaves by default, with native Claude fallback for explicit opt-out or unavailable Codex. Provider choice does not change phase prompts, artifacts, completion checks, carry-forward or verdict semantics. Effort/model plumbing follows shared phase-effort policy and remains host mechanics. Model self-reporting is normalized before canonical task finish state.

Claude text handling removes dedicated line-ending/BOM normalization work only; it does not relax content validation, owned paths, review, tests or publication gates. Source-lineage decisions remain at the shared orchestrator boundaries: startup/pre-freeze mismatch pauses for direction, safe pre-Phase-1 branch switching stays with the parent, post-Phase-1 mismatch is a task-local performer Block, and branch integration is never manufactured as queue work.

UI-driver capability is fail-closed when no equivalent tool is exposed. The only background Agent exception is the shared concurrent routing lane for discovered routing and pending consolidation; ordinary performers and phase leaves remain foreground. Publication resumes through the existing routing helper/state and must be landed before the invocation ends.

## Direct fixed-U consumers

Explicit loaders:

- `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`
- `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`
- `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`
- `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`

Shared workflow dependencies include `.agents/shared/phase-effort.md@8d7b385dbb526e87fc090774ee35191e1619bb08` and `.agents/shared/codex-cli-leaves.md@11c59b68df43e442e596c3052296fc51a044d46a`, plus the already closed continue / perform-task / process-inbox authorities.

Explicit negative boundary:

- `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` says not to read this adapter because branch integration is not AI queue work.

## Existing-owner resolution

Existing-owner-first maps this cross-cutting source into **Fabushi repository development process authority + repository quality/test-evidence governance**.

A new ledger responsibility is appropriate because one independent cross-cutting source blob constrains several existing process responsibilities, but it does not create a new state owner. Rejected owners are shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, platform-core runtime, a Claude-specific task queue/performer/review store, and a second publication/evidence truth.

No ADR or platform-core production delta is required.

## Stage-A authored contracts

- `CONTRACT-TDRP-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-CLAUDE-DELEGATION-SINGLE-WRITER-001`
- `CONTRACT-TDRP-CLAUDE-LEAF-COMPLETION-RECOVERY-001`
- `CONTRACT-TDRP-CLAUDE-REVIEW-PROVIDER-001`
- `CONTRACT-TDRP-CLAUDE-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-CLAUDE-TEXT-HANDLING-001`
- `INT-TDRP-CLAUDE-HOST-PROCESS-COMPOSITION-001`
- `INT-TDRP-CLAUDE-PUBLICATION-BOUNDARY-001`
- `FAULT-TDRP-CLAUDE-DUPLICATE-WRITER-RECOVERY-001`

All are authored only. No Agent/Codex leaf, helper, source-lineage command, build, typecheck, lint, validator, generator, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deploy, install, runtime evidence or acceptance was executed.

## Source-closure disposition

Exactly one independent fixed-U source entry is represented by this dossier. The next independent fixed-U blob in tree order is `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`.
