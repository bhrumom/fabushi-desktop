# Fixed-U complete read — process-inbox skill

Date: 2026-11-01  
Task round: 130  
Stage: A / zero executable validation

## Authority

- Desktop checkpoint parent: `cfbdce82ae388fa763205e74590f73f52dcfe029`.
- Desktop main authority: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core authority: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed Telegram Desktop source: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Recursive fixed-U tree: `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`, recursive-tree index 75.
- Exact source size: 22,475 Git bytes / 22,467 decoded characters / 441 newline-delimited lines.

## Complete read

The complete source was read from front matter through the final report contract. Its sections are Workspace; Route and plan; Assign task paths; Write tracked artifacts; supplied-video handling; Validate and publish; and Report.

The source defines one repository-development intake transaction. It locates the canonical AI repository and a checkout-specific isolated inbox worktree from the source checkout/machine tag, resumes exactly one matching active transaction, snapshots the complete ignored inbox with a digest, and requires new transactions to start from clean publishable canonical/inbox state. The task-execution slot is deliberately excluded from reads, writes, staging, rebases and publication.

Routing is project- and lineage-aware. It reads the transaction snapshot rather than the mutable live inbox, selectively expands source/project/task context, resolves durable consolidation aliases and split records to live successors, and treats retired dated paths as permanently occupied. Project continuity wins unless the request is demonstrably coherent, implementable and independently testable with the originating project absent. Pure branch-history operations are receipt-only. Task splitting follows independently useful shipped outcomes and direct acceptance oracles, while inseparable API/caller changes stay together.

Every created task is a dated collision-safe `type: implement` record with a self-contained `task.md`, outcome-focused acceptance, durable copied inputs and the exact pristine `state.yaml` lifecycle. New tasks are always `todo`, unclaimed, without phase/lease ownership and without a completion `model`. Dependencies encode shipped source lineage as well as queue readiness; no backport/cherry-pick/rebase/merge task is created to manufacture reachability. Q0/Q1/Q2 code-health requests remain one standalone task exactly as pre-sized.

Initial inbox sizing is only the first scope gate. A later performer may publish `split-required` after deeper code/evidence assessment, but the performer does not mutate the queue; the continue scheduler owns replacement creation, dependency/project rewrites and source retirement. Inbox deduplication must follow existing `superseded.yaml` and `split.yaml` history and never create new live references to a retired alias/split id.

Supplied videos stay local. Planning may use probe/extract/sheet tooling to derive timing and compact frame evidence, but the original video and full extraction are not tracked. Tracked planning also excludes raw inbox paths, portable accounts, browser profiles, credentials, complete run directories/build logs and persisted Telegram/AI commit hashes.

Publication is explicit-path only and receipt-bound. The helper validates the transaction checkout/repository, receipt digest and changed-path allowlist; stages only declared task/project/receipt roots; commits the isolated inbox branch; refreshes/rebases against canonical master; retries ordinary non-fast-forward races; and preserves a recoverable commit on semantic conflict or remote outage. Finalization requires the exact receipt/digest on canonical master, always preserves a local backup, and clears the live inbox only if its current payload digest still matches the snapshot. Abort is only for failure before tracked commit creation; after a generated inbox commit exists, the transaction is retained for publication recovery.

## Direct fixed-U callers and consumers

- Root repository contract: `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`.
- Claude command wrapper: `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`.
- Grok command wrapper: `.grok/commands/process-inbox.md@5fcd9055de9b14a5163dc96a6c3551096898de18`.
- OpenAI host adapter: `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`; it remains the next independently unclosed source blob after this round.
- Continue scheduler: `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`.
- Performer boundary: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
- Split routing: `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`.
- Consolidation: `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`.
- Shared helper: `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`.
- Host adapters: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` and `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154`.

## Existing-owner resolution

Existing owner: `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` / Fabushi repository development process authority + repository quality/test-evidence governance.

This source adds inbox acquisition, immutable snapshot, project/lineage-aware routing, pristine task-state creation, split/consolidation interoperability, explicit-path publication, finalization and failure-recovery detail to the already established source-neutral repository task/process owner.

Rejected duplicate/different owners: shipping Product Tasks / Automations; Agent / Coordinator / Host / Runner runtime; platform-core production runtime; a Telegram-specific inbox queue or scheduler; and a second receipt, ownership or acceptance truth.

No new ADR or platform-core production delta is required.

## Stage-A authored contracts

- `CONTRACT-TDRP-PROCESS-INBOX-ACQUISITION-SNAPSHOT-001`
- `CONTRACT-TDRP-PROCESS-INBOX-ROUTING-LINEAGE-001`
- `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-SCHEMA-001`
- `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-PROCESS-INBOX-SPLIT-CONSOLIDATION-INTEROP-001`
- `INT-TDRP-PROCESS-INBOX-PUBLICATION-FINALIZE-001`
- `FAULT-TDRP-PROCESS-INBOX-RECOVERY-001`

All are authored only. No helper invocation, inbox transaction, source-lineage command, build, typecheck, lint, validator, generator, dependency resolution, test, reviewer, UI driver, package, deploy, install, runtime evidence or acceptance was executed.

## Closure disposition

The fixed-U source is fully understood and maps to an existing owner. The ledger may close exactly this independent source entry as implemented/workflow-equivalent with `validation_status=not-run` and `accepted=false`; executable qualification remains Stage B.
