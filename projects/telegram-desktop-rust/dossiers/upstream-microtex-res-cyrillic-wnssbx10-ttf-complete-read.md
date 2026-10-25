# desktop-app/MicroTeX res/cyrillic/wnssbx10.ttf complete read - strict234

## Authority
- PR #154 pre-write lease: b2566fe85cc0e6389e5e85ece6f77eeebce39347.
- Live main at audit start: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned MicroTeX: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f, tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Target pinned blob: res/cyrillic/wnssbx10.ttf@125e05ccc6956bdfbc2b94a5398f6a7d482f0b08, 30,864 bytes.
- Root 17f3fc7816e4de1f5764036f34d759d50098ae58 and c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e resolve to d45380bc61be5669590da9ce036237b452a14c0f, 30,732 bytes.
- c1546216f32d398dca90e1dc33df34842683ec42 and pinned resolve to 125e05ccc6956bdfbc2b94a5398f6a7d482f0b08, 30,864 bytes.
- Exact-path history is not root/c154/pinned byte identity: c154 "Attempt to force our fonts." is the material binary rewrite point.
- Pre-write independent responsibility row count: 0.

## sfnt / name / head / OS2 / glyph-layout
Pinned WNSSBX10 has 15 sfnt tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, gasp, glyf, head, hhea, hmtx, loca, maxp, name, post. maxp reports 98 glyphs and head unitsPerEm=1000. head bounds are (-70,-205)..(1210,905), macStyle=0, lowestRecPPEM=8, indexToLocFormat=0. hhea is ascender=905, descender=-206, lineGap=90, advanceWidthMax=1250, numberOfHMetrics=98. OS/2 v3 reports xAvgCharWidth=683, weightClass=400, widthClass=5, fsType=0, fsSelection=64 and character range 0..1141. post is format 2.0, italicAngle=0, underlinePosition=-175, underlineThickness=50, non-fixed-pitch.

Root name records use family/full/PostScript "wnssbx10" and unique "wnssbx10:Version 1.001". Pinned/c154 use family/full "TDesktop wnssbx10", PostScript "TDesktop-wnssbx10" and unique "TDesktop-wnssbx10:Version 1.001"; version remains "Version 1.001 December 29, 2009".

The root->c154 binary delta is intentionally narrow: name/head checksum semantics change, while glyf, loca, hmtx, cmap, GDEF, GPOS, GSUB, OS/2, maxp, hhea and post preserve semantic checksums/lengths. c154's Qt patch explains the responsibility: a system font sharing the family could shadow a bundled font and make glyphs disappear, so bundled families were prefixed and unexpected family resolution is warned. This is namespace isolation, not a glyph/layout redesign.

## Adjacent wnssbx10.xml metrics and style graph
Pinned res/cyrillic/wnssbx10.xml@9fefc1c78c8b4bb1406173b8ac491c11f030a076 is 25,211 bytes / 606 lines and is byte-identical at root, c154 parent, c154 and pinned. Font root: name=wnssbx10.ttf, id=wnssbx10, space=0.366669, xHeight=0.458333, quad=1.100006, unicode=95, romanVersion=wnbx10, ttVersion=wntt10.

The XML contains 95 Char records / 95 unique codes 171..1141, 406 Kern and 36 Lig relations. Width spans 0.255557..1.250008, height 0.458333..0.902727, depth 0..0.194445, italic 0..0.02139; 13 nonzero-depth and 7 nonzero-italic records. Kern values are -0.030556 x274, -0.091667 x105, -0.061111 x26 and +0.030556 x1. Historical/pinned FontInfo retains roman/typewriter style IDs and lazily materializes the adjacent font through Font::create(path).

## Accepted runtime fallback boundary
Accepted src/fonts/fonts.cpp keeps addTeXFontDescription disabled by throwing ex_alphabet_registration; addAlphabet catches/fences that registration failure. WNSSBX10 XML therefore does not become an accepted-production FontInfo. Formula::getExternalFont creates default FontInfos("SansSerif", "Serif") for an absent non-Latin block; parser.cpp can request that generic fallback. It is not WNSSBX10 XML loading and does not prove this TTF ships or renders.

## Install, shipping and license
res/RES_README lists wnssbx10.ttf/xml. meson.build installs res/cyrillic. Top-level latex.qrc includes both target files; those are standalone/legacy inclusion provenance. Accepted res/bundled.qrc explicitly excludes Cyrillic and Greek. Accepted tdesktop registers/copies external_microtex_bundled.rcc and calls tex::LaTeX::initBundled(), so WNSSBX10 is not in the accepted production bundle. MicroTeX README excludes fonts/XML under res from the project MIT blanket; res/cyrillic/LICENSE is GPLv3 provenance. OS/2 fsType=0 is embedding metadata, not the legal license grant.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release remain contextual owners only. No second Resource owner, font registry, font-metric/style store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict234: total=16,379, unknown=16,143, unread=16,118, omitted=0, responsibilities=251.
After strict234: total=16,379, unknown=16,142, unread=16,117, omitted=0, responsibilities=252.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.
Next deterministic entry: res/cyrillic/wnssbx10.xml@9fefc1c78c8b4bb1406173b8ac491c11f030a076.

## Parent exact-head evidence
Lease b2566fe85cc0e6389e5e85ece6f77eeebce39347: TDRP Revision 9 run 40201112885 completed/success, 61/61; Rust desktop runtime 40201112856 completed/success, 6/6; Official MCP marketplace 40201112905 completed/success, 1/1. Authority artifact 12601629648 digest sha256:547d095268c4806c3e3fcdc567e1b610a97060dfd6a46bcadb7d2b36221ac95d; recursive closure 12601503472 digest sha256:63dcde3446f29ad741026f37bf439127a1ed38ff4bdba4f7d071f0b8fe7c5da5; recursive reachability 12601949650 digest sha256:232e0e39937c1779c392df91dd882012ed2884f0f15743a4c0fb239f7113a5c2; boost reachability 12602475024 digest sha256:44b537fde3553b4212b0149ad2d911235f1952d9baae3f6c615e59e09a91542e; Rust messaging artifact 12601898909 digest sha256:66a0aadb1c4ee9f7bbd5453fdb67f78e28420704850a1e3f09052abdb4bac171. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNSSBX10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-SFNT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-NAME-HEAD-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-OS2-GLYPH-LAYOUT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-XML-METRICS-STYLE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-INSTALL-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-30864-15-98-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-ROOT-30732-C154-30864-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-NAMESPACE-REWRITE-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-GLYPH-LAYOUT-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-XML-95-406-36-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-SFNT-NAMESPACE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-XML-STYLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-SHIPPING-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-TTF-EXACT-HEAD-EVIDENCE-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
