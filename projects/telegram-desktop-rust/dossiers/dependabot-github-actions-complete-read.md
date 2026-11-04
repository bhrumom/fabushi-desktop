# Fixed-U `.github/dependabot.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source path/blob: `.github/dependabot.yml@5ace4600a1f26e6892982f3e2f069ebfab108d87`
- Tree index: 110 zero-based; mode `100644`; Git size 118 bytes
- Decoded content: 118 ASCII/Unicode code points, 6 LF, 0 CR, trailing newline, 7 split lines
- Triage: `full-read` (no prior exact-blob closure found in current ledger/dossiers/inventory)
- Target responsibility: `TDRP-R9-REPOSITORY-DEPENDABOT-GITHUB-ACTIONS-001`

## Complete source semantics

The file contains Dependabot schema `version: 2` and exactly one `updates` entry. That entry selects `package-ecosystem: "github-actions"`, repository-root `directory: "/"`, and `schedule.interval: "weekly"`. It declares no additional keys.

The contract is therefore narrow: GitHub Dependabot is asked to inspect GitHub Actions dependency references under the repository-root scope and periodically propose eligible updates on a weekly cadence. This is repository-host automation metadata, not application runtime behavior and not Telegram client dependency loading logic.

## Direct consumers and dependency surface

Direct host consumer: GitHub Dependabot's repository configuration loader. Human consumers are maintainers/reviewers of any generated dependency-update proposals.

The contemporaneous fixed-U tree contains 20 workflow files under `.github/workflows/`. Complete semantic inspection of those workflow blobs confirms action references span three trust/identity forms: immutable commit-SHA pins (for example checkout/cache/upload/download and several Docker/Azure actions in canary workflows), version-tag pins (for example `actions/checkout@v7`, `actions/cache@v6`, `actions/stale@v11`), and mutable branch/tag references (for example `desktop-app/action_code_updater@master`, `Eden-CI/msvc-dev-cmd@master`, `telegramdesktop/winget-releaser@main`). The Dependabot source config does not itself enumerate or freeze these references; directory/ecosystem only declares the update discovery scope.

## Source-lineage, security and quality contract

A Dependabot proposal is navigation/change-candidate evidence, not adoption authority. Weekly cadence does not prove freshness of every upstream, security applicability, release trust, compatibility, absence of compromise, safe auto-merge, or release readiness. Mutable workflow refs remain mutable until separately reviewed/pinned; a proposal cannot convert them into immutable provenance.

Security assessment remains with the existing source-neutral repository dependency-advisory / supply-chain owner. A candidate requires the same release identity, vulnerability applicability, fork/copy provenance, trust/regression checks, decision state and later executable validation required by current Fabushi governance. Build/test success alone cannot establish supply-chain safety, and absence from one advisory database cannot establish safety.

The source has no credentials/registry declaration, no auto-merge rule, no target branch, labels/reviewers, ignore/allow policy, grouping, open-PR limit, rebase strategy, security-only mode, commit-message policy, or deployment/release instruction. These absences must not be invented as implied behavior.

## Existing-owner-first disposition

Use the existing Fabushi repository dependency-advisory / supply-chain quality owner, repository process governance, and security/quality evidence. Do not create a Dependabot-specific product owner, second dependency monitor, product updater, platform-core service, release engine, or automatic merge service.

The current target checkpoint has no root `.github/dependabot.yml`. Therefore this Stage-A closure records source provenance and contracts only; it does not copy Telegram's Dependabot configuration, enable target Dependabot, generate a dependency PR, or execute any workflow.

## Authored validation (not run)

- `CONTRACT-TDRP-DEPENDABOT-SCHEMA-001`
- `CONTRACT-TDRP-DEPENDABOT-ECOSYSTEM-DIRECTORY-001`
- `CONTRACT-TDRP-DEPENDABOT-WEEKLY-SCHEDULE-001`
- `CONTRACT-TDRP-DEPENDABOT-WORKFLOW-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-DEPENDABOT-PROPOSAL-NONAUTHORITY-001`
- `INT-TDRP-DEPENDABOT-DEPENDENCY-OWNER-001`
- `FAULT-TDRP-DEPENDABOT-MISSING-WORKFLOW-SCOPE-001`
- `FAULT-TDRP-DEPENDABOT-MUTABLE-REF-TRUST-001`
- `SEC-TDRP-DEPENDABOT-NO-AUTO-ADOPTION-001`
- `SEC-TDRP-DEPENDABOT-NO-SECURITY-CLEARANCE-001`

All remain Stage-A authored-only: no validator, Dependabot execution, workflow run, dependency mutation, build, lint, test, E2E, packaging, deployment, release or acceptance was executed.
