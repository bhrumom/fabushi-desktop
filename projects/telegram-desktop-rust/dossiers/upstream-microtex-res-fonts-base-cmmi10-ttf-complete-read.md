# desktop-app/MicroTeX res/fonts/base/cmmi10.ttf complete read - strict243

## Authority and exact-path history
PR #154 pre-write lease is `476a7968f5c1bc1522e1ab3b54c7d093438664aa`. Accepted tdesktop is `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; `Telegram/ThirdParty/MicroTeX` pins `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Target `res/fonts/base/cmmi10.ttf` is blob `c12851691ef7a5424f83752e7d39737af9860c79`, exactly 26,708 bytes. Root `17f3fc7816e4de1f5764036f34d759d50098ae58` and c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` resolve to `f10c3af2ddad41897404302d0b514d8d4c9b6a48` (26,576 bytes); c154 `c1546216f32d398dca90e1dc33df34842683ec42` and pinned resolve to the accepted blob.

## sfnt identity and c154 namespace rewrite
Independent binary parsing finds 15 tables, 134 glyphs, unitsPerEm=1000, short loca, bounds `-32,-250..1048,750`, hhea `800/-200/0`, advanceWidthMax=1000, 134 long metrics, OS/2 v3 weight=500 width=5 fsType=0, and format-4 cmap coverage of 133 codepoints. Pinned family/full is `TDesktop cmmi10`, style `Medium`, PostScript `TDesktop-cmmi10`; parent used `cmmi10`.

Per-table comparison shows only `name` and `head` differ. The other 13 tables are byte-identical. In `head`, changed bytes are checksumAdjustment and modified timestamp; glyph/layout/cmap/OS2/hhea semantics are unchanged. `src/platform/qt/graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5` explicitly documents system texlive `cmmi10` collision risk, making the TDesktop prefix an active collision-isolation responsibility.

## Compiled metric, kerning and growth semantics
`src/res/font/cmmi10.def.cpp@54de79039aaba6a2710ddd5eb7bcfc7c3d09f2e2` declares `DEF_FONT(cmmi10, fonts/base/cmmi10.ttf, 0)`, xHeight=.430555, quad=1.000003, skew=196, exactly 128 metric rows, 166 kern rows and 3 next-larger links; style graph is bold=cmmib10, roman=cmr10, ss=cmss10, tt=cmtt10, it=cmti10.

`src/res/font/cmmi10_unchanged.def.cpp@066bd72a3c04cbee18618dc953336389c3d53371` intentionally reuses the same TTF with 57 metrics, 37 kern rows, the same 3 next-larger links and bold=cmmib10_unchanged. `builtin_font_reg.cpp@c92411673b6dfd6b7bd550a20747c5b7eba545bc` registers both definitions; `FontInfo::getFont()` lazily calls `Font::create`.

## Accepted runtime, install and packaging reachability
`res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` embeds `fonts/base/cmmi10.ttf`. `LaTeX::initBundled()` initializes the builtin set; Qt prepends `:/` when disk font is absent and loads through QFontDatabase. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` and calls `LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies the rcc into app Resources and `build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` requires it in release resources.

`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`, standalone Meson install, `src/res/font/meson.build@aa2abb92de7c5c029390060a16df10ff2843d244`, CMake `75059dec500cff8114728331eb5cb317abf6daf9`, and legacy `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` agree on source/install/runtime reachability.

## License and existing-owner-first
MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts/XML under `res` from the project MIT blanket. `Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` permits copy/distribution/modification provided modified software carries a different name. The c154 TDesktop namespace therefore matters to license provenance and collision avoidance. OS/2 fsType is not legal-license authority.

Existing-owner-first selects existing TDRP recursive source/resource/font/rendering/license/install/packaging/provenance governance. `novel_capability=false`; no new MicroTeX/Telegram font registry, Resource owner, font store, packaging root or formula renderer is introduced.

## Coverage and parent exact-head evidence
Before strict243: total=16,379 unknown=16,134 unread=16,109 omitted=0 responsibilities=260. After strict243: total=16,379 unknown=16,133 unread=16,108 omitted=0 responsibilities=261. Baseline identity remains unaccepted, source closure remains open, independent release acceptance remains not accepted.

Parent `476a7968f5c1bc1522e1ab3b54c7d093438664aa`: TDRP Revision 9 run `40219459433` is 30/30 success, including source authority `121247653236` and recursive accounting `121247979043`. Authority artifact `12609077100` digest `sha256:536dd2252126fe81a843451fa756e5f1d168fb2b7dbb4696df0500ee0a2aeafe`; closure `12609071508` digest `sha256:6ae3862c8ea8eb2689b75f7137cb7f8e5847fba50e8c04e98ea3c20c4f2a0f34`; reachability `12608716889` digest `sha256:3d621d50726ece643e24fdd1b6552503530de7d22c73bf909bd370e434073a07`; boost `12609232157` digest `sha256:ad2c0ceb0e49e0dbf3f9bb3b2f6f5b45585c1903176ebb145052e9be68661073`. Rust run `40219459435` is 6/6 success including rust-host `121247653219`; artifact `12608682352` digest `sha256:b3a7591b99544b91affd694a41efa67e5e4a4dc02d53696ca0edf646ab83276d`. Official MCP `40219459427` is 1/1 success, contracts `121247652966`.

Next deterministic entry: `res/fonts/base/cmmib10.ttf@6772eafd5b8c4d0d0a85b12d9e92c4a05d2caf64`.

## Traceability anchors
- TDRP_MICROTEX_RES_FONTS_BASE_CMMI10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-SFNT-NAME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-COMPILED-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-KERNS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-NEXT-LARGER-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-BUILTIN-RUNTIME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-26708-15-134-133-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-C154-NAMESPACE-HEAD-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-128-166-3-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-UNCHANGED-57-37-3-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-BUNDLED-PRODUCTION-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-KNUTH-NAME-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-SFNT-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-COMPILED-MATH-ITALIC-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-RUNTIME-BUNDLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-LICENSE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-NAME-ATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-METRIC-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-KERN-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-LARGER-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-UNCHANGED-VARIANT-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-BUNDLE-OMISSION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMI10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
