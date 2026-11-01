# Complete read — fixed-U repository scheduled thread-lock workflow

- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/lock.yml@7904e79591f79a4ea61c0dc194c189f048bf5c67`
- Tree index: 125 zero-based
- Mode/size: 100644 / 274 ASCII bytes/code points
- Line endings: 15 LF, 0 CR, trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-LOCK-WORKFLOW-001`
- Stage A: authored only; `validation_status=not-run`; `accepted=false`

## Authority note

The active semantic queue/ledger/closure continues fixed U above. `upstream.lock.json` and `inventory/index.json` still carry older rebaseline identity `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`; that pre-existing whole-project mismatch remains explicit and is not silently repaired by this one source-entry closure.

## Trigger, runner and permissions

The source was read through EOF. Its only trigger is schedule cron `0 0 * * *`; the only job is `lock` on mutable `ubuntu-slim`. There is no explicit `permissions`, environment, concurrency, timeout, retry/backoff, checkout, job condition, matrix, push/pull-request/issue trigger, manual dispatch, repository dispatch or reusable-workflow declaration. The sole step passes `github.token`, issue inactivity 45 days and pull-request inactivity 45 days to `dessant/lock-threads@v6`.

Stage A does not execute the workflow or probe effective token scope. Effective permissions remain frozen-candidate evidence.

## Third-party Action lineage

Observed on 2026-11-01: `dessant/lock-threads@v6 -> 89ae32b08ed1a541efecbab17912962a5e38981c`. Exact identities at that commit:

- `action.yml@dd441d19ed1f3fe3d6e5d0c00c77d3b2b767a240` — Node 24, main `dist/index.js`
- `dist/index.js@78034b629a87f75af21c1cba9f361aa1b42a5d8f`
- `package.json@16ec483e2259859b74639bf1c289b647caa1fe7c` — version 6.0.2, MIT
- `package-lock.json@8eb0c7b72beeedceadf9d2bb9df5aaeed288bb88` — lockfile v3
- `src/index.js@2782ee7dd3137660021496bbef987f649a548d35`
- `src/schema.js@6a2850f10abff4dd2be93d156455ea1828741f8e`
- `src/data.js@740d1d9a67d344614dacedd6ce606908c4aed7c8`
- `src/utils.js@7daaca323090861a6315409c090178b6aff71155`
- `README.md@9faacb670e151b3db90b952638c968e76526c0fd`
- `LICENSE@a4ac43133b3847adb439a1d5103075a6e9078fa9`

The lockfile resolves @actions/core 3.0.1, @actions/github 9.1.1, @octokit/plugin-retry 8.1.0, @octokit/plugin-throttling 11.0.3 and joi 18.2.1 with npm registry integrity metadata. The source workflow still references mutable `v6`; observed commit/bundle identities are provenance, not a fixed source pin.

## Lock policy and state transition

The workflow sets only issue/pr inactivity and token. Action default `process-only` is empty, so processing expands to `issue`, then `pr`, then `discussion`. Thus explicit 45-day thresholds apply to issues and PRs, while discussions retain the Action default 365 days.

For each type the Action computes an ISO cutoff from current time minus inactivity days and searches `repo:<owner>/<repo> updated:<cutoff is:closed is:unlocked`. Issue and PR searches append `is:issue` / `is:pr`, use REST search sorted by `updated` descending with `per_page: 50`, do not paginate, and filter any locked items still returned. Discussion GraphQL search requests `first: 50`. One run therefore cannot prove exhaustion of a backlog larger than 50 eligible records per type.

No include/exclude labels, created/closed-date exclusions, comments, label add/remove, `process-only` override or `log-output` are configured. Issue/PR lock reason defaults to `resolved`; discussion is locked by GraphQL `lockLockable` without a reason. Per-result mutations are awaited, and thread types are awaited issue -> PR -> discussion. Non-empty processed sets are written to outputs as JSON thread selectors; no later workflow step consumes them.

## Retry, failure and recovery

The Action configures Octokit `request.retries=3` and throttling callbacks for primary/secondary rate limits; callbacks allow retry while `retryCount < 3`. Unrecovered search/lock errors propagate to the top-level catch and call `setFailed`, stopping remaining work.

There is no workflow-level retry, timeout, concurrency lock, transaction, rollback or unlock compensation. Partial state is therefore expected under failure: earlier issues may be locked before a later issue error; issue locks may settle before PR failure; issue+PR locks may settle before discussion failure. More than 50 eligible records require later scheduled runs. Schedule delay/skip, cutoff-boundary movement and concurrent update/lock races are non-atomic evidence boundaries. Authoritative repository state, not step completion alone, is settlement evidence.

## Security and privacy

Mutable third-party JavaScript receives `github.token` while the workflow has no explicit permissions block. The Action README demonstrates explicit `issues: write`, `pull-requests: write` and `discussions: write` in its examples; actual frozen-candidate scopes must be proven and token values must never be captured.

Under this exact configuration the Action does not request thread-body content, does not post comments, does not add/remove labels, and leaves `log-output` false. Repository/thread identities can still appear in outputs/debug metadata and remain repository data. Mutable runner/action identity, third-party execution with repository credentials and the implicit discussion path are the primary security/governance boundaries.

## Callers and consumers

Direct caller is GitHub scheduled-event admission. Direct dependencies/consumers are the third-party Action runtime, GitHub Search/Issues REST APIs, GitHub GraphQL discussion APIs, repository issue/PR/discussion state, maintainers/triagers and thread participants.

All 20 fixed-U workflow blobs were inspected for direct `lock.yml`, `dessant/lock-threads`, `workflow_call` or `workflow_run` coupling. None invokes this workflow. No Fabushi shipping runtime, product Task/Automation, Agent/Coordinator/Host/Runner runtime or platform-core service consumes it.

## Ownership and authored verification

Existing-owner-first maps applicable behavior to current Fabushi root repository issue/PR/reporting/development-process governance + repository security/privacy governance + shared quality/evidence governance. Rejected owners are a copied Telegram lock workflow, a second issue/PR tracker, product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, platform-core service and a second security/quality truth.

Authored only, not run: 18 contract, 1 integration, 12 fault/recovery and 10 security/privacy cases. No executable validation or repository thread mutation is performed in Stage A.
