# desktop-app/MicroTeX res/fonts/base/cmmib10.ttf complete read - strict244

## Authority and exact-path history
PR #154 pre-write lease is `128eadfe2836d61e513c42e93d07e6ff1dcc190f`. Accepted tdesktop is `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; `Telegram/ThirdParty/MicroTeX` pins `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Target `res/fonts/base/cmmib10.ttf` is blob `6772eafd5b8c4d0d0a85b12d9e92c4a05d2caf64`, exactly 24,484 bytes. Root `17f3fc7816e4de1f5764036f34d759d50098ae58` and c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` resolve to `5ae39768acaed536d7d13305421c7b86b57397dd` (24,348 bytes); c154 `c1546216f32d398dca90e1dc33df34842683ec42` and pinned resolve to the accepted blob.

## sfnt identity and c154 namespace rewrite
Independent binary parsing finds 15 tables, 134 glyphs, unitsPerEm=1000, short loca, bounds `-15,-250..1216,750`, hhea `750/-250/90`, advanceWidthMax=1150, 134 long metrics, OS/2 v3 weight=700 width=5 fsType=0, and format-4 cmap coverage of 133 codepoints. Pinned family/full is `TDesktop cmmib10`, style `Bold`, PostScript `TDesktop-cmmib10`; parent used `cmmib10`.

Per-table comparison shows only `name` and `head` differ. The other 13 tables are byte-identical. In `head`, changed bytes are checksumAdjustment and modified timestamp; glyph/layout/cmap/OS2/hhea semantics are unchanged. The accepted Qt backend documents system TeX font-family collision isolation, so the c154 TDesktop namespace is an active resource-identity responsibility rather than cosmetic metadata.

## Compiled metric, kerning and growth semantics
`src/res/font/cmmib10.def.cpp@0b207285e1bdaea4017a1573809d20feef917014` declares `DEF_FONT(cmmib10, fonts/base/cmmib10.ttf, 0)`, xHeight=.444445 and quad=1.149994, exactly 128 metric rows, 166 kern rows and 3 next-larger links; style graph is roman=cmbx10, ss=cmssbx10, tt=cmtt10, it=cmbxti10.

`src/res/font/cmmib10_unchanged.def.cpp@80ca98effe023687a31d361fb9cbe170286fc53f` intentionally reuses the same TTF with 67 metrics, 37 kern rows and the same 3 next-larger links. `builtin_font_reg.cpp@c92411673b6dfd6b7bd550a20747c5b7eba545bc` registers both definitions; `FontInfo::getFont()` lazily calls `Font::create`.

## Accepted runtime, install and packaging reachability
`res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` embeds `fonts/base/cmmib10.ttf`. `LaTeX::initBundled()` initializes the builtin set; Qt prepends `:/` when disk font is absent and loads through QFontDatabase. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` and calls `LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies the rcc into app Resources and `build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` requires it in release resources.

`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`, standalone Meson install, `src/res/font/meson.build@aa2abb92de7c5c029390060a16df10ff2843d244`, CMake `75059dec500cff8114728331eb5cb317abf6daf9`, and legacy `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` agree on source/install/runtime reachability.

## License and existing-owner-first
MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts/XML under `res` from the project MIT blanket. `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` permits copy/distribution/modification provided modified software carries a different name. The c154 TDesktop namespace therefore matters to license provenance and collision avoidance. OS/2 fsType is not legal-license authority.

Existing-owner-first selects existing TDRP recursive source/resource/font/rendering/license/install/packaging/provenance governance. `novel_capability=false`; no new MicroTeX/Telegram font registry, Resource owner, font store, packaging root or formula renderer is introduced.

## Coverage and parent exact-head evidence
Before strict244: total=16,379 unknown=16,133 unread=16,108 omitted=0 responsibilities=261. After strict244: total=16,379 unknown=16,132 unread=16,107 omitted=0 responsibilities=262. Baseline identity remains unaccepted, source closure remains open, independent release acceptance remains not accepted.

Parent `128eadfe2836d61e513c42e93d07e6ff1dcc190f`: TDRP Revision 9 run `40221044843` is 30/30 success, including source authority `121252480870` and recursive accounting `121253109996`. Authority artifact `12609725213` digest `sha256:6664b4cf159eb68588a9c7bdbf8a733c9e8db66767ec2e13030ca72b02f3ecc6`; closure `12608733993` digest `sha256:6a4da0cbd975f096dabf754427ebdddb2a2f855d8673915cf432f14f4886c3d3`; reachability `12608868996` digest `sha256:a4d71f646380553b92b53ef6691f22f953d9dc0529fe0588af73d0fac65ab55d`; boost `12609740519` digest `sha256:7b014cac53c12f8d655695e2da695f27dbe62937b1381c2e42ee7653516df358`. Rust run `40221044813` is 6/6 success including rust-host `121252481106`; artifact `12609387547` digest `sha256:bb1a624abf2a4e2cf2c353eda30cfeec8c784580e6b8e7fd68fee568eb7d333b`. Official MCP `40221044851` is 1/1 success, contracts `121252480687`.

Next deterministic entry: `res/fonts/euler/eufb10.ttf@935f84bd27c682b1ed2ed6e79c89c139d55943b9`.

## Traceability anchors
- TDRP_MICROTEX_RES_FONTS_BASE_CMMIB10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-SFNT-NAME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-COMPILED-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-KERNS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-NEXT-LARGER-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-BUILTIN-RUNTIME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-24484-15-134-133-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-C154-NAMESPACE-HEAD-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-128-166-3-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-UNCHANGED-67-37-3-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-BUNDLED-PRODUCTION-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-KNUTH-NAME-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-SFNT-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-COMPILED-BOLD-MATH-ITALIC-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-RUNTIME-BUNDLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-LICENSE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-NAME-ATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-METRIC-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-KERN-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-LARGER-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-UNCHANGED-VARIANT-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-BUNDLE-OMISSION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMMIB10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
