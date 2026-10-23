# WinGet publication workflow complete read

## Exact source and parent evidence
- Upstream authority: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Strict98 source: `.github/workflows/winget.yml@0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390`, 700 bytes, fully read.
- Parent Fabushi exact HEAD: `510e15bfd0e0d7191b147cbf6ed65317c9f277a5`, tree `c2a99596b1552608f27f070e85c15473f114cfc0`.
- Parent TDRP run `39899622743` is terminal success with `61/61` jobs successful.
  - authority artifact `12478173024`, `sha256:6710f1de2ac6a1e904f242309a1d799e511041ac50cdc03bb6f13028e28c8702`
  - closure artifact `12478442455`, `sha256:cbd0ab8a0a361d5ce1f1c04ddd3dc554a5b5a7aeef489d0153962c854cb51edb`
  - reachability artifact `12478746234`, `sha256:56e2648b175c73de8c27ece8f778e01eee6f086257c84382ea4d9c23e69d46bd`
- Parent Rust desktop runtime run `39899622803` is terminal `6/6` success; artifact `12478496730`, `sha256:3f2fb828aa73481b4c4287f2c67c0cc3fa2030d542b67f59ba626b9c2e0c0d89`.

## Source responsibility decomposition
- The workflow runs only on GitHub release `released` and `prereleased` events.
- `released` publishes identifier `Telegram.TelegramDesktop`; `prereleased` publishes `Telegram.TelegramDesktop.Beta`.
- Both use `telegramdesktop/winget-releaser@main`, select release assets with `t(setup|portable).*(exe|zip)$`, and pass `WINGET_TOKEN`.
- The caller does not set `max-versions-to-keep`; provider default `0` disables the optional old-version removal branch for this workflow, while merged-branch cleanup still runs.

## Provider and supply-chain provenance
- `telegramdesktop/winget-releaser@main` re-resolved immediately before admission to `4ffc7888bffd451b357355dc214d43bb9f23917e`, tree `0b09c320f951cb196fa758241df02baa6f893f72`; `action.yml@a6de3aab26491d01a7a459c3fcaca8b6d5bce393`; `LICENSE.md@74c892aed62da338fe12600bfba912596f044ced` is AGPL-3.0.
- The provider calls mutable `cargo-bins/cargo-binstall@main`, re-resolved to `8a2045612414e945291083f820cd50b4d8fc2218`, tree `f25f032637b6d602a60199c7893f4441f1bdc210`; `action.yml@bfe4fcd6726e2a272313ea120c3492fa1d979e0e`; repository license metadata is GPL-3.0.
- It then executes `cargo binstall komac -y` without a Komac version pin. Audit-time Komac main is `e9882dcfd22d9bb2defbb0e25044c331d5e4390d`, tree `35a45b7bef8d48fd5ba0723ba7d26cec385dd832`; `Cargo.toml@ac8123ca854d673dee45e716d225f0150bf0a2aa` reports version `2.17.1` and `GPL-3.0-or-later`; `LICENSE.md@2fb2e74d8d7fa1c9286b18af0afa5c00402f56e3`. This is observation, not an immutable install pin.
- Provider state machine: require an existing winget-pkgs package -> validate retention -> bootstrap tools -> query source release/tag -> derive version and matching asset URLs -> `komac update --submit` -> `komac cleanup --only-merged` -> optional removals only when retention is non-zero.
- External catalog/fork/PR mutation never establishes Fabushi signing, package identity, install/startup behavior, device acceptance, promotion, or independent release ACCEPT.

## Existing-owner-first disposition
- Existing canonical owners remain `FQT-EVID-001` in `evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` and `FQT-REL-001` in `release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72`.
- Exact target symbols are unique in the current parity ledger:
  - `相关 source/spec/oracle/baseline 改变后旧证据降 historical。截图/视频存在但未关联 case/verdict 不算 pass。独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。`
  - `frozen exact SHA；applicable specs/oracles/RTM ready；required services/accounts/signing/config available or blocked；fixtures/baselines versioned；known P0=0；workflows target exact source。`
- Preflight found zero prior source-path, responsibility-ID, or source-blob rows for strict98.
- No second WinGet/Telegram release owner, package ledger, credential owner, catalog truth, or release verdict is introduced; `novel_capability=false`.

## Failure, concurrency, recovery, and security boundaries
- `winget-releaser@main`, nested `cargo-binstall@main`, and unpinned Komac remain mutable fail-closed supply-chain edges.
- The workflow has no concurrency group. Overlap, cancellation, token/API failure, release-asset mismatch, catalog/fork races, or outcome-unknown submission cannot advance canonical release acceptance.
- GitHub's workflow token is provider/bootstrap and release-read authority; `WINGET_TOKEN` is external catalog mutation authority only. Neither becomes Fabushi product/session credential authority.
- Missing installer matches, absent catalog package, invalid credentials, provider/tool install failure, submit failure, cleanup failure, or mutable-provider drift are non-passing publication evidence.

## Traceability
- Requirement: `TDRP-R9-WINGET-PUBLICATION-PROVENANCE-001`.
- Oracles: `ORA-TDRP-WINGET-RELEASE-EVENT-BRANCH-001`, `ORA-TDRP-WINGET-ASSET-SELECTION-001`, `ORA-TDRP-WINGET-NO-RELEASE-AUTHORITY-001`.
- Invariants: `INV-TDRP-WINGET-EXTERNAL-CATALOG-NOT-RELEASE-ACCEPT-001`, `INV-TDRP-WINGET-MUTABLE-SUPPLY-CHAIN-FAIL-CLOSED-001`, `INV-TDRP-WINGET-CREDENTIAL-BOUNDARY-001`.
- Contracts: `CONTRACT-TDRP-WINGET-PUBLICATION-PROVENANCE-001`, `CONTRACT-TDRP-WINGET-FQT-RELEASE-BOUNDARY-001`.
- Temporal: `TEMP-TDRP-WINGET-PROVIDER-HEAD-DRIFT-001`, `TEMP-TDRP-WINGET-OVERLAPPING-RELEASES-001`.
- Fault/recovery: `FAULT-TDRP-WINGET-NO-ASSETS-001`, `FAULT-TDRP-WINGET-TOKEN-001`, `FAULT-TDRP-WINGET-SUBMIT-OUTCOME-UNKNOWN-001`, `FAULT-TDRP-WINGET-CLEANUP-001`.
- Security: `SEC-TDRP-WINGET-WINGET-TOKEN-BOUNDARY-001`, `SEC-TDRP-WINGET-MUTABLE-SUPPLY-CHAIN-001`.
- Regression: `REG-TDRP-WINGET-FQT-OWNERS-REUSED-001`, `REG-TDRP-WINGET-NO-CATALOG-RELEASE-ACCEPT-001`.

Closing strict index 98 changes counters to unknown=16,278, unread=16,253, omitted=0, responsibilities=116. Next deterministic strict root is index 99 `.gitignore@320c160d093a0a36ceb3f6437500d01a63d5ed00` (1013 bytes). `baseline_identity_accepted=false` and `source_closure_ready=false`; the strict98 child requires fresh exact-head TDRP/Rust evidence.
