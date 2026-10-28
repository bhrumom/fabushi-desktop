# desktop-app/MicroTeX res/cyrillic/wnssbx10.xml complete read - strict235

## Authority
- PR #154 pre-write lease: 33b13662714f6f532ba942e5d888fe9d5780d6cb.
- Live main at audit start: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned MicroTeX: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f, tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- Target: res/cyrillic/wnssbx10.xml@9fefc1c78c8b4bb1406173b8ac491c11f030a076, 25,211 bytes / 606 lines.
- Root 17f3fc7816e4de1f5764036f34d759d50098ae58, c154 parent 43904b5cbae8154f041209eea6d13f7c8c4ffd6e, c1546216f32d398dca90e1dc33df34842683ec42 and pinned all resolve to the same target blob and identical bytes.
- Exact-path history contains only root 17f3fc7816e4de1f5764036f34d759d50098ae58 (init); c154 rewrites the adjacent TTF family namespace, not this XML.
- Pre-write independent responsibility row count: 0.

## Exact XML font-metrics surface
The Font root is `Font(name="wnssbx10.ttf", id="wnssbx10", space="0.366669", xHeight="0.458333", quad="1.100006", unicode="95", romanVersion="wnbx10", ttVersion="wntt10")`.

The file contains exactly 95 `Char` records and all 95 codes are unique, spanning 171 through 1141. Width spans 0.255557..1.250008, height 0.458333..0.902727, depth 0..0.194445 and italic correction 0..0.02139; 13 records have nonzero depth and 7 have nonzero italic correction.

It contains 406 `Kern` relations with four exact values: `-0.030556` x274, `-0.091667` x105, `-0.061111` x26 and `+0.030556` x1. It also contains exactly 36 `Lig` relations. These are executable TeX metric/shaping semantics, not presentation-only metadata.

## Style-version graph and historical FontInfo semantics
`language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8` includes `wnssbx10.xml` among eight Metrics descriptions. The target declares `romanVersion=wnbx10` and `ttVersion=wntt10`. `wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a` reciprocally declares `ssVersion=wnssbx10` and also points to `ttVersion=wntt10`; `wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2` is the shared typewriter variant and declares `romanVersion=wnr10`, `ssVersion=wnss10`, not a direct reverse edge to WNSSBX10.

Historical root `fonts/font_basic.cpp@bd97ac19f06210ccbc65b1c96a621e5e1a04c6a7` parses per-character width/height/depth/italic data plus Kern/Lig children and style IDs, binds the XML name to the adjacent TTF path, and lazily calls `Font::create`. Pinned `src/fonts/font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8` retains style IDs and lazily calls `Font::create(_path, Formula::PIXELS_PER_POINT)`. The XML path stays `wnssbx10.ttf`; c154 changes that TTF's internal family namespace to `TDesktop wnssbx10` but does not change this XML.

## Accepted runtime boundary
Pinned `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still registers Cyrillic through `cyrillic/language_cyrillic.xml`, but pinned `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` makes `addTeXFontDescription` throw `ex_alphabet_registration`; `addAlphabet` catches/fences that failure. WNSSBX10 XML therefore does not create an accepted-production `FontInfo`.

Pinned `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` can request `Formula::getExternalFont(block)` for non-Latin text. `src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303` creates default `FontInfos("SansSerif", "Serif")` when the block has no explicit mapping. `TextRenderingAtom` can render through that generic fallback. It does not parse WNSSBX10 XML metrics, does not load `wnssbx10.ttf` through this XML, and does not make WNSSBX10 a guaranteed shipping font.

## Install, shipping and license
`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists `cyrillic/wnssbx10.xml`. `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs the whole `res/cyrillic` subtree. Legacy/general `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` lists both `wnssbx10.ttf` and `wnssbx10.xml`. These are standalone/legacy inclusion provenance.

Accepted tdesktop shipping differs. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly excludes Cyrillic/Greek because that XML parser path is stripped. Accepted `Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` on macOS and calls `tex::LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies that resource and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` checks it in packaged resources. Because the accepted MicroTeX bundle excludes Cyrillic/Greek, WNSSBX10 XML is not in accepted tdesktop production composition.

MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts and XML resources under `res` from the project MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance.

## Existing-owner-first
Live architecture authority was re-read at this lease. Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNSSBX10 XML runtime or shipping asset. No second Resource owner, font registry, font-metric/style store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced. `novel_capability=false`.

## Coverage
Before strict235: total=16,379, unknown=16,142, unread=16,117, omitted=0, responsibilities=252.
After strict235: total=16,379, unknown=16,141, unread=16,116, omitted=0, responsibilities=253.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnssi10.ttf@affdad63d2ae3393cb25554cbc3df05a98d6556e` (25,056 bytes).

## Parent exact-head evidence
Lease 33b13662714f6f532ba942e5d888fe9d5780d6cb: TDRP Revision 9 run `40206011406` completed/success, 61/61 jobs; Rust desktop runtime `40206011382` completed/success, 6/6 jobs; Official MCP marketplace `40206011383` completed/success, 1/1 job.
Authority artifact `12603329184` digest `sha256:e85732b5bf7251a7e530e8e0537d84dad6ac14b2b9aa0dc9bb8834d828b15f4d`; recursive closure `12603958086` digest `sha256:bab54130255cc2a3b021302a524583e7181ab18c53079f63fa9ace514a509228`; recursive reachability `12603029823` digest `sha256:db146d3b8a7a57e4ff2ca0b112cd35409c0ffe886af4f44b66d925d5336c0c1a`; boost reachability `12602929912` digest `sha256:229deb1aee153c3ea0827aeb84e175b00498a8f84fd8671e3d63a5ee7b53e7b2`; Rust messaging artifact `12603663186` digest `sha256:9075ecd6f3989187bd698f715cb8cee7db298abb3b98e225f7ae304c747c3e81`.
Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- TDRP_MICROTEX_RES_CYRILLIC_WNSSBX10_XML_PROVENANCE_001
- TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-PROVENANCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-EXACT-SOURCE-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORY-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-FONT-CONSTANTS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-METRICS-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-KERN-LIG-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-STYLE-VERSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORICAL-FONTINFO-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-ACCEPTED-FALLBACK-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-INSTALL-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-SHIPPING-EXCLUSION-001
- ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-LICENSE-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-25211-606-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-95-CHAR-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-406-KERN-36-LIG-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-ROOT-C154-PINNED-UNCHANGED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-TTF-PATH-BINDING-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-STYLE-GRAPH-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-DORMANT-ACCEPTED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-NOT-BUNDLED-001
- INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-NO-PARALLEL-OWNER-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-PINNED-SOURCE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-METRIC-TABLE-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-STYLE-VERSION-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORICAL-FONTINFO-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-ACCEPTED-FALLBACK-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-LICENSE-BOUNDARY-001
- CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-SOURCE-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-FONT-CONSTANT-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-METRIC-CARDINALITY-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-KERN-LIG-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-STYLE-LINK-LOSS-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-TTF-BINDING-DRIFT-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-HISTORICAL-AS-PRODUCTION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-FALLBACK-AS-GUARANTEED-RENDER-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-LATEX-QRC-AS-SHIPPING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-INSTALL-SHIPPING-CONFLATION-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-LICENSE-FLATTENING-001
- FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNSSBX10-XML-RUNTIME-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
