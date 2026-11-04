# Complete read — fixed-U process-inbox workspace regression suite

Date: 2026-11-01  
Task round: 134  
Stage: A / zero executable validation

## Authority snapshot

- Desktop checkpoint parent: `24d2801162c92226483a578e2b1f2ce84cfa0967`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact authority: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`, recursive-tree index 81 zero-based.
- Exact source: 214,649 Git bytes / decoded characters, 6,447 newline characters, trailing newline, 6,448 split lines, 254 class/function declarations, 10 class declarations, 140 `test_*` methods.

## Requested regression-contract read

The test source directly imports `workspace` and is the executable regression consumer of `workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`. It ends with `if __name__ == "__main__": unittest.main()`, so it can run standalone; it does not define a second task/process state owner.

Identity and worktree-isolation cases prove that inbox setup creates only its inbox worktree, not the task slot; a dirty active task slot remains untouched through prepare/publish/finalize; source checks ignore only registered linked worktrees while general workspace cleanliness remains strict; moved/unregistered repositories, a symlink to a linked worktree, submodule-target overlap and staged/local source edits fail closed. Recursive source preparation follows recorded gitlink pins rather than an unrelated remote tip and preserves registered worktree content.

Transaction/publication cases bind a prepare transaction to the current checkout, carry the prepare-produced digest into the tracked receipt, reject broad publication roots, retain literal deletion/non-ASCII path identity and retry only fetch-first/non-fast-forward push races. A foreign-checkout transaction is rejected before reuse. Finalize rejects a receipt path outside canonical state and a receipt that exists locally but is not in canonical master HEAD; the normal inbox lifecycle proves successful publication, processed backup retention and clearing only of the unchanged live inbox.

Task-state cases cover durable alias resolution, dependency graph validity, owned blocked Retry, atomic Start, no mutation when required source ancestry is absent, local-only checkpoint, Start/Approve as the only normal published task transitions, refusal to compete with another active task, batch metadata without pre-claiming, unknown batch ids rejected before mutation, and lost Start rollback. Queue output retains actual claimed owner/status rather than treating batch membership as ownership.

Source-lineage cases distinguish queue dependency readiness from source ancestry. They exercise missing source-producing dependency history, compatible local branches, already-satisfied outcomes, retained task commits that precede later independent work, and source-begin creation/recovery of local base/green/run refs. Source-preparation/preflight additionally proves linked worktrees do not contaminate task-owned dirty-path and overlay/carry snapshots.

Publication and recovery cases constrain source commits to owned paths, preserve Task trailers and green/run refs, keep a successful task commit even when the optional private recovery mirror is missing/unreachable, and demonstrate that the mirror force-updates only the dedicated `ai-<checkout-tag>` recovery branch. Split publication transfers retained carried work only through the designated implementation carrier. Approval publication retains source refs through fetch failure, dirty/unsafe source, foreign/non-approved canonical state or ref mismatch; refs are removed only after canonical approval and source invariants are restored, and rerun is idempotent.

## Exact coverage boundary

The exact blob contains six direct `workspace.command_prepare` calls, three `command_finalize` calls, eleven `command_start` calls, two `command_retry` calls, three `command_inbox_publish` calls, one `command_publish` call and three `source_lineage_report` calls. It contains zero direct `workspace.command_abort` calls and does not directly invoke a transaction/payload/inbox digest helper. It uses prepare-produced digests in receipts, so it validates binding at the workflow boundary but does not independently prove the digest implementation.

Accordingly, the source test suite is not credited with an abort regression it does not contain. The previously authored helper-level `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-TRANSACTION-DIGEST-001` and `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-RECOVERY-001` remain required, and this round adds an explicit finalize/abort regression contract so Stage B cannot infer missing coverage as a pass.

## Direct callers and consumers

- Direct dependency/subject: `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d` via `import workspace`.
- Direct execution boundary: the file's own `unittest.main()` entrypoint.
- The exact fixed-U process-inbox, perform-task, continue and code-health skills, performer pipeline, split/consolidation references and shared build-lock document surveyed in this closure contain no direct `workspace_test` / `workspace_test.py` reference.
- The suite therefore remains a standalone repository regression consumer. No production shipping entrypoint, Telegram product runtime or platform-core runtime consumes it.

## Existing-owner resolution

Existing owner remains `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus repository quality/test-evidence governance. The test source strengthens that owner's regression oracle; it does not create a new responsibility, state store or runtime.

Rejected duplicate/different owners remain shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, product Storage/Recovery, platform-core production runtime, a Telegram-specific workspace scheduler, and any second repository task/receipt/lineage/evidence owner. No ADR and no platform-core production delta are required.

## Fabushi authored contracts

Authored, not run:

- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-IDENTITY-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-WORKTREE-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TRANSACTION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TASK-STATE-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-FINALIZE-ABORT-001`
- `INT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-PUBLICATION-001`
- `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-RECOVERY-001`

No workspace helper/test execution, source-lineage command, Git publication simulation, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

This regression source maps into the existing repository process/quality owner with no new responsibility count. Stage A implementation status remains `implemented`, executable validation remains `not-run`, and acceptance remains false. The ledger/source-closure counters are updated only in the subsequent closure commit after this spec-first contract commit is published by non-force expected-SHA CAS.
