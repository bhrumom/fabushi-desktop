# desktop-app/MicroTeX readme/example_custom_symbols.svg complete read — strict196

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: b21696b41d33e0666e0151f47fecf24041120870
- Pre-receipt tree: bdde96e8137611c6a4437f94c88e9b8d5a5c3978
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_custom_symbols.svg@45888ac3f5cf19100b9c4096eb8d38bb29d0419f, mode 100644, 9,078 bytes, 93 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target ledger mention count: 0; independent responsibility count: 0.
- Previous sequential source: readme/example_cjk_trying.svg@3434f8ef08e6cb63a0fa379cf46ef07b03ca3454 (strict195).
- Next non-directory entry: readme/example_debug.svg@7253c8c54e66c98bb95e2c2174e064cd05825662.

## Exact SVG provenance
The accepted custom-symbol example is an SVG 1.1 document sized 257pt x 33pt with viewBox 0 0 257 33. It contains 8 symbol elements, 8 inline path elements and 20 use elements. The href/xlink:href sequence is #d, #c, #h, #b, #a, #d, #c, #g, #b, #a, #d, #c, #f, #b, #a, #e, #d, #c, #b and #a; every reference is a local fragment. The exact blob contains zero script, image, text, style or external href surface and no font identity metadata.

This establishes a passive inline vector/documentation boundary for this exact blob only. Any later active/external SVG content or blob drift requires a fresh audit.

## README direct consumer and Custom symbols documentation context
README.md@824af04fd8607532efed3f73752b81e5093577ef has a dedicated "Custom symbols" section. It states there are four custom symbols, shows the exact script lines `AB \\varparallel CD`, `AB \\nvarparallel CD`, `AB \\varparalleleq CD`, and `\\parallelogram ABCD`, then directly embeds readme/example_custom_symbols.svg.

The documentation visual therefore belongs to custom-symbol/source provenance. It does not establish a second formula engine, a source-specific symbol runtime, or production equivalence.

## Runtime symbol/macro context-only boundary
The runtime sources are context only and remain independently open.

- src/res/builtin/tex_symbols.res.cpp@8ce76d17a0b995cf745601a82e569dc96ab31a30 defines the built-in SymbolAtom map and registers varparallel, nvarparallel, varparalleleq and parallelogram as `sym(ord, ...)`, i.e. ordinary SymbolAtom entries.
- src/res/sym/symspecial.def.cpp@c6fcc8c76f1be5af57794d768c372d3e54791689 maps varparallel/nvarparallel/varparalleleq to font 8 codes 197/198/199 and parallelogram to font 24 code 70.
- src/res/reg/builtin_syms_reg.cpp@58d5da4916a59d9374d2617181bda6d39aafcacb composes the base/amssymb/amsfonts/stmaryrd/special registration sets. src/res/symbol_def.res.h@e8eaf98d54421462312f78ddd0df5d539e32d397 pushes components through DefaultTeXFont::__push_symbols, and src/fonts/fonts.cpp@21ab54c071a72db3d13ecfb89561bc8ffa149c5b registers SymbolsSetBuiltin during font initialization.
- src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c processEscape checks MacroInfo first, then predefined Formula, then SymbolAtom::get. The four README names are built-in symbols, not separate MacroInfo command owners.
- src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2 initializes the font/formula/macro runtime. That initialization is runtime context, not evidence that this SVG is a runtime resource.

These reads explain the documented symbols without promoting the visual asset to runtime implementation evidence. Formula parsing, symbol registration, font lookup/rendering, resource lifecycle and production equivalence remain separate responsibilities.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README".

That same-commit relationship records introduction provenance but does not identify the exact generator invocation, executable, font selection, rendering backend, script or CI step that produced this blob. Concrete generation provenance remains unknown and is not inferred from generic rendering support.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates default res/SAMPLES.tex through the headless demo, builds samples.md, compresses SVGs with scour into readme/samples/*, and stages only readme/samples plus samples.md. It contains zero example_custom_symbols reference. This top-level README example is outside strict187's sample-regeneration responsibility.

## Build/package/shipping bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7, res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa, ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838.

The shipping path is distinct: cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 compiles tex_symbols.res.cpp, symspecial.def.cpp and the rest of external_microtex_bundled and packages res/bundled.qrc; tdesktop Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 composes td_iv and copies external_microtex_bundled.rcc. That proves runtime symbol code is separately composed while this README SVG itself bypasses shipping.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML resources under res from the blanket project MIT statement and points to their separate licenses. This SVG contains path outlines but no font identity metadata. Exact-source inspection therefore cannot prove which font/resource produced each outline or collapse resource-derived redistribution obligations into project MIT. Font/resource-derived provenance remains independently open.

## Existing-owner-first disposition
This asset is source-only documentation visual provenance. Existing TDRP recursive documentation/source/license provenance governance remains the selected owner. Formula/SymbolAtom/parser/font runtime and pinned external_microtex/td_iv shipping owners are context only. No CustomSymbolRuntime, DocumentationSvgRuntime, MicroTeXRuntime, second renderer, second resource root or source-specific product shell is introduced.

## Negative oracles
- Static custom-symbol SVG is formula/symbol runtime implementation evidence: false.
- README proves the complete parser/font/render caller chain: false.
- varparallel/nvarparallel/varparalleleq/parallelogram are standalone MacroInfo command owners: false; the pinned source registers them as ordinary SymbolAtom entries.
- symspecial font/code mappings make the README SVG a runtime resource: false.
- Generic rendering support or same-commit history proves the exact generator/font: false.
- strict187 pre-push generates this top-level asset: false.
- Checked MicroTeX/tdesktop build or shipping composition includes this asset: false.
- Project MIT settles all path-outline/font/resource derivation: false.
- This row closes formula parsing, symbol registration, font rendering, resource lifecycle, source closure or release acceptance: false.

## Coverage
Before strict196: total 16,379; unknown 16,181; unread 16,156; omitted 0; responsibilities 213.
After this one independent registration: total 16,379; unknown 16,180; unread 16,155; omitted 0; responsibilities 214.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent b21696b41d33e0666e0151f47fecf24041120870 was revalidated before this receipt:
- TDRP Revision 9 source authority run 40126294914: completed/success, 61/61 jobs success.
  - authority 12573307414 sha256:5871e70d455a47ed24dfb524601f28cda984aceb2ce9a7d7fac947c7128d8297
  - recursive closure 12573147926 sha256:337704e98a87093501b9d0d09971adb725b20fdb888694c6d38cd0e7ddd28eba
  - recursive reachability 12573437884 sha256:25a8765e0b2d79f18b114c74c321d61cf7f84a534e5004cc5e37e6b6ea2133dc
  - Boost 12573752331 sha256:d36921e69dd03ab82ab555b8664cb107730c2017e11f388539b8ad8bc86c0dbc
- Rust desktop runtime run 40126294907: completed/success, 6/6 jobs success.
  - artifact 12573447103 sha256:e26234ceb003ec33d8c3d9737ec89f48ca8acd0b74257165ab05aa0ee54ea557
- Official MCP marketplace run 40126294916: completed/success, 1/1 job success.

These become historical parent evidence after the receipt. The receipt exact-head must be validated independently.

## Traceability
Requirement: TDRP-R9-MICROTEX-README-CUSTOM-SYMBOLS-SVG-PROVENANCE-001
Target symbol: TDRP_MICROTEX_README_CUSTOM_SYMBOLS_SVG_PROVENANCE_001
Oracles: exact source; local-only SVG references; README direct custom-symbol consumer; built-in SymbolAtom/font-code/parser runtime-context boundary; exact-path history/generator boundary; strict187 de-dup; build/package zero-consumer; shipping bypass; license/font/resource-derived boundary.
Invariants: documentation is not runtime authority; built-in symbols are not new macro/runtime owners; target is not strict187 sample output; target is not a shipping resource; production responsibilities remain open.

## Machine-readable traceability anchors
- TDRP-R9-MICROTEX-README-CUSTOM-SYMBOLS-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-README-CONSUMER-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-BUILTIN-SYMBOL-CONTEXT-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-PARSER-CONTEXT-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-HISTORY-GENERATOR-BOUNDARY-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-BUILD-PACKAGE-CONSUMER-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-TDESKTOP-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-CUSTOM-SYMBOLS-LICENSE-FONT-RESOURCE-BOUNDARY-001
- INV-TDRP-MICROTEX-CUSTOM-SYMBOLS-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-CUSTOM-SYMBOLS-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-CUSTOM-SYMBOLS-SYMBOLS-NOT-MACRO-OWNERS-001
- INV-TDRP-MICROTEX-CUSTOM-SYMBOLS-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-CUSTOM-SYMBOLS-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-CUSTOM-SYMBOLS-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-CUSTOM-SYMBOLS-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-CUSTOM-SYMBOLS-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-CUSTOM-SYMBOLS-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-CUSTOM-SYMBOLS-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-GENERATOR-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-MACRO-CONFLATION-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-LICENSE-CONFLATION-001
- FAULT-TDRP-MICROTEX-CUSTOM-SYMBOLS-SHIPPING-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
