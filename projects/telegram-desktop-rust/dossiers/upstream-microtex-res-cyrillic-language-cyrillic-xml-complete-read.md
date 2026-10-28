# desktop-app/MicroTeX res/cyrillic/language_cyrillic.xml complete read — strict223

## Authority

- PR #154 pre-write lease: `143d215007dec47c432a2d868263b38208fa0d47`.
- Accepted tdesktop: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Pinned MicroTeX: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.
- Target: `res/cyrillic/language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8`, 579 bytes / 28 line slots.
- Pre-write independent responsibility row count: 0.

## Exact manifest

The complete file has one `TeXFont` root with exactly: one `SymbolMappings` include (`cyrillic.map.xml`); eight `Metrics` includes (`wnr10.xml`, `wnti10.xml`, `wntt10.xml`, `wnss10.xml`, `wnssi10.xml`, `wnssbx10.xml`, `wnbx10.xml`, `wnbxti10.xml`); one `TeXSymbols` include (`symbols_cyrillic.xml`); and one `FormulaSettings` include (`mappings_cyrillic.xml`). It is the package/orchestration manifest, not the Unicode map, symbol glyph table, or metric table itself.

## Exact history

Exact-path history contains only root commit `17f3fc7816e4de1f5764036f34d759d50098ae58` (`init`, 2016-06-24); page 2 is empty. The root has no parent. Recursive tree `f8a661f00ff420f91227f4e2bb1a3c39405a7bdd` is `truncated=false`, 171 entries, and contains the same target blob. The target is root-created and unchanged through the accepted pin.

## Historical TeXFont loader semantics

Root `fonts/font_basic.cpp@bd97ac19f06210ccbc65b1c96a621e5e1a04c6a7` shows `parseFontDescriptions` iterating all eight `Metrics` includes and registering font metrics; `parseSymbolMappings` loading `cyrillic.map.xml` and converting name/ch/fontId to `CharFont`; and `parseExtraPath` sending `TeXSymbols` to `SymbolAtom::addSymbolAtom` and `FormulaSettings` to `TeXFormula::addSymbolMappings`. Root `core/formula.cpp@eea169a1dc21678040f38e977ee835f9abbc8f75` registers Cyrillic and loads formula mappings; root `core/parser.cpp@92c5010d144f51f4df6f83a93c6da9bf0a5813d2` lazily loads registered Unicode blocks.

The manifest coordinates separate responsibilities: `cyrillic.map.xml@5431dc86108e722f7899fbf3db4db7af67badcc4`, `mappings_cyrillic.xml@15d7f60b319e8f7ca6b6c35824d4e7c2948e11ee`, `symbols_cyrillic.xml@80e88bf296d68d4f0d65e4b502f1a60f6281626f`, and eight metric XMLs: `wnr10@319d5db2d3ab2471d565e77e83dfdd29450fb106`, `wnti10@55046e15cbd13b5ceaed2ea6955cd378f6167599`, `wntt10@4baea371bade60de759650135b4773fbf2f28ee2`, `wnss10@660c29af27ddab8188fcc7594bae0941cc45d9f6`, `wnssi10@5c2318c66e402f47c05ffb67015bcff0780523a8`, `wnssbx10@9fefc1c78c8b4bb1406173b8ac491c11f030a076`, `wnbx10@3e42b024a4d96f96a650babd183e58c31a02d31a`, `wnbxti10@de5036e7cec6e7cfdda57f63ed2be550aa04781f`.

## Accepted dormant/fallback boundary

Accepted `src/fonts/alphabet.cpp@e9604b6ac6e73d2a2f6017f4164675a2121cf891` still points `CyrillicRegistration` at `cyrillic/language_cyrillic.xml`. Accepted `src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b` disables the tinyxml2-backed loader: `addTeXFontDescription` throws `ex_alphabet_registration`; `addAlphabet` records the attempted block as loaded and restores its loading fence; `addAlphabet(reg)` catches that exception. Accepted `src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303` keeps registration but disables `Formula::addSymbolMappings(file)`. Accepted `src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c` therefore falls through to `Formula::getExternalFont(block)` / `TextRenderingAtom` when XML mappings are absent.

Thus this manifest is historical package semantics plus source/install provenance, not an active accepted production XML runtime.

## Install, shipping, license

`res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee` includes the target among 20 Cyrillic resources. `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs the entire `res/cyrillic` subtree. `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` contains 30 entries and zero Cyrillic/Greek paths, and accepted tdesktop ships/registers that bundled qrc/.rcc path, so the target is excluded from production shipping.

`README.md@824af04fd8607532efed3f73752b81e5093577ef` excludes `res/cyrillic` XML/font resources from the project MIT blanket; `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` is GPLv3 provenance for the Cyrillic resource subtree. No broader legal scope is inferred.

## Existing-owner-first

Selected owner remains existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Current Fabushi runtime-assets, attachments, packaging and release owners are context only. No second Resource owner, font store, XML parser, license store, MicroTeX/Cyrillic runtime, install root, or packaging root is introduced.

## Negative oracles

- Keep exact 1/8/1/1 include cardinality.
- Do not conflate the manifest with its mapping/symbol/metric children.
- Do not claim the historical XML loader is active in accepted `61aaa7cc`.
- Preserve the accepted external-font/TextRenderingAtom fallback boundary.
- Do not conflate standalone Meson install with accepted tdesktop bundled shipping.
- Do not claim bundled.qrc contains Cyrillic/Greek resources.
- Do not flatten the MIT blanket over `res/cyrillic`.
- Do not create a source-specific Fabushi MicroTeX/Cyrillic runtime or parallel canonical owner.

## Coverage

Before strict223: total=16379, unknown=16154, unread=16129, omitted=0, responsibilities=240.
After strict223: total=16379, unknown=16153, unread=16128, omitted=0, responsibilities=241.
`baseline_identity_accepted=false`; `source_closure_ready=false`; independent release acceptance remains not accepted.

Next deterministic entry: `res/cyrillic/mappings_cyrillic.xml@15d7f60b319e8f7ca6b6c35824d4e7c2948e11ee`.

## Parent exact-head evidence

Lease `143d215007dec47c432a2d868263b38208fa0d47`: TDRP Revision 9 run `40174349155` completed/success, 61/61 jobs; source authority artifact `12591657083` sha256 `0dd4355f49aa3584b8809dfc48208c5b95da952cb8699037cefe3ff5434d835b`; recursive closure `12591612450` sha256 `432072ffe76eb1990b1d39a6ca2f9177f1eb84f5760292c1e00c63d6db90b99e`; recursive reachability `12591917016` sha256 `db6698cf8faf926c1f6baa3c4cd6f52494b94659885d24b718202ff4db4e742e`; Boost `12591607244` sha256 `ab29ce655c0cb2a2354ca6db3b52ec8de7cec7fb5f6162ca0bbd4918011a76c7`. Rust desktop runtime `40174349139` completed/success, 6/6 jobs, artifact `12591033824` sha256 `7562f14936a7e80bccce21cd86082cae01fd8f0b09d28c047c3f4c617ebef67d`. Official MCP marketplace `40174349146` completed/success, 1/1 job. Fresh receipt-head validation remains pending until this audit commit exists.

## Traceability anchors

- `TDRP_MICROTEX_RES_CYRILLIC_LANGUAGE_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-MANIFEST-SEMANTICS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-1-8-1-1-MANIFEST-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-ROOT-CREATED-UNCHANGED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-LANGUAGE-XML-NO-PARALLEL-OWNER-001`
