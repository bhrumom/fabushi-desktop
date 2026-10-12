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

## Existing-owner audit on Fabushi exact head 9294401…

### macOS

`.github/workflows/release-macos-main.yml` is the canonical existing owner. It already requires signing/notarization credentials, installs a Developer ID identity, packages signed/notarized output, runs notarytool, staples and validates, verifies codesign and Gatekeeper assessment, verifies updater metadata, installs the exact candidate into `/Applications`, runs signed packaged acceptance, generates SHA256SUMS and can publish an exact-source GitHub prerelease.

This is a real owner, so no new macOS release subsystem is introduced.

### Cross-platform updater contract

`.github/workflows/rust-desktop-runtime.yml` owns updater contract qualification on Ubuntu, macOS and Windows. Current exact-head run `38273831359` completed successfully:
- Ubuntu updater job `114894362259`
- macOS updater job `114894362334`
- Windows updater job `114894362359`
- Source authority job `114894362277`
- Authority artifact `11758466363`, digest `sha256:1cf412bb2399e3c6f6dbf7ad5cfd80c907692244a3070439b0d22837befbbcb6`

These prove existing updater contract behavior, not signed Windows/Linux release packages.

### Open owner gap

No complete current-head source-neutral Windows/Linux signed package + real install/upgrade/restart/publication/provenance owner/evidence equivalent to the macOS release workflow was found in the workflow inventory. Those responsibilities remain mapped/open. A future implementation must extend canonical Fabushi release infrastructure rather than create a Telegram canary runtime.

## Security and provenance

Fabushi must retain the transferable security properties: exact source binding, stale-tip refusal, monotonic ordering, least privilege, platform signature verification, signed updater trust, digest-pinned infrastructure and fail-closed publication. Telegram transport secrets/runtime are not adopted. Provenance remains tied to the exact upstream blob above and to same-head Fabushi GitHub Actions.

## Status

`applicable-mapped-incomplete`. The source responsibility is fully decomposed, but it is **not verified** and does not reduce unknown yet because signed Windows/Linux package acceptance and one-release-candidate install/upgrade/restart/provenance gates remain open.
