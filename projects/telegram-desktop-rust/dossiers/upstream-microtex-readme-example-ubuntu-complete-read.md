# desktop-app/MicroTeX readme/example_ubuntu.png complete read — strict201

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: f5fa360696735248d4982c78d1b2575fb0a659fc
- Pre-receipt tree: d8298bf9f3b4e44fbaaf2152acb6fb3019c8ad51
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca / tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_ubuntu.png@006e26c90dd76187249b36f5987b29a2c314fa15, mode 100644, 227,676 bytes, PNG 1363 x 675.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target path/responsibility mention count in parity-ledger.json, inventory/index.json and upstream.lock.json: 0 / 0 / 0.
- Previous sequential source: readme/example_logos.svg@d0c1d1f9659ace5e73ac750c6c359215999fa5b1.
- Next non-directory entry: readme/example_windows.png@1d5682ae38d994aa0b4ebb1ac31d7a38b8f8581c.

## README Ubuntu/GTK screenshot provenance
README.md@824af04fd8607532efed3f73752b81e5093577ef directly states that the displayed pictures demonstrate the application running in Ubuntu using GTK and Windows, and directly embeds readme/example_ubuntu.png. The same README says GTKMM and GSVMM are required on Linux for the GTK build. This establishes the documentation meaning of the committed PNG as an Ubuntu/GTK demo screenshot, not a shipping resource or runtime owner.

The accepted PNG is a normal PNG with IHDR 1363 x 675 and a tEXt chunk `Software=gnome-screenshot`. This directly proves the committed image was captured by the GNOME Screenshot tool family. It does not encode the Ubuntu release, desktop/session type, GTK/GTKMM/GSVMM/Cairo versions, display server, DPI/scale factor, font package set, exact window geometry, command line, capture timestamp, or source executable revision.

## Three-change exact-path history
Exact-path history through the accepted MicroTeX pin has exactly three path changes:
1. ad981e7914c639d21be43f5155ee9571f0fca0b0 (2017-09-25, "fix readme") introduced the path with blob cff8f3ab084b278ff9ad72b8cbd07e5433c287a2: 253,780 bytes, 1366 x 928, Software=gnome-screenshot.
2. 01fbd49cef8eedd8133cc212794b6bec88489154 (2019-02-16, "update example in ubuntu") changed it to blob 812783a86cabe7e09c0122eac311e4a08f86a125: 256,147 bytes, 1369 x 781, Software=gnome-screenshot.
3. 63555c2e902a42bfad457d3d58a4a24c5c51c53e (2019-02-25, "new version README") changed it to accepted blob 006e26c90dd76187249b36f5987b29a2c314fa15: 227,676 bytes, 1363 x 675, Software=gnome-screenshot.

History proves three independently committed screenshot states and the GNOME Screenshot metadata in each checked blob. It does not prove which MicroTeX executable, build flags, dependency versions, formula input, user interaction sequence, GTK theme, window manager, or machine generated any state.

## GTKMM / GSVMM / gtkmm_main / Cairo runtime context-only boundary
Pinned CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 selects the UNIX GTK path when QT is not selected: BUILD_GTK, Fontconfig, GTKMM gtkmm-3.0, GSVMM gtksourceviewmm-3.0 and CairoMM cairomm-1.0; it compiles src/platform/cairo/graphic_cairo.cpp and creates LaTeXGtkSample from src/samples/gtkmm_main.cpp.

src/samples/gtkmm_main.cpp@d491af61d05fbcc5d17c48f64e76664f91d5fce8 owns a real standalone GTK sample. Its draw path constructs Graphics2D_cairo over Cairo contexts; it can save SVG surfaces and generate SVG sample outputs. src/platform/cairo/graphic_cairo.h@99ee55dd019d95e994ce201be31405510d57b035 and graphic_cairo.cpp@16331bfdb1d69926b7e14c7276acb3df052f40a8 implement the Cairo/Pango/Fontconfig graphics and text-layout backend.

These are real runtime responsibilities but remain context only for this documentation screenshot row. No exact source ties example_ubuntu.png to a specific invocation of gtkmm_main.cpp, a specific formula/input state, or any screenshot-generation command.

## Accepted external_microtex BUILD_QT shipping negative oracle
desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 defines external_microtex_bundled, explicitly sets PUBLIC BUILD_QT, compiles src/platform/qt/graphic_qt.cpp, and packages MicroTeX res/bundled.qrc. It does not compile the GTK sample, gtkmm_main.cpp or graphic_cairo.cpp and contains zero example_ubuntu.png reference.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies external_microtex_bundled.rcc into Telegram resources. The accepted tdesktop composition therefore has a runtime-positive Qt MicroTeX path but a target-asset-negative and GTK-sample-negative path. The README screenshot is not accepted shipping evidence.

## Build / CI / bundled.qrc bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa.

ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764 builds the GTK demo on Ubuntu 20.04 through Meson TARGET_DEMO=GTK and separately CMake Release/Ninja. The workflow has no screenshot capture step and no target PNG reference. windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 also contain no target reference; the Qt workflow configures QT=ON.

Therefore buildability of the GTK demo, accepted Qt shipping, and the existence of bundled.qrc do not promote this committed screenshot into runtime or packaging composition.

## Screenshot generator / capture environment provability boundary
The PNG metadata directly identifies GNOME Screenshot software. No checked exact source establishes an automated generator script, screenshot test, deterministic capture fixture, command line, golden-image pipeline, formula/input snapshot, window-state script, environment lockfile, Ubuntu image identifier, GTK/GSVMM/Cairo package versions, theme, fonts, display server, DPI, monitor geometry or timestamp for the committed PNG.

Accordingly:
- GNOME Screenshot tool-family attribution: proven.
- Ubuntu/GTK semantic meaning from README: proven.
- Exact capture environment or deterministic reproduction recipe: unproven.
- gtkmm_main / Cairo as the exact committed screenshot generator: unproven.
- Any current CI run as reproduction evidence for the historical PNG: false.

## License and resource boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README separately excludes fonts and XML under res from the blanket MIT statement, and res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee plus font notices preserve layered resource licensing.

The committed screenshot is a documentation image under the repository history, but its pixels may depict rendered fonts, GTK/theme assets and desktop chrome whose exact versions and derivation are not encoded. The root MIT statement must not be used to infer ownership/license provenance for every visually captured third-party font/theme/desktop asset. No new binary is copied into Fabushi shipping.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains selected. GTKMM/GSVMM, gtkmm_main, Cairo backend, Qt backend, external_microtex and tdesktop packaging remain context owners. No UbuntuScreenshotRuntime, GtkRuntime, CairoRuntime, MicroTeX product root, image-golden subsystem or parallel renderer is created or closed.

## Negative oracles
- README screenshot is production implementation or release acceptance evidence: false.
- gnome-screenshot metadata proves exact Ubuntu/GTK dependency versions or capture command: false.
- gtkmm_main/Cairo context proves it generated this PNG: false.
- ubuntu-gtk CI builds reproduce or validate this PNG: false.
- accepted external_microtex ships GTK/Cairo sample behavior: false; it selects BUILD_QT.
- accepted external_microtex or bundled.qrc packages example_ubuntu.png: false.
- static screenshot closes GTKMM/GSVMM/Cairo/Qt runtime responsibilities: false.
- project MIT settles all captured font/theme/desktop visual provenance: false.

## Coverage
Before strict201: total 16,379; unknown 16,176; unread 16,151; omitted 0; responsibilities 218.
After strict201: total 16,379; unknown 16,175; unread 16,150; omitted 0; responsibilities 219.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent f5fa360696735248d4982c78d1b2575fb0a659fc was revalidated before this receipt:
- TDRP Revision 9 source authority run 40134995178: completed/success, 61/61 jobs success.
  - authority 12576742490 sha256:d607615596b8f1196dd8c4b35fd31afbdd0f8448238267db321781ffb96c7308
  - recursive closure 12576837764 sha256:9ba6906416b287b4c0bae2fd41e0b98a341493ac4f5484ad495d3c604c43cd78
  - recursive reachability 12576862834 sha256:83520ebcae7b10c2c55c2eeb053f7c128603df1f44c03f27944d725bf9da0936
  - Boost 12576932327 sha256:d1e34c997a8b06a9ed3585a13e98c3452449d47660517c990f696909dddbab3b
- Rust desktop runtime run 40134995182: completed/success, 6/6 jobs success.
  - artifact 12576083487 sha256:adc45b91b83313ad94d890a29012373ae5740c1c3089adef2fc4d90866a1198f
- Official MCP marketplace run 40134995102: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-UBUNTU-PNG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_UBUNTU_PNG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.

## Machine-readable anchors
- TDRP-R9-MICROTEX-README-UBUNTU-PNG-PROVENANCE-001
- ORA-TDRP-MICROTEX-UBUNTU-PNG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-UBUNTU-README-GTK-CONSUMER-001
- ORA-TDRP-MICROTEX-UBUNTU-GNOME-SCREENSHOT-METADATA-001
- ORA-TDRP-MICROTEX-UBUNTU-THREE-STATE-HISTORY-001
- ORA-TDRP-MICROTEX-UBUNTU-GTKMM-GSVMM-CAIRO-CONTEXT-001
- ORA-TDRP-MICROTEX-UBUNTU-BUILD-QT-SHIPPING-NEGATIVE-001
- ORA-TDRP-MICROTEX-UBUNTU-CI-BUNDLE-BYPASS-001
- ORA-TDRP-MICROTEX-UBUNTU-CAPTURE-ENVIRONMENT-BOUNDARY-001
- ORA-TDRP-MICROTEX-UBUNTU-LICENSE-VISUAL-BOUNDARY-001
- INV-TDRP-MICROTEX-UBUNTU-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-UBUNTU-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-UBUNTU-GTK-CONTEXT-NOT-GENERATOR-PROOF-001
- INV-TDRP-MICROTEX-UBUNTU-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-UBUNTU-CI-NOT-SCREENSHOT-ORACLE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-UBUNTU-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-UBUNTU-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-UBUNTU-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-UBUNTU-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-UBUNTU-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-UBUNTU-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-UBUNTU-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-UBUNTU-CAPTURE-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-UBUNTU-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-UBUNTU-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-UBUNTU-LICENSE-CONFLATION-001
