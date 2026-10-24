# desktop-app/MicroTeX readme/example_keep_trying.svg complete read — strict199

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 9d3b7342fdff56c7ff677c7c1a614faa270293fc
- Pre-receipt tree: cbd1242b3cb16aeefcb99e03dd788a647b1f174e
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095.
- Target: readme/example_keep_trying.svg@e2ed234fb25c82c47f6852e0c21edfa341e0e38d, mode 100644, 20,571 bytes, 102 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target SHA/path/responsibility mention count in parity-ledger.json, inventory/index.json and upstream.lock.json: 0.
- Previous sequential source: readme/example_have_log.svg@33e769ade7b0c73864528d43ea8162df32e5cd1e.
- Next non-directory entry: readme/example_logos.svg@d0c1d1f9659ace5e73ac750c6c359215999fa5b1.

## Exact SVG provenance
The accepted Keep trying example is SVG 1.1 sized 131pt x 55pt with viewBox 0 0 131 55. It contains 15 symbol elements, 15 inline path elements, one rect, 17 group elements and 16 use elements. The 16 href/xlink:href values are all local fragments (#o,#n,#m,#l,#k,#j,#i,#a,#a,#h,#g,#f,#e,#d,#c,#b).

The exact blob contains zero script, image, text, style, foreignObject, event-handler attribute, url() reference, xml:base or external href surface. This is a passive inline documentation visual boundary for this exact blob only.

## README direct consumer and formula/render example
README.md@824af04fd8607532efed3f73752b81e5093577ef directly documents `\\int_{now}^{+\\infty} \\text{Keep trying}`. General mode passes it to `LaTeX::parse` with width 720, text size 20, line spacing 10 and black foreground. Builder mode documents the same formula with display style, size, width/alignment, max-width, line spacing and foreground before building a TeXRender.

The README then demonstrates drawing the resulting render at (10,10) through `Graphics2D_cairo`, deleting the render, and directly embeds `readme/example_keep_trying.svg` as the illustration. Historical README names such as TeXFormula/setSize/setForground are not treated as current compile acceptance: the pinned current implementation uses Formula, TexStyle::display, setTextSize and setForeground.

## LaTeX::parse runtime context-only boundary
src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2 implements current `LaTeX::parse`: mutable parser/render state is reset, lined/alignment behavior is derived, `_formula->setLaTeX(latex)` is called, the shared TeXRenderBuilder is configured, `build(*_formula)` is invoked, and the TeXRender is returned.

This is real runtime behavior but remains independently open. The documentation SVG does not close or duplicate the canonical formula/parser owner.

## TeXRenderBuilder and TeXRender draw runtime context-only boundary
src/render.cpp@e58454a430f08cefbc1330482e12c709902bc699 and src/render.h@07a4f95422198ded07023ca5bafcab9caf02913e own the current render path. `TeXRenderBuilder::build` creates the TeX font/environment, applies interline/width settings, creates the box tree, optionally performs width/line splitting and alignment, constructs a TeXRender and applies a non-transparent foreground.

`TeXRender::draw(Graphics2D&, int, int)` preserves/restores graphics color and transform through a scoped restorer, scales by text size, applies foreground/default color, then delegates to the formula box tree's draw method at inset-adjusted coordinates.

These builder/box/draw contracts are runtime context only.

## Graphics interface and backend context-only boundary
src/graphic/graphic.h@fac688576f302a5f4987b0e869f3f9baafccaf7d declares the source-neutral graphics interfaces. The fixed tree contains Cairo (graphic_cairo.cpp@16331bfdb1d69926b7e14c7276acb3df052f40a8), Qt (graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5), Win32/GDI+ (graphic_win32.cpp@054860effc94bd42c7004e3bd6523f71aee059b5) and Skia (graphic_skia.cpp@d4e4c24b9b2f6cde04b0f0821982999faa601451) implementations.

README chooses Cairo for its documented draw example. Neither the README nor exact-path history proves which executable, backend, font set or invocation generated the committed target SVG. Backend/runtime ownership remains independently open.

## Accepted tdesktop shipping negative oracle
cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds `external_microtex_bundled` from the pinned runtime. It directly compiles formula/parser support plus src/latex.cpp, src/render.cpp and src/platform/qt/graphic_qt.cpp, defines BUILD_QT, and composes res/bundled.qrc.

That manifest contains zero `example_keep_trying.svg` and zero target readme path reference. Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links `desktop-app::external_microtex`; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 carries the shipping MicroTeX composition; both contain zero direct target-asset reference.

Thus accepted shipping compiles real formula/render/Qt graphics runtime sources while bypassing this README SVG. Runtime composition is context only and does not make the target a shipping resource.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README". That commit includes README.md, this target, the sibling top-level readme examples and res/SAMPLES.tex.

This proves introduction provenance only. Exact generator invocation, executable, font selection and rendering backend remain unproven.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates `readme/samples/*` and samples.md from the sample set through the headless demo/scour path. It contains zero `example_keep_trying.svg` reference. This top-level README example is outside strict187's regeneration responsibility.

## Build, CI, bundled resource and shipping bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7, res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa and workflows ubuntu-gtk@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838. The accepted external_microtex/td_iv/Telegram shipping manifests also contain zero direct target-asset reference.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML resources under res from the blanket project MIT statement. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates the resource set; layered resource notices include Knuth_License.txt, License_for_dsrom.txt and res/greek/LICENSE.

The SVG contains path outlines but no font identity metadata. Exact-source inspection cannot prove which font/resource produced each outline or collapse resource-derived obligations into project MIT.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains the selected owner. Formula parsing/reset, Formula/Atom/Box construction, TeXRenderBuilder, TeXRender draw, graphics interfaces/backends and external_microtex/td_iv shipping are context only. No DocumentationSvgRuntime, FormulaRuntime, GraphicsRuntime or MicroTeX-specific product root is created or closed.

## Negative oracles
- Static example_keep_trying.svg is runtime implementation or release acceptance evidence: false.
- Historical README sample syntax proves current API compile compatibility: false.
- README's Graphics2D_cairo example proves the committed SVG was generated by Cairo: false.
- Same-commit history proves exact generator/font/backend: false.
- strict187 generates this top-level asset: false.
- res/bundled.qrc or accepted external_microtex packages this target SVG: false.
- Compiling latex/render/graphic_qt promotes this target into shipping composition: false.
- Project MIT settles all path-outline font/resource derivation: false.
- This row closes formula/parser/render/graphics production responsibilities, source closure or release acceptance: false.

## Coverage
Before strict199: total 16,379; unknown 16,178; unread 16,153; omitted 0; responsibilities 216.
After strict199: total 16,379; unknown 16,177; unread 16,152; omitted 0; responsibilities 217.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent 9d3b7342fdff56c7ff677c7c1a614faa270293fc was revalidated before this receipt:
- TDRP Revision 9 source authority run 40131942467: completed/success, 61/61 jobs success.
  - authority 12574734716 sha256:bd6f01cc18652a6d19b96d5692f7594a23eac1f72b609f6f5be9a514bb925824
  - recursive closure 12575915233 sha256:3a53a2e36b8235b6f40929c9712c60b5fa546e07a75ecc7a7f051d0d9e8aa8bc
  - recursive reachability 12575557680 sha256:6cdb2bb493c5e34a209fb049ab2995d5cbe10dfd6dd9b4210b7ff1f0edde0c91
  - Boost 12575502388 sha256:3c7db6b11d7ed95208432719db7f22ac38ae8798bfe4736c2e4f3895638bf0b7
- Rust desktop runtime run 40131942556: completed/success, 6/6 jobs success.
  - artifact 12575477037 sha256:7b82a5500471563ed8e78cb08905f4a298b8e22a32a01e629eec97815e45f1ec
- Official MCP marketplace run 40131942464: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-KEEP-TRYING-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_KEEP_TRYING_SVG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.

## Machine-readable anchors
- TDRP-R9-MICROTEX-README-KEEP-TRYING-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-README-FORMULA-CONSUMER-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-LATEX-PARSE-CONTEXT-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-RENDER-BUILDER-DRAW-CONTEXT-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-GRAPHICS-BACKEND-CONTEXT-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-HISTORY-GENERATOR-BACKEND-BOUNDARY-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-TDESKTOP-SHIPPING-COMPOSITION-BYPASS-001
- ORA-TDRP-MICROTEX-KEEP-TRYING-LICENSE-FONT-RESOURCE-BOUNDARY-001
- INV-TDRP-MICROTEX-KEEP-TRYING-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-KEEP-TRYING-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-KEEP-TRYING-README-BACKEND-NOT-GENERATOR-PROOF-001
- INV-TDRP-MICROTEX-KEEP-TRYING-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-KEEP-TRYING-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-KEEP-TRYING-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-KEEP-TRYING-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-KEEP-TRYING-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-KEEP-TRYING-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-KEEP-TRYING-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-GENERATOR-BACKEND-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-KEEP-TRYING-LICENSE-CONFLATION-001
