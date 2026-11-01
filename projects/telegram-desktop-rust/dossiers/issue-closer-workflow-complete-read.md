# Complete read — fixed-U repository issue closer workflow

- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/issue_closer.yml@355405165590712b1826b2f92a8f867c8d568dd9`
- Tree index: 123 zero-based
- Mode/size: 100644 / 251 ASCII bytes/code points
- Line endings: 14 LF, 0 CR, trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-ISSUE-CLOSER-WORKFLOW-001`
- Stage A: authored only; `validation_status=not-run`; `accepted=false`

## Workflow and external Action

Trigger is only `issues.opened`; one mutable `ubuntu-slim` job invokes `desktop-app/action_issue_closer@master` with `secrets.GITHUB_TOKEN`. No explicit permissions/environment/concurrency/timeout/retry/checkout/manual/schedule/reusable trigger exists.

Observed 2026-11-01 Action master: `b2fc72db6cc262c8e84e10740c44296ec4be2e02`, tree `a949a72cc7db2446287cc7c88e651966c1243f77`, `action.yml@91083e19bf270c6b8fe109fd264522032a5f10a1`, `index.js@ea3b9b265e070522b56e75da86debaa80140278f`. This is mutable-master observation only. Composite setup downloads `https://osx.telegram.org/updates/versions.xml` with suppressed wget failure, installs four npm packages without versions/lock, and passes the token in Node argv.

## Classification

The script reads current issue context/body, lists repository tags, searches exact headings `Version of Telegram Desktop` and `Installation source`, parses the first match with `/[0-9]{1,2}\.[0-9][0-9.]{0,}/g`, parses `tags.data[0].name`, and reads the first updater XML item's `@_sparkle:shortVersionString`.

Ordered decisions: (1) issue <= first tag and issue < macOS => no mutation; (2) issue > macOS and not prefixed by macOS => indeterminate/no mutation; (3) same first major character and issue > first tag => stale-tag/no mutation; (4) otherwise mutate. Missing expected markers or parseable version return without mutation where handled. Null/non-string body and malformed XML/tag structures are not fully guarded.

## Mutation

Mutation intent is static comment -> exact label `TG macOS Swift` -> issue state `closed`. None of the three API promises is awaited or returned; `processIssue` is likewise not awaited by the XML callback. Thus completion/order/atomicity are not guaranteed by source syntax. Partial or unsettled mutation must be reconciled against authoritative issue state.

## Security/privacy and ownership

The mutable third-party Action receives a secret token while the workflow has no explicit permissions block. Npm dependencies and updater XML are unpinned. Token is in Node argv. The script logs the full issue body and parsed indices/versions, creating workflow-log privacy exposure for reporter-controlled text.

Direct caller is GitHub Issues opened-event admission. Consumers/dependencies are Action runtime, issue body/context, repository tags, updater XML, GitHub comment/label/state APIs, maintainers/triagers and issue authors/readers. No Fabushi shipping runtime or platform-core service consumes this workflow.

Existing-owner-first maps applicable behavior to root issue/reporting/development-process + repository privacy/security + shared quality/evidence governance. No Telegram issue closer, second issue tracker, support subsystem, product automation, runtime provider or platform-core owner is created.

Authored only: 16 contract, 1 integration, 9 fault/recovery and 7 security/privacy cases; none executed.
