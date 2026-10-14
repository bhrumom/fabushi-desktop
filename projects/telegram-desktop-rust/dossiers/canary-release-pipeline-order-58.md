# Canary release pipeline responsibility — deterministic order 58

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Accepted commit: `63a1831728147e941ad2dc8939879cf5afe419c7`
- Path: `.github/workflows/canary.yml`
- Exact blob: `cff0a8cd8bb0008ddd1dc833c37005c8890e6a8c`
- Deterministic order: **58**

This workflow is not migrated as a Telegram canary runtime. It is decomposed into source-neutral release responsibilities and mapped existing-owner-first.

## Applicable responsibilities

1. Release lane / branch / repository fencing and current-tip binding.
2. Refuse reruns of an old source tip from publishing as a newer release.
3. Monotonic build/update counter independent of branch history.
4. Least-privilege secret and OIDC scope; signing token only where needed.
5. Windows Authenticode plus update and portable package semantics.
6. macOS dual-arch/universal assembly, Developer ID codesign, notarization, stapling and assessment.
7. Linux x64 update and portable package semantics.
8. v2 signed update envelope / trust-chain behavior.
9. Explicit isolation of unsigned bring-up from formal signed publication.
10. Digest-pinned publication infrastructure.
11. Per-platform build/publication failure isolation.
12. Portable first-install artifacts.
13. Symbols/checksums/provenance.
14. Platform-owned update publication metadata and ordering semantics.

Telegram Bot API/channel identifiers, Telegram API credentials, Telegram channel metadata messages and Telegram-specific branch names are source-specific transport details and are **N/A individually**, not grounds to mark the entire workflow N/A.

## Existing-owner audit and predecessor evidence

### macOS

`.github/workflows/release-macos-main.yml` is the canonical existing owner. It already requires signing/notarization credentials, installs a Developer ID identity, packages signed/notarized output, runs notarytool, staples and validates, verifies codesign and Gatekeeper assessment, verifies updater metadata, installs the exact candidate into `/Applications`, runs signed packaged acceptance, generates SHA256SUMS and can publish an exact-source GitHub prerelease.

This is a real owner, so no new macOS release subsystem is introduced.

### Cross-platform updater contract

`.github/workflows/rust-desktop-runtime.yml` owns updater contract qualification on Ubuntu, macOS and Windows. Predecessor run `38273831359` at `92944016319539f8115607f05aeeff0c7fb7de4f` completed successfully, but is historical after descendant ledger/canary changes:
- Ubuntu updater job `114894362259`
- macOS updater job `114894362334`
- Windows updater job `114894362359`
- Source authority job `114894362277`
- Authority artifact `11758466363`, digest `sha256:1cf412bb2399e3c6f6dbf7ad5cfd80c907692244a3070439b0d22837befbbcb6`

These prove the existing updater contract behavior at that predecessor SHA, not current-head acceptance and not signed Windows/Linux release packages. Any descendant must obtain fresh exact-head Actions.

### Windows and Linux formal release owners

.github/workflows/release-windows-main.yml and .github/workflows/release-linux-main.yml now extend the existing source-neutral Fabushi release infrastructure. Windows owns exact-source fencing, Authenticode-signed NSIS plus portable artifacts, generated updater metadata, signed installed-candidate trust checks, real packaged product acceptance, checksums/provenance and shared GitHub prerelease publication. Linux owns exact-source fencing, signed AppImage plus deb artifacts, generated updater metadata, AppImage restart smoke, installed deb candidate execution, real packaged product acceptance, checksums/provenance and the same shared prerelease identity.

The shipping updater has also been extended existing-owner-first: Electron Main injects the canonical electron-updater release port; macOS and Windows packaged builds may use it directly, while Linux is fail-closed unless it is actually running from an AppImage. The legacy custom feed remains an explicit override path rather than the production default.

These owners close the prior implementation-owner absence, but **do not close acceptance**. Current workflows still install the current candidate and exercise restart/package behavior; they do not yet prove a real older trusted baseline discovering, downloading, applying and recovering into the same exact candidate through the shipping updater. Windows/Linux also lack a previously accepted published package baseline. Therefore E2E-TDRP-SIGNED-PACKAGE-UPGRADE-001, SOAK-TDRP-UPDATER-RESTART-001, same-candidate three-platform acceptance and independent release acceptance remain open, and this responsibility must not reduce unknown yet.

## Traceability binding

This dossier is the source-analysis owner for `TDRP-R9-RELEASE-CANARY-SEMANTICS-001`. The ledger's exact traceability set is intentionally repeated here so the authority validator can fail closed when the release responsibility and its quality contracts drift.

- requirement_ids: `TDRP-R9-RELEASE-CANARY-SEMANTICS-001`, `AC-47`, `AC-48`, `AC-49`, `AC-50`
- oracle_ids: `ORA-TDRP-RELEASE-EXACT-SOURCE-001`, `ORA-TDRP-UPDATER-TRUST-001`
- invariant_ids: `INV-TDRP-RELEASE-NO-STALE-TIP-001`, `INV-TDRP-RELEASE-MONOTONIC-001`, `INV-TDRP-RELEASE-SIGNED-PRODUCTION-001`
- contract/integration: `CONTRACT-TDRP-UPDATER-MATRIX-001`, `INT-TDRP-RELEASE-PACKAGING-001`
- e2e/temporal/fault: `E2E-TDRP-SIGNED-PACKAGE-UPGRADE-001`, `TEMP-TDRP-UPDATER-MONOTONIC-001`, `FAULT-TDRP-RELEASE-STALE-RERUN-001`
- soak/security/regression: `SOAK-TDRP-UPDATER-RESTART-001`, `SEC-TDRP-RELEASE-LEAST-PRIVILEGE-001`, `SEC-TDRP-UPDATE-SIGNATURE-001`, `REG-TDRP-FABUSHI-BOT-PACKAGED-001`

The macOS, Windows and Linux release owners plus the three-platform updater contract are implementation bindings, not current-head acceptance evidence. They do not turn `AC-50` into ACCEPT without fresh exact-head signed package evidence and real baseline-to-candidate updater install/upgrade/restart/recovery on the same candidate, followed by independent release review.

## Security and provenance

Fabushi must retain the transferable security properties: exact source binding, stale-tip refusal, monotonic ordering, least privilege, platform signature verification, signed updater trust, digest-pinned infrastructure and fail-closed publication. Telegram transport secrets/runtime are not adopted. Provenance remains tied to the exact upstream blob above and to same-head Fabushi GitHub Actions.

## Status

`applicable-mapped-incomplete`. The source responsibility is fully decomposed and formal three-platform release owners now exist, but it is **not verified** and does not reduce unknown yet because fresh signed-package evidence, real baseline-to-candidate updater upgrade/restart/recovery and independent one-release-candidate acceptance remain open.
