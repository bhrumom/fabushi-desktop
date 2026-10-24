# desktop-app/MicroTeX readme/example_bw_true.svg complete read — strict194

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: fa47cfc02991ee858fe695fd90ffd82ec3b6128f
- Pre-receipt tree: a908ce8f39ae3848578d46f420ef0750e1e46b87
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_bw_true.svg@c7f73236f4569d8b25b9ebcc481f2372864adb45, mode 100644, 57,295 bytes, 681 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write independent responsibility count: 0. The target appears only as strict193 false-sibling context, not as an independent responsibility.
- Next non-directory entry: readme/example_cjk_trying.svg@3434f8ef08e6cb63a0fa379cf46ef07b03ca3454.

## Exact SVG provenance
The accepted true-state SVG is 499pt x 71pt with viewBox 0 0 499 71. It contains 44 symbol elements, 44 path elements and 180 use instances. All 180 href/xlink:href values are local fragment references. There are no script, image, style, text, font-face/font-family markers or external hrefs.

This establishes a passive inline vector/documentation boundary for this exact blob only. Active or external SVG content in any later blob requires a new audit.

## TRUE/FALSE sibling difference and de-duplication
The true asset is not a duplicate registration of readme/example_bw_false.svg@9e812c823d3c3be06ab43ee77f10b6122bd638dd. The sibling is a different 57,285-byte blob. Both blobs retain the same 44-symbol / 44-path / 180-use structure, and the 180 local use fragment IDs occur in the same sequence, but 58 use records differ in coordinates/layout (first differing use index 122, last 179). Their vertical use distributions also differ: true has 61/63/55/1 uses at y=19.722656/36.832031/56.820312/56.822255, while false has 61/61/57/1.

The evidence therefore records two separate documentation visual outcomes over the same glyph-reference sequence. It does not infer a generator command, a runtime state transition, or a new renderer from the static coordinate difference.

## README consumer
README.md@824af04fd8607532efed3f73752b81e5093577ef documents \\breakEverywhere, says its predefined value is false, embeds readme/example_bw_false.svg for \\breakEverywhere{false}, and directly embeds this exact true asset for \\breakEverywhere{true}.

## Context-only product semantics
The image does not own runtime behavior. macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726 registers breakEverywhere. macro_impl.h@0e6f6712dc079d4f0283ba71d659e22819dc8b6f assigns RowAtom::_breakEveywhere = args[1] == L"true"; every non-exact-lowercase-true value becomes false. atom_row.cpp@fab326690dc46e2200b52be6c4b78b02643981ee initializes the flag false. When a row is breakable, true adds a break position after every emitted child; false still permits positions after BreakMark or ASCII-digit cases. _breakable=false suppresses both branches.

These semantics are context only. The true SVG does not close formula/layout runtime responsibility, implementation, packaging, or acceptance.

## Generation/update provenance
Exact-path history through the accepted pin has one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README". That commit added README.md and the false/true assets together with other README examples.

README documents generic headless SVG generation, including single-input/output operation, but no exact command, fixture, script, CI step, commit message or other repository evidence binds that facility to this true asset. Concrete generator provenance remains unknown and is not inferred from the visual similarity or the generic headless capability.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates samples from res/SAMPLES.tex, writes readme/samples/* plus readme/samples.md, stages only those paths, and has zero example_bw_true/example_bw_false references. This top-level README asset is outside the strict187 sample-regeneration responsibility.

## Build/package/shipping bypass
Direct target references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7, ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194ac023a93acb57235635608a0d9e04, windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 and res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa. (The windows-gdi workflow exact blob is 4adace4d194d60ffc027a1b5d99fa33d200b07aa; the similarly prefixed hook SHA above is not the workflow authority.)

Direct target references are also zero in cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334, tdesktop Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 and Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277. Accepted shipping continues through external_microtex and external_microtex_bundled.rcc, not this README asset.

## License/font-derived boundary
MicroTeX's project-level MIT license does not flatten the separately licensed font/XML resource boundary documented by the project. This SVG stores path outlines but no font identity metadata. Exact source inspection cannot prove which font/resource produced every outline or relicense any resource-derived shape. Root/project MIT therefore does not close font/resource-derived redistribution provenance.

## Existing-owner-first disposition
Current Fabushi exact-head shipping owners were re-read before registration, including the React renderer/workspace/composer, Host/Runner composition, canonical search, storage/recovery, updater, packaging, Plugins/MCP/Marketplace, Computer and Automations surfaces. None makes this static third-party README asset a product runtime. The existing TDRP recursive documentation/source/license provenance governance remains the selected owner; no DocumentationSvgRuntime, MicroTeXLineBreakRuntime, second renderer, second resource root or source-specific product shell is introduced.

## Negative oracles
- Static SVG is runtime line-break implementation evidence: false.
- true and false share one exact asset/blob: false.
- Same use fragment-ID sequence means the visual outcomes are identical: false; 58 use records differ in layout coordinates.
- Static coordinate differences prove the exact generator command: false.
- true bypasses _breakable=false: false.
- strict187 pre-push generates this top-level asset: false.
- README generic headless support proves this file's exact generator: false.
- checked build/package/shipping composes this asset: false.
- project/root MIT settles all outline/font/resource derivation: false.
- this row closes formula/layout production, baseline identity, source closure or release acceptance: false.

## Coverage
Before strict194: total 16,379; unknown 16,183; unread 16,158; omitted 0; responsibilities 211.
After this one independent registration: total 16,379; unknown 16,182; unread 16,157; omitted 0; responsibilities 212.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent fa47cfc02991ee858fe695fd90ffd82ec3b6128f was revalidated before this receipt:
- TDRP Revision 9 source authority run 40122873598: completed/success, 61/61 jobs success.
  - authority 12571723049 sha256:8da0d72fce46c2363420d61d7490ae1115b23bae6e1e3eaa7089d332b09ea283
  - recursive closure 12571763444 sha256:dbeb6603f1f185e6f543dd610063181a6789d380e1caf60dc3270a8ed64425a8
  - recursive reachability 12571658549 sha256:5ad91326d6b3cb0283ff6d1846ea603b71d5d9ebe5186846a975583030048334
  - Boost 12571971723 sha256:ed9577c9179cc3fa4e8ef36a4e848685b7056317aae89fe86de9ef5852ef5cd0
- Rust desktop runtime run 40122873610: completed/success, 6/6 jobs success.
  - artifact 12571876367 sha256:d3f7eda4d4a0ea410b3e602b71bdd7f6df45e9433815bd1a9899d3913922c698
- Official MCP marketplace run 40122873642: completed/success, 1/1 job success.

These become historical parent evidence after the receipt. The receipt exact-head must be validated independently.

## Traceability
Requirement: TDRP-R9-MICROTEX-README-BREAK-EVERYWHERE-TRUE-SVG-PROVENANCE-001
Target symbol: TDRP_MICROTEX_README_BREAK_EVERYWHERE_TRUE_SVG_PROVENANCE_001
Oracles: exact source; local-only SVG references; README true-state consumer; true/false sibling difference; runtime-context boundary; history/generator boundary; strict187 de-dup; build/package zero-consumer; shipping bypass; license/font-derived boundary.
Invariants: documentation is not runtime authority; sibling evidence remains independent; target is not strict187 sample output; target is not a checked shipping resource; production responsibilities remain open.

## Machine-readable traceability anchors
- TDRP-R9-MICROTEX-README-BREAK-EVERYWHERE-TRUE-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-BW-TRUE-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-BW-TRUE-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-BW-TRUE-README-CONSUMER-001
- ORA-TDRP-MICROTEX-BW-TRUE-FALSE-SIBLING-DIFF-001
- ORA-TDRP-MICROTEX-BREAK-EVERYWHERE-RUNTIME-CONTEXT-001
- ORA-TDRP-MICROTEX-BW-TRUE-HISTORY-GENERATOR-BOUNDARY-001
- ORA-TDRP-MICROTEX-BW-TRUE-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-BW-TRUE-BUILD-PACKAGE-CONSUMER-001
- ORA-TDRP-MICROTEX-BW-TRUE-TDESKTOP-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-BW-TRUE-LICENSE-FONT-DERIVATION-BOUNDARY-001
- INV-TDRP-MICROTEX-BW-TRUE-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-BW-TRUE-SIBLING-NON-DUPLICATION-001
- INV-TDRP-MICROTEX-BW-TRUE-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-BW-TRUE-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-BW-TRUE-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-BW-TRUE-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-BW-TRUE-SIBLING-DIFF-001
- CONTRACT-TDRP-MICROTEX-BW-TRUE-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-BW-TRUE-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-BW-TRUE-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-BW-TRUE-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-BW-TRUE-SIBLING-CONFLATION-001
- FAULT-TDRP-MICROTEX-BW-TRUE-GENERATOR-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-BW-TRUE-LICENSE-CONFLATION-001
- FAULT-TDRP-MICROTEX-BW-TRUE-SHIPPING-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
