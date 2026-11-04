# Fixed-U complete read — Claude rebase command adapter

## Authority snapshot

- Task round: 145.
- Source evidence round: 176.
- Date: 2026-11-01.
- Desktop checkpoint parent: `9bccccc5021abfb10a5f0dfe5c811191919e53e5`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120`, recursive-tree index 95 zero-based.
- Exact source read completely: 1,681 Git bytes / 1,673 decoded characters / 36 newline characters / trailing newline / 37 split lines.
- Required shared authority read completely: `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`.
- Peer host adapter read: `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780`.
- Next fixed-U tree blob: `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d`.

## Complete source semantics

The fixed-U Claude file is a host adapter, not a second rebase implementation. It requires the shared intent-aware rebase skill to be read completely, passes the user's raw `$ARGUMENTS` request into that authority, and explicitly says not to read `.claude/ai-workflow-adapter.md`. The exclusion is semantic: branch integration is not AI queue work, so task records, phase leaves and AI publication do not apply.

The shared skill therefore retains the complete source-lineage and rebase lifecycle: resolve `ONTO`, exclusive `REPLAY_BASE` and pre-operation `OLD_TIP`; show the exact replay list and overlap surface; fence sibling worktrees, dirty state, ambiguous replay intent and stale remote snapshots; resume rather than nest an in-progress rebase; orient stage 2 as ONTO and stage 3 as the replayed commit; reconstruct both sides' intent for overlapping logic; re-review rerere; positively prove empty-commit equivalence; preserve reflog recovery; and verify the finished series semantically without pushing.

Claude-specific substitutions are narrow. Where the shared skill asks for a decision, the wrapper uses `AskUserQuestion`, states both intents and offers two to four concrete resolution directions. The question is a hard blocking boundary: the rebase must not continue before an answer. For a replay list longer than a handful of commits, `TodoWrite` may mirror one item per commit so the session remains legible, but Todo is not lineage, persistence, recovery or acceptance state.

Claude Bash has no interactive terminal, so every `rebase --continue` must use `git -c core.editor=true`, and every todo-list edit must use a non-interactive `GIT_SEQUENCE_EDITOR`. A command that opens an editor is a host failure that leaves the operation recoverable; it does not authorize bypassing a decision or resetting state.

The adapter deliberately has no `Agent` tool and forbids delegation. Conflict resolution requires both sides' history in one context; a delegated summary cannot become semantic authority. Stage dumps and the running report remain in the session scratchpad and are never written into the repository. Recovery/publication remain shared-owner boundaries: reconstruct/resume or explicit abort from Git metadata/reflog, never autonomous fetch/push/force-push, never AI queue publication, and never a second durable rebase store.

## Consumers, dependencies and peers

Direct host consumer:
- Claude Code command discovery/user `/rebase` invocation.

Shared-authority composition consumer:
- `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430` explicitly reads `.claude/commands/rebase.md` when running in Claude Code. This is a host-substitution back-reference, not a competing state owner.

Peer host adapter:
- `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780` reads the same shared skill but uses Grok-specific question/subagent mechanics. It is not a caller of the Claude wrapper.

Repository instruction pointer:
- `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` points Claude Code to canonical `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`.

Negative dependency:
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` is intentionally not loaded. Its task/phase/publication pipeline is outside branch integration.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-REBASE-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- `TDRP-R9-REPOSITORY-INTENT-AWARE-REBASE-AUTHORITY-001` — source-lineage, conflict, recovery and no-push rebase semantics;
- Fabushi repository development-process governance in `AGENTS.md`;
- repository quality/test-evidence governance in `TEST_STRATEGY.md` and `evidence-contract.md`.

Rejected owners:
- Claude AI workflow/task/phase/publication adapter;
- canonical continue/perform-task/process-inbox state machines;
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime/service;
- Claude-specific rebase queue/store, scratch store or publication owner;
- a second acceptance truth.

No ADR, new shipping owner, service or platform-core production delta is required.

## Authored contracts — not run

- `CONTRACT-TDRP-REBASE-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-INVOCATION-LINEAGE-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-DECISION-GATE-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-NONINTERACTIVE-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-DELEGATION-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-SCRATCH-001`
- `CONTRACT-TDRP-REBASE-CLAUDE-RECOVERY-PUBLICATION-001`
- `INT-TDRP-REBASE-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-REBASE-CLAUDE-UNANSWERED-CONFLICT-001`
- `FAULT-TDRP-REBASE-CLAUDE-EDITOR-RECOVERY-001`
- `SEC-TDRP-REBASE-CLAUDE-NO-AI-PUBLICATION-001`

All remain `validation_status=not-run`, `accepted=false`. No rebase, fetch, range-diff, Todo/Agent execution, compile, build, typecheck, lint, validator, reviewer, test/evidence campaign, CI, publication simulation, packaging, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

Exactly one independent fixed-U source entry is represented by this dossier. The Claude wrapper maps onto the existing shared rebase + repository quality/evidence owners and introduces no second state machine or publication path. The next independent fixed-U blob in tree order is `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d`.
