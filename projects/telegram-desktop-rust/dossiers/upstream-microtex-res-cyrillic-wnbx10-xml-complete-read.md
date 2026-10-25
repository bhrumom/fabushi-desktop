# desktop-app/MicroTeX res/cyrillic/wnbx10.xml complete read — strict227

## Authority
- PR #154 pre-write lease: `ebb184391f700ebd32a2cbb5fccbb037e44580ce`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a`, 25,643 bytes, 616 lines.
- Pre-write independent responsibility row count: 0.

## Exact XML font-metrics surface
The root is `Font(name="wnbx10.ttf", id="wnbx10", space="0.383331", xHeight="0.444445", quad="1.149994", unicode="95", itVersion="wnbxti10", ssVersion="wnssbx10", ttVersion="wntt10")`.

The file contains exactly 95 `Char` records and all 95 character codes are unique, spanning code points 171 through 1141. It contains 416 `Kern` relations with five exact values: `-0.031944` x266, `-0.095833` x117, `-0.127777` x13, `-0.063889` x19 and `+0.031944` x1. It also contains 36 `Lig` relations, 13 `Char` records with nonzero depth and five with nonzero italic correction.

Historical `DefaultTeXFontParser::processCharElement` reads each Char code and width/height/depth/italic, defaulting omitted values to zero, then dispatches child `Kern` and `Lig` records. `parse_kern` stores the pair/value and `parse_lig` stores the pair/replacement. This XML carried actual TeX font-metric, kerning, ligature and style-variant semantics rather than presentation-only metadata.

## Exact source/history
Exact-path history at the pinned commit contains only root `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`). The root recursive tree is `truncated=false` with 171 entries and already contains the same `wnbx10.xml@3e42b024...` blob at 25,643 bytes. There is no later exact-path commit for this XML.

Strict226 established that `c1546216f32d398dca90e1dc33df34842683ec42` rewrote the WNBX10 TTF family/PostScript names. It did not rewrite `wnbx10.xml`: the XML still names adjacent `wnbx10.ttf`, and its metric constants, character records, kerning and ligatures remain root-stable.

## Historical FontInfo/runtime semantics
`language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8` includes `wnbx10.xml` among eight `FontDescriptions/Metrics` files.

Historical `DefaultTeXFontParser::parseFontDescriptions` resolves each Metrics include relative to the alphabet base. For an individual font XML it reads name/id/space/xHeight/quad, optional Unicode/skew and style-version attributes, derives the TTF path from the XML parent plus `name`, constructs `FontInfo`, consumes every Char/Kern/Lig record, then resolves style-version IDs. `FontInfo::getFont()` lazily calls `Font::create(_path, ...)`. Thus this XML historically coupled TeX metrics and variant selection to the adjacent WNBX10 TTF.

## Accepted runtime boundary
Accepted `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still registers Cyrillic `0x0400..0x04ff` with `cyrillic/language_cyrillic.xml`, but accepted `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` removed the tinyxml2-backed font-description parser. `addTeXFontDescription` throws `ex_alphabet_registration`; the registration wrapper catches/fences that failure, so this XML does not create an accepted-production `FontInfo`.

Accepted `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` checks symbol/formula mappings and, when absent, asks `Formula::getExternalFont(block)`; when a mapping exists it emits `TextRenderingAtom`. Without such a mapping it follows unknown-character/partial-mode handling. Therefore the accepted boundary is an optional external-font/TextRenderingAtom fallback, not guaranteed WNBX10 rendering and not a reason to revive the XML loader.

## Install, resource manifests, shipping and license
`RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists `cyrillic/wnbx10.xml`, and Meson installs the full `res/cyrillic` subtree. Legacy/general `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270` also lists the target XML and Cyrillic fonts.

That is not the accepted tdesktop shipping composition. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly excludes Cyrillic and Greek because their XML loader was stripped. Accepted tdesktop `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` registers `external_microtex_bundled.rcc` on macOS (or the compiled bundled qrc elsewhere) and calls `tex::LaTeX::initBundled()`; `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` and `build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3` copy/package that bundled resource. Standalone Meson/legacy `latex.qrc` inclusion is provenance; accepted tdesktop production bundling excludes the target.

MicroTeX README excludes fonts and XML resources under `res` from the project MIT blanket and points to `res/cyrillic`; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the subtree. The target has no inline license header that overrides that repository boundary.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNBX10 XML runtime or shipping asset. No second Resource owner, font registry, font-metric store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict227: total=16,379, unknown=16,150, unread=16,125, omitted=0, responsibilities=244.
After strict227: total=16,379, unknown=16,149, unread=16,124, omitted=0, responsibilities=245.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnbxti10.ttf@e85a2a442738554dc728a75546d10fd6fef419d1`.

## Parent exact-head evidence
Lease `ebb184391f700ebd32a2cbb5fccbb037e44580ce`: TDRP Revision 9 run `40182572571` completed/success, 61/61 jobs; Rust desktop runtime `40182572570` completed/success, 6/6 jobs; Official MCP marketplace `40182572601` completed/success, 1/1 job. TDRP authority artifact `12594727335` has digest `sha256:dcedb6419a7eb405111e8e6216dd7fa687e19c2657b6fc56d858071f974b22c9`; recursive closure `12594827322`, reachability `12594747530`, boost reachability `12594567800`, and Rust messaging artifact `12594956629` are all bound to the same exact parent head. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_WNBX10_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNBX10-XML-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-FONT-CONSTANTS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-METRICS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-KERN-LIG-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORICAL-FONTINFO-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-25643-616-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-95-CHAR-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-416-KERN-36-LIG-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-ROOT-UNCHANGED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-TTF-PATH-BINDING-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-NO-PARALLEL-OWNER-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-METRIC-TABLE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORICAL-FONTINFO-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-ACCEPTED-FALLBACK-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-LICENSE-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-FONT-CONSTANT-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-METRIC-CARDINALITY-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-KERN-LIG-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-TTF-REWRITE-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-HISTORICAL-AS-PRODUCTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-FALLBACK-AS-GUARANTEED-RENDER-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-LATEX-QRC-AS-SHIPPING-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-INSTALL-SHIPPING-CONFLATION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-LICENSE-FLATTENING-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-XML-RUNTIME-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
