# Fixed-U complete read — intent-aware rebase skill

- Task round: 135
- Source evidence round: 166
- Date: 2026-11-01
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs
- Source: `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`
- Tree index: 83 zero-based
- Identity: 15,114 Git bytes / 15,036 decoded characters; 321 newline characters; trailing newline; 322 split lines; 12 headings
- Desktop parent at read: `6e8822013bbfc6bd7222381525eb802133d70317`
- Desktop main at read: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete behavior read

The fixed-U skill owns one intent-aware branch rebase in the current checkout. It accepts current-branch, named-branch, `--onto` tail, upstream-default, resume/continue and abort forms. It resolves the request to `ONTO`, exclusive `REPLAY_BASE` and pre-operation `OLD_TIP`, prints the actual replay list and overlapping-path surface, and treats a user's literal command as evidence to verify rather than authority to obey blindly. Remote-tracking refs are local snapshots; freshness requires an explicit fetch decision.

Preflight refuses sibling-worktree branch ownership and dirty-tree mutation, resumes an already-running rebase instead of nesting one, records `OLD_TIP` as the reflog undo authority, and treats rerere as a hazard requiring review. It never creates backup branches/tags, never pushes or force-pushes, and never routes the operation through ai-tdesktop task creation, perform-task, continue or process-inbox.

Conflict handling is intent-first. Stage 2 is the ONTO side and stage 3 is the replayed commit. Mechanical adjacent additions can be resolved from the local region; overlapping logic requires base/ONTO/replayed stages, replayed-series history and relevant upstream history. Both intents must be stateable before editing. The resolution reapplies the replayed behavior in the current upstream shape, does not resurrect deleted structure merely to host an old patch, does not use blanket ours/theirs, does not skip a hard conflict, and asks on a genuine semantic contradiction. Rerere output is re-reviewed as a pre-filled conflict. Empty commits may be skipped only after end-state equivalence is positively established.

Resume reconstructs the running state from Git metadata and replay history. Unknown state is preserved and reported instead of auto-aborted. Explicit abort restores the pre-operation tip and stops. Final verification is semantic: per-conflict final-region/original-change accounting plus whole-rebase range-diff, final replay log and status; any disappeared or changed mapping requires explanation. Build/compile evidence is supplementary and belongs to the applicable later validation campaign.

Publication is deliberately external to the skill. A rewritten published branch may need force publication, but the rebase owner only reports that fact and never performs the push itself.

## Direct callers / consumers

Direct fixed-U host callers:

- `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` — explicitly reads the skill completely; uses structured questions; forbids the normal Claude AI workflow adapter; keeps decisions local; stores scratch only outside the repository.
- `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780` — explicitly reads the skill completely; uses numbered questions; forbids the normal Grok AI workflow adapter and subagent decision-making.

Dependencies/boundaries:

- `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` — checkout/host/task semantics and repository instructions.
- `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376` — style guidance when conflict resolution requires new glue.
- `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` — boundary consumer: source-lineage failure must not be converted into branch integration inside a performer task.

No `.agents/skills/rebase/agents/openai.yaml` exists at fixed U.

## Existing-owner-first resolution

This is a new responsibility but not a new owner. It maps to the existing Fabushi repository development-process authority in `AGENTS.md` plus existing repository quality/test-evidence owners in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `evidence-contract.md`.

Rejected owners:

- canonical continue/perform-task/process-inbox state machines, because the source explicitly excludes ai-task routing and branch integration inside a task;
- shipping Product Tasks/Automations and Agent/Coordinator/Host/Runner, because this is repository history integration, not product execution;
- platform-core production runtime, because no shipping runtime behavior is involved;
- a Telegram-specific rebase runtime or second task/publication store, because source-neutral repository governance already owns this domain.

No ADR or platform-core production change is required.

## Authored / not-run contracts

- `CONTRACT-TDRP-REBASE-SCOPE-LINEAGE-001`
- `CONTRACT-TDRP-REBASE-CONFLICT-INTENT-001`
- `CONTRACT-TDRP-REBASE-RERERE-EMPTY-001`
- `CONTRACT-TDRP-REBASE-RESUME-ABORT-RECOVERY-001`
- `CONTRACT-TDRP-REBASE-NONINTERACTIVE-HOST-001`
- `INT-TDRP-REBASE-HOST-COMPOSITION-001`
- `INT-TDRP-REBASE-PUBLICATION-BOUNDARY-001`
- `FAULT-TDRP-REBASE-UNKNOWN-CONTRADICTION-001`
- `SEC-TDRP-REBASE-NO-FORCE-PUBLISH-001`

FMEP-001 Stage A remains strict: these are authored contracts only. No rebase, fetch, range-diff, source-lineage operation, Git publication simulation, build, compile, test, validator, reviewer, CI, package, deployment, installation, runtime evidence or acceptance was executed.
