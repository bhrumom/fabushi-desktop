# desktop-app/MicroTeX res/cyrillic/wnss10.ttf complete read - strict232

## Authority
- PR #154 pre-write lease: 3589fba407360791dae7b34380a10aac7eff810a.
- Live main at audit start: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned MicroTeX: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f, tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Target: res/cyrillic/wnss10.ttf@6cc01cfd7d25ab5db44f01466bb0c5f54e9a144e, 19,856 bytes.
- Root 17f3fc7816e4de1f5764036f34d759d50098ae58 resolves to dfe3d8d59505a90447cb26ab3a4dda699f078102, 19,740 bytes.
- c1546216f32d398dca90e1dc33df34842683ec42 resolves to the pinned target blob; its parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e still resolves to the root blob.
- Exact-path history contains exactly root init and c154 Attempt to force our fonts.
- Pre-write independent responsibility row count: 0.

## sfnt / name / head / OS2 / glyph-layout
Pinned WNSS10 has 15 sfnt tables, 98 glyphs and unitsPerEm=1000. head bounds are (-61,-205)..(1069,903), macStyle=0, lowestRecPPEM=8, indexToLocFormat=0. OS/2 v3 has xAvgCharWidth=621, weight=400, width=5, fsType=0, fsSelection=64, lastChar=1141, typo 800/-200 with gap 90, and Win 903/205. hhea has ascender=903, descender=-205, lineGap=90, advanceWidthMax=1111 and 98 metrics. post has italicAngle=0 and isFixedPitch=0.

Current name records expose family/full TDesktop wnss10, subfamily Regular, unique TDesktop-wnss10:Version 1.001, and PostScript TDesktop-wnss10; there are no name IDs 13/14. Root names are unprefixed wnss10 variants.

Root/current comparison changes name from 606 to 723 bytes and changes head checksum/checkSumAdjustment. GDEF, GPOS, GSUB, OS/2, cmap, glyf, hhea, hmtx, loca, maxp, and post keep identical checksums and lengths. Later-table offsets move only because name is longer. The c154 graphic_qt.cpp patch states that same-named system fonts could shadow bundled fonts and that bundled families are prefixed to force the intended application font.

## XML style/runtime semantics
Adjacent wnss10.xml@660c29af27ddab8188fcc7594bae0941cc45d9f6 is root/c154/pinned byte-identical and exact-path history is root init only. Its Font root is name=wnss10.ttf, id=wnss10, space=0.333334, xHeight=0.444445, quad=1.000003, unicode=95, romanVersion=wnr10, ttVersion=wntt10, boldVersion=wnssbx10, itVersion=wnssi10.

It contains 95 Char records / 95 unique codes spanning 171..1141, 406 Kern relations (-0.027779 x274, -0.083334 x105, -0.055555 x26, +0.027779 x1), and 36 Lig relations. Width spans 0.23889..1.111117, height 0.438889..0.902727, depth 0..0.194445 and italic correction 0..0.019444; 13 records have nonzero depth and 7 have nonzero italic correction.

The reciprocal style graph is consistent: wnr10 exposes ssVersion=wnss10; wnti10 exposes ssVersion=wnssi10; wnssi10 points to roman wnti10, tt wntt10 and bold wnssbx10; wnssbx10 points to roman wnbx10 and tt wntt10. language_cyrillic.xml includes all eight Cyrillic Metrics descriptions. Historical root fonts/font_basic.cpp materializes character metrics plus Kern/Lig and style IDs and lazily calls Font::create(path). Pinned src/fonts/font_info.cpp retains bold/roman/ss/tt/it IDs and lazy Font::create.

## Accepted runtime boundary
Pinned alphabet.cpp still identifies Cyrillic through cyrillic/language_cyrillic.xml, but fonts.cpp makes addTeXFontDescription throw ex_alphabet_registration and its caller fences the failure. WNSS10 therefore does not become an accepted-production XML FontInfo. parser.cpp emits TextRenderingAtom only when Formula::getExternalFont(block) returns an explicit mapping. This is optional fallback, not guaranteed WNSS10 rendering and not permission to revive the stripped XML loader.

## Install, shipping and license
RES_README lists cyrillic/wnss10.ttf; meson.build installs res/cyrillic; legacy/general latex.qrc lists both WNSS10 TTF and XML. Accepted shipping is different: res/bundled.qrc explicitly excludes Cyrillic and Greek. Accepted tdesktop iv_markdown_microtex.cpp registers external_microtex_bundled.rcc on macOS and calls tex::LaTeX::initBundled(); Telegram/CMakeLists.txt and Telegram/build/build_mac.py copy/check that accepted resource. Standalone inclusion is provenance only; accepted production excludes WNSS10.

MicroTeX README excludes fonts and XML resources under res from the project MIT blanket; res/cyrillic/LICENSE is GPLv3 provenance. OS/2.fsType=0 is embedding metadata, not license authority.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNSS10 runtime or shipping asset. No second Resource owner, font registry, font store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict232: total=16,379, unknown=16,145, unread=16,120, omitted=0, responsibilities=249.
After strict232: total=16,379, unknown=16,144, unread=16,119, omitted=0, responsibilities=250.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.
Next deterministic entry: res/cyrillic/wnss10.xml@660c29af27ddab8188fcc7594bae0941cc45d9f6.

## Parent exact-head evidence
Lease 3589fba407360791dae7b34380a10aac7eff810a: TDRP Revision 9 run 40196802710 completed/success, 61/61; Rust desktop runtime 40196802696 completed/success, 6/6; Official MCP marketplace 40196802784 completed/success, 1/1. Authority artifact 12600287617 digest sha256:7bc7234f120128b9c4e20c393f577194ebc35a97325961a3eb4d35829397008a; recursive closure 12599848092 digest sha256:9459bdbe742afe3b31b92e0477f4b967de7d2d9a8d28698f804eb4510952e4f3; recursive reachability 12600562561 digest sha256:ea3be94cfcc8823822c49d3b313da299c8f39d5efaba4d1ef59e67298630cc36; boost reachability 12600502423 digest sha256:6f5f2924e7e18fe8ecd2ea1c0ae1fe824574a772ca8f1d5f6f34d1e169ae5406; Rust messaging artifact 12600671733 digest sha256:1e661f492b1355e9b57b688cb84f76c84c300c6dbf5afd69e915a6a6365fb1cc. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNSS10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SFNT-METADATA-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-NAME-HEAD-REWRITE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-GLYPH-LAYOUT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-XML-METRIC-STYLE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-19856-98-1000-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-ROOT-19740-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-TDESKTOP-NAME-PREFIX-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-GLYPH-LAYOUT-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-FSTYPE-NOT-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-HISTORY-REWRITE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SFNT-XML-BINDING-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-SFNT-METADATA-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-NAME-PREFIX-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-GLYPH-MUTATION-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-FSTYPE-AS-LICENSE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
