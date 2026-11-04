# Fixed-U complete read — copyright year updater workflow

- Task round: 165
- Source evidence round: 196
- Desktop checkpoint parent: `f8a99e5ac3b98e8a7994cceeba7c716bfe03f6bd`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082`
- Tree index: 121 zero-based; mode 100644; 338 ASCII bytes/code points; 16 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-COPYRIGHT-YEAR-UPDATER-WORKFLOW-001`
- Stage-A validation: `not-run`; accepted: false

## Trigger and admission

The workflow has exactly two trigger classes: `repository_dispatch` with exact type `Restart copyright_year_updater workflow.`, and cron `0 3 1 1 *` (03:00 UTC on January 1). It has no push, pull_request, workflow_dispatch, workflow_call or path filter.

All other 19 fixed-U workflow blobs were directly checked for `Restart copyright_year_updater workflow.`, `copyright_year_updater` and `license-year`; none contains those caller/type strings. Sibling `master_updater.yml`, `unused_styles_updater.yml` and `user_agent_updater.yml` reuse `desktop-app/action_code_updater@master` with other types only. The direct workflow callers are therefore GitHub's scheduler and an external producer of the repository-dispatch event. The external producer's identity/authentication is not present in fixed-U and remains later host evidence.

## Permission and runner contract

There is one job, `Copyright-year`, on mutable `ubuntu-slim`. The YAML has no explicit `permissions`, environment, concurrency, timeout, retry, checkout or secret/input declaration other than the Action input. Effective `github.token` scope is determined by GitHub repository/organization host policy at runtime. Because the Action attempts clone/push and PR creation, source behavior requires write authority to succeed, but source review does not infer that such authority is actually granted.

## External Action source lineage

The workflow references mutable `desktop-app/action_code_updater@master`. On 2026-11-01 the branch resolves to `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`. The complete observed Action tree has three blobs; license-year consumes:
- `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`
- `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`

The composite Action runs in `github.action_path`, executes `npm init -y`, installs `simple-git isbinaryfile` and `@actions/core @actions/github` without versions or a repository lockfile, then executes `node index.mjs ${{ github.token }} ${{ inputs.type }}`. Exact runtime npm/transitive identities are therefore mutable acquisition evidence. The token is conveyed in argv and is also embedded by the script into an HTTPS Git remote URL. No dependency installation or token-scope probe was executed in this round.

## Year detection and file mutation

For `type=license-year`, `LicenseYear` sets `year = new Date().getFullYear()` and `previousYear = year - 1`. `getFullYear()` is process-local time, not `getUTCFullYear()`; the workflow cron is UTC but runner timezone is not declared, and repository_dispatch can run at any date/time.

The Action clones the target repository through the token URL with no ref argument and no special clone options for license-year. It therefore scans the then-current default branch, not the event SHA or fixed U. Current repository metadata reports default branch `dev`; actual execution branch/HEAD remains host evidence.

`processFiles` recursively enumerates the cloned working tree, excludes paths whose string contains `.git`, reads each remaining path, skips content classified binary by `isbinaryfile`, and applies `LicenseYear.replace` to text. The eligibility predicate lowercases the complete file and searches exact `copyright (c) 2014-<previousYear>`. If present, mutation invokes ordinary string `replace("2014-<previousYear>", "2014-<year>")`, changing only the first exact numeric range anywhere in the file. It is not anchored to the matched copyright phrase and is not global; an unrelated earlier range can therefore be modified, and multiple notices in one file are not all updated by one pass.

Fixed-U has zero tracked symlinks, so this fixed snapshot has no tracked symlink-following mutation surface. It has 36 gitlinks. The license-year clone does not use `--recurse-submodules`, so gitlink contents are not checked out/scanned by this workflow mode.

When zero files change, `processFiles` rejects with `No modified files.`; the top-level catch converts that exact string into exit 0. This is an intentional source no-op outcome, not evidence that a different repository or frozen Fabushi candidate has complete/current notices.

## Commit, branch and PR publication

For changed files, `processGit` records the cloned current branch as `baseBranch`, derives `copyright_to_<year>`, and refuses to continue if `branches.all` already exposes a branch containing that name. Otherwise it:
1. resets origin to the token-bearing Git URL;
2. configures `user.name=GitHub Action`, `user.email=action@github.com`;
3. creates local `copyright_to_<year>`;
4. stages `.`;
5. commits `Update copyright year to <year>.`;
6. verifies the latest log message matches;
7. pushes `-u origin copyright_to_<year>`.

After branch push, `processPullRequest` calls `pulls.list` with repository owner/name and no pagination arguments, searches the returned page for matching head/base, and creates a PR if none is found. For LicenseYear there is no custom PR body; title defaults to the commit message.

The entire PR list/create block is inside `try/catch` that logs errors and does not rethrow or set a nonzero exit. Therefore branch push may succeed while PR list/create fails, yet the Action can still finish without a failing exit. A preexisting matching PR outside the unpaginated list page can also lead to a duplicate-create attempt whose error is swallowed. Publication qualification must distinguish pushed branch, created PR and overall process exit.

## Direct consumers

Direct process chain:
1. GitHub scheduler or external repository-dispatch producer admits the workflow.
2. GitHub runner resolves mutable `ubuntu-slim` and mutable Action `master`.
3. Composite Action resolves mutable npm packages and receives `github.token`.
4. Action clones the repository's runtime default branch.
5. Text-file traversal consumes the cloned working tree; binary files and non-recursed gitlink contents are excluded.
6. Git operations consume modified files to create commit/branch/push.
7. GitHub Pull Requests API consumes branch/base metadata.
8. Maintainers/reviewers consume the resulting branch/PR.

No Fabushi application runtime, Agent/Coordinator/Host/Runner, updater service, UI, Task/Automation product feature or platform-core service consumes this source workflow.

## Security, failure and recovery

Fail-closed boundaries:
- exact repository-dispatch producer/authentication is external to fixed-U;
- effective `github.token` permissions are implicit host policy;
- `ubuntu-slim`, Action `master`, runtime Node/npm and unpinned npm packages are mutable;
- the token is passed through Node argv and embedded in Git remote configuration during the Action;
- the clone is runtime default-branch state, not event/fixed-U SHA;
- local timezone affects `getFullYear()`;
- whole-repository non-binary text scanning is broader than license files;
- first numeric-range replacement can target an earlier unrelated occurrence and updates only one occurrence per eligible file;
- no-change exits 0;
- existing copyright branch is a hard failure;
- clone/install/read/write/commit/push errors are non-success;
- PR list/create errors are swallowed after a successful push and therefore create possible partial publication;
- mutable source success or no-op does not grant Fabushi legal/release acceptance.

Recovery requires fresh exact frozen-candidate binding of source/action/dependencies/runner/timezone/default branch/token policy, deterministic before/after mutation manifest, branch commit identity and PR identity. A branch-without-PR or ambiguous replacement is not promoted to success.

## Existing-owner-first target disposition

Applicable guarantees map to existing Fabushi root repository source-maintenance/process governance, repository security/provenance governance, shared `TEST_STRATEGY.md` and `evidence-contract.md`, plus final release/legal review where applicable. Rejected owners: copied Telegram copyright workflow, second legal/copyright truth, shipping updater, Product Tasks/Automations, Agent/Coordinator/Host/Runner, Telegram-branded maintenance subsystem and platform-core service.

No target workflow, product runtime, release workflow, package owner or platform-core code is changed for this repository-only source responsibility.

## Authored Stage-A cases

Contracts (18):
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-TRIGGER-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-DISPATCH-CALLER-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PERMISSIONS-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-RUNNER-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-ACTION-LINEAGE-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-RUNTIME-DEPS-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-TOKEN-DELIVERY-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-YEAR-DETECTION-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-MATCH-REPLACE-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-FILE-SCAN-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-GITLINK-SCOPE-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NOOP-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-COMMIT-PUSH-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-PUBLICATION-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-ERROR-BOUNDARY-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-CONSUMERS-001`
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-TARGET-COPY-001`

Integration (1):
- `INT-TDRP-COPYRIGHT-YEAR-WORKFLOW-REPOSITORY-SOURCE-MAINTENANCE-001`

Fault/recovery (10):
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-DISPATCH-TYPE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PERMISSION-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTABLE-MASTER-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NPM-ACQUISITION-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-YEAR-TIMEZONE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-WRONG-FIRST-RANGE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-MODIFIED-FILES-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-EXISTS-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-COMMIT-PUSH-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-SWALLOWED-001`

Security/provenance (8):
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-IMPLICIT-TOKEN-SCOPE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTABLE-ACTION-MASTER-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-UNPINNED-NPM-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-TOKEN-ARGV-REMOTE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-BROAD-TEXT-SCAN-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-PR-WRITE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-SOURCE-MAINTENANCE-AUTHORITY-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-STAGE-A-EXECUTION-001`

Oracles:
- `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-ACTION-LINEAGE-001`
- `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTATION-PUBLICATION-001`
- `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-FAILURE-BOUNDARY-001`

Invariants:
- `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-MUTABLE-MASTER-TRUST-001`
- `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-HOST-PERMISSION-ASSUMPTION-001`
- `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-PARTIAL-PR-SUCCESS-001`
- `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-BROAD-SCAN-AUTHORITY-001`
- `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-STAGE-A-EXECUTION-001`

All are authored only. No executable qualification was run.
