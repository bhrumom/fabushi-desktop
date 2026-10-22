# Claude release command complete read — strict index 67

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Source: `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d`, 120 lines / 5822 characters; complete read and SHA revalidated.
- Dependencies read: `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`; `Telegram/build/set_version.bat@7e7c3f69e8706905032e58cc05dcc25802dccebe`; `Telegram/build/set_version.py@305a2a5f37aa13c8bef29d13ea37cab7a776cde6`.
- Existing owners: `AGENTS.md@4f888d465d29d962542da07bcb6f6efe2b5be897` Stage 6; `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` version/package/publish metadata; `FQT-REL-001@c5eb9d0897234bb2fdad4f505549b2350c44aa72`.
- Parent exact head/tree: `17cbe4d71ce710895466949adad9e52eafed039a` / `2e9c9d445723cadee97e0e4efb0ad8cdf2c9c9df`.
- Parent TDRP run `39725439132` completed success, 61/61; authority `12403141162` / `sha256:f7c555318af709ed801121c6d89f375248093344cd74fbc01f8dd0472a8ed445`; closure `12403550709` / `sha256:2d860a64d0ae3cdd4d90a03e9650823db689d20ca422d696f7f019ded658443c`; reachability `12402679596` / `sha256:47df356257a960edc496254431113c894f93e7a346d775a9d503b7af46691797`.
- Parent Rust run `39725438916` completed success, 6/6; artifact `12402637306` / `sha256:4173e4a3f892a895e558365da9e527084b0d08d1fdf0cfbb6d5660c886ba9446`.

## Existing-owner-first disposition
The command is repository release preparation, not shipping runtime. Map clean exact authority, non-reused version sequencing, exact release interval, user-facing release notes, human approval fences, deterministic version metadata mutation and preparation-commit provenance into the existing Stage 6 Integrate / Deliver owner. Keep `desktop/package.json` as version/package metadata authority and FQT-REL-001 as independent release acceptance. Telegram changelog/set_version mechanics and Claude syntax remain provenance only. No second release/version owner, updater runtime, task queue or auto-publish path is introduced; `novel_capability=false`.

## Traceability
Requirement `TDRP-R9-CLAUDE-RELEASE-COMMAND-001`; oracles cover clean authority, version sequence, exact release range, approval fence and commit provenance; invariants forbid version reuse, historical-note rewrite, pre-approval mutation, auto-publish and Telegram release runtime; contracts cover version authority, release notes, human approval and separation from actual release; fault cases cover dirty state, duplicate version, partial mutation and commit mismatch; security forbids unauthorized tag/publish bypass.

Closing strict index 67 changes counters to unknown=16,236, unread=16,211, omitted=0, responsibilities=85. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 68 `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79`.

### Exact traceability IDs
- Requirement: `TDRP-R9-CLAUDE-RELEASE-COMMAND-001`
- Oracles: `ORA-TDRP-RELEASE-COMMAND-CLEAN-AUTHORITY-001`, `ORA-TDRP-RELEASE-COMMAND-VERSION-SEQUENCE-001`, `ORA-TDRP-RELEASE-COMMAND-RELEASE-RANGE-001`, `ORA-TDRP-RELEASE-COMMAND-APPROVAL-FENCE-001`, `ORA-TDRP-RELEASE-COMMAND-COMMIT-PROVENANCE-001`
- Invariants: `INV-TDRP-RELEASE-COMMAND-NO-VERSION-REUSE-001`, `INV-TDRP-RELEASE-COMMAND-NO-HISTORICAL-RELEASE-NOTE-REWRITE-001`, `INV-TDRP-RELEASE-COMMAND-NO-PREAPPROVAL-VERSION-MUTATION-001`, `INV-TDRP-RELEASE-COMMAND-NO-AUTO-PUBLISH-001`, `INV-TDRP-RELEASE-COMMAND-NO-TELEGRAM-VERSION-RUNTIME-001`
