# Can't reproduce issue-maintenance workflow complete read — strict index 81

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/workflows/cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186`, 26 line slots / 902 bytes; complete read and SHA revalidated before publication.
- Dependency: `lee-dohm/no-response@v0.5.0` resolves to commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`; `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a`; `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`.
- Existing canonical defect-closure owner: `FQT-DEF-001@b31ecd424cac2abf8a2322e2996bb1b53c561378`.
- Parent exact head/tree: `38e568c9a98ad428a23aef5024d11b50bc989062` / `6012cc85a9a84c5481a9a2287ed0b6910c8e63f9`.
- Parent TDRP run `39761337683`: 61/61 success; authority `12418356033` / `sha256:dd2fdbe8f8bf9351409290904ac9b2f2e65e181168973122abed1052690611dc`; closure `12418014415` / `sha256:5e4c1daddf0d9de1f83ac24c828eccf3cd809b62a3dd33afedffa16b8e5bdcf6`; reachability `12419260305` / `sha256:1dcbf3ec7b0251169ef359cacb27748434d8b6c2a767c3ae243f6d4a3c274302`.
- Parent Rust run `39761337691`: 6/6 success; artifact `12418720447` / `sha256:e5d6c81bfdd66cdb82027f2339a5539876224d5352ce917e02dab9453a2cc470`.

## Complete-read behavior
The workflow is repository issue maintenance, not defect adjudication. It runs daily at 03:00 UTC and invokes `lee-dohm/no-response@v0.5.0` with `github.token`, `responseRequiredLabel: cant reproduce`, and an explanatory close comment.

The dependency defaults `daysUntilClose` to 14. Its bundled implementation searches open issues carrying the configured label, finds the latest matching labeled event, compares that event to the cutoff, then comments and closes qualifying issues. The dependency also implements `issue_comment -> unmark()` and can reopen/unlabel after an original-author response, but the TDesktop workflow subscribes only to `schedule`; therefore that recovery path is dependency provenance, not an active source-workflow consumer path.

## Existing-owner-first disposition
`FQT-DEF-001` remains the sole canonical defect-closure owner. Its normative rule is `reproduce -> fix -> targeted regression -> affected regression -> current-head evidence` before close. A provider issue that auto-closes after 14 days proves none of those conditions and cannot advance canonical Fabushi defect, regression, release, or acceptance state.

The source's label name, close comment, 14-day timeout, mutable action tag and GitHub issue state are retained only as provenance / issue-maintenance semantics. `novel_capability=false`. No second defect ledger, support truth, closure authority, release gate, or shipping runtime is introduced.

## Fail-closed and security conclusions
- GitHub issue close is administrative provider state only.
- Action/tag/API/token/scheduler failure or drift leaves maintenance incomplete and cannot imply a clean defect state.
- The mutable `v0.5.0` tag is audited through its resolved immutable commit and bundle blobs; it is not promoted as a stronger Fabushi dependency rule.
- Provider labels/events/comments may persist across runs, but canonical closure is reconstructed only from FQT defect evidence.
- `github.token` is used for repository issue operations only; no product credential or production runtime owner is introduced.

## Exact traceability IDs
- Requirement: `TDRP-R9-ISSUE-NO-RESPONSE-MAINTENANCE-001`
- Oracles: `ORA-TDRP-NO-RESPONSE-LABEL-AGE-001`, `ORA-TDRP-NO-RESPONSE-MAINTENANCE-NOT-DEFECT-CLOSURE-001`
- Invariants: `INV-TDRP-FQT-DEF-001-ONLY-CLOSURE-OWNER-001`, `INV-TDRP-NO-RESPONSE-NO-AUTO-ACCEPT-001`
- Contract: `CONTRACT-TDRP-NO-RESPONSE-PROVIDER-ADAPTER-001`
- Fault/recovery: `FAULT-TDRP-NO-RESPONSE-PROVIDER-CLOSE-WITHOUT-FIX-001`
- Security: `SEC-TDRP-NO-RESPONSE-GITHUB-TOKEN-BOUNDARY-001`
- Regression: `REG-TDRP-FQT-DEF-001-REMAINS-SOLE-CLOSURE-OWNER-001`

Closing strict index 81 changes counters to unknown=16,222, unread=16,197, omitted=0, responsibilities=99. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 82 `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c`.
