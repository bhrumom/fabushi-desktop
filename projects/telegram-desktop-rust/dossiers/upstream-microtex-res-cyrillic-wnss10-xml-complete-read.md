# desktop-app/MicroTeX res/cyrillic/wnss10.xml complete read - strict233

## Authority
- PR #154 pre-write lease: f7727fbe491dd6bffa4edb6ea3ff119d0745cf52.
- Live main at audit start: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned MicroTeX: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f, tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Target: res/cyrillic/wnss10.xml@660c29af27ddab8188fcc7594bae0941cc45d9f6, 25,262 bytes / 606 lines.
- Root 17f3fc7816e4de1f5764036f34d759d50098ae58, c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc all resolve to the exact same target blob and bytes.
- Exact-path history contains exactly root init; c154 does not touch this XML.
- Pre-write independent responsibility row count: 0.

## Font metrics and style graph
The Font root is name=wnss10.ttf, id=wnss10, space=0.333334, xHeight=0.444445, quad=1.000003, unicode=95, romanVersion=wnr10, ttVersion=wntt10, boldVersion=wnssbx10, itVersion=wnssi10.

The file contains 95 Char records / 95 unique codes spanning 171..1141, 406 Kern relations and 36 Lig relations. Width spans 0.23889..1.111117, height 0.438889..0.902727, depth 0..0.194445 and italic correction 0..0.019444. There are 13 nonzero-depth Char records and 7 nonzero-italic records. Kern values are -0.027779 x274, -0.083334 x105, -0.055555 x26 and +0.027779 x1.

The adjacent graph is consistent: wnr10 exposes ssVersion=wnss10; wntt10 exposes romanVersion=wnr10 and ssVersion=wnss10; wnssi10 points to roman wnti10, tt wntt10 and bold wnssbx10; wnssbx10 points to roman wnbx10 and tt wntt10. language_cyrillic.xml includes wnss10.xml among the eight Cyrillic Metrics descriptions.

## FontInfo / parser / runtime semantics
Historical root fonts/font_basic.cpp parses Kern and Lig children, materializes bold/roman/ss/tt/it style IDs, and lazily loads the adjacent font with Font::create(path). Pinned src/fonts/font_info.cpp preserves those style IDs and the same lazy Font::create(path) behavior.

Accepted runtime does not revive that XML loader. Pinned src/fonts/fonts.cpp makes addTeXFontDescription throw ex_alphabet_registration; addAlphabet records the attempted block and its caller catches the registration exception. Therefore wnss10.xml does not become an accepted-production FontInfo.

The accepted fallback needs a precise distinction: src/core/parser.cpp requests Formula::getExternalFont(block) for non-Latin text when normal symbol mappings do not apply, and pinned Formula::getExternalFont creates default FontInfos("SansSerif", "Serif") when a block has no entry. TextRenderingAtom can therefore render through that default external-font fallback. This fallback is not WNSS10 XML loading, does not consume wnss10.xml metrics, and does not make WNSS10 a guaranteed shipping font.

## Install, shipping and license
res/RES_README lists cyrillic/wnss10.xml. meson.build installs res/cyrillic, and legacy latex.qrc lists both wnss10.ttf and wnss10.xml. These are standalone/legacy inclusion provenance.

Accepted production differs: res/bundled.qrc explicitly excludes Cyrillic and Greek XML/font resources. Accepted tdesktop Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp registers external_microtex_bundled.rcc on macOS (or initializes the bundled resource elsewhere) and calls tex::LaTeX::initBundled(); Telegram/CMakeLists.txt and Telegram/build/build_mac.py copy/check that resource. WNSS10 XML is therefore not in the accepted production bundle.

MicroTeX README excludes fonts and XML resources under res from the project MIT blanket; res/cyrillic/LICENSE carries GPLv3 provenance.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release remain contextual owners only. No second Resource owner, font registry, font-metric/style store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict233: total=16,379, unknown=16,144, unread=16,119, omitted=0, responsibilities=250.
After strict233: total=16,379, unknown=16,143, unread=16,118, omitted=0, responsibilities=251.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.
Next deterministic entry: res/cyrillic/wnssbx10.ttf@125e05ccc6956bdfbc2b94a5398f6a7d482f0b08.

## Parent exact-head evidence
Lease f7727fbe491dd6bffa4edb6ea3ff119d0745cf52: TDRP Revision 9 run 40199119593 completed/success, 61/61; Rust desktop runtime 40199119589 completed/success, 6/6; Official MCP marketplace 40199119579 completed/success, 1/1.
Authority artifact 12601760503 digest sha256:e50a424310f8eba9856dc6f205a872ab3866ea270fe16045475ba41c07c3edf8; recursive closure 12601756477 digest sha256:4faed19a4d5adb0edc16aebd8b6cbb8ecc2d8e3a8b12d54dd32b6a53f5501f40; recursive reachability 12601057402 digest sha256:4ca167438ae4545b48d3d92418baae1642f9f79e508e3172fb5f3efa1d4a332d; boost reachability 12601480867 digest sha256:08048a65313481d4c0c46b009ac0ae1802a173a3f8e7618afe262d0f0be31ecc; Rust messaging artifact 12601805330 digest sha256:2a5487821c72969b17b39f6f44328abfb8d1b0a4e6c34f38da9519625af86cca.
Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNSS10_XML_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-FONT-CONSTANTS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-KERN-LIG-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-STYLE-VERSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-25262-606-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-95-CHAR-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-406-KERN-36-LIG-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-ROOT-C154-PINNED-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-TTF-PATH-BINDING-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-METRIC-TABLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-STYLE-VERSION-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-FONT-CONSTANT-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-METRIC-CARDINALITY-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-KERN-LIG-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-TTF-BINDING-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-LATEX-QRC-AS-SHIPPING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSS10-XML-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
