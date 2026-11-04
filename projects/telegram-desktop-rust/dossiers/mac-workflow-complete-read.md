# Complete read — fixed-U repository macOS CI build/artifact workflow

- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/mac.yml@184a94b091d2d7bab19ae67d7f3a5881b18213ba`
- Tree index: 126 zero-based
- Mode/size: 100644 / 4,273 ASCII bytes/code points
- Line endings: 138 LF, 0 CR, trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-MAC-WORKFLOW-001`
- Stage A: authored only; `validation_status=not-run`; `accepted=false`

## Authority note

The active semantic queue/ledger/closure continues fixed U above. `upstream.lock.json` and inventory metadata still carry older rebaseline identity `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`; that pre-existing whole-project mismatch remains explicit and is not silently repaired by this one source-entry closure.

## Trigger, runner and permissions

The source was read through EOF. Push excludes `public-canary` and `private-canary` and ignores docs/Markdown/changelog/legal, generic `.github/**` except this workflow, xdg/snap/docker, UWP/Windows resources, Windows/Linux platform source and `Telegram/configure.bat`. Pull requests use the same path-ignore set without the canary branch exclusion. There is no schedule/manual/repository-dispatch/workflow_call, explicit permissions, environment, concurrency, timeout or workflow-level retry.

The only matrix value is empty `defines`. Pull requests, tag-like refs and nightly select `depot-macos-latest`; other admitted refs select `macos-latest`. Both labels are mutable. Environment fixes artifact upload on, cache-only mode off and PREPARE_PATH to `Telegram/build/prepare/prepare.py`.

## External Action lineage

Observed on 2026-11-01:
- `actions/checkout@v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1`; `action.yml@5b0524f730db83f9513c18ab31a6c086c7239076`; package 7.0.1 `@9b02e96595db9e2d3a735a8adbda17a61e56fd5f`; MIT `LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5`.
- `actions/cache@v6 -> 55cc8345863c7cc4c66a329aec7e433d2d1c52a9`; `action.yml@2606455d4cedc2a06e05c1bf7feb8cc41aaf28f9`; package 6.1.0 `@b409614a4fd77b5eb49ee1d36bbf5936dab2792f`; MIT `LICENSE@a426ef259d6c5d705e9c1405075c3b318093c65e`.
- `actions/upload-artifact@v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9`; `action.yml@7cb4d1e81db55320b41217e1a78a1a46e3d2baef`; package 7.0.2 `@2c96bdb1cbafd4beb37cbc65d96de2eac1bee638`; MIT `LICENSE@a67dca8b4f65d6bd351f6b1e333ce2cd84d843a5`.

The workflow still references mutable major tags. These resolutions are observation/provenance only and must be rebound at frozen-candidate execution.

## Host setup and toolchain

Checkout recursively initializes submodules into a repository-name subdirectory. The host setup changes ownership below `/usr/local/share`, refreshes Homebrew, tolerates `brew upgrade` failure with `|| true`, installs unpinned automake/libtool/meson/nasm/ninja/pkg-config, disables Spotlight, selects the Xcode application developer directory and edits Homebrew CMake's GNU compiler module. These are mutable, host-global build-environment side effects; there is no rollback/compensation.

Exact prepare lineage is `Telegram/build/prepare/mac.sh@82810b6cd5a13c78233232bee38602dcf1707de4` -> `prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`. `qt_version.py@24c81dcf88775f24943295d4477e2357f13f00f1` resolves macOS Qt 6.11.2. `build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` carries preserved release toolchain validation, while prepare defaults `TDESKTOP_MAC_TOOLCHAIN` to auto and may select system Xcode/SDK when the private release toolchain is not selected. This workflow does not set the selector.

The prepare graph also owns fixed-U dependency acquisitions, Rust 1.96.1 Apple targets and mutable bootstrap surfaces including `sh.rustup.rs` and Chromium gyp master when reached. Existing build-time-acquisition inventory is the detailed authority; candidate evidence must bind actual reached inputs rather than assuming a private/system path.

## Cache and dependency preparation

Outer Actions cache stores `Libraries` and `ThirdParty`. Its primary key is runner OS plus a hash of exact checked-out `prepare.py`; restore fallback is the broad OS prefix.

Prepare has a second internal cache-integrity layer. It writes per-stage `cache_keys`, computes stage keys from environment/toolchain plus dependency file hashes, classifies entries Good/NotFound/Stale, and skips only Good. Invocation is `skip-release silent`; `skip-release` excludes release-only stages and `silent` enables automatic stale rebuild instead of interaction. Therefore outer cache reuse is optimization; internal cache keys can reject stale material, and neither layer is immutable source/dependency evidence.

After prepare the workflow prunes `Libraries` with a destructive find rule, preserving static libraries, headers/includes, selected CMake/pkg-config metadata, object/cache-key/patch trees and executables while deleting other regular files and empty paths. Partial prune state is a failure boundary.

## Build, signing and artifact boundary

Exact build chain is `Telegram/configure.sh@c550d988cff5fe3047311cc330aeee943ee328e5` -> `configure.py@592b2f02fd0030dffe6a79d94f1d18c529c53645` -> CMake. The only matrix value leaves DEFINE empty and sets `ARTIFACT_NAME=Telegram`.

The source requests Debug, warnings-as-errors, API-test build support, auto-update/crash-report disable flags OFF, and `CMAKE_XCODE_ATTRIBUTE_CODE_SIGNING_ALLOWED=NO`, then runs a parallel CMake build. There is no test command. A build configured with `TDESKTOP_API_TEST=ON` is not test-execution evidence.

Artifact staging moves `Telegram.app` and then `Updater` into `out/Debug/artifact`. If the second move fails, the first may already be settled. Upload uses name `Telegram`, the staged path, and one-day retention. No digest is source-recorded and there is no installer/package construction, signing, notarization, install, SBOM, attestation or release publication.

Fabushi already has a stronger canonical macOS package/release owner at `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`, `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`, `desktop/scripts/notarize-after-sign.cjs@5f140c6ff8587b4969e1611ec9030a8c5dc5a6f1` and existing Electron update owners. The unsigned Debug Telegram artifact is source provenance only and cannot replace Fabushi package/sign/notarize/install/release evidence.

## Callers and consumers

Direct caller is GitHub push/pull_request event admission. Direct dependencies/consumers are checkout/submodules, Homebrew/Xcode/CMake, prepare/configure/dependency services, GitHub cache/artifact stores and maintainers/reviewers/manual artifact users.

All 20 fixed-U workflow blobs were inspected for direct `mac.yml`, `workflow_call`, `workflow_run`, `prepare/mac.sh` or sibling packaged coupling. None calls `mac.yml`. `canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947` independently calls `prepare/mac.sh silent`; `mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc` is an independent sibling graph.

## Security and failure recovery

No explicit workflow permissions plus checkout token authentication, recursive submodules and PR-controlled source form a credential/trust boundary. Effective token permissions, checkout credential persistence/cleanup and submodule credentials are Stage-B evidence and token values must never be captured.

Mutable runner labels, Action major tags, Homebrew metadata/packages, Xcode/SDK/toolchain, rustup/gyp/network acquisitions and broad cache-prefix restores are provenance boundaries. Host-global chown/Spotlight/Xcode/CMake mutations and destructive Library pruning also require isolation/recovery evidence. Cache content is never identity proof.

There is no workflow-level timeout, retry, concurrency group, transaction, rollback or compensation. The tolerated Homebrew upgrade failure can continue with mixed state; hard failures stop normal progression. Cache/prune/staging may be partial. A later successful run cannot retroactively prove an earlier execution.

## Ownership and authored verification

Existing-owner-first maps applicable behavior to current Fabushi repository build/toolchain + canonical macOS package/release/process + repository security/provenance + shared quality/evidence governance. Rejected owners are a copied Telegram macOS workflow, second macOS build/package/release truth, product runtime/provider/sidecar, Tasks/Automations, Agent/Coordinator/Host/Runner and platform-core.

Authored only, not run: 28 contract, 1 integration, 16 fault/recovery and 14 security/provenance cases. Stage-B oracles: `ORA-TDRP-MAC-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-MAC-WORKFLOW-TRIGGER-RUNNER-PERMISSIONS-001`, `ORA-TDRP-MAC-WORKFLOW-ACTION-TOOLCHAIN-LINEAGE-001`, `ORA-TDRP-MAC-WORKFLOW-CACHE-PREPARE-CONSISTENCY-001`, `ORA-TDRP-MAC-WORKFLOW-BUILD-OUTPUT-IDENTITY-001`, `ORA-TDRP-MAC-WORKFLOW-ARTIFACT-PUBLICATION-001`, `ORA-TDRP-MAC-WORKFLOW-FAILURE-RECOVERY-001`. Invariants: `INV-TDRP-MAC-WORKFLOW-NO-IMPLICIT-PERMISSION-TRUST-001`, `INV-TDRP-MAC-WORKFLOW-NO-MUTABLE-TAG-RUNNER-TRUST-001`, `INV-TDRP-MAC-WORKFLOW-CACHE-NOT-IDENTITY-001`, `INV-TDRP-MAC-WORKFLOW-UNSIGNED-DEBUG-NOT-RELEASE-001`, `INV-TDRP-MAC-WORKFLOW-NO-TELEGRAM-REPOSITORY-AUTHORITY-001`, `INV-TDRP-MAC-WORKFLOW-NO-STAGE-A-EXECUTION-001`.

No workflow run/query, external Action/cache/bootstrap execution, Homebrew/Xcode/CMake mutation, build, test, artifact upload/download, package, signing, notarization, release, deployment, installation, runtime acceptance or platform-core mutation is performed in Stage A.
