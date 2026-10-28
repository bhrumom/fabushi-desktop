# desktop-app/MicroTeX example/latex2png.cpp complete read — strict185

## Authority
- Parent PR #154 / branch exact HEAD before receipt: \`12b2e83954cc1605d680ff6ade507047949a1384\`, tree \`b5f459d337712ded85527d0f3c45c7e760b6bcfc\`.
- Live Fabushi main: \`16aae12ea258f6a872a46aa47824766f64e1bdca\`, tree \`82126ec4a712fca352a49d6a3caae70c7d21ca8c\`.
- Accepted tdesktop: \`a7f453612f4896ac04d0007a36ab0d377e5bcbcf\`, tree \`c539eabfb05e7dcaa84821ac44c35cdfb6551ae5\`.
- Pinned MicroTeX: \`61aaa7cc354de91d5898ffb0b2a6c62628d9a76f\`, tree \`462d2794c73bb87f57d9426621bc6ce0f57c5fc2\`; recursive tree truncated=false, 315 entries / 279 blobs.
- Pinned cmake_helpers: \`7ceca3019b4750d7a5acb2de0faffd523ee14095\`, tree \`f5c386cb34208309aae521187611fac4a2045577\`.
- Target: \`example/latex2png.cpp@49d80c656e8437e56cf50cccbad451fa2ae46c91\`, 76 line slots / 1,733 characters.
- Pre-write parity-ledger independent registration count: zero; strict184 mentioned it only as context-read and explicitly left it unread/unregistered.

## Complete source behavior
The optional CLI constructs QGuiApplication, then TexGuard calls LaTeX::init() and guarantees LaTeX::release() on ordinary return/C++ unwinding. BUILD_SKIA additionally calls initGL(). It requires argc==3 (else exit 1), logs argv, appends .png when absent, checks input existence (exit 2), opens read-only (exit 3), reads the whole file, and parses with width 600, text size 20, line space 20/3.f and foreground 0xff424242. It allocates QPixmap from TeXRender width/height, fills white, enables QPainter antialiasing, wraps QPainter in Graphics2D_qt, draws at 0,0, and maps QPixmap::save(false) to exit 4; success returns 0. It does not run app.exec().

## Direct call boundary
- \`src/latex.h@0608e92d3b4821fb8cf5c402014f31c1692f014f\`: lifecycle/parse declarations.
- \`src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2\`: resource initialization, per-parse global-state reset, builder/alignment configuration and release.
- \`src/platform/qt/graphic_qt.h@777faa8d7dd18695fb4b42f4a2992c4f913c79d4\`: QPainter-backed Graphics2D_qt contract.
- \`src/render.cpp@e58454a430f08cefbc1330482e12c709902bc699\`: saturating raster sizes and draw-time graphics reset/color restoration.
- \`example/CMakeLists.txt@cd997be0b043913d17c49c1e1e2c50df2c5d6942\`: optional latex2png target linked to local LaTeX plus Qt Core/Gui.
- root \`CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9\`: BUILD_EXAMPLE defaults OFF.

The sample has no own formula-length/image-memory cap, no device-pixel-ratio policy, no fallback/cache, and no explicit parse/draw exception translation. Those absences are provenance, not product requirements.

## Accepted tdesktop shipping separation
Accepted tdesktop does not launch/package latex2png. \`desktop-app/cmake_helpers:external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334\` directly compiles the pinned fork and bundled resources as external_microtex_bundled. \`Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30\` links desktop-app::external_microtex. Shipping \`iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2\` uses bundled-resource initialization, initBundled(), input/render/physical-image caps, exception containment and DPR-aware QImage rendering. \`iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea\` owns normalization/fallback/cache. \`Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277\` includes/links td_iv and packages external_microtex_bundled.rcc. These production responsibilities remain independently open.

## Failure and negative oracles
- Wrong argc -> 1; missing input -> 2; read-only open failure -> 3; QPixmap save false -> 4.
- Whole-file read has no explicit size bound; parse/draw exceptions are not converted to those exit codes.
- BUILD_EXAMPLE defaults OFF.
- QGuiApplication/TexGuard are sample bootstrap/lifecycle, not Fabushi owners.
- Fixed width/text/line-space/color and white QPixmap are sample defaults only.
- No MicroTeXExampleRuntime, Latex2PngService, Telegram renderer root, second application shell, second image pipeline or second build graph is introduced.

## License
README.md@824af04fd8607532efed3f73752b81e5093577ef documents standalone init/parse/render use. LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT; README keeps fonts/XML resources under separate licenses.

## Coverage
Before: source_entries=16379 unknown=16192 unread=16167 omitted=0 responsibilities=202.
After: source_entries=16379 unknown=16191 unread=16166 omitted=0 responsibilities=203.
baseline_identity_accepted=false; source_closure_ready=false; independent release acceptance not accepted; production MicroTeX responsibilities remain open.

## Parent exact-head evidence
Parent \`12b2e83954cc1605d680ff6ade507047949a1384\`: TDRP Revision 9 run 40106609346 terminal success 61/61; authority 12565055929 sha256:82d3588d769150942ce963df104c50a166371c97667f35b77f6dcab9d988b0e8; closure 12564838478 sha256:67b2c80dd301dbb2990baddd19e3a0c81da7fcc7a421e6cacf04c3a009d647d9; reachability 12565146856 sha256:62c3bf945a8704ed94e3d9184b73bf565319a3299ca77bb2cca234c400cc7b8a; Boost 12565191571 sha256:3d6507e4eb0b7535ad90c6ee821edf45bcdd27acb795654f0238d0bd9e58573f. Rust desktop runtime 40106609299 terminal success 6/6; artifact 12565500319 sha256:8eaa9f8c627521edfae7dc63bbfa93db4961504f6b0b7d66621add8e092845d6. Official MCP marketplace 40106609306 terminal success 1/1. Receipt-head Actions are required.

## Traceability IDs
- CAP-DEPENDENCY-SOURCE-PROVENANCE
- CAP-DEPENDENCY-OPTIONAL-EXAMPLE-PROVENANCE
- CAP-DEPENDENCY-QT-RENDER-PROVENANCE
- TDRP-R9-MICROTEX-EXAMPLE-LATEX2PNG-PROVENANCE-001
- TDRP_MICROTEX_EXAMPLE_LATEX2PNG_PROVENANCE_001
- ORA-TDRP-MICROTEX-LATEX2PNG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-LATEX2PNG-CLI-FAILURES-001
- ORA-TDRP-MICROTEX-LATEX2PNG-RENDER-BOUNDARY-001
- ORA-TDRP-MICROTEX-LATEX2PNG-SHIPPING-SEPARATION-001
- INV-TDRP-MICROTEX-LATEX2PNG-NO-PARALLEL-RUNTIME-001
- INV-TDRP-MICROTEX-LATEX2PNG-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- INV-TDRP-MICROTEX-LATEX2PNG-FAILURE-SEMANTICS-SAMPLE-ONLY-001
- CONTRACT-TDRP-MICROTEX-LATEX2PNG-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-LATEX2PNG-OPTIONAL-BUILD-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-LATEX2PNG-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-LATEX2PNG-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-LATEX2PNG-EXAMPLE-PROMOTION-001
- FAULT-TDRP-MICROTEX-LATEX2PNG-PRODUCT-PROMOTION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
