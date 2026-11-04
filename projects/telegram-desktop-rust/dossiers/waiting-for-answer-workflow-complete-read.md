# Complete read — .github/workflows/waiting-for-answer.yml

Date: 2026-11-02
Task round: 178
Desktop base: `4f95f3adda1aa2e3462c2e90ee2fe41279d93f44`
Desktop main observed: `e0f99f278a6494a35bda378e6d3203149486e170`
Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
Source: `.github/workflows/waiting-for-answer.yml@0cccca6b2d434303da915b12a13e7620cf1b8bc4`
Tree index: 134 zero-based
Mode/size: 100644 / 316 ASCII bytes/code points / 16 LF / 0 CR / trailing newline
Responsibility: `TDRP-R9-REPOSITORY-WAITING-FOR-ANSWER-WORKFLOW-001`
Disposition: existing-owner-first repository issue/process/security/quality authority; executable validation deferred.

## Trigger, runner and token surface

The workflow has exactly two triggers: `issue_comment.created` and daily schedule `30 0 * * *`. Its one `waiting-for-answer` job runs on mutable `ubuntu-slim`; there is no explicit permissions block, environment, concurrency, timeout, retry, checkout, manual/repository/reusable/workflow-run trigger or job condition. The sole step invokes `lee-dohm/no-response@v0.5.0` with `github.token` and exact `responseRequiredLabel: waiting for answer`.

Omitted Action inputs inherit v0.5.0 defaults: `daysUntilClose=14`, `responseRequiredColor=ffffff`, and the static public no-response close comment. Effective token permissions are not stated by source and remain frozen-candidate host evidence.

## Third-party Action lineage

Observed on 2026-11-02, `lee-dohm/no-response@v0.5.0` resolves to commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`, tree `8600ce78cc9ed1dad710f332ac434cdf1d49fc0d`. The tag snapshot contains `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a` and declared Node12 bundle `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`; repository metadata reports MIT. The workflow pins only the mutable tag, so candidate qualification must re-resolve the tag and bind exact code/runtime/legal provenance.

## Scheduled waiting-for-answer lifecycle

On `schedule`, the bundle starts `sweep()`. It attempts to ensure the configured label, searches `repo:<owner>/<repo> is:issue is:open label:"waiting for answer"`, sorts updated descending and requests only `per_page=30`; the issue search is not paginated. For each returned issue it paginates issue events, reverses them and selects the newest exact matching `labeled` event. The cutoff compares that event timestamp against `now - 14 days`.

The timer therefore follows the newest label event, not issue `updated_at`, latest comment time, or a direct silence calculation. Activity after the label does not reset the source cutoff. For each selected issue `close()` first awaits creation of the static public close comment and then awaits the state update to `closed`.

## Comment, unmark and reopen lifecycle

On `issue_comment.created`, `unmark()` reads the event payload, fetches the issue-like object and current labels, and removes the exact label only when the commenter login equals the original author's login. Comment contents are not validated. If the object is closed and `closed_by` differs from the original author, the bundle starts `issues.update(state=open)`.

GitHub `issue_comment` applies to both issues and pull-request conversations, and the Action source does not gate on `payload.pull_request`. PR-shaped comment events can therefore enter the unmark/reopen branch even though the scheduled sweep query is explicitly `is:issue`. Source alone does not prove GitHub will accept every PR-shaped state mutation, so that settlement is deferred evidence rather than an assumed guarantee.

## Failure and recovery boundaries

`ensureLabelExists()` treats any get-label exception as a reason to call create-label; the create call is not awaited. The entrypoint calls `sweep()` / `unmark()` without awaiting, `sweep()` starts each per-issue `close()` without awaiting, and the conditional reopen update is also not awaited. The outer try/catch therefore cannot be treated as authoritative settlement of asynchronous failures.

The first-30 search can starve older labeled issues. Comment creation can settle before close fails; label removal can settle before reopen fails. There is no workflow concurrency fence, timeout, retry/backoff, transaction, rollback or compensation job. Any partial/ambiguous state requires fresh issue/label/comment/state reconciliation in Stage B.

## Security and privacy

The workflow has no explicit permissions while third-party code receives `github.token`. The Action can debug-serialize issue search and event objects; private repository metadata and reporter content can therefore reach workflow logs. The default close comment is public. Durable evidence must exclude token values and unnecessary private issue/comment/debug payloads.

## Callers, consumers and existing owner

Direct callers are the repository scheduler and `issue_comment.created`. Direct software consumers are `lee-dohm/no-response@v0.5.0` and GitHub Issues search/events/labels/comments/state APIs; human consumers include maintainers/triagers and issue/PR authors/readers. Fixed-U workflow review found no `workflow_call` caller. `needs-user-action.yml@55ad7891ce36682accae01d739203aa487876053` and `cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186` are separate lanes sharing the Action.

Applicable target responsibility stays with existing Fabushi repository issue/reporting/development-process + privacy/security + shared quality/evidence governance. Rejected owners: Telegram support subsystem, copied no-response workflow, second issue tracker/triage service, Product Tasks/Automations, Conversation/Transcript product runtime, Agent/Coordinator/Host/Runner, platform-core service, and any second privacy/security/quality truth. No shipping product code or platform-core delta is required for this repository-host governance entry.

## Authored Stage-B qualification

Authored only, not run: 18 contract, 1 integration, 12 fault/recovery, 8 security/privacy/provenance cases. Stage A remains `implemented / validation_status=not-run / accepted=false`. No workflow, issue/PR mutation, Action execution, token probe, build, typecheck, lint, validator, test/E2E, package, deployment, installation, Actions/check/status/workflow-run query or platform-core mutation was performed.
