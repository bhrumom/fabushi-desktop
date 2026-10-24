# desktop-app/MicroTeX example/CMakeLists.txt complete read — strict184

## Authority
- Parent PR #154 / branch exact HEAD: `a67a56cb4073d375629bb0d325b20a4dcf26257b`, tree `89cce26c414e71c7422b49696fc1be981b38b40e`.
- Live Fabushi main at reconciliation: `16aae12ea258f6a872a46aa47824766f64e1bdca`, tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`; recursive tree `truncated=false`, 315 entries / 279 blobs.
- Pinned cmake_helpers: `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`, tree `f5c386cb34208309aae521187611fac4a2045577`.
- Target: `example/CMakeLists.txt@cd997be0b043913d17c49c1e1e2c50df2c5d6942`, 300 Git-tree bytes.
- Pre-write parity-ledger exact target occurrence: zero.

## Complete source read
The target creates one optional example executable:
- `add_executable(latex2png latex2png.cpp)`
- `find_package(QT NAMES Qt6 Qt5 COMPONENTS Core Gui REQUIRED)`
- `find_package(Qt${QT_VERSION_MAJOR} COMPONENTS Core Gui REQUIRED)`
- `target_link_libraries(latex2png PRIVATE LaTeX Qt${QT_VERSION_MAJOR}::Core Qt${QT_VERSION_MAJOR}::Gui)`.

It is build/example metadata, not a product owner. The two package-discovery calls resolve Qt Core/Gui only for this example; MicroTeX itself is the already-defined local `LaTeX` target.

## Generation and consumption path
Pinned root `CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` creates `LaTeX`, enumerates its core/resource/platform sources, and defines `option(BUILD_EXAMPLE "Build examples" OFF)`. It executes `add_subdirectory(example)` only when `BUILD_EXAMPLE` is explicitly enabled, so the target manifest is absent from the default standalone child CMake graph.

Direct source `example/latex2png.cpp@49d80c656e8437e56cf50cccbad451fa2ae46c91` was fully read only to verify this edge. It creates a QGuiApplication, guards `tex::LaTeX::init()/release()`, validates two CLI paths, reads a TeX file, parses at bounded width/size, paints through `Graphics2D_qt` into a QPixmap, and saves PNG with explicit usage/file/open/save failure codes. That source remains an independent unread/unregistered source entry after strict184 and is not counted by this CMake row.

## Accepted tdesktop shipping boundary
Accepted tdesktop does not consume the child root/example graph.
- Pinned `desktop-app/cmake_helpers:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334` directly builds `external_microtex_bundled`, enumerates the fork's production sources, selects the Qt backend, bundles `res/bundled.qrc`, and exposes `desktop-app::external_microtex`.
- Pinned `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `desktop-app::external_microtex` for the real Instant View markdown/formula consumer.
- Pinned `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies `external_microtex_bundled.rcc` into macOS app resources.

Therefore `latex2png` is dependency example/build provenance only, not a tdesktop/Fabushi shipping executable, renderer owner, package authority, acceptance test, or product entrypoint.

## README and license boundary
`README.md@824af04fd8607532efed3f73752b81e5093577ef` documents standalone CMake/Meson demo builds, Qt mode, headless conversion, runtime `LaTeX::init/release`, parsing/rendering and graphics backends. Root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT. README explicitly excludes fonts/XML resources under `res` from the blanket project license, so this row does not flatten resource licensing.

## Negative oracles
- `BUILD_EXAMPLE` defaults OFF; this manifest is not in the default fixed-child CMake graph.
- The target only builds an upstream example and links the local `LaTeX` library.
- Qt `find_package` calls here are optional-example package discovery, not tdesktop/Fabushi shipping package selection.
- Contextual reading of `latex2png.cpp` does not independently disposition or close that source entry.
- Accepted tdesktop shipping directly compiles the pinned fork through pinned cmake_helpers and links it from `td_iv`.
- This row closes no formula parsing, measuring, rendering, resource initialization, Qt painting, packaging, platform, service, UI, E2E or release responsibility.
- No `MicroTeXExampleRuntime`, `TelegramRenderer`, second package registry or second build graph is introduced.

## Traceability and coverage
Target symbol: `TDRP_MICROTEX_EXAMPLE_CMAKE_PROVENANCE_001`.
Responsibility: `TDRP-R9-MICROTEX-EXAMPLE-CMAKE-PROVENANCE-001`.
Selected owner: existing TDRP recursive source/dependency example-build and package-discovery provenance governance.
Production entrypoints: none.

Before: source entries 16,379; unknown 16,193; unread 16,168; omitted 0; responsibilities 201.
After: source entries 16,379; unknown 16,192; unread 16,167; omitted 0; responsibilities 202.

## Parent exact-head evidence
Parent `a67a56cb4073d375629bb0d325b20a4dcf26257b` is terminal-success:
- TDRP Revision 9 run `40104206558`: 61/61 success; authority `12563787417` sha256:ec6bdf03e11f001533114106b613b5fe9bffe0c335d843cbc7751f67ba3ab613; closure `12564316685` sha256:ac6ea5ef46faa389c3e679de58a9ec5ee15facf2ea155fe649d503860d7ec23c; reachability `12564261113` sha256:f32a174adc02aafcb55ba7f7e725729fb6785a337471ce4f0f3229857527020e; Boost `12564207063` sha256:303f6e343938006084828014528d31971a96c674cea7e9c4958da3694171df05.
- Rust desktop runtime run `40104206565`: 6/6 success; artifact `12563962663` sha256:fa4337f42022f12e307255559cef7487c191039d64890e1dc6e6d3727db4973d.
- Official MCP marketplace run `40104206597`: 1/1 success.

These are parent-head evidence. The strict184 receipt commit requires fresh exact-head terminal results.

## Closure
- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- production MicroTeX formula/render/resource/platform/package responsibilities remain open
- next independent MicroTeX source entry is `example/latex2png.cpp@49d80c656e8437e56cf50cccbad451fa2ae46c91`.

## Traceability IDs
- `CAP-DEPENDENCY-SOURCE-PROVENANCE`
- `CAP-DEPENDENCY-EXAMPLE-BUILD-PROVENANCE`
- `CAP-DEPENDENCY-PACKAGE-DISCOVERY-PROVENANCE`
- `TDRP-R9-MICROTEX-EXAMPLE-CMAKE-PROVENANCE-001`
- `TDRP_MICROTEX_EXAMPLE_CMAKE_PROVENANCE_001`
- `ORA-TDRP-MICROTEX-EXAMPLE-CMAKE-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-EXAMPLE-CMAKE-OPTIONAL-GRAPH-001`
- `ORA-TDRP-MICROTEX-EXAMPLE-CMAKE-SHIPPING-SEPARATION-001`
- `ORA-TDRP-MICROTEX-EXAMPLE-CMAKE-NO-PRODUCTION-CLOSURE-001`
- `INV-TDRP-MICROTEX-EXAMPLE-CMAKE-NO-PARALLEL-RUNTIME-001`
- `INV-TDRP-MICROTEX-EXAMPLE-CMAKE-DIRECT-CONSUMER-INDEPENDENT-001`
- `INV-TDRP-MICROTEX-EXAMPLE-CMAKE-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-EXAMPLE-CMAKE-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-EXAMPLE-CMAKE-OPTIONAL-BUILD-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-EXAMPLE-CMAKE-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-EXAMPLE-CMAKE-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-EXAMPLE-CMAKE-EXAMPLE-PROMOTION-001`
- `FAULT-TDRP-MICROTEX-EXAMPLE-CMAKE-PRODUCT-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
