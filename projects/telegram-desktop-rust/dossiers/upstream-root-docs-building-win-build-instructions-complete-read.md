# Upstream Windows build instructions complete read — strict index 120

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/building-win.md@d333539fc0709be6f98a8ef719f22fac0346b49a` (3,342 bytes), fully read.
- Parent exact HEAD: `56ffe86edf90a9e246b0be2881a7597dd004de48`, tree `362b621ef2ef2a15a35d4206b384827fd8b13813`.
- Parent TDRP run `39951083002`: 61/61 success; authority `12500995492` sha256:8bedd5210a0b86192aadf26a97184f55d81ac941348d5f17e45976e8a39b0c30; closure `12501160409` sha256:a0ec1abdad6cb8bac5d0a3d957c26a03845d4f320d4bf42e2bcde15a426ee06d; reachability `12500971253` sha256:bf5f93e89fd47608776851a54fd24f4ccaab1adf33c60e7bdbbd224a057aab8a.
- Parent Rust run `39951082971`: 6/6 success; artifact `12500940550` sha256:bd017823ef92c06374a24a7a24ff6da34971e1b6f48fab11696d96d688dadee6.
- Publication preflight: PR/ref remained `56ffe86edf90a9e246b0be2881a7597dd004de48` / tree `362b621ef2ef2a15a35d4206b384827fd8b13813`; live main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`; live upstream `dev` remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Complete source responsibility

The document defines Telegram Desktop's current local Windows developer build. It selects Visual Studio 2026 with Windows SDK 10.0.26100.0, deliberately overrides the default v145 toolset with `-vcvars_ver=14.44` for the documented x86/x64 Windows-7-compatible path, installs Python 3.10 and Git, recursively clones source, runs `Telegram/build/prepare/win.bat`, configures with `TDESKTOP_API_ID` / `TDESKTOP_API_HASH`, opens the generated `Telegram.slnx`, and builds Debug/Release `Telegram.exe`.

The direct build owners were read at the same upstream commit. `win.bat@c73fe2a3e5b0d32f8ec34db2c2bdb148ecb50e59` delegates to `prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`; that Python owner requires a Native Tools `Platform`, maps x86/x64/arm64 to win/win64/winarm, and prepares platform-scoped dependencies through Ninja Multi-Config. The current Rust dependency stage uses `i686-win7-windows-msvc` / `x86_64-win7-windows-msvc` for x86/x64 while arm64 uses `aarch64-pc-windows-msvc`, so the documentation's downlevel toolset requirement is real upstream compatibility provenance, not a generic Fabushi toolchain rule.

`configure.bat@85b9ddebafd5bc95b3590240fb3a11eae590afb4` delegates to `configure.py@592b2f02fd0030dffe6a79d94f1d18c529c53645`. For an official upstream target, configure.py requires `DesktopPrivate/custom_api_id.h`, extracts ApiId/ApiHash and converts them to the same CMake definitions before invoking the canonical configure path. `docs/api_credentials.md@0d2819bd3cd6b6407ced257edbbd6623b438f262` remains strict111 authority: Telegram application credentials and its test-only pair are upstream-only provenance and never become Fabushi production credentials.

History was also checked instead of treating the current prose as timeless truth. `1b104bcf29db6e31ad44c16e28d33a135e7a9554` changed the documented SDK from 10.0.22000.0 to 10.0.26100.0; `ff3b0fe2d92f83a069096ac78eaa7d1551ed28b2` moved the guide to Visual Studio 2026, merged x86/x64 guidance and added vcvars 14.44; `719cae0406daac3d68e7e316b4c78c1ef04a5585` moved the generated solution reference from `.sln` to `.slnx`. Older history also records the move to Python 3.10 and script-owned dependency preparation.

## Existing-owner-first Windows mapping

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` is the canonical Fabushi package owner. Its `package` script builds Host/Electron/renderer, checks the runtime and invokes electron-builder; `build.win` declares the single NSIS target and artifact name.
- `desktop/scripts/stage-host.mjs@83c1f868d3df62683a216e1523e2a9ef5cc01ae2` already owns Windows native executable staging: `mahayana-app-host.exe`, coordinator, box daemon and network worker.
- `desktop/scripts/check-electron-runtime-bundle.mjs@279082ad89215229e3dd2d219dbf5602c73ee358` is the existing co-staged runtime-graph validator.
- `.github/workflows/rust-desktop-runtime.yml@905fc0e17e5b4e54cb18708c55d1eb8725d18f83` binds exact HEAD and runs a `windows-2025` pressure-profiler job, but it does not run electron-builder or produce/install NSIS.
- `desktop/e2e/openbot-packaged-acceptance.spec.ts@4075f76ec7e4b74e2e219cb3f49e984bd96132a6` and `miniapp-packaged-acceptance.spec.ts@2663a61e3c1f5284d2a30403b6c633b3df6dc807` are existing executable-driven packaged validators and should be reused for a future Windows candidate rather than duplicated.
- The current shipping wiring for those validators is macOS-only: `release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643` builds with `electron-builder --mac --arm64`, installs/codesigns the .app and supplies `FABUSHI_ELECTRON_EXECUTABLE`; `miniapp-packaged-iteration.yml@1cfcf7aa9250c4f2b3b9405c6cc42ff4d23b8b37` runs on `macos-15`.
- Current strict111 credential owners were re-read and remain unique: account service `6913ae84043204ca74d89f671a128f2984f7f9da`, secret store `c71c39527858771835f59745905ce6f72b078ace`, issuer policy `291fd47ae89aca36a24ca6f1866f2099af2b4a5f`, auth contract `c23df86b7b11d38b3eea4f317da6f039586cb7f2`. No Windows-specific credential root is introduced.

The complete `desktop` subtree was recursively enumerated at the parent exact head (52 entries, non-truncated); `package.json` is the sole package/Windows configuration owner. The root `scripts` subtree contains no Windows package/release owner.

## Windows shipping / signing fail-closed gap

All 25 exact-head workflow files were enumerated and content-scanned for Windows/win32/`--win`/electron-builder/NSIS/CSC/certificate/signing/signtool/package markers. Windows runners exist for runtime or functional test matrices, but no workflow invokes electron-builder for Windows, emits an NSIS installer, imports a Windows signing certificate, invokes signtool, installs a Windows candidate, or runs packaged acceptance against a Windows executable.

The `build.win` block also does not declare `forceCodeSigning` or a Windows certificate owner. No workflow supplies `WIN_CSC_LINK`, `CSC_LINK`, or an equivalent Windows signing boundary. Therefore Windows build metadata and runtime tests are mapped, while signed package/install/release evidence remains open. The upstream Windows-7-compatible toolchain also cannot be upgraded into a claim that Fabushi currently supports Windows 7.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-BUILDING-WIN-BUILD-INSTRUCTIONS-PROVENANCE-001`.
- Oracles: `ORA-TDRP-BUILDING-WIN-EXACT-SOURCE-001`, `ORA-TDRP-BUILDING-WIN-TOOLCHAIN-HISTORY-001`, `ORA-TDRP-BUILDING-WIN-EXISTING-BUILD-PACKAGE-OWNER-001`, `ORA-TDRP-BUILDING-WIN-CREDENTIAL-SEPARATION-001`, `ORA-TDRP-BUILDING-WIN-SHIPPING-WORKFLOW-FAIL-CLOSED-001`, `ORA-TDRP-BUILDING-WIN-PACKAGED-VALIDATOR-REUSE-001`.
- Invariants: `INV-TDRP-BUILDING-WIN-NO-TELEGRAM-CREDENTIAL-001`, `INV-TDRP-BUILDING-WIN-NO-SECOND-BUILD-RUNTIME-001`, `INV-TDRP-BUILDING-WIN-WINDOWS-TEST-NOT-PACKAGE-EVIDENCE-001`, `INV-TDRP-BUILDING-WIN-NO-UNPROVEN-WINDOWS7-ACCEPTANCE-001`, `INV-TDRP-BUILDING-WIN-EXACT-HEAD-ACTIONS-ONLY-001`.
- Contracts: `CONTRACT-TDRP-BUILDING-WIN-EXACT-SOURCE-001`, `CONTRACT-TDRP-BUILDING-WIN-CANONICAL-PACKAGE-OWNER-001`, `CONTRACT-TDRP-BUILDING-WIN-CREDENTIAL-SEPARATION-001`, `CONTRACT-TDRP-BUILDING-WIN-SHIPPING-EVIDENCE-001`.
- Temporal/fault: `TEMP-TDRP-BUILDING-WIN-UPSTREAM-DRIFT-001`, `TEMP-TDRP-BUILDING-WIN-TOOLCHAIN-DRIFT-001`, `TEMP-TDRP-BUILDING-WIN-PACKAGE-WORKFLOW-DRIFT-001`, `FAULT-TDRP-BUILDING-WIN-MISSING-SIGNING-001`, `FAULT-TDRP-BUILDING-WIN-INSTALL-ACCEPTANCE-001`.
- Security: `SEC-TDRP-BUILDING-WIN-TELEGRAM-CREDENTIAL-SEPARATION-001`, `SEC-TDRP-BUILDING-WIN-FABUSHI-AUTH-OWNER-001`, `SEC-TDRP-BUILDING-WIN-SIGNING-CREDENTIAL-BOUNDARY-001`, `SEC-TDRP-BUILDING-WIN-LOCAL-BUILD-NO-AUTHORITY-001`.
- Regression: `REG-TDRP-BUILDING-WIN-NO-TELEGRAM-BUILD-RUNTIME-001`, `REG-TDRP-BUILDING-WIN-NO-SECOND-WINDOWS-PACKAGE-OWNER-001`, `REG-TDRP-BUILDING-WIN-REUSE-PACKAGED-VALIDATORS-001`, `REG-TDRP-BUILDING-WIN-FAIL-CLOSED-UNTIL-SHIPPING-EVIDENCE-001`.

## Fail-closed boundaries

- No local/container/Mac/Windows build or test was run; all executable evidence is exact-head GitHub Actions.
- Telegram Visual Studio/SDK/vcvars/Python/Git/CMake details are source/build provenance, not a second Fabushi toolchain owner.
- Telegram `api_id` / `api_hash`, including test-only values, are never Fabushi credentials.
- `windows-2025` runtime/functional test success is not Windows installer, code-signing, install or release evidence.
- Existing generic packaged acceptance code is not Windows evidence until wired to an immutable Windows candidate.
- Missing Windows signing/package/install evidence is blocking, never pass.
- Any source/history/script/package/auth/workflow/PR/main/upstream drift requires fresh reconciliation.
- Independent release acceptance remains open.

Closing strict120 yields unknown=16,256, unread=16,231, omitted=0, responsibilities=138. Next strict root index 121 is `docs/web-proxy-plan.md@9c805e32d33e8e1ac6b0417b26242b4a09198bd9` (34,990 bytes). Fresh child exact-head TDRP/Rust Actions are required.
