# desktop-app/MicroTeX readme/samples/sample_3.svg complete read — strict207

Authority: PR #154 lease 3249f5918b474fbddc2c0037a24307afa2084b48; live main 16aae12ea258f6a872a46aa47824766f64e1bdca; accepted tdesktop a7f453612f4896ac04d0007a36ab0d377e5bcbcf; pinned/live MicroTeX 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f. Fixed MicroTeX tree is truncated=false with 315 entries; target is mode 100644, blob 3937fe165e4c90414c4b9e18ae5b32c27784c7e6, size 459,845. Pre-write independent shipping/product responsibility count is 0 by exact current-ledger target path/blob search. Previous sample_2 is strict206; next is sample_4.svg@77d65ed8eabdd51063b6a54104f6d5c96d595a7d.

## Complete SVG
Full accepted blob read: 460pt x 494pt, viewBox 0 0 460 494; 370 symbol, 379 path, 508 use, 1 rect, 509 g; 371 unique ids; all 508 hrefs are local/resolved; zero text/script/image/foreignObject/style/@font-face/external href/DOCTYPE/comments. The accepted content has 2,654 split-line slots including the terminal-LF slot. Passive self-contained generated documentation only.

The creation state at 95a54b5 is blob 8796bae4d676c7fb662aa94fabe4f084fcc93aae, 459,855 bytes, and preserves exactly the same dimensions, element cardinalities and local-reference topology. Creation and accepted content differ at exactly 64 line slots, predominantly use coordinates.

## Index-3 source transition and fatalIfCmdConflict
The source is not stable across the two generated states. At creation 95a54b5, res/SAMPLES.tex@7fea73994bec7453fc82f3cb91bab842112cd0ab and samples.md@bed457ee0da253f6ff4a37b0152be6eaaca9c20c have exact-equal index-3 bodies of 2,232 characters with no fatalIfCmdConflict line.

Commit 63555c2e902a42bfad457d3d58a4a24c5c51c53e changes res/SAMPLES.tex to e03f0088e3fba1d225b90d06d586f32c5f70310b and its index-3 body to 2,259 characters by adding exactly `\fatalIfCmdConflict{false}` after the color definitions. samples.md remains the older bed457ee blob and 2,232-character body at that commit, so the source and generated-document manifest are deliberately out of sync at this point. Commit 08d9d6b then updates samples.md to 4eb0be8a26e15f098156040366683f4e835631f6 and sample_3.svg; at 08d9d6b and the accepted pin, SAMPLES.tex and samples.md are exact-equal at 2,259 characters. The real source delta is therefore +27 characters.

The command itself predates that source insertion. At creation, macro_def.cpp@0d66ad682283f594cc92fd6f2ce4be04d2768332 already registers fatalIfCmdConflict, macro_impl.h@2676b16587088ca2c37a8de3d852bb1fe1422b8e toggles NewCommandMacro::_errIfConflict, and macro.cpp@c556062a8f792241919344f0f47296bde84cfd87 guards new/renew conflicts with that flag. By 63555c2/08d9d6b, macro_impl.h@5d91cd3fc101cc6177d912d6a10ce9a1ba1f701c implements it as args[1] == L"true" and macro.cpp@7c73335b034b6d7fb2b4050c9212c724244a1715 retains conflict checks. Current accepted macro.cpp@f7d09b90eea98e191ee9226f50c79b85b91b35f4 additionally protects built-ins independently and resets the formula-settable flag to true. This is bounded runtime context: strict207 does not close NewCommandMacro or parser responsibility.

## Exact-path / co-change history
History is exhausted at exactly two target-path commits and page 2 is empty. 95a54b5a493335679f227ed42dbe29660309d9f1 ("update samples") creates sample_3.svg as blob 8796bae4d676c7fb662aa94fabe4f084fcc93aae; parent 105a81c58cac19ba3be2d66cd88ba05bf92ebcc1 returns target-not-found. Creation co-adds exactly samples.md plus sample_0.svg..sample_11.svg (13 paths). 08d9d6b3ca060f4923ccf0117565430c865059e9 ("update samples") changes exactly samples.md plus sample_11.svg, sample_2.svg, sample_3.svg, sample_4.svg, sample_6.svg, sample_7.svg and sample_8.svg (8 paths), producing accepted blob 3937fe165e4c90414c4b9e18ae5b32c27784c7e6.

The 95a54b5..08d9d6b compare spans 34 commits and 155 changed files, including the +1 SAMPLES.tex source line and formula/atom/render/font/resource/Cairo changes. The +27-character source-policy change is exact evidence and must not be erased as in the strict206 unchanged-source case; equally, it is not evidence that the single source line caused every one of the 64 vector-coordinate deltas. Sole-cause attribution remains unproven.

## Generator lineage
githooks/pre-push is the same b426a77d194ac023a93acb57235635608a0d9e04 at creation, update and accepted pin. It runs ./LaTeX -headless with outputdir/prefix and the established textsize=14, black foreground and white background, rebuilds samples.md from SAMPLES.tex, then scour-normalizes each SVG with viewboxing/id-comment normalization before staging/optional commit. samples.h is 633cdcc8163f34914d80eb172e1a049f924b81a7 at creation/update and 2e8bcd0d3d8e2461ad12bce4e9e4763124b6b652 current; it provides percent-line sample splitting.

The headless/Cairo implementation drifts: gtkmm_main.cpp is 3b7b1c8ac7e40c43e948afe9a332d3f9780a975e at creation, e0dc57832ea7d722e5d089ab8aa87b4f079962fa at generated update, and d491af61d05fbcc5d17c48f64e76664f91d5fce8 at accepted pin; graphic_cairo.cpp is 5d8c3b465e37a2fba9177088eff20278da214f69 -> ae1535053ade8b3919e32fa69ee1eef34a3af681 -> 16331bfdb1d69926b7e14c7276acb3df052f40a8. The proven source chain remains Samples -> LaTeX::parse -> Cairo::SvgSurface/Cairo::Context/Graphics2D_cairo -> indexed SVG -> scour. This is source-level provenance, not an installed-hook execution receipt or deterministic-byte proof.

## Build / CI / shipping negative oracles
Pinned MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9, QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 and bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa have zero sample_3/readme-samples/SAMPLES shipping reference. meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 has zero sample_3/readme-samples reference but does install res/SAMPLES.tex as source/demo data; that does not promote the generated readme/samples/sample_3.svg into a shipping resource. ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764, windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa and windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 have zero target regeneration/byte-compare reference.

Accepted shipping is separate: desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds external_microtex_bundled with BUILD_QT, graphic_qt.cpp and bundled.qrc; tdesktop td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex; Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 and build_mac.py@881a1c212486976b318777b7f6e5c944da2bd5d3 carry external_microtex_bundled.rcc. None references sample_3.svg, readme/samples or SAMPLES.tex. The generated SVG is documentation, not accepted shipping content.

## Reproducibility / font / resource / license boundary
Not proven: installed-hook dispatch at either generated history point, exact historical executable/compiler/GTK/Cairo/Pango/Freetype/Fontconfig/OS/locale/machine, exact scour version, raw pre-scour bytes, exact font selected for each finalized outline, or bit-for-bit regeneration. Here both the source and the renderer/resource environment evolve, so the committed delta does not isolate either factor.

MicroTeX LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2 is MIT for the project scope, while README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML resources under res from blanket MIT. res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee enumerates font/resource inputs. Separate inputs include Knuth_License@5b03bf8a533e66c79d113b82ba95586f8c7af3b8, dsrom@35e3ff2160fe16cb26f7830f936557ebc44c3e1c, OFL@6836a0be000498319aebcc92a172159b6f80bb52 and Greek/Cyrillic GPLv3 LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7. Finalized SVG paths do not preserve exact per-outline font identity, so layered derivation/licensing remains bounded rather than inferred.

## Existing-owner-first / coverage
Selected owner is existing TDRP recursive dependency documentation/tooling/license provenance governance. NewCommandMacro/fatalIfCmdConflict, formula/render, GTK/Cairo/font and accepted shipping remain context owners and are not closed by this generated document. No SampleSvgRuntime, duplicate macro/parser/renderer/font owner, CI gate, shipping root or UI root is created. Before: unknown 16170, unread 16145, responsibilities 224. After: unknown 16169, unread 16144, responsibilities 225; omitted 0. baseline_identity_accepted=false; source_closure_ready=false; release acceptance not accepted.

## Parent evidence
3249f5918b474fbddc2c0037a24307afa2084b48: TDRP 40146766145 completed/success 61/61; authority 12581047983 sha256:e6bacfc101b7eb155c0a44b30daa3dd73848f238b0caae0898270d5ac970f2c7; closure 12581282724 sha256:628b1f64f004af289acff0b4cd109993399916d869442691e6ad57adbe06350e; reachability 12580623249 sha256:9ce6a2dc8240875096b8242959154f96bc0990afd51941d47a577e6f7b381cf3; Boost 12580144141 sha256:79aba74eed8ccce52a0f16f76dcb74d63512772bb67689daf6c9039dd57fa97d. Rust 40146766202 completed/success 6/6, artifact 12581217560 sha256:59a406e8d5175b1cc39d7dbd11b418d8c0933ba9ece52552c8858145da906026. Official MCP 40146766170 completed/success 1/1. Receipt-head evidence must be fresh.

Responsibility TDRP-R9-MICROTEX-README-SAMPLE-3-SVG-PROVENANCE-001; target symbol TDRP_MICROTEX_README_SAMPLE_3_SVG_PROVENANCE_001; gates G-FILE/G-TRACEABILITY/G-EVIDENCE.

## Machine-readable traceability anchors
- TDRP-R9-MICROTEX-README-SAMPLE-3-SVG-PROVENANCE-001
- TDRP_MICROTEX_README_SAMPLE_3_SVG_PROVENANCE_001
- ORA-TDRP-MICROTEX-SAMPLE-3-SVG-EXACT-SOURCE-001
- ORA-TDRP-MICROTEX-SAMPLE-3-SVG-PASSIVE-LOCAL-CONTENT-001
- ORA-TDRP-MICROTEX-SAMPLE-3-SOURCE-BLOCK-TRANSITION-001
- ORA-TDRP-MICROTEX-SAMPLE-3-FATAL-CMD-CONFLICT-SOURCE-CHANGE-001
- ORA-TDRP-MICROTEX-SAMPLE-3-EXACT-PATH-HISTORY-001
- ORA-TDRP-MICROTEX-SAMPLE-3-OUTPUT-DELTA-CAUSAL-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-3-HISTORICAL-PREPUSH-001
- ORA-TDRP-MICROTEX-SAMPLE-3-HISTORICAL-HEADLESS-CAIRO-001
- ORA-TDRP-MICROTEX-SAMPLE-3-BUILD-CI-SHIPPING-BYPASS-001
- ORA-TDRP-MICROTEX-SAMPLE-3-REPRODUCIBILITY-BOUNDARY-001
- ORA-TDRP-MICROTEX-SAMPLE-3-LICENSE-FONT-BOUNDARY-001
- INV-TDRP-MICROTEX-SAMPLE-3-DOCUMENTATION-NOT-RUNTIME-AUTHORITY-001
- INV-TDRP-MICROTEX-SAMPLE-3-GENERATOR-SOURCE-NOT-EXECUTION-PROOF-001
- INV-TDRP-MICROTEX-SAMPLE-3-FATAL-CMD-CONTEXT-NOT-RUNTIME-CLOSURE-001
- INV-TDRP-MICROTEX-SAMPLE-3-NOT-SHIPPING-RESOURCE-001
- INV-TDRP-MICROTEX-SAMPLE-SIBLINGS-REMAIN-INDEPENDENT-001
- INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-3-PINNED-SOURCE-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-3-SOURCE-TRANSITION-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-3-GENERATOR-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-3-SHIPPING-BOUNDARY-001
- CONTRACT-TDRP-MICROTEX-SAMPLE-3-EXACT-HEAD-EVIDENCE-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-SOURCE-DRIFT-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-HISTORY-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-SOURCE-CHANGE-CAUSAL-CONFLATION-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-MACRO-RUNTIME-PREMATURE-CLOSURE-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-ENVIRONMENT-MISATTRIBUTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-SIBLING-PREMATURE-CLOSURE-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-SHIPPING-PROMOTION-001
- FAULT-TDRP-MICROTEX-SAMPLE-3-LICENSE-CONFLATION-001
- G-FILE
- G-TRACEABILITY
- G-EVIDENCE
