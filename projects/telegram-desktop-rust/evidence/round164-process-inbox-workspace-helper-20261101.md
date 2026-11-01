# Round 164 — fixed-U Process Inbox workspace helper

- Task round: 133
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `fbc9949492825cad7b959bfa2750a07f53973396`
- Spec-first desktop commit: `3d3d38156ad6327e37408013745b77c003ce4e02`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs
- Closed source: `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`
- Source identity: recursive-tree index 80 zero-based; 185,107 Git bytes/decoded characters; 6,122 newline characters with trailing newline; 6,123 split lines; 221 class/function declarations

The exact helper was read across repository identity, task state, source preparation/lineage, local recovery refs, owned-path source commit, test/evidence support, split/consolidation, routing/code-health, publication, stale-project archive, and inbox transaction families. The portable responsibility is repository-process authority, not shipping Fabushi runtime.

Identity/worktree contracts bind one source checkout and stable machine/checkout tag to one canonical AI repository plus isolated slot, inbox, routing and code-health linked worktrees. Common Git directory, exact branch, cleanliness and unpublished-count fences fail closed foreign, moved, dirty or unsafe worktrees.

Task/source contracts preserve canonical published state. Start atomically claims only eligible todo work after dependency/source-lineage gates; Retry is limited to current-checkout blocked work; competing active work is rejected. Approved source-producing prerequisites require retained task commits reachable from the current source history. Local base/green/run refs and carried-work seals are resumable recovery metadata, not task truth.

Inbox transactions are sealed by SHA-256 over sorted relative payload paths and exact bytes; backup is excluded and symlinks are rejected. Prepare resumes exactly one matching transaction or creates an immutable snapshot for nonempty input. Publication validates transaction identity, digest-bearing receipt and explicit normalized paths, then refreshes/rebases and reruns semantic validation. Only ordinary non-fast-forward/fetch-first races retry; conflicts/outages preserve recoverable work. The optional private ai-checkout force mirror is recovery-only and does not authorize canonical force publication.

Finalize requires the exact receipt/digest on canonical master, archives the snapshot, and clears live input only when its digest is unchanged; human edits remain intact. Abort retains a failed pre-publication snapshot. Approval refs are deleted only when canonical approval and source invariants all agree.

Direct fixed-U consumers include process-inbox, perform-task, continue and code-health skills; performer pipeline; split/consolidation references; shared build-lock recovery; Claude workflow adapter; and `workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`, which directly imports the helper. The test source was not executed and remains the next independent closure item.

Existing owner remains `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`. No new responsibility, ADR, shipping product owner or platform-core production delta was introduced.

Authored/not-run contracts: `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-IDENTITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKTREE-ISOLATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-TRANSACTION-DIGEST-001`, `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-TRANSITION-001`, `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-REFS-001`, `INT-TDRP-PROCESS-INBOX-WORKSPACE-PUBLICATION-001`, `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-RECOVERY-001`.

Unique-ledger closure: `source_entries_total=16202 / unknown=16144 / unread=16117 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`. Implementation state remains `implemented`, `validation_status=not-run`, `accepted=false`.

Next independent fixed-U blob: `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no workspace helper/test execution, build, typecheck, lint, validator, generator, dependency resolution, source-lineage command, Git publication simulation, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deploy, install, runtime evidence or acceptance was performed.
