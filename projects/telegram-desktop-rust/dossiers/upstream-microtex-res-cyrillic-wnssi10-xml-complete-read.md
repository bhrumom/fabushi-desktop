# desktop-app/MicroTeX res/cyrillic/wnssi10.xml complete read - strict237

## Authority and exact identity
PR #154 pre-write lease is 1edd236d3fb6c08f3afeacdbad34f7924d8a8f80. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; Telegram/ThirdParty/MicroTeX pins desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. Target res/cyrillic/wnssi10.xml is blob 5c2318c66e402f47c05ffb67015bcff0780523a8, 26,762 bytes / 606 lines.

Root 17f3fc7816e4de1f5764036f34d759d50098ae58, c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e, c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc all resolve to the same blob and identical bytes. Exact-path history returns only root init; c154 changed the adjacent wnssi10.ttf family/PostScript namespace, not this XML.

## XML font metrics and shaping surface
Font root: name=wnssi10.ttf, id=wnssi10, space=0.333334, xHeight=0.444445, quad=1.000003, unicode=95, romanVersion=wnti10, ttVersion=wntt10, boldVersion=wnssbx10.

Exactly 95 Char records / 95 unique codes span 171..1141. Width 0.23889..1.111117; height 0.438889..0.902727; depth 0..0.194445 with 13 nonzero; italic 0..0.161496 with 93 nonzero.

Exactly 406 Kern relations: -0.027779 x274, -0.083334 x105, -0.055555 x26, +0.027779 x1. Exactly 36 Lig relations. These are executable TeX metric/shaping semantics.

## Style graph, TTF binding and historical behavior
language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8 includes wnssi10.xml. Target style edges are roman=wnti10, tt=wntt10, bold=wnssbx10. wnss10.xml@660c29af27ddab8188fcc7594bae0941cc45d9f6 has itVersion=wnssi10; wnti10.xml@55046e15cbd13b5ceaed2ea6955cd378f6167599 has ssVersion=wnssi10. wnssbx10.xml@9fefc1c78c8b4bb1406173b8ac491c11f030a076 and wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2 provide the bold sans-serif and shared typewriter variants.

Pinned src/fonts/font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8 preserves style IDs and lazily calls Font::create(_path, Formula::PIXELS_PER_POINT). The XML path stays wnssi10.ttf. c154 namespaces that TTF internally to TDesktop wnssi10 / TDesktop-wnssi10 while leaving this XML unchanged.

## Accepted runtime boundary
src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891 identifies cyrillic/language_cyrillic.xml, but src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b makes addTeXFontDescription throw ex_alphabet_registration and addAlphabet fences it. WNSSI10 XML therefore does not create an accepted-production FontInfo.

src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c may request Formula::getExternalFont for non-Latin text. src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 creates default FontInfos("SansSerif","Serif") when a block has no mapping. That fallback does not parse WNSSI10 XML metrics.

## Install, shipping, license and ownership
res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee lists wnssi10.xml/ttf. meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/cyrillic. Legacy latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270 includes both.

Accepted res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa excludes Cyrillic/Greek because the XML parser path is stripped. tdesktop initializes external_microtex_bundled.rcc via iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2; CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies it into macOS Resources and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 requires it.

README.md@824af04fd8607532efed3f73752b81e5093577ef excludes fonts/XML under res from the project MIT blanket; res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 is GPLv3 provenance.

Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. novel_capability=false.

## Parent exact-head reconciliation and repair
Parent 1edd236d3fb6c08f3afeacdbad34f7924d8a8f80 TDRP Revision 9 run 40211520354 deterministically failed because strict236 declared target symbol TDRP_MICROTEX_RES_CYRILLIC_WNSSI10_TTF_PROVENANCE_001 but its dossier omitted that marker. This receipt repairs the strict236 dossier with its synthetic target symbol and full traceability anchors; no production code, validator, RTM, workflow or source baseline changes.

At last pre-write read, Rust desktop runtime 40211520333 had renderer/platform-worker and all three pressure-profiler jobs success while rust-host 121223566828 remained in_progress. Last terminal-success parent evidence remains b13adb5f: TDRP 40208877191 61/61, Rust 40208877250 6/6, MCP 40208877193 1/1.

## Coverage
Before strict237: total=16,379 unknown=16,140 unread=16,115 omitted=0 responsibilities=254.
After strict237: total=16,379 unknown=16,139 unread=16,114 omitted=0 responsibilities=255.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Next deterministic non-directory entry: res/cyrillic/wnti10.ttf@1dd164b2a3602494a1418eb16aca22d298b01ebd.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNSSI10_XML_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-FONT-CONSTANTS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-KERN-LIG-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-STYLE-VERSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-26762-606-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-95-CHAR-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-406-KERN-36-LIG-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-ROOT-C154-PINNED-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-TTF-PATH-BINDING-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-METRIC-TABLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-STYLE-VERSION-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-FONT-CONSTANT-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-METRIC-CARDINALITY-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-KERN-LIG-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-TTF-BINDING-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-LATEX-QRC-AS-SHIPPING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSI10-XML-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
