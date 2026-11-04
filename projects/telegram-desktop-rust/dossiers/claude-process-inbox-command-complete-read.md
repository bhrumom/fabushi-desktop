# Fixed-U complete read — Claude process-inbox command adapter

## Authority snapshot

- Desktop checkpoint parent: `be48eebd8b1a4b35cced47225dc00fab64e0a9fb`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`, recursive-tree index 94 zero-based.
- Exact source read completely: 1,761 bytes / 1,761 decoded characters / 39 newline characters / trailing newline / 40 split lines.
- Next fixed-U blob: `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120`.

## Complete source semantics

The command is a Claude Code host wrapper for the repository's ignored AI inbox. It completely loads the generic Claude workflow adapter and shared process-inbox skill, then applies only Claude-specific delegation/text-handling substitutions. It routes and plans requests into durable AI task/project/receipt artifacts; it must not implement, build, test, claim or start those tasks. `$ARGUMENTS` supply additional routing hints only.

Project continuity is preferred. A request derived from existing project work remains in that project unless the shared affirmative independence test proves it coherent, implementable and independently testable without the originating project. Shared/cross-surface files alone do not make the request standalone. A request whose only purpose is moving an existing commit across branches gets a receipt-only disposition; no backport, forward-port, cherry-pick, rebase, merge or branch-sync task is created. Product work that depends on an existing source task records that dependency directly.

Parallel task execution does not block inbox routing. The command uses the checkout-specific `inbox_worktree`, `inbox_branch` and shared `inbox-*` helper operations and never substitutes the task `slot_worktree`, even if that slot is dirty. The helper owns transaction snapshot/digest, isolated tracked writes, inbox commit, fetch/rebase/master publication, finalization and resumable recovery. Broad Git staging and force-push are outside the contract.

A planner Agent, when used, must read the Claude adapter before the shared skill instructions, must not delegate, and may only propose routing. The orchestrator owns tracked AI writes and publication. Claude subprocesses are not launched through Bash. The Claude text-handling exception removes the shared skill's explicit tracked-text line-ending/BOM validation item only; all content/routing/publication checks remain.

Publication writes explicit task/project/receipt paths through the helper. Finalize is allowed only after the exact receipt/digest is present on master; it preserves the raw snapshot under ignored backup and clears the live inbox only if the digest did not change. Semantic rebase conflict, unsafe worktree, remote outage or an existing unpublished inbox commit leaves the transaction recoverable. Abort is allowed only before tracked changes/commits exist. Final reporting gives friendly task/project results, publication status, backup path, clear status and unused input without exposing a commit hash or automatically starting implementation.

## Dependencies, consumers and peers

Direct required dependencies:
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` — Claude delegation/single-writer/text/recovery mechanics.
- `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d` — canonical transaction, routing, task/project/receipt, publication/finalization/recovery owner.

Direct host consumer:
- Claude Code command discovery/user `/process-inbox` invocation.

Shared-skill scheduler consumer, not a caller of this wrapper:
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` may spawn one inbox worker, instructing that worker to read the shared process-inbox skill directly and return the receipt/created ids.

Peer host adapters, not callers:
- `.grok/commands/process-inbox.md@5fcd9055de9b14a5163dc96a6c3551096898de18`.
- `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`.

Root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` says ordinary conversational work must not be routed through process-inbox/continue unless the user asks for that workflow. Root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` is only the pointer back to canonical `AGENTS.md`.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-PROCESS-INBOX-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` — already carries the shared process-inbox skill/helper source set and repository queue/process publication semantics;
- `TDRP-R9-REPOSITORY-AI-WORKFLOW-HOST-ADAPTER-001` — Claude Agent/single-writer/text/recovery host mechanics;
- Fabushi repository quality/test-evidence governance.

Rejected owners:
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime/service;
- Claude-specific task/project queue or state store;
- Claude-specific inbox publication/finalization store;
- duplicate process helper, Git publication pipeline or second acceptance truth.

No ADR, new shipping owner, service or platform-core production delta is required.

## Authored contracts — not run

- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-INVOCATION-ROUTING-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-PROJECT-CONTINUITY-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-WORKTREE-ISOLATION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-DELEGATION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-PUBLICATION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-FINALIZE-DIGEST-001`
- `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-RECOVERY-001`
- `INT-TDRP-PROCESS-INBOX-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-PROCESS-INBOX-CLAUDE-PUBLICATION-RACE-001`
- `FAULT-TDRP-PROCESS-INBOX-CLAUDE-INPUT-MUTATION-001`

All remain `validation_status=not-run`, `accepted=false`. No process-inbox helper, planner Agent, Git fetch/rebase/push, build, test, validator, reviewer, evidence campaign, UI driver, CI, publication simulation, packaging, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

Exactly one independent fixed-U source entry is represented by this dossier. Its Claude-wrapper semantics are mapped onto the existing repository process + Claude host-adapter + quality/evidence owners without creating a second queue, state owner or publication pipeline. The next independent fixed-U blob in tree order is `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120`.
