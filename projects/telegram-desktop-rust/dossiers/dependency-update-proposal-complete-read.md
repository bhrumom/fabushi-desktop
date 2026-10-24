# Dependency-update proposal complete read — strict index 75

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Source: `.github/dependabot.yml@5ace4600a1f26e6892982f3e2f069ebfab108d87`, 6 lines / 118 bytes; complete read and SHA revalidated.
- Applicable responsibilities: bounded dependency-update discovery, reviewable exact-version proposal creation, provenance preservation, canonical owner review, exact-head verification, and fail-closed provider outage/staleness semantics.
- Provenance-only policy: Dependabot provider syntax, `github-actions` ecosystem selector, root directory and weekly cadence.
- Exact current Fabushi `.github` tree has no parallel `dependabot.yml`; current `.github/workflows` tree is `621e4f4a157858d784908ef397fab1eaeafb621a`.
- Existing owner evidence: `AGENTS.md@e94952bdf075bf87fa4e9f9a003b2d97a3e34066` owns atomic repository changes and exact-head verification; declared workflow/package refs remain owned by their current files/lockfiles rather than an update bot.
- Parent exact head/tree: `69c8375ba01f5699096d085a45a37c4a5d13b441` / `df8e8cd35e08ef0725cf9d120f550d68af2768f6`.
- Parent TDRP run `39742062788` completed success, 61/61; authority `12409297677` / `sha256:6f545db7b868e955b66834f2f3557868ff20bb174d4c9ff5ec925668a7fba35f`; closure `12409578417` / `sha256:9f7eb73d3fb61d04fefc4388e7a7c2c2098dd90821e0bbca6bdc2b8a27910ba2`; reachability `12409124345` / `sha256:db8d7ce0dbf202bd394bf36f2b71aff2ac98f6f15d4002d943afacff902d8791`.
- Parent Rust run `39742062809` completed success, 6/6; artifact `12410031793` / `sha256:4dddbb8bd245d450a2b1b9e79f9481a2b628a1626714d9b080e31388a85f9e1b`.

## Existing-owner-first disposition
Dependency-update automation is a replaceable proposal adapter over current manifest/lockfile/workflow owners and the ordinary review/atomic-publication lifecycle. It may discover and propose a bounded version/ref change but cannot own dependency truth, merge permission, security/license acceptance, release publication, source closure or migration acceptance. Missing, disabled or stale provider state remains incomplete maintenance evidence. No Dependabot runtime, second package owner, update store, release root or security authority is introduced; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-DEPENDENCY-UPDATE-PROPOSAL-001`
- Oracles: `ORA-TDRP-DEPENDENCY-UPDATE-BOUNDED-SCOPE-001`, `ORA-TDRP-DEPENDENCY-UPDATE-REVIEWABLE-PROPOSAL-001`, `ORA-TDRP-DEPENDENCY-UPDATE-PROVIDER-FAIL-CLOSED-001`
- Invariants: `INV-TDRP-DEPENDENCY-UPDATE-NO-AUTO-ACCEPT-001`, `INV-TDRP-DEPENDENCY-UPDATE-NO-SECOND-OWNER-001`, `INV-TDRP-DEPENDENCY-UPDATE-NO-PROVIDER-AS-CLEAN-STATE-001`
- Contracts: `CONTRACT-TDRP-DEPENDENCY-UPDATE-ECOSYSTEM-SCOPE-001`, `CONTRACT-TDRP-DEPENDENCY-UPDATE-PROVENANCE-001`, `CONTRACT-TDRP-DEPENDENCY-UPDATE-OWNER-REVIEW-001`
- Fault/recovery: `FAULT-TDRP-DEPENDENCY-UPDATE-PROVIDER-UNAVAILABLE-001`, `FAULT-TDRP-DEPENDENCY-UPDATE-STALE-PROPOSAL-001`, `FAULT-TDRP-DEPENDENCY-UPDATE-CONFLICTING-PROPOSALS-001`
- Security: `SEC-TDRP-DEPENDENCY-UPDATE-SUPPLY-CHAIN-PROVENANCE-001`, `SEC-TDRP-DEPENDENCY-UPDATE-NO-BOT-RELEASE-AUTHORITY-001`
- Regression: `REG-TDRP-DEPENDENCY-UPDATE-NO-DEPENDABOT-RUNTIME-001`, `REG-TDRP-DEPENDENCY-UPDATE-EXACT-HEAD-GATES-001`

Closing strict index 75 changes counters to unknown=16,228, unread=16,203, omitted=0, responsibilities=93. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 76 `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0`.
