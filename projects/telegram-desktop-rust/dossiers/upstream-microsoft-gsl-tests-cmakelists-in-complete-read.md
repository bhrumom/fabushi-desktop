# Microsoft/GSL tests/CMakeLists.txt.in complete read — strict178

## Authority

- Parent PR #154 / branch exact HEAD before this write: `08f1d0c64ebc342d268d4fc161f768310765c209`, tree `5a95a2ed402c4aefc272752ed06793cd1ddacb84`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned child: `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`, tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Target blob: `tests/CMakeLists.txt.in@2535f8f4a53b0a39c9df1ef65f9f216416464ea4` (477 bytes; 15 line slots).
- Pinned child tree recursive enumeration: `truncated=false`, 51 entries / 44 blobs. Exact path+blob comparison with the live ledger before this write left exactly one unmatched blob: this target.

## Complete source read

The file is a CMake project named `googletest-download`. It includes `ExternalProject` and declares one `ExternalProject_Add(googletest)` with repository `https://github.com/google/googletest.git`, immutable commit `1b18723e874b256c1e39378c6774a90701d70f7a`, source/binary directories under the current build tree, and empty configure/build/install/test commands. It is therefore a download/materialization template, not the actual GoogleTest build/test owner.

## Generation and consumption chain

Pinned `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a` owns the caller chain. It probes `gtest_main` via pkg-config; if absent it requires Git, calls `configure_file(CMakeLists.txt.in googletest-download/CMakeLists.txt)`, configures and builds that generated project, and then adds the materialized `googletest-src` / `googletest-build` with `add_subdirectory(... EXCLUDE_FROM_ALL)`. GSL tests then link `Microsoft.GSL::GSL` and the test configuration / `gtest_main`.

Negative oracle: the template's ExternalProject configure/build/install/test commands are all empty. The template only acquires the pinned source; it does not itself compile or execute GoogleTest.

## CMake package-discovery callers

- `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`: when tests/ is the top-level source, calls `find_package(Microsoft.GSL CONFIG REQUIRED)`, requires `Microsoft.GSL_VERSION`, and links `Microsoft.GSL::GSL`.
- `.github/workflows/cmake_find_package.yml@fd2f7a812ca21e4e125058e50acadd6b5ccb28ec`: on Ubuntu/macOS configures GSL with `GSL_TEST=OFF` and an install prefix, installs it, then configures tests/ with `CMAKE_PREFIX_PATH` pointing to that install tree.
- Root `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda`: defines `GSL` and alias `Microsoft.GSL::GSL`; under `GSL_INSTALL`, exports/installs config/version, headers and NatVis.

This validates child dependency install/discovery mechanics only, not Fabushi shipping package/build ownership.

## README and license/provenance

- `README.md@dd61e3fdc4d30614c0ca35f58cce5f58e66dd4df` states the implementation is inline/header-only and documents direct include paths, `find_package(Microsoft.GSL CONFIG REQUIRED)`, `Microsoft.GSL::GSL`, and FetchContent.
- `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02` is MIT.
- `ThirdPartyNotices.txt@552b254c63a2f9909aa46ef21c256ef9a79a0035` records GoogleTest under BSD 3-Clause terms.

The test-only GoogleTest acquisition is license/provenance evidence; it is not proof Fabushi ships GoogleTest.

## Accepted tdesktop negative oracle

At accepted `tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, exact search for `ThirdParty/GSL` resolves only `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099`; exact searches for `Microsoft.GSL` and `GSL_TEST` return zero. tdesktop production source does use GSL header types/utilities (for example `Telegram/SourceFiles/stdafx.h@252ab2c2aa698e3337894ab6d50acef697789036` includes `<gsl/gsl>`), but that does not make this child test template or package-discovery workflow a production entrypoint.

No `GslTestRuntime`, `GslPackageRuntime`, `TelegramBuildRuntime`, second package owner, or second production build root is created.

## Owner disposition

Selected owner: existing TDRP recursive source/dependency and CI/test-provenance governance. Fabushi shipping build/package/release owners are rejected because this is third-party child test scaffolding. New GSL/Telegram runtimes are rejected because they would promote dependency provenance into product architecture and violate existing-owner-first. Production entrypoints and production evidence remain empty.

## Coverage accounting

Before this write: source entries 16,379; unknown 16,199; unread 16,174; omitted 0; responsibilities 195. The fixed Microsoft/GSL child tree had 44 blobs, 43 already registered, and this target was the sole unmatched exact path+blob.

After recording this source: unknown 16,198; unread 16,173; omitted 0; responsibilities 196; fixed Microsoft/GSL child tree exact path+blob registration 44/44. This does not imply accepted tdesktop source closure: thousands of other recursive entries remain unread/unknown.

## Parent exact-head evidence

Parent HEAD `08f1d0c64ebc342d268d4fc161f768310765c209` is terminal-success:
- TDRP Revision 9 run `40093199412`: 61/61 success; authority artifact `12559188820` sha256:a14ef23f18ab27866451555347228fecc871cd32c307dbee55d708c2f81dcdec; recursive closure `12559862862` sha256:3cd729b373c8e5c3774bb1f791a11adfedb3427376e45077d9e5bcf7825ec1b9; recursive reachability `12559842992` sha256:1e2e7c41dee6642dfbf6a4db5cf55bb7cf764aa386324a6e2397c113fda0c924; Boost `12559613559` sha256:1b13a4928f4dc00371a131e7322a96f3f40ea447f277086166b22eab7f47203e.
- Rust desktop runtime run `40093199398`: 6/6 success; artifact `12560170521` sha256:bd8a1d10e3296fffdea343a9a18fa1e42bc03c06c9a0c4dbb019079dc52696e3.
- Official MCP marketplace run `40093199411`: contracts success.

These are parent-head evidence; the strict178 receipt commit must obtain its own exact-head workflow terminal results.

## Closure

- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- no production implementation or shipping-owner claim is made for this dependency test/package source.
