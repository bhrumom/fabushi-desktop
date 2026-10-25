# desktop-app/MicroTeX res/cyrillic/wnti10.xml complete read - strict239

## Authority and exact-path history
PR #154 pre-write lease is 2fa0a0d811ea04247a13495284919fc46a1e456c. Accepted tdesktop is a7f453612f4896ac04d0007a36ab0d377e5bcbcf; Telegram/ThirdParty/MicroTeX pins desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. GitHub resolves the public repository lineage through NanoMichael/MicroTeX while preserving the pinned commit identity.

Target res/cyrillic/wnti10.xml is blob 55046e15cbd13b5ceaed2ea6955cd378f6167599, 31,229 UTF-8 bytes / 708 lines. Root 17f3fc7816e4de1f5764036f34d759d50098ae58, c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e, c1546216f32d398dca90e1dc33df34842683ec42 and pinned 61aaa7cc all resolve to the same blob. Exact-path history contains only root init; c154 changed the adjacent TTF namespace, not this XML.

## Font constants, metrics, shaping and style graph
Font root: name=wnti10.ttf, id=wnti10, space=0.357776, xHeight=0.430555, quad=1.022217, unicode=95, romanVersion=wnr10, boldVersion=wnbxti10, ssVersion=wnssi10, ttVersion=wntt10.

Complete parse: 95 Char / 95 unique codes 171..1141; width 0.306665..1.152216; height 0.430555..0.891615; 12 nonzero depth (max 0.194445); 93 nonzero italic (max 0.183611). Exactly 512 Kern and 36 Lig. Kern distribution: -0.025556 x325, -0.076666 x128, -0.1022215 x35, -0.051111 x23, +0.025556 x1.

wnr10.xml@319d5db2d3ab2471d565e77e83dfdd29450fb106 has itVersion=wnti10; wnssi10.xml@5c2318c66e402f47c05ffb67015bcff0780523a8 has romanVersion=wnti10; language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8 includes wnti10.xml. Adjacent wnti10.ttf is root/c154-parent eb5217f6c6b9d1a3a99fbbcf0840b8188bfb8a7a and c154/pinned 1dd164b2a3602494a1418eb16aca22d298b01ebd; c154 namespaces family/full/PostScript while XML metrics remain unchanged.

## Historical runtime and accepted boundary
font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8 historically lazy-loads Font::create(_path, Formula::PIXELS_PER_POINT). alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891 points Cyrillic to cyrillic/language_cyrillic.xml. Accepted fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b removes the tinyxml2 parser: addTeXFontDescription throws ex_alphabet_registration and addAlphabet fences it, so accepted production does not create WNTI10 XML FontInfo state. formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 defaults absent external-font blocks to SansSerif/Serif; this is not WNTI10 execution.

## Install, shipping, license and ownership
RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 and legacy latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270 include/install the target. Accepted res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa explicitly excludes Cyrillic/Greek. Accepted tdesktop initializes/packages external_microtex_bundled.rcc via iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2, Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3, therefore that bundle omits WNTI10 XML/TTF.

README.md@824af04fd8607532efed3f73752b81e5093577ef excludes res fonts/XML from the MIT blanket; res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 is GPLv3 provenance. Existing-owner-first selects existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance; novel_capability=false.

## Coverage and exact-head evidence
Before strict239: total=16,379 unknown=16,138 unread=16,113 omitted=0 responsibilities=256.
After strict239: total=16,379 unknown=16,137 unread=16,112 omitted=0 responsibilities=257.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance remains not accepted.

Parent 2fa0a0d811ea04247a13495284919fc46a1e456c exact-head evidence is terminal-success: TDRP Revision 9 run 40214093535 is 30/30 success including Source authority / recursive inventory 121231364510. Authority artifact 12606159782 digest sha256:8c2ada2d0292d785faff0c54f467089ecf1c5e6d9abf27eb0045e39fdc1d8063; recursive closure 12607102507 digest sha256:597c30d5918ee5c29e08b7668d2654592da60fcc87523932d04ba4ac07772fce; recursive reachability 12607156158 digest sha256:693e31156a13b88686c2e23f796868dfff30c846a0424510f3224d8ede6c247d; boost reachability 12606858026 digest sha256:79aab3ab1a6832ceca63cdba1152b16dbb2bea4cd809f1e122d97f65c11200f9. Rust desktop runtime 40214093545 is 6/6 success including rust-host 121231365052; messaging artifact 12606912325 digest sha256:6054710b6914157fb2ef72e280bed4ac5ebaa2a0ab70f7a3d069eeecd5bd8eb6. Official MCP marketplace 40214093561 is 1/1 success.

Next deterministic non-directory entry: res/cyrillic/wntt10.ttf@649e9b1b24b3805646ad7cee71ae6e8d6e73e2c9 (33,424 bytes).

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNTI10_XML_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-FONT-CONSTANTS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-KERN-LIG-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-STYLE-VERSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-31229-708-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-95-CHAR-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-512-KERN-36-LIG-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-ROOT-C154-PINNED-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-TTF-PATH-BINDING-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-METRIC-TABLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-STYLE-VERSION-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-FONT-CONSTANT-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-METRIC-CARDINALITY-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-KERN-LIG-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-TTF-BINDING-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-TTF-NAMESPACE-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-LATEX-QRC-AS-SHIPPING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNTI10-XML-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
