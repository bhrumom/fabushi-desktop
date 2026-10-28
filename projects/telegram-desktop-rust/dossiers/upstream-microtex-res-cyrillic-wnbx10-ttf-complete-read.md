# desktop-app/MicroTeX res/cyrillic/wnbx10.ttf complete read — strict226

## Authority
- PR #154 pre-write lease: `03cddd03d9e2317c57703804bc0d10180ae05b3a`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/wnbx10.ttf@5a79b6d484953b3664ae4875afc06139b56a2dc0`, 31,940 bytes.
- Pre-write independent responsibility row count: 0.

## Exact sfnt surface
The target is a TrueType sfnt with 15 tables: DSIG, GDEF, GPOS, GSUB, OS/2, cmap, gasp, glyf, head, hhea, hmtx, loca, maxp, name and post. `maxp.numGlyphs=98`, `head.unitsPerEm=1000`, and `OS/2.fsType=0`. Windows/Unicode name records report family/full `TDesktop wnbx10`, subfamily `Regular`, unique/PostScript `TDesktop-wnbx10`, and `Version 1.001 December 29, 2009`. Name ID 0 says the font was generated from a MetaFont bitmap by mftrace 1.2.5. There are no name ID 13 or 14 license-description/license-URL records.

## Exact history and namespace rewrite
Exact-path history page 1 contains exactly root `17f3fc7816e4de1f5764036f34d759d50098ae58` and `c1546216f32d398dca90e1dc33df34842683ec42` (`Attempt to force our fonts.`); page 2 is empty. Root tree `f8a661f00ff420f91227f4e2bb1a3c39405a7bdd` is `truncated=false`/171 and holds blob `436ff077793a52495d39927068f43e09dd35d394`, 31,824 bytes. c154 tree `c2dce56581dba8a9d129aebbface4c7189b777fb` is `truncated=false`/315 and holds the current blob; c154 parent `43904b5cbae8154f041209eea6d13f7c8c4ffd6e` still holds the root blob.

Root/current sfnt table directories differ only in `name` and `head`: name grows 606 -> 723 bytes and family/full/unique/PostScript names are prefixed with `TDesktop`; `head` changes checksum adjustment. maxp/glyf/cmap/GDEF/GPOS/GSUB and the remaining table records retain their checksums/lengths. The same c154 commit explains that same-named system fonts could shadow bundled families and make glyphs disappear, and adds Qt resolved-family diagnostics. Thus c154 is a family namespace/checksum rewrite, not a glyph/layout rewrite.

## Historical font runtime semantics
`language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8` includes `wnbx10.xml` among eight Metrics. `wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a` binds `name="wnbx10.ttf"`, `id="wnbx10"`, `unicode="95"` and style variants. Historical `DefaultTeXFontParser::parseFontDescriptions` resolves the Metrics include, builds a TTF path beside the XML and constructs `FontInfo(path,...)`; historical `FontInfo::getFont()` lazily calls `Font::create(_path,...)`. The binary therefore carried a real historical runtime font-loading responsibility.

## Accepted runtime boundary
Accepted `alphabet.cpp` still registers Cyrillic, but accepted `fonts.cpp` makes `addTeXFontDescription` throw `ex_alphabet_registration` because the tinyxml2-backed XML font-description loader is removed. `addAlphabet(reg)` catches/fences it. Parser behavior then uses `Formula::getExternalFont(block)` / `TextRenderingAtom` fallback rather than installing the WNBX10 `FontInfo`.

Accepted Qt `Font_qt` still supports `QFontDatabase::addApplicationFont` from disk or `:/` and verifies resolved family, but this capability does not make WNBX10 reachable: accepted Cyrillic XML registration is disabled and accepted `bundled.qrc` contains no Cyrillic font.

## Install, shipping and license
`RES_README` lists `cyrillic/wnbx10.ttf`; Meson installs the whole `res/cyrillic` subtree. `bundled.qrc@303fb6bd...` explicitly excludes Cyrillic/Greek. tdesktop registers/copies `external_microtex_bundled.rcc` and calls `tex::LaTeX::initBundled()`; because the qrc excludes the target, WNBX10 is standalone-install provenance, not accepted tdesktop production-bundle content.

MicroTeX README excludes fonts/XML under `res/cyrillic` from its MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the subtree. The TTF has no name ID 13/14 license fields. `OS/2.fsType=0` is embedding metadata, not a copyright-license grant and does not override repository provenance.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Renderer runtime-assets, resource/attachment lifecycle and desktop packaging/release are contextual canonical owners only; none receives a new WNBX10 shipping asset. No second Resource owner, font registry/store, license store, MicroTeX/Cyrillic runtime, install root or packaging root is introduced.

## Coverage
Before strict226: total=16,379, unknown=16,151, unread=16,126, omitted=0, responsibilities=243.
After strict226: total=16,379, unknown=16,150, unread=16,125, omitted=0, responsibilities=244.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnbx10.xml@3e42b024a4d96f96a650babd183e58c31a02d31a`.

## Parent exact-head evidence
Lease `03cddd03d9e2317c57703804bc0d10180ae05b3a`: TDRP Revision 9 run `40180703488` completed/success, 61/61 jobs; Rust desktop runtime `40180703490` completed/success, 6/6 jobs; Official MCP marketplace `40180703500` completed/success, 1/1 job. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_WNBX10_TTF_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-WNBX10-TTF-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-SFNT-METADATA-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-NAME-REWRITE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORICAL-FONT-BINDING-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-ACCEPTED-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-31940-98-1000-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-ROOT-31824-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-TDESKTOP-NAME-PREFIX-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-GLYPH-TABLES-UNCHANGED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-FSTYPE-NOT-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-NO-PARALLEL-OWNER-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-PINNED-SOURCE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORY-REWRITE-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORICAL-FONT-BINDING-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-ACCEPTED-FALLBACK-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-SHIPPING-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-LICENSE-BOUNDARY-001`
- `CONTRACT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-EXACT-HEAD-EVIDENCE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-SOURCE-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORY-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-SFNT-METADATA-DRIFT-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-NAME-PREFIX-LOSS-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-GLYPH-MUTATION-MISATTRIBUTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-HISTORICAL-AS-PRODUCTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-UNREACHABLE-DIRECT-LOAD-ASSUMPTION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-INSTALL-SHIPPING-CONFLATION-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-FSTYPE-AS-LICENSE-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-LICENSE-FLATTENING-001`
- `FAULT-TDRP-MICROTEX-RES-CYRILLIC-WNBX10-TTF-RUNTIME-PROMOTION-001`
- `G-FILE`
- `G-TRACEABILITY`
- `G-EVIDENCE`
