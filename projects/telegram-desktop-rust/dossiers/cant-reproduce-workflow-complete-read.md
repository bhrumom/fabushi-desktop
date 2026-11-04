# Fixed-U `.github/workflows/cant-reproduce.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source path/blob: `.github/workflows/cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186`
- Recursive-tree index: 119 zero-based
- Mode: `100644`
- Source identity: 902 ASCII bytes/code points / 25 LF / 0 CR / trailing newline / 26 split lines
- Responsibility: `TDRP-R9-REPOSITORY-CANT-REPRODUCE-WORKFLOW-001`
- Stage: A; executable validation deferred

## Exact workflow surface

The exact source was read through EOF. It is named `Can't reproduce.` and has exactly one trigger: schedule cron `0 3 * * *`, once daily at 03:00 UTC. It declares no `issues`, `issue_comment`, push, pull_request, workflow_dispatch, workflow_call or repository_dispatch trigger.

It contains one job, `cant-reproduce`, on `ubuntu-slim`. There is no explicit permissions block, environment, concurrency, timeout, retry/backoff, conditional, checkout or shell step. The only step calls `lee-dohm/no-response@v0.5.0` with:
- `token: ${{ github.token }}`
- `responseRequiredLabel: cant reproduce`
- a static multiline `closeComment` stating reproduction has not succeeded, information is insufficient, and ordinary users should seek upstream support via the application's Settings -> Ask question path.

That support copy is Telegram Desktop repository provenance and does not establish Fabushi support UX or routing.

## Action input and source-lineage contract

The referenced Action's v0.5.0 tag was observed resolving to commit `9bb0a4b5e6a45046f00353d5de7d90fb8bd773bb`. Its release snapshot exposes `action.yml@1eb04cb05934c6b5d299918356d4042115026b2a` and Node12 bundle `dist/index.js@7dada7580c55196f93eb911f8a4646152b7d039e`. The workflow reference is nevertheless a mutable tag, not a full commit SHA; observed resolution is therefore provenance only and must be rebound during frozen-candidate qualification.

The Action requires token input. The source workflow omits `daysUntilClose` and `responseRequiredColor`; v0.5.0 defaults them to 14 and `ffffff`. The configured label/comment are literal values. No workflow-side validation, regex, policy expression or parameter surface exists.

## Schedule routing and cutoff semantics

On `schedule`, the bundle calls `sweep()`. It searches `repo:<owner>/<repo> is:issue is:open label:"cant reproduce"`, sorts by updated descending, requests `per_page: 30`, and does not paginate this search. For each returned issue, it paginates issue events, reverses them and finds the newest matching `labeled` event. The issue is considered closeable only when that label event is older than `now - daysUntilClose`.

The cutoff therefore follows the latest label event, not the issue's `updated_at`, latest comment time, or a direct check that the issue author has remained silent. A comment or other activity after the label does not itself reset the timer. The 30-item non-paginated search is a source limitation: when more than 30 open matching issues exist, older matching issues can remain outside the sweep window while newer results occupy it.

## Label and state mutation

Before scanning, `ensureLabelExists()` attempts to read the configured label; any caught error starts label creation with the configured/default color. The create call is not awaited, and the catch is not restricted to a proven 404, so neither a read failure nor the fallback alone proves that the label is available.

For each selected issue, `close()` first awaits creation of the configured static comment and then awaits an issue update to `state: closed`. If the comment succeeds and the state mutation fails, repository state is partially mutated and requires reconciliation.

The bundle also contains `unmark()` for `issue_comment`: when the issue carries the response-required label and the original issue author comments, it removes the label and may reopen a closed issue if the closer was someone else. This workflow never subscribes to `issue_comment`, so that recovery behavior is unreachable here. Exact fixed-U siblings `needs-user-action.yml@55ad7891ce36682accae01d739203aa487876053` and `waiting-for-answer.yml@0cccca6b2d434303da915b12a13e7620cf1b8bc4` do subscribe to created comments while using the same Action, demonstrating the distinction.

## Async failure/recovery boundary

The Action's async entrypoint invokes `noResponse.sweep()` or `noResponse.unmark()` without awaiting either promise. `sweep()` invokes each `close()` without awaiting it, and the label-create fallback is also not awaited. Consequently the outer try/catch and sequential-looking control flow do not guarantee that asynchronous rejection/completion is observed before the action exits. This is a source reliability limitation, not an implementation to copy.

The workflow adds no concurrency control, timeout, retry/backoff, compensation or recovery job. Non-passing states include missing permission, mutable Action drift, label lookup/create ambiguity, search-window starvation, post-label activity being ignored by the cutoff, partial close mutation, asynchronous rejection, and reliance on the unwired comment-recovery path. Recovery belongs to the existing repository issue/process owner and requires fresh exact state/evidence.

## Permissions, security and privacy

The workflow declares no explicit `permissions` block. Effective `github.token` permissions therefore come from GitHub/repository/workflow host defaults and must be proved during Stage B; source semantics require sufficient rights for issue search/events plus label/comment/state mutation. No source statement proves least privilege.

The token is passed to third-party Action code. The bundle talks to GitHub APIs and debug-serializes issue/search/event objects; workflow-log access and private-repository issue metadata are therefore privacy/security boundaries. The source does not declare an extra telemetry endpoint, and the close comment is static rather than interpolating reporter-controlled issue text, but that does not eliminate token, Action-source or log-governance requirements.

The mutable v0.5.0 tag and Node12 action runtime are source provenance risks to qualify, not evidence of target trust. No Telegram support service, credential owner or public-help destination is imported into Fabushi.

## Direct callers and consumers

Direct caller: GitHub Actions scheduler for the repository at 03:00 UTC daily.

Direct software consumers: `lee-dohm/no-response@v0.5.0`, then GitHub Issues search/events/labels/comments/state APIs and repository issue state. No `workflow_call` caller exists.

Direct human participants/consumers: maintainers/triagers who apply/manage the `cant reproduce` label and issue authors/readers who observe comment/closure state. Repository source search found no independent exact workflow caller; sibling no-response workflows are separate consumers of the same Action rather than callers of this workflow.

## Existing-owner-first disposition

Portable responsibility maps to:
- root Fabushi repository issue/reporting/development-process authority;
- existing repository privacy/security governance;
- shared TEST_STRATEGY and Test Evidence Contract;
- FMEP-001 Stage-A deferred executable validation authority.

Rejected owners: Telegram-branded support runtime; copied no-response workflow as a product subsystem; second issue tracker/triage service; Product Tasks/Automations; Conversation/Transcript/Settings/Permissions product runtime; Agent/Coordinator/Host/Runner; platform-core authorization/service; second privacy/security/quality truth.

The current Fabushi checkpoint has no equivalent source-neutral issue-lifecycle workflow. This entry therefore qualifies repository/process provenance only and introduces no production shipping delta or target workflow copy.

## Authored Stage-B qualification

Authored only, not run:
- `CONTRACT-TDRP-CANT-REPRODUCE-TRIGGER-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-LABEL-ROUTING-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-AGE-CUTOFF-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-SEARCH-WINDOW-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-CLOSE-COMMENT-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-STATE-MUTATION-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-PERMISSIONS-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-RUNNER-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-ACTION-LINEAGE-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-INPUT-BOUNDARY-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-ISSUE-COMMENT-ABSENCE-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-CONSUMER-001`
- `CONTRACT-TDRP-CANT-REPRODUCE-NO-TARGET-WORKFLOW-COPY-001`
- `INT-TDRP-CANT-REPRODUCE-REPOSITORY-ISSUE-PROCESS-001`
- `FAULT-TDRP-CANT-REPRODUCE-MISSING-PERMISSION-001`
- `FAULT-TDRP-CANT-REPRODUCE-LABEL-LOOKUP-CREATE-001`
- `FAULT-TDRP-CANT-REPRODUCE-SEARCH-WINDOW-001`
- `FAULT-TDRP-CANT-REPRODUCE-POST-LABEL-ACTIVITY-001`
- `FAULT-TDRP-CANT-REPRODUCE-PARTIAL-CLOSE-001`
- `FAULT-TDRP-CANT-REPRODUCE-ASYNC-NOT-AWAITED-001`
- `FAULT-TDRP-CANT-REPRODUCE-NO-UNMARK-EVENT-001`
- `SEC-TDRP-CANT-REPRODUCE-TOKEN-SCOPE-001`
- `SEC-TDRP-CANT-REPRODUCE-MUTABLE-ACTION-TAG-001`
- `SEC-TDRP-CANT-REPRODUCE-THIRD-PARTY-ACTION-BOUNDARY-001`
- `SEC-TDRP-CANT-REPRODUCE-ISSUE-LOG-PRIVACY-001`
- `SEC-TDRP-CANT-REPRODUCE-NO-TELEGRAM-SUPPORT-AUTHORITY-001`

Oracles:
- `ORA-TDRP-CANT-REPRODUCE-EXACT-WORKFLOW-001`
- `ORA-TDRP-CANT-REPRODUCE-LABEL-AGE-CLOSE-001`
- `ORA-TDRP-CANT-REPRODUCE-ACTION-LINEAGE-001`
- `ORA-TDRP-CANT-REPRODUCE-FAILURE-BOUNDARY-001`

Invariants:
- `INV-TDRP-CANT-REPRODUCE-NO-ISSUE-COMMENT-RECOVERY-ASSUMPTION-001`
- `INV-TDRP-CANT-REPRODUCE-NO-MUTABLE-TAG-TRUST-001`
- `INV-TDRP-CANT-REPRODUCE-NO-TELEGRAM-SUPPORT-IMPORT-001`
- `INV-TDRP-CANT-REPRODUCE-NO-STAGE-A-EXECUTION-001`

Stage A executes no schedule/workflow, issue search, label/comment/state mutation, token probe, third-party Action runtime, build/typecheck/lint/validator/test/E2E, package, deployment, installation, runtime evidence or acceptance.
