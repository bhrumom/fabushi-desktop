# desktop-app/MicroTeX readme/example_debug.svg complete read — strict197

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 6c09fb90c1e966706b7e3a3033cf852c57fee0b9
- Pre-receipt tree: 0a023d235d472287ff9857af815c791f6ca5505e
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca / tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_debug.svg@7253c8c54e66c98bb95e2c2174e064cd05825662, mode 100644, 116,055 bytes, 765 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target ledger mention count: 0; independent responsibility count: 0.
- Previous sequential source: readme/example_custom_symbols.svg@45888ac3f5cf19100b9c4096eb8d38bb29d0419f (strict196).
- Next non-directory entry: readme/example_have_log.svg@33e769ade7b0c73864528d43ea8162df32e5cd1e.

## Exact SVG provenance
The accepted debug example is SVG 1.1 sized 539pt x 174pt with viewBox 0 0 539 174. It contains 65 symbol elements, 319 path elements, one rect, 103 group elements and 103 use elements. All 103 href/xlink:href values are local fragment references. The exact blob contains zero script, image, text, style, foreignObject, event-handler attribute, url() reference or external href surface.

This is a passive inline documentation visual boundary for this exact blob only. Blob or active/external-content drift requires a fresh audit.

## README direct consumer
README.md@824af04fd8607532efed3f73752b81e5093577ef has a GRAPHICS_DEBUG section stating that debug and undebug are compiled when GRAPHICS_DEBUG is defined, with the standalone option default ON. The example invokes \debug and directly embeds readme/example_debug.svg. The later \debug and \undebug section points back to GRAPHICS_DEBUG.

The SVG documents the graphical-debug example. It does not itself implement the compile gate or runtime behavior.

## Compile and runtime context-only boundary
- CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 declares option(GRAPHICS_DEBUG "If enable graphics debug" ON) and conditionally adds -DGRAPHICS_DEBUG.
- src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726 registers zero-argument debug and undebug only under #ifdef GRAPHICS_DEBUG.
- src/core/macro_impl.h@0e6f6712dc079d4f0283ba71d659e22819dc8b6f contains the inline handlers under the same gate: debug calls Formula::setDEBUG(true) and undebug calls Formula::setDEBUG(false). src/core/macro_impl.cpp@13343176be7cccdc44e2e4275a754759a6c4e4c8 has no separate debug body.
- src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 maps Formula::setDEBUG(bool) to Box::DEBUG.
- src/box/box.cpp@81e9b159c8a72e84dc0a111edeb308ca14f8cb68 initializes Box::DEBUG=false.
- src/render.cpp@e58454a430f08cefbc1330482e12c709902bc699 checks Box::DEBUG and projects selected boxes through DebugBox.
- src/box/box_single.cpp@07e2c286da5b08ffe3c926e974ccd2a08b35cb57 implements DebugBox metric copying and debug drawing.

These are context-only edges for this source row. No GraphicsDebugRuntime, second formula engine, second renderer, or source-specific product owner is introduced.

## Accepted tdesktop shipping compile-gate boundary
cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 compiles the macro/formula/box/render context sources into external_microtex_bundled. Its checked compile definitions are PUBLIC BUILD_QT and PRIVATE _USE_MATH_DEFINES; it does not define GRAPHICS_DEBUG.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 packages external_microtex_bundled.rcc.

Therefore the checked accepted shipping path compiles these runtime sources but does not establish enabled GRAPHICS_DEBUG-gated debug/undebug registration. This is a negative shipping oracle, not a claim that debug behavior is product-complete.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README".

That proves introduction provenance only. It does not identify the exact generator invocation, executable, font selection, rendering backend, script or CI step. Concrete generation provenance remains unknown.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates readme/samples/* and samples.md from the sample set. It contains zero example_debug reference. This top-level README example is outside strict187's regeneration responsibility.

## Build/package/shipping bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt, meson.build, src/QtLatex.pri, res/bundled.qrc and the three MicroTeX workflows. The accepted cmake_helpers external_microtex, td_iv and Telegram packaging manifests also contain zero direct target-asset references. Runtime source compilation is a separate context edge and does not promote this SVG into shipping composition.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML resources under res from the blanket project MIT statement. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates the resource set; Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c and res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 demonstrate layered notices.

The SVG contains path outlines but no font identity metadata. Exact-source inspection therefore cannot prove which font/resource produced each outline or collapse resource-derived obligations into project MIT.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains the selected owner. Build configuration, formula/macro/box/render debug behavior and external_microtex/td_iv shipping are context only. No new runtime or product UI owner is created.

## Negative oracles
- Static example_debug.svg is runtime implementation or release acceptance evidence: false.
- Standalone MicroTeX CMake GRAPHICS_DEBUG=ON proves accepted tdesktop shipping enables the flag: false.
- macro_impl.cpp owns the inline debug/undebug handlers: false; the pinned handlers are in macro_impl.h.
- README proves the complete parser/render lifecycle: false.
- Box::DEBUG/DebugBox creates a new Fabushi renderer owner: false.
- Same-commit history proves exact generator/font/backend: false.
- strict187 generates this top-level asset: false.
- Checked shipping composition includes this target asset: false.
- Project MIT settles all outline/font/resource derivation: false.
- This row closes build, macro, Formula, Box, render, source closure or release acceptance: false.

## Coverage
Before strict197: total 16,379; unknown 16,180; unread 16,155; omitted 0; responsibilities 214.
After strict197: total 16,379; unknown 16,179; unread 16,154; omitted 0; responsibilities 215.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent 6c09fb90c1e966706b7e3a3033cf852c57fee0b9 was revalidated before this receipt:
- TDRP Revision 9 source authority run 40127838507: completed/success, 61/61 jobs success.
  - authority 12572824676 sha256:9ba2a3efb44b39dc7e10bffeb2817820ba886161a261b4a6c01e8d0fcb0ae153
  - recursive closure 12573950609 sha256:29dbc8324995dcd5bc9a3d66980e95d8ff91bf71b79124dbfd73f79ceaa899be
  - recursive reachability 12574430340 sha256:16aabf61231cdd99513d29c17fa715a1f532e697886877d1d97d47e56bbe70a6
  - Boost 12574020354 sha256:4fb80d00ef9a333203cdf5e98a78b5a8c85f0341128a08bd48de6050456c4d8f
- Rust desktop runtime run 40127838533: completed/success, 6/6 jobs success.
  - artifact 12572699717 sha256:61a719bbd6109af72a28db29882a2c5288f614f45655ee6f5587e9155fef78c2
- Official MCP marketplace run 40127838518: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-GRAPHICS-DEBUG-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_GRAPHICS_DEBUG_SVG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.
