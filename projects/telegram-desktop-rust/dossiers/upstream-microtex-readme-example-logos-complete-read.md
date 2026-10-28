# desktop-app/MicroTeX readme/example_logos.svg complete read — strict200

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 1e77e558cdfab5661d51b95e05f140320ba3d74e
- Pre-receipt tree: 3ed0b42d697417a1b1dce65fb6e4423643739251
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca / tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_logos.svg@d0c1d1f9659ace5e73ac750c6c359215999fa5b1, mode 100644, 5,456 bytes, 51 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target SHA/path/responsibility mention count in parity-ledger.json, inventory/index.json and upstream.lock.json: 0.
- Previous sequential source: readme/example_keep_trying.svg@e2ed234fb25c82c47f6852e0c21edfa341e0e38d.
- Next non-directory entry: readme/example_ubuntu.png@006e26c90dd76187249b36f5987b29a2c314fa15.

## Exact SVG provenance
The accepted logo example is SVG 1.1 sized 81pt x 63pt with viewBox 0 0 81 63. It contains 7 symbol elements, 7 inline path elements, one rect, 8 group elements and 7 use elements. All 7 href/xlink:href values are local fragments. The exact blob has zero script, image, text, style, foreignObject, event-handler attribute, url() reference, xml:base or external href surface. It is a passive inline documentation visual boundary.

## README direct consumer
README.md@824af04fd8607532efed3f73752b81e5093577ef has a dedicated `\TeX and \AndroidTeX` section. It states that the TeX logo and Android-version logo are produced by `\TeX` and `\AndroidTeX`, shows both commands, and directly embeds `readme/example_logos.svg`. The SVG does not itself define macro expansion, symbol lookup, font metrics, font loading, rendering, or shipping composition.

## Predefined formula runtime context-only boundary
src/core/formula_def.cpp@ec6143102ae919689ca4fbae4ae371812598a367 defines `TeX -> \large{\dT\dE\dX}\normalsize`, `Android -> \Huge{\android}\normalsize`, and `AndroidTeX -> \Android\large{\dT\dE\dX}\normalsize`. src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 and src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c provide the normal Formula/TeXParser expansion path. These are real runtime responsibilities but remain context only for this documentation row.

## Special symbol/font runtime context-only boundary
src/res/sym/symspecial.def.cpp@c6fcc8c76f1be5af57794d768c372d3e54791689 maps `android`, `dT`, `dE` and `dX` to font id 24 codes 65/66/67/68. src/res/font/special.def.cpp@fafe0f9975056f716316948aa2cc938f3cbe28b1 defines `special` as `fonts/maths/special.ttf` and provides the matching metrics; the pinned font blob is res/fonts/maths/special.ttf@6c859a1ee00901e48d6f97691d76e2b4391e3259. builtin_syms_reg.cpp@58d5da4916a59d9374d2617181bda6d39aafcacb and builtin_font_reg.cpp@c92411673b6dfd6b7bd550a20747c5b7eba545bc register these built-ins. This is current runtime provenance, not proof that the historical SVG was generated from the current special.ttf blob.

## Accepted tdesktop shipping negative target-asset oracle
cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds `external_microtex_bundled`, directly compiles formula_def.cpp, special.def.cpp and symspecial.def.cpp plus normal parser/render/Qt graphics runtime, and composes res/bundled.qrc. res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa contains `fonts/maths/special.ttf`, so the corresponding symbol font resource genuinely ships in this accepted runtime package. It contains zero `example_logos.svg` reference.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 includes td_iv and copies external_microtex_bundled.rcc into application resources. Both contain zero target-SVG reference. Accepted shipping is therefore runtime-positive for the relevant formula/symbol/font implementation and target-asset-negative for this README SVG.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README". That commit introduced README.md, the sibling top-level example assets including example_logos.svg, example_ubuntu.png, tex_logo.svg and res/SAMPLES.tex. This establishes introduction provenance only.

No exact source proves the executable, command line, graphics backend or font snapshot used to create the committed logo SVG. Present formula/symbol/special.ttf runtime provenance cannot be backfilled as exact historical generation provenance for committed path outlines.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates only `readme/samples/*` and samples.md through the sample/demo/scour path. It contains zero `example_logos.svg` reference. This top-level asset is outside strict187's regeneration responsibility.

## Build, CI, bundled resource and shipping bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and workflows ubuntu-gtk@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838. CMakeLists.txt/QtLatex.pri do include formula/symbol/font sources, and bundled.qrc packages special.ttf but not the SVG. Runtime composition does not promote the documentation asset.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML under res from the blanket MIT statement. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates resources; Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c and OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52 demonstrate layered font licensing. The SVG contains path outlines and no font identity metadata, so current special.ttf mapping cannot settle its exact font derivation or flatten resource obligations into project MIT.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains selected. Formula/TeXParser expansion, special symbols, built-in font registration/metrics/resources, graphics backends and external_microtex/td_iv shipping are context only. No DocumentationSvgRuntime, LogoRuntime, FormulaRuntime, SymbolRuntime, FontRuntime, MicroTeX product root or parallel renderer is created or closed.

## Negative oracles
- Static example_logos.svg is runtime implementation or release acceptance evidence: false.
- README commands alone close Formula/TeXParser behavior: false.
- Current formula_def/symspecial/special.ttf proves the committed SVG generator/font: false.
- A checked runtime backend proves the committed SVG backend: false.
- Same-commit history proves exact generator/font/backend: false.
- strict187 generates this top-level asset: false.
- res/bundled.qrc packages this target SVG: false.
- Packaging special.ttf promotes the documentation SVG into shipping composition: false.
- Project MIT settles all path-outline font/resource derivation: false.
- This row closes formula/parser/symbol/font/render production responsibilities, source closure or release acceptance: false.

## Coverage
Before strict200: total 16,379; unknown 16,177; unread 16,152; omitted 0; responsibilities 217.
After strict200: total 16,379; unknown 16,176; unread 16,151; omitted 0; responsibilities 218.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent 1e77e558cdfab5661d51b95e05f140320ba3d74e was revalidated before this receipt:
- TDRP Revision 9 source authority run 40133741335: completed/success, 61/61 jobs success.
  - authority 12576645483 sha256:a874d0a2fb9995465d03012e24572c4713ca23ede752a55d67edf2e3aeaf32f9
  - recursive closure 12576346091 sha256:917a1a4de68e512421d7c5d9ba76c2a28f78ba7ad814aa21de741d8707270d1d
  - recursive reachability 12576945380 sha256:9d623d6335c22478c5d52b55673be608b9b80e2ca20e3fc11b5cea78977464a9
  - Boost 12576291246 sha256:cc528f520cdc56fbd596ff8fd4b6ace871dede92729d3e4c9f905a568246313d
- Rust desktop runtime run 40133741379: completed/success, 6/6 jobs success.
  - artifact 12576355478 sha256:15bf1d2dc3ed6961f6f52f6848e5dbe37b86a48ec4e2ab9de7678138f944fb45
- Official MCP marketplace run 40133741350: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-LOGOS-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_LOGOS_SVG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.

## Machine-readable anchors
- TDRP-R9-MICROTEX-README-LOGOS-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-LOGOS-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-LOGOS-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-LOGOS-README-MACRO-CONSUMER-001
- ORA-TDRP-MICROTEX-LOGOS-PREDEFINED-FORMULA-CONTEXT-001
- ORA-TDRP-MICROTEX-LOGOS-SPECIAL-SYMBOL-FONT-CONTEXT-001
- ORA-TDRP-MICROTEX-LOGOS-RUNTIME-CONTEXT-001
- ORA-TDRP-MICROTEX-LOGOS-HISTORY-GENERATOR-FONT-BACKEND-BOUNDARY-001
- ORA-TDRP-MICROTEX-LOGOS-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-LOGOS-TDESKTOP-SHIPPING-COMPOSITION-BYPASS-001
- ORA-TDRP-MICROTEX-LOGOS-LICENSE-FONT-RESOURCE-BOUNDARY-001
- INV-TDRP-MICROTEX-LOGOS-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-LOGOS-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-LOGOS-CURRENT-FONT-NOT-HISTORICAL-GENERATOR-PROOF-001
- INV-TDRP-MICROTEX-LOGOS-BACKEND-NOT-GENERATOR-PROOF-001
- INV-TDRP-MICROTEX-LOGOS-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-LOGOS-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-LOGOS-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-LOGOS-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-LOGOS-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-LOGOS-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-LOGOS-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-LOGOS-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-LOGOS-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-LOGOS-GENERATOR-FONT-BACKEND-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-LOGOS-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-LOGOS-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-LOGOS-LICENSE-CONFLATION-001
