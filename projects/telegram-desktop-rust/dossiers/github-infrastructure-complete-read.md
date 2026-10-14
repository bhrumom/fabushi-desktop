# GitHub/community/release infrastructure — complete source read

Status: entries 45–53 disposition-closed; entry 54 mapped/open  
Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db`

## Entries 45–53

The following were completely read and recorded with exact blobs in `projects/telegram-desktop-rust/inventory/source-dispositions.json`:

- 45 `.github/CONTRIBUTING.md` @ `4265e974d0b87622af223d5b8096b67edcbd4aab`
- 46 `.github/ISSUE_TEMPLATE/BUG_REPORT.yml` @ `847b56e543fc28586ad281640218dfe8954d8bee`
- 47 `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml` @ `e2a8becdb9afcbd028b62378321458a2028fe348`
- 48 `.github/ISSUE_TEMPLATE/config.yml` @ `1b864704be6a3e3aafdb1e02519bec84886942ce`
- 49 `.github/dependabot.yml` @ `5ace4600a1f26e6892982f3e2f069ebfab108d87`
- 50 `.github/scripts/generate_changelog.py` @ `6cdd12652d31b2a47ed9a8ef906308af59cca3d0`
- 51 `.github/telegram-bot-api/Dockerfile` @ `a361602441040028a3332c3732c4902d9c880122`
- 52 `.github/telegram-bot-api/entrypoint.sh` @ `d40a7372efd8e5862e00f9b214b36a840688797e`
- 53 `.github/workflows/canary-bot-api.yml` @ `7285f29ed2a09695a02b8f7821c10df5aa50bc44`

Community templates and dependency tooling are repository/process concerns. The Telegram Bot API container/image workflow is a Telegram-channel-specific release transport and is explicitly replaced, not ported, by Fabushi's canonical release transport.

## Entry 54 — canary release/update pipeline

Path: `.github/workflows/canary.yml`  
Blob: `7f30e734048ec69d0b269acc1c5f229d3be4767c`  
Read status: complete in contiguous ranges through end-of-file.

This entry is **applicable and still open**. It defines real release/update product responsibilities rather than mere developer convenience:

- exact branch/repository/source-tip binding and monotonic update ordering;
- Windows x64 Release+LTO, Authenticode signing and signature verification;
- macOS thin x86_64/arm64 builds, universal assembly, Developer ID signing, notarization, stapling and Gatekeeper assessment;
- Linux x64 release build and symbol extraction;
- updater trust-chain tests, ES256 v2 update-envelope signing and per-platform update artifacts;
- portable first-install archives and artifact provenance;
- independent platform publication with a publish-side signature witness.

Fabushi already has a canonical macOS owner at `.github/workflows/release-macos-main.yml` that binds exact source, signs with Developer ID, notarizes/staples, checks updater metadata, installs the signed candidate, runs real packaged production-account acceptance, emits checksums/artifacts and can publish a GitHub prerelease. This is valid partial parity and must be reused.

Open platform delta: current head has no equivalent complete Windows/Linux signed packaging/publication path and no same-head evidence for a unified monotonic update-channel/order contract across all three desktop platforms. Entry 54 therefore stays `applicable-mapped-incomplete`; it does **not** lower unknown.

## Accounting

- full-read: `114`
- unread: `15,698`
- unknown: `15,759`
- omitted: `0`
- machine-readable unknown-closed through entry: `53`
- fully read/decomposed through entry: `54`

The next deterministic entry is `.github/workflows/cant-reproduce.yml` at blob `8e20e07349b5b43277f5371a4e931066bece4186`.

## Entries 55–72 — remaining GitHub workflows

All workflow sources through `.github/workflows/winget.yml` are now fully read and exact-blob dispositioned in `inventory/source-dispositions.json`.

Closed repository/community/infrastructure-only entries: 55–59, 61, 64–65, 67–70. macOS build entries 62–63 are closed by an explicitly stronger canonical replacement: Fabushi `.github/workflows/release-macos-main.yml` binds exact source, packages, Developer-ID signs, notarizes/staples, verifies updater metadata, installs the signed candidate and runs real packaged acceptance.

Still applicable/open:
- entry 54 `canary.yml`: cross-platform signed release/update channel and publication;
- entry 60 `linux.yml`: full Linux desktop + Updater build qualification;
- entry 66 `snap.yml`: Linux distribution package artifact;
- entry 71 `win.yml`: Windows x86/x64/ARM64 and build-generator qualification;
- entry 72 `winget.yml`: stable/beta Windows release distribution.

Current Fabushi exact source has three-platform updater contract tests, but repository search found no Linux package workflow, Windows full-package workflow, Snap/AppImage or WinGet publisher. These entries therefore remain open rather than being converted into false development-only dispositions.

Accounting after entry 72:
- full-read: `132`
- unread: `15,680`
- unknown: `15,746`
- omitted: `0`
- read/decomposed through: `72`
- unknown closed among first 72: `66`

The next deterministic entry is `.gitignore` at blob `320c160d093a0a36ceb3f6437500d01a63d5ed00`.

