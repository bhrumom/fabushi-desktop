# Dependency Watch release-trust reference — complete read

Task round: 121  
Evidence round: 152  
Stage: A / executable validation deferred  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32`

## Exact source and direct consumer

The target blob was read completely: 5,849 bytes / 95 lines, fixed-U recursive tree index 61. It is the release-adoption trust policy used by Dependency Watch; it does not install, approve deployment, or itself mutate a dependency.

The direct in-repository workflow consumer is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`. That skill requires reading this reference before recommending any candidate and explicitly separates update urgency from the decision to update, backport, hold, skip, or track. Repository code search found no additional direct `release-trust.md` consumer. Exact fixed-U `scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` and `scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` contain no release-trust field/parser reference, so they do not create a second release-decision state owner.

## Release identity and publication provenance

For each actionable candidate, evidence binds publication date, canonical source, exact tag object and peeled commit (or package revision/artifact digest), and the source form the recipe actually consumes: Git, release archive, binary, or distro package. Mutable tags/assets are rechecked against the previous observation; a changed tag or asset requires an explained replacement rather than silent acceptance.

Release/advisory research includes upstream withdrawals, compromise reports, regressions affecting enabled features/platforms, and—where exposure warrants it—an independent authoritative security tracker or equivalent source. Reposts of one announcement are not independent validation. Unavailable checks remain explicit unknown evidence.

Relevant source and package/build deltas are part of the trust boundary: new install hooks, generated scripts, binary/test blobs, downloads, unexpected dependency changes, and changes in publisher, signer, repository ownership, or release process all raise investigation scope without automatically proving compromise.

## Signature, attestation, checksum and trust boundaries

Signatures, attestations, reproducibility evidence and checksums are useful only when their verification actually succeeds. Signer identity is compared with established project history or separately published keys when available. A verified signature establishes attribution/integrity only within the assumed trust chain; it does not prove benign code, because an authorized or compromised maintainer can sign malware. Conversely, an unsigned project is not automatically compromised. A checksum published on the same compromised release page is not independent proof.

If an archive includes generated files absent from Git and the recipe consumes the archive, those differences are reviewed. Git builds still require source/build-input review. Known malicious releases are not downloaded or executed by the advisory monitor, and build/test success cannot rule out a targeted backdoor.

## Regression, observation hold and decisions

The decision states are:

- **Update:** benefit warrants adoption after relevant release-trust and regression checks have no unresolved concern; later implementation validation is proposed without claiming the release is proven safe.
- **Backport:** a reviewed applicable fix and prerequisites can be taken with less disruption; the backport receives the same trust assessment.
- **Hold:** the candidate is fresh, evidence is missing, a relevant regression is unresolved, or a trust change needs investigation. The missing check/release condition and review point are recorded; missing evidence is not described as compromise.
- **Skip:** the exact candidate is withdrawn, known compromised, incompatible, or superseded. Evidence and an alternative are named; the project is not globally blacklisted.
- **Track:** no adoption case is established yet, such as a major/milestone gap without a verified applicable benefit.

Routine bugfix releases use seven days after publication as the default observation window. This is configurable caution against fresh regressions, not a safety guarantee. Time alone never clears a trust or compatibility concern. A confirmed applicable security fix—especially with active exploitation—can justify immediate reviewed adoption or minimal backport; the tradeoff is recorded. An unverified CVE/version match does not establish urgency. If both staying and upgrading are risky, the finding remains pressing with a reviewed fix, mitigation, or alternative rather than indefinite silence.

Durable candidate state persists `decision`, `decision_reason`, `release_published_at`, `source_identity`, `trust_checks`, `regression_checks`, `unknown_checks`, and `review_after` or `review_condition`. Older findings without those fields require assessment and do not inherit an unconditional update recommendation. Holds/skips do not erase unresolved vulnerabilities.

## XZ malicious-release regression fixture and current fixed-U consumer facts

The reference treats the XZ incident as a standing case, not a permanent ban. It names compromised 5.6.0 and 5.6.1 release tarballs and requires those candidates to be rejected while later releases are assessed from current evidence. It also warns that the historical initial Telegram snapshot example—prepare 5.4.5 and Docker 5.8.1, both using Git+CMake and outside the named compromised pair—must be re-established from each snapshot rather than inherited as blanket clearance.

Current fixed-U no longer matches that historical pair:

- `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b` clones `https://github.com/tukaani-project/xz.git` on branch `v5.4`, then checks out immutable commit `05af863c3166cddeb4ab935829b14e995c8346b4` and builds with CMake. GitHub identifies that commit as a 2026-09-09 maintenance-branch commit.
- `Telegram/build/docker/centos_env/Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2` shallow-clones tag `v5.8.4` and builds with CMake. Current upstream tag evidence is annotated tag object `9151b328e76bbc468aa64f85c741886b6227cec1`, peeled commit `d3e650e63c110e830fd5391e7f8b45df0b91d3da`, tagger date 2026-09-09. GitHub reports `verification.verified=true`; that fact is provenance evidence only and is explicitly not a safety clearance.

This delta is not a dependency-update recommendation. It is evidence for the contract that release identity and trust must be rebuilt from the exact inspected snapshot.

## Existing-owner-first resolution

No new runtime, provider, task owner, updater, dependency decision store, release monitor, ADR, or platform-core production delta is introduced. The reference specializes the already implemented `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` responsibility under existing Fabushi repository quality/test-evidence governance.

The root `AGENTS.md` repository dependency-advisory owner already covers source-neutral exact snapshot provenance, release/advisory/regression/trust evidence, signature/attestation/checksum limitations, known-malicious non-execution, stable findings, and strict separation from product updater/build/test/backport/release/deployment authority. Therefore this round does not create another production owner; it adds the finer release-trust qualification/evidence oracles to the existing Test Strategy and Evidence Contract.

Rejected owners remain:
- `source/electron-main/update/*`: signed product update/install/rollback, not repository dependency release-adoption trust.
- platform-core runtime: no repository dependency-advisory lifecycle.
- build-time acquisition inventory: source/build-input evidence only, not release trust/decision authority.
- Product Tasks/Automations or Agent/Coordinator/Host/Runner: shipping execution state, not repository supply-chain advisory findings.
- copied Telegram-specific Python monitor: would create a source-specific second owner.

## Authored validation, not run

- `CONTRACT-TDRP-DEPENDENCY-RELEASE-PROVENANCE-001`
- `CONTRACT-TDRP-DEPENDENCY-RELEASE-DECISION-001`

Both remain `validation_status=not-run`; no build, test, validator, generator, dependency resolution, malicious artifact download/execution, package, deployment, installation, or runtime acceptance was performed.
