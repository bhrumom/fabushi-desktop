# desktop-app/MicroTeX res/cyrillic/mappings_cyrillic.xml complete read — strict224

## Authority
- PR #154 pre-write lease: `9730e386e1e4f2db1e6a9a8d40e45defb520dfac`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.
- Target: `res/cyrillic/mappings_cyrillic.xml@15d7f60b319e8f7ca6b6c35824d4e7c2948e11ee`, 4556 bytes / 117 line slots.
- Pre-write independent responsibility row count: 0.

## Exact mapping surface
The complete file is `FormulaSettings` with exactly 91 `CharacterToSymbolMappings` entries (91 unique chars / 91 unique symbols) and 13 `CharacterToFormulaMappings` entries over 11 unique chars. Character 1027 appears three times in source order as `\'\CYRG`, `\'\CYRG`, `\'\CYRK`. Historical character-keyed assignment is ordered last-write-wins, so character 1027 historically resolves to the third value `\'\CYRK`.

## Exact history
Exact-path history contains only root commit `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`, 2016-06-24); page 2 is empty. The root has no parent. Recursive tree `f8a661f00ff420f91227f4e2bb1a3c39405a7bdd` is `truncated=false`, 171 entries, and contains the same target blob.

## Historical parser semantics
Historical `core/formula.cpp@eea169a1dc21678040f38e977ee835f9abbc8f75` constructs `TeXFormulaSettingParser`; `TeXFormula::addSymbolMappings(file)` invokes `parseSymbol` followed by `parseSymbol2Formula`. Duplicate character-to-formula keys are therefore source-ordered assignments, not entries to discard.

## Accepted dormant/fallback boundary
Accepted `src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303` disables `Formula::addSymbolMappings(file)`: tinyxml2/XML resource loaders are removed and the function throws `ex_invalid_state`. Accepted Cyrillic registration remains and missing XML mappings fall through the external-font/`TextRenderingAtom` path. This file is historical mapping semantics plus source/install provenance, not an active accepted XML runtime.

## Install, shipping, license
`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` includes the target and `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs `res/cyrillic`. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` has 30 entries and zero Cyrillic/Greek paths, so accepted tdesktop bundled shipping excludes it. `README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes `res/cyrillic` from the MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for that subtree.

## Existing-owner-first
Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. No second Resource owner, formula-map store, XML parser, font store, license store, MicroTeX/Cyrillic runtime, install root, or packaging root is introduced.

## Negative oracles
- Preserve 91 symbol mappings and 13 formula mappings over 11 formula chars.
- Preserve all three character-1027 records and source order; historical final value is the third `\'\CYRK`.
- Do not represent the historical XML loader as accepted production.
- Preserve external-font/`TextRenderingAtom` fallback.
- Do not conflate standalone install with bundled shipping.
- Do not flatten the GPLv3 subtree boundary into the MIT blanket.
- Do not create a source-specific Fabushi mapping runtime.

## Coverage
Before strict224: total=16379, unknown=16153, unread=16128, omitted=0, responsibilities=241.
After strict224: total=16379, unknown=16152, unread=16127, omitted=0, responsibilities=242.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.

Next deterministic entry: `res/cyrillic/symbols_cyrillic.xml@80e88bf296d68d4f0d65e4b502f1a60f6281626f`.

## Parent exact-head evidence
Lease `9730e386e1e4f2db1e6a9a8d40e45defb520dfac`: TDRP Revision 9 run `40176112166` completed/success, 61/61 jobs; source authority artifact `12592131491` sha256 `fb0681b3a5b259645d29a76bf880ae23b05704593cf86474a53cbbdb5ad36716`; recursive closure `12592900732` sha256 `a699e10bd6fcde23e9a638c9d07242cb837b049e93be96e72a893e8a5abd9387`; recursive reachability `12592068727` sha256 `4020da6ddf7ee13ccbe020090d70344e3ebb8240e07f13c25bb8bbb9a23a2f64`; Boost `12592336273` sha256 `bec0375de2e7c17a5fa68f5b93e3ad78204ad01a42da3d8d244dbd7783c68065`. Rust desktop runtime run `40176112119` completed/success, 6/6 jobs; artifact `12592695450` sha256 `fa18cd3663e764caadf7cde2afb92cdba6665ac273438ee7989c0286bedd4b0f`. Official MCP marketplace run `40176112125` completed/success, 1/1 job. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors
- `TDRP_MICROTEX_RES_CYRILLIC_MAPPINGS_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-SYMBOL-MAPPINGS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-FORMULA-MAPPINGS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-DUPLICATE-ORDER-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-SHIPPING-EXCLUSION-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-91-13-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-1027-TRIPLE-ORDER-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-1027-LAST-WINS-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAPPINGS-XML-NO-PARALLEL-OWNER-001`
