# desktop-app/MicroTeX res/cyrillic/wnbxti10.xml complete read — strict229

## Authority
- PR #154 pre-write lease: `4d48d348780b129099c1776897219a24e51a4459`.
- Live main at audit start: `16aae12ea258f6a872a46aa47824766f64e1bdca`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/wnbxti10.xml@de5036e7cec6e7cfdda57f63ed2be550aa04781f`, 31,184 bytes, 708 lines.
- Root target at `17f3fc7816e4de1f5764036f34d759d50098ae58` is the same blob and bytes.
- Exact-path history contains only root `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`).
- Pre-write independent responsibility row count: 0.

## Exact XML font-metrics surface
The root is `Font(name="wnbxti10.ttf", id="wnbxti10", space="0.414441", xHeight="0.444445", quad="1.182211", unicode="95", romanVersion="wnbx10", ttVersion="wntt10", ssVersion="wnssbx10")`.

The file contains exactly 95 `Char` records and all 95 codes are unique, spanning 171 through 1141. It contains 512 `Kern` relations with five exact values: `-0.029445` x325, `-0.088333` x128, `-0.117777` x35, `-0.058888` x23 and `+0.029445` x1. It also contains 36 `Lig` relations, 12 `Char` records with nonzero depth and 93 with nonzero italic correction. Width spans 0.355553..1.318319, height 0.444445..0.894394, depth 0..0.194445 and italic correction 0..0.186251.

Historical `DefaultTeXFontParser::processCharElement` reads `code`, width, height, depth and italic (omitted values default to zero), stores metrics, then dispatches child `Kern` and `Lig`; `parse_kern` stores pair/value and `parse_lig` stores pair/replacement. This is executable TeX font-metric and shaping data, not presentation-only metadata.

## Style-version graph and FontInfo semantics
`language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8` includes `wnbxti10.xml` among eight Metrics files. `wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a` points to this font through `itVersion="wnbxti10"`. The target points back with `romanVersion="wnbx10"` and additionally declares `ttVersion="wntt10"` and `ssVersion="wnssbx10"`; `wnssbx10.xml@9fefc1c78c8b4bb1406173b8ac491c11f030a076` itself links `romanVersion="wnbx10"` and `ttVersion="wntt10"`, while `wntt10.xml@4baea371bade60de759650135b4773fbf2f28ee2` belongs to the regular roman/ss graph.

Historical root `fonts/font_basic.cpp@bd97ac19f06210ccbc65b1c96a621e5e1a04c6a7` resolves Metrics includes relative to the alphabet base, builds each `FontInfo` from name/id/space/xHeight/quad plus optional Unicode/style attributes, consumes every Char/Kern/Lig record, and resolves version IDs after loading. Current pinned `src/fonts/font_info.cpp@f957a586e157f5a192498dd14bf3d609ca4738c8` still shows the style-ID fields and lazy `Font::create(_path, Formula::PIXELS_PER_POINT)`. Therefore the XML historically couples the exact metric table and style graph to adjacent `wnbxti10.ttf`; the role is not inferred from the TTF's sfnt style bits.

## Accepted runtime boundary
Accepted `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still registers Cyrillic with `cyrillic/language_cyrillic.xml`, but accepted `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` makes `addTeXFontDescription` throw `ex_alphabet_registration`, and the registration wrapper fences the failure. The target XML therefore does not create an accepted-production `FontInfo`.

Accepted `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` only emits `TextRenderingAtom` when `Formula::getExternalFont(block)` returns an explicit mapping; otherwise unknown-character/partial-mode handling remains. This is an optional fallback boundary, not guaranteed WNBXTI10 rendering and not permission to revive the stripped XML loader.

## Install, shipping and license
`RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists the target and Meson `install_subdir('res/cyrillic', ...)` installs the subtree. Legacy/general `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` lists `wnbxti10.xml` and the Cyrillic fonts.

That is not accepted tdesktop shipping composition. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly excludes Cyrillic/Greek because the XML parser path is stripped. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` on macOS (or compiled bundled qrc elsewhere) and calls `tex::LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` copy/check that resource. Thus standalone/legacy inclusion is provenance only; accepted production excludes this XML.

MicroTeX README excludes fonts and XML resources under `res` from the project MIT blanket and points to `res/cyrillic`; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the subtree.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNBXTI10 XML runtime or shipping asset. No second Resource owner, font registry, font-metric/style store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict229: total=16,379, unknown=16,148, unread=16,123, omitted=0, responsibilities=246.
After strict229: total=16,379, unknown=16,147, unread=16,122, omitted=0, responsibilities=247.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnr10.ttf@a93586f1efd00c4009b72fc63a365661ce461435`.

## Parent exact-head evidence
Lease `4d48d348780b129099c1776897219a24e51a4459`: TDRP Revision 9 run `40188785537` completed/success, 61/61 jobs; Rust desktop runtime `40188785494` completed/success, 6/6 jobs; Official MCP marketplace `40188785498` completed/success, 1/1 job. Authority artifact `12597617359` digest `sha256:9d9fbb65571023b24faf4ede10997002051e4978ef8c83141340e55c0de14e53`; recursive closure `12596468777` digest `sha256:3bdb61e1be138ada6968a81a3068281d345eecd844d4286dd82d962e47718d52`; reachability `12596717872` digest `sha256:efd843d0ff5544958ee73921a67c2e9a34d3b6aff379713d1f6b08f648637f7f`; boost reachability `12597582603` digest `sha256:ff12a91a07d322d9e8997d81448e6c5364f6520be84e478d5bf709ca9377a835`; Rust messaging artifact `12596742517` digest `sha256:7f77c08f3cdcedc3465af0225bb1a4c2981d121eda75ab912967117b894d278d`. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_WNBXTI10_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-PROVENANCE-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-EXACT-SOURCE-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORY-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-FONT-CONSTANTS-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-METRICS-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-KERN-LIG-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-STYLE-VERSION-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORICAL-FONTINFO-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-INSTALL-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-LICENSE-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-31184-708-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-95-CHAR-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-512-KERN-36-LIG-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-ROOT-UNCHANGED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-TTF-PATH-BINDING-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-STYLE-GRAPH-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-NOT-BUNDLED-001`
- `INV-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-NO-PARALLEL-OWNER-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-PINNED-SOURCE-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-METRIC-TABLE-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-STYLE-VERSION-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORICAL-FONTINFO-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-ACCEPTED-FALLBACK-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-LICENSE-BOUNDARY-001`
- `CONTRACT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-SOURCE-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-FONT-CONSTANT-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-METRIC-CARDINALITY-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-KERN-LIG-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-STYLE-LINK-LOSS-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-TTF-BINDING-DRIFT-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-HISTORICAL-AS-PRODUCTION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-FALLBACK-AS-GUARANTEED-RENDER-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-LATEX-QRC-AS-SHIPPING-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-INSTALL-SHIPPING-CONFLATION-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-LICENSE-FLATTENING-001`
- `FAULT-TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-XML-RUNTIME-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
