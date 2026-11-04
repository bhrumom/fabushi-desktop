# Round 172 — fixed-U Claude cross-review command

- Task round: 141
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `f21130c8dd374d4dc5101ea8ca92f229224e90c6`
- Spec-first desktop commit: `91a8a6a5134c44bfd10441a0b7719c9822322972`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`
- Required protocol: `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27`
- Required prompts: `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d`
- Required worker contract: `.agents/shared/codex-cli-leaves.md@11c59b68df43e442e596c3052296fc51a044d46a`
- Direct helpers: `.agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`, `.agents/shared/scripts/cross_review_scope.py@d9fafdf9fa0857cc83f0ccb49d15d4fad8782768`

Complete read confirms a Claude host adapter over already-canonical source-neutral cross-review governance. It freezes or resumes one durable scope/session, requires the independent Codex provider, runs fresh read-only general/lens reviewers with exact foreground waits and bounded retry, resumes the general reviewer for synthesis, records every arbitration decision before edits, and re-reviews accepted fixes until a protocol stop.

The generic Claude workflow adapter is not loaded. Cross-review therefore preserves its own stronger provider boundary: Codex admission failure reports and stops rather than using a native/Claude fallback. Twice-failed lens review is retained as an explicit coverage gap; twice-failed general review stops. Review-only has no edits; resume continues the same session's first incomplete step.

Direct host consumer is Claude Code command discovery/user `/cross-review`. The shared protocol's reverse reference names this wrapper as the current runner. Existing-owner-first maps the responsibility to root repository process authority, existing independent cross-review protocol/snapshot/prompt owners, quality/test-evidence ledger, and external AI-worker lifecycle; no shipping runtime, service, ADR, platform-core delta, or second session/findings/arbitration store is introduced.

Authored/not-run: eight contract cases, one integration case and two fault-recovery cases covering host authority, invocation/scope, provider fence, delegation/wait, report routing, arbitration/fix boundary, resume/recovery, build/hook deferral, composition, leaf failure and resume fencing.

Unique ledger after closure: `source_entries_total=16202 / unknown=16136 / unread=16109 / omitted=0 / responsibilities_total=142 / unmapped_responsibilities=0`; implementation `implemented`, validation `not-run`, accepted `false`.

Next fixed-U tree blob: `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no executable migration validation was performed.
