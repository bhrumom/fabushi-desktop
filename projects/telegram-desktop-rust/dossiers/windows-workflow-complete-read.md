# Complete read — fixed-U repository Windows CI build/artifact workflow

- Fixed U: \`f985723b552b30fe1c17d377d57f44e61a15a8c4\`
- Recursive tree: \`91f2b083ab2877c0a17d07868eabec362a5cfca6\` (\`truncated=false\`, 7,194 entries)
- Source: \`.github/workflows/win.yml@219c42ebf679ca5d93add5cdbd51b0bb6e516b57\`
- Tree index: 135 zero-based
- Mode/size: 100644 / 9,167 ASCII bytes/code points
- Line endings: 253 LF, 0 CR, trailing newline
- Responsibility: \`TDRP-R9-REPOSITORY-WINDOWS-WORKFLOW-001\`
- Stage A: authored only; \`validation_status=not-run\`; \`accepted=false\`

## Authority note

The active semantic queue/ledger/closure continues fixed U above. \`upstream.lock.json\` and inventory metadata still carry older rebaseline identity \`351204f3695b4a39bc3f4d2f230c061b9a01fe2a\`; that pre-existing cross-document mismatch is not silently repaired by this one source-entry closure. Platform-core remains unchanged.

## Triggers, permissions and runner matrix

Push excludes \`public-canary\` and \`private-canary\`; push and pull_request share path ignores that exclude generic docs/Markdown/legal, xdg/snap/docker/UWP, Linux/macOS platform code, Telegram packaging metadata and configure.sh while negatively re-including \`docs/building-win*.md\` and this workflow. There is no schedule, manual/repository dispatch, workflow_call, explicit permissions, environment, concurrency, timeout, retry or job condition.

Matrix axes are \`arch=[x64_x86,x64,arm64]\`, \`qt=["",qt6]\`, \`generator=["","Ninja Multi-Config"]\`. Excluding arm64+empty-Qt and x64_x86+qt6 removes four Cartesian entries and leaves eight jobs. ARM64 always uses mutable \`windows-11-arm\`; non-ARM pull requests and nightly use mutable \`depot-windows-latest-16\`; other admitted non-ARM branch pushes use mutable \`windows-latest\`. The runner expression also mentions tag refs, but the current push branch filtering does not admit tag pushes, so this is a latent expression branch rather than a source trigger.

## Source lineage and external Actions

Observed on 2026-11-02:
- \`actions/checkout@v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1\`; \`action.yml@5b0524f730db83f9513c18ab31a6c086c7239076\`; MIT \`LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5\`.
- \`actions/cache@v6 -> 55cc8345863c7cc4c66a329aec7e433d2d1c52a9\`; \`action.yml@2606455d4cedc2a06e05c1bf7feb8cc41aaf28f9\`; MIT \`LICENSE@a426ef259d6c5d705e9c1405075c3b318093c65e\`.
- \`actions/upload-artifact@v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9\`; \`action.yml@7cb4d1e81db55320b41217e1a78a1a46e3d2baef\`; MIT \`LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5\`.
- \`Eden-CI/msvc-dev-cmd@master -> 1bd71f95d6f3d1b2b3395b335cc63bcdc90cf223\`; \`action.yml@6b247098f64d28499a4f468680a823e34c52e43d\`; Node24 \`dist/index.js@e4db4ce930cad3e47801a2f5029d86c72e690f24\`; MIT \`LICENSE@ddd5435732c99642e64d7745d1e78849f0e9d753\`.

The workflow still references mutable major tags and master. Observed resolutions are provenance anchors only and must be rebound for a frozen candidate.

## Toolchain/bootstrap

The workflow creates a user-profile TBuild root and workspace link, checks out recursively into a repository-name subdirectory, and derives SDK from fixed-U \`docs/building-win.md@d333539fc0709be6f98a8ef719f22fac0346b49a\` (current documented SDK 10.0.26100.0). It edits the installed CMake Windows-MSVC module to remove debug-flag initialization and sets global Git user identity for dependency cherry-picks. Those are host-global mutations, not immutable toolchain evidence.

ARM64 downloads the VS 2026 Build Tools bootstrapper from the mutable aka.ms stable endpoint without a workflow-level digest assertion, requests VC tools plus ARM64/ATL components, deletes the bootstrapper after execution, and accepts exit 0 or 3010. The observed msvc-dev-cmd source uses vswhere/standard locations to find vcvarsall, normalizes target architecture and passes SDK plus \`-vcvars_ver=14.44\` when toolset is nonempty. This workflow leaves ARM64 toolset empty and uses 14.44 for non-ARM jobs.

NuGet source setup disables the Visual Studio Offline Packages source and appends an unconditional \`exit 0\` to the nuget.org add command. Failure of that add command therefore cannot by itself fail the step and must not be recorded as successful source configuration without later evidence.

## Cache and dependency preparation

ThirdParty cache key uses runner OS/architecture plus a key derived from \`prepare.py\` and SDK. Libraries and Qt caches use target matrix architecture, with separate Qt flavor, and broad prefix restore fallbacks. Generator, full dependency graph, Visual Studio bootstrap identity and other toolchain inputs are not all part of the outer key. Cache is an optimization, not source/dependency identity.

Dependency preparation is \`Telegram/build/prepare/win.bat@c73fe2a3e5b0d32f8ec34db2c2bdb148ecb50e59\` -> \`prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b\`, invoked \`skip-release silent\` plus Qt selection. The existing fixed-U build-time-acquisition inventory remains the detailed dependency authority. A subsequent find command destructively prunes Libraries while preserving static libraries, executables, headers/includes, selected CMake/pkg-config data, object/cache-key/patch trees and nv-codec headers. Failure can leave a partially pruned cache.

## Configuration and build

The configuration step derives unique artifact labels from target architecture, Qt and generator. x64_x86 maps to x86; arm64 maps to arm; x64 remains x64. A nonempty generator becomes a CMake generator argument and clears the explicit architecture argument. Source references \`matrix.defines\`, but no such matrix axis is declared, so it contributes no configured value.

Ordinary admitted refs use \`TDESKTOP_API_TEST=ON\`; nightly switches to Telegram source-public API application literals. Those literals are fixed-U source behavior, not Fabushi application-secret authority.

Exact build path is \`Telegram/configure.bat@85b9ddebafd5bc95b3590240fb3a11eae590afb4\` -> \`configure.py@592b2f02fd0030dffe6a79d94f1d18c529c53645\` -> CMake, with Debug configuration, warnings-as-errors, empty MSVC debug-information format override and parallel build. \`DESKTOP_APP_DISABLE_AUTOUPDATE=OFF\` and \`DESKTOP_APP_DISABLE_CRASH_REPORTS=OFF\` leave those capabilities enabled. No test command is present; compiling API-test support is not test execution.

## Artifact, package, signing and publication

\`UPLOAD_ARTIFACT\` is source-fixed true and \`ONLY_CACHE\` false, so admitted successful jobs traverse build/artifact lanes. Staging creates \`out\\Debug\\artifact\`, moves Telegram.exe first and Updater.exe second, and can be partial if the second move fails. Upload-artifact uses the matrix-derived name, staged directory and one-day retention. The source records no artifact digest.

This workflow does **not** build an installer/package, Authenticode-sign binaries, attest/SBOM them, publish a release/update, install a package or verify updater behavior. Its Debug artifact cannot be release authority.

Fixed-U \`canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947\` independently owns signed canary Windows packaging/publication; it is not a caller of win.yml. \`winget.yml@0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390\` independently responds to GitHub release events and is not a consumer of this Debug artifact lane. Fixed-U \`README.md@15a6194f73ae6084a2ab25d14005dc10262cdfb3\` does consume the workflow name via its Windows build-status badge.

## Fabushi existing-owner resolution

Current checkpoint canonical Windows package/update surfaces are:
- \`desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84\`: cross-platform Electron build/package scripts, Windows NSIS target and Fabushi setup artifact naming, plus GitHub publish configuration;
- shipping \`source/electron-main/entry.ts@3c877893f5f6478251587a1130e08328c9cf15f7\`;
- \`source/electron-main/update/update-wiring.ts@9165ea7349bbaf6c55424ba8c61b99a37275e037\`;
- \`source/electron-main/update/sand-update-service.ts@7faf203c88d33f116abf7cef4ce8758007a730d2\`;
- \`source/electron-main/update/win32-installer.ts@b337cd7ab8e9ca7d112dd9c18a309912b2de30d7\`;
- root repository build/release/process/security governance and shared Test Strategy/Test Evidence Contract.

No Telegram workflow, second Windows build/package/update truth, runtime/provider/sidecar or platform-core delta is created. The target architecture does not need Telegram's Qt/Visual-Studio matrix literally; it needs equivalent Fabushi Windows package/release qualification under the existing Electron owner.

Current exact-head release gaps are retained rather than hidden: there is no dedicated Windows release workflow in the checkpoint, the electron-builder Windows config does not force signing, and the Windows installer verifier still allowlists \`Anysphere, Inc.\` / \`Anysphere\`. These facts mean G-RELEASE/signing acceptance is still open. They do not justify copying Telegram's unsigned Debug CI workflow as a second release authority.

## Security, failure recovery and quality

No explicit permissions plus checkout's default token/credential behavior and recursive submodules are a credential/trust boundary. Mutable runners/actions, ARM64 bootstrap URL, host-global CMake/Git mutation, nuget.org acquisition, broad cache fallback and source-public Telegram API literals are provenance/security boundaries. Public source literals are not treated as Fabushi secrets, and no user/repository secret value is retained in evidence.

There is no workflow concurrency group, timeout, retry/backoff, transaction, rollback or compensation. SDK parse, host patch, ARM64 installer, msvc setup, dependency preparation, destructive prune, configure/build and artifact staging can fail at distinct partial-state boundaries. NuGet add is explicitly masked. Upload-artifact defaults cannot substitute for the earlier ordered move settlement. A later successful run cannot prove an earlier environment or artifact identity.

Direct callers are GitHub push/pull_request event admission. Direct dependencies/consumers are checkout/submodules, SDK documentation, Visual Studio/CMake/NuGet, prepare/configure/build scripts, GitHub cache/artifact stores, maintainers/manual artifact users and the README status badge. No workflow_call is declared, so win.yml is not a reusable workflow.

Authored only, not run: 33 contract, 1 integration, 18 fault/recovery and 15 security/provenance cases. Stage-B oracles are \`ORA-TDRP-WINDOWS-WORKFLOW-EXACT-SOURCE-001\`, \`ORA-TDRP-WINDOWS-WORKFLOW-TRIGGER-RUNNER-PERMISSIONS-001\`, \`ORA-TDRP-WINDOWS-WORKFLOW-MATRIX-TOOLCHAIN-001\`, \`ORA-TDRP-WINDOWS-WORKFLOW-CACHE-PREPARE-001\`, \`ORA-TDRP-WINDOWS-WORKFLOW-BUILD-OUTPUT-IDENTITY-001\`, \`ORA-TDRP-WINDOWS-WORKFLOW-ARTIFACT-PUBLICATION-001\`, and \`ORA-TDRP-WINDOWS-WORKFLOW-FAILURE-RECOVERY-001\`.

No workflow run/query, external Action/cache/submodule/bootstrap execution, host mutation, build, test, artifact upload/download, package, signing, release, deployment, installation, runtime acceptance or platform-core mutation is performed in Stage A.
