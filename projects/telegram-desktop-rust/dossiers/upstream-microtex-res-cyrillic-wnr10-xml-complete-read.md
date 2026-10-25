# desktop-app/MicroTeX res/cyrillic/wnr10.xml complete read — strict231

## Authority
- PR #154 pre-write lease: `ad85605443cf1208905817345e86cbc9fdced966`.
- Live main at audit start: `16aae12ea258f6a872a46aa47824766f64e1bdca`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/wnr10.xml@319d5db2d3ab2471d565e77e83dfdd29450fb106`, 25,900 bytes, 616 lines.
- Root `17f3fc7816e4de1f5764036f34d759d50098ae58`, c154 `c1546216f32d398dca90e1dc33df34842683ec42`, and pinned all resolve to the same target blob and identical bytes.
- Exact-path history contains only root `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`); c154 changed adjacent TTF naming, not this XML.
- Pre-write independent responsibility row count: 0.

## Exact XML font-metrics surface
The root is `Font(name="wnr10.ttf", id="wnr10", space="0.333334", xHeight="0.430555", quad="1.000003", unicode="95", boldVersion="wnbx10", ssVersion="wnss10", ttVersion="wntt10", itVersion="wnti10")`.

The file contains exactly 95 `Char` records and all 95 codes are unique, spanning 171 through 1141. Width spans 0.277779..1.194448, height 0.430555..0.891615, depth 0..0.194445 and italic correction 0..0.013888; 13 records have nonzero depth and 19 have nonzero italic correction.

It contains 416 `Kern` relations with five exact values: `-0.027779` x266, `-0.083334` x117, `-0.111112` x13, `-0.055555` x19 and `+0.027779` x1. It also contains exactly 36 `Lig` relations. These are executable TeX metric/shaping semantics, not presentation-only metadata.

## Style-version graph and historical FontInfo semantics
`language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8` includes `wnr10.xml` among eight Metrics descriptions. The target declares `boldVersion=wnbx10`, `ssVersion=wnss10`, `ttVersion=wntt10`, and `itVersion=wnti10`. `wnss10.xml@660c29af27ddab8188fcc7594bae0941cc45d9f6`, `wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2`, and `wnti10.xml@55046e15cbd13b5ceaed2ea6955cd378f6167599` point back with `romanVersion=wnr10`.

Historical root `fonts/font_basic.cpp@bd97ac19f06210ccbc65b1c96a621e5e1a04c6a7` resolves Metrics includes, stores per-character width/height/depth/italic plus Kern/Lig relations and style IDs, and binds the XML name to the adjacent TTF path. Current pinned `src/fonts/font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8` retains bold/roman/ss/tt/it IDs and lazily calls `Font::create(_path, Formula::PIXELS_PER_POINT)`. Therefore WNR10's historical style role is XML-driven and tied to adjacent `wnr10.ttf`.

## Accepted runtime boundary
Accepted `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still registers Cyrillic through `cyrillic/language_cyrillic.xml`, but accepted `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` makes `addTeXFontDescription` throw `ex_alphabet_registration`; the registration wrapper catches/fences the failure. The target XML therefore does not create an accepted-production `FontInfo`.

Accepted `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` emits `TextRenderingAtom` only when `Formula::getExternalFont(block)` returns an explicit mapping; otherwise unknown-character/partial-mode handling applies. This is optional fallback, not guaranteed WNR10 rendering and not permission to revive the stripped XML loader.

## Install, shipping and license
`RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists `cyrillic/wnr10.xml`; Meson installs the `res/cyrillic` subtree; legacy/general `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` lists both `wnr10.ttf` and `wnr10.xml`.

That is not accepted tdesktop shipping composition. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly excludes Cyrillic/Greek because the XML parser path is stripped. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` on macOS and calls `tex::LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` copy/check that accepted resource. Standalone/legacy inclusion is provenance only; accepted production excludes this XML.

MicroTeX `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes fonts and XML resources under `res` from the project MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNR10 XML runtime or shipping asset. No second Resource owner, font registry, font-metric/style store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict231: total=16,379, unknown=16,146, unread=16,121, omitted=0, responsibilities=248.
After strict231: total=16,379, unknown=16,145, unread=16,120, omitted=0, responsibilities=249.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnss10.ttf@6cc01cfd7d25ab5db44f01466bb0c5f54e9a144e`.

## Parent exact-head evidence
Lease `ad85605443cf1208905817345e86cbc9fdced966`: TDRP Revision 9 run `40194771569` completed/success, 61/61 jobs; Rust desktop runtime `40194771568` completed/success, 6/6 jobs; Official MCP marketplace `40194771559` completed/success, 1/1 job. Authority artifact `12599303531` digest `sha256:088a43130c499d37cc2c5966b7c99618947bcdb5e4dfcabc74a03da133460ae6`; recursive closure `12599203972` digest `sha256:485710aa1a9fe58f1da4fde5c1217907751ae2fb134602c936ac257419c503c5`; recursive reachability `12599498393` digest `sha256:aa996735a731f513a6017c6239a39dac51450181465b0b29f032c5caf588d923`; boost reachability `12599383594` digest `sha256:7a91475529a940b774d6d74aa95498a97549969ed95ed26f3a18780c6ab5f5ef`; Rust messaging artifact `12599128628` digest `sha256:3170cd1e350fda0d3863627acdd9dad450075d675d73ffeda27f2e098e8fffe8`. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_WNR10_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-PROVENANCE-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-EXACT-SOURCE-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORY-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-FONT-CONSTANTS-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-METRICS-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-KERN-LIG-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-STYLE-VERSION-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORICAL-FONTINFO-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-INSTALL-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-LICENSE-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-25900-616-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-95-CHAR-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-416-KERN-36-LIG-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-ROOT-C154-PINNED-UNCHANGED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-TTF-PATH-BINDING-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-STYLE-GRAPH-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-NOT-BUNDLED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-NO-PARALLEL-OWNER-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-PINNED-SOURCE-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-METRIC-TABLE-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-STYLE-VERSION-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORICAL-FONTINFO-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-ACCEPTED-FALLBACK-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-LICENSE-BOUNDARY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-SOURCE-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-FONT-CONSTANT-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-METRIC-CARDINALITY-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-KERN-LIG-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-STYLE-LINK-LOSS-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-TTF-BINDING-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-HISTORICAL-AS-PRODUCTION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-FALLBACK-AS-GUARANTEED-RENDER-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-LATEX-QRC-AS-SHIPPING-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-INSTALL-SHIPPING-CONFLATION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-LICENSE-FLATTENING-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNR10-XML-RUNTIME-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
