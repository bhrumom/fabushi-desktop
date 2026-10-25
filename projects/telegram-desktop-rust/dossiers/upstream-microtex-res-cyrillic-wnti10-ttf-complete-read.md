# desktop-app/MicroTeX res/cyrillic/wnti10.ttf complete read - strict238

## Authority and history
PR #154 pre-write lease is b0eb1da1f1180d184c7fcb8ab72496decde8e8f7. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; Telegram/ThirdParty/MicroTeX pins desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. GitHub currently resolves that public repository lineage through NanoMichael/MicroTeX while preserving the pinned commit identity. Target res/cyrillic/wnti10.ttf is blob 1dd164b2a3602494a1418eb16aca22d298b01ebd, 51,280 decoded bytes.

Root 17f3fc7816e4de1f5764036f34d759d50098ae58 and c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e resolve to eb5217f6c6b9d1a3a99fbbcf0840b8188bfb8a7a, 51,164 bytes. c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc resolve to 1dd164b2a3602494a1418eb16aca22d298b01ebd, 51,280 bytes. Pinned-lineage exact-path history contains exactly root init and c154 "Attempt to force our fonts."

## sfnt and c154 namespace semantics
Pinned WNTI10 is TrueType sfnt 0x00010000 with 15 tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, gasp, glyf, head, hhea, hmtx, loca, maxp, name and post. maxp has 98 glyphs.

head: unitsPerEm=1000, bounds -36,-206..1243,868, macStyle=0, lowestRecPPEM=8, short loca. hhea: ascent=868, descent=-207, lineGap=90, advanceWidthMax=1152, 98 long metrics. OS/2 v3: xAvg=654, weight=400, width=5, fsType=0, fsSelection=64, firstChar=0, lastChar=1141, typoAsc=800, typoDesc=-200, winAsc=868, winDesc=207. post is format 2.0, italicAngle=0, underlinePosition=-175, underlineThickness=50, isFixedPitch=0.

cmap maps 97 code points through 1141. glyf has 95 simple, zero composite and three empty glyphs. Root family/full/PostScript names are wnti10; c154/pinned family/full are "TDesktop wnti10" and PostScript is "TDesktop-wnti10". Style Regular and version "Version 1.001 December 29, 2009" are unchanged. Root-to-c154 table-directory changes are limited to the name table plus the head checksum adjustment; measured glyph/layout/OS2/cmap/hhea/maxp semantics are unchanged. This is bundled-font namespace/collision-isolation provenance, not a new Fabushi font runtime.

## Adjacent XML/style graph and historical behavior
wnti10.xml@55046e15cbd13b5ceaed2ea6955cd378f6167599 is 31,229 bytes / 708 lines. Its Font root binds name=wnti10.ttf, id=wnti10, space=0.357776, xHeight=0.430555, quad=1.022217, unicode=95, romanVersion=wnr10, boldVersion=wnbxti10, ssVersion=wnssi10 and ttVersion=wntt10.

It has exactly 95 Char records / 95 unique codes spanning 171..1141, width 0.306665..1.152216, height 0.430555..0.891615, 12 nonzero-depth records (max 0.194445) and 93 nonzero-italic records (max 0.183611). It has 512 Kern relations and 36 Lig relations. Kern distribution is -0.025556 x325, -0.076666 x128, -0.1022215 x35, -0.051111 x23 and +0.025556 x1.

Style reciprocity includes wnr10.itVersion=wnti10 and wnssi10.romanVersion=wnti10. language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8 explicitly includes wnti10.xml. Historical FontInfo lazily calls Font::create(_path, Formula::PIXELS_PER_POINT), so the XML path historically bound to this adjacent TTF.

## Accepted runtime boundary
src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891 identifies cyrillic/language_cyrillic.xml. src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b removes the tinyxml2-backed description parser: addTeXFontDescription throws and addAlphabet fences ex_alphabet_registration. Accepted production therefore does not create WNTI10 XML FontInfo state.

src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 creates default FontInfos("SansSerif","Serif") when Formula::getExternalFont has no block mapping. That generic fallback is not WNTI10 loading and must not be represented as source-font execution.

## Install, shipping, license and ownership
res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee lists wnti10.ttf and wnti10.xml. meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/cyrillic. Legacy latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270 includes both target TTF/XML.

Accepted res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa explicitly excludes Cyrillic/Greek because the XML parser path is stripped. tdesktop initializes external_microtex_bundled.rcc via iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies it into macOS Resources and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 requires it. The accepted bundle therefore omits WNTI10.

README.md@824af04fd8607532efed3f73752b81e5093577ef excludes fonts/XML under res from the project MIT blanket; res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 is GPLv3 provenance. OS/2 fsType=0 is embedding metadata, not legal-license authority.

Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. No second Resource owner, font registry/store, MicroTeX/Cyrillic runtime, license store or package root is introduced. novel_capability=false.

## Coverage and exact-head evidence
Before strict238: total=16,379 unknown=16,139 unread=16,114 omitted=0 responsibilities=255.
After strict238: total=16,379 unknown=16,138 unread=16,113 omitted=0 responsibilities=256.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Parent b0eb1da1f1180d184c7fcb8ab72496decde8e8f7 exact-head evidence is terminal-success: TDRP Revision 9 run 40212507141 is 30/30 success, including Source authority / recursive inventory 121226587203. Authority artifact 12606565351 digest sha256:644e9b4234bca60fffb9a5a4919a8f413cda54031777efd92ab2148818c10735; recursive closure 12606064691 digest sha256:f91c91a88c7bb75f74ba2fde01193264cad7821cfe77352cffb687602f021e96; recursive reachability 12605888380 digest sha256:e45001fd3438215c85a492d5db69384bb27e3f73dbed420629426c49816bd0ea; boost reachability 12606530752 digest sha256:5a8a1921f951b19e39da31d863d2a1661f7b82c72d1e5fa608fc483111a30b53.

Rust desktop runtime 40212507186 is 6/6 success including rust-host 121226587470; messaging artifact 12605779659 digest sha256:0470d1545aa0b8a278d82d29b823134d4205d0b0f279331c85dbd770d7b3e1f5. Official MCP marketplace 40212507152 is success.

Next deterministic non-directory entry: res/cyrillic/wnti10.xml@55046e15cbd13b5ceaed2ea6955cd378f6167599 (31,229 bytes / 708 lines).

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNTI10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-SFNT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-NAME-HEAD-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-OS2-GLYPH-LAYOUT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-XML-METRICS-STYLE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-INSTALL-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-51280-15-98-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-ROOT-51164-C154-51280-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-NAMESPACE-REWRITE-ONLY-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-GLYPH-LAYOUT-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-XML-95-512-36-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-SFNT-NAMESPACE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-XML-STYLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-SHIPPING-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-NAMESPACE-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-GLYPH-LAYOUT-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-XML-STYLE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-FSTYPE-AS-LICENSE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
