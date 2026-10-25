# desktop-app/MicroTeX res/cyrillic/wntt10.xml complete read - strict241

## Authority and exact-path history
PR #154 pre-write lease is f5fcf6f3523ac506ad4312c597bdfe136668becf. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; Telegram/ThirdParty/MicroTeX pins desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. GitHub resolves the public repository lineage through NanoMichael/MicroTeX while preserving the pinned commit identity.

Target res/cyrillic/wntt10.xml is blob 4baea371bade60de759650135b4773fbf2f28ee2, 7,679 bytes / 163 text lines including the terminal newline. Root 17f3fc7816e4de1f5764036f34d759d50098ae58, c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e, c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f all resolve to the same XML blob. In pinned ancestry the path originates at root init; later d5124121 removal is outside the pinned ancestry and does not change the accepted pinned identity. c154 changes the adjacent TTF namespace, not this XML.

## Font constants, metrics and shaping
The Font root is name=wntt10.ttf, id=wntt10, space=0.524996, xHeight=0.430555, quad=1.049991, unicode=95, romanVersion=wnr10 and ssVersion=wnss10. There are exactly 95 Char records / 95 unique codes spanning 171..1141, 36 Lig relations and zero Kern records. Every Char width is 0.524996, matching the typewriter fixed-width role. Heights are drawn from 0.430555, 0.438889, 0.611112, 0.638838, 0.694445 and 0.819394. Depth is absent/zero or 0.13889, 0.166667 or 0.222223, with 13 nonzero-depth records. No Char declares nonzero italic correction.

The XML is therefore not just a file list entry: it carries the historical fixed-width Cyrillic metric table plus ligature substitutions. The absence of Kern is a semantic distinction from proportional/italic Cyrillic metric files and must not be reconstructed as a generic proportional font profile.

## Style graph and historical behavior
language_cyrillic.xml includes wntt10.xml. wnr10.xml declares ttVersion=wntt10, and wnss10.xml also declares ttVersion=wntt10. WNTT10 itself points back to romanVersion=wnr10 and ssVersion=wnss10, so it participates in the historical Cyrillic style-variant graph without defining bold/italic versions of its own.

Historical FontInfo lazily calls Font::create(_path, Formula::PIXELS_PER_POINT), so path=wntt10.ttf binds these metrics to the adjacent TTF when the XML-driven registration path is active. The c154 font-namespace change affects that adjacent TTF family/full/PostScript names while this XML remains byte-identical.

## Accepted runtime boundary
src/fonts/alphabet.cpp identifies cyrillic/language_cyrillic.xml. Accepted src/fonts/fonts.cpp removes the tinyxml2-backed description parser: addTeXFontDescription throws and addAlphabet fences ex_alphabet_registration. Accepted production therefore does not create WNTT10 XML FontInfo state.

src/core/formula.cpp creates default FontInfos("SansSerif","Serif") when Formula::getExternalFont has no block mapping. That generic fallback is not WNTT10 metric execution and must not be represented as source-font rendering.

## Install, shipping, license and ownership
res/RES_README lists wntt10.ttf and wntt10.xml; meson.build installs res/cyrillic; legacy latex.qrc includes both files. Accepted res/bundled.qrc explicitly excludes Cyrillic/Greek because registration throws and falls back. tdesktop initializes/packages external_microtex_bundled.rcc through iv_markdown_microtex.cpp, Telegram/CMakeLists.txt and build_mac.py, so the accepted bundle omits WNTT10 XML/TTF.

README.md excludes fonts and XML under res from the MIT blanket; res/cyrillic/LICENSE is GPLv3 provenance. Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance; novel_capability=false. No MicroTeX/Cyrillic font registry/runtime, second Resource owner, second font store, second license store or second packaging root is introduced.

## Coverage and parent exact-head evidence
Before strict241: total=16,379 unknown=16,136 unread=16,111 omitted=0 responsibilities=258.
After strict241: total=16,379 unknown=16,135 unread=16,110 omitted=0 responsibilities=259.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Parent f5fcf6f3523ac506ad4312c597bdfe136668becf TDRP Revision 9 run 40216969686 is terminal-success, including Source authority / recursive inventory 121240094087 and Recursive root authority accounting gate 121240322978. Authority artifact 12608231983 digest sha256:c7f86b952f1b5fa920e3a94c5e39869448b030a8473e7b27e3314b7338a9f010; recursive closure 12608182798 digest sha256:7dee063cc378fcf01deb4131a816e069487ed035ebfcc8ca2792a9cae98132c2; recursive reachability 12607318939 digest sha256:b4cc3482dfd5815051f58b1bd827e961704cd8f71468600b4111763e19cd5423; boost reachability 12607219103 digest sha256:1207c06268030a6461799606a7e1848216622f1bcd516ba508457419c17a572e.

Rust desktop runtime run 40216969649 is 6/6 terminal-success including rust-host 121240094161; messaging artifact 12608037673 digest sha256:6eba39137c8dbcac81b40f5c84696a5951bc681a8fc0cc76c222913a9395989c. Official MCP marketplace run 40216969644 is 1/1 terminal-success; contracts job 121240093447 succeeded.

Next deterministic non-directory entry after the pinned MicroTeX recursive order: res/fonts/base/cmex10.ttf.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNTT10_XML_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-FONT-CONSTANTS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-LIG-NO-KERN-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-STYLE-VERSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-INSTALL-SHIPPING-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-7679-95-36-0-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-FIXED-WIDTH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-METRICS-SHAPING-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-STYLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-SHIPPING-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-METRIC-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-KERN-INVENTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-STYLE-GRAPH-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-FALLBACK-AS-SOURCE-FONT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTT10-XML-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
