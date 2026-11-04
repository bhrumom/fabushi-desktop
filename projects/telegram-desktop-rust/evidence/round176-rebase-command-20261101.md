# Round 176 — fixed-U Claude rebase command adapter

- Task round: 145
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `9bccccc5021abfb10a5f0dfe5c811191919e53e5`
- Spec-first desktop commit: `16e0dba10d422c65d06c0233ef68477fc00d51f3`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120`
- Required shared rebase authority: `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`
- Peer host: `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780`

Complete read confirms a thin Claude Code host wrapper over the existing source-neutral intent-aware rebase authority. The source is 1,681 Git bytes / 1,673 decoded characters / 36 newlines with trailing newline / 37 split lines, recursive-tree index 95 zero-based. It passes raw `$ARGUMENTS`, explicitly excludes the Claude AI workflow adapter, and leaves ONTO/REPLAY_BASE/OLD_TIP, conflict semantics, rerere/empty replay, resume/abort and publication boundaries with the shared skill.

Claude-only host contracts are blocking structured `AskUserQuestion` decisions, optional one-item-per-commit `TodoWrite` for long replay legibility, mandatory non-interactive `core.editor=true` / `GIT_SEQUENCE_EDITOR` mechanics, no `Agent` delegation, and scratchpad-only stage/report data. Direct host consumer is Claude Code/user `/rebase`; the shared skill explicitly reads this wrapper in Claude Code. Grok is a peer adapter, and the generic Claude AI workflow adapter is a negative dependency.

Existing-owner-first reuses `TDRP-R9-REPOSITORY-INTENT-AWARE-REBASE-AUTHORITY-001` plus repository quality/evidence governance and introduces no second rebase state store, AI task/phase publication path, shipping Product Tasks/Automations owner, Agent/Coordinator/Host/Runner runtime owner, platform-core service, autonomous Git publication owner or second acceptance truth.

Authored/not-run: seven contract cases, one integration case, two fault-recovery cases and one security case. Unique ledger after closure: `source_entries_total=16202 / unknown=16132 / unread=16105 / omitted=0 / responsibilities_total=146 / unmapped_responsibilities=0`; implementation `implemented`, validation `not-run`, accepted `false`.

Next fixed-U tree blob: `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no Git rebase/fetch/range-diff, Todo/Agent execution, build, compile, typecheck, lint, validator, reviewer, test/evidence campaign, UI driver, publication simulation, packaging, deployment, installation, runtime evidence or acceptance was performed.
