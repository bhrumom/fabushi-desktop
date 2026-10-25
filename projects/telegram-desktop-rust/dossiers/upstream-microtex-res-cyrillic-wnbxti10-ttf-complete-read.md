# desktop-app/MicroTeX res/cyrillic/wnbxti10.ttf complete read — strict228

## Authority
- PR #154 pre-write lease: `ed8aab4a19ec53e489d41947146cf933f978a085`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/wnbxti10.ttf@e85a2a442738554dc728a75546d10fd6fef419d1`, 52,160 bytes.
- Root target blob: `d7b5cb69ae108e64f2de62b72442d237e98a5bc4`, 52,028 bytes.
- Exact-path history: root `17f3fc7816e4de1f5764036f34d759d50098ae58` plus `c1546216f32d398dca90e1dc33df34842683ec42` (`Attempt to force our fonts.`).
- Pre-write independent responsibility row count: 0.

## Exact sfnt surface
The target is a TrueType sfnt with 15 tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, gasp, glyf, head, hhea, hmtx, loca, maxp, name and post. `maxp.numGlyphs=98`; `head.unitsPerEm=1000`, bounds `(-30,-203)..(1401,926)`, `macStyle=0`, `lowestRecPPEM=8`, `indexToLocFormat=0`. `OS/2` is version 3 with `xAvgCharWidth=756`, `usWeightClass=400`, `usWidthClass=5`, `fsType=0`, `fsSelection=64`, `usLastCharIndex=1141`, typo ascender/descender `800/-200`, Win ascent/descent `926/204`. `hhea` has `advanceWidthMax=1318` and 98 horizontal metrics. `post.italicAngle=0`, `isFixedPitch=0`.

Current Mac/Windows name records report family/full `TDesktop wnbxti10`, subfamily `Regular`, unique `TDesktop-wnbxti10:Version 1.001`, PostScript `TDesktop-wnbxti10`, and `Version 1.001 December 29, 2009`. Name ID 0 records mftrace 1.2.5 generation provenance. There are no name ID 13 or 14 license-description/license-URL records.

## Exact history and namespace rewrite
The root file is 52,028 bytes and names family/full/unique/PostScript simply `wnbxti10`. c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` still materializes that root blob.

Root/current table-directory comparison is exact: only `name` and `head` differ. `name` grows 612 -> 741 bytes and checksum changes `18d9cdff -> 4b316ae2`; `head` remains 54 bytes but checksum changes `f338644d -> 1228f079` because checksumAdjustment follows the binary rewrite. DSIG/GDEF/GPOS/GSUB/OS/2/cmap/gasp/glyf/hhea/hmtx/loca/maxp/post retain identical checksums and lengths. In particular `glyf=4e563c70/48612`, `loca=2e38ffae/198`, `hmtx=1e892243/392`, `cmap=1b701f47/690`, `GSUB=6c91748f/32`, `GPOS=0019000c/18`. Thus c154 is a family namespace/checksum rewrite, not a glyph, advance, cmap, metric or OpenType-layout rewrite.

c154's Qt patch states the reason: same-named system fonts could shadow bundled families and make glyphs disappear, so bundled font families were prefixed and resolved-family diagnostics were added.

## FontInfo/style-version runtime semantics
The TTF's own sfnt metadata does not encode the historical TeX style role: subfamily is `Regular`, weight is 400, `fsSelection=64`, `head.macStyle=0`, and `post.italicAngle=0`.

Historical style selection is explicit XML graph data. `wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a` declares `itVersion="wnbxti10"`. `wnbxti10.xml@de5036e7cec6e7cfdda57f63ed2be550aa04781f` binds `name="wnbxti10.ttf"`, `id="wnbxti10"`, `space="0.414441"`, `xHeight="0.444445"`, `quad="1.182211"`, `unicode="95"`, with `romanVersion="wnbx10"`, `ttVersion="wntt10"`, and `ssVersion="wnssbx10"`. Historical DefaultTeXFontParser resolves these style-version IDs while constructing FontInfo, and FontInfo lazily calls `Font::create(path)`. The variant relationship is XML/FontInfo semantics, not an inference from the filename or sfnt style bits.

## Accepted runtime boundary
Accepted `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still registers Cyrillic, but accepted `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` makes `addTeXFontDescription` throw `ex_alphabet_registration`; the registration wrapper fences the failure. The XML metrics/style graph therefore does not create an accepted-production WNBXTI10 FontInfo.

Accepted `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` may use `Formula::getExternalFont(block)` and `TextRenderingAtom` when an external mapping exists; otherwise it follows unknown-character/partial-mode handling. This is a fallback boundary, not guaranteed WNBXTI10 rendering and not permission to revive the stripped XML loader.

## Install, shipping and license
`RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` lists `cyrillic/wnbxti10.ttf`; Meson installs the full `res/cyrillic` subtree. Accepted `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` explicitly excludes Cyrillic and Greek because the XML loader is stripped, so accepted tdesktop's bundled MicroTeX resource does not ship this target.

MicroTeX README excludes fonts and XML resources under `res` from the project MIT blanket and points to `res/cyrillic`; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the subtree. `OS/2.fsType=0` is embedding metadata, not a copyright-license grant and does not override repository provenance.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Fabushi renderer runtime-assets, canonical Resource/attachment lifecycle, Markdown/text rendering and desktop packaging/release are contextual owners only; none receives a new WNBXTI10 runtime or shipping asset. No second Resource owner, font registry/store, style registry, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict228: total=16,379, unknown=16,149, unread=16,124, omitted=0, responsibilities=245.
After strict228: total=16,379, unknown=16,148, unread=16,123, omitted=0, responsibilities=246.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnbxti10.xml@de5036e7cec6e7cfdda57f63ed2be550aa04781f`.

## Parent exact-head evidence
Lease `ed8aab4a19ec53e489d41947146cf933f978a085`: TDRP Revision 9 run `40184403670` completed/success, 61/61 jobs; Rust desktop runtime `40184403653` completed/success, 6/6 jobs; Official MCP marketplace `40184403594` completed/success, 1/1 job. Authority artifact `12596055295` digest `sha256:7b83f3fd69c587826b96f4cfdd69ef84cd486b4c2c56f72fd4569c4fca4b8964`; recursive closure `12596140443` digest `sha256:912ca377faad0f5a1d27deeeac295e8889aaac1fa3d0a694274b6021df67c603`; reachability `12595866038` digest `sha256:d43410ebccd6bab4da11aec3943603dd01502bd838986191f7d5df61c9b810dd`; boost reachability `12595362411` digest `sha256:d6f4e079283c48a1d0ef94c659b04e20a1240c13f3224f48f8de512d44d562fb`; Rust messaging artifact `12596010071` digest `sha256:c03e6b79c0dd639410b4669513342aac6b762a6aaf69f17a5d2d8276d8571e9f`. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_WNBXTI10_TTF_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SFNT-METADATA-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NAME-REWRITE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-GLYPH-LAYOUT-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-STYLE-VERSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-52160-98-1000-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-ROOT-52028-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-TDESKTOP-NAME-PREFIX-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-GLYPH-TABLES-UNCHANGED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-XML-STYLE-ROLE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-FSTYPE-NOT-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NO-PARALLEL-OWNER-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`

## Complete row traceability IDs
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SFNT-METADATA-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NAME-REWRITE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-GLYPH-LAYOUT-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-STYLE-VERSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-52160-98-1000-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-ROOT-52028-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-TDESKTOP-NAME-PREFIX-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-GLYPH-TABLES-UNCHANGED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-XML-STYLE-ROLE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-FSTYPE-NOT-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NO-PARALLEL-OWNER-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-HISTORY-REWRITE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-STYLE-VERSION-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-ACCEPTED-FALLBACK-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-LICENSE-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SFNT-METADATA-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-NAME-PREFIX-LOSS-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-GLYPH-MUTATION-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-SFNT-STYLE-AS-RUNTIME-ROLE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-XML-STYLE-LINK-LOSS-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-HISTORICAL-AS-PRODUCTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-INSTALL-SHIPPING-CONFLATION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-FSTYPE-AS-LICENSE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-LICENSE-FLATTENING-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBXTI10-TTF-RUNTIME-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
