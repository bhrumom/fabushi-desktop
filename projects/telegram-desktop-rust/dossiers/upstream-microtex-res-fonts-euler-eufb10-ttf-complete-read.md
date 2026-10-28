# desktop-app/MicroTeX res/fonts/euler/eufb10.ttf complete read - strict245

## Authority and exact-path history
PR #154 pre-write lineage is `e40549e8cdd408aba2e8c58870f657367c2938d5` after the strict244 receipt invariant repair. Accepted tdesktop is `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; `Telegram/ThirdParty/MicroTeX` pins `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Target `res/fonts/euler/eufb10.ttf` is blob `935f84bd27c682b1ed2ed6e79c89c139d55943b9`, exactly 23,860 bytes. Root `17f3fc7816e4de1f5764036f34d759d50098ae58` and c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` resolve to `0c09926d99666ee4c2b48401f3e266d635bc1bd3` (23,744 bytes); c154 `c1546216f32d398dca90e1dc33df34842683ec42` and pinned resolve to the accepted blob. Exact-path history contains only the root init and c154 `Attempt to force our fonts` change.

## sfnt identity and c154 namespace rewrite
Independent binary parsing finds 17 tables, 97 glyphs, unitsPerEm=2048, short loca, bounds `-51,-457..2537,1516`, hhea `1487/-473/0`, advanceWidthMax=2537, 97 long metrics, OS/2 v3 weight=400 width=5 fsType=0, and format-4 cmap coverage of 97 codepoints. Pinned family/full is `TDesktop eufb10`, style `Regular`, PostScript=`TDesktop-eufb10`; parent used `eufb10`.

Per-table comparison shows only `name` and `head` differ. The other 15 tables are byte-identical. `head` differs only at checksumAdjustment bytes 8..11 and the low modified-timestamp bytes 32..35; glyph/layout/cmap/OS2/hhea/maxp semantics are unchanged. The c154 namespace therefore preserves binary rendering semantics while avoiding collision with the original reserved family name.

## Compiled Euler Fraktur bold semantics
`src/res/font/eufb10.def.cpp@10bd002b718c552814abe57e45da03f9549ada88` declares `DEF_FONT(eufb10, fonts/euler/eufb10.ttf, 0)`, xHeight=.475342, quad=1.0 and space=.333333, with exactly 92 metric rows and no compiled kern or next-larger table.

The bold relation is owned by the companion `src/res/font/eufm10.def.cpp@d16c1c5bc3c1ccdddf868d345316fa817092d5d1`, which declares `bold(eufb10)`. `src/res/builtin/tex_param.res.cpp@c7b5b993d377db79a0e67dca4369f695fd82790c` maps the `mathfrak` text style to eufm10; `src/core/macro_impl.cpp@13343176be7cccdc44e2e4275a754759a6c4e4c8` normalizes `frak` to `mathfrak`. Therefore eufb10 is the compiled bold Euler Fraktur continuation of the canonical mathfrak path, not an independent font-selection root.

`src/res/reg/builtin_font_reg.cpp@c92411673b6dfd6b7bd550a20747c5b7eba545bc` registers both eufm10 and eufb10. `src/fonts/font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8` lazily calls `Font::create` for the selected path.

## Accepted runtime, install and packaging reachability
`res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` embeds `fonts/euler/eufb10.ttf`; legacy `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` embeds both Euler TTFs. `RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`, `src/res/font/meson.build@aa2abb92de7c5c029390060a16df10ff2843d244`, and CMake `75059dec500cff8114728331eb5cb317abf6daf9` preserve source/install/build reachability.

`FontInfo::getFont()` lazily binds the path, and Qt `graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5` prepends `:/` when the disk path is absent and registers it through QFontDatabase. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` and calls `LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies the rcc into app Resources and `build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` requires it in release resources.

## License and existing-owner-first
MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts/XML under `res` from the project MIT blanket. `res/fonts/licences/OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52` identifies `eufb10` and `eufm10` as Reserved Font Names under SIL Open Font License 1.1 and forbids a Modified Version from using those reserved primary names without permission. c154's primary family `TDesktop eufb10` is therefore a license-provenance responsibility as well as collision isolation. OS/2 fsType is not legal-license authority.

Existing-owner-first selects existing TDRP recursive source/resource/font/rendering/license/install/packaging/provenance governance. Current Fabushi code search finds no MicroTeX/eufb10 source-specific runtime to preserve or extend. `novel_capability=false`; no Telegram/MicroTeX font registry, second Resource owner, font store, packaging root or formula renderer is introduced.

## Coverage and parent exact-head evidence
Before strict245: total=16,379 unknown=16,132 unread=16,107 omitted=0 responsibilities=262. After strict245: total=16,379 unknown=16,131 unread=16,106 omitted=0 responsibilities=263. Baseline identity remains unaccepted, source closure remains open, independent release acceptance remains not accepted.

- Parent exact-head TDRP Revision 9 run 40223320224: 61/61 success; source authority 121259384742 and recursive accounting 121259607337 success.
- Authority artifact 12610083306 sha256:5579f0ce38086dfd2c5d96c3ac002342588716ba408f766dfb54ee1314d0ad10; closure 12609854180 sha256:a225cb97b437112c3c6c5259397e46f2149d5e7c88cc2e142f90aae06e68b5f4; reachability 12610143741 sha256:92c41352325dd87463779f4cd7a8c61fc9aeb6c906a3fba99c4eba942b3d9a2d; boost 12610706013 sha256:457dff07a14d8d5a5368e510ef54a638d8b3fc97b06de3287cb6d132cb9e9b7c.
- Parent exact-head Rust desktop runtime 40223320234: 6/6 success; rust-host 121259569461 success; artifact 12610198039 sha256:7a4d0349e9506a07056b1cb29b44574201a8e597124d4bb7c633240e804d767b.
- Official MCP marketplace 40223320279: 1/1 success.

Next deterministic entry: `res/fonts/euler/eufm10.ttf@93c3452962c71ac7364c6184a58b2ef06b337c79`.

## Traceability anchors
- TDRP_MICROTEX_RES_FONTS_EULER_EUFB10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-SFNT-NAME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-COMPILED-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-FRAKTUR-BOLD-LINK-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-BUILTIN-RUNTIME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-23860-17-97-97-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-C154-NAMESPACE-HEAD-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-92-0-0-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-EUFM-BOLD-EUFB10-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-BUNDLED-PRODUCTION-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-OFL-RESERVED-NAME-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-SFNT-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-COMPILED-EULER-BOLD-FRAKTUR-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-FRAKTUR-BOLD-LINK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-RUNTIME-BUNDLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-LICENSE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-NAME-ATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-METRIC-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-BOLD-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-BUNDLE-OMISSION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-EULER-EUFB10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
