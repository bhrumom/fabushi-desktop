# desktop-app/MicroTeX res/cyrillic/cyrillic.map.xml complete read — strict222

## Authority

- PR #154 lease / pre-receipt HEAD: f8b6d2559775c08677fb939a766c51ae893e9a65
- Live Fabushi main: 16aae12ea258f6a872a46aa47824766f64e1bdca
- Accepted/live tdesktop dev: a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Pinned/live MicroTeX master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f; recursive tree truncated=false, 315 entries.
- Target: res/cyrillic/cyrillic.map.xml@5431dc86108e722f7899fbf3db4db7af67badcc4, 5,472 bytes / 101 line slots.
- Pre-write independent responsibility row count for TDRP-R9-MICROTEX-RES-CYRILLIC-MAP-XML-PROVENANCE-001: 0.

## Complete target read

The target was read completely. It contains one SymbolMappings root with 93 SymbolMapping children. Parsing the full blob yields 93 unique names and 93 unique decimal codepoints; every entry has fontId=wnr10. There are no alternate fontId values. The set includes dotlessi, cyrbreve/cyrddot, modern Russian upper/lowercase names, Ukrainian names, and additional Slavic/Old Russian names.

Examples of the contract are CYRA -> ch=1040 -> wnr10, cyra -> ch=1072 -> wnr10, CYRYO -> ch=1025 -> wnr10, cyrbreve -> ch=774 -> wnr10, and cyrddot -> ch=776 -> wnr10. This file is a symbol-name-to-glyph/font table; it is not itself the Unicode-to-symbol table.

## Exact-path history and creation boundary

Exact-path history at the accepted pin contains one commit: 17f3fc7816e4de1f5764036f34d759d50098ae58 (2016-06-24, message init). Page 2 is empty. That commit is the repository root and has no parent, so there is no predecessor tree in which to prove absence. Its recursive tree f8a661f00ff420f91227f4e2bb1a3c39405a7bdd is truncated=false with 171 entries and contains res/cyrillic/cyrillic.map.xml at the same accepted blob 5431dc86108e722f7899fbf3db4db7af67badcc4. Therefore the target is root-created and unchanged through the accepted pin.

## Cyrillic SymbolMappings and Unicode -> wnr10 semantics

res/cyrillic/language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8 includes cyrillic.map.xml under SymbolMappings, includes eight Cyrillic metric XML files including wnr10.xml, and also includes symbols_cyrillic.xml and mappings_cyrillic.xml.

res/cyrillic/mappings_cyrillic.xml@15d7f60b319e8f7ca6b6c35824d4e7c2948e11ee supplies the Unicode-to-symbol side of the chain: for example U+0410/1040 maps to CYRA and U+0430/1072 maps to cyra. res/cyrillic/symbols_cyrillic.xml@80e88bf296d68d4f0d65e4b502f1a60f6281626f declares the corresponding TeX symbols. res/cyrillic/wnr10.xml@319d5db2d3ab2471d565e77e83dfdd29450fb106 declares Font name=wnr10.ttf id=wnr10 and carries glyph metrics/kerning for these codepoints; wnr10.ttf is a93586f1efd00c4009b72fc63a365661ce461435.

At the root-era implementation that created this XML, DefaultTeXFontParser::parseSymbolMappings in fonts/font_basic.cpp@bd97ac19f06210ccbc65b1c96a621e5e1a04c6a7 loaded each Mapping include, parsed name/ch/fontId, and stored CharFont(ch, indexOf(fontId)); CyrillicRegistration pointed at res/cyrillic/language_cyrillic.xml; TeXFormula registered CyrillicRegistration; and TeXParser lazily loaded a registered Unicode block. Thus the historical XML semantics were Unicode codepoint -> symbol name (mappings_cyrillic) -> symbol name -> CharFont(codepoint, wnr10) (cyrillic.map) -> wnr10 metrics/font.

## Accepted-fork runtime boundary

The accepted MicroTeX pin deliberately does not execute that XML loading path. src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 still registers CyrillicRegistration, and src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c still attempts lazy alphabet loading. But src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b replaces addTeXFontDescription with an ex_alphabet_registration throw and explicitly documents that Cyrillic/Greek XML packages are no longer loadable after the tinyxml2 strip. addAlphabet records the attempted block as loaded and restores _isLoading before rethrow; addAlphabet(reg) catches that exception. With the XML symbol/formula mappings absent, parser.cpp falls through to Formula::getExternalFont(block) / TextRenderingAtom.

This distinction is material: cyrillic.map.xml carries a real historical/runtime resource contract, but under the accepted production fork it is dormant source/install provenance, not an active bundled XML parser input. The audit therefore does not create or claim a Fabushi Cyrillic/MicroTeX parser runtime.

## RES_README and standalone Meson install

res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee lists cyrillic.map.xml among exactly 20 Cyrillic runtime resource entries (8 TTF + 12 XML). meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/cyrillic as a whole directory using install_subdir, so standalone MicroTeX installation includes this XML together with the rest of the subtree.

## bundled.qrc and accepted tdesktop shipping exclusion

res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa parses to 30 file entries and has zero Cyrillic and zero Greek paths. Its comments explicitly describe Cyrillic/Greek as not loaded by the bundled resource path.

Accepted tdesktop pins MicroTeX at 61aaa7cc and cmake_helpers at 7ceca3019b4750d7a5acb2de0faffd523ee14095. cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds external_microtex_bundled and target_sources only res/bundled.qrc for Qt resources. Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 calls Q_INIT_RESOURCE(bundled) off macOS or registers external_microtex_bundled.rcc on macOS, then calls tex::LaTeX::initBundled(). Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies external_microtex_bundled.rcc into the macOS app Resources, and Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 requires that file in the release bundle.

Therefore accepted tdesktop production shipping includes only the bundled qrc/.rcc resource set; cyrillic.map.xml is excluded. Standalone Meson installation and accepted tdesktop bundled shipping are separate boundaries.

## License and provenance boundary

MicroTeX README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML resources under res from the project MIT blanket and points to res/fonts/license, res/greek, and res/cyrillic for distinct licenses. res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 is the full GPLv3 text; its sole introduction commit was described as adding the license for Cyrillic fonts and XML resources. This audit records that upstream provenance boundary without inferring broader legal scope.

## Existing-owner-first audit

Current Fabushi exact-head candidates were re-read: frontend/src/production/runtime-assets.ts@470eb11f3d7f22b959008755e639a96a7f71aba0, source/electron-main/attachments/attachment-manager.ts@0d055b7d65c49911deb18ff08af2caa1639d93f7, desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84, and .github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643. No current Fabushi source search identified a math/MicroTeX formula-runtime owner that would justify adopting this dormant XML path.

Selected owner remains the existing TDRP recursive source/resource/font/parser/license/install/packaging/provenance governance. Renderer runtime-assets, attachments, and packaging/release are context owners only; none should acquire this excluded third-party XML. No second Resource owner, font store, parser, license store, MicroTeX/Cyrillic runtime, install root, or packaging root is introduced.

## Negative oracles

- Do not treat cyrillic.map.xml as the Unicode-to-symbol table; mappings_cyrillic.xml owns that direction.
- Do not lose the 93/93 unique name/codepoint and all-wnr10 invariant.
- Do not describe the historical XML loader as accepted 61aaa7cc production behavior; that loader is deliberately disabled.
- Do not omit the accepted external-font fallback after the XML load failure.
- Do not conflate standalone Meson install_subdir(res/cyrillic) with accepted tdesktop runtime shipping.
- Do not claim bundled.qrc contains Cyrillic resources; it has 30 entries and zero Cyrillic/Greek paths.
- Do not flatten the project MIT blanket over res/cyrillic.
- Do not create a source-specific Fabushi MicroTeX/Cyrillic parser or font runtime.

## Coverage

Before strict222: total 16,379; unknown 16,155; unread 16,130; omitted 0; responsibilities 239.
After strict222: total 16,379; unknown 16,154; unread 16,129; omitted 0; responsibilities 240.
baseline_identity_accepted=false, source_closure_ready=false, independent release acceptance remains not accepted.

Next deterministic non-directory entry: res/cyrillic/language_cyrillic.xml@b638f957889b97b24d1f6d98c615e4290acab1a8.

## Parent exact-head validation

Lease HEAD f8b6d2559775c08677fb939a766c51ae893e9a65 is bound to TDRP Revision 9 source authority run 40172373927 completed/success with 61/61 jobs, source-authority artifact 12591086083 sha256:8dd4fc0e725ba4c0086633ab5cefdb53a3ef0178d1ea4f79a00119c534641f9c, recursive-closure artifact 12591101348 sha256:02eab94355073e7fbca3a09d6f8b73d7642636cde295dd4574e9177ed058c1d8, recursive-reachability artifact 12590703411 sha256:702d40aaa600f00a18e5ba827c63e384c1dee7942de62d75ebb58116483fa39e, and Boost reachability artifact 12590169273 sha256:43a53a2b3c1f863f8c7556b0c693f88e56cd32e79ba32c7e80ca3187d71603f3. Rust desktop runtime run 40172373932 completed/success with 6/6 jobs and rust-messaging artifact 12590777953 sha256:149c6f9096893cc41f48c7aa442844c8e28411f60b9a5ffa97fe5d799036c230. Official MCP marketplace run 40172373934 completed/success with 1/1 jobs.

Fresh receipt-head validation is intentionally pending until this dossier/ledger/inventory/lock commit exists.

## Traceability anchors

- `TDRP_MICROTEX_RES_CYRILLIC_MAP_XML_PROVENANCE_001`
- `TDRP-R9-MICROTEX-RES-CYRILLIC-MAP-XML-PROVENANCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-EXACT-SOURCE-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-HISTORY-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-SEMANTICS-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-FALLBACK-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-INSTALL-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-SHIPPING-EXCLUSION-001`
- `ORA-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-LICENSE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-93-UNIQUE-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-WNR10-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-DORMANT-ACCEPTED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-NOT-BUNDLED-001`
- `INV-TDRP-MICROTEX-RES-CYRILLIC-MAP-XML-NO-PARALLEL-OWNER-001`
