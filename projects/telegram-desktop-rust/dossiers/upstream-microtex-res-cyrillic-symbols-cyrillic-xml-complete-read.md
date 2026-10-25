# desktop-app/MicroTeX res/cyrillic/symbols_cyrillic.xml complete read — strict225

## Authority
- PR #154 pre-write lease: `f78f321a063b54a1a705266bd397089f81792b0b`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`, tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2` (`truncated=false`, 315 entries).
- Target: `res/cyrillic/symbols_cyrillic.xml@80e88bf296d68d4f0d65e4b502f1a60f6281626f`, 3577 bytes / 105 line slots.
- Pre-write independent responsibility row count: 0.

## Exact symbol surface
The complete `TeXSymbols` file contains exactly 95 `Symbol` records and 93 unique names. Type distribution is 93 `ord` and 2 `acc`: `cyrbreve` and `cyrddot` are the two accents. `CYRE` is duplicated at record positions 9 and 13, and `cyre` at positions 43 and 47; both copies of both names are `ord`.

## Exact history
Exact-path history contains only root commit `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`, 2016-06-24); page 2 is empty. The root has no parent. Recursive tree `f8a661f00ff420f91227f4e2bb1a3c39405a7bdd` is `truncated=false`, 171 entries, and contains the same target blob.

## Historical parser semantics
Historical `language_cyrillic.xml` includes `symbols_cyrillic.xml` as `TeXSymbols`. Historical `DefaultTeXFontParser::parseExtraPath()` calls `SymbolAtom::addSymbolAtom(file)`, and historical `TeXSymbolParser::readSymbols` iterates source-order `Symbol` records, validates type, and writes `res[name] = SymbolAtom(...)`. Duplicate names are therefore last-write-wins. For `CYRE` and `cyre` the later record is semantically identical because both definitions are `ord`.

## Accepted dormant/fallback boundary
Accepted `src/fonts/alphabet.cpp` still registers Cyrillic, but accepted `src/fonts/fonts.cpp` deliberately disables external XML font descriptions: `addTeXFontDescription` throws `ex_alphabet_registration`, which `addAlphabet(reg)` catches/fences. Accepted `src/res/builtin/tex_symbols.res.cpp` contains no `CYRA` or `cyrbreve` Cyrillic symbol definitions. Accepted `src/core/parser.cpp` falls unmapped non-Latin characters through to `Formula::getExternalFont(block)` and `TextRenderingAtom`. This XML therefore remains historical symbol-registry semantics plus source/install provenance, not an active accepted XML runtime.

## Install, shipping, license
`res/RES_README` lists the target and top-level Meson installs `res/cyrillic`. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` has 30 file entries and excludes Cyrillic/Greek, so accepted tdesktop bundled shipping excludes the target. MicroTeX `README.md` excludes `res/cyrillic` from the MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the subtree.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. No second Resource owner, symbol registry, XML parser, font store, license store, MicroTeX/Cyrillic runtime, install root, or packaging root is introduced.

## Coverage
Before strict225: total=16379, unknown=16152, unread=16127, omitted=0, responsibilities=242.
After strict225: total=16379, unknown=16151, unread=16126, omitted=0, responsibilities=243.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.
Next deterministic entry: `res/cyrillic/wnbx10.ttf@5a79b6d484953b3664ae4875afc06139b56a2dc0`.

## Parent exact-head evidence
Lease `f78f321a063b54a1a705266bd397089f81792b0b`: TDRP Revision 9 run `40178765618` completed/success, 61/61 jobs; Rust desktop runtime run `40178765629` completed/success, 6/6 jobs; Official MCP marketplace run `40178765627` completed/success, 1/1 job. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_SYMBOLS_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-CARDINALITY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-TYPES-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-DUPLICATE-ORDER-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-HISTORICAL-PARSER-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-SHIPPING-EXCLUSION-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-95-93-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-TYPES-93-2-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-LAST-WINS-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-SYMBOLS-XML-NOT-BUNDLED-001`
