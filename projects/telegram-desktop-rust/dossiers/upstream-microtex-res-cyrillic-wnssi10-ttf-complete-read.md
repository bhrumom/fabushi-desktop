# desktop-app/MicroTeX res/cyrillic/wnssi10.ttf complete read - strict236

## Authority and history
PR #154 pre-write lease is b13adb5f77ac9e8cd11146ffa953f699ff3dfcd5. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; pinned MicroTeX is 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. Target res/cyrillic/wnssi10.ttf is affdad63d2ae3393cb25554cbc3df05a98d6556e, 25,056 bytes.

Root 17f3fc7816e4de1f5764036f34d759d50098ae58 and c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e resolve to bd1f8718691fe0f27030c09bca58358ee12c8048, 24,936 bytes. c1546216f32d398dca90e1dc33df34842683ec42 and pinned resolve to affdad63d2ae3393cb25554cbc3df05a98d6556e, 25,056 bytes. Exact-path history contains exactly root init and c154 Attempt to force our fonts.

## sfnt and c154 namespace semantics
Pinned WNSSI10 has 15 tables and 98 glyphs. head: unitsPerEm=1000, bounds -97,-205..1202,903, macStyle=0, lowestRecPPEM=8, short loca. hhea: 904/-205/90, advanceWidthMax=1111, 98 long metrics. OS/2 v3: xAvg=621, weight=400, width=5, fsType=0, fsSelection=64, firstChar=0, lastChar=1141. cmap format-4 maps 97 entries through 1141. glyf has 95 simple, zero composite and three empty glyphs.

Root family/full/PostScript names are wnssi10. c154/pinned family/full are TDesktop wnssi10 and PostScript is TDesktop-wnssi10; style Regular and version 1.001 are unchanged. Root-to-c154 table-directory changes are limited to name plus head checksum adjustment; measured glyph/layout/OS2/cmap/hhea/maxp semantics are unchanged.

c154 also changes the Qt loader to document same-named system-font shadowing, use the application-font family returned by QFontDatabase, and warn with QFontInfo if resolution differs. The namespace rewrite therefore carries bundled-font collision-isolation responsibility.

## Adjacent XML/style graph and runtime boundary
wnssi10.xml@5c2318c66e402f47c05ffb67015bcff0780523a8 is 26,762 bytes / 606 lines. It has name=wnssi10.ttf, id=wnssi10, space=0.333334, xHeight=0.444445, quad=1.000003, unicode=95, romanVersion=wnti10, ttVersion=wntt10 and boldVersion=wnssbx10. It contains 95 Char / 95 unique codes 171..1141, 406 Kern, 36 Lig, 13 nonzero-depth and 93 nonzero-italic records. wnss10.itVersion=wnssi10 and wnti10.ssVersion=wnssi10; language_cyrillic.xml includes wnssi10.xml.

Historical FontInfo lazily calls Font::create on the adjacent path. Accepted production does not traverse the XML path: addTeXFontDescription throws ex_alphabet_registration and addAlphabet fences it. Formula::getExternalFont may create default SansSerif/Serif FontInfos; that fallback is not WNSSI10 loading.

## Install, shipping, license and ownership
RES_README, Meson and legacy latex.qrc include/install the target. Accepted bundled.qrc explicitly excludes Cyrillic/Greek and accepted external_microtex_bundled.rcc therefore omits WNSSI10. MicroTeX README excludes res fonts/XML from the project MIT blanket; res/cyrillic/LICENSE is GPLv3 provenance. OS/2 fsType=0 is embedding metadata, not legal-license authority.

Existing-owner-first selects the existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. No second Resource owner, font registry/store, MicroTeX/Cyrillic runtime, license store or package root is introduced. novel_capability=false.

## Coverage and evidence
Before strict236: total=16,379 unknown=16,141 unread=16,116 omitted=0 responsibilities=253.
After strict236: total=16,379 unknown=16,140 unread=16,115 omitted=0 responsibilities=254.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Parent b13adb5f exact-head evidence: TDRP Revision 9 run 40208877191 is 61/61 success; Rust desktop runtime 40208877250 is 6/6 success including rust-host 121215628890, with messaging artifact 12604593649 digest sha256:dedd0c36ddea027a5995aa3b5aa8c64cce03521e20a720795172cccb03e62aee; Official MCP marketplace 40208877193 is 1/1 success. Authority artifact 12604683682 digest sha256:2174b1c15a0730fa15307a7334f10584ad09e45cdf66e9164d281b8ec3312008; recursive closure 12604908806; recursive reachability 12604034096; boost reachability 12605133022.

Next deterministic non-directory entry: res/cyrillic/wnssi10.xml@5c2318c66e402f47c05ffb67015bcff0780523a8 (26,762 bytes / 606 lines).
