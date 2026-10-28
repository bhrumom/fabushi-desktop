# desktop-app/MicroTeX readme/example_windows.png complete read — strict202

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: e1945db2693742c9eac23086a5a79e93e9aa6ef3
- Pre-receipt tree: 7a2d06f7715ec9bfe86c3828607a2817c6aa075c
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_windows.png@1d5682ae38d994aa0b4ebb1ac31d7a38b8f8581c, mode 100644, 77,867 bytes, PNG 1151 x 711.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target path/responsibility mention count in parity-ledger.json, inventory/index.json and upstream.lock.json: 0 / 0 / 0.
- Previous sequential source: readme/example_ubuntu.png@006e26c90dd76187249b36f5987b29a2c314fa15.
- Next non-directory entry: readme/samples.md@4eb0be8a26e15f098156040366683f4e835631f6.

## README Windows screenshot provenance
README.md@824af04fd8607532efed3f73752b81e5093577ef directly states that the displayed pictures demonstrate the application running in Ubuntu using GTK and Windows, and directly embeds readme/example_windows.png. The same README says Gdiplus is required for the Windows demo and separately documents the optional -DQT=ON build mode.

The accepted PNG is a normal PNG with IHDR 1151 x 711 and no tEXt, iTXt or zTXt chunks. Unlike the Ubuntu screenshot, the file does not identify a screenshot utility or any capture environment. This proves only the committed raster identity and README Windows-demo meaning. It does not establish Windows release/edition/build, screenshot software, GDI+ or Qt as the capture backend, compiler/toolchain, DPI/scale factor, desktop theme, font package set, exact window geometry, formula/input state, command line, capture timestamp or machine.

## Same-blob rename-followed history
The current lowercase exact-path query reaches 01fbd49cef8eedd8133cc212794b6bec88489154, whose GitHub file record is a pure rename from readme/example_windows.PNG with zero additions/deletions. Following previous_filename closes the target lineage:
1. c125f3f623b76d3555290f622aada29139d6e4d4 (2016-07-13, "update README.md") added readme/example.PNG with blob 1d5682ae38d994aa0b4ebb1ac31d7a38b8f8581c.
2. ad981e7914c639d21be43f5155ee9571f0fca0b0 (2017-09-25, "fix readme") renamed readme/example.PNG to readme/example_windows.PNG with the same blob and no content change.
3. 01fbd49cef8eedd8133cc212794b6bec88489154 (2019-02-16, "update example in ubuntu") renamed readme/example_windows.PNG to readme/example_windows.png with the same blob and no content change.

At c125f3f6, readme/example1.PNG@4b925a27e79654cf6d586911183d4ab79570fcca was removed as a different blob. It is a sibling history event and is not part of the target image lineage. The target therefore has three tracked path/name states but one binary image state throughout; reporting them as three independent screenshot revisions would be false.

## Windows GDI+ and Qt runtime context-only boundary
Pinned CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 selects two different Windows-capable sample graphs. With QT enabled it defines BUILD_QT, compiles src/platform/qt/graphic_qt.cpp, links Qt Gui/Widgets/PrintSupport and builds Qt sample sources. Otherwise on WIN32 it defines BUILD_WIN32, compiles src/platform/gdi_win/graphic_win32.cpp plus src/samples/win32_main.cpp and links gdiplus.

src/samples/win32_main.cpp@a7c2a150c96ec2568394fa017c9bf67204006713 is a real Win32/GDI+ sample surface with native window/control/render lifecycle. graphic_win32.cpp@054860effc94bd42c7004e3bd6523f71aee059b5 and graphic_win32.h@f2ee130adfd6e61c5871d972197466bd163cf88b implement the Windows font/text/graphics adapter. src/samples/qt_main.cpp@2b089368c88d415ded50725987ecfbdc40053f49 and qt_mainwindow.cpp@ba837ed5450c5e40e07ef983e95c2243995cd500 form a separate QApplication/QWidget demo; graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5 and graphic_qt.h@777faa8d7dd18695fb4b42f4a2992c4f913c79d4 implement the Qt graphics adapter.

All are real runtime responsibilities but remain context only for this documentation screenshot row. No exact source, metadata or history record ties the committed PNG to a particular invocation of win32_main, the GDI+ backend, the Qt sample, a formula/input state, or any screenshot-generation command.

## Accepted external_microtex BUILD_QT shipping negative oracle
desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 defines external_microtex_bundled, explicitly sets PUBLIC BUILD_QT, compiles src/platform/qt/graphic_qt.cpp and packages MicroTeX res/bundled.qrc. It contains zero example_windows.png reference and does not compile win32_main.cpp or the standalone GDI+ sample.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/SourceFiles/iv/markdown/iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 directly includes platform/qt/graphic_qt.h, uses the Qt MicroTeX rendering path and initializes bundled resources. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and Telegram/build/build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 package external_microtex_bundled.rcc.

The accepted tdesktop composition is therefore runtime-positive for the Qt MicroTeX integration but target-asset-negative and standalone-GDI-sample-negative. The README screenshot is not accepted shipping evidence.

## Build / CI / bundled.qrc / githook bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa.

windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa checks out, provisions CMake/vcpkg/MSVC, configures Release/Ninja and compiles. windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 additionally installs Qt and configures -DQT=ON before compiling. Neither workflow launches a sample, executes tests, captures a screenshot, compares a golden image, uploads a screenshot or references the target PNG.

githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates readme/samples/sample_*.svg and samples.md through its separate sample path. It contains zero target PNG, screenshot or capture reference and is not a generator for this asset.

The fixed tree contains only the two tracked PNG demo assets and no screenshot/capture/generator-named path. File naming alone cannot prove absence of an external/manual generator, so the historical capture mechanism remains unknown.

## Screenshot generator / capture environment provability boundary
No checked exact evidence establishes an automated generator script, screenshot test, deterministic capture fixture, screenshot utility, command line, golden-image pipeline, formula/input snapshot, window-state script, environment lockfile, Windows image/build, GDI+ version, Qt version used for capture, compiler version, theme, fonts, display scale, monitor geometry or timestamp for the committed PNG.

Accordingly:
- Windows demo semantic meaning from README: proven.
- Exact PNG blob/dimensions and no textual capture metadata: proven.
- Same-blob rename-followed Git lineage: proven.
- Exact screenshot utility or capture environment: unproven.
- win32_main/GDI+ or Qt sample as the exact committed screenshot generator: unproven.
- Any current child CI or accepted shipping run as reproduction evidence for the historical PNG: false.

## License and resource boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README separately excludes fonts and XML under res from the blanket MIT statement. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee preserves the resource inventory; Greek/Cyrillic license text and font-specific notices remain separate layered provenance.

The committed screenshot is a documentation image under repository history, but its pixels may depict rendered fonts and Windows/theme/window chrome whose exact versions and derivation are not encoded. The root MIT statement must not be used to infer ownership/license provenance for every visually captured third-party font/theme/OS asset. No new binary is copied into Fabushi shipping.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains selected. Windows GDI+, Qt sample/backend, external_microtex and tdesktop packaging remain context owners. No WindowsScreenshotRuntime, GDIRuntime, QtRuntime, MicroTeX product root, image-golden subsystem or parallel renderer is created or closed.

## Negative oracles
- README screenshot is production implementation or release acceptance evidence: false.
- no PNG text metadata permits attribution of a screenshot utility or exact environment: false.
- three path/name states are three image revisions: false; the blob is unchanged.
- removed example1.PNG belongs to the target lineage: false.
- win32_main/GDI+ or Qt sample context proves it generated this PNG: false.
- windows-gdi/windows-qt CI reproduces or validates this PNG: false.
- accepted external_microtex ships the standalone GDI sample: false; it selects BUILD_QT.
- accepted external_microtex or bundled.qrc packages example_windows.png: false.
- static screenshot closes GDI+/Qt runtime responsibilities: false.
- project MIT settles all captured font/theme/Windows visual provenance: false.

## Coverage
Before strict202: total 16,379; unknown 16,175; unread 16,150; omitted 0; responsibilities 219.
After strict202: total 16,379; unknown 16,174; unread 16,149; omitted 0; responsibilities 220.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent e1945db2693742c9eac23086a5a79e93e9aa6ef3 was revalidated before this receipt:
- TDRP Revision 9 source authority run 40136791732: completed/success, 61/61 jobs success.
  - authority 12577815850 sha256:59f84d7fc78a89ee2aa5c7fb36c3b5c297b7899f20818c8ace7e19fc4529597c
  - recursive closure 12577311091 sha256:0037bfea9775ee456539f0395f5c5acc227681b1edef368478af37c34f31ea4f
  - recursive reachability 12577415928 sha256:5ee222757137231c754dc26cca32c5c635e78dde6c9021fecf16608ad4eabc07
  - Boost 12577691137 sha256:44c34ad11a01369980ffa8b6bfbdbafa4116eb39dcf26f40fb219251239a6d08
- Rust desktop runtime run 40136791811: completed/success, 6/6 jobs success.
  - artifact 12577855522 sha256:deda681b0bd6ba0c87b5647a8bbb1038c0977e95518aebdda563ad5b47e0e584
- Official MCP marketplace run 40136791692: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-WINDOWS-PNG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_WINDOWS_PNG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.

## Machine-readable anchors
- TDRP-R9-MICROTEX-README-WINDOWS-PNG-PROVENANCE-001
- ORA-TDRP-MICROTEX-WINDOWS-PNG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-WINDOWS-README-CONSUMER-001
- ORA-TDRP-MICROTEX-WINDOWS-PNG-NO-CAPTURE-METADATA-001
- ORA-TDRP-MICROTEX-WINDOWS-SAME-BLOB-RENAME-LINEAGE-001
- ORA-TDRP-MICROTEX-WINDOWS-GDI-QT-CONTEXT-001
- ORA-TDRP-MICROTEX-WINDOWS-BUILD-QT-SHIPPING-NEGATIVE-001
- ORA-TDRP-MICROTEX-WINDOWS-CI-BUNDLE-BYPASS-001
- ORA-TDRP-MICROTEX-WINDOWS-CAPTURE-ENVIRONMENT-BOUNDARY-001
- ORA-TDRP-MICROTEX-WINDOWS-LICENSE-VISUAL-BOUNDARY-001
- INV-TDRP-MICROTEX-WINDOWS-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-WINDOWS-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-WINDOWS-GDI-QT-CONTEXT-NOT-GENERATOR-PROOF-001
- INV-TDRP-MICROTEX-WINDOWS-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-WINDOWS-CI-NOT-SCREENSHOT-ORACLE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-WINDOWS-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-WINDOWS-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-WINDOWS-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-WINDOWS-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-WINDOWS-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-WINDOWS-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-WINDOWS-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-WINDOWS-CAPTURE-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-WINDOWS-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-WINDOWS-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-WINDOWS-LICENSE-CONFLATION-001
