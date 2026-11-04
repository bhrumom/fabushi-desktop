# Fixed-U complete read — unused styles updater workflow

- Task round: 176
- Source evidence round: 207
- Desktop checkpoint parent: `ceeef9bd392c012f9b3204c7898d5457a3e23f00`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/unused_styles_updater.yml@2c021d1cd09f7ac59b0e0d5ef6be2bf546167274`
- Tree index: 132 zero-based; mode 100644; 330 ASCII bytes/code points; 16 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-UNUSED-STYLES-UPDATER-WORKFLOW-001`
- Stage-A validation: `not-run`; accepted: false

## Trigger, permissions and runner

The workflow admits exactly three trigger classes: repository dispatch type `Restart unused_styles_updater workflow.`, monthly cron `0 3 1 * *`, and manual `workflow_dispatch`. There is no push, pull_request, release, workflow_call or workflow_run trigger.

The single `Unused-styles` job runs on mutable `ubuntu-latest`. The workflow has no explicit `permissions`, environment, concurrency group, timeout, retry or checkout step. Effective `github.token` permissions are runtime repository/organization policy. Source behavior needs repository clone/push and PR-create authority to publish, but source review does not infer those permissions are actually granted.

All 20 fixed-U workflow blobs were checked for `unused_styles_updater` and `unused-styles`; only this source contains those strings. The direct workflow callers are GitHub's monthly scheduler, an external repository-dispatch producer not identified in fixed-U, and an authenticated manual dispatcher. Sibling copyright-year/master/user-agent workflows use the same external Action under other job types only.

## External Action lineage

The workflow references mutable `desktop-app/action_code_updater@master`. On 2026-11-02 master resolves to `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`; branch metadata reports `protected=false`, and the commit is unsigned.

Observed complete Action tree:
- `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`
- `check_unused_styles.py@6a061912121f49ffa386ff9692b989378b6bdd2b`
- `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`

No LICENSE, package.json or package lock is present in the observed repository tree. This round records behavior/provenance only and does not copy Action code into Fabushi.

The Action is composite. It performs `npm init -y`, then two unversioned `npm install` operations for `simple-git`, `isbinaryfile`, `@actions/core` and `@actions/github`, then launches Node with `github.token` and `unused-styles` as argv. Node/npm/Python versions therefore come from the mutable runner. Exact package/transitive identities are later frozen-candidate evidence.

## Clone and submodule contract

For `unused-styles`, clone options are recursive submodules, `--depth=1`, `--shallow-submodules`, and `--no-single-branch`. The clone URL contains `github.token`. No event SHA/ref is supplied, so the Action operates on the repository's then-current default branch rather than fixed U. Current tdesktop repository metadata reports default branch `dev`; actual execution HEAD remains host evidence.

`--no-single-branch` intentionally makes remote `unused_styles_*` branches visible for collision detection. The clone is still shallow.

Submodule paths are obtained from `git submodule status --recursive`. Any command exception becomes an empty list. For tdesktop, an empty list is rejected before scanning. A non-empty but incomplete/misparsed list is not independently detected. The parser takes the second whitespace-delimited token from each status line, so exact runtime path evidence matters.

## Style definition discovery

For tdesktop:
- definitions dirs: cloned `Telegram`
- search dirs: cloned `Telegram`
- definition/removal excludes: all parsed recursive submodule roots

Definition files are `.style` files. The parser removes block comments while preserving line count, strips `//` comments, tracks brace depth, ignores `using` lines, skips PascalCase struct declarations followed by `{`, and collects top-level `name:` definitions.

A definition is protected when `[[maybe_unused]]` occurs on its line or in the preceding contiguous run of blank/comment lines before another non-comment line. Protection is accumulated by name, so one protected definition suppresses removal for all definitions sharing that name.

Definition exclusion is deliberately different from reference search: submodule paths are excluded only from definition/removal discovery. References are searched across the entire `Telegram` search tree, including initialized submodules.

## Reference discovery

C++ reference search scans only `.cpp`, `.h` and `.mm` and recognizes exact `st::name` references. Other dynamic/string/reflection-like usages are invisible to that detector.

Style reference search scans all `.style` under the search root, strips comments, ignores `using`, tokenizes identifiers after a definition colon or across other style lines, and also recognizes parenthesized identifiers.

Files are read as UTF-8 with replacement on decoding errors. Read exceptions during reference scans are skipped. These choices can create false absence evidence. References outside `Telegram`, syntax not matched by the parsers, runtime-generated names, malformed encoding or unreadable files are therefore fail-closed review conditions.

## Unused-set and cascading mutation

The first-pass unused set is:
`all definition names - C++ references - style references - protected names`.

Removal groups unused definitions by file. A one-line entry removes that line. An entry with unmatched opening braces removes through the line where its brace depth closes. The range scanner strips only `//` comments while counting braces.

After range deletion, the writer collapses consecutive blank lines across the entire rewritten file, not only around the removed range. It writes UTF-8 text back to disk.

The tool then re-analyzes the modified source. If removing one unused entry causes a dependency it referenced to become unused, a later pass removes that newly orphaned entry. Passes continue until the unused set is empty. This cascading behavior is part of source semantics and must be visible in any future diff review.

The Python process prints a machine-readable `[summary] removed=N` line. The Node wrapper uses a regex to recover `N`. The subprocess output buffer is capped at 100 MiB.

If no entries are unused, the Python script reports removed=0 and the Node wrapper raises `No unused entries found.`; the top-level handler converts that exact condition into exit 0. If output format drifts so the summary cannot be parsed after actual file mutation, the wrapper also sees removedCount=0 and can exit as a no-op before Git publication. Process success is therefore not sufficient mutation/publication evidence.

## Branch, commit and pull-request publication

`UnusedStyles` captures Action-local year/month at construction time and derives `unused_styles_YYYY_MM`. The workflow cron is UTC, but Action-local timezone is not declared; repository/manual dispatch can also run near any month boundary.

After scanning, `processGit` reads local/remote branches. It rejects when any branch string contains the derived branch name, which is a substring test rather than exact-ref equality.

For a non-colliding branch it:
1. resets origin to the token-bearing HTTPS URL;
2. configures `GitHub Action <action@github.com>`;
3. creates local `unused_styles_YYYY_MM`;
4. stages `.`;
5. commits `Removed N unused style entries.`;
6. verifies latest log message;
7. pushes `-u origin <branch>`.

The PR title is the commit message. The PR body states the removed count and that references were checked across `.cpp`, `.h`, `.mm` and `.style`. PR base is the cloned current branch.

PR discovery uses an unpaginated pull-request list. If no matching head/base appears on that page, the Action attempts PR creation. The entire list/create block catches errors, logs them, and does not rethrow. Consequently branch push may succeed while PR creation fails and the Action still finishes apparently successfully. A preexisting matching PR outside the returned page can also cause a duplicate-create attempt whose error is swallowed.

The Action never merges the PR. Automated cleanup remains a proposed source diff requiring repository review/policy.

## Failure, recovery and security

Fail-closed boundaries:
- external repository-dispatch producer/authentication is not in fixed-U;
- effective token permissions are implicit;
- mutable `ubuntu-latest`, Action master, Node/npm/Python and runtime npm packages can drift;
- Action master is unprotected and observed commit unsigned;
- observed Action repository has no license metadata/file;
- token is passed on process argv and embedded in clone/remote URL;
- runtime default-branch HEAD is not event/fixed-U identity;
- recursive shallow submodule acquisition can drift/fail;
- empty submodule discovery fails closed, but incomplete non-empty parsing is not detected;
- parser/reference false negatives can propose deletion of semantically used styles;
- read exceptions and UTF-8 replacement can suppress references;
- range/brace parsing and global blank-line normalization can change more text than a single logical value;
- cascading passes can amplify an early false-negative decision;
- 100 MiB output-buffer overflow is a hard subprocess failure;
- missing/malformed summary can hide local mutation as no-op;
- month/timezone changes can alter branch identity;
- branch collision is a hard failure;
- clone/read/write/commit/push failures stop publication;
- PR list/create failures are swallowed after push;
- no concurrency fence, retry, transaction or rollback exists;
- workflow logs expose source paths/style names and must not become unnecessary durable sensitive evidence.

Recovery requires fresh frozen-candidate binding of action/runner/runtime packages, exact cloned default-branch/submodule commits, complete discovered definition/reference sets, before/after bytes, per-pass unused sets, branch commit/tree and PR identity. Human review must distinguish genuine source cleanup from parser blind spots. A branch without PR, no-op exit, or green workflow alone is not successful publication evidence.

## Existing-owner-first disposition

Applicable guarantees map to existing Fabushi root repository source-maintenance/process governance, repository security/provenance governance and shared `TEST_STRATEGY.md` / `evidence-contract.md`. Fabushi's design-system and UI source remain their existing owners; Telegram's Qt `.style` parser is not imported as a second design-system/source truth.

Rejected owners: copied Telegram unused-style workflow, Telegram style subsystem, second design-system/source-maintenance truth, Product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, shipping updater, provider/sidecar and platform-core service.

No production runtime, product UI, design-system implementation, source-cleaner workflow or platform-core code is changed for this repository-only source responsibility.

## Authored Stage-A cases

Contracts (34):
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-EXACT-SOURCE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-TRIGGERS-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-DISPATCH-CALLER-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-MANUAL-DISPATCH-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-RUNNER-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-PERMISSIONS-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-ACTION-LINEAGE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-ACTION-LICENSE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-RUNTIME-DEPS-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-TOKEN-DELIVERY-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-CLONE-MODE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-DEFAULT-BRANCH-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-SUBMODULE-DISCOVERY-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-DEFINITIONS-SCOPE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-REFERENCE-SCOPE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-STYLE-PARSER-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-PROTECTION-MARKER-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-CPP-REFERENCE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-STYLE-REFERENCE-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-UNUSED-SET-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-CASCADE-PASSES-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-ENTRY-RANGE-REMOVAL-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-BLANK-LINE-NORMALIZATION-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-ENCODING-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-SUMMARY-COUNT-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-NOOP-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-BRANCH-NAMING-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-BRANCH-COLLISION-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-COMMIT-PUSH-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-PR-TITLE-BODY-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-PR-PUBLICATION-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-PR-ERROR-BOUNDARY-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-CALLERS-CONSUMERS-001`
- `CONTRACT-TDRP-UNUSED-STYLES-WORKFLOW-NO-TARGET-COPY-001`

Integration (1):
- `INT-TDRP-UNUSED-STYLES-WORKFLOW-REPOSITORY-SOURCE-MAINTENANCE-001`

Fault/recovery (19):
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-DISPATCH-TYPE-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-PERMISSION-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-MUTABLE-MASTER-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-NPM-ACQUISITION-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-RUNNER-PYTHON-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-DEFAULT-BRANCH-DRIFT-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-CLONE-SUBMODULE-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-SUBMODULE-STATUS-EMPTY-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-SUBMODULE-PARSE-INCOMPLETE-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-REFERENCE-FALSE-NEGATIVE-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-INVALID-UTF8-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-ENTRY-RANGE-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-SUMMARY-MISSING-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-NO-UNUSED-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-BRANCH-EXISTS-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-COMMIT-PUSH-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-PR-SWALLOWED-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-OVERLAPPING-RUNS-001`
- `FAULT-TDRP-UNUSED-STYLES-WORKFLOW-NO-ROLLBACK-001`

Security/provenance (15):
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-IMPLICIT-TOKEN-SCOPE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-MUTABLE-ACTION-MASTER-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-ACTION-NO-LICENSE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-UNPINNED-NPM-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-TOKEN-ARGV-REMOTE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-MUTABLE-RUNNER-RUNTIMES-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-RECURSIVE-SUBMODULE-PROVENANCE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-DEFAULT-BRANCH-NOT-EVENT-SHA-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-STYLE-MUTATION-BOUNDARY-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-LOG-METADATA-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-BRANCH-PR-WRITE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-REVIEW-BEFORE-MERGE-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-NO-SOURCE-MAINTENANCE-AUTHORITY-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-NO-TARGET-DESIGN-TRUTH-001`
- `SEC-TDRP-UNUSED-STYLES-WORKFLOW-NO-STAGE-A-EXECUTION-001`

Oracles:
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-EXACT-SOURCE-001`
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-TRIGGER-PERMISSION-LINEAGE-001`
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-DISCOVERY-REFERENCE-001`
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-MUTATION-001`
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-PUBLICATION-001`
- `ORA-TDRP-UNUSED-STYLES-WORKFLOW-FAILURE-BOUNDARY-001`

Invariants:
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-RUNTIME-HEAD-NOT-FIXED-U-001`
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-SUBMODULE-DEFINITIONS-NOT-MUTATED-001`
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-REFERENCE-ABSENCE-NOT-SEMANTIC-PROOF-001`
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-BRANCH-PR-NOT-MERGE-001`
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-PARTIAL-PR-NOT-SUCCESS-001`
- `INV-TDRP-UNUSED-STYLES-WORKFLOW-NO-STAGE-A-EXECUTION-001`

All are authored only. No executable qualification was run.
