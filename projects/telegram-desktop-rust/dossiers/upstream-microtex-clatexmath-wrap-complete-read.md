# desktop-app/MicroTeX clatexmath.wrap complete read — strict183

## Authority
- Parent PR #154 / branch exact HEAD before this write: `e46b8e9f0c95f3020b4b8bdd61195f5a6dc0dd94`, tree `a6a8c17390b0cd93e50d74658f8763ff1c926124`.
- Live Fabushi main at reconciliation: `2d915242d67a9ad1c335459b34d8a8c8561605ab`.
- Accepted Telegram Desktop: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Accepted tdesktop gitlinks re-read from that tree: `cmake@7ceca3019b4750d7a5acb2de0faffd523ee14095`; `Telegram/ThirdParty/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.
- Pinned MicroTeX child: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`.
- Pinned cmake_helpers child: `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`, tree `f5c386cb34208309aae521187611fac4a2045577`.
- Target: `clatexmath.wrap@8f3ab2d23c4dee823ec31aeeb5843950ed9b6a33` (120 Git-tree bytes; 7 line slots including the trailing slot).
- Pre-write ledger exact target occurrence: zero.

## Complete source read
The complete target is:

`[wrap-git]`
- `url = https://github.com/NanoMichael/cLaTeXMath.git`
- `revision = head`

`[provide]`
- `dependency_names = clatexmath`

This is Meson wrap/package-discovery metadata. It names a remote Git acquisition and advertises the dependency name that a downstream Meson project could request. It does not itself compile MicroTeX, select the tdesktop shipping fork, initialize formula resources, render formulas, or define Fabushi product state.

## Mutable acquisition boundary
The source literally uses `revision = head`, not an immutable commit. The historical URL now redirects on GitHub to `NanoMichael/MicroTeX`; its default branch is `master`, observed at `0e3707f6dafebb121d98b53c64364d16fefe481d` / tree `960f63c2bb1f2541b7f4ba40dfb1de5de9454a08` during strict183.

That observation is provenance only. Because the wrap says `head`, no observed commit can be promoted to accepted authority without changing the source contract. Future branch movement or repository redirection is expected to change what a downstream wrap resolver could acquire.

## Fixed-child Meson consumer check
The fixed MicroTeX tree is `truncated=false`, 315 entries / 279 blobs. All 18 Meson manifest/options blobs were re-read by exact SHA (7,352 characters total):
- root `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` and `meson_options.txt@998b92ad0052398df0f7843422c79f674c20dd58`;
- `src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a`;
- the atom/box/core/fonts/graphic/platform/res/samples/utils Meson files in the fixed tree.

Across those fixed manifests there is no `subproject(...)` and no wrap reference. Root Meson declares the `clatexmath` project, and `src/meson.build` builds the local `clatexmath` library and `clatexmath_dep = declare_dependency(...)`. The dependency calls found are ordinary platform dependencies such as fontconfig/gdkmm, qt5, gtkmm and gtksourceviewmm. Therefore this wrap is not consumed by MicroTeX's own fixed Meson build graph; it is downstream acquisition/package-discovery metadata.

## Accepted tdesktop shipping boundary
The accepted tdesktop tree pins the desktop-app fork directly as the `Telegram/ThirdParty/MicroTeX` gitlink. It also pins `cmake_helpers@7ceca301...`.

Pinned `cmake_helpers/external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334` explicitly says there is no system-package fallback and that the desktop-app fork is required. It creates `external_microtex_bundled`, compiles the pinned fork's source set including `src/platform/qt/graphic_qt.cpp`, bundles `res/bundled.qrc`, and exposes `desktop-app::external_microtex`.

Pinned `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` includes `iv_markdown_microtex.cpp/.h` and links `desktop-app::external_microtex`. Pinned `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` initializes the bundled resource and `tex::LaTeX::initBundled()`, then owns bounded measure/render calls and the Qt graphics adapter. Pinned `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies `external_microtex_bundled.rcc` into the macOS app resources.

Thus the mutable upstream wrap is not the tdesktop shipping acquisition path. The real formula parse/measure/render/resource/package responsibilities remain independent production work and are not closed by this source-only row.

## README and license boundary
`README.md@824af04fd8607532efed3f73752b81e5093577ef` documents the child's Meson build and runtime `LaTeX::init/release` behavior. Root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT, while README explicitly excludes fonts/XML resources from the blanket project license. This wrap row does not flatten that layered resource-license boundary.

## Negative oracles
- `revision=head` is mutable and cannot be treated as an immutable accepted source pin.
- The current redirected repository/master SHA is an observation, not shipping authority.
- None of the 18 fixed child Meson manifests consumes a wrap/subproject.
- Accepted tdesktop pins the desktop-app MicroTeX fork directly and compiles it through pinned cmake_helpers.
- `dependency_names = clatexmath` is package-discovery metadata; it is not a Fabushi dependency registry or runtime owner.
- This row does not close formula parsing, measuring, rendering, Qt painting, bundled resources, packaging, licensing, platform behavior or release acceptance.
- No `MicroTeXRuntime`, `TelegramRenderer`, second package manager, or second build graph is introduced.

## Traceability and coverage
Target symbol: `TDRP_MICROTEX_CLATEXMATH_WRAP_PROVENANCE_001`.
Responsibility: `TDRP-R9-MICROTEX-CLATEXMATH-WRAP-PROVENANCE-001`.
Selected owner: existing TDRP recursive source/dependency acquisition and package-discovery provenance governance.
Production entrypoints: none for this wrap row.

Before: source entries 16,379; unknown 16,194; unread 16,169; omitted 0; responsibilities 200.
After: source entries 16,379; unknown 16,193; unread 16,168; omitted 0; responsibilities 201.

## Traceability IDs
- `CAP-DEPENDENCY-SOURCE-PROVENANCE`
- `CAP-DEPENDENCY-BUILD-TIME-ACQUISITION-PROVENANCE`
- `CAP-DEPENDENCY-PACKAGE-DISCOVERY-PROVENANCE`
- `TDRP-R9-MICROTEX-CLATEXMATH-WRAP-PROVENANCE-001`
- `TDRP_MICROTEX_CLATEXMATH_WRAP_PROVENANCE_001`
- `ORA-TDRP-MICROTEX-CLATEXMATH-WRAP-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-CLATEXMATH-WRAP-MUTABLE-HEAD-001`
- `ORA-TDRP-MICROTEX-CLATEXMATH-WRAP-MESON-NO-CONSUMER-001`
- `ORA-TDRP-MICROTEX-CLATEXMATH-WRAP-SHIPPING-PIN-SEPARATION-001`
- `ORA-TDRP-MICROTEX-CLATEXMATH-WRAP-NO-PRODUCTION-CLOSURE-001`
- `INV-TDRP-MICROTEX-CLATEXMATH-WRAP-MUTABLE-NON-AUTHORITY-001`
- `INV-TDRP-MICROTEX-CLATEXMATH-WRAP-NO-PARALLEL-RUNTIME-001`
- `INV-TDRP-MICROTEX-CLATEXMATH-WRAP-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
- `CONTRACT-TDRP-MICROTEX-CLATEXMATH-WRAP-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-CLATEXMATH-WRAP-ACQUISITION-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-CLATEXMATH-WRAP-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-CLATEXMATH-WRAP-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-CLATEXMATH-WRAP-MUTABLE-HEAD-DRIFT-001`
- `FAULT-TDRP-MICROTEX-CLATEXMATH-WRAP-PACKAGE-PROMOTION-001`
- `FAULT-TDRP-MICROTEX-CLATEXMATH-WRAP-PRODUCT-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`

Dependency anchors:
- `Telegram/ThirdParty/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`
- `desktop-app/MicroTeX:clatexmath.wrap@8f3ab2d23c4dee823ec31aeeb5843950ed9b6a33`
- `NanoMichael/MicroTeX@master -> 0e3707f6dafebb121d98b53c64364d16fefe481d (mutable observation only)`
- `cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334`
- `Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30`
- `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2`
- `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277`
- `desktop-app/MicroTeX:README.md@824af04fd8607532efed3f73752b81e5093577ef`
- `desktop-app/MicroTeX:LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2`

## Parent exact-head evidence
Parent `e46b8e9f0c95f3020b4b8bdd61195f5a6dc0dd94` is terminal-success:
- TDRP Revision 9 run `40102519928`: 61/61 success; authority `12562794860` sha256:e3168e9bd2c7c1eda128e315d1628fb3b347bff5d6e85f9b4ed88c32f169ac60; closure `12562714036` sha256:d4c611f44165b5a2565bdb4ec08ad1a5120bcc76084b77afa6ca4870a15a114e; reachability `12563488092` sha256:805d1f5551c9d35de3217937cd524c3b3d597f66573b40584fe11c31b47e2978; Boost `12563537572` sha256:2c7540999bdd6b2a537de0ae60451f79832a0b035b18f1ffbe062f241f5a043e.
- Rust desktop runtime run `40102519960`: 6/6 success; artifact `12562589533` sha256:a0b8accd531113c1638d59d9db0fa5e68a4dcf2e9b83656be1f2dc11e120a68b.
- Official MCP marketplace run `40102519980`: 1/1 success.

These are parent-head evidence. The strict183 receipt commit requires fresh exact-head terminal results.

## Closure
- `baseline_identity_accepted=false`
- `source_closure_ready=false`
- independent release acceptance remains not accepted
- no production formula/render/resource/platform/package implementation or shipping acceptance is claimed by this mutable-wrap provenance row.
