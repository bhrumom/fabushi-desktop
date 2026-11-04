# Complete read — .github/workflows/mac_packaged.yml

## Authority and identity

- Upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
- Entry: tree index 127 zero-based
- Blob: `.github/workflows/mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc`
- Mode/size: 100644; 9,952 ASCII bytes/code points; 254 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-MAC-PACKAGED-WORKFLOW-001`
- Stage-A status: implemented / validation not-run / accepted false

## Trigger, permissions and runner contract

Push ignores public/private canary branches and the source path classes enumerated in the workflow; pull_request uses the same path exclusions but no canary branch exclusion. Both path lists ignore generic `.github/**` and then re-include only this workflow. No schedule, workflow_dispatch, repository_dispatch, workflow_call or workflow_run exists.

The workflow has no explicit `permissions`, environment, concurrency, timeout or retry declaration. It does consume `secrets.GITHUB_TOKEN` in shell API calls that read mutable rnnoise and tg_owt master refs.

The single `defines` matrix value is empty. Pull requests, tag-like refs and nightly use `depot-macos-latest`; ordinary admitted pushes use `macos-latest`. Both are mutable runner labels.

## Toolchain and host contract

Global environment selects Ninja, Debug, FFmpeg/OpenAL Homebrew prefixes, Rust 1.96.1 and exact TDE2E/tlottie/wallet-engine/patches commits. Checkout uses `actions/checkout@v7`, recursive submodules, and a repository-name subdirectory.

Setup refreshes Homebrew, tolerates upgrade failure, installs unpinned build tools and media/Qt packages, selects the system Xcode app and edits Homebrew CMake GNU debug flags. It computes a cache key from Xcode version, installed package versions, manual-cache toggle, workspace path and optionally this workflow's MD5, derives deployment target from the installed Qt6 config, and stores the workspace as `LibrariesPath`.

Observed mutable Action refs on 2026-11-01 are checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`, cache `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`, upload-artifact `cf430e030ddbb5b0abf93d22962f4752f3646cd9`. Adjacent mac workflow evidence already binds their metadata/package/MIT license blobs; the major tags remain mutable.

## Dependency/cache lineage

Rust cache key includes OS, exact tlottie/wallet-engine/patches/Rust identities and workflow cache key. On miss the workflow downloads mutable rustup bootstrap, fetches exact `desktop-app/patches@aec474953ff7ee9b6e4cd9b8658288ea86d124f3`, exact `dkaraush/tlottie@92df98dc209bc39b1e567ec74a8c86a0af5239de`, and exact `i582/wallet-engine@e59e0d89d7ee90c388bf36e3334c5b276f396697`; wallet LFS smudge is disabled, `wallet-engine.patch` is applied, then a local Rust aggregate library is built as staticlib/cdylib and wallet C++ headers are generated through bindgen.

RNNoise cache key uses a master SHA queried through GitHub API; audit-time master was `6cbfd53eb348a8d394e0757b4025c6ded34eb2b6`. On miss the workflow separately shallow-clones master and does not reset to that queried SHA before autotools build/install.

WebRTC cache has the same structural issue: audit-time `desktop-app/tg_owt` master was `020feec5c040fdae2cf752c2c2be62ddde7c236d`, but the miss path separately shallow-clones mutable master recursively with shallow submodules and never resets to the lookup SHA before CMake build/install.

Therefore the lookup SHA is cache-key input, not necessarily the built source identity. A branch move between lookup and clone can produce a mismatched cache key. Actual clone commit and recursive submodule commits are required provenance.

TDE2E is fetched more strongly: `tdlib/td@51743dfd01dff6179e2d8f7095729caa4e2222e9` is fetched by SHA, reset to FETCH_HEAD, then built/installed with `TD_E2E_ONLY=ON`.

## Build, package/sign and artifact contract

All local dependency roots are appended to `CMAKE_PREFIX_PATH`. With the sole empty matrix define, artifact name is `Telegram`. CMake configures the repository with `TDESKTOP_API_TEST=ON`, then builds Debug. No test command follows; the option does not constitute test execution.

After build, `macdeployqt build/Telegram.app` deploys Qt content into the app bundle. The workflow then deliberately executes `codesign --remove-signature build/Telegram.app`. It does not create DMG/ZIP, apply a signing identity, notarize, staple, install, verify updater metadata, compute checksums, create SBOM/attestation, or publish a GitHub release.

When upload is enabled, it creates `build/artifact`, moves only `Telegram.app` there, and uploads it with `actions/upload-artifact@v7` as `Telegram` for one day. The source therefore produces an ephemeral unsigned deployed app bundle, not a release package.

## Direct callers and consumers

All 20 workflow blobs in the fixed-U tree were read for reusable/workflow-run/local-workflow coupling. None invokes `mac_packaged.yml`, and none declares workflow_call/workflow_run. `mac.yml` is an independent sibling build/artifact graph. `canary.yml` independently uses `prepare/mac.sh`; it does not consume this workflow or its artifact.

## Existing-owner-first mapping

Applicable source-neutral responsibility is already owned by:
- repository build/toolchain governance;
- canonical macOS package/release process `.github/workflows/release-macos-main.yml`;
- electron-builder packaging `desktop/package.json`;
- canonical nested signing/notarization `desktop/scripts/notarize-after-sign.cjs`;
- Electron updater/release metadata owner;
- repository security/provenance governance;
- shared Test Strategy/Test Evidence Contract;
- FMEP-001 Stage-A deferred executable validation.

The canonical Fabushi release workflow packages an exact source, uses Developer ID signing/notarization, verifies app/worker identities and installed byte equality, writes SHA-256 checksums, preserves candidate evidence and gates public promotion. Upstream's unsigned one-day bundle must never be treated as equivalent release proof.

Rejected owners are a copied Telegram workflow, second package/release truth, Telegram runtime/provider/sidecar, Tasks/Automations, Agent/Coordinator/Host/Runner runtime, and platform-core.

## Failure/recovery and security findings

Fail-closed conditions include mutable runner/Action/Homebrew/rustup inputs, tolerated partial Homebrew upgrade, recursive checkout/submodule identity, cache poisoning/staleness, patch/build/bindgen failure, mutable rnnoise/tg_owt lookup-to-clone races, recursive tg_owt submodule drift, TDE2E failure, missing local prefix, configure/build/macdeployqt failure, failed signature removal, missing app/staging/upload, and absent artifact digest.

Security/provenance must preserve effective token-scope evidence without secrets, checkout credential boundaries, PR-source trust boundaries, exact external git/patch identities, actual clone commits rather than lookup-only SHAs, and the fact that the published artifact is explicitly unsigned and unattested.

## Stage-A authored verification

Authored-only coverage is 36 contract cases, 1 integration case, 22 fault/recovery cases and 18 security/provenance cases in the shared Test Strategy and Evidence Contract. None is executed in Stage A.

No workflow/Action/cache/bootstrap execution, Actions/check/status query, submodule/network acquisition, Homebrew/Xcode/CMake mutation, build/test, artifact transfer, packaging/signing/notarization, release/deploy/install/runtime acceptance or platform-core mutation occurred while producing this dossier.

The whole-project authority gap remains pre-existing and separate: active ledger/source-closure follows fixed U `f985723...`, while `upstream.lock.json` / inventory metadata still carries older `351204f...` baseline identity. This single-entry closure does not claim baseline reconciliation.
