# Round 210 evidence — fixed-U repository Windows CI build/artifact workflow

Date: 2026-11-02
Task round: 179
Desktop input checkpoint: \`178d251d4025b1054591416623b7b5165c37d2de\`
Desktop main reread: \`f14198bd82a355a0a66f95d7b62cdf10a370c173\`
Platform-core: \`e92b51e23b39dae5719f033fa1f7a20d6579dc8c\`
Fixed U: \`f985723b552b30fe1c17d377d57f44e61a15a8c4\`
Fixed-U tree: \`91f2b083ab2877c0a17d07868eabec362a5cfca6\`, non-truncated, 7,194 entries
Source: \`.github/workflows/win.yml@219c42ebf679ca5d93add5cdbd51b0bb6e516b57\`
Tree index: 135 zero-based
Responsibility: \`TDRP-R9-REPOSITORY-WINDOWS-WORKFLOW-001\`

Exact source is 9,167 ASCII bytes/code points with 253 LF, zero CR and trailing newline. Push excludes public/private canary branches; push and pull_request preserve Windows-oriented path admission. No reusable/manual/scheduled trigger, explicit permissions, environment, concurrency, timeout, retry or job condition is declared.

Matrix exclusions leave eight Windows build jobs across x86/x64/ARM64, Qt and generator variants. ARM64 uses mutable windows-11-arm, non-ARM PR/nightly uses mutable depot-windows-latest-16, and ordinary non-ARM branch push uses mutable windows-latest. Observed Action lineage: checkout v7 \`3d3c42e5aac5ba805825da76410c181273ba90b1\`, cache v6 \`55cc8345863c7cc4c66a329aec7e433d2d1c52a9\`, upload-artifact v7 \`cf430e030ddbb5b0abf93d22962f4752f3646cd9\`, msvc-dev-cmd master \`1bd71f95d6f3d1b2b3395b335cc63bcdc90cf223\`; exact metadata/runtime/license identities are bound in the dossier.

Toolchain review records fixed-U Windows docs (VS 2026, SDK 10.0.26100.0, toolset 14.44), host CMake/Git mutation, mutable ARM64 Build Tools bootstrap, msvc vcvars, masked NuGet source-add failure, broad outer caches, prepare.py cache/dependency graph and destructive Library prune. Build review records configure.bat/configure.py/CMake Debug, warnings-as-errors, source-public nightly API literals, no test execution, ordered Telegram.exe/Updater.exe staging and one-day artifact upload with no source-recorded digest.

The workflow contains no installer/package build, Authenticode signing, release/update publication, SBOM/attestation, install or updater acceptance. Canary and WinGet are independent sibling graphs. Existing-owner-first keeps the target responsibility under Fabushi repository build/toolchain + canonical Electron package/update/release-process + security/provenance + shared quality/evidence governance. Current Fabushi Windows release/signing remains explicitly open: no dedicated Windows release workflow, Windows package config does not force signing, and the installer verifier still allowlists Anysphere signer names.

Authored only and not run: 33 contract, 1 integration, 18 fault/recovery and 15 security/provenance cases. Ledger closure target: source entries 16,202; unknown 16,099 -> 16,098; unread 16,072 -> 16,071; omitted 0; responsibilities 179 -> 180; unmapped 0. Next entry: \`.github/workflows/winget.yml@0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390\`, index 136. Existing upstream.lock/inventory identity remains older \`351204f...\`. No CI/Actions/check/status query or executable validation was performed.
