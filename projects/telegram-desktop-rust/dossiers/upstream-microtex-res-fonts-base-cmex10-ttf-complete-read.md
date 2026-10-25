# desktop-app/MicroTeX res/fonts/base/cmex10.ttf complete read - strict242

## Authority and exact-path history
PR #154 pre-write lease is `804c3b78a465246e1a63a02641cff1e9a5b2dfaa`. Accepted tdesktop is `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; `Telegram/ThirdParty/MicroTeX` pins `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Target `res/fonts/base/cmex10.ttf` is blob `4df1d6c83b039817d1d97d0ee794e5c0fb0124f2`, exactly 19,356 bytes. Root `17f3fc7816e4de1f5764036f34d759d50098ae58` and c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` resolve the path to blob `5ac93ac4fbb46754769bd065c304db38978840e4` (19,240 bytes); c154 `c1546216f32d398dca90e1dc33df34842683ec42` and pinned resolve to the accepted blob. The exact-path change is therefore the c154 font-namespace commit boundary.

## sfnt identity and the c154 namespace rewrite
A binary sfnt read of the pinned file finds 17 tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, cvt, fpgm, glyf, head, hhea, hmtx, loca, maxp, name, post and prep. It has 134 glyphs, unitsPerEm=2048, short loca, bounds `-49,-6062..2978,1581`, hhea ascent/descent/lineGap `1581/-6082/0` with 134 long metrics, and OS/2 v4 weight=400, width=5, fsType=0, typo `1581/-6062/0`. The Windows Unicode format-4 cmap covers 133 codepoints from 0 through 8729. There is no legacy `kern` table; tiny GPOS/GSUB tables are present.

The pinned name records are family/full `TDesktop cmex10`, style `Regular`, PostScript `TDesktop-cmex10`; the root/c154-parent font used `cmex10`. An independent per-table byte comparison shows that c154 changes only the `name` table (615 -> 732 bytes) plus the expected `head` checksum adjustment. The other 15 sfnt tables are byte-identical, so glyph/cmap/layout/metric semantics are not silently changed by the namespace rewrite.

## Compiled metric, delimiter and growth semantics
Pinned `src/res/font/cmex10.def.cpp@c578f824c9799bdfe9a15625e082d7eaa31251db` is the metric authority; there is no `res/fonts/base/cmex10.xml`. It declares `DEF_FONT(cmex10, fonts/base/cmex10.ttf, 0)`, xHeight `0.430555`, quad `1.000003`, exactly 128 metric rows, 28 extension recipes and 76 next-larger links. These extension and next-larger tables implement TeX delimiter construction/growth; the TTF therefore carries an active math-rendering responsibility, not merely decorative asset provenance.

Pinned `src/res/reg/builtin_font_reg.cpp@c92411673b6dfd6b7bd550a20747c5b7eba545bc` registers `REG_FONT(cmex10)` in `FontSetBuiltin`, with the matching declaration in `builtin_font_reg.h@ae07cde4c55433e744517fa0ae278ec0b571f143`. `DefaultTeXFont` registers that builtin set, and `FontInfo::getFont()` lazily resolves the TTF path through `Font::create`.

## Accepted runtime, install and packaging reachability
`res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly embeds `fonts/base/cmex10.ttf` among the compiled-in font resources. This is the opposite of the prior Cyrillic XML-loader boundary: CMEX10 is an accepted bundled production font. `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` initializes the packaged `external_microtex_bundled.rcc` and calls `tex::LaTeX::initBundled()` before formula measurement/rendering. On macOS, `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies that rcc into app Resources and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` rejects a release bundle missing it.

`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists the font, standalone Meson installs `res/fonts`, and legacy `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` includes it. Install inclusion, bundled resource inclusion and tdesktop release packaging all agree on reachability.

## License, naming and existing-owner-first
MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts/XML under `res` from the project MIT blanket. `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8` permits copying/distribution/modification provided modified software carries a different name. The c154 `TDesktop cmex10` / `TDesktop-cmex10` namespace change therefore matters to license provenance as well as collision avoidance. OS/2 metadata is not treated as the legal license authority.

Existing-owner-first selects the existing TDRP recursive source/resource/font/rendering/license/install/packaging/provenance governance, with Fabushi renderer runtime-assets and desktop packaging as product contexts. `novel_capability=false`; no MicroTeX/Telegram font registry, second Resource owner, second font store, second license store, second packaging root or second formula renderer is introduced.

## Coverage and parent exact-head evidence
Before strict242: total=16,379 unknown=16,135 unread=16,110 omitted=0 responsibilities=259. After strict242: total=16,379 unknown=16,134 unread=16,109 omitted=0 responsibilities=260. `inventory/index.json` still carried the pre-strict241 numeric minimum (16,136/16,111) despite its strict241 note and canonical ledger/lock already being 16,135/16,110; strict242 reconciles that stale index field directly to the canonical 16,134/16,109 state rather than double-counting a source entry. Baseline identity remains unaccepted, source closure remains open, and independent release acceptance remains not accepted.

Parent `804c3b78a465246e1a63a02641cff1e9a5b2dfaa` TDRP Revision 9 run `40218118036` is 30/30 terminal-success, including Source authority / recursive inventory `121243587166` and Recursive root authority accounting gate `121243794861`. Authority artifact `12608124578` digest `sha256:6a99dfb5879259449f5277447d9e149ed6826505804ad101afc8d462e7440ba0`; recursive closure `12608420808` digest `sha256:469380f6c9225101762ad512ba15fad9ef4bf48121a2cc3ba3e3ddb6c8897771`; recursive reachability `12608840536` digest `sha256:3db3f322f52d864cf9c7ddfcc13315809c0cbda23dcd991f351aebaa143755c7`; boost reachability `12608920268` digest `sha256:921a2c14a2baad8c13f800cfdffc85ac970e64fa09e8649c053808480abc9c8c`. Rust desktop runtime run `40218118026` is 6/6 terminal-success including rust-host `121243587386`; messaging artifact `12608189021` digest `sha256:a1bbf45a4aa497f425101484791c76e6b2a41f1b5f0aaf00a2ed3c704ab1ae34`. Official MCP marketplace run `40218118035` is 1/1 terminal-success; contracts job `121243586924` succeeded.

Next deterministic non-directory entry is `res/fonts/base/cmmi10.ttf@c12851691ef7a5424f83752e7d39737af9860c79`.

## Traceability anchors
- TDRP_MICROTEX_RES_FONTS_BASE_CMEX10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-SFNT-NAME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-COMPILED-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-EXTENSIONS-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-NEXT-LARGER-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-BUILTIN-RUNTIME-001
- ORA-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-19356-17-134-133-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-C154-NAME-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-128-28-76-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-BUNDLED-PRODUCTION-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-KNUTH-NAME-001
- INV-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-SFNT-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-COMPILED-DELIMITER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-RUNTIME-BUNDLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-LICENSE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-NAME-ATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-METRIC-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-EXTENSION-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-LARGER-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-BUNDLE-OMISSION-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-FONTS-BASE-CMEX10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
