# desktop-app/MicroTeX githooks/pre-commit complete read — strict186

## Authority
- Parent PR #154 / branch exact HEAD before receipt: `73d1f353c12b90806c6ae4e329037c7c3ae9a856`, tree `fdb4d1c5a5a27da2e23da60b13b93b718836a2d6`.
- Live Fabushi main: `16aae12ea258f6a872a46aa47824766f64e1bdca`, tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`; current `master` equals the pin; recursive tree truncated=false, 315 entries / 279 blobs.
- Pinned cmake_helpers: `7ceca3019b4750d7a5acb2de0faffd523ee14095`, tree `f5c386cb34208309aae521187611fac4a2045577`.
- Target: `githooks/pre-commit@c8ff63e581ecabf277c9592f8967a7bf665587b8`, tracked mode `100644`, 20 line slots / 424 characters.
- Pre-write parity-ledger independent registration count: zero for both the exact path and blob SHA.

## Complete source behavior
The Bash file labels itself “Check compiling before commit”. If an external Git-hook installation actually dispatches it, it prints a cyan START BUILD banner, captures `pwd` in `workingDir`, changes into `build`, runs `cmake -DCMAKE_BUILD_TYPE=Debug -DGRAPHICS_DEBUG=ON -DMEM_CHECK=OFF -DHAVE_LOG=ON ..`, runs `make -j32`, changes back with `cd $workingDir`, then prints a DONE banner.

The script does not use `set -e`, `set -o pipefail`, status capture, conditional failure handling, traps, or an explicit final `exit`. Under normal Bash semantics, a failed `cd build`, CMake configure, or `make -j32` does not stop later commands; if the final `echo` succeeds, its status can become the hook's successful exit status. Therefore this source is not a fail-closed compile/test gate. `workingDir` is also expanded unquoted on the return `cd`, and the hook assumes an already-created `build/` directory.

## Direct consumer and installation boundary
- The only semantic dispatcher is Git's hook mechanism after an external installation/configuration step. In the pinned repository, the file itself is tracked as `100644`, not executable.
- Exact default-branch searches are exact-ref equivalent because MicroTeX `master` currently equals the pin. Searches for `githooks`, `core.hooksPath`, `pre-commit`, `START BUILD`, and the hook's description returned no tracked installer/consumer reference.
- `README.md@824af04fd8607532efed3f73752b81e5093577ef` documents manual `mkdir build; cd build; cmake ..; make -j32`, but no Git-hook installation.
- Root `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` supplies the build graph and defines `HAVE_LOG=ON`, `GRAPHICS_DEBUG=ON`, `MEM_CHECK=OFF`, `QT=OFF`, and `BUILD_EXAMPLE=OFF` defaults. The hook overrides only the first three listed diagnostic options plus Debug build type.
- `.gitignore@d72676278fbc7981de253a1a0725eb0677bedf30` excludes `build/`, so the hook's intended configure/build side effects are ephemeral worktree output rather than pinned source entries.
- Sibling `githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04` is a separate interactive sample-regeneration hook and is not a direct consumer of this pre-commit file.

## Accepted tdesktop shipping separation
Accepted tdesktop does not install or invoke this hook in its shipping composition. `desktop-app/cmake_helpers:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334` directly compiles the pinned fork and bundled resources into `external_microtex_bundled`; `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `desktop-app::external_microtex`. Shipping `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` owns bundled-resource initialization, input/image limits, exception containment and QImage rendering; `iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` owns normalized caching/fallback; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` includes `td_iv` and packages `external_microtex_bundled.rcc`. Those production responsibilities remain independently open.

## Failure and negative oracles
- Tracked mode `100644` plus absence of a tracked installer/configuration means repository presence alone does not activate a Git pre-commit hook.
- Missing `build/` is not guarded; after a failed `cd build`, subsequent commands still run from the prior directory.
- CMake or `make -j32` failure is not propagated fail-closed; the final successful banner can mask an earlier failure.
- `cd $workingDir` is unquoted, so whitespace/glob-sensitive paths are not robustly restored.
- Fixed `-j32` is a developer convenience assumption, not a product concurrency/resource policy.
- The hook runs no `ctest`, packaged test, runtime acceptance, artifact provenance, or release verification.
- Local ignored build output cannot substitute for exact-head GitHub Actions evidence.
- No `PreCommitGate`, `MicroTeXHookRuntime`, second build owner, second CI owner, or shipping formula/render owner is introduced.

## License
`LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT. `README.md@824af04fd8607532efed3f73752b81e5093577ef` keeps fonts/XML resources under separate licenses. The hook is retained only as source/developer-tooling provenance.

## Coverage
Before: source_entries=16379 unknown=16191 unread=16166 omitted=0 responsibilities=203.
After: source_entries=16379 unknown=16190 unread=16165 omitted=0 responsibilities=204.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance is not accepted; production MicroTeX responsibilities remain open.

## Parent exact-head evidence
Parent `73d1f353c12b90806c6ae4e329037c7c3ae9a856`: TDRP Revision 9 run 40108442473 terminal success 61/61; authority artifact 12565462338 sha256:584a7eed05e949a44a4c40ca8191316131b13ff56a25b8dac51c49c89a5ca388; recursive closure artifact 12564889446 sha256:d55120be8ba1e2781ec9c5ea6e849c1f07d82d5c743a62aea9499cc6de9fa827; recursive reachability artifact 12565294593 sha256:3260ccd7500f603b60a51654b65db69b8ec04b3229ee30e082f69bc0358461c7; Boost reachability artifact 12565448763 sha256:c169e7fddaa76e3da65de99e234dc1f28f4b315dbc0548e61ce8518f252137b0. Rust desktop runtime run 40108442190 terminal success 6/6; artifact 12565856269 sha256:e094552d508e2962dbb33918c44c9a533d607bc16d943803676ca253d4497377. Official MCP marketplace run 40108442147 terminal success 1/1. Receipt-head Actions are required.

## Traceability IDs
- CAP-DEPENDENCY-SOURCE-PROVENANCE
- CAP-DEPENDENCY-DEVELOPER-HOOK-PROVENANCE
- CAP-DEPENDENCY-BUILD-VALIDATION-PROVENANCE
- TDRP-R9-MICROTEX-GITHOOKS-PRECOMMIT-PROVENANCE-001
- TDRP_MICROTEX_GITHOOKS_PRECOMMIT_PROVENANCE_001
- ORA-TDRP-MICROTEX-PRECOMMIT-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-PRECOMMIT-INSTALLATION-BOUNDARY-001
- ORA-TDRP-MICROTEX-PRECOMMIT-FAILURE-PROPAGATION-001
- ORA-TDRP-MICROTEX-PRECOMMIT-SHIPPING-SEPARATION-001
- INV-TDRP-MICROTEX-PRECOMMIT-NO-PARALLEL-BUILD-GATE-001
- INV-TDRP-MICROTEX-PRECOMMIT-NOT-FAIL-CLOSED-001
- INV-TDRP-MICROTEX-PRECOMMIT-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-PRECOMMIT-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-PRECOMMIT-DEVELOPER-HOOK-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-PRECOMMIT-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-PRECOMMIT-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-PRECOMMIT-MASKED-BUILD-FAILURE-001
- FAULT-TDRP-MICROTEX-PRECOMMIT-PRODUCT-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
