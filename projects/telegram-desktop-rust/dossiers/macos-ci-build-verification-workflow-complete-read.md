# macOS CI build verification workflow complete read — strict index 88

## Authority
- Upstream: `telegramdesktop/tdesktop@52a47639d9aa38713098974dd2ce07e631cb3381`, tree `42998681c09cdffcf75c0f38df71f663fc7b7b28`.
- Source: `.github/workflows/mac.yml@184a94b091d2d7bab19ae67d7f3a5881b18213ba`, 138 line slots / 4273 bytes.
- Parent exact head/tree: `a6cda99db19e161ce0fb10293ce9dfab6a774d1e` / `81ef64453a23de9deb3cb8745f9056f80cad1ced`.
- Parent exact-head a6cda99db19e161ce0fb10293ce9dfab6a774d1e completed TDRP run 39802801210 success (authority 12436243276 sha256:d8da6ce0fdc6223df6608a3f93591898c9bc4853d4ae76cd258753888d47dfc2; closure 12435669870 sha256:3039c7e0645c5a2800dda1d8a2a8f802e912cd6fb92744e4cfe429b558955478; reachability 12437790900 sha256:85e4887959f310b9f372bf0728680421d2d5cb6e7488987d9a9af05005a5e388) and Rust desktop runtime run 39802800404 at 6/6 success (artifact 12437170317 sha256:391fa1075fac02659cb1242ba046eadf885583d0a5967fc71f598131553c5526).
- Existing executable-validation owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` — `For FBCP/TDRP, executable validation remains exact-head GitHub-Actions-only.`

## Complete-read behavior
The workflow handles macOS-relevant pushes and pull requests, excludes canary pushes and broad docs/legal/Linux/Windows/snap/Docker paths while explicitly retaining its own workflow path. Pull requests, tags and nightly use a Depot macOS runner; other refs use the GitHub macOS runner.

It recursively checks out source, mutates the runner through Homebrew/Xcode/CMake setup, restores a Libraries/ThirdParty cache keyed by the prepare script, invokes `Telegram/build/prepare/mac.sh skip-release silent`, deletes nonessential workspace material to reduce disk pressure, then configures an unsigned Debug build with warnings-as-errors and `TDESKTOP_API_TEST=ON`. Autoupdate and crash-report code paths remain enabled. It builds through CMake, stages `Telegram.app` plus `Updater`, and uploads a one-day artifact.

## Direct input and provider provenance
- `actions/checkout@v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1`.
- `actions/cache@v6 -> 55cc8345863c7cc4c66a329aec7e433d2d1c52a9`.
- `actions/upload-artifact@v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9`.
- `Telegram/build/prepare/mac.sh@82810b6cd5a13c78233232bee38602dcf1707de4`.
- `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`.
- `Telegram/configure.sh@c550d988cff5fe3047311cc330aeee943ee328e5`.
- Homebrew packages, Xcode, CMake modules and runner images are mutable CI environment provenance rather than accepted Fabushi source/release authority.

## Existing-owner-first disposition
This is source-build verification and supply-chain provenance, not a Telegram runtime to preserve. Fabushi already owns exact-head executable validation, packaging/signing boundaries, FQT evidence and independent release acceptance. The upstream workflow maps into those existing owners without introducing a macOS product runtime, cache truth, packaging root, signing owner or release authority.

## Fail-closed conclusions
- Recursive checkout and prepare-time acquisition remain source/build provenance and cannot silently promote unknown dependencies.
- A cache hit is not correctness evidence.
- Runner/Homebrew/Xcode drift is non-passing provenance unless current-head evidence revalidates it.
- The workflow explicitly disables code signing; Telegram.app and Updater output cannot establish Fabushi signed/package release readiness.
- One-day artifact existence or upstream build success cannot substitute for Fabushi packaged/device/release acceptance.
- Cancelled, failed, stale-head, missing-output or expired-artifact runs are non-passing.

## Exact traceability
- Requirement: `TDRP-R9-MACOS-CI-BUILD-VERIFICATION-PROVENANCE-001`.
- Oracles: `ORA-TDRP-MACOS-CI-EXACT-SOURCE-001`, `ORA-TDRP-MACOS-CI-BUILD-RESULT-001`, `ORA-TDRP-MACOS-CI-NO-RELEASE-AUTHORITY-001`.
- Invariants: `INV-TDRP-MACOS-CI-CACHE-NO-AUTHORITY-001`, `INV-TDRP-MACOS-CI-UNSIGNED-NO-RELEASE-001`, `INV-TDRP-MACOS-CI-EXACT-HEAD-EVIDENCE-001`.
- Contracts: `CONTRACT-TDRP-MACOS-CI-BUILD-PROVENANCE-001`, `CONTRACT-TDRP-MACOS-CI-FABUSHI-RELEASE-BOUNDARY-001`.
- Temporal/fault: `TEMP-TDRP-MACOS-CI-ACTION-TAG-DRIFT-001`, `TEMP-TDRP-MACOS-CI-RUNNER-HOMEBREW-DRIFT-001`, `TEMP-TDRP-MACOS-CI-CACHE-DRIFT-001`, `FAULT-TDRP-MACOS-CI-PREPARE-ACQUISITION-001`, `FAULT-TDRP-MACOS-CI-DISK-CLEANUP-001`, `FAULT-TDRP-MACOS-CI-ARTIFACT-001`.
- Security: `SEC-TDRP-MACOS-CI-SUPPLY-CHAIN-001`, `SEC-TDRP-MACOS-CI-UNSIGNED-BOUNDARY-001`.

Closing strict index 88 changes counters to unknown=16,280, unread=16,255, omitted=0, responsibilities=106. `baseline_identity_accepted=false` and `source_closure_ready=false`. Next strict root is index 89 `.github/workflows/mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc`.
