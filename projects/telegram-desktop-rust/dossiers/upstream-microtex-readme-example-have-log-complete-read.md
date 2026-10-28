# desktop-app/MicroTeX readme/example_have_log.svg complete read — strict198

## Authority
- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: e5ba2e675bedf236521dfb5555cc297ddb963615
- Pre-receipt tree: f3cb9cdee3812be8b4112e89371a4db530bc328f
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca / tree 82126ec4a712fca352a49d6a3caae70c7d21ca8c.
- Accepted tdesktop: a7f453612f4896ac04d0007a36ab0d377e5bcbcf / tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- MicroTeX pin: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f / tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2.
- cmake_helpers pin: 7ceca3019b4750d7a5acb2de0faffd523ee14095 / tree f5c386cb34208309aae521187611fac4a2045577.
- Target: readme/example_have_log.svg@33e769ade7b0c73864528d43ea8162df32e5cd1e, mode 100644, 15,276 bytes, 80 line slots.
- Pinned MicroTeX tree: truncated=false, 315 entries / 279 blobs.
- Pre-write exact target ledger/inventory/lock mention count: 0; independent responsibility count: 0.
- Previous sequential source: readme/example_debug.svg@7253c8c54e66c98bb95e2c2174e064cd05825662 (strict197).
- Next non-directory entry: readme/example_keep_trying.svg@e2ed234fb25c82c47f6852e0c21edfa341e0e38d.

## Exact SVG provenance
The accepted HAVE_LOG example is SVG 1.1 sized 184pt x 46pt with viewBox 0 0 184 46. It contains 12 symbol elements, 13 path elements, one rect, 12 group elements and 12 use elements. All 12 href/xlink:href values are local fragment references. The exact blob contains zero script, image, text, style, foreignObject, event-handler attribute, url() reference or external href surface.

This is a passive inline documentation visual boundary for this exact blob only. Blob or active/external-content drift requires a fresh audit.

## README direct consumer
README.md@824af04fd8607532efed3f73752b81e5093577ef has a HAVE_LOG section. It describes diagnostic output such as symbol parse results and generated box trees, calls the standalone option default ON, states release-mode builds disable it, and directly embeds readme/example_have_log.svg. A later build example explicitly passes -DHAVE_LOG=OFF.

The SVG documents an example of diagnostic output. It does not itself implement compile configuration, log emission or shipping behavior.

## Standalone CMake and config gate
CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 declares option(HAVE_LOG "If enable log" ON) and conditionally adds -DHAVE_LOG.

src/config.h@b43a3954ac3da819c3c31ec9d9e79c53f5b08d2d defines CLATEX_DEBUG when _DEBUG, DEBUG or __DEBUG__ is defined, or when NDEBUG is absent. Later, when CLATEX_DEBUG is absent, it explicitly undefines HAVE_LOG. Therefore the standalone CMake default is not by itself the effective runtime logging contract; config.h provides an additional compile-time gate.

## Logging runtime context-only boundary
src/utils/log.h@447ea39eac7c6c59b4a50f6548b74514e575a51e defines __dbg generally, while __log is defined as std::cout only under HAVE_LOG.

src/core/parser.cpp@b20e0bc1bf2604f7cd67868d8bb131fef525c09c has a HAVE_LOG-gated Unicode-block diagnostic through __log. src/core/formula.cpp@c4781c1da1369d71f4f303cd341709dda5ecb303 has HAVE_LOG-gated initialization diagnostics through __dbg and symbol-mapping dumps through __log.

These are context-only edges for this source row. No LoggingRuntime, DiagnosticsRuntime, second parser, or source-specific product owner is introduced or closed.

## Accepted tdesktop shipping negative oracle
cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 compiles the pinned parser.cpp and formula.cpp among the external_microtex_bundled runtime sources. Its exact file has zero HAVE_LOG occurrence; the checked target compile definitions are BUILD_QT and _USE_MATH_DEFINES.

Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 packages external_microtex_bundled.rcc. Both exact files have zero HAVE_LOG occurrence and zero direct target-asset reference.

Therefore the checked accepted shipping path compiles the logging-capable source context but does not establish HAVE_LOG-enabled diagnostics. This is a negative shipping oracle, not a claim that diagnostic behavior is globally impossible or production-complete.

## Exact-path history and generation provenance
Exact-path history through the accepted MicroTeX pin contains one change commit: 63555c2e902a42bfad457d3d58a4a24c5c51c53e, parent c85c84194835439319981a38485dc3fa4e8d4236, authored 2019-02-25, message "new version README".

That proves introduction provenance only. It does not identify the exact generator invocation, executable, font selection, rendering backend, script or CI step. Concrete generation provenance remains unknown.

## strict187 de-dup
githooks/pre-push@b426a77d194ac023a93acb57235635608a0d9e04 regenerates readme/samples/* and samples.md from the sample set. It contains zero example_have_log.svg reference. This top-level README example is outside strict187's regeneration responsibility.

## Build, CI, bundled resource and shipping bypass
Direct target-asset references are zero in pinned MicroTeX CMakeLists.txt, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7, res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa and the three pinned MicroTeX workflows ubuntu-gtk@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838. The accepted external_microtex/td_iv/Telegram packaging manifests also contain zero direct target-asset references.

Runtime source compilation is a separate context edge and does not promote this SVG into shipping composition.

## License/font/resource-derived boundary
MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT. README explicitly excludes fonts and XML resources under res from the blanket project MIT statement. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates the resource set; Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c and res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7 demonstrate layered notices.

The SVG contains path outlines but no font identity metadata. Exact-source inspection therefore cannot prove which font/resource produced each outline or collapse resource-derived obligations into project MIT.

## Existing-owner-first disposition
Existing TDRP recursive documentation/source/license provenance governance remains the selected owner. Build configuration, CLATEX_DEBUG/HAVE_LOG gating, log macros, parser/formula diagnostics and external_microtex/td_iv shipping are context only. No new runtime or product UI owner is created.

## Negative oracles
- Static example_have_log.svg is runtime implementation or release acceptance evidence: false.
- Standalone MicroTeX CMake HAVE_LOG=ON proves effective logging in every build: false.
- Standalone CMake HAVE_LOG=ON proves accepted tdesktop shipping enables logging: false.
- config.h CLATEX_DEBUG/NDEBUG gating can be omitted from the effective contract: false.
- __dbg and HAVE_LOG-gated __log have the same definition gate: false.
- parser/formula diagnostics create a new Fabushi LoggingRuntime owner: false.
- Same-commit history proves exact generator/font/backend: false.
- strict187 generates this top-level asset: false.
- Checked shipping composition includes this target asset: false.
- Project MIT settles all outline/font/resource derivation: false.
- This row closes build, diagnostics, parser, formula, source closure or release acceptance: false.

## Coverage
Before strict198: total 16,379; unknown 16,179; unread 16,154; omitted 0; responsibilities 215.
After strict198: total 16,379; unknown 16,178; unread 16,153; omitted 0; responsibilities 216.
baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence
Parent e5ba2e675bedf236521dfb5555cc297ddb963615 was revalidated before this receipt:
- TDRP Revision 9 source authority run 40129796087: completed/success, 61/61 jobs success.
  - authority 12573858690 sha256:8f069806358053aa1758f8e6f7bb5a57ca40c28e95b8e236843b1760bea08ede
  - recursive closure 12574738611 sha256:deebc2b648ffbe3e655b48e139e30e558a57e8bf95798d028bba9aeb2fb1be36
  - recursive reachability 12574867200 sha256:1402a2cbb95c26f0c6b269add7b29a7e545ef76214c7720954ab5371ab7bec6e
  - Boost 12573912537 sha256:b8a5f05dfbf43a65167ca464df2d81f932b1eaef3a74ec12208a199cf4e49c1f
- Rust desktop runtime run 40129796060: completed/success, 6/6 jobs success.
  - artifact 12574558352 sha256:7dfd1c72f9a06389628508e9a5908b28af4f7b0a9cac67ea5f5ff998714caa07
- Official MCP marketplace run 40129796061: completed/success, 1/1 job success.

These are historical parent evidence after the receipt; receipt exact-head requires independent validation.

## Traceability
- Responsibility: TDRP-R9-MICROTEX-README-HAVE-LOG-SVG-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_README_HAVE_LOG_SVG_PROVENANCE_001
- Gates remain G-FILE, G-TRACEABILITY and G-EVIDENCE.

## Machine-readable anchors
- TDRP-R9-MICROTEX-README-HAVE-LOG-SVG-PROVENANCE-001
- ORA-TDRP-MICROTEX-HAVE-LOG-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-HAVE-LOG-SVG-INLINE-LOCAL-REFERENCE-001
- ORA-TDRP-MICROTEX-HAVE-LOG-README-CONSUMER-001
- ORA-TDRP-MICROTEX-HAVE-LOG-CMAKE-CONFIG-GATE-001
- ORA-TDRP-MICROTEX-HAVE-LOG-LOG-HEADER-CONTEXT-001
- ORA-TDRP-MICROTEX-HAVE-LOG-PARSER-FORMULA-CONTEXT-001
- ORA-TDRP-MICROTEX-HAVE-LOG-HISTORY-GENERATOR-BOUNDARY-001
- ORA-TDRP-MICROTEX-HAVE-LOG-STRICT187-DEDUP-001
- ORA-TDRP-MICROTEX-HAVE-LOG-TDESKTOP-SHIPPING-COMPILE-GATE-001
- ORA-TDRP-MICROTEX-HAVE-LOG-LICENSE-FONT-RESOURCE-BOUNDARY-001
- INV-TDRP-MICROTEX-HAVE-LOG-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-HAVE-LOG-NO-PARALLEL-RUNTIME-OWNER-001
- INV-TDRP-MICROTEX-HAVE-LOG-STANDALONE-CMAKE-NOT-SHIPPING-DEFINE-001
- INV-TDRP-MICROTEX-HAVE-LOG-NOT-STRICT187-SAMPLE-OUTPUT-001
- INV-TDRP-MICROTEX-HAVE-LOG-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-HAVE-LOG-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-HAVE-LOG-README-CONTEXT-001
- CONTRACT-TDRP-MICROTEX-HAVE-LOG-RUNTIME-CONTEXT-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-HAVE-LOG-CONSUMER-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-HAVE-LOG-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-EXTERNAL-SVG-CONTENT-DRIFT-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-GENERATOR-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-RUNTIME-CONFLATION-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-SHIPPING-GATE-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-HAVE-LOG-LICENSE-CONFLATION-001
