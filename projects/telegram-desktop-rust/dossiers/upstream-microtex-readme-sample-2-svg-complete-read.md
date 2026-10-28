# desktop-app/MicroTeX readme/samples/sample_2.svg complete read — strict206

Authority: PR #154 lease 56f34b67d2c386487de54a87ec95906dea066723; live main 16aae12ea258f6a872a46aa47824766f64e1bdca; accepted tdesktop a7f453612f4896ac04d0007a36ab0d377e5bcbcf; pinned/live MicroTeX 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. Fixed MicroTeX tree is truncated=false with 315 entries; target is mode 100644, blob 64ce25f438a4d4d88ff7b9d8947e0b3dd74257b0, size 424,975. Pre-write independent shipping/product responsibility count is 0 by exact current-ledger target path/blob search. Previous sample_1 is strict205; next is sample_3.svg@3937fe165e4c90414c4b9e18ae5b32c27784c7e6.

## Complete SVG
Full accepted blob read: 515pt x 443pt, viewBox 0 0 515 443; 368 symbol, 378 path, 447 use, 1 rect, 447 g; 368 unique ids; all 447 hrefs are local/resolved; zero text/script/image/foreignObject/style/@font-face/external href/DOCTYPE/comments. Explicit fills are #fff and none. Passive self-contained generated documentation only.

## Source identity
Current res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b and samples.md@4eb0be8a26e15f098156040366683f4e835631f6 each yield 12 bodies; index 2 is exactly equal at 1,562 characters and maps to sample_2.svg. At creation 95a54b5, SAMPLES.tex@7fea73994bec7453fc82f3cb91bab842112cd0ab and samples.md@bed457ee0da253f6ff4a37b0152be6eaaca9c20c also have the same index-2 body. Commit 63555c2 changes the overall SAMPLES.tex blob to the accepted e03f0088 version but leaves index 2 byte-identical; 08d9d6b then updates samples.md and sample_2.svg while that same index-2 body remains unchanged. The source is the complex-formula sample containing continuity, determinant, sideset, integral and metric-length expressions.

## Exact-path / co-change history
History is exhausted at exactly two commits and page 2 is empty. 95a54b5a493335679f227ed42dbe29660309d9f1 ("update samples") creates sample_2.svg as blob 40d2bb0a7c304497a994c92ed4a5d4ed76e2d1fb; parent 105a81c58cac19ba3be2d66cd88ba05bf92ebcc1 lacks the target; creation co-adds exactly samples.md plus sample_0.svg..sample_11.svg (13 paths). 08d9d6b3ca060f4923ccf0117565430c865059e9 ("update samples") changes exactly samples.md plus sample_11.svg, sample_2.svg, sample_3.svg, sample_4.svg, sample_6.svg, sample_7.svg and sample_8.svg (8 paths) and produces accepted blob 64ce25f438a4d4d88ff7b9d8947e0b3dd74257b0.

Creation and accepted sample_2 preserve 515pt x 443pt, the same 368 symbols / 378 paths / 447 uses / one rect and local-reference structure, but differ at 72 line slots, predominantly use coordinates and tiny path transform values. The source body itself did not change. The 95a54b5..08d9d6b compare spans 34 commits and 155 changed files, including formula/atom/render/font/resource/Cairo changes, so the exact reason for each coordinate delta cannot be assigned to one dependency or font.

## Generator lineage
githooks/pre-push is the same b426a77d194ac023a93acb57235635608a0d9e04 at creation, update and accepted pin. It runs ./LaTeX -headless with outputdir/prefix, textsize=14, black foreground and white background, rebuilds samples.md from SAMPLES.tex, then scour-normalizes each SVG with viewboxing, id/comment stripping, shortened ids and no indentation before staging/optional commit. samples.h@2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652 performs the percent-line sample split.

The headless/Cairo implementation itself drifts: gtkmm_main.cpp is 3b7b1c8ac7e40c43e948afe9a332d3f9780a975e at creation, e0dc57832ea7d722e5d089ab8aa87b4f079962fa at the generated update, and d491af61d05fbcc5d17c48f64e76664f91d5fce8 at the accepted pin; graphic_cairo.cpp is 5d8c3b465e37a2fba9177088eff20278da214f69 -> ae1535053ade8b3919e32fa69ee1eef34a3af681 -> 16331bfdb1d69926b7e14c7276acb3df052f40a8. The proven source chain remains Samples -> LaTeX::parse -> Cairo::SvgSurface/Cairo::Context/Graphics2D_cairo -> indexed SVG -> scour. This is source-level provenance, not an execution receipt or deterministic-byte proof.

## Build / CI / shipping negative oracles
Pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5, QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa do not ship sample_2.svg. ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 have zero target regeneration/byte-compare reference.

Accepted shipping is separate: desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds external_microtex_bundled with BUILD_QT, graphic_qt.cpp and bundled.qrc; tdesktop td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 carry external_microtex_bundled.rcc. None references sample_2.svg, readme/samples or SAMPLES.tex. The asset is documentation, not shipping content.

## Reproducibility / font / resource / license boundary
Not proven: installed-hook dispatch at either history point, exact historical executable/compiler/GTK/Cairo/Pango/Freetype/Fontconfig/OS/locale/machine, exact scour version, raw pre-scour bytes, exact font selected for each finalized outline, or bit-for-bit regeneration. The two committed outputs with identical index-2 source and drifting renderer/resource code are positive evidence that source identity alone is insufficient for byte reproducibility.

MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT for the project scope, while README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML resources under res from blanket MIT. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates font/resource inputs; separate license files include Knuth_License@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, dsrom@35e3ff2160fe16cb26f7830f936557ebc44c3e1c, OFL@6836a0be000498319aebcc92a172159b6f80bb52 and Greek/Cyrillic LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7. Finalized SVG paths do not preserve exact per-outline font identity, so layered derivation/licensing remains bounded rather than inferred.

## Existing-owner-first / coverage
Selected owner is existing TDRP recursive dependency documentation/tooling/license provenance governance. No SampleSvgRuntime, duplicate renderer/font owner, CI gate, shipping root or UI root is created. Before: unknown 16171, unread 16146, responsibilities 223. After: unknown 16170, unread 16145, responsibilities 224; omitted 0. baseline_identity_accepted=false; source_closure_ready=false; release acceptance not accepted.

## Parent evidence
56f34b67d2c386487de54a87ec95906dea066723: TDRP 40145378248 completed/success 61/61; authority 12580161997 sha256:d08e55bd80a87f8f7937cae7dc9da98a3761fc1ce021599c8c047e76fcf21a8f; closure 12580137518 sha256:b698bbd8cafa960988f72bcaf06e3027926a9e5bef1aef952e94ea8da69b8fc8; reachability 12581165645 sha256:c55e7a029d05f5b943d1ffa8c23c2c90a7ae92681bc554217feb0e534a37c61f; Boost 12580182130 sha256:7dadaf2bde51c164374e94dbc1a3cc9b748a1ed7a7193e5d5f79fe5f396c1021. Rust 40145378277 6/6, artifact 12580925588 sha256:8155e83d82448ff7d68997e23b909e8a82583493125e8773a2d855a20dda1e3b. Official MCP 40145378249 1/1. Receipt-head evidence must be fresh.

Responsibility TDRP-R9-MICROTEX-README-SAMPLE-2-SVG-PROVENANCE-001; target symbol TDRP_MICROTEX_README_SAMPLE_2_SVG_PROVENANCE_001; gates G-FILE/G-TRACEABILITY/G-EVIDENCE.

## Machine-readable traceability anchors
- TDRP-R9-MICROTEX-README-SAMPLE-2-SVG-PROVENANCE-001
- TDRP_MICROTEX_README_SAMPLE_2_SVG_PROVENANCE_001
- ORA-TDRP-MICROTEX-SAMPLE-2-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-SAMPLE-2-SVG-PASSIVE-LOCAL-CONTENT-001
- ORA-TDRP-MICROTEX-SAMPLE-2-SOURCE-BLOCK-IDENTITY-001
- ORA-TDRP-MICROTEX-SAMPLE-2-EXACT-PATH-HISTORY-001
- ORA-TDRP-MICROTEX-SAMPLE-2-OUTPUT-DRIFT-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-2-HISTORICAL-PREPUSH-001
- ORA-TDRP-MICROTEX-SAMPLE-2-HISTORICAL-HEADLESS-CAIRO-001
- ORA-TDRP-MICROTEX-SAMPLE-2-BUILD-CI-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-SAMPLE-2-REPRODUCIBILITY-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-2-LICENSE-FONT-BOUNDARY-001
- INV-TDRP-MICROTEX-SAMPLE-2-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-SAMPLE-2-GENERATOR-SOURCE-NOT-EXECUTION-PROOF-001
- INV-TDRP-MICROTEX-SAMPLE-2-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-SAMPLE-SIBLINGS-REMAIN-INDEPENDENT-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-2-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-2-GENERATOR-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-2-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-2-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-OUTPUT-DRIFT-CAUSE-CONFLATION-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-SIBLING-PREMATURE-CLOSURE-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-2-LICENSE-CONFLATION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
