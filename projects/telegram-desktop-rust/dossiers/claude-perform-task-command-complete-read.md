# Fixed-U complete read — Claude perform-task command adapter

## Authority snapshot

- Desktop checkpoint parent: `ce410ff4f35bdfdfcf1a2418e254a7fe9d437c2d`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`, recursive-tree index 93 zero-based.
- Exact source read completely: 2,068 bytes / 2,068 decoded characters / 42 newline characters / trailing newline / 43 split lines.
- Next fixed-U blob: `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`.

## Complete source semantics

The command is a Claude Code host wrapper for one already-existing repository AI task. It explicitly loads the generic Claude workflow adapter and the shared single-task performer skill completely, then applies only Claude-specific delegation/text/review-provider substitutions. It resolves, starts or resumes, implements, verifies and publishes exactly one named task. Post-result queue draining, discovered routing, pending consolidation and creation of replacements for a split result belong to the `continue` scheduler.

If assessment or review convergence proves intrinsic breadth, the wrapper publishes the shared `split-required` result with retained source state and stops. The shared conditional `[ai] ` commit-subject rule is decided per commit: only an exclusively AI-workflow/infrastructure purpose and change set requires the prefix; product or mixed commits forbid it.

The source-lineage gate runs before Phase 1 and explicit source requirements are passed to Start/Retry. A pre-Phase-1 mismatch returns a clean routing stop without integrating history or publishing a Block. A mismatch first established after Phase 1 follows the shared owned-state recovery and task-local Block path. Neither boundary authorizes cherry-pick, rebase, merge, backport or branch sync inside this command.

Phase delegation uses synchronous foreground Claude Agent calls with self-contained leaf prompts and no Bash-launched Claude subprocess. Review/assessment uses Codex CLI leaves per the Claude adapter unless the invocation explicitly opts out or a scheduler prompt selects `Review provider: native`; the opt-out is not task-name syntax.

## Dependencies, consumers and peers

Direct required dependencies:
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` — Claude host mechanics, single-writer/completion/recovery/provider/lineage/text rules.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` — canonical single-task state, phase, review/evidence and publication owner.

Direct host consumer:
- Claude Code command discovery/user `/perform-task` invocation.

Shared-skill scheduler consumer, not a caller of this wrapper:
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` delegates one selected task to the performer skill directly.

Peer host adapters, not callers:
- `.grok/commands/perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509`.
- `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.

Root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` exposes `perform-task <slug or full id>` as the repository-level one-task workflow entry. Root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` is only the pointer back to canonical `AGENTS.md`.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-PERFORM-TASK-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` — canonical one-task acquisition/resume/phase/review/evidence/final-publication lifecycle;
- `TDRP-R9-REPOSITORY-AI-WORKFLOW-HOST-ADAPTER-001` — Claude Agent/Codex/provider/single-writer/source-lineage host mechanics;
- repository quality/test-evidence governance.

Rejected owners:
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime/service;
- Claude-specific task state, queue, review, evidence or publication store;
- a duplicate performer pipeline or second acceptance truth.

No ADR, new shipping owner, service or platform-core production delta is required.

## Authored contracts — not run

- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-INVOCATION-SCOPE-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-DELEGATION-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-REVIEW-PROVIDER-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-COMMIT-SUBJECT-001`
- `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-SPLIT-PUBLICATION-001`
- `INT-TDRP-PERFORM-TASK-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-PERFORM-TASK-CLAUDE-PREPHASE-LINEAGE-001`
- `FAULT-TDRP-PERFORM-TASK-CLAUDE-DUPLICATE-WRITER-001`

All remain `validation_status=not-run`, `accepted=false`. No workspace helper, Agent, Codex leaf, source-lineage command, build, test, validator, reviewer, evidence campaign, UI driver, CI, publication simulation, packaging, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

Exactly one independent fixed-U source entry is represented by this dossier. Its wrapper-specific semantics are now mapped to the existing Fabushi performer + Claude host-adapter + quality/evidence owners without creating a second runtime/state owner. The next independent fixed-U blob in tree order is `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`.
