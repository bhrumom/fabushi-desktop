# Complete read — .github/workflows/master_updater.yml

## Authority and identity

- Upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U commit date: 2026-10-26T18:17:45Z
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
- Entry: tree index 128 zero-based
- Blob: `.github/workflows/master_updater.yml@fedc24f8e5d781a0bbdcdf80e24baf3eb2fe199f`
- Mode/size: 100644; 221 ASCII bytes/code points; 13 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-MASTER-UPDATER-WORKFLOW-001`
- Stage-A status: implemented / validation not-run / accepted false

## Trigger, permissions and runner contract

The only trigger is GitHub `release` with `types: released`. The sole job is named `User-agent`, runs on mutable `ubuntu-slim`, and contains one step: `desktop-app/action_code_updater@master` with `type: "dev-to-master"`.

There is no checkout, explicit `permissions`, environment, concurrency, timeout, retry, schedule, workflow_dispatch, repository_dispatch, workflow_call or workflow_run. The source therefore delegates both code and mutation semantics to a mutable third-party Action and the repository's effective default token policy.

## Action source lineage

Observed on 2026-11-02:
- `desktop-app/action_code_updater@master = 0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`
- master is not protected
- `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`, 544 bytes
- `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`, 11,890 bytes
- `check_unused_styles.py@6a061912121f49ffa386ff9692b989378b6bdd2b`, 14,075 bytes
- repository metadata reports no license
- latest observed master commit was created 2026-04-22

The composite Action runs `npm init -y`, dynamically installs unpinned `simple-git`, `isbinaryfile`, `@actions/core` and `@actions/github`, then invokes `node index.mjs ${{ github.token }} ${{ inputs.type }}`. No committed package manifest/lock fixes those runtime package versions or registry bytes.

The `dev-to-master` disable guard was introduced by `5f85512cf8e8f15cfbde4a10cb29208bcc3d3e52` on 2025-10-02. It still exists at observed master and exits 0 before the script assigns the token to a variable, clones the repository or reaches mutation logic. The composite shell nevertheless supplies the token as process argv. Because the workflow selects mutable `@master`, neither current no-op behavior nor historical chronology is immutable source identity.

## Dormant source-update and mutation contract

The dormant `DevToMaster` implementation sets source branch `dev` and target branch `master`. Its replacement function is a no-op because this mode is branch promotion rather than file rewriting.

If the guard is removed, the generic pipeline clones the same repository through `https://x-access-token:<github.token>@github.com/<owner>/<repo>.git`. `processFiles` skips content traversal for `dev-to-master`. `processGit` sets the base branch to `master` and deliberately skips local branch creation, add, commit and push for this mode.

`processPullRequest` constructs an Octokit client from the token, lists pull requests, checks for an existing head `dev` / base `master`, and creates one only when absent. The generated body resolves `origin/dev` and `origin/master`, includes their seven-character prefixes and the generation date. Thus even dormant active behavior proposes a PR from existing `dev` to existing `master`; it does not itself merge or directly update master.

The PR list/create block catches API exceptions, prints them, and does not rethrow. Consequently a failed PR mutation can be masked as a successful Action step after earlier pipeline stages succeeded. No rollback, workflow-level retry or concurrency group exists. Overlapping release events can race between duplicate detection and creation, and branch heads can advance after the body hashes are read.

## Direct callers and consumers

The direct caller is GitHub's release event of type `released`; the workflow is not reusable. No fixed-U workflow invokes `master_updater.yml` through workflow_call/workflow_run/local workflow composition.

Three sibling fixed-U workflows independently consume the same mutable Action with different inputs:
- `copyright_year_updater.yml` -> `license-year`
- `unused_styles_updater.yml` -> `unused-styles`
- `user_agent_updater.yml` -> `user-agent`

When `dev-to-master` is enabled, the only direct repository output is a same-repository PR proposal. Review/merge and any real master-branch mutation are external repository-policy responsibilities.

## Existing-owner-first mapping

Applicable source-neutral responsibility is owned by:
- Fabushi root repository source-maintenance and branch/release process governance;
- repository security/provenance governance;
- shared Test Strategy and Test Evidence Contract;
- FMEP-001 Stage-A deferred executable validation.

There is no reason to copy Telegram's release-triggered `dev -> master` policy into Fabushi. Any Fabushi source promotion remains under its existing repository/branch/release governance and protection rules. Rejected owners are a copied Telegram updater workflow, product Tasks/Automations, Telegram runtime/provider/sidecar, Agent/Coordinator/Host/Runner runtime, second source/release truth and platform-core.

## Failure, recovery and security findings

Fail-closed conditions include release-event admission drift, mutable runner or Action master drift, dynamic npm registry/package drift, token permission denial, removal/change of the disable guard, clone/auth failure, missing `dev` or `master` refs, PR list/create failure, masked PR API errors, duplicate-PR races, branch movement after captured hashes, and overlapping releases without serialization.

Security/provenance must preserve effective token scope without secret capture, exact resolved Action tree, runtime npm versions/integrities, runner identity, release actor/event revision, same-repository PR mutation boundary and repository branch/ruleset policy. Token argv and tokenized clone URLs are sensitive process surfaces. The Action repository currently has no declared license metadata, so reuse/copying is additionally inappropriate absent separate licensing resolution.

## Stage-A authored verification

Authored-only coverage is 24 contract cases, 1 integration case, 15 fault/recovery cases and 14 security/provenance cases in the shared Test Strategy and Evidence Contract. None is executed in Stage A.

No release/workflow/Action/npm/token/clone/branch/PR operation, Actions/check/status/workflow-run query, build/test/package/release/deployment/installation/runtime acceptance or platform-core mutation occurred while producing this dossier.

The whole-project authority gap remains pre-existing and separate: active ledger/source-closure follows fixed U `f985723...`, while `upstream.lock.json` / inventory metadata still carries older `351204f...` baseline identity. This single-entry closure does not claim baseline reconciliation.
