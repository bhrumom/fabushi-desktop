# desktop-app/MicroTeX res/cyrillic/wnr10.ttf complete read — strict230

## Authority
- PR #154 pre-write lease: 21eb85311a9833f7d3e9f8c798b6af0d74472f70.
- Accepted tdesktop a7f453612f4896ac04d0007a36ab0d377e5bcbcf; pinned MicroTeX 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f, tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2, truncated=false, 315 entries.
- Target a93586f1efd00c4009b72fc63a365661ce461435, 36,196 bytes; root b13a442c026d8520a0cb02ab755e297c484b19b1, 36,084 bytes.
- Exact-path history is root 17f3fc7816e4de1f5764036f34d759d50098ae58 plus c1546216f32d398dca90e1dc33df34842683ec42; c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e has root blob and c154 equals pinned.
- Pre-write independent responsibility row count: 0.

## sfnt / name / head / OS2 / glyph-layout
Pinned WNR10 has 15 sfnt tables, 98 glyphs and unitsPerEm=1000. head bounds (-41,-205)..(1152,868), macStyle=0, lowestRecPPEM=8, indexToLocFormat=0. OS/2 v3 xAvgCharWidth=658, weight=400, width=5, fsType=0, fsSelection=64, lastChar=1141, typo 800/-200, Win 868/205. hhea advanceWidthMax=1194 with 98 metrics; post italicAngle=0, isFixedPitch=0.
Current names: family/full TDesktop wnr10, subfamily Regular, unique TDesktop-wnr10:Version 1.001, PostScript TDesktop-wnr10; no name IDs 13/14. Root/current table-directory comparison changes only name (603->714 bytes) and head checksum/checksumAdjustment. GDEF/GPOS/GSUB/OS2/cmap/glyf/hhea/hmtx/loca/maxp/post remain checksum/length-identical, so c154 is namespace/checksum rewrite rather than glyph/layout/advance/cmap mutation. The c154 Qt patch attributes prefixing to same-named system-font shadowing.

## XML FontInfo/runtime semantics
Adjacent wnr10.xml@319d5db2d3ab2471d565e77e83dfdd29450fb106 is root/c154/pinned-identical. Root name=wnr10.ttf id=wnr10 space=0.333334 xHeight=0.430555 quad=1.000003 unicode=95 boldVersion=wnbx10 ssVersion=wnss10 ttVersion=wntt10 itVersion=wnti10. It has 95 Char/95 unique codes 171..1141, 416 Kern (-0.027779 x266, -0.083334 x117, -0.111112 x13, -0.055555 x19, +0.027779 x1), 36 Lig, 13 nonzero-depth and 19 nonzero-italic records.
language_cyrillic.xml includes it among eight Metrics; wnss10/wntt10/wnti10 point back to romanVersion=wnr10 and define the corresponding style graph. Historical parser semantics materialized metrics/Kern/Lig/style IDs into FontInfo, whose pinned getFont lazily calls Font::create(_path,...). The sfnt itself stays Regular/weight400/macStyle0/italicAngle0, so style semantics are XML-driven.

## Accepted boundary / install / license / owner
Accepted Cyrillic registration remains, but addTeXFontDescription throws ex_alphabet_registration and the wrapper fences it. parser.cpp only emits TextRenderingAtom when an explicit external-font mapping exists; fallback is optional, not guaranteed WNR10 rendering. RES_README, Meson and legacy latex.qrc include/install WNR10; accepted bundled.qrc explicitly excludes Cyrillic/Greek because the XML parser path is stripped. README excludes res fonts/XML from the MIT blanket; res/cyrillic/LICENSE is GPLv3 provenance. fsType=0 is embedding metadata, not license authority.
Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. No second Resource/font/license/package/runtime owner is introduced.

## Coverage
Before strict230: total=16,379 unknown=16,147 unread=16,122 omitted=0 responsibilities=247.
After strict230: total=16,379 unknown=16,146 unread=16,121 omitted=0 responsibilities=248. baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.
Next deterministic entry: res/cyrillic/wnr10.xml@319d5db2d3ab2471d565e77e83dfdd29450fb106.

## Parent exact-head evidence
Lease 21eb85311a9833f7d3e9f8c798b6af0d74472f70: TDRP 40191431059 61/61 success; Rust 40191431064 6/6 success; Official MCP 40191431065 1/1 success. Authority artifact 12597992808 sha256:89e85147d707646af0e1c95fdfbdbfb4801c318afc70110236217125e9b99d4f; recursive closure 12597888461 sha256:a8f8a2c46a673d388b9d1e05d8cea4cf1464856a7bf3693cda5ea35a633fff86; recursive reachability 12598167938 sha256:bb2b9983b0eea51cf3059f98b61999cf16934a4944bcc180afb82d4dac33baef; boost reachability 12598057886 sha256:dd04814971eb8d9ddc0a0af306e103c342615a37c18d797ddbd6c444bf54bc64; Rust messaging 12597938020 sha256:358c18e5530d1bf7d607731c87ef3f570476cef1824cd0323eb458b14d8e93d4. Receipt-head validation pending.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNR10_TTF_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SFNT-METADATA-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-NAME-HEAD-REWRITE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-GLYPH-LAYOUT-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-XML-METRIC-STYLE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-36196-98-1000-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-ROOT-36084-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-TDESKTOP-NAME-PREFIX-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-GLYPH-LAYOUT-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-FSTYPE-NOT-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-HISTORY-REWRITE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SFNT-XML-BINDING-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-SFNT-METADATA-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-NAME-PREFIX-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-GLYPH-MUTATION-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-FSTYPE-AS-LICENSE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-TTF-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
