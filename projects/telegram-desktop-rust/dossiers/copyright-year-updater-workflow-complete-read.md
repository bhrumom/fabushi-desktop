# Copyright year updater workflow complete read — strict index 83

## Authority
- Upstream: `telegramdesktop/tdesktop@42e2af80cdd20a99edecb4255470f0aa6bae6bcb`, tree `19740740209ea84bf3768f3f63ca92ea8d861087`.
- Source: `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082`, 17 line slots / 338 bytes; complete read and SHA revalidated immediately before publication.
- Provider action mutable ref: `desktop-app/action_code_updater@master` resolved immediately before publication to `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`.
- Provider bundle: `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`, `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`.
- Existing migration owners: TDRP-SRC-03 `documentation_or_license` and module-map `build/generator/resources/dependencies`; ordinary repository PR/review remains the mutation/acceptance boundary.
- Parent exact head/tree: `4261f91555c49daba7ac761aa1439d8b677f6a4c` / `976d178f0c21f8613817c3f9018cbcec8d7c0030`.
- Parent exact-head 4261f91555c49daba7ac761aa1439d8b677f6a4c completed TDRP run 39768716075 at 61/61 success (authority 12421323101 sha256:6f77d0ea6cff72cfd1b26d2b4c10ec6cdc5ee1235dc77d83a9a7138ca540fd99; closure 12421337564 sha256:834c7d0b823ea4005d78d83ed19d9380d9156925b26ac936e100d3c19dd7258b; reachability 12421028190 sha256:cd24aeedea5f4a0ee5c1fa9b536b4ad48f0d3c130865e0a0765d16ba140e9d8f) and Rust desktop runtime run 39768715965 at 6/6 success (artifact 12421582351 sha256:a60d59a9e4af0d3f1ed00a4ddb0e47c96799fd1461d78445abb553c8bcae187b).

## Complete-read behavior
The workflow has two triggers only: repository dispatch type `Restart copyright_year_updater workflow.` and the yearly schedule `0 3 1 1 *`. Its only job runs on `ubuntu-slim` and invokes the external updater with `type: license-year`.

At the resolved provider commit, `LicenseYear` derives the current year at runtime, looks for the previous-year range beginning at 2014, and replaces the matching textual range. The provider recursively scans non-binary repository files while excluding `.git`. No modified files is treated as a clean no-op. When changes exist, the provider creates `copyright_to_YEAR`, commits, pushes, and attempts to open a pull request if an equivalent PR is absent.

## Existing-owner-first disposition
This is a repository legal-record maintenance adapter, not product behavior. TDRP-SRC-03 already defines `documentation_or_license` as research/update/retention of necessary legal provenance without fabricating runtime migration, while the module map assigns build/generator/resources/dependencies and licenses to Fabushi's own CI/packaging/review context.

No new copyright runtime, legal service, release, packaging, source store, acceptance owner, UI route, Rust domain, or TypeScript feature is introduced. The durable canonical state is the reviewed repository revision; provider branches and PRs are proposals.

## Fail-closed and security conclusions
- `desktop-app/action_code_updater@master` is mutable. Its current resolved commit is audit provenance only, not Fabushi immutable authority.
- The composite action performs runtime `npm install` operations without immutable package locks in the action definition. Those resolutions are also mutable provider provenance.
- The provider receives `github.token` and performs authenticated clone/push/PR operations; reviewed repository diffs remain the only canonical source boundary.
- The text matcher is intentionally narrow but the scan is repository-wide; any proposed diff must remain reviewable rather than being auto-accepted as legally correct.
- An existing yearly branch, push failure, token failure or provider drift is non-passing maintenance evidence.
- `processPullRequest` catches/logs API errors internally, so a provider process exit cannot alone prove that a PR was actually created.
- A no-change run means only that the provider found no matching previous-year text; it is not proof of overall license/copyright completeness.
- Neither workflow success nor a legal-text update can create shipping release ACCEPT or satisfy TDRP source closure.

## Exact traceability IDs
- Requirement: `TDRP-R9-COPYRIGHT-YEAR-LEGAL-PROVENANCE-001`
- Oracles: `ORA-TDRP-COPYRIGHT-YEAR-EXACT-SOURCE-001`, `ORA-TDRP-COPYRIGHT-YEAR-PR-ONLY-001`, `ORA-TDRP-COPYRIGHT-YEAR-NO-RELEASE-AUTHORITY-001`
- Invariants: `INV-TDRP-COPYRIGHT-YEAR-NO-PRODUCT-RUNTIME-001`, `INV-TDRP-COPYRIGHT-YEAR-MUTABLE-PROVIDER-NO-AUTHORITY-001`, `INV-TDRP-COPYRIGHT-YEAR-NO-AUTO-ACCEPT-001`
- Contracts: `CONTRACT-TDRP-COPYRIGHT-YEAR-REVIEWABLE-PROPOSAL-001`, `CONTRACT-TDRP-COPYRIGHT-YEAR-PROVENANCE-001`
- Fault/recovery: `FAULT-TDRP-COPYRIGHT-YEAR-PROVIDER-DRIFT-001`, `FAULT-TDRP-COPYRIGHT-YEAR-BRANCH-PUSH-PR-001`
- Security: `SEC-TDRP-COPYRIGHT-YEAR-GITHUB-TOKEN-001`, `SEC-TDRP-COPYRIGHT-YEAR-MUTABLE-SUPPLY-CHAIN-001`
- Regression: `REG-TDRP-COPYRIGHT-YEAR-NO-RUNTIME-001`, `REG-TDRP-COPYRIGHT-YEAR-NO-RELEASE-ACCEPT-001`

Closing strict index 83 changes counters to unknown=16,220, unread=16,195, omitted=0, responsibilities=101. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 84 `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0`.
