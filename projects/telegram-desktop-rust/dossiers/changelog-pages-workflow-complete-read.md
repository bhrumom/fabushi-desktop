# Fixed-U `.github/workflows/changelog.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source path/blob: `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c`
- Tree index: 120 zero-based; mode `100644`; Git size 847 bytes
- Decoded content: 847 ASCII code points, 45 LF, 0 CR, trailing newline, 46 split lines
- Triage: `full-read` with exact generator dependency reused from prior complete-read
- Responsibility: `TDRP-R9-REPOSITORY-CHANGELOG-PAGES-WORKFLOW-001`
- Task round: 164
- Stage: A; executable validation deferred

## Trigger, permission and concurrency contracts

The exact workflow was read through EOF. It is named `Changelog`. It admits only:
1. `push` to branch `dev` when exact root `changelog.txt` is in the changed path set; and
2. `workflow_dispatch`.

There is no pull_request, schedule, workflow_call, repository_dispatch, issue, release or tag trigger. The workflow-level permissions are `contents: read`, `pages: write`, `id-token: write`. No job-level override narrows them, therefore both build and deploy inherit the full set. The source does not prove least privilege.

Concurrency is a single group `pages` with `cancel-in-progress: true`. A later admitted run can cancel an earlier in-progress publication attempt regardless of whether the older run came from push or manual dispatch. There is no YAML timeout or retry/backoff policy.

## Build and changelog-generation chain

`build` runs on mutable `ubuntu-latest` and executes:
- `actions/checkout@v7`;
- `actions/setup-python@v7` with `python-version: "3.12"`;
- `python .github/scripts/generate_changelog.py` with no argv overrides;
- `actions/upload-pages-artifact@v5` with `path: docs`.

The exact generator dependency `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0` was already completely read under `TDRP-R9-REPOSITORY-CHANGELOG-PAGES-GENERATOR-001`. Reuse remains valid because the source blob and direct invocation are unchanged. Its default fixed-U input is `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`; it generates `docs/changelog/index.html` and optionally copies exact icon inputs. No application runtime consumes the generated page.

## External Action lineage

The workflow references version tags, not immutable commits. Observed tag resolution on 2026-11-01:
- `actions/checkout@v7` -> commit `3d3c42e5aac5ba805825da76410c181273ba90b1`, `action.yml@5b0524f730db83f9513c18ab31a6c086c7239076`, Node24.
- `actions/setup-python@v7` -> commit `6d4fd22b20465ccc0f7354476588e8df7c9ca0e0`, `action.yml@645de4f5108b28bcc7ed043ee171770b0754cc35`, Node24.
- `actions/upload-pages-artifact@v5` -> commit `fc324d3547104276b827a68afc52ff2a11cc49c9`, `action.yml@82713b4cbb35ccf98dfa8f2e0deedc5e83b6d845`, composite action.
- `actions/deploy-pages@v5` -> commit `368f82528645a54fb793d4d04e342629a3f51346`, `action.yml@9640517bb026e0790a53bd6d35fd13a94617c48e`, Node24.

These observed commits are lineage evidence only; the fixed-U source identity is the literal tag reference. Stage-B frozen-candidate evidence must resolve the actual bytes again.

Checkout defaults `persist-credentials: true`; the workflow does not override it. Setup-Python is asked for `3.12`, not an exact patch version, and may source distributions from the action's mutable tool-cache/version/mirror chain. Both facts are runtime provenance/security boundaries.

## Pages artifact contract

The observed upload-pages-artifact v5 action defaults:
- artifact name: `github-pages`;
- retention: 1 day;
- include hidden files: false.

It archives `docs` using tar with `--dereference --hard-dereference`, excluding `.git` and `.github`, then invokes full-SHA `actions/upload-artifact@bbbca2ddaa5d8feaa63e36b76fdaad77386f024f` (v7.0.0) with if-no-files-found=error.

Fixed-U `docs` has 14 recursive-tree entries: 2 trees and 12 blobs, with zero tracked symlinks. The artifact nevertheless publishes the whole docs directory, not only `docs/changelog`. That includes ordinary public project documentation and large static assets. Publication-scope review must therefore be whole-directory and fail closed on unexpected generated/runtime files.

## Deployment/publication contract

`deploy` has `needs: build`, runs on mutable `ubuntu-latest`, targets environment `github-pages`, sets environment URL from `steps.deploy.outputs.page_url`, and calls `actions/deploy-pages@v5`.

The observed deploy action:
- requests an OIDC token via `core.getIDToken()`;
- looks up exactly one artifact named `github-pages` in the same workflow run;
- creates a Pages deployment using the artifact id, build version and OIDC token;
- returns page_url/preview_url;
- polls deployment status until success or terminal failure.

Defaults are 600000 ms maximum timeout, 10 status-report errors and 5000 ms initial reporting interval with capped backoff/jitter. Missing or duplicate artifact fails. OIDC acquisition failure explicitly points to `id-token: write`; 403 creation failure points to `pages: write`; 404 points to Pages enablement; 5xx remains server failure. Permanent statuses include deployment failure/content failure/cancel/lost. Timeout, too many polling errors, SIGINT and SIGTERM attempt explicit deployment cancellation.

No source statement proves environment protection rules, Pages configuration, runner trust or service availability; those remain later host evidence.

## Direct callers and consumers

Direct caller/admission owner: GitHub Actions push/manual-dispatch event handling.

Direct machine consumers:
- checkout and setup-python actions;
- exact changelog generator;
- upload-pages-artifact and Actions artifact service;
- deploy-pages, GitHub Pages deployment API and `github-pages` environment.

Direct human consumers: changelog maintainers who edit root `changelog.txt` and readers of the published Pages site. There is no `workflow_call` caller and no Telegram/Fabushi client runtime consumer.

## Security/privacy and failure-recovery boundaries

The source workflow gives pages-write and id-token-write to the build job even though publication occurs only in deploy. Checkout persists repository credentials by default before source-controlled Python executes. Mutable tags, ubuntu-latest, `3.12`, GitHub service state and action tool-cache/download paths are not immutable provenance.

Whole-docs publication is broader than the changelog. Fixed-U has no tracked docs symlink, but archive dereference semantics mean later runtime content must be inspected before publication. A generated static page or successful upstream deployment is not Fabushi release acceptance and does not authorize target deployment.

Failures include trigger/path drift, concurrency cancellation, checkout/setup drift or failure, generator I/O/parse failure, unexpected docs scope, archive/upload failure, missing/duplicate artifact, OIDC/permission failure, Pages-disabled/server failure, terminal deployment failure, polling error threshold, timeout and explicit cancellation. Recovery is a fresh exact execution/qualification on an authorized frozen candidate; no cancelled or partial state is promoted to success.

## Existing-owner-first disposition

Use:
- root repository release/changelog/process governance in `AGENTS.md`;
- current Fabushi release/package process, including `.github/workflows/release-macos-main.yml` and `desktop/package.json`, only as target owner context;
- shared `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- shared `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md` / `FQT-REL-001`.

Reject:
- copying Telegram's changelog Pages workflow into Fabushi;
- a second release/version/changelog acceptance truth;
- shipping updater/update-feed ownership;
- Product Tasks/Automations;
- Agent/Coordinator/Host/Runner product runtime;
- platform-core service;
- Telegram-branded Pages/navigation authority.

The current checkpoint has no target changelog Pages workflow. This source entry is repository-process provenance and requires no production or platform-core delta.

## Authored validation — not run

Contracts (17):
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-TRIGGER-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PATH-FILTER-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PERMISSIONS-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CONCURRENCY-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-JOB-GRAPH-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-BUILD-RUNNER-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-LINEAGE-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PYTHON-LINEAGE-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-GENERATOR-CALL-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DOCS-SCOPE-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PAGES-ARTIFACT-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-ENVIRONMENT-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-OIDC-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-STATUS-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PAGE-URL-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CONSUMERS-001`
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-NO-TARGET-COPY-001`

Integration (1):
- `INT-TDRP-CHANGELOG-WORKFLOW-REPOSITORY-RELEASE-001`

Fault/recovery (8):
- `FAULT-TDRP-CHANGELOG-WORKFLOW-CONCURRENT-CANCEL-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-SETUP-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-GENERATOR-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-ARCHIVE-UPLOAD-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-ARTIFACT-IDENTITY-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-CREATE-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-POLL-TIMEOUT-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-PUBLICATION-SCOPE-001`

Security/provenance (7):
- `SEC-TDRP-CHANGELOG-WORKFLOW-LEAST-PRIVILEGE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-OIDC-JOB-SCOPE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-CREDENTIALS-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-MUTABLE-ACTION-TAGS-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-RUNNER-PYTHON-MUTABILITY-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-WHOLE-DOCS-SCOPE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-NO-SOURCE-PUBLISH-AUTHORITY-001`

Oracles:
- `ORA-TDRP-CHANGELOG-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-CHANGELOG-WORKFLOW-ACTION-LINEAGE-001`
- `ORA-TDRP-CHANGELOG-WORKFLOW-ARTIFACT-DEPLOY-001`
- `ORA-TDRP-CHANGELOG-WORKFLOW-FAILURE-BOUNDARY-001`

Invariants:
- `INV-TDRP-CHANGELOG-WORKFLOW-NO-MUTABLE-TAG-TRUST-001`
- `INV-TDRP-CHANGELOG-WORKFLOW-NO-PARTIAL-DEPLOY-SUCCESS-001`
- `INV-TDRP-CHANGELOG-WORKFLOW-NO-WHOLE-DOCS-BLIND-PUBLISH-001`
- `INV-TDRP-CHANGELOG-WORKFLOW-NO-SOURCE-RELEASE-AUTHORITY-001`
- `INV-TDRP-CHANGELOG-WORKFLOW-NO-STAGE-A-EXECUTION-001`

All remain Stage-A authored-only. No workflow run/status/check is queried, and no dispatch/push, action execution, generator, artifact upload, OIDC/Pages deployment, build, test, package, release, installation or runtime acceptance is executed.

## Queue continuity

Exactly one independent fixed-U source entry is represented. The next independent blob is `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082` at recursive-tree index 121 zero-based.
