# Complete read — .github/workflows/needs-user-action.yml

Date: 2026-11-02
Task round: 173
Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
Source: `.github/workflows/needs-user-action.yml@55ad7891ce36682accae01d739203aa487876053`
Fixed-U tree: non-truncated, 7,194 entries
Tree index: 129 zero-based
Mode/size: 100644 / 312 ASCII bytes/code points / 16 LF / 0 CR / trailing newline
Responsibility: `TDRP-R9-REPOSITORY-NEEDS-USER-ACTION-WORKFLOW-001`
Disposition: existing-owner-first repository process/security/quality authority; executable validation deferred.

The workflow has exactly two triggers: `issue_comment.created` and daily schedule `0 2 * * *`. Its one `needs-user-action` job runs on mutable `ubuntu-slim`; no explicit permissions, environment, concurrency, timeout, retry, checkout, manual/repository/reusable/workflow-run trigger exists. It invokes `lee-dohm/no-response@v0.5.0` with `github.token` and exact label `needs user action`, while days/color/close copy use Action defaults.

Observed Action tag resolution is commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`, tree `8600ce78cc9ed1dad710f332ac434cdf1d49fc0d`, `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a`, Node12 `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`; repository metadata reports MIT. The tag remains mutable provenance.

Scheduled sweep ensures the label, searches only open issues with that exact label, requests only the first 30 updated-descending results, then paginates issue events and uses the latest matching label-event timestamp. Default eligibility is older than 14 days. It posts the Action's public default close comment before closing state. Post-label activity alone does not reset the cutoff.

On `issue_comment.created`, the Action reads the issue-like payload, checks the exact label and original-author identity, removes the label only for an original-author comment, and conditionally starts reopen if the item is closed by someone else. It does not validate comment contents. Since GitHub issue_comment also covers PR conversations and the Action does not gate `payload.pull_request`, PR-shaped payloads can enter this lane even though scheduled search is explicitly `is:issue`.

Failure/recovery boundaries: any label-get exception enters create-label; create-label is not awaited. Top-level sweep/unmark, per-issue close and reopen update also contain unawaited calls. Search is capped at 30; comment-before-close and unmark-before-reopen can partially settle; no workflow concurrency/timeout/retry/rollback exists. Workflow completion therefore cannot prove all intended mutations.

Security/privacy boundaries: implicit token permissions; mutable third-party tag; declared legacy Node12 runtime; possible debug serialization of issue search/event objects; public close comments. Token values and unnecessary private issue content must not enter evidence.

Direct callers are the scheduler and issue_comment.created. Direct consumers are no-response and GitHub Issues APIs. Siblings `waiting-for-answer.yml@0cccca6b2d434303da915b12a13e7620cf1b8bc4` and `cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186` independently use the same Action. No fixed-U workflow calls this file as a reusable workflow.

Target owner is existing Fabushi repository issue/reporting/development-process + privacy/security + shared quality/evidence governance. Rejected: Telegram support subsystem, copied no-response workflow, second issue tracker, product Tasks/Automations, Conversation runtime, Agent/Coordinator/Host/Runner, platform-core service, second quality/security truth. No product shipping code or platform-core delta is required for this repository-host governance responsibility.

Authored only, not run: 18 contract, 1 integration, 12 fault/recovery, 8 security/privacy/provenance cases. Stage A remains `implemented / validation_status=not-run / accepted=false`. No workflow, issue mutation, Action execution, token probe, build, validator, test/E2E, package, deployment, installation, Actions/check/status query or platform-core mutation was performed.
