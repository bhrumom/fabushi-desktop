# Needs-user-action issue-maintenance workflow complete read — strict index 91

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/needs-user-action.yml@55ad7891ce36682accae01d739203aa487876053`, 17 line slots / 312 bytes; complete read and SHA revalidated before publication.
- Dependency: `lee-dohm/no-response@v0.5.0` resolves to commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`; `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a`; `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`; tree `8600ce78cc9ed1dad710f332ac434cdf1d49fc0d` contains only those two blobs and no LICENSE/package manifest/package lock.
- Existing canonical defect-closure owner: `FQT-DEF-001@b31ecd424cac2abf8a2322e2996bb1b53c561378`.
- Parent exact head/tree: `acd975db4f2db36c6a5b1d633453a1a3ba3398c2` / `ec716fe46e4edb383413712f4acc92c1609df380`.
- Parent exact-head acd975db4f2db36c6a5b1d633453a1a3ba3398c2 completed TDRP run 39886008535 at 61/61 success (authority 12472536855 sha256:3c8b0ba0ab91341142ea2e25594e4aa9b2f2c460faafe7149feaff00cf0c14cf; closure 12472802476 sha256:ec3a9d6ff5f7d6174b5c6b60e7f62b38ad505509b5ed3e0471a8c85c66fe6f44; reachability 12472727917 sha256:e73aff0b73ca60843e488f01c80e976b06aea807482c2010a421d3b5a9381834) and Rust desktop runtime run 39886008486 at 6/6 success (artifact 12471609701 sha256:5da1450c1745f7a951aae94c33889e02c48060403b52e67a8a6dd77fef71a771).

## Complete-read behavior
The workflow is repository issue maintenance, not defect adjudication. It subscribes both to `issue_comment.created` and a daily 02:00 schedule, then invokes `lee-dohm/no-response@v0.5.0` with `github.token` and `responseRequiredLabel: needs user action`.

The dependency defaults `daysUntilClose` to 14. On `schedule`, `sweep()` ensures the label exists, searches open issues carrying that label with `per_page: 30`, finds the latest matching labeled event, filters by the 14-day cutoff, and attempts the default explanatory comment plus provider issue close. The sweep loop invokes `close()` without awaiting the returned Promise, so a green action cannot establish that every comment/close mutation settled successfully.

On `issue_comment`, `unmark()` reads the event payload and current issue. It removes the label only when the issue still carries it and the commenter is the original issue author. If that issue is closed and `closed_by` differs from the original author, it requests reopen; that update is also not awaited at the call site. Non-author comments do not satisfy the response state.

## Existing-owner-first disposition
`FQT-DEF-001` remains the sole canonical defect-closure owner. Its normative rule is `reproduce -> fix -> targeted regression -> affected regression -> current-head evidence` before close. Provider label age, auto-close, unmark, reopen, an empty search page, or a green workflow proves none of those conditions.

The source's label wording, 14-day threshold, 30-result search bound, GitHub issue state, mutable action tag and async-settlement limitations are retained only as provenance / issue-maintenance semantics. `novel_capability=false`. No second defect/support ledger, waiting-for-user-action product state owner, release gate or shipping runtime is introduced.

## Fail-closed, concurrency and security conclusions
- Schedule and author-comment activity may race with human/provider label, close and reopen operations; stale provider state never advances canonical defect truth.
- The provider does not await every close/reopen mutation, so run success is not mutation-settlement evidence.
- A sweep only requests 30 matching issues; untouched later results are incomplete maintenance, not evidence of absence.
- The mutable `v0.5.0` tag is audited through its resolved immutable commit and bundle blobs; the provider tree has no license/package manifest/package lock and is not promoted as a Fabushi dependency authority.
- `github.token` is used for repository issue operations only; no product credential or production runtime owner is introduced.
- Provider close/reopen state cannot suppress reproduction, fix, regression, exact-head or release-acceptance requirements.

## Exact traceability IDs
- Requirement: `TDRP-R9-ISSUE-NEEDS-USER-ACTION-MAINTENANCE-001`
- Oracles: `ORA-TDRP-NEEDS-USER-ACTION-LABEL-AGE-001`, `ORA-TDRP-NEEDS-USER-ACTION-ORIGINAL-AUTHOR-RECOVERY-001`, `ORA-TDRP-NEEDS-USER-ACTION-NOT-DEFECT-CLOSURE-001`
- Invariants: `INV-TDRP-FQT-DEF-001-ONLY-CLOSURE-OWNER-002`, `INV-TDRP-NEEDS-USER-ACTION-NO-AUTO-ACCEPT-001`, `INV-TDRP-NEEDS-USER-ACTION-GREEN-NOT-MUTATION-SETTLEMENT-001`
- Contracts: `CONTRACT-TDRP-NEEDS-USER-ACTION-PROVIDER-ADAPTER-001`, `CONTRACT-TDRP-NEEDS-USER-ACTION-ORIGINAL-AUTHOR-RECOVERY-001`
- Temporal: `TEMP-TDRP-NEEDS-USER-ACTION-LABEL-AGE-001`, `TEMP-TDRP-NEEDS-USER-ACTION-COMMENT-RACE-001`
- Fault/recovery: `FAULT-TDRP-NEEDS-USER-ACTION-UNAWAITED-MUTATION-001`, `FAULT-TDRP-NEEDS-USER-ACTION-PROVIDER-CLOSE-WITHOUT-FIX-001`, `FAULT-TDRP-NEEDS-USER-ACTION-SEARCH-BOUND-001`
- Security: `SEC-TDRP-NEEDS-USER-ACTION-GITHUB-TOKEN-BOUNDARY-001`
- Regression: `REG-TDRP-FQT-DEF-001-REMAINS-SOLE-CLOSURE-OWNER-002`, `REG-TDRP-NEEDS-USER-ACTION-NO-PROVIDER-AUTHORITY-001`

Closing strict index 91 changes counters to unknown=16,283, unread=16,258, omitted=0, responsibilities=109. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 92 `.github/workflows/snap.yml@82f85ae9066a2773512afecb0bcf80875116a8c8`.
