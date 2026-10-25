# desktop-app/MicroTeX res/cyrillic/wntt10.ttf complete read - strict240

## Authority and exact-path history
PR #154 pre-write lease is 7ff4a99f4be96e2f82035dd2da2c8106e237fd54. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; Telegram/ThirdParty/MicroTeX pins desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. GitHub resolves the public repository lineage through NanoMichael/MicroTeX while preserving the pinned commit identity.

Target res/cyrillic/wntt10.ttf is pinned blob 649e9b1b24b3805646ad7cee71ae6e8d6e73e2c9, 33,424 decoded bytes. Root 17f3fc7816e4de1f5764036f34d759d50098ae58 and c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e resolve to cb01bb52e08d3b44502eefadd96077622e55da37, 33,308 bytes. c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc resolve to 649e9b1b24b3805646ad7cee71ae6e8d6e73e2c9. c154 is the font-namespace change ("Attempt to force our fonts."); root/c154-parent and c154/pinned are each byte-identical pairs.

## sfnt and c154 namespace semantics
Pinned WNTT10 is TrueType sfnt 0x00010000 with 15 tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, gasp, glyf, head, hhea, hmtx, loca, maxp, name and post. maxp reports 99 glyphs. head has unitsPerEm=1000, bounds -11,-228..527,838 and short loca. hhea has ascender=838, descender=-228, lineGap=90 and numberOfHMetrics=5. OS/2 v3 has weight=400, width=5, fsSelection=64, firstChar=0, lastChar=1141, typoAsc=800, typoDesc=-200, winAsc=838 and winDesc=228.

Root family/full/PostScript names are wntt10. c154/pinned family/full become "TDesktop wntt10" and PostScript becomes "TDesktop-wntt10" while style remains Regular. Independent root-to-c154 table comparison finds only the `name` table and `head` checksum-related bytes changed; cmap/glyf/OS2/hhea/maxp semantics remain stable. This is collision-isolation provenance, not a new Fabushi font runtime.

## Adjacent XML/style graph and historical behavior
wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2 is 7,679 bytes / 163 lines. Its Font root binds name=wntt10.ttf, id=wntt10, space=0.524996, xHeight=0.430555, quad=1.049991, unicode=95, romanVersion=wnr10 and ssVersion=wnss10. It contains 95 Char records and 36 Lig relations; unlike the proportional WNTI10 XML, no Kern records are declared.

Historical FontInfo lazily calls Font::create(_path, Formula::PIXELS_PER_POINT), so the XML path historically bound to this adjacent TTF. language_cyrillic.xml includes the Cyrillic font descriptions. The style graph keeps WNTT10 as the typewriter variant selected by other Cyrillic font descriptions where ttVersion=wntt10.

## Accepted runtime boundary
src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891 identifies cyrillic/language_cyrillic.xml. src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b removes the tinyxml2-backed description parser: addTeXFontDescription throws and addAlphabet fences ex_alphabet_registration. Accepted production therefore does not create WNTT10 XML FontInfo state.

src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 creates default FontInfos("SansSerif","Serif") when Formula::getExternalFont has no block mapping. That generic fallback is not WNTT10 loading and must not be represented as source-font execution.

## Install, shipping, license and ownership
res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 and legacy latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270 include/install the Cyrillic font resources. Accepted res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa explicitly excludes Cyrillic/Greek. tdesktop initializes/packages external_microtex_bundled.rcc through iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2, Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3, so the accepted bundle omits WNTT10.

README.md@824af04fd8607532efed3f73752b81e5093577ef excludes res fonts/XML from the MIT blanket; res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 is GPLv3 provenance. OS/2 embedding metadata is not legal-license authority. Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance; novel_capability=false.

## Coverage and exact-head evidence
Before strict240: total=16,379 unknown=16,137 unread=16,112 omitted=0 responsibilities=257.
After strict240: total=16,379 unknown=16,136 unread=16,111 omitted=0 responsibilities=258.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Parent 7ff4a99f4be96e2f82035dd2da2c8106e237fd54 exact-head evidence is terminal-success: TDRP Revision 9 run 40215710329 is 30/30 success, including Source authority / recursive inventory 121236271657 and Recursive root authority accounting gate 121236460922. Authority artifact 12607885727 digest sha256:4373a0dd3947decec2703630d2174158f222230ccc3e9a940955b42fb5f64d59; recursive closure 12607741584 digest sha256:42b22158c10ee4a2d92620845cd8a4bdc4b89ab94ed22605ebe254b11fe12028; recursive reachability 12607986089 digest sha256:ca7a91822c51277b571322f8bd9fa61b72bbfe00f5737fdbda8303f5afef4d62; boost reachability 12607826047 digest sha256:ffdacf569ad9e5a455a0b521172e04fe372b38756e6e379d8960da6516367001.

Rust desktop runtime 40215710305 is 6/6 success including rust-host 121236271644; messaging artifact 12607481643 digest sha256:ea1920ed93632195a7536c89df31a09f1e4d43b42d0dd2db3ac3a41128e51336. Official MCP marketplace 40215710357 is 1/1 success.

Next deterministic non-directory entry: res/cyrillic/wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2 (7,679 bytes / 163 lines).

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNTT10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-SFNT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-NAME-HEAD-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-OS2-GLYPH-LAYOUT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-XML-STYLE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-INSTALL-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-33424-15-99-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-ROOT-33308-C154-33424-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-NAMESPACE-REWRITE-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-GLYPH-LAYOUT-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-XML-95-36-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-SFNT-NAMESPACE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-XML-STYLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-SHIPPING-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-NAMESPACE-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-GLYPH-LAYOUT-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-XML-STYLE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
