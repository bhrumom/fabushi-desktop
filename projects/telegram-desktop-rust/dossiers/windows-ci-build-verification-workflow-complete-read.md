# Windows CI build verification workflow complete read — strict index 97

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/win.yml@219c42ebf679ca5d93add5cdbd51b0bb6e516b57`, 254 line slots / 9167 bytes; complete read and SHA revalidated immediately before publication.
- Parent exact-head 14c2deef2bcd7b0f7d9b589a29bc17d570b59b00/tree 149892529be4be2c439455436fe71c03096f72a4 completed TDRP run 39896888856 at 61/61 success (authority 12476159790 sha256:a30706f41f6ec3f85d40590533bcfd326cc455d9adbfd82c1085c0e0c5f94401; closure 12477142047 sha256:35321430d8d859ee277246abf655d8ba059cadf8903f1b38389c9b93b14f140b; reachability 12476768596 sha256:cf7db38792778f6333f5ff52ee8a118f712dcf228488310f56f8e9d04f294bbf) and Rust desktop runtime run 39896888741 at 6/6 success (artifact 12477426522 sha256:02f87806bd32ec0d7fd1e32ee478ead2de0c96655559c4e40261bd9a02ab7233).
- Stable existing evidence owner: `projects/fabushi-communication-platform/quality/evidence-contract.md`, `Evidence ID: FQT-EVID-001`; branch blob `2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`, live-main blob `b1c3b67fef2ddb0d0a017700600c2395aeeac7a7`, both retaining the same passing-evidence identity requirement.
- Stable executable artifact/release boundary: `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72`, `Policy ID: FQT-REL-001`.
- FBCP/TDRP executable verification remains GitHub-Actions-only. The branch and live-main `AGENTS.md` blobs differ, so this row does not depend on the old branch-only wording as its unique validator key.

## Complete-read behavior
The workflow is Windows source-build verification provenance. It handles Windows-relevant push and pull-request events, excludes public/private canary pushes, and filters broad documentation, Linux, macOS, Snap and unrelated build paths while re-including itself.

The matrix crosses `arch=[x64_x86,x64,arm64]`, default/Qt6 and default/`Ninja Multi-Config`, excluding arm64 with the default Qt variant and x64_x86 with Qt6. That leaves eight admitted configurations. arm64 uses `windows-11-arm`; the other configurations use `depot-windows-latest-16` for pull requests, tags and nightly and otherwise use `windows-latest`.

Each job creates the disposable TBuild layout, recursively checks out source, derives the Windows SDK from `docs/building-win.md`, and keys caches from the prepare script plus SDK. arm64 downloads and installs Visual Studio 2026 Build Tools from the mutable `aka.ms` alias. The workflow enters the native toolchain through `Eden-CI/msvc-dev-cmd@master`, rewrites NuGet source configuration, restores ThirdParty/Libraries/Qt caches with prefix fallback keys, and calls `Telegram/build/prepare/win.bat skip-release silent`.

The configuration step projects architecture, Qt and generator flags. Ordinary builds use `TDESKTOP_API_TEST=ON`; nightly uses the upstream Telegram open API ID/hash from the workflow. Those are Telegram source-specific test credentials, not Fabushi credentials or secrets. The workflow deletes nonessential cached library files to reduce disposable runner pressure, configures a Debug build with warnings-as-errors while keeping autoupdate/crash-report code paths enabled, builds with CMake, then requires and stages `Telegram.exe` plus `Updater.exe`. The staged directory is uploaded as a one-day artifact.

## Direct build inputs and provider provenance
- `docs/building-win.md@d333539fc0709be6f98a8ef719f22fac0346b49a`: Visual Studio 2026, Windows SDK 10.0.26100.0 and v14.44 compatibility toolset guidance.
- `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`.
- `Telegram/build/prepare/win.bat@c73fe2a3e5b0d32f8ec34db2c2bdb148ecb50e59`.
- `Telegram/configure.bat@85b9ddebafd5bc95b3590240fb3a11eae590afb4`.
- `Telegram/configure.py@592b2f02fd0030dffe6a79d94f1d18c529c53645`.
These direct inputs remain independent inventory responsibilities. This workflow row does not silently mark them read/dispositioned.

- `actions/checkout@v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1`; `action.yml@5b0524f730db83f9513c18ab31a6c086c7239076`; MIT `LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5`.
- `actions/cache@v6 -> 55cc8345863c7cc4c66a329aec7e433d2d1c52a9`; `action.yml@2606455d4cedc2a06e05c1bf7feb8cc41aaf28f9`; MIT `LICENSE@a426ef259d6c5d705e9c1405075c3b318093c65e`.
- `actions/upload-artifact@v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9`; `action.yml@7cb4d1e81db55320b41217e1a78a1a46e3d2baef`; MIT `LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5`.
- `Eden-CI/msvc-dev-cmd@master` resolved at publication audit to `1bd71f95d6f3d1b2b3395b335cc63bcdc90cf223`, tree `e7c89b22b26307c38516ac07942068d0be0b7e88`; `action.yml@6b247098f64d28499a4f468680a823e34c52e43d`; MIT `LICENSE@ddd5435732c99642e64d7745d1e78849f0e9d753`; `package.json@ba199623309e19aa3884462789a5374b7d024ef7`. The workflow still references mutable `master`.
- `https://aka.ms/vs/18/stable/vs_buildtools.exe`, NuGet packages/registry, runner images, mutable action tags/master, cache contents and prepare-time acquisitions remain mutable supply-chain inputs.

## Existing-owner-first disposition
This workflow is not a Windows product runtime, package owner, signing owner or release authority. Fabushi already has FQT-EVID-001 for exact executable evidence and FQT-REL-001 for executable artifact provenance, plus the project-wide GitHub-Actions-only verification boundary and existing packaging/signing/device acceptance owners.

The selected mapping therefore reuses FQT-EVID-001 and FQT-REL-001. A green upstream matrix, cache hit, `Telegram.exe` existence, `Updater.exe` existence or one-day artifact upload proves only the bounded upstream CI observation. None can establish Fabushi production composition, signed/package identity, install/startup behavior, physical-device acceptance or independent release ACCEPT. `novel_capability=false`; no second Windows build system, Telegram runtime, cache truth, package ledger, release root or acceptance owner is introduced.

## Fail-closed, concurrency and security conclusions
- Mutable action tags/master, `aka.ms` installer bytes, NuGet packages, runner/toolchain images and prepare-time acquisitions are source/build provenance and must be re-resolved for evidence that depends on them.
- Prefix cache restore can return an older compatible cache; cache state is performance state, never correctness or source identity.
- The workflow has no concurrency group. Overlap, cancellation, stale cache writes or partial provider state cannot advance canonical acceptance.
- Nightly Telegram API values remain upstream source-specific test configuration and are not imported into Fabushi credential ownership.
- Disk cleanup is bounded to the disposable CI library tree; its success does not prove production storage/recovery behavior.
- One-day Telegram executable artifacts have no Fabushi signing/package/device/release authority. FQT-REL-001 requires the applicable executable provenance edge and raw identity before staging/promotion/release.
- Cancelled, failed, stale-head, missing-output, expired-artifact or unresolved-provider evidence is non-passing.

## Exact traceability IDs
- Requirement: `TDRP-R9-WINDOWS-CI-BUILD-VERIFICATION-PROVENANCE-001`.
- Oracles: `ORA-TDRP-WINDOWS-CI-EXACT-SOURCE-001`, `ORA-TDRP-WINDOWS-CI-MATRIX-001`, `ORA-TDRP-WINDOWS-CI-NO-RELEASE-AUTHORITY-001`.
- Invariants: `INV-TDRP-WINDOWS-CI-CACHE-NO-AUTHORITY-001`, `INV-TDRP-WINDOWS-CI-MUTABLE-TOOLCHAIN-FAIL-CLOSED-001`, `INV-TDRP-WINDOWS-CI-EXACT-HEAD-EVIDENCE-001`.
- Contracts: `CONTRACT-TDRP-WINDOWS-CI-BUILD-PROVENANCE-001`, `CONTRACT-TDRP-WINDOWS-CI-FABUSHI-RELEASE-BOUNDARY-001`.
- Temporal: `TEMP-TDRP-WINDOWS-CI-ACTION-TAG-DRIFT-001`, `TEMP-TDRP-WINDOWS-CI-EDEN-MASTER-DRIFT-001`, `TEMP-TDRP-WINDOWS-CI-RUNNER-TOOLCHAIN-DRIFT-001`, `TEMP-TDRP-WINDOWS-CI-CACHE-DRIFT-001`.
- Fault/recovery: `FAULT-TDRP-WINDOWS-CI-VS-BUILD-TOOLS-DOWNLOAD-001`, `FAULT-TDRP-WINDOWS-CI-NUGET-001`, `FAULT-TDRP-WINDOWS-CI-CACHE-RESTORE-001`, `FAULT-TDRP-WINDOWS-CI-DISK-CLEANUP-001`, `FAULT-TDRP-WINDOWS-CI-ARTIFACT-001`.
- Security: `SEC-TDRP-WINDOWS-CI-SUPPLY-CHAIN-001`, `SEC-TDRP-WINDOWS-CI-UPSTREAM-API-CREDENTIAL-BOUNDARY-001`, `SEC-TDRP-WINDOWS-CI-RUNNER-BOUNDARY-001`.
- Regression: `REG-TDRP-WINDOWS-CI-FQT-EVIDENCE-OWNER-REUSED-001`, `REG-TDRP-WINDOWS-CI-NO-RELEASE-ACCEPT-001`.

Closing strict index 97 changes counters to unknown=16,277, unread=16,252, omitted=0, responsibilities=115. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root is index 98 `.github/workflows/winget.yml@0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390`.
