# Waiting-for-answer issue-maintenance workflow complete read — strict index 96

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/waiting-for-answer.yml@0cccca6b2d434303da915b12a13e7620cf1b8bc4`, 17 line slots / 316 ASCII bytes; complete read and SHA revalidated.
- Dependency: `lee-dohm/no-response@v0.5.0` resolves to commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`; `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a`; `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`; tree `8600ce78cc9ed1dad710f332ac434cdf1d49fc0d` contains only those two blobs and no LICENSE/package manifest/package lock.
- Existing canonical defect-closure owner: `projects/fabushi-communication-platform/quality/defect-regression-policy.md@b31ecd424cac2abf8a2322e2996bb1b53c561378` / `FQT-DEF-001`; this blob is identical on live main and the PR parent.
- Parent exact-head 324c3de2868595e1e2af335aaec517725bdecd22/tree 0a7b937644e19a1805128f373b0f4af09207a544 completed TDRP run 39895682438 at 61/61 success (authority 12476960570 sha256:2b05e8ccc97402418dc9c659722e66798b35ad8f4296a1530c3ba15abd8370a5; closure 12477390119 sha256:6b66f44f462d41709ad1480e699b6e2454e6b656d677cc38ab531102839ac96a; reachability 12476906498 sha256:1d0a5ab03b4b91733f78a26990673c96b815d9a6663b3a0e72cd8e7040102f83) and Rust desktop runtime run 39895682382 at 6/6 success (artifact 12476134596 sha256:18c38e5052293d0113b69108397a3167eddfdeefd8c085104fdf3b9b20801563).
- Validator preflight: target `projects/fabushi-communication-platform/quality/defect-regression-policy.md` contains literal `Policy ID: FQT-DEF-001`; no existing ledger row claims that exact target-symbol tuple.

## Complete-read behavior
The workflow is repository issue maintenance, not defect adjudication. It subscribes to `issue_comment.created` and a daily 00:30 UTC schedule, then invokes `lee-dohm/no-response@v0.5.0` with `github.token` and `responseRequiredLabel: waiting for answer`.

The dependency defaults `daysUntilClose` to 14. On `schedule`, `sweep()` ensures the label exists, searches open issues carrying that label with `per_page: 30`, finds the latest matching labeled event, filters by the 14-day cutoff, and requests the default explanatory comment plus provider issue close. `ensureLabelExists()` does not await label creation from its catch path, and the sweep loop calls `close()` without awaiting the returned Promise, so green execution cannot prove every requested mutation settled.

On `issue_comment`, `unmark()` reads the event payload and current issue. It removes the label only when the issue still carries it and the commenter is the original issue author. If that issue is closed and `closed_by` differs from the original author, it requests reopen; that update is not awaited. Non-author comments do not satisfy the waiting state.

## Existing-owner-first disposition
`FQT-DEF-001` remains the sole canonical defect-closure owner. Its normative rule requires reproduction, fix, targeted regression, affected regression and current-head evidence before close. Provider label age, auto-close, unmark, reopen, an empty/partial search result, or a green workflow proves none of those conditions.

For non-defect issues, provider close/reopen state likewise cannot decide that an applicable source responsibility is N/A, reject product work, satisfy support resolution, or create release/migration acceptance. The source's label wording, 14-day threshold, 30-result search bound, mutable action tag and async-settlement limitations are retained only as repository-maintenance provenance. `novel_capability=false`; no second findings store, defect/support ledger, waiting-state product owner or release gate is introduced.

## Fail-closed, concurrency and security conclusions
- Schedule and author-comment activity may race with human/provider label, close and reopen operations; the workflow declares no concurrency group.
- Label creation, sweep close calls and conditional reopen have unawaited call sites, so run success is not per-item mutation-settlement evidence.
- A sweep requests only the first 30 matching open issues; untouched later results are incomplete maintenance, not evidence of absence.
- The mutable `v0.5.0` tag is audited through its resolved immutable commit and bundle blobs; the provider tree has no LICENSE/package manifest/package lock and uses the legacy Node 12 action runtime.
- `github.token` is repository-administration authority only; no product credential or production runtime owner is introduced.
- Provider close/reopen state cannot suppress reproduction, fix, regression, exact-head, source-completeness or release-acceptance requirements.

## Exact traceability IDs
- Requirement: `TDRP-R9-ISSUE-WAITING-FOR-ANSWER-MAINTENANCE-001`
- Oracles: `ORA-TDRP-WAITING-FOR-ANSWER-LABEL-AGE-001`, `ORA-TDRP-WAITING-FOR-ANSWER-ORIGINAL-AUTHOR-RECOVERY-001`, `ORA-TDRP-WAITING-FOR-ANSWER-NOT-CANONICAL-CLOSURE-001`
- Invariants: `INV-TDRP-FQT-DEF-001-ONLY-CLOSURE-OWNER-004`, `INV-TDRP-WAITING-FOR-ANSWER-NO-AUTO-ACCEPT-001`, `INV-TDRP-WAITING-FOR-ANSWER-GREEN-NOT-MUTATION-SETTLEMENT-001`
- Contracts: `CONTRACT-TDRP-WAITING-FOR-ANSWER-PROVIDER-ADAPTER-001`, `CONTRACT-TDRP-WAITING-FOR-ANSWER-ORIGINAL-AUTHOR-RECOVERY-001`
- Temporal: `TEMP-TDRP-WAITING-FOR-ANSWER-LABEL-AGE-001`, `TEMP-TDRP-WAITING-FOR-ANSWER-COMMENT-RACE-001`
- Fault/recovery: `FAULT-TDRP-WAITING-FOR-ANSWER-UNAWAITED-MUTATION-001`, `FAULT-TDRP-WAITING-FOR-ANSWER-PROVIDER-CLOSE-WITHOUT-FIX-001`, `FAULT-TDRP-WAITING-FOR-ANSWER-SEARCH-BOUND-001`
- Security: `SEC-TDRP-WAITING-FOR-ANSWER-GITHUB-TOKEN-BOUNDARY-001`, `SEC-TDRP-WAITING-FOR-ANSWER-MUTABLE-TAG-001`
- Regression: `REG-TDRP-FQT-DEF-001-REMAINS-SOLE-CLOSURE-OWNER-004`, `REG-TDRP-WAITING-FOR-ANSWER-NO-PROVIDER-AUTHORITY-001`

Closing strict index 96 changes counters to unknown=16,278, unread=16,253, omitted=0, responsibilities=114. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 97 `.github/workflows/win.yml@219c42ebf679ca5d93add5cdbd51b0bb6e516b57`.
