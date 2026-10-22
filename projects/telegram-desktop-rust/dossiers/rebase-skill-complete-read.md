# Rebase skill complete read — strict index 56

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict index 56: `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`, 322 lines, fully read.
- Parent exact-head TDRP evidence for index 55/rebaseline: Fabushi `7d81d55c7de65566dd2b8a2f62bc77ba28a10bd2`, run `39696604147`, 61/61 jobs success.
- Source-authority artifact: `12391321875`, digest `sha256:f00d374c367d90d723ef2e309d161f571121bd716e44636df98ab06b762d44cb`.
- Recursive reachability artifact: `12391447254`, digest `sha256:88d635be66019f79d7b0a859e8c12ca957af2aa4c1b1d12a9a7d32afb4cb170e`.
- Fresh child exact-head Actions are required after this mapping commit.

## Dependency reads
Read as dependency evidence only: `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120`, `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780`, upstream root `AGENTS.md@b138b4fbfae7a8d1e92d28c154e09eaca2418e51`, and `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376`. These reads do not close their own strict entries.

## Complete responsibility decomposition
- Hard rules (17–41): current-worktree-only ownership; no implicit push/force-push; no backup refs; no interactive editor; no whole-file ours/theirs; no hard-conflict skip; no task-queue routing.
- Host mechanics (42–56): fail-closed question handling plus environment-specific Git/line-ending mechanics; Fabushi executable acceptance remains Actions-only.
- Read request (57–102): resolve exact `ONTO`, `REPLAY_BASE`, `OLD_TIP`, replay list and conflict surface; prose/command disagreement and stale remote snapshots remain explicit.
- Preflight (103–115): resume existing rebase; dirty tree requires explicit choice; reflog is undo; rerere is reviewed evidence.
- Start (116–124): non-interactive rebase; merge-preservation choice is explicit.
- Read effort (125–156): bounded mechanical conflicts versus full history for overlapping logic; range-diff catches semantic drift.
- Per-conflict protocol (157–266): orient stages 1/2/3; reconstruct both intents; port behavior into current shape; stop for genuine contradiction; verify markers/final region/original change.
- Two traps (267–281): rerere must be inspected; empty commit may be skipped only after intent proof.
- Resume (282–293): reconstruct exact in-progress state; unknown state is reported, not silently aborted.
- Whole-rebase verify (294–308): `git range-diff REPLAY_BASE..OLD_TIP ONTO..HEAD` plus final log/status; executable Fabushi verification is exact-head Actions.
- Report (309–322): exact command, undo tip, replayed/skipped commits, conflict intents/resolutions, decisions and lower-confidence items; no automatic publication.

## Existing-owner-first
Selected owner: Fabushi root `AGENTS.md` Stage 6 — Integrate / Deliver. Process-inbox/task governance is rejected because the source explicitly classifies rebase outside that queue. Shipping product owners are rejected because Git history integration is repository state. No new owner is introduced.

## Platform adaptation
Windows/WSL routing, line endings and local translation-unit compile commands are upstream-environment mechanics. Fabushi retains exact Git identity, worktree ownership, intent-aware resolution, rerere/empty-commit safeguards, range-diff and recoverable reporting, while executable verification remains GitHub Actions-only.

## Traceability anchors
- `TDRP-R9-REBASE-INTENT-PRESERVATION-001`
- `ORA-TDRP-REBASE-RANGE-IDENTITY-001`
- `ORA-TDRP-REBASE-CONFLICT-HISTORY-001`
- `ORA-TDRP-REBASE-RANGE-DIFF-001`
- `INV-TDRP-REBASE-CURRENT-WORKTREE-ONLY-001`
- `INV-TDRP-REBASE-NO-IMPLICIT-PUBLISH-001`
- `INV-TDRP-REBASE-NO-TASK-OWNER-COLLISION-001`

Closing strict index 56 changes counters to unknown=16,247, unread=16,222, omitted=0, responsibilities=74. `baseline_identity_accepted=false` and `source_closure_ready=false`. Next strict root entry: index 57 `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`.
